use glam::{Vec3, Vec4, Mat4};

/// A view frustum defined by 6 planes for culling
#[derive(Clone, Copy)]
pub struct Frustum {
    planes: [Vec4; 6], // Left, Right, Bottom, Top, Near, Far
}

impl Frustum {
    /// Extract frustum planes from a view-projection matrix
    pub fn from_view_proj(vp: &Mat4) -> Self {
        let m = vp.to_cols_array_2d();

        // Extract planes using Gribb/Hartmann method
        // Each plane is Ax + By + Cz + D = 0, stored as Vec4(A, B, C, D)
        let planes = [
            // Left: row3 + row0
            Self::normalize_plane(Vec4::new(
                m[0][3] + m[0][0],
                m[1][3] + m[1][0],
                m[2][3] + m[2][0],
                m[3][3] + m[3][0],
            )),
            // Right: row3 - row0
            Self::normalize_plane(Vec4::new(
                m[0][3] - m[0][0],
                m[1][3] - m[1][0],
                m[2][3] - m[2][0],
                m[3][3] - m[3][0],
            )),
            // Bottom: row3 + row1
            Self::normalize_plane(Vec4::new(
                m[0][3] + m[0][1],
                m[1][3] + m[1][1],
                m[2][3] + m[2][1],
                m[3][3] + m[3][1],
            )),
            // Top: row3 - row1
            Self::normalize_plane(Vec4::new(
                m[0][3] - m[0][1],
                m[1][3] - m[1][1],
                m[2][3] - m[2][1],
                m[3][3] - m[3][1],
            )),
            // Near: row3 + row2
            Self::normalize_plane(Vec4::new(
                m[0][3] + m[0][2],
                m[1][3] + m[1][2],
                m[2][3] + m[2][2],
                m[3][3] + m[3][2],
            )),
            // Far: row3 - row2
            Self::normalize_plane(Vec4::new(
                m[0][3] - m[0][2],
                m[1][3] - m[1][2],
                m[2][3] - m[2][2],
                m[3][3] - m[3][2],
            )),
        ];

        Self { planes }
    }

    fn normalize_plane(plane: Vec4) -> Vec4 {
        let normal_length = Vec3::new(plane.x, plane.y, plane.z).length();
        if normal_length > 0.0 {
            plane / normal_length
        } else {
            plane
        }
    }

    /// Test if a sphere intersects or is inside the frustum
    pub fn contains_sphere(&self, center: Vec3, radius: f32) -> bool {
        for plane in &self.planes {
            // Distance from point to plane
            let distance = plane.x * center.x + plane.y * center.y + plane.z * center.z + plane.w;
            if distance < -radius {
                return false; // Sphere is completely outside this plane
            }
        }
        true // Sphere intersects or is inside all planes
    }

    /// Test if an axis-aligned bounding box intersects or is inside the frustum
    pub fn contains_aabb(&self, min: Vec3, max: Vec3) -> bool {
        for plane in &self.planes {
            // Find the corner of the AABB closest to the plane (in the direction of the normal)
            let p = Vec3::new(
                if plane.x >= 0.0 { max.x } else { min.x },
                if plane.y >= 0.0 { max.y } else { min.y },
                if plane.z >= 0.0 { max.z } else { min.z },
            );

            // If this corner is outside the plane, the entire AABB is outside
            let distance = plane.x * p.x + plane.y * p.y + plane.z * p.z + plane.w;
            if distance < 0.0 {
                return false;
            }
        }
        true
    }
}

/// Bounding information for a chunk
#[derive(Clone, Copy)]
pub struct ChunkBounds {
    pub center: Vec3,
    pub radius: f32,
    pub min: Vec3,
    pub max: Vec3,
}

impl ChunkBounds {
    /// Create bounds for a chunk given its world offset and size
    pub fn new(offset_x: f32, offset_z: f32, chunk_size: f32, height_min: f32, height_max: f32) -> Self {
        let min = Vec3::new(offset_x, height_min, offset_z);
        let max = Vec3::new(offset_x + chunk_size, height_max, offset_z + chunk_size);
        let center = (min + max) * 0.5;
        let radius = (max - min).length() * 0.5;

        Self { center, radius, min, max }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frustum_sphere() {
        // Identity projection - should contain everything in front
        let vp = Mat4::perspective_rh(1.0, 1.0, 0.1, 100.0);
        let frustum = Frustum::from_view_proj(&vp);

        // Point in front should be visible
        assert!(frustum.contains_sphere(Vec3::new(0.0, 0.0, -10.0), 1.0));

        // Point behind should not be visible
        assert!(!frustum.contains_sphere(Vec3::new(0.0, 0.0, 10.0), 1.0));
    }
}
