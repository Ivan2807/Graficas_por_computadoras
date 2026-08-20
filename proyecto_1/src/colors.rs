use raylib::prelude::Color;

pub fn wall_color(id: u8) -> Color {
    match id {
        0 => Color::new(120, 72, 40, 255),
        1 => Color::new(180, 40, 40, 255),
        2 => Color::new(60, 90, 160, 255),
        3 => Color::new(90, 140, 70, 255),
        4 => Color::new(150, 120, 60, 255),
        5 => Color::new(120, 60, 140, 255),
        6 => Color::new(160, 160, 160, 255),
        7 => Color::new(200, 170, 60, 255),
        8 => Color::new(90, 90, 90, 255),
        9 => Color::new(210, 210, 210, 255),
        _ => Color::new(255, 0, 255, 255),
    }
}

pub fn wall_color_dark(id: u8) -> Color {
    darken(wall_color(id))
}

/// Oscurece cualquier color (paredes planas Y pixeles muestreados de una
/// textura), para las caras "sombreadas" del raycaster.
pub fn darken(c: Color) -> Color {
    Color::new(
        (c.r as f32 * 0.6) as u8,
        (c.g as f32 * 0.6) as u8,
        (c.b as f32 * 0.6) as u8,
        255,
    )
}