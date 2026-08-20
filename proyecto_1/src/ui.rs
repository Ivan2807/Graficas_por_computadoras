use crate::item::{Item, ItemType};
use raylib::prelude::*;
use std::collections::HashMap;

pub const HUD_HEIGHT: i32 = 90;

pub const KILL_STREAK_WINDOW: f32 = 3.0;
pub const BASE_KILL_SCORE: u32 = 100;
pub const KILLS_PER_BONUS_KEY: u32 = 5;

pub struct PlayerStats {
    pub health: i32,
    pub max_health: i32,
    pub shield: i32,
    pub max_shield: i32,
    /// Bono de daño plano acumulado por items (no por el arma equipada).
    pub damage_bonus_flat: f32,
    pub damage_multiplier: f32,
    pub keys: u8,
    pub inventory: Vec<Item>,
    pub score: u32,
    pub kills: u32,
    pub kills_since_key: u32,
    pub kill_streak_count: u32,
    pub kill_streak_timer: f32,
    /// Municion por arma, indexada por el `id` de WeaponDef (ej. "pistol").
    pub clip_ammo: HashMap<String, i32>,
    pub reserve_ammo: HashMap<String, i32>,
}

impl Default for PlayerStats {
    fn default() -> Self {
        Self {
            health: 100,
            max_health: 100,
            shield: 0,
            max_shield: 50,
            damage_bonus_flat: 0.0,
            damage_multiplier: 1.0,
            keys: 0,
            inventory: Vec::new(),
            score: 0,
            kills: 0,
            kills_since_key: 0,
            kill_streak_count: 0,
            kill_streak_timer: 0.0,
            clip_ammo: HashMap::new(),
            reserve_ammo: HashMap::new(),
        }
    }
}

impl PlayerStats {
    /// Llamar una vez al inicio: le da al jugador el cargador+reserva de
    /// fabrica de la pistola; el resto de armas arrancan en 0/0 (ya tiene
    /// el arma en el HUD, pero sin municion hasta encontrarla).
    pub fn init_ammo(&mut self, defs: &[crate::weapon::WeaponDef]) {
        for d in defs {
            let starting_clip = if d.id == "pistol" { d.clip_size } else { 0 };
            let starting_reserve = if d.id == "pistol" { d.reserve } else { 0 };
            self.clip_ammo.insert(d.id.clone(), starting_clip);
            self.reserve_ammo.insert(d.id.clone(), starting_reserve);
        }
    }

    pub fn clip(&self, weapon_id: &str) -> i32 {
        *self.clip_ammo.get(weapon_id).unwrap_or(&0)
    }

    pub fn reserve(&self, weapon_id: &str) -> i32 {
        *self.reserve_ammo.get(weapon_id).unwrap_or(&0)
    }

    pub fn add_reserve(&mut self, weapon_id: &str, amount: i32, max_reserve: i32) {
        let e = self.reserve_ammo.entry(weapon_id.to_string()).or_insert(0);
        *e = (*e + amount).min(max_reserve);
    }

    /// Descuenta 1 bala del cargador del arma indicada. false = no habia municion.
    pub fn consume_clip(&mut self, weapon_id: &str) -> bool {
        let clip = self.clip_ammo.entry(weapon_id.to_string()).or_insert(0);
        if *clip > 0 {
            *clip -= 1;
            true
        } else {
            false
        }
    }

    pub fn reload(&mut self, weapon_id: &str, clip_size: i32) {
        let reserve = *self.reserve_ammo.get(weapon_id).unwrap_or(&0);
        let clip = *self.clip_ammo.get(weapon_id).unwrap_or(&0);
        let needed = (clip_size - clip).max(0);
        let take = needed.min(reserve);
        self.clip_ammo.insert(weapon_id.to_string(), clip + take);
        self.reserve_ammo.insert(weapon_id.to_string(), reserve - take);
    }

    pub fn get_final_damage(&self, weapon_damage: f32) -> f32 {
        (weapon_damage + self.damage_bonus_flat) * self.damage_multiplier
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

    pub fn register_kill(&mut self) -> bool {
        if self.kill_streak_timer > 0.0 {
            self.kill_streak_count += 1;
        } else {
            self.kill_streak_count = 1;
        }
        self.kill_streak_timer = KILL_STREAK_WINDOW;

        self.score += BASE_KILL_SCORE * self.kill_streak_count;
        self.kills += 1;
        self.kills_since_key += 1;

        if self.kills_since_key >= KILLS_PER_BONUS_KEY {
            self.kills_since_key = 0;
            true
        } else {
            false
        }
    }

    pub fn update_streak_timer(&mut self, dt: f32) {
        if self.kill_streak_timer > 0.0 {
            self.kill_streak_timer -= dt;
            if self.kill_streak_timer <= 0.0 {
                self.kill_streak_timer = 0.0;
                self.kill_streak_count = 0;
            }
        }
    }
}

/// `weapon_name`, `clip`, `reserve` y `final_damage` ya vienen resueltos
/// del arma actualmente equipada (main.rs decide cual es).
pub fn render_hud(
    d: &mut RaylibDrawHandle,
    screen_w: i32,
    screen_h: i32,
    stats: &PlayerStats,
    weapon_name: &str,
    clip: i32,
    reserve: i32,
    final_damage: f32,
) {
    let hud_y = screen_h - HUD_HEIGHT;

    d.draw_rectangle(0, hud_y, screen_w, HUD_HEIGHT, Color::new(20, 20, 25, 235));
    d.draw_rectangle_lines(0, hud_y, screen_w, HUD_HEIGHT, Color::new(90, 90, 100, 255));

    let margin = 20;
    let bar_w = 220;
    let bar_h = 22;

    let score_text = format!("Puntos: {}   Racha x{}", stats.score, stats.kill_streak_count.max(1));
    d.draw_text(&score_text, margin, hud_y - 28, 20, Color::SKYBLUE);

    let hp_x = margin;
    let hp_y = hud_y + 15;
    let hp_ratio = (stats.health as f32 / stats.max_health.max(1) as f32).clamp(0.0, 1.0);
    d.draw_rectangle(hp_x, hp_y, bar_w, bar_h, Color::new(60, 20, 20, 255));
    d.draw_rectangle(hp_x, hp_y, (bar_w as f32 * hp_ratio) as i32, bar_h, Color::new(200, 40, 40, 255));
    d.draw_rectangle_lines(hp_x, hp_y, bar_w, bar_h, Color::RAYWHITE);
    d.draw_text(&format!("HP {}/{}", stats.health, stats.max_health), hp_x + 6, hp_y + 3, 16, Color::WHITE);

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
    d.draw_text(&format!("Escudo {}/{}", stats.shield, stats.max_shield), sh_x + 6, sh_y + 3, 16, Color::WHITE);

    let ammo_text = format!("{}  {} / {}", weapon_name, clip, reserve);
    let ammo_w = d.measure_text(&ammo_text, 22);
    d.draw_text(&ammo_text, screen_w - ammo_w - margin, hud_y + 15, 22, Color::GOLD);

    let dmg_text = format!("Daño: {:.1}", final_damage);
    let dmg_w = d.measure_text(&dmg_text, 16);
    d.draw_text(&dmg_text, screen_w - dmg_w - margin, hud_y + 45, 16, Color::LIGHTGRAY);

    let keys_text = format!("Llaves: {}", stats.keys);
    let keys_w = d.measure_text(&keys_text, 16);
    d.draw_text(&keys_text, screen_w - keys_w - margin, hud_y + 65, 16, Color::YELLOW);
}