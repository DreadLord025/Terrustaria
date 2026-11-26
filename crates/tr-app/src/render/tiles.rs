//! The row-major tile pass (`Main.DrawTiles`-shaped): wall layer, the greedy
//! dark-rectangle merge, slope/half/sliced block variants and the per-frame
//! draw statistics.
use crate::assets;
use crate::item::ItemDrop;
use crate::net;
use crate::npc::Npc;
use crate::player::{self, Player};
use crate::render::entities::{
    draw_npcs, draw_player, flip_uv, ArmorKinds, NpcAssets, PlayerAssets,
};
use crate::render::foliage::{draw_tree_foliage, emit_living_tree_leaf};
use crate::render::liquids::{draw_tile_liquid_behind, draw_waters, LiquidPass};
use crate::ui::text::{hud_text, hud_text_width};
use glam::Vec2;
use std::time::Instant;
use tr_core::lighting::{LightMap, LightParams, LightStyle};
use tr_core::tiledata as td;
use tr_core::{tree, World, TILE_PX};
use tr_render::{Atlas, SpriteRenderer};

#[derive(Default, Clone, Copy, Debug)]
pub(crate) struct TileDrawStats {
    pub(crate) visible: u32,
    pub(crate) walls: u32,
    pub(crate) walls_culled: u32,
    pub(crate) dark_cells: u32,
    pub(crate) dark_rects: u32,
    pub(crate) chunks_skipped: u32,
    pub(crate) lit_tiles: u32,
    pub(crate) quads: u32,
    pub(crate) foliage: u32,
    pub(crate) liquids: u32,
    pub(crate) waterfalls: u32,
    pub(crate) items: u32,
    pub(crate) light_ms: f32,
    pub(crate) draw_ms: f32,
    pub(crate) instances: u32,
    pub(crate) world_reused: bool,
}

/// Scratch storage for the greedy dark-rectangle merge (A3/A4).
#[derive(Default)]
pub(crate) struct DrawScratch {
    /// Rectangles still being extended downwards: `[x0, x1, y0, y1]`.
    open: Vec<[i32; 4]>,
    next: Vec<[i32; 4]>,
    /// Dark x-runs of the current row.
    cur: Vec<(i32, i32)>,
    /// The waterfall `AddLight` sources of the previous frame
    /// (`WaterfallManager.AddLight`): vanilla's light engine consumes the
    /// previous frame's light list when `LightTiles` runs, so the lights are
    /// swapped here after each world pass and consumed by the next compute.
    pub(crate) waterfall_lights: Vec<tr_core::lighting::DynamicLight>,
    /// The `waterfall_lights` + `entity_lights` union the light compute reads
    /// (`LightingEngine.cs:215-225` walks one list; the port concatenates the
    /// two already-swapped source lists here). Rebuilt each world pass.
    pub(crate) all_lights: Vec<tr_core::lighting::DynamicLight>,
    /// The waterfall quads walked this frame (`WaterfallManager.draw`), consumed
    /// by the tile pass below.
    pub(crate) waterfall_quads: Vec<tr_core::waterfall::WaterfallDraw>,
    /// The liquid rects of the last world-pass build (world pixels) — the
    /// WaterDistortion mask input. It is refreshed exactly when the G6-cached
    /// world range is rebuilt, so it always matches the liquid sprites the
    /// cached range still holds.
    pub(crate) water_rects: Vec<[f32; 4]>,
    /// S1-G-13 consumers: the vine roots the `tile2` callback's crawl
    /// collected (`TileDrawing._vineRootsPositions`, `TileDrawing.cs:120`)
    /// — the `PostDrawTiles(solidLayer: false)` screen-batch `DrawVines`
    /// (`:429`) consumes them AFTER the tile2 composite, so the split path
    /// parks them here between the two callbacks. The fused default never
    /// reads it.
    pub(crate) vine_roots: Vec<(i32, i32)>,
    /// S1-G-13 consumers (GATE RNG fix): the shared leaf-roll stream the
    /// gated halves continue (`TileDrawing._rand` is ONE `UnifiedRandom`
    /// across every `Draw`/`PostDrawTiles` call, `TileDrawing.cs:104,191`).
    /// The fused default re-seeds per call (the frozen per-frame pin) and
    /// never reads this; the gated path pre-seeds it once per frame
    /// (`layer_targets::render_layer_targets`) and each `draw_tiles` call
    /// takes/continues/returns it, so the solid half's 192-cell rolls
    /// advance the stream the extras block's `EmitTreeLeaves` rolls read —
    /// the fused single-stream shape. ANALYSIS "The render-target family
    /// (S1-G-13)" A/B section.
    pub(crate) leaf_rng: Option<tr_core::tile_light::FastRandom>,
    /// Whether [`Self::leaf_rng`] rides along: the gated consumers set it
    /// (the layer callbacks AND their direct-screen blocks share one
    /// stream per frame); the fused default leaves it false — every fused
    /// call re-seeds from `sparkle_seed` and never touches the slot.
    pub(crate) leaf_rng_shared: bool,
    /// W4-d127 — `Main.wofDrawFrameIndex` (`Main.cs:761`): the WoF body's
    /// 18-tick mouth clock (`DrawWOFBody`, `:38426-38430`). Advances once
    /// per frame unless paused; feeds `WorldFrameKey.wof_key` so the
    /// cached world range rebuilds while a wall lives.
    pub(crate) wof_draw_frame_index: i32,
}

/// `WallID.EchoWall` (`WallID.cs:741`). `WallDrawing.DrawWalls` skips it
/// unless `Main.ShouldShowInvisibleBlocksAndWalls()` grants sight
/// (`WallDrawing.cs:91`'s `_shouldShowInvisibleWalls`, W4-g78 wired) —
/// hidden, it contributes neither a sprite nor a black quad.
pub(crate) const WALL_ECHO: u16 = 318;

/// `WallDrawing.DrawWalls` draw test (`WallDrawing.cs:91`), verbatim:
/// `wall <= 0 || (wall == 318 && !sight) || (tile.invisibleWall() && !sight)`.
/// The `FullTile` cull is the caller's separate gate (`tile_hides_wall`).
/// W4-g78: `show_invisible` is `Main.ShouldShowInvisibleBlocksAndWalls()`
/// (`Main.cs:58023-58030`) — the Echo Monolith / Spectre Goggles grants;
/// before this wave it was hardwired false.
#[inline]
pub(crate) fn wall_visible(wall: u16, invisible_wall: bool, show_invisible: bool) -> bool {
    wall != 0
        && (wall != WALL_ECHO || show_invisible)
        && (!invisible_wall || show_invisible)
}

/// Vanilla stores light as bytes; a channel below `1/255` is zero. Such a cell is
/// guaranteed to rasterise to black whatever the texture colour, so it can be
/// replaced by one opaque black quad without changing a single pixel.
#[inline]
fn light_is_zero(l: [f32; 3]) -> bool {
    l[0] * 255.0 < 1.0 && l[1] * 255.0 < 1.0 && l[2] * 255.0 < 1.0
}

/// Extends open dark rectangles with this row's runs and emits the ones that end.
/// Returns the number of emitted quads.
fn merge_dark_row(
    r: &mut SpriteRenderer,
    white: [f32; 4],
    scratch: &mut DrawScratch,
    y: i32,
) -> u32 {
    let mut emitted = 0u32;
    scratch.next.clear();
    let mut oi = 0usize;
    let mut ci = 0usize;
    while oi < scratch.open.len() || ci < scratch.cur.len() {
        let take_old = if oi >= scratch.open.len() {
            false
        } else if ci >= scratch.cur.len() {
            true
        } else {
            let o = scratch.open[oi];
            let c = scratch.cur[ci];
            o[0] < c.0 || (o[0] == c.0 && o[1] != c.1)
        };
        if take_old {
            let o = scratch.open[oi];
            r.push_sprite(
                [o[0] as f32 * TILE_PX, o[2] as f32 * TILE_PX],
                [
                    (o[1] - o[0] + 1) as f32 * TILE_PX,
                    (o[3] - o[2] + 1) as f32 * TILE_PX,
                ],
                white,
                [0.0, 0.0, 0.0, 1.0],
            );
            emitted += 1;
            oi += 1;
        } else if oi >= scratch.open.len() || scratch.cur[ci].0 < scratch.open[oi][0] {
            let (s, e) = scratch.cur[ci];
            scratch.next.push([s, e, y, y]);
            ci += 1;
        } else {
            // Same x span: grow the rectangle one row down.
            let mut o = scratch.open[oi];
            o[3] = y;
            scratch.next.push(o);
            oi += 1;
            ci += 1;
        }
    }
    std::mem::swap(&mut scratch.open, &mut scratch.next);
    emitted
}

/// The drawn frame of one world cell — `TileDrawing`'s source-rect resolution
/// (`TileDrawing.cs:1007`): `GetTileDrawData`'s sizes/strides feed the sheet
/// lookup, tree trunks ride the 20x20 trunk grid, and plain 16x16 cells hit the
/// 18 px frame table. A cell whose frames were never written
/// (`frameX/frameY = -1` — the state `WorldGen.PlaceTile` leaves between the
/// tile write and `SquareTileFrame`; the old seed bug wrote it permanently,
/// `PlaceAlch` fixes it) draws nothing instead of sampling sheet-corner
/// garbage: vanilla never draws one, because framing runs before the draw.
pub(crate) fn resolve_tile_frame(
    lookups: &mut assets::Lookups,
    atlas: Option<&Atlas>,
    is_tree: bool,
    trunk_offset: i16,
    kind: u16,
    frame_x: i16,
    frame_y: i16,
    dd: &tr_core::drawdata::TileDrawData,
) -> Option<assets::FrameTex> {
    if frame_x < 0 || frame_y < 0 {
        return None;
    }
    if is_tree {
        // Tree trunks are 20x20 sprites on a 22 px grid.
        return atlas.and_then(|a| {
            lookups.rect_uv(
                a,
                kind,
                frame_x + trunk_offset,
                frame_y,
                tree::TRUNK_SIZE as u32,
                tree::TRUNK_SIZE as u32,
            )
        });
    }
    // Cases that rewrite C#'s `ref tileFrameX/Y` hand the final frame to us;
    // the source rectangle is built from it (`TileDrawing.cs:1007`).
    let fx = dd.frame_x_override.unwrap_or(frame_x);
    let fy = dd.frame_y_override.unwrap_or(frame_y);
    let frame = atlas.and_then(|a| {
        if dd.width != 16 || dd.height != 16 || dd.add_fr_x != 0 || dd.add_fr_y != 0 {
            lookups.rect_uv(
                a,
                kind,
                fx + dd.add_fr_x,
                fy + dd.add_fr_y,
                dd.width as u32,
                dd.height as u32,
            )
        } else {
            lookups.frame_tex(a, kind, fx, fy)
        }
    });
    frame.or_else(|| lookups.plain_tex(kind))
}

/// The shroom-cap frame column (`TileDrawing.cs:994-1005`): only a
/// type-72 trunk-top cell at `tileFrameX >= 36` carries a cap;
/// `num4 = 0`, lifted to 1 at `tileFrameY == 18` and 2 at `== 36`
/// (exact equality — every other row keeps 0). The source rect is
/// `(num4 * 62, 0, 60, 42)` on `TextureAssets.ShroomCap`
/// (`Images/Shroom_Tops`, `AssetInitializer.cs:532`).
pub(crate) fn shroom_cap_frame(frame_x: i16, frame_y: i16) -> Option<i32> {
    if frame_x < 36 {
        return None;
    }
    Some(if frame_y == 18 {
        1
    } else if frame_y == 36 {
        2
    } else {
        0
    })
}

/// One cell of the sunflower's special-pass sway
/// (`TileDrawing.DrawMultiTileGrassInWind`, `TileDrawing.cs:8916-9041`).
#[derive(Clone, Copy, Debug, PartialEq)]
struct SunflowerSway {
    /// `vector3.Y` — the draw position's vertical offset
    /// `|windCycle| * 2 * num3` (`TileDrawing.cs:8985`); the horizontal offset
    /// comes from the rotation about the flower's bottom-center anchor.
    dy: f32,
    /// `windCycle * num * num3` with the type-27 factor `num = 0.07`
    /// (`TileDrawing.cs:8926`, `:9001`).
    rot: f32,
    /// `origin = vector - vector2` (`TileDrawing.cs:8990`) — the rotation pivot
    /// relative to the cell sprite's top-left, i.e. the flower's bottom-center
    /// anchor at `(topLeftX*16 + sizeX*8, topLeftY*16 + 16*sizeY)` with the
    /// special pass's `sizeX = 2, sizeY = 5` (`TileDrawing.cs:8748-8752`).
    pivot: [f32; 2],
}

/// The sway state of the sunflower cell at `(tx, ty)` whose flower's top-left
/// is reconstructed from the frame grid: `frameX = 36*style + 18*col`,
/// `frameY = 18*row` (`StyleHorizontal`, `RandomStyleRange = 3`,
/// `TileObjectData.cs:4851-4864`).
fn sunflower_sway(
    wind: &tr_core::wind::Wind,
    world: &World,
    tx: i32,
    ty: i32,
    frame_x: i16,
    frame_y: i16,
    sunflower_wind_counter: f32,
) -> SunflowerSway {
    let col = (frame_x.rem_euclid(36) / 18) as i32;
    let row = (frame_y / 18) as i32;
    let top_left_x = tx - col;
    let top_left_y = ty - row;
    // `GetWindCycle(topLeftX, topLeftY, _sunflowerWindCounter)` (`:8918`); the
    // surface-band and |wind| 0.08..0.18 gates live inside `Wind::cycle`.
    let cycle = wind.cycle(top_left_x, top_left_y, sunflower_wind_counter);
    // `flag = InAPlaceWithWind(topLeftX, topLeftY, sizeX, sizeY)` (`:8923`)
    // with the special pass's `sizeY = 5` — the probe reaches one row into the
    // flower's anchor ground.
    let in_wind = wind.in_a_place_with_wind(world, top_left_x, top_left_y, 2, 5);
    // `num3 = 1 - (j - topLeftY + 1) / sizeY` per row (`:8952`), the 0-row
    // guard `num3 == 0 -> 0.1` (`:8957`), and `!flag -> 0` (`:8961`).
    let mut num3 = 1.0 - (row + 1) as f32 / 5.0;
    if num3 == 0.0 {
        num3 = 0.1;
    }
    if !in_wind {
        num3 = 0.0;
    }
    SunflowerSway {
        dy: cycle.abs() * 2.0 * num3,
        rot: cycle * 0.07 * num3,
        pivot: [
            (top_left_x * 16 + 16 - tx * 16) as f32,
            (top_left_y * 16 + 80 - ty * 16) as f32,
        ],
    }
}

/// S1-G-13 consumers: which cells the tile pass draws — the fused default
/// (`All`, every cell, the frozen path) or one `solidLayer` half of
/// `Main.DrawTiles` (`Main.cs:21820`): `RenderTiles` draws the solid layer
/// (`Main.cs:57329-57348`), `RenderTiles2` the non-solid one (`:57340-57348`).
/// The C# split is `TileDrawing.Draw(solidLayer)`'s per-cell
/// `tileSolid[type]` test; the port's `td::TILE_SOLID` is that set.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TileFilter {
    /// The fused pass: solid and non-solid cells, one row-major walk.
    All,
    /// `solidLayer: true` — only `TILE_SOLID` cells.
    Solid,
    /// `solidLayer: false` — only the non-solid cells.
    NonSolid,
    /// S1-G-13 consumers: the wallTarget callback's black half —
    /// `RenderBlacks` (`Main.cs:58069-58075`, `DrawBlack(intoRenderTargets:
    /// true)`): the dark-cell merge only, no tile sprites (the wall sprites
    /// ride [`draw_wall_layer`] beside it).
    Wall,
    /// S1-G-13 consumers: the gated frame path's DIRECT-SCREEN extras —
    /// the cluster `Main.DoDraw_WallsTilesNPCs` draws between the tile2
    /// composite (`Main.cs:63367`) and the tile composite (`:63333`):
    /// `DoDraw_Waterfalls` (`:63306-63320`), the behind-tiles NPCs
    /// (`DoDraw_DrawNPCsBehindTiles`, `:63293/63298`) and `PostDrawTiles
    /// (solidLayer: false)`'s foliage/vines (`TileDrawing.cs:414-432`).
    /// NO tile cells, NO liquids, NO entity tail.
    ScreenExtras,
    /// S1-G-13 consumers: the gated frame path's DIRECT-SCREEN entity tail
    /// — the passes after the tile composite: NPCs over tiles
    /// (`Main.cs:63301`), the players (`:62256`), the dropped items
    /// (`:62278`) and the edit cursor. NO tile cells (the tile targets own
    /// them), NO liquid passes (the water/backWater targets own them).
    Screen,
}

/// `TileDrawing.IsTileDrawLayerSolid` (`TileDrawing.cs:4469-4475`): the
/// `TileID.Sets.DrawTileInSolidLayer` override first (the generated
/// `SET_DRAW_TILE_IN_SOLID_LAYER` table — 1 = forced solid, 0 = unset, the
/// `bool?` null; `TileID.cs:516-528` sets 11/78/470/475/579 true), else the
/// `_tileSolid` table — the port's `td::TILE_SOLID`.
pub(crate) fn tile_draw_layer_solid(kind: u16) -> bool {
    let forced = td::SET_DRAW_TILE_IN_SOLID_LAYER
        .get(kind as usize)
        .copied()
        .unwrap_or(0);
    if forced != 0 {
        return forced > 0;
    }
    td::TILE_SOLID.has(kind)
}

impl TileFilter {
    /// Does this cell draw under the filter? `TileDrawing.Draw`'s per-cell
    /// test (`TileDrawing.cs:517`): `!tile.active() ||
    /// IsTileDrawLayerSolid(type) != solidLayer → continue`. An actuated
    /// cell is still `active()` in the C# (the inactive bit only ghosts the
    /// tint, `GetFinalLight` `:4380-4397`), so it keeps its solidity half.
    pub(crate) fn keeps(self, kind: u16, active: bool) -> bool {
        match self {
            Self::All => true,
            Self::Solid => active && tile_draw_layer_solid(kind),
            Self::NonSolid => active && !tile_draw_layer_solid(kind),
            // `DrawBlack`'s own walk keeps every cell (the wall pass runs
            // before the per-cell filter arm ever asks); never reached.
            Self::Wall => true,
            // The direct-screen blocks draw no tile cells — the cell loop
            // skips before ever asking.
            Self::ScreenExtras | Self::Screen => false,
        }
    }
}

#[allow(clippy::too_many_arguments)]
/// Records one draw phase for the order-pinning test (`S1-B-7`) — the
/// `Main.DoDraw` sequence (`Main.cs:62249-62326`) made assertable.
#[inline]
fn draw_phase(name: &'static str) {
    #[cfg(test)]
    crate::app::frame::draw_phase_trace::record(name);
}

pub(crate) fn draw_tiles(
    r: &mut SpriteRenderer,
    world: &World,
    lookups: &mut assets::Lookups,
    atlas: Option<&Atlas>,
    camera: Vec2,
    zoom: f32,
    screen: Vec2,
    params: &LightParams,
    light: &mut LightMap,
    scratch: &mut DrawScratch,
    style: &LightStyle,
    entity_lights: &crate::entity_lights::EntityLights,
    anim_time: f32,
    // `LiquidRenderer.Update`'s two integer frames (the wind-driven liquid
    // texture clock, `LiquidRenderer.cs:844-845`) — ticked by the app frame
    // head, consumed by both liquid passes.
    liquid_frames: (i32, i32),
    waterfall: &mut tr_core::waterfall::WaterfallManager,
    player: Option<&Player>,
    player_assets: &PlayerAssets,
    // `DrawPlayer_27_HeldItem`'s input: the selected item kind, the
    // remaining use fraction (`PlayerDrawLayers.cs:3077`) and the item's
    // prefix (S1-P-i05 — the prefixed `GetAdjustedItemScale`); `None` =
    // empty hand.
    held: Option<(u16, f32, u16)>,
    // `Player.armor[0..3]` item kinds (head/body/legs) — the armor layers.
    armor: ArmorKinds,
    npcs: &[Npc],
    npc_assets: &NpcAssets,
    items: &[ItemDrop],
    remote: &[net::PlayerNet],
    edit_target: Option<(i32, i32)>,
    // `Main.liquidAlpha` (`Main.cs:1103`): the per-water-style cross-fade alphas
    // the `DrawWaters` ramp drives; the liquid and waterfall passes consume it.
    liquid_alpha: &[f32; 15],
    // `Main.SceneMetrics.ActiveFountainColor` (`SceneMetrics.cs:486-517`):
    // the tile-207 flowing-frame colour the `CalculateWaterStyle` head arm
    // reads (`Main.cs:57390-57393`); -1 = no flowing fountain in the scan.
    active_fountain_color: i32,
    // The frame's deterministic seed for the `TrySparkling` draws.
    sparkle_seed: u64,
    // The wind state and the sway counters (`TileDrawing.DrawGrass`/`DrawTrees`;
    // the sunflower's own counter rides its special pass,
    // `TileDrawing.DrawMultiTileGrassInWind`).
    wind: &tr_core::wind::Wind,
    grass_wind_counter: f32,
    tree_wind_counter: f32,
    sunflower_wind_counter: f32,
    // `EmitTreeLeaves` spawns now go straight into the gore pool (W4-g68):
    // this is the pool (`Main.gore`) and the `Main.rand` stream the
    // `NewGore*` internals roll on. The gate draws stay on `leaf_rng`
    // (the `TileDrawing._rand` stand-in below).
    gore: &mut tr_core::gore::GorePool,
    main_rand: &mut tr_core::UnifiedRandom,
    // `TileDrawing._isActiveAndNotPaused` (`TileDrawing.cs:469`,
    // `FocusHelper.AllowTileDrawingToEmitEffects`) — the particle-emission
    // gate; the port reads `!game_paused` (the focus gate is unported).
    emit_effects: bool,
    // `TileDrawing._vineWindCounter` (`TileDrawing.cs:375`): the vine
    // strips' own sway counter (S1-G-1).
    vine_wind_counter: f32,
    // S1-F-9 slice B: the `GetTileDrawData` engine-driven case arm's extra
    // inputs — `PressurePlateHelper.PressurePlatesPressed` (case 428),
    // `BirthdayParty.PartyIsUp` (case 455) and
    // `TileDrawing._shouldShowInvisibleBlocks` (case 541). W4-g80:
    // `rain_cloud_alpha` (`Main.cloudAlpha`) feeds the 83 herb-maturity sheet
    // swap's waterleaf gate (`WorldGen.IsAlchemyPlantHarvestable`,
    // `TileDrawing.cs:1450-1455`); the probe's day/night/moon inputs read
    // `world.*` internally.
    wiring: &tr_core::wiring::Wiring,
    party_is_up: bool,
    show_invisible_blocks: bool,
    rain_cloud_alpha: f32,
    // S1-G-1f: the wired tile animator (`Main.tileFrame`/`tileFrameCounter`,
    // `Main.cs:1596-1597`) — `GetTileDrawData`'s `Main.tileFrame[typeCache]`
    // row (`TileDrawing.cs:4533`), slice B's per-type counter and the
    // weather-vane bob (`Main.cs:2237`). Ticked once per client update by
    // `App::animate_tiles_phase` (`Main.cs:17150`).
    tile_anim: &tr_core::drawdata_anim::TileAnimator,
    // The 92-cage-id `addFrY` rewrite (`TileDrawing.cs:5629-5907`, W4-g34's
    // `critter_cage_anim` tables + W4-f14's wire) is served by the ONE
    // `cage_anim` param below: f14's animator carries the same `frame_of`
    // table walk plus `Main.critterCage`'s live flag. The duplicate g34
    // immutable param was dropped at the union; its module/surface stay live.
    critter_cage: &mut bool,
    // W4-d58: `Main.chest[]` (`Main.cs:66994`) — the chest rewrite
    // (`CacheSpecialDraws_Part2`, `TileDrawing.cs:4289-4330`) reads each
    // chest's lid `frame` (the S1-P-i43 animator's `Chest.frame`, advanced
    // on the world step). Read-only: the draw side rolls no RNG.
    chests: &[crate::chest::WorldChest],
    // S1-G-13 consumers: `Main.DrawTiles`' `solidLayer` split
    // (`Main.cs:21820`, the `RenderTiles`/`RenderTiles2` callbacks at
    // `:57329-57348`). The fused default passes `None` — every cell draws
    // here exactly as before; the layer callbacks pass the two halves.
    tile_filter: TileFilter,
    // W4-f14: the critter-cage animator (`Main.critterCage` + the
    // `Main.*CageFrame` tables, `Main.cs:1445`/`:66902-66972`). The draw
    // CLEARS the flag at the non-solid layer start (`TileDrawing.cs:474`)
    // and SETS it while a cage cell draws (`:5672` etc.); the cage arms'
    // `frame_of` reads the tables. Ticked at the `AnimateTiles` tail
    // (`Main.cs:19303`) by `App::animate_tiles_phase`.
    cage_anim: &mut tr_core::drawdata_cages::CritterCageAnimator,
    // W4-f6 wiring — the lava-bubble drain (`LiquidRenderer.cs:552-577`,
    // sites 560/564): the renderer's PRIVATE `_random` stream
    // (`LiquidRenderer.cs:132`, NOT a swap key), the dust pool + spawn ctx.
    liquid_rand: &mut tr_core::UnifiedRandom,
    dust: &mut tr_core::particles::DustPool,
    spawn_ctx: &tr_core::particles::SpawnCtx,
    // W4-g12 doc-sweep: `Main.martianLight` (`Main.cs:1231`, the
    // `DoUpdate_AnimateTileGlows` ±0.015 bounce `:18251-18261`) — the
    // `_martianGlow` feed (`TileDrawing.cs:498-499`). Ticks EVERY frame in
    // C#, no event gate; the glow record rides unused until the glow pass
    // binds it (TILE-DRAW-2), but the value is now the live one.
    martian_light: f32,
    // W4-g12 doc-sweep: `DrawGoreBehind` (`Main.cs:21842-21871`) reads the
    // SAME `gore` pool above — the drip rows outside frames 7..=9 draw in
    // the behind-tiles slot of this pass (the `:56934-56940` site). The
    // union's second `gore_pool` alias was a duplicate borrow and is folded.
    // W4-g85: the orb-family emit slice's inputs (`DrawTiles_EmitParticles`,
    // `TileDrawing.cs:7335-7347`/`:7413-7529`, entry gate `:965-968`).
    // The pool is the SAME `dust` above (`Main.dust`) — the union's second
    // `dust` param was an aliasing duplicate and cannot be inhabited by
    // two simultaneous `&mut`; the body reads `dust`. `orb_main_rand` is the
    // `Main.rand` stand-in for the `NewDust` internals (`world.frame_rng`,
    // borrowed through the draw like the d54 break seam). `orb_ctx` carries
    // the `Dust.NewDust` cull inputs (`App::spawn_ctx`). The `_rand` gate
    // stream is the pass-local `leaf_rng` (the pinned per-rebuild stand-in).
    orb_main_rand: &mut tr_core::UnifiedRandom,
    orb_ctx: &tr_core::particles::SpawnCtx,
    // W4-d127 — `Main.gamePaused` for the WoF mouth clock's advance gate
    // (`DrawWOFBody`, `Main.cs:38427`).
    game_paused: bool,
    // W4-d70: `MoonLordScreenShaderData.LatestFilterStrength` — the
    // `GetMoonLordColoring` input threaded to the NPC pass.
    latest_filter_strength: f32,
) -> TileDrawStats {
    let t_total = Instant::now();
    // W4-d127 — `Main.wofDrawFrameIndex`'s per-frame tick (`Main.cs:38427-
    // 38430`): advance unless paused, wrap at 18. Runs BEFORE the body draw
    // (the counter feeds the key and the pass reads the fresh value).
    crate::render::wof::tick_wof_frame_index(&mut scratch.wof_draw_frame_index, game_paused);
    // `TileDrawing._rand` (`TileDrawing.cs:104,191`): ONE `UnifiedRandom`
    // lives across every `Draw`/`PostDrawTiles` call. The fused default
    // keeps the frozen per-frame pin (re-seed each call); the gated halves
    // continue the shared stream the caller pre-seeded once per frame
    // (GATE RNG fix — the solid half's cell rolls must advance the stream
    // the extras block's foliage rolls read, exactly like the fused walk).
    let mut leaf_rng = if scratch.leaf_rng_shared {
        scratch.leaf_rng.take().unwrap_or_else(|| {
            tr_core::tile_light::FastRandom::new(sparkle_seed ^ 0x9E37_79B9_7F4A_7C15)
        })
    } else {
        tr_core::tile_light::FastRandom::new(sparkle_seed ^ 0x9E37_79B9_7F4A_7C15)
    };
    let half = screen * 0.5 / zoom;
    let min = camera - half;
    let max = camera + half;
    let x0 = ((min.x / TILE_PX).floor() as i32 - 1).clamp(0, world.width - 1);
    let x1 = ((max.x / TILE_PX).ceil() as i32 + 1).clamp(0, world.width - 1);
    let y0 = ((min.y / TILE_PX).floor() as i32 - 1).clamp(0, world.height - 1);
    let y1 = ((max.y / TILE_PX).ceil() as i32 + 1).clamp(0, world.height - 1);

    // W4-f14: `Main.critterCage = false` at the non-solid layer's start
    // (`TileDrawing.Draw`, TileDrawing.cs:472-475). The port draws both
    // layers in ONE walk (no solidLayer split), so the clear rides the
    // walk's start — the cage arms below re-set it per in-view cage cell
    // exactly like the C# non-solid pass; the flag then gates the NEXT
    // `animate_tiles_phase` tick (Main.cs:19303).
    cage_anim.critter_cage = false;

    // S1-F-9 slice B: the engine-driven `GetTileDrawData` cases' per-frame
    // environment (`drawdata_cases_b.rs`). S1-G-1f: the animator is WIRED —
    // the weather-vane bob rides `TileAnimator::weather_vane_bobframe`
    // (`Main.weatherVaneBobframe`, `Main.cs:2237`); the clock is
    // `Main.GlobalTimeWrappedHourly = TotalGameTime.TotalSeconds % 3600`
    // (`Main.cs:16836`).
    let case_env = crate::drawdata_cases_b::CaseEnv {
        world,
        wind,
        wiring,
        inputs: crate::drawdata_cases_b::CaseInputs {
            weather_vane_bobframe: tile_anim.weather_vane_bobframe,
            player_center_x: player.map(|p| p.center().0),
            show_invisible_blocks,
            party_is_up,
            global_time_wrapped_hourly: anim_time % 3600.0,
        },
    };

    // `Main.waterStyle` is a single value for the whole frame
    // (`Main.DrawWaters`, `Main.cs:57475`: `CalculateWaterStyle()`) — the
    // fountain head arm (`:57390-57393`) first, then the biome walk.
    let mut water_style = crate::biome::calculate_water_style_with_fountain(
        world,
        camera,
        active_fountain_color,
    );
    // `TR_WATER_STYLE=<n>`: pin the water style so the per-style passes, the
    // `liquidAlpha` cross-fade and the biome liquid sheets stay screenshotable.
    if let Some(n) = crate::env::water_style() {
        water_style = n.min(14);
    }
    let t_light = Instant::now();
    // Vanilla applies the `AddLight` list one frame late (the engine keeps the
    // previous frame's list when `LightTiles` runs, `LightingEngine.cs:210-214`);
    // `scratch` carries the waterfall sources and `entity_lights` the entity
    // ones, both already swapped by the previous frame.
    scratch.all_lights.clear();
    scratch.all_lights.extend_from_slice(&scratch.waterfall_lights);
    scratch.all_lights.extend_from_slice(entity_lights.visible());
    // S1-E-19: in the legacy modes the frame pass already ran the legacy
    // engine's `ProcessArea` and published its presented colours into this
    // map (`Lighting.LightTiles`, `Main.cs:62145`); the modern blur compute
    // stays untouched and is skipped — the legacy engine owns the light.
    if style.mode == tr_core::lighting::LightMode::Color {
        light.compute_into_with_lights(
            world,
            x0,
            y0,
            x1,
            y1,
            params,
            &scratch.all_lights,
            waterfall.lights_key() ^ entity_lights.key().rotate_left(17),
        );
    }
    let light_ms = t_light.elapsed().as_secs_f32() * 1000.0;

    if let Some((lx, ly)) = crate::env::light_at() {
        use std::sync::atomic::{AtomicBool, Ordering};
        static ONCE: AtomicBool = AtomicBool::new(false);
        if !ONCE.swap(true, Ordering::Relaxed) {
            let c = light.get(lx, ly);
            let tile = world.tile(lx, ly).copied().unwrap_or_default();
            log::info!(
                "light at {lx},{ly} = {:.3},{:.3},{:.3} (tile {} wall {} liquid {})",
                c[0],
                c[1],
                c[2],
                tile.kind,
                tile.wall,
                tile.liquid
            );
        }
    }

    let width = world.width as usize;
    let white = r.white_uv();
    // `TileDrawing.Draw`'s non-solid-pass head clears `Main.critterCage`
    // (`TileDrawing.cs:474`); the port's single row-major pass clears it
    // here and the cage cells below re-raise it (W4-g34).
    *critter_cage = false;
    let mut stats = TileDrawStats {
        lit_tiles: light.lit_tiles,
        light_ms,
        ..Default::default()
    };
    let chunk = tr_core::world::CHUNK;
    // Vanilla's quality thresholds (`TileDrawing.Draw`, default gfxQuality 0.5):
    // above the high mark blocks get nine light slices, above the medium mark four,
    // below it a single flat quad.
    const HIGH_LIGHT: f32 = 142.0 / 255.0;
    const MEDIUM_LIGHT: f32 = 26.0 / 255.0;

    // `LiquidRenderer.InternalPrepareDraw` runs once per frame; both the opaque
    // background pass and the translucent foreground pass consume this record.
    // W4-f6 wiring: the bubble cells drain `world_dust::lava_bubbles` on the
    // renderer's private `_random` (the `Dust.lavaBubbles < 200` gate + the
    // two independent `Next` rolls live in the emitter).
    let (liquid_draws, bubble_cells) = tr_core::liquid_draw::build_with_bubbles(world, x0, y0, x1, y1);
    for cell in bubble_cells {
        crate::world_dust::lava_bubbles(
            cell,
            dust.lava_bubbles,
            liquid_rand,
            dust,
            spawn_ctx,
            world,
        );
    }
    // M1.4-BG-18 phase 4: the same records feed the WaterDistortion mask.
    scratch.water_rects = crate::render::liquids::water_mask_rects(&liquid_draws);
    if crate::env::dump_liquid() {
        let cam_x = camera.x as i32 / 16;
        let cam_y = camera.y as i32 / 16;
        let mut n_vis = 0;
        let mut n_tot = 0;
        for d in &liquid_draws {
            if (d.x - cam_x).abs() < 40 && (d.y - cam_y).abs() < 40 {
                n_tot += 1;
                if d.visible {
                    n_vis += 1;
                }
            }
        }
        log::info!(
            "LIQDUMP near cam {},{}: total={} visible={}",
            cam_x,
            cam_y,
            n_tot,
            n_vis
        );
        for d in &liquid_draws {
            if d.visible && (d.x - cam_x).abs() < 40 && (d.y - cam_y).abs() < 40 {
                log::info!(
                    "  vis tile {},{} type{} lvl{} sw{} sh{} fy{} fx{} sx{} sy{}",
                    d.x,
                    d.y,
                    d.liquid_type,
                    d.src_w,
                    d.src_h,
                    d.is_surface as i32,
                    d.src_y,
                    d.src_x,
                    d.offset_x,
                    d.offset_y
                );
            }
        }
    }
    // `Main.DrawLiquid(bg: true, ...)`: water is drawn before walls and tiles at
    // `Opacity` (usually 1.0), so the parallax background never shows through it.
    // S1-G-13 consumers: only the fused default draws it here — under the
    // gated path the backWaterTarget callback owns this pass
    // (`Main.RenderBackgroundWater`, `Main.cs:55183-55198` drives
    // `DrawWaters(isBackground: true)`), and the direct-screen block draws
    // no liquids at all (the composites carry them).
    if tile_filter == TileFilter::All && !crate::env::no_liqback() {
        stats.liquids += draw_waters(
            r,
            world,
            lookups,
            atlas,
            light,
            style,
            anim_time,
            liquid_frames,
            &liquid_draws,
            LiquidPass::Background,
            water_style,
            liquid_alpha,
        );
    }

    // `Main.DoDraw_Waterfalls`: vanilla draws falls after the non-solid tile pass
    // and before solid tiles. This renderer keeps one row-major tile pass, so the
    // falls are emitted here; tile quads drawn later occlude their base, while a
    // wall behind a fall is drawn over it (documented approximation). The quads
    // were walked once per frame in the pre-draw phase (`App::frame`), which also
    // collects their `AddLight` list for the light map — keeping the walk out of
    // the G6-cached path makes the light feed deterministic per frame index.
    // S1-G-13 consumers: under a split tile half the falls stay in the
    // scratch for the direct-screen extras' `DoDraw_Waterfalls` push
    // (`Main.cs:63306-63320`, between the tile2 and tile composites); the
    // fused default AND the extras block emit them here (the fused path's
    // documented approximation keeps them early; the extras block sits at
    // the C# site).
    if matches!(tile_filter, TileFilter::All | TileFilter::ScreenExtras) {
        for d in std::mem::take(&mut scratch.waterfall_quads) {
            // `WaterfallManager.DrawWaterfall(int, ...)`'s type-25 arm
            // (`WaterfallManager.cs:972-980`): the shimmer fall replaces the tint
            // with the self-luminous base + sparkle pair.
            if d.texture == 25 {
                stats.waterfalls +=
                    crate::render::liquids::draw_shimmer_fall(r, lookups, anim_time, &d);
                continue;
            }
            let Some(uv) = lookups.waterfall_uv(d.texture as usize, d.source) else {
                continue;
            };
            r.push_sprite(
                d.pos,
                [d.source[2] as f32, d.source[3] as f32],
                flip_uv(uv, d.flip),
                d.color,
            );
            stats.waterfalls += 1;
        }
    }

    // `Main.DrawWalls`: the whole wall layer is drawn before the tile layer. Each
    // frame is a 32x32 sprite placed 8 px up/left of the cell, so neighbouring walls
    // join without seams; drawing it interleaved with tiles would let the overhang
    // cut into tiles drawn earlier in the row.
    if crate::env::walls_enabled() {
        for ty in y0..=y1 {
            for tx in x0..=x1 {
                let tile = &world.tiles[ty as usize * width + tx as usize];
                if tile.wall == 0 {
                    continue;
                }
                // `WallDrawing.cs:91` evaluates `FullTile` BEFORE the echo
                // gates. `FullTile` (`WallDrawing.cs:258-261`, W4-g78): a
                // hidden Echo-coated block does NOT hide its wall — the wall
                // draws behind it (the `invisibleBlock() &&
                // !_shouldShowInvisibleWalls → return false` arm; the raw
                // coating bit, NOT the IsVisible overrides).
                if tile.active()
                    && (!world.invisible_block(tx, ty) || show_invisible_blocks)
                    && tile_hides_wall(
                        lookups,
                        atlas,
                        tile.kind,
                        tile.frame_x,
                        tile.frame_y,
                        tile.slope(),
                        tile.half_brick(),
                    )
                {
                    stats.walls_culled += 1;
                    continue;
                }
                // `WallDrawing.cs:91` (W4-g78): the echo wall 318 AND the
                // Echo-coating bit both ride `ShouldShowInvisibleBlocksAndWalls`
                // — the monolith/goggles sight lifts them together.
                if !wall_visible(tile.wall, world.invisible_wall(tx, ty), show_invisible_blocks) {
                    continue;
                }
                let Some(wf) = lookups.wall_tex(tile.wall, tile.wall_col(), tile.wall_row()) else {
                    continue;
                };
                // `WallDrawing.DrawWalls` skips black cells (`:104-115`);
                // the tile pass merges the fully covered ones into black
                // rectangles. The shown echo wall is EXEMPT: `:100-103`
                // repaints it white BEFORE the black test (W4-g78).
                if light_is_zero(light.get(tx, ty)) && tile.wall != WALL_ECHO {
                    continue;
                }
                // The 32 px sprite is centred on the 16 px cell (`x*16 - 8`), like
                // `new Vector2(j * 16 - 8, i * 16 - 8)` in the original.
                stats.quads += push_wall_lit(
                    r,
                    [tx as f32 * TILE_PX - 8.0, ty as f32 * TILE_PX - 8.0],
                    wf.uv,
                    light,
                    style,
                    tx,
                    ty,
                    anim_time,
                    world.wall_paint(tx, ty),
                    // `WallDrawing.cs:96-103`: a Glow-coated wall AND the
                    // echo wall 318 both sample white (W4-g78 — the shown
                    // echo wall draws its sprite white, never black).
                    world.fullbright_wall(tx, ty) || tile.wall == WALL_ECHO,
                );
                stats.walls += 1;
            }
        }
    }

    // Main.cs:22320,56951,63390 — behind-tiles NPCs draw before the tile layer
    // so they appear inside/behind solid tiles (worms, WoF segments). Town
    // NPCs never ride this pass, so the party-hat context is None here.
    draw_phase("npcs_behind_tiles");
    // W4-d125: the HallowBoss draw's enrage/expert inputs (the world
    // statics + the player-buff expert mirror, the npc.rs convention).
    let empress_ctx = Some(crate::render::entities::EmpressDrawCtx {
        day_time: world.day_time,
        remix_world: world.seeds.contains(tr_core::seeds::SeedFlags::REMIX),
        empress_rage_mode: world.empress_rage_mode,
        world_surface: world.world_surface,
        expert: player.map_or(false, |p| p.buffs.expert_mode),
    });
    stats.quads += draw_npcs(
        r,
        npc_assets,
        npcs,
        light,
        style,
        anim_time,
        camera,
        zoom,
        screen,
        true,
        empress_ctx,
        None,
        latest_filter_strength,
    );
    // cut into tiles drawn earlier in the row. S1-G-13 consumers: the loop body
    // moved verbatim into `draw_wall_layer` so the wallTarget callback
    // (`Main.RenderWalls`, `Main.cs:58081-58091`) drives the SAME pass; under a
    // split tile half the wallTarget callback owns it, so it is skipped here.
    if tile_filter == TileFilter::All {
        draw_wall_layer(
            r, world, lookups, atlas, light, style, anim_time, x0, y0, x1, y1, &mut stats,
        );
    }

    // W4-d127 — `Main.DrawWoF` (`Main.cs:38377-38398`): the wall's body
    // sweep + the Hungry rope leashes + the tongue chains. The C# runs it
    // AFTER `DoDraw_WallsAndBlacks` and BEFORE the behind-tiles NPC pass
    // (`Main.cs:63272-63279`); the port's fused pass draws the walls above,
    // so the seam sits here, ahead of `npcs_behind_tiles`.
    {
        let sheets = crate::render::wof::WofSheets {
            body: npc_assets.wof_body,
            chain12: npc_assets.wof_chain12,
            chain4: npc_assets.wof_chain4,
        };
        let wof_ctx = crate::render::wof::WofDrawCtx {
            world,
            npcs,
            player,
            frame_index: scratch.wof_draw_frame_index,
        };
        stats.quads += crate::render::wof::draw_wof(
            r, &sheets, &wof_ctx, light, style, anim_time, camera, zoom, screen,
        );
    }
        let _ = latest_filter_strength;

    // W4-g12 doc-sweep: `Main.DrawGoreBehind` (`Main.cs:21842-21871`,
    // capture site `:56934-56940`) — the drip rows outside frames 7..=9
    // draw after the walls and BEFORE the behind-tiles NPCs, under the
    // tile layer. The C# gate is the `Main.drawBackGore` latch the front
    // `DrawGore` walk sets (`:21885`); `draw_behind`'s own walk gate is
    // the same pool predicate, so the latch needs no port state. These
    // quads ride the G6-CACHED world range, so the drip state rides the
    // key (`WorldFrameKey::back_gore`, the waterfall convention: 0 while
    // no drip slot draws — ANALYSIS "The back gore pass (W4-g12)").
    stats.quads += crate::render::gore::draw_behind(
        gore, r, lookups, light, style, anim_time,
    );

    // S1-G-13 consumers: the direct-screen extras block (`ScreenExtras`)
    // owns the behind-tiles NPC pass at its `DoDraw_DrawNPCsBehindTiles`
    // site (`Main.cs:63293/63298`, between the tile2 and tile composites);
    // the tile halves never draw entities. The fused default is unchanged.
    if matches!(tile_filter, TileFilter::All | TileFilter::ScreenExtras) {
        draw_phase("npcs_behind_tiles");
        stats.quads += draw_npcs(
            r,
            npc_assets,
            npcs,
            light,
            style,
            anim_time,
            camera,
            zoom,
            screen,
            true,
            empress_ctx,
            None,
            latest_filter_strength,
        );
    }

    scratch.open.clear();
    // `TileDrawing._vineRootsPositions` / the `TileCounterType.Vine` special
    // points (`TileDrawing.cs:120`, `:744-762`): rebuilt per draw in
    // row-major draw-area order, cleared like `ClearSpecialBlockCounts`
    // (`:390-395`).
    let mut vine_roots: Vec<(i32, i32)> = Vec::new();
    let mut vine_roots_seen: std::collections::HashSet<(i32, i32)> =
        std::collections::HashSet::new();
    // W4-g83: `TileDrawing._reverseVineRootsPositions` / the
    // `TileCounterType.ReverseVine` special points (`TileDrawing.cs:121`,
    // `:764-782`): the seaweed (549) roots, same per-draw rebuild + dedup as
    // the hanging vines (`ClearSpecialBlockCounts`, `:391`). The g31 union
    // folds this list into `SpecialPoints.buckets[ReverseVine]` at the
    // tile-pass wiring (docs/research/123-vine-wire-contract.md §1).
    let mut reverse_vine_roots: Vec<(i32, i32)> = Vec::new();
    let mut reverse_vine_roots_seen: std::collections::HashSet<(i32, i32)> =
        std::collections::HashSet::new();
    // W4-d58: `TileDrawing._chestPositions` (`TileDrawing.cs:96`,
    // `:4300-4303`) — the anchor → `Chest.FindChest` result cache, cleared
    // each draw (`ClearLegacyCachedDraws`, `:6613-6615`): a fresh map per
    // `draw_tiles` call. The cached value is the chest's lid frame (`None`
    // = the `-1` miss), read through `chest::chest_draw_frame`.
    let mut chest_positions: std::collections::HashMap<(i32, i32), Option<i32>> =
        std::collections::HashMap::new();
    // S1-G-13 consumers: the direct-screen blocks draw NO tile cells — the
    // tile targets own them, and re-walking the cells here would re-roll the
    // leaf RNG (`EmitTreeLeaves`) the tile halves already rolled. The cell
    // loop runs only for the fused default and the tile-target callbacks.
    let skip_cells = matches!(tile_filter, TileFilter::ScreenExtras | TileFilter::Screen);
    for ty in y0..=y1 {
        if skip_cells {
            break;
        }
        scratch.cur.clear();
        let mut run_start = -1i32;
        let cy = ty / chunk;
        let row_base = ty as usize * width;
        for cx in (x0 / chunk)..=(x1 / chunk) {
            if !world.chunk_has(cx, cy) {
                stats.chunks_skipped += 1;
                if run_start >= 0 {
                    let sx = (cx * chunk).max(x0);
                    scratch.cur.push((run_start, sx - 1));
                    run_start = -1;
                }
                continue;
            }
            let sx = (cx * chunk).max(x0);
            let ex = ((cx + 1) * chunk - 1).min(x1);
            for tx in sx..=ex {
                let tile = &world.tiles[row_base + tx as usize];
                let raw_l = light.get(tx, ty);
                let inactive = tile.flags & tr_core::tile::FLAG_INACTIVE != 0;
                let active = tile.active();
                // W4-g85 — the spelunker pre-entry arm (`TileDrawing.cs:920-937`),
                // the orb-family slice. `findTreasure` (buff 9, landed S1-E-7a
                // `player.rs:6464`) + `IsTileSpelunkable` admits 12/639 (the
                // family's `tileSpelunker` members, `Main.cs:9424`/`:9651`);
                // 31/696 are NOT in the set. The light lift (`:922-929`, R→200
                // G→170, B untouched) runs on the RAW cell light BEFORE the
                // actuation ghost — C# `GetFinalLight` (`:4380`) applies
                // `actColor` AFTER the lift — and the lifted `tileLight` is what
                // the emit battery receives (`:967`), so a buff-lit 12/639 cell
                // is never dark to the shine arm. The `Next(60)` sparkle
                // (`:930`) draws OUTSIDE the `:965` entry quarter-roll, once per
                // admitted cell per unpaused frame, on the `_rand` stand-in.
                let spelunk = active
                    && player.is_some_and(|p| p.find_treasure)
                    && crate::render::orb_draw::spelunker_family(tile.kind);
                let tile_light = if spelunk {
                    crate::render::orb_draw::spelunker_light_lift(raw_l)
                } else {
                    raw_l
                };
                let mut l = tile_light;
                // `Tile.actColor` (`Tile.cs:272-290`) via
                // `TileDrawing.GetFinalLight` (`TileDrawing.cs:4380-4397`):
                // an actuated (inactive) tile draws ghosted — RGB × 0.4.
                // This is `drawData.finalColor` (`:1019`); the sliced arm
                // re-derives its own per-slice colour from the RAW
                // `drawData.tileLight` (`:3528`) instead.
                if inactive {
                    l = [l[0] * 0.4, l[1] * 0.4, l[2] * 0.4];
                }
                let active = tile.active();
                // S1-G-13 consumers: the wallTarget callback's black half
                // (`TileFilter::Wall`, `RenderBlacks` → `DrawBlack`): the
                // dark-cell merge only — no sprite, no drawdata, no RNG roll
                // (the tile halves own the leaf stream; the wall pass never
                // rolls it, `TileDrawing._rand` is per-layer, `:191`). The
                // run-break mirrors the fused arm below.
                if tile_filter == TileFilter::Wall {
                    let flame_tile = active
                        && (tr_core::flame::tile_flame(tile.kind)
                            || tr_core::flame::has_glow_branch(tile.kind));
                    if light_is_zero(l)
                        && !flame_tile
                        && (active
                            || (crate::env::walls_enabled()
                                && wall_visible(
                                    tile.wall,
                                    world.invisible_wall(tx, ty),
                                    show_invisible_blocks,
                                )))
                    {
                        if run_start < 0 {
                            run_start = tx;
                        }
                        stats.dark_cells += 1;
                    } else if run_start >= 0 {
                        scratch.cur.push((run_start, tx - 1));
                        run_start = -1;
                    }
                    continue;
                }
                // `TileDrawing.Draw`'s per-cell layer test
                // (`TileDrawing.cs:517` — `!tile.active() ||
                // IsTileDrawLayerSolid(type) != solidLayer → continue`),
                // BEFORE the vine crawl (the C# `switch` sits after it). The
                // fused default (`All`) keeps every cell — unchanged.
                if !tile_filter.keeps(tile.kind, active) {
                    continue;
                }
                // `DrawSingleTile`'s first gate (`TileDrawing.cs:872-875`):
                // a WET lily pad (active 518, liquid > 0) never draws in the
                // tile pass — `LiquidRenderer` redraws it under the water
                // quad via `Main.DrawTileInWater` (liquids.rs, W4-g12
                // render-sweep-2). Before GetTileDrawData, so no frame
                // data, no black quad, no dust either. Dry pads (liquid 0)
                // keep the normal path — the case-518 dry-float arm.
                if active && tile.kind == 518 && tile.liquid > 0 {
                    // The C# return precedes DrawBlack (`:988-990`): the
                    // cell emits NO black quad, so an open dark run must
                    // break here (a merged rect would paint over the pad
                    // the liquid pass draws).
                    if run_start >= 0 {
                        scratch.cur.push((run_start, tx - 1));
                        run_start = -1;
                    }
                    continue;
                }
                // `TileDrawing.Draw`'s vine arm (`TileDrawing.cs:536-545`):
                // a `VineThreads` cell never draws through `DrawSingleTile`
                // — the crawl registers the strip's root as a special point
                // and `PostDrawTiles` draws the whole strip after the trees
                // (`DrawVines`, `:429`, S1-G-1). The root list is rebuilt in
                // draw-area row-major order, deduplicated like
                // `_vineRootsPositions` (`:757-760`). The crawl runs for
                // every active vine cell regardless of light (the C#
                // switch precedes any draw/black test); the dark-cell merge
                // below still covers unlit vine cells exactly as before.
                let vine_cell = active && crate::render::vines::is_vine_thread(tile.kind);
                if vine_cell {
                    crate::render::vines::collect_vine_root(
                        world,
                        tx,
                        ty,
                        &mut vine_roots_seen,
                        &mut vine_roots,
                    );
                }
                // W4-g83: the switch's `case 549` arm (`TileDrawing.cs:546-
                // 548`) — `CrawlToBottomOfReverseVineAndAddSpecialPoint`
                // registers the rising strip's root and `continue`s, exactly
                // like the vine arm above. Same position: before any
                // light/draw gate, regardless of cell light.
                let reverse_vine_cell =
                    active && crate::render::vines::is_reverse_vine_thread(tile.kind);
                if reverse_vine_cell {
                    crate::render::vines::collect_reverse_vine_root(
                        world,
                        tx,
                        ty,
                        &mut reverse_vine_roots_seen,
                        &mut reverse_vine_roots,
                    );
                }
                let is_tree = active && tree::is_tree(tile.kind);
                let marker = if is_tree {
                    tree::TreePart::marker(tile.frame_x, tile.frame_y)
                } else {
                    None
                };

                // `GetTileDrawData`: per-type sprite size/top/flip/frame data
                // (torches 20x20 with `tileTop 4`, plants 20 px, herbs,
                // candles, palms, frame strides...). `tile_frame` is
                // `Main.tileFrame[kind]` (`TileDrawing.cs:4533`), fed from
                // the wired animator (S1-G-1f, `drawdata_anim.rs`).
                let dd = if active && marker.is_none() && !is_tree {
                    // W4-g34 deviation fix (d61/d64 R3): case 4's probe is the
                    // FULL `WorldGen.SolidTile(x, y - 1)` (`WorldGen.cs:70111-
                    // 70132`) — active + solid + not-solid-top + not-half-brick
                    // + slope 0 + not-inactive, null-true — not the old
                    // three-clause subset. The exact predicate already lives
                    // in `drawdata_cases_a::solid_tile` (the 518 probe).
                    let solid_above = crate::drawdata_cases_a::solid_tile(world, tx, ty - 1);
                    tr_core::drawdata::TileDrawData::default()
                } else {
                    tr_core::drawdata::TileDrawData::default()
                };
                let (dd, glow) = if active && marker.is_none() && !is_tree {
                    let solid_above = world.tile(tx, ty - 1).is_some_and(|t| {
                        t.active() && td::TILE_SOLID.has(t.kind) && !td::TILE_SOLID_TOP.has(t.kind)
                    });
                    // The animator arrays are `[TileID.Count]`
                    // (`Main.cs:66976-66977`); an out-of-range kind keeps the
                    // vanilla t=0 row (the `solid` probe below bounds-checks
                    // the same way).
                    let anim_row = if (tile.kind as usize) < td::TILE_COUNT {
                        tile_anim.animation_row(tile.kind)
                    } else {
                        0
                    };
                    let anim_counter = if (tile.kind as usize) < td::TILE_COUNT {
                        tile_anim.tile_frame_counter[tile.kind as usize]
                    } else {
                        0
                    };
                    let mut dd = tr_core::drawdata::draw_data(
                        tile.kind,
                        tx,
                        ty,
                        tile.frame_x,
                        tile.frame_y,
                        anim_row,
                        solid_above,
                    );
                    // The world-probing frame corrections C# applies in
                    // `GetTileDrawData` (palm biome row 323, dye plants 227)
                    // and `DrawSingleTile` (cactus 80, cog 272).
                    // The 83 herb-maturity probe reads `world.day_time`/
                    // `world.time`/`world.blood_moon`/`world.moon_phase`
                    // internally (`IsAlchemyPlantHarvestable`); the WeatherCtx
                    // carries the live rain/clouds. `Main.remixWorld` has no
                    // port state (the M5-SEED-2 arm, same as the smart-cursor
                    // harvest arm, `world_sim/smart_cursor.rs`).
                    let herb_wx = tr_core::growth::WeatherCtx {
                        raining: params.raining,
                        cloud_alpha: rain_cloud_alpha,
                        remix_world: false,
                    };
                    dd.apply_world_corrections(
                        world,
                        tile.kind,
                        tx,
                        ty,
                        tile.frame_x,
                        tile.frame_y,
                        anim_row,
                        &herb_wx,
                    );
                    // S1-F-9 slice B: the engine-driven special-tile cases
                    // (wind, the player, the party, pressed plates, echo
                    // sight, the clock) applied onto the static table's
                    // output — the delegated dispatch arm of
                    // `GetTileDrawData` (`drawdata_cases_b.rs`). The C#
                    // FIRST switch runs before the second (glow) switch;
                    // slice A's 518/529/530/441/468 ids are deduped out of
                    // slice B (A owns them), so the two passes are disjoint.
                    if let Some(cased) = crate::drawdata_cases_b::case_draw_data(
                        tile.kind,
                        tx,
                        ty,
                        tile.frame_x,
                        tile.frame_y,
                        anim_row,
                        anim_counter,
                        dd,
                        &case_env,
                    ) {
                        dd = cased;
                    }
                    // W4-g34 union (research/197 §2.2/§2.3): the critter-cage
                    // rewrite — the C# FIRST GetTileDrawData switch's four
                    // cage case groups (TileDrawing.cs:5629-5907, ALL 92
                    // ids: g34's 50 + g33's 42), at the FIRST switch's
                    // TAIL position: AFTER the static
                    // table + slice B's delegated first-switch cases,
                    // BEFORE slice A's SECOND switch (:6049-6507). The
                    // 568/569/570/580 glow rects read `tileFrameY + addFrY`
                    // POST-cage (:6085-6104), so the rewrite must land
                    // before `apply_cases_a` builds them, and before the
                    // source-rect resolve (:1007 reads the final
                    // addFrX/addFrY). `cage_draw_data` REPLACES the static
                    // cage arms with `top = 2` + the per-animal
                    // `addFrY = Main.<table>[idx] * stride` read from the
                    // wired cage animator (`AnimateTiles_CritterCages`,
                    // `Main.cs:13511-16596`). The chest rewrite stays with
                    // the chest lane (W4-d58's `:993` seam). The
                    // `Main.critterCage = true` side effect (:5672 etc.)
                    // rides the post-dd flag block below — the port's
                    // fused-pass stand-in, one cell later in the same walk.
                    if tr_core::drawdata_furniture::is_cage_kind(tile.kind) {
                        if let Some(cage) = tr_core::drawdata_furniture::cage_draw_data(
                            tile.kind,
                            tx,
                            ty,
                            tile.frame_x,
                            tile.frame_y,
                            tr_core::drawdata_furniture::CAGE_FRAMES,
                            |table, row, idx| cage_anim.frame_of(table, row, idx),
                        ) {
                            dd = cage;
                        }
                    }
                    // S1-F-9d slice A: the delegated static-case pass —
                    // LilyPad's liquid float (518), the oasis/sea-oats
                    // biome rows (530/529) and the SECOND GetTileDrawData
                    // switch's frame/top maths + glow pick
                    // (TileDrawing.cs:6049-6507), applied after the main
                    // switch like the C#. TILE-DRAW-2 (W4-f15): the glow
                    // record is BOUND — `render/tile_glow.rs` draws it at
                    // the C# glow-bind site (:1343-1403) after the cell
                    // sprite + flames. The cell light converts to XNA
                    // bytes (alpha 255, the `Lighting.GetColor` colour has
                    // no alpha); the UI text pulse and disco colours ride
                    // `LightParams`. martianLight is the LIVE bounce
                    // (W4-g12 doc-sweep: `Main.cs:17145` ticks it
                    // unconditionally; the old "0 outside the event"
                    // reading was wrong).
                    let cell = raw_l;
                    let cell_bytes = [
                        (cell[0] * 255.0).min(255.0) as u8,
                        (cell[1] * 255.0).min(255.0) as u8,
                        (cell[2] * 255.0).min(255.0) as u8,
                        255,
                    ];
                    // W4-f14: the census furniture post-pass — the C#
                    // FIRST GetTileDrawData switch's four cage case groups
                    // (TileDrawing.cs:5629-5907), at the FIRST switch's
                    // tail position: AFTER the static table + slice B's
                    // delegated first-switch cases, and BEFORE slice A's
                    // SECOND switch (`:6049-6507`) — the 568/569/570/580
                    // glow rects read `tileFrameY + addFrY` POST-cage
                    // (`:6085-6104`), so the rewrite must land before
                    // `apply_cases_a` builds them, and before the
                    // source-rect resolve (`:1007`'s rectangle reads
                    // addFrX/addFrY). The cage half REPLACES the static
                    // cage arms (top 2 + the `Main.*CageFrame[row, idx] *
                    // stride` rewrite); the `Main.critterCage = true`
                    // side effect (`:5672` etc.) gates the animator tick.
                    // The chest half of `furniture_post_pass` stays
                    // UNWIRED here — the `CacheSpecialDraws_Part2`
                    // position (`:993`) is owned by the in-flight W4-d58
                    // chest-frames wiring (the `chest_frame: None` call
                    // keeps this pass cage-only; `chest_draw_rewrite` is
                    // a no-op for non-chest kinds and the cage kinds
                    // never overlap BasicChest).
                    if tr_core::drawdata_furniture::is_cage_kind(tile.kind) {
                        cage_anim.critter_cage = true; // TileDrawing.cs:5672 etc.
                        tr_core::drawdata_furniture::furniture_post_pass(
                            tile.kind,
                            tx,
                            ty,
                            tile.frame_x,
                            tile.frame_y,
                            tr_core::drawdata_furniture::CAGE_FRAMES,
                            |table, row, idx| cage_anim.frame_of(table, row, idx),
                            &mut dd,
                        );
                    }
                    let glow = crate::drawdata_cases_a::apply_cases_a(
                        &mut dd,
                        world,
                        tile,
                        tx,
                        ty,
                        cell_bytes,
                        params.mouse_text_color,
                        martian_light,
                        params.disco,
                        anim_row,
                    );
                    // W4-d58: `CacheSpecialDraws_Part2`'s BasicChest block
                    // (`TileDrawing.cs:4289-4330`), which `DrawSingleTile`
                    // runs AFTER `GetTileDrawData` + both switches (`:993`)
                    // and BEFORE the source rectangle (`:1007`). The lid
                    // frame comes from the store through the per-draw
                    // `_chestPositions` cache; `tr_render::chest_frame_adjust`
                    // owns the +0/38/76 row maths + the bottom-row 18 px
                    // clamp. The golden-style glow rect (`:4326-4329`) is
                    // recorded but unbound (the glow pass is TILE-DRAW-2).
                    // Read-only — no sim RNG on the draw side.
                    if tr_core::drawdata_furniture::is_basic_chest(tile.kind) {
                        let lid = crate::chest::chest_draw_frame(
                            chests,
                            &mut chest_positions,
                            tx,
                            ty,
                            tile.frame_x,
                            tile.frame_y,
                        );
                        let adj = tr_render::chest_frame_adjust(
                            tile.kind,
                            tile.frame_x,
                            tile.frame_y,
                            dd.width as i16,
                            dd.height as i16,
                            lid,
                        );
                        dd.add_fr_x = adj.add_fr_x;
                        dd.add_fr_y = adj.add_fr_y;
                        if let Some(h) = adj.height_override {
                            dd.height = h as u16;
                        }
                    }
                    (dd, glow)
                } else {
                    (tr_core::drawdata::TileDrawData::default(), None)
                };

                // `Main.critterCage = true` (`TileDrawing.cs:5672`/`:5781`/
                // `:5838` etc.): every drawn cage cell raises the proximity
                // flag that gates `AnimateTiles_CritterCages`. The port's
                // single row-major pass raises it here for ANY cage kind;
                // the caller reads it back after the pass and the animator
                // consumes it on the next tick (the C# clears it at the
                // non-solid pass head, `:474`).
                if active && tr_core::drawdata_furniture::is_cage_kind(tile.kind) {
                    *critter_cage = true;
                }

                // Resolve the drawn tile frame once; reused for opacity and drawing.
                let mut tile_frame = None;
                if active && marker.is_none() {
                    let trunk_offset = if is_tree && tile.frame_x >= 0 && tile.frame_y >= 0 {
                        // `GetTileDrawData` case 5's biome sheet section
                        // (TileDrawing.cs:4645-4649) via the GetTreeBiome
                        // probe; variant trunks keep their own sheets.
                        tree::trunk_sheet_offset_at(
                            world,
                            tx,
                            ty,
                            tile.kind,
                            tile.frame_x,
                            tile.frame_y,
                        ) as i16
                    } else {
                        0
                    };
                    tile_frame = resolve_tile_frame(
                        lookups,
                        atlas,
                        is_tree,
                        trunk_offset,
                        // `case 83`'s `GetTileDrawTexture` re-pick
                        // (TileDrawing.cs:1149-1151 → :1450-1455): the
                        // harvestable herb draws the mature sheet 84.
                        dd.sheet_kind_override.unwrap_or(tile.kind),
                        tile.frame_x,
                        tile.frame_y,
                        &dd,
                    );
                }
                // W4-g85 — the spelunker sparkle (`TileDrawing.cs:930-936`),
                // drawn BEFORE the emit battery (the C# arm order: :920
                // precedes :965). The `_rand.Next(60)` rides `leaf_rng` and
                // is NOT gated by the entry quarter-roll; a paused frame
                // draws nothing (`_isActiveAndNotPaused`). The light lift
                // above already ran (it is draw-free and unconditional on
                // the roll). Keep-one: d60's whole-prologue port owns the
                // dangerSense `:911` / biomeSight `:955` siblings — the
                // family kinds fail both sets (census §6/§7), so this slice
                // draws nothing extra for them.
                if spelunk && !game_paused {
                    crate::render::orb_draw::spelunker_dust(
                        &mut crate::render::orb_draw::GateRand(&mut leaf_rng),
                        orb_main_rand,
                        dust,
                        orb_ctx,
                        world,
                        tx,
                        ty,
                    );
                }
                // W4-g85 — `DrawSingleTile`'s `DrawTiles_EmitParticles` entry
                // (`TileDrawing.cs:965-968`), the orb-family slice only. The
                // C# slot is BEFORE the visibility/black gates (`:975-991`):
                // a fully dark orb cell still runs the battery (the ambient
                // arm has no light test; the shine arm's dark return is
                // inside the body). The kind gate precedes every draw: a
                // non-family cell costs zero RNG (the full battery is
                // W4-d60's keep-one lane; the family kinds reach their arms
                // with zero prior `_rand` draws — every preceding arm fails
                // its type/set test first, census
                // docs/research/W4-g85-orb-draw.md §6). The entry quarter-roll
                // is a fresh per-tile FastRandom derivation (no stream state);
                // the arm gates ride `leaf_rng` (the `_rand` stand-in) and the
                // NewDust internals ride `orb_main_rand` (Main.rand).
                if active
                    && matches!(tile.kind, 31 | 696 | 12 | 639)
                    && crate::render::orb_draw::emit_entry_gate(
                        !game_paused,
                        // `Lighting.UpdateEveryFrame` (`Lighting.cs:77-87`):
                        // `!RenderTargetsRequired && !NotRetro` — Retro/Trippy
                        // at zoom 1. The port's frame path never binds the
                        // target family (S1-G-13 allocation-only), so
                        // `RenderTargetsRequired` stands false (recorded
                        // deviation: the zoom > 1 capture path is unported).
                        matches!(
                            style.mode,
                            tr_core::lighting::LightMode::Retro
                                | tr_core::lighting::LightMode::Trippy
                        ),
                        tr_core::flame::tile_frame_seed_at(anim_time),
                        tx,
                        ty,
                    )
                {
                    // The battery receives `drawData.tileLight` (`:967`) —
                    // the spelunker-lifted light, NOT the raw map colour.
                    let light_bytes = tr_core::particles::Color::new(
                        (tile_light[0] * 255.0).min(255.0) as u8,
                        (tile_light[1] * 255.0).min(255.0) as u8,
                        (tile_light[2] * 255.0).min(255.0) as u8,
                        255,
                    );
                    // The `:7335` ambient arm — 31/696 only; the type test
                    // inside draws nothing for 12/639.
                    crate::render::orb_draw::orb_ambient_dust(
                        &mut crate::render::orb_draw::GateRand(&mut leaf_rng),
                        orb_main_rand,
                        dust,
                        orb_ctx,
                        world,
                        tx,
                        ty,
                        tile.kind,
                        tile.frame_x,
                    );
                    // The `:7413` shine arm — the family's live rows are
                    // 12/639 (`Main.tileShine` 300); 31/696 have no row and
                    // the slice returns before any draw. The C# reaches the
                    // shine arm only when the ambient arm's chain test
                    // failed — for 12/639 the ambient type test always
                    // fails, so both calls in sequence are stream-exact.
                    crate::render::orb_draw::orb_shine_sparkle(
                        &mut crate::render::orb_draw::GateRand(&mut leaf_rng),
                        orb_main_rand,
                        dust,
                        orb_ctx,
                        world,
                        tx,
                        ty,
                        tile.kind,
                        light_bytes,
                    );
                }
                let solid = (tile.kind as usize) < td::TILE_COUNT && td::TILE_SOLID.has(tile.kind);
                // A3/A4: fully black cell -> greedy merged rectangle.
                // Vanilla paints black coverage from two places, both under
                // `TileDrawingBase.DrawOwnBlacks`:
                //  * `TileDrawing.DrawSingleTile`: every *active* tile whose light is
                //    exactly zero emits a black quad behind its sprite (`DrawBlack`).
                //  * `WallDrawing.DrawWalls`: every cell with a wall that is not a
                //    `FullTile` (block already covering it) emits a black quad.
                // That coverage is what hides the parallax background inside unlit
                // caves. A 32x32 wall sheet frame is mostly transparent outside its
                // 16x16 core, so `frame.opaque` is false; a wall has to count as
                // covering on its own. A fully dark tile renders identically whether
                // its sprite is drawn or skipped (both are solid black), so the cell
                // is folded into one merged rectangle — except flame/glow tiles:
                // `DrawSingleTile_Flames` draws their self-lit overlay over the
                // black sprite (`TileDrawing.cs:985-1016` sets `flag` for
                // `Main.tileFlame`/glow types regardless of light), so their cells
                // must keep their own quad. W4-g80: 692 (RainbowMossBlock)
                // carries GlowMask 341 (`Main.cs:10043`), so the C# `flag`
                // (`TileDrawing.cs:971`) lifts it out of the dark merge even
                // before the glow bind lands (TILE-DRAW-2 / W4-f15) — its
                // positional-switch disco `finalColor` (`:1078-1088`) draws
                // over the black. 160 (RainbowBrick) has NO mask/flame row:
                // the C# `!flag` return (`:1014-1017`) precedes the switch,
                // so a dark 160 cell draws black in the C# too — the port's
                // merge is faithful there.
                let glow_mask_tile = active && tile.kind == 692;
                // must keep their own quad. TILE-DRAW-2: a `glowTexture`
                // record sets the same `flag` (`TileDrawing.cs:971`), so a
                // glow-carrying cell (e.g. the 711 boulder) never merges.
                let flame_tile = active
                    && (tr_core::flame::tile_flame(tile.kind)
                        || tr_core::flame::has_glow_branch(tile.kind));
                // W4-g78: a shown echo wall (318) draws WHITE
                // (`WallDrawing.cs:100-103`, `Main.wallLight[318] = true`
                // `Main.cs:10580` takes the flat-colour arm) — its dark cell
                // is NOT black coverage, so the merge must not claim it.
                let echo_wall_lit = tile.wall == WALL_ECHO && show_invisible_blocks;
                // W4-g78: hidden Echo-coated cells paint NO black coverage —
                // `DrawSingleTile`'s `DrawBlack` needs `flag2` (`:988`, the
                // IsVisible veto) and `Main.DrawBlackTiles`' block/wall arms
                // both carry `(!invisible… || flag)` (`Main.cs:58006-58007`).
                // The port's merged rectangle is the fold of both passes, so
                // the coverage test rides the same sight gates (the parallax
                // shows through a hidden echo block in a dark cave, exactly
                // like the C#).
                let block_covers = active
                    && tr_core::scene::tile_is_visible(
                        tile.kind,
                        tile.frame_y,
                        world.invisible_block(tx, ty),
                        show_invisible_blocks,
                    );
                let wall_covers = crate::env::walls_enabled()
                    && tile.wall != 0
                    && (!world.invisible_wall(tx, ty) || show_invisible_blocks)
                    && (tile.wall != WALL_ECHO || show_invisible_blocks);
                let keep_quad = (light_is_zero(l) && !flame_tile && !echo_wall_lit && (block_covers || wall_covers)
                        || tr_core::flame::has_glow_branch(tile.kind)
                        || glow_mask_tile
                        || glow.is_some());
                if light_is_zero(l)
                    && !flame_tile
                    && (active
                        || (crate::env::walls_enabled()
                            && wall_visible(
                                tile.wall,
                                world.invisible_wall(tx, ty),
                                show_invisible_blocks,
                            )))
                {
                    // S1-G-13 consumers: under a split half the black coverage
                    // belongs to the wallTarget callback (`RenderBlacks`,
                    // `Main.cs:58069-58075` — `DrawBlack(intoRenderTargets:
                    // true)`); the tile half only skips the cell (the C#
                    // `!flag` return, `TileDrawing.cs:1014-1017`).
                    if tile_filter == TileFilter::All {
                        if run_start < 0 {
                            run_start = tx;
                        }
                        stats.dark_cells += 1;
                    }
                    continue;
                }
                if run_start >= 0 {
                    scratch.cur.push((run_start, tx - 1));
                    run_start = -1;
                }

                // `DrawSingleTile`'s `vector` (`TileDrawing.cs:1010-1014`) plus
                // the switch's per-type nudges (`:1020-1159`, `dd.draw_dx/dy`);
                // the width centring stays per-arm (`:1013`'s `num5`).
                let pos = [
                    tx as f32 * TILE_PX + dd.draw_dx as f32,
                    ty as f32 * TILE_PX + dd.draw_dy as f32,
                ];
                // W4-g80: `case 160/692` (`TileDrawing.cs:1078-1088`) —
                // `finalColor = new Color(Main.DiscoR, Main.DiscoG,
                // Main.DiscoB, 255)`, then `inActive → Tile.actColor`
                // (`Tile.cs:272-280`, the double 0.4 truncation). The
                // override REPLACES `GetFinalLight`'s light colour, so the
                // arms that draw `finalColor` (flat/sized/slope/half) take
                // these bytes; the sliced arm draws its own per-slice
                // `tileLight` (`:3578-3601`) and never reads `finalColor`.
                let disco_final: Option<[f32; 4]> = if dd.disco_override {
                    Some(disco_final_color(params.disco, inactive))
                } else {
                    None
                };
                // Walls are drawn in their own pass above (`Main.DrawWalls`).
                // `DrawTile_LiquidBehindTile` (solid layer): a slope that stands in
                // liquid draws the water mask behind its open half, so the shoreline
                // is water instead of the parallax background.
                if solid && tile_frame.is_none_or(|f| !f.opaque) {
                    stats.quads += draw_tile_liquid_behind(
                        r,
                        world,
                        lookups,
                        light,
                        style,
                        anim_time,
                        tx,
                        ty,
                        tile,
                        water_style,
                        waterfall,
                    );
                }
                // W4-g12 doc-sweep: the glowing-mushroom cap
                // (`TileDrawing.cs:994-1006`, research/09's "shroom cap
                // MISSING"): a type-72 trunk top at `frameX >= 36` draws
                // the `Shroom_Tops` 60x42 sprite over the cell BEFORE the
                // trunk quad, at `(x*16 - 22, y*16 - 26)` with the raw
                // `Lighting.GetColor(x, y)` tint and the case's flip.
                // Runs for every visible cap cell regardless of the draw
                // `flag` (the C# site precedes the `!flag` return); the
                // port's dark-cell merge above already folded the black
                // cells, whose cap tint is zero anyway.
                if active && tile.kind == 72 {
                    if let Some(col) = shroom_cap_frame(tile.frame_x, tile.frame_y) {
                        if let Some(uv) = r.uv("Shroom_Tops", col as f32 * 62.0, 0.0, 60.0, 42.0)
                        {
                            let c = style.shade(raw_l, tx, ty, anim_time);
                            r.push_sprite(
                                [tx as f32 * TILE_PX - 22.0, ty as f32 * TILE_PX - 26.0],
                                [60.0, 42.0],
                                flip_uv(uv, dd.flip),
                                [c[0], c[1], c[2], 1.0],
                            );
                            stats.quads += 1;
                        }
                    }
                }
                // The switch's early `return` arms (751/752 off their (0,0)
                // frame, the palm's frond columns 323): no sprite at all.
                if dd.skip_draw {
                    continue;
                }
                // The vine cells draw in the deferred strip pass below
                // (`DrawVines`, `TileDrawing.cs:429`) — never inline. The
                // reverse-vine (549) cells ride `DrawReverseVines` (`:430`,
                // W4-g83) — the switch's `continue` at `:548`.
                if vine_cell || reverse_vine_cell {
                    continue;
                }
                let Some(tf) = tile_frame else {
                    continue;
                };
                // `TileDrawing.IsVisible` (`TileDrawing.cs:1411-1432`),
                // W4-g78: the shared verbatim port — the Echo-coating bit
                // hides unless sight is granted (Echo Monolith / Spectre
                // Goggles / vanity 5345), and the platform row 48 + the
                // always-invisible kinds 541/631 ride the flag regardless
                // of the coating bit (the pre-wave port missed all three).
                if !tr_core::scene::tile_is_visible(
                    tile.kind,
                    tile.frame_y,
                    world.invisible_block(tx, ty),
                    show_invisible_blocks,
                ) {
                    continue;
                }
                stats.visible += 1;
                // S1-G-1f: `DrawTile_BackRope` (`TileDrawing.cs:3766-3784`),
                // wired at the two C# dispatch sites: `DrawBasicTile`'s
                // planter/platform gate (`:1511-1514`, BEFORE the cell's own
                // sprite) and `DrawTile_MinecartTrack` (`:3792`, before the
                // track sprites). Both run after `DrawSingleTile`'s `!flag`
                // return (`:1014-1017`) and the switch's early-return arms —
                // the dark-cell merge, `skip_draw`, unframed and echo-hidden
                // `continue`s above are those gates. The tint is the RAW
                // `drawData.tileLight` (`:3781`, `raw_l` — never the
                // actuation-ghosted `l`); the sheet/paint come from the rope
                // tile at the column's top end (`:3776-3780`).
                if crate::render::ropes::wants_back_rope(tile.kind) {
                    stats.quads += crate::render::ropes::draw_back_rope(
                        r, world, lookups, atlas, raw_l, style, anim_time, tx, ty, dd.flip,
                    );
                }
                // `TileDrawing.DrawGrass` (`TileDrawing.cs:8326-8375`): the
                // `SwaysInWindBasic` tiles draw with the grass wind counter —
                // position shifted by `(cycle, |cycle|)` and rotated
                // `cycle * 0.1` about `(tileWidth / 2, 16 - tileTop)`. The
                // row-major loop draws them inline instead of deferring them to
                // a special pass (documented z-order approximation); the
                // `WindGrid` gust push is not ported. Vanilla's type-227 frame
                // exceptions apply (`ShouldSwayInWind`, `TileDrawing.cs:7712`).
                // `TileDrawing.DrawGrass` (`TileDrawing.cs:8326-8375`): the
                // `SwaysInWindBasic` tiles draw with the grass wind counter —
                // position shifted by `(cycle, |cycle|)` and rotated
                // `cycle * 0.1` about `(tileWidth / 2, 16 - tileTop)`. The
                // row-major loop draws them inline instead of deferring them to
                // a special pass (documented z-order approximation); the
                // `WindGrid` gust push is not ported. Vanilla's type-227 frame
                // exceptions apply (`ShouldSwayInWind`, `TileDrawing.cs:7712`).
                let sway = if tr_core::wind::SWAYS_IN_WIND_BASIC.has(tile.kind)
                    && !matches!(tile.frame_x, 204 | 238 | 408 | 442 | 476)
                    && wind.sway_enabled
                {
                    let mut num3 = wind.cycle(tx, ty, grass_wind_counter);
                    if !tr_core::wind::wall_allows_wind(tile.wall) {
                        num3 = 0.0;
                    }
                    if !wind.in_a_place_with_wind(world, tx, ty, 1, 1) {
                        num3 = 0.0;
                    }
                    num3
                } else {
                    0.0
                };
                if is_tree {
                    // 20x20 trunk sprite, drawn like `TileDrawing` does: the 20 px
                    // sprite is centred on the 16 px cell (position shifted by -2),
                    // which is what makes branches meet the trunk exactly.
                    push_lit(
                        r,
                        [pos[0] - 2.0, pos[1]],
                        [tree::TRUNK_SIZE, tree::TRUNK_SIZE],
                        tf.uv,
                        l,
                        style,
                        tx,
                        ty,
                        anim_time,
                        tile.paint(),
                        &dd,
                        None,
                    );
                    stats.quads += 1;
                } else if tile.kind == 27 {
                    // The sunflower never draws through `DrawSingleTile` — C#
                    // registers the flower's top-left cell as a MultiTileGrass
                    // special (`TileDrawing.cs:600-606`) and
                    // `DrawMultiTileGrass`/`DrawMultiTileGrassInWind`
                    // (`TileDrawing.cs:8721-8805`, `:8916-9041`) draw the 2x4
                    // from `_sunflowerWindCounter` (the cadence
                    // `1/420 + num*5/420`, `:374`) about the flower's
                    // bottom-center. The row-major loop draws each cell inline
                    // with the same per-cell transform (the documented z-order
                    // approximation, as for the grass sway); `windCycle = 0`
                    // degenerates to the static draw, exactly like the special
                    // pass with no wind.
                    let sw = sunflower_sway(
                        wind,
                        world,
                        tx,
                        ty,
                        tile.frame_x,
                        tile.frame_y,
                        sunflower_wind_counter,
                    );
                    let (dw, dh) = (dd.width as f32, dd.height as f32);
                    let tint = style.shade(l, tx, ty, anim_time);
                    // `vector2 + (0, vector3.Y)` in world pixels — the XNA call
                    // places the pivot `origin` at `vector + (0, vector3.Y)`
                    // (`:8985-9001`), so the sprite's top-left lands at
                    // `(i*16, j*16 + tileTop + dy)`; type 27 has `tileTop = 0`.
                    r.push_sprite_origin_rot(
                        [pos[0], pos[1] + sw.dy],
                        [dw, dh],
                        tf.uv,
                        [tint[0], tint[1], tint[2], 1.0],
                        sw.rot,
                        sw.pivot,
                    );
                    stats.quads += 1;
                    // The glowmask: case 27 sets `flag2`/`num2 = 74`
                    // (`:8924-8928`) — the same transform draws the sheet region
                    // 74 px below the cell with `Color.White` (`:9004-9008`),
                    // the face glow (Tiles_27.xnb is 108x148: 74 px of styles +
                    // 74 px of glow art).
                    if let Some(a) = atlas {
                        let glow = lookups.rect_uv(
                            a,
                            tile.kind,
                            dd.frame_x_override.unwrap_or(tile.frame_x) + dd.add_fr_x,
                            dd.frame_y_override.unwrap_or(tile.frame_y) + dd.add_fr_y + 74,
                            dd.width as u32,
                            dd.height as u32,
                        );
                        if let Some(glow) = glow {
                            r.push_sprite_origin_rot(
                                [pos[0], pos[1] + sw.dy],
                                [dw, dh],
                                glow.uv,
                                [1.0, 1.0, 1.0, 1.0],
                                sw.rot,
                                sw.pivot,
                            );
                            stats.quads += 1;
                        }
                    }
                } else if matches!(
                    tile.kind,
                    236 | 238 | 702 | 485 | 490 | 652 | 521 | 522 | 523 | 524 | 525 | 526
                        | 527 | 530 | 651 | 705 | 489 | 493 | 233 | 519
                ) {
                    // W4-g84: the MultiTileGrass 2×2 / 3×2 family never draws
                    // through `DrawSingleTile` — C# registers the object's
                    // top-left as a MultiTileGrass special
                    // (`TileDrawing.cs:606-687`) and
                    // `DrawMultiTileGrass`/`DrawMultiTileGrassInWind`
                    // (`:8760-8795`, `:8916-9032`) draw it about the
                    // bottom-centre anchor. The row-major loop draws each
                    // cell inline with the same per-cell transform (the
                    // sunflower's documented z-order approximation);
                    // `windCycle = 0` (underground / calm) degenerates to
                    // the static draw. The 2×2 default-arm family
                    // (236/238/702 bulb + Life Fruit, 485 antlion larva,
                    // 490 weather vane, 652 echo detritus) takes `num = 0.15`;
                    // the dragonfly jars 521-527 freeze (`num = 0`,
                    // `flag = false`, `:8940-8949`); the 3×2 oasis/echo plants
                    // 530/651/705 use the wider anchor (`:8769-8774`); the
                    // 489 pinwheel is 2×3 (`:8788-8791`) and the 493 golf flag
                    // 1×2 with a wind-sign flip + ∓6 px shift (`:8792-8795`,
                    // `:8983-8997`); the 233 detritus splits 3×2 / 2×2 by frame
                    // row (`:8765-8768`) and the 519 cattail climbs its stem
                    // (`:8796-8800`, `ClimbCatTail :8807-8827`). Only 238
                    // animates a row (`:5441-5444`); 485/490/489/493's
                    // animation rides the landed `drawdata_cases_b` arms.
                    let (sw, flip_xor, dx) = if matches!(tile.kind, 530 | 651 | 705) {
                        (
                            crate::render::multi_tile_grass::grass_sway_3x2(
                                wind, world, tx, ty, tile.frame_x, tile.frame_y, dd.top,
                                sunflower_wind_counter,
                            ),
                            false,
                            0.0,
                        )
                    } else if tile.kind == 489 {
                        (
                            crate::render::multi_tile_grass::grass_sway_2x3(
                                wind, world, tx, ty, tile.frame_x, tile.frame_y, dd.top,
                                sunflower_wind_counter,
                            ),
                            false,
                            0.0,
                        )
                    } else if tile.kind == 493 {
                        crate::render::multi_tile_grass::golf_flag_sway(
                            wind, world, tx, ty, tile.frame_x, tile.frame_y, dd.top,
                            sunflower_wind_counter, dd.flip,
                        )
                    } else if tile.kind == 233 {
                        (
                            crate::render::multi_tile_grass::detritus_sway(
                                wind, world, tx, ty, tile.frame_x, tile.frame_y, dd.top,
                                sunflower_wind_counter,
                            ),
                            false,
                            0.0,
                        )
                    } else if tile.kind == 519 {
                        (
                            crate::render::multi_tile_grass::cattail_sway(
                                wind, world, tx, ty, dd.top, sunflower_wind_counter,
                            ),
                            false,
                            0.0,
                        )
                    } else {
                        let freeze = matches!(tile.kind, 521..=527);
                        (
                            crate::render::multi_tile_grass::grass_sway_2x2(
                                wind, world, tx, ty, tile.frame_x, tile.frame_y, dd.top,
                                sunflower_wind_counter, 0.15, freeze,
                            ),
                            false,
                            0.0,
                        )
                    };
                    let (dw, dh) = (dd.width as f32, dd.height as f32);
                    // `tileLight = Lighting.GetColor(i, j)` (`:8975`) — the
                    // special pass applies NO `GetFinalLight` actuation
                    // ghost; `DrawTiles_GetLightOverride` (`:8977`,
                    // `:6626-6629`) maps a Glow-coated cell to White. The
                    // vision-changer chain (`:8976`) is identity without
                    // the unported potions (the d56 lane).
                    let bulb_l = if tile.fullbright_block() {
                        [1.0; 3]
                    } else {
                        raw_l
                    };
                    let tint = style.shade(bulb_l, tx, ty, anim_time);
                    // `DrawNature(vector + (0, vector3.Y), …, tileLight,
                    // windCycle * num * num3, origin, …)` (`:9024`) — the
                    // painted texture (`GetTileDrawTexture`, `:9000`), the
                    // cell's `vector2 = (i*16, j*16 + tileTop)` (`:8982`);
                    // the 493 golf flag's ∓6 px shift rides `vector2.X`
                    // (`:8991-8995`) and its flip toggles `tileSpriteEffect`.
                    r.push_sprite_origin_rot_paint(
                        [pos[0] + dx, pos[1] + dd.top as f32 + sw.dy],
                        [dw, dh],
                        flip_uv(tf.uv, dd.flip ^ flip_xor),
                        [tint[0], tint[1], tint[2], 1.0],
                        sw.rot,
                        sw.pivot,
                        tile.paint(),
                    );
                    stats.quads += 1;
                } else if crate::render::multi_tile_vines::is_multi_tile_vine(tile.kind) {
                    // W4-g84 follow-up: the hanging-object bucket never draws
                    // through `DrawSingleTile` — C# registers the object's
                    // top-left as a MultiTileVine special
                    // (`TileDrawing.cs:549-599`) and
                    // `DrawMultiTileVines`/`DrawMultiTileVinesInWind`
                    // (`:8829-8888`, `:9238-9603`) draw it about the
                    // TOP-centre anchor with the platform lift, the style
                    // tables, the jar glow / lid / item-icon specials and the
                    // shared-seed flame tail. The row-major loop draws each
                    // cell inline with the same per-cell transform (the
                    // sunflower / bulb documented z-order approximation).
                    let (tlx, tly, sx, sy) = crate::render::multi_tile_vines::vine_top_left(
                        tile.kind, tx, ty, tile.frame_x, tile.frame_y,
                    );
                    let sw = crate::render::multi_tile_vines::vine_sway(
                        wind, world, tx, ty, tlx, tly, sx, sy, dd.top,
                        sunflower_wind_counter,
                    );
                    // `tileLight = Lighting.GetColor(i, j)` (`:9486`) — the
                    // special pass applies NO `GetFinalLight` actuation ghost;
                    // `DrawTiles_GetLightOverride` (`:9488`, `:6626-6629`)
                    // maps a Glow-coated cell to White. The vision-changer
                    // chain (`:9487`) is identity without the unported
                    // potions (the d56 lane).
                    let vine_l = if tile.fullbright_block() { [1.0; 3] } else { raw_l };
                    let tint = style.shade(vine_l, tx, ty, anim_time);
                    // The shared flame seed (`:9590-9594`): the object's
                    // first cell in the C#'s x-outer/y-inner walk is the
                    // top-left; the inline pass derives it from that cell.
                    let shared_seed = tr_core::flame::frame_seed_for(
                        tr_core::flame::tile_frame_seed_at(anim_time), tlx, tly,
                    );
                    stats.quads += crate::render::multi_tile_vines::draw_vine_cell(
                        r, lookups, atlas, world, tile, &dd, tf, &sw, tlx, tly, tx, ty,
                        tint, anim_time, params, shared_seed,
                    );
                } else if sway != 0.0 {
                    // `DrawGrass`'s swaying sprite: shifted by `(cycle, |cycle|)`
                    // and rotated `cycle * 0.1` about
                    // `(tileWidth / 2, 16 - tileTop)` (`TileDrawing.cs:8369-8374`).
                    let (dw, dh) = (dd.width as f32, dd.height as f32);
                    let sprite_pos = [pos[0] - (dw - 16.0) * 0.5, pos[1] + dd.top as f32];
                    let origin = [dw * 0.5, 16.0 - dd.top as f32];
                    let tint = style.shade(l, tx, ty, anim_time);
                    r.push_sprite_origin_rot(
                        [sprite_pos[0] + sway, sprite_pos[1] + sway.abs()],
                        [dw, dh],
                        flip_uv(tf.uv, dd.flip),
                        [tint[0], tint[1], tint[2], 1.0],
                        sway * 0.1,
                        origin,
                    );
                    stats.quads += 1;
                } else if dd.top != 0 || dd.width != 16 || dd.height != 16 || dd.flip {
                    // `x*16 - (tileWidth - 16) / 2`, `y*16 + tileTop`; horizontal flip
                    // swaps the source columns. The centring is skipped for the
                    // `DoNotAdjustDrawPositionBasedOnTileWidth` kinds (711,
                    // `TileDrawing.cs:1010-1015`).
                    let (dw, dh) = (dd.width as f32, dd.height as f32);
                    let num5 = if dd.no_width_adjust {
                        0.0
                    } else {
                        (dw - 16.0) * 0.5
                    };
                    push_lit(
                        r,
                        [pos[0] - num5, pos[1] + dd.top as f32],
                        [dw, dh],
                        flip_uv(tf.uv, dd.flip),
                        l,
                        style,
                        tx,
                        ty,
                        anim_time,
                        tile.paint(),
                        &dd,
                        disco_final,
                    );
                    stats.quads += 1;
                } else if tile.slope() != 0 && !td::SET_DONT_DRAW_TILE_SLOPES.has(tile.kind) {
                    // `DrawBasicTile`'s slope family (`TileDrawing.cs:1510-1584`).
                    // `flag2 = !DontDrawTileSlopes` (`:1510`): gated kinds skip
                    // every slope arm and fall through to the sliced/plain
                    // draws below, exactly like the C# `if` chain.
                    if td::SET_PLATFORMS.has(tile.kind) {
                        // The platform-slope arm (`TileDrawing.cs:1515-1545`):
                        // the FULL normal rect draws flat (`:1523`, never the
                        // column path), plus one behind-tiles side-fill quad
                        // when the cell below-left/right joins a solid block.
                        // The C# returns early (`:1544`) — no slope columns.
                        push_lit(
                            r,
                            pos,
                            [TILE_PX, TILE_PX],
                            flip_uv(flat_src_uv(&dd, tile, tf, atlas, lookups), dd.flip),
                            l,
                            style,
                            tx,
                            ty,
                            anim_time,
                            tile.paint(),
                            &dd,
                            None,
                        );
                        stats.quads += 1;
                        stats.quads += draw_platform_side_fill(
                            r, world, lookups, atlas, tile, &dd, pos, l, style, tx, ty,
                            anim_time, show_invisible_blocks,
                        );
                    } else if td::SET_HAS_SLOPE_FRAMES.has(tile.kind) {
                        // `HasSlopeFrames` (`TileDrawing.cs:1546-1550`): the
                        // kind's sheet carries pre-sloped frames — one flat
                        // 16x16 draw from `tileFrameX/Y + addFrX/addFrY`.
                        push_lit(
                            r,
                            pos,
                            [TILE_PX, TILE_PX],
                            flip_uv(tf.uv, dd.flip),
                            l,
                            style,
                            tx,
                            ty,
                            anim_time,
                            tile.paint(),
                            &dd,
                            None,
                        );
                        stats.quads += 1;
                    } else {
                        stats.quads +=
                            push_slope(r, pos, tf.uv, l, style, tx, ty, anim_time, tile.slope(), disco_final);
                    }
                } else if let Some(join) = half_brick_join(world, tile, tx, ty) {
                    // The half-brick neighbour joins (`TileDrawing.cs:1586-1629`):
                    // a full solid block beside half bricks rebuilds its top
                    // strip from the sheet's join columns (126/90/144/148/156).
                    stats.quads += push_half_brick_join(
                        r, lookups, atlas, tile, &dd, pos, l, style, tx, ty, anim_time, &join,
                    );
                } else if tile.half_brick() {
                    // The C#'s half-brick arm rebuilds its own rect from
                    // `tileFrameX/Y + addFrX/addFrY` (`TileDrawing.cs:1298`) —
                    // case 80's biome shift lands on `normalTileRect` only, so
                    // the half-brick draw never carries `src_y_shift`.
                    push_half(r, pos, tf.uv, l, style, tx, ty, anim_time, disco_final);
                    stats.quads += 1;
                } else if style.sliced_blocks
                    && solid
                    // W4-g80: `flag = !TileID.Sets.DontDrawTileSliced`
                    // (`TileDrawing.cs:1509`, the `:1630` sliced gate) —
                    // the set (TileID.cs:482) includes 160, so the
                    // RainbowBrick's disco `finalColor` always reaches the
                    // flat draw; the set's other kinds take the flat path
                    // exactly like the C#.
                    && !td::SET_DONT_DRAW_TILE_SLICED.has(tile.kind)
                    && raw_l[0].max(raw_l[1]).max(raw_l[2]) > MEDIUM_LIGHT
                {
                    stats.quads += push_sliced(
                        r,
                        pos,
                        tf.uv,
                        light,
                        style,
                        tx,
                        ty,
                        anim_time,
                        HIGH_LIGHT,
                        inactive,
                        tile.fullbright_block(),
                        tile.paint(),
                    );
                } else if sway != 0.0 {
                    // The plain-size sway members (e.g. the sunflower 113, whose
                    // `TileDrawData` is the default): the same `DrawGrass`
                    // transform with the 16x16 origin `(8, 16)`.
                    let tint = style.shade(l, tx, ty, anim_time);
                    r.push_sprite_origin_rot(
                        [pos[0] + sway, pos[1] + sway.abs()],
                        [TILE_PX, TILE_PX],
                        tf.uv,
                        [tint[0], tint[1], tint[2], 1.0],
                        sway * 0.1,
                        [8.0, 16.0],
                    );
                    stats.quads += 1;
                } else {
                    let flat_uv = flat_src_uv(&dd, tile, tf, atlas, lookups);
                    push_lit(
                        r,
                        pos,
                        [TILE_PX, TILE_PX],
                        flat_uv,
                        l,
                        style,
                        tx,
                        ty,
                        anim_time,
                        tile.paint(),
                        &dd,
                        disco_final,
                    );
                    stats.quads += 1;
                    // W4-g68: the inline leaf arm is reconciled with the
                    // faithful `EmitLivingTreeLeaf` body (the W4-d60 flagged
                    // deviation): no `sway_enabled` gate, the sideways
                    // `Next(2)` is UNCONDITIONAL, the below-spawn needs two
                    // open downwind cells in strong wind, the paint column
                    // rides the gore frame, and type 384 sheds 914.
                    // `TileDrawing.cs:6848-6855` + `:7549-7592`.
                    // W4-g68 wiring-3: the site-1 quarter-roll gate
                    // (`DrawSingleTile`, `:965-968`) — the C# wraps the
                    // WHOLE `DrawTiles_EmitParticles` call (the 192/384
                    // arms included) in `_isActiveAndNotPaused &&
                    // (!Lighting.UpdateEveryFrame || FastRandom(
                    // Main.TileFrameSeed).WithModifier(x,y).Next(4)==0)`.
                    // `UpdateEveryFrame` (`Lighting.cs:77-87`) is false
                    // whenever render targets are required (the normal
                    // path), so the quarter-roll normally APPLIES: the
                    // shed rate is frequency/4 × 1/4 per eligible draw,
                    // and the eligible set rotates on the 5-draw
                    // TileFrameSeed cycle (`Main.cs:61719-61723` — the
                    // port's `flame::tile_frame_seed_at` schedule). The
                    // roll is a THROWAWAY per-tile FastRandom: it does
                    // NOT step `leaf_rng`.
                    if emit_effects
                        && matches!(tile.kind, 192 | 384)
                        && tr_core::tile_light::FastRandom::new(
                            tr_core::flame::tile_frame_seed_at(anim_time),
                        )
                        .with_modifier(tx, ty)
                        .next(4)
                            == 0
                    {
                        let max_value =
                            tr_core::wind::leaf_frequency(wind.speed_current) / 4;
                        if leaf_rng.next_range(0, max_value) == 0 {
                            let leaf_type = if tile.kind == 192 { 910 } else { 914 };
                            emit_living_tree_leaf(
                                world,
                                tx,
                                ty,
                                leaf_type,
                                wind.speed_current,
                                &mut leaf_rng,
                                main_rand,
                                gore,
                            );
                        }
                    }
                }
                // `TileDrawing.DrawSingleTile` closes with `DrawSingleTile_Flames`
                // (`TileDrawing.cs:1723`): the flame/glow overlays draw right
                // after the tile sprite, in the same row-major order, with their
                // own self-lit colours (the light map does not tint them).
                // The MultiTileVine bucket draws its OWN flame tail + glow in
                // `draw_vine_cell` (`:9585-9600`) — C# never calls
                // `DrawSingleTile_Flames` for a special-pass tile, so the
                // inline overlay is skipped for those kinds (no double-draw).
                if !crate::render::multi_tile_vines::is_multi_tile_vine(tile.kind) {
                    stats.quads +=
                        draw_tile_flames(r, lookups, world, tile, &dd, tx, ty, params, anim_time);
                }
                // TILE-DRAW-2 glow bind (`TileDrawing.cs:1343-1403`): the
                // second switch's glow record draws AFTER `DrawBasicTile`
                // (whose tail is the flames hook, `:1723`) and BEFORE the
                // highlight (`:1404`). The platform arm reuses the C#
                // `vector` (`:1363`, built at `:1008-1013`: centring +
                // tileTop + halfBrickHeight), every other kind re-derives
                // the plain centred anchor (`:1360`).
                if let Some(g) = glow {
                    let num5 = if dd.no_width_adjust {
                        0.0
                    } else {
                        (dd.width as f32 - 16.0) * 0.5
                    };
                    let half = if tile.half_brick() { 8.0 } else { 0.0 };
                    let vector = [pos[0] - num5, pos[1] + dd.top as f32 + half];
                    stats.quads += crate::render::tile_glow::draw_tile_glow(
                        r,
                        lookups,
                        atlas,
                        world,
                        tile.kind,
                        tx,
                        ty,
                        &dd,
                        &g,
                        vector,
                        tile_anim.animation_row(412),
                        show_invisible_blocks,
                    );
                }
            }
        }
        if run_start >= 0 {
            scratch.cur.push((run_start, x1));
        }
        stats.dark_rects += merge_dark_row(r, white, scratch, ty);
    }
    // Emit rectangles that reach the bottom edge of the visible area.
    for o in scratch.open.drain(..) {
        r.push_sprite(
            [o[0] as f32 * TILE_PX, o[2] as f32 * TILE_PX],
            [
                (o[1] - o[0] + 1) as f32 * TILE_PX,
                (o[3] - o[2] + 1) as f32 * TILE_PX,
            ],
            white,
            [0.0, 0.0, 0.0, 1.0],
        );
        stats.dark_rects += 1;
    }

    // S1-G-13 consumers: the layer callbacks (solid/nonSolid halves, the
    // wall blacks) stop here — `PostDrawTiles(solidLayer: false)`
    // (`TileDrawing.cs:414-432`) draws the foliage/vines into the SCREEN
    // batch after the tile2 composite (`Main.cs:63358-63380`); the
    // direct-screen extras block (`TileFilter::ScreenExtras`) re-runs them
    // there. The tile2 callback's crawl parks its roots for that block
    // (`_vineRootsPositions` survives the callback, `:120`). The entity
    // tail (`TileFilter::Screen`) skips the foliage/vines and continues.
    if matches!(
        tile_filter,
        TileFilter::Solid | TileFilter::NonSolid | TileFilter::Wall
    ) {
        if tile_filter == TileFilter::NonSolid {
            scratch.vine_roots = std::mem::take(&mut vine_roots);
        }
        // GATE RNG fix: hand the advanced stream to the next gated call.
        if scratch.leaf_rng_shared {
            scratch.leaf_rng = Some(leaf_rng);
        }
        stats.instances = stats.quads;
        stats.draw_ms = t_total.elapsed().as_secs_f32() * 1000.0;
        return stats;
    }

    if tile_filter != TileFilter::Screen && !crate::env::no_foliage() {
        stats.foliage = draw_tree_foliage(
            r,
            world,
            lookups,
            light,
            style,
            anim_time,
            wind,
            tree_wind_counter,
            &mut leaf_rng,
            main_rand,
            gore,
            emit_effects,
            x0,
            y0,
            x1,
            y1,
        );
    }
    // `TileDrawing.PostDrawTiles`' `DrawTrees` → `DrawVines` →
    // `DrawReverseVines` order (`TileDrawing.cs:428-430`): the vine strips
    // draw right after the tree canopies (S1-G-1), then the rising seaweed
    // strips (W4-g83, `DrawReverseVines` `:8903-8914`). The multi-tile-vine
    // pass (`:421`) is the furniture lane (d57 / g31 drains).
    // `TileDrawing.PostDrawTiles`' `DrawTrees` → `DrawVines` order
    // (`TileDrawing.cs:428-429`): the vine strips draw right after the tree
    // canopies (S1-G-1). `DrawReverseVines` (`:430`) and the multi-tile-vine
    // pass (`:421`) are separate passes, still unported. S1-G-13 consumers:
    // the direct-screen extras block reads the roots the tile2 callback
    // parked.
    if tile_filter == TileFilter::ScreenExtras {
        vine_roots = std::mem::take(&mut scratch.vine_roots);
    }
    if !vine_roots.is_empty() {
        stats.quads += crate::render::vines::draw_vines(
            r,
            world,
            lookups,
            atlas,
            light,
            style,
            anim_time,
            wind,
            vine_wind_counter,
            &vine_roots,
            show_invisible_blocks,
        );
    }
    // W4-g31 attach phase (UNION-CONTRACT-tilepass §8): the TR_TILEPASS
    // SHADOW attach — the union's draw_layer/post_draw_tiles drivers run
    // at the C# PostDrawTiles position (after the fused walk + the inline
    // foliage/vine passes, before the NPC pass) over the same camera
    // rect, PLAN-ONLY: no sprites pushed, a separate _rand stream (the
    // fused pass's roll stream is never passed in), no shared state
    // mutated, the G6 key untouched. Default
    // OFF — unset the pin and zero union code runs here. The pixel
    // binding (the fused loop split into the two draw_layer calls) is
    // §8's merge script.
    if crate::env::tilepass_shadow() {
        let shadow_inputs = crate::render::tile_attach::ShadowInputs {
            camera,
            zoom,
            screen,
            // The fused pass's pinned-default convention (the
            // HIGH_LIGHT/MEDIUM_LIGHT constants at :425): the adaptive
            // App.gfx_quality.quality threads in at the pixel merge (§8).
            gfx_quality: 0.5,
            grass_counter: grass_wind_counter,
            sunflower_counter: sunflower_wind_counter,
            vine_counter: vine_wind_counter,
            anim_time,
            sparkle_seed,
            disco: params.disco,
        };
        let shadow = crate::render::tile_attach::run_shadow(
            world, wind, light, &shadow_inputs,
        );
        log::info!("TILEPASS shadow: {}", shadow.summary());
    }
    // `TileDrawing.PostDrawTiles` closes before the entity tail — the
    // direct-screen extras block stops here; the entity tail rides its own
    // `TileFilter::Screen` pass after the tile composite.
    if tile_filter == TileFilter::ScreenExtras {
        // GATE RNG fix: the extras block consumed the foliage rolls; the
        // entity tail (`Screen`) draws none, but keep the stream parked.
        if scratch.leaf_rng_shared {
            scratch.leaf_rng = Some(leaf_rng);
        }
        stats.instances = stats.quads;
        stats.draw_ms = t_total.elapsed().as_secs_f32() * 1000.0;
        return stats;
    }
    // Main.cs:22320,63305 — normal NPC pass (non-behind-tiles).
    if !reverse_vine_roots.is_empty() {
        stats.quads += crate::render::vines::draw_reverse_vines(
            r,
            world,
            lookups,
            atlas,
            light,
            style,
            anim_time,
            wind,
            vine_wind_counter,
            &reverse_vine_roots,
            // W4-g83 seam 4: the sight override threads the caller's
            // `show_invisible_blocks` (the g31 keep-one delta — the strip's
            // IsVisible read now matches the union's full form).
            show_invisible_blocks,
        );
    }
    // Main.cs:22320,63305 - normal NPC pass (non-behind-tiles). The party
    // hats ride it (`Main.DrawNPCExtras`, W4-c39): the colour hash reads the
    // moon phase + day flag, the strip rides the lookups.
    draw_phase("npcs");
    stats.quads += draw_npcs(
        r,
        npc_assets,
        npcs,
        light,
        style,
        anim_time,
        camera,
        zoom,
        screen,
        false,
        empress_ctx,
        Some((party_is_up, world.moon_phase, world.day_time, lookups)),
        latest_filter_strength,
    );
    draw_phase("player");
    if let Some(p) = player {
        // W4-d115 wiring round — `LegacyPlayerRenderer.cs:433-467`: the
        // dodge afterimages (the shadow pair + the brain ring) draw
        // BEFORE the main copy (`:468-475`). The ghost seed rides the
        // frame's sparkle pin (the `TrySparkling` convention — the two
        // `Next(-10,11)` alpha rolls replay per frame); the counter step
        // already ran in `App::draw_region` (the C# steps it inside this
        // same block, `:433-448`).
        stats.quads += crate::render::entities::draw_dodge_afterimages(
            r,
            player_assets,
            lookups,
            p,
            held,
            armor,
            light,
            style,
            anim_time,
            camera,
            zoom,
            screen,
            sparkle_seed ^ 0x2545_F491_4F0A_DDB7,
        );
        stats.quads += draw_player(
            r,
            player_assets,
            lookups,
            p,
            held,
            armor,
            light,
            style,
            anim_time,
            camera,
            zoom,
            screen,
        );
    }
    for rp in remote {
        // The `netOffset` accumulator (the C#'s `LegacyPlayerRenderer.cs:304`:
        // `drawPlayer.position + drawPlayer.netOffset`): the snapshot position
        // plus the decaying visual lag. In single-player/hosting the offsets
        // are zero, so the drawn pixels are unchanged.
        let mut p = Player::spawn(world);
        let rx = rp.x + rp.off_x;
        let ry = rp.y + rp.off_y;
        p.x = rx;
        p.y = ry;
        p.prev_x = rx;
        p.prev_y = ry;
        p.vx = rp.vx;
        p.vy = rp.vy;
        p.facing = rp.facing;
        p.on_ground = rp.on_ground;
        p.leg_frame = rp.leg_frame;
        p.body_frame = rp.body_frame;
        p.gfx_off_y = 0.0;
        stats.quads += draw_player(
            r,
            player_assets,
            lookups,
            &p,
            None,
            [0, 0, 0],
            light,
            style,
            anim_time,
            camera,
            zoom,
            screen,
        );
        let sx = (p.x + player::WIDTH * 0.5 - camera.x) * zoom + screen.x * 0.5;
        let sy = (p.y - 14.0 - camera.y) * zoom + screen.y * 0.5;
        let tw = hud_text_width(r, lookups, &rp.name, 0.8);
        hud_text(
            r,
            lookups,
            sx + 1.0 - tw * 0.5,
            sy + 1.0,
            0.8,
            [0.0, 0.0, 0.0, 1.0],
            &rp.name,
        );
        hud_text(
            r,
            lookups,
            sx - tw * 0.5,
            sy,
            0.8,
            [1.0, 1.0, 1.0, 1.0],
            &rp.name,
        );
    }
    // `Main.DrawItems` (`Main.cs:62278`) — the dropped items are drawn
    // AFTER the players and the NPCs (the C#'s entity section: the
    // projectiles → the players → the NPCs over the players → the items).
    draw_phase("items");
    for it in items {
        // W4-g15 lane 5 — the mana-crystal pulse reads `Main.mouseTextColor`
        // (`Main.cs:38724-38725`); it rides `params` (the same value the
        // torch/lava light pulses read, already in the G6 `light_params`).
        stats.items += it.draw(r, lookups, light, style, anim_time, params.mouse_text_color);
    }
    // `Main.DrawLiquid(bg: false, ...)` runs after entities: the foreground water
    // layer tints everything that stands in it. S1-G-13 consumers: the
    // waterTarget callback owns this pass under the gated path
    // (`Main.RenderWater` → `DrawWaters()`, `Main.cs:57371-57386`, composited
    // at `:62308`); the direct-screen block skips it.
    if tile_filter == TileFilter::All && !crate::env::no_liqfront() {
        stats.liquids += draw_waters(
            r,
            world,
            lookups,
            atlas,
            light,
            style,
            anim_time,
            liquid_frames,
            &liquid_draws,
            LiquidPass::Foreground,
            water_style,
            liquid_alpha,
        );
    }
    if let Some((tx, ty)) = edit_target {
        let white = r.white_uv();
        let pos = [tx as f32 * TILE_PX, ty as f32 * TILE_PX];
        let c = [1.0, 1.0, 1.0, 0.55];
        for (dx, dy, w, h) in [
            (0.0, 0.0, 16.0, 1.0),
            (0.0, 15.0, 16.0, 1.0),
            (0.0, 0.0, 1.0, 16.0),
            (15.0, 0.0, 1.0, 16.0),
        ] {
            r.push_sprite([pos[0] + dx, pos[1] + dy], [w, h], white, c);
        }
    }
    stats.instances = stats.quads
        + stats.walls
        + stats.dark_rects
        + stats.foliage
        + stats.liquids
        + stats.waterfalls
        + stats.items;
    stats.draw_ms = t_total.elapsed().as_secs_f32() * 1000.0;
    // GATE RNG fix: the tail exit parks the stream too (the fused default
    // never reads the slot — `leaf_rng_shared` is false there).
    if scratch.leaf_rng_shared {
        scratch.leaf_rng = Some(leaf_rng);
    }
    stats
}

/// `WallDrawing.DrawWalls` (`Main.DrawWalls`, the wall half of
/// `Main.RenderWallsAndBlacks`, `Main.cs:58056-58091`): the whole wall layer,
/// drawn before the tile layer. Each frame is a 32x32 sprite placed 8 px
/// up/left of the cell so neighbouring walls join without seams. S1-G-13
/// consumers: extracted verbatim from the fused `draw_tiles` loop so the
/// wallTarget callback drives the SAME pass; the fused default calls it at
/// the identical site (the draw order is unchanged).
#[allow(clippy::too_many_arguments)]
pub(crate) fn draw_wall_layer(
    r: &mut SpriteRenderer,
    world: &World,
    lookups: &mut assets::Lookups,
    atlas: Option<&Atlas>,
    light: &LightMap,
    style: &LightStyle,
    anim_time: f32,
    x0: i32,
    y0: i32,
    x1: i32,
    y1: i32,
    stats: &mut TileDrawStats,
) {
    if !crate::env::walls_enabled() {
        return;
    }
    let width = world.width as usize;
    for ty in y0..=y1 {
        for tx in x0..=x1 {
            let tile = &world.tiles[ty as usize * width + tx as usize];
            if !wall_visible(
                tile.wall,
                world.invisible_wall(tx, ty),
                crate::env::show_invisible(),
            ) {
                continue;
            }
            let Some(wf) = lookups.wall_tex(tile.wall, tile.wall_col(), tile.wall_row()) else {
                continue;
            };
            if tile.active()
                && tile_hides_wall(
                    lookups,
                    atlas,
                    tile.kind,
                    tile.frame_x,
                    tile.frame_y,
                    tile.slope(),
                    tile.half_brick(),
                )
            {
                stats.walls_culled += 1;
                continue;
            }
            // `WallDrawing.cs:91`: Echo-coated walls are hidden unless the
            // Echo Monolith or an Echo item grants sight (the port has
            // neither yet, so they stay hidden; wall 318 is handled above).
            if world.invisible_wall(tx, ty) {
                continue;
            }
            // `WallDrawing.DrawWalls` skips black cells; the tile pass merges the
            // fully covered ones into black rectangles.
            if light_is_zero(light.get(tx, ty)) {
                continue;
            }
            // The 32 px sprite is centred on the 16 px cell (`x*16 - 8`), like
            // `new Vector2(j * 16 - 8, i * 16 - 8)` in the original.
            stats.quads += push_wall_lit(
                r,
                [tx as f32 * TILE_PX - 8.0, ty as f32 * TILE_PX - 8.0],
                wf.uv,
                light,
                style,
                tx,
                ty,
                anim_time,
                world.wall_paint(tx, ty),
                world.fullbright_wall(tx, ty),
            );
            stats.walls += 1;
        }
    }
}

/// `WallDrawing.DrawWalls`: a 32x32 wall sprite lit by the cell's four corner colours
/// (`Lighting.GetCornerColors`). Flat-lit cells stay a single quad.
#[allow(clippy::too_many_arguments)]
fn push_wall_lit(
    r: &mut SpriteRenderer,
    pos: [f32; 2],
    uv: [f32; 4],
    light: &LightMap,
    style: &LightStyle,
    tx: i32,
    ty: i32,
    time: f32,
    paint: u8,
    fullbright: bool,
) -> u32 {
    let mut corners = light.corner4(tx, ty);
    if fullbright {
        // `WallDrawing.cs:96-99`: a Glow-coated wall samples white light
        // (`vertices = WallDrawing._glowPaintColors`).
        corners = [[1.0; 3]; 4];
    }
    let mut lo = f32::MAX;
    let mut hi = f32::MIN;
    for c in &corners {
        for ch in c {
            lo = lo.min(*ch);
            hi = hi.max(*ch);
        }
    }
    let tint = |c: [f32; 3]| {
        let c = style.shade(c, tx, ty, time);
        [c[0].min(1.0), c[1].min(1.0), c[2].min(1.0), 1.0]
    };
    if hi - lo < 1.5 / 255.0 {
        r.push_sprite_paint(pos, [32.0, 32.0], uv, tint(corners[0]), 0.0, paint);
        return 1;
    }
    let mut quads = 0;
    for (i, u) in [0.25f32, 0.75].iter().enumerate() {
        for (j, v) in [0.25f32, 0.75].iter().enumerate() {
            let x = 16.0 * i as f32;
            let y = 16.0 * j as f32;
            r.push_sprite_paint(
                [pos[0] + x, pos[1] + y],
                [16.0, 16.0],
                sub_uv_frac(uv, x, y, 16.0, 16.0, 32.0, 32.0),
                tint(bilerp_corners(corners, *u, *v)),
                0.0,
                paint,
            );
            quads += 1;
        }
    }
    quads
}

/// `WallDrawing.FullTile`: a full, opaque, unsloped block hides its wall completely.
fn tile_hides_wall(
    lookups: &mut assets::Lookups,
    atlas: Option<&Atlas>,
    kind: u16,
    frame_x: i16,
    frame_y: i16,
    slope: u8,
    half_brick: bool,
) -> bool {
    // `WorldGen.cs:15715`: leaf blocks are non-solid after generation, so they
    // never `FullTile` — the wall draws behind them.
    if tree::is_tree(kind) || kind == 192 || slope != 0 || half_brick || frame_x < 0 || frame_y < 0
    {
        return false;
    }
    let Some(atlas) = atlas else {
        return false;
    };
    match lookups.frame_tex(atlas, kind, frame_x, frame_y) {
        Some(f) => f.opaque,
        None => lookups.plain_tex(kind).is_some_and(|f| f.opaque),
    }
}

/// `WorldGen.SolidTile`: active, solid, not a platform, and a full block.
#[inline]
pub(crate) fn solid_full(world: &World, x: i32, y: i32) -> bool {
    world.tile(x, y).is_some_and(|t| {
        t.active()
            && td::TILE_SOLID.has(t.kind)
            && !td::TILE_SOLID_TOP.has(t.kind)
            && !t.half_brick()
            && t.slope() == 0
    })
}

/// Bilinear sample of `LightMap::corner4` (TL, TR, BL, BR).
#[inline]
pub(crate) fn bilerp_corners(c: [[f32; 3]; 4], u: f32, v: f32) -> [f32; 3] {
    let mut out = [0.0f32; 3];
    for k in 0..3 {
        let top = c[0][k] + (c[1][k] - c[0][k]) * u;
        let bottom = c[2][k] + (c[3][k] - c[2][k]) * u;
        out[k] = top + (bottom - top) * v;
    }
    out
}

/// Tint and push one flat-lit sprite.
#[allow(clippy::too_many_arguments)]
#[inline]
/// The plain flat arm's source rect: C#'s `normalTileRect`, the ONE rectangle
/// `TileDrawing.DrawSingleTile`'s switch mutates (`TileDrawing.cs:1007`) and
/// which only `DrawBasicTile`'s plain arm draws (`:1523`). That is why case
/// 80's biome-cactus row shift (`src_y_shift`, `:1132-1147`) applies here and
/// to no other arm — the half-brick, slice, slope and edge arms rebuild their
/// rects from `tileFrameX/Y + addFrX/addFrY` and stay unshifted.
fn flat_src_uv(
    dd: &tr_core::drawdata::TileDrawData,
    tile: &tr_core::tile::Tile,
    tf: assets::FrameTex,
    atlas: Option<&Atlas>,
    lookups: &assets::Lookups,
) -> [f32; 4] {
    if dd.src_y_shift == 0 {
        return tf.uv;
    }
    atlas
        .and_then(|a| {
            lookups.rect_uv(
                a,
                tile.kind,
                dd.frame_x_override.unwrap_or(tile.frame_x) + dd.add_fr_x,
                dd.frame_y_override.unwrap_or(tile.frame_y) + dd.add_fr_y + dd.src_y_shift,
                dd.width.max(16) as u32,
                dd.height.max(16) as u32,
            )
        })
        .unwrap_or(tf)
        .uv
}

/// W4-g80: the `case 160/692` final colour (`TileDrawing.cs:1078-1088`):
/// `Color color = new Color(Main.DiscoR, Main.DiscoG, Main.DiscoB, 255)`
/// (the 3-byte ctor sets alpha 255), then `if (tileCache.inActive()) color =
/// tileCache.actColor(color)` — the `Color` overload's double 0.4 truncating
/// multiply (`Tile.cs:272-280`), alpha preserved. Returned as the instance
/// tint (the WGSL multiplies the sRGB bytes in gamma space, S1-G-4).
fn disco_final_color(disco: [u8; 3], inactive: bool) -> [f32; 4] {
    let mut c = [disco[0], disco[1], disco[2], 255u8];
    if inactive {
        c = tr_core::drawdata::act_color_bytes(c);
    }
    [
        c[0] as f32 / 255.0,
        c[1] as f32 / 255.0,
        c[2] as f32 / 255.0,
        c[3] as f32 / 255.0,
    ]
}

#[allow(clippy::too_many_arguments)]
fn push_lit(
    r: &mut SpriteRenderer,
    pos: [f32; 2],
    size: [f32; 2],
    uv: [f32; 4],
    light: [f32; 3],
    style: &LightStyle,
    tx: i32,
    ty: i32,
    time: f32,
    paint: u8,
    dd: &tr_core::drawdata::TileDrawData,
    final_override: Option<[f32; 4]>,
) {
    // `finalColor = tileLight * 0.5f` for the cobwebs (`TileDrawing.cs:1078`)
    // and the shard alpha (`:1091`) ride the shaded light. W4-g80: the
    // 160/692 disco arm's `finalColor` override (`:1078-1088`) replaces the
    // whole colour — the C# draws `drawData.finalColor` verbatim, no light
    // scale or shard alpha rides on it (those arms are other types').
    if let Some(c) = final_override {
        r.push_sprite_paint(pos, size, uv, c, 0.0, paint);
        return;
    }
    let c = style.shade(light, tx, ty, time);
    let k = dd.light_scale;
    r.push_sprite_paint(
        pos,
        size,
        uv,
        [c[0] * k, c[1] * k, c[2] * k, dd.src_alpha],
        0.0,
        paint,
    );
}

/// Sub-rectangle of a 16x16 frame's UV rect, in tile pixels.
#[inline]
fn sub_uv(uv: [f32; 4], x: f32, y: f32, w: f32, h: f32) -> [f32; 4] {
    sub_uv_frac(uv, x, y, w, h, 16.0, 16.0)
}

/// `sub_uv` over a destination rect that is not 16x16.
#[inline]
pub(crate) fn sub_uv_frac(
    uv: [f32; 4],
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    full_w: f32,
    full_h: f32,
) -> [f32; 4] {
    let du = (uv[2] - uv[0]) / full_w;
    let dv = (uv[3] - uv[1]) / full_h;
    [
        uv[0] + du * x,
        uv[1] + dv * y,
        uv[0] + du * (x + w),
        uv[1] + dv * (y + h),
    ]
}

/// `DrawBasicTile` slope path: eight 2 px columns plus a 2 px strip, sampling the
/// tile's own 16x16 frame. Slopes 1/2 are ground slopes, 3/4 ceiling slopes.
/// Every quad draws `drawData.finalColor` (`TileDrawing.cs:1580-1583`), so the
/// W4-g80 disco override replaces the shaded light on all of them.
#[allow(clippy::too_many_arguments)]
fn push_slope(
    r: &mut SpriteRenderer,
    pos: [f32; 2],
    uv: [f32; 4],
    light: [f32; 3],
    style: &LightStyle,
    tx: i32,
    ty: i32,
    time: f32,
    slope: u8,
    final_override: Option<[f32; 4]>,
) -> u32 {
    let color = match final_override {
        Some(c) => c,
        None => {
            let c = style.shade(light, tx, ty, time);
            [c[0], c[1], c[2], 1.0]
        }
    };
    let mut quads = 0;
    for i in 0..8u8 {
        let f = i as f32 * 2.0;
        let (dx, dy, sx, sy, w, h) = match slope {
            1 => (f, f, f, 0.0, 2.0, 14.0 - f),
            2 => (14.0 - f, f, 14.0 - f, 0.0, 2.0, 14.0 - f),
            3 => (f, 0.0, f, f, 2.0, 16.0 - f),
            _ => (14.0 - f, 0.0, 14.0 - f, f, 2.0, 16.0 - f),
        };
        if h <= 0.0 {
            continue;
        }
        r.push_sprite(
            [pos[0] + dx, pos[1] + dy],
            [w, h],
            sub_uv(uv, sx, sy, w, h),
            color,
        );
        quads += 1;
    }
    let strip_y = if slope <= 2 { 14.0 } else { 0.0 };
    r.push_sprite(
        [pos[0], pos[1] + strip_y],
        [16.0, 2.0],
        sub_uv(uv, 0.0, strip_y, 16.0, 2.0),
        color,
    );
    quads + 1
}

/// Half bricks keep the top half of the art and sit on the bottom 8 px of the cell.
/// The draw colour is `drawData.finalColor` (`TileDrawing.cs:1643-1644`), so the
/// W4-g80 disco override rides here too.
#[allow(clippy::too_many_arguments)]
fn push_half(
    r: &mut SpriteRenderer,
    pos: [f32; 2],
    uv: [f32; 4],
    light: [f32; 3],
    style: &LightStyle,
    tx: i32,
    ty: i32,
    time: f32,
    final_override: Option<[f32; 4]>,
) {
    let color = match final_override {
        Some(c) => c,
        None => {
            let c = style.shade(light, tx, ty, time);
            [c[0], c[1], c[2], 1.0]
        }
    };
    r.push_sprite(
        [pos[0], pos[1] + 8.0],
        [16.0, 8.0],
        sub_uv(uv, 0.0, 0.0, 16.0, 8.0),
        color,
    );
}

/// The platform-slope side-fill draw hook (`DrawBasicTile`,
/// `TileDrawing.cs:1515-1545`): the cell's normal rect already drew flat
/// (`:1523`); this pushes the ONE behind-tiles fill quad one cell below when
/// the below-left/right neighbour joins a solid block. The neighbour facts
/// carry the real Echo bit (`FillNeighbor::from_world`); the source rect's Y
/// is `drawData.tileFrameY` (`:1527`/`:1537`) and the sheet is the platform's
/// own painted `drawTexture` (`:1532`/`:1542`). The tint is
/// `drawData.finalColor` — the shaded light with `light_scale`/`src_alpha`,
/// exactly the `push_lit` colour. The behind-tiles LAYER decision rides
/// `PlatformSideFill.behind_tiles` until S1-G-13 lands (the quad draws into
/// the world range meanwhile); the glow arm's twin fill (`:1367-1401`) rides
/// the still-unwired glow pass (TILE-DRAW-2). Returns the quad count (0/1).
#[allow(clippy::too_many_arguments)]
fn draw_platform_side_fill(
    r: &mut SpriteRenderer,
    world: &World,
    lookups: &assets::Lookups,
    atlas: Option<&Atlas>,
    tile: &tr_core::tile::Tile,
    dd: &tr_core::drawdata::TileDrawData,
    pos: [f32; 2],
    light: [f32; 3],
    style: &LightStyle,
    tx: i32,
    ty: i32,
    time: f32,
    show_invisible_blocks: bool,
) -> u32 {
    use crate::render::special_tiles::{
        platform_side_fill, push_platform_side_fill, FillNeighbor,
    };
    // `Main.tile[tileX, tileY+1]` / `[tileX-1, tileY+1]` / `[tileX+1, tileY+1]`
    // (`TileDrawing.cs:1519-1521`).
    let below = FillNeighbor::from_world(world, tx, ty + 1);
    let left_below = FillNeighbor::from_world(world, tx - 1, ty + 1);
    let right_below = FillNeighbor::from_world(world, tx + 1, ty + 1);
    let frame_y = dd.frame_y_override.unwrap_or(tile.frame_y);
    let Some(fill) = platform_side_fill(
        tile.slope(),
        frame_y,
        &below,
        &left_below,
        &right_below,
        show_invisible_blocks,
    ) else {
        return 0;
    };
    let c = style.shade(light, tx, ty, time);
    let k = dd.light_scale;
    let color = [c[0] * k, c[1] * k, c[2] * k, dd.src_alpha];
    let sheet_uv = |sx: f32, sy: f32, sw: f32, sh: f32| {
        atlas
            .and_then(|a| {
                lookups.rect_uv(a, tile.kind, sx as i16, sy as i16, sw as u32, sh as u32)
            })
            .map(|t| t.uv)
    };
    push_platform_side_fill(r, sheet_uv, pos, &fill, color, dd.flip, tile.paint())
}

/// The half-brick neighbour-join decision (`TileDrawing.cs:1586`): a FULL
/// solid block with a half-brick neighbour on either side rebuilds its top
/// strip from the sheet's join columns. `None` when any gate fails — the
/// `flag2` (`!DontDrawTileSlopes`), `!Platforms`,
/// `!IgnoresNearbyHalfbricksWhenDrawn`, `tileSolid`, `!NotReallySolid` and
/// `!halfBrick` chain, then the left/right half-brick probe (the C# reads the
/// neighbours' `halfBrick()` bit WITHOUT an active test).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct HalfBrickJoin {
    left: bool,
    right: bool,
    /// `AllBlocksWithSmoothBordersToResolveHalfBlockIssue` (`:1601`/`:1616`):
    /// the corner strip shrinks from 4 px to 2 px.
    smooth: bool,
    /// The both-sides arm's column pick (`:1592-1595`): an active same-kind
    /// tile above that is NOT a bottom slope swaps join column 126 → 90.
    same_above: bool,
}

fn half_brick_join(
    world: &World,
    tile: &tr_core::tile::Tile,
    tx: i32,
    ty: i32,
) -> Option<HalfBrickJoin> {
    let kind = tile.kind;
    // TileDrawing.cs:1586 — the gate chain, in the C# order.
    if td::SET_DONT_DRAW_TILE_SLOPES.has(kind)
        || td::SET_PLATFORMS.has(kind)
        || td::SET_IGNORES_NEARBY_HALFBRICKS_WHEN_DRAWN.has(kind)
        || !td::TILE_SOLID.has(kind)
        || td::SET_NOT_REALLY_SOLID.has(kind)
        || tile.half_brick()
    {
        return None;
    }
    let left = world.tile(tx - 1, ty).is_some_and(|t| t.half_brick());
    let right = world.tile(tx + 1, ty).is_some_and(|t| t.half_brick());
    if !left && !right {
        return None;
    }
    Some(HalfBrickJoin {
        left,
        right,
        smooth: td::SET_ALL_BLOCKS_WITH_SMOOTH_BORDERS_TO_RESOLVE_HALF_BLOCK_ISSUE.has(kind),
        same_above: world
            .tile(tx, ty - 1)
            .is_some_and(|t| t.active() && !t.bottom_slope() && t.kind == kind),
    })
}

/// The half-brick join's draw rects (`TileDrawing.cs:1588-1627`), as
/// `(dx, dy, srcX, srcY, w, h)` in draw order: the bottom 8 px of the cell's
/// own frame, then the join-column top strip(s). Every rect rides
/// `tileFrameX/Y + addFrX/addFrY` EXCEPT the right arm's corner strips,
/// which read the sheet's fixed (144+16-n, 0) / (156, 0) columns with NO
/// addFr (`:1622`/`:1625`).
fn half_brick_join_rects(
    tile: &tr_core::tile::Tile,
    dd: &tr_core::drawdata::TileDrawData,
    join: &HalfBrickJoin,
) -> Vec<(f32, f32, i32, i32, i32, i32)> {
    let fx = dd.frame_x_override.unwrap_or(tile.frame_x) as i32;
    let fy = dd.frame_y_override.unwrap_or(tile.frame_y) as i32;
    let (afx, afy) = (dd.add_fr_x as i32, dd.add_fr_y as i32);
    let (tw, th) = (dd.width as i32, dd.height as i32);
    let mut rects = Vec::with_capacity(4);
    // The bottom half of the cell's own frame (`:1590`/`:1605`/`:1620`).
    rects.push((0.0, 8.0, fx + afx, fy + afy + 8, tw, 8));
    if join.left && join.right {
        // `:1591-1596` — the 16x8 top strip from join column 126 (90 under a
        // same-kind non-bottom-slope block above); the row is addFrY only.
        let col = if join.same_above { 90 } else { 126 };
        rects.push((0.0, 0.0, col + afx, afy, 16, 8));
    } else if join.left {
        // `:1598-1611` — the left neighbour's half brick: the body shifts
        // right by n, the corner strip fills the top-left n x 8.
        let n = if join.smooth { 2 } else { 4 };
        rects.push((n as f32, 0.0, fx + n + afx, afy + fy, tw - n, th));
        rects.push((0.0, 0.0, 144 + afx, afy, n, 8));
        if n == 2 {
            rects.push((0.0, 0.0, 148 + afx, afy, 2, 2));
        }
    } else {
        // `:1613-1626` — the mirror arm: the body keeps its left edge and
        // loses n px on the right; the corner strips read FIXED sheet
        // columns (no addFrX/addFrY).
        let n = if join.smooth { 2 } else { 4 };
        rects.push((0.0, 0.0, fx + afx, afy + fy, tw - n, th));
        rects.push(((16 - n) as f32, 0.0, 144 + (16 - n), 0, n, 8));
        if n == 2 {
            rects.push((14.0, 0.0, 156, 0, 2, 2));
        }
    }
    rects
}

/// Push the half-brick join rects: all draws carry `finalColor` (the shaded
/// light with `light_scale`/`src_alpha`), the painted `drawTexture` and
/// `tileSpriteEffect`. Returns the quad count.
#[allow(clippy::too_many_arguments)]
fn push_half_brick_join(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    atlas: Option<&Atlas>,
    tile: &tr_core::tile::Tile,
    dd: &tr_core::drawdata::TileDrawData,
    pos: [f32; 2],
    light: [f32; 3],
    style: &LightStyle,
    tx: i32,
    ty: i32,
    time: f32,
    join: &HalfBrickJoin,
) -> u32 {
    let Some(a) = atlas else {
        return 0;
    };
    let c = style.shade(light, tx, ty, time);
    let k = dd.light_scale;
    let color = [c[0] * k, c[1] * k, c[2] * k, dd.src_alpha];
    let paint = tile.paint();
    let mut quads = 0u32;
    for (dx, dy, sx, sy, w, h) in half_brick_join_rects(tile, dd, join) {
        if w <= 0 || h <= 0 {
            continue;
        }
        let Some(t) =
            lookups.rect_uv(a, tile.kind, sx as i16, sy as i16, w as u32, h as u32)
        else {
            continue;
        };
        r.push_sprite_paint(
            [pos[0] + dx, pos[1] + dy],
            [w as f32, h as f32],
            flip_uv(t.uv, dd.flip),
            color,
            0.0,
            paint,
        );
        quads += 1;
    }
    quads
}

/// `DrawSingleTile_SlicedBlock`'s per-slice light (`TileDrawing.cs:3578-3581`,
/// `:3634-3637`): the slice averages the neighbourhood colour with the tile's
/// own RAW light, then `GetFinalLight` (`:4391-4402`) applies the tint and the
/// actuation ghost (`Tile.actColor`, `Tile.cs:282-288`) to that average.
/// `Main.shine` (`:4400`) is unported (tileShine2); the tint is `Color.White`
/// everywhere but the minecart path (`:3788`), so it is not threaded here.
#[inline]
fn sliced_light(slice: [f32; 3], own: [f32; 3], ghost: bool) -> [f32; 3] {
    let avg = [
        (slice[0] + own[0]) * 0.5,
        (slice[1] + own[1]) * 0.5,
        (slice[2] + own[2]) * 0.5,
    ];
    if ghost {
        [avg[0] * 0.4, avg[1] * 0.4, avg[2] * 0.4]
    } else {
        avg
    }
}

/// Bug 19: the half-brick arm with `DrawSingleTile`'s local rectangle Y shift
/// (`TileDrawing.cs:1298` draws `rectangle`, which case 80 mutated) — the
/// source row moves by `src_y`, the draw box stays the top 8 rows.
/// `DrawSingleTile_SlicedBlock`: the block becomes a 4/8/4 grid whose outer slices
/// take light from the adjacent tile, smoothing the seams between blocks.
#[allow(clippy::too_many_arguments)]
fn push_sliced(
    r: &mut SpriteRenderer,
    pos: [f32; 2],
    uv: [f32; 4],
    light: &LightMap,
    style: &LightStyle,
    tx: i32,
    ty: i32,
    time: f32,
    high_threshold: f32,
    ghost: bool,
    fullbright: bool,
    paint: u8,
) -> u32 {
    const SLICE_POS: [(f32, f32, f32, f32); 9] = [
        (0.0, 0.0, 4.0, 4.0),
        (4.0, 0.0, 8.0, 4.0),
        (12.0, 0.0, 4.0, 4.0),
        (0.0, 4.0, 4.0, 8.0),
        (4.0, 4.0, 8.0, 8.0),
        (12.0, 4.0, 4.0, 8.0),
        (0.0, 12.0, 4.0, 4.0),
        (4.0, 12.0, 8.0, 4.0),
        (12.0, 12.0, 4.0, 4.0),
    ];
    // `TileDrawing.cs:3528`: `vector = drawData.tileLight.ToVector3()` — the
    // RAW light (`Lighting.GetColor`, `:871`). `:1019`'s `GetFinalLight`
    // assigns `finalColor` and never touches `tileLight`, so the ghost must
    // not be applied here twice.
    let own = light.get(tx, ty);
    let mut quads = 0;
    if own[0].max(own[1]).max(own[2]) > high_threshold {
        let mut slices = light.slice9(tx, ty);
        // `TileDrawing.cs:3530-3533`: a Glow-coated block replaces every slice
        // with `_glowPaintColorSlices` (`:160-...`, all `Vector3.One`).
        if fullbright {
            slices = [[1.0; 3]; 9];
        }
        for (i, &(x, y, w, h)) in SLICE_POS.iter().enumerate() {
            let c = style.shade(sliced_light(slices[i], own, ghost), tx, ty, time);
            r.push_sprite_paint(
                [pos[0] + x, pos[1] + y],
                [w, h],
                sub_uv(uv, x, y, w, h),
                [c[0], c[1], c[2], 1.0],
                0.0,
                paint,
            );
            quads += 1;
        }
    } else {
        let mut slices = light.quad4(tx, ty);
        const QUAD_POS: [(f32, f32); 4] = [(0.0, 0.0), (8.0, 0.0), (0.0, 8.0), (8.0, 8.0)];
        // `TileDrawing.cs:3611-3614`: the same Glow slice replacement.
        if fullbright {
            slices = [[1.0; 3]; 4];
        }
        for (i, &(x, y)) in QUAD_POS.iter().enumerate() {
            let c = style.shade(sliced_light(slices[i], own, ghost), tx, ty, time);
            r.push_sprite_paint(
                [pos[0] + x, pos[1] + y],
                [8.0, 8.0],
                sub_uv(uv, x, y, 8.0, 8.0),
                [c[0], c[1], c[2], 1.0],
                0.0,
                paint,
            );
            quads += 1;
        }
    }
    quads
}

/// XNA's `SpriteBatch` blends premultiplied (`TileBatch.Begin` passes a null
/// blend state → `BlendState.AlphaBlend`, `TileBatch.cs:217`): with a tint
/// alpha of 0 the flame colour is added to the destination in full
/// (`out.rgb = tex.rgb * tint + dst`). The renderer's single alpha blend
/// cannot express `src = One`, so alpha-zero tints are pushed with alpha 1 —
/// on the flame art's opaque texels the two blend equations agree exactly,
/// and the transparent texels are discarded in both. The partial-alpha tints
/// (the furniture/jar colours with `a > 0`) keep their alpha, which dims the
/// flame slightly against vanilla's full add.
#[inline]
/// `DrawSingleTile_Flames` (`TileDrawing.cs:2684-3491`): the tile-flame
/// spread (`Main.tileFlame` types — torches, candles, chandeliers...) and the
/// per-type glow overlays, drawn right after the tile sprite with self-lit
/// colours. The glow branches precede the flame block in the C# `if` chain,
/// but no tile type matches both, so the two vecs can push in either order.
#[allow(clippy::too_many_arguments)]
fn draw_tile_flames(
    r: &mut SpriteRenderer,
    lookups: &mut assets::Lookups,
    world: &World,
    tile: &tr_core::Tile,
    dd: &tr_core::drawdata::TileDrawData,
    tx: i32,
    ty: i32,
    params: &LightParams,
    anim_time: f32,
) -> u32 {
    use tr_core::flame::{self, FlameEnv, GlowSheet};
    let kind = tile.kind;
    if !flame::tile_flame(kind) && !flame::has_glow_branch(kind) {
        return 0;
    }
    let env = FlameEnv {
        tile_frame_seed: flame::tile_frame_seed_at(anim_time),
        global_time: params.global_time,
        disco: params.disco,
        mouse_text_color: params.mouse_text_color,
        moon_phase: world.moon_phase,
        sundial_cooldown: params.sundial_cooldown,
        moondial_cooldown: params.moondial_cooldown,
    };
    let anchor = [
        tx as f32 * TILE_PX - (dd.width as f32 - 16.0) * 0.5,
        ty as f32 * TILE_PX + dd.top as f32,
    ];
    let mut quads = 0u32;

    // The glow branches (`TileDrawing.cs:2684-2825`, `:3463-3491`).
    for g in flame::glow_quads(kind, tx, ty, tile.frame_x, tile.frame_y, dd, &env) {
        let (uv, size) = match g.sheet {
            GlowSheet::Flames(sheet) => (
                lookups.flame_uv(
                    sheet,
                    g.src[0],
                    g.src[1],
                    g.src[2].max(1) as u32,
                    g.src[3].max(1) as u32,
                ),
                [g.src[2] as f32, g.src[3] as f32],
            ),
            GlowSheet::GlowMask(id) if g.src[2] == 0 => {
                // `value3.Frame()`: the whole sheet at its own pixel size.
                let name = format!("Glow_{id}");
                match lookups.glow_frame_uv(&name) {
                    Some((uv, w, h)) => (Some(uv), [w, h]),
                    None => (None, [0.0; 2]),
                }
            }
            GlowSheet::GlowMask(id) => (
                lookups.glow_uv(
                    &format!("Glow_{id}"),
                    g.src[0],
                    g.src[1],
                    g.src[2] as u32,
                    g.src[3] as u32,
                ),
                [g.src[2] as f32, g.src[3] as f32],
            ),
            GlowSheet::Named(name) if g.src[2] == 0 => match lookups.glow_frame_uv(name) {
                Some((uv, w, h)) => (Some(uv), [w, h]),
                None => (None, [0.0; 2]),
            },
            GlowSheet::Named(name) => (
                lookups.glow_uv(name, g.src[0], g.src[1], g.src[2] as u32, g.src[3] as u32),
                [g.src[2] as f32, g.src[3] as f32],
            ),
        };
        let Some(uv) = uv else { continue };
        r.push_sprite([anchor[0] + g.dx, anchor[1] + g.dy], size, uv, g.color);
        quads += 1;
    }

    // The `Main.tileFlame` spread (`TileDrawing.cs:2826-3462`).
    if flame::tile_flame(kind) {
        let sheet = flame::flame_sheet(kind);
        for q in flame::flame_quads(
            kind,
            tx,
            ty,
            tile.frame_x,
            tile.frame_y,
            dd.add_fr_x,
            dd.add_fr_y,
            env.tile_frame_seed,
            env.global_time,
            env.disco,
        ) {
            let Some(uv) = lookups.flame_uv(
                sheet,
                q.src_x as i32,
                q.src_y as i32,
                dd.width as u32,
                dd.height as u32,
            ) else {
                continue;
            };
            r.push_sprite(
                [anchor[0] + q.dx, anchor[1] + q.dy],
                [dd.width as f32, dd.height as f32],
                uv,
                q.color,
            );
            quads += 1;
        }
    }
    quads
}

#[cfg(test)]
mod tests {
    use super::*;
    use tr_core::wind::Wind;

    /// W4-g68 wiring-3: the site-1 leaf cadence
    /// (`TileDrawing.cs:965-968` → `:6848-6855`). The C# gates the WHOLE
    /// `DrawTiles_EmitParticles` call on the THROWAWAY
    /// `FastRandom(Main.TileFrameSeed).WithModifier(x,y).Next(4)==0`
    /// quarter-roll (the normal render-target path —
    /// `Lighting.UpdateEveryFrame` is false, `Lighting.cs:77-87`), THEN
    /// the 192/384 arms roll `_rand.Next(_leafFrequency/4)`. The port's
    /// inline arm must carry BOTH gates, the quarter-roll FIRST, and the
    /// throwaway roll must not step `leaf_rng`.
    #[test]
    fn w4_g68_the_leaf_arm_carries_the_site1_quarter_roll() {
        let src = include_str!("tiles.rs");
        let body = &src[src.find("pub(crate) fn draw_tiles(").unwrap()..];
        let body = &body[..body.find("#[cfg(test)]").unwrap()];
        let arm = &body[body.find("matches!(tile.kind, 192 | 384)").unwrap()..];
        let arm = &arm[..arm.find("draw_tile_flames").unwrap()];
        let quarter = arm
            .find("tile_frame_seed_at(anim_time)")
            .expect("the throwaway TileFrameSeed quarter-roll");
        let with_mod = arm.find(".with_modifier(tx, ty)").expect("WithModifier(x,y)");
        let next4 = arm.find(".next(4)").expect("Next(4)");
        let freq = arm
            .find("leaf_frequency(wind.speed_current) / 4")
            .expect("the frequency/4 gate");
        assert!(
            quarter < freq && with_mod < freq && next4 < freq,
            "the quarter-roll precedes the frequency gate (the C# call-site wrap)"
        );
        // The throwaway is a fresh FastRandom — never `leaf_rng` (the
        // `_rand` stand-in): the C# roll does not step the gate stream.
        let roll = &arm[arm.find("FastRandom::new(").unwrap()..next4];
        assert!(
            !roll.contains("leaf_rng"),
            "the quarter-roll is THROWAWAY — leaf_rng steps only on the frequency gate"
        );
        // The rate census: frequency/4 × 1/4 — at dead calm the C# shed
        // chance per eligible tile-draw is 1/(14000/4) × 1/4 = 1/14000.
        assert_eq!(tr_core::wind::leaf_frequency(0.0) / 4, 3500);
        // The 5-draw seed cycle: the port's schedule advances the seed 12
        // times per second (60 fps / 5), the flame pass's convention.
        let s0 = tr_core::flame::tile_frame_seed_at(0.0);
        let s1 = tr_core::flame::tile_frame_seed_at(1.0 / 12.0);
        assert_eq!(s1, tr_core::flame::random_next_seed(s0));
        assert_eq!(tr_core::flame::tile_frame_seed_at(4.9 / 60.0), s0);
    }
    /// W4-g12 doc-sweep: the shroom-cap frame pick
    /// (`TileDrawing.cs:994-1005`) — the `frameX >= 36` gate and the two
    /// EXACT `frameY` equalities (18 → 1, 36 → 2, everything else 0).
    #[test]
    fn w4_g12_shroom_cap_frame_matches_the_two_equality_arms() {
        // Below 36: no cap at all (the trunk-body columns).
        assert_eq!(shroom_cap_frame(0, 0), None);
        assert_eq!(shroom_cap_frame(18, 18), None);
        assert_eq!(shroom_cap_frame(34, 36), None);
        // The gate is >=, so 36 itself carries a cap.
        assert_eq!(shroom_cap_frame(36, 0), Some(0));
        assert_eq!(shroom_cap_frame(36, 18), Some(1));
        assert_eq!(shroom_cap_frame(54, 36), Some(2));
        // Exact equality, not a band: 19/37/54 keep column 0.
        assert_eq!(shroom_cap_frame(36, 19), Some(0));
        assert_eq!(shroom_cap_frame(36, 37), Some(0));
        assert_eq!(shroom_cap_frame(36, 54), Some(0));
    }

    /// W4-g12 doc-sweep: the cap draw sits BEFORE the `skip_draw`/vine
    /// `continue`s and the cell sprite (`TileDrawing.cs:994` precedes the
    /// `!flag` return `:1014`), uses the raw cell light and the
    /// `(-22, -26)` anchor with the 60x42 rect at `num4 * 62`.
    #[test]
    fn w4_g12_shroom_cap_draws_before_the_skip_arms_with_the_raw_light() {
        let src = include_str!("tiles.rs");
        let body = &src[src.find("pub(crate) fn draw_tiles(").unwrap()..];
        let body = &body[..body.find("#[cfg(test)]").unwrap()];
        let cap = body.find("shroom_cap_frame(tile.frame_x").expect("the cap hook");
        let skip = body.find("if dd.skip_draw").expect("the skip arm");
        assert!(cap < skip, "the cap draws before the switch's return arms");
        let cap_block = &body[cap - 400..skip];
        assert!(cap_block.contains("- 22.0"), "the -22 x anchor (:1005)");
        assert!(cap_block.contains("- 26.0"), "the -26 y anchor (:1005)");
        assert!(cap_block.contains("60.0, 42.0"), "the 60x42 rect (:1005)");
        assert!(cap_block.contains("raw_l"), "Lighting.GetColor, not the ghosted l");
        assert!(cap_block.contains("dd.flip"), "tileSpriteEffect rides the case flip");
    }

    /// S1-G-1f draw-order witness: the back-rope hook sits at the C#
    /// dispatch position — AFTER `DrawSingleTile`'s visibility gates
    /// (`!flag` return `:1014-1017`, `IsVisible` `:983`, the dark-cell
    /// merge) and BEFORE the cell's own sprite (`DrawBasicTile` draws it
    /// first, `TileDrawing.cs:1511-1514`; `DrawTile_MinecartTrack` before
    /// the track sprites, `:3792`). The tint argument is the RAW
    /// `raw_l` (`drawData.tileLight`, `:3781`), never the ghosted `l`.
    #[test]
    fn back_rope_draws_after_the_gates_before_the_cell_sprite() {
        let src = include_str!("tiles.rs");
        let body = &src[src.find("pub(crate) fn draw_tiles(").unwrap()..];
        // Cut before the test module — its own literals must not count.
        let body = &body[..body.find("#[cfg(test)]").unwrap()];
        let visible = body.find("stats.visible += 1;").expect("the visibility gate");
        let hook = body.find("ropes::wants_back_rope").expect("the back-rope hook");
        let cell_sprite = body[hook..].find("if is_tree {").expect("the cell sprite arms");
        assert!(
            visible < hook,
            "the back rope must draw after the visibility gates (the C# !flag return)"
        );
        assert!(
            cell_sprite > 0,
            "the back rope must draw before the cell's own sprite"
        );
        // The hook call passes the RAW light, not the actuation-ghosted `l`.
        let call = &body[hook..hook + 400];
        assert!(
            call.contains("raw_l, style"),
            "the tint is drawData.tileLight (TileDrawing.cs:3781), not finalColor"
        );
        // Exactly one dispatch site — the gate covers both C# call sites
        // (380/platforms :1511 and the track :3792) via wants_back_rope.
        assert_eq!(
            body.matches("ropes::draw_back_rope").count(),
            1,
            "one hook, gated by wants_back_rope for both C# sites"
        );
    }

    /// W4-g84 draw-order witness: the bulb arm sits BEFORE the generic
    /// `dd.top != 0` arm (the case-238 top 2 would otherwise route the cell
    /// into the static raised draw) and AFTER the sunflower arm — the
    /// `DrawMultiTileGrass` roster order (`TileDrawing.cs:8756-8764`, 27
    /// first, 236/238/702 next). The transform rides the sunflower counter
    /// (`_sunflowerWindCounter`, `:8918`) and the RAW cell light (`:8975`,
    /// never the actuation-ghosted `l`), with the Glow-coating White
    /// override (`DrawTiles_GetLightOverride :6626-6629`). The follow-up
    /// folds the Life Fruit siblings 236/702 into the same arm (the shared
    /// `236/238/702` registration `:606-613` and size arm `:8760-8764`).
    #[test]
    fn w4_g84_the_bulb_arm_draws_before_the_generic_raised_arm() {
        let src = include_str!("tiles.rs");
        let body = &src[src.find("pub(crate) fn draw_tiles(").unwrap()..];
        let body = &body[..body.find("#[cfg(test)]").unwrap()];
        let sunflower = body.find("} else if tile.kind == 27 {").expect("sunflower arm");
        let bulb = body
            .find("236 | 238 | 702 | 485 | 490 | 652")
            .expect("MultiTileGrass family arm");
        let raised = body.find("} else if dd.top != 0").expect("generic raised arm");
        assert!(sunflower < bulb && bulb < raised, "roster order 27 → MultiTileGrass family → generic");
        let arm = &body[bulb..raised];
        assert!(
            arm.contains("multi_tile_grass::grass_sway_2x2")
                && arm.contains("multi_tile_grass::grass_sway_3x2"),
            "the family draws through the MultiTileGrass transforms"
        );
        assert!(
            arm.contains("sunflower_wind_counter"),
            "the pass rides _sunflowerWindCounter (:8918)"
        );
        assert!(
            arm.contains("raw_l") && arm.contains("fullbright_block"),
            "raw Lighting.GetColor + the Glow-coating White override"
        );
        assert!(
            arm.contains("push_sprite_origin_rot_paint"),
            "the painted DrawNature call (:9000/:9024)"
        );
        // The 521-527 freeze rides the general 2x2 arm's `freeze` flag.
        assert!(
            arm.contains("521..=527") && arm.contains("freeze"),
            "the dragonfly jars freeze (num = 0, flag = false, :8940-8949)"
        );
        // Exactly one family dispatch site.
        assert_eq!(body.matches("236 | 238 | 702 | 485").count(), 1);
        assert_eq!(body.matches("tile.kind == 238").count(), 0);
    }

    /// W4-g84 follow-up draw-order witness: the MultiTileVine arm sits AFTER
    /// the MultiTileGrass family and BEFORE the generic raised arm — the
    /// `PostDrawTiles` bucket order (`TileDrawing.cs:421-422`, MultiTileVines
    /// bucket 3 drains before MultiTileGrass bucket 2, but both precede the
    /// inline generic draw). The arm rides the sunflower counter (`:9240`),
    /// the RAW cell light (`:9486`) + Glow override, and skips the inline
    /// `draw_tile_flames` (the bucket draws its own flame tail, `:9589-9600`).
    #[test]
    fn w4_g84_the_vine_arm_draws_before_the_generic_raised_arm() {
        let src = include_str!("tiles.rs");
        let body = &src[src.find("pub(crate) fn draw_tiles(").unwrap()..];
        let body = &body[..body.find("#[cfg(test)]").unwrap()];
        let grass = body
            .find("236 | 238 | 702 | 485 | 490 | 652")
            .expect("MultiTileGrass family arm");
        let vine = body
            .find("multi_tile_vines::is_multi_tile_vine(tile.kind)")
            .expect("MultiTileVine arm");
        let raised = body.find("} else if dd.top != 0").expect("generic raised arm");
        assert!(grass < vine && vine < raised, "roster order grass → vine → generic");
        let arm = &body[vine..raised];
        assert!(
            arm.contains("multi_tile_vines::vine_sway")
                && arm.contains("multi_tile_vines::draw_vine_cell"),
            "the bucket draws through the MultiTileVine transform + cell draw"
        );
        assert!(
            arm.contains("sunflower_wind_counter"),
            "the pass rides _sunflowerWindCounter (:9240)"
        );
        assert!(
            arm.contains("raw_l") && arm.contains("fullbright_block"),
            "raw Lighting.GetColor + the Glow-coating White override (:9486/:9488)"
        );
        // The inline flame overlay is gated off for the bucket (no double-draw).
        let flame_gate = body
            .find("if !crate::render::multi_tile_vines::is_multi_tile_vine(tile.kind) {")
            .expect("the flame-overlay gate");
        assert!(
            body[flame_gate..].contains("draw_tile_flames"),
            "the gate guards the inline DrawSingleTile_Flames call"
        );
        // Exactly one vine dispatch site.
        assert_eq!(
            body.matches("multi_tile_vines::is_multi_tile_vine(tile.kind)").count(),
            2, // the dispatch arm + the flame-overlay gate
        );
    }
    /// W4-g83 wiring witness: the reverse-vine (549) crawl sits at the C#
    /// switch position (`TileDrawing.cs:546-548`) — right after the forward
    /// vine arm (`:536-545`), BEFORE the draw-data dispatch and any
    /// light/visibility gate (the switch precedes them all), and the cell
    /// `continue`s inline (`:548`) exactly like the vine arm (`:545`).
    #[test]
    fn w4_g83_the_reverse_crawl_rides_the_switch_and_skips_the_inline_draw() {
        let src = include_str!("tiles.rs");
        let body = &src[src.find("pub(crate) fn draw_tiles(").unwrap()..];
        let body = &body[..body.find("#[cfg(test)]").unwrap()];
        let forward = body
            .find("vines::collect_vine_root(")
            .expect("the forward crawl");
        let reverse = body
            .find("vines::collect_reverse_vine_root(")
            .expect("the reverse crawl");
        let dd = body.find("let dd = if active").expect("the drawdata dispatch");
        let visible = body.find("stats.visible += 1;").expect("the visibility gate");
        assert!(forward < reverse, "case 549 follows the vine cases (:546)");
        assert!(
            reverse < dd && reverse < visible,
            "the crawl runs before any draw/light gate (the switch head)"
        );
        // The inline skip covers both families — the C# `continue`s.
        assert!(
            body.contains("if vine_cell || reverse_vine_cell {"),
            "the 549 cell never draws through DrawSingleTile (:548)"
        );
        // One crawl site, one dedup set — `_reverseVineRootsPositions`.
        assert_eq!(body.matches("vines::collect_reverse_vine_root(").count(), 1);
    }

    /// W4-g83 wiring witness: `PostDrawTiles`' nature order — `DrawVines`
    /// (`TileDrawing.cs:429`) BEFORE `DrawReverseVines` (`:430`), both after
    /// the tree-canopy pass (`DrawTrees`, `:428`).
    #[test]
    fn w4_g83_the_rising_pass_draws_after_the_hanging_pass() {
        let src = include_str!("tiles.rs");
        let body = &src[src.find("pub(crate) fn draw_tiles(").unwrap()..];
        let body = &body[..body.find("#[cfg(test)]").unwrap()];
        let trees = body.find("draw_tree_foliage(").expect("the canopy pass");
        let vines = body.find("vines::draw_vines(").expect("the hanging pass");
        let reverse = body
            .find("vines::draw_reverse_vines(")
            .expect("the rising pass");
        assert!(trees < vines && vines < reverse, "DrawTrees→DrawVines→DrawReverseVines");
        assert_eq!(body.matches("vines::draw_reverse_vines(").count(), 1);
    }

    /// TILE-DRAW-2 (W4-f15) draw-order witness: the glow bind sits at the
    /// C# `DrawSingleTile` position — AFTER `DrawBasicTile`'s tail (the
    /// flames hook, `TileDrawing.cs:1723`) and gated on the second
    /// switch's record (`:1343` `glowTexture != null`). The record is no
    /// longer dropped (`_glow` is gone), the platform anchor is the
    /// sprite `pos`, and the 412 cross-fade reads `Main.tileFrame[412]`.
    #[test]
    fn td2_glow_bind_draws_after_the_flames_hook() {
        let src = include_str!("tiles.rs");
        let body = &src[src.find("pub(crate) fn draw_tiles(").unwrap()..];
        let body = &body[..body.find("#[cfg(test)]").unwrap()];
        assert!(
            !body.contains("let _glow"),
            "the glow record must be consumed, not dropped (TILE-DRAW-2)"
        );
        let flames = body.find("draw_tile_flames(r, lookups").expect("flames hook");
        let glow = body
            .find("tile_glow::draw_tile_glow(")
            .expect("the glow bind");
        assert!(
            glow > flames,
            "the glow bind follows DrawBasicTile's flames tail (:1723 → :1343)"
        );
        // Exactly one bind site; the record gate is `if let Some(g)`.
        assert_eq!(body.matches("tile_glow::draw_tile_glow").count(), 1);
        let gate = &body[flames..glow];
        assert!(gate.contains("if let Some(g) = glow"));
        // The call feeds the sprite anchor + the 412 animator row.
        let call = &body[glow..glow + 700];
        assert!(call.contains("tile_anim.animation_row(412)"));
        assert!(call.contains("show_invisible_blocks,"));
        // The dark-merge gate counts the glow record (:971 flag).
        assert!(body.contains("|| glow.is_some()"));
    }

    /// S1-G-1f draw-feed witness: the animator's rows/counters feed the
    /// per-cell drawdata resolution (`Main.tileFrame[typeCache]`,
    /// `TileDrawing.cs:4533`) — `draw_data`, `apply_world_corrections` and
    /// `case_draw_data` all receive `anim_row`, and slice B's counter arm
    /// receives `anim_counter`; the weather-vane bob rides `CaseInputs`.
    #[test]
    fn the_animator_rows_feed_the_drawdata_dispatch() {
        let src = include_str!("tiles.rs");
        let body = &src[src.find("pub(crate) fn draw_tiles(").unwrap()..];
        let body = &body[..body.find("#[cfg(test)]").unwrap()];
        let dd_block = &body[body.find("let (dd, glow) = if active").unwrap()
            ..body.find("// Resolve the drawn tile frame once").unwrap()];
        assert!(
            dd_block.contains("tile_anim.animation_row(tile.kind)"),
            "draw_data's tile_frame is Main.tileFrame[kind]"
        );
        assert!(
            dd_block.contains("tile_anim.tile_frame_counter[tile.kind as usize]"),
            "slice B's counter is Main.tileFrameCounter[kind]"
        );
        // draw_data, apply_world_corrections and case_draw_data take
        // anim_row, and apply_cases_a takes it for 564's pre-clear glow
        // rect (TILE-DRAW-2) — four `anim_row,` arguments; case_draw_data
        // takes the counter pair.
        assert_eq!(dd_block.matches("anim_row,").count(), 4);
        assert!(dd_block.contains("anim_counter,"));
        assert!(body.contains("weather_vane_bobframe: tile_anim.weather_vane_bobframe"));
    }

    /// W4-d115 wiring round — the dodge afterimages draw BEFORE the main
    /// player copy (`LegacyPlayerRenderer.cs:433-467` precedes the
    /// `:468-475` final draw), inside the same cached player pass, and
    /// the counter step runs in the draw orchestrator (the C# steps it
    /// inside the renderer block — the port steps the live field before
    /// the render clone). Source-scan witness, the back-rope convention.
    #[test]
    fn w4_d115_wiring_the_dodge_ghosts_draw_before_the_main_player_copy() {
        let src = include_str!("tiles.rs");
        let body = &src[src.find("pub(crate) fn draw_tiles(").unwrap()..];
        let body = &body[..body.find("#[cfg(test)]").unwrap()];
        let ghosts = body
            .find("draw_dodge_afterimages(")
            .expect("the afterimage pass is wired");
        let player_pass = body.find("draw_phase(\"player\")").unwrap();
        let main = body[ghosts..].find("draw_player(").expect("the main copy");
        assert!(player_pass < ghosts, "the ghosts ride the player pass");
        assert!(main > 0, "the ghosts draw before the main copy (:433-467 < :468)");
        // Exactly one afterimage dispatch — the local player only (the
        // remote copies carry no dodge state, the peer lane is parked).
        assert_eq!(body.matches("draw_dodge_afterimages(").count(), 1);
        // The step lives in the draw orchestrator, before the clone.
        let draw_src = include_str!("../app/frame/draw.rs");
        let step = draw_src
            .find("step_shadow_dodge_draw_counter()")
            .expect("the counter step is wired");
        let clone = draw_src.find("let render_player =").unwrap();
        assert!(step < clone, "the step runs before the render clone");
    }
    /// W4-g78 draw-feed witness (WRITTEN NOT RUN): the tile pass's three
    /// Echo gates read the ONE sight input — the wall layer
    /// (`WallDrawing.cs:91`), the dark-coverage merge
    /// (`Main.DrawBlackTiles` `:58006-58007`) and the per-cell `IsVisible`
    /// (`TileDrawing.cs:983`) — and the per-cell gate is the shared
    /// `scene::tile_is_visible` port (the row-48/541/631 overrides included),
    /// not the bare coating bit the pre-wave pass read.
    #[test]
    fn w4_g78_the_echo_gates_ride_the_sight_input() {
        let src = include_str!("tiles.rs");
        let body = &src[src.find("pub(crate) fn draw_tiles(").unwrap()..];
        let body = &body[..body.find("#[cfg(test)]").unwrap()];
        // The wall layer: one wall_visible call with the coating bit + sight.
        assert!(body.contains(
            "wall_visible(tile.wall, world.invisible_wall(tx, ty), show_invisible_blocks)"
        ));
        // The per-cell gate: the shared IsVisible port, sight-fed.
        assert_eq!(body.matches("tr_core::scene::tile_is_visible").count(), 2,
            "the dark-coverage arm + the per-cell IsVisible gate");
        assert!(body.contains("world.invisible_block(tx, ty),\n                        show_invisible_blocks,")
            || body.contains("show_invisible_blocks,\n                ) {"));
        // No bare-coating `continue` survives (the pre-wave gate).
        assert!(!body.contains("if world.invisible_block(tx, ty) {"));
        assert!(!body.contains("if world.invisible_wall(tx, ty) {"));
        // The vine strips receive the same sight value.
        assert!(body.contains("&vine_roots,\n            show_invisible_blocks,"));
    }

    /// W4-g78: the shown echo wall (318) draws WHITE and never merges into
    /// black coverage — `WallDrawing.cs:100-103` (`color = Color.White` for
    /// wall 318) + `Main.wallLight[318] = true` (`Main.cs:10580`) take the
    /// flat-colour arm regardless of the cell light. The wall pass feeds
    /// the white flag as `fullbright || kind == WALL_ECHO`.
    #[test]
    fn w4_g78_the_shown_echo_wall_feeds_white() {
        let src = include_str!("tiles.rs");
        let body = &src[src.find("pub(crate) fn draw_tiles(").unwrap()..];
        let body = &body[..body.find("#[cfg(test)]").unwrap()];
        assert!(
            body.contains("world.fullbright_wall(tx, ty) || tile.wall == WALL_ECHO"),
            "the echo wall rides the white-light flag (WallDrawing.cs:100-103)"
        );
        assert!(
            body.contains("let echo_wall_lit = tile.wall == WALL_ECHO && show_invisible_blocks;"),
            "a shown echo wall is never dark coverage"
        );
    }
    /// W4-g34 union draw-feed witness (WRITTEN NOT RUN — code-only mode):
    /// the cage rewrite covers ALL 92 ids (`is_cage_kind`, research/197
    /// §2.2 — the g33 42 + g34 50 union) and reads the wired cage animator
    /// through `cage_draw_data`'s `frame_of` (the C# `GetTileDrawData` cage
    /// arms, `TileDrawing.cs:5629-5907`); the `Main.critterCage` flag is
    /// cleared at the pass head (`:474`) and raised for every cage cell
    /// (`:5672` etc.). The old `cage > glow` assertion is INVERTED by the
    /// order witness below (research/197 §2.3 — f14's slice-B/A order).
    #[test]
    fn the_cage_rewrite_feeds_from_the_cage_animator() {
        let src = include_str!("tiles.rs");
        let body = &src[src.find("pub(crate) fn draw_tiles(").unwrap()..];
        let body = &body[..body.find("#[cfg(test)]").unwrap()];
        let dd_block = &body[body.find("let dd = if active").unwrap()
            ..body.find("// Resolve the drawn tile frame once").unwrap()];
        // The rewrite is gated on the FULL cage set and reads the animator.
        assert!(
            dd_block.contains("is_cage_kind(tile.kind)"),
            "the rewrite covers all 92 cage ids (the union wiring)"
        );
        assert!(
            dd_block.contains("cage_anim.frame_of(table, row, idx)"),
            "the cage frame rides the wired animator's tables"
        );
        assert_eq!(
            dd_block.matches("cage_draw_data(").count(),
            1,
            "exactly one live cage dispatch site"
        );
        // The proximity flag: cleared at the head, raised per cage cell.
        assert!(
            body.contains("*critter_cage = false;"),
            "the pass head clears Main.critterCage (TileDrawing.cs:474)"
        );
        assert!(
            body.contains("*critter_cage = true;"),
            "a drawn cage cell raises Main.critterCage (TileDrawing.cs:5672)"
        );
        assert!(
            body.contains("is_cage_kind(tile.kind)"),
            "the flag covers EVERY cage kind (the animator is whole)"
        );
    }

    /// W4-g34 composite-order witness (WRITTEN NOT RUN — f14's
    /// `w4_f14_the_cage_post_pass_rides_the_first_switch_tail` adapted to
    /// the direct `cage_draw_data` call, research/197 §2.3/§5): the cage
    /// rewrite sits at the C# FIRST `GetTileDrawData` switch's tail
    /// (TileDrawing.cs:5629-5907) — AFTER `draw_data` + slice B's
    /// delegated first-switch cases and BEFORE slice A's SECOND switch,
    /// whose 568/569/570/580 glow rects read `tileFrameY + addFrY`
    /// POST-cage (`:6085-6104`), and BEFORE the source-rect resolve
    /// (`:1007`). This REPLACES the old post-slice-A `cage > glow` pin
    /// (the assertion INVERTS to glow > post-cage-read).
    #[test]
    fn the_cage_rewrite_rides_the_first_switch_tail() {
        let src = include_str!("tiles.rs");
        let body = &src[src.find("pub(crate) fn draw_tiles(").unwrap()..];
        let body = &body[..body.find("#[cfg(test)]").unwrap()];
        let dd_block = &body[body.find("let dd = if active").unwrap()
            ..body.find("// Resolve the drawn tile frame once").unwrap()];
        let cases_a = dd_block.find("apply_cases_a").expect("slice A dispatch");
        let chest = dd_block
            .find("is_basic_chest(tile.kind)")
            .expect("the chest rewrite gate");
        assert!(
            cases_a < chest,
            "the chest rewrite runs after GetTileDrawData (TileDrawing.cs:993)"
        );
    }

    /// W4-d58 draw-feed witness: the chest rewrite sits at the C#
    /// `CacheSpecialDraws_Part2` position — AFTER the whole `GetTileDrawData`
    /// dispatch (static table + world corrections + slice B + slice A) and
    /// BEFORE the source-rect resolution (`DrawSingleTile`,
    /// `TileDrawing.cs:993` vs `:1007`), gated on `BasicChest` and keyed on
    /// the per-draw `_chestPositions` cache. The lid frame comes from the
    /// store (`Main.chest[i].frame`, `:4311`) — read-only, no RNG draw.
    #[test]
    fn w4_d58_the_chest_rewrite_runs_after_the_dispatch_before_the_rect() {
        let src = include_str!("tiles.rs");
        let body = &src[src.find("pub(crate) fn draw_tiles(").unwrap()..];
        let body = &body[..body.find("#[cfg(test)]").unwrap()];
        let dd_block = &body[body.find("let dd = if active").unwrap()
            ..body.find("// Resolve the drawn tile frame once").unwrap()];
        let chest = dd_block
            .find("is_basic_chest(tile.kind)")
            .expect("the chest rewrite gate");
        assert!(
            dd_block[chest..].contains("tr_render::chest_frame_adjust"),
            "the +0/38/76 selection lives in tr-render"
        );
        assert!(
            dd_block[chest..].contains("crate::chest::chest_draw_frame"),
            "the lid frame rides the _chestPositions cache (:4300-4303)"
        );
        // Exactly one rewrite site, and the per-draw cache is fresh per call
        // (ClearLegacyCachedDraws, :6613-6615). Count the CALL (`let adj =`),
        // not the name — the citation comment above the gate mentions it too.
        assert_eq!(body.matches("let adj = tr_render::chest_frame_adjust").count(), 1);
        assert!(body.contains("let mut chest_positions"));
        // The rewrite writes addFr + the height clamp, nothing else.
        let rewrite = &dd_block[chest..];
        assert!(rewrite.contains("dd.add_fr_x = adj.add_fr_x;"));
        assert!(rewrite.contains("dd.add_fr_y = adj.add_fr_y;"));
        assert!(rewrite.contains("dd.height = h as u16;"));
    }

    /// W4-f14 composite-order witness: the cage post-pass sits at the C#
    /// FIRST `GetTileDrawData` switch's tail (TileDrawing.cs:5629-5907) —
    /// AFTER `draw_data` + slice B's delegated first-switch cases and
    /// BEFORE slice A's SECOND switch, whose 568/569/570/580 glow rects
    /// read `tileFrameY + addFrY` POST-cage (`:6085-6104`), and BEFORE the
    /// source-rect resolve (`:1007`). The `Main.critterCage = true` side
    /// effect rides the same gated block (`:5672` etc.).
    #[test]
    fn w4_f14_the_cage_post_pass_rides_the_first_switch_tail() {
        let src = include_str!("tiles.rs");
        let body = &src[src.find("pub(crate) fn draw_tiles(").unwrap()..];
        let body = &body[..body.find("#[cfg(test)]").unwrap()];
        let dd_block = &body[body.find("let dd = if active").unwrap()
            ..body.find("// Resolve the drawn tile frame once").unwrap()];
        let table = dd_block.find("tr_core::drawdata::draw_data(").expect("the static table");
        let cases_b = dd_block.find("crate::drawdata_cases_b::case_draw_data(").expect("slice B");
        let cage = dd_block
            .find("tr_core::drawdata_furniture::cage_draw_data(")
            .expect("the cage rewrite");
        let _post_pass = dd_block
            .find("tr_core::drawdata_furniture::furniture_post_pass(")
            .expect("the cage post-pass");
        let cases_a = dd_block
            .find("crate::drawdata_cases_a::apply_cases_a")
            .expect("slice A");
        assert!(table < cases_b, "slice B follows the static table");
        assert!(cases_b < cage, "the cage groups are the FIRST switch's tail");
        assert!(
            cage < cases_a,
            "the SECOND switch's glow rects read the POST-cage addFrY (:6085-6104)"
        );
        // The gate precedes the rewrite and feeds the animator's tables.
        let gate = dd_block.find("is_cage_kind(tile.kind)").unwrap();
        assert!(gate < cage, "the cage-kind gate precedes the rewrite");
        assert!(dd_block.contains("cage_anim.frame_of(table, row, idx)"));
        // The source-rect resolve reads the final addFrX/addFrY (:1007).
        assert!(cage < body.find("// Resolve the drawn tile frame once").unwrap());
    }

    /// W4-g34 union acceptance pin (WRITTEN NOT RUN — g33's
    /// `wired_cage_rewrite_matches_the_static_arms_at_t0` retargeted to
    /// `CritterCageAnimator`, research/197 §5): with the vanilla t=0 tables
    /// the wired cage rewrite is byte-equal to the static arms for EVERY
    /// one of the 92 cage ids — the union wiring changes no pixel until
    /// the animator moves the tables.
    #[test]
    fn wired_cage_rewrite_matches_the_static_arms_at_t0() {
        let a = tr_core::critter_cage_anim::CritterCageAnimator::new();
        let src = include_str!("tiles.rs");
        let body = &src[src.find("pub(crate) fn draw_tiles(").unwrap()..];
        let body = &body[..body.find("#[cfg(test)]").unwrap()];
        let dd_block = &body[body.find("let dd = if active").unwrap()
            ..body.find("// Resolve the drawn tile frame once").unwrap()];
        let cage = dd_block
            .find("tr_core::drawdata_furniture::cage_draw_data(")
            .expect("the cage rewrite");
        let cages: &[u16] = &[
            275, 276, 277, 278, 279, 280, 281, 296, 297, 309, 358, 359, 413, 414, 542, 550, 551,
            553, 554, 558, 559, 599, 600, 601, 602, 603, 604, 605, 606, 607, 608, 609, 610, 611,
            612, 632, 640, 643, 644, 645, 710, 285, 286, 298, 299, 310, 339, 361, 362, 363, 364,
            391, 392, 393, 394, 532, 533, 538, 544, 555, 556, 582, 619, 629, 282, 505, 543, 598,
            568, 569, 570, 288, 289, 290, 291, 292, 293, 294, 295, 360, 580, 620, 521, 522, 523,
            524, 525, 526, 527, 316, 317, 318,
        ];
        assert_eq!(cages.len(), 92);
        for &kind in cages {
            let static_arm = tr_core::drawdata::draw_data(kind, 13, 7, 18, 18, 0, false);
            let wired = tr_core::drawdata_furniture::cage_draw_data(
                kind, 13, 7, 18, 18,
                tr_core::drawdata_furniture::CAGE_FRAMES,
                |t, r, i| a.frame_of(t, r, i),
            )
            .unwrap();
            assert_eq!(wired, static_arm, "kind {kind}");
        }
        // Exactly one dispatch site, gated on a cage kind, and the flag
        // set rides the same block BEFORE the table lookup.
        assert_eq!(
            dd_block.matches("tr_core::drawdata_furniture::furniture_post_pass(").count(),
            1
        );
        assert!(dd_block.contains("is_cage_kind(tile.kind)"));
        let gate = dd_block.find("is_cage_kind(tile.kind)").unwrap();
        let flag = dd_block.find("cage_anim.critter_cage = true").unwrap();
        assert!(gate < flag && flag < cage, "the flag set gates the rewrite");
        // The lookup feeds the animator's tables (not a zero stand-in).
        assert!(dd_block.contains("cage_anim.frame_of(table, row, idx)"));
        // The chest half stays cage-only here (the W4-d58 seam owns :993).
        let call = &dd_block[cage..(cage + 600).min(dd_block.len())];
        assert!(call.contains("None,"), "chest_frame stays None (W4-d58 seam)");
    }

    /// W4-f14 flag-clear witness: `Main.critterCage = false` rides the
    /// walk's start (the port's stand-in for the non-solid layer's
    /// `Draw` head, TileDrawing.cs:472-475), BEFORE any cell can set it.
    #[test]
    fn w4_f14_the_walk_start_clears_the_cage_flag() {
        let src = include_str!("tiles.rs");
        let body = &src[src.find("pub(crate) fn draw_tiles(").unwrap()..];
        let body = &body[..body.find("#[cfg(test)]").unwrap()];
        let clear = body.find("cage_anim.critter_cage = false").expect("the clear");
        let set = body.find("cage_anim.critter_cage = true").expect("the set");
        let walk = body
            .find("scratch.open.clear();")
            .expect("the tile walk's head");
        assert!(clear < walk && walk < set, "the clear precedes the walk");
        assert_eq!(body.matches("cage_anim.critter_cage = false").count(), 1);
    }

    // S1-G-9: `DrawSingleTile_SlicedBlock`'s per-slice light chain
    // (`TileDrawing.cs:3578-3581` high arm, `:3634-3637` medium arm): the
    // slice averages the neighbourhood colour with the tile's own RAW light
    // (`drawData.tileLight`, `:3528`), then `GetFinalLight` (`:4391-4402`)
    // applies the tint and the actuation ghost (`Tile.actColor`,
    // `Tile.cs:282-288`) to that AVERAGE.
    #[test]
    fn sliced_block_averages_the_neighbourhood_with_the_own_light() {
        // The centre slice: (own + own) * 0.5 = own; an edge slice: the
        // average of the two.
        let own = [0.8, 0.6, 0.4];
        assert_eq!(sliced_light(own, own, false), own);
        let edge = sliced_light([0.2, 0.2, 0.2], own, false);
        assert_eq!(edge, [0.5, 0.4, 0.3]);
    }

    #[test]
    fn sliced_block_applies_the_actuation_ghost_per_slice() {
        // `TileDrawing.cs:4394-4396`: the actuated tile's RGB x 0.4 rides
        // the per-slice GetFinalLight — the ghost hits the AVERAGE, not the
        // raw slice.
        let got = sliced_light([0.5, 0.5, 0.5], [0.1, 0.1, 0.1], true);
        assert!((got[0] - 0.12).abs() < 1e-6, "{got:?}");
        // The pre-fix shape (the ghost on the raw neighbour, no avg) reads
        // 0.04 — the regression pin.
        let raw_neighbour_ghost = 0.5f32 * 0.4;
        assert!((raw_neighbour_ghost - got[0]).abs() > 0.05);
    }

    #[test]
    fn sliced_block_uses_the_raw_own_light_in_the_average() {
        // S1-G-9's second half: `:3528` feeds the sliced arm the RAW
        // `drawData.tileLight`, so the caller must not pre-ghost `own` — the
        // per-slice `GetFinalLight` is the only ×0.4 in the chain. Ghosting
        // the caller's value too would square the factor (0.4 -> 0.16).
        let raw_own = [0.5, 0.5, 0.5];
        let slice = [0.5, 0.5, 0.5];
        let good = sliced_light(slice, raw_own, true);
        let double_ghosted = sliced_light(slice, [0.2, 0.2, 0.2], true);
        assert!((good[0] - 0.2).abs() < 1e-6, "{good:?}");
        assert!((double_ghosted[0] - 0.14).abs() < 1e-6, "{double_ghosted:?}");
        assert!((good[0] - double_ghosted[0]).abs() > 0.05);
    }

    #[test]
    fn sliced_block_glow_coating_draws_full_white_slices() {
        // `TileDrawing.cs:3530-3533` / `:3611-3614`: a Glow-coated
        // (`fullbrightBlock`) block replaces every slice with
        // `_glowPaintColorSlices` (`:160-...`, all `Vector3.One`), so each
        // slice averages 1.0 with the own light — the centre slice is at
        // least 0.5 whatever the darkness.
        for own in [[0.0, 0.0, 0.0], [0.2, 0.3, 0.4]] {
            let centre = sliced_light([1.0; 3], own, false);
            assert!(centre.iter().zip(own).all(|(c, o)| c >= &(o * 0.5)));
        }
        assert_eq!(sliced_light([1.0; 3], [1.0; 3], false), [1.0; 3]);
        // Without the glow replacement a black cell stays black.
        assert_eq!(sliced_light([0.0; 3], [0.0; 3], false), [0.0; 3]);
    }


    /// 40x40 world, no tiles — the `sunflower_sway` probe needs `World::tile`
    /// inside the 2x5 `InAPlaceWithWind` window to answer "no liquid, no
    /// blocking wall" (the `object_world` fixture shape without the floor).
    fn empty_world() -> World {
        let mut w = World::new("t".into(), 40, 40);
        w.compute_column_surface();
        w.compute_chunks();
        w
    }

    /// A wind above the 0.08..0.18 gate (so the cycle lives) under an infinite
    /// surface (so the surface band never zeroes it).
    fn windy() -> Wind {
        let mut wind = Wind::new(7);
        wind.speed_current = 0.5;
        wind.world_surface = f64::INFINITY;
        wind
    }

    // The per-cell transform of `DrawMultiTileGrassInWind`
    // (`TileDrawing.cs:8916-9041`): `num3 = 1 - (j - topLeftY + 1) / 5` per row
    // (0.8/0.6/0.4/0.2 for the flower's four rows), the vertical offset
    // `|windCycle| * 2 * num3`, the rotation `windCycle * 0.07 * num3` about
    // the flower's bottom-center anchor `(topLeftX*16 + 16, topLeftY*16 + 80)`
    // — the special pass's `sizeX = 2, sizeY = 5` (`:8748-8752`).
    #[test]
    fn test_DW_4_1_sunflower_sway_transform_per_row() {
        let wind = windy();
        let world = empty_world();
        // A style-0 flower with its top-left at (10, 20): the four rows' cells
        // carry frameY 0/18/36/54 and the two columns frameX 0/18.
        let mut per_row: Vec<SunflowerSway> = Vec::new();
        for row in 0..4_i32 {
            for col in 0..2_i32 {
                let (tx, ty) = (10 + col, 20 + row);
                let fx = (18 * col) as i16;
                let fy = (18 * row) as i16;
                let sw = sunflower_sway(&wind, &world, tx, ty, fx, fy, 0.0);
                // The pivot reconstructs the same anchor from every cell:
                // (topLeftX*16 + 16 - tx*16, topLeftY*16 + 80 - ty*16).
                assert_eq!(
                    sw.pivot,
                    [(16 - 16 * col) as f32, (80 - 16 * row) as f32],
                    "pivot of cell ({tx}, {ty})"
                );
                per_row.push(sw);
            }
        }
        // The rows scale exactly 0.8 : 0.6 : 0.4 : 0.2 — dy ratios 4:3:2:1.
        let (d0, d1, d2, d3) = (
            per_row[0].dy,
            per_row[2].dy,
            per_row[4].dy,
            per_row[6].dy,
        );
        assert!(d3 > 0.0, "the bottom row still sways (num3 = 0.2)");
        assert!((d0 / d3 - 4.0).abs() < 1e-4, "{d0} / {d3}");
        assert!((d1 / d3 - 3.0).abs() < 1e-4, "{d1} / {d3}");
        assert!((d2 / d3 - 2.0).abs() < 1e-4, "{d2} / {d3}");
        // |rot| / dy = 0.07 / 2 for every cell, whatever the cycle's sign.
        for sw in &per_row {
            assert!(
                (sw.rot.abs() / sw.dy - 0.035).abs() < 1e-4,
                "rot {} dy {}",
                sw.rot,
                sw.dy
            );
        }
        // A style-1 flower (frameX offset 36) reconstructs the same top-left.
        let sw1 = sunflower_sway(&wind, &world, 11, 20, 54, 0, 0.0);
        assert_eq!(sw1.pivot, [0.0, 80.0]);
        let sw1_bottom = sunflower_sway(&wind, &world, 10, 23, 36, 54, 0.0);
        assert_eq!(sw1_bottom.pivot, [16.0, 32.0]);
    }

    // The counter moves the flower: feeding the cadence
    // `1/420 + num*5/420` (`TileDrawing.cs:374`) changes both the vertical
    // offset and the rotation of the same cell.
    #[test]
    fn test_DW_4_1_sunflower_sway_tracks_the_counter() {
        let wind = windy();
        let world = empty_world();
        let before = sunflower_sway(&wind, &world, 10, 20, 0, 0, 0.0);
        // One in-game second of the fixed-step advance at |wind| = 0.5.
        let mut counter = 0.0_f32;
        for _ in 0..60 {
            tr_core::wind::advance_sunflower_counter(&mut counter, 0.5);
        }
        let after = sunflower_sway(&wind, &world, 10, 20, 0, 0, counter);
        assert_ne!(before.dy, after.dy, "the vertical offset tracks the counter");
        assert_ne!(before.rot, after.rot, "the rotation tracks the counter");
    }

    // `GetWindCycle`'s gates freeze the flower (`TileDrawing.cs:7676-7700`):
    // below the surface band, with sway disabled, or outside
    // `InAPlaceWithWind(topLeft, 2, 5)` (`:8923`, `:8961`) every cell draws
    // statically.
    #[test]
    fn sunflower_sway_gates_freeze_the_flower() {
        let world = empty_world();
        // Below the surface band (y >= worldSurface): cycle 0.
        let mut wind = windy();
        wind.world_surface = 25.0;
        let sw = sunflower_sway(&wind, &world, 10, 30, 0, 0, 0.0);
        assert_eq!(sw.dy, 0.0);
        assert_eq!(sw.rot, 0.0);
        // Sway disabled (`SettingsEnabled_TilesSwayInWind`): cycle 0.
        let mut wind = windy();
        wind.sway_enabled = false;
        let sw = sunflower_sway(&wind, &world, 10, 20, 0, 0, 0.0);
        assert_eq!(sw.dy, 0.0);
        assert_eq!(sw.rot, 0.0);
        // A wind-blocking wall inside the 2x5 probe window (`WallID.Sets.
        // AllowsWind` fails for wall 1): `num3 = 0` — dy and rot are 0 even
        // though the cycle is not.
        let mut wind = windy();
        wind.world_surface = f64::INFINITY;
        let mut walled = empty_world();
        let i = walled.index(10, 22);
        walled.tiles[i].wall = 1;
        let sw = sunflower_sway(&wind, &walled, 11, 20, 18, 0, 0.0);
        assert_eq!(sw.dy, 0.0);
        assert_eq!(sw.rot, 0.0);
        assert_eq!(sw.pivot, [0.0, 80.0]);
    }

    fn sheet_atlas(names: &[&str], w: u32, h: u32) -> Atlas {
        let images = names
            .iter()
            .map(|&name| tr_render::AtlasImage {
                name: name.to_string(),
                width: w,
                height: h,
                rgba: vec![180; (w * h * 4) as usize],
                layout: tr_render::SheetLayout::Tile,
            })
            .collect();
        Atlas::build(images, 1024)
    }

    // Bug 19/19b: the cactus draw. `DrawSingleTile` case 80 shifts the source
    // rectangle down by the ground sand's biome row (`TileDrawing.cs:1132-1147`),
    // and only the plain flat arm draws that rect (`:1523`). A port that drops
    // the shift shows the wrong cactus sprite in the evil/hallow/crimson desert.
    #[test]
    fn test_BUG19b_the_cactus_flat_draw_carries_the_biome_row_shift() {
        let atlas = sheet_atlas(&["Tiles_80"], 270, 252);
        let mut lookups = assets::Lookups::build(
            &atlas,
            [(80_u16, "Tiles_80".to_string())].into_iter(),
            std::iter::empty(),
            None,
            None,
            None,
            None,
            None,
        );
        let tf = lookups
            .frame_tex(&atlas, 80, 0, 0)
            .expect("the cactus frame resolves");
        let mut yy = |ground: u16| {
            let mut w = World::new("t".into(), 40, 40);
            for (x, y, kind) in [(20_i32, 19_i32, 80_u16), (20, 20, ground)] {
                let i = w.index(x, y);
                w.tiles[i].kind = kind;
                w.tiles[i].flags = tr_core::tile::FLAG_ACTIVE;
            }
            let mut dd = tr_core::drawdata::TileDrawData::default();
            let tile = *w.tile(20, 19).unwrap();
            dd.apply_world_corrections(&w, 80, 20, 19, 0, 0, 0, &Default::default());
            let uv = flat_src_uv(&dd, &tile, tf, Some(&atlas), &lookups);
            (dd.src_y_shift, uv[1])
        };
        let (plain_shift, plain_y) = yy(53); // normal sand
        let (evil_shift, evil_y) = yy(112); // ebonstone sand
        assert_eq!(plain_shift, 0, "normal sand keeps the cell row");
        assert_eq!(evil_shift, 54, "evil sand moves the sprite 54 px down");
        assert!(evil_y > plain_y, "the evil draw samples a lower row");
        assert!(
            (evil_y - plain_y) * 252.0 > 50.0,
            "the shift reaches the UV: {} vs {}",
            evil_y,
            plain_y
        );
        // `src_y_shift == 0` must reproduce the plain frame exactly.
        assert_eq!(plain_y, tf.uv[1], "no shift, no UV change");
    }

    // A tile whose frames were never written (`frameX/frameY = -1`, the state
    // the old seed bug left behind) resolves no frame at all — the plain-sheet
    // fallback that used to draw sheet-corner garbage is gated out, while the
    // same kind with real frames still resolves.
    #[test]
    fn test_DW_4_3_unframed_tile_draws_nothing() {
        let atlas = sheet_atlas(&["Tiles_27", "Tiles_82"], 126, 148);
        let mut lookups = assets::Lookups::build(
            &atlas,
            [(27_u16, "Tiles_27".to_string()), (82, "Tiles_82".to_string())]
                .into_iter(),
            std::iter::empty(),
            None,
            None,
            None,
            None,
            None,
        );
        let dd = tr_core::drawdata::TileDrawData::default();
        // The fallback WOULD have produced a sprite (the plain sheet cell
        // exists) — the guard is what keeps the unframed tile blank.
        assert!(lookups.plain_tex(27).is_some());
        assert!(lookups.plain_tex(82).is_some());
        for (kind, fx, fy) in [(27_u16, -1_i16, -1_i16), (82, -1, -1), (82, 0, -1)] {
            let frame = resolve_tile_frame(
                &mut lookups,
                Some(&atlas),
                false,
                0,
                kind,
                fx,
                fy,
                &dd,
            );
            assert!(frame.is_none(), "kind {kind} frame ({fx},{fy}) draws nothing");
        }
        // Real frames still resolve: the herbs sit on the 18 px grid, the
        // sunflower's bottom row draws the 16x18 slice.
        let herb = resolve_tile_frame(&mut lookups, Some(&atlas), false, 0, 82, 18, 0, &dd);
        assert!(herb.is_some(), "herb frame (18,0) resolves");
        let bottom = resolve_tile_frame(
            &mut lookups,
            Some(&atlas),
            false,
            0,
            27,
            0,
            54,
            &tr_core::drawdata::TileDrawData {
                height: 18,
                ..Default::default()
            },
        );
        assert!(bottom.is_some(), "sunflower bottom row resolves");
    }

    /// W4-g80 DOC-SWEEP: the 83 herb-maturity sheet swap reaches the pixels.
    /// `DrawSingleTile` case 83 (`TileDrawing.cs:1149-1151`) re-picks the draw
    /// texture through `GetTileDrawTexture` (`:1450-1455`): a harvestable
    /// herb samples `Tiles_84` at the SAME frame, an unharvestable one keeps
    /// `Tiles_83`. The renderer consumes `dd.sheet_kind_override` at the
    /// `resolve_tile_frame` call.
    #[test]
    fn w4_g80_the_harvestable_herb_draws_the_mature_sheet() {
        let atlas = sheet_atlas(&["Tiles_83", "Tiles_84"], 126, 148);
        let mut lookups = assets::Lookups::build(
            &atlas,
            [
                (83_u16, "Tiles_83".to_string()),
                (84, "Tiles_84".to_string()),
            ]
            .into_iter(),
            std::iter::empty(),
            None,
            None,
            None,
            None,
            None,
        );
        let mut w = World::new("t".into(), 40, 40);
        w.day_time = true; // daybloom (style 0) harvests by day
        let dry = tr_core::growth::WeatherCtx::default();
        let mut dd = tr_core::drawdata::draw_data(83, 10, 20, 0, 0, 0, false);
        dd.apply_world_corrections(&w, 83, 10, 20, 0, 0, 0, &dry);
        assert_eq!(dd.sheet_kind_override, Some(84));
        let mature = resolve_tile_frame(
            &mut lookups,
            Some(&atlas),
            false,
            0,
            dd.sheet_kind_override.unwrap_or(83),
            0,
            0,
            &dd,
        )
        .expect("the mature herb frame resolves");
        let own = lookups.frame_tex(&atlas, 83, 0, 0).expect("Tiles_83 (0,0)");
        let sheet84 = lookups.frame_tex(&atlas, 84, 0, 0).expect("Tiles_84 (0,0)");
        assert_eq!(mature.uv, sheet84.uv, "the swap samples Tiles_84");
        assert_ne!(mature.uv, own.uv, "not the herb's own sheet");
        // Night: no swap — the same call shape keeps Tiles_83.
        w.day_time = false;
        let mut dd = tr_core::drawdata::draw_data(83, 10, 20, 0, 0, 0, false);
        dd.apply_world_corrections(&w, 83, 10, 20, 0, 0, 0, &dry);
        assert_eq!(dd.sheet_kind_override, None);
        let green = resolve_tile_frame(
            &mut lookups,
            Some(&atlas),
            false,
            0,
            dd.sheet_kind_override.unwrap_or(83),
            0,
            0,
            &dd,
        )
        .expect("the green herb frame resolves");
        assert_eq!(green.uv, own.uv, "the unharvestable herb keeps its sheet");
        // The dispatch witness: the production resolve call consumes the
        // override (a dropped unwrap_or would silently keep Tiles_83).
        let src = include_str!("tiles.rs");
        let body = &src[src.find("pub(crate) fn draw_tiles(").unwrap()..];
        let body = &body[..body.find("#[cfg(test)]").unwrap()];
        assert!(
            body.contains("dd.sheet_kind_override.unwrap_or(tile.kind)"),
            "the resolve call must consume the case-83 sheet override"
        );
        assert!(
            body.contains("&herb_wx,"),
            "apply_world_corrections must receive the live weather"
        );
    }

    // fix/vines-destroy-walls: the wall pass draws BEHIND vine cells. C#
    // `WallDrawing.DrawWalls` skips a cell only when `FullTile` holds
    // (`WallDrawing.cs:91`, `:241-442`) — a full SOLID unsloped block; a vine
    // is not solid, so its wall always draws. The port's gate is
    // `tile_hides_wall` (the frame-opacity proxy): a vine frame is
    // transparent → not opaque → the wall draws; an opaque dirt frame hides
    // it. The pre-fix worldgen pass erased the wall itself; this pins the
    // draw side so a vine cell never suppresses its background wall.
    #[test]
    fn walls_draw_under_vine_cells() {
        // A transparent vine sheet (alpha 0) and an opaque dirt sheet
        // (alpha 255), both on the 18 px frame grid.
        let sheet = |name: &str, alpha: u8| tr_render::AtlasImage {
            name: name.to_string(),
            width: 36,
            height: 36,
            rgba: (0..(36 * 36)).flat_map(|_| [120, 90, 60, alpha]).collect(),
            layout: tr_render::SheetLayout::Tile,
        };
        let atlas = Atlas::build(vec![sheet("Tiles_62", 0), sheet("Tiles_0", 255)], 1024);
        let mut lookups = assets::Lookups::build(
            &atlas,
            [(62_u16, "Tiles_62".to_string()), (0, "Tiles_0".to_string())].into_iter(),
            std::iter::empty(),
            None,
            None,
            None,
            None,
            None,
        );
        // The dirt wall is visible (not 0, not the echo wall 318).
        assert!(wall_visible(2, false, false), "WallID.DirtUnsafe draws");
        // The vine cell does NOT hide its wall — the wall pass draws it.
        assert!(
            !tile_hides_wall(&mut lookups, Some(&atlas), 62, 0, 0, 0, false),
            "a jungle-vine cell keeps its background wall"
        );
        // Control: an opaque full dirt block hides the wall (FullTile).
        assert!(
            tile_hides_wall(&mut lookups, Some(&atlas), 0, 0, 0, 0, false),
            "a full dirt block hides its wall"
        );
        // A slope / half-brick never hides the wall, even on dirt
        // (`blockType() != 0`, `WallDrawing.cs:243` / `Tile.cs:202-214`).
        assert!(!tile_hides_wall(&mut lookups, Some(&atlas), 0, 0, 0, 1, false));
        assert!(!tile_hides_wall(&mut lookups, Some(&atlas), 0, 0, 0, 0, true));
    }

    // S1-G-13 consumers: the `solidLayer` split (`TileDrawing.Draw`'s per-cell
    // test, `TileDrawing.cs:517`, via `IsTileDrawLayerSolid` `:4469-4475`).
    // The fused default keeps every cell; the two halves partition the active
    // cells by solidity; the direct-screen blocks draw no cells.
    #[test]
    fn tile_filter_partitions_the_active_cells_by_solidity() {
        // Dirt (0) is solid; a torch (4) is not. The forced-solid override
        // table (`TileID.Sets.DrawTileInSolidLayer`, `TileID.cs:516-528`)
        // pins 11/78/470/475/579 solid regardless of `_tileSolid`.
        assert!(tile_draw_layer_solid(0), "dirt is solid");
        assert!(!tile_draw_layer_solid(4), "a torch is non-solid");
        // The fused default keeps every cell, active or not.
        assert!(TileFilter::All.keeps(0, true));
        assert!(TileFilter::All.keeps(4, false));
        // The solid half keeps only active solid cells.
        assert!(TileFilter::Solid.keeps(0, true));
        assert!(!TileFilter::Solid.keeps(4, true));
        assert!(!TileFilter::Solid.keeps(0, false), "inactive cells draw in neither half");
        // The non-solid half keeps only active non-solid cells.
        assert!(TileFilter::NonSolid.keeps(4, true));
        assert!(!TileFilter::NonSolid.keeps(0, true));
        assert!(!TileFilter::NonSolid.keeps(4, false));
        // The two halves are complementary on active cells.
        for kind in [0_u16, 4, 11, 78, 470, 475, 579, 82, 192] {
            assert_ne!(
                TileFilter::Solid.keeps(kind, true),
                TileFilter::NonSolid.keeps(kind, true),
                "kind {kind} lands in exactly one half"
            );
        }
        // The direct-screen blocks draw no tile cells.
        assert!(!TileFilter::Screen.keeps(0, true));
        assert!(!TileFilter::ScreenExtras.keeps(4, true));
    }

    // S1-G-13 consumers: the forced-solid override wins over `_tileSolid`
    // (`IsTileDrawLayerSolid` checks `DrawTileInSolidLayer[type].HasValue`
    // first, `TileDrawing.cs:4471-4474`).
    #[test]
    fn tile_filter_honours_the_forced_solid_override() {
        // The generated table sets these to 1 (forced solid).
        for kind in [11_u16, 78, 470, 475, 579] {
            assert_eq!(
                td::SET_DRAW_TILE_IN_SOLID_LAYER.get(kind as usize).copied(),
                Some(1),
                "kind {kind} is forced solid in the generated table"
            );
            assert!(tile_draw_layer_solid(kind), "kind {kind} draws in the solid half");
            assert!(TileFilter::Solid.keeps(kind, true));
            assert!(!TileFilter::NonSolid.keeps(kind, true));
        }
    }

    // GATE RNG fix witnesses (S1-G-13 consumers): the shared leaf-roll
    // stream reproduces the fused single-stream consumption order.

    /// The kind-192 leaf cells (the ONLY cell-loop rolls,
    /// `TileDrawing.cs:6849-6855`) are SOLID — they land in the Solid half,
    /// so the gated Solid callback's rolls are exactly the fused walk's
    /// cell rolls, and the NonSolid half rolls nothing.
    #[test]
    fn the_leaf_cell_rolls_live_in_the_solid_half() {
        assert!(td::TILE_SOLID.has(192), "living-leaf blocks are solid");
        assert!(tile_draw_layer_solid(192));
        assert!(TileFilter::Solid.keeps(192, true));
        assert!(!TileFilter::NonSolid.keeps(192, true));
    }

    /// The shared-stream mechanics: a gated call takes the parked stream,
    /// advances it, and parks it back; the next call continues from the
    /// advanced state — NOT from the pristine seed. The fused default
    /// (`leaf_rng_shared == false`) ignores the slot entirely.
    #[test]
    fn the_shared_leaf_stream_continues_across_gated_calls() {
        use tr_core::tile_light::FastRandom;
        let seed = 0x1234_5678_9ABC_DEF0u64 ^ 0x9E37_79B9_7F4A_7C15;
        let mut scratch = DrawScratch::default();
        scratch.leaf_rng_shared = true;
        scratch.leaf_rng = Some(FastRandom::new(seed));
        // Call 1 (the Solid half): take + roll + park.
        let mut rng = scratch.leaf_rng.take().unwrap();
        let first = rng.next_range(0, 500);
        scratch.leaf_rng = Some(rng);
        // Call 2 (the extras foliage): continues — the fused single-stream
        // shape. Re-seeding here (the pre-fix bug) would repeat `first`.
        let mut rng = scratch.leaf_rng.take().unwrap();
        let second = rng.next_range(0, 500);
        scratch.leaf_rng = Some(rng);
        // The reference: one uninterrupted stream from the same seed.
        let mut reference = FastRandom::new(seed);
        assert_eq!(first, reference.next_range(0, 500));
        assert_eq!(second, reference.next_range(0, 500));
        // The pristine re-seed (the pre-fix gated behaviour) diverges.
        let mut pristine = FastRandom::new(seed);
        assert_eq!(pristine.next_range(0, 500), first);
        assert_ne!(
            pristine.next_range(0, 500),
            second,
            "the pre-fix re-seed repeats the first draw — the divergence pin"
        );
        // The fused default never reads the slot: with the flag off the
        // parked stream stays untouched while the call re-seeds its own.
        scratch.leaf_rng_shared = false;
        let parked_seed = scratch.leaf_rng.unwrap().seed;
        let mut fused = FastRandom::new(seed);
        let _ = fused.next_range(0, 500);
        assert_eq!(scratch.leaf_rng.unwrap().seed, parked_seed);
        assert_ne!(fused.seed, parked_seed);
    }

    /// The gated frame pre-seeds the shared stream with the fused pin
    /// (`sparkle_seed ^ 0x9E37…`) and clears the shared mode at the tail;
    /// the capture chunk pins its own stream once per render.
    #[test]
    fn the_gated_consumers_pre_seed_and_clear_the_shared_stream() {
        let lt = include_str!("../app/layer_targets.rs");
        assert!(lt.contains("frame.scratch.leaf_rng_shared = true;"));
        assert!(lt.contains("frame.sparkle_seed ^ 0x9E37_79B9_7F4A_7C15"));
        assert!(lt.contains("frame.scratch.leaf_rng_shared = false;"));
        let cr = include_str!("../app/capture_render.rs");
        assert!(cr.contains("cw.scratch.leaf_rng_shared = true;"));
        assert!(cr.contains("self.cw.scratch.leaf_rng_shared = false;"));
        // Every draw_tiles exit parks the stream back when shared.
        let src = include_str!("tiles.rs");
        let body = &src[src.find("pub(crate) fn draw_tiles(").unwrap()..];
        let body = &body[..body.find("#[cfg(test)]").unwrap()];
        assert_eq!(
            body.matches("scratch.leaf_rng = Some(leaf_rng);").count(),
            3,
            "all three exits (halves, extras, tail) park the stream"
        );
    }

    // ---- W4-g80: the actuated draw lane — disco finalColor + ghost ----

    /// W4-g80 witness — the 160/692 final colour (`TileDrawing.cs:1078-1088`):
    /// the disco bytes with alpha 255 (the 3-byte `Color` ctor), actuated →
    /// `Tile.actColor` double 0.4 truncation (255 → 102, alpha stays 255 —
    /// the `Color` overload preserves A, unlike the wire plate's
    /// `Color * float` which scales alpha too).
    #[test]
    fn w4_g80_the_disco_final_colour_ghosts_through_act_color() {
        // Active: the raw disco cycle, alpha 255.
        let c = disco_final_color([255, 128, 0], false);
        assert_eq!(c, [1.0, 128.0 / 255.0, 0.0, 1.0]);
        // Actuated: 0.4*255=102, 0.4*128=51.2→51, 0.4*0=0 — alpha 255 kept.
        let g = disco_final_color([255, 128, 0], true);
        assert_eq!(g, [102.0 / 255.0, 51.0 / 255.0, 0.0, 1.0]);
        // The vanilla boot state (DiscoR 255, G/B 0, `Main.Initialize`).
        let boot = disco_final_color([255, 0, 0], true);
        assert_eq!(boot, [102.0 / 255.0, 0.0, 0.0, 1.0]);
    }

    /// W4-g80 witness — the override replaces the WHOLE final colour: the
    /// caller's `tileLight` ghost (`tiles.rs`'s `l = raw * 0.4`) must not
    /// ride on top (that would square the factor to 0.16 on the disco
    /// bytes). `push_lit`'s override branch draws the resolved bytes and
    /// skips the light shade / `light_scale` / `src_alpha` entirely —
    /// pinned by source order: the `final_override` early-return precedes
    /// the `style.shade` call.
    #[test]
    fn w4_g80_push_lit_draws_the_override_before_the_light_shade() {
        let src = include_str!("tiles.rs");
        let body = &src[src.find("fn push_lit(").unwrap()..];
        // Cut at the next item's doc comment (CRLF-safe).
        let body = &body[..body.find("/// Sub-rectangle").unwrap()];
        let ov = body.find("if let Some(c) = final_override").unwrap();
        let shade = body.find("style.shade").unwrap();
        assert!(ov < shade, "the override branch must precede the shade");
        // The disco resolution feeds `Tile.actColor`'s byte overload, never
        // the ghosted `l`.
        let walk = &src[src.find("let disco_final: Option<[f32; 4]>").unwrap()..];
        let walk = &walk[..walk.find("};").unwrap()];
        assert!(walk.contains("disco_final_color(params.disco, inactive)"));
        assert!(!walk.contains("* 0.4"), "no double ghost on the override");
    }

    /// W4-g80 witness — the sliced-block gate carries
    /// `TileID.Sets.DontDrawTileSliced` (`TileDrawing.cs:1509`, the set at
    /// `TileID.cs:482` includes 160): the RainbowBrick never takes the
    /// per-slice `tileLight` path, so its disco `finalColor` always reaches
    /// the flat draw. 692 is NOT in the set, but the sliced arm draws
    /// per-slice light colours (`:3578-3601`) and never reads `finalColor`
    /// — the C# behaves identically, the override simply does not apply
    /// there.
    #[test]
    fn w4_g80_rainbow_brick_is_dont_draw_tile_sliced() {
        assert!(td::SET_DONT_DRAW_TILE_SLICED.has(160), "TileID.cs:482");
        assert!(!td::SET_DONT_DRAW_TILE_SLICED.has(692));
        // The walk gate reads the set (source-order pin).
        let src = include_str!("tiles.rs");
        assert!(src.contains("!td::SET_DONT_DRAW_TILE_SLICED.has(tile.kind)"));
    }

    /// W4-g80 witness — the dark-cell merge exempts 692: its GlowMask 341
    /// (`Main.cs:10043`) sets the C# draw `flag` (`TileDrawing.cs:971`)
    /// regardless of light, so the cell keeps its own quad (the disco
    /// sprite draws over the black). 160 has no mask/flame row — the C#
    /// `!flag` return (`:1014-1017`) precedes the positional switch, so a
    /// dark 160 cell draws black in the C# too; the merge stays faithful.
    #[test]
    fn w4_g80_the_glow_mask_kind_escapes_the_dark_merge() {
        let src = include_str!("tiles.rs");
        let gate = &src[src.find("let glow_mask_tile").unwrap()..];
        let gate = &gate[..gate.find("if light_is_zero").unwrap()];
        assert!(gate.contains("tile.kind == 692"));
        assert!(gate.contains("|| glow_mask_tile"));
    }

    // ---- W4-d59: the DrawBasicTile slope family wiring --------------------

    /// 40x40 world; `place` activates a tile of `kind` at (x, y).
    fn join_world(tiles: &[(i32, i32, u16)]) -> World {
        let mut w = World::new("joins".into(), 40, 40);
        for &(x, y, kind) in tiles {
            let t = w.tile_mut(x, y).unwrap();
            t.kind = kind;
            t.flags |= tr_core::tile::FLAG_ACTIVE;
        }
        w
    }

    fn half_at(w: &mut World, x: i32, y: i32, kind: u16) {
        let t = w.tile_mut(x, y).unwrap();
        t.kind = kind;
        t.flags |= tr_core::tile::FLAG_ACTIVE | tr_core::tile::FLAG_HALF_BRICK;
    }

    // `TileDrawing.cs:1586`'s gate chain: only a FULL solid, really-solid,
    // non-platform, non-ignoring, slope-drawing block with a half-brick
    // neighbour joins.
    #[test]
    fn half_brick_join_gates_match_the_csharp_chain() {
        // Stone (1) with a half-brick stone on the left joins.
        let mut w = join_world(&[(20, 20, 1)]);
        half_at(&mut w, 19, 20, 1);
        let t = *w.tile(20, 20).unwrap();
        let j = half_brick_join(&w, &t, 20, 20).expect("stone joins");
        assert!(j.left && !j.right && !j.smooth && !j.same_above);
        // The neighbour's halfBrick() bit is read WITHOUT an active test
        // (`:1586` reads `Main.tile[x-1, y].halfBrick()` directly).
        w.tile_mut(19, 20).unwrap().flags &= !tr_core::tile::FLAG_ACTIVE;
        assert!(half_brick_join(&w, &t, 20, 20).is_some());
        // No half-brick neighbours: no join.
        let w2 = join_world(&[(20, 20, 1), (19, 20, 1), (21, 20, 1)]);
        assert!(half_brick_join(&w2, w2.tile(20, 20).unwrap(), 20, 20).is_none());
        // Platforms (19) never join (`!TileID.Sets.Platforms`).
        let mut w3 = join_world(&[(20, 20, 19)]);
        half_at(&mut w3, 19, 20, 1);
        assert!(half_brick_join(&w3, w3.tile(20, 20).unwrap(), 20, 20).is_none());
        // NotReallySolid (10, the wooden beam) never joins.
        let mut w4 = join_world(&[(20, 20, 10)]);
        half_at(&mut w4, 19, 20, 1);
        assert!(half_brick_join(&w4, w4.tile(20, 20).unwrap(), 20, 20).is_none());
        // DontDrawTileSlopes + IgnoresNearbyHalfbricks (137, the detector).
        let mut w5 = join_world(&[(20, 20, 137)]);
        half_at(&mut w5, 19, 20, 1);
        assert!(half_brick_join(&w5, w5.tile(20, 20).unwrap(), 20, 20).is_none());
        // The block itself a half brick: no join (`!drawData.tileCache.halfBrick()`).
        let mut w6 = join_world(&[]);
        half_at(&mut w6, 20, 20, 1);
        half_at(&mut w6, 19, 20, 1);
        assert!(half_brick_join(&w6, w6.tile(20, 20).unwrap(), 20, 20).is_none());
    }

    // The smooth-border set shrinks the corner strip 4 → 2 (`:1601`/`:1616`),
    // and the both-sides column pick reads the tile ABOVE (`:1592-1595`).
    #[test]
    fn half_brick_join_smooth_and_same_above_flags() {
        // Grey brick (38) is in AllBlocksWithSmoothBorders…; stone is not.
        let mut w = join_world(&[(20, 20, 38)]);
        half_at(&mut w, 19, 20, 1);
        assert!(half_brick_join(&w, w.tile(20, 20).unwrap(), 20, 20).unwrap().smooth);
        let mut w2 = join_world(&[(20, 20, 1)]);
        half_at(&mut w2, 19, 20, 1);
        half_at(&mut w2, 21, 20, 1);
        let j = half_brick_join(&w2, w2.tile(20, 20).unwrap(), 20, 20).unwrap();
        assert!(j.left && j.right && !j.same_above);
        // An active same-kind full block above swaps 126 → 90.
        let t = w2.tile_mut(20, 19).unwrap();
        t.kind = 1;
        t.flags |= tr_core::tile::FLAG_ACTIVE;
        assert!(half_brick_join(&w2, w2.tile(20, 20).unwrap(), 20, 20).unwrap().same_above);
        // A bottom slope above (3/4) keeps 126 (`!bottomSlope()`).
        w2.tile_mut(20, 19).unwrap().set_slope(3);
        assert!(!half_brick_join(&w2, w2.tile(20, 20).unwrap(), 20, 20).unwrap().same_above);
        // A different kind above keeps 126.
        let t = w2.tile_mut(20, 19).unwrap();
        t.set_slope(0);
        t.kind = 0;
        assert!(!half_brick_join(&w2, w2.tile(20, 20).unwrap(), 20, 20).unwrap().same_above);
    }

    // The join rects (`TileDrawing.cs:1588-1627`) with frame (36, 18):
    // the bottom strip rides the cell frame + 8; the both-sides top strip
    // reads join column 126/90 at row addFrY ONLY.
    #[test]
    fn half_brick_join_rects_both_sides() {
        let mut t = tr_core::tile::Tile::default();
        t.frame_x = 36;
        t.frame_y = 18;
        let dd = tr_core::drawdata::TileDrawData::default();
        let both = HalfBrickJoin { left: true, right: true, smooth: false, same_above: false };
        assert_eq!(
            half_brick_join_rects(&t, &dd, &both),
            vec![(0.0, 8.0, 36, 26, 16, 8), (0.0, 0.0, 126, 0, 16, 8)]
        );
        let above = HalfBrickJoin { same_above: true, ..both };
        assert_eq!(half_brick_join_rects(&t, &dd, &above)[1], (0.0, 0.0, 90, 0, 16, 8));
        // addFrX/addFrY ride both rects (`:1590-1591`).
        let dd2 = tr_core::drawdata::TileDrawData { add_fr_x: 2, add_fr_y: 4, ..dd };
        assert_eq!(
            half_brick_join_rects(&t, &dd2, &both),
            vec![(0.0, 8.0, 38, 30, 16, 8), (0.0, 0.0, 128, 4, 16, 8)]
        );
    }

    // The one-sided arms: n = 4 (2 for smooth-border kinds); the LEFT arm's
    // corner strips carry addFr, the RIGHT arm's read FIXED sheet columns
    // (`:1607`/`:1610` vs `:1622`/`:1625`).
    #[test]
    fn half_brick_join_rects_one_sided_arms() {
        let mut t = tr_core::tile::Tile::default();
        t.frame_x = 36;
        t.frame_y = 18;
        let dd = tr_core::drawdata::TileDrawData::default();
        let left = HalfBrickJoin { left: true, right: false, smooth: false, same_above: false };
        assert_eq!(
            half_brick_join_rects(&t, &dd, &left),
            vec![
                (0.0, 8.0, 36, 26, 16, 8),
                (4.0, 0.0, 40, 18, 12, 16),
                (0.0, 0.0, 144, 0, 4, 8),
            ]
        );
        let right = HalfBrickJoin { left: false, right: true, smooth: false, same_above: false };
        assert_eq!(
            half_brick_join_rects(&t, &dd, &right),
            vec![
                (0.0, 8.0, 36, 26, 16, 8),
                (0.0, 0.0, 36, 18, 12, 16),
                (12.0, 0.0, 156, 0, 4, 8),
            ]
        );
        // Smooth kinds: n = 2, the extra 2x2 corner pixel (148 left / 156
        // right), and the right arm's strips ignore addFr entirely.
        let dd2 = tr_core::drawdata::TileDrawData { add_fr_x: 2, add_fr_y: 4, ..dd };
        let smooth_left = HalfBrickJoin { smooth: true, ..left };
        assert_eq!(
            half_brick_join_rects(&t, &dd2, &smooth_left),
            vec![
                (0.0, 8.0, 38, 30, 16, 8),
                (2.0, 0.0, 40, 22, 14, 16),
                (0.0, 0.0, 146, 4, 2, 8),
                (0.0, 0.0, 150, 4, 2, 2),
            ]
        );
        let smooth_right = HalfBrickJoin { smooth: true, ..right };
        assert_eq!(
            half_brick_join_rects(&t, &dd2, &smooth_right),
            vec![
                (0.0, 8.0, 38, 30, 16, 8),
                (0.0, 0.0, 38, 22, 14, 16),
                (14.0, 0.0, 158, 0, 2, 8),
                (14.0, 0.0, 156, 0, 2, 2),
            ]
        );
    }

    // Draw-order witness: the slope family sits in the C# `if`-chain order —
    // the platform arm (with its side-fill) BEFORE the generic slope columns,
    // the HasSlopeFrames arm between them, and the half-brick join arm BEFORE
    // the cell's own half-brick arm (`TileDrawing.cs:1515-1629`).
    #[test]
    fn slope_family_dispatch_matches_the_csharp_chain() {
        let src = include_str!("tiles.rs");
        let body = &src[src.find("pub(crate) fn draw_tiles(").unwrap()..];
        let body = &body[..body.find("#[cfg(test)]").unwrap()];
        let gate = body.find("SET_DONT_DRAW_TILE_SLOPES.has(tile.kind)").expect("flag2 gate");
        let platform = body.find("td::SET_PLATFORMS.has(tile.kind)").expect("the platform arm");
        let side_fill = body.find("draw_platform_side_fill(").expect("the side-fill hook");
        let slope_frames = body.find("SET_HAS_SLOPE_FRAMES.has(tile.kind)").expect("HasSlopeFrames");
        let columns = body.find("push_slope(r, pos").expect("the generic slope columns");
        let join = body.find("half_brick_join(world, tile, tx, ty)").expect("the join arm");
        let half = body.find("else if tile.half_brick()").expect("the half-brick arm");
        assert!(gate < platform, "flag2 gates the whole slope family (:1515)");
        assert!(platform < side_fill && side_fill < slope_frames, "the platform arm draws its fill first (:1517-1545)");
        assert!(slope_frames < columns, "HasSlopeFrames precedes the columns (:1546)");
        assert!(columns < join && join < half, "the join arm precedes the half-brick arm (:1586)");
        // The platform arm draws the FULL normal rect flat (never columns).
        let arm = &body[platform..slope_frames];
        assert!(arm.contains("flat_src_uv"), "the platform rect is normalTileRect (:1523)");
        assert!(!arm.contains("push_slope"), "the C# returns early (:1544)");
    }
}
#[cfg(test)]
mod w4_g75_retire_contract_tests {
    /// W4-g75 retire-contract (`a3093a66f`, folded) - the solid-layer
    /// `DrawTile_LiquidBehindTile` call feeds the manager handle
    /// (`TileDrawing.cs:3858`'s
    /// `Main.instance.waterfallManager.CheckForWaterfall(tileX, tileY)`):
    /// `draw_tiles` already carries `waterfall` (the `DoDraw_Waterfalls`
    /// seam), so the spout clause needs no new plumbing. The `inActive()`
    /// half of the same line rides the g80 lane's gate (af6cd44f).
    #[test]
    fn the_liquid_behind_call_feeds_the_waterfall_manager() {
        let src = include_str!("tiles.rs");
        let call = src
            .find("stats.quads += draw_tile_liquid_behind(")
            .expect("the solid-layer liquid-behind call");
        let tail = &src[call..call + 600];
        let end = tail.find(");").expect("the call's argument list closes");
        assert!(
            tail[..end].contains("waterfall,"),
            "the :3858 spout clause needs the manager handle at the call site"
        );
    }

    /// W4-g73 retire phase — the `CalculateWaterStyle` fountain head arm
    /// (`Main.cs:57390-57393`) reaches the tile pass: `draw_tiles` takes
    /// `active_fountain_color` and resolves `water_style` through
    /// `calculate_water_style_with_fountain` (the head arm BEFORE the
    /// biome walk), so a flowing fountain recolours the liquid/waterfall
    /// sheets exactly like `Main.waterStyle` (`Main.cs:57475`).
    #[test]
    fn the_tile_pass_water_style_carries_the_fountain_head_arm() {
        let src = include_str!("tiles.rs");
        let body = &src[src.find("pub(crate) fn draw_tiles(").unwrap()..];
        let body = &body[..body.find("#[cfg(test)]").unwrap()];
        assert!(
            body.contains("active_fountain_color: i32,"),
            "draw_tiles takes the scene scan's fountain colour"
        );
        let resolve = body
            .find("calculate_water_style_with_fountain(")
            .expect("the head-arm resolution");
        let tail = &body[resolve..resolve + 200];
        assert!(
            tail.contains("active_fountain_color,"),
            "the resolution feeds the fountain colour (the :57390 arm)"
        );
        // The frame's caller supplies the scene-metric value.
        let frame = include_str!("../app/frame/draw.rs");
        let call = frame
            .find("tile_stats = draw_tiles(")
            .expect("the frame's tile-pass call");
        let tail = &frame[call..call + 6000];
        let end = tail.find(");").expect("the call's argument list closes");
        assert!(
            tail[..end].contains("self.scene_metrics.active_fountain_color,"),
            "the frame feeds the scan's flowing-fountain colour"
        );
    }
}
