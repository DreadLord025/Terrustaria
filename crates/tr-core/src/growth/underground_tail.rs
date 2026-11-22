//! The UNDERGROUND tail of `WorldGen.UpdateWorld_GrassGrowth`
//! (`WorldGen.cs:74414-74507`, the second `switch (type)` after the soil
//! scan) plus its `PlaceCatTail` dependency (`:58629-58742`) — W4-g42
//! dispatcher follow-up D4. The port's `grass.rs::update_world_grass_growth`
//! carried only the table + plant roll + soil scan; the tail switch (the
//! jungle case-60 Plantera-bulb/life-fruit/rose arms and the mushroom
//! case-70 cattail/tree arm) never ran underground.
//!
//! `PlaceCatTail` is c33's claimed helper on the worldgen side
//! (`worldgen/ocean.rs::place_cat_tail` over the xorshift `Rng`, fd7fae0e);
//! this runtime copy rides `UnifiedRandom` and is RNG-free at runtime — the
//! only draw (`:58724`, the gen-time slope flatten) is behind
//! `isGeneratingOrLoadingWorld`, which is false here.
//!
//! Deviations (ANALYSIS): the port is single-difficulty normal — the
//! life-fruit odds stay `Next(40)` and the scan radius 60 (the expert
//! `Next(30)`/50 arms fold off); `SquareTileFrame` re-frames and the
//! `netMode == 2` sends are the framer's / MP's job.

use crate::tile::FLAG_ACTIVE;
use crate::UnifiedRandom;
use crate::world::World;

/// The underground tail switch (`WorldGen.cs:74414-74507`). `flag` is the
/// plant-above roll's attempt flag (the C# `flag` at `:74365` — set BEFORE
/// `PlaceTile`, so true when the roll fired and the density gate passed,
/// regardless of whether the placement took).
pub(super) fn underground_grass_tail(
    world: &mut World,
    i: i32,
    j: i32,
    min_j: i32,
    kind: u16,
    flag: bool,
    rng: &mut UnifiedRandom,
) {
    match kind {
        60 => {
            // `:74418`: `flag || Next(25) != 0 || liquid != 0` → break. The
            // Next(25) draws only when the plant roll did NOT fire.
            if flag
                || rng.next_max(25) != 0
                || world.tile(i, min_j).is_some_and(|t| t.liquid != 0)
            {
                return;
            }
            // `:74422-74448`: the Plantera bulb 238 — hardmode + all three
            // mechs down + Next(60), then the ±150 density scan (step 2).
            if world.hard_mode
                && world.downed_mech_boss1
                && world.downed_mech_boss2
                && world.downed_mech_boss3
                && rng.next_max(60) == 0
            {
                let mut flag4 = true;
                let num6 = 150;
                'scan: for m in (i - num6..i + num6).step_by(2) {
                    for n in (j - num6..j + num6).step_by(2) {
                        // The m>1/n>1 guards keep the reads in-world (the
                        // port's index would wrap a negative coordinate).
                        if m > 1
                            && m < world.width as i32 - 2
                            && n > 1
                            && n < world.height as i32 - 2
                            && world
                                .tile(m, n)
                                .is_some_and(|t| t.active() && t.kind == 238)
                        {
                            flag4 = false;
                            break 'scan;
                        }
                    }
                }
                if flag4 {
                    super::jungle::place_jungle_plant(world, i, min_j, 238, 0, 0, true);
                }
            }
            // `:74449-74480`: the life fruit 236 — hardmode + any mech down
            // + Next(40) (normal; expert 30 folds off), the ±60 scan (step
            // 2), the Next(3) style. A taken arm BREAKS — no rose below.
            let any_mech =
                world.downed_mech_boss1 || world.downed_mech_boss2 || world.downed_mech_boss3;
            if world.hard_mode && any_mech && rng.next_max(40) == 0 {
                let mut flag5 = true;
                let num7 = 60;
                'scan: for num8 in (i - num7..i + num7).step_by(2) {
                    for num9 in (j - num7..j + num7).step_by(2) {
                        if num8 > 1
                            && num8 < world.width as i32 - 2
                            && num9 > 1
                            && num9 < world.height as i32 - 2
                            && world
                                .tile(num8, num9)
                                .is_some_and(|t| t.active() && t.kind == 236)
                        {
                            flag5 = false;
                            break 'scan;
                        }
                    }
                }
                if flag5 {
                    super::jungle::place_jungle_plant(
                        world,
                        i,
                        min_j,
                        236,
                        rng.next_max(3),
                        0,
                        true,
                    );
                }
                return;
            }
            // `:74481-74495`: the jungle rose 233 — the 3-wide Next(8)
            // style, the 2-wide Next(12)/styleY-1 fallback.
            super::jungle::place_jungle_plant(world, i, min_j, 233, rng.next_max(8), 0, true);
            if !world.tile(i, min_j).is_some_and(|t| t.kind == 233) {
                super::jungle::place_jungle_plant(world, i, min_j, 233, rng.next_max(12), 1, true);
            }
        }
        70 => {
            // `:74497-74506`: the liquid-above cattail branch, then the
            // Next(250) giant-mushroom-tree roll. The `&& PlayerLOS`
            // evaluates the grow first — the FX gate is unported.
            // `:74498`: the C# reads the RAW `j - 1` (not the clamped
            // minJ) for the liquid gate and the cattail call.
            if world.tile(i, j - 1).is_some_and(|t| t.liquid > 0) {
                let _ = place_cat_tail(world, i, j - 1);
            }
            if rng.next_max(250) == 0 {
                let _ = super::trees::try_growing_tree_by_type(world, 5, i, j, rng);
            }
        }
        _ => {}
    }
}

/// `WorldGen.PlaceCatTail` (`WorldGen.cs:58629-58742`), the runtime copy:
/// the cattail 519 rises from the water column above `(x, j)` — up through
/// the liquid, back down one, the wall/density/depth gates, the ground-type
/// frame row, and the write one above the ground. Returns the placed cell
/// (`(-1, -1)` on refusal). RNG-free at runtime: the only draw (`:58724`)
/// is the gen-time slope flatten behind `isGeneratingOrLoadingWorld`.
pub(super) fn place_cat_tail(world: &mut World, x: i32, j: i32) -> (i32, i32) {
    let mut num = j;
    let result = (-1i32, -1i32);
    // `:58633-58636`: the 50-tile margin.
    if x < 50 || x > world.width as i32 - 50 || num < 50 || num > world.height as i32 - 50 {
        return result;
    }
    // `:58637-58640`: the start cell — free (or a 71 mushroom), water only.
    let start_ok = world.tile(x, num).is_some_and(|t| {
        (!t.active() || t.kind == 71)
            && t.liquid > 0
            && t.liquid_kind() == crate::tile::LiquidKind::Water
    });
    if !start_ok {
        return result;
    }
    // `:58641-58645`: up through the liquid, back down one.
    while num > 50 && world.tile(x, num).is_some_and(|t| t.liquid > 0) {
        num -= 1;
    }
    num += 1;
    // `:58646-58649`: the surface cell and the one above must be free, the
    // surface cell still water.
    if world.tile(x, num).is_some_and(|t| t.active())
        || world.tile(x, num - 1).is_some_and(|t| t.active())
        || !world.tile(x, num).is_some_and(|t| {
            t.liquid > 0 && t.liquid_kind() == crate::tile::LiquidKind::Water
        })
    {
        return result;
    }
    // `:58650-58653`: the wall gate — none, or the mushroom/jungle family.
    let wall = world.tile(x, num).map(|t| t.wall).unwrap_or(0);
    if wall != 0
        && !matches!(wall, 80 | 81 | 69)
        && !(63..=68).contains(&wall)
    {
        return result;
    }
    // `:58654-58670`: the ±7 density scan — at most 3 columns holding a 519.
    let mut num3 = 0;
    for i in (x - 7)..=(x + 7) {
        for k in (num - 7)..=(num + 7) {
            // The 50-margin gate keeps x/num >= 50, but the ±7 box can
            // still read past the world edge on a small world — skip.
            if i < 0 || k < 0 || i >= world.width as i32 || k >= world.height as i32 {
                continue;
            }
            if world.tile(i, k).is_some_and(|t| t.active() && t.kind == 519) {
                num3 += 1;
                break;
            }
        }
    }
    if num3 > 3 {
        return result;
    }
    // `:58671-58678`: down to the ground — a non-71 active cell on the way
    // refuses.
    let mut l = num;
    while l < world.height as i32 - 50
        && !world.tile(x, l).is_some_and(|t| {
            t.active()
                && crate::tiledata::TILE_SOLID.has(t.kind)
                && !crate::tiledata::TILE_SOLID_TOP.has(t.kind)
        })
    {
        if world.tile(x, l).is_some_and(|t| t.active() && t.kind != 71) {
            return result;
        }
        l += 1;
    }
    // `:58679-58686`: the depth window — 2..=catTailDistance-1 (8-1).
    if l - num > super::decorations::CAT_TAIL_DISTANCE - 1 || l - num < 2 {
        return result;
    }
    // `:58687-58691`: the ground must be nactive.
    let Some(ground) = world.tile(x, l) else {
        return result;
    };
    if !(ground.active() && !ground.inactive()) {
        return result;
    }
    // `:58692-58723`: the ground-type frame row.
    let num4: i16 = match ground.kind {
        2 | 477 => 0,
        53 => {
            // `:58700-58703`: sand only inside the beach margins.
            if x < super::BEACH_DISTANCE || x > world.width as i32 - super::BEACH_DISTANCE {
                return result;
            }
            18
        }
        199 | 234 | 662 => 54,
        23 | 112 | 661 => 72,
        70 => 90,
        _ => -1,
    };
    if num4 < 0 {
        return result;
    }
    // `:58724-58731`: the gen-time slope flatten draws Next(3) — at runtime
    // `isGeneratingOrLoadingWorld` is false, so a sloped/half-brick ground
    // simply refuses (no draw).
    if ground.top_slope() || ground.half_brick() {
        return result;
    }
    // `:58732-58741`: the write one above the ground.
    num = l - 1;
    let idx = world.index(x, num);
    world.tiles[idx].flags |= FLAG_ACTIVE;
    world.tiles[idx].kind = 519;
    world.tiles[idx].frame_x = 0;
    world.tiles[idx].frame_y = num4;
    // halfBrick(false) + slope(0) — the slope rides `meta` (tile.rs).
    world.tiles[idx].flags &= !crate::tile::FLAG_HALF_BRICK;
    world.tiles[idx].set_slope(0);
    // `CopyPaintAndCoating(Main.tile[x, num + 1])`.
    let (meta, extras) = {
        let si = world.index(x, num + 1);
        (world.tiles[si].meta, world.extras_at(si))
    };
    let di = world.index(x, num);
    world.tiles[di].meta = meta;
    world.set_extras_at(di, extras);
    if world.frame_important.len() <= 519 {
        world.frame_important.resize(crate::tiledata::TILE_COUNT, false);
    }
    world.frame_important[519] = true;
    (x, num)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tile::Tile;

    fn world() -> World {
        let mut w = World::new("t".into(), 4200, 400);
        w.world_surface = 100.0;
        w.rock_layer = 160.0;
        w
    }

    fn put(w: &mut World, x: i32, y: i32, kind: u16) {
        let i = w.index(x, y);
        w.tiles[i] = Tile { kind, flags: FLAG_ACTIVE, ..Default::default() };
    }

    /// W4-g42 D4: the underground case-60 tail — the `Next(25)` gate draws
    /// only when the plant roll did NOT fire (`flag` false), then the
    /// pre-hardmode walk falls to the jungle rose: `Next(8)` (3-wide),
    /// `Next(12)` (2-wide fallback). Stream-pinned with the rose refused
    /// (no jungle-grass ground under the box → both placements refuse
    /// without drawing).
    #[test]
    fn w4_g42_d4_underground_jungle_tail_draw_order_pre_hardmode() {
        let mut w = world();
        put(&mut w, 100, 200, 60);
        // flag=false, dry cell above, Next(25) must hit 0 for the walk to
        // continue — sweep seeds until it does, then pin the rose draws.
        let mut pinned = false;
        for seed in 1i32..400 {
            let mut rng = UnifiedRandom::new(seed);
            let mut witness = UnifiedRandom::new(seed);
            underground_grass_tail(&mut w, 100, 200, 199, 60, false, &mut rng);
            let gate = witness.next_max(25);
            if gate != 0 {
                assert_eq!(rng, witness, "seed {seed}: the gate miss draws once");
                continue;
            }
            // Pre-hardmode: no Next(60)/Next(40) — straight to the rose.
            let _style3 = witness.next_max(8);
            let _style2 = witness.next_max(12);
            assert_eq!(rng, witness, "seed {seed}: Next(25), Next(8), Next(12)");
            pinned = true;
            break;
        }
        assert!(pinned, "some stream hits the Next(25) gate");
        // The flag=true control: the plant roll fired — NO Next(25) draw.
        let mut rng2 = UnifiedRandom::new(5);
        let witness2 = UnifiedRandom::new(5);
        underground_grass_tail(&mut w, 100, 200, 199, 60, true, &mut rng2);
        assert_eq!(rng2, witness2, "the fired plant roll skips the tail entirely");
    }

    /// W4-g42 D4: the underground case-70 tail — the liquid-above cattail
    /// branch then the `Next(250)` tree roll (`:74497-74506`). With no
    /// liquid above, only the Next(250) draws; the tree grow itself is
    /// gated on the roll (the G5 GrowTree draws only when it fires).
    #[test]
    fn w4_g42_d4_underground_mushroom_tail_next250_only() {
        let mut w = world();
        put(&mut w, 100, 200, 70);
        let mut saw_miss = false;
        for seed in 1i32..100 {
            let mut rng = UnifiedRandom::new(seed);
            let mut witness = UnifiedRandom::new(seed);
            underground_grass_tail(&mut w, 100, 200, 199, 70, false, &mut rng);
            let roll = witness.next_max(250);
            if roll != 0 {
                assert_eq!(rng, witness, "seed {seed}: one Next(250), no tree draws");
                saw_miss = true;
            }
        }
        assert!(saw_miss, "the miss streams pin the single draw");
    }

    /// W4-g42 D4: the runtime `PlaceCatTail` — the water column above the
    /// mushroom grass grows a 519 with the ground's frame row 90, RNG-free
    /// (the gen-time slope-flatten draw never fires at runtime). The depth
    /// window (2..=7) and the ±7 density cap bind.
    #[test]
    fn w4_g42_d4_place_cat_tail_rises_from_the_water_column() {
        let mut w = world();
        // The ground at y=210 (mushroom grass), water y=205..=209 (5 deep).
        put(&mut w, 100, 210, 70);
        for y in 205..=209 {
            let i = w.index(100, y);
            w.tiles[i].liquid = 255;
        }
        let (px, py) = place_cat_tail(&mut w, 100, 209);
        assert_eq!((px, py), (100, 209), "the stalk base sits one above the ground");
        let t = *w.tile(100, 209).unwrap();
        assert_eq!(t.kind, 519);
        assert_eq!((t.frame_x, t.frame_y), (0, 90), "the mushroom ground's frame row");
        // The shallow control: 1-deep water refuses (l - num < 2).
        let mut w2 = world();
        put(&mut w2, 100, 210, 70);
        let i = w2.index(100, 209);
        w2.tiles[i].liquid = 255;
        assert_eq!(place_cat_tail(&mut w2, 100, 209), (-1, -1), "the shallow column refuses");
        // The dry control: no liquid, no cattail.
        let mut w3 = world();
        put(&mut w3, 100, 210, 70);
        assert_eq!(place_cat_tail(&mut w3, 100, 209), (-1, -1), "the dry cell refuses");
    }

    /// W4-c31 DUP-RECONCILE (research/300, the g42 12j phase-9 sanctioned
    /// one-witness port): the liquid veto — a water cell at `min_j` blocks
    /// the whole case-60 tail (`:74418`'s `|| tile[i, minJ].liquid != 0`).
    /// Driven on the tail fn directly (flag=false) so the plant-above roll
    /// cannot place a 61 that would wipe the liquid. The Next(25) gate may
    /// pass, but no 236/238/233 lands and no post-gate draw is spent.
    #[test]
    fn w4_c31_the_liquid_veto_blocks_the_tail() {
        let mut w = world();
        put(&mut w, 100, 200, 60);
        w.hard_mode = true;
        w.downed_mech_boss1 = true;
        w.downed_mech_boss2 = true;
        w.downed_mech_boss3 = true;
        let i = w.index(100, 199);
        w.tiles[i].liquid = 200; // water at the placement row
        let mut rng = UnifiedRandom::new(5);
        for _ in 0..500 {
            underground_grass_tail(&mut w, 100, 200, 199, 60, false, &mut rng);
        }
        assert!(
            !w.tile(100, 199).is_some_and(|t| t.active() && matches!(t.kind, 236 | 238 | 233)),
            "the liquid veto blocks the placement"
        );
    }

    /// W4-g42 D4: the life-fruit arm's hardmode gate — with hardmode + any
    /// mech down the `Next(40)` draws (after the Next(25) gate and the
    /// Next(60) bulb miss when all three mechs are down); a taken arm
    /// BREAKS before the rose. The ±60 scan refuses when a 236 sits inside.
    #[test]
    fn w4_g42_d4_underground_life_fruit_gate_and_break() {
        let mut w = world();
        put(&mut w, 100, 200, 60);
        w.hard_mode = true;
        w.downed_mech_boss1 = true; // any-mech, not all-three
        // A life fruit already inside the ±60 box → the scan refuses the
        // placement, but the Next(40) + Next(3)-less walk still BREAKS (no
        // rose draws).
        put(&mut w, 120, 210, 236);
        let mut pinned = false;
        for seed in 1i32..800 {
            let mut rng = UnifiedRandom::new(seed);
            let mut witness = UnifiedRandom::new(seed);
            underground_grass_tail(&mut w, 100, 200, 199, 60, false, &mut rng);
            let gate = witness.next_max(25);
            if gate != 0 {
                assert_eq!(rng, witness);
                continue;
            }
            let fruit = witness.next_max(40);
            if fruit == 0 {
                // The scan found the 236 → no placement, no Next(3), and
                // the arm breaks — no rose Next(8)/Next(12).
                assert_eq!(rng, witness, "seed {seed}: the taken arm breaks before the rose");
                assert!(!w.tile(100, 199).is_some_and(|t| t.active()));
                pinned = true;
                break;
            }
            // The fruit roll missed → the rose draws follow.
            let _s3 = witness.next_max(8);
            let _s2 = witness.next_max(12);
            assert_eq!(rng, witness, "seed {seed}: the miss falls to the rose");
        }
        assert!(pinned, "some stream hits both gates");
    }
}
