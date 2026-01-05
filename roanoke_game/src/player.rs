use glam::Vec3;
use croatoan_wfc::mesh_gen::get_height_at;
use crate::biped_ik::{BipedIK, PlayerMoveState};

pub struct Player {
    pub position: Vec3,
    pub velocity: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub on_ground: bool,
    pub speed: f32,
    pub jump_force: f32,
    pub gravity: f32,
    pub height: f32, // Eye height

    /// Inverse kinematics for ground adaptation
    pub biped_ik: BipedIK,
}

impl Player {
    pub fn new(position: Vec3) -> Self {
        Self {
            position,
            velocity: Vec3::ZERO,
            yaw: -90.0f32.to_radians(), // Look East
            pitch: 0.0,
            on_ground: false,
            speed: 10.0,
            jump_force: 15.0,
            gravity: 30.0,
            height: 1.8, // Standard human height
            biped_ik: BipedIK::new(position.y),
        }
    }

    /// Get current movement state for IK blending
    pub fn move_state(&self) -> PlayerMoveState {
        let speed = self.velocity.length();
        if !self.on_ground {
            PlayerMoveState::Jumping
        } else if speed > 5.0 {
            PlayerMoveState::Running
        } else if speed > 0.5 {
            PlayerMoveState::Walking
        } else {
            PlayerMoveState::Idle
        }
    }

    /// Update inverse kinematics for ground adaptation
    pub fn update_ik(&mut self, seed: u32) {
        let state = self.move_state();
        self.biped_ik.update(
            self.position,
            self.yaw,
            state,
            self.on_ground,
            |x, z| {
                let (height, _) = get_height_at(x, z, seed);
                height
            },
        );
    }

    pub fn update(&mut self, dt: f32, input_dir: Vec3, seed: u32) {
        // Use default terrain height function
        self.update_with_height_fn(dt, input_dir, seed, |x, z| {
            let (h, _) = get_height_at(x, z, seed);
            h
        });
    }

    /// Update with a custom height function (for cave-aware collision)
    pub fn update_with_height_fn<F>(&mut self, dt: f32, input_dir: Vec3, seed: u32, height_fn: F)
    where
        F: Fn(f32, f32) -> f32,
    {
        // Apply Gravity
        self.velocity.y -= self.gravity * dt;

        // Movement (XZ plane)
        // Input dir is relative to camera rotation
        let forward = Vec3::new(self.yaw.cos(), 0.0, self.yaw.sin()).normalize();
        let right = Vec3::new(-self.yaw.sin(), 0.0, self.yaw.cos()).normalize();

        let move_vec = (forward * input_dir.z + right * input_dir.x).normalize_or_zero();

        // Simple movement (no inertia for now)
        self.velocity.x = move_vec.x * self.speed;
        self.velocity.z = move_vec.z * self.speed;

        // Apply Velocity
        self.position += self.velocity * dt;

        // Terrain Collision (height_fn may account for caves)
        let ground_height = height_fn(self.position.x, self.position.z);

        if self.position.y < ground_height + self.height {
            self.position.y = ground_height + self.height;
            self.velocity.y = 0.0;
            self.on_ground = true;
        } else {
            self.on_ground = false;
        }

        // Update IK for ground adaptation
        self.update_ik(seed);
    }

    pub fn jump(&mut self) {
        if self.on_ground {
            self.velocity.y = self.jump_force;
            self.on_ground = false;
        }
    }
}
