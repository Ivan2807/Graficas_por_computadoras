pub mod plane;
pub mod cube;
pub mod sphere;

use crate::ray_intersect::{Intersect, Ray};
use crate::material::Material;

pub use plane::Plane;
pub use cube::Cube;
pub use sphere::Sphere;

pub enum Object {
    Plane(Plane),
    Cube(Cube),
    Sphere(Sphere),
}

impl Object {
    pub fn intersect(&self, ray: &Ray) -> Intersect {
        match self {
            Object::Plane(p)  => p.intersect(ray),
            Object::Cube(c)   => c.intersect(ray),
            Object::Sphere(s) => s.intersect(ray),
        }
    }

    pub fn material(&self) -> Material {
        match self {
            Object::Plane(p)  => p.material,
            Object::Cube(c)   => c.material,
            Object::Sphere(s) => s.material,
        }
    }

    pub fn set_material(&mut self, mat: Material) {
        match self {
            Object::Plane(p)  => p.material = mat,
            Object::Cube(c)   => c.material = mat,
            Object::Sphere(s) => s.material = mat,
        }
    }
}