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
use tr_core::tiledata::TILE_SOLID;
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

/// The `TR_HOOK=<type>` pin (the granting items land with M3-EQUIP-3).
pub(crate) static HOOK_TYPE: std::sync::OnceLock<i32> = std::sync::OnceLock::new();

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

/// `Projectile.ai[0]` states (`Projectile.cs:52565+`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum HookAi {
    /// `ai[0] == 0f`: flying out toward the aim.
    Fly,
    /// `ai[0] == 1f`: retracting toward the player.
    Retract,
    /// `ai[0] == 2f`: latched onto a tile.
    Attached,
}

/// The single grappling hook (`AI_007_GrapplingHooks` state machine).
pub(crate) struct GrappleHook {
    /// `Projectile.type` (the hook id, `TR_HOOK=`).
    pub hook_type: i32,
    pub pos: Vec2,
    pub vel: Vec2,
    /// `Projectile.ai[0]`.
    pub ai: HookAi,
    /// The latched tile coordinates (`ai == Attached` only).
    pub latched_tile: (i32, i32),
}

/// `Projectile.AI_007_GrapplingHooks_CanTileBeLatchedOnTo`
/// (`Projectile.cs:52868-52879`): `nactive()` and (solid, or tile 314 for the
/// TrackHook, or a tree trunk for the Squirrel Hook). The port's latch set:
/// solid tiles only (tile 314 / the tree trunks ride with their items).
fn can_latch(hook_type: i32, world: &World, x: i32, y: i32) -> bool {
    let Some(t) = world.tile(x, y) else {
        return false;
    };
    // `theTile.nactive()`.
    if !t.active() || t.liquid > 0 && false {
        return false;
    }
    if hook_type == 403 {
        // The TrackHook latches only onto minecart tracks (tile 314).
        return t.kind == 314;
    }
    TILE_SOLID.has(t.kind)
}

impl GrappleHook {
    /// `Projectile.NewProjectile` from `QuickGrapple` (`Player.cs:6464-6478`):
    /// the spawn at the player center, the velocity = the unit vector toward
    /// the aim scaled to `shootSpeed` (16 px/tick; the per-item values land
    /// with M3-EQUIP-3).
    pub fn fire(hook_type: i32, player_center: Vec2, aim: Vec2, shoot_speed: f32) -> Self {
        let mut dir = aim - player_center;
        let len = dir.length();
        let vel = if len < 0.001 {
            Vec2::new(1.0, 0.0) * shoot_speed
        } else {
            dir /= len;
            dir * shoot_speed
        };
        Self {
            hook_type,
            pos: player_center,
            vel,
            ai: HookAi::Fly,
            latched_tile: (0, 0),
        }
    }

    /// One 60 Hz `AI_007_GrapplingHooks` tick. `player_center` is
    /// `MountedCenter`; returns false when the hook died (the owner should
    /// drop it).
    pub fn update(&mut self, world: &World, player_center: Vec2) -> bool {
        // `:52598-52600`: hard range cap.
        if self.pos.distance(player_center) > 2500.0 {
            return false;
        }
        // `:52572-52599`: rotation toward the player (visual; not modelled).
        match self.ai {
            HookAi::Fly => {
                // `:52646-52665`: past the per-type distance → retract.
                if self.pos.distance(player_center) > hook_spec(self.hook_type).max_distance {
                    self.ai = HookAi::Retract;
                }
            }
            HookAi::Retract => {
                // `:52787-52842`: the velocity = the unit toward the player
                // x the retract speed; dead within 24f.
                let dist = self.pos.distance(player_center);
                if dist < 24.0 {
                    return false;
                }
                let to_player = (player_center - self.pos) / dist;
                self.vel = to_player * hook_spec(self.hook_type).retract_speed;
            }
            HookAi::Attached => {
                // `:52844-52858`: the latch re-validation — the tile gone
                // (broken/unsolidified) retracts the hook.
                let (tx, ty) = self.latched_tile;
                if !can_latch(self.hook_type, world, tx, ty) {
                    self.ai = HookAi::Retract;
                }
                return true;
            }
        }
        if self.ai == HookAi::Retract {
            self.pos += self.vel;
            return true;
        }
        // The Fly state: the tile latch scan (`:52666-52790`) — the hook's
        // center tile and, like the original's 16px-scan, just that tile:
        // `tileCollide = false` means the engine collision never latches; the
        // manual scan checks the tile under the hook tip.
        let tip = self.pos + self.vel;
        let tx = (tip.x / 16.0) as i32;
        let ty = (tip.y / 16.0) as i32;
        if can_latch(self.hook_type, world, tx, ty) {
            // `:52703-52719`: latch — zero velocity, center on the tile,
            // damage zeroed.
            self.pos = Vec2::new(tx as f32 * 16.0 + 8.0, ty as f32 * 16.0 + 8.0);
            self.vel = Vec2::ZERO;
            self.ai = HookAi::Attached;
            self.latched_tile = (tx, ty);
        } else {
            self.pos += self.vel;
        }
        true
    }
}

/// The per-tick pull of `Player.GrappleMovement` + `GetGrapplingForces`
/// (`Player.cs:22613-22856`): the velocity is REPLACED by the vector toward
/// the attached hook center (the average of all attached hooks — the port's
/// single hook), clamped to the per-type pull speed. Returns `false` when no
/// hook is attached (the caller falls through to the normal movement).
pub(crate) fn grapple_pull(
    player_center: Vec2,
    hook: Option<&GrappleHook>,
    input_vel: Vec2,
) -> Option<(Vec2, bool)> {
    let hook = hook?;
    if hook.ai != HookAi::Attached {
        return None;
    }
    // `GetGrapplingForces` (`Player.cs:22820-22846`): the pull is the vector
    // to the hook center clamped to the pull speed; the current velocity
    // carries over when no hook is attached (a single hook always is).
    let to_hook = hook.pos - player_center;
    let dist = to_hook.length();
    let pull_speed = hook_spec(hook.hook_type).pull_speed;
    let mut vel = if dist > 0.001 {
        to_hook * (pull_speed / dist).min(1.0)
    } else {
        Vec2::ZERO
    };
    // The pull keeps the horizontal input component when it does not fight
    // the hook (the vanilla sets the velocity outright; the input is only
    // read for the direction via `ChangeDir` — not modelled).
    let _ = input_vel;
    // `:22653-22658`: `GoingDownWithGrapple` — the pull points downward.
    let going_down = vel.y > 0.0;
    Some((vel, going_down))
}
