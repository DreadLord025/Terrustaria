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
#[derive(Clone, Copy, Default, Debug)]
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
    /// 0 = none, 1..=4 = slope type.
    pub slope: u8,
    pub paint: u8,
}

impl Tile {
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

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LiquidKind {
    Water,
    Lava,
    Honey,
    Shimmer,
}
