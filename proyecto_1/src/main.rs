mod colors;
mod debug2d;
mod framebuffer;
mod level;
mod minimap;
mod player;
mod raycaster;
mod ui;
mod weapon;
mod item;

use framebuffer::Framebuffer;
use level::generate_level;
use minimap::render_minimap;
use player::Player;
use raylib::prelude::*;
use ui::{render_hud, PlayerStats, HUD_HEIGHT};
use weapon::Weapon;
use item::Item;

const SCREEN_WIDTH: i32 = 1280;
const SCREEN_HEIGHT: i32 = 720;

const FB_WIDTH: usize = 320;
const FB_HEIGHT: usize = 180;

const MOVE_SPEED: f32 = 3.0;
const ROT_SPEED: f32 = 2.5;
const MOUSE_SENSITIVITY: f32 = 0.003;

const LEVEL_COLS: usize = 4;
const LEVEL_ROWS: usize = 4;
const LEVEL_MIN_ROOMS: usize = 8;
const LEVEL_FILL_CHANCE: i32 = 70;

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(SCREEN_WIDTH, SCREEN_HEIGHT)
        .title("Roguelike Raycaster")
        .build();

    rl.set_target_fps(60);
    rl.disable_cursor();

    let templates = level::load_room_templates("assets/rooms");

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
    let mut weapon = Weapon::new("Pistola");
    let mut fb = Framebuffer::new(FB_WIDTH, FB_HEIGHT);

    let image = Image::gen_image_color(FB_WIDTH as i32, FB_HEIGHT as i32, Color::BLACK);
    let mut texture = rl
        .load_texture_from_image(&thread, &image)
        .expect("no se pudo crear la textura del framebuffer");

    let mut view_3d = true;

    while !rl.window_should_close() {
        let dt = rl.get_frame_time();

        if rl.is_key_pressed(KeyboardKey::KEY_ZERO) {
            view_3d = !view_3d;
        }

        // --- Disparo ---
        if rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) && !weapon.is_shooting {
            if stats.ammo_in_clip > 0 {
                stats.ammo_in_clip -= 1;
                weapon.is_shooting = true;
            }
        }
        weapon.update(dt);

        // --- Movimiento ---
        let mut move_dir = 0.0f32;
        if rl.is_key_down(KeyboardKey::KEY_W) { move_dir += 1.0; }
        if rl.is_key_down(KeyboardKey::KEY_S) { move_dir -= 1.0; }
        if rl.is_key_down(KeyboardKey::KEY_A) { player.angle -= ROT_SPEED * dt; }
        if rl.is_key_down(KeyboardKey::KEY_D) { player.angle += ROT_SPEED * dt; }

        let mouse_delta = rl.get_mouse_delta();
        player.angle += mouse_delta.x * MOUSE_SENSITIVITY;

        let dx = player.angle.cos() * move_dir * MOVE_SPEED * dt;
        let dy = player.angle.sin() * move_dir * MOVE_SPEED * dt;
        player.try_move(dx, dy, &level);

        level.update_exploration(player.x, player.y);

        if view_3d {
            raycaster::render(&mut fb, &level, &player);
            texture
                .update_texture(&fb.pixels)
                .expect("fallo al actualizar la textura");
        }

        // --- RENDERING POR CAPAS ---
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::BLACK);

        if view_3d {
            let game_view_h = (SCREEN_HEIGHT - HUD_HEIGHT) as f32;

            // CAPA 1: Fondo (Juego 3D)
            d.draw_texture_pro(
                &texture,
                Rectangle::new(0.0, 0.0, FB_WIDTH as f32, FB_HEIGHT as f32),
                Rectangle::new(0.0, 0.0, SCREEN_WIDTH as f32, game_view_h),
                Vector2::new(0.0, 0.0),
                0.0,
                Color::WHITE,
            );

            // CAPA 2: Intermedia (Mira + Arma centreda)
            weapon.render(&mut d, SCREEN_WIDTH, game_view_h);

            // CAPA 3: Overlay / UI (HUD gris abajo y Minimapa arriba)
            render_hud(&mut d, SCREEN_WIDTH, SCREEN_HEIGHT, &stats);
            render_minimap(&mut d, &level, &player, SCREEN_WIDTH);
        } else {
            debug2d::render_2d(&mut d, &level, &player, SCREEN_WIDTH, SCREEN_HEIGHT);
        }

        d.draw_fps(10, 10);
    }
}