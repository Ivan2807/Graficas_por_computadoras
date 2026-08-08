use crate::colors::wall_color;
use crate::level::{Level, Tile};
use crate::player::Player;
use raylib::prelude::*;

const TILE_PX: f32 = 14.0;

/// Vista 2D de arriba hacia abajo de TODO el nivel generado: sirve para ver
/// como quedaron armados los cuadrantes, cuales tienen sala y donde se
/// abrieron las puertas entre salas vecinas. Se activa/desactiva con la
/// tecla 0 (ver main.rs). Dibuja directo con raylib, no usa el framebuffer
/// del raycaster (es una vista de depuracion, no parte del juego 3D).
pub fn render_2d(
    d: &mut RaylibDrawHandle,
    level: &Level,
    player: &Player,
    screen_w: i32,
    screen_h: i32,
) {
    let map_px_w = level.width as f32 * TILE_PX;
    let map_px_h = level.height as f32 * TILE_PX;
    let ox = (screen_w as f32 - map_px_w) / 2.0;
    let oy = (screen_h as f32 - map_px_h) / 2.0;

    // tiles
    for y in 0..level.height {
        for x in 0..level.width {
            let color = match level.get(x as i32, y as i32) {
                Tile::Wall(id) => wall_color(id),
                Tile::Door => Color::new(230, 220, 120, 255), // amarillo = puerta
                Tile::Empty => Color::new(35, 35, 40, 255),   // piso
            };
            d.draw_rectangle(
                (ox + x as f32 * TILE_PX) as i32,
                (oy + y as f32 * TILE_PX) as i32,
                TILE_PX as i32 - 1,
                TILE_PX as i32 - 1,
                color,
            );
        }
    }

    // lineas guia mostrando la cuadricula de cuadrantes (4x4, etc.)
    let grid_line_color = Color::new(255, 255, 255, 70);
    for cy in 0..=level.rows {
        let py = oy + cy as f32 * level.cell_h as f32 * TILE_PX;
        d.draw_line(
            ox as i32,
            py as i32,
            (ox + map_px_w) as i32,
            py as i32,
            grid_line_color,
        );
    }
    for cx in 0..=level.cols {
        let px = ox + cx as f32 * level.cell_w as f32 * TILE_PX;
        d.draw_line(
            px as i32,
            oy as i32,
            px as i32,
            (oy + map_px_h) as i32,
            grid_line_color,
        );
    }

    // jugador: punto rojo + linea indicando hacia donde mira
    let px = ox + player.x * TILE_PX;
    let py = oy + player.y * TILE_PX;
    d.draw_circle(px as i32, py as i32, 5.0, Color::RED);
    let dir_len = 16.0;
    d.draw_line(
        px as i32,
        py as i32,
        (px + player.angle.cos() * dir_len) as i32,
        (py + player.angle.sin() * dir_len) as i32,
        Color::RED,
    );

    d.draw_text(
        &format!("Salas generadas: {}", level.room_cells.len()),
        10,
        40,
        20,
        Color::WHITE,
    );
    d.draw_text("Modo 2D (presiona 0 para volver a 3D)", 10, 65, 18, Color::WHITE);
}
