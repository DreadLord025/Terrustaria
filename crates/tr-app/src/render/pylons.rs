//! S1-G-1 wave D: the pylon / master-trophy / void-lens special tile passes.
//!
//! C# `TileDrawing.PostDrawTiles(solidLayer: false)` (`TileDrawing.cs:414-434`)
//! draws the deferred special-point families after the tile layers; this module
//! ports three of them 1:1:
//! * `DrawTeleportationPylons` (`TileDrawing.cs:8639-8706`) — the pylon crystal
//!   (`TextureAssets.Extra[181]`) with its bobbing 6-copy glow ring,
//! * `DrawMasterTrophies` (`TileDrawing.cs:8597-8637`) — the boss relic
//!   (`TextureAssets.Extra[198]`) with the same ring, tinted by the tile light,
//! * `DrawVoidLenses` (`TileDrawing.cs:8708-8736` + `VoidLensHelper.cs:11-105`)
//!   — the void lens (`TextureAssets.Projectile[734]`) with its 3-copy purple
//!   ring and the `Lighting.AddLight(0.4, 0.2, 0.9)` source.
//!
//! The special points are collected by the `Draw` classification switch
//! (`TileDrawing.cs:695-712`): pylon 597 at `frameX % 54 == 0 && frameY == 0`,
//! trophy 617 at `frameX % 54 == 0 && frameY % 72 == 0`, void lens 491 at
//! `frameX == 18 && frameY == 18`. The port scans the view rect row-major (the
//! collection order) instead of carrying a registry — the same documented
//! approximation the inline grass sway uses (`tiles.rs`).
//!
//! Deliberate deviations (ANALYSIS "The pylon/trophy/void-lens passes"):
//! * the sprites ride the always-rebuilt world-effect range (screen space,
//!   like the leaf gores) so the G6 world-range cache is untouched; in the
//!   vanilla they draw between the non-solid and solid tile layers — the
//!   port's fused tile pass keeps the grass-style z-order approximation,
//! * the dust rolls (`TeleportPylonsSystem.SpawnInWorldDust`,
//!   `VoidLensHelper.SpawnVoidLensDust`) are skipped: they draw `Main.rand` /
//!   `TileDrawing._rand` and belong to the unported particle pass — skipping
//!   them wholesale leaves every RNG stream untouched,
//! * `Main.tileFrameCounter[597]/[491]` (`Main.cs:19284-19287`/`:19227-19230`,
//!   `AnimateTiles`' wrap-at-64/40 counters) are stood in by
//!   `game_update_count % 64` / `% 40` — the animator is unported
//!   (`drawdata.rs`'s `tile_frame ≡ 0` note), the cadence and wrap match,
//! * the smart-cursor selection glow arms are ported but the hook passes
//!   `selection_mode = 0`: `Main.InSmartCursorHighlightArea` is unported.

use glam::Vec2;
use tr_core::lighting::LightStyle;
use tr_core::world::World;
use tr_core::TILE_PX;
use tr_render::sprite::SpriteRenderer;

/// One collected special-tile sprite in world pixels: `pos` is the top-left
/// (the C# draws centre-anchored with `origin = rectangle.Size() / 2f`, so the
/// collectors subtract half the scaled size), `src` the source rect in sheet
/// pixels, `sheet` the atlas sheet name.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct SpecialSprite {
    pub pos: [f32; 2],
    pub size: [f32; 2],
    pub sheet: &'static str,
    pub src: [f32; 4],
    pub color: [f32; 4],
    pub flip: bool,
}

/// The per-frame inputs the three passes read off `Main`.
pub(crate) struct SpecialEnv {
    /// `Main.GlobalTimeWrappedHourly` (`anim_time % 3600`).
    pub global_time: f32,
    /// `Main.tileFrameCounter[597]` stand-in (`Main.cs:19284-19287`).
    pub pylon_counter: u32,
    /// `Main.tileFrameCounter[491]` stand-in (`Main.cs:19227-19230`).
    pub lens_counter: u32,
    /// `TileDrawing._shouldShowInvisibleBlocks` (`TileDrawing.cs:379`).
    pub show_invisible: bool,
}

/// The frame pixel sizes of the three sheets (`Utils.Frame`,
/// `Utils.cs:1344-1349`: `Width / horizontalFrames`, `Height /
/// verticalFrames`). The wrapper derives them from the atlas; the tests pin
/// them so the collectors stay GPU-free.
#[derive(Clone, Copy)]
pub(crate) struct FrameSizes {
    /// `Extra_181.Frame(14, 8, ..)` — one pylon crystal cell.
    pub pylon: [f32; 2],
    /// `Extra_198.Frame(1, 28, ..)` — one trophy row.
    pub trophy: [f32; 2],
    /// `Projectile_734.Frame(1, 8, ..)` — one lens frame.
    pub lens: [f32; 2],
}

// ---------------------------------------------------------------------------
// XNA byte-colour helpers. The C# colours here are `Color` structs: every
// `Lerp`/`operator*` step truncates back to bytes (`(byte)MathHelper.Lerp`,
// `(int)(c.R * scale)`), so the port quantises at each step, not once at the
// end — the intermediate truncation is visible in the glow tints.
// ---------------------------------------------------------------------------

/// `new Color(Vector3)` (`Color.cs`): `(int)(v * 255f)` per channel, clamped.
#[inline]
fn color_bytes(light: [f32; 3]) -> [u8; 3] {
    [
        (light[0].clamp(0.0, 1.0) * 255.0) as u8,
        (light[1].clamp(0.0, 1.0) * 255.0) as u8,
        (light[2].clamp(0.0, 1.0) * 255.0) as u8,
    ]
}

/// `Color.Lerp(a, b, amount)` — byte lerp, truncated per channel.
#[inline]
fn lerp_bytes(a: [u8; 3], b: [u8; 3], amount: f32) -> [u8; 3] {
    [
        (a[0] as f32 + (b[0] as f32 - a[0] as f32) * amount) as u8,
        (a[1] as f32 + (b[1] as f32 - a[1] as f32) * amount) as u8,
        (a[2] as f32 + (b[2] as f32 - a[2] as f32) * amount) as u8,
    ]
}

/// `Color operator*(Color, float)` — `(int)(c * scale)` per channel; alpha is
/// scaled the same way (the glow rings keep `A = 0`, which stays 0).
#[inline]
fn scale_bytes(c: [u8; 4], scale: f32) -> [u8; 4] {
    [
        (c[0] as f32 * scale) as u8,
        (c[1] as f32 * scale) as u8,
        (c[2] as f32 * scale) as u8,
        (c[3] as f32 * scale) as u8,
    ]
}

#[inline]
fn tint(c: [u8; 4]) -> [f32; 4] {
    [
        c[0] as f32 / 255.0,
        c[1] as f32 / 255.0,
        c[2] as f32 / 255.0,
        c[3] as f32 / 255.0,
    ]
}

/// `Colors.GetSelectionGlowColor` (`Colors.cs:89-92`): the smart-cursor
/// highlight tint — integer thirds/halves of the average tile lighting.
pub(crate) fn selection_glow_color(selected: bool, avg: i32) -> [f32; 4] {
    let a = avg as f32 / 255.0;
    if selected {
        [a, a, (avg / 3) as f32 / 255.0, a]
    } else {
        let h = (avg / 2) as f32 / 255.0;
        [h, h, h, a]
    }
}

/// `TileDrawing.IsVisible` (`TileDrawing.cs:1411-1432`): an Echo-coated block
/// hides unless sight is granted; platforms on row 48 and the two invisible
/// wiring tiles (541/631) always ride the flag. KEEP-ONE (batch-H compose):
/// thin wrapper over the promoted `tr_render::tile_visibility::is_visible`
/// (g31 TILEPASS final, research/136 §4 — the d56/d60 rebind target); the
/// g78 `tr_core::scene::tile_is_visible` copy is the byte-identical twin
/// (its sim-side callers keep it; the render passes ride this one).
fn is_visible(world: &World, x: i32, y: i32, tile: &tr_core::Tile, show_invisible: bool) -> bool {
    tr_render::tile_visibility::is_visible(
        tile.kind,
        tile.frame_y,
        world.invisible_block(x, y),
        show_invisible,
    )
}

/// `Point.ToWorldCoordinates(autoAddX, autoAddY)` (`Utils.cs`): `p * 16 + add`.
#[inline]
fn to_world(x: i32, y: i32, add_x: f32, add_y: f32) -> [f32; 2] {
    [x as f32 * TILE_PX + add_x, y as f32 * TILE_PX + add_y]
}

/// `Utils.ToRotationVector2` — `(cos f, sin f)`.
#[inline]
fn rotation_vector(f: f32) -> [f32; 2] {
    [f.cos(), f.sin()]
}

/// The classification-switch gate for the three families
/// (`TileDrawing.cs:695-712`).
#[inline]
fn special_kind(tile: &tr_core::Tile) -> Option<u16> {
    if !tile.active() {
        return None;
    }
    match tile.kind {
        491 if tile.frame_x == 18 && tile.frame_y == 18 => Some(491),
        597 if tile.frame_x % 54 == 0 && tile.frame_y == 0 => Some(597),
        617 if tile.frame_x % 54 == 0 && tile.frame_y % 72 == 0 => Some(617),
        _ => None,
    }
}

/// `DrawTeleportationPylons` (`TileDrawing.cs:8639-8706`), one pylon.
/// `selection_mode` is 0/1/2 (`InSmartCursorHighlightArea` + actuallySelected).
/// The dust roll (`:8669-8673`) is skipped — see the module docs.
pub(crate) fn pylon_sprites(
    world: &World,
    light_at: impl Fn(i32, i32) -> [f32; 3],
    style: &LightStyle,
    anim_time: f32,
    env: &SpecialEnv,
    sizes: &FrameSizes,
    x: i32,
    y: i32,
    selection_mode: u8,
) -> Vec<SpecialSprite> {
    let Some(tile) = world.tile(x, y).filter(|t| t.active()) else {
        return Vec::new();
    };
    // `:8651-8655`: the bottom-right cell must be active and visible.
    let Some(tile2) = world.tile(x + 1, y + 1).filter(|t| t.active()) else {
        return Vec::new();
    };
    if !is_visible(world, x + 1, y + 1, tile2, env.show_invisible) {
        return Vec::new();
    }
    let num3 = tile.frame_x as i32 / 54;
    // `:8661`: `frameY = (tileFrameCounter[597] + p.X + p.Y) % 64 / 8`.
    let frame_y = ((env.pylon_counter as i32 + x + y).rem_euclid(64)) / 8;
    let (fw, fh) = (sizes.pylon[0], sizes.pylon[1]);
    // `value.Frame(14, 8, 3 + num3, frameY)` (`:8662`); the selection mask is
    // column 2 (`value2`, `:8663`).
    let src = [(3 + num3) as f32 * fw, frame_y as f32 * fh, fw, fh];
    let src_sel = [2.0 * fw, frame_y as f32 * fh, fw, fh];
    // `:8666-8668`: `p.ToWorldCoordinates(24, 64) + (0, -40) + (0, sin*4)`.
    let bob = (env.global_time * (std::f32::consts::PI * 2.0) / 5.0).sin();
    let base = to_world(x, y, 24.0, 64.0);
    let center = [base[0], base[1] - 40.0 + bob * 4.0];
    // `:8674-8679`: `Lighting.GetColor(p.X, p.Y)`, white when the anchor cell
    // is Glow-coated, then `Lerp(_, White, 0.8)`.
    let mut c = color_bytes(style.shade(light_at(x, y), x, y, anim_time));
    if tile2.fullbright_block() {
        c = [255, 255, 255];
    }
    let value3 = lerp_bytes(c, [255, 255, 255], 0.8);
    let mut out = Vec::with_capacity(8);
    // `:8680`: the crystal at `value3 * 0.7f`, alpha 255.
    out.push(SpecialSprite {
        pos: [center[0] - fw / 2.0, center[1] - fh / 2.0],
        size: [fw, fh],
        sheet: "Extra_181",
        src: [src[0], src[1], src[0] + fw, src[1] + fh],
        color: tint(scale_bytes([value3[0], value3[1], value3[2], 255], 0.7)),
        flip: false,
    });
    // `:8681-8686`: the glow ring — `new Color(255,255,255,0) * 0.1f * num6`
    // (two truncating byte multiplies), six copies at `2πk/6`, radius
    // `6 + bob*2`. Alpha 0 = additive in the premultiplied pipeline.
    let num6 = (env.global_time * (std::f32::consts::PI * 2.0) / 1.0).sin() * 0.2 + 0.8;
    let ring = scale_bytes(scale_bytes([255, 255, 255, 0], 0.1), num6);
    let mut k = 0.0f32;
    while k < 1.0 {
        let rv = rotation_vector(std::f32::consts::PI * 2.0 * k);
        let r = 6.0 + bob * 2.0;
        out.push(SpecialSprite {
            pos: [
                center[0] + rv[0] * r - fw / 2.0,
                center[1] + rv[1] * r - fh / 2.0,
            ],
            size: [fw, fh],
            sheet: "Extra_181",
            src: [src[0], src[1], src[0] + fw, src[1] + fh],
            color: tint(ring),
            flip: false,
        });
        k += 1.0 / 6.0;
    }
    // `:8687-8704`: the smart-cursor selection glow (`value2`, column 2).
    if selection_mode != 0 {
        let avg = (value3[0] as i32 + value3[1] as i32 + value3[2] as i32) / 3;
        if avg > 10 {
            out.push(SpecialSprite {
                pos: [center[0] - fw / 2.0, center[1] - fh / 2.0],
                size: [fw, fh],
                sheet: "Extra_181",
                src: [src_sel[0], src_sel[1], src_sel[0] + fw, src_sel[1] + fh],
                color: selection_glow_color(selection_mode == 2, avg),
                flip: false,
            });
        }
    }
    out
}

/// `DrawMasterTrophies` (`TileDrawing.cs:8597-8637`), one trophy.
pub(crate) fn trophy_sprites(
    world: &World,
    light_at: impl Fn(i32, i32) -> [f32; 3],
    style: &LightStyle,
    anim_time: f32,
    env: &SpecialEnv,
    sizes: &FrameSizes,
    x: i32,
    y: i32,
) -> Vec<SpecialSprite> {
    let Some(tile) = world.tile(x, y).filter(|t| t.active()) else {
        return Vec::new();
    };
    let Some(tile2) = world.tile(x + 1, y + 1).filter(|t| t.active()) else {
        return Vec::new();
    };
    if !is_visible(world, x + 1, y + 1, tile2, env.show_invisible) {
        return Vec::new();
    }
    // `:8613-8615`: `frameY = tile.frameX / 54` on the 1×28 sheet; the flip
    // is `tile.frameY / 72 != 0` (the mirrored mount rows).
    let frame_y = tile.frame_x as i32 / 54;
    let flip = tile.frame_y as i32 / 72 != 0;
    let (fw, fh) = (sizes.trophy[0], sizes.trophy[1]);
    let src = [0.0, frame_y as f32 * fh, fw, (frame_y as f32 + 1.0) * fh];
    // `:8617-8619`: the same `(24, 64) + (0, -40) + (0, sin*4)` anchor.
    let bob = (env.global_time * (std::f32::consts::PI * 2.0) / 5.0).sin();
    let base = to_world(x, y, 24.0, 64.0);
    let center = [base[0], base[1] - 40.0 + bob * 4.0];
    // `:8620-8624`: the tile light, white under a Glow-coated anchor.
    let mut c = color_bytes(style.shade(light_at(x, y), x, y, anim_time));
    if tile2.fullbright_block() {
        c = [255, 255, 255];
    }
    let mut out = Vec::with_capacity(7);
    out.push(SpecialSprite {
        pos: [center[0] - fw / 2.0, center[1] - fh / 2.0],
        size: [fw, fh],
        sheet: "Extra_198",
        src,
        color: tint([c[0], c[1], c[2], 255]),
        flip,
    });
    // `:8627-8634`: the ring — `color2.A = 0; color2 * 0.1f * num4`, six
    // copies at radius `6 + bob*2`. Unlike the pylon's fixed white, the
    // trophy ring carries the tile light colour.
    let num4 = (env.global_time * (std::f32::consts::PI * 2.0) / 2.0).sin() * 0.3 + 0.7;
    let ring = scale_bytes(scale_bytes([c[0], c[1], c[2], 0], 0.1), num4);
    let mut k = 0.0f32;
    while k < 1.0 {
        let rv = rotation_vector(std::f32::consts::PI * 2.0 * k);
        let r = 6.0 + bob * 2.0;
        out.push(SpecialSprite {
            pos: [
                center[0] + rv[0] * r - fw / 2.0,
                center[1] + rv[1] * r - fh / 2.0,
            ],
            size: [fw, fh],
            sheet: "Extra_198",
            src,
            color: tint(ring),
            flip,
        });
        k += 1.0 / 6.0;
    }
    out
}

/// `VoidLensHelper.DrawToDrawData` (`VoidLensHelper.cs:77-104`), one lens.
/// The tile pass builds the helper with `opacity = 1f`
/// (`TileDrawing.cs:8716`); the dust/`AddLight` half of `Update`
/// (`:34-38`) is the hook's job (light) or skipped (dust).
pub(crate) fn void_lens_sprites(
    light_at: impl Fn(i32, i32) -> [f32; 3],
    style: &LightStyle,
    anim_time: f32,
    env: &SpecialEnv,
    sizes: &FrameSizes,
    x: i32,
    y: i32,
    selection_mode: u8,
) -> Vec<SpecialSprite> {
    // `VoidLensHelper.cs:26-32`: `p.ToWorldCoordinates()` (the 8,8 default),
    // `Y -= 2`, frame `(int)(((float)tileFrameCounter[491] + X + Y) % 40f) / 5`.
    let center = to_world(x, y, 8.0, 8.0 - 2.0);
    let frame = (((env.lens_counter as f32 + center[0] + center[1]) % 40.0) as i32) / 5;
    let (fw, fh) = (sizes.lens[0], sizes.lens[1]);
    let src = [0.0, frame as f32 * fh, fw, (frame as f32 + 1.0) * fh];
    // `:82-83`: `Lerp(GetColor(position.ToTileCoordinates()), White, 0.5)`,
    // `*= opacity`. `ToTileCoordinates` is `(int)v >> 4` (`Utils.cs`).
    let (tx, ty) = ((center[0] as i32) >> 4, (center[1] as i32) >> 4);
    let c = color_bytes(style.shade(light_at(tx, ty), tx, ty, anim_time));
    let color = scale_bytes(
        {
            let l = lerp_bytes(c, [255, 255, 255], 0.5);
            [l[0], l[1], l[2], 255]
        },
        1.0,
    );
    let mut out = Vec::with_capacity(5);
    out.push(SpecialSprite {
        pos: [center[0] - fw / 2.0, center[1] - fh / 2.0],
        size: [fw, fh],
        sheet: "Projectile_734",
        src,
        color: tint(color),
        flip: false,
    });
    // `:86-95`: the purple ring — `for (num = 0; num < 1; num += 0.34f)`
    // (three copies at f32 cadence), scale 1.1, colour
    // `(127, 50, 127, 0) * opacity * (x*0.1 + 0.3)` where
    // `x = cos(GlobalTimeWrappedHourly / 5 * 2π)`; the offset vector rotates
    // at `(t/5 + num) * 2π` with radius `x + 2`.
    let x = (env.global_time / 5.0 * (std::f32::consts::PI * 2.0)).cos();
    let mut num = 0.0f32;
    while num < 1.0 {
        let ring = scale_bytes(scale_bytes([127, 50, 127, 0], 1.0), x * 0.1 + 0.3);
        let rv = rotation_vector((env.global_time / 5.0 + num) * (std::f32::consts::PI * 2.0));
        let r = x * 1.0 + 2.0;
        let (sw, sh) = (fw * 1.1, fh * 1.1);
        out.push(SpecialSprite {
            pos: [
                center[0] + rv[0] * r - sw / 2.0,
                center[1] + rv[1] * r - sh / 2.0,
            ],
            size: [sw, sh],
            sheet: "Projectile_734",
            src,
            color: tint(ring),
            flip: false,
        });
        num += 0.34;
    }
    // `:96-103`: the selection glow on `Extra_93` (the highlight mask).
    if selection_mode != 0 {
        let avg = (color[0] as i32 + color[1] as i32 + color[2] as i32) / 3;
        if avg > 10 {
            out.push(SpecialSprite {
                pos: [center[0] - fw / 2.0, center[1] - fh / 2.0],
                size: [fw, fh],
                sheet: "Extra_93",
                src,
                color: selection_glow_color(selection_mode == 2, avg),
                flip: false,
            });
        }
    }
    out
}

/// The `VoidLensHelper.Update` light half (`VoidLensHelper.cs:34-38`):
/// `Lighting.AddLight(position, 0.4f, 0.2f, 0.9f)` — the Vector2 overload
/// tiles by `(int)(v / 16f)` (`Lighting.cs:140-143`).
pub(crate) fn void_lens_light(x: i32, y: i32) -> (i32, i32, [f32; 3]) {
    let center = to_world(x, y, 8.0, 6.0);
    (
        (center[0] / TILE_PX) as i32,
        (center[1] / TILE_PX) as i32,
        [0.4, 0.2, 0.9],
    )
}

/// The pass hook: scan the view rect (the `draw_tiles`/`draw_wires` shape),
/// collect the three families in the C# order — void lenses, pylons,
/// trophies (`PostDrawTiles`, `TileDrawing.cs:423-425`) — and push them into
/// the always-rebuilt world-effect range (screen space, like the leaf gores)
/// so the G6-cached world range is untouched. Returns the quad count.
#[allow(clippy::too_many_arguments)]
pub(crate) fn draw_pylons_trophies_void(
    r: &mut SpriteRenderer,
    world: &World,
    light: &tr_core::lighting::LightMap,
    style: &LightStyle,
    camera: Vec2,
    zoom: f32,
    screen: Vec2,
    anim_time: f32,
    env: &SpecialEnv,
    game_paused: bool,
    entity_lights: &mut crate::entity_lights::EntityLights,
) -> u32 {
    // The sheet frame sizes from the atlas (`Utils.Frame`'s integer divide).
    let sheet_px = |name: &str| -> Option<[f32; 2]> {
        let uv = r.sheets.get(name)?;
        Some([
            ((uv[2] - uv[0]) * r.atlas_size.0 as f32).round(),
            ((uv[3] - uv[1]) * r.atlas_size.1 as f32).round(),
        ])
    };
    let Some(pylon_sheet) = sheet_px("Extra_181") else {
        return 0;
    };
    let Some(trophy_sheet) = sheet_px("Extra_198") else {
        return 0;
    };
    let Some(lens_sheet) = sheet_px("Projectile_734") else {
        return 0;
    };
    let sizes = FrameSizes {
        pylon: [
            (pylon_sheet[0] as u32 / 14) as f32,
            (pylon_sheet[1] as u32 / 8) as f32,
        ],
        trophy: [trophy_sheet[0], (trophy_sheet[1] as u32 / 28) as f32],
        lens: [lens_sheet[0], (lens_sheet[1] as u32 / 8) as f32],
    };
    let half = screen * 0.5 / zoom;
    let min = camera - half;
    let max = camera + half;
    let x0 = ((min.x / TILE_PX).floor() as i32 - 1).clamp(0, world.width - 1);
    let x1 = ((max.x / TILE_PX).ceil() as i32 + 1).clamp(0, world.width - 1);
    let y0 = ((min.y / TILE_PX).floor() as i32 - 1).clamp(0, world.height - 1);
    let y1 = ((max.y / TILE_PX).ceil() as i32 + 1).clamp(0, world.height - 1);

    let mut sprites = Vec::new();
    for ty in y0..=y1 {
        for tx in x0..=x1 {
            let Some(tile) = world.tile(tx, ty) else {
                continue;
            };
            match special_kind(tile) {
                // `selection_mode = 0`: the smart cursor is unported.
                Some(491) => {
                    // `TileDrawing.cs:8717-8720`: `Update` (the light half)
                    // runs only when `!Main.gamePaused`; the source lands in
                    // the current list and the engine reads it one frame
                    // late, exactly like the item/NPC feeds.
                    if !game_paused {
                        let (lx, ly, rgb) = void_lens_light(tx, ty);
                        entity_lights.add(lx, ly, rgb);
                    }
                    sprites.extend(void_lens_sprites(
                        |lx, ly| light.get(lx, ly),
                        style,
                        anim_time,
                        env,
                        &sizes,
                        tx,
                        ty,
                        0,
                    ));
                }
                Some(597) => sprites.extend(pylon_sprites(
                    world,
                    |lx, ly| light.get(lx, ly),
                    style,
                    anim_time,
                    env,
                    &sizes,
                    tx,
                    ty,
                    0,
                )),
                Some(617) => sprites.extend(trophy_sprites(
                    world,
                    |lx, ly| light.get(lx, ly),
                    style,
                    anim_time,
                    env,
                    &sizes,
                    tx,
                    ty,
                )),
                _ => {}
            }
        }
    }
    let mut quads = 0u32;
    for s in sprites {
        let Some(uv) = r.uv(
            s.sheet,
            s.src[0],
            s.src[1],
            s.src[2] - s.src[0],
            s.src[3] - s.src[1],
        ) else {
            continue;
        };
        let uv = crate::render::player_layers::flip_uv(uv, s.flip);
        r.push_fx_sprite_rot(
            [
                (s.pos[0] - camera.x) * zoom + screen.x * 0.5,
                (s.pos[1] - camera.y) * zoom + screen.y * 0.5,
            ],
            [s.size[0] * zoom, s.size[1] * zoom],
            uv,
            s.color,
            0.0,
        );
        quads += 1;
    }
    quads
}

#[cfg(test)]
mod tests {
    use super::*;
    use tr_core::tile::FLAG_ACTIVE;

    const SIZES: FrameSizes = FrameSizes {
        pylon: [54.0, 62.0],
        trophy: [54.0, 70.0],
        lens: [40.0, 40.0],
    };

    fn env() -> SpecialEnv {
        SpecialEnv {
            global_time: 0.0,
            pylon_counter: 0,
            lens_counter: 0,
            show_invisible: false,
        }
    }

    fn style() -> LightStyle {
        // A pass-through style: brightness 1, no quantisation — the tests pin
        // the C# colour math, not the light-style transform.
        let mut s = LightStyle::for_mode(tr_core::lighting::LightMode::Color);
        s.brightness = 1.0;
        s
    }

    fn world_with(kind: u16, fx: i16, fy: i16) -> World {
        let mut w = World::new("test".into(), 64, 64);
        // The 3x4 object's cells: the anchor (10,10) plus the bottom-right
        // (11,11) the visibility gate reads.
        for (x, y) in [(10, 10), (11, 11)] {
            let t = w.tile_mut(x, y).unwrap();
            t.kind = kind;
            t.frame_x = fx;
            t.frame_y = fy;
            t.flags = FLAG_ACTIVE;
        }
        w
    }

    fn white_light(_: i32, _: i32) -> [f32; 3] {
        [1.0, 1.0, 1.0]
    }

    /// `TileDrawing.cs:695-712`: the classification gates.
    #[test]
    fn s1_g_1d_the_classification_gates_match_the_switch() {
        let mut t = tr_core::Tile {
            flags: FLAG_ACTIVE,
            ..Default::default()
        };
        t.kind = 597;
        t.frame_x = 54;
        t.frame_y = 0;
        assert_eq!(special_kind(&t), Some(597));
        t.frame_y = 18; // off the (0,0) row: no special point
        assert_eq!(special_kind(&t), None);
        t.kind = 617;
        t.frame_x = 108;
        t.frame_y = 72;
        assert_eq!(special_kind(&t), Some(617));
        t.frame_y = 36;
        assert_eq!(special_kind(&t), None);
        t.kind = 491;
        t.frame_x = 18;
        t.frame_y = 18;
        assert_eq!(special_kind(&t), Some(491));
        t.frame_x = 0;
        assert_eq!(special_kind(&t), None);
        t.flags = 0; // inactive tiles never collect
        t.frame_x = 18;
        assert_eq!(special_kind(&t), None);
    }

    /// `DrawTeleportationPylons` (`TileDrawing.cs:8656-8686`): the frame
    /// column/row, the bob anchor, the 0.8-white lerp × 0.7 tint and the
    /// six-copy additive ring.
    #[test]
    fn s1_g_1d_the_pylon_crystal_bobs_and_rings_six_copies() {
        let w = world_with(597, 54, 0); // style 1 -> column 3+1
        let mut e = env();
        e.pylon_counter = 9; // (9 + 10 + 10) % 64 / 8 = 29/8 = 3
        let s = pylon_sprites(&w, white_light, &style(), 0.0, &e, &SIZES, 10, 10, 0);
        assert_eq!(s.len(), 7, "one crystal + six ring copies");
        let crystal = &s[0];
        assert_eq!(crystal.sheet, "Extra_181");
        // Frame(14, 8, 4, 3): x = 4*54, y = 3*62.
        assert_eq!(crystal.src, [216.0, 186.0, 270.0, 248.0]);
        // sin(0) = 0: centre = (10*16+24, 10*16+64-40) = (184, 184).
        assert_eq!(crystal.pos, [184.0 - 27.0, 184.0 - 31.0]);
        // White light: Lerp(255,255,0.8) = 255, *0.7 -> (int)178.5 = 178.
        // XNA's `Color operator*(float)` scales the alpha too: A 255 -> 178.
        assert_eq!(
            crystal.color,
            [178.0 / 255.0, 178.0 / 255.0, 178.0 / 255.0, 178.0 / 255.0]
        );
        // The ring: 255*0.1 = 25 (truncated), *num6 = 0.8 -> 20, alpha 0.
        for ring in &s[1..] {
            assert_eq!(ring.color, [20.0 / 255.0, 20.0 / 255.0, 20.0 / 255.0, 0.0]);
        }
        // Copy k=0 sits at angle 0, radius 6 + sin(0)*2 = 6 to the right.
        assert_eq!(s[1].pos, [crystal.pos[0] + 6.0, crystal.pos[1]]);
        // The bob: at t = 1.25s the 5s sine is sin(π/2) = 1 -> +4px down.
        e.global_time = 1.25;
        let s2 = pylon_sprites(&w, white_light, &style(), 0.0, &e, &SIZES, 10, 10, 0);
        assert_eq!(s2[0].pos[1], crystal.pos[1] + 4.0);
        // ...and the ring radius grows to 6 + 2 = 8.
        assert_eq!(s2[1].pos[0], s2[0].pos[0] + 8.0);
    }

    /// `:8651-8655` + `IsVisible`: an inactive or Echo-coated anchor cell
    /// hides the crystal; the selection glow uses column 2 and the
    /// `Colors.GetSelectionGlowColor` thirds.
    #[test]
    fn s1_g_1d_the_pylon_gates_on_the_anchor_cell_and_selection() {
        let mut w = world_with(597, 0, 0);
        let mut e = env();
        // (44 + 10 + 10) % 64 = 0 -> frame row 0.
        e.pylon_counter = 44;
        // Echo coating on (11,11) hides the pylon...
        w.set_invisible_block(11, 11, true);
        assert!(pylon_sprites(&w, white_light, &style(), 0.0, &e, &SIZES, 10, 10, 0).is_empty());
        // ...unless sight is granted.
        let mut e2 = env();
        e2.show_invisible = true;
        assert_eq!(
            pylon_sprites(&w, white_light, &style(), 0.0, &e2, &SIZES, 10, 10, 0).len(),
            7
        );
        w.set_invisible_block(11, 11, false);
        // Selection mode 2 appends the column-2 mask with the selected tint:
        // avg = 255 -> (255, 255, 85, 255).
        let s = pylon_sprites(&w, white_light, &style(), 0.0, &e, &SIZES, 10, 10, 2);
        assert_eq!(s.len(), 8);
        let sel = s.last().unwrap();
        assert_eq!(sel.src, [108.0, 0.0, 162.0, 62.0]);
        assert_eq!(sel.color, [1.0, 1.0, 85.0 / 255.0, 1.0]);
        // Mode 1 halves the rgb: (127, 127, 127, 255).
        let s1 = pylon_sprites(&w, white_light, &style(), 0.0, &e, &SIZES, 10, 10, 1);
        assert_eq!(
            s1.last().unwrap().color,
            [127.0 / 255.0, 127.0 / 255.0, 127.0 / 255.0, 1.0]
        );
        // An inactive anchor cell hides everything.
        w.tile_mut(11, 11).unwrap().flags = 0;
        assert!(pylon_sprites(&w, white_light, &style(), 0.0, &e, &SIZES, 10, 10, 2).is_empty());
    }

    /// `DrawMasterTrophies` (`TileDrawing.cs:8612-8634`): the 1×28 row, the
    /// `frameY / 72` mirror flip, the light-tinted ring (not the pylon's
    /// fixed white) and the Glow-coating white override.
    #[test]
    fn s1_g_1d_the_trophy_row_flips_and_rings_in_the_tile_light() {
        let mut w = world_with(617, 108, 72); // row 2, mirrored
        let e = env();
        let dim = |_: i32, _: i32| [0.5f32, 0.25, 0.0];
        let s = trophy_sprites(&w, dim, &style(), 0.0, &e, &SIZES, 10, 10);
        assert_eq!(s.len(), 7);
        assert_eq!(s[0].sheet, "Extra_198");
        assert_eq!(s[0].src, [0.0, 140.0, 54.0, 210.0]);
        assert!(s[0].flip, "frameY 72 / 72 = 1 -> FlipHorizontally");
        // (int)(0.5*255) = 127, (int)(0.25*255) = 63.
        assert_eq!(s[0].color, [127.0 / 255.0, 63.0 / 255.0, 0.0, 1.0]);
        // Ring: 127*0.1 = 12, 63*0.1 = 6; num4 at t=0 is 0.7 ->
        // (int)(12*0.7) = 8, (int)(6*0.7) = 4; alpha stays 0.
        assert_eq!(s[1].color, [8.0 / 255.0, 4.0 / 255.0, 0.0, 0.0]);
        // The Glow-coated anchor paints white (base and ring).
        w.tile_mut(11, 11).unwrap().flags |= tr_core::tile::FLAG_FULLBRIGHT;
        let s2 = trophy_sprites(&w, dim, &style(), 0.0, &e, &SIZES, 10, 10);
        assert_eq!(s2[0].color, [1.0, 1.0, 1.0, 1.0]);
        // White ring: 255*0.1 = 25, *num4 (0.7) = (int)17.5 = 17.
        assert_eq!(s2[1].color, [17.0 / 255.0, 17.0 / 255.0, 17.0 / 255.0, 0.0]);
        // frameY 0 does not flip.
        w.tile_mut(10, 10).unwrap().frame_y = 0;
        let s3 = trophy_sprites(&w, dim, &style(), 0.0, &e, &SIZES, 10, 10);
        assert!(!s3[0].flip);
    }

    /// `VoidLensHelper` (`VoidLensHelper.cs:26-32`, `:77-95`): the -2px
    /// anchor, the float-mod frame, the 0.5-white lerp and the THREE-copy
    /// 1.1-scale purple ring at the 0.34f cadence.
    #[test]
    fn s1_g_1d_the_void_lens_frames_and_rings_three_copies() {
        let e = env();
        // centre = (10*16+8, 10*16+8-2) = (168, 166); (0+168+166)%40 = 334%40
        // = 14 -> frame 14/5 = 2.
        let s = void_lens_sprites(white_light, &style(), 0.0, &e, &SIZES, 10, 10, 0);
        assert_eq!(s.len(), 4, "one lens + three ring copies (0, .34, .68)");
        assert_eq!(s[0].sheet, "Projectile_734");
        assert_eq!(s[0].src, [0.0, 80.0, 40.0, 120.0]);
        assert_eq!(s[0].pos, [168.0 - 20.0, 166.0 - 20.0]);
        // White light: Lerp(255,255,0.5) = 255, *1f keeps 255.
        assert_eq!(s[0].color, [1.0, 1.0, 1.0, 1.0]);
        // t=0: x = cos(0) = 1 -> ring scale (1*0.1+0.3) = 0.4:
        // (int)(127*0.4) = 50, (int)(50*0.4) = 20, alpha 0.
        assert_eq!(s[1].color, [50.0 / 255.0, 20.0 / 255.0, 50.0 / 255.0, 0.0]);
        // Scale 1.1 and radius x+2 = 3 at angle 0.
        assert_eq!(s[1].size, [44.0, 44.0]);
        assert_eq!(s[1].pos, [168.0 + 3.0 - 22.0, 166.0 - 22.0]);
        // The counter rides the frame: (26+334)%40 = 0 -> frame 0.
        let mut e2 = env();
        e2.lens_counter = 26;
        let s2 = void_lens_sprites(white_light, &style(), 0.0, &e2, &SIZES, 10, 10, 0);
        assert_eq!(s2[0].src, [0.0, 0.0, 40.0, 40.0]);
        // Selection mode 1 appends the Extra_93 mask, avg 255 -> halves.
        let s3 = void_lens_sprites(white_light, &style(), 0.0, &e, &SIZES, 10, 10, 1);
        assert_eq!(s3.len(), 5);
        assert_eq!(s3[4].sheet, "Extra_93");
        assert_eq!(
            s3[4].color,
            [127.0 / 255.0, 127.0 / 255.0, 127.0 / 255.0, 1.0]
        );
    }

    /// `VoidLensHelper.Update` (`:34-38`): the light source tiles by
    /// `(int)(v/16)` and carries the fixed purple.
    #[test]
    fn s1_g_1d_the_void_lens_lights_its_tile() {
        assert_eq!(void_lens_light(10, 10), (10, 10, [0.4, 0.2, 0.9]));
        // The -2px nudge never crosses a tile boundary at the 8,8 anchor.
        assert_eq!(void_lens_light(0, 0), (0, 0, [0.4, 0.2, 0.9]));
    }

    /// `Colors.GetSelectionGlowColor` (`Colors.cs:89-92`): integer halves and
    /// thirds, alpha = the average.
    #[test]
    fn s1_g_1d_the_selection_glow_uses_integer_division() {
        let c = selection_glow_color(false, 101);
        assert_eq!(c, [50.0 / 255.0, 50.0 / 255.0, 50.0 / 255.0, 101.0 / 255.0]);
        let c2 = selection_glow_color(true, 101);
        assert_eq!(
            c2,
            [101.0 / 255.0, 101.0 / 255.0, 33.0 / 255.0, 101.0 / 255.0]
        );
    }

    /// The byte helpers truncate at every step (XNA `Color` semantics), not
    /// once at the end: `255 * 0.1f * 0.8f` is 20, not `(int)(255*0.08)` = 20
    /// — but `127 * 0.1f * 0.7f` is 8, while the fused product gives 8.89->8
    /// too; the visible case is `63 * 0.1f * 0.7f`: (int)6.3 = 6, (int)4.2 = 4
    /// vs the fused (int)(63*0.07) = 4 — and `25*0.7` = 17 vs 17.5 fused.
    #[test]
    fn s1_g_1d_the_byte_math_truncates_per_step() {
        assert_eq!(scale_bytes([255, 255, 255, 0], 0.1), [25, 25, 25, 0]);
        assert_eq!(scale_bytes([25, 25, 25, 0], 0.7), [17, 17, 17, 0]);
        assert_eq!(lerp_bytes([0, 0, 0], [255, 255, 255], 0.8), [204, 204, 204]);
        // (int)(0.8*255) = 204 exactly; a 0.5 lerp of 127 -> 191.
        assert_eq!(
            lerp_bytes([127, 127, 127], [255, 255, 255], 0.5),
            [191, 191, 191]
        );
    }
}
