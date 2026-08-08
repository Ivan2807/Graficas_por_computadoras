use crate::item::{Item, ItemType};

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