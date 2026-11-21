//! World-item `Dust.NewDust*` sites (W4-g15) — the item-entity dust lane.
//!
//! **Census correction (the lane's claim doc, research/65):** `Item.cs`
//! carries ZERO `Dust.NewDust*` call sites (verified by direct-file scan,
//! 49 981 lines). The item USE / PICKUP / SPAWN dust lives in
//! `WorldItem.cs` — the world-drop entity — exactly as research/15's
//! per-file table records (`WorldItem.cs | 17 | 1 | none | MISSING`).
//! This module ports those 17 `Dust.NewDust*` sites + documents the one
//! `Gore.NewGore` sibling (`:653`, the gore lane's).
//!
//! **Pool contract (W4-d50):** every spawn rides the verbatim
//! `tr_core::particles::DustPool::new_dust` / `new_dust_perfect`
//! (`Dust.cs:75-256` / `:54-63`) on the App's shared `Main.dust` pool —
//! the guards, the off-screen cull, the pressure bands, the default block,
//! the ChildSafety re-type and the per-type tweaks all ride it. The sentinel
//! (6000) is written through exactly like the C# (`Main.dust[6000]`).
//!
//! **RNG contract (S1-A-10):** one slot per C# call-pass — every draw here
//! rides the `UpdateItems` stream (`Main.SwapRandom("UpdateItems")` wraps
//! `WorldItem.UpdateItem`, `Main.cs:17779-17781`), in `Main.item` slot order.
//! No new streams are created. The `ShimmerEffect` burst is the one site the
//! C# runs from `GetShimmered` — still inside the same `UpdateItems` swap.
//!
//! **The seam:** the functions take a [`DustSink`] (disjoint field borrows),
//! NOT the whole `App`, so they run inside the items-pass closure where
//! `items = &mut app.items` is held (the `world_sim/npcs.rs` dust convention).
//! The caller captures `app.spawn_ctx()` once per tick before the borrow.
//!
//! Site map (line-ordered, the 17 dust sites):
//! * `:461`  — `UpdateShimmer`'s shimmered sparkle (309), 1/30 gate.
//! * `:649`  — `DespawnIfMeetingConditions` fallen-star (75) daytime burn (15 ×10).
//! * `:661`  — the 4143 mana-cloak-star expiry (15 ×20, CornflowerBlue→Indigo lerp).
//! * `:670`  — the 3822 DD2-energy despawn (61, Next(18,24) count).
//! * `:892`/`:971` — entry/exit SHIMMER splash (308 ×10, colour Next(6)).
//! * `:919`/`:998` — entry/exit HONEY splash (152 ×5).
//! * `:932`/`:1011` — entry/exit WATER splash (`dustWater()` ×10).
//! * `:946`/`:1025` — entry/exit LAVA splash (35 ×5).
//! * `:1217` — `UpdateItem_VisualEffects` meteorite (116) 1/250 gate (6).
//! * `:1299` — pixie dust (501) 1/6 gate (55, `this.color`).
//! * `:1629` — fallen-star (75) `% 12` sparkle (`NewDustPerfect` 228).
//! * `:1641` — boss-bag `% 12` sparkle (`NewDustPerfect` 279).
//! * `:1671` — `ShimmerEffect` transform burst (309 ×20, colour Next(6)).
//!
//! The `Gore.NewGore` sibling (`:653`, the fallen-star burn's 16/17 gore ×3)
//! is the gore lane's (W4-d55/f10) — its `Main.rand.Next(16,18)` draws are a
//! documented skip here, exactly as the rest of the port treats unported gore.

use glam::Vec2;
use tr_core::particles::{Color, DustPool, SpawnCtx};
use tr_core::world::World;
use tr_core::UnifiedRandom;

use crate::item::{ItemDrop, SplashKind, SIZE};

/// `Color.CornflowerBlue` (MonoGame): `(100, 149, 237, 255)`.
const CORNFLOWER_BLUE: Color = Color::new(100, 149, 237, 255);
/// `Color.Indigo` (MonoGame): `(75, 0, 130, 255)`.
const INDIGO: Color = Color::new(75, 0, 130, 255);

/// The disjoint field borrows a world-item dust site needs (the
/// `world_sim/npcs.rs` dust convention). `rng` is the `UpdateItems` stream,
/// `dust` the shared `Main.dust` pool, `world` the tile grid (the type-34 wet
/// probe + the cull read it), `ctx` the spawn inputs captured once per tick.
/// `audio` carries the splash / shimmer plays.
pub(crate) struct DustSink<'a> {
    pub rng: &'a mut UnifiedRandom,
    pub dust: &'a mut DustPool,
    pub world: &'a World,
    pub ctx: &'a SpawnCtx,
    pub audio: &'a mut crate::audio::Audio,
}

/// The four shimmer-palette colours the `Next(6)` switch picks
/// (`WorldItem.cs:466-480`, `:897-911`, `:976-990`, `:1673-1687`): cases
/// 0/1/2 are the three bright tints, the default (3/4/5) the lavender.
fn shimmer_palette(roll: i32) -> Color {
    match roll {
        0 => Color::new(255, 255, 210, 255),
        1 => Color::new(190, 245, 255, 255),
        2 => Color::new(255, 150, 255, 255),
        _ => Color::new(190, 175, 255, 255),
    }
}

/// XNA `Color.Lerp` per channel (`MonoGame Color.cs`): `MathHelper.Lerp`
/// then the ctor's truncating `(int)` cast. Shared shape with
/// `dust_table::lerp_color` (that one is module-private).
fn lerp_color(a: Color, b: Color, t: f32) -> Color {
    let ch = |x: u8, y: u8| {
        let (x, y) = (x as f32, y as f32);
        (x + (y - x) * t) as i32 as u8
    };
    Color::new(ch(a.r, b.r), ch(a.g, b.g), ch(a.b, b.b), ch(a.a, b.a))
}

/// `Utils.NextVector2Unit` (`Utils.cs:1261-1264`) — one `NextFloat`; the
/// angle is FLOAT arithmetic (`rotationRange * NextFloat()`, default range
/// `(float)Math.PI * 2f`), then `ToRotationVector2` (`Utils.cs:1665-1668`)
/// cos/sin in double of the widened float, cast back to f32. The
/// `npc_ai_empress::next_vector2_unit` precedent.
fn next_vector2_unit(rng: &mut UnifiedRandom) -> Vec2 {
    let angle = std::f32::consts::PI * 2.0 * rng.next_float();
    Vec2::new((angle as f64).cos() as f32, (angle as f64).sin() as f32)
}

/// `Utils.NextVector2CircularEdge` (`Utils.cs:1271-1274`) — the unit draw
/// times the half-extents, NO radius `NextFloat` (unlike `Circular`).
fn next_vector2_circular_edge(rng: &mut UnifiedRandom, half_w: f32, half_h: f32) -> Vec2 {
    let unit = next_vector2_unit(rng);
    Vec2::new(unit.x * half_w, unit.y * half_h)
}

// ---------------------------------------------------------------------------
// The `UpdateShimmer` sparkle (runs before the movement half's draws).
// ---------------------------------------------------------------------------

/// `WorldItem.UpdateShimmer`'s shimmered sparkle (`WorldItem.cs:459-481`):
/// a 1/30 `Next(30) == 0` gate, then one type-309 `NewDust` over the drop box,
/// the position jitter `Next(-8,5)` ×2, `scale *= 1.1`, `velocity *= 0.3` and
/// the `Next(6)` colour pick. Runs inside `UpdateShimmer` (the non-grabbed
/// branch), so the draw rides `UpdateItems` BEFORE the movement half's draws.
///
/// `shimmered` is the gate; a dry transformed drop still sparkles (the C#
/// arm is not liquid-gated). Returns `true` when a dust spawned.
pub(crate) fn shimmered_sparkle(s: &mut DustSink, drop: &ItemDrop) -> bool {
    if !drop.shimmered {
        return false;
    }
    // `:459` — the gate draws even when it refuses.
    if s.rng.next_max(30) != 0 {
        return false;
    }
    let pos = drop.pos;
    let (w, h) = (SIZE as i32, SIZE as i32);
    let idx = s.dust.new_dust(
        s.rng, s.ctx, s.world, pos, w, h, 309, 0.0, 0.0, 0, Color::default(), 1.0,
    );
    // `:462-465` — the post-writes (the sentinel included).
    let d = s.dust.slot_mut(idx);
    d.position.x += s.rng.next_range(-8, 5) as f32;
    d.position.y += s.rng.next_range(-8, 5) as f32;
    d.scale *= 1.1;
    d.velocity *= 0.3;
    // `:466-480` — the colour switch.
    let roll = s.rng.next_max(6);
    s.dust.slot_mut(idx).color = shimmer_palette(roll);
    true
}

// ---------------------------------------------------------------------------
// The `UpdateItem_VisualEffects` dust arms (meteorite / pixie dust).
// ---------------------------------------------------------------------------

/// `WorldItem.UpdateItem_VisualEffects`'s meteorite arm (`WorldItem.cs:1211-1222`,
/// type 116): the flicker light `Next(95,106)` (the `item_lights` head), then a
/// `Next(250) == 0` gate whose `NewDust(6, …, scale = Next(3))` chain spawns one
/// dust; `noGravity` when the resulting `scale > 1`. This is the dust half that
/// `item_lights::rand_light` deferred (it returned `None` for 116).
///
/// Returns the flicker light to add (the caller adds it to `entity_lights`).
/// The draws are ordered exactly as the C#: flicker, gate, then the `Next(3)`
/// scale argument, THEN the `NewDust` internals.
pub(crate) fn meteorite_visual(s: &mut DustSink, drop: &ItemDrop) -> Option<(i32, i32, [f32; 3])> {
    if drop.kind != 116 {
        return None;
    }
    // `:1213` — the flicker (no essScale term for 116).
    let num2 = s.rng.next_range(95, 106) as f32 * 0.01;
    let light = (
        crate::entity_lights::tile_x(drop.pos, SIZE, crate::entity_lights::XAnchor::Center),
        crate::entity_lights::tile_y(drop.pos, SIZE, crate::entity_lights::XAnchor::Center),
        [0.56 * num2, 0.43 * num2, 0.15 * num2],
    );
    // `:1215` — the 1/250 gate.
    if s.rng.next_max(250) != 0 {
        return Some(light);
    }
    // `:1217` — the scale argument `Main.rand.Next(3)` draws BEFORE NewDust.
    let scale = s.rng.next_max(3) as f32;
    let pos = drop.pos;
    let (w, h) = (SIZE as i32, SIZE as i32);
    let idx = s.dust.new_dust(
        s.rng, s.ctx, s.world, pos, w, h, 6, 0.0, 0.0, 0, Color::default(), scale,
    );
    // `:1218-1221` — noGravity when the landed scale > 1.
    if s.dust.slot(idx).scale > 1.0 {
        s.dust.slot_mut(idx).no_gravity = true;
    }
    Some(light)
}

/// `WorldItem.UpdateItem_VisualEffects`'s pixie-dust arm (`WorldItem.cs:1295-1303`,
/// type 501): a `Next(6) == 0` gate, then `NewDust(55, …, alpha 200, this.color)`,
/// `velocity *= 0.3`, `scale *= 0.5`. No `AddLight` — dust only.
///
/// `this.color` is the item's `inner.color` (`WorldItem.cs:84-92`); the C# 501
/// row sets no colour (`Item.cs:7349-7354`), so `SetDefaults`' `this.color =
/// default` (`Item.cs:49005`, all-zero) stands — `Color::default()` is exact.
/// Returns whether a dust spawned.
pub(crate) fn pixie_dust_visual(s: &mut DustSink, drop: &ItemDrop) -> bool {
    if drop.kind != 501 {
        return false;
    }
    if s.rng.next_max(6) != 0 {
        return false;
    }
    let pos = drop.pos;
    let (w, h) = (SIZE as i32, SIZE as i32);
    let idx = s.dust.new_dust(
        s.rng, s.ctx, s.world, pos, w, h, 55, 0.0, 0.0, 200, Color::default(), 1.0,
    );
    let d = s.dust.slot_mut(idx);
    d.velocity *= 0.3;
    d.scale *= 0.5;
    true
}

// ---------------------------------------------------------------------------
// The `% 12` sparkles (fallen star / boss bag).
// ---------------------------------------------------------------------------

/// The shared `% 12` sparkle body (`WorldItem.cs:1627-1634` fallen star /
/// `:1639-1647` boss bag): a `NewDustPerfect` at
/// `Center + (0, height*y_off) + CircularEdge(w*edge_w, height*0.6) *
/// (0.3 + NextFloat*0.5)`, velocity `(0, -NextFloat*0.3 - 1.5)`, alpha 127,
/// then `scale = 0.5`, `fadeIn = 1.1`, `noGravity`, `noLight` (the boss bag
/// also `alpha = 0`).
///
/// Argument draw order (the C# evaluates the `NewDustPerfect` arguments
/// first): `CircularEdge` (one `NextFloat`), the position-scale `NextFloat`,
/// the velocity `NextFloat`, THEN the `NewDust` internals. `y_off` is `0.2`
/// (star) / `-0.1` (bag); `edge_w` is `1.0` (star) / `0.6` (bag).
fn sparkle_perfect(
    s: &mut DustSink,
    drop: &ItemDrop,
    dust_type: i32,
    y_off: f32,
    edge_w: f32,
    bag_alpha_zero: bool,
) {
    let center = drop.center();
    let (w, h) = (SIZE, SIZE);
    // The CircularEdge draw (one NextFloat) — arguments evaluate first.
    let edge = next_vector2_circular_edge(s.rng, w * edge_w, h * 0.6);
    let pos_scale = 0.3 + s.rng.next_float() * 0.5;
    let pos = center + Vec2::new(0.0, h * y_off) + edge * pos_scale;
    let vy = (0.0 - s.rng.next_float()) * 0.3 - 1.5;
    let idx = s.dust.new_dust_perfect(
        s.rng,
        s.ctx,
        s.world,
        pos,
        dust_type,
        Some(Vec2::new(0.0, vy)),
        127,
        Color::default(),
        1.0,
    );
    let d = s.dust.slot_mut(idx);
    d.scale = 0.5;
    d.fade_in = 1.1;
    d.no_gravity = true;
    d.no_light = true;
    if bag_alpha_zero {
        d.alpha = 0;
    }
}

/// The fallen-star (75) `% 12` sparkle (`WorldItem.cs:1624-1635`). The light
/// half is already in `entity_lights::item_light`; this is the dust half.
/// Returns whether a dust spawned (the `% 12` gate is checked here).
pub(crate) fn fallen_star_sparkle(s: &mut DustSink, drop: &ItemDrop) -> bool {
    if drop.kind != 75 || drop.age_ticks % 12 != 0 {
        return false;
    }
    sparkle_perfect(s, drop, 228, 0.2, 1.0, false);
    true
}

/// The boss-bag `% 12` sparkle (`WorldItem.cs:1636-1648`). `BOSS_BAG` is the
/// `entity_lights` set (`ItemID.cs:1383`). The dust also zeroes `alpha`.
pub(crate) fn boss_bag_sparkle(s: &mut DustSink, drop: &ItemDrop) -> bool {
    if !crate::entity_lights::BOSS_BAG.contains(&drop.kind) || drop.age_ticks % 12 != 0 {
        return false;
    }
    sparkle_perfect(s, drop, 279, -0.1, 0.6, true);
    true
}

// ---------------------------------------------------------------------------
// The liquid entry/exit splash bursts.
// ---------------------------------------------------------------------------

/// One liquid splash edge (`WorldItem.cs:879-1035`): the entry (`entry=true`)
/// or exit (`entry=false`) burst for the resolved [`SplashKind`], plus the
/// case-19 play. The dust box is `(pos.X - 6, pos.Y + height/2 - 8)` with
/// `width + 12, 24` — EXCEPT the exit water arm (`:1011`) which drops the
/// `- 8` (the C#'s own asymmetry, kept verbatim).
///
/// The sound styles: entry shimmer 4 (`:913`), exit shimmer 5 (`:992`),
/// honey/lava/water the default 1 (`:926`/`:939`/`:953`/`:1005`/`:1018`/`:1032`).
/// `water_style` is `Main.waterStyle` (`Main.cs:57475`, the draw-pass value)
/// for the `dustWater()` pick.
pub(crate) fn splash_edge(s: &mut DustSink, drop: &ItemDrop, entry: bool, kind: SplashKind, water_style: i32) {
    let (w, h) = (SIZE as i32, SIZE as i32);
    // The common box (`:892` etc.); the exit-water arm overrides the y below.
    let mut pos = Vec2::new(drop.pos.x - 6.0, drop.pos.y + (h / 2) as f32 - 8.0);
    let bw = w + 12;
    let bh = 24;
    match kind {
        SplashKind::Shimmer => {
            // `:890-913` entry / `:969-992` exit — 10 × type 308.
            for _ in 0..10 {
                let idx = s.dust.new_dust(
                    s.rng, s.ctx, s.world, pos, bw, bh, 308, 0.0, 0.0, 0, Color::default(), 1.0,
                );
                let d = s.dust.slot_mut(idx);
                d.velocity.y -= 4.0;
                d.velocity.x *= 2.5;
                d.scale = 0.8;
                d.no_gravity = true;
                let roll = s.rng.next_max(6);
                s.dust.slot_mut(idx).color = shimmer_palette(roll);
            }
            // Entry style 4 (`:913`), exit style 5 (`:992`).
            let style = if entry { 4 } else { 5 };
            s.audio.play_splash_sound(style, drop.pos.x, drop.pos.y, s.rng);
        }
        SplashKind::Honey => {
            // `:917-926` entry / `:996-1005` exit — 5 × type 152.
            for _ in 0..5 {
                let idx = s.dust.new_dust(
                    s.rng, s.ctx, s.world, pos, bw, bh, 152, 0.0, 0.0, 0, Color::default(), 1.0,
                );
                let d = s.dust.slot_mut(idx);
                d.velocity.y -= 1.0;
                d.velocity.x *= 2.5;
                d.scale = 1.3;
                d.alpha = 100;
                d.no_gravity = true;
            }
            s.audio.play_splash_sound(1, drop.pos.x, drop.pos.y, s.rng);
        }
        SplashKind::Water => {
            // `:930-939` entry / `:1009-1018` exit — 10 × dustWater().
            // The EXIT arm's box drops the `- 8` (`:1011`).
            if !entry {
                pos = Vec2::new(drop.pos.x - 6.0, drop.pos.y + (h / 2) as f32);
            }
            let ty = tr_core::particles::dust_water(water_style);
            for _ in 0..10 {
                let idx = s.dust.new_dust(
                    s.rng, s.ctx, s.world, pos, bw, bh, ty, 0.0, 0.0, 0, Color::default(), 1.0,
                );
                let d = s.dust.slot_mut(idx);
                d.velocity.y -= 4.0;
                d.velocity.x *= 2.5;
                d.scale *= 0.8;
                d.alpha = 100;
                d.no_gravity = true;
            }
            s.audio.play_splash_sound(1, drop.pos.x, drop.pos.y, s.rng);
        }
        SplashKind::Lava => {
            // `:944-953` entry / `:1023-1032` exit — 5 × type 35.
            for _ in 0..5 {
                let idx = s.dust.new_dust(
                    s.rng, s.ctx, s.world, pos, bw, bh, 35, 0.0, 0.0, 0, Color::default(), 1.0,
                );
                let d = s.dust.slot_mut(idx);
                d.velocity.y -= 1.5;
                d.velocity.x *= 2.5;
                d.scale = 1.3;
                d.alpha = 100;
                d.no_gravity = true;
            }
            s.audio.play_splash_sound(1, drop.pos.x, drop.pos.y, s.rng);
        }
    }
}

// ---------------------------------------------------------------------------
// The despawn burns (the caller airs the drop out on a `true` return).
// ---------------------------------------------------------------------------

/// `WorldItem.DespawnIfMeetingConditions`'s fallen-star daytime burn
/// (`WorldItem.cs:645-656`, type 75): 10 × `NewDust(15, velocity, alpha 150,
/// scale 1.2)`, then the 3 × `Gore.NewGore(16/17)` siblings (the gore lane's
/// skip), then `TurnToAirAndSync`. The caller gates on the local-player
/// reservation (`:641`) and removes the drop. Returns `true` when the burn
/// fired (the drop should air out).
pub(crate) fn fallen_star_burn(s: &mut DustSink, drop: &ItemDrop, day_time: bool, remix: bool, was_grabbed: bool) -> bool {
    if drop.kind != 75 || !day_time || remix || drop.shimmered || was_grabbed {
        return false;
    }
    let pos = drop.pos;
    let (w, h) = (SIZE as i32, SIZE as i32);
    for _ in 0..10 {
        s.dust.new_dust(
            s.rng, s.ctx, s.world, pos, w, h, 15, drop.vel.x, drop.vel.y, 150,
            Color::default(), 1.2,
        );
    }
    // `:651-654` — the 3 gore siblings are the gore lane's (W4-d55/f10);
    // their `Next(16,18)` draws are a documented skip here.
    true
}

/// `WorldItem.DespawnIfMeetingConditions`'s 4143 mana-cloak-star expiry
/// (`WorldItem.cs:657-664`): after 300 ticks, 20 × `NewDust(15, velocity,
/// alpha 150, Color.Lerp(CornflowerBlue, Indigo, NextFloat), scale 1.2)`,
/// then `TurnToAirAndSync`. The per-dust `NextFloat` colour draw rides the
/// stream BEFORE the `NewDust` internals (the argument evaluates first).
pub(crate) fn mana_cloak_star_expiry(s: &mut DustSink, drop: &ItemDrop) -> bool {
    if drop.kind != 4143 || drop.age_ticks <= 300 {
        return false;
    }
    let pos = drop.pos;
    let (w, h) = (SIZE as i32, SIZE as i32);
    for _ in 0..20 {
        // `:661` — the colour argument draws first.
        let t = s.rng.next_float();
        let color = lerp_color(CORNFLOWER_BLUE, INDIGO, t);
        s.dust.new_dust(
            s.rng, s.ctx, s.world, pos, w, h, 15, drop.vel.x, drop.vel.y, 150, color, 1.2,
        );
    }
    true
}

/// `WorldItem.DespawnIfMeetingConditions`'s 3822 DD2-energy despawn
/// (`WorldItem.cs:665-678`): when no DD2 event is ongoing, `Next(18,24)`
/// dust at `Center` (0×0 box, type 61, scale 1.7), each `velocity *= 8`,
/// `velocity.Y--`, `position = Lerp(position, Center, 0.5)`, `noGravity`,
/// `noLight`, then `TurnToAirAndSync`.
pub(crate) fn dd2_energy_despawn(s: &mut DustSink, drop: &ItemDrop, dd2_ongoing: bool) -> bool {
    if drop.kind != 3822 || dd2_ongoing {
        return false;
    }
    let count = s.rng.next_range(18, 24);
    let center = drop.center();
    for _ in 0..count {
        let idx = s.dust.new_dust(
            s.rng, s.ctx, s.world, center, 0, 0, 61, 0.0, 0.0, 0, Color::default(), 1.7,
        );
        let d = s.dust.slot_mut(idx);
        d.velocity *= 8.0;
        d.velocity.y -= 1.0;
        // `Vector2.Lerp(position, Center, 0.5)` — the midpoint.
        d.position = d.position + (center - d.position) * 0.5;
        d.no_gravity = true;
        d.no_light = true;
    }
    true
}

/// `WorldItem.ShimmerEffect` (`WorldItem.cs:1661-1689`): the transform burst
/// — 20 × `NewDust(shimmerPositon, 1, 1, 309)`, each `scale *= 1.2` and the
/// `Next(6)` colour pick. The `SoundID.Item176` play precedes the loop
/// (`:1668`); the port plays it through the item-use façade (the audio-only
/// `GetRandomPitch` NextFloat, variance 0). Called from `GetShimmered`'s tail
/// (`:1846`) — still inside the `UpdateItems` swap.
///
/// The `netMode == 2` server branch (`:1663-1667`) is dropped (single-player).
pub(crate) fn shimmer_effect(s: &mut DustSink, center: Vec2) {
    // `:1668` — Item176 (the SOUND_STYLES index, `soundid.rs:329`).
    let idx176 = crate::soundid::SOUND_STYLES
        .iter()
        .position(|st| st.name == "Item176")
        .expect("Item176 in the style table") as u16;
    s.audio.play_use_sound(idx176, center.x, center.y, 0.0, s.rng);
    for _ in 0..20 {
        let idx = s.dust.new_dust(
            s.rng, s.ctx, s.world, center, 1, 1, 309, 0.0, 0.0, 0, Color::default(), 1.0,
        );
        s.dust.slot_mut(idx).scale *= 1.2;
        let roll = s.rng.next_max(6);
        s.dust.slot_mut(idx).color = shimmer_palette(roll);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tr_core::particles::DUST_SENTINEL;

    /// A 400×300 air world (the `npc_ai_style74` fixture) — no liquid, so the
    /// type-34 wet probe never fires and the cull sees the whole screen.
    fn air() -> World {
        World::new("t".into(), 400, 300)
    }

    /// On-screen ctx, low pressure band (the style56/74 fixture): every slot
    /// is below `max * 0.5`, so the pressure bands reset `dCount` and draw
    /// nothing — the witnesses isolate the sites' EXPLICIT draws.
    fn ctx() -> SpawnCtx {
        SpawnCtx {
            game_menu: false,
            game_paused: false,
            generating: false,
            screen_position: Vec2::ZERO,
            screen_size: Vec2::new(1920.0, 1080.0),
            max_dust_to_draw: 6000,
        }
    }

    /// The menu ctx: `NewDust` refuses BEFORE any draw (`Dust.cs:77`) —
    /// isolates a site's explicit argument draws for the stream witnesses.
    fn menu_ctx() -> SpawnCtx {
        SpawnCtx { game_menu: true, ..ctx() }
    }

    fn drop_at(kind: u16, x: f32, y: f32) -> ItemDrop {
        ItemDrop::new(kind, 1, 0, x, y, 0.0, 0.0, 0.0)
    }

    /// `Dust.NewDust`'s default-block draw mirror (`Dust.cs:170-175`, `:190`):
    /// six draws under the no-pressure / no-ChildSafety assumption.
    fn nd_draws(rng: &mut UnifiedRandom, w: i32, h: i32) {
        let w = w.max(5);
        let h = h.max(5);
        rng.next_max(w - 4);
        rng.next_max(h - 4);
        rng.next_range(-20, 21);
        rng.next_range(-20, 21);
        rng.next_max(3);
        rng.next_range(-20, 21);
    }

    /// The per-type tweak chain's draw (`Dust.cs:214-219`): the types in the
    /// first `if` (228/279/6/61/…) draw one `Next(-10,6)` for velocity.y. The
    /// 308/33/35/152 arms draw nothing. Appended to `nd_draws` for those types.
    fn tweak_draw(rng: &mut UnifiedRandom, dust_type: i32) {
        if matches!(
            dust_type,
            228 | 279 | 269 | 135 | 6 | 242 | 75 | 169 | 29 | 59..=65 | 158
                | 293..=298 | 302 | 307 | 310
        ) {
            rng.next_range(-10, 6);
        }
    }

    // --- The shimmered sparkle (`:459-481`). ------------------------------

    /// A non-shimmered drop never draws (the gate is `shimmered`).
    #[test]
    fn w4_g15_sparkle_gates_on_shimmered() {
        let w = air();
        let c = ctx();
        let mut pool = DustPool::new();
        let mut audio = crate::audio::Audio::new(None);
        let mut rng = UnifiedRandom::new(7);
        let witness = UnifiedRandom::new(7);
        let d = drop_at(2, 100.0, 100.0); // not shimmered
        let fired = {
            let mut s = DustSink { rng: &mut rng, dust: &mut pool, world: &w, ctx: &c, audio: &mut audio };
            shimmered_sparkle(&mut s, &d)
        };
        assert!(!fired);
        assert_eq!(rng, witness, "no draw for a plain drop");
    }

    /// The 1/30 gate draws even when it refuses; a refusal spawns nothing.
    #[test]
    fn w4_g15_sparkle_gate_draws_on_refusal() {
        let w = air();
        let c = menu_ctx(); // NewDust would refuse, but the gate draws first
        let mut pool = DustPool::new();
        let mut audio = crate::audio::Audio::new(None);
        let mut rng = UnifiedRandom::new(7);
        let mut witness = UnifiedRandom::new(7);
        let mut d = drop_at(2, 100.0, 100.0);
        d.shimmered = true;
        let gate = witness.next_max(30);
        let fired = {
            let mut s = DustSink { rng: &mut rng, dust: &mut pool, world: &w, ctx: &c, audio: &mut audio };
            shimmered_sparkle(&mut s, &d)
        };
        if gate != 0 {
            assert!(!fired);
            assert_eq!(rng, witness, "exactly the one gate draw");
        }
    }

    /// A fired sparkle spends the gate + NewDust's six + the two position
    /// jitters + the colour pick, in C# order, and writes the post-arms.
    #[test]
    fn w4_g15_sparkle_full_draw_sequence() {
        let w = air();
        let c = ctx();
        // Find a seed whose first Next(30) == 0 so the gate fires.
        let mut seed = 0;
        while UnifiedRandom::new(seed).next_max(30) != 0 {
            seed += 1;
        }
        let mut pool = DustPool::new();
        let mut audio = crate::audio::Audio::new(None);
        let mut rng = UnifiedRandom::new(seed);
        let mut witness = UnifiedRandom::new(seed);
        let mut d = drop_at(2, 100.0, 100.0);
        d.shimmered = true;
        witness.next_max(30); // the gate
        nd_draws(&mut witness, 14, 14);
        witness.next_range(-8, 5);
        witness.next_range(-8, 5);
        witness.next_max(6);
        let fired = {
            let mut s = DustSink { rng: &mut rng, dust: &mut pool, world: &w, ctx: &c, audio: &mut audio };
            shimmered_sparkle(&mut s, &d)
        };
        assert!(fired);
        assert_eq!(rng, witness, "gate + NewDust + jitter×2 + colour");
        // The spawned dust is type 309 with the post-arms applied.
        let slot = pool.slots().iter().find(|p| p.active).expect("a dust spawned");
        assert_eq!(slot.dust_type, 309);
        assert!(!slot.no_gravity, "309 keeps gravity");
    }

    // --- The meteorite arm (`:1211-1222`). --------------------------------

    /// The flicker draws `Next(95,106)` then the 1/250 gate; a refusal still
    /// returns the light and spends exactly those two draws.
    #[test]
    fn w4_g15_meteorite_flicker_and_gate() {
        let w = air();
        let c = menu_ctx();
        let mut pool = DustPool::new();
        let mut audio = crate::audio::Audio::new(None);
        let mut rng = UnifiedRandom::new(7);
        let mut witness = UnifiedRandom::new(7);
        let flicker = witness.next_range(95, 106);
        let gate = witness.next_max(250);
        // If the gate fires, the `Next(3)` scale argument draws BEFORE NewDust
        // (which then refuses on menu — no default-block draws).
        if gate == 0 {
            witness.next_max(3);
        }
        let d = drop_at(116, 160.0, 176.0);
        let light = {
            let mut s = DustSink { rng: &mut rng, dust: &mut pool, world: &w, ctx: &c, audio: &mut audio };
            meteorite_visual(&mut s, &d).expect("116 lights")
        };
        assert_eq!(rng, witness, "flicker + gate (+ scale arg if fired)");
        let num = flicker as f32 * 0.01;
        assert_eq!(light.2, [0.56 * num, 0.43 * num, 0.15 * num]);
        // center (167, 183) -> tile (10, 11).
        assert_eq!((light.0, light.1), (10, 11));
        // Menu ctx: NewDust refuses, so no dust ever spawns here.
        assert!(pool.slots().iter().all(|p| !p.active), "menu refusal: no dust");
    }

    /// A non-116 kind returns None without drawing.
    #[test]
    fn w4_g15_meteorite_other_kinds_draw_nothing() {
        let w = air();
        let c = ctx();
        let mut pool = DustPool::new();
        let mut audio = crate::audio::Audio::new(None);
        let mut rng = UnifiedRandom::new(7);
        let witness = UnifiedRandom::new(7);
        let d = drop_at(2, 100.0, 100.0);
        let none = {
            let mut s = DustSink { rng: &mut rng, dust: &mut pool, world: &w, ctx: &c, audio: &mut audio };
            meteorite_visual(&mut s, &d).is_none()
        };
        assert!(none);
        assert_eq!(rng, witness);
    }

    // --- The pixie-dust arm (`:1295-1303`). -------------------------------

    /// The 1/6 gate draws even on refusal; a fired dust is type 55, alpha 200.
    #[test]
    fn w4_g15_pixie_dust_gate_and_type() {
        let w = air();
        let c = ctx();
        let mut pool = DustPool::new();
        let mut audio = crate::audio::Audio::new(None);
        let mut rng = UnifiedRandom::new(1);
        let mut witness = UnifiedRandom::new(1);
        let gate = witness.next_max(6);
        let d = drop_at(501, 100.0, 100.0);
        let fired = {
            let mut s = DustSink { rng: &mut rng, dust: &mut pool, world: &w, ctx: &c, audio: &mut audio };
            pixie_dust_visual(&mut s, &d)
        };
        if gate == 0 {
            assert!(fired);
            nd_draws(&mut witness, 14, 14);
            assert_eq!(rng, witness, "gate + NewDust's six");
            let slot = pool.slots().iter().find(|p| p.active).expect("a dust");
            assert_eq!(slot.dust_type, 55);
            assert_eq!(slot.alpha, 200);
        } else {
            assert!(!fired);
            assert_eq!(rng, witness, "just the gate draw");
        }
    }

    // --- The `% 12` sparkles (`:1624-1648`). ------------------------------

    /// The fallen-star sparkle gates on kind 75 AND `age_ticks % 12 == 0`.
    #[test]
    fn w4_g15_fallen_star_sparkle_gate() {
        let w = air();
        let c = ctx();
        let mut pool = DustPool::new();
        let mut audio = crate::audio::Audio::new(None);
        let mut rng = UnifiedRandom::new(7);
        let witness = UnifiedRandom::new(7);
        let mut d = drop_at(75, 100.0, 100.0);
        d.age_ticks = 5; // not a multiple of 12
        let fired = {
            let mut s = DustSink { rng: &mut rng, dust: &mut pool, world: &w, ctx: &c, audio: &mut audio };
            fallen_star_sparkle(&mut s, &d)
        };
        assert!(!fired);
        assert_eq!(rng, witness, "the % 12 gate draws nothing");
        // A non-75 kind also refuses.
        let d2 = drop_at(2, 100.0, 100.0);
        let fired2 = {
            let mut s = DustSink { rng: &mut rng, dust: &mut pool, world: &w, ctx: &c, audio: &mut audio };
            fallen_star_sparkle(&mut s, &d2)
        };
        assert!(!fired2);
        assert_eq!(rng, witness);
    }

    /// A fired star sparkle spends CircularEdge (1 float) + the position-scale
    /// float + the velocity float + NewDustPerfect's six, and writes the
    /// post-arms (scale 0.5, fadeIn 1.1, noGravity, noLight).
    #[test]
    fn w4_g15_fallen_star_sparkle_draw_sequence() {
        let w = air();
        let c = ctx();
        let mut pool = DustPool::new();
        let mut audio = crate::audio::Audio::new(None);
        let mut rng = UnifiedRandom::new(7);
        let mut witness = UnifiedRandom::new(7);
        // Arguments first: CircularEdge (1 NextFloat), pos-scale, velocity.
        witness.next_float();
        witness.next_float();
        witness.next_float();
        nd_draws(&mut witness, 0, 0); // NewDustPerfect forwards 0×0 -> clamp 5
        tweak_draw(&mut witness, 228); // the per-type velocity.y draw
        let mut d = drop_at(75, 100.0, 100.0);
        d.age_ticks = 12;
        let fired = {
            let mut s = DustSink { rng: &mut rng, dust: &mut pool, world: &w, ctx: &c, audio: &mut audio };
            fallen_star_sparkle(&mut s, &d)
        };
        assert!(fired);
        assert_eq!(rng, witness, "edge + 2 floats + NewDust's six");
        let slot = pool.slots().iter().find(|p| p.active).expect("a dust");
        assert_eq!(slot.dust_type, 228);
        assert_eq!(slot.scale, 0.5);
        assert_eq!(slot.fade_in, 1.1);
        assert!(slot.no_gravity);
        assert!(slot.no_light);
    }

    /// The boss-bag sparkle gates on the BOSS_BAG set + `% 12`, and zeroes
    /// alpha (the bag's extra post-write).
    #[test]
    fn w4_g15_boss_bag_sparkle_zeroes_alpha() {
        let w = air();
        let c = ctx();
        let mut pool = DustPool::new();
        let mut audio = crate::audio::Audio::new(None);
        let mut rng = UnifiedRandom::new(7);
        let mut witness = UnifiedRandom::new(7);
        witness.next_float();
        witness.next_float();
        witness.next_float();
        nd_draws(&mut witness, 0, 0);
        tweak_draw(&mut witness, 279); // the per-type velocity.y draw
        let mut d = drop_at(3318, 100.0, 100.0); // Eye of Cthulhu Boss Bag
        d.age_ticks = 24;
        let fired = {
            let mut s = DustSink { rng: &mut rng, dust: &mut pool, world: &w, ctx: &c, audio: &mut audio };
            boss_bag_sparkle(&mut s, &d)
        };
        assert!(fired);
        assert_eq!(rng, witness);
        let slot = pool.slots().iter().find(|p| p.active).expect("a dust");
        assert_eq!(slot.dust_type, 279);
        assert_eq!(slot.alpha, 0, "the bag zeroes alpha");
        assert!(slot.no_gravity && slot.no_light);
    }

    // --- The splash bursts (`:879-1035`). ---------------------------------

    /// A water entry spawns 10 dust of `dustWater(style)` and plays case-19
    /// style 1 (one pitch draw). The dust type follows the water style.
    #[test]
    fn w4_g15_water_entry_splash_count_and_type() {
        let w = air();
        let c = ctx();
        let mut pool = DustPool::new();
        let mut audio = crate::audio::Audio::new(None);
        let mut rng = UnifiedRandom::new(7);
        let d = drop_at(2, 100.0, 100.0);
        {
            let mut s = DustSink { rng: &mut rng, dust: &mut pool, world: &w, ctx: &c, audio: &mut audio };
            splash_edge(&mut s, &d, true, SplashKind::Water, 0);
        }
        let active: Vec<_> = pool.slots().iter().filter(|p| p.active).collect();
        assert_eq!(active.len(), 10, "10 water dust");
        for p in &active {
            assert_eq!(p.dust_type, tr_core::particles::dust_water(0));
            assert!(p.no_gravity);
            assert_eq!(p.alpha, 100);
        }
    }

    /// A shimmer entry spawns 10 type-308 dust, each with a colour-pick draw,
    /// and plays case-19 style 4 (the narrow pitch window).
    #[test]
    fn w4_g15_shimmer_entry_splash() {
        let w = air();
        let c = ctx();
        let mut pool = DustPool::new();
        let mut audio = crate::audio::Audio::new(None);
        let mut rng = UnifiedRandom::new(7);
        let d = drop_at(2, 100.0, 100.0);
        {
            let mut s = DustSink { rng: &mut rng, dust: &mut pool, world: &w, ctx: &c, audio: &mut audio };
            splash_edge(&mut s, &d, true, SplashKind::Shimmer, 0);
        }
        let active: Vec<_> = pool.slots().iter().filter(|p| p.active).collect();
        assert_eq!(active.len(), 10);
        for p in &active {
            assert_eq!(p.dust_type, 308);
            assert!(p.no_gravity);
            assert_eq!(p.scale, 0.8);
        }
    }

    /// A lava exit spawns 5 type-35 dust; a honey entry spawns 5 type-152.
    #[test]
    fn w4_g15_lava_and_honey_splash_counts() {
        let w = air();
        let c = ctx();
        for (kind, ty, entry, n) in [
            (SplashKind::Lava, 35, false, 5),
            (SplashKind::Honey, 152, true, 5),
        ] {
            let mut pool = DustPool::new();
            let mut audio = crate::audio::Audio::new(None);
            let mut rng = UnifiedRandom::new(7);
            let d = drop_at(2, 100.0, 100.0);
            {
                let mut s = DustSink { rng: &mut rng, dust: &mut pool, world: &w, ctx: &c, audio: &mut audio };
                splash_edge(&mut s, &d, entry, kind, 0);
            }
            let active: Vec<_> = pool.slots().iter().filter(|p| p.active).collect();
            assert_eq!(active.len(), n, "{kind:?}");
            for p in &active {
                assert_eq!(p.dust_type, ty);
                assert!(p.no_gravity);
            }
        }
    }

    // --- The despawn burns (`:639-679`). ----------------------------------

    /// The fallen-star burn gates on dayTime / !remix / !shimmered /
    /// !beingGrabbed; a refusal draws nothing.
    #[test]
    fn w4_g15_fallen_star_burn_gates() {
        let w = air();
        let c = ctx();
        let mut pool = DustPool::new();
        let mut audio = crate::audio::Audio::new(None);
        let mut rng = UnifiedRandom::new(7);
        let witness = UnifiedRandom::new(7);
        let d = drop_at(75, 100.0, 100.0);
        // Night: refuses.
        let night = {
            let mut s = DustSink { rng: &mut rng, dust: &mut pool, world: &w, ctx: &c, audio: &mut audio };
            fallen_star_burn(&mut s, &d, false, false, false)
        };
        assert!(!night);
        // Remix: refuses.
        let remix = {
            let mut s = DustSink { rng: &mut rng, dust: &mut pool, world: &w, ctx: &c, audio: &mut audio };
            fallen_star_burn(&mut s, &d, true, true, false)
        };
        assert!(!remix);
        assert_eq!(rng, witness, "the gates draw nothing");
    }

    /// A fired burn spawns 10 type-15 dust at the drop's velocity.
    #[test]
    fn w4_g15_fallen_star_burn_spawns_ten() {
        let w = air();
        let c = ctx();
        let mut pool = DustPool::new();
        let mut audio = crate::audio::Audio::new(None);
        let mut rng = UnifiedRandom::new(7);
        let mut d = drop_at(75, 100.0, 100.0);
        d.vel = Vec2::new(1.0, 2.0);
        let fired = {
            let mut s = DustSink { rng: &mut rng, dust: &mut pool, world: &w, ctx: &c, audio: &mut audio };
            fallen_star_burn(&mut s, &d, true, false, false)
        };
        assert!(fired);
        let active: Vec<_> = pool.slots().iter().filter(|p| p.active).collect();
        assert_eq!(active.len(), 10);
        for p in &active {
            assert_eq!(p.dust_type, 15);
            assert_eq!(p.alpha, 150);
        }
    }

    /// The 4143 expiry gates on `age_ticks > 300`; a fired expiry spawns 20
    /// type-15 dust with the CornflowerBlue→Indigo lerp colour.
    #[test]
    fn w4_g15_mana_cloak_star_expiry() {
        let w = air();
        let c = ctx();
        let mut pool = DustPool::new();
        let mut audio = crate::audio::Audio::new(None);
        let mut rng = UnifiedRandom::new(7);
        let witness = UnifiedRandom::new(7);
        let mut d = drop_at(4143, 100.0, 100.0);
        d.age_ticks = 300; // not > 300: refuses
        let refused = {
            let mut s = DustSink { rng: &mut rng, dust: &mut pool, world: &w, ctx: &c, audio: &mut audio };
            mana_cloak_star_expiry(&mut s, &d)
        };
        assert!(!refused);
        assert_eq!(rng, witness);
        d.age_ticks = 301;
        let fired = {
            let mut s = DustSink { rng: &mut rng, dust: &mut pool, world: &w, ctx: &c, audio: &mut audio };
            mana_cloak_star_expiry(&mut s, &d)
        };
        assert!(fired);
        let active: Vec<_> = pool.slots().iter().filter(|p| p.active).collect();
        assert_eq!(active.len(), 20);
        for p in &active {
            assert_eq!(p.dust_type, 15);
        }
    }

    /// The 3822 despawn gates on !DD2; a fired despawn spawns Next(18,24)
    /// type-61 dust with noGravity + noLight.
    #[test]
    fn w4_g15_dd2_energy_despawn() {
        let w = air();
        let c = ctx();
        let mut pool = DustPool::new();
        let mut audio = crate::audio::Audio::new(None);
        let mut rng = UnifiedRandom::new(7);
        let witness = UnifiedRandom::new(7);
        let d = drop_at(3822, 100.0, 100.0);
        // DD2 ongoing: refuses, no draw.
        let refused = {
            let mut s = DustSink { rng: &mut rng, dust: &mut pool, world: &w, ctx: &c, audio: &mut audio };
            dd2_energy_despawn(&mut s, &d, true)
        };
        assert!(!refused);
        assert_eq!(rng, witness);
        let fired = {
            let mut s = DustSink { rng: &mut rng, dust: &mut pool, world: &w, ctx: &c, audio: &mut audio };
            dd2_energy_despawn(&mut s, &d, false)
        };
        assert!(fired);
        let n = pool.slots().iter().filter(|p| p.active).count();
        assert!((18..24).contains(&n), "Next(18,24) dust, got {n}");
        for p in pool.slots().iter().filter(|p| p.active) {
            assert_eq!(p.dust_type, 61);
            assert!(p.no_gravity && p.no_light);
        }
    }

    // --- The shimmer-transform burst (`:1661-1689`). ----------------------

    /// The ShimmerEffect burst spawns 20 type-309 dust, each scale ×1.2 and a
    /// colour pick; the Item176 play precedes the loop.
    #[test]
    fn w4_g15_shimmer_effect_burst() {
        let w = air();
        let c = ctx();
        let mut pool = DustPool::new();
        let mut audio = crate::audio::Audio::new(None);
        let mut rng = UnifiedRandom::new(7);
        {
            let mut s = DustSink { rng: &mut rng, dust: &mut pool, world: &w, ctx: &c, audio: &mut audio };
            shimmer_effect(&mut s, Vec2::new(100.0, 100.0));
        }
        let active: Vec<_> = pool.slots().iter().filter(|p| p.active).collect();
        assert_eq!(active.len(), 20);
        for p in &active {
            assert_eq!(p.dust_type, 309);
        }
    }

    /// The palette switch: cases 0/1/2 the bright tints, default the lavender.
    #[test]
    fn w4_g15_shimmer_palette_rows() {
        assert_eq!(shimmer_palette(0), Color::new(255, 255, 210, 255));
        assert_eq!(shimmer_palette(1), Color::new(190, 245, 255, 255));
        assert_eq!(shimmer_palette(2), Color::new(255, 150, 255, 255));
        for r in 3..6 {
            assert_eq!(shimmer_palette(r), Color::new(190, 175, 255, 255));
        }
    }

    /// The CornflowerBlue→Indigo lerp at the endpoints (the 4143 colour).
    #[test]
    fn w4_g15_lerp_endpoints() {
        assert_eq!(lerp_color(CORNFLOWER_BLUE, INDIGO, 0.0), CORNFLOWER_BLUE);
        assert_eq!(lerp_color(CORNFLOWER_BLUE, INDIGO, 1.0), INDIGO);
    }

    /// The sentinel is written through on a menu refusal (the C# writes
    /// `Main.dust[6000]`); the sparkle's post-arms land on the sentinel slot.
    /// `NewDust` returns BEFORE the default block on menu (`Dust.cs:77`), so
    /// only the gate + the post-arm draws ride the stream.
    #[test]
    fn w4_g15_menu_refusal_writes_sentinel() {
        let w = air();
        let c = menu_ctx();
        // Find a seed whose first Next(30) == 0 so the gate fires.
        let mut seed = 0;
        while UnifiedRandom::new(seed).next_max(30) != 0 {
            seed += 1;
        }
        let mut pool = DustPool::new();
        let mut audio = crate::audio::Audio::new(None);
        let mut rng = UnifiedRandom::new(seed);
        let mut witness = UnifiedRandom::new(seed);
        witness.next_max(30); // the gate
        // NewDust refused (menu): no default-block draws. The post-arms draw.
        witness.next_range(-8, 5);
        witness.next_range(-8, 5);
        witness.next_max(6);
        let mut d = drop_at(2, 100.0, 100.0);
        d.shimmered = true;
        let fired = {
            let mut s = DustSink { rng: &mut rng, dust: &mut pool, world: &w, ctx: &c, audio: &mut audio };
            shimmered_sparkle(&mut s, &d)
        };
        assert!(fired, "the slot index is returned even on refusal");
        assert_eq!(rng, witness, "menu refusal: gate + post-arms only");
        assert!(!pool.slot(DUST_SENTINEL).active, "the sentinel stays inactive");
    }
}