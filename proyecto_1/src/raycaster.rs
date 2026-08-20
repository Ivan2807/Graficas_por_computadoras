use crate::colors::{self, darken};
use crate::framebuffer::Framebuffer;
use crate::level::{Level, Tile};
use crate::player::Player;
use crate::textures::Textures;
use raylib::prelude::Color;

pub const FOV: f32 = std::f32::consts::PI / 3.0;

pub struct RayHit {
    pub distance: f32,
    pub wall_id: u8,
    pub side: u8,
    pub tile_x: i32,
    pub tile_y: i32,
    pub is_locked_door: bool,
    pub wall_u: f32,
}

pub fn render(fb: &mut Framebuffer, level: &Level, player: &Player, textures: &Textures) {
    let half = fb.height / 2;
    for y in 0..half {
        for x in 0..fb.width {
            fb.set_pixel(x, y, Color::new(50, 50, 70, 255));
        }
    }
    for y in half..fb.height {
        for x in 0..fb.width {
            fb.set_pixel(x, y, Color::new(70, 60, 50, 255));
        }
    }

    for col in 0..fb.width {
        let camera_x = col as f32 / fb.width as f32 - 0.5;
        let ray_angle = player.angle + camera_x * FOV;

        let hit = cast_ray(level, player.x, player.y, ray_angle);
        let corrected = (hit.distance * (ray_angle - player.angle).cos()).max(0.0001);

        let wall_height = (fb.height as f32 / corrected).min(fb.height as f32 * 4.0);
        let draw_start = ((fb.height as f32 - wall_height) / 2.0).max(0.0) as usize;
        let draw_end = ((fb.height as f32 + wall_height) / 2.0).min(fb.height as f32) as usize;

        if hit.is_locked_door {
            // Puertas cerradas: planas (cafe), no texturizadas, y se
            // "hunden" segun su progreso de apertura.
            let progress = level.door_progress_at(hit.tile_x as usize, hit.tile_y as usize);
            let remaining = (1.0 - progress).clamp(0.0, 1.0);
            let clipped_end = draw_start + (((draw_end - draw_start) as f32) * remaining) as usize;
            let color = if hit.side == 1 { darken(colors::wall_color(0)) } else { colors::wall_color(0) };
            for y in draw_start..clipped_end {
                fb.set_pixel(col, y, color);
            }
            continue;
        }

        let tex = textures.for_wall_id(hit.wall_id);
        let span = (draw_end.saturating_sub(draw_start)).max(1) as f32;
        for y in draw_start..draw_end {
            let v = (y - draw_start) as f32 / span;
            let mut color = tex.sample(hit.wall_u, v);
            if hit.side == 1 {
                color = darken(color);
            }
            fb.set_pixel(col, y, color);
        }
    }
}

pub fn cast_ray(level: &Level, px: f32, py: f32, angle: f32) -> RayHit {
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
    let mut is_locked_door = false;

    for _ in 0..2000 {
        if side_dist_x < side_dist_y {
            side_dist_x += delta_dist_x;
            map_x += step_x;
            side = 0;
        } else {
            side_dist_y += delta_dist_y;
            map_y += step_y;
            side = 1;
        }

        match level.get(map_x, map_y) {
            Tile::Wall(id) => {
                wall_id = id;
                break;
            }
            Tile::LockedDoor => {
                wall_id = 0;
                is_locked_door = true;
                break;
            }
            _ => continue,
        }
    }

    let dist = if side == 0 {
        side_dist_x - delta_dist_x
    } else {
        side_dist_y - delta_dist_y
    };
    let dist = dist.max(0.0001);

    let hit_x = px + dir_x * dist;
    let hit_y = py + dir_y * dist;
    let wall_u = if side == 0 {
        hit_y - hit_y.floor()
    } else {
        hit_x - hit_x.floor()
    };

    RayHit {
        distance: dist,
        wall_id,
        side,
        tile_x: map_x,
        tile_y: map_y,
        is_locked_door,
        wall_u,
    }
}