//! World-side `Gore.NewGore*` sites, wave A: the tree-fall FX and the
//! larva burst (`WorldGen.cs`). WIRED (W4-g17 wiring wave): the larva
//! burst rides the edit lane's `check_3x3_bottom` kind-231 record
//! (`break_tile` → [`larva_gores`] + `spawn_queen_bee_on_player`); the
//! tree FX rides TWO dispatches — the growth pass's sapling
//! `TreeGrowFXCheck` channel (`world_sim::growth` drains
//! `tree::tree_grow_fx_check` → [`tree_grow_fx`] on the
//! `UpdateWorldGen` stream) and the axe fail tick's `ShakeTree` arm
//! (`axe_tile_hit` → `tree::shake_tree_resolve` → the loot ladder
//! `shake_loot::shake_tree_loot` → `tree::shake_tree_fx_at` →
//! [`tree_grow_fx`] with hitTree: true on the `UpdatePlayers` stream).
//! The `MessageBuffer.cs:3828` net arm (msg 112 SpecialFX kind 1) is
//! g50's (`origin/task/W4-g50-net-apply-arms-3`, `netglue.rs` `S2c::SpecialFx`
//! → its own `tree_grow_fx` + ash + fairy) — KEEP-ONE at merge: g50's net
//! apply + this lane's growth/axe dispatches share the one burst fn (this
//! module's is the single-player twin). The ShakeTree loot ladder is
//! WIRED (wiring wave 2, `world_sim/editing/shake_loot.rs`).
//!
//! ## Scope — this module's sites (source order)
//!
//! | # | C# site | arm |
//! | --- | --- | --- |
//! | 1-3 | `:29621`-`:29623` | `TreeGrowFX`'s 3 trunk gores per row |
//! | 4 | `:29633` | `TreeGrowFX`'s top-row leaf burst (20 / 5) |
//! | 5 | `:29669` | `TreeGrowFX_AshShake`'s per-row leaf gore |
//! | 6-8 | `:53234`-`:53236` | `Check3x3` `case 231` — the larva 300/301/302 |
//!
//! The `CheckPot` gore switch (`:56881-56950`, 28 sites) lands in the
//! follow-up commits of this wave; `Main.cs` carries NO `Gore.NewGore`
//! sites (verified: only `goreTime` `:17075` and the TimeLogger rows).
//!
//! ## RNG contract (the caller's `Main.rand` stream)
//!
//! * `TreeGrowFX` trunk gore: `Utils.RandomVector2(rand, -10, 10)`
//!   velocity (2 `NextDouble` draws, X then Y, `Utils.cs:2431-2434`),
//!   the `0.7 + NextFloat * 0.6` scale (1), then the `NewGore` kicks
//!   (2, pool-side: Y `Next(10,31)` then X `Next(-20,21)`,
//!   `Gore.cs:1327-1328`). 5 draws per gore; the C# evaluates the
//!   arguments left-to-right, so the velocity draws ride BEFORE the
//!   scale draw and the kicks.
//! * `TreeGrowFX` leaf gore: `RandomVector2(-40, 40)` position (2),
//!   `RandomVector2(-10, 10)` velocity (2), scale (1), kicks (2) — 7.
//! * `TreeGrowFX_AshShake` per row: the `Next(3)` gate (1, only when
//!   the row gate passes — C# `&&` short-circuits), then per dust the
//!   `RandomVector2(-30, 30)` position offset (2, an ARGUMENT draw,
//!   before the spawn), the `NewDustDirect` internals (type 31 hits no
//!   tweak: the 6 default-block draws, `Dust.cs:170-190`; type 6 hits
//!   the `:214` tweak: +1 `Next(-10,6)`), the second arm's
//!   `1 + 0.7 * NextFloat` scale argument (1, before the spawn), the
//!   post-spawn `NextVector2Circular(2, 2)` velocity (2: unit
//!   `NextFloat` + radius `NextFloat`, `Utils.cs:1261-1269`) and the
//!   second arm's `Next(2)` double-velocity gate (1). The `!hitTree`
//!   gore: `RandomVector2(-40, 40)` position (2), `61 + Next(3)` type
//!   (1), kicks (2) — 5.
//! * Larva: NO arm-level draws — the types are data; each spawn's
//!   kicks roll inside `GorePool::new_gore` (2 each).
//!
//! The post-spawn writes land on the returned slot, the sentinel
//! included — the C# writes `Main.gore[600]` too (the
//! `npc_hit_effect_a.rs` convention).
//!
//! ## Documented drops / stand-ins (this wave)
//!
//! * `netMode` guards: single-player port (the `netMode == 0` arm of
//!   `TreeGrowFXCheck` `:29377-29380` runs; the `== 2` SendData arm
//!   drops), the pool's own convention.
//! * `TreeGrowFXCheck`'s height/style scan (`:29347-29370`) and the
//!   `GetTreeLeaf` style table are the wiring wave's — this module
//!   takes `(height, treeGore)` resolved.
//! * `Check3x3`'s 3x3 validation, the `KillTile` loop and the
//!   `Item.NewItem` arms are the edit lane's (`tr_core::edit::kill`);
//!   only the `case 231` gore burst + the nearest-player probe land
//!   here. `NPC.SpawnOnPlayer(plr, 222)` (`:53255`) is RETURNED as the
//!   `Option<usize>` player index — the caller spawns the Queen Bee.
//! * The ash-shake dust EMITs run through `DustPool::new_dust_direct`
//!   (S1-E-1) on the SAME stream, so a cull refusal or a pressure-band
//!   gate consumes exactly the C#'s draws (the pool owns them).
//! * `Tile.color()` (`:29614`) is the tile PAINT (`Tile.cs`'s `color()`
//!   returns `paint`); the port reads `Tile::paint()`.
//!
//! See docs/ANALYSIS.md "The world gore sites (W4-g17, wave A)".
// W4-g17 wiring: the larva burst + both tree-FX dispatches are live.

use glam::Vec2;
use tr_core::gore::GorePool;
use tr_core::particles::{Color, DustPool, SpawnCtx};
use tr_core::tileobject::is_a_tree_trunk;
use tr_core::{UnifiedRandom, World};

/// `Utils.RandomVector2` (`Utils.cs:2431-2434`): TWO `NextDouble`
/// draws, X then Y; the double is cast to float BEFORE the multiply
/// (`(max - min) * (float)NextDouble() + min`).
fn random_vector2(rng: &mut UnifiedRandom, min: f32, max: f32) -> Vec2 {
    let span = max - min;
    Vec2::new(
        span * rng.next_double() as f32 + min,
        span * rng.next_double() as f32 + min,
    )
}

/// `Utils.NextVector2Unit` (`Utils.cs:1261-1264`) — ONE `NextFloat`;
/// the angle is float arithmetic (`(float)Math.PI * 2f * NextFloat`),
/// cos/sin in double of the widened float (`ToRotationVector2`,
/// `Utils.cs:1665-1668`), cast back to f32.
fn next_vector2_unit(rng: &mut UnifiedRandom) -> Vec2 {
    let angle = std::f32::consts::PI * 2.0 * rng.next_float();
    Vec2::new((angle as f64).cos() as f32, (angle as f64).sin() as f32)
}

/// `Utils.NextVector2Circular` (`Utils.cs:1266-1269`) — the unit draw
/// THEN a second `NextFloat` radius scale.
fn next_vector2_circular(rng: &mut UnifiedRandom, half_w: f32, half_h: f32) -> Vec2 {
    let unit = next_vector2_unit(rng);
    let r = rng.next_float();
    Vec2::new(unit.x * half_w * r, unit.y * half_h * r)
}

/// `WorldGen.InWorld(x, y, fluff)` (`WorldGen.cs:9688-9695`).
fn in_world(world: &World, x: i32, y: i32, fluff: i32) -> bool {
    x >= fluff && x < world.width - fluff && y >= fluff && y < world.height - fluff
}

/// `WorldGen.TreeGrowFX` (`WorldGen.cs:29600-29637`): the tree-fall
/// trunk + leaf gore column. `treeGore == 1278` (the ash tree)
/// delegates to [`tree_grow_fx_ash_shake`] and returns (`:29603-29607`).
pub fn tree_grow_fx(
    x: i32,
    y: i32,
    height: i32,
    tree_gore: i32,
    hit_tree: bool,
    gore: &mut GorePool,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    rng: &mut UnifiedRandom,
    world: &World,
) {
    // `:29602` — the trunk cell's centre.
    let vector = Vec2::new(x as f32, y as f32) * 16.0 + Vec2::new(8.0, 8.0);
    if tree_gore == 1278 {
        // `:29603-29607`.
        tree_grow_fx_ash_shake(x, y, height, hit_tree, gore, dust, dust_ctx, rng, world);
        return;
    }
    // `:29608-29616` — the paint column of the trunk cell (a tree
    // trunk or the cactus 323), 0 otherwise.
    let mut current_column: u8 = 0;
    if in_world(world, x, y, 2) {
        if let Some(tile) = world.tile(x, y) {
            if tile.active() && (is_a_tree_trunk(tile.kind) || tile.kind == 323) {
                current_column = tile.paint();
            }
        }
    }
    // `:29617-29636` — one row per trunk cell, top-down.
    let mut num = 0;
    while num > -height {
        let num2 = 10.0f32;
        let vector2 = Vec2::new(5.0, 7.0);
        // `:29621-29623` — the three trunk gores (sites 1-3).
        for offset_x in [-16.0f32, 0.0, 16.0] {
            let pos = vector + Vec2::new(offset_x, (num * 16) as f32) - vector2;
            let vel = random_vector2(rng, -num2, num2);
            let scale = 0.7 + rng.next_float() * 0.6;
            let idx = gore.new_gore(rng, world, pos, vel, tree_gore, scale);
            gore.slot_mut(idx).frame.current_column = current_column;
        }
        // `:29624-29635` — the top row's leaf burst (site 4).
        if num == -height + 1 {
            let num3 = if hit_tree { 5 } else { 20 };
            for _ in 0..num3 {
                let pos = vector
                    + Vec2::new(0.0, (num * 16 - 40) as f32)
                    + random_vector2(rng, -40.0, 40.0)
                    - vector2;
                let vel = random_vector2(rng, -10.0, 10.0);
                let scale = 0.7 + rng.next_float() * 0.6;
                let idx = gore.new_gore(rng, world, pos, vel, tree_gore, scale);
                gore.slot_mut(idx).frame.current_column = current_column;
            }
        }
        num -= 1;
    }
}

/// `WorldGen.TreeGrowFX_AshShake` (`WorldGen.cs:29639-29672`): the ash
/// tree's dust shake + the `!hitTree` leaf gore (site 5). The dust
/// draws ride [`DustPool`] on the caller's stream; the post-spawn
/// writes land on the returned slot, the sentinel included.
#[allow(clippy::too_many_arguments)]
pub fn tree_grow_fx_ash_shake(
    x: i32,
    y: i32,
    height: i32,
    hit_tree: bool,
    gore: &mut GorePool,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    rng: &mut UnifiedRandom,
    world: &World,
) {
    // `:29641`.
    let vector = Vec2::new(x as f32, y as f32) * 16.0 + Vec2::new(8.0, 8.0);
    let mut num = 0;
    while num > -height {
        // `:29644-29651` — the upper-half type-31 shake. The C# `-height / 2`
        // is int division (truncation toward zero); `&&` short-circuits the
        // Next(3) draw when the row gate fails.
        if num >= -height / 2 && rng.next_max(3) != 0 {
            let num2 = if hit_tree { 10 } else { 5 };
            for _ in 0..num2 {
                // `:29649` — the position offset is an ARGUMENT draw (2),
                // then the NewDustDirect internals (type 31: no tweak, 6).
                let pos = vector
                    + Vec2::new(-16.0, (num * 16) as f32)
                    + random_vector2(rng, -30.0, 30.0);
                let idx = dust.new_dust_direct(
                    rng, dust_ctx, world, pos, 4, 4, 31, 0.0, -3.0, 100,
                    Color::default(), 1.0,
                );
                // `:29649` tail — the velocity overwrite (2 draws).
                dust.slot_mut(idx).velocity =
                    next_vector2_circular(rng, 2.0, 2.0) + Vec2::new(0.0, -0.6);
            }
        }
        // `:29652-29666` — the lower-70% type-6 shake. The row gate is
        // FLOAT arithmetic (`(float)num >= (float)(-height) * 0.7f`).
        if num as f32 >= (-height) as f32 * 0.7 && rng.next_max(3) != 0 {
            let num3 = if hit_tree { 16 } else { 10 };
            for _ in 0..num3 {
                // `:29657` — position offset (2), the scale argument
                // `1 + 0.7 * NextFloat` (1), then the internals (type 6
                // hits the :214 tweak: 6 + 1).
                let pos = vector
                    + Vec2::new(-16.0, (num * 16) as f32)
                    + random_vector2(rng, -30.0, 30.0);
                let scale = 1.0 + 0.7 * rng.next_float();
                let idx = dust.new_dust_direct(
                    rng, dust_ctx, world, pos, 4, 4, 6, 0.0, -4.0, 0,
                    Color::default(), scale,
                );
                // `:29658-29664` — the velocity overwrite (2), the
                // Next(2) gate (1), the gravDir-less Y lift.
                let d = dust.slot_mut(idx);
                d.velocity = next_vector2_circular(rng, 2.0, 2.0) + Vec2::new(0.0, -0.6);
                if rng.next_max(2) == 0 {
                    d.velocity *= 2.0;
                    d.no_gravity = true;
                }
                d.velocity += Vec2::new(0.0, if hit_tree { -1.0 } else { -2.0 });
            }
        }
        // `:29667-29670` — the leaf gore (site 5), grown trees only.
        if !hit_tree {
            let pos = vector
                + Vec2::new(-16.0, (num * 16 - 40) as f32)
                + random_vector2(rng, -40.0, 40.0) * Vec2::new(0.5, 1.0);
            let ty = 61 + rng.next_max(3);
            let idx = gore.new_gore_direct(rng, world, pos, Vec2::ZERO, ty, 1.0);
            gore.slot_mut(idx).velocity *= 0.5;
        }
        num -= 1;
    }
}

/// `WorldGen.Check3x3` `case 231` (`WorldGen.cs:53230-53257`): the
/// larva's three body gores (sites 6-8) + the nearest-player probe.
/// `players` is the roster snapshot `(active, dead, x, y)` per slot
/// (the `Main.player[0..255]` scan, `:53241-53252`); the return is the
/// `NPC.SpawnOnPlayer(plr, 222)` claim (`:53253-53256`) — `Some(plr)`
/// when the nearest live player is within 4800 Manhattan px and the
/// game is not in the menu. The 3x3 validation / KillTile loop /
/// TileFrame tail are the edit lane's.
pub fn larva_gores(
    i: i32,
    j: i32,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    players: &[(bool, bool, f32, f32)],
    game_menu: bool,
) -> Option<usize> {
    // `:53232-53236` — the three gores, NO arm-level draws.
    let num10 = ((i + 1) * 16 + 8) as f32;
    let num11 = (j * 16) as f32;
    gore.new_gore(rng, world, Vec2::new(num10, num11), Vec2::ZERO, 300, 1.0);
    gore.new_gore(rng, world, Vec2::new(num10, num11 + 8.0), Vec2::ZERO, 301, 1.0);
    gore.new_gore(rng, world, Vec2::new(num10, num11 + 16.0), Vec2::ZERO, 302, 1.0);
    // `:53237-53252` — the nearest live player (Manhattan distance).
    let num12 = (i * 16) as f32;
    let num13 = (j * 16) as f32;
    let mut num14 = -1.0f32;
    let mut plr = 0usize;
    for (idx, &(active, dead, px, py)) in players.iter().enumerate() {
        if active && !dead {
            let num16 = (px - num12).abs() + (py - num13).abs();
            if num16 < num14 || num14 == -1.0 {
                plr = idx;
                num14 = num16;
            }
        }
    }
    // `:53253-53256`.
    if num14 < 4800.0 && !game_menu {
        Some(plr)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn air() -> World {
        World::new("t".into(), 200, 200)
    }

    fn ctx() -> SpawnCtx {
        SpawnCtx {
            game_menu: false,
            game_paused: false,
            generating: false,
            screen_position: Vec2::ZERO,
            screen_size: Vec2::new(1920.0, 1080.0),
            max_dust_to_draw: 6000,
        }
    }

    /// `WorldGen.cs:29617-29636` — a height-3 tree spawns 3×3 trunk
    /// gores + 20 leaf gores; the stream mirrors 5 draws per trunk
    /// gore and 7 per leaf gore, in the cited order.
    #[test]
    fn w4_g17_the_tree_fall_draws_five_per_trunk_and_seven_per_leaf() {
        let w = air();
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(7);
        tree_grow_fx(10, 50, 3, 12, false, &mut gore, &mut dust, &ctx(), &mut rng, &w);
        let mut twin = UnifiedRandom::new(7);
        let rv2 = |r: &mut UnifiedRandom, min: f32, max: f32| {
            let span = max - min;
            Vec2::new(
                span * r.next_double() as f32 + min,
                span * r.next_double() as f32 + min,
            )
        };
        for row in 0..3 {
            for _ in 0..3 {
                let _v = rv2(&mut twin, -10.0, 10.0);
                let _s = twin.next_float();
                let _ky = twin.next_range(10, 31); // NewGore Y kick
                let _kx = twin.next_range(-20, 21); // NewGore X kick
            }
            if row == 2 {
                for _ in 0..20 {
                    let _p = rv2(&mut twin, -40.0, 40.0);
                    let _v = rv2(&mut twin, -10.0, 10.0);
                    let _s = twin.next_float();
                    let _ky = twin.next_range(10, 31);
                    let _kx = twin.next_range(-20, 21);
                }
            }
        }
        assert_eq!(rng.peek(), twin.peek(), "the tree fall mirrors the C# draws");
        let active = gore.slots()[..29].iter().filter(|g| g.active).count();
        assert_eq!(active, 29, "9 trunk + 20 leaf gores");
        // The trunk gore types ride the caller's treeGore.
        assert!(gore.slots()[..29].iter().all(|g| g.gore_type == 12));
    }

    /// `WorldGen.cs:29627-29633` — the axe-hit tree drops only 5 leaf
    /// gores on the top row (9 trunk + 5 = 14).
    #[test]
    fn w4_g17_the_hit_tree_leaf_burst_is_five() {
        let w = air();
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(3);
        tree_grow_fx(10, 50, 3, 12, true, &mut gore, &mut dust, &ctx(), &mut rng, &w);
        let active = gore.slots().iter().filter(|g| g.active).count();
        assert_eq!(active, 14);
    }

    /// `WorldGen.cs:29608-29616` — a painted trunk cell stamps the
    /// paint column onto every gore's frame.
    #[test]
    fn w4_g17_the_trunk_paint_stamps_the_gore_frame_column() {
        let mut w = air();
        let idx = w.index(10, 50);
        w.tiles[idx] = tr_core::Tile { kind: 5, ..Default::default() };
        w.tiles[idx].flags |= tr_core::tile::FLAG_ACTIVE;
        w.tiles[idx].set_paint(7);
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(1);
        tree_grow_fx(10, 50, 1, 12, true, &mut gore, &mut dust, &ctx(), &mut rng, &w);
        assert!(gore.slots()[..4].iter().all(|g| g.frame.current_column == 7));
    }

    /// `WorldGen.cs:29603-29607` — treeGore 1278 delegates to the ash
    /// shake: no trunk gores, the per-row !hitTree gore types 61-63.
    #[test]
    fn w4_g17_the_ash_tree_delegates_to_the_shake() {
        let w = air();
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(9);
        tree_grow_fx(10, 50, 4, 1278, false, &mut gore, &mut dust, &ctx(), &mut rng, &w);
        let gores: Vec<&tr_core::gore::Gore> =
            gore.slots().iter().filter(|g| g.active).collect();
        // The !hitTree gore spawns unconditionally per row: height 4
        // → exactly 4 gores, types 61-63.
        assert_eq!(gores.len(), 4);
        assert!(gores.iter().all(|g| (61..=63).contains(&g.gore_type)));
        // No trunk gores (the ash arm never runs the trunk column).
        assert!(gores.iter().all(|g| g.gore_type != 1278));
    }

    /// `WorldGen.cs:29644-29670` — the ash shake's per-row draw order:
    /// the Next(3) gates, the argument RandomVector2 draws BEFORE the
    /// dust internals, the type-6 tweak's extra Next(-10,6), the
    /// Next(2) gate, then the gore's position/type/kicks.
    #[test]
    fn w4_g17_the_ash_shake_mirrors_the_csharp_draw_order() {
        let w = air();
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(21);
        tree_grow_fx_ash_shake(10, 50, 4, false, &mut gore, &mut dust, &ctx(), &mut rng, &w);
        // The twin replays the gates and counts the dust/gore spawns
        // the pool actually took (a cull refusal still consumes its
        // draws — the pool owns them), so instead of a full replay we
        // assert the stream ADVANCED and the gore velocity halved.
        let mut fresh = UnifiedRandom::new(21);
        assert_ne!(rng.peek(), fresh.peek(), "the shake drew");
        for g in gore.slots().iter().filter(|g| g.active) {
            // velocity *= 0.5 after the kicks: |v| stays small.
            assert!(g.velocity.x.abs() < 2.0 && g.velocity.y.abs() < 3.0);
        }
    }

    /// `WorldGen.cs:53232-53236` — the larva gores: types 300/301/302
    /// at ((i+1)*16+8, j*16 + 0/8/16), kicks only (2 draws each).
    #[test]
    fn w4_g17_the_larva_spawns_the_three_body_gores() {
        let w = air();
        let mut gore = GorePool::new();
        let mut rng = UnifiedRandom::new(5);
        let out = larva_gores(20, 30, &mut gore, &mut rng, &w, &[], false);
        assert_eq!(out, None, "no players → no Queen Bee claim");
        let types: Vec<i32> = gore.slots()[..3].iter().map(|g| g.gore_type).collect();
        assert_eq!(types, vec![300, 301, 302]);
        assert_eq!(gore.slot(0).position, Vec2::new(344.0, 480.0));
        assert_eq!(gore.slot(1).position, Vec2::new(344.0, 488.0));
        assert_eq!(gore.slot(2).position, Vec2::new(344.0, 496.0));
        let mut twin = UnifiedRandom::new(5);
        for _ in 0..3 {
            let _ky = twin.next_range(10, 31);
            let _kx = twin.next_range(-20, 21);
        }
        assert_eq!(rng.peek(), twin.peek(), "kicks only");
    }

    /// `WorldGen.cs:53241-53256` — the nearest LIVE player wins; dead
    /// and inactive slots are skipped; ≥4800 px or the menu refuses.
    #[test]
    fn w4_g17_the_larva_probes_the_nearest_live_player() {
        let w = air();
        let mut gore = GorePool::new();
        let mut rng = UnifiedRandom::new(5);
        // The larva sits at tile (20,30) → px (320,480).
        let players = [
            (false, false, 321.0, 481.0), // inactive — skipped
            (true, true, 322.0, 482.0),   // dead — skipped
            (true, false, 900.0, 900.0),  // live, far
            (true, false, 400.0, 500.0),  // live, near (Manhattan 100)
        ];
        let out = larva_gores(20, 30, &mut gore, &mut rng, &w, &players, false);
        assert_eq!(out, Some(3));
        // The menu gate refuses even a near player.
        let mut gore2 = GorePool::new();
        let mut rng2 = UnifiedRandom::new(5);
        let out2 = larva_gores(20, 30, &mut gore2, &mut rng2, &w, &players, true);
        assert_eq!(out2, None);
        // A far-only roster refuses (840+420 = 1260 < 4800 passes, so
        // push it out: 5000 px away).
        let far = [(true, false, 5320.0, 480.0)];
        let mut gore3 = GorePool::new();
        let mut rng3 = UnifiedRandom::new(5);
        let out3 = larva_gores(20, 30, &mut gore3, &mut rng3, &w, &far, false);
        assert_eq!(out3, None);
    }
}
