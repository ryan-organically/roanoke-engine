use glam::Vec2;
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
    n = n.wrapping_mul(n.wrapping_mul(n).wrapping_mul(15731) + 789221) + 1376312589;
    (n & 0x7fffffff) as f32 / 0x7fffffff as f32
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
