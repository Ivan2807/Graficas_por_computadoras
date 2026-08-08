use crate::colors::{wall_color, wall_color_dark};
use crate::framebuffer::Framebuffer;
use crate::level::{Level, Tile};
use crate::player::Player;
use raylib::prelude::Color;

pub const FOV: f32 = std::f32::consts::PI / 3.0; // 60 grados

/// Dibuja una escena 3D completa (techo, piso y paredes) sobre `fb`,
/// usando raycasting por columnas. No toca raylib para nada: solo escribe
/// pixeles en el framebuffer propio.
pub fn render(fb: &mut Framebuffer, level: &Level, player: &Player) {
    let half = fb.height / 2;
    for y in 0..half {
        for x in 0..fb.width {
            fb.set_pixel(x, y, Color::new(50, 50, 70, 255)); // techo
        }
    }
    for y in half..fb.height {
        for x in 0..fb.width {
            fb.set_pixel(x, y, Color::new(70, 60, 50, 255)); // piso
        }
    }

    for col in 0..fb.width {
        let camera_x = col as f32 / fb.width as f32 - 0.5; // -0.5 .. 0.5
        let ray_angle = player.angle + camera_x * FOV;

        let (dist, wall_id, side) = cast_ray(level, player.x, player.y, ray_angle);

        // correccion de "ojo de pez" (fisheye)
        let corrected = (dist * (ray_angle - player.angle).cos()).max(0.0001);

        let wall_height = (fb.height as f32 / corrected).min(fb.height as f32 * 4.0);
        let draw_start = ((fb.height as f32 - wall_height) / 2.0).max(0.0) as usize;
        let draw_end = ((fb.height as f32 + wall_height) / 2.0).min(fb.height as f32) as usize;

        let color = if side == 1 {
            wall_color_dark(wall_id)
        } else {
            wall_color(wall_id)
        };

        for y in draw_start..draw_end {
            fb.set_pixel(col, y, color);
        }
    }
}

/// Algoritmo DDA (Digital Differential Analysis): lanza un rayo desde
/// (px, py) en direccion `angle` y avanza celda por celda hasta chocar con
/// una pared. Devuelve (distancia, id_de_pared, lado_golpeado).
/// lado: 0 = golpeo una pared "vertical" (variacion en X), 1 = "horizontal".
fn cast_ray(level: &Level, px: f32, py: f32, angle: f32) -> (f32, u8, u8) {
    let dir_x = angle.cos();
    let dir_y = angle.sin();

    let mut map_x = px.floor() as i32;
    let mut map_y = py.floor() as i32;

    let delta_dist_x = if dir_x == 0.0 { 1e30 } else { (1.0 / dir_x).abs() };
    let delta_dist_y = if dir_y == 0.0 { 1e30 } else { (1.0 / dir_y).abs() };

    let (step_x, mut side_dist_x) = if dir_x < 0.0 {
        (-1, (px - map_x as f32) * delta_dist_x)
    } else {
        (1, (map_x as f32 + 1.0 - px) * delta_dist_x)
    };
    let (step_y, mut side_dist_y) = if dir_y < 0.0 {
        (-1, (py - map_y as f32) * delta_dist_y)
    } else {
        (1, (map_y as f32 + 1.0 - py) * delta_dist_y)
    };

    let mut side = 0u8;
    let mut wall_id: u8 = 8;

    for _ in 0..2000 {
        // limite de pasos para evitar loops infinitos si algo sale mal
        if side_dist_x < side_dist_y {
            side_dist_x += delta_dist_x;
            map_x += step_x;
            side = 0;
        } else {
            side_dist_y += delta_dist_y;
            map_y += step_y;
            side = 1;
        }

        if let Tile::Wall(id) = level.get(map_x, map_y) {
            wall_id = id;
            break;
        }
    }

    let dist = if side == 0 {
        side_dist_x - delta_dist_x
    } else {
        side_dist_y - delta_dist_y
    };

    (dist.max(0.0001), wall_id, side)
}
