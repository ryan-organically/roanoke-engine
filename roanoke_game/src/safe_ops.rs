//! Safe Operations Module
//!
//! Provides security-hardened helper functions for common operations
//! that can panic or cause undefined behavior in edge cases.
//!
//! Key features:
//! - Mutex locking with poison recovery
//! - Float comparison without NaN panics
//! - Integer arithmetic with overflow protection
//! - Safe vector/slice access helpers

use std::cmp::Ordering;
use std::sync::{Mutex, MutexGuard, PoisonError};

// =============================================================================
// MUTEX SAFETY
// =============================================================================

/// Result type for mutex operations
pub type MutexResult<T> = Result<T, MutexError>;

/// Error type for mutex operations
#[derive(Debug)]
pub enum MutexError {
    /// Mutex was poisoned (a thread panicked while holding it)
    Poisoned,
}

impl std::fmt::Display for MutexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Poisoned => write!(f, "Mutex was poisoned by a panicked thread"),
        }
    }
}

impl std::error::Error for MutexError {}

/// Extension trait for safe mutex operations
pub trait SafeMutex<T> {
    /// Lock the mutex, recovering from poison if necessary
    ///
    /// Unlike `lock().unwrap()`, this will not panic if the mutex is poisoned.
    /// Instead, it recovers the data and returns it (the data may be in an
    /// inconsistent state, but at least we don't crash).
    fn safe_lock(&self) -> MutexGuard<'_, T>;

    /// Try to lock the mutex, returning an error if poisoned
    fn try_safe_lock(&self) -> MutexResult<MutexGuard<'_, T>>;
}

impl<T> SafeMutex<T> for Mutex<T> {
    fn safe_lock(&self) -> MutexGuard<'_, T> {
        match self.lock() {
            Ok(guard) => guard,
            Err(poisoned) => {
                log::warn!("Mutex was poisoned, recovering data");
                poisoned.into_inner()
            }
        }
    }

    fn try_safe_lock(&self) -> MutexResult<MutexGuard<'_, T>> {
        match self.lock() {
            Ok(guard) => Ok(guard),
            Err(_) => Err(MutexError::Poisoned),
        }
    }
}

// =============================================================================
// FLOAT COMPARISON SAFETY
// =============================================================================

/// Safe float comparison that handles NaN values
///
/// Returns `Ordering::Equal` if either value is NaN, preventing panics
/// in sorting and comparison operations.
#[inline]
pub fn safe_cmp_f32(a: f32, b: f32) -> Ordering {
    a.partial_cmp(&b).unwrap_or(Ordering::Equal)
}

/// Safe float comparison for f64
#[inline]
pub fn safe_cmp_f64(a: f64, b: f64) -> Ordering {
    a.partial_cmp(&b).unwrap_or(Ordering::Equal)
}

/// Wrapper for total ordering of floats (treats NaN as greater than all values)
#[derive(Debug, Clone, Copy)]
pub struct TotalF32(pub f32);

impl PartialEq for TotalF32 {
    fn eq(&self, other: &Self) -> bool {
        if self.0.is_nan() && other.0.is_nan() {
            true
        } else {
            self.0 == other.0
        }
    }
}

impl Eq for TotalF32 {}

impl PartialOrd for TotalF32 {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TotalF32 {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self.0.is_nan(), other.0.is_nan()) {
            (true, true) => Ordering::Equal,
            (true, false) => Ordering::Greater,
            (false, true) => Ordering::Less,
            (false, false) => self.0.partial_cmp(&other.0).unwrap_or(Ordering::Equal),
        }
    }
}

// =============================================================================
// INTEGER OVERFLOW PROTECTION
// =============================================================================

/// Safely calculate a range-based total (e.g., for chunk iteration)
///
/// Prevents overflow when calculating `(range * 2 + 1) * (range * 2 + 1)`
#[inline]
pub fn safe_range_area(range: i32) -> Option<usize> {
    let side = range.checked_mul(2)?.checked_add(1)?;
    let side_usize = usize::try_from(side).ok()?;
    side_usize.checked_mul(side_usize)
}

/// Safely calculate range area with a default fallback
#[inline]
pub fn safe_range_area_or(range: i32, default: usize) -> usize {
    safe_range_area(range).unwrap_or(default)
}

/// Saturating range area calculation (clamps on overflow)
#[inline]
pub fn saturating_range_area(range: i32) -> usize {
    let range = range.max(0) as usize;
    let side = range.saturating_mul(2).saturating_add(1);
    side.saturating_mul(side)
}

/// Safe index calculation for 2D arrays
///
/// Returns None if the calculation would overflow or be out of bounds
#[inline]
pub fn safe_2d_index(x: usize, y: usize, width: usize) -> Option<usize> {
    y.checked_mul(width)?.checked_add(x)
}

/// Safe multiplication with overflow check
#[inline]
pub fn safe_mul(a: usize, b: usize) -> Option<usize> {
    a.checked_mul(b)
}

/// Safe addition with overflow check
#[inline]
pub fn safe_add(a: usize, b: usize) -> Option<usize> {
    a.checked_add(b)
}

// =============================================================================
// VECTOR/SLICE SAFETY
// =============================================================================

/// Safe vector access with logging
pub trait SafeAccess<T> {
    /// Get element at index, logging and returning None if out of bounds
    fn safe_get(&self, index: usize) -> Option<&T>;

    /// Get first element, logging if empty
    fn safe_first(&self) -> Option<&T>;

    /// Get last element, logging if empty
    fn safe_last(&self) -> Option<&T>;
}

impl<T> SafeAccess<T> for Vec<T> {
    fn safe_get(&self, index: usize) -> Option<&T> {
        if index >= self.len() {
            log::trace!("Vector access out of bounds: {} >= {}", index, self.len());
        }
        self.get(index)
    }

    fn safe_first(&self) -> Option<&T> {
        if self.is_empty() {
            log::trace!("Attempted to get first element of empty vector");
        }
        self.first()
    }

    fn safe_last(&self) -> Option<&T> {
        if self.is_empty() {
            log::trace!("Attempted to get last element of empty vector");
        }
        self.last()
    }
}

impl<T> SafeAccess<T> for [T] {
    fn safe_get(&self, index: usize) -> Option<&T> {
        if index >= self.len() {
            log::trace!("Slice access out of bounds: {} >= {}", index, self.len());
        }
        self.get(index)
    }

    fn safe_first(&self) -> Option<&T> {
        if self.is_empty() {
            log::trace!("Attempted to get first element of empty slice");
        }
        self.first()
    }

    fn safe_last(&self) -> Option<&T> {
        if self.is_empty() {
            log::trace!("Attempted to get last element of empty slice");
        }
        self.last()
    }
}

// =============================================================================
// SANITIZATION HELPERS
// =============================================================================

/// Sanitize a float value (replace NaN/Inf with safe default)
#[inline]
pub fn sanitize_f32(v: f32) -> f32 {
    if v.is_finite() {
        v
    } else {
        0.0
    }
}

/// Sanitize a float with custom default
#[inline]
pub fn sanitize_f32_or(v: f32, default: f32) -> f32 {
    if v.is_finite() {
        v
    } else {
        default
    }
}

/// Sanitize and clamp a float to a range
#[inline]
pub fn sanitize_clamp_f32(v: f32, min: f32, max: f32) -> f32 {
    sanitize_f32(v).clamp(min, max)
}

/// Sanitize a time value (common in game logic)
#[inline]
pub fn sanitize_time(v: f64) -> f64 {
    if v.is_finite() && v >= 0.0 {
        v
    } else {
        0.0
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_cmp_f32() {
        assert_eq!(safe_cmp_f32(1.0, 2.0), Ordering::Less);
        assert_eq!(safe_cmp_f32(2.0, 1.0), Ordering::Greater);
        assert_eq!(safe_cmp_f32(1.0, 1.0), Ordering::Equal);

        // NaN handling - should not panic
        assert_eq!(safe_cmp_f32(f32::NAN, 1.0), Ordering::Equal);
        assert_eq!(safe_cmp_f32(1.0, f32::NAN), Ordering::Equal);
        assert_eq!(safe_cmp_f32(f32::NAN, f32::NAN), Ordering::Equal);
    }

    #[test]
    fn test_safe_range_area() {
        assert_eq!(safe_range_area(0), Some(1));   // (0*2+1)^2 = 1
        assert_eq!(safe_range_area(1), Some(9));   // (1*2+1)^2 = 9
        assert_eq!(safe_range_area(2), Some(25));  // (2*2+1)^2 = 25
        assert_eq!(safe_range_area(5), Some(121)); // (5*2+1)^2 = 121

        // Overflow protection
        assert_eq!(safe_range_area(i32::MAX), None);
        assert_eq!(safe_range_area(1_000_000_000), None);
    }

    #[test]
    fn test_saturating_range_area() {
        assert_eq!(saturating_range_area(0), 1);
        assert_eq!(saturating_range_area(1), 9);
        assert_eq!(saturating_range_area(-1), 1); // Negative clamped to 0

        // Large values saturate instead of overflowing
        let result = saturating_range_area(i32::MAX);
        assert!(result > 0); // Should not overflow/wrap
    }

    #[test]
    fn test_total_f32_ordering() {
        let mut values = vec![
            TotalF32(3.0),
            TotalF32(f32::NAN),
            TotalF32(1.0),
            TotalF32(2.0),
        ];
        values.sort();

        assert_eq!(values[0].0, 1.0);
        assert_eq!(values[1].0, 2.0);
        assert_eq!(values[2].0, 3.0);
        assert!(values[3].0.is_nan()); // NaN sorts to end
    }

    #[test]
    fn test_sanitize_f32() {
        assert_eq!(sanitize_f32(1.0), 1.0);
        assert_eq!(sanitize_f32(f32::NAN), 0.0);
        assert_eq!(sanitize_f32(f32::INFINITY), 0.0);
        assert_eq!(sanitize_f32(f32::NEG_INFINITY), 0.0);
    }

    #[test]
    fn test_safe_2d_index() {
        assert_eq!(safe_2d_index(0, 0, 10), Some(0));
        assert_eq!(safe_2d_index(5, 2, 10), Some(25));
        assert_eq!(safe_2d_index(usize::MAX, 1, 10), None); // Overflow
        assert_eq!(safe_2d_index(0, usize::MAX, 10), None); // Overflow
    }
}
