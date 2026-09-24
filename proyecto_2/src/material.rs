use raylib::prelude::{Color, Vector3};

#[derive(Clone, Copy)]
pub struct Material {
    pub albedo: Color,
    pub ambient: f32,
    pub diffuse: f32,
    pub specular: f32,
    pub shininess: f32,
}

impl Material {
    pub fn new(albedo: Color, ambient: f32, diffuse: f32, specular: f32, shininess: f32) -> Self {
        Material { albedo, ambient, diffuse, specular, shininess }
    }

    pub fn diffuse(albedo: Color) -> Self {
        Material { albedo, ambient: 0.15, diffuse: 0.85, specular: 0.0, shininess: 1.0 }
    }

    pub fn shade(&self, normal: Vector3, light_dir: Vector3, view_dir: Vector3) -> Color {
        let n_dot_l = normal.dot(light_dir).max(0.0);
        let diffuse = self.diffuse * n_dot_l;

        let reflect_dir = ((normal * (2.0 * n_dot_l)) - light_dir).normalized();
        let spec_angle = reflect_dir.dot(view_dir).max(0.0);
        let specular = self.specular * spec_angle.powf(self.shininess);

        let intensity = (self.ambient + diffuse + specular).min(1.0);

        Color::new(
            (self.albedo.r as f32 * intensity) as u8,
            (self.albedo.g as f32 * intensity) as u8,
            (self.albedo.b as f32 * intensity) as u8,
            255,
        )
    }
}