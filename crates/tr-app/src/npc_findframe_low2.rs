//! W4-d45 — FindFrame low-band slice 2: the SECOND 12 missing case ids
//! of `NPC.FindFrame` (`NPC.cs:66684-77370`) in the low band, counted
//! off the wave census (`ff_missing` — every C# top-level case id whose
//! port has no arm: the `npc.rs::find_frame` match kinds mapped through
//! the bestiary net ids + the wave-A/wave-B module rosters, diffed
//! against the 602 top-level `case` labels of the C# switch). Slice 1
//! (`task/W4-d44-findframe-low-1`, `npc_findframe_low1.rs`) owns the
//! FIRST 12 (37/43/46/47/48/56/57/61/62/63/64/67); THIS module owns
//! **69/73/74/83/84/93/94/101/102/103/116/120** — the Antlion, the
//! Goblin Scout, the Bird, the Cursed Hammer / Enchanted Sword pair,
//! the Giant Bat, the Corruptor, the Clinger, the Angler Fish, the
//! Green Jellyfish, The Hungry II and the Chaos Elemental
//! (`ID/NPCID.cs:10815-10917`). Landed in two 6-kind batches
//! (69/73/74/83/84/93, then 94/101/102/103/116/120).
//!
//! Shape: the kinds are NOT in `NpcKind` yet (their AI/spawn machines
//! are later rounds), so — like `npc_findframe_a.rs` /
//! `npc_findframe_b.rs` (S1-D-3 waves A/B) and `npc_findframe_mid1.rs`
//! (W4-d46) — the frame machines land first as pure functions over a
//! small [`FrameState`]/[`FrameInput`] pair. The wiring is a
//! `cs_id`-keyed hook at the head of `npc.rs::find_frame`
//! ([`find_frame_kind_for_cs_id`] + `Npc::find_frame_wave_low2`, the
//! W4-d46 mid-1 precedent): NO current kind's `cs_id` lands in this set
//! (the bestiary's net ids skip all twelve — `bestiary.rs`'s entry
//! list), so the hook is stream-inert today and auto-activates when
//! the roster wave adds the kinds. No arm draws `Main.rand` (the
//! floater group's dust spawns are types 60/634's, not the Giant
//! Bat's), so the wiring cannot move a stream.
//!
//! Row convention: the port tracks frame ROW indices, not pixels — the
//! C#'s `num` (the sheet height / `Main.npcFrameCount[type]`,
//! `NPC.cs:66694`) cancels out of every comparison, exactly as in
//! `npc.rs::find_frame`'s existing arms. The per-kind row counts are
//! `Main.npcFrameCount` (`Main.cs:67082`'s 697-entry initializer), counted
//! entry-by-entry in [`FindFrameLow2Kind::frame_count`] and cross-checked
//! against the port's own `render/entities.rs::NPC_FRAME_COUNT` table.
//!
//! Deliberate skips (recorded in docs/ANALYSIS.md "W4-d45 FindFrame
//! low-band slice 2"):
//! * the humanoid town group's `ai[0]` sit/talk/attack pose zones
//!   (`NPC.cs:68662-69488`) — the port's fighter machines do not model
//!   them (the `npc_findframe_b.rs::humanoid_fighter_frame` precedent);
//!   the 73/120 fighters' turn counter leaves `ai[0]` at 0/1
//!   (`NPC.cs:57381-57397`). Both are aiStyle-3 fighters with NO
//!   `townNPC` row (`NPC.cs:9732-9746`/`:10453-10465`), so
//!   `num57 = ExtraFramesCount` never applies (`isLikeATownNPC` is
//!   `townNPC || type == 453`, `:6856-6866`); the 120's `:69647`
//!   air-pin-0 membership IS carried (batch 2);
//! * the floater group's type-60 flame dust (`:74839-74843`), the
//!   type-634 `Main.rand.Next(10)` dust (`:74844-74849`), the 210/211
//!   double-increment + 0.2 tilt (`:74859-74863`) and the
//!   49/51/60/634 `num292--` list (`:74867-74870`) — other members',
//!   the Giant Bat is a plain 6-tick full-count member;
//! * the bird / enchanted-weapon groups' other members (297/298/442;
//!   179) — the shared arms are ported for this slice's members only
//!   (the d44 slice-1 precedent);
//! * the disabled profile-texture block (`NPC.cs:68637`, `if (false &&
//!   …)`) — dead in the C# itself.

// The wiring is a later round (the kinds are not in `NpcKind` yet) — the
// `npc_findframe_a.rs`/`npc_findframe_mid1.rs` precedent.
#![allow(dead_code)]

/// The slice-2 kinds, keyed by their VANILLA `NPC.type`
/// (`ID/NPCID.cs`'s `public const short` rows). Batch 1:
/// 69/73/74/83/84/93.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FindFrameLow2Kind {
    /// `NPCID.Antlion` = 69 (`NPCID.cs:10815`) — `case 69`.
    Antlion,
    /// `NPCID.GoblinScout` = 73 (`NPCID.cs:10823`) — `case 73` (the
    /// humanoid town case list, the aiStyle-3 fighter path).
    GoblinScout,
    /// `NPCID.Bird` = 74 (`NPCID.cs:10825`) — `case 74` (the
    /// 297/298/442 critter-flap group).
    Bird,
    /// `NPCID.CursedHammer` = 83 (`NPCID.cs:10843`) — `case 83` (the
    /// 84/179 enchanted-weapon spin group).
    CursedHammer,
    /// `NPCID.EnchantedSword` = 84 (`NPCID.cs:10845`) — `case 84`.
    EnchantedSword,
    /// `NPCID.GiantBat` = 93 (`NPCID.cs:10863`) — `case 93` (the
    /// floater group, the plain 6-tick member).
    GiantBat,
}

impl FindFrameLow2Kind {
    /// The vanilla `NPC.type` (`ID/NPCID.cs`'s const rows).
    pub fn vanilla_type(self) -> i16 {
        match self {
            Self::Antlion => 69,
            Self::GoblinScout => 73,
            Self::Bird => 74,
            Self::CursedHammer => 83,
            Self::EnchantedSword => 84,
            Self::GiantBat => 93,
        }
    }

    /// `Main.npcFrameCount[type]` — the row counts counted entry-by-entry
    /// out of the 697-entry initializer (`Main.cs:67082-67153`; entry
    /// `type` sits on line `67084 + type / 10`, slot `type % 10`),
    /// cross-checked against `render/entities.rs::NPC_FRAME_COUNT`.
    pub fn frame_count(self) -> u32 {
        match self {
            // Main.cs:67090 (entries 60-69): 69 = 5.
            Self::Antlion => 5,
            // Main.cs:67091 (entries 70-79): 73 = 16, 74 = 5.
            Self::GoblinScout => 16,
            Self::Bird => 5,
            // Main.cs:67092 (entries 80-89): 83/84 = 6.
            Self::CursedHammer => 6,
            Self::EnchantedSword => 6,
            // Main.cs:67093 (entries 90-99): 93 = 4.
            Self::GiantBat => 4,
        }
    }
}

/// The `cs_id`-keyed wiring seam (the W4-d46 mid-1 precedent): map a
/// kind's vanilla `NPC.type` onto its slice-2 machine. NO current
/// `NpcKind::cs_id` lands in the set today — the hook is stream-inert
/// until the roster wave adds the kinds.
pub fn find_frame_kind_for_cs_id(cs_id: u16) -> Option<FindFrameLow2Kind> {
    match cs_id {
        69 => Some(FindFrameLow2Kind::Antlion),
        73 => Some(FindFrameLow2Kind::GoblinScout),
        74 => Some(FindFrameLow2Kind::Bird),
        83 => Some(FindFrameLow2Kind::CursedHammer),
        84 => Some(FindFrameLow2Kind::EnchantedSword),
        93 => Some(FindFrameLow2Kind::GiantBat),
        _ => None,
    }
}

/// The `NPC` fields `FindFrame` reads/writes for these arms:
/// `frame.Y / num` (the row), `frameCounter`, `spriteDirection`,
/// `rotation`. The wiring step copies them to/from `Npc`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FrameState {
    /// `frame.Y / num` — the sheet row.
    pub row: u32,
    /// `NPC.frameCounter`.
    pub counter: f32,
    /// `NPC.spriteDirection` (±1).
    pub sprite_dir: i32,
    /// `NPC.rotation` — only the tilt arms (74/93) write it.
    pub rotation: f32,
}

/// The per-tick inputs the arms read: `velocity`, `direction`, `wet` and
/// the `ai` slots (the Antlion's `ai[0]` dig gate and the enchanted
/// weapons' `ai[0] == 2` spin gate read them). `ai_action` is not
/// carried — none of these kinds reads the `num2` motion class
/// (`NPC.cs:66696-66704`) in its arm.
#[derive(Clone, Copy, Debug)]
pub struct FrameInput {
    pub vx: f32,
    pub vy: f32,
    /// `Entity.direction` (±1; 0 leaves `spriteDirection` alone in the
    /// arms that gate on `direction == 1` / `== -1`; the unconditional
    /// writers copy it verbatim, 0 included).
    pub direction: i32,
    pub wet: bool,
    pub ai: [f32; 4],
}

impl Default for FrameInput {
    fn default() -> Self {
        Self { vx: 0.0, vy: 0.0, direction: 1, wet: false, ai: [0.0; 4] }
    }
}

/// `NPC.FindFrame` (`NPC.cs:66684`) for the slice-2 kinds — one tick.
pub fn find_frame_low2(kind: FindFrameLow2Kind, s: &mut FrameState, i: &FrameInput) {
    let count = kind.frame_count();
    match kind {
        // `case 69` (`NPC.cs:74291-74310`): the Antlion walks rows
        // 0..count-2 on a 6-tick clock while `ai[0] < 190` (the wrap
        // is INSIDE the step: the row never reaches count-1 through
        // it); `ai[0] >= 190` (the buried pose) pins the LAST row
        // (count-1) and zeroes the counter. No flip/tilt writes.
        FindFrameLow2Kind::Antlion => {
            if i.ai[0] < 190.0 {
                // `:74294-74303`.
                s.counter += 1.0;
                if s.counter >= 6.0 {
                    s.counter = 0.0;
                    s.row += 1;
                    if s.row >= count - 1 {
                        s.row = 0;
                    }
                }
            } else {
                // `:74307-74308`.
                s.counter = 0.0;
                s.row = count - 1;
            }
        }
        // `case 73` (`NPC.cs:68557` in the humanoid town case list,
        // `:68634-69655`): the Goblin Scout is an aiStyle-3 fighter
        // (`:9737`) with NO `townNPC` row, so `num57` = 0 — the walk
        // band wraps at the FULL 16 rows; the idle snaps to row 0
        // (73 is not the 140/489 idle-row-1 pair nor the 638 cycle);
        // the air holds row 1 (73 is NOT in the `:69647` pin list).
        FindFrameLow2Kind::GoblinScout => {
            humanoid_fighter_low2(s, i, count);
        }
        // `case 74 : case 297 : case 298 : case 442`
        // (`NPC.cs:73628-73653`): the Bird flips by `direction`
        // unconditionally, tilts `vx * 0.1`; the FULL still snap
        // (`vx == 0 && vy == 0`) pins row 4 (`num * 4`) + zero
        // counter; otherwise the 4-tick cycle walks rows
        // 0..count-2 (the wrap bound is `count - 1`, so the still
        // row 4 is never reached through the cycle).
        FindFrameLow2Kind::Bird => {
            // `:73633-73634`.
            s.sprite_dir = i.direction;
            s.rotation = i.vx * 0.1;
            if i.vx == 0.0 && i.vy == 0.0 {
                // `:73635-73640`.
                s.row = 4;
                s.counter = 0.0;
                return;
            }
            // `:73641-73651` — num124 = count - 1.
            s.counter += 1.0;
            if s.counter >= 4.0 {
                s.row += 1;
                s.counter = 0.0;
            }
            if s.row >= count - 1 {
                s.row = 0;
            }
        }
        // `case 83 : case 84 : case 179` (`NPC.cs:74622-74641`): the
        // enchanted weapons spin on a 4-tick clock over the FULL
        // count (the wrap is INSIDE the step); `ai[0] == 2` pins
        // row 0 + zero counter. No flip/tilt writes.
        FindFrameLow2Kind::CursedHammer | FindFrameLow2Kind::EnchantedSword => {
            if i.ai[0] == 2.0 {
                // `:74625-74630`.
                s.counter = 0.0;
                s.row = 0;
                return;
            }
            // `:74631-74640`.
            s.counter += 1.0;
            if s.counter >= 4.0 {
                s.counter = 0.0;
                s.row += 1;
                if s.row >= count {
                    s.row = 0;
                }
            }
        }
        // `case 93` in the floater group (`NPC.cs:74823-74885`,
        // shared with 48/49/51/60/82/137/182/210/211/253/316/634/
        // 662/664): the Giant Bat faces by velocity sign (0 leaves
        // the flip alone), tilts `vx * 0.1`, and steps every 6 ticks
        // (`num291` = 6 — the type-48 override does not carry 93)
        // over the FULL 4 rows (the `num292--` list 49/51/60/634
        // does not carry 93). The group's dust spawns (60/634) and
        // the 210/211 double-increment are other kinds'.
        FindFrameLow2Kind::GiantBat => {
            // `:74850-74857`.
            if i.vx > 0.0 {
                s.sprite_dir = 1;
            }
            if i.vx < 0.0 {
                s.sprite_dir = -1;
            }
            // `:74858`.
            s.rotation = i.vx * 0.1;
            // `:74864-74883`.
            s.counter += 1.0;
            if s.counter >= 6.0 {
                s.row += 1;
                s.counter = 0.0;
            }
            if s.row >= count {
                s.row = 0;
            }
        }
    }
}

/// The humanoid-group fighter walk (`NPC.cs:68647-69655`) for this
/// slice's non-town members: the grounded flip (`:68649-68656` — only
/// `direction` ±1 writes), the idle snap (`:69515-69519` — row 0;
/// neither member is the 140/489 idle-row-1 pair), the `|vx| * 2 + 1`
/// accumulator stepping every > 6 over rows 2..count-1
/// (`:69550-69584` — `num108` = 6, `num109` = row 2, `num57` = 0), and
/// the airborne row 1 (`:69642-69646`; the `:69647-69650` pin-0 tail
/// is the batch-2 Chaos Elemental's, applied at its arm). The town
/// `ai[0]` pose zones (`:68662-69488`) are unreachable for these
/// aiStyle-3 fighters (`NPC.cs:57381-57397`) and are not carried —
/// the `npc_findframe_b.rs::humanoid_fighter_frame` precedent.
fn humanoid_fighter_low2(s: &mut FrameState, i: &FrameInput, count: u32) {
    if i.vy == 0.0 {
        if i.direction == 1 {
            s.sprite_dir = 1;
        }
        if i.direction == -1 {
            s.sprite_dir = -1;
        }
        if i.vx == 0.0 {
            // `NPC.cs:69515-69519`.
            s.row = 0;
            s.counter = 0.0;
        } else {
            // `NPC.cs:69550-69584` — the floor, the |vx| * 2 add THEN
            // the ++, the > 6 step, the wrap (num57 = 0).
            if s.row < 2 {
                s.row = 2;
            }
            s.counter += i.vx.abs() * 2.0;
            s.counter += 1.0;
            if s.counter > 6.0 {
                s.row += 1;
                s.counter = 0.0;
            }
            if s.row >= count {
                s.row = 2;
            }
        }
    } else {
        // `NPC.cs:69642-69646`: the group's air row 1.
        s.counter = 0.0;
        s.row = 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(vx: f32, vy: f32) -> FrameInput {
        FrameInput { vx, vy, ..Default::default() }
    }

    fn rows_over(kind: FindFrameLow2Kind, i: FrameInput, ticks: usize) -> Vec<u32> {
        let mut s = FrameState::default();
        let mut out = Vec::new();
        for _ in 0..ticks {
            find_frame_low2(kind, &mut s, &i);
            out.push(s.row);
        }
        out
    }

    /// The batch-1 `Main.npcFrameCount` rows, counted entry-by-entry
    /// out of the `Main.cs:67082` initializer (the line/slot arithmetic
    /// in `frame_count`'s doc).
    #[test]
    fn the_batch_1_frame_counts_match_the_main_initializer() {
        let table = [
            (FindFrameLow2Kind::Antlion, 69, 5),
            (FindFrameLow2Kind::GoblinScout, 73, 16),
            (FindFrameLow2Kind::Bird, 74, 5),
            (FindFrameLow2Kind::CursedHammer, 83, 6),
            (FindFrameLow2Kind::EnchantedSword, 84, 6),
            (FindFrameLow2Kind::GiantBat, 93, 4),
        ];
        for (kind, vanilla, count) in table {
            assert_eq!(kind.vanilla_type(), vanilla, "{kind:?} id");
            assert_eq!(kind.frame_count(), count, "{kind:?} npcFrameCount");
        }
    }

    /// The Antlion (`NPC.cs:74292-74309`): the walk strip wraps at
    /// count-1 = 4 INSIDE the 6-tick step (rows 0..3 only), and the
    /// `ai[0] >= 190` buried gate pins the last row 4 + zero counter.
    #[test]
    fn the_antlion_walks_rows_zero_to_three_and_buries_on_row_four() {
        let rows = rows_over(FindFrameLow2Kind::Antlion, input(1.0, 0.0), 24);
        assert_eq!(
            rows,
            [0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 3, 3, 3, 3, 3, 3, 0]
        );
        // The buried gate (the inclusive >= 190).
        let mut s = FrameState { row: 2, counter: 5.0, sprite_dir: 1, rotation: 0.0 };
        let buried = FrameInput { ai: [190.0, 0.0, 0.0, 0.0], ..input(0.0, 0.0) };
        find_frame_low2(FindFrameLow2Kind::Antlion, &mut s, &buried);
        assert_eq!((s.row, s.counter), (4, 0.0));
        // No flip write — the arm carries none.
        assert_eq!(s.sprite_dir, 1);
    }

    /// The Goblin Scout's fighter walk (`NPC.cs:69550-69584`): the
    /// `|vx| * 2 + 1` = 3 a tick at vx 1, the row steps at > 6, the
    /// band floor is row 2 and the wrap at the FULL 16 (num57 = 0 —
    /// no townNPC row at `:9732-9746`) returns to 2.
    #[test]
    fn the_goblin_scout_walk_wraps_at_sixteen_the_full_sheet() {
        let mut s = FrameState::default();
        let i = input(1.0, 0.0);
        let mut rows = Vec::new();
        for _ in 0..6 {
            find_frame_low2(FindFrameLow2Kind::GoblinScout, &mut s, &i);
            rows.push(s.row);
        }
        assert_eq!(rows, [2, 2, 3, 3, 3, 4]);
        assert_eq!(s.counter, 0.0);
        assert_eq!(s.sprite_dir, 1);
        // Row 15 + one step = 16 → the wrap to 2 (`:69581-69584`).
        s.row = 15;
        s.counter = 6.0;
        find_frame_low2(FindFrameLow2Kind::GoblinScout, &mut s, &i);
        assert_eq!(s.row, 2);
    }

    /// The Goblin Scout's idle row 0 + air row 1 (`NPC.cs:69515-69519`,
    /// `:69642-69646` — 73 is NOT in the `:69647` pin list), and the
    /// direction-0 no-flip gate (`:68649-68656`).
    #[test]
    fn the_goblin_scout_idle_air_and_direction_zero() {
        let mut s = FrameState { row: 9, counter: 4.0, sprite_dir: -1, rotation: 0.0 };
        let i = FrameInput { direction: 0, ..input(0.0, 0.0) };
        find_frame_low2(FindFrameLow2Kind::GoblinScout, &mut s, &i);
        assert_eq!((s.row, s.counter), (0, 0.0));
        assert_eq!(s.sprite_dir, -1);
        find_frame_low2(FindFrameLow2Kind::GoblinScout, &mut s, &input(0.0, -2.0));
        assert_eq!((s.row, s.counter), (1, 0.0));
        s.row = 9;
        find_frame_low2(FindFrameLow2Kind::GoblinScout, &mut s, &input(0.0, 2.0));
        assert_eq!(s.row, 1);
    }

    /// The Bird (`NPC.cs:73633-73651`): the still snap pins row 4 (the
    /// last row); the moving 4-tick cycle walks rows 0..3 — the wrap
    /// bound is `count - 1` = 4, so the cycle NEVER shows row 4. The
    /// flip copies direction verbatim (0 included), the tilt vx * 0.1.
    #[test]
    fn the_bird_still_row_four_and_the_count_minus_one_cycle() {
        let mut s = FrameState { row: 2, counter: 3.0, sprite_dir: 1, rotation: 0.5 };
        find_frame_low2(FindFrameLow2Kind::Bird, &mut s, &input(0.0, 0.0));
        assert_eq!((s.row, s.counter), (4, 0.0));
        // Moving (vy only still counts — the snap needs BOTH zero).
        let i = input(0.0, 1.0);
        let mut s = FrameState::default();
        let mut rows = Vec::new();
        for _ in 0..16 {
            find_frame_low2(FindFrameLow2Kind::Bird, &mut s, &i);
            rows.push(s.row);
        }
        assert_eq!(rows, [0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 0]);
        // The unconditional flip + tilt.
        let tilt = FrameInput { direction: 0, ..input(-2.0, 0.0) };
        find_frame_low2(FindFrameLow2Kind::Bird, &mut s, &tilt);
        assert_eq!(s.sprite_dir, 0);
        assert_eq!(s.rotation, -0.2);
    }

    /// The enchanted weapons (`NPC.cs:74625-74640`): the 4-tick spin
    /// over the FULL 6 rows (the wrap inside the step), the
    /// `ai[0] == 2` pin to row 0 + zero counter. Both ride the arm.
    #[test]
    fn the_enchanted_weapons_spin_six_rows_and_pin_on_ai_zero_two() {
        for kind in [FindFrameLow2Kind::CursedHammer, FindFrameLow2Kind::EnchantedSword] {
            let rows = rows_over(kind, input(0.0, 0.0), 24);
            assert_eq!(
                rows,
                [0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 0]
            );
            let mut s = FrameState { row: 3, counter: 2.0, sprite_dir: -1, rotation: 0.0 };
            let pinned = FrameInput { ai: [2.0, 0.0, 0.0, 0.0], ..input(0.0, 0.0) };
            find_frame_low2(kind, &mut s, &pinned);
            assert_eq!((s.row, s.counter), (0, 0.0));
            // ai[0] == 1 does NOT pin (the exact == 2 gate).
            let other = FrameInput { ai: [1.0, 0.0, 0.0, 0.0], ..input(0.0, 0.0) };
            find_frame_low2(kind, &mut s, &other);
            assert_eq!(s.counter, 1.0);
        }
    }

    /// The Giant Bat's floater strip (`NPC.cs:74850-74883`): the
    /// velocity flip (0 leaves it alone), the vx * 0.1 tilt, the
    /// 6-tick step over the FULL 4 rows (93 is in neither the
    /// num291=5 override nor the num292-- list).
    #[test]
    fn the_giant_bat_flaps_every_six_ticks_over_four_rows() {
        let mut s = FrameState::default();
        let i = input(2.0, 0.0);
        let mut rows = Vec::new();
        for _ in 0..12 {
            find_frame_low2(FindFrameLow2Kind::GiantBat, &mut s, &i);
            rows.push(s.row);
        }
        assert_eq!(rows, [0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 2]);
        assert_eq!(s.sprite_dir, 1);
        assert_eq!(s.rotation, 0.2);
        // Row 3 + one step = 4 → the wrap to 0.
        s.row = 3;
        s.counter = 5.0;
        find_frame_low2(FindFrameLow2Kind::GiantBat, &mut s, &i);
        assert_eq!(s.row, 0);
        // vx 0: no flip write, the tilt zeroes.
        s.sprite_dir = -1;
        find_frame_low2(FindFrameLow2Kind::GiantBat, &mut s, &input(0.0, 0.0));
        assert_eq!(s.sprite_dir, -1);
        assert_eq!(s.rotation, 0.0);
    }

    /// The `cs_id` seam maps exactly the six batch-1 ids (batch 2's
    /// six land with its commit).
    #[test]
    fn the_batch_1_cs_id_seam_maps_exactly_the_six_ids() {
        for (id, kind) in [
            (69, FindFrameLow2Kind::Antlion),
            (73, FindFrameLow2Kind::GoblinScout),
            (74, FindFrameLow2Kind::Bird),
            (83, FindFrameLow2Kind::CursedHammer),
            (84, FindFrameLow2Kind::EnchantedSword),
            (93, FindFrameLow2Kind::GiantBat),
        ] {
            assert_eq!(find_frame_kind_for_cs_id(id), Some(kind));
        }
        for id in [0, 68, 70, 75, 82, 85, 92, 95, u16::MAX] {
            assert_eq!(find_frame_kind_for_cs_id(id), None, "id {id}");
        }
    }
}
