//! Pipeline Validation Module
//!
//! Provides robust validation for GPU pipeline operations to prevent crashes,
//! GPU memory exhaustion, and undefined behavior.

use std::fmt;

/// Maximum vertices per mesh (prevents GPU memory exhaustion)
pub const MAX_VERTICES: usize = 500_000;
/// Maximum indices per mesh
pub const MAX_INDICES: usize = 3_000_000;
/// Maximum instances per draw call
pub const MAX_INSTANCES: usize = 100_000;
/// Minimum buffer size (wgpu requires non-zero buffers)
pub const MIN_BUFFER_SIZE: usize = 4;

/// Pipeline validation error types
#[derive(Debug, Clone)]
pub enum PipelineError {
    /// Empty vertex data provided
    EmptyVertexData,
    /// Empty index data provided
    EmptyIndexData,
    /// Vertex arrays have mismatched lengths
    MismatchedVertexArrays {
        positions: usize,
        colors: usize,
        normals: usize,
    },
    /// Vertex/UV arrays have mismatched lengths
    MismatchedVertexUVArrays {
        positions: usize,
        normals: usize,
        uvs: usize,
    },
    /// Index references non-existent vertex
    IndexOutOfBounds {
        index: u32,
        max_vertex: usize,
    },
    /// Too many vertices (would exhaust GPU memory)
    TooManyVertices {
        count: usize,
        max: usize,
    },
    /// Too many indices
    TooManyIndices {
        count: usize,
        max: usize,
    },
    /// Too many instances
    TooManyInstances {
        count: usize,
        max: usize,
    },
    /// Index count not divisible by 3 (not valid triangles)
    InvalidTriangleCount {
        index_count: usize,
    },
    /// Shader compilation failed
    ShaderCompilationFailed(String),
    /// Buffer creation failed
    BufferCreationFailed(String),
    /// Pipeline creation failed
    PipelineCreationFailed(String),
    /// Surface configuration failed
    SurfaceConfigFailed(String),
    /// Texture creation failed
    TextureCreationFailed(String),
}

impl fmt::Display for PipelineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyVertexData => write!(f, "Empty vertex data provided"),
            Self::EmptyIndexData => write!(f, "Empty index data provided"),
            Self::MismatchedVertexArrays { positions, colors, normals } => {
                write!(f, "Mismatched vertex arrays: positions={}, colors={}, normals={}",
                       positions, colors, normals)
            }
            Self::MismatchedVertexUVArrays { positions, normals, uvs } => {
                write!(f, "Mismatched vertex/UV arrays: positions={}, normals={}, uvs={}",
                       positions, normals, uvs)
            }
            Self::IndexOutOfBounds { index, max_vertex } => {
                write!(f, "Index {} out of bounds (max vertex: {})", index, max_vertex)
            }
            Self::TooManyVertices { count, max } => {
                write!(f, "Too many vertices: {} (max: {})", count, max)
            }
            Self::TooManyIndices { count, max } => {
                write!(f, "Too many indices: {} (max: {})", count, max)
            }
            Self::TooManyInstances { count, max } => {
                write!(f, "Too many instances: {} (max: {})", count, max)
            }
            Self::InvalidTriangleCount { index_count } => {
                write!(f, "Index count {} not divisible by 3 (invalid triangles)", index_count)
            }
            Self::ShaderCompilationFailed(msg) => {
                write!(f, "Shader compilation failed: {}", msg)
            }
            Self::BufferCreationFailed(msg) => {
                write!(f, "Buffer creation failed: {}", msg)
            }
            Self::PipelineCreationFailed(msg) => {
                write!(f, "Pipeline creation failed: {}", msg)
            }
            Self::SurfaceConfigFailed(msg) => {
                write!(f, "Surface configuration failed: {}", msg)
            }
            Self::TextureCreationFailed(msg) => {
                write!(f, "Texture creation failed: {}", msg)
            }
        }
    }
}

impl std::error::Error for PipelineError {}

/// Result type for pipeline operations
pub type PipelineResult<T> = Result<T, PipelineError>;

/// Validation context for mesh data
pub struct MeshValidator {
    max_vertices: usize,
    max_indices: usize,
    validate_triangles: bool,
}

impl Default for MeshValidator {
    fn default() -> Self {
        Self {
            max_vertices: MAX_VERTICES,
            max_indices: MAX_INDICES,
            validate_triangles: true,
        }
    }
}

impl MeshValidator {
    /// Create a new validator with custom limits
    pub fn new(max_vertices: usize, max_indices: usize) -> Self {
        Self {
            max_vertices,
            max_indices,
            validate_triangles: true,
        }
    }

    /// Disable triangle validation (for non-triangle topology)
    pub fn skip_triangle_validation(mut self) -> Self {
        self.validate_triangles = false;
        self
    }

    /// Validate terrain mesh data (positions, colors, normals, indices)
    pub fn validate_terrain(
        &self,
        positions: &[[f32; 3]],
        colors: &[[f32; 3]],
        normals: &[[f32; 3]],
        indices: &[u32],
    ) -> PipelineResult<()> {
        // Check for empty data
        if positions.is_empty() {
            return Err(PipelineError::EmptyVertexData);
        }
        if indices.is_empty() {
            return Err(PipelineError::EmptyIndexData);
        }

        // Check array lengths match
        if positions.len() != colors.len() || positions.len() != normals.len() {
            return Err(PipelineError::MismatchedVertexArrays {
                positions: positions.len(),
                colors: colors.len(),
                normals: normals.len(),
            });
        }

        // Check limits
        if positions.len() > self.max_vertices {
            return Err(PipelineError::TooManyVertices {
                count: positions.len(),
                max: self.max_vertices,
            });
        }
        if indices.len() > self.max_indices {
            return Err(PipelineError::TooManyIndices {
                count: indices.len(),
                max: self.max_indices,
            });
        }

        // Validate triangle count
        if self.validate_triangles && indices.len() % 3 != 0 {
            return Err(PipelineError::InvalidTriangleCount {
                index_count: indices.len(),
            });
        }

        // Validate indices (spot check for performance - check every 100th index)
        let vertex_count = positions.len();
        for (i, &idx) in indices.iter().enumerate() {
            // Full check for small meshes, spot check for large
            if indices.len() < 10000 || i % 100 == 0 {
                if idx as usize >= vertex_count {
                    return Err(PipelineError::IndexOutOfBounds {
                        index: idx,
                        max_vertex: vertex_count,
                    });
                }
            }
        }

        Ok(())
    }

    /// Validate tree/model mesh data (positions, normals, uvs, indices)
    pub fn validate_model(
        &self,
        positions: &[[f32; 3]],
        normals: &[[f32; 3]],
        uvs: &[[f32; 2]],
        indices: &[u32],
    ) -> PipelineResult<()> {
        // Check for empty data
        if positions.is_empty() {
            return Err(PipelineError::EmptyVertexData);
        }
        if indices.is_empty() {
            return Err(PipelineError::EmptyIndexData);
        }

        // Check array lengths match
        if positions.len() != normals.len() || positions.len() != uvs.len() {
            return Err(PipelineError::MismatchedVertexUVArrays {
                positions: positions.len(),
                normals: normals.len(),
                uvs: uvs.len(),
            });
        }

        // Check limits
        if positions.len() > self.max_vertices {
            return Err(PipelineError::TooManyVertices {
                count: positions.len(),
                max: self.max_vertices,
            });
        }
        if indices.len() > self.max_indices {
            return Err(PipelineError::TooManyIndices {
                count: indices.len(),
                max: self.max_indices,
            });
        }

        // Validate triangle count
        if self.validate_triangles && indices.len() % 3 != 0 {
            return Err(PipelineError::InvalidTriangleCount {
                index_count: indices.len(),
            });
        }

        // Validate indices
        let vertex_count = positions.len();
        for (i, &idx) in indices.iter().enumerate() {
            if indices.len() < 10000 || i % 100 == 0 {
                if idx as usize >= vertex_count {
                    return Err(PipelineError::IndexOutOfBounds {
                        index: idx,
                        max_vertex: vertex_count,
                    });
                }
            }
        }

        Ok(())
    }

    /// Validate grass mesh data (positions, colors, indices)
    pub fn validate_grass(
        &self,
        positions: &[[f32; 3]],
        colors: &[[f32; 3]],
        indices: &[u32],
    ) -> PipelineResult<()> {
        // Check for empty data
        if positions.is_empty() {
            return Err(PipelineError::EmptyVertexData);
        }
        if indices.is_empty() {
            return Err(PipelineError::EmptyIndexData);
        }

        // Check array lengths match
        if positions.len() != colors.len() {
            return Err(PipelineError::MismatchedVertexArrays {
                positions: positions.len(),
                colors: colors.len(),
                normals: positions.len(), // Not used for grass
            });
        }

        // Check limits
        if positions.len() > self.max_vertices {
            return Err(PipelineError::TooManyVertices {
                count: positions.len(),
                max: self.max_vertices,
            });
        }
        if indices.len() > self.max_indices {
            return Err(PipelineError::TooManyIndices {
                count: indices.len(),
                max: self.max_indices,
            });
        }

        // Validate indices
        let vertex_count = positions.len();
        for (i, &idx) in indices.iter().enumerate() {
            if indices.len() < 10000 || i % 100 == 0 {
                if idx as usize >= vertex_count {
                    return Err(PipelineError::IndexOutOfBounds {
                        index: idx,
                        max_vertex: vertex_count,
                    });
                }
            }
        }

        Ok(())
    }

    /// Validate instance count
    pub fn validate_instances(&self, count: usize) -> PipelineResult<()> {
        if count > MAX_INSTANCES {
            return Err(PipelineError::TooManyInstances {
                count,
                max: MAX_INSTANCES,
            });
        }
        Ok(())
    }
}

/// Check if a value contains NaN or infinity (invalid for GPU)
#[inline]
pub fn is_valid_float(v: f32) -> bool {
    v.is_finite()
}

/// Check if a vec3 contains valid floats
#[inline]
pub fn is_valid_vec3(v: &[f32; 3]) -> bool {
    v.iter().all(|&f| is_valid_float(f))
}

/// Sanitize a float value (replace NaN/Inf with 0)
#[inline]
pub fn sanitize_float(v: f32) -> f32 {
    if v.is_finite() { v } else { 0.0 }
}

/// Sanitize a vec3 (replace invalid values with 0)
#[inline]
pub fn sanitize_vec3(v: [f32; 3]) -> [f32; 3] {
    [sanitize_float(v[0]), sanitize_float(v[1]), sanitize_float(v[2])]
}

/// Log pipeline warning (non-fatal issues)
pub fn log_pipeline_warning(pipeline: &str, message: &str) {
    log::warn!("[{}] {}", pipeline, message);
}

/// Log pipeline error
pub fn log_pipeline_error(pipeline: &str, error: &PipelineError) {
    log::error!("[{}] Pipeline error: {}", pipeline, error);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_vertex_data() {
        let validator = MeshValidator::default();
        let result = validator.validate_terrain(&[], &[], &[], &[0, 1, 2]);
        assert!(matches!(result, Err(PipelineError::EmptyVertexData)));
    }

    #[test]
    fn test_mismatched_arrays() {
        let validator = MeshValidator::default();
        let positions = [[0.0; 3]; 10];
        let colors = [[0.0; 3]; 5]; // Wrong size
        let normals = [[0.0; 3]; 10];
        let indices = [0, 1, 2];

        let result = validator.validate_terrain(&positions, &colors, &normals, &indices);
        assert!(matches!(result, Err(PipelineError::MismatchedVertexArrays { .. })));
    }

    #[test]
    fn test_index_out_of_bounds() {
        let validator = MeshValidator::default();
        let positions = [[0.0; 3]; 3];
        let colors = [[0.0; 3]; 3];
        let normals = [[0.0; 3]; 3];
        let indices = [0, 1, 10]; // Index 10 is out of bounds

        let result = validator.validate_terrain(&positions, &colors, &normals, &indices);
        assert!(matches!(result, Err(PipelineError::IndexOutOfBounds { .. })));
    }

    #[test]
    fn test_valid_mesh() {
        let validator = MeshValidator::default();
        let positions = [[0.0; 3]; 3];
        let colors = [[1.0; 3]; 3];
        let normals = [[0.0, 1.0, 0.0]; 3];
        let indices = [0, 1, 2];

        let result = validator.validate_terrain(&positions, &colors, &normals, &indices);
        assert!(result.is_ok());
    }

    #[test]
    fn test_sanitize_float() {
        assert_eq!(sanitize_float(1.5), 1.5);
        assert_eq!(sanitize_float(f32::NAN), 0.0);
        assert_eq!(sanitize_float(f32::INFINITY), 0.0);
        assert_eq!(sanitize_float(f32::NEG_INFINITY), 0.0);
    }
}
