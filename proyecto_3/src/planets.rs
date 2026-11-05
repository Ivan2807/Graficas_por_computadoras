use crate::framebuffer::{Framebuffer, Pixel};
use crate::mesh::Mesh;

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
    pub mesh: Mesh,
}

pub struct SolarSystem {
    pub planets: Vec<Planet>,
    pub sun_mesh: Mesh,
}

impl SolarSystem {
    pub fn new() -> Self {
        let sun_mesh = Mesh::sphere(16, 16, 25.0);
        let planet_mesh = Mesh::sphere(12, 12, 1.0);

        Self {
            sun_mesh,
            planets: vec![
                Planet { 
                    name: "Moho", distance: 70.0, speed: 1.0, radius: 4.0, 
                    color: Pixel { r: 140, g: 110, b: 90, a: 255 }, angle: 0.0, 
                    period_text: "108d", gravity: 0.27, is_gas_giant: false, 
                    rotation_angle: 0.0, rotation_speed: 1.2, mesh: planet_mesh.clone() 
                },
                Planet { 
                    name: "Eve", distance: 120.0, speed: 0.7, radius: 8.5, 
                    color: Pixel { r: 150, g: 80, b: 200, a: 255 }, angle: 1.2, 
                    period_text: "1y 380d", gravity: 1.7, is_gas_giant: false, 
                    rotation_angle: 0.0, rotation_speed: 0.8, mesh: planet_mesh.clone() 
                },
                Planet { 
                    name: "Kerbin", distance: 180.0, speed: 0.5, radius: 9.0, 
                    color: Pixel { r: 50, g: 120, b: 220, a: 255 }, angle: 2.5, 
                    period_text: "1y 0d", gravity: 1.0, is_gas_giant: false, 
                    rotation_angle: 0.0, rotation_speed: 2.5, mesh: planet_mesh.clone() 
                },
                Planet { 
                    name: "Duna", distance: 250.0, speed: 0.35, radius: 7.0, 
                    color: Pixel { r: 210, g: 90, b: 60, a: 255 }, angle: 3.8, 
                    period_text: "2y 54d", gravity: 0.3, is_gas_giant: false, 
                    rotation_angle: 0.0, rotation_speed: 2.2, mesh: planet_mesh.clone() 
                },
                Planet { 
                    name: "Jool", distance: 340.0, speed: 0.2, radius: 18.0, 
                    color: Pixel { r: 80, g: 190, b: 90, a: 255 }, angle: 5.0, 
                    period_text: "6y 45d", gravity: 0.8, is_gas_giant: true, 
                    rotation_angle: 0.0, rotation_speed: 4.0, mesh: planet_mesh.clone() 
                },
            ],
        }
    }

    pub fn update(&mut self, dt: f32) {
        for p in &mut self.planets {
            p.angle += p.speed * dt * 0.3;
            p.rotation_angle += p.rotation_speed * dt;
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
        sin_x: f32
    ) -> Option<&'static str> {
        // --- SKYBOX PROCEDURAL CON BANDA GALÁCTICA ---
        let fov = 500.0;
        let cx = (fb.width / 2) as f32;
        let cy = (fb.height / 2) as f32;

        for y in 0..fb.height {
            for x in 0..fb.width {
                let fx = (x as f32 - cx) / fov;
                let fy = (y as f32 - cy) / fov;
                let fz = 1.0;

                let len = (fx * fx + fy * fy + fz * fz).sqrt();
                let nx = fx / len;
                let ny = fy / len;
                let nz = fz / len;

                // Desrotar con pitch y yaw de la cámara
                let y1 = ny * cos_x + nz * sin_x;
                let z1 = -ny * sin_x + nz * cos_x;
                let x1 = nx;

                let world_x = x1 * cos_y - z1 * sin_y;
                let world_y = y1;
                let world_z = x1 * sin_y + z1 * cos_y;

                // Definir el plano galáctico (inclinado para que cruce la pantalla como en KSP)
                let galactic_plane = (world_y + world_x * 0.3).abs();
                let density_factor = (-galactic_plane * 3.0).exp(); // Mayor concentración en el centro de la banda

                // Color base de espacio profundo con tinte lechoso en el plano galáctico
                let mut r = 5.0 + density_factor * 35.0;
                let mut g = 5.0 + density_factor * 30.0;
                let mut b = 10.0 + density_factor * 50.0;

                // Nebulosas suaves acumuladas en la banda galáctica
                let nebula = ((world_x * 4.0).sin() * (world_y * 4.0).cos() * (world_z * 4.0).sin()).abs();
                r += nebula * density_factor * 60.0;
                g += nebula * density_factor * 45.0;
                b += nebula * density_factor * 90.0;

                // Estrellas esparcidas globalmente pero más densas y brillantes en la banda galáctica
                let p_val = (world_x * 129.898 + world_y * 78.233 + world_z * 37.719).sin() * 43758.5453;
                let random_fract = p_val - p_val.floor();

                // Probabilidad de estrella modulada por la densidad galáctica (más estrellas ahí)
                let threshold = 0.975 - (density_factor * 0.015);

                if random_fract > threshold {
                    // Determinar brillo: las estrellas muy brillantes están más agrupadas en la banda
                    let intensity_raw = (random_fract - threshold) / (1.0 - threshold);
                    
                    if intensity_raw > 0.85 && galactic_plane < 0.5 {
                        // Estrellas gigantes / cúmulos muy brillosos en la Vía Láctea
                        let boost = intensity_raw * 1.5;
                        r += 255.0 * boost;
                        g += 245.0 * boost;
                        b += 230.0 * boost;
                    } else if intensity_raw > 0.5 {
                        // Estrellas medianas
                        r += 180.0;
                        g += 190.0;
                        b += 220.0;
                    } else {
                        // Estrellas tenues esparcidas por todas partes
                        r += 90.0;
                        g += 90.0;
                        b += 110.0;
                    }
                }

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

        let sun_pos = (0.0, 0.0, 200.0);

        // Rango de detección estricto para culling de planetas
        let mut active_planet_idx: Option<usize> = None;
        for (i, p) in self.planets.iter().enumerate() {
            let px = p.distance * p.angle.cos();
            let pz = sun_pos.2 + p.distance * p.angle.sin();
            let dist = ((cam_pos.0 - px).powi(2) + (cam_pos.1 - 0.0).powi(2) + (cam_pos.2 - pz).powi(2)).sqrt();
            if dist < 35.0 {
                active_planet_idx = Some(i);
                break;
            }
        }

        // Renderizar órbitas si no hay un planeta enfocado de cerca
        if active_planet_idx.is_none() {
            let orbit_color = Pixel { r: 200, g: 200, b: 200, a: 255 };
            for p in &self.planets {
                let segments = 80;
                for i in 0..segments {
                    let t1 = (i as f32) * 2.0 * std::f32::consts::PI / (segments as f32);
                    let t2 = ((i + 1) as f32) * 2.0 * std::f32::consts::PI / (segments as f32);

                    let x1 = p.distance * t1.cos();
                    let z1 = sun_pos.2 + p.distance * t1.sin();
                    let x2 = p.distance * t2.cos();
                    let z2 = sun_pos.2 + p.distance * t2.sin();

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

        // --- SOL CENTRAL Y CORONA ESTELAR ---
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

            let glow_radius = 175.0; 
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

                            let r = (current.r as f32 + (255.0 * 0.75 * factor)).min(255.0) as u8;
                            let g = (current.g as f32 + (225.0 * 0.70 * factor)).min(255.0) as u8;
                            let b = (current.b as f32 + (140.0 * 0.55 * factor)).min(255.0) as u8;

                            fb.color_buffer[idx] = Pixel { r, g, b, a: 255 };
                        }
                    }
                }
            }
        }

        let sun_color = Pixel { r: 255, g: 230, b: 90, a: 255 };
        let sun_rot = fb.width as f32 * 0.001;
        for face in &self.sun_mesh.faces {
            let v0 = transform(self.sun_mesh.vertices[face[0]], sun_pos, 1.0, sun_rot);
            let v1 = transform(self.sun_mesh.vertices[face[1]], sun_pos, 1.0, sun_rot);
            let v2 = transform(self.sun_mesh.vertices[face[2]], sun_pos, 1.0, sun_rot);
            fb.draw_triangle(v0, v1, v2, sun_color, 1.0);
        }

        // --- PLANETAS ---
        let mut targeted_name = None;
        for (i, p) in self.planets.iter().enumerate() {
            if let Some(active_idx) = active_planet_idx {
                if active_idx != i {
                    continue; 
                } else {
                    targeted_name = Some(p.name);
                }
            }

            let px = p.distance * p.angle.cos();
            let pz = sun_pos.2 + p.distance * p.angle.sin();
            let p_pos = (px, 0.0, pz);

            for face in &p.mesh.faces {
                let local_v0 = p.mesh.vertices[face[0]];
                let local_v1 = p.mesh.vertices[face[1]];
                let local_v2 = p.mesh.vertices[face[2]];

                let v0 = transform(local_v0, p_pos, p.radius, p.rotation_angle);
                let v1 = transform(local_v1, p_pos, p.radius, p.rotation_angle);
                let v2 = transform(local_v2, p_pos, p.radius, p.rotation_angle);

                let avg_x = (local_v0.0 + local_v1.0 + local_v2.0) / 3.0;
                let avg_y = (local_v0.1 + local_v1.1 + local_v2.1) / 3.0;
                let avg_z = (local_v0.2 + local_v1.2 + local_v2.2) / 3.0;
                
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
        }

        targeted_name
    }
}