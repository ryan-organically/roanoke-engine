// Quick test to verify noise output
use glam::Vec2;

// Inline the noise calculation from the port
use noise::{NoiseFn, Perlin};

fn fbm(point: Vec2, octaves: u32, lacunarity: f32, persistence: f32, seed: u32) -> f32 {
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

    value / max_value
}

fn main() {
    let world_seed = 1587u32;
    let sample_point = Vec2::new(0.5, 0.5);

    let noise_value = fbm(
        sample_point,
        4,
        2.0,
        0.5,
        world_seed,
    );

    let normalized_noise = (noise_value + 1.0) / 2.0;

    println!("World Seed: {}", world_seed);
    println!("Sample Point: {:?}", sample_point);
    println!("Generated Noise Value (raw): {}", noise_value);
    println!("Normalized Noise Value: {}", normalized_noise);

    if normalized_noise > 0.5 {
        println!("\n==> Screen will clear to BLUE (Ocean)");
        println!("    RGB: (0.1, 0.3, 0.8)");
    } else {
        println!("\n==> Screen will clear to GREEN (Land)");
        println!("    RGB: (0.2, 0.6, 0.3)");
    }
}
