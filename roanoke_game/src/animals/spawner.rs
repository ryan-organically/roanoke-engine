//! Chunk-based animal spawning system

use super::manager::AnimalManager;
use super::types::{AnimalSpecies, Habitat, TimeOfDay};
use glam::Vec3;
use noise::{NoiseFn, Perlin};

/// Configuration for animal spawning
pub struct AnimalSpawner {
    spawn_noise: Perlin,
    pub max_animals: usize,
    pub min_spawn_distance: f32,
    pub animals_per_chunk: f32,
}

impl AnimalSpawner {
    /// Create a new spawner with the given seed
    pub fn new(seed: u32) -> Self {
        Self {
            spawn_noise: Perlin::new(seed),
            max_animals: 50,
            min_spawn_distance: 40.0,
            animals_per_chunk: 0.5, // Average animals per chunk
        }
    }

    /// Called when a chunk finishes loading - spawns animals appropriate for the area
    pub fn on_chunk_loaded(
        &self,
        chunk_x: i32,
        chunk_z: i32,
        chunk_size: f32,
        manager: &mut AnimalManager,
        player_pos: Vec3,
        seed: u32,
    ) {
        // Check global cap
        if manager.animal_count() >= self.max_animals {
            return;
        }

        // Determine habitats for this chunk based on terrain
        let chunk_center = Vec3::new(
            (chunk_x as f32 + 0.5) * chunk_size,
            0.0,
            (chunk_z as f32 + 0.5) * chunk_size,
        );

        let habitats = self.determine_habitats(chunk_x, chunk_z, seed);

        // Get eligible species
        let eligible: Vec<AnimalSpecies> = AnimalSpecies::all()
            .filter(|species| {
                // Check habitat match
                let species_habitats = species.habitats();
                let habitat_match = species_habitats.iter().any(|h| habitats.contains(h));

                // Check time of day
                let time_match = species
                    .active_times()
                    .iter()
                    .any(|t| t.matches(manager.time_of_day));

                habitat_match && time_match
            })
            .collect();

        if eligible.is_empty() {
            return;
        }

        // Generate spawn positions
        let spawn_points = self.generate_spawn_points(chunk_x, chunk_z, chunk_size, seed);

        for (spawn_pos, spawn_seed) in spawn_points {
            // Distance check from player
            let dist_to_player = spawn_pos.distance(player_pos);
            if dist_to_player < self.min_spawn_distance {
                continue;
            }

            // Another animal count check
            if manager.animal_count() >= self.max_animals {
                break;
            }

            // Select species based on spawn rates
            if let Some(species) = self.select_species(&eligible, spawn_seed, &manager.difficulty) {
                // Pack spawning
                if let Some((min, max)) = species.pack_size() {
                    let pack_size = seeded_range(spawn_seed, min, max);
                    let pack_id = manager.create_pack(species);

                    for i in 0..pack_size {
                        let offset = Vec3::new(
                            seeded_float(spawn_seed + i as u32 * 17) * 6.0 - 3.0,
                            0.0,
                            seeded_float(spawn_seed + i as u32 * 31) * 6.0 - 3.0,
                        );
                        let member_pos = spawn_pos + offset;
                        manager.spawn(species, member_pos, (chunk_x, chunk_z), Some(pack_id));
                    }
                } else {
                    // Solo animal
                    manager.spawn(species, spawn_pos, (chunk_x, chunk_z), None);
                }
            }
        }
    }

    /// Called when chunk unloads - mark animals for potential despawn
    pub fn on_chunk_unloaded(&self, chunk_x: i32, chunk_z: i32, chunk_size: f32, manager: &mut AnimalManager) {
        let in_chunk = manager.query_chunk(chunk_x, chunk_z, chunk_size);

        for id in in_chunk {
            if let Some(animal) = manager.get_mut(id) {
                // Give a grace period before despawn
                if animal.despawn_timer.is_none() {
                    animal.despawn_timer = Some(30.0);
                }
            }
        }
    }

    /// Determine habitats present in a chunk based on terrain characteristics
    fn determine_habitats(&self, chunk_x: i32, chunk_z: i32, seed: u32) -> Vec<Habitat> {
        let mut habitats = Vec::new();

        // Sample noise for terrain characteristics
        let x = chunk_x as f64 * 0.1;
        let z = chunk_z as f64 * 0.1;

        let elevation_noise = self.spawn_noise.get([x, z, seed as f64 * 0.01]);
        let moisture_noise = self.spawn_noise.get([x + 100.0, z + 100.0, seed as f64 * 0.01]);

        // Map noise to terrain types
        let elevation = (elevation_noise + 1.0) * 0.5; // 0-1
        let moisture = (moisture_noise + 1.0) * 0.5; // 0-1

        // Height-based habitats
        if elevation > 0.7 {
            habitats.push(Habitat::Mountains);
            habitats.push(Habitat::RockyAreas);
        }
        if elevation > 0.3 && elevation < 0.7 {
            habitats.push(Habitat::Forests);
            if moisture > 0.5 {
                habitats.push(Habitat::Meadows);
            }
        }
        if elevation < 0.4 {
            habitats.push(Habitat::Plains);
            habitats.push(Habitat::Fields);
        }

        // Moisture-based habitats
        if moisture > 0.7 {
            habitats.push(Habitat::Swamps);
            habitats.push(Habitat::Marshes);
            habitats.push(Habitat::NearWater);
        }
        if moisture > 0.5 && elevation < 0.5 {
            habitats.push(Habitat::Rivers);
        }

        // Coastal (low elevation, medium moisture)
        if elevation < 0.3 && moisture > 0.3 && moisture < 0.7 {
            habitats.push(Habitat::CoastalPlains);
        }

        // Default to forests if nothing else matches
        if habitats.is_empty() {
            habitats.push(Habitat::Forests);
        }

        habitats
    }

    /// Generate potential spawn points within a chunk
    fn generate_spawn_points(
        &self,
        chunk_x: i32,
        chunk_z: i32,
        chunk_size: f32,
        seed: u32,
    ) -> Vec<(Vec3, u32)> {
        let mut points = Vec::new();

        // Use noise to determine spawn density for this chunk
        let density_noise = self.spawn_noise.get([
            chunk_x as f64 * 0.05,
            chunk_z as f64 * 0.05,
            seed as f64 * 0.001 + 50.0,
        ]);

        let density = ((density_noise + 1.0) * 0.5 * self.animals_per_chunk as f64 * 2.0) as usize;
        let num_points = density.min(4); // Max 4 spawn points per chunk

        // Generate points using deterministic noise
        for i in 0..num_points {
            let point_seed = hash_combine(seed, chunk_x as u32, chunk_z as u32, i as u32);

            let local_x = seeded_float(point_seed) * chunk_size;
            let local_z = seeded_float(point_seed.wrapping_add(12345)) * chunk_size;

            let world_x = chunk_x as f32 * chunk_size + local_x;
            let world_z = chunk_z as f32 * chunk_size + local_z;

            // Get terrain height at this position
            // For now, use a placeholder - this should query actual terrain
            let world_y = get_terrain_height(world_x, world_z, seed);

            points.push((Vec3::new(world_x, world_y, world_z), point_seed));
        }

        points
    }

    /// Select a species based on spawn rates and difficulty
    fn select_species(
        &self,
        eligible: &[AnimalSpecies],
        seed: u32,
        difficulty: &super::types::Difficulty,
    ) -> Option<AnimalSpecies> {
        if eligible.is_empty() {
            return None;
        }

        // Calculate total spawn weight
        let spawn_modifier = difficulty.spawn_rate_multiplier();
        let total_weight: f32 = eligible
            .iter()
            .map(|s| s.spawn_rate() * spawn_modifier)
            .sum();

        // Random selection weighted by spawn rate
        let roll = seeded_float(seed) * total_weight;
        let mut cumulative = 0.0;

        for species in eligible {
            cumulative += species.spawn_rate() * spawn_modifier;
            if roll < cumulative {
                return Some(*species);
            }
        }

        // Fallback to first eligible
        eligible.first().copied()
    }
}

/// Get terrain height at world position (placeholder - should integrate with actual terrain)
fn get_terrain_height(x: f32, z: f32, seed: u32) -> f32 {
    // This should call into croatoan_wfc::mesh_gen::get_height_at
    // For now, return a reasonable default
    use croatoan_wfc::mesh_gen::get_height_at;
    let (height, _) = get_height_at(x, z, seed);
    height
}

/// Simple hash combination for seeding
fn hash_combine(a: u32, b: u32, c: u32, d: u32) -> u32 {
    let mut h = a;
    h = h.wrapping_mul(31).wrapping_add(b);
    h = h.wrapping_mul(31).wrapping_add(c);
    h = h.wrapping_mul(31).wrapping_add(d);
    h
}

/// Generate a float [0, 1) from a seed
fn seeded_float(seed: u32) -> f32 {
    // Simple LCG
    let a: u32 = 1664525;
    let c: u32 = 1013904223;
    let next = seed.wrapping_mul(a).wrapping_add(c);
    (next as f32) / (u32::MAX as f32)
}

/// Generate a u8 in range [min, max] from a seed
fn seeded_range(seed: u32, min: u8, max: u8) -> u8 {
    let range = (max - min + 1) as u32;
    let value = (seeded_float(seed) * range as f32) as u8;
    min + value.min(max - min)
}
