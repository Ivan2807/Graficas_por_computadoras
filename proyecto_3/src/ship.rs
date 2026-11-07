use crate::framebuffer::{Framebuffer, Pixel};

#[derive(Clone, Copy, Debug)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

pub struct Ship {
    pub vertices: Vec<Vec3>,
    pub faces: Vec<[usize; 3]>,
}

impl Ship {
    pub fn load_from_obj() -> Self {
        let obj_content = include_str!("nave low poly.obj");
        
        let mut raw_vertices: Vec<Vec3> = Vec::new();
        let mut faces: Vec<[usize; 3]> = Vec::new();

        for line in obj_content.lines() {
            let line = line.trim();
            if line.starts_with("v ") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 4 {
                    if let (Ok(x), Ok(y), Ok(z)) = (
                        parts[1].parse::<f32>(),
                        parts[2].parse::<f32>(),
                        parts[3].parse::<f32>(),
                    ) {
                        raw_vertices.push(Vec3 { x, y, z });
                    }
                }
            } else if line.starts_with("f ") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 4 {
                    let parse_vertex_idx = |s: &str| -> Option<usize> {
                        let idx_str = s.split('/').next()?;
                        let idx = idx_str.parse::<isize>().ok()?;
                        if idx < 0 {
                            Some((raw_vertices.len() as isize + idx) as usize)
                        } else {
                            Some((idx - 1) as usize)
                        }
                    };

                    if let (Some(i0), Some(i1), Some(i2)) = (
                        parse_vertex_idx(parts[1]),
                        parse_vertex_idx(parts[2]),
                        parse_vertex_idx(parts[3]),
                    ) {
                        faces.push([i0, i1, i2]);
                        if parts.len() >= 5 {
                            if let Some(i3) = parse_vertex_idx(parts[4]) {
                                faces.push([i0, i2, i3]);
                            }
                        }
                    }
                }
            }
        }

        if raw_vertices.is_empty() || faces.is_empty() {
            raw_vertices = vec![
                Vec3 { x: 0.0, y: 0.5, z: 2.0 },
                Vec3 { x: -1.0, y: -0.5, z: -1.0 },
                Vec3 { x: 1.0, y: -0.5, z: -1.0 },
                Vec3 { x: 0.0, y: -0.5, z: -1.5 },
            ];
            faces = vec![[0, 1, 2], [0, 2, 3], [0, 3, 1], [1, 3, 2]];
        }

        let mut min_x = f32::MAX; let mut max_x = f32::MIN;
        let mut min_y = f32::MAX; let mut max_y = f32::MIN;
        let mut min_z = f32::MAX; let mut max_z = f32::MIN;

        for v in &raw_vertices {
            min_x = min_x.min(v.x); max_x = max_x.max(v.x);
            min_y = min_y.min(v.y); max_y = max_y.max(v.y);
            min_z = min_z.min(v.z); max_z = max_z.max(v.z);
        }

        let center_x = (min_x + max_x) / 2.0;
        let center_y = (min_y + max_y) / 2.0;
        let center_z = (min_z + max_z) / 2.0;
        let scale = 1.5 / (max_x - min_x).max(max_y - min_y).max(max_z - min_z);

        let vertices = raw_vertices.into_iter().map(|v| {
            Vec3 {
                x: (v.x - center_x) * scale,
                y: (v.y - center_y) * scale,
                z: (v.z - center_z) * scale,
            }
        }).collect();

        Self { vertices, faces }
    }

    // Renderiza la nave FIJA Y CENTRADA en pantalla con iluminación proyectada desde el Sol
    pub fn render(&self, fb: &mut Framebuffer, cam_pos: (f32, f32, f32), yaw: f32, pitch: f32, scale: f32) {
        let ship_color = Pixel { r: 195, g: 205, b: 220, a: 255 };
        let sun_pos = (0.0, 0.0, 200.0);

        let cos_y = yaw.cos();
        let sin_y = yaw.sin();
        let cos_x = pitch.cos();
        let sin_x = pitch.sin();

        // 1. Vector dirección hacia el Sol desde la posición del mundo
        let to_sun_x = sun_pos.0 - cam_pos.0;
        let to_sun_y = sun_pos.1 - cam_pos.1;
        let to_sun_z = sun_pos.2 - cam_pos.2;
        let ts_len = (to_sun_x * to_sun_x + to_sun_y * to_sun_y + to_sun_z * to_sun_z).sqrt().max(0.001);
        let lnx = to_sun_x / ts_len;
        let lny = to_sun_y / ts_len;
        let lnz = to_sun_z / ts_len;

        for face in &self.faces {
            let v0 = self.vertices[face[0]];
            let v1 = self.vertices[face[1]];
            let v2 = self.vertices[face[2]];

            // 2. Normal local de la cara de la nave
            let edge1_x = v1.x - v0.x;
            let edge1_y = v1.y - v0.y;
            let edge1_z = v1.z - v0.z;

            let edge2_x = v2.x - v0.x;
            let edge2_y = v2.y - v0.y;
            let edge2_z = v2.z - v0.z;

            let nx = edge1_y * edge2_z - edge1_z * edge2_y;
            let ny = edge1_z * edge2_x - edge1_x * edge2_z;
            let nz = edge1_x * edge2_y - edge1_y * edge2_x;

            let n_len = (nx * nx + ny * ny + nz * nz).sqrt().max(0.001);
            let norm_x = nx / n_len;
            let norm_y = ny / n_len;
            let norm_z = nz / n_len;

            // 3. Orientar la normal según el ángulo de la cámara para saber qué cara da al Sol
            let x1 = norm_x * cos_y + norm_z * sin_y;
            let y1 = norm_y;
            let z1 = -norm_x * sin_y + norm_z * cos_y;

            let world_nx = x1;
            let world_ny = y1 * cos_x - z1 * sin_x;
            let world_nz = y1 * sin_x + z1 * cos_x;

            // 4. Producto punto para calcular la iluminación
            let dot = (world_nx * lnx + world_ny * lny + world_nz * lnz).max(0.15);

            // 5. Posición FIJA frente a la cámara (centrada en pantalla)
            let p0 = (v0.x * scale, v0.y * scale, v0.z * scale + 3.0);
            let p1 = (v1.x * scale, v1.y * scale, v1.z * scale + 3.0);
            let p2 = (v2.x * scale, v2.y * scale, v2.z * scale + 3.0);

            fb.draw_triangle(p0, p1, p2, ship_color, dot);
        }
    }
}
