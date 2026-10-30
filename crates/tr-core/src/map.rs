//! Packed map colour lookup and biome gradients.
//!
//! Ports `MapHelper.Initialize` (`decompiled/Terraria/Map/MapHelper.cs:1683-1788`):
//! per-tile colours first, then per-wall, then the four liquid colours, the
//! 256-entry sky/dirt/rock gradients and finally the single hell colour.

use std::sync::OnceLock;

use crate::mapdata::{TILE_COLORS, TILE_COUNT, WALL_COLORS, WALL_COUNT};
use crate::tile::{FLAG_FULLBRIGHT, LiquidKind, Tile};
use crate::tile_light::paint_color;
use crate::world::World;

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

/// One world-map pixel (`Terraria.Map.MapTile`).
///
/// `extra` mirrors the C# `_extraData` byte: bit 7 `IsChanged`, bit 6
/// `UpdateQueued`, bits 0-4 `Color` (`MapTile.cs:9-59`).
#[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
pub struct MapTile {
    /// Packed `colorLookup` index: `baseType + baseOption`.
    pub ty: u16,
    /// Map light 0..=255; `0` means the tile is not revealed.
    pub light: u8,
    pub extra: u8,
}

impl MapTile {
    const CHANGED: u8 = 0x80;
    const QUEUED: u8 = 0x40;
    const COLOR_MASK: u8 = 0x1f;

    /// `MapTile.Color` (`MapTile.cs:49-59`).
    #[inline]
    pub fn color(&self) -> u8 {
        self.extra & Self::COLOR_MASK
    }

    #[inline]
    pub fn set_color(&mut self, c: u8) {
        self.extra = (self.extra & !Self::COLOR_MASK) | (c & Self::COLOR_MASK);
    }

    /// `MapTile.IsChanged` (`MapTile.cs:11-28`).
    #[inline]
    pub fn is_changed(&self) -> bool {
        self.extra & Self::CHANGED != 0
    }

    #[inline]
    pub fn set_changed(&mut self, v: bool) {
        if v {
            self.extra |= Self::CHANGED;
        } else {
            self.extra &= !Self::CHANGED;
        }
    }

    /// `MapTile.UpdateQueued` (`MapTile.cs:30-47`).
    #[inline]
    pub fn update_queued(&self) -> bool {
        self.extra & Self::QUEUED != 0
    }

    #[inline]
    pub fn set_update_queued(&mut self, v: bool) {
        if v {
            self.extra |= Self::QUEUED;
        } else {
            self.extra &= !Self::QUEUED;
        }
    }

    /// `MapTile.Create` (`MapTile.cs:98-101`): the changed bit is always set.
    #[inline]
    pub fn create(ty: u16, light: u8, color: u8) -> Self {
        Self {
            ty,
            light,
            extra: color | Self::CHANGED,
        }
    }

    /// `MapTile.Clear` (`MapTile.cs:86-91`).
    #[inline]
    pub fn clear(&mut self) {
        self.ty = 0;
        self.light = 0;
        self.extra = 0;
    }

    /// `MapTile.WithLight` (`MapTile.cs:93-96`): sets the changed bit.
    #[inline]
    pub fn with_light(&self, light: u8) -> Self {
        Self {
            ty: self.ty,
            light,
            extra: self.extra | Self::CHANGED,
        }
    }

    /// `MapTile.Equals` (`MapTile.cs:68-75`).
    #[inline]
    pub fn equals_all(&self, o: &Self) -> bool {
        self.light == o.light && self.ty == o.ty && self.color() == o.color()
    }

    /// `MapTile.EqualsWithoutLight` (`MapTile.cs:77-84`).
    #[inline]
    pub fn equals_without_light(&self, o: &Self) -> bool {
        self.ty == o.ty && self.color() == o.color()
    }
}

/// `WorldMap.IsRevealed` (`WorldMap.cs:44-47`): `Light > 0`.
#[inline]
pub fn is_revealed(t: MapTile) -> bool {
    t.light > 0
}

/// `Tile.liquidType()` -> the `LIQUID_COLORS` slot (`MapHelper.cs:1996-1998`).
pub fn liquid_index(kind: LiquidKind) -> u16 {
    match kind {
        LiquidKind::Water => 0,
        LiquidKind::Lava => 1,
        LiquidKind::Honey => 2,
        LiquidKind::Shimmer => 3,
    }
}

/// `MapHelper.IsBackground` (`MapHelper.cs:1895-1898`).
#[inline]
pub fn is_background(map_type: u16) -> bool {
    map_type >= color_table().sky_position
}

/// `MapHelper.MapColor` (`MapHelper.cs:1823-1874`).
///
/// `paintColor(c)` is reused from [`crate::tile_light::paint_color`] (a faithful
/// `WorldGen.paintColor` port); its normalised floats are converted back to the
/// original bytes (divergence ruling 7).
fn map_color(ty: u16, old: &mut [u8; 3], color_type: u8) {
    let pc = paint_color(color_type);
    let color = [
        (pc[0] * 255.0).round() as u8,
        (pc[1] * 255.0).round() as u8,
        (pc[2] * 255.0).round() as u8,
    ];
    // Sort so `num` is the largest channel (MapHelper.cs:1826-1840).
    let mut num = old[0] as f32 / 255.0;
    let mut num2 = old[1] as f32 / 255.0;
    let mut num3 = old[2] as f32 / 255.0;
    if num2 > num {
        std::mem::swap(&mut num, &mut num2);
    }
    if num3 > num {
        std::mem::swap(&mut num, &mut num3);
    }
    let table = color_table();
    match color_type {
        29 => {
            // Shadow paint (MapHelper.cs:1843-1850).
            let num7 = num3 * 0.3;
            old[0] = (color[0] as f32 * num7) as u8;
            old[1] = (color[1] as f32 * num7) as u8;
            old[2] = (color[2] as f32 * num7) as u8;
        }
        30 => {
            // Negative paint inverts, or halves a wall (MapHelper.cs:1851-1864).
            if ty >= table.wall_range_start && ty <= table.wall_range_end {
                old[0] = ((255 - old[0]) as f32 * 0.5) as u8;
                old[1] = ((255 - old[1]) as f32 * 0.5) as u8;
                old[2] = ((255 - old[2]) as f32 * 0.5) as u8;
            } else {
                old[0] = 255 - old[0];
                old[1] = 255 - old[1];
                old[2] = 255 - old[2];
            }
        }
        _ => {
            // Default paint multiplies by the max channel (MapHelper.cs:1865-1872).
            old[0] = (color[0] as f32 * num) as u8;
            old[1] = (color[1] as f32 * num) as u8;
            old[2] = (color[2] as f32 * num) as u8;
        }
    }
}

/// `MapHelper.GetMapTileXnaColor` (`MapHelper.cs:1876-1893`).
///
/// `i`/`j` are unused in the C# too; kept for signature parity (ruling 10).
pub fn get_map_tile_color(t: MapTile, _i: i32, _j: i32) -> [u8; 3] {
    let mut old = color_table().color(t.ty);
    let color = t.color();
    if color > 0 {
        map_color(t.ty, &mut old, color);
    }
    if t.light == 255 {
        return old;
    }
    let num = t.light as f32 / 255.0;
    old[0] = (old[0] as f32 * num) as u8;
    old[1] = (old[1] as f32 * num) as u8;
    old[2] = (old[2] as f32 * num) as u8;
    old
}

/// `MapHelper.CalcSkyGradient` (`MapHelper.cs:2078-2081`), called with
/// `maxSkyGradients == 256`, so `(maxSkyGradients - 1) == 255`.
fn calc_sky_gradient(j: i32, world_surface: f64) -> u16 {
    color_table().sky_position + (255.0 * (j as f64 / world_surface)) as u8 as u16
}

/// `Main.Map[x, y].Type` read from our row-major map slice (`World::index`).
fn map_type_at(world: &World, map: &[MapTile], x: i32, y: i32) -> u16 {
    if !world.in_bounds(x, y) {
        return 0;
    }
    map.get(world.index(x, y)).map_or(0, |t| t.ty)
}

/// `MapHelper.GetTileType` (`MapHelper.cs:1925-1985`).
///
/// Our `Tile` has a single 8-bit `flags` byte: `invisibleBlock()`
/// (`bTileHeader3 & 0x20`) is not representable (ruling 1), `fullbrightBlock()`
/// (`bTileHeader3 & 0x80`) maps to [`FLAG_FULLBRIGHT`] (ruling 2) and
/// `DebugOptions.ShowUnbreakableWall` is a debug toggle treated as `false`.
fn get_tile_type(
    i: i32,
    j: i32,
    tile: &Tile,
    new_color: &mut i32,
    new_light: &mut i32,
    base_type: &mut i32,
    base_option: &mut i32,
    world: &World,
) {
    if !tile.active() {
        return;
    }
    let mut num = tile.kind as i32;
    // Vanilla indexes `tileLookup[num]` directly; guard malformed ids.
    *base_type = color_table()
        .tile_lookup
        .get(tile.kind as usize)
        .copied()
        .unwrap_or(0) as i32;
    let invisible = false; // tile.invisibleBlock() — ruling 1.
    if tile.flags & FLAG_FULLBRIGHT != 0 && !invisible {
        *new_light = 255;
    }
    if invisible {
        *base_type = 0;
        return;
    }
    match num {
        5 => {
            // WorldGen.IsThisAMushroomTree is not ported (ruling 5); only this
            // tile's option (1) is affected, so it stays 0.
            *new_color = tile.paint() as i32; // tile.color() — ruling 6.
            return;
        }
        51 | 697 => {
            if (i + j) % 2 == 0 {
                *base_type = 0;
            }
        }
        19 => {
            if tile.frame_y as i32 == 864 {
                *base_type = 0;
            }
        }
        184 => {
            if tile.frame_x / 22 == 10 {
                num = 627;
                *base_type = color_table()
                    .tile_lookup
                    .get(num as usize)
                    .copied()
                    .unwrap_or(0) as i32;
            }
        }
        _ => {}
    }
    if *base_type != 0 {
        // `MapHelper.GetTileBaseOption` (MapHelper.cs:1975).
        *base_option = get_tile_base_option(i, j, num as u16, tile, world) as i32;
        if num == 160 {
            *new_color = 0;
        } else {
            *new_color = tile.paint() as i32; // tile.color() — ruling 6.
        }
    }
}

/// `MapHelper.GetTileBaseOption` (`MapHelper.cs:2083-3005`): the colour variant
/// within a tile's option list, added to `baseType` to index the packed
/// `colorLookup`.
///
/// The switch is transcribed case-for-case in source order. Four cases need data
/// we do not have yet and are substituted with `0` (flagged in place, and listed
/// in `docs/ANALYSIS.md` by Task 9):
/// - `461`  — `Main.SceneMetrics.ZoneCorrupt/ZoneCrimson/ZoneHallow`
/// - `80`   — `WorldGen.GetCactusType`
/// - `529`  — `WorldGen.GetBiomeInfluence`
/// - `530`  — `WorldGen.GetBiomeInfluence`
///
/// `world` is only read by those substituted cases, hence `_world`.
#[allow(unused_assignments)] // `case 4` mirrors the source's assign-then-overwrite.
pub fn get_tile_base_option(
    x: i32,
    y: i32,
    tile_type: u16,
    tile: &Tile,
    _world: &World,
) -> u8 {
    let frame_x = tile.frame_x as i32;
    let frame_y = tile.frame_y as i32;
    let mut base_option = 0i32;
    match tile_type {
        89 => {
            base_option = match frame_x / 54 {
                0 | 21 | 23 => 0,
                43 => 2,
                _ => 1,
            };
        }
        160 | 627 | 628 | 692 => {
            base_option = (x + y) % 9;
        }
        461 => {
            // `Main.SceneMetrics.ZoneCorrupt/ZoneCrimson/ZoneHallow` — not ported.
            base_option = 0;
        }
        80 => {
            // `WorldGen.GetCactusType(x, y, frameX, frameY, ...)` — not ported.
            base_option = 0;
        }
        529 => {
            // `WorldGen.GetBiomeInfluence(x, x, y+1, y+1, ...)` — not ported.
            base_option = 0;
        }
        530 => {
            // `WorldGen.GetBiomeInfluence(...)` — not ported.
            base_option = 0;
        }
        19 => {
            // MapHelper.cs:2179-2188.
            let num = frame_y / 18;
            base_option = 0;
            if num == 48 {
                base_option = 1;
            }
        }
        15 => {
            // MapHelper.cs:2189-2198.
            let num = frame_y / 40;
            base_option = 0;
            if num == 1 || num == 20 {
                base_option = 1;
            }
        }
        518 | 519 => {
            // MapHelper.cs:2199-2202.
            base_option = frame_y / 18;
        }
        4 => {
            // MapHelper.cs:2203-2209: assigns 1 then unconditionally 0.
            if frame_x < 66 {
                base_option = 1;
            }
            base_option = 0;
        }
        572 => {
            // MapHelper.cs:2210-2212.
            base_option = frame_y / 36;
        }
        21 | 441 => {
            // MapHelper.cs:2213-2239.
            base_option = match frame_x / 36 {
                1 | 2 | 10 | 13 | 15 => 1,
                3 | 4 => 2,
                6 => 3,
                11 | 17 => 4,
                _ => 0,
            };
        }
        467 | 468 => {
            // MapHelper.cs:2240-2278.
            let num = frame_x / 36;
            base_option = match num {
                0..=11 => num,
                12 | 13 | 15 | 18 => 10,
                14 | 17 => 7,
                16 => 3,
                _ => 12,
            };
        }
        560 => {
            // MapHelper.cs:2279-2291.
            let num = frame_x / 36;
            if (0..=2).contains(&num) {
                base_option = num;
            } else {
                base_option = 0;
            }
        }
        28 | 653 => {
            // MapHelper.cs:2292-2342.
            base_option = if frame_y < 144 {
                0
            } else if frame_y < 252 {
                1
            } else if frame_y < 360 || (frame_y >= 900 && frame_y < 1008) {
                2
            } else if frame_y < 468 {
                3
            } else if frame_y < 576 {
                4
            } else if frame_y < 684 {
                5
            } else if frame_y < 792 {
                6
            } else if frame_y < 900 {
                8
            } else if frame_y < 1008 {
                7
            } else if frame_y < 1116 {
                0
            } else if frame_y < 1224 {
                3
            } else {
                7
            };
        }
        27 => {
            // MapHelper.cs:2343-2352.
            base_option = if frame_y < 34 { 1 } else { 0 };
        }
        31 | 696 => {
            // MapHelper.cs:2353-2363.
            base_option = if frame_x >= 36 { 1 } else { 0 };
        }
        26 | 695 => {
            // MapHelper.cs:2364-2374.
            base_option = if frame_x >= 54 { 1 } else { 0 };
        }
        137 => {
            // MapHelper.cs:2375-2391.
            base_option = match frame_y / 18 {
                1 | 2 | 3 | 4 => 1,
                5 => 2,
                _ => 0,
            };
        }
        82 | 83 | 84 => {
            // MapHelper.cs:2392-2423.
            base_option = if frame_x < 18 {
                0
            } else if frame_x < 36 {
                1
            } else if frame_x < 54 {
                2
            } else if frame_x < 72 {
                3
            } else if frame_x < 90 {
                4
            } else if frame_x < 108 {
                5
            } else {
                6
            };
        }
        591 => {
            // MapHelper.cs:2424-2426.
            base_option = frame_x / 36;
        }
        105 => {
            // MapHelper.cs:2427-2440.
            base_option = if frame_x >= 1548 && frame_x <= 1654 {
                1
            } else if frame_x >= 1656 && frame_x <= 1798 {
                2
            } else {
                0
            };
        }
        133 => {
            // MapHelper.cs:2441-2450.
            base_option = if frame_x < 52 { 0 } else { 1 };
        }
        134 => {
            // MapHelper.cs:2451-2460.
            base_option = if frame_x < 28 { 0 } else { 1 };
        }
        149 => {
            // MapHelper.cs:2461-2463.
            base_option = y % 3;
        }
        165 | 693 | 694 => {
            // MapHelper.cs:2464-2487.
            base_option = if frame_x < 54 {
                0
            } else if frame_x < 106 {
                1
            } else if frame_x >= 216 {
                1
            } else if frame_x < 162 {
                2
            } else {
                3
            };
        }
        178 => {
            // MapHelper.cs:2488-2517.
            base_option = if frame_x < 18 {
                0
            } else if frame_x < 36 {
                1
            } else if frame_x < 54 {
                2
            } else if frame_x < 72 {
                3
            } else if frame_x < 90 {
                4
            } else if frame_x < 108 {
                5
            } else {
                6
            };
        }
        184 => {
            // MapHelper.cs:2518-2563 (no final else: >= 242 leaves it untouched).
            if frame_x < 22 {
                base_option = 0;
            } else if frame_x < 44 {
                base_option = 1;
            } else if frame_x < 66 {
                base_option = 2;
            } else if frame_x < 88 {
                base_option = 3;
            } else if frame_x < 110 {
                base_option = 4;
            } else if frame_x < 132 {
                base_option = 5;
            } else if frame_x < 154 {
                base_option = 6;
            } else if frame_x < 176 {
                base_option = 7;
            } else if frame_x < 198 {
                base_option = 8;
            } else if frame_x < 220 {
                base_option = 9;
            } else if frame_x < 242 {
                base_option = 10;
            }
        }
        650 => {
            // MapHelper.cs:2564-2600 (no final else).
            let num = frame_x / 18;
            if num < 6 || num == 28 || num == 29 || num == 30 || num == 31 || num == 32 {
                base_option = 0;
            } else if num < 12 || num == 33 || num == 34 || num == 35 {
                base_option = 1;
            } else if num < 28 {
                base_option = 2;
            } else if num < 48 {
                base_option = 3;
            } else if num < 54 {
                base_option = 4;
            } else if num < 72 {
                base_option = 0;
            } else if num == 72 {
                base_option = 1;
            } else if num < 78 {
                base_option = 11;
            }
        }
        649 => {
            // MapHelper.cs:2601-2637 (no final else).
            let num = frame_x / 36 + (frame_y / 18 - 1) * 18;
            if num < 6
                || num == 19
                || num == 20
                || num == 21
                || num == 22
                || num == 23
                || num == 24
                || num == 33
                || num == 38
                || num == 39
                || num == 40
            {
                base_option = 0;
            } else if num < 16 {
                base_option = 2;
            } else if num < 19 || num == 31 || num == 32 {
                base_option = 1;
            } else if num < 31 {
                base_option = 3;
            } else if num < 38 {
                base_option = 4;
            } else if num < 59 {
                base_option = 0;
            } else if num < 62 {
                base_option = 1;
            } else if num < 65 {
                base_option = 11;
            }
        }
        185 => {
            // MapHelper.cs:2638-2711: frame Y selects the 650 or 649 layout.
            if frame_y < 18 {
                let num = frame_x / 18;
                if num < 6 || num == 28 || num == 29 || num == 30 || num == 31 || num == 32 {
                    base_option = 0;
                } else if num < 12 || num == 33 || num == 34 || num == 35 {
                    base_option = 1;
                } else if num < 28 {
                    base_option = 2;
                } else if num < 48 {
                    base_option = 3;
                } else if num < 54 {
                    base_option = 4;
                } else if num < 72 {
                    base_option = 0;
                } else if num == 72 {
                    base_option = 1;
                } else if num < 78 {
                    base_option = 11;
                }
            } else {
                let num = frame_x / 36 + (frame_y / 18 - 1) * 18;
                if num < 6
                    || num == 19
                    || num == 20
                    || num == 21
                    || num == 22
                    || num == 23
                    || num == 24
                    || num == 33
                    || num == 38
                    || num == 39
                    || num == 40
                {
                    base_option = 0;
                } else if num < 16 {
                    base_option = 2;
                } else if num < 19 || num == 31 || num == 32 {
                    base_option = 1;
                } else if num < 31 {
                    base_option = 3;
                } else if num < 38 {
                    base_option = 4;
                } else if num < 59 {
                    base_option = 0;
                } else if num < 62 {
                    base_option = 1;
                } else if num < 65 {
                    base_option = 11;
                }
            }
        }
        186 | 647 => {
            // MapHelper.cs:2712-2737 (no final else).
            let num = frame_x / 54;
            if num < 7 {
                base_option = 2;
            } else if num < 22 || num == 33 || num == 34 || num == 35 {
                base_option = 0;
            } else if num < 25 {
                base_option = 1;
            } else if num == 25 {
                base_option = 5;
            } else if num < 32 {
                base_option = 3;
            }
        }
        187 | 648 => {
            // MapHelper.cs:2738-2791 (no final else).
            let num = frame_x / 54 + frame_y / 36 * 36;
            if num < 3 || num == 14 || num == 15 || num == 16 {
                base_option = 0;
            } else if num < 6 {
                base_option = 6;
            } else if num < 9 {
                base_option = 7;
            } else if num < 14 {
                base_option = 4;
            } else if num < 18 {
                base_option = 4;
            } else if num < 23 {
                base_option = 8;
            } else if num < 25 {
                base_option = 0;
            } else if num < 29 {
                base_option = 1;
            } else if num < 47 {
                base_option = 0;
            } else if num < 50 {
                base_option = 1;
            } else if num < 52 {
                base_option = 10;
            } else if num < 55 {
                base_option = 2;
            }
        }
        227 => {
            // MapHelper.cs:2792-2794.
            base_option = frame_x / 34;
        }
        129 => {
            // MapHelper.cs:2795-2804.
            base_option = if frame_x >= 324 { 1 } else { 0 };
        }
        240 => {
            // MapHelper.cs:2805-2868.
            let num = frame_x / 54 + frame_y / 54 * 36;
            if (0..=11).contains(&num)
                || (47..=53).contains(&num)
                || num == 72
                || num == 73
                || num == 75
            {
                base_option = 0;
            } else if (num < 12 || num > 15)
                && (num < 18 || num > 35)
                && (num < 63 || num > 71)
            {
                // The source's `return`s mean every arm here terminates the
                // function; only the 74/76..100 arms set 1, the rest keep 0.
                match num {
                    16 | 17 => base_option = 2,
                    41..=45 => base_option = 3,
                    46 => base_option = 4,
                    74 | 76..=100 => base_option = 1,
                    _ => {}
                }
            } else {
                base_option = 1;
            }
        }
        242 => {
            // MapHelper.cs:2869-2881.
            let num = frame_y / 72;
            base_option = if frame_x / 106 == 0 && (22..=24).contains(&num) {
                1
            } else {
                0
            };
        }
        440 => {
            // MapHelper.cs:2882-2891.
            let mut num = frame_x / 54;
            if num > 6 {
                num = 6;
            }
            base_option = num;
        }
        457 => {
            // MapHelper.cs:2892-2901.
            let mut num = frame_x / 36;
            if num > 4 {
                num = 4;
            }
            base_option = num;
        }
        453 => {
            // MapHelper.cs:2902-2911.
            let mut num = frame_x / 36;
            if num > 2 {
                num = 2;
            }
            base_option = num;
        }
        419 => {
            // MapHelper.cs:2912-2921.
            let mut num = frame_x / 18;
            if num > 2 {
                num = 2;
            }
            base_option = num;
        }
        428 => {
            // MapHelper.cs:2922-2931.
            let mut num = frame_y / 18;
            if num > 3 {
                num = 3;
            }
            base_option = num;
        }
        420 => {
            // MapHelper.cs:2932-2941.
            let mut num = frame_y / 18;
            if num > 5 {
                num = 5;
            }
            base_option = num;
        }
        423 => {
            // MapHelper.cs:2942-2951.
            let mut num = frame_y / 18;
            if num > 6 {
                num = 6;
            }
            base_option = num;
        }
        493 => {
            // MapHelper.cs:2952-2977.
            base_option = if frame_x < 18 {
                0
            } else if frame_x < 36 {
                1
            } else if frame_x < 54 {
                2
            } else if frame_x < 72 {
                3
            } else if frame_x < 90 {
                4
            } else {
                5
            };
        }
        548 => {
            // MapHelper.cs:2978-2987.
            base_option = if frame_x / 54 < 7 { 0 } else { 1 };
        }
        597 => {
            // MapHelper.cs:2988-3000.
            let num = frame_x / 54;
            if (0..=10).contains(&num) {
                base_option = num;
            } else {
                base_option = 0;
            }
        }
        _ => {
            // MapHelper.cs:3001-3003.
            base_option = 0;
        }
    }
    base_option as u8
}

/// `MapHelper.GetWallType` (`MapHelper.cs:1987-2025`).
///
/// `invisibleWall()` (`bTileHeader3 & 0x40`, ruling 1) and `fullbrightWall()`
/// (`sTileHeader & 0x8000`, ruling 3 — a *separate* wall flag we do not store)
/// are both absent, so the `newLight = 255` branch is unreachable.
fn get_wall_type(
    i: i32,
    _j: i32,
    tile: &Tile,
    new_color: &mut i32,
    _new_light: &mut i32,
    base_type: &mut i32,
    base_option: &mut i32,
) {
    let invisible = false; // tile.invisibleWall() — ruling 1.
    if tile.liquid > 32 {
        // Tile.liquidType() -> LIQUID_COLORS slot (MapHelper.cs:1996-1998).
        *base_type = color_table().liquid_position as i32 + liquid_index(tile.liquid_kind()) as i32;
    } else if !invisible && tile.wall > 0 && (tile.wall as usize) < WALL_COUNT {
        let wall = tile.wall as usize;
        *base_type = color_table().wall_lookup[wall] as i32;
        *new_color = tile.paint() as i32; // tile.wallColor() — ruling 6.
        match tile.wall {
            21 | 88 | 89 | 90 | 91 | 92 | 93 | 168 | 241 => *new_color = 0,
            27 => *base_option = i % 2,
            _ => *base_option = 0,
        }
    }
}

/// `MapHelper.GetBackgroundType` (`MapHelper.cs:2027-2076`).
fn get_background_type(world: &World, map: &[MapTile], i: i32, j: i32, new_light: &mut i32) -> u16 {
    if (j as f64) < world.world_surface {
        // Main.remixWorld == false (ruling 4).
        *new_light = 255;
        return calc_sky_gradient(j, world.world_surface);
    }
    // Main.UnderworldLayer == Main.maxTilesY - 200 (Main.cs:2790).
    let underworld_layer = world.height - 200;
    if j < underworld_layer {
        let mut b = 0u8;
        // `sceneArea`/`sceneSnowiness` are not ported (ruling 8): the neighbour
        // scan always runs, bounds-checked against the world by `map_type_at`.
        'scan: for k in (i - 36..=i + 30).step_by(10) {
            for l in (j - 36..=j + 30).step_by(10) {
                let ty = map_type_at(world, map, k, l);
                for snow in color_table().snow_types {
                    if snow == ty {
                        b = 255;
                        break 'scan;
                    }
                }
            }
        }
        if (j as f64) < world.rock_layer {
            return color_table().dirt_position + b as u16;
        }
        return color_table().rock_position + b as u16;
    }
    color_table().hell_position
}

/// `MapHelper.CreateMapTile` (`MapHelper.cs:1900-1923`).
pub fn create_map_tile(
    world: &World,
    map: &[MapTile],
    i: i32,
    j: i32,
    light: u8,
    background_override: u16,
) -> MapTile {
    let Some(tile) = world.tile(i, j) else {
        return MapTile::default();
    };
    let mut new_color = 0i32;
    let mut new_light = light as i32;
    let mut base_type = 0i32;
    let mut base_option = 0i32;
    get_tile_type(
        i,
        j,
        tile,
        &mut new_color,
        &mut new_light,
        &mut base_type,
        &mut base_option,
        world,
    );
    if base_type == 0 {
        get_wall_type(
            i,
            j,
            tile,
            &mut new_color,
            &mut new_light,
            &mut base_type,
            &mut base_option,
        );
    }
    if base_type == 0 {
        new_color = 0;
        new_light = light as i32;
        base_type = if background_override == 0 {
            get_background_type(world, map, i, j, &mut new_light) as i32
        } else {
            background_override as i32
        };
    }
    MapTile::create(
        (base_type + base_option) as u16,
        new_light as u8,
        new_color as u8,
    )
}

// ---------------------------------------------------------------------------
// Task 4: `.map` v326 persistence.
//
// Ports `MapHelper.InternalSaveMap` (`MapHelper.cs:3030-3322`) and
// `MapHelper.LoadMapVersion2` (`:3749-4027`). The file is one contiguous stream:
// an uncompressed header, the uncompressed option tables, then the row RLE body
// wrapped in a raw deflate stream (`3124`, `3929`). `tr-core` owns every byte-
// layout primitive; the deflate wrapping and file IO live in `tr-app`, which is
// the only crate that depends on `flate2`.
//
// Not ported: `Main.ActivePlayerFileData.MapFileName` / cloud save paths,
// `Main.statusText` progress text, `WorldMap.ClearEdges`, and the persistent
// `MapFileMetadata` revision counter — a fresh metadata is written, so the
// revision is always `1` and the output is deterministic.
// ---------------------------------------------------------------------------

/// `MapHelper` map format version for 1.4.5 (`MapHelper.cs:3050`).
pub const MAP_FORMAT_VERSION: u32 = 326;

/// `FileMetadata.MAGIC_NUMBER` (`FileMetadata.cs:8`).
const FILE_METADATA_MAGIC: u64 = 0x0063_6967_6F6C_6572;
/// `(1u64 << 56)` mask for the low 6 bytes (`FileMetadata.cs:59`).
const FILE_METADATA_MAGIC_MASK: u64 = 0x00FF_FFFF_FFFF_FFFF;
/// `FileType.Map` (`FileMetadata.cs:24`).
const FILE_TYPE_MAP: u64 = 1;

/// Fixed prefix (`InternalSaveMap` 3050-3061).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MapHeader {
    pub version: u32,
    pub seed: i32,
    pub height: i32,
    pub width: i32,
    pub tile_count: i16,   // TileID.Count, written
    pub wall_count: i16,   // WallID.Count
    pub liquid_count: i16, // 4
    pub sky_count: i16,    // 256
    pub dirt_count: i16,   // 256
    pub rock_count: i16,   // 256
}

/// The whole uncompressed prefix (`LoadMapVersion2` 3749-3838).
#[derive(Clone, Debug)]
pub struct MapPrefix {
    pub header: MapHeader,
    pub world_name: String,
    pub tile_options: Vec<u8>, // `array3`: len == tile_count
    pub wall_options: Vec<u8>, // `array4`: len == wall_count
    pub body_offset: usize,    // where the deflate stream starts
}

/// `BinaryWriter.Write(string)`: 7-bit-encoded byte length, then UTF-8.
fn write_cs_string(out: &mut Vec<u8>, s: &str) {
    let bytes = s.as_bytes();
    let mut len = bytes.len();
    while len >= 0x80 {
        out.push(((len & 0x7f) | 0x80) as u8);
        len >>= 7;
    }
    out.push(len as u8);
    out.extend_from_slice(bytes);
}

/// `InternalSaveMap` 3050-3061 — version, `FileMetadata`, name, seed, sizes, six i16.
pub fn write_map_header(world: &World) -> Vec<u8> {
    let mut out = Vec::with_capacity(64);
    out.extend_from_slice(&MAP_FORMAT_VERSION.to_le_bytes());
    // `Main.MapFileMetadata.IncrementAndWrite` from a fresh metadata (revision 1)
    // then `FileMetadata.Write`: magic|type, revision, favourite flag.
    out.extend_from_slice(&(FILE_METADATA_MAGIC | (FILE_TYPE_MAP << 56)).to_le_bytes());
    out.extend_from_slice(&1u32.to_le_bytes());
    out.extend_from_slice(&0u64.to_le_bytes());
    write_cs_string(&mut out, &world.name);
    out.extend_from_slice(&(world.seed as i32).to_le_bytes());
    out.extend_from_slice(&world.height.to_le_bytes());
    out.extend_from_slice(&world.width.to_le_bytes());
    for count in [
        TILE_COUNT as i16,
        WALL_COUNT as i16,
        4,
        256,
        256,
        256,
    ] {
        out.extend_from_slice(&count.to_le_bytes());
    }
    out
}

/// `InternalSaveMap` 3062-3109: one LSB-first bitmap, flushed every 8 bits.
fn write_option_bitmap(out: &mut Vec<u8>, counts: &[u8]) {
    let mut bit: u8 = 1;
    let mut acc: u8 = 0;
    for &c in counts {
        if c != 1 {
            acc |= bit;
        }
        if bit == 128 {
            out.push(acc);
            acc = 0;
            bit = 1;
        } else {
            bit <<= 1;
        }
    }
    if bit != 1 {
        out.push(acc);
    }
}

/// `InternalSaveMap` 3062-3123 — option bitmaps + the non-default option counts.
pub fn write_option_tables() -> Vec<u8> {
    let table = color_table();
    let mut out = Vec::new();
    write_option_bitmap(&mut out, &table.tile_option_counts);
    write_option_bitmap(&mut out, &table.wall_option_counts);
    for &c in &table.tile_option_counts {
        if c != 1 {
            out.push(c);
        }
    }
    for &c in &table.wall_option_counts {
        if c != 1 {
            out.push(c);
        }
    }
    out
}

/// `InternalSaveMap` 3125-3318 — the per-row RLE body, before deflate.
///
/// The source stages bytes in a 16384-byte `array` flushed to the `DeflateStream`
/// once `num >= 4096`; since every flush is appended to the same stream the
/// concatenated bytes are identical to appending directly here.
pub fn encode_tiles(map: &[MapTile], width: usize, height: usize) -> Vec<u8> {
    let table = color_table();
    let mut out = Vec::new();
    for j in 0..height {
        let mut x = 0usize;
        while x < width {
            let map_tile = map[j * width + x];
            let mut b4 = 0u8;
            let b5 = 0u8; // `3138`: never assigned in the source
            let mut b6 = 0u8;
            let mut ty = 0u16;
            let mut run = 0usize;
            let mut flag = true; // write the type
            let mut flag2 = true; // write the light
            let mut run_start = 0usize;
            let mut run_end = 0usize;
            let category: u8;

            if map_tile.light <= 18 {
                // `3147-3162`: an unrevealed skip-run.
                flag2 = false;
                flag = false;
                category = 0;
                let mut n9 = x + 1;
                let mut remaining = width - x - 1;
                while remaining > 0 && map[j * width + n9].light <= 18 {
                    run += 1;
                    remaining -= 1;
                    n9 += 1;
                }
            } else {
                b6 = map_tile.color();
                ty = map_tile.ty;
                if ty < table.wall_position {
                    category = 1;
                    ty = ty.wrapping_sub(table.tile_position);
                } else if ty < table.liquid_position {
                    category = 2;
                    ty = ty.wrapping_sub(table.wall_position);
                } else if ty < table.sky_position {
                    let mut liquid = ty.wrapping_sub(table.liquid_position);
                    if liquid == 3 {
                        b4 |= 0x40; // shimmer rides the water category
                        liquid = 0;
                    }
                    category = 3 + liquid as u8;
                    flag = false;
                } else if ty < table.dirt_position {
                    category = 6;
                    flag2 = false;
                    flag = false;
                } else if ty < table.hell_position {
                    category = 7;
                    ty = if ty >= table.rock_position {
                        ty.wrapping_sub(table.rock_position)
                    } else {
                        ty.wrapping_sub(table.dirt_position)
                    };
                } else {
                    category = 6;
                    flag = false;
                }
                if map_tile.light == 255 {
                    flag2 = false;
                }
                if flag2 {
                    // `3208-3226`: light != 255 keeps `EqualsWithoutLight` runs.
                    run = 0;
                    let mut n9 = x + 1;
                    let mut remaining = width - x - 1;
                    run_start = n9;
                    while remaining > 0 {
                        if map_tile.equals_without_light(&map[j * width + n9]) {
                            remaining -= 1;
                            run += 1;
                            n9 += 1;
                            continue;
                        }
                        break;
                    }
                    // The decompiled `num6 = 0` default would leave the run end at 0
                    // when the run reaches the last column and drop the run's light
                    // bytes, but `LoadMapVersion2:4017-4023` unconditionally reads
                    // `num27` lights whenever `light != 255`. End the run at
                    // `run_start + run` (exactly the C# `num6` whenever a mismatch
                    // sets it) so the reader stays in sync.
                    run_end = run_start + run;
                } else {
                    // `3227-3238`: light == 255 keeps exact `Equals` runs.
                    run = 0;
                    let mut n9 = x + 1;
                    let mut remaining = width - x - 1;
                    while remaining > 0 && map_tile.equals_all(&map[j * width + n9]) {
                        remaining -= 1;
                        run += 1;
                        n9 += 1;
                    }
                }
            }

            if b6 > 0 {
                b4 |= b6 << 1;
            }
            if b5 != 0 {
                b4 |= 1;
            }
            let mut b3 = 0u8;
            if b4 != 0 {
                b3 |= 1; // bit0: a second header byte follows
            }
            b3 |= category << 1;
            if flag && ty > 255 {
                b3 |= 0x10; // bit4: 16-bit type
            }
            if flag2 {
                b3 |= 0x20; // bit5: a light byte follows
            }
            if run > 0 {
                b3 |= if run <= 255 { 0x40 } else { 0x80 };
            }
            out.push(b3);
            if b4 != 0 {
                out.push(b4);
            }
            if b5 != 0 {
                out.push(b5);
            }
            if flag {
                out.push(ty as u8);
                if ty > 255 {
                    out.push((ty >> 8) as u8);
                }
            }
            if flag2 {
                out.push(map_tile.light);
            }
            if run > 0 {
                out.push(run as u8);
                if run > 255 {
                    out.push((run >> 8) as u8);
                }
            }
            // `3302-3306`: the run tiles' own light values.
            for k in run_start..run_end {
                out.push(map[j * width + k].light);
            }
            x += run + 1;
        }
    }
    out
}

fn eof() -> std::io::Error {
    std::io::Error::new(
        std::io::ErrorKind::UnexpectedEof,
        "truncated .map body",
    )
}

/// A cursor over the uncompressed prefix; returns `None` past the end.
struct PrefixReader<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> PrefixReader<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, pos: 0 }
    }
    fn u8(&mut self) -> Option<u8> {
        let b = *self.bytes.get(self.pos)?;
        self.pos += 1;
        Some(b)
    }
    fn u16(&mut self) -> Option<u16> {
        Some(self.u8()? as u16 | ((self.u8()? as u16) << 8))
    }
    fn i16(&mut self) -> Option<i16> {
        self.u16().map(|v| v as i16)
    }
    fn u32(&mut self) -> Option<u32> {
        let mut v = 0u32;
        for i in 0..4 {
            v |= (self.u8()? as u32) << (8 * i);
        }
        Some(v)
    }
    fn i32(&mut self) -> Option<i32> {
        self.u32().map(|v| v as i32)
    }
    fn u64(&mut self) -> Option<u64> {
        let mut v = 0u64;
        for i in 0..8 {
            v |= (self.u8()? as u64) << (8 * i);
        }
        Some(v)
    }
    fn cs_string(&mut self) -> Option<String> {
        let mut len = 0usize;
        let mut shift = 0u32;
        loop {
            let b = self.u8()?;
            len |= ((b & 0x7f) as usize) << shift;
            if b & 0x80 == 0 {
                break;
            }
            shift += 7;
            if shift > 28 {
                return None;
            }
        }
        let end = self.pos.checked_add(len)?;
        let bytes = self.bytes.get(self.pos..end)?;
        self.pos = end;
        String::from_utf8(bytes.to_vec()).ok()
    }
}

/// `LoadMapVersion2` 3773-3810: one `count`-bit LSB-first bitmap.
fn read_option_bitmap(r: &mut PrefixReader, count: usize) -> Option<Vec<bool>> {
    let mut set = vec![false; count];
    let mut b = 0u8;
    let mut bit = 128u8;
    for slot in set.iter_mut() {
        if bit == 128 {
            b = r.u8()?;
            bit = 1;
        } else {
            bit <<= 1;
        }
        if b & bit == bit {
            *slot = true;
        }
    }
    Some(set)
}

/// `LoadMapVersion2` 3811-3838: `array3`/`array4`, one count per set bitmap bit.
fn read_option_counts(r: &mut PrefixReader, set: &[bool]) -> Option<Vec<u8>> {
    let mut options = Vec::with_capacity(set.len());
    for &is_set in set {
        options.push(if is_set { r.u8()? } else { 1 });
    }
    Some(options)
}

/// Reads the header + option tables. `Ok(None)` on a short / foreign / wrong-size
/// prefix (never panics).
pub fn read_map_prefix(bytes: &[u8]) -> std::io::Result<Option<MapPrefix>> {
    let mut r = PrefixReader::new(bytes);
    let Some(version) = r.u32() else {
        return Ok(None);
    };
    if version != MAP_FORMAT_VERSION {
        return Ok(None);
    }
    let Some(metadata) = r.u64() else {
        return Ok(None);
    };
    if metadata & FILE_METADATA_MAGIC_MASK != FILE_METADATA_MAGIC {
        return Ok(None);
    }
    if (metadata >> 56) & 0xFF != FILE_TYPE_MAP {
        return Ok(None);
    }
    let Some(_revision) = r.u32() else {
        return Ok(None);
    };
    let Some(_favourite) = r.u64() else {
        return Ok(None);
    };
    let Some(world_name) = r.cs_string() else {
        return Ok(None);
    };
    let Some(seed) = r.i32() else {
        return Ok(None);
    };
    let Some(height) = r.i32() else {
        return Ok(None);
    };
    let Some(width) = r.i32() else {
        return Ok(None);
    };
    let mut counts = [0i16; 6];
    for count in &mut counts {
        let Some(v) = r.i16() else {
            return Ok(None);
        };
        *count = v;
    }
    let [tile_count, wall_count, liquid_count, sky_count, dirt_count, rock_count] = counts;
    // Same-version files always carry 754/367/4/256/256/256. Reject any negative
    // count: `decode_tiles` sizes its offset tables from these, and a negative i16
    // cast to `usize` would panic in `vec![...]` with a capacity overflow.
    if counts.iter().any(|&c| c < 0) {
        return Ok(None);
    }
    if tile_count as usize != TILE_COUNT || wall_count as usize != WALL_COUNT {
        return Ok(None);
    }
    // `LoadMapVersion2` reads *both* bitmaps first, then both count lists.
    let Some(tile_set) = read_option_bitmap(&mut r, tile_count as usize) else {
        return Ok(None);
    };
    let Some(wall_set) = read_option_bitmap(&mut r, wall_count as usize) else {
        return Ok(None);
    };
    let Some(tile_options) = read_option_counts(&mut r, &tile_set) else {
        return Ok(None);
    };
    let Some(wall_options) = read_option_counts(&mut r, &wall_set) else {
        return Ok(None);
    };
    Ok(Some(MapPrefix {
        header: MapHeader {
            version,
            seed,
            height,
            width,
            tile_count,
            wall_count,
            liquid_count,
            sky_count,
            dirt_count,
            rock_count,
        },
        world_name,
        tile_options,
        wall_options,
        body_offset: r.pos,
    }))
}

/// A bounds-checked cursor over the inflated body; short reads become `Err`.
struct BodyReader<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> BodyReader<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, pos: 0 }
    }
    fn u8(&mut self) -> std::io::Result<u8> {
        let b = *self.bytes.get(self.pos).ok_or_else(eof)?;
        self.pos += 1;
        Ok(b)
    }
    fn u16(&mut self) -> std::io::Result<u16> {
        Ok(self.u8()? as u16 | ((self.u8()? as u16) << 8))
    }
}

/// `LoadMapVersion2` 3930-4025 — decodes the already-inflated body.
///
/// `world_surface` / `rock_layer` are `Main.worldSurface` / `Main.rockLayer`,
/// which the source reads as globals in the sky (`3999`) and dirt/rock (`4002`)
/// cases; they are passed in so `tr-core` stays engine-agnostic.
#[allow(clippy::too_many_arguments)]
pub fn decode_tiles(
    body: &[u8],
    header: &MapHeader,
    tile_options: &[u8],
    wall_options: &[u8],
    width: usize,
    height: usize,
    world_surface: f64,
    rock_layer: f64,
) -> std::io::Result<Vec<MapTile>> {
    let table = color_table();
    let n4 = header.tile_count as usize;
    let n5 = header.wall_count as usize;
    let n6 = header.liquid_count as usize;
    let n7 = header.sky_count as usize;
    let n8 = header.dirt_count as usize;
    let n9 = header.rock_count as usize;

    // `array5` remap (`3839-3928`): file index -> our `colorLookup` index.
    let num10: usize = tile_options.iter().map(|&c| c as usize).sum();
    let num11: usize = wall_options.iter().map(|&c| c as usize).sum();
    let mut array5 = vec![0u16; num10 + num11 + n6 + n7 + n8 + n9 + 2];
    let mut num12: u16 = 1;
    let mut num13: usize = 1;
    let num14 = num13 as u16; // tile region start
    for i in 0..TILE_COUNT {
        if i < n4 {
            let file_count = tile_options.get(i).copied().unwrap_or(0) as usize;
            let our_count = table.tile_option_counts[i] as usize;
            for j in 0..our_count {
                if j < file_count {
                    *array5.get_mut(num13).ok_or_else(eof)? = num12;
                    num13 += 1;
                }
                num12 += 1;
            }
        } else {
            num12 += table.tile_option_counts.get(i).copied().unwrap_or(0) as u16;
        }
    }
    let num17 = num13 as u16; // wall region start
    for i in 0..WALL_COUNT {
        if i < n5 {
            let file_count = wall_options.get(i).copied().unwrap_or(0) as usize;
            let our_count = table.wall_option_counts[i] as usize;
            for j in 0..our_count {
                if j < file_count {
                    *array5.get_mut(num13).ok_or_else(eof)? = num12;
                    num13 += 1;
                }
                num12 += 1;
            }
        } else {
            num12 += table.wall_option_counts.get(i).copied().unwrap_or(0) as u16;
        }
    }
    let num20 = num13 as u16; // liquid region start
    for i in 0..4 {
        if i < n6 {
            *array5.get_mut(num13).ok_or_else(eof)? = num12;
            num13 += 1;
        }
        num12 += 1;
    }
    let num21 = num13 as u16; // sky region start
    for i in 0..256 {
        if i < n7 {
            *array5.get_mut(num13).ok_or_else(eof)? = num12;
            num13 += 1;
        }
        num12 += 1;
    }
    let num22 = num13 as u16; // dirt region start
    for i in 0..256 {
        if i < n8 {
            *array5.get_mut(num13).ok_or_else(eof)? = num12;
            num13 += 1;
        }
        num12 += 1;
    }
    let num23 = num13 as u16; // rock region start
    for i in 0..256 {
        if i < n9 {
            *array5.get_mut(num13).ok_or_else(eof)? = num12;
            num13 += 1;
        }
        num12 += 1;
    }
    let num24 = num13 as u16; // hell position
    *array5.get_mut(num13).ok_or_else(eof)? = num12;

    let mut reader = BodyReader::new(body);
    let mut map = vec![MapTile::default(); width * height];
    for l in 0..height {
        let mut m = 0usize;
        // The C# `for (m = 0; m < maxTilesX; m++)`; run tiles advance `m` inside
        // the body and the loop increment adds the final `+ 1` (kept as a `while`).
        while m < width {
            let b3 = reader.u8()?;
            let mut b4 = 0u8;
            if b3 & 1 == 1 {
                b4 = reader.u8()?;
            }
            if b4 & 1 == 1 {
                // Legacy format carried a third header byte; never written by us.
                let _ = reader.u8()?;
            }
            let category = (b3 & 0x0E) >> 1;
            let has_type = matches!(category, 1 | 2 | 7);
            let mut ty: u16 = if has_type {
                if b3 & 0x10 != 0 {
                    reader.u16()?
                } else {
                    reader.u8()? as u16
                }
            } else {
                0
            };
            let light = if b3 & 0x20 != 0 {
                reader.u8()?
            } else {
                255
            };
            let run = match (b3 & 0xC0) >> 6 {
                0 => 0usize,
                1 => reader.u8()? as usize,
                _ => reader.u16()? as usize,
            };

            match category {
                0 => {
                    // `3977-3979`: light <= 18 skip-run, left unrevealed.
                    m += run + 1;
                    continue;
                }
                1 => ty += num14,
                2 => ty += num17,
                3 | 4 | 5 => {
                    let mut liquid = (category - 3) as u16;
                    if b4 & 0x40 == 0x40 {
                        liquid = 3;
                    }
                    ty = num20 + liquid;
                }
                6 => {
                    if (l as f64) < world_surface {
                        // `CalcSkyGradient` (`2078-2081`), with the file's counts.
                        let gradient = ((n7.saturating_sub(1)) as f64
                            * (l as f64 / world_surface))
                            as u8 as u16;
                        ty = num21 + gradient;
                    } else {
                        ty = num24;
                    }
                }
                7 => {
                    ty += if (l as f64) < rock_layer { num22 } else { num23 };
                }
                _ => {}
            }

            let tile = MapTile::create(
                *array5.get(ty as usize).ok_or_else(eof)?,
                light,
                (b4 >> 1) & 0x1F,
            );
            *map.get_mut(l * width + m).ok_or_else(eof)? = tile;
            if light == 255 {
                // `4007-4016`: a 255-light run repeats the same tile.
                let mut remaining = run;
                while remaining > 0 {
                    m += 1;
                    *map.get_mut(l * width + m).ok_or_else(eof)? = tile;
                    remaining -= 1;
                }
            } else {
                // `4017-4023`: an `EqualsWithoutLight` run reads each light.
                let mut remaining = run;
                while remaining > 0 {
                    m += 1;
                    let run_tile = tile.with_light(reader.u8()?);
                    *map.get_mut(l * width + m).ok_or_else(eof)? = run_tile;
                    remaining -= 1;
                }
            }
            m += 1; // the `for` increment
        }
    }
    Ok(map)
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

    // ---- Task 3: MapTile + CreateMapTile + colour ----

    use crate::tile::{Tile, FLAG_ACTIVE};
    use crate::world::World;

    fn blank_map(world: &World) -> Vec<MapTile> {
        vec![MapTile::default(); (world.width * world.height) as usize]
    }

    fn active_stone(world: &mut World, x: i32, y: i32) -> &mut Tile {
        let t = world.tile_mut(x, y).unwrap();
        t.kind = 1; // TileID.Stone, map colour [128, 128, 128]
        t.flags |= FLAG_ACTIVE;
        t
    }

    #[test]
    fn stone_tile_maps_to_its_lookup() {
        let mut world = World::new("test".into(), 20, 20);
        active_stone(&mut world, 5, 5);
        let map = blank_map(&world);
        let mt = create_map_tile(&world, &map, 5, 5, 255, 0);
        assert_eq!(mt.ty, color_table().tile_lookup[1]);
        assert_ne!(mt.ty, 0);
    }

    #[test]
    fn unlit_tile_is_not_revealed() {
        let mut world = World::new("test".into(), 20, 20);
        active_stone(&mut world, 5, 5);
        let map = blank_map(&world);
        let mt = create_map_tile(&world, &map, 5, 5, 0, 0);
        assert_eq!(mt.light, 0);
        assert!(!is_revealed(mt));
    }

    #[test]
    fn wall_only_tile_uses_a_wall_colour() {
        let mut world = World::new("test".into(), 20, 20);
        world.tile_mut(5, 5).unwrap().wall = 1; // WallID.Stone
        let map = blank_map(&world);
        let mt = create_map_tile(&world, &map, 5, 5, 255, 0);
        let table = color_table();
        assert!(
            mt.ty >= table.wall_range_start && mt.ty <= table.wall_range_end,
            "ty {} not in wall range {}..={}",
            mt.ty,
            table.wall_range_start,
            table.wall_range_end
        );
    }

    #[test]
    fn water_maps_to_a_liquid_colour() {
        let mut world = World::new("test".into(), 20, 20);
        world.tile_mut(5, 5).unwrap().liquid = 200;
        let map = blank_map(&world);
        let mt = create_map_tile(&world, &map, 5, 5, 255, 0);
        assert_eq!(mt.ty, color_table().liquid_position);
    }

    #[test]
    fn surface_air_uses_the_sky_gradient() {
        let world = World::new("test".into(), 20, 1000); // world_surface == 250
        let map = blank_map(&world);
        let j = 100;
        let mt = create_map_tile(&world, &map, 5, j, 0, 0);
        // CalcSkyGradient (MapHelper.cs:2078): (byte)(255 * (j / worldSurface)).
        let expected =
            color_table().sky_position + (255.0 * (j as f64 / world.world_surface)) as u8 as u16;
        assert_eq!(mt.ty, expected);
        // The surface branch sets newLight = 255 (MapHelper.cs:2036).
        assert_eq!(mt.light, 255);
    }

    #[test]
    fn deep_air_uses_dirt_then_rock_then_hell() {
        let world = World::new("test".into(), 20, 2000); // surface 500, rock 900
        let map = blank_map(&world);
        let table = color_table();

        let dirt = create_map_tile(&world, &map, 5, 600, 255, 0);
        assert!(
            dirt.ty >= table.dirt_position && dirt.ty < table.dirt_position + 256,
            "dirt ty {}",
            dirt.ty
        );

        let rock = create_map_tile(&world, &map, 5, 1000, 255, 0);
        assert!(
            rock.ty >= table.rock_position && rock.ty < table.rock_position + 256,
            "rock ty {}",
            rock.ty
        );

        // UnderworldLayer == height - 200 (Main.cs:2790), so j == 1900 >= 1800.
        let hell = create_map_tile(&world, &map, 5, 1900, 255, 0);
        assert_eq!(hell.ty, table.hell_position);
    }

    #[test]
    fn negative_paint_inverts_the_colour() {
        // Grey stone [128,128,128] with paint 30 → MapHelper.cs:1851-1863 inverts.
        let mut world = World::new("test".into(), 20, 20);
        active_stone(&mut world, 5, 5).set_paint(30);
        let map = blank_map(&world);
        let mt = create_map_tile(&world, &map, 5, 5, 255, 0);
        assert_eq!(mt.color(), 30);
        assert_eq!(get_map_tile_color(mt, 5, 5), [127, 127, 127]);
    }

    #[test]
    fn paint_multiplies_by_max_channel() {
        // Paint 1 is red; only the red channel survives (MapHelper.cs:1865-1872).
        let mut world = World::new("test".into(), 20, 20);
        active_stone(&mut world, 5, 5).set_paint(1);
        let map = blank_map(&world);
        let mt = create_map_tile(&world, &map, 5, 5, 255, 0);
        assert_eq!(mt.color(), 1);
        assert_eq!(get_map_tile_color(mt, 5, 5), [128, 0, 0]);
    }

    #[test]
    fn painted_wall_half_inverts() {
        // A wall-range type with paint 30 → (255 - c) / 2 (MapHelper.cs:1852-1856).
        let table = color_table();
        let ty = table.wall_lookup[1]; // WallID.Stone == [52, 52, 52]
        assert_eq!(table.color(ty), [52, 52, 52]);
        let mt = MapTile::create(ty, 255, 30);
        assert_eq!(get_map_tile_color(mt, 5, 5), [101, 101, 101]);
    }

    #[test]
    fn fully_lit_returns_the_raw_colour() {
        let ty = color_table().tile_lookup[1];
        let mt = MapTile::create(ty, 255, 0);
        assert_eq!(get_map_tile_color(mt, 0, 0), color_table().color(ty));
        assert_eq!(get_map_tile_color(mt, 0, 0), [128, 128, 128]);
    }

    #[test]
    fn changed_bit_is_set_by_create_and_with_light() {
        let created = MapTile::create(1, 2, 3);
        assert!(created.is_changed());
        assert_eq!(created.color(), 3);
        assert!(MapTile::default().with_light(7).is_changed());
    }

    // ---- Task 3b: GetTileBaseOption ----

    fn option_world() -> World {
        World::new("test".into(), 20, 20)
    }

    fn framed(kind: u16, frame_x: i16, frame_y: i16) -> Tile {
        Tile {
            kind,
            frame_x,
            frame_y,
            ..Default::default()
        }
    }

    /// `(x, y)` fixed at `(5, 5)`; only the `(x + y) % 9` cases read them.
    fn base_option_of(kind: u16, frame_x: i16, frame_y: i16) -> u8 {
        let world = option_world();
        let tile = framed(kind, frame_x, frame_y);
        get_tile_base_option(5, 5, kind, &tile, &world)
    }

    #[test]
    fn tile_518_options_use_frame_row() {
        // MapHelper.cs:2199-2202: `baseOption = frameY / 18`. (The task brief
        // wrote `frameY = 36 -> 1`; `36 / 18 == 2`, so the source-true values
        // are used here.)
        assert_eq!(base_option_of(518, 0, 0), 0);
        assert_eq!(base_option_of(518, 0, 18), 1);
        assert_eq!(base_option_of(518, 0, 36), 2);
    }

    #[test]
    fn tile_19_options_use_the_waterfall_row() {
        // MapHelper.cs:2179-2188: `frameY / 18 == 48`.
        assert_eq!(base_option_of(19, 0, 864), 1);
        assert_eq!(base_option_of(19, 0, 0), 0);
    }

    #[test]
    fn tile_21_options_use_frame_column() {
        // MapHelper.cs:2213-2239: `switch (frameX / 36)`.
        assert_eq!(base_option_of(21, 0, 0), 0);
        assert_eq!(base_option_of(21, 36, 0), 1);
        assert_eq!(base_option_of(21, 108, 0), 2); // 3
        assert_eq!(base_option_of(21, 216, 0), 3); // 6
        assert_eq!(base_option_of(21, 396, 0), 4); // 11
    }

    #[test]
    fn tile_89_options_special_case() {
        // MapHelper.cs:2087-2102: `switch (frameX / 54)`.
        assert_eq!(base_option_of(89, 0, 0), 0);
        assert_eq!(base_option_of(89, 54, 0), 1); // default
        assert_eq!(base_option_of(89, 43 * 54, 0), 2); // case 43
    }

    #[test]
    fn tile_160_repeats_every_nine() {
        // MapHelper.cs:2103-2108: `(x + y) % 9`.
        let world = option_world();
        let tile = framed(160, 0, 0);
        assert_eq!(get_tile_base_option(4, 5, 160, &tile, &world), 0); // 9 % 9
        assert_eq!(get_tile_base_option(4, 6, 160, &tile, &world), 1); // 10 % 9
        assert_eq!(get_tile_base_option(2, 1, 160, &tile, &world), 3); // 3 % 9
    }

    #[test]
    fn tile_4_always_zero() {
        // MapHelper.cs:2203-2209: assigns 1 then unconditionally 0.
        assert_eq!(base_option_of(4, 0, 0), 0);
        assert_eq!(base_option_of(4, 999, 0), 0);
    }

    #[test]
    fn unknown_tile_is_zero() {
        // MapHelper.cs:3001-3003: default.
        assert_eq!(base_option_of(12345, 0, 0), 0);
    }

    #[test]
    fn zone_dependent_cases_default_to_zero() {
        // Substituted in the port: 461 (SceneMetrics zones), 80
        // (WorldGen.GetCactusType), 529/530 (WorldGen.GetBiomeInfluence).
        for kind in [461u16, 529, 530, 80] {
            assert_eq!(base_option_of(kind, 1234, 567), 0, "type {kind}");
        }
    }

    #[test]
    fn get_tile_type_uses_the_option() {
        let mut world = World::new("test".into(), 20, 20);
        {
            let t = world.tile_mut(5, 5).unwrap();
            t.kind = 518;
            t.frame_y = 18; // frameY / 18 == 1
            t.flags |= FLAG_ACTIVE;
        }
        let map = blank_map(&world);
        let mt = create_map_tile(&world, &map, 5, 5, 255, 0);
        assert_eq!(mt.ty, color_table().tile_lookup[518] + 1);
    }

    // ---- Task 4: `.map` save/load ----

    fn map_test_world() -> World {
        let mut w = World::new("Terrustaria".into(), 40, 40);
        w.seed = 0x1234_5678_9abc_def0;
        w.world_surface = 10.0;
        w.rock_layer = 18.0;
        w
    }

    fn map_test_header(world: &World) -> MapHeader {
        MapHeader {
            version: MAP_FORMAT_VERSION,
            seed: world.seed as i32,
            height: world.height,
            width: world.width,
            tile_count: TILE_COUNT as i16,
            wall_count: WALL_COUNT as i16,
            liquid_count: 4,
            sky_count: 256,
            dirt_count: 256,
            rock_count: 256,
        }
    }

    /// The per-entry option counts a file we wrote carries (`array3`/`array4`).
    fn map_test_options() -> (Vec<u8>, Vec<u8>) {
        let t = color_table();
        (t.tile_option_counts.clone(), t.wall_option_counts.clone())
    }

    fn put(map: &mut [MapTile], width: usize, x: usize, y: usize, tile: MapTile) {
        map[y * width + x] = tile;
    }

    /// A 40x40 map exercising every RLE branch: a `Light <= 18` skip-run (default
    /// tiles), a `Light == 255` run, an equal type+colour run with differing light,
    /// a type offset `> 255`, a painted tile, and the wall/liquid regions.
    fn map_test_tiles(width: usize, height: usize) -> Vec<MapTile> {
        let t = color_table();
        let mut map = vec![MapTile::default(); width * height];
        // x0..x3 stay `default` (light 0 <= 18): one skip-run.
        for x in 4..=6 {
            put(&mut map, width, x, 0, MapTile::create(t.tile_position + 2, 255, 0));
        }
        for (x, light) in [(7usize, 30u8), (8, 60), (9, 90)] {
            put(&mut map, width, x, 0, MapTile::create(t.tile_position + 3, light, 0));
        }
        put(&mut map, width, 10, 0, MapTile::create(t.tile_position + 4, 100, 7));
        put(&mut map, width, 11, 0, MapTile::create(t.tile_position + 300, 255, 0));
        put(&mut map, width, 12, 0, MapTile::create(t.wall_position, 200, 0));
        put(&mut map, width, 13, 0, MapTile::create(t.liquid_position, 200, 0));
        put(&mut map, width, 14, 0, MapTile::create(t.liquid_position + 3, 200, 0));
        map
    }

    #[test]
    fn body_round_trips() {
        let world = map_test_world();
        let (w, h) = (40usize, 40usize);
        let original = map_test_tiles(w, h);
        let header = map_test_header(&world);
        let (tile_options, wall_options) = map_test_options();
        let body = encode_tiles(&original, w, h);
        let decoded = decode_tiles(
            &body,
            &header,
            &tile_options,
            &wall_options,
            w,
            h,
            world.world_surface,
            world.rock_layer,
        )
        .expect("body must decode");
        assert_eq!(decoded.len(), original.len());
        for (i, (a, b)) in original.iter().zip(decoded.iter()).enumerate() {
            assert_eq!(a.ty, b.ty, "ty at {i}");
            assert_eq!(a.light, b.light, "light at {i}");
            assert_eq!(a.color(), b.color(), "color at {i}");
            assert_eq!(a.is_changed(), b.is_changed(), "changed at {i}");
        }
        // Light <= 18 tiles are never saved: the skip-run decodes to `default`.
        assert_eq!(decoded[0], MapTile::default());
        assert_eq!(decoded[2], MapTile::default());
    }

    #[test]
    fn variable_light_run_flush_to_row_end_round_trips() {
        let world = map_test_world();
        let (w, h) = (6usize, 1usize);
        let t = color_table();
        // A width-6 row that is one type+colour with distinct lights in 19..=254,
        // so the `EqualsWithoutLight` run ends exactly on the last column (the
        // row-end case where the decompiled `num6` stays 0).
        let mut map = vec![MapTile::default(); w * h];
        for (x, light) in [(0usize, 20u8), (1, 30), (2, 40), (3, 50), (4, 60), (5, 70)] {
            put(&mut map, w, x, 0, MapTile::create(t.tile_position + 5, light, 0));
        }
        let header = map_test_header(&world);
        let (tile_options, wall_options) = map_test_options();
        let body = encode_tiles(&map, w, h);
        let decoded = decode_tiles(
            &body,
            &header,
            &tile_options,
            &wall_options,
            w,
            h,
            world.world_surface,
            world.rock_layer,
        )
        .expect("a row-end variable-light run must round-trip");
        assert_eq!(decoded, map);
    }

    #[test]
    fn background_categories_round_trip() {
        let world = map_test_world(); // world_surface 10, rock_layer 18
        let (w, h) = (40usize, 40usize);
        let t = color_table();
        let mut map = vec![MapTile::default(); w * h];
        let sky = t.sky_position + (255.0 * (2.0 / world.world_surface)) as u8 as u16;
        put(&mut map, w, 5, 2, MapTile::create(sky, 255, 0));
        put(&mut map, w, 5, 12, MapTile::create(t.dirt_position + 40, 255, 0));
        put(&mut map, w, 5, 25, MapTile::create(t.rock_position + 40, 255, 0));
        put(&mut map, w, 5, 30, MapTile::create(t.hell_position, 255, 0));

        let header = map_test_header(&world);
        let (tile_options, wall_options) = map_test_options();
        let body = encode_tiles(&map, w, h);
        let decoded = decode_tiles(
            &body,
            &header,
            &tile_options,
            &wall_options,
            w,
            h,
            world.world_surface,
            world.rock_layer,
        )
        .expect("body must decode");
        assert_eq!(decoded[2 * w + 5].ty, sky, "sky row");
        assert_eq!(decoded[12 * w + 5].ty, t.dirt_position + 40, "dirt row");
        assert_eq!(decoded[25 * w + 5].ty, t.rock_position + 40, "rock row");
        assert_eq!(decoded[30 * w + 5].ty, t.hell_position, "hell row");
    }

    #[test]
    fn header_is_version_326_and_world_size() {
        let world = map_test_world();
        let mut bytes = write_map_header(&world);
        bytes.extend_from_slice(&write_option_tables());
        let prefix = read_map_prefix(&bytes)
            .expect("read must not error")
            .expect("prefix must parse");
        assert_eq!(prefix.header.version, 326);
        assert_eq!(prefix.header.seed, world.seed as i32);
        assert_eq!(prefix.header.width, world.width);
        assert_eq!(prefix.header.height, world.height);
        assert_eq!(prefix.world_name, "Terrustaria");
        // The tables are the whole prefix, so the body starts right after them.
        assert_eq!(prefix.body_offset, bytes.len());
        assert_eq!(prefix.tile_options, color_table().tile_option_counts);
        // A fresh metadata revision is always 1, so the header is deterministic.
        assert_eq!(write_map_header(&world), write_map_header(&world));
    }

    #[test]
    fn foreign_version_is_rejected() {
        let world = map_test_world();
        let mut bytes = write_map_header(&world);
        bytes.extend_from_slice(&write_option_tables());
        bytes[0..4].copy_from_slice(&999u32.to_le_bytes());
        assert!(read_map_prefix(&bytes).expect("no error").is_none());
    }

    #[test]
    fn negative_table_count_is_rejected() {
        let world = map_test_world();
        let mut bytes = write_map_header(&world);
        bytes.extend_from_slice(&write_option_tables());
        // `sky_count` is the 4th of the six i16 counts. Its offset is the fixed
        // header layout: version(4) + FileMetadata(8) + revision(4) + favourite(8)
        // + name (7-bit length byte + ASCII) + seed(4) + height(4) + width(4), then
        // the counts. Patch it to -1; a negative i16 cast to `usize` would size
        // `decode_tiles`' `array5` from a huge value and panic on capacity overflow.
        let name_field = 1 + world.name.len();
        let sky_off = 4 + 8 + 4 + 8 + name_field + 4 + 4 + 4 + 3 * 2;
        bytes[sky_off..sky_off + 2].copy_from_slice(&(-1i16).to_le_bytes());

        let parsed = read_map_prefix(&bytes).expect("read must not error");
        assert!(parsed.is_none(), "a negative table count must be rejected");

        // The unpatched prefix still parses, so the test targets the count only.
        let mut ok = write_map_header(&world);
        ok.extend_from_slice(&write_option_tables());
        assert!(read_map_prefix(&ok).expect("no error").is_some());
    }

    #[test]
    fn truncated_prefix_is_rejected() {
        let short = vec![0u8; 39];
        assert!(read_map_prefix(&short).expect("no error").is_none());
    }

    #[test]
    fn truncated_body_is_rejected() {
        let world = map_test_world();
        let (w, h) = (40usize, 40usize);
        let original = map_test_tiles(w, h);
        let header = map_test_header(&world);
        let (tile_options, wall_options) = map_test_options();
        let mut body = encode_tiles(&original, w, h);
        body.truncate(body.len() / 2);
        let result = decode_tiles(
            &body,
            &header,
            &tile_options,
            &wall_options,
            w,
            h,
            world.world_surface,
            world.rock_layer,
        );
        assert!(result.is_err(), "a cut body must be rejected");
    }

    #[test]
    #[ignore]
    fn loads_a_real_vanilla_map() {
        // Manual verification: set `TR_VANILLA_MAP` to a vanilla `.map`. `tr-core`
        // can only check the uncompressed prefix (the body is deflate-wrapped and
        // `flate2` is deliberately not a `tr-core` dependency); the full decode is
        // the `#[ignore] loads_a_real_vanilla_map` test in `tr-app`, which counts
        // the revealed tiles.
        let Ok(path) = std::env::var("TR_VANILLA_MAP") else {
            return;
        };
        let bytes = std::fs::read(path).expect("read map");
        let prefix = read_map_prefix(&bytes)
            .expect("no error")
            .expect("vanilla prefix must parse");
        assert_eq!(prefix.header.version, MAP_FORMAT_VERSION);
        assert!(
            prefix.body_offset < bytes.len(),
            "vanilla map has an empty body"
        );
    }
}
