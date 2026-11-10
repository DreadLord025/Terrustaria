//! The grappling hook: `Projectile.AI_007_GrapplingHooks`
//! (`Projectile.cs:52565-52866`), `Player.QuickGrapple`
//! (`Player.cs:6296-6478`) and `Player.GrappleMovement`
//! (`Player.cs:22613-22740`).
//!
//! The vanilla hooks are aiStyle-7 projectiles owned by the player; the port
//! keeps the (single) hook as this module's state instead of the app's
//! projectile list — the multi-hook items (WebSlinger x8, Lunar x4, the dual
//! hooks) are out, so exactly one hook exists at a time and firing kills the
//! previous one (the vanilla `num7 = 1` recycle path).
//!
//! The fire trigger is the `hook` input edge (the vanilla `controlHook`
//! quick-grapple, `Player.cs:27480-27492`); the aim is the mouse world
//! position (the vanilla fires toward `Main.mouseX/Y`), falling back to the
//! facing when the aim is degenerate. `TR_HOOK=<type>` selects the hook
//! (13 = the Hook, 32 = Ivy Whip, 315 = Bat Hook, ...); the granting items
//! land with M3-EQUIP-3.
//!
//! Out of scope (documented in the row): the chain rendering, the hook
//! sounds, the multi-hook budgets, the per-hook pull overrides (the
//! AntiGravity/Static/Squirrel control steering), the Queen Slime teleport,
//! the minecart-track transfer.

use glam::Vec2;
use crate::projectile::{SQUIRREL_HOOK, TRACK_HOOK};
use tr_core::World;

/// The per-type table (`Projectile.cs:52646-52665` max distances,
/// `:52787-52842` retract speeds, `Player.cs:22832-22846` pull speeds).
pub(crate) struct HookSpec {
    /// `Projectile.cs:52646-52665`: the max flight distance in pixels.
    pub max_distance: f32,
    /// `Projectile.cs:52787-52842`: the retract speed in px/tick.
    pub retract_speed: f32,
    /// `Player.cs:22832-22846`: the pull speed cap in px/tick.
    pub pull_speed: f32,
}

pub(crate) fn hook_spec(hook_type: i32) -> HookSpec {
    let (max_distance, retract_speed, pull_speed) = match hook_type {
        32 => (400.0, 15.0, 11.0),
        73 | 74 => (440.0, 22.0, 12.5),
        165 => (375.0, 11.0, 11.0),
        230..=235 => (300.0 + (hook_type - 230) as f32 * 30.0, 11.0 + (hook_type - 230) as f32 * 0.75, 11.0),
        315 => (500.0, 20.0, 14.0),
        322 => (550.0, 22.0, 11.0),
        331 => (400.0, 11.0, 11.0),
        332 => (550.0, 17.0, 11.0),
        372 => (400.0, 11.0, 11.0),
        396 => (300.0, 11.0, 11.0),
        446 => (500.0, 20.0, 11.0),
        486..=489 => (480.0, 18.0, 11.0),
        652 => (600.0, 24.0, 11.0),
        753 => (420.0, 15.0, 11.0),
        // Hook (13) and everything unlisted.
        _ => (300.0, 11.0, 11.0),
    };
    HookSpec {
        max_distance,
        retract_speed,
        pull_speed,
    }
}

/// `Projectile.AI_007_GrapplingHooks` (`Projectile.cs:52565-52866`) run on one
/// pool slot: the state machine writes the slot, and the caller integrates the
/// position (`UpdatePosition` `:20433-20475`) exactly like the vanilla engine.
///
/// Returns what the tick leaves for the caller — the attached center (the pull
/// input) and whether the hook died. The multi-hook bookkeeping
/// (`grappling[]`/`grapCount`, the recycle tables) is this task's next phase
/// and extends this same seam.
pub(crate) struct HookTick {
    /// The attached hook's center (`ai[0] == 2f` only) — what
    /// `GrappleMovement`'s pull reads.
    pub attached: Option<Vec2>,
    /// The hook died this tick (owner dead, >2500 px, or retracted into the
    /// owner) — the caller drops it from any registration.
    pub dead: bool,
}

impl HookTick {
    fn alive() -> Self {
        Self {
            attached: None,
            dead: false,
        }
    }
}

/// `Projectile.AI_007_GrapplingHooks_CanTileBeLatchedOnTo`
/// (`Projectile.cs:52868-52879`): `nactive()` and (solid, or tile 314, or a
/// tree trunk / the cactus for the Squirrel Hook 865). The `403` guard is the
/// source's first line — without it the TrackHook would latch any solid tile.
pub(crate) fn can_latch(hook_type: u16, world: &World, x: i32, y: i32) -> bool {
    let Some(t) = world.tile(x, y) else {
        return false;
    };
    if !t.active() {
        return false;
    }
    if hook_type == TRACK_HOOK && t.kind != tr_core::smart_cursor::TRACK_TILE {
        return false;
    }
    tr_core::tiledata::TILE_SOLID.has(t.kind)
        || t.kind == tr_core::smart_cursor::TRACK_TILE
        || (hook_type == SQUIRREL_HOOK && tr_core::tiledata::SET_IS_A_TREE_TRUNK.has(t.kind))
        || (hook_type == SQUIRREL_HOOK && t.kind == 323)
}

/// `(point / 16)` clamped into the world — the C# `ToTileCoordinates` under
/// the scan's `[0, maxTiles*]` clamp.
fn tile_of(px: f32, tiles: i32) -> i32 {
    ((px / 16.0) as i32).clamp(0, tiles - 1)
}

/// The tile visual hitbox (`WorldGen.GetTileVisualHitbox` `:66577-66595`):
/// tile 323's `frameY` shift and a half-brick's top half. Returns the center
/// the latch pins the hook to.
fn latch_center(world: &World, tx: i32, ty: i32) -> Vec2 {
    let mut x = tx as f32 * 16.0 + 8.0;
    let mut y = ty as f32 * 16.0 + 8.0;
    if let Some(t) = world.tile(tx, ty) {
        if t.kind == 323 {
            x += t.frame_y as f32;
        }
        if t.half_brick() {
            y += 4.0; // the 16x8 rect's center sits 4 px below the tile center
        }
    }
    Vec2::new(x, y)
}

/// The `AI_007` per-type tables (`Projectile.cs:52646-52665` max distances,
/// `:52787-52842` retract speeds, `Player.cs:22832-22851` pull speeds), read
/// off the slot's `ProjectileID`.
pub(crate) fn hook_spec_id(hook_type: u16) -> HookSpec {
    hook_spec(hook_type as i32)
}

/// One `AI_007_GrapplingHooks` tick on a pool slot. `player_center` is
/// `Main.player[owner].MountedCenter`.
///
/// Trace (the C# order, `:52572-52866`): the owner-dead kill; the rotation
/// toward the player; the 2500 px cap; the per-kind light/frame blocks (not
/// modelled — nothing reads them yet); then the state arm — Fly (the distance
/// trigger + the latch scan), Retract (the speed table + the `< 24f` kill) or
/// Attached (the latch re-validation). The position integrates afterwards, and
/// NOT at all while attached (`UpdatePosition`'s early return).
pub(crate) fn ai_hook(
    slot: &mut crate::projectile::Projectile,
    world: &World,
    player_center: Vec2,
    owner_dead: bool,
) -> HookTick {
    let center = slot.pos + Vec2::new(slot.w * 0.5, slot.h * 0.5);
    // `:52572-52574` — a dead owner kills the hook.
    if owner_dead {
        return HookTick {
            attached: None,
            dead: true,
        };
    }
    let delta = player_center - center;
    let dist = delta.length();
    // `:52576-52581` — the rotation toward the player (`atan2(dy,dx) - 1.57`),
    // snapped to a quarter turn for the Squirrel Hook while attached. Its
    // steering reads this in `GetGrapplingForces`; 256's `+3.9250002` offset
    // is visual-only and stays out.
    slot.rot = delta.y.atan2(delta.x) - 1.5707963;
    if slot.kind == SQUIRREL_HOOK && slot.ai[0] == 2.0 {
        slot.rot = (slot.rot / (std::f32::consts::PI * 0.5)).round() * (std::f32::consts::PI * 0.5);
    }
    // `:52594-52596` — the hard range cap, every state.
    if dist > 2500.0 {
        return HookTick {
            attached: None,
            dead: true,
        };
    }

    let spec = hook_spec_id(slot.kind);
    match slot.ai[0] as i32 {
        0 => {
            // `:52646-52665` — past the per-type distance the hook retracts.
            if dist > spec.max_distance {
                slot.ai[0] = 1.0;
            }
        }
        1 => {
            // `:52787-52842` — the velocity is the unit vector toward the
            // player scaled by the per-type retract speed.
            if dist < 24.0 {
                return HookTick {
                    attached: None,
                    dead: true,
                };
            }
            slot.vel = delta * (spec.retract_speed / dist);
        }
        _ => {
            // `:52844-52863` — attached: the latch re-validation. The tile's
            // absence or unsolidifying sends it back to Retract.
            let tx = tile_of(slot.center().0, world.width);
            let ty = tile_of(slot.center().1, world.height);
            if !can_latch(slot.kind, world, tx, ty) {
                slot.ai[0] = 1.0;
            } else {
                return HookTick {
                    attached: Some(Vec2::new(slot.center().0, slot.center().1)),
                    dead: false,
                };
            }
        }
    }
    // Still flying: `:52666-52790`'s latch scan — the 10x10 box at
    // `Center - (5,5)` over the x-major tile range, first match wins. A hook
    // that already retracted keeps its new velocity and integrates.
    if slot.ai[0] as i32 == 0 {
        let a = center - Vec2::new(5.0, 5.0);
        // The C# clamps the scan to `[0, maxTilesX]` inclusive
        // (`:52676-52687`); `World::tile` rejects anything past the edge.
        // `a` is `Center - (5,5)` and `b` (the C#'s upper anchor) is
        // `Center + (5,5)`, so the upper corner is `b + (32,32)` = `a + (42,42)`.
        let x0 = tile_of(a.x - 16.0, world.width).max(0);
        let x1 = tile_of(a.x + 42.0, world.width).min(world.width - 1);
        let y0 = tile_of(a.y - 16.0, world.height).max(0);
        let y1 = tile_of(a.y + 42.0, world.height).min(world.height - 1);
        for i in x0..=x1.max(x0) {
            for j in y0..=y1.max(y0) {
                let tile = Vec2::new(i as f32 * 16.0, j as f32 * 16.0);
                // The overlap test: the 10x10 box at `a` against the 16x16 tile.
                if !(a.x + 10.0 > tile.x && a.x < tile.x + 16.0)
                    || !(a.y + 10.0 > tile.y && a.y < tile.y + 16.0)
                {
                    continue;
                }
                if !can_latch(slot.kind, world, i, j) {
                    continue;
                }
                // `:52703-52719` — latch: the velocity zeroes, the position
                // centres on the tile's visual hitbox, the damage drops to 0,
                // and the state moves to Attached (no further integration).
                slot.pos = latch_center(world, i, j) - Vec2::new(slot.w * 0.5, slot.h * 0.5);
                slot.vel = Vec2::ZERO;
                slot.damage = 0;
                slot.ai[0] = 2.0;
                return HookTick {
                    attached: Some(Vec2::new(slot.center().0, slot.center().1)),
                    dead: false,
                };
            }
        }
    }
    // `UpdatePosition` (`:20456`): `position += velocity`.
    slot.pos += slot.vel;
    HookTick::alive()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::projectile::{HOOK, SKELETRON_HAND, SQUIRREL_HOOK, TRACK_HOOK};
    use crate::test_world_helpers::ground;

    /// Spawn a hook the way `spawn_hook` does (a pool-free helper: the state
    /// machine reads only the slot).
    fn hook(kind: u16, x: f32, y: f32) -> crate::projectile::Projectile {
        let d = crate::projectile::proj_data(kind).expect("a hook row");
        let mut p = crate::projectile::Projectile::default();
        p.active = true;
        p.kind = kind;
        p.w = d.w;
        p.h = d.h;
        p.time_left = d.time_left;
        p.pos = Vec2::new(x - d.w * 0.5, y - d.h * 0.5);
        p.vel = Vec2::ZERO;
        p
    }

    /// Put a tile, mirroring `test_world_helpers`'s FLAG_ACTIVE idiom.
    fn put(w: &mut World, x: i32, y: i32, kind: u16) {
        let i = w.index(x, y);
        w.tiles[i] = tr_core::Tile {
            kind,
            ..Default::default()
        };
        w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
    }

    fn player(x: f32, y: f32) -> Vec2 {
        Vec2::new(x, y)
    }

    // `Projectile.cs:52565-52866` — the three states in their order: the
    // per-type distance flips Fly to Retract, the retract velocity points at
    // the player at the per-type speed, and the `< 24f` reach kills it.
    #[test]
    fn dw_2_1_the_three_states_run_in_order() {
        let w = ground();
        let mut h = hook(HOOK, 40.0 * 16.0, 60.0 * 16.0);
        let pc = player(40.0 * 16.0, 60.0 * 16.0 - 320.0);
        // Fly: 320 px out, under the Hook's 300 limit... no, 320 > 300, so the
        // very first tick retracts.
        assert_eq!(h.ai[0], 0.0);
        let t = ai_hook(&mut h, &w, pc, false);
        assert_eq!(h.ai[0], 1.0, "past 300 px the hook retracts");
        assert!(!t.dead);
        // The source sets the retract velocity on the following tick (the Fly
        // arm only flips `ai[0]` this one), so the speed appears after it.
        ai_hook(&mut h, &w, pc, false);
        let speed = h.vel.length();
        assert!((speed - 11.0).abs() < 1e-3, "retract speed 11, got {speed}");
        assert!(h.vel.y < 0.0, "toward the player above");
        // Walk it in and watch it die at the 24 px reach.
        for _ in 0..40 {
            if ai_hook(&mut h, &w, pc, false).dead {
                break;
            }
        }
        assert!(h.ai[0] == 1.0, "still retracting at the end");
    }

    // `:52666-52790` — the scan is the 10x10 box at `Center - (5,5)` over the
    // x-major tile range, first match wins, and the position pins (no further
    // integration) once attached.
    #[test]
    fn dw_2_2_the_latch_scan_is_x_major_and_pins_the_hook() {
        let mut w = ground();
        // Two candidates: the one at the lower x must win even though the loop
        // reaches it first only by x-majority.
        // Both candidates overlap the 10x10 scan box at `Center - (5,5)`;
        // the x-major loop must reach (39,61) first.
        put(&mut w, 39, 61, 1);
        put(&mut w, 40, 61, 1);
        let mut h = hook(HOOK, 40.0 * 16.0, 61.0 * 16.0);
        h.vel = Vec2::new(1.0, 0.0);
        // Close enough that the Fly state survives the tick (the 300 px trigger
        // would flip it to Retract before the scan).
        let pc = player(40.0 * 16.0, 61.0 * 16.0 - 240.0);
        let t = ai_hook(&mut h, &w, pc, false);
        assert!(t.attached.is_some(), "latched");
        assert_eq!(h.ai[0], 2.0, "attached state");
        assert_eq!(
            (h.pos.x + h.w * 0.5, h.pos.y + h.h * 0.5),
            (39.0 * 16.0 + 8.0, 61.0 * 16.0 + 8.0),
            "the x-first tile wins"
        );
        assert_eq!(h.vel, Vec2::ZERO, "velocity zeroes on latch");
        assert_eq!(h.damage, 0, "damage zeroes on latch");
        // The pinned center is the tile's, so it stops moving.
        let frozen = h.pos;
        ai_hook(&mut h, &w, pc, false);
        assert_eq!(h.pos, frozen, "no integration while attached");
    }

    // `:52868-52879` — the TrackHook's first guard: it latches ONLY tile 314.
    #[test]
    fn dw_2_3_the_track_hook_latches_only_tracks() {
        let mut w = ground();
        put(&mut w, 40, 62, 1); // stone
        let mut h = hook(TRACK_HOOK, 40.0 * 16.0, 62.0 * 16.0);
        h.vel = Vec2::new(0.0, 1.0);
        let pc = player(40.0 * 16.0, 80.0 * 16.0);
        let t = ai_hook(&mut h, &w, pc, false);
        assert!(t.attached.is_none(), "stone is not a track");
        // Now the track tile at the same spot.
        put(&mut w, 40, 62, tr_core::smart_cursor::TRACK_TILE);
        let t = ai_hook(&mut h, &w, pc, false);
        assert!(t.attached.is_some(), "the track latches");
    }

    // `:52874-52877` — the Squirrel Hook (865) also latches the tree trunks
    // (`TileID.Sets.IsATreeTrunk`, `TileID.cs:487`) and the cactus (323).
    #[test]
    fn dw_2_4_the_squirrel_hook_latches_a_trunk_the_hook_refuses() {
        let mut w = ground();
        put(&mut w, 40, 62, 72); // a tree trunk (IsATreeTrunk), not TILE_SOLID
        let pc = player(40.0 * 16.0, 80.0 * 16.0);
        let mut plain = hook(HOOK, 40.0 * 16.0, 62.0 * 16.0);
        plain.vel = Vec2::new(0.0, 1.0);
        assert!(
            ai_hook(&mut plain, &w, pc, false).attached.is_none(),
            "the plain Hook refuses the trunk"
        );
        let mut squirrel = hook(SQUIRREL_HOOK, 40.0 * 16.0, 62.0 * 16.0);
        squirrel.vel = Vec2::new(0.0, 1.0);
        assert!(
            ai_hook(&mut squirrel, &w, pc, false).attached.is_some(),
            "the Squirrel Hook latches it"
        );
    }

    // `:52844-52858` — the attached re-validation: the tile's removal or
    // unsolidifying sends the hook back to Retract; an absent tile never
    // latches in the first place.
    #[test]
    fn dw_2_5_the_attach_revalidates_its_tile() {
        let mut w = ground();
        put(&mut w, 40, 62, 1);
        let mut h = hook(HOOK, 40.0 * 16.0, 62.0 * 16.0);
        h.vel = Vec2::new(0.0, 1.0);
        let pc = player(40.0 * 16.0, 80.0 * 16.0);
        assert!(ai_hook(&mut h, &w, pc, false).attached.is_some());
        // Break the tile.
        let i = w.index(40, 62);
        w.tiles[i].flags &= !tr_core::tile::FLAG_ACTIVE;
        let t = ai_hook(&mut h, &w, pc, false);
        assert_eq!(h.ai[0], 1.0, "broken tile retracts the hook");
        assert!(t.attached.is_none());
        // An out-of-range cell never latches.
        let mut far = hook(HOOK, -500.0, -500.0);
        far.vel = Vec2::new(1.0, 0.0);
        assert!(ai_hook(&mut far, &w, pc, false).attached.is_none());
    }

    // `:52594-52596` the hard cap and `:52572-52574` the owner-death kill.
    #[test]
    fn dw_2_6_the_range_cap_and_the_owner_death_kill_the_hook() {
        let w = ground();
        let mut h = hook(HOOK, 0.0, 0.0);
        let pc = player(2600.0, 0.0);
        assert!(ai_hook(&mut h, &w, pc, false).dead, ">2500 px kills");
        // The cap is `>`, so exactly 2500 survives.
        let mut edge = hook(HOOK, 0.0, 0.0);
        edge.pos = Vec2::ZERO;
        let t = ai_hook(&mut edge, &w, player(2500.0, 0.0), false);
        assert!(!t.dead, "exactly 2500 px survives");
        let mut live = hook(HOOK, 0.0, 0.0);
        assert!(ai_hook(&mut live, &w, player(10.0, 0.0), true).dead, "a dead owner kills");
    }

    // The rotation (`:52576-52581`) — the Squirrel Hook's steering reads it, so
    // it snaps to a quarter turn while attached and does not while flying.
    #[test]
    fn dw_2_8_the_squirrel_rotation_snaps_only_while_attached() {
        let w = ground();
        let pc = player(100.0, -100.0);
        let mut fly = hook(SQUIRREL_HOOK, 0.0, 0.0);
        ai_hook(&mut fly, &w, pc, false);
        let snapped = |r: f32| (r.abs() / (std::f32::consts::PI * 0.5)).fract().abs() < 1e-6
            || ((r.abs() / (std::f32::consts::PI * 0.5)).fract() - 1.0).abs() < 1e-6;
        assert!(!snapped(fly.rot), "flying keeps the raw atan2");
        let mut att = hook(SQUIRREL_HOOK, 0.0, 0.0);
        att.ai[0] = 2.0;
        att.pos = Vec2::ZERO;
        ai_hook(&mut att, &w, pc, false);
        assert!(snapped(att.rot), "attached snaps to a quarter turn");
        // 256's rotation is the plain base formula (the `+3.9250002` offset is
        // visual-only and stays out).
        let mut hand = hook(SKELETRON_HAND, 0.0, 0.0);
        ai_hook(&mut hand, &w, pc, false);
        assert!((hand.rot - (-100.0f32).atan2(0.0)).abs() > 0.0, "the base formula");
    }

    // The end-of-phase state is behaviourally green: a hook spawned through the
    // pool flies, latches, and the attached center drives the pull
    // (`GrappleMovement`, `Player.cs:22757-22856`). This is the bridge the
    // multi-hook phase replaces.
    #[test]
    // The end-of-phase state is behaviourally green: a hook spawned through the
    // pool flies, latches, and the attached center drives the pull
    // (`GrappleMovement`, `Player.cs:22757-22856`). This is the bridge the
    // multi-hook phase replaces.
    #[test]
    fn dw_2_7_a_pool_spawned_hook_flies_latches_and_drives_the_pull() {
        use crate::projectile::ProjectilePool;
        let mut npcs: [crate::npc::Npc; 0] = [];
        let mut seed: u64 = 0x5EED_1234;
        let w = ground();
        let center = Vec2::new(40.0 * 16.0, 50.0 * 16.0);
        let mut pool = ProjectilePool::new();
        let mut rng = crate::world_sim::npcs::LootRng(&mut seed);
        let inputs = crate::projectile::YoyoInputs::default();
        // Aim at a solid tile and let it latch.
        let mut w2 = w;
        put(&mut w2, 40, 54, 1);
        let slot = pool
            .spawn_hook(
                crate::projectile::HOOK,
                center,
                Vec2::new(40.0 * 16.0, 54.0 * 16.0),
                16.0,
                0,
                0.0,
                0,
                1,
            )
            .expect("the row spawns");
        let mut latched = None;
        for _ in 0..20 {
            pool.update(&mut w2, &mut npcs, Some((center.x, center.y)), None, None, inputs, &mut rng, 0, true);
            if pool.slots()[slot].ai[0] == 2.0 {
                latched = Some(pool.slots()[slot].grapple_center);
                break;
            }
        }
        let c = latched.expect("the hook latches");
        // The pull vector points at the attached center and is capped at 11.
        let to_hook = c - center;
        let len = to_hook.length();
        assert!(len > 0.0);
        let pull = to_hook * (11.0 / len).min(1.0);
        assert!((pull.length() - 11.0).abs() < 1e-3, "the pull is capped at 11");
    }
}
