use crate::primitives::Object;
use crate::ray_intersect::{Intersect, Ray};
use crate::material::Material;

pub struct Scene {
    pub objects: Vec<Object>,
}

impl Scene {
    pub fn new() -> Self {
        Scene { objects: Vec::new() }
    }

    pub fn add(&mut self, obj: Object) -> usize {
        self.objects.push(obj);
        self.objects.len() - 1
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
}