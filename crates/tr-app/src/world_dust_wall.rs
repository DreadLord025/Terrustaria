//! W4-d54 — the `KillWall` / `ReplaceWall` dust-loop WRAPPERS. The
//! wall→dust type ladder (`WorldGen.cs:61345-62136`) and the four-arm
//! spawn tail (`:62138-62157`) now live in
//! `tr_core::worldgen_dust_wall::kill_wall_make_wall_dust` (g14's
//! canonical port, research/66 union exec — the ladders' 11-gen
//! multisets verified identical, the tails arm-for-arm identical;
//! g14 returns the slot, d54's void shape was cosmetic). The
//! `PlaceTile` 22/140 puff is `tr_core::worldgen_dust::
//! place_tile_ore_dust` (the caller-side 22/140 gate stays HERE, at
//! the seam — research/66 §2 "pick one at the seam").
//!
//! Live seams (the C# orders):
//! - `break_wall` (`editing.rs`, `WorldGen.cs:60385-60394`): sound →
//!   dust → drop, ten bodies.
//! - `place_wall_route`'s replacement arm (`:62545-62551`): dust →
//!   sound → drop, ten bodies on the OLD wall.
//! - `PlaceTile` (`:60283-60289`): the 22/140 three-type-14 puff AFTER
//!   the placement sound switch.

use tr_core::particles::{DustPool, SpawnCtx};
use tr_core::{UnifiedRandom, World};

/// The `KillWall` / `ReplaceWall` dust loop: `count` calls of the body
/// (`:60391-60394` — 10 on a break, 3 on a fail; `:62545-62549` — 10 on
/// a replacement). `gen` is the caller's `frame_rng` snapshot.
#[allow(clippy::too_many_arguments)]
pub(crate) fn wall_break_burst(
    rng: &mut UnifiedRandom,
    gen: &mut UnifiedRandom,
    disco: [u8; 3],
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
    i: i32,
    j: i32,
    wall: u16,
    count: i32,
) {
    for _ in 0..count {
        tr_core::worldgen_dust_wall::kill_wall_make_wall_dust(
            i,
            j,
            wall,
            (disco[0], disco[1], disco[2]),
            rng,
            gen,
            dust,
            ctx,
            world,
        );
    }
}

/// `WorldGen.PlaceTile`'s torch/campfire puff (`WorldGen.cs:60283-
/// 60289`): on a successful UNMUTED placement of kind 22 (campfire) or
/// 140 (torch-on-a-platform… the C# reads the placed `num`), three
/// type-14 puffs at the tile corner, AFTER the placement sound switch.
/// The 22/140 gate is the caller's per the tr-core contract — kept
/// here at the seam (research/66 §2). Type 14 adds no per-type tweak.
pub(crate) fn place_tile_torch_dust(
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
    i: i32,
    j: i32,
    kind: u16,
) {
    if !matches!(kind, 22 | 140) {
        return;
    }
    tr_core::worldgen_dust::place_tile_ore_dust(i, j, rng, dust, ctx, world);
}

#[cfg(test)]
mod tests {
    //! W4-d54 batch-2 dust witnesses — WRITTEN, NOT RUN (code-only).
    //! Re-pointed at the union survivors (research/66 §3): the type
    //! ladder oracles read the tr-core body's spawned type through
    //! its slot return (the ladder chunks are private there); the
    //! 116-135 silent band and the negative veto surface as the
    //! sentinel.

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

    /// The default block + the FULL per-type tweak family
    /// (`Dust.cs:214-220`): only the 228/279/269/135/6/242/75/169/29/
    /// 59..65/158/293..298/302/307/310 family adds the `Next(-10,6)`.
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

    /// One tr-core wall-body call; the resolved type or None on the
    /// sentinel (the band silence / negative veto).
    fn body(
        w: &World,
        ctx: &SpawnCtx,
        rng: &mut UnifiedRandom,
        gen: &mut UnifiedRandom,
        dust: &mut DustPool,
        wall: u16,
    ) -> Option<i32> {
        let idx = tr_core::worldgen_dust_wall::kill_wall_make_wall_dust(
            10, 20, wall, (0, 0, 0), rng, gen, dust, ctx, w,
        );
        if idx == DUST_SENTINEL {
            None
        } else {
            Some(dust.slot(idx).dust_type)
        }
    }

    /// The type chain's constant arms pinned to the C# literals (no
    /// genRand draws) — a spread across the switch, the overwrite chain
    /// and the later-wins order. Read through the tr-core body.
    #[test]
    fn w4_d54_wall_dust_type_matches_the_csharp_table() {
        let w = World::new("t".into(), 200, 200);
        let ctx = dust_ctx(&w);
        let mut rng = UnifiedRandom::new(1);
        let mut gen = UnifiedRandom::new(1);
        let mut dust = DustPool::new();
        let cases: &[(u16, i32)] = &[
            (0, 0), (366, 1), (365, 13), (364, 10), (363, 148), (362, 26),
            (361, 78), (360, 236), (359, 96), (353, 196), (357, 68),
            (354, 277), (351, 0), (348, 57), (316, 36), (317, 36),
            (7, 275), (331, 275), (8, 276), (332, 276), (9, 277), (333, 277),
            (232, 82), (231, 8), (233, 18), (237, 6), (238, 61), (239, 242),
            (240, 135), (15, 38), (247, 38), (22, 51), (248, 51), (40, 51),
            (48, 1), (58, 1), (185, 1), (260, 1), (274, 1), (314, 1),
            (59, 0), (262, 0), (62, 0), (70, 47), (71, 80), (79, 37),
            (81, 5), (268, 123), (170, 0), (187, 0), (188, 37), (279, 37),
            (192, 117), (283, 117), (196, 0), (287, 0), (201, 17),
            (204, 0), (206, 1), (212, 1), (216, 0), (217, 37), (218, 155),
            (219, 17), (220, 37), (221, 155), (222, 37), (223, 0),
            (312, -1), (313, -1), (315, -1),
            // The overwrite chain + later-wins:
            (148, -1), (1, 1), (349, 1), (35, 37), (4, 7), (12, 9),
            (336, 9), (10, 10), (334, 10), (11, 11), (335, 11), (21, 13),
            (34, 32), (225, 1), (145, 8), (23, 38), (24, 36), (25, 48),
            (179, 236), (181, 240), (113, 189), (114, 190), (115, 191),
            (177, 25), (319, 171), (321, 308), (318, 226), (142, 210),
            (224, 265), (323, 314), (330, 321), (320, 0), (341, 258),
            (342, 301), (343, 299), (344, 300), (345, 305), (173, 128),
            (174, 117), (175, 42), (176, 226),
            (154, 86), (163, 87), (162, 88), (156, 89), (161, 90),
            (155, 91), (153, 138),
            (26, 49), (29, 50), (31, 51), (14, 109), (241, 286),
            (88, 86), (89, 87), (90, 88), (91, 89), (92, 90),
            (33, 14), (41, 77), (42, 78), (36, 26), (37, 32), (38, 2),
            (39, 1), (45, 81), (46, 83), (47, 84), (85, 126), (63, 3),
            (64, 40), (84, 80), (60, 3), (167, 81), (147, 51), (146, 9),
            (109, 144), (110, 145), (111, 146), (86, 147), (87, 148),
            (136, 13), (243, 13), (72, 40), (73, 16), (74, 26), (75, 26),
            (76, 4), (77, 5), (78, 7), (82, 36), (138, 77), (139, 78),
            (140, 79), (141, 126), (149, 214), (151, 215), (245, 195),
        ];
        for &(wall, want) in cases {
            let got = body(&w, &ctx, &mut rng, &mut gen, &mut dust, wall);
            let got_ty = got.unwrap_or(-1);
            assert!(
                if want < 0 { got.is_none() } else { got_ty == want },
                "wall {wall}: want {want}, got {got:?}"
            );
            drain(&mut dust);
        }
        // The constant cases drew NOTHING on gen.
        assert_eq!(gen.peek(), UnifiedRandom::new(1).peek());
    }

    /// The genRand arms draw exactly one roll each, with the C#'s
    /// bounds and branch polarity (`:61455`/`:61509`/`:61530`/`:61576`/
    /// `:61584`/`:61606`/`:61750`/`:61854`/`:61915`/`:62084`/`:62108`).
    /// The spawn block is replayed on the rng probe so both stream
    /// peeks stay comparable.
    #[test]
    fn w4_d54_wall_dust_type_genrand_arms_match_the_oracle() {
        let w = World::new("t".into(), 200, 200);
        let ctx = dust_ctx(&w);
        // (wall, oracle draw) — the oracle replays the roll and derives
        // the type with the C#'s polarity.
        for seed in [1, 54, 99] {
            let mut rng = UnifiedRandom::new(seed);
            let mut rng_p = UnifiedRandom::new(seed);
            let mut gen = UnifiedRandom::new(seed);
            let mut probe = UnifiedRandom::new(seed);
            let mut dust = DustPool::new();
            let cases: &[u16] = &[3, 246, 69, 264, 83, 234, 269, 200, 202, 288,
                290, 203, 291, 208, 299, 186, 182, 93, 144, 27];
            for &wall in cases {
                let got = body(&w, &ctx, &mut rng, &mut gen, &mut dust, wall);
                let want = match wall {
                    3 | 246 => if probe.next_max(2) != 0 { 1 } else { 14 },
                    69 | 264 => if probe.next_max(2) != 0 { 17 } else { 14 },
                    83 | 234 | 269 => if probe.next_max(2) == 0 { 1 } else { 117 },
                    200 | 202 | 288 | 290 => if probe.next_max(2) != 0 { 70 } else { 69 },
                    203 | 291 => if probe.next_max(2) != 0 { 68 } else { 69 },
                    208..=211 | 296..=299 => if probe.next_max(2) != 0 { 125 } else { 155 },
                    186 => probe.next_range(68, 71),
                    182 => if probe.next_max(2) != 0 { 23 } else { 6 },
                    93 => probe.next_range(88, 94),
                    144 => if probe.next_max(2) != 0 { 118 } else { 10 },
                    27 => if probe.next_max(2) != 0 { 1 } else { 7 },
                    _ => unreachable!(),
                };
                assert_eq!(got, Some(want), "wall {wall} seed {seed}");
                block(&mut rng_p, want, 16, 16);
                assert_eq!(gen.peek(), probe.peek(), "gen stream wall {wall}");
                assert_eq!(rng.peek(), rng_p.peek(), "rng stream wall {wall}");
                drain(&mut dust);
            }
        }
    }

    /// `WorldGen.cs:62138-62157` — the four spawn arms' draw census:
    /// 348 (type 57 block + no post draws), 44/346 (type 66 block),
    /// 76 (type 4 block), a plain wall (type 1 block), a -1 wall (no
    /// spawn) and the 116..=135 band (no spawn). The gen stream stays
    /// untouched for these constant-type walls.
    #[test]
    fn w4_d54_make_wall_dust_draws_match_the_oracle() {
        let w = World::new("t".into(), 200, 200);
        let ctx = dust_ctx(&w);
        let disco = [10u8, 20, 30];
        // (wall, spawned type or None)
        let cases: &[(u16, Option<i32>)] = &[
            (348, Some(57)), (44, Some(66)), (346, Some(66)), (76, Some(4)),
            (1, Some(1)), (2, Some(0)), (312, None), (116, None),
            (125, None), (126, None), (132, None), (133, None), (135, None),
            (136, Some(13)),
        ];
        for &(wall, ty) in cases {
            let mut rng = UnifiedRandom::new(54);
            let mut probe = UnifiedRandom::new(54);
            let mut gen = UnifiedRandom::new(7);
            let mut dust = DustPool::new();
            for i in 0..50 {
                wall_break_burst(&mut rng, &mut gen, disco, &mut dust, &ctx, &w, 10, 20, wall, 1);
                if let Some(t) = ty {
                    block(&mut probe, t, 16, 16);
                }
                assert_eq!(rng.peek(), probe.peek(), "wall {wall} @{i}");
                drain(&mut dust);
            }
            assert_eq!(gen.peek(), UnifiedRandom::new(7).peek(), "wall {wall} gen");
        }
    }

    /// The post writes: 348's velocity ×0.4, the disco arm's colour +
    /// noGravity + 0.75 scale argument, the 76 blue arm's colour/alpha.
    #[test]
    fn w4_d54_make_wall_dust_writes_the_post_fields() {
        let w = World::new("t".into(), 200, 200);
        let ctx = dust_ctx(&w);
        let disco = [10u8, 20, 30];
        let mut rng = UnifiedRandom::new(54);
        let mut gen = UnifiedRandom::new(7);
        let mut dust = DustPool::new();
        wall_break_burst(&mut rng, &mut gen, disco, &mut dust, &ctx, &w, 10, 20, 348, 1);
        let mut probe = UnifiedRandom::new(54);
        probe.next_max(12);
        probe.next_max(12);
        let vx = probe.next_range(-20, 21) as f32 * 0.1;
        let vy = probe.next_range(-20, 21) as f32 * 0.1;
        let d = dust.slot(0);
        assert_eq!(d.dust_type, 57);
        assert!((d.velocity.x - vx * 0.4).abs() < 1e-6);
        assert!((d.velocity.y - vy * 0.4).abs() < 1e-6);
        drain(&mut dust);
        let mut rng = UnifiedRandom::new(54);
        wall_break_burst(&mut rng, &mut gen, disco, &mut dust, &ctx, &w, 10, 20, 44, 1);
        let d = dust.slot(0);
        assert_eq!(d.dust_type, 66);
        assert_eq!(d.color, Color::new(10, 20, 30, 255));
        assert_eq!(d.alpha, 100);
        assert!(d.no_gravity);
        drain(&mut dust);
        let mut rng = UnifiedRandom::new(54);
        wall_break_burst(&mut rng, &mut gen, disco, &mut dust, &ctx, &w, 10, 20, 76, 1);
        let d = dust.slot(0);
        assert_eq!(d.dust_type, 4);
        assert_eq!(d.color, Color::new(0, 80, 255, 100));
        assert_eq!(d.alpha, 75);
        assert!(!d.no_gravity);
    }

    /// `WorldGen.cs:60283-60289` — the PlaceTile puff is exactly three
    /// type-14 blocks for kinds 22/140 and NOTHING for every other kind.
    #[test]
    fn w4_d54_place_tile_torch_dust_draws_match_the_oracle() {
        let w = World::new("t".into(), 200, 200);
        let ctx = dust_ctx(&w);
        for kind in [22u16, 140, 4, 1, 0] {
            let mut rng = UnifiedRandom::new(54);
            let mut probe = UnifiedRandom::new(54);
            let mut dust = DustPool::new();
            for i in 0..50 {
                place_tile_torch_dust(&mut rng, &mut dust, &ctx, &w, 10, 20, kind);
                if matches!(kind, 22 | 140) {
                    for _ in 0..3 {
                        block(&mut probe, 14, 16, 16);
                    }
                }
                assert_eq!(rng.peek(), probe.peek(), "kind {kind} @{i}");
                drain(&mut dust);
            }
        }
    }
}
