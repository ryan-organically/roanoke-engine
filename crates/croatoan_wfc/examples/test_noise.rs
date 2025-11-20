use croatoan_wfc::{noise_util, WorldSeed};
use glam::Vec2;

fn main() {
    let world_seed = WorldSeed::new(1587);
    let sample_point = Vec2::new(0.5, 0.5);

    let noise_value = noise_util::fbm(
        sample_point,
        4,
        2.0,
        0.5,
        world_seed.value,
    );

    let normalized_noise = (noise_value + 1.0) / 2.0;

    println!("World Seed: {}", world_seed.value);
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
