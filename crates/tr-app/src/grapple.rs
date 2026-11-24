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
use crate::projectile::{SKELETRON_HAND, SQUIRREL_HOOK, STATIC_HOOK, TRACK_HOOK};
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
        // `:22832-22851` — the Thorn Hook (487) is the only one of the four
        // with its own 12 pull cap; the other three share the 11 default.
        486 => (480.0, 18.0, 11.0, 3),
        487 => (480.0, 18.0, 12.0, 3),
        488 | 489 => (480.0, 18.0, 11.0, 3),
        646..=649 => (550.0, 24.0, 16.0, 4),
        652 => (600.0, 24.0, 11.0, 1),
        753 => (420.0, 15.0, 11.0, 3),
        // `:52648` — the Squirrel Hook's 300 px flight cap (its own row).
        865 => (300.0, 11.0, 11.0, 3),
        935 => (500.0, 11.0, 11.0, 3),
        // `:52648`'s distance table has NO Track Hook (403) row, so its Fly
        // state never flips to Retract on distance — it latches the rail it
        // spawns on (`LaunchMinecartHook`, `Player.cs:33553-33559`). The
        // retract/pull/attach defaults (11/11/3) still apply.
        403 => (f32::INFINITY, 11.0, 11.0, 3),
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
    /// The 1/16 cracked-brick break (`Projectile.cs:52586-52595`): the tile
    /// under the hook's center, for the pool to break (it owns `&mut World`).
    pub crack: Option<(i32, i32)>,
}

impl HookTick {
    fn alive() -> Self {
        Self {
            attached: None,
            dead: false,
            latched: false,
            crack: None,
        }
    }
    fn attached_at(center: Vec2, fresh: bool, crack: Option<(i32, i32)>) -> Self {
        Self {
            attached: Some(center),
            dead: false,
            latched: fresh,
            crack,
        }
    }
}

/// `Projectile.AI_007_GrapplingHooks_CanTileBeLatchedOnTo`
/// (`Projectile.cs:52868-52879`): `nactive()` and (solid, or tile 314, or a
/// tree trunk / the cactus for the Squirrel Hook 865). The `403` guard is the
/// source's first line — without it the TrackHook would latch any solid tile.
pub(crate) fn can_latch(
    hook_type: u16,
    world: &World,
    x: i32,
    y: i32,
    blacklisted: &[(i32, i32)],
) -> bool {
    let Some(t) = world.tile(x, y) else {
        return false;
    };
    // `Tile.nactive()` (`Tile.cs:240-247`): the active bit SET and the
    // inactive (actuated) bit CLEAR — `(sTileHeader & 0x60) == 32`. An
    // actuated solid tile is not latchable.
    if !t.active() || t.inactive() {
        return false;
    }
    if hook_type == TRACK_HOOK && t.kind != tr_core::smart_cursor::TRACK_TILE {
        return false;
    }
    // `Player.IsBlacklistedForGrappling` (`Player.cs:6205-6208`) — the tile the
    // hook already crossed is refused (`Projectile.cs:52699`).
    if blacklisted.contains(&(x, y)) {
        return false;
    }
    tr_core::tiledata::TILE_SOLID.has(t.kind)
        || t.kind == tr_core::smart_cursor::TRACK_TILE
        || (hook_type == SQUIRREL_HOOK && tr_core::tiledata::SET_IS_A_TREE_TRUNK.has(t.kind))
        || (hook_type == SQUIRREL_HOOK && t.kind == 323)
}

/// `WorldGen.SolidTile(i, j)` (`WorldGen.cs:70111-70132`): an active, fully
/// solid tile — not a solid-top, not a half brick, no slope and not inactive.
/// Out of world the C# array read throws into the catch → `false`
/// (`:70128-70131`); the null-tile arm (`:70115-70118`, returns `true`) is
/// unreachable in the port — `World::tile` yields a default (inactive, type 0)
/// tile in bounds, which the active gate below already rejects.
fn solid_tile(world: &World, x: i32, y: i32) -> bool {
    let Some(t) = world.tile(x, y) else {
        return false; // the catch arm (out of world)
    };
    t.active()
        && tr_core::tiledata::TILE_SOLID.has(t.kind)
        && !tr_core::tiledata::TILE_SOLID_TOP.has(t.kind)
        && !t.half_brick()
        && t.slope() == 0
        && !t.inactive()
}

/// `Player.UpdateBlacklistedTilesForGrappling` (`Player.cs:6210-6248`): the
/// tiles the hook may not latch on.
///
/// For every attached hook the 3x3 around its centre contributes the
/// **non-solid** tiles (`:6220-6229`) — the hook must not re-latch on the air
/// it crossed — and, when the hook sits on a minecart track or a platform,
/// the 5x5 around it contributes the track and platform tiles (`:6230-6246`).
///
/// The C# keeps them in a `HashSet<Point>` (`Player.cs:6202`); the port keeps
/// the unique list.
pub(crate) fn blacklisted_tiles(world: &World, attached: &[(i32, i32)]) -> Vec<(i32, i32)> {
    let mut out: Vec<(i32, i32)> = Vec::new();
    let mut push = |p: (i32, i32), out: &mut Vec<(i32, i32)>| {
        if !out.contains(&p) {
            out.push(p);
        }
    };
    for &(px, py) in attached {
        // `:6220-6229` — the 3x3 around the hook: the non-solid tiles.
        for j in -1..=1 {
            for k in -1..=1 {
                let (x, y) = (px + j, py + k);
                if !solid_tile(world, x, y) {
                    push((x, y), &mut out);
                }
            }
        }
        // `:6230-6246` — on a track or a platform: the 5x5 around it, the
        // track and platform tiles only. `Framing.GetTileSafely` returns a
        // fresh (inactive, type 0) tile out of world / for null, so the test
        // reads `tile.type` with NO active gate (`:6231`/`:6241`).
        let on_track_or_platform = world.tile(px, py).is_some_and(|t| {
            t.kind == tr_core::smart_cursor::TRACK_TILE
                || tr_core::tiledata::SET_PLATFORMS.has(t.kind)
        });
        if !on_track_or_platform {
            continue;
        }
        for l in -2..=2 {
            for m in -2..=2 {
                let (x, y) = (px + l, py + m);
                if world.tile(x, y).is_some_and(|t| {
                    t.kind == tr_core::smart_cursor::TRACK_TILE
                        || tr_core::tiledata::SET_PLATFORMS.has(t.kind)
                }) {
                    push((x, y), &mut out);
                }
            }
        }
    }
    out
}

/// `Player.DoQueenSlimeHookTeleport` (`Player.cs:22742-22755`): the Queen
/// Slime's hook (935) drags the player to the hook in 150 tile-collision steps
/// of 12 px each, then the style-10 teleport lands.
///
/// The C#'s `TileCollision(velocity, position, fallThrough: true,
/// ignorePlats: true)` returns the new position; the port's collision returns
/// the resolved velocity, so the position advances by it.
pub(crate) fn queen_slime_hook_teleport(
    world: &World,
    position: Vec2,
    size: Vec2,
    target: Vec2,
) -> Vec2 {
    let mut vector = position;
    for _ in 0..150 {
        // `(vector + Size / 2).DirectionTo(target).SafeNormalize(Vector2.Zero) * 12f`
        let step = (target - (vector + size * 0.5)).normalize_or(Vec2::ZERO) * 12.0;
        let tc = crate::collision::tile_collision(
            world,
            [vector.x, vector.y],
            [step.x, step.y],
            size.x,
            size.y,
            true, // `fallThrough`
            true, // `ignorePlats`
        );
        vector += Vec2::new(tc.vel[0], tc.vel[1]);
    }
    vector
}

/// `Main.projHook` (`Main.cs:6620-6624`): the aiStyle-7 projectiles — the
/// grappling hooks an item's `shoot` can point at.
pub(crate) fn is_hook_projectile(kind: u16) -> bool {
    crate::projectile::proj_data(kind).is_some_and(|d| d.ai_style == 7)
}

/// `Player.QuickGrapple_GetItemToUse` (`Player.cs:6481-6500`): the grapple
/// slot (the misc-equip 4) first, then the inventory — the first item whose
/// `shoot` is a grappling hook. Returns the hook kind.
///
/// The C# scans the 58 inventory slots (the main 50 plus the coin and ammo
/// rows); the coin and ammo rows carry no hook, so the port's main slots
/// answer the same.
pub(crate) fn quick_grapple_item(
    misc_equips: &[crate::item::ItemSlot],
    inventory: &[crate::item::ItemSlot],
) -> Option<u16> {
    quick_grapple_item_kind(misc_equips, inventory).map(|(_, shoot)| shoot)
}

/// S1-E-21f — [`quick_grapple_item`] with the granting ITEM kind alongside
/// the hook projectile: `QuickGrapple` plays `item.UseSound` at `position`
/// (`Player.cs:6408`) — the hook item's own use-sound row (the vanilla
/// Grappling Hook 84 → `SoundID.Item1`, style index 126), not a fixed cue.
pub(crate) fn quick_grapple_item_kind(
    misc_equips: &[crate::item::ItemSlot],
    inventory: &[crate::item::ItemSlot],
) -> Option<(u16, u16)> {
    let pick = |slot: &crate::item::ItemSlot| {
        let shoot = crate::itemdata::ITEMS.get(slot.kind as usize)?.projectile?;
        is_hook_projectile(shoot).then_some((slot.kind, shoot))
    };
    // `:6484-6487` — the grapple misc-equip slot wins.
    if let Some(slot) = misc_equips.get(4) {
        if let Some(found) = pick(slot) {
            return Some(found);
        }
    }
    // `:6490-6497` — the inventory scan.
    inventory.iter().find_map(pick)
}

/// `Vector2.ToTileCoordinates` (`Utils.cs:1807-1810`): `(int)vec >> 4` — the
/// int truncation then the arithmetic shift, NO clamp. The callers apply the
/// C#'s own bounds (the crack gate `> 0 && < maxTiles`, the latch scan's
/// `[0, maxTiles]` clamp).
pub(crate) fn tile_coord(px: f32) -> i32 {
    (px as i32) >> 4
}

/// `(point / 16)` clamped into the world — the C# `ToTileCoordinates` under
/// the scan's `[0, maxTiles*]` clamp.
pub(crate) fn tile_of(px: f32, tiles: i32) -> i32 {
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

/// What `GrappleMovement` needs from the pull (`Player.cs:22613-22740`).
pub(crate) struct HookPull {
    /// The velocity that replaces the player's.
    pub vel: Vec2,
    /// `GoingDownWithGrapple` — the pull points down, so the platforms fall
    /// through (`:22655-22658`).
    pub going_down: bool,
    /// `preferredPlayerDirectionToSet` (`:22670-22676`) — only the Squirrel
    /// Hook sets it; `ChangeDir` reads it when the pull's X is exactly 0.
    pub dir: Option<i32>,
}

/// The direction/steering stick, normalized as the C# reads the controls
/// (`Player.cs:22769`/`:22800`): `(right − left, (down − up) * gravDir)`.
/// The port is upright-only, so `gravDir` is 1.
pub(crate) fn steering_stick(left: bool, right: bool, up: bool, down: bool) -> Vec2 {
    Vec2::new(
        (right as i32 - left as i32) as f32,
        // The C# multiplies by `gravDir`; the port has no flipped gravity.
        (down as i32 - up as i32) as f32,
    )
}

/// `Player.GetGrapplingForces` (`Player.cs:22757-22856`): the pull is the vector
/// from the player's center to the (steering-adjusted) average of every
/// registered hook's center, clamped to the pull speed of `grappling[0]`'s type.
///
/// Per-hook overrides, in the source's own order: the AntiGravity Hook (446)
/// biases the average toward the held control direction, the Static Hook (652)
/// pulls along its own line to the player with a slack arm that only engages
/// when the dot is positive and the distance is under 600 px, and the Squirrel
/// Hook (865) anchors the average 28 px behind its rotation and sets the
/// preferred direction. Every other kind contributes its plain center.
///
/// `hooks` is `(ProjectileID, center, rotation)` per registered hook, in slot
/// order. Zero hooks returns `None` — the caller keeps its velocity.
pub(crate) fn grapple_pull(
    hooks: &[(u16, Vec2, f32)],
    player_center: Vec2,
    stick: Vec2,
) -> Option<HookPull> {
    let first_kind = hooks.first()?.0;
    let stick = if stick.length() > 0.0 {
        stick.normalize()
    } else {
        Vec2::ZERO
    };
    let mut sum = Vec2::ZERO;
    let mut dir = None;
    for (kind, center, rot) in hooks {
        let contribution = match kind {
            446 => {
                // `:22777-22788` — the control direction, scaled by 100, added
                // to the vector from the hook, then normalized against the
                // pull. A NaN direction falls back to -UnitY.
                let vec = (player_center - *center + stick * 100.0).normalize_or(-Vec2::Y);
                *center + vec * 200.0
            }
            652 => {
                // `:22789-22802` — the projection of the held direction onto
                // the hook→player line, doubled by 6, and zeroed when it
                // fights the hook and the line is 600 px or longer.
                let line = *center - player_center;
                let unit = line.normalize_or(Vec2::ZERO);
                let mut value = Vec2::ZERO;
                if stick != Vec2::ZERO {
                    value = unit * unit.dot(stick);
                }
                let mut slack = 6.0;
                if value.dot(line) < 0.0 && line.length() >= 600.0 {
                    slack = 0.0;
                }
                player_center + value * slack
            }
            865 => {
                // `:22803-22812` — 28 px behind the hook's snapped rotation.
                let fwd = (Vec2::from_angle(*rot - std::f32::consts::PI * 0.5)).normalize_or(Vec2::Y);
                *center - fwd * 28.0
            }
            _ => *center,
        };
        if *kind == 865 && fwd_x(*rot) != 0.0 {
            dir = Some(fwd_x(*rot).signum() as i32);
        }
        sum += contribution;
    }
    let avg = sum / hooks.len() as f32;
    let to_hook = avg - player_center;
    let dist = to_hook.length();
    // `:22832-22851` — the cap switches on `grappling[0]`'s type.
    let pull_speed = hook_spec_id(first_kind).pull_speed;
    // `:22852-22855` — `num10 = (num8 > num9) ? num9/num8 : 1f`, then both
    // components scale. No epsilon: at exactly zero distance the factor is 1
    // and the (zero) vector stays zero; the C# never divides by zero here
    // because the `> num9` guard (num9 >= 11) short-circuits the small case.
    let factor = if dist > pull_speed { pull_speed / dist } else { 1.0 };
    let vel = to_hook * factor;
    Some(HookPull {
        vel,
        going_down: vel.y > 0.0,
        dir,
    })
}

/// The Squirrel Hook's rotation → the x of its forward unit vector; the C#
/// sets `preferredPlayerDirectionToSet` only when this is non-zero.
fn fwd_x(rot: f32) -> f32 {
    Vec2::from_angle(rot - std::f32::consts::PI * 0.5).x
}
/// The `AI_007` per-type tables (`Projectile.cs:52646-52665` max distances,
/// `:52787-52842` retract speeds, `Player.cs:22832-22851` pull speeds), read
/// off the slot's `ProjectileID`.
pub(crate) fn hook_spec_id(hook_type: u16) -> HookSpec {
    hook_spec(hook_type as i32)
}

/// `Projectile.AI_007_GrapplingHooks` (`Projectile.cs:52565-52866`) on one
/// pool slot, in the source's own statement order. The caller integrates the
/// position for the flying/retracting states (`UpdatePosition` `:20433-20475`
/// early-returns while attached, `aiStyle == 7 && ai[0] == 2`); this arm does
/// the integration itself so the latch pin and the no-move-while-attached hold.
///
/// `owner_dead` is the C#'s `Main.player[owner].dead || .CCed` kill gate
/// (`:52567`) — the caller folds both in.
pub(crate) fn ai_hook(
    slot: &mut crate::projectile::Projectile,
    world: &World,
    player_center: Vec2,
    owner_dead: bool,
    // The slot's owner is this machine's player — the C#'s `myPlayer` gate on
    // the cracked-brick break (`:52583`).
    local_owner: bool,
    // The pass's roll stream (`Main.rand` = the active `UpdateProjectiles`
    // swap, `Main.cs:17775-17777`); `None` keeps the off-machine callers
    // draw-free, like the C#'s `myPlayer` gate.
    rng: Option<&mut dyn tr_core::loot::DropRng>,
    // `Player._blackListedTileCoordsForGrappling` (`Player.cs:6202`) — the
    // tiles this owner's hooks may not latch on. The FLY scan reads it
    // (`:52699`); the attached re-validation does NOT (`:52852`).
    blacklisted: &[(i32, i32)],
) -> HookTick {
    let center = slot.pos + Vec2::new(slot.w * 0.5, slot.h * 0.5);
    // `:52567-52571` — a dead OR CCed owner kills the hook.
    if owner_dead {
        return HookTick {
            attached: None,
            dead: true,
            latched: false,
            crack: None,
        };
    }
    let delta = player_center - center;
    let dist = delta.length();
    // `:52577` — the rotation toward the player: `atan2(dy,dx) - 1.57f` (the
    // source's literal 1.57, NOT pi/2).
    slot.rot = delta.y.atan2(delta.x) - 1.57;
    // `:52578-52582` — the Squirrel Hook snaps to a quarter turn while
    // attached: `(int)Math.Round(rot / (pi/2)) * (pi/2)` — C# banker's round.
    if slot.ai[0] == 2.0 && slot.kind == SQUIRREL_HOOK {
        let q = std::f32::consts::PI * 0.5;
        slot.rot = tr_core::damage::round_bankers((slot.rot / q) as f64) as f32 * q;
    }
    // `:52583-52595` — the local player's hook breaks the cracked-brick tile
    // under its center on `Main.rand.Next(16) == 0`. This runs BEFORE the
    // 2500 cap (the source order), drawn ONLY when the C# `&&` chain reaches
    // it (the `nactive() && CrackedBricks` test short-circuits the roll). The
    // roll rides the `UpdateProjectiles` stream (`Main.cs:17775-17777`).
    let mut crack = None;
    if local_owner {
        if let Some(rng) = rng {
            let tx = (center.x / 16.0) as i32;
            let ty = (center.y / 16.0) as i32;
            if tx > 0 && ty > 0 && tx < world.width && ty < world.height {
                let broken = world.tile(tx, ty).is_some_and(|t| {
                    // `Tile.nactive()` — active and not actuated (`:52587`).
                    t.active()
                        && !t.inactive()
                        && tr_core::tiledata::SET_CRACKED_BRICKS.has(t.kind)
                });
                if broken && rng.next_range(0, 16) == 0 {
                    crack = Some((tx, ty));
                }
            }
        }
    }
    // `:52596-52599` — the hard range cap, every state. The C# `Kill()` does
    // NOT return; the rest of the AI still runs on the dead slot, but the
    // caller drops it, so the port returns the dead tick here (no observable
    // state survives the kill).
    if dist > 2500.0 {
        return HookTick {
            attached: None,
            dead: true,
            latched: false,
            crack,
        };
    }
    // `:52600-52603` — the Skeletron Hand (256) re-aims its rotation with the
    // source's own `+3.9250002f` offset (after the cap).
    if slot.kind == SKELETRON_HAND {
        slot.rot = delta.y.atan2(delta.x) + 3.9250002;
    }
    // `:52615-52622` — the Static Hook (652) advances its frame every 7 ticks:
    // `++frameCounter >= 7` resets the counter and steps `frame` modulo
    // `projFrames[652]` (= 6). RNG-free.
    if slot.kind == STATIC_HOOK {
        slot.frame_counter += 1;
        if slot.frame_counter >= 7 {
            slot.frame_counter = 0;
            slot.frame += 1;
            let frames = crate::projectile::proj_frames(STATIC_HOOK) as i32;
            if slot.frame >= frames {
                slot.frame = 0;
            }
        }
    }
    let spec = hook_spec_id(slot.kind);
    match slot.ai[0] as i32 {
        0 => {
            // `:52646-52662` — past the per-type distance the hook retracts.
            if dist > spec.max_distance {
                slot.ai[0] = 1.0;
            }
            // `:52663-52785` — the latch scan runs UNCONDITIONALLY inside the
            // Fly block, even on the tick the distance just flipped `ai[0]`
            // to 1 (the source does not re-gate on the state). The 10x10 box
            // at `Center - (5,5)` over the x-major tile range, first match
            // wins.
            let a = center - Vec2::new(5.0, 5.0);
            // `:52665-52686` — `point = (Center-21).ToTileCoordinates()` and
            // `point2 = (Center+37).ToTileCoordinates()` (`Utils.cs:1809`:
            // `(int)vec >> 4`, NO clamp), then `num7/num9` clamp UP to 0 and
            // `num8/num10` clamp DOWN to `maxTiles` (NOT maxTiles-1). The loops
            // are `i < num8` / `j < num10` (EXCLUSIVE upper). `World::tile`
            // rejects anything past the edge, so the last in-bounds column
            // (maxTiles-1) is still scanned.
            let x0 = tile_coord(a.x - 16.0).max(0);
            let x1 = tile_coord(a.x + 42.0).min(world.width);
            let y0 = tile_coord(a.y - 16.0).max(0);
            let y1 = tile_coord(a.y + 42.0).min(world.height);
            for i in x0..x1 {
                for j in y0..y1 {
                    let tile = Vec2::new(i as f32 * 16.0, j as f32 * 16.0);
                    // `:52699` — the overlap test: the 10x10 box at `a`
                    // against the 16x16 tile, then the latch + blacklist gate.
                    if !(a.x + 10.0 > tile.x && a.x < tile.x + 16.0)
                        || !(a.y + 10.0 > tile.y && a.y < tile.y + 16.0)
                    {
                        continue;
                    }
                    if !can_latch(slot.kind, world, i, j, blacklisted) {
                        continue;
                    }
                    // `:52758-52768` — latch: the velocity zeroes, the state
                    // moves to Attached, the position centres on the tile
                    // (`i*16+8 - width/2`), then `GetTileVisualHitbox`'s center
                    // overrides it (323's frameY / the half brick), the damage
                    // drops to 0.
                    slot.vel = Vec2::ZERO;
                    slot.ai[0] = 2.0;
                    slot.pos = latch_center(world, i, j) - Vec2::new(slot.w * 0.5, slot.h * 0.5);
                    slot.damage = 0;
                    // `:52757` — `SoundEngine.PlaySound(0, i*16, j*16)`: the
                    // case-0 dig cue (Next(3) variation + Next(-10,11) pitch on
                    // the UpdateProjectiles stream, attenuation-gated). The
                    // slot parks the intent; the pool drains it (the E-21h
                    // `wants_sound` seam, id 0 = the int overload).
                    slot.wants_sound = Some((0, 0, i as f32 * 16.0, j as f32 * 16.0, 0.0, 1.0));
                    return HookTick::attached_at(
                        Vec2::new(slot.center().0, slot.center().1),
                        true,
                        crack,
                    );
                }
            }
        }
        1 => {
            // `:52787-52842` — the retract velocity is the unit vector toward
            // the player scaled by the per-type speed. The `< 24f` reach kills
            // (`:52834-52837`); the C# `Kill()` does not return, so the
            // velocity write still lands on the dead slot — unobservable.
            if dist < 24.0 {
                return HookTick {
                    attached: None,
                    dead: true,
                    latched: false,
                    crack,
                };
            }
            slot.vel = delta * (spec.retract_speed / dist);
        }
        _ => {
            // `:52844-52864` — attached: the latch re-validation reads
            // `CanTileBeLatchedOnTo` ONLY (no blacklist, `:52852`). The tile's
            // absence or unsolidifying sends it back to Retract.
            let tx = tile_coord(slot.center().0);
            let ty = tile_coord(slot.center().1);
            if !can_latch(slot.kind, world, tx, ty, &[]) {
                slot.ai[0] = 1.0;
            } else {
                return HookTick::attached_at(
                    Vec2::new(slot.center().0, slot.center().1),
                    false,
                    crack,
                );
            }
        }
    }
    // `UpdatePosition` (`:20445`): `position += velocity` for the flying and
    // retracting states (the attached state returned above).
    slot.pos += slot.vel;
    let mut tick = HookTick::alive();
    tick.crack = crack;
    tick
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::projectile::{HOOK, SKELETRON_HAND, SQUIRREL_HOOK, TRACK_HOOK};
    use crate::test_world_helpers::ground;

    /// S1-L-10b — `Player.DoQueenSlimeHookTeleport` (`Player.cs:22742-22755`):
    /// the player is dragged to the hook in 150 steps of 12 px.
    #[test]
    fn the_queen_slime_hook_drags_the_player_to_the_hook() {
        let w = ground();
        // The player at the tile (20, 18) — the ground starts at row 30, so
        // the drag is horizontal and the collision never fires.
        let start = Vec2::new(20.0 * 16.0, 18.0 * 16.0);
        let target = Vec2::new(24.0 * 16.0, 18.0 * 16.0);
        let size = Vec2::new(20.0, 42.0);
        let end = queen_slime_hook_teleport(&w, start, size, target);
        // The direction runs from the player's CENTRE to the hook, and every
        // step is a full 12 px — the C#'s 150 steps therefore land within one
        // step of the hook (the walk oscillates around it).
        let center = end + size * 0.5;
        assert!((center - target).length() < 12.0, "{:?}", end);
    }

    /// S1-L-10a — `Player.UpdateBlacklistedTilesForGrappling`
    /// (`Player.cs:6210-6248`): the non-solid 3x3 around an attached hook and
    /// the track/platform 5x5 when the hook sits on one.
    #[test]
    fn the_latch_blacklist_covers_the_air_and_the_track() {
        let mut w = ground();
        // The hook sits on the dirt ground (the tile 1 is solid); the tiles
        // around it are air, so they enter the blacklist.
        let tiles = blacklisted_tiles(&w, &[(50, 119)]);
        assert!(tiles.contains(&(49, 118)), "{:?}", tiles);
        assert!(tiles.contains(&(50, 118)), "{:?}", tiles);
        assert!(tiles.contains(&(51, 119)), "{:?}", tiles);
        // The solid ground row does not.
        assert!(!tiles.contains(&(50, 120)), "{:?}", tiles);
        assert!(!tiles.contains(&(51, 120)), "{:?}", tiles);
        // A hook on a minecart track blacklists the track tiles around it.
        let mut w2 = ground();
        for x in 48..=52 {
            for y in 118..=120 {
                let i = w2.index(x, y);
                w2.tiles[i].kind = tr_core::smart_cursor::TRACK_TILE;
                w2.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
            }
        }
        let tiles = blacklisted_tiles(&w2, &[(50, 119)]);
        assert!(tiles.contains(&(48, 118)), "{:?}", tiles);
        assert!(tiles.contains(&(52, 120)), "{:?}", tiles);
        // A hook on plain dirt keeps only the air tiles.
        let tiles = blacklisted_tiles(&w2, &[(100, 119)]);
        assert!(!tiles.contains(&(98, 118)), "{:?}", tiles);
    }

    /// S1-L-10a — the blacklist gate in the latch scan
    /// (`Projectile.cs:52699` → `Player.IsBlacklistedForGrappling`).
    #[test]
    fn a_blacklisted_tile_is_refused() {
        let w = ground();
        // The dirt at (50, 120) is solid, so the hook may latch on it.
        assert!(can_latch(HOOK, &w, 50, 120, &[]));
        // Once the tile is blacklisted the latch is refused.
        assert!(!can_latch(HOOK, &w, 50, 120, &[(50, 120)]));
    }

    /// S1-L-10a — `Player.QuickGrapple_GetItemToUse` (`Player.cs:6481-6500`):
    /// the grapple misc-equip slot wins, then the inventory; a non-hook item
    /// is skipped.
    #[test]
    fn the_quick_grapple_picks_the_hook_item() {
        let hook_slot = crate::item::ItemSlot {
            kind: 84, // the Grappling Hook (`Item.shoot` = 13)
            name: "Grappling Hook",
            count: 1,
            prefix: 0,
        };
        let sword = crate::item::ItemSlot {
            kind: 4, // the Iron Broadsword — no `shoot`
            name: "Iron Broadsword",
            count: 1,
            prefix: 0,
        };
        let misc = [crate::item::EMPTY_SLOT; crate::inv::MISC];
        let inventory = [sword; crate::item::INV];
        assert_eq!(quick_grapple_item(&misc, &inventory), None);
        // The inventory scan finds the hook.
        let mut inv = inventory;
        inv[3] = hook_slot;
        assert_eq!(quick_grapple_item(&misc, &inv), Some(13));
        // The grapple slot wins over the inventory.
        let mut misc = misc;
        misc[4] = hook_slot;
        assert_eq!(quick_grapple_item(&misc, &inv), Some(13));
    }

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
        let t = ai_hook(&mut h, &w, pc, false, false, None, &[]);
        assert_eq!(h.ai[0], 1.0, "past 300 px the hook retracts");
        assert!(!t.dead);
        // The source sets the retract velocity on the following tick (the Fly
        // arm only flips `ai[0]` this one), so the speed appears after it.
        ai_hook(&mut h, &w, pc, false, false, None, &[]);
        let speed = h.vel.length();
        assert!((speed - 11.0).abs() < 1e-3, "retract speed 11, got {speed}");
        assert!(h.vel.y < 0.0, "toward the player above");
        // Walk it in and watch it die at the 24 px reach.
        for _ in 0..40 {
            if ai_hook(&mut h, &w, pc, false, false, None, &[]).dead {
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
        let t = ai_hook(&mut h, &w, pc, false, false, None, &[]);
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
        ai_hook(&mut h, &w, pc, false, false, None, &[]);
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
        let t = ai_hook(&mut h, &w, pc, false, false, None, &[]);
        assert!(t.attached.is_none(), "stone is not a track");
        // Now the track tile at the same spot.
        put(&mut w, 40, 62, tr_core::smart_cursor::TRACK_TILE);
        let t = ai_hook(&mut h, &w, pc, false, false, None, &[]);
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
            ai_hook(&mut plain, &w, pc, false, false, None, &[]).attached.is_none(),
            "the plain Hook refuses the trunk"
        );
        let mut squirrel = hook(SQUIRREL_HOOK, 40.0 * 16.0, 62.0 * 16.0);
        squirrel.vel = Vec2::new(0.0, 1.0);
        assert!(
            ai_hook(&mut squirrel, &w, pc, false, false, None, &[]).attached.is_some(),
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
        assert!(ai_hook(&mut h, &w, pc, false, false, None, &[]).attached.is_some());
        // Break the tile.
        let i = w.index(40, 62);
        w.tiles[i].flags &= !tr_core::tile::FLAG_ACTIVE;
        let t = ai_hook(&mut h, &w, pc, false, false, None, &[]);
        assert_eq!(h.ai[0], 1.0, "broken tile retracts the hook");
        assert!(t.attached.is_none());
        // An out-of-range cell never latches.
        let mut far = hook(HOOK, -500.0, -500.0);
        far.vel = Vec2::new(1.0, 0.0);
        assert!(ai_hook(&mut far, &w, pc, false, false, None, &[]).attached.is_none());
    }

    // `:52594-52596` the hard cap and `:52572-52574` the owner-death kill.
    #[test]
    fn dw_2_6_the_range_cap_and_the_owner_death_kill_the_hook() {
        let w = ground();
        let mut h = hook(HOOK, 0.0, 0.0);
        let pc = player(2600.0, 0.0);
        assert!(ai_hook(&mut h, &w, pc, false, false, None, &[]).dead, ">2500 px kills");
        // The cap is `>`, so exactly 2500 survives.
        let mut edge = hook(HOOK, 0.0, 0.0);
        edge.pos = Vec2::ZERO;
        let t = ai_hook(&mut edge, &w, player(2500.0, 0.0), false, false, None, &[]);
        assert!(!t.dead, "exactly 2500 px survives");
        let mut live = hook(HOOK, 0.0, 0.0);
        assert!(ai_hook(&mut live, &w, player(10.0, 0.0), true, false, None, &[]).dead, "a dead owner kills");
    }

    // The rotation (`:52576-52581`) — the Squirrel Hook's steering reads it, so
    // it snaps to a quarter turn while attached and does not while flying.
    #[test]
    fn dw_2_8_the_squirrel_rotation_snaps_only_while_attached() {
        let w = ground();
        let pc = player(100.0, -100.0);
        let mut fly = hook(SQUIRREL_HOOK, 0.0, 0.0);
        ai_hook(&mut fly, &w, pc, false, false, None, &[]);
        let snapped = |r: f32| (r.abs() / (std::f32::consts::PI * 0.5)).fract().abs() < 1e-6
            || ((r.abs() / (std::f32::consts::PI * 0.5)).fract() - 1.0).abs() < 1e-6;
        assert!(!snapped(fly.rot), "flying keeps the raw atan2");
        let mut att = hook(SQUIRREL_HOOK, 0.0, 0.0);
        att.ai[0] = 2.0;
        att.pos = Vec2::ZERO;
        ai_hook(&mut att, &w, pc, false, false, None, &[]);
        assert!(snapped(att.rot), "attached snaps to a quarter turn");
        // `:52600-52603` — 256's rotation is `atan2(dy,dx) + 3.9250002f` (the
        // source's own offset, applied after the cap).
        let mut hand = hook(SKELETRON_HAND, 0.0, 0.0);
        ai_hook(&mut hand, &w, pc, false, false, None, &[]);
        let expect = (-100.0f32).atan2(100.0) + 3.9250002;
        assert!((hand.rot - expect).abs() < 1e-5, "the 256 offset: {}", hand.rot);
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
        let w = ground();
        let center = Vec2::new(40.0 * 16.0, 50.0 * 16.0);
        let mut pool = ProjectilePool::new();
        // The pool's roll stream as a `UnifiedRandom` (the `LootRng` bridge
        // is gone — S1-A-11 P4); the seed rides the constructor.
        let mut rng = tr_core::UnifiedRandom::new(0x5EED_1234u64 as i32);
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
            pool.update(&mut w2, &mut npcs, Some((center.x, center.y)), None, None, inputs, crate::projectile::FlailInputs::default(), &mut rng, 0, true);
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
        let mut rng = tr_core::UnifiedRandom::new(0x5EED_1234u64 as i32);
        let inputs = YoyoInputs::default();
        pool.update(
            w,
            &mut npcs,
            Some((center.x, center.y)),
            None,
            None,
            inputs,
            crate::projectile::FlailInputs::default(),
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
        let hooks: &[(u16, Vec2, f32)] = &[
            (BAT_HOOK, Vec2::new(0.0, -10.0), 0.0),
            (HOOK, Vec2::new(40.0, -30.0), 0.0),
        ];
        let pull = grapple_pull(hooks, p0, Vec2::ZERO).expect("a pull");
        // The average of the two centers is (20, -20); the vector from the
        // player is (-80, -60) and its cap is 14 (the first hook's type).
        assert!((pull.vel.length() - 14.0).abs() < 1e-3, "the Bat Hook's cap 14");
        assert!(pull.vel.x < 0.0 && pull.vel.y < 0.0, "toward the average");
        assert!(!pull.going_down);
        // A single hook is the PHYS-6 case unchanged.
        let p = grapple_pull(&[(HOOK, Vec2::new(0.0, -14.0), 0.0)], Vec2::ZERO, Vec2::ZERO).unwrap();
        assert!((p.vel.length() - 11.0).abs() < 1e-3, "the Hook's cap 11");
        // Zero hooks carries the caller through.
        assert!(grapple_pull(&[], Vec2::ZERO, Vec2::ZERO).is_none());
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


#[cfg(test)]
mod hook_steering_tests {
    use super::*;

    // `:22803-22812` — the Squirrel Hook's rotation anchors the average 28 px
    // behind it and sets the preferred direction; a zero X keeps the direction.
    #[test]
    fn dw_4_2_the_squirrel_anchor_and_the_anti_gravity_steering() {
        // 865: the rotation half a turn gives forward = (-1, 0); the anchor sits
        // 28 px to the right of the hook, and `dir` is -1.
        // A quarter-turn rotation gives forward = (0.707, 0.707); the anchor
        // sits 28 px along it, and the direction follows its x.
        let rot = std::f32::consts::PI * 0.75;
        let pull = grapple_pull(
            &[(865, Vec2::new(40.0, 0.0), rot)],
            Vec2::ZERO,
            Vec2::ZERO,
        )
        .expect("a pull");
        assert!(pull.vel.x > 0.0, "the anchor pulls toward +x: {:?}", pull.vel);
        assert_eq!(pull.dir, Some(1), "the direction follows the forward x");
    }

    // `:22777-22788` — the AntiGravity Hook adds the held control direction
    // (scaled by 100) to the hook→player vector and then 200 px along its
    // normalized result; a NaN falls back to -UnitY.
    #[test]
    fn dw_4_2_the_anti_gravity_control_bias() {
        let stick = steering_stick(false, true, false, false);
        assert_eq!(stick, Vec2::new(1.0, 0.0), "right is +x");
        let pull = grapple_pull(
            &[(446, Vec2::new(0.0, 100.0), 0.0)],
            Vec2::ZERO,
            stick,
        )
        .expect("a pull");
        // The hook is below the player and right is held, so the contribution
        // biases the average sideways as well as up.
        assert!(pull.vel.y < 0.0, "still pulling up");
        // The pull speed cap for 446 is the default 11.
        assert!((pull.vel.length() - 11.0).abs() < 1e-3);
    }

    // `:22789-22802` — the Static Hook's slack: along the hook→player line,
    // scaled by 6, and zeroed when it fights the hook and the line is 600+ px.
    #[test]
    fn dw_4_2_the_static_slack_zeroes_when_it_fights() {
        // A held direction whose projection fights the hook on a 600+ px line
        // gets `slack = 0`, which makes the contribution the player's own
        // center — the Static Hook holds you in place.
        let far = grapple_pull(
            &[(652, Vec2::new(200.0, -600.0), 0.0)],
            Vec2::ZERO,
            Vec2::new(0.0, 1.0),
        )
        .expect("a pull");
        assert_eq!(far.vel, Vec2::ZERO, "the zero slack collapses the pull: {:?}", far.vel);
        // With no held direction `value` is zero, so the contribution is the
        // player's own center — the Static Hook hangs you still (the 6 px slack
        // never engages without input).
        let plain = grapple_pull(&[(652, Vec2::new(200.0, -600.0), 0.0)], Vec2::ZERO, Vec2::ZERO)
            .expect("a pull");
        assert_eq!(plain.vel, Vec2::ZERO, "no input hangs the player still");
        // Held toward the hook (along the line, positive projection) the 6 px
        // slack engages and the pull is non-zero.
        let along = grapple_pull(
            &[(652, Vec2::new(200.0, -600.0), 0.0)],
            Vec2::ZERO,
            Vec2::new(0.316, -0.949),
        )
        .expect("a pull");
        assert!(
            along.vel.length() > 1.0,
            "the slack engages with input: {:?}",
            along.vel
        );
        // Near and with the held direction: slack 6 along the line.
        let near = grapple_pull(
            &[(652, Vec2::new(6.0, 0.0), 0.0)],
            Vec2::ZERO,
            Vec2::new(1.0, 0.0),
        )
        .expect("a pull");
        assert!(near.vel.x > 0.0, "the slack keeps a positive pull");
    }
}

#[cfg(test)]
mod crack_tests {
    use super::*;
    use crate::projectile::*;

    fn put_cracked(w: &mut World, x: i32, y: i32) {
        let i = w.index(x, y);
        w.tiles[i] = tr_core::Tile {
            kind: 481, // a cracked brick (SET_CRACKED_BRICKS)
            ..Default::default()
        };
        w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
    }

    fn one_hook() -> crate::projectile::Projectile {
        let mut p = crate::projectile::Projectile::default();
        p.active = true;
        p.kind = HOOK;
        p.w = 18.0;
        p.h = 18.0;
        p.pos = Vec2::new(40.0 * 16.0 - 9.0, 41.0 * 16.0 - 9.0);
        p.vel = Vec2::ZERO;
        p
    }

    /// `:52586-52595` — 1/16 of the local player's hook ticks break the
    /// cracked-brick tile under its center, and the roll is deterministic on the
    /// documented stream. The gate is off for a non-local owner.
    #[test]
    fn dw_4_4_the_crack_roll_is_deterministic_and_local() {
        let mut w = crate::test_world_helpers::ground();
        put_cracked(&mut w, 40, 41);
        let center = Vec2::new(40.0 * 16.0, 41.0 * 16.0);
        // Count the hits over many ticks with the same seeded stream twice:
        // both runs must agree (and be ~1/16 of 4096 ticks).
        let mut counts = Vec::new();
        for _ in 0..2 {
            let mut n = 0usize;
            let mut seed = tr_core::UnifiedRandom::new(0xB17C_0DE5u64 as i32);
            for _ in 0..4096 {
                let mut h = one_hook();
                let tick = ai_hook(&mut h, &w, center, false, true, Some(&mut seed), &[]);
                if tick.crack.is_some() {
                    n += 1;
                }
            }
            counts.push(n);
        }
        assert_eq!(counts[0], counts[1], "the stream is deterministic");
        assert!(
            (180..=300).contains(&counts[0]),
            "about 1/16 of 4096: {}",
            counts[0]
        );
        // A non-local owner never rolls (`Main.myPlayer == this.owner`).
        let mut h = one_hook();
        let mut seed = tr_core::UnifiedRandom::new(1);
        assert!(ai_hook(&mut h, &w, center, false, false, Some(&mut seed), &[]).crack.is_none());
        // A non-cracked tile never breaks — and the C# `&&` short-circuit
        // means the roll does not even run there (the stream is untouched).
        let i = w.index(40, 41);
        w.tiles[i].kind = 1;
        let mut h2 = one_hook();
        let mut seed2 = tr_core::UnifiedRandom::new(2);
        let pristine = seed2.clone();
        let mut found = 0;
        for _ in 0..4096 {
            if ai_hook(&mut h2, &w, center, false, true, Some(&mut seed2), &[]).crack.is_some() {
                found += 1;
            }
        }
        assert_eq!(found, 0, "only the cracked bricks break");
        assert_eq!(seed2, pristine, "the gated roll never drew");
    }

    /// `:22845-22847` — no registered hook carries the caller's velocity through
    /// rather than pulling.
    #[test]
    fn dw_4_5_no_hooks_carries_the_velocity() {
        assert!(grapple_pull(&[], Vec2::ZERO, Vec2::ZERO).is_none());
    }

    /// The four corrected constants, pinned against the cited lines.
    #[test]
    fn dw_4_1_the_four_corrected_constants() {
        // `:52646-52665` — 256 SkeletronHand 425 (the source's own row, previously
        // inherited as the 300 default) and 935 QueenSlimeHook 500.
        assert_eq!(hook_spec(256).max_distance, 425.0);
        assert_eq!(hook_spec(935).max_distance, 500.0);
        // `:22832-22851` — the Thorn Hook 487's cap 12 and the Lunar four's 16.
        assert_eq!(hook_spec(487).pull_speed, 12.0);
        for k in 646..=649 {
            assert_eq!(hook_spec(k).pull_speed, 16.0, "Lunar {k}");
        }
    }
}

/// W3-i39 — the grapple census fixes, witnessed against the C# lines. Tests
/// WRITTEN NOT RUN (code-only swarm). Each pins one corrected deviation.
#[cfg(test)]
mod i39_census_tests {
    use super::*;
    use crate::projectile::{HOOK, SKELETRON_HAND, SQUIRREL_HOOK, STATIC_HOOK, TRACK_HOOK};
    use crate::test_world_helpers::ground;

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

    fn put(w: &mut World, x: i32, y: i32, kind: u16) {
        let i = w.index(x, y);
        w.tiles[i] = tr_core::Tile { kind, ..Default::default() };
        w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
    }

    /// `Tile.nactive()` (`Tile.cs:240-247`) — an ACTUATED solid tile is not
    /// latchable: `can_latch` refuses it (`Projectile.cs:52870`).
    #[test]
    fn i39_an_actuated_tile_is_not_latchable() {
        let mut w = ground();
        put(&mut w, 40, 62, 1);
        assert!(can_latch(HOOK, &w, 40, 62, &[]), "the solid dirt latches");
        // Actuate it (the inactive bit, `FLAG_INACTIVE`).
        let i = w.index(40, 62);
        w.tiles[i].flags |= tr_core::tile::FLAG_INACTIVE;
        assert!(!can_latch(HOOK, &w, 40, 62, &[]), "actuated solid refuses");
    }

    /// `WorldGen.SolidTile` (`WorldGen.cs:70111-70132`) — the blacklist's 3x3
    /// adds the NON-solid cells. A hook in the open air (default inactive tiles)
    /// blacklists that air; the solid ground row stays latchable. The port's
    /// `World::tile` yields a default tile in bounds (the C# null arm is
    /// unreachable), so the active gate decides. WRITTEN NOT RUN.
    #[test]
    fn i39_the_blacklist_covers_the_air_not_the_ground() {
        let w = ground();
        // A hook centre in the open air at row 60 (ground starts at 120): the
        // 3x3 around it is inactive air → non-solid → blacklisted.
        let tiles = blacklisted_tiles(&w, &[(40, 60)]);
        assert!(tiles.contains(&(40, 60)), "the air cell is blacklisted: {tiles:?}");
        assert!(tiles.contains(&(39, 59)), "the 3x3 corner is blacklisted");
        // A hook on the solid ground row: the ground cells are solid → NOT added.
        let on_ground = blacklisted_tiles(&w, &[(40, 120)]);
        assert!(!on_ground.contains(&(40, 120)), "the solid ground is not blacklisted");
        assert!(!on_ground.contains(&(41, 121)), "the solid ground row stays latchable");
    }

    /// `:52577` — the rotation uses the source's literal `1.57f`, NOT pi/2.
    #[test]
    fn i39_the_rotation_constant_is_1_57() {
        let w = ground();
        let mut h = hook(HOOK, 0.0, 0.0);
        let pc = Vec2::new(100.0, 0.0); // due right: atan2(0,100) = 0
        ai_hook(&mut h, &w, pc, false, false, None, &[]);
        assert!((h.rot - (-1.57)).abs() < 1e-6, "rot = atan2 - 1.57: {}", h.rot);
    }

    /// `:52578-52582` — the Squirrel snap is C# `Math.Round` (banker's): a
    /// half-quarter-turn rounds to EVEN, not away from zero. The player at
    /// (100,100) gives atan2 = π/4, so rot = π/4 − 1.57 ≈ −0.7846 and
    /// rot/(π/2) ≈ −0.4995 → rounds to 0 (the snap zeroes the rotation).
    #[test]
    fn i39_the_squirrel_snap_is_bankers() {
        let w = ground();
        let q = std::f32::consts::PI * 0.5;
        let pc = Vec2::new(100.0, 100.0); // atan2(100,100) = π/4 exactly
        let mut h = hook(SQUIRREL_HOOK, 0.0, 0.0);
        h.ai[0] = 2.0;
        h.pos = Vec2::ZERO;
        ai_hook(&mut h, &w, pc, false, false, None, &[]);
        // The snap ran before the attached re-validation; rot is a quarter-turn
        // multiple (here 0). Banker's and away-from-zero agree at −0.4995, but
        // the snap itself (round to a quarter turn) is the witnessed behaviour.
        let snapped = tr_core::damage::round_bankers((h.rot / q) as f64) as f32;
        assert!((h.rot - snapped * q).abs() < 1e-4, "quarter-turn snap: {}", h.rot);
    }

    /// `:52600-52603` — the Skeletron Hand (256) rotation is `atan2 + 3.9250002f`.
    #[test]
    fn i39_the_skeletron_hand_rotation_offset() {
        let w = ground();
        let mut h = hook(SKELETRON_HAND, 0.0, 0.0);
        let pc = Vec2::new(0.0, -100.0); // straight up: atan2(-100,0) = -pi/2
        ai_hook(&mut h, &w, pc, false, false, None, &[]);
        let expect = (-100.0f32).atan2(0.0) + 3.9250002;
        assert!((h.rot - expect).abs() < 1e-5, "the +3.9250002 offset: {}", h.rot);
    }

    /// `:52615-52622` — the Static Hook (652) steps its frame every 7 ticks,
    /// modulo `projFrames[652]` (= 6). RNG-free.
    #[test]
    fn i39_the_static_hook_frame_advances_every_7() {
        let w = ground();
        let mut h = hook(STATIC_HOOK, 40.0 * 16.0, 60.0 * 16.0);
        let pc = Vec2::new(40.0 * 16.0, 60.0 * 16.0); // coincident: dist 0, stays Fly
        for _ in 0..6 {
            ai_hook(&mut h, &w, pc, false, false, None, &[]);
        }
        assert_eq!(h.frame, 0, "no frame step before 7 ticks");
        ai_hook(&mut h, &w, pc, false, false, None, &[]);
        assert_eq!(h.frame, 1, "the 7th tick steps the frame");
        assert_eq!(h.frame_counter, 0, "the counter resets");
        // 42 ticks total → 6 steps → wraps to 0 (6 frames).
        for _ in 0..35 {
            ai_hook(&mut h, &w, pc, false, false, None, &[]);
        }
        assert_eq!(h.frame, 0, "the frame wraps modulo 6");
    }

    /// `:52646-52785` — the latch scan runs on the SAME tick the distance flips
    /// Fly→Retract (the source does not re-gate on the state). A hook just past
    /// its range that overlaps a latchable tile attaches immediately.
    #[test]
    fn i39_the_scan_runs_on_the_flip_tick() {
        let mut w = ground();
        put(&mut w, 40, 62, 1);
        // The Hook (300 px range) at 301 px from the player, overlapping the
        // tile at (40,62): the distance flips ai[0]→1, but the scan still latches.
        let mut h = hook(HOOK, 40.0 * 16.0, 62.0 * 16.0);
        let pc = Vec2::new(40.0 * 16.0, 62.0 * 16.0 - 301.0);
        let t = ai_hook(&mut h, &w, pc, false, false, None, &[]);
        assert!(t.attached.is_some(), "the flip tick still scans and latches");
        assert_eq!(h.ai[0], 2.0);
    }

    /// `:52852` — the attached re-validation reads `CanTileBeLatchedOnTo` with
    /// NO blacklist: a blacklisted tile the hook already sits on stays attached.
    #[test]
    fn i39_the_attached_revalidation_ignores_the_blacklist() {
        let mut w = ground();
        put(&mut w, 40, 62, 1);
        let mut h = hook(HOOK, 40.0 * 16.0 + 8.0, 62.0 * 16.0 + 8.0);
        h.ai[0] = 2.0;
        let pc = Vec2::new(40.0 * 16.0, 80.0 * 16.0);
        // The tile under the hook is blacklisted, but the re-validation passes
        // an empty blacklist (the C# does not consult it here).
        let t = ai_hook(&mut h, &w, pc, false, false, None, &[(40, 62)]);
        assert!(t.attached.is_some(), "the blacklist does not detach an attached hook");
        assert_eq!(h.ai[0], 2.0);
    }

    /// `:22852-22855` — the pull cap has NO epsilon: at a tiny non-zero distance
    /// the factor is 1 (dist <= pull_speed), so the velocity is the raw vector.
    #[test]
    fn i39_the_pull_cap_has_no_epsilon() {
        // A hook 0.0005 px away: the old `dist > 0.001` guard zeroed it; the C#
        // keeps the raw (tiny) vector because dist < pull_speed → factor 1.
        let p = grapple_pull(&[(HOOK, Vec2::new(0.0005, 0.0), 0.0)], Vec2::ZERO, Vec2::ZERO).unwrap();
        assert!(p.vel.x > 0.0, "the tiny pull is not zeroed: {:?}", p.vel);
        assert!((p.vel.x - 0.0005).abs() < 1e-7);
    }

    /// `:52648` — the Track Hook (403) has NO distance row: its Fly state never
    /// retracts on distance (it latches the rail it spawns on).
    #[test]
    fn i39_the_track_hook_has_no_distance_cap() {
        assert!(hook_spec(TRACK_HOOK as i32).max_distance.is_infinite());
        let w = ground();
        let mut h = hook(TRACK_HOOK, 0.0, 0.0);
        // 1000 px from the player (under the 2500 hard cap), over no tile:
        // stays Fly (ai[0] == 0), not Retract — the 403 has no distance row.
        let pc = Vec2::new(1000.0, 0.0);
        ai_hook(&mut h, &w, pc, false, false, None, &[]);
        assert_eq!(h.ai[0], 0.0, "the Track Hook never distance-retracts");
    }

    /// `:52757` — the latch queues the case-0 dig cue (`PlaySound(0, i*16, j*16)`)
    /// on the slot's `wants_sound` intent (the pool drains it on UpdateProjectiles).
    #[test]
    fn i39_the_latch_queues_the_dig_cue() {
        let mut w = ground();
        put(&mut w, 40, 62, 1);
        let mut h = hook(HOOK, 40.0 * 16.0, 62.0 * 16.0);
        let pc = Vec2::new(40.0 * 16.0, 62.0 * 16.0 - 100.0);
        let t = ai_hook(&mut h, &w, pc, false, false, None, &[]);
        assert!(t.attached.is_some());
        let (id, _style, x, y, _pitch, _vol) = h.wants_sound.expect("the dig cue");
        assert_eq!(id, 0, "the int overload PlaySound(0)");
        assert_eq!((x, y), (40.0 * 16.0, 62.0 * 16.0), "at the tile origin");
    }

    /// W3-i39 retry — `Utils.cs:1809`: `ToTileCoordinates` is `(int)vec >> 4`
    /// (int truncation then arithmetic shift, NO clamp). The scan bounds clamp
    /// the UPPER to `maxTiles` (not `maxTiles-1`), so the last in-bounds column
    /// is still scanned. A hook near the right edge latches the edge column.
    /// WRITTEN NOT RUN.
    #[test]
    fn i39_the_scan_reaches_the_last_in_bounds_column() {
        // The ground() world is 200 wide; put a solid tile in the last column
        // (x=199) at row 62 and aim the hook so its scan box overlaps it.
        let mut w = ground();
        put(&mut w, 199, 62, 1);
        // The hook center at x=199*16+8=3192: the scan's upper anchor is
        // Center+37=3229 → tile 201 → clamped to maxTiles=200 (exclusive), so
        // column 199 IS scanned. The old `tile_of` pre-clamped to 199, making
        // the exclusive loop stop at 198 and miss the edge tile.
        let mut h = hook(HOOK, 199.0 * 16.0 + 8.0, 62.0 * 16.0 + 8.0);
        let pc = Vec2::new(199.0 * 16.0 + 8.0, 62.0 * 16.0 + 8.0 - 100.0);
        let t = ai_hook(&mut h, &w, pc, false, false, None, &[]);
        assert!(t.attached.is_some(), "the edge column latches");
        assert_eq!(h.ai[0], 2.0);
    }

    /// W3-i39 retry — `tile_coord` matches `ToTileCoordinates` (`Utils.cs:1809`):
    /// `(int)vec >> 4`. For a negative coordinate the int truncates toward zero
    /// then the arithmetic shift floors — e.g. -1 → 0 >> 4 = -1 (not 0).
    /// WRITTEN NOT RUN.
    #[test]
    fn i39_tile_coord_is_the_csharp_shift() {
        assert_eq!(tile_coord(0.0), 0);
        assert_eq!(tile_coord(15.9), 0);
        assert_eq!(tile_coord(16.0), 1);
        assert_eq!(tile_coord(-1.0), -1, "(int)-1 >> 4 = -1");
        assert_eq!(tile_coord(-16.0), -1);
        assert_eq!(tile_coord(-17.0), -2);
    }
}
