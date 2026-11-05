use image::GenericImageView;

#[derive(Clone, Copy, Debug)]
pub struct Pixel {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Pixel {
    pub const BLACK: Pixel = Pixel { r: 5, g: 5, b: 12, a: 255 };
}

pub struct Texture {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<Pixel>,
}

impl Texture {
    pub fn load(path: &str) -> Self {
        let img = image::open(path).expect("No se pudo cargar la imagen de fondo space_bg.png");
        let (width, height) = img.dimensions();
        let mut pixels = Vec::new();

        for y in 0..height {
            for x in 0..width {
                let p = img.get_pixel(x, y);
                pixels.push(Pixel {
                    r: p[0],
                    g: p[1],
                    b: p[2],
                    a: p[3],
                });
            }
        }

        Self { width: width as usize, height: height as usize, pixels }
    }

    // Muestreo con filtrado bilineal para suavizar y aumentar la calidad visual
    pub fn sample(&self, u: f32, v: f32) -> Pixel {
        let u_clamped = u.clamp(0.0, 1.0);
        let v_clamped = (1.0 - v).clamp(0.0, 1.0);

        let fx = u_clamped * (self.width as f32 - 1.0);
        let fy = v_clamped * (self.height as f32 - 1.0);

        let x0 = fx.floor() as usize;
        let y0 = fy.floor() as usize;
        let x1 = (x0 + 1).min(self.width - 1);
        let y1 = (y0 + 1).min(self.height - 1);

        let dx = fx - x0 as f32;
        let dy = fy - y0 as f32;

        let p00 = self.pixels[y0 * self.width + x0];
        let p10 = self.pixels[y0 * self.width + x1];
        let p01 = self.pixels[y1 * self.width + x0];
        let p11 = self.pixels[y1 * self.width + x1];

        let r = (p00.r as f32 * (1.0 - dx) * (1.0 - dy)
            + p10.r as f32 * dx * (1.0 - dy)
            + p01.r as f32 * (1.0 - dx) * dy
            + p11.r as f32 * dx * dy) as u8;

        let g = (p00.g as f32 * (1.0 - dx) * (1.0 - dy)
            + p10.g as f32 * dx * (1.0 - dy)
            + p01.g as f32 * (1.0 - dx) * dy
            + p11.g as f32 * dx * dy) as u8;

        let b = (p00.b as f32 * (1.0 - dx) * (1.0 - dy)
            + p10.b as f32 * dx * (1.0 - dy)
            + p01.b as f32 * (1.0 - dx) * dy
            + p11.b as f32 * dx * dy) as u8;

        Pixel { r, g, b, a: 255 }
    }
}

pub struct Framebuffer {
    pub width: usize,
    pub height: usize,
    pub color_buffer: Vec<Pixel>,
    pub depth_buffer: Vec<f32>,
}

impl Framebuffer {
    pub fn new(width: usize, height: usize) -> Self {
        let size = width * height;
        Self {
            width,
            height,
            color_buffer: vec![Pixel::BLACK; size],
            depth_buffer: vec![f32::MAX; size],
        }
    }

    pub fn clear(&mut self) {
        for p in self.color_buffer.iter_mut() { *p = Pixel::BLACK; }
        for d in self.depth_buffer.iter_mut() { *d = f32::MAX; }
    }

    pub fn set_pixel(&mut self, x: i32, y: i32, depth: f32, color: Pixel) {
        if x >= 0 && x < self.width as i32 && y >= 0 && y < self.height as i32 {
            let idx = (y as usize) * self.width + (x as usize);
            if depth < self.depth_buffer[idx] {
                self.depth_buffer[idx] = depth;
                self.color_buffer[idx] = color;
            }
        }
    }

    pub fn draw_triangle(
        &mut self,
        p0: (f32, f32, f32),
        p1: (f32, f32, f32),
        p2: (f32, f32, f32),
        color: Pixel,
        light_intensity: f32,
    ) {
        let fov = 500.0;
        let cx = (self.width / 2) as f32;
        let cy = (self.height / 2) as f32;

        let project = |v: (f32, f32, f32)| -> Option<(i32, i32, f32)> {
            if v.2 <= 0.1 { return None; }
            let x = ((v.0 * fov) / v.2 + cx) as i32;
            let y = ((v.1 * fov) / v.2 + cy) as i32;
            Some((x, y, v.2))
        };

        let v0 = match project(p0) { Some(val) => val, None => return };
        let v1 = match project(p1) { Some(val) => val, None => return };
        let v2 = match project(p2) { Some(val) => val, None => return };

        let min_x = v0.0.min(v1.0).min(v2.0).max(0);
        let max_x = v0.0.max(v1.0).max(v2.0).min(self.width as i32 - 1);
        let min_y = v0.1.min(v1.1).min(v2.1).max(0);
        let max_y = v0.1.max(v1.1).max(v2.1).min(self.height as i32 - 1);

        let area = (v1.0 as f32 - v0.0 as f32) * (v2.1 as f32 - v0.1 as f32) - (v1.1 as f32 - v0.1 as f32) * (v2.0 as f32 - v0.0 as f32);
        if area.abs() < 0.5 { return; }

        let shaded_color = Pixel {
            r: (color.r as f32 * light_intensity.clamp(0.15, 1.0)) as u8,
            g: (color.g as f32 * light_intensity.clamp(0.15, 1.0)) as u8,
            b: (color.b as f32 * light_intensity.clamp(0.15, 1.0)) as u8,
            a: color.a,
        };

        for y in min_y..=max_y {
            for x in min_x..=max_x {
                let px = x as f32 + 0.5;
                let py = y as f32 + 0.5;

                let w0 = ((v1.0 as f32 - px) * (v2.1 as f32 - py) - (v1.1 as f32 - py) * (v2.0 as f32 - px)) / area;
                let w1 = ((v2.0 as f32 - px) * (v0.1 as f32 - py) - (v2.1 as f32 - py) * (v0.0 as f32 - px)) / area;
                let w2 = 1.0 - w0 - w1;

                if w0 >= 0.0 && w1 >= 0.0 && w2 >= 0.0 {
                    let depth = w0 * v0.2 + w1 * v1.2 + w2 * v2.2;
                    let idx = (y as usize) * self.width + (x as usize);
                    if depth < self.depth_buffer[idx] {
                        self.depth_buffer[idx] = depth;
                        self.color_buffer[idx] = shaded_color;
                    }
                }
            }
        }
    }
}