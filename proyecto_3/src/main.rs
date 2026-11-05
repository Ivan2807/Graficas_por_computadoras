mod framebuffer;
mod mesh;
mod ship;
mod planets;

use raylib::prelude::*;
use framebuffer::Framebuffer;
use planets::SolarSystem;
use ship::Ship;

const SCREEN_WIDTH: i32 = 1024;
const SCREEN_HEIGHT: i32 = 768;

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(SCREEN_WIDTH, SCREEN_HEIGHT)
        .title("KSP 3D Software Renderer - Skybox Procedural & Culling")
        .build();

    rl.set_target_fps(60);

    let mut fb = Framebuffer::new(SCREEN_WIDTH as usize, SCREEN_HEIGHT as usize);
    let mut solar_system = SolarSystem::new();
    let ship = Ship::load_from_obj();

    let mut cam_pos = (0.0, 0.0, -40.0);
    let mut cam_yaw: f32 = 0.0;
    let mut cam_pitch: f32 = 0.0;

    let mut render_texture = rl.load_render_texture(
        &thread,
        SCREEN_WIDTH as u32,
        SCREEN_HEIGHT as u32
    ).expect("Error al crear la textura de renderizado");

    while !rl.window_should_close() {
        let dt = rl.get_frame_time();

        let move_speed = 45.0 * dt;
        let rot_speed = 1.5 * dt;

        // Controles de vuelo FPS
        if rl.is_key_down(KeyboardKey::KEY_LEFT) { cam_yaw -= rot_speed; }
        if rl.is_key_down(KeyboardKey::KEY_RIGHT) { cam_yaw += rot_speed; }
        if rl.is_key_down(KeyboardKey::KEY_UP) { cam_pitch += rot_speed; }
        if rl.is_key_down(KeyboardKey::KEY_DOWN) { cam_pitch -= rot_speed; }

        cam_pitch = cam_pitch.clamp(-1.5, 1.5);

        let cos_y = cam_yaw.cos();
        let sin_y = cam_yaw.sin();
        let cos_x = cam_pitch.cos();
        let sin_x = cam_pitch.sin();

        let forward_x = sin_y;
        let forward_z = cos_y;
        let right_x = cos_y;
        let right_z = -sin_y;

        if rl.is_key_down(KeyboardKey::KEY_W) {
            cam_pos.0 += forward_x * move_speed;
            cam_pos.2 += forward_z * move_speed;
        }
        if rl.is_key_down(KeyboardKey::KEY_S) {
            cam_pos.0 -= forward_x * move_speed;
            cam_pos.2 -= forward_z * move_speed;
        }
        if rl.is_key_down(KeyboardKey::KEY_A) {
            cam_pos.0 -= right_x * move_speed;
            cam_pos.2 -= right_z * move_speed;
        }
        if rl.is_key_down(KeyboardKey::KEY_D) {
            cam_pos.0 += right_x * move_speed;
            cam_pos.2 += right_z * move_speed;
        }
        if rl.is_key_down(KeyboardKey::KEY_Q) { cam_pos.1 += move_speed; }
        if rl.is_key_down(KeyboardKey::KEY_E) { cam_pos.1 -= move_speed; }

        solar_system.update(dt);

        // --- PIPELINE DE RENDERIZADO ---
        fb.clear();

        // 1. Renderizar Sistema Solar y Esfera Celeste Procedural (Estrellas y Nebulosas)
        let active_planet = solar_system.render(
            &mut fb, 
            cam_pos, 
            cam_yaw, 
            cam_pitch, 
            cos_y, 
            sin_y, 
            cos_x, 
            sin_x
        );

        // 2. Renderizar nave fija en pantalla con iluminación solar proyectada
        ship.render(&mut fb, cam_pos, cam_yaw, cam_pitch);

        // --- TRANSFERIR FRAMEBUFFER A RAYLIB ---
        let mut d = rl.begin_drawing(&thread);
        {
            let mut texture_drawer = d.begin_texture_mode(&thread, &mut render_texture);
            texture_drawer.clear_background(Color::BLACK);

            for y in 0..SCREEN_HEIGHT {
                for x in 0..SCREEN_WIDTH {
                    let idx = (y as usize) * (SCREEN_WIDTH as usize) + (x as usize);
                    let p = fb.color_buffer[idx];
                    texture_drawer.draw_pixel(x, y, Color::new(p.r, p.g, p.b, p.a));
                }
            }
        }

        d.clear_background(Color::BLACK);
        d.draw_texture_rec(
            &render_texture,
            Rectangle::new(0.0, 0.0, SCREEN_WIDTH as f32, -(SCREEN_HEIGHT as f32)),
            Vector2::new(0.0, 0.0),
            Color::WHITE,
        );

        // HUD / Interfaz
        d.draw_text("KSP 3D ENGINE - SKYBOX PROCEDURAL", 20, 20, 18, Color::GREEN);
        d.draw_text("Controles: W, A, S, D (Mover) | Flechas (Girar cámara)", 20, 45, 14, Color::LIGHTGRAY);
        
        if let Some(name) = active_planet {
            d.draw_text(&format!("OBJETIVO AISLADO EN RANGO: {}", name), 20, 70, 16, Color::YELLOW);
        } else {
            d.draw_text("SISTEMA ESTELAR KERBOL - NAVEGACIÓN LIBRE", 20, 70, 14, Color::SKYBLUE);
        }
    }
}