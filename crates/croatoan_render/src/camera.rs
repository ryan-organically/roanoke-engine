use glam::{Mat4, Vec3};

/// 3D Camera with view and projection matrices
pub struct Camera {
    pub position: Vec3,
    pub target: Vec3,
    pub up: Vec3,
    pub fov: f32,
    pub aspect_ratio: f32,
    pub near: f32,
    pub far: f32,
    pub yaw: f32,
    pub pitch: f32,
}

impl Camera {
    /// Create a new camera
    pub fn new(position: Vec3, target: Vec3, aspect_ratio: f32) -> Self {
        // Calculate initial yaw and pitch from position and target
        let direction = (target - position).normalize();
        let yaw = direction.z.atan2(direction.x);
        let pitch = direction.y.asin();

        Self {
            position,
            target,
            up: Vec3::Y,
            fov: 45.0_f32.to_radians(),
            aspect_ratio,
            near: 0.1,
            far: 1000.0,
            yaw,
            pitch,
        }
    }

    /// Get the view matrix (camera transform)
    pub fn view_matrix(&self) -> Mat4 {
        Mat4::look_at_rh(self.position, self.target, self.up)
    }

    /// Get the projection matrix
    pub fn projection_matrix(&self) -> Mat4 {
        Mat4::perspective_rh(self.fov, self.aspect_ratio, self.near, self.far)
    }

    /// Get combined view-projection matrix
    pub fn view_projection_matrix(&self) -> Mat4 {
        self.projection_matrix() * self.view_matrix()
    }

    /// Update aspect ratio (for window resize)
    pub fn set_aspect_ratio(&mut self, aspect_ratio: f32) {
        self.aspect_ratio = aspect_ratio;
    }

    /// Update the view matrix based on yaw and pitch
    pub fn update_vectors(&mut self) {
        // Calculate forward direction from yaw and pitch
        let forward = Vec3::new(
            self.yaw.cos() * self.pitch.cos(),
            self.pitch.sin(),
            self.yaw.sin() * self.pitch.cos(),
        )
        .normalize();

        // Update target to be 1 unit in front of position
        self.target = self.position + forward;
    }

    /// Get the forward direction vector
    pub fn forward(&self) -> Vec3 {
        Vec3::new(
            self.yaw.cos() * self.pitch.cos(),
            self.pitch.sin(),
            self.yaw.sin() * self.pitch.cos(),
        )
        .normalize()
    }

    /// Get the right direction vector
    pub fn right(&self) -> Vec3 {
        self.forward().cross(Vec3::Y).normalize()
    }

    /// Process mouse movement
    pub fn process_mouse(&mut self, delta_x: f32, delta_y: f32, sensitivity: f32) {
        self.yaw += delta_x * sensitivity;
        self.pitch -= delta_y * sensitivity;

        // Clamp pitch to prevent gimbal lock
        self.pitch = self.pitch.clamp(-1.5, 1.5);

        self.update_vectors();
    }

    /// Move the camera forward/backward
    pub fn move_forward(&mut self, amount: f32) {
        let forward = Vec3::new(
            self.yaw.cos(),
            0.0,
            self.yaw.sin(),
        )
        .normalize();
        self.position += forward * amount;
        self.update_vectors();
    }

    /// Move the camera left/right
    pub fn move_right(&mut self, amount: f32) {
        let right = self.right();
        self.position += Vec3::new(right.x, 0.0, right.z).normalize() * amount;
        self.update_vectors();
    }

    /// Move the camera up/down (global Y)
    pub fn move_up(&mut self, amount: f32) {
        self.position.y += amount;
        self.update_vectors();
    }
}
