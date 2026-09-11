use crate::ray_intersect::{Intersect, Ray};
use crate::material::Material;
use raylib::prelude::Vector3;

#[derive(Clone, Copy)]
pub struct Plane {
    pub point: Vector3,   // un punto cualquiera del plano
    pub normal: Vector3,  // normal 
    pub material: Material,
}

impl Plane {
    pub fn new(point: Vector3, normal: Vector3, material: Material) -> Self {
        Plane { point, normal: normal.normalized(), material }
    }

    pub fn intersect(&self, ray: &Ray) -> Intersect {
        let denom = self.normal.dot(ray.direction);
        if denom.abs() < 1e-6 {
            return Intersect::none(); // rayo paralelo al plano
        }
        let t = (self.point - ray.origin).dot(self.normal) / denom;
        if t <= 1e-3 {
            return Intersect::none();
        }
        Intersect {
            hit: true,
            distance: t,
            point: ray.at(t),
            normal: self.normal,
        }
    }
}