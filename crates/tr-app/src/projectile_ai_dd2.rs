//! W4-d15 — the DD2 weapon/pet AI slice: aiStyles **141-146** + **148**
//! (147 Celebration Mk2 already rides `projectile_ai_misc::ai_celeb2_rocket`;
//! 149/150 golf already ride `projectile_ai_golf` — the census of the
//! `match data.ai_style` dispatch). Seven MISSING styles ported verbatim:
//!
//! - aiStyle 141: `AI_141_MonkStaffT1Explosion` `Projectile.cs:69602-69620`
//!   + `CreateImpactExplosion_SleepyOctopod` `:69680-69731` +
//!   `CreateImpactExplosion2_SleepyOctopod` `:69733-69772` (the 698 row
//!   `:7385-7401`).
//! - aiStyle 142: `AI_142_MonkStaffT2And3` `:69817-69968` (the 699 row
//!   `:7402-7418`, the 708 row `:7491-7507`).
//! - aiStyle 143: `AI_143_MonkStaffT2Ghast` `:69970-70036` (the 700 row
//!   `:7419-7432`).
//! - aiStyle 144: `AI_144_DD2Pet` `:70038-70324` (the 701/702/703 row
//!   `:7433-7446`, the 764 row `:7866-7878`, the 1050 row `:9953-9966`).
//! - aiStyle 145: `AI_145_BookStaffStorm` `:70326-70468` (the 704 row
//!   `:7447-7460`).
//! - aiStyle 146: `AI_146_DD2Victory` `:53960-53962` — an EMPTY C# body
//!   (the 713 row `:7560-7571`); the port keeps the arm a documented no-op.
//! - aiStyle 148: `AI_148_StarSpawner` `:53094-53128` (the 720 row
//!   `:7617-7625`) + `Star.StarFall`/`Star.Fall` (`Star.cs`) + the spawned
//!   FallingStar 12's row (`Projectile.cs:924-938`).
//!
//! RNG contract (the `UpdateProjectiles` stream, `Main.cs:17775-17777`):
//! every EXPLICIT `Main.rand` draw at the call sites stays, in the C#'s
//! order; the `Dust.NewDust*` internal rolls go with the dust pool (the
//! `projectile_ai_misc` precedent). The `Gore.NewGoreDirect` internal kicks
//! DO stay — `Next(10, 31)` on Y first, then `Next(-20, 21)` on X
//! (`Gore.cs:1327-1328`, the LEAD-REVIEW G3 standard). `NextVector2Square`
//! is 2 `NextDouble` draws (`Utils.cs:1251-1254`, the
//! `npc_ai_frost_boss_b` census convention); `Utils.SelectRandom` is one
//! `Next(choices.Length)` (`Utils.cs:2459-2462`).
//!
//! Recorded folds (docs/ANALYSIS.md "The W4-d15 DD2 slice"):
//! - the tracked sounds (`AI_143`'s `DD2_GhastlyGlaiveImpactGhost`
//!   `:69974`, `AI_145`'s `DD2_BookStaffTwisterLoop` `:70332` + the volume
//!   ramp `:70338`) — the pool has no tracked-sound layer (S1-E-23); the
//!   `localAI[1]` slot latch is kept as inert state (`SlotId.Invalid` =
//!   `f32::from_bits(65535)`, `ReLogic.Utilities/SlotId.cs:5`/`:75-78`),
//!   zero draws.
//! - `AI_144`'s `Utils.PlotTileLine` light casts (`:70087-70091`), the
//!   `Opacity` write (`:70155`, no port field — the Last Prism fold) and
//!   the dead `Hitbox.Intersects` read (`:70143-70144`, result discarded).
//! - `AI_144`'s 1050 tree-shake `KillTile_MakeTileDust`/`PlaySounds`/
//!   `DropItems` (`:70217-70229`) — no tile-drop seam in the pool; the
//!   walk + ALL its `Main.rand` draws stay (`Next(100)`, the conditional
//!   `Next(20)`, the conditional `Next(5)`), the effects are the recorded
//!   skip. `KillTile_MakeTileDust`'s own rolls for the shakeable kinds
//!   ride `WorldGen.genRand` (the worldgen stream, not `Main.rand`) or
//!   the dust pool — zero `UpdateProjectiles` draws. `TileID.Sets.
//!   IsShakeable` (`TileID.cs:488`) rides the generated
//!   `tr_core::tiledata::SET_IS_SHAKEABLE`.
//! - `AI_148`'s `Star.StarFall` (`Star.cs:63-84`) picks a BACKGROUND star
//!   and `Star.Fall()` rolls `Next(700,1001)` then `Next(-400,401)` — the
//!   port has no falling-background-star machine (`render/sky.rs`'s
//!   `StarSprite` has no fall state), so the draws stay on the stream as
//!   the census and the visual fall is the recorded skip. The one-shot
//!   gate keeps the C#'s `Main.netMode != 2` form as single-player true.
//! - `AI_142`'s `player.heldProj` write rides the mutable player channel
//!   (the `ai_stardust_guardian` shape); `player.reuseDelay = 2`
//!   (`:69966`) rides [`Dd2Tick::reuse_delay`] back to the world-sim pass
//!   (the `FlailTick.facing` write-back precedent).
//! - `manualDirectionChange = true` on the pet rows (`:7445`) — no port
//!   field; the arms pin `direction`/`spriteDirection` themselves and the
//!   dispatch's free integrate never rewrites facing (the aiStyle-120
//!   precedent).
//! - the 698's `Colliding` overlay (`:15451-15467`, the +70 inflate and
//!   the velocity-shifted second rect) — `damage_hitbox`'s per-type
//!   overlays are the S1-C-8 follow-up, unchanged here.
//!
//! Code-only round (W4-d15): the witness tests below are WRITTEN NOT RUN.

use glam::Vec2;
use tr_core::World;
use tr_core::loot::DropRng;

use crate::projectile::Projectile;

/// The sibling-module deactivation (`Projectile::kill` is private — the
/// `projectile_ai_spear` precedent).
fn kill(p: &mut Projectile) {
    p.kill_time_left = p.time_left;
    p.active = false;
    p.time_left = 0;
}

/// `AI_141_MonkStaffT1Explosion` (`Projectile.cs:69602-69620`) — the
/// Sleepy Octopod's ground blast 698. The scan box is the slot's
/// TopLeft..BottomRight tile span with `scanAreaEnd.Y++` (`:69609`), the
/// explosion range is the WIDTH (`:69608`); the counter kills past 20
/// and the tick-1 arm runs both impact helpers. The `scanAreaStart.X / 2`
/// discards (`:69606-69607`) are C# dead reads — not carried.
pub(crate) fn ai_monk_staff_t1_explosion(p: &mut Projectile, world: &World, rng: &mut dyn DropRng) {
    // `:69604-69609` — the tile span (`ToTileCoordinates` = `(int)(X/16)`).
    let x0 = (p.pos.x / 16.0) as i32;
    let y0 = (p.pos.y / 16.0) as i32;
    let x1 = ((p.pos.x + p.w) / 16.0) as i32;
    let y1 = ((p.pos.y + p.h) / 16.0) as i32 + 1;
    let explosion_range = p.w;
    // `:69610-69614` — the counter and the 20-tick kill.
    p.ai[0] += 1.0;
    if p.ai[0] > 20.0 {
        kill(p);
    } else if p.ai[0] == 1.0 {
        // `:69617` — `base.Bottom` = (x + w/2, y + h) (`Entity.cs:122`).
        let bottom = Vec2::new(p.pos.x + p.w * 0.5, p.pos.y + p.h);
        let shocked = impact_explosion_sleepy_octopod(p, world, rng, 8, bottom, x0, y0, x1, y1, explosion_range);
        // `:69618` — the ring origin backs off 10× the velocity.
        impact_explosion2_sleepy_octopod(p, rng, bottom - p.vel * 10.0, shocked);
    }
}

/// `CreateImpactExplosion_SleepyOctopod` (`Projectile.cs:69680-69731`) —
/// the tile walk as the DRAW CENSUS: `num = 3f` (the velocity.X kick
/// scale), `num2 = 4` (the dust layer), `num3 = 0.75f` (the scale pin).
/// Per qualifying tile: the dust amount × multiplier, then that many
/// `KillTile_MakeTileDust` bodies (each one `NextFloat` velocity.Y scale,
/// `:69709`; the X kick `:69710` rolls its own `NextFloat`), then
/// `num2 >= 2`'s second row of amount−1 (one `NextFloat` each, `:69720`;
/// the X kick `:69721` is fixed — no draw). `KillTile_GetTileDustAmount`
/// rides the misc module's port (`WorldGen.cs:66597-66685`).
fn impact_explosion_sleepy_octopod(
    _p: &Projectile,
    world: &World,
    rng: &mut dyn DropRng,
    dust_amount_multiplier: i32,
    origin: Vec2,
    x0: i32,
    y0: i32,
    x1: i32,
    y1: i32,
    explosion_range: f32,
) -> bool {
    let mut caused_shockwaves = false;
    for i in x0..=x1 {
        for j in y0..=y1 {
            // `:69690-69693` — the radius gate against the tile CORNER.
            if origin.distance(Vec2::new(i as f32 * 16.0, j as f32 * 16.0)) > explosion_range {
                continue;
            }
            // `:69694-69703` — the exposed full-solid gate (the
            // create_ground_explosion shape).
            let Some(t) = world.tile(i, j) else { continue };
            let solid = tr_core::tiledata::TILE_SOLID.has(t.kind);
            let solid_top = tr_core::tiledata::TILE_SOLID_TOP.has(t.kind);
            let important = world.frame_important.get(t.kind as usize).copied().unwrap_or(false);
            if !t.active() || !solid || solid_top || important {
                continue;
            }
            let above_full_solid = world.tile(i, j - 1).is_some_and(|a| {
                a.active()
                    && tr_core::tiledata::TILE_SOLID.has(a.kind)
                    && !tr_core::tiledata::TILE_SOLID_TOP.has(a.kind)
            });
            if above_full_solid {
                continue;
            }
            // `:69704-69713` — the first dust row: TWO NextFloats each
            // (the Y scale `:69709` + the X kick's `0.3+0.7*NextFloat`
            // `:69710`).
            let amount = crate::projectile_ai_misc::kill_tile_dust_amount_fail(t.kind, t.frame_x)
                * dust_amount_multiplier;
            for _ in 0..amount {
                let _ = rng.next_float();
                let _ = rng.next_float();
            }
            // `:69714-69724` — num2 = 4 >= 2: the second row, amount−1
            // dusts with ONE NextFloat each (`:69720`).
            for _ in 0..(amount - 1) {
                let _ = rng.next_float();
            }
            if amount > 0 {
                caused_shockwaves = true;
            }
        }
    }
    caused_shockwaves
}

/// `CreateImpactExplosion2_SleepyOctopod` (`Projectile.cs:69733-69772`) —
/// the 25-point ring census: each particle rolls TWO `NextFloat`s (the
/// X kick `:69752` + the fadeIn `:69753`); the `!causedShockwaves` second
/// ring adds 25 more with ONE each (the fadeIn `:69768` — the X kick
/// `:69767` is fixed). The rotations/colour are draw-only — skipped.
fn impact_explosion2_sleepy_octopod(p: &Projectile, rng: &mut dyn DropRng, _origin: Vec2, shocked: bool) {
    let _ = p;
    for _ in 0..25 {
        let _ = rng.next_float(); // `:69752`
        let _ = rng.next_float(); // `:69753`
    }
    if !shocked {
        for _ in 0..25 {
            let _ = rng.next_float(); // `:69768`
        }
    }
}

// Witnesses (W4-d15 code-only round — WRITTEN NOT RUN).

#[cfg(test)]
mod tests {
    use super::*;
    use crate::projectile::{MONK_STAFF_T1_EXPLOSION, proj_data};
    use tr_core::tile::{FLAG_ACTIVE, Tile};

    /// A draw-counting `Main.rand` stand-in (the `projectile_ai_misc`
    /// CensusRng shape): `next_range` returns `min`, `next_float` 0.
    struct CensusRng {
        ranges: Vec<(i32, i32)>,
        floats: usize,
    }
    impl DropRng for CensusRng {
        fn next_range(&mut self, min: i32, max: i32) -> i32 {
            self.ranges.push((min, max));
            min
        }
        fn next_float(&mut self) -> f32 {
            self.floats += 1;
            0.0
        }
    }
    fn census() -> CensusRng {
        CensusRng { ranges: Vec::new(), floats: 0 }
    }

    fn world() -> World {
        World::new("t".into(), 300, 200)
    }

    fn proj(kind: u16) -> Projectile {
        let d = proj_data(kind).expect("the rowed kind");
        let mut p = Projectile::default();
        p.active = true;
        p.kind = kind;
        p.owner = 0;
        p.pos = Vec2::new(100.0 * 16.0, 50.0 * 16.0);
        p.w = d.w;
        p.h = d.h;
        p.tile_collide = d.tile_collide;
        p.alpha = d.alpha;
        p.scale = d.scale;
        p.time_left = d.time_left;
        p.penetrate = d.penetrate;
        p.max_penetrate = d.penetrate;
        p
    }

    fn put(w: &mut World, x: i32, y: i32, kind: u16) {
        let i = w.index(x, y);
        w.tiles[i] = Tile { kind, ..Default::default() };
        w.tiles[i].flags |= FLAG_ACTIVE;
    }

    /// The 698 row is verbatim `SetDefaults` (`Projectile.cs:7385-7401`).
    #[test]
    fn the_698_row_is_verbatim() {
        let d = proj_data(MONK_STAFF_T1_EXPLOSION).expect("the 698 row");
        assert_eq!((d.w, d.h), (75.0, 200.0), ":7387-7388");
        assert_eq!(d.ai_style, 141, ":7389");
        assert_eq!(d.alpha, 255, ":7390");
        assert!(!d.tile_collide, ":7391");
        assert!(d.ignore_water, ":7392");
        assert_eq!(d.time_left, 5, ":7393");
        assert_eq!(d.penetrate, -1, ":7395");
        assert!(d.uses_local_immunity, ":7396");
        assert_eq!(d.local_hit_cooldown, -1, ":7397");
        assert!(d.melee, ":7398");
    }

    /// Tick 1 over ONE exposed dirt tile: amount 3 × 8 = 24 → the first
    /// row 24×2 floats + the second row 23×1 = 71, plus the shocked ring
    /// 25×2 = 50 → 121 floats, zero ranges. The counter advances to 1.
    #[test]
    fn tick_one_over_one_tile_is_the_121_float_census() {
        let mut w = world();
        // The blast Bottom sits at (100*16 + 37.5, 52*16); the tile at
        // (102, 62) is inside the 75-px radius and exposed.
        put(&mut w, 102, 62, 1); // dirt — amount 3 (the fail table default)
        let mut p = proj(MONK_STAFF_T1_EXPLOSION);
        let mut rng = census();
        ai_monk_staff_t1_explosion(&mut p, &w, &mut rng);
        assert_eq!(p.ai[0], 1.0, ":69610");
        assert!(p.active);
        assert_eq!(rng.floats, 24 * 2 + 23 + 25 * 2, ":69705-69724 + :69739-69755");
        assert!(rng.ranges.is_empty());
    }

    /// No qualifying tile → `causedShockwaves = false` → BOTH rings:
    /// 50 + 25 = 75 floats.
    #[test]
    fn tick_one_in_open_air_draws_both_rings() {
        let w = world();
        let mut p = proj(MONK_STAFF_T1_EXPLOSION);
        let mut rng = census();
        ai_monk_staff_t1_explosion(&mut p, &w, &mut rng);
        assert_eq!(rng.floats, 75, ":69739-69770 — the unshocked double ring");
    }

    /// The counter kills past 20 (`:69611-69613`); ticks 2..20 draw
    /// nothing (the impact helpers run on tick 1 only).
    #[test]
    fn the_counter_kills_past_twenty_and_the_middle_ticks_are_draw_free() {
        let w = world();
        let mut p = proj(MONK_STAFF_T1_EXPLOSION);
        p.ai[0] = 19.0;
        let mut rng = census();
        ai_monk_staff_t1_explosion(&mut p, &w, &mut rng);
        assert!(p.active, "tick 20 stays live");
        assert_eq!(rng.floats, 0);
        ai_monk_staff_t1_explosion(&mut p, &w, &mut rng);
        assert!(!p.active, "tick 21 kills");
        assert_eq!(p.time_left, 0);
    }
}
