//! One world tile, packed into 12 bytes (vs. the managed `Terraria.Tile` object).

pub const FLAG_ACTIVE: u8 = 1 << 0;
pub const FLAG_HALF_BRICK: u8 = 1 << 1;
pub const FLAG_ACTUATOR: u8 = 1 << 2;
pub const FLAG_INACTIVE: u8 = 1 << 3;
pub const FLAG_FULLBRIGHT: u8 = 1 << 4;
pub const FLAG_LIQUID_LAVA: u8 = 1 << 5;
pub const FLAG_LIQUID_HONEY: u8 = 1 << 6;
pub const FLAG_LIQUID_SHIMMER: u8 = 1 << 7;

/// Matches `Terraria.Tile` fields used for rendering.
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
#[repr(C)]
pub struct Tile {
    /// `tile.type` (TileID)
    pub kind: u16,
    /// `tile.wall` (WallID)
    pub wall: u16,
    /// Frame X in tileset pixels; `-1` means "must be computed".
    pub frame_x: i16,
    /// Frame Y in tileset pixels; `-1` means "must be computed".
    pub frame_y: i16,
    /// Liquid amount 0..=255.
    pub liquid: u8,
    pub flags: u8,
    /// Packed `slope | paint << 3` (slope 0..=4, paint 0..=31); E1 keeps the tile
    /// at 12 bytes without changing any save field. Use the accessors.
    pub meta: u8,
    /// Packed `wall_col | wall_row << 4` (col 0..=12, row 0..=4). Use the accessors.
    pub wall_frame: u8,
}

impl Tile {
    /// 0 = none, 1..=4 = slope type (`tile.slope()`).
    #[inline]
    pub fn slope(&self) -> u8 {
        self.meta & 0x7
    }

    #[inline]
    pub fn set_slope(&mut self, slope: u8) {
        self.meta = (self.meta & !0x7) | (slope & 0x7);
    }

    #[inline]
    pub fn paint(&self) -> u8 {
        self.meta >> 3
    }

    #[inline]
    pub fn set_paint(&mut self, paint: u8) {
        self.meta = (self.meta & 0x7) | ((paint & 0x1f) << 3);
    }

    /// Wall autotile cell (column) inside the wall sheet, in 36 px cells.
    #[inline]
    pub fn wall_col(&self) -> u8 {
        self.wall_frame & 0xf
    }

    /// Wall autotile cell (row) inside the wall sheet, in 36 px cells.
    #[inline]
    pub fn wall_row(&self) -> u8 {
        self.wall_frame >> 4
    }

    #[inline]
    pub fn set_wall_frame(&mut self, col: u8, row: u8) {
        self.wall_frame = (col & 0xf) | ((row & 0x7) << 4);
    }

    #[inline]
    pub fn active(&self) -> bool {
        self.flags & FLAG_ACTIVE != 0
    }

    #[inline]
    pub fn half_brick(&self) -> bool {
        self.flags & FLAG_HALF_BRICK != 0
    }

    #[inline]
    pub fn has_any_liquid(&self) -> bool {
        self.liquid > 0
    }

    #[inline]
    pub fn liquid_kind(&self) -> LiquidKind {
        if self.flags & FLAG_LIQUID_SHIMMER != 0 {
            LiquidKind::Shimmer
        } else if self.flags & FLAG_LIQUID_HONEY != 0 {
            LiquidKind::Honey
        } else if self.flags & FLAG_LIQUID_LAVA != 0 {
            LiquidKind::Lava
        } else {
            LiquidKind::Water
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packed_tile_is_twelve_bytes() {
        assert_eq!(std::mem::size_of::<Tile>(), 12);
        let mut t = Tile::default();
        t.set_slope(4);
        t.set_paint(31);
        t.set_wall_frame(12, 4);
        assert_eq!(t.slope(), 4);
        assert_eq!(t.paint(), 31);
        assert_eq!((t.wall_col(), t.wall_row()), (12, 4));
        assert_eq!(std::mem::size_of::<Tile>(), 12);
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LiquidKind {
    Water,
    Lava,
    Honey,
    Shimmer,
}
