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
}
