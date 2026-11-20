//! The TILE-draw `Lighting.AddLight` sources (W4-e8, the second tile batch;
//! W4-c20 owns the first mixed NPC/tile batch).
//!
//! C# `Lighting.AddLight` funnels every overload into
//! `Lighting.AddLight(int i, int j, float r, float g, float b)`
//! (`Lighting.cs:157-163`), which appends to the active engine's per-frame
//! list behind `!Main.gamePaused && Main.netMode != 2`
//! (`LightingEngine.cs:51-54`). The engine max-merges that list onto the
//! working light map one frame late (`LightingEngine.cs:208-235`). There is
//! ONE list pair per engine — [`crate::entity_lights::EntityLights`] — and
//! this family feeds the same pair as the item/NPC/player/proj/dust
//! families; the tile-anchor vocabulary is shared from there too.
//!
//! **Scope (the census):** the draw-side `AddLight` sites that project light
//! from TILES and the tile-anchored draw passes — `Main.DrawProj`'s hook
//! chains (`Main.cs:29016-29028`, `:29186`), `TileDrawing.DrawTrees`'
//! mushroom-tree flicker (`TileDrawing.cs:8127/8132`, `:8172/8177`,
//! `:8220/8225`) and the `DelegateMethods.CastLight*` tile-cast delegates
//! (`DelegateMethods.cs:722/738/775/795/812`) the `Utils.PlotTileLine`
//! laser/hook lights ride. The NPC/projectile/dust/item/player update-side
//! families stay with their own modules (`npc_lights.rs`, `proj_lights.rs`,
//! `dust_lights.rs`, `entity_lights.rs`, `player_lights.rs`); the gore/mount/
//! leashed-critter/particle draw lights land in the sibling W4-e8 modules
//! (`gore_lights.rs`, `mount_lights.rs`, `critter_lights.rs`,
//! `particle_lights.rs`).
//!
//! **RNG rule:** the mushroom flicker draws `TileDrawing._rand.Next(28, 42)`
//! (`TileDrawing.cs:8124/8169/8217`) — a DRAW-pass stream, not `Main.rand`,
//! so it rides the foliage pass's pinned per-frame `FastRandom` (the
//! `EmitTreeLeaves` stream precedent, `tiles.rs:332`) and cannot move any
//! fixed-step stream. Every other arm here is RNG-free.

#![allow(dead_code)] // unwired pure module (code-only mode) — the hook-chain
                     // lights ride the unported grapple-chain draw, the flicker
                     // rides foliage.rs's leaf pass and the casts ride the
                     // laser/hook PlotTileLine wiring wave.

use glam::Vec2;
use tr_core::tile_light::FastRandom;
use tr_core::TILE_PX;

/// The `Lighting.AddLight(Vector2, ...)` overload's tile anchor
/// (`Lighting.cs:140-143`): `(int)(v / 16f)` per axis — truncate the
/// QUOTIENT (toward zero), not the coordinate first.
#[inline]
pub(crate) fn vector_tile(v: Vec2) -> (i32, i32) {
    ((v.x / TILE_PX) as i32, (v.y / TILE_PX) as i32)
}

/// `Utils.ToTileCoordinates` (`Utils.cs:1807-1810`): `(int)v >> 4` per
/// axis — truncate the coordinate, then ARITHMETIC-shift (floor for
/// negatives). NOT the same as [`vector_tile`] below zero.
#[inline]
pub(crate) fn shift_tile(v: Vec2) -> (i32, i32) {
    ((v.x as i32) >> 4, (v.y as i32) >> 4)
}

/// `Main.DrawProj`'s Christmas Hook (332) chain lights
/// (`Main.cs:29012-29029`): the chain walk steps from the hook centre
/// toward `mountedCenter` in 16 px strides (`num115 = 16f / num115`,
/// `:29030`), and the LIGHT tile is sampled at the PRE-step `vector19`;
/// the first four links light their tile with a rotating colour
/// (`num112` wraps 0..3 at `:29039-29043`).
///
/// The walk itself is the grapple-chain draw's (unported — `grapple.rs`
/// documents the chain-rendering drop); this fn replays ONLY the light
/// half, verbatim: the `< 30f` / NaN exits (`:29002-29011`), the
/// `(int)vector19.X / 16` truncate-then-divide anchor (`:29012-29013`),
/// the four colour rows, and the 16 px step + mounted-centre re-aim
/// (`:29030-29036`). `center` starts at the hook's `proj.Center`
/// (`:29093`'s `vector19` init at `:28993`); the 332 arm has no `num117`
/// offset (`:29097-29111` lists 383/446/489/487 only).
pub(crate) fn christmas_hook_chain_lights(
    center: Vec2,
    mounted_center: Vec2,
    out: &mut Vec<(i32, i32, [f32; 3])>,
) {
    let mut vector19 = center;
    let mut num113 = mounted_center.x - vector19.x;
    let mut num114 = mounted_center.y - vector19.y;
    let mut num112 = 0i32;
    // `:28998-29044` — `bool flag21 = true; while (flag21)`.
    loop {
        let mut num115 = (num113 * num113 + num114 * num114).sqrt();
        if num115 < 30.0 {
            break; // `:29002-29006`
        }
        if num115.is_nan() {
            break; // `:29007-29011`
        }
        // `:29012-29013` — `(int)vector19.X / 16`: truncate FIRST.
        let i2 = vector19.x as i32 / 16;
        let j2 = vector19.y as i32 / 16;
        // `:29014-29029` — the four colour rows, in source order.
        if num112 == 0 {
            out.push((i2, j2, [0.0, 0.2, 0.2])); // `:29016`
        }
        if num112 == 1 {
            out.push((i2, j2, [0.1, 0.2, 0.0])); // `:29020`
        }
        if num112 == 2 {
            out.push((i2, j2, [0.2, 0.1, 0.0])); // `:29024`
        }
        if num112 == 3 {
            out.push((i2, j2, [0.2, 0.0, 0.2])); // `:29028`
        }
        // `:29030-29036` — the 16 px step toward the mount, re-aim.
        num115 = 16.0 / num115;
        num113 *= num115;
        num114 *= num115;
        vector19.x += num113;
        vector19.y += num114;
        num113 = mounted_center.x - vector19.x;
        num114 = mounted_center.y - vector19.y;
        // `:29039-29043` — the colour index wraps after 3.
        num112 += 1;
        if num112 > 3 {
            num112 = 0;
        }
    }
}

/// `Main.DrawProj`'s Illuminant Hook (488) chain light
/// (`Main.cs:29184-29188`): inside the 372/383/396/.../652 chain walk
/// (`:29046`), every link of the 488 hook lights its tile with
/// `(0.2, 0, 0.175)` at the `Lighting.AddLight(Vector2, ...)` quotient
/// anchor. The walk steps `center2` by `vector22 * num116` (`:29173`)
/// where `num116 = value11.Height` — the `Chains[488 - 486] = Chains[2]`
/// sheet height (`:29071-29074`); the 488 arm has no `num117` offset and
/// no 383-style bend (`:29131-29143`/`:29153-29172` are 383-only), so
/// `vector22` is the plain normalized aim at `mountedCenter`.
///
/// `chain_height` is the sheet height the caller reads (the port's chain
/// rendering is unported — the light half takes it as an input, the
/// `grapple.rs` chain-drop precedent). The exit is
/// `vector21.Length() < num116 + 1f` (`:29146-29150`), checked BEFORE the
/// step; the NaN double-gates (`:29123-29130`) kill the walk up front.
pub(crate) fn illuminant_hook_chain_lights(
    center: Vec2,
    mounted_center: Vec2,
    chain_height: f32,
    out: &mut Vec<(i32, i32, [f32; 3])>,
) {
    let mut center2 = center;
    let mut vector21 = mounted_center - center2;
    // `:29123-29130` — both NaN pairs must hold to kill the walk.
    if (center2.x.is_nan() && center2.y.is_nan())
        || (vector21.x.is_nan() && vector21.y.is_nan())
    {
        return;
    }
    // `:29144-29198` — `while (flag22)`.
    loop {
        if vector21.length() < chain_height + 1.0 {
            break; // `:29146-29150`
        }
        // `:29151-29152` — normalize the aim (no 383 bend for 488).
        let mut vector22 = vector21;
        vector22 = vector22.normalize_or_zero();
        // `:29173-29174` — step, then re-aim.
        center2 += vector22 * chain_height;
        vector21 = mounted_center - center2;
        // `:29184-29188` — the 488 light at the stepped link.
        let (x, y) = vector_tile(center2);
        out.push((x, y, [0.2, 0.0, 0.175]));
    }
}

/// `TileDrawing.DrawTrees`' mushroom-tree flicker
/// (`TileDrawing.cs:8122-8134`, the top arm; `:8167-8179` the frameX-44
/// branch; `:8215-8227` the frameX-66 branch — all three identical):
/// `treeStyle == 14` (the glowing-mushroom ground, `WorldGen.cs:62924`)
/// lights the marker tile `(x, y)` with
/// `num = (float)_rand.Next(28, 42) * 0.005f + (270 - mouseTextColor) / 1000f`
/// then `(0.1, 0.2 + num/2, 0.7 + num)` unpainted, or the paint colour's
/// `R/G/B / 255f` painted. The draw happens AFTER `EmitTreeLeaves` and
/// BEFORE the canopy sprite (`:8121-8135`), once per marker.
///
/// `rng` is the foliage pass's `_rand` stand-in (the pinned per-frame
/// stream); the draw rides it exactly where the C# draws — only when
/// `tree_style == 14`, so non-mushroom markers consume nothing.
pub(crate) fn mushroom_tree_light(
    tree_style: usize,
    paint: u8,
    mouse_text_color: u8,
    rng: &mut FastRandom,
) -> Option<[f32; 3]> {
    if tree_style != 14 {
        return None;
    }
    // `:8124` — one Next(28, 42) draw, then the mouseTextColor offset.
    let num = rng.next_range(28, 42) as f32 * 0.005
        + (270.0 - mouse_text_color as f32) / 1000.0;
    if paint == 0 {
        // `:8127` — the unpainted flicker.
        Some([0.1, 0.2 + num / 2.0, 0.7 + num])
    } else {
        // `:8131-8132` — `WorldGen.paintColor(tile.color())` bytes / 255.
        Some(tr_core::tile_light::paint_color(paint))
    }
}

/// The `DelegateMethods` statics the tile-cast delegates read
/// (`DelegateMethods.cs:60-66` — `v3_1` the colour, `v2_1` the origin,
/// `f_1` the range). The laser/hook draw passes set them before
/// `Utils.PlotTileLine`; the port carries them as an explicit env so the
/// casts stay pure.
#[derive(Clone, Copy)]
pub(crate) struct CastLightEnv {
    /// `DelegateMethods.v3_1` — the light colour.
    pub rgb: [f32; 3],
    /// `DelegateMethods.v2_1` — the distance origin (tile coords).
    pub origin: Vec2,
    /// `DelegateMethods.f_1` — the range divisor / radius.
    pub range: f32,
}

/// `WorldGen.SolidTile`'s open-test the casts share: a tile is "open" when
/// it is absent, inactive, actuated, a solid-top, or not solid
/// (`DelegateMethods.cs:736`/`:771`/`:791`). `None` (out of array) is open.
#[inline]
fn tile_is_open(world: &tr_core::World, x: i32, y: i32) -> bool {
    let Some(t) = world.tile(x, y) else {
        return true;
    };
    !t.active()
        || t.inactive()
        || tr_core::tiledata::TILE_SOLID_TOP.has(t.kind)
        || !tr_core::tiledata::TILE_SOLID.has(t.kind)
}

/// `DelegateMethods.CastLight` (`DelegateMethods.cs:712-724`): bounds +
/// null gate, then an unconditional `AddLight(x, y, v3_1)`. Returns the
/// C# `bool` (false only when out of bounds / null).
pub(crate) fn cast_light(
    world: &tr_core::World,
    x: i32,
    y: i32,
    env: &CastLightEnv,
    out: &mut Vec<(i32, i32, [f32; 3])>,
) -> bool {
    if x < 0 || x >= world.width || y < 0 || y >= world.height {
        return false; // `:714-717`
    }
    if world.tile(x, y).is_none() {
        return false; // `:718-721`
    }
    out.push((x, y, env.rgb)); // `:722`
    true
}

/// `DelegateMethods.CastLightOpen` (`DelegateMethods.cs:726-741`): bounds +
/// null gate, then lights only an OPEN tile; always returns true once in
/// bounds (the open-test gates the light, not the return).
pub(crate) fn cast_light_open(
    world: &tr_core::World,
    x: i32,
    y: i32,
    env: &CastLightEnv,
    out: &mut Vec<(i32, i32, [f32; 3])>,
) -> bool {
    if x < 0 || x >= world.width || y < 0 || y >= world.height {
        return false;
    }
    if world.tile(x, y).is_none() {
        return false;
    }
    if tile_is_open(world, x, y) {
        out.push((x, y, env.rgb)); // `:738`
    }
    true
}

/// `DelegateMethods.CastLightOpen_StopForSolids`
/// (`DelegateMethods.cs:781-799`): like [`cast_light_open`] but returns
/// FALSE on a solid tile (the walk stops). The `new Vector2(x, y)` at
/// `:794` is a dead expression in the C# — dropped.
pub(crate) fn cast_light_open_stop_for_solids(
    world: &tr_core::World,
    x: i32,
    y: i32,
    env: &CastLightEnv,
    out: &mut Vec<(i32, i32, [f32; 3])>,
) -> bool {
    if x < 0 || x >= world.width || y < 0 || y >= world.height {
        return false;
    }
    if world.tile(x, y).is_none() {
        return false;
    }
    if tile_is_open(world, x, y) {
        out.push((x, y, env.rgb)); // `:795`
        return true;
    }
    false
}

/// `DelegateMethods.CastLightOpen_StopForSolids_ScaleWithDistance`
/// (`DelegateMethods.cs:761-779`): the open-stop cast whose colour scales
/// by `Lerp(0.65, 1, Distance((x,y), v2_1) / f_1)` (`:774`).
pub(crate) fn cast_light_open_stop_scale(
    world: &tr_core::World,
    x: i32,
    y: i32,
    env: &CastLightEnv,
    out: &mut Vec<(i32, i32, [f32; 3])>,
) -> bool {
    if x < 0 || x >= world.width || y < 0 || y >= world.height {
        return false;
    }
    if world.tile(x, y).is_none() {
        return false;
    }
    if tile_is_open(world, x, y) {
        let dist = Vec2::new(x as f32, y as f32).distance(env.origin);
        let t = dist / env.range;
        let scale = 0.65 + (1.0 - 0.65) * t; // MathHelper.Lerp(0.65, 1, t)
        let rgb = [env.rgb[0] * scale, env.rgb[1] * scale, env.rgb[2] * scale];
        out.push((x, y, rgb)); // `:775`
        return true;
    }
    false
}

/// `DelegateMethods.SpreadLightOpen_StopForSolids`
/// (`DelegateMethods.cs:801-816`): the radius-gated spread. NOTE the C#
/// quirks kept verbatim — there is NO bounds check, and the condition at
/// `:808` is `tile == null || tile.active() || tile.inActive() ||
/// tileSolidTop || !tileSolid`: it lights a NULL tile or a SOLID/actuated/
/// solid-top/non-solid tile (the `active()` term is POSITIVE, unlike the
/// `!active()` open-test the other casts use). Returns false past the
/// radius or on an inactive plain-open tile.
pub(crate) fn spread_light_open_stop(
    world: &tr_core::World,
    x: i32,
    y: i32,
    env: &CastLightEnv,
    out: &mut Vec<(i32, i32, [f32; 3])>,
) -> bool {
    if Vec2::new(x as f32, y as f32).distance(env.origin) > env.range {
        return false; // `:803-806`
    }
    let lit = match world.tile(x, y) {
        None => true, // `tile == null`
        Some(t) => {
            t.active()
                || t.inactive()
                || tr_core::tiledata::TILE_SOLID_TOP.has(t.kind)
                || !tr_core::tiledata::TILE_SOLID.has(t.kind)
        }
    };
    if lit {
        out.push((x, y, env.rgb)); // `:812`
        return true;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn w4_e8_the_two_tile_anchors_differ_below_zero() {
        // `(int)(v/16f)` truncates the quotient; `(int)v >> 4` floors.
        let v = Vec2::new(-20.0, -20.0);
        assert_eq!(vector_tile(v), (-1, -1)); // -1.25 -> -1
        assert_eq!(shift_tile(v), (-2, -2)); // -20 >> 4 = -2
        let v = Vec2::new(20.0, 33.0);
        assert_eq!(vector_tile(v), (1, 2));
        assert_eq!(shift_tile(v), (1, 2));
    }

    #[test]
    fn w4_e8_the_christmas_hook_lights_four_rotating_colours() {
        // Hook centre (160, 160), mount (160, 320): the walk steps +16 y
        // per link; links at y=160/176/192/208 light tiles (10, 10..13)
        // with the 0/1/2/3 colours, then the index wraps.
        let mut out = Vec::new();
        christmas_hook_chain_lights(Vec2::new(160.0, 160.0), Vec2::new(160.0, 320.0), &mut out);
        // The walk exits when the remaining distance < 30: from 160 the
        // links are 160,176,192,208,224,240,256,272,288 (dist 32 at 288,
        // then 304 leaves 16 < 30). Nine links, colours wrap 0..3,0..3,0.
        assert_eq!(out.len(), 9);
        assert_eq!(out[0], (10, 10, [0.0, 0.2, 0.2]));
        assert_eq!(out[1], (10, 11, [0.1, 0.2, 0.0]));
        assert_eq!(out[2], (10, 12, [0.2, 0.1, 0.0]));
        assert_eq!(out[3], (10, 13, [0.2, 0.0, 0.2]));
        assert_eq!(out[4], (10, 14, [0.0, 0.2, 0.2]));
        assert_eq!(out[8], (10, 18, [0.0, 0.2, 0.2]));
    }

    #[test]
    fn w4_e8_the_christmas_hook_walk_is_rng_free_and_short_hooks_stay_dark() {
        // A hook closer than 30 px to the mount never lights (`:29002`).
        let mut out = Vec::new();
        christmas_hook_chain_lights(Vec2::new(160.0, 300.0), Vec2::new(160.0, 320.0), &mut out);
        assert!(out.is_empty());
    }

    #[test]
    fn w4_e8_the_illuminant_hook_lights_every_stepped_link() {
        // Chain height 24 (the Chains[2] sheet), centre (160,160), mount
        // (160,320): the gate runs BEFORE the step, the light AFTER it.
        let mut out = Vec::new();
        illuminant_hook_chain_lights(
            Vec2::new(160.0, 160.0),
            Vec2::new(160.0, 320.0),
            24.0,
            &mut out,
        );
        // Steps: 184,208,232,256,280,304 — at 304 the remaining dist is
        // 16 < 25, but the light at 304 was pushed on the PREVIOUS pass
        // (gate at 280: dist 40 >= 25 -> step to 304, light 304; next
        // gate dist 16 < 25 breaks). Six lights, all (0.2, 0, 0.175).
        assert_eq!(out.len(), 6);
        assert_eq!(out[0], (10, 11, [0.2, 0.0, 0.175])); // 184/16 = 11.5
        assert_eq!(out[5], (10, 19, [0.2, 0.0, 0.175])); // 304/16 = 19
        assert!(out.iter().all(|&(_, _, c)| c == [0.2, 0.0, 0.175]));
    }

    #[test]
    fn w4_e8_the_mushroom_flicker_draws_once_and_reads_the_mouse_colour() {
        // treeStyle 14, unpainted, mouseTextColor 255: num =
        // Next(28,42)*0.005 + (270-255)/1000. Pin the stream on a twin.
        let mut rng = FastRandom::new(7);
        let mut twin = FastRandom::new(7);
        let draw = twin.next_range(28, 42) as f32;
        let got = mushroom_tree_light(14, 0, 255, &mut rng).expect("style 14 lights");
        let num = draw * 0.005 + 15.0 / 1000.0;
        assert_eq!(got, [0.1, 0.2 + num / 2.0, 0.7 + num]);
        // The stream advanced exactly one draw.
        assert_eq!(rng.next_range(0, 1000), twin.next_range(0, 1000));
    }

    #[test]
    fn w4_e8_the_painted_mushroom_uses_the_paint_bytes_and_other_styles_skip() {
        let mut rng = FastRandom::new(7);
        let mut twin = FastRandom::new(7);
        // Painted: the draw still happens (the C# computes num before the
        // paint branch), but the colour is the paint table's.
        let got = mushroom_tree_light(14, 1, 255, &mut rng).expect("painted lights");
        assert_eq!(got, tr_core::tile_light::paint_color(1));
        assert_eq!(rng.next_range(0, 1000), twin.next_range(0, 1000));
        // Non-mushroom styles draw NOTHING and return None.
        let mut rng2 = FastRandom::new(7);
        assert!(mushroom_tree_light(0, 0, 255, &mut rng2).is_none());
        assert_eq!(rng2.next_range(0, 1000), twin.next_range(0, 1000));
    }
}
