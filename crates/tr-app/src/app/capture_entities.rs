//! S1-G-13 consumers — `Main.DrawCapture`'s `captureEntities` arms
//! (`Main.cs:56876-57017`): the entity draws the chunk render interleaves
//! with the tile/wall/liquid composites. The C# splits them across a dozen
//! `DrawCachedNPCs`/`DrawCachedProjs` buckets; the port has no draw-cache
//! split, so the buckets collapse into the single `draw_npcs(behind_tiles)`
//! / projectile-slot passes at the C# arm positions (the full census lives
//! in ANALYSIS "The DrawCapture entity arms").
//!
//! The tail rides the chunk camera at zoom 1 into the 2048² framebuffer.
//! The world-range passes (NPCs/player/items) render through
//! `render_world_only`; the screen-space passes (projectiles/rain/gore/
//! dust — the port's `push_ui_*`/`push_fx_*` ranges) render through a
//! world+fx+ui composite with the back range parked (the C# draws them all
//! through the one `Main.spriteBatch` under `Main.Transform`).

use glam::Vec2;
use tr_render::SpriteRenderer;

use crate::app::capture_render::CaptureWorld;

/// The entity inputs the capture tail draws — the App fields the capture
/// driver snapshots for the chunk render. `capture_entities` gates every
/// arm except the unconditional behind-tiles NPC pass (`Main.cs:56951`).
pub(crate) struct CaptureEntities<'a> {
    pub player: Option<&'a crate::player::Player>,
    /// `DrawPlayer_27_HeldItem`'s input (kind, use fraction, prefix).
    pub held: Option<(u16, f32, u16)>,
    pub armor: crate::render::entities::ArmorKinds,
    pub npcs: &'a [crate::npc::Npc],
    pub items: &'a [crate::item::ItemDrop],
    pub projectiles: &'a crate::projectile::ProjectilePool,
    pub rain: &'a crate::rain::Rain,
    // The gore pool lives on `CaptureWorld` (the W4-g68 `&mut` tile-pass
    // channel); this view reads it through `cw.gore`.
    // The dust pool lives on `CaptureWorld` (the tile pass's `&mut` orb
    // channel); this view reads it through `cw.dust`.
    pub scene_state: &'a tr_core::scene::SceneState,
}

/// One direct-to-framebuffer batch: clear the world range, run `f`, render
/// the world range alone over the composites (`Main.spriteBatch.Begin()`
/// … `End()` around a world-space arm).
fn world_batch(
    gpu: &tr_render::Gpu,
    renderer: &mut SpriteRenderer,
    camera: Vec2,
    fb: &wgpu::TextureView,
    anim_time: f32,
    f: impl FnOnce(&mut SpriteRenderer),
) {
    renderer.begin_world_batch();
    f(renderer);
    renderer.render_world_only(gpu, fb, camera.to_array(), 1.0, anim_time, None);
}

/// The `captureEntities` arms between the wall and tile2 composites
/// (`Main.cs:56928-56953`): `DrawWoF` (`:56931`, the port's WoF draws
/// through `draw_npcs` — no separate pass), `DrawGoreBehind` (`:56937`,
/// unported — the gore pool's behind pass is a documented gap),
/// `MoonlordDeathDrama.DrawPieces`/`DrawExplosions` (`:56944-56945`), then
/// the UNCONDITIONAL `DrawCachedNPCs(BehindNonSolidTiles, behindTiles:
/// true)` (`:56951` — runs even without `captureEntities`).
pub(crate) fn draw_capture_mid_entities(
    gpu: &tr_render::Gpu,
    renderer: &mut SpriteRenderer,
    cw: &CaptureWorld<'_>,
    ents: &CaptureEntities<'_>,
    camera: Vec2,
    screen: Vec2,
    fb: &wgpu::TextureView,
    capture_entities: bool,
) {
    if capture_entities {
        // `MoonlordDeathDrama.DrawPieces` + `DrawExplosions`
        // (`Main.cs:56944-56945`).
        world_batch(gpu, renderer, camera, fb, cw.anim_time, |r| {
            crate::render::screen_fx::draw_moonlord_pieces(
                r, cw.lookups, cw.light, camera, 1.0, screen, ents.scene_state,
            );
            crate::render::screen_fx::draw_moonlord_explosions(
                r, cw.lookups, camera, 1.0, screen, ents.scene_state,
            );
        });
    }
    // `DrawCachedNPCs(this.DrawCacheNPCsBehindNonSolidTiles, behindTiles:
    // true)` (`Main.cs:56951`) — unconditional, like the C#.
    world_batch(gpu, renderer, camera, fb, cw.anim_time, |r| {
        crate::render::entities::draw_npcs(
            r, cw.npc_assets, ents.npcs, cw.light, cw.style, cw.anim_time,
            camera, 1.0, screen, true,
            // No empress/party/glow context exists for the chunk render.
            None,
            None,
            0.0,
        );
    });
}

/// The `captureEntities` waterfall arm (`Main.cs:56964-56972`):
/// `waterfallManager.FindWaterfalls(forced: true)` + `Draw()` between the
/// non-solid and solid tile composites. The port's manager walks the chunk
/// view, draws the per-style passes and hands back the quads + lights; the
/// quads push into the world range (the C#'s `Main.tileBatch`), the lights
/// feed the NEXT frame's compute (the one-frame-late `AddLight` rule — the
/// capture's own light already ran, `Main.cs:56727-56732`).
pub(crate) fn draw_capture_waterfalls(
    gpu: &tr_render::Gpu,
    renderer: &mut SpriteRenderer,
    cw: &mut CaptureWorld<'_>,
    camera: Vec2,
    screen: Vec2,
    fb: &wgpu::TextureView,
    liquid_alpha: &[f32; 15],
) {
    let (x0, y0, x1, y1) = capture_view_rect(cw.world, camera, screen);
    cw.waterfall.find_waterfalls(cw.world, [x0, y0, x1, y1], true);
    let (lights, _ambient_scan, _ambient_scan2) = cw.waterfall.draw(
        cw.world, cw.light, cw.style, cw.anim_time, liquid_alpha,
        // `Main.mouseTextColor` — the sparkle tint pulse; the capture keeps
        // the steady 255 (the port's frame path feeds the live value).
        255,
        // The capture's sparkle seed is 0 (the chunk render pins no frame
        // count); the stream stays deterministic per chunk.
        0,
        tr_core::waterfall::ScreenInfo {
            position: [x0 as f32, y0 as f32],
            width: (x1 - x0) as f32,
            height: (y1 - y0) as f32,
        },
        false,
    );
    let quads = cw.waterfall.take_quads();
    renderer.begin_world_batch();
    for d in &quads {
        let Some(uv) = cw.lookups.waterfall_uv(d.texture as usize, d.source) else {
            continue;
        };
        renderer.push_sprite(
            d.pos,
            [d.source[2] as f32, d.source[3] as f32],
            crate::render::entities::flip_uv(uv, d.flip),
            d.color,
        );
    }
    renderer.render_world_only(gpu, fb, camera.to_array(), 1.0, cw.anim_time, None);
    // The one-frame-late swap (`LightingEngine.ApplyPerFrameLights`,
    // `LightingEngine.cs:226-234`): the capture's lights land in the slot
    // the next frame's compute reads.
    cw.scratch.waterfall_lights = lights;
}

/// The `captureEntities` tail (`Main.cs:56989-57017`): the players, the
/// NPCs over tiles, the projectiles, the items, the rain, the gore and the
/// dust — between the solid tile composite and the water composite. The
/// port draws them in its frozen entity-tail order (player → npcs → items,
/// the `TileFilter::Screen` shape; the C#'s players-behind-NPCs split is
/// the documented z-order approximation the frame path carries).
pub(crate) fn draw_capture_entity_tail(
    gpu: &tr_render::Gpu,
    renderer: &mut SpriteRenderer,
    cw: &mut CaptureWorld<'_>,
    ents: &mut CaptureEntities<'_>,
    camera: Vec2,
    screen: Vec2,
    fb: &wgpu::TextureView,
) {
    // `DrawPlayers_BehindNPCs`/`DrawPlayers_AfterProjectiles`
    // (`:56991`/`:57002`) + `DrawNPCs()` (`:56994`) + `DrawItems`
    // (`:57008`) — the world-range half.
    world_batch(gpu, renderer, camera, fb, cw.anim_time, |r| {
        if let Some(p) = ents.player {
            crate::render::entities::draw_player(
                r, cw.player_assets, cw.lookups, p, ents.held, ents.armor,
                cw.light, cw.style, cw.anim_time, camera, 1.0, screen,
            );
        }
        crate::render::entities::draw_npcs(
            r, cw.npc_assets, ents.npcs, cw.light, cw.style, cw.anim_time,
            camera, 1.0, screen, false,
            // No empress/party/glow context exists for the chunk render.
            None,
            None,
            0.0,
        );
        for it in ents.items {
            it.draw(
                r, cw.lookups, cw.light, cw.style, cw.anim_time, cw.mouse_text_color,
            );
        }
    });
    // The screen-space half: `DrawProjectiles` (`:57001`, the port's
    // projectile slots push `push_ui_*`), `DrawRain` (`:57011`), `DrawGore`
    // (`:57014`), `DrawDust` (`:57016`, the flying sand + the wheel
    // sparks). The back range stays parked — the framebuffer already holds
    // the composited background. The world range is cleared first (the
    // player/npcs/items batch already rendered); the yoyo strings push
    // fresh world quads below.
    let parked_back = renderer.take_back();
    renderer.begin_world_batch();
    renderer.fx.clear();
    renderer.ui.clear();
    let hand = ents.player.map(|pl| {
        let (cx, cy) = pl.center();
        (cx, cy + pl.gfx_off_y)
    });
    for p in ents.projectiles.slots().iter().filter(|p| p.active) {
        p.draw_with_hand(
            renderer, cw.lookups, cw.light, camera, 1.0, screen, hand,
            cw.anim_time % 3600.0,
            0.0,
            (cw.anim_time as f64 * 60.0) % 216000.0,
        );
    }
    // The capture keeps the shimmer ramp at 0 (no aether pass in a chunk).
    ents.rain
        .draw(renderer, cw.lookups, cw.light, camera, 1.0, screen, 0.0);
    crate::render::gore::draw(
        &*cw.gore, renderer, cw.lookups, cw.light, cw.style, cw.anim_time,
        camera, 1.0, screen,
    );
    crate::sand::draw(&mut *cw.dust, renderer, cw.lookups, cw.light, camera, 1.0, screen);
    crate::minecart::draw(
        &mut *cw.dust, renderer, cw.lookups, cw.light, camera, 1.0, screen,
    );
    render_screen_ranges(gpu, renderer, camera, fb, cw.anim_time);
    renderer.restore_back(parked_back);
}

/// Render the world+fx+ui ranges over the framebuffer with the back range
/// parked (the screen-space entity passes — the C# draws them through the
/// one `Main.spriteBatch` under `Main.Transform`; the port's fx/ui ranges
/// ride the identity camera, the world range the chunk camera). The
/// caller's screen override (the 2048² framebuffer) stays in force.
fn render_screen_ranges(
    gpu: &tr_render::Gpu,
    renderer: &mut SpriteRenderer,
    camera: Vec2,
    fb: &wgpu::TextureView,
    anim_time: f32,
) {
    // The world-range half (the yoyo strings push `push_sprite_origin_rot`)
    // under the chunk camera — `render_world_only` parks back AND fx, so
    // the fx range survives for the second half.
    renderer.render_world_only(gpu, fb, camera.to_array(), 1.0, anim_time, None);
    // The fx range (rain/gore/dust) alone under the UI camera with Load —
    // `render_fx_only` parks back+world and draws only fx (`draw_ranges`
    // world_half=true, ui_half=false, clear=None). The camera/zoom args
    // feed globals_world (unused — world is parked); fx rides globals_ui's
    // identity camera.
    renderer.render_fx_only(gpu, fb, [FB_SIZE * 0.5, FB_SIZE * 0.5], 1.0, anim_time);
    // The ui range (the projectile sprites) under the UI camera —
    // `render_map_ui` draws map+ui (world_half=false skips back/world/fx).
    renderer.render_map_ui(gpu, fb, [FB_SIZE * 0.5, FB_SIZE * 0.5], 1.0, anim_time);
}

/// `CaptureCamera.FRAMEBUFFER_PIXEL_SIZE` (`CaptureCamera.cs:36`) — the
/// identity camera's centre for the screen-space ranges.
const FB_SIZE: f32 = 2048.0;

/// The capture view rect (zoom 1) — the chunk's visible tile rectangle.
fn capture_view_rect(world: &tr_core::World, camera: Vec2, screen: Vec2) -> (i32, i32, i32, i32) {
    let half = screen * 0.5;
    let min = camera - half;
    let max = camera + half;
    let x0 = ((min.x / 16.0).floor() as i32 - 1).clamp(0, world.width - 1);
    let x1 = ((max.x / 16.0).ceil() as i32 + 1).clamp(0, world.width - 1);
    let y0 = ((min.y / 16.0).floor() as i32 - 1).clamp(0, world.height - 1);
    let y1 = ((max.y / 16.0).ceil() as i32 + 1).clamp(0, world.height - 1);
    (x0, y0, x1, y1)
}

#[cfg(test)]
mod tests {
    /// S1-G-13 consumers: the `captureEntities` arms ride their C#
    /// composite sites — the mid entities between the wall and tile2
    /// composites, the waterfalls between tile2 and tile, the entity tail
    /// between tile and water (`Main.cs:56928-57027`).
    #[test]
    fn the_capture_arms_interleave_at_the_csharp_composite_sites() {
        let cr = include_str!("capture_render.rs");
        let order = [
            "SceneLayer::Background);",
            "SceneLayer::Wall);",
            "draw_capture_mid_entities(",
            "SceneLayer::Tile2);",
            "draw_capture_waterfalls(",
            "SceneLayer::Tile);",
            "draw_capture_entity_tail(",
            "SceneLayer::Water);",
        ];
        let mut at = 0;
        for marker in order {
            let found = cr[at..].find(marker).unwrap_or_else(|| panic!("missing {marker}"));
            at += found + marker.len();
        }
        // The mid entities' behind-tiles NPC pass is unconditional (the
        // C# `:56951` runs without `captureEntities`); the moonlord half
        // is gated.
        let ce = include_str!("capture_entities.rs");
        let mid = &ce[ce.find("pub(crate) fn draw_capture_mid_entities").unwrap()
            ..ce.find("pub(crate) fn draw_capture_waterfalls").unwrap()];
        let gated = mid.find("if capture_entities {").unwrap();
        let unconditional = mid.find("draw_npcs(").unwrap();
        assert!(gated < unconditional, "the moonlord arm is gated, the NPC pass is not");
        assert!(mid[gated..].contains("draw_moonlord_pieces"));
    }

    /// The waterfall arm forces the find (`FindWaterfalls(forced: true)`,
    /// `Main.cs:56968`) and swaps the lights one frame late.
    #[test]
    fn the_capture_waterfall_arm_forces_the_find_and_swaps_the_lights() {
        let ce = include_str!("capture_entities.rs");
        let wf = &ce[ce.find("pub(crate) fn draw_capture_waterfalls").unwrap()
            ..ce.find("pub(crate) fn draw_capture_entity_tail").unwrap()];
        assert!(wf.contains("find_waterfalls(cw.world, [x0, y0, x1, y1], true)"));
        assert!(wf.contains("cw.scratch.waterfall_lights = lights;"));
    }
}