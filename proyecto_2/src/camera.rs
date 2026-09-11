use raylib::prelude::Vector3;

pub struct Camera {
    pub eye: Vector3,
    pub yaw: f32,
    pub pitch: f32,
}

impl Camera {
    pub fn new(eye: Vector3) -> Self {
        Camera { eye, yaw: 0.0, pitch: 0.0 }
    }

    pub fn rotate(&mut self, delta_yaw: f32, delta_pitch: f32) {
        self.yaw += delta_yaw;
        self.pitch = (self.pitch + delta_pitch).clamp(
            -89.0_f32.to_radians(),
            89.0_f32.to_radians(),
        );
    }

    pub fn basis(&self) -> (Vector3, Vector3, Vector3) {
        let forward = Vector3::new(
            self.yaw.sin() * self.pitch.cos(),
            self.pitch.sin(),
            -self.yaw.cos() * self.pitch.cos(),
        )
        .normalized();

        let world_up = Vector3::new(0.0, 1.0, 0.0);
        let right = forward.cross(world_up).normalized();
        let up = right.cross(forward).normalized();

        (forward, right, up)
    }
}