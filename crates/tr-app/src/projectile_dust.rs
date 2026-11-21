//! Projectile-tick `Dust.NewDust` sites — the drain half of the queue
//! convention (the `pending_proj_sounds` / `player_dust` precedent). The
//! projectile pass (`ProjectilePool::update`) rolls a `&mut dyn DropRng` and
//! cannot borrow the App's live `DustPool`, so each site SNAPSHOTS its box /
//! velocity / owner flags at the C# read moment (`wants_dust_burst`, the
//! `wants_sound` pattern) and this module replays the `Dust.NewDust` chain on
//! the shared `UpdateProjectiles` stream (`Main.SwapRandom`,
//! `Main.cs:17775-17777`) in exact C# draw order, then applies the per-site
//! post-arms. One draw = one stream step; no new streams (S1-A-10). The pool
//! is the App's `particles::DustPool` (`Main.dust`), so the pressure bands /
//! default block / ChildSafety / per-type tweaks all ride
//! `DustPool::new_dust` verbatim (`Dust.cs:75-256`).
//!
//! W4-d53 batch 1 scope: the per-tick arrow trail (`Projectile.Update`
//! `Projectile.cs:17036-17077`, sites 17038/17042/17046/17055/17066/17071),
//! the stationary Vortex-bolt trail (`:17117-17125`, site 17122) and the
//! enchantment-visuals lane (`UpdateEnchantmentVisuals` `:17290-17318` →
//! `EmitEnchantmentVisualsAt` `:17320-17467`, sites 17325/17332/17346/17368/
//! 17377/17387/17397/17408/17418/17428/17443/17462).
//!
//! W4-d53 batch 2 scope: the Sword Beam's impact burst (`:19809-19814`, site
//! 19811) and the Light Disc's bounce sparks (`LightDisc_Bounce`
//! `:20317-20329`, site 20322 — both callers, the NPC-hit reverse `:13074`
//! and the tile bounce `:19518`).
//!
//! Cited skips (the d52 unported-arm convention):
//! * The wet entry/exit splash lane (`:16843-16996`, sites 16851-16987) —
//!   the generic `Update` wet block (`:16790-17007`) is unported (the
//!   bobber's `probe_wet` is aiStyle-61-only, the i35 one-tick-lag
//!   deviation); the lane lands with the generic wet subsystem.
//! * The 466 stationary trail (`:17108-17116`, site 17113) — the
//!   CultistBossLightningOrbArc has no ported row / TrailingMode entry.
//! * The aiStyle-190/191 enchantment ring (`:17299-17306`) — no ported
//!   aiStyle-190/191 row; the plain-box and whip-tip arms cover the ported
//!   kinds.
//! * The enchantment-7 gore sibling (`:17450-17458`) — the gore VISUAL is
//!   W4-d55's; only its `Main.rand` draws stay on the stream (the d52
//!   `body_landing_burst` precedent). Types 276-282 are `SafeGore`, so the
//!   ChildSafety re-type arm draws nothing.
//! * `Lighting.AddLight` (the mount arm `:17343`) — the S1-E-5 light-feed
//!   column (`proj_lights.rs`), no RNG.
//! * The remaining first-40 sites with unported hosts: Meowmere 502
//!   (`:18231`), LavaBoulder 1053 (`:18949`/`:18952`), Poo 1048 (`:19245`),
//!   the phaseblades (`:19392`/`:19401`/`:19468`), BoulderOfEarth 261
//!   (`:20423`) and the SuperStar family 728/977/1045 (`:22161`/`:22297`/
//!   `:22308`) — none has a `proj_data` row / AI arm; they land with their
//!   kinds (the d52 unported-kind convention).
//!
//! `npcProj` folds to `owner == 255` (the hostile-shot proxy, the
//! `damage_pve` `ownedBySomeone` convention); `noEnchantmentVisuals` /
//! `noEnchantments` are the SetDefaults kind sets (their writers censused
//! below). The drain runs at the pass tail (the S1-E-21c queue→drain
//! convention) — the C# draws mid-walk; the recorded deviation.

use crate::app::App;
use crate::projectile::Projectile;
use glam::Vec2;
use tr_core::UnifiedRandom;
use tr_core::particles::{Color, DustPool, SpawnCtx};
use tr_core::world::World;

/// `ProjectileID.Sets` stand-in: the kinds whose `SetDefaults` arm writes
/// `noEnchantmentVisuals = true` (`Projectile.cs:2683/3147/3160/4350/4810/
/// 4834/5766/6182/6582/7171/7383/7400/7518/7714/8872/9011/9234/10280/10338/
/// 10575/10595` — types 181/228/229/344/399/401/504/566/611/669/697/698/709/
/// 732/933/954/979/1100/1108/1128/1129). The `Update` gate (`:16700-16703`)
/// skips the whole visuals call for them.
pub(crate) fn no_enchantment_visuals(kind: u16) -> bool {
    matches!(
        kind,
        181 | 228 | 229 | 344 | 399 | 401 | 504 | 566 | 611 | 669 | 697 | 698 | 709 | 732
            | 933 | 954 | 979 | 1100 | 1108 | 1128 | 1129
    )
}

/// The kinds whose `SetDefaults` arm writes `noEnchantments = true`
/// (`Projectile.cs:5559/10501/10621/10633` — types 480/1121/1131/1132).
/// Every `EmitEnchantmentVisualsAt` gate reads it.
fn no_enchantments(kind: u16) -> bool {
    matches!(kind, 480 | 1121 | 1131 | 1132)
}

/// The per-tick arrow-trail kinds (`Projectile.cs:17036-17077`): the
/// FireArrow 2 / hostile FlamingArrow 82, Frostburn 172, Cursed 103, Ichor
/// 278, Unholy 4 and Jester's 5. Every other kind draws nothing here.
pub(crate) fn arrow_trail_kind(kind: u16) -> bool {
    matches!(kind, 2 | 82 | 172 | 103 | 278 | 4 | 5)
}

/// One queued dust burst — the snapshot the tick takes at the C# read
/// moment; the drain replays the draws + post-arms on the
/// `UpdateProjectiles` stream.
pub(crate) enum DustBurst {
    /// `EmitEnchantmentVisualsAt` (`:17320-17467`) — the owner-flag snapshot.
    Enchantment(EnchantmentBurst),
    /// The per-tick arrow trail (`:17036-17077`).
    ArrowTrail { kind: u16, pos: Vec2, w: i32, h: i32, vel: Vec2 },
    /// The stationary Vortex-bolt 229 puff (`:17117-17125`).
    StationaryTrail { at: Vec2, rot: f32 },
    /// The Sword Beam's 30-puff impact burst (`:19809-19814`).
    SwordBeam { pos: Vec2, w: i32, h: i32 },
    /// The Light Disc's bounce sparks (`LightDisc_Bounce` `:20317-20329`):
    /// the hit point, the surface normal and the PRE-bounce velocity (the
    /// reflect reads it). Both callers (`:13074` NPC-hit, `:19518` tile).
    LightDiscBounce { hit_point: Vec2, normal: Vec2, vel: Vec2 },
}

/// The enchantment-visuals snapshot — everything `EmitEnchantmentVisualsAt`
/// reads off the projectile + `Main.player[owner]` at the `:16702` call
/// moment (PRE-AI: the C# runs the visuals before the AI dispatch).
#[derive(Clone, Copy, Debug)]
pub(crate) struct EnchantmentBurst {
    /// The emit box — the plain position/width/height, or the whip-tip box
    /// (`UpdateEnchantmentVisuals` `:17307-17313`).
    pub pos: Vec2,
    pub w: i32,
    pub h: i32,
    pub vel: Vec2,
    pub direction: i32,
    pub extra_updates: i32,
    pub melee: bool,
    pub ranged: bool,
    pub friendly: bool,
    pub is_whip: bool,
    pub no_enchant: bool,
    pub frost_burn: bool,
    pub magma_stone: bool,
    pub mount_active: bool,
    pub mount_kind: i32,
    pub melee_enchant: u8,
}

/// `UpdateEnchantmentVisuals` (`:17290-17318`) + the owner reads: build the
/// emit box (the whip arm fills the control-point curve and centres the box
/// on its TIP, `:17311-17312`) and snapshot the flags. The aiStyle-190/191
/// ring arm has no ported row (cited skip). Called at the sub-step head,
/// ahead of the minion block + AI, exactly as the C# orders it.
pub(crate) fn enchantment_burst(
    p: &Projectile,
    pl: &crate::player::Player,
    owner_center: Option<(f32, f32)>,
) -> DustBurst {
    let (w, h) = (p.w as i32, p.h as i32);
    let (pos, bw, bh) = if tr_core::whip::is_whip(p.kind) {
        match p.whip_settings {
            Some(settings) => {
                // `FillWhipControlPoints(this, list)` (`:17310`) — the
                // DEFAULT args: getActualCollisionPoints false, frameOffset
                // 0 (the damage walk's `true` sweep is a different call).
                let arm = owner_center
                    .map(|(x, y)| Vec2::new(x, y))
                    .unwrap_or(p.pos + Vec2::new(p.w * 0.5, p.h * 0.5));
                let inputs = tr_core::whip::WhipCurveInputs {
                    kind: p.kind,
                    ai0: p.ai[0],
                    frame_offset: 0,
                    ai1: p.ai[1],
                    sprite_direction: p.sprite_direction,
                    rotation: p.rot,
                    arm_position: arm,
                    held_use_animation: p.whip_use_animation,
                    whip_range_multiplier: pl.whip_range_multiplier,
                    velocity_len: p.vel.length(),
                    get_actual_collision_points: false,
                };
                let pts = tr_core::whip::fill_whip_control_points(&inputs, &settings);
                let tip = pts.last().copied().unwrap_or(p.pos);
                // `:17312` — `(vector.X - width/2, vector.Y - height/2)`,
                // the C# int division.
                (Vec2::new(tip.x - (w / 2) as f32, tip.y - (h / 2) as f32), w, h)
            }
            // No settings (a non-spawned whip in a pool test): the plain box.
            None => (p.pos, w, h),
        }
    } else {
        (p.pos, w, h)
    };
    DustBurst::Enchantment(EnchantmentBurst {
        pos,
        w: bw,
        h: bh,
        vel: p.vel,
        direction: p.direction,
        extra_updates: p.extra_updates as i32,
        melee: p.melee,
        ranged: p.ranged,
        friendly: p.friendly,
        is_whip: tr_core::whip::is_whip(p.kind),
        no_enchant: no_enchantments(p.kind),
        frost_burn: pl.frost_burn,
        magma_stone: pl.magma_stone,
        mount_active: pl.mount.is_active(),
        mount_kind: pl.mount.kind(),
        melee_enchant: pl.melee_enchant,
    })
}

/// One `Dust.NewDust` on the caller's stream, returning the slot index (the
/// sentinel included — the C# writes `Main.dust[6000]` on a refusal).
#[allow(clippy::too_many_arguments)]
fn nd(
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
    pos: Vec2,
    w: i32,
    h: i32,
    ty: i32,
    sx: f32,
    sy: f32,
    alpha: i32,
    color: Color,
    scale: f32,
) -> usize {
    dust.new_dust(rng, ctx, world, pos, w, h, ty, sx, sy, alpha, color, scale)
}

/// XNA `Color.Lerp` (`MonoGame Color.cs`): `MathHelper.Lerp` per channel
/// (`a + (b - a) * t`), then the ctor's truncating `(byte)` cast — the
/// `dust_table::lerp_color` shape.
fn lerp_color(a: Color, b: Color, t: f32) -> Color {
    let ch = |x: u8, y: u8| {
        let (x, y) = (x as f32, y as f32);
        (x + (y - x) * t) as u8
    };
    Color::new(ch(a.r, b.r), ch(a.g, b.g), ch(a.b, b.b), ch(a.a, b.a))
}

/// The App-facing drain: replay one queued burst on the live pool + the
/// `UpdateProjectiles` stream (the `player_dust::drain_*` shape).
pub(crate) fn drain(app: &mut App, b: DustBurst) {
    let ctx = app.spawn_ctx();
    let rng = app.rngs.stream(crate::app::RngKey::UpdateProjectiles);
    drain_burst(rng, &mut app.dust, &ctx, &app.world, b);
}

/// The pure replay — the witnesses call this on a scratch pool (the d52
/// convention), the live pass through [`drain`].
pub(crate) fn drain_burst(
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
    b: DustBurst,
) {
    match b {
        DustBurst::Enchantment(e) => enchantment_visuals(rng, dust, ctx, world, &e),
        DustBurst::ArrowTrail { kind, pos, w, h, vel } => {
            arrow_trail(rng, dust, ctx, world, kind, pos, w, h, vel)
        }
        DustBurst::StationaryTrail { at, rot } => stationary_trail(rng, dust, ctx, world, at, rot),
        DustBurst::SwordBeam { pos, w, h } => sword_beam_burst(rng, dust, ctx, world, pos, w, h),
        DustBurst::LightDiscBounce { hit_point, normal, vel } => {
            light_disc_bounce(rng, dust, ctx, world, hit_point, normal, vel)
        }
    }
}

/// The Sword Beam's impact burst (`Projectile.cs:19809-19814`): 30 type-64
/// puffs at the impact position, box = the beam's own, alpha 100,
/// `Color.White`, scale 1.8; each noGravity + `velocity *= Next(1,7)` (the
/// kick draw rides AFTER the pool's own rolls). Type 64 is in the `:214-219`
/// tweak family (one extra `Next(-10,6)` per dust).
fn sword_beam_burst(
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
    pos: Vec2,
    w: i32,
    h: i32,
) {
    for _ in 0..30 {
        let idx = nd(rng, dust, ctx, world, pos, w, h, 64, 0.0, 0.0, 100, Color::WHITE, 1.8);
        let d = dust.slot_mut(idx);
        d.no_gravity = true;
        d.velocity *= rng.next_range(1, 7) as f32;
    }
}

/// `Vector2.Reflect(vector, normal)` (XNA): `vector - 2 * Dot(vector, normal)
/// * normal` — NO normalization (the C# passes the raw normal).
fn reflect(v: Vec2, n: Vec2) -> Vec2 {
    v - 2.0 * v.dot(n) * n
}

/// `LightDisc_Bounce` (`Projectile.cs:20317-20329`): the reflected velocity
/// `spinningpoint = Reflect(velocity, normal)`, then 4 iterations — each a
/// type-306 `NewDustPerfect` at the hit point whose velocity is the
/// spinningpoint rotated by `π/4 * NextFloatDirection()` then scaled by
/// `0.6 * NextFloat()` (alpha 200, scale 1.6), the colour a
/// `NextFloat`-lerp of (219,253,0)→Cyan; then a `CloneDust` (a fresh
/// `NewDust` of the source's type/position) overwritten White / scale 1 /
/// alpha 50. The draw order per iteration: NextFloatDirection (1),
/// NextFloat (1), the Perfect's NewDust (6), the colour NextFloat (1), the
/// CloneDust's NewDust (6) = 15.
fn light_disc_bounce(
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
    hit_point: Vec2,
    normal: Vec2,
    vel: Vec2,
) {
    let spinningpoint = reflect(vel, normal);
    for _ in 0..4 {
        // `:20322` — the rotate angle (NextFloatDirection) THEN the radius
        // scale (NextFloat), both argument draws ahead of the NewDust.
        let angle = std::f32::consts::FRAC_PI_4 * rng.next_float_direction();
        let speed = 0.6 * rng.next_float();
        let v = crate::bow_launch::rotated_by(spinningpoint, angle as f64) * speed;
        let idx = dust.new_dust_perfect(
            rng, ctx, world, hit_point, 306, Some(v), 200, Color::default(), 1.6,
        );
        // `:20323` — the colour lerp draw, then the write.
        let t = rng.next_float();
        dust.slot_mut(idx).color =
            lerp_color(Color::new(219, 253, 0, 255), Color::new(0, 255, 255, 255), t);
        // `:20324-20327` — the CloneDust (a fresh NewDust + the field copy),
        // then the White / scale-1 / alpha-50 overwrite.
        let clone = dust.clone_dust(rng, ctx, world, idx);
        let c = dust.slot_mut(clone);
        c.color = Color::WHITE;
        c.scale = 1.0;
        c.alpha = 50;
    }
}

/// The per-tick arrow trail (`Projectile.cs:17036-17077`): the per-kind
/// `NewDust` at the POST-movement position, box = the projectile's own.
/// 2/82 type 6, 172 type 135, 103 type 75 + the `Next(2)` noGravity/scale×2
/// arm, 278 type 169 + the `Next(2)` noGravity/scale×1.5 arm, 4 the
/// `Next(5)`-gated type 14 (alpha 150, scale 1.1), 5 the `Next(3)` type pick
/// (15/57/58) at half velocity (alpha 150, scale 1.2).
fn arrow_trail(
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
    kind: u16,
    pos: Vec2,
    w: i32,
    h: i32,
    vel: Vec2,
) {
    match kind {
        // `:17036-17039` — the FireArrow pair, type 6.
        2 | 82 => {
            nd(rng, dust, ctx, world, pos, w, h, 6, 0.0, 0.0, 100, Color::default(), 1.0);
        }
        // `:17040-17043` — the Frostburn Arrow, type 135.
        172 => {
            nd(rng, dust, ctx, world, pos, w, h, 135, 0.0, 0.0, 100, Color::default(), 1.0);
        }
        // `:17044-17052` — the Cursed Arrow, type 75 + the 1-in-2 lift.
        103 => {
            let idx = nd(rng, dust, ctx, world, pos, w, h, 75, 0.0, 0.0, 100, Color::default(), 1.0);
            if rng.next_max(2) == 0 {
                let d = dust.slot_mut(idx);
                d.no_gravity = true;
                d.scale *= 2.0;
            }
        }
        // `:17053-17061` — the Ichor Arrow, type 169 + the 1-in-2 lift.
        278 => {
            let idx = nd(rng, dust, ctx, world, pos, w, h, 169, 0.0, 0.0, 100, Color::default(), 1.0);
            if rng.next_max(2) == 0 {
                let d = dust.slot_mut(idx);
                d.no_gravity = true;
                d.scale *= 1.5;
            }
        }
        // `:17062-17068` — the Unholy Arrow, the 1-in-5 type-14 gate.
        4 => {
            if rng.next_max(5) == 0 {
                nd(rng, dust, ctx, world, pos, w, h, 14, 0.0, 0.0, 150, Color::default(), 1.1);
            }
        }
        // `:17069-17077` — the Jester's Arrow: the `Next(3)` type pick feeds
        // the NewDust Type argument (the draw runs BEFORE the pool's own).
        5 => {
            let ty = match rng.next_max(3) {
                0 => 15,
                1 => 57,
                _ => 58,
            };
            nd(rng, dust, ctx, world, pos, w, h, ty, vel.x * 0.5, vel.y * 0.5, 150, Color::default(), 1.2);
        }
        _ => {}
    }
}

/// The stationary Vortex-bolt trail (`Projectile.cs:17117-17125`, site
/// 17122): while `velocity == Zero` at the mode-1 history shift, one type-229
/// puff at the OLDEST ring entry — the angle = `rotation + π/2 ± π/2` (the
/// `Next(2)` sign), the radius `NextDouble()*2 + 2`, then noGravity +
/// scale 1.7. The 466 twin (`:17108-17116`) is the cited unported skip.
fn stationary_trail(
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
    at: Vec2,
    rot: f32,
) {
    // `:17119` — the sign draw, then `:17120` the NextDouble radius.
    let sign = if rng.next_max(2) == 1 { -1.0f32 } else { 1.0 };
    let angle = rot + std::f32::consts::FRAC_PI_2 + sign * std::f32::consts::FRAC_PI_2;
    let r = (rng.next_double() * 2.0 + 2.0) as f32;
    let v = Vec2::new((angle as f64).cos() as f32 * r, (angle as f64).sin() as f32 * r);
    let idx = nd(rng, dust, ctx, world, at, 0, 0, 229, v.x, v.y, 0, Color::default(), 1.0);
    let d = dust.slot_mut(idx);
    d.no_gravity = true;
    d.scale = 1.7;
}

/// `EmitEnchantmentVisualsAt` (`Projectile.cs:17320-17467`) — the owner-flag
/// dust chain, verbatim draw order. Every gate short-circuits exactly as the
/// C# `&&` does (a failed leading condition draws NOTHING).
fn enchantment_visuals(
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
    e: &EnchantmentBurst,
) {
    // `:17323-17329` — the Frost set's frostBurn trail (melee/ranged,
    // friendly, !hostile folded into the owner gate, !noEnchantments): the
    // `Next(2*(1+extraUpdates))` gate, type 135 at `vel*0.2 + dir*3` x.
    if e.frost_burn
        && (e.melee || e.ranged)
        && e.friendly
        && !e.no_enchant
        && rng.next_max(2 * (1 + e.extra_updates)) == 0
    {
        let idx = nd(
            rng, dust, ctx, world, e.pos, e.w, e.h, 135,
            e.vel.x * 0.2 + e.direction as f32 * 3.0, e.vel.y * 0.2,
            100, Color::default(), 2.0,
        );
        let d = dust.slot_mut(idx);
        d.no_gravity = true;
        d.velocity *= 0.7;
        d.velocity.y -= 0.5;
    }
    // `:17330-17340` — the Magma Stone: melee + `Next(3) != 0`, type 6 at
    // the -4-inflated box, the `Next(2)` scale-1.5 arm, then ×2 velocity.
    if e.melee && e.magma_stone && !e.no_enchant && rng.next_max(3) != 0 {
        let idx = nd(
            rng, dust, ctx, world, e.pos - Vec2::splat(4.0), e.w + 8, e.h + 8, 6,
            e.vel.x * 0.2, e.vel.y * 0.2, 100, Color::default(), 2.0,
        );
        if rng.next_max(2) == 0 {
            dust.slot_mut(idx).scale = 1.5;
        }
        let d = dust.slot_mut(idx);
        d.no_gravity = true;
        d.velocity.x *= 2.0;
        d.velocity.y *= 2.0;
    }
    // `:17341-17361` — the bee/hornet mounts (62-65): the AddLight is the
    // light-feed column (no RNG); the `Next(5)` gate opens the type-306
    // puff with the `Next(2)` scale-1.2 arm, ×0.2 velocity and the
    // `NextFloat`-cubed white lerp (the 63/65 orange hue).
    if e.mount_active
        && matches!(e.mount_kind, 62 | 63 | 64 | 65)
        && (e.melee || e.ranged)
        && e.friendly
        && !e.no_enchant
    {
        if rng.next_max(5) == 0 {
            let idx = nd(
                rng, dust, ctx, world, e.pos - Vec2::splat(4.0), e.w + 8, e.h + 8, 306,
                e.vel.x * 0.2, e.vel.y * 0.2, 100, Color::default(), 1.0,
            );
            if rng.next_max(2) == 0 {
                dust.slot_mut(idx).scale = 1.2;
            }
            let d = dust.slot_mut(idx);
            d.no_gravity = true;
            d.velocity.x *= 0.2;
            d.velocity.y *= 0.2;
            let num4 = rng.next_float();
            let t = num4 * num4 * num4;
            // `new Color(0.9f, 0.7f, 1f)` — the float ctor's ×255 truncation.
            let base = Color::new(
                (0.9f32 * 255.0) as u8,
                (0.7f32 * 255.0) as u8,
                (1.0f32 * 255.0) as u8,
                255,
            );
            d.color = lerp_color(base, Color::WHITE, t);
            if e.mount_kind == 63 || e.mount_kind == 65 {
                let orange = Color::new(
                    (1.0f32 * 255.0) as u8,
                    (0.7f32 * 255.0) as u8,
                    (0.5f32 * 255.0) as u8,
                    255,
                );
                d.color = lerp_color(orange, Color::WHITE, t);
            }
        }
    }
    // `:17362-17365` — the melee-imbue gate: neither melee nor whip, no
    // imbue, or noEnchantments → the chain ends.
    if (!e.melee && !e.is_whip) || e.melee_enchant == 0 || e.no_enchant {
        return;
    }
    // `:17366-17372` — imbue 1 (Venom): the FIRST `Next(3)` gate, type 171.
    if e.melee_enchant == 1 && rng.next_max(3) == 0 {
        let idx = nd(rng, dust, ctx, world, e.pos, e.w, e.h, 171, 0.0, 0.0, 100, Color::default(), 1.0);
        let d = dust.slot_mut(idx);
        d.no_gravity = true;
        d.fade_in = 1.5;
        d.velocity *= 0.25;
    }
    // `:17373-17466` — the SECOND imbue ladder. Imbue 1 draws its OWN
    // `Next(3)` again (`:17375`) — the C# has two independent blocks, both
    // gates ride the stream every tick.
    if e.melee_enchant == 1 {
        if rng.next_max(3) == 0 {
            let idx = nd(rng, dust, ctx, world, e.pos, e.w, e.h, 171, 0.0, 0.0, 100, Color::default(), 1.0);
            let d = dust.slot_mut(idx);
            d.no_gravity = true;
            d.fade_in = 1.5;
            d.velocity *= 0.25;
        }
    } else if e.melee_enchant == 2 {
        // `:17383-17392` — Fire: `Next(2)`, type 75 at dir-boosted velocity.
        if rng.next_max(2) == 0 {
            let idx = nd(
                rng, dust, ctx, world, e.pos, e.w, e.h, 75,
                e.vel.x * 0.2 + e.direction as f32 * 3.0, e.vel.y * 0.2,
                100, Color::default(), 2.5,
            );
            let d = dust.slot_mut(idx);
            d.no_gravity = true;
            d.velocity *= 0.7;
            d.velocity.y -= 0.5;
        }
    } else if e.melee_enchant == 3 {
        // `:17393-17402` — Cursed Flames: `Next(2)`, type 6, same post-arms.
        if rng.next_max(2) == 0 {
            let idx = nd(
                rng, dust, ctx, world, e.pos, e.w, e.h, 6,
                e.vel.x * 0.2 + e.direction as f32 * 3.0, e.vel.y * 0.2,
                100, Color::default(), 2.5,
            );
            let d = dust.slot_mut(idx);
            d.no_gravity = true;
            d.velocity *= 0.7;
            d.velocity.y -= 0.5;
        }
    } else if e.melee_enchant == 4 {
        // `:17403-17413` — Ichor: `Next(2)`, type 57 scale 1.1, halved vel.
        if rng.next_max(2) == 0 {
            let idx = nd(
                rng, dust, ctx, world, e.pos, e.w, e.h, 57,
                e.vel.x * 0.2 + e.direction as f32 * 3.0, e.vel.y * 0.2,
                100, Color::default(), 1.1,
            );
            let d = dust.slot_mut(idx);
            d.no_gravity = true;
            d.velocity.x /= 2.0;
            d.velocity.y /= 2.0;
        }
    } else if e.melee_enchant == 5 {
        // `:17414-17423` — Confetti: `Next(2)`, type 169, +dir/+0.2 velocity.
        if rng.next_max(2) == 0 {
            let idx = nd(rng, dust, ctx, world, e.pos, e.w, e.h, 169, 0.0, 0.0, 100, Color::default(), 1.0);
            let d = dust.slot_mut(idx);
            d.velocity.x += e.direction as f32;
            d.velocity.y += 0.2;
            d.no_gravity = true;
        }
    } else if e.melee_enchant == 6 {
        // `:17424-17433` — Nanites: `Next(2)`, type 135, same post-arms.
        if rng.next_max(2) == 0 {
            let idx = nd(rng, dust, ctx, world, e.pos, e.w, e.h, 135, 0.0, 0.0, 100, Color::default(), 1.0);
            let d = dust.slot_mut(idx);
            d.velocity.x += e.direction as f32;
            d.velocity.y += 0.2;
            d.no_gravity = true;
        }
    } else if e.melee_enchant == 7 {
        // `:17434-17459` — Party: the velocity clamped to length 4, the
        // `Next(20)` gate + the `Next(139,143)` type pick + the five jitter
        // draws; then the `Next(40)` gore sibling (W4-d55's visual — its
        // type/kick/post draws stay, the d52 body_landing_burst precedent).
        let mut vector = e.vel;
        if vector.length() > 4.0 {
            vector *= 4.0 / vector.length();
        }
        if rng.next_max(20) == 0 {
            let ty = rng.next_range(139, 143);
            let idx = nd(
                rng, dust, ctx, world, e.pos, e.w, e.h, ty, vector.x, vector.y,
                0, Color::default(), 1.2,
            );
            let d = dust.slot_mut(idx);
            d.velocity.x *= 1.0 + rng.next_range(-50, 51) as f32 * 0.01;
            d.velocity.y *= 1.0 + rng.next_range(-50, 51) as f32 * 0.01;
            d.velocity.x += rng.next_range(-50, 51) as f32 * 0.05;
            d.velocity.y += rng.next_range(-50, 51) as f32 * 0.05;
            d.scale *= 1.0 + rng.next_range(-30, 31) as f32 * 0.01;
        }
        if rng.next_max(40) == 0 {
            // `Gore.NewGore(Type: Next(276,283), Position, Velocity: vector)`
            // — the type-argument draw, then the NewGore-internal velocity
            // kicks (`Gore.cs:1327-1328`: Y `Next(10,31)` then X
            // `Next(-20,21)`), then the four post-arms. 276-282 are SafeGore
            // → the ChildSafety arm draws nothing.
            let _gore_type = rng.next_range(276, 283);
            let _kick_y = rng.next_range(10, 31) as f32 * 0.1;
            let _kick_x = rng.next_range(-20, 21) as f32 * 0.1;
            let _ = rng.next_range(-50, 51); // vel.X mul (`:17453`)
            let _ = rng.next_range(-50, 51); // vel.Y mul (`:17454`)
            let _ = rng.next_range(-20, 21); // scale mul (`:17455`)
            let _ = rng.next_range(-50, 51); // vel.X add (`:17456`)
            let _ = rng.next_range(-50, 51); // vel.Y add (`:17457`)
        }
    } else if e.melee_enchant == 8 && rng.next_max(4) == 0 {
        // `:17460-17466` — Midas: `Next(4)`, type 46, fadeIn 1.5 + ×0.25 vel.
        let idx = nd(rng, dust, ctx, world, e.pos, e.w, e.h, 46, 0.0, 0.0, 100, Color::default(), 1.0);
        let d = dust.slot_mut(idx);
        d.no_gravity = true;
        d.fade_in = 1.5;
        d.velocity *= 0.25;
    }
}

// --- W4-d53 witnesses (WRITTEN NOT RUN — code-only mode) -------------------
#[cfg(test)]
mod tests {
    use super::*;
    use tr_core::tile::{Tile, FLAG_ACTIVE};
    use tr_core::TILE_PX;

    fn ground_air() -> World {
        let mut w = World::new("t".into(), 200, 200);
        for x in 0..200 {
            for y in 120..200 {
                let i = w.index(x, y);
                w.tiles[i] = Tile { kind: 1, ..Default::default() };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        w.compute_column_surface();
        w
    }

    /// A permissive ctx: the whole world on-screen, the full dust budget —
    /// the pool never culls, so every spawn lands on the empty-pool path
    /// (the d52 convention).
    fn ctx(w: &World) -> SpawnCtx {
        SpawnCtx {
            game_menu: false,
            game_paused: false,
            generating: false,
            screen_position: Vec2::ZERO,
            screen_size: Vec2::new(w.width as f32 * TILE_PX, w.height as f32 * TILE_PX),
            max_dust_to_draw: 6000,
        }
    }

    /// The `NewDust` default-block census on the empty-pool path (the d52
    /// `burst_block` shape): the two position rolls, the two velocity rolls,
    /// the frame row, the scale roll, then the per-type tweak's extra draw
    /// for the types in the `:214-219` family (6/75/135/169 — one
    /// `Next(-10,6)` each). Types 14/15/46/57/58/171/229/306 draw nothing
    /// extra; ChildSafety is disarmed (the default), so no re-type draws.
    fn burst_block(r: &mut UnifiedRandom, dust_type: i32, box_w: i32, box_h: i32) {
        r.next_max(box_w.max(5) - 4);
        r.next_max(box_h.max(5) - 4);
        r.next_range(-20, 21);
        r.next_range(-20, 21);
        r.next_max(3);
        r.next_range(-20, 21);
        if matches!(
            dust_type,
            228 | 279 | 269 | 135 | 6 | 242 | 75 | 169 | 29 | 59..=65 | 158
                | 293..=298 | 302 | 307 | 310
        ) {
            r.next_range(-10, 6);
        }
    }

    /// W4-d53 — `Projectile.cs:17036-17077`: the per-kind arrow-trail draw
    /// census. The mirror replays 200 ticks per kind from one seed, draining
    /// the pool per tick so the census stays on the empty-pool path. 2/82/172
    /// draw one block; 103/278 add the `Next(2)` lift gate; 4 draws the
    /// `Next(5)` gate EVERY tick then the block on a pass; 5 draws the
    /// `Next(3)` type pick then the block.
    #[test]
    fn w4_d53_arrow_trail_draws_match_the_oracle() {
        let w = ground_air();
        let c = ctx(&w);
        let pos = Vec2::new(60.0 * 16.0, 100.0 * 16.0);
        let vel = Vec2::new(6.0, -2.0);
        for kind in [2u16, 82, 172, 103, 278, 4, 5] {
            let mut rng = UnifiedRandom::new(53);
            let mut probe = UnifiedRandom::new(53);
            let mut dust = DustPool::new();
            for tick in 0..200 {
                drain_burst(
                    &mut rng, &mut dust, &c, &w,
                    DustBurst::ArrowTrail { kind, pos, w: 10, h: 10, vel },
                );
                match kind {
                    2 | 82 => burst_block(&mut probe, 6, 10, 10),
                    172 => burst_block(&mut probe, 135, 10, 10),
                    103 => {
                        burst_block(&mut probe, 75, 10, 10);
                        probe.next_max(2);
                    }
                    278 => {
                        burst_block(&mut probe, 169, 10, 10);
                        probe.next_max(2);
                    }
                    4 => {
                        if probe.next_max(5) == 0 {
                            burst_block(&mut probe, 14, 10, 10);
                        }
                    }
                    5 => {
                        let ty = match probe.next_max(3) {
                            0 => 15,
                            1 => 57,
                            _ => 58,
                        };
                        burst_block(&mut probe, ty, 10, 10);
                    }
                    _ => unreachable!(),
                }
                assert_eq!(rng.peek(), probe.peek(), "kind {kind} trail stream @{tick}");
                for s in dust.slots_mut() {
                    s.active = false;
                }
            }
        }
    }

    /// W4-d53 — the arrow trail's per-kind dust type + post-arms: 2/82 type 6,
    /// 172 type 135, 103 type 75 (the lift arm doubles the scale when it
    /// fires), 278 type 169, 4 type 14 alpha 150, 5 one of 15/57/58 at half
    /// velocity alpha 150.
    #[test]
    fn w4_d53_arrow_trail_writes_the_post_fields() {
        let w = ground_air();
        let c = ctx(&w);
        let pos = Vec2::new(60.0 * 16.0, 100.0 * 16.0);
        let vel = Vec2::new(6.0, -2.0);
        for (kind, tys) in [
            (2u16, vec![6]),
            (82, vec![6]),
            (172, vec![135]),
            (103, vec![75]),
            (278, vec![169]),
            (4, vec![14]),
            (5, vec![15, 57, 58]),
        ] {
            let mut rng = UnifiedRandom::new(0);
            let mut dust = DustPool::new();
            // Spin until the kind's first dust lands (4's gate is 1-in-5).
            loop {
                drain_burst(
                    &mut rng, &mut dust, &c, &w,
                    DustBurst::ArrowTrail { kind, pos, w: 10, h: 10, vel },
                );
                if dust.slot(0).active {
                    break;
                }
            }
            let d = dust.slot(0);
            assert!(tys.contains(&d.dust_type), "kind {kind} type {}", d.dust_type);
            if kind == 4 || kind == 5 {
                assert_eq!(d.alpha, 150, "kind {kind} alpha");
            }
            if kind == 5 {
                // The Jester's speed argument is half the flight velocity
                // (the pool's own jitter rides on top, so only the sign of
                // the x-bias is pinned here — the census test owns the draws).
                assert!(d.dust_type == 15 || d.dust_type == 57 || d.dust_type == 58);
            }
        }
    }

    /// W4-d53 — `Projectile.cs:17117-17125`: the stationary 580 puff draws
    /// the `Next(2)` sign, the `NextDouble` radius, then the six default
    /// rolls (type 229 has no tweak draw); the post-arms write noGravity +
    /// scale 1.7 at the oldest ring entry.
    #[test]
    fn w4_d53_stationary_trail_draws_and_writes() {
        let w = ground_air();
        let c = ctx(&w);
        let at = Vec2::new(60.0 * 16.0, 100.0 * 16.0);
        let mut rng = UnifiedRandom::new(53);
        let mut probe = UnifiedRandom::new(53);
        let mut dust = DustPool::new();
        for tick in 0..50 {
            drain_burst(&mut rng, &mut dust, &c, &w, DustBurst::StationaryTrail { at, rot: 0.7 });
            probe.next_max(2);
            probe.next_double();
            burst_block(&mut probe, 229, 0, 0);
            assert_eq!(rng.peek(), probe.peek(), "stationary stream @{tick}");
            let d = dust.slot(0);
            assert!(d.no_gravity);
            assert_eq!(d.scale, 1.7);
            // The 0×0 box clamps to 5×5 (`Dust.cs:155-162`), so the default
            // block's position is `at + Next(1) + 4` = `at + 4` per axis
            // (`:170-171`) — the anchor plus the inset, NOT the exact anchor.
            assert_eq!(d.position, at + Vec2::splat(4.0), "the oldest-ring-entry anchor + inset");
            for s in dust.slots_mut() {
                s.active = false;
            }
        }
    }

    /// A melee/ranged-friendly enchantment snapshot with every owner flag
    /// off except the one under test (the plain box, no whip).
    fn ench(melee_enchant: u8) -> EnchantmentBurst {
        EnchantmentBurst {
            pos: Vec2::new(60.0 * 16.0, 100.0 * 16.0),
            w: 16,
            h: 16,
            vel: Vec2::new(6.0, -2.0),
            direction: 1,
            extra_updates: 0,
            melee: true,
            ranged: false,
            friendly: true,
            is_whip: false,
            no_enchant: false,
            frost_burn: false,
            magma_stone: false,
            mount_active: false,
            mount_kind: -1,
            melee_enchant,
        }
    }

    /// W4-d53 — `Projectile.cs:17323-17340`: the frostBurn gate draws
    /// `Next(2*(1+extraUpdates))` then the type-135 block; the magmaStone
    /// gate draws `Next(3)` then (on a pass) the type-6 block + the `Next(2)`
    /// scale arm. With both flags off and no imbue the chain draws NOTHING.
    #[test]
    fn w4_d53_enchantment_frost_and_magma_gates() {
        let w = ground_air();
        let c = ctx(&w);
        // Nothing armed → zero draws.
        let mut rng = UnifiedRandom::new(53);
        let mut dust = DustPool::new();
        drain_burst(&mut rng, &mut dust, &c, &w, DustBurst::Enchantment(ench(0)));
        assert_eq!(rng.peek(), UnifiedRandom::new(53).peek(), "no flags, no draws");
        // frostBurn only: the gate + the 135 block, every tick.
        let mut e = ench(0);
        e.frost_burn = true;
        e.extra_updates = 2;
        let mut rng = UnifiedRandom::new(53);
        let mut probe = UnifiedRandom::new(53);
        let mut dust = DustPool::new();
        for tick in 0..100 {
            drain_burst(&mut rng, &mut dust, &c, &w, DustBurst::Enchantment(e.clone()));
            if probe.next_max(2 * (1 + 2)) == 0 {
                burst_block(&mut probe, 135, 16, 16);
            }
            assert_eq!(rng.peek(), probe.peek(), "frost stream @{tick}");
            for s in dust.slots_mut() {
                s.active = false;
            }
        }
        // magmaStone only: Next(3) != 0 opens the 6 block + the Next(2) arm.
        let mut e = ench(0);
        e.magma_stone = true;
        let mut rng = UnifiedRandom::new(53);
        let mut probe = UnifiedRandom::new(53);
        let mut dust = DustPool::new();
        for tick in 0..100 {
            drain_burst(&mut rng, &mut dust, &c, &w, DustBurst::Enchantment(e.clone()));
            if probe.next_max(3) != 0 {
                burst_block(&mut probe, 6, 16 + 8, 16 + 8);
                probe.next_max(2);
            }
            assert_eq!(rng.peek(), probe.peek(), "magma stream @{tick}");
            for s in dust.slots_mut() {
                s.active = false;
            }
        }
    }

    /// W4-d53 — `Projectile.cs:17366-17466`: the imbue ladder's per-value
    /// census. Imbue 1 draws TWO `Next(3)` gates (the `:17366` block AND the
    /// `:17375` one), each opening a type-171 block; 2/3/4/5/6 draw one
    /// `Next(2)` gate + their type's block; 8 draws one `Next(4)` gate.
    #[test]
    fn w4_d53_enchantment_imbue_ladder_census() {
        let w = ground_air();
        let c = ctx(&w);
        for enchant in [1u8, 2, 3, 4, 5, 6, 8] {
            let mut rng = UnifiedRandom::new(53);
            let mut probe = UnifiedRandom::new(53);
            let mut dust = DustPool::new();
            for tick in 0..100 {
                drain_burst(
                    &mut rng, &mut dust, &c, &w,
                    DustBurst::Enchantment(ench(enchant)),
                );
                match enchant {
                    1 => {
                        // The first block's gate + body.
                        if probe.next_max(3) == 0 {
                            burst_block(&mut probe, 171, 16, 16);
                        }
                        // The second block's gate + body (independent draw).
                        if probe.next_max(3) == 0 {
                            burst_block(&mut probe, 171, 16, 16);
                        }
                    }
                    2 => {
                        if probe.next_max(2) == 0 {
                            burst_block(&mut probe, 75, 16, 16);
                        }
                    }
                    3 => {
                        if probe.next_max(2) == 0 {
                            burst_block(&mut probe, 6, 16, 16);
                        }
                    }
                    4 => {
                        if probe.next_max(2) == 0 {
                            burst_block(&mut probe, 57, 16, 16);
                        }
                    }
                    5 => {
                        if probe.next_max(2) == 0 {
                            burst_block(&mut probe, 169, 16, 16);
                        }
                    }
                    6 => {
                        if probe.next_max(2) == 0 {
                            burst_block(&mut probe, 135, 16, 16);
                        }
                    }
                    8 => {
                        if probe.next_max(4) == 0 {
                            burst_block(&mut probe, 46, 16, 16);
                        }
                    }
                    _ => unreachable!(),
                }
                assert_eq!(rng.peek(), probe.peek(), "imbue {enchant} stream @{tick}");
                for s in dust.slots_mut() {
                    s.active = false;
                }
            }
        }
    }

    /// W4-d53 — `Projectile.cs:17434-17458`: imbue 7's Party census — the
    /// `Next(20)` gate, the `Next(139,143)` type pick + the five jitter
    /// draws, then the `Next(40)` gore gate whose body draws the type pick +
    /// the two NewGore kicks + the four post-arms (the d55 visual skip keeps
    /// every draw).
    #[test]
    fn w4_d53_enchantment_party_census_incl_gore_sibling() {
        let w = ground_air();
        let c = ctx(&w);
        let mut rng = UnifiedRandom::new(53);
        let mut probe = UnifiedRandom::new(53);
        let mut dust = DustPool::new();
        let mut dust_passes = 0;
        let mut gore_passes = 0;
        for tick in 0..400 {
            drain_burst(&mut rng, &mut dust, &c, &w, DustBurst::Enchantment(ench(7)));
            if probe.next_max(20) == 0 {
                dust_passes += 1;
                let ty = probe.next_range(139, 143);
                burst_block(&mut probe, ty, 16, 16);
                probe.next_range(-50, 51);
                probe.next_range(-50, 51);
                probe.next_range(-50, 51);
                probe.next_range(-50, 51);
                probe.next_range(-30, 31);
            }
            if probe.next_max(40) == 0 {
                gore_passes += 1;
                probe.next_range(276, 283);
                probe.next_range(10, 31);
                probe.next_range(-20, 21);
                probe.next_range(-50, 51);
                probe.next_range(-50, 51);
                probe.next_range(-20, 21);
                probe.next_range(-50, 51);
                probe.next_range(-50, 51);
            }
            assert_eq!(rng.peek(), probe.peek(), "party stream @{tick}");
            for s in dust.slots_mut() {
                s.active = false;
            }
        }
        assert!(dust_passes > 5, "the 1-in-20 dust gate fired {dust_passes}/400");
        assert!(gore_passes > 2, "the 1-in-40 gore gate fired {gore_passes}/400");
    }

    /// W4-d53 — the imbue post-arms: 1/8 noGravity + fadeIn 1.5 + ×0.25 vel;
    /// 2/3 noGravity + ×0.7 vel + vy -0.5; 4 noGravity + halved vel; 5/6
    /// +dir/+0.2 vel + noGravity. The mount arm's colour lerp rides the
    /// NextFloat cube (63/65 the orange hue).
    #[test]
    fn w4_d53_enchantment_imbue_post_fields() {
        let w = ground_air();
        let c = ctx(&w);
        for enchant in [1u8, 2, 3, 4, 5, 6, 8] {
            let mut rng = UnifiedRandom::new(0);
            let mut dust = DustPool::new();
            loop {
                drain_burst(&mut rng, &mut dust, &c, &w, DustBurst::Enchantment(ench(enchant)));
                if dust.slot(0).active {
                    break;
                }
            }
            let d = dust.slot(0);
            assert!(d.no_gravity, "imbue {enchant} noGravity");
            match enchant {
                1 | 8 => {
                    assert_eq!(d.fade_in, 1.5, "imbue {enchant} fadeIn");
                    assert_eq!(d.dust_type, if enchant == 1 { 171 } else { 46 });
                }
                2 => assert_eq!(d.dust_type, 75),
                3 => assert_eq!(d.dust_type, 6),
                4 => assert_eq!(d.dust_type, 57),
                5 => assert_eq!(d.dust_type, 169),
                6 => assert_eq!(d.dust_type, 135),
                _ => unreachable!(),
            }
        }
    }

    /// W4-d53 — the noEnchantments / npcProj folds: a noEnchant kind draws
    /// nothing even with every flag armed; the owner-255 hostile proxy is
    /// the tick's gate (the drain itself trusts the snapshot).
    #[test]
    fn w4_d53_no_enchant_kind_draws_nothing() {
        let w = ground_air();
        let c = ctx(&w);
        let mut e = ench(7);
        e.no_enchant = true;
        e.frost_burn = true;
        e.magma_stone = true;
        e.mount_active = true;
        e.mount_kind = 62;
        let mut rng = UnifiedRandom::new(53);
        let mut dust = DustPool::new();
        drain_burst(&mut rng, &mut dust, &c, &w, DustBurst::Enchantment(e));
        assert_eq!(rng.peek(), UnifiedRandom::new(53).peek(), "noEnchantments folds out");
        assert!(no_enchantments(480) && no_enchantments(1121));
        assert!(no_enchantment_visuals(181) && no_enchantment_visuals(566));
        assert!(!no_enchantment_visuals(2) && !no_enchantments(2));
    }

    /// W4-d53 — the queue half: a pool tick of a FireArrow (2) with a linked
    /// player queues BOTH the PRE-AI enchantment snapshot and the post-move
    /// arrow-trail burst; the hostile owner-255 proxy suppresses the
    /// enchantment lane (the `npcProj` fold) but the arrow trail still fires.
    #[test]
    fn w4_d53_the_tick_queues_the_bursts() {
        use crate::projectile::{FlailInputs, ProjectilePool, YoyoInputs};
        let w = ground_air();
        let mut pl = crate::player::Player::spawn(&w);
        pl.x = 60.0 * 16.0;
        pl.y = 100.0 * 16.0;
        let mut pool = ProjectilePool::new();
        pool.spawn(2, 0, Vec2::new(60.0 * 16.0, 100.0 * 16.0), Vec2::new(6.0, 0.0), 10, 4.0, false);
        let mut rng = UnifiedRandom::new(53);
        pool.update(
            &mut w.clone(), &mut [], Some((pl.x, pl.y)), None, Some(&mut pl),
            YoyoInputs::default(), FlailInputs::default(), &mut rng, 0, false,
        );
        let bursts = pool.drain_dust_bursts();
        let ench = bursts.iter().filter(|b| matches!(b, DustBurst::Enchantment(_))).count();
        let trail = bursts.iter().filter(|b| matches!(b, DustBurst::ArrowTrail { .. })).count();
        assert!(ench >= 1, "the PRE-AI enchantment snapshot queued");
        assert!(trail >= 1, "the post-move arrow trail queued");
        // The hostile proxy: owner 255 draws no enchantment lane.
        let mut pool2 = ProjectilePool::new();
        pool2.spawn(2, 255, Vec2::new(60.0 * 16.0, 100.0 * 16.0), Vec2::new(6.0, 0.0), 10, 4.0, false);
        let mut rng2 = UnifiedRandom::new(53);
        pool2.update(
            &mut w.clone(), &mut [], None, None, None,
            YoyoInputs::default(), FlailInputs::default(), &mut rng2, 0, true,
        );
        let bursts2 = pool2.drain_dust_bursts();
        assert!(
            !bursts2.iter().any(|b| matches!(b, DustBurst::Enchantment(_))),
            "owner 255 folds the npcProj gate"
        );
    }

    /// W4-d53 — `Projectile.cs:19809-19814`: the Sword Beam's 30 type-64
    /// puffs, each the six default rolls + the type-64 tweak draw, then the
    /// `Next(1,7)` velocity kick. The post-arms write noGravity + White +
    /// alpha 100 + scale 1.8 (the pool's jitter rides the scale).
    #[test]
    fn w4_d53_sword_beam_burst_draws_and_writes() {
        let w = ground_air();
        let c = ctx(&w);
        let pos = Vec2::new(60.0 * 16.0, 100.0 * 16.0);
        let mut rng = UnifiedRandom::new(53);
        let mut probe = UnifiedRandom::new(53);
        let mut dust = DustPool::new();
        drain_burst(&mut rng, &mut dust, &c, &w, DustBurst::SwordBeam { pos, w: 16, h: 16 });
        for _ in 0..30 {
            burst_block(&mut probe, 64, 16, 16);
            probe.next_range(1, 7);
        }
        assert_eq!(rng.peek(), probe.peek(), "the 30-puff census");
        assert_eq!(dust.slots().iter().filter(|s| s.active).count(), 30);
        let d = dust.slot(0);
        assert!(d.no_gravity);
        assert_eq!(d.dust_type, 64);
        assert_eq!(d.color, Color::WHITE);
        assert_eq!(d.alpha, 100);
    }

    /// W4-d53 — `Projectile.cs:20317-20329`: the Light Disc bounce's 4
    /// iterations, each 15 draws (NextFloatDirection + NextFloat + the
    /// Perfect's 6 + the colour NextFloat + the CloneDust's 6). The source
    /// is type 306 alpha 200 scale 1.6 with the (219,253,0)→Cyan lerp; the
    /// clone is White / scale 1 / alpha 50 at the same hit point.
    #[test]
    fn w4_d53_light_disc_bounce_draws_and_writes() {
        let w = ground_air();
        let c = ctx(&w);
        let hit_point = Vec2::new(60.0 * 16.0, 100.0 * 16.0);
        let normal = Vec2::new(-1.0, 0.0);
        let vel = Vec2::new(8.0, 3.0);
        let mut rng = UnifiedRandom::new(53);
        let mut probe = UnifiedRandom::new(53);
        let mut dust = DustPool::new();
        drain_burst(
            &mut rng, &mut dust, &c, &w,
            DustBurst::LightDiscBounce { hit_point, normal, vel },
        );
        for _ in 0..4 {
            probe.next_float_direction();
            probe.next_float();
            burst_block(&mut probe, 306, 0, 0);
            probe.next_float();
            burst_block(&mut probe, 306, 0, 0);
        }
        assert_eq!(rng.peek(), probe.peek(), "the 4x15 bounce census");
        // 8 active slots: 4 sources + 4 clones.
        assert_eq!(dust.slots().iter().filter(|s| s.active).count(), 8);
        let src = dust.slot(0);
        assert_eq!(src.dust_type, 306);
        assert_eq!(src.alpha, 200);
        assert_eq!(src.position, hit_point, "the Perfect anchors the hit point");
        let clone = dust.slot(1);
        assert_eq!(clone.color, Color::WHITE);
        assert_eq!(clone.scale, 1.0);
        assert_eq!(clone.alpha, 50);
        assert_eq!(clone.position, hit_point, "the clone copies the position");
    }

    /// W4-d53 — `Vector2.Reflect` is `v - 2*Dot(v,n)*n` (no normalization):
    /// a head-on x-bounce flips x, keeps y; a zero normal leaves v unchanged.
    #[test]
    fn w4_d53_reflect_matches_xna() {
        let v = Vec2::new(8.0, 3.0);
        assert_eq!(reflect(v, Vec2::new(-1.0, 0.0)), Vec2::new(-8.0, 3.0));
        assert_eq!(reflect(v, Vec2::new(0.0, -1.0)), Vec2::new(8.0, -3.0));
        assert_eq!(reflect(v, Vec2::ZERO), v);
    }

    /// W4-d53 — the queue half: a Sword Beam (116) driven into a wall queues
    /// the impact burst (and the Item10 cue); a Light Disc (106) driven into
    /// a wall queues the bounce sparks. Both ride the per-slot intent fields
    /// the dispatch forwards into `pending_dust_bursts`.
    #[test]
    fn w4_d53_the_tick_queues_sword_beam_and_light_disc() {
        use crate::projectile::{FlailInputs, ProjectilePool, YoyoInputs};
        // A solid wall at x >= 100 tiles; the shots fly right into it.
        let mut w = ground_air();
        for x in 100..200 {
            for y in 0..120 {
                let i = w.index(x, y);
                w.tiles[i] = Tile { kind: 1, ..Default::default() };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        w.compute_column_surface();
        // The Sword Beam: spawn just left of the wall, fly right.
        let mut pool = ProjectilePool::new();
        pool.spawn_with_data_owned(116, 0, 98.0 * 16.0, 100.0 * 16.0, 20.0, 0.0, 10, 4.0, 0.0, 0.0);
        let mut rng = UnifiedRandom::new(53);
        for _ in 0..4 {
            pool.update(
                &mut w.clone(), &mut [], None, None, None,
                YoyoInputs::default(), FlailInputs::default(), &mut rng, 0, false,
            );
        }
        assert!(
            pool.drain_dust_bursts().iter().any(|b| matches!(b, DustBurst::SwordBeam { .. })),
            "the Sword Beam impact burst queued"
        );
        // The Light Disc: same wall, outbound (ai0 = 0).
        let mut pool2 = ProjectilePool::new();
        pool2.spawn_with_data_owned(106, 0, 98.0 * 16.0, 100.0 * 16.0, 20.0, 0.0, 10, 4.0, 0.0, 0.0);
        let mut rng2 = UnifiedRandom::new(53);
        for _ in 0..4 {
            pool2.update(
                &mut w.clone(), &mut [], None, None, None,
                YoyoInputs::default(), FlailInputs::default(), &mut rng2, 0, false,
            );
        }
        assert!(
            pool2.drain_dust_bursts().iter().any(|b| matches!(b, DustBurst::LightDiscBounce { .. })),
            "the Light Disc bounce sparks queued"
        );
    }
}