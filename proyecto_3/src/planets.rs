use crate::framebuffer::{Framebuffer, Pixel};
use crate::mesh::Mesh;

pub struct Moon {
    pub name: &'static str,
    pub distance: f32,
    pub speed: f32,
    pub radius: f32,
    pub color: Pixel,
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
    stars: Vec<(f32, f32, f32, Pixel)>,
}

impl SolarSystem {
    pub fn new() -> Self {
        let sun_mesh = Mesh::sphere(16, 16, 1.0);
        let planet_mesh = Mesh::sphere(12, 12, 1.0);
        let moon_mesh = Mesh::sphere(8, 8, 1.0);
        let star_count = 72000;
        let random = |seed: usize, salt: f32| {
            let value = (seed as f32 * 12.9898 + salt * 78.233).sin() * 43_758.547;
            value - value.floor()
        };
        let cloud_cells = 96;
        let stars = (0..star_count).map(|i| {
            let longitude = random(i, 9.0) * std::f32::consts::TAU;
            let cell_position = longitude / std::f32::consts::TAU * cloud_cells as f32;
            let cell = cell_position.floor() as usize % cloud_cells;
            let next_cell = (cell + 1) % cloud_cells;
            let blend = cell_position.fract();
            let cell_noise = |index: usize, salt: f32| random(index, salt) * 2.0 - 1.0;
            let offset = cell_noise(cell, 21.0) * (1.0 - blend)
                + cell_noise(next_cell, 21.0) * blend;
            let local_width = 0.035 + (random(cell, 22.0) * (1.0 - blend)
                + random(next_cell, 22.0) * blend) * 0.16;
            let center_line = (longitude * 1.35 + (longitude * 3.1).sin() * 0.24).sin() * 0.15
                + (longitude * 3.7 + 1.2).sin() * 0.065
                + (longitude * 8.3).sin() * 0.022
                + offset * 0.13;
            let y = if i % 10 < 2 {
                center_line + (random(i, 10.0) - 0.5 + random(i, 11.0) - 0.5) * local_width * 1.7
            } else if i % 10 < 4 {
                center_line + (random(i, 12.0) - 0.5 + random(i, 15.0) - 0.5) * (0.8 + local_width)
            } else {
                random(i, 13.0) * 2.0 - 1.0
            }.clamp(-0.98, 0.98);
            let ring = (1.0 - y * y).sqrt();
            let (x, z) = (ring * longitude.cos(), ring * longitude.sin());
            let pulse = match i % 8 { 0 | 1 => 1.7, 2 | 3 => 1.3, 4 | 5 => 0.9, _ => 0.58 };
            let variation = (((i * 73 % 121) as f32 + 70.0) * pulse).clamp(35.0, 255.0) as u8;
            let color = match i % 31 {
                0 => Pixel { r: 145, g: 190, b: 255, a: 255 },
                1 => Pixel { r: 255, g: 220, b: 180, a: 255 },
                _ => Pixel { r: variation.saturating_sub(12), g: variation, b: variation.saturating_add(24), a: 255 },
            };
            (x, y, z, color)
        }).collect();



        Self {
            sun_mesh,
            stars,
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
                            color: Pixel { r: 180, g: 180, b: 180, a: 255 }, angle: 0.0,
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
                            color: Pixel { r: 100, g: 150, b: 220, a: 255 }, angle: 1.0,
                            mesh: moon_mesh.clone(),
                        }
                    ],
                    mesh: planet_mesh.clone() 
                },
            ],
        }
    }

    pub fn update(&mut self, dt: f32, real_scale_mode: bool) {
        let speed_multiplier = if !real_scale_mode { 1.4 } else { 0.12 }; // Órbita ultralenta y majestuosa en Modo 2

        for p in &mut self.planets {
            p.angle += p.speed * dt * 0.3 * speed_multiplier;
            p.rotation_angle += p.rotation_speed * dt * speed_multiplier;
            for m in &mut p.moons {
                m.angle += m.speed * dt * 0.5 * speed_multiplier;
            }
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
            for (i, (star_x, star_y, star_z, color)) in self.stars.iter().enumerate() {
                let parallax = match i % 10 { 0 => 0.000035, 1..=2 => 0.000012, _ => 0.000002 };
                let world_x = star_x - cam_pos.0 * parallax;
                let world_y = star_y - cam_pos.1 * parallax;
                let world_z = star_z - (cam_pos.2 + 40.0) * parallax;
                let length = (world_x * world_x + world_y * world_y + world_z * world_z).sqrt();
                let world_x = world_x / length;
                let world_y = world_y / length;
                let world_z = world_z / length;
                let view_x = world_x * cos_y + world_z * sin_y;
                let yaw_z = -world_x * sin_y + world_z * cos_y;
                let view_y = world_y * cos_x - yaw_z * sin_x;
                let view_z = world_y * sin_x + yaw_z * cos_x;
                if view_z <= 0.1 { continue; }

                let sx = (view_x * fov / view_z + cx) as i32;
                let sy = (view_y * fov / view_z + cy) as i32;
                if sx >= 0 && sx < fb.width as i32 && sy >= 0 && sy < fb.height as i32 {
                    let idx = sy as usize * fb.width + sx as usize;
                    fb.color_buffer[idx] = *color;
                    // El núcleo queda definido y el halo redondo evita que las estrellas parezcan bloques.
                    if i % 2 == 0 {
                        let halo = Pixel { r: color.r / 3, g: color.g / 3, b: color.b / 3, a: 255 };
                        let faint = Pixel { r: color.r / 8, g: color.g / 8, b: color.b / 8, a: 255 };
                        for (dx, dy, tint) in [
                            (-1, 0, halo), (1, 0, halo), (0, -1, halo), (0, 1, halo),
                            (-1, -1, faint), (1, -1, faint), (-1, 1, faint), (1, 1, faint),
                        ] {
                            let hx = sx + dx;
                            let hy = sy + dy;
                            if hx >= 0 && hx < fb.width as i32 && hy >= 0 && hy < fb.height as i32 {
                                fb.color_buffer[hy as usize * fb.width + hx as usize] = tint;
                            }
                        }
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
                let mut v_loc0 = p.mesh.vertices[face[0]];
                let mut v_loc1 = p.mesh.vertices[face[1]];
                let mut v_loc2 = p.mesh.vertices[face[2]];

                let deform = |v: (f32, f32, f32)| -> (f32, f32, f32) {
                    let noise = (v.0 * 8.0).sin() * (v.1 * 8.0).cos() * (v.2 * 8.0).sin();
                    let factor = 1.0 + noise * 0.06;
                    (v.0 * factor, v.1 * factor, v.2 * factor)
                };

                v_loc0 = deform(v_loc0);
                v_loc1 = deform(v_loc1);
                v_loc2 = deform(v_loc2);

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
                fb.draw_triangle(v0, v1, v2, p.color, dot);
            }

            // Anillos
            if p.has_rings {
                let ring_segments = 36;
                let inner_r = actual_rad * 1.4;
                let outer_r = actual_rad * 2.3;
                let ring_color = Pixel { r: 170, g: 160, b: 130, a: 190 };

                for r_seg in 0..ring_segments {
                    let theta1 = (r_seg as f32) * 2.0 * std::f32::consts::PI / (ring_segments as f32);
                    let theta2 = ((r_seg + 1) as f32) * 2.0 * std::f32::consts::PI / (ring_segments as f32);

                    let p1_in = (inner_r * theta1.cos(), 0.0, inner_r * theta1.sin());
                    let p1_out = (outer_r * theta1.cos(), 0.0, outer_r * theta1.sin());
                    let p2_out = (outer_r * theta2.cos(), 0.0, outer_r * theta2.sin());
                    let p2_in = (inner_r * theta2.cos(), 0.0, inner_r * theta2.sin());

                    let tv1 = transform(p1_in, p_pos, 1.0, 0.0);
                    let tv2 = transform(p1_out, p_pos, 1.0, 0.0);
                    let tv3 = transform(p2_out, p_pos, 1.0, 0.0);
                    let tv4 = transform(p2_in, p_pos, 1.0, 0.0);

                    fb.draw_triangle(tv1, tv2, tv3, ring_color, 0.8);
                    fb.draw_triangle(tv1, tv3, tv4, ring_color, 0.8);
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

                    fb.draw_triangle(v0, v1, v2, m.color, dot);
                }
            }
        }

        targeted_name
    }
}
