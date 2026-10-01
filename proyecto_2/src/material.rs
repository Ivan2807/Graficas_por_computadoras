use raylib::prelude::{Color, Vector3};

#[derive(Clone, Copy)]
pub struct Material {
    pub albedo: Color,
    pub ambient: f32,
    pub diffuse: f32,
    pub specular: f32,
    pub shininess: f32,
    pub emissive: f32, // 0 = normal, 1 = siempre brillante
}

impl Material {
    pub fn new(albedo: Color, ambient: f32, diffuse: f32, specular: f32, shininess: f32) -> Self {
        Material { albedo, ambient, diffuse, specular, shininess, emissive: 0.0 }
    }

    pub fn diffuse(albedo: Color) -> Self {
        Material { albedo, ambient: 0.15, diffuse: 0.85, specular: 0.0, shininess: 1.0, emissive: 0.0 }
    }

    pub fn emissive(albedo: Color) -> Self {
        Material { albedo, ambient: 0.0, diffuse: 0.0, specular: 0.0, shininess: 1.0, emissive: 1.0 }
    }

    /// Placeholder para que compile el shade previo (ya no se usa)
    pub fn shade(&self, _n: Vector3, _l: Vector3, _v: Vector3) -> Color {
        self.albedo
    }
}