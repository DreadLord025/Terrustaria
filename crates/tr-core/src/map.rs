//! Packed map colour lookup and biome gradients.
//!
//! Ports `MapHelper.Initialize` (`decompiled/Terraria/Map/MapHelper.cs:1683-1788`):
//! per-tile colours first, then per-wall, then the four liquid colours, the
//! 256-entry sky/dirt/rock gradients and finally the single hell colour.

use std::sync::OnceLock;

use crate::mapdata::{TILE_COLORS, TILE_COUNT, WALL_COLORS, WALL_COUNT};

/// The four liquid colours (`array3`, `MapHelper.cs:1272-1278`).
pub const LIQUID_COLORS: [[u8; 3]; 4] = [
    [9, 61, 191],
    [253, 32, 3],
    [254, 194, 20],
    [161, 127, 255],
];

/// One channel of a gradient: `(byte)(c0 * (1 - v) + c1 * v)`.
///
/// C# truncates a `float` to `byte` (`MapHelper.cs:1669`), which is what `as u8`
/// does in Rust; the operand order matches the source exactly.
fn gradient_channel(c0: u8, c1: u8, v: f32) -> u8 {
    (c0 as f32 * (1.0 - v) + c1 as f32 * v) as u8
}

fn gradient(c0: [u8; 3], c1: [u8; 3], i: usize, denominator: f32) -> [u8; 3] {
    let v = i as f32 / denominator;
    [
        gradient_channel(c0[0], c1[0], v),
        gradient_channel(c0[1], c1[1], v),
        gradient_channel(c0[2], c1[2], v),
    ]
}

/// `array5` (`MapHelper.cs:1662-1670`): sky gradient, `v = i / 256`.
pub fn sky_gradient(i: usize) -> [u8; 3] {
    gradient([50, 40, 255], [145, 185, 255], i, 256.0)
}

/// `array6` (`MapHelper.cs:1671-1679`): dirt gradient, `v = i / 255`.
pub fn dirt_gradient(i: usize) -> [u8; 3] {
    gradient([88, 61, 46], [37, 78, 123], i, 255.0)
}

/// `array7` (`MapHelper.cs:1680-1688`): rock gradient, `v = i / 255`.
pub fn rock_gradient(i: usize) -> [u8; 3] {
    gradient([74, 67, 60], [53, 70, 97], i, 255.0)
}

/// The packed `colorLookup` table plus the offsets the map renderer indexes it with.
pub struct MapColorTable {
    /// C# `colorLookup`; index 0 is `[0, 0, 0]` (`Color.Transparent`).
    pub lookup: Vec<[u8; 3]>,
    /// Per `TileID` -> first index in `lookup` (0 = no options).
    pub tile_lookup: Vec<u16>,
    /// Per `WallID` -> first index in `lookup` (0 = no options).
    pub wall_lookup: Vec<u16>,
    /// Option count per `TileID` (0..=13).
    pub tile_option_counts: Vec<u8>,
    /// Option count per `WallID` (0..=2).
    pub wall_option_counts: Vec<u8>,
    pub tile_position: u16,
    pub wall_position: u16,
    pub wall_range_start: u16,
    pub wall_range_end: u16,
    pub liquid_position: u16,
    pub sky_position: u16,
    pub dirt_position: u16,
    pub rock_position: u16,
    pub hell_position: u16,
    pub snow_types: [u16; 6],
}

impl MapColorTable {
    pub fn new() -> Self {
        // Option counts (`MapHelper.cs:1691-1712`). `TILE_COLORS` rows are
        // `TileID.Count x 13` and `WALL_COLORS` rows `WallID.Count x 2`, so the
        // source's `< 13` / `< 2` loop bounds are implicit in the row lengths.
        let mut tile_option_counts = Vec::with_capacity(TILE_COUNT);
        let mut total = 0usize;
        for row in TILE_COLORS.iter() {
            let count = row.iter().take_while(|c| c[3] != 0).count();
            tile_option_counts.push(count as u8);
            total += count;
        }
        let mut wall_option_counts = Vec::with_capacity(WALL_COUNT);
        for row in WALL_COLORS.iter() {
            let count = row.iter().take_while(|c| c[3] != 0).count();
            wall_option_counts.push(count as u8);
            total += count;
        }

        // `colorLookup = new Color[num16 + 774]` with `colorLookup[0] = Transparent`.
        let mut lookup = vec![[0u8; 3]; total + 774];

        let tile_position: u16 = 1;
        let mut pos = 1usize;
        let mut tile_lookup = vec![0u16; TILE_COUNT];
        for (ty, row) in TILE_COLORS.iter().enumerate() {
            let count = tile_option_counts[ty] as usize;
            if count > 0 {
                tile_lookup[ty] = pos as u16;
                for c in row.iter().take(count) {
                    lookup[pos] = [c[0], c[1], c[2]];
                    pos += 1;
                }
            }
        }

        let wall_position = pos as u16;
        let wall_range_start = pos as u16;
        let mut wall_lookup = vec![0u16; WALL_COUNT];
        for (ty, row) in WALL_COLORS.iter().enumerate() {
            let count = wall_option_counts[ty] as usize;
            if count > 0 {
                wall_lookup[ty] = pos as u16;
                for c in row.iter().take(count) {
                    lookup[pos] = [c[0], c[1], c[2]];
                    pos += 1;
                }
            }
        }
        let wall_range_end = pos as u16;

        let liquid_position = pos as u16;
        for color in LIQUID_COLORS {
            lookup[pos] = color;
            pos += 1;
        }

        let sky_position = pos as u16;
        for i in 0..256 {
            lookup[pos] = sky_gradient(i);
            pos += 1;
        }
        let dirt_position = pos as u16;
        for i in 0..256 {
            lookup[pos] = dirt_gradient(i);
            pos += 1;
        }
        let rock_position = pos as u16;
        for i in 0..256 {
            lookup[pos] = rock_gradient(i);
            pos += 1;
        }

        let hell_position = pos as u16;
        lookup[pos] = [50, 44, 38]; // `color11`, MapHelper.cs:1689.
        pos += 1;
        debug_assert_eq!(pos, lookup.len());

        let snow_types = [
            tile_lookup[147],
            tile_lookup[161],
            tile_lookup[162],
            tile_lookup[163],
            tile_lookup[164],
            tile_lookup[200],
        ];

        Self {
            lookup,
            tile_lookup,
            wall_lookup,
            tile_option_counts,
            wall_option_counts,
            tile_position,
            wall_position,
            wall_range_start,
            wall_range_end,
            liquid_position,
            sky_position,
            dirt_position,
            rock_position,
            hell_position,
            snow_types,
        }
    }

    /// Colour at packed index `ty`; out of range yields `[0, 0, 0]`.
    pub fn color(&self, ty: u16) -> [u8; 3] {
        self.lookup.get(ty as usize).copied().unwrap_or([0, 0, 0])
    }
}

impl Default for MapColorTable {
    fn default() -> Self {
        Self::new()
    }
}

/// The process-wide packed table (`MapHelper.colorLookup`).
pub fn color_table() -> &'static MapColorTable {
    static TABLE: OnceLock<MapColorTable> = OnceLock::new();
    TABLE.get_or_init(MapColorTable::new)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lookup_packs_tiles_then_walls_then_gradients() {
        let table = MapColorTable::new();
        assert_eq!(table.lookup[0], [0, 0, 0]);
        assert_ne!(table.tile_lookup[0], 0);
        assert_eq!(table.lookup[table.tile_lookup[0] as usize], [151, 107, 75]);
        assert_eq!(table.tile_position, 1);
        assert_eq!(table.liquid_position, table.wall_range_end);
    }

    #[test]
    fn lookup_length_is_option_counts_plus_774() {
        let table = MapColorTable::new();
        let total = table
            .tile_option_counts
            .iter()
            .map(|&c| c as usize)
            .sum::<usize>()
            + table
                .wall_option_counts
                .iter()
                .map(|&c| c as usize)
                .sum::<usize>();
        assert_eq!(table.lookup.len(), total + 774);
    }

    #[test]
    fn hell_is_last() {
        let table = MapColorTable::new();
        assert_eq!(table.hell_position, table.lookup.len() as u16 - 1);
        assert_eq!(table.lookup[table.hell_position as usize], [50, 44, 38]);
    }

    #[test]
    fn snow_types_point_at_snow_tiles() {
        let table = MapColorTable::new();
        assert_eq!(
            table.snow_types,
            [
                table.tile_lookup[147],
                table.tile_lookup[161],
                table.tile_lookup[162],
                table.tile_lookup[163],
                table.tile_lookup[164],
                table.tile_lookup[200],
            ]
        );
    }

    #[test]
    fn sky_gradient_ends_match_the_source() {
        assert_eq!(sky_gradient(0), [50, 40, 255]);
        assert_eq!(sky_gradient(255), [144, 184, 255]);
    }

    #[test]
    fn dirt_and_rock_gradient_ends() {
        assert_eq!(dirt_gradient(0), [88, 61, 46]);
        assert_eq!(dirt_gradient(255), [37, 78, 123]);
        assert_eq!(rock_gradient(0), [74, 67, 60]);
        assert_eq!(rock_gradient(255), [53, 70, 97]);
    }

    #[test]
    fn liquid_colors_are_vanilla() {
        assert_eq!(
            LIQUID_COLORS,
            [
                [9, 61, 191],
                [253, 32, 3],
                [254, 194, 20],
                [161, 127, 255]
            ]
        );
    }
}
