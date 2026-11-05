#[derive(Clone)]
pub struct Mesh {
    pub vertices: Vec<(f32, f32, f32)>,
    pub faces: Vec<[usize; 3]>,
}

impl Mesh {
    pub fn sphere(rings: usize, sectors: usize, radius: f32) -> Self {
        let mut vertices = Vec::new();
        let mut faces = Vec::new();

        for r in 0..=rings {
            let theta = (r as f32) * std::f32::consts::PI / (rings as f32);
            let sin_theta = theta.sin();
            let cos_theta = theta.cos();

            for s in 0..=sectors {
                let phi = (s as f32) * 2.0 * std::f32::consts::PI / (sectors as f32);
                let x = radius * phi.cos() * sin_theta;
                let y = radius * cos_theta;
                let z = radius * phi.sin() * sin_theta;
                vertices.push((x, y, z));
            }
        }

        for r in 0..rings {
            for s in 0..sectors {
                let current = r * (sectors + 1) + s;
                let next = current + sectors + 1;

                faces.push([current, next, current + 1]);
                faces.push([current + 1, next, next + 1]);
            }
        }

        Self { vertices, faces }
    }
}