use raylib::prelude::Vector3;

pub struct Ray {
    pub origin: Vector3,
    pub direction: Vector3,
}

impl Ray {
    pub fn new(origin: Vector3, direction: Vector3) -> Self {
        Ray { origin, direction: direction.normalized() }
    }

    pub fn at(&self, t: f32) -> Vector3 {
        self.origin + self.direction * t
    }
}

#[derive(Clone, Copy)]
pub struct Intersect {
    pub hit: bool,
    pub distance: f32,
    pub point: Vector3,
    pub normal: Vector3,
}

impl Intersect {
    pub fn none() -> Self {
        Intersect {
            hit: false,
            distance: f32::INFINITY,
            point: Vector3::zero(),
            normal: Vector3::zero(),
        }
    }
}