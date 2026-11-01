//! Tile flames: the per-type flame-sheet data, the flame-spread quads and the
//! glow-mask overlays of `DrawSingleTile_Flames`.
//!
//! Three pieces of `Terraria.GameContent.Drawing.TileDrawing` live here:
//!
//! * [`tile_flame_data`] — `GetTileFlameData` (`TileDrawing.cs:1847-2677`), the
//!   per-type flame sheet / count / colour / spread table.
//! * [`flame_quads`] — the inline flame branch of `DrawSingleTile_Flames`
//!   (`TileDrawing.cs:2826-3462`), which duplicates the type→sheet switch and
//!   hard-codes its own per-style loops. The two C# tables disagree in a few
//!   places (colour of style 13/14, discarded offsets for sheet 7); both are
//!   ported literally and kept separate on purpose.
//! * [`glow_quads`] — the remaining `DrawSingleTile_Flames` branches
//!   (`TileDrawing.cs:2684-2825`, `:3463-3491`): the per-type glow-mask /
//!   jar / campfire / sundial overlays. Not ported here: type 85's graveyard
//!   flames (`:2724-2750`) need `Main.GraveyardVisualIntensity`, which the
//!   port does not model (`ZoneGraveyard` is not simulated), and the separate
//!   `Main.tileGlowMask` table (`Main.cs:10024+`, consumed inside
//!   `DrawSingleTile` at `TileDrawing.cs:1178`) is its own wide task.
//!
//! The shared 48-bit LCG behind `Utils.RandomInt` is also here because the
//! flame spread is its only tile-render consumer. `Main.TileFrameSeed` is the
//! seed source: C# initialises it from `Guid.NewGuid().GetHashCode()`
//! (`Main.cs:66567`) — unobservable randomness — and advances it every fifth
//! draw (`Main.cs:61716-61721`). The port keeps the same advance schedule and
//! starts from a fixed 32-bit constant, so flame jitter is deterministic
//! across runs while staying distribution-identical to the original.

use crate::drawdata::TileDrawData;

/// `Main.TileFrameSeed`'s port start value. C# uses
/// `(ulong)Guid.NewGuid().GetHashCode()` (`Main.cs:66567`) — any 32-bit value;
/// this one is fixed so two runs render identical frames.
pub const TILE_FRAME_SEED_INIT: u64 = 0x5F37_59DF;

/// `Utils.RandomNextSeed` (`Utils.cs:2290-2293`): one step of the shared
/// 48-bit LCG (`seed * 25214903917 + 11`, masked to 48 bits).
#[inline]
pub fn random_next_seed(seed: u64) -> u64 {
    seed.wrapping_mul(25214903917).wrapping_add(11) & 0xFFFF_FFFF_FFFF
}

/// `Utils.RandomNext(ref seed, bits)` (`Utils.cs:2295-2299`).
#[inline]
fn random_next(seed: &mut u64, bits: u32) -> i32 {
    *seed = random_next_seed(*seed);
    (*seed >> (48 - bits)) as u32 as i32
}

/// `Utils.RandomInt(ref seed, max)` (`Utils.cs:2310-2331`).
fn random_int_max(seed: &mut u64, max: i32) -> i32 {
    if (max & max.wrapping_neg()) == max {
        // C#: `(int)((long)max * (long)RandomNext(seed, 31) >> 31)`.
        return (((max as i64) * (random_next(seed, 31) as i64)) >> 31) as i32;
    }
    loop {
        let num = random_next(seed, 31);
        let num2 = num % max;
        if num.wrapping_sub(num2).wrapping_add(max - 1) >= 0 {
            return num2;
        }
    }
}

/// `Utils.RandomInt(ref seed, min, max)` (`Utils.cs:2332-2335`).
#[inline]
pub fn random_int(seed: &mut u64, min: i32, max: i32) -> i32 {
    random_int_max(seed, max - min) + min
}

/// `Main.TileFrameSeed ^ ((long)tileX << 32 | (uint)tileY)`
/// (`TileDrawing.cs:1878`, consumed at `:2828` as well).
#[inline]
pub fn frame_seed_for(tile_frame_seed: u64, tile_x: i32, tile_y: i32) -> u64 {
    tile_frame_seed ^ (((tile_x as i64) << 32) as u64 | tile_y as i32 as u64)
}

/// `Main.DoDraw`'s seed schedule (`Main.cs:61716-61721`) rebuilt as a pure
/// function of the animation clock: C# advances `Main.TileFrameSeed` once
/// every fifth rendered frame (`_drawCycleCounter`), which at the game's 60 fps
/// is 12 advances per second. The port has no deterministic frame counter, so
/// the pass derives the seed from `Main.animTime`-equivalent seconds instead —
/// same LCG, same cadence, and the flame jitter stays a pure function of time.
pub fn tile_frame_seed_at(anim_time: f32) -> u64 {
    let steps = (anim_time.max(0.0) * 12.0) as u64;
    let mut seed = TILE_FRAME_SEED_INIT;
    for _ in 0..steps {
        seed = random_next_seed(seed);
    }
    seed
}

/// The `Main`-state snapshot the flame and glow passes consume
/// (`DrawSingleTile_Flames` reads these straight off `Main`).
#[derive(Clone, Copy, Debug)]
pub struct FlameEnv {
    /// `Main.TileFrameSeed` for this frame ([`tile_frame_seed_at`]).
    pub tile_frame_seed: u64,
    /// `Main.GlobalTimeWrappedHourly` (seconds mod 3600).
    pub global_time: f32,
    /// `Main.DiscoR/G/B` (the rainbow torch colour cycle).
    pub disco: [u8; 3],
    /// `Main.mouseTextColor` (190..=255): the moondial/teleporter pulse.
    pub mouse_text_color: u8,
    /// `Main.moonPhase` (0..7): the moondial glow's source row.
    pub moon_phase: u8,
    /// `Main.sundialCooldown`.
    pub sundial_cooldown: i32,
    /// `Main.moondialCooldown`.
    pub moondial_cooldown: i32,
}

/// `Main.tileFlame` (`Main.cs:9300-9313`): tiles that draw the animated
/// `Images/Flame_N` overlay in `DrawSingleTile_Flames`.
const TILE_FLAME: &[u16] = &[4, 33, 34, 35, 42, 49, 93, 98, 100, 173, 174, 372, 646];

/// The tile types with a glow branch in `DrawSingleTile_Flames` outside the
/// `Main.tileFlame` table (`TileDrawing.cs:2686-2825`, `:3463-3491`). Type 85
/// (the graveyard tombstone flames, `:2724-2750`) is excluded — see
/// [`glow_quads`].
const GLOW_BRANCH: &[u16] = &[
    548, 613, 614, 593, 594, 215, 356, 663, 286, 582, 391, 619, 270, 271, 581, 316, 317, 318,
    149, 300, 302, 303, 306, 144, 237, 658,
];

/// `Main.tileFlame[type]`.
#[inline]
pub fn tile_flame(kind: u16) -> bool {
    TILE_FLAME.contains(&kind)
}

/// Whether [`glow_quads`] can emit anything for this tile type (the cheap
/// per-tile early-out of the render pass).
#[inline]
pub fn has_glow_branch(kind: u16) -> bool {
    GLOW_BRANCH.contains(&kind)
}

/// The `int num = 0; switch (type)` selection of the `Images/Flame_N` sheet
/// (`TileDrawing.cs:1880-1897`, duplicated at `:2830-2869`).
#[inline]
pub fn flame_sheet(kind: u16) -> usize {
    match kind {
        4 => 0,
        33 | 174 => 1,
        100 | 173 => 2,
        34 => 3,
        93 => 4,
        49 => 5,
        98 => 6,
        35 => 7,
        42 => 13,
        372 => 16,
        646 => 17,
        _ => 0,
    }
}

/// `TileDrawing.TileFlameData` (`TileDrawing.cs:37-60`), with `flameTexture`
/// replaced by the `Images/Flame_N` sheet index and colours normalised.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TileFlameData {
    /// `TextureAssets.Flames[num]` sheet index.
    pub sheet: usize,
    /// `flameSeed`.
    pub seed: u64,
    /// `flameCount`.
    pub count: i32,
    /// `flameColor` as a normalized RGBA tint.
    pub color: [f32; 4],
    /// `flameRangeXMin/Max`.
    pub rx: (i32, i32),
    /// `flameRangeYMin/Max`.
    pub ry: (i32, i32),
    /// `flameRangeMultX`.
    pub mx: f32,
    /// `flameRangeMultY`.
    pub my: f32,
}

impl Default for TileFlameData {
    /// C# `default(TileFlameData)`: sheet 0, zero seed/count, `Color(0,0,0,0)`,
    /// zero ranges and multipliers (`TileDrawing.cs:37-60`).
    fn default() -> Self {
        Self {
            sheet: 0,
            seed: 0,
            count: 0,
            color: [0.0; 4],
            rx: (0, 0),
            ry: (0, 0),
            mx: 0.0,
            my: 0.0,
        }
    }
}

/// `Color(r, g, b, a)` as the normalized tint the renderer consumes.
#[inline]
fn col(r: u8, g: u8, b: u8, a: u8) -> [f32; 4] {
    [
        r as f32 / 255.0,
        g as f32 / 255.0,
        b as f32 / 255.0,
        a as f32 / 255.0,
    ]
}

/// `Utils.WrappedLerp` (`Utils.cs:305-312`): a triangle wave between
/// `value1` and `value2`.
fn wrapped_lerp(value1: f32, value2: f32, percent: f32) -> f32 {
    let mut num = percent * 2.0;
    if num > 1.0 {
        num = 2.0 - num;
    }
    // `MathHelper.Lerp`.
    value1 + (value2 - value1) * num
}

/// `TileDrawing.GetFallenStarFurnitureFlameColor` (`TileDrawing.cs:1809-1814`).
fn fallen_star_furniture_flame_color(global_time: f32) -> [f32; 4] {
    let num = (150.0 * wrapped_lerp(0.5, 1.0, global_time % 2.0 / 2.0)) as i32;
    [
        150.0 / 255.0,
        num as f32 / 255.0,
        num as f32 / 255.0,
        50.0 / 255.0,
    ]
}

/// `TileDrawing.GetHallowedFurnitureFlameColor` (`TileDrawing.cs:1816-1820`).
fn hallowed_furniture_flame_color(global_time: f32) -> [f32; 4] {
    let num = (170.0 * wrapped_lerp(0.5, 1.0, global_time % 2.0 / 2.0)) as i32;
    [
        170.0 / 255.0,
        num as f32 / 255.0,
        num as f32 / 255.0,
        75.0 / 255.0,
    ]
}

/// `TileDrawing.GetWrappedFurnitureFlameColor` (`TileDrawing.cs:1841-1845`):
/// `baseColor * WrappedLerp(min, max, GlobalTimeWrappedHourly % 2 / 2)`.
/// C# multiplies byte colours with truncation; the port multiplies the
/// normalized floats (at most 1/255 per channel apart).
fn wrapped_furniture_flame_color(base: [f32; 4], global_time: f32, min: f32) -> [f32; 4] {
    let w = wrapped_lerp(min, 1.0, global_time % 2.0 / 2.0);
    [base[0] * w, base[1] * w, base[2] * w, base[3] * w]
}

/// `GetCloudFurnitureFlameColor` (`:1822-1825`), `GetLibrarianFurnitureFlameColor`
/// (`:1827-1830`), `GetForbiddenFurnitureFlameColor` (`:1832-1835`),
/// `GetBoulderFurnitureFlameColor` (`:1836-1839`): white with a different
/// minimum. `style` 0 = cloud (0.75), 1 = librarian/forbidden/boulder (0.25).
fn wrapped_white_furniture_flame_color(style: u8, global_time: f32) -> [f32; 4] {
    wrapped_furniture_flame_color(
        [1.0, 1.0, 1.0, 0.0],
        global_time,
        match style {
            0 => 0.75,
            _ => 0.25,
        },
    )
}

/// One assigned row of the `GetTileFlameData` result table.
fn set(
    d: &mut TileFlameData,
    count: i32,
    color: [f32; 4],
    rx: (i32, i32),
    ry: (i32, i32),
    mx: f32,
    my: f32,
) {
    d.count = count;
    d.color = color;
    d.rx = rx;
    d.ry = ry;
    d.mx = mx;
    d.my = my;
}

/// The plain style groups shared by several sheet cases: a single steady flame
/// with the spread table filled in (count 1, multiplier 0) and the "steady,
/// zero-range" group whose ranges stay at the C# default (0, 0) — the RNG
/// consumption differs (max 0 takes the power-of-two branch), so the two are
/// kept apart.
fn steady(d: &mut TileFlameData, color: [f32; 4], y_max: i32) {
    set(d, 1, color, (-10, 11), (-10, y_max), 0.0, 0.0);
}

fn zero_range(d: &mut TileFlameData, color: [f32; 4]) {
    set(d, 1, color, (0, 0), (0, 0), 0.0, 0.0);
}

/// `TileDrawing.GetTileFlameData` (`TileDrawing.cs:1847-2677`).
///
/// `frame_x`/`frame_y` are `Main.tile[tileX, tileY].frameX/frameY` (the C#
/// method reads them straight off the tile except for the `tileFrameY`
/// parameter of the sheet-13 switch, which carries the same value).
/// `tile_frame_seed` is `Main.TileFrameSeed`; `global_time` is
/// `Main.GlobalTimeWrappedHourly`; `disco` is `Main.DiscoR/G/B`.
pub fn tile_flame_data(
    tile_x: i32,
    tile_y: i32,
    kind: u16,
    frame_x: i16,
    frame_y: i16,
    tile_frame_seed: u64,
    global_time: f32,
    disco: [u8; 3],
) -> TileFlameData {
    // `TileDrawing.cs:1849-1874`: the three jar types use their own textures and
    // never reach the flame-sheet table.
    match kind {
        270 => {
            return TileFlameData {
                sheet: 0,
                count: 1,
                color: col(200, 200, 200, 0),
                ..Default::default()
            };
        }
        271 => {
            return TileFlameData {
                sheet: 0,
                count: 1,
                color: col(200, 200, 200, 0),
                ..Default::default()
            };
        }
        581 => {
            return TileFlameData {
                sheet: 0,
                count: 1,
                color: col(200, 100, 100, 0),
                ..Default::default()
            };
        }
        _ => {}
    }
    // `TileDrawing.cs:1876-1879`: `if (!Main.tileFlame[type]) return default;`
    if !tile_flame(kind) {
        return TileFlameData::default();
    }
    // `TileDrawing.cs:1878`.
    let seed = frame_seed_for(tile_frame_seed, tile_x, tile_y);
    let sheet = flame_sheet(kind);
    let mut d = TileFlameData {
        sheet,
        seed,
        ..Default::default()
    };
    // `TileDrawing.cs:1923-2677`.
    match sheet {
        // `TileDrawing.cs:1925-1934` — sheet 7 (candelabra, type 35).
        7 => set(&mut d, 4, col(50, 50, 50, 0), (-10, 11), (-10, 10), 0.0, 0.0),
        // `TileDrawing.cs:1935-2074` — sheet 1 (chandeliers, types 33/174),
        // styled by `Main.tile[x, y].frameY / 22`.
        1 => match frame_y as i32 / 22 {
            5 | 6 | 7 | 10 => set(&mut d, 7, col(50, 50, 50, 0), (-10, 11), (-10, 11), 0.075, 0.075),
            8 => set(&mut d, 7, col(50, 50, 50, 0), (-10, 11), (-10, 11), 0.3, 0.3),
            12 => set(&mut d, 7, col(50, 50, 50, 0), (-10, 11), (-10, 1), 0.1, 0.15),
            14 => set(&mut d, 8, col(75, 75, 75, 0), (-10, 11), (-10, 11), 0.1, 0.1),
            16 => set(&mut d, 4, col(75, 75, 75, 0), (-10, 11), (-10, 11), 0.15, 0.15),
            27 | 28 => steady(&mut d, col(75, 75, 75, 0), 11),
            43 => set(&mut d, 1, fallen_star_furniture_flame_color(global_time), (0, 0), (0, 0), 0.0, 0.0),
            44 => set(&mut d, 3, col(200, 200, 200, 150), (-10, 11), (-10, 1), 0.15, 0.35),
            45 => set(&mut d, 1, hallowed_furniture_flame_color(global_time), (0, 0), (0, 0), 0.0, 0.0),
            56 => set(&mut d, 1, wrapped_white_furniture_flame_color(0, global_time), (0, 0), (0, 0), 0.0, 0.0),
            57 | 60 => zero_range(&mut d, col(200, 200, 200, 150)),
            58 => set(&mut d, 1, wrapped_white_furniture_flame_color(1, global_time), (0, 0), (0, 0), 0.0, 0.0),
            61 => set(&mut d, 1, wrapped_white_furniture_flame_color(1, global_time), (0, 0), (0, 0), 0.0, 0.0),
            63 => set(&mut d, 1, wrapped_white_furniture_flame_color(1, global_time), (0, 0), (0, 0), 0.0, 0.0),
            47 | 48 | 49 | 51 | 52 | 54 => d.count = 0,
            _ => set(&mut d, 7, col(100, 100, 100, 0), (-10, 11), (-10, 1), 0.15, 0.35),
        },
        // `TileDrawing.cs:2075-2211` — sheet 2 (types 100/173), styled by
        // `Main.tile[x, y].frameY / 36`.
        2 => match frame_y as i32 / 36 {
            3 => set(&mut d, 3, col(50, 50, 50, 0), (-10, 11), (-10, 11), 0.05, 0.15),
            6 => set(&mut d, 5, col(75, 75, 75, 0), (-10, 11), (-10, 11), 0.15, 0.15),
            9 => set(&mut d, 7, col(100, 100, 100, 0), (-10, 11), (-10, 11), 0.3, 0.3),
            11 => set(&mut d, 7, col(50, 50, 50, 0), (-10, 11), (-10, 1), 0.1, 0.15),
            13 => set(&mut d, 8, col(75, 75, 75, 0), (-10, 11), (-10, 11), 0.1, 0.1),
            28 | 29 => steady(&mut d, col(75, 75, 75, 0), 1),
            44 => set(&mut d, 1, fallen_star_furniture_flame_color(global_time), (0, 0), (0, 0), 0.0, 0.0),
            45 => set(&mut d, 3, col(200, 200, 200, 150), (-10, 11), (-10, 1), 0.15, 0.35),
            46 => set(&mut d, 1, hallowed_furniture_flame_color(global_time), (0, 0), (0, 0), 0.0, 0.0),
            57 => set(&mut d, 1, wrapped_white_furniture_flame_color(0, global_time), (0, 0), (0, 0), 0.0, 0.0),
            58 | 61 => zero_range(&mut d, col(200, 200, 200, 150)),
            59 => set(&mut d, 1, wrapped_white_furniture_flame_color(1, global_time), (0, 0), (0, 0), 0.0, 0.0),
            62 => set(&mut d, 1, wrapped_white_furniture_flame_color(1, global_time), (0, 0), (0, 0), 0.0, 0.0),
            64 => set(&mut d, 1, wrapped_white_furniture_flame_color(1, global_time), (0, 0), (0, 0), 0.0, 0.0),
            48 | 49 | 50 | 52 | 53 | 55 => d.count = 0,
            _ => set(&mut d, 7, col(100, 100, 100, 0), (-10, 11), (-10, 1), 0.15, 0.35),
        },
        // `TileDrawing.cs:2212-2366` — sheet 3 (chandeliers, type 34), styled by
        // `frameY / 54` plus `37 * (frameX / 108)` from `frameX >= 108`.
        3 => {
            let mut n = frame_y as i32 / 54;
            if frame_x >= 108 {
                n += 37 * (frame_x as i32 / 108);
            }
            match n {
                8 => set(&mut d, 7, col(50, 50, 50, 0), (-10, 11), (-10, 11), 0.075, 0.075),
                9 => set(&mut d, 3, col(50, 50, 50, 0), (-1, 1), (-1, 1), 2.0, 2.0),
                11 => set(&mut d, 7, col(50, 50, 50, 0), (-10, 11), (-10, 11), 0.3, 0.3),
                15 => set(&mut d, 7, col(50, 50, 50, 0), (-10, 11), (-10, 1), 0.1, 0.15),
                17 | 20 => set(&mut d, 7, col(50, 50, 50, 0), (-10, 11), (-10, 11), 0.075, 0.075),
                18 => set(&mut d, 8, col(75, 75, 75, 0), (-10, 11), (-10, 11), 0.1, 0.1),
                34 | 35 => steady(&mut d, col(75, 75, 75, 0), 11),
                50 => set(&mut d, 1, fallen_star_furniture_flame_color(global_time), (0, 0), (0, 0), 0.0, 0.0),
                51 => set(&mut d, 3, col(200, 200, 200, 150), (-10, 11), (-10, 1), 0.15, 0.35),
                52 => set(&mut d, 1, hallowed_furniture_flame_color(global_time), (0, 0), (0, 0), 0.0, 0.0),
                63 => set(&mut d, 1, wrapped_white_furniture_flame_color(0, global_time), (0, 0), (0, 0), 0.0, 0.0),
                64 | 67 => zero_range(&mut d, col(200, 200, 200, 150)),
                65 => set(&mut d, 1, wrapped_white_furniture_flame_color(1, global_time), (0, 0), (0, 0), 0.0, 0.0),
                68 => set(&mut d, 1, wrapped_white_furniture_flame_color(1, global_time), (0, 0), (0, 0), 0.0, 0.0),
                70 => set(&mut d, 1, wrapped_white_furniture_flame_color(1, global_time), (0, 0), (0, 0), 0.0, 0.0),
                54 | 55 | 56 | 58 | 59 | 61 => d.count = 0,
                _ => set(&mut d, 7, col(100, 100, 100, 0), (-10, 11), (-10, 1), 0.15, 0.35),
            }
        }
        // `TileDrawing.cs:2367-2524` — sheet 4 (type 93), styled by
        // `Main.tile[x, y].frameY / 54`.
        4 => match frame_y as i32 / 54 {
            1 => set(&mut d, 3, col(50, 50, 50, 0), (-10, 11), (-10, 11), 0.15, 0.15),
            2 | 4 => set(&mut d, 7, col(50, 50, 50, 0), (-10, 11), (-10, 11), 0.075, 0.075),
            3 => set(&mut d, 7, col(100, 100, 100, 0), (-10, 11), (-20, 1), 0.2, 0.35),
            5 => set(&mut d, 7, col(50, 50, 50, 0), (-10, 11), (-10, 11), 0.3, 0.3),
            9 => set(&mut d, 7, col(50, 50, 50, 0), (-10, 11), (-10, 1), 0.1, 0.15),
            13 => set(&mut d, 8, col(50, 50, 50, 0), (-10, 11), (-10, 11), 0.1, 0.1),
            12 => set(&mut d, 1, col(100, 100, 100, 0), (-10, 11), (-10, 11), 0.01, 0.01),
            28 | 29 => steady(&mut d, col(75, 75, 75, 0), 11),
            44 => set(&mut d, 1, fallen_star_furniture_flame_color(global_time), (0, 0), (0, 0), 0.0, 0.0),
            45 => set(&mut d, 3, col(200, 200, 200, 150), (-10, 11), (-10, 1), 0.15, 0.35),
            46 => set(&mut d, 1, hallowed_furniture_flame_color(global_time), (0, 0), (0, 0), 0.0, 0.0),
            57 => set(&mut d, 1, wrapped_white_furniture_flame_color(0, global_time), (0, 0), (0, 0), 0.0, 0.0),
            58 | 61 => zero_range(&mut d, col(200, 200, 200, 150)),
            59 => set(&mut d, 1, wrapped_white_furniture_flame_color(1, global_time), (0, 0), (0, 0), 0.0, 0.0),
            62 => set(&mut d, 1, wrapped_white_furniture_flame_color(1, global_time), (0, 0), (0, 0), 0.0, 0.0),
            64 => set(&mut d, 1, wrapped_white_furniture_flame_color(1, global_time), (0, 0), (0, 0), 0.0, 0.0),
            48 | 49 | 50 | 52 | 53 | 55 => d.count = 0,
            _ => set(&mut d, 7, col(100, 100, 100, 0), (-10, 11), (-10, 1), 0.15, 0.35),
        },
        // `TileDrawing.cs:2525-2663` — sheet 13 (candles, type 42), styled by the
        // `tileFrameY / 36` parameter.
        13 => match frame_y as i32 / 36 {
            1 | 3 | 6 | 8 | 19 | 27 | 29 | 30 | 31 | 32 | 36 | 39 | 53 | 57 | 60 | 62 | 66 | 69 => {
                set(&mut d, 7, col(100, 100, 100, 0), (-10, 11), (-10, 1), 0.15, 0.35)
            }
            2 | 16 | 25 => set(&mut d, 7, col(50, 50, 50, 0), (-10, 11), (-10, 1), 0.15, 0.1),
            11 => set(&mut d, 7, col(50, 50, 50, 0), (-10, 11), (-10, 11), 0.075, 0.075),
            34 | 35 => steady(&mut d, col(75, 75, 75, 0), 1),
            44 => set(&mut d, 7, col(100, 100, 100, 0), (-10, 11), (-10, 1), 0.15, 0.35),
            50 => set(&mut d, 1, fallen_star_furniture_flame_color(global_time), (0, 0), (0, 0), 0.0, 0.0),
            51 => set(&mut d, 3, col(200, 200, 200, 150), (-10, 11), (-10, 1), 0.15, 0.35),
            52 => set(&mut d, 1, hallowed_furniture_flame_color(global_time), (0, 0), (0, 0), 0.0, 0.0),
            63 => set(&mut d, 1, wrapped_white_furniture_flame_color(0, global_time), (0, 0), (0, 0), 0.0, 0.0),
            64 | 67 => zero_range(&mut d, col(200, 200, 200, 150)),
            65 => set(&mut d, 1, wrapped_white_furniture_flame_color(1, global_time), (0, 0), (0, 0), 0.0, 0.0),
            68 => set(&mut d, 1, wrapped_white_furniture_flame_color(1, global_time), (0, 0), (0, 0), 0.0, 0.0),
            70 => set(&mut d, 1, wrapped_white_furniture_flame_color(1, global_time), (0, 0), (0, 0), 0.0, 0.0),
            54 | 55 | 56 | 58 | 59 | 61 => d.count = 0,
            _ => d.count = 0,
        },
        // `TileDrawing.cs:2664-2677` — every other sheet index (torches are
        // sheet 0), with the disco-torch colour override.
        _ => {
            let color = if frame_y as i32 / 22 == 14 {
                col(disco[0], disco[1], disco[2], 0)
            } else {
                col(100, 100, 100, 0)
            };
            set(&mut d, 7, color, (-10, 11), (-10, 1), 0.15, 0.35);
        }
    }
    d
}

/// One flame sprite of [`flame_quads`]: an offset from the tile's flame anchor
/// (`tileX * 16 - (tileWidth - 16) / 2`, `tileY * 16 + tileTop`), a tint and
/// the absolute source-rectangle top-left inside the `Images/Flame_N` sheet
/// (the tile's own frame coordinates, optionally shifted by `addFr`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FlameQuad {
    pub dx: f32,
    pub dy: f32,
    pub color: [f32; 4],
    pub src_x: i16,
    pub src_y: i16,
}

/// The inline flame branch of `DrawSingleTile_Flames`
/// (`TileDrawing.cs:2826-3462`). Callers draw every quad on
/// `TextureAssets.Flames[flame_sheet(kind)]` at the anchor plus `(dx, dy)`
/// with the tile's own frame rectangle shifted by `(src_x, src_y)`.
///
/// The C# branch hard-codes per-style loops that read the same tile frame but
/// disagree with `GetTileFlameData` in colour/offset for a few styles; this
/// port mirrors the drawing branch exactly (that is the one that renders).
#[allow(clippy::too_many_arguments)]
pub fn flame_quads(
    kind: u16,
    tile_x: i32,
    tile_y: i32,
    frame_x: i16,
    frame_y: i16,
    add_fr_x: i16,
    add_fr_y: i16,
    tile_frame_seed: u64,
    global_time: f32,
    disco: [u8; 3],
) -> Vec<FlameQuad> {
    let mut out = Vec::new();
    // `TileDrawing.cs:2828`.
    let mut seed = frame_seed_for(tile_frame_seed, tile_x, tile_y);
    let sheet = flame_sheet(kind);
    let fx = frame_x;
    let fy = frame_y;
    // Source rectangles: the plain tile frame and the `addFr`-shifted one.
    let plain = (fx, fy);
    let added = (fx + add_fr_x, fy + add_fr_y);

    // A hard-coded spread loop: `count` flames with per-axis jitter
    // `RandomInt(min, max) * mult`, drawn at the anchor plus the offset.
    fn spread(
        out: &mut Vec<FlameQuad>,
        seed: &mut u64,
        count: i32,
        rx: (i32, i32),
        mx: f32,
        ry: (i32, i32),
        my: f32,
        color: [f32; 4],
        src: (i16, i16),
    ) {
        for _ in 0..count {
            let dx = random_int(seed, rx.0, rx.1) as f32 * mx;
            let dy = random_int(seed, ry.0, ry.1) as f32 * my;
            out.push(FlameQuad {
                dx,
                dy,
                color,
                src_x: src.0,
                src_y: src.1,
            });
        }
    }

    // A single flame with no jitter (`TileDrawing.cs:2960-2961` and peers).
    fn single(out: &mut Vec<FlameQuad>, color: [f32; 4], src: (i16, i16)) {
        out.push(FlameQuad {
            dx: 0.0,
            dy: 0.0,
            color,
            src_x: src.0,
            src_y: src.1,
        });
    }

    // The `GetTileFlameData`-driven path (`TileDrawing.cs:2946-2959` and peers):
    // the seed quirk `if (seed2 == 0L) seed2 = tileFlameData.flameSeed;` is a
    // no-op by construction (both are `TileFrameSeed ^ position`), ported anyway.
    fn from_data(
        out: &mut Vec<FlameQuad>,
        seed_in: u64,
        tile_x: i32,
        tile_y: i32,
        kind: u16,
        frame_x: i16,
        frame_y: i16,
        tile_frame_seed: u64,
        global_time: f32,
        disco: [u8; 3],
        src: (i16, i16),
    ) {
        let mut data = tile_flame_data(
            tile_x,
            tile_y,
            kind,
            frame_x,
            frame_y,
            tile_frame_seed,
            global_time,
            disco,
        );
        let mut s = seed_in;
        if s == 0 {
            s = data.seed;
        }
        data.seed = s;
        for _ in 0..data.count {
            let dx = random_int(&mut data.seed, data.rx.0, data.rx.1) as f32 * data.mx;
            let dy = random_int(&mut data.seed, data.ry.0, data.ry.1) as f32 * data.my;
            out.push(FlameQuad {
                dx,
                dy,
                color: data.color,
                src_x: src.0,
                src_y: src.1,
            });
        }
    }

    // `TileDrawing.cs:2871-3462` (`switch (num9)`).
    match sheet {
        // `TileDrawing.cs:2871-2880` — sheet 7: four flames whose offsets are
        // drawn from the seed but drawn at `+ 0f` (C# computes `num93/num94`
        // and discards them; the RNG consumption is the only effect).
        7 => {
            for _ in 0..4 {
                let _dx = random_int(&mut seed, -10, 11) as f32 * 0.15;
                let _dy = random_int(&mut seed, -10, 10) as f32 * 0.15;
                single(&mut out, col(50, 50, 50, 0), plain);
            }
        }
        // `TileDrawing.cs:2881-2988` — sheet 1 (types 33/174).
        1 => {
            let n = frame_y as i32 / 22;
            let flag = n >= 44;
            match n {
                5 | 6 | 7 | 10 => spread(&mut out, &mut seed, 7, (-10, 11), 0.075, (-10, 11), 0.075, col(50, 50, 50, 0), plain),
                8 => spread(&mut out, &mut seed, 7, (-10, 11), 0.3, (-10, 11), 0.3, col(50, 50, 50, 0), plain),
                12 => spread(&mut out, &mut seed, 7, (-10, 11), 0.1, (-10, 1), 0.15, col(50, 50, 50, 0), plain),
                14 => spread(&mut out, &mut seed, 8, (-10, 11), 0.1, (-10, 11), 0.1, col(75, 75, 75, 0), plain),
                16 => spread(&mut out, &mut seed, 4, (-10, 11), 0.15, (-10, 11), 0.15, col(75, 75, 75, 0), plain),
                27 | 28 => single(&mut out, col(75, 75, 75, 0), plain),
                43 => from_data(&mut out, seed, tile_x, tile_y, kind, frame_x, frame_y, tile_frame_seed, global_time, disco, plain),
                _ => {
                    if flag {
                        from_data(&mut out, seed, tile_x, tile_y, kind, frame_x, frame_y, tile_frame_seed, global_time, disco, plain);
                    } else {
                        spread(&mut out, &mut seed, 7, (-10, 11), 0.15, (-10, 1), 0.35, col(100, 100, 100, 0), plain);
                    }
                }
            }
        }
        // `TileDrawing.cs:2989-3093` — sheet 2 (types 100/173).
        2 => {
            let n = frame_y as i32 / 36;
            let flag = n >= 45;
            match n {
                3 => spread(&mut out, &mut seed, 3, (-10, 11), 0.05, (-10, 11), 0.15, col(50, 50, 50, 0), plain),
                6 => spread(&mut out, &mut seed, 5, (-10, 11), 0.15, (-10, 11), 0.15, col(75, 75, 75, 0), plain),
                9 => spread(&mut out, &mut seed, 7, (-10, 11), 0.3, (-10, 11), 0.3, col(100, 100, 100, 0), plain),
                11 => spread(&mut out, &mut seed, 7, (-10, 11), 0.1, (-10, 1), 0.15, col(50, 50, 50, 0), plain),
                13 => spread(&mut out, &mut seed, 8, (-10, 11), 0.1, (-10, 11), 0.1, col(75, 75, 75, 0), plain),
                28 | 29 => single(&mut out, col(75, 75, 75, 0), plain),
                44 => from_data(&mut out, seed, tile_x, tile_y, kind, frame_x, frame_y, tile_frame_seed, global_time, disco, added),
                _ => {
                    if flag {
                        from_data(&mut out, seed, tile_x, tile_y, kind, frame_x, frame_y, tile_frame_seed, global_time, disco, added);
                    } else {
                        spread(&mut out, &mut seed, 7, (-10, 11), 0.15, (-10, 1), 0.35, col(100, 100, 100, 0), added);
                    }
                }
            }
        }
        // `TileDrawing.cs:3094-3213` — sheet 3 (type 34).
        3 => {
            let mut n = frame_y as i32 / 54;
            if frame_x >= 108 {
                n += 37 * (frame_x as i32 / 108);
            }
            let flag = n >= 51;
            match n {
                8 => spread(&mut out, &mut seed, 7, (-10, 11), 0.075, (-10, 11), 0.075, col(50, 50, 50, 0), plain),
                9 => spread(&mut out, &mut seed, 3, (-10, 11), 0.05, (-10, 11), 0.15, col(50, 50, 50, 0), plain),
                11 => spread(&mut out, &mut seed, 7, (-10, 11), 0.3, (-10, 11), 0.3, col(50, 50, 50, 0), plain),
                15 => spread(&mut out, &mut seed, 7, (-10, 11), 0.1, (-10, 1), 0.15, col(50, 50, 50, 0), plain),
                17 | 20 => spread(&mut out, &mut seed, 7, (-10, 11), 0.075, (-10, 11), 0.075, col(50, 50, 50, 0), plain),
                18 => spread(&mut out, &mut seed, 8, (-10, 11), 0.1, (-10, 11), 0.1, col(75, 75, 75, 0), plain),
                34 | 35 => single(&mut out, col(75, 75, 75, 0), plain),
                50 => from_data(&mut out, seed, tile_x, tile_y, kind, frame_x, frame_y, tile_frame_seed, global_time, disco, added),
                _ => {
                    if flag {
                        from_data(&mut out, seed, tile_x, tile_y, kind, frame_x, frame_y, tile_frame_seed, global_time, disco, plain);
                    } else {
                        spread(&mut out, &mut seed, 7, (-10, 11), 0.15, (-10, 1), 0.35, col(100, 100, 100, 0), added);
                    }
                }
            }
        }
        // `TileDrawing.cs:3214-3336` — sheet 4 (type 93). Note the drawing
        // branch styles case 13 with (75,75,75,0) while `GetTileFlameData`
        // uses (50,50,50,0); the C# tables genuinely disagree and both are
        // ported as written.
        4 => {
            let n = frame_y as i32 / 54;
            let flag = n >= 45;
            match n {
                1 => spread(&mut out, &mut seed, 3, (-10, 11), 0.15, (-10, 11), 0.15, col(50, 50, 50, 0), plain),
                2 | 4 => spread(&mut out, &mut seed, 7, (-10, 11), 0.075, (-10, 11), 0.075, col(50, 50, 50, 0), plain),
                3 => spread(&mut out, &mut seed, 7, (-10, 11), 0.2, (-20, 1), 0.35, col(100, 100, 100, 0), plain),
                5 => spread(&mut out, &mut seed, 7, (-10, 11), 0.3, (-10, 11), 0.3, col(50, 50, 50, 0), plain),
                9 => spread(&mut out, &mut seed, 7, (-10, 11), 0.1, (-10, 1), 0.15, col(50, 50, 50, 0), plain),
                13 => spread(&mut out, &mut seed, 8, (-10, 11), 0.1, (-10, 11), 0.1, col(75, 75, 75, 0), plain),
                // `TileDrawing.cs:3280-3285`: two jitter axes at 0.01 and the
                // colour drawn from three more `RandomInt(90, 111)` calls.
                12 => {
                    let dx = random_int(&mut seed, -10, 11) as f32 * 0.01;
                    let dy = random_int(&mut seed, -10, 11) as f32 * 0.01;
                    let r = random_int(&mut seed, 90, 111) as u8;
                    let g = random_int(&mut seed, 90, 111) as u8;
                    let b = random_int(&mut seed, 90, 111) as u8;
                    out.push(FlameQuad {
                        dx,
                        dy,
                        color: col(r, g, b, 0),
                        src_x: plain.0,
                        src_y: plain.1,
                    });
                }
                28 | 29 => single(&mut out, col(75, 75, 75, 0), plain),
                44 => from_data(&mut out, seed, tile_x, tile_y, kind, frame_x, frame_y, tile_frame_seed, global_time, disco, added),
                _ => {
                    if flag {
                        from_data(&mut out, seed, tile_x, tile_y, kind, frame_x, frame_y, tile_frame_seed, global_time, disco, added);
                    } else {
                        spread(&mut out, &mut seed, 7, (-10, 11), 0.15, (-10, 1), 0.35, col(100, 100, 100, 0), added);
                    }
                }
            }
        }
        // `TileDrawing.cs:3337-3430` — sheet 13 (type 42, candles). The nested
        // `default` re-lists `case 29` (unreachable — 29 is in the first
        // group); the port keeps only the reachable arms.
        13 => {
            let n = frame_y as i32 / 36;
            let flag = n >= 51;
            match n {
                1 | 3 | 6 | 8 | 19 | 27 | 29 | 30 | 31 | 32 | 36 | 39 => {
                    spread(&mut out, &mut seed, 7, (-10, 11), 0.15, (-10, 1), 0.35, col(100, 100, 100, 0), plain)
                }
                2 | 16 | 25 => spread(&mut out, &mut seed, 7, (-10, 11), 0.15, (-10, 1), 0.1, col(50, 50, 50, 0), plain),
                34 | 35 => single(&mut out, col(75, 75, 75, 0), plain),
                50 => from_data(&mut out, seed, tile_x, tile_y, kind, frame_x, frame_y, tile_frame_seed, global_time, disco, added),
                _ => {
                    if flag {
                        from_data(&mut out, seed, tile_x, tile_y, kind, frame_x, frame_y, tile_frame_seed, global_time, disco, plain);
                    }
                }
            }
        }
        // `TileDrawing.cs:3431-3462` — torches (sheet 0) and every other sheet:
        // the disco/ultrabright overrides keyed on the torch style.
        _ => {
            let mut color = col(100, 100, 100, 0);
            if kind == 4 {
                match frame_y as i32 / 22 {
                    14 => color = col(disco[0], disco[1], disco[2], 0),
                    22 => color = col(50, 50, 100, 20),
                    23 => color = col(255, 255, 255, 200),
                    _ => {}
                }
            }
            if kind == 646 {
                color = col(100, 100, 100, 150);
            }
            spread(&mut out, &mut seed, 7, (-10, 11), 0.15, (-10, 1), 0.35, color, plain);
        }
    }
    out
}

// ---------------------------------------------------------------------------
// `DrawSingleTile_Flames`' glow branches (`TileDrawing.cs:2684-2825`,
// `:3463-3491`). Every branch draws on the shared anchor
// `(tileX * 16 - (tileWidth - 16) / 2, tileY * 16 + tileTop)` with a source
// rectangle built from the tile's own frame, except the teleporter pulse
// (`:3471`), which centres on `tileX * 16 - tileWidth / 2`, rises 10 px and
// samples the whole sheet. Type 85 (`:2724-2750`, the graveyard tombstone
// flames) is not ported: it needs `Main.GraveyardVisualIntensity`, which the
// port does not model (`ZoneGraveyard` is not simulated).

/// Which sheet a [`GlowQuad`] samples.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GlowSheet {
    /// `TextureAssets.GlowMask[id]` = `Images/Glow_{id}`
    /// (`AssetInitializer.cs:352-354`).
    GlowMask(u16),
    /// `TextureAssets.Flames[i]` = `Images/Flame_{i}`.
    Flames(usize),
    /// A named single sheet (`TextureAssets.FireflyJar`, `Timer`, ...).
    Named(&'static str),
}

/// One glow overlay quad of `DrawSingleTile_Flames`. `dx`/`dy` are relative to
/// the tile cell origin (`tileX * 16`, `tileY * 16`); the draw size is the
/// source rectangle's (w, h) — except `src.w == 0`, which means the whole
/// sheet at its own pixel size (the `value.Frame()` draws).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GlowQuad {
    pub sheet: GlowSheet,
    /// The source rectangle inside the sheet (x, y, w, h); w == 0 selects the
    /// whole sheet.
    pub src: [i32; 4],
    pub dx: f32,
    pub dy: f32,
    /// The normalized `Color(r, g, b, a)` tint.
    pub color: [f32; 4],
}

/// The glow overlays of one flame-capable tile (`TileDrawing.cs:2684-2825`,
/// `:3463-3491`). `dd` carries the `GetTileDrawData` metrics (anchor offset,
/// `addFrX`/`addFrY`); `env` the `Main` state the branches read.
#[allow(clippy::too_many_arguments)]
pub fn glow_quads(
    kind: u16,
    tile_x: i32,
    tile_y: i32,
    frame_x: i16,
    frame_y: i16,
    dd: &TileDrawData,
    env: &FlameEnv,
) -> Vec<GlowQuad> {
    let mut out = Vec::new();
    // The standard anchor terms (`TileDrawing.cs:2687` and peers), baked into
    // every quad so the renderer only adds the cell origin.
    let ax = -((dd.width as f32 - 16.0) * 0.5);
    let ay = dd.top as f32;
    let (fx, fy) = (frame_x as i32, frame_y as i32);
    let (w, h) = (dd.width as i32, dd.height as i32);
    // The source-rectangle shapes the branches build: the frame shifted by
    // both `addFr`s, by `addFrX` only (`:2790-2800` — the jars and 581 drop
    // `addFrY`), and the plain / `addFrY`-only frames.
    let full = [fx + dd.add_fr_x as i32, fy + dd.add_fr_y as i32, w, h];
    let no_add_y = [fx + dd.add_fr_x as i32, fy, w, h];
    let plain = [fx, fy, w, h];
    let add_fr_y = [fx, fy + dd.add_fr_y as i32, w, h];
    let opaque = [1.0, 1.0, 1.0, 1.0];

    // `TileDrawing.cs:2686-2710` — the five per-type glow-mask sheets.
    if kind == 548 && fx / 54 > 6 {
        out.push(GlowQuad { sheet: GlowSheet::GlowMask(297), src: full, dx: ax, dy: ay, color: opaque });
    }
    if kind == 613 {
        out.push(GlowQuad { sheet: GlowSheet::GlowMask(298), src: full, dx: ax, dy: ay, color: opaque });
    }
    if kind == 614 {
        out.push(GlowQuad { sheet: GlowSheet::GlowMask(299), src: full, dx: ax, dy: ay, color: opaque });
    }
    if kind == 593 {
        out.push(GlowQuad { sheet: GlowSheet::GlowMask(295), src: full, dx: ax, dy: ay, color: opaque });
    }
    if kind == 594 {
        out.push(GlowQuad { sheet: GlowSheet::GlowMask(296), src: full, dx: ax, dy: ay, color: opaque });
    }
    // `TileDrawing.cs:2706-2723` — the trophy pedestal's `Flames[15]`
    // overlay, colour-switched on `tileFrameX / 54`.
    if kind == 215 && frame_y < 36 {
        let color = match fx / 54 {
            5 => [
                env.disco[0] as f32 / 255.0,
                env.disco[1] as f32 / 255.0,
                env.disco[2] as f32 / 255.0,
                0.0,
            ],
            14 => col(50, 50, 100, 20),
            15 => col(255, 255, 255, 200),
            _ => col(255, 255, 255, 0),
        };
        out.push(GlowQuad { sheet: GlowSheet::Flames(15), src: add_fr_y, dx: ax, dy: ay, color });
    }
    // `TileDrawing.cs:2751-2765` — the sundial's seven-jitter glow. The seed
    // mixes the tile's own cell minus the frame offset
    // (`tileX - tileFrameX / 18`), so the whole multi-cell sundial jitters as
    // one flame.
    if kind == 356 && env.sundial_cooldown == 0 {
        let mut seed = frame_seed_for(
            env.tile_frame_seed,
            tile_x - fx / 18,
            tile_y - fy / 18,
        );
        for _ in 0..7 {
            let dx = random_int(&mut seed, -10, 11) as f32 * 0.15;
            let dy = random_int(&mut seed, -10, 1) as f32 * 0.35;
            out.push(GlowQuad {
                sheet: GlowSheet::GlowMask(325),
                src: add_fr_y,
                dx: ax + dx,
                dy: ay + dy,
                color: col(100, 100, 100, 0),
            });
        }
    }
    // `TileDrawing.cs:2766-2772` — the moondial glow, one source row per moon
    // phase, tinted `Color.White * mouseTextColor / 255` (all four channels —
    // the alpha rides along, unlike the other glow tints).
    if kind == 663 && env.moondial_cooldown == 0 {
        let m = env.mouse_text_color as f32 / 255.0;
        out.push(GlowQuad {
            sheet: GlowSheet::GlowMask(335),
            src: [fx, fy + dd.add_fr_y as i32 + 54 * env.moon_phase as i32, w, h],
            dx: ax,
            dy: ay,
            color: [m, m, m, m],
        });
    }
    // `TileDrawing.cs:2773-2804` — the static per-type overlays.
    if kind == 286 {
        out.push(GlowQuad { sheet: GlowSheet::Named("GlowSnail"), src: full, dx: ax, dy: ay, color: col(75, 100, 255, 0) });
    }
    if kind == 582 {
        out.push(GlowQuad { sheet: GlowSheet::GlowMask(293), src: full, dx: ax, dy: ay, color: col(200, 100, 100, 0) });
    }
    if kind == 391 {
        out.push(GlowQuad { sheet: GlowSheet::GlowMask(131), src: full, dx: ax, dy: ay, color: col(250, 250, 250, 200) });
    }
    if kind == 619 {
        out.push(GlowQuad { sheet: GlowSheet::GlowMask(300), src: full, dx: ax, dy: ay, color: col(75, 100, 255, 0) });
    }
    if kind == 270 {
        out.push(GlowQuad { sheet: GlowSheet::Named("FireflyJar"), src: no_add_y, dx: ax, dy: ay, color: col(200, 200, 200, 0) });
    }
    if kind == 271 {
        out.push(GlowQuad { sheet: GlowSheet::Named("LightningBugJar"), src: no_add_y, dx: ax, dy: ay, color: col(200, 200, 200, 0) });
    }
    if kind == 581 {
        out.push(GlowQuad { sheet: GlowSheet::GlowMask(291), src: no_add_y, dx: ax, dy: ay, color: col(200, 200, 200, 0) });
    }
    // `TextureAssets.JellyfishBowl[type - 316]` = `Images/jellyfishBowl{1..3}`
    // (`AssetInitializer.cs:329-330`).
    if (316..=318).contains(&kind) {
        let name = match kind {
            316 => "jellyfishBowl1",
            317 => "jellyfishBowl2",
            _ => "jellyfishBowl3",
        };
        out.push(GlowQuad { sheet: GlowSheet::Named(name), src: full, dx: ax, dy: ay, color: col(200, 200, 200, 0) });
    }
    // `TileDrawing.cs:2805-2808` — the Christmas-light strand's lit bulbs.
    if kind == 149 && frame_x < 54 {
        out.push(GlowQuad { sheet: GlowSheet::Named("XmasLight"), src: plain, dx: ax, dy: ay, color: col(200, 200, 200, 0) });
    }
    // `TileDrawing.cs:2809-2825` — the campfire variants on `Flames[9..12]`.
    if matches!(kind, 300 | 302 | 303 | 306) {
        let sheet = match kind {
            302 => 10,
            303 => 11,
            306 => 12,
            _ => 9,
        };
        out.push(GlowQuad { sheet: GlowSheet::Flames(sheet), src: add_fr_y, dx: ax, dy: ay, color: col(200, 200, 200, 0) });
    }
    // `TileDrawing.cs:3463-3470` — the timer face and the sun altar.
    if kind == 144 {
        out.push(GlowQuad { sheet: GlowSheet::Named("Timer"), src: plain, dx: ax, dy: ay, color: col(200, 200, 200, 0) });
    }
    if kind == 237 {
        // `new Color(Main.mouseTextColor / 2, ..., 0)` — integer division.
        let m = (env.mouse_text_color / 2) as f32 / 255.0;
        out.push(GlowQuad { sheet: GlowSheet::Named("SunAltar"), src: plain, dx: ax, dy: ay, color: [m, m, m, 0.0] });
    }
    // `TileDrawing.cs:3471-3491` — the teleporter pulse: only the top-left
    // cell of the 3x3 frame draws, on the whole `Glow_334` sheet, 10 px up
    // and centred on `tileX * 16 - tileWidth / 2`.
    if kind == 658 && fx % 36 == 0 && fy % 54 == 0 {
        let n = fy / 54;
        if n != 2 {
            let m = env.mouse_text_color as f32 / 255.0;
            let mut color = [m, m, m, 0.0];
            if n == 0 {
                // `color5 *= 0.75f` (byte truncation; the port multiplies the
                // normalized floats — at most 1/255 per channel apart).
                for c in color.iter_mut() {
                    *c *= 0.75;
                }
            }
            out.push(GlowQuad {
                sheet: GlowSheet::GlowMask(334),
                // `src = [0, 0, 0, 0]`: the renderer resolves the whole-sheet
                // frame (`value3.Frame()`).
                src: [0, 0, 0, 0],
                dx: -(dd.width as f32) * 0.5,
                dy: ay - 10.0,
                color,
            });
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The shared seed for the oracles below: `0x5F3759DF` (the port's
    /// [`TILE_FRAME_SEED_INIT`]) at tile (10, 10), i.e.
    /// `TileFrameSeed ^ (10 << 32 | 10)`.
    fn seed0() -> u64 {
        frame_seed_for(TILE_FRAME_SEED_INIT, 10, 10)
    }

    #[test]
    fn random_int_matches_the_csharp_lcg() {
        // Values precomputed with the exact C# algorithm
        // (`Utils.RandomNextSeed`/`RandomNext`/`RandomInt`):
        // seed=0xa5f3759d5 → RandomInt(-10,11) = -9, then -1, -3, 10, 0, 5, 2.
        let mut s = seed0();
        let seq: Vec<i32> = (0..7).map(|_| random_int(&mut s, -10, 11)).collect();
        assert_eq!(seq, [-9, -1, -3, 10, 0, 5, 2]);
        // RandomInt(-10,1) draws: 0, -10, -1, -8, -8, -6, -7.
        let mut s = seed0();
        let seq: Vec<i32> = (0..7).map(|_| random_int(&mut s, -10, 1)).collect();
        assert_eq!(seq, [0, -10, -1, -8, -8, -6, -7]);
        // Power-of-two range consumes the generator through the shift branch:
        // RandomInt(0, 16) from a fresh seed 0x5F3759DF.
        let mut s = TILE_FRAME_SEED_INIT;
        let seq: Vec<i32> = (0..4).map(|_| random_int(&mut s, 0, 16)).collect();
        assert_eq!(seq, [13, 12, 15, 7]);
    }

    #[test]
    fn frame_seed_mixes_the_tile_position() {
        // `Main.TileFrameSeed ^ ((long)tileX << 32 | (uint)tileY)`.
        assert_eq!(frame_seed_for(0, 0, 0), 0);
        assert_eq!(frame_seed_for(0x1234, 10, 10), 0x1234 ^ 0xA_0000_000A);
        // Negative X sign-extends into the high word, like the C# `(long)x << 32`.
        assert_eq!(frame_seed_for(0, -1, 0), 0xFFFF_FFFF_0000_0000);
        assert_eq!(frame_seed_for(0, 1, 5), 1 << 32 | 5);
    }

    #[test]
    fn flame_sheet_matches_the_type_switch() {
        // `TileDrawing.cs:1880-1897`.
        let table = [
            (4u16, 0usize),
            (33, 1),
            (174, 1),
            (100, 2),
            (173, 2),
            (34, 3),
            (93, 4),
            (49, 5),
            (98, 6),
            (35, 7),
            (42, 13),
            (372, 16),
            (646, 17),
        ];
        for (kind, sheet) in table {
            assert_eq!(flame_sheet(kind), sheet, "type {kind}");
        }
        // Unknown types keep the C# `num = 0` initialiser.
        assert_eq!(flame_sheet(999), 0);
    }

    #[test]
    fn tile_flame_table_lists_the_main_types() {
        // `Main.cs:9300-9313`.
        for kind in [4, 33, 34, 35, 42, 49, 93, 98, 100, 173, 174, 372, 646] {
            assert!(tile_flame(kind), "type {kind}");
        }
        for kind in [0, 1, 2, 19, 270, 271, 581, 300] {
            assert!(!tile_flame(kind), "type {kind}");
        }
    }

    #[test]
    fn torch_flame_uses_the_default_spread() {
        // `TileDrawing.cs:2664-2677` (num 0 → default): 7 flames,
        // Color(100,100,100,0), x in [-10,11]*0.15, y in [-10,1]*0.35.
        let d = tile_flame_data(10, 10, 4, 0, 0, TILE_FRAME_SEED_INIT, 0.0, [255, 0, 0]);
        assert_eq!(d.sheet, 0);
        assert_eq!(d.count, 7);
        assert_eq!(d.color, col(100, 100, 100, 0));
        assert_eq!((d.rx, d.ry), ((-10, 11), (-10, 1)));
        assert_eq!((d.mx, d.my), (0.15, 0.35));
        assert_eq!(d.seed, seed0());
    }

    #[test]
    fn torch_disco_style_gets_the_disco_colour() {
        // `TileDrawing.cs:2666-2669`: `tileFrameY / 22 == 14` (disco torch).
        let d = tile_flame_data(10, 10, 4, 0, 14 * 22, TILE_FRAME_SEED_INIT, 0.0, [12, 34, 56]);
        assert_eq!(d.color, col(12, 34, 56, 0));
        // Style 22/23 override in the *drawing* branch only; `GetTileFlameData`
        // keeps the plain/Disco colour split.
        let d22 = tile_flame_data(10, 10, 4, 0, 22 * 22, TILE_FRAME_SEED_INIT, 0.0, [12, 34, 56]);
        assert_eq!(d22.color, col(100, 100, 100, 0));
    }

    #[test]
    fn candelabra_sheet_seven_is_four_steady_flames() {
        // `TileDrawing.cs:1925-1934`.
        let d = tile_flame_data(10, 10, 35, 0, 0, TILE_FRAME_SEED_INIT, 0.0, [0, 0, 0]);
        assert_eq!(d.sheet, 7);
        assert_eq!(d.count, 4);
        assert_eq!(d.color, col(50, 50, 50, 0));
        assert_eq!(d.ry, (-10, 10)); // note: XMax 11, YMax 10 in C#
        assert_eq!(d.rx, (-10, 11));
    }

    #[test]
    fn chandelier_styles_map_to_their_table_rows() {
        // `TileDrawing.cs:1935-2074`, sheet 1 keyed on `frameY / 22`.
        let mk = |fy: i16| tile_flame_data(10, 10, 33, 0, fy, TILE_FRAME_SEED_INIT, 0.0, [0, 0, 0]);
        let d = mk(5 * 22);
        assert_eq!((d.count, d.mx, d.my), (7, 0.075, 0.075));
        assert_eq!(d.color, col(50, 50, 50, 0));
        let d = mk(8 * 22);
        assert_eq!((d.count, d.mx, d.my), (7, 0.3, 0.3));
        let d = mk(12 * 22);
        assert_eq!((d.count, d.ry), (7, (-10, 1)));
        let d = mk(14 * 22);
        assert_eq!((d.count, d.color), (8, col(75, 75, 75, 0)));
        let d = mk(16 * 22);
        assert_eq!((d.count, d.mx), (4, 0.15));
        let d = mk(27 * 22);
        assert_eq!((d.count, d.mx), (1, 0.0));
        // 43 = fallen-star furniture colour; at time 0 the lerp is 0.5 → 75.
        let d = mk(43 * 22);
        assert_eq!(d.count, 1);
        assert_eq!(d.color, [150.0 / 255.0, 75.0 / 255.0, 75.0 / 255.0, 50.0 / 255.0]);
        // 44 = three (200,200,200,150) flames.
        let d = mk(44 * 22);
        assert_eq!((d.count, d.color), (3, col(200, 200, 200, 150)));
        // Off rows draw nothing.
        let d = mk(47 * 22);
        assert_eq!(d.count, 0);
    }

    #[test]
    fn chandelier_sheet_three_adds_the_frame_x_block() {
        // `TileDrawing.cs:2215-2219`: `num2 += 37 * (frameX / 108)` once
        // `frameX >= 108`.
        let d = tile_flame_data(10, 10, 34, 108, 8 * 54, TILE_FRAME_SEED_INIT, 0.0, [0, 0, 0]);
        // 8 + 37*1 = 45 → falls through to the default row.
        assert_eq!(d.count, 7);
        assert_eq!(d.color, col(100, 100, 100, 0));
        let d = tile_flame_data(10, 10, 34, 0, 8 * 54, TILE_FRAME_SEED_INIT, 0.0, [0, 0, 0]);
        assert_eq!((d.count, d.mx, d.my), (7, 0.075, 0.075));
        let d = tile_flame_data(10, 10, 34, 216, 9 * 54, TILE_FRAME_SEED_INIT, 0.0, [0, 0, 0]);
        // 9 + 74 = 83 → default row.
        assert_eq!(d.count, 7);
        let d = tile_flame_data(10, 10, 34, 0, 9 * 54, TILE_FRAME_SEED_INIT, 0.0, [0, 0, 0]);
        // Style 9: three wide flames, ±2 px jitter.
        assert_eq!((d.count, d.rx, d.mx), (3, (-1, 1), 2.0));
    }

    #[test]
    fn sheet_four_style_three_spreads_downwards() {
        // `TileDrawing.cs:2377-2384`: y in [-20, 1], mult 0.35.
        let d = tile_flame_data(10, 10, 93, 0, 3 * 54, TILE_FRAME_SEED_INIT, 0.0, [0, 0, 0]);
        assert_eq!(d.sheet, 4);
        assert_eq!(d.ry, (-20, 1));
        assert_eq!((d.mx, d.my), (0.2, 0.35));
        assert_eq!(d.color, col(100, 100, 100, 0));
    }

    #[test]
    fn candle_styles_key_on_frame_y_over_36() {
        // `TileDrawing.cs:2525-2663`, sheet 13.
        let mk = |fy: i16| tile_flame_data(10, 10, 42, 0, fy, TILE_FRAME_SEED_INIT, 0.0, [0, 0, 0]);
        let d = mk(1 * 36);
        assert_eq!((d.count, d.color, d.my), (7, col(100, 100, 100, 0), 0.35));
        let d = mk(2 * 36);
        assert_eq!((d.count, d.color, d.my), (7, col(50, 50, 50, 0), 0.1));
        let d = mk(11 * 36);
        assert_eq!((d.count, d.ry), (7, (-10, 11)));
        let d = mk(34 * 36);
        assert_eq!(d.count, 1);
        // Off styles and the unknown default both give count 0 here.
        assert_eq!(mk(54 * 36).count, 0);
        assert_eq!(mk(700 * 36).count, 0);
    }

    #[test]
    fn jar_types_use_their_own_table_rows() {
        // `TileDrawing.cs:1849-1874`.
        let d = tile_flame_data(10, 10, 270, 0, 0, TILE_FRAME_SEED_INIT, 0.0, [0, 0, 0]);
        assert_eq!((d.count, d.color), (1, col(200, 200, 200, 0)));
        let d = tile_flame_data(10, 10, 271, 0, 0, TILE_FRAME_SEED_INIT, 0.0, [0, 0, 0]);
        assert_eq!((d.count, d.color), (1, col(200, 200, 200, 0)));
        let d = tile_flame_data(10, 10, 581, 0, 0, TILE_FRAME_SEED_INIT, 0.0, [0, 0, 0]);
        assert_eq!((d.count, d.color), (1, col(200, 100, 100, 0)));
        // Non-flame tiles get the zeroed struct.
        assert_eq!(
            tile_flame_data(10, 10, 1, 0, 0, TILE_FRAME_SEED_INIT, 0.0, [0, 0, 0]),
            TileFlameData::default()
        );
    }

    #[test]
    fn furniture_colours_follow_the_time_triangle() {
        // `GetFallenStarFurnitureFlameColor`: `(int)(150 * WrappedLerp(0.5, 1, t))`.
        // t = 1.0 → triangle 1.0 → 150; t = 2.0 → triangle 0.5 → 75.
        let full = tile_flame_data(10, 10, 33, 0, 43 * 22, TILE_FRAME_SEED_INIT, 1.0, [0, 0, 0]);
        assert_eq!(full.color, [150.0 / 255.0, 150.0 / 255.0, 150.0 / 255.0, 50.0 / 255.0]);
        let half = tile_flame_data(10, 10, 33, 0, 43 * 22, TILE_FRAME_SEED_INIT, 2.0, [0, 0, 0]);
        assert_eq!(half.color, [150.0 / 255.0, 75.0 / 255.0, 75.0 / 255.0, 50.0 / 255.0]);
        // Hallowed: base (170,170,170,75); t = 1.0 → scale 1.0.
        let d = tile_flame_data(10, 10, 33, 0, 45 * 22, TILE_FRAME_SEED_INIT, 1.0, [0, 0, 0]);
        assert_eq!(d.color, [170.0 / 255.0, 170.0 / 255.0, 170.0 / 255.0, 75.0 / 255.0]);
    }

    #[test]
    fn torch_quads_follow_the_lcg_sequence() {
        // `TileDrawing.cs:3431-3462`: seven quads, colour (100,100,100,0),
        // offsets (RandomInt(-10,11)*0.15, RandomInt(-10,1)*0.35). The oracle
        // sequence was precomputed with the exact C# algorithm:
        // (-9,-10) (-3,-8) (0,-6) (2,-2) (-3,-4) (1,-9) (-8,-3).
        let q = flame_quads(4, 10, 10, 0, 0, 0, 0, TILE_FRAME_SEED_INIT, 0.0, [255, 0, 0]);
        assert_eq!(q.len(), 7);
        let expect = [
            (-9.0, -10.0),
            (-3.0, -8.0),
            (0.0, -6.0),
            (2.0, -2.0),
            (-3.0, -4.0),
            (1.0, -9.0),
            (-8.0, -3.0),
        ];
        for (quad, (x, y)) in q.iter().zip(expect) {
            assert_eq!((quad.dx, quad.dy), (x * 0.15, y * 0.35));
            assert_eq!(quad.color, col(100, 100, 100, 0));
            assert_eq!((quad.src_x, quad.src_y), (0, 0));
        }
        // All quads sample the sheet-0 texture at the tile's 20x20 frame.
        assert_eq!(flame_sheet(4), 0);
    }

    #[test]
    fn candelabra_quads_discard_their_offsets() {
        // `TileDrawing.cs:2871-2880`: four flames at (0,0); the two
        // RandomInt calls per flame only advance the seed. Oracle:
        // (-9,-8) (-3,-9) (0,0) (2,-2).
        let q = flame_quads(35, 10, 10, 44, 0, 0, 0, TILE_FRAME_SEED_INIT, 0.0, [0, 0, 0]);
        assert_eq!(q.len(), 4);
        for quad in &q {
            assert_eq!((quad.dx, quad.dy), (0.0, 0.0));
            assert_eq!(quad.color, col(50, 50, 50, 0));
            assert_eq!((quad.src_x, quad.src_y), (44, 0));
        }
        // The seed advanced by 8 RandomInt calls: the next draw with the same
        // tile is NOT a repeat of the oracle sequence (unlike a stateless call).
        let q2 = flame_quads(35, 10, 10, 44, 0, 0, 0, TILE_FRAME_SEED_INIT, 0.0, [0, 0, 0]);
        assert_eq!(q, q2, "flame quads are a pure function of the seed");
    }

    #[test]
    fn candle_quads_use_the_style_multipliers() {
        // Style 2 (`frameY / 36 == 2`): 7 flames, x*0.15, y*0.1, colour (50,50,50,0).
        let q = flame_quads(42, 10, 10, 0, 2 * 36, 0, 0, TILE_FRAME_SEED_INIT, 0.0, [0, 0, 0]);
        assert_eq!(q.len(), 7);
        // The pair oracle for the x/y draw order: (-9,-10) (-3,-8) (0,-6) ...
        assert_eq!((q[0].dx, q[0].dy), (-9.0 * 0.15, -10.0 * 0.1));
        assert_eq!((q[1].dx, q[1].dy), (-3.0 * 0.15, -8.0 * 0.1));
        assert_eq!(q[0].color, col(50, 50, 50, 0));
        // Style 34/35: a single steady flame.
        let q = flame_quads(42, 10, 10, 0, 34 * 36, 0, 0, TILE_FRAME_SEED_INIT, 0.0, [0, 0, 0]);
        assert_eq!(q.len(), 1);
        assert_eq!((q[0].dx, q[0].dy), (0.0, 0.0));
    }

    #[test]
    fn sheet_four_style_twelve_draws_the_random_colour() {
        // `TileDrawing.cs:3280-3285`: dx/dy at 0.01 then three RandomInt(90,111).
        // Oracle from seed 0xa5f3759d5: dx=-9, dy=-1, rgb=(97,110,100).
        let q = flame_quads(93, 10, 10, 0, 12 * 54, 0, 0, TILE_FRAME_SEED_INIT, 0.0, [0, 0, 0]);
        assert_eq!(q.len(), 1);
        assert_eq!((q[0].dx, q[0].dy), (-9.0 * 0.01, -1.0 * 0.01));
        assert_eq!(q[0].color, col(97, 110, 100, 0));
    }

    #[test]
    fn data_driven_paths_forward_to_get_tile_flame_data() {
        // Sheet 2 style 44 (`TileDrawing.cs:3040-3053`): one fallen-star flame
        // on the `addFr`-shifted source rect; at time 1.0 the colour is
        // (150,150,150,50). The source rect is absolute: (frameX + addFrX,
        // frameY + addFrY) = (22, 44*36 + 6).
        let q = flame_quads(100, 10, 10, 18, 44 * 36, 4, 6, TILE_FRAME_SEED_INIT, 1.0, [0, 0, 0]);
        assert_eq!(q.len(), 1);
        assert_eq!((q[0].dx, q[0].dy), (0.0, 0.0));
        assert_eq!(q[0].color, [150.0 / 255.0, 150.0 / 255.0, 150.0 / 255.0, 50.0 / 255.0]);
        assert_eq!((q[0].src_x, q[0].src_y), (22, 44 * 36 + 6));
        // Sheet 13 style 51+ uses the flag path (`TileDrawing.cs:3416-3429`).
        let q = flame_quads(42, 10, 10, 0, 51 * 36, 0, 0, TILE_FRAME_SEED_INIT, 0.0, [0, 0, 0]);
        // GetTileFlameData sheet 13, style 51 → 3 (200,200,200,150) flames.
        assert_eq!(q.len(), 3);
        assert_eq!(q[0].color, col(200, 200, 200, 150));
    }

    #[test]
    fn shadow_candle_overrides_the_default_colour() {
        // `TileDrawing.cs:3451-3453`: type 646 → Color(100,100,100,150).
        let q = flame_quads(646, 10, 10, 0, 0, 0, 0, TILE_FRAME_SEED_INIT, 0.0, [0, 0, 0]);
        assert_eq!(q.len(), 7);
        assert_eq!(q[0].color, col(100, 100, 100, 150));
    }

    #[test]
    fn tile_frame_seed_advances_on_the_vanilla_cadence() {
        // `Main.cs:61716-61721`: one `RandomNextSeed` step every fifth draw —
        // 12 per second at 60 fps. The port folds the same steps out of the
        // animation clock, so equal times give equal seeds and the seed always
        // moves forward with time.
        assert_eq!(tile_frame_seed_at(0.0), TILE_FRAME_SEED_INIT);
        assert_eq!(
            tile_frame_seed_at(1.0 / 12.0),
            random_next_seed(TILE_FRAME_SEED_INIT)
        );
        let mut expected = TILE_FRAME_SEED_INIT;
        for _ in 0..24 {
            expected = random_next_seed(expected);
        }
        assert_eq!(tile_frame_seed_at(2.0), expected);
        // Sub-steps truncate: below the first 12 Hz boundary the seed stays
        // at the initial value.
        assert_eq!(tile_frame_seed_at(0.08), TILE_FRAME_SEED_INIT);
        // Negative clocks clamp to the initial seed.
        assert_eq!(tile_frame_seed_at(-5.0), TILE_FRAME_SEED_INIT);
    }

    fn env() -> FlameEnv {
        FlameEnv {
            tile_frame_seed: TILE_FRAME_SEED_INIT,
            global_time: 0.0,
            disco: [255, 0, 0],
            mouse_text_color: 225,
            moon_phase: 3,
            sundial_cooldown: 0,
            moondial_cooldown: 0,
        }
    }

    #[test]
    fn glow_quads_match_the_branch_table() {
        let dd = TileDrawData::default();
        let e = env();
        // `TileDrawing.cs:2690-2692`: type 613 draws GlowMask[298] on the
        // `addFr`-shifted tile frame, white, on the standard anchor.
        let q = glow_quads(613, 10, 10, 36, 54, &dd, &e);
        assert_eq!(q.len(), 1);
        assert_eq!(q[0].sheet, GlowSheet::GlowMask(298));
        assert_eq!(q[0].src, [36, 54, 16, 16]);
        assert_eq!((q[0].dx, q[0].dy), (0.0, 0.0));
        assert_eq!(q[0].color, [1.0; 4]);
        // `:2805-2808`: Xmas lights only glow on the first strand cell.
        assert!(glow_quads(149, 10, 10, 54, 0, &dd, &e).is_empty());
        let q = glow_quads(149, 10, 10, 0, 0, &dd, &e);
        assert_eq!(q[0].sheet, GlowSheet::Named("XmasLight"));
        assert_eq!(q[0].color, col(200, 200, 200, 0));
        // `:2809-2825`: the campfire picks Flames[9..12] per variant.
        for (kind, sheet) in [(300u16, 9usize), (302, 10), (303, 11), (306, 12)] {
            let q = glow_quads(kind, 10, 10, 0, 0, &dd, &e);
            assert_eq!(q[0].sheet, GlowSheet::Flames(sheet), "kind {kind}");
            assert_eq!(q[0].color, col(200, 200, 200, 0));
        }
        // `:3463-3470`: the sun altar tints with `mouseTextColor / 2`.
        let q = glow_quads(237, 10, 10, 0, 0, &dd, &e);
        assert_eq!(q[0].color, col(112, 112, 112, 0));
        // Non-glow tiles stay empty.
        assert!(glow_quads(4, 10, 10, 0, 0, &dd, &e).is_empty());
        assert!(glow_quads(1, 10, 10, 0, 0, &dd, &e).is_empty());
    }

    #[test]
    fn dial_glows_follow_their_cooldown_and_phase() {
        let dd = TileDrawData::default();
        // `TileDrawing.cs:2751-2765`: the sundial draws seven jittered quads on
        // `GlowMask[325]`; the seed mixes `tile - frame / 18`.
        let q = glow_quads(356, 10, 10, 36, 54, &dd, &env());
        assert_eq!(q.len(), 7);
        let mut seed = frame_seed_for(TILE_FRAME_SEED_INIT, 10 - 36 / 18, 10 - 54 / 18);
        for quad in &q {
            assert_eq!(
                (quad.dx, quad.dy),
                (
                    random_int(&mut seed, -10, 11) as f32 * 0.15,
                    random_int(&mut seed, -10, 1) as f32 * 0.35
                )
            );
            assert_eq!(quad.color, col(100, 100, 100, 0));
        }
        // A non-zero cooldown silences it.
        let mut e = env();
        e.sundial_cooldown = 1;
        assert!(glow_quads(356, 10, 10, 36, 54, &dd, &e).is_empty());
        // `:2766-2772`: the moondial rides the moon phase's 54 px row and
        // `Color.White * mouseTextColor / 255` (alpha included).
        let q = glow_quads(663, 10, 10, 0, 0, &dd, &env());
        assert_eq!(q.len(), 1);
        assert_eq!(q[0].src, [0, 54 * 3, 16, 16]);
        assert_eq!(q[0].color, [225.0 / 255.0; 4]);
        let mut e = env();
        e.moondial_cooldown = 1;
        assert!(glow_quads(663, 10, 10, 0, 0, &dd, &e).is_empty());
    }

    #[test]
    fn teleporter_pulse_draws_one_cell() {
        // `TileDrawing.cs:3471-3491`: the pulse draws for the frame cells with
        // `frameX % 36 == 0` (the left/right columns of the 3x3 — 0 and 36 of
        // {0,18,36}) and `frameY % 54 == 0` (the top row), on the whole
        // `Glow_334` sheet (src w == 0), 10 px up and centred on
        // `tileX * 16 - tileWidth / 2`.
        let dd = TileDrawData {
            width: 48,
            height: 48,
            ..TileDrawData::default()
        };
        let e = env();
        assert!(glow_quads(658, 10, 10, 18, 0, &dd, &e).is_empty(), "middle column");
        assert!(glow_quads(658, 10, 10, 0, 18, &dd, &e).is_empty(), "middle row");
        // Top-left cell → row `num136 == 0` → the tint scales by 0.75
        // (rgb only — the alpha started at 0).
        let m = 225.0f32 / 255.0 * 0.75;
        let q = glow_quads(658, 10, 10, 0, 0, &dd, &e);
        assert_eq!(q.len(), 1);
        assert_eq!(q[0].sheet, GlowSheet::GlowMask(334));
        assert_eq!(q[0].src, [0, 0, 0, 0]);
        assert_eq!((q[0].dx, q[0].dy), (-24.0, -10.0));
        assert_eq!(q[0].color, [m, m, m, 0.0]);
        // Top-right cell is row 0 too and draws the same pulse.
        let q = glow_quads(658, 10, 10, 36, 0, &dd, &e);
        assert_eq!(q.len(), 1);
        assert_eq!(q[0].color, [m, m, m, 0.0]);
        // Non-zero rows (`num136 == 2`) draw nothing.
        assert!(glow_quads(658, 10, 10, 0, 108, &dd, &e).is_empty());
    }
}
