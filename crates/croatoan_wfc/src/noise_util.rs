use glam::{Vec2, Vec3};
use noise::{NoiseFn, Perlin};

/// Fractional Brownian Motion (FBM) noise
/// Combines multiple octaves of noise with decreasing amplitude
pub fn fbm(point: Vec2, octaves: u32, lacunarity: f32, persistence: f32, seed: u32) -> f32 {
    let noise = Perlin::new(seed);
    let mut value = 0.0;
    let mut amplitude = 1.0;
    let mut frequency = 1.0;
    let mut max_value = 0.0;

    for _ in 0..octaves {
        let sample_x = point.x as f64 * frequency as f64;
        let sample_y = point.y as f64 * frequency as f64;

        value += noise.get([sample_x, sample_y]) as f32 * amplitude;
        max_value += amplitude;

        amplitude *= persistence;
        frequency *= lacunarity;
    }

    // Normalize to [-1, 1] range
    value / max_value
}

/// Ridged Multifractal noise
/// Creates sharp ridge-like features, useful for mountains and terrain
pub fn ridged(point: Vec2, octaves: u32, lacunarity: f32, persistence: f32, seed: u32) -> f32 {
    let noise = Perlin::new(seed);
    let mut value = 0.0;
    let mut amplitude = 1.0;
    let mut frequency = 1.0;
    let mut weight = 1.0;

    for _ in 0..octaves {
        let sample_x = point.x as f64 * frequency as f64;
        let sample_y = point.y as f64 * frequency as f64;

        // Get noise value and create ridges by taking absolute value and inverting
        let mut signal = noise.get([sample_x, sample_y]) as f32;
        signal = signal.abs();
        signal = 1.0 - signal;

        // Square the signal to sharpen the ridges
        signal *= signal;

        // Weight successive octaves
        signal *= weight;
        weight = signal.clamp(0.0, 1.0);

        value += signal * amplitude;

        amplitude *= persistence;
        frequency *= lacunarity;
    }

    value
}

/// Turbulence noise
/// Creates chaotic, turbulent patterns by summing absolute values of noise
pub fn turbulence(point: Vec2, octaves: u32, lacunarity: f32, persistence: f32, seed: u32) -> f32 {
    let noise = Perlin::new(seed);
    let mut value = 0.0;
    let mut amplitude = 1.0;
    let mut frequency = 1.0;
    let mut max_value = 0.0;

    for _ in 0..octaves {
        let sample_x = point.x as f64 * frequency as f64;
        let sample_y = point.y as f64 * frequency as f64;

        // Take absolute value to create turbulence effect
        let noise_val = noise.get([sample_x, sample_y]) as f32;
        value += noise_val.abs() * amplitude;
        max_value += amplitude;

        amplitude *= persistence;
        frequency *= lacunarity;
    }

    // Normalize to [0, 1] range
    value / max_value
}

/// Simple hash function for deterministic randomness
pub fn hash(n: u32) -> f32 {
    let mut n = n;
    n = (n << 13) ^ n;
    n = n.wrapping_mul(n.wrapping_mul(n).wrapping_mul(15731).wrapping_add(789221)).wrapping_add(1376312589);
    (n & 0x7fffffff) as f32 / 0x7fffffff as f32
}

// ============================================================================
// 3D NOISE FUNCTIONS (for Perlin worm cave generation)
// ============================================================================

/// 3D Fractional Brownian Motion noise
pub fn fbm_3d(point: Vec3, octaves: u32, lacunarity: f32, persistence: f32, seed: u32) -> f32 {
    let noise = Perlin::new(seed);
    let mut value = 0.0;
    let mut amplitude = 1.0;
    let mut frequency = 1.0;
    let mut max_value = 0.0;

    for _ in 0..octaves {
        let sample_x = point.x as f64 * frequency as f64;
        let sample_y = point.y as f64 * frequency as f64;
        let sample_z = point.z as f64 * frequency as f64;

        value += noise.get([sample_x, sample_y, sample_z]) as f32 * amplitude;
        max_value += amplitude;

        amplitude *= persistence;
        frequency *= lacunarity;
    }

    value / max_value
}

/// 3D turbulence noise
pub fn turbulence_3d(point: Vec3, octaves: u32, lacunarity: f32, persistence: f32, seed: u32) -> f32 {
    let noise = Perlin::new(seed);
    let mut value = 0.0;
    let mut amplitude = 1.0;
    let mut frequency = 1.0;
    let mut max_value = 0.0;

    for _ in 0..octaves {
        let sample_x = point.x as f64 * frequency as f64;
        let sample_y = point.y as f64 * frequency as f64;
        let sample_z = point.z as f64 * frequency as f64;

        let noise_val = noise.get([sample_x, sample_y, sample_z]) as f32;
        value += noise_val.abs() * amplitude;
        max_value += amplitude;

        amplitude *= persistence;
        frequency *= lacunarity;
    }

    value / max_value
}

/// Sample 3D Perlin noise at a point (single value, no octaves)
pub fn perlin_3d(point: Vec3, seed: u32) -> f32 {
    let noise = Perlin::new(seed);
    noise.get([point.x as f64, point.y as f64, point.z as f64]) as f32
}

/// Calculate approximate gradient of 3D Perlin noise at a point
/// Returns a normalized direction vector following the noise gradient
pub fn noise_gradient_3d(point: Vec3, seed: u32) -> Vec3 {
    let eps = 0.1;
    let noise = Perlin::new(seed);

    let p = [point.x as f64, point.y as f64, point.z as f64];

    // Central difference approximation of gradient
    let dx = noise.get([p[0] + eps as f64, p[1], p[2]])
           - noise.get([p[0] - eps as f64, p[1], p[2]]);
    let dy = noise.get([p[0], p[1] + eps as f64, p[2]])
           - noise.get([p[0], p[1] - eps as f64, p[2]]);
    let dz = noise.get([p[0], p[1], p[2] + eps as f64])
           - noise.get([p[0], p[1], p[2] - eps as f64]);

    let grad = Vec3::new(dx as f32, dy as f32, dz as f32);

    // Return normalized gradient, or zero vector if gradient is too small
    let len = grad.length();
    if len > 0.001 {
        grad / len
    } else {
        Vec3::ZERO
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fbm() {
        let point = Vec2::new(0.5, 0.5);
        let value = fbm(point, 4, 2.0, 0.5, 42);
        assert!(value >= -1.0 && value <= 1.0);
    }

    #[test]
    fn test_ridged() {
        let point = Vec2::new(0.5, 0.5);
        let value = ridged(point, 4, 2.0, 0.5, 42);
        assert!(value >= 0.0);
    }

    #[test]
    fn test_turbulence() {
        let point = Vec2::new(0.5, 0.5);
        let value = turbulence(point, 4, 2.0, 0.5, 42);
        assert!(value >= 0.0 && value <= 1.0);
    }
}
