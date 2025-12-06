//! Spatial hashing for efficient proximity queries

use glam::Vec3;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;

/// Spatial hash grid for efficient proximity queries
///
/// Uses a 2D grid (XZ plane) with configurable cell size.
/// Provides O(1) insertion/removal and efficient radius queries.
pub struct SpatialHash<T: Copy + Eq + Hash> {
    cell_size: f32,
    cells: HashMap<(i32, i32), HashSet<T>>,
    positions: HashMap<T, Vec3>,
}

impl<T: Copy + Eq + Hash> SpatialHash<T> {
    /// Create a new spatial hash with the given cell size
    pub fn new(cell_size: f32) -> Self {
        Self {
            cell_size,
            cells: HashMap::new(),
            positions: HashMap::new(),
        }
    }

    /// Convert world position to cell coordinate
    fn cell_coord(&self, pos: Vec3) -> (i32, i32) {
        (
            (pos.x / self.cell_size).floor() as i32,
            (pos.z / self.cell_size).floor() as i32,
        )
    }

    /// Insert an entity at a position
    pub fn insert(&mut self, id: T, pos: Vec3) {
        let coord = self.cell_coord(pos);
        self.cells.entry(coord).or_default().insert(id);
        self.positions.insert(id, pos);
    }

    /// Remove an entity (uses stored position)
    pub fn remove(&mut self, id: T) -> bool {
        if let Some(pos) = self.positions.remove(&id) {
            let coord = self.cell_coord(pos);
            if let Some(cell) = self.cells.get_mut(&coord) {
                cell.remove(&id);
                if cell.is_empty() {
                    self.cells.remove(&coord);
                }
            }
            true
        } else {
            false
        }
    }

    /// Update an entity's position
    pub fn update(&mut self, id: T, new_pos: Vec3) {
        if let Some(old_pos) = self.positions.get(&id).copied() {
            let old_coord = self.cell_coord(old_pos);
            let new_coord = self.cell_coord(new_pos);

            if old_coord != new_coord {
                // Remove from old cell
                if let Some(cell) = self.cells.get_mut(&old_coord) {
                    cell.remove(&id);
                    if cell.is_empty() {
                        self.cells.remove(&old_coord);
                    }
                }
                // Add to new cell
                self.cells.entry(new_coord).or_default().insert(id);
            }

            self.positions.insert(id, new_pos);
        }
    }

    /// Query all entities within a radius of a point
    pub fn query_radius(&self, center: Vec3, radius: f32) -> Vec<T> {
        let mut results = Vec::new();
        let radius_sq = radius * radius;

        // Calculate cell range to check
        let min_coord = self.cell_coord(center - Vec3::new(radius, 0.0, radius));
        let max_coord = self.cell_coord(center + Vec3::new(radius, 0.0, radius));

        for cx in min_coord.0..=max_coord.0 {
            for cz in min_coord.1..=max_coord.1 {
                if let Some(cell) = self.cells.get(&(cx, cz)) {
                    for &id in cell {
                        if let Some(&pos) = self.positions.get(&id) {
                            let dist_sq = (pos - center).length_squared();
                            if dist_sq <= radius_sq {
                                results.push(id);
                            }
                        }
                    }
                }
            }
        }

        results
    }

    /// Query all entities in a specific cell
    pub fn query_cell(&self, cx: i32, cz: i32) -> impl Iterator<Item = T> + '_ {
        self.cells
            .get(&(cx, cz))
            .into_iter()
            .flat_map(|cell| cell.iter().copied())
    }

    /// Query all entities in cells overlapping a chunk
    pub fn query_chunk(&self, chunk_x: i32, chunk_z: i32, chunk_size: f32) -> Vec<T> {
        let mut results = Vec::new();

        let world_min_x = chunk_x as f32 * chunk_size;
        let world_min_z = chunk_z as f32 * chunk_size;
        let world_max_x = world_min_x + chunk_size;
        let world_max_z = world_min_z + chunk_size;

        let min_cell = self.cell_coord(Vec3::new(world_min_x, 0.0, world_min_z));
        let max_cell = self.cell_coord(Vec3::new(world_max_x, 0.0, world_max_z));

        for cx in min_cell.0..=max_cell.0 {
            for cz in min_cell.1..=max_cell.1 {
                if let Some(cell) = self.cells.get(&(cx, cz)) {
                    for &id in cell {
                        if let Some(&pos) = self.positions.get(&id) {
                            if pos.x >= world_min_x
                                && pos.x < world_max_x
                                && pos.z >= world_min_z
                                && pos.z < world_max_z
                            {
                                results.push(id);
                            }
                        }
                    }
                }
            }
        }

        results
    }

    /// Get the stored position for an entity
    pub fn get_position(&self, id: T) -> Option<Vec3> {
        self.positions.get(&id).copied()
    }

    /// Get total number of tracked entities
    pub fn len(&self) -> usize {
        self.positions.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.positions.is_empty()
    }

    /// Clear all entities
    pub fn clear(&mut self) {
        self.cells.clear();
        self.positions.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_query() {
        let mut hash: SpatialHash<u32> = SpatialHash::new(16.0);

        hash.insert(1, Vec3::new(0.0, 0.0, 0.0));
        hash.insert(2, Vec3::new(5.0, 0.0, 5.0));
        hash.insert(3, Vec3::new(100.0, 0.0, 100.0));

        let near = hash.query_radius(Vec3::ZERO, 10.0);
        assert_eq!(near.len(), 2);
        assert!(near.contains(&1));
        assert!(near.contains(&2));
    }

    #[test]
    fn test_update() {
        let mut hash: SpatialHash<u32> = SpatialHash::new(16.0);

        hash.insert(1, Vec3::new(0.0, 0.0, 0.0));
        hash.update(1, Vec3::new(100.0, 0.0, 100.0));

        let near_origin = hash.query_radius(Vec3::ZERO, 10.0);
        assert!(near_origin.is_empty());

        let near_new = hash.query_radius(Vec3::new(100.0, 0.0, 100.0), 10.0);
        assert_eq!(near_new.len(), 1);
    }
}
