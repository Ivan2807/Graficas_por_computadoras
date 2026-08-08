use crate::level::Level;

pub struct Player {
    pub x: f32,
    pub y: f32,
    pub angle: f32, // radianes
    pub radius: f32,
}

impl Player {
    pub fn new(x: f32, y: f32) -> Self {
        Player { x, y, angle: 0.0, radius: 0.2 }
    }

    /// Intenta mover al jugador dx, dy (en unidades de tile). Revisa
    /// colision contra el mapa ANTES de aplicar el movimiento, y prueba
    /// cada eje por separado para poder "deslizarse" sobre las paredes en
    /// vez de quedar pegado (igual que en Wolfenstein/DOOM).
    pub fn try_move(&mut self, dx: f32, dy: f32, level: &Level) {
        if !self.collides(self.x + dx, self.y, level) {
            self.x += dx;
        }
        if !self.collides(self.x, self.y + dy, level) {
            self.y += dy;
        }
    }

    fn collides(&self, x: f32, y: f32, level: &Level) -> bool {
        // revisa 4 puntos alrededor del jugador (bounding box del radio)
        // para no "encajarse" al pasar cerca de una esquina de pared
        let offsets = [
            (-self.radius, -self.radius),
            (self.radius, -self.radius),
            (-self.radius, self.radius),
            (self.radius, self.radius),
        ];
        for (ox, oy) in offsets {
            let tx = (x + ox).floor() as i32;
            let ty = (y + oy).floor() as i32;
            if level.is_wall(tx, ty) {
                return true;
            }
        }
        false
    }
}
