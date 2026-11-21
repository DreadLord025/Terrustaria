//! W4-f27 — FindFrame extra wave 7 (slice 7): the FIRST 15 unclaimed
//! missing case ids of `NPC.FindFrame` (`NPC.cs:66684-77370`) in the
//! **306+ band** of the f26 census (219 unclaimed after the
//! d44-d49/e13/e14/f26/f27 claims; the early band claim is the
//! ANALYSIS "W4-f27 FindFrame extra wave 7" section — f26c keeps the
//! 278-305 band below 306). The 306..350 window's 11 unclaimed ids
//! **316/317/318/322/323/324/333/334/335/336/337** plus the next 4
//! ascending **437/438/440/442** — the Ghost, the two costume demon
//! eyes, the three costume skeletons, the four ribbon slimes, the
//! Christmas bunny, the Cultist Tablet, the Cultist Devote, the
//! Cultist Boss Clone and the Gold Bird (`ID/NPCID.cs:11309-11557`).
//! Landed 5 kinds per commit.
//!
//! Shape: the kinds are NOT in `NpcKind` yet (their AI/spawn machines
//! are later rounds), so — like `npc_findframe_a.rs` /
//! `npc_findframe_b.rs` (S1-D-3 waves A/B) and the extra-4 wave — the
//! frame machines land first as pure functions over a small
//! [`FrameState`]/[`FrameInput`] pair. The wiring seam is
//! [`find_frame_kind_extra7`]: when a kind joins `NpcKind`, its
//! `find_frame` arm copies `row`/`counter`/`sprite_dir`/`rotation`
//! out, runs one tick and copies them back (the
//! `Npc::find_frame_wave_a` bridge precedent). No arm draws
//! `Main.rand` — the floater group's only draw (`NPC.cs:74844`) is
//! gated `type == 634` and the eye group's dust (`:74007`) is gated
//! `type == 23`, neither is 316/317/318 — so the wiring cannot move a
//! stream.
//!
//! Row convention: the port tracks frame ROW indices, not pixels —
//! the C#'s `num` (the sheet height / `Main.npcFrameCount[type]`,
//! `NPC.cs:66694`) cancels out of every comparison, exactly as in
//! `npc.rs::find_frame`'s existing arms. The per-kind row counts are
//! `Main.npcFrameCount` (`Main.cs:67082`'s 697-entry initializer),
//! counted entry-by-entry in [`FindFrameExtra7Kind::frame_count`] and
//! cross-checked against the port's own `render/entities.rs::
//! NPC_FRAME_COUNT` table.
//!
//! Deliberate skips (recorded in docs/ANALYSIS.md "W4-f27 FindFrame
//! extra wave 7"):
//! * the floater group's type-60 dust (`NPC.cs:74839-74843`), the
//!   type-634 `Main.rand.Next(10)` dust (`:74844-74849`), the 210/211
//!   double-increment + `vx * 0.2` tilt (`:74859-74863`), the
//!   49/51/60/634 `num292--` (`:74867-74870`) and the type-48
//!   `num291 = 5` (`:74871-74874`) — none is 316 (it flaps the FULL
//!   4 rows every 6 ticks);
//! * the eye group's type-23 dust (`NPC.cs:74007-74013`), the
//!   121/660 `localAI[0]` one-shot row-2 latch (`:74014-74018`), the
//!   SECOND flip branch (`:74032-74043` — unreachable for 317/318,
//!   the first branch's type list carries them) and the type-660
//!   `num142 = 6` override (`:74046-74049`) — 317/318 ride the
//!   8-tick strip with the atan2 aim;
//! * the humanoid group's town `ai[0]` sit/talk/attack pose zones
//!   (`NPC.cs:68662-69488`) — 322/323/324 are aiStyle-3 fighters
//!   (`NPC.cs:13079`/`:13092`/`:13105`, no `townNPC`), the wave-B
//!   `humanoid_fighter_frame` keep-one precedent.

// The wiring is a later round (the kinds are not in `NpcKind` yet) —
// the `npc_findframe_a.rs`/`npc_findframe_b.rs` precedent.
#![allow(dead_code)]

/// The extra-wave-7 kinds, keyed by their VANILLA `NPC.type`
/// (`ID/NPCID.cs`'s `public const short` rows).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FindFrameExtra7Kind {
    /// `NPCID.Ghost` = 316 (`NPCID.cs:11309`) — the floater group
    /// (`NPC.cs:74834`, shared with 48/49/51/60/82/93/137/182/210/
    /// 211/253/634/662/664).
    Ghost,
    /// `NPCID.DemonEyeOwl` = 317 (`NPCID.cs:11311`) — the eye group
    /// (`NPC.cs:74003`, shared with 2/23/121/169/190-194/660).
    DemonEyeOwl,
    /// `NPCID.DemonEyeSpaceship` = 318 (`NPCID.cs:11313`) — the same
    /// eye group (`NPC.cs:74004`).
    DemonEyeSpaceship,
    /// `NPCID.SkeletonTopHat` = 322 (`NPCID.cs:11321`) — the
    /// humanoid-group fighter arm (`NPC.cs:68599`).
    SkeletonTopHat,
    /// `NPCID.SkeletonAstonaut` = 323 (`NPCID.cs:11323`, the C#'s
    /// spelling) — the same group (`NPC.cs:68600`).
    SkeletonAstonaut,
}

impl FindFrameExtra7Kind {
    /// The vanilla `NPC.type` — the `FindFrame` switch key.
    pub fn vanilla_type(self) -> i16 {
        match self {
            Self::Ghost => 316,
            Self::DemonEyeOwl => 317,
            Self::DemonEyeSpaceship => 318,
            Self::SkeletonTopHat => 322,
            Self::SkeletonAstonaut => 323,
        }
    }

    /// `Main.npcFrameCount[type]` (`Main.cs:67082`'s initializer,
    /// entry `type` on line `67084 + type/10`, slot `type % 10`) —
    /// cross-checked against `render/entities.rs::NPC_FRAME_COUNT`.
    pub fn frame_count(self) -> u32 {
        match self {
            // Main.cs:67115 (entries 316-318): 4, 2, 2.
            Self::Ghost => 4,
            Self::DemonEyeOwl => 2,
            Self::DemonEyeSpaceship => 2,
            // Main.cs:67116 (322/323): 16, 15.
            Self::SkeletonTopHat => 16,
            Self::SkeletonAstonaut => 15,
        }
    }
}

/// The wiring seam: the vanilla type → this module's kind (the
/// `find_frame_kind_extra4` precedent). `None` outside the slice.
pub fn find_frame_kind_extra7(vanilla_type: i16) -> Option<FindFrameExtra7Kind> {
    Some(match vanilla_type {
        316 => FindFrameExtra7Kind::Ghost,
        317 => FindFrameExtra7Kind::DemonEyeOwl,
        318 => FindFrameExtra7Kind::DemonEyeSpaceship,
        322 => FindFrameExtra7Kind::SkeletonTopHat,
        323 => FindFrameExtra7Kind::SkeletonAstonaut,
        _ => return None,
    })
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
    /// `NPC.rotation` — the floater/eye arms write it; the skeleton
    /// arms never touch it.
    pub rotation: f32,
}

/// The per-tick inputs the arms read: `velocity`, `direction` and
/// the `ai` slots. The later batches add `aiAction` (the ribbon
/// slimes' num2 feed) and `localAI[2]` (the cultist pose ladder).
#[derive(Clone, Copy, Debug)]
pub struct FrameInput {
    pub vx: f32,
    pub vy: f32,
    /// `Entity.direction` (±1; 0 leaves `spriteDirection` alone in
    /// the arms that gate on `direction == 1` / `== -1`).
    pub direction: i32,
    pub ai: [f32; 4],
}

impl Default for FrameInput {
    fn default() -> Self {
        Self { vx: 0.0, vy: 0.0, direction: 1, ai: [0.0; 4] }
    }
}

/// `NPC.FindFrame` (`NPC.cs:66684`) for the extra-wave-7 kinds — one
/// tick.
pub fn find_frame_extra7(kind: FindFrameExtra7Kind, s: &mut FrameState, i: &FrameInput) {
    match kind {
        // `case 316` in the floater group (`NPC.cs:74823-74884`,
        // shared with 48/49/51/60/82/93/137/182/210/211/253/634/662/
        // 664 — wave-B's Wraith 82 and extra-4's Reaper 253 are the
        // same machine, keep-one at the merge): the vx-sign flip
        // (0 leaves it), the `vx * 0.1` tilt, the 6-tick strip over
        // the FULL 4 rows — 316 is not in the `num292--` list
        // (49/51/60/634, `:74867-74870`), not the type-48
        // `num291 = 5` override, not the 210/211 double-increment.
        // The group's dust spawns are types 60/634's (the header's
        // skip — the only `Main.rand` draw `:74844` is 634-gated).
        FindFrameExtra7Kind::Ghost => {
            // `:74850-74857`.
            if i.vx > 0.0 {
                s.sprite_dir = 1;
            }
            if i.vx < 0.0 {
                s.sprite_dir = -1;
            }
            // `:74858`.
            s.rotation = i.vx * 0.1;
            // `:74864-74883` — num291 = 6, num292 = the full count.
            s.counter += 1.0;
            if s.counter >= 6.0 {
                s.row += 1;
                s.counter = 0.0;
            }
            if s.row >= kind.frame_count() {
                s.row = 0;
            }
        }
        // `case 317/318` in the eye group (`NPC.cs:73994-74059`,
        // shared with 2/23/121/169/190-194/660 — wave-A's five eye
        // variants are the same machine, keep-one at the merge):
        // the FIRST aim branch (`:74019-74031` — the type list
        // carries 317/318): `spriteDirection` mirrors the flight
        // sign and `rotation = atan2(vy, vx)`, `+ 3.14f` (the
        // literal, not PI) when moving left; both freeze while
        // `vx == 0`. The 8-tick strip (`num142` = 8; the 6 is type
        // 660's) cycles the 2 rows. The 23 dust, the 121/660
        // `localAI[0]` latch and the SECOND flip branch are other
        // kinds' (the header's skips).
        FindFrameExtra7Kind::DemonEyeOwl | FindFrameExtra7Kind::DemonEyeSpaceship => {
            // `:74019-74031`.
            if i.vx > 0.0 {
                s.sprite_dir = 1;
                s.rotation = i.vy.atan2(i.vx);
            }
            if i.vx < 0.0 {
                s.sprite_dir = -1;
                // The original adds the literal `3.14f`, not PI.
                s.rotation = i.vy.atan2(i.vx) + 3.14;
            }
            // `:74044-74058`.
            s.counter += 1.0;
            if s.counter >= 8.0 {
                s.row += 1;
                s.counter = 0.0;
            }
            if s.row >= kind.frame_count() {
                s.row = 0;
            }
        }
        // `case 322/323` in the humanoid-group case list
        // (`NPC.cs:68599-68600`, 324 joins in batch 2) → the shared
        // fighter walk (`NPC.cs:68647-69655`) via the wave-B keep-one
        // helper (the extra-3 SnowFlinx/Nymph/Lihzahrd delegate
        // precedent, the pub(crate) lift). None is `isLikeATownNPC`
        // (`NPC.cs:6856-6866` — only 453 + the town flag; the
        // SetDefaults arms `:13075-13114` set aiStyle 3, no
        // townNPC), so `num57` = 0 and the walk wraps at the FULL
        // `npcFrameCount`; none is a town slime, so `num108` = 6 and
        // `num109` = row 2. The airborne pin list (`:69647`) carries
        // NONE of 322/323/324 (the group's row 1 stands); none is
        // the 140/489 idle-row-1 pair. The 323 astronaut is the ONE
        // 15-row sheet of the trio (322/324 walk to row 15 before
        // the wrap, 323 to row 14).
        FindFrameExtra7Kind::SkeletonTopHat | FindFrameExtra7Kind::SkeletonAstonaut => {
            let bi = crate::npc_findframe_b::FrameInput {
                vx: i.vx,
                vy: i.vy,
                direction: i.direction,
                wet: false,
                ai: i.ai,
            };
            let mut bs = crate::npc_findframe_b::FrameState {
                row: s.row,
                counter: s.counter,
                sprite_dir: s.sprite_dir,
                rotation: s.rotation,
            };
            crate::npc_findframe_b::humanoid_fighter_frame(
                &mut bs,
                &bi,
                kind.frame_count(),
                false,
                false,
            );
            s.row = bs.row;
            s.counter = bs.counter;
            s.sprite_dir = bs.sprite_dir;
            s.rotation = bs.rotation;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(vx: f32, vy: f32) -> FrameInput {
        FrameInput { vx, vy, ..Default::default() }
    }

    fn rows_over(kind: FindFrameExtra7Kind, i: FrameInput, ticks: usize) -> Vec<u32> {
        let mut s = FrameState::default();
        let mut out = Vec::new();
        for _ in 0..ticks {
            find_frame_extra7(kind, &mut s, &i);
            out.push(s.row);
        }
        out
    }

    /// The frame-count table agrees with the port's verbatim
    /// `NPC_FRAME_COUNT` (`Main.cs:67082` initializer cross-check).
    #[test]
    fn f27s7_frame_counts_match_the_port_table() {
        let table = crate::render::entities::NPC_FRAME_COUNT;
        for k in [
            FindFrameExtra7Kind::Ghost,
            FindFrameExtra7Kind::DemonEyeOwl,
            FindFrameExtra7Kind::DemonEyeSpaceship,
            FindFrameExtra7Kind::SkeletonTopHat,
            FindFrameExtra7Kind::SkeletonAstonaut,
        ] {
            assert_eq!(
                k.frame_count(),
                table[k.vanilla_type() as usize],
                "type {}",
                k.vanilla_type()
            );
        }
    }

    /// 316 Ghost: the floater 6-tick strip over the FULL 4 rows, the
    /// vx-sign flip (0 holds) and the vx*0.1 tilt
    /// (`NPC.cs:74850-74883`). Expectation re-derived by mechanical
    /// replay of the C# order.
    #[test]
    fn f27s7_ghost_flaps_the_full_four_rows() {
        let rows = rows_over(FindFrameExtra7Kind::Ghost, input(2.0, -1.0), 25);
        assert_eq!(
            rows,
            vec![0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 3, 3, 3, 3, 3, 3, 0, 0]
        );
        let mut s = FrameState { sprite_dir: -1, ..Default::default() };
        find_frame_extra7(FindFrameExtra7Kind::Ghost, &mut s, &input(0.0, 0.0));
        assert_eq!(s.sprite_dir, -1, "vx == 0 leaves the flip alone");
        assert_eq!(s.rotation, 0.0);
        find_frame_extra7(FindFrameExtra7Kind::Ghost, &mut s, &input(-3.0, 0.0));
        assert_eq!(s.sprite_dir, -1);
        assert_eq!(s.rotation, -0.3);
    }

    /// 317/318 the costume eyes: the atan2 aim (the literal 3.14f
    /// left-turn offset), the vx == 0 freeze, the 8-tick 2-row strip
    /// (`NPC.cs:74019-74058`).
    #[test]
    fn f27s7_costume_eyes_aim_and_flap() {
        for k in [FindFrameExtra7Kind::DemonEyeOwl, FindFrameExtra7Kind::DemonEyeSpaceship] {
            let rows = rows_over(k, input(2.0, -1.0), 17);
            assert_eq!(rows, vec![0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0]);
            let mut s = FrameState::default();
            find_frame_extra7(k, &mut s, &input(3.0, 4.0));
            assert_eq!(s.sprite_dir, 1);
            assert_eq!(s.rotation, 4.0f32.atan2(3.0));
            find_frame_extra7(k, &mut s, &input(-3.0, 4.0));
            assert_eq!(s.sprite_dir, -1);
            assert_eq!(s.rotation, 4.0f32.atan2(-3.0) + 3.14);
            // vx == 0 freezes BOTH the flip and the aim.
            let mut s = FrameState { sprite_dir: -1, rotation: 0.5, ..Default::default() };
            find_frame_extra7(k, &mut s, &input(0.0, 2.0));
            assert_eq!((s.sprite_dir, s.rotation), (-1, 0.5));
        }
    }

    /// 322/323 the costume skeletons: the wave-B fighter walk — the
    /// |vx|*2+1 accumulator, the > 6 step, the lift-to-2 and the
    /// wrap-to-2 at the FULL count; the 323 astronaut's 15-row sheet
    /// wraps one row earlier (`NPC.cs:69521-69585`).
    #[test]
    fn f27s7_costume_skeletons_walk_the_fighter_bands() {
        for k in [FindFrameExtra7Kind::SkeletonTopHat, FindFrameExtra7Kind::SkeletonAstonaut] {
            // vx 1: +3 a tick, the row steps every 3rd tick.
            let rows = rows_over(k, input(1.0, 0.0), 6);
            assert_eq!(rows, vec![2, 2, 3, 3, 3, 4]);
            // The idle snap + the air row 1 (none is in the pin list).
            let mut s = FrameState { row: 5, counter: 4.0, ..Default::default() };
            find_frame_extra7(k, &mut s, &input(0.0, 0.0));
            assert_eq!((s.row, s.counter), (0, 0.0));
            find_frame_extra7(k, &mut s, &input(1.0, -2.0));
            assert_eq!((s.row, s.counter), (1, 0.0));
        }
        // The count variant: 322 reaches row 15, 323 wraps at 14
        // (324's 16-row twin joins in batch 2).
        for (k, last) in [
            (FindFrameExtra7Kind::SkeletonTopHat, 15u32),
            (FindFrameExtra7Kind::SkeletonAstonaut, 14),
        ] {
            let mut s = FrameState { row: last, counter: 6.0, ..Default::default() };
            find_frame_extra7(k, &mut s, &input(1.0, 0.0));
            assert_eq!(s.row, 2, "type {} wraps at {last}", k.vanilla_type());
        }
    }

    /// The batch's 5-kind strip sweep: a 5x3x3 vx/vy/direction input
    /// grid, 40 ticks each — every row stays inside its sheet (the
    /// extra-4 precedent; the full 15-kind sweep lands with batch 3).
    #[test]
    fn f27s7_batch_strip_sweep_stays_inside_the_sheets() {
        let kinds = [
            FindFrameExtra7Kind::Ghost,
            FindFrameExtra7Kind::DemonEyeOwl,
            FindFrameExtra7Kind::DemonEyeSpaceship,
            FindFrameExtra7Kind::SkeletonTopHat,
            FindFrameExtra7Kind::SkeletonAstonaut,
        ];
        for k in kinds {
            let count = k.frame_count();
            for vx in [-4.0f32, -1.0, 0.0, 1.0, 4.0] {
                for vy in [-3.0f32, 0.0, 3.0] {
                    for direction in [-1i32, 0, 1] {
                        let i = FrameInput { vx, vy, direction, ..Default::default() };
                        let mut s = FrameState::default();
                        for tick in 0..40 {
                            find_frame_extra7(k, &mut s, &i);
                            assert!(
                                s.row < count,
                                "type {} tick {tick} row {} >= {count} (vx {vx}, vy {vy}, dir {direction})",
                                k.vanilla_type(),
                                s.row
                            );
                        }
                    }
                }
            }
        }
    }

    /// The seam maps exactly the batch's vanilla types and nothing
    /// else.
    #[test]
    fn f27s7_seam_covers_exactly_the_slice() {
        for t in 0..697i16 {
            let k = find_frame_kind_extra7(t);
            let claimed = [316, 317, 318, 322, 323];
            assert_eq!(k.is_some(), claimed.contains(&t), "type {t}");
            if let Some(k) = k {
                assert_eq!(k.vanilla_type(), t);
            }
        }
    }
}
