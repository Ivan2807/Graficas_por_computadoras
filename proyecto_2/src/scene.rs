use crate::plane::Plane;
use crate::ray_intersect::{Intersect, Ray};
use raylib::prelude::Vector3;

pub struct Scene {
    pub planes: Vec<Plane>,
}

impl Scene {
    pub fn new() -> Self {
        Scene { planes: Vec::new() }
    }

    pub fn add_plane(&mut self, plane: Plane) -> usize {
        self.planes.push(plane);
        self.planes.len() - 1
    }

    pub fn update_material(&mut self, index: usize, mat: crate::material::Material) -> bool {
        if let Some(p) = self.planes.get_mut(index) {
            p.material = mat;
            true
        } else {
            false
        }
    }

    pub fn closest_hit(&self, ray: &Ray) -> Option<(usize, Intersect)> {
        let mut best: Option<(usize, Intersect)> = None;
        for (i, plane) in self.planes.iter().enumerate() {
            let hit = plane.intersect(ray);
            if hit.hit {
                if best.is_none() || hit.distance < best.as_ref().unwrap().1.distance {
                    best = Some((i, hit));
                }
            }
        }
        best
    }
}