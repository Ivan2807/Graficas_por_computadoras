use raylib::prelude::{Color, Vector3};

#[derive(Clone, Copy)]
pub struct Light {
    pub position: Vector3,
    pub color: Color,
    pub intensity: f32,
    pub row: usize,
    pub enabled: bool,
}

impl Light {
    pub fn new(position: Vector3, color: Color, intensity: f32, row: usize) -> Self {
        Light { position, color, intensity, row, enabled: true }
    }
}