use raylib::prelude::*;
use crate::level::LevelMap;

#[derive(Clone, Debug)]
pub struct Enemy {
    pub id: usize,
    pub home_room: (usize, usize),
    pub is_mega: bool,
    pub x: f32,
    pub y: f32,
    pub health: f32,
    pub max_health: f32,
    pub speed: f32,
    pub attack_damage: i32,
    pub attack_cooldown: f32,
    pub corpse_color: Color,
}

impl Enemy {
    pub fn new(id: usize, x: f32, y: f32, home_room: (usize, usize)) -> Self {
        // Asignar un color único y llamativo para el cadáver según su ID
        let corpse_colors = [
            Color::ORANGE, Color::PURPLE, Color::LIME, Color::PINK,
            Color::MAROON, Color::TEAL, Color::GOLD, Color::DARKBLUE,
        ];
        let corpse_color = corpse_colors[id % corpse_colors.len()];

        Self {
            id,
            home_room,
            is_mega: false,
            x,
            y,
            health: 75.0,
            max_health: 75.0,
            speed: 0.6,
            attack_damage: 15,
            attack_cooldown: 0.0,
            corpse_color,
        }
    }

    pub fn mega(id: usize, x: f32, y: f32, home_room: (usize, usize), health: f32) -> Self {
        let mut enemy = Self::new(id, x, y, home_room);
        enemy.is_mega = true;
        enemy.health = health;
        enemy.max_health = health;
        enemy.speed = 0.7;
        enemy.attack_damage = 60;
        enemy.corpse_color = Color::PURPLE;
        enemy
    }

    pub fn is_alive(&self) -> bool {
        self.health > 0.0
    }

    pub fn update(&mut self, dt: f32, player_x: f32, player_y: f32, map: &LevelMap) -> bool {
        if self.attack_cooldown > 0.0 {
            self.attack_cooldown -= dt;
        }

        if !self.is_alive() {
            return false; // El enemigo muerto no se mueve ni ataca
        }

        // Persecución hacia el jugador
        let dx = player_x - self.x;
        let dy = player_y - self.y;
        let dist = (dx * dx + dy * dy).sqrt();

        // Si el jugador está cerca pero no encima, avanzar
        if dist > 0.4 {
            let step_x = (dx / dist) * self.speed * dt;
            let step_y = (dy / dist) * self.speed * dt;

            // Simple verificación de pared
            if !map.is_wall_or_locked(self.x + step_x, self.y) {
                self.x += step_x;
            }
            if !map.is_wall_or_locked(self.x, self.y + step_y) {
                self.y += step_y;
            }
        }

        // Atacar si está al alcance
        if dist < 0.6 && self.attack_cooldown <= 0.0 {
            self.attack_cooldown = 1.2;
            return true; // Aplica daño
        }

        false
    }
}