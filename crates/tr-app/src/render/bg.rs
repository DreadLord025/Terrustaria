//! The underground background passes (`Main.DrawBackground` family): the
//! surface transition row, the dirt sheet, the underworld black box, the rock
//! bands, the magma field and the screen-space parallax layer helpers the sky
//! pass shares.
use super::BgTex;
use super::sky::SkyAssets;
use glam::Vec2;
use tr_core::lighting::LightMap;
use tr_core::{TILE_PX, World};
use tr_render::SpriteRenderer;

/// `Main.caveParallax = 1f - (float)Main.bgScroll / 500f` (`Main.cs:50383`,
/// `:51643`): the "Parallax" slider (`Main.bgScroll`, `Main.cs:2429`) maps to
/// the underground backdrop's scroll factor. A fresh config lands on 60, not 0:
/// `static Main()` seeds `caveParallax = 0.88f` (`Main.cs:66809`); with no
/// stored keys `bgScroll = (int)((1f - 0.88f) * 500f)` = 60 and both
/// `Get("Parallax", ...)` / `Get("BackgroundParallax", ...)` keep it
/// (`Main.cs:4405-4408`) — the 0.88 backdrop factor, "Parallax: 60" on screen.
pub(crate) fn cave_parallax(bg_scroll: u8) -> f32 {
    1.0 - bg_scroll as f32 / 500.0
}

/// `Math.IEEERemainder`: `x - y * Math.Round(x / y)` with banker's rounding.
fn ieee_remainder(x: f32, y: f32) -> f32 {
    x - y * round_half_even(x / y)
}

/// `Math.Round(double)`: round half to even.
fn round_half_even(v: f32) -> f32 {
    let f = v.floor();
    let d = v - f;
    if d > 0.5 {
        f + 1.0
    } else if d < 0.5 {
        f
    } else if (f as i64) % 2 == 0 {
        f
    } else {
        f + 1.0
    }
}

/// One 16x16 cell of the underground backdrop, lit by the world tile behind it.
///
/// `pos` is the screen-space top-left of the cell (C#'s
/// `bgStartX + num * i + 16 * k + num2`, `bgStartY + backgroundHeight * j + 16 * l`);
/// `sample` is the screen-space point the light is read at (C#'s `num3`). The original
/// reads the light at the *un-jittered* cell centre, so `sample.x = pos.x + 8 - num2`,
/// while the draw itself keeps the `num2` half-tile jitter.
#[allow(clippy::too_many_arguments)]
fn push_cave_cell(
    r: &mut SpriteRenderer,
    camera: Vec2,
    zoom: f32,
    screen: Vec2,
    tex: &BgTex,
    sx: f32,
    sy: f32,
    pos: [f32; 2],
    sample: [f32; 2],
    light: &LightMap,
    bg_brightness: f32,
) {
    if pos[0] >= screen.x || pos[1] >= screen.y || pos[0] + 16.0 <= 0.0 || pos[1] + 16.0 <= 0.0 {
        return;
    }
    // `Lighting.GetColor((int)((num3 + screenPosition.X) / 16f), ...)`: the light is read on
    // the *unscaled* grid (`Main.screenPosition = camera - screenSize / 2`), not at the tile
    // actually behind the pixel. `Lighting.GetColor` folds in the (ramped) `GlobalBrightness`,
    // which the light map stores unscaled, and both backdrop layers then multiply the result
    // by `backgroundColor` (0.9).
    let sp = camera - screen * 0.5;
    let wx = ((sample[0] + sp.x) / TILE_PX).floor() as i32;
    let wy = ((sample[1] + sp.y) / TILE_PX).floor() as i32;
    let c = light.get(wx, wy);
    let du = (tex.uv[2] - tex.uv[0]) / tex.w;
    let dv = (tex.uv[3] - tex.uv[1]) / tex.h;
    let k = bg_brightness * 0.9;
    r.push_back_sprite(
        [
            camera.x + (pos[0] - screen.x * 0.5) / zoom,
            camera.y + (pos[1] - screen.y * 0.5) / zoom,
        ],
        [16.0 / zoom, 16.0 / zoom],
        [
            tex.uv[0] + du * sx,
            tex.uv[1] + dv * sy,
            tex.uv[0] + du * (sx + 16.0),
            tex.uv[1] + dv * (sy + 16.0),
        ],
        [c[0] * k, c[1] * k, c[2] * k, 1.0],
    );
}

/// `Main.GetRockTransitionPoint`: first rock-backdrop row, in **world** pixels.
///
/// ```text
/// num  = ((int)rockLayer - (int)worldSurface) * 16   (each row truncated first)
/// num2 = num / backgroundHeight[2]; if (num % h != 0) num2++
/// return (int)worldSurface * 16 + num2 * h + 32
/// ```
fn get_rock_transition_point(world: &World, block: f32) -> f32 {
    let block = (block as f64).max(1.0);
    // `Main.cs:53090`: the two rows are cast to int BEFORE the difference, so a
    // fractional layer line is dropped per-row, not from the difference.
    let num = (world.rock_layer as i32 - world.world_surface as i32) as f64 * 16.0;
    let mut blocks = (num / block) as i32;
    if num.rem_euclid(block) != 0.0 {
        blocks += 1;
    }
    ((world.world_surface as i32) as f64 * 16.0 + blocks as f64 * block + 32.0) as f32
}

/// Screen-space anchors of the underground backdrop, split out so the layout can be
/// unit-tested without a GPU.
#[derive(Clone, Copy, Debug, PartialEq)]
struct UndergroundBgLayout {
    /// `Main.DrawBG` uses `Main.BackgroundViewMatrix` (zoom `ForcedMinimumZoom` = 1), so
    /// the backdrop lives in *unscaled* screen space: `Main.screenPosition`, i.e.
    /// `camera - screenSize / 2`, used verbatim.
    screen_pos: Vec2,
    /// `DrawBackground_SurfaceTransitionBackground` `bgTopY = worldSurface*16 - screenPosition.Y`.
    transition_top: f32,
    /// `DrawBackground_DirtBackground` `bgTopY = worldSurface*16 - screenPosition.Y + 16`.
    dirt_top: f32,
    /// `Main.Camera.ScaledPosition.Y + screenHeight`: the dirt sheet is clamped at the
    /// rock transition while this bottom edge is below `GetRockTransitionPoint`.
    scaled_bottom: f32,
}

fn underground_bg_layout(
    world: &World,
    camera: Vec2,
    zoom: f32,
    screen: Vec2,
) -> UndergroundBgLayout {
    let screen_pos = camera - screen * 0.5;
    let transition_top = world.world_surface as f32 * TILE_PX - screen_pos.y;
    UndergroundBgLayout {
        screen_pos,
        transition_top,
        dirt_top: transition_top + 16.0,
        // `ScaledPosition.Y = UnscaledPosition.Y + GameViewMatrix.Translation.Y` and the
        // translation is `screenSize.Y / 2 * (1 - 1 / zoom)`, so the top-left of the
        // scaled view is `camera.Y - screenSize.Y / (2 * zoom)`.
        scaled_bottom: camera.y - screen.y / (2.0 * zoom) + screen.y,
    }
}

/// `Main.offScreenRange` (`Main.cs:58175`): the underground passes render into a
/// target padded by this on every side (`WorldSceneLayerTarget.cs:49-57`) and draw
/// everything `+drawOffset` into it. The port renders the unpadded viewport, so the
/// pad cancels out of the draw positions; it survives only in the visibility padding
/// of the dirt sheet (`Main.cs:53411`).
const OFF_SCREEN_RANGE: f32 = 192.0;

/// The tiling state shared by the underground passes (`Main.bgStartX`, `Main.bgStartY`,
/// `Main.bgLoops`, `Main.bgLoopsY`, `Main.bgTopY`): `DrawBackground_DrawMagmaTransition`
/// consumes the `bgStartY`/`bgLoopsY` its caller `DrawBackground_DrawRockLayer` wrote.
#[derive(Default)]
struct BgPassState {
    start_x: f32,
    start_y: f32,
    loops: i32,
    loops_y: i32,
    top_y: f32,
}

/// The prologue every underground pass opens with (`bgStartX`/`bgLoops` plus the
/// half-tile source jitter): phase the 128 px tiling block against the screen's
/// left edge under `caveParallax`
/// (`bgStartX = (int)(-IEEERemainder(screenPosition.X * bgParallax + block, block)
/// - block / 2)`, `Main.cs:53420-53421` and its per-pass twins), then derive the
/// `num2`/`num4` source-rect jitter with the `-8 -> 8` snap (`Main.cs:53422-53429`,
/// `:53134-53140`, `:52887-52891`). Shared so the five passes cannot drift apart;
/// a plain struct, no indirection in the hot path.
struct PassPrologue {
    start_x: f32,
    loops: i32,
    /// The half-tile source-rectangle jitter (`diff` in the original).
    jitter: f32,
}

fn pass_prologue(screen: Vec2, screen_pos: Vec2, block_w: f32, cave_parallax: f32) -> PassPrologue {
    let start_x = (0.0
        - ieee_remainder(block_w + screen_pos.x * cave_parallax, block_w)
        - block_w * 0.5) as i32 as f32;
    let loops = (screen.x / block_w) as i32 + 2;
    let mut jitter = round_half_even(-ieee_remainder(start_x + screen_pos.x, 16.0));
    if jitter as i32 == -8 {
        jitter = 8.0;
    }
    PassPrologue {
        start_x,
        loops,
        jitter,
    }
}

/// `Main.DrawBackground_UpdateBackgroundStyles` (`Main.cs:53802`): the
/// `_drawBackground_backTexture[0..7]` sheet table. Only `undergroundBackground == 0`
/// — the classic set (`Main.cs:53817-53824`) — is reachable in the port: the style
/// pick (`DrawBackground_PickUndergroundBackgroundStyle`, `Main.cs:54033`) needs
/// `Main.caveBackX`/`caveBackStyle` and the `SceneMetrics` zone counts, none of which
/// the port models, so the per-biome `case` arms (`Main.cs:53825-53987`) stay dead.
/// `hellBackStyle` (a per-world roll, `WorldGen.cs:8247`) still selects the animated
/// magma sheets (`Main.cs:54000-54008`).
fn draw_background_update_background_styles(hell_back_style: usize) -> [usize; 7] {
    let mut back = [0usize; 7]; // Main.cs:53804
    // `case 0:` (Main.cs:53817-53824).
    back[0] = 1; // Background_1 — surface transition row
    back[1] = 2; // Background_2 — dirt sheet
    back[2] = 4; // Background_4 — rock-transition band
    back[3] = 3; // Background_3 — rock bands
    back[4] = 6; // Background_6 — magma transition
    back[5] = 5; // dead store: always overridden below
    back[5] = 125 + hell_back_style; // Main.cs:54000-54007
    back[6] = 185 + hell_back_style; // Main.cs:54008
    back
}

/// `_drawBackground_backTexture` id → the port's decoded sheet (`LoadBackground`,
/// `Main.cs:59039`, fills `backgroundWidth`/`backgroundHeight` from the texture).
fn back_texture(sky: &SkyAssets, id: usize) -> Option<&BgTex> {
    match id {
        1 => sky.underground_transition.as_ref(),
        2 => sky.underground.as_ref(),
        3 => sky.rock.as_ref(),
        4 => sky.rock_band.as_ref(),
        6 => sky.magma_transition.as_ref(),
        125..=127 => sky.magma[id - 125].as_ref(),
        185..=187 => sky.hell_bottom[id - 185].as_ref(),
        _ => None,
    }
}

/// `Main.WallLightAt` (`Main.cs:55160`): `Main.wallLight[tile.wall]`. The
/// `tile.invisibleWall()` escape is never set anywhere in the original (see the
/// `WALL_ECHO` note), so the table alone decides.
fn wall_light_at(world: &World, x: i32, y: i32) -> bool {
    use tr_core::tiledata::{WALL_COUNT, WALL_LIGHT};
    world
        .tile(x, y)
        .is_some_and(|t| (t.wall as usize) < WALL_COUNT && WALL_LIGHT[t.wall as usize])
}

/// `Main.ShouldDrawBackgroundTileAt` (`Main.cs:55136`): with a non-zero parallax the
/// wall light of the horizontal neighbours counts too.
fn should_draw_background_tile_at(world: &World, x: i32, y: i32, cave_parallax: f32) -> bool {
    if cave_parallax != 0.0 {
        wall_light_at(world, x, y)
            || wall_light_at(world, x - 1, y)
            || wall_light_at(world, x + 1, y)
    } else {
        wall_light_at(world, x, y)
    }
}

/// `WorldGen.InWorld(x, y, 1)` (`WorldGen.cs:9688`).
fn in_world_1(world: &World, x: i32, y: i32) -> bool {
    x >= 1 && x < world.width - 1 && y >= 1 && y < world.height - 1
}

/// `color.R != 0` for the byte colour `Lighting.GetColor` returns: the engine keeps
/// light as bytes, so a channel below one step (`1/255` after the GlobalBrightness
/// fold) truncates to zero.
fn light_channel_nonzero(c: f32, brightness: f32) -> bool {
    c * brightness * 255.0 >= 1.0
}

/// Any channel lit (`color.R > 0 || color.G > 0 || color.B > 0`).
fn light_any_nonzero(light: [f32; 3], brightness: f32) -> bool {
    light_channel_nonzero(light[0], brightness)
        || light_channel_nonzero(light[1], brightness)
        || light_channel_nonzero(light[2], brightness)
}

/// Every channel lit (`color.R != 0 && color.G != 0 && color.B != 0`).
fn light_all_nonzero(light: [f32; 3], brightness: f32) -> bool {
    light_channel_nonzero(light[0], brightness)
        && light_channel_nonzero(light[1], brightness)
        && light_channel_nonzero(light[2], brightness)
}

/// The cell skip chain of `DrawBackground_DrawRockLayer` (`Main.cs:53152-53159`): a
/// cell is skipped when it is covered by an active tile but still lit, when it is
/// fully dark, or when it has no light wall and the parallax is zero.
fn rock_cell_skipped(
    should_draw: bool,
    wall_light: bool,
    cave_parallax: f32,
    light: [f32; 3],
    brightness: f32,
) -> bool {
    (!should_draw && light_all_nonzero(light, brightness))
        || !light_any_nonzero(light, brightness)
        || (!wall_light && cave_parallax == 0.0)
}

/// The variant in `DrawBackground_DrawMagmaLayer` (`Main.cs:52907`): the fully-dark
/// skip only reaches down to `maxTilesY - 300`, so the deepest hell rows draw even
/// when black.
fn magma_cell_skipped(
    should_draw: bool,
    wall_light: bool,
    cave_parallax: f32,
    light: [f32; 3],
    brightness: f32,
    row: i32,
    max_rows: i32,
) -> bool {
    (!should_draw && light_all_nonzero(light, brightness))
        || (!light_any_nonzero(light, brightness) && row <= max_rows - 300)
        || (!wall_light && cave_parallax == 0.0)
}

/// `Main.magmaBGFrame` (`Main.cs:62218-62224`): the counter ticks once per update
/// (60/s) and the frame advances every 8 ticks, wrapping at 3.
fn magma_bg_frame(anim_time: f32) -> usize {
    ((anim_time * 60.0) / 8.0) as usize % 3
}

/// XNA samples the source rectangle with `SamplerState.PointClamp`
/// (`Main.DefaultSamplerState`, `Main.cs:2774`), so a source column past the sheet
/// edge smears the last column. The quad-UV pipeline cannot smear per pixel; clamping
/// the cell's source offset keeps it inside the sheet (and inside the atlas).
fn cave_cell_sx(tex: &BgTex, sx: f32) -> f32 {
    sx.clamp(0.0, tex.w - 16.0)
}

/// `Lighting.GlobalBrightness` inside the background pass (`Main.cs:52820-52821` with
/// `Lighting.cs:92`'s per-frame reset to 1.2): the light map stores the unscaled engine
/// value (tiles re-apply `1.2` through `LightStyle::shade`), so the backdrop folds the
/// ramp in itself: `1.2 - 0.2*num3`, `num3 = clamp((screenTop - worldSurface*16)/300)`.
fn background_brightness(screen_top: f32, world_surface: f64) -> f32 {
    let num3 = ((screen_top - world_surface as f32 * TILE_PX) / 300.0).clamp(0.0, 1.0);
    1.2 - 0.2 * num3
}

/// `Main.cs:52812-52813`: `magmaLayer` — the span below the surface, truncated to
/// whole 6-row blocks, minus 5 rows.
fn magma_layer_of(world: &World) -> f64 {
    let num2 = (((world.height - 330) as f64 - world.world_surface) / 6.0) as i32;
    world.world_surface + num2 as f64 * 6.0 - 5.0
}

/// `Main.DrawBackground` (`Main.cs:52798`): the underground background tree — surface
/// transition → dirt sheet → underworld black box → rock bands → magma — laid out in
/// unzoomed `Main.screenPosition` space and scrolled with `Main.caveParallax`. Every
/// 16x16 cell takes the light of the world tile it covers, tinted by the constant
/// `backgroundColor` (0.9, `Main.cs:52818`) under the `GlobalBrightness` ramp.
#[allow(clippy::too_many_arguments)]
pub(crate) fn draw_background(
    r: &mut SpriteRenderer,
    camera: Vec2,
    zoom: f32,
    screen: Vec2,
    world: &World,
    sky: &SkyAssets,
    light: &LightMap,
    // `Main.caveParallax = 1f - (float)Main.bgScroll / 500f` (`Main.cs:50383`,
    // `:51643`), fed by the "Parallax" slider.
    cave_parallax: f32,
    anim_time: f32,
    // `Main.hellBlackBoxBottom` (`Main.cs:2463`): carried across frames — the black
    // box consumes the value the previous frame's magma pass produced.
    hell_black_box_bottom: &mut f32,
) {
    let Some(dirt) = sky.underground else {
        return;
    };
    // `Main.DrawBG` runs the backdrop under `Main.BackgroundViewMatrix`, whose zoom is
    // `ForcedMinimumZoom` (1 for any viewport <= 1920x1200), so the matrix is identity
    // and the layout is in *unzoomed* screen coordinates: `Main.screenPosition`
    // (`camera - screenSize / 2`) is used verbatim, never divided by the game zoom.
    // `push_cave_cell` maps those screen px 1:1 with a fixed 16 px cell.
    let layout = underground_bg_layout(world, camera, zoom, screen);
    let screen_pos = layout.screen_pos;
    // `Main.cs:52820-52821`: the whole pass runs with
    // `Lighting.GlobalBrightness = 1.2*(1-num3) + num3`, where
    // `num3 = clamp((screenPosition.Y - worldSurface*16)/300, 0, 1)` (`Lighting.cs:92`
    // resets GlobalBrightness to 1.2 every frame). See `background_brightness`.
    let bg_brightness = background_brightness(screen_pos.y, world.world_surface);
    // `Main.cs:52812-52813`: `magmaLayer` — see `magma_layer_of`.
    let magma_layer = magma_layer_of(world);
    // `Main.cs:52814-52815` with `Main.gfxQuality = 0` (the port has no quality
    // auto-scaler): the thresholds only gate the collapsed 9/4-slice branches.
    let (q1, q2) = (255, 200);
    // `Main.cs:52816-52818`: `drawToScreen` is false, so `drawOffset = offScreenRange`;
    // `backgroundColor` is a flat 0.9 multiplier, folded into `push_cave_cell`.
    let draw_offset = Vec2::new(OFF_SCREEN_RANGE, OFF_SCREEN_RANGE);
    // `Main.cs:52822`/`:52850`: after the passes `GlobalBrightness` becomes
    // `1.2 - 0.2*num4` for the rest of the frame (the tiles); the port's tile light
    // re-applies its own 1.2 (established with the dirt pass), so only the backdrop
    // folds the ramp here.
    // `Main.cs:52823-52840`: `PickUndergroundBackgroundStyle` + the `ugBackTransition`
    // cross-fade — `undergroundBackground` is pinned to 0 (see
    // `draw_background_update_background_styles`), the style never swaps and every
    // `_drawBackground_oldBackTexture` overlay draw is dead.
    let back = draw_background_update_background_styles(world.hell_back_style as usize);
    let rock_transition_point = get_rock_transition_point(world, dirt.h);
    let frame = magma_bg_frame(anim_time);

    // `Main.cs:52844-52849`: transition → dirt → black box → rock → magma, in order —
    // the box paints first so the light-gated rock/magma cells draw over it.
    draw_background_surface_transition_background(
        r,
        camera,
        zoom,
        screen,
        sky,
        light,
        &back,
        &layout,
        cave_parallax,
        bg_brightness,
    );
    draw_background_dirt_background(
        r,
        camera,
        zoom,
        screen,
        world,
        sky,
        light,
        &back,
        &layout,
        rock_transition_point,
        cave_parallax,
        bg_brightness,
        &dirt,
    );
    draw_background_draw_underworld_black_box(
        r,
        camera,
        zoom,
        screen,
        world,
        sky,
        magma_layer,
        screen_pos,
        draw_offset,
        *hell_black_box_bottom,
    );
    *hell_black_box_bottom = screen_pos.y + screen.y + 100.0; // Main.cs:52847
    let mut state = BgPassState::default();
    draw_background_draw_rock_layer(
        r,
        camera,
        zoom,
        screen,
        world,
        sky,
        light,
        0.0,
        magma_layer,
        q1,
        q2,
        draw_offset,
        &mut state,
        bg_brightness,
        &back,
        dirt.h,
        rock_transition_point,
        cave_parallax,
        frame,
    );
    draw_background_draw_magma_layer(
        r,
        camera,
        zoom,
        screen,
        world,
        sky,
        light,
        magma_layer,
        q1,
        q2,
        draw_offset,
        &mut state,
        bg_brightness,
        &back,
        dirt.h,
        cave_parallax,
        frame,
        hell_black_box_bottom,
    );
}

/// `Main.DrawBackground_SurfaceTransitionBackground` (`Main.cs:53736`): the single
/// `Background_1` row on the surface line, light per covered world tile.
#[allow(clippy::too_many_arguments)]
fn draw_background_surface_transition_background(
    r: &mut SpriteRenderer,
    camera: Vec2,
    zoom: f32,
    screen: Vec2,
    sky: &SkyAssets,
    light: &LightMap,
    back: &[usize; 7],
    layout: &UndergroundBgLayout,
    cave_parallax: f32,
    bg_brightness: f32,
) {
    let Some(transition) = back_texture(sky, back[0]) else {
        return;
    };
    let screen_pos = layout.screen_pos;
    let surface_screen = layout.transition_top;
    let num = transition.w - 32.0;
    if num < 16.0 {
        return;
    }
    let PassPrologue {
        start_x,
        loops,
        jitter: n2,
    } = pass_prologue(screen, screen_pos, num, cave_parallax);
    for i in 0..loops {
        for j in 0..(num as i32 / 16) {
            let cell_x = start_x + num * i as f32 + 16.0 * j as f32;
            push_cave_cell(
                r,
                camera,
                zoom,
                screen,
                transition,
                16.0 * j as f32 + n2 + 16.0,
                0.0,
                [cell_x + n2, surface_screen],
                [cell_x + 8.0, surface_screen],
                light,
                bg_brightness,
            );
        }
    }
}

/// `Main.DrawBackground_DirtBackground` (`Main.cs:53403`): the `Background_2` dirt
/// sheet from one tile below the surface down to the rock transition, closed by the
/// `Background_4` band when the rock backdrop takes over (`Main.cs:53700-53735`).
#[allow(clippy::too_many_arguments)]
fn draw_background_dirt_background(
    r: &mut SpriteRenderer,
    camera: Vec2,
    zoom: f32,
    screen: Vec2,
    world: &World,
    sky: &SkyAssets,
    light: &LightMap,
    back: &[usize; 7],
    layout: &UndergroundBgLayout,
    rock_transition_point: f32,
    cave_parallax: f32,
    bg_brightness: f32,
    dirt: &BgTex,
) {
    let screen_pos = layout.screen_pos;
    let surface_screen = layout.transition_top;
    // `Main.cs:53411-53412`: vanilla still enters while the surface line is up to
    // `offScreenRange` (192) px below the screen bottom (the padded target's
    // overscan). Those cells are off-screen and clipped by `push_cave_cell`, so the
    // padding only matters for exactness, not for the visible picture.
    if surface_screen > screen.y + OFF_SCREEN_RANGE {
        return;
    }
    let num = dirt.w - 32.0;
    if num < 16.0 {
        return;
    }
    let PassPrologue {
        start_x,
        loops,
        jitter: n2,
    } = pass_prologue(screen, screen_pos, num, cave_parallax);
    // `bgTopY = worldSurface * 16 - screenPosition.Y + 16` (Main.cs:53410).
    let top = layout.dirt_top;
    // `bgStartY`: `if (worldSurface * 16 < screenPosition.Y - 16)` the sheet is repeated
    // from above the screen (`(int)(IEEERemainder(bgTopY, h) - h)`), otherwise it starts
    // exactly one tile below the surface line (`(int)bgTopY`). The test is on
    // `screenPosition.Y`, NOT on `bgTopY`, so it still holds while the surface line is a
    // few tiles off the top of the screen. (Main.cs:53419-53429)
    let start_y = if world.world_surface as f32 * TILE_PX < screen_pos.y - 16.0 {
        (ieee_remainder(top, dirt.h) - dirt.h) as i32 as f32
    } else {
        top as i32 as f32
    };
    let mut loops_y = ((screen.y - start_y) / dirt.h) as i32 + 1;
    // `if (rockTransitionPoint < Main.Camera.ScaledPosition.Y + screenHeight - 16)`
    // clamps the dirt sheet at the rock transition (`GetRockTransitionPoint`,
    // `Main.cs:53431-53434`): the rock backdrop takes over below it
    // (`DrawBackground_DrawRockLayer`), so the dirt must not bleed into the caverns.
    // The row count is measured from `bgStartY` (the wrapped start above).
    let flag = rock_transition_point < layout.scaled_bottom - 16.0;
    if flag {
        // C#: `(int)(rockTransitionPoint - screenPosition.Y - bgStartY) / backgroundHeight[2]`
        // — the numerator is truncated to `int` *before* the (integer) division.
        loops_y = ((rock_transition_point - screen_pos.y - start_y) as i32) / (dirt.h as i32);
    }
    // The band below still draws when the row count collapses (`Main.cs:53707` gates on
    // its own top), so the cells run under an `if`, not an early return.
    if loops_y > 0 {
        let rows = (dirt.h as i32 / 16).max(1);
        for i in 0..loops {
            for j in 0..loops_y {
                for k in 0..(num as i32 / 16) {
                    for l in 0..rows {
                        let cell_x = start_x + num * i as f32 + 16.0 * k as f32;
                        let cell_y = start_y + dirt.h * j as f32 + 16.0 * l as f32;
                        push_cave_cell(
                            r,
                            camera,
                            zoom,
                            screen,
                            dirt,
                            16.0 * k as f32 + n2 + 16.0,
                            16.0 * l as f32,
                            [cell_x + n2, cell_y],
                            [cell_x + 8.0, cell_y + 8.0],
                            light,
                            bg_brightness,
                        );
                    }
                }
            }
        }
    }
    // `Main.cs:53700-53735`: one row of `Background_4` right above the rock
    // transition closes the gap the clamped dirt sheet leaves.
    if !flag {
        return;
    }
    let Some(band) = back_texture(sky, back[2]) else {
        return;
    };
    let num = 128.0; // Main.cs:53701 — the band block is 128 px wide
    let PassPrologue {
        start_x,
        loops,
        jitter: n2,
    } = pass_prologue(screen, screen_pos, num, cave_parallax);
    let band_top = rock_transition_point - screen_pos.y - 16.0; // Main.cs:53705
    if !(band_top > -32.0) {
        return; // Main.cs:53707-53709
    }
    for i in 0..loops {
        for j in 0..(num as i32 / 16) {
            let cell_x = start_x + num * i as f32 + 16.0 * j as f32;
            push_cave_cell(
                r,
                camera,
                zoom,
                screen,
                band,
                cave_cell_sx(band, 16.0 * j as f32 + n2 + 16.0),
                0.0,
                [cell_x + n2, band_top],
                [cell_x + 8.0, band_top],
                light,
                bg_brightness,
            );
        }
    }
}

/// `Main.DrawBackground_DrawUnderworldBlackBox` (`Main.cs:53382`): the solid black
/// rectangle painted under the rock/magma cells once the camera reaches the magma
/// layer, so their light-gated skips show black instead of the sky. The `remixWorld`
/// branch (`Main.cs:53384`) has no equivalent in the port.
#[allow(clippy::too_many_arguments)]
fn draw_background_draw_underworld_black_box(
    r: &mut SpriteRenderer,
    camera: Vec2,
    zoom: f32,
    screen: Vec2,
    world: &World,
    sky: &SkyAssets,
    magma_layer: f64,
    screen_pos: Vec2,
    draw_offset: Vec2,
    hell_black_box_bottom: f32,
) {
    let _ = draw_offset;
    if !(magma_layer * 16.0 <= (screen_pos.y + screen.y) as f64) {
        return; // Main.cs:53384-53386
    }
    let mut num = screen.y as i32 + 200; // Main.cs:53388
    let width = screen.x as i32 + 100; // Main.cs:53389
    // `Main.UnderworldLayer = maxTilesY - 200` (`Main.cs:2790`).
    let underworld_px = (world.height - 200) as f32 * 16.0;
    if underworld_px < screen_pos.y + screen.y {
        // `Main.cs:53390-53394`: stop at the previous frame's hell floor line (the
        // original adds `drawOffset.Y`, the padded target's crop, which cancels in
        // the port's unpadded viewport).
        let num2 = (hell_black_box_bottom - screen_pos.y) as i32;
        if num > num2 {
            num = num2;
        }
    }
    if num <= 0 {
        // XNA draws nothing for a non-positive height; this is the first frame's
        // `hellBlackBoxBottom == 0` case.
        return;
    }
    // `Main.cs:53396`: `Draw(BlackTile, (0, 0, width, num), Color(0, 0, 0))` — a
    // solid opaque black quad (`Black_Tile` is uniformly black).
    let uv = sky
        .black_tile
        .as_ref()
        .map_or_else(|| r.white_uv(), |t| t.uv);
    push_screen(
        r,
        camera,
        zoom,
        screen,
        [0.0, 0.0],
        [width as f32, num as f32],
        uv,
        [0.0, 0.0, 0.0, 1.0],
    );
}

/// `Main.DrawBackground_DrawRockLayer` (`Main.cs:53100`): the `Background_3` rock
/// bands, tiled from `GetRockTransitionPoint` down to `magmaLayer` and light-gated
/// per world tile. Ends with the rock→magma transition band (`Main.cs:53341`).
#[allow(clippy::too_many_arguments)]
fn draw_background_draw_rock_layer(
    r: &mut SpriteRenderer,
    camera: Vec2,
    zoom: f32,
    screen: Vec2,
    world: &World,
    sky: &SkyAssets,
    light: &LightMap,
    // `Main.shimmerAlpha` (`Main.cs:53101`): the port never simulates shimmer, so
    // this stays 0 and every `localShimmerAlpha` branch collapses to its zero arm.
    local_shimmer_alpha: f32,
    magma_layer: f64,
    q1: i32,
    q2: i32,
    draw_offset: Vec2,
    state: &mut BgPassState,
    bg_brightness: f32,
    back: &[usize; 7],
    // `Main.backgroundHeight[2]` — the dirt height `GetRockTransitionPoint` snaps to.
    block_h: f32,
    rock_transition_point: f32,
    cave_parallax: f32,
    magma_bg_frame: usize,
) {
    let _ = (q1, q2, local_shimmer_alpha, draw_offset);
    // The 9-slice/4-slice smoothing branches (`Main.cs:53156-53320`) are collapsed to
    // the plain cell, like the dirt pass (see ANALYSIS.md).
    let Some(tex) = back_texture(sky, back[3]) else {
        return;
    };
    let screen_pos = camera - screen * 0.5;
    // Main.cs:53101-53102.
    let magma_transition = rock_transition_point <= screen_pos.y + screen.y
        && magma_layer * 16.0 < (screen_pos.y + screen.y) as f64;
    // Main.cs:53103-53104: `num` anchors the sheet top, `num2` the magma floor
    // 600 px below the magma layer.
    let num = rock_transition_point;
    let num2 = magma_layer * 16.0 + 600.0;
    state.top_y = num - screen_pos.y;
    let num3: f32 = 128.0; // Main.cs:53110 — the tiling block is 128 px wide.
    if !(rock_transition_point <= screen_pos.y + screen.y) {
        return; // Main.cs:53111-53113
    }
    let PassPrologue {
        start_x,
        loops,
        jitter: num4,
    } = pass_prologue(screen, screen_pos, num3, cave_parallax);
    state.start_x = start_x;
    state.loops = loops;
    // Main.cs:53119-53128: far above the band the sheet repeats from above the
    // screen; otherwise it starts at `bgTopY`. The row pitch divides by
    // `backgroundHeight[2]` (the dirt height) in both arms.
    if rock_transition_point + screen.y < screen_pos.y - 16.0 {
        state.start_y = (ieee_remainder(state.top_y, tex.h) - tex.h) as i32 as f32;
        state.loops_y = ((screen.y - state.start_y) / block_h) as i32 + 1;
    } else {
        state.start_y = state.top_y as i32 as f32;
        state.loops_y = ((screen.y - state.top_y) / block_h) as i32 + 1;
    }
    if magma_layer * 16.0 < (screen_pos.y + screen.y) as f64 {
        // Main.cs:53130-53132: the sheet ends at the magma floor; the numerator is
        // truncated before the (integer) division.
        state.loops_y =
            ((num2 - state.start_y as f64 - screen_pos.y as f64) as i32) / block_h as i32;
    }
    for i in 0..state.loops {
        for j in 0..state.loops_y {
            for k in 0..(num3 as i32 / 16) {
                for l in 0..6 {
                    // Main.cs:53142: the light is sampled on the hardcoded 96 px
                    // pitch (= backgroundHeight[2]; Background_3 is 96 px tall too).
                    let sample_y = state.start_y + j as f32 * 96.0 + l as f32 * 16.0 + 8.0;
                    let cell_x = state.start_x + num3 * i as f32 + 16.0 * k as f32;
                    let sample_x = cell_x + 8.0;
                    let tx = ((sample_x + screen_pos.x) / 16.0) as i32;
                    let ty = ((sample_y + screen_pos.y) / 16.0) as i32;
                    if !in_world_1(world, tx, ty) {
                        continue; // Main.cs:53146
                    }
                    let c = light.get(tx, ty);
                    if rock_cell_skipped(
                        should_draw_background_tile_at(world, tx, ty, cave_parallax),
                        wall_light_at(world, tx, ty),
                        cave_parallax,
                        c,
                        bg_brightness,
                    ) {
                        continue; // Main.cs:53152-53159
                    }
                    let cell_y = state.start_y + tex.h * j as f32 + 16.0 * l as f32;
                    push_cave_cell(
                        r,
                        camera,
                        zoom,
                        screen,
                        tex,
                        cave_cell_sx(tex, 16.0 * k as f32 + num4 + 16.0),
                        16.0 * l as f32,
                        [cell_x + num4, cell_y],
                        [sample_x, sample_y],
                        light,
                        bg_brightness,
                    );
                }
            }
        }
    }
    // Main.cs:53341.
    draw_background_draw_magma_transition(
        r,
        camera,
        zoom,
        screen,
        sky,
        light,
        magma_transition,
        state,
        bg_brightness,
        back,
        block_h,
        num4,
        cave_parallax,
        magma_bg_frame,
    );
}

/// `Main.DrawBackground_DrawMagmaTransition` (`Main.cs:53344`): the one-row band
/// between the rock sheet and the magma field, drawn at `bgStartY + bgLoopsY *
/// backgroundHeight[2]` — the state its rock-layer caller wrote. Its cells draw
/// unconditionally (no light skip, `Main.cs:53358-53380`).
#[allow(clippy::too_many_arguments)]
fn draw_background_draw_magma_transition(
    r: &mut SpriteRenderer,
    camera: Vec2,
    zoom: f32,
    screen: Vec2,
    sky: &SkyAssets,
    light: &LightMap,
    magma_transition: bool,
    state: &mut BgPassState,
    bg_brightness: f32,
    back: &[usize; 7],
    block_h: f32,
    // `diff` — the rock pass's half-tile jitter (`Main.cs:53344`).
    diff: f32,
    cave_parallax: f32,
    magma_bg_frame: usize,
) {
    let background_width: f32 = 128.0; // Main.cs:53349
    if !magma_transition {
        return; // Main.cs:53351-53353
    }
    let Some(tex) = back_texture(sky, back[4]) else {
        return;
    };
    let screen_pos = camera - screen * 0.5;
    // The band keeps the rock pass's `diff` jitter, so only the phase/loop count
    // comes from the shared prologue.
    let PassPrologue { start_x, loops, .. } =
        pass_prologue(screen, screen_pos, background_width, cave_parallax);
    state.start_x = start_x;
    state.loops = loops;
    state.top_y = state.start_y + state.loops_y as f32 * block_h; // Main.cs:53357
    for i in 0..state.loops {
        for j in 0..(background_width as i32 / 16) {
            let cell_x = state.start_x + background_width * i as f32 + 16.0 * j as f32;
            push_cave_cell(
                r,
                camera,
                zoom,
                screen,
                tex,
                cave_cell_sx(tex, 16.0 * j as f32 + diff + 16.0),
                (magma_bg_frame * 16) as f32,
                [cell_x + diff, state.top_y],
                [cell_x + 8.0, state.top_y],
                light,
                bg_brightness,
            );
        }
    }
}

/// `Main.DrawBackground_DrawMagmaLayer` (`Main.cs:52856`): the animated hell backdrop
/// (`Background_125..127`, three 96 px frames cycled by `Main.magmaBGFrame`) from
/// `magmaLayer` down to the underworld floor, then the hell floor row
/// (`Background_185..187`) that pins `hellBlackBoxBottom` (`Main.cs:53063`).
#[allow(clippy::too_many_arguments)]
fn draw_background_draw_magma_layer(
    r: &mut SpriteRenderer,
    camera: Vec2,
    zoom: f32,
    screen: Vec2,
    world: &World,
    sky: &SkyAssets,
    light: &LightMap,
    magma_layer: f64,
    q1: i32,
    q2: i32,
    draw_offset: Vec2,
    state: &mut BgPassState,
    bg_brightness: f32,
    back: &[usize; 7],
    block_h: f32,
    cave_parallax: f32,
    magma_bg_frame: usize,
    hell_black_box_bottom: &mut f32,
) {
    let _ = (q1, q2, draw_offset);
    // The 9-slice/4-slice smoothing branches (`Main.cs:52910-53049`) are collapsed to
    // the plain cell, like the dirt pass (see ANALYSIS.md).
    let Some(tex) = back_texture(sky, back[5]) else {
        return;
    };
    let screen_pos = camera - screen * 0.5;
    // Main.cs:52860: the sheet top sits 600 px (− 8) below the magma layer.
    state.top_y = (magma_layer * 16.0) as f32 - screen_pos.y + 16.0 + 600.0 - 8.0;
    let num: f32 = 128.0; // Main.cs:52862
    if !(magma_layer * 16.0 <= (screen_pos.y + screen.y) as f64) {
        return; // Main.cs:52863-52865
    }
    let PassPrologue {
        start_x,
        loops,
        jitter: num2,
    } = pass_prologue(screen, screen_pos, num, cave_parallax);
    state.start_x = start_x;
    state.loops = loops;
    let mut flag = false;
    // Main.cs:52868-52878.
    if magma_layer * 16.0 + (screen.y as f64) < (screen_pos.y - 16.0) as f64 {
        state.start_y = (ieee_remainder(state.top_y, block_h) - block_h) as i32 as f32;
        state.loops_y = ((screen.y - state.start_y) / block_h) as i32 + 1;
    } else {
        state.start_y = state.top_y as i32 as f32;
        state.loops_y = ((screen.y - state.top_y) / block_h) as i32 + 1;
    }
    // Main.cs:52879-52883: the sheet stops at the underworld floor
    // (`Main.UnderworldLayer = maxTilesY - 200`, `Main.cs:2790`).
    let underworld_px = (world.height - 200) as f32 * 16.0;
    if underworld_px < screen_pos.y + screen.y {
        state.loops_y = ((underworld_px - screen_pos.y - state.start_y) / block_h).ceil() as i32;
        flag = true;
    }
    // Main.cs:52884-52885: the q thresholds of the collapsed slice branches.
    let (q1, q2) = ((q1 as f64 * 1.5) as i32, (q2 as f64 * 1.5) as i32);
    let _ = (q1, q2);
    for i in 0..state.loops {
        for j in 0..state.loops_y {
            for k in 0..(num as i32 / 16) {
                for l in 0..6 {
                    // Main.cs:52898-52900: the light is sampled on the hardcoded
                    // 96 px pitch (= backgroundHeight[2]).
                    let sample_y = state.start_y + j as f32 * 96.0 + l as f32 * 16.0 + 8.0;
                    let cell_x = state.start_x + num * i as f32 + 16.0 * k as f32;
                    let sample_x = cell_x + 8.0;
                    let tx = ((sample_x + screen_pos.x) / 16.0) as i32;
                    let ty = ((sample_y + screen_pos.y) / 16.0) as i32;
                    if !in_world_1(world, tx, ty) {
                        continue; // Main.cs:52901
                    }
                    let c = light.get(tx, ty);
                    if magma_cell_skipped(
                        should_draw_background_tile_at(world, tx, ty, cave_parallax),
                        wall_light_at(world, tx, ty),
                        cave_parallax,
                        c,
                        bg_brightness,
                        ty,
                        world.height,
                    ) {
                        continue; // Main.cs:52907
                    }
                    let cell_y = state.start_y + block_h * j as f32 + 16.0 * l as f32;
                    push_cave_cell(
                        r,
                        camera,
                        zoom,
                        screen,
                        tex,
                        cave_cell_sx(tex, 16.0 * k as f32 + num2 + 16.0),
                        16.0 * l as f32 + block_h * magma_bg_frame as f32,
                        [cell_x + num2, cell_y],
                        [sample_x, sample_y],
                        light,
                        bg_brightness,
                    );
                }
            }
        }
    }
    if !flag {
        return; // Main.cs:53056-53057
    }
    // Main.cs:53058-53063: the hell floor row, pinned to the underworld floor line.
    // Same 128 px block as the field above, so the phase repeats the prologue.
    let PassPrologue { start_x, loops, .. } = pass_prologue(screen, screen_pos, num, cave_parallax);
    state.start_x = start_x;
    state.loops = loops;
    state.top_y = state.start_y + state.loops_y as f32 * block_h;
    *hell_black_box_bottom = state.top_y + screen_pos.y;
    let Some(bottom) = back_texture(sky, back[6]) else {
        return;
    };
    for i in 0..state.loops {
        for j in 0..(num as i32 / 16) {
            let cell_x = state.start_x + num * i as f32 + 16.0 * j as f32;
            push_cave_cell(
                r,
                camera,
                zoom,
                screen,
                bottom,
                cave_cell_sx(bottom, 16.0 * j as f32 + num2 + 16.0),
                (magma_bg_frame * 16) as f32,
                [cell_x + num2, state.top_y],
                [cell_x + 8.0, state.top_y],
                light,
                bg_brightness,
            );
        }
    }
}

/// Push a sprite with a screen-space position/size, compensating for the camera zoom.
/// Used by the sky, sun/moon, stars and parallax background layers.
pub(crate) fn push_screen(
    r: &mut SpriteRenderer,
    camera: Vec2,
    zoom: f32,
    screen: Vec2,
    pos: [f32; 2],
    size: [f32; 2],
    uv: [f32; 4],
    color: [f32; 4],
) {
    let wx = camera.x + (pos[0] - screen.x * 0.5) / zoom;
    let wy = camera.y + (pos[1] - screen.y * 0.5) / zoom;
    r.push_back_sprite([wx, wy], [size[0] / zoom, size[1] / zoom], uv, color);
}

/// Like [`push_screen`] but rotates the sprite about its centre (the original's
/// `origin = size / 2`). `pos` is the *centre* in screen space; the renderer's
/// rotation pivot is the top-left, so back the offset off by the rotated half-size.
#[allow(clippy::too_many_arguments)]
pub(crate) fn push_screen_rot(
    r: &mut SpriteRenderer,
    camera: Vec2,
    zoom: f32,
    screen: Vec2,
    pos: [f32; 2],
    size: [f32; 2],
    uv: [f32; 4],
    color: [f32; 4],
    rot: f32,
) {
    let (s, c) = rot.sin_cos();
    let (hx, hy) = (size[0] * 0.5, size[1] * 0.5);
    let tl = [pos[0] - (hx * c - hy * s), pos[1] - (hx * s + hy * c)];
    let wx = camera.x + (tl[0] - screen.x * 0.5) / zoom;
    let wy = camera.y + (tl[1] - screen.y * 0.5) / zoom;
    r.push_back_sprite_rot([wx, wy], [size[0] / zoom, size[1] / zoom], uv, color, rot);
}

/// The ambience draw state threaded through a biome's front-row chain: the
/// entities, their sheets, the draw context and the `SkyManager` depth
/// tracker (`SkyManager.cs:53-60`). `draw_front_trees` emits one
/// `DrawToDepth(1f/parallax)` boundary before each row, like the C# biome
/// functions do (`Main.cs:61304+`).
pub(crate) struct AmbiencePass<'a> {
    pub entities: &'a [crate::ambience::SkyEntity],
    pub lookups: &'a crate::assets::Lookups,
    pub ctx: &'a crate::ambience::SkyDrawCtx<'a>,
    pub last: &'a mut f32,
}

/// The three foreground layers of one biome (`DrawSurfaceBG_Forest` /
/// `_Desert` / `_Snow`).
///
/// `ks`/`tops` are the `backgroundTopMagicNumber * k + c + scAdj` terms from the
/// original; `pushBGTopHack` is already folded into `tops` (front layers use
/// `-150`). Every layer is tinted by `ColorOfSurfaceBackgroundsBase * alpha`,
/// where `alpha` is the biome's `Main.bgAlphaFrontLayer` entry.
#[allow(clippy::too_many_arguments)]
pub(crate) fn draw_front_trees(
    r: &mut SpriteRenderer,
    camera: Vec2,
    zoom: f32,
    screen: Vec2,
    trees: &[Option<BgTex>; 3],
    alpha: f32,
    scales: [f32; 3],
    parallax: [f32; 3],
    ks: [f32; 3],
    tops: [f32; 3],
    flip_layer: Option<usize>,
    magic: f32,
    sc_adj: f32,
    tint: [f32; 3],
    mut ambience: Option<&mut AmbiencePass<'_>>,
    // Only the Snow body ends with `DrawToDepth(1f)` (`Main.cs:60186`); the
    // other biomes leave the stratum to the flush (`Main.cs:59626`).
    closes_with_one: bool,
) {
    if alpha <= 0.0 {
        return;
    }
    for i in 0..3 {
        // The biome's row boundary: `bgParallax = parallax[i]; DrawToDepth(
        // 1f/bgParallax)` runs before the row draws (`Main.cs:61304-61306`
        // and its siblings).
        if let Some(p) = ambience.as_deref_mut() {
            crate::ambience::draw_range_min(
                r,
                p.entities,
                p.lookups,
                p.ctx,
                1.0 / parallax[i],
                p.last,
            );
        }
        if let Some(t) = trees[i] {
            draw_bg_layer(
                r,
                camera,
                zoom,
                screen,
                &t,
                scales[i],
                parallax[i],
                magic * ks[i] + tops[i] + sc_adj,
                flip_layer == Some(i),
                tint,
                alpha,
            );
        }
    }
    // The Snow chain closes with `DrawToDepth(1f)` (`Main.cs:60186`); the
    // other biomes' strata wait for the flush.
    if closes_with_one && let Some(p) = ambience.as_deref_mut() {
        crate::ambience::draw_range_min(r, p.entities, p.lookups, p.ctx, 1.0, p.last);
    }
}

/// `Main.ComputeScAdj`: small vertical screen-space correction for backgrounds.
#[allow(clippy::too_many_arguments)]
pub(crate) fn draw_bg_layer(
    r: &mut SpriteRenderer,
    camera: Vec2,
    zoom: f32,
    screen: Vec2,
    tex: &BgTex,
    scale: f32,
    parallax: f32,
    top: f32,
    flip: bool,
    tint: [f32; 3],
    alpha: f32,
) {
    // Terraria renders backgrounds through `BackgroundViewMatrix`, whose zoom is
    // `ForcedMinimumZoom` (1): the layers keep their screen size at any game zoom,
    // and `bgTopY` is already a screen-space top, so no further transform applies.
    let sw = tex.w * scale;
    let sh = tex.h * scale;
    let uv = if flip {
        [tex.uv[2], tex.uv[1], tex.uv[0], tex.uv[3]]
    } else {
        tex.uv
    };
    // `bgStartX = (int)(-IEEERemainder(screenPosition.X * bgParallax, bgWidthScaled)
    // - bgWidthScaled / 2)`: phase against the screen's left edge, not the camera
    // centre, and half a tile of offset so the tiling is centred on the view.
    let screen_left = camera.x - screen.x * 0.5;
    let start = -(ieee_remainder(screen_left * parallax, sw) as i32 as f32) - sw * 0.5;
    let mut x = start;
    while x < screen.x {
        push_screen(
            r,
            camera,
            zoom,
            screen,
            [x, top],
            [sw, sh],
            uv,
            [tint[0], tint[1], tint[2], alpha],
        );
        x += sw;
    }
}

/// `Main.ComputeScAdj`: a screen-height/altitude correction subtracted from every
/// background layer's top.
pub(crate) fn bg_scale_adjust(world: &World, view_top: f32, view_height: f32) -> f32 {
    let world_surface_px = (world.world_surface as f32 * TILE_PX).max(1.0);
    let num = world_surface_px / (view_top + view_height).max(1.0);
    let mut altitude = world.height as f32 * 0.15 * TILE_PX - view_top;
    if altitude < 0.0 {
        altitude = 0.0;
    }
    altitude *= 0.00025;
    let num = num * (0.45 - altitude * altitude);
    let factor = if world.height <= 1200 {
        -500.0
    } else if world.height <= 1800 {
        -300.0
    } else {
        -150.0
    };
    num * factor
}

#[cfg(test)]
mod underground_bg_tests {
    use super::*;
    use crate::render::tiles::{WALL_ECHO, wall_visible};

    fn world() -> World {
        let mut w = World::new("t".into(), 4200, 1200);
        w.world_surface = 356.0;
        w.rock_layer = 516.0;
        w
    }

    #[test]
    fn layout_matches_dirt_background_with_identity_background_matrix() {
        // `Terrustaria.trwld` reference camera: tile (2196, 361) at 1600x900.
        let w = world();
        let cam = Vec2::new(2196.0 * TILE_PX, 361.0 * TILE_PX);
        let l = underground_bg_layout(&w, cam, 1.28, Vec2::new(1600.0, 900.0));
        assert_eq!(l.screen_pos, Vec2::new(cam.x - 800.0, cam.y - 450.0));
        // bgTopY = worldSurface*16 - screenPosition.Y  (+16 for the dirt sheet)
        assert_eq!(l.transition_top, 356.0 * 16.0 - (cam.y - 450.0));
        assert_eq!(l.dirt_top, l.transition_top + 16.0);
    }

    #[test]
    fn backdrop_is_zoom_invariant() {
        // `Main.BackgroundViewMatrix.Zoom == ForcedMinimumZoom == 1`, so the game zoom
        // must not move the backdrop: only `ScaledPosition`, the rock clamp, does.
        let w = world();
        let cam = Vec2::new(2196.0 * TILE_PX, 361.0 * TILE_PX);
        let screen = Vec2::new(1600.0, 900.0);
        let a = underground_bg_layout(&w, cam, 1.0, screen);
        let b = underground_bg_layout(&w, cam, 2.0, screen);
        assert_eq!(a.transition_top, b.transition_top);
        assert_eq!(a.dirt_top, b.dirt_top);
        // The rock clamp *is* scaled: `camera.Y - screenHeight / (2 * zoom)`.
        assert_eq!(a.scaled_bottom, cam.y - 450.0 + 900.0);
        assert_eq!(b.scaled_bottom, cam.y - 225.0 + 900.0);
    }

    #[test]
    fn cave_parallax_maps_the_slider_range() {
        // `Main.caveParallax = 1f - (float)Main.bgScroll / 500f` (`Main.cs:50383`,
        // `:51643`): a 0.37 bar value stores bgScroll 37 (`:51642`), which the
        // underground backdrop then scrolls by 0.926.
        assert_eq!((0.37f32 * 100.0) as u8, 37);
        assert!((cave_parallax(37) - 0.926).abs() < 1e-6);
        assert_eq!(cave_parallax(0), 1.0);
        assert!((cave_parallax(100) - 0.8).abs() < 1e-6);
    }

    #[test]
    fn echo_walls_are_invisible_like_walldrawing() {
        // `WallDrawing.DrawWalls` skips wall 318 (`WallID.EchoWall`) unless the debug
        // "show invisible walls" option is on, so it neither draws nor paints black.
        assert!(!wall_visible(0));
        assert!(!wall_visible(WALL_ECHO));
        assert!(wall_visible(1));
        assert!(wall_visible(350));
    }

    #[test]
    fn rock_transition_point_matches_getrocktransitionpoint() {
        // num = (516-356)*16 = 2560; 2560/96 = 26.67 -> 27 blocks; 356*16 + 27*96 + 32.
        let w = world();
        assert_eq!(
            get_rock_transition_point(&w, 96.0),
            5696.0 + 27.0 * 96.0 + 32.0
        );
        // Exact multiple of the block height must not round up.
        assert_eq!(
            get_rock_transition_point(&w, 2560.0),
            5696.0 + 2560.0 + 32.0
        );
    }

    #[test]
    fn rock_transition_point_truncates_each_row_first() {
        // `Main.cs:53090`: `(int)rockLayer - (int)worldSurface` truncates both rows
        // before the difference, so a fractional layer line must not leak into the
        // span; the base is `(int)worldSurface * 16` (`Main.cs:53097`).
        let mut w = world();
        w.world_surface = 356.4;
        w.rock_layer = 516.9;
        // (516 - 356) * 16 = 2560 -> 27 blocks of 96.
        assert_eq!(
            get_rock_transition_point(&w, 96.0),
            5696.0 + 27.0 * 96.0 + 32.0
        );
    }

    #[test]
    fn magma_layer_truncates_the_span_to_six_row_blocks() {
        // `Main.cs:52812-52813`: ((1200 - 330) - 356) / 6 = 85.67 -> 85 blocks.
        let w = world();
        assert_eq!(magma_layer_of(&w), 356.0 + 85.0 * 6.0 - 5.0);
    }

    #[test]
    fn background_brightness_ramps_down_over_300_px() {
        // `Main.cs:52820-52821` with `Lighting.cs:92`'s reset to 1.2: full brightness
        // at and above the surface line, 1.0 once the view top is 300 px below it.
        let surface_px = 356.0 * TILE_PX;
        assert_eq!(background_brightness(surface_px, 356.0), 1.2);
        assert_eq!(background_brightness(surface_px - 1000.0, 356.0), 1.2);
        assert!((background_brightness(surface_px + 300.0, 356.0) - 1.0).abs() < 1e-6);
        assert_eq!(background_brightness(surface_px + 2000.0, 356.0), 1.0);
    }

    #[test]
    fn background_styles_follow_hell_back_style() {
        // `Main.cs:53817-53824` + `:54000-54008`: the classic set with the animated
        // magma sheet `125 + hellBackStyle` and the hell floor `185 + hellBackStyle`.
        assert_eq!(
            draw_background_update_background_styles(0),
            [1, 2, 4, 3, 6, 125, 185]
        );
        assert_eq!(
            draw_background_update_background_styles(1),
            [1, 2, 4, 3, 6, 126, 186]
        );
        assert_eq!(
            draw_background_update_background_styles(2),
            [1, 2, 4, 3, 6, 127, 187]
        );
    }

    #[test]
    fn rock_cells_skip_covered_lit_and_dark_tiles() {
        // `Main.cs:53152-53159`.
        let lit = [0.5, 0.4, 0.3];
        let dark = [0.0, 0.0, 0.0];
        // Covered by an active tile yet lit -> skipped (the tile hides the backdrop).
        assert!(rock_cell_skipped(false, false, 0.88, lit, 1.2));
        // The wall light of a neighbour keeps it drawn (`ShouldDrawBackgroundTileAt`).
        assert!(!rock_cell_skipped(true, false, 0.88, lit, 1.2));
        // Fully dark -> skipped.
        assert!(rock_cell_skipped(true, true, 0.88, dark, 1.2));
        // No light wall and zero parallax -> skipped even when lit.
        assert!(rock_cell_skipped(false, false, 0.0, lit, 1.2));
        // The zero tests run on byte colours: anything below one 1/255 step after the
        // GlobalBrightness fold truncates to zero, anything above stays lit.
        let faint = [0.01; 3]; // 0.01 * 1.2 * 255 = 3.06 -> byte 3
        assert!(!rock_cell_skipped(true, true, 0.88, faint, 1.2));
        let dim = [0.003 / 255.0; 3]; // 0.003 * 1.2 * 255 = 0.918 -> byte 0
        assert!(rock_cell_skipped(true, true, 0.88, dim, 1.2));
    }

    #[test]
    fn magma_cells_draw_even_when_dark_below_maxtilesy_minus_300() {
        // `Main.cs:52907`: the fully-dark skip stops at `maxTilesY - 300`.
        let dark = [0.0, 0.0, 0.0];
        assert!(magma_cell_skipped(true, true, 0.88, dark, 1.2, 900, 1200));
        assert!(!magma_cell_skipped(true, true, 0.88, dark, 1.2, 901, 1200));
    }

    #[test]
    fn magma_bg_frame_cycles_three_frames_every_eight_ticks() {
        // `Main.cs:62218-62224`: one tick per update (60/s), a frame every 8 ticks.
        assert_eq!(magma_bg_frame(0.0), 0);
        assert_eq!(magma_bg_frame(7.0 / 60.0), 0);
        assert_eq!(magma_bg_frame(8.0 / 60.0), 1);
        assert_eq!(magma_bg_frame(16.0 / 60.0), 2);
        assert_eq!(magma_bg_frame(24.0 / 60.0), 0);
    }

    #[test]
    fn worldgen_rolls_hell_back_style() {
        // `WorldGen.cs:8247`: one `genRand.Next(3)` draw; the port fixes it per seed
        // (drawn after the moon-type roll, so no other roll shifts).
        let opts = tr_core::worldgen::GenOptions {
            width: 512,
            height: 512,
            seed: 42,
            ..Default::default()
        };
        let a = tr_core::worldgen::generate(&opts);
        let b = tr_core::worldgen::generate(&opts);
        assert_eq!(a.hell_back_style, b.hell_back_style);
        assert!(a.hell_back_style < 3);
    }
}
