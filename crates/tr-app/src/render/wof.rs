//! `Main.DrawWoF` (`Main.cs:38377-38398`) — the Wall of Flesh's dedicated
//! draw pass, W4-d127. The C# runs it AFTER `DoDraw_WallsAndBlacks` and
//! BEFORE the behind-tiles NPC pass (`Main.cs:63271-63279`, the capture
//! path `:56927-56933`); the port's fused tile pass draws the walls then
//! the behind-tiles NPCs, so the seam sits between those two sites
//! (`tiles.rs`, the `npcs_behind_tiles` phase).
//!
//! Three pieces, in the C#'s order:
//! 1. `DrawWOFTongueToPlayer` (`:38498-38525`) — every active, tongued,
//!    living player gets a Chain12 link chain from its FLOAT centre to the
//!    wall's INTEGER-half centre; the links step `Chain12.Height()` (28 px)
//!    at a time, the last partial link dropped (`num5 < 40f`), scale 1,
//!    rotation `Atan2(dy, dx) - 1.57f` about the link centre.
//! 2. `DrawWOFRopeToTheHungry` (`:38448-38496`) — every live aiStyle-29
//!    NPC (the Hungry, 115) gets a chain from its centre-X / scaled
//!    half-height anchor to the wall's band anchor
//!    (`wofDrawAreaTop + (bottom - top) * ai[0]`, `:38458` — the SAME
//!    anchor the Hungry's AI chases, `NPC.cs:26283`). The link step is
//!    `28f * scale`, the source rect's height truncates to
//!    `(int)dist - (int)(40f * scale) + 28` on the final link (`:38483`),
//!    the width is `Chain4.Width()` (16 — Chain4 and Chain12 are both
//!    16x28, measured), the flip alternates per link starting from
//!    `frameCounter > 7` (`:38454`/`:38469-38477`).
//! 3. `DrawWOFBody` (`:38400-38446`) — the mouth sweep: the
//!    `TextureAssets.Wof` sheet (190x420, measured; `num = Height() / 3`
//!    = 140-px rows) tiles from the band top to the SCREEN bottom in
//!    16-px slices, the row picked by `wofDrawFrameIndex / 6` (the 18-tick
//!    draw clock, `:38426-38430`), the column at the wall's X (−80 when
//!    `direction > 0`, `:38417-38420`), flipped on `spriteDirection == 1`
//!    (`:38422-38425`). The `num4 > 12f` early-out (`:38407`) skips the
//!    body when the band top sits more than 12 rows below the screen top.
//!
//! RNG-free by construction: the whole pass is deterministic geometry +
//! the draw clock. The clock advances HERE (`!gamePaused`, `:38427`) —
//! the port's stand-in is the frame-driven counter in `DrawScratch`
//! (the `wofDrawFrameIndex` static, `Main.cs:761`), fed through
//! `WorldFrameKey.wof_key` so the G6-cached world range rebuilds while a
//! wall lives.
//!
//! Faithfulness notes (documented deviations):
//! * `Lighting.GetColor` is the port's `light.get` + `style.shade` (the
//!   NPC pass's convention); the tile X is the C#'s INTEGER
//!   `(int)(num6 + (float)(Width() / 2)) / 16` (`:38441` — `Width()/2`
//!   is integer division), the link tints use `(int)vector.X / 16` and
//!   `(int)(vector.Y / 16f)` (`:38493`/`:38522`).
//! * The C# draws at `position - screenPosition`; the port's world-range
//!   pipeline draws in world pixels with the `snap_to_screen` rounding
//!   (the NPC pass's convention). The link quads rotate about their
//!   centres (`push_sprite_origin_rot`, origin = half the SCALED size —
//!   XNA's `origin` is in source pixels, multiplied by `scale`).
//! * `netOffset` (`:38461`) rides the Hungry's `offset_clone` like the
//!   NPC pass (`Main.cs:22335`); zero on the host.
//! * The sheet handles come from the atlas (`WallOfFlesh`, `Chain12`,
//!   `Chain4` — packed by `assets.rs`'s static list); a missing sheet
//!   skips its piece (`SpriteBatch.Draw` ignores null textures).

use super::BgTex;
use super::entities::snap_to_screen_pub;
use super::player_layers::flip_uv;
use crate::npc::{Npc, NpcKind};
use crate::player::Player;
use tr_core::lighting::{LightMap, LightStyle};
use tr_core::World;
use tr_render::SpriteRenderer;

/// The pass's sheet handles (`TextureAssets.Wof`/`Chain12`/`Chain4`).
#[derive(Clone, Copy)]
pub(crate) struct WofSheets {
    /// `TextureAssets.Wof` — the 190x420 body strip (three 140-px rows).
    pub body: Option<BgTex>,
    /// `TextureAssets.Chain12` — the 16x28 link (rope + tongue).
    pub chain12: Option<BgTex>,
    /// `TextureAssets.Chain4` — the rope's source-rect WIDTH (16).
    pub chain4: Option<BgTex>,
}

/// The wall's draw statics the pass reads (`Main.wofDrawArea*` + the
/// wall NPC's own fields through the sim's mirror).
pub(crate) struct WofDrawCtx<'a> {
    pub world: &'a World,
    pub npcs: &'a [Npc],
    /// The session player (the tongue's `Main.player[i]` scan is the one
    /// slot; `active` = present, `tongued`/`dead` gate the chain).
    pub player: Option<&'a Player>,
    /// `Main.wofDrawFrameIndex` (`Main.cs:761`) — the 18-tick mouth clock.
    pub frame_index: i32,
}

/// `Main.DrawWoF` (`Main.cs:38377-38398`), one frame. Returns the quad
/// count pushed (the tile pass's stats).
pub(crate) fn draw_wof(
    r: &mut SpriteRenderer,
    sheets: &WofSheets,
    ctx: &WofDrawCtx<'_>,
    light: &LightMap,
    style: &LightStyle,
    anim_time: f32,
    camera: glam::Vec2,
    zoom: f32,
    screen: glam::Vec2,
) -> u32 {
    // `:38379-38382` — no living wall: the whole pass returns.
    if ctx.world.wof_npc_index < 0 || !ctx.world.wof_present || ctx.world.wof_life <= 0 {
        return 0;
    }
    // The wall NPC itself (the body's X/direction/spriteDirection reads,
    // `:38416-38425`). The sim's mirror carries the index; the pool scan
    // finds the slot (the compacting pool's index IS the mirror's).
    let Some(wall) = ctx.npcs.iter().find(|n| n.kind == NpcKind::WallOfFlesh && n.hp > 0) else {
        return 0;
    };
    let mut quads = 0;
    // `:38383-38389` — the tongue chains (active && tongued && !dead).
    if let Some(p) = ctx.player {
        if p.tongued && !p.dead {
            quads += draw_tongue_to_player(r, sheets, wall, p, light, style, anim_time, camera, zoom, screen);
        }
    }
    // `:38390-38396` — the Hungry ropes (every live aiStyle-29 slot).
    for n in ctx.npcs {
        if n.hp > 0 && n.kind.ai_style() == 29 {
            quads += draw_rope_to_the_hungry(r, sheets, ctx, wall, n, light, style, anim_time, camera, zoom, screen);
        }
    }
    // `:38397` — the body sweep LAST (the C#'s order).
    quads += draw_wof_body(r, sheets, ctx, wall, light, style, anim_time, camera, zoom, screen);
    quads
}

/// The link-chain walk shared by the rope and the tongue: the C#'s
/// `while (flag2)` steps `vector` toward the anchor by `step` px per link
/// (`num6 = step / dist`, `:38486-38490`/`:38515-38519`), drawing one
/// Chain12 quad per step. `final_height` computes the last link's source
/// height (the rope truncates, the tongue drops the partial link).
#[allow(clippy::too_many_arguments)]
fn chain_walk(
    r: &mut SpriteRenderer,
    chain12: &BgTex,
    chain4_w: f32,
    mut vx: f32,
    mut vy: f32,
    anchor_x: f32,
    anchor_y: f32,
    rotation: f32,
    scale: f32,
    step: f32,
    // The rope's `(int)(40f * scale)` terminal threshold; the tongue's 40.
    threshold: f32,
    // true = the rope (draw the truncated final link); false = the tongue
    // (drop it, `:38510-38514`'s `continue`).
    draw_final: bool,
    mut flip: bool,
    // The rope alternates the flip per link (`:38469-38477`); the tongue
    // draws every link `SpriteEffects.None` (`:38523`).
    alternate_flip: bool,
    light: &LightMap,
    style: &LightStyle,
    anim_time: f32,
    camera: glam::Vec2,
    zoom: f32,
    screen: glam::Vec2,
) -> u32 {
    let mut quads = 0;
    let mut dx = anchor_x - vx;
    let mut dy = anchor_y - vy;
    loop {
        let dist = (dx * dx + dy * dy).sqrt();
        let mut height = 28.0f32;
        if dist < threshold {
            if !draw_final {
                break; // the tongue drops the partial link
            }
            // `:38483` — `height = (int)num6 - num5 + 28` (the rope's
            // final link truncates to the remaining span).
            height = dist as i32 as f32 - threshold + 28.0;
        }
        // `:38486-38490` — the step toward the anchor.
        let k = step / dist;
        dx *= k;
        dy *= k;
        vx += dx;
        vy += dy;
        dx = anchor_x - vx;
        dy = anchor_y - vy;
        // `:38493` — the link tint at its own tile.
        let tx = vx as i32 / 16;
        let ty = (vy / 16.0) as i32;
        let c = style.shade(light.get(tx, ty), tx, ty, anim_time);
        let tint = [c[0], c[1], c[2], 1.0];
        // The source rect: `(0, 0, Chain4.Width(), height)` (`:38494`) —
        // the top `height` px of the 16x28 link, flipped per link.
        let uv = flip_uv(
            [
                chain12.uv[0],
                chain12.uv[1],
                chain12.uv[0] + (chain12.uv[2] - chain12.uv[0]) * (chain4_w / chain12.w),
                chain12.uv[1] + (chain12.uv[3] - chain12.uv[1]) * (height / chain12.h),
            ],
            flip,
        );
        if alternate_flip {
            flip = !flip; // `:38469-38477` — the rope's alternating flip
        }
        // XNA: `Draw(vector, rect, color, rotation, origin = (Chain4.W/2,
        // Chain4.H/2), scale)` — the origin is the FULL link's centre
        // (14 px down), in source pixels, scaled.
        let size = [chain4_w * scale, height * scale];
        let origin = [chain4_w * 0.5 * scale, (chain12.h * 0.5) * scale];
        let pos = snap_to_screen_pub([vx, vy], camera, zoom, screen);
        r.push_sprite_origin_rot(pos, size, uv, tint, rotation, origin);
        quads += 1;
        if dist < threshold {
            break; // the final link drawn (rope) — the walk ends
        }
    }
    quads
}

/// `Main.DrawWOFRopeToTheHungry` (`Main.cs:38448-38496`).
#[allow(clippy::too_many_arguments)]
fn draw_rope_to_the_hungry(
    r: &mut SpriteRenderer,
    sheets: &WofSheets,
    ctx: &WofDrawCtx<'_>,
    wall: &Npc,
    hungry: &Npc,
    light: &LightMap,
    style: &LightStyle,
    anim_time: f32,
    camera: glam::Vec2,
    zoom: f32,
    screen: glam::Vec2,
) -> u32 {
    let (Some(chain12), Some(chain4)) = (sheets.chain12, sheets.chain4) else {
        return 0;
    };
    // `:38450-38451` — the wall anchor X: position.X + (float)(width/2)
    // (the INTEGER half-width).
    let (ww, _) = wall.size();
    let anchor_x = wall.x + (ww as i32 / 2) as f32;
    // `:38452`/`:38458` — the band anchor Y.
    let band = ctx.world.wof_draw_bottom - ctx.world.wof_draw_top;
    let anchor_y = ctx.world.wof_draw_top as f32 + band as f32 * hungry.ai[0];
    // `:38454-38457` — the first link's flip: frameCounter > 7.
    let flip = hungry.frame_counter > 7.0;
    // `:38459-38461` — the Hungry anchor: Centre.X, Bottom.Y -
    // height * 0.5 * scale (+ netOffset).
    let n_owned = hungry.offset_clone();
    let n: &Npc = if hungry.net_offset == glam::Vec2::ZERO { hungry } else { &n_owned };
    let (hw, hh) = n.size();
    let scale = n.scale;
    let vx = n.x + hw * 0.5;
    let vy = n.y + hh - hh * 0.5 * scale;
    // `:38462-38464` — the rotation rides the FIRST delta.
    let rotation = (anchor_y - vy).atan2(anchor_x - vx) - 1.57;
    chain_walk(
        r, &chain12, chain4.w, vx, vy, anchor_x, anchor_y, rotation, scale,
        28.0 * scale,       // `:38486` — the step is 28 * scale
        (40.0 * scale) as i32 as f32, // `:38479` — num5 = (int)(40f * scale)
        true,               // the rope draws the truncated final link
        flip,
        true,               // the rope alternates the flip per link
        light, style, anim_time, camera, zoom, screen,
    )
}

/// `Main.DrawWOFTongueToPlayer` (`Main.cs:38498-38525`).
#[allow(clippy::too_many_arguments)]
fn draw_tongue_to_player(
    r: &mut SpriteRenderer,
    sheets: &WofSheets,
    wall: &Npc,
    player: &Player,
    light: &LightMap,
    style: &LightStyle,
    anim_time: f32,
    camera: glam::Vec2,
    zoom: f32,
    screen: glam::Vec2,
) -> u32 {
    let Some(chain12) = sheets.chain12 else {
        return 0;
    };
    // `:38500-38501` — the wall centre, INTEGER halves on both axes.
    let (ww, wh) = wall.size();
    let anchor_x = wall.x + (ww as i32 / 2) as f32;
    let anchor_y = wall.y + (wh as i32 / 2) as f32;
    // `:38502` — the player centre, FLOAT halves.
    let vx = player.x + player.width * 0.5;
    let vy = player.y + player.height * 0.5;
    let rotation = (anchor_y - vy).atan2(anchor_x - vx) - 1.57;
    // The tongue: scale 1, step Chain12.Height() (28), threshold 40, the
    // partial link dropped, no flip (`SpriteEffects.None`, `:38523`).
    chain_walk(
        r, &chain12, chain12.w, vx, vy, anchor_x, anchor_y, rotation, 1.0,
        chain12.h,          // `:38515` — the step is Chain12.Height()
        40.0,               // `:38510`
        false,              // the tongue drops the partial link
        false,              // SpriteEffects.None (`:38523`)
        false,              // no flip alternation
        light, style, anim_time, camera, zoom, screen,
    )
}

/// `Main.DrawWOFBody` (`Main.cs:38400-38446`) — the mouth sweep.
#[allow(clippy::too_many_arguments)]
fn draw_wof_body(
    r: &mut SpriteRenderer,
    sheets: &WofSheets,
    ctx: &WofDrawCtx<'_>,
    wall: &Npc,
    light: &LightMap,
    style: &LightStyle,
    anim_time: f32,
    camera: glam::Vec2,
    zoom: f32,
    screen: glam::Vec2,
) -> u32 {
    let Some(body) = sheets.body else {
        return 0;
    };
    // `:38402` — num = Height() / 3 (the 140-px row).
    let row_h = (body.h as i32 / 3) as f32;
    // `:38403-38405` — the band top; the bottom is the SCREEN bottom
    // (`screenPosition.Y + screenHeight` — the port's view rect).
    let top = ctx.world.wof_draw_top as f32;
    let view_bottom = camera.y + screen.y * 0.5 / zoom;
    let view_top = camera.y - screen.y * 0.5 / zoom;
    // `:38406-38415` — the row alignment (the pure helper below).
    let Some(mut top) = body_row_start(top, view_top, row_h) else {
        return 0; // num4 > 12: the body is off-screen below
    };
    // `:38416-38420` — the column X (−80 when direction > 0).
    let mut col_x = wall.x;
    if wall.facing > 0 {
        col_x -= 80.0;
    }
    // `:38421-38425` — the flip: spriteDirection == 1.
    let flip = wall.sprite_dir == 1;
    // `:38426` — the mouth row: wofDrawFrameIndex / 6 * num.
    let row_y = (ctx.frame_index / 6) as f32 * row_h;
    // `:38432-38445` — the 16-px slice walk from the band top to the
    // screen bottom.
    let mut quads = 0;
    let mut i = top as i32 as f32;
    while i < view_bottom {
        // `:38434-38438` — num8 = min(screenBottom - i, num).
        let span = (view_bottom - i).min(row_h);
        let mut j = 0.0f32;
        while j < span {
            // `:38441-38442` — the tint tile: the column's INTEGER centre
            // X, the slice's Y.
            let tx = (col_x + (body.w as i32 / 2) as f32) as i32 / 16;
            let ty = (i + j) as i32 / 16;
            let c = style.shade(light.get(tx, ty), tx, ty, anim_time);
            let tint = [c[0], c[1], c[2], 1.0];
            // `:38443` — the source rect `(0, num7 + j, Width(), 16)`.
            let sy = row_y + j;
            let uv = flip_uv(
                [
                    body.uv[0],
                    body.uv[1] + (body.uv[3] - body.uv[1]) * (sy / body.h),
                    body.uv[2],
                    body.uv[1] + (body.uv[3] - body.uv[1]) * ((sy + 16.0) / body.h),
                ],
                flip,
            );
            let pos = snap_to_screen_pub([col_x, i + j], camera, zoom, screen);
            r.push_sprite(pos, [body.w, 16.0], uv, tint);
            quads += 1;
            j += 16.0;
        }
        i += row_h;
    }
    quads
}

/// `DrawWOFBody`'s row alignment (`Main.cs:38406-38415`): `num4 =
/// (int)((top - screenTop) / row) + 1`; `> 12` → `None` (the body is more
/// than 12 rows below the screen top, the early-out). Otherwise `num5 =
/// num4 * row`; `> 0` → `top -= num5` (the sweep starts one row above the
/// band top's row). Pure so the witness can pin the truncation.
fn body_row_start(top: f32, view_top: f32, row_h: f32) -> Option<f32> {
    let num4 = ((top - view_top) / row_h) as i32 + 1;
    if num4 > 12 {
        return None;
    }
    let num5 = num4 as f32 * row_h;
    Some(if num5 > 0.0 { top - num5 } else { top })
}

/// `Main.wofDrawFrameIndex`'s tick (`Main.cs:38427-38430`): advance unless
/// paused, wrap at 18. The caller runs it once per frame BEFORE the pass
/// (the C# increments inside the body draw; the port's counter lives in
/// `DrawScratch` and feeds the key, so it advances ahead of the draw).
pub(crate) fn tick_wof_frame_index(index: &mut i32, game_paused: bool) {
    if !game_paused {
        *index += 1;
        if *index >= 18 {
            *index = 0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The mouth clock: 18 ticks wrap, the row picks every 6
    /// (`Main.cs:38426-38430`), paused frames hold.
    #[test]
    fn the_mouth_clock_wraps_at_18_and_pauses() {
        let mut idx = 0;
        let mut rows = Vec::new();
        for _ in 0..18 {
            tick_wof_frame_index(&mut idx, false);
            rows.push(idx / 6);
        }
        assert_eq!(idx, 0, "the 18th tick wraps to 0");
        // Ticks 1..6 → row 0 (idx 1..5 → 0, idx 6 → 1): the row changes
        // on the 6th/12th ticks, exactly `wofDrawFrameIndex / 6`.
        assert_eq!(rows[5], 1, "the 6th tick picks row 1");
        assert_eq!(rows[11], 2, "the 12th tick picks row 2");
        assert_eq!(rows[17], 0, "the wrap returns to row 0");
        let held = idx;
        tick_wof_frame_index(&mut idx, true);
        assert_eq!(idx, held, "paused frames hold the clock");
    }

    /// The rope's final-link truncation (`Main.cs:38479-38484`):
    /// `height = (int)dist - (int)(40 * scale) + 28`. At scale 1 a 50-px
    /// span's last link is 38 px; the walk's step is 28.
    #[test]
    fn the_rope_final_link_truncates_to_the_remaining_span() {
        let scale = 1.0f32;
        let threshold = (40.0 * scale) as i32 as f32;
        let dist = 50.0f32;
        let height = dist as i32 as f32 - threshold + 28.0;
        assert_eq!(height, 38.0);
        // The first link steps 28 px, leaving 22 < 40 → the final link's
        // height is (int)22 - 40 + 28 = 10.
        let remaining = dist - 28.0 * scale;
        let final_h = remaining as i32 as f32 - threshold + 28.0;
        assert_eq!(final_h, 10.0);
    }

    /// The body's row alignment (`Main.cs:38406-38415`): the band top's
    /// row index truncates toward zero, the sweep starts one row above it,
    /// and > 12 rows below the screen top is the early-out.
    #[test]
    fn the_body_row_alignment_truncates_and_gates() {
        let row = 140.0f32;
        // Band top exactly on the screen top: num4 = 0 + 1 = 1, top -= 140.
        assert_eq!(body_row_start(1000.0, 1000.0, row), Some(860.0));
        // Band top 150 px below: (int)(150/140) = 1, num4 = 2, top -= 280.
        assert_eq!(body_row_start(1150.0, 1000.0, row), Some(870.0));
        // Band top ABOVE the screen top: (int)(-50/140) = 0 (trunc toward
        // zero), num4 = 1, top -= 140.
        assert_eq!(body_row_start(950.0, 1000.0, row), Some(810.0));
        // 12 rows below is still drawn (num4 = 13? no: (int)(1680/140)=12,
        // num4 = 13 > 12 → None). 11 rows: num4 = 12, drawn.
        assert_eq!(body_row_start(1000.0 + 11.0 * row, 1000.0, row), Some(1000.0 - row));
        assert_eq!(body_row_start(1000.0 + 12.0 * row, 1000.0, row), None);
    }

    /// The pass gate (`Main.cs:38379-38382`): the living-wall check precedes
    /// any draw, and the three pieces run in the C#'s order (tongue, rope,
    /// body). Source-inspection witness (the render passes' convention —
    /// no GPU in the test harness).
    #[test]
    fn the_pass_gates_on_a_living_wall_and_orders_the_pieces() {
        let raw = include_str!("wof.rs");
        let src = raw.replace("\r\n", "\n");
        let body = &src[src.find("pub(crate) fn draw_wof(").unwrap()..];
        let body = &body[..body.find("\n}\n").unwrap()];
        let gate = body.find("wof_npc_index < 0").expect("the wall gate");
        let tongue = body.find("draw_tongue_to_player(").expect("the tongue");
        let rope = body.find("draw_rope_to_the_hungry(").expect("the rope");
        let sweep = body.find("draw_wof_body(").expect("the body");
        assert!(gate < tongue, "the gate precedes every piece (:38379)");
        assert!(tongue < rope, "the tongue draws before the ropes (:38383-38396)");
        assert!(rope < sweep, "the body sweep draws LAST (:38397)");
        // The rope keys on aiStyle 29 (the C#'s `Main.npc[j].aiStyle == 29`,
        // `:38392`), not the kind — every style-29 slot gets a leash.
        assert!(body.contains("ai_style() == 29"), "the rope's aiStyle-29 gate");
        // The tongue gates on tongued && !dead (`:38385`).
        assert!(body.contains("p.tongued && !p.dead"), "the tongue gate");
    }
}
