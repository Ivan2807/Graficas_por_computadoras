use crate::item::{Item, ItemType};
use crate::enemy::Enemy;
use raylib::prelude::*;
use std::collections::HashMap;

pub const HUD_HEIGHT: i32 = 110;

pub struct PlayerStats {
    pub health: i32,
    pub max_health: i32,
    pub shield: i32,
    pub max_shield: i32,
    pub damage_bonus_flat: f32,
    pub damage_multiplier: f32,
    pub keys: u8,
    pub inventory: Vec<Item>,
    pub score: u32,
    pub kills: u32,
    pub clip_ammo: HashMap<String, i32>,
    pub reserve_ammo: HashMap<String, i32>,
}

impl Default for PlayerStats {
    fn default() -> Self {
        Self {
            health: 100,
            max_health: 100,
            shield: 25,
            max_shield: 50,
            damage_bonus_flat: 0.0,
            damage_multiplier: 1.0,
            keys: 0,
            inventory: Vec::new(),
            score: 0,
            kills: 0,
            clip_ammo: HashMap::new(),
            reserve_ammo: HashMap::new(),
        }
    }
}

impl PlayerStats {
    /// Aplicar items con la lógica de desborde de vida a escudo
    pub fn apply_item(&mut self, item: Item) {
        match item.item_type {
            ItemType::Health => {
                let cur = self.health + item.flat_value as i32;
                if cur > self.max_health {
                    let overflow = cur - self.max_health;
                    self.health = self.max_health;
                    // El sobrante recarga el escudo
                    self.shield = (self.shield + overflow).min(self.max_shield);
                } else {
                    self.health = cur;
                }
            }
            ItemType::Shield => {
                self.shield = (self.shield + item.flat_value as i32).min(self.max_shield);
            }
            ItemType::Damage => {
                self.damage_bonus_flat += item.flat_value;
                self.damage_multiplier *= item.multiplier;
            }
            ItemType::Key => {
                self.keys += 1;
            }
            ItemType::SingleUse => {
                self.inventory.push(item);
            }
        }
    }

    pub fn take_damage(&mut self, amount: i32) {
        if self.shield >= amount {
            self.shield -= amount;
        } else {
            let rem = amount - self.shield;
            self.shield = 0;
            self.health = (self.health - rem).max(0);
        }
    }
}

/// Estado del Radar de Barrido (180 grados en el frente)
pub struct RadarSweep {
    pub current_angle_offset: f32, // -PI/2 a +PI/2 respecto a la vista del jugador
    pub sweep_dir: f32,
    pub pings: Vec<RadarPing>,
}

pub struct RadarPing {
    pub enemy_id: usize,
    pub is_mega: bool,
    pub rel_x: f32,
    pub rel_y: f32,
    pub alpha: f32,
}

impl RadarSweep {
    pub fn new() -> Self {
        Self {
            current_angle_offset: -std::f32::consts::FRAC_PI_2,
            sweep_dir: 1.0,
            pings: Vec::new(),
        }
    }

    pub fn update(&mut self, dt: f32, player_x: f32, player_y: f32, player_angle: f32, enemies: &[Enemy]) {
        let speed = 2.5; // Velocidad del barrido
        self.current_angle_offset += self.sweep_dir * speed * dt;

        let half_pi = std::f32::consts::FRAC_PI_2;
        if self.current_angle_offset > half_pi {
            self.current_angle_offset = half_pi;
            self.sweep_dir = -1.0;
        } else if self.current_angle_offset < -half_pi {
            self.current_angle_offset = -half_pi;
            self.sweep_dir = 1.0;
        }

        // Rayo de barrido actual en coordenadas globales
        let sweep_angle = player_angle + self.current_angle_offset;

        // Comprobar si el barrido pasa cerca de un enemigo vivo
        for e in enemies {
            if !e.is_alive() { continue; }
            let dx = e.x - player_x;
            let dy = e.y - player_y;
            let dist = (dx * dx + dy * dy).sqrt();

            if dist < 12.0 { // Alcance del radar
                let enemy_angle = dy.atan2(dx);
                let diff = (enemy_angle - sweep_angle).sin().abs();
                
                // Si el rayo alcanza al enemigo en este frame, registramos un 'Ping'
                if diff < 0.08 && !self.pings.iter().any(|ping| ping.enemy_id == e.id) {
                    // Convertir a coordenadas locales relativas a la rotación del jugador
                    let local_x = dx * player_angle.cos() + dy * player_angle.sin();
                    let local_y = -dx * player_angle.sin() + dy * player_angle.cos();

                    self.pings.push(RadarPing {
                        enemy_id: e.id,
                        is_mega: e.is_mega,
                        rel_x: local_x,
                        rel_y: local_y,
                        alpha: 1.0,
                    });
                }
            }
        }

        // Desvanecer pings
        for p in &mut self.pings {
            p.alpha -= dt * 2.0;
        }
        self.pings.retain(|p| p.alpha > 0.0);
    }

    pub fn render(&self, d: &mut RaylibDrawHandle, x: i32, y: i32, radius: f32) {
        // Fondo de la pantalla del Radar
        d.draw_circle_sector(Vector2::new(x as f32, y as f32), radius, 180.0, 360.0, 16, Color::new(10, 30, 15, 230));
        d.draw_circle_sector_lines(Vector2::new(x as f32, y as f32), radius, 180.0, 360.0, 16, Color::GREEN);

        // Línea de barrido (180° frontal)
        let ray_len = radius - 2.0;
        let line_end_x = x as f32 + ray_len * self.current_angle_offset.sin();
        let line_end_y = y as f32 - ray_len * self.current_angle_offset.cos();
        d.draw_line(x, y, line_end_x as i32, line_end_y as i32, Color::LIME);

        // Dibujar Pings de enemigos
        let scale = radius / 12.0; // Escalado según alcance del radar
        for p in &self.pings {
            let px = x as f32 + p.rel_y * scale; 
            let py = y as f32 - p.rel_x * scale;

            if py <= y as f32 { // Asegurar que solo se dibuje en el arco frontal
                let col = Color::new(255, 30, 30, (p.alpha * 255.0) as u8);
                d.draw_circle(px as i32, py as i32, 3.5, col);
            }
        }

        d.draw_text("RADAR 180°", x - 35, y + 5, 12, Color::GREEN);
    }
}

pub fn render_hud(
    d: &mut RaylibDrawHandle,
    screen_w: i32,
    screen_h: i32,
    stats: &PlayerStats,
    weapon_name: &str,
    clip: i32,
    reserve: i32,
    radar: &RadarSweep,
    level_timer: f32,
) {
    let hud_y = screen_h - HUD_HEIGHT;

    // Panel HUD Inferior
    d.draw_rectangle(0, hud_y, screen_w, HUD_HEIGHT, Color::new(15, 18, 22, 245));
    d.draw_rectangle_lines(0, hud_y, screen_w, HUD_HEIGHT, Color::DARKGRAY);

    let margin = 20;
    let bar_w = 180;
    let bar_h = 20;

    // --- BARRAS DE VIDA Y ESCUDO ---
    let hp_y = hud_y + 15;
    let hp_ratio = (stats.health as f32 / stats.max_health as f32).clamp(0.0, 1.0);
    d.draw_rectangle(margin, hp_y, bar_w, bar_h, Color::RED);
    d.draw_rectangle(margin, hp_y, (bar_w as f32 * hp_ratio) as i32, bar_h, Color::GREEN);
    d.draw_text(&format!("HP: {}/{}", stats.health, stats.max_health), margin + 5, hp_y + 2, 15, Color::WHITE);

    let sh_y = hp_y + bar_h + 8;
    let sh_ratio = (stats.shield as f32 / stats.max_shield as f32).clamp(0.0, 1.0);
    d.draw_rectangle(margin, sh_y, bar_w, bar_h, Color::DARKBLUE);
    d.draw_rectangle(margin, sh_y, (bar_w as f32 * sh_ratio) as i32, bar_h, Color::SKYBLUE);
    d.draw_text(&format!("ESCUDO: {}/{}", stats.shield, stats.max_shield), margin + 5, sh_y + 2, 15, Color::WHITE);

    // --- AMMO / ARMA EQUIPADA ---
    let ammo_x = margin + bar_w + 30;
    d.draw_text(&format!("ARMA: {}", weapon_name), ammo_x, hp_y, 18, Color::GOLD);
    d.draw_text(&format!("BALAS: {} / {}", clip, reserve), ammo_x, hp_y + 25, 20, Color::YELLOW);
    d.draw_text(&format!("LLAVES OBTENIDAS: {}/3", stats.keys.min(3)), ammo_x, hp_y + 50, 16, Color::GOLD);
    let timer_color = if level_timer <= 15.0 { Color::RED } else { Color::WHITE };
    let seconds = level_timer.max(0.0) as i32;
    d.draw_text(&format!("TIEMPO: {:02}:{:02}", seconds / 60, seconds % 60), screen_w / 2 - 70, 15, 20, timer_color);
    if level_timer <= 30.0 {
        d.draw_text("ESCAPE INMINENTE", screen_w / 2 - 95, 40, 18, Color::RED);
    }

    // --- INVENTARIO DE CONSUMIBLES / ITEMS ---
    let inv_x = ammo_x + 200;
    d.draw_text("INVENTARIO:", inv_x, hp_y, 16, Color::LIGHTGRAY);
    if stats.inventory.is_empty() {
        d.draw_text("(Vacío)", inv_x, hp_y + 22, 14, Color::GRAY);
    } else {
        for (idx, item) in stats.inventory.iter().enumerate().take(3) {
            d.draw_text(&format!("- {}", item.name), inv_x, hp_y + 22 + (idx as i32 * 18), 14, Color::WHITE);
        }
    }

    // --- RENDERIZAR RADAR DE BARRIDO EN LA ESQUINA DEL HUD ---
    radar.render(d, screen_w - 90, screen_h - 25, 65.0);
}