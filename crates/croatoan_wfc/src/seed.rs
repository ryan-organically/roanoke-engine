/// World seed structure for procedural generation
/// Provides deterministic random values based on a base seed
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorldSeed {
    pub value: u32,
}

impl WorldSeed {
    /// Create a new WorldSeed with the given value
    pub fn new(seed: u32) -> Self {
        Self { value: seed }
    }

    /// Hash combine function from Boost C++ library
    /// Formula: seed ^ (value + 0x9e3779b9 + (seed << 6) + (seed >> 2))
    /// This is the standard hash combine used in Unity and many C# codebases
    pub fn hash_combine(&self, value: u32) -> u32 {
        let seed = self.value;

        // Using wrapping operations to handle overflow safely
        seed ^ (value
            .wrapping_add(0x9e3779b9)
            .wrapping_add(seed << 6)
            .wrapping_add(seed >> 2))
    }

    /// Combine this seed with a value to create a new derived seed
    pub fn combine(&self, value: u32) -> WorldSeed {
        WorldSeed::new(self.hash_combine(value))
    }

    /// Combine this seed with multiple values
    pub fn combine_multiple(&self, values: &[u32]) -> WorldSeed {
        let mut result = self.value;
        for &value in values {
            result = WorldSeed::new(result).hash_combine(value);
        }
        WorldSeed::new(result)
    }

    /// Generate a seed for a specific coordinate (useful for chunk-based generation)
    pub fn for_position(&self, x: i32, y: i32) -> WorldSeed {
        self.combine_multiple(&[x as u32, y as u32])
    }

    /// Generate a seed for a specific coordinate with a layer identifier
    pub fn for_layer(&self, x: i32, y: i32, layer: u32) -> WorldSeed {
        self.combine_multiple(&[x as u32, y as u32, layer])
    }
}

impl From<u32> for WorldSeed {
    fn from(value: u32) -> Self {
        Self::new(value)
    }
}

impl Default for WorldSeed {
    fn default() -> Self {
        Self::new(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_combine() {
        let seed = WorldSeed::new(12345);
        let hash1 = seed.hash_combine(67890);
        let hash2 = seed.hash_combine(67890);

        // Should be deterministic
        assert_eq!(hash1, hash2);

        // Should be different from input
        assert_ne!(hash1, 12345);
        assert_ne!(hash1, 67890);
    }

    #[test]
    fn test_combine() {
        let seed = WorldSeed::new(12345);
        let new_seed = seed.combine(67890);

        assert_ne!(seed.value, new_seed.value);
    }

    #[test]
    fn test_for_position() {
        let seed = WorldSeed::new(12345);
        let pos_seed1 = seed.for_position(10, 20);
        let pos_seed2 = seed.for_position(10, 20);
        let pos_seed3 = seed.for_position(10, 21);

        // Same position should give same seed
        assert_eq!(pos_seed1.value, pos_seed2.value);

        // Different position should give different seed
        assert_ne!(pos_seed1.value, pos_seed3.value);
    }

    #[test]
    fn test_for_layer() {
        let seed = WorldSeed::new(12345);
        let layer1 = seed.for_layer(10, 20, 0);
        let layer2 = seed.for_layer(10, 20, 1);

        // Different layers should give different seeds
        assert_ne!(layer1.value, layer2.value);
    }
}
