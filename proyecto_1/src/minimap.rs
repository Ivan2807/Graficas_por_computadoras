use crate::colors::wall_color;
use crate::level::{Level, Tile};
use crate::player::Player;
use raylib::prelude::*;

const TILE_PX: f32 = 4.0; // Tamaño de cada celda en el minimapa
const MARGIN: i32 = 15;   // Distancia desde la esquina de la pantalla

pub fn render_minimap(
    d: &mut RaylibDrawHandle,
    level: &Level,
    player: &Player,
    screen_w: i32,
) {
    let map_px_w = level.width as f32 * TILE_PX;
    let map_px_h = level.height as f32 * TILE_PX;

    // Posición en la esquina superior derecha
    let ox = screen_w as f32 - map_px_w - MARGIN as f32;
    let oy = MARGIN as f32;

    // Fondo del minimapa
    d.draw_rectangle(
        ox as i32 - 4,
        oy as i32 - 4,
        map_px_w as i32 + 8,
        map_px_h as i32 + 8,
        Color::new(10, 10, 15, 220),
    );
    d.draw_rectangle_lines(
        ox as i32 - 4,
        oy as i32 - 4,
        map_px_w as i32 + 8,
        map_px_h as i32 + 8,
        Color::new(200, 200, 200, 255),
    );

    // Dibujar únicamente los tiles de salas o puertas ya exploradas
    for y in 0..level.height {
        for x in 0..level.width {
            if !level.is_tile_explored(x, y) {
                continue;
            }

            let color = match level.get(x as i32, y as i32) {
                Tile::Wall(id) => wall_color(id),
                Tile::Door => Color::new(230, 220, 120, 255),
                Tile::LockedDoor => Color::new(120, 72, 40, 255),
                Tile::Empty => Color::new(50, 50, 60, 255),
            };
            d.draw_rectangle(
                (ox + x as f32 * TILE_PX) as i32,
                (oy + y as f32 * TILE_PX) as i32,
                TILE_PX as i32,
                TILE_PX as i32,
                color,
            );
        }
    }

    // Dibujar la posición del jugador
    let px = ox + player.x * TILE_PX;
    let py = oy + player.y * TILE_PX;

    d.draw_circle(px as i32, py as i32, 3.0, Color::RED);

    // Línea de orientación de la mirada
    let dir_len = 7.0;
    d.draw_line(
        px as i32,
        py as i32,
        (px + player.angle.cos() * dir_len) as i32,
        (py + player.angle.sin() * dir_len) as i32,
        Color::YELLOW,
    );
}