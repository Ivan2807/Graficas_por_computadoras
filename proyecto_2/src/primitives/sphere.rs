use crate::ray_intersect::{Intersect, Ray};
use crate::material::Material;
use raylib::prelude::Vector3;

#[derive(Clone, Copy)]
pub struct Sphere {
    pub center: Vector3,
    pub radius: f32,
    pub material: Material,
}

impl Sphere {
    pub fn new(center: Vector3, radius: f32, material: Material) -> Self {
        Sphere { center, radius, material }
    }

    pub fn intersect(&self, ray: &Ray) -> Intersect {
        let oc = ray.origin - self.center;
        let a = ray.direction.dot(ray.direction);
        let b = 2.0 * oc.dot(ray.direction);
        let c = oc.dot(oc) - self.radius * self.radius;

        let disc: f32 = b * b - 4.0 * a * c;
        if disc < 0.0 {
            return Intersect::none();
        }

        let sqrt_d = disc.sqrt();
        let t0 = (-b - sqrt_d) / (2.0 * a);
        let t1 = (-b + sqrt_d) / (2.0 * a);

        let t = if t0 > 1e-3 { t0 } else if t1 > 1e-3 { t1 } else { return Intersect::none(); };

        let point = ray.at(t);
        let normal = (point - self.center).normalized();

        Intersect { hit: true, distance: t, point, normal }
    }
}