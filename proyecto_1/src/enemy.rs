use raylib::prelude::*;

pub struct Enemy {
    pub x: f32,
    pub y: f32,
    pub health: i32,
    pub max_health: i32,
    pub is_alive: bool,
    pub attack_damage: i32,
    pub attack_range: f32,
    pub attack_cooldown: f32,
    attack_timer: f32,
}

impl Enemy {
    pub fn new(x: f32, y: f32, health: i32) -> Self {
        Self {
            x,
            y,
            health,
            max_health: health,
            is_alive: true,
            attack_damage: 8,
            attack_range: 1.2,
            attack_cooldown: 1.0,
            attack_timer: 0.0,
        }
    }

    pub fn take_damage(&mut self, damage: i32) {
        if !self.is_alive {
            return;
        }
        self.health -= damage;
        if self.health <= 0 {
            self.health = 0;
            self.is_alive = false;
        }
    }

    ///Si el jugador esta en rango y ya
    /// paso el cooldown, devuelve el daño que debe recibir.
    pub fn update_attack(&mut self, dt: f32, player_x: f32, player_y: f32) -> Option<i32> {
        if !self.is_alive {
            return None;
        }
        if self.attack_timer > 0.0 {
            self.attack_timer -= dt;
            return None;
        }
        let dx = player_x - self.x;
        let dy = player_y - self.y;
        let dist = (dx * dx + dy * dy).sqrt();
        if dist <= self.attack_range {
            self.attack_timer = self.attack_cooldown;
            Some(self.attack_damage)
        } else {
            None
        }
    }

    pub fn get_room_cell(&self, cell_w: usize, cell_h: usize) -> (i32, i32) {
        ((self.x / cell_w as f32) as i32, (self.y / cell_h as f32) as i32)
    }

    pub fn render_2d(&self, d: &mut RaylibDrawHandle, scale: f32, ox: f32, oy: f32) {
        if !self.is_alive {
            return;
        }
        let px = (ox + self.x * scale) as i32;
        let py = (oy + self.y * scale) as i32;
        let size = 16;

        d.draw_rectangle(px - size / 2, py - size / 2, size, size, Color::RED);
        d.draw_rectangle_lines(px - size / 2, py - size / 2, size, size, Color::MAROON);

        let bar_w = 20;
        let bar_h = 4;
        let hp_ratio = (self.health as f32 / self.max_health as f32).clamp(0.0, 1.0);

        d.draw_rectangle(px - bar_w / 2, py - size / 2 - 8, bar_w, bar_h, Color::BLACK);
        d.draw_rectangle(
            px - bar_w / 2,
            py - size / 2 - 8,
            (bar_w as f32 * hp_ratio) as i32,
            bar_h,
            Color::GREEN,
        );
    }
}