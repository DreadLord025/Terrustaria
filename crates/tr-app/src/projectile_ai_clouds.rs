//! W4-d19 follow-up — aiStyle 92, the drifting cloud family: the inline
//! `AI()` arm `Projectile.cs:32558-32820`. The style's FIVE `SetDefaults`
//! rows (the whole census): 511/512/513 ToxicCloud (`:5805-5840`, the
//! Poison Staff's magic clouds, `UseGroupStaticImmunity(511)`), 1007
//! GasTrap (`:9558-9569`, the dart trap's gas, hostile + friendly + trap +
//! hide) and 1128 ChlorophyteClaymoreSporeCloud (`:10565-10579`, the
//! Chlorophyte Claymore's spore child — the W4-d19 aiStyle-190 spore
//! machine's spawn payload, `projectile_ai_sword_styles::ClaymoreSpores`).
//!
//! C# sources read WHOLE (the golden rule):
//! - the arm `Projectile.cs:32558-32820` (the three kind flags, the
//!   lifetime caps, the per-kind scale/opacity curves, the mutual
//!   repulsion scans, the wall-bounce ladder, the 1007 gas buff);
//! - `Collision.IsWorldPointSolid` `Collision.cs:1434-1491` (the bounce
//!   probe — ported as `collision::is_world_point_solid`);
//! - `ApplyBuffTo` `Projectile.cs:12391-12398` (the trap + deadMansSweater
//!   halving — the sweater has no port field, the fold is documented);
//! - `Main.projFrames[1128] = 5` (`Main.cs:8726`) — the first-tick frame
//!   roll's bound.
//!
//! RNG contract (the `UpdateProjectiles` stream): the arm's ONLY explicit
//! draw is the 1128 first-tick `frame = Main.rand.Next(projFrames[type])`
//! (`:32565`, one `Next(5)` while `ai[0] == 0`). The 1007 first-tick
//! `ParticleOrchestrator.BroadcastParticleSpawn` (`:32574-32578`) is the
//! particle layer's skip — no explicit call-site draw. ZERO other draws:
//! the repulsion scans, the bounce ladder and the gas buff are rand-free.
//!
//! Movement: the arm keeps `tileCollide = false` (`:32567`), so the
//! dispatch's post-arm `kill_step` is the free `position += velocity`
//! (`UpdatePosition` `:20445`; none of the five kinds sits in the `:20435`
//! skip list). `HandleMovement` runs even on the lifetime-kill tick
//! (`:17015` before the `!active` return `:17027`).
//!
//! A POOL-side step (the `dragon_step` shape): the two repulsion scans
//! (`:32638-32705`) read AND write the OTHER cloud slots' velocities, so
//! the arm takes the whole slot slice plus its own index (the C# `whoAmI`
//! — the rotation seed `:32625` and the zero-delta tie-break `:32655`).
//!
//! KEEP-ONE (research/26g-clouds-keep-one.md): W4-d10's `b34ad73c` ported
//! the same arm into `projectile_ai_91_100.rs::ai_clouds`. This module is
//! the surviving canonical copy (the faithful superset — the shared
//! `collision::is_world_point_solid`, the 190 spore drain, the ungated
//! gas player lane); the d10-only hunk folded here is the GasTrap
//! broadcast's Item16 cue (`ParticleOrchestrator.cs:2913`). d10's private
//! `world_point_solid` is superseded by `collision.rs` (its `(x/16) as
//! i32` truncates toward zero vs the C# `ToTileCoordinates` floor,
//! `Utils.cs:1807-1810` — unreachable under the fluff-1 gate, but the
//! shared copy is structurally verbatim).
//!
//! Documented deviations (also `docs/ANALYSIS.md` "W4-d19 spore children"):
//! * the 1007 `hide = true` (`:9568`) has no port field (the draw seam);
//! * the 1007 GasTrap particle broadcast is the particle layer's skip —
//!   its `Spawn_GasTrap` body (`ParticleOrchestrator.cs:2911-2953`, the
//!   12+10 particles' NextFloat/RotatedByRandom/Next draws) goes with the
//!   unported particle pool (the dust-body convention); its Item16 cue
//!   (`:2913`) queues through `wants_sound` (the W4-d10 keep-one fold);
//! * `ApplyBuffTo`'s `trap && deadMansSweater` halving (`:12393-12396`):
//!   the 1007 IS a trap, but `deadMansSweater` has no port field — the
//!   halving folds out (the 949 self-hurt precedent);
//! * the bounce ladder's fallback `Normalize()` calls (`:32724` etc.) use
//!   the house safe-normalize (the C# XNA Normalize NaNs on a zero delta;
//!   the yoyo dropped-twin precedent) — the zero delta is reachable only
//!   in the second ladder's ±16 probe when the first ladder's mirror
//!   probes were all solid;
//! * the `Main.netMode` gates fold to single-player (netMode 0): the
//!   particle broadcast runs (skipped), the gas player buff runs, the NPC
//!   scan runs;
//! * the 1129 blade-slam child (aiStyle 209) is W4-d21's claim — its row +
//!   `ai_claymore_blade_slam` arm landed on `task/W4-d21-proj-ai-201-210`
//!   (verified verbatim against `AI_209_ChlorophyteClaymoreBladeSlam`
//!   `:69622-69631`). The 190 drain spawns it through the data seam, which
//!   no-ops until that row merges (the documented cross-branch contract).
//!
//! Code-only round (W4-d19): the witness tests below are WRITTEN NOT RUN.

use glam::Vec2;
use tr_core::World;
use tr_core::loot::DropRng;

use crate::npc::Npc;
use crate::projectile::{Projectile, proj_data, proj_frames};

/// `Utils.Remap` (`Utils.cs:345-348`) — the clamped lerp-of-lerp (the
/// pool's helper, W4-d10 keep-one: no private twin).
fn remap(v: f32, from_min: f32, from_max: f32, to_min: f32, to_max: f32) -> f32 {
    crate::projectile::remap(v, from_min, from_max, to_min, to_max)
}

/// `Projectile.Opacity`'s setter (`Projectile.cs:546-549`).
fn set_opacity(p: &mut Projectile, opacity: f32) {
    p.alpha = ((1.0 - opacity) * 255.0).clamp(0.0, 255.0) as i32;
}

/// `Entity.Center` (`Entity.cs`).
fn center_of(p: &Projectile) -> Vec2 {
    Vec2::new(p.pos.x + p.w * 0.5, p.pos.y + p.h * 0.5)
}

/// `Entity.Center`'s setter.
fn set_center(p: &mut Projectile, c: Vec2) {
    p.pos = c - Vec2::new(p.w * 0.5, p.h * 0.5);
}

/// `Vector2.SafeNormalize(fallback)` (`Utils.cs:1837-1846`).
fn safe_normalize(v: Vec2, fallback: Vec2) -> Vec2 {
    if v == Vec2::ZERO || v.is_nan() { fallback } else { v.normalize() }
}

/// XNA `Rectangle.Intersects` — strict overlap on the integer hitbox.
fn hitbox_intersects(a: &Projectile, b: &Projectile) -> bool {
    let (ax, ay, aw, ah) = (a.pos.x as i32, a.pos.y as i32, a.w as i32, a.h as i32);
    let (bx, by, bw, bh) = (b.pos.x as i32, b.pos.y as i32, b.w as i32, b.h as i32);
    ax < bx + bw && bx < ax + aw && ay < by + bh && by < ay + ah
}

/// `Kill()`'s state core (`Projectile.cs:70644-70650`) — the death-hook
/// visuals are the pool-wide kill-dust skip (the sword-styles convention).
fn kill(p: &mut Projectile) {
    p.kill_time_left = p.time_left;
    p.active = false;
    p.time_left = 0;
}

/// `AI()`'s aiStyle-92 arm (`Projectile.cs:32558-32820`) — the drifting
/// cloud family. `i` is the C# `whoAmI` (the rotation seed + the repulsion
/// tie-break). The 1007 gas buff needs the local player + the NPC vec; the
/// repulsion scans need the whole slot slice.
pub(crate) fn cloud_family_step(
    slots: &mut [Projectile],
    i: usize,
    world: &World,
    player: Option<&mut crate::player::Player>,
    npcs: &mut [Npc],
    rng: &mut dyn DropRng,
) {
    let kind = slots[i].kind;
    let flag44 = kind == 1007;
    let flag45 = (511..=513).contains(&kind);
    let flag46 = kind == 1128;
    // `:32563-32566` — the 1128 first-tick frame roll: the ONLY draw.
    if flag46 && slots[i].ai[0] == 0.0 {
        slots[i].frame = rng.next_range(0, proj_frames(kind) as i32);
    }
    // `:32567` — the clouds never collide with tiles.
    slots[i].tile_collide = false;
    // `:32568-32579` — the 1007 first-tick facing + particle broadcast
    // (the `netMode != 1` arm folds open in single-player). The C# gate
    // is `localAI[0] == 0` ONLY — the direction fix is the INNER `if`
    // (the stock spawn direction 1 must not suppress the broadcast).
    // The orchestrator body is the particle layer's skip; its Item16
    // cue (`ParticleOrchestrator.cs:2913`, at `PositionInWorld = Center`)
    // queues through `wants_sound` (the W4-d10 keep-one fold).
    if flag44 && slots[i].local_ai[0] == 0.0 {
        if slots[i].direction == 0 {
            slots[i].direction = 1;
        }
        let (cx, cy) = slots[i].center();
        slots[i].wants_sound = Some((2, 16, cx, cy, 0.0, 1.0));
    }
    // `:32580-32584` — the age; a hit cloud (ai[1] >= 1) ages triple.
    slots[i].ai[0] += 1.0;
    if slots[i].ai[1] >= 1.0 {
        slots[i].ai[0] += 2.0;
    }
    // `:32585-32593` — the lifetime cap: 260 stock, 80 for the gas, the
    // 1128's ai[1] seed (220 when the bloom passed none).
    let mut num763 = 260.0f32;
    if flag44 {
        num763 = 80.0;
    }
    if flag46 {
        num763 = if slots[i].ai[1] == 0.0 { 220.0 } else { slots[i].ai[1] };
    }
    // `:32594-32599` — the kill pins ai[0] AT the cap and returns (the
    // post-arm kill_step still integrates, `:17015`). The 1007 death has
    // no Kill() dust arm (the census — the gas just expires).
    if slots[i].ai[0] > num763 {
        kill(&mut slots[i]);
        slots[i].ai[0] = num763;
        return;
    }
    let from_value2 = slots[i].ai[0] / num763;
    cloud_scale_and_opacity(&mut slots[i], flag44, flag46, from_value2);
    // `:32624-32625` — the spin: the whoAmI seed + the direction accrual.
    slots[i].local_ai[0] += slots[i].direction as f32;
    slots[i].rot = i as f32 * 0.4002029
        + slots[i].local_ai[0] * (std::f32::consts::PI * 2.0) / 480.0;
    // `:32626-32637` — the per-kind drag.
    if flag44 {
        slots[i].vel = Vec2::ZERO;
    } else if flag46 {
        slots[i].vel *= 0.97;
    } else {
        slots[i].vel *= 0.96;
    }
    // `:32638-32705` — the mutual repulsion scans.
    if flag45 {
        toxic_cloud_repulsion(slots, i);
    }
    if flag46 {
        spore_repulsion(slots, i);
    }
    // `:32706-32783` — the wall probe + the bounce.
    cloud_wall_bounce(&mut slots[i], world, flag44, flag46);
    // `:32784-32787` — only the gas continues.
    if !flag44 {
        return;
    }
    gas_trap_buff(&mut slots[i], player, npcs, from_value2);
}

/// The arm's per-kind scale/opacity half (`Projectile.cs:32600-32623`):
/// the 1007 grows 1→6 over 95% of its life with the clamped double-remap
/// fade; the 1128 blooms 0.25→1.1 over the first 10% then shrinks its box
/// with the scale and floats up (`velocity.Y -= 0.01`); the toxic clouds
/// only fade.
fn cloud_scale_and_opacity(p: &mut Projectile, flag44: bool, flag46: bool, fv: f32) {
    if flag44 {
        // `:32603-32608`.
        p.scale = remap(fv, 0.0, 0.95, 1.0, 6.0);
        let c = center_of(p);
        p.w = (50.0 * p.scale) as i32 as f32;
        p.h = p.w;
        set_center(p, c);
        let o = (remap(fv, 0.0, 0.25, 0.0, 1.0) * remap(fv, 0.75, 1.0, 1.0, 0.0))
            .clamp(0.0, 1.0)
            * 0.85;
        set_opacity(p, o);
    } else if flag46 {
        // `:32612-32618`.
        p.scale = remap(fv, 0.0, 0.1, 0.25, 1.1);
        let c = center_of(p);
        p.w = (30.0 * p.scale / 1.1) as i32 as f32;
        p.h = p.w;
        set_center(p, c);
        let o = remap(fv, 0.0, 0.25, 0.0, 1.0) * remap(fv, 0.5, 1.0, 1.0, 0.0) * 0.75;
        set_opacity(p, o);
        p.vel.y -= 0.01;
    } else {
        // `:32622`.
        let o = remap(fv, 0.0, 0.3, 0.0, 1.0) * remap(fv, 0.3, 1.0, 1.0, 0.0) * 0.7;
        set_opacity(p, o);
    }
}

/// The 511-513 mutual repulsion (`Projectile.cs:32638-32670`): every
/// overlapping toxic cloud pair pushes apart along the centre delta —
/// `SafeNormalize(UnitX) * 0.005`, both velocities lerped 0.6 toward the
/// push (the C# writes each slot live, so a later pair reads the earlier
/// pair's result). The zero-delta tie-break leans on the index order.
fn toxic_cloud_repulsion(slots: &mut [Projectile], i: usize) {
    for j in 0..slots.len() {
        if j == i || !slots[j].active || !(511..=513).contains(&slots[j].kind) {
            continue;
        }
        if !hitbox_intersects(&slots[i], &slots[j]) {
            continue;
        }
        let mut delta = center_of(&slots[j]) - center_of(&slots[i]);
        if delta == Vec2::ZERO {
            delta = if j < i { Vec2::new(-1.0, 1.0) } else { Vec2::new(1.0, -1.0) };
        }
        let push = safe_normalize(delta, Vec2::new(1.0, 0.0)) * 0.005;
        let sv = slots[i].vel;
        slots[i].vel = sv.lerp(sv - push, 0.6);
        let ov = slots[j].vel;
        slots[j].vel = ov.lerp(ov + push, 0.6);
    }
}

/// The 1128 mutual repulsion (`Projectile.cs:32671-32705`): the same scan
/// with the overlap-depth push — `normalized * max(width − dist, 0) /
/// width * 0.005`, subtracted from self / added to the other (no lerp).
fn spore_repulsion(slots: &mut [Projectile], i: usize) {
    for j in 0..slots.len() {
        if j == i || !slots[j].active || slots[j].kind != 1128 {
            continue;
        }
        if !hitbox_intersects(&slots[i], &slots[j]) {
            continue;
        }
        let mut delta = center_of(&slots[j]) - center_of(&slots[i]);
        if delta == Vec2::ZERO {
            delta = if j < i { Vec2::new(-1.0, 1.0) } else { Vec2::new(1.0, -1.0) };
        }
        let dist = delta.length();
        let dir = safe_normalize(delta, Vec2::ZERO);
        let width = slots[i].w;
        let push = dir * ((width - dist).max(0.0) / width) * 0.005;
        slots[i].vel -= push;
        slots[j].vel += push;
    }
}

/// The wall probe + bounce (`Projectile.cs:32706-32783`): the point 16 px
/// along the normalized velocity; a solid hit (platforms non-solid) brakes
/// the toxic clouds with the 0.5 lerp, while the 1128 walks the reflection
/// ladder — the X-mirror first (when the flight is more horizontal), then
/// the ±16/±32 vertical probes, then the Y-mirror with its ±16 horizontal
/// probes, then the same 0.5 lerp fallback.
fn cloud_wall_bounce(p: &mut Projectile, world: &World, flag44: bool, flag46: bool) {
    let vector108 = safe_normalize(p.vel, Vec2::ZERO);
    let mut vector109 = center_of(p) + vector108 * 16.0;
    let solid = |pt: Vec2| crate::collision::is_world_point_solid(world, pt.x, pt.y, true);
    if flag44 || !solid(vector109) {
        return;
    }
    if !flag46 {
        // `:32781` — the toxic clouds' brake.
        p.vel = p.vel.lerp(p.vel - vector108 * 1.0, 0.5);
        return;
    }
    let speed = p.vel.length();
    let c = center_of(p);
    let mut flag47 = false;
    // `:32713-32749` — the X-mirror ladder (the horizontal flight).
    if vector108.x.abs() > vector108.y.abs() {
        let mut v110 = Vec2::new(vector108.x, -vector108.y);
        if !solid(c + v110 * 16.0) {
            p.vel = v110 * speed;
            flag47 = true;
        } else {
            for dy in [-16.0f32, 16.0, -32.0, 32.0] {
                let probe = vector109 + Vec2::new(0.0, dy);
                if !solid(probe) {
                    v110 = safe_normalize(probe - c, Vec2::ZERO);
                    p.vel = v110 * speed;
                    flag47 = true;
                    break;
                }
            }
        }
    }
    if flag47 {
        return;
    }
    // `:32750-32777` — the Y-mirror ladder.
    let v110 = Vec2::new(-vector108.x, vector108.y);
    vector109 = c + v110 * 16.0;
    if !solid(vector109) {
        p.vel = v110 * speed;
        return;
    }
    for dx in [-16.0f32, 16.0] {
        let probe = vector109 + Vec2::new(dx, 0.0);
        if !solid(probe) {
            p.vel = safe_normalize(probe - c, Vec2::ZERO) * speed;
            return;
        }
    }
    // `:32775` — the fallback brake.
    p.vel = p.vel.lerp(p.vel - vector108 * 1.0, 0.5);
}

/// The 1007 gas buff (`Projectile.cs:32788-32819`): the radius opens over
/// the first 20% of the 80-tick life to 180 px; every 15 ticks the local
/// player and every non-immune NPC inside it take Poisoned (20) for 2940
/// ticks. `ApplyBuffTo`'s trap + deadMansSweater halving folds out (the
/// sweater has no port field — module doc).
fn gas_trap_buff(
    p: &mut Projectile,
    player: Option<&mut crate::player::Player>,
    npcs: &mut [Npc],
    fv: f32,
) {
    let num767 = 20;
    let num768 = 2940;
    let num769 = remap(fv, 0.0, 0.2, 0.0, 1.0).clamp(0.0, 1.0) * 180.0;
    // `:32791-32799` — the 15-tick cadence.
    if p.local_ai[1] > 0.0 {
        p.local_ai[1] -= 1.0;
    }
    if !(p.local_ai[1] <= 0.0) {
        return;
    }
    p.local_ai[1] = 15.0;
    let c = center_of(p);
    // `:32800-32807` — the local player (netMode != 2 folds true).
    if let Some(pl) = player {
        if !pl.dead {
            let (pcx, pcy) = pl.center();
            if Vec2::new(pcx, pcy).distance(c) <= num769 {
                pl.buffs.add_buff(num767, num768);
            }
        }
    }
    // `:32812-32819` — the NPC scan (netMode == 1 folds false).
    for n in npcs.iter_mut() {
        if n.hp <= 0 || n.buffs.buff_immune[num767 as usize] {
            continue;
        }
        let (ncx, ncy) = n.center();
        if Vec2::new(ncx, ncy).distance(c) <= num769 {
            n.buffs.add_buff(num767, num768);
        }
    }
}

/// The 1128/1129 child drain for the aiStyle-190 spore machine
/// (`SpawnChlorophyteSporeCloud` `:41944`/`:41967`, the sweep spawn
/// `:42172`): the queued clouds spawn as 1128 with their lifetime seed in
/// ai[1]; the slam spawns as 1129 with ai1 = 128 / ai2 = 96. Both ride
/// `NewProjectile`'s centre convention (`:10876-10877` — the payload is a
/// CENTRE, the slot takes the top-left). The 1129 row + arm are W4-d21's
/// (module doc) — until they merge, the data seam refuses the slam and the
/// clouds still bloom.
pub(crate) fn drain_claymore_spores(
    pool: &mut crate::projectile::ProjectilePool,
    owner: u8,
    spores: crate::projectile_ai_sword_styles::ClaymoreSpores,
) {
    for (x, y, vx, vy, damage, ai1) in spores.clouds {
        let row = proj_data(1128).expect("the 1128 row (this module's census)");
        pool.spawn_with_data_ai_owned(
            1128,
            owner,
            x - row.w * 0.5,
            y - row.h * 0.5,
            vx,
            vy,
            damage,
            0.0,
            0.0,
            ai1,
        );
    }
    if let Some((x, y, damage, ai1, ai2)) = spores.slam {
        // BATCH-D: the 1129 row + the aiStyle-209 arm landed with d21 -
        // the slam spawn is LIVE (the 26g s5 cross-branch contract
        // composed: the centre offset reads the row, `NewProjectile`
        // `:10876-10877`).
        if let Some(row) = proj_data(1129) {
            if let Some(slot) = pool.spawn_with_data_ai_owned(
                1129,
                owner,
                x - row.w * 0.5,
                y - row.h * 0.5,
                0.0,
                0.0,
                damage,
                0.0,
                0.0,
                ai1,
            ) {
                pool.set_ai2(slot, ai2);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::npc::NpcKind;
    use crate::projectile::ProjectilePool;
    use tr_core::tile::{FLAG_ACTIVE, Tile};

    /// A draw-recording `Main.rand` stand-in (the sword-styles CensusRng
    /// shape): `next_range` returns `min`, `next_float` 0.
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

    /// One slot in a pool-sized slice (the repulsion scans walk the whole
    /// slice; the inactive defaults are skipped).
    fn slots_with(p: Projectile) -> Vec<Projectile> {
        let mut slots = vec![Projectile::default(); 8];
        slots[0] = p;
        slots
    }

    /// The five rows keep their `SetDefaults` slices (the data half).
    #[test]
    fn w4_d19_the_cloud_rows_are_verbatim() {
        for (k, w, h) in [(511u16, 32.0, 32.0), (512, 40.0, 38.0), (513, 30.0, 28.0)] {
            let d = proj_data(k).unwrap();
            assert_eq!(d.ai_style, 92, "kind {k} (`:5809`/`:5821`/`:5833`)");
            assert_eq!((d.w, d.h), (w, h), "kind {k}");
            assert_eq!(d.penetrate, -1);
            assert_eq!(d.alpha, 255);
            assert_eq!(d.scale, 1.1);
            assert!(d.magic, "kind {k} (`:5813`)");
            // `UseGroupStaticImmunity(511)` (`:10676-10681`).
            assert!(d.uses_id_static_immunity);
            assert_eq!(d.id_static_hit_cooldown, 10);
            assert_eq!(d.immunity_identity, 511);
        }
        let gas = proj_data(1007).unwrap();
        assert_eq!(gas.ai_style, 92, "`:9562`");
        assert_eq!((gas.w, gas.h), (50.0, 50.0));
        assert!(gas.trap, "`:9567`");
        assert_eq!(gas.scale, 1.1);
        assert_eq!(gas.penetrate, -1);
        let spore = proj_data(1128).unwrap();
        assert_eq!(spore.ai_style, 92, "`:10570`");
        assert_eq!((spore.w, spore.h), (30.0, 30.0));
        assert!(!spore.tile_collide, "`:10567`");
        assert!(spore.melee, "`:10572`");
        assert_eq!(spore.scale, 1.1);
        assert_eq!(spore.penetrate, -1);
        assert!(spore.uses_id_static_immunity);
        assert_eq!(spore.id_static_hit_cooldown, 30, "`:10577`");
        // The 1128 sheet slices 5 frames (`Main.cs:8726`).
        assert_eq!(proj_frames(1128), 5);
        // The 1129 row is W4-d21's claim: absent on this branch, aiStyle
        // 209 once it merges (the drain's documented contract — the
        // witness holds in BOTH states).
        if let Some(slam) = proj_data(1129) {
            assert_eq!(slam.ai_style, 209, "d21's row (`:10584`)");
        }
    }

    /// The 1128 first tick (`:32563-32566`, `:32580`, `:32612-32618`): ONE
    /// `Next(5)` frame roll, the age, the bloom scale and the float-up.
    #[test]
    fn w4_d19_the_spore_first_tick_rolls_the_frame_and_blooms() {
        let w = world();
        let mut slots = slots_with(proj(1128));
        slots[0].vel = Vec2::new(2.0, 0.0);
        let mut rng = census();
        let mut npcs: Vec<Npc> = Vec::new();
        cloud_family_step(&mut slots, 0, &w, None, &mut npcs, &mut rng);
        assert_eq!(rng.ranges, vec![(0, 5)], "the ONLY draw (`:32565`)");
        assert_eq!(rng.floats, 0);
        assert_eq!(slots[0].frame, 0, "CensusRng returns min");
        assert_eq!(slots[0].ai[0], 1.0);
        assert!(!slots[0].tile_collide);
        // scale = Remap(1/220, 0, 0.1, 0.25, 1.1) — the GetLerpValue
        // CLAMPS below the ramp: 1/220 ≈ 0.0045 < 0.1... the from-range
        // starts at 0, so the ramp is live; the d10 witness's "clamped
        // 0.25 start" pin re-derived: GetLerpValue(0, 0.1, 0.0045) =
        // 0.04545 → scale = 0.25 + 0.85·0.04545 ≈ 0.2886 (NOT 0.25 — d10's
        // assert was wrong, the C# re-derivation wins; the box is
        // (int)(30·0.2886/1.1) = 7).
        let fv = 1.0 / 220.0;
        let expected = remap(fv, 0.0, 0.1, 0.25, 1.1);
        assert!((slots[0].scale - expected).abs() < 1e-6);
        assert!((slots[0].scale - 0.28863636).abs() < 1e-5, "the live ramp");
        // The box shrinks with the scale: (int)(30·scale/1.1).
        assert_eq!(slots[0].w, (30.0 * expected / 1.1) as i32 as f32);
        assert_eq!(slots[0].h, slots[0].w);
        // velocity.Y -= 0.01 then ×0.97 (`:32618`/`:32632`).
        assert!((slots[0].vel.y - (-0.01 * 0.97)).abs() < 1e-6);
        assert!((slots[0].vel.x - 2.0 * 0.97).abs() < 1e-6);
        // The second tick draws NOTHING (ai[0] != 0).
        cloud_family_step(&mut slots, 0, &w, None, &mut npcs, &mut rng);
        assert_eq!(rng.ranges.len(), 1, "no second frame roll");
    }

    /// The lifetime caps (`:32585-32599`): the 1128's ai[1] seed (90 from
    /// the sweep), 220 when the bloom passed none, 260 for the toxic
    /// clouds, 80 for the gas; the kill pins ai[0] AT the cap. ANY cloud
    /// with ai[1] >= 1 ages triple (`:32581-32584`) — the sweep seed
    /// doubles as the hit-acceleration flag (the C# verbatim: the sweep
    /// clouds age 3/tick AND `Damage_CanDealDamage` refuses them,
    /// `:12448`; only the bloom clouds with ai1 = 0 deal damage).
    #[test]
    fn w4_d19_the_cloud_lifetime_caps_pin_the_age() {
        let w = world();
        let mut npcs: Vec<Npc> = Vec::new();
        let mut rng = census();
        // The sweep seed: ai1 = 90 → the cap is 90 AND the age runs 3/tick
        // → 30 ticks reach 90, tick 31 kills (93 > 90, pinned at 90).
        let mut slots = slots_with(proj(1128));
        slots[0].ai[1] = 90.0;
        for _ in 0..30 {
            cloud_family_step(&mut slots, 0, &w, None, &mut npcs, &mut rng);
        }
        assert!(slots[0].active, "tick 30 is AT the cap, not past it");
        assert_eq!(slots[0].ai[0], 90.0);
        cloud_family_step(&mut slots, 0, &w, None, &mut npcs, &mut rng);
        assert!(!slots[0].active, "the > cap kill (`:32594-32596`)");
        assert_eq!(slots[0].ai[0], 90.0, "the age pins at the cap");
        // The bloom cloud (ai1 = 0) ages 1/tick and lives 220.
        let mut slots = slots_with(proj(1128));
        for _ in 0..220 {
            cloud_family_step(&mut slots, 0, &w, None, &mut npcs, &mut rng);
        }
        assert!(slots[0].active);
        cloud_family_step(&mut slots, 0, &w, None, &mut npcs, &mut rng);
        assert!(!slots[0].active);
        assert_eq!(slots[0].ai[0], 220.0);
        // The toxic cloud lives 260; the gas 80.
        for (kind, cap) in [(511u16, 260.0f32), (1007, 80.0)] {
            let mut slots = slots_with(proj(kind));
            for _ in 0..(cap as usize) {
                cloud_family_step(&mut slots, 0, &w, None, &mut npcs, &mut rng);
            }
            assert!(slots[0].active, "kind {kind} at the cap");
            cloud_family_step(&mut slots, 0, &w, None, &mut npcs, &mut rng);
            assert!(!slots[0].active, "kind {kind} past the cap");
            assert_eq!(slots[0].ai[0], cap);
        }
        // The hit toxic cloud ages 3/tick: 260/3 → dead on tick 87.
        let mut slots = slots_with(proj(511));
        slots[0].ai[1] = 1.0;
        for _ in 0..87 {
            cloud_family_step(&mut slots, 0, &w, None, &mut npcs, &mut rng);
        }
        assert!(!slots[0].active, "3·87 = 261 > 260");
        assert_eq!(slots[0].ai[0], 260.0);
    }

    /// The 1128 repulsion (`:32671-32705`): two overlapping spores push
    /// apart along the centre delta with the overlap-depth factor; the
    /// zero-delta tie-break leans on the index order. Both start mid-life
    /// (ai[0] = 22 → the scale ramp is done, the box is the full 30).
    #[test]
    fn w4_d19_the_spores_repel_with_the_overlap_depth() {
        let w = world();
        let mut slots = slots_with(proj(1128));
        slots[0].ai[0] = 22.0;
        slots[1] = proj(1128);
        // Overlap: the second spore 10 px right (both boxes 30 wide).
        slots[1].pos = slots[0].pos + Vec2::new(10.0, 0.0);
        let mut rng = census();
        let mut npcs: Vec<Npc> = Vec::new();
        cloud_family_step(&mut slots, 0, &w, None, &mut npcs, &mut rng);
        // delta = (10, 0), dist 10 → push = (1,0)·((w−10)/w)·0.005 with
        // the POST-resize width (the tick-23 box is the full ramp end).
        // The arm's ×0.97 drag runs BEFORE the scan; the push subtracts.
        let width = slots[0].w;
        let push = (width - 10.0) / width * 0.005;
        assert!((slots[0].vel.x - (-push)).abs() < 1e-6, "self pushed left");
        // The other slot did not run its own arm — it only took the push.
        assert!((slots[1].vel.x - push).abs() < 1e-6, "other pushed right");
        // The zero-delta tie-break: j > i → self gets (+1,−1) normalized.
        let mut slots = slots_with(proj(1128));
        slots[0].ai[0] = 22.0;
        slots[1] = proj(1128);
        slots[1].pos = slots[0].pos;
        let mut rng = census();
        cloud_family_step(&mut slots, 0, &w, None, &mut npcs, &mut rng);
        let n = Vec2::new(1.0, -1.0).normalize();
        // The tie-break delta (1,−1) has length √2 (the C# reads
        // `vector106.Length()` AFTER the override, `:32699`) → factor
        // (w−√2)/w; the −0.01 float-up and the ×0.97 drag ran first.
        let width = slots[0].w;
        let factor = (width - 2.0f32.sqrt()) / width;
        let expected = Vec2::new(0.0, -0.01 * 0.97) - n * factor * 0.005;
        assert!((slots[0].vel - expected).length() < 1e-6);
    }

    /// The wall bounce (`:32706-32783`): a solid probe ahead mirrors the
    /// 1128's vertical component (the horizontal flight); the toxic cloud
    /// takes the 0.5 lerp brake instead.
    #[test]
    fn w4_d19_the_spore_mirrors_off_the_wall() {
        let mut w = world();
        // The spore centre is (1615, 815); the probe at +16 along the
        // (4, 0.99) flight lands at ≈(1630.5, 818.8) → tile (101, 51).
        // The X-mirror probe lands at ≈(1630.5, 811.2) → tile (101, 50),
        // which stays OPEN (the mirror succeeds).
        put(&mut w, 101, 51, 1);
        let mut slots = slots_with(proj(1128));
        slots[0].vel = Vec2::new(4.0, 1.0);
        let mut rng = census();
        let mut npcs: Vec<Npc> = Vec::new();
        cloud_family_step(&mut slots, 0, &w, None, &mut npcs, &mut rng);
        // |dir.X| > |dir.Y| → the X-mirror (x, −y) at the pre-bounce
        // speed (the −0.01 float ran BEFORE the ×0.97 drag, `:32618`
        // then `:32632`).
        let v = Vec2::new(4.0 * 0.97, (1.0 - 0.01) * 0.97);
        let speed = v.length();
        let dir = v.normalize();
        let expected = Vec2::new(dir.x, -dir.y) * speed;
        assert!((slots[0].vel - expected).length() < 1e-4, "the X-mirror");
        // The toxic cloud brakes instead: Lerp(v, v − dir, 0.5). Its
        // centre is (1616, 816); the probe lands in the same solid tile.
        let mut slots = slots_with(proj(511));
        slots[0].vel = Vec2::new(4.0, 1.0);
        let mut rng = census();
        cloud_family_step(&mut slots, 0, &w, None, &mut npcs, &mut rng);
        let v = Vec2::new(4.0, 1.0) * 0.96;
        let expected = v.lerp(v - v.normalize(), 0.5);
        assert!((slots[0].vel - expected).length() < 1e-4, "the 0.5 brake");
    }

    /// The 1007 gas (`:32788-32819`): the 15-tick cadence, the radius
    /// ramp, the player + NPC Poisoned (20, 2940), the immunity gate.
    /// The first tick ALSO queues the broadcast's Item16 cue at the
    /// PRE-growth centre (the W4-d10 keep-one fold —
    /// `ParticleOrchestrator.cs:2913`, `PositionInWorld = base.Center`
    /// captured at `:32576` BEFORE the `:32605` resize).
    #[test]
    fn w4_d19_the_gas_poisons_on_the_15_tick_cadence() {
        let w = world();
        let mut pl = crate::player::Player::spawn(&w);
        // The player centre at the gas centre (1625, 825).
        pl.x = 1625.0 - crate::player::WIDTH * 0.5;
        pl.y = 825.0 - pl.height * 0.5;
        let mut n = Npc::new(NpcKind::Zombie, 0.0, 0.0, 1, 0.0);
        n.hp = 40;
        n.max_hp = 40;
        let (nw, nh) = n.size();
        n.x = 1625.0 - nw * 0.5;
        n.y = 825.0 - nh * 0.5;
        let mut immune = n.clone();
        immune.buffs.buff_immune[20] = true;
        let mut npcs = vec![n, immune];
        let mut slots = slots_with(proj(1007));
        let c0 = slots[0].center();
        let mut rng = census();
        cloud_family_step(&mut slots, 0, &w, Some(&mut pl), &mut npcs, &mut rng);
        assert_eq!(
            slots[0].wants_sound,
            Some((2, 16, c0.0, c0.1, 0.0, 1.0)),
            "the broadcast cue at the pre-growth centre"
        );
        assert!(slots[0].scale > 1.0, "the 1→6 growth ramp (`:32603`)");
        assert_eq!(slots[0].local_ai[1], 15.0, "the cadence re-arms");
        assert!(pl.buffs.find_buff_index(20).is_some(), "the player poisoned");
        assert!(npcs[0].buffs.find_buff_index(20).is_some(), "the NPC poisoned");
        assert!(npcs[1].buffs.find_buff_index(20).is_none(), "the immune NPC refused");
        assert_eq!(rng.ranges.len(), 0, "the gas path draws nothing");
        // Tick 2: the cadence counts down, no re-application needed to
        // observe the counter (the buff refresh is idempotent).
        cloud_family_step(&mut slots, 0, &w, Some(&mut pl), &mut npcs, &mut rng);
        assert_eq!(slots[0].local_ai[1], 14.0);
        // The gas velocity is pinned zero every tick (`:32628`).
        assert_eq!(slots[0].vel, Vec2::ZERO);
        // The cue is ONE-shot: the `localAI[0] == 0` gate re-reads the
        // spin accumulator, which the first tick already advanced by
        // `direction` (`:32624`) — tick 2 never re-broadcasts.
        slots[0].wants_sound = None;
        cloud_family_step(&mut slots, 0, &w, Some(&mut pl), &mut npcs, &mut rng);
        assert_eq!(slots[0].wants_sound, None, "no second broadcast");
    }

    /// The toxic cloud's opacity ramp + the whoAmI rotation seed
    /// (`:32622`, `:32624-32625`) — the d10 keep-one fold's numeric pins.
    /// Tick 1 at index 2: fv = 1/260, opacity = Remap(fv,0,.3,0,1) ·
    /// Remap(fv,.3,1,1,0) · 0.7 ≈ 0.008941 → alpha = (int)((1−o)·255) =
    /// 252; rot = 2·0.4002029 + direction·2π/480. ZERO draws.
    #[test]
    fn w4_d19_the_toxic_cloud_fades_and_spins_from_its_index() {
        let w = world();
        let mut slots = vec![Projectile::default(); 4];
        slots[2] = proj(511);
        slots[2].vel = Vec2::new(2.0, 0.0);
        let mut rng = census();
        let mut npcs: Vec<Npc> = Vec::new();
        cloud_family_step(&mut slots, 2, &w, None, &mut npcs, &mut rng);
        assert_eq!(slots[2].ai[0], 1.0);
        assert_eq!(slots[2].alpha, 252, "the tick-1 opacity truncation");
        assert!((slots[2].vel.x - 2.0 * 0.96).abs() < 1e-6, "the 0.96 drag");
        let expected_rot =
            2.0 * 0.4002029 + 1.0 * (std::f32::consts::PI * 2.0) / 480.0;
        assert!((slots[2].rot - expected_rot).abs() < 1e-5, "whoAmI seed");
        assert!(rng.ranges.is_empty() && rng.floats == 0, "ZERO draws");
        // The hit counter triples the age (`:32581-32584`): 1 → 4.
        slots[2].ai[1] = 1.0;
        cloud_family_step(&mut slots, 2, &w, None, &mut npcs, &mut rng);
        assert_eq!(slots[2].ai[0], 4.0);
    }

    /// The TOXIC pair separation (`:32638-32670`) — the d10 keep-one
    /// fold: the 0.005 UnitX push lerped 0.6 on BOTH slots, the
    /// zero-delta index tie-break. (The 1128 scan is pinned above.)
    #[test]
    fn w4_d19_the_toxic_clouds_separate_symmetrically() {
        let w = world();
        let mut slots = slots_with(proj(511));
        slots[1] = proj(511);
        slots[1].pos = slots[0].pos + Vec2::new(8.0, 0.0);
        let mut rng = census();
        let mut npcs: Vec<Npc> = Vec::new();
        cloud_family_step(&mut slots, 0, &w, None, &mut npcs, &mut rng);
        // delta = (8,0) → push = (0.005, 0); self lerps toward −push,
        // the sibling toward +push (both 0.6).
        assert!((slots[0].vel.x - (-0.005 * 0.6)).abs() < 1e-6);
        assert!((slots[1].vel.x - (0.005 * 0.6)).abs() < 1e-6);
        // Exact overlap: j > i → the (+1,−1) tie-break delta; SELF lerps
        // toward velocity − push, so it moves (−1,+1). (d10's witness
        // asserted the opposite sign — a written-not-run defect, the
        // C# `:32667` subtraction re-derived; research/26g §1.7.)
        slots[1].pos = slots[0].pos;
        slots[0].vel = Vec2::ZERO;
        slots[1].vel = Vec2::ZERO;
        cloud_family_step(&mut slots, 0, &w, None, &mut npcs, &mut rng);
        let n = Vec2::new(1.0, -1.0).normalize();
        assert!((slots[0].vel - (Vec2::ZERO.lerp(-n * 0.005, 0.6))).length() < 1e-6);
        assert!(slots[0].vel.x < 0.0 && slots[0].vel.y > 0.0, "self takes −push");
        assert!(slots[1].vel.x > 0.0 && slots[1].vel.y < 0.0, "sibling takes +push");
    }

    /// The drain (`:41944`/`:41967`/`:42172`): the clouds spawn as 1128
    /// CENTRED with their ai[1] seed; the slam no-ops until d21's 1129 row
    /// merges (the documented contract).
    #[test]
    fn w4_d19_the_spore_drain_centres_the_clouds_and_parks_the_slam() {
        let mut pool = ProjectilePool::new();
        let spores = crate::projectile_ai_sword_styles::ClaymoreSpores {
            clouds: vec![(1608.0, 832.0, 1.5, -2.0, 75, 90.0)],
            slam: Some((1700.0, 700.0, 100, 128.0, 96.0)),
        };
        drain_claymore_spores(&mut pool, 0, spores);
        let cloud = pool
            .slots()
            .iter()
            .find(|p| p.active && p.kind == 1128)
            .expect("the cloud spawned");
        // NewProjectile centres the box (`:10876-10877`): 30×30 → −15.
        assert_eq!(cloud.pos, Vec2::new(1608.0 - 15.0, 832.0 - 15.0));
        assert_eq!(cloud.vel, Vec2::new(1.5, -2.0));
        assert_eq!(cloud.damage, 75);
        assert_eq!(cloud.ai[1], 90.0, "the lifetime seed rides ai[1]");
        assert_eq!(cloud.owner, 0);
        // The 1129 slam: absent until d21's row merges; once it does, it
        // spawns CENTRED with ai1 = 128 / ai2 = 96 (the witness holds in
        // BOTH states — the cross-branch contract).
        let slam = pool.slots().iter().find(|p| p.active && p.kind == 1129);
        match proj_data(1129) {
            None => assert!(slam.is_none(), "the slam waits for d21's row"),
            Some(row) => {
                let slam = slam.expect("the slam spawned with d21's row");
                assert_eq!(slam.pos, Vec2::new(1700.0 - row.w * 0.5, 700.0 - row.h * 0.5));
                assert_eq!(slam.damage, 100, "the full damage (`:41967`)");
                assert_eq!(slam.ai[1], 128.0);
                assert_eq!(slam.ai[2], 96.0);
            }
        }
    }
}
