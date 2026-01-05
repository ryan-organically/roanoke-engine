//! Remote Player Rendering
//!
//! Placeholder for rendering other players in the world.
//! Currently renders as colored capsules/orbs.

use glam::{Vec3, Mat4};
use crate::network::RemotePlayer;

/// Capsule mesh for remote player placeholder
pub struct PlayerCapsule {
    pub position: Vec3,
    pub yaw: f32,
    pub color: [f32; 3],
    pub height: f32,
    pub radius: f32,
}

impl PlayerCapsule {
    pub fn from_remote(player: &RemotePlayer) -> Self {
        Self {
            position: player.position,
            yaw: player.yaw,
            color: player.color,
            height: 1.8,
            radius: 0.4,
        }
    }

    /// Get transform matrix for rendering
    pub fn transform(&self) -> Mat4 {
        Mat4::from_translation(self.position)
            * Mat4::from_rotation_y(self.yaw)
    }

    /// Get eye position (for nametag placement)
    pub fn eye_position(&self) -> Vec3 {
        self.position + Vec3::Y * self.height
    }
}

/// Data for rendering remote players with existing orb pipeline
pub struct RemotePlayerOrb {
    pub position: [f32; 3],
    pub color: [f32; 4],
    pub scale: f32,
}

impl RemotePlayerOrb {
    pub fn from_remote(player: &RemotePlayer) -> Self {
        Self {
            position: player.position.to_array(),
            color: [player.color[0], player.color[1], player.color[2], 1.0],
            scale: 0.5, // Smaller than animal orbs
        }
    }
}

/// Nametag data for UI rendering
pub struct PlayerNametag {
    pub name: String,
    pub world_position: Vec3,
    pub color: [f32; 3],
}

impl PlayerNametag {
    pub fn from_remote(player: &RemotePlayer) -> Self {
        Self {
            name: player.name.clone(),
            world_position: player.position + Vec3::Y * 2.2, // Above head
            color: player.color,
        }
    }

    /// Project to screen coordinates (returns None if behind camera)
    pub fn screen_position(&self, view: Mat4, proj: Mat4, screen_size: (f32, f32)) -> Option<(f32, f32)> {
        let clip = proj * view * self.world_position.extend(1.0);

        // Behind camera check
        if clip.w <= 0.0 {
            return None;
        }

        let ndc = clip.truncate() / clip.w;

        // Off-screen check
        if ndc.x.abs() > 1.0 || ndc.y.abs() > 1.0 {
            return None;
        }

        // Convert to screen coordinates
        let x = (ndc.x + 1.0) * 0.5 * screen_size.0;
        let y = (1.0 - ndc.y) * 0.5 * screen_size.1; // Flip Y

        Some((x, y))
    }
}

/// Batch of remote player render data
pub struct RemotePlayerBatch {
    pub orbs: Vec<RemotePlayerOrb>,
    pub nametags: Vec<PlayerNametag>,
}

impl RemotePlayerBatch {
    pub fn new() -> Self {
        Self {
            orbs: Vec::new(),
            nametags: Vec::new(),
        }
    }

    pub fn add_player(&mut self, player: &RemotePlayer) {
        self.orbs.push(RemotePlayerOrb::from_remote(player));
        self.nametags.push(PlayerNametag::from_remote(player));
    }

    pub fn clear(&mut self) {
        self.orbs.clear();
        self.nametags.clear();
    }
}
