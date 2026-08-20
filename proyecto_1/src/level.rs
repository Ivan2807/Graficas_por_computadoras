use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tile {
    Empty,
    Wall(u8),
    Door,
    /// Puerta cerrada (café). Bloquea igual que una pared hasta que termina
    /// de abrirse (ver DoorLink / door_progress).
    LockedDoor,
}

pub struct RoomTemplate {
    pub width: usize,
    pub height: usize,
    pub color_id: u8,
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
        let mut color_id = 8u8;
        let mut found_color = false;
        for (y, line) in lines.iter().enumerate() {
            for (x, ch) in line.chars().enumerate() {
                let tile = match ch {
                    '#' => Tile::Empty,
                    '.' => Tile::Door,
                    d if d.is_ascii_digit() && d != '0' => {
                        let id = d.to_digit(10).unwrap() as u8;
                        if !found_color {
                            color_id = id;
                            found_color = true;
                        }
                        Tile::Wall(id)
                    }
                    ' ' => Tile::Empty,
                    _ => Tile::Wall(8),
                };
                tiles[y * width + x] = tile;
            }
        }
        RoomTemplate { width, height, color_id, tiles }
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

/// Conecta dos cuadrantes vecinos. `tile_a`/`tile_b` son los dos tiles-puerta
/// (uno en cada lado del borde compartido) que forman ese pasaje.
#[derive(Clone, Copy, Debug)]
pub struct DoorLink {
    pub room_a: (usize, usize),
    pub room_b: (usize, usize),
    pub tile_a: (usize, usize),
    pub tile_b: (usize, usize),
}

pub struct Level {
    pub width: usize,
    pub height: usize,
    pub cell_w: usize,
    pub cell_h: usize,
    pub cols: usize,
    pub rows: usize,
    pub room_cells: Vec<(usize, usize)>,
    pub room_color: HashMap<(usize, usize), u8>,
    pub door_links: Vec<DoorLink>,
    /// Progreso de apertura de cada link (indice = mismo indice que
    /// door_links). 1.0 = totalmente abierta (tiles ya son Tile::Door).
    /// 0.0 = totalmente cerrada. Mientras esta entre 0 y 1, los tiles siguen
    /// siendo Tile::LockedDoor y el raycaster dibuja la animacion de "se hunde".
    door_progress: Vec<f32>,
    /// Si el link `i` esta actualmente en proceso de abrirse.
    door_opening: Vec<bool>,
    pub vault_cell: Option<(usize, usize)>,
    tiles: Vec<Tile>,
    pub explored: Vec<bool>,
}

impl Level {
    pub fn get(&self, x: i32, y: i32) -> Tile {
        if x < 0 || y < 0 || x as usize >= self.width || y as usize >= self.height {
            return Tile::Wall(8);
        }
        self.tiles[y as usize * self.width + x as usize]
    }

    pub fn set_tile(&mut self, x: usize, y: usize, tile: Tile) {
        if x < self.width && y < self.height {
            self.tiles[y * self.width + x] = tile;
        }
    }

    pub fn is_wall(&self, x: i32, y: i32) -> bool {
        matches!(self.get(x, y), Tile::Wall(_) | Tile::LockedDoor)
    }

    pub fn room_at(&self, x: f32, y: f32) -> Option<(usize, usize)> {
        let cx = (x / self.cell_w as f32).floor();
        let cy = (y / self.cell_h as f32).floor();
        if cx < 0.0 || cy < 0.0 {
            return None;
        }
        let (cx, cy) = (cx as usize, cy as usize);
        if cx < self.cols && cy < self.rows && self.room_color.contains_key(&(cx, cy)) {
            Some((cx, cy))
        } else {
            None
        }
    }

    /// Fuerza el link `idx` a estado cerrado (café) de una vez, sin
    /// animacion. Se usa SOLO al generar el nivel, para arrancar con la
    /// puerta ya sellada frente a un cuarto con monstruos.
    pub fn force_lock_link(&mut self, idx: usize) {
        if idx >= self.door_links.len() {
            return;
        }
        self.door_progress[idx] = 0.0;
        self.door_opening[idx] = false;
        let link = self.door_links[idx];
        self.set_tile(link.tile_a.0, link.tile_a.1, Tile::LockedDoor);
        self.set_tile(link.tile_b.0, link.tile_b.1, Tile::LockedDoor);
    }

    /// Empieza la animacion de apertura del link `idx` (si no esta ya
    /// abierta o abriendose)
    pub fn begin_open_link(&mut self, idx: usize) {
        if idx >= self.door_links.len() {
            return;
        }
        if self.door_progress[idx] >= 1.0 || self.door_opening[idx] {
            return;
        }
        self.door_opening[idx] = true;
    }

    /// Avanza la animacion de todas las puertas que estan abriendose.
    /// Llamar UNA vez por frame desde main.rs.
    pub fn update_doors(&mut self, dt: f32) {
        const DOOR_OPEN_TIME: f32 = 0.8; // segundos que tarda en abrir del todo
        for i in 0..self.door_links.len() {
            if !self.door_opening[i] {
                continue;
            }
            self.door_progress[i] += dt / DOOR_OPEN_TIME;
            if self.door_progress[i] >= 1.0 {
                self.door_progress[i] = 1.0;
                self.door_opening[i] = false;
                let link = self.door_links[i];
                self.set_tile(link.tile_a.0, link.tile_a.1, Tile::Door);
                self.set_tile(link.tile_b.0, link.tile_b.1, Tile::Door);
            }
        }
    }

    /// Progreso de apertura (0.0-1.0) del link que contiene el tile (x, y).
    /// Lo usa el raycaster para dibujar la animacion de "se hunde".
    pub fn door_progress_at(&self, x: usize, y: usize) -> f32 {
        for (i, link) in self.door_links.iter().enumerate() {
            if link.tile_a == (x, y) || link.tile_b == (x, y) {
                return self.door_progress[i];
            }
        }
        1.0
    }

    /// Indice del DoorLink que contiene el tile (x, y), si existe. Lo usa
    /// main.rs para saber a cual puerta dispararle o hacerle clic.
    pub fn find_door_link_at(&self, x: usize, y: usize) -> Option<usize> {
        self.door_links
            .iter()
            .position(|l| l.tile_a == (x, y) || l.tile_b == (x, y))
    }

    pub fn update_exploration(&mut self, px: f32, py: f32) {
        let tile_x = px.floor() as i32;
        let tile_y = py.floor() as i32;

        let cx = (px / self.cell_w as f32).floor() as i32;
        let cy = (py / self.cell_h as f32).floor() as i32;

        if cx >= 0 && cy >= 0 && (cx as usize) < self.cols && (cy as usize) < self.rows {
            let idx = cy as usize * self.cols + cx as usize;
            self.explored[idx] = true;

            if matches!(self.get(tile_x, tile_y), Tile::Door) {
                let neighbors = [(cx - 1, cy), (cx + 1, cy), (cx, cy - 1), (cx, cy + 1)];
                for (nx, ny) in neighbors {
                    if nx >= 0 && ny >= 0 && (nx as usize) < self.cols && (ny as usize) < self.rows {
                        let n_idx = ny as usize * self.cols + nx as usize;
                        self.explored[n_idx] = true;
                    }
                }
            }
        }
    }

    pub fn is_tile_explored(&self, x: usize, y: usize) -> bool {
        let cx = x / self.cell_w;
        let cy = y / self.cell_h;
        if cx < self.cols && cy < self.rows {
            self.explored[cy * self.cols + cx]
        } else {
            false
        }
    }

    pub fn is_room_explored(&self, cell: (usize, usize)) -> bool {
        if cell.0 < self.cols && cell.1 < self.rows {
            self.explored[cell.1 * self.cols + cell.0]
        } else {
            false
        }
    }
}

pub const RED_ROOM_ID: u8 = 1;
pub const PURPLE_ROOM_ID: u8 = 5;
pub const VAULT_ROOM_ID: u8 = 9;
pub const MAX_RED_ROOMS: usize = 2;

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

    let scan_order = || (0..rows).flat_map(|cy| (0..cols).map(move |cx| (cx, cy)));

    let spawn_cell = scan_order().find(|&(cx, cy)| present[cy * cols + cx]);

    let vault_cell = scan_order()
        .filter(|&(cx, cy)| present[cy * cols + cx])
        .filter(|&pos| Some(pos) != spawn_cell)
        .last()
        .or(spawn_cell);

    let width = cols * cell_w;
    let height = rows * cell_h;
    let mut tiles = vec![Tile::Wall(8); width * height];
    let mut room_cells = Vec::new();
    let mut room_color = HashMap::new();
    let mut red_count = 0usize;

    let vault_template = templates.iter().find(|t| t.color_id == VAULT_ROOM_ID);

    for cy in 0..rows {
        for cx in 0..cols {
            if !present[cy * cols + cx] {
                continue;
            }
            room_cells.push((cx, cy));
            let is_spawn = spawn_cell == Some((cx, cy));
            let is_vault = vault_cell == Some((cx, cy));

            let template = if is_vault && vault_template.is_some() {
                vault_template.unwrap()
            } else {
                let mut allowed: Vec<usize> = (0..templates.len())
                    .filter(|&i| {
                        let cid = templates[i].color_id;
                        if cid == VAULT_ROOM_ID {
                            return false;
                        }
                        if is_spawn && cid == PURPLE_ROOM_ID {
                            return false;
                        }
                        if cid == RED_ROOM_ID && red_count >= MAX_RED_ROOMS {
                            return false;
                        }
                        true
                    })
                    .collect();
                if allowed.is_empty() {
                    allowed = (0..templates.len()).collect();
                }
                let pick = rand_range(0, allowed.len() as i32) as usize;
                &templates[allowed[pick.min(allowed.len() - 1)]]
            };

            if template.color_id == RED_ROOM_ID {
                red_count += 1;
            }
            room_color.insert((cx, cy), template.color_id);

            for ty in 0..cell_h {
                for tx in 0..cell_w {
                    let world_x = cx * cell_w + tx;
                    let world_y = cy * cell_h + ty;
                    tiles[world_y * width + world_x] = template.get(tx, ty);
                }
            }
        }
    }

    let mut door_links = Vec::new();
    for &(cx, cy) in &room_cells {
        if cx + 1 < cols && present[cy * cols + (cx + 1)] {
            let (ta, tb) = carve_door(&mut tiles, width, cell_w, cell_h, cx, cy, cx + 1, cy);
            door_links.push(DoorLink { room_a: (cx, cy), room_b: (cx + 1, cy), tile_a: ta, tile_b: tb });
        }
        if cy + 1 < rows && present[(cy + 1) * cols + cx] {
            let (ta, tb) = carve_door(&mut tiles, width, cell_w, cell_h, cx, cy, cx, cy + 1);
            door_links.push(DoorLink { room_a: (cx, cy), room_b: (cx, cy + 1), tile_a: ta, tile_b: tb });
        }
    }

    let door_progress = vec![1.0f32; door_links.len()];
    let door_opening = vec![false; door_links.len()];
    let explored = vec![false; cols * rows];

    Level {
        width,
        height,
        cell_w,
        cell_h,
        cols,
        rows,
        room_cells,
        room_color,
        door_links,
        door_progress,
        door_opening,
        vault_cell,
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
) -> ((usize, usize), (usize, usize)) {
    if ax == bx {
        let (top, bottom) = if ay < by { (ay, by) } else { (by, ay) };
        let door_x = ax * cell_w + cell_w / 2;
        let border_bottom_of_top = top * cell_h + cell_h - 1;
        let border_top_of_bottom = bottom * cell_h;
        tiles[border_bottom_of_top * width + door_x] = Tile::Door;
        tiles[border_top_of_bottom * width + door_x] = Tile::Door;
        ((door_x, border_bottom_of_top), (door_x, border_top_of_bottom))
    } else {
        let (left, right) = if ax < bx { (ax, bx) } else { (bx, ax) };
        let door_y = ay * cell_h + cell_h / 2;
        let border_right_of_left = left * cell_w + cell_w - 1;
        let border_left_of_right = right * cell_w;
        tiles[door_y * width + border_right_of_left] = Tile::Door;
        tiles[door_y * width + border_left_of_right] = Tile::Door;
        ((border_right_of_left, door_y), (border_left_of_right, door_y))
    }
}