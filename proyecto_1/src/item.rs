use std::fs;

#[derive(Debug, Clone, PartialEq)]
pub enum ItemType {
    Damage,
    Health,
    Shield,
    SingleUse,
    Key,
}

#[derive(Debug, Clone)]
pub struct Item {
    pub id: String,
    pub name: String,
    pub item_type: ItemType,
    pub flat_value: f32,   // Incremento plano (ej: +20 HP, +10 daño)
    pub multiplier: f32,   // Multiplicador directo (ej: 1.5x daño)
}

impl Item {
    pub fn parse_from_file(path: &str) -> Vec<Item> {
        let content = fs::read_to_string(path).unwrap_or_else(|_| String::new());
        let mut items = Vec::new();

        let mut current_id = String::new();
        let mut current_name = String::new();
        let mut current_type = ItemType::SingleUse;
        let mut current_flat = 0.0;
        let mut current_mult = 1.0;

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                if !current_id.is_empty() {
                    items.push(Item {
                        id: current_id.clone(),
                        name: current_name.clone(),
                        item_type: current_type.clone(),
                        flat_value: current_flat,
                        multiplier: current_mult,
                    });
                    current_id.clear();
                    current_mult = 1.0;
                    current_flat = 0.0;
                }
                continue;
            }

            if let Some((key, val)) = line.split_once('=') {
                let key = key.trim();
                let val = val.trim();
                match key {
                    "id" => current_id = val.to_string(),
                    "name" => current_name = val.to_string(),
                    "flat" => current_flat = val.parse().unwrap_or(0.0),
                    "mult" => current_mult = val.parse().unwrap_or(1.0),
                    "type" => {
                        current_type = match val {
                            "Damage" => ItemType::Damage,
                            "Health" => ItemType::Health,
                            "Shield" => ItemType::Shield,
                            "Key" => ItemType::Key,
                            _ => ItemType::SingleUse,
                        };
                    }
                    _ => {}
                }
            }
        }

        // Guarda el último elemento del archivo si no termina en línea en blanco
        if !current_id.is_empty() {
            items.push(Item {
                id: current_id,
                name: current_name,
                item_type: current_type,
                flat_value: current_flat,
                multiplier: current_mult,
            });
        }

        items
    }
}

/// Un Item ya colocado en el mundo, esperando a que el jugador lo recoja.
#[derive(Debug, Clone)]
pub struct ItemPickup {
    pub item: Item,
    pub x: f32,
    pub y: f32,
}