//! W4-d10 — the projectile aiStyles 91-100 census port (the f19 F1 finding
//! included): the nine MISSING styles plus the aiStyle-99 counterweight
//! SUB-arm, ported verbatim from `Projectile.cs` behind the pool's
//! `match data.ai_style` dispatch (`projectile.rs::update`).
//!
//! Census at the task's base (`a725acb0`, in-flight probe: d9/g1/g2/g3 carry
//! no unique commits, f19 is census-only `049cc8e2`):
//!
//! | style | C# arm | kinds (SetDefaults) | port state at base |
//! |-------|--------|---------------------|--------------------|
//! | 91 | inline `AI()` `:32515-32557` | ShadowFlame 496 (`:5692-5704`) | MISSING → this module |
//! | 92 | inline `:32558-32820` | ToxicCloud 511/512/513 (`:5805-5840`), GasTrap 1007 (`:9558-9569`), ChlorophyteClaymoreSporeCloud 1128 (`:10565-10579`) | MISSING → this module |
//! | 93 | inline `:32821-32882` | NailFriendly 514 (`:5841-5850`) | MISSING → this module |
//! | 94 | inline `:32883-32963` | CoinPortal 518 (`:5879-5888`) | MISSING → this module |
//! | 95 | inline `:32964-33003` | ToxicBubble 523 (`:5929-5938`) | MISSING → this module |
//! | 96 | inline `:33004-33028` | IchorSplash 524 (`:5939-5952`) | MISSING → this module |
//! | 97 | inline `:33029-33133` | FlyingPiggyBank 525 (`:5953-5961`) | MISSING → this module |
//! | 98 | inline `:33134-33152` | CultistBossParticle 526 (`:5962-5970`) | MISSING → this module |
//! | 99 | `AI_099_1_Counterweights` `:67380-67731` (gate `:33153-33156`) + `AI_099_2_Yoyos` `:67733+` (gate `:33157-33160`) | yoyos 534/541-555/562-564/603/999; counterweights 556-561/1079 (`:6084-6093`) | yoyo half LANDED (`projectile.rs::ai_yoyo`); counterweight sub-arm MISSING (f19 F1) → this module |
//! | 100 | `AI_100_Medusa` `:44476-44632` (gate `:33161-33164`) | MedusaHead 535 (`:6094-6106`) | MISSING → this module |
//!
//! RNG contract (the `UpdateProjectiles` stream, `Main.cs:17775-17777`):
//! every EXPLICIT `Main.rand` draw at the call sites stays, in the C#'s
//! order; the dust BODIES are skipped (no draw layer wired here) and the
//! `Dust.NewDust*` internal rolls go with the dust pool (ANALYSIS "Dust &
//! gore", the `projectile_ai_misc` precedent). `RotatedByRandom`
//! (`Utils.cs:1697-1700`) is TWO `NextDouble` draws — mapped onto
//! `next_float` (one Sample() per draw either way, the ai067_b convention).
//! Per style:
//! - 91: ZERO draws — the dust loop (`:32546-32555`) rolls only NewDust
//!   internals.
//! - 92: the 1128 first-tick `Next(projFrames[type])` frame roll
//!   (`:32565`); the GasTrap particle broadcast (`:32574-32578`) is the
//!   particle-layer skip — `Spawn_GasTrap`'s internals
//!   (`ParticleOrchestrator.cs:2911-2953`) go with the unported particle
//!   pool, the `Dust.NewDust` body convention.
//! - 93: the NPC-death release's TWO `Next(-100, 101)` draws (`:32864`).
//!   The `Kill()` visuals (`:76343-76365`: Item14 + the dust bursts + the
//!   6×`Next(2)`) are the pool-wide kill-dust skip (the empress precedent).
//! - 94: the tick-40 `Next(3)` cadence pick (`:32905`); the coin drop's
//!   `RotatedByRandom(2π)` pair + the `NextFloat` speed (`:32933`) — the
//!   explicit velocity skips `Item.NewItem`'s scatter pair
//!   (`Item.cs:49738-49741`) and `prefix: 0` rolls nothing, so the drop
//!   request is stream-neutral at the drain; the tail dust gate's
//!   `Next(3)` (`:32953`) + the taken branch's `RotatedByRandom(2π)` pair
//!   (`:32957`).
//! - 95: ZERO draws in the AI; the `Kill()` burst (`:73257-73269`) is the
//!   pool-wide kill-dust skip.
//! - 96: the 2-iteration dust gate's `Next(3)` per iteration (`:33013`).
//! - 97: ZERO draws.
//! - 98: the 2-dust loop's `NextFloat` scale per dust (`:33150`).
//! - 99 counterweight: ZERO draws (the machine is pure maths; the
//!   magicString clone spawn `:67641` is an unported accessory grant).
//! - 100: the fire window's per-scatter-ray `Next(4)` +
//!   `RotatedByRandom` pair + `NextFloat` (`:44603`), in the C#'s order
//!   BEFORE the ray spawns' damage walks (the pool-side step keeps the
//!   inline order).
//!
//! Documented deviations (the module-wide folds, ANALYSIS "W4-d10"):
//! - `Lighting.AddLight` (`:32962`, `:44612`) — no per-tick AI light layer
//!   (the `proj_lights.rs` frame feed is the SetDefaults `light` slice).
//! - the yoyoString/magicString/counterWeight accessory grants stay with
//!   M3-EQUIP-3 (the `ai_yoyo` precedent) — their terms fold to the
//!   no-accessory branch.
//! - `Main.player[owner].deadMansSweater` (`ApplyBuffTo` `:12393-12396`)
//!   is unported — the gas trap's buff time never halves.
//! - `CurrentFrameFlags.HadAnActiveInteractableProjectile` (`:33070`) and
//!   the piggy bank's container UI (`TryGetContainerIndex` `:78815`) have
//!   no port consumer.
//! - the 535's rays are the aiStyle-101 kind 536 — d11's claim; until its
//!   row lands the ray spawns decline (`spawn_with_data`'s no-row no-op)
//!   and the Medusa Head deals no damage (the dependency is recorded).
//! - `hide` (1007) / `drawLayer` / `usesOwnerLight` (535) have no port
//!   slot fields — draw-layer state, the module doc convention.
//!
//! CODE-ONLY MODE (W4-d10): the witness tests at the bottom are WRITTEN
//! NOT RUN.

use glam::Vec2;
use tr_core::World;
use tr_core::loot::DropRng;

use crate::npc::Npc;
// The later styles' scans share these helpers (93/99/100); the wave lands
// one style per commit, so the early commits carry the unused seams.
#[allow(unused_imports)]
use crate::projectile::{
    Projectile, can_hit, chaseable, npc_center, proj_frames, remap, safe_normalize,
};

/// `Collision.IsWorldPointSolid(pos, treatPlatformsAsNonSolid)`
/// (`Collision.cs:1434-1491`) — the aiStyle-92 clouds' forward probe. The
/// tile must be in-world (fluff 1), active, not actuated and solid; the
/// platform exclusion reads `TileID.Sets.Platforms` + the 380; the
/// `blockType()` ladder (`Tile.cs:202-214`: halfBrick → 1, slope>0 →
/// slope+1) tests the point against the tile's shape. The case-2..5 head
/// guard (`pos.X < left && pos.X > right && …`, `:1470`) is the C#'s own
/// always-false conjunction — kept verbatim (it never fires; the inner
/// slope switch decides).
fn world_point_solid(world: &World, pos: Vec2, platforms_non_solid: bool) -> bool {
    let tx = (pos.x / 16.0) as i32;
    let ty = (pos.y / 16.0) as i32;
    // `WorldGen.InWorld(x, y, 1)`.
    if tx < 1 || tx >= world.width - 1 || ty < 1 || ty >= world.height - 1 {
        return false;
    }
    let Some(t) = world.tile(tx, ty) else { return false };
    if !t.active() || t.inactive() || !tr_core::tiledata::TILE_SOLID.has(t.kind) {
        return false;
    }
    if platforms_non_solid
        && t.kind > 0
        && (tr_core::tiledata::SET_PLATFORMS.has(t.kind) || t.kind == 380)
    {
        return false;
    }
    let (l, t_) = (tx as f32 * 16.0, ty as f32 * 16.0);
    let block = if t.half_brick() {
        1
    } else if t.slope() > 0 {
        t.slope() + 1
    } else {
        0
    };
    match block {
        0 => pos.x >= l && pos.x <= l + 16.0 && pos.y >= t_ && pos.y <= t_ + 16.0,
        1 => pos.x >= l && pos.x <= l + 16.0 && pos.y >= t_ + 8.0 && pos.y <= t_ + 16.0,
        2..=5 => {
            // `:1470` — the always-false head guard, verbatim.
            if pos.x < l && pos.x > l + 16.0 && pos.y < t_ && pos.y > t_ + 16.0 {
                return false;
            }
            let num2 = pos.x % 16.0;
            let num3 = pos.y % 16.0;
            match block {
                3 => num2 + num3 >= 16.0,
                2 => num3 >= num2,
                5 => num3 <= num2,
                4 => num2 + num3 <= 16.0,
                _ => false,
            }
        }
        _ => false,
    }
}

/// `Projectile.Opacity`'s setter (`Projectile.cs:546-549`):
/// `alpha = (int)MathHelper.Clamp((1f - value) * 255f, 0f, 255f)`.
fn set_opacity(p: &mut Projectile, value: f32) {
    p.alpha = ((1.0 - value) * 255.0).clamp(0.0, 255.0) as i32;
}

/// `AI()`'s aiStyle-92 arm (`Projectile.cs:32558-32820`) — the Toxic Cloud
/// trio 511-513 (`flag45`), the Gas Trap 1007 (`flag44`) and the
/// Chlorophyte Claymore's spore cloud 1128 (`flag46`). Pool-side: the
/// separation scans write the SIBLING slots' velocities (`:32641-32704`),
/// the gas tail buffs the local player + the NPC vec (`:32800-32819`).
///
/// RNG: the 1128 first-tick `Next(projFrames[type])` frame roll (`:32565`)
/// is the arm's only AI draw. The GasTrap particle broadcast
/// (`:32574-32578`) rides the UNPORTED particle pool — its
/// `Spawn_GasTrap` body (`ParticleOrchestrator.cs:2911-2953`, the 12+10
/// particles' NextFloat/RotatedByRandom/Next draws) is the particle-layer
/// skip (the dust-body convention); its `SoundID.Item16` cue queues
/// through `wants_sound` (the S1-E-21h seam).
#[allow(clippy::too_many_arguments)]
pub(crate) fn ai_clouds(
    slots: &mut [Projectile],
    i: usize,
    world: &World,
    npcs: &mut [Npc],
    mut player: Option<&mut crate::player::Player>,
    local_owner: u8,
    rng: &mut dyn DropRng,
) {
    let kind = slots[i].kind;
    let flag44 = kind == 1007;
    let flag45 = (511..=513).contains(&kind);
    let flag46 = kind == 1128;
    // `:32563-32566` — the spore cloud's first-tick frame roll.
    if flag46 && slots[i].ai[0] == 0.0 {
        slots[i].frame = rng.next_range(0, proj_frames(kind) as i32);
    }
    // `:32567` — the whole family flies through tiles.
    slots[i].tile_collide = false;
    // `:32568-32579` — the gas trap's one-shot particle broadcast (the
    // `netMode != 1` arm folds open in single-player). The direction fix
    // stays; the broadcast is the particle-layer skip, its Item16 cue
    // queues (module doc).
    if flag44 && slots[i].local_ai[0] == 0.0 {
        if slots[i].direction == 0 {
            slots[i].direction = 1;
        }
        // `PositionInWorld = base.Center` — the cue rides the centre (the
        // SoundEngine vector overload, `ParticleOrchestrator.cs:2913`).
        let (cx, cy) = slots[i].center();
        slots[i].wants_sound = Some((2, 16, cx, cy, 0.0));
    }
    // `:32580-32584` — the age counter (the hit-counter `ai[1] >= 1`
    // doubles the aging — `Damage_PVE_Inner` `:13218-13221` bumps it).
    slots[i].ai[0] += 1.0;
    if slots[i].ai[1] >= 1.0 {
        slots[i].ai[0] += 2.0;
    }
    // `:32585-32593` — the per-kind lifetime.
    let mut num763 = 260.0f32;
    if flag44 {
        num763 = 80.0;
    }
    if flag46 {
        num763 = if slots[i].ai[1] == 0.0 { 220.0 } else { slots[i].ai[1] };
    }
    // `:32594-32599` — the kill (the ai[0] pin lands AFTER, verbatim).
    if slots[i].ai[0] > num763 {
        slots[i].kill_time_left = slots[i].time_left;
        slots[i].active = false;
        slots[i].time_left = 0;
        slots[i].ai[0] = num763;
        return;
    }
    let from_value = slots[i].ai[0] / num763;
    // `:32601-32623` — the per-kind scale/box/opacity ramps.
    if flag44 {
        slots[i].scale = remap(from_value, 0.0, 0.95, 1.0, 6.0);
        let center = Vec2::from(slots[i].center());
        slots[i].w = (50.0 * slots[i].scale) as i32 as f32;
        slots[i].h = (50.0 * slots[i].scale) as i32 as f32;
        // `base.Center = center17` — the Entity.Center SETTER (`Entity.cs:
        // 56-59`): float halves, no int truncation.
        slots[i].pos = center - Vec2::new(slots[i].w / 2.0, slots[i].h / 2.0);
        let o = (remap(from_value, 0.0, 0.25, 0.0, 1.0)
            * remap(from_value, 0.75, 1.0, 1.0, 0.0))
        .clamp(0.0, 1.0)
            * 0.85;
        set_opacity(&mut slots[i], o);
    } else if flag46 {
        slots[i].scale = remap(from_value, 0.0, 0.1, 0.25, 1.1);
        let center = Vec2::from(slots[i].center());
        slots[i].w = (30.0 * slots[i].scale / 1.1) as i32 as f32;
        slots[i].h = slots[i].w;
        slots[i].pos = center - Vec2::new(slots[i].w / 2.0, slots[i].h / 2.0);
        let o = remap(from_value, 0.0, 0.25, 0.0, 1.0)
            * remap(from_value, 0.5, 1.0, 1.0, 0.0)
            * 0.75;
        set_opacity(&mut slots[i], o);
        slots[i].vel.y -= 0.01;
    } else {
        let o = remap(from_value, 0.0, 0.3, 0.0, 1.0)
            * remap(from_value, 0.3, 1.0, 1.0, 0.0)
            * 0.7;
        set_opacity(&mut slots[i], o);
    }
    // `:32624-32625` — the spin accumulator + the whoAmI-seeded rotation.
    slots[i].local_ai[0] += slots[i].direction as f32;
    slots[i].rot = i as f32 * 0.4002029
        + slots[i].local_ai[0] * (std::f32::consts::PI * 2.0) / 480.0;
    // `:32626-32637` — the per-kind velocity decay.
    if flag44 {
        slots[i].vel = Vec2::ZERO;
    } else if flag46 {
        slots[i].vel *= 0.97;
    } else {
        slots[i].vel *= 0.96;
    }
    // `:32638-32670` — the toxic clouds' same-family separation: the
    // truncated-box intersect scan, the zero-delta index tie-break, the
    // 0.005 SafeNormalize(UnitX) push and the 0.6 Lerp on BOTH slots.
    if flag45 {
        let (px, py, pw, ph) = {
            let p = &slots[i];
            (p.pos.x as i32, p.pos.y as i32, p.w as i32, p.h as i32)
        };
        let center = Vec2::from(slots[i].center());
        for j in 0..slots.len() {
            if j == i {
                continue;
            }
            let o = &slots[j];
            if !o.active || !(511..=513).contains(&o.kind) {
                continue;
            }
            let (ox, oy, ow, oh) = (o.pos.x as i32, o.pos.y as i32, o.w as i32, o.h as i32);
            if !(px < ox + ow && px + pw > ox && py < oy + oh && py + ph > oy) {
                continue;
            }
            let mut v = Vec2::from(o.center()) - center;
            if v == Vec2::ZERO {
                v = if j < i { Vec2::new(-1.0, 1.0) } else { Vec2::new(1.0, -1.0) };
            }
            let push = crate::projectile::safe_normalize_or(v, Vec2::X) * 0.005;
            let vel = slots[i].vel;
            slots[i].vel = vel.lerp(vel - push, 0.6);
            let ov = slots[j].vel;
            slots[j].vel = ov.lerp(ov + push, 0.6);
        }
    }
    // `:32671-32705` — the spore clouds' same-type separation: the
    // width-falloff push (`max(width - dist, 0) / width * 0.005`), the
    // plain Normalize (the zero case is the tie-break above).
    if flag46 {
        let (px, py, pw, ph) = {
            let p = &slots[i];
            (p.pos.x as i32, p.pos.y as i32, p.w as i32, p.h as i32)
        };
        let center = Vec2::from(slots[i].center());
        let width = slots[i].w;
        for j in 0..slots.len() {
            if j == i {
                continue;
            }
            let o = &slots[j];
            if !o.active || o.kind != 1128 {
                continue;
            }
            let (ox, oy, ow, oh) = (o.pos.x as i32, o.pos.y as i32, o.w as i32, o.h as i32);
            if !(px < ox + ow && px + pw > ox && py < oy + oh && py + ph > oy) {
                continue;
            }
            let mut v = Vec2::from(o.center()) - center;
            if v == Vec2::ZERO {
                v = if j < i { Vec2::new(-1.0, 1.0) } else { Vec2::new(1.0, -1.0) };
            }
            let dist = v.length();
            v = v.normalize();
            let push = v * ((width - dist).max(0.0) / width) * 0.005;
            slots[i].vel -= push;
            slots[j].vel += push;
        }
    }
    // `:32706-32783` — the forward solid probe (16 px along the velocity
    // direction); the gas trap never probes. The spore cloud runs the
    // reflection ladder, the toxic clouds the plain 0.5 Lerp brake.
    let dir = safe_normalize(slots[i].vel);
    let probe = Vec2::from(slots[i].center()) + dir * 16.0;
    if !flag44 && world_point_solid(world, probe, true) {
        if flag46 {
            let mut flag47 = false;
            let speed = slots[i].vel.length();
            let center = Vec2::from(slots[i].center());
            if dir.x.abs() > dir.y.abs() {
                let mut v110 = Vec2::new(dir.x, -dir.y);
                if !world_point_solid(world, center + v110 * 16.0, true) {
                    slots[i].vel = v110 * speed;
                    flag47 = true;
                } else if !world_point_solid(world, probe - Vec2::new(0.0, 16.0), true) {
                    v110 = probe - Vec2::new(0.0, 16.0) - center;
                    slots[i].vel = v110.normalize() * speed;
                    flag47 = true;
                } else if !world_point_solid(world, probe + Vec2::new(0.0, 16.0), true) {
                    v110 = probe + Vec2::new(0.0, 16.0) - center;
                    slots[i].vel = v110.normalize() * speed;
                    flag47 = true;
                } else if !world_point_solid(world, probe - Vec2::new(0.0, 32.0), true) {
                    v110 = probe - Vec2::new(0.0, 32.0) - center;
                    slots[i].vel = v110.normalize() * speed;
                    flag47 = true;
                } else if !world_point_solid(world, probe + Vec2::new(0.0, 32.0), true) {
                    v110 = probe + Vec2::new(0.0, 32.0) - center;
                    slots[i].vel = v110.normalize() * speed;
                    flag47 = true;
                }
            }
            if !flag47 {
                let mut v110 = Vec2::new(-dir.x, dir.y);
                let probe2 = center + v110 * 16.0;
                if !world_point_solid(world, probe2, true) {
                    slots[i].vel = v110 * speed;
                    flag47 = true;
                } else if !world_point_solid(world, probe2 - Vec2::new(16.0, 0.0), true) {
                    v110 = probe2 - Vec2::new(16.0, 0.0) - center;
                    slots[i].vel = v110.normalize() * speed;
                    flag47 = true;
                } else if !world_point_solid(world, probe2 + Vec2::new(16.0, 0.0), true) {
                    v110 = probe2 + Vec2::new(16.0, 0.0) - center;
                    slots[i].vel = v110.normalize() * speed;
                    flag47 = true;
                }
                if !flag47 {
                    let vel = slots[i].vel;
                    slots[i].vel = vel.lerp(vel - dir * 1.0, 0.5);
                }
            }
        } else {
            let vel = slots[i].vel;
            slots[i].vel = vel.lerp(vel - dir * 1.0, 0.5);
        }
    }
    // `:32784-32787` — only the gas trap runs the buff tail.
    if !flag44 {
        return;
    }
    // `:32788-32799` — the Poisoned (20) cloud: the 180-px radius ramp,
    // the 15-tick cadence (`localAI[1]`).
    let num769 = remap(from_value, 0.0, 0.2, 0.0, 1.0).clamp(0.0, 1.0) * 180.0;
    if slots[i].local_ai[1] > 0.0 {
        slots[i].local_ai[1] -= 1.0;
    }
    if slots[i].local_ai[1] > 0.0 {
        return;
    }
    slots[i].local_ai[1] = 15.0;
    let center = Vec2::from(slots[i].center());
    // `:32800-32807` — the local player's buff (`netMode != 2` folds
    // open). `ApplyBuffTo` (`:12391-12398`): the trap's deadMansSweater
    // halving is an unported accessory — the time stays 2940.
    if slots[i].owner == local_owner {
        if let Some(pl) = player.as_deref_mut() {
            if !pl.dead && Vec2::from(pl.center()).distance(center) <= num769 {
                pl.buffs.add_buff(20, 2940);
            }
        }
    }
    // `:32808-32811` — the `netMode == 1` client bail folds out (SP).
    // `:32812-32819` — the NPC buff scan.
    for n in npcs.iter_mut() {
        if n.hp > 0 && !n.buffs.buff_immune[20] {
            if Vec2::from(n.center()).distance(center) <= num769 {
                n.buffs.add_buff(20, 2940);
            }
        }
    }
}


/// `AI()`'s aiStyle-91 arm (`Projectile.cs:32515-32557`) — the Shadowflame
/// Hex Doll's flame (496). The center-preserving scale shrink (the box is
/// `(int)(20·scale)` square), the localAI[0] fade ramp (0.01 below 0.1,
/// 0.025 above — the DOUBLE comparison `(double)localAI[0] < 0.1`), the
/// 0.95 kill, the ai-driven 1.5 accel with the 16 cap and the 1.05 ai
/// growth. The dust loop (`:32544-32556`) feeds only skipped NewDust
/// bodies — ZERO draws by construction.
pub(crate) fn ai_shadowflame(p: &mut Projectile) {
    // `:32517-32522` — the center latch, the scale, the int-truncated box,
    // the position re-anchor (`width / 2` is the C# INT division).
    let center = Vec2::from(p.center());
    p.scale = 1.0 - p.local_ai[0];
    p.w = (20.0 * p.scale) as i32 as f32;
    p.h = p.w;
    p.pos.x = center.x - (p.w as i32 / 2) as f32;
    p.pos.y = center.y - (p.h as i32 / 2) as f32;
    // `:32523-32530` — the fade ramp.
    if p.local_ai[0] < 0.1 {
        p.local_ai[0] += 0.01;
    } else {
        p.local_ai[0] += 0.025;
    }
    // `:32531-32534` — the 0.95 kill.
    if p.local_ai[0] >= 0.95 {
        p.kill_time_left = p.time_left;
        p.active = false;
        p.time_left = 0;
    }
    // `:32535-32541` — the ai accel + the 16 cap.
    p.vel.x += p.ai[0] * 1.5;
    p.vel.y += p.ai[1] * 1.5;
    if p.vel.length() > 16.0 {
        p.vel = p.vel.normalize() * 16.0;
    }
    // `:32542-32543` — the ai growth.
    p.ai[0] *= 1.05;
    p.ai[1] *= 1.05;
    // `:32544-32556` — the scale<1 dust loop: `(float)num < scale * 10f`
    // iterations of NewDust 27 — the bodies and their internal rolls are
    // the dust-layer skip; the loop writes no slot state.
}

/// The 496's `HandleMovement` blocked-move arm (`Projectile.cs:18710-18735`):
/// per axis, a blocked move with `|velocity| < 1` reverses off the
/// pre-collision velocity, a harder one KILLS. The free move and the
/// post-arm `UpdatePosition` integrate (`:20445` — the 496 is not on the
/// skip list); the collision box is the 10×10 `GetCollisionParams` slice
/// (`:19957-19961`), centered (`resizeAnchor` 0.5).
pub(crate) fn shadowflame_move(p: &mut Projectile, world: &World) {
    if !p.tile_collide {
        p.pos += p.vel;
        return;
    }
    let last = p.vel;
    // The 10×10 centered collision box (`:19862`'s `Size/2 - col*anchor`).
    let off = Vec2::new(p.w * 0.5 - 5.0, p.h * 0.5 - 5.0);
    let tc = crate::collision::tile_collision(
        world,
        [p.pos.x + off.x, p.pos.y + off.y],
        [p.vel.x, p.vel.y],
        10.0,
        10.0,
        true,
        true,
    );
    p.vel = Vec2::new(tc.vel[0], tc.vel[1]);
    // `:18712-18722` — the X axis.
    if p.vel.x != last.x {
        if p.vel.x.abs() < 1.0 {
            p.vel.x = -last.x;
        } else {
            p.kill_time_left = p.time_left;
            p.active = false;
            p.time_left = 0;
        }
    }
    // `:18723-18733` — the Y axis (the C# runs it even after the X kill).
    if p.vel.y != last.y {
        if p.vel.y.abs() < 1.0 {
            p.vel.y = -last.y;
        } else {
            p.kill_time_left = p.time_left;
            p.active = false;
            p.time_left = 0;
        }
    }
    // `UpdatePosition` (`:20445`) + `AutomaticallyChangesDirection`
    // (`:17016-17026` — aiStyle 91 is not excluded).
    p.pos += p.vel;
    p.direction = if p.vel.x < 0.0 { -1 } else { 1 };
}

/// `AI()`'s aiStyle-93 arm (`Projectile.cs:32821-32882`) — the Nail Gun's
/// nail (514). The alpha fade (−25, floor 0), the 18 fall cap, the free
/// state's 20-tick hover then the 0.1 gravity + 0.992 X decay with the
/// atan2+1.57 tilt; the stuck state (`ai[0] < 0`, the victim index
/// `-ai[0]-1`) re-anchors to the NPC's center every tick, releases on the
/// NPC's death with TWO `Next(-100, 101)` draws + the 6-speed normalize,
/// and the 90-tick stuck fuse kills. `ai[0] == 2` is the death-blast
/// state (`Kill()` `:76373`) — it rides the else arm's anchor read with
/// index −3 (the C#'s own out-of-range read; the port guards it to the
/// release, the documented fold — the blast's Damage() runs in the kill
/// hook before any AI tick sees the state).
pub(crate) fn ai_nail(p: &mut Projectile, npcs: &[Npc], rng: &mut dyn DropRng) {
    // `:32823-32830` — the alpha fade.
    if p.alpha > 0 {
        p.alpha -= 25;
        if p.alpha <= 0 {
            p.alpha = 0;
        }
    }
    // `:32831-32834` — the fall cap.
    if p.vel.y > 18.0 {
        p.vel.y = 18.0;
    }
    // `:32835-32845` — the free flight: the 20-tick hover, then gravity.
    if p.ai[0] == 0.0 {
        p.ai[1] += 1.0;
        if p.ai[1] > 20.0 {
            p.vel.y += 0.1;
            p.vel.x *= 0.992;
        }
        p.rot = p.vel.y.atan2(p.vel.x) + 1.57;
        return;
    }
    // `:32846` — every non-free state flies through tiles.
    p.tile_collide = false;
    if p.ai[0] == 1.0 {
        // `:32847-32851` — the wall-stuck brake.
        p.vel *= 0.6;
    } else {
        // `:32852-32873` — the NPC anchor. The C# reads
        // `Main.npc[(int)(0f - ai[0]) - 1]` and anchors FIRST, then tests
        // the death; the port guards the index (the ai[0]==2 blast
        // state's −3 read never reaches the AI — Kill() deactivates at
        // its tail — the documented fold).
        let idx = (0.0 - p.ai[0]) as i32 - 1;
        let anchor = npcs.get(idx.max(0) as usize).filter(|_| idx >= 0);
        if let Some(n) = anchor {
            // `:32856-32858` — the anchor write runs even for a dead NPC
            // (the C# order: position first, the death test after).
            let (ncx, ncy) = n.center();
            p.pos = Vec2::new(ncx, ncy) - p.vel;
            p.pos.x -= (p.w as i32 / 2) as f32;
            p.pos.y -= (p.h as i32 / 2) as f32;
        }
        let alive = anchor.is_some_and(|n| n.hp > 0);
        if !alive {
            // `:32859-32868` — the release: tileCollide back on, the
            // free state re-armed at the hover boundary, the random
            // 6-speed kick. TWO Next(-100, 101) draws, X then Y.
            p.tile_collide = true;
            p.ai[0] = 0.0;
            p.ai[1] = 20.0;
            let x = rng.next_range(-100, 101) as f32;
            let y = rng.next_range(-100, 101) as f32;
            let v = Vec2::new(x, y);
            p.vel = safe_normalize(v) * 6.0;
            p.net_update = true;
        } else if let Some(n) = anchor {
            // `:32869-32872` — the (width+height)/3 INT-division
            // speed cap's 0.99 decay.
            let (nw, nh) = n.size();
            if p.vel.length() > ((nw as i32 + nh as i32) / 3) as f32 {
                p.vel *= 0.99;
            }
        }
    }
    // `:32874-32881` — the stuck fuse (every non-free state).
    if p.ai[0] != 0.0 {
        p.ai[1] += 1.0;
        if p.ai[1] > 90.0 {
            p.kill_time_left = p.time_left;
            p.active = false;
            p.time_left = 0;
        }
    }
}

/// The nail's movement: `HandleMovement`'s blocked-move aiStyle-93 arm
/// (`Projectile.cs:18819-18832`) — a blocked move re-arms the wall-stuck
/// state (`ai[0]=1`, `ai[1]=0`, tileCollide off, the position steps once,
/// the velocity restores to the pre-collision direction at speed 3). The
/// collision box is the 4×4 `GetCollisionParams` slice (`:19962-19966`).
/// `UpdatePosition` (`:20435`) SKIPS the integrate while stuck
/// (`aiStyle == 93 && ai[0] < 0`); the free/wall states integrate.
pub(crate) fn nail_move(p: &mut Projectile, world: &World) {
    if p.tile_collide {
        let last = p.vel;
        let off = Vec2::new(p.w * 0.5 - 2.0, p.h * 0.5 - 2.0);
        let tc = crate::collision::tile_collision(
            world,
            [p.pos.x + off.x, p.pos.y + off.y],
            [p.vel.x, p.vel.y],
            4.0,
            4.0,
            true,
            true,
        );
        p.vel = Vec2::new(tc.vel[0], tc.vel[1]);
        if p.vel != last {
            // `:18821-18831` — the wall-stuck re-arm.
            p.ai[1] = 0.0;
            p.ai[0] = 1.0;
            p.net_update = true;
            p.tile_collide = false;
            p.pos += p.vel;
            p.vel = safe_normalize(last) * 3.0;
        }
    }
    // `UpdatePosition` (`:20435-20446`): the stuck nail holds position.
    if !(p.ai[0] < 0.0) {
        p.pos += p.vel;
    }
    // `AutomaticallyChangesDirection` (`:17283` — 93 not excluded).
    p.direction = if p.vel.x < 0.0 { -1 } else { 1 };
}

/// The nail's `Kill()` blast (`Projectile.cs:76343-76375`): the Item14
/// cue, the 10×dust-31 + 6×(dust-6 pair + the `Next(2)` third) burst —
/// the dust bodies are the pool-wide kill-dust skip, the SIX explicit
/// `Next(2)` draws stay (the empress kill-dust census precedent keeps the
/// call-site draws). The local-owner arm resizes to 112×112 centred,
/// `penetrate = -1`, `ai[0] = 2` and runs `Damage()` through the box (the
/// pool's `damage_pve_boxed` seam — the caller runs it when this returns
/// true). Returns true when the resize fired.
pub(crate) fn nail_kill(p: &mut Projectile, rng: &mut dyn DropRng, local_owner: u8) -> bool {
    if p.kind != 514 {
        return false;
    }
    // `:76345` — the Item14 cue at base.position.
    p.wants_sound = Some((2, 14, p.pos.x, p.pos.y, 0.0));
    // `:76346-76363` — the dust bursts: the bodies skip, the 6 Next(2)
    // gates draw in order.
    for _ in 0..6 {
        let _ = rng.next_range(0, 2);
    }
    // `:76364-76375` — the owner-gated blast resize (the pool simulates
    // local-owned slots; the gate stays explicit for the 255 hostile case).
    if p.owner != local_owner {
        return false;
    }
    p.penetrate = -1;
    let cx = p.pos.x + (p.w as i32 / 2) as f32;
    let cy = p.pos.y + (p.h as i32 / 2) as f32;
    p.w = 112.0;
    p.h = 112.0;
    p.pos = Vec2::new(cx - 56.0, cy - 56.0);
    p.ai[0] = 2.0;
    true
}

/// `AI()`'s aiStyle-94 arm (`Projectile.cs:32883-32963`) — the Coin Gun's
/// portal (518). The 4-tick frame machine (wraps at `projFrames[518]` = 4,
/// `Main.cs:8486`), the five-phase age machine: 0-40 the fade-in (alpha
/// −5 floor 0, the 0.85 decay, the tick-40 `Next(3)` cadence pick
/// 10/15/30), 41-60 the hold (velocity zero), 61-210 the coin fountain
/// (every `ai[1]` ticks one gold coin 73 with the explicit
/// `RotatedByRandom(2π)` velocity — TWO NextDouble + one NextFloat, the
/// drop request rides `wants_coin_drop`), 211+ the fade-out (scale
/// −1/30, alpha +15, the 240 kill). The tail dust gate (`Next(3)` +
/// the taken branch's `RotatedByRandom(2π)` pair) runs every tick; the
/// `Lighting.AddLight` (`:32962`) is the light-layer skip.
pub(crate) fn ai_coin_portal(p: &mut Projectile, rng: &mut dyn DropRng) {
    // `:32885-32892` — the frame machine.
    p.frame_counter += 1;
    if p.frame_counter >= 4 {
        p.frame_counter = 0;
        p.frame += 1;
        if p.frame >= proj_frames(p.kind) as i32 {
            p.frame = 0;
        }
    }
    // `:32893-32952` — the age machine.
    p.ai[0] += 1.0;
    if p.ai[0] <= 40.0 {
        p.alpha -= 5;
        if p.alpha < 0 {
            p.alpha = 0;
        }
        p.vel *= 0.85;
        if p.ai[0] == 40.0 {
            p.net_update = true;
            // `:32905-32916` — the cadence pick.
            match rng.next_range(0, 3) {
                0 => p.ai[1] = 10.0,
                1 => p.ai[1] = 15.0,
                2 => p.ai[1] = 30.0,
                _ => {}
            }
        }
    } else if p.ai[0] <= 60.0 {
        p.vel = Vec2::ZERO;
        if p.ai[0] == 60.0 {
            p.net_update = true;
        }
    } else if p.ai[0] <= 210.0 {
        // `:32929-32934` — the coin fountain (`netMode != 1` folds open).
        p.local_ai[0] += 1.0;
        if p.local_ai[0] >= p.ai[1] {
            p.local_ai[0] = 0.0;
            // `Item.NewItem(..., (int)Center.X, (int)Center.Y, 0, 0, 73)`
            // — the 0×0 box's centre is the truncated slot centre
            // (`Item.cs:49651`). The explicit velocity:
            // `UnitY.RotatedByRandom(2π) * (3,2) * (NextFloat*0.5+0.5)
            //  - UnitY` — TWO NextDouble (the rotation) then ONE NextFloat
            // (the speed), the C#'s left-to-right operand order.
            let (cx, cy) = p.center();
            let a = rng.next_float() as f64 * std::f64::consts::TAU
                - rng.next_float() as f64 * std::f64::consts::TAU;
            let speed = rng.next_float() * 0.5 + 0.5;
            let v = crate::projectile::rotate(Vec2::Y, a as f32)
                * Vec2::new(3.0, 2.0)
                * speed
                - Vec2::Y;
            p.wants_coin_drop = Some((cx as i32 as f32, cy as i32 as f32, v.x, v.y));
        }
        if p.ai[0] == 210.0 {
            p.net_update = true;
        }
    } else {
        // `:32940-32952` — the fade-out.
        p.scale -= 1.0 / 30.0;
        p.alpha += 15;
        if p.ai[0] == 239.0 {
            p.net_update = true;
        }
        if p.ai[0] == 240.0 {
            p.kill_time_left = p.time_left;
            p.active = false;
            p.time_left = 0;
        }
    }
    // `:32953-32961` — the tail dust gate: the Next(3) ALWAYS draws; the
    // taken branch's RotatedByRandom(2π) pair draws too (the dust body
    // and its position/velocity writes are the dust-layer skip).
    if p.alpha < 90 && rng.next_range(0, 3) == 0 {
        let _ = (rng.next_float(), rng.next_float()); // RotatedByRandom
    }
    // `:32962` — Lighting.AddLight: the light-layer skip (module doc).
}

/// `AI()`'s aiStyle-95 arm (`Projectile.cs:32964-33003`) — the Toxic
/// Flask's bubble (523). The 3-tick fade-in (localAI[0] counts to 2, then
/// alpha −20 floors at 100), the 30-tick launch hold then the −0.05 rise
/// (cap −8) + 0.98 X decay, the vx·0.1 tilt, the wet arm's 0.98 fall decay
/// + −0.2 buoyancy + 0.94 X decay. ZERO draws — the arm is pure maths.
/// The `wet` read is the port's one-tick-lag snapshot (the bobber
/// precedent, ANALYSIS "Fishing rod").
pub(crate) fn ai_toxic_bubble(p: &mut Projectile) {
    // `:32966-32977` — the fade-in.
    if p.local_ai[0] > 2.0 {
        p.alpha -= 20;
        if p.alpha < 100 {
            p.alpha = 100;
        }
    } else {
        p.local_ai[0] += 1.0;
    }
    // `:32978-32989` — the launch hold then the rise.
    if p.ai[0] > 30.0 {
        if p.vel.y > -8.0 {
            p.vel.y -= 0.05;
        }
        p.vel.x *= 0.98;
    } else {
        p.ai[0] += 1.0;
    }
    // `:32990` — the tilt.
    p.rot = p.vel.x * 0.1;
    // `:32991-33002` — the wet arm.
    if p.wet {
        if p.vel.y > 0.0 {
            p.vel.y *= 0.98;
        }
        if p.vel.y > -8.0 {
            p.vel.y -= 0.2;
        }
        p.vel.x *= 0.94;
    }
}

/// The bubble's movement: no per-type `HandleMovement` arm exists for the
/// 523 — the generic blocked-move default (`Projectile.cs:19849-19853`:
/// `position += velocity; Kill()`) serves it, over the 10×10 centered
/// `GetCollisionParams` box (`:19957-19961`). The port's blocked test is
/// the destination-box probe (the `kill_step` convention — the swept
/// `TileCollision`'s exact death pixel is the recorded stand-in). The
/// free move integrates (`UpdatePosition` `:20445` — the 523 is not on
/// the skip list).
pub(crate) fn toxic_bubble_move(p: &mut Projectile, world: &World) {
    if !p.tile_collide {
        p.pos += p.vel;
        return;
    }
    let off = Vec2::new(p.w * 0.5 - 5.0, p.h * 0.5 - 5.0);
    let nx = p.pos.x + off.x + p.vel.x;
    let ny = p.pos.y + off.y + p.vel.y;
    let hit_x = crate::player::collides_rect(world, nx, p.pos.y + off.y, 10.0, 10.0);
    let hit_y = crate::player::collides_rect(world, p.pos.x + off.x, ny, 10.0, 10.0);
    p.pos += p.vel;
    if hit_x || hit_y {
        p.kill_time_left = p.time_left;
        p.active = false;
        p.time_left = 0;
    }
    p.direction = if p.vel.x < 0.0 { -1 } else { 1 };
}

/// The bubble's `Kill()` arm (`Projectile.cs:73257-73269`): the Item54
/// cue + the 25-dust burst — each dust's velocity is TWO explicit
/// `Next(-100, 101)` draws + ONE `Next(1, 30)` speed (the call-site draws
/// stay, the dust bodies skip — the fairy-queen/celeb2 kill precedent).
/// 75 draws per death, in the C#'s per-dust X-Y-speed order.
pub(crate) fn toxic_bubble_kill(p: &mut Projectile, rng: &mut dyn DropRng) {
    if p.kind != 523 {
        return;
    }
    p.wants_sound = Some((2, 54, p.pos.x, p.pos.y, 0.0));
    for _ in 0..25 {
        let _ = rng.next_range(-100, 101); // velocity.X
        let _ = rng.next_range(-100, 101); // velocity.Y
        let _ = rng.next_range(1, 30); // the speed
    }
}

/// `AI()`'s aiStyle-96 arm (`Projectile.cs:33004-33028`) — the Ichor
/// Flask's splash (524). The 0.6 age counter with the 500 kill, the
/// 2-iteration dust gate (each `Next(3) != 0` — the draw ALWAYS runs, the
/// taken branch's dust body skips), the 0.008 gravity. The dust's
/// `(800 - ai[0]) / 800 + 0.1` scale feeds only the skipped body.
pub(crate) fn ai_ichor_splash(p: &mut Projectile, rng: &mut dyn DropRng) {
    // `:33006-33010` — the age + kill.
    p.ai[0] += 0.6;
    if p.ai[0] > 500.0 {
        p.kill_time_left = p.time_left;
        p.active = false;
        p.time_left = 0;
    }
    // `:33011-33026` — the 2-iteration dust gate.
    for _ in 0..2 {
        if rng.next_range(0, 3) != 0 {
            // The dust body (position/velocity/scale writes) is the
            // dust-layer skip; no call-site draws inside.
        }
    }
    // `:33027` — the gravity.
    p.vel.y += 0.008;
}

/// The splash's `HandleMovement` blocked-move arm
/// (`Projectile.cs:18807-18818`): a blocked move ages `ai[0] += 100` and
/// reverses BOTH axes off the pre-collision velocity. The free move
/// integrates; the collision box is the slot's own 10×10 (no
/// `GetCollisionParams` override for 524).
pub(crate) fn ichor_splash_move(p: &mut Projectile, world: &World) {
    if p.tile_collide {
        let last = p.vel;
        let tc = crate::collision::tile_collision(
            world,
            [p.pos.x, p.pos.y],
            [p.vel.x, p.vel.y],
            p.w,
            p.h,
            true,
            true,
        );
        p.vel = Vec2::new(tc.vel[0], tc.vel[1]);
        if p.vel != last {
            p.ai[0] += 100.0;
            if p.vel.x != last.x {
                p.vel.x = -last.x;
            }
            if p.vel.y != last.y {
                p.vel.y = -last.y;
            }
        }
    }
    p.pos += p.vel;
    p.direction = if p.vel.x < 0.0 { -1 } else { 1 };
}

/// `AI()`'s aiStyle-97 arm (`Projectile.cs:33029-33133`) — the Money
/// Trough's flying piggy bank (525). The 4-tick frame ladder
/// (0,1,2,3,4,3,2,1 then the reset), the local-owner same-type dedup
/// (the higher-timeLeft twin kills the other — the C# `>=` keeps the
/// NEWER on a tie), the throw decay (0.94, the velocity-sign facing) then
/// the rest state's owner-facing hover with the ±0.005 bob and the 90
/// flip. `CurrentFrameFlags.HadAnActiveInteractableProjectile` (`:33070`)
/// and the container UI (`TryGetContainerIndex` `:78815`) have no port
/// consumer. ZERO draws. Pool-side: the dedup kills sibling slots.
pub(crate) fn ai_flying_piggy(
    slots: &mut [Projectile],
    i: usize,
    owner_center: Option<(f32, f32)>,
    local_owner: u8,
) {
    let p = &mut slots[i];
    // `:33031-33069` — the frame ladder.
    p.frame_counter += 1;
    let fc = p.frame_counter as f32;
    let num777 = 4.0f32;
    if fc < num777 {
        p.frame = 0;
    } else if fc < num777 * 2.0 {
        p.frame = 1;
    } else if fc < num777 * 3.0 {
        p.frame = 2;
    } else if fc < num777 * 4.0 {
        p.frame = 3;
    } else if fc < num777 * 5.0 {
        p.frame = 4;
    } else if fc < num777 * 6.0 {
        p.frame = 3;
    } else if fc < num777 * 7.0 {
        p.frame = 2;
    } else if fc < num777 * 8.0 {
        p.frame = 1;
    } else {
        p.frame_counter = 0;
        p.frame = 0;
    }
    // `:33071-33087` — the local-owner same-type dedup. The C# kills
    // INLINE mid-scan: a self-kill zeroes `this.timeLeft`, so every later
    // comparison reads 0 (no further sibling dies — the `>=` fails
    // against a live sibling's positive timeLeft). The port keeps that
    // order with the index loop.
    if slots[i].owner == local_owner {
        let (owner, kind) = (slots[i].owner, slots[i].kind);
        for j in 0..slots.len() {
            if j == i {
                continue;
            }
            let (active, o_owner, o_kind, o_tl) = {
                let o = &slots[j];
                (o.active, o.owner, o.kind, o.time_left)
            };
            if active && o_owner == owner && o_kind == kind {
                if slots[i].time_left >= o_tl {
                    slots[j].kill_time_left = slots[j].time_left;
                    slots[j].active = false;
                    slots[j].time_left = 0;
                } else if slots[i].active {
                    // `Kill()`'s `!active` head (`:70644`): a second
                    // self-kill is a no-op (the first zeroed timeLeft).
                    let p = &mut slots[i];
                    p.kill_time_left = p.time_left;
                    p.active = false;
                    p.time_left = 0;
                }
            }
        }
    }
    let p = &mut slots[i];
    // `:33088-33109` — the throw state: the 0.94 decay until the velocity
    // dies below 0.1, then the rest latch (ai[0]=1, ai[1]=45).
    if p.ai[0] == 0.0 {
        if p.vel.length() < 0.1 {
            p.vel = Vec2::ZERO;
            p.ai[0] = 1.0;
            p.ai[1] = 45.0;
            return;
        }
        p.vel *= 0.94;
        p.direction = if p.vel.x < 0.0 { -1 } else { 1 };
        p.sprite_direction = p.direction;
        return;
    }
    // `:33110-33118` — the rest state faces the owner.
    let (bcx, _) = p.center();
    if let Some((ox, _)) = owner_center {
        p.direction = if ox < bcx { -1 } else { 1 };
    }
    p.sprite_direction = p.direction;
    // `:33119-33132` — the bob: ai[1] counts up, the sign picks the
    // ±0.005 drift, the 90 boundary flips the counter negative.
    p.ai[1] += 1.0;
    if p.ai[1] > 0.0 {
        p.vel.y -= 0.005;
    } else {
        p.vel.y += 0.005;
    }
    if p.ai[1] >= 90.0 {
        p.ai[1] *= -1.0;
    }
}

/// `AI()`'s aiStyle-98 arm (`Projectile.cs:33134-33152`) — the Lunatic
/// Cultist's homing particle (526). The `(ai[0], ai[1])` target point: if
/// the distance is shorter than the current speed the particle KILLS
/// (the arrival), else the velocity lerps 10% toward the 15-speed seek.
/// The 2-dust trail: each dust's scale draws ONE `NextFloat` (`:33150`)
/// — the bodies skip. The spawner (`NPC.cs:36863`/`:36874`/`:36885`)
/// packs the target into ai0/ai1.
pub(crate) fn ai_cultist_particle(p: &mut Projectile, rng: &mut dyn DropRng) {
    // `:33136-33141` — the arrival kill.
    let center = Vec2::from(p.center());
    let mut value7 = Vec2::new(p.ai[0], p.ai[1]) - center;
    if value7.length() < p.vel.length() {
        p.kill_time_left = p.time_left;
        p.active = false;
        p.time_left = 0;
        return;
    }
    // `:33142-33144` — the 15-speed seek, the 0.1 lerp.
    value7 = safe_normalize(value7) * 15.0;
    p.vel = p.vel.lerp(value7, 0.1);
    // `:33145-33151` — the 2-dust trail: ONE NextFloat scale per dust.
    for _ in 0..2 {
        let _ = rng.next_float();
    }
}

// Witnesses (W4-d10 code-only round — WRITTEN NOT RUN).

#[cfg(test)]
mod tests {
    use super::*;
    use crate::projectile::proj_data;

    /// A draw-recording stream (the `projectile_ai_misc` CensusRng shape).
    #[derive(Default)]
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

    fn proj(kind: u16) -> Projectile {
        let d = proj_data(kind).expect("the rowed kind");
        let mut p = Projectile::default();
        p.active = true;
        p.kind = kind;
        p.owner = 0;
        p.pos = Vec2::new(1600.0, 800.0);
        p.w = d.w;
        p.h = d.h;
        p.tile_collide = d.tile_collide;
        p.alpha = d.alpha;
        p.scale = d.scale;
        p.time_left = d.time_left;
        p.penetrate = d.penetrate;
        p.max_penetrate = d.penetrate;
        p.extra_updates = d.extra_updates;
        p
    }

    /// The 496 row (`Projectile.cs:5692-5704`): 40×40, aiStyle 91, magic,
    /// `MaxUpdates = 3` (extraUpdates 2), penetrate 3, the local-immunity
    /// −1 pair, alpha 255.
    #[test]
    fn w4_d10_s91_the_shadowflame_row_is_verbatim() {
        let d = proj_data(496).unwrap();
        assert_eq!((d.w, d.h), (40.0, 40.0));
        assert_eq!(d.ai_style, 91);
        assert_eq!(d.penetrate, 3);
        assert_eq!(d.extra_updates, 2, "MaxUpdates = 3 (`:5700`)");
        assert!(d.uses_local_immunity);
        assert_eq!(d.local_hit_cooldown, -1);
        assert_eq!(d.alpha, 255);
        assert!(d.magic);
        assert!(d.tile_collide, "the stock true (`:775`)");
    }

    /// The shrink keeps the center (`:32517-32522`): tick 1 scale 1 → box
    /// 20, the ramp 0.01; the box is int-truncated and re-anchored.
    #[test]
    fn w4_d10_s91_the_shrink_keeps_the_center_and_ramps() {
        let mut p = proj(496);
        let c0 = Vec2::from(p.center());
        ai_shadowflame(&mut p);
        assert_eq!(p.scale, 1.0);
        assert_eq!((p.w, p.h), (20.0, 20.0));
        assert_eq!(Vec2::from(p.center()), c0, "the center latch");
        assert_eq!(p.local_ai[0], 0.01);
        // The ramp switches at 0.1 (`:32523` — strict <).
        p.local_ai[0] = 0.1;
        ai_shadowflame(&mut p);
        assert_eq!(p.local_ai[0], 0.125, "0.1 is NOT < 0.1 — the 0.025 arm");
        assert!((p.scale - 0.9).abs() < 1e-6);
        assert_eq!(p.w, 18.0, "(int)(20 * 0.9)");
    }

    /// The 0.95 kill (`:32531-32534`) and the ai accel/cap/growth
    /// (`:32535-32543`). ZERO rand draws.
    #[test]
    fn w4_d10_s91_the_fade_kills_at_095_and_the_ai_grows() {
        let mut p = proj(496);
        p.local_ai[0] = 0.94;
        p.ai = [1.0, 0.0, 0.0, 0.0];
        ai_shadowflame(&mut p);
        // 0.94 >= 0.1 → +0.025 → 0.965 >= 0.95 → dead.
        assert!(!p.active);
        assert_eq!(p.time_left, 0);
        assert!((p.vel.x - 1.5).abs() < 1e-6, "the 1.5 accel");
        assert!((p.ai[0] - 1.05).abs() < 1e-6, "the 1.05 growth");
        // The 16 cap normalizes.
        let mut q = proj(496);
        q.ai = [20.0, 0.0, 0.0, 0.0];
        ai_shadowflame(&mut q);
        assert!((q.vel.length() - 16.0).abs() < 1e-4);
    }

    /// The blocked-move arm (`:18710-18735`): a soft block reverses, a
    /// hard block kills. Open air keeps the velocity.
    #[test]
    fn w4_d10_s91_the_bounce_reverses_soft_and_kills_hard() {
        let w = World::new("t".into(), 300, 200);
        let mut p = proj(496);
        p.vel = Vec2::new(4.0, 0.0);
        let before = p.pos;
        shadowflame_move(&mut p, &w);
        assert_eq!(p.pos, before + Vec2::new(4.0, 0.0), "the free integrate");
        assert!(p.active);
        assert_eq!(p.direction, 1);
        // A wall one pixel ahead: the 10×10 box's right edge at 1620+...
        // place the solid so the move blocks with |v| >= 1 → the kill.
        let mut w2 = World::new("t".into(), 300, 200);
        let tx = ((p.pos.x + p.w * 0.5 + 5.0 + 4.0) / 16.0) as i32;
        let ty = (p.pos.y / 16.0) as i32;
        let i = w2.index(tx, ty);
        w2.tiles[i] = tr_core::tile::Tile { kind: 1, ..Default::default() };
        w2.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
        let mut q = proj(496);
        q.pos = p.pos;
        q.vel = Vec2::new(4.0, 0.0);
        shadowflame_move(&mut q, &w2);
        assert!(!q.active, "the hard block kills (`:18720`)");
    }

    /// The style-92 rows (`Projectile.cs:5805-5840`, `:9558-9569`,
    /// `:10565-10579`): the boxes, the shared 511 identity, the gas trap's
    /// trap flag, the spore cloud's idStatic-30 + melee + tileCollide false.
    #[test]
    fn w4_d10_s92_the_five_rows_are_verbatim() {
        for (kind, w, h) in [(511u16, 32.0f32, 32.0f32), (512, 40.0, 38.0), (513, 30.0, 28.0)] {
            let d = proj_data(kind).unwrap();
            assert_eq!((d.w, d.h), (w, h), "kind {kind}");
            assert_eq!(d.ai_style, 92);
            assert_eq!(d.penetrate, -1);
            assert_eq!(d.scale, 1.1);
            assert_eq!(d.alpha, 255);
            assert!(d.magic);
            assert!(d.uses_id_static_immunity);
            assert_eq!(d.id_static_hit_cooldown, 10);
            assert_eq!(d.immunity_identity, 511, "UseGroupStaticImmunity(511)");
        }
        let g = proj_data(1007).unwrap();
        assert_eq!((g.w, g.h), (50.0, 50.0));
        assert_eq!(g.ai_style, 92);
        assert!(g.trap);
        assert_eq!(g.scale, 1.1);
        let s = proj_data(1128).unwrap();
        assert_eq!((s.w, s.h), (30.0, 30.0));
        assert!(!s.tile_collide);
        assert!(s.melee);
        assert!(s.uses_id_static_immunity);
        assert_eq!(s.id_static_hit_cooldown, 30);
        assert_eq!(crate::projectile::proj_frames(1128), 5, "Main.cs:8726");
    }

    /// The toxic cloud's age/opacity/decay (`:32580-32637`): tick 1 age 1,
    /// the opacity ramp, the 0.96 decay, the whoAmI-seeded rotation. The
    /// hit counter doubles the aging. ZERO draws.
    #[test]
    fn w4_d10_s92_the_cloud_ages_and_fades() {
        let w = World::new("t".into(), 300, 200);
        let mut slots = vec![Projectile::default(); 4];
        slots[2] = proj(511);
        slots[2].vel = Vec2::new(2.0, 0.0);
        let mut rng = CensusRng::default();
        ai_clouds(&mut slots, 2, &w, &mut [], None, 0, &mut rng);
        let p = &slots[2];
        assert_eq!(p.ai[0], 1.0);
        assert!(!p.tile_collide, ":32567");
        assert!((p.vel.x - 2.0 * 0.96).abs() < 1e-6, "the 0.96 decay");
        // Opacity = remap(1/260, 0, .3, 0, 1) * remap(1/260, .3, 1, 1, 0)
        // * 0.7 ≈ 0.00897 → alpha = (int)((1-o)*255) = 252.
        assert_eq!(p.alpha, 252);
        assert!((p.rot - (2.0 * 0.4002029 + 1.0 * std::f32::consts::PI * 2.0 / 480.0)).abs() < 1e-5);
        assert!(rng.ranges.is_empty() && rng.floats == 0, "the clouds draw nothing");
        // The hit-armed cloud ages 3 per tick (`:32581-32584`).
        slots[2].ai[1] = 1.0;
        ai_clouds(&mut slots, 2, &w, &mut [], None, 0, &mut rng);
        assert_eq!(slots[2].ai[0], 4.0);
    }

    /// The gas trap (`:32585-32609`, `:32788-32819`): the 80-tick life, the
    /// 1→6 scale ramp with the center latch, the zero velocity, the Item16
    /// cue on the first tick, the 15-tick Poisoned cloud. The kill pins
    /// ai[0] AFTER the death (`:32596-32597`).
    #[test]
    fn w4_d10_s92_the_gas_trap_grows_and_poisons() {
        let w = World::new("t".into(), 300, 200);
        let mut slots = vec![Projectile::default(); 1];
        slots[0] = proj(1007);
        let c0 = Vec2::from(slots[0].center());
        let mut rng = CensusRng::default();
        let mut npcs = [crate::npc::Npc::new(crate::npc::NpcKind::Zombie, c0.x + 10.0, c0.y, 1, 0.0)];
        npcs[0].hp = 40;
        npcs[0].max_hp = 40;
        ai_clouds(&mut slots, 0, &w, &mut npcs, None, 0, &mut rng);
        let p = &slots[0];
        assert_eq!(p.wants_sound, Some((2, 16, c0.x, c0.y, 0.0)), "the broadcast cue");
        assert_eq!(p.vel, Vec2::ZERO);
        assert!(p.scale > 1.0, "the 1→6 ramp");
        assert_eq!(p.local_ai[1], 15.0, "the cadence re-arm");
        assert!(npcs[0].buffs.buff_type.contains(&20), "the Poisoned cloud");
        // The 80-tick kill: ai[0] 81 > 80 → dead, ai[0] pinned to 80.
        slots[0].ai[0] = 80.0;
        slots[0].local_ai[1] = 5.0;
        ai_clouds(&mut slots, 0, &w, &mut npcs, None, 0, &mut rng);
        assert!(!slots[0].active);
        assert_eq!(slots[0].ai[0], 80.0);
    }

    /// The spore cloud's first-tick frame roll (`:32563-32566`) draws ONE
    /// Next(5); later ticks draw nothing. The 220/ai[1] lifetime split.
    #[test]
    fn w4_d10_s92_the_spore_cloud_rolls_its_frame_once() {
        let w = World::new("t".into(), 300, 200);
        let mut slots = vec![Projectile::default(); 1];
        slots[0] = proj(1128);
        let mut rng = CensusRng::default();
        ai_clouds(&mut slots, 0, &w, &mut [], None, 0, &mut rng);
        assert_eq!(rng.ranges, vec![(0, 5)], "Next(projFrames[1128])");
        assert_eq!(slots[0].frame, 0, "CensusRng returns min");
        // tick 1: fromValue 1/220 = 0.004545 < 0.1 → the ramp clamps at
        // 0.25 (`:32612`), the box (int)(30*0.25/1.1) = 6.
        assert!((slots[0].scale - 0.25).abs() < 1e-6, "the ramp's clamped start");
        assert_eq!(slots[0].w, 6.0);
        // The hit-armed lifetime reads ai[1] (`:32592`).
        slots[0].ai[1] = 100.0;
        slots[0].ai[0] = 99.0;
        ai_clouds(&mut slots, 0, &w, &mut [], None, 0, &mut rng);
        // age 100 + 2 = 102 > 100 → the kill.
        assert!(!slots[0].active);
        assert_eq!(slots[0].ai[0], 100.0);
        assert_eq!(rng.ranges.len(), 1, "no second frame roll");
    }

    /// The separation scans (`:32638-32705`): two overlapping toxic clouds
    /// push apart symmetrically (the 0.6 Lerp on both); the zero-delta
    /// tie-break reads the index order.
    #[test]
    fn w4_d10_s92_the_clouds_separate() {
        let w = World::new("t".into(), 300, 200);
        let mut slots = vec![Projectile::default(); 2];
        slots[0] = proj(511);
        slots[1] = proj(511);
        slots[1].pos = slots[0].pos + Vec2::new(8.0, 0.0);
        let mut rng = CensusRng::default();
        ai_clouds(&mut slots, 0, &w, &mut [], None, 0, &mut rng);
        assert!(slots[0].vel.x < 0.0, "the left cloud pushes left");
        assert!(slots[1].vel.x > 0.0, "the sibling got the opposite push");
        // Exact overlap: the index tie-break (`:32653-32665`).
        slots[1].pos = slots[0].pos;
        slots[0].vel = Vec2::ZERO;
        slots[1].vel = Vec2::ZERO;
        ai_clouds(&mut slots, 0, &w, &mut [], None, 0, &mut rng);
        assert!(slots[0].vel.x > 0.0 && slots[0].vel.y < 0.0, "j>i → (1,-1)");
    }

    /// The 514 row (`Projectile.cs:5841-5850`): 10×10, aiStyle 93,
    /// penetrate 3, alpha 255, ranged, the stock tileCollide/timeLeft.
    #[test]
    fn w4_d10_s93_the_nail_row_is_verbatim() {
        let d = proj_data(514).unwrap();
        assert_eq!((d.w, d.h), (10.0, 10.0));
        assert_eq!(d.ai_style, 93);
        assert_eq!(d.penetrate, 3);
        assert_eq!(d.alpha, 255);
        assert!(d.ranged);
        assert!(d.tile_collide);
        assert_eq!(d.time_left, 3600);
    }

    /// The free flight (`:32823-32845`): the alpha fade floors at 0, the
    /// 20-tick hover holds the velocity, tick 21 adds the 0.1 gravity +
    /// 0.992 X decay, the tilt is atan2+1.57. ZERO draws.
    #[test]
    fn w4_d10_s93_the_nail_hovers_then_falls() {
        let mut p = proj(514);
        p.vel = Vec2::new(5.0, 0.0);
        let mut rng = CensusRng::default();
        for _ in 0..20 {
            ai_nail(&mut p, &[], &mut rng);
        }
        assert_eq!(p.ai[1], 20.0);
        assert_eq!(p.vel, Vec2::new(5.0, 0.0), "the hover holds");
        assert_eq!(p.alpha, 0, "255 - 25*11 floors at 0");
        ai_nail(&mut p, &[], &mut rng);
        assert!((p.vel.y - 0.1).abs() < 1e-6);
        assert!((p.vel.x - 5.0 * 0.992).abs() < 1e-5);
        assert!((p.rot - (0.1f32.atan2(5.0 * 0.992) + 1.57)).abs() < 1e-5);
        assert!(rng.ranges.is_empty() && rng.floats == 0);
        // The 18 fall cap.
        p.vel.y = 20.0;
        ai_nail(&mut p, &[], &mut rng);
        assert!(p.vel.y <= 18.1, "the cap precedes the gravity add");
    }

    /// The NPC anchor + release (`:32852-32873`): the stuck nail rides the
    /// victim's center; the victim's death releases with TWO Next(-100,101)
    /// draws and the 6-speed normalize; the 90-tick fuse kills.
    #[test]
    fn w4_d10_s93_the_nail_sticks_releases_and_fuses() {
        let mut n = crate::npc::Npc::new(crate::npc::NpcKind::Zombie, 2000.0, 900.0, 1, 0.0);
        n.hp = 40;
        n.max_hp = 40;
        let mut p = proj(514);
        p.ai[0] = -1.0; // victim index 0
        p.vel = Vec2::new(2.0, 0.0);
        let mut rng = CensusRng::default();
        ai_nail(&mut p, &[n.clone()], &mut rng);
        let (ncx, ncy) = n.center();
        assert_eq!(p.pos.x, ncx - 2.0 - 5.0, "center - velocity - width/2");
        assert_eq!(p.pos.y, ncy - 5.0);
        assert!(!p.tile_collide);
        assert_eq!(p.ai[1], 1.0, "the fuse ticks");
        assert!(rng.ranges.is_empty(), "the live anchor draws nothing");
        // The death release: two draws, the 6-speed kick, the free re-arm.
        n.hp = 0;
        ai_nail(&mut p, &[n], &mut rng);
        assert_eq!(rng.ranges, vec![(-100, 101), (-100, 101)]);
        assert_eq!(p.ai[0], 0.0);
        assert_eq!(p.ai[1], 20.0);
        assert!(p.tile_collide);
        assert!(p.net_update);
        // CensusRng returns min: (-100,-100) normalized * 6.
        assert!((p.vel.length() - 6.0).abs() < 1e-4);
        // The 90-tick fuse: ai[1] 91 > 90 → dead.
        let mut q = proj(514);
        q.ai[0] = 1.0;
        q.ai[1] = 90.0;
        ai_nail(&mut q, &[], &mut rng);
        assert!(!q.active, "the wall-stuck fuse kills at 91");
    }

    /// The wall-stuck movement (`:18819-18832`): a blocked move re-arms
    /// ai[0]=1 with the 3-speed restore; the stuck nail's UpdatePosition
    /// skip (`:20435`) holds the position.
    #[test]
    fn w4_d10_s93_the_wall_block_sticks_the_nail() {
        let mut w = World::new("t".into(), 300, 200);
        let mut p = proj(514);
        p.vel = Vec2::new(8.0, 0.0);
        // A wall directly ahead of the 4×4 probe box.
        let tx = ((p.pos.x + p.w * 0.5 + 2.0 + 8.0) / 16.0) as i32;
        let ty = ((p.pos.y + p.h * 0.5) / 16.0) as i32;
        let i = w.index(tx, ty);
        w.tiles[i] = tr_core::tile::Tile { kind: 1, ..Default::default() };
        w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
        nail_move(&mut p, &w);
        assert_eq!(p.ai[0], 1.0, "the wall-stuck re-arm");
        assert_eq!(p.ai[1], 0.0);
        assert!(!p.tile_collide);
        assert!((p.vel.length() - 3.0).abs() < 1e-4, "the 3-speed restore");
        // The NPC-stuck skip: ai[0] < 0 holds the position.
        let mut q = proj(514);
        q.ai[0] = -1.0;
        q.vel = Vec2::new(4.0, 0.0);
        let before = q.pos;
        nail_move(&mut q, &w);
        assert_eq!(q.pos, before, ":20435 — the stuck nail holds");
    }

    /// The kill blast (`:76343-76375`): the Item14 cue, the SIX Next(2)
    /// draws, the 112×112 resize + ai[0]=2 + penetrate −1 (local owner).
    #[test]
    fn w4_d10_s93_the_nail_blast_resizes_and_draws_six() {
        let mut p = proj(514);
        let mut rng = CensusRng::default();
        assert!(nail_kill(&mut p, &mut rng, 0));
        assert_eq!(p.wants_sound, Some((2, 14, 1600.0, 800.0, 0.0)));
        assert_eq!(rng.ranges, vec![(0, 2); 6], "the six dust gates");
        assert_eq!((p.w, p.h), (112.0, 112.0));
        assert_eq!(p.ai[0], 2.0);
        assert_eq!(p.penetrate, -1);
        // The centre holds: 1600+5 → 1605, the new box 1605-56 = 1549.
        assert_eq!(p.pos, Vec2::new(1549.0, 749.0));
        // A foreign owner skips the resize (the draws still ran).
        let mut q = proj(514);
        q.owner = 1;
        assert!(!nail_kill(&mut q, &mut rng, 0));
        assert_eq!(q.w, 10.0);
    }

    /// The 518 row (`Projectile.cs:5879-5888`): 32×32, aiStyle 94, alpha
    /// 255, tileCollide false, ignoreWater, netImportant, the stock
    /// penetrate 1; the 4-frame sheet (`Main.cs:8486`).
    #[test]
    fn w4_d10_s94_the_portal_row_is_verbatim() {
        let d = proj_data(518).unwrap();
        assert_eq!((d.w, d.h), (32.0, 32.0));
        assert_eq!(d.ai_style, 94);
        assert_eq!(d.alpha, 255);
        assert!(!d.tile_collide);
        assert!(d.ignore_water);
        assert!(d.net_important);
        assert_eq!(d.penetrate, 1);
        assert_eq!(crate::projectile::proj_frames(518), 4);
    }

    /// The five-phase machine (`:32893-32952`): the fade-in decays 0.85,
    /// tick 40 draws ONE Next(3) for the cadence (CensusRng min → 10), the
    /// hold zeroes the velocity, the fountain drops a coin every ai[1]
    /// ticks with the 3-draw velocity, the 240 kill.
    #[test]
    fn w4_d10_s94_the_portal_phases_and_fountain() {
        let mut p = proj(518);
        p.vel = Vec2::new(4.0, 0.0);
        let mut rng = CensusRng::default();
        for _ in 0..40 {
            ai_coin_portal(&mut p, &mut rng);
        }
        assert_eq!(p.ai[0], 40.0);
        assert_eq!(p.alpha, 55, "255 - 5*40");
        assert!((p.vel.x - 4.0 * 0.85f32.powi(40)).abs() < 1e-4);
        assert_eq!(rng.ranges, vec![(0, 3)], "the tick-40 cadence pick");
        assert_eq!(p.ai[1], 10.0, "CensusRng min → case 0");
        assert_eq!(p.frame, 0, "40 ticks / 4 = 10 wraps");
        // The hold: 41-60 zeroes the velocity.
        ai_coin_portal(&mut p, &mut rng);
        assert_eq!(p.vel, Vec2::ZERO);
        // The fountain: tick 61 starts the counter; the 10th tick drops.
        for _ in 0..9 {
            ai_coin_portal(&mut p, &mut rng);
        }
        assert!(p.wants_coin_drop.is_none(), "9 ticks < the 10 cadence");
        ai_coin_portal(&mut p, &mut rng);
        let (x, y, vx, vy) = p.wants_coin_drop.expect("the 10th tick drops");
        assert_eq!((x, y), (1616.0, 816.0), "the truncated centre");
        // CensusRng floats are 0: angle 0, speed 0.5 → UnitY*(3,2)*0.5 - Y.
        assert!((vx - 0.0).abs() < 1e-6 && (vy - (1.0 - 1.0)).abs() < 1e-6);
        assert_eq!(rng.floats, 3, "TWO rotation + ONE speed");
        // The 240 kill: the scale shrinks 1/30 per tick past 210.
        p.ai[0] = 239.0;
        p.wants_coin_drop = None;
        ai_coin_portal(&mut p, &mut rng);
        assert!(!p.active);
        assert_eq!(p.time_left, 0);
        assert!(p.scale < 1.0);
    }

    /// The tail dust gate (`:32953-32961`): alpha < 90 draws Next(3) every
    /// tick; the taken branch adds the RotatedByRandom pair. alpha >= 90
    /// draws nothing.
    #[test]
    fn w4_d10_s94_the_tail_dust_gate_draws_in_order() {
        let mut p = proj(518);
        p.ai[0] = 100.0; // the hold phase — no other draws
        p.alpha = 200;
        let mut rng = CensusRng::default();
        ai_coin_portal(&mut p, &mut rng);
        assert!(rng.ranges.is_empty() && rng.floats == 0, "alpha >= 90 skips");
        p.alpha = 80;
        ai_coin_portal(&mut p, &mut rng);
        // CensusRng: Next(3) → 0 (taken) + the 2-float rotation pair.
        assert_eq!(rng.ranges, vec![(0, 3)]);
        assert_eq!(rng.floats, 2);
    }

    /// The 523 row (`Projectile.cs:5929-5938`): 32×32, aiStyle 95, alpha
    /// 255, ranged, penetrate 1, the stock tileCollide.
    #[test]
    fn w4_d10_s95_the_bubble_row_is_verbatim() {
        let d = proj_data(523).unwrap();
        assert_eq!((d.w, d.h), (32.0, 32.0));
        assert_eq!(d.ai_style, 95);
        assert_eq!(d.alpha, 255);
        assert!(d.ranged);
        assert_eq!(d.penetrate, 1);
        assert!(d.tile_collide);
    }

    /// The fade/rise machine (`:32966-32990`): the 3-tick counter precedes
    /// the fade (alpha floors at 100), the 30-tick hold then the −0.05
    /// rise capped at −8 with the 0.98 X decay, the vx·0.1 tilt. ZERO
    /// draws.
    #[test]
    fn w4_d10_s95_the_bubble_fades_and_rises() {
        let mut p = proj(523);
        p.vel = Vec2::new(4.0, 2.0);
        for _ in 0..3 {
            ai_toxic_bubble(&mut p);
        }
        assert_eq!(p.local_ai[0], 3.0);
        assert_eq!(p.alpha, 255, "the counter ticks precede the fade");
        ai_toxic_bubble(&mut p);
        assert_eq!(p.alpha, 235);
        // The fade floor.
        p.local_ai[0] = 10.0;
        p.alpha = 110;
        ai_toxic_bubble(&mut p);
        assert_eq!(p.alpha, 100);
        // The launch hold: ai[0] counts to 31 before the rise.
        let mut q = proj(523);
        q.vel = Vec2::new(4.0, 0.0);
        for _ in 0..31 {
            ai_toxic_bubble(&mut q);
        }
        assert_eq!(q.ai[0], 31.0);
        assert_eq!(q.vel.y, 0.0, "the hold");
        ai_toxic_bubble(&mut q);
        assert!((q.vel.y - (-0.05)).abs() < 1e-6);
        assert!((q.vel.x - 4.0 * 0.98).abs() < 1e-5);
        assert!((q.rot - q.vel.x * 0.1).abs() < 1e-6);
        // The −8 cap.
        q.vel.y = -7.99;
        ai_toxic_bubble(&mut q);
        assert!(q.vel.y >= -8.0);
        // The wet arm: the 0.98 fall decay + the −0.2 buoyancy + 0.94 X.
        let mut r = proj(523);
        r.ai[0] = 40.0;
        r.wet = true;
        r.vel = Vec2::new(2.0, 4.0);
        ai_toxic_bubble(&mut r);
        // rise first: 4 - 0.05 = 3.95; wet: 3.95*0.98 - 0.2 = 3.671.
        assert!((r.vel.y - (3.95 * 0.98 - 0.2)).abs() < 1e-5);
        assert!((r.vel.x - 2.0 * 0.98 * 0.94).abs() < 1e-5);
    }

    /// The kill burst (`:73257-73269`): the Item54 cue + 25×3 draws in
    /// X-Y-speed order; a foreign kind draws nothing.
    #[test]
    fn w4_d10_s95_the_bubble_death_draws_75() {
        let mut p = proj(523);
        let mut rng = CensusRng::default();
        toxic_bubble_kill(&mut p, &mut rng);
        assert_eq!(p.wants_sound, Some((2, 54, 1600.0, 800.0, 0.0)));
        assert_eq!(rng.ranges.len(), 75);
        assert_eq!(rng.ranges[0], (-100, 101));
        assert_eq!(rng.ranges[1], (-100, 101));
        assert_eq!(rng.ranges[2], (1, 30));
        let mut q = proj(496);
        let mut rng2 = CensusRng::default();
        toxic_bubble_kill(&mut q, &mut rng2);
        assert!(rng2.ranges.is_empty());
    }

    /// The movement: open air integrates; a wall kills (the generic
    /// blocked-move default, `:19849-19853`).
    #[test]
    fn w4_d10_s95_the_bubble_dies_on_tiles() {
        let w = World::new("t".into(), 300, 200);
        let mut p = proj(523);
        p.vel = Vec2::new(3.0, -1.0);
        let before = p.pos;
        toxic_bubble_move(&mut p, &w);
        assert_eq!(p.pos, before + Vec2::new(3.0, -1.0));
        assert!(p.active);
        let mut w2 = World::new("t".into(), 300, 200);
        let tx = ((p.pos.x + p.w * 0.5 + 5.0 + 3.0) / 16.0) as i32;
        let ty = ((p.pos.y + p.h * 0.5) / 16.0) as i32;
        let i = w2.index(tx, ty);
        w2.tiles[i] = tr_core::tile::Tile { kind: 1, ..Default::default() };
        w2.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
        let mut q = proj(523);
        q.vel = Vec2::new(3.0, 0.0);
        toxic_bubble_move(&mut q, &w2);
        assert!(!q.active, "the contact kill");
    }

    /// The 524 row (`Projectile.cs:5939-5952`): 10×10, aiStyle 96, alpha
    /// 255, penetrate 5, ignoreWater, melee, extraUpdates 5, idStatic-10.
    #[test]
    fn w4_d10_s96_the_splash_row_is_verbatim() {
        let d = proj_data(524).unwrap();
        assert_eq!((d.w, d.h), (10.0, 10.0));
        assert_eq!(d.ai_style, 96);
        assert_eq!(d.alpha, 255);
        assert_eq!(d.penetrate, 5);
        assert!(d.ignore_water);
        assert!(d.melee);
        assert_eq!(d.extra_updates, 5);
        assert!(d.uses_id_static_immunity);
        assert_eq!(d.id_static_hit_cooldown, 10);
    }

    /// The age/kill/dust machine (`:33006-33027`): +0.6 per sub-step, the
    /// 500 kill, TWO Next(3) draws per tick (always), the 0.008 gravity.
    #[test]
    fn w4_d10_s96_the_splash_ages_and_draws_two() {
        let mut p = proj(524);
        let mut rng = CensusRng::default();
        ai_ichor_splash(&mut p, &mut rng);
        assert!((p.ai[0] - 0.6).abs() < 1e-6);
        assert!((p.vel.y - 0.008).abs() < 1e-6);
        assert_eq!(rng.ranges, vec![(0, 3), (0, 3)], "the 2-iteration gate");
        // The 500 kill (the C# keeps running the dust + gravity after).
        p.ai[0] = 499.8;
        ai_ichor_splash(&mut p, &mut rng);
        assert!(!p.active);
        assert_eq!(rng.ranges.len(), 4, "the post-kill gates still draw");
    }

    /// The bounce (`:18807-18818`): a blocked move ages +100 and reverses
    /// both axes; open air integrates untouched.
    #[test]
    fn w4_d10_s96_the_splash_bounces_and_ages() {
        let mut w = World::new("t".into(), 300, 200);
        let mut p = proj(524);
        p.vel = Vec2::new(6.0, 2.0);
        let tx = ((p.pos.x + p.w + 6.0) / 16.0) as i32;
        let ty = ((p.pos.y + p.h * 0.5 + 2.0) / 16.0) as i32;
        let i = w.index(tx, ty);
        w.tiles[i] = tr_core::tile::Tile { kind: 1, ..Default::default() };
        w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
        ichor_splash_move(&mut p, &w);
        assert_eq!(p.ai[0], 100.0, "the +100 age");
        assert!(p.vel.x < 0.0, "the X reverse");
        // Open air: no age, no reverse.
        let w2 = World::new("t".into(), 300, 200);
        let mut q = proj(524);
        q.vel = Vec2::new(6.0, 2.0);
        let before = q.pos;
        ichor_splash_move(&mut q, &w2);
        assert_eq!(q.ai[0], 0.0);
        assert_eq!(q.pos, before + Vec2::new(6.0, 2.0));
    }

    /// The 525 row (`Projectile.cs:5953-5961`): 30×24, aiStyle 97,
    /// tileCollide false, timeLeft 10800, the stock penetrate 1; the
    /// 5-frame sheet (`Main.cs:8479`).
    #[test]
    fn w4_d10_s97_the_piggy_row_is_verbatim() {
        let d = proj_data(525).unwrap();
        assert_eq!((d.w, d.h), (30.0, 24.0));
        assert_eq!(d.ai_style, 97);
        assert!(!d.tile_collide);
        assert_eq!(d.time_left, 10800);
        assert_eq!(d.penetrate, 1);
        assert_eq!(crate::projectile::proj_frames(525), 5);
    }

    /// The frame ladder (`:33031-33069`): 4 ticks per frame, the
    /// 0-1-2-3-4-3-2-1 palindrome then the reset at 32.
    #[test]
    fn w4_d10_s97_the_piggy_flaps_in_a_palindrome() {
        let mut slots = vec![proj(525)];
        let mut frames = Vec::new();
        for _ in 0..32 {
            ai_flying_piggy(&mut slots, 0, Some((1600.0, 800.0)), 0);
            frames.push(slots[0].frame);
        }
        assert_eq!(&frames[0..4], &[0, 0, 0, 0]);
        assert_eq!(&frames[4..8], &[1, 1, 1, 1]);
        assert_eq!(&frames[16..20], &[4, 4, 4, 4]);
        assert_eq!(&frames[20..24], &[3, 3, 3, 3]);
        assert_eq!(&frames[28..32], &[1, 1, 1, 0], "the 32nd tick resets");
        assert_eq!(slots[0].frame_counter, 0);
    }

    /// The throw decay + rest latch (`:33088-33109`): the 0.94 decay with
    /// the velocity-sign facing; below 0.1 the velocity zeroes and the
    /// rest state arms (ai[0]=1, ai[1]=45). The bob (`:33119-33132`):
    /// ±0.005 with the 90 flip.
    #[test]
    fn w4_d10_s97_the_piggy_throws_then_hovers() {
        let mut slots = vec![proj(525)];
        slots[0].vel = Vec2::new(-4.0, 0.0);
        ai_flying_piggy(&mut slots, 0, Some((1600.0, 800.0)), 0);
        assert!((slots[0].vel.x - (-4.0 * 0.94)).abs() < 1e-5);
        assert_eq!(slots[0].direction, -1);
        assert_eq!(slots[0].sprite_direction, -1);
        // The rest latch.
        slots[0].vel = Vec2::new(0.05, 0.0);
        ai_flying_piggy(&mut slots, 0, Some((1600.0, 800.0)), 0);
        assert_eq!(slots[0].vel, Vec2::ZERO);
        assert_eq!(slots[0].ai[0], 1.0);
        assert_eq!(slots[0].ai[1], 45.0);
        // The bob: the owner left of the bank → facing -1; ai[1] 46 > 0
        // → the -0.005 rise.
        ai_flying_piggy(&mut slots, 0, Some((1000.0, 800.0)), 0);
        assert_eq!(slots[0].direction, -1);
        assert!((slots[0].vel.y - (-0.005)).abs() < 1e-6);
        assert_eq!(slots[0].ai[1], 46.0);
        // The 90 flip.
        slots[0].ai[1] = 89.0;
        ai_flying_piggy(&mut slots, 0, Some((1000.0, 800.0)), 0);
        assert_eq!(slots[0].ai[1], -90.0);
        // The negative counter drifts down.
        ai_flying_piggy(&mut slots, 0, Some((1000.0, 800.0)), 0);
        assert_eq!(slots[0].ai[1], -89.0);
        assert!((slots[0].vel.y - (-0.005 + 0.005)).abs() < 1e-6);
    }

    /// The dedup (`:33071-33087`): the higher-timeLeft twin kills the
    /// older; a tie keeps the scanning slot (the C# `>=`); the lower
    /// twin kills ITSELF. Foreign owners never dedup.
    #[test]
    fn w4_d10_s97_the_piggy_dedups_by_time_left() {
        let mut slots = vec![proj(525), proj(525)];
        slots[0].time_left = 100;
        slots[1].time_left = 50;
        ai_flying_piggy(&mut slots, 0, None, 0);
        assert!(slots[0].active);
        assert!(!slots[1].active, "the older twin dies");
        // The reverse: the scanning slot is older → it dies.
        let mut s2 = vec![proj(525), proj(525)];
        s2[0].time_left = 50;
        s2[1].time_left = 100;
        ai_flying_piggy(&mut s2, 0, None, 0);
        assert!(!s2[0].active);
        assert!(s2[1].active);
        // The tie: >= keeps the scanner.
        let mut s3 = vec![proj(525), proj(525)];
        s3[0].time_left = 100;
        s3[1].time_left = 100;
        ai_flying_piggy(&mut s3, 0, None, 0);
        assert!(s3[0].active);
        assert!(!s3[1].active);
        // A foreign owner's bank never dedups.
        let mut s4 = vec![proj(525), proj(525)];
        s4[1].owner = 1;
        ai_flying_piggy(&mut s4, 0, None, 0);
        assert!(s4[0].active && s4[1].active);
    }

    /// The 526 row (`Projectile.cs:5962-5970`): 8×8, aiStyle 98,
    /// tileCollide false, timeLeft 120, alpha 255, the stock penetrate 1.
    #[test]
    fn w4_d10_s98_the_particle_row_is_verbatim() {
        let d = proj_data(526).unwrap();
        assert_eq!((d.w, d.h), (8.0, 8.0));
        assert_eq!(d.ai_style, 98);
        assert!(!d.tile_collide);
        assert_eq!(d.time_left, 120);
        assert_eq!(d.alpha, 255);
        assert_eq!(d.penetrate, 1);
    }

    /// The seek + arrival (`:33136-33144`): the velocity lerps 10% toward
    /// the 15-speed target direction; the arrival (dist < speed) kills
    /// with NO dust draws; the flight draws TWO NextFloat per tick.
    #[test]
    fn w4_d10_s98_the_particle_seeks_and_arrives() {
        let mut p = proj(526);
        // The target 100 px right of the centre (1604, 804).
        p.ai[0] = 1704.0;
        p.ai[1] = 804.0;
        p.vel = Vec2::new(5.0, 0.0);
        let mut rng = CensusRng::default();
        ai_cultist_particle(&mut p, &mut rng);
        // lerp(5, 15, 0.1) = 6.
        assert!((p.vel.x - 6.0).abs() < 1e-5);
        assert_eq!(rng.floats, 2, "the 2-dust trail");
        // The arrival: the target 1 px away, speed 6 → the kill, no draws.
        let mut q = proj(526);
        q.ai[0] = 1605.0;
        q.ai[1] = 804.0;
        q.vel = Vec2::new(6.0, 0.0);
        let mut rng2 = CensusRng::default();
        ai_cultist_particle(&mut q, &mut rng2);
        assert!(!q.active);
        assert_eq!(rng2.floats, 0, "the arrival precedes the trail");
    }
}
