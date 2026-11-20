//! W4-d13 — `AI()`'s `else if (this.aiStyle == 125)` arm
//! (`Projectile.cs:35697-35833`): the Wire Kite (651, the Grand Design's
//! channel projectile — `ProjectileID.WireKite`, `ProjectileID.cs:1856`;
//! the item 3611's `projectile: Some(651)` row). The `SetDefaults` row is
//! `:6990-7000` (10x10, friendly, ignoreWater, tileCollide false,
//! penetrate -1, drawLayer 5 — a draw-side field with no pool consumer).
//!
//! **RNG census: ZERO draws.** The arm reads no `Main.rand` anywhere; the
//! two L-shaped tile scans (`:35746-35823`) are NO-OPS in the source — the
//! `flag69` break flag is seeded false and never set, and the loop bodies
//! only bind an unused `Tile tile4` read behind `WorldGen.InWorld` (the
//! decompiler kept the reads, dropped nothing observable). The port cites
//! the loops and skips them (zero state, zero draws either way).
//!
//! Deviations (the standing pool precedents, ANALYSIS): `gravDir` folds to
//! 1 (no flipped gravity in the port — the bow-launch precedent), so the
//! `:35727-35730` mouse-Y flip never fires; `player.mouseInterface = true`
//! (`:35713`) has no port field — the use-input gates fold it (the
//! S1-P-i09 `use_input_live` precedent); `Main.blockMouse = true`
//! (`:35714`) rides the pool's `take_block_mouse` drain; the
//! `SetDummyItemTime(2)` tail (`:35831`) rides `take_kite_hold` (the
//! `pending_spear_hold` pattern). `ChangeDir` (`:35828`) folds to the
//! facing write (the golf `PlayerWrites` precedent — the pulley arm and
//! the composite-arm flips are unported; the `itemRotation *= -1` half is
//! unobservable, the `:35832` zero follows it).
//!
//! The `Kill()` hook's `Wiring.MassWireOperation` (`Projectile.cs:72051-
//! 72067`, the `localAI[0] == 1` release latch this arm sets at `:35720`)
//! is the wiring system's documented deferral (`world_sim/editing/
//! wiring.rs:28-29` — the mass wire operation needs the unported `WiresUI`
//! tool modes); the death-hook site cites it.

use glam::Vec2;

use crate::player::Player;
use crate::projectile::{Projectile, csign};

/// The per-tick mouse snapshot the arm reads off `Main` (the pool cannot
/// borrow the App): `Main.mouseRight` / `Main.mouseRightRelease`
/// (`Projectile.cs:35710`). The release flag lags one frame in the C#
/// (`Main.cs:62458-62465` runs after Draw), so the pair's `true && true`
/// is the PRESS tick — the world sim feeds `place_down && !place_prev_down`
/// (the golf context's same-edge convention). The mouse WORLD point rides
/// the dispatch's `inputs.aim` (the yoyo's `Main.MouseScreen +
/// screenPosition` math, `gravDir = 1`).
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct WireKiteInputs {
    pub mouse_right: bool,
    pub mouse_right_release: bool,
}

/// What the arm hands back to the dispatch (the pool/App writes it cannot
/// make itself): the `Main.blockMouse` intent and the `SetDummyItemTime(2)`
/// hold. The facing/heldProj/itemRotation writes ride the `&mut Player`
/// link directly (the pool only simulates local-owned slots, so the link
/// IS `Main.player[owner]`).
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct WireKiteTick {
    /// `Main.blockMouse = true` (`:35714`) — the right-click release kill.
    pub block_mouse: bool,
    /// `player.SetDummyItemTime(2)` (`:35831`) — the per-tick item hold.
    pub dummy_hold: bool,
}

/// `Kill()` (`Projectile.cs:70644-70648`) — the sibling-module deactivation
/// (the `projectile_ai_spear` precedent; `Projectile::kill` is private).
fn kill(p: &mut Projectile) {
    p.kill_time_left = p.time_left;
    p.active = false;
    p.time_left = 0;
}

impl Projectile {
    /// `AI()`'s aiStyle-125 arm (`Projectile.cs:35697-35833`). `whoami` is
    /// the C# `base.whoAmI` the `heldProj` write stamps. A `None` player
    /// link holds the slot (the orphan guard, the spear precedent —
    /// unreachable in game: the kite spawns owner 0).
    pub(crate) fn ai_wire_kite(
        &mut self,
        player: Option<&mut Player>,
        aim: Option<Vec2>,
        mouse_right: bool,
        mouse_right_release: bool,
        local_owner: u8,
        whoami: usize,
    ) -> WireKiteTick {
        let mut tick = WireKiteTick::default();
        let Some(pl) = player else {
            return tick;
        };
        // `:35700` — `Main.myPlayer == this.owner`: the pool only simulates
        // local-owned slots, so the gate folds true for every simulated
        // kite; kept explicit for the owner-255 shape.
        if self.owner == local_owner {
            // `:35702-35705` — the move cooldown ticks down.
            if self.local_ai[1] > 0.0 {
                self.local_ai[1] -= 1.0;
            }
            // `:35706-35709` — the noItems/CCed/dead kill.
            if pl.no_items || pl.cced() || pl.dead {
                kill(self);
            } else if mouse_right && mouse_right_release {
                // `:35710-35715` — the right-click press kill: the
                // mouseInterface write folds into the use-input gates (the
                // module doc); blockMouse rides the tick out.
                kill(self);
                tick.block_mouse = true;
            } else if !pl.channel {
                // `:35716-35723` — the channel release: the localAI[0]
                // latch arms the Kill hook's MassWireOperation (the
                // wiring deferral, the module doc), then the kill.
                if self.local_ai[0] == 0.0 {
                    self.local_ai[0] = 1.0;
                }
                kill(self);
            } else if self.local_ai[1] == 0.0 {
                // `:35724-35744` — the mouse follow. `vector155 =
                // screenPosition + (mouseX, mouseY)`; the gravDir == -1
                // Y-flip (`:35727-35730`) folds away (gravDir = 1).
                if let Some(point) = aim {
                    let center = Vec2::new(self.pos.x + self.w * 0.5, self.pos.y + self.h * 0.5);
                    if point != center {
                        self.net_update = true;
                        self.pos = point - Vec2::new(self.w * 0.5, self.h * 0.5);
                        self.local_ai[1] = 1.0;
                    }
                    // `:35737-35743` — the first-move tile anchor: the
                    // `(int)Center / 16` truncation pair, the velocity pin.
                    if self.ai[0] == 0.0 && self.ai[1] == 0.0 {
                        let center =
                            Vec2::new(self.pos.x + self.w * 0.5, self.pos.y + self.h * 0.5);
                        self.ai[0] = (center.x as i32 / 16) as f32;
                        self.ai[1] = (center.y as i32 / 16) as f32;
                        self.net_update = true;
                        self.vel = Vec2::ZERO;
                    }
                }
            }
            // `:35745` — the velocity pin (every local tick, even a kill).
            self.vel = Vec2::ZERO;
            // `:35746-35823` — the two L-shaped tile scans are NO-OPS in
            // the source: `flag69` never sets, the bodies only bind an
            // unused tile read behind `InWorld` (the module doc's census).
            // Skipped — zero state, zero draws either way.
        }
        // `:35825-35832` — the owner tail (runs for every owner, dead
        // slots included — the C# has no return after the kills).
        let num937 = csign(pl.vx);
        if num937 != 0.0 {
            // `player.ChangeDir(num937)` — the facing write (the golf
            // precedent; the pulley/composite halves are unported, the
            // itemRotation flip is zeroed below anyway).
            pl.facing = num937 as i32;
        }
        pl.held_proj = whoami as i32;
        // `SetDummyItemTime(2)` (`Player.cs:4432-4441`) — the item timers
        // live on the App; the hold rides the tick out.
        tick.dummy_hold = true;
        pl.item_rotation = 0.0;
        tick
    }
}

#[cfg(test)]
mod w4_d13_style125_tests {
    use super::*;
    use crate::projectile::{WIRE_KITE, proj_data};
    use tr_core::World;

    fn world() -> World {
        World::new("t".into(), 64, 64)
    }

    fn player(w: &World) -> Player {
        let mut p = Player::spawn(w);
        p.facing = 1;
        p.channel = true;
        p
    }

    fn kite(pos: Vec2) -> Projectile {
        let mut p = Projectile::default();
        p.active = true;
        p.kind = WIRE_KITE;
        p.owner = 0;
        p.w = 10.0;
        p.h = 10.0;
        p.pos = pos;
        p.tile_collide = false;
        p
    }

    /// The row's `SetDefaults` slice (`Projectile.cs:6990-7000`).
    #[test]
    fn w4_d13_the_wire_kite_row_carries_the_csharp_set_defaults() {
        let d = proj_data(WIRE_KITE).unwrap();
        assert_eq!((d.w, d.h), (10.0, 10.0));
        assert_eq!(d.ai_style, 125);
        assert_eq!(d.penetrate, -1);
        assert_eq!(d.time_left, 3600, "the stock life (no override)");
        assert!(!d.tile_collide);
        assert!(d.ignore_water);
        assert!(!d.trap);
    }

    /// `:35724-35744` — the first move snaps the Centre to the mouse, arms
    /// the 1-tick cooldown and latches the tile anchor; the next tick (the
    /// cooldown still up) holds position.
    #[test]
    fn w4_d13_the_first_move_snaps_and_latches_the_tile_anchor() {
        let w = world();
        let mut pl = player(&w);
        let mut p = kite(Vec2::new(100.0, 100.0));
        let aim = Vec2::new(200.0, 300.0);
        let tick = p.ai_wire_kite(Some(&mut pl), Some(aim), false, false, 0, 7);
        assert_eq!(p.pos, Vec2::new(195.0, 295.0), "Centre = the mouse point");
        assert_eq!(p.local_ai[1], 1.0, "the move cooldown");
        assert_eq!(p.ai[0], (200.0f32 as i32 / 16) as f32, "the tile X anchor");
        assert_eq!(p.ai[1], (300.0f32 as i32 / 16) as f32, "the tile Y anchor");
        assert!(p.net_update);
        assert_eq!(p.vel, Vec2::ZERO);
        assert!(tick.dummy_hold, "the SetDummyItemTime(2) tail");
        assert_eq!(pl.held_proj, 7, "heldProj = whoAmI");
        assert_eq!(pl.item_rotation, 0.0);
        // The cooldown tick: localAI[1] 1 → 0, no second snap.
        p.pos = Vec2::new(50.0, 50.0);
        let _ = p.ai_wire_kite(Some(&mut pl), Some(aim), false, false, 0, 7);
        assert_eq!(p.local_ai[1], 0.0);
        assert_eq!(p.pos, Vec2::new(50.0, 50.0), "the cooldown holds the move");
    }

    /// `:35706-35723` — the three kill gates: dead, the right-click press
    /// (blockMouse rides out), and the channel release (the localAI[0]
    /// latch arms the Kill hook).
    #[test]
    fn w4_d13_the_three_kill_gates() {
        let w = world();
        // Dead owner.
        let mut pl = player(&w);
        pl.dead = true;
        let mut p = kite(Vec2::new(100.0, 100.0));
        let tick = p.ai_wire_kite(Some(&mut pl), None, false, false, 0, 0);
        assert!(!p.active);
        assert_eq!(p.time_left, 0);
        assert!(!tick.block_mouse);
        // The right-click PRESS (mouseRight && mouseRightRelease).
        let mut pl = player(&w);
        let mut p = kite(Vec2::new(100.0, 100.0));
        let tick = p.ai_wire_kite(Some(&mut pl), None, true, true, 0, 0);
        assert!(!p.active);
        assert!(tick.block_mouse, "Main.blockMouse = true");
        assert_eq!(p.local_ai[0], 0.0, "no release latch on the press kill");
        // The channel release: the latch arms, then the kill.
        let mut pl = player(&w);
        pl.channel = false;
        let mut p = kite(Vec2::new(100.0, 100.0));
        let _ = p.ai_wire_kite(Some(&mut pl), None, false, false, 0, 0);
        assert!(!p.active);
        assert_eq!(p.local_ai[0], 1.0, "the MassWireOperation latch");
        // A held right button (no release edge) does NOT kill.
        let mut pl = player(&w);
        let mut p = kite(Vec2::new(100.0, 100.0));
        let _ = p.ai_wire_kite(Some(&mut pl), None, true, false, 0, 0);
        assert!(p.active, "the press edge only");
    }

    /// `:35825-35829` — the velocity-sign facing write; a zero velocity
    /// keeps the facing.
    #[test]
    fn w4_d13_the_tail_faces_the_owner_velocity() {
        let w = world();
        let mut pl = player(&w);
        pl.vx = -2.0;
        let mut p = kite(Vec2::new(100.0, 100.0));
        let _ = p.ai_wire_kite(Some(&mut pl), None, false, false, 0, 0);
        assert_eq!(pl.facing, -1, "ChangeDir(sign(velocity.X))");
        pl.vx = 0.0;
        let _ = p.ai_wire_kite(Some(&mut pl), None, false, false, 0, 0);
        assert_eq!(pl.facing, -1, "a zero velocity keeps the facing");
    }
}