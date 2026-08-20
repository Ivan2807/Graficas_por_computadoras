mod colors;
mod debug2d;
mod enemy;
mod framebuffer;
mod item;
mod level;
mod minimap;
mod player;
mod raycaster;
mod textures;
mod ui;
mod weapon;

use enemy::Enemy;
use framebuffer::Framebuffer;
use item::{Item, ItemPickup, ItemType};
use level::generate_level;
use minimap::render_minimap;
use player::Player;
use raylib::prelude::*;
use std::collections::HashSet;
use textures::Textures;
use ui::{render_hud, PlayerStats, HUD_HEIGHT};
use weapon::{Weapon, WeaponDef};

const SCREEN_WIDTH: i32 = 1280;
const SCREEN_HEIGHT: i32 = 720;

const FB_WIDTH: usize = 320;
const FB_HEIGHT: usize = 180;

const MOVE_SPEED: f32 = 3.0;
const ROT_SPEED: f32 = 2.5;
const MOUSE_SENSITIVITY: f32 = 0.003;
const LOOK_DEADZONE: f32 = 45.0;
const VERTICAL_DEADZONE: f32 = 30.0;
const WEAPON_RETURN_SPEED: f32 = 6.0;

const LEVEL_COLS: usize = 4;
const LEVEL_ROWS: usize = 4;
const LEVEL_MIN_ROOMS: usize = 8;
const LEVEL_FILL_CHANCE: i32 = 70;

const MAX_ENEMIES_PER_ROOM: i32 = 5;
const ENEMY_ROOM_CHANCE: i32 = 65;
const DOOR_SHOOT_RANGE: f32 = 20.0;
const REQUIRED_KEYS: u8 = 2;
const PISTOL_AMMO_DROP: i32 = 12;

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(SCREEN_WIDTH, SCREEN_HEIGHT)
        .title("Roguelike Raycaster")
        .build();

    rl.set_target_fps(60);
    rl.disable_cursor();

    let templates = level::load_room_templates("assets/rooms");
    let items_catalog: Vec<Item> = Item::parse_from_file("assets/rooms/item.txt");
    let weapon_defs: Vec<WeaponDef> = WeaponDef::parse_from_file("assets/weapons/weapons.txt");
    let textures = Textures::load_all();

    let mut level = generate_level(
        &templates,
        LEVEL_COLS,
        LEVEL_ROWS,
        LEVEL_MIN_ROOMS,
        LEVEL_FILL_CHANCE,
        |min, max| {
            let hi = (max - 1).max(min);
            rl.get_random_value::<i32>(min..hi)
        },
    );

    let spawn_cell = level.room_cells.first().copied().unwrap_or((0, 0));
    let mut player = Player::new(
        spawn_cell.0 as f32 * level.cell_w as f32 + level.cell_w as f32 / 2.0,
        spawn_cell.1 as f32 * level.cell_h as f32 + level.cell_h as f32 / 2.0,
    );

    let mut stats = PlayerStats::default();
    stats.init_ammo(&weapon_defs);

    let mut current_weapon_idx = weapon_defs.iter().position(|d| d.id == "pistol").unwrap_or(0);
    let mut weapon = Weapon::new(&weapon_defs[current_weapon_idx].id, &weapon_defs[current_weapon_idx].name);

    let mut fb = Framebuffer::new(FB_WIDTH, FB_HEIGHT);
    let mut enemies: Vec<Enemy> = Vec::new();

    let key_item_template: Option<Item> = items_catalog
        .iter()
        .find(|i| matches!(i.item_type, ItemType::Key))
        .cloned();

    // --- Colocar items en el nivel ---
    let mut item_pickups: Vec<ItemPickup> = Vec::new();
    if !items_catalog.is_empty() {
        let health_or_damage: Vec<&Item> = items_catalog
            .iter()
            .filter(|i| matches!(i.item_type, ItemType::Health | ItemType::Damage))
            .collect();

        for &(cx, cy) in &level.room_cells {
            if (cx, cy) == spawn_cell {
                continue;
            }
            let center_x = cx as f32 * level.cell_w as f32 + level.cell_w as f32 / 2.0;
            let center_y = cy as f32 * level.cell_h as f32 + level.cell_h as f32 / 2.0;

            let is_red_room = level.room_color.get(&(cx, cy)) == Some(&level::RED_ROOM_ID);

            if is_red_room && !health_or_damage.is_empty() {
                let idx =
                    rl.get_random_value::<i32>(0..(health_or_damage.len() as i32 - 1).max(0)) as usize;
                item_pickups.push(ItemPickup {
                    item: health_or_damage[idx].clone(),
                    x: center_x - 0.5,
                    y: center_y,
                });
                if let Some(key) = &key_item_template {
                    item_pickups.push(ItemPickup {
                        item: key.clone(),
                        x: center_x + 0.5,
                        y: center_y,
                    });
                }
                continue;
            }

            if rl.get_random_value::<i32>(0..99) < 60 {
                let last = (items_catalog.len() as i32 - 1).max(0);
                let idx = rl.get_random_value::<i32>(0..last) as usize;
                item_pickups.push(ItemPickup {
                    item: items_catalog[idx].clone(),
                    x: center_x,
                    y: center_y,
                });
            }
        }
    }

    // --- Colocar enemigos (evitando spawnear encima de paredes) ---
    for &(cx, cy) in &level.room_cells {
        if (cx, cy) == spawn_cell {
            continue;
        }
        if rl.get_random_value::<i32>(0..99) >= ENEMY_ROOM_CHANCE {
            continue;
        }
        let count = rl.get_random_value::<i32>(1..MAX_ENEMIES_PER_ROOM);
        let center_x = cx as f32 * level.cell_w as f32 + level.cell_w as f32 / 2.0;
        let center_y = cy as f32 * level.cell_h as f32 + level.cell_h as f32 / 2.0;

        for _ in 0..count {
            let mut spawn_x = center_x;
            let mut spawn_y = center_y;
            const SPAWN_ATTEMPTS: i32 = 12;
            for _ in 0..SPAWN_ATTEMPTS {
                let ox = rl.get_random_value::<i32>(-2..2) as f32;
                let oy = rl.get_random_value::<i32>(-2..2) as f32;
                let tx = (center_x + ox).floor() as i32;
                let ty = (center_y + oy).floor() as i32;
                if !level.is_wall(tx, ty) {
                    spawn_x = center_x + ox + 0.5;
                    spawn_y = center_y + oy + 0.5;
                    break;
                }
            }
            enemies.push(Enemy::new(spawn_x, spawn_y, 50));
        }
    }

    // --- Sellar (café) los cuartos que arrancan con enemigos ---
    let locked_link_indices: Vec<usize> = level
        .door_links
        .iter()
        .enumerate()
        .filter(|(_, link)| {
            let a = enemies.iter().any(|e| level.room_at(e.x, e.y) == Some(link.room_a));
            let b = enemies.iter().any(|e| level.room_at(e.x, e.y) == Some(link.room_b));
            a || b
        })
        .map(|(i, _)| i)
        .collect();
    for idx in locked_link_indices {
        level.force_lock_link(idx);
    }

    let image = Image::gen_image_color(FB_WIDTH as i32, FB_HEIGHT as i32, Color::BLACK);
    let mut texture = rl
        .load_texture_from_image(&thread, &image)
        .expect("no se pudo crear la textura del framebuffer");

    let mut view_3d = true;
    let mut debug_mode = false;
    let mut level_complete = false;
    let mut refilled_rooms: HashSet<(usize, usize)> = HashSet::new();

    let mut feedback_msg: Option<String> = None;
    let mut feedback_timer: f32 = 0.0;
    const HEAL_AMOUNT: i32 = 25;
    const FEEDBACK_DURATION: f32 = 1.5;

    while !rl.window_should_close() {
        let dt = rl.get_frame_time();
        stats.update_streak_timer(dt);

        if rl.is_key_pressed(KeyboardKey::KEY_ZERO) {
            view_3d = !view_3d;
        }
        if rl.is_key_pressed(KeyboardKey::KEY_F1) {
            debug_mode = !debug_mode;
        }
        if debug_mode {
            if rl.is_key_pressed(KeyboardKey::KEY_N) {
                let spawn_dist = 2.0;
                let ex = player.x + player.angle.cos() * spawn_dist;
                let ey = player.y + player.angle.sin() * spawn_dist;
                enemies.push(Enemy::new(ex, ey, 50));
            }
            if rl.is_key_pressed(KeyboardKey::KEY_P) {
                let def = &weapon_defs[current_weapon_idx];
                stats.clip_ammo.insert(def.id.clone(), def.clip_size);
            }
        }

        if rl.is_key_pressed(KeyboardKey::KEY_E) {
            let before = stats.health;
            stats.health = (stats.health + HEAL_AMOUNT).min(stats.max_health);
            let healed = stats.health - before;
            feedback_msg = Some(if healed > 0 {
                format!("+{} HP", healed)
            } else {
                "Vida al máximo".to_string()
            });
            feedback_timer = FEEDBACK_DURATION;
        }

        if feedback_timer > 0.0 {
            feedback_timer -= dt;
            if feedback_timer <= 0.0 {
                feedback_msg = None;
            }
        }

        // --- Cambiar de arma (1-9) ---
        const NUM_KEYS: [KeyboardKey; 9] = [
            KeyboardKey::KEY_ONE, KeyboardKey::KEY_TWO, KeyboardKey::KEY_THREE,
            KeyboardKey::KEY_FOUR, KeyboardKey::KEY_FIVE, KeyboardKey::KEY_SIX,
            KeyboardKey::KEY_SEVEN, KeyboardKey::KEY_EIGHT, KeyboardKey::KEY_NINE,
        ];
        for (i, key) in NUM_KEYS.iter().enumerate() {
            if i >= weapon_defs.len() {
                break;
            }
            if rl.is_key_pressed(*key) {
                current_weapon_idx = i;
                weapon.switch_to(&weapon_defs[i].id, &weapon_defs[i].name);
            }
        }

        // --- Recargar ---
        if rl.is_key_pressed(KeyboardKey::KEY_R) {
            let def = &weapon_defs[current_weapon_idx];
            stats.reload(&def.id, def.clip_size);
        }

        // --- Disparo ---
        if rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) && !weapon.is_shooting {
            let current_def = weapon_defs[current_weapon_idx].clone();
            if stats.consume_clip(&current_def.id) {
                weapon.is_shooting = true;

                const HIT_CONE: f32 = 0.12;
                const HIT_RANGE: f32 = 20.0;

                let player_room = level.room_at(player.x, player.y);

                let mut closest: Option<(usize, f32)> = None;
                for (i, e) in enemies.iter().enumerate() {
                    if !e.is_alive {
                        continue;
                    }
                    if level.room_at(e.x, e.y) != player_room {
                        continue;
                    }
                    let ex = e.x - player.x;
                    let ey = e.y - player.y;
                    let dist = (ex * ex + ey * ey).sqrt();
                    if dist <= 0.0001 || dist > HIT_RANGE {
                        continue;
                    }
                    let angle_to_enemy = ey.atan2(ex);
                    let mut diff = angle_to_enemy - player.angle;
                    while diff > std::f32::consts::PI {
                        diff -= std::f32::consts::TAU;
                    }
                    while diff < -std::f32::consts::PI {
                        diff += std::f32::consts::TAU;
                    }
                    if diff.abs() <= HIT_CONE {
                        if closest.map_or(true, |(_, d)| dist < d) {
                            closest = Some((i, dist));
                        }
                    }
                }

                if let Some((idx, _)) = closest {
                    let damage = stats.get_final_damage(current_def.damage) as i32;
                    enemies[idx].take_damage(damage);
                    if !enemies[idx].is_alive {
                        // --- Drop de municion: siempre pistola + 50% otra arma random ---
                        if let Some(pistol_def) = weapon_defs.iter().find(|d| d.id == "pistol") {
                            stats.add_reserve("pistol", PISTOL_AMMO_DROP, pistol_def.reserve);
                        }
                        if rl.get_random_value::<i32>(0..99) < 50 {
                            let others: Vec<&WeaponDef> =
                                weapon_defs.iter().filter(|d| d.id != "pistol").collect();
                            if !others.is_empty() {
                                let pick =
                                    rl.get_random_value::<i32>(0..(others.len() as i32 - 1).max(0)) as usize;
                                let other = others[pick.min(others.len() - 1)];
                                stats.add_reserve(&other.id, other.clip_size, other.reserve);
                            }
                        }

                        let dropped_key = stats.register_kill();
                        if dropped_key {
                            if let Some(key) = &key_item_template {
                                item_pickups.push(ItemPickup {
                                    item: key.clone(),
                                    x: enemies[idx].x,
                                    y: enemies[idx].y,
                                });
                            }
                        }
                    }
                } else if weapon.can_open_doors() {
                    // Solo el lanzacohetes abre puertas cerradas a distancia.
                    let hit = raycaster::cast_ray(&level, player.x, player.y, player.angle);
                    if hit.is_locked_door && hit.distance <= DOOR_SHOOT_RANGE {
                        if let Some(idx) = level.find_door_link_at(hit.tile_x as usize, hit.tile_y as usize) {
                            level.begin_open_link(idx);
                        }
                    }
                }
            }
        }
        weapon.update(dt);

        // --- Clic derecho: "usar" una puerta cerrada de cerca (cualquier arma) ---
        if rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_RIGHT) {
            const USE_RANGE: f32 = 3.0;
            let hit = raycaster::cast_ray(&level, player.x, player.y, player.angle);
            if hit.is_locked_door && hit.distance <= USE_RANGE {
                if let Some(idx) = level.find_door_link_at(hit.tile_x as usize, hit.tile_y as usize) {
                    level.begin_open_link(idx);
                }
            }
        }

        // --- Movimiento ---
        let mut move_dir = 0.0f32;
        if rl.is_key_down(KeyboardKey::KEY_W) { move_dir += 1.0; }
        if rl.is_key_down(KeyboardKey::KEY_S) { move_dir -= 1.0; }
        if rl.is_key_down(KeyboardKey::KEY_A) { player.angle -= ROT_SPEED * dt; }
        if rl.is_key_down(KeyboardKey::KEY_D) { player.angle += ROT_SPEED * dt; }

        let mouse_delta = rl.get_mouse_delta();
        let camera_delta = weapon.apply_look(
            mouse_delta.x,
            mouse_delta.y,
            dt,
            LOOK_DEADZONE,
            VERTICAL_DEADZONE,
            WEAPON_RETURN_SPEED,
        );
        player.angle += camera_delta * MOUSE_SENSITIVITY;

        let dx = player.angle.cos() * move_dir * MOVE_SPEED * dt;
        let dy = player.angle.sin() * move_dir * MOVE_SPEED * dt;
        player.try_move(dx, dy, &level);

        level.update_exploration(player.x, player.y);
        level.update_doors(dt);

        // --- Enemigos atacan al jugador ---
        for e in enemies.iter_mut() {
            if let Some(dmg) = e.update_attack(dt, player.x, player.y) {
                if stats.shield > 0 {
                    let absorbed = dmg.min(stats.shield);
                    stats.shield -= absorbed;
                    let remaining = dmg - absorbed;
                    if remaining > 0 {
                        stats.health = (stats.health - remaining).max(0);
                    }
                } else {
                    stats.health = (stats.health - dmg).max(0);
                }
            }
        }

        // Red de seguridad: si ya no quedan enemigos vivos a ningun lado de
        // una puerta cerrada, se abre sola.
        for i in 0..level.door_links.len() {
            let link = level.door_links[i];
            let a_clear = !enemies.iter().any(|e| e.is_alive && level.room_at(e.x, e.y) == Some(link.room_a));
            let b_clear = !enemies.iter().any(|e| e.is_alive && level.room_at(e.x, e.y) == Some(link.room_b));
            if a_clear && b_clear {
                level.begin_open_link(i);
            }
        }

        // --- +25% municion al despejar un cuarto (una sola vez por cuarto) ---
        let rooms_with_enemies: HashSet<(usize, usize)> =
            enemies.iter().filter_map(|e| level.room_at(e.x, e.y)).collect();
        for room in rooms_with_enemies {
            if refilled_rooms.contains(&room) {
                continue;
            }
            let alive_here = enemies.iter().any(|e| e.is_alive && level.room_at(e.x, e.y) == Some(room));
            if !alive_here {
                for def in &weapon_defs {
                    let add = (def.reserve as f32 * 0.25).round() as i32;
                    stats.add_reserve(&def.id, add, def.reserve);
                }
                refilled_rooms.insert(room);
                feedback_msg = Some("Cuarto despejado: +25% municion".to_string());
                feedback_timer = FEEDBACK_DURATION;
            }
        }

        let room_now = level.room_at(player.x, player.y);

        if !level_complete {
            if let Some(vault) = level.vault_cell {
                if room_now == Some(vault) && stats.keys >= REQUIRED_KEYS {
                    level_complete = true;
                    feedback_msg = Some("¡Llaves usadas! Nivel completado".to_string());
                    feedback_timer = 3.0;
                }
            }
        }

        const PICKUP_RADIUS: f32 = 0.5;
        item_pickups.retain(|pickup| {
            let ddx = pickup.x - player.x;
            let ddy = pickup.y - player.y;
            if ddx * ddx + ddy * ddy < PICKUP_RADIUS * PICKUP_RADIUS {
                stats.apply_item(pickup.item.clone());
                false
            } else {
                true
            }
        });

        if view_3d {
            raycaster::render(&mut fb, &level, &player, &textures);
            texture
                .update_texture(&fb.pixels)
                .expect("fallo al actualizar la textura");
        }

        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::BLACK);

        if view_3d {
            let game_view_h = (SCREEN_HEIGHT - HUD_HEIGHT) as f32;

            d.draw_texture_pro(
                &texture,
                Rectangle::new(0.0, 0.0, FB_WIDTH as f32, FB_HEIGHT as f32),
                Rectangle::new(0.0, 0.0, SCREEN_WIDTH as f32, game_view_h),
                Vector2::new(0.0, 0.0),
                0.0,
                Color::WHITE,
            );

            weapon.render(&mut d, SCREEN_WIDTH, game_view_h);

            let current_def = &weapon_defs[current_weapon_idx];
            let final_dmg = stats.get_final_damage(current_def.damage);
            render_hud(
                &mut d,
                SCREEN_WIDTH,
                SCREEN_HEIGHT,
                &stats,
                &current_def.name,
                stats.clip(&current_def.id),
                stats.reserve(&current_def.id),
                final_dmg,
            );
            render_minimap(&mut d, &level, &player, SCREEN_WIDTH);

            if let Some(msg) = &feedback_msg {
                let font_size = 26;
                let text_w = d.measure_text(msg, font_size);
                d.draw_text(
                    msg,
                    (SCREEN_WIDTH - text_w) / 2,
                    SCREEN_HEIGHT - HUD_HEIGHT - 36,
                    font_size,
                    Color::LIME,
                );
            }
        } else {
            debug2d::render_2d(
                &mut d,
                &level,
                &player,
                &enemies,
                &item_pickups,
                SCREEN_WIDTH,
                SCREEN_HEIGHT,
                debug_mode,
            );
        }

        d.draw_fps(10, 10);
    }
}