//! The Wire Kite's placement machine (S1-W4-g81 residuals) — aiStyle 125,
//! the `else if (this.aiStyle == 125)` arm of `Projectile.AI()`
//! (`Projectile.cs:35697-35833`) plus `Kill()`'s type-651 mass-wire arm
//! (`:72051-72067`). The ghost DRAW is `render/wire_kite.rs`
//! (`Main.cs:31620-31907`).
//!
//! The arm is owner-local (`Main.myPlayer == this.owner`, `:35700`): the
//! kite rides the cursor while the channel holds, latches the anchor tile
//! on the first tick (`ai[0]/ai[1]`, `:35737-35743`), and on the channel
//! release sets `localAI[0] = 1` then kills itself (`:35716-35723`) — the
//! Kill arm fires `Wiring.MassWireOperation(ps, pe)` from the anchor to
//! the release cell. The two L-walk scans (`:35746-35823`) read tiles and
//! do NOTHING with them (`Tile tile4 = ...;` — dead reads in the C#);
//! they are not ported (no observable effect).
//!
//! Deviations (ANALYSIS "The Wire Kite ghost (S1-W4-g81)"):
//! * `Wiring.MassWireOperation` is unported ON THIS BRANCH - the kill
//!   intent rides out to the world sim (the `wants_mark_spawn` pattern)
//!   and is DROPPED there with a documented note. The compose verdict
//!   (research/283): c42 owns the apply (`world_sim/wiring.rs::
//!   mass_wire_operation`, fold-133), f11 threads the `drop_rng` param,
//!   d13 owns the pool seam - the merger replaces this lane's drop with
//!   the published direct-drain hunk (f11 research/133 §8.4b);
//! * the right-click kill's `player.mouseInterface = true` /
//!   `Main.blockMouse = true` writes (`:35713-35714`) ride the tick
//!   struct — `blockMouse` has a port field, `mouseInterface` folds into
//!   the slot-surface stand-in (no write needed: the next frame's
//!   `over_ui_slot` re-derives);
//! * `player.gravDir == -1` (`:35727-35730`) folds out — the port has no
//!   gravity flip (the smart-cursor precedent).

use glam::Vec2;

use crate::projectile::Projectile;

/// The per-tick owner snapshot (`Main.player[owner]` reads,
/// `Projectile.cs:35699-35745`) — the pool cannot borrow the player
/// mid-slot (the `GolfClubPlayer` pattern).
#[derive(Clone, Copy, Debug)]
pub(crate) struct KiteOwner {
    /// `player.noItems || player.CCed || player.dead` (`:35706`).
    pub refuse: bool,
    /// `player.channel` (`:35716`).
    pub channel: bool,
    /// `player.direction` (`:35754`) — the L-walk's axis pick; the walks
    /// are dead reads, so only the tail's `ChangeDir` uses it.
    pub direction: i32,
    /// `player.velocity.X`'s sign (`:35825-35829`).
    pub vel_x: f32,
}

/// The arm's queued writes, drained by the pool's dispatch (the
/// `GolfClubTick` pattern).
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct KiteTick {
    /// `player.ChangeDir(Math.Sign(velocity.X))` when non-zero
    /// (`:35825-35829`).
    pub facing: Option<i32>,
    /// `player.heldProj = whoAmI` (`:35830`).
    pub held_proj: bool,
    /// `player.SetDummyItemTime(2)` (`:35831`) — the tick count.
    pub dummy_item_time: Option<i32>,
    /// `player.itemRotation = 0f` (`:35832`).
    pub zero_item_rotation: bool,
    /// `Main.blockMouse = true` (the right-click kill, `:35714`).
    pub block_mouse: bool,
}

/// `Projectile.AI()`'s aiStyle-125 arm (`Projectile.cs:35697-35833`) for
/// one slot. `mouse_world` is `Main.screenPosition + (mouseX, mouseY)`
/// (`:35726`); `right_press` is `Main.mouseRight && Main.mouseRightRelease`
/// (`:35710`). The caller gates on `owner == Main.myPlayer` (`:35700`).
pub(crate) fn ai_wire_kite(
    p: &mut Projectile,
    owner: KiteOwner,
    mouse_world: Vec2,
    right_press: bool,
) -> KiteTick {
    let mut tick = KiteTick::default();
    // `:35702-35705` — the one-tick move cooldown.
    if p.local_ai[1] > 0.0 {
        p.local_ai[1] -= 1.0;
    }
    if owner.refuse {
        // `:35706-35709` — noItems/CCed/dead: plain Kill (the death hook
        // owns the channel cancel + the mass-op gate; localAI[0] stays 0
        // so no operation fires).
        kill(p);
    } else if right_press {
        // `:35710-35715` — the right-click cancel: Kill +
        // mouseInterface + blockMouse.
        kill(p);
        tick.block_mouse = true;
    } else if !owner.channel {
        // `:35716-35723` — the channel release: arm the mass-op flag
        // THEN Kill — the Kill arm reads localAI[0] == 1.
        if p.local_ai[0] == 0.0 {
            p.local_ai[0] = 1.0;
        }
        kill(p);
    } else if p.local_ai[1] == 0.0 {
        // `:35726-35736` — ride the cursor: Center = mouse world point
        // (the port's box is top-left, so subtract the half-box), one
        // tick of move cooldown per actual move.
        let center = Vec2::new(p.pos.x + p.w * 0.5, p.pos.y + p.h * 0.5);
        if mouse_world != center {
            p.net_update = true;
            p.pos = mouse_world - Vec2::new(p.w * 0.5, p.h * 0.5);
            p.local_ai[1] = 1.0;
        }
        // `:35737-35743` — the first tick latches the anchor tile:
        // `(int)Center.X / 16` — the C# casts BEFORE the divide (the
        // negative-coordinate truncation differs from a floor-divide;
        // verbatim).
        if p.ai[0] == 0.0 && p.ai[1] == 0.0 {
            let cx = p.pos.x + p.w * 0.5;
            let cy = p.pos.y + p.h * 0.5;
            p.ai[0] = (cx as i32 / 16) as f32;
            p.ai[1] = (cy as i32 / 16) as f32;
            p.net_update = true;
            p.vel = Vec2::ZERO;
        }
    }
    // `:35745` - the velocity zero runs on EVERY local tick (outside the
    // cooldown gate).
    p.vel = Vec2::ZERO;
    // `:35746-35823` - the two L-walk tile scans are dead reads in the
    // C# (module doc); not ported.
    // `:35825-35832` - the owner writes. The C# has NO early return after
    // the kill arms - the tail runs on the kill tick too (the d13 census).
    // `Math.Sign(float)` is the float sign (-1/0/+1 on the RAW value, NOT
    // the int-truncated one - a 0.5 velocity still flips the facing).
    if owner.vel_x != 0.0 {
        tick.facing = Some(if owner.vel_x > 0.0 { 1 } else { -1 });
    }
    tick.held_proj = true;
    tick.dummy_item_time = Some(2);
    tick.zero_item_rotation = true;
    tick
}

/// `Projectile.Kill`'s shared head for the kite (`:70644-70650`): the
/// `!active` guard, `kill_time_left` latch, `timeLeft = 0`, deactivate.
/// The channel-cancel + the type-651 mass-op arm ride the pool's death
/// hook (the port's once-per-death seam).
fn kill(p: &mut Projectile) {
    if !p.active {
        return;
    }
    p.kill_time_left = p.time_left;
    p.time_left = 0;
    p.active = false;
}

/// `Kill()`'s type-651 arm (`Projectile.cs:72051-72067`): with
/// `localAI[0] == 1` and a local owner, `MassWireOperation` runs from the
/// anchor `(ai[0], ai[1])` to the release cell (`Center.ToTileCoordinates`
/// — `(int)X >> 4`, `Utils.cs:1807-1810`: the C# casts to int FIRST, then
/// arithmetic-shifts; for positives that equals the truncating divide,
/// for negatives it floors — the d13 pool shape is the same form).
/// Returns the `(ps, pe)` pair; the world sim drops it with the
/// documented mass-op note.
pub(crate) fn kill_mass_wire_op(p: &Projectile) -> Option<((i32, i32), (i32, i32))> {
    if p.kind != crate::projectile::WIRE_KITE || p.local_ai[0] != 1.0 || p.owner != 0 {
        return None;
    }
    let ps = (p.ai[0] as i32, p.ai[1] as i32);
    let cx = p.pos.x + p.w * 0.5;
    let cy = p.pos.y + p.h * 0.5;
    let pe = ((cx as i32) >> 4, (cy as i32) >> 4);
    Some((ps, pe))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::projectile::WIRE_KITE;

    fn kite(x: f32, y: f32) -> Projectile {
        Projectile {
            active: true,
            kind: WIRE_KITE,
            owner: 0,
            pos: Vec2::new(x, y),
            w: 10.0,
            h: 10.0,
            time_left: 3600,
            ..Default::default()
        }
    }

    fn owner(channel: bool) -> KiteOwner {
        KiteOwner {
            refuse: false,
            channel,
            direction: 1,
            vel_x: 0.0,
        }
    }

    #[test]
    fn the_first_channel_tick_latches_the_anchor_tile() {
        // `Projectile.cs:35737-35743`: ai0/ai1 = (int)Center/16.
        let mut p = kite(160.0 - 5.0, 320.0 - 5.0); // centre (160, 320)
        let tick = ai_wire_kite(&mut p, owner(true), Vec2::new(160.0, 320.0), false);
        assert_eq!(p.ai[0], 10.0);
        assert_eq!(p.ai[1], 20.0);
        assert!(p.net_update);
        assert_eq!(p.vel, Vec2::ZERO);
        assert!(tick.held_proj);
        assert_eq!(tick.dummy_item_time, Some(2));
        assert!(tick.zero_item_rotation);
    }

    #[test]
    fn the_kite_rides_the_cursor_with_a_one_tick_cooldown() {
        // `:35724-35736`: the move sets localAI[1] = 1; the next tick
        // decrements it to 0 and does NOT move; the tick after moves.
        let mut p = kite(155.0, 315.0);
        p.ai = [10.0, 20.0, 0.0, 0.0]; // anchor already latched
        ai_wire_kite(&mut p, owner(true), Vec2::new(200.0, 320.0), false);
        assert_eq!(p.pos, Vec2::new(195.0, 315.0), "centre rides the mouse");
        assert_eq!(p.local_ai[1], 1.0);
        ai_wire_kite(&mut p, owner(true), Vec2::new(240.0, 320.0), false);
        assert_eq!(p.pos, Vec2::new(195.0, 315.0), "the cooldown tick holds");
        assert_eq!(p.local_ai[1], 0.0);
        ai_wire_kite(&mut p, owner(true), Vec2::new(240.0, 320.0), false);
        assert_eq!(p.pos, Vec2::new(235.0, 315.0));
    }

    #[test]
    fn the_channel_release_arms_the_mass_op_then_kills() {
        // `:35716-35723` + `:72051-72067`.
        let mut p = kite(195.0, 315.0);
        p.ai = [10.0, 20.0, 0.0, 0.0];
        let tick = ai_wire_kite(&mut p, owner(false), Vec2::ZERO, false);
        assert!(!p.active);
        assert_eq!(p.local_ai[0], 1.0);
        assert_eq!(p.time_left, 0);
        // `:35825-35832` — the owner tail runs on the KILL tick too (the
        // C# has no return after the kill arms; the d13 census).
        assert!(tick.held_proj);
        assert_eq!(tick.dummy_item_time, Some(2));
        assert!(tick.zero_item_rotation);
        let op = kill_mass_wire_op(&p).expect("the release fires the op");
        assert_eq!(op.0, (10, 20));
        assert_eq!(op.1, (12, 20), "centre (200,320) → tile (12,20)");
    }

    #[test]
    fn the_release_cell_floors_negative_centres() {
        // `Utils.cs:1807-1810`: `(int)X >> 4` — the arithmetic shift
        // floors: centre (-1, -1) → tile (-1, -1), NOT the truncating
        // divide's (0, 0).
        let mut p = kite(-6.0, -6.0);
        p.local_ai[0] = 1.0;
        let op = kill_mass_wire_op(&p).expect("the latch fires");
        assert_eq!(op.1, (-1, -1), "(-1)>>4 = -1");
    }

    #[test]
    fn the_right_click_kills_without_the_mass_op() {
        // `:35710-35715`: localAI[0] stays 0 - no operation.
        let mut p = kite(195.0, 315.0);
        let tick = ai_wire_kite(&mut p, owner(true), Vec2::ZERO, true);
        assert!(!p.active);
        assert_eq!(p.local_ai[0], 0.0);
        assert!(tick.block_mouse);
        assert!(kill_mass_wire_op(&p).is_none());
        // The owner tail runs on the kill tick (`:35825-35832`).
        assert!(tick.held_proj);
        assert!(tick.zero_item_rotation);
    }

    #[test]
    fn the_cced_owner_kills_without_the_mass_op() {
        // `:35706-35709`.
        let mut p = kite(195.0, 315.0);
        let mut o = owner(true);
        o.refuse = true;
        let tick = ai_wire_kite(&mut p, o, Vec2::ZERO, false);
        assert!(!p.active);
        assert!(kill_mass_wire_op(&p).is_none());
        // The owner tail runs on the kill tick (the C# else-if chain has
        // no return; `:35825-35832` follows it).
        assert!(tick.held_proj);
        assert_eq!(tick.dummy_item_time, Some(2));
    }

    #[test]
    fn the_velocity_sign_flips_the_facing() {
        // `:35825-35829`: ChangeDir(Sign(velocity.X)) when non-zero — the
        // FLOAT sign, so a sub-1 velocity still flips.
        let mut p = kite(155.0, 315.0);
        let mut o = owner(true);
        o.vel_x = -3.0;
        let tick = ai_wire_kite(&mut p, o, Vec2::new(160.0, 320.0), false);
        assert_eq!(tick.facing, Some(-1));
        o.vel_x = 0.5;
        let tick = ai_wire_kite(&mut p, o, Vec2::new(160.0, 320.0), false);
        assert_eq!(tick.facing, Some(1), "Math.Sign(0.5f) = 1");
        o.vel_x = 0.0;
        let tick = ai_wire_kite(&mut p, o, Vec2::new(160.0, 320.0), false);
        assert_eq!(tick.facing, None, "a standing player keeps the facing");
    }

    #[test]
    fn the_anchor_cast_truncates_before_the_divide() {
        // `:35739-35740`: `(int)Center.X / 16` — 15.9 → 15 → 0, NOT
        // (int)(15.9/16) = 0 here but 31.9 → 31 → 1 vs floor(31.9/16)=1;
        // the negative arm differs: -1.0 → -1 → 0 (C# int divide rounds
        // toward zero), while (-1.0/16) as i32 = 0 too — pin the
        // positive-boundary form 15.99 → 0.
        let mut p = kite(15.99 - 5.0, 16.0 - 5.0);
        ai_wire_kite(&mut p, owner(true), Vec2::new(15.99, 16.0), false);
        assert_eq!(p.ai[0], 0.0);
        assert_eq!(p.ai[1], 1.0);
    }
}
