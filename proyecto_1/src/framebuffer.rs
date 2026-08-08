use raylib::prelude::Color;

/// Framebuffer propio: es solo un arreglo de bytes RGBA en el que el
/// raycaster dibuja pixel por pixel, sin llamar a ninguna funcion de
/// dibujo de raylib. Al final de cada frame, este buffer se sube a una
/// Texture2D de raylib para poder mostrarlo en pantalla (ver main.rs).
pub struct Framebuffer {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<u8>, // RGBA8, largo = width * height * 4
}

impl Framebuffer {
    pub fn new(width: usize, height: usize) -> Self {
        Framebuffer {
            width,
            height,
            pixels: vec![0u8; width * height * 4],
        }
    }

    #[inline]
    pub fn set_pixel(&mut self, x: usize, y: usize, color: Color) {
        if x >= self.width || y >= self.height {
            return;
        }
        let idx = (y * self.width + x) * 4;
        self.pixels[idx] = color.r;
        self.pixels[idx + 1] = color.g;
        self.pixels[idx + 2] = color.b;
        self.pixels[idx + 3] = color.a;
    }

    pub fn clear(&mut self, color: Color) {
        for i in 0..(self.width * self.height) {
            let idx = i * 4;
            self.pixels[idx] = color.r;
            self.pixels[idx + 1] = color.g;
            self.pixels[idx + 2] = color.b;
            self.pixels[idx + 3] = color.a;
        }
    }
}
