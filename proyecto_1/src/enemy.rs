use raylib::prelude::*;

pub struct Enemy {
    pub x: f32,
    pub y: f32,
    pub health: i32,
    pub max_health: i32,
    pub is_alive: bool,
}

impl Enemy {
    pub fn new(x: f32, y: f32, health: i32) -> Self {
        Self {
            x,
            y,
            health,
            max_health: health,
            is_alive: true,
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

    pub fn get_room_cell(&self, cell_w: usize, cell_h: usize) -> (i32, i32) {
        (
            (self.x / cell_w as f32) as i32,
            (self.y / cell_h as f32) as i32,
        )
    }

    // Renderizado geométrico simple en la vista 2D de depuración
    pub fn render_2d(&self, d: &mut RaylibDrawHandle, scale: f32) {
        if !self.is_alive {
            return;
        }

        let px = (self.x * scale) as i32;
        let py = (self.y * scale) as i32;
        let size = 16;

        // Cuadrado Rojo que representa al monstruo
        d.draw_rectangle(px - size / 2, py - size / 2, size, size, Color::RED);
        d.draw_rectangle_lines(px - size / 2, py - size / 2, size, size, Color::MAROON);

        // Barra de vida superior
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