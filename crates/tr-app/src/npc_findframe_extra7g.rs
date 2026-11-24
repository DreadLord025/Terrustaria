//! W4-f26 slice 7g — FindFrame extra wave 7g: the NEXT 5 TRULY
//! unclaimed case ids of `NPC.FindFrame` (`NPC.cs:66684-77370`) after
//! the 7f slice — **435/436/631/632/635** — the two d49-deferred armed
//! zombies (Twiggy/Cenx), the Rock Golem, the Maggot Zombie and the
//! Spore Skeleton (`ID/NPCID.cs:11545-11547` + `:11939-11947`).
//!
//! The brief's original window (537/538/539/540 + 541) is CLOSED: the
//! f27 pushed tip `1a24a133` NOW carries the slice-10 claim band
//! (`git show origin/task/W4-f27-findframe-extra-4:docs/ANALYSIS.md |
//! findstr /C:"slice 10"` = the 531..545 + 549 band, ALL 15 arms LIVE
//! in `npc_findframe_extra10.rs` — the batch-2/3 catch-all is gone).
//! 531/532/534/535/536 dedupe against this branch's own 7f claims;
//! 537/538/539/540/541/542/543/544/545 are f27's 9 NEW claims; 549 is
//! a FIX of f27's own kind (no claim-count change); 533 DesertDjinn
//! stays port-covered EXCLUDED (the S1-E-5 `npc.rs:8086` arm);
//! 546/547/548 have NO case label. The window re-probed mechanically
//! against the d44 master census refresh 3 (`c863a7a5`, the 130-kind
//! UNCLAIMED worklist) minus every pushed claim:
//!
//! * the f26 7d/7e/7f pushed claims (516/521/523-530/531/532/534-536)
//!   = 15 of the 130 (the refresh-3 vintage note flags them stale);
//! * the f27 slice-10 NEW claims = 9;
//! * the d42 tip `2d7ac08c` LANDED find_frame arms for 38 of the 130
//!   (578/580/582-586/590/591/593-603/608-614/616-620/624-629 — its
//!   own `npc.rs` find_frame region, mechanical kind scan);
//! * the c4 tip `0df6964b` owns 581/587 (the style-44 arm) + 592/607/
//!   615 (the style-16 fish arm), the c8 tip `3705670d` owns 606 (the
//!   crawler group), the e26 tip `048cca6c` owns 604/605 (the luck
//!   ladybugs) — the d42 roster comment (`npc.rs:1094-1095`) names
//!   exactly these lanes; all eight read INLINE-ARM on their tips;
//! * the d41 tip `5d036721` (batch 2, the salamanders) greps ZERO for
//!   this slice's five ids + kind names; the d40 tip `8fe1d6d3` stops
//!   at 490; the d43 tip `a725acb0` IS the wave-3 base; the g6/g7/
//!   d44-d49/e13/e14 module seams grep ZERO for the five (mechanical
//!   `\b<id> =>` scan over every pushed findframe module);
//! * this branch's own `npc.rs` has no kind/arm for any of the five.
//!
//! Census: 130 − 15 − 9 − 38 − 8 = 60 unclaimed before THIS slice;
//! − 5 = **55 unclaimed** (the 7f post-state 112 was the 7c-method
//! compatible count; the d44 refresh-2/3 master re-derivation is the
//! new base — the addendum reconciles both). The five picked ids are
//! the ascending head of that 60: 435/436 (the d49 ANALYSIS's own
//! "435/436/591 ride the next wave" deferral — 591 has since landed
//! on d42), then 631/632/635 (633/634 skipped: 633 BestiaryGirl is
//! port-covered `npc.rs:Zoologist`, 634 SporeBat's arm draws
//! `Main.rand.Next(10)` at `NPC.cs:74844` — the zero-RNG rule excludes
//! it from THIS wave; it stays unclaimed for a seam-fed wave).
//!
//! The five arms: 435/436 ride the `case 430..436/591` armed-zombie
//! body (`NPC.cs:70879-70951`, labels `:70884-70885`) — the machine
//! copied VERBATIM from the d49 high2 module (the 7e ghoul ONE-group
//! precedent): the `ai[2] > 0` attack ladder rows 3/4/5/6 off the
//! <5/<10/<15 bands (counter UNTOUCHED), then the wave-A zombie walk
//! (the air/turn row-2 pin, the <8/<16/<24/<32 bands, the lap
//! row-hold). 631 RockGolem rides its own case (`:75259-75313`): the
//! grounded sprite copy is UNCONDITIONAL (direction 0 rides), the
//! `ai[2]` windup pose zone (`0 < ai[2] < 100`) lerps rows 15..21 /
//! 10..15 off the player-relative flags, the walk accumulates
//! `0.2 + |vx|` with the STRICT >8 step over rows 1..7 (the LITERAL
//! `num31 = 8` cap — NOT the 21-row sheet count; rows 8..20 are the
//! pose zone's, never walkable), the air zeroes BOTH. 632/635 ride
//! the humanoid-group case list (`:68619`/`:68621`) → the shared
//! fighter walk with the 632-ONLY `num108 = 12` stride override
//! (`:69524-69527` — the 7f tax-collector twin; 635 keeps the group
//! >6), num57 = 0 (both aiStyle 3, `:17138`/`:17179`, no townNPC),
//! idle row 0, air row 1 (neither in the `:69647` pin list).
//!
//! Shape: the kinds are NOT in `NpcKind` yet (the d43 roster wave
//! owns them), so — the `npc_findframe_extra7f.rs` precedent — the
//! frame machines land first as pure functions over a small
//! [`FrameState`]/[`FrameInput`] pair. The wiring seam is
//! [`find_frame_kind_extra7g`]: when a kind joins `NpcKind`, its
//! `find_frame` arm copies the state fields out, runs one tick and
//! copies them back (the `Npc::find_frame_wave_a` bridge precedent).
//!
//! RNG: NONE — the mechanical grep over the cited ranges
//! (`:70887-70951` + `:75259-75313` + `:68647-68661` +
//! `:69515-69585` + `:69642-69655`, each ≤ 80 lines) = ZERO
//! `Main.rand`/`.Next(`/`NextFloat`/`NextDouble`. World reads: ONLY
//! the golem's pose zone
//! (`:75270-75273` — `Main.player[target]`, the hitbox intersect and
//! `Collision.CanHit`): the pure machine takes the two RESOLVED
//! booleans as [`FrameInput::golem_facing_player`] /
//! [`FrameInput::golem_overlapping`] / [`FrameInput::golem_can_hit`]
//! (the extra-7b slope/frame_x36 seam precedent — the bridge owns the
//! probe). The humanoid `ai[0]` pose zones (`:68662-69488`, with their
//! `Main.rand` draws at `:68944`/`:68951`/`:68953`/`:68959`/`:69003`/
//! `:69007`/`:69412`/`:69436`) are the documented skip (the 7f
//! tax-collector precedent): AI_003 writes `ai[0]` ONLY 0/1 for these
//! kinds (the stand-turn `:57385-57396`; the 1000+ confused writes
//! are 379/380/111-gated), so no zone condition can hold.
//!
//! Row convention: the port tracks frame ROW indices, not pixels —
//! the C#'s `num` cancels out of every comparison. The per-kind row
//! counts are `Main.npcFrameCount` (`Main.cs:67082`'s 697-entry
//! initializer): 435/436 = 7 (`:67127` entries 5/6), 631 = 21
//! (`:67147` entry 1), 632 = 16 (entry 2), 635 = 16 (entry 5) —
//! counted entry-by-entry in [`FindFrameExtra7gKind::frame_count`]
//! and cross-checked against `render/entities.rs::NPC_FRAME_COUNT`.

// The wiring is a later round (the kinds are not in `NpcKind` yet) —
// the `npc_findframe_extra7f.rs` precedent.
#![allow(dead_code)]

/// The extra-wave-7g kinds, keyed by their VANILLA `NPC.type`
/// (`ID/NPCID.cs`'s `public const short` rows).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FindFrameExtra7gKind {
    /// `NPCID.ArmedZombieTwiggy` = 435 (`NPCID.cs:11545`) — the
    /// 430..436/591 armed-zombie group (`NPC.cs:70884`); the
    /// d49-deferred id (430-434 ride the high2 module, 591 landed
    /// on d42).
    ArmedZombieTwiggy,
    /// `NPCID.ArmedZombieCenx` = 436 (`NPCID.cs:11547`) — the same
    /// group (`NPC.cs:70885`), the second d49-deferred id.
    ArmedZombieCenx,
    /// `NPCID.RockGolem` = 631 (`NPCID.cs:11939`) — its own case
    /// (`NPC.cs:75259-75313`), the ai[2] windup pose zone + the
    /// literal-8 walk cap.
    RockGolem,
    /// `NPCID.MaggotZombie` = 632 (`NPCID.cs:11941`) — the
    /// humanoid-group case list (`NPC.cs:68619`), the `num108 = 12`
    /// stride override (`:69524-69527`).
    MaggotZombie,
    /// `NPCID.SporeSkeleton` = 635 (`NPCID.cs:11947`) — the same
    /// humanoid group (`NPC.cs:68621`), the DEFAULT >6 stride.
    SporeSkeleton,
}

impl FindFrameExtra7gKind {
    /// The vanilla `NPC.type` id.
    pub fn vanilla_type(self) -> i16 {
        match self {
            Self::ArmedZombieTwiggy => 435,
            Self::ArmedZombieCenx => 436,
            Self::RockGolem => 631,
            Self::MaggotZombie => 632,
            Self::SporeSkeleton => 635,
        }
    }

    /// `Main.npcFrameCount[type]` (`Main.cs:67082`'s initializer,
    /// counted entry-by-entry): 435/436 = 7 (`:67127` entries 5/6),
    /// 631 = 21 (`:67147` entry 1), 632 = 16 (entry 2), 635 = 16
    /// (entry 5).
    pub fn frame_count(self) -> u32 {
        match self {
            Self::ArmedZombieTwiggy => 7,
            Self::ArmedZombieCenx => 7,
            Self::RockGolem => 21,
            Self::MaggotZombie => 16,
            Self::SporeSkeleton => 16,
        }
    }
}

/// The vanilla-id wiring seam: `Some(kind)` for exactly this wave's
/// ids (the `find_frame_kind_extra7f` precedent). The wiring round
/// hooks it at the `npc.rs::find_frame` head by `cs_id`.
pub fn find_frame_kind_extra7g(vanilla_type: i16) -> Option<FindFrameExtra7gKind> {
    Some(match vanilla_type {
        435 => FindFrameExtra7gKind::ArmedZombieTwiggy,
        436 => FindFrameExtra7gKind::ArmedZombieCenx,
        631 => FindFrameExtra7gKind::RockGolem,
        632 => FindFrameExtra7gKind::MaggotZombie,
        635 => FindFrameExtra7gKind::SporeSkeleton,
        _ => return None,
    })
}

/// The `NPC` fields `FindFrame` reads/writes for these arms:
/// `frame.Y / num` (the row), `frameCounter` and `spriteDirection`.
/// The wiring step copies them to/from `Npc`. `rotation`/`local_ai`
/// are carried for the uniform 7f wiring copy, never read/written
/// here.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FrameState {
    /// `frame.Y / num` — the sheet row.
    pub row: u32,
    /// `NPC.frameCounter` (the C# field is a `double`, `NPC.cs:6485`;
    /// the port's f32 is the existing `npc.rs` convention — the
    /// integer/half accumulators here are exact in both).
    pub counter: f32,
    /// `NPC.spriteDirection` (±1).
    pub sprite_dir: i32,
    /// `NPC.rotation` — no arm writes it this wave.
    pub rotation: f32,
    /// `NPC.localAI[0]`/`[1]` — never read/written this wave; carried
    /// for the uniform wiring copy.
    pub local_ai: [f32; 2],
}

/// The per-tick inputs the arms read: `velocity`, `direction`, the
/// armed-zombie/golem `ai[2]` state and the golem pose zone's three
/// RESOLVED world booleans (the extra-7b seam precedent — the bridge
/// owns the `Main.player`/hitbox/`Collision.CanHit` probe,
/// `NPC.cs:75270-75273`). No RNG draws this wave.
#[derive(Clone, Copy, Debug)]
pub struct FrameInput {
    pub vx: f32,
    pub vy: f32,
    /// `Entity.direction` (±1; 0 leaves `spriteDirection` alone for
    /// the ±1-gated copies — the golem's grounded copy is
    /// UNCONDITIONAL and rides 0 verbatim).
    pub direction: i32,
    /// `NPC.ai[2]` — the armed-zombie attack clock (`:70887`) and the
    /// golem windup timer (`:75267`). The humanoid pair never reads
    /// it in the ported zones.
    pub ai2: f32,
    /// `NPC.cs:75271` — `player != null && (player.Center.X <
    /// npc.Center.X ? direction == -1 : direction == 1)`.
    pub golem_facing_player: bool,
    /// `NPC.cs:75272` — `player?.Hitbox.Intersects(npc.Hitbox)`.
    pub golem_overlapping: bool,
    /// `NPC.cs:75273` — `Collision.CanHit(player, this)`.
    pub golem_can_hit: bool,
}

impl Default for FrameInput {
    fn default() -> Self {
        Self {
            vx: 0.0,
            vy: 0.0,
            direction: 1,
            ai2: 0.0,
            golem_facing_player: false,
            golem_overlapping: false,
            golem_can_hit: false,
        }
    }
}

/// `NPC.FindFrame` (`NPC.cs:66684`) for the extra-wave-7g kinds — one
/// tick.
pub fn find_frame_extra7g(kind: FindFrameExtra7gKind, s: &mut FrameState, i: &FrameInput) {
    match kind {
        // `case 430..436/591` (`NPC.cs:70879-70951`): the armed
        // zombies (only 435/436 are this wave's — the machine copied
        // VERBATIM from the d49 high2 module, the 7e ONE-group
        // precedent). The `ai[2] > 0` attack ladder (`:70887-70906`)
        // pins rows 3/4/5/6 off the <5/<10/<15 bands WITHOUT touching
        // the counter or the sprite; the walk is the wave-A zombie
        // machine: the grounded ±1 flip (`:70907-70917`), the air OR
        // turn-around row-2 pin (`:70918-70923`), the stand row-0
        // zero (`:70924-70929`), the |vx| NO-+1 accumulator with the
        // <8/<16/<24/<32 bands rows 0/1/2/1 (`:70930-70946`), the
        // final else counter-zero WITHOUT a row write (`:70947-70950`
        // — the lap HOLDS row 1, the 7f bride quirk).
        FindFrameExtra7gKind::ArmedZombieTwiggy | FindFrameExtra7gKind::ArmedZombieCenx => {
            // `:70887-70906`.
            if i.ai2 > 0.0 {
                if i.ai2 < 5.0 {
                    s.row = 3;
                } else if i.ai2 < 10.0 {
                    s.row = 4;
                } else if i.ai2 < 15.0 {
                    s.row = 5;
                } else {
                    s.row = 6;
                }
                return;
            }
            // `:70907-70917`.
            if i.vy == 0.0 {
                if i.direction == 1 {
                    s.sprite_dir = 1;
                }
                if i.direction == -1 {
                    s.sprite_dir = -1;
                }
            }
            // `:70918-70950`.
            if i.vy != 0.0 || (i.direction == -1 && i.vx > 0.0) || (i.direction == 1 && i.vx < 0.0) {
                s.counter = 0.0;
                s.row = 2;
            } else if i.vx == 0.0 {
                s.counter = 0.0;
                s.row = 0;
            } else {
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
                    // `:70949` — the row is NOT written here.
                    s.counter = 0.0;
                }
            }
        }
        // `case 631` (`NPC.cs:75259-75313`): the Rock Golem. The
        // grounded sprite copy is UNCONDITIONAL (`:75266` — direction
        // 0 rides verbatim, NOT the ±1-gated fighter shape). The
        // windup pose zone (`:75267-75286`, `0 < ai[2] < 100`): the
        // amount = `(ai[2] - 68) / 32` (`:75269`, NOT clamped — the
        // AI keeps ai[2] in 68..100 here), the facing+can-hit branch
        // lerps rows 15..21 (`:75275`, `MathHelper.Lerp(15, count,
        // amount)` truncated), the overlap branch rows 10..15
        // (`:75279`), the else row 0 (`:75283`); the counter is
        // UNTOUCHED in the whole zone. The walk (`:75287-75306`):
        // idle zeroes BOTH (`:75289-75290`), the `0.2 + |vx|`
        // accumulator with the STRICT >8 step (`:75293-75298`), the
        // LITERAL `num31 = 8` cap wrapping to row 1 (`:75299-75302` —
        // NOT the 21-row count; rows 8..20 are the pose zone's) and
        // the row-<1 lift to 1 (`:75303-75306`). The air zeroes BOTH
        // to row 0 (`:75308-75312`).
        FindFrameExtra7gKind::RockGolem => {
            if i.vy == 0.0 {
                // `:75266` — UNCONDITIONAL.
                s.sprite_dir = i.direction;
                // `:75267-75286`.
                if i.ai2 != 0.0 && i.ai2 < 100.0 {
                    // `:75269`.
                    let amount = (i.ai2 - 68.0) / 32.0;
                    if i.golem_facing_player && !i.golem_overlapping && i.golem_can_hit {
                        // `:75275` — Lerp(15, count, amount), (int).
                        s.row = (15.0 + (kind.frame_count() as f32 - 15.0) * amount) as u32;
                    } else if i.golem_facing_player && i.golem_overlapping {
                        // `:75279` — Lerp(10, 15, amount), (int).
                        s.row = (10.0 + 5.0 * amount) as u32;
                    } else {
                        // `:75283`.
                        s.row = 0;
                    }
                    return;
                }
                if i.vx == 0.0 {
                    // `:75289-75290`.
                    s.counter = 0.0;
                    s.row = 0;
                    return;
                }
                // `:75293-75306`.
                s.counter += 0.2 + i.vx.abs();
                if s.counter > 8.0 {
                    s.row += 1;
                    s.counter = 0.0;
                }
                if s.row >= 8 {
                    // `:75299-75302` — the LITERAL num31 = 8 cap.
                    s.row = 1;
                } else if s.row < 1 {
                    // `:75303-75306`.
                    s.row = 1;
                }
            } else {
                // `:75308-75312`.
                s.counter = 0.0;
                s.row = 0;
            }
        }
        // `case 632`/`case 635` in the humanoid-group case list
        // (`NPC.cs:68619`/`:68621`) → the shared fighter walk
        // (`:68647-69655`). 632 has the ONE per-type override
        // `num108 = 12` (`:69524-69527` — the 7f tax-collector twin);
        // 635 keeps the group default 6 (`:69523`). NOT
        // `isLikeATownNPC` (both aiStyle 3, `:17138`/`:17179`, no
        // townNPC — `NPC.cs:6856-6866`), so `num57` = 0 and the wrap
        // runs at the FULL 16-row count; NOT a town slime
        // (`NPCID.cs:10119`'s set), so `num109` = row 2; NOT the
        // 140/489 idle-row-1 pair (`:69510`); NOT in the `:69647` air
        // pin list — the group air tail pins row 1 (`:69644-69645`).
        // The accumulator is the group's `|vx| * 2` THEN `++`
        // (`:69552-69553` — not the 489 variant). The ai[0] pose
        // zones (`:68662-69488`) are the documented skip (the 7f
        // precedent — AI_003 rests ai[0] at 0/1 for these kinds).
        FindFrameExtra7gKind::MaggotZombie | FindFrameExtra7gKind::SporeSkeleton => {
            // `:69523-69527` — the 632-only 12 override.
            let num108: f32 = if kind == FindFrameExtra7gKind::MaggotZombie { 12.0 } else { 6.0 };
            if i.vy == 0.0 {
                // `:68649-68656`.
                if i.direction == 1 {
                    s.sprite_dir = 1;
                }
                if i.direction == -1 {
                    s.sprite_dir = -1;
                }
                if i.vx == 0.0 {
                    // `:69515-69519`.
                    s.row = 0;
                    s.counter = 0.0;
                } else {
                    // `:69521-69585` — num109 = row 2.
                    if s.row < 2 {
                        s.row = 2;
                    }
                    s.counter += i.vx.abs() * 2.0;
                    s.counter += 1.0;
                    if s.counter > num108 {
                        s.row += 1;
                        s.counter = 0.0;
                    }
                    if s.row >= kind.frame_count() {
                        s.row = 2;
                    }
                }
            } else {
                // `:69642-69646` — the group air tail: row 1, the
                // counter zeroed; neither id is in the `:69647` list.
                s.counter = 0.0;
                s.row = 1;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(vx: f32, vy: f32) -> FrameInput {
        FrameInput { vx, vy, ..Default::default() }
    }

    fn facing(vx: f32, vy: f32, direction: i32) -> FrameInput {
        FrameInput { vx, vy, direction, ..Default::default() }
    }

    fn rows_over(kind: FindFrameExtra7gKind, i: FrameInput, ticks: usize) -> Vec<u32> {
        let mut s = FrameState::default();
        let mut out = Vec::new();
        for _ in 0..ticks {
            find_frame_extra7g(kind, &mut s, &i);
            out.push(s.row);
        }
        out
    }

    /// The frame-count table agrees with the port's verbatim
    /// `NPC_FRAME_COUNT` (`Main.cs:67082` initializer cross-check).
    #[test]
    fn f26s7g_frame_counts_match_the_port_table() {
        let table = crate::render::entities::NPC_FRAME_COUNT;
        for k in [
            FindFrameExtra7gKind::ArmedZombieTwiggy,
            FindFrameExtra7gKind::ArmedZombieCenx,
            FindFrameExtra7gKind::RockGolem,
            FindFrameExtra7gKind::MaggotZombie,
            FindFrameExtra7gKind::SporeSkeleton,
        ] {
            assert_eq!(
                k.frame_count(),
                table[k.vanilla_type() as usize],
                "{k:?} ({})",
                k.vanilla_type()
            );
        }
    }

    /// The armed zombies' ai[2] ladder: rows 3/4/5/6 off the
    /// <5/<10/<15 bands, the counter and sprite UNTOUCHED (the d49
    /// verbatim quirk — the attack pose never feeds the walk clock).
    #[test]
    fn f26s7g_the_armed_zombie_attack_ladder() {
        for k in [FindFrameExtra7gKind::ArmedZombieTwiggy, FindFrameExtra7gKind::ArmedZombieCenx] {
            for (ai2, row) in [(1.0, 3), (4.0, 3), (5.0, 4), (9.0, 4), (10.0, 5), (14.0, 5), (15.0, 6), (99.0, 6)] {
                let mut s = FrameState { row: 1, counter: 7.0, sprite_dir: -1, ..Default::default() };
                find_frame_extra7g(k, &mut s, &FrameInput { ai2, ..input(3.0, 2.0) });
                assert_eq!((s.row, s.counter, s.sprite_dir), (row, 7.0, -1), "ai2 {ai2}");
            }
        }
    }

    /// The armed-zombie walk is the VERBATIM wave-A machine (the 7f
    /// bride twin): vx 3 adds 3 a tick — rows 0 (ticks 1-2), 1 (3-5),
    /// 2 (6-7), 1 (8-10); tick 11 hits the >= 32 else: the counter
    /// zeroes but the row HOLDS 1, then tick 12 restarts at row 0.
    #[test]
    fn f26s7g_the_armed_zombie_walk_bands_and_lap_hold() {
        let k = FindFrameExtra7gKind::ArmedZombieTwiggy;
        assert_eq!(
            rows_over(k, input(3.0, 0.0), 12),
            vec![0, 0, 1, 1, 1, 2, 2, 1, 1, 1, 1, 0]
        );
        // The lap edge: counter 30 + 3 = 33 >= 32 → zero, row HOLDS 1.
        let mut s = FrameState { row: 1, counter: 30.0, ..Default::default() };
        find_frame_extra7g(k, &mut s, &input(3.0, 0.0));
        assert_eq!((s.row, s.counter), (1, 0.0));
        // The air/turn pin: airborne OR against the facing → row 2,
        // counter 0; the grounded flip is ±1-gated (0 keeps sprite).
        let mut s = FrameState { row: 0, counter: 9.0, sprite_dir: 1, ..Default::default() };
        find_frame_extra7g(k, &mut s, &facing(-3.0, 2.0, -1));
        assert_eq!((s.row, s.counter, s.sprite_dir), (2, 0.0, 1));
        let mut s = FrameState { row: 1, counter: 9.0, sprite_dir: 1, ..Default::default() };
        find_frame_extra7g(k, &mut s, &facing(3.0, 0.0, -1));
        assert_eq!((s.row, s.counter, s.sprite_dir), (2, 0.0, -1));
        // The stand: row 0 + counter 0; direction 0 keeps sprite -1.
        let mut s = FrameState { row: 2, counter: 9.0, sprite_dir: -1, ..Default::default() };
        find_frame_extra7g(k, &mut s, &facing(0.0, 0.0, 0));
        assert_eq!((s.row, s.counter, s.sprite_dir), (0, 0.0, -1));
    }

    /// The golem's windup pose zone: ai2 84 → amount 0.5 — the
    /// facing+can-hit branch truncates Lerp(15, 21, 0.5) = 18, the
    /// overlap branch Lerp(10, 15, 0.5) = 12, the else row 0; the
    /// counter is UNTOUCHED everywhere. ai2 68 → amount 0 → rows
    /// 15/10; ai2 99 → amount 0.96875 → (int)20.8125 = 20 / 14.
    #[test]
    fn f26s7g_the_golem_windup_pose_lerps() {
        let k = FindFrameExtra7gKind::RockGolem;
        let base = FrameState { row: 3, counter: 5.0, ..Default::default() };
        let mk = |ai2, f, o, c| FrameInput {
            ai2,
            golem_facing_player: f,
            golem_overlapping: o,
            golem_can_hit: c,
            ..input(1.0, 0.0)
        };
        let mut s = base;
        find_frame_extra7g(k, &mut s, &mk(84.0, true, false, true));
        assert_eq!((s.row, s.counter), (18, 5.0));
        let mut s = base;
        find_frame_extra7g(k, &mut s, &mk(84.0, true, true, true));
        assert_eq!((s.row, s.counter), (12, 5.0));
        let mut s = base;
        find_frame_extra7g(k, &mut s, &mk(84.0, false, false, true));
        assert_eq!((s.row, s.counter), (0, 5.0));
        // Facing but no line of sight → the else row 0.
        let mut s = base;
        find_frame_extra7g(k, &mut s, &mk(84.0, true, false, false));
        assert_eq!((s.row, s.counter), (0, 5.0));
        let mut s = base;
        find_frame_extra7g(k, &mut s, &mk(68.0, true, false, true));
        assert_eq!(s.row, 15);
        let mut s = base;
        find_frame_extra7g(k, &mut s, &mk(68.0, true, true, false));
        assert_eq!(s.row, 10);
        let mut s = base;
        find_frame_extra7g(k, &mut s, &mk(99.0, true, false, true));
        assert_eq!(s.row, 20);
        let mut s = base;
        find_frame_extra7g(k, &mut s, &mk(99.0, true, true, false));
        assert_eq!(s.row, 14);
        // ai2 100 is NOT < 100 — the zone exits to the walk (vx 0.8
        // adds EXACTLY 1.0: counter 5 → 6, no ulp edge).
        let mut s = base;
        find_frame_extra7g(k, &mut s, &FrameInput { ai2: 100.0, golem_facing_player: true, golem_can_hit: true, ..input(0.8, 0.0) });
        assert_eq!((s.row, s.counter), (3, 6.0));
    }

    /// The golem walk: vx 0.8 adds EXACTLY 1.0 a tick (0.2f + 0.8f
    /// rounds to 1.0f) — the STRICT >8 step lands every 9th tick
    /// (the counter RESETS on the step), rows walk 1..7 and the
    /// LITERAL-8 cap wraps to row 1 (NOT the 21-row count). The
    /// row-<1 lift fires on the first tick.
    #[test]
    fn f26s7g_the_golem_walk_literal_eight_cap() {
        let k = FindFrameExtra7gKind::RockGolem;
        assert_eq!(
            rows_over(k, input(0.8, 0.0), 18),
            vec![1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 2, 2, 2, 3]
        );
        // The cap edge: row 7 + counter 8 → +1 = 9 > 8 → row 8 → the
        // literal cap → 1, counter 0.
        let mut s = FrameState { row: 7, counter: 8.0, ..Default::default() };
        find_frame_extra7g(k, &mut s, &input(0.8, 0.0));
        assert_eq!((s.row, s.counter), (1, 0.0));
        // The idle zeroes BOTH (the UNCONDITIONAL grounded sprite
        // copy still runs — direction -1 rides); the air zeroes BOTH
        // (no sprite copy in the air — the grounded-only :75266).
        let mut s = FrameState { row: 5, counter: 7.0, sprite_dir: 1, ..Default::default() };
        find_frame_extra7g(k, &mut s, &facing(0.0, 0.0, -1));
        assert_eq!((s.row, s.counter, s.sprite_dir), (0, 0.0, -1));
        let mut s = FrameState { row: 5, counter: 7.0, sprite_dir: 1, ..Default::default() };
        find_frame_extra7g(k, &mut s, &facing(3.0, 2.0, -1));
        assert_eq!((s.row, s.counter, s.sprite_dir), (0, 0.0, 1));
        // The UNCONDITIONAL grounded copy: direction 0 rides verbatim
        // (NOT the ±1-gated fighter shape).
        let mut s = FrameState { sprite_dir: 1, ..Default::default() };
        find_frame_extra7g(k, &mut s, &facing(0.8, 0.0, 0));
        assert_eq!(s.sprite_dir, 0);
    }

    /// The Maggot Zombie's num108 = 12 stride (the 7f tax-collector
    /// twin on the 16-row sheet): vx 1 adds 3 a tick, the > 12 step
    /// lands every 5th tick; the wrap runs at the FULL 16 back to 2.
    #[test]
    fn f26s7g_the_maggot_zombie_twelve_stride() {
        let k = FindFrameExtra7gKind::MaggotZombie;
        assert_eq!(
            rows_over(k, input(1.0, 0.0), 15),
            vec![2, 2, 2, 2, 3, 3, 3, 3, 3, 4, 4, 4, 4, 4, 5]
        );
        // The wrap edge: row 15 + counter 11 → +3 = 14 > 12 → row 16
        // → >= 16 → 2, counter 0.
        let mut s = FrameState { row: 15, counter: 11.0, ..Default::default() };
        find_frame_extra7g(k, &mut s, &input(1.0, 0.0));
        assert_eq!((s.row, s.counter), (2, 0.0));
    }

    /// The Spore Skeleton keeps the group DEFAULT >6 stride: vx 1
    /// adds 3 a tick, the step lands every 3rd tick (vs the maggot's
    /// 5th) — the ONE-override difference on the same 16-row sheet.
    #[test]
    fn f26s7g_the_spore_skeleton_default_six_stride() {
        let k = FindFrameExtra7gKind::SporeSkeleton;
        assert_eq!(
            rows_over(k, input(1.0, 0.0), 10),
            vec![2, 2, 3, 3, 3, 4, 4, 4, 5, 5]
        );
        let mut s = FrameState { row: 15, counter: 5.0, ..Default::default() };
        find_frame_extra7g(k, &mut s, &input(1.0, 0.0));
        assert_eq!((s.row, s.counter), (2, 0.0));
    }

    /// The humanoid pair's idle/air pins: the idle zeroes BOTH row
    /// and counter (NOT the 140/489 row-1 pair); the air pins row 1 +
    /// zeroes the counter (neither in the :69647 row-0 pin list) with
    /// the sprite UNTOUCHED; the grounded flip is ±1-gated.
    #[test]
    fn f26s7g_the_humanoid_pair_idle_and_air_pins() {
        for k in [FindFrameExtra7gKind::MaggotZombie, FindFrameExtra7gKind::SporeSkeleton] {
            let mut s = FrameState { row: 5, counter: 7.0, sprite_dir: -1, ..Default::default() };
            find_frame_extra7g(k, &mut s, &facing(0.0, 0.0, 0));
            assert_eq!((s.row, s.counter, s.sprite_dir), (0, 0.0, -1));
            let mut s = FrameState { row: 5, counter: 7.0, sprite_dir: 1, ..Default::default() };
            find_frame_extra7g(k, &mut s, &facing(3.0, -2.0, -1));
            assert_eq!((s.row, s.counter, s.sprite_dir), (1, 0.0, 1));
            let mut s = FrameState { sprite_dir: 1, ..Default::default() };
            find_frame_extra7g(k, &mut s, &facing(-1.0, 0.0, -1));
            assert_eq!(s.sprite_dir, -1);
        }
    }

    /// The seam maps exactly this wave's 5 ids — the d49-covered
    /// 430-434, the d42-covered 591/633-adjacent ids, the RNG-excluded
    /// 634, the f27-claimed 537/541 and the no-case 546 stay `None`.
    #[test]
    fn f26s7g_the_seam_maps_exactly_the_slice() {
        for (id, k) in [
            (435, FindFrameExtra7gKind::ArmedZombieTwiggy),
            (436, FindFrameExtra7gKind::ArmedZombieCenx),
            (631, FindFrameExtra7gKind::RockGolem),
            (632, FindFrameExtra7gKind::MaggotZombie),
            (635, FindFrameExtra7gKind::SporeSkeleton),
        ] {
            assert_eq!(find_frame_kind_extra7g(id), Some(k));
            assert_eq!(k.vanilla_type(), id);
        }
        for id in [430, 434, 537, 541, 546, 591, 633, 634, 636] {
            assert_eq!(find_frame_kind_extra7g(id), None, "type {id}");
        }
    }

    /// The 5-kind strip sweep: every kind ticks 27 input combos
    /// (vx 0/1/3 × vy 0/-2/2 × direction 1/-1/0) for 40 ticks each
    /// from a fresh state — every row stays inside its own sheet
    /// (the 7f sweep shape; the golem's walk cap 8 and the zombie
    /// lap both exercise inside the window).
    #[test]
    fn f26s7g_the_5_kind_strip_sweep_stays_in_sheet() {
        let kinds = [
            FindFrameExtra7gKind::ArmedZombieTwiggy,
            FindFrameExtra7gKind::ArmedZombieCenx,
            FindFrameExtra7gKind::RockGolem,
            FindFrameExtra7gKind::MaggotZombie,
            FindFrameExtra7gKind::SporeSkeleton,
        ];
        for k in kinds {
            let mut combo = 0;
            for vx in [0.0f32, 1.0, 3.0] {
                for vy in [0.0f32, -2.0, 2.0] {
                    for direction in [1i32, -1, 0] {
                        let mut s = FrameState::default();
                        let i = FrameInput { vx, vy, direction, ..Default::default() };
                        for _ in 0..40 {
                            find_frame_extra7g(k, &mut s, &i);
                            assert!(
                                s.row < k.frame_count(),
                                "{k:?} combo {combo} row {} escapes the {}-row sheet",
                                s.row,
                                k.frame_count()
                            );
                        }
                        combo += 1;
                    }
                }
            }
        }
    }

    /// The golem pose-zone sweep: ai2 68/84/99 × the 8 flag combos —
    /// every pose row stays inside the 21-row sheet and the counter
    /// is never touched (the zone's verbatim invariant).
    #[test]
    fn f26s7g_the_golem_pose_sweep_stays_in_sheet() {
        let k = FindFrameExtra7gKind::RockGolem;
        for ai2 in [68.0f32, 84.0, 99.0] {
            for f in [false, true] {
                for o in [false, true] {
                    for c in [false, true] {
                        let mut s = FrameState { row: 3, counter: 5.0, ..Default::default() };
                        let i = FrameInput {
                            ai2,
                            golem_facing_player: f,
                            golem_overlapping: o,
                            golem_can_hit: c,
                            ..input(1.0, 0.0)
                        };
                        find_frame_extra7g(k, &mut s, &i);
                        assert!(s.row < 21, "ai2 {ai2} flags {f}/{o}/{c} row {}", s.row);
                        assert_eq!(s.counter, 5.0);
                    }
                }
            }
        }
    }
}
