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
        _ => {
            let _ = (world, player_dead);
            None
        }
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
}
