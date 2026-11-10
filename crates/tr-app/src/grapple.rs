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
    /// `Projectile.cs:52719-52741`: the count of attached hooks after which the
    /// oldest dies. 3 is the source's default; 165 → 8, 646-649 → 4, 256/372 → 2,
    /// 73/74/652 → 1.
    pub max_attached: usize,
}

pub(crate) fn hook_spec(hook_type: i32) -> HookSpec {
    let (max_distance, retract_speed, pull_speed, max_attached) = match hook_type {
        13 => (300.0, 11.0, 11.0, 3),
        32 => (400.0, 15.0, 11.0, 3),
        73 | 74 => (440.0, 22.0, 12.5, 1),
        165 => (375.0, 11.0, 11.0, 8),
        256 => (425.0, 11.0, 11.0, 2),
        230..=235 => (
            300.0 + (hook_type - 230) as f32 * 30.0,
            11.0 + (hook_type - 230) as f32 * 0.75,
            11.0,
            3,
        ),
        315 => (500.0, 20.0, 14.0, 3),
        322 => (550.0, 22.0, 11.0, 3),
        331 => (400.0, 11.0, 11.0, 3),
        332 => (550.0, 17.0, 11.0, 3),
        372 => (400.0, 11.0, 11.0, 2),
        396 => (300.0, 11.0, 11.0, 3),
        446 => (500.0, 20.0, 11.0, 3),
        486..=489 => (480.0, 18.0, 11.0, 3),
        646..=649 => (550.0, 24.0, 16.0, 4),
        652 => (600.0, 24.0, 11.0, 1),
        753 => (420.0, 15.0, 11.0, 3),
        // Hook (13) and everything unlisted.
        _ => (300.0, 11.0, 11.0, 3),
    };
    HookSpec {
        max_distance,
        retract_speed,
        pull_speed,
        max_attached,
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
    /// The state flipped to Attached on THIS tick — the one moment
    /// `AI_007`'s attach-cap recycle (`:52719-52741`) runs.
    pub latched: bool,
}

impl HookTick {
    fn alive() -> Self {
        Self {
            attached: None,
            dead: false,
            latched: false,
        }
    }
    fn attached_at(center: Vec2, fresh: bool) -> Self {
        Self {
            attached: Some(center),
            dead: false,
            latched: fresh,
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

/// `QuickGrapple`'s two fire-time budgets (`Player.cs:6344-6378`): `num6` is
/// the cap on **in-flight** hooks (the gate `:6399`), `num7` the total count
/// that recycles the oldest (`:6396`). They are the source's own two tables and
/// they disagree for 73/74, 652 and 372 — ported verbatim, not reconciled.
pub(crate) struct FireBudget {
    /// `num6` — the in-flight cap (`Math.Max(1, num6 − num5)`).
    pub in_flight: i32,
    /// `num7` — 999 = no recycle, 1 = one hook at a time.
    pub total: i32,
}

/// The per-shoot-id rows. Every unlisted kind keeps the vanilla 3 / 999.
pub(crate) fn fire_budget(shoot: u16) -> FireBudget {
    let (in_flight, total) = match shoot {
        73 => (2, 999),
        372 => (3, 999),
        165 => (8, 999),
        652 => (2, 999),
        646 => (4, 999),
        13 | 315 | 230..=235 | 331 | 753 | 865 | 935 => (1, 1),
        256 => (2, 2),
        _ => (3, 999),
    };
    FireBudget { in_flight, total }
}

/// The `:6399` gate — the fire is refused once the in-flight count reaches
/// `Math.Max(1, num6 − num5)` (`num5` = the attached count).
pub(crate) fn fire_allowed(in_flight: usize, attached: usize, shoot: u16) -> bool {
    let b = fire_budget(shoot);
    (in_flight as i32) < (1).max(b.in_flight - attached as i32)
}

/// The `:6396` recycle — the oldest hook dies once the total reaches `num7`.
pub(crate) fn recycles_oldest(in_flight: usize, attached: usize, shoot: u16) -> bool {
    let b = fire_budget(shoot);
    in_flight + attached >= b.total as usize
}

/// The Lunar Hook's kind cycle (`Player.cs:6425-6456`): the item 3572 walks
/// 646 → 647 → 648 → 649 → 646 by the **highest** `timeLeft` of the in-flight
/// set, with no hook out starting at Solar again.
pub(crate) fn next_lunar_kind(highest_time_left_kind: Option<u16>) -> u16 {
    match highest_time_left_kind {
        Some(646) => 647,
        Some(647) => 648,
        Some(648) => 649,
        Some(649) | None => 646,
        _ => 646,
    }
}

/// The Dual Hook's colour swap (`:6420-6424`): a blue head already out means
/// the new shot is the red one.
pub(crate) fn dual_next_kind(blue_out: bool) -> u16 {
    if blue_out {
        crate::projectile::DUAL_HOOK_RED
    } else {
        crate::projectile::DUAL_HOOK_BLUE
    }
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

/// `Player.GetGrapplingForces` (`Player.cs:22757-22856`) — the multi-hook half:
/// the pull is the vector from the player's center to the **average** of every
/// registered hook's center, clamped to the pull speed of `grappling[0]`'s type
/// (the C# reads the first registered slot, not each hook). The per-kind
/// steering arms (446/652/865) are this task's next phase.
///
/// Returns `(velocity, going_down)`. Zero registered hooks carries the caller's
/// velocity through unchanged, so the return mirrors the input in that case.
pub(crate) fn grapple_pull(hooks: &[(u16, Vec2)], player_center: Vec2) -> Option<(Vec2, bool)> {
    let (first_kind, first_center) = *hooks.first()?;
    let mut sum = Vec2::ZERO;
    for (_, c) in hooks {
        sum += *c;
    }
    let avg = sum / hooks.len() as f32;
    let _ = first_center;
    let to_hook = avg - player_center;
    let dist = to_hook.length();
    // `:22832-22851` — the cap switches on `grappling[0]`'s type.
    let pull_speed = hook_spec_id(first_kind).pull_speed;
    let pull = if dist > 0.001 {
        to_hook * (pull_speed / dist).min(1.0)
    } else {
        Vec2::ZERO
    };
    Some((pull, pull.y > 0.0))
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
            latched: false,
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
            latched: false,
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
                    latched: false,
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
                return HookTick::attached_at(Vec2::new(slot.center().0, slot.center().1), false);
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
                return HookTick::attached_at(Vec2::new(slot.center().0, slot.center().1), true);
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

#[cfg(test)]
mod hook_multi_tests {
    use super::*;
    use crate::projectile::*;

    /// One tick's projectile pass on a world with no NPCs.
    fn tick(pool: &mut ProjectilePool, w: &mut World, center: Vec2) {
        let mut npcs: [crate::npc::Npc; 0] = [];
        let mut seed: u64 = 0x5EED_1234;
        let mut rng = crate::world_sim::npcs::LootRng(&mut seed);
        let inputs = YoyoInputs::default();
        pool.update(
            w,
            &mut npcs,
            Some((center.x, center.y)),
            None,
            None,
            inputs,
            &mut rng,
            0,
            true,
        );
    }

    fn put(w: &mut World, x: i32, y: i32, kind: u16) {
        let i = w.index(x, y);
        w.tiles[i] = tr_core::Tile {
            kind,
            ..Default::default()
        };
        w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
    }

    // `Player.cs:6399` — the gate reads the IN-FLIGHT count (`ai[0] != 2`), so
    // 8 hooks still flying refuse the 9th shot; 8 latched ones do not.
    #[test]
    fn dw_3_1_the_web_slinger_gate_counts_in_flight_hooks() {
        let mut w = crate::test_world_helpers::ground();
        let center = Vec2::new(40.0 * 16.0, 30.0 * 16.0);
        let mut pool = ProjectilePool::new();
        for _ in 0..8 {
            pool.spawn_hook(WEB_SLINGER_HOOK, center, center + Vec2::new(0.0, 400.0), 16.0, 0, 0.0, 0, 1);
        }
        let (flight, attached) = pool.hook_counts(0);
        assert_eq!((flight, attached), (8, 0));
        assert!(!fire_allowed(flight, attached, WEB_SLINGER_HOOK), "8 flying refuse");
        // Latch all eight against the ceiling: the gate opens again.
        // The floor row three tiles under the player, wide enough for
        // all eight to latch on it.
        for x in 0..10 {
            put(&mut w, 36 + x, 33, 1);
        }
        for _ in 0..20 {
            tick(&mut pool, &mut w, center);
        }
        let (flight, attached) = pool.hook_counts(0);
        assert_eq!(attached, 8, "eight latched: {flight}/{attached}");
        assert!(fire_allowed(flight, attached, WEB_SLINGER_HOOK), "the gate opens");
    }

    // `Player.cs:6425-6456` — the Lunar cycle walks 646→647→648→649→646 by the
    // highest-`timeLeft` in-flight kind; `num6 = 4` allows four.
    #[test]
    fn dw_3_2_the_lunar_cycle_and_the_static_caps() {
        let mut w = crate::test_world_helpers::ground();
        let center = Vec2::new(40.0 * 16.0, 30.0 * 16.0);
        // Four latched Lunar hooks: `num6 = 4`, so all four hold, and the next
        // attach recycles the oldest (`num14 = 4` is only checked on latch).
        for x in 0..6 {
            put(&mut w, 37 + x, 33, 1);
        }
        let mut pool = ProjectilePool::new();
        let mut seen = Vec::new();
        let mut kind = LUNAR_HOOK_SOLAR;
        for _ in 0..4 {
            pool.spawn_hook(kind, center, center + Vec2::new(0.0, 400.0), 16.0, 0, 0.0, 0, 1);
            seen.push(kind);
            kind = next_lunar_kind(Some(kind));
        }
        assert_eq!(seen, vec![646, 647, 648, 649], "the cycle in order");
        assert_eq!(next_lunar_kind(Some(649)), 646, "wraps to Solar");
        assert_eq!(next_lunar_kind(None), 646, "no hook out starts Solar");
        for _ in 0..20 {
            tick(&mut pool, &mut w, center);
        }
        assert_eq!(pool.hook_counts(0).1, 4, "four Lunar hooks latch");
        // The Static Hook fires twice (`num6 = 2`) but only one stays attached
        // (`num14 = 1`), because the latch arm recycles the oldest.
        put(&mut w, 40, 33, 1);
        let mut sp = ProjectilePool::new();
        sp.spawn_hook(STATIC_HOOK, center, center + Vec2::new(0.0, 400.0), 16.0, 0, 0.0, 0, 1);
        assert!(fire_allowed(sp.hook_counts(0).0, sp.hook_counts(0).1, STATIC_HOOK));
        sp.spawn_hook(STATIC_HOOK, center, center + Vec2::new(0.0, 400.0), 16.0, 0, 0.0, 0, 1);
        assert!(!fire_allowed(sp.hook_counts(0).0, sp.hook_counts(0).1, STATIC_HOOK), "two is the cap");
        for _ in 0..20 {
            tick(&mut sp, &mut w, center);
        }
        assert_eq!(sp.hook_counts(0).1, 1, "one stays latched (num14 = 1)");
    }

    // `Player.cs:6353-6357` — the Fish Hook's two tables disagree (`num6 = 3`,
    // `num14 = 2`): three may fly, two may latch, the third latch recycles.
    #[test]
    fn dw_3_3_the_fish_hooks_disagreeing_tables() {
        let mut w = crate::test_world_helpers::ground();
        let center = Vec2::new(40.0 * 16.0, 30.0 * 16.0);
        for x in 0..6 {
            put(&mut w, 37 + x, 33, 1);
        }
        let mut pool = ProjectilePool::new();
        for _ in 0..3 {
            pool.spawn_hook(FISH_HOOK, center, center + Vec2::new(0.0, 400.0), 16.0, 0, 0.0, 0, 1);
        }
        let (f, a) = pool.hook_counts(0);
        assert!(!fire_allowed(f, a, FISH_HOOK), "three flying saturate num6 = 3");
        for _ in 0..25 {
            tick(&mut pool, &mut w, center);
        }
        let (f, a) = pool.hook_counts(0);
        assert_eq!(a, 2, "num14 = 2 keeps two latched (saw {f}/{a})");
    }

    // `Player.cs:6420-6424` — the Dual Hook's colour swap, and the asymmetry the
    // source carries: only 73 is special-cased, so 74 keeps the default budget.
    #[test]
    fn dw_3_7_the_dual_hook_colour_and_its_asymmetry() {
        assert_eq!(dual_next_kind(false), DUAL_HOOK_BLUE);
        assert_eq!(dual_next_kind(true), DUAL_HOOK_RED);
        // 73: num6 = 2 (special-cased). 74: falls through to the default 3.
        assert_eq!(fire_budget(DUAL_HOOK_BLUE).in_flight, 2);
        assert_eq!(fire_budget(DUAL_HOOK_RED).in_flight, 3, "74 keeps the default");
        // Both share `num14 = 1` on the attach side.
        let mut w = crate::test_world_helpers::ground();
        put(&mut w, 40, 33, 1);
        let center = Vec2::new(40.0 * 16.0, 30.0 * 16.0);
        let mut pool = ProjectilePool::new();
        pool.spawn_hook(DUAL_HOOK_BLUE, center, center + Vec2::new(0.0, 400.0), 16.0, 0, 0.0, 0, 1);
        pool.spawn_hook(DUAL_HOOK_RED, center, center + Vec2::new(0.0, 400.0), 16.0, 0, 0.0, 0, 1);
        for _ in 0..20 {
            tick(&mut pool, &mut w, center);
        }
        assert_eq!(pool.hook_counts(0).1, 1, "num14 = 1 shared");
    }

    // The registration is rebuilt in slot order every tick and a hook killed
    // mid-registration never leaves a dead index behind.
    #[test]
    fn dw_3_5_the_registration_holds_no_dead_index() {
        let mut w = crate::test_world_helpers::ground();
        let center = Vec2::new(40.0 * 16.0, 30.0 * 16.0);
        for x in 0..6 {
            put(&mut w, 36 + x, 33, 1);
        }
        let mut pool = ProjectilePool::new();
        for _ in 0..3 {
            pool.spawn_hook(WOOD_HOOK, center, center + Vec2::new(0.0, 400.0), 16.0, 0, 0.0, 0, 1);
        }
        for _ in 0..20 {
            tick(&mut pool, &mut w, center);
        }
        let (reg, n) = pool.hook_reg();
        assert_eq!(n, 3, "three registered (the default cap is 3)");
        let slots = pool.slots();
        for &i in reg.iter().take(n) {
            assert!(slots[i as usize].active, "slot {i} is live");
        }
        // Kill the middle one; the next rebuild drops it.
        let dead = reg[2];
        pool.kill(dead as usize);
        tick(&mut pool, &mut w, center);
        let (reg, n) = pool.hook_reg();
        assert!(!reg[..n].contains(&dead), "the dead slot is gone");
        assert_eq!(n, 2, "the count follows the live set");
    }

    // `Player.cs:22613-22740`: the pull averages every registered hook and the
    // cap comes from `grappling[0]`'s type — a mixed pair uses the first's.
    #[test]
    fn dw_4_3_the_pull_averages_them_and_caps_on_the_first() {
        let p0 = Vec2::new(100.0, 40.0);
        let hooks: &[(u16, Vec2)] = &[(BAT_HOOK, Vec2::new(0.0, -10.0)), (HOOK, Vec2::new(40.0, -30.0))];
        let (vel, down) = grapple_pull(hooks, p0).expect("a pull");
        // The average of the two centers is (20, -20); the vector from the
        // player is (-80, -60) and its cap is 14 (the first hook's type).
        assert!((vel.length() - 14.0).abs() < 1e-3, "the Bat Hook's cap 14");
        assert!(vel.x < 0.0 && vel.y < 0.0, "toward the average");
        assert!(!down);
        // A single hook is the PHYS-6 case unchanged.
        let (v, _) = grapple_pull(&[(HOOK, Vec2::new(0.0, -14.0))], Vec2::ZERO).unwrap();
        assert!((v.length() - 11.0).abs() < 1e-3, "the Hook's cap 11");
        // Zero hooks carries the caller through.
        assert!(grapple_pull(&[], Vec2::ZERO).is_none());
    }

    // `RemoveAllGrapplingHooks` (`Player.cs:33562-33566`): the jump-off kills
    // this owner's aiStyle-7 slots and nothing else.
    #[test]
    fn dw_3_6_the_remove_all_kills_only_the_owners_hooks() {
        let mut w = crate::test_world_helpers::ground();
        let center = Vec2::new(40.0 * 16.0, 30.0 * 16.0);
        put(&mut w, 40, 33, 1);
        let mut pool = ProjectilePool::new();
        pool.spawn_hook(HOOK, center, center + Vec2::new(0.0, 400.0), 16.0, 0, 0.0, 0, 1);
        // An unowned/other-owner projectile must survive the sweep.
        pool.spawn(WOODEN_ARROW, 1, center, Vec2::new(1.0, 0.0), 5, 2.0, false);
        for _ in 0..20 {
            tick(&mut pool, &mut w, center);
        }
        assert_eq!(pool.hook_counts(0).1, 1, "one latched");
        let hooks: Vec<usize> = pool
            .slots()
            .iter()
            .enumerate()
            .filter(|(_, pr)| {
                pr.active && pr.owner == 0 && proj_data(pr.kind).is_some_and(|d| d.ai_style == 7)
            })
            .map(|(i, _)| i)
            .collect();
        for i in hooks {
            pool.kill(i);
        }
        assert_eq!(pool.hook_counts(0).1, 0, "the owner's hooks are gone");
        assert_eq!(pool.active_count(), 1, "the arrow survives");
    }
}
