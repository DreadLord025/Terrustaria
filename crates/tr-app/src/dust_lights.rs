//! The Dust `Lighting.AddLight` sources (`S1-E-5`, dust family).
//!
//! C# `Lighting.AddLight` funnels every overload into
//! `Lighting.AddLight(int i, int j, float r, float g, float b)`
//! (`Lighting.cs:157-163`), which appends to the active engine's per-frame
//! list behind `!Main.gamePaused && Main.netMode != 2`
//! (`LightingEngine.cs:51-54`). The engine then max-merges that list onto the
//! working light map (`LightingEngine.cs:208-235`), one frame late. There is
//! ONE list pair per engine — [`crate::entity_lights::EntityLights`] — and
//! this family feeds the same pair as the item family
//! (`entity_lights.rs::item_light`) and the NPC family
//! (`npc_lights.rs::npc_light`); the tile-anchor vocabulary is shared from
//! there too.
//!
//! Ported here: the RNG-free `AddLight` arms of `Dust.UpdateDust`'s FIRST
//! ladder — the standalone per-type statements between the `:477` move and
//! the `:822` torch-family chain, the torch/fire family itself
//! (`Dust.cs:822-1023`) and the Sandnado arm (`:1037-1047`). 42 of the dust
//! family's 94 call sites. Every dust arm anchors on the dust's own
//! `position` — `(int)(dust.position.X / 16f)` per axis, i.e. the
//! [`XAnchor::Center`] form with a zero box (a dust has no width/height).
//!
//! # What stays dropped, exactly
//!
//! * **No arm here draws RNG and none mutates the dust.** The `Main.rand`
//!   arms (235's `Next(-100,101)` pair `:817`, 159's `Next(2)` tail `:1068`,
//!   and every rand-bearing arm of the second chain) are skipped wholesale,
//!   so the stream position is untouched — the same rule the item/NPC
//!   families follow. The scale/velocity mutations that share an arm with a
//!   light (206's `scale -= 0.1f` `:766`, 174's `scale -= 0.01f` `:807`, the
//!   family's `velocity.Y += 0.05f` `:826`) belong to the per-type chain
//!   (`tr_core::dust_physics::type_chain`, WIRED as the `type_arm` hook at
//!   W4-d50): the feed reads the POST-update pool record, so the former
//!   local-decrement compensations are RETIRED arm-by-arm (research/69 §5):
//!   206/174 read `d.scale` directly (post-decrement), 230 recovers the
//!   PRE-increment `d.scale - 0.02` (the C# light reads it before `:639`),
//!   309 recovers the PRE-shrink `d.scale / 0.97` (the light precedes
//!   `:486`), 261 recovers the PRE-drift scale (the light precedes
//!   `:695-704`: undo the −0.0025 and the fadeIn-gated +0.0025).
//! * **`customData` is dropped** (`particles.rs`'s `Dust` record): the 75
//!   (CursedTorch) arm's `customData is float` branch (`:867-870`,
//!   `num30 * (float)customData`) cannot fire, so the arm always takes its
//!   `else` (`:873`). The follower halves (325/86-92/263/264/182/261/230's
//!   `position += owner delta`) are position mutations, not lights — S1-E-3.
//! * **The second chain's lights (`Dust.cs:1099-2312`, 52 sites)** — the
//!   147/156-family glow sticks, the moon-lord/boss dusts, the 213/260
//!   fadeIn ramp's neighbours — interleave their `AddLight` with
//!   velocity/scale mutations, `Main.rand` draws and `customData` reads;
//!   they land with S1-E-3's chain port. (213/260's own light at `:757` is
//!   in the FIRST ladder and IS ported — the minecart wheel sparks glow.)
//!
//! The feed runs in the frame pass (`app/frame/draw.rs`) beside the item and
//! NPC feeds, after the step, in `Main.dust` slot order — the same order
//! `UpdateDust` walks (`Dust.cs:431`). Slots past `maxDustToDraw` were
//! already deactivated by the update pass (`Dust.cs:2393-2395`,
//! `particles.rs`), so the active check is the whole gate.

use tr_core::lighting::LightParams;
use tr_core::particles::{DUST_SENTINEL, Dust, DustPool};
use tr_core::tile_light::torch_color;

use crate::entity_lights::{EntityLights, XAnchor, tile_x, tile_y};

/// The C# cap idiom `if (num > m) num = m;` — not `f32::min`, which resolves
/// NaN differently; the port mirrors the branch.
#[inline]
fn cap(v: f32, m: f32) -> f32 {
    if v > m { m } else { v }
}

/// `Lighting.AddLight(int i, int j, int torchID, float lightAmount)`
/// (`Lighting.cs:145-149`): `TorchID.TorchColor(torchID)` scaled per channel
/// by `lightAmount`.
#[inline]
fn torch_rgb(id: i32, amount: f32, p: &LightParams) -> [f32; 3] {
    let c = torch_color(id, p);
    [c[0] * amount, c[1] * amount, c[2] * amount]
}

/// The torch/fire family chain (`Dust.cs:822-1023`): the shared
/// `!noLight && !noLightEmittance` gate, `num30 = scale * 1.4f`, then the
/// per-type cap + colour. The `velocity.Y += 0.05f` gravity half (`:826`)
/// and the 228/229/279 follower half (`:828-849`) are mutations — S1-E-3's.
fn torch_family_light(d: &Dust, p: &LightParams) -> Option<[f32; 3]> {
    // `:850` — the gate wraps the whole light block.
    if d.no_light || d.no_light_emittance {
        return None;
    }
    let mut num30 = d.scale * 1.4; // `:852`
    let rgb = match d.dust_type {
        // `:853-860` — WaterCandle: cap 1, (0.1, 0.4, 1).
        29 => {
            num30 = cap(num30, 1.0);
            [num30 * 0.1, num30 * 0.4, num30]
        }
        // `:861-875` — CursedTorch: cap 1, torch id 8. The `customData is
        // float` branch (`:869`) is dropped with the field (module docs);
        // the `else` (`:873`) always runs.
        75 => {
            num30 = cap(num30, 1.0);
            torch_rgb(8, num30, p)
        }
        // `:876-883` — IchorTorch: torch id 11.
        169 => {
            num30 = cap(num30, 1.0);
            torch_rgb(11, num30, p)
        }
        // `:884-891` — IceTorch: torch id 9.
        135 => {
            num30 = cap(num30, 1.0);
            torch_rgb(9, num30, p)
        }
        // `:892-899` — OrangeTorch: torch id 10.
        158 => {
            num30 = cap(num30, 1.0);
            torch_rgb(10, num30, p)
        }
        // `:900-907` — GoldFlame: (0.7, 0.65, 0.3).
        228 => {
            num30 = cap(num30, 1.0);
            [num30 * 0.7, num30 * 0.65, num30 * 0.3]
        }
        // `:908-915` — Vortex: (0.3, 0.65, 0.7).
        229 => {
            num30 = cap(num30, 1.0);
            [num30 * 0.3, num30 * 0.65, num30 * 0.7]
        }
        // `:916-923` — PinkTorch: torch id 15.
        242 => {
            num30 = cap(num30, 1.0);
            torch_rgb(15, num30, p)
        }
        // `:924-931` — DesertTorch: torch id 16, amount * 0.95.
        293 => {
            num30 = cap(num30, 1.0);
            torch_rgb(16, num30 * 0.95, p)
        }
        // `:932-939` — CoralTorch: torch id 17.
        294 => {
            num30 = cap(num30, 1.0);
            torch_rgb(17, num30, p)
        }
        // `:940-947` — the seven basic torches 59-65: cap 0.8 (not 1),
        // torch id `1 + type - 59`.
        59..=65 => {
            num30 = cap(num30, 0.8);
            torch_rgb(1 + d.dust_type - 59, num30, p)
        }
        // `:948-956` — Flare: *1.3 THEN cap 1, (1, 0.45, 0.2).
        127 => {
            num30 *= 1.3;
            num30 = cap(num30, 1.0);
            [num30, num30 * 0.45, num30 * 0.2]
        }
        // `:957-965` — BlueFlare: *1.3 THEN cap 1, (0.2, 0.45, 1).
        187 => {
            num30 *= 1.3;
            num30 = cap(num30, 1.0);
            [num30 * 0.2, num30 * 0.45, num30]
        }
        // `:966-973` — CorruptTorch: torch id 18.
        295 => {
            num30 = cap(num30, 1.0);
            torch_rgb(18, num30, p)
        }
        // `:974-981` — CrimsonTorch: torch id 19.
        296 => {
            num30 = cap(num30, 1.0);
            torch_rgb(19, num30, p)
        }
        // `:982-989` — HallowedTorch: torch id 20.
        297 => {
            num30 = cap(num30, 1.0);
            torch_rgb(20, num30, p)
        }
        // `:990-997` — JungleTorch: torch id 21.
        298 => {
            num30 = cap(num30, 1.0);
            torch_rgb(21, num30, p)
        }
        // `:998-1005` — MushroomTorch: torch id 22.
        307 => {
            num30 = cap(num30, 1.0);
            torch_rgb(22, num30, p)
        }
        // `:1006-1013` — ShimmerTorch: torch id 23 (animated, rides
        // `LightParams::mouse_text_color`).
        310 => {
            num30 = cap(num30, 1.0);
            torch_rgb(23, num30, p)
        }
        // `:1014-1021` — the else arm (Torch 6, SilverFlame 279, Terragrim
        // 302): cap 0.6, (1, 0.65, 0.4).
        _ => {
            num30 = cap(num30, 0.6);
            [num30, num30 * 0.65, num30 * 0.4]
        }
    };
    Some(rgb)
}

/// The `Dust.UpdateDust` light contribution of one dust record this tick,
/// or `None`. Pure: no RNG, no mutation (module docs). `p` resolves the
/// animated torch colours (demon 65, shimmer 310) exactly as the item
/// family's torch arm does.
pub(crate) fn dust_light(d: &Dust, p: &LightParams) -> Option<(i32, i32, [f32; 3])> {
    // The standalone arms of the FIRST ladder, in source order
    // (`Dust.cs:483-820`). Each type matches at most one arm, so the match
    // is the ladder's dispatch; the family chain (`:822`) and the Sandnado
    // arm (`:1037`) follow it in the C#'s else-if order.
    let rgb: [f32; 3] = match d.dust_type {
        // `:483-487` — ShimmerSpark: `color/255 * scale` per channel, the
        // `AddLight(Vector2, r, g, b)` overload (`Lighting.cs:140-143`).
        // The light reads the PRE-shrink scale (`:485` precedes `:486`'s
        // `scale *= 0.97f`, now the chain's): recover it from the POST-update
        // record (research/69 §5 retire, the 309 key).
        309 => {
            let pre = d.scale / 0.97;
            [
                d.color.r as f32 / 255.0 * pre,
                d.color.g as f32 / 255.0 * pre,
                d.color.b as f32 / 255.0 * pre,
            ]
        }
        // `:488-498` — PinkPhaseblade: gate both flags, `scale*0.6` cap 1,
        // quadratic (1.05, 0.1, 0.4).
        325 => {
            if d.no_light || d.no_light_emittance {
                return None;
            }
            let num4 = cap(d.scale * 0.6, 1.0);
            [
                num4 * (num4 * 1.05),
                num4 * (num4 * 0.1),
                num4 * (num4 * 0.4),
            ]
        }
        // `:513-557` — the gem dusts 86-92 + OrangeStainedGlass 286: gate
        // both flags, `scale*0.6` cap 1, then the `type - 85` switch. The
        // switch has no case 7 (GemDiamond is 91 = case 6; the unnamed 92
        // falls through) and no case 201 (286), so both keep the
        // unmultiplied white `num5 * num5`.
        86..=92 | 286 => {
            if d.no_light || d.no_light_emittance {
                return None;
            }
            let num5 = cap(d.scale * 0.6, 1.0);
            let (mut num7, mut num8, mut num9) = (num5, num5, num5);
            match d.dust_type - 85 {
                // `:526-530` — Sapphire (88): (0, 0.1, 1.3).
                3 => {
                    num7 *= 0.0;
                    num8 *= 0.1;
                    num9 *= 1.3;
                }
                // `:531-535` — Ruby (90): (1, 0.1, 0.1).
                5 => {
                    num7 *= 1.0;
                    num8 *= 0.1;
                    num9 *= 0.1;
                }
                // `:536-540` — Emerald (89): (0, 1, 0.1).
                4 => {
                    num7 *= 0.0;
                    num8 *= 1.0;
                    num9 *= 0.1;
                }
                // `:541-545` — Amethyst (86): (0.9, 0, 0.9).
                1 => {
                    num7 *= 0.9;
                    num8 *= 0.0;
                    num9 *= 0.9;
                }
                // `:546-550` — Diamond (91): (1.3, 1.3, 1.3).
                6 => {
                    num7 *= 1.3;
                    num8 *= 1.3;
                    num9 *= 1.3;
                }
                // `:551-555` — Topaz (87): (0.9, 0.9, 0).
                2 => {
                    num7 *= 0.9;
                    num8 *= 0.9;
                    num9 *= 0.0;
                }
                _ => {}
            }
            [num5 * num7, num5 * num8, num5 * num9]
        }
        // `:575-578` — AmberBolt: gate `!noLight` ONLY (no emittance gate),
        // `(0.9, 0.6, 0) * scale * 0.6`, the Vector2 overload.
        262 => {
            if d.no_light {
                return None;
            }
            [
                0.9 * d.scale * 0.6,
                0.6 * d.scale * 0.6,
                0.0 * d.scale * 0.6,
            ]
        }
        // `:612-617` — PortalBolt/Trail: gate `!noLight` only,
        // `color.ToVector3() * scale * 0.4`.
        263 | 264 => {
            if d.no_light {
                return None;
            }
            [
                d.color.r as f32 / 255.0 * d.scale * 0.4,
                d.color.g as f32 / 255.0 * d.scale * 0.4,
                d.color.b as f32 / 255.0 * d.scale * 0.4,
            ]
        }
        // `:630-640` — type 230: NO gate, NO cap; `num10 = scale*0.6`,
        // multipliers (0.5, 0.9, 1). The colour is computed BEFORE the
        // `scale += 0.02f` (`:639`, now the chain's): the POST-update record
        // carries the incremented scale, so recover the PRE-increment value
        // (research/69 §5 retire, the 230 key — d50's `:208-213` citation).
        230 => {
            let num10 = (d.scale - 0.02) * 0.6;
            [num10 * (num10 * 0.5), num10 * (num10 * 0.9), num10 * num10]
        }
        // `:656-664` — type 172: `scale*0.5` cap 1, (0, 0.25, 1).
        172 => {
            let num15 = cap(d.scale * 0.5, 1.0);
            [num15 * (num15 * 0.0), num15 * (num15 * 0.25), num15 * num15]
        }
        // `:665-676` — type 182: gate `!noLight` only, `scale*0.25` cap 1,
        // (1, 0.2, 0.1). The `rotation++` is S1-E-3's.
        182 => {
            if d.no_light {
                return None;
            }
            let num16 = cap(d.scale * 0.25, 1.0);
            [num16 * num16, num16 * (num16 * 0.2), num16 * (num16 * 0.1)]
        }
        // `:684-694` — AncientLight: gate both flags, `scale*0.3` cap 1,
        // `(0.4, 0.6, 0.7) * num17`, the Vector2 overload. The light reads
        // the PRE-drift scale (`:693` precedes the `:695-704` noGravity
        // grow + the −0.0025 shrink, now the chain's): undo the net drift
        // on the POST-update record (research/69 §5 retire, the 261 key).
        // The C# gates the grow on the PRE-arm fadeIn; the chain's ramp
        // (`:2101`) may have zeroed it since — a record standing at
        // `fadeIn == 0` after a ramp tick (scale crossed it) took the grow,
        // so the net is 0 there too; the residual boundary (a ramp tick
        // whose PRE fadeIn was non-zero but whose scale did NOT cross) is
        // indistinguishable on the record — documented in ANALYSIS.
        261 => {
            if d.no_light || d.no_light_emittance {
                return None;
            }
            let pre = if d.no_gravity && d.fade_in == 0.0 {
                d.scale
            } else {
                d.scale + 0.0025
            };
            let num17 = cap(pre * 0.3, 1.0);
            [0.4 * num17, 0.6 * num17, 0.7 * num17]
        }
        // `:711-719` — CrystalPulse: `scale*0.35` cap 1, (0.9, 0.1, 0.75).
        254 => {
            let num18 = cap(d.scale * 0.35, 1.0);
            [
                num18 * (num18 * 0.9),
                num18 * (num18 * 0.1),
                num18 * (num18 * 0.75),
            ]
        }
        // `:720-728` — CrystalPulse2: `scale*0.25` cap 1, same colour.
        255 => {
            let num19 = cap(d.scale * 0.25, 1.0);
            [
                num19 * (num19 * 0.9),
                num19 * (num19 * 0.1),
                num19 * (num19 * 0.75),
            ]
        }
        // `:737-757` — the minecart Sparks (213) and SparksMech (260):
        // `num20 = scale / 2.5 * 0.2`, the base vector /255 THEN cap num20
        // at 1 THEN multiply. 213 = (255, 217, 48), 260 = (255, 48, 48).
        // The `rotation = 0f` is S1-E-3's (the port's 213 arm already
        // rebuilds rotation, `minecart.rs`).
        213 | 260 => {
            let num20 = d.scale / 2.5 * 0.2;
            let base = if d.dust_type == 213 {
                [255.0f32, 217.0, 48.0]
            } else {
                [255.0f32, 48.0, 48.0]
            };
            let num20 = cap(num20, 1.0);
            [
                base[0] / 255.0 * num20,
                base[1] / 255.0 * num20,
                base[2] / 255.0 * num20,
            ]
        }
        // `:759-763` — Chlorophyte: `scale*0.2`, NO cap, (0.25, 1, 0.5).
        157 => {
            let num21 = d.scale * 0.2;
            [num21 * (num21 * 0.25), num21 * num21, num21 * (num21 * 0.5)]
        }
        // `:764-769` — type 206: the C# decrements `scale -= 0.1f` FIRST
        // (`:766`), then lights from the new scale. The decrement is the
        // chain's now (W4-d50 wiring): the POST-update record already
        // carries it — the former local compensation is RETIRED
        // (research/69 §5, the 206 key).
        206 => {
            let num22 = d.scale * 0.4;
            [num22 * (num22 * 0.1), num22 * (num22 * 0.6), num22 * num22]
        }
        // `:770-774` — type 163: `scale*0.25`, (0.25, 1, 0.05).
        163 => {
            let num23 = d.scale * 0.25;
            [
                num23 * (num23 * 0.25),
                num23 * num23,
                num23 * (num23 * 0.05),
            ]
        }
        // `:775-779` — type 205: `scale*0.25`, (1, 0.05, 1).
        205 => {
            let num24 = d.scale * 0.25;
            [num24 * num24, num24 * (num24 * 0.05), num24 * num24]
        }
        // `:780-784` — IchorSplash: `scale*0.5`, (1, 1, 0.05).
        170 => {
            let num25 = d.scale * 0.5;
            [num25 * num25, num25 * num25, num25 * (num25 * 0.05)]
        }
        // `:785-794` — UltraBrightTorch: torch id 12 at `scale*0.6`, NO
        // cap. `num27`/`num28` (`:789-792`) are computed and unused in the
        // C# — dead, not ported.
        156 => torch_rgb(12, d.scale * 0.6, p),
        // `:795-800` — BoneTorch: gate `!noLightEmittance` ONLY (no noLight
        // gate), torch id 13 at `scale*0.6`, NO cap.
        234 => {
            if d.no_light_emittance {
                return None;
            }
            torch_rgb(13, d.scale * 0.6, p)
        }
        // `:805-814` — InfernoFlame: `scale -= 0.01f` FIRST (`:807`), then
        // `num29 = scale*1` cap 0.6, (1, 0.4, 0). The decrement is the
        // chain's now (W4-d50 wiring): the POST-update record carries it —
        // the local compensation RETIRED (research/69 §5, the 174 key).
        174 => {
            let num29 = cap(d.scale, 0.6);
            [num29, num29 * 0.4, 0.0]
        }
        // `:822-1023` — the torch/fire family (its own else-if chain after
        // the rand-drawing 235 arm, which is skipped wholesale).
        228
        | 279
        | 229
        | 6
        | 242
        | 135
        | 127
        | 187
        | 75
        | 169
        | 29
        | 59..=65
        | 158
        | 293..=298
        | 302
        | 307
        | 310 => match torch_family_light(d, p) {
            Some(rgb) => rgb,
            None => return None,
        },
        // `:1037-1047` — Sandnado: gate `!noLight` only, `scale*1.4` cap 1,
        // `(0.7, 0.65, 0.3) * num31`, the Vector2 overload. The customData
        // velocity half (`:1048-1052`) is dropped with the field.
        269 => {
            if d.no_light {
                return None;
            }
            let num31 = cap(d.scale * 1.4, 1.0);
            [0.7 * num31, 0.65 * num31, 0.3 * num31]
        }
        _ => return None,
    };
    // Every arm anchors `(int)(position / 16f)` per axis — the Center form
    // with a zero box (the dust position IS the anchor; no width/height).
    Some((
        tile_x(d.position, 0.0, XAnchor::Center),
        tile_y(d.position, 0.0, XAnchor::Center),
        rgb,
    ))
}

/// The dust family's feed: walk the pool in `Main.dust` slot order
/// (`Dust.cs:431`) and append every active dust's contribution to the shared
/// [`EntityLights`] pair. Pure over the pool — no RNG parameter exists, so
/// the feed cannot move any stream (the witness test pins the pool update's
/// draws around it).
pub(crate) fn feed_dust_lights(pool: &DustPool, p: &LightParams, lights: &mut EntityLights) {
    for d in &pool.slots()[..DUST_SENTINEL] {
        if !d.active {
            continue;
        }
        if let Some((x, y, rgb)) = dust_light(d, p) {
            lights.add(x, y, rgb);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec2;
    use tr_core::UnifiedRandom;
    use tr_core::particles::{Color, UpdateCtx};

    fn params() -> LightParams {
        let mut p = LightParams::default();
        p.demon_torch = 1.0;
        p.disco = [255, 0, 0];
        p
    }

    fn dust(ty: i32, scale: f32, pos: Vec2) -> Dust {
        Dust {
            active: true,
            dust_type: ty,
            scale,
            position: pos,
            ..Default::default()
        }
    }

    #[test]
    fn a_plain_torch_dust_takes_the_else_arm_cap_06() {
        // `:1014-1021` — type 6 at scale 1: num30 = 1.4 -> cap 0.6,
        // (0.6, 0.39, 0.24). Position (160, 176) -> tile (10, 11).
        let (x, y, rgb) =
            dust_light(&dust(6, 1.0, Vec2::new(160.0, 176.0)), &params()).expect("torch 6 lights");
        assert_eq!((x, y), (10, 11));
        assert_eq!(rgb, [0.6, 0.6 * 0.65, 0.6 * 0.4]);
    }

    #[test]
    fn the_basic_torch_dusts_cap_at_08_and_ride_the_torch_table() {
        // `:940-947` — 59 (BlueTorch) -> torch id 1, cap 0.8:
        // TORCH_LIGHT[1] = [0, 0.1, 1.3] * 0.8.
        let p = params();
        let got = dust_light(&dust(59, 1.0, Vec2::ZERO), &p).unwrap().2;
        assert_eq!(got, torch_rgb(1, 0.8, &p));
        assert_eq!(got, [0.0, 0.1 * 0.8, 1.3 * 0.8]);
        // 65 (DemonTorch) -> torch id 7, the animated ramp.
        let mut p1 = params();
        p1.demon_torch = 1.0;
        assert_eq!(
            dust_light(&dust(65, 1.0, Vec2::ZERO), &p1).unwrap().2,
            [0.5 * 0.8, 0.3 * 0.8, 1.0 * 0.8]
        );
        p1.demon_torch = 0.0;
        assert_eq!(
            dust_light(&dust(65, 1.0, Vec2::ZERO), &p1).unwrap().2,
            [1.0 * 0.8, 0.3 * 0.8, 0.5 * 0.8]
        );
    }

    #[test]
    fn the_cursed_torch_dust_always_takes_the_no_custom_data_branch() {
        // `:861-875` — 75: cap 1, torch id 8 = [1.0, 1.6, 0.5]. The
        // `customData is float` branch is dropped with the field.
        let got = dust_light(&dust(75, 1.0, Vec2::ZERO), &params()).unwrap().2;
        assert_eq!(got, [1.0, 1.6, 0.5]);
        // Scale 0.5: num30 = 0.7, under the cap.
        let got = dust_light(&dust(75, 0.5, Vec2::ZERO), &params()).unwrap().2;
        assert_eq!(got, [0.7, 1.6 * 0.7, 0.5 * 0.7]);
    }

    #[test]
    fn the_flare_arms_multiply_13_before_capping() {
        // `:948-965` — 127/187: num30 = scale*1.4*1.3 THEN cap 1. Scale 0.6
        // gives 0.6*1.4 = 0.84, *1.3 = 1.092 -> cap 1.
        let p = params();
        assert_eq!(
            dust_light(&dust(127, 0.6, Vec2::ZERO), &p).unwrap().2,
            [1.0, 0.45, 0.2]
        );
        // Scale 0.5: 0.5*1.4*1.3 = 0.91, under the cap.
        let n = 0.5f32 * 1.4 * 1.3;
        assert_eq!(
            dust_light(&dust(187, 0.5, Vec2::ZERO), &p).unwrap().2,
            [n * 0.2, n * 0.45, n]
        );
    }

    #[test]
    fn the_family_gate_is_both_flags() {
        // `:850` — either flag kills the family light.
        let p = params();
        let mut d = dust(6, 1.0, Vec2::ZERO);
        d.no_light = true;
        assert!(dust_light(&d, &p).is_none());
        d.no_light = false;
        d.no_light_emittance = true;
        assert!(dust_light(&d, &p).is_none());
    }

    #[test]
    fn the_single_flag_gates_differ_per_arm() {
        let p = params();
        // 262 (AmberBolt, `:575`) gates `!noLight` only — emittance alone
        // does NOT mute it.
        let mut d = dust(262, 1.0, Vec2::ZERO);
        d.no_light_emittance = true;
        assert_eq!(dust_light(&d, &p).unwrap().2, [0.9 * 0.6, 0.6 * 0.6, 0.0]);
        d.no_light = true;
        assert!(dust_light(&d, &p).is_none());
        // 234 (BoneTorch, `:795`) gates `!noLightEmittance` only — noLight
        // alone does NOT mute it. Torch id 13 = [0.95, 0.75, 1.3], amount
        // scale*0.6 with NO cap.
        let mut d = dust(234, 2.0, Vec2::ZERO);
        d.no_light = true;
        assert_eq!(
            dust_light(&d, &p).unwrap().2,
            [0.95 * 1.2, 0.75 * 1.2, 1.3 * 1.2]
        );
        d.no_light_emittance = true;
        assert!(dust_light(&d, &p).is_none());
    }

    #[test]
    fn the_minecart_spark_lights_its_own_tile() {
        // `:737-757` — 213 at scale 2.5: num20 = 2.5/2.5*0.2 = 0.2,
        // (255, 217, 48)/255 * 0.2. This is the port's live dust type
        // (`minecart.rs::spawn_spark`).
        let got = dust_light(&dust(213, 2.5, Vec2::new(160.0, 176.0)), &params()).unwrap();
        assert_eq!((got.0, got.1), (10, 11));
        assert_eq!(
            got.2,
            [255.0 / 255.0 * 0.2, 217.0 / 255.0 * 0.2, 48.0 / 255.0 * 0.2]
        );
        // 260 (SparksMech) is the red variant.
        let n = 0.2f32;
        assert_eq!(
            dust_light(&dust(260, 2.5, Vec2::ZERO), &params())
                .unwrap()
                .2,
            [n, 48.0 / 255.0 * n, 48.0 / 255.0 * n]
        );
        // The cap: scale 25 -> num20 = 2 -> 1.
        assert_eq!(
            dust_light(&dust(213, 25.0, Vec2::ZERO), &params())
                .unwrap()
                .2,
            [1.0, 217.0 / 255.0, 48.0 / 255.0]
        );
    }

    #[test]
    fn negative_positions_truncate_the_quotient() {
        // `(int)(v / 16f)` truncates toward zero: -20/16 = -1.25 -> -1.
        let (x, y, _) = dust_light(&dust(213, 1.0, Vec2::new(-20.0, -20.0)), &params()).unwrap();
        assert_eq!((x, y), (-1, -1));
        // -4/16 = -0.25 -> 0 (the CenterTruncated form would differ only
        // after adding a half-box; a dust has none).
        let (x, y, _) = dust_light(&dust(6, 1.0, Vec2::new(-4.0, -4.0)), &params()).unwrap();
        assert_eq!((x, y), (0, 0));
    }

    #[test]
    fn the_gem_dust_switch_colours_and_the_default_white() {
        let p = params();
        // 88 (Sapphire, case 3): num5 = 0.6 at scale 1 -> (0, 0.036, 0.468).
        let n = 0.6f32;
        assert_eq!(
            dust_light(&dust(88, 1.0, Vec2::ZERO), &p).unwrap().2,
            [0.0, n * (n * 0.1), n * (n * 1.3)]
        );
        // 92 (case 7) and 286 (case 201) have NO switch case: white n*n.
        assert_eq!(
            dust_light(&dust(92, 1.0, Vec2::ZERO), &p).unwrap().2,
            [n * n, n * n, n * n]
        );
        assert_eq!(
            dust_light(&dust(286, 1.0, Vec2::ZERO), &p).unwrap().2,
            [n * n, n * n, n * n]
        );
    }

    #[test]
    fn the_uncapped_arms_stay_uncapped() {
        let p = params();
        // 230 (`:630-640`): no gate, no cap. POST-update record 2.02 (the
        // chain's +0.02 from 2.0) -> pre 2.0 -> num10 = 1.2 (the §5 retire).
        let n = (2.02f32 - 0.02) * 0.6;
        assert_eq!(
            dust_light(&dust(230, 2.02, Vec2::ZERO), &p).unwrap().2,
            [n * (n * 0.5), n * (n * 0.9), n * n]
        );
        // 156 (`:785-794`): torch 12 at scale*0.6, no cap. Scale 2 -> 1.2.
        assert_eq!(
            dust_light(&dust(156, 2.0, Vec2::ZERO), &p).unwrap().2,
            [0.75 * 1.2, 1.35 * 1.2, 1.5 * 1.2]
        );
        // 325 (`:488-498`) DOES cap: scale 2 -> num4 = 1.
        assert_eq!(
            dust_light(&dust(325, 2.0, Vec2::ZERO), &p).unwrap().2,
            [1.05, 0.1, 0.4]
        );
    }

    #[test]
    fn the_post_decrement_arms_read_the_decremented_scale() {
        // W4-d50 retire (research/69 §5): the records below are POST-update
        // (the chain already ran the decrement) — the feed reads them
        // directly, no local compensation.
        let p = params();
        // 206 (`:764-769`): the record's 0.9 IS the post-`-= 0.1` scale
        // (the C# arm ran from 1.0); num22 = 0.36.
        let n = 0.9f32 * 0.4;
        assert_eq!(
            dust_light(&dust(206, 0.9, Vec2::ZERO), &p).unwrap().2,
            [n * (n * 0.1), n * (n * 0.6), n * n]
        );
        // 174 (`:805-814`): the record's 0.99 IS post-`-= 0.01` -> cap 0.6.
        assert_eq!(
            dust_light(&dust(174, 0.99, Vec2::ZERO), &p).unwrap().2,
            [0.6, 0.6 * 0.4, 0.0]
        );
        // Record 0.29 (post-decrement from 0.3), under the cap.
        let n = 0.29f32;
        assert_eq!(
            dust_light(&dust(174, 0.29, Vec2::ZERO), &p).unwrap().2,
            [n, n * 0.4, 0.0]
        );
    }

    #[test]
    fn w4_d50_the_230_recovers_the_pre_increment_scale() {
        // `:630-640`: the C# light reads the scale BEFORE `+= 0.02` (`:639`).
        // The POST-update record 1.02 (the chain grew it from 1.0) recovers
        // num10 = 1.0 * 0.6 (research/69 §5, the 230 retire key).
        let n = (1.02f32 - 0.02) * 0.6;
        assert_eq!(
            dust_light(&dust(230, 1.02, Vec2::ZERO), &params()).unwrap().2,
            [n * (n * 0.5), n * (n * 0.9), n * n]
        );
    }

    #[test]
    fn w4_d50_the_261_recovers_the_pre_drift_scale() {
        // `:684-704`: the light precedes the noGravity grow (+0.0025 when
        // fadeIn == 0) and the −0.0025 shrink. POST-update records:
        // (a) noGravity + fadeIn 0: net drift 0 — read the record directly.
        let mut d = dust(261, 1.0, Vec2::ZERO);
        d.no_gravity = true;
        d.fade_in = 0.0;
        let n = 1.0f32 * 0.3;
        assert_eq!(
            dust_light(&d, &params()).unwrap().2,
            [0.4 * n, 0.6 * n, 0.7 * n]
        );
        // (b) gravity: the record lost 0.0025 — recover 0.9975 + 0.0025 = 1.
        let d = dust(261, 0.9975, Vec2::ZERO);
        let n = (0.9975f32 + 0.0025) * 0.3;
        assert_eq!(
            dust_light(&d, &params()).unwrap().2,
            [0.4 * n, 0.6 * n, 0.7 * n]
        );
        // (c) noGravity + live fadeIn: no grow, only the shrink — same undo.
        let mut d = dust(261, 0.9975, Vec2::ZERO);
        d.no_gravity = true;
        d.fade_in = 5.0;
        assert_eq!(
            dust_light(&d, &params()).unwrap().2,
            [0.4 * n, 0.6 * n, 0.7 * n]
        );
    }

    #[test]
    fn the_shimmer_spark_rides_its_own_colour() {
        // 309 (`:483-487`): color/255 * the PRE-shrink scale — the light
        // precedes `*= 0.97` (`:486`). POST-update record 0.485 (= 0.5 *
        // 0.97) recovers 0.5 (research/69 §5, the 309 retire key).
        let mut d = dust(309, 0.5 * 0.97, Vec2::ZERO);
        d.color = Color::new(200, 100, 50, 255);
        let pre = (0.5f32 * 0.97) / 0.97;
        assert_eq!(
            dust_light(&d, &params()).unwrap().2,
            [200.0 / 255.0 * pre, 100.0 / 255.0 * pre, 50.0 / 255.0 * pre]
        );
    }

    #[test]
    fn an_unported_type_is_dark() {
        // 268 (the flying sand) has no AddLight arm in the ported range;
        // 153 (the web puff) likewise. The second chain's types (e.g. 227)
        // wait for S1-E-3.
        let p = params();
        for ty in [268, 153, 227, 0, 16] {
            assert!(dust_light(&dust(ty, 1.0, Vec2::ZERO), &p).is_none());
        }
    }

    #[test]
    fn the_feed_walks_slot_order_and_skips_dead_slots() {
        let mut pool = DustPool::new();
        // Slot 2: a torch; slot 5: a spark; slot 3: dead.
        *pool.slot_mut(2) = dust(6, 1.0, Vec2::new(32.0, 32.0));
        *pool.slot_mut(5) = dust(213, 2.5, Vec2::new(64.0, 64.0));
        let mut dead = dust(6, 1.0, Vec2::ZERO);
        dead.active = false;
        *pool.slot_mut(3) = dead;
        let mut lights = EntityLights::default();
        feed_dust_lights(&pool, &params(), &mut lights);
        lights.end_frame();
        let vis = lights.visible();
        assert_eq!(vis.len(), 2);
        assert_eq!((vis[0].x, vis[0].y), (2, 2), "slot 2 first");
        assert_eq!((vis[1].x, vis[1].y), (4, 4), "slot 5 second");
        // The feed is deterministic: the same pool gives the same key.
        let mut again = EntityLights::default();
        feed_dust_lights(&pool, &params(), &mut again);
        again.end_frame();
        assert_eq!(again.key(), lights.key());
    }

    #[test]
    fn the_feed_leaves_the_update_stream_untouched() {
        // The RNG witness: two identical pools, two identical streams. Run
        // the pool update on both (the ChildSafety re-type rolls ride
        // `Main.rand`), feed the lights on ONE between the updates, and
        // compare the streams afterwards — the feed takes no RNG and draws
        // nothing, so every subsequent draw must match.
        let setup = || {
            let mut pool = DustPool::new();
            pool.child_safety.disabled = false; // arm the gate so the re-type rolls rand
            let mut d = dust(227, 1.0, Vec2::new(100.0, 100.0));
            d.first_frame = true; // the dangerous-type re-type rolls rand
            *pool.slot_mut(0) = d;
            *pool.slot_mut(1) = dust(213, 2.5, Vec2::new(100.0, 100.0));
            *pool.slot_mut(2) = dust(6, 1.0, Vec2::new(100.0, 100.0));
            pool
        };
        let ctx = UpdateCtx {
            max_dust_to_draw: 6000,
            screen_position: Vec2::ZERO,
            screen_height: 10000.0,
        };
        let mut a = setup();
        let mut b = setup();
        let mut ra = UnifiedRandom::new(42);
        let mut rb = UnifiedRandom::new(42);
        a.update_dust(&ctx, &mut ra, |_, _| false);
        b.update_dust(&ctx, &mut rb, |_, _| false);
        // The feed under test — only on A.
        let mut lights = EntityLights::default();
        feed_dust_lights(&a, &params(), &mut lights);
        // A second update (more potential rolls), then the raw streams.
        a.update_dust(&ctx, &mut ra, |_, _| false);
        b.update_dust(&ctx, &mut rb, |_, _| false);
        for _ in 0..32 {
            assert_eq!(ra.next_max(1_000_000), rb.next_max(1_000_000));
        }
        // And the feed found the live spark + torch (the 227 was re-typed
        // or killed by ChildSafety — either way it emits no ported light).
        lights.end_frame();
        assert!(
            lights
                .visible()
                .iter()
                .any(|l| l.rgb == [0.6, 0.6 * 0.65, 0.6 * 0.4])
        );
    }
}
