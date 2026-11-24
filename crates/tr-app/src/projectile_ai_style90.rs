//! W4-d9 aiStyle 90 — the MagicLantern 492 light pet (`AI()`'s inline arm
//! `Projectile.cs:32387-32514`). Census: aiStyles 82/83/84/85/88 were armed
//! on main; 90 was a gap (the id constant landed with the W4-d9 census).
//!
//! The lantern machine:
//! * `:32389-32396` — the owner-death kill (NO return — the C# keeps walking
//!   the arm on the dead slot) and the `magicLantern` keep-alive
//!   (`timeLeft = 2`).
//! * `:32397-32407` — the tileCollide flip: `!CanHit(owner)` drops it;
//!   `!SolidCollision && CanHitLine(owner)` restores it.
//! * `:32408-32409` — direction/spriteDirection pin to the owner's.
//! * `:32411-32434` — the 10-tick spelunker scan: the local-player distance
//!   gate (`screenWidth + 17*16`), the ±17-tile window, the per-cell
//!   `Next(4)` roll FIRST (the `&&` head — consumed for EVERY cell), the
//!   17-tile radius, the world bounds, the active + `IsTileSpelunkable`
//!   tile test, the dust 204 (skipped body).
//! * `:32435-32460` — the hover: the owner-centre offset (+40·direction,
//!   −40), the >1000 teleport (the stale vector/length keep driving the
//!   accel — the C# does not recompute), the >200 speed ramp +
//!   tileCollide drop, the near-brake (×0.25), the (vel·2 + aim)/3 blend.
//! * `:32461-32491` — the flying rotation (atan2 + 1.57, the ±3.14 wrap,
//!   the (rot·4 + target)/5 lerp) + the 4..7 frame cycle; the `return`
//!   exits `AI()` only — `Update` still runs HandleMovement
//!   (`Projectile.cs:16775` calls `AI()`, `:17015` the movement), so the
//!   dispatch runs `kill_step` after the arm either way.
//! * `:32492-32513` — the idle rotation decay (the >3.14 unwrap, the
//!   ±0.01 zero-snap, the ×0.9 decay) + the 0..3 frame cycle.
//!
//! `Main.IsTileSpelunkable` (`Main.cs:21783-21798`): the `tileSpelunker`
//! bitset (the `Main.Initialize` census `:9420-9467` + `:9651`, 49 ids)
//! plus the 185/186 frame exceptions. Hand-transcribed `TileSet::from_words`
//! (the `tr_core::wind::SWAYS_IN_WIND_BASIC` precedent — the tiledata
//! generator is due its own regeneration pass); the witness below asserts
//! every listed id.
//!
//! Documented deviations (W4-d9, code-only):
//! * the light (`:32410`) is skipped — the ProjLight seam owns lights;
//! * the scan's dust 204 body (`:32426-32429`) is skipped (no dust layer);
//!   only the EXPLICIT `Main.rand.Next(4)` draws are consumed, in C# order
//!   (the style-81/86 convention — `Dust.NewDust` internals are not drawn);
//! * the scan gate's `Main.player[Main.myPlayer].Center` folds onto the
//!   owner snapshot: the port simulates only the local owner's slots
//!   single-player (`Projectile.cs:17210`'s owner gate), so owner ==
//!   myPlayer here;
//! * `Main.screenWidth` rides the pool's `set_screen_width` seam (the
//!   `set_disco` precedent), fed from `app.screen_size()` per fixed step.
//!
//! The `Player.magicLantern` flag (`Player.cs:2082`, grant `:11372`, reset
//! `:18485`) is the field this arm reads; the grant side (the light-pet
//! buff pass) is unported — the flag stays false until it lands (the
//! AI_026 slice-A pet-flag precedent).
//!
//! Code-only round (W4-d9): the witness tests below are WRITTEN NOT RUN.

#![allow(dead_code)] // wiring deferred (W4-d9, code-only mode)

use glam::Vec2;
use tr_core::loot::DropRng;
use tr_core::tiledata::TileSet;
use tr_core::World;

use crate::projectile::{can_hit, can_hit_line, solid_collision, Projectile};

/// `Main.tileSpelunker` (`Main.cs:9420-9467` + `:9651`): the 49 ids
/// `{ 6, 7, 8, 9, 12, 21, 467, 441, 468, 28, 107, 108, 111, 63, 64, 65, 66,
/// 67, 68, 566, 166, 167, 168, 169, 178, 211, 221, 222, 223, 236, 702, 37,
/// 407, 227, 84, 83, 404, 242, 240, 246, 245, 105, 349, 337, 531, 506, 751,
/// 752, 639 }`. Hand-transcribed (the `SWAYS_IN_WIND_BASIC` precedent).
pub static TILE_SPELUNKER: TileSet = TileSet::from_words(&[
    0x8000_0020_1020_13C0, // 6,7,8,9,12,21,28,37,63
    0x0000_9A00_0018_001F, // 64,65,66,67,68,83,84,105,107,108,111
    0x0004_03C0_0000_0000, // 166,167,168,169,178
    0x0065_1008_E008_0000, // 211,221,222,223,227,236,240,242,245,246
    0x0000_0000_0000_0000,
    0x0000_0000_2002_0000, // 337,349
    0x0200_0000_0090_0000, // 404,407,441
    0x0400_0000_0018_0000, // 467,468,506
    0x0040_0000_0008_0000, // 531,566
    0x8000_0000_0000_0000, // 639
    0x4000_0000_0000_0000, // 702
    0x0001_8000_0000_0000, // 751,752
]);

/// `Main.IsTileSpelunkable(typeCache, tileFrameX, tileFrameY)`
/// (`Main.cs:21783-21798`): the bitset, then the 185/186 frame exceptions.
pub fn is_tile_spelunkable(kind: u16, frame_x: i16, frame_y: i16) -> bool {
    if TILE_SPELUNKER.has(kind) {
        return true;
    }
    // `:21789-21792` — the 185 statue frames (frameY 18, frameX 576..=882).
    if kind == 185 && frame_y == 18 && (576..=882).contains(&frame_x) {
        return true;
    }
    // `:21793-21796` — the 186 pile frames (frameX 864..=1170).
    if kind == 186 && (864..=1170).contains(&frame_x) {
        return true;
    }
    false
}

/// `Kill()` (`Projectile.cs:70644-70648`) — the sibling-module deactivation
/// (the `projectile_ai_spear` precedent; `Projectile::kill` is private).
fn kill(p: &mut Projectile) {
    p.kill_time_left = p.time_left;
    p.active = false;
    p.time_left = 0;
}

/// `Entity.Center` for the pool's slots (`pos + size/2`).
fn center(p: &Projectile) -> Vec2 {
    Vec2::new(p.pos.x + p.w * 0.5, p.pos.y + p.h * 0.5)
}

/// The arm's `Main.player[owner]` snapshot (the `SpearOwner` seam shape):
/// the centre, the direction, the death flag and the `magicLantern`
/// keep-alive (`Player.cs:2082`). The scan gate's `Main.myPlayer` centre
/// folds onto this snapshot (single-player, the module doc).
#[derive(Clone, Copy, Debug)]
pub(crate) struct LanternOwner {
    pub center: Vec2,
    pub direction: i32,
    pub dead: bool,
    pub magic_lantern: bool,
}

/// `AI()`'s aiStyle-90 arm (`Projectile.cs:32387-32514`). `screen_width` is
/// `Main.screenWidth` (the pool's `set_screen_width` seam), `rng` the
/// `UpdateProjectiles` stream.
pub(crate) fn ai_magic_lantern(
    p: &mut Projectile,
    world: &World,
    owner: LanternOwner,
    screen_width: i32,
    rng: &mut dyn DropRng,
) {
    // `:32389-32392` — the owner-death kill. NO return: the C# keeps
    // walking the arm on the dead slot (the writes are harmless).
    if owner.dead {
        kill(p);
    }
    // `:32393-32396` — the magicLantern keep-alive.
    if owner.magic_lantern {
        p.time_left = 2;
    }
    // `:32397-32407` — the tileCollide flip against the owner's 1x1 centre.
    if p.tile_collide {
        if !can_hit(
            world, p.pos.x, p.pos.y, p.w, p.h, owner.center.x, owner.center.y, 1.0, 1.0,
        ) {
            p.tile_collide = false;
        } else if !solid_collision(world, p.pos, p.w, p.h)
            && can_hit_line(
                world, p.pos.x, p.pos.y, p.w, p.h, owner.center.x, owner.center.y, 1.0, 1.0,
            )
        {
            p.tile_collide = true;
        }
    }
    // `:32408-32409` — the direction pin.
    p.direction = owner.direction;
    p.sprite_direction = p.direction;
    // `:32410` — the light (skipped: the ProjLight seam owns lights).
    // `:32411-32434` — the 10-tick spelunker scan.
    p.local_ai[0] += 1.0;
    if p.local_ai[0] >= 10.0 {
        p.local_ai[0] = 0.0;
        let num751 = 17;
        let c = center(p);
        // `:32416` — the local-player distance gate (the owner snapshot
        // stands in for Main.myPlayer, the module doc).
        if (c - owner.center).length() < (screen_width + num751 * 16) as f32 {
            let num752 = c.x as i32 / 16;
            let num753 = c.y as i32 / 16;
            for num754 in (num752 - num751)..=(num752 + num751) {
                for num755 in (num753 - num751)..=(num753 + num751) {
                    // `:32424` — the Next(4) roll is the && HEAD: consumed
                    // for EVERY cell, before the radius/bounds/tile tests.
                    let roll = rng.next_range(0, 4);
                    let radius =
                        Vec2::new((num752 - num754) as f32, (num753 - num755) as f32).length();
                    if roll == 0
                        && radius < num751 as f32
                        && num754 > 0
                        && num754 < world.width - 1
                        && num755 > 0
                        && num755 < world.height - 1
                    {
                        if let Some(t) = world.tile(num754, num755) {
                            if t.active() && is_tile_spelunkable(t.kind, t.frame_x, t.frame_y) {
                                // `:32426-32429` — the dust 204 (skipped
                                // body; the explicit draws above stand).
                            }
                        }
                    }
                }
            }
        }
    }
    // `:32435-32442` — the hover offset + the >1000 teleport. The C# keeps
    // the STALE vector/length driving the accel below (no recompute).
    let mut vector103 = owner.center - center(p);
    vector103.x += 40.0 * p.direction as f32;
    vector103.y -= 40.0;
    let num757 = vector103.length();
    if num757 > 1000.0 {
        // `base.Center = owner.Center` — the top-left re-centres.
        p.pos = owner.center - Vec2::new(p.w * 0.5, p.h * 0.5);
    }
    // `:32443-32459` — the speed ramp / near-brake / aim scale.
    let num758 = 3.0f32;
    let mut num759 = 4.0f32;
    if num757 > 200.0 {
        num759 += (num757 - 200.0) * 0.1;
        p.tile_collide = false;
    }
    if num757 < num759 {
        p.vel *= 0.25;
        num759 = num757;
    }
    if vector103.x != 0.0 || vector103.y != 0.0 {
        vector103 = vector103.normalize();
        vector103 *= num759;
    }
    // `:32460` — the (vel·(3−1) + aim)/3 blend.
    p.vel = (p.vel * (num758 - 1.0) + vector103) / num758;
    // `:32461-32491` — the flying rotation + frame cycle. The C# literals
    // are the truncated 1.57/3.14/6.28 floats, NOT PI/TAU.
    if p.vel.length() > 6.0 {
        let num760 = p.vel.y.atan2(p.vel.x) + 1.57;
        if (p.rot - num760).abs() >= 3.14 {
            if num760 < p.rot {
                p.rot -= 6.28;
            } else {
                p.rot += 6.28;
            }
        }
        p.rot = (p.rot * 4.0 + num760) / 5.0;
        p.frame_counter += 1;
        if p.frame_counter > 4 {
            p.frame_counter = 0;
            p.frame += 1;
            if p.frame > 7 {
                p.frame = 4;
            }
        }
        if p.frame < 4 {
            p.frame = 7;
        }
        // `:32490`'s return exits AI() only — Update still moves the slot
        // (the dispatch runs kill_step after the arm either way).
        return;
    }
    // `:32492-32503` — the idle rotation decay.
    if p.rot > 3.14 {
        p.rot -= 6.28;
    }
    if p.rot > -0.01 && p.rot < 0.01 {
        p.rot = 0.0;
    } else {
        p.rot *= 0.9;
    }
    // `:32504-32513` — the idle frame cycle.
    p.frame_counter += 1;
    if p.frame_counter > 6 {
        p.frame_counter = 0;
        p.frame += 1;
        if p.frame > 3 {
            p.frame = 0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::projectile::{proj_data, MAGIC_LANTERN};
    use tr_core::tile::{Tile, FLAG_ACTIVE};
    use tr_core::TILE_PX;

    struct Recorder {
        draws: Vec<(i32, i32)>,
    }
    impl DropRng for Recorder {
        fn next_range(&mut self, min: i32, max: i32) -> i32 {
            self.draws.push((min, max));
            1 // never 0 — the dust gate stays shut, the draws still count
        }
        fn next_float(&mut self) -> f32 {
            0.5
        }
    }

    fn proj() -> Projectile {
        let d = proj_data(MAGIC_LANTERN).expect("the 492 row");
        let mut p = Projectile::default();
        p.active = true;
        p.kind = MAGIC_LANTERN;
        p.owner = 0;
        p.friendly = true;
        p.pos = Vec2::new(10.0 * TILE_PX, 10.0 * TILE_PX);
        p.vel = Vec2::ZERO;
        p.w = d.w;
        p.h = d.h;
        p.damage = 0;
        p.time_left = d.time_left;
        p.penetrate = d.penetrate;
        p.tile_collide = d.tile_collide;
        p.net_important = d.net_important;
        p
    }

    fn owner_at(x: f32, y: f32) -> LanternOwner {
        LanternOwner { center: Vec2::new(x, y), direction: 1, dead: false, magic_lantern: false }
    }

    fn empty_world() -> World {
        World::new("t".into(), 64, 64)
    }

    /// The row matches `SetDefaults` verbatim (`Projectile.cs:5645-5654`).
    #[test]
    fn w4_d9_the_lantern_row_matches_set_defaults() {
        let d = proj_data(MAGIC_LANTERN).unwrap();
        assert_eq!((d.w, d.h), (18.0, 32.0));
        assert_eq!(d.ai_style, 90);
        assert_eq!(d.penetrate, -1);
        assert_eq!(d.time_left, 3600 * 5, "the timeLeft *= 5 (:5653)");
        assert!(d.net_important);
        assert!(d.tile_collide, "the stock tileCollide true (:775) stands");
    }

    /// The hand-transcribed bitset carries every census id, and the
    /// 185/186 frame exceptions match `Main.cs:21789-21796`.
    #[test]
    fn w4_d9_the_spelunker_set_and_frame_exceptions() {
        for id in [
            6, 7, 8, 9, 12, 21, 467, 441, 468, 28, 107, 108, 111, 63, 64, 65, 66, 67, 68,
            566, 166, 167, 168, 169, 178, 211, 221, 222, 223, 236, 702, 37, 407, 227, 84,
            83, 404, 242, 240, 246, 245, 105, 349, 337, 531, 506, 751, 752, 639,
        ] {
            assert!(is_tile_spelunkable(id, 0, 0), "the census id {id}");
        }
        for id in [0, 1, 2, 185, 186, 753] {
            assert!(!is_tile_spelunkable(id, 0, 0), "the non-member {id}");
        }
        // The 185 statue frames: frameY 18, frameX 576..=882.
        assert!(is_tile_spelunkable(185, 576, 18));
        assert!(is_tile_spelunkable(185, 882, 18));
        assert!(!is_tile_spelunkable(185, 574, 18));
        assert!(!is_tile_spelunkable(185, 884, 18));
        assert!(!is_tile_spelunkable(185, 600, 0));
        // The 186 pile frames: frameX 864..=1170 (any frameY).
        assert!(is_tile_spelunkable(186, 864, 0));
        assert!(is_tile_spelunkable(186, 1170, 36));
        assert!(!is_tile_spelunkable(186, 862, 0));
        assert!(!is_tile_spelunkable(186, 1172, 0));
    }

    /// The owner-death kill (`:32389-32392`).
    #[test]
    fn w4_d9_the_lantern_dies_with_its_owner() {
        let mut p = proj();
        let w = empty_world();
        let mut o = owner_at(p.pos.x + 9.0, p.pos.y + 16.0);
        o.dead = true;
        let mut rng = Recorder { draws: Vec::new() };
        ai_magic_lantern(&mut p, &w, o, 1280, &mut rng);
        assert!(!p.active, "the dead owner kills the lantern");
    }

    /// The magicLantern keep-alive pins timeLeft = 2 (`:32393-32396`).
    #[test]
    fn w4_d9_the_lantern_keep_alive_pins_time_left() {
        let mut p = proj();
        let w = empty_world();
        let mut o = owner_at(p.pos.x + 9.0, p.pos.y + 16.0);
        o.magic_lantern = true;
        let mut rng = Recorder { draws: Vec::new() };
        ai_magic_lantern(&mut p, &w, o, 1280, &mut rng);
        assert_eq!(p.time_left, 2);
        assert!(p.active);
    }

    /// The >1000 teleport re-centres on the owner (`:32439-32442`) and the
    /// >200 ramp drops tileCollide (`:32445-32449`).
    #[test]
    fn w4_d9_the_lantern_teleports_when_far() {
        let mut p = proj();
        let w = empty_world();
        let o = owner_at(p.pos.x + 9.0 + 2000.0, p.pos.y + 16.0);
        let mut rng = Recorder { draws: Vec::new() };
        ai_magic_lantern(&mut p, &w, o, 1280, &mut rng);
        assert_eq!(
            p.pos,
            o.center - Vec2::new(9.0, 16.0),
            "the Centre pin re-centres the top-left"
        );
        assert!(!p.tile_collide, "the >200 ramp drops tileCollide");
    }

    /// The 10-tick scan consumes one Next(4) per cell of the ±17 window
    /// (35×35 = 1225 draws), in row-major order (`:32420-32424`).
    #[test]
    fn w4_d9_the_lantern_scan_consumes_every_cell_roll() {
        let mut p = proj();
        // Centre on tile (32, 32): the whole ±17 window is in bounds.
        p.pos = Vec2::new(32.0 * TILE_PX - 9.0, 32.0 * TILE_PX - 16.0);
        p.local_ai[0] = 9.0; // the next tick is the scan tick
        let w = empty_world();
        let o = owner_at(32.0 * TILE_PX, 32.0 * TILE_PX);
        let mut rng = Recorder { draws: Vec::new() };
        ai_magic_lantern(&mut p, &w, o, 1280, &mut rng);
        assert_eq!(p.local_ai[0], 0.0, "the counter resets");
        assert_eq!(rng.draws.len(), 35 * 35, "one roll per window cell");
        assert!(rng.draws.iter().all(|&d| d == (0, 4)), "every draw is Next(4)");
    }

    /// The tileCollide flip: a solid wall between the lantern and the
    /// owner fails CanHit (`:32399-32402`); open air keeps it true
    /// (`:32403-32406`).
    #[test]
    fn w4_d9_the_lantern_tile_collide_flips_on_the_wall() {
        let mut w = empty_world();
        // A full solid column at x = 20 (the can_hit walk clamps to
        // height − 40 = 24, so 0..24 seals it).
        for y in 0..24 {
            let i = w.index(20, y);
            w.tiles[i] = Tile { kind: 1, ..Default::default() };
            w.tiles[i].flags |= FLAG_ACTIVE;
        }
        let mut p = proj();
        p.pos = Vec2::new(280.0 - 9.0, 160.0 - 16.0); // centre (280, 160)
        let o = owner_at(380.0, 160.0); // 100 px away, across the wall
        let mut rng = Recorder { draws: Vec::new() };
        ai_magic_lantern(&mut p, &w, o, 1280, &mut rng);
        assert!(!p.tile_collide, "the wall fails CanHit");

        let mut p2 = proj();
        p2.pos = Vec2::new(280.0 - 9.0, 160.0 - 16.0);
        let w2 = empty_world();
        let mut rng2 = Recorder { draws: Vec::new() };
        ai_magic_lantern(&mut p2, &w2, o, 1280, &mut rng2);
        assert!(p2.tile_collide, "open air keeps tileCollide");
    }

    /// The idle rotation decay: the ±0.01 zero-snap and the ×0.9 decay
    /// (`:32492-32503`), plus the 0..3 frame cycle (`:32504-32513`).
    #[test]
    fn w4_d9_the_lantern_idle_rotation_and_frames() {
        let mut p = proj();
        let w = empty_world();
        // The owner exactly at the hover point: vector103 = (40, -40)
        // minus itself → zero aim, the near-brake keeps the speed under 6.
        let c = center(&p);
        let o = owner_at(c.x - 40.0, c.y + 40.0);
        p.rot = 0.005;
        let mut rng = Recorder { draws: Vec::new() };
        ai_magic_lantern(&mut p, &w, o, 1280, &mut rng);
        assert_eq!(p.rot, 0.0, "the ±0.01 snap");
        p.rot = 1.0;
        p.frame_counter = 0; // isolate the 7-tick cycle from the snap call
        for _ in 0..7 {
            ai_magic_lantern(&mut p, &w, o, 1280, &mut rng);
        }
        assert!((p.rot - 1.0 * 0.9f32.powi(7)).abs() < 1e-6, "the ×0.9 decay");
        assert_eq!(p.frame, 1, "the 7th idle tick rolls the frame");
        assert_eq!(p.frame_counter, 0);
    }
}
