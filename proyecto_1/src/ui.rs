use crate::item::{Item, ItemType};
use raylib::prelude::*;

pub const HUD_HEIGHT: i32 = 90;

pub struct PlayerStats {
    pub health: i32,
    pub max_health: i32,
    pub shield: i32,
    pub max_shield: i32,
    pub base_damage: f32,
    pub damage_multiplier: f32,
    pub ammo_in_clip: i32,
    pub clip_size: i32,
    pub ammo_reserve: i32,
    pub weapon_name: String,
    pub keys: u8,
    pub inventory: Vec<Item>,
}

impl Default for PlayerStats {
    fn default() -> Self {
        Self {
            health: 100,
            max_health: 100,
            shield: 0,
            max_shield: 50,
            base_damage: 10.0,
            damage_multiplier: 1.0,
            ammo_in_clip: 12,
            clip_size: 12,
            ammo_reserve: 48,
            weapon_name: "Pistola".to_string(),
            keys: 0,
            inventory: Vec::new(),
        }
    }
}

impl PlayerStats {
    pub fn get_final_damage(&self) -> f32 {
        self.base_damage * self.damage_multiplier
    }

    pub fn apply_item(&mut self, item: Item) {
        match item.item_type {
            ItemType::Health => {
                self.max_health = (self.max_health as f32 * item.multiplier) as i32;
                self.health = (self.health + item.flat_value as i32).min(self.max_health);
            }
            ItemType::Shield => {
                self.max_shield = (self.max_shield as f32 * item.multiplier) as i32;
                self.shield = (self.shield + item.flat_value as i32).min(self.max_shield);
            }
            ItemType::Damage => {
                self.base_damage += item.flat_value;
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
}

/// Dibuja la franja de HUD en la parte inferior de la pantalla:
/// barra de vida, barra de escudo, munición y llaves.
pub fn render_hud(
    d: &mut RaylibDrawHandle,
    screen_w: i32,
    screen_h: i32,
    stats: &PlayerStats,
) {
    let hud_y = screen_h - HUD_HEIGHT;

    // Fondo del HUD
    d.draw_rectangle(0, hud_y, screen_w, HUD_HEIGHT, Color::new(20, 20, 25, 235));
    d.draw_rectangle_lines(0, hud_y, screen_w, HUD_HEIGHT, Color::new(90, 90, 100, 255));

    let margin = 20;
    let bar_w = 220;
    let bar_h = 22;

    // --- Barra de vida ---
    let hp_x = margin;
    let hp_y = hud_y + 15;
    let hp_ratio = (stats.health as f32 / stats.max_health.max(1) as f32).clamp(0.0, 1.0);
    d.draw_rectangle(hp_x, hp_y, bar_w, bar_h, Color::new(60, 20, 20, 255));
    d.draw_rectangle(hp_x, hp_y, (bar_w as f32 * hp_ratio) as i32, bar_h, Color::new(200, 40, 40, 255));
    d.draw_rectangle_lines(hp_x, hp_y, bar_w, bar_h, Color::RAYWHITE);
    d.draw_text(
        &format!("HP {}/{}", stats.health, stats.max_health),
        hp_x + 6,
        hp_y + 3,
        16,
        Color::WHITE,
    );

    // --- Barra de escudo ---
    let sh_x = hp_x;
    let sh_y = hp_y + bar_h + 8;
    let sh_ratio = if stats.max_shield > 0 {
        (stats.shield as f32 / stats.max_shield as f32).clamp(0.0, 1.0)
    } else {
        0.0
    };
    d.draw_rectangle(sh_x, sh_y, bar_w, bar_h, Color::new(20, 30, 60, 255));
    d.draw_rectangle(sh_x, sh_y, (bar_w as f32 * sh_ratio) as i32, bar_h, Color::new(60, 120, 220, 255));
    d.draw_rectangle_lines(sh_x, sh_y, bar_w, bar_h, Color::RAYWHITE);
    d.draw_text(
        &format!("Escudo {}/{}", stats.shield, stats.max_shield),
        sh_x + 6,
        sh_y + 3,
        16,
        Color::WHITE,
    );

    // --- Munición y arma ---
    let ammo_text = format!("{}  {} / {}", stats.weapon_name, stats.ammo_in_clip, stats.ammo_reserve);
    let ammo_w = d.measure_text(&ammo_text, 22);
    d.draw_text(
        &ammo_text,
        screen_w - ammo_w - margin,
        hud_y + 15,
        22,
        Color::GOLD,
    );

    // --- Daño final (informativo) ---
    let dmg_text = format!("Daño: {:.1}", stats.get_final_damage());
    let dmg_w = d.measure_text(&dmg_text, 16);
    d.draw_text(
        &dmg_text,
        screen_w - dmg_w - margin,
        hud_y + 45,
        16,
        Color::LIGHTGRAY,
    );

    // --- Llaves ---
    let keys_text = format!("Llaves: {}", stats.keys);
    let keys_w = d.measure_text(&keys_text, 16);
    d.draw_text(
        &keys_text,
        screen_w - keys_w - margin,
        hud_y + 65,
        16,
        Color::YELLOW,
    );
}