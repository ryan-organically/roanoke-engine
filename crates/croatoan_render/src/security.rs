//! Security Module for GPU Pipeline Protection
//!
//! Provides comprehensive security hardening for GPU operations:
//! - Buffer overflow prevention
//! - Memory exhaustion protection
//! - Input validation and sanitization
//! - Shader security checks
//! - Resource limit enforcement

use std::sync::atomic::{AtomicUsize, Ordering};

// =============================================================================
// GLOBAL MEMORY TRACKING
// =============================================================================

/// Global GPU memory usage tracker (in bytes)
static GPU_MEMORY_USED: AtomicUsize = AtomicUsize::new(0);

/// Maximum allowed GPU memory usage (512 MB default)
const MAX_GPU_MEMORY_BYTES: usize = 512 * 1024 * 1024;

/// Memory allocation thresholds
pub mod limits {
    /// Maximum single buffer allocation (64 MB)
    pub const MAX_BUFFER_SIZE: usize = 64 * 1024 * 1024;
    /// Maximum texture size (4096x4096 RGBA = 64 MB)
    pub const MAX_TEXTURE_PIXELS: usize = 4096 * 4096;
    /// Maximum shader source length (1 MB - prevents shader bombs)
    pub const MAX_SHADER_SIZE: usize = 1024 * 1024;
    /// Maximum vertex count per mesh
    pub const MAX_VERTICES: usize = 2_000_000;
    /// Maximum index count per mesh
    pub const MAX_INDICES: usize = 12_000_000;
    /// Maximum instance count per draw call
    pub const MAX_INSTANCES: usize = 100_000;
    /// Maximum bind groups per pipeline
    pub const MAX_BIND_GROUPS: usize = 8;
    /// Maximum uniforms size (64 KB)
    pub const MAX_UNIFORM_SIZE: usize = 64 * 1024;
}

// =============================================================================
// SECURITY ERROR TYPES
// =============================================================================

/// Security violation types
#[derive(Debug, Clone, PartialEq)]
pub enum SecurityError {
    /// Attempted to allocate too much GPU memory
    MemoryExhaustion { requested: usize, available: usize },
    /// Single buffer too large
    BufferTooLarge { size: usize, max: usize },
    /// Texture dimensions too large
    TextureTooLarge { width: u32, height: u32 },
    /// Shader source too large or suspicious
    ShaderSuspicious { reason: String },
    /// Index buffer references invalid vertex
    InvalidIndex { index: u32, max_vertex: usize },
    /// Vertex data contains invalid floats
    InvalidVertexData { reason: String },
    /// Too many draw instances
    TooManyInstances { count: usize, max: usize },
    /// Resource limit exceeded
    ResourceLimitExceeded { resource: String, count: usize, max: usize },
    /// Potential denial of service detected
    PotentialDoS { description: String },
    /// Invalid or malformed input
    MalformedInput { description: String },
}

impl std::fmt::Display for SecurityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MemoryExhaustion { requested, available } => {
                write!(f, "GPU memory exhaustion: requested {} bytes, only {} available",
                       requested, available)
            }
            Self::BufferTooLarge { size, max } => {
                write!(f, "Buffer too large: {} bytes (max: {})", size, max)
            }
            Self::TextureTooLarge { width, height } => {
                write!(f, "Texture too large: {}x{}", width, height)
            }
            Self::ShaderSuspicious { reason } => {
                write!(f, "Suspicious shader: {}", reason)
            }
            Self::InvalidIndex { index, max_vertex } => {
                write!(f, "Invalid index {} (max vertex: {})", index, max_vertex)
            }
            Self::InvalidVertexData { reason } => {
                write!(f, "Invalid vertex data: {}", reason)
            }
            Self::TooManyInstances { count, max } => {
                write!(f, "Too many instances: {} (max: {})", count, max)
            }
            Self::ResourceLimitExceeded { resource, count, max } => {
                write!(f, "Resource limit exceeded: {} has {} (max: {})", resource, count, max)
            }
            Self::PotentialDoS { description } => {
                write!(f, "Potential DoS detected: {}", description)
            }
            Self::MalformedInput { description } => {
                write!(f, "Malformed input: {}", description)
            }
        }
    }
}

impl std::error::Error for SecurityError {}

pub type SecurityResult<T> = Result<T, SecurityError>;

// =============================================================================
// MEMORY TRACKING
// =============================================================================

/// Track GPU memory allocation
pub fn track_allocation(bytes: usize) -> SecurityResult<()> {
    let current = GPU_MEMORY_USED.load(Ordering::Relaxed);
    if current + bytes > MAX_GPU_MEMORY_BYTES {
        return Err(SecurityError::MemoryExhaustion {
            requested: bytes,
            available: MAX_GPU_MEMORY_BYTES.saturating_sub(current),
        });
    }
    GPU_MEMORY_USED.fetch_add(bytes, Ordering::Relaxed);
    Ok(())
}

/// Release tracked GPU memory
pub fn release_allocation(bytes: usize) {
    GPU_MEMORY_USED.fetch_sub(bytes.min(GPU_MEMORY_USED.load(Ordering::Relaxed)), Ordering::Relaxed);
}

/// Get current GPU memory usage
pub fn get_gpu_memory_usage() -> usize {
    GPU_MEMORY_USED.load(Ordering::Relaxed)
}

/// Reset GPU memory tracking (for testing or recovery)
pub fn reset_memory_tracking() {
    GPU_MEMORY_USED.store(0, Ordering::Relaxed);
}

// =============================================================================
// BUFFER VALIDATION
// =============================================================================

/// Validate buffer allocation before creating
pub fn validate_buffer_allocation(size: usize) -> SecurityResult<()> {
    if size > limits::MAX_BUFFER_SIZE {
        return Err(SecurityError::BufferTooLarge {
            size,
            max: limits::MAX_BUFFER_SIZE,
        });
    }
    track_allocation(size)
}

/// Validate vertex buffer data
pub fn validate_vertex_data(
    positions: &[[f32; 3]],
    check_bounds: bool,
) -> SecurityResult<()> {
    // Check count
    if positions.len() > limits::MAX_VERTICES {
        return Err(SecurityError::ResourceLimitExceeded {
            resource: "vertices".to_string(),
            count: positions.len(),
            max: limits::MAX_VERTICES,
        });
    }

    if check_bounds {
        // Check for invalid floats and extreme values
        for (i, pos) in positions.iter().enumerate() {
            for (j, &v) in pos.iter().enumerate() {
                if !v.is_finite() {
                    return Err(SecurityError::InvalidVertexData {
                        reason: format!("Non-finite value at vertex {} component {}", i, j),
                    });
                }
                // Check for extreme values that might cause GPU issues
                if v.abs() > 1e10 {
                    return Err(SecurityError::InvalidVertexData {
                        reason: format!("Extreme value {} at vertex {} component {}", v, i, j),
                    });
                }
            }
        }
    }

    Ok(())
}

/// Validate index buffer data
pub fn validate_index_data(
    indices: &[u32],
    vertex_count: usize,
) -> SecurityResult<()> {
    // Check count
    if indices.len() > limits::MAX_INDICES {
        return Err(SecurityError::ResourceLimitExceeded {
            resource: "indices".to_string(),
            count: indices.len(),
            max: limits::MAX_INDICES,
        });
    }

    // Validate indices reference valid vertices
    // For large buffers, spot-check to avoid O(n) overhead
    let check_count = if indices.len() > 10000 {
        indices.len() / 100 // Check 1% for large buffers
    } else {
        indices.len() // Check all for small buffers
    };

    for i in 0..check_count {
        let idx = if check_count < indices.len() {
            // Spot check: sample evenly across the buffer
            (i * indices.len()) / check_count
        } else {
            i
        };

        if indices[idx] as usize >= vertex_count {
            return Err(SecurityError::InvalidIndex {
                index: indices[idx],
                max_vertex: vertex_count,
            });
        }
    }

    Ok(())
}

/// Validate instance count
pub fn validate_instance_count(count: usize) -> SecurityResult<()> {
    if count > limits::MAX_INSTANCES {
        return Err(SecurityError::TooManyInstances {
            count,
            max: limits::MAX_INSTANCES,
        });
    }
    Ok(())
}

// =============================================================================
// SHADER VALIDATION
// =============================================================================

/// Suspicious shader patterns that might indicate an attack
const SUSPICIOUS_PATTERNS: &[(&str, &str)] = &[
    ("while(true)", "Infinite loop"),
    ("for(;;)", "Infinite loop"),
    ("loop {", "Unbounded loop"),
    ("discard", "Excessive fragment discard (potential performance attack)"),
    // Add more patterns as needed
];

/// Maximum loop iterations allowed in shader
const MAX_SHADER_LOOP_ITERATIONS: usize = 10000;

/// Validate shader source for security issues
pub fn validate_shader_source(source: &str) -> SecurityResult<()> {
    // Size check
    if source.len() > limits::MAX_SHADER_SIZE {
        return Err(SecurityError::ShaderSuspicious {
            reason: format!("Shader source too large: {} bytes (max: {})",
                          source.len(), limits::MAX_SHADER_SIZE),
        });
    }

    // Check for empty shader
    if source.trim().is_empty() {
        return Err(SecurityError::ShaderSuspicious {
            reason: "Empty shader source".to_string(),
        });
    }

    // Pattern checks (basic - not a full parser)
    let source_lower = source.to_lowercase();

    // Check for suspicious patterns
    for (pattern, reason) in SUSPICIOUS_PATTERNS {
        if source_lower.contains(&pattern.to_lowercase()) {
            log::warn!("Shader contains potentially dangerous pattern: {} ({})", pattern, reason);
            // Don't fail, just warn - these might be legitimate
        }
    }

    // Check for excessive complexity (rough heuristic)
    let brace_count = source.chars().filter(|&c| c == '{').count();
    if brace_count > 500 {
        return Err(SecurityError::ShaderSuspicious {
            reason: format!("Shader too complex: {} blocks (potential shader bomb)", brace_count),
        });
    }

    Ok(())
}

// =============================================================================
// TEXTURE VALIDATION
// =============================================================================

/// Validate texture dimensions
pub fn validate_texture_dimensions(width: u32, height: u32) -> SecurityResult<()> {
    let pixels = width as usize * height as usize;
    if pixels > limits::MAX_TEXTURE_PIXELS {
        return Err(SecurityError::TextureTooLarge { width, height });
    }
    Ok(())
}

// =============================================================================
// INPUT SANITIZATION
// =============================================================================

/// Sanitize a float value (replace NaN/Inf with safe values)
#[inline]
pub fn sanitize_float(v: f32) -> f32 {
    if v.is_finite() && v.abs() <= 1e10 {
        v
    } else {
        0.0
    }
}

/// Sanitize a 3D vector
#[inline]
pub fn sanitize_vec3(v: [f32; 3]) -> [f32; 3] {
    [sanitize_float(v[0]), sanitize_float(v[1]), sanitize_float(v[2])]
}

/// Sanitize a 4D vector
#[inline]
pub fn sanitize_vec4(v: [f32; 4]) -> [f32; 4] {
    [sanitize_float(v[0]), sanitize_float(v[1]), sanitize_float(v[2]), sanitize_float(v[3])]
}

/// Sanitize a 4x4 matrix
#[inline]
pub fn sanitize_mat4(m: [[f32; 4]; 4]) -> [[f32; 4]; 4] {
    [
        sanitize_vec4(m[0]),
        sanitize_vec4(m[1]),
        sanitize_vec4(m[2]),
        sanitize_vec4(m[3]),
    ]
}

/// Sanitize a color value (clamp to valid range)
#[inline]
pub fn sanitize_color(v: [f32; 3]) -> [f32; 3] {
    [
        sanitize_float(v[0]).clamp(0.0, 1.0),
        sanitize_float(v[1]).clamp(0.0, 1.0),
        sanitize_float(v[2]).clamp(0.0, 1.0),
    ]
}

/// Sanitize UV coordinates
#[inline]
pub fn sanitize_uv(v: [f32; 2]) -> [f32; 2] {
    [sanitize_float(v[0]), sanitize_float(v[1])]
}

// =============================================================================
// RATE LIMITING
// =============================================================================

use std::time::{Duration, Instant};
use std::collections::VecDeque;

/// Rate limiter for expensive operations
pub struct RateLimiter {
    window: Duration,
    max_operations: usize,
    operations: VecDeque<Instant>,
}

impl RateLimiter {
    /// Create a new rate limiter
    pub fn new(max_operations: usize, window: Duration) -> Self {
        Self {
            window,
            max_operations,
            operations: VecDeque::with_capacity(max_operations),
        }
    }

    /// Try to perform an operation, returns false if rate limited
    pub fn try_operation(&mut self) -> bool {
        let now = Instant::now();

        // Remove expired operations
        while let Some(&oldest) = self.operations.front() {
            if now.duration_since(oldest) > self.window {
                self.operations.pop_front();
            } else {
                break;
            }
        }

        // Check if we can perform the operation
        if self.operations.len() >= self.max_operations {
            return false;
        }

        self.operations.push_back(now);
        true
    }

    /// Reset the rate limiter
    pub fn reset(&mut self) {
        self.operations.clear();
    }
}

// =============================================================================
// SECURITY AUDIT LOG
// =============================================================================

use std::sync::{Mutex, OnceLock};
use std::collections::HashMap;

/// Global security event log
fn security_log() -> &'static Mutex<SecurityLog> {
    static SECURITY_LOG: OnceLock<Mutex<SecurityLog>> = OnceLock::new();
    SECURITY_LOG.get_or_init(|| Mutex::new(SecurityLog::new()))
}

/// Security event types
#[derive(Debug, Clone, PartialEq)]
pub enum SecurityEvent {
    AllocationAttempt { bytes: usize, success: bool },
    ValidationFailure { error: String },
    SuspiciousActivity { description: String },
    MemoryPressure { used: usize, max: usize },
}

/// Security event log
pub struct SecurityLog {
    events: VecDeque<(Instant, SecurityEvent)>,
    event_counts: HashMap<String, usize>,
    max_events: usize,
}

impl SecurityLog {
    fn new() -> Self {
        Self {
            events: VecDeque::with_capacity(1000),
            event_counts: HashMap::new(),
            max_events: 1000,
        }
    }

    fn log(&mut self, event: SecurityEvent) {
        // Rate limit logging
        if self.events.len() >= self.max_events {
            self.events.pop_front();
        }

        // Track event counts
        let event_type = match &event {
            SecurityEvent::AllocationAttempt { .. } => "allocation",
            SecurityEvent::ValidationFailure { .. } => "validation_failure",
            SecurityEvent::SuspiciousActivity { .. } => "suspicious",
            SecurityEvent::MemoryPressure { .. } => "memory_pressure",
        };
        *self.event_counts.entry(event_type.to_string()).or_insert(0) += 1;

        self.events.push_back((Instant::now(), event));
    }

    fn get_recent_events(&self, count: usize) -> Vec<(Instant, SecurityEvent)> {
        self.events.iter().rev().take(count).cloned().collect()
    }

    fn get_event_counts(&self) -> HashMap<String, usize> {
        self.event_counts.clone()
    }
}

/// Log a security event
pub fn log_security_event(event: SecurityEvent) {
    if let Ok(mut log) = security_log().lock() {
        log.log(event);
    }
}

/// Get recent security events
pub fn get_recent_security_events(count: usize) -> Vec<(Instant, SecurityEvent)> {
    security_log().lock().map(|log| log.get_recent_events(count)).unwrap_or_default()
}

/// Get security event counts
pub fn get_security_event_counts() -> HashMap<String, usize> {
    security_log().lock().map(|log| log.get_event_counts()).unwrap_or_default()
}

// =============================================================================
// SECURITY STATUS
// =============================================================================

/// Get a security status report
pub fn get_security_status() -> SecurityStatus {
    let memory_used = get_gpu_memory_usage();
    let memory_percent = (memory_used as f32 / MAX_GPU_MEMORY_BYTES as f32) * 100.0;
    let event_counts = get_security_event_counts();

    SecurityStatus {
        gpu_memory_used: memory_used,
        gpu_memory_max: MAX_GPU_MEMORY_BYTES,
        gpu_memory_percent: memory_percent,
        validation_failures: *event_counts.get("validation_failure").unwrap_or(&0),
        suspicious_activities: *event_counts.get("suspicious").unwrap_or(&0),
        memory_pressure_events: *event_counts.get("memory_pressure").unwrap_or(&0),
        is_healthy: memory_percent < 80.0 && *event_counts.get("suspicious").unwrap_or(&0) == 0,
    }
}

/// Security status summary
#[derive(Debug, Clone)]
pub struct SecurityStatus {
    pub gpu_memory_used: usize,
    pub gpu_memory_max: usize,
    pub gpu_memory_percent: f32,
    pub validation_failures: usize,
    pub suspicious_activities: usize,
    pub memory_pressure_events: usize,
    pub is_healthy: bool,
}

impl std::fmt::Display for SecurityStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "GPU: {:.1}% ({}/{} MB) | Failures: {} | Suspicious: {} | Health: {}",
            self.gpu_memory_percent,
            self.gpu_memory_used / (1024 * 1024),
            self.gpu_memory_max / (1024 * 1024),
            self.validation_failures,
            self.suspicious_activities,
            if self.is_healthy { "OK" } else { "WARNING" }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_float() {
        assert_eq!(sanitize_float(1.5), 1.5);
        assert_eq!(sanitize_float(f32::NAN), 0.0);
        assert_eq!(sanitize_float(f32::INFINITY), 0.0);
        assert_eq!(sanitize_float(f32::NEG_INFINITY), 0.0);
        assert_eq!(sanitize_float(1e15), 0.0); // Too extreme
    }

    #[test]
    fn test_validate_instance_count() {
        assert!(validate_instance_count(100).is_ok());
        assert!(validate_instance_count(limits::MAX_INSTANCES + 1).is_err());
    }

    #[test]
    fn test_rate_limiter() {
        let mut limiter = RateLimiter::new(3, Duration::from_secs(1));
        assert!(limiter.try_operation());
        assert!(limiter.try_operation());
        assert!(limiter.try_operation());
        assert!(!limiter.try_operation()); // Should be rate limited
    }

    #[test]
    fn test_sanitize_color() {
        assert_eq!(sanitize_color([0.5, 0.5, 0.5]), [0.5, 0.5, 0.5]);
        assert_eq!(sanitize_color([1.5, -0.5, 0.5]), [1.0, 0.0, 0.5]); // Clamped
    }
}
