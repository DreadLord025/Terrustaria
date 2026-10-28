//! In-memory world: tile grid + metadata.

use crate::tile::Tile;

#[derive(Clone, Debug)]
pub struct World {
    pub name: String,
    pub width: i32,
    pub height: i32,
    /// Row-major tile storage: `tiles[y * width + x]`.
    pub tiles: Vec<Tile>,
    /// Y of the first solid tile at the center of the map (approx. spawn altitude).
    pub spawn_y: i32,
}

impl World {
    pub fn new(name: String, width: i32, height: i32) -> Self {
        Self {
            name,
            width,
            height,
            tiles: vec![Tile::default(); (width as usize) * (height as usize)],
            spawn_y: height / 4,
        }
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
