use raylib::prelude::*;

pub struct Weapon {
    pub name: String,
    pub is_shooting: bool,
    pub anim_timer: f32,
}

impl Weapon {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            is_shooting: false,
            anim_timer: 0.0,
        }
    }

    pub fn update(&mut self, dt: f32) {
        if self.is_shooting {
            self.anim_timer += dt * 10.0;
            if self.anim_timer >= 1.0 {
                self.is_shooting = false;
                self.anim_timer = 0.0;
            }
        }
    }

    pub fn render(&self, d: &mut RaylibDrawHandle, screen_w: i32, game_h: f32) {
        let center_x = screen_w / 2;
        let center_y = (game_h / 2.0) as i32;

        // --- Mira / Retícula (Capa intermedia) ---
        d.draw_circle_lines(center_x, center_y, 6.0, Color::GREEN);
        d.draw_pixel(center_x, center_y, Color::RED);

        // --- Sprite/Placeholder del Arma ---
        let base_w = 100;
        let base_h = 130;
        
        // Retroceso sutil si se presiona disparo
        let recoil_offset = if self.is_shooting {
            (self.anim_timer * std::f32::consts::PI).sin() * 15.0
        } else {
            0.0
        };

        let weapon_x = center_x - (base_w / 2);
        let weapon_y = (game_h as i32 - base_h) + recoil_offset as i32;

        // Cuerpo principal del arma
        d.draw_rectangle(weapon_x, weapon_y, base_w, base_h + 30, Color::DARKGRAY);
        d.draw_rectangle_lines(weapon_x, weapon_y, base_w, base_h + 30, Color::RAYWHITE);
        
        // Cañón
        d.draw_rectangle(center_x - 12, weapon_y - 20, 24, 30, Color::GRAY);
        
        // Fogonazo si dispara
        if self.is_shooting && self.anim_timer < 0.4 {
            d.draw_circle(center_x, weapon_y - 25, 20.0, Color::GOLD);
            d.draw_circle(center_x, weapon_y - 25, 12.0, Color::YELLOW);
        }
    }
}