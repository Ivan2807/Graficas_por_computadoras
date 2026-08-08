use std::fs;
use std::path::Path;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tile {
    Empty,
    Wall(u8),
    Door,
}

pub struct RoomTemplate {
    pub width: usize,
    pub height: usize,
    tiles: Vec<Tile>,
}

impl RoomTemplate {
    pub fn get(&self, x: usize, y: usize) -> Tile {
        self.tiles[y * self.width + x]
    }

    fn from_str(s: &str) -> Self {
        let lines: Vec<&str> = s.lines().filter(|l| !l.trim().is_empty()).collect();
        let height = lines.len();
        let width = lines.iter().map(|l| l.chars().count()).max().unwrap_or(0);
        let mut tiles = vec![Tile::Empty; width * height];
        for (y, line) in lines.iter().enumerate() {
            for (x, ch) in line.chars().enumerate() {
                let tile = match ch {
                    '#' => Tile::Empty,
                    '.' => Tile::Door,
                    d if d.is_ascii_digit() && d != '0' => {
                        Tile::Wall(d.to_digit(10).unwrap() as u8)
                    }
                    ' ' => Tile::Empty,
                    _ => Tile::Wall(8),
                };
                tiles[y * width + x] = tile;
            }
        }
        RoomTemplate { width, height, tiles }
    }
}

pub fn load_room_templates<P: AsRef<Path>>(dir: P) -> Vec<RoomTemplate> {
    let mut entries: Vec<_> = fs::read_dir(dir)
        .expect("no se pudo leer el directorio de habitaciones (assets/rooms)")
        .filter_map(|e| e.ok())
        .collect();
    entries.sort_by_key(|e| e.path());

    let mut templates = Vec::new();
    for entry in entries {
        let path = entry.path();
        if path.extension().map(|e| e == "txt").unwrap_or(false) {
            let content = fs::read_to_string(&path)
                .unwrap_or_else(|_| panic!("no se pudo leer {:?}", path));
            templates.push(RoomTemplate::from_str(&content));
        }
    }
    templates
}

pub struct Level {
    pub width: usize,
    pub height: usize,
    pub cell_w: usize,
    pub cell_h: usize,
    pub cols: usize,
    pub rows: usize,
    pub room_cells: Vec<(usize, usize)>,
    tiles: Vec<Tile>,
    /// Matriz que indica si el cuadrante (cx, cy) ya fue revelado/explorado.
    pub explored: Vec<bool>,
}

impl Level {
    pub fn get(&self, x: i32, y: i32) -> Tile {
        if x < 0 || y < 0 || x as usize >= self.width || y as usize >= self.height {
            return Tile::Wall(8);
        }
        self.tiles[y as usize * self.width + x as usize]
    }

    pub fn is_wall(&self, x: i32, y: i32) -> bool {
        matches!(self.get(x, y), Tile::Wall(_))
    }

    /// Revisa en qué cuadrante está la posición (px, py) y marca ese cuadrante
    /// (y sus vecinos inmediatos si está en una puerta) como explorado.
    pub fn update_exploration(&mut self, px: f32, py: f32) {
        let tile_x = px.floor() as i32;
        let tile_y = py.floor() as i32;

        let cx = (px / self.cell_w as f32).floor() as i32;
        let cy = (py / self.cell_h as f32).floor() as i32;

        if cx >= 0 && cy >= 0 && (cx as usize) < self.cols && (cy as usize) < self.rows {
            let idx = cy as usize * self.cols + cx as usize;
            self.explored[idx] = true;

            // Si el jugador está parado sobre una puerta (o adyacente a ella),
            // se revelan también los cuadrantes adyacentes a la puerta.
            if matches!(self.get(tile_x, tile_y), Tile::Door) {
                let neighbors = [
                    (cx - 1, cy),
                    (cx + 1, cy),
                    (cx, cy - 1),
                    (cx, cy + 1),
                ];
                for (nx, ny) in neighbors {
                    if nx >= 0 && ny >= 0 && (nx as usize) < self.cols && (ny as usize) < self.rows {
                        let n_idx = ny as usize * self.cols + nx as usize;
                        self.explored[n_idx] = true;
                    }
                }
            }
        }
    }

    /// Verifica si la celda de mapa global (x, y) pertenece a un cuadrante explorado.
    pub fn is_tile_explored(&self, x: usize, y: usize) -> bool {
        let cx = x / self.cell_w;
        let cy = y / self.cell_h;
        if cx < self.cols && cy < self.rows {
            self.explored[cy * self.cols + cx]
        } else {
            false
        }
    }
}

pub fn generate_level(
    templates: &[RoomTemplate],
    cols: usize,
    rows: usize,
    min_rooms: usize,
    fill_chance_percent: i32,
    mut rand_range: impl FnMut(i32, i32) -> i32,
) -> Level {
    assert!(!templates.is_empty(), "se necesita al menos una plantilla de cuarto");
    let cell_w = templates[0].width;
    let cell_h = templates[0].height;
    for t in templates {
        assert_eq!(t.width, cell_w, "todas las plantillas deben medir lo mismo");
        assert_eq!(t.height, cell_h, "todas las plantillas deben medir lo mismo");
    }

    let total_cells = cols * rows;
    let min_rooms = min_rooms.min(total_cells).max(1);

    let mut present = vec![false; total_cells];
    let mut found_valid = false;
    const MAX_ATTEMPTS: usize = 500;

    for _ in 0..MAX_ATTEMPTS {
        for cell in present.iter_mut() {
            *cell = rand_range(0, 100) < fill_chance_percent;
        }
        let count = present.iter().filter(|&&p| p).count();
        if count < min_rooms {
            continue;
        }
        if is_fully_connected(&present, cols, rows) {
            found_valid = true;
            break;
        }
    }

    if !found_valid {
        present.iter_mut().for_each(|p| *p = true);
    }

    let width = cols * cell_w;
    let height = rows * cell_h;
    let mut tiles = vec![Tile::Wall(8); width * height];
    let mut room_cells = Vec::new();

    for cy in 0..rows {
        for cx in 0..cols {
            if !present[cy * cols + cx] {
                continue;
            }
            room_cells.push((cx, cy));

            let raw_idx = rand_range(0, templates.len() as i32) as usize;
            let t_idx = raw_idx.min(templates.len() - 1);
            let template = &templates[t_idx];
            for ty in 0..cell_h {
                for tx in 0..cell_w {
                    let world_x = cx * cell_w + tx;
                    let world_y = cy * cell_h + ty;
                    tiles[world_y * width + world_x] = template.get(tx, ty);
                }
            }
        }
    }

    for &(cx, cy) in &room_cells {
        if cx + 1 < cols && present[cy * cols + (cx + 1)] {
            carve_door(&mut tiles, width, cell_w, cell_h, cx, cy, cx + 1, cy);
        }
        if cy + 1 < rows && present[(cy + 1) * cols + cx] {
            carve_door(&mut tiles, width, cell_w, cell_h, cx, cy, cx, cy + 1);
        }
    }

    let explored = vec![false; cols * rows];

    Level {
        width,
        height,
        cell_w,
        cell_h,
        cols,
        rows,
        room_cells,
        tiles,
        explored,
    }
}

fn is_fully_connected(present: &[bool], cols: usize, rows: usize) -> bool {
    let total_present = present.iter().filter(|&&p| p).count();
    if total_present == 0 {
        return false;
    }
    let start = present.iter().position(|&p| p).unwrap();

    let mut visited = vec![false; present.len()];
    let mut stack = vec![start];
    visited[start] = true;
    let mut visited_count = 1;

    while let Some(idx) = stack.pop() {
        let cx = (idx % cols) as i32;
        let cy = (idx / cols) as i32;
        let neighbors = [(cx - 1, cy), (cx + 1, cy), (cx, cy - 1), (cx, cy + 1)];
        for (nx, ny) in neighbors {
            if nx < 0 || ny < 0 || nx as usize >= cols || ny as usize >= rows {
                continue;
            }
            let nidx = ny as usize * cols + nx as usize;
            if present[nidx] && !visited[nidx] {
                visited[nidx] = true;
                visited_count += 1;
                stack.push(nidx);
            }
        }
    }

    visited_count == total_present
}

fn carve_door(
    tiles: &mut [Tile],
    width: usize,
    cell_w: usize,
    cell_h: usize,
    ax: usize,
    ay: usize,
    bx: usize,
    by: usize,
) {
    if ax == bx {
        let (top, bottom) = if ay < by { (ay, by) } else { (by, ay) };
        let door_x = ax * cell_w + cell_w / 2;
        let border_bottom_of_top = top * cell_h + cell_h - 1;
        let border_top_of_bottom = bottom * cell_h;
        tiles[border_bottom_of_top * width + door_x] = Tile::Door;
        tiles[border_top_of_bottom * width + door_x] = Tile::Door;
    } else {
        let (left, right) = if ax < bx { (ax, bx) } else { (bx, ax) };
        let door_y = ay * cell_h + cell_h / 2;
        let border_right_of_left = left * cell_w + cell_w - 1;
        let border_left_of_right = right * cell_w;
        tiles[door_y * width + border_right_of_left] = Tile::Door;
        tiles[door_y * width + border_left_of_right] = Tile::Door;
    }
}