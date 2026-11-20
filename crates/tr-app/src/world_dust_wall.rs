//! W4-d54 batch 2 — `WorldGen.KillWall_MakeWallDust`
//! (`WorldGen.cs:61345-62158`, sites 62140/62144/62151/62155) and the
//! `PlaceTile` torch/campfire puff (`:60283-60289`, site 60287).
//!
//! The wall body is the hammer-break / wall-replacement dust: the C#
//! resolves a dust TYPE from the wall id (a switch, then a long
//! overwrite-if chain — later arms win), then one of four spawn arms.
//! The `genRand` rolls inside the type chain ride the port's runtime
//! `genRand` stand-in (`World::frame_rng`, the caller passes a snapshot
//! and writes it back — the repo's borrow-snapshot precedent); the
//! spawn's default block rides `Main.rand` (`UpdatePlayers`).
//!
//! Callers (the C# order):
//! - `KillWall` (`:60385-60394`): sound, then 10 × this body (3 on a
//!   fail tick — the port's fail path is the recorded i29 deviation),
//!   then the drop. Wired in `editing.rs::break_wall`.
//! - `ReplaceWall` (`:62545-62551`): 10 × this body BEFORE the sound and
//!   the drop. Wired in `editing.rs::place_wall_route`.
//!
//! The disco colour arms read `Main.DiscoR/G/B` — the caller passes the
//! App's live `disco` triple (`app/mod.rs:840`, `DoUpdate_AnimateDiscoRGB`).

use glam::Vec2;
use tr_core::particles::{Color, DustPool, SpawnCtx};
use tr_core::{UnifiedRandom, World, TILE_PX};

/// The wall→dust type chain (`WorldGen.cs:61347-62137`): the switch
/// (`:61348-61655`) then the overwrite-if chain (`:61656-62137`), in
/// source order — a later arm replaces an earlier one. `gen` is the
/// `genRand` stand-in; only the marked arms draw.
pub(crate) fn wall_dust_type(wall: u16, gen: &mut UnifiedRandom) -> i32 {
    let mut num = 0i32;
    // `:61348-61655` — the switch.
    match wall {
        366 => num = 1,
        365 => num = 13,
        364 => num = 10,
        363 => num = 148,
        362 => num = 26,
        361 => num = 78,
        360 => num = 236,
        359 => num = 96,
        358 => num = 78,
        353 => num = 196,
        357 => num = 68,
        352 => num = 10,
        354 => num = 277,
        355 => num = 26,
        356 => num = 277,
        351 => num = 0,
        348 => num = 57,
        316 | 317 => num = 36,
        7 | 17 | 94 | 95 | 100 | 101 | 331 => num = 275,
        8 | 18 | 98 | 99 | 104 | 105 | 332 => num = 276,
        9 | 19 | 96 | 97 | 102 | 103 | 333 => num = 277,
        232 => num = 82,
        231 => num = 8,
        233 => num = 18,
        237 => num = 6,
        238 => num = 61,
        239 => num = 242,
        240 => num = 135,
        3 | 246 => num = if gen.next_max(2) != 0 { 1 } else { 14 },
        15 | 247 => num = 38,
        22 | 28 | 248 => num = 51,
        40 | 249 => num = 51,
        48..=58 | 185 | 250..=260 | 274 | 314 => num = 1,
        59 | 61 | 261 | 262 => num = 0,
        62 | 263 => num = 0,
        69 | 264 => num = if gen.next_max(2) != 0 { 17 } else { 14 },
        70 | 265 => num = 47,
        71 | 266 => num = 80,
        79 | 267 => num = 37,
        81 | 268 => num = 123,
        83 | 234 | 269 => num = if gen.next_max(2) == 0 { 1 } else { 117 },
        170 | 171 | 270 | 271 => num = 0,
        187 | 275 => num = 0,
        188..=191 | 276..=279 => num = 37,
        192..=195 | 280..=283 => num = 117,
        196..=199 | 284..=287 => num = 0,
        200 | 202 | 288 | 290 => num = if gen.next_max(2) != 0 { 70 } else { 69 },
        201 | 289 => num = 17,
        203 | 291 => num = if gen.next_max(2) != 0 { 68 } else { 69 },
        204 | 205 | 207 | 292 | 293 | 295 => num = 0,
        206 | 294 => num = 1,
        208..=211 | 296..=299 => num = if gen.next_max(2) != 0 { 125 } else { 155 },
        212..=215 | 300..=303 => num = 1,
        216 | 304 => num = 0,
        217 | 305 => num = 37,
        218 | 306 => num = 155,
        219 | 307 => num = 17,
        220 | 308 => num = 37,
        221 | 309 => num = 155,
        222 | 310 => num = 37,
        223 | 311 => num = 0,
        312 | 313 | 315 => num = -1,
        _ => {}
    }
    // `:61656-62137` — the overwrite-if chain, source order.
    if wall == 148 {
        num = -1;
    }
    if matches!(wall, 1 | 5 | 6 | 107 | 349) {
        num = 1;
    }
    if wall == 35 {
        num = 37;
    }
    if matches!(wall, 4 | 106) {
        num = 7;
    }
    if wall == 12 {
        num = 9;
    }
    if wall == 336 {
        num = 9;
    }
    if wall == 10 {
        num = 10;
    }
    if wall == 334 {
        num = 10;
    }
    if wall == 11 {
        num = 11;
    }
    if wall == 335 {
        num = 11;
    }
    if wall == 21 {
        num = 13;
    }
    if wall == 34 {
        num = 32;
    }
    if wall == 225 {
        num = 1;
    }
    if wall == 145 {
        num = 8;
    }
    if wall == 23 {
        num = 38;
    }
    if wall == 24 {
        num = 36;
    }
    if matches!(wall, 25 | 339) {
        num = 48;
    }
    if matches!(wall, 179 | 178 | 183) {
        num = 236;
    }
    if matches!(wall, 181 | 180 | 184) {
        num = 240;
    }
    if wall == 113 {
        num = 189;
    }
    if wall == 114 {
        num = 190;
    }
    if wall == 115 {
        num = 191;
    }
    if matches!(wall, 177 | 13 | 338) {
        num = 25;
    }
    if wall == 186 {
        num = gen.next_range(68, 71);
    }
    if wall == 319 {
        num = 171;
    }
    if wall == 321 {
        num = 308;
    }
    if wall == 322 {
        num = 308;
    }
    if wall == 318 {
        num = 226;
    }
    if wall == 142 {
        num = 210;
    }
    if wall == 143 {
        num = 210;
    }
    if wall == 224 {
        num = 265;
    }
    if wall == 323 {
        num = 314;
    }
    if wall == 324 {
        num = 315;
    }
    if wall == 325 {
        num = 316;
    }
    if wall == 326 {
        num = 317;
    }
    if wall == 327 {
        num = 318;
    }
    if wall == 328 {
        num = 319;
    }
    if wall == 329 {
        num = 320;
    }
    if wall == 330 {
        num = 321;
    }
    if wall == 320 {
        num = 0;
    }
    if wall == 341 {
        num = 258;
    }
    if wall == 342 {
        num = 301;
    }
    if wall == 343 {
        num = 299;
    }
    if wall == 344 {
        num = 300;
    }
    if wall == 345 {
        num = 305;
    }
    if wall == 173 {
        num = 128;
    }
    if wall == 174 {
        num = 117;
    }
    if wall == 175 {
        num = 42;
    }
    if wall == 176 {
        num = 226;
    }
    if wall == 182 {
        num = if gen.next_max(2) != 0 { 23 } else { 6 };
    }
    // `:61856-61889` — the 153..=166 range's inner switch (the other
    // ids in the range keep the prior num).
    if (153..=166).contains(&wall) {
        match wall {
            154 | 158 => num = 86,
            163 | 166 => num = 87,
            162 | 165 => num = 88,
            156 | 160 => num = 89,
            161 | 164 => num = 90,
            155 | 159 => num = 91,
            153 | 157 => num = 138,
            _ => {}
        }
    }
    if matches!(wall, 26 | 30 | 340) {
        num = 49;
    }
    if matches!(wall, 29 | 32) {
        num = 50;
    }
    if wall == 31 {
        num = 51;
    }
    if matches!(wall, 14 | 20 | 337) {
        num = 109;
    }
    if wall == 241 {
        num = 286;
    }
    // `:61910-61917` — the 88..=93 gem walls: 86 + wall - 88, then the
    // 93 arm REPLACES it with a Next(88,94) roll.
    if (88..=93).contains(&wall) {
        num = 86 + i32::from(wall) - 88;
        if wall == 93 {
            num = gen.next_range(88, 94);
        }
    }
    if wall == 33 {
        num = 14;
    }
    if wall == 41 {
        num = 77;
    }
    if wall == 42 {
        num = 78;
    }
    if wall == 43 {
        num = 78;
    }
    if wall == 36 {
        num = 26;
    }
    if wall == 37 {
        num = 32;
    }
    if wall == 38 {
        num = 2;
    }
    if wall == 39 {
        num = 1;
    }
    if wall == 45 {
        num = 81;
    }
    if wall == 46 {
        num = 83;
    }
    if wall == 47 {
        num = 84;
    }
    if wall == 85 {
        num = 126;
    }
    if wall == 63 {
        num = 3;
    }
    if wall == 65 {
        num = 3;
    }
    if wall == 66 {
        num = 3;
    }
    if wall == 68 {
        num = 3;
    }
    if wall == 64 {
        num = 40;
    }
    if wall == 67 {
        num = 40;
    }
    if wall == 84 {
        num = 80;
    }
    if wall == 60 {
        num = 3;
    }
    if wall == 167 {
        num = 81;
    }
    if wall == 147 {
        num = 51;
    }
    if wall == 146 {
        num = 9;
    }
    if wall == 109 {
        num = 144;
    }
    if wall == 110 {
        num = 145;
    }
    if wall == 111 {
        num = 146;
    }
    if matches!(wall, 86 | 108) {
        num = 147;
    }
    if wall == 87 {
        num = 148;
    }
    if matches!(wall, 136 | 137 | 168 | 169 | 172 | 226 | 227 | 347 | 242 | 243) {
        num = 13;
    }
    if wall == 72 {
        num = 40;
    }
    if wall == 73 {
        num = 16;
    }
    if matches!(wall, 74 | 80) {
        num = 26;
    }
    if wall == 144 {
        num = if gen.next_max(2) != 0 { 118 } else { 10 };
    }
    if wall == 75 {
        num = 26;
    }
    if wall == 76 {
        num = 4;
    }
    if matches!(wall, 77 | 81) {
        num = 5;
    }
    if matches!(wall, 78 | 244) {
        num = 7;
    }
    if wall == 82 {
        num = 36;
    }
    if wall == 27 {
        num = if gen.next_max(2) != 0 { 1 } else { 7 };
    }
    if wall == 138 {
        num = 77;
    }
    if wall == 139 {
        num = 78;
    }
    if wall == 140 {
        num = 79;
    }
    if wall == 141 {
        num = 126;
    }
    if matches!(wall, 149 | 150) {
        num = 214;
    }
    if matches!(wall, 151 | 152) {
        num = 215;
    }
    if wall == 245 {
        num = 195;
    }
    num
}

/// One `KillWall_MakeWallDust` call (`WorldGen.cs:62138-62157`): the
/// type chain, then the four spawn arms. `disco` is `Main.DiscoR/G/B`
/// (the 3-byte `Color` ctor keeps alpha 255). The sentinel slot takes
/// the post writes like every C# `Main.dust[num]` shape.
#[allow(clippy::too_many_arguments)]
pub(crate) fn make_wall_dust(
    rng: &mut UnifiedRandom,
    gen: &mut UnifiedRandom,
    disco: [u8; 3],
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
    i: i32,
    j: i32,
    wall: u16,
) {
    let num = wall_dust_type(wall, gen);
    let pos = Vec2::new(i as f32 * TILE_PX, j as f32 * TILE_PX);
    if wall == 348 {
        // `:62138-62141` — the shimmer wall's slowed puff.
        let idx = dust.new_dust(rng, ctx, world, pos, 16, 16, num, 0.0, 0.0, 0, Color::default(), 1.0);
        dust.slot_mut(idx).velocity *= 0.4;
    } else if matches!(wall, 44 | 346) {
        // `:62142-62146` — the disco arm: type 66, alpha 100, the live
        // disco colour, scale 0.75, noGravity.
        let idx = dust.new_dust(
            rng, ctx, world, pos, 16, 16, 66, 0.0, 0.0, 100,
            Color::new(disco[0], disco[1], disco[2], 255), 0.75,
        );
        dust.slot_mut(idx).no_gravity = true;
    } else if (wall < 133 || wall > 135) && (wall < 116 || wall > 125) && (wall < 126 || wall > 132)
    {
        // `:62147-62157` — the 116..=135 unsafe-wall band spawns
        // NOTHING; every other wall takes the 76 blue arm or the
        // default (num >= 0 gate — a -1 type spawns nothing).
        if wall == 76 {
            dust.new_dust(
                rng, ctx, world, pos, 16, 16, num, 0.0, 0.0, 75,
                Color::new(0, 80, 255, 100), 0.75,
            );
        } else if num >= 0 {
            dust.new_dust(rng, ctx, world, pos, 16, 16, num, 0.0, 0.0, 0, Color::default(), 1.0);
        }
    }
}

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
        make_wall_dust(rng, gen, disco, dust, ctx, world, i, j, wall);
    }
}

/// `WorldGen.PlaceTile`'s torch/campfire puff (`WorldGen.cs:60283-
/// 60289`): on a successful UNMUTED placement of kind 22 (campfire) or
/// 140 (torch-on-a-platform… the C# reads the placed `num`), three
/// type-14 puffs at the tile corner, AFTER the placement sound switch.
/// Type 14 adds no per-type tweak.
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
    let pos = Vec2::new(i as f32 * TILE_PX, j as f32 * TILE_PX);
    for _ in 0..3 {
        dust.new_dust(rng, ctx, world, pos, 16, 16, 14, 0.0, 0.0, 0, Color::default(), 1.0);
    }
}

#[cfg(test)]
mod tests {
    //! W4-d54 batch-2 dust witnesses — WRITTEN, NOT RUN (code-only).

    use super::*;

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

    /// The type chain's constant arms pinned to the C# literals (no
    /// genRand draws) — a spread across the switch, the overwrite chain
    /// and the later-wins order.
    #[test]
    fn w4_d54_wall_dust_type_matches_the_csharp_table() {
        let mut gen = UnifiedRandom::new(1);
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
            assert_eq!(wall_dust_type(wall, &mut gen), want, "wall {wall}");
        }
        // The constant cases drew NOTHING on gen.
        assert_eq!(gen.peek(), UnifiedRandom::new(1).peek());
    }

    /// The genRand arms draw exactly one roll each, with the C#'s
    /// bounds and branch polarity (`:61455`/`:61509`/`:61530`/`:61576`/
    /// `:61584`/`:61606`/`:61750`/`:61854`/`:61915`/`:62084`/`:62108`).
    #[test]
    fn w4_d54_wall_dust_type_genrand_arms_match_the_oracle() {
        // (wall, oracle draw) — the oracle replays the roll and derives
        // the type with the C#'s polarity.
        for seed in [1, 54, 99] {
            let mut gen = UnifiedRandom::new(seed);
            let mut probe = UnifiedRandom::new(seed);
            let cases: &[u16] = &[3, 246, 69, 264, 83, 234, 269, 200, 202, 288,
                290, 203, 291, 208, 299, 186, 182, 93, 144, 27];
            for &wall in cases {
                let got = wall_dust_type(wall, &mut gen);
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
                assert_eq!(got, want, "wall {wall} seed {seed}");
                assert_eq!(gen.peek(), probe.peek(), "gen stream wall {wall}");
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
                make_wall_dust(&mut rng, &mut gen, disco, &mut dust, &ctx, &w, 10, 20, wall);
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
        make_wall_dust(&mut rng, &mut gen, disco, &mut dust, &ctx, &w, 10, 20, 348);
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
        make_wall_dust(&mut rng, &mut gen, disco, &mut dust, &ctx, &w, 10, 20, 44);
        let d = dust.slot(0);
        assert_eq!(d.dust_type, 66);
        assert_eq!(d.color, Color::new(10, 20, 30, 255));
        assert_eq!(d.alpha, 100);
        assert!(d.no_gravity);
        drain(&mut dust);
        let mut rng = UnifiedRandom::new(54);
        make_wall_dust(&mut rng, &mut gen, disco, &mut dust, &ctx, &w, 10, 20, 76);
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
