//! W4-d54 — the `KillTile` / `ReplaceTile` dust-loop WRAPPER
//! (`WorldGen.cs:63699-63703` / `:62570-62574`). The
//! `KillTile_GetTileDustAmount` gate + the ~250-arm type ladder + the
//! 14-arm spawn tail now live in their canonical homes (research/66
//! union exec): the ladder + tail are
//! `tr_core::worldgen_dust_tile::kill_tile_make_tile_dust` (g14's
//! 8-chunk C#-order split, with the arm-11 torch round-trip fix and
//! `edit::solid_tile`'s OOB-false edge), and the amount table is
//! `collision::tile_dust_amount` (the main-branch copy — NOTE the
//! argument order: kind FIRST, then fail). This module keeps only the
//! count loop and the witnesses that supplement g14's ten (the widest
//! spawn-tail census, the type-table + random-arms oracles re-pointed
//! through the tr-core body's slot/sentinel return).
//!
//! Callers (the C# order):
//! - `KillTile` (`:63699-63703`): the amount-gated loop, AFTER
//!   `KillTile_PlaySounds` and BEFORE the drops. Wired in
//!   `editing.rs::break_tile` with the pre-kill tile snapshot (the C#
//!   `tile` reference is still intact at `:63699` — the clear happens
//!   in the drop half).
//! - `ReplaceTile` (`:62570-62574`): the same loop with `fail: false`
//!   — no live port caller yet (the smart-cursor replacement lane).

use tr_core::particles::{DustPool, SpawnCtx};
use tr_core::tile::Tile;
use tr_core::{UnifiedRandom, World};

/// The `KillTile` / `ReplaceTile` dust loop (`WorldGen.cs:63699-63703`
/// / `:62570-62574`): `KillTile_GetTileDustAmount` calls of the body.
/// `gen_rng` is the caller's `frame_rng` snapshot; `t` the pre-kill tile.
#[allow(clippy::too_many_arguments)]
pub(crate) fn tile_break_burst(
    rng: &mut UnifiedRandom,
    gen_rng: &mut UnifiedRandom,
    disco: [u8; 3],
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
    i: i32,
    j: i32,
    t: &Tile,
    fail: bool,
) {
    let count = crate::collision::tile_dust_amount(t.kind, fail, t.frame_x);
    for _ in 0..count {
        tr_core::worldgen_dust_tile::kill_tile_make_tile_dust(
            i,
            j,
            t,
            (disco[0], disco[1], disco[2]),
            rng,
            gen_rng,
            dust,
            ctx,
            world,
        );
    }
}

#[cfg(test)]
mod tests {
    //! W4-d54 batch-3 dust witnesses — WRITTEN, NOT RUN (code-only).
    //! Re-pointed at the union survivors (research/66 §3): the amount
    //! table pins `collision::tile_dust_amount`; the type-table and
    //! random-arms oracles read the tr-core body's spawned type (the
    //! ladder is private there — the slot's `dust_type` is the
    //! observable; negative resolutions surface as the sentinel).

    use super::*;
    use tr_core::particles::{Color, DUST_SENTINEL};
    use glam::Vec2;

    fn dust_ctx(w: &World) -> SpawnCtx {
        SpawnCtx {
            game_menu: false,
            game_paused: false,
            generating: false,
            screen_position: Vec2::ZERO,
            screen_size: Vec2::new(w.width as f32 * 16.0, w.height as f32 * 16.0),
            max_dust_to_draw: 6000,
        }
    }

    fn tile(kind: u16, fx: i16, fy: i16) -> Tile {
        Tile { kind, frame_x: fx, frame_y: fy, ..Default::default() }
    }

    /// The default block + the per-type tweak family (`Dust.cs:214-
    /// 252`): only the 228/279/269/135/6/242/75/169/29/59..65/158/
    /// 293..298/302/307/310 family adds the `Next(-10,6)`; the other
    /// tweak arms (127/187, 308, 33/52/266/98..105, 41, 80, 34/35/152)
    /// draw nothing.
    fn block(r: &mut UnifiedRandom, dust_type: i32, box_w: i32, box_h: i32) {
        r.next_max(box_w.max(5) - 4);
        r.next_max(box_h.max(5) - 4);
        r.next_range(-20, 21);
        r.next_range(-20, 21);
        r.next_max(3);
        r.next_range(-20, 21);
        if matches!(
            dust_type,
            228 | 279 | 269 | 135 | 6 | 242 | 75 | 169 | 29 | 59..=65 | 158
                | 293..=298 | 302 | 307 | 310
        ) {
            r.next_range(-10, 6);
        }
    }

    fn drain(dust: &mut DustPool) {
        for s in dust.slots_mut() {
            s.active = false;
        }
    }

    /// One tr-core body call; returns the resolved type or None on the
    /// sentinel (the ladder's negative-num veto).
    fn body(
        w: &World,
        ctx: &SpawnCtx,
        rng: &mut UnifiedRandom,
        gen_rng: &mut UnifiedRandom,
        dust: &mut DustPool,
        t: &Tile,
    ) -> Option<i32> {
        let idx = tr_core::worldgen_dust_tile::kill_tile_make_tile_dust(
            100, 100, t, (0, 0, 0), rng, gen_rng, dust, ctx, w,
        );
        if idx == DUST_SENTINEL {
            None
        } else {
            Some(dust.slot(idx).dust_type)
        }
    }

    /// `WorldGen.cs:66597-66685` — the dust-amount table pinned to the
    /// C# literals, both fail arms + the 184 frame gate, on the KEPT
    /// `collision::tile_dust_amount` copy (kind first, then fail).
    #[test]
    fn w4_d54_tile_dust_amount_matches_the_csharp_table() {
        for fail in [false, true] {
            let d = |k: u16, fx: i16| crate::collision::tile_dust_amount(k, fail, fx);
            assert_eq!(d(1, 0), if fail { 3 } else { 10 });
            assert_eq!(d(654, 0), 0);
            assert_eq!(d(630, 0), 0);
            assert_eq!(d(631, 0), 0);
            assert_eq!(d(231, 0), if fail { 3 } else { 6 });
            assert_eq!(d(481, 0), 3);
            assert_eq!(d(482, 0), 3);
            assert_eq!(d(483, 0), 3);
            assert_eq!(d(541, 0), 1);
            assert_eq!(d(549, 0), 2);
            for k in [125u16, 138, 172, 287, 300, 308, 354, 355, 373, 376, 461,
                484, 499, 564, 565, 593, 594, 617, 664, 665, 699, 709, 711, 716] {
                assert_eq!(d(k, 0), 0, "kind {k}");
            }
            for k in [534u16, 536, 539, 625, 627] {
                assert_eq!(d(k, 0), if fail { 3 } else { 5 }, "kind {k}");
            }
            assert_eq!(d(184, 110), 5); // frameX/22 == 5
            assert_eq!(d(184, 109), if fail { 3 } else { 10 });
        }
    }

    /// The type chain's constant arms pinned to the C# literals — a
    /// spread across the head, the switches, the frame ladders and the
    /// later-wins overwrites. No genRand/Main.rand draws for these.
    /// Read through the tr-core body's spawned type (research/66: the
    /// ladder is g14's private chunks; None = the sentinel veto).
    #[test]
    fn w4_d54_tile_dust_type_matches_the_csharp_table() {
        let w = World::new("t".into(), 200, 200);
        let ctx = dust_ctx(&w);
        let mut rng = UnifiedRandom::new(1);
        let mut gen_rng = UnifiedRandom::new(1);
        let mut dust = DustPool::new();
        let cases: &[(Tile, i32)] = &[
            (tile(0, 0, 0), 0),
            (tile(216, 0, 0), -1),
            (tile(324, 0, 0), 280),
            (tile(324, 18, 18), 282),
            (tile(335, 0, 0), -1),
            (tile(192, 0, 0), 3),
            (tile(208, 0, 0), 126),
            (tile(408, 0, 0), 265),
            (tile(669, 0, 0), 314),
            (tile(676, 0, 0), 321),
            (tile(16, 0, 0), 1),
            (tile(16, 36, 0), 82),
            (tile(415, 0, 0), 6),
            (tile(474, 0, 0), 18),
            (tile(1, 0, 0), 1),
            (tile(63, 0, 0), 1), // TILE_STONE
            (tile(566, 0, 0), 1),
            (tile(753, 0, 0), -1),
            (tile(379, 0, 0), 257),
            (tile(311, 0, 0), 207),
            (tile(104, 0, 0), -1),
            (tile(95, 0, 0), 6),
            (tile(30, 0, 0), 7),
            (tile(646, 0, 0), 29),
            (tile(10, 0, 0), -1),
            (tile(668, 0, 0), 0),
            (tile(407, 0, 0), 10),
            (tile(454, 0, 0), 139),
            (tile(41, 0, 0), 275), // the switch overwrites the stone 1
            (tile(43, 0, 0), 276),
            (tile(44, 0, 0), 277),
            (tile(473, 0, 0), 82),
            (tile(498, 0, 0), 30),
            (tile(517, 0, 0), 258),
            (tile(184, 110, 0), 258), // style 5
            (tile(184, 220, 0), 267), // style 10
            (tile(184, 0, 0), 93),    // style 0 default
            (tile(515, 0, 0), 96),
            (tile(541, 0, 0), 226),
            (tile(595, 0, 0), 78),
            (tile(637, 0, 0), 237),
            (tile(179, 0, 0), 93),  // moss default: 179-179+93
            (tile(381, 0, 0), 258), // moss arm
            (tile(627, 0, 0), 267),
            (tile(240, 0, 0), 7),
            (tile(240, 54 * 16, 0), 26),
            (tile(240, 54 * 46, 0), -1),
            (tile(241, 0, 0), 1),
            (tile(365, 0, 0), 239),
            (tile(357, 0, 0), 236),
            (tile(170, 0, 0), 196),
            (tile(326, 0, 0), 13),
            (tile(336, 0, 0), 6),
            (tile(340, 0, 0), 75),
            (tile(562, 0, 0), 273),
            (tile(330, 0, 0), 9),
            (tile(722, 0, 0), 57),
            (tile(734, 0, 0), 0),
            (tile(750, 0, 0), 26),
            (tile(742, 0, 0), 96),
            (tile(101, 0, 0), -1),
            (tile(19, 0, 0), 7),
            (tile(19, 0, 18 * 50), 309),
            // The 79 frame ladder (`:67424-67428`) is dead: the final
            // `type == 79 || 90 || 101 -> num = -1` overwrite (`:69039-69042`)
            // runs after it for every 79 cell.
            (tile(79, 0, 0), -1),
            (tile(79, 0, 36 * 8), -1),
            (tile(18, 0, 0), 7),
            (tile(18, 36 * 15, 0), 126),
            (tile(18, 36 * 16, 0), -1),
            (tile(14, 0, 0), -1),
            (tile(255, 0, 0), 86),
            (tile(261, 0, 0), 138),
            (tile(268, 0, 0), 138),
            (tile(178, 18 * 6, 0), 138),
            (tile(440, 0, 54), 90),
            (tile(440, 0, 0), -1), // frameY < 54
            (tile(426, 0, 0), 90),
            (tile(732, 0, 0), 91),
            (tile(549, 0, 0), 3),
            (tile(186, 0, 0), 26),
            (tile(186, 1400, 0), 80),
            (tile(187, 0, 0), 1),
            (tile(187, 1200, 0), 1), // the dead 155 arm never fires
            (tile(647, 0, 0), 26),
            (tile(648, 0, 0), 1),
            (tile(105, 1600, 0), 148),
            (tile(239, 18 * 22, 0), 265),
            (tile(185, 0, 18), 1),
            (tile(185, 36 * 20, 18), 87),
            (tile(185, 18 * 75, 0), 32),
            (tile(649, 36 * 64, 0), 40),
            (tile(650, 18 * 75, 0), 40),
            (tile(237, 0, 0), 148),
            (tile(157, 0, 0), 77),
            (tile(15, 0, 0), -1),
            (tile(137, 0, 18 * 2), 148),
            (tile(137, 0, 18 * 5), 1),
            (tile(213, 0, 0), 129),
            (tile(215, 0, 0), -6),
            (tile(254, 0, 0), 3),
            (tile(21, 0, 0), 7),
            (tile(21, 1008, 0), -1),
            (tile(21, 612, 0), 11),
            (tile(382, 0, 0), 3),
            (tile(127, 0, 0), 67),
            (tile(6, 0, 0), 8),
            (tile(7, 0, 0), 9),
            (tile(166, 0, 0), 81),
            (tile(227, 34 * 8, 0), 17),
            (tile(203, 0, 0), 117),
            (tile(642, 0, 0), -128),
            (tile(13, 0, 0), 13),
            (tile(13, 90, 0), -1),
            (tile(717, 0, 0), 326),
            (tile(460, 0, 0), 108), // the later 460 arm overwrites 16
            (tile(518, 0, 18), 47),
            (tile(519, 0, 90), 26),
            (tile(636, 0, 0), 17),
            (tile(12, 0, 0), 12),
            (tile(3, 0, 0), 3),
            (tile(22, 0, 0), 14),
            (tile(28, 0, 0), 22),
            (tile(28, 0, 700), 18),
            (tile(163, 0, 0), 118),
            (tile(224, 0, 0), 149),
            (tile(226, 0, 0), 148),
            (tile(29, 0, 0), 23),
            (tile(50, 0, 0), 22),
            (tile(53, 0, 0), 32),
            (tile(57, 0, 0), 36),
            (tile(61, 0, 0), 40),
            (tile(69, 0, 0), 7),
            (tile(70, 0, 0), 17),
            (tile(112, 0, 0), 14),
            (tile(123, 0, 0), 53),
            (tile(165, 54 * 6, 0), 117),
            (tile(666, 0, 0), 322),
            (tile(193, 0, 0), 4),
            (tile(196, 0, 0), 108),
            (tile(116, 0, 0), 51),
            (tile(110, 0, 0), 47),
            (tile(107, 0, 0), 48),
            (tile(133, 54, 0), 146),
            (tile(134, 0, 0), 49),
            (tile(82, 18 * 4, 0), 289), // herb style 4
            (tile(83, 18 * 6, 0), 224),
            (tile(20, 54 * 3, 0), 77),
            (tile(129, 0, 0), 68),
            (tile(129, 18, 0), 69),
            (tile(129, 36, 0), 70),
            (tile(129, 324, 0), 70), // the dead 69 write never survives
            (tile(4, 0, 0), 6),        // torch style 0
            (tile(4, 0, 22 * 14), 66), // torch style 14
            (tile(4, 0, 22 * 99), 310), // clamped to 23
            (tile(321, 0, 0), 214),
            (tile(635, 0, 0), 36),
            (tile(79, 0, 36 * 9), -1), // the final 79/90/101 overwrite
            (tile(33, 0, 0), -1),
        ];
        for (t, want) in cases {
            let got = body(&w, &ctx, &mut rng, &mut gen_rng, &mut dust, t);
            // The 139 arm's spawned type carries the Next(4) roll —
            // pin the family, not the exact byte (the draw census
            // witness below pins the stream).
            if t.kind == 454 {
                assert!((139..=142).contains(&got.unwrap()), "kind 454");
            } else {
                let got_ty = got.unwrap_or(-1);
                assert!(
                    if *want < 0 { got.is_none() } else { got_ty == *want },
                    "kind {} fx {} fy {}: want {want}, got {got:?}",
                    t.kind, t.frame_x, t.frame_y
                );
            }
            drain(&mut dust);
        }
        assert_eq!(gen_rng.peek(), UnifiedRandom::new(1).peek());
    }

    /// The genRand/Main.rand arms draw exactly the C#'s rolls, in
    /// order, with the right polarity — one oracle replay per arm,
    /// read through the tr-core body (the spawned type + both stream
    /// peeks; the spawn block is replayed on the probe so the peeks
    /// stay comparable).
    #[test]
    fn w4_d54_tile_dust_type_random_arms_match_the_oracle() {
        let w = World::new("t".into(), 200, 200);
        let ctx = dust_ctx(&w);
        for seed in [1, 54, 99] {
            let mut rng = UnifiedRandom::new(seed);
            let mut rng_p = UnifiedRandom::new(seed);
            let mut gen_rng = UnifiedRandom::new(seed);
            let mut gen_p = UnifiedRandom::new(seed);
            let mut dust = DustPool::new();
            // (tile, oracle arm id)
            let cases: &[(Tile, u8)] = &[
                (tile(583, 0, 0), 1), (tile(589, 0, 0), 22),
                (tile(633, 0, 0), 2), (tile(634, 0, 0), 3),
                (tile(641, 0, 0), 4), (tile(370, 0, 0), 4),
                (tile(171, 0, 0), 5), (tile(254, 90, 0), 6),
                (tile(254, 120, 0), 7), (tile(254, 200, 0), 8),
                (tile(2, 0, 0), 4), (tile(204, 0, 0), 9),
                (tile(243, 0, 0), 4), (tile(244, 0, 0), 9),
                (tile(597, 0, 0), 10), (tile(275, 0, 0), 11),
                (tile(231, 0, 0), 12), (tile(238, 0, 0), 13),
                (tile(109, 0, 0), 4), (tile(58, 0, 0), 4),
                (tile(37, 0, 0), 4), (tile(32, 0, 0), 4),
                (tile(352, 0, 0), 14), (tile(23, 0, 0), 4),
                (tile(25, 0, 0), 4), (tile(31, 36, 0), 15),
                (tile(26, 54, 0), 16), (tile(27, 0, 0), 4),
                (tile(385, 0, 0), 17), (tile(35, 0, 0), 18),
                (tile(35, 36, 0), 19), (tile(34, 0, 0), 20),
                (tile(19, 0, 18 * 30), 21),
            ];
            for (t, arm) in cases {
                // The 352 num==5 arm FORCES scale 1.5 (the :69105
                // post-write) — the default block's scale roll still
                // draws, so the peek census below is unaffected.
                let got = body(&w, &ctx, &mut rng, &mut gen_rng, &mut dust, t);
                let want = match arm {
                    1 => if gen_p.next_max(10) != 0 { 1 } else { 87 },
                    22 => if gen_p.next_max(10) != 0 { 1 } else { 138 },
                    2 => if gen_p.next_max(6) != 0 { 237 } else { 36 },
                    3 => {
                        let mut n = if gen_p.next_max(10) != 0 { 36 } else { 31 };
                        if gen_p.next_max(12) == 0 { n = 6; }
                        n
                    }
                    4 => {
                        let r = gen_p.next_max(2);
                        match t.kind {
                            641 => if r != 0 { 161 } else { 243 },
                            370 => if r != 0 { 23 } else { 6 },
                            2 | 477 => if r != 0 { 2 } else { 0 },
                            243 => if r != 0 { 13 } else { 7 },
                            109 | 492 => if r != 0 { 47 } else { 0 },
                            58 | 76 | 77 | 684 => if r != 0 { 25 } else { 6 },
                            37 => if r != 0 { 23 } else { 6 },
                            32 => if r != 0 { 24 } else { 14 },
                            23 | 24 | 661 => if r != 0 { 17 } else { 14 },
                            25 => if r != 0 { 1 } else { 14 },
                            27 => if r != 0 { 19 } else { 3 },
                            _ => unreachable!(),
                        }
                    }
                    5 => if gen_p.next_max(2) != 0 { -1 } else { 196 },
                    6 => { let mut n = 3; if gen_p.next_max(3) == 0 { n = 189; } n }
                    7 => { let mut n = 3; if gen_p.next_max(2) == 0 { n = 189; } n }
                    8 => { let mut n = 3; if gen_p.next_max(4) != 0 { n = 189; } n }
                    9 => {
                        // 204: 117 then Next(2)==0 → 1; 244: Next(2)==0 → 1 else 13
                        if t.kind == 244 {
                            if gen_p.next_max(2) == 0 { 1 } else { 13 }
                        } else {
                            let mut n = 117;
                            if gen_p.next_max(2) == 0 { n = 1; }
                            n
                        }
                    }
                    10 => -1, // 597: no roll
                    11 => { let mut n = 13; if gen_p.next_max(3) != 0 { n = -1; } n }
                    12 => { let mut n = 153; if gen_p.next_max(3) == 0 { n = 26; } n }
                    13 => if gen_p.next_max(3) != 0 { 166 } else { 167 },
                    14 => if gen_p.next_max(3) != 0 { 125 } else { 5 },
                    15 => 5,  // 31 fx>=36: no roll
                    16 => 5,  // 26 fx>=54: no roll
                    17 => gen_p.next_range(68, 71),
                    18 => { let mut n = 189; if gen_p.next_max(2) == 0 { n = 6; } n }
                    19 => 189, // 35 fx>=36: the && short-circuit, no roll
                    20 => {
                        // 34: the roll fires, but the final 33|34|42|
                        // 93|100 overwrite (`:69043`) resets to -1.
                        gen_p.next_max(2);
                        -1
                    }
                    21 => 68 + rng_p.next_max(3), // Main.rand!
                    _ => unreachable!(),
                };
                let got_ty = got.unwrap_or(-1);
                assert!(
                    if want < 0 { got.is_none() } else { got_ty == want },
                    "kind {} arm {arm} seed {seed}: want {want}, got {got:?}", t.kind
                );
                // Replay the spawn block on the probe (the negative
                // arms spawned nothing).
                if want >= 0 {
                    block(&mut rng_p, want, 16, 16);
                }
                assert_eq!(rng.peek(), rng_p.peek(), "main kind {}", t.kind);
                assert_eq!(gen_rng.peek(), gen_p.peek(), "gen kind {}", t.kind);
                drain(&mut dust);
            }
        }
    }

    /// `WorldGen.cs:69085-69167` — the spawn arms' draw census: the
    /// disco tile (627), the fountain (518), the gem noLight arm (426),
    /// the blue arm (193), the torch-66 float arm (4 style 14), the
    /// 139 random-type arm (454), the 722 slowed arm, the default (1),
    /// the negative-type refusal (216) and the minecart (314). The
    /// widest spawn-tail census either lane has — now on the tr-core
    /// body (research/66 §3).
    #[test]
    fn w4_d54_make_tile_dust_draws_match_the_oracle() {
        let w = World::new("t".into(), 200, 200);
        let ctx = dust_ctx(&w);
        let disco = [10u8, 20, 30];
        // (tile, expected spawned type, arm id)
        // arm: 0 default, 1 disco-tile, 2 fountain, 7 flag2, 8 blue,
        //      11 torch-float, 12 rand139, 99 no-spawn, 6 minecart
        let cases: &[(Tile, i32, u8)] = &[
            (tile(1, 0, 0), 1, 0),
            (tile(627, 0, 0), 267, 1),
            (tile(518, 0, 0), 3, 2),
            (tile(426, 0, 0), 90, 7),
            (tile(193, 0, 0), 4, 8),
            (tile(4, 0, 22 * 14), 66, 11),
            (tile(454, 0, 0), 139, 12),
            (tile(722, 0, 0), 57, 0),
            (tile(216, 0, 0), -1, 99),
            (tile(314, 0, 0), 213, 6),
        ];
        for (t, ty, arm) in cases {
            let mut rng = UnifiedRandom::new(54);
            let mut rng_p = UnifiedRandom::new(54);
            let mut gen_rng = UnifiedRandom::new(7);
            let mut gen_p = UnifiedRandom::new(7);
            let mut dust = DustPool::new();
            for i in 0..40 {
                tr_core::worldgen_dust_tile::kill_tile_make_tile_dust(
                    10, 20, t, (disco[0], disco[1], disco[2]),
                    &mut rng, &mut gen_rng, &mut dust, &ctx, &w,
                );
                match arm {
                    99 => {}
                    12 => {
                        let ty2 = ty + rng_p.next_max(4);
                        block(&mut rng_p, ty2, 16, 16);
                    }
                    6 => {
                        gen_p.next_range(-2, 3);
                        gen_p.next_range(-2, 3);
                        block(&mut rng_p, 213, 16, 16);
                        gen_p.next_range(0, 51);
                    }
                    _ => block(&mut rng_p, *ty, 16, 16),
                }
                assert_eq!(rng.peek(), rng_p.peek(), "kind {} @{i}", t.kind);
                assert_eq!(gen_rng.peek(), gen_p.peek(), "kind {} gen @{i}", t.kind);
                drain(&mut dust);
            }
        }
    }

    /// The post writes: the disco tile's noGravity + noLightEmittance,
    /// the minecart's fadeIn = scale + 1 + 0.01*roll + the gen velocity
    /// arguments, the 722 velocity ×0.4, the torch-float arm's byte
    /// round-trip (the :2329-2336 assertion research/66 keeps as the
    /// replacement for g14's wrong-semantics witness).
    #[test]
    fn w4_d54_make_tile_dust_writes_the_post_fields() {
        let w = World::new("t".into(), 200, 200);
        let ctx = dust_ctx(&w);
        let disco = (10u8, 20, 30);
        let mut rng = UnifiedRandom::new(54);
        let mut gen_rng = UnifiedRandom::new(7);
        let mut dust = DustPool::new();
        // 627 disco tile.
        tr_core::worldgen_dust_tile::kill_tile_make_tile_dust(
            10, 20, &tile(627, 0, 0), disco, &mut rng, &mut gen_rng, &mut dust, &ctx, &w,
        );
        let d = dust.slot(0);
        assert_eq!(d.dust_type, 267);
        assert_eq!(d.color, Color::new(10, 20, 30, 255));
        assert!(d.no_gravity && d.no_light_emittance);
        drain(&mut dust);
        // 314 minecart: replay the block to recover scale, then fadeIn.
        let mut rng = UnifiedRandom::new(54);
        let mut gen_rng = UnifiedRandom::new(7);
        tr_core::worldgen_dust_tile::kill_tile_make_tile_dust(
            10, 20, &tile(314, 0, 0), disco, &mut rng, &mut gen_rng, &mut dust, &ctx, &w,
        );
        let mut gen_p = UnifiedRandom::new(7);
        let sx = gen_p.next_range(-2, 3) as f32;
        let sy = gen_p.next_range(-2, 3) as f32;
        let mut rng_p = UnifiedRandom::new(54);
        rng_p.next_max(12);
        rng_p.next_max(12);
        let vx = rng_p.next_range(-20, 21) as f32 * 0.1 + sx;
        let vy = rng_p.next_range(-20, 21) as f32 * 0.1 + sy;
        rng_p.next_max(3);
        let scale = 1.0 + rng_p.next_range(-20, 21) as f32 * 0.01;
        let fade_roll = gen_p.next_range(0, 51);
        let d = dust.slot(0);
        assert_eq!(d.dust_type, 213);
        assert!(d.no_gravity);
        assert!((d.velocity.x - vx).abs() < 1e-6);
        assert!((d.velocity.y - vy).abs() < 1e-6);
        assert!((d.fade_in - (scale + 1.0 + 0.01 * fade_roll as f32)).abs() < 1e-5);
        drain(&mut dust);
        // 722 velocity ×0.4.
        let mut rng = UnifiedRandom::new(54);
        tr_core::worldgen_dust_tile::kill_tile_make_tile_dust(
            10, 20, &tile(722, 0, 0), disco, &mut rng, &mut gen_rng, &mut dust, &ctx, &w,
        );
        let mut rng_p = UnifiedRandom::new(54);
        rng_p.next_max(12);
        rng_p.next_max(12);
        let vx = rng_p.next_range(-20, 21) as f32 * 0.1;
        let vy = rng_p.next_range(-20, 21) as f32 * 0.1;
        let d = dust.slot(0);
        assert!((d.velocity.x - vx * 0.4).abs() < 1e-6);
        assert!((d.velocity.y - vy * 0.4).abs() < 1e-6);
        drain(&mut dust);
        // The torch-float arm: the f32 round-trip keeps the bytes.
        let mut rng = UnifiedRandom::new(54);
        tr_core::worldgen_dust_tile::kill_tile_make_tile_dust(
            10, 20, &tile(4, 0, 22 * 14), disco, &mut rng, &mut gen_rng, &mut dust, &ctx, &w,
        );
        let d = dust.slot(0);
        assert_eq!(d.dust_type, 66);
        assert!(d.no_gravity);
        let ch = |b: u8| (b as f32 / 255.0 * 255.0).clamp(0.0, 255.0) as u8;
        assert_eq!(d.color, Color::new(ch(10), ch(20), ch(30), 255));
    }

    /// `WorldGen.cs:63699-63703` — the break loop runs exactly
    /// `tile_dust_amount` bodies; a zero-amount kind draws nothing.
    #[test]
    fn w4_d54_tile_break_burst_counts_the_amount() {
        let w = World::new("t".into(), 200, 200);
        let ctx = dust_ctx(&w);
        let disco = [0u8; 3];
        let mut rng = UnifiedRandom::new(54);
        let mut rng_p = UnifiedRandom::new(54);
        let mut gen_rng = UnifiedRandom::new(7);
        let mut dust = DustPool::new();
        let t = tile(1, 0, 0);
        for i in 0..20 {
            tile_break_burst(&mut rng, &mut gen_rng, disco, &mut dust, &ctx, &w, 10, 20, &t, false);
            for _ in 0..10 {
                block(&mut rng_p, 1, 16, 16);
            }
            assert_eq!(rng.peek(), rng_p.peek(), "stone burst @{i}");
            drain(&mut dust);
        }
        // The zero-amount kind (654) draws nothing.
        let mut rng = UnifiedRandom::new(54);
        tile_break_burst(&mut rng, &mut gen_rng, disco, &mut dust, &ctx, &w, 10, 20, &tile(654, 0, 0), false);
        assert_eq!(rng.peek(), UnifiedRandom::new(54).peek());
    }
}
