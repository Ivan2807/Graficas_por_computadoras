use crate::primitives::Object;
use crate::ray_intersect::{Intersect, Ray};
use crate::material::Material;
use crate::light::Light;
use raylib::prelude::{Color, Vector3};

pub struct Scene {
    pub objects: Vec<Object>,
    pub lights: Vec<Light>,
    pub light_row_enabled: Vec<bool>,
}

impl Scene {
    pub fn new(num_rows: usize) -> Self {
        Scene {
            objects: Vec::new(),
            lights: Vec::new(),
            light_row_enabled: vec![true; num_rows],
        }
    }

    pub fn add(&mut self, obj: Object) -> usize {
        self.objects.push(obj);
        self.objects.len() - 1
    }

    pub fn add_light(&mut self, light: Light) {
        self.lights.push(light);
    }

    pub fn update_material(&mut self, index: usize, mat: Material) -> bool {
        if let Some(obj) = self.objects.get_mut(index) {
            obj.set_material(mat);
            true
        } else {
            false
        }
    }

    pub fn closest_hit(&self, ray: &Ray) -> Option<(usize, Intersect)> {
        let mut best: Option<(usize, Intersect)> = None;
        for (i, obj) in self.objects.iter().enumerate() {
            let hit = obj.intersect(ray);
            if hit.hit {
                if best.is_none() || hit.distance < best.as_ref().unwrap().1.distance {
                    best = Some((i, hit));
                }
            }
        }
        best
    }

    pub fn toggle_row(&mut self, row: usize) {
        if row < self.light_row_enabled.len() {
            let new_state = !self.light_row_enabled[row];
            self.light_row_enabled[row] = new_state;
            for light in self.lights.iter_mut() {
                if light.row == row {
                    light.enabled = new_state;
                }
            }
        }
    }

    pub fn toggle_all(&mut self) {
        let all_on = self.light_row_enabled.iter().all(|&b| b);
        let new_state = !all_on;
        for row in self.light_row_enabled.iter_mut() {
            *row = new_state;
        }
        for light in self.lights.iter_mut() {
            light.enabled = new_state;
        }
    }

    pub fn shade(
        &self,
        mat: &Material,
        hit: &Intersect,
        view_dir: Vector3,
        sun_dir: Vector3,
        sun_color: Color,
        sun_intensity: f32,
        ambient_scale: f32,
    ) -> Color {
        let amb = mat.ambient * ambient_scale;
        let mut r = mat.albedo.r as f32 * amb;
        let mut g = mat.albedo.g as f32 * amb;
        let mut b = mat.albedo.b as f32 * amb;

        // Emisivo: no le afecta la iluminación
        if mat.emissive > 0.0 {
            r += mat.albedo.r as f32 * mat.emissive;
            g += mat.albedo.g as f32 * mat.emissive;
            b += mat.albedo.b as f32 * mat.emissive;
        }

        // Sol (direccional)
        if sun_intensity > 0.001 {
            let n_dot_l = hit.normal.dot(sun_dir).max(0.0);
            if n_dot_l > 0.0 {
                let diff = mat.diffuse * n_dot_l * sun_intensity;
                let refl = (hit.normal * (2.0 * n_dot_l) - sun_dir).normalized();
                let spec = mat.specular
                    * refl.dot(view_dir).max(0.0).powf(mat.shininess)
                    * sun_intensity;
                let total = diff + spec;
                r += mat.albedo.r as f32 * total * (sun_color.r as f32 / 255.0);
                g += mat.albedo.g as f32 * total * (sun_color.g as f32 / 255.0);
                b += mat.albedo.b as f32 * total * (sun_color.b as f32 / 255.0);
            }
        }

        // Luces puntuales del techo
        for light in &self.lights {
            if !light.enabled { continue; }

            let to_light = light.position - hit.point;
            let dist_sq = to_light.dot(to_light);
            if dist_sq < 0.01 { continue; }
            let dist = dist_sq.sqrt();
            let l = to_light / dist;
            let n_dot_l = hit.normal.dot(l).max(0.0);
            if n_dot_l <= 0.0 { continue; }

            // Atenuación suave (evita que explote cerca de la luz)
            let att = light.intensity / (1.0 + 0.12 * dist_sq);

            let diff = mat.diffuse * n_dot_l * att;
            let refl = (hit.normal * (2.0 * n_dot_l) - l).normalized();
            let spec = mat.specular
                * refl.dot(view_dir).max(0.0).powf(mat.shininess)
                * att;
            let total = diff + spec;

            r += mat.albedo.r as f32 * total * (light.color.r as f32 / 255.0);
            g += mat.albedo.g as f32 * total * (light.color.g as f32 / 255.0);
            b += mat.albedo.b as f32 * total * (light.color.b as f32 / 255.0);
        }

        Color::new(
            r.min(255.0) as u8,
            g.min(255.0) as u8,
            b.min(255.0) as u8,
            255,
        )
    }
}