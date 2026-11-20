//! The NPC `Lighting.AddLight` sources, FIRST half (`S1-E-5`, W4-c20).
//!
//! C# `Lighting.AddLight` funnels every overload into
//! `Lighting.AddLight(int i, int j, float r, float g, float b)`
//! (`Lighting.cs:157-163`), which appends to the active engine's per-frame
//! list behind `!Main.gamePaused && Main.netMode != 2`
//! (`LightingEngine.cs:51-54`); the engine max-merges it one frame late
//! (`LightingEngine.cs:208-235`). ONE list pair per engine —
//! [`crate::entity_lights::EntityLights`] — shared with the item/player/
//! projectile/dust families. This module ports the FIRST 40 of NPC.cs's
//! 80 `AddLight` sites (`:21307`-`:48641`); the second 40
//! (`:49929`-`:92412`) are W4-e7's `npc_lights_b.rs`.
//!
//! Three dispositions per site (the per-site table: ANALYSIS "The NPC
//! lights, first half"):
//!
//! * **FED** — the kind is in the roster and the light's gate is readable
//!   from the post-step state: [`npc_light_a`] returns it and the draw
//!   feed (`app/frame/draw.rs`) appends it in `Main.npc` slot order.
//! * **DEDUP** — the site is already ported AND fed elsewhere; this
//!   module only records it: `:21360`/`:21365` ride
//!   `npc_lights.rs::npc_light` (the aiStyle-8 533/else tail), `:24664`
//!   rides `npc_ai_style21::blazing_wheel_light`, `:35092` rides the
//!   ForceBubble inline arm — all three already live in the draw feed.
//! * **TWIN** — the site's type has no `NpcKind` yet (or its AI machine
//!   is unwired, or the light's value rides a `Main.rand` roll the
//!   unported machine owns): the site is ported VERBATIM as a pure fn
//!   with witness tests, and the dispatcher returns `None` until the
//!   kind/machine wave lands (the `npc_lights.rs` aiStyle-23 precedent).
//!   The firefly/snail twins (`:34162-34170`, `:34387`/`:34391`) dedup
//!   against `npc_ai_style64.rs`'s `Ai64Effects::light` /
//!   `npc_ai_style67.rs`'s `SnailEvents::light` at THEIR wiring — one
//!   feed per site, never two. The Tesla Turret twins
//!   (`:35167`/`:35175`) dedup against `npc_ai_style73.rs`'s
//!   `Style73Outcome::light` the same way (the machine owns the `ai[1]`
//!   cadence; the turret's kind arm is still `{}` in `update_kind_ai`).
//!
//! No FED arm draws `Main.rand`, so the feed's placement after the step
//! leaves every stream untouched (the `npc_lights.rs` rule). The one
//! rand-bearing site in this half (`:34258`, the Enchanted Nightcrawler's
//! `Next(90, 111)` flicker) is a TWIN taking the rolled factor — its
//! draw rides the future aiStyle-66 machine on the `UpdateNPCs` stream
//! (the `item_lights.rs` rand-head precedent).
//!
//! Anchor forms are NOT interchangeable: `(int)(v/16f)` truncates the
//! quotient, `(int)v / 16` truncates first, and `(float)(width / 2)` is
//! an INTEGER half (odd boxes lose the 0.5). Each site keeps its own
//! form verbatim; the shared vocabulary is
//! [`crate::entity_lights::{tile_x, tile_y, XAnchor}`].
#![allow(dead_code)] // the TWIN fns are census ports; their feeds land with the kind/machine waves

use glam::Vec2;
use tr_core::World;

use crate::entity_lights::{tile_x, tile_y, XAnchor};
use crate::npc::{Npc, NpcKind};

/// The C#'s `(float)(width / 2)` — INTEGER division before the float
/// cast (`NPC.cs`'s int `Entity.width`). Odd boxes lose the 0.5; the
/// float-half `XAnchor::Center` form is a DIFFERENT anchor.
#[inline]
fn int_half(v: f32) -> f32 {
    (v as i32 / 2) as f32
}

/// `Utils.Remap(fromValue, fromMin, fromMax, toMin, toMax)`
/// (`Utils.cs:345-348`, clamped = true): `MathHelper.Lerp(toMin, toMax,
/// GetLerpValue(fromMin, fromMax, fromValue, clamped: true))`.
fn remap_clamped(value: f32, from_min: f32, from_max: f32, to_min: f32, to_max: f32) -> f32 {
    let t = if from_min < from_max {
        value.clamp(from_min, from_max)
    } else {
        // The descending branch clamps the OTHER way (`Utils.cs:330-340`).
        if value < from_max {
            from_max
        } else if value > from_min {
            from_min
        } else {
            value
        }
    };
    let f = (t - from_min) / (from_max - from_min);
    to_min + (to_max - to_min) * f
}

// ---------------------------------------------------------------------
// Site 1 — `:21307`, the aiStyle-8 tail's type-693 arm (LibrarianSkeleton,
// `NPCID.cs:12063`). TWIN: no NpcKind yet.
// ---------------------------------------------------------------------

/// `NPC.cs:21301-21309`: `num111 = Utils.Remap(ai[0], 0, 60, 0, 1)`,
/// `num111 *= num111`, and only `num111 > 0` lights `base.Top` with
/// `(num111*0.91, num111*0.82, num111*0.57)`. `base.Top` =
/// `(x + width/2f, y)` (`Entity.cs:86-91`), the Vector2 overload
/// truncating the quotient per axis (`Lighting.cs:140-143`).
pub(crate) fn librarian_skeleton_light(
    ai0: f32,
    pos: Vec2,
    w: f32,
    h: f32,
) -> Option<(i32, i32, [f32; 3])> {
    let mut num111 = remap_clamped(ai0, 0.0, 60.0, 0.0, 1.0);
    num111 *= num111;
    if num111 > 0.0 {
        Some((
            tile_x(pos, w, XAnchor::Top),
            tile_y(pos, h, XAnchor::Top),
            [num111 * 0.91, num111 * 0.82, num111 * 0.57],
        ))
    } else {
        None
    }
}

// Sites 2/3 — `:21360` (Desert Djinn 533, Top, 0.6/0.6/0.3) and `:21365`
// (the else arm's Fire Imp 24, Center, 0.6/0.39/0.24): DEDUP — already
// ported AND fed by `npc_lights.rs::npc_light` (the aiStyle-8 tail).

// ---------------------------------------------------------------------
// Site 4 — `:21561`, the aiStyle-9 dust loop's else arm. FED
// (BurningSphere 25 is the roster's only else-arm kind: 30/665, 33 and
// 112/666 take the dust arms `:21519`/`:21531`/`:21553`, and 516 returns
// at `:21514` before the loop).
// ---------------------------------------------------------------------

/// `NPC.cs:21559-21566`: `AddLight((int)((x + (float)(width/2)) / 16f),
/// (int)((y + (float)(height/2)) / 16f), 1f, 0.3f, 0.1f)` — the INTEGER
/// half, quotient truncation. `pos` is the NPC's top-left INCLUDING
/// `netOffset` (the loop brackets it, `:21516`/`:21569`). The light sits
/// INSIDE the two-iteration dust loop (`:21517`), so the C# appends it
/// TWICE per tick — [`npc_light_a_emits`] carries that count.
pub(crate) fn burning_sphere_light(pos: Vec2, w: f32, h: f32) -> (i32, i32, [f32; 3]) {
    (
        ((pos.x + int_half(w)) / 16.0) as i32,
        ((pos.y + int_half(h)) / 16.0) as i32,
        [1.0, 0.3, 0.1],
    )
}

/// The `:21517` loop count for the FED sites: the BurningSphere's else
/// arm fires once per iteration of the 2-iteration dust loop; every
/// other site fires once. The duplicate append is exact — the C# list
/// carries both entries (the max-merge folds them, but the list pair's
/// cache key hashes every entry).
pub(crate) fn npc_light_a_emits(kind: NpcKind) -> u32 {
    match kind {
        NpcKind::BurningSphere => 2,
        _ => 1,
    }
}

// ---------------------------------------------------------------------
// Site 5 — `:23764`, the aiStyle-16 arm's type-102 light (AnglerFish,
// `NPCID.cs:10881`). TWIN: no NpcKind yet.
// ---------------------------------------------------------------------

/// `NPC.cs:23762-23765`: `AddLight((int)(x + (float)(width/2) +
/// (float)(direction * (width + 8))) / 16, (int)(y + 2f) / 16,
/// 0.07f, 0.04f, 0.025f)` — the light sits a full box + 8 px AHEAD of
/// the centre, at the top edge + 2; integer half, cast-before-divide.
pub(crate) fn angler_fish_light(pos: Vec2, w: f32, h: f32, direction: i32) -> (i32, i32, [f32; 3]) {
    let _ = h;
    (
        (pos.x + int_half(w) + (direction * (w as i32 + 8)) as f32) as i32 / 16,
        (pos.y + 2.0) as i32 / 16,
        [0.07, 0.04, 0.025],
    )
}

// ---------------------------------------------------------------------
// Sites 6/7/8 — `:24211`/`:24215`/`:24219`, the aiStyle-18 jellyfish
// chain (63 BlueJellyfish / 103 GreenJellyfish / the else pink; 221
// Squid and 242 BloodJelly are excluded). TWIN: no NpcKind yet.
// ---------------------------------------------------------------------

/// `NPC.cs:24204-24220`: `num263 = 1f`, `+0.5f` while `flag26`
/// (`base.wet && ai[1] == 1f`, `:24156`). All three arms read
/// `(int)(x + (float)(HEIGHT/2)) / 16` for the X tile — the HEIGHT
/// half on the X axis is the C#'s own quirk, kept verbatim — and
/// `(int)(y + (float)(height/2)) / 16` for Y; integer halves,
/// cast-before-divide. 63 → (0.05, 0.15, 0.4)·num263, 103 →
/// (0.05, 0.45, 0.1)·num263, the else (not 221/242) →
/// (0.35, 0.05, 0.2)·num263.
pub(crate) fn jellyfish_light(
    vanilla_type: i32,
    pos: Vec2,
    w: f32,
    h: f32,
    wet: bool,
    ai1: f32,
) -> Option<(i32, i32, [f32; 3])> {
    let _ = w;
    let mut num263 = 1.0f32;
    if wet && ai1 == 1.0 {
        num263 += 0.5;
    }
    let rgb = if vanilla_type == 63 {
        [0.05 * num263, 0.15 * num263, 0.4 * num263]
    } else if vanilla_type == 103 {
        [0.05 * num263, 0.45 * num263, 0.1 * num263]
    } else if vanilla_type != 221 && vanilla_type != 242 {
        [0.35 * num263, 0.05 * num263, 0.2 * num263]
    } else {
        return None;
    };
    Some((
        (pos.x + int_half(h)) as i32 / 16,
        (pos.y + int_half(h)) as i32 / 16,
        rgb,
    ))
}

// Site 9 — `:24664` (BlazingWheel 72, the mouseTextColor-pulsed centre
// light): DEDUP — already ported AND fed by
// `npc_ai_style21::blazing_wheel_light` (the draw feed's small-kinds
// arm reads `App.mouse_text_color`).

// ---------------------------------------------------------------------
// Site 10 — `:24801`, the aiStyle-22 arm's type-122 light (Gastropod,
// `NPCID.cs:10921`). TWIN: no NpcKind yet.
// ---------------------------------------------------------------------

/// `NPC.cs:24796-24803`: `AddLight((int)Center.X / 16, (int)Center.Y /
/// 16, 0.4f, 0f, 0.25f)` inside the type-122 `netOffset` bracket —
/// cast-before-divide ([`XAnchor::CenterTruncated`]); `pos` includes
/// `netOffset`.
pub(crate) fn gastropod_light(pos: Vec2, w: f32, h: f32) -> (i32, i32, [f32; 3]) {
    (
        tile_x(pos, w, XAnchor::CenterTruncated),
        tile_y(pos, h, XAnchor::CenterTruncated),
        [0.4, 0.0, 0.25],
    )
}

// ---------------------------------------------------------------------
// Site 11 — `:24862`, the aiStyle-22 arm's type-169 light (IceElemental,
// `NPCID.cs:11015`). TWIN: no NpcKind yet.
// ---------------------------------------------------------------------

/// `NPC.cs:24859-24870`: `AddLight((int)((x + (float)(width/2)) / 16f),
/// (int)((y + (float)(height/2)) / 16f), 0f, 0.6f, 0.75f)` inside the
/// type-169 `netOffset` bracket (`:24861`/`:24870`) — INTEGER halves,
/// quotient truncation; `pos` includes `netOffset`.
pub(crate) fn ice_elemental_light(pos: Vec2, w: f32, h: f32) -> (i32, i32, [f32; 3]) {
    (
        ((pos.x + int_half(w)) / 16.0) as i32,
        ((pos.y + int_half(h)) / 16.0) as i32,
        [0.0, 0.6, 0.75],
    )
}

// ---------------------------------------------------------------------
// Sites 12/13/14 — `:25241`/`:25245`/`:25249`, the aiStyle-23 head chain
// (83 CursedHammer / 179 CrimsonAxe / the else = 84 EnchantedSword,
// `NPCID.cs:10843`/`:11035`/`:10845`). TWIN: none of the three kinds is
// in the roster (`npc_lights.rs`'s module docs already park this chain
// — this module carries the verbatim twins so the wiring wave has the
// exact colours/anchors).
// ---------------------------------------------------------------------

/// `NPC.cs:25239-25250`: all three arms light the centre tile with the
/// INTEGER-half quotient form `(int)((x + (float)(width/2)) / 16f)`;
/// 83 → (0.2, 0.05, 0.3), 179 → (0.3, 0.15, 0.05), the else →
/// (0.05, 0.2, 0.3). No `netOffset` bracket in this arm.
pub(crate) fn cursed_weapon_light(vanilla_type: i32, pos: Vec2, w: f32, h: f32) -> (i32, i32, [f32; 3]) {
    let rgb = if vanilla_type == 83 {
        [0.2, 0.05, 0.3]
    } else if vanilla_type == 179 {
        [0.3, 0.15, 0.05]
    } else {
        [0.05, 0.2, 0.3]
    };
    (
        ((pos.x + int_half(w)) / 16.0) as i32,
        ((pos.y + int_half(h)) / 16.0) as i32,
        rgb,
    )
}

// ---------------------------------------------------------------------
// Site 15 — `:26403`, the aiStyle-29 tail (the Hungry 115, the style's
// only member). FED: the arm's wall-presence gate (`:26215-26218`)
// deactivates the NPC when the wall is gone — the port's `hp = 0`
// convention (`npc_ai_style29.rs`) makes the feed's `hp <= 0` skip the
// exact stand-in, so a live Hungry always lights.
// ---------------------------------------------------------------------

/// `NPC.cs:26403`: `AddLight((int)(x + (float)(width/2)) / 16,
/// (int)(y + (float)(height/2)) / 16, 0.3f, 0.2f, 0.1f)` — INTEGER
/// halves, cast-before-divide. No `netOffset` bracket in aiStyle 29.
pub(crate) fn hungry_light(pos: Vec2, w: f32, h: f32) -> (i32, i32, [f32; 3]) {
    (
        (pos.x + int_half(w)) as i32 / 16,
        (pos.y + int_half(h)) as i32 / 16,
        [0.3, 0.2, 0.1],
    )
}

// ---------------------------------------------------------------------
// Sites 16/17 — `:32781`/`:32785`, the aiStyle-57 head lights (344
// Everscream / 325 MourningWood). FED: both kinds ride the wired
// aiStyle-57 axis arms (`npc.rs`'s `npc_ai_pumpkin_boss` /
// `npc_ai_frost_boss_b` machines); the lights are unconditional per
// tick (no gate, no `netOffset`, no early return before them).
// ---------------------------------------------------------------------

/// `NPC.cs:32779-32786`: `AddLight(base.Bottom + new Vector2(0f, -30f),
/// ...)`. `base.Bottom` = `(x + width/2f, y + height)` (`Entity.cs:
/// 122-127`, FLOAT halves); the Vector2 overload truncates the quotient
/// per axis (`Lighting.cs:140-143`). 344 → (0.3, 0.16, 0.125), 325 →
/// (0.3, 0.125, 0.06).
pub(crate) fn moon_boss_bottom_light(
    vanilla_type: i32,
    pos: Vec2,
    w: f32,
    h: f32,
) -> (i32, i32, [f32; 3]) {
    let rgb = if vanilla_type == 344 {
        [0.3, 0.16, 0.125]
    } else {
        [0.3, 0.125, 0.06]
    };
    (
        ((pos.x + w / 2.0) / 16.0) as i32,
        ((pos.y + h - 30.0) / 16.0) as i32,
        rgb,
    )
}

// ---------------------------------------------------------------------
// Site 18 — `:33059`, the aiStyle-58 head light (327 Pumpking, the
// style's only member). FED: the wired `pumpking_step` axis arm; the
// light is unconditional (no gate before it, no early return in the
// arm).
// ---------------------------------------------------------------------

/// `NPC.cs:33057-33060`: `AddLight(base.Center, 0.4f, 0.4f, 0.2f)` —
/// the Vector2 overload's quotient truncation ([`XAnchor::Center`]).
pub(crate) fn pumpking_light(pos: Vec2, w: f32, h: f32) -> (i32, i32, [f32; 3]) {
    (
        tile_x(pos, w, XAnchor::Center),
        tile_y(pos, h, XAnchor::Center),
        [0.4, 0.4, 0.2],
    )
}

// ---------------------------------------------------------------------
// Site 19 — `:33709`, the aiStyle-61 centre light (346 SantaNK1, the
// style's only member). FED: the wired `frost_boss_b_step` axis arm.
// The light is gated on the centre tile being in-world and NOT solid
// (`WorldGen.InWorld` + `!WorldGen.SolidTile`, `:33707`); the probe
// reads `ToTileCoordinates` (`(int)v >> 4` — the arithmetic shift AFTER
// truncation, `Utils.cs:1807-1810`), a DIFFERENT form from the light's
// own quotient truncation for negative centres. Both kept verbatim.
// ---------------------------------------------------------------------

/// `NPC.cs:33705-33710`: `center15 = base.Center`; the gate probes
/// `center15.ToTileCoordinates()`; the light itself is
/// `AddLight(center15, 0.3f, 0.26f, 0.05f)` (quotient truncation).
/// The solid-tile probe is the frost module's verbatim
/// `WorldGen.SolidTile` port (shared, not copied — the house rule).
pub(crate) fn santa_nk1_light(
    pos: Vec2,
    w: f32,
    h: f32,
    world: &World,
) -> Option<(i32, i32, [f32; 3])> {
    let cx = pos.x + w / 2.0;
    let cy = pos.y + h / 2.0;
    let tx = (cx as i32) >> 4;
    let ty = (cy as i32) >> 4;
    if tx >= 0
        && tx < world.width
        && ty >= 0
        && ty < world.height
        && !crate::npc_ai_frost_boss_b::worldgen_solid_tile(world, tx, ty)
    {
        Some((
            tile_x(pos, w, XAnchor::Center),
            tile_y(pos, h, XAnchor::Center),
            [0.3, 0.26, 0.05],
        ))
    } else {
        None
    }
}

// Site 20 — `:34162` (the aiStyle-64 firefly glow, type 358
// LightningBug): DEDUP-TWIN — `npc_ai_style64.rs` already ports the
// whole `:34154-34173` glow tail verbatim behind
// `Ai64Effects::light` (the `localAI[2] > 3` gate, the truncated-centre
// tiles, the scale-multiplied colours). The module is UNWIRED (no
// firefly NpcKind); its wiring feeds the light ONCE — this module
// never double-feeds it. Sites 21/22 (`:34166` Lavafly 654 / `:34170`
// the else) ride the same seam.

// Sites 21/22 — `:34166` (Lavafly 654) / `:34170` (the else firefly):
// DEDUP-TWIN — the same `npc_ai_style64.rs` glow tail
// (`Ai64Effects::light`), one feed at its wiring.

// ---------------------------------------------------------------------
// Site 23 — `:34258`, the aiStyle-66 head light (484
// EnchantedNightcrawler, `NPCID.cs:11641`). TWIN: no NpcKind yet, and
// the light's factor draws `Main.rand.Next(90, 111)` — the draw belongs
// to the future aiStyle-66 machine on the `UpdateNPCs` stream (the
// `item_lights.rs` rand-head precedent); this twin takes the ROLLED
// factor so the wiring feeds the exact value without re-drawing.
// ---------------------------------------------------------------------

/// `NPC.cs:34255-34259`: `num983 = (float)Main.rand.Next(90, 111) *
/// 0.01f * ((Main.essScale + 0.5f) / 2f)`; `AddLight((int)((x +
/// (float)(width/2)) / 16f), (int)((y + (float)(height/2)) / 16f),
/// 0.3f*num983, 0.1f*num983, 0.25f*num983)` — INTEGER halves, quotient
/// truncation. `rolled` is the `Next(90, 111)` result (90..=110);
/// `ess_scale` is `Main.essScale` at the draw.
pub(crate) fn nightcrawler_light(
    rolled: i32,
    ess_scale: f32,
    pos: Vec2,
    w: f32,
    h: f32,
) -> (i32, i32, [f32; 3]) {
    let num983 = rolled as f32 * 0.01 * ((ess_scale + 0.5) / 2.0);
    (
        ((pos.x + int_half(w)) / 16.0) as i32,
        ((pos.y + int_half(h)) / 16.0) as i32,
        [0.3 * num983, 0.1 * num983, 0.25 * num983],
    )
}

// Sites 24/25 — `:34387` (GlowingSnail 360, 0.1/0.2/0.7) / `:34391`
// (MagmaSnail 655, 0.6/0.3/0.1): DEDUP-TWIN — `npc_ai_style67.rs`
// already ports both verbatim behind `SnailEvents::light` (the
// cast-before-divide centre tiles, `:34385-34392`); the module is
// UNWIRED (no snail NpcKind) and its wiring feeds the light ONCE.

// Site 26 — `:35092` (ForceBubble 384, the attached-bubble light
// 0.1/0.5/0.7 at the truncated centre): DEDUP — already fed by the
// draw feed's small-kinds inline arm (`app/frame/draw.rs`, the
// S1-D-3-wire-smallkinds wave).

// Sites 27/28 — `:35167`/`:35175` (the Tesla Turret 387's fade/solid
// lights): DEDUP-TWIN — `npc_ai_style73.rs` ports both behind
// `Style73Outcome::light` (the `ai[1]/60` fade factor at the truncated
// centre 10 px up, then the constant solid light). The machine is
// UNWIRED (the turret's `update_kind_ai` arm is `{}`); its wiring feeds
// the outcome ONCE — the fade factor rides the machine's own `ai[1]`
// cadence, which a draw-side twin cannot reproduce without re-stepping.

// ---------------------------------------------------------------------
// Site 29 — `:35462`, the aiStyle-74 state-3 implosion light (388
// MartianDrone / 418 SolarCorite). FED: both kinds' stand-in machines
// (`update_martian_drone` / `update_solar_corite`) hold `ai[0] == 3`
// for the 3-tick fuse. The C# re-centres the position and swells the
// box to 192×192 (`:35455-35458`) BEFORE the light — but the re-centre
// keeps `Center` invariant (position = oldCenter - 96, +96 back), so
// the light's tile is the same with the port's kind-fixed size. The
// C# light fires on the death tick too (`:35492-35498` kills AFTER the
// AddLight); the port's feed skips `hp <= 0` — the documented
// dead-skip convention shared by every NPC feed (ANALYSIS).
// ---------------------------------------------------------------------

/// `NPC.cs:35462`: `AddLight((int)Center.X / 16, (int)Center.Y / 16,
/// 0.2f, 0.7f, 1.1f)` — cast-before-divide
/// ([`XAnchor::CenterTruncated`]).
pub(crate) fn implosion_light(pos: Vec2, w: f32, h: f32) -> (i32, i32, [f32; 3]) {
    (
        tile_x(pos, w, XAnchor::CenterTruncated),
        tile_y(pos, h, XAnchor::CenterTruncated),
        [0.2, 0.7, 1.1],
    )
}

// ---------------------------------------------------------------------
// Site 30 — `:36623`, the aiStyle-80 tail light (399 MartianProbe, the
// style's only member). FED: the wired `update_martian_probe` stand-in.
// The C#'s state-0 arm returns at `:36575` when `FindClosestPlayer`
// finds no live player — the light is skipped on those ticks; states
// 1/2 always reach the tail. `rgb = Color.SkyBlue.ToVector3()`, or
// `Color.Red.ToVector3()` in state 2, then `rgb *= 0.65f` (`:36617-
// 36622`); `AddLight(base.Center, rgb)` truncates the quotient.
// ---------------------------------------------------------------------

/// `NPC.cs:36617-36623`. `Color.SkyBlue` = (135, 206, 235) (XNA's
/// named colour); `ToVector3` divides each byte by 255f; the ×0.65 is
/// per-component f32. `player_dead` folds the `:36572-36576` gate
/// (the port is single-player: an absent player reads as dead).
pub(crate) fn martian_probe_light(
    ai0: f32,
    player_dead: bool,
    pos: Vec2,
    w: f32,
    h: f32,
) -> Option<(i32, i32, [f32; 3])> {
    if ai0 == 0.0 && player_dead {
        return None;
    }
    let base = if ai0 == 2.0 {
        [1.0, 0.0, 0.0]
    } else {
        [135.0 / 255.0, 206.0 / 255.0, 235.0 / 255.0]
    };
    Some((
        tile_x(pos, w, XAnchor::Center),
        tile_y(pos, h, XAnchor::Center),
        [base[0] * 0.65, base[1] * 0.65, base[2] * 0.65],
    ))
}

// ---------------------------------------------------------------------
// The dispatcher: the FED sites, keyed on the roster kind. The TWIN
// sites' kinds are absent (or their machines unwired), so they return
// `None` here until their wave lands — the per-site table in ANALYSIS
// names each one's owner.
// ---------------------------------------------------------------------

/// The FIRST-half `AddLight` source this NPC contributes this frame, or
/// `None`. Called from the draw feed after the step, in `Main.npc` slot
/// order, beside `npc_lights.rs::npc_light` (the aiStyle-8 tail — the
/// two modules' kind sets are disjoint, so no site double-feeds).
///
/// `player_dead` stands in for the probe light's `FindClosestPlayer`
/// gate (`:36572-36576` — a dead/absent player skips the MartianProbe's
/// state-0 light); `world` carries the SantaNK1 centre-tile probe
/// (`:33707`). No arm draws RNG.
pub(crate) fn npc_light_a(n: &Npc, world: &World, player_dead: bool) -> Option<(i32, i32, [f32; 3])> {
    let (w, h) = n.size();
    // The raw top-left: every FED site below reads `base.position`
    // WITHOUT `netOffset` except the BurningSphere's dust-loop else arm
    // (the `:21516` bracket), which adds it explicitly.
    let pos = Vec2::new(n.x, n.y);
    match n.kind {
        // Site 4 — `:21561`, the aiStyle-9 else arm (offset position).
        NpcKind::BurningSphere => Some(burning_sphere_light(pos + n.net_offset, w, h)),
        // Site 15 — `:26403`, the aiStyle-29 tail. A live Hungry always
        // lights: the wall-gone deactivation (`:26217`) is the port's
        // `hp = 0` (the feed's dead skip).
        NpcKind::Hungry => Some(hungry_light(pos, w, h)),
        // Sites 16/17 — `:32781`/`:32785`, the aiStyle-57 head lights.
        NpcKind::Everscream => Some(moon_boss_bottom_light(344, pos, w, h)),
        NpcKind::MourningWood => Some(moon_boss_bottom_light(325, pos, w, h)),
        // Site 18 — `:33059`, the aiStyle-58 head light.
        NpcKind::Pumpking => Some(pumpking_light(pos, w, h)),
        // Site 19 — `:33709`, the aiStyle-61 centre light (tile probe).
        NpcKind::SantaNK1 => santa_nk1_light(pos, w, h, world),
        // Site 29 — `:35462`, the aiStyle-74 state-3 implosion light.
        // The C#'s 192-box re-centre keeps Center invariant (module
        // docs), so the kind-fixed size reads the same tile.
        NpcKind::MartianDrone | NpcKind::SolarCorite if n.ai[0] == 3.0 => {
            Some(implosion_light(pos, w, h))
        }
        // Site 30 — `:36623`, the aiStyle-80 tail light (the state-0
        // dead-player gate folds the `:36575` return).
        NpcKind::MartianProbe => martian_probe_light(n.ai[0], player_dead, pos, w, h),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Site 1 — `:21303-21307`: ai[0] = 30 remaps to 0.5, squared
    /// 0.25; Top of a 30x30 box at (160, 176) = (175, 176) → tile
    /// (10, 11); colour 0.25·(0.91, 0.82, 0.57).
    #[test]
    fn w4_c20_s01_librarian_skeleton_squares_the_remap() {
        let (x, y, rgb) =
            librarian_skeleton_light(30.0, Vec2::new(160.0, 176.0), 30.0, 30.0).unwrap();
        assert_eq!((x, y), (10, 11));
        assert_eq!(rgb, [0.25 * 0.91, 0.25 * 0.82, 0.25 * 0.57]);
    }

    /// Site 1 — the `num111 > 0` gate: ai[0] <= 0 clamps to 0, and the
    /// squared 0 stays dark; ai[0] >= 60 clamps to the full 1.
    #[test]
    fn w4_c20_s01_librarian_skeleton_clamps_both_ends() {
        assert!(librarian_skeleton_light(0.0, Vec2::ZERO, 30.0, 30.0).is_none());
        assert!(librarian_skeleton_light(-10.0, Vec2::ZERO, 30.0, 30.0).is_none());
        let (_, _, rgb) =
            librarian_skeleton_light(90.0, Vec2::ZERO, 30.0, 30.0).unwrap();
        assert_eq!(rgb, [0.91, 0.82, 0.57]);
    }

    /// Site 4 — `:21561`: the 16x16 BurningSphere (`NPC.cs:9095-9096`)
    /// at (160, 176) centres (168, 184) → tile (10, 11), (1, 0.3, 0.1).
    #[test]
    fn w4_c20_s04_burning_sphere_center_light() {
        let (x, y, rgb) = burning_sphere_light(Vec2::new(160.0, 176.0), 16.0, 16.0);
        assert_eq!((x, y), (10, 11));
        assert_eq!(rgb, [1.0, 0.3, 0.1]);
    }

    /// Site 4 — the two-iteration dust loop (`:21517`) appends the
    /// else-arm light TWICE; every other kind once.
    #[test]
    fn w4_c20_s04_burning_sphere_emits_twice() {
        assert_eq!(npc_light_a_emits(NpcKind::BurningSphere), 2);
        assert_eq!(npc_light_a_emits(NpcKind::ChaosBall), 1);
        assert_eq!(npc_light_a_emits(NpcKind::WaterSphere), 1);
    }

    /// Site 4 — the integer half: an ODD width loses the 0.5
    /// (`(float)(width / 2)`), unlike the float-half Center anchor.
    #[test]
    fn w4_c20_s04_integer_half_drops_the_odd_pixel() {
        // width 17: int half 8 (not 8.5). x = 151 + 8 = 159 → 159/16
        // = 9.9375 → 9; the float half would give 159.5 → still 9, so
        // pick the boundary: x = 152 + 8 = 160 → 10 vs 160.5 → 10.
        // The cast-before-divide twin below pins the odd case exactly.
        let (x, _, _) = burning_sphere_light(Vec2::new(152.0, 0.0), 17.0, 16.0);
        assert_eq!(x, 10);
        assert_eq!(int_half(17.0), 8.0);
    }

    /// Site 5 — `:23764`: a 24x24 AnglerFish at (160, 176) facing +1
    /// lights x = 160 + 12 + 32 = 204 → 204/16 = 12 (int divide),
    /// y = (176 + 2)/16 = 11; facing -1 lights x = 160 + 12 - 32 = 140
    /// → 8.
    #[test]
    fn w4_c20_s05_angler_fish_lights_a_box_ahead() {
        let (x, y, rgb) = angler_fish_light(Vec2::new(160.0, 176.0), 24.0, 24.0, 1);
        assert_eq!((x, y), (12, 11));
        assert_eq!(rgb, [0.07, 0.04, 0.025]);
        let (x2, _, _) = angler_fish_light(Vec2::new(160.0, 176.0), 24.0, 24.0, -1);
        assert_eq!(x2, 8);
    }

    /// Sites 6/7/8 — `:24204-24219`: the wet-shield num263 = 1.5
    /// brightens the 63 blue; the X tile reads the HEIGHT half (the
    /// C# quirk): a 22x22 box at (160, 176) → x = (160 + 11)/16 = 10,
    /// y = (176 + 11)/16 = 11.
    #[test]
    fn w4_c20_s06_jellyfish_wet_shield_brightens() {
        let dry = jellyfish_light(63, Vec2::new(160.0, 176.0), 22.0, 22.0, false, 0.0).unwrap();
        assert_eq!(dry, (10, 11, [0.05, 0.15, 0.4]));
        let wet = jellyfish_light(63, Vec2::new(160.0, 176.0), 22.0, 22.0, true, 1.0).unwrap();
        assert_eq!(wet.2, [0.05 * 1.5, 0.15 * 1.5, 0.4 * 1.5]);
        // wet but ai[1] != 1: no shield, no bonus.
        let wet2 = jellyfish_light(63, Vec2::new(160.0, 176.0), 22.0, 22.0, true, 0.0).unwrap();
        assert_eq!(wet2.2, [0.05, 0.15, 0.4]);
    }

    /// Sites 7/8 — the 103 green and the else pink; 221/242 stay dark.
    #[test]
    fn w4_c20_s07_s08_jellyfish_chain_arms() {
        let g = jellyfish_light(103, Vec2::ZERO, 22.0, 22.0, false, 0.0).unwrap();
        assert_eq!(g.2, [0.05, 0.45, 0.1]);
        let p = jellyfish_light(64, Vec2::ZERO, 22.0, 22.0, false, 0.0).unwrap();
        assert_eq!(p.2, [0.35, 0.05, 0.2]);
        assert!(jellyfish_light(221, Vec2::ZERO, 22.0, 22.0, false, 0.0).is_none());
        assert!(jellyfish_light(242, Vec2::ZERO, 22.0, 22.0, false, 0.0).is_none());
    }

    /// Site 10 — `:24801`: cast-before-divide. A 26x26 Gastropod at
    /// (160, 176) centres (173, 189): 173/16 = 10, 189/16 = 11. The
    /// negative witness separates the forms: centre x = -1 casts to
    /// -1 then divides to 0 (truncation toward zero), where the
    /// quotient form `(int)(-1/16f)` also gives 0 but `(int)(-17/16f)`
    /// = -1 vs `-17/16` = -1 — the split is at the cast boundary:
    /// centre -16.5 → (int)(-16.5)/16 = -16/16 = -1, while the
    /// quotient (int)(-16.5/16) = (int)(-1.03125) = -1 too; the
    /// clean split is centre -0.5: cast 0 → 0, quotient (int)(-0.03125)
    /// = 0 — so the pin uses centre -1 (cast -1 → 0) against a floor
    /// implementation, which would give -1.
    #[test]
    fn w4_c20_s10_gastropod_truncated_center() {
        let (x, y, rgb) = gastropod_light(Vec2::new(160.0, 176.0), 26.0, 26.0);
        // Centre (173, 189): 173/16 = 10, 189/16 = 11.
        assert_eq!((x, y), (10, 11));
        assert_eq!(rgb, [0.4, 0.0, 0.25]);
        // The cast-before-divide form: centre x = -1 → (int)(-1)/16
        // = 0, NOT the floor -1.
        let (xn, _, _) = gastropod_light(Vec2::new(-14.0, 0.0), 26.0, 26.0);
        assert_eq!(xn, 0); // centre -1 → 0
    }

    /// Site 11 — `:24862`: the IceElemental's 30x42 box at (160, 176)
    /// centres (175, 197) → tile (10, 12); (0, 0.6, 0.75).
    #[test]
    fn w4_c20_s11_ice_elemental_center_light() {
        let (x, y, rgb) = ice_elemental_light(Vec2::new(160.0, 176.0), 30.0, 42.0);
        assert_eq!((x, y), (10, 12));
        assert_eq!(rgb, [0.0, 0.6, 0.75]);
    }

    /// Sites 12/13/14 — `:25239-25249`: the three head-chain colours at
    /// the integer-half centre tile.
    #[test]
    fn w4_c20_s12_s13_s14_cursed_weapon_chain() {
        let h = cursed_weapon_light(83, Vec2::new(160.0, 176.0), 32.0, 32.0);
        assert_eq!(h, (11, 12, [0.2, 0.05, 0.3]));
        let a = cursed_weapon_light(179, Vec2::new(160.0, 176.0), 32.0, 32.0);
        assert_eq!(a.2, [0.3, 0.15, 0.05]);
        let e = cursed_weapon_light(84, Vec2::new(160.0, 176.0), 32.0, 32.0);
        assert_eq!(e.2, [0.05, 0.2, 0.3]);
    }

    /// Site 15 — `:26403`: the Hungry's 30x30 box (`NPC.cs:10380-10381`)
    /// at (160, 176) centres (175, 191): cast-before-divide gives
    /// 175/16 = 10, 191/16 = 11.
    #[test]
    fn w4_c20_s15_hungry_center_light() {
        let (x, y, rgb) = hungry_light(Vec2::new(160.0, 176.0), 30.0, 30.0);
        assert_eq!((x, y), (10, 11));
        assert_eq!(rgb, [0.3, 0.2, 0.1]);
    }

    /// Sites 16/17 — `:32781`/`:32785`: Bottom + (0, -30). The
    /// Everscream's 172x130 box (`NPC.cs:13355+`) at (160, 176) has
    /// Bottom = (246, 306); -30 → (246, 276) → tile (15, 17).
    #[test]
    fn w4_c20_s16_s17_moon_boss_bottom_lights() {
        let e = moon_boss_bottom_light(344, Vec2::new(160.0, 176.0), 172.0, 130.0);
        assert_eq!(e, (15, 17, [0.3, 0.16, 0.125]));
        let m = moon_boss_bottom_light(325, Vec2::new(160.0, 176.0), 164.0, 154.0);
        // Bottom = (242, 330); -30 → 300 → 300/16 = 18.75 → 18.
        assert_eq!(m, (15, 18, [0.3, 0.125, 0.06]));
    }

    /// Site 18 — `:33059`: the Pumpking's 100x100 box at (160, 176)
    /// centres (210, 226) → tile (13, 14); (0.4, 0.4, 0.2).
    #[test]
    fn w4_c20_s18_pumpking_center_light() {
        let (x, y, rgb) = pumpking_light(Vec2::new(160.0, 176.0), 100.0, 100.0);
        assert_eq!((x, y), (13, 14));
        assert_eq!(rgb, [0.4, 0.4, 0.2]);
    }

    /// Site 19 — `:33705-33710`: the light fires only when the centre
    /// tile is in-world and not solid. An air world passes; an
    /// out-of-world centre (negative x) stays dark.
    #[test]
    fn w4_c20_s19_santa_light_gates_on_the_center_tile() {
        let w = tr_core::World::new("t".into(), 200, 120);
        let lit = santa_nk1_light(Vec2::new(160.0, 176.0), 112.0, 140.0, &w).unwrap();
        // Centre (216, 246) → tile (13, 15).
        assert_eq!(lit, (13, 15, [0.3, 0.26, 0.05]));
        // Centre x = -1 → probe tx = -1 >> 4 = -1: out of world, dark.
        assert!(santa_nk1_light(Vec2::new(-57.0, 176.0), 112.0, 140.0, &w).is_none());
    }

    /// Site 23 — `:34257-34258`: rolled 100 → 1.0, essScale 1.0 →
    /// (1.5/2) = 0.75 → num983 = 0.75; colour 0.75·(0.3, 0.1, 0.25).
    /// A 20x20 box at (160, 176) centres (170, 186) → tile (10, 11).
    #[test]
    fn w4_c20_s23_nightcrawler_flicker_factor() {
        let (x, y, rgb) = nightcrawler_light(100, 1.0, Vec2::new(160.0, 176.0), 20.0, 20.0);
        assert_eq!((x, y), (10, 11));
        assert_eq!(rgb, [0.3 * 0.75, 0.1 * 0.75, 0.25 * 0.75]);
        // The roll range's low end: 90 → 0.9; essScale 0 → 0.25.
        let (_, _, rgb2) = nightcrawler_light(90, 0.0, Vec2::ZERO, 20.0, 20.0);
        let n = 0.9 * 0.25;
        assert_eq!(rgb2, [0.3 * n, 0.1 * n, 0.25 * n]);
    }

    /// Site 29 — `:35462`: cast-before-divide at the centre. A 40x30
    /// drone (`NPC.cs:13937-13938`) at (160, 176) centres (180, 191):
    /// 180/16 = 11, 191/16 = 11.
    #[test]
    fn w4_c20_s29_implosion_center_light() {
        let (x, y, rgb) = implosion_light(Vec2::new(160.0, 176.0), 40.0, 30.0);
        assert_eq!((x, y), (11, 11));
        assert_eq!(rgb, [0.2, 0.7, 1.1]);
    }

    /// Site 30 — `:36617-36623`: SkyBlue·0.65 in states 0/1, Red·0.65
    /// in state 2; the state-0 dead-player gate returns None.
    #[test]
    fn w4_c20_s30_probe_skyblue_and_escape_red() {
        let w = 42.0;
        let h = 72.0;
        let pos = Vec2::new(160.0, 176.0);
        let idle = martian_probe_light(0.0, false, pos, w, h).unwrap();
        assert_eq!(
            idle.2,
            [135.0 / 255.0 * 0.65, 206.0 / 255.0 * 0.65, 235.0 / 255.0 * 0.65]
        );
        let esc = martian_probe_light(2.0, true, pos, w, h).unwrap();
        assert_eq!(esc.2, [0.65, 0.0, 0.0]);
        // The state-0 dead-player return (`:36575`); state 1 lights
        // regardless (the gate lives in the state-0 arm only).
        assert!(martian_probe_light(0.0, true, pos, w, h).is_none());
        assert!(martian_probe_light(1.0, true, pos, w, h).is_some());
    }
}
