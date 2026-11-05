pub struct RayHit {
    pub hit: bool,
    pub distance: f32,
    pub normal: (f32, f32, f32),
}

// Raytracing optimizado mediante intersección analítica rayo-esfera para los planetas
pub fn intersect_sphere(ray_origin: (f32, f32, f32), ray_dir: (f32, f32, f32), sphere_center: (f32, f32, f32), radius: f32) -> RayHit {
    let oc = (
        ray_origin.0 - sphere_center.0,
        ray_origin.1 - sphere_center.1,
        ray_origin.2 - sphere_center.2,
    );
    let b = oc.0 * ray_dir.0 + oc.1 * ray_dir.1 + oc.2 * ray_dir.2;
    let c = (oc.0 * oc.0 + oc.1 * oc.1 + oc.2 * oc.2) - radius * radius;
    let discriminant = b * b - c;
    
    if discriminant < 0.0 {
        return RayHit { hit: false, distance: 0.0, normal: (0.0, 0.0, 0.0) };
    }
    
    let t = -b - discriminant.sqrt();
    let hit_t = if t < 0.1 {
        let t2 = -b + discriminant.sqrt();
        if t2 < 0.1 { return RayHit { hit: false, distance: 0.0, normal: (0.0, 0.0, 0.0) }; }
        t2
    } else {
        t
    };

    let hit_pos = (
        ray_origin.0 + ray_dir.0 * hit_t,
        ray_origin.1 + ray_dir.1 * hit_t,
        ray_origin.2 + ray_dir.2 * hit_t,
    );
    let normal = (
        (hit_pos.0 - sphere_center.0) / radius,
        (hit_pos.1 - sphere_center.1) / radius,
        (hit_pos.2 - sphere_center.2) / radius,
    );

    RayHit { hit: true, distance: hit_t, normal }
}