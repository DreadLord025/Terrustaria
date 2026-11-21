//! W4-d16 — projectile aiStyle 160, the kites (`AI_160_Kites`
//! `Projectile.cs:48421-48464` + `KiteLogic` `:48466-48693`, the dispatch
//! `:36603-36606`). The 24 `DefaultToKite` kinds (`:10744-10751`): 4x4,
//! penetrate −1, `extraUpdates = 60` — the FIRST tick runs 61 sub-steps
//! (the `numUpdates` while `:16675-16678`), the `numUpdates == 1`
//! sub-step zeroes `extraUpdates` (`:48516-48519`) and every later tick
//! is a single sub-step. The early sub-steps' `KiteLogic` returns at
//! `:48542-48546` (`position -= velocity` — the dry move's net-zero
//! rewind); the reel counter (`:48473-48494`) runs only while
//! `extraUpdates == 0`.
//!
//! RNG contract (the `projectile_ai_misc` W4-d16 precedent): the ONLY
//! explicit `Main.rand` draw is the 824/839/840/853 frame roll
//! `Next(4)` (`:48670`) — state-bearing (it pins the sheet row), so it
//! stays on the pass's `UpdateProjectiles` stream. No dust/gore bodies.
//!
//! Skips (ANALYSIS): the `player.afkCounter`/`afkCounterForKiting`
//! resets (`:48597-48598`) have no port field (the `hotbar.rs:67`
//! precedent); `ChangeDir`'s pulley snap is unreachable (the pulley
//! gate kills the kite first, `:48435-48438`) and its composite-arm /
//! itemRotation flips are render-side — the facing write stands alone
//! (the yoyo `tick.facing` precedent); `GetArmPosition`'s hand offset
//! folds to `RotatedRelativePoint(MountedCenter)` (the spear anchor
//! precedent, `Projectile.cs:45860`); `ApplyBuffTo`'s trap +
//! deadMansSweater halving (`:12393-12396`) is moot (kites are not
//! traps). The Item192 cue's `Projectile.kiteSoundPitch` static
//! (`:48505`, `LegacySoundPlayer.cs:628-630`) rides the `wants_sound`
//! pitchOffset — the case-2 192 arm draws NOTHING and returns pitch 0
//! (`sound.rs`), so the final `pitch + pitchOffset` is the kite pitch
//! exactly.
//!
//! CODE-ONLY MODE (W4-d16): the witness tests are WRITTEN NOT RUN.

use glam::Vec2;
use tr_core::World;
use tr_core::loot::DropRng;

use crate::projectile::{Projectile, get_lerp_value, safe_normalize};

/// Every kind `DefaultToKite` rows (`Projectile.cs:7889-7892`/
/// `:8040-8044`/`:8055-8059`/`:8136-8140`/`:8149-8153`/`:8168-8172`/
/// `:8182-8186`): 766-771, 822-824, 826-830, 838-840, 843-846, 850,
/// 852-853.
pub const KITE_KINDS: [u16; 24] = [
    766, 767, 768, 769, 770, 771, 822, 823, 824, 826, 827, 828, 829, 830, 838,
    839, 840, 843, 844, 845, 846, 850, 852, 853,
];

/// The app-level per-tick reads `AI_160_Kites`/`KiteLogic` take from
/// `Main.player[owner]`'s item state (the pool cannot borrow the App).
/// The world-sim pass feeds it through `set_kite_context` (the
/// `set_spear_owner` pattern); the live player fields (channel, facing,
/// buffs, heldProj) ride the dispatch's `&mut Player` directly (the
/// baby-bird precedent).
#[derive(Clone, Copy, Debug)]
pub struct KiteContext {
    /// `player.inventory[player.selectedItem].shoot` (`:48431`) — 0
    /// when the held slot is empty or shoots nothing.
    pub selected_shoot: u16,
    /// `player.itemAnimation` (`:48485`) — the port's swing timer in
    /// ticks (the `ticks_of` convention, `world_sim/editing.rs`).
    pub item_animation: i32,
    /// `player.controlUseItem` (`:48485`) — the raw use button (the
    /// flail `control_use_item` precedent, NOT the channel latch).
    pub control_use_item: bool,
}

/// `WorldGen.InAPlaceWithWind(position, width, height)`
/// (`WorldGen.cs:86841-86846` → `:86858-86879`): the tile rect the box
/// covers, the wind-blown height gate (`DoesWindBlowAtThisHeight`
/// `:86848-86856`, the REMIX flip) and the per-cell liquid/wall refusal.
/// The pool holds no `Wind` struct — the world's `world_surface` +
/// seed flags carry the same state (`tr_core::wind::Wind`'s fields).
fn in_a_place_with_wind(world: &World, pos: Vec2, w: f32, h: f32) -> bool {
    let x = (pos.x / 16.0) as i32;
    let y = (pos.y / 16.0) as i32;
    let x2 = ((pos.x + w) / 16.0) as i32;
    let y2 = ((pos.y + h) / 16.0) as i32;
    let (ww, hh) = (1 + x2 - x, 1 + y2 - y);
    let mut blows = (y as f64) < world.world_surface;
    if world.seeds.contains(tr_core::seeds::SeedFlags::REMIX) {
        blows = !blows;
    }
    if !blows {
        return false;
    }
    for i in 0..ww {
        for j in 0..hh {
            let Some(t) = world.tile(x + i, y + j) else {
                return false;
            };
            if t.liquid > 0 || (t.wall > 0 && !tr_core::wind::wall_allows_wind(t.wall)) {
                return false;
            }
        }
    }
    true
}

/// `GetCollisionParams`' kite arms (`Projectile.cs:19892-19915`): the
/// tall collision boxes (34/58/38/22 px) anchored at 0.75 — the rest
/// keep the stock box at 0.5.
fn collision_params(kind: u16, w: f32, h: f32) -> (f32, f32, Vec2) {
    match kind {
        771 | 822 | 823 | 843 | 845 | 846 | 852 => (w, 34.0, Vec2::splat(0.75)),
        824 | 839 | 840 | 850 | 853 => (w, 58.0, Vec2::splat(0.75)),
        826 | 830 | 838 => (w, 38.0, Vec2::splat(0.75)),
        827 | 828 | 829 | 844 => (w, 22.0, Vec2::splat(0.75)),
        _ => (w, h, Vec2::splat(0.5)),
    }
}

/// The port's death convention (`Projectile::kill`, the aiStyle-155
/// precedent): latch `kill_time_left`, deactivate, zero the life.
fn kill(p: &mut Projectile) {
    p.kill_time_left = p.time_left;
    p.active = false;
    p.time_left = 0;
}

/// `AI_160_Kites` (`Projectile.cs:48421-48464`) + `KiteLogic`
/// (`:48466-48693`) on one slot, ONE sub-step. `num_updates` is the C#
/// `numUpdates` the sub-step body sees (`sub_steps - 2 - sub`, the
/// ai001_c precedent — the decrement at `:16678` runs BEFORE `AI()`).
/// `who_am_i` is the pool slot index (`base.whoAmI`, the dragon
/// precedent). `scene` is the pool's `(WindForVisuals,
/// timeForVisualEffects, cloudAlpha, GlobalTimeWrappedHourly)`.
///
/// Movement: the arm owns its `HandleMovement` half (the flail
/// precedent — the slot never integrates twice). The blocked-move
/// chain EXCLUDES aiStyle 160 (`:18398`), so a kite slides on tiles
/// (the `TileCollision` axis zeroing) and never dies on contact.
pub(crate) fn kite_step(
    p: &mut Projectile,
    world: &World,
    player: Option<&mut crate::player::Player>,
    ctx: &KiteContext,
    scene: (f32, f64, f32, f32),
    rng: &mut dyn DropRng,
    local_owner: u8,
    num_updates: i32,
    who_am_i: i32,
) {
    // The C# reads `Main.player[owner]` unconditionally; an unresolved
    // owner folds the generic owner-inactive kill (`:16782-16785`).
    // `HandleMovement` still runs after it (`:17015` precedes the
    // `!active` return `:17027`).
    let Some(pl) = player else {
        kill(p);
        kite_move(p, world);
        return;
    };
    // `:48424-48425` — the held-proj pin + the life refresh (EVERY
    // sub-step, ahead of the kill gates).
    pl.held_proj = who_am_i;
    p.time_left = 60;
    let local = p.owner == local_owner;
    // `:48426-48446` — the kill gates (the else-if chain all set the
    // same flag — the OR form is equivalent; the distance gate runs
    // only when none fired).
    let mut flag = pl.cced() || pl.no_items;
    if !flag {
        flag = ctx.selected_shoot != p.kind;
    }
    if !flag {
        flag = pl.pulley;
    }
    if !flag {
        flag = pl.dead;
    }
    if !flag {
        let (pcx, pcy) = pl.center();
        flag = Vec2::new(pcx, pcy).distance(Vec2::from(p.center())) > 2000.0;
    }
    if flag {
        kill(p);
        // The C# `return` lands in `Update`, which still runs
        // `HandleMovement` (`:17015`) before the `!active` return
        // (`:17027`) — the dying kite integrates one final step.
        kite_move(p, world);
        return;
    }
    // `:48452-48459` — the KiteFlyingInfo: BobOffset 0, WindForVisuals,
    // cloudAlpha, GlobalTimeWrappedHourly, CanReelThroughBlocks true;
    // the anchor is `player.GetArmPosition()` (the module-doc stand-in).
    let (mcx, mcy) = pl.mounted_center();
    let anchor = pl.rotated_relative_point(Vec2::new(mcx, mcy), 0);
    kite_logic(p, world, pl, ctx, scene, anchor, rng, local, num_updates);
    // `:48460-48463` — the kite-luck buff gate (runs even on the
    // early-return sub-steps — the C# `return` is KiteLogic's, not
    // AI_160's). `ApplyBuffTo` → `player.AddBuff(383, 12)`.
    if local && p.local_ai[1] < 1.0 && p.pos.y + p.h < pl.y {
        pl.buffs.add_buff(383, 12);
    }
    // `HandleMovement` (`:17015`) — the kite's slide half. The C# runs it
    // BEFORE the `!active` return (`:17027`), so a slot the reel kill just
    // ended still integrates one final step (the `kill_step` convention).
    kite_move(p, world);
}

/// `KiteLogic` (`Projectile.cs:48466-48693`) — the reel counter, the
/// length-change cue latch, the wind hover, the anchor reel and the
/// frame machine. See the module doc for the sub-step contract.
fn kite_logic(
    p: &mut Projectile,
    world: &World,
    pl: &mut crate::player::Player,
    ctx: &KiteContext,
    scene: (f32, f64, f32, f32),
    anchor: Vec2,
    rng: &mut dyn DropRng,
    local: bool,
    num_updates: i32,
) {
    let (wind_visual, _visual_time, cloud_alpha, global_time) = scene;
    // `:48469-48472` — the parameter ladder.
    let flag = p.owner == 255;
    let num = 4.0f32;
    let num2 = 500.0f32;
    let num3 = num2 / 2.0;
    // `:48473-48494` — the reel counter (local owner, settled extra
    // updates only). The channel reels IN (−5), the use button with no
    // item animation reels OUT (+5); the 4..500 clamp; the length
    // change flags the net sync.
    if local && p.extra_updates == 0 {
        let num4 = p.ai[0];
        if p.ai[0] == 0.0 {
            p.ai[0] = num3;
        }
        let mut num5 = p.ai[0];
        if pl.channel {
            num5 -= 5.0;
        }
        if ctx.control_use_item && ctx.item_animation == 0 {
            num5 += 5.0;
        }
        p.ai[0] = num5.clamp(num, num2);
        if num4 != p.ai[0] {
            p.net_update = true;
        }
    }
    // `:48495-48511` — the length-change cue latch: the first sight
    // pins localAI[2] silently; every later change re-arms the 30-tick
    // cooldown and plays Item192 at the kite pitch (the static
    // `Projectile.kiteSoundPitch` — the port rides the pitchOffset,
    // the module doc). The owner-255 flag refuses the play.
    if p.local_ai[2] == 0.0 {
        p.local_ai[2] = p.ai[0];
    } else if p.local_ai[2] != p.ai[0] {
        p.local_ai[2] = p.ai[0];
        if p.sound_delay == 0 {
            p.sound_delay = 30;
            let pitch = (p.local_ai[2] - num) / (num2 - num) * -0.12 + 0.06;
            if !flag {
                p.wants_sound = Some((2, 192, pl.x, pl.y, pitch));
            }
        }
    }
    // `:48512-48515` — the per-sub-step cooldown tick (the pool's
    // once-per-update decrement at `:16768` runs IN ADDITION — the C#
    // has both).
    if p.sound_delay > 0 {
        p.sound_delay -= 1;
    }
    // `:48516-48519` — the first tick's 60th sub-step settles the
    // extra updates; every later tick is a single sub-step.
    if num_updates == 1 {
        p.extra_updates = 0;
    }
    // `:48520-48529` — the wind read + the 300-cap wind counter.
    let mut num6 = 0.0f32;
    if in_a_place_with_wind(world, p.pos, p.w, p.h) {
        num6 = wind_visual;
    }
    let value = num6.abs() >= 0.2;
    p.local_ai[0] = (p.local_ai[0] - if value { 1.0 } else { -1.0 }).clamp(0.0, 300.0);
    // `:48529-48541` — the hover target: the wind X + the global-time
    // sine bob (the cloud lift), the storm squash, the 3+7·cloud pull.
    let num7 = get_lerp_value(0.2, 0.5, num6.abs()) * 0.5;
    let center0 = Vec2::from(p.center());
    // BobOffset is 0 (`:48454`); `Math.Sin` widens the float sum to double.
    let bob = (global_time as f64).sin() as f32;
    let mut vector = Vec2::new(num6, bob + cloud_alpha * 5.0) * 25.0;
    if cloud_alpha > 0.0 && num6.abs() >= cloud_alpha / 2.0 && vector.y > 0.0 {
        vector.y *= 1.0 - num6.abs() * 2.0;
    }
    let center = center0 + vector;
    let mut value2 = safe_normalize(center - center0) * (3.0 + cloud_alpha * 7.0);
    if num7 == 0.0 {
        value2 = Vec2::ZERO;
    }
    // `:48542-48546` — the unsettled sub-steps rewind the move and
    // return (KiteLogic's return — AI_160's buff gate still runs).
    if p.extra_updates > 0 {
        p.pos -= p.vel;
        return;
    }
    // `:48547-48565` — the hover steer: the 5..10 eased blend toward
    // the wind pull (the Y restore keeps the gravity lane separate),
    // the lift/gravity pair, the ±2 clamp, the below-target catch-up.
    let num8 = center0.distance(center);
    let lerp_value = get_lerp_value(5.0, 10.0, num8);
    let y = p.vel.y;
    if num8 > 10.0 {
        p.vel = Vec2::lerp(p.vel, value2, 0.075 * lerp_value);
    }
    p.vel.y = y;
    p.vel.y -= num7;
    p.vel.y += 0.02 + num7 * 0.25;
    p.vel.y = p.vel.y.clamp(-2.0, 2.0);
    if center0.y + p.vel.y < center.y {
        // `Lerp(vy, vy + num7 + 0.01, 0.75)` — the below-target catch-up.
        p.vel.y = p.vel.y + (p.vel.y + num7 + 0.01 - p.vel.y) * 0.75;
    }
    if p.vel.x.abs() > 0.02 {
        p.vel.x *= 0.98;
    }
    // `:48566-48602` — the anchor reel: past the string length the
    // kite snaps the overflow toward the arm (through blocks — the
    // KiteFlyingInfo flag), the sway adds, and the fully-reeled local
    // kite dies (the afk resets are the module-doc skip).
    let num9 = center0.distance(anchor);
    let num10 = p.ai[0];
    let flag2 = num9 > num10;
    let flag3 = num9 > num10 - 2.0;
    if flag2 {
        let vector2 = (anchor - center0).normalize();
        let vector3 = vector2 * (num9 - num10);
        // CanReelThroughBlocks true — the centre snap, no collision.
        let c = center0 + vector3;
        p.pos = c - Vec2::new(p.w * 0.5, p.h * 0.5);
        let mut flag4 = vector2.y < 0.8 || num7 > 0.0;
        if num7 == 0.0 && p.vel.x.abs() < 0.05 {
            flag4 = false;
        }
        p.vel.y += vector2.y * 0.05;
        if flag4 {
            p.vel.y -= 0.15;
        }
        p.vel.x += vector2.x * 0.2;
        if num10 == num && local {
            kill(p);
            return;
        }
    }
    // `:48603-48609` — the life pin + the facing turn (the wide-side
    // gate; `ChangeDir`'s pulley arm is unreachable — the module doc).
    p.time_left = 2;
    let vector5 = Vec2::from(p.center()) - anchor;
    let dir = if vector5.x > 0.0 { 1 } else { -1 };
    if vector5.x.abs() > vector5.y.abs() / 2.0 && dir != pl.facing {
        pl.facing = dir;
    }
    // `:48610-48624` — the rotation: the calm-rest damp vs the
    // anchor-facing + π/4 (the left-facing −π/2·direction flip) + the
    // velocity-X lean.
    let vector6 = safe_normalize(anchor - Vec2::from(p.center()));
    let flag5 = num7 == 0.0 && p.vel.y >= 0.02 && !flag3;
    if flag5 {
        p.rot *= 0.95;
    } else {
        let mut num11 = (-vector6).y.atan2((-vector6).x) + std::f32::consts::FRAC_PI_4;
        if p.sprite_direction == -1 {
            num11 -= std::f32::consts::FRAC_PI_2 * pl.facing as f32;
        }
        p.rot = num11 + p.vel.x * 0.05;
    }
    // `:48625` — the rest accumulator (the buff gate's < 1 read).
    p.local_ai[1] = (p.local_ai[1] + if flag5 { 1.0 } else { -1.0 } * 0.005).clamp(0.0, 1.0);
    // `:48626-48691` — the per-kind frame machine. The 824/839/840/853
    // wind-flap is the machine's ONLY RNG draw (`Next(4)`, `:48670`).
    let num12 = p.vel.length();
    match p.kind {
        771 | 822 | 823 | 827 | 830 | 838 | 843 | 844 | 845 | 846 | 850 | 852 => {
            p.frame = 0;
        }
        828 | 829 => {
            p.frame = if num12 < 6.0 { 1 } else { 0 };
        }
        826 => {
            p.frame = p.ai[1] as i32;
        }
        824 | 839 | 840 | 853 => {
            if num7 < 0.15 {
                p.frame = 0;
            } else {
                p.frame_counter += 1;
                if p.frame_counter as f32 > (1.0 - num7) * 10.0 {
                    p.frame_counter = 0;
                    p.frame = rng.next_range(0, 4);
                }
            }
        }
        _ => {
            p.frame = if num12 < 3.0 {
                0
            } else if num12 < 5.0 {
                1
            } else if num12 < 7.0 {
                2
            } else {
                3
            };
        }
    }
    // `:48692` — the sprite facing follows the owner (post-ChangeDir).
    p.sprite_direction = pl.facing;
}

/// The kite's `HandleMovement` half (`Projectile.cs:17469-17890` +
/// `UpdatePosition` `:20433-20446`): the resized collision box
/// (`GetCollisionParams`), the `TileCollision` slide (fallThrough true
/// — kites are not pets), the wet velocity scale (the blocked axes
/// keep their full value, `:17642-17655`), the slope follow-up, then
/// the wet/dry integrate. The blocked-move chain EXCLUDES aiStyle 160
/// (`:18398`) — no bounce, no kill. The wet factor is the centre-cell
/// liquid stand-in (the `flail_wet_factor` idiom, ANALYSIS).
fn kite_move(p: &mut Projectile, world: &World) {
    let last = p.vel;
    let (col_w, col_h, anchor) = collision_params(p.kind, p.w, p.h);
    let center = Vec2::from(p.center());
    let vector = center - Vec2::new(col_w, col_h) * anchor;
    let mut wet_vel = p.vel;
    if p.tile_collide {
        let tc = crate::collision::tile_collision(
            world,
            [vector.x, vector.y],
            [p.vel.x, p.vel.y],
            col_w,
            col_h,
            true,
            true,
        );
        p.vel = Vec2::new(tc.vel[0], tc.vel[1]);
        if p.wet {
            let cx = (p.pos.x + p.w * 0.5) / 16.0;
            let cy = (p.pos.y + p.h * 0.5) / 16.0;
            let f = world
                .tile(cx as i32, cy as i32)
                .filter(|t| t.liquid > 0)
                .map(|t| match t.liquid_kind() {
                    tr_core::tile::LiquidKind::Shimmer => 0.375,
                    tr_core::tile::LiquidKind::Honey => 0.25,
                    _ => 0.5,
                })
                .unwrap_or(0.5);
            wet_vel = p.vel * f;
            if p.vel.x != last.x {
                wet_vel.x = p.vel.x;
            }
            if p.vel.y != last.y {
                wet_vel.y = p.vel.y;
            }
        }
        // The slope follow-up (`:17656-17699`, correctSlopeCollision
        // false): the box slides the ramps, the position keeps its
        // offset from the collision anchor.
        let sc = crate::collision::slope_collision(
            world,
            [vector.x, vector.y],
            [p.vel.x, p.vel.y],
            col_w,
            col_h,
            true,
        );
        p.pos = Vec2::new(sc.pos[0], sc.pos[1]) + (p.pos - vector);
        p.vel = Vec2::new(sc.vel[0], sc.vel[1]);
    }
    // `UpdatePosition` (`:20439-20446`): wet integrates the scaled
    // velocity, dry the full velocity.
    p.pos += if p.wet { wet_vel } else { p.vel };
}

#[cfg(test)]
mod w4_d16_160_tests {
    //! W4-d16 — aiStyle 160 (`AI_160_Kites` + `KiteLogic`,
    //! `Projectile.cs:48421-48693`, the dispatch `:36603-36606`, the
    //! `DefaultToKite` row `:10744-10751`). WRITTEN NOT RUN (code-only
    //! mode). The row, the kill gates, the reel counter + the Item192
    //! pitch latch, the sub-step settle + rewind, the frame machine's
    //! `Next(4)` census, the buff gate, the dispatch wiring.
    use super::*;
    use crate::npc::Npc;
    use crate::projectile::{FlailInputs, ProjectilePool, YoyoInputs, proj_data, proj_frames};

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
        // The player spawns high (row 20) so the hovering kite sits
        // ABOVE worldSurface (row 50) — the wind-blown height gate
        // (`DoesWindBlowAtThisHeight`) the flap census needs.
        let mut w = World::new("t".into(), 300, 200);
        w.spawn_y = 20;
        w
    }

    /// A settled kite (extraUpdates 0, the string at the 250 default)
    /// hovering 100 px above the player's centre.
    fn kite(w: &World, pl: &crate::player::Player) -> Projectile {
        let d = proj_data(766).unwrap();
        let mut p = Projectile::default();
        p.active = true;
        p.kind = 766;
        p.owner = 0;
        p.w = d.w;
        p.h = d.h;
        p.time_left = d.time_left;
        p.penetrate = d.penetrate;
        p.max_penetrate = d.penetrate;
        p.tile_collide = d.tile_collide;
        p.extra_updates = 0;
        p.ai[0] = 250.0;
        p.local_ai[2] = 250.0;
        let (pcx, pcy) = pl.center();
        p.pos = Vec2::new(pcx - p.w * 0.5, pcy - 100.0 - p.h * 0.5);
        p
    }

    fn ctx() -> KiteContext {
        KiteContext { selected_shoot: 766, item_animation: 0, control_use_item: false }
    }

    /// The row (`DefaultToKite`, `Projectile.cs:10744-10751`).
    #[test]
    fn w4_d16_160_the_row_matches_default_to_kite() {
        for &kind in KITE_KINDS.iter() {
            let d = proj_data(kind).unwrap();
            assert_eq!((d.w, d.h, d.ai_style), (4.0, 4.0, 160), "kind {kind}");
            assert_eq!(d.penetrate, -1);
            assert_eq!(d.extra_updates, 60, "the first tick's 61 sub-steps");
            assert!(d.tile_collide);
            assert_eq!(d.time_left, 3600);
        }
        // The sheets (`Main.cs:8632-8658`).
        assert_eq!(proj_frames(766), 4);
        assert_eq!(proj_frames(826), 3);
        assert_eq!(proj_frames(828), 2);
        assert_eq!(proj_frames(853), 4);
        assert_eq!(proj_frames(771), 1, "the wyvern keeps the stock 1");
        // The 822+ netImportant (`:8042` etc.); the 766-771 keep false.
        assert!(proj_data(822).unwrap().net_important);
        assert!(!proj_data(766).unwrap().net_important);
    }

    /// The kill gates (`:48426-48450`): a mismatched held shoot, the
    /// pulley, the death and the 2000-px leash each kill; the heldProj
    /// pin + the timeLeft refresh run ahead of them.
    #[test]
    fn w4_d16_160_the_kill_gates() {
        let w = world();
        let mut pl = crate::player::Player::spawn(&w);
        let mut p = kite(&w, &pl);
        // The held-shoot mismatch.
        let mut c = ctx();
        c.selected_shoot = 767;
        kite_step(&mut p, &w, Some(&mut pl), &c, (0.0, 0.0, 0.0, 0.0), &mut census(), 0, 3, 3);
        assert!(!p.active, "the shoot mismatch kills");
        assert_eq!(pl.held_proj, 3, "the heldProj pin precedes the gate");
        // The pulley.
        let mut p2 = kite(&w, &pl);
        pl.pulley = true;
        kite_step(&mut p2, &w, Some(&mut pl), &ctx(), (0.0, 0.0, 0.0, 0.0), &mut census(), 0, 3, 3);
        assert!(!p2.active, "the pulley kills");
        pl.pulley = false;
        // The death.
        let mut p3 = kite(&w, &pl);
        pl.dead = true;
        kite_step(&mut p3, &w, Some(&mut pl), &ctx(), (0.0, 0.0, 0.0, 0.0), &mut census(), 0, 3, 3);
        assert!(!p3.active, "the death kills");
        pl.dead = false;
        // The 2000-px leash.
        let mut p4 = kite(&w, &pl);
        p4.pos.x += 3000.0;
        kite_step(&mut p4, &w, Some(&mut pl), &ctx(), (0.0, 0.0, 0.0, 0.0), &mut census(), 0, 3, 3);
        assert!(!p4.active, "the leash kills");
        // The unresolved owner folds the generic kill (`:16782`).
        let mut p5 = kite(&w, &pl);
        kite_step(&mut p5, &w, None, &ctx(), (0.0, 0.0, 0.0, 0.0), &mut census(), 0, 3, 3);
        assert!(!p5.active, "the orphan kill");
    }

    /// The reel counter (`:48473-48494`): the channel reels in −5, the
    /// use button (no item animation) reels out +5, the 4..500 clamp;
    /// the length change re-arms the 30-tick cooldown and queues Item192
    /// at the kite pitch (`:48502-48510`).
    #[test]
    fn w4_d16_160_the_reel_counter_and_cue() {
        let w = world();
        let mut pl = crate::player::Player::spawn(&w);
        let mut p = kite(&w, &pl);
        pl.channel = true;
        let mut rng = census();
        kite_step(&mut p, &w, Some(&mut pl), &ctx(), (0.0, 0.0, 0.0, 0.0), &mut rng, 0, 3, 3);
        assert_eq!(p.ai[0], 245.0, "the channel reels in −5");
        assert_eq!(p.local_ai[2], 245.0, "the length latch follows");
        // The cue: soundDelay was 0 → 30, then the per-sub-step tick → 29.
        assert_eq!(p.sound_delay, 29);
        let (id, style, _, _, pitch) = p.wants_sound.expect("the Item192 cue");
        assert_eq!((id, style), (2, 192));
        let want = (245.0 - 4.0) / (500.0 - 4.0) * -0.12 + 0.06;
        assert!((pitch - want).abs() < 1e-6, "the kite pitch: {pitch} vs {want}");
        assert!(p.net_update, "the length change flags the sync");
        assert_eq!(rng.ranges, Vec::new(), "the reel draws nothing");
        // The use button reels out (the item animation must be 0).
        pl.channel = false;
        let mut c = ctx();
        c.control_use_item = true;
        let mut p2 = kite(&w, &pl);
        kite_step(&mut p2, &w, Some(&mut pl), &c, (0.0, 0.0, 0.0, 0.0), &mut census(), 0, 3, 3);
        assert_eq!(p2.ai[0], 255.0, "the use button reels out +5");
        // The 4 floor: a fully-reeled string clamps.
        let mut p3 = kite(&w, &pl);
        p3.ai[0] = 6.0;
        p3.local_ai[2] = 6.0;
        pl.channel = true;
        kite_step(&mut p3, &w, Some(&mut pl), &ctx(), (0.0, 0.0, 0.0, 0.0), &mut census(), 0, 3, 3);
        assert_eq!(p3.ai[0], 4.0, "the 4 floor");
    }

    /// The first tick's sub-step contract (`:48516-48519`/`:48542-48546`):
    /// the numUpdates==1 sub-step settles extraUpdates to 0; the
    /// unsettled sub-steps rewind the move (`position -= velocity`) and
    /// skip the hover/reel body.
    #[test]
    fn w4_d16_160_the_substep_settle_and_rewind() {
        let w = world();
        let mut pl = crate::player::Player::spawn(&w);
        let mut p = kite(&w, &pl);
        p.extra_updates = 60;
        p.vel = Vec2::new(2.0, -1.0);
        let pos0 = p.pos;
        // numUpdates 59 (the FIRST sub-step): the rewind, no settle.
        kite_step(&mut p, &w, Some(&mut pl), &ctx(), (0.0, 0.0, 0.0, 0.0), &mut census(), 0, 59, 3);
        assert_eq!(p.extra_updates, 60, "only numUpdates==1 settles");
        assert_eq!(p.pos, pos0 - p.vel, "the rewind");
        assert_eq!(p.ai[0], 250.0, "the reel counter is gated on extraUpdates==0");
        // numUpdates 1: the settle.
        let mut p2 = kite(&w, &pl);
        p2.extra_updates = 60;
        kite_step(&mut p2, &w, Some(&mut pl), &ctx(), (0.0, 0.0, 0.0, 0.0), &mut census(), 0, 1, 3);
        assert_eq!(p2.extra_updates, 0, "the numUpdates==1 settle");
    }

    /// The frame machine's ONLY RNG draw (`:48657-48672`): the
    /// 824/839/840/853 wind-flap rolls `Next(4)` when the wind lift
    /// num7 ≥ 0.15 and the counter passes the (1−num7)·10 threshold.
    #[test]
    fn w4_d16_160_the_wind_flap_frame_census() {
        let w = world();
        let mut pl = crate::player::Player::spawn(&w);
        let mut p = kite(&w, &pl);
        p.kind = 824;
        // Wind 0.5 → num7 = GetLerpValue(0.2,0.5,0.5)*0.5 = 0.5; the
        // threshold (1−0.5)*10 = 5 — the counter must exceed 5.
        p.frame_counter = 5;
        let mut rng = census();
        // The kite must be in a wind-blown place: above worldSurface
        // (the test world's surface is height*0.25 = 50 rows = 800 px;
        // the kite hovers at pcy−100, well above).
        kite_step(&mut p, &w, Some(&mut pl), &ctx(), (0.5, 0.0, 0.0, 0.0), &mut rng, 0, 3, 3);
        assert_eq!(rng.ranges, vec![(0, 4)], "the Next(4) frame roll");
        assert_eq!(p.frame, 0, "census min 0");
        assert_eq!(p.frame_counter, 0, "the wrap resets the counter");
        // The calm wind (num7 0) pins frame 0 with NO draw.
        let mut p2 = kite(&w, &pl);
        p2.kind = 824;
        let mut rng2 = census();
        kite_step(&mut p2, &w, Some(&mut pl), &ctx(), (0.0, 0.0, 0.0, 0.0), &mut rng2, 0, 3, 3);
        assert_eq!(p2.frame, 0);
        assert_eq!(rng2.ranges, Vec::new(), "the calm flap draws nothing");
        // The default ladder: speed < 3 → frame 0 (the 766 kite).
        let mut p3 = kite(&w, &pl);
        p3.vel = Vec2::new(4.0, 0.0); // speed 4 → frame 1
        kite_step(&mut p3, &w, Some(&mut pl), &ctx(), (0.0, 0.0, 0.0, 0.0), &mut census(), 0, 3, 3);
        assert_eq!(p3.frame, 1, "the 3..5 speed band");
    }

    /// The kite-luck buff gate (`:48460-48463`): the local owner's kite
    /// above the player's top with localAI[1] < 1 grants buff 383 for
    /// 12 ticks.
    #[test]
    fn w4_d16_160_the_luck_buff_gate() {
        let w = world();
        let mut pl = crate::player::Player::spawn(&w);
        let mut p = kite(&w, &pl);
        p.local_ai[1] = 0.0;
        kite_step(&mut p, &w, Some(&mut pl), &ctx(), (0.0, 0.0, 0.0, 0.0), &mut census(), 0, 3, 3);
        assert!(pl.buffs.find_buff_index(383).is_some(), "the kite luck buff");
        // The foreign owner's kite grants nothing.
        let mut pl2 = crate::player::Player::spawn(&w);
        let mut p2 = kite(&w, &pl2);
        p2.owner = 1;
        kite_step(&mut p2, &w, Some(&mut pl2), &ctx(), (0.0, 0.0, 0.0, 0.0), &mut census(), 0, 3, 3);
        assert!(pl2.buffs.find_buff_index(383).is_none(), "the owner gate");
    }

    /// The dispatch wiring: a spawned kite with the context fed runs the
    /// machine (the heldProj pin + the life refresh); without the
    /// context the arm is inert (the pool-only no-op).
    #[test]
    fn w4_d16_160_the_dispatch_arms_the_kite() {
        let mut w = world();
        let mut pl = crate::player::Player::spawn(&w);
        let mut pool = ProjectilePool::new();
        let mut npcs: Vec<Npc> = Vec::new();
        let (pcx, pcy) = pl.center();
        let slot = pool
            .spawn_with_data_ai(766, pcx - 2.0, pcy - 100.0, 0.0, 0.0, 0, 0.0, 0.0, 0.0)
            .unwrap();
        pool.set_kite_context(Some(ctx()));
        let mut rng = census();
        pool.update(
            &mut w, &mut npcs, None, None, Some(&mut pl),
            YoyoInputs::default(), FlailInputs::default(), &mut rng, 0, false,
        );
        // The first tick's 61 sub-steps settle extraUpdates to 0; the
        // kite survives (the held shoot matches) and pins heldProj.
        assert!(pool.slots()[slot].active, "the kite survives");
        assert_eq!(pool.slots()[slot].extra_updates, 0, "the settle");
        assert_eq!(pl.held_proj, slot as i32, "the heldProj pin");
        // The reel counter ran on the settled sub-steps: ai[0] 0 → 250.
        assert_eq!(pool.slots()[slot].ai[0], 250.0, "the 250 default string");
    }
}