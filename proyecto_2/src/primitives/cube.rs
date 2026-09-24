use crate::ray_intersect::{Intersect, Ray};
use crate::material::Material;
use raylib::prelude::Vector3;

#[derive(Clone, Copy)]
pub struct Cube {
    pub center: Vector3,
    pub size: Vector3, // ancho, alto, profundidad
    pub material: Material,
}

impl Cube {
    pub fn new(center: Vector3, size: Vector3, material: Material) -> Self {
        Cube { center, size, material }
    }

    pub fn intersect(&self, ray: &Ray) -> Intersect {
        let half = self.size * 0.5;
        let min = self.center - half;
        let max = self.center + half;

        let inv = Vector3::new(
            if ray.direction.x.abs() > 1e-9 { 1.0 / ray.direction.x } else { f32::INFINITY },
            if ray.direction.y.abs() > 1e-9 { 1.0 / ray.direction.y } else { f32::INFINITY },
            if ray.direction.z.abs() > 1e-9 { 1.0 / ray.direction.z } else { f32::INFINITY },
        );

        let t1 = (min.x - ray.origin.x) * inv.x;
        let t2 = (max.x - ray.origin.x) * inv.x;
        let t3 = (min.y - ray.origin.y) * inv.y;
        let t4 = (max.y - ray.origin.y) * inv.y;
        let t5 = (min.z - ray.origin.z) * inv.z;
        let t6 = (max.z - ray.origin.z) * inv.z;

        let tmin = f32::max(f32::max(f32::min(t1, t2), f32::min(t3, t4)), f32::min(t5, t6));
        let tmax = f32::min(f32::min(f32::max(t1, t2), f32::max(t3, t4)), f32::max(t5, t6));

        if tmax < 0.0 || tmin > tmax {
            return Intersect::none();
        }

        let t = if tmin > 1e-3 { tmin } else { tmax };
        if t <= 1e-3 {
            return Intersect::none();
        }

        let point = ray.at(t);
        let normal = compute_normal(&point, &self.center, &half);

        Intersect { hit: true, distance: t, point, normal }
    }
}

fn compute_normal(p: &Vector3, c: &Vector3, half: &Vector3) -> Vector3 {
    let local = *p - *c;
    let dx = half.x - local.x.abs();
    let dy = half.y - local.y.abs();
    let dz = half.z - local.z.abs();

    if dx <= dy && dx <= dz {
        Vector3::new(local.x.signum(), 0.0, 0.0)
    } else if dy <= dz {
        Vector3::new(0.0, local.y.signum(), 0.0)
    } else {
        Vector3::new(0.0, 0.0, local.z.signum())
    }
}