use raylib::prelude::*;

const WALL_TEXTURE_SIZE: i32 = 32;

pub struct WallTexture {
    pub width: i32,
    pub height: i32,
    pixels: Vec<Color>,
}

impl WallTexture {
    pub fn load(path: &str) -> Self {
        let mut img = Image::load_image(path)
            .unwrap_or_else(|_| panic!("no se pudo cargar la textura {}", path));
        // Las paredes usan texturas pixel-art normalizadas a 32x32.
        img.resize_nn(WALL_TEXTURE_SIZE, WALL_TEXTURE_SIZE);
        let width = img.width;
        let height = img.height;
        
        // Convertimos ImageColors al tipo Vec<Color> que requiere el struct
        let pixels = img.get_image_data().as_ref().to_vec();

        WallTexture { width, height, pixels }
    }

    pub fn sample(&self, u: f32, v: f32) -> Color {
        let u = u.rem_euclid(1.0);
        let v = v.clamp(0.0, 0.999);
        let x = ((u * self.width as f32) as i32).clamp(0, self.width - 1);
        let y = ((v * self.height as f32) as i32).clamp(0, self.height - 1);
        self.pixels[(y * self.width + x) as usize]
    }
}

pub struct Textures {
    pub red: WallTexture,
    pub purple: WallTexture,
    pub stone: WallTexture,
}

impl Textures {
    pub fn load_all() -> Self {
        Textures {
            red: WallTexture::load("assets/textures/brick_red.png"),
            purple: WallTexture::load("assets/textures/brick_purple.png"),
            stone: WallTexture::load("assets/textures/stone_gray.png"),
        }
    }

    pub fn for_wall_id(&self, id: u8) -> &WallTexture {
        match id {
            1 => &self.red,
            5 => &self.purple,
            _ => &self.stone,
        }
    }
}