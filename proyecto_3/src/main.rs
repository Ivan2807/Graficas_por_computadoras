mod framebuffer;
mod mesh;
mod ship;
mod planets;

use raylib::prelude::*;
use framebuffer::Framebuffer;
use raylib::core::texture::RaylibTexture2D;
use planets::SolarSystem;
use ship::Ship;

const SCREEN_WIDTH: i32 = 1024;
const SCREEN_HEIGHT: i32 = 768;

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(SCREEN_WIDTH, SCREEN_HEIGHT)
        .title("KSP 3D Engine - Escala Masiva & Inercia Espacial")
        .build();

    rl.set_target_fps(60);

    let mut fb = Framebuffer::new(SCREEN_WIDTH as usize, SCREEN_HEIGHT as usize);
    let mut solar_system = SolarSystem::new();
    let ship = Ship::load_from_obj();

    let mut cam_pos = (0.0, 0.0, -40.0);
    let mut cam_yaw: f32 = 0.0;
    let mut cam_pitch: f32 = 0.0;

    let mut velocity = (0.0, 0.0, 0.0);
    let acceleration_power = 65.0;

    let mut real_scale_mode = false;

    let mut render_texture = rl.load_render_texture(
        &thread,
        SCREEN_WIDTH as u32,
        SCREEN_HEIGHT as u32,
    ).expect("Error al crear la textura de renderizado");
    let mut rgba_buffer = Vec::with_capacity((SCREEN_WIDTH * SCREEN_HEIGHT * 4) as usize);

    while !rl.window_should_close() {
        let dt = rl.get_frame_time();
        let rot_speed = 1.5 * dt;

        if rl.is_key_pressed(KeyboardKey::KEY_ONE) { real_scale_mode = false; }
        if rl.is_key_pressed(KeyboardKey::KEY_TWO) { real_scale_mode = true; }

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

        let mut accel_x = 0.0;
        let mut accel_y = 0.0;
        let mut accel_z = 0.0;

        if rl.is_key_down(KeyboardKey::KEY_W) {
            accel_x += forward_x;
            accel_z += forward_z;
        }
        if rl.is_key_down(KeyboardKey::KEY_S) {
            accel_x -= forward_x;
            accel_z -= forward_z;
        }
        if rl.is_key_down(KeyboardKey::KEY_A) {
            accel_x -= right_x;
            accel_z -= right_z;
        }
        if rl.is_key_down(KeyboardKey::KEY_D) {
            accel_x += right_x;
            accel_z += right_z;
        }
        if rl.is_key_down(KeyboardKey::KEY_Q) { accel_y += 1.0; }
        if rl.is_key_down(KeyboardKey::KEY_E) { accel_y -= 1.0; }

        let mut current_accel_power = acceleration_power;
        if rl.is_key_down(KeyboardKey::KEY_LEFT_SHIFT) || rl.is_key_down(KeyboardKey::KEY_RIGHT_SHIFT) {
            let turbo_multiplier = if real_scale_mode { 8.0 } else { 3.0 };
            current_accel_power *= turbo_multiplier;
        }

        velocity.0 += accel_x * current_accel_power * dt;
        velocity.1 += accel_y * current_accel_power * dt;
        velocity.2 += accel_z * current_accel_power * dt;

        // Fricción baja (0.3) en Modo 2 para sentir el deslizamiento espacial inercial
        let current_friction = if real_scale_mode { 0.3 } else { 2.0 };

        velocity.0 -= velocity.0 * current_friction * dt;
        velocity.1 -= velocity.1 * current_friction * dt;
        velocity.2 -= velocity.2 * current_friction * dt;

        let mut next_pos = (
            cam_pos.0 + velocity.0 * dt,
            cam_pos.1 + velocity.1 * dt,
            cam_pos.2 + velocity.2 * dt,
        );

        // Sincronizado exactamente con el nuevo sol colosal y distancias del Modo 2
        let (dist_mult, rad_mult, sun_radius, sun_z) = if real_scale_mode {
            (14.0, 3.0, 120.0, 2200.0)
        } else {
            (1.0, 1.0, 30.0, 300.0)
        };

        let sun_pos = (0.0, 0.0, sun_z);

        // --- COLISIÓN CON EL SOL ---
        let dist_sun = ((next_pos.0 - sun_pos.0).powi(2) + (next_pos.1 - sun_pos.1).powi(2) + (next_pos.2 - sun_pos.2).powi(2)).sqrt();
        if dist_sun < (sun_radius + 15.0) {
            velocity.0 = 0.0;
            velocity.1 = 0.0;
            velocity.2 = 0.0;
            next_pos = cam_pos;
        }

        // --- COLISIÓN CON LOS PLANETAS ---
        for p in &solar_system.planets {
            let actual_dist = p.distance * dist_mult;
            let actual_rad = p.radius * rad_mult;

            let px = actual_dist * p.angle.cos();
            let pz = sun_pos.2 + actual_dist * p.angle.sin();
            let dist_planet = ((next_pos.0 - px).powi(2) + (next_pos.1 - 0.0).powi(2) + (next_pos.2 - pz).powi(2)).sqrt();
            
            if dist_planet < actual_rad + 5.0 {
                velocity.0 = 0.0;
                velocity.1 = 0.0;
                velocity.2 = 0.0;
                next_pos = cam_pos;
                break;
            }
        }

        cam_pos = next_pos;
        
        solar_system.update(dt, real_scale_mode);

        // --- RENDER ---
        fb.clear();

        let active_planet = solar_system.render(
            &mut fb, 
            cam_pos, 
            cam_yaw, 
            cam_pitch, 
            cos_y, 
            sin_y, 
            cos_x, 
            sin_x,
            real_scale_mode
        );

        let ship_scale = if real_scale_mode { 0.62 } else { 1.0 };
        ship.render(&mut fb, cam_pos, cam_yaw, cam_pitch, ship_scale);

        let mut d = rl.begin_drawing(&thread);
        rgba_buffer.clear();
        for p in &fb.color_buffer {
            rgba_buffer.extend_from_slice(&[p.r, p.g, p.b, p.a]);
        }
        render_texture.update_texture(&rgba_buffer).expect("Error al actualizar la textura de pantalla");

        d.clear_background(Color::BLACK);
        d.draw_texture_rec(
            &render_texture,
            Rectangle::new(0.0, 0.0, SCREEN_WIDTH as f32, -(SCREEN_HEIGHT as f32)),
            Vector2::new(0.0, 0.0),
            Color::WHITE,
        );

        // HUD
        d.draw_text("KSP 3D ENGINE - ESCALA MASIVA & INERCIA ESPACIAL", 20, 20, 18, Color::GREEN);
        d.draw_text("Presiona 1: Modo Arcade | Presiona 2: Espacio Profundo Masivo", 20, 45, 14, Color::LIGHTGRAY);
        
        if real_scale_mode {
            d.draw_text("MODO ACTIVO: 2 - ESPACIO PROFUNDO ", 20, 70, 15, Color::ORANGE);
            let speed = (velocity.0 * velocity.0 + velocity.1 * velocity.1 + velocity.2 * velocity.2).sqrt();
            let sun_distance = (cam_pos.0 * cam_pos.0 + cam_pos.1 * cam_pos.1 + (sun_z - cam_pos.2).powi(2)).sqrt();
            d.draw_text(&format!("VELOCIDAD: {:.1} u/s   DISTANCIA AL SOL: {:.0} u", speed, sun_distance), 20, 95, 14, Color::LIGHTGRAY);
        } else {
            d.draw_text("MODO ACTIVO: 1 - MODO ARCADE", 20, 70, 14, Color::SKYBLUE);
        }

        if let Some(name) = active_planet {
            let target_y = if real_scale_mode { 120 } else { 95 };
            d.draw_text(&format!("OBJETIVO AISLADO EN RANGO: {}", name), 20, target_y, 16, Color::YELLOW);
        }
    }
}
