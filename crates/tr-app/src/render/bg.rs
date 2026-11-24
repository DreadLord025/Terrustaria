//! The underground background passes (`Main.DrawBackground` family): the
//! surface transition row, the dirt sheet, the underworld black box, the rock
//! bands, the magma field and the screen-space parallax layer helpers the sky
//! pass shares.
//!
//! The `ugBackTransition` cross-fade overlays are WIRED (W4-g72 closeout):
//! `UgOld` resolves the old-style sheet table once per frame
//! (`Main.cs:53804-53810`'s second switch run) and every pass draws its
//! overlay while `transition > 0` — the per-cell twins through
//! `push_cave_cell`'s fade arm (tint via `bg_target::ug_crossfade_tint`,
//! row pitch `backgroundHeight[oldBackTexture[slot]]`), the surface row and
//! the dirt sheet as SECOND whole loops re-phased against the OLD sheet's
//! width (`Main.cs:53768-53797` / `:53651-53694`, the dirt one corner-lit).
//! The magma FIELD (slot 5) never fades in the C# — census in
//! `draw_background_draw_magma_layer`. Non-classic old sheets are
//! atlas-blocked (d35 research/113 §8.4) and their overlays skip.
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

/// The `ugBackTransition` cross-fade inputs (`Main.cs:53075-53082` and its six
/// twins — W4-g72 closeout WIRED). `sheet` is the OLD style's resolved slot
/// sheet (`None` while the per-biome sheets are atlas-blocked, d35 research/113
/// §8.4 — the overlay skips, never a wrong sheet).
#[derive(Clone, Copy)]
struct UgFade<'a> {
    sheet: Option<&'a BgTex>,
    transition: f32,
    /// The OLD sheet's row pitch (`Main.backgroundHeight[oldBackTexture[slot]]`);
    /// the NEW pass's pitch when the old sheet is unresolved — the overlay
    /// skips then, so the pitch never matters.
    old_pitch: f32,
}

/// The OLD-style sheet table + fade, resolved once per `DrawBackground` frame
/// (`Main.DrawBackground_UpdateBackgroundStyles` runs its switch twice,
/// `Main.cs:53804-53810` — `undergroundBackground` then
/// `oldUndergroundBackground`). `h[slot]` is the resolved OLD sheet's
/// `backgroundHeight`: the dirt/rock overlay draws carry
/// `backgroundHeight[oldBackTexture[slot]] * j` (`Main.cs:53632`, `:53313`),
/// so an UNRESOLVED old sheet (atlas-blocked) falls back to the NEW pass's
/// pitch — the overlay skips anyway, the pitch never matters then. The
/// single-row passes (surface row, rock band, magma band, hell floor) have no
/// `j` term at all.
struct UgOld<'a> {
    sheets: [Option<&'a BgTex>; 7],
    h: [f32; 7],
    transition: f32,
}

impl<'a> UgOld<'a> {
    fn fade(&self, slot: usize, fallback_h: f32) -> UgFade<'a> {
        UgFade {
            sheet: self.sheets[slot],
            transition: self.transition,
            old_pitch: self.pitch(slot, fallback_h),
        }
    }

    /// The OLD sheet's row pitch for slot `slot`, the NEW pass's `new_h` when
    /// the old sheet is unresolved (`Main.backgroundHeight[oldBackTexture]`).
    fn pitch(&self, slot: usize, new_h: f32) -> f32 {
        if self.sheets[slot].is_some() { self.h[slot] } else { new_h }
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
    fade: UgFade<'_>,
    row_j: i32,
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
    let wpos = [
        camera.x + (pos[0] - screen.x * 0.5) / zoom,
        camera.y + (pos[1] - screen.y * 0.5) / zoom,
    ];
    let size = [16.0 / zoom, 16.0 / zoom];
    r.push_back_sprite(
        wpos,
        size,
        [
            tex.uv[0] + du * sx,
            tex.uv[1] + dv * sy,
            tex.uv[0] + du * (sx + 16.0),
            tex.uv[1] + dv * (sy + 16.0),
        ],
        [c[0] * k, c[1] * k, c[2] * k, 1.0],
    );
    // `Main.cs:53075-53082`: the OLD style's sheet draws over the new one while
    // `ugBackTransition > 0`, every channel (incl. alpha) byte-truncated by the
    // transition. The tint is the same lit colour the base cell just drew
    // (`color7 = color6` AFTER the 0.9 multiply), so the port folds it through
    // the landed `bg_target::ug_crossfade_tint`. The row pitch is the OLD
    // sheet's (`backgroundHeight[oldBackTexture[slot]] * j`); the source rect,
    // jitter and sample point stay the NEW pass's.
    if fade.transition > 0.0 && let Some(old) = fade.sheet {
        let pos_y = pos[1] + (fade.old_pitch - tex.h) * row_j as f32;
        if pos_y < screen.y && pos_y + 16.0 > 0.0 {
            let ou = (old.uv[2] - old.uv[0]) / old.w;
            let ov = (old.uv[3] - old.uv[1]) / old.h;
            let osx = cave_cell_sx(old, sx);
            r.push_back_sprite(
                [wpos[0], camera.y + (pos_y - screen.y * 0.5) / zoom],
                size,
                [
                    old.uv[0] + ou * osx,
                    old.uv[1] + ov * sy,
                    old.uv[0] + ou * (osx + 16.0),
                    old.uv[1] + ov * (sy + 16.0),
                ],
                super::bg_target::ug_crossfade_tint([c[0] * k, c[1] * k, c[2] * k, 1.0], fade.transition),
            );
        }
    }
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
/// `_drawBackground_backTexture[0..7]` sheet table. The style pick
/// (`DrawBackground_PickUndergroundBackgroundStyle`, `Main.cs:54033`) is LIVE
/// since W4-g72 final: `draw_background` builds a `BgStylePickInput` from the
/// World's rolled/persisted cave arrays + the frame's `SceneMetrics` and the
/// picked style feeds `bg_styles::background_style_sheets` (the full 22-case
/// table). The per-biome SHEETS are still atlas-blocked (d35 research/113
/// §8.4 per-world bg pages), so `back_texture` resolves only the classic-set
/// ids and a non-classic pick gracefully degrades (missing sheet → pass
/// skipped, never a wrong sheet). `hellBackStyle` (a per-world roll,
/// `WorldGen.cs:8247`) still selects the animated magma sheets
/// (`Main.cs:54000-54008`) — those three ARE loaded.
fn draw_background_update_background_styles(
    style: i32,
    world: &World,
) -> [usize; 7] {
    let back = super::bg_styles::background_style_sheets(
        style,
        world.ice_back_style as i32,
        world.jungle_back_style as i32,
        world.hell_back_style as i32,
        world.world_id,
    );
    back.map(|id| id as usize)
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

/// `Main.WallLightAt(i, j, showInvisibleWalls)` (`Main.cs:55169-55181`): a wall
/// transmits sky light when `Main.wallLight[tile.wall]` holds; otherwise the cell
/// still counts while it carries the Echo coating (`tile.invisibleWall()`) and the
/// invisible walls are NOT being shown. The coating IS set in the original —
/// `WorldGen.DoCoatEverythingEcho` (`WorldGen.cs:966/970/976`) and the `.wld` load
/// (`IO/WorldFile.cs:2672`) — so the old "never set, table alone decides" shortcut
/// dropped the arm; an Echo-coated wall must keep lighting the parallax backdrop
/// exactly like a lit wall does. `show_invisible_walls` is
/// `Main.ShouldShowInvisibleBlocksAndWalls` (`Main.cs:58023-58030`): the port has no
/// Echo Monolith / see-invisible accessory yet, so it rides the `TR_SHOW_INVISIBLE`
/// A/B pin (`env::show_invisible`), false in normal play.
fn wall_light_at(world: &World, x: i32, y: i32, show_invisible_walls: bool) -> bool {
    use tr_core::tiledata::{WALL_COUNT, WALL_LIGHT};
    let Some(t) = world.tile(x, y) else {
        return false;
    };
    if (t.wall as usize) < WALL_COUNT && WALL_LIGHT[t.wall as usize] {
        return true; // Main.cs:55172-55180: wallLight[wall] -> true
    }
    if !show_invisible_walls {
        return world.invisible_wall(x, y); // Main.cs:55176: tile.invisibleWall()
    }
    false // Main.cs:55178
}

/// `Main.ShouldDrawBackgroundTileAt` (`Main.cs:55136-55162`): with a non-zero
/// parallax the wall light of the horizontal neighbours counts too (`:55152`);
/// at zero parallax only the cell itself (`:55157`). Both arms read the same
/// `WallLightAt(i, j, ShouldShowInvisibleBlocksAndWalls())` (`:55164-55167`).
fn should_draw_background_tile_at(
    world: &World,
    x: i32,
    y: i32,
    cave_parallax: f32,
    show_invisible_walls: bool,
) -> bool {
    if cave_parallax != 0.0 {
        wall_light_at(world, x, y, show_invisible_walls)
            || wall_light_at(world, x - 1, y, show_invisible_walls)
            || wall_light_at(world, x + 1, y, show_invisible_walls)
    } else {
        wall_light_at(world, x, y, show_invisible_walls)
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
    // `Main.shimmerAlpha` (`Main.cs:52800`): the SceneState ramp — 1 skips the
    // whole pass (W4-e19 aether arm).
    shimmer_alpha: f32,
) {
    // `Main.cs:52800-52805`: full shimmer skips the pass (the aether's black
    // backdrop replaces it) and zeroes `ugBackTransition`. The port pins
    // `undergroundBackground` 0 (the cross-fade never runs — see
    // `draw_background_update_background_styles`), so the zeroing is a no-op
    // here; g72's live `UgBackState` merge must zero its transition on this arm.
    if shimmer_alpha == 1.0 {
        return;
    }
    // `Main.ShouldShowInvisibleBlocksAndWalls` (`Main.cs:58023-58030`): the Echo
    // coating's see-invisible gate the wall-parallax `WallLightAt` reads. The port
    // has no Echo Monolith / accessory yet, so it rides the `TR_SHOW_INVISIBLE` pin.
    show_invisible_walls: bool,
    // `Main.SceneMetrics` (the frame's tile-window scan) — the style pick's
    // snow/jungle counts + zone flags (`Main.cs:54038-54093`). W4-g72 final.
    metrics: &tr_core::scene::SceneMetrics,
    // `Main.undergroundBackground`/`oldUndergroundBackground`/`ugBackTransition`
    // (`Main.cs:52823-52841`) — carried across frames like `hellBlackBoxBottom`.
    ug_back: &mut super::bg_styles::UgBackState,
    // `Main.shimmerAlpha` (`Main.cs:52800`): 1 skips the whole pass, >0 kills
    // the cross-fade. The port's shimmer ramp is not wired yet (draw.rs feeds
    // 0.0), so this stays 0 in normal play.
    shimmer_alpha: f32,
) {
    // `Main.cs:52800-52805`: full shimmer skips the pass and zeroes the fade.
    if shimmer_alpha == 1.0 {
        ug_back.transition = 0.0;
        return;
    }
    // `Main.backgroundHeight[2]` / the rock-transition snap keep reading the
    // DIRT sheet's dims even for non-classic styles (the C# loads
    // `Background_2` at boot and `GetRockTransitionPoint` reads its height);
    // the classic-set guard stays for that reason.
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
    // cross-fade — LIVE since W4-g72 final: the pick reads the World's rolled cave
    // arrays + the frame's SceneMetrics (`bg_style_pick_input`), the latch/decay
    // runs `ug_back_step`. W4-g72 closeout: the cross-fade OVERLAY draws
    // (`_drawBackground_oldBackTexture`) are WIRED too — the old-style table
    // resolves below and every pass draws its overlay while `transition > 0`.
    // The per-biome sheets are atlas-blocked (d35 research/113 §8.4), so a
    // non-classic old slot resolves `None` and its overlay skips (graceful
    // degradation — never a wrong sheet); the classic↔classic fades draw today.
    let picked = super::bg_styles::pick_underground_background_style(
        &super::bg_styles::bg_style_pick_input(
            world,
            metrics,
            [screen_pos.x, screen_pos.y],
            [screen.x, screen.y],
            magma_layer,
        ),
    );
    super::bg_styles::ug_back_step(ug_back, picked, shimmer_alpha);
    let back = draw_background_update_background_styles(ug_back.style, world);
    // `Main.DrawBackground_UpdateBackgroundStyles`' second switch run
    // (`Main.cs:53804-53810`): the OLD style's table, resolved once per frame.
    let old_back = draw_background_update_background_styles(ug_back.old_style, world);
    let old_sheets: [Option<&BgTex>; 7] = old_back.map(|id| back_texture(sky, id));
    let old = UgOld {
        sheets: old_sheets,
        h: old_sheets.map(|t| t.map_or(0.0, |t| t.h)),
        transition: ug_back.transition,
    };
    // W4-g72 final: the dirt pass draws `back[1]` (the picked style's dirt
    // sheet), not the pinned `Background_2`. The per-biome sheets are
    // atlas-blocked (d35 research/113 §8.4), so `back_texture` resolves only
    // the classic id 2 today; a non-classic pick skips the dirt cells
    // (graceful degradation — never the WRONG biome's sheet). The dims
    // (`dirt.h` for the rock snap / row pitch) keep reading `Background_2`
    // like the C#'s `backgroundHeight[2]`.
    let dirt_sheet = back_texture(sky, back[1]);
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
        &old,
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
        dirt_sheet,
        &old,
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
        show_invisible_walls,
        &old,
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
        show_invisible_walls,
        &old,
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
    old: &UgOld<'_>,
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
    // The surface row's overlay is NOT a per-cell twin — the C# runs a second
    // whole loop below (`Main.cs:53768-53797`), so the base cells draw with the
    // fade disabled.
    let no_fade = UgFade { sheet: None, transition: 0.0, old_pitch: 0.0 };
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
                no_fade,
                0,
            );
        }
    }
    // `Main.cs:53768-53797`: the surface row's overlay is a SECOND whole loop,
    // re-phased against the OLD sheet's own width (`num = old.Width() - 32`,
    // `bgStartX`/`bgLoops` recomputed, the jitter re-derived) — not a per-cell
    // twin. The light is re-read at the OLD loop's sample points; the fade
    // tint truncates every channel incl. alpha (`:53790-53794`).
    if old.transition <= 0.0 {
        return;
    }
    let Some(old_tex) = old.sheets[0] else {
        return;
    };
    let num = old_tex.w - 32.0;
    if num < 16.0 {
        return;
    }
    let PassPrologue {
        start_x,
        loops,
        jitter: n5,
    } = pass_prologue(screen, screen_pos, num, cave_parallax);
    for i in 0..loops {
        for j in 0..(num as i32 / 16) {
            let cell_x = start_x + num * i as f32 + 16.0 * j as f32;
            let sp = screen_pos;
            let wx = ((cell_x + 8.0 + sp.x) / TILE_PX).floor() as i32;
            let wy = ((surface_screen + sp.y) / TILE_PX).floor() as i32;
            let c = light.get(wx, wy);
            let k = bg_brightness * 0.9;
            let color = super::bg_target::ug_crossfade_tint(
                [c[0] * k, c[1] * k, c[2] * k, 1.0],
                old.transition,
            );
            let du = (old_tex.uv[2] - old_tex.uv[0]) / old_tex.w;
            let sx = 16.0 * j as f32 + n5 + 16.0;
            push_screen(
                r,
                camera,
                zoom,
                screen,
                [cell_x + n5, surface_screen],
                [16.0, 16.0],
                [
                    old_tex.uv[0] + du * sx,
                    old_tex.uv[1],
                    old_tex.uv[0] + du * (sx + 16.0),
                    old_tex.uv[1] + (old_tex.uv[3] - old_tex.uv[1]) / old_tex.h * 16.0,
                ],
                color,
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
    // W4-g72 final: the picked style's `back[1]` sheet (`None` while the
    // per-biome sheets are atlas-blocked — the cells skip, the band below
    // still draws like the C#'s own missing-texture arm).
    dirt_sheet: Option<&BgTex>,
    old: &UgOld<'_>,
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
    // The C# reads `Background[backTexture[1]].Width()` for the tiling block;
    // the port keeps `Background_2`'s dims for the geometry (all dirt-family
    // sheets share the 128x96 shape) and draws the PICKED sheet's pixels.
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
    //
    // The dirt cells' per-cell overlay twins (`Main.cs:53560-53567` /
    // `:53608-53615` / `:53625-53632`) live ONLY inside the 9-slice/4-slice
    // arms, which the port collapses to the plain cell (`Main.cs:53636-53645`
    // — the plain arm draws NO overlay). The dirt overlay is instead the
    // SECOND whole corner-lit loop after the cells (`Main.cs:53651-53694`),
    // ported below — so the base cells draw with the fade disabled.
    let no_fade = UgFade { sheet: None, transition: 0.0, old_pitch: 0.0 };
    if loops_y > 0 {
        if let Some(dirt_tex) = dirt_sheet {
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
                                dirt_tex,
                                16.0 * k as f32 + n2 + 16.0,
                                16.0 * l as f32,
                                [cell_x + n2, cell_y],
                                [cell_x + 8.0, cell_y + 8.0],
                                light,
                                bg_brightness,
                                no_fade,
                                j,
                            );
                        }
                    }
                }
            }
        }
    }
    // `Main.cs:53651-53694`: the dirt overlay — a SECOND whole loop over the
    // OLD sheet's own width, corner-lit (`GetCornerColors(.., ugBackTransition)`
    // scales the corner sum by the transition, `Lighting.cs:299`), every
    // corner's alpha `(byte)(255 * transition)` (`:53682-53686`). The port's
    // quad pipeline has no per-vertex colours, so each cell takes the bilinear
    // corner sample at its centre (the same approximation `push_wall_lit`
    // documents) — the transition rides the sample AND the alpha. The light
    // gate is `InWorld` + any-channel-nonzero (`:53672-53679`), NOT the base
    // pass's wall-light gate; the row count is the base pass's `bgLoopsY`.
    if old.transition > 0.0 && loops_y > 0 {
        if let Some(old_tex) = old.sheets[1] {
            let onum = old_tex.w - 32.0;
            if onum >= 16.0 {
                let PassPrologue {
                    start_x: osx,
                    loops: oloops,
                    jitter: on2,
                } = pass_prologue(screen, screen_pos, onum, cave_parallax);
                let a = ((255.0 * old.transition) as u8) as f32 / 255.0;
                for i in 0..oloops {
                    for j in 0..loops_y {
                        for k in 0..(onum as i32 / 16) {
                            for l in 0..6 {
                                let sample_y = start_y + j as f32 * 96.0 + l as f32 * 16.0 + 8.0;
                                let cell_x = osx + onum * i as f32 + 16.0 * k as f32;
                                let tx = ((cell_x + 8.0 + screen_pos.x) / TILE_PX) as i32;
                                let ty = ((sample_y + screen_pos.y) / TILE_PX) as i32;
                                // Main.cs:53672: InWorld WITHOUT the 1 margin.
                                if tx < 0 || tx >= world.width || ty < 0 || ty >= world.height {
                                    continue;
                                }
                                let c = light.get(tx, ty);
                                if !light_any_nonzero(c, bg_brightness) {
                                    continue; // Main.cs:53679
                                }
                                let corners = light.corner4(tx, ty);
                                let cc = super::tiles::bilerp_corners(corners, 0.5, 0.5);
                                let k2 = bg_brightness * old.transition;
                                let cell_y = start_y + old_tex.h * j as f32 + 16.0 * l as f32;
                                let du = (old_tex.uv[2] - old_tex.uv[0]) / old_tex.w;
                                let dv = (old_tex.uv[3] - old_tex.uv[1]) / old_tex.h;
                                let sx = cave_cell_sx(old_tex, 16.0 * k as f32 + on2 + 16.0);
                                push_screen(
                                    r,
                                    camera,
                                    zoom,
                                    screen,
                                    [cell_x + on2, cell_y],
                                    [16.0, 16.0],
                                    [
                                        old_tex.uv[0] + du * sx,
                                        old_tex.uv[1] + dv * 16.0 * l as f32,
                                        old_tex.uv[0] + du * (sx + 16.0),
                                        old_tex.uv[1] + dv * (16.0 * l as f32 + 16.0),
                                    ],
                                    [
                                        (cc[0] * k2).min(1.0),
                                        (cc[1] * k2).min(1.0),
                                        (cc[2] * k2).min(1.0),
                                        a,
                                    ],
                                );
                            }
                        }
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
    // Single-row pass: the overlay twin (`Main.cs:53723-53730`) has no `j`
    // row term — the fade's pitch never applies.
    let band_fade = old.fade(2, band.h);
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
                band_fade,
                0,
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
    show_invisible_walls: bool,
    old: &UgOld<'_>,
) {
    let _ = (q1, q2, local_shimmer_alpha, draw_offset);
    // The 9-slice/4-slice smoothing branches (`Main.cs:53156-53320`) are collapsed to
    // the plain cell, like the dirt pass (see ANALYSIS.md). The rock overlay twins
    // (`:53249-53256` / `:53289-53296` / `:53306-53313` / `:53327-53334`) cover ALL
    // four arms incl. the plain one — the collapsed cell keeps the overlay.
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
                        should_draw_background_tile_at(world, tx, ty, cave_parallax, show_invisible_walls),
                        wall_light_at(world, tx, ty, show_invisible_walls),
                        cave_parallax,
                        c,
                        bg_brightness,
                    ) {
                        continue; // Main.cs:53152-53159
                    }
                    let cell_y = state.start_y + tex.h * j as f32 + 16.0 * l as f32;
                    // `Main.cs:53305-53313`: the overlay's row pitch is
                    // `backgroundHeight[oldBackTexture[3]] * j` — the OLD
                    // sheet's height, not the new one's.
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
                        old.fade(3, tex.h),
                        j,
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
        old,
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
    old: &UgOld<'_>,
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
    // Single-row band (`Main.cs:53369-53376`): the overlay twin has no `j`
    // row term — the fade's pitch never applies.
    let band_fade = old.fade(4, tex.h);
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
                band_fade,
                0,
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
    show_invisible_walls: bool,
    old: &UgOld<'_>,
) {
    let _ = (q1, q2, draw_offset);
    // The 9-slice/4-slice smoothing branches (`Main.cs:52910-53049`) are collapsed to
    // the plain cell, like the dirt pass (see ANALYSIS.md). The magma FIELD has NO
    // overlay twin in ANY arm (`backTexture[5]` draws at `:53002/:53033/:53041/:53049`
    // never pair with `oldBackTexture[5]` — the C# never fades the animated hell
    // field itself); only the hell FLOOR row (slot 6) fades (`:53075-53082`).
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
                        should_draw_background_tile_at(world, tx, ty, cave_parallax, show_invisible_walls),
                        wall_light_at(world, tx, ty, show_invisible_walls),
                        cave_parallax,
                        c,
                        bg_brightness,
                        ty,
                        world.height,
                    ) {
                        continue; // Main.cs:52907
                    }
                    let cell_y = state.start_y + block_h * j as f32 + 16.0 * l as f32;
                    // The magma field never fades (the slot-5 census above).
                    let no_fade = UgFade { sheet: None, transition: 0.0, old_pitch: 0.0 };
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
                        no_fade,
                        j,
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
    // `Main.cs:53075-53082`: the hell floor row fades like the other single-row
    // passes — no `j` row term, the fade's pitch never applies.
    let floor_fade = old.fade(6, bottom.h);
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
                floor_fade,
                0,
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
    /// W4-c39 exec (research/114 §1e): the `PartySky` balloon field + its
    /// own depth tracker + the surface gate, threaded so each front-row
    /// `DrawToDepth` boundary also draws the party band (the f40
    /// `lantern_sky`/`lantern_last` shape).
    pub event_skies: &'a crate::event_skies::EventSkies,
    pub party_last: &'a mut f32,
    pub party_surface_px: f32,
    /// W4-f39: the SlimeSky rides the same depth boundaries (the c39
    /// `draw_balloons_band` shape) — `None` outside the event.
    pub slime_sky: Option<&'a crate::slime_sky::SlimeSky>,
}

/// W4-f39: one `SkyManager.DrawDepthRange` boundary for BOTH sandwich
/// members — the ambient entities (`AmbientSky`) and the SlimeSky's
/// falling slimes. The tracker moves once, exactly like the C# manager's
/// `_lastDepth` (`SkyManager.cs:53-60`); the two draws share the band.
pub(crate) fn draw_sandwich_band(r: &mut SpriteRenderer, p: &mut AmbiencePass<'_>, min_depth: f32) {
    if *p.last <= min_depth {
        return;
    }
    // The band is (min, last] BEFORE the tracker moves — the C# manager
    // draws every active sky over the same range (`DrawDepthRange`).
    let band_max = *p.last;
    crate::ambience::draw_range_min(r, p.entities, p.lookups, p.ctx, min_depth, p.last);
    if let Some(sky) = p.slime_sky {
        let origin = p.ctx.camera - p.ctx.screen * 0.5 / p.ctx.zoom;
        sky.draw_band(
            r,
            &p.lookups.slime_sky_sheets,
            p.ctx.camera,
            p.ctx.zoom,
            p.ctx.screen,
            origin,
            p.ctx.color_of_the_skies,
            min_depth,
            band_max,
        );
    }
    /// W4-f40: the `LanternSky` + its own depth tracker + the hourly bob
    /// phase, drawn at the same front-row boundaries as the ambience.
    pub lantern_sky: &'a crate::render::lantern_sky::LanternSky,
    pub lantern_last: &'a mut f32,
    pub global_time: f32,
}

/// One row of a `DrawSurfaceBG_*` chain (`Main.cs:59986-61370`).
///
/// `k`/`top` are the `backgroundTopMagicNumber * k + top + scAdj` terms of the
/// original's `bgTopY`, with `pushBGTopHack` already folded into `top` (the
/// front layers see `-150`: `30 - 180`, `Main.cs:59302`, `:59346`, `:59525`).
/// `depth` is the `SkyManager` boundary the row opens with — normally
/// `1f / bgParallax`, but Hallow's first row uses `1.2f / bgParallax`
/// (`Main.cs:60788`).
#[derive(Clone, Copy)]
pub(crate) struct FrontLayer {
    pub tex: Option<BgTex>,
    pub scale: f32,
    pub parallax: f32,
    pub k: f32,
    pub top: f32,
    pub depth: f32,
    /// `SpriteEffects.FlipHorizontally` on this row.
    pub flip: bool,
    /// `bgStartX` shift (Crimson `175`: `-1000`, `Main.cs:60289`).
    pub x_shift: f32,
    /// Extra tiles (`bgLoops++`, Crimson `175`, `Main.cs:60291`).
    pub extra: u8,
    /// `GetBackgroundRect` frame step; `0` draws the whole sheet.
    pub frame_step: u32,
    /// Per-row `ColorOfSurfaceBackgroundsModified` override (rgb, 0..1); only
    /// the Mushroom chain replaces it (`Main.cs:60001`, `:60065`, `:60127`).
    pub tint: Option<[f32; 3]>,
}

/// Build one biome's three rows from its `Main.<biome>BG` texture ids.
///
/// `base` is the hard-coded `(bgScale, bgParallax, num, num2)` of each row **in
/// call order** (the caller's `bgTopY = magic * num + num2 + scAdj + hack`).
/// `SetBackgroundOffsets` then overrides `bgScale`/`bgParallax` and, when its
/// `num`/`num2` pair is non-zero, replaces `bgTopY` wholesale
/// (`Main.cs:60771-60774`), so a style whose id is in that table wins over the
/// caller's magic numbers. `bgGlobalScaleMultiplier` is `num4 = 2f`
/// (`Main.cs:59329`); `hack` is the front loop's `pushBGTopHack` (`-150`).
///
/// Per-id row quirks (the `== 56` nudges, `GetLayerYOffset`) are the caller's,
/// because the C# applies them *after* `SetBackgroundOffsets`.
pub(crate) fn front_layers(
    ids: [i32; 3],
    base: [(f32, f32, f32, f32); 3],
    texs: [Option<BgTex>; 3],
    hack: f32,
) -> [FrontLayer; 3] {
    std::array::from_fn(|i| {
        let (mut scale, mut parallax, mut k, mut top) = base[i];
        if let Some((o_scale, o_parallax, num, num2)) = bg_offsets(ids[i]) {
            if let Some(p) = o_parallax {
                parallax = p;
            }
            if let Some(s) = o_scale {
                scale = s;
            }
            if num != 0 || num2 != 0 {
                k = num as f32;
                top = num2 as f32;
            }
        }
        FrontLayer {
            tex: texs[i],
            scale: scale * 2.0,
            parallax,
            k,
            top: top + hack,
            depth: 1.0 / parallax,
            flip: false,
            x_shift: 0.0,
            extra: 0,
            frame_step: background_frame_step(ids[i]),
            tint: None,
        }
    })
}

/// `Main.DrawSurfaceBG_Forest` (`Main.cs:61273-61519`) for one of the four
/// tree sets: the three rows with every per-id quirk in the C#'s order —
/// the row-0 scale/parallax overrides (`:61278-61301`), the row-0 top
/// rewrites + nudges (`:61311-61358`), the row-1/row-2 top rewrites + nudges
/// (`:61399-61436`, `:61473-61510`), the row-1 mirror on 51/54 (`:61440`)
/// and the row-2 `ids[0] == 91` scale/parallax drop (`:61457-61462`).
/// `ids[0] == 173` swaps row 0's DRAWN sheet to the animated 251..254 frame
/// (`:61371-61376`) while every offset/rect read keeps id 173 — the caller
/// passes that resolved frame as `frame_173`.
pub(crate) fn forest_layers(
    ids: [i32; 3],
    texs: [Option<BgTex>; 3],
    hack: f32,
    frame_173: Option<BgTex>,
) -> [FrontLayer; 3] {
    // The C#'s per-row write order is NOT the other biomes' (`front_layers`):
    // row 0 runs `SetBackgroundOffsets` at `:61302`, multiplies by 2 at
    // `:61303`, then runs it AGAIN at `:61307` with no re-multiply — a tabled
    // id's scale ends UN-multiplied. Rows 1/2 multiply FIRST (`:61389`,
    // `:61453`) and call the offsets once after (`:61391`, `:61455`), so
    // their tabled scales are un-multiplied too.
    let row = |i: usize, scale: f32, parallax: f32, k: f32, top: f32| FrontLayer {
        tex: texs[i],
        scale,
        parallax,
        k,
        top: top + hack,
        depth: 1.0 / parallax,
        flip: false,
        x_shift: 0.0,
        extra: 0,
        frame_step: background_frame_step(ids[i]),
        tint: None,
    };
    let mut rows = [
        row(0, 1.25, 0.40, 1800.0, 1500.0),
        row(1, 1.31 * 2.0, 0.43, 1950.0, 1750.0),
        row(2, 1.34 * 2.0, 0.49, 2100.0, 2000.0),
    ];
    // Row 0's pre-offset overrides (`Main.cs:61278-61301`), on the
    // un-multiplied scale.
    match ids[0] {
        91 => {
            rows[0].parallax = 0.27;
            rows[0].scale = 1.2;
        }
        173 => {
            rows[0].parallax = 0.25;
            rows[0].scale = 1.3;
        }
        178 => {
            rows[0].parallax = 0.30;
            rows[0].scale = 1.2;
        }
        184 => {
            rows[0].parallax = 0.25;
            rows[0].scale = 1.2;
        }
        282 => rows[0].scale = 1.4,
        _ => {}
    }
    // `SetBackgroundOffsets` pass 1 (`:61302`), the ×2 (`:61303`), then pass 2
    // (`:61307`) — the second pass's scale write wins without a re-multiply.
    // (282's 1.4 pre-write is dead: its own table entry 2.6 overwrites it.)
    // `apply_top`: row 0's top survives the offsets (its base write precedes
    // them, `:61277`); rows 1/2 rewrite `bgTopY` unconditionally AFTER the
    // call (`:61399`, `:61473`), so their table top pair is dead code.
    let apply_offsets = |r: &mut FrontLayer, id: i32, apply_top: bool| {
        if let Some((s, p, num, num2)) = bg_offsets(id) {
            if let Some(p) = p {
                r.parallax = p;
            }
            if let Some(s) = s {
                r.scale = s;
            }
            if apply_top && (num != 0 || num2 != 0) {
                r.k = num as f32;
                r.top = num2 as f32 + hack;
            }
        }
    };
    apply_offsets(&mut rows[0], ids[0], true);
    rows[0].scale *= 2.0;
    apply_offsets(&mut rows[0], ids[0], true);
    rows[0].depth = 1.0 / rows[0].parallax;
    // Rows 1/2: the single post-multiply offsets call (`:61391`, `:61455`).
    apply_offsets(&mut rows[1], ids[1], false);
    rows[1].depth = 1.0 / rows[1].parallax;
    apply_offsets(&mut rows[2], ids[2], false);
    rows[2].depth = 1.0 / rows[2].parallax;
    // Row 0's post-offset top rewrites + nudges (`Main.cs:61311-61358`).
    match ids[0] {
        91 | 173 | 184 => {
            rows[0].k = 1600.0;
            rows[0].top = 1400.0 + hack;
        }
        _ => {}
    }
    rows[0].top += match ids[0] {
        50 => -50.0,
        53 => -100.0,
        91 | 173 => 200.0,
        178 => 75.0,
        330 | 335 => -150.0,
        343 => -300.0,
        _ => 0.0,
    };
    // Row 1 (`Main.cs:61385-61445`): the top rewrites, the nudges, the mirror.
    match ids[1] {
        329 => {
            rows[1].k = 1900.0;
            rows[1].top = 1600.0 + hack;
        }
        334 => {
            rows[1].k = 1900.0;
            rows[1].top = 1650.0 + hack;
        }
        _ => {}
    }
    rows[1].top += match ids[1] {
        51 => -50.0,
        54 => -100.0,
        329 | 334 => -150.0,
        342 => -300.0,
        _ => 0.0,
    };
    rows[1].flip = ids[1] == 51 || ids[1] == 54;
    // Row 2 (`Main.cs:61447-61518`): the `ids[0] == 91` drop runs AFTER
    // `SetBackgroundOffsets` AND after the row's `DrawToDepth` (`:61456`),
    // so it wins over the table for the X phase but the depth boundary
    // keeps the pre-drop parallax.
    if ids[0] == 91 {
        rows[2].scale = 1.3 * 2.0;
        rows[2].parallax = 0.42;
    }
    match ids[2] {
        328 => {
            rows[2].k = 2600.0;
            rows[2].top = 2300.0 + hack;
        }
        333 => {
            rows[2].k = 2800.0;
            rows[2].top = 2550.0 + hack;
        }
        _ => {}
    }
    rows[2].top += match ids[2] {
        52 => -50.0,
        55 => -100.0,
        92 => 150.0,
        328 | 333 => -150.0,
        341 => -50.0,
        _ => 0.0,
    };
    // The 173 draw-time sheet swap (`Main.cs:61371-61376`): the frame is
    // `251 + (GetBackgroundCounter() / 10) % 4`, a whole sheet (no
    // `GetBackgroundRect` frame — that reads id 173, which has none).
    if ids[0] == 173 {
        rows[0].tex = frame_173;
    }
    rows
}

/// The 173 tree line's draw-time frame id (`Main.cs:61373`):
/// `251 + (GetBackgroundCounter() / 10) % 4`.
pub(crate) fn forest_173_frame_id(counter: u32) -> i32 {
    251 + (counter / 10 % 4) as i32
}

/// The Mushroom chain's per-row colour (`Main.cs:60001-60035` / `:60065-60099` /
/// `:60127-60157`): each channel lifted to the glow value, kept at or above the
/// base colour's byte.
///
/// `mul` is the row's `num` (0.5 / 0.625 / 0.75), `rgb` the row's `Vector3`
/// (already `* 3f`), `num5` the `Main.rand.Next(28, 42) * 0.001f +
/// (270 - Main.mouseTextColor) / divisor` roll, `alpha` the biome's
/// `bgAlphaFrontLayer[9]` — the C#'s `num4 = ColorOfSurfaceBackgroundsModified.A
/// / 255f`, i.e. that alpha (the base colour's own alpha is 255).
pub(crate) fn mushroom_tint(
    base: [f32; 3],
    alpha: f32,
    mul: f32,
    rgb: [f32; 3],
    num5: f32,
) -> [f32; 3] {
    let v = [rgb[0], rgb[1] + num5 * 0.5, rgb[2] + num5];
    std::array::from_fn(|i| {
        let x = (v[i] * 255.0 * mul * alpha).min(255.0) as i32;
        // `ColorOfSurfaceBackgroundsModified` = base * alpha, byte-truncated.
        let base_byte = (base[i] * alpha * 255.0) as i32;
        base_byte.max(x) as f32 / 255.0
    })
}

/// `Main.SetBackgroundOffsets` (`Main.cs:60384-60775`) for the texture ids the
/// eight surface-variant style tables can produce: the `bgScale`/`bgParallax`
/// overrides (either may be absent — `222`/`223`/`224`/`237`/`238`/`239` only
/// replace `bgTopY`, `261`/`262`/`219`/`220`/`221`/`321`/`207` only the
/// parallax), plus the `num`/`num2` pair that replaces `bgTopY` when non-zero.
/// Rows the C# does not list return `None` (the caller keeps its own defaults).
///
/// Deliberately absent: the underworld ids — that biome's style table rides
/// the W4-g72 lane. The forest tree-set ids ride here since W4-g70 (styles
/// 9..13; the classic 9..11/50..55 rows have no C# entry); the snow/desert/
/// ocean rows since W4-g71.
#[allow(clippy::type_complexity)]
pub(crate) fn bg_offsets(id: i32) -> Option<(Option<f32>, Option<f32>, i32, i32)> {
    Some(match id {
        // Snow styles 5/6/7/8 (`Main.cs:60390-60443`, `:60620-60630`).
        258 => (Some(1.25), Some(0.23), 1800, 850),
        259 => (Some(1.31), Some(0.33), 1950, 1500),
        260 => (Some(1.34), Some(0.40), 2100, 1650),
        263 => (Some(1.25), Some(0.23), 1800, 1450),
        264 => (Some(1.31), Some(0.33), 1950, 1700),
        265 => (Some(1.34), Some(0.40), 2100, 2000),
        267 => (Some(1.25), Some(0.23), 1700, 1300),
        266 => (Some(1.31), Some(0.33), 1950, 1600),
        268 => (Some(1.34), Some(0.41), 2100, 1850),
        298 => (Some(1.25), Some(0.35), 2000, 1700),
        299 => (None, Some(0.28), 1600, 1400),
        // Desert style 4 (`Main.cs:60444-60461`) and style 2's parallax-only
        // row (`:60617-60619`).
        248 => (Some(1.30), Some(0.37), 1800, 1100),
        249 => (Some(1.40), Some(0.43), 1950, 1200),
        250 => (Some(1.70), Some(0.49), 2000, 1000),
        207 => (None, Some(0.32), 0, 0),
        // Desert styles 51/52/53 (`Main.cs:60649-60672`): the middle rows
        // share one arm, the far rows another.
        303 | 304 | 305 | 307 | 308 | 309 | 311 | 312 | 313 | 315 | 316 | 317 => {
            (None, Some(0.35), 2400, 1900)
        }
        306 | 310 | 314 | 318 => (None, Some(0.26), 1800, 1700),
        240 => (Some(1.15), Some(0.30), 1800, 1500),
        241 => (Some(1.21), Some(0.43), 1950, 1300),
        242 => (Some(1.34), Some(0.49), 2100, 1400),
        243 => (Some(1.15), Some(0.25), 1800, 1400),
        244 => (Some(1.21), Some(0.35), 1950, 1550),
        245 => (Some(1.24), Some(0.45), 2100, 1650),
        273 => (None, Some(0.49), 2100, 1560),
        234 => (None, Some(0.23), 1700, 1150),
        235 => (None, Some(0.33), 1950, 1550),
        236 => (None, Some(0.41), 2100, 2000),
        261 => (None, Some(0.27), 0, 0),
        262 => (None, Some(0.40), 0, 0),
        219 => (None, Some(0.25), 0, 0),
        220 => (None, Some(0.34), 0, 0),
        221 => (None, Some(0.43), 0, 0),
        222 => (None, None, 1800, 1400),
        223 => (None, None, 2150, 1850),
        224 => (None, None, 2500, 2400),
        237 => (None, None, 1800, 1500),
        238 => (None, None, 1950, 1500),
        239 => (None, None, 2100, 1900),
        284 => (None, Some(0.23), 1600, 900),
        285 => (None, Some(0.36), 1900, 960),
        286 => (None, Some(0.42), 2100, 1200),
        287 => (None, Some(0.21), 1700, 1560),
        288 => (None, Some(0.33), 1950, 1730),
        289 => (None, Some(0.41), 2100, 1400),
        300 => (Some(2.41), Some(0.53), 3000, 2050),
        301 => (Some(2.36), Some(0.415), 2200, 1700),
        302 => (Some(2.25), Some(0.24), 1400, 950),
        319 => (None, Some(0.60), 4600, 3250),
        320 => (None, Some(0.30), 2250, 1950),
        321 => (None, Some(0.25), 0, 0),
        322 => (None, Some(0.45), 2400, 1950),
        323 => (None, Some(0.35), 1900, 1500),
        324 => (None, Some(0.25), 1600, 1300),
        325 => (None, Some(0.49), 2800, 2400),
        326 => (None, Some(0.33), 2000, 1740),
        327 => (None, Some(0.22), 1450, 1165),
        337 => (Some(1.01), Some(0.35), 2500, 2200),
        338 => (Some(0.98), Some(0.24), 1900, 1650),
        339 => (Some(0.94), Some(0.20), 1600, 1350),
        255 => (Some(1.15), Some(0.25), 1800, 1450),
        256 => (Some(1.21), Some(0.32), 1950, 1550),
        257 => (Some(1.34), Some(0.40), 2100, 1550),
        // W4-g70: the forest tree-set ids (`SetForestBGSet` styles 9..13).
        279 => (Some(2.5), Some(0.35), 1850, 1750),
        282 => (Some(2.6), Some(0.25), 1800, 1340),
        328 => (Some(2.01), Some(0.34), 0, 0),
        329 => (Some(2.29), Some(0.26), 0, 0),
        330 => (None, Some(0.25), 1750, 1450),
        333 => (Some(2.01), Some(0.37), 0, 0),
        334 => (None, Some(0.26), 0, 0),
        335 => (None, Some(0.22), 1650, 1470),
        341 => (None, Some(0.30), 0, 0),
        342 => (None, Some(0.25), 0, 0),
        343 => (None, Some(0.20), 1650, 1550),
        _ => return None,
    })
}

/// `Main.DrawSurfaceBG_GetLayerYOffset` (`Main.cs:59740-59797`), the complete
/// switch: the jungle 59 nudge, the forest-set ids (W4-g70), the snow/ocean
/// mountain ids (W4-g71) and the hallow 246/247 pair.
pub(crate) fn layer_y_offset(id: i32) -> f32 {
    match id {
        59 => -550.0,
        93 | 168 | 169 | 170 => -50.0,
        171 => -100.0,
        172 => 130.0,
        176 => -760.0,
        177 => -200.0,
        179 => -100.0,
        180 | 181 | 182 | 183 => -350.0,
        246 | 247 => -150.0,
        263 => -700.0,
        269 => -600.0,
        270 => -50.0,
        271 => -300.0,
        272 => -380.0,
        277 => -260.0,
        278 => -120.0,
        280 => -170.0,
        281 => -300.0,
        283 => -800.0,
        332 | 340 => -25.0,
        _ => 0.0,
    }
}

/// `Main.UpdateOceanWaterLineForAmbience` (`Main.cs:59965-59984`): the ocean
/// sheet's water-line offset below its `GetLayerYOffset`-adjusted top, in
/// unscaled sheet pixels (`num2 * Main.bgScale` in the C#). The C# writes
/// `yScreenPosition = bgTopY + yOffset + num2 * bgScale` and the far-layer-3
/// alpha into `AmbientSkyDrawCache.OceanLineInfo`; the port's ambience pass
/// keeps the ocean-line term at constant 1 (the documented Q4 deviation), so
/// only the geometry helper is ported here.
pub(crate) fn ocean_water_line_offset(ocean_bg: i32) -> f32 {
    match ocean_bg {
        28 | 110 | 111 | 209 | 210 => 102.0,
        283 => 124.0,
        // 332/340 fall through the C# switch with num2 = 0.
        _ => 0.0,
    }
}

/// `Main.GetBackgroundRect`'s `GetBackgroundCounter()` divisor
/// (`Main.cs:61220-61262`): the animated 2x2 `Background_*` grids advance a
/// frame every N ticks. `0` = the sheet is not animated (the C# returns `null`
/// and the whole texture is drawn).
pub(crate) fn background_frame_step(id: i32) -> u32 {
    match id {
        235 => 20,
        219 | 220 | 221 | 271 | 272 | 273 | 302 | 303 | 305 | 307 | 309 | 311 | 313 | 315 | 317
        | 326 | 337 | 338 | 341 => 15,
        281 => 5,
        _ => 0,
    }
}

/// One frame of a `GetBackgroundRect` 2x2 grid, or the whole sheet.
///
/// `Frame(2, 2, num % 2, num / 2)` then `Inflate(0, -2)` — XNA's inflate moves
/// the top down 2 px and shortens the height by 4, in source-texture pixels.
/// `Main.backgroundWidth/Height` are halved for the animated ids at load
/// (`Main.cs:59046-59072`), which is why the frame is half the sheet.
pub(crate) fn background_frame(tex: &BgTex, step: u32, counter: u32) -> BgTex {
    if step == 0 {
        return *tex;
    }
    let (du, dv) = (tex.uv[2] - tex.uv[0], tex.uv[3] - tex.uv[1]);
    let num = (counter / step) % 4;
    let (col, row) = ((num % 2) as f32, (num / 2) as f32);
    let px = dv / tex.h; // one source pixel, in `v` units
    let u0 = tex.uv[0] + du * 0.5 * col;
    let v0 = tex.uv[1] + dv * 0.5 * row + px * 2.0;
    BgTex {
        uv: [
            u0,
            v0,
            u0 + du * 0.5,
            tex.uv[1] + dv * 0.5 * (row + 1.0) - px * 2.0,
        ],
        w: tex.w * 0.5,
        h: tex.h * 0.5 - 4.0,
    }
}

/// The three foreground layers of one biome (`DrawSurfaceBG_*`).
///
/// Every row is tinted by `ColorOfSurfaceBackgroundsBase * alpha`, where
/// `alpha` is the biome's `Main.bgAlphaFrontLayer` entry. `counter` is
/// `Main.GameUpdateCount` (`GetBackgroundCounter`, `Main.cs:61264-61271`).
#[allow(clippy::too_many_arguments)]
pub(crate) fn draw_front_trees(
    r: &mut SpriteRenderer,
    camera: Vec2,
    zoom: f32,
    screen: Vec2,
    layers: &[FrontLayer; 3],
    alpha: f32,
    magic: f32,
    sc_adj: f32,
    tint: [f32; 3],
    counter: u32,
    mut ambience: Option<&mut AmbiencePass<'_>>,
    // Only the Mushroom body ends with `DrawToDepth(1f)` (`Main.cs:60186`);
    // the others (incl Snow, `:60302-60382`) leave the stratum to the flush
    // (`Main.cs:59626`).
    closes_with_one: bool,
) {
    if alpha <= 0.0 {
        return;
    }
    for l in layers {
        if let Some(p) = ambience.as_deref_mut() {
            draw_sandwich_band(r, p, l.depth);
            // W4-c39: the party balloons' band rides the same boundary.
            draw_party_band_min(r, p, l.depth);
            crate::ambience::draw_range_min(r, p.entities, p.lookups, p.ctx, l.depth, p.last);
            // W4-f40: the lantern sky's own band at the same boundary.
            if *p.lantern_last > l.depth {
                p.lantern_sky.draw(
                    r,
                    p.lookups,
                    p.ctx.camera,
                    p.ctx.zoom,
                    p.ctx.screen,
                    p.ctx.world_surface,
                    p.global_time,
                    l.depth,
                    *p.lantern_last,
                );
                *p.lantern_last = l.depth;
            }
        }
        if let Some(t) = l.tex {
            let tex = background_frame(&t, l.frame_step, counter);
            draw_tiled(
                r,
                camera,
                zoom,
                screen,
                &tex,
                l.scale,
                l.parallax,
                magic * l.k + l.top + sc_adj,
                l.flip,
                l.tint.unwrap_or(tint),
                alpha,
                l.x_shift,
                l.extra,
            );
        }
    }
    // The Mushroom chain closes with `DrawToDepth(1f)`; the other biomes'
    // strata wait for the flush.
    if closes_with_one && let Some(p) = ambience.as_deref_mut() {
        draw_sandwich_band(r, p, 1.0);
        // W4-c39: the party balloons' band rides the closing boundary too.
        draw_party_band_min(r, p, 1.0);
        crate::ambience::draw_range_min(r, p.entities, p.lookups, p.ctx, 1.0, p.last);
        if *p.lantern_last > 1.0 {
            p.lantern_sky.draw(
                r,
                p.lookups,
                p.ctx.camera,
                p.ctx.zoom,
                p.ctx.screen,
                p.ctx.world_surface,
                p.global_time,
                1.0,
                *p.lantern_last,
            );
            *p.lantern_last = 1.0;
        }
    }
}

/// W4-c39 exec (research/114 §1e): one `DrawToDepth(min)` boundary for the
/// `PartySky` balloon field — the `[min, party_last)` band, then the tracker
/// advances (the `draw_range_min` shape, the f40 `lantern_sky` precedent).
fn draw_party_band_min(r: &mut SpriteRenderer, p: &mut AmbiencePass<'_>, min_depth: f32) {
    if *p.party_last <= min_depth {
        return;
    }
    crate::event_skies::draw_balloons_band(
        r,
        p.event_skies,
        p.lookups,
        p.ctx.camera,
        p.ctx.zoom,
        p.ctx.screen,
        p.ctx.color_of_the_skies,
        p.party_surface_px,
        min_depth,
        *p.party_last,
    );
    *p.party_last = min_depth;
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
    draw_tiled(
        r, camera, zoom, screen, tex, scale, parallax, top, flip, tint, alpha, 0.0, 0,
    );
}

/// The tiling loop of `DrawSurfaceBG_DrawBackMountainsLayer` /
/// `DrawSurfaceBG_*`: `bgLoops = screenWidth / bgWidthScaled + 2` copies from
/// `bgStartX`.
#[allow(clippy::too_many_arguments)]
fn draw_tiled(
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
    x_shift: f32,
    extra: u8,
) {
    // Terraria renders backgrounds through `BackgroundViewMatrix`, whose zoom is
    // `ForcedMinimumZoom` (1): the layers keep their screen size at any game zoom,
    // and `bgTopY` is already a screen-space top, so no further transform applies.
    let sw = tex.w * scale;
    let sh = tex.h * scale;
    if sw <= 0.0 {
        return;
    }
    // `Main.bgWidthScaled` is `(int)(width * bgScale)` — an int — while the sprite
    // itself is drawn at the float `bgScale`, so the stride, the phase and the
    // half-stride offset are all truncated but the quad is not.
    let stride = sw as i32 as f32;
    let uv = if flip {
        [tex.uv[2], tex.uv[1], tex.uv[0], tex.uv[3]]
    } else {
        tex.uv
    };
    // `bgStartX = (int)(-IEEERemainder(screenPosition.X * bgParallax, bgWidthScaled)
    // - bgWidthScaled / 2)`: phase against the screen's left edge, not the camera
    // centre, and half a tile of offset so the tiling is centred on the view.
    // The `(int)` truncates toward zero over the whole sum (C#), and the halving
    // is integer division on `bgWidthScaled`.
    let screen_left = camera.x - screen.x * 0.5;
    let half = (stride as i32 / 2) as f32;
    let start = (-ieee_remainder(screen_left * parallax, stride) - half) as i32 as f32 + x_shift;
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
        x += stride;
    }
    for _ in 0..extra {
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
        x += stride;
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
        // `WallDrawing.DrawWalls` (`WallDrawing.cs:91`) skips wall 318
        // (`WallID.EchoWall`) and Echo-coated walls unless
        // `ShouldShowInvisibleBlocksAndWalls` grants sight (W4-g78: the
        // Echo Monolith / Spectre Goggles) — hidden, neither draws nor
        // paints black; shown, both draw.
        assert!(!wall_visible(0, false, false));
        assert!(!wall_visible(WALL_ECHO, false, false));
        assert!(wall_visible(1, false, false));
        assert!(wall_visible(350, false, false));
        // The Echo-coating bit hides ANY wall without sight.
        assert!(!wall_visible(1, true, false));
        // Sight lifts both the 318 kind and the coating bit.
        assert!(wall_visible(WALL_ECHO, false, true));
        assert!(wall_visible(1, true, true));
        // Wall 0 never draws, sight or not.
        assert!(!wall_visible(0, false, true));
    }

    #[test]
    fn wall_light_at_reads_the_walllight_table() {
        // `Main.WallLightAt` (`Main.cs:55169-55181`): `wallLight[wall]` decides.
        // Wall 21 transmits (`Main.cs:10579`), wall 1 does not; an empty cell
        // (wall 0) transmits too (`Main.cs:10578`, `wallLight[0] = true`).
        let mut w = world();
        w.tile_mut(10, 10).unwrap().wall = 21;
        w.tile_mut(11, 10).unwrap().wall = 1;
        assert!(wall_light_at(&w, 10, 10, false));
        assert!(!wall_light_at(&w, 11, 10, false));
        assert!(wall_light_at(&w, 12, 10, false)); // wall 0 -> wallLight[0]
        // Out-of-world cells have no tile -> false (the C# would null-fill a
        // `new Tile()` whose wall 0 transmits, but the bg passes gate on
        // `InWorld(.., 1)` first, so the port never asks outside).
        assert!(!wall_light_at(&w, -1, 10, false));
    }

    #[test]
    fn wall_light_at_keeps_the_echo_coating_arm() {
        // `Main.cs:55172-55179`: a NON-light wall still counts while it carries the
        // Echo coating (`tile.invisibleWall()`) and the invisible walls are hidden.
        // The coating IS set in the original (`WorldGen.cs:966/970/976`,
        // `IO/WorldFile.cs:2672`) — the old "never set" shortcut dropped this arm.
        let mut w = world();
        w.tile_mut(10, 10).unwrap().wall = 1; // wallLight[1] == false
        assert!(!wall_light_at(&w, 10, 10, false));
        w.set_invisible_wall(10, 10, true);
        // Coated + hidden -> the backdrop still lights through it (`:55176`).
        assert!(wall_light_at(&w, 10, 10, false));
        // Coated + SHOWING invisible walls -> the escape closes (`:55178`).
        assert!(!wall_light_at(&w, 10, 10, true));
        // A light wall stays lit regardless of the coating / the show flag.
        w.tile_mut(11, 10).unwrap().wall = 21;
        w.set_invisible_wall(11, 10, true);
        assert!(wall_light_at(&w, 11, 10, false));
        assert!(wall_light_at(&w, 11, 10, true));
    }

    #[test]
    fn should_draw_background_tile_at_spreads_wall_light_under_parallax() {
        // `Main.ShouldDrawBackgroundTileAt` (`Main.cs:55136-55162`): non-zero
        // parallax counts the horizontal neighbours' wall light (`:55152`); zero
        // parallax only the cell itself (`:55157`).
        let mut w = world();
        // A light wall at x=11, the probe cell x=10 has none.
        w.tile_mut(11, 10).unwrap().wall = 21;
        // Parallax on: the right neighbour keeps the cell drawn.
        assert!(should_draw_background_tile_at(&w, 10, 10, 0.88, false));
        // Parallax off: only the cell's own (absent) wall light counts.
        assert!(!should_draw_background_tile_at(&w, 10, 10, 0.0, false));
        // The left neighbour counts too (`:55152`'s `i - 1` arm).
        let mut w2 = world();
        w2.tile_mut(9, 10).unwrap().wall = 21;
        assert!(should_draw_background_tile_at(&w2, 10, 10, 0.88, false));
        // The Echo coating on a neighbour spreads the same way under parallax.
        let mut w3 = world();
        w3.tile_mut(11, 10).unwrap().wall = 1;
        w3.set_invisible_wall(11, 10, true);
        assert!(should_draw_background_tile_at(&w3, 10, 10, 0.88, false));
        assert!(!should_draw_background_tile_at(&w3, 10, 10, 0.88, true));
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
        // W4-g72 final: the table now routes through the full 22-case
        // `background_style_sheets`; style 0 + a fresh World reproduces the
        // classic set for every hell roll.
        let mut w = world();
        w.hell_back_style = 0;
        assert_eq!(
            draw_background_update_background_styles(0, &w),
            [1, 2, 4, 3, 6, 125, 185]
        );
        w.hell_back_style = 1;
        assert_eq!(
            draw_background_update_background_styles(0, &w),
            [1, 2, 4, 3, 6, 126, 186]
        );
        w.hell_back_style = 2;
        assert_eq!(
            draw_background_update_background_styles(0, &w),
            [1, 2, 4, 3, 6, 127, 187]
        );
    }

    /// W4-g72 final: the LIVE roll→persist→draw chain. A world whose cave
    /// arrays were rolled (or loaded from the `.wld`/`.trwld`/net header —
    /// the same fields) picks a NON-zero style through the same
    /// `BgStylePickInput` the frame builds, and the sheet table follows the
    /// pick + the ice/jungle/hell variant bytes. The per-biome sheets are
    /// atlas-blocked, so `back_texture` returns None for them (graceful
    /// degradation, never a wrong sheet) — the ids themselves are pinned here.
    #[test]
    fn w4_g72_the_draw_side_reads_the_rolled_cave_arrays() {
        let mut w = world();
        // Arm-A shape: one centre band, styles [5, 2], trailing edges at width.
        w.cave_back_x = [2100, 4200, 4200, 0];
        w.cave_back_style = [5, 2, 0, 0];
        w.ice_back_style = 2;
        w.jungle_back_style = 1;
        w.hell_back_style = 1;
        w.world_id = 7;
        let metrics = tr_core::scene::SceneMetrics::new();
        // Probe column left of the band edge → style[0] = 5, +3 = 8.
        let input = super::super::bg_styles::bg_style_pick_input(
            &w,
            &metrics,
            [100.0 * TILE_PX, 700.0 * TILE_PX],
            [1600.0, 900.0],
            magma_layer_of(&w),
        );
        let picked = super::super::bg_styles::pick_underground_background_style(&input);
        assert_eq!(picked, 8);
        // Style 8's table (Main.cs:53895-53901): 83/84/85/86 + 137+hell.
        assert_eq!(
            draw_background_update_background_styles(picked, &w),
            [83, 84, 85, 86, 138, 126, 186]
        );
        // Right of the band edge → style[1] = 2 (glowshroom), +3 = 5.
        let input = super::super::bg_styles::bg_style_pick_input(
            &w,
            &metrics,
            [3000.0 * TILE_PX, 700.0 * TILE_PX],
            [1600.0, 900.0],
            magma_layer_of(&w),
        );
        let picked = super::super::bg_styles::pick_underground_background_style(&input);
        assert_eq!(picked, 5);
        assert_eq!(
            draw_background_update_background_styles(picked, &w),
            [73, 74, 75, 76, 132, 126, 186]
        );
        // The glowshroom trump reads the metrics scan, not a World field.
        let mut m = tr_core::scene::SceneMetrics::new();
        m.mushroom_tile_count = tr_core::scene::thresholds::MUSHROOM_TILE;
        let input = super::super::bg_styles::bg_style_pick_input(
            &w,
            &m,
            [100.0 * TILE_PX, 700.0 * TILE_PX],
            [1600.0, 900.0],
            magma_layer_of(&w),
        );
        assert_eq!(
            super::super::bg_styles::pick_underground_background_style(&input),
            2
        );
        // Style 2 (Main.cs:53853-53859): 62/63/64/65 + 143+hell.
        assert_eq!(
            draw_background_update_background_styles(2, &w),
            [62, 63, 64, 65, 144, 126, 186]
        );
    }

    /// W4-g72 final: the latch/decay rides the draw — a style change arms the
    /// fade at 1 then decays 0.25 per draw (`Main.cs:52823-52841`), and full
    /// shimmer skips the pass outright (`:52800-52805`).
    #[test]
    fn w4_g72_the_ug_back_latch_steps_with_the_draw() {
        let mut s = super::super::bg_styles::UgBackState {
            style: 0,
            old_style: 0,
            transition: 0.0,
        };
        super::super::bg_styles::ug_back_step(&mut s, 8, 0.0);
        assert_eq!((s.style, s.old_style), (8, 0));
        assert_eq!(s.transition, 0.75); // armed 1, decayed once in the same step
        super::super::bg_styles::ug_back_step(&mut s, 8, 0.0);
        assert_eq!(s.transition, 0.5);
        super::super::bg_styles::ug_back_step(&mut s, 8, 0.5);
        assert_eq!(s.transition, 0.0); // any shimmer kills the fade
        super::super::bg_styles::ug_back_step(&mut s, 5, 0.0);
        assert_eq!((s.style, s.old_style), (5, 8));
        assert_eq!(s.transition, 0.75);
    }

    /// W4-g72 closeout: the cross-fade overlay wiring — the OLD-style table
    /// resolves through the same `back_texture` gate as the new one, the
    /// fade carries the OLD sheet's row pitch, and an unresolved (atlas-
    /// blocked) old slot degrades to a skipped overlay with the NEW pass's
    /// pitch (`UgOld::pitch`'s fallback).
    #[test]
    fn w4_g72_the_crossfade_old_table_resolves_and_degrades() {
        let w = world();
        // Style 0 (classic) vs style 8 (per-biome): the old table's ids are
        // the second switch run's output (`Main.cs:53804-53810`).
        let old_back = draw_background_update_background_styles(0, &w);
        assert_eq!(old_back, [1, 2, 4, 3, 6, 125, 185]);
        let new_back = draw_background_update_background_styles(8, &w);
        assert_eq!(new_back, [83, 84, 85, 86, 137, 125, 185]);
        // The classic old slots all resolve through `back_texture`.
        for id in old_back {
            assert!(back_texture(&sky_stub(), id).is_some());
        }
        for id in [83usize, 84, 85, 86] {
            assert!(back_texture(&sky_stub(), id).is_none());
        }
        // The fade's pitch: a resolved old sheet keeps its OWN height, an
        // unresolved one falls back to the new pass's pitch.
        let dirt = BgTex { uv: [0.0, 0.0, 0.1, 0.1], w: 128.0, h: 96.0 };
        let old_dirt = BgTex { uv: [0.0, 0.0, 0.1, 0.1], w: 128.0, h: 120.0 };
        let resolved = UgOld {
            sheets: [None, Some(&old_dirt), None, None, None, None, None],
            h: [0.0, 120.0, 0.0, 0.0, 0.0, 0.0, 0.0],
            transition: 0.75,
        };
        assert_eq!(resolved.pitch(1, dirt.h), 120.0);
        assert_eq!(resolved.fade(1, dirt.h).old_pitch, 120.0);
        assert_eq!(resolved.fade(1, dirt.h).transition, 0.75);
        // Unresolved slot 3: the overlay skips (sheet None), pitch = new.
        assert_eq!(resolved.pitch(3, dirt.h), 96.0);
        assert!(resolved.fade(3, dirt.h).sheet.is_none());
        // transition 0 disables every overlay regardless of the sheet.
        let dead = UgOld { transition: 0.0, ..resolved };
        assert_eq!(dead.fade(1, dirt.h).transition, 0.0);
    }

    /// A `SkyAssets` stub carrying only the classic underground sheets — the
    /// overlay-resolution witness needs no GPU, just the `back_texture` gate.
    fn sky_stub() -> SkyAssets {
        let mut sky = SkyAssets::default();
        let tex = BgTex { uv: [0.0, 0.0, 0.1, 0.1], w: 128.0, h: 96.0 };
        sky.underground_transition = Some(tex);
        sky.underground = Some(tex);
        sky.rock = Some(tex);
        sky.rock_band = Some(tex);
        sky.magma_transition = Some(tex);
        sky.magma = [Some(tex), Some(tex), Some(tex)];
        sky.hell_bottom = [Some(tex), Some(tex), Some(tex)];
        sky
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
        // `WorldGen.cs:8247`: one `Next(3)` draw. W4-g71 FINAL keep-one: it is
        // folded into `cave_bg::randomize_cave_backgrounds` on the SHARED
        // `streams.rng` (g72's landed seat, ahead of the moon draw — the former
        // standalone tail draw AND the g71 pass_rng duplicate are both gone).
        // The value moves by design; the per-seed determinism + the `<3` bound hold.
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
        // The wired cave roll also lands the dividers/styles + ice/jungle.
        assert_eq!(a.cave_back_x, b.cave_back_x);
        assert_eq!(a.cave_back_style, b.cave_back_style);
        assert_eq!(a.ice_back_style, b.ice_back_style);
        assert!(a.ice_back_style < 4);
        assert!(a.jungle_back_style < 2);
        // The four cave styles are pairwise distinct (the rejection loops).
        for i in 0..4 {
            for j in (i + 1)..4 {
                assert_ne!(a.cave_back_style[i], a.cave_back_style[j]);
            }
        }
    }
}

#[cfg(test)]
mod surface_variant_bg_tests {
    use super::*;

    fn sheet() -> BgTex {
        // 100x200 px at the atlas origin, so `du == tex.w` and `dv == tex.h`.
        BgTex {
            uv: [0.0, 0.0, 0.5, 0.5],
            w: 100.0,
            h: 200.0,
        }
    }

    #[test]
    fn background_offsets_match_set_background_offsets() {
        // Spot rows of `Main.SetBackgroundOffsets` (`Main.cs:60384-60775`):
        // corrupt style 4, jungle style 5, mushroom style 2, crimson style 6.
        assert_eq!(bg_offsets(240), Some((Some(1.15), Some(0.30), 1800, 1500)));
        assert_eq!(bg_offsets(242), Some((Some(1.34), Some(0.49), 2100, 1400)));
        assert_eq!(bg_offsets(273), Some((None, Some(0.49), 2100, 1560)));
        assert_eq!(bg_offsets(235), Some((None, Some(0.33), 1950, 1550)));
        assert_eq!(bg_offsets(326), Some((None, Some(0.33), 2000, 1740)));
        assert_eq!(bg_offsets(337), Some((Some(1.01), Some(0.35), 2500, 2200)));
        // `261`/`262`/`219`/`220`/`221`/`321` set only the parallax — the caller's
        // `bgTopY` magic numbers survive.
        assert_eq!(bg_offsets(261), Some((None, Some(0.27), 0, 0)));
        // `222`/`223`/`224`/`237`/`238`/`239` set only `num`/`num2` (the caller's
        // `bgScale`/`bgParallax` survive).
        assert_eq!(bg_offsets(222), Some((None, None, 1800, 1400)));
        assert_eq!(bg_offsets(239), Some((None, None, 2100, 1900)));
        // Ids the five style tables reach but the C# table does not list.
        for id in [
            12, 13, 14, 15, 16, 17, 29, 30, 31, 43, 44, 45, 46, 47, 48, 56, 57, 58, 59, 60, 61,
            102, 103, 104, 105, 106, 107, 174, 175, 211, 212, 213, 214, 215, 216, 225, 226, 227,
            229, 230, 231, 232, 233, 271, 272,
        ] {
            assert_eq!(bg_offsets(id), None, "id {id} must keep the caller's row");
        }
    }

    #[test]
    fn layer_y_offsets_match_get_layer_y_offset() {
        // The whole switch (`Main.cs:59744-59795`), spot rows per arm.
        assert_eq!(layer_y_offset(59), -550.0);
        for id in [93, 168, 169, 170] {
            assert_eq!(layer_y_offset(id), -50.0, "id {id}");
        }
        assert_eq!(layer_y_offset(171), -100.0);
        assert_eq!(layer_y_offset(172), 130.0);
        assert_eq!(layer_y_offset(176), -760.0);
        assert_eq!(layer_y_offset(177), -200.0);
        assert_eq!(layer_y_offset(179), -100.0);
        for id in [180, 181, 182, 183] {
            assert_eq!(layer_y_offset(id), -350.0, "id {id}");
        }
        assert_eq!(layer_y_offset(246), -150.0);
        assert_eq!(layer_y_offset(247), -150.0);
        assert_eq!(layer_y_offset(263), -700.0);
        assert_eq!(layer_y_offset(269), -600.0);
        assert_eq!(layer_y_offset(270), -50.0);
        assert_eq!(layer_y_offset(271), -300.0);
        assert_eq!(layer_y_offset(272), -380.0);
        assert_eq!(layer_y_offset(277), -260.0);
        assert_eq!(layer_y_offset(278), -120.0);
        assert_eq!(layer_y_offset(280), -170.0);
        assert_eq!(layer_y_offset(281), -300.0);
        assert_eq!(layer_y_offset(283), -800.0);
        assert_eq!(layer_y_offset(332), -25.0);
        assert_eq!(layer_y_offset(340), -25.0);
        // The `default:` arm.
        assert_eq!(layer_y_offset(273), 0.0);
        assert_eq!(layer_y_offset(60), 0.0);
        assert_eq!(layer_y_offset(28), 0.0);
    }

    #[test]
    fn w4_g71_snow_desert_ocean_offsets_match_set_background_offsets() {
        // Snow styles 5/6/7/8 (`Main.cs:60390-60443`, `:60620-60630`).
        assert_eq!(bg_offsets(258), Some((Some(1.25), Some(0.23), 1800, 850)));
        assert_eq!(bg_offsets(259), Some((Some(1.31), Some(0.33), 1950, 1500)));
        assert_eq!(bg_offsets(260), Some((Some(1.34), Some(0.40), 2100, 1650)));
        assert_eq!(bg_offsets(263), Some((Some(1.25), Some(0.23), 1800, 1450)));
        assert_eq!(bg_offsets(264), Some((Some(1.31), Some(0.33), 1950, 1700)));
        assert_eq!(bg_offsets(265), Some((Some(1.34), Some(0.40), 2100, 2000)));
        assert_eq!(bg_offsets(267), Some((Some(1.25), Some(0.23), 1700, 1300)));
        assert_eq!(bg_offsets(266), Some((Some(1.31), Some(0.33), 1950, 1600)));
        assert_eq!(bg_offsets(268), Some((Some(1.34), Some(0.41), 2100, 1850)));
        // Style 8's rows: 298 keeps the scale, 299 keeps the caller's scale.
        assert_eq!(bg_offsets(298), Some((Some(1.25), Some(0.35), 2000, 1700)));
        assert_eq!(bg_offsets(299), Some((None, Some(0.28), 1600, 1400)));
        // Desert style 4 (`:60444-60461`) + style 2's parallax-only 207.
        assert_eq!(bg_offsets(248), Some((Some(1.30), Some(0.37), 1800, 1100)));
        assert_eq!(bg_offsets(249), Some((Some(1.40), Some(0.43), 1950, 1200)));
        assert_eq!(bg_offsets(250), Some((Some(1.70), Some(0.49), 2000, 1000)));
        assert_eq!(bg_offsets(207), Some((None, Some(0.32), 0, 0)));
        // Desert styles 51/52/53: the shared middle-row arm and far-row arm
        // (`:60649-60672`); the 2x2-animated middles keep their frame steps.
        for id in [303, 304, 305, 307, 308, 309, 311, 312, 313, 315, 316, 317] {
            assert_eq!(bg_offsets(id), Some((None, Some(0.35), 2400, 1900)), "id {id}");
        }
        for id in [306, 310, 314, 318] {
            assert_eq!(bg_offsets(id), Some((None, Some(0.26), 1800, 1700)), "id {id}");
        }
        // Classic snow/desert/ocean ids are NOT in the C# table.
        for id in [37, 38, 39, 95, 96, 97, 21, 20, 108, 109, 217, 218, 28, 110, 111, 209, 210, 283, 332, 340] {
            assert_eq!(bg_offsets(id), None, "id {id} keeps the caller's row");
        }
        // The desert-51 middle rows animate (`GetBackgroundRect` 15-tick set).
        for id in [303, 305, 307, 309, 311, 313, 315, 317] {
            assert_eq!(background_frame_step(id), 15, "id {id}");
        }
        // 304/306/etc are static sheets.
        for id in [304, 306, 308, 310, 312, 314, 316, 318] {
            assert_eq!(background_frame_step(id), 0, "id {id}");
        }
    }

    #[test]
    fn w4_g71_ocean_water_line_matches_update_ocean_water_line_for_ambience() {
        // `Main.cs:59969-59981`: the five classic sheets sit 102 px below the
        // layer top, the shimmer ocean 283 at 124; 332/340 keep num2 = 0.
        for id in [28, 110, 111, 209, 210] {
            assert_eq!(ocean_water_line_offset(id), 102.0, "ocean {id}");
        }
        assert_eq!(ocean_water_line_offset(283), 124.0);
        assert_eq!(ocean_water_line_offset(332), 0.0);
        assert_eq!(ocean_water_line_offset(340), 0.0);
        // The full line: bgTopY + GetLayerYOffset + num2 * bgScale. With the
        // Step2 state (scale 2.3, top magic*1400+1260+scAdj+30) at magic 0,
        // scAdj 0: ocean 28 -> 1290 + 0 + 102*2.3 = 1524.6; ocean 283 ->
        // 1290 - 800 + 124*2.3 = 775.2.
        let top = 1260.0 + 30.0;
        let scale = 2.3;
        let line28 = top + layer_y_offset(28) + ocean_water_line_offset(28) * scale;
        assert!((line28 - 1524.6).abs() < 1e-3, "line28 {line28}");
        let line283 = top + layer_y_offset(283) + ocean_water_line_offset(283) * scale;
        assert!((line283 - 775.2).abs() < 1e-3, "line283 {line283}");
    }

    /// W4-g72 closeout: the SURFACE-OWNERSHIP verification — the five
    /// v16-owned variants read the g71-shape `setBG` style→ids expansion
    /// (`surface_bg_texture_ids`, slots 1/2/4/5/8), while the forest/desert/
    /// snow/ocean style tables are g70/g71's (this lane pins their style-0
    /// rows and does NOT re-port the tables). Pins the five variants' default
    /// (style 0) rows and one non-default row each, plus the slot-3/6/7/9
    /// `None` verdicts that mark the g71-owned tables as unported HERE.
    #[test]
    fn w4_g72_the_surface_variants_read_the_g71_shape_set_bg_tables() {
        use tr_core::world::surface_bg_texture_ids;
        // The five v16-owned variants' style-0 (default) rows — the exact ids
        // the sky.rs front-loop chains draw (`world.*_bg()` at style 0).
        assert_eq!(surface_bg_texture_ids(1, 0), Some([12, 13, 14])); // corrupt
        assert_eq!(surface_bg_texture_ids(2, 0), Some([15, 16, 17])); // jungle
        assert_eq!(surface_bg_texture_ids(4, 0), Some([29, 30, 31])); // hallow
        assert_eq!(surface_bg_texture_ids(5, 0), Some([43, 44, 45])); // crimson
        assert_eq!(surface_bg_texture_ids(8, 0), Some([46, 47, 48])); // mushroom
        // One non-default row each — the style→ids expansion is LIVE, not a
        // style-0 pin (the g71-shape the surface chains read).
        assert_eq!(surface_bg_texture_ids(1, 1), Some([56, 57, 58]));
        assert_eq!(surface_bg_texture_ids(2, 1), Some([59, 60, 61]));
        assert_eq!(surface_bg_texture_ids(4, 1), Some([102, 103, 104]));
        assert_eq!(surface_bg_texture_ids(5, 1), Some([105, 106, 107]));
        assert_eq!(surface_bg_texture_ids(8, 1), Some([231, 232, 233]));
        // The g71-owned slots (snow 3, desert 6, ocean 7, underworld 9) and the
        // g70 forest slots (0/10/11/12) are NOT in this lane's table — their
        // style tables are g70/g71's to port (the sky.rs chains pin style 0).
        // Merger re-pin (batch-M): g71's snow table (slot 3) landed in the
        // SAME `surface_bg_texture_ids` match — its style-0 row is now Some
        // ([37,38,39], the g71 witness pins the full 16-arm table). The
        // forest/desert/ocean/underworld slots keep their own tables
        // (`forest_bg_set`/`desert_bg_set`/`ocean_bg_texture_id`/
        // `underworld_bg_set`) and stay None here.
        assert_eq!(surface_bg_texture_ids(3, 0), Some([37, 38, 39]));
        for slot in [0, 6, 7, 9, 10, 11, 12] {
            assert_eq!(surface_bg_texture_ids(slot, 0), None, "slot {slot} has its own table");
        }
    }

    #[test]
    fn background_frame_steps_match_get_background_rect() {
        // `Main.cs:61226-61258`.
        assert_eq!(background_frame_step(235), 20);
        assert_eq!(background_frame_step(219), 15);
        assert_eq!(background_frame_step(271), 15);
        assert_eq!(background_frame_step(281), 5);
        assert_eq!(background_frame_step(240), 0);
        assert_eq!(background_frame_step(105), 0);
    }

    #[test]
    fn background_frame_halves_the_sheet_and_insets_two_pixels() {
        let tex = sheet();
        // `step == 0`: the whole sheet (`GetBackgroundRect` returns `null`).
        assert_eq!(background_frame(&tex, 0, 7).w, 100.0);

        // `Frame(2, 2, num % 2, num / 2)` then `Inflate(0, -2)`:
        // half the sheet, 4 px shorter, top down 2 px.
        let f = background_frame(&tex, 15, 0); // num = 0 -> col 0, row 0
        assert_eq!((f.w, f.h), (50.0, 96.0));
        assert_eq!(f.uv[0], 0.0);
        assert_eq!(f.uv[2], 0.25);
        // 2 px of 200 -> 0.005 in `v`; the bottom keeps the row's end minus 2 px.
        assert!((f.uv[1] - 0.005).abs() < 1e-6, "v0 = {}", f.uv[1]);
        assert!((f.uv[3] - 0.245).abs() < 1e-6, "v1 = {}", f.uv[3]);

        // num = 1 -> col 1, row 0; num = 2 -> col 0, row 1.
        let f = background_frame(&tex, 15, 15);
        assert!((f.uv[0] - 0.25).abs() < 1e-6, "u0 = {}", f.uv[0]);
        assert!((f.uv[1] - 0.005).abs() < 1e-6, "v0 = {}", f.uv[1]);
        let f = background_frame(&tex, 15, 30);
        assert_eq!(f.uv[0], 0.0);
        assert!((f.uv[1] - 0.255).abs() < 1e-6, "v0 = {}", f.uv[1]);
        // num wraps every four frames.
        assert_eq!(
            background_frame(&tex, 15, 60).uv[1],
            background_frame(&tex, 15, 0).uv[1]
        );
    }

    #[test]
    fn front_layers_apply_set_background_offsets_and_double_the_scale() {
        // Corrupt style 4 = `[240, 241, 242]`: all three rows are in the table,
        // so both the scale and the `bgTopY` magic numbers are replaced.
        let l = front_layers(
            [240, 241, 242],
            [
                (1.25, 0.40, 1800.0, 1500.0),
                (1.31, 0.43, 1950.0, 1750.0),
                (1.34, 0.49, 2100.0, 2000.0),
            ],
            [Some(sheet()), None, None],
            -150.0,
        );
        assert_eq!(l[0].scale, 1.15 * 2.0);
        assert_eq!(l[0].parallax, 0.30);
        assert_eq!((l[0].k, l[0].top), (1800.0, 1500.0 - 150.0));
        assert_eq!(l[2].scale, 1.34 * 2.0);
        assert_eq!((l[2].k, l[2].top), (2100.0, 1400.0 - 150.0));
        assert!(l[1].tex.is_none());

        // Jungle style 1 = `[59, 60, 61]`: no offsets, so the caller's rows win.
        let l = front_layers(
            [59, 60, 61],
            [
                (1.25, 0.40, 1800.0, 1660.0),
                (1.31, 0.43, 1950.0, 1840.0),
                (1.34, 0.49, 2100.0, 2060.0),
            ],
            [None, None, None],
            -150.0,
        );
        assert_eq!(l[0].scale, 2.5);
        assert_eq!(l[0].parallax, 0.40);
        assert_eq!((l[0].k, l[0].top), (1800.0, 1510.0));

        // Mushroom style 2 keeps row 1's 20-tick frame animation.
        let l = front_layers([234, 235, 236], [(1.25, 0.4, 0.0, 0.0); 3], [None; 3], 0.0);
        assert_eq!(
            (l[1].parallax, l[1].top, l[1].frame_step),
            (0.33, 1550.0, 20)
        );
    }

    #[test]
    fn mushroom_tint_lifts_the_blue_channel_most() {
        // A dark base so the byte-max never swallows the glow.
        let base = [0.05, 0.1, 0.2];
        // `num5` at the middle of `Next(28, 42) * 0.001` with `mouseTextColor`
        // at its rest value 225: 0.035 + 45/5000 = 0.044.
        let num5 = 0.035 + (270.0 - 225.0) / 5000.0;
        let t = mushroom_tint(base, 1.0, 0.5, [0.3, 0.45, 0.9], num5);
        // `(0.3, 0.45 + num5/2, 0.9 + num5) * 255 * 0.5`, clamped to 255.
        assert_eq!((t[0] * 255.0) as i32, 38);
        assert_eq!((t[1] * 255.0) as i32, 60);
        assert_eq!((t[2] * 255.0) as i32, 120);
        // The channel is never pushed below the base colour's byte.
        let dark = mushroom_tint([0.0, 0.0, 0.9], 1.0, 0.5, [0.0, 0.0, 0.0], 0.0);
        assert_eq!(dark[2], (0.9f32 * 255.0) as i32 as f32 / 255.0);
        // The cross-fade alpha scales the tint (`num4 = Colour.A / 255`).
        let faded = mushroom_tint(base, 0.25, 0.5, [0.3, 0.45, 0.9], num5);
        assert!(faded[2] < t[2] * 0.3, "faded b = {}", faded[2] * 255.0);
    }
}

/// W4-g70: `DrawSurfaceBG_Forest`'s per-id parallax math (`Main.cs:61273-61519`)
/// and the forest rows of `SetBackgroundOffsets` (`:60536-60770`). The tops
/// carry the front loop's `pushBGTopHack = -150`. Written, not run.
#[cfg(test)]
mod w4_g70_forest_layer_tests {
    use super::*;

    const HACK: f32 = -150.0;

    fn tex() -> Option<BgTex> {
        Some(BgTex { uv: [0.0, 0.0, 1.0, 1.0], w: 1024.0, h: 512.0 })
    }

    /// Style 0 (`[9, 10, 11]`): the plain row bases — scale 1.25/1.31/1.34 x2,
    /// parallax 0.40/0.43/0.49, tops 1500/1750/2000 minus the 150 hack. The
    /// row-0 double offsets call is a no-op for untabled ids, so its ×2
    /// survives (`Main.cs:61302-61307`).
    #[test]
    fn style_zero_keeps_the_row_bases() {
        let rows = forest_layers([9, 10, 11], [tex(); 3], HACK, None);
        assert!((rows[0].scale - 2.5).abs() < 1e-6);
        assert_eq!(rows[0].parallax, 0.40);
        assert_eq!((rows[0].k, rows[0].top), (1800.0, 1350.0));
        assert!((rows[1].scale - 2.62).abs() < 1e-6);
        assert_eq!(rows[1].parallax, 0.43);
        assert_eq!((rows[1].k, rows[1].top), (1950.0, 1600.0));
        assert!(!rows[1].flip, "only 51/54 mirror");
        assert!((rows[2].scale - 2.68).abs() < 1e-6);
        assert_eq!(rows[2].parallax, 0.49);
        assert_eq!((rows[2].k, rows[2].top), (2100.0, 1850.0));
        // The DrawToDepth boundaries are 1/parallax per row.
        assert!((rows[0].depth - 2.5).abs() < 1e-6);
        assert!((rows[2].depth - 1.0 / 0.49).abs() < 1e-6);
    }

    /// Style 1 (`[50, 51, 52]`): the -50 nudges on every row and the row-1
    /// mirror (`Main.cs:61327-61330`, `:61417-61420`, `:61440`, `:61487-61490`).
    #[test]
    fn style_one_nudges_fifty_and_mirrors_row_one() {
        let rows = forest_layers([50, 51, 52], [tex(); 3], HACK, None);
        assert_eq!(rows[0].top, 1300.0);
        assert_eq!(rows[1].top, 1550.0);
        assert!(rows[1].flip);
        assert_eq!(rows[2].top, 1800.0);
        assert!(!rows[0].flip && !rows[2].flip);
    }

    /// Style 3 (`[91, -1, 92]`): row 0 drops to 0.27/1.2 and its top rewrites
    /// to 1600/1400 then +200; row 2 drops to 0.42/1.3 AFTER the offsets call
    /// and nudges +150 (`Main.cs:61278-61282`, `:61311-61314`, `:61335-61338`,
    /// `:61457-61462`, `:61495-61498`).
    #[test]
    fn style_91_rewrites_row_zero_and_drops_row_two() {
        let rows = forest_layers([91, -1, 92], [tex(), None, tex()], HACK, None);
        assert_eq!(rows[0].parallax, 0.27);
        assert!((rows[0].scale - 2.4).abs() < 1e-6);
        assert_eq!((rows[0].k, rows[0].top), (1600.0, 1450.0));
        assert!(rows[1].tex.is_none(), "-1 draws nothing");
        assert_eq!(rows[2].parallax, 0.42);
        assert!((rows[2].scale - 2.6).abs() < 1e-6);
        assert_eq!((rows[2].k, rows[2].top), (2100.0, 2000.0));
        // The drop lands AFTER the row's DrawToDepth (`:61456`): the
        // boundary keeps the pre-drop 0.49, the X phase uses 0.42.
        assert!((rows[2].depth - 1.0 / 0.49).abs() < 1e-6);
    }

    /// Style 6 (`[173, -1, -1]`): row 0 is 0.25/1.3, top 1600/1400 +200, and
    /// the DRAWN sheet is the 251..254 frame the caller resolved
    /// (`Main.cs:61283-61287`, `:61315-61318`, `:61339-61342`, `:61371-61376`).
    #[test]
    fn style_173_swaps_the_drawn_sheet_to_the_frame() {
        let frame = Some(BgTex { uv: [0.5, 0.5, 1.0, 1.0], w: 100.0, h: 100.0 });
        let rows = forest_layers([173, -1, -1], [tex(), None, None], HACK, frame);
        assert_eq!(rows[0].parallax, 0.25);
        assert!((rows[0].scale - 2.6).abs() < 1e-6);
        assert_eq!((rows[0].k, rows[0].top), (1600.0, 1450.0));
        assert_eq!(rows[0].tex, frame);
        // The frame id ladder: 251 + (counter / 10) % 4 (`Main.cs:61373`).
        assert_eq!(forest_173_frame_id(0), 251);
        assert_eq!(forest_173_frame_id(9), 251);
        assert_eq!(forest_173_frame_id(10), 252);
        assert_eq!(forest_173_frame_id(39), 254);
        assert_eq!(forest_173_frame_id(40), 251);
    }

    /// Style 10 (`[282, -1, -1]`): the 1.4 pre-write is dead — the id's own
    /// `SetBackgroundOffsets` entry (2.6/0.25, 1800/1340) overwrites it, and
    /// the second offsets call (`:61307`) lands AFTER the ×2 with no
    /// re-multiply, so the tabled scale stays 2.6 (`Main.cs:61298-61307`,
    /// `:60542-60547`).
    #[test]
    fn style_282_rides_its_offsets_table_entry() {
        let rows = forest_layers([282, -1, -1], [tex(), None, None], HACK, None);
        assert_eq!(rows[0].scale, 2.6);
        assert_eq!(rows[0].parallax, 0.25);
        assert_eq!((rows[0].k, rows[0].top), (1800.0, 1190.0));
    }

    /// Style 11 (`[330, 329, 328]`): row 0's table top 1750/1450 then -150;
    /// row 1's 1900/1600 rewrite then -150 — its tabled scale 2.29 lands
    /// after the ×2 un-multiplied; row 2's 2600/2300 rewrite then -150,
    /// scale 2.01 likewise (`Main.cs:61389-61391`, `:61453-61455`).
    #[test]
    fn style_11_rewrites_all_three_rows() {
        let rows = forest_layers([330, 329, 328], [tex(); 3], HACK, None);
        assert_eq!(rows[0].parallax, 0.25);
        assert_eq!((rows[0].k, rows[0].top), (1750.0, 1150.0));
        assert_eq!(rows[1].scale, 2.29);
        assert_eq!(rows[1].parallax, 0.26);
        assert_eq!((rows[1].k, rows[1].top), (1900.0, 1300.0));
        assert_eq!(rows[2].scale, 2.01);
        assert_eq!(rows[2].parallax, 0.34);
        assert_eq!((rows[2].k, rows[2].top), (2600.0, 2000.0));
    }

    /// Style 13 (`[343, 342, 341]`): row 0 table 1650/1550 then -300; row 1
    /// parallax 0.25 + -300; row 2 parallax 0.30 + -50 and the 15-tick
    /// `GetBackgroundRect` frame step (`Main.cs:61249-61253`).
    #[test]
    fn style_13_nudges_and_animates_row_two() {
        let rows = forest_layers([343, 342, 341], [tex(); 3], HACK, None);
        assert_eq!(rows[0].parallax, 0.20);
        assert_eq!((rows[0].k, rows[0].top), (1650.0, 1100.0));
        assert_eq!(rows[1].parallax, 0.25);
        assert_eq!(rows[1].top, 1300.0);
        assert_eq!(rows[2].parallax, 0.30);
        assert_eq!(rows[2].top, 1800.0);
        assert_eq!(rows[2].frame_step, 15);
    }

    /// The forest `SetBackgroundOffsets` rows verbatim (`Main.cs:60536-60547`,
    /// `:60716-60770`): scale/parallax/num/num2, `None` where the C# leaves
    /// the caller's value.
    #[test]
    fn the_forest_offsets_rows_match_the_csharp_switch() {
        assert_eq!(bg_offsets(279), Some((Some(2.5), Some(0.35), 1850, 1750)));
        assert_eq!(bg_offsets(282), Some((Some(2.6), Some(0.25), 1800, 1340)));
        assert_eq!(bg_offsets(328), Some((Some(2.01), Some(0.34), 0, 0)));
        assert_eq!(bg_offsets(329), Some((Some(2.29), Some(0.26), 0, 0)));
        assert_eq!(bg_offsets(330), Some((None, Some(0.25), 1750, 1450)));
        assert_eq!(bg_offsets(333), Some((Some(2.01), Some(0.37), 0, 0)));
        assert_eq!(bg_offsets(334), Some((None, Some(0.26), 0, 0)));
        assert_eq!(bg_offsets(335), Some((None, Some(0.22), 1650, 1470)));
        assert_eq!(bg_offsets(341), Some((None, Some(0.30), 0, 0)));
        assert_eq!(bg_offsets(342), Some((None, Some(0.25), 0, 0)));
        assert_eq!(bg_offsets(343), Some((None, Some(0.20), 1650, 1550)));
        // The classic rows have no entry — the caller's bases survive.
        assert_eq!(bg_offsets(9), None);
        assert_eq!(bg_offsets(50), None);
        assert_eq!(bg_offsets(173), None);
    }

    /// `GetLayerYOffset`'s forest-hill arms (`Main.cs:59744-59793`).
    #[test]
    fn the_hill_y_offsets_match_the_csharp_switch() {
        assert_eq!(layer_y_offset(7), 0.0);
        assert_eq!(layer_y_offset(8), 0.0);
        assert_eq!(layer_y_offset(90), 0.0);
        assert_eq!(layer_y_offset(93), -50.0);
        assert_eq!(layer_y_offset(168), -50.0);
        assert_eq!(layer_y_offset(170), -50.0);
        assert_eq!(layer_y_offset(171), -100.0);
        assert_eq!(layer_y_offset(172), 130.0);
        assert_eq!(layer_y_offset(176), -760.0);
        assert_eq!(layer_y_offset(177), -200.0);
        assert_eq!(layer_y_offset(179), -100.0);
        assert_eq!(layer_y_offset(180), -350.0);
        assert_eq!(layer_y_offset(183), -350.0);
        assert_eq!(layer_y_offset(277), -260.0);
        assert_eq!(layer_y_offset(278), -120.0);
        assert_eq!(layer_y_offset(280), -170.0);
        assert_eq!(layer_y_offset(281), -300.0);
        assert_eq!(layer_y_offset(331), 0.0);
        assert_eq!(layer_y_offset(336), 0.0);
    }
}
