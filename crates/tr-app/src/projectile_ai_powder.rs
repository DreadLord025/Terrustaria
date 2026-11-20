//! W4-d1 — the aiStyle-6 powder / fertilizer cloud (`AI()`'s inline
//! `else if (this.aiStyle == 6)` arm, `Projectile.cs:24100-24228`). The
//! style's five rowed kinds (the census found NO arm and NO rows on main):
//!
//! * **10 PurificationPowder** (`Projectile.cs:897-912`) — the tile scan
//!   converts through `WorldGen.Convert(x, y, 11, 1)` (`:24193`), the
//!   purify ladder (`tr_core::spread::convert_sized`, type 11). The
//!   infectedSeed box growth (`:907-911`) rides the spawn seam, not here.
//! * **11 VilePowder** (`:913-923`) — `Convert(x, y, 1, 1)` (`:24197`),
//!   the corruption ladder.
//! * **463 ViciousPowder** (`:5340-5350`) — `Convert(x, y, 4, 1)`
//!   (`:24201`), the crimson ladder; its dust type remaps 10+463 → 231
//!   (`:24115-24118`).
//! * **1019 Fertilizer** (`:9648-9658`) — NO conversion; the `flag2`
//!   sapling-growth arm (`:24203-24220`) and the slow-velocity kill
//!   (`:24224-24227`).
//! * **1051 SuperFertilizer** (`:9967-9977`) — the fertilizer's `flag`
//!   twin: the 50-dust burst with the per-dust `Next(3)` type pick
//!   (`:24124-24133`) and the +15 tree-height addon (`:24208-24211`).
//!
//! C# sources read WHOLE (the golden rule): the arm `:24100-24228`, the
//! five `SetDefaults` arms above, `WorldGen.Convert` (`WorldGen.cs:55067-
//! 55083` + the type-1/4/11 cells), `AttemptToGrowTreeFromSapling`
//! (`:72810-72925`), `TileID.Sets.CommonSapling` (`TileID.cs:493`).
//!
//! The machine, in the C#'s own order:
//! 1. `flag = type == 1051`, `flag2 = (type == 1019) | flag` (`:24102-3`).
//! 2. `velocity *= 0.95f` (`:24104`), `ai[0]++` (`:24105`), the
//!    `ai[0] == 180f` kill (`:24106-24109`).
//! 3. The one-shot `ai[1] == 0f` dust burst (`:24110-24151`): `ai[1] = 1`,
//!    the dust type / count ladder, the loop. The dust BODIES are SKIPPED
//!    (no dust layer); the SuperFertilizer's per-dust `Next(3)` type pick
//!    (`:24132`) is an EXPLICIT `Main.rand` draw at the call site, so it
//!    STAYS on the stream (the `projectile_ai_misc` convention) — 50 draws
//!    on the first tick for the 1051 alone.
//! 4. `flag3 = Main.myPlayer == owner` (`:24152`); the fertilizer pair
//!    overrides it to `Main.netMode != 1` (`:24153-24156`) — single-player
//!    netMode 0, so true.
//! 5. The owner-gated tile scan (`:24157-24223`): the box's overlapping
//!    cells, the per-type `Convert` (10/11/463), then — for the fertilizer
//!    pair on an ACTIVE CommonSapling cell — the sapling growth. The growth
//!    draws `WorldGen.genRand`, so it DEFERS to the world-sim pass via
//!    `wants_sapling_grow` (the `palm_frame_rolls` precedent).
//! 6. The fertilizer pair's `velocity.Length() < 0.5f` kill (`:24224-7`).
//!
//! Movement: the arm owns its integration — the row's `tileCollide = false`
//! (`UpdatePosition` `:20445`, style 6 is NOT on the skip list `:20435`), so
//! `position += velocity` rides after the AI (the `ai_ice_block` shape).
//!
//! RNG contract (the `UpdateProjectiles` stream, `Main.cs:17775-17777`):
//! - 10/11/463/1019: ZERO `Main.rand` draws in the arm (the dust burst's
//!   non-flag loop rolls nothing; the conversion / growth are rand-free at
//!   this layer — the growth's genRand rides the deferred pass).
//! - 1051: 50 `Next(3)` draws on the first tick (`ai[1] == 0`), one per
//!   dust iteration (`:24132`), in loop order. ZERO on every later tick.
//!
//! CODE-ONLY MODE (W4-d1): the witness tests at the bottom are WRITTEN NOT
//! RUN (no compiler invoked on this branch); every expected value is
//! hand-derived from the cited C# lines.

use tr_core::World;
use tr_core::loot::DropRng;

use crate::projectile::{
    FERTILIZER, Projectile, PURIFICATION_POWDER, SUPER_FERTILIZER,
    VICIOUS_POWDER, VILE_POWDER,
};

/// The port's `Kill()` stand-in for the AI early-kill paths (the slice-C
/// convention — the death hooks run on the pool's `!active` transition).
fn kill(p: &mut Projectile) {
    p.kill_time_left = p.time_left;
    p.active = false;
    p.time_left = 0;
}

/// `AI()`'s aiStyle-6 arm (`Projectile.cs:24100-24228`) for one slot, in the
/// C#'s own block order. `world` is mutably reborrowed at the dispatch call
/// site (the tile-writing carve-out); `rng` is the pass's `UpdateProjectiles`
/// stream; `local_owner` is the pool's `Main.myPlayer`.
pub(crate) fn step(
    p: &mut Projectile,
    world: &mut World,
    rng: &mut dyn DropRng,
    local_owner: u8,
) {
    // `:24102-24103` — the SuperFertilizer flag and the fertilizer pair.
    let flag = p.kind == SUPER_FERTILIZER;
    let flag2 = p.kind == FERTILIZER || flag;
    // `:24104-24109` — the decay, the age counter, the 180-tick kill. The
    // C#'s `Kill()` (`:70644-70648`) sets active false / timeLeft 0 but does
    // NOT return — the burst / scan / slow-kill blocks below still run this
    // tick, so the port keeps going too (the `ai_dungeon_debris` shape).
    p.vel *= 0.95;
    p.ai[0] += 1.0;
    if p.ai[0] == 180.0 {
        kill(p);
    }
    // `:24110-24151` — the one-shot dust burst.
    if p.ai[1] == 0.0 {
        p.ai[1] = 1.0;
        // `:24113-24127` — the dust type / count ladder. The type feeds
        // only the skipped dust; the count drives the loop (and the 1051's
        // per-dust Next(3) draws).
        let mut dust_type: i32 = 10 + p.kind as i32;
        let mut count: i32 = 30;
        if p.kind == VICIOUS_POWDER {
            dust_type = 231; // `:24115-24118`
        }
        if flag2 {
            dust_type = 0; // `:24119-24123`
            count = 40;
        }
        if flag {
            count = 50; // `:24124-24127`
        }
        for _ in 0..count {
            // `:24130-24133` — the SuperFertilizer's per-dust type pick:
            // the EXPLICIT Next(3) stays on the stream (the dust body and
            // its NewDust internals are SKIPPED). The non-flag kinds roll
            // nothing here.
            if flag {
                dust_type = if rng.next_range(0, 3) == 0 { 309 } else { 0 };
            }
            // `:24134` — Dust.NewDust(...) — SKIPPED (no dust layer).
            // `:24135-24149` — the flag2 noGravity / scale / velocity
            // adjustments feed only the skipped dust — SKIPPED.
            let _ = dust_type;
        }
    }
    // `:24152-24156` — the owner gate; the fertilizer pair overrides it to
    // `netMode != 1` (single-player netMode 0 → true).
    let mut flag3 = p.owner == local_owner;
    if flag2 {
        flag3 = true; // `Main.netMode != 1`, the single-player port.
    }
    // `:24157-24223` — the owner-gated tile scan.
    if flag3 && (p.kind == PURIFICATION_POWDER || p.kind == VILE_POWDER
        || p.kind == VICIOUS_POWDER || flag2)
    {
        tile_scan(p, world);
    }
    // `:24224-24227` — the fertilizer pair's slow-velocity kill.
    if flag2 && p.vel.length() < 0.5 {
        kill(p);
    }
    // `UpdatePosition` (`:20445`) — the free move (tileCollide false).
    p.pos += p.vel;
}

/// The tile scan (`Projectile.cs:24159-24222`): the box's overlapping
/// cells, the per-type `WorldGen.Convert`, then the fertilizer pair's
/// sapling growth on active CommonSapling cells.
fn tile_scan(p: &mut Projectile, world: &mut World) {
    // `:24159-24178` — the scan bounds, clamped to the world.
    let mut x0 = (p.pos.x / 16.0) as i32 - 1;
    let mut x1 = ((p.pos.x + p.w) / 16.0) as i32 + 2;
    let mut y0 = (p.pos.y / 16.0) as i32 - 1;
    let mut y1 = ((p.pos.y + p.h) / 16.0) as i32 + 2;
    x0 = x0.max(0);
    x1 = x1.min(world.width);
    y0 = y0.max(0);
    y1 = y1.min(world.height);
    let flag = p.kind == SUPER_FERTILIZER;
    let flag2 = p.kind == FERTILIZER || flag;
    let remix = world.seeds.contains(tr_core::seeds::SeedFlags::REMIX);
    // `:24207-24211` — the SuperFertilizer's +15 tree-height addon.
    let addon = if flag { 15 } else { 0 };
    for tx in x0..x1 {
        for ty in y0..y1 {
            // `:24184-24189` — the cell's 16×16 box must overlap the slot.
            let cx = (tx * 16) as f32;
            let cy = (ty * 16) as f32;
            if !(p.pos.x + p.w > cx && p.pos.x < cx + 16.0
                && p.pos.y + p.h > cy && p.pos.y < cy + 16.0)
            {
                continue;
            }
            // `:24190` — the active flag is captured BEFORE the conversion
            // (the Convert may change the tile, but the growth gate reads
            // the pre-conversion state).
            let flag4 = world.tile(tx, ty).is_some_and(|t| t.active());
            // `:24191-24202` — the per-type conversion (size 1). The
            // fertilizer pair matches none of the three, so it converts
            // nothing — only grows.
            match p.kind {
                PURIFICATION_POWDER => tr_core::spread::convert_sized(world, tx, ty, 11, 1),
                VILE_POWDER => tr_core::spread::convert_sized(world, tx, ty, 1, 1),
                VICIOUS_POWDER => tr_core::spread::convert_sized(world, tx, ty, 4, 1),
                _ => {}
            }
            // `:24203-24206` — only the fertilizer pair on an active cell
            // grows (`!(flag4 & flag2)` continues).
            if !(flag4 && flag2) {
                continue;
            }
            // `:24212-24220` — the CommonSapling growth. The genRand draw
            // defers to the pass (`wants_sapling_grow`); the remix double-
            // call (`:24215-24218`) queues an extra surface growth first.
            let Some(t) = world.tile(tx, ty) else { continue };
            // `TileID.Sets.CommonSapling` (`TileID.cs:493`) — the generated
            // set's `has` carries its own TILE_COUNT bound.
            if tr_core::tiledata::SET_COMMON_SAPLING.has(t.kind) {
                if remix
                    && ty >= world.world_surface as i32 - 1
                    && ty < world.height - 20
                {
                    p.wants_sapling_grow.push((tx, ty, false, addon));
                }
                p.wants_sapling_grow
                    .push((tx, ty, ty > world.world_surface as i32 - 1, addon));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::projectile::proj_data;
    use glam::Vec2;
    use tr_core::tile::{FLAG_ACTIVE, Tile};

    /// A draw-recording `Main.rand` stand-in (the `projectile_ai_misc`
    /// CensusRng shape): `next_range` returns `min`, `next_float` 0.
    struct CensusRng {
        ranges: Vec<(i32, i32)>,
    }
    impl DropRng for CensusRng {
        fn next_range(&mut self, min: i32, max: i32) -> i32 {
            self.ranges.push((min, max));
            min
        }
        fn next_float(&mut self) -> f32 {
            0.0
        }
    }
    fn census() -> CensusRng {
        CensusRng { ranges: Vec::new() }
    }

    fn world() -> World {
        World::new("t".into(), 300, 200)
    }

    fn proj(kind: u16) -> Projectile {
        let d = proj_data(kind).expect("the powder row");
        let mut p = Projectile::default();
        p.active = true;
        p.kind = kind;
        p.owner = 0;
        p.pos = Vec2::new(100.0 * 16.0, 50.0 * 16.0);
        p.vel = Vec2::new(2.0, 0.0);
        p.w = d.w;
        p.h = d.h;
        p.tile_collide = d.tile_collide;
        p.alpha = d.alpha;
        p.time_left = d.time_left;
        p.penetrate = d.penetrate;
        p
    }

    fn put(w: &mut World, x: i32, y: i32, kind: u16) {
        let i = w.index(x, y);
        w.tiles[i] = Tile { kind, ..Default::default() };
        w.tiles[i].flags |= FLAG_ACTIVE;
    }

    #[test]
    fn w4_d1_the_decay_and_age_counter() {
        // `:24104-24105` — velocity *= 0.95, ai[0]++ per tick.
        let mut p = proj(PURIFICATION_POWDER);
        let mut w = world();
        let mut rng = census();
        step(&mut p, &mut w, &mut rng, 0);
        assert_eq!(p.ai[0], 1.0);
        assert!((p.vel.x - 2.0 * 0.95).abs() < 1e-6, "the 0.95 decay");
        assert_eq!(p.ai[1], 1.0, "the one-shot burst latched");
        assert!(rng.ranges.is_empty(), "the 10 rolls nothing");
    }

    #[test]
    fn w4_d1_the_180_tick_kill() {
        // `:24106-24109` — ai[0] == 180 kills.
        let mut p = proj(VILE_POWDER);
        p.ai[0] = 179.0;
        p.ai[1] = 1.0; // past the burst
        let mut w = world();
        let mut rng = census();
        step(&mut p, &mut w, &mut rng, 0);
        assert_eq!(p.ai[0], 180.0);
        assert!(!p.active, "the 180-tick kill");
        assert_eq!(p.time_left, 0);
    }

    #[test]
    fn w4_d1_the_super_fertilizer_burst_draws_50() {
        // `:24124-24133` — the 1051's first tick: count 50, one Next(3)
        // per iteration. Every later tick rolls nothing.
        let mut p = proj(SUPER_FERTILIZER);
        let mut w = world();
        let mut rng = census();
        step(&mut p, &mut w, &mut rng, 0);
        assert_eq!(rng.ranges.len(), 50, "50 Next(3) draws on the burst");
        assert!(rng.ranges.iter().all(|&(a, b)| (a, b) == (0, 3)));
        rng.ranges.clear();
        step(&mut p, &mut w, &mut rng, 0);
        assert!(rng.ranges.is_empty(), "no draws after the one-shot burst");
    }

    #[test]
    fn w4_d1_the_fertilizer_burst_draws_nothing() {
        // `:24119-24123` — the 1019's count is 40 but flag is false, so the
        // loop rolls nothing (the Next(3) is flag-gated).
        let mut p = proj(FERTILIZER);
        let mut w = world();
        let mut rng = census();
        step(&mut p, &mut w, &mut rng, 0);
        assert!(rng.ranges.is_empty(), "the 1019 burst rolls nothing");
        assert_eq!(p.ai[1], 1.0);
    }

    #[test]
    fn w4_d1_the_purify_scan_converts_the_box() {
        // `:24191-24194` — the 10 converts its overlapping cells through
        // type 11. A corrupt grass cell under the cloud purifies to 2.
        let mut p = proj(PURIFICATION_POWDER);
        p.pos = Vec2::new(100.0 * 16.0, 50.0 * 16.0);
        let mut w = world();
        put(&mut w, 101, 51, 23); // corrupt grass inside the 64x64 box
        let mut rng = census();
        step(&mut p, &mut w, &mut rng, 0);
        assert_eq!(w.tile(101, 51).unwrap().kind, 2, "the corrupt grass purifies");
    }

    #[test]
    fn w4_d1_the_vile_scan_corrupts_the_box() {
        // `:24195-24198` — the 11 converts through type 1.
        let mut p = proj(VILE_POWDER);
        p.pos = Vec2::new(100.0 * 16.0, 50.0 * 16.0);
        let mut w = world();
        put(&mut w, 101, 51, 2); // plain grass inside the box
        let mut rng = census();
        step(&mut p, &mut w, &mut rng, 0);
        assert_eq!(w.tile(101, 51).unwrap().kind, 23, "the grass corrupts");
    }

    #[test]
    fn w4_d1_the_fertilizer_queues_sapling_growth() {
        // `:24203-24220` — the 1019 grows active CommonSapling cells; the
        // request defers (x, y, underground, addon 0). A non-sapling cell
        // queues nothing.
        let mut p = proj(FERTILIZER);
        p.pos = Vec2::new(100.0 * 16.0, 50.0 * 16.0);
        let mut w = world();
        put(&mut w, 101, 51, 20); // the ash sapling (CommonSapling)
        put(&mut w, 102, 51, 2); // plain grass — not a sapling
        let mut rng = census();
        step(&mut p, &mut w, &mut rng, 0);
        assert_eq!(p.wants_sapling_grow.len(), 1, "only the sapling queues");
        let (x, y, underground, addon) = p.wants_sapling_grow[0];
        assert_eq!((x, y, addon), (101, 51, 0));
        // ty 51 > world_surface(50) - 1 = 49 → underground true.
        assert!(underground, "below the surface line");
    }

    #[test]
    fn w4_d1_the_super_fertilizer_addon_is_15() {
        // `:24208-24211` — the 1051's addon is 15.
        let mut p = proj(SUPER_FERTILIZER);
        p.pos = Vec2::new(100.0 * 16.0, 50.0 * 16.0);
        let mut w = world();
        put(&mut w, 101, 51, 20);
        let mut rng = census();
        step(&mut p, &mut w, &mut rng, 0);
        assert_eq!(p.wants_sapling_grow.len(), 1);
        assert_eq!(p.wants_sapling_grow[0].3, 15, "the +15 addon");
    }

    #[test]
    fn w4_d1_the_slow_fertilizer_kills() {
        // `:24224-24227` — the fertilizer pair kills below 0.5 speed.
        let mut p = proj(FERTILIZER);
        p.vel = Vec2::new(0.4, 0.0);
        p.ai[1] = 1.0;
        let mut w = world();
        let mut rng = census();
        step(&mut p, &mut w, &mut rng, 0);
        // 0.4 * 0.95 = 0.38 < 0.5 → kill.
        assert!(!p.active, "the slow fertilizer dies");
    }

    #[test]
    fn w4_d1_the_powder_does_not_slow_kill() {
        // The slow-kill is flag2-only (`:24224`); the 10/11/463 keep flying
        // at low speed until the 180-tick / timeLeft death.
        let mut p = proj(PURIFICATION_POWDER);
        p.vel = Vec2::new(0.4, 0.0);
        p.ai[1] = 1.0;
        let mut w = world();
        let mut rng = census();
        step(&mut p, &mut w, &mut rng, 0);
        assert!(p.active, "the powder keeps flying at low speed");
    }

    #[test]
    fn w4_d1_the_non_owner_powder_skips_the_scan() {
        // `:24152-24157` — flag3 = myPlayer == owner; a hostile-owner powder
        // (owner 255) converts nothing. The fertilizer pair overrides flag3
        // to true regardless (netMode != 1).
        let mut p = proj(PURIFICATION_POWDER);
        p.owner = 255;
        p.pos = Vec2::new(100.0 * 16.0, 50.0 * 16.0);
        let mut w = world();
        put(&mut w, 101, 51, 23);
        let mut rng = census();
        step(&mut p, &mut w, &mut rng, 0);
        assert_eq!(w.tile(101, 51).unwrap().kind, 23, "the non-owner scan skips");
    }

    #[test]
    fn w4_d1_the_free_move_integrates() {
        // `UpdatePosition` `:20445` — tileCollide false, position += velocity.
        let mut p = proj(VICIOUS_POWDER);
        p.ai[1] = 1.0;
        let start = p.pos;
        let mut w = world();
        let mut rng = census();
        step(&mut p, &mut w, &mut rng, 0);
        let expected = start + Vec2::new(2.0 * 0.95, 0.0);
        assert!((p.pos - expected).length() < 1e-4, "the free move");
    }
}
