use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub type LevelMap = Level;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tile {
    Empty,
    Wall(u8),
    Door,
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

    pub fn from_str(s: &str) -> Self {
        let lines: Vec<&str> = s.lines().filter(|l| !l.trim().is_empty()).collect();
        let height = lines.len();
        let width = lines.iter().map(|l| l.chars().count()).max().unwrap_or(0);
        let mut tiles = vec![Tile::Empty; width * height];
        let mut color_id = 1u8;
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
                    _ => Tile::Wall(1),
                };
                tiles[y * width + x] = tile;
            }
        }
        RoomTemplate { width, height, color_id, tiles }
    }
}

pub fn load_room_templates<P: AsRef<Path>>(dir: P) -> Vec<RoomTemplate> {
    let mut entries: Vec<_> = fs::read_dir(dir)
        .expect("No se pudo leer el directorio assets/rooms")
        .filter_map(|e| e.ok())
        .collect();
    entries.sort_by_key(|e| e.path());

    let mut templates = Vec::new();
    for entry in entries {
        let path = entry.path();
        if path.extension().map(|e| e == "txt").unwrap_or(false)
            && path.file_name().map(|name| name != "room_final.txt").unwrap_or(false)
        {
            let content = fs::read_to_string(&path)
                .unwrap_or_else(|_| panic!("Error al leer {:?}", path));
            templates.push(RoomTemplate::from_str(&content));
        }
    }
    templates
}

pub fn load_final_room<P: AsRef<Path>>(path: P) -> RoomTemplate {
    let content = fs::read_to_string(path).expect("No se pudo leer assets/rooms/room_final.txt");
    RoomTemplate::from_str(&content)
}

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
    pub door_progress: Vec<f32>,
    pub door_opening: Vec<bool>,
    pub vault_cell: Option<(usize, usize)>,
    pub tiles: Vec<Tile>,
    pub explored: Vec<bool>,
}

impl Level {
    pub fn new() -> Self {
        let cell_w = 16;
        let cell_h = 16;
        let cols = 4;
        let rows = 4;
        let width = cols * cell_w;
        let height = rows * cell_h;
        let mut tiles = vec![Tile::Empty; width * height];

        for x in 0..width {
            tiles[x] = Tile::Wall(1);
            tiles[(height - 1) * width + x] = Tile::Wall(1);
        }
        for y in 0..height {
            tiles[y * width] = Tile::Wall(1);
            tiles[y * width + (width - 1)] = Tile::Wall(1);
        }

        Level {
            width,
            height,
            cell_w,
            cell_h,
            cols,
            rows,
            room_cells: vec![(0, 0)],
            room_color: HashMap::new(),
            door_links: Vec::new(),
            door_progress: Vec::new(),
            door_opening: Vec::new(),
            vault_cell: None,
            tiles,
            explored: vec![true; width * height],
        }
    }

    pub fn get(&self, x: usize, y: usize) -> Tile {
        if x >= self.width || y >= self.height {
            return Tile::Wall(1);
        }
        self.tiles[y * self.width + x]
    }

    pub fn get_tile(&self, x: i32, y: i32) -> Tile {
        if x < 0 || y < 0 || x as usize >= self.width || y as usize >= self.height {
            return Tile::Wall(1);
        }
        self.tiles[y as usize * self.width + x as usize]
    }

    pub fn is_wall(&self, x: i32, y: i32) -> bool {
        matches!(self.get_tile(x, y), Tile::Wall(_) | Tile::LockedDoor)
    }

    pub fn is_wall_or_locked(&self, x: f32, y: f32) -> bool {
        if x < 0.0 || y < 0.0 {
            return true;
        }
        self.is_wall(x.floor() as i32, y.floor() as i32)
    }

    pub fn room_at(&self, x: f32, y: f32) -> Option<(usize, usize)> {
        let cell_x = (x / self.cell_w as f32).floor() as i32;
        let cell_y = (y / self.cell_h as f32).floor() as i32;
        if cell_x < 0 || cell_y < 0 {
            return None;
        }
        let cell = (cell_x as usize, cell_y as usize);
        self.room_cells.contains(&cell).then_some(cell)
    }

    pub fn is_same_room(&self, first: (f32, f32), second: (f32, f32)) -> bool {
        self.room_at(first.0, first.1).is_some()
            && self.room_at(first.0, first.1) == self.room_at(second.0, second.1)
    }

    pub fn rooms_are_open(&self, first: (usize, usize), second: (usize, usize)) -> bool {
        if first == second {
            return true;
        }

        let mut visited = vec![false; self.cols * self.rows];
        let mut pending = vec![first];
        visited[first.1 * self.cols + first.0] = true;

        while let Some(room) = pending.pop() {
            for (index, link) in self.door_links.iter().enumerate() {
                if self.door_progress[index] < 1.0 {
                    continue;
                }
                let next = if link.room_a == room {
                    Some(link.room_b)
                } else if link.room_b == room {
                    Some(link.room_a)
                } else {
                    None
                };
                let Some(next) = next else { continue; };
                let next_index = next.1 * self.cols + next.0;
                if next == second {
                    return true;
                }
                if !visited[next_index] {
                    visited[next_index] = true;
                    pending.push(next);
                }
            }
        }
        false
    }

    pub fn rooms_are_open_at(&self, first: (f32, f32), second: (f32, f32)) -> bool {
        match (self.room_at(first.0, first.1), self.room_at(second.0, second.1)) {
            (Some(first_room), Some(second_room)) => self.rooms_are_open(first_room, second_room),
            _ => false,
        }
    }

    pub fn is_tile_explored(&self, x: usize, y: usize) -> bool {
        self.room_at(x as f32 + 0.5, y as f32 + 0.5)
            .map(|(cx, cy)| self.explored[cy * self.cols + cx])
            .unwrap_or(false)
    }

    pub fn is_room_explored(&self, room: (usize, usize)) -> bool {
        room.0 < self.cols && room.1 < self.rows && self.explored[room.1 * self.cols + room.0]
    }

    pub fn toggle_door(&mut self, tile_x: usize, tile_y: usize) -> bool {
        let Some(index) = self.door_links.iter().position(|link| {
            link.tile_a == (tile_x, tile_y) || link.tile_b == (tile_x, tile_y)
        }) else {
            return false;
        };

        if self.door_progress[index] >= 1.0 && !self.door_opening[index] {
            self.close_door(index);
        } else {
            self.door_opening[index] = true;
        }
        true
    }

    pub fn force_open_door(&mut self, tile_x: usize, tile_y: usize) -> bool {
        let Some(index) = self.door_links.iter().position(|link| {
            link.tile_a == (tile_x, tile_y) || link.tile_b == (tile_x, tile_y)
        }) else {
            return false;
        };
        self.door_progress[index] = 1.0;
        self.door_opening[index] = false;
        self.set_door_tiles(index, Tile::Door);
        true
    }

    fn close_door(&mut self, index: usize) {
        self.door_progress[index] = 0.0;
        self.door_opening[index] = false;
        self.set_door_tiles(index, Tile::LockedDoor);
    }

    fn set_door_tiles(&mut self, index: usize, tile: Tile) {
        let link = self.door_links[index];
        self.tiles[link.tile_a.1 * self.width + link.tile_a.0] = tile;
        self.tiles[link.tile_b.1 * self.width + link.tile_b.0] = tile;
    }

    pub fn door_progress_at(&self, tile_x: usize, tile_y: usize) -> f32 {
        self.door_links
            .iter()
            .position(|link| link.tile_a == (tile_x, tile_y) || link.tile_b == (tile_x, tile_y))
            .map(|index| self.door_progress[index])
            .unwrap_or(0.0)
    }

    pub fn update_doors(&mut self, dt: f32) {
        const DOOR_OPEN_TIME: f32 = 0.8;
        for i in 0..self.door_links.len() {
            if !self.door_opening[i] {
                continue;
            }
            self.door_progress[i] += dt / DOOR_OPEN_TIME;
            if self.door_progress[i] >= 1.0 {
                self.door_progress[i] = 1.0;
                self.door_opening[i] = false;
                self.set_door_tiles(i, Tile::Door);
            }
        }
    }

    pub fn update_exploration(&mut self, px: f32, py: f32) {
        let cx = (px / self.cell_w as f32).floor() as i32;
        let cy = (py / self.cell_h as f32).floor() as i32;

        if cx >= 0 && cy >= 0 && (cx as usize) < self.cols && (cy as usize) < self.rows {
            let idx = cy as usize * self.cols + cx as usize;
            self.explored[idx] = true;
        }
    }

    pub fn replace_room(&mut self, room: (usize, usize), template: &RoomTemplate) {
        if template.width != self.cell_w || template.height != self.cell_h {
            return;
        }
        for y in 0..self.cell_h {
            for x in 0..self.cell_w {
                let world_x = room.0 * self.cell_w + x;
                let world_y = room.1 * self.cell_h + y;
                self.tiles[world_y * self.width + world_x] = template.get(x, y);
            }
        }
        self.room_color.insert(room, template.color_id);
        self.restore_doors();
    }

    pub fn restore_doors(&mut self) {
        for index in 0..self.door_links.len() {
            let tile = if self.door_progress[index] >= 1.0 {
                Tile::Door
            } else {
                Tile::LockedDoor
            };
            self.set_door_tiles(index, tile);
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
    assert!(!templates.is_empty(), "No hay plantillas de rooms disponibles");
    let cell_w = templates[0].width;
    let cell_h = templates[0].height;
    let total_cells = cols * rows;

    let target_rooms = ((total_cells as i32 * fill_chance_percent) / 100)
        .max(min_rooms as i32)
        .clamp(4, total_cells as i32) as usize;
    let mut present = vec![false; total_cells];
    present[0] = true;
    let corners = [
        (0usize, 0usize),
        (cols - 1, 0),
        (0, rows - 1),
        (cols - 1, rows - 1),
    ];

    // Crecimiento desde (0,0): cada celda nueva toca una ya existente,
    // por lo que todas las salas quedan en un solo componente conectado.
    while present.iter().filter(|&&room| room).count() < target_rooms
        || corners.iter().any(|&(x, y)| !present[y * cols + x])
    {
        let mut candidates = Vec::new();
        for y in 0..rows {
            for x in 0..cols {
                if present[y * cols + x] {
                    continue;
                }
                let adjacent = (x > 0 && present[y * cols + x - 1])
                    || (x + 1 < cols && present[y * cols + x + 1])
                    || (y > 0 && present[(y - 1) * cols + x])
                    || (y + 1 < rows && present[(y + 1) * cols + x]);
                if adjacent {
                    candidates.push((x, y));
                }
            }
        }
        if candidates.is_empty() {
            break;
        }
        let index = rand_range(0, candidates.len() as i32 - 1) as usize;
        let (x, y) = candidates[index];
        present[y * cols + x] = true;
    }

    let width = cols * cell_w;
    let height = rows * cell_h;
    let mut tiles = vec![Tile::Wall(1); width * height];
    let mut room_cells = Vec::new();
    let mut room_color = HashMap::new();

    for cy in 0..rows {
        for cx in 0..cols {
            if !present[cy * cols + cx] {
                continue;
            }
            room_cells.push((cx, cy));
            let mut template_index = rand_range(0, templates.len() as i32 - 1) as usize;
            // La sala inicial nunca usa room_05 (indice 4 en el orden de carga).
            if room_cells.len() == 1 && templates.len() > 4 && template_index == 4 {
                template_index = 0;
            }
            let template = &templates[template_index];
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
    let mut door_progress = Vec::new();
    let mut door_opening = Vec::new();

    for cy in 0..rows {
        for cx in 0..cols {
            if !present[cy * cols + cx] {
                continue;
            }
            if cx + 1 < cols && present[cy * cols + cx + 1] {
                let left = (cx * cell_w + cell_w - 1, cy * cell_h + cell_h / 2);
                let right = ((cx + 1) * cell_w, cy * cell_h + cell_h / 2);
                tiles[left.1 * width + left.0] = Tile::LockedDoor;
                tiles[right.1 * width + right.0] = Tile::LockedDoor;
                door_links.push(DoorLink {
                    room_a: (cx, cy),
                    room_b: (cx + 1, cy),
                    tile_a: left,
                    tile_b: right,
                });
                door_progress.push(0.0);
                door_opening.push(false);
            }
            if cy + 1 < rows && present[(cy + 1) * cols + cx] {
                let top = (cx * cell_w + cell_w / 2, cy * cell_h + cell_h - 1);
                let bottom = (cx * cell_w + cell_w / 2, (cy + 1) * cell_h);
                tiles[top.1 * width + top.0] = Tile::LockedDoor;
                tiles[bottom.1 * width + bottom.0] = Tile::LockedDoor;
                door_links.push(DoorLink {
                    room_a: (cx, cy),
                    room_b: (cx, cy + 1),
                    tile_a: top,
                    tile_b: bottom,
                });
                door_progress.push(0.0);
                door_opening.push(false);
            }
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
        room_color,
        door_links,
        door_progress,
        door_opening,
        vault_cell: None,
        tiles,
        explored,
    }
}