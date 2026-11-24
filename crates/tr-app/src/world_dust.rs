//! W4-d54 — WorldGen editing-lane dust SUPPLEMENT WITNESSES.
//! The bodies (cutter/unlock-door/paint/coat/ash-shake) moved to their
//! canonical tr-core home `tr_core::worldgen_dust` (g14's port,
//! research/66 union exec): d54's `ash_shake` DROPPED the gore
//! sibling's ARGUMENT draws (`RandomVector2(-40,40)` + `61+Next(3)`,
//! `WorldGen.cs:29669`), breaking the !hitTree stream by 3 draws/row —
//! g14 consumes them (the g17 argument/body hand-off) and its census
//! witnesses are the authority. The LIVE cutter seams
//! (`editing/wiring.rs`) call `kill_actuator_dust`/`kill_wire_dust`
//! directly; the latent bodies' seams arrive with their lanes.
//!
//! This module keeps d54's seven oracle witnesses as supplements
//! (research/66 §3), re-pointed at the tr-core bodies — the ash
//! census oracle ADAPTED to replay the gore argument draws g14
//! consumes. WRITTEN NOT RUN (code-only).

#[cfg(test)]
mod tests {
    //! Each mirror replays the C# draw census on a probe
    //! `UnifiedRandom` and compares `peek()` against the ported
    //! body's stream, then checks the post-spawn field writes.

    use tr_core::particles::{Color, DustPool, SpawnCtx};
    use tr_core::worldgen_dust::{
        coating_color, kill_actuator_dust, kill_wire_dust, paint_coat_effect_dust,
        paint_effect_dust, tree_grow_fx_ash_shake, unlock_door_dust,
    };
    use tr_core::{UnifiedRandom, World};
    use glam::Vec2;

    /// A permissive whole-world ctx (the d52 `dust_ctx` house pattern):
    /// the pool never culls, so every spawn lands and the draw census
    /// matches the live empty-pool path.
    fn dust_ctx(w: &World) -> SpawnCtx {
//! W4-f6 batch 4 — the small world-file dust sites (13): the chest unlock
//! (`Chest.cs:420`), `Utils.PoofOfSmoke` (`Utils.cs:1089`), the pylon
//! teleport dust (`TeleportPylonsSystem.cs:390`), the void-lens pair
//! (`VoidLensHelper.cs:48`/`:63`), the fancy item-text flash
//! (`PopupText.cs:461`/`:470`/`:479`/`:488`), the lava bubbles
//! (`LiquidRenderer.cs:560`/`:564`), the rain splash (`Rain.cs:122`) and the
//! nebula level-up beam (`MessageBuffer.cs:3697`).
//!
//! STAGED emitters (the e2 Skeletron/Brain precedent): every enclosing
//! machine here is unported or draw-side — `Chest.Unlock`'s dust half is
//! explicitly out of `chest.rs::unlock_chest` ("no spawn context on this
//! route"), `Utils.PoofOfSmoke` is the standing gore/dust gap, the pylon
//! `SpawnInWorldDust` / void-lens `SpawnVoidLensDust` are the pylons.rs
//! documented skips, `PopupText` has no port (the combat-text lane is the
//! damage numbers, not the item popups), the `LiquidRenderer` lava bubbles
//! are draw-side, the rain splash is the rain.rs stand-in's gap, and the
//! nebula level-up is an MP-only net message. Each fn takes the snapshot at
//! the C# read moment; the wiring wave calls them on the cited pass.
//!
//! RNG passes (one slot per C# pass, S1-A-10): the chest unlock + the
//! poof + the nebula beam ride the bare `Main.rand` (the `UpdateInventory`
//! / net-dispatch paths run outside every `SwapRandom`, the clicks.rs /
//! netglue.rs `NamedRngs::global()` convention); the pylon dust rides its
//! caller's pass (`Main.TeleportEffect` = the g11/g12 lane's,
//! `Player.UpdateTeleportVisuals` = `UpdatePlayers`); the void-lens rides
//! `UpdateProjectiles` (`Projectile.cs:52195`); the fancy flash rides
//! `UpdateFloatingText` (`Main.cs:17791`); the lava bubbles ride the
//! renderer's PRIVATE `_random` (`LiquidRenderer.cs:132`, a distinct
//! `UnifiedRandom` — documented, NOT a swap key); the rain splash rides the
//! draw's bare `Main.rand` (`Main.DrawItems` :38821, no swap).
//!
//! Gore siblings: `Utils.PoofOfSmoke`'s 3-7 gores (`Utils.cs:1083`) are
//! W4-d55's lane — their ARGUMENT draws (the `NextFloat` angle + the
//! `Next(11, 14)` type per gore) are consumed here in order, the `NewGore`
//! bodies are d55's (the e2 spin hand-off split).
//!
//! WRITTEN NOT RUN (code-only mode): the witnesses below are compiled by
//! the merge-verify wave, not executed here.

use crate::bow_launch::rotated_by;
use glam::Vec2;
use tr_core::particles::{Color, DustPool, SpawnCtx};
use tr_core::{UnifiedRandom, World};

/// `Dust.dustWater()` (`Dust.cs:402-418`): the water-style → dust-type map.
pub fn dust_water(water_style: i32) -> i32 {
    match water_style {
        2 => 98,
        3 => 99,
        4 => 100,
        5 => 101,
        6 => 102,
        7 => 103,
        8 => 104,
        9 => 105,
        10 => 123,
        12 => 288,
        _ => 33,
    }
}

/// Site 420 — the chest unlock's 4-per-tile dust (`Chest.cs:408-424`):
/// after the `PlaySound(22)`, every 2×2 tile whose type matches the
/// origin's (`type2`) shifts its frames and spawns FOUR
/// `NewDust((i*16, j*16), 16, 16, type)` — `type` is 11 (every unlock arm
/// sets it, `:364`/:369/:381/:388/:402). The frame shift is the caller's
/// (`chest.rs::shift_chest_frames`); this fn drains the dust for the tiles
/// that matched. `matched` is the per-tile match flags in the C#'s
/// `i = X..X+1, j = Y..Y+1` order (the caller reads them BEFORE the shift).
/// ZERO explicit rand — only the pool internals roll.
pub fn chest_unlock_dust(
    origin: (i32, i32),
    matched: [bool; 4],
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    let (x, y) = origin;
    let mut idx = 0;
    for i in x..=x + 1 {
        for j in y..=y + 1 {
            if matched[idx] {
                // `:418-421` — four type-11 dusts per matched tile.
                for _ in 0..4 {
                    dust.new_dust(
                        rng, ctx, world, Vec2::new((i * 16) as f32, (j * 16) as f32),
                        16, 16, 11, 0.0, 0.0, 0, Color::default(), 1.0,
                    );
                }
            }
            idx += 1;
        }
    }
}

/// Site 1089 — `Utils.PoofOfSmoke`'s 10-dust tail (`Utils.cs:1078-1093`):
/// the `Next(3, 7)` gore count + the per-gore `NextFloat` angle +
/// `Next(11, 14)` type ARGUMENT draws run FIRST (the `NewGore` bodies are
/// d55's), then 10 `NewDust(position, 14, 14, 16, 0, 0, 100, default,
/// 1.5)`; each `position += (5, 5)`, `velocity =
/// (NextFloat()*2π).ToRotationVector2() * (2, 0.7) * 0.7 * (0.5 +
/// 0.5*NextFloat())`. The gore velocity's `*0.7` and the dust's are the
/// same shape; the gore `scale = 0.7` / `velocity *= 0.5` writes are d55's.
pub fn poof_of_smoke(
    position: Vec2,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    // `:1080-1086` — the gore loop's ARGUMENT draws (d55 bodies).
    let gore_count = rng.next_range(3, 7);
    for _ in 0..gore_count {
        let _angle = rng.next_float() * (std::f32::consts::PI * 2.0); // `:1083`
        let _gore_type = rng.next_range(11, 14); // `:1083`
    }
    // `:1087-1092` — the 10-dust loop (site 1089).
    for _ in 0..10 {
        let idx = dust.new_dust(
            rng, ctx, world, position, 14, 14, 16, 0.0, 0.0, 100, Color::default(), 1.5,
        );
        let d = dust.slot_mut(idx);
        d.position += Vec2::new(5.0, 5.0); // `:1090`
        // `:1091` — the angle NextFloat, then the scale NextFloat.
        let angle = rng.next_float() * (std::f32::consts::PI * 2.0);
        let unit = Vec2::new(
            (angle as f64).cos() as f32,
            (angle as f64).sin() as f32,
        );
        d.velocity = unit * Vec2::new(2.0, 0.7) * 0.7 * (0.5 + 0.5 * rng.next_float());
    }
}

/// Site 390 — the pylon teleport dust (`TeleportPylonsSystem.cs:327-393`):
/// the per-style colour switch (the `TeleportPylonType` byte enum), then
/// ONE `NewDust(dustBox.TopLeft(), dustBox.Width, dustBox.Height, 43, 0, 0,
/// 254, Color(r, g, b, 1), 0.5)`; `velocity *= 0.1`, `velocity.Y -= 0.2`.
/// The float colour is XNA's `(byte)(v*255)` per channel (the Snow/Mushroom
/// 1.2 blue clamps at 255). `tile_style` is the pylon style byte.
pub fn pylon_spawn_dust(
    tile_style: i32,
    box_top_left: Vec2,
    box_w: i32,
    box_h: i32,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    // `:329-389` — the colour switch (default white).
    let (r, g, b) = match tile_style {
        0 => (0.05, 0.8, 0.3),   // SurfacePurity
        1 => (0.7, 0.8, 0.05),   // Jungle
        2 => (0.5, 0.3, 0.7),    // Hallow
        3 => (0.4, 0.4, 0.6),    // Underground
        4 => (0.2, 0.2, 0.95),   // Beach
        5 => (0.85, 0.45, 0.1),  // Desert
        6 => (1.0, 1.0, 1.2),    // Snow
        7 => (0.4, 0.7, 1.2),    // GlowingMushroom
        8 => (0.7, 0.7, 0.7),    // Victory
        9 => (0.05, 0.8, 0.3),   // Underworld
        10 => (0.05, 0.8, 0.3),  // Shimmer
        _ => (1.0, 1.0, 1.0),
    };
    // `new Color(r, g, b, 1f)` — (byte)(v*255) per channel, the 1.2 clamps.
    let color = Color::new(
        (r * 255.0).min(255.0) as u8,
        (g * 255.0).min(255.0) as u8,
        (b * 255.0).min(255.0) as u8,
        255,
    );
    let idx = dust.new_dust(
        rng, ctx, world, box_top_left, box_w, box_h, 43, 0.0, 0.0, 254, color, 0.5,
    );
    let d = dust.slot_mut(idx);
    d.velocity *= 0.1; // `:391`
    d.velocity.y -= 0.2; // `:392`
}

/// Sites 48/63 — the void-lens dust pair (`VoidLensHelper.cs:40-75`): a
/// `Next(3) == 0` gate EVERY tick; on a 0, a `Next(2) == 0` picks arm A
/// (site 48) else arm B (site 63). Both: `vector = UnitY.RotatedByRandom(2π)`
/// (TWO NextDouble), `vector *= (0.5, 1)`, then
/// `NewDustDirect(position - vector*30, 0, 0, SelectRandom(86, 88), ...)`
/// (the `Next(2)` type pick is the ARGUMENT draw); `noGravity`,
/// `noLightEmittance`, `position = position - SafeNormalize(vector) *
/// Next(10, 21)` (arm A) or `Next(5, 10)` (arm B), `velocity =
/// vector.RotatedBy(±π/2) * 2` (A: +π/2 ×2, B: -π/2 ×3), `scale = 0.5 +
/// NextFloat()`, `fadeIn = 0.5`, `position += velocity*10`,
/// `velocity *= -1`. The `customData = this` write is the dropped field.
/// Returns whether the gate fired.
pub fn void_lens_dust(
    position: Vec2,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) -> bool {
    if rng.next_max(3) != 0 {
        return false; // `:42` — the gate draw still ran.
    }
    let arm_a = rng.next_max(2) == 0; // `:44`
    // `:46`/`:61` — RotatedByRandom(2π): TWO NextDouble, the double angle.
    let max = 6.2831854820251465f64;
    let angle = rng.next_double() * max - rng.next_double() * max;
    let mut vector = rotated_by(Vec2::new(0.0, 1.0), angle);
    vector *= Vec2::new(0.5, 1.0); // `:47`/`:62`
    // `:48`/`:63` — the SelectRandom(86, 88) type ARGUMENT draw.
    let ty = rng.next_from_slice(&[86, 88]);
    let idx = dust.new_dust_direct(
        rng, ctx, world, position - vector * 30.0, 0, 0, ty, 0.0, 0.0, 0,
        Color::default(), 1.0,
    );
    let d = dust.slot_mut(idx);
    d.no_gravity = true; // `:49`/`:64`
    d.no_light_emittance = true; // `:50`/`:65`
    // `:51`/`:66` — the SafeNormalize offset, the per-arm Next range.
    let norm = if vector == Vec2::ZERO || vector.x.is_nan() || vector.y.is_nan() {
        Vec2::ZERO
    } else {
        vector.normalize()
    };
    let dist = if arm_a {
        rng.next_range(10, 21) as f32 // `:51`
    } else {
        rng.next_range(5, 10) as f32 // `:66`
    };
    d.position = position - norm * dist;
    // `:52`/`:67` — the rotated velocity (A: +π/2 ×2, B: -π/2 ×3).
    d.velocity = if arm_a {
        rotated_by(vector, 1.5707963705062866) * 2.0
    } else {
        rotated_by(vector, -1.5707963705062866) * 3.0
    };
    d.scale = 0.5 + rng.next_float(); // `:53`/`:68`
    d.fade_in = 0.5; // `:54`/`:69`
    // `:55`/`:70` — customData (dropped).
    d.position += d.velocity * 10.0; // `:56`/`:71`
    d.velocity *= -1.0; // `:57`/`:72`
    true
}

/// Sites 461/470/479/488 — the fancy item-text flash
/// (`PopupText.cs:442-494`, `EmitFancyFlashDust`): `textHitbox =
/// GetTextHitbox() / scale`; `num = 6 + effectIntensity/2`,
/// `num2 = max(0, (-3 + effectIntensity)*4)`, `num -= num2`; the Potion
/// style zeroes both. Loop 1 (sites 461/470): `num` dusts, each a
/// `NextFloat()` position factor `num3 = -0.1 + 1.2*NextFloat()`, then
/// `NewDustPerfect((position.X + textHitbox.X*num3, position.Y +
/// textHitbox.Y*(1 + 0.4*sin(num3*π))), 306, (0, NextFloatDirection()), 0,
/// color, 2)`; `noGravity`, `noLight`, `noLightEmittance`,
/// `velocity.Y += -2`, `fadeIn = 1.4*(1 + 0.4*NextFloat())`,
/// `scale = 0.6 + 0.4*NextFloat()`; a `scale >= 0.9` clones (site 470) with
/// `color = White`, `scale *= 0.65`, `fadeIn = 1.1`. Loop 2 (sites 479/488):
/// `num2` dusts, the same shape but the Y factor is `0.6 + 0.4*sin`, the
/// velocity X is `RotatedBy(2π*NextFloatDirection()).X`, `fadeIn =
/// 2.4*(1 + 0.4*NextFloat())`, and the clone is site 488. `text_hitbox` is
/// the caller's `GetTextHitbox() / scale` (the font measure is the caller's).
#[allow(clippy::too_many_arguments)]
pub fn fancy_flash_dust(
    position: Vec2,
    text_hitbox: Vec2,
    effect_intensity: i32,
    is_potion: bool,
    color: Color,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    // `:446-457` — the counts.
    let mut num = 6 + effect_intensity / 2;
    let mut num2 = (-3 + effect_intensity) * 4;
    if num2 < 0 {
        num2 = 0;
    }
    num -= num2;
    if is_potion {
        num = 0;
        num2 = 0;
    }
    // `:458-475` — loop 1 (sites 461/470).
    for _ in 0..num {
        let num3 = -0.1 + 1.2 * rng.next_float(); // `:460`
        let pos = Vec2::new(
            position.x + text_hitbox.x * num3,
            position.y + text_hitbox.y * (1.0 + 0.4 * (num3 * std::f32::consts::PI).sin()),
        );
        let vel = Vec2::new(0.0, rng.next_float_direction()); // `:461` argument
        let idx = dust.new_dust_perfect(
            rng, ctx, world, pos, 306, Some(vel), 0, color, 2.0,
        );
        let d = dust.slot_mut(idx);
        d.no_gravity = true; // `:462`
        d.no_light = true; // `:463`
        d.no_light_emittance = true; // `:464`
        d.velocity.y += -2.0; // `:465`
        d.fade_in = 1.4 * (1.0 + 0.4 * rng.next_float()); // `:466`
        d.scale = 0.6 + 0.4 * rng.next_float(); // `:467`
        if d.scale >= 0.9 {
            // `:468-474` — the clone (site 470).
            let cidx = dust.clone_dust(rng, ctx, world, idx);
            let cd = dust.slot_mut(cidx);
            cd.color = Color::WHITE;
            cd.scale *= 0.65;
            cd.fade_in = 1.1;
        }
    }
    // `:476-493` — loop 2 (sites 479/488).
    for _ in 0..num2 {
        let num4 = -0.1 + 1.2 * rng.next_float(); // `:478`
        let pos = Vec2::new(
            position.x + text_hitbox.x * num4,
            position.y + text_hitbox.y * (0.6 + 0.4 * (num4 * std::f32::consts::PI).sin()),
        );
        let vel = Vec2::new(0.0, rng.next_float_direction()); // `:479` argument
        let idx = dust.new_dust_perfect(
            rng, ctx, world, pos, 306, Some(vel), 0, color, 2.0,
        );
        let d = dust.slot_mut(idx);
        d.no_light = true; // `:480`
        d.no_light_emittance = true; // `:481`
        // `:482` — velocity.X = RotatedBy(2π*NextFloatDirection()).X.
        let spin = rng.next_float_direction() * (std::f32::consts::PI * 2.0);
        d.velocity.x = rotated_by(d.velocity, spin as f64).x;
        d.velocity.y += -2.0; // `:483`
        d.fade_in = 2.4 * (1.0 + 0.4 * rng.next_float()); // `:484`
        d.scale = 0.6 + 0.4 * rng.next_float(); // `:485`
        if d.scale >= 0.9 {
            // `:486-492` — the clone (site 488).
            let cidx = dust.clone_dust(rng, ctx, world, idx);
            let cd = dust.slot_mut(cidx);
            cd.color = Color::WHITE;
            cd.scale *= 0.65;
            cd.fade_in = 1.1;
        }
    }
}

/// Sites 560/564 — the lava bubbles (`LiquidRenderer.cs:552-574`): per
/// visible lava tile, while `Dust.lavaBubbles < 200`, a `Next(700) == 0`
/// gate spawns ONE `NewDust((i*16, j*16), 16, 16, 35, 0, 0, 0, White)`
/// (site 560); a `Next(350) == 0` gate spawns ONE
/// `NewDust((i*16, j*16), 16, 8, 35, 0, 0, 50, White, 1.5)` (site 564) with
/// `velocity *= 0.8`, `velocity.X *= 2`, `velocity.Y -= Next(1, 7)*0.1`, a
/// `Next(10) == 0` ? `velocity.Y *= Next(2, 5)`, `noGravity`. BOTH gates
/// draw every tile (the `if`s are independent). The `_random` stream is the
/// renderer's PRIVATE `UnifiedRandom` (`:132`), NOT a swap key — the caller
/// passes it. `lava_bubbles` is the live `Dust.lavaBubbles` count (the
/// pool's `lava_bubbles`, reset per `UpdateDust` pass); the fn reads it but
/// the type-35 increment is the `UpdateDust` pass's (`Dust.cs:475`).
/// Returns the number of dusts spawned (0, 1 or 2).
pub fn lava_bubbles(
    tile: (i32, i32),
    lava_bubbles: i32,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) -> i32 {
    if lava_bubbles >= 200 {
        return 0; // `:556` — the counter gate, no draws.
    }
    let (i, j) = tile;
    let pos = Vec2::new((i * 16) as f32, (j * 16) as f32);
    let mut spawned = 0;
    // `:558-561` — the small bubble (site 560).
    if rng.next_max(700) == 0 {
        dust.new_dust(
            rng, ctx, world, pos, 16, 16, 35, 0.0, 0.0, 0, Color::WHITE, 1.0,
        );
        spawned += 1;
    }
    // `:562-573` — the big bubble (site 564).
    if rng.next_max(350) == 0 {
        let idx = dust.new_dust(
            rng, ctx, world, pos, 16, 8, 35, 0.0, 0.0, 50, Color::WHITE, 1.5,
        );
        let d = dust.slot_mut(idx);
        d.velocity *= 0.8; // `:565`
        d.velocity.x *= 2.0; // `:566`
        d.velocity.y -= rng.next_range(1, 7) as f32 * 0.1; // `:567`
        if rng.next_max(10) == 0 {
            d.velocity.y *= rng.next_range(2, 5) as f32; // `:568-571`
        }
        d.no_gravity = true; // `:572`
        spawned += 1;
    }
    spawned
}

/// Site 122 — the rain splash (`Rain.cs:117-131`): on the drop's death
/// (solid/below-screen/wet), a `Next(100) < gfxQuality*100` gate spawns ONE
/// `NewDust(position - velocity, 2, 2, dustWater())`; `position.X -= 2`,
/// `position.Y += 2`, `alpha = 38`, `velocity *= 0.1`,
/// `velocity += -velocity*0.025`, `velocity.Y -= 2`, `scale = 0.6`,
/// `noGravity`. `position`/`velocity` are the drop's POST-move state
/// (`:102` `position += velocity` ran first); `water_style` feeds
/// `dustWater()`. Returns whether the gate fired.
pub fn rain_splash(
    position: Vec2,
    velocity: Vec2,
    water_style: i32,
    gfx_quality: f32,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) -> bool {
    if !(rng.next_max(100) as f32 < gfx_quality * 100.0) {
        return false; // `:120` — the gate draw still ran.
    }
    let idx = dust.new_dust(
        rng, ctx, world, position - velocity, 2, 2, dust_water(water_style),
        0.0, 0.0, 0, Color::default(), 1.0,
    );
    let d = dust.slot_mut(idx);
    d.position.x -= 2.0; // `:123`
    d.position.y += 2.0; // `:124`
    d.alpha = 38; // `:125`
    d.velocity *= 0.1; // `:126`
    d.velocity += -velocity * 0.025; // `:127`
    d.velocity.y -= 2.0; // `:128`
    d.scale = 0.6; // `:129`
    d.no_gravity = true; // `:130`
    true
}

/// Site 3697 — the nebula level-up beam (`MessageBuffer.cs:3652-3706`,
/// case 102): per nearby teammate (`Distance(other) < 700`), the
/// `player11.Center - player12.Center` direction (normalized, the NaN guard
/// skips), the buff-type → dust-type pick (179→86, 173→90, 176→88, else
/// 90), then `(int)(length/6)` dusts along the beam: each
/// `NewDust(player12.Center + 6*i*dir + (0, -8).RotatedBy(angle) + (-3, -3),
/// 6, 6, type, 0, 0, 100, default, 1.5)`; `noGravity`, `velocity = Zero`,
/// `fadeIn = num110 + 0.005` (the accumulator climbs per dust),
/// `velocity += dir*1.5`. The angle climbs π/15 per dust. `source_center`
/// is player11's, `target_center` player12's, `buff_type` the level-up id.
/// ZERO rand outside the pool internals (the beam is deterministic).
pub fn nebula_levelup_beam(
    source_center: Vec2,
    target_center: Vec2,
    buff_type: i32,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    let value2 = source_center - target_center; // `:3670`
    let dir = value2.normalize_or_zero(); // `:3671`
    if dir.x.is_nan() || dir.y.is_nan() {
        return; // `:3672` — the HasNaNs guard.
    }
    // `:3674-3692` — the type pick.
    let ty = match buff_type {
        179 => 86,
        173 => 90,
        176 => 88,
        _ => 90,
    };
    let mut angle = 0.0f32; // `:3675` num108
    let angle_step = std::f32::consts::PI / 15.0; // `:3676` num109
    let spinning = Vec2::new(0.0, -8.0); // `:3677`
    let offset = Vec2::new(-3.0, -3.0); // `:3678` vector5
    let mut fade = 0.0f32; // `:3679` num110
    let count = (value2.length() / 6.0) as i32; // `:3693` the loop bound
    for i in 0..count {
        // `:3695` — the beam position, the rotated spinning point.
        let pos = target_center + dir * (6.0 * i as f32)
            + rotated_by(spinning, angle as f64)
            + offset;
        angle += angle_step; // `:3696`
        let idx = dust.new_dust(
            rng, ctx, world, pos, 6, 6, ty, 0.0, 0.0, 100, Color::default(), 1.5,
        );
        let d = dust.slot_mut(idx);
        d.no_gravity = true; // `:3698`
        d.velocity = Vec2::ZERO; // `:3699`
        fade += 0.005; // `:3700` num110 += num111
        d.fade_in = fade;
        d.velocity += dir * 1.5; // `:3701`
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn world() -> World {
        World::new("f6-world".into(), 200, 200)
    }

    fn ctx() -> SpawnCtx {
        SpawnCtx {
            game_menu: false,
            game_paused: false,
            generating: false,
            screen_position: Vec2::ZERO,
            screen_size: Vec2::new(w.width as f32 * 16.0, w.height as f32 * 16.0),
            screen_size: Vec2::new(3200.0, 3200.0),
            max_dust_to_draw: 6000,
        }
    }

    /// One `NewDust` default block on an empty pool under the 50 % band
    /// (`Dust.cs:153-193`): the two position `Next(box-4)` rolls (the box
    /// clamps to >= 5), the velocity `Next(-20,21)` pair, the frame
    /// `Next(3)`, the scale `Next(-20,21)`; then the per-type tweak — the
    /// 6/228 family adds one `Next(-10,6)` (`Dust.cs:214-220`); 50/11/
    /// 143/31 add nothing. ChildSafety is disabled (the pool default).
    fn block(r: &mut UnifiedRandom, dust_type: i32, box_w: i32, box_h: i32) {
        r.next_max(box_w.max(5) - 4);
        r.next_max(box_h.max(5) - 4);
        r.next_range(-20, 21);
        r.next_range(-20, 21);
        r.next_max(3);
        r.next_range(-20, 21);
        if matches!(dust_type, 6 | 228) {
            r.next_range(-10, 6);
        }
    }

    /// `Utils.RandomVector2` — two `NextDouble` draws.
    fn rv2(r: &mut UnifiedRandom) {
        r.next_double();
        r.next_double();
    }

    /// `Utils.NextVector2Circular` — the unit `NextFloat` + the radius
    /// `NextFloat`.
    fn circular(r: &mut UnifiedRandom) {
        r.next_float();
        r.next_float();
    }

    fn drain(dust: &mut DustPool) {
        for s in dust.slots_mut() {
            s.active = false;
        }
    }

    /// `WorldGen.cs:58032-58035` — the cutter burst is exactly five
    /// type-50 default blocks, no post draws; the actuator and wire
    /// entry points share the body (the wiring.rs seams).
    #[test]
    fn w4_d54_cutter_burst_draws_match_the_oracle() {
        let w = World::new("t".into(), 200, 200);
        let ctx = dust_ctx(&w);
        let mut rng = UnifiedRandom::new(54);
        let mut probe = UnifiedRandom::new(54);
        let mut dust = DustPool::new();
        for i in 0..200 {
            if i % 2 == 0 {
                kill_actuator_dust(10, 20, &mut rng, &mut dust, &ctx, &w);
            } else {
                kill_wire_dust(10, 20, &mut rng, &mut dust, &ctx, &w);
            }
            for _ in 0..5 {
                block(&mut probe, 50, 16, 16);
            }
            assert_eq!(rng.peek(), probe.peek(), "cutter stream @{i}");
            drain(&mut dust);
        }
        assert_ne!(rng.peek(), UnifiedRandom::new(54).peek(), "the burst drew");
    }

    /// `WorldGen.cs:37615-37626` — the unlock burst is 3 rows × 4
    /// type-11 blocks; the puffs land at the row's tile corner + the
    /// default block's `Next(12)+4` offsets.
    #[test]
    fn w4_d54_unlock_door_burst_draws_match_the_oracle() {
        let w = World::new("t".into(), 200, 200);
        let ctx = dust_ctx(&w);
        let mut rng = UnifiedRandom::new(54);
        let mut probe = UnifiedRandom::new(54);
        let mut dust = DustPool::new();
        for i in 0..100 {
            unlock_door_dust(12, 30, &mut rng, &mut dust, &ctx, &w);
            for _ in 0..12 {
                block(&mut probe, 11, 16, 16);
            }
            assert_eq!(rng.peek(), probe.peek(), "unlock stream @{i}");
            drain(&mut dust);
        }
        // The first puff's position: (12*16 + Next(12)+4, 30*16 + Next(12)+4).
        let mut replay = UnifiedRandom::new(54);
        let px = 192.0 + replay.next_max(12) as f32 + 4.0;
        let py = 480.0 + replay.next_max(12) as f32 + 4.0;
        let mut dust2 = DustPool::new();
        let mut rng2 = UnifiedRandom::new(54);
        unlock_door_dust(12, 30, &mut rng2, &mut dust2, &ctx, &w);
        let d = dust2.slot(0);
        assert!(d.active);
        assert_eq!(d.dust_type, 11);
        assert_eq!((d.position.x, d.position.y), (px, py));
        // Row 2's first puff (index 8) sits two tiles lower: 512 + Next(12)+4.
        let d8 = dust2.slot(8);
        assert!(d8.position.y >= 516.0 && d8.position.y < 528.0);
    }

    /// `WorldGen.cs:44072-44084` — ten type-143 blocks on `Main.rand`,
    /// each followed by ONE `genRand.Next(2)`; the two streams never
    /// interleave. Both post arms fire across the replay.
    #[test]
    fn w4_d54_paint_effect_draws_match_the_oracle() {
        let w = World::new("t".into(), 200, 200);
        let ctx = dust_ctx(&w);
        let mut rng = UnifiedRandom::new(54);
        let mut probe = UnifiedRandom::new(54);
        let mut gen = UnifiedRandom::new(7);
        let mut gen_probe = UnifiedRandom::new(7);
        let mut dust = DustPool::new();
        let mut saw_big = false;
        let mut saw_small = false;
        for i in 0..100 {
            paint_effect_dust(10, 20, 9, 0, &mut rng, &mut gen, &mut dust, &ctx, &w);
            for _ in 0..10 {
                block(&mut probe, 143, 16, 16);
                if gen_probe.next_max(2) == 0 {
                    saw_big = true;
                } else {
                    saw_small = true;
                }
            }
            assert_eq!(rng.peek(), probe.peek(), "paint main stream @{i}");
            assert_eq!(gen.peek(), gen_probe.peek(), "paint gen stream @{i}");
            drain(&mut dust);
        }
        assert!(saw_big && saw_small, "both Next(2) arms exercised");
    }

    /// `WorldGen.cs:44066-44071` + `:43921-44034` — the colour fold
    /// (`color == 0` takes `oldColor`) and the alpha-150 color-30 arm
    /// (the union fix, research/66 §2); the post-arm scale writes land
    /// on the spawned slot.
    #[test]
    fn w4_d54_paint_effect_writes_the_color_and_scale() {
        let w = World::new("t".into(), 200, 200);
        let ctx = dust_ctx(&w);
        let mut rng = UnifiedRandom::new(54);
        let mut gen = UnifiedRandom::new(7);
        let mut dust = DustPool::new();
        paint_effect_dust(10, 20, 0, 30, &mut rng, &mut gen, &mut dust, &ctx, &w);
        // Replay the first spawn: the block's scale roll is the LAST
        // Next(-20,21); re-derive it.
        let mut replay2 = UnifiedRandom::new(54);
        replay2.next_max(12);
        replay2.next_max(12);
        replay2.next_range(-20, 21);
        replay2.next_range(-20, 21);
        replay2.next_max(3);
        let scale = 1.0 + replay2.next_range(-20, 21) as f32 * 0.01;
        let mut gen_replay = UnifiedRandom::new(7);
        let big = gen_replay.next_max(2) == 0;
        let d = dust.slot(0);
        assert_eq!(d.color, Color::new(200, 200, 200, 150));
        assert_eq!(d.alpha, 50);
        assert_eq!(d.no_gravity, big);
        let want = if big { scale * 1.2 } else { scale * 0.5 };
        assert!((d.scale - want).abs() < 1e-6);
        // The non-zero color keeps its own id (no fold): paint 1 is
        // red at alpha 255 (the color-30 alpha arm is the ONLY one).
        let mut dust2 = DustPool::new();
        let mut rng2 = UnifiedRandom::new(54);
        let mut gen2 = UnifiedRandom::new(7);
        paint_effect_dust(10, 20, 1, 30, &mut rng2, &mut gen2, &mut dust2, &ctx, &w);
        assert_eq!(dust2.slot(0).color, Color::new(255, 0, 0, 255));
    }

    /// `WorldGen.cs:44090-44107` — coat 0 with a non-empty old-colour
    /// list draws `Next(count)` BEFORE each spawn; coats 1/2 draw
    /// nothing extra and carry the fixed coating colours.
    #[test]
    fn w4_d54_paint_coat_effect_draws_match_the_oracle() {
        let w = World::new("t".into(), 200, 200);
        let ctx = dust_ctx(&w);
        let olds = [Color::new(1, 2, 3, 255), Color::new(4, 5, 6, 255)];
        for coat in [0u8, 1, 2, 3] {
            let mut rng = UnifiedRandom::new(54);
            let mut probe = UnifiedRandom::new(54);
            let mut gen = UnifiedRandom::new(7);
            let mut gen_probe = UnifiedRandom::new(7);
            let mut dust = DustPool::new();
            for i in 0..50 {
                paint_coat_effect_dust(10, 20, coat, &olds, &mut rng, &mut gen, &mut dust, &ctx, &w);
                for _ in 0..10 {
                    if coat == 0 {
                        probe.next_max(2); // Next(oldColors.Count)
                    }
                    block(&mut probe, 143, 16, 16);
                    gen_probe.next_max(2);
                }
                assert_eq!(rng.peek(), probe.peek(), "coat {coat} main @{i}");
                assert_eq!(gen.peek(), gen_probe.peek(), "coat {coat} gen @{i}");
                drain(&mut dust);
            }
        }
        // The fixed coating colours + the transparent default.
        assert_eq!(coating_color(1), Color::new(235, 170, 255, 255));
        assert_eq!(coating_color(2), Color::new(180, 245, 255, 255));
        assert_eq!(coating_color(0), Color::TRANSPARENT);
        assert_eq!(coating_color(9), Color::TRANSPARENT);
    }

    /// `WorldGen.cs:29642-29671` — the ash-shake's full draw census:
    /// per row the two short-circuit `Next(3)` gates, then per puff the
    /// position `RandomVector2` (2), the type-6 scale argument (1, before
    /// the block), the 4×4→5-clamped default block (+ the type-6
    /// `Next(-10,6)` tweak), the velocity `Circular` (2) and the ember's
    /// `Next(2)` gate; then the !hitTree gore sibling's ARGUMENT draws
    /// (`RandomVector2(-40,40)` = 2 NextDouble + `Next(3)`, `:29669` —
    /// ADAPTED per research/66: g14 consumes them, d54's original
    /// oracle lacked them). Both `hitTree` arms replayed from several
    /// seeds.
    #[test]
    fn w4_d54_ash_shake_draws_match_the_oracle() {
        let w = World::new("t".into(), 200, 200);
        let ctx = dust_ctx(&w);
        for hit_tree in [false, true] {
            for seed in [54, 1, 99] {
                let height = 20;
                let mut rng = UnifiedRandom::new(seed);
                let mut probe = UnifiedRandom::new(seed);
                let mut dust = DustPool::new();
                tree_grow_fx_ash_shake(100, 100, height, hit_tree, &mut rng, &mut dust, &ctx, &w);
                let mut num = 0i32;
                while num > -height {
                    if num >= -height / 2 && probe.next_max(3) != 0 {
                        for _ in 0..if hit_tree { 10 } else { 5 } {
                            rv2(&mut probe);
                            block(&mut probe, 31, 4, 4);
                            circular(&mut probe);
                        }
                    }
                    if num as f32 >= (-height) as f32 * 0.7 && probe.next_max(3) != 0 {
                        for _ in 0..if hit_tree { 16 } else { 10 } {
                            rv2(&mut probe);
                            probe.next_float(); // the scale argument
                            block(&mut probe, 6, 4, 4);
                            circular(&mut probe);
                            probe.next_max(2);
                        }
                    }
                    if !hit_tree {
                        // `:29669` — the gore argument draws (g17's body).
                        rv2(&mut probe);
                        probe.next_max(3);
                    }
                    num -= 1;
                }
                assert_eq!(
                    rng.peek(),
                    probe.peek(),
                    "ash shake hit={hit_tree} seed={seed}"
                );
                drain(&mut dust);
            }
        }
    }

    /// `WorldGen.cs:29657-29664` — the ember's post writes: velocity is
    /// the Circular + (0,-0.6) overwrite (the spawn's own velocity is
    /// dead), the `Next(2)` pass doubles it and lifts gravity, then the
    /// constant `(0, hitTree ? -1 : -2)` lift always lands. The gore
    /// argument draws run AFTER both loops — the first ember's replay
    /// is unaffected.
    #[test]
    fn w4_d54_ash_shake_writes_the_ember_fields() {
        let w = World::new("t".into(), 200, 200);
        let ctx = dust_ctx(&w);
        let mut rng = UnifiedRandom::new(54);
        let mut dust = DustPool::new();
        // height 1: row 0 passes BOTH position gates; find a seed whose
        // first gate passes so the first spawn is a type-31 puff and the
        // ember lane starts at index 5 (hitTree=false).
        tree_grow_fx_ash_shake(100, 100, 1, false, &mut rng, &mut dust, &ctx, &w);
        let mut probe = UnifiedRandom::new(54);
        let gate31 = probe.next_max(3) != 0;
        let mut idx = 0usize;
        if gate31 {
            for _ in 0..5 {
                rv2(&mut probe);
                block(&mut probe, 31, 4, 4);
                circular(&mut probe);
                idx += 1;
            }
        }
        let gate6 = probe.next_max(3) != 0;
        if gate6 {
            // First ember: replay its draws to recover the velocity.
            rv2(&mut probe);
            probe.next_float(); // scale arg
            block(&mut probe, 6, 4, 4);
            let angle = std::f32::consts::PI * 2.0 * probe.next_float();
            let unit = Vec2::new((angle as f64).cos() as f32, (angle as f64).sin() as f32);
            let r = probe.next_float();
            let mut vel = Vec2::new(unit.x * 2.0 * r, unit.y * 2.0 * r) + Vec2::new(0.0, -0.6);
            let doubled = probe.next_max(2) == 0;
            if doubled {
                vel *= 2.0;
            }
            vel += Vec2::new(0.0, -2.0);
            let d = dust.slot(idx);
            assert_eq!(d.dust_type, 6);
            assert_eq!(d.no_gravity, doubled);
            assert!((d.velocity.x - vel.x).abs() < 1e-5);
            assert!((d.velocity.y - vel.y).abs() < 1e-5);
        }
    fn active(pool: &DustPool) -> Vec<usize> {
        (0..6000).filter(|&i| pool.slot(i).active).collect()
    }

    /// W4-f6 — site 420: four type-11 dusts per matched tile, zero explicit
    /// draws. A 2-match unlock spawns 8; the mirror lands identical records.
    #[test]
    fn w4_f6_chest_unlock_spawns_four_per_matched_tile() {
        let w = world();
        let c = ctx();
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(71);
        // Two matched tiles (the (0,0) and (1,1) corners).
        chest_unlock_dust((10, 10), [true, false, false, true], &mut rng, &mut pool, &c, &w);
        assert_eq!(active(&pool).len(), 8, "2 matched tiles x 4");
        for i in 0..8 {
            assert_eq!(pool.slot(i).dust_type, 11);
        }
        // The first tile's dusts anchor at (160, 160).
        assert!(pool.slot(0).position.x >= 160.0 && pool.slot(0).position.x < 176.0);
        // The mirror: 8 bare NewDust calls, no explicit draws.
        let mut mpool = DustPool::new();
        let mut mirror = UnifiedRandom::new(71);
        for _ in 0..4 {
            mpool.new_dust(&mut mirror, &c, &w, Vec2::new(160.0, 160.0), 16, 16, 11, 0.0, 0.0, 0, Color::default(), 1.0);
        }
        for _ in 0..4 {
            mpool.new_dust(&mut mirror, &c, &w, Vec2::new(176.0, 176.0), 16, 16, 11, 0.0, 0.0, 0, Color::default(), 1.0);
        }
        for i in 0..8 {
            assert_eq!(pool.slot(i).position, mpool.slot(i).position);
            assert_eq!(pool.slot(i).velocity, mpool.slot(i).velocity);
        }
        assert_eq!(rng.next(), mirror.next());
        // No matched tiles: nothing spawns, no draws.
        let mut npool = DustPool::new();
        let mut nrng = UnifiedRandom::new(71);
        chest_unlock_dust((10, 10), [false; 4], &mut nrng, &mut npool, &c, &w);
        assert!(active(&npool).is_empty());
        let mut probe = UnifiedRandom::new(71);
        assert_eq!(nrng.next(), probe.next());
    }

    /// W4-f6 — site 1089: the gore argument draws (Next(3,7) count + per-gore
    /// NextFloat + Next(11,14)) run before the 10 dusts; each dust draws the
    /// angle NextFloat then the scale NextFloat. The mirror replays the
    /// whole chain.
    #[test]
    fn w4_f6_poof_mirrors_the_gore_then_dust_order() {
        let w = world();
        let c = ctx();
        let pos = Vec2::new(500.0, 500.0);
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(83);
        poof_of_smoke(pos, &mut rng, &mut pool, &c, &w);
        let mut mpool = DustPool::new();
        let mut mirror = UnifiedRandom::new(83);
        let gore_count = mirror.next_range(3, 7);
        for _ in 0..gore_count {
            let _ = mirror.next_float() * (std::f32::consts::PI * 2.0);
            let _ = mirror.next_range(11, 14);
        }
        for _ in 0..10 {
            let idx = mpool.new_dust(&mut mirror, &c, &w, pos, 14, 14, 16, 0.0, 0.0, 100, Color::default(), 1.5);
            let md = mpool.slot_mut(idx);
            md.position += Vec2::new(5.0, 5.0);
            let angle = mirror.next_float() * (std::f32::consts::PI * 2.0);
            let unit = Vec2::new((angle as f64).cos() as f32, (angle as f64).sin() as f32);
            md.velocity = unit * Vec2::new(2.0, 0.7) * 0.7 * (0.5 + 0.5 * mirror.next_float());
        }
        assert_eq!(active(&pool).len(), 10);
        for i in 0..10 {
            assert_eq!(pool.slot(i).position, mpool.slot(i).position);
            assert_eq!(pool.slot(i).velocity, mpool.slot(i).velocity);
            assert_eq!(pool.slot(i).dust_type, 16);
            assert_eq!(pool.slot(i).alpha, 100);
        }
        assert_eq!(rng.next(), mirror.next());
    }

    /// W4-f6 — site 390: the per-style colour, one type-43 dust, the
    /// velocity ×0.1 / Y-0.2 chain. The Snow style's 1.2 blue clamps at 255.
    #[test]
    fn w4_f6_pylon_dust_colours_and_mirrors() {
        let w = world();
        let c = ctx();
        let tl = Vec2::new(480.0, 480.0);
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(97);
        pylon_spawn_dust(6, tl, 30, 52, &mut rng, &mut pool, &c, &w); // Snow
        assert_eq!(active(&pool).len(), 1);
        let d = pool.slot(0);
        assert_eq!(d.dust_type, 43);
        assert_eq!(d.alpha, 254);
        assert_eq!(d.color, Color::new(255, 255, 255, 255)); // 1.2 blue clamps
        // The scale is the pool's default-block roll (1 + Next(-20,21)*0.01)
        // times the 0.5 argument — the mirror below pins the exact value.
        // The mirror: one NewDust + the velocity chain.
        let mut mpool = DustPool::new();
        let mut mirror = UnifiedRandom::new(97);
        let idx = mpool.new_dust(&mut mirror, &c, &w, tl, 30, 52, 43, 0.0, 0.0, 254, Color::new(255, 255, 255, 255), 0.5);
        let md = mpool.slot_mut(idx);
        md.velocity *= 0.1;
        md.velocity.y -= 0.2;
        assert_eq!(pool.slot(0).velocity, mpool.slot(0).velocity);
        assert_eq!(pool.slot(0).scale, mpool.slot(0).scale);
        assert_eq!(rng.next(), mirror.next());
        // The Jungle style's colour differs.
        let mut jpool = DustPool::new();
        let mut jrng = UnifiedRandom::new(97);
        pylon_spawn_dust(1, tl, 30, 52, &mut jrng, &mut jpool, &c, &w);
        assert_eq!(jpool.slot(0).color, Color::new(178, 204, 12, 255));
    }

    /// W4-f6 — sites 48/63: the Next(3) gate, the Next(2) arm pick, the
    /// RotatedByRandom two-NextDouble, the SelectRandom(86,88) type, the
    /// per-arm Next range + rotation. The mirror replays the full chain.
    #[test]
    fn w4_f6_void_lens_mirrors_both_arms() {
        let w = world();
        let c = ctx();
        let pos = Vec2::new(500.0, 500.0);
        // Find a seed whose Next(3) == 0 (the gate fires).
        let mut seed = 0;
        loop {
            let mut p = UnifiedRandom::new(seed);
            if p.next_max(3) == 0 {
                break;
            }
            seed += 1;
        }
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(seed);
        assert!(void_lens_dust(pos, &mut rng, &mut pool, &c, &w));
        let mut mpool = DustPool::new();
        let mut mirror = UnifiedRandom::new(seed);
        let _ = mirror.next_max(3);
        let arm_a = mirror.next_max(2) == 0;
        let max = 6.2831854820251465f64;
        let angle = mirror.next_double() * max - mirror.next_double() * max;
        let mut vector = rotated_by(Vec2::new(0.0, 1.0), angle);
        vector *= Vec2::new(0.5, 1.0);
        let ty = mirror.next_from_slice(&[86, 88]);
        let idx = mpool.new_dust_direct(&mut mirror, &c, &w, pos - vector * 30.0, 0, 0, ty, 0.0, 0.0, 0, Color::default(), 1.0);
        let md = mpool.slot_mut(idx);
        md.no_gravity = true;
        md.no_light_emittance = true;
        let norm = if vector == Vec2::ZERO { Vec2::ZERO } else { vector.normalize() };
        let dist = if arm_a { mirror.next_range(10, 21) as f32 } else { mirror.next_range(5, 10) as f32 };
        md.position = pos - norm * dist;
        md.velocity = if arm_a { rotated_by(vector, 1.5707963705062866) * 2.0 } else { rotated_by(vector, -1.5707963705062866) * 3.0 };
        md.scale = 0.5 + mirror.next_float();
        md.fade_in = 0.5;
        md.position += md.velocity * 10.0;
        md.velocity *= -1.0;
        assert_eq!(active(&pool).len(), 1);
        assert_eq!(pool.slot(0).position, mpool.slot(0).position);
        assert_eq!(pool.slot(0).velocity, mpool.slot(0).velocity);
        assert_eq!(pool.slot(0).scale, mpool.slot(0).scale);
        assert_eq!(pool.slot(0).dust_type, ty);
        assert!(pool.slot(0).no_gravity && pool.slot(0).no_light_emittance);
        assert_eq!(rng.next(), mirror.next());
        // A failed gate spawns nothing, one draw.
        let mut fseed = seed;
        loop {
            let mut p = UnifiedRandom::new(fseed);
            if p.next_max(3) != 0 {
                break;
            }
            fseed += 1;
        }
        let mut fpool = DustPool::new();
        let mut frng = UnifiedRandom::new(fseed);
        assert!(!void_lens_dust(pos, &mut frng, &mut fpool, &c, &w));
        assert!(active(&fpool).is_empty());
        let mut probe = UnifiedRandom::new(fseed);
        let _ = probe.next_max(3);
        assert_eq!(frng.next(), probe.next());
    }

    /// W4-f6 — sites 461/470/479/488: the intensity counts, the Potion
    /// zero, the per-dust NextFloat position + NextFloatDirection velocity
    /// + the clone gate. The mirror replays both loops.
    #[test]
    fn w4_f6_fancy_flash_mirrors_both_loops() {
        let w = world();
        let c = ctx();
        let pos = Vec2::new(500.0, 500.0);
        let hitbox = Vec2::new(40.0, 20.0);
        let color = Color::new(255, 200, 100, 255);
        // intensity 5: num = 6 + 2 = 8, num2 = (2)*4 = 8, num -= 8 -> 0.
        // So loop 1 = 0, loop 2 = 8. Use intensity 3: num = 6+1 = 7,
        // num2 = 0, num -= 0 -> 7. Loop 1 = 7, loop 2 = 0.
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(103);
        fancy_flash_dust(pos, hitbox, 3, false, color, &mut rng, &mut pool, &c, &w);
        let mut mpool = DustPool::new();
        let mut mirror = UnifiedRandom::new(103);
        let mut total = 0;
        for _ in 0..7 {
            let num3 = -0.1 + 1.2 * mirror.next_float();
            let p = Vec2::new(pos.x + hitbox.x * num3, pos.y + hitbox.y * (1.0 + 0.4 * (num3 * std::f32::consts::PI).sin()));
            let vel = Vec2::new(0.0, mirror.next_float_direction());
            let idx = mpool.new_dust_perfect(&mut mirror, &c, &w, p, 306, Some(vel), 0, color, 2.0);
            let md = mpool.slot_mut(idx);
            md.no_gravity = true;
            md.no_light = true;
            md.no_light_emittance = true;
            md.velocity.y += -2.0;
            md.fade_in = 1.4 * (1.0 + 0.4 * mirror.next_float());
            md.scale = 0.6 + 0.4 * mirror.next_float();
            total += 1;
            if md.scale >= 0.9 {
                let cidx = mpool.clone_dust(&mut mirror, &c, &w, idx);
                let cd = mpool.slot_mut(cidx);
                cd.color = Color::WHITE;
                cd.scale *= 0.65;
                cd.fade_in = 1.1;
                total += 1;
            }
        }
        assert_eq!(active(&pool).len(), total);
        assert_eq!(pool.slot(0).dust_type, 306);
        assert_eq!(pool.slot(0).velocity, mpool.slot(0).velocity);
        assert_eq!(pool.slot(0).scale, mpool.slot(0).scale);
        assert_eq!(rng.next(), mirror.next());
        // The Potion style zeroes both loops: nothing spawns, no draws.
        let mut pp = DustPool::new();
        let mut pr = UnifiedRandom::new(103);
        fancy_flash_dust(pos, hitbox, 5, true, color, &mut pr, &mut pp, &c, &w);
        assert!(active(&pp).is_empty());
        let mut probe = UnifiedRandom::new(103);
        assert_eq!(pr.next(), probe.next());
    }

    /// W4-f6 — sites 560/564: both gates draw every tile; the counter gate
    /// (>= 200) skips both. The big bubble's velocity chain mirrors.
    #[test]
    fn w4_f6_lava_bubbles_gates_and_mirror() {
        let w = world();
        let c = ctx();
        // Counter >= 200: no draws.
        let mut full = DustPool::new();
        let mut frng = UnifiedRandom::new(109);
        assert_eq!(lava_bubbles((10, 10), 200, &mut frng, &mut full, &c, &w), 0);
        let mut probe = UnifiedRandom::new(109);
        assert_eq!(frng.next(), probe.next());
        // Find a seed where the Next(350) == 0 (the big bubble fires).
        let mut seed = 0;
        loop {
            let mut p = UnifiedRandom::new(seed);
            let _ = p.next_max(700);
            if p.next_max(350) == 0 {
                break;
            }
            seed += 1;
        }
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(seed);
        let n = lava_bubbles((10, 10), 0, &mut rng, &mut pool, &c, &w);
        assert!(n >= 1);
        let mut mpool = DustPool::new();
        let mut mirror = UnifiedRandom::new(seed);
        let pos = Vec2::new(160.0, 160.0);
        let mut spawned = 0;
        if mirror.next_max(700) == 0 {
            mpool.new_dust(&mut mirror, &c, &w, pos, 16, 16, 35, 0.0, 0.0, 0, Color::WHITE, 1.0);
            spawned += 1;
        }
        if mirror.next_max(350) == 0 {
            let idx = mpool.new_dust(&mut mirror, &c, &w, pos, 16, 8, 35, 0.0, 0.0, 50, Color::WHITE, 1.5);
            let md = mpool.slot_mut(idx);
            md.velocity *= 0.8;
            md.velocity.x *= 2.0;
            md.velocity.y -= mirror.next_range(1, 7) as f32 * 0.1;
            if mirror.next_max(10) == 0 {
                md.velocity.y *= mirror.next_range(2, 5) as f32;
            }
            md.no_gravity = true;
            spawned += 1;
        }
        assert_eq!(n, spawned);
        assert_eq!(active(&pool).len(), spawned);
        // The big bubble (last spawned) mirrors the velocity chain.
        let big = &pool.slot(spawned - 1);
        let mbig = &mpool.slot(spawned - 1);
        assert_eq!(big.velocity, mbig.velocity);
        assert!(big.no_gravity);
        assert_eq!(rng.next(), mirror.next());
    }

    /// W4-f6 — site 122: the Next(100) < gfxQuality*100 gate, the dustWater
    /// type, the position/velocity chain. The mirror replays the spawn.
    #[test]
    fn w4_f6_rain_splash_gates_and_mirrors() {
        let w = world();
        let c = ctx();
        let pos = Vec2::new(500.0, 500.0);
        let vel = Vec2::new(1.0, 8.0);
        // gfxQuality 1.0: Next(100) < 100 always fires.
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(113);
        assert!(rain_splash(pos, vel, 0, 1.0, &mut rng, &mut pool, &c, &w));
        let mut mpool = DustPool::new();
        let mut mirror = UnifiedRandom::new(113);
        let _ = mirror.next_max(100);
        let idx = mpool.new_dust(&mut mirror, &c, &w, pos - vel, 2, 2, 33, 0.0, 0.0, 0, Color::default(), 1.0);
        let md = mpool.slot_mut(idx);
        md.position.x -= 2.0;
        md.position.y += 2.0;
        md.alpha = 38;
        md.velocity *= 0.1;
        md.velocity += -vel * 0.025;
        md.velocity.y -= 2.0;
        md.scale = 0.6;
        md.no_gravity = true;
        assert_eq!(active(&pool).len(), 1);
        assert_eq!(pool.slot(0).position, mpool.slot(0).position);
        assert_eq!(pool.slot(0).velocity, mpool.slot(0).velocity);
        assert_eq!(pool.slot(0).alpha, 38);
        assert_eq!(pool.slot(0).scale, 0.6);
        assert_eq!(pool.slot(0).dust_type, 33); // waterStyle 0 -> 33
        assert!(pool.slot(0).no_gravity);
        assert_eq!(rng.next(), mirror.next());
        // gfxQuality 0: the gate never fires, one draw.
        let mut zp = DustPool::new();
        let mut zr = UnifiedRandom::new(113);
        assert!(!rain_splash(pos, vel, 0, 0.0, &mut zr, &mut zp, &c, &w));
        assert!(active(&zp).is_empty());
        let mut probe = UnifiedRandom::new(113);
        let _ = probe.next_max(100);
        assert_eq!(zr.next(), probe.next());
        // dustWater map: style 10 -> 123, style 12 -> 288.
        assert_eq!(dust_water(10), 123);
        assert_eq!(dust_water(12), 288);
        assert_eq!(dust_water(7), 103);
    }

    /// W4-f6 — site 3697: the deterministic beam — (int)(length/6) dusts,
    /// the climbing fadeIn + angle, the type pick. Zero rand outside the
    /// pool internals; the mirror lands identical records.
    #[test]
    fn w4_f6_nebula_beam_mirrors_the_deterministic_chain() {
        let w = world();
        let c = ctx();
        let source = Vec2::new(500.0, 500.0);
        let target = Vec2::new(560.0, 500.0); // length 60 -> 10 dusts
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(127);
        nebula_levelup_beam(source, target, 179, &mut rng, &mut pool, &c, &w);
        let mut mpool = DustPool::new();
        let mut mirror = UnifiedRandom::new(127);
        let value2 = source - target;
        let dir = value2.normalize_or_zero();
        let mut angle = 0.0f32;
        let mut fade = 0.0f32;
        let count = (value2.length() / 6.0) as i32;
        for i in 0..count {
            let p = target + dir * (6.0 * i as f32) + rotated_by(Vec2::new(0.0, -8.0), angle as f64) + Vec2::new(-3.0, -3.0);
            angle += std::f32::consts::PI / 15.0;
            let idx = mpool.new_dust(&mut mirror, &c, &w, p, 6, 6, 86, 0.0, 0.0, 100, Color::default(), 1.5);
            let md = mpool.slot_mut(idx);
            md.no_gravity = true;
            md.velocity = Vec2::ZERO;
            fade += 0.005;
            md.fade_in = fade;
            md.velocity += dir * 1.5;
        }
        assert_eq!(count, 10);
        assert_eq!(active(&pool).len(), 10);
        for i in 0..10 {
            assert_eq!(pool.slot(i).position, mpool.slot(i).position);
            assert_eq!(pool.slot(i).velocity, mpool.slot(i).velocity);
            assert_eq!(pool.slot(i).fade_in, mpool.slot(i).fade_in);
            assert_eq!(pool.slot(i).dust_type, 86); // buff 179 -> 86
            assert!(pool.slot(i).no_gravity);
        }
        // The fadeIn climbs 0.005 per dust.
        assert!((pool.slot(9).fade_in - 0.05).abs() < 1e-6);
        assert_eq!(rng.next(), mirror.next());
        // The NaN guard: a zero-length beam spawns nothing.
        let mut zp = DustPool::new();
        let mut zr = UnifiedRandom::new(127);
        nebula_levelup_beam(source, source, 179, &mut zr, &mut zp, &c, &w);
        assert!(active(&zp).is_empty());
    }
}
