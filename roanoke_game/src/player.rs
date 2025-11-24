use glam::Vec3;
use croatoan_wfc::mesh_gen::get_height_at;

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
        }
    }

    pub fn update(&mut self, dt: f32, input_dir: Vec3, seed: u32) {
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

        // Terrain Collision
        let (terrain_height, _) = get_height_at(self.position.x, self.position.z, seed);
        
        if self.position.y < terrain_height + self.height {
            self.position.y = terrain_height + self.height;
            self.velocity.y = 0.0;
            self.on_ground = true;
        } else {
            self.on_ground = false;
        }
    }

    pub fn jump(&mut self) {
        if self.on_ground {
            self.velocity.y = self.jump_force;
            self.on_ground = false;
        }
    }
}
