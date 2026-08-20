use raylib::prelude::*;
use crate::level::LevelMap;

pub struct Player {
    pub pos_x: f32,
    pub pos_y: f32,
    pub angle: f32,
    pub speed: f32,
    pub rot_speed: f32,
    pub radius: f32,
}

impl Player {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            pos_x: x,
            pos_y: y,
            angle: 0.0,
            speed: 3.0,      // Velocidad de movimiento
            rot_speed: 2.2,  // Velocidad de rotación con Q / E
            radius: 0.25,
        }
    }

    pub fn handle_input(&mut self, rl: &RaylibHandle, dt: f32, map: &LevelMap) {
        // --- ROTACIÓN CON Q Y E ---
        if rl.is_key_down(KeyboardKey::KEY_Q) {
            self.angle -= self.rot_speed * dt;
        }
        if rl.is_key_down(KeyboardKey::KEY_E) {
            self.angle += self.rot_speed * dt;
        }

        // Mantener ángulo normalizado [0, 2PI)
        let two_pi = std::f32::consts::TAU;
        self.angle = (self.angle % two_pi + two_pi) % two_pi;

        // --- MOVIMIENTO CON W, A, S, D ---
        let mut move_forward = 0.0f32;
        let mut move_strafe = 0.0f32;

        if rl.is_key_down(KeyboardKey::KEY_W) { move_forward += 1.0; }
        if rl.is_key_down(KeyboardKey::KEY_S) { move_forward -= 1.0; }
        if rl.is_key_down(KeyboardKey::KEY_D) { move_strafe += 1.0; }  // Strafe Derecha
        if rl.is_key_down(KeyboardKey::KEY_A) { move_strafe -= 1.0; }  // Strafe Izquierda

        if move_forward != 0.0 || move_strafe != 0.0 {
            let cos_a = self.angle.cos();
            let sin_a = self.angle.sin();

            // Dirección hacia adelante + dirección lateral
            let dir_x = cos_a * move_forward - sin_a * move_strafe;
            let dir_y = sin_a * move_forward + cos_a * move_strafe;

            let len = (dir_x * dir_x + dir_y * dir_y).sqrt();
            if len > 0.0 {
                let dx = (dir_x / len) * self.speed * dt;
                let dy = (dir_y / len) * self.speed * dt;
                self.move_with_collision(dx, dy, map);
            }
        }
    }

    fn move_with_collision(&mut self, dx: f32, dy: f32, map: &LevelMap) {
        // Deslizamiento en X
        let new_x = self.pos_x + dx;
        if !map.is_wall_or_locked(new_x + self.radius.copysign(dx), self.pos_y) {
            self.pos_x = new_x;
        }
        // Deslizamiento en Y
        let new_y = self.pos_y + dy;
        if !map.is_wall_or_locked(self.pos_x, new_y + self.radius.copysign(dy)) {
            self.pos_y = new_y;
        }
    }
}