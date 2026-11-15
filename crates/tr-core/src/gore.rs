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
//! `:1336`) then `NextFloat()` (the re-scaled size, `:1338`). Nothing else
//! rolls: the remaining arms (`:1355-1429`) draw no random numbers.
//!
//! Documented drops (each at its site below): the `Main.rand == null` re-seed
//! (`Gore.cs:1300-1303`; streams are caller-owned and never null), the
//! `Main.netMode == 2` server guard (`:1288-1291`; single-player port), and
//! the texture-dependent arms — the port has no texture tracking in `tr-core`,
//! so every `TextureAssets.Gore[...].IsLoaded` test is `false` (the geometry
//! fallbacks are exact, the position-adjust arms are no-ops until the asset
//! seam lands), and `GoreID.Sets.IsDrip`/`SpecialAI`/`DisappearSpeed` are not
//! ported yet, so the arms gated on them (`:1355-1360`, `:1390-1430`) wait for
//! the `GoreID` step.

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
    /// returns `1f` — and the port has no texture tracking yet, so the
    /// fallback is the only path until the asset seam lands (see the module
    /// drops).
    pub fn width(&self) -> f32 {
        1.0
    }

    /// `Gore.Height` (`Gore.cs:56-66`): the `1f` fallback, as [`Gore::width`].
    pub fn height(&self) -> f32 {
        1.0
    }

    /// `Gore.AABBRectangle` (`Gore.cs:68-79`): the texture-unloaded fallback
    /// `new Rectangle(0, 0, 1, 1)` — see [`Gore::width`].
    pub fn aabb_rectangle(&self) -> (i32, i32, i32, i32) {
        (0, 0, 1, 1)
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
        }
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
        // `GoreID.Sets.IsDrip` arm (`Gore.cs:1355-1360`): the Sets table is
        // not ported yet (module docs) — no-op here.
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
        // `Gore.cs:1379-1389` (type 1225): every field write is inside
        // `TextureAssets.Gore[Type].IsLoaded` except the `timeLeft` roll
        // (`:1382`) and the two flags — ported exactly, position adjustment
        // waits for the asset seam (module drops).
        if gore_type == 1225 {
            let gore = self.slot_mut(num);
            gore.frame = SpriteFrame::new(1, 3);
            gore.time_left = 10 + rng.next_max(6);
            gore.sticky = false;
        }
        // `GoreID.Sets.SpecialAI` arms (`Gore.cs:1390-1406`) and the 411-430
        // (`:1407-1421`), 907-909 (`:1422-1429`) and SpecialAI == 2
        // (`:1430-1441`) chains: the Sets tables and the texture probes are
        // not ported yet (module drops).
        if matches!(num2_dropped(gore_type), 4 | 5 | 6) {
            // The alpha/timeLeft pairs of `SpecialAI == 4/5/6`
            // (`Gore.cs:1442-1456`): the gate itself is the dropped
            // `SpecialAI` table — unreachable until it lands; kept here so
            // the follow-up is a table-only change.
            let gore = self.slot_mut(num);
            gore.alpha = 254;
            gore.time_left = match num2_dropped(gore_type) {
                4 => 300,
                5 => 240,
                _ => 480,
            };
        }
        // Deactivate if outside of the world (`Gore.cs:1457-1460`): returns
        // the sentinel with the slot left inactive — the C#'s own shape.
        if self.deactivate_if_outside_of_world(num, world) {
            return GORE_SENTINEL;
        }
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

/// The `GoreID.Sets.SpecialAI[type]` lookup stub: the table is not ported yet,
/// so this reports "no special AI" for every type (module drops) and the
/// 4/5/6 arm above is currently unreachable. Kept as the single seam the
/// `GoreID` step fills in.
fn num2_dropped(_gore_type: i32) -> i32 {
    0
}
// Adding required GoreID.Sets table and Update logic as per exploration.
