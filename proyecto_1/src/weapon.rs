use raylib::prelude::*;
use std::fs;

#[derive(Debug, Clone)]
pub struct WeaponDef {
    pub id: String,
    pub name: String,
    pub damage: f32,
    pub clip_size: i32,
    pub reserve: i32, // tope maximo de municion en reserva para esta arma
}

impl WeaponDef {
    pub fn parse_from_file(path: &str) -> Vec<WeaponDef> {
        let content = fs::read_to_string(path).unwrap_or_else(|_| String::new());
        let mut weapons = Vec::new();

        let mut id = String::new();
        let mut name = String::new();
        let mut damage = 10.0;
        let mut clip_size = 12;
        let mut reserve = 0;

        fn flush(
            id: &mut String,
            name: &mut String,
            damage: &mut f32,
            clip_size: &mut i32,
            reserve: &mut i32,
            weapons: &mut Vec<WeaponDef>,
        ) {
            if !id.is_empty() {
                weapons.push(WeaponDef {
                    id: id.clone(),
                    name: name.clone(),
                    damage: *damage,
                    clip_size: *clip_size,
                    reserve: *reserve,
                });
                id.clear();
                *damage = 10.0;
                *clip_size = 24;
                *reserve = 0;
            }
        }

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                flush(&mut id, &mut name, &mut damage, &mut clip_size, &mut reserve, &mut weapons);
                continue;
            }
            if let Some((key, val)) = line.split_once('=') {
                let key = key.trim();
                let val = val.trim();
                match key {
                    "id" => id = val.to_string(),
                    "name" => name = val.to_string(),
                    "damage" => damage = val.parse().unwrap_or(10.0),
                    "clip_size" => clip_size = val.parse().unwrap_or(12),
                    "reserve" => reserve = val.parse().unwrap_or(0),
                    _ => {}
                }
            }
        }
        flush(&mut id, &mut name, &mut damage, &mut clip_size, &mut reserve, &mut weapons);

        weapons
    }
}

/// Componente puramente visual del arma en pantalla. La municion y el daño
/// viven en PlayerStats / WeaponDef, no aqui.
pub struct Weapon {
    pub id: String,
    pub name: String,
    pub is_shooting: bool,
    pub anim_timer: f32,
    pub drag_x: f32,
    pub drag_y: f32,
}

impl Weapon {
    pub fn new(id: &str, name: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            is_shooting: false,
            anim_timer: 0.0,
            drag_x: 0.0,
            drag_y: 0.0,
        }
    }

    /// Solo el lanzacohetes puede abrir puertas cerradas disparando a
    /// distancia.
    pub fn can_open_doors(&self) -> bool {
        self.id == "rocket"
    }

    pub fn switch_to(&mut self, id: &str, name: &str) {
        self.id = id.to_string();
        self.name = name.to_string();
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

    pub fn apply_look(
        &mut self,
        mouse_dx: f32,
        mouse_dy: f32,
        _dt: f32,
        deadzone_x: f32,
        deadzone_y: f32,
        _return_speed: f32,
    ) -> f32 {
        self.drag_x += mouse_dx;

        let mut camera_delta = 0.0;
        if self.drag_x > deadzone_x {
            camera_delta = self.drag_x - deadzone_x;
            self.drag_x = deadzone_x;
        } else if self.drag_x < -deadzone_x {
            camera_delta = self.drag_x + deadzone_x;
            self.drag_x = -deadzone_x;
        }

        self.drag_y += mouse_dy;
        self.drag_y = self.drag_y.clamp(-deadzone_y, deadzone_y);

        camera_delta
    }

    pub fn render(&self, d: &mut RaylibDrawHandle, screen_w: i32, game_h: f32) {
        let center_x = screen_w / 2;
        let center_y = (game_h / 2.0) as i32;

        let reticle_x = center_x + self.drag_x as i32;
        let reticle_y = center_y + self.drag_y as i32;

        d.draw_circle_lines(reticle_x, reticle_y, 6.0, Color::GREEN);
        d.draw_pixel(reticle_x, reticle_y, Color::RED);

        let base_w = 100;
        let base_h = 130;

        let recoil_offset = if self.is_shooting {
            (self.anim_timer * std::f32::consts::PI).sin() * 15.0
        } else {
            0.0
        };

        let weapon_x = center_x - (base_w / 2) + self.drag_x as i32;
        let weapon_y = (game_h as i32 - base_h) + recoil_offset as i32 + self.drag_y as i32;

        // color distinto para el lanzacohetes, para identificarlo a simple vista
        let body_color = if self.id == "rocket" { Color::new(90, 50, 30, 255) } else { Color::DARKGRAY };

        d.draw_rectangle(weapon_x, weapon_y, base_w, base_h + 30, body_color);
        d.draw_rectangle_lines(weapon_x, weapon_y, base_w, base_h + 30, Color::RAYWHITE);
        d.draw_rectangle(reticle_x - 12, weapon_y - 20, 24, 30, Color::GRAY);

        if self.is_shooting && self.anim_timer < 0.4 {
            d.draw_circle(reticle_x, weapon_y - 25, 20.0, Color::GOLD);
            d.draw_circle(reticle_x, weapon_y - 25, 12.0, Color::YELLOW);
        }
    }
}