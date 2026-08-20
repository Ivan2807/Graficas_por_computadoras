mod colors;
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
use item::{Item, ItemPickup, ItemType};
use level::{generate_level, load_final_room, load_room_templates, Level};
use minimap::render_minimap;
use player::Player;
use raycaster::cast_ray;
use raylib::prelude::*;
use raylib::audio::RaylibAudio;
use std::collections::HashSet;
use textures::Textures;
use ui::{render_hud, PlayerStats, RadarSweep};
use weapon::{Weapon, WeaponDef};

const SCREEN_W: i32 = 1280;
const SCREEN_H: i32 = 720;

#[derive(Clone, Copy, PartialEq)]
enum GameMode {
    Normal,
    Hard,
    Taylor,
}

fn choose_mode(rl: &mut RaylibHandle, thread: &RaylibThread) -> Option<GameMode> {
    let mut screen = 0u8;
    let audio = RaylibAudio::init_audio_device().ok();
    let mut menu_music = audio.as_ref()
        .and_then(|audio| audio.new_music("assets/Songs/Intro.m4a").ok());
    if let Some(track) = menu_music.as_mut() { track.play_stream(); }
    while !rl.window_should_close() {
        let mouse = rl.get_mouse_position();
        let clicked = rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT);
        if let Some(track) = menu_music.as_mut() { track.update_stream(); }
        let mut d = rl.begin_drawing(thread);
        d.clear_background(Color::new(12, 12, 20, 255));
        d.draw_text(if screen == 0 { "INTRO" } else { "MODO DE JUEGO" }, 470, 100, 42, Color::WHITE);

        if screen == 0 {
            draw_menu_button(&mut d, 440, 260, 400, 70, "JUGAR", mouse);
            draw_menu_button(&mut d, 440, 360, 400, 70, "SALIR", mouse);
            if clicked {
                if mouse_in(mouse, 440, 260, 400, 70) {
                    screen = 1;
                    if let Some(track) = menu_music.as_mut() { track.stop_stream(); }
                    menu_music = audio.as_ref()
                        .and_then(|audio| audio.new_music("assets/Songs/(Audio) Taylor Swift - Blank Space.m4a").ok());
                    if let Some(track) = menu_music.as_mut() { track.play_stream(); }
                }
                if mouse_in(mouse, 440, 360, 400, 70) {
                    if let Some(track) = menu_music.as_mut() {
                        track.stop_stream();
                        if let Ok(exit_track) = audio.as_ref().unwrap().new_music("assets/Songs/Shake It Off.m4a") {
                            exit_track.play_stream();
                        }
                    }
                    return None;
                }
            }
        } else {
            draw_menu_button(&mut d, 360, 210, 560, 65, "NORMAL", mouse);
            draw_menu_button(&mut d, 360, 300, 560, 65, "DIFICIL", mouse);
            draw_menu_button(&mut d, 360, 390, 560, 65, "TAYLOR", mouse);
            if clicked {
                if mouse_in(mouse, 360, 210, 560, 65) { return Some(GameMode::Normal); }
                if mouse_in(mouse, 360, 300, 560, 65) { return Some(GameMode::Hard); }
                if mouse_in(mouse, 360, 390, 560, 65) { return Some(GameMode::Taylor); }
            }
        }
    }
    None
}

fn mouse_in(mouse: Vector2, x: i32, y: i32, width: i32, height: i32) -> bool {
    mouse.x >= x as f32 && mouse.x <= (x + width) as f32 && mouse.y >= y as f32 && mouse.y <= (y + height) as f32
}

fn draw_menu_button(d: &mut RaylibDrawHandle, x: i32, y: i32, width: i32, height: i32, label: &str, mouse: Vector2) {
    let hovered = mouse_in(mouse, x, y, width, height);
    let color = if hovered { Color::PURPLE } else { Color::DARKGRAY };
    d.draw_rectangle(x, y, width, height, color);
    d.draw_rectangle_lines(x, y, width, height, Color::RAYWHITE);
    d.draw_text(label, x + width / 2 - label.len() as i32 * 7, y + 20, 24, Color::WHITE);
}

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(SCREEN_W, SCREEN_H)
        .title("Doom Rooms - Raycasting Survival")
        .build();
    rl.set_target_fps(60);
    let Some(game_mode) = choose_mode(&mut rl, &thread) else { return; };
    rl.disable_cursor();
    let audio = RaylibAudio::init_audio_device().ok();
    let music_path = match game_mode {
        GameMode::Taylor => "assets/Songs/(Audio) Taylor Swift - Blank Space.m4a",
        GameMode::Normal | GameMode::Hard => "assets/Songs/Normal.m4a",
    };
    let mut music = audio.as_ref().and_then(|audio| audio.new_music(music_path).ok());
    if let Some(track) = music.as_mut() {
        track.play_stream();
    }

    let templates = load_room_templates("assets/rooms");
    let final_template = load_final_room("assets/rooms/room_final.txt");
    let mut grid_size = 4usize;
    let mut map = generate_level(&templates, grid_size, grid_size, 8, 75, random_range);
    let final_room = *map.room_cells.last().unwrap();
    map.replace_room(final_room, &final_template);
    let mut red_rooms = apply_red_rooms(&mut map, &templates, final_room);
    let textures = Textures::load_all();
    let weapon_defs = load_weapon_defs();
    let mut stats = PlayerStats::default();
    for def in &weapon_defs {
        stats.clip_ammo.insert(def.id.clone(), def.clip_size);
        stats.reserve_ammo.insert(def.id.clone(), def.reserve);
    }

    let spawn = map.room_cells.first().copied().unwrap_or((0, 0));
    let mut player = Player::new(
        spawn.0 as f32 * map.cell_w as f32 + map.cell_w as f32 / 2.0,
        spawn.1 as f32 * map.cell_h as f32 + map.cell_h as f32 / 2.0,
    );
    let mut enemies = spawn_enemies(&map, game_mode);
    let mut pickups = load_pickups(&map, final_room);
    let mut cleared_rooms = HashSet::new();
    let mut rewarded_red_rooms = HashSet::new();
    let mut rewarded_enemies = HashSet::new();
    let mut visited_rooms = HashSet::new();
    let item_defs = Item::parse_from_file("assets/items/item.txt");
    let mut radar = RadarSweep::new();
    let mut weapon_index = 0usize;
    let mut weapon = Weapon::new(&weapon_defs[0].id, &weapon_defs[0].name);
    let weapon_sprites = load_weapon_sprites(&mut rl, &thread, &weapon_defs);
    let enemy_sprite_1 = rl.load_texture(&thread, "assets/sprites/Enemy_p1.png").ok();
    let enemy_sprite_2 = rl.load_texture(&thread, "assets/sprites/Enemy_p2.png").ok();
    let tailor_sprite = rl.load_texture(&thread, "assets/sprites/tailor.png").ok();
    let mut hit_marker_timer = 0.0f32;
    let mut automatic_timer = 0.0f32;
    let mut global_message_timer = 0.0f32;
    let mut global_message = String::new();
    let mut key_chance = 25i32;
    let mut rooms_since_key = 0u8;
    let mut level_timer = 90.0f32;
    let mut mega_stage = 0u32;
    let mut mega_id = None;
    let mut space_uses = 0u8;
    let mut escape_mode = false;
    let mut transition_timer = 0.0f32;

    while !rl.window_should_close() {
        let dt = rl.get_frame_time();
        if let Some(track) = music.as_mut() {
            track.update_stream();
        }

        if transition_timer > 0.0 {
            transition_timer -= dt;
            let mut d = rl.begin_drawing(&thread);
            d.clear_background(Color::BLACK);
            d.draw_text("NIVEL 2", SCREEN_W / 2 - 70, SCREEN_H / 2 - 20, 40, Color::WHITE);
            if transition_timer <= 0.0 {
                grid_size = 5;
                map = generate_level(&templates, grid_size, grid_size, 12, 75, random_range);
                let next_final_room = *map.room_cells.last().unwrap();
                map.replace_room(next_final_room, &final_template);
                red_rooms = apply_red_rooms(&mut map, &templates, next_final_room);
                let next_spawn = map.room_cells.first().copied().unwrap_or((0, 0));
                player.pos_x = next_spawn.0 as f32 * map.cell_w as f32 + map.cell_w as f32 / 2.0;
                player.pos_y = next_spawn.1 as f32 * map.cell_h as f32 + map.cell_h as f32 / 2.0;
                enemies = spawn_enemies(&map, game_mode);
                pickups = load_pickups(&map, next_final_room);
                cleared_rooms.clear();
                rewarded_red_rooms.clear();
                rewarded_enemies.clear();
                visited_rooms.clear();
                stats.keys = 0;
                key_chance = 25;
                rooms_since_key = 0;
                level_timer = 90.0;
                mega_stage = 0;
                mega_id = None;
                space_uses = 0;
                escape_mode = false;
            }
            continue;
        }

        let mouse_delta = rl.get_mouse_delta();
        let camera_delta = weapon.apply_look(mouse_delta.x, mouse_delta.y, dt, 90.0, 55.0, 0.0);
        player.angle = (player.angle + camera_delta * 0.0025).rem_euclid(std::f32::consts::TAU);

        for (index, key) in [
            KeyboardKey::KEY_ONE,
            KeyboardKey::KEY_TWO,
            KeyboardKey::KEY_THREE,
            KeyboardKey::KEY_FOUR,
            KeyboardKey::KEY_FIVE,
            KeyboardKey::KEY_SIX,
        ]
        .iter()
        .enumerate()
        {
            if index < weapon_defs.len() && rl.is_key_pressed(*key) {
                weapon_index = index;
                weapon.switch_to(&weapon_defs[index].id, &weapon_defs[index].name);
            }
        }

        let current_weapon = &weapon_defs[weapon_index];

        player.handle_input(&rl, dt, &map);
        map.update_doors(dt);
        map.update_exploration(player.pos_x, player.pos_y);
        let current_room = map.room_at(player.pos_x, player.pos_y);
        if let Some(room) = current_room {
            if visited_rooms.insert(room) {
                apply_room_entry_reward(&item_defs, &weapon_defs, &mut stats);
                if room != map.room_cells[0] {
                    rooms_since_key += 1;
                }
                if room != map.room_cells[0]
                    && rooms_since_key >= 2
                    && random_range(0, 99) < key_chance
                {
                    stats.keys = (stats.keys + 1).min(3);
                    key_chance = 25;
                    rooms_since_key = 0;
                    global_message = format!("LLAVE ENCONTRADA: {}/3", stats.keys);
                    global_message_timer = 1.5;
                } else if room != map.room_cells[0] && rooms_since_key >= 2 {
                    key_chance = (key_chance + 25).min(100);
                    rooms_since_key = 0;
                }
            }
        }
        apply_red_room_rewards(
            &map,
            map.room_at(player.pos_x, player.pos_y),
            &red_rooms,
            &item_defs,
            &weapon_defs,
            &mut stats,
            &mut rewarded_red_rooms,
        );

        if rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_RIGHT) {
            interact_with_door(&mut map, &player);
        }
        if rl.is_key_pressed(KeyboardKey::KEY_R) {
            reload_weapon(&mut stats, current_weapon);
        }
        if rl.is_key_pressed(KeyboardKey::KEY_F) {
            if throw_bomb(&mut map, &mut enemies, &mut stats, &player) {
                global_message = "BOMBA DETONADA".to_string();
                global_message_timer = 1.0;
            }
        }
        if rl.is_key_pressed(KeyboardKey::KEY_SPACE) && space_uses < 5 {
            if use_health_item(&mut stats) {
                space_uses += 1;
                global_message = "VIDA APLICADA".to_string();
                global_message_timer = 1.0;
            }
        }

        automatic_timer = (automatic_timer - dt).max(0.0);
        let automatic_weapon = current_weapon.id == "smg" || current_weapon.id == "rifle";
        let should_fire = rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT)
            || (automatic_weapon
                && rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT)
                && automatic_timer <= 0.0);
        if should_fire {
            weapon.is_shooting = true;
            if fire_weapon(&mut map, &mut enemies, &mut stats, &mut weapon, current_weapon, &player) {
                hit_marker_timer = 0.18;
            }
            automatic_timer = weapon_fire_interval(current_weapon);
        }
        for enemy in &enemies {
            if !enemy.is_alive() && rewarded_enemies.insert(enemy.id) {
                grant_kill_ammo(&mut stats, &weapon_defs);
            }
        }
        weapon.update(dt);
        hit_marker_timer = (hit_marker_timer - dt).max(0.0);
        global_message_timer = (global_message_timer - dt).max(0.0);

        level_timer -= dt;
        if level_timer <= 0.0 && mega_id.is_none() {
            if let Some(mega) = spawn_mega(&map, &player, mega_stage) {
                mega_id = Some(mega.id);
                enemies.push(mega);
                global_message = "MEGA-MONSTRUO DETECTADO".to_string();
                global_message_timer = 2.0;
            }
        }

        if let Some(id) = mega_id {
            if enemies.iter().any(|enemy| enemy.id == id && !enemy.is_alive()) {
                mega_id = None;
                mega_stage += 1;
                level_timer = 60.0;
                global_message = "MEGA ELIMINADO: 60 SEGUNDOS".to_string();
                global_message_timer = 2.0;
            }
        }

        for enemy in &mut enemies {
            let player_room = map.room_at(player.pos_x, player.pos_y);
            let enemy_room = map.room_at(enemy.x, enemy.y);
            let can_use_room_path = enemy.is_mega || (player_room.is_some()
                && enemy_room.is_some()
                && map.rooms_are_open(player_room.unwrap(), enemy_room.unwrap()));
            if enemy.is_mega && escape_mode {
                enemy.speed = 1.0;
                enemy.attack_damage = 75;
            }
            if can_use_room_path && enemy.update(dt, player.pos_x, player.pos_y, &map) {
                stats.take_damage(enemy.attack_damage);
            }
        }
        heal_cleared_rooms(&map, &enemies, &red_rooms, &mut stats, &mut cleared_rooms);

        pickups.retain(|pickup| {
            let collected = distance(player.pos_x, player.pos_y, pickup.x, pickup.y) < 0.7;
            if collected {
                stats.apply_item(pickup.item.clone());
            }
            !collected
        });

        let at_final_room = map.room_at(player.pos_x, player.pos_y) == Some(*map.room_cells.last().unwrap());
        if stats.keys >= 3 && !escape_mode {
            escape_mode = true;
            level_timer = level_timer.min(30.0);
            global_message = "ESCAPE INMINENTE - SAL DE AHI".to_string();
            global_message_timer = 3.0;
        }
        if escape_mode && at_final_room && transition_timer <= 0.0 {
            transition_timer = 3.0;
        }
        let player_room = map.room_at(player.pos_x, player.pos_y);
        let room_enemies: Vec<Enemy> = enemies
            .iter()
            .filter(|enemy| {
                enemy.is_mega || player_room
                    .zip(map.room_at(enemy.x, enemy.y))
                    .map(|(player_room, enemy_room)| map.rooms_are_open(player_room, enemy_room))
                    .unwrap_or(false)
            })
            .cloned()
            .collect();
        radar.update(dt, player.pos_x, player.pos_y, player.angle, &room_enemies);
        if game_mode == GameMode::Taylor && !radar.pings.is_empty() {
            if let Some(track) = music.as_mut() {
                if !track.is_stream_playing() {
                    track.play_stream();
                }
            }
        }
        let fps = rl.get_fps();

        let mut d = rl.begin_drawing(&thread);
        render_raycast(&mut d, &map, &player, &textures, SCREEN_W, SCREEN_H - ui::HUD_HEIGHT);
        render_visible_enemies(
            &mut d,
            &map,
            &player,
            &enemies,
            &radar,
            game_mode,
            enemy_sprite_1.as_ref(),
            enemy_sprite_2.as_ref(),
            tailor_sprite.as_ref(),
            SCREEN_W,
            SCREEN_H - ui::HUD_HEIGHT,
        );
        render_radar_reveals(&mut d, &radar, SCREEN_W, SCREEN_H - ui::HUD_HEIGHT);
        render_minimap(&mut d, &map, &player, SCREEN_W);
        render_weapon_sprite(
            &mut d,
            weapon_sprites[weapon_index].0.as_ref(),
            weapon_sprites[weapon_index].1.as_ref(),
            &weapon,
            SCREEN_W,
            SCREEN_H,
        );
        render_hud(
            &mut d,
            SCREEN_W,
            SCREEN_H,
            &stats,
            &current_weapon.name,
            *stats.clip_ammo.get(&current_weapon.id).unwrap_or(&0),
            *stats.reserve_ammo.get(&current_weapon.id).unwrap_or(&0),
            &radar,
            level_timer,
        );
        if hit_marker_timer > 0.0 {
            render_hit_marker(&mut d, &weapon, SCREEN_W, SCREEN_H);
        }
        d.draw_text(&format!("FPS: {}", fps), 12, 12, 20, Color::LIME);
        d.draw_text(&format!("ESPACIO: {}/5", space_uses), 12, 36, 18, Color::WHITE);
        if global_message_timer > 0.0 {
            d.draw_text(&global_message, SCREEN_W / 2 - 90, 38, 20, Color::GOLD);
        }
    }
}

fn random_range(min: i32, max: i32) -> i32 {
    unsafe { raylib::ffi::GetRandomValue(min, max) }
}

fn load_weapon_defs() -> Vec<WeaponDef> {
    let mut defs = WeaponDef::parse_from_file("assets/weapons/weapons.txt");
    if defs.is_empty() {
        defs.push(WeaponDef { id: "pistol".into(), name: "Pistola".into(), damage: 10.0, clip_size: 12, reserve: 48 });
    }
    defs
}

fn load_weapon_sprites(
    rl: &mut RaylibHandle,
    thread: &RaylibThread,
    definitions: &[WeaponDef],
) -> Vec<(Option<Texture2D>, Option<Texture2D>)> {
    definitions.iter().map(|definition| {
        let prefix = match definition.id.as_str() {
            "pistol" => "Pistola",
            "shotgun" => "Escopeta",
            "smg" => "Smg",
            "rifle" => "Assault",
            "sniper" => "Sniper",
            "rocket" => "Rocket",
            _ => "Pistola",
        };
        (
            rl.load_texture(thread, &format!("assets/sprites/{}_1.png", prefix)).ok(),
            rl.load_texture(thread, &format!("assets/sprites/{}_2.png", prefix)).ok(),
        )
    }).collect()
}

fn spawn_enemies(map: &Level, mode: GameMode) -> Vec<Enemy> {
    let mut enemies = Vec::new();
    let mut next_id = 0;
    for &(cx, cy) in map.room_cells.iter().skip(1) {
        let count = random_range(3, 7) as usize;
        let mut positions = open_spawn_positions(map, cx, cy);
        for _ in 0..count {
            if positions.is_empty() {
                break;
            }
            let position_index = random_range(0, positions.len() as i32 - 1) as usize;
            let (x, y) = positions.swap_remove(position_index);
            let mut enemy = Enemy::new(next_id, x, y, (cx, cy));
            if mode == GameMode::Hard {
                enemy.health *= 1.1;
                enemy.max_health *= 1.1;
                enemy.speed *= 1.1;
            }
            enemies.push(enemy);
            next_id += 1;
        }
    }
    enemies
}

fn apply_room_entry_reward(item_defs: &[Item], weapon_defs: &[WeaponDef], stats: &mut PlayerStats) {
    let usable_items: Vec<&Item> = item_defs.iter().filter(|item| item.item_type != ItemType::Key).collect();
    if !usable_items.is_empty() {
        let item = usable_items[random_range(0, usable_items.len() as i32 - 1) as usize];
        stats.apply_item(item.clone());
    }
    if let Some(bomb) = item_defs.iter().find(|item| item.item_type == ItemType::SingleUse) {
        stats.apply_item(bomb.clone());
    }
    for definition in weapon_defs {
        *stats.reserve_ammo.entry(definition.id.clone()).or_insert(0) += 2;
    }
}

fn spawn_mega(map: &Level, player: &Player, stage: u32) -> Option<Enemy> {
    let current_room = map.room_at(player.pos_x, player.pos_y)?;
    let facing_x = player.angle.cos();
    let facing_y = player.angle.sin();
    let mut candidates = Vec::new();
    let back_room = (
        (current_room.0 as i32 - facing_x.signum() as i32 * 2) as usize,
        (current_room.1 as i32 - facing_y.signum() as i32 * 2) as usize,
    );
    if map.room_cells.contains(&back_room) {
        candidates.push(back_room);
    }
    let mut fallback: Vec<(usize, usize)> = map.room_cells.iter().copied()
        .filter(|room| *room != current_room)
        .filter(|room| {
            let dx = room.0 as i32 - current_room.0 as i32;
            let dy = room.1 as i32 - current_room.1 as i32;
            dx.abs() + dy.abs() >= 2
        })
        .collect();
    fallback.sort_by_key(|room| (room.1, room.0));
    candidates.extend(fallback);

    for room in candidates {
        let positions = open_spawn_positions(map, room.0, room.1);
        if positions.is_empty() { continue; }
        let (x, y) = positions[random_range(0, positions.len() as i32 - 1) as usize];
        let health = 250.0 * 2.5f32.powi(stage as i32);
        return Some(Enemy::mega(10000 + stage as usize, x, y, room, health));
    }
    None
}

fn open_spawn_positions(map: &Level, room_x: usize, room_y: usize) -> Vec<(f32, f32)> {
    let mut positions = Vec::new();
    for y in 0..map.cell_h {
        for x in 0..map.cell_w {
            let world_x = room_x * map.cell_w + x;
            let world_y = room_y * map.cell_h + y;
            if matches!(map.get_tile(world_x as i32, world_y as i32), level::Tile::Empty | level::Tile::Door) {
                positions.push((world_x as f32 + 0.5, world_y as f32 + 0.5));
            }
        }
    }
    positions
}

fn reload_weapon(stats: &mut PlayerStats, definition: &WeaponDef) {
    let clip = *stats.clip_ammo.get(&definition.id).unwrap_or(&0);
    let reserve = *stats.reserve_ammo.get(&definition.id).unwrap_or(&0);
    let needed = (definition.clip_size - clip).max(0).min(reserve);
    stats.clip_ammo.insert(definition.id.clone(), clip + needed);
    stats.reserve_ammo.insert(definition.id.clone(), reserve - needed);
}

fn weapon_fire_interval(definition: &WeaponDef) -> f32 {
    match definition.id.as_str() {
        "smg" => 0.09,
        "rifle" => 0.16,
        _ => 0.25,
    }
}

fn use_health_item(stats: &mut PlayerStats) -> bool {
    let item = if let Some(index) = stats.inventory.iter().position(|item| item.item_type == ItemType::Health) {
        stats.inventory.remove(index)
    } else if stats.health < stats.max_health || stats.shield < stats.max_shield {
        Item {
            id: "space_health_charge".to_string(),
            name: "Carga de recuperacion".to_string(),
            item_type: ItemType::Health,
            flat_value: 25.0,
            multiplier: 1.0,
        }
    } else {
        return false;
    };
    stats.apply_item(item);
    true
}

fn throw_bomb(map: &mut Level, enemies: &mut [Enemy], stats: &mut PlayerStats, player: &Player) -> bool {
    let Some(index) = stats.inventory.iter().position(|item| item.item_type == ItemType::SingleUse) else {
        return false;
    };
    stats.inventory.remove(index);

    let direction_x = player.angle.cos();
    let direction_y = player.angle.sin();
    let mut explosion_x = player.pos_x + direction_x * 2.0;
    let mut explosion_y = player.pos_y + direction_y * 2.0;
    let mut travel_distance = 0.5;
    while travel_distance <= 2.0 {
        let check_x = player.pos_x + direction_x * travel_distance;
        let check_y = player.pos_y + direction_y * travel_distance;
        if map.is_wall_or_locked(check_x, check_y) {
            explosion_x = check_x;
            explosion_y = check_y;
            if map.force_open_door(check_x.floor() as usize, check_y.floor() as usize) {
                break;
            }
            break;
        }
        travel_distance += 0.5;
    }

    let explosion_radius = 1.5;
    for enemy in enemies.iter_mut().filter(|enemy| enemy.is_alive()) {
        if distance(explosion_x, explosion_y, enemy.x, enemy.y) <= explosion_radius
            && has_line_of_sight(map, explosion_x, explosion_y, enemy.x, enemy.y)
        {
            enemy.health = (enemy.health - 100.0).max(0.0);
            if !enemy.is_alive() {
                stats.kills += 1;
                stats.score += 100;
            }
        }
    }
    true
}

fn has_line_of_sight(map: &Level, from_x: f32, from_y: f32, to_x: f32, to_y: f32) -> bool {
    let dx = to_x - from_x;
    let dy = to_y - from_y;
    let length = (dx * dx + dy * dy).sqrt();
    if length <= 0.1 {
        return true;
    }
    let steps = (length / 0.2).ceil() as usize;
    for step in 1..steps {
        let factor = step as f32 / steps as f32;
        if map.is_wall_or_locked(from_x + dx * factor, from_y + dy * factor) {
            return false;
        }
    }
    true
}

fn grant_kill_ammo(stats: &mut PlayerStats, weapon_defs: &[WeaponDef]) {
    *stats.reserve_ammo.entry("pistol".to_string()).or_insert(0) += 6;
    let other_weapons: Vec<&WeaponDef> = weapon_defs.iter().filter(|definition| definition.id != "pistol").collect();
    if !other_weapons.is_empty() && random_range(0, 99) < 50 {
        let definition = other_weapons[random_range(0, other_weapons.len() as i32 - 1) as usize];
        *stats.reserve_ammo.entry(definition.id.clone()).or_insert(0) += (definition.clip_size / 2).max(1);
    }
}

fn apply_red_rooms(map: &mut Level, templates: &[level::RoomTemplate], final_room: (usize, usize)) -> HashSet<(usize, usize)> {
    let Some(red_template) = templates.iter().find(|template| template.color_id == 1) else {
        return HashSet::new();
    };
    let rooms: Vec<(usize, usize)> = map.room_cells.iter().copied()
        .filter(|room| *room != map.room_cells[0] && *room != final_room)
        .take(3)
        .collect();
    for room in &rooms {
        map.replace_room(*room, red_template);
    }
    rooms.into_iter().collect()
}

fn apply_red_room_rewards(
    _map: &Level,
    current_room: Option<(usize, usize)>,
    red_rooms: &HashSet<(usize, usize)>,
    item_defs: &[Item],
    weapon_defs: &[WeaponDef],
    stats: &mut PlayerStats,
    rewarded_red_rooms: &mut HashSet<(usize, usize)>,
) {
    let Some(room) = current_room else { return; };
    if !red_rooms.contains(&room) || !rewarded_red_rooms.insert(room) {
        return;
    }

    for item in item_defs.iter().filter(|item| item.item_type != ItemType::Key) {
        stats.apply_item(item.clone());
    }
    for definition in weapon_defs {
        *stats.reserve_ammo.entry(definition.id.clone()).or_insert(0) += definition.clip_size;
    }
}

fn heal_cleared_rooms(
    map: &Level,
    enemies: &[Enemy],
    red_rooms: &HashSet<(usize, usize)>,
    stats: &mut PlayerStats,
    cleared_rooms: &mut HashSet<(usize, usize)>,
) {
    for room in map.room_cells.iter().copied() {
        let room_enemies: Vec<&Enemy> = enemies.iter().filter(|enemy| enemy.home_room == room).collect();
        if !room_enemies.is_empty()
            && room_enemies.iter().all(|enemy| !enemy.is_alive())
            && cleared_rooms.insert(room)
        {
            let recovery = (stats.max_health as f32 * 0.25).ceil() as i32;
            stats.health = (stats.health + recovery).min(stats.max_health);
            if red_rooms.contains(&room) {
                stats.keys += 1;
            }
        }
    }
}

fn load_pickups(map: &Level, final_room: (usize, usize)) -> Vec<ItemPickup> {
    let items = Item::parse_from_file("assets/items/item.txt");
    if items.is_empty() { return Vec::new(); }
    map.room_cells.iter().skip(1).enumerate().filter_map(|(index, &(cx, cy))| {
        if (cx, cy) == final_room { return None; }
        let usable_items: Vec<&Item> = items.iter().filter(|item| item.item_type != ItemType::Key).collect();
        let item = (*usable_items.get(index % usable_items.len())?).clone();
        Some(ItemPickup {
            item,
            x: cx as f32 * map.cell_w as f32 + map.cell_w as f32 / 2.0,
            y: cy as f32 * map.cell_h as f32 + map.cell_h as f32 / 2.0,
        })
    }).collect()
}

fn fire_weapon(
    map: &mut Level,
    enemies: &mut [Enemy],
    stats: &mut PlayerStats,
    weapon: &mut Weapon,
    definition: &WeaponDef,
    player: &Player,
) -> bool {
    let clip = stats.clip_ammo.entry(definition.id.clone()).or_insert(definition.clip_size);
    if *clip <= 0 { return false; }
    *clip -= 1;

    let hit = cast_ray(map, player.pos_x, player.pos_y, player.angle);
    if hit.is_locked_door && weapon.can_open_doors() {
        map.force_open_door(hit.tile_x as usize, hit.tile_y as usize);
        return false;
    }
    for enemy in enemies {
        if !enemy.is_alive() { continue; }
        let dx = enemy.x - player.pos_x;
        let dy = enemy.y - player.pos_y;
        let dist = (dx * dx + dy * dy).sqrt();
        let alignment = (dx * player.angle.cos() + dy * player.angle.sin()) / dist.max(0.001);
        if alignment > 0.94 && dist < hit.distance + 0.4 {
            enemy.health = (enemy.health - definition.damage * stats.damage_multiplier - stats.damage_bonus_flat).max(0.0);
            if !enemy.is_alive() { stats.kills += 1; stats.score += 100; }
            return true;
        }
    }
    false
}

fn interact_with_door(map: &mut Level, player: &Player) {
    let mut closest = None;
    let mut closest_distance = 2.2;
    for link in &map.door_links {
        for &(x, y) in &[link.tile_a, link.tile_b] {
            let dx = x as f32 + 0.5 - player.pos_x;
            let dy = y as f32 + 0.5 - player.pos_y;
            let distance = (dx * dx + dy * dy).sqrt();
            let alignment = (dx * player.angle.cos() + dy * player.angle.sin()) / distance.max(0.001);
            if distance < closest_distance && alignment > 0.3 {
                closest = Some((x, y));
                closest_distance = distance;
            }
        }
    }
    if let Some((x, y)) = closest { map.toggle_door(x, y); }
}

fn render_raycast(d: &mut RaylibDrawHandle, map: &Level, player: &Player, textures: &Textures, width: i32, height: i32) {
    d.draw_rectangle(0, 0, width, height / 2, Color::new(28, 32, 48, 255));
    d.draw_rectangle(0, height / 2, width, height / 2, Color::new(48, 40, 34, 255));
    for column in (0..width).step_by(2) {
        let camera = (column as f32 + 0.5) / width as f32 - 0.5;
        let angle = player.angle + camera * raycaster::FOV;
        let hit = cast_ray(map, player.pos_x, player.pos_y, angle);
        let corrected = (hit.distance * (angle - player.angle).cos()).max(0.05);
        let wall_height = (height as f32 / corrected).min(height as f32 * 4.0);
        let start = ((height as f32 - wall_height) / 2.0).max(0.0) as i32;
        let end = ((height as f32 + wall_height) / 2.0).min(height as f32) as i32;
        if hit.is_locked_door {
            let progress = map.door_progress_at(hit.tile_x as usize, hit.tile_y as usize);
            let visible_height = ((end - start) as f32 * (1.0 - progress)).round() as i32;
            let door_start = end - visible_height;
            if visible_height > 0 {
                d.draw_line(column, door_start, (column + 1).min(width - 1), end, Color::new(130, 78, 38, 255));
            }
            continue;
        }

        let texture = textures.for_wall_id(hit.wall_id);
        let wall_span = (end - start).max(1) as f32;
        for y in start..end {
            let v = (y - start) as f32 / wall_span;
            let mut color = texture.sample(hit.wall_u, v);
            if hit.side == 1 { color = colors::darken(color); }
            d.draw_line(column, y, (column + 1).min(width - 1), y, color);
        }
    }
}

fn render_visible_enemies(
    d: &mut RaylibDrawHandle,
    map: &Level,
    player: &Player,
    enemies: &[Enemy],
    radar: &RadarSweep,
    mode: GameMode,
    enemy_sprite_1: Option<&Texture2D>,
    enemy_sprite_2: Option<&Texture2D>,
    tailor_sprite: Option<&Texture2D>,
    width: i32,
    height: i32,
) {
    for enemy in enemies {
        let Some(ping) = radar.pings.iter().find(|ping| ping.enemy_id == enemy.id) else {
            continue;
        };
        let dx = enemy.x - player.pos_x;
        let dy = enemy.y - player.pos_y;
        let distance = (dx * dx + dy * dy).sqrt();
        let relative = (dy.atan2(dx) - player.angle + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU) - std::f32::consts::PI;
        if relative.abs() > raycaster::FOV / 2.0 || distance < 0.1 { continue; }
        let wall = cast_ray(map, player.pos_x, player.pos_y, dy.atan2(dx));
        if wall.distance + 0.3 < distance { continue; }
        let blink_phase = (ping.alpha * 12.0).floor() as i32;
        if blink_phase % 2 != 0 { continue; }
        let screen_x = width / 2 + (relative / raycaster::FOV * width as f32) as i32;
        let size = (height as f32 / distance).clamp(8.0, 150.0) as i32;
        let body_y = height / 2 - size / 2;
        let body_color = if enemy.is_mega {
            Color::PURPLE
        } else if enemy.is_alive() {
            Color::RED
        } else {
            enemy.corpse_color
        };
        if mode == GameMode::Taylor {
            if let Some(texture) = tailor_sprite {
                let phase = (ping.alpha * if enemy.is_mega { 28.0 } else { 14.0 }).floor() as i32;
                let scale = (size as f32 / texture.width as f32).max(0.1);
                d.draw_texture_ex(texture, Vector2::new(screen_x as f32 - size as f32 / 2.0, body_y as f32), 0.0, scale, Color::WHITE);
                if phase < 0 { continue; }
            }
        } else if let Some(texture) = if (ping.alpha * if enemy.is_mega { 28.0 } else { 14.0 }).floor() as i32 % 2 == 0 { enemy_sprite_1 } else { enemy_sprite_2 } {
            let scale = (size as f32 / texture.width as f32).max(0.1);
            d.draw_texture_ex(texture, Vector2::new(screen_x as f32 - size as f32 / 2.0, body_y as f32), 0.0, scale, Color::WHITE);
            continue;
        }
        let body_height = if enemy.is_alive() { size } else { (size / 3).max(4) };
        d.draw_rectangle(screen_x - size / 3, body_y, (size / 3).max(4), body_height, body_color);
        d.draw_circle(screen_x, body_y, (size / 3).max(4) as f32, body_color);
        d.draw_rectangle_lines(screen_x - size / 3, body_y, (size / 3).max(4), body_height, Color::BLACK);
    }
}

fn render_radar_reveals(d: &mut RaylibDrawHandle, radar: &RadarSweep, width: i32, height: i32) {
    for ping in &radar.pings {
        let distance = (ping.rel_x * ping.rel_x + ping.rel_y * ping.rel_y).sqrt();
        if distance < 0.1 { continue; }
        let relative = ping.rel_y.atan2(ping.rel_x);
        if relative.abs() > raycaster::FOV / 2.0 { continue; }
        let screen_x = width / 2 + (relative / raycaster::FOV * width as f32) as i32;
        let size = (height as f32 / distance).clamp(12.0, 110.0) as i32;
        let alpha = (ping.alpha * 255.0).clamp(0.0, 255.0) as u8;
        let color = if ping.is_mega {
            Color::new(180, 40, 255, alpha)
        } else {
            Color::new(255, 45, 45, alpha)
        };
        let top = height / 2 - size / 2;
        d.draw_rectangle(screen_x - size / 3, top, (size / 3).max(4), size, color);
        d.draw_circle(screen_x, top, (size / 3).max(4) as f32, color);
        d.draw_rectangle_lines(screen_x - size / 3, top, (size / 3).max(4), size, Color::new(255, 220, 220, alpha));
    }
}

fn render_weapon_sprite(
    d: &mut RaylibDrawHandle,
    idle_sprite: Option<&Texture2D>,
    fire_sprite: Option<&Texture2D>,
    weapon: &Weapon,
    width: i32,
    height: i32,
) {
    let sprite = if weapon.is_shooting { fire_sprite.or(idle_sprite) } else { idle_sprite.or(fire_sprite) };
    if let Some(texture) = sprite {
        let scale = 1.0;
        let draw_w = texture.width as f32 * scale;
        let draw_h = texture.height as f32 * scale;
        const WEAPON_RAISE: f32 = 70.0;
        let animation_x = if weapon.is_shooting {
            (weapon.anim_timer * std::f32::consts::TAU).sin() * 12.0
        } else {
            0.0
        };
        let animation_y = if weapon.is_shooting {
            (weapon.anim_timer * std::f32::consts::TAU).cos() * 8.0
        } else {
            0.0
        };
        d.draw_texture_ex(
            texture,
            Vector2::new(
                width as f32 / 2.0 - draw_w / 2.0 + weapon.drag_x + animation_x,
                height as f32 - draw_h - WEAPON_RAISE + weapon.drag_y + animation_y,
            ),
            0.0,
            scale,
            Color::WHITE,
        );
    } else {
        weapon.render(d, width, height as f32);
    }
    weapon.render_crosshair(d, width, height as f32);
}

fn render_hit_marker(d: &mut RaylibDrawHandle, weapon: &Weapon, width: i32, game_height: i32) {
    let center_x = width / 2 + weapon.drag_x as i32;
    let center_y = game_height / 2 + weapon.drag_y as i32;
    let color = Color::new(255, 255, 255, 230);
    d.draw_line(center_x - 10, center_y - 10, center_x - 3, center_y - 3, color);
    d.draw_line(center_x + 10, center_y - 10, center_x + 3, center_y - 3, color);
    d.draw_line(center_x - 10, center_y + 10, center_x - 3, center_y + 3, color);
    d.draw_line(center_x + 10, center_y + 10, center_x + 3, center_y + 3, color);
}

fn distance(x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
    ((x2 - x1).powi(2) + (y2 - y1).powi(2)).sqrt()
}