//! W4-e13 FindFrame coverage extra-1 — the FIRST 15 remaining
//! `NPC.FindFrame` case ids (`NPC.cs:66684-77370`) after the d44-d49
//! waves' claims, landed 6 kinds per commit. The ascending census
//! (every C# top-level `case` label of the switch on `this.type`,
//! `NPC.cs:66705`, diffed against the port's `npc.rs::find_frame` arms
//! mapped through `cs_id` + the wave-A/B rosters, minus the d44 low-1
//! 37/43/46/47/48/56/57/61/62/63/64/67, the d45 low-2 claim
//! 69/73/74/83/84/93/94/101/102/103/116/120, the d46 mid-1
//! 150/151/152/156/157/158/159/160/162/163/164/165, the d47 mid-2
//! 166/168/169/170/171/172/173/175/176/178/179/180, the d48 high-1
//! 355-366 and the d49 high-2 367/374/375/377/378/379/380/430-436/591)
//! leaves 279 ids; this module owns the first 15:
//! **121, 137, 141, 142, 148, 149, 181, 182, 185, 195, 196, 198, 199,
//! 201, 202**. Commit 1 carries the first 6 (121/137/141/142/148/149);
//! the next 15 (203/204/205/209/217/218/219/220/221/224/226/229/230/
//! 231/232) ride `task/W4-e14-findframe-extra-2`.
//!
//! Shape: the kinds are NOT in `NpcKind` yet (the roster waves
//! `task/W4-d34..d43-npc-kinds-*` own them), so — the
//! `npc_findframe_b.rs` (S1-D-3 wave B) / `npc_findframe_mid1.rs`
//! (W4-d46) precedent — the frame machines land first as pure
//! functions over a [`FrameState`]/[`FrameInput`] pair, wired through
//! a `cs_id`-keyed hook at the head of `npc.rs::find_frame`
//! ([`find_frame_kind_for_cs_id`] + `Npc::find_frame_wave_extra1`).
//! NO current kind's `cs_id` lands in the set (verified over the hand
//! rows + the bestiary net ids), so the hook is stream-inert today and
//! auto-activates when the roster wave adds the kinds. No arm draws
//! `Main.rand` (the floater group's dust is types 60/634's, the eye
//! group's is type 23's, the humanoid pose zones' draws are the
//! documented skip below), so the wiring cannot move a stream.
//!
//! Row convention: the port tracks frame ROW indices, not pixels —
//! the C#'s `num` (the sheet height / `Main.npcFrameCount[type]`,
//! `NPC.cs:66694`) cancels out of every comparison, exactly as in
//! `npc.rs::find_frame`'s existing arms. The per-kind row counts are
//! `Main.npcFrameCount` (`Main.cs:67082`'s 697-entry initializer),
//! counted entry-by-entry in [`FindFrameExtra1Kind::frame_count`] and
//! cross-checked against the port's `NPC_FRAME_COUNT` in the tests.
//!
//! Deliberate skips (recorded in docs/ANALYSIS.md "W4-e13 FindFrame
//! extra-1"): the humanoid group's town `ai[0]` pose zones
//! (`NPC.cs:68662-69488` — the sit/talk/attack/emote poses; the port's
//! town machines do not model them, the `npc_findframe_mid1.rs`
//! precedent — the zones carry `Main.rand` draws at `:68944`/`:68951`/
//! `:68959` that the skip keeps off every stream), and the case-group
//! siblings outside this wave's 15 (the eye group's 23/169/317/318/660,
//! the floater group's 253/316/634/662/664, the penguin group's
//! 168/470 — d47 owns 168, the humanoid list's many later-wave ids).
//! The penguin arm is the SAME machine as d47's 168 CorruptPenguin
//! (`case 148/149/168/470`, `NPC.cs:76392`), but d47 is an in-flight
//! sibling branch — the verbatim twin lands here so this branch
//! compiles alone; the merger dedupes to one shared arm (the keep-one
//! pattern, the d49 scorpion precedent), noted in ANALYSIS.

// The `npc.rs::find_frame` wiring is a later commit (the kinds are not
// in `NpcKind` yet) — the `npc_findframe_b.rs` precedent.
#![allow(dead_code)]

/// The extra-1 kinds, keyed by their VANILLA `NPC.type`
/// (`ID/NPCID.cs`'s `public const short` rows — the port-local
/// `NpcKind` ids do not exist for them yet).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FindFrameExtra1Kind {
    /// `NPCID.Slimer` = 121 — the `case 2` eye group's winged slime.
    Slimer,
    /// `NPCID.IlluminantBat` = 137 — the floater group.
    IlluminantBat,
    /// `NPCID.ToxicSludge` = 141 — `case 141` (the sludge hop).
    ToxicSludge,
    /// `NPCID.SantaClaus` = 142 — the humanoid town group.
    SantaClaus,
    /// `NPCID.Penguin` = 148 — the `case 148/149/168/470` group.
    Penguin,
    /// `NPCID.PenguinBlack` = 149 — the penguin group.
    PenguinBlack,
    /// `NPCID.FaceMonster` = 181 — the humanoid fighter group.
    FaceMonster,
    /// `NPCID.FloatyGross` = 182 — the floater group.
    FloatyGross,
    /// `NPCID.SnowFlinx` = 185 — the humanoid fighter group.
    SnowFlinx,
    /// `NPCID.LostGirl` = 195 — `case 195` (the ai[0] pose ladder).
    LostGirl,
    /// `NPCID.Nymph` = 196 — the humanoid fighter group.
    Nymph,
    /// `NPCID.Lihzahrd` = 198 — the humanoid fighter group.
    Lihzahrd,
}

impl FindFrameExtra1Kind {
    /// The vanilla `NPC.type` (`ID/NPCID.cs`'s const rows).
    pub fn vanilla_type(self) -> i16 {
        match self {
            Self::Slimer => 121,
            Self::IlluminantBat => 137,
            Self::ToxicSludge => 141,
            Self::SantaClaus => 142,
            Self::Penguin => 148,
            Self::PenguinBlack => 149,
            Self::FaceMonster => 181,
            Self::FloatyGross => 182,
            Self::SnowFlinx => 185,
            Self::LostGirl => 195,
            Self::Nymph => 196,
            Self::Lihzahrd => 198,
        }
    }

    /// `Main.npcFrameCount[type]` — the row counts counted entry-by-entry
    /// out of the 697-entry initializer (`Main.cs:67082-67153`; entry
    /// `type` sits on line `67084 + type / 10`, slot `type % 10`).
    pub fn frame_count(self) -> u32 {
        match self {
            // Main.cs:67096 (entries 120-129): 121 = 4.
            Self::Slimer => 4,
            // Main.cs:67097 (entries 130-139): 137 = 4.
            Self::IlluminantBat => 4,
            // Main.cs:67098 (entries 140-149): 141 = 3, 142 = 25,
            // 148 = 12, 149 = 12.
            Self::ToxicSludge => 3,
            Self::SantaClaus => 25,
            Self::Penguin => 12,
            Self::PenguinBlack => 12,
            // Main.cs:67102 (entries 180-189): 181 = 16, 182 = 5,
            // 185 = 12.
            Self::FaceMonster => 16,
            Self::FloatyGross => 5,
            Self::SnowFlinx => 12,
            // Main.cs:67103 (entries 190-199): 195 = 7, 196 = 14,
            // 198 = 16.
            Self::LostGirl => 7,
            Self::Nymph => 14,
            Self::Lihzahrd => 16,
        }
    }
}

/// The `cs_id` → [`FindFrameExtra1Kind`] mapping the
/// `npc.rs::find_frame` hook reads. Returns `None` for every id outside
/// this wave's set — so the hook is inert for all current kinds (none
/// has a `cs_id` in the set) and fires only once the roster wave adds
/// them.
pub fn find_frame_kind_for_cs_id(cs_id: u16) -> Option<FindFrameExtra1Kind> {
    Some(match cs_id {
        121 => FindFrameExtra1Kind::Slimer,
        137 => FindFrameExtra1Kind::IlluminantBat,
        141 => FindFrameExtra1Kind::ToxicSludge,
        142 => FindFrameExtra1Kind::SantaClaus,
        148 => FindFrameExtra1Kind::Penguin,
        149 => FindFrameExtra1Kind::PenguinBlack,
        181 => FindFrameExtra1Kind::FaceMonster,
        182 => FindFrameExtra1Kind::FloatyGross,
        185 => FindFrameExtra1Kind::SnowFlinx,
        195 => FindFrameExtra1Kind::LostGirl,
        196 => FindFrameExtra1Kind::Nymph,
        198 => FindFrameExtra1Kind::Lihzahrd,
        _ => return None,
    })
}

/// The `NPC` fields `FindFrame` reads/writes for these arms:
/// `frame.Y / num` (the row), `frameCounter`, `spriteDirection`,
/// `rotation` (the Slimer/floater tilt) and `localAI[0]` (the Slimer's
/// one-shot latch `NPC.cs:74014`, the penguin colour bank
/// `NPC.cs:76398-76409`). The wiring step copies them to/from `Npc`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FrameState {
    /// `frame.Y / num` — the sheet row.
    pub row: u32,
    /// `NPC.frameCounter`.
    pub counter: f32,
    /// `NPC.spriteDirection` (±1).
    pub sprite_dir: i32,
    /// `NPC.rotation` — only the Slimer + the floater group write it.
    pub rotation: f32,
    /// `NPC.localAI[0]` — the Slimer latch writes it, the penguin bank
    /// reads it.
    pub local_ai0: f32,
}

/// The per-tick inputs the arms read: `velocity`, `direction` and the
/// `ai` slots (the humanoid arm's future town pose zones — not read
/// today, the zones are a documented skip).
#[derive(Clone, Copy, Debug)]
pub struct FrameInput {
    pub vx: f32,
    pub vy: f32,
    /// `Entity.direction` (±1; 0 leaves `spriteDirection` alone in the
    /// arms that gate on `direction == 1` / `== -1`).
    pub direction: i32,
    pub ai: [f32; 4],
}

impl Default for FrameInput {
    fn default() -> Self {
        Self { vx: 0.0, vy: 0.0, direction: 1, ai: [0.0; 4] }
    }
}

/// `NPC.FindFrame` (`NPC.cs:66684`) for the extra-1 kinds — one tick.
pub fn find_frame_extra1(kind: FindFrameExtra1Kind, s: &mut FrameState, i: &FrameInput) {
    match kind {
        // `case 121` in the eye group (`NPC.cs:73994-74059`, shared
        // with 2/23/169/190-194/317/318/660): the one-shot
        // `localAI[0] == 0` latch pins row 2 and sets the flag
        // (`:74014-74018` — 660 shares it, not this wave's). 121 is in
        // NEITHER flip list's first arm (`:74019` carries
        // 2/190-194/317/318), so the `else if` (`:74032`, the 2/121/660
        // arm) runs: the velocity-sign flip (0 leaves it) + the
        // `vx * 0.1` tilt. The 8-tick strip (`num142` = 8 — the 6 is
        // type 660's, `:74046-74049`) walks the full 4 rows. The type-23
        // dust (`:74007-74013`) is another member's, skipped.
        FindFrameExtra1Kind::Slimer => {
            if s.local_ai0 == 0.0 {
                s.local_ai0 = 1.0;
                s.row = 2;
            }
            if i.vx > 0.0 {
                s.sprite_dir = 1;
            }
            if i.vx < 0.0 {
                s.sprite_dir = -1;
            }
            s.rotation = i.vx * 0.1;
            s.counter += 1.0;
            if s.counter >= 8.0 {
                s.row += 1;
                s.counter = 0.0;
            }
            if s.row >= 4 {
                s.row = 0;
            }
        }
        // `case 137` in the floater group (`NPC.cs:74823-74884`, shared
        // with 48/49/51/60/82/93/182/210/211/253/316/634/662/664): the
        // velocity-sign flip (0 leaves it), the `vx * 0.1` tilt (the
        // 210/211 0.2 + double-increment override `:74859-74863` is
        // other members'), the 6-tick strip (`num291` = 6 — the 5 is
        // type 48's, `:74871-74874`) over the FULL 4 rows (`num292` —
        // the `--` list 49/51/60/634 `:74867-74870` does not carry 137).
        // The group's dust spawns (60's flame `:74839-74843`, 634's
        // `Next(10)`-gated `:74844-74849`) are other members' — no RNG
        // for 137.
        FindFrameExtra1Kind::IlluminantBat => {
            if i.vx > 0.0 {
                s.sprite_dir = 1;
            }
            if i.vx < 0.0 {
                s.sprite_dir = -1;
            }
            s.rotation = i.vx * 0.1;
            s.counter += 1.0;
            if s.counter >= 6.0 {
                s.row += 1;
                s.counter = 0.0;
            }
            if s.row >= 4 {
                s.row = 0;
            }
        }
        // `case 141` (`NPC.cs:72977-72994`): the ToxicSludge faces by
        // `direction` UNCONDITIONALLY (0 included, `:72978`); the air
        // pins row 2 and breaks BEFORE the counter (`:72979-72983` —
        // the counter holds); the ground steps every 8 ticks and wraps
        // past row 1 back to 0 (`:72990-72993` — the `> num` check, so
        // row 2 is air-only on the 3-row sheet).
        FindFrameExtra1Kind::ToxicSludge => {
            s.sprite_dir = i.direction;
            if i.vy != 0.0 {
                s.row = 2;
                return;
            }
            s.counter += 1.0;
            if s.counter >= 8.0 {
                s.row += 1;
                s.counter = 0.0;
            }
            if s.row > 1 {
                s.row = 0;
            }
        }
        // The humanoid-group case list (`NPC.cs:68568` carries 142) →
        // the shared fighter/town walk (`NPC.cs:68647-69655`). Santa is
        // `townNPC` (`SetDefaults` `:10793`), so `num57` =
        // `ExtraFramesCount[142]` = 9 (`NPCID.cs:10464`) — the walk
        // wraps at 25 - 9 = 16 (the attack rows 16..24 are never
        // walked). Not `IsTownSlime` (`NPCID.cs:10119`), so `num108` = 6
        // / `num109` = row 2; the idle snaps to row 0 (142 is not the
        // 140/489 idle-row-1 pair, `:69510`); the air holds the group's
        // row 1 (142 is NOT in the `:69647` pin list). The town `ai[0]`
        // pose zones (`:68662-69488`) are the documented skip.
        FindFrameExtra1Kind::SantaClaus => {
            humanoid_frame_extra1(s, i, kind.frame_count(), 9, false);
        }
        // The penguin group (`case 148/149/168/470`,
        // `NPC.cs:76392-76456`): the `localAI[0]` 2/3/4 colour bank
        // offsets EVERY row write +3/+6/+9 (`num40`, `:76397-76410`).
        // The grounded flip writes only on direction ±1; the idle
        // snaps to the bank row + zero counter (`:76421-76426`); the
        // walk accumulates `|vx| * 1` and the <6/<12/<15 bands pick
        // rows 0/1/2 (+bank), the >=15 tail zeroes the counter AND
        // writes row 2 (+bank) (`:76443-76444` — unlike the case-3
        // walker's hold quirk, the penguin tail writes the row). Both
        // air arms write row 2 (+bank) + zero counter
        // (`:76446-76455`). 168/470 are d47/later-wave kinds — the
        // verbatim twin of d47's machine (the keep-one note above).
        FindFrameExtra1Kind::Penguin | FindFrameExtra1Kind::PenguinBlack => {
            let bank: u32 = if s.local_ai0 == 2.0 {
                3
            } else if s.local_ai0 == 3.0 {
                6
            } else if s.local_ai0 == 4.0 {
                9
            } else {
                0
            };
            if i.vy == 0.0 {
                if i.direction == 1 {
                    s.sprite_dir = 1;
                }
                if i.direction == -1 {
                    s.sprite_dir = -1;
                }
                if i.vx == 0.0 {
                    s.row = bank;
                    s.counter = 0.0;
                    return;
                }
                s.counter += i.vx.abs() * 1.0;
                if s.counter < 6.0 {
                    s.row = bank;
                } else if s.counter < 12.0 {
                    s.row = 1 + bank;
                } else if s.counter < 15.0 {
                    s.row = 2 + bank;
                } else {
                    s.counter = 0.0;
                    s.row = 2 + bank;
                }
            } else {
                // `:76446-76455` — the rise and fall arms are identical.
                s.counter = 0.0;
                s.row = 2 + bank;
            }
        }
        // The humanoid-group case list (`NPC.cs:68574-68578` carries
        // 181/185/196/198) → the shared fighter walk
        // (`NPC.cs:68647-69655`). None is `isLikeATownNPC` (the
        // `townNPC = true` census over `SetDefaults` carries none of
        // the four), so `num57` = 0 and the walk wraps at the FULL
        // `npcFrameCount`; none is `IsTownSlime`, so `num108` = 6 /
        // `num109` = row 2. None is in the airborne pin list
        // (`:69647` — 201/202 are, later in this wave), so the air
        // holds the group's row 1; none is the 140/489 idle-row-1 pair.
        FindFrameExtra1Kind::FaceMonster
        | FindFrameExtra1Kind::SnowFlinx
        | FindFrameExtra1Kind::Nymph
        | FindFrameExtra1Kind::Lihzahrd => {
            humanoid_frame_extra1(s, i, kind.frame_count(), 0, false);
        }
        // `case 182` in the floater group (`NPC.cs:74823-74884`): the
        // same machine as the IlluminantBat above — the velocity-sign
        // flip, the `vx * 0.1` tilt, the 6-tick strip — over the FULL
        // 5 rows (`num292` = 5; the `--` list 49/51/60/634 does not
        // carry 182). No dust, no RNG.
        FindFrameExtra1Kind::FloatyGross => {
            if i.vx > 0.0 {
                s.sprite_dir = 1;
            }
            if i.vx < 0.0 {
                s.sprite_dir = -1;
            }
            s.rotation = i.vx * 0.1;
            s.counter += 1.0;
            if s.counter >= 6.0 {
                s.row += 1;
                s.counter = 0.0;
            }
            if s.row >= 5 {
                s.row = 0;
            }
        }
        // `case 195` (`NPC.cs:73146-73171`): the LostGirl's pose ladder
        // — `ai[0]` <3/<6/<9/<12/<15 picks rows 0/1/2/3/4, the `else`
        // row 5. No counter, no sprite/rotation write; row 6 of the
        // 7-row sheet is never written by this machine.
        FindFrameExtra1Kind::LostGirl => {
            if i.ai[0] < 3.0 {
                s.row = 0;
            } else if i.ai[0] < 6.0 {
                s.row = 1;
            } else if i.ai[0] < 9.0 {
                s.row = 2;
            } else if i.ai[0] < 12.0 {
                s.row = 3;
            } else if i.ai[0] < 15.0 {
                s.row = 4;
            } else {
                s.row = 5;
            }
        }
    }
}

/// The humanoid-group walk (`NPC.cs:68647-69655`) for the extra-1
/// members: the grounded flip (`:68649-68656` — only `direction` ±1
/// writes), the idle snap to row 0 (`:69515-69519` — none of these is
/// the 140/489 idle-row-1 quirk), the `|vx| * 2 + 1` accumulator
/// (`:69552-69553`) stepping every > 6 (`num108` = 6) over rows
/// 2..count-1-extra (`num109` = row 2, the wrap at
/// `npcFrameCount - num57`, `:69581`), and the airborne row 1 → pin-0
/// tail (`:69642-69650`). The town `ai[0]` pose zones
/// (`:68662-69488`) are a documented skip (see the header).
fn humanoid_frame_extra1(
    s: &mut FrameState,
    i: &FrameInput,
    count: u32,
    extra: u32,
    air_row0: bool,
) {
    if i.vy == 0.0 {
        if i.direction == 1 {
            s.sprite_dir = 1;
        }
        if i.direction == -1 {
            s.sprite_dir = -1;
        }
        if i.vx == 0.0 {
            // `NPC.cs:69515-69519`: the idle row 0.
            s.row = 0;
            s.counter = 0.0;
        } else {
            // `NPC.cs:69552-69553`: the accumulator.
            s.counter += i.vx.abs() * 2.0;
            s.counter += 1.0;
            // `:69572-69575`: the walk band's floor (row 2).
            if s.row < 2 {
                s.row = 2;
            }
            // `:69576-69580`: the step.
            if s.counter > 6.0 {
                s.row += 1;
                s.counter = 0.0;
            }
            // `:69581-69584`: the wrap at `npcFrameCount - num57`.
            if s.row >= count.saturating_sub(extra) {
                s.row = 2;
            }
        }
    } else {
        // `NPC.cs:69642-69650`: the group's air row 1, then the pin list.
        s.counter = 0.0;
        s.row = if air_row0 { 0 } else { 1 };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(vx: f32, vy: f32) -> FrameInput {
        FrameInput { vx, vy, ..Default::default() }
    }

    fn rows_over(kind: FindFrameExtra1Kind, i: FrameInput, ticks: usize) -> Vec<u32> {
        let mut s = FrameState::default();
        let mut out = Vec::new();
        for _ in 0..ticks {
            find_frame_extra1(kind, &mut s, &i);
            out.push(s.row);
        }
        out
    }

    /// The `Main.npcFrameCount` rows, counted entry-by-entry out of
    /// the `Main.cs:67082` initializer and cross-checked against the
    /// port's `NPC_FRAME_COUNT` table.
    #[test]
    fn the_extra1_frame_counts_match_the_main_initializer() {
        let table = [
            (FindFrameExtra1Kind::Slimer, 121, 4),
            (FindFrameExtra1Kind::IlluminantBat, 137, 4),
            (FindFrameExtra1Kind::ToxicSludge, 141, 3),
            (FindFrameExtra1Kind::SantaClaus, 142, 25),
            (FindFrameExtra1Kind::Penguin, 148, 12),
            (FindFrameExtra1Kind::PenguinBlack, 149, 12),
            (FindFrameExtra1Kind::FaceMonster, 181, 16),
            (FindFrameExtra1Kind::FloatyGross, 182, 5),
            (FindFrameExtra1Kind::SnowFlinx, 185, 12),
            (FindFrameExtra1Kind::LostGirl, 195, 7),
            (FindFrameExtra1Kind::Nymph, 196, 14),
            (FindFrameExtra1Kind::Lihzahrd, 198, 16),
        ];
        for (kind, vanilla, count) in table {
            assert_eq!(kind.vanilla_type(), vanilla, "{kind:?} id");
            assert_eq!(kind.frame_count(), count, "{kind:?} npcFrameCount");
            assert_eq!(
                crate::render::entities::npc_rows(vanilla),
                count,
                "{kind:?} vs the port's NPC_FRAME_COUNT"
            );
            assert_eq!(find_frame_kind_for_cs_id(vanilla), Some(kind), "{kind:?} cs_id map");
        }
    }

    /// The `cs_id` hook maps exactly the wave's ids and nothing else —
    /// the inertness seam (no current kind's `cs_id` is in the set).
    #[test]
    fn the_cs_id_hook_maps_only_the_extra1_ids() {
        for id in 0u16..=697 {
            let mapped = find_frame_kind_for_cs_id(id);
            let in_set = matches!(
                id,
                121 | 137 | 141 | 142 | 148 | 149 | 181 | 182 | 185 | 195 | 196 | 198
            );
            assert_eq!(mapped.is_some(), in_set, "cs_id {id}");
        }
    }

    /// The Slimer (`NPC.cs:73994-74059`): the one-shot latch pins row 2
    /// on the first tick, then the 8-tick strip walks rows 2/3/0/1;
    /// the flip + tilt ride the velocity sign.
    #[test]
    fn the_slimer_latches_row_two_then_walks_the_eight_tick_strip() {
        let mut s = FrameState::default();
        let i = input(2.0, 0.0);
        let mut rows = Vec::new();
        for _ in 0..16 {
            find_frame_extra1(FindFrameExtra1Kind::Slimer, &mut s, &i);
            rows.push(s.row);
        }
        // Tick 1 latches row 2; ticks 2-7 hold it; tick 8 steps to 3;
        // ticks 9-15 hold 3; tick 16 steps to 4 → wraps 0.
        assert_eq!(
            rows,
            vec![2, 2, 2, 2, 2, 2, 2, 3, 3, 3, 3, 3, 3, 3, 3, 0]
        );
        assert_eq!(s.local_ai0, 1.0, "the latch flag stays set");
        assert_eq!(s.sprite_dir, 1, "the velocity-sign flip");
        assert_eq!(s.rotation, 0.2, "the vx * 0.1 tilt");
        // The latch does not re-fire (localAI[0] == 1 now).
        let before = s.row;
        find_frame_extra1(FindFrameExtra1Kind::Slimer, &mut s, &i);
        assert_ne!(s.row, 2, "no second latch pin (row {before} + tick)");
        // vx == 0 leaves the flip alone but still writes rotation 0.
        let mut t = FrameState { sprite_dir: -1, ..Default::default() };
        find_frame_extra1(FindFrameExtra1Kind::Slimer, &mut t, &input(0.0, 0.0));
        assert_eq!(t.sprite_dir, -1, "0 leaves the flip");
        assert_eq!(t.rotation, 0.0);
    }

    /// The IlluminantBat (`NPC.cs:74823-74884`): the 6-tick strip over
    /// the full 4 rows, the velocity-sign flip + 0.1 tilt.
    #[test]
    fn the_illuminant_bat_flaps_four_rows_every_six_ticks() {
        let mut s = FrameState::default();
        let i = input(-2.0, 0.0);
        find_frame_extra1(FindFrameExtra1Kind::IlluminantBat, &mut s, &i);
        assert_eq!(s.sprite_dir, -1, "flip");
        assert_eq!(s.rotation, -0.2, "tilt");
        assert_eq!(
            rows_over(FindFrameExtra1Kind::IlluminantBat, i, 24),
            vec![
                0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 3, 3, 3, 3, 3, 3, 0
            ]
        );
    }

    /// The ToxicSludge (`NPC.cs:72977-72994`): the ground 0/1 cycle on
    /// the 8-tick clock (the `> num` wrap keeps row 2 air-only), the
    /// air pins row 2 WITHOUT ticking the counter, the flip is
    /// unconditional (direction 0 writes 0).
    #[test]
    fn the_toxic_sludge_hops_two_ground_rows_and_pins_the_air_row() {
        assert_eq!(
            rows_over(FindFrameExtra1Kind::ToxicSludge, input(1.0, 0.0), 16),
            vec![0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 0]
        );
        let mut s = FrameState { row: 1, counter: 5.0, ..Default::default() };
        find_frame_extra1(FindFrameExtra1Kind::ToxicSludge, &mut s, &input(0.0, -3.0));
        assert_eq!((s.row, s.counter), (2, 5.0), "the air break holds the counter");
        let mut t = FrameState { sprite_dir: 1, ..Default::default() };
        let mut i = input(0.0, 0.0);
        i.direction = 0;
        find_frame_extra1(FindFrameExtra1Kind::ToxicSludge, &mut t, &i);
        assert_eq!(t.sprite_dir, 0, "the unconditional direction copy");
    }

    /// Santa (`NPC.cs:68568` → `:68647-69655`): the walk wraps at
    /// 25 - 9 = 16 (ExtraFramesCount[142] = 9, `NPCID.cs:10464`), the
    /// idle snaps to row 0, the air holds row 1 (not in the pin list).
    #[test]
    fn santa_walks_rows_two_to_fifteen_and_wraps_at_sixteen() {
        // vx 1 → +3 a tick; the step fires every 3rd tick (> 6).
        let rows = rows_over(FindFrameExtra1Kind::SantaClaus, input(1.0, 0.0), 45);
        assert_eq!(rows[0], 2, "the first walk tick floors to row 2");
        assert_eq!(rows[2], 3, "the third tick steps");
        // Rows 2..15 = 14 rows; row r (r >= 2) lands on tick 3(r-2),
        // so row 15 lands on tick 39 and holds through tick 41; tick
        // 42 steps to 16 → the wrap at npcFrameCount - num57 = 16.
        assert_eq!(rows[38], 15);
        assert_eq!(rows[40], 15);
        assert_eq!(rows[41], 2, "the wrap at npcFrameCount - num57 = 16");
        let mut s = FrameState { row: 9, counter: 4.0, ..Default::default() };
        find_frame_extra1(FindFrameExtra1Kind::SantaClaus, &mut s, &input(0.0, 0.0));
        assert_eq!((s.row, s.counter), (0, 0.0), "the idle snap");
        find_frame_extra1(FindFrameExtra1Kind::SantaClaus, &mut s, &input(1.0, -2.0));
        assert_eq!((s.row, s.counter), (1, 0.0), "the air row 1");
    }

    /// The penguins (`NPC.cs:76392-76456`): the walk bands <6/<12/<15
    /// over rows 0/1/2, the >=15 tail zeroes the counter AND writes
    /// row 2, the idle snaps to row 0, both air arms pin row 2.
    #[test]
    fn the_penguins_walk_three_bands_and_hold_the_tail_row() {
        for kind in [FindFrameExtra1Kind::Penguin, FindFrameExtra1Kind::PenguinBlack] {
            // vx 1 → +1 a tick: rows 0 (c1-5), 1 (c6-11), 2 (c12-14),
            // then c15 → tail: counter 0, row 2 again.
            assert_eq!(
                rows_over(kind, input(1.0, 0.0), 15),
                vec![0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2],
                "{kind:?}"
            );
            let mut s = FrameState { row: 1, counter: 14.0, ..Default::default() };
            find_frame_extra1(kind, &mut s, &input(1.0, 0.0));
            assert_eq!((s.row, s.counter), (2, 0.0), "{kind:?} the tail write");
            find_frame_extra1(kind, &mut s, &input(0.0, 0.0));
            assert_eq!((s.row, s.counter), (0, 0.0), "{kind:?} the idle snap");
            find_frame_extra1(kind, &mut s, &input(0.0, -1.0));
            assert_eq!((s.row, s.counter), (2, 0.0), "{kind:?} the rise pin");
            find_frame_extra1(kind, &mut s, &input(0.0, 1.0));
            assert_eq!((s.row, s.counter), (2, 0.0), "{kind:?} the fall pin");
        }
    }

    /// The penguin colour banks (`NPC.cs:76397-76410`): localAI[0]
    /// 2/3/4 offset every row write +3/+6/+9.
    #[test]
    fn the_penguin_banks_offset_every_row_write() {
        for (flag, bank) in [(2.0f32, 3u32), (3.0, 6), (4.0, 9)] {
            let mut s = FrameState { local_ai0: flag, ..Default::default() };
            let i = input(0.0, 0.0);
            find_frame_extra1(FindFrameExtra1Kind::Penguin, &mut s, &i);
            assert_eq!(s.row, bank, "bank {flag} idle");
            find_frame_extra1(FindFrameExtra1Kind::Penguin, &mut s, &input(0.0, 2.0));
            assert_eq!(s.row, 2 + bank, "bank {flag} air");
            // A non-bank flag (1) keeps offset 0.
            let mut t = FrameState { local_ai0: 1.0, ..Default::default() };
            find_frame_extra1(FindFrameExtra1Kind::Penguin, &mut t, &i);
            assert_eq!(t.row, 0);
        }
    }

    /// The fighter group (`NPC.cs:68574-68578` → `:68647-69655`):
    /// num57 = 0 so the walk wraps at the FULL count; the air holds
    /// row 1 (none of the four is in the `:69647` pin list); the idle
    /// snaps to row 0.
    #[test]
    fn the_fighter_group_wraps_at_the_full_count_and_holds_air_row_one() {
        for (kind, count) in [
            (FindFrameExtra1Kind::FaceMonster, 16u32),
            (FindFrameExtra1Kind::SnowFlinx, 12),
            (FindFrameExtra1Kind::Nymph, 14),
            (FindFrameExtra1Kind::Lihzahrd, 16),
        ] {
            // vx 1 → +3 a tick; row r (r >= 2) lands on tick 3(r-2);
            // the last walk row count-1 holds 3 ticks, then the wrap.
            let rows = rows_over(kind, input(1.0, 0.0), 3 * (count as usize - 2) + 1);
            assert_eq!(rows[0], 2, "{kind:?} floor");
            assert_eq!(rows[3 * (count as usize - 3)], count - 1, "{kind:?} last row");
            assert_eq!(rows[3 * (count as usize - 2)], 2, "{kind:?} full-count wrap");
            let mut s = FrameState { row: 7, counter: 4.0, ..Default::default() };
            find_frame_extra1(kind, &mut s, &input(1.0, -2.0));
            assert_eq!((s.row, s.counter), (1, 0.0), "{kind:?} air row 1");
            find_frame_extra1(kind, &mut s, &input(0.0, 0.0));
            assert_eq!((s.row, s.counter), (0, 0.0), "{kind:?} idle row 0");
        }
    }

    /// The FloatyGross (`NPC.cs:74823-74884`): the floater machine over
    /// the FULL 5 rows (the `--` list does not carry 182).
    #[test]
    fn the_floaty_gross_flaps_five_rows_every_six_ticks() {
        let mut s = FrameState::default();
        let i = input(3.0, 0.0);
        find_frame_extra1(FindFrameExtra1Kind::FloatyGross, &mut s, &i);
        assert_eq!(s.sprite_dir, 1);
        assert_eq!(s.rotation, 0.3, "the vx * 0.1 tilt");
        assert_eq!(
            rows_over(FindFrameExtra1Kind::FloatyGross, i, 29),
            vec![
                0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 3, 3, 3, 3, 3, 3, 4,
                4, 4, 4, 4, 4
            ]
        );
    }

    /// The LostGirl (`NPC.cs:73146-73171`): the ai[0] <3/<6/<9/<12/<15
    /// ladder over rows 0..5; no counter/sprite/rotation write.
    #[test]
    fn the_lost_girl_pose_ladder_reads_ai_zero() {
        for (ai0, row) in [
            (0.0f32, 0u32),
            (2.9, 0),
            (3.0, 1),
            (5.9, 1),
            (6.0, 2),
            (9.0, 3),
            (12.0, 4),
            (14.9, 4),
            (15.0, 5),
            (100.0, 5),
        ] {
            let mut s = FrameState { counter: 9.0, sprite_dir: -1, rotation: 0.5, ..Default::default() };
            let mut i = input(1.0, 1.0);
            i.ai[0] = ai0;
            find_frame_extra1(FindFrameExtra1Kind::LostGirl, &mut s, &i);
            assert_eq!(s.row, row, "ai[0] {ai0}");
            assert_eq!(s.counter, 9.0, "the ladder never touches the counter");
            assert_eq!(s.sprite_dir, -1, "no sprite write");
            assert_eq!(s.rotation, 0.5, "no rotation write");
        }
    }
}
