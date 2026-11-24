//! W4-d49 FindFrame coverage wave D — the SECOND 12 missing HIGH-id
//! kinds of `NPC.FindFrame` (`NPC.cs:66684-77370`) in the 350-697 band,
//! landed 6 kinds per commit. Ascending census against the port's
//! `npc.rs::find_frame` match: 350-352 ride the frost arms, 353 the
//! humanoid town group, 355-366 are wave C (`npc_findframe_c.rs`,
//! `task/W4-d48-findframe-high-1`), 368/369 the town group, 370-373 the
//! fishron family, 376 the SleepingAngler arm — so the second gap is
//! exactly **367, 374, 375, 377, 378, 379, 380, 430, 431, 432, 433,
//! 434**. Commit 1 carried the first 6 (367/374/375/377/378/379); this
//! adds 380 + the five armed zombies 430-434. The next gap (435
//! ArmedZombieTwiggy onward) rides a later wave.
//!
//! Shape: the kinds are NOT in `NpcKind` yet (the roster wave
//! `task/W4-d39-npc-kinds-351-420` owns them), so — the
//! `npc_findframe_b.rs` (S1-D-3 wave B) precedent — the frame machines
//! land first as pure functions over a [`FrameState`]/[`FrameInput`]
//! pair and the `npc.rs::find_frame` wiring is a separate step: when a
//! kind joins `NpcKind`, its arm calls [`find_frame_high2`] with the
//! NPC's fields and copies `row`/`counter`/`sprite_dir`/`rotation`/
//! `local_ai0` back. No RNG is touched (none of these arms draws
//! `Main.rand`), so the wiring cannot move a stream.
//!
//! Row convention: the port tracks frame ROW indices, not pixels — the
//! C#'s `num` (the sheet height / `Main.npcFrameCount[type]`,
//! `NPC.cs:66694`) cancels out of every comparison, exactly as in
//! `npc.rs::find_frame`'s existing arms. The per-kind row counts are
//! `Main.npcFrameCount` (`Main.cs:67082`'s 697-entry initializer),
//! counted entry-by-entry in [`FindFrameKind::frame_count`] and
//! cross-checked against the port's `NPC_FRAME_COUNT` in the tests.
//!
//! Deliberate skips (recorded in docs/ANALYSIS.md "W4-d49 FindFrame
//! wave D"): the `IsABestiaryIconDummy` branch of the grasshopper arm
//! (`NPC.cs:70960-70972` — the port has no bestiary dummy), and the
//! case-group siblings outside this wave's 12 (446 GoldGrasshopper,
//! 591 ArmedZombieCenx2 — later waves). The scorpion arm is the SAME
//! machine as wave C's 366 (`case 366 : case 367`, `NPC.cs:76820`); the
//! verbatim twin landed here so this branch compiled alone, and the
//! integration merge deduped it to wave C's shared
//! `npc_findframe_c::scorpion_scuttle` (the keep-one pattern), noted in
//! ANALYSIS.

// The `npc.rs::find_frame` wiring is a later wave (the kinds are not
// in `NpcKind` yet) — the `npc_findframe_b.rs` precedent.
#![allow(dead_code)]

/// The 12 wave-D kinds, keyed by their VANILLA `NPC.type`
/// (`ID/NPCID.cs`'s `public const short` rows — the port-local
/// `NpcKind` ids do not exist for them yet).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FindFrameKind {
    /// `NPCID.Scorpion` = 367 — the `case 366/367` scuttle arm.
    Scorpion,
    /// `NPCID.TruffleWorm` = 374 — `case 374` (the bait crawler).
    TruffleWorm,
    /// `NPCID.TruffleWormDigger` = 375 — `case 375` (the digging form).
    TruffleWormDigger,
    /// `NPCID.Grasshopper` = 377 — `case 377/446`.
    Grasshopper,
    /// `NPCID.ChatteringTeethBomb` = 378 — `case 378` (the two-bank
    /// wind-up teeth).
    ChatteringTeethBomb,
    /// `NPCID.CultistArcherBlue` = 379 — `case 379/380`.
    CultistArcherBlue,
    /// `NPCID.CultistArcherWhite` = 380 — `case 379/380`.
    CultistArcherWhite,
    /// `NPCID.ArmedZombie` = 430 — the `case 430..436/591` group.
    ArmedZombie,
    /// `NPCID.ArmedZombieEskimo` = 431 — the group.
    ArmedZombieEskimo,
    /// `NPCID.ArmedZombiePincussion` = 432 — the group (the C# const
    /// spells it "Pincussion", `NPCID.cs`).
    ArmedZombiePincussion,
    /// `NPCID.ArmedZombieSlimed` = 433 — the group.
    ArmedZombieSlimed,
    /// `NPCID.ArmedZombieSwamp` = 434 — the group.
    ArmedZombieSwamp,
}

impl FindFrameKind {
    /// The vanilla `NPC.type` (`ID/NPCID.cs`'s const rows).
    pub fn vanilla_type(self) -> i16 {
        match self {
            Self::Scorpion => 367,
            Self::TruffleWorm => 374,
            Self::TruffleWormDigger => 375,
            Self::Grasshopper => 377,
            Self::ChatteringTeethBomb => 378,
            Self::CultistArcherBlue => 379,
            Self::CultistArcherWhite => 380,
            Self::ArmedZombie => 430,
            Self::ArmedZombieEskimo => 431,
            Self::ArmedZombiePincussion => 432,
            Self::ArmedZombieSlimed => 433,
            Self::ArmedZombieSwamp => 434,
        }
    }

    /// `Main.npcFrameCount[type]` — the row counts counted entry-by-entry
    /// out of the 697-entry initializer (`Main.cs:67082-67153`; entry
    /// `type` sits on line `67084 + type / 10`, slot `type % 10`).
    pub fn frame_count(self) -> u32 {
        match self {
            // Main.cs:67120 (entries 360-369): 367 = 4.
            Self::Scorpion => 4,
            // Main.cs:67121 (entries 370-379): 374 = 4, 375 = 4,
            // 377 = 2, 378 = 4, 379 = 12.
            Self::TruffleWorm => 4,
            Self::TruffleWormDigger => 4,
            Self::Grasshopper => 2,
            Self::ChatteringTeethBomb => 4,
            Self::CultistArcherBlue => 12,
            // Main.cs:67122 (entries 380-389): 380 = 12.
            Self::CultistArcherWhite => 12,
            // Main.cs:67127 (entries 430-439): 430-436 = 7 each.
            Self::ArmedZombie
            | Self::ArmedZombieEskimo
            | Self::ArmedZombiePincussion
            | Self::ArmedZombieSlimed
            | Self::ArmedZombieSwamp => 7,
        }
    }
}

/// The `NPC` fields `FindFrame` reads/writes for these arms:
/// `frame.Y / num` (the row), `frameCounter`, `spriteDirection`,
/// `rotation` (the truffle worm's spin, `NPC.cs:71125`/`:71144`) and
/// `localAI[0]` (the truffle worm's `-2f` pin, `NPC.cs:71122`). The
/// wiring step copies them to/from `Npc`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FrameState {
    /// `frame.Y / num` — the sheet row.
    pub row: u32,
    /// `NPC.frameCounter`.
    pub counter: f32,
    /// `NPC.spriteDirection` (±1).
    pub sprite_dir: i32,
    /// `NPC.rotation`.
    pub rotation: f32,
    /// `NPC.localAI[0]` — only the truffle worm (374) writes it.
    pub local_ai0: f32,
}

/// The per-tick inputs the arms read: `velocity`, `direction` and the
/// archers'/armed zombies' `ai[2]` (the bow-draw frame / the swing
/// band). None of these kinds sets `aiAction` (the `num2` motion class
/// of `NPC.cs:66696-66704` is unused by every wave-D arm).
#[derive(Clone, Copy, Debug)]
pub struct FrameInput {
    pub vx: f32,
    pub vy: f32,
    /// `Entity.direction` (±1; 0 leaves `spriteDirection` alone in the
    /// arms that gate on `direction == 1` / `== -1`).
    pub direction: i32,
    /// `NPC.ai[2]` — the cultist archers' draw frame (1..5, `> 0` rides
    /// rows 2..6, `NPC.cs:71590-71596`) and the armed zombies' swing
    /// band (`< 5/10/15` picks rows 3/4/5/6, `NPC.cs:70887-70905`).
    pub ai2: f32,
}

impl Default for FrameInput {
    fn default() -> Self {
        Self { vx: 0.0, vy: 0.0, direction: 1, ai2: 0.0 }
    }
}

/// `NPC.FindFrame` (`NPC.cs:66684`) for the wave-D kinds — one tick.
pub fn find_frame_high2(kind: FindFrameKind, s: &mut FrameState, i: &FrameInput) {
    let count = kind.frame_count();
    match kind {
        // `case 366 : case 367` (`NPC.cs:76820-76850`): the scuttle —
        // the merge-side keep-one dedupe DONE: this arm delegates to
        // wave C's shared `scorpion_scuttle` machine (the verbatim
        // twin is gone; `|vx| * 1` == `|vx|` bit-exact).
        FindFrameKind::Scorpion => {
            crate::npc_findframe_c::scorpion_scuttle(s, i.vx, i.vy, i.direction, count);
        }
        // `case 374` (`NPC.cs:71121-71155`, the aiStyle-66 family's
        // FindFrame sibling — 448 GoldWorm rides the 357 arm instead):
        // the `localAI[0] = -2` pin every tick; grounded the rotation
        // zeroes, the idle snaps row 0, the walk steps every > 6 ticks
        // wrapping `>= num*4` back to row 1 (rows 1/2/3 cycle); air
        // accumulates the `direction * 0.1` spin and steps every > 3.
        FindFrameKind::TruffleWorm => {
            s.local_ai0 = -2.0;
            if i.vy == 0.0 {
                s.rotation = 0.0;
                if i.vx == 0.0 {
                    s.row = 0;
                    s.counter = 0.0;
                } else {
                    s.counter += 1.0;
                    if s.counter > 6.0 {
                        s.counter = 0.0;
                        s.row += 1;
                        if s.row >= 4 {
                            s.row = 1;
                        }
                    }
                }
            } else {
                s.rotation += i.direction as f32 * 0.1;
                s.counter += 1.0;
                if s.counter > 3.0 {
                    s.counter = 0.0;
                    s.row += 1;
                    if s.row >= 4 {
                        s.row = 1;
                    }
                }
            }
        }
        // `case 375` (`NPC.cs:71109-71120`): the digger's unconditional
        // > 4-tick step over rows 0..3 (wrap at `num*4` to 0).
        FindFrameKind::TruffleWormDigger => {
            s.counter += 1.0;
            if s.counter > 4.0 {
                s.counter = 0.0;
                s.row += 1;
                if s.row >= 4 {
                    s.row = 0;
                }
            }
        }
        // `case 377 : case 446` (`NPC.cs:70958-70982`): the airborne
        // hop rides row 1, the ground pins row 0, the flip is
        // unconditional. (The `IsABestiaryIconDummy` 2-row blink
        // `:70960-70972` is skipped — the port has no bestiary dummy.)
        FindFrameKind::Grasshopper => {
            if i.vy != 0.0 {
                s.row = 1;
            } else {
                s.row = 0;
            }
            s.sprite_dir = i.direction;
        }
        // `case 378` (`NPC.cs:70983-71006`): the teeth's two 2-row
        // banks — airborne shifts +2 rows ONCE (`frame.Y < num*2`
        // gate), landing shifts −2 (`> num*2`, so row 2 stays in the
        // upper bank for one step); then the >= 6-tick stride wraps
        // within the bank (`>= num*4` → 2 air, `>= num*2` → 0 ground).
        FindFrameKind::ChatteringTeethBomb => {
            if i.vy != 0.0 && s.row < 2 {
                s.row += 2;
            }
            if i.vy == 0.0 && s.row > 2 {
                s.row -= 2;
            }
            s.counter += 1.0;
            if s.counter >= 6.0 {
                s.counter = 0.0;
                s.row += 1;
                if i.vy != 0.0 && s.row >= 4 {
                    s.row = 2;
                }
                if i.vy == 0.0 && s.row >= 2 {
                    s.row = 0;
                }
            }
        }
        // `case 379 : case 380` (`NPC.cs:71578-71622`): grounded, the
        // two one-sided direction ifs flip the sprite; `ai[2] > 0`
        // pins the bow-draw row `(int)ai[2] - 1 + 2` (2..6), re-copies
        // `spriteDirection = direction` and zeroes the counter; the
        // idle snaps row 0 WITHOUT zeroing the counter; the walk floor
        // is row 7, the `|vx| * 2` accumulator steps at > 6 and wraps
        // at `npcFrameCount` (12) back to 7. Air pins row 1, counter 0.
        FindFrameKind::CultistArcherBlue | FindFrameKind::CultistArcherWhite => {
            if i.vy == 0.0 {
                if i.direction == 1 {
                    s.sprite_dir = 1;
                }
                if i.direction == -1 {
                    s.sprite_dir = -1;
                }
                if i.ai2 > 0.0 {
                    s.sprite_dir = i.direction;
                    s.row = (i.ai2 as i32 - 1 + 2).max(0) as u32;
                    s.counter = 0.0;
                } else if i.vx == 0.0 {
                    s.row = 0;
                } else {
                    if s.row < 7 {
                        s.row = 7;
                    }
                    s.counter += i.vx.abs() * 2.0;
                    if s.counter > 6.0 {
                        s.row += 1;
                        s.counter = 0.0;
                    }
                    if s.row >= count {
                        s.row = 7;
                    }
                }
            } else {
                s.counter = 0.0;
                s.row = 1;
            }
        }
        // `case 430..436 : case 591` (`NPC.cs:70879-70951`): the armed
        // zombies' swing band first — `ai[2] > 0` picks rows 3/4/5/6
        // at the < 5/10/15 ladder and breaks BEFORE any flip. Then the
        // grounded one-sided flips; the air OR counter-motion row-2
        // pin (`direction == -1 && vx > 0` / `== 1 && vx < 0`); the
        // idle row 0; then the wave-A zombie stride — the `|vx|`
        // accumulator walks the 8/16/24/32 bands over rows 0/1/2/1 and
        // at >= 32 only ZEROES the counter, holding the row (`:70947-
        // 70950`, the reset-tick row-hold quirk).
        FindFrameKind::ArmedZombie
        | FindFrameKind::ArmedZombieEskimo
        | FindFrameKind::ArmedZombiePincussion
        | FindFrameKind::ArmedZombieSlimed
        | FindFrameKind::ArmedZombieSwamp => {
            if i.ai2 > 0.0 {
                s.row = if i.ai2 < 5.0 {
                    3
                } else if i.ai2 < 10.0 {
                    4
                } else if i.ai2 < 15.0 {
                    5
                } else {
                    6
                };
                return;
            }
            if i.vy == 0.0 {
                if i.direction == 1 {
                    s.sprite_dir = 1;
                }
                if i.direction == -1 {
                    s.sprite_dir = -1;
                }
            }
            if i.vy != 0.0
                || (i.direction == -1 && i.vx > 0.0)
                || (i.direction == 1 && i.vx < 0.0)
            {
                s.counter = 0.0;
                s.row = 2;
                return;
            }
            if i.vx == 0.0 {
                s.counter = 0.0;
                s.row = 0;
                return;
            }
            s.counter += i.vx.abs();
            if s.counter < 8.0 {
                s.row = 0;
            } else if s.counter < 16.0 {
                s.row = 1;
            } else if s.counter < 24.0 {
                s.row = 2;
            } else if s.counter < 32.0 {
                s.row = 1;
            } else {
                s.counter = 0.0;
            }
        }
    }
}

/// The wiring seam: the vanilla `NPC.type` → wave-D kind map (the
/// `npc_findframe_a::find_frame_kind` shape, keyed on the vanilla id
/// because the `NpcKind` rows do not exist yet — the roster wave
/// `task/W4-d39-npc-kinds-351-420` owns them). Returns `None` for every
/// id outside this wave's 12 kinds / 13 ids (the W4-g7 wire-2 keep-one
/// folds the d48 ScorpionBlack 366 onto the shared 366/367 C# arm).
pub fn find_frame_kind_high2(vanilla_type: i16) -> Option<FindFrameKind> {
    Some(match vanilla_type {
        // W4-g7 wire-2 keep-one: the C# groups `case 366 : case 367`
        // (`NPC.cs:76820-76850`) — the d48 wave-C ScorpionBlack twin
        // delegates here, so this ONE machine serves both ids.
        366 | 367 => FindFrameKind::Scorpion,
        374 => FindFrameKind::TruffleWorm,
        375 => FindFrameKind::TruffleWormDigger,
        377 => FindFrameKind::Grasshopper,
        378 => FindFrameKind::ChatteringTeethBomb,
        379 => FindFrameKind::CultistArcherBlue,
        380 => FindFrameKind::CultistArcherWhite,
        430 => FindFrameKind::ArmedZombie,
        431 => FindFrameKind::ArmedZombieEskimo,
        432 => FindFrameKind::ArmedZombiePincussion,
        433 => FindFrameKind::ArmedZombieSlimed,
        434 => FindFrameKind::ArmedZombieSwamp,
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(vx: f32, vy: f32) -> FrameInput {
        FrameInput { vx, vy, ..Default::default() }
    }

    fn rows_over(kind: FindFrameKind, i: FrameInput, ticks: usize) -> Vec<u32> {
        let mut s = FrameState::default();
        let mut out = Vec::new();
        for _ in 0..ticks {
            find_frame_high2(kind, &mut s, &i);
            out.push(s.row);
        }
        out
    }

    /// The 12 `Main.npcFrameCount` rows, counted entry-by-entry out of
    /// the `Main.cs:67082` initializer (the line/slot arithmetic in
    /// `frame_count`'s doc) and cross-checked against the port's
    /// verbatim `NPC_FRAME_COUNT` table.
    #[test]
    fn the_wave_d_frame_counts_match_the_main_initializer() {
        let table = [
            (FindFrameKind::Scorpion, 367, 4),
            (FindFrameKind::TruffleWorm, 374, 4),
            (FindFrameKind::TruffleWormDigger, 375, 4),
            (FindFrameKind::Grasshopper, 377, 2),
            (FindFrameKind::ChatteringTeethBomb, 378, 4),
            (FindFrameKind::CultistArcherBlue, 379, 12),
            (FindFrameKind::CultistArcherWhite, 380, 12),
            (FindFrameKind::ArmedZombie, 430, 7),
            (FindFrameKind::ArmedZombieEskimo, 431, 7),
            (FindFrameKind::ArmedZombiePincussion, 432, 7),
            (FindFrameKind::ArmedZombieSlimed, 433, 7),
            (FindFrameKind::ArmedZombieSwamp, 434, 7),
        ];
        for (kind, vanilla, count) in table {
            assert_eq!(kind.vanilla_type(), vanilla, "{kind:?} id");
            assert_eq!(kind.frame_count(), count, "{kind:?} npcFrameCount");
            assert_eq!(
                crate::render::entities::npc_rows(vanilla as u16),
                count,
                "{kind:?} NPC_FRAME_COUNT cross-check"
            );
            assert_eq!(find_frame_kind_high2(vanilla), Some(kind), "{kind:?} seam");
        }
        // W4-g7 wire-2 keep-one: the C# `case 366 : case 367` group —
        // the d48 ScorpionBlack delegates to this Scorpion machine.
        assert_eq!(find_frame_kind_high2(366), Some(FindFrameKind::Scorpion));
        // The band edges stay outside this wave (d48 owns 355-365,
        // the next wave owns 435+).
        assert_eq!(find_frame_kind_high2(365), None);
        assert_eq!(find_frame_kind_high2(376), None);
        assert_eq!(find_frame_kind_high2(435), None);
        assert_eq!(find_frame_kind_high2(591), None);
    }

    /// `NPC.cs:76833-76849`: at vx = 1 the accumulator gains 2 a tick
    /// (|vx|*1 + 1), so the row steps every 4th tick (counter 2/4/6/8)
    /// and wraps 0..3 back to 0 — the same |vx|+1 scuttle as wave C.
    #[test]
    fn scorpion_scuttles_at_abs_vx_plus_one() {
        let rows = rows_over(FindFrameKind::Scorpion, input(1.0, 0.0), 20);
        assert_eq!(&rows[..8], &[0, 0, 0, 1, 1, 1, 1, 2]);
        assert_eq!(rows[14], 3);
        assert_eq!(rows[15], 0); // the >= npcFrameCount wrap to 0
    }

    /// `NPC.cs:76822-76831`: the flip only fires grounded, and only on
    /// direction ±1 — a direction-0 or airborne tick keeps the sprite.
    #[test]
    fn scorpion_flips_only_grounded_on_unit_direction() {
        let mut s = FrameState { sprite_dir: 1, ..Default::default() };
        let mut i = input(0.0, -2.0);
        i.direction = -1;
        find_frame_high2(FindFrameKind::Scorpion, &mut s, &i);
        assert_eq!(s.sprite_dir, 1); // airborne: no flip
        i.vy = 0.0;
        i.direction = 0;
        find_frame_high2(FindFrameKind::Scorpion, &mut s, &i);
        assert_eq!(s.sprite_dir, 1); // direction 0: neither if fires
        i.direction = -1;
        find_frame_high2(FindFrameKind::Scorpion, &mut s, &i);
        assert_eq!(s.sprite_dir, -1);
        assert_eq!(s.row, 0); // vx == 0 idle snap
    }

    /// `NPC.cs:71122-71142`: the pin fires EVERY tick; grounded idle
    /// pins row 0 + rotation 0; the walk steps at the 7th tick and the
    /// wrap returns to row 1 (not 0) — rows cycle 1/2/3.
    #[test]
    fn truffle_worm_walk_cycles_one_two_three() {
        let mut s = FrameState::default();
        let i = input(1.0, 0.0);
        let mut rows = Vec::new();
        for _ in 0..28 {
            find_frame_high2(FindFrameKind::TruffleWorm, &mut s, &i);
            rows.push(s.row);
            assert_eq!(s.local_ai0, -2.0);
            assert_eq!(s.rotation, 0.0);
        }
        assert_eq!(rows[6], 1); // first step on the 7th tick (> 6)
        assert_eq!(rows[13], 2);
        assert_eq!(rows[20], 3);
        assert_eq!(rows[27], 1); // >= num*4 wraps to num (row 1)
        // The idle snap (`:71126-71131`).
        find_frame_high2(FindFrameKind::TruffleWorm, &mut s, &input(0.0, 0.0));
        assert_eq!((s.row, s.counter), (0, 0.0));
    }

    /// `NPC.cs:71144-71154`: airborne the rotation accumulates
    /// `direction * 0.1` per tick and the stride is > 3 (4 ticks).
    #[test]
    fn truffle_worm_spins_and_strides_in_the_air() {
        let mut s = FrameState::default();
        let mut i = input(0.0, 1.0);
        i.direction = -1;
        for _ in 0..8 {
            find_frame_high2(FindFrameKind::TruffleWorm, &mut s, &i);
        }
        assert!((s.rotation - (-0.8)).abs() < 1e-5);
        assert_eq!(s.row, 2); // steps on ticks 4 and 8
        assert_eq!(s.counter, 0.0);
    }

    /// `NPC.cs:71110-71119`: the digger steps every 5th tick over
    /// rows 0..3, wrapping to 0.
    #[test]
    fn truffle_worm_digger_walks_four_rows_at_five_ticks() {
        let rows = rows_over(FindFrameKind::TruffleWormDigger, input(0.0, 0.0), 20);
        assert_eq!(rows[4], 1);
        assert_eq!(rows[9], 2);
        assert_eq!(rows[14], 3);
        assert_eq!(rows[19], 0);
    }

    /// `NPC.cs:70973-70981`: airborne row 1, grounded row 0, the flip
    /// unconditional (no counter is touched by either branch).
    #[test]
    fn grasshopper_hops_row_one_and_faces_direction() {
        let mut s = FrameState { counter: 3.0, ..Default::default() };
        let mut i = input(2.0, -1.0);
        i.direction = -1;
        find_frame_high2(FindFrameKind::Grasshopper, &mut s, &i);
        assert_eq!((s.row, s.sprite_dir, s.counter), (1, -1, 3.0));
        i.vy = 0.0;
        find_frame_high2(FindFrameKind::Grasshopper, &mut s, &i);
        assert_eq!(s.row, 0);
    }

    /// `NPC.cs:70984-71005`: the teeth shift banks on the vy edge, and
    /// the 6-tick stride wraps within the bank. Landing from row 3
    /// shifts to 1 (the `> num*2` gate); row 2 stays until the stride
    /// wraps it to 0.
    #[test]
    fn chattering_teeth_shift_banks_on_the_vy_edge() {
        let mut s = FrameState::default();
        let air = input(0.0, 1.0);
        let ground = input(0.0, 0.0);
        find_frame_high2(FindFrameKind::ChatteringTeethBomb, &mut s, &air);
        assert_eq!(s.row, 2); // +2 once
        find_frame_high2(FindFrameKind::ChatteringTeethBomb, &mut s, &air);
        assert_eq!(s.row, 2); // the < num*2 gate blocks a second shift
        for _ in 0..4 {
            find_frame_high2(FindFrameKind::ChatteringTeethBomb, &mut s, &air);
        }
        assert_eq!((s.row, s.counter), (3, 0.0)); // 6-tick stride
        for _ in 0..6 {
            find_frame_high2(FindFrameKind::ChatteringTeethBomb, &mut s, &air);
        }
        assert_eq!(s.row, 2); // >= num*4 wraps to 2 in the air
        // Landing from row 3: the > num*2 shift fires.
        s.row = 3;
        find_frame_high2(FindFrameKind::ChatteringTeethBomb, &mut s, &ground);
        assert_eq!(s.row, 1);
        // Landing on exactly row 2: no shift; the stride wraps it to 0.
        s.row = 2;
        s.counter = 5.0;
        find_frame_high2(FindFrameKind::ChatteringTeethBomb, &mut s, &ground);
        assert_eq!(s.row, 0); // 2 stays, stride → 3 → >= num*2 → 0
    }

    /// `NPC.cs:71590-71596`: the draw band pins `(int)ai[2] + 1`,
    /// re-copies the sprite from direction and zeroes the counter —
    /// even when direction is 0 (the plain copy, not the gated ifs).
    #[test]
    fn cultist_archer_draw_band_pins_the_ai2_row() {
        for kind in [FindFrameKind::CultistArcherBlue, FindFrameKind::CultistArcherWhite] {
            let mut s = FrameState { counter: 5.0, sprite_dir: -1, ..Default::default() };
            let mut i = input(0.0, 0.0);
            i.direction = 0;
            i.ai2 = 3.0;
            find_frame_high2(kind, &mut s, &i);
            assert_eq!((s.row, s.counter, s.sprite_dir), (4, 0.0, 0));
        }
    }

    /// `NPC.cs:71597-71620`: the idle snaps row 0 but KEEPS the
    /// counter; the walk floor is 7, the |vx|*2 accumulator steps at
    /// > 6 and wraps at 12 back to 7; the air pins row 1, counter 0.
    #[test]
    fn cultist_archer_walks_seven_to_eleven_and_idles_without_reset() {
        let mut s = FrameState { counter: 4.0, ..Default::default() };
        find_frame_high2(FindFrameKind::CultistArcherBlue, &mut s, &input(0.0, 0.0));
        assert_eq!((s.row, s.counter), (0, 4.0)); // no counter reset
        let rows = rows_over(FindFrameKind::CultistArcherBlue, input(1.0, 0.0), 21);
        // floor 7 on tick 1; counter 2/4/6/8 → step on the 4th tick.
        assert_eq!(&rows[..5], &[7, 7, 7, 7, 8]);
        assert_eq!(rows[15], 11);
        assert_eq!(rows[19], 7); // the >= 12 wrap back to the floor
        let mut s = FrameState { counter: 9.0, ..Default::default() };
        find_frame_high2(FindFrameKind::CultistArcherWhite, &mut s, &input(3.0, -1.0));
        assert_eq!((s.row, s.counter), (1, 0.0));
    }

    /// `NPC.cs:70887-70905`: the swing ladder picks 3/4/5/6 at the
    /// < 5/10/15 boundaries and breaks BEFORE the flip — the sprite
    /// keeps its old value even with a unit direction.
    #[test]
    fn armed_zombie_swing_ladder_and_early_break() {
        let mut s = FrameState { sprite_dir: 1, ..Default::default() };
        let mut i = input(1.0, 0.0);
        i.direction = -1;
        for (ai2, row) in [(1.0, 3u32), (4.9, 3), (5.0, 4), (9.9, 4), (10.0, 5), (14.9, 5), (15.0, 6)] {
            i.ai2 = ai2;
            find_frame_high2(FindFrameKind::ArmedZombie, &mut s, &i);
            assert_eq!(s.row, row, "ai2 {ai2}");
            assert_eq!(s.sprite_dir, 1, "the ladder breaks before the flip");
        }
    }

    /// `NPC.cs:70918-70928`: the air OR counter-motion pins row 2 with
    /// the counter zeroed; the plain idle pins row 0.
    #[test]
    fn armed_zombie_counter_motion_pins_row_two() {
        let mut s = FrameState::default();
        let mut i = input(1.0, 0.0); // moving right...
        i.direction = -1; // ...while facing left
        find_frame_high2(FindFrameKind::ArmedZombieEskimo, &mut s, &i);
        assert_eq!((s.row, s.counter), (2, 0.0));
        i.direction = 1;
        i.vx = -1.0; // the mirrored arm
        find_frame_high2(FindFrameKind::ArmedZombieSlimed, &mut s, &i);
        assert_eq!(s.row, 2);
        i.vy = -3.0;
        i.vx = 1.0;
        find_frame_high2(FindFrameKind::ArmedZombieSwamp, &mut s, &i);
        assert_eq!(s.row, 2); // airborne
        find_frame_high2(FindFrameKind::ArmedZombiePincussion, &mut s, &input(0.0, 0.0));
        assert_eq!((s.row, s.counter), (0, 0.0));
    }

    /// `NPC.cs:70930-70950`: at vx = 4 the |vx| accumulator walks the
    /// 8/16/24/32 bands 0/1/2/1 and the >= 32 tick only zeroes the
    /// counter — the row HOLDS at 1 for the reset tick (the wave-A
    /// zombie quirk), then the next tick restarts at row 0.
    #[test]
    fn armed_zombie_stride_holds_the_row_on_the_reset_tick() {
        let rows = rows_over(FindFrameKind::ArmedZombie, input(4.0, 0.0), 9);
        assert_eq!(rows, vec![0, 1, 1, 2, 2, 1, 1, 1, 0]);
    }
}
