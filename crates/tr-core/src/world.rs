//! In-memory world: tile grid + metadata.

use crate::tile::Tile;

#[derive(Clone, Debug)]
pub struct World {
    pub name: String,
    pub width: i32,
    pub height: i32,
    /// Row-major tile storage: `tiles[y * width + x]`.
    pub tiles: Vec<Tile>,
    /// `Main.tileFrameImportant`: tile types whose frames are stored in the save file.
    pub frame_important: Vec<bool>,
    /// Y of the first solid tile at the center of the map (approx. spawn altitude).
    pub spawn_y: i32,
    /// Spawn point from the save header (tile coordinates).
    pub spawn_x: i32,
    /// `Main.worldSurface`: below this row the sun does not reach the surface.
    pub world_surface: f64,
    /// `Main.rockLayer`: top of the underground rock layer.
    pub rock_layer: f64,
    /// `Main.time`: world time in ticks (0..=54000).
    pub time: f64,
    /// `Main.dayTime`: whether the saved world is in daylight.
    pub day_time: bool,
    /// Y of the first solid tile in each column (background/sky horizon).
    pub column_surface: Vec<i32>,
}

impl World {
    pub fn new(name: String, width: i32, height: i32) -> Self {
        Self {
            name,
            width,
            height,
            tiles: vec![Tile::default(); (width as usize) * (height as usize)],
            frame_important: Vec::new(),
            spawn_y: height / 4,
            spawn_x: width / 2,
            world_surface: height as f64 * 0.25,
            rock_layer: height as f64 * 0.45,
            time: 0.0,
            day_time: true,
            column_surface: Vec::new(),
        }
    }

    /// Finds the first solid tile per column, used to place the sky horizon and the
    /// underground background. Trees and plants are not solid, so they do not count.
    pub fn compute_column_surface(&mut self) {
        use crate::tiledata as td;
        self.column_surface.resize(self.width as usize, self.height);
        for x in 0..self.width {
            let mut top = self.height;
            for y in 0..self.height {
                let t = &self.tiles[self.index(x, y)];
                if t.active()
                    && (t.kind as usize) < td::TILE_COUNT
                    && td::TILE_SOLID.has(t.kind)
                {
                    top = y;
                    break;
                }
            }
            self.column_surface[x as usize] = top;
        }
    }

    /// Surface row of a column, falling back to the map bottom outside the world.
    #[inline]
    pub fn surface_of(&self, x: i32) -> i32 {
        if x < 0 || x >= self.width {
            return self.height;
        }
        self.column_surface
            .get(x as usize)
            .copied()
            .unwrap_or(self.height)
    }

    #[inline]
    pub fn index(&self, x: i32, y: i32) -> usize {
        (y as usize) * (self.width as usize) + (x as usize)
    }

    #[inline]
    pub fn in_bounds(&self, x: i32, y: i32) -> bool {
        x >= 0 && y >= 0 && x < self.width && y < self.height
    }

    #[inline]
    pub fn tile(&self, x: i32, y: i32) -> Option<&Tile> {
        if self.in_bounds(x, y) { Some(&self.tiles[self.index(x, y)]) } else { None }
    }

    #[inline]
    pub fn tile_mut(&mut self, x: i32, y: i32) -> Option<&mut Tile> {
        if self.in_bounds(x, y) {
            let i = self.index(x, y);
            Some(&mut self.tiles[i])
        } else {
            None
        }
    }

    #[inline]
    pub fn is_active(&self, x: i32, y: i32) -> bool {
        self.tile(x, y).is_some_and(|t| t.active())
    }

    /// Approximate surface (first active tile scanning down the middle column).
    pub fn compute_spawn_y(&mut self) {
        let x = self.width / 2;
        for y in 0..self.height {
            if self.is_active(x, y) {
                self.spawn_y = (y - 6).max(0);
                return;
            }
        }
    }

    pub fn memory_bytes(&self) -> usize {
        self.tiles.len() * std::mem::size_of::<Tile>()
    }
}
