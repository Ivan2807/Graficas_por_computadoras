use raylib::prelude::Color;

/// Cada pared "distinta" del mapa tiene un color propio (mas adelante se puede
/// cambiar por una textura real, cargando una Texture2D por id en vez de un Color).
/// El id (1-9) se define directamente en los archivos .txt de las habitaciones.
pub fn wall_color(id: u8) -> Color {
    match id {
        1 => Color::new(180, 40, 40, 255),   // rojo ladrillo
        2 => Color::new(60, 90, 160, 255),   // azul piedra
        3 => Color::new(90, 140, 70, 255),   // verde musgo
        4 => Color::new(150, 120, 60, 255),  // madera
        5 => Color::new(120, 60, 140, 255),  // morado
        6 => Color::new(160, 160, 160, 255), // gris concreto
        7 => Color::new(200, 170, 60, 255),  // dorado
        8 => Color::new(90, 90, 90, 255),    // carbon (default / bordes exteriores)
        9 => Color::new(210, 210, 210, 255), // blanco hueso
        _ => Color::new(255, 0, 255, 255),   // magenta = id invalido (para detectar bugs)
    }
}

/// Version oscurecida, usada en las caras "sombreadas" (paredes horizontales
/// vs verticales) para dar sensacion de volumen, como en Wolfenstein/DOOM.
pub fn wall_color_dark(id: u8) -> Color {
    let c = wall_color(id);
    Color::new(
        (c.r as f32 * 0.6) as u8,
        (c.g as f32 * 0.6) as u8,
        (c.b as f32 * 0.6) as u8,
        255,
    )
}
