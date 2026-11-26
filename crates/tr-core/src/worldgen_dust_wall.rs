//! W4-g14 batch 2 — `WorldGen.KillWall_MakeWallDust`
//! (`WorldGen.cs:61345-62158`), sites 62140/62144/62151/62155 of the
//! research/65 census, plus the `PlaceTile` kind-22/140 arm (site
//! 60287) which lives in `worldgen_dust.rs`.
//!
//! The wall dust-TYPE ladder is RNG-bearing (eleven `WorldGen.genRand`
//! draws: the 3/246, 69/264, 83/234/269, 200/202/288/290, 203/291,
//! 208-211/296-299 switch arms, the 186 `Next(68,71)`, the 182, 93
//! `Next(88,94)`, 144 and 27 if-chain arms) — it ports verbatim so the
//! draw ORDER survives even though the pool's generating guard makes
//! the generation-time spawns themselves no-ops.
//!
//! Two streams (research/65): `gen` = `WorldGen.genRand` (the ladder
//! rolls), `rng` = `Main.rand` (the `NewDust` internals). The disco
//! colour (`Main.DiscoR/G/B`) is a caller snapshot — the App owns the
//! cycle (`app/mod.rs:839`), tr-core takes the bytes.
//!
//! The spawn tail (`:62138-62157`) is a four-way if/else chain:
//! wall 348 → velocity ×0.4; wall 44/346 → the type-66 disco puff
//! (alpha 100, scale 0.75, noGravity); the range-gated generic arm →
//! wall 76's blue-tinted 0.75 or the plain `num >= 0` spawn. Walls
//! 116-135 (the gemsparkle family) spawn NOTHING here.
//!
//! WRITTEN NOT RUN (code-only mode).

use crate::particles::{Color, DustPool, SpawnCtx};
use crate::unified_random::UnifiedRandom;
use crate::world::World;
use glam::Vec2;

/// The wall dust-type ladder, switch half (`WorldGen.cs:61347-61655`):
/// `num = 0`, then the big switch over `tileCache.wall`. The six
/// genRand arms roll `gen` exactly where the C# evaluates them.
fn wall_dust_type_switch(wall: u16, gen_rng: &mut UnifiedRandom) -> i32 {
    match wall {
        366 => 1,
        365 => 13,
        364 => 10,
        363 => 148,
        362 => 26,
        361 => 78,
        360 => 236,
        359 => 96,
        358 => 78,
        353 => 196,
        357 => 68,
        352 => 10,
        354 | 356 => 277,
        355 => 26,
        351 => 0,
        348 => 57,
        316 | 317 => 36,
        7 | 17 | 94 | 95 | 100 | 101 | 331 => 275,
        8 | 18 | 98 | 99 | 104 | 105 | 332 => 276,
        9 | 19 | 96 | 97 | 102 | 103 | 333 => 277,
        232 => 82,
        231 => 8,
        233 => 18,
        237 => 6,
        238 => 61,
        239 => 242,
        240 => 135,
        // `:61453-61456` — genRand arm.
        3 | 246 => {
            if gen_rng.next_max(2) != 0 {
                1
            } else {
                14
            }
        }
        15 | 247 => 38,
        22 | 28 | 248 => 51,
        40 | 249 => 51,
        48..=58 | 185 | 250..=260 | 274 | 314 => 1,
        59 | 61 | 261 | 262 => 0,
        62 | 263 => 0,
        // `:61507-61510` — genRand arm.
        69 | 264 => {
            if gen_rng.next_max(2) != 0 {
                17
            } else {
                14
            }
        }
        70 | 265 => 47,
        71 | 266 => 80,
        79 | 267 => 37,
        81 | 268 => 123,
        // `:61527-61531` — genRand arm (the == 0 polarity).
        83 | 234 | 269 => {
            if gen_rng.next_max(2) == 0 {
                1
            } else {
                117
            }
        }
        170 | 171 | 270 | 271 => 0,
        187 | 275 => 0,
        188..=191 | 276..=279 => 37,
        192..=195 | 280..=283 => 117,
        196..=199 | 284..=287 => 0,
        // `:61572-61577` — genRand arm.
        200 | 202 | 288 | 290 => {
            if gen_rng.next_max(2) != 0 {
                70
            } else {
                69
            }
        }
        201 | 289 => 17,
        // `:61582-61585` — genRand arm.
        203 | 291 => {
            if gen_rng.next_max(2) != 0 {
                68
            } else {
                69
            }
        }
        204 | 205 | 207 | 292 | 293 | 295 => 0,
        206 | 294 => 1,
        // `:61598-61607` — genRand arm.
        208..=211 | 296..=299 => {
            if gen_rng.next_max(2) != 0 {
                125
            } else {
                155
            }
        }
        212..=215 | 300..=303 => 1,
        216 | 304 => 0,
        217 | 305 => 37,
        218 | 306 => 155,
        219 | 307 => 17,
        220 | 308 => 37,
        221 | 309 => 155,
        222 | 310 => 37,
        223 | 311 => 0,
        312 | 313 | 315 => -1,
        _ => 0,
    }
}

/// The wall dust-type ladder, if-chain half (`WorldGen.cs:61656-
/// 62137`): the sequential overrides AFTER the switch. Each `if` runs
/// in C# order; the five genRand arms (186, 182, 93, 144, 27) roll only
/// when their wall matches. The 153-166 band's inner switch is a
/// no-op for the walls it does not list (the C# leaves `num` alone).
fn wall_dust_type_overrides(wall: u16, mut num: i32, gen_rng: &mut UnifiedRandom) -> i32 {
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
        // `:61750` — genRand arm.
        num = gen_rng.next_range(68, 71);
    }
    if wall == 319 {
        num = 171;
    }
    if matches!(wall, 321 | 322) {
        num = 308;
    }
    if wall == 318 {
        num = 226;
    }
    if matches!(wall, 142 | 143) {
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
        // `:61854` — genRand arm.
        num = if gen_rng.next_max(2) != 0 { 23 } else { 6 };
    }
    if (153..=166).contains(&wall) {
        // `:61858-61888` — the inner switch (no default: unlisted walls
        // keep `num`).
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
    if (88..=93).contains(&wall) {
        // `:61910-61917` — the 93 arm rolls genRand.
        num = 86 + wall as i32 - 88;
        if wall == 93 {
            num = gen_rng.next_range(88, 94);
        }
    }
    if wall == 33 {
        num = 14;
    }
    if wall == 41 {
        num = 77;
    }
    if matches!(wall, 42 | 43) {
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
    if matches!(wall, 63 | 65 | 66 | 68) {
        num = 3;
    }
    if matches!(wall, 64 | 67) {
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
        // `:62084` — genRand arm.
        num = if gen_rng.next_max(2) != 0 { 118 } else { 10 };
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
        // `:62108` — genRand arm.
        num = if gen_rng.next_max(2) != 0 { 1 } else { 7 };
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

/// `WorldGen.KillWall_MakeWallDust(i, j, tileCache)`
/// (`WorldGen.cs:61345-62158`): the wall-break dust. The type ladder
/// (switch + overrides above) runs FIRST on `gen`, then the four-way
/// spawn tail on `rng`. `disco` is the `(Main.DiscoR, DiscoG, DiscoB)`
/// snapshot. Returns the spawned slot (the C# returns void; the port
/// returns the last spawn's index, or the sentinel when nothing
/// spawns) so witnesses can pin the record.
pub fn kill_wall_make_wall_dust(
    i: i32,
    j: i32,
    wall: u16,
    disco: (u8, u8, u8),
    rng: &mut UnifiedRandom,
    gen_rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) -> usize {
    let num = wall_dust_type_switch(wall, gen_rng);
    let num = wall_dust_type_overrides(wall, num, gen_rng);
    let pos = Vec2::new((i * 16) as f32, (j * 16) as f32);
    // `:62138-62141` — wall 348: the plain spawn, velocity ×0.4.
    if wall == 348 {
        let idx = dust.new_dust(
            rng, ctx, world, pos, 16, 16, num, 0.0, 0.0, 0, Color::default(), 1.0,
        );
        dust.slot_mut(idx).velocity *= 0.4;
        return idx;
    }
    // `:62142-62146` — wall 44/346: the disco puff.
    if matches!(wall, 44 | 346) {
        let idx = dust.new_dust(
            rng,
            ctx,
            world,
            pos,
            16,
            16,
            66,
            0.0,
            0.0,
            100,
            Color::new(disco.0, disco.1, disco.2, 255),
            0.75,
        );
        dust.slot_mut(idx).no_gravity = true;
        return idx;
    }
    // `:62147-62157` — the range-gated generic arm: walls 116-135
    // (inclusive, three bands) spawn nothing.
    if !(133..=135).contains(&wall)
        && !(116..=125).contains(&wall)
        && !(126..=132).contains(&wall)
    {
        if wall == 76 {
            // `:62151` — the blue-tinted 0.75.
            return dust.new_dust(
                rng,
                ctx,
                world,
                pos,
                16,
                16,
                num,
                0.0,
                0.0,
                75,
                Color::new(0, 80, 255, 100),
                0.75,
            );
        }
        if num >= 0 {
            // `:62155` — the plain spawn.
            return dust.new_dust(
                rng, ctx, world, pos, 16, 16, num, 0.0, 0.0, 0, Color::default(), 1.0,
            );
        }
    }
    crate::particles::DUST_SENTINEL
}

#[cfg(test)]
mod tests {
    use super::*;

    fn world() -> World {
        World::new("test".into(), 200, 200)
    }

    fn ctx() -> SpawnCtx {
        SpawnCtx {
            screen_position: Vec2::ZERO,
            screen_size: Vec2::new(3200.0, 3200.0),
            max_dust_to_draw: 6000,
            ..SpawnCtx::default()
        }
    }

    fn streams_agree(a: &mut UnifiedRandom, b: &mut UnifiedRandom) {
        for _ in 0..16 {
            assert_eq!(a.next(), b.next());
        }
    }

    /// W4-g14 — site 62155: a plain wall (4, stone) resolves num 7 via
    /// the override chain and spawns one type-7 dust; the ladder drew
    /// NOTHING on gen (no genRand arm matches wall 4). RE-DERIVED: the C#
    /// call is `Dust.NewDust(new Vector2(i*16, j*16), 16, 16, num)`
    /// (`WorldGen.cs:62155`), so the spawn is the 16x16 box with its own
    /// jitter — mirrored against a `new_dust` of the same shape, not pinned
    /// to the raw cell origin.
    #[test]
    fn w4_g14_wall_plain_spawn_and_no_gen_draws() {
        let w = world();
        let c = ctx();
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(3);
        let mut gen_rng = UnifiedRandom::new(5);
        let mut gtail = UnifiedRandom::new(5);
        let idx = kill_wall_make_wall_dust(
            4, 5, 4, (0, 0, 0), &mut rng, &mut gen_rng, &mut pool, &c, &w,
        );
        assert_ne!(idx, crate::particles::DUST_SENTINEL);
        assert_eq!(pool.slot(idx).dust_type, 7);
        let mut mpool = DustPool::new();
        let mut m = UnifiedRandom::new(3);
        let midx = mpool.new_dust(
            &mut m, &c, &w, Vec2::new(64.0, 80.0), 16, 16, 7,
            0.0, 0.0, 0, Color::default(), 1.0,
        );
        assert_eq!(pool.slot(idx).position, mpool.slot(midx).position);
        assert_eq!(pool.slot(idx).velocity, mpool.slot(midx).velocity);
        assert_eq!(pool.slot(idx).frame_x, mpool.slot(midx).frame_x);
        streams_agree(&mut gen_rng, &mut gtail); // the ladder drew nothing
    }

    /// W4-g14 — site 62140: wall 348 spawns num 57 with velocity ×0.4;
    /// the switch's 348 arm is draw-free.
    #[test]
    fn w4_g14_wall_348_damps_velocity() {
        let w = world();
        let c = ctx();
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(9);
        let mut gen_rng = UnifiedRandom::new(11);
        let mut mpool = DustPool::new();
        let mut m = UnifiedRandom::new(9);
        let idx = kill_wall_make_wall_dust(
            2, 2, 348, (0, 0, 0), &mut rng, &mut gen_rng, &mut pool, &c, &w,
        );
        let midx = mpool.new_dust(
            &mut m, &c, &w, Vec2::new(32.0, 32.0), 16, 16, 57,
            0.0, 0.0, 0, Color::default(), 1.0,
        );
        mpool.slot_mut(midx).velocity *= 0.4;
        assert_eq!(pool.slot(idx).dust_type, 57);
        assert_eq!(pool.slot(idx).velocity, mpool.slot(midx).velocity);
        streams_agree(&mut rng, &mut m);
    }

    /// W4-g14 — site 62144: wall 44 spawns the type-66 disco puff —
    /// alpha 100, scale 0.75, noGravity, the caller's disco bytes.
    #[test]
    fn w4_g14_wall_44_disco_puff() {
        let w = world();
        let c = ctx();
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(13);
        let mut gen_rng = UnifiedRandom::new(17);
        let idx = kill_wall_make_wall_dust(
            3, 3, 44, (200, 100, 50), &mut rng, &mut gen_rng, &mut pool, &c, &w,
        );
        let d = pool.slot(idx);
        assert_eq!(d.dust_type, 66);
        assert_eq!(d.alpha, 100);
        assert_eq!(d.color, Color::new(200, 100, 50, 255));
        assert!(d.no_gravity);
        // scale = (1 + jitter*0.01) * 0.75 — the 0.75 argument applied.
        assert!(d.scale > 0.5 && d.scale < 1.0);
    }

    /// W4-g14 — site 62151: wall 76 resolves num 4 and takes the
    /// blue-tinted arm (alpha 75, Color(0,80,255,100), scale 0.75).
    #[test]
    fn w4_g14_wall_76_blue_tint() {
        let w = world();
        let c = ctx();
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(19);
        let mut gen_rng = UnifiedRandom::new(23);
        let idx = kill_wall_make_wall_dust(
            1, 1, 76, (0, 0, 0), &mut rng, &mut gen_rng, &mut pool, &c, &w,
        );
        let d = pool.slot(idx);
        assert_eq!(d.dust_type, 4);
        assert_eq!(d.alpha, 75);
        assert_eq!(d.color, Color::new(0, 80, 255, 100));
    }

    /// W4-g14 — the gemsparkle band 116-135 spawns NOTHING (the
    /// range gate), but the ladder's genRand arms still roll when they
    /// match: wall 144 is OUTSIDE the band and rolls Next(2).
    #[test]
    fn w4_g14_wall_band_silence_and_144_roll() {
        let w = world();
        let c = ctx();
        for wall in [116u16, 120, 126, 132, 133, 135] {
            let mut pool = DustPool::new();
            let mut rng = UnifiedRandom::new(29);
            let mut gen_rng = UnifiedRandom::new(31);
            let idx = kill_wall_make_wall_dust(
                1, 1, wall, (0, 0, 0), &mut rng, &mut gen_rng, &mut pool, &c, &w,
            );
            assert_eq!(idx, crate::particles::DUST_SENTINEL, "wall {wall} silent");
        }
        // Wall 144: the override rolls genRand.Next(2) then spawns.
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(29);
        let mut gen_rng = UnifiedRandom::new(31);
        let mut mgen = UnifiedRandom::new(31);
        let want = if mgen.next_max(2) != 0 { 118 } else { 10 };
        let idx = kill_wall_make_wall_dust(
            1, 1, 144, (0, 0, 0), &mut rng, &mut gen_rng, &mut pool, &c, &w,
        );
        assert_eq!(pool.slot(idx).dust_type, want);
        streams_agree(&mut gen_rng, &mut mgen);
    }

    /// W4-g14 — the genRand switch arms roll in C# order: wall 27
    /// (if-chain) and wall 3 (switch) each draw one Next(2); a wall
    /// that matches neither draws nothing. The type picks mirror the
    /// drawn polarity.
    #[test]
    fn w4_g14_wall_genrand_arms_roll_once() {
        let w = world();
        let c = ctx();
        for (wall, switch_arm) in [(3u16, true), (27u16, false)] {
            let mut pool = DustPool::new();
            let mut rng = UnifiedRandom::new(37);
            let mut gen_rng = UnifiedRandom::new(41);
            let mut mgen = UnifiedRandom::new(41);
            let roll = mgen.next_max(2);
            let idx = kill_wall_make_wall_dust(
                1, 1, wall, (0, 0, 0), &mut rng, &mut gen_rng, &mut pool, &c, &w,
            );
            let want = if switch_arm {
                if roll != 0 { 1 } else { 14 }
            } else if roll != 0 {
                1
            } else {
                7
            };
            assert_eq!(pool.slot(idx).dust_type, want, "wall {wall}");
            streams_agree(&mut gen_rng, &mut mgen);
        }
    }

    /// W4-g14 — the negative-num walls spawn nothing through the
    /// generic arm (num >= 0 gate): wall 148 resolves -1.
    #[test]
    fn w4_g14_wall_negative_num_silent() {
        let w = world();
        let c = ctx();
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(43);
        let mut gen_rng = UnifiedRandom::new(47);
        let idx = kill_wall_make_wall_dust(
            1, 1, 148, (0, 0, 0), &mut rng, &mut gen_rng, &mut pool, &c, &w,
        );
        assert_eq!(idx, crate::particles::DUST_SENTINEL);
    }
}
