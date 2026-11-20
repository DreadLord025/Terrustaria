//! The NPC `Lighting.AddLight` sources, SECOND half (W4-e7, S1-E-5 wave 4).
//!
//! `NPC.cs` carries exactly 80 `Lighting.AddLight` call sites. The first 40
//! (`:21307`–`:48641`) ride `task/W4-c20-addlight-npcs-tiles`; this module
//! ports the SECOND 40 (`:49929`–`:92412`), verbatim, draw-side, feeding the
//! SAME [`crate::entity_lights::EntityLights`] pair as the aiStyle-8 tail
//! (`npc_lights.rs`) and every other family (one list pair per engine,
//! `LightingEngine.cs:36-38`, max-merged one frame late `:208-235`).
//!
//! ## The first slice, sites 41-50, in C# source order
//!
//! | # | site | arm | port |
//! |---|------|-----|------|
//! | 41 | `:49929` | Destroyer family airborne light | [`destroyer_light`] |
//! | 42 | `:50630` | Probe 139 tile-light probe | [`probe_light`] |
//! | 43 | `:52428` | The Hungry II 116 | cited skip (no kind — see below) |
//! | 44 | `:53232` | town shimmer state 25 | [`town_shimmer_light`] |
//! | 45 | `:54984` | town magic-attack aura | [`town_magic_light`] |
//! | 46 | `:55131` | town shimmer-arrival pose | [`town_arrival_light`] |
//! | 47 | `:55554` | TownSlimeRainbow 681 | [`town_slime_light`] |
//! | 48 | `:55559` | TownSlimeYellow 683/687 | [`town_slime_light`] |
//! | 49 | `:56420` | Medusa 480 charge-up | [`medusa_light`] |
//! | 50 | `:56427` | Medusa 480 charge hold | [`medusa_light`] |
//!
//! ## RNG census — why the feed is draw-side
//!
//! Every ported arm's `AddLight` line is itself RNG-free. The rand-bearing
//! halves are skipped WHOLESALE (the `npc_lights.rs` precedent): the
//! shimmer-state dust (`:53231`/`:53233`/`:53237`) and Medusa's charge dust
//! (`:56409-56417`, `:56442-56455`). Skipping an arm wholesale leaves the
//! `Main.rand` stream untouched, so the fixed-step order stays exact.
//!
//! ## netOffset conventions (verbatim per site)
//!
//! The C# adds `netOffset` only inside the arms that say so — Medusa
//! (`:56386`/`:56421`, `:56426`/`:56457`). The Destroyer (`AI_037` has ZERO
//! netOffset writes), the Probe's `:50630` (the arm's wraps are elsewhere),
//! the Hungry II and the town arms (`AI_007` has ZERO) read the RAW
//! position.
//!
//! ## Folds and cited skips (ANALYSIS "The NPC lights, second half")
//!
//! * The Hungry II 116 (`:52428`) has NO port kind: the roster's `Hungry`
//!   is TheHungry 115 (aiStyle 29, 30×30/240 HP/kb 1.1, `NPC.cs:10378-
//!   10392`; `world_sim/npcs.rs:6017` maps the WoF spawn's net 115 onto
//!   it) — the II is a SEPARATE 30×32/80 HP/aiStyle-2 kind
//!   (`NPC.cs:10393-10404`, `npc_ai_eye.rs`'s census: "116 … have no
//!   kinds yet"). Folding 116 onto `Hungry` would light the wrong NPC —
//!   the arm stays a cited skip until the kind lands (NO `cs_type` row:
//!   the `Hungry` fallback resolves to the bestiary's 115, which the
//!   `type == 116` gate can never match).
//! * The town shimmer state 25 is a cited skip in `npc_ai_town.rs`
//!   (`:53187-53277`); its light is ported here behind the same state
//!   shape, unreachable until that wiring. The `AddLight` line fires for
//!   EVERY state-25 tick that reaches it — the `ai[2] > 0` countdown
//!   returns first (`:53206-53214`), and below `ai[1] == 30` the
//!   `Utils.Remap` clamps to 0 (a zero-amount source still enters the
//!   C#'s per-frame list).
//! * The town magic-attack (14) / shimmer-arrival (24) arms are gated on
//!   the aiStyle-7 dispatch (`NPC.cs:20812-20816`): `ai[0]` states are
//!   per-machine, so a non-town machine's state 14 must never light.
//!   Only the five `MagicAuraColor` rows (54/108/160/20/663) carry a
//!   colour; every other type's aura is the set's `Color.White` default
//!   (`NPCID.cs:10475`), but only those five reach the attack arms
//!   (`NPCID.Sets.AttackType == 2`, `:10473`) and only the Dryad 20 the
//!   arrival pose (`HaveDryadDoStardewAnimation`, `:19184-19194`) — the
//!   `None` fold for unrowed types is unreachable, recorded here.

use tr_core::lighting::LightParams;
use tr_core::tile_light::hsl_to_rgb;
use tr_core::world::World;

use crate::entity_lights::{tile_x, tile_y, XAnchor};
use crate::npc::{Npc, NpcKind};

/// One `AddLight` source: tile x, tile y, RGB (the `EntityLights::add`
/// triple).
pub(crate) type Light = (i32, i32, [f32; 3]);

/// The C# `this.type` for the light arms: the hand table (`cs_id`) plus
/// the folded-se bestiary ids the light sites read — the Probe 139
/// (`NPCID.cs:10955`; the bestiary folds it onto the Destroyer head,
/// `npc_ai_style37.rs`'s `family_type` precedent), the Golem parts
/// 246/247/248 (`NPCID.cs`' GolemHead/Fists; the bestiary folds them onto
/// 245). NO Hungry row: the roster's `Hungry` is TheHungry 115 and the
/// 116 light arm is a cited skip (module docs).
fn cs_type(n: &Npc) -> i32 {
    match n.kind {
        NpcKind::Probe => 139,
        NpcKind::GolemHead => 246,
        NpcKind::GolemFistLeft => 247,
        NpcKind::GolemFistRight => 248,
        NpcKind::GolemHeadFree => 249,
        NpcKind::TheDestroyerBody => 135,
        NpcKind::TheDestroyerTail => 136,
        _ => n.kind.cs_id() as i32,
    }
}

/// `WorldGen.InWorld(x, y)` (`WorldGen.cs:9688-9695`, fluff 0).
fn in_world(world: &World, x: i32, y: i32) -> bool {
    x >= 0 && x < world.width && y >= 0 && y < world.height
}

/// `WorldGen.SolidTile(i, j)` (`WorldGen.cs:70111-70132`): active, solid,
/// not top-only, no half-brick, no slope, not inactive. Out of world the
/// C# array read throws into the catch → false (`:70128-70131`); the
/// null-tile arm (`:70115-70118`, true) is unreachable in the port
/// (`World::tile` yields a default inactive tile in bounds — the
/// `grapple.rs` precedent).
fn solid_tile(world: &World, x: i32, y: i32) -> bool {
    let Some(t) = world.tile(x, y) else {
        return false;
    };
    t.active()
        && tr_core::tiledata::TILE_SOLID.has(t.kind)
        && !tr_core::tiledata::TILE_SOLID_TOP.has(t.kind)
        && !t.half_brick()
        && t.slope() == 0
        && !t.inactive()
}

/// `Utils.GetLerpValue(from, to, t, clamped: true)` (`Utils.cs:315-343`)
/// — the clamped 0..1 ramp.
fn get_lerp_value_clamped(from: f32, to: f32, t: f32) -> f32 {
    if from < to {
        if t < from {
            return 0.0;
        }
        if t > to {
            return 1.0;
        }
    } else {
        if t < to {
            return 1.0;
        }
        if t > from {
            return 0.0;
        }
    }
    (t - from) / (to - from)
}

/// `Utils.Remap(fromValue, fromMin, fromMax, toMin, toMax)`
/// (`Utils.cs:345-348`, clamped default true) = `MathHelper.Lerp(toMin,
/// toMax, GetLerpValue(...))`.
fn remap(v: f32, from_min: f32, from_max: f32, to_min: f32, to_max: f32) -> f32 {
    let t = get_lerp_value_clamped(from_min, from_max, v);
    to_min + (to_max - to_min) * t
}

/// Site 41 — `NPC.AI_037_Destroyer`'s airborne light (`NPC.cs:49925-49930`):
/// while the family segment overlaps NO solid/liquid tile (`flag2`, the
/// `:49884-49924` scan), every type except a body in its `ai[2] == 1`
/// state lights the quotient-truncated centre (0.3, 0.1, 0.05). RAW
/// position — `AI_037` never adds `netOffset`. The draw-side scan below
/// is the verbatim twin of `npc_ai_style37.rs::overlaps_solid_or_liquid`
/// (kept here so the feed does not reach into the sim module's private
/// fn).
pub(crate) fn destroyer_light(n: &Npc, world: &World) -> Option<Light> {
    let t = cs_type(n);
    if !(134..=136).contains(&t) {
        return None;
    }
    if destroyer_overlaps_solid_or_liquid(world, n) {
        return None; // `:49925` — `if (!flag2)`
    }
    if t == 135 && n.ai[2] == 1.0 {
        return None; // `:49927`
    }
    let (w, h) = n.size();
    Some((
        tile_x_pos(n.x, w),
        tile_y_pos(n.y, h),
        [0.3, 0.1, 0.05],
    ))
}

/// The `:49884-49924` tile-overlap scan (`flag2`), verbatim (the
/// `npc_ai_style37.rs::overlaps_solid_or_liquid` twin).
fn destroyer_overlaps_solid_or_liquid(world: &World, n: &Npc) -> bool {
    let (w, h) = n.size();
    let mut x0 = (n.x / 16.0).trunc() as i32 - 1;
    let mut x1 = ((n.x + w) / 16.0).trunc() as i32 + 2;
    let mut y0 = (n.y / 16.0).trunc() as i32 - 1;
    let mut y1 = ((n.y + h) / 16.0).trunc() as i32 + 2;
    if x0 < 0 {
        x0 = 0;
    }
    if x1 > world.width {
        x1 = world.width;
    }
    if y0 < 0 {
        y0 = 0;
    }
    if y1 > world.height {
        y1 = world.height;
    }
    for k in x0..x1 {
        for l in y0..y1 {
            let Some(t) = world.tile(k, l) else {
                continue;
            };
            // `:49912` — NO `inActive()` test here (unlike Collision's
            // solid gates); the Living Trees 192 exclusion is the port's
            // table convention (`WorldGen.cs:15715`, collision.rs).
            let solid = t.active()
                && ((tr_core::tiledata::TILE_SOLID.has(t.kind) && t.kind != 192)
                    || (tr_core::tiledata::TILE_SOLID_TOP.has(t.kind) && t.frame_y == 0));
            if solid || t.liquid > 64 {
                let tx = (k * 16) as f32;
                let ty = (l * 16) as f32;
                if n.x + w > tx && n.x < tx + 16.0 && n.y + h > ty && n.y < ty + 16.0 {
                    return true;
                }
            }
        }
    }
    false
}

/// Site 42 — the Probe 139's tile-light probe (`NPC.cs:50624-50631`,
/// inside `AI_005_EaterOfSouls`' type-139 arm): the SNAPPED centre
/// (`(int)position.X + width / 2`, INTEGER half — `:50624-50627`) must be
/// in-world and non-solid; the light itself uses the QUOTIENT-truncated
/// float centre (`:50630`). RAW position (the arm's netOffset wraps are
/// at `:50689`/`:51007`, elsewhere).
pub(crate) fn probe_light(n: &Npc, world: &World) -> Option<Light> {
    if cs_type(n) != 139 {
        return None;
    }
    let (w, h) = n.size();
    let mut num12 = n.x as i32 + w as i32 / 2;
    let mut num13 = n.y as i32 + h as i32 / 2;
    num12 /= 16;
    num13 /= 16;
    if !(in_world(world, num12, num13) && !solid_tile(world, num12, num13)) {
        return None;
    }
    Some((tile_x_pos(n.x, w), tile_y_pos(n.y, h), [0.3, 0.1, 0.05]))
}

/// Site 44 — the town shimmer state 25's hue-cycling light
/// (`NPC.cs:53187-53232`): every state-25 tick that reaches the line
/// lights `hslToRgb(timeForVisualEffects / 360 % 1, 0.6, 0.65) *
/// Remap(ai[1], 30, 90, 0, 0.7)` — the `ai[2] > 0` countdown returns
/// first (`:53206-53214`), and below `ai[1] == 30` the Remap clamps to a
/// ZERO amount (the source still enters the C#'s list). The `Main.rand`
/// draws around it (`:53231`/`:53233`/`:53237`) are the wholesale skip.
/// The state itself is a cited skip in `npc_ai_town.rs` — this arm is
/// unreachable until that wiring lands (the 75-lantern precedent). RAW
/// position (`AI_007` has no netOffset).
pub(crate) fn town_shimmer_light(n: &Npc, p: &LightParams) -> Option<Light> {
    if n.kind.ai_style() != 7 || n.ai[0] != 25.0 || n.ai[2] > 0.0 {
        return None;
    }
    let hue = (p.time_for_visuals / 360.0) % 1.0;
    let rgb = hsl_to_rgb(hue, 0.6, 0.65);
    let amount = remap(n.ai[1], 30.0, 90.0, 0.0, 0.7);
    let (w, h) = n.size();
    Some((
        tile_x(n_pos(n), w, XAnchor::Center),
        tile_y(n_pos(n), h, XAnchor::Center),
        [rgb[0] * amount, rgb[1] * amount, rgb[2] * amount],
    ))
}

/// `NPC.GetMagicAuraColor` (`NPC.cs:52916-52932`) + the
/// `NPCID.Sets.MagicAuraColor` rows (`NPCID.cs:10475`): the per-type
/// colour bytes /255 (XNA `Color.ToVector3`), the variation-1 swaps for
/// the Truffle (BGR), the Dryad (90, 20, 210) and the Princess
/// (255, 0, 77). The set's default is `Color.White`; the princess's base
/// is `Main.hslToRgb(0.92, 1, 0.78)` (byte-quantised, alpha 127 dropped
/// by ToVector3). Only the five rowed types reach the aura arms (module
/// docs) — the unrowed default stays `None`, unreachable.
fn magic_aura_color(t: i32, variation: i32) -> Option<[f32; 3]> {
    let base: [u8; 3] = match t {
        54 => [100, 4, 227],
        108 => [255, 80, 60],
        160 => [40, 80, 255],
        20 => [40, 255, 80],
        663 => {
            let c = hsl_to_rgb(0.92, 1.0, 0.78);
            [(c[0] * 255.0) as u8, (c[1] * 255.0) as u8, (c[2] * 255.0) as u8]
        }
        _ => return None,
    };
    let rgb = if variation == 1 {
        match t {
            160 => [base[2], base[1], base[0]],
            20 => [90, 20, 210],
            663 => [255, 0, 77],
            _ => base,
        }
    } else {
        base
    };
    Some([
        rgb[0] as f32 / 255.0,
        rgb[1] as f32 / 255.0,
        rgb[2] as f32 / 255.0,
    ])
}

/// Site 45 — the town magic attack (state 14)'s aura light
/// (`NPC.cs:54981-54985`): `GetMagicAuraColor().ToVector3() * num65`
/// where `num65 = 1f` (`:54838`, never rewritten in the arm). The
/// aiStyle-7 gate is the `AI()` dispatch (`:20812-20816`). RAW position.
pub(crate) fn town_magic_light(n: &Npc) -> Option<Light> {
    if n.kind.ai_style() != 7 || n.ai[0] != 14.0 {
        return None;
    }
    let rgb = magic_aura_color(cs_type(n), n.variation_index)?;
    let (w, h) = n.size();
    Some((
        tile_x(n_pos(n), w, XAnchor::Center),
        tile_y(n_pos(n), h, XAnchor::Center),
        rgb,
    ))
}

/// Site 46 — the town shimmer-arrival pose (state 24)'s aura light
/// (`NPC.cs:55123-55131`): `GetMagicAuraColor().ToVector3()` at full
/// amount, every tick of the pose. The aiStyle-7 gate as site 45. RAW
/// position.
pub(crate) fn town_arrival_light(n: &Npc) -> Option<Light> {
    if n.kind.ai_style() != 7 || n.ai[0] != 24.0 {
        return None;
    }
    let rgb = magic_aura_color(cs_type(n), n.variation_index)?;
    let (w, h) = n.size();
    Some((
        tile_x(n_pos(n), w, XAnchor::Center),
        tile_y(n_pos(n), h, XAnchor::Center),
        rgb,
    ))
}

/// Sites 47/48 — the town slimes' tail lights (`NPC.cs:55547-55560`,
/// `AI_007_TownEntities`' unconditional tail): 681 (TownSlimeRainbow)
/// lights `TorchID.TorchColor(23) * 0.35` (the shimmer torch, animated
/// by `mouseTextColor`); 683/687 (TownSlimeYellow + its bound form)
/// pulse `(0.25, 0.25, 0.1) * WrappedLerp(0.75, 1, timeForVisualEffects
/// % 120 / 120)`. The type gates are the C#'s own (all three types are
/// aiStyle 7, `NPC.cs:17594-17600`/`:17640-17644` — the tail only runs
/// inside `AI_007`). RAW position. No port kind yet (the town-slime
/// roster wave) — the arms are ported whole, unreachable until it lands.
pub(crate) fn town_slime_light(n: &Npc, p: &LightParams) -> Option<Light> {
    let t = cs_type(n);
    let rgb = if t == 681 {
        let c = tr_core::tile_light::torch_color(23, p);
        [c[0] * 0.35, c[1] * 0.35, c[2] * 0.35]
    } else if t == 683 || t == 687 {
        // `Utils.WrappedLerp` (`Utils.cs:305-313`): `num = percent * 2;
        // if (num > 1) num = 2 - num; Lerp(v1, v2, num)`.
        let mut num = ((p.time_for_visuals % 120.0) / 120.0) * 2.0;
        if num > 1.0 {
            num = 2.0 - num;
        }
        let num132 = 0.75 + (1.0 - 0.75) * num;
        [0.25 * num132, 0.25 * num132, 0.1 * num132]
    } else {
        return None;
    };
    let (w, h) = n.size();
    Some((
        tile_x(n_pos(n), w, XAnchor::Center),
        tile_y(n_pos(n), h, XAnchor::Center),
        rgb,
    ))
}

/// Sites 49/50 — Medusa 480's charge lights (`NPC.cs:56356-56457`,
/// `AI_003_Fighters`' type-480 arm): while `ai[2] < 0` (the wind-up
/// countdown), BOTH the deep wind-up (`ai[2] < -num22`, `:56384-56422`)
/// and the hold (`-num22 <= ai[2] < 0`, `:56424-56457`) light
/// (0.9, 0.75, 0.1) at the centre — `num22` is 180, or 240 below a third
/// life (`:56360-56369`; both arms share the colour so the boundary is
/// not re-derived here, the two `ai[2] < 0` arms cover the span). The
/// C# adds `netOffset` around both arms (`:56386`/`:56421`,
/// `:56426`/`:56457`) — the feed reads the offset position. The
/// charge-up dust (`:56405-56418`, three `Main.rand` draws) is the
/// wholesale skip; the `num25 == 1` sound (`:56401-56404`) is the audio
/// task's. No port kind yet — ported whole, unreachable until it lands.
pub(crate) fn medusa_light(n: &Npc) -> Option<Light> {
    if cs_type(n) != 480 || n.ai[2] >= 0.0 {
        return None;
    }
    let (w, h) = n.size();
    let pos = n_pos(n) + n.net_offset;
    Some((
        tile_x(pos, w, XAnchor::Center),
        tile_y(pos, h, XAnchor::Center),
        [0.9, 0.75, 0.1],
    ))
}

/// The raw top-left as a `Vec2` for the shared anchor helpers.
fn n_pos(n: &Npc) -> glam::Vec2 {
    glam::Vec2::new(n.x, n.y)
}

/// The quotient-truncated centre tile x (`(int)((position.X + width/2)
/// / 16f)` — the common AI form).
fn tile_x_pos(x: f32, w: f32) -> i32 {
    ((x + w / 2.0) / 16.0) as i32
}

/// The quotient-truncated centre tile y.
fn tile_y_pos(y: f32, h: f32) -> i32 {
    ((y + h / 2.0) / 16.0) as i32
}

#[cfg(test)]
mod tests {
    use super::*;
    use tr_core::tile::{Tile, FLAG_ACTIVE};

    fn air() -> World {
        World::new("t".into(), 64, 64)
    }

    fn solid(w: &mut World, x: i32, y: i32) {
        let i = w.index(x, y);
        w.tiles[i] = Tile { kind: 1, ..Default::default() };
        w.tiles[i].flags |= FLAG_ACTIVE;
    }

    #[test]
    fn destroyer_head_lights_while_airborne_and_goes_dark_over_ground() {
        // `:49925-49929` — the head (134, 48×48) at (160, 160) in an air
        // world: centre (184, 184) → tile (11, 11), (0.3, 0.1, 0.05).
        let mut n = Npc::new(NpcKind::TheDestroyer, 160.0, 160.0, 1, 0.0);
        let w = air();
        let l = destroyer_light(&n, &w).expect("airborne head lights");
        assert_eq!(l, (11, 11, [0.3, 0.1, 0.05]));
        // A solid tile INSIDE the box flips flag2 → dark (tile (11,11)
        // spans 176..192, overlapping the 160..208 box).
        let mut w2 = air();
        solid(&mut w2, 11, 11);
        assert!(destroyer_light(&n, &w2).is_none());
        // netOffset is NOT added by AI_037 — the light stays put.
        n.net_offset = glam::Vec2::new(32.0, 0.0);
        assert_eq!(destroyer_light(&n, &w).unwrap().0, 11);
    }

    #[test]
    fn destroyer_body_ai2_one_stays_dark_but_the_tail_does_not() {
        // `:49927` — only the BODY (135) skips in its ai[2] == 1 state.
        let mut body = Npc::new(NpcKind::TheDestroyerBody, 160.0, 160.0, 1, 0.0);
        let w = air();
        body.ai[2] = 1.0;
        assert!(destroyer_light(&body, &w).is_none());
        body.ai[2] = 0.0;
        assert!(destroyer_light(&body, &w).is_some());
        let mut tail = Npc::new(NpcKind::TheDestroyerTail, 160.0, 160.0, 1, 0.0);
        tail.ai[2] = 1.0;
        assert!(destroyer_light(&tail, &w).is_some());
    }

    #[test]
    fn probe_lights_only_when_the_snapped_centre_is_open() {
        // `:50624-50630` — the Probe's 30×30 box at (160, 160): the GATE
        // uses the snapped integer centre (160+15)/16 = 10, the LIGHT the
        // float quotient centre 175/16 = 10.9 → 11. Open gate tile → light
        // at (11, 11); solid gate tile → dark.
        let n = Npc::new(NpcKind::Probe, 160.0, 160.0, 1, 0.0);
        let mut w = air();
        assert_eq!(probe_light(&n, &w).unwrap(), (11, 11, [0.3, 0.1, 0.05]));
        solid(&mut w, 10, 10);
        assert!(probe_light(&n, &w).is_none());
    }

    #[test]
    fn the_hungry_115_never_lights_the_116_arm() {
        // The roster's Hungry is TheHungry 115 — the `type == 116` arm
        // (`:52425-52428`) is a cited skip; cs_type resolves to 115.
        let n = Npc::new(NpcKind::Hungry, 160.0, 160.0, 1, 0.0);
        assert_eq!(cs_type(&n), 115);
    }

    #[test]
    fn town_shimmer_emits_zero_amount_below_30_and_ramps_after() {
        // `:53232` fires for EVERY reached state-25 tick: ai[1] = 10 →
        // Remap clamps to 0 (the source still enters the list); the
        // ai[2] > 0 countdown returns first (`:53206-53214`).
        let mut n = Npc::new(NpcKind::Guide, 160.0, 160.0, 1, 0.0);
        let mut p = LightParams::default();
        p.time_for_visuals = 180.0; // hue 0.5
        n.ai[0] = 25.0;
        n.ai[1] = 10.0;
        let l = town_shimmer_light(&n, &p).expect("state 25 always emits");
        assert_eq!(l.2, [0.0, 0.0, 0.0]);
        n.ai[1] = 60.0; // Remap(60, 30, 90, 0, 0.7) = 0.35
        let l = town_shimmer_light(&n, &p).unwrap();
        let rgb = hsl_to_rgb(0.5, 0.6, 0.65);
        assert_eq!(l.2, [rgb[0] * 0.35, rgb[1] * 0.35, rgb[2] * 0.35]);
        n.ai[2] = 5.0; // the countdown return
        assert!(town_shimmer_light(&n, &p).is_none());
    }

    #[test]
    fn town_magic_and_arrival_aura_are_town_only() {
        // The Dryad (20) base aura (40, 255, 80)/255; variation 1 swaps
        // to (90, 20, 210)/255 (`:52923-52926`).
        let mut n = Npc::new(NpcKind::Dryad, 160.0, 160.0, 1, 0.0);
        n.ai[0] = 14.0;
        let l = town_magic_light(&n).expect("attack aura");
        assert_eq!(l.2, [40.0 / 255.0, 1.0, 80.0 / 255.0]);
        n.variation_index = 1;
        assert_eq!(
            town_magic_light(&n).unwrap().2,
            [90.0 / 255.0, 20.0 / 255.0, 210.0 / 255.0]
        );
        n.ai[0] = 24.0;
        assert!(town_arrival_light(&n).is_some());
        // A non-town machine's state 14 never lights (the aiStyle-7
        // dispatch gate, `:20812-20816`).
        let mut bat = Npc::new(NpcKind::CaveBat, 160.0, 160.0, 1, 0.0);
        bat.ai[0] = 14.0;
        assert!(town_magic_light(&bat).is_none());
    }

    #[test]
    fn medusa_charge_lights_the_offset_position() {
        // No Medusa kind yet — the arm is ported whole behind the type
        // gate; the netOffset convention is pinned through the helper.
        let mut n = Npc::new(NpcKind::Zombie, 160.0, 160.0, 1, 0.0);
        n.ai[2] = -5.0;
        assert!(medusa_light(&n).is_none(), "no 480 kind yet");
        // The anchor math the arm uses: offset centre, quotient form.
        let pos = n_pos(&n) + glam::Vec2::new(16.0, 0.0);
        assert_eq!(tile_x(pos, 18.0, XAnchor::Center), 11);
    }
}
