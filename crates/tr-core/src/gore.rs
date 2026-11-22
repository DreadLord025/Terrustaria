//! The gore layer: the `Gore` record and the 600-slot pool of `Main.gore`,
//! with the verbatim `NewGore` spawn family (`decompiled/Terraria/Gore.cs`).
//!
//! Structure and spawn are this step (S1-E-4 part 1); the sticky/timeLeft/fade
//! `Update` walk, the draw pass, the `GoreID.Sets` per-type tables and the
//! 1 643 `Gore.NewGore*` drop sites are the following steps.
//!
//! The C# keeps the pool as the static `Main.gore[601]` array
//! (`Main.cs:66990`); the port carries it on [`GorePool`], which the App owns
//! like [`crate::particles::DustPool`]. Slot 600 is the sentinel `NewGore`
//! returns when a spawn is refused (the scan bound, `Gore.cs:1308-1320`) — it
//! is never scanned.
//!
//! The pool draws from the **caller-passed** [`UnifiedRandom`] — the same seam
//! as `particles.rs` (S1-E-1): the C#'s global `Main.rand` inside `NewGore`
//! (`Gore.cs:1300`, `:1327-1328`, `:1336-1338`, `:1382`, `:1393-1427`) runs
//! inside the caller's `Main.SwapRandom("UpdateGores")` stage
//! (`Main.cs:17749-17809`), and the port's clients hand the same
//! `RngKey::UpdateGores` stream in (`tr-app/src/app/rng.rs`, S1-A-2).
//!
//! **S1-A-10 RNG-order contract for `new_gore`:** in source order —
//! `Next(10, 31)` (the velocity.Y kick, `Gore.cs:1327`), `Next(-20, 21)`
//! (the velocity.X kick, `:1328`), and only when the ChildSafety gate is
//! armed *and* the type is dangerous: `Next(11, 14)` (the safe re-type,
//! `:1336`) then `NextFloat()` (the re-scaled size, `:1338`). Then the
//! per-type arms in order: 1225's `Next(6)` (`:1382`, unconditional),
//! SpecialAI-3's four draws (`:1393-1401`), SpecialAI-1's two
//! (`:1405`), the LOADED-GATED 411-430 arm's four (+1 when `goreTime==0`,
//! `:1411-1419`), 907-909's two (`:1426-1427`), the LOADED-GATED
//! SpecialAI-2 arm's one (`:1439`). W4-g68 wiring-2 opened the two
//! loaded-texture gates: an id in [`GorePool::set_texture_sizes`]' map
//! rolls the arm's extra randoms AND shifts the spawn position (the
//! A/B witness pins both branches); an absent id keeps the unloaded
//! behaviour — no draws, no shift.
//!
//! Documented drops (each at its site below): the `Main.rand == null` re-seed
//! (`Gore.cs:1300-1303`; streams are caller-owned and never null), the
//! `Main.netMode == 2` server guard (`:1288-1291`; single-player port).
//! The texture-dependent arms are WIRED (W4-g68): the `Width`/`Height`
//! asset seam ([`GorePool::set_texture_sizes`]: an id in the map is the C#'s
//! `IsLoaded == true` branch, an absent id keeps the `1f` fallback) and the
//! spawn-time position-adjust arms (`:1384-1388`, `:1407-1420`,
//! `:1433-1440`) behind the same gate — the position terms read the RAW
//! asset size (`Utils.cs:1301-1316`), not the framed cell, with the C#'s
//! integer `Width()/2` division kept. The `GoreID.Sets`
//! tables (`GoreID.cs:20-23`) are ported in [`crate::gore_id`], so the arms
//! gated on them (`:1355-1360`, `:1390-1456`) run.

use crate::unified_random::UnifiedRandom;
use crate::world::World;
use glam::Vec2;

/// The draw seam the pool rolls on: the C# `Next` conversions
/// (`UnifiedRandom.cs:120-141`) over the **caller's** stream. One draw = one
/// stream step; no new streams or instances are created.
pub trait GoreRng {
    /// `Next(max)` — `[0, max)`.
    fn next_max(&mut self, max: i32) -> i32;
    /// `Next(min, max)` — `[min, max)`.
    fn next_range(&mut self, min: i32, max: i32) -> i32;
    /// `Utils.NextFloat` (`Utils.cs:1198-1201`).
    fn next_float(&mut self) -> f32;
    /// `Utils.NextFloatDirection` (`Utils.cs:1203-1206`) — `NextDouble() * 2 - 1`.
    fn next_float_direction(&mut self) -> f32;
}

impl GoreRng for UnifiedRandom {
    fn next_max(&mut self, max: i32) -> i32 {
        UnifiedRandom::next_max(self, max)
    }
    fn next_range(&mut self, min: i32, max: i32) -> i32 {
        UnifiedRandom::next_range(self, min, max)
    }
    fn next_float(&mut self) -> f32 {
        UnifiedRandom::next_float(self)
    }
    fn next_float_direction(&mut self) -> f32 {
        UnifiedRandom::next_float_direction(self)
    }
}

/// The dust-spawn adapter (W4-g68): the C#'s `Gore_UpdateLeaf` lava arm calls
/// `Dust.NewDust` on the SAME `Main.rand` the gore walk rolls (both sit inside
/// `Main.SwapRandom("UpdateGores")`, `Main.cs:17771-17774`), so the gore
/// stream feeds the dust pool directly — [`GoreRng`]'s three draw kinds are
/// exactly [`crate::particles::DustRng`]'s. One adapter borrow per spawn; no
/// new stream instance (the S1-E-1 seam rule).
struct DustFromGore<'a, R: GoreRng>(&'a mut R);

impl<R: GoreRng> crate::particles::DustRng for DustFromGore<'_, R> {
    fn next_max(&mut self, max: i32) -> i32 {
        self.0.next_max(max)
    }
    fn next_range(&mut self, min: i32, max: i32) -> i32 {
        self.0.next_range(min, max)
    }
    fn next_float(&mut self) -> f32 {
        self.0.next_float()
    }
}

/// The collision seam the update walk needs. The port's collision ports live
/// in `tr-app` (`collision.rs`, `player.rs::wet_collision`), which `tr-core`
/// cannot depend on — the App injects the probes once (the same seam the
/// dust pool uses for its wet probe, `particles.rs:197-202`).
pub trait GoreCollision {
    /// `Collision.TileCollision(Position, Velocity, Width, Height)`
    /// (`Collision.cs:2292`) with the C#'s defaults (`fallThrough = false`,
    /// `fall2 = false`, `gravDir = 1`, `ignoreDoors = false`, `hoik = true`).
    fn tile_collision(
        &self,
        world: &World,
        position: Vec2,
        velocity: Vec2,
        width: f32,
        height: f32,
    ) -> Vec2;

    /// `Collision.WetCollision(Position, Width, Height)` (`Collision.cs:1572`).
    fn wet_collision(&self, world: &World, position: Vec2, width: f32, height: f32) -> bool;

    /// `Collision.SlopeCollision(Position, Velocity, Width, Height, gravity,
    /// fall)` (`Collision.cs:1791`) — the C#'s `Vector4`: the new position
    /// and the new velocity.
    #[allow(clippy::too_many_arguments)]
    fn slope_collision(
        &self,
        world: &World,
        position: Vec2,
        velocity: Vec2,
        width: f32,
        height: f32,
        gravity: f32,
        fall: bool,
    ) -> (Vec2, Vec2);
}

/// `Gore.goreTime`'s static-constructor value (`Gore.cs:1507-1510`).
pub const GORE_TIME_DEFAULT: i32 = 600;

/// `NewGore`'s pool size: the scan bound and the sentinel it returns on a
/// refusal (`Gore.cs:1308-1320`), `Main.gore = new Gore[601]` holding it
/// (`Main.cs:66990`).
pub const GORE_SLOTS: usize = 600;
/// The refuse sentinel: `NewGore` returns 600 when no slot is taken
/// (`Gore.cs:1308`, `:1317-1320`); `Main.gore[600]` exists but is never
/// scanned (`Main.cs:66990`, scan `Gore.cs:1309`).
pub const GORE_SENTINEL: usize = 600;

/// The safe gore ids `DangerousGore` re-types when ChildSafety is armed
/// (`ChildSafety.cs:25-31`): `CreateBoolSet(11, 12, …)` defaults the table to
/// `false` and flips the listed ids to `true` (`SetFactory.cs:97-109`), so
/// `DangerousGore(id)` is true exactly for the ids **not** listed here.
const SAFE_GORE: &[i32] = &[
    11, 12, 13, 16, 17, 42, 53, 44, 51, 52, 53, 54, 55, 56, 57, 61, 62, 63, 67, 68, 69, 99, 106,
    120, 130, 131, 147, 148, 149, 150, 156, 166, 167, 168, 169, 170, 171, 172, 173, 174, 197, 198,
    199, 200, 201, 202, 203, 204, 213, 217, 218, 219, 220, 221, 222, 257, 265, 266, 267, 268, 269,
    276, 277, 278, 279, 280, 281, 282, 314, 315, 316, 321, 322, 326, 331, 360, 361, 362, 363, 364,
    365, 366, 367, 368, 369, 370, 375, 376, 377, 406, 407, 408, 409, 410, 411, 412, 413, 414, 415,
    416, 417, 418, 419, 420, 421, 422, 423, 424, 425, 426, 427, 428, 429, 430, 435, 436, 437, 521,
    522, 523, 525, 526, 527, 542, 570, 571, 572, 580, 581, 582, 603, 604, 605, 606, 610, 611, 612,
    613, 614, 615, 616, 617, 618, 639, 660, 661, 695, 696, 697, 698, 699, 700, 701, 702, 703, 704,
    705, 706, 707, 708, 709, 710, 711, 712, 713, 714, 715, 716, 717, 718, 719, 720, 721, 734, 728,
    729, 730, 731, 732, 733, 770, 825, 826, 827, 848, 849, 850, 851, 853, 854, 855, 856, 857, 858,
    859, 860, 861, 862, 892, 893, 898, 899, 907, 908, 909, 910, 911, 912, 913, 914, 915, 916, 917,
    918, 919, 920, 921, 922, 923, 924, 925, 926, 939, 940, 941, 942, 943, 964, 965, 966, 970, 971,
    972, 973, 992, 1007, 1008, 1024, 1025, 1026, 1027, 1028, 1029, 1030, 1060, 1066, 1087, 1088,
    1089, 1090, 1091, 1092, 1093, 1113, 1114, 1115, 1116, 1117, 1118, 1119, 1120, 1121, 1122, 1123,
    1124, 1125, 1126, 1127, 1128, 1129, 1130, 1131, 1132, 1133, 1134, 1135, 1136, 1137, 1138, 1139,
    1140, 1141, 1142, 1143, 1144, 1145, 1146, 1147, 1160, 1161, 1162, 1201, 1202, 1203, 1204, 1205,
    1206, 1207, 1208, 1209, 1210, 1211, 1212, 1213, 1218, 1225, 1226, 1248, 1249, 1250, 1251, 1252,
    1253, 1254, 1255, 1257, 1258, 1261, 1269, 1278, 1333, 1334, 1346, 1361, 1372, 1373, 1374, 1375,
    1376, 1383, 1394, 1395, 1396, 1397, 1398, 1399, 1400, 1401, 1402,
];

/// `Terraria.GameContent.ChildSafety` — the blood gate, carried per-pool like
/// `particles.rs`. `Disabled` is `true` by default (`ChildSafety.cs:33`), i.e.
/// the gate is off until the App arms it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChildSafety {
    /// `ChildSafety.Disabled` (`ChildSafety.cs:33`).
    pub disabled: bool,
}

impl Default for ChildSafety {
    fn default() -> Self {
        Self { disabled: true }
    }
}

impl ChildSafety {
    /// `ChildSafety.DangerousGore` (`ChildSafety.cs:25-27`):
    /// `!SafeGore[id]`, true exactly for the ids not in [`SAFE_GORE`].
    pub fn dangerous_gore(id: i32) -> bool {
        !SAFE_GORE.contains(&id)
    }
}

/// `Terraria.DataStructures.SpriteFrame` (`SpriteFrame.cs:4-77`) — the row/
/// column cursor of the gore's sheet cell. `PaddingX`/`PaddingY` (both 2,
/// `SpriteFrame.cs:7-12`) and `GetSourceRectangle` (`:70-76`) are draw-side and
/// wait for the asset seam; the pool's spawn writes here are exact.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SpriteFrame {
    /// `SpriteFrame.CurrentColumn` (`SpriteFrame.cs:17-27`).
    pub current_column: u8,
    /// `SpriteFrame.CurrentRow` (`SpriteFrame.cs:29-39`) — the C# `frame`
    /// alias reads and writes this (`Gore.cs:81-92`).
    pub current_row: u8,
    /// `SpriteFrame.ColumnCount` (`SpriteFrame.cs:14`).
    pub column_count: u8,
    /// `SpriteFrame.RowCount` (`SpriteFrame.cs:15`) — the C# `numFrames`
    /// alias (`Gore.cs:94-105`).
    pub row_count: u8,
}

impl SpriteFrame {
    /// `new SpriteFrame(columns, rows)` (`SpriteFrame.cs:46-56`): cursor at
    /// 0/0, padding 2/2 (draw-side, not carried).
    pub fn new(columns: u8, rows: u8) -> Self {
        Self {
            current_column: 0,
            current_row: 0,
            column_count: columns,
            row_count: rows,
        }
    }

    /// `SpriteFrame.GetSourceRectangle`'s size half (`SpriteFrame.cs:72-77`):
    /// the cell is `texture / count` (integer division), minus the 2 px
    /// padding on an axis with more than one cell. The `max(1)` guards only
    /// fire on a zero-count frame the pool never produces (`Gore.Frame`
    /// starts at `new SpriteFrame(1, 1)`, `Gore.cs:42`). The draw pass keeps
    /// its own copy with the cursor offsets (`render/gore.rs::source_rect`).
    pub fn source_size(&self, tex_w: u32, tex_h: u32) -> (i32, i32) {
        let num = (tex_w / self.column_count.max(1) as u32) as i32;
        let num2 = (tex_h / self.row_count.max(1) as u32) as i32;
        (
            num - if self.column_count != 1 { 2 } else { 0 },
            num2 - if self.row_count != 1 { 2 } else { 0 },
        )
    }
}

/// One gore record — `Gore.cs:14-42` with the static `goreTime` carried by the
/// pool ([`GorePool::gore_time`]).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Gore {
    /// `Gore.position` (`Gore.cs:18`).
    pub position: Vec2,
    /// `Gore.velocity` (`Gore.cs:20`).
    pub velocity: Vec2,
    /// `Gore.rotation` (`Gore.cs:22`).
    pub rotation: f32,
    /// `Gore.scale` (`Gore.cs:24`).
    pub scale: f32,
    /// `Gore.alpha` (`Gore.cs:26`).
    pub alpha: i32,
    /// `Gore.type` (`Gore.cs:28`).
    pub gore_type: i32,
    /// `Gore.light` (`Gore.cs:30`).
    pub light: f32,
    /// `Gore.active` (`Gore.cs:32`).
    pub active: bool,
    /// `Gore.sticky` (`Gore.cs:34`) — `true` by default.
    pub sticky: bool,
    /// `Gore.timeLeft` (`Gore.cs:36`) — `Gore.goreTime` by default.
    pub time_left: i32,
    /// `Gore.behindTiles` (`Gore.cs:38`).
    pub behind_tiles: bool,
    /// `Gore.frameCounter` (`Gore.cs:40`).
    pub frame_counter: u8,
    /// `Gore.Frame` (`Gore.cs:42`) — `new SpriteFrame(1, 1)` by default.
    pub frame: SpriteFrame,
    /// The port-side `TextureAssets.Gore[type]` size (W4-g68): `Some` when
    /// the sheet is packed (the `IsLoaded == true` branch of
    /// `Width`/`Height`/`AABBRectangle`), `None` = unloaded, the C#'s `1f`
    /// fallback. The pool refreshes it from its injected size map at spawn
    /// and per update tick (the type can re-type inside `NewGore`).
    pub texture_size: Option<(u32, u32)>,
}

impl Gore {
    /// The struct-initializer shape: every field of a fresh `new Gore()`
    /// (`Gore.cs:18-42`) — `sticky = true`, `timeLeft = goreTime`,
    /// `Frame = new SpriteFrame(1, 1)`, everything else default.
    fn fresh(gore_time: i32) -> Self {
        Gore {
            sticky: true,
            time_left: gore_time,
            frame: SpriteFrame::new(1, 1),
            ..Gore::default()
        }
    }

    /// `Gore.Width` (`Gore.cs:44-54`): with the texture unloaded the C#
    /// returns `1f`; a loaded sheet gives `scale * source-rect width`.
    /// W4-g68: the App injects the packed sheet sizes
    /// ([`GorePool::set_texture_sizes`]) — an id absent from the map is the
    /// unloaded fallback, exactly the C#'s `IsLoaded` test.
    pub fn width(&self) -> f32 {
        match self.texture_size {
            Some((w, h)) => self.scale * self.frame.source_size(w, h).0 as f32,
            None => 1.0,
        }
    }

    /// `Gore.Height` (`Gore.cs:56-66`): as [`Gore::width`].
    pub fn height(&self) -> f32 {
        match self.texture_size {
            Some((w, h)) => self.scale * self.frame.source_size(w, h).1 as f32,
            None => 1.0,
        }
    }

    /// `Gore.AABBRectangle` (`Gore.cs:68-79`): with the sheet packed (the
    /// W4-g68 seam) `(int)position` plus the `(int)(sourceRect * scale)`
    /// size — note the size casts truncate the PRODUCT, unlike
    /// `Width`/`Height`'s float getters. Unloaded: `(0, 0, 1, 1)`.
    pub fn aabb_rectangle(&self) -> (i32, i32, i32, i32) {
        match self.texture_size {
            Some((w, h)) => {
                let (sw, sh) = self.frame.source_size(w, h);
                (
                    self.position.x as i32,
                    self.position.y as i32,
                    (sw as f32 * self.scale) as i32,
                    (sh as f32 * self.scale) as i32,
                )
            }
            None => (0, 0, 1, 1),
        }
    }
}

/// The `Main.gore` pool: 601 slots (0..600 scanned, 600 the sentinel) plus
/// the static `goreTime` (`Gore.cs:16`, `:1507-1510`) and the blood gate.
pub struct GorePool {
    slots: Vec<Gore>,
    /// `Gore.goreTime` (`Gore.cs:16`) — the static constructor seeds 600
    /// (`Gore.cs:1507-1510`); `WorldGen` style consumers may write it.
    pub gore_time: i32,
    /// The blood gate; `disabled` defaults `true` (`ChildSafety.cs:33`).
    pub child_safety: ChildSafety,
    /// The `TextureAssets.Gore[type]` sizes (W4-g68 asset seam): the App
    /// injects its packed `Gore_<id>` sheet sizes once; an absent id is the
    /// C#'s unloaded-texture branch (`Width`/`Height` → `1f`). Empty by
    /// default, so pool-only consumers keep the recorded fallback.
    texture_sizes: std::collections::HashMap<i32, (u32, u32)>,
}

impl Default for GorePool {
    fn default() -> Self {
        Self::new()
    }
}

impl GorePool {
    /// `Main.gore = new Gore[601]` with each slot's field initializer
    /// (`Main.cs:66990`, `Gore.cs:18-42`).
    pub fn new() -> Self {
        let gore_time = GORE_TIME_DEFAULT;
        let mut slots = Vec::with_capacity(GORE_SENTINEL + 1);
        for _ in 0..=GORE_SENTINEL {
            slots.push(Gore::fresh(gore_time));
        }
        Self {
            slots,
            gore_time,
            child_safety: ChildSafety::default(),
            texture_sizes: std::collections::HashMap::new(),
        }
    }

    /// The W4-g68 asset seam: inject the packed `Gore_<id>` sheet sizes (the
    /// App's `Lookups::gore_sheets`). Ids present here answer the C#'s
    /// `TextureAssets.Gore[type].IsLoaded == true` branch; absent ids keep
    /// the `1f` fallback. Refreshes every live slot's cached size at once.
    pub fn set_texture_sizes(&mut self, sizes: std::collections::HashMap<i32, (u32, u32)>) {
        self.texture_sizes = sizes;
        for num in 0..=GORE_SENTINEL {
            self.refresh_texture_size(num);
        }
    }

    /// Re-read the slot's `texture_size` from the injected map (the type may
    /// have re-typed inside `NewGore`'s ChildSafety or 411-430 arms).
    fn refresh_texture_size(&mut self, num: usize) {
        self.slots[num].texture_size = self
            .texture_sizes
            .get(&self.slots[num].gore_type)
            .copied();
    }

    /// The whole slot array, sentinel included (`Main.gore`).
    pub fn slots(&self) -> &[Gore] {
        &self.slots
    }

    /// `Main.gore[i]`.
    pub fn slot(&self, i: usize) -> &Gore {
        &self.slots[i]
    }

    /// `Main.gore[i]` mutable.
    pub fn slot_mut(&mut self, i: usize) -> &mut Gore {
        &mut self.slots[i]
    }

    /// `Gore.NewGorePerfect` (`Gore.cs:1273-1279`): `NewGoreDirect`, then the
    /// position/velocity overwrite. Returns [`GORE_SENTINEL`] untouched on a
    /// refused spawn (the C# `Main.gore[600]` read).
    pub fn new_gore_perfect<R: GoreRng>(
        &mut self,
        rng: &mut R,
        world: &World,
        position: Vec2,
        velocity: Vec2,
        gore_type: i32,
        scale: f32,
    ) -> usize {
        let num = self.new_gore(rng, world, position, velocity, gore_type, scale);
        if num != GORE_SENTINEL {
            let gore = self.slot_mut(num);
            gore.position = position;
            gore.velocity = velocity;
        }
        num
    }

    /// `Gore.NewGoreDirect` (`Gore.cs:1281-1284`): `NewGore`, then the slot
    /// reference (the returned index; [`GORE_SENTINEL`] when refused).
    pub fn new_gore_direct<R: GoreRng>(
        &mut self,
        rng: &mut R,
        world: &World,
        position: Vec2,
        velocity: Vec2,
        gore_type: i32,
        scale: f32,
    ) -> usize {
        self.new_gore(rng, world, position, velocity, gore_type, scale)
    }

    /// `Gore.NewGore` (`Gore.cs:1286-1462`) — the only allocator, verbatim:
    /// guards → first-inactive-slot scan → default block → ChildSafety →
    /// sticky/timeLeft chain → per-type arms → out-of-world deactivate.
    /// Returns the slot index, or [`GORE_SENTINEL`]. Every roll hits the
    /// caller's stream (the RNG-order contract in the module docs).
    pub fn new_gore<R: GoreRng>(
        &mut self,
        rng: &mut R,
        world: &World,
        position: Vec2,
        velocity: Vec2,
        gore_type: i32,
        scale: f32,
    ) -> usize {
        // `Main.netMode == 2` (`Gore.cs:1288-1291`): server-only, dropped
        // (single-player port). `Main.gamePaused` (`:1292-1295`) and
        // `WorldGen.isGeneratingOrLoadingWorld` (`:1296-1299`) are not carried
        // by this module's seam yet — they read `Main` statics like the dust
        // pool's `SpawnCtx`; the App-side wiring is a later step. The
        // `Main.rand == null` re-seed (`:1300-1303`) has no port equivalent:
        // streams are caller-owned and never null.
        if gore_type == -1 {
            // `Gore.cs:1304-1307`.
            return GORE_SENTINEL;
        }
        let mut num = GORE_SENTINEL;
        // First-inactive-slot scan (`Gore.cs:1308-1320`); the sentinel stands
        // when the pool is full.
        for (i, slot) in self.slots[..GORE_SLOTS].iter().enumerate() {
            if !slot.active {
                num = i;
                break;
            }
        }
        if num == GORE_SENTINEL {
            return num;
        }
        let mut gore_type = gore_type;
        // Default block (`Gore.cs:1321-1333`).
        {
            let gore = self.slot_mut(num);
            gore.frame = SpriteFrame::new(1, 1);
            gore.frame_counter = 0;
            gore.behind_tiles = false;
            gore.light = 0.0;
            gore.position = position;
            gore.velocity = velocity;
            gore.velocity.y -= rng.next_range(10, 31) as f32 * 0.1;
            gore.velocity.x += rng.next_range(-20, 21) as f32 * 0.1;
            gore.gore_type = gore_type;
            gore.active = true;
            gore.alpha = 0;
            gore.rotation = 0.0;
            gore.scale = scale;
        }
        // ChildSafety arm (`Gore.cs:1334-1340`).
        if !self.child_safety.disabled && ChildSafety::dangerous_gore(gore_type) {
            gore_type = rng.next_range(11, 14);
            self.slot_mut(num).gore_type = gore_type;
            self.slot_mut(num).scale = rng.next_float() * 0.5 + 0.5;
            self.slot_mut(num).velocity /= 2.0;
        }
        // Sticky/timeLeft chain (`Gore.cs:1341-1354`).
        if self.gore_time == 0
            || matches!(gore_type, 11 | 12 | 13 | 16 | 17 | 61 | 62 | 63 | 99 | 220 | 221 | 222
                | 435 | 436 | 437)
            || (861..=862).contains(&gore_type)
        {
            let gore = self.slot_mut(num);
            gore.sticky = false;
        } else if (375..=377).contains(&gore_type) {
            // `Gore.cs:1345-1349`.
            let gore = self.slot_mut(num);
            gore.sticky = false;
            gore.alpha = 100;
        } else {
            // `Gore.cs:1350-1354`.
            let gore_time = self.gore_time;
            let gore = self.slot_mut(num);
            gore.sticky = true;
            gore.time_left = gore_time;
        }
        // `GoreID.Sets.IsDrip` arm (`Gore.cs:1355-1360`): the drip types get a
        // 15-row sheet, sit behind the tiles and live three times as long.
        if gore_type >= 0
            && (gore_type as usize) < crate::gore_id::GORE_COUNT
            && crate::gore_id::is_drip(gore_type as usize)
        {
            let gore_time = self.gore_time;
            let gore = self.slot_mut(num);
            gore.frame.row_count = 15;
            gore.behind_tiles = true;
            gore.time_left = gore_time * 3;
        }
        // Type 16/17 arm (`Gore.cs:1361-1366`).
        if gore_type == 16 || gore_type == 17 {
            let gore = self.slot_mut(num);
            gore.alpha = 100;
            gore.scale = 0.7;
            gore.light = 1.0;
        }
        if (570..=572).contains(&gore_type) {
            // `Gore.cs:1367-1370`.
            self.slot_mut(num).velocity = velocity;
        }
        if gore_type == 1201 || gore_type == 1208 {
            // `Gore.cs:1371-1374`.
            self.slot_mut(num).frame = SpriteFrame::new(1, 4);
        }
        if gore_type == 1217 || gore_type == 1218 {
            // `Gore.cs:1375-1378`.
            self.slot_mut(num).frame = SpriteFrame::new(1, 3);
        }
        // `Gore.cs:1379-1389` (type 1225): the frame/timeLeft/sticky writes
        // are unconditional; the position adjust rides the loaded-texture
        // gate (W4-g68 wiring-2: `texture_sizes` IS the `IsLoaded` test).
        // The C# reads the RAW asset `Width()/Height()` (`Utils.cs:1301-
        // 1316`), NOT the framed cell — and `Width()/2` is INTEGER division
        // before the float cast, while the Y term is `Height()*Scale/2f`.
        // Both terms use the `Position`/`Scale` ARGUMENTS (a ChildSafety
        // re-scale wrote `gore.scale`, not `Scale`).
        if gore_type == 1225 {
            let gore = self.slot_mut(num);
            gore.frame = SpriteFrame::new(1, 3);
            gore.time_left = 10 + rng.next_max(6);
            gore.sticky = false;
            if let Some(&(w, h)) = self.texture_sizes.get(&gore_type) {
                let gore = self.slot_mut(num);
                gore.position.x = position.x - (w / 2) as f32 * scale;
                gore.position.y = position.y - h as f32 * scale / 2.0;
            }
        }
        // `GoreID.Sets.SpecialAI` arms (`Gore.cs:1390-1406`), the 411-430
        // (`:1407-1421`), 907-909 (`:1422-1429`) and SpecialAI == 2
        // (`:1430-1441`) chains.
        let special_ai = if gore_type >= 0 && (gore_type as usize) < crate::gore_id::GORE_COUNT {
            crate::gore_id::special_ai(gore_type as usize)
        } else {
            0
        };
        if special_ai == 3 {
            // `Gore.cs:1391-1402`: the velocity rolls first, then the sheet's
            // row and the frame counter — four draws, in that order.
            let vel_x = (rng.next_float() - 0.5) * 1.0;
            let vel_y = rng.next_float() * (std::f32::consts::PI * 2.0);
            let leaf = (910..=925).contains(&gore_type)
                || (1113..=1121).contains(&gore_type)
                || (1248..=1255).contains(&gore_type)
                || gore_type == 1257
                || gore_type == 1278;
            let row = rng.next_max(8);
            let counter = rng.next_max(8);
            let gore = self.slot_mut(num);
            gore.velocity = Vec2::new(vel_x, vel_y);
            gore.frame = SpriteFrame {
                current_column: 0,
                current_row: row as u8,
                column_count: if leaf { 32 } else { 1 },
                row_count: 8,
            };
            gore.frame_counter = counter as u8;
        }
        if special_ai == 1 {
            // `Gore.cs:1403-1406`: two `NextFloat` draws, no other roll.
            let vel_x = (rng.next_float() - 0.5) * 3.0;
            let vel_y = rng.next_float() * (std::f32::consts::PI * 2.0);
            self.slot_mut(num).velocity = Vec2::new(vel_x, vel_y);
        }
        // The 411-430 arm (`Gore.cs:1407-1421`), behind the loaded-texture
        // gate (W4-g68 wiring-2). The C# order: the X/Y position adjusts
        // (Y is the FULL height, not half), the velocity.Y then velocity.X
        // scale rolls, the `Next(4)*5` type bump, the timeLeft roll — and
        // ONLY when `goreTime == 0` a FIFTH draw (`Next(150, 600)`)
        // overwrites timeLeft. The gate reads the ORIGINAL `Type` (a
        // ChildSafety re-type to 11-13 never lands in 411-430 anyway); the
        // type bump can push past 430 (428+15=443) — the later
        // `refresh_texture_size` re-reads the FINAL type's sheet, exactly
        // the C#'s lazy `TextureAssets.Gore[type]` reads at draw time.
        if (411..=430).contains(&gore_type) {
            if let Some(&(w, h)) = self.texture_sizes.get(&gore_type) {
                let vy_scale = rng.next_range(90, 150) as f32 * 0.01;
                let vx_scale = rng.next_range(40, 90) as f32 * 0.01;
                let bump = rng.next_max(4) * 5;
                let gore_time = self.gore_time;
                let life = rng.next_range(gore_time / 2, gore_time * 2);
                let gore = self.slot_mut(num);
                gore.position.x = position.x - (w / 2) as f32 * scale;
                gore.position.y = position.y - h as f32 * scale;
                gore.velocity.y *= vy_scale;
                gore.velocity.x *= vx_scale;
                gore.gore_type += bump;
                gore.time_left = life;
                gore.sticky = true;
                if gore_time == 0 {
                    // `:1417-1420`: the fifth draw, only in the zero-time
                    // branch (the roll above still fired).
                    let life = rng.next_range(150, 600);
                    self.slot_mut(num).time_left = life;
                }
            }
        }
        if (907..=909).contains(&gore_type) {
            // `Gore.cs:1422-1429`: the sheet row, then the frame counter.
            let row = rng.next_max(3);
            let counter = rng.next_max(5);
            let gore = self.slot_mut(num);
            gore.sticky = true;
            gore.frame.row_count = 3;
            gore.frame.current_row = row as u8;
            gore.frame_counter = counter as u8;
            gore.rotation = 0.0;
        }
        if special_ai == 2 {
            // `Gore.cs:1430-1441`: the sticky flag is unconditional; the
            // alpha/velocity/position/timeLeft writes + the ONE timeLeft
            // roll sit behind `TextureAssets.Gore[Type].IsLoaded`
            // (W4-g68 wiring-2). The velocity resets to the ARGUMENT
            // (the default-block kick is discarded), the position adjusts
            // read the raw asset halves (X integer-divided first), and
            // `Next(goreTime/2, goreTime+1)` is the arm's only draw.
            self.slot_mut(num).sticky = false;
            if let Some(&(w, h)) = self.texture_sizes.get(&gore_type) {
                let gore_time = self.gore_time;
                let life = rng.next_range(gore_time / 2, gore_time + 1);
                let gore = self.slot_mut(num);
                gore.alpha = 150;
                gore.velocity = velocity;
                gore.position.x = position.x - (w / 2) as f32 * scale;
                gore.position.y = position.y - h as f32 * scale / 2.0;
                gore.time_left = life;
            }
        }
        match special_ai {
            // `Gore.cs:1442-1456`: the 4/5/6 rows.
            4 => {
                let gore = self.slot_mut(num);
                gore.alpha = 254;
                gore.time_left = 300;
            }
            5 => {
                let gore = self.slot_mut(num);
                gore.alpha = 254;
                gore.time_left = 240;
            }
            6 => {
                let gore = self.slot_mut(num);
                gore.alpha = 254;
                gore.time_left = 480;
            }
            _ => {}
        }
        // Deactivate if outside of the world (`Gore.cs:1457-1460`): returns
        // the sentinel with the slot left inactive — the C#'s own shape.
        if self.deactivate_if_outside_of_world(num, world) {
            return GORE_SENTINEL;
        }
        // The W4-g68 asset seam: the final type (post ChildSafety re-type)
        // picks the texture size the `Width`/`Height` getters read.
        self.refresh_texture_size(num);
        num
    }

    /// `Gore.DeactivateIfOutsideOfWorld` (`Gore.cs:292-306`), private in the
    /// C#: `position.ToTileCoordinates()` (`Utils.cs:1807-1810`, the `>> 4`
    /// shift), then the `InWorld` fluff-0 bounds (`WorldGen.cs:9688-9695`) and
    /// the `Main.tile[x, y] == null` probe. The port's `World` answers
    /// `None` for the same cells the C# holds a null tile for.
    fn deactivate_if_outside_of_world(&mut self, num: usize, world: &World) -> bool {
        let gore = self.slot(num);
        // `(int)vec >> 4` (`Utils.cs:1809`): the shift truncates toward
        // positive infinity for negatives, exactly like the C#.
        let x = (gore.position.x as i32) >> 4;
        let y = (gore.position.y as i32) >> 4;
        let outside = x < 0
            || x >= world.width
            || y < 0
            || y >= world.height
            || world.tile(x, y).is_none();
        if outside {
            self.slot_mut(num).active = false;
            return true;
        }
        false
    }
}

/// `WorldGen.SolidTile` (`WorldGen.cs:69957-69974` and `:70111-70121`) — the
/// solid test of the leaf/cloud arms: a null tile counts as solid, otherwise
/// the tile must be active (and not framed off), solid, not a solid-top, not
/// a half brick and not sloped. The living-wood blocks (192) are walk-through
/// after the worldgen pass (`WorldGen.cs:16607`), as in the port's collision.
fn solid_tile(tile: Option<&crate::tile::Tile>) -> bool {
    let Some(t) = tile else { return true; };
    t.active()
        && !t.inactive()
        && t.kind != 192
        && crate::tiledata::TILE_SOLID.has(t.kind)
        && !crate::tiledata::TILE_SOLID_TOP.has(t.kind)
        && !t.half_brick()
        && t.slope() == 0
}

/// `Vector2.ToTileCoordinates` (`Utils.cs:1807-1810`) — `(int)x >> 4` (the
/// arithmetic shift, so a negative coordinate stays negative).
fn to_tile_coordinates(v: Vec2) -> (i32, i32) {
    ((v.x as i32) >> 4, (v.y as i32) >> 4)
}

/// `Rectangle.Intersects` (`Microsoft.Xna.Framework.Rectangle`) — the strict
/// overlap test (touching edges do not overlap). The tuple is
/// `(left, top, width, height)`.
fn rect_intersects(a: (i32, i32, i32, i32), b: (i32, i32, i32, i32)) -> bool {
    a.0 < b.0 + b.2 && a.0 + a.2 > b.0 && a.1 < b.1 + b.3 && a.1 + a.3 > b.1
}

impl GorePool {
    /// `Main.UpdateWorld_Gores` (`Main.cs:17971-17996`) — the per-frame walk
    /// over the 600 live slots (the sentinel 600 is never updated).
    ///
    /// W4-g68: the dust seam — `dust`/`dust_ctx` carry the App's `Main.dust`
    /// pool and the per-call `Dust.NewDust` inputs; `None` keeps the recorded
    /// dust-seam drops (the lava leaf puff, the 1218 coin burst) inactive.
    /// When present, the spawns roll the SAME stage stream (`rng`) the gore
    /// walk does — the C# runs both inside `SwapRandom("UpdateGores")`.
    pub fn update<R: GoreRng>(
        &mut self,
        rng: &mut R,
        world: &World,
        collision: &dyn GoreCollision,
        wind_for_visuals: f32,
        mut dust: Option<(&mut crate::particles::DustPool, &crate::particles::SpawnCtx)>,
    ) {
        for num in 0..GORE_SLOTS {
            if !self.slots[num].active {
                continue;
            }
            let dust_slot = dust.as_mut().map(|(p, c)| (&mut **p, &**c));
            self.update_slot(rng, world, collision, wind_for_visuals, num, dust_slot);
        }
    }

    /// `Gore.Update` (`Gore.cs:308-1083`) — one gore's frame.
    ///
    /// Documented drops (the port has no texture tracking, no entity lights
    /// and no player/sound seams in `tr-core`): the texture-gated
    /// position/light/player/dust writes (`Gore.cs:794-806`, `:825-841`,
    /// `:850-864`, `:883-911`, `:1064-1082`) and the drip's sound and ripple
    /// (`:953`, `:965-968`). None of them draws a random number, so the
    /// stream order is untouched. The lava leaf's five-dust puff
    /// (`:1119-1122`) IS wired (W4-g68): it fires when the caller hands the
    /// `dust` seam in, on the same stage stream.
    fn update_slot<R: GoreRng>(
        &mut self,
        rng: &mut R,
        world: &World,
        collision: &dyn GoreCollision,
        wind_for_visuals: f32,
        num: usize,
        dust: Option<(&mut crate::particles::DustPool, &crate::particles::SpawnCtx)>,
    ) {
        // `Gore.cs:314-325`: the sticky clamp — the world check first, then
        // the speed cap.
        // W4-g68: re-read the texture size each tick — callers may re-type a
        // slot through `slot_mut` (the drip rows' paint swaps etc.), and the
        // C#'s `Width`/`Height` getters re-evaluate `TextureAssets.Gore[type]`
        // on every read.
        self.refresh_texture_size(num);
        if self.slots[num].sticky {
            if self.deactivate_if_outside_of_world(num, world) {
                return;
            }
            let speed = self.slots[num].velocity.length();
            if speed > 32.0 {
                self.slots[num].velocity *= 32.0 / speed;
            }
        }
        // The `SpecialAI` switch (`Gore.cs:326-340`): the four cloud/spark
        // rows own the whole frame.
        let gore_type = self.slots[num].gore_type;
        let special_ai = crate::gore_id::special_ai_checked(gore_type);
        match special_ai {
            4 => {
                self.update_ambient_floor_cloud(world, wind_for_visuals, num);
                return;
            }
            5 => {
                self.update_ambient_airborne_cloud(rng, world, num);
                return;
            }
            6 => {
                self.update_fog_machine_cloud(rng, world, num);
                return;
            }
            7 => {
                self.update_lightning_bunny_sparks(rng, num);
                return;
            }
            _ => {}
        }
        // `Gore.cs:341-345`: the 1217/1218 first-frame row roll.
        if (gore_type == 1217 || gore_type == 1218) && self.slots[num].frame_counter == 0 {
            self.slots[num].frame_counter = 1;
            self.slots[num].frame.current_row = rng.next_max(3) as u8;
        }
        // `Gore.cs:346`: `flag` — the 1024-1026 ogre-spit rows.
        let flag = (1024..=1026).contains(&gore_type);
        // `Gore.cs:347-366`: the 276-282 rows.
        if (276..=282).contains(&gore_type) {
            self.slots[num].velocity.x *= 0.98;
            self.slots[num].velocity.y *= 0.98;
            if self.slots[num].velocity.y < self.slots[num].scale {
                self.slots[num].velocity.y += 0.05;
            }
            if self.slots[num].velocity.y as f64 > 0.1 {
                if self.slots[num].velocity.x > 0.0 {
                    self.slots[num].rotation += 0.01;
                } else {
                    self.slots[num].rotation -= 0.01;
                }
            }
        }
        // `Gore.cs:367-761`: the per-type chain.
        if (570..=572).contains(&gore_type) {
            // `Gore.cs:367-377`.
            self.slots[num].scale -= 0.001;
            if self.slots[num].scale as f64 <= 0.01 {
                self.slots[num].scale = 0.01;
                self.slots[num].time_left = 0;
            }
            self.slots[num].sticky = false;
            self.slots[num].rotation = self.slots[num].velocity.x * 0.1;
        } else if gore_type >= 0
            && (gore_type as usize) < crate::gore_id::GORE_COUNT
            && crate::gore_id::is_drip(gore_type as usize)
        {
            // `Gore.cs:378-529`: the drip animation.
            self.update_drip(rng, world, num);
        } else if matches!(
            gore_type,
            11 | 12
                | 13
                | 61
                | 62
                | 63
                | 99
                | 220..=222
                | 375..=377
                | 435..=437
                | 861..=862
        ) {
            // `Gore.cs:530-540`: the smoke/gas rows shrink and fade.
            self.slots[num].velocity.y *= 0.98;
            self.slots[num].velocity.x *= 0.98;
            self.slots[num].scale -= 0.007;
            if (self.slots[num].scale as f64) < 0.1 {
                self.slots[num].scale = 0.1;
                self.slots[num].alpha = 255;
            }
        } else if gore_type == 16 || gore_type == 17 {
            // `Gore.cs:541-551`.
            self.slots[num].velocity.y *= 0.98;
            self.slots[num].velocity.x *= 0.98;
            self.slots[num].scale -= 0.01;
            if (self.slots[num].scale as f64) < 0.1 {
                self.slots[num].scale = 0.1;
                self.slots[num].alpha = 255;
            }
        } else if gore_type == 1201 {
            // `Gore.cs:552-589`: the chum bucket.
            self.update_chum_bucket(rng, world, num);
        } else if gore_type == 1208 {
            // `Gore.cs:590-671`: the floating chum chunk.
            self.update_chum_floating_chunk(rng, world, num);
        } else if gore_type == 331 {
            // `Gore.cs:672-678`.
            self.slots[num].alpha += 5;
            self.slots[num].velocity.y *= 0.95;
            self.slots[num].velocity.x *= 0.95;
            self.slots[num].rotation = self.slots[num].velocity.x * 0.1;
        } else if special_ai == 3 {
            // `Gore.cs:679-690`: the leaf rows walk their sheet.
            self.slots[num].frame_counter = self.slots[num].frame_counter.wrapping_add(1);
            if self.slots[num].frame_counter >= 8 && self.slots[num].velocity.y > 0.2 {
                self.slots[num].frame_counter = 0;
                let row = self.slots[num].frame.current_row;
                let num15 = row / 4;
                let row = row.wrapping_add(1);
                self.slots[num].frame.current_row = row;
                if row as i32 >= 4 + num15 as i32 * 4 {
                    self.slots[num].frame.current_row = num15 * 4;
                }
            }
        } else if special_ai != 1 && special_ai != 2 {
            // `Gore.cs:691-761`.
            if (907..=909).contains(&gore_type) {
                // `Gore.cs:693-717`: the balloons.
                self.slots[num].rotation = 0.0;
                self.slots[num].velocity.x *= 0.98;
                if self.slots[num].velocity.y > 0.0 && self.slots[num].velocity.y < 0.001 {
                    self.slots[num].velocity.y = -0.5 + rng.next_float() * -3.0;
                }
                if self.slots[num].velocity.y > -1.0 {
                    self.slots[num].velocity.y -= 0.1;
                }
                if self.slots[num].scale < 1.0 {
                    self.slots[num].scale += 0.1;
                }
                self.slots[num].frame_counter = self.slots[num].frame_counter.wrapping_add(1);
                if self.slots[num].frame_counter >= 8 {
                    self.slots[num].frame_counter = 0;
                    let row = self.slots[num].frame.current_row.wrapping_add(1);
                    self.slots[num].frame.current_row = row;
                    if row >= 3 {
                        self.slots[num].frame.current_row = 0;
                    }
                }
            } else if gore_type == 1218 {
                // `Gore.cs:718-751`: the mimic coins.
                if self.slots[num].time_left > 8 {
                    self.slots[num].time_left = 8;
                }
                self.slots[num].velocity.x *= 0.95;
                if self.slots[num].velocity.x.abs() <= 0.1 {
                    self.slots[num].velocity.x = 0.0;
                }
                if self.slots[num].alpha < 100
                    && self.slots[num].velocity.length() > 0.0
                    && rng.next_max(5) == 0
                {
                    // The dust burst (`Gore.cs:744-747`) waits for the dust
                    // seam; the roll above is the C#'s and stays.
                }
                self.slots[num].velocity.y += 0.2;
                self.slots[num].rotation = 0.0;
            } else if !(411..=430).contains(&gore_type) {
                // `Gore.cs:752-756`: the generic fall.
                self.slots[num].velocity.y += 0.2;
                self.slots[num].rotation += self.slots[num].velocity.x * 0.05;
            } else if special_ai != 3 {
                // `Gore.cs:757-760`.
                self.slots[num].rotation += self.slots[num].velocity.x * 0.1;
            }
        }
        // `Gore.cs:762-766`: the 580-582 rows.
        if (580..=582).contains(&gore_type) {
            self.slots[num].rotation = 0.0;
            self.slots[num].velocity.x *= 0.95;
        }
        // `Gore.cs:767-843`: the `SpecialAI == 2` smoke row.
        if special_ai == 2 {
            self.update_special_ai_two(rng, wind_for_visuals, num);
        }
        // `Gore.cs:844-1024`: the 411-430 rows and the drip/sticky/fade tail.
        if (411..=430).contains(&gore_type) {
            self.update_pages(rng, wind_for_visuals, num);
        } else if special_ai != 3 && special_ai != 1 {
            if gore_type >= 0
                && (gore_type as usize) < crate::gore_id::GORE_COUNT
                && crate::gore_id::is_drip(gore_type as usize)
            {
                // `Gore.cs:915-977`: the drip's collision pass.
                self.update_drip_collision(world, collision, num);
            } else if self.slots[num].sticky {
                // `Gore.cs:978-1019`: the sticky gore rests and fades.
                self.update_sticky(world, collision, flag, num);
            } else {
                // `Gore.cs:1020-1023`: the free-floating fade.
                let alpha = crate::gore_id::disappear_speed_alpha_checked(gore_type);
                self.slots[num].alpha += 2 * alpha;
            }
        }
        // `Gore.cs:1025-1042`: the 907-909 slope pass.
        if (907..=909).contains(&gore_type) {
            let num37 = ((32.0 * 0.9) as i32) as f32;
            let (position, velocity) = collision.slope_collision(
                world,
                self.slots[num].position,
                self.slots[num].velocity,
                num37,
                num37,
                0.0,
                true,
            );
            self.slots[num].position = position;
            self.slots[num].velocity = velocity;
        }
        // `Gore.cs:1043-1054`: the final move.
        if special_ai == 1 {
            self.update_sail(world, collision, num);
        } else if special_ai == 3 {
            self.update_leaf(world, collision, wind_for_visuals, num, rng, dust);
        } else {
            let velocity = self.slots[num].velocity;
            self.slots[num].position += velocity;
        }
        // `Gore.cs:1055-1058`: the fade-out.
        if self.slots[num].alpha >= 255 {
            self.slots[num].active = false;
        }
        // `Gore.cs:1059-1082`: the light — the port has no entity-light
        // system yet (S1-E-5), so the glow is dropped.
    }

    // ---- the `Update` helpers (`Gore.cs:110-1271`) ----

    /// `Gore.UpdateAmbientFloorCloud` (`Gore.cs:110-154`).
    fn update_ambient_floor_cloud(&mut self, world: &World, wind_for_visuals: f32, num: usize) {
        self.slots[num].time_left -= crate::gore_id::disappear_speed_checked(self.slots[num].gore_type);
        if self.slots[num].time_left <= 0 {
            self.slots[num].active = false;
            return;
        }
        let point = to_tile_coordinates(self.slots[num].position + Vec2::new(15.0, 0.0));
        let tile = world.tile(point.0, point.1);
        let tile2 = world.tile(point.0, point.1 + 1);
        let tile3 = world.tile(point.0, point.1 + 2);
        if tile.is_none() || tile2.is_none() || tile3.is_none() {
            self.slots[num].active = false;
            return;
        }
        // `:128`: a solid tile under the cloud, or two open ones, ends the drift.
        let mut flag = solid_tile(tile) || (!solid_tile(tile2) && !solid_tile(tile3));
        if self.slots[num].time_left <= 30 {
            flag = true;
        }
        self.slots[num].velocity.x = 0.4 * wind_for_visuals;
        if !flag {
            if self.slots[num].alpha > 220 {
                self.slots[num].alpha -= 1;
            }
        } else {
            self.slots[num].alpha += 1;
            if self.slots[num].alpha >= 255 {
                self.slots[num].active = false;
                return;
            }
        }
        let velocity = self.slots[num].velocity;
        self.slots[num].position += velocity;
    }

    /// `Gore.UpdateAmbientAirborneCloud` (`Gore.cs:156-201`).
    fn update_ambient_airborne_cloud<R: GoreRng>(&mut self, rng: &mut R, world: &World, num: usize) {
        self.slots[num].time_left -= crate::gore_id::disappear_speed_checked(self.slots[num].gore_type);
        if self.slots[num].time_left <= 0 {
            self.slots[num].active = false;
            return;
        }
        let point = to_tile_coordinates(self.slots[num].position + Vec2::new(15.0, 0.0));
        let velocity = self.slots[num].velocity;
        self.slots[num].rotation = velocity.y.atan2(velocity.x);
        let tile = world.tile(point.0, point.1);
        if tile.is_none() {
            self.slots[num].active = false;
            return;
        }
        let mut flag = solid_tile(tile);
        if self.slots[num].time_left <= 60 {
            flag = true;
        }
        if !flag {
            if self.slots[num].alpha > 240 && rng.next_max(5) == 0 {
                self.slots[num].alpha -= 1;
            }
        } else {
            if rng.next_max(5) == 0 {
                self.slots[num].alpha += 1;
            }
            if self.slots[num].alpha >= 255 {
                self.slots[num].active = false;
                return;
            }
        }
        let velocity = self.slots[num].velocity;
        self.slots[num].position += velocity;
    }

    /// `Gore.UpdateFogMachineCloud` (`Gore.cs:203-241`).
    fn update_fog_machine_cloud<R: GoreRng>(&mut self, rng: &mut R, world: &World, num: usize) {
        self.slots[num].time_left -= crate::gore_id::disappear_speed_checked(self.slots[num].gore_type);
        if self.slots[num].time_left <= 0 {
            self.slots[num].active = false;
            return;
        }
        let point = to_tile_coordinates(self.slots[num].position + Vec2::new(15.0, 0.0));
        let mut flag = solid_tile(world.tile(point.0, point.1));
        if self.slots[num].time_left <= 240 {
            flag = true;
        }
        if !flag {
            if self.slots[num].alpha > 225 && rng.next_max(2) == 0 {
                self.slots[num].alpha -= 1;
            }
        } else {
            if rng.next_max(2) == 0 {
                self.slots[num].alpha += 1;
            }
            if self.slots[num].alpha >= 255 {
                self.slots[num].active = false;
                return;
            }
        }
        let velocity = self.slots[num].velocity;
        self.slots[num].position += velocity;
    }

    /// `Gore.UpdateLightningBunnySparks` (`Gore.cs:243-260`).
    fn update_lightning_bunny_sparks<R: GoreRng>(&mut self, rng: &mut R, num: usize) {
        if self.slots[num].frame_counter == 0 {
            self.slots[num].frame_counter = 1;
            self.slots[num].frame.current_row = rng.next_max(3) as u8;
        }
        self.slots[num].time_left -= crate::gore_id::disappear_speed_checked(self.slots[num].gore_type);
        if self.slots[num].time_left <= 0 {
            self.slots[num].active = false;
            return;
        }
        // `:256` — `MathHelper.Lerp(255f, 0f, timeLeft / 15f)`, truncated.
        let t = self.slots[num].time_left as f32 / 15.0;
        self.slots[num].alpha = MathHelper::lerp(255.0, 0.0, t) as i32;
        // `:258` `Lighting.AddLight` — the entity-light seam (S1-E-5) is absent.
        let velocity = self.slots[num].velocity;
        self.slots[num].position += velocity;
    }

    /// `Gore.Update`'s `IsDrip` arm (`Gore.cs:378-529`) — the 15-frame drip
    /// sheet: the splash/slide/fade ladder and the mid-sheet `NewGore` spawn
    /// (`:440-443`), which rolls on the same stream.
    fn update_drip<R: GoreRng>(&mut self, rng: &mut R, world: &World, num: usize) {
        let gore_type = self.slots[num].gore_type;
        // `:380-391` — the alpha by type and depth.
        if gore_type == 943 || (1160..=1162).contains(&gore_type) {
            self.slots[num].alpha = 0;
        } else if (self.slots[num].position.y as f64) < world.world_surface * 16.0 + 8.0 {
            self.slots[num].alpha = 0;
        } else {
            self.slots[num].alpha = 100;
        }
        let mut num2: i32 = 4;
        self.slots[num].frame_counter = self.slots[num].frame_counter.wrapping_add(1);
        let frame = self.slots[num].frame.current_row;
        if frame <= 4 {
            // `:396-401` — the warm-up frames fall off the tile above.
            let num3 = (self.slots[num].position.x / 16.0) as i32;
            let num4 = (self.slots[num].position.y / 16.0) as i32 - 1;
            if world.in_bounds(num3, num4) && !world.tile_active(num3, num4) {
                self.slots[num].active = false;
            }
            // `:402-421` — the per-frame hang time (one roll each).
            if frame == 0 || frame == 1 || frame == 2 {
                num2 = 24 + rng.next_max(256);
            } else if frame == 3 {
                num2 = 24 + rng.next_max(96);
            } else if frame == 5 {
                num2 = 16 + rng.next_max(64);
            }
            if gore_type == 716 {
                num2 *= 2;
            }
            if gore_type == 717 {
                num2 *= 4;
            }
            if (gore_type == 943 || (1160..=1162).contains(&gore_type)) && frame < 6 {
                num2 = 4;
            }
            if self.slots[num].frame_counter as i32 >= num2 {
                self.slots[num].frame_counter = 0;
                self.slots[num].frame.current_row = self.slots[num].frame.current_row.wrapping_add(1);
                if self.slots[num].frame.current_row == 5 {
                    let position = self.slots[num].position;
                    let velocity = self.slots[num].velocity;
                    let idx = self.new_gore(rng, world, position, velocity, gore_type, 1.0);
                    self.slots[idx].frame.current_row = 9;
                    self.slots[idx].velocity = Vec2::ZERO;
                }
            }
        } else if frame <= 6 {
            // `:446-466` — the sheet's step-down frames.
            num2 = 8;
            if gore_type == 716 {
                num2 *= 2;
            }
            if gore_type == 717 {
                num2 *= 3;
            }
            if self.slots[num].frame_counter as i32 >= num2 {
                self.slots[num].frame_counter = 0;
                self.slots[num].frame.current_row = self.slots[num].frame.current_row.wrapping_add(1);
                if self.slots[num].frame.current_row == 7 {
                    self.slots[num].active = false;
                }
            }
        } else if frame <= 9 {
            // `:467-506` — the fall frames.
            num2 = 6;
            if gore_type == 716 {
                num2 = (num2 as f64 * 1.5) as i32;
                self.slots[num].velocity.y += 0.175;
            } else if gore_type == 717 {
                num2 *= 2;
                self.slots[num].velocity.y += 0.15;
            } else if gore_type == 943 {
                num2 = (num2 as f64 * 1.5) as i32;
                self.slots[num].velocity.y += 0.2;
            } else {
                self.slots[num].velocity.y += 0.2;
            }
            if (self.slots[num].velocity.y as f64) < 0.5 {
                self.slots[num].velocity.y = 0.5;
            }
            if self.slots[num].velocity.y > 12.0 {
                self.slots[num].velocity.y = 12.0;
            }
            if self.slots[num].frame_counter as i32 >= num2 {
                self.slots[num].frame_counter = 0;
                self.slots[num].frame.current_row = self.slots[num].frame.current_row.wrapping_add(1);
            }
            if self.slots[num].frame.current_row > 9 {
                self.slots[num].frame.current_row = 7;
            }
        } else {
            // `:507-528` — the rest frames, then the fade.
            if gore_type == 716 {
                num2 *= 2;
            } else if gore_type == 717 {
                num2 *= 6;
            }
            self.slots[num].velocity.y += 0.1;
            if self.slots[num].frame_counter as i32 >= num2 {
                self.slots[num].frame_counter = 0;
                self.slots[num].frame.current_row = self.slots[num].frame.current_row.wrapping_add(1);
            }
            self.slots[num].velocity = Vec2::ZERO;
            if self.slots[num].frame.current_row > 14 {
                self.slots[num].active = false;
            }
        }
    }

    /// `Gore.Update`'s drip collision pass (`Gore.cs:915-977`). Documented
    /// drops: the 716/1383 shimmer light (`:917-941`, needs
    /// `LiquidRenderer.GetShimmerBaseColor`), the `SoundEngine` hits (`:953`,
    /// `:965`) and the water-shader ripple (`:967`) — none draws a random
    /// number.
    fn update_drip_collision(&mut self, world: &World, collision: &dyn GoreCollision, num: usize) {
        let before = self.slots[num].velocity;
        let after = collision.tile_collision(world, self.slots[num].position, before, 16.0, 14.0);
        self.slots[num].velocity = after;
        if after != before {
            if self.slots[num].frame.current_row < 10 {
                self.slots[num].frame.current_row = 10;
                self.slots[num].frame_counter = 0;
            }
        } else if collision.wet_collision(world, self.slots[num].position + after, 16.0, 14.0) {
            if self.slots[num].frame.current_row < 10 {
                self.slots[num].frame.current_row = 10;
                self.slots[num].frame_counter = 0;
            }
            // `:969-975` — the drip settles on the liquid surface.
            let num34 = (self.slots[num].position.x + 8.0) as i32 / 16;
            let num35 = (self.slots[num].position.y + 14.0) as i32 / 16;
            if let Some(t) = world.tile(num34, num35) {
                if t.liquid > 0 {
                    self.slots[num].velocity = Vec2::ZERO;
                    self.slots[num].position.y =
                        (num35 * 16) as f32 - (t.liquid as i32 / 16) as f32;
                }
            }
        }
    }

    /// `Gore.Update`'s sticky arm (`Gore.cs:978-1019`): the tile pass at the
    /// texture-derived size (the `IsLoaded` arms fall back to the C#'s `32f`,
    /// and to `4f` for the ogre-spit rows), the rest friction and the fade.
    fn update_sticky(&mut self, world: &World, collision: &dyn GoreCollision, flag: bool, num: usize) {
        let mut num36: i32 = 32;
        if flag {
            num36 = 4;
        }
        num36 = (num36 as f32 * 0.9) as i32;
        let size = (num36 as f32 * self.slots[num].scale) as i32;
        let after = collision.tile_collision(
            world,
            self.slots[num].position,
            self.slots[num].velocity,
            size as f32,
            size as f32,
        );
        self.slots[num].velocity = after;
        if after.y == 0.0 {
            if flag {
                self.slots[num].velocity.x *= 0.94;
            } else {
                self.slots[num].velocity.x *= 0.97;
            }
            if self.slots[num].velocity.x > -0.01 && self.slots[num].velocity.x < 0.01 {
                self.slots[num].velocity.x = 0.0;
            }
        }
        if self.slots[num].time_left > 0 {
            self.slots[num].time_left -=
                crate::gore_id::disappear_speed_checked(self.slots[num].gore_type);
        } else {
            self.slots[num].alpha +=
                crate::gore_id::disappear_speed_alpha_checked(self.slots[num].gore_type);
        }
    }

    /// `Gore.Update`'s 1201 chum-bucket arm (`Gore.cs:552-589`).
    fn update_chum_bucket<R: GoreRng>(&mut self, rng: &mut R, world: &World, num: usize) {
        if self.slots[num].frame_counter == 0 {
            self.slots[num].frame_counter = 1;
            self.slots[num].frame.current_row = rng.next_max(4) as u8;
        }
        self.slots[num].scale -= 0.002;
        if (self.slots[num].scale as f64) < 0.1 {
            self.slots[num].scale = 0.1;
            self.slots[num].alpha = 255;
        }
        self.slots[num].rotation += self.slots[num].velocity.x * 0.1;
        // `:566-567` — cast, then integer division.
        let num6 = (self.slots[num].position.x + 6.0) as i32 / 16;
        let num7 = (self.slots[num].position.y - 6.0) as i32 / 16;
        let wet = world.tile(num6, num7).is_some_and(|t| t.liquid > 0);
        if !wet {
            self.slots[num].velocity.y += 0.2;
            if self.slots[num].velocity.y < 0.0 {
                self.slots[num].velocity *= 0.92;
            }
        } else {
            self.slots[num].velocity.y += 0.005;
            let len = self.slots[num].velocity.length();
            if len > 1.0 {
                self.slots[num].velocity *= 0.1;
            } else if len > 0.1 {
                self.slots[num].velocity *= 0.98;
            }
        }
    }

    /// `Gore.ChumFloatingChunk_GetWaterLine` (`Gore.cs:262-290`). The C#
    /// materialises the three probe tiles before reading them (`:265-276`);
    /// the port's world already answers every cell, so a null tile is the
    /// empty tile the C# writes — liquid 0.
    fn chum_water_line(&self, world: &World, x: i32, y: i32, base: f32) -> f32 {
        let l1 = world.tile(x, y - 1).map_or(0, |t| t.liquid as i32);
        let l2 = world.tile(x, y).map_or(0, |t| t.liquid as i32);
        let l3 = world.tile(x, y + 1).map_or(0, |t| t.liquid as i32);
        if l1 > 0 {
            (y * 16) as f32 - (l1 / 16) as f32
        } else if l2 > 0 {
            ((y + 1) * 16) as f32 - (l2 / 16) as f32
        } else if l3 > 0 {
            ((y + 2) * 16) as f32 - (l3 / 16) as f32
        } else {
            base
        }
    }

    /// `Gore.Update`'s 1208 floating-chunk arm (`Gore.cs:590-671`), with the
    /// 1201 splash spawn at `:645-656` (three stream steps when the roll hits).
    fn update_chum_floating_chunk<R: GoreRng>(&mut self, rng: &mut R, world: &World, num: usize) {
        if self.slots[num].frame_counter == 0 {
            self.slots[num].frame_counter = 1;
            self.slots[num].frame.current_row = rng.next_max(4) as u8;
        }
        let vector = self.slots[num].position
            + Vec2::new(self.slots[num].width(), self.slots[num].height()) / 2.0;
        let num9 = vector.x as i32 / 16;
        let num10 = vector.y as i32 / 16;
        let flag2 = world.tile(num9, num10).is_some_and(|t| t.liquid > 0);
        self.slots[num].scale -= 0.0005;
        if (self.slots[num].scale as f64) < 0.1 {
            self.slots[num].scale = 0.1;
            self.slots[num].alpha = 255;
        }
        self.slots[num].rotation += self.slots[num].velocity.x * 0.1;
        if flag2 {
            self.slots[num].velocity.x *= 0.9;
            let num11 = vector.x as i32 / 16;
            let num12 = (vector.y / 16.0) as i32;
            let num13 = ((self.slots[num].position.y + self.slots[num].height()) / 16.0) as i32;
            let _ = num13;
            if self.slots[num].velocity.y > 0.0 {
                self.slots[num].velocity.y *= 0.5;
            }
            let base = self.slots[num].position.y + self.slots[num].height();
            let num14 = self.chum_water_line(world, num11, num12, base);
            if vector.y > num14 {
                self.slots[num].velocity.y -= 0.1;
                if self.slots[num].velocity.y < -8.0 {
                    self.slots[num].velocity.y = -8.0;
                }
                if vector.y + self.slots[num].velocity.y < num14 {
                    self.slots[num].velocity.y = num14 - vector.y;
                }
            } else {
                self.slots[num].velocity.y = num14 - vector.y;
            }
            // `:644`: `flag3 = !flag2 && ...` — always false inside this arm.
            if rng.next_max(270) == 0 {
                let position = self.slots[num].position + Vec2::new(0.0, 6.0);
                let scale = self.slots[num].scale * 0.7;
                let idx = self.new_gore(rng, world, position, Vec2::ZERO, 1201, scale);
                let dx = rng.next_float_direction() * 0.5;
                let dy = rng.next_float();
                self.slots[idx].velocity = Vec2::new(dx, dy);
            }
        } else {
            if self.slots[num].velocity.y == 0.0 {
                self.slots[num].velocity.x *= 0.95;
            }
            self.slots[num].velocity.x *= 0.98;
            self.slots[num].velocity.y += 0.3;
            if self.slots[num].velocity.y > 15.9 {
                self.slots[num].velocity.y = 15.9;
            }
        }
    }

    /// `Gore.Update`'s `SpecialAI == 2` smoke row (`Gore.cs:767-843`).
    /// Documented drops: the texture-gated player-rect probe (`:794-806`, so
    /// `num19` stays `-1`) and the dust burst it feeds (`:825-841`).
    fn update_special_ai_two<R: GoreRng>(
        &mut self,
        rng: &mut R,
        wind_for_visuals: f32,
        num: usize,
    ) {
        if self.slots[num].time_left < 60 {
            self.slots[num].alpha += rng.next_range(1, 7);
        } else if self.slots[num].alpha > 100 {
            self.slots[num].alpha -= rng.next_range(1, 4);
        }
        if self.slots[num].alpha < 0 {
            self.slots[num].alpha = 0;
        }
        if self.slots[num].alpha > 255 {
            self.slots[num].time_left = 0;
        }
        // `:785` — the sideways drift, on the freshly damped X.
        let vx = self.slots[num].velocity.x;
        let ax = rng.next_range(-10, 11) as f32;
        self.slots[num].velocity.x = (vx * 50.0 + wind_for_visuals * 2.0 + ax * 0.1) / 51.0;
        let mut num18 = 0.0;
        if self.slots[num].velocity.x < 0.0 {
            num18 = self.slots[num].velocity.x * 0.2;
        }
        let vy = self.slots[num].velocity.y;
        let ay = rng.next_range(-10, 11) as f32 * 0.2;
        self.slots[num].velocity.y = (vy * 50.0 + -0.35 + num18 + ay) / 51.0;
        self.slots[num].rotation = self.slots[num].velocity.x * 0.6;
        if self.slots[num].time_left > 0 {
            if rng.next_max(2) == 0 {
                self.slots[num].time_left -= 1;
            }
            if rng.next_max(50) == 0 {
                self.slots[num].time_left -= 5;
            }
            if rng.next_max(100) == 0 {
                self.slots[num].time_left -= 10;
            }
        } else {
            self.slots[num].alpha = 255;
        }
    }

    /// `Gore.Update`'s 411-430 paper-page row (`Gore.cs:844-912`). Documented
    /// drops: the texture-gated player/solid probe (`:850-864`) and the dust
    /// burst (`:883-910`); the three `timeLeft` rolls stay.
    fn update_pages<R: GoreRng>(&mut self, rng: &mut R, wind_for_visuals: f32, num: usize) {
        self.slots[num].alpha = 50;
        let vx = self.slots[num].velocity.x;
        let ax = rng.next_range(-10, 11) as f32;
        self.slots[num].velocity.x = (vx * 50.0 + wind_for_visuals * 2.0 + ax * 0.1) / 51.0;
        let vy = self.slots[num].velocity.y;
        let ay = rng.next_range(-10, 11) as f32 * 0.2;
        self.slots[num].velocity.y = (vy * 50.0 + -0.25 + ay) / 51.0;
        self.slots[num].rotation = self.slots[num].velocity.x * 0.3;
        if self.slots[num].time_left > 0 {
            if rng.next_max(2) == 0 {
                self.slots[num].time_left -= 1;
            }
            if rng.next_max(50) == 0 {
                self.slots[num].time_left -= 5;
            }
            if rng.next_max(100) == 0 {
                self.slots[num].time_left -= 10;
            }
        } else {
            self.slots[num].alpha = 255;
        }
    }

    /// `Gore.Gore_UpdateLeaf` (`Gore.cs:1085-1209`) — the falling leaf: the
    /// rising half, then the flutter with the slope pass.
    fn update_leaf<R: GoreRng>(
        &mut self,
        world: &World,
        collision: &dyn GoreCollision,
        wind_for_visuals: f32,
        num: usize,
        rng: &mut R,
        dust: Option<(&mut crate::particles::DustPool, &crate::particles::SpawnCtx)>,
    ) {
        // `:1087-1089` — the leaf's own origin, 4 px right and 4 px down of
        // its pivot's top edge.
        let position = self.slots[num].position;
        let vector = position + Vec2::new(12.0, 12.0) / 2.0 - Vec2::new(4.0, 4.0) / 2.0
            - Vec2::new(0.0, 4.0);
        let vector2 = position - vector;
        if self.slots[num].velocity.y < 0.0 {
            let mut vector3 = Vec2::new(self.slots[num].velocity.x, -0.2);
            let num_v: i32 = 3;
            let point = to_tile_coordinates(
                Vec2::new(num_v as f32, num_v as f32) / 2.0 + vector,
            );
            if !world.in_bounds(point.0, point.1) {
                self.slots[num].active = false;
                return;
            }
            let Some(tile) = world.tile(point.0, point.1) else {
                self.slots[num].active = false;
                return;
            };
            let num2 = 6;
            let liquid = tile.liquid as i32;
            let rectangle = (point.0 * 16, point.1 * 16 + liquid / 16, 16, 16 - liquid / 16);
            let value = (vector.x as i32, vector.y as i32 + num2, num_v, num_v);
            let flag = tile.liquid > 0 && rect_intersects(rectangle, value);
            if flag {
                match tile.liquid_kind() {
                    crate::tile::LiquidKind::Honey => vector3.x = 0.0,
                    // `:1116-1123` — the lava arm deactivates, then the five
                    // dust-31 puffs at the leaf's position (`NewDust(position,
                    // 3, 3, 31, 0f, -0.2f)`), rolled on the stage stream
                    // (W4-g68 dust seam). The leaf keeps falling through the
                    // rest of the tick — the C# does not return here.
                    crate::tile::LiquidKind::Lava => {
                        self.slots[num].active = false;
                        if let Some((pool, ctx)) = dust {
                            let position = self.slots[num].position;
                            for _ in 0..5 {
                                pool.new_dust(
                                    &mut DustFromGore(&mut *rng),
                                    ctx,
                                    world,
                                    position,
                                    num_v,
                                    num_v,
                                    31,
                                    0.0,
                                    -0.2,
                                    0,
                                    crate::particles::Color::default(),
                                    1.0,
                                );
                            }
                        }
                    }
                    _ => vector3.x = wind_for_visuals,
                }
                if self.slots[num].position.y as f64 > world.world_surface * 16.0 {
                    vector3.x = 0.0;
                }
            }
            if !solid_tile(world.tile(point.0, point.1 + 1)) && !flag {
                self.slots[num].velocity.y = 0.1;
                self.slots[num].time_left = 0;
                self.slots[num].alpha += 20;
            }
            vector3 = collision.tile_collision(
                world,
                vector,
                vector3,
                num_v as f32,
                num_v as f32,
            );
            if flag {
                self.slots[num].rotation = vector3.y.atan2(vector3.x) + std::f32::consts::FRAC_PI_2;
            }
            vector3.x *= 0.94;
            if !flag || (vector3.x > -0.01 && vector3.x < 0.01) {
                vector3.x = 0.0;
            }
            if self.slots[num].time_left > 0 {
                self.slots[num].time_left -=
                    crate::gore_id::disappear_speed_checked(self.slots[num].gore_type);
            } else {
                self.slots[num].alpha +=
                    crate::gore_id::disappear_speed_alpha_checked(self.slots[num].gore_type);
            }
            self.slots[num].velocity.x = vector3.x;
            self.slots[num].position.x += self.slots[num].velocity.x;
            return;
        }
        // `:1161-1208` — the falling half.
        self.slots[num].velocity.y += std::f32::consts::PI / 180.0;
        let r = rotated_unit_y(self.slots[num].velocity.y);
        let mut vector4 = Vec2::new(r.x, r.y.abs());
        let num3: i32 = 4;
        if (self.slots[num].position.y as f64) < world.world_surface * 16.0 {
            vector4.x += wind_for_visuals * 4.0;
        }
        let vector5 = vector4;
        vector4 = collision.tile_collision(world, vector, vector4, num3 as f32, num3 as f32);
        let (pos, vel) = collision.slope_collision(
            world,
            vector,
            vector4,
            num3 as f32,
            num3 as f32,
            1.0,
            false,
        );
        self.slots[num].position = pos;
        vector4 = vel;
        self.slots[num].position += vector2;
        if vector4 != vector5 {
            self.slots[num].velocity.y = -1.0;
        }
        // `:1180` — `(Width, Height) * 0.5f + position`: the source-rect
        // half-size when the sheet is packed (the W4-g68 asset seam), the
        // C#'s `1f` fallback otherwise.
        let point2 = to_tile_coordinates(
            Vec2::new(self.slots[num].width(), self.slots[num].height()) * 0.5
                + self.slots[num].position,
        );
        if !world.in_bounds(point2.0, point2.1) {
            self.slots[num].active = false;
            return;
        }
        let Some(tile2) = world.tile(point2.0, point2.1) else {
            self.slots[num].active = false;
            return;
        };
        let num4 = 6;
        let liquid2 = tile2.liquid as i32;
        let rectangle2 = (
            point2.0 * 16,
            point2.1 * 16 + liquid2 / 16,
            16,
            16 - liquid2 / 16,
        );
        let value2 = (vector.x as i32, vector.y as i32 + num4, num3, num3);
        if tile2.liquid > 0 && rect_intersects(rectangle2, value2) {
            self.slots[num].velocity.y = -1.0;
        }
        self.slots[num].position += vector4;
        self.slots[num].rotation = vector4.y.atan2(vector4.x) + std::f32::consts::FRAC_PI_2;
        if self.slots[num].time_left > 0 {
            self.slots[num].time_left -=
                crate::gore_id::disappear_speed_checked(self.slots[num].gore_type);
        } else {
            self.slots[num].alpha +=
                crate::gore_id::disappear_speed_alpha_checked(self.slots[num].gore_type);
        }
    }

    /// `Gore.Gore_UpdateSail` (`Gore.cs:1211-1271`) — the `SpecialAI == 1`
    /// sail/flag flutter.
    fn update_sail(&mut self, world: &World, collision: &dyn GoreCollision, num: usize) {
        if self.slots[num].velocity.y < 0.0 {
            let mut vector = Vec2::new(self.slots[num].velocity.x, 0.6);
            let num_v: i32 = 32;
            let num_v = (num_v as f32 * 0.9) as i32;
            let size = (num_v as f32 * self.slots[num].scale) as i32;
            vector = collision.tile_collision(
                world,
                self.slots[num].position,
                vector,
                size as f32,
                size as f32,
            );
            vector.x *= 0.97;
            if vector.x > -0.01 && vector.x < 0.01 {
                vector.x = 0.0;
            }
            if self.slots[num].time_left > 0 {
                self.slots[num].time_left -= 1;
            } else {
                self.slots[num].alpha += 1;
            }
            self.slots[num].velocity.x = vector.x;
            return;
        }
        self.slots[num].velocity.y += std::f32::consts::PI / 60.0;
        let r = rotated_unit_y(self.slots[num].velocity.y);
        let mut vector2 = Vec2::new(r.x * 2.0, r.y.abs() * 3.0);
        vector2 *= 2.0;
        let num2: i32 = 32;
        let size = (num2 as f32 * self.slots[num].scale) as i32;
        let vector3 = vector2;
        vector2 = collision.tile_collision(
            world,
            self.slots[num].position,
            vector2,
            size as f32,
            size as f32,
        );
        if vector2 != vector3 {
            self.slots[num].velocity.y = -1.0;
        }
        let vector2c = vector2;
        self.slots[num].position += vector2c;
        self.slots[num].rotation = vector2c.y.atan2(vector2c.x) + std::f32::consts::PI;
        if self.slots[num].time_left > 0 {
            self.slots[num].time_left -= 1;
        } else {
            self.slots[num].alpha += 1;
        }
    }
}

/// `MathHelper.Lerp` (`MathHelper.cs`): `value1 + (value2 - value1) * amount`.
struct MathHelper;

impl MathHelper {
    fn lerp(value1: f32, value2: f32, amount: f32) -> f32 {
        value1 + (value2 - value1) * amount
    }
}

/// `Vector2.UnitY.RotatedBy(angle)` — `(0, 1)` rotated by `angle` in XNA's
/// clockwise-positive frame, i.e. `(-sin θ, cos θ)`.
fn rotated_unit_y(angle: f32) -> Vec2 {
    Vec2::new(-angle.sin(), angle.cos())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::World;

    struct VecRng { draws: Vec<i32>, idx: usize }
    impl VecRng { fn new(draws: Vec<i32>) -> Self { Self { draws, idx: 0 } } }
    impl GoreRng for VecRng {
        fn next_max(&mut self, max: i32) -> i32 { let v = self.draws[self.idx]; self.idx += 1; assert!(v < max); v }
        fn next_range(&mut self, min: i32, max: i32) -> i32 { let v = self.draws[self.idx]; self.idx += 1; assert!(v >= min && v < max); v }
        fn next_float(&mut self) -> f32 { self.next_range(0, 100) as f32 / 100.0 }
        fn next_float_direction(&mut self) -> f32 {
            self.next_range(-100, 100) as f32 / 100.0
        }
    }

    /// A 64x64 world — the tile probe of `DeactivateIfOutsideOfWorld`
    /// (`Gore.cs:292-306`) must find a tile under the spawn.
    fn world() -> World {
        World::new("test".into(), 64, 64)
    }

    #[test]
    fn new_gore_consumes_rng_in_order_and_sets_velocity() {
        let mut pool = GorePool::new();
        let mut rng = VecRng::new(vec![10, 20]);
        let world = world();
        let idx = pool.new_gore(&mut rng, &world, Vec2::new(32.0, 32.0), Vec2::new(0.0, 0.0), 1, 1.0);
        assert_eq!(idx, 0);
        let gore = pool.slot(0);
        assert_eq!(gore.gore_type, 1);
        assert_eq!(gore.position, Vec2::new(32.0, 32.0));
        assert_eq!(gore.velocity.x, 2.0);
        assert_eq!(gore.velocity.y, -1.0);
    }

    /// `Gore.cs:1355-1360` — the `IsDrip` row: 15 frames, behind the tiles
    /// and three times the life.
    #[test]
    fn the_drip_types_get_the_fifteen_frame_sheet() {
        let mut pool = GorePool::new();
        let mut rng = VecRng::new(vec![10, 20]);
        let world = world();
        let idx = pool.new_gore(&mut rng, &world, Vec2::new(32.0, 32.0), Vec2::new(0.0, 0.0), 706, 1.0);
        assert_eq!(idx, 0);
        let gore = pool.slot(0);
        assert_eq!(gore.frame.row_count, 15);
        assert_eq!(gore.frame.column_count, 1);
        assert!(gore.behind_tiles);
        assert_eq!(gore.time_left, GORE_TIME_DEFAULT * 3);
    }

    /// `Gore.cs:1391-1402` — `SpecialAI == 3`: the two `NextFloat` velocity
    /// draws come first, then the sheet row, then the frame counter.
    #[test]
    fn the_special_ai_three_arm_rolls_velocity_then_row_then_counter() {
        let mut pool = GorePool::new();
        let mut rng = VecRng::new(vec![10, 20, 50, 60, 3, 4]);
        let world = world();
        let idx = pool.new_gore(&mut rng, &world, Vec2::new(32.0, 32.0), Vec2::new(0.0, 0.0), 910, 1.0);
        assert_eq!(idx, 0);
        let gore = pool.slot(0);
        // (NextFloat() - 0.5f) * 1f and NextFloat() * (PI * 2f).
        assert_eq!(gore.velocity.x, 0.0);
        assert_eq!(gore.velocity.y, 0.6 * (std::f32::consts::PI * 2.0));
        // The leaf rows get the 32-column sheet.
        assert_eq!(gore.frame.column_count, 32);
        assert_eq!(gore.frame.row_count, 8);
        assert_eq!(gore.frame.current_row, 3);
        assert_eq!(gore.frame_counter, 4);
    }

    /// `Gore.cs:1403-1406` — `SpecialAI == 1`: the wider velocity kick.
    #[test]
    fn the_special_ai_one_arm_rolls_two_floats() {
        let mut pool = GorePool::new();
        let mut rng = VecRng::new(vec![10, 20, 25, 75]);
        let world = world();
        let idx = pool.new_gore(&mut rng, &world, Vec2::new(32.0, 32.0), Vec2::new(0.0, 0.0), 860, 1.0);
        assert_eq!(idx, 0);
        let gore = pool.slot(0);
        assert_eq!(gore.velocity.x, -0.75);
        assert_eq!(gore.velocity.y, 0.75 * (std::f32::consts::PI * 2.0));
    }

    /// `Gore.cs:1422-1429` — the 907-909 balloons: the sheet row, then the
    /// frame counter.
    #[test]
    fn the_balloon_types_roll_the_row_and_the_counter() {
        let mut pool = GorePool::new();
        let mut rng = VecRng::new(vec![10, 20, 2, 4]);
        let world = world();
        let idx = pool.new_gore(&mut rng, &world, Vec2::new(32.0, 32.0), Vec2::new(0.0, 0.0), 907, 1.0);
        assert_eq!(idx, 0);
        let gore = pool.slot(0);
        assert!(gore.sticky);
        assert_eq!(gore.frame.row_count, 3);
        assert_eq!(gore.frame.current_row, 2);
        assert_eq!(gore.frame_counter, 4);
        assert_eq!(gore.rotation, 0.0);
    }

    /// `Gore.cs:1442-1456` — the `SpecialAI == 4/5/6` alpha/life rows.
    #[test]
    fn the_special_ai_four_five_six_rows_set_alpha_and_life() {
        for (gore_type, life) in [(1087, 300), (1091, 240), (1202, 480)] {
            let mut pool = GorePool::new();
            let mut rng = VecRng::new(vec![10, 20]);
            let world = world();
            let idx = pool.new_gore(
                &mut rng,
                &world,
                Vec2::new(32.0, 32.0),
                Vec2::new(0.0, 0.0),
                gore_type,
                1.0,
            );
            assert_eq!(idx, 0);
            let gore = pool.slot(0);
            assert_eq!(gore.alpha, 254);
            assert_eq!(gore.time_left, life);
        }
    }

    /// `Gore.cs:1430-1441` — `SpecialAI == 2`: only the sticky flag is
    /// unconditional; the rest waits for the texture seam.
    #[test]
    fn the_special_ai_two_arm_only_clears_sticky() {
        let mut pool = GorePool::new();
        let mut rng = VecRng::new(vec![10, 20]);
        let world = world();
        let idx = pool.new_gore(&mut rng, &world, Vec2::new(32.0, 32.0), Vec2::new(0.0, 0.0), 825, 1.0);
        assert_eq!(idx, 0);
        let gore = pool.slot(0);
        assert!(!gore.sticky);
        assert_eq!(gore.time_left, GORE_TIME_DEFAULT);
    }

    /// A collision seam that reports a floor under the gore: `TileCollision`
    /// keeps X and zeroes Y, exactly as the C# does when the AABB rests on a
    /// solid tile. The gore then holds still, so the timer/fade arms can be
    /// observed without the body drifting out of the 64x64 test world.
    struct NoCollision;

    impl GoreCollision for NoCollision {
        fn tile_collision(
            &self,
            _world: &World,
            _position: Vec2,
            velocity: Vec2,
            _width: f32,
            _height: f32,
        ) -> Vec2 {
            Vec2::new(velocity.x, 0.0)
        }
        fn wet_collision(&self, _world: &World, _position: Vec2, _w: f32, _h: f32) -> bool {
            false
        }
        fn slope_collision(
            &self,
            _world: &World,
            position: Vec2,
            velocity: Vec2,
            _w: f32,
            _h: f32,
            _gravity: f32,
            _fall: bool,
        ) -> (Vec2, Vec2) {
            (position, velocity)
        }
    }

    /// The ROADMAP's S1-E-4 lifecycle: spawn → `timeLeft` countdown →
    /// alpha fade → deactivation, at the sticky row's one point per tick
    /// (`Gore.cs:1011-1018`). The spawn sits mid-world because
    /// `Gore.cs:314-325` probes the world every tick and a gore that drifts
    /// out is deactivated before the fade can finish.
    #[test]
    fn the_sticky_gore_counts_down_then_fades_out() {
        let mut pool = GorePool::new();
        let mut rng = VecRng::new(vec![10, 20]);
        let world = world();
        let idx = pool.new_gore(&mut rng, &world, Vec2::new(500.0, 900.0), Vec2::ZERO, 1, 1.0);
        assert_eq!(idx, 0);
        assert!(pool.slot(0).active);
        pool.slot_mut(0).time_left = 2;
        let collisions = NoCollision;
        // Two ticks spend the life, then one alpha point per tick
        // (`Gore.cs:1017`): 2 + 255 ticks to the `alpha >= 255` kill
        // (`:1055-1058`).
        for tick in 0..257 {
            pool.update(&mut rng, &world, &collisions, 0.0, None);
            if tick < 2 {
                assert_eq!(pool.slot(0).time_left, 1 - tick as i32);
                assert_eq!(pool.slot(0).alpha, 0, "the life runs out before the fade");
            } else {
                assert_eq!(pool.slot(0).alpha, tick as i32 - 1);
            }
        }
        assert!(!pool.slot(0).active, "alpha 255 deactivates the gore");
        // The inactive slot is skipped from now on (`Gore.cs:310-313`).
        pool.update(&mut rng, &world, &collisions, 0.0, None);
        assert_eq!(pool.slot(0).alpha, 255);
    }

    /// The free-floating fade (`Gore.cs:1020-1023`) is twice the
    /// `DisappearSpeedAlpha` row: a default type dies after 128 ticks.
    #[test]
    fn the_free_gore_fades_at_double_speed() {
        let mut pool = GorePool::new();
        let mut rng = VecRng::new(vec![10, 20]);
        let world = world();
        let idx = pool.new_gore(&mut rng, &world, Vec2::new(500.0, 900.0), Vec2::ZERO, 1, 1.0);
        assert_eq!(idx, 0);
        // `Gore.cs:1341-1354` makes every non-listed type sticky; clear it so
        // the tick takes the free-floating arm.
        pool.slot_mut(0).sticky = false;
        let collisions = NoCollision;
        for _ in 0..127 {
            pool.update(&mut rng, &world, &collisions, 0.0, None);
        }
        assert!(pool.slot(0).active);
        assert_eq!(pool.slot(0).alpha, 254);
        pool.update(&mut rng, &world, &collisions, 0.0, None);
        assert!(!pool.slot(0).active);
    }

    /// `Gore.UpdateAmbientFloorCloud` (`Gore.cs:110-154`): the drift is
    /// `0.4 * Main.WindForVisuals` on X (`:136`), one `DisappearSpeed` point
    /// of life per tick (`:112`), and a spent `timeLeft` deactivates the cloud
    /// on the spot (`:113-117`).
    #[test]
    fn the_floor_cloud_drifts_with_the_wind_and_expires() {
        let mut pool = GorePool::new();
        let mut rng = VecRng::new(vec![10, 20]);
        let world = world();
        // 1087 is `SpecialAI == 4` (`GoreID.cs:20`).
        let idx = pool.new_gore(&mut rng, &world, Vec2::new(500.0, 900.0), Vec2::ZERO, 1087, 1.0);
        assert_eq!(idx, 0);
        assert_eq!(pool.slot(0).time_left, 300);
        // The spawn sets alpha 254 (`Gore.cs:1442-1446`); the open sky under
        // the probe makes the row fade out at once, so clear it to observe the
        // drift and the countdown.
        pool.slot_mut(0).alpha = 0;
        pool.slot_mut(0).time_left = 2;
        let collisions = NoCollision;
        pool.update(&mut rng, &world, &collisions, 2.5, None);
        let gore = pool.slot(0);
        assert_eq!(gore.velocity.x, 0.4 * 2.5);
        assert_eq!(gore.time_left, 1);
        assert!(gore.active);
        pool.update(&mut rng, &world, &collisions, 2.5, None);
        assert_eq!(pool.slot(0).time_left, 0);
        assert!(!pool.slot(0).active, "an empty timeLeft deactivates the cloud");
    }

    // ---- W4-g68: Gore_UpdateLeaf, the leaf physics the foliage shed now
    // rides (WRITTEN NOT RUN — code-only mode). ----

    /// A pass-through collision seam: both probes return their inputs, so
    /// the leaf's own velocity math is observable without tile interference.
    struct FreeCollision;
    impl GoreCollision for FreeCollision {
        fn tile_collision(
            &self,
            _world: &World,
            _position: Vec2,
            velocity: Vec2,
            _width: f32,
            _height: f32,
        ) -> Vec2 {
            velocity
        }
        fn wet_collision(&self, _world: &World, _position: Vec2, _w: f32, _h: f32) -> bool {
            false
        }
        fn slope_collision(
            &self,
            _world: &World,
            position: Vec2,
            velocity: Vec2,
            _w: f32,
            _h: f32,
            _gravity: f32,
            _fall: bool,
        ) -> (Vec2, Vec2) {
            (position, velocity)
        }
    }

    /// A seam whose `TileCollision` brakes X to 0 (a wall) — the
    /// `vector4 != vector5` rest switch (`Gore.cs:1176-1179`).
    struct WallCollision;
    impl GoreCollision for WallCollision {
        fn tile_collision(
            &self,
            _world: &World,
            _position: Vec2,
            velocity: Vec2,
            _width: f32,
            _height: f32,
        ) -> Vec2 {
            Vec2::new(0.0, velocity.y)
        }
        fn wet_collision(&self, _world: &World, _position: Vec2, _w: f32, _h: f32) -> bool {
            false
        }
        fn slope_collision(
            &self,
            _world: &World,
            position: Vec2,
            velocity: Vec2,
            _w: f32,
            _h: f32,
            _gravity: f32,
            _fall: bool,
        ) -> (Vec2, Vec2) {
            (position, velocity)
        }
    }

    /// A 64x64 world whose surface sits BELOW every test position, so the
    /// `position.Y < worldSurface * 16` wind term applies.
    fn leaf_physics_world() -> World {
        let mut w = World::new("leafphys".into(), 64, 64);
        w.world_surface = 1000.0;
        w
    }

    fn spawn_leaf(pool: &mut GorePool, world: &World, pos: Vec2) {
        let mut rng = VecRng::new(vec![10, 20, 50, 60, 3, 4]);
        let idx = pool.new_gore(&mut rng, world, pos, Vec2::ZERO, 910, 1.0);
        assert_eq!(idx, 0);
        // Pin the sway angle to 0 so the rotated unit vector is (0, 1).
        pool.slot_mut(0).velocity = Vec2::new(0.0, 0.0);
    }

    /// The falling half (`Gore.cs:1161-1167`): the angle grows π/180 per
    /// tick, the velocity is `(-sin θ, |cos θ|)`, and above the surface the
    /// wind adds `WindForVisuals * 4` to X. The position moves by the
    /// resolved velocity (`:1199`), the rotation is `atan2 + π/2` (`:1200`).
    #[test]
    fn w4_g68_the_leaf_fall_sways_and_rides_the_wind() {
        let world = leaf_physics_world();
        let mut pool = GorePool::new();
        spawn_leaf(&mut pool, &world, Vec2::new(500.0, 300.0));
        let wind = 0.5_f32;
        pool.update(&mut VecRng::new(vec![]), &world, &FreeCollision, wind, None);
        let g = pool.slot(0);
        let theta = std::f32::consts::PI / 180.0;
        assert_eq!(g.velocity.y, theta, "the angle grows one degree per tick");
        let want_x = -theta.sin() + wind * 4.0;
        let want_y = theta.cos().abs();
        assert!((g.position.x - (500.0 + want_x)).abs() < 1e-4);
        assert!((g.position.y - (300.0 + want_y)).abs() < 1e-4);
        assert!(
            (g.rotation - (want_y.atan2(want_x) + std::f32::consts::FRAC_PI_2)).abs() < 1e-4
        );
        assert_eq!(g.time_left, GORE_TIME_DEFAULT - 1, "one life point per tick");
    }

    /// Below the surface the wind term is dropped (`Gore.cs:1164-1167`).
    #[test]
    fn w4_g68_underground_leaves_get_no_wind_push() {
        let mut world = leaf_physics_world();
        world.world_surface = 0.0; // every position is underground
        let mut pool = GorePool::new();
        spawn_leaf(&mut pool, &world, Vec2::new(500.0, 300.0));
        pool.update(&mut VecRng::new(vec![]), &world, &FreeCollision, 0.5, None);
        let g = pool.slot(0);
        let theta = std::f32::consts::PI / 180.0;
        assert!((g.position.x - (500.0 - theta.sin())).abs() < 1e-4);
    }

    /// A collision that changes the velocity flips the leaf to the rest
    /// phase: `velocity.Y = -1` (`Gore.cs:1176-1179`).
    #[test]
    fn w4_g68_a_blocked_fall_switches_to_rest() {
        let world = leaf_physics_world();
        let mut pool = GorePool::new();
        spawn_leaf(&mut pool, &world, Vec2::new(500.0, 300.0));
        pool.update(&mut VecRng::new(vec![]), &world, &WallCollision, 0.5, None);
        assert_eq!(pool.slot(0).velocity.y, -1.0, "the rest-phase sentinel");
    }

    /// The rest phase in WATER (`Gore.cs:1124-1131`): X becomes the wind,
    /// the collision passes it through, then ×0.94; the leaf drifts by the
    /// result and the rotation follows the velocity. Underground the wind
    /// is zeroed first (`:1128-1131`).
    #[test]
    fn w4_g68_the_rest_phase_drifts_on_water_with_the_wind() {
        let mut world = leaf_physics_world();
        // The rest probe: vector = pos + (4, 0); point = (vector + 1.5) >> 4.
        // pos (160, 160) → vector (164, 160) → point (10, 10). The liquid
        // rect is `(x*16, y*16 + liquid/16, 16, 16 - liquid/16)` — liquid
        // 100 gives top 166, height 10, intersecting the 3x3 probe at
        // (164, 166) (`Gore.cs:1107-1109`).
        let i = world.index(10, 10);
        world.tiles[i].liquid = 100;
        let mut pool = GorePool::new();
        spawn_leaf(&mut pool, &world, Vec2::new(160.0, 160.0));
        pool.slot_mut(0).velocity = Vec2::new(0.0, -1.0);
        let wind = 0.5_f32;
        pool.update(&mut VecRng::new(vec![]), &world, &FreeCollision, wind, None);
        let g = pool.slot(0);
        assert!((g.velocity.x - wind * 0.94).abs() < 1e-5, "the 0.94 decay");
        assert!((g.position.x - (160.0 + wind * 0.94)).abs() < 1e-4);
        assert_eq!(g.position.y, 160.0, "the rest phase only moves X");
        // Underground: the wind zeroes, then the |x| < 0.01 cut zeroes X.
        world.world_surface = 0.0;
        pool.slot_mut(0).velocity = Vec2::new(0.0, -1.0);
        pool.slot_mut(0).position = Vec2::new(160.0, 160.0);
        pool.update(&mut VecRng::new(vec![]), &world, &FreeCollision, wind, None);
        assert_eq!(pool.slot(0).velocity.x, 0.0);
    }

    /// HONEY freezes the rest drift (`Gore.cs:1112-1115`): X = 0 regardless
    /// of the wind.
    #[test]
    fn w4_g68_honey_freezes_the_rest_drift() {
        let mut world = leaf_physics_world();
        let i = world.index(10, 10);
        world.tiles[i].liquid = 100;
        world.tiles[i].flags |= crate::tile::FLAG_LIQUID_HONEY;
        let mut pool = GorePool::new();
        spawn_leaf(&mut pool, &world, Vec2::new(160.0, 160.0));
        pool.slot_mut(0).velocity = Vec2::new(0.0, -1.0);
        pool.update(&mut VecRng::new(vec![]), &world, &FreeCollision, 0.5, None);
        assert_eq!(pool.slot(0).velocity.x, 0.0);
        assert_eq!(pool.slot(0).position.x, 160.0);
    }

    /// LAVA burns the leaf up (`Gore.cs:1116-1123`): the slot deactivates.
    /// Without the dust seam the five-dust puff stays dropped (the recorded
    /// S1-E-4 behaviour); the seam-on witness below pins the puffs.
    #[test]
    fn w4_g68_lava_burns_the_leaf() {
        let mut world = leaf_physics_world();
        let i = world.index(10, 10);
        world.tiles[i].liquid = 100;
        world.tiles[i].flags |= crate::tile::FLAG_LIQUID_LAVA;
        let mut pool = GorePool::new();
        spawn_leaf(&mut pool, &world, Vec2::new(160.0, 160.0));
        pool.slot_mut(0).velocity = Vec2::new(0.0, -1.0);
        pool.update(&mut VecRng::new(vec![]), &world, &FreeCollision, 0.5, None);
        assert!(!pool.slot(0).active, "the lava arm deactivates at once");
    }

    /// The W4-g68 dust seam: the lava arm's five `Dust.NewDust(position, 3,
    /// 3, 31, 0f, -0.2f)` puffs (`Gore.cs:1119-1122`) roll the SAME stream
    /// the gore walk does (`SwapRandom("UpdateGores")`, `Main.cs:17771`).
    /// Each `NewDust` consumes six draws in `Dust.cs:170-190` order:
    /// `Next(1)` X-offset, `Next(1)` Y-offset (the 3×3 box clamps to 5×5,
    /// so `num - 4 = 1`), `Next(-20, 21)` vel-X, `Next(-20, 21)` vel-Y,
    /// `Next(3)` frame row, `Next(-20, 21)` scale; the type-31 tweak chain
    /// draws NOTHING (31 is in no arm, `Dust.cs:214-252`).
    #[test]
    fn w4_g68_the_lava_arm_puffs_five_dust_on_the_gore_stream() {
        let mut world = leaf_physics_world();
        let i = world.index(10, 10);
        world.tiles[i].liquid = 100;
        world.tiles[i].flags |= crate::tile::FLAG_LIQUID_LAVA;
        let mut pool = GorePool::new();
        spawn_leaf(&mut pool, &world, Vec2::new(160.0, 160.0));
        pool.slot_mut(0).velocity = Vec2::new(0.0, -1.0);
        let mut dust = crate::particles::DustPool::new();
        let ctx = crate::particles::SpawnCtx {
            game_menu: false,
            game_paused: false,
            generating: false,
            screen_position: Vec2::new(0.0, 0.0),
            screen_size: Vec2::new(640.0, 480.0),
            max_dust_to_draw: 6000,
        };
        // Six draws per puff × five puffs; the offsets pinned to 0, the
        // velocity rolls to +1.0/+0.5, the frame row to 1, the scale to
        // 1 + 10*0.01 = 1.1.
        let mut draws = Vec::new();
        for _ in 0..5 {
            draws.extend_from_slice(&[0, 0, 10, 5, 1, 10]);
        }
        let mut rng = VecRng::new(draws);
        pool.update(
            &mut rng,
            &world,
            &FreeCollision,
            0.5,
            Some((&mut dust, &ctx)),
        );
        assert_eq!(rng.idx, 30, "six stream draws per puff, five puffs");
        assert!(!pool.slot(0).active);
        let live: Vec<_> = (0..crate::particles::DUST_SENTINEL)
            .filter(|&i| dust.slot(i).active)
            .collect();
        assert_eq!(live.len(), 5, "five dust-31 puffs");
        for &i in &live {
            let d = dust.slot(i);
            assert_eq!(d.dust_type, 31);
            // The 5×5 clamp: offset Next(1)=0 + 4 → position + 4 on both axes.
            assert_eq!(d.position, Vec2::new(164.0, 164.0));
            assert_eq!(d.velocity, Vec2::new(1.0, 0.5 - 0.2), "the -0.2 Y kick");
            assert_eq!(d.frame_y, 10, "the Next(3)=1 row");
            assert!((d.scale - 1.1).abs() < 1e-5);
            assert_eq!(d.alpha, 0);
        }
    }

    /// The W4-g68 asset seam (`Gore.cs:44-66` + `SpriteFrame.cs:72-77`):
    /// a packed 32-column × 8-row leaf sheet gives the source-rect size
    /// `(w/32 − 2, h/8 − 2)`; `Width`/`Height` scale it, `AABBRectangle`
    /// truncates the scaled product and anchors at `(int)position`. An
    /// absent id keeps the `1f` / `(0,0,1,1)` unloaded fallbacks.
    #[test]
    fn w4_g68_width_height_read_the_packed_sheet_cell() {
        let world = leaf_physics_world();
        let mut pool = GorePool::new();
        // Gore_910's retail sheet is 448x112: cell 14x14 − 2 padding = 12x12.
        pool.set_texture_sizes(std::collections::HashMap::from([(910, (448, 112))]));
        spawn_leaf(&mut pool, &world, Vec2::new(160.0, 160.0));
        let g = pool.slot(0);
        assert_eq!(g.texture_size, Some((448, 112)));
        assert_eq!(g.width(), 12.0, "scale 1 × (448/32 − 2)");
        assert_eq!(g.height(), 12.0, "scale 1 × (112/8 − 2)");
        assert_eq!(g.aabb_rectangle(), (160, 160, 12, 12));
        pool.slot_mut(0).scale = 0.5;
        assert_eq!(pool.slot(0).width(), 6.0);
        assert_eq!(pool.slot(0).aabb_rectangle(), (160, 160, 6, 6));
        // A type with no sheet keeps the C#'s unloaded fallbacks.
        let n = pool.new_gore(
            &mut VecRng::new(vec![10, 20]),
            &world,
            Vec2::new(200.0, 200.0),
            Vec2::ZERO,
            1,
            1.0,
        );
        assert_eq!(pool.slot(n).texture_size, None);
        assert_eq!(pool.slot(n).width(), 1.0);
        assert_eq!(pool.slot(n).height(), 1.0);
        assert_eq!(pool.slot(n).aabb_rectangle(), (0, 0, 1, 1));
    }

    /// W4-g68 wiring-2 A/B: the 1225 position-adjust arm
    /// (`Gore.cs:1384-1388`). LOADED (id in `set_texture_sizes`): X shifts
    /// by the INTEGER-halved raw width × scale, Y by the raw height ×
    /// scale / 2 — the frame/timeLeft/sticky writes are unconditional and
    /// the `Next(6)` rolls on BOTH branches. UNLOADED: no shift, same
    /// draws (the A/B stream equality is the witness).
    #[test]
    fn w4_g68_the_1225_position_adjust_rides_the_loaded_gate() {
        let world = world();
        // Sheet 21x30 (odd width pins the integer halving: 21/2 = 10).
        let mut loaded = GorePool::new();
        loaded.set_texture_sizes(std::collections::HashMap::from([(1225, (21, 30))]));
        let mut rng = VecRng::new(vec![10, 20, 3]);
        let n = loaded.new_gore(
            &mut rng, &world, Vec2::new(100.0, 100.0), Vec2::ZERO, 1225, 2.0,
        );
        assert_eq!(rng.idx, 3, "kick x2 + Next(6)");
        let g = loaded.slot(n);
        assert_eq!(g.position, Vec2::new(100.0 - 10.0 * 2.0, 100.0 - 30.0 * 2.0 / 2.0));
        assert_eq!(g.time_left, 13);
        assert!(!g.sticky);
        assert_eq!(g.frame.row_count, 3);
        // UNLOADED: identical stream consumption, no position shift.
        let mut bare = GorePool::new();
        let mut rng_b = VecRng::new(vec![10, 20, 3]);
        let n = bare.new_gore(
            &mut rng_b, &world, Vec2::new(100.0, 100.0), Vec2::ZERO, 1225, 2.0,
        );
        assert_eq!(rng_b.idx, 3, "the Next(6) is unconditional");
        assert_eq!(bare.slot(n).position, Vec2::new(100.0, 100.0));
        assert_eq!(bare.slot(n).time_left, 13);
    }

    /// W4-g68 wiring-2 A/B: the 411-430 arm (`Gore.cs:1407-1421`) — the
    /// unloaded-extra-RNG census. LOADED: FOUR extra draws
    /// (`Next(90,150)`, `Next(40,90)`, `Next(4)`, `Next(goreTime/2,
    /// goreTime*2)`), the position adjusts (Y is the FULL height), the
    /// velocity scales, the `+Next(4)*5` type bump and sticky=true.
    /// UNLOADED: only the two kick draws — the A/B delta is exactly 4.
    /// `goreTime == 0` adds a FIFTH draw (`Next(150,600)`).
    #[test]
    fn w4_g68_the_pages_arm_rolls_four_extra_draws_when_loaded() {
        let world = world();
        let mut loaded = GorePool::new();
        loaded.set_texture_sizes(std::collections::HashMap::from([(411, (16, 20))]));
        // kick(10,20), vy Next(90,150)=120 → 1.2, vx Next(40,90)=50 → 0.5,
        // bump Next(4)=2 → +10 (type 421), life Next(300,1200)=777.
        let mut rng = VecRng::new(vec![10, 20, 120, 50, 2, 777]);
        let n = loaded.new_gore(
            &mut rng, &world, Vec2::new(100.0, 100.0), Vec2::new(4.0, 6.0), 411, 1.0,
        );
        assert_eq!(rng.idx, 6, "2 kicks + the arm's 4 draws");
        let g = loaded.slot(n);
        assert_eq!(g.gore_type, 421, "the +10 bump");
        assert_eq!(g.position, Vec2::new(100.0 - 8.0, 100.0 - 20.0), "Y is the FULL height");
        // The kick wrote velocity (4+2, 6-1) = (6, 5); the arm scales it.
        // The expected product re-derives the C# float shape (120f * 0.01f
        // is NOT the 1.2 literal — one ulp apart).
        assert_eq!(g.velocity, Vec2::new(6.0 * (50.0_f32 * 0.01), 5.0 * (120.0_f32 * 0.01)));
        assert_eq!(g.time_left, 777);
        assert!(g.sticky);
        // The bump's final type picks the texture size (421 absent here —
        // the C#'s lazy read at draw time; the ARM gate read 411).
        assert_eq!(g.texture_size, None);
        // UNLOADED: two draws only, no shift, no bump, generic sticky chain.
        let mut bare = GorePool::new();
        let mut rng_b = VecRng::new(vec![10, 20]);
        let n = bare.new_gore(
            &mut rng_b, &world, Vec2::new(100.0, 100.0), Vec2::new(4.0, 6.0), 411, 1.0,
        );
        assert_eq!(rng_b.idx, 2, "the unloaded branch draws NOTHING extra");
        let g = bare.slot(n);
        assert_eq!(g.gore_type, 411);
        assert_eq!(g.position, Vec2::new(100.0, 100.0));
        assert_eq!(g.velocity, Vec2::new(6.0, 5.0));
        assert_eq!(g.time_left, GORE_TIME_DEFAULT);
        // goreTime == 0: the fifth draw fires (Next(150,600)) and the
        // sticky chain's first arm (goreTime==0 → sticky=false) is
        // overwritten by the loaded arm's sticky=true. The life roll is
        // Next(0, 0) — the C# STILL draws a sample for an empty range
        // (`UnifiedRandom.cs:126-130`), so the census uses a real stream
        // twin (VecRng's bounds assert cannot express the empty range).
        let mut zero = GorePool::new();
        zero.gore_time = 0;
        zero.set_texture_sizes(std::collections::HashMap::from([(411, (16, 20))]));
        let mut rng_z = UnifiedRandom::new(9);
        let n = zero.new_gore(
            &mut rng_z, &world, Vec2::new(100.0, 100.0), Vec2::ZERO, 411, 1.0,
        );
        let mut twin = UnifiedRandom::new(9);
        twin.next_range(10, 31);
        twin.next_range(-20, 21);
        twin.next_range(90, 150);
        twin.next_range(40, 90);
        twin.next_max(4);
        twin.next_range(0, 0); // the empty-range life roll still draws
        let life2 = twin.next_range(150, 600);
        assert_eq!(rng_z, twin, "the zero-time branch adds the fifth draw");
        assert_eq!(zero.slot(n).time_left, life2);
        assert!(zero.slot(n).sticky);
    }

    /// W4-g68 wiring-2 A/B: the SpecialAI-2 arm (`Gore.cs:1430-1441`,
    /// types 825-827). LOADED: ONE extra draw (`Next(goreTime/2,
    /// goreTime+1)`), alpha 150, velocity reset to the ARGUMENT (the kick
    /// discarded), the raw-half position adjust, sticky false. UNLOADED:
    /// sticky false only — the A/B delta is exactly 1 draw.
    #[test]
    fn w4_g68_the_smoke_arm_rolls_one_extra_draw_when_loaded() {
        let world = world();
        let mut loaded = GorePool::new();
        loaded.set_texture_sizes(std::collections::HashMap::from([(825, (24, 24))]));
        let mut rng = VecRng::new(vec![10, 20, 450]);
        let n = loaded.new_gore(
            &mut rng, &world, Vec2::new(100.0, 100.0), Vec2::new(1.5, -0.5), 825, 1.0,
        );
        assert_eq!(rng.idx, 3, "2 kicks + the arm's 1 draw");
        let g = loaded.slot(n);
        assert_eq!(g.alpha, 150);
        assert_eq!(g.velocity, Vec2::new(1.5, -0.5), "the argument velocity survives");
        assert_eq!(g.position, Vec2::new(100.0 - 12.0, 100.0 - 12.0));
        assert_eq!(g.time_left, 450);
        assert!(!g.sticky);
        // UNLOADED: the kick velocity stays, alpha 0, no shift.
        let mut bare = GorePool::new();
        let mut rng_b = VecRng::new(vec![10, 20]);
        let n = bare.new_gore(
            &mut rng_b, &world, Vec2::new(100.0, 100.0), Vec2::new(1.5, -0.5), 825, 1.0,
        );
        assert_eq!(rng_b.idx, 2, "the unloaded branch draws NOTHING extra");
        let g = bare.slot(n);
        assert_eq!(g.alpha, 0);
        assert_eq!(g.velocity, Vec2::new(3.5, -1.5));
        assert_eq!(g.position, Vec2::new(100.0, 100.0));
        assert!(!g.sticky, "the sticky flag is unconditional");
    }

    /// The falling-leaf liquid probe (`Gore.cs:1180`) centres on
    /// `(Width, Height) * 0.5 + position`. Use a 1-column × 1-row sheet
    /// (12×12, no padding cut) so the half-size is 6: pos.x 170.1 − the
    /// tick's −sin(π/180) drift + 6 ≈ 176.08 → tile 11, while the 1f
    /// fallback's +0.5 stays in tile 10. The liquid in THAT cell flips
    /// `velocity.Y` to −1 (the rest sentinel).
    #[test]
    fn w4_g68_the_fall_probe_uses_the_sheet_half_size() {
        let mut world = leaf_physics_world();
        // The probe box is `(vector.x, vector.y + 6, 4, 4)` with
        // vector = pos + (4, 0) → (174, 306..310); tile 11 spans x
        // 176..192 (overlap) and the liquid rect must reach y < 310:
        // liquid 16 → top 304 + 1 = 305 (the fd9415f3 fixture lesson).
        let i = world.index(11, 19);
        world.tiles[i].liquid = 16;
        let mut pool = GorePool::new();
        pool.set_texture_sizes(std::collections::HashMap::from([(910, (12, 12))]));
        spawn_leaf(&mut pool, &world, Vec2::new(170.1, 300.0));
        // The leaf frame is 32×8; pin it to 1×1 for this probe witness (the
        // C# reads whatever frame the slot carries — a paint-swapped leaf
        // can ride any cell shape).
        pool.slot_mut(0).frame = SpriteFrame::new(1, 1);
        pool.update(&mut VecRng::new(vec![]), &world, &FreeCollision, 0.0, None);
        assert_eq!(
            pool.slot(0).velocity.y,
            -1.0,
            "the sheet-sized probe found the liquid at tile 11"
        );
        // Without the seam the probe reads tile 10 (dry) and keeps falling.
        let mut bare = GorePool::new();
        spawn_leaf(&mut bare, &world, Vec2::new(170.1, 300.0));
        bare.update(&mut VecRng::new(vec![]), &world, &FreeCollision, 0.0, None);
        assert!(bare.slot(0).velocity.y > 0.0);
    }

    /// The ground freeing up under a resting leaf resumes the fall
    /// (`Gore.cs:1133-1138`): `velocity.Y = 0.1`, `timeLeft = 0`,
    /// `alpha += 20` — then the same tick's fade tail (`:1149-1156`) adds
    /// one `DisappearSpeedAlpha` point (timeLeft is already 0), so the
    /// observable alpha after the tick is 21.
    #[test]
    fn w4_g68_open_ground_resumes_the_fall_and_starts_the_fade() {
        let world = leaf_physics_world(); // no solid under the probe
        let mut pool = GorePool::new();
        spawn_leaf(&mut pool, &world, Vec2::new(160.0, 160.0));
        pool.slot_mut(0).velocity = Vec2::new(0.0, -1.0);
        pool.slot_mut(0).alpha = 0;
        pool.update(&mut VecRng::new(vec![]), &world, &FreeCollision, 0.0, None);
        let g = pool.slot(0);
        assert_eq!(g.velocity.y, 0.1);
        assert_eq!(g.time_left, 0);
        assert_eq!(g.alpha, 21, "the +20 resume plus the same-tick fade point");
    }

    /// The fade tail (`Gore.cs:1201-1208` + `:1055-1058`): one
    /// `DisappearSpeed` point per tick while `timeLeft > 0`, then one alpha
    /// point per tick; `alpha >= 255` deactivates.
    #[test]
    fn w4_g68_the_leaf_fade_counts_down_then_dies() {
        let world = leaf_physics_world();
        let mut pool = GorePool::new();
        spawn_leaf(&mut pool, &world, Vec2::new(500.0, 300.0));
        pool.slot_mut(0).time_left = 1;
        // Keep the leaf falling in open air; the FreeCollision seam never
        // brakes, so the rest switch never fires.
        pool.update(&mut VecRng::new(vec![]), &world, &FreeCollision, 0.0, None);
        assert_eq!(pool.slot(0).time_left, 0);
        assert_eq!(pool.slot(0).alpha, 0);
        for tick in 1..=255 {
            pool.update(&mut VecRng::new(vec![]), &world, &FreeCollision, 0.0, None);
            if tick < 255 {
                assert!(pool.slot(0).active);
                assert_eq!(pool.slot(0).alpha, tick);
            }
        }
        assert!(!pool.slot(0).active, "alpha 255 kills the leaf");
    }
}
