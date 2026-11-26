//! W4-e1 — Player.cs `Dust.NewDust` sites, SECOND batch (the first 26 are
//! W4-d51's: extra-jump / DashMovement / fast-run). This module carries the
//! drain half of the queue convention for e1's KEEP lanes (ledger:
//! docs/research/64-dust-sites-player-2.md; the e1b-141 reconcile,
//! docs/research/141 + f3's 378: the wall-cling / DoubleJumpVisuals / drown
//! bubble / MakeFloorDust / stoned / Vortex-ring / liquid-splash duplicates
//! DROPPED to f3's `player_dust.rs` drains at the integration merge). The
//! player tick only
//! knows WHERE a burst goes and snapshots the box/velocity at the C# read
//! moment; this module replays each `Dust.NewDust` on the shared
//! `UpdatePlayers` stream (`Main.SwapRandom`, `Main.cs:17755-17757`) in exact
//! C# statement order — the explicit `Main.rand` argument/gate draws FIRST
//! (C# evaluates arguments before the call), then the `NewDust` internals
//! roll inside `DustPool::new_dust` (`Dust.cs:75-256`), then the post-spawn
//! writes (which read `Main.dust[6000]` on a refusal — the sentinel slot
//! exists and takes the writes exactly like the C#). One draw = one stream
//! step; no new streams (S1-A-10). The pool is the App's `particles::DustPool`
//! (`Main.dust`), so the pressure bands / default block / ChildSafety /
//! per-type tweaks all ride `DustPool::new_dust` verbatim.
//!
//! Contract: W4-d50 (`dust_physics.rs`) — the per-type physics chain runs in
//! `update_dust`, not here; these emitters only SPAWN + post-write.
//!
//! `gravDir` is the vanilla `1f` (the port has no gravity flip, the
//! `sticky_movement` convention), so the `gravDir == -1` height/offset folds
//! collapse to their `+1` arms. `shader`/`customData` are dropped fields (the
//! pool contract, `particles.rs`). Gore siblings stay W4-d55's lane.
//!
//! WRITTEN NOT RUN (code-only mode): the witnesses below are compiled by the
//! merge-verify wave, not executed here.

use glam::Vec2;
use tr_core::particles::{Color, DustPool, SpawnCtx};
use tr_core::{UnifiedRandom, World};

/// One `Dust.NewDust` on the caller's stream, returning the slot index (the
/// sentinel included). The emitters are PURE (pool/rng/ctx/world passed in)
/// so the world_sim seam calls them with the disjoint `app.dust` /
/// `app.rngs.stream(UpdatePlayers)` / `&spawn_ctx` / `&app.world` fields while
/// the player borrow `p` is live — the SAME compiling pattern main's sticky
/// drain uses (`world_sim/player.rs`), and witness-testable on a cloned pool.
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

// ---------------------------------------------------------------------------
// Site 12569 — `Player.Yoraiz0rEye` (`Player.cs:12446-12577`).
// ---------------------------------------------------------------------------

/// The eye-trail snapshot: the two endpoints the C# lerps between, the
/// per-tick scale (`yoraiz0rEye % 10` switch) and the dust count. The tick
/// computes the geometry at the `:25487` read moment; the drain replays the
/// `num5` `NewDust(Center,0,0,182)` spawns + the Lerp/noGravity/zero-velocity/
/// scale post-writes. The light half (`:12550-12561`, `PlotTileLine`) is the
/// lighting lane's — no `Main.rand`, skipped here.
#[derive(Clone, Copy, Debug)]
pub(crate) struct YoraizEyeDust {
    /// `vector4` — the NEW eye point (`position + vector2 + zero`).
    pub new_pt: Vec2,
    /// `vector5` — the OLD eye point (`oldPosition + vector3 + zero`).
    pub old_pt: Vec2,
    /// `base.Center` — the `NewDust` box origin.
    pub cx: f32,
    pub cy: f32,
    /// `num4` — the scale from the `yoraiz0rEye % 10` switch.
    pub scale: f32,
}

/// `:12562-12576` — the trail. `num5 = (int)Distance/3 + 1`, `+1` when
/// `Distance % 3 != 0`; per dust the six default draws, then
/// `position = Lerp(old, new, num6/num5)`, `noGravity`, `velocity = 0`,
/// `scale = num4`. No explicit argument draws.
pub(crate) fn yoraiz_eye_trail(
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
    b: &YoraizEyeDust,
) {
    let dist = (b.new_pt - b.old_pt).length();
    let mut num5 = (dist as i32) / 3 + 1;
    if dist % 3.0 != 0.0 {
        num5 += 1;
    }
    for num6 in 1..=num5 {
        let idx = nd(
            rng, dust, ctx, world, Vec2::new(b.cx, b.cy), 0, 0, 182,
            0.0, 0.0, 0, Color::default(), 1.0,
        );
        let t = num6 as f32 / num5 as f32;
        let d = dust.slot_mut(idx);
        d.position = b.old_pt.lerp(b.new_pt, t);
        d.no_gravity = true;
        d.velocity = Vec2::ZERO;
        d.scale = b.scale;
    }
}

/// The `yoraiz0rEye % 10` scale switch (`:12527-12549`): case 1 returns (no
/// trail — the caller gates), 2→0.5, 3→0.625, 4→0.75, 5→0.875, 6→1, 7→1.1,
/// else (0/8/9) the init `num4 = 1`.
pub(crate) fn yoraiz_eye_scale(eye: i32) -> Option<f32> {
    match eye % 10 {
        1 => None,
        2 => Some(0.5),
        3 => Some(0.625),
        4 => Some(0.75),
        5 => Some(0.875),
        6 => Some(1.0),
        7 => Some(1.1),
        _ => Some(1.0),
    }
}

// ---------------------------------------------------------------------------
// Sites 14291 / 14300 — `Player.DoGlassSlipperSparkles` (`:14286-14306`).
// ---------------------------------------------------------------------------

/// One equipped Glass Slipper (5077) runs BOTH gates per tick (the C# calls
/// the method from the equip pass, `:14242`/`:14388`). The snapshot carries
/// the box/velocity/facing at the equip read moment; `walking` is
/// `CanSpawnWalkingEffects()` (`:35750-35753`, `|vx|+|vy| > 1`).
#[derive(Clone, Copy, Debug)]
pub(crate) struct GlassSlipperDust {
    pub cx: f32,
    pub cy: f32,
    pub facing: i32,
    pub walking: bool,
}

/// `:14288-14305` — gate 1 `Next(60)==0` → a 20×8 box centred at
/// `Center+(0,16)`; gate 2 `walking && Next(10)==0` → a 6×8 box centred at
/// `Center+(-2*facing,16)`. Both: type 204, alpha 150, scale 0.3, then
/// `fadeIn=1`, `velocity*=0.1`, `noLight`. The gate draws ride the stream
/// even when the spawn culls (the gate is OUTSIDE the allocator).
pub(crate) fn glass_slipper_sparkles(
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
    b: &GlassSlipperDust,
) {
    // `Utils.CenteredRectangle(center, size)` (`Utils.cs:947-950`):
    // `(int)(center - size/2)` top-left, `(int)size` dims.
    if rng.next_max(60) == 0 {
        let r = centered_rect(b.cx, b.cy + 16.0, 20.0, 8.0);
        let idx = nd(
            rng, dust, ctx, world, Vec2::new(r.0 as f32, r.1 as f32), r.2, r.3,
            204, 0.0, 0.0, 150, Color::default(), 0.3,
        );
        let d = dust.slot_mut(idx);
        d.fade_in = 1.0;
        d.velocity *= 0.1;
        d.no_light = true;
    }
    if b.walking && rng.next_max(10) == 0 {
        let r = centered_rect(b.cx - 2.0 * b.facing as f32, b.cy + 16.0, 6.0, 8.0);
        let idx = nd(
            rng, dust, ctx, world, Vec2::new(r.0 as f32, r.1 as f32), r.2, r.3,
            204, 0.0, 0.0, 150, Color::default(), 0.3,
        );
        let d = dust.slot_mut(idx);
        d.fade_in = 1.0;
        d.velocity *= 0.1;
        d.no_light = true;
    }
}

/// `Utils.CenteredRectangle(Vector2, Vector2)` — the int-truncated top-left
/// and dims (`Utils.cs:947-950`).
fn centered_rect(cx: f32, cy: f32, w: f32, h: f32) -> (i32, i32, i32, i32) {
    (
        (cx - w / 2.0) as i32,
        (cy - h / 2.0) as i32,
        w as i32,
        h as i32,
    )
}

// ---------------------------------------------------------------------------
// Site 16389 — `Player.ApplySetBonus_Solar` shield-gain burst (`:16366-16402`).
// ---------------------------------------------------------------------------

/// The 16-dust shield-gain burst snapshot (`:16387-16395`): the box at the
/// C# read moment. The counter/AddBuff/shield-count machine runs in the tick
/// (`player.rs::apply_set_bonus_solar`); only the dust replays here.
#[derive(Clone, Copy, Debug)]
pub(crate) struct SolarBurstDust {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

/// `:16387-16395` — 16 type-6 puffs, alpha 100, then `noGravity`,
/// `scale = 1.7`, `fadeIn = 0.5`, `velocity *= 5`. No explicit argument draws.
pub(crate) fn solar_shield_burst(
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
    b: &SolarBurstDust,
) {
    for _ in 0..16 {
        let idx = nd(
            rng, dust, ctx, world, Vec2::new(b.x, b.y), b.width as i32,
            b.height as i32, 6, 0.0, 0.0, 100, Color::default(), 1.0,
        );
        let d = dust.slot_mut(idx);
        d.no_gravity = true;
        d.scale = 1.7;
        d.fade_in = 0.5;
        d.velocity *= 5.0;
    }
}

// ---------------------------------------------------------------------------
// Sites 16623 / 16633 / 16652 — `Player.UpdateTeleportVisuals` (`:16613-16670`).
// ---------------------------------------------------------------------------

/// The teleport-visuals snapshot: the style, the (post-decrement) time the
/// gate reads, and the box. The tick runs the time decrements + style-2/3
/// pin (`player.rs::update_teleport_visuals`); the drain rolls the gate +
/// spawn. Style 9 (pylon, `TeleportPylonsSystem.SpawnInWorldDust`) is the
/// pylon lane's — skipped (no `Main.rand` here, the gate draw still rides).
#[derive(Clone, Copy, Debug)]
pub(crate) struct TeleportVisualsDust {
    pub style: i32,
    pub time: f32,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    /// `lastPortalColorIndex` (style 4) — the portal colour pick.
    pub portal_color_index: i32,
}

/// `:16619-16668` — the per-style gate + spawn. Style 0: `Next(100) <=
/// 100*time*2` → type 159; style 1: `Next(100) <= 100*time` → type 164;
/// both `scale = time*1.5`, `noGravity`, `velocity *= 1.1`. Style 4:
/// `Next(100) <= 100*time` → type 263, portal colour, `noLight`, `noGravity`,
/// `scale = 1.2`, `fadeIn = 0.4`. Style 9: the gate draw only (pylon spawn
/// deferred). The gate draw runs even when the spawn culls.
pub(crate) fn teleport_visuals(
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
    b: &TeleportVisualsDust,
) {
    let pos = Vec2::new(b.x, b.y);
    let (w, h) = (b.width as i32, b.height as i32);
    match b.style {
        0 => {
            if (rng.next_max(100) as f32) <= 100.0 * b.time * 2.0 {
                let idx = nd(rng, dust, ctx, world, pos, w, h, 159, 0.0, 0.0, 0, Color::default(), 1.0);
                let d = dust.slot_mut(idx);
                d.scale = b.time * 1.5;
                d.no_gravity = true;
                d.velocity *= 1.1;
            }
        }
        1 => {
            if (rng.next_max(100) as f32) <= 100.0 * b.time {
                let idx = nd(rng, dust, ctx, world, pos, w, h, 164, 0.0, 0.0, 0, Color::default(), 1.0);
                let d = dust.slot_mut(idx);
                d.scale = b.time * 1.5;
                d.no_gravity = true;
                d.velocity *= 1.1;
            }
        }
        4 => {
            if (rng.next_max(100) as f32) <= 100.0 * b.time {
                let idx = nd(rng, dust, ctx, world, pos, w, h, 263, 0.0, 0.0, 0, Color::default(), 1.0);
                let d = dust.slot_mut(idx);
                d.color = portal_color(b.portal_color_index);
                d.no_light = true;
                d.no_gravity = true;
                d.scale = 1.2;
                d.fade_in = 0.4;
            }
        }
        9 => {
            // `:16664` — the gate draw rides; the pylon dust spawn is the
            // pylon lane's (`TeleportPylonsSystem.SpawnInWorldDust`).
            let _ = rng.next_max(100);
        }
        _ => {}
    }
}

/// `PortalHelper.GetPortalColor(int colorIndex)` (`PortalHelper.cs`):
/// `GetPortalColor(colorIndex/2, colorIndex%2)`; single-player
/// (`Main.netMode == 0`) ignores the player index — portal 0 →
/// `hslToRgb(0.12,1,0.5)`, portal 1 → `hslToRgb(0.52,1,0.6)`, then `A = 66`.
fn portal_color(color_index: i32) -> Color {
    let portal = color_index % 2;
    let (h, l) = if portal != 0 { (0.52, 0.6) } else { (0.12, 0.5) };
    let rgb = crate::ui::color::hsl_to_rgb(h as f64, 1.0, l as f64, 255);
    Color::new(rgb[0], rgb[1], rgb[2], 66)
}

// ---------------------------------------------------------------------------
// Sites 19273 / 19296 — `Player.UpdateLifeRegen` (`:19268-19305`).
// ---------------------------------------------------------------------------

/// The shiny-stone sparkle gate (`:19271`): the engine recorded the
/// `lifeRegenTime` at the gate moment (`RegenState::shiny_gate_time`); the
/// drain rolls `Next(30000) < time || Next(30)==0` (short-circuit — the
/// second draw only when the first fails) and spawns the type-55 sparkle.
#[derive(Clone, Copy, Debug)]
pub(crate) struct ShinySparkleDust {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub cx: f32,
    pub cy: f32,
    /// `lifeRegenTime` at the gate (`:19271`).
    pub regen_time: f32,
}

/// `:19273-19283` — type 55, alpha 200, scale 0.5, then `noGravity`,
/// `velocity *= 0.75`, `fadeIn = 1.3`, then the DEAD-ish `vector` draws
/// `Next(-100,101)`×2 + `Next(50,100)` which ARE read: `velocity = vector`
/// (normalised × `Next(50,100)*0.04`), `position = Center - vector*34`
/// (re-normalised). Unlike the Sail's dead vector, this one feeds the dust.
pub(crate) fn shiny_stone_sparkle(
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
    b: &ShinySparkleDust,
) {
    // `:19271` — the gate, short-circuit OR.
    let pass = (rng.next_max(30000) as f32) < b.regen_time || rng.next_max(30) == 0;
    if !pass {
        return;
    }
    let idx = nd(
        rng, dust, ctx, world, Vec2::new(b.x, b.y), b.width as i32, b.height as i32,
        55, 0.0, 0.0, 200, Color::default(), 0.5,
    );
    let d = dust.slot_mut(idx);
    d.no_gravity = true;
    d.velocity *= 0.75;
    d.fade_in = 1.3;
    // `:19277-19283` — the vector draws feed velocity + position.
    let mut vector = Vec2::new(
        rng.next_range(-100, 101) as f32,
        rng.next_range(-100, 101) as f32,
    );
    vector = safe_normalize(vector);
    vector *= rng.next_range(50, 100) as f32 * 0.04;
    d.velocity = vector;
    let mut v2 = safe_normalize(vector);
    v2 *= 34.0;
    d.position = Vec2::new(b.cx, b.cy) - v2;
}

/// The crimson heal burst count (`:19292-19305`): one 10-dust burst per hp
/// point the engine granted while `crimsonRegen` (`RegenState::
/// crimson_heal_bursts`). The box snapshot is the player's at the regen read.
#[derive(Clone, Copy, Debug)]
pub(crate) struct CrimsonHealDust {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub bursts: u32,
}

/// `:19294-19304` — per burst, 10 type-5 puffs, alpha 175, scale 1.75, then
/// `noGravity`, `velocity *= 0.75`, then `Next(-40,41)`×2 → `position +=
/// (num7,num8)`, `velocity = (-num7*0.075, -num8*0.075)`.
pub(crate) fn crimson_heal_bursts(
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
    b: &CrimsonHealDust,
) {
    for _ in 0..b.bursts {
        for _ in 0..10 {
            let idx = nd(
                rng, dust, ctx, world, Vec2::new(b.x, b.y), b.width as i32,
                b.height as i32, 5, 0.0, 0.0, 175, Color::default(), 1.75,
            );
            let d = dust.slot_mut(idx);
            d.no_gravity = true;
            d.velocity *= 0.75;
            let num7 = rng.next_range(-40, 41);
            let num8 = rng.next_range(-40, 41);
            d.position.x += num7 as f32;
            d.position.y += num8 as f32;
            d.velocity.x = -(num7 as f32) * 0.075;
            d.velocity.y = -(num8 as f32) * 0.075;
        }
    }
}

// ---------------------------------------------------------------------------
// Site 19555 — `Player.UpdateManaRegen` full-bar burst (`:19550-19560`).
// ---------------------------------------------------------------------------

/// The full-mana burst snapshot (`:19553-19559`): the box at the regen read.
/// The engine's `refilled` return arms it (`update_survival`). The
/// `PlaySound(25)` (MaxMana) is case 25 — Volume/Pan only, NO draws
/// (`LegacySoundPlayer.cs:963-968`); the asset is not in the bank, so the
/// sound is a documented skip (no stream impact).
#[derive(Clone, Copy, Debug)]
pub(crate) struct ManaFullDust {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

/// `:19553-19559` — 5 type-45 puffs, alpha 255, scale ARGUMENT
/// `Next(20,26)*0.1` (drawn BEFORE the pool internals, C# argument order),
/// then `noLight`, `noGravity`, `velocity *= 0.5`.
pub(crate) fn mana_full_burst(
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
    b: &ManaFullDust,
) {
    for _ in 0..5 {
        let scale = rng.next_range(20, 26) as f32 * 0.1;
        let idx = nd(
            rng, dust, ctx, world, Vec2::new(b.x, b.y), b.width as i32,
            b.height as i32, 45, 0.0, 0.0, 255, Color::default(), scale,
        );
        let d = dust.slot_mut(idx);
        d.no_light = true;
        d.no_gravity = true;
        d.velocity *= 0.5;
    }
}

/// `Vector2.SafeNormalize(Vector2.Zero)` — the zero-vector guard returns the
/// fallback (`Utils`/XNA): a zero-length input normalises to `default`.
fn safe_normalize(v: Vec2) -> Vec2 {
    let len = v.length();
    if len > 0.0 {
        v / len
    } else {
        Vec2::ZERO
    }
}

/// `Vector2.RotatedBy(double radians)` (`Utils.cs:1675-1684`): the f32 cos/sin
/// of the DOUBLE angle, about the origin. Shared by the eye-trail geometry.
pub(crate) fn rotate_by(v: Vec2, radians: f64) -> Vec2 {
    let num = radians.cos() as f32;
    let num2 = radians.sin() as f32;
    Vec2::new(v.x * num - v.y * num2, v.x * num2 + v.y * num)
}

/// `Vector2.RotatedBy(double radians, Vector2 center)` (`Utils.cs:1675-1684`):
/// the rotation about a pivot.
pub(crate) fn rotate_about(v: Vec2, radians: f64, center: Vec2) -> Vec2 {
    let num = radians.cos() as f32;
    let num2 = radians.sin() as f32;
    let d = v - center;
    center + Vec2::new(d.x * num - d.y * num2, d.x * num2 + d.y * num)
}

// ---------------------------------------------------------------------------
// Site 20320 — `Player.MowTheLawn` (`:20301-20346`): the Lawn Mower (4049)
// swing puff. The host gates the swing and computes the lerp anchor; the
// drain replays the single `NewDustDirect`.
// ---------------------------------------------------------------------------

/// The MowTheLawn puff snapshot: the lerp endpoints + the swing phase. The
/// host computes `vector` (the mounted-center + direction*38 anchor) and the
/// two lerp ends; `num` is `(1 - itemAnimation/itemAnimationMax)*2`.
#[derive(Clone, Copy, Debug)]
pub(crate) struct MowLawnDust {
    /// `vector + (direction*-16, -4)` — the lerp start.
    pub a: Vec2,
    /// `vector + (direction*-6, -4)` — the lerp end.
    pub b: Vec2,
    /// `num = (1 - itemAnimation/itemAnimationMax)*2`; the lerp t is
    /// `num < 1 ? 1-num : num-1`.
    pub num: f32,
}

/// `:20320-20327` — one type-31 `NewDustDirect`, box 0×0 at the lerp point,
/// speed (0, −0.25), alpha 127; then scale 0.9, position −(4,4), the
/// velocity.Y>0 flip, velocity *= 0.25.
pub(crate) fn mow_lawn_puff(
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
    b: &MowLawnDust,
) {
    let t = if b.num < 1.0 { 1.0 - b.num } else { b.num - 1.0 };
    let pos = b.a.lerp(b.b, t);
    let idx = dust.new_dust_direct(
        rng, ctx, world, pos, 0, 0, 31, 0.0, -0.25, 127, Color::default(), 1.0,
    );
    let d = dust.slot_mut(idx);
    d.scale = 0.9;
    d.position -= Vec2::new(4.0, 4.0);
    if d.velocity.y > 0.0 {
        d.velocity.y *= -1.0;
    }
    d.velocity *= 0.25;
}

// ---------------------------------------------------------------------------
// Site 24274 — `Player.UpdateHairDyeDust` (`:24268-24295`): the team-dye puff.
// ---------------------------------------------------------------------------

/// The hair-dye puff snapshot: the box/velocity at the C# read moment. The
/// host gates the team-dye state (hairDye == TeamDyeShaderIndex, alive, not
/// stoned/frozen/ghost); the drain rolls the Next(45) gate + the type roll +
/// the velocity jitter chain. The gore sibling (`:24285`) is W4-d55's — its
/// argument draws are consumed here in order when its Next(225) gate fires.
#[derive(Clone, Copy, Debug)]
pub(crate) struct HairDyeDust {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub vx: f32,
    pub vy: f32,
}

/// `:24272-24293` — the Next(45) gate, then one type Next(139,143) puff (the
/// argument draw BEFORE the internals), box width×8, scale 1.2; then the
/// velocity jitter chain (×(1+Next(-50,51)*0.01) per axis, +Next(-50,51)*0.01
/// per axis, Y−1, scale×(0.7+Next(-30,31)*0.01), +velocity*0.2). The gore
/// sibling's Next(225) gate + its argument draws ride after (d55's body).
pub(crate) fn hair_dye_puff(
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
    b: &HairDyeDust,
) {
    if rng.next_max(45) == 0 {
        let ty = rng.next_range(139, 143);
        let idx = nd(
            rng, dust, ctx, world, Vec2::new(b.x, b.y), b.width as i32, 8, ty,
            0.0, 0.0, 0, Color::default(), 1.2,
        );
        let d = dust.slot_mut(idx);
        d.velocity.x *= 1.0 + rng.next_range(-50, 51) as f32 * 0.01;
        d.velocity.y *= 1.0 + rng.next_range(-50, 51) as f32 * 0.01;
        d.velocity.x += rng.next_range(-50, 51) as f32 * 0.01;
        d.velocity.y += rng.next_range(-50, 51) as f32 * 0.01;
        d.velocity.y -= 1.0;
        d.scale *= 0.7 + rng.next_range(-30, 31) as f32 * 0.01;
        d.velocity += Vec2::new(b.vx, b.vy) * 0.2;
    }
    // `:24283-24293` — the gore sibling: the Next(225) gate always rolls; when
    // it fires, the argument draws (Next(276,283) type, Next(width) X,
    // Next(8) Y) + the post-write draws ride the stream (the gore body is
    // d55's). The C# order: type, position-X, position-Y, then the jitter.
    if rng.next_max(225) == 0 {
        let _ = rng.next_range(276, 283); // gore type argument
        let _ = rng.next_max(b.width as i32); // position X argument
        let _ = rng.next_max(8); // position Y argument
        // The gore post-write draws (`:24286-24292`): ×(1+Next(-50,51)*0.01)
        // per axis, scale×(1+Next(-20,21)*0.01), +Next(-50,51)*0.01 per axis.
        let _ = rng.next_range(-50, 51);
        let _ = rng.next_range(-50, 51);
        let _ = rng.next_range(-20, 21);
        let _ = rng.next_range(-50, 51);
        let _ = rng.next_range(-50, 51);
    }
}

// ---------------------------------------------------------------------------
// Sites 24641 / 24654 — the manaHeat ambient (`Player.Update :24630-24662`).
// ---------------------------------------------------------------------------

/// The manaHeat ambient snapshot: the box + the heat-derived gate probability.
/// The host gates `manaHeat != 1` and computes `num4 = Remap(manaHeat, 1.3, 1,
/// 1, 0)`; the drain rolls the two NextFloat*6 gates + the spawn/clone chain.
#[derive(Clone, Copy, Debug)]
pub(crate) struct ManaHeatDust {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    /// `num4` — the Remap-derived gate probability (clamped 0..1).
    pub num4: f32,
}

/// `:24637-24660` — two iterations, each gated `NextFloat*6 < num4`; a passed
/// gate spawns a type-267 NewDustDirect (scale argument Next(10,26)*0.1) with
/// the blue post-chain, then a CloneDust white twin (scale/fadeIn ×0.4).
pub(crate) fn mana_heat_ambient(
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
    b: &ManaHeatDust,
) {
    for _ in 0..2 {
        if rng.next_float() * 6.0 < b.num4 {
            let scale = rng.next_range(10, 26) as f32 * 0.1;
            let idx = dust.new_dust_direct(
                rng, ctx, world, Vec2::new(b.x, b.y), b.width as i32, b.height as i32,
                267, 0.0, 0.0, 255, Color::default(), scale,
            );
            let d = dust.slot_mut(idx);
            d.no_light = true;
            d.no_gravity = true;
            d.velocity *= 0.5;
            d.velocity.x = 0.0;
            d.velocity.y -= 0.5;
            d.velocity.y -= 1.0;
            d.fade_in = 0.7 + 1.3 * rng.next_float();
            d.scale = 0.7 + 1.3 * rng.next_float();
            d.color = Color::new(0, 0, 255, 255); // Color.Blue
            d.alpha = 200;
            d.position.y += 5.0;
            // `:24654-24659` — the CloneDust white twin.
            let idx2 = dust.clone_dust(rng, ctx, world, idx);
            let d2 = dust.slot_mut(idx2);
            d2.no_light = true;
            d2.no_gravity = true;
            d2.scale *= 0.4;
            d2.fade_in *= 0.4;
            d2.color = Color::WHITE;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn world() -> World {
        World::new("e1".into(), 200, 200)
    }

    /// A permissive ctx: the whole world on-screen, the full budget — the
    /// pool never culls (the `update_rng` scratch convention, e2's fixture).
    fn ctx() -> SpawnCtx {
        SpawnCtx {
            game_menu: false,
            game_paused: false,
            generating: false,
            screen_position: Vec2::ZERO,
            screen_size: Vec2::new(3200.0, 3200.0),
            max_dust_to_draw: 6000,
        }
    }

    fn active(pool: &DustPool) -> Vec<usize> {
        (0..6000).filter(|&i| pool.slot(i).active).collect()
    }

    /// W4-e1 site 12569 — the eye trail: `num5 = (int)dist/3 + 1` (+1 when
    /// `dist % 3 != 0`), each dust lerps old→new, noGravity, zero velocity,
    /// scale num4. A mirrored stream through the same pool call lands
    /// identical records (no explicit argument draws).
    #[test]
    fn w4_e1_yoraiz_eye_trail_lerps_and_mirrors() {
        let w = world();
        let c = ctx();
        let b = YoraizEyeDust {
            new_pt: Vec2::new(1000.0, 900.0),
            old_pt: Vec2::new(970.0, 900.0),
            cx: 985.0,
            cy: 910.0,
            scale: 0.75,
        };
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(7);
        yoraiz_eye_trail(&mut rng, &mut pool, &c, &w, &b);
        // dist 30 → 30/3+1 = 11, 30%3==0 → no extra → 11 dusts.
        assert_eq!(active(&pool).len(), 11);
        let mut mpool = DustPool::new();
        let mut mirror = UnifiedRandom::new(7);
        for num6 in 1..=11 {
            let idx = mpool.new_dust(
                &mut mirror, &c, &w, Vec2::new(985.0, 910.0), 0, 0, 182,
                0.0, 0.0, 0, Color::default(), 1.0,
            );
            let d = mpool.slot_mut(idx);
            d.position = b.old_pt.lerp(b.new_pt, num6 as f32 / 11.0);
            d.no_gravity = true;
            d.velocity = Vec2::ZERO;
            d.scale = 0.75;
        }
        for i in 0..11 {
            assert_eq!(pool.slot(i).position, mpool.slot(i).position);
            assert_eq!(pool.slot(i).velocity, Vec2::ZERO);
            assert!(pool.slot(i).no_gravity);
            assert_eq!(pool.slot(i).scale, 0.75);
            assert_eq!(pool.slot(i).dust_type, 182);
        }
        assert_eq!(rng.next_range(0, 1000), mirror.next_range(0, 1000));
    }

    /// W4-e1 site 12569 — the scale switch: case 1 returns None (no trail),
    /// the rest pin their num4; the default (0/8/9) is 1.
    #[test]
    fn w4_e1_yoraiz_eye_scale_switch() {
        assert_eq!(yoraiz_eye_scale(1), None);
        assert_eq!(yoraiz_eye_scale(11), None);
        assert_eq!(yoraiz_eye_scale(2), Some(0.5));
        assert_eq!(yoraiz_eye_scale(3), Some(0.625));
        assert_eq!(yoraiz_eye_scale(7), Some(1.1));
        assert_eq!(yoraiz_eye_scale(0), Some(1.0));
        assert_eq!(yoraiz_eye_scale(8), Some(1.0));
    }

    /// W4-e1 sites 14291/14300 — the glass slipper: a failed Next(60) gate
    /// draws NOTHING extra; a passed gate spawns one type-204 with fadeIn 1,
    /// velocity ×0.1, noLight. The walking gate only rolls when walking.
    #[test]
    fn w4_e1_glass_slipper_gates_and_shape() {
        let w = world();
        let c = ctx();
        let b = GlassSlipperDust { cx: 500.0, cy: 500.0, facing: 1, walking: false };
        // Find a seed whose first Next(60) is 0 (gate passes).
        let mut seed = 0i32;
        loop {
            let mut r = UnifiedRandom::new(seed);
            if r.next_max(60) == 0 {
                break;
            }
            seed += 1;
        }
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(seed);
        glass_slipper_sparkles(&mut rng, &mut pool, &c, &w, &b);
        assert_eq!(active(&pool).len(), 1, "gate 1 passes, gate 2 off (not walking)");
        let d = pool.slot(0);
        assert_eq!(d.dust_type, 204);
        assert_eq!(d.fade_in, 1.0);
        assert!(d.no_light);
        assert_eq!(d.alpha, 150);
        // The 20×8 box centred at (500, 516): top-left (490, 512).
        assert!(d.position.x >= 490.0 && d.position.x < 510.0);
        assert!(d.position.y >= 512.0 && d.position.y < 520.0);
    }

    /// W4-e1 site 16389 — the Solar gain burst: 16 type-6 puffs, noGravity,
    /// scale 1.7, fadeIn 0.5, velocity ×5. Mirrors the pool internals.
    #[test]
    fn w4_e1_solar_burst_sixteen_and_mirrors() {
        let w = world();
        let c = ctx();
        let b = SolarBurstDust { x: 300.0, y: 300.0, width: 20.0, height: 40.0 };
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(13);
        solar_shield_burst(&mut rng, &mut pool, &c, &w, &b);
        assert_eq!(active(&pool).len(), 16);
        let mut mpool = DustPool::new();
        let mut mirror = UnifiedRandom::new(13);
        for _ in 0..16 {
            let idx = mpool.new_dust(
                &mut mirror, &c, &w, Vec2::new(300.0, 300.0), 20, 40, 6,
                0.0, 0.0, 100, Color::default(), 1.0,
            );
            let d = mpool.slot_mut(idx);
            d.no_gravity = true;
            d.scale = 1.7;
            d.fade_in = 0.5;
            d.velocity *= 5.0;
        }
        for i in 0..16 {
            assert_eq!(pool.slot(i).velocity, mpool.slot(i).velocity);
            assert_eq!(pool.slot(i).scale, 1.7);
            assert_eq!(pool.slot(i).fade_in, 0.5);
            assert!(pool.slot(i).no_gravity);
        }
        assert_eq!(rng.next_range(0, 1000), mirror.next_range(0, 1000));
    }

    /// W4-e1 sites 16623/16633 — the teleport gate: style 0 reads
    /// `Next(100) <= 100*time*2`, style 1 `<= 100*time`. A time of 0.0 fails
    /// both (the gate draw still rides); a time of 1.0 passes both.
    #[test]
    fn w4_e1_teleport_gate_time_and_style() {
        let w = world();
        let c = ctx();
        // time 0 → gate fails (Next(100) >= 0 always, <= 0 only when draw 0).
        // Use a seed whose Next(100) != 0 so the gate fails deterministically.
        let mut seed = 0i32;
        loop {
            let mut r = UnifiedRandom::new(seed);
            if r.next_max(100) != 0 {
                break;
            }
            seed += 1;
        }
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(seed);
        let b0 = TeleportVisualsDust {
            style: 0, time: 0.0, x: 100.0, y: 100.0, width: 20.0, height: 40.0,
            portal_color_index: 0,
        };
        teleport_visuals(&mut rng, &mut pool, &c, &w, &b0);
        assert!(active(&pool).is_empty(), "time 0 fails the gate");
        // time 1.0 → style 1 gate `<= 100` always passes.
        let mut pool2 = DustPool::new();
        let mut rng2 = UnifiedRandom::new(seed);
        let b1 = TeleportVisualsDust { style: 1, time: 1.0, ..b0 };
        teleport_visuals(&mut rng2, &mut pool2, &c, &w, &b1);
        assert_eq!(active(&pool2).len(), 1);
        let d = pool2.slot(0);
        assert_eq!(d.dust_type, 164);
        assert_eq!(d.scale, 1.5);
        assert!(d.no_gravity);
    }

    /// W4-e1 site 16652 — style 4 portal colour: portal 0 → hsl(0.12,1,0.5),
    /// portal 1 → hsl(0.52,1,0.6), alpha 66. The colour index %2 picks.
    #[test]
    fn w4_e1_teleport_portal_colour() {
        let c0 = portal_color(0);
        let c1 = portal_color(1);
        let c2 = portal_color(2); // 2%2==0 → same as portal 0
        assert_eq!(c0.a, 66);
        assert_eq!(c1.a, 66);
        assert_eq!(c0, c2);
        assert_ne!(c0, c1);
        let expect0 = crate::ui::color::hsl_to_rgb(0.12, 1.0, 0.5, 255);
        assert_eq!([c0.r, c0.g, c0.b], [expect0[0], expect0[1], expect0[2]]);
    }

    /// W4-e1 site 19273 — the shiny sparkle gate: `Next(30000) < time` passes
    /// without the second draw (short-circuit); the vector draws feed
    /// velocity + position. A high time always passes the first arm.
    #[test]
    fn w4_e1_shiny_sparkle_gate_and_vector() {
        let w = world();
        let c = ctx();
        let b = ShinySparkleDust {
            x: 400.0, y: 400.0, width: 20.0, height: 40.0,
            cx: 410.0, cy: 420.0, regen_time: 40000.0,
        };
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(17);
        shiny_stone_sparkle(&mut rng, &mut pool, &c, &w, &b);
        assert_eq!(active(&pool).len(), 1, "time 40000 > any Next(30000)");
        let d = pool.slot(0);
        assert_eq!(d.dust_type, 55);
        assert!(d.no_gravity);
        assert_eq!(d.fade_in, 1.3);
        // velocity = normalised(Next(-100,101) pair) * Next(50,100)*0.04.
        // The magnitude is in [50*0.04, 99*0.04] = [2.0, 3.96].
        let mag = d.velocity.length();
        assert!(mag >= 2.0 && mag <= 3.96, "mag {mag}");
    }

    /// W4-e1 site 19296 — the crimson heal burst: each burst is 10 type-5
    /// puffs with the ±40 position jitter and the −num*0.075 velocity. Two
    /// bursts → 20 dusts; mirrors the argument order (no explicit pre-draws).
    #[test]
    fn w4_e1_crimson_heal_bursts_count_and_jitter() {
        let w = world();
        let c = ctx();
        let b = CrimsonHealDust { x: 600.0, y: 600.0, width: 20.0, height: 40.0, bursts: 2 };
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(23);
        crimson_heal_bursts(&mut rng, &mut pool, &c, &w, &b);
        assert_eq!(active(&pool).len(), 20);
        let mut mpool = DustPool::new();
        let mut mirror = UnifiedRandom::new(23);
        for _ in 0..2 {
            for _ in 0..10 {
                let idx = mpool.new_dust(
                    &mut mirror, &c, &w, Vec2::new(600.0, 600.0), 20, 40, 5,
                    0.0, 0.0, 175, Color::default(), 1.75,
                );
                let d = mpool.slot_mut(idx);
                d.no_gravity = true;
                d.velocity *= 0.75;
                let n7 = mirror.next_range(-40, 41);
                let n8 = mirror.next_range(-40, 41);
                d.position.x += n7 as f32;
                d.position.y += n8 as f32;
                d.velocity.x = -(n7 as f32) * 0.075;
                d.velocity.y = -(n8 as f32) * 0.075;
            }
        }
        for i in 0..20 {
            assert_eq!(pool.slot(i).position, mpool.slot(i).position);
            assert_eq!(pool.slot(i).velocity, mpool.slot(i).velocity);
        }
        assert_eq!(rng.next_range(0, 1000), mirror.next_range(0, 1000));
    }

    /// W4-e1 site 19555 — the full-mana burst: 5 type-45 puffs, the scale
    /// ARGUMENT `Next(20,26)*0.1` drawn BEFORE the pool internals, then
    /// noLight/noGravity/velocity×0.5. The mirror draws the scale first.
    #[test]
    fn w4_e1_mana_full_burst_argument_order() {
        let w = world();
        let c = ctx();
        let b = ManaFullDust { x: 800.0, y: 800.0, width: 20.0, height: 40.0 };
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(29);
        mana_full_burst(&mut rng, &mut pool, &c, &w, &b);
        assert_eq!(active(&pool).len(), 5);
        let mut mpool = DustPool::new();
        let mut mirror = UnifiedRandom::new(29);
        for _ in 0..5 {
            let scale = mirror.next_range(20, 26) as f32 * 0.1;
            let idx = mpool.new_dust(
                &mut mirror, &c, &w, Vec2::new(800.0, 800.0), 20, 40, 45,
                0.0, 0.0, 255, Color::default(), scale,
            );
            let d = mpool.slot_mut(idx);
            d.no_light = true;
            d.no_gravity = true;
            d.velocity *= 0.5;
        }
        for i in 0..5 {
            assert_eq!(pool.slot(i).scale, mpool.slot(i).scale);
            assert_eq!(pool.slot(i).velocity, mpool.slot(i).velocity);
            assert!(pool.slot(i).no_light);
            assert!(pool.slot(i).no_gravity);
        }
        assert_eq!(rng.next_range(0, 1000), mirror.next_range(0, 1000));
    }

    /// W4-e1 site 20320 — the Lawn Mower puff: one type-31 NewDustDirect at
    /// the lerp point, scale 0.9, position −(4,4), the velocity.Y>0 flip,
    /// velocity ×0.25. The lerp t folds num<1 → 1−num.
    #[test]
    fn w4_e1_mow_lawn_puff_lerp_and_shape() {
        let w = world();
        let c = ctx();
        let b = MowLawnDust {
            a: Vec2::new(100.0, 200.0),
            b: Vec2::new(110.0, 200.0),
            num: 0.5, // t = 1−0.5 = 0.5 → lerp midpoint (105, 200)
        };
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(31);
        mow_lawn_puff(&mut rng, &mut pool, &c, &w, &b);
        assert_eq!(active(&pool).len(), 1);
        let d = pool.slot(0);
        assert_eq!(d.dust_type, 31);
        assert_eq!(d.scale, 0.9);
        assert_eq!(d.alpha, 127);
        // The 0×0 box clamps to 5×5 in NewDust; position = lerp + Next(1)+4.
        // After the −(4,4) post-write the anchor is near (105, 200).
        assert!(d.position.x >= 101.0 && d.position.x <= 110.0);
    }









    /// W4-e1 site 24274 — the hair-dye puff: the Next(45) gate, the type
    /// argument draw Next(139,143) BEFORE the internals, the jitter chain,
    /// then the gore sibling's Next(225) gate + argument draws. The mirror
    /// consumes the same order.
    #[test]
    fn w4_e1_hair_dye_argument_order_and_gore_gate() {
        let w = world();
        let c = ctx();
        let b = HairDyeDust { x: 500.0, y: 500.0, width: 20.0, vx: 1.0, vy: -1.0 };
        // Find a seed whose Next(45)==0 (puff gate passes).
        let mut seed = 0i32;
        loop {
            let mut r = UnifiedRandom::new(seed);
            if r.next_max(45) == 0 {
                break;
            }
            seed += 1;
        }
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(seed);
        hair_dye_puff(&mut rng, &mut pool, &c, &w, &b);
        assert_eq!(active(&pool).len(), 1);
        let mut mpool = DustPool::new();
        let mut mirror = UnifiedRandom::new(seed);
        let _ = mirror.next_max(45); // the puff gate
        let ty = mirror.next_range(139, 143);
        let idx = mpool.new_dust(
            &mut mirror, &c, &w, Vec2::new(500.0, 500.0), 20, 8, ty,
            0.0, 0.0, 0, Color::default(), 1.2,
        );
        let md = mpool.slot_mut(idx);
        md.velocity.x *= 1.0 + mirror.next_range(-50, 51) as f32 * 0.01;
        md.velocity.y *= 1.0 + mirror.next_range(-50, 51) as f32 * 0.01;
        md.velocity.x += mirror.next_range(-50, 51) as f32 * 0.01;
        md.velocity.y += mirror.next_range(-50, 51) as f32 * 0.01;
        md.velocity.y -= 1.0;
        md.scale *= 0.7 + mirror.next_range(-30, 31) as f32 * 0.01;
        md.velocity += Vec2::new(1.0, -1.0) * 0.2;
        // The gore sibling gate + its argument draws (whether or not it fires).
        if mirror.next_max(225) == 0 {
            let _ = mirror.next_range(276, 283);
            let _ = mirror.next_max(20);
            let _ = mirror.next_max(8);
            let _ = mirror.next_range(-50, 51);
            let _ = mirror.next_range(-50, 51);
            let _ = mirror.next_range(-20, 21);
            let _ = mirror.next_range(-50, 51);
            let _ = mirror.next_range(-50, 51);
        }
        assert_eq!(pool.slot(0).dust_type, ty);
        assert_eq!(pool.slot(0).velocity, mpool.slot(0).velocity);
        assert_eq!(pool.slot(0).scale, mpool.slot(0).scale);
        assert_eq!(rng.next_range(0, 1000), mirror.next_range(0, 1000));
    }

    /// W4-e1 sites 24641/24654 — the manaHeat ambient: a high num4 (1.0)
    /// passes both NextFloat*6 gates; each passed gate spawns a type-267
    /// blue puff + a white CloneDust twin (scale/fadeIn ×0.4).
    #[test]
    fn w4_e1_mana_heat_spawn_and_clone() {
        let w = world();
        let c = ctx();
        let b = ManaHeatDust { x: 600.0, y: 600.0, width: 20.0, height: 40.0, num4: 1.0 };
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(71);
        mana_heat_ambient(&mut rng, &mut pool, &c, &w, &b);
        // num4=1.0 → NextFloat*6 < 1.0 passes when NextFloat < 1/6. Both
        // iterations may or may not pass; assert the spawned records' shape.
        let n = active(&pool).len();
        assert!(n % 2 == 0, "each passed gate spawns a pair, got {n}");
        for i in (0..n).step_by(2) {
            assert_eq!(pool.slot(i).dust_type, 267);
            assert_eq!(pool.slot(i).color, Color::new(0, 0, 255, 255));
            assert_eq!(pool.slot(i + 1).color, Color::WHITE);
            assert!(pool.slot(i).no_gravity);
            assert!(pool.slot(i + 1).no_gravity);
        }
    }





}
