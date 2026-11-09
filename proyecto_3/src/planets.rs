use crate::framebuffer::{Framebuffer, Pixel};
use crate::mesh::Mesh;

fn surface_color(name: &str, point: (f32, f32, f32)) -> Pixel {
    let (x, y, z) = point;
    let broad_noise = (x * 7.0 + (z * 4.0).sin()).sin()
        + (z * 6.0 - y * 3.0).cos()
        + (x * 4.0 + y * 8.0).sin() * 0.55;
    let fine_noise = (x * 23.0 + z * 17.0).sin() * (y * 19.0 - z * 13.0).cos();

    match name {
        "Moho" => {
            let shade = (broad_noise * 13.0 + fine_noise * 8.0).clamp(-25.0, 25.0);
            Pixel {
                r: (145.0 + shade) as u8,
                g: (103.0 + shade * 0.75) as u8,
                b: (73.0 + shade * 0.55) as u8,
                a: 255,
            }
        }
        "Eve" => {
            let clouds = (y * 34.0 + (x * 8.0 + z * 5.0).sin() * 1.8
                + broad_noise * 0.7)
                .sin();
            let light = (clouds * 18.0 + fine_noise * 5.0).clamp(-23.0, 23.0);
            Pixel {
                r: (164.0 + light) as u8,
                g: (88.0 + light * 0.62) as u8,
                b: (194.0 + light * 0.9) as u8,
                a: 255,
            }
        }
        "Kerbin" => {
            let land = broad_noise + fine_noise * 0.28;
            if y.abs() > 0.88 {
                Pixel { r: 222, g: 231, b: 229, a: 255 }
            } else if land > 0.43 {
                Pixel { r: 62, g: 145, b: 76, a: 255 }
            } else if land > 0.27 {
                Pixel { r: 166, g: 155, b: 103, a: 255 }
            } else {
                let clouds = (y * 29.0 + (x * 10.0 + z * 7.0).sin() * 2.0
                    + fine_noise)
                    .sin();
                if clouds > 0.91 {
                    Pixel { r: 205, g: 220, b: 229, a: 255 }
                } else {
                    Pixel { r: 39, g: 105, b: 190, a: 255 }
                }
            }
        }
        "Duna" => {
            if y.abs() > 0.87 {
                Pixel { r: 220, g: 205, b: 178, a: 255 }
            } else {
                let terrain = (broad_noise * 12.0 + fine_noise * 9.0).clamp(-20.0, 22.0);
                Pixel {
                    r: (205.0 + terrain) as u8,
                    g: (91.0 + terrain * 0.58) as u8,
                    b: (52.0 + terrain * 0.4) as u8,
                    a: 255,
                }
            }
        }
        "Mun" => {
            let craters = (x * 9.0 + (z * 6.0).cos()).sin()
                + (z * 10.0 - y * 7.0).cos();
            let shade = (craters * 13.0 + fine_noise * 5.0).clamp(-25.0, 25.0);
            Pixel {
                r: (166.0 + shade) as u8,
                g: (163.0 + shade) as u8,
                b: (155.0 + shade) as u8,
                a: 255,
            }
        }
        "Laythe" => {
            let land = broad_noise + fine_noise * 0.25;
            if y.abs() > 0.91 {
                Pixel { r: 203, g: 220, b: 219, a: 255 }
            } else if land > 0.5 {
                Pixel { r: 151, g: 166, b: 113, a: 255 }
            } else {
                Pixel { r: 54, g: 145, b: 169, a: 255 }
            }
        }
        "Jool" => {
            let bands = (y * 31.0 + (x * 5.0 + z * 4.0).sin() * 1.6 + broad_noise * 0.22)
                .sin();
            let storm = ((x - 0.33) / 0.34).powi(2)
                + ((y + 0.18) / 0.16).powi(2)
                + ((z - 0.55) / 0.45).powi(2);
            if storm < 1.0 {
                Pixel { r: 196, g: 113, b: 69, a: 255 }
            } else if bands > 0.42 {
                Pixel { r: 170, g: 204, b: 119, a: 255 }
            } else if bands < -0.5 {
                Pixel { r: 91, g: 151, b: 83, a: 255 }
            } else {
                Pixel { r: 128, g: 185, b: 99, a: 255 }
            }
        }
        _ => Pixel { r: 180, g: 180, b: 180, a: 255 },
    }
}

fn add_star_light(background: Pixel, star: Pixel, intensity: f32) -> Pixel {
    let add = |base: u8, light: u8| {
        (base as f32 + light as f32 * intensity).clamp(0.0, 255.0) as u8
    };
    Pixel {
        r: add(background.r, star.r),
        g: add(background.g, star.g),
        b: add(background.b, star.b),
        a: 255,
    }
}

fn star_intensity(time: f32, phase: f32, speed: f32) -> f32 {
    0.72 + 0.28 * (time * speed + phase).sin()
}

fn next_random(seed: &mut u64) -> f32 {
    *seed ^= *seed << 13;
    *seed ^= *seed >> 7;
    *seed ^= *seed << 17;
    (*seed >> 40) as f32 / (1u32 << 24) as f32
}

fn radical_inverse(mut index: usize, base: usize) -> f32 {
    let mut value = 0.0;
    let mut denominator = 1.0;
    while index > 0 {
        denominator *= base as f32;
        value += (index % base) as f32 / denominator;
        index /= base;
    }
    value
}

struct Star {
    direction: (f32, f32, f32),
    color: Pixel,
    twinkle_phase: f32,
    twinkle_speed: f32,
}

struct ShootingStar {
    longitude: f32,
    latitude: f32,
    longitude_speed: f32,
    latitude_speed: f32,
    age: f32,
    lifetime: f32,
}

pub struct Moon {
    pub name: &'static str,
    pub distance: f32,
    pub speed: f32,
    pub radius: f32,
    pub angle: f32,
    pub mesh: Mesh,
}

pub struct Planet {
    pub name: &'static str,
    pub distance: f32,
    pub speed: f32,
    pub radius: f32,
    pub color: Pixel,
    pub angle: f32,
    pub period_text: &'static str,
    pub gravity: f32,
    pub is_gas_giant: bool,
    pub rotation_angle: f32,
    pub rotation_speed: f32,
    pub has_rings: bool,
    pub moons: Vec<Moon>,
    pub mesh: Mesh,
}

pub struct SolarSystem {
    pub planets: Vec<Planet>,
    pub sun_mesh: Mesh,
    stars: Vec<Star>,
    animation_time: f32,
    shooting_stars: Vec<ShootingStar>,
    shooting_star_timer: f32,
    random_seed: u64,
}

impl SolarSystem {
    pub fn new() -> Self {
        let sun_mesh = Mesh::sphere(16, 16, 1.0);
        let planet_mesh = Mesh::sphere(20, 28, 1.0);
        let moon_mesh = Mesh::sphere(12, 16, 1.0);
        let star_count = 72000;
        let random = |seed: usize, salt: f32| {
            let value = (seed as f32 * 12.9898 + salt * 78.233).sin() * 43_758.547;
            value - value.floor()
        };
        let cloud_cells = 96;
        let stars = (0..star_count).map(|i| {
            let longitude = (radical_inverse(i + 1, 2) + random(i, 9.0) / star_count as f32)
                .fract()
                * std::f32::consts::TAU;
            let cell_position = longitude / std::f32::consts::TAU * cloud_cells as f32;
            let cell = cell_position.floor() as usize % cloud_cells;
            let next_cell = (cell + 1) % cloud_cells;
            let blend = cell_position.fract();
            let cell_noise = |index: usize, salt: f32| random(index, salt) * 2.0 - 1.0;
            let offset = cell_noise(cell, 21.0) * (1.0 - blend)
                + cell_noise(next_cell, 21.0) * blend;
            let local_width = 0.045 + (random(cell, 22.0) * (1.0 - blend)
                + random(next_cell, 22.0) * blend) * 0.2;
            let center_line = (longitude * 1.35 + (longitude * 3.1).sin() * 0.24).sin() * 0.15
                + (longitude * 3.7 + 1.2).sin() * 0.065
                + (longitude * 8.3).sin() * 0.022
                + offset * 0.13;
            let y = if i % 10 == 0 {
                center_line + (random(i, 10.0) - 0.5 + random(i, 11.0) - 0.5) * local_width * 1.7
            } else if i % 10 < 3 {
                center_line + (random(i, 12.0) - 0.5 + random(i, 15.0) - 0.5) * (0.8 + local_width)
            } else {
                random(i, 13.0) * 2.0 - 1.0
            }.clamp(-1.0, 1.0);
            let ring = (1.0 - y * y).sqrt();
            let (x, z) = (ring * longitude.cos(), ring * longitude.sin());
            let pulse = match i % 8 { 0 | 1 => 1.7, 2 | 3 => 1.3, 4 | 5 => 0.9, _ => 0.58 };
            let variation = (((i * 73 % 121) as f32 + 70.0) * pulse).clamp(35.0, 255.0) as u8;
            let color = match i % 31 {
                0 => Pixel { r: 145, g: 190, b: 255, a: 255 },
                1 => Pixel { r: 255, g: 220, b: 180, a: 255 },
                _ => Pixel { r: variation.saturating_sub(12), g: variation, b: variation.saturating_add(24), a: 255 },
            };
            Star {
                direction: (x, y, z),
                color,
                twinkle_phase: random(i, 31.0) * std::f32::consts::TAU,
                twinkle_speed: 0.8 + random(i, 32.0) * 2.2,
            }
        }).collect();



        Self {
            sun_mesh,
            stars,
            animation_time: 0.0,
            shooting_stars: Vec::new(),
            shooting_star_timer: 1.5,
            random_seed: 0x9E37_79B9_7F4A_7C15,
            planets: vec![
                Planet { 
                    name: "Moho", distance: 90.0, speed: 1.2, radius: 4.0, 
                    color: Pixel { r: 140, g: 110, b: 90, a: 255 }, angle: 0.0, 
                    period_text: "108d", gravity: 0.27, is_gas_giant: false, 
                    rotation_angle: 0.0, rotation_speed: 1.5, has_rings: false, moons: vec![], mesh: planet_mesh.clone() 
                },
                Planet { 
                    name: "Eve", distance: 160.0, speed: 0.8, radius: 8.0, 
                    color: Pixel { r: 150, g: 80, b: 200, a: 255 }, angle: 1.2, 
                    period_text: "1y 380d", gravity: 1.7, is_gas_giant: false, 
                    rotation_angle: 0.0, rotation_speed: 1.0, has_rings: false, moons: vec![], mesh: planet_mesh.clone() 
                },
                Planet { 
                    name: "Kerbin", distance: 240.0, speed: 0.5, radius: 8.5, 
                    color: Pixel { r: 50, g: 120, b: 220, a: 255 }, angle: 2.5, 
                    period_text: "1y 0d", gravity: 1.0, is_gas_giant: false, 
                    rotation_angle: 0.0, rotation_speed: 2.5, has_rings: false,
                    moons: vec![
                        Moon {
                            name: "Mun", distance: 42.0, speed: 3.0, radius: 2.5,
                            angle: 0.0,
                            mesh: moon_mesh.clone(),
                        }
                    ], 
                    mesh: planet_mesh.clone() 
                },
                Planet { 
                    name: "Duna", distance: 340.0, speed: 0.35, radius: 6.5, 
                    color: Pixel { r: 210, g: 90, b: 60, a: 255 }, angle: 3.8, 
                    period_text: "2y 54d", gravity: 0.3, is_gas_giant: false, 
                    rotation_angle: 0.0, rotation_speed: 2.0, has_rings: false, moons: vec![], mesh: planet_mesh.clone() 
                },
                Planet { 
                    name: "Jool", distance: 470.0, speed: 0.2, radius: 16.0, 
                    color: Pixel { r: 80, g: 190, b: 90, a: 255 }, angle: 5.0, 
                    period_text: "6y 45d", gravity: 0.8, is_gas_giant: true, 
                    rotation_angle: 0.0, rotation_speed: 3.5, has_rings: true,
                    moons: vec![
                        Moon {
                            name: "Laythe", distance: 75.0, speed: 2.8, radius: 3.5,
                            angle: 1.0,
                            mesh: moon_mesh.clone(),
                        }
                    ],
                    mesh: planet_mesh.clone() 
                },
            ],
        }
    }

    pub fn update(&mut self, dt: f32, real_scale_mode: bool) {
        self.animation_time += dt;
        let speed_multiplier = if !real_scale_mode { 1.4 } else { 0.12 }; // Órbita ultralenta y majestuosa en Modo 2

        for p in &mut self.planets {
            p.angle += p.speed * dt * 0.3 * speed_multiplier;
            p.rotation_angle += p.rotation_speed * dt * speed_multiplier;
            for m in &mut p.moons {
                m.angle += m.speed * dt * 0.5 * speed_multiplier;
            }
        }

        if real_scale_mode {
            self.shooting_star_timer -= dt;
            while self.shooting_star_timer <= 0.0 {
                let longitude = next_random(&mut self.random_seed) * std::f32::consts::TAU;
                let latitude = (next_random(&mut self.random_seed) - 0.5) * 1.3;
                let direction = if next_random(&mut self.random_seed) < 0.5 { -1.0 } else { 1.0 };
                let speed = 0.22 + next_random(&mut self.random_seed) * 0.22;
                self.shooting_stars.push(ShootingStar {
                    longitude,
                    latitude,
                    longitude_speed: speed * direction,
                    latitude_speed: (next_random(&mut self.random_seed) - 0.5) * 0.12,
                    age: 0.0,
                    lifetime: 0.7 + next_random(&mut self.random_seed) * 0.6,
                });
                self.shooting_star_timer += 2.0 + next_random(&mut self.random_seed) * 3.0;
            }

            for star in &mut self.shooting_stars {
                star.age += dt;
                star.longitude = (star.longitude + star.longitude_speed * dt)
                    .rem_euclid(std::f32::consts::TAU);
                star.latitude = (star.latitude + star.latitude_speed * dt).clamp(-1.3, 1.3);
            }
            self.shooting_stars.retain(|star| star.age < star.lifetime);
        }
    }

    pub fn render(
        &self, 
        fb: &mut Framebuffer, 
        cam_pos: (f32, f32, f32), 
        cam_yaw: f32, 
        cam_pitch: f32, 
        cos_y: f32, 
        sin_y: f32, 
        cos_x: f32, 
        sin_x: f32,
        real_scale_mode: bool
    ) -> Option<&'static str> {
        // FOV dinámico ligeramente más amplio en Modo 2 para enfatizar la inmensidad del vacío
        // Debe coincidir con el FOV del rasterizador de Framebuffer para alinear planetas y halos.
        let fov = 500.0;
        let cx = (fb.width / 2) as f32;
        let cy = (fb.height / 2) as f32;

        // Modo 2: Sol colosal (120. de radio, 1500. de distancia Z) para empequeñecer a la nave
        let (dist_mult, rad_mult, sun_radius, sun_z) = if real_scale_mode {
            (14.0, 3.0, 120.0, 2200.0) 
        } else {
            (1.0, 1.0, 30.0, 300.0)  
        };

        // --- SKYBOX: VÍA LÁCTEA FOTOGRÁFICA Y ESTRELLAS ---
        for y in 0..fb.height {
            for x in 0..fb.width {
                let fx = (x as f32 - cx) / fov;
                let fy = (y as f32 - cy) / fov;
                let fz = 1.0;

                let len = (fx * fx + fy * fy + fz * fz).sqrt();
                let nx = fx / len;
                let ny = fy / len;
                let nz = fz / len;

                let y1 = ny * cos_x + nz * sin_x;
                let z1 = -ny * sin_x + nz * cos_x;
                let x1 = nx;

                let world_x = x1 * cos_y - z1 * sin_y;
                let world_y = y1;
                let world_z = x1 * sin_y + z1 * cos_y;

                let arch_factor = world_y + (world_x * 0.22).sin() * 0.34;
                let galactic_band = (-arch_factor * arch_factor * 3.8).exp();
                let cloud_a = (world_x * 13.0 + (world_z * 8.0).sin() * 2.2).sin()
                    * (world_y * 17.0 + world_z * 5.0).cos();
                let cloud_b = (world_x * 31.0 - world_z * 23.0 + (world_y * 19.0).sin()).sin()
                    * (world_z * 27.0 + world_y * 11.0).cos();
                let cloud = (0.72 + cloud_a * 0.19 + cloud_b * 0.09).clamp(0.35, 1.25);
                let dust_lane = ((world_x * 18.0 + (world_z * 12.0).sin() * 2.0).sin()
                    * (world_z * 9.0 - world_y * 14.0).cos()).abs();
                let nebula = galactic_band * cloud * (1.0 - (1.0 - dust_lane) * 0.48);

                let r = 1.0 + nebula * 20.0 + galactic_band * cloud_a.max(0.0) * 5.0;
                let g = 1.5 + nebula * 50.0 + galactic_band * cloud_b.max(0.0) * 12.0;
                let b = 4.0 + nebula * 105.0 + galactic_band * cloud_a.max(0.0) * 24.0;

                // Estrellas dispersas por celdas angulares para evitar el ruido por píxel.
                let idx = y * fb.width + x;
                fb.color_buffer[idx] = Pixel {
                    r: r.clamp(0.0, 255.0) as u8,
                    g: g.clamp(0.0, 255.0) as u8,
                    b: b.clamp(0.0, 255.0) as u8,
                    a: 255,
                };
                fb.depth_buffer[idx] = f32::MAX;
            }
        }

        if real_scale_mode {
            for (i, star) in self.stars.iter().enumerate() {
                let (world_x, world_y, world_z) = star.direction;
                let view_x = world_x * cos_y + world_z * sin_y;
                let yaw_z = -world_x * sin_y + world_z * cos_y;
                let view_y = world_y * cos_x - yaw_z * sin_x;
                let view_z = world_y * sin_x + yaw_z * cos_x;
                if view_z <= 0.1 { continue; }

                let sx = (view_x * fov / view_z + cx) as i32;
                let sy = (view_y * fov / view_z + cy) as i32;
                if sx >= 0 && sx < fb.width as i32 && sy >= 0 && sy < fb.height as i32 {
                    let idx = sy as usize * fb.width + sx as usize;
                    let intensity = star_intensity(
                        self.animation_time,
                        star.twinkle_phase,
                        star.twinkle_speed,
                    );
                    fb.color_buffer[idx] =
                        add_star_light(fb.color_buffer[idx], star.color, intensity * 0.9);
                    if i % 2 == 0 {
                        for (dx, dy, tint) in [
                            (-1, 0, 0.24), (1, 0, 0.24), (0, -1, 0.24), (0, 1, 0.24),
                            (-1, -1, 0.08), (1, -1, 0.08), (-1, 1, 0.08), (1, 1, 0.08),
                        ] {
                            let hx = sx + dx;
                            let hy = sy + dy;
                            if hx >= 0 && hx < fb.width as i32 && hy >= 0 && hy < fb.height as i32 {
                                let halo_idx = hy as usize * fb.width + hx as usize;
                                fb.color_buffer[halo_idx] = add_star_light(
                                    fb.color_buffer[halo_idx],
                                    star.color,
                                    tint * intensity,
                                );
                            }
                        }
                    }
                }
            }

            for star in &self.shooting_stars {
                let project_direction = |longitude: f32, latitude: f32| -> Option<(i32, i32)> {
                    let cos_latitude = latitude.cos();
                    let world_x = cos_latitude * longitude.cos();
                    let world_y = latitude.sin();
                    let world_z = cos_latitude * longitude.sin();
                    let view_x = world_x * cos_y + world_z * sin_y;
                    let yaw_z = -world_x * sin_y + world_z * cos_y;
                    let view_y = world_y * cos_x - yaw_z * sin_x;
                    let view_z = world_y * sin_x + yaw_z * cos_x;
                    if view_z <= 0.1 {
                        return None;
                    }
                    Some((
                        (view_x * fov / view_z + cx) as i32,
                        (view_y * fov / view_z + cy) as i32,
                    ))
                };

                let trail_time = star.age.min(0.16);
                let tail_longitude =
                    star.longitude - star.longitude_speed * trail_time;
                let tail_latitude = star.latitude - star.latitude_speed * trail_time;
                let Some((head_x, head_y)) =
                    project_direction(star.longitude, star.latitude)
                else {
                    continue;
                };
                let Some((tail_x, tail_y)) = project_direction(tail_longitude, tail_latitude)
                else {
                    continue;
                };

                let steps = (head_x - tail_x).abs().max((head_y - tail_y).abs()).max(1);
                let fade_in = (star.age / 0.12).clamp(0.0, 1.0);
                let fade_out = ((star.lifetime - star.age) / 0.2).clamp(0.0, 1.0);
                let brightness = fade_in.min(fade_out);
                for step in 0..=steps {
                    let progress = step as f32 / steps as f32;
                    let x = tail_x + ((head_x - tail_x) as f32 * progress) as i32;
                    let y = tail_y + ((head_y - tail_y) as f32 * progress) as i32;
                    if x >= 0 && x < fb.width as i32 && y >= 0 && y < fb.height as i32 {
                        let intensity = brightness * (0.18 + progress * 0.72);
                        let idx = y as usize * fb.width + x as usize;
                        fb.color_buffer[idx] = add_star_light(
                            fb.color_buffer[idx],
                            Pixel { r: 220, g: 235, b: 255, a: 255 },
                            intensity,
                        );
                    }
                }
            }
        }

        let sun_pos = (0.0, 0.0, sun_z);

        let mut active_planet_idx: Option<usize> = None;
        for (i, p) in self.planets.iter().enumerate() {
            let actual_dist = p.distance * dist_mult;
            let px = actual_dist * p.angle.cos();
            let pz = sun_pos.2 + actual_dist * p.angle.sin();
            let dist = ((cam_pos.0 - px).powi(2) + (cam_pos.1 - 0.0).powi(2) + (cam_pos.2 - pz).powi(2)).sqrt();
            if dist < 45.0 {
                active_planet_idx = Some(i);
                break;
            }
        }

        if active_planet_idx.is_none() {
            let orbit_color = Pixel { r: 130, g: 130, b: 130, a: 140 };
            for p in &self.planets {
                let actual_dist = p.distance * dist_mult;
                let segments = 100;
                for i in 0..segments {
                    let t1 = (i as f32) * 2.0 * std::f32::consts::PI / (segments as f32);
                    let t2 = ((i + 1) as f32) * 2.0 * std::f32::consts::PI / (segments as f32);

                    let x1 = actual_dist * t1.cos();
                    let z1 = sun_pos.2 + actual_dist * t1.sin();
                    let x2 = actual_dist * t2.cos();
                    let z2 = sun_pos.2 + actual_dist * t2.sin();

                    let transform_orbit = |wx: f32, wz: f32| -> (f32, f32, f32) {
                        let rel_x = wx - cam_pos.0;
                        let rel_y = 0.0 - cam_pos.1;
                        let rel_z = wz - cam_pos.2;
                        let x1 = rel_x * cos_y - rel_z * sin_y;
                        let y1 = rel_y;
                        let z1 = rel_x * sin_y + rel_z * cos_y;
                        (x1, y1 * cos_x - z1 * sin_x, y1 * sin_x + z1 * cos_x)
                    };

                    let p1 = transform_orbit(x1, z1);
                    let p2 = transform_orbit(x2, z2);

                    if p1.2 > 0.5 && p2.2 > 0.5 {
                        let sx1 = ((p1.0 * fov) / p1.2 + cx) as i32;
                        let sy1 = ((p1.1 * fov) / p1.2 + cy) as i32;
                        let sx2 = ((p2.0 * fov) / p2.2 + cx) as i32;
                        let sy2 = ((p2.1 * fov) / p2.2 + cy) as i32;

                        fb.set_pixel(sx1, sy1, p1.2, orbit_color);
                        fb.set_pixel(sx2, sy2, p2.2, orbit_color);
                    }
                }
            }
        }

        let transform = |v: (f32, f32, f32), world_pos: (f32, f32, f32), scale: f32, rot_angle: f32| -> (f32, f32, f32) {
            let cos_r = rot_angle.cos();
            let sin_r = rot_angle.sin();
            let rx = v.0 * cos_r + v.2 * sin_r;
            let ry = v.1;
            let rz = -v.0 * sin_r + v.2 * cos_r;

            let wx = rx * scale + world_pos.0;
            let wy = ry * scale + world_pos.1;
            let wz = rz * scale + world_pos.2;

            let rel_x = wx - cam_pos.0;
            let rel_y = wy - cam_pos.1;
            let rel_z = wz - cam_pos.2;

            let x1 = rel_x * cos_y - rel_z * sin_y;
            let y1 = rel_y;
            let z1 = rel_x * sin_y + rel_z * cos_y;

            let x2 = x1;
            let y2 = y1 * cos_x - z1 * sin_x;
            let z2 = y1 * sin_x + z1 * cos_x;

            (x2, y2, z2)
        };

        // --- SOL COLOSAL ---
        let rel_sun_x = sun_pos.0 - cam_pos.0;
        let rel_sun_y = sun_pos.1 - cam_pos.1;
        let rel_sun_z = sun_pos.2 - cam_pos.2;

        let sx1 = rel_sun_x * cos_y - rel_sun_z * sin_y;
        let sy1 = rel_sun_y;
        let sz1 = rel_sun_x * sin_y + rel_sun_z * cos_y;
        let sun_cam_z = sy1 * sin_x + sz1 * cos_x;

        if sun_cam_z > 0.5 {
            let sun_cam_x = sx1;
            let sun_cam_y = sy1 * cos_x - sz1 * sin_x;
            let sun_sx = (sun_cam_x * fov / sun_cam_z + cx) as i32;
            let sun_sy = (sun_cam_y * fov / sun_cam_z + cy) as i32;

            let glow_radius = sun_radius * 3.5; 
            let min_x = (sun_sx as f32 - glow_radius) as i32;
            let max_x = (sun_sx as f32 + glow_radius) as i32;
            let min_y = (sun_sy as f32 - glow_radius) as i32;
            let max_y = (sun_sy as f32 + glow_radius) as i32;

            for y in min_y..=max_y {
                for x in min_x..=max_x {
                    if x >= 0 && x < fb.width as i32 && y >= 0 && y < fb.height as i32 {
                        let dx = x as f32 - sun_sx as f32;
                        let dy = y as f32 - sun_sy as f32;
                        let dist = (dx * dx + dy * dy).sqrt();

                        if dist < glow_radius {
                            let factor = (1.0 - (dist / glow_radius)).powf(1.4); 
                            let idx = (y as usize) * fb.width + (x as usize);
                            let current = fb.color_buffer[idx];

                            let r = (current.r as f32 + (255.0 * 0.85 * factor)).min(255.0) as u8;
                            let g = (current.g as f32 + (235.0 * 0.8 * factor)).min(255.0) as u8;
                            let b = (current.b as f32 + (160.0 * 0.65 * factor)).min(255.0) as u8;

                            fb.color_buffer[idx] = Pixel { r, g, b, a: 255 };
                        }
                    }
                }
            }
        }

        let sun_color = Pixel { r: 255, g: 230, b: 90, a: 255 };
        let sun_rot = fb.width as f32 * 0.001;
        for face in &self.sun_mesh.faces {
            let v0 = transform(self.sun_mesh.vertices[face[0]], sun_pos, sun_radius, sun_rot);
            let v1 = transform(self.sun_mesh.vertices[face[1]], sun_pos, sun_radius, sun_rot);
            let v2 = transform(self.sun_mesh.vertices[face[2]], sun_pos, sun_radius, sun_rot);
            fb.draw_triangle(v0, v1, v2, sun_color, 1.0);
        }

        // --- PLANETAS Y LUNAS ---
        let mut targeted_name = None;
        for (i, p) in self.planets.iter().enumerate() {
            let actual_dist = p.distance * dist_mult;
            let actual_rad = p.radius * rad_mult;

            let px = actual_dist * p.angle.cos();
            let pz = sun_pos.2 + actual_dist * p.angle.sin();
            let p_pos = (px, 0.0, pz);

            if let Some(active_idx) = active_planet_idx {
                if active_idx != i { continue; }
                else { targeted_name = Some(p.name); }
            }

            // Aura atmosférica suave para separar visualmente cada planeta del fondo.
            let center_cam = transform((0.0, 0.0, 0.0), p_pos, 1.0, 0.0);
            if center_cam.2 > 0.5 {
                let screen_x = center_cam.0 * fov / center_cam.2 + cx;
                let screen_y = center_cam.1 * fov / center_cam.2 + cy;
                let apparent_radius = (actual_rad * fov / center_cam.2).clamp(1.0, 100.0);
                let inner_radius = apparent_radius * 0.8;
                let outer_radius = (apparent_radius * 1.65 + 2.0).min(130.0);
                let min_x = (screen_x - outer_radius).floor().max(0.0) as usize;
                let max_x = (screen_x + outer_radius).ceil().min((fb.width - 1) as f32) as usize;
                let min_y = (screen_y - outer_radius).floor().max(0.0) as usize;
                let max_y = (screen_y + outer_radius).ceil().min((fb.height - 1) as f32) as usize;

                for y in min_y..=max_y {
                    for x in min_x..=max_x {
                        let dx = x as f32 + 0.5 - screen_x;
                        let dy = y as f32 + 0.5 - screen_y;
                        let distance = (dx * dx + dy * dy).sqrt();
                        if distance <= inner_radius || distance >= outer_radius { continue; }

                        let idx = y * fb.width + x;
                        if fb.depth_buffer[idx] < center_cam.2 - actual_rad { continue; }
                        let fade = (1.0 - (distance - inner_radius) / (outer_radius - inner_radius)).powi(2) * 0.28;
                        let current = fb.color_buffer[idx];
                        fb.color_buffer[idx] = Pixel {
                            r: (current.r as f32 + p.color.r as f32 * fade).min(255.0) as u8,
                            g: (current.g as f32 + p.color.g as f32 * fade).min(255.0) as u8,
                            b: (current.b as f32 + p.color.b as f32 * fade).min(255.0) as u8,
                            a: 255,
                        };
                    }
                }
            }

            if p.is_gas_giant {
                let core_radius = actual_rad * 0.55;
                let core_color = Pixel { r: 130, g: 240, b: 140, a: 255 };
                for face in &p.mesh.faces {
                    let v0 = transform(p.mesh.vertices[face[0]], p_pos, core_radius, -p.rotation_angle);
                    let v1 = transform(p.mesh.vertices[face[1]], p_pos, core_radius, -p.rotation_angle);
                    let v2 = transform(p.mesh.vertices[face[2]], p_pos, core_radius, -p.rotation_angle);
                    fb.draw_triangle(v0, v1, v2, core_color, 0.8);
                }
            }

            for face in &p.mesh.faces {
                let v_loc0 = p.mesh.vertices[face[0]];
                let v_loc1 = p.mesh.vertices[face[1]];
                let v_loc2 = p.mesh.vertices[face[2]];

                let v0 = transform(v_loc0, p_pos, actual_rad, p.rotation_angle);
                let v1 = transform(v_loc1, p_pos, actual_rad, p.rotation_angle);
                let v2 = transform(v_loc2, p_pos, actual_rad, p.rotation_angle);

                let avg_x = (v_loc0.0 + v_loc1.0 + v_loc2.0) / 3.0;
                let avg_y = (v_loc0.1 + v_loc1.1 + v_loc2.1) / 3.0;
                let avg_z = (v_loc0.2 + v_loc1.2 + v_loc2.2) / 3.0;
                
                let cos_r = p.rotation_angle.cos();
                let sin_r = p.rotation_angle.sin();
                let nx = avg_x * cos_r + avg_z * sin_r;
                let ny = avg_y;
                let nz = -avg_x * sin_r + avg_z * cos_r;
                
                let n_len = (nx * nx + ny * ny + nz * nz).sqrt().max(0.001);
                let nx = nx / n_len;
                let ny = ny / n_len;
                let nz = nz / n_len;

                let to_sun_x = sun_pos.0 - p_pos.0;
                let to_sun_y = sun_pos.1 - p_pos.1;
                let to_sun_z = sun_pos.2 - p_pos.2;
                let ts_len = (to_sun_x * to_sun_x + to_sun_y * to_sun_y + to_sun_z * to_sun_z).sqrt().max(0.001);
                let lnx = to_sun_x / ts_len;
                let lny = to_sun_y / ts_len;
                let lnz = to_sun_z / ts_len;

                let dot = (nx * lnx + ny * lny + nz * lnz).max(0.15);
                let color = surface_color(p.name, (avg_x, avg_y, avg_z));
                fb.draw_triangle(v0, v1, v2, color, dot);
            }

            // Bandas y huecos del sistema de anillos de Jool.
            if p.has_rings {
                let ring_segments = 128;
                let ring_bands = [
                    (1.25, 1.48, Pixel { r: 163, g: 157, b: 133, a: 255 }),
                    (1.50, 1.76, Pixel { r: 199, g: 187, b: 151, a: 255 }),
                    (1.84, 2.10, Pixel { r: 133, g: 139, b: 119, a: 255 }),
                    (2.12, 2.28, Pixel { r: 181, g: 173, b: 146, a: 255 }),
                    (2.34, 2.58, Pixel { r: 145, g: 150, b: 133, a: 255 }),
                    (2.62, 2.82, Pixel { r: 191, g: 181, b: 150, a: 255 }),
                ];

                for (inner_ratio, outer_ratio, ring_color) in ring_bands {
                    let inner_r = actual_rad * inner_ratio;
                    let outer_r = actual_rad * outer_ratio;
                    for r_seg in 0..ring_segments {
                        let theta1 =
                            (r_seg as f32) * 2.0 * std::f32::consts::PI / ring_segments as f32;
                        let theta2 = ((r_seg + 1) as f32) * 2.0 * std::f32::consts::PI
                            / ring_segments as f32;

                        let tilt = 0.18;
                        let tilted = |radius: f32, angle: f32| {
                            let z = radius * angle.sin();
                            (radius * angle.cos(), z * tilt, z)
                        };
                        let p1_in = tilted(inner_r, theta1);
                        let p1_out = tilted(outer_r, theta1);
                        let p2_out = tilted(outer_r, theta2);
                        let p2_in = tilted(inner_r, theta2);

                        let tv1 = transform(p1_in, p_pos, 1.0, 0.0);
                        let tv2 = transform(p1_out, p_pos, 1.0, 0.0);
                        let tv3 = transform(p2_out, p_pos, 1.0, 0.0);
                        let tv4 = transform(p2_in, p_pos, 1.0, 0.0);

                        fb.draw_triangle(tv1, tv2, tv3, ring_color, 0.8);
                        fb.draw_triangle(tv1, tv3, tv4, ring_color, 0.8);
                    }
                }
            }

            // Lunas
            for m in &p.moons {
                let scaled_moon_dist = m.distance * (rad_mult.powf(0.65));
                let mx = px + scaled_moon_dist * m.angle.cos();
                let mz = pz + scaled_moon_dist * m.angle.sin();
                let m_pos = (mx, 0.0, mz);

                for face in &m.mesh.faces {
                    let local_v0 = m.mesh.vertices[face[0]];
                    let local_v1 = m.mesh.vertices[face[1]];
                    let local_v2 = m.mesh.vertices[face[2]];

                    let v0 = transform(local_v0, m_pos, m.radius * rad_mult, 0.0);
                    let v1 = transform(local_v1, m_pos, m.radius * rad_mult, 0.0);
                    let v2 = transform(local_v2, m_pos, m.radius * rad_mult, 0.0);

                    let to_sun_x = sun_pos.0 - m_pos.0;
                    let to_sun_y = sun_pos.1 - m_pos.1;
                    let to_sun_z = sun_pos.2 - m_pos.2;
                    let ts_len = (to_sun_x * to_sun_x + to_sun_y * to_sun_y + to_sun_z * to_sun_z).sqrt().max(0.001);
                    
                    let avg_x = (local_v0.0 + local_v1.0 + local_v2.0) / 3.0;
                    let avg_y = (local_v0.1 + local_v1.1 + local_v2.1) / 3.0;
                    let avg_z = (local_v0.2 + local_v1.2 + local_v2.2) / 3.0;
                    let n_len = (avg_x * avg_x + avg_y * avg_y + avg_z * avg_z).sqrt().max(0.001);

                    let dot = ((avg_x / n_len) * (to_sun_x / ts_len) 
                             + (avg_y / n_len) * (to_sun_y / ts_len) 
                             + (avg_z / n_len) * (to_sun_z / ts_len)).max(0.15);

                    let color = surface_color(m.name, (avg_x, avg_y, avg_z));
                    fb.draw_triangle(v0, v1, v2, color, dot);
                }
            }
        }

        targeted_name
    }
}

#[cfg(test)]
mod tests {
    use super::{radical_inverse, star_intensity};

    #[test]
    fn stars_dim_without_turning_off() {
        let bright = star_intensity(std::f32::consts::FRAC_PI_2, 0.0, 1.0);
        let dim = star_intensity(3.0 * std::f32::consts::FRAC_PI_2, 0.0, 1.0);

        assert!((bright - 1.0).abs() < 0.001);
        assert!(dim > 0.4);
        assert!(dim < bright);
    }

    #[test]
    fn star_longitudes_are_evenly_distributed() {
        let mut bins = [0; 64];
        for i in 1..=1024 {
            let bin = (radical_inverse(i, 2) * bins.len() as f32) as usize;
            bins[bin] += 1;
        }

        assert!(bins.iter().all(|count| *count == 16));
    }
}
