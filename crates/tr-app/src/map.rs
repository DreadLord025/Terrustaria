//! Minimap and full map (`Main.DrawMap`, `Terraria.Map.WorldMap`): reveals
//! `MapTile`s from the light map (`WorldMap.UpdateLighting`,
//! `LightingEngine.ExportToMiniMap`) and draws them from a world-sized texture
//! (one texel per map tile) at `Main.DrawMap`'s geometry.

use glam::Vec2;
use tr_core::lighting::LightMap;
use tr_core::map::{
    MapTile, clear_edges, get_map_tile_color, queue_update, reveal_rect, unlock_map_tile_pretty,
    update_map_lighting, update_type,
};
use tr_core::{TILE_PX, World};
use tr_render::SpriteRenderer;

use crate::assets;
use crate::player::Player;

/// `Main.cs:41101`: the scale the fullscreen map opens at.
pub const MAP_FULLSCREEN_SCALE_DEFAULT: f32 = 2.5;

/// `Main.DrawMap` scale clamp (`Main.cs:55387-55395`): floor
/// `screen_w / world_w * 0.599`, and anything above `31.2` becomes `31.18`.
pub fn map_scale_clamp(screen_w: f32, world_w: i32, proposed: f32) -> f32 {
    let min = screen_w / world_w as f32 * 0.599;
    let s = proposed.max(min);
    if s > 31.2 { 31.18 } else { s }
}

/// `Player.cs:25190-25197`: `scale * (1 + (delta_y / 120) * 0.3)`, `delta_y` in
/// wheel units (a notch is 120).
pub fn map_wheel_scale(scale: f32, delta_y: f32) -> f32 {
    scale * (1.0 + (delta_y / 120.0) * 0.3)
}

/// `Player.cs:24898-24908`: `MapZoomIn` → `* 1.05`, `MapZoomOut` → `* 0.95`.
pub fn map_key_scale(scale: f32, zoom_in: bool) -> f32 {
    if zoom_in { scale * 1.05 } else { scale * 0.95 }
}

/// `Main.cs:55398-55413`: X into `10..=maxTilesX-10`, Y into `10..=maxTilesY-19`.
pub fn map_clamp_pos(pos: [f32; 2], world: &World) -> [f32; 2] {
    [
        pos[0].clamp(10.0, (world.width - 10) as f32),
        pos[1].clamp(10.0, (world.height - 19) as f32),
    ]
}

/// `Main.cs:55355-55358`: the left-button drag. `delta` is the cursor's pixel
/// delta since the previous frame; the map centre moves
/// `-delta * 0.06255 * (16 / scale)` tiles (`num20 *= 0.06255f` then
/// `* (16f / mapFullscreenScale)`).
pub fn map_drag_delta(delta: [f32; 2], scale: f32) -> [f32; 2] {
    let s = 0.06255 * (16.0 / scale);
    [-delta[0] * s, -delta[1] * s]
}

/// One map quad: `dest` is a screen rect `[x, y, w, h]`, `uv` the texture subrect
/// `[u0, v0, u1, v1]` (normalised over the world-sized texture).
pub struct MapQuad {
    pub dest: [f32; 4],
    pub uv: [f32; 4],
}

/// The fullscreen map origin (`num`/`num2`, `Main.cs:55425-55430`): `pos` in tiles,
/// `scale` in px/tile, `screen` in px.
fn map_origin(pos: [f32; 2], scale: f32, screen: Vec2) -> [f32; 2] {
    [
        screen.x * 0.5 - pos[0] * scale + 10.0 * scale,
        screen.y * 0.5 - pos[1] * scale + 10.0 * scale,
    ]
}

/// `Main.DrawMap`'s fullscreen head-icon position (`Main.cs:55937-55944`).
/// `center_px` is `Main.player[k].Center` (world px; `gfxOffY` already added to Y).
/// `pos`/`scale` are `Main.mapFullscreenPos`/`Main.mapFullscreenScale`; `screen` is
/// the screen size in px. `num12`/`num13` are 0 on the fullscreen map
/// (`Main.cs:55302-55303`).
/// Returns the icon centre in screen px (the 84x84 head target is drawn centred here).
pub fn head_icon_pos(center_px: [f32; 2], pos: [f32; 2], scale: f32, screen: Vec2) -> [f32; 2] {
    // `Main.cs:55425-55430`: `num`/`num2`.
    let origin = map_origin(pos, scale, screen);
    // `Main.cs:55937-55944`, keeping the C# left-to-right order (`a/16*scale`).
    let x = center_px[0] / TILE_PX * scale + origin[0] - 6.0 - 10.0 * scale;
    let y = center_px[1] / TILE_PX * scale + origin[1] - 2.0 - (2.0 - scale / 5.0 * 2.0)
        - 10.0 * scale;
    [x, y]
}

/// `Main.DrawMap` fullscreen geometry (`Main.cs:55425-55430` + `55591-55607`): the
/// drawn tile window is `[10, 10] ..= [maxTilesX-11, maxTilesY-20]`.
pub fn map_full_quad(pos: [f32; 2], scale: f32, screen: Vec2, world: &World) -> MapQuad {
    let origin = map_origin(pos, scale, screen);
    let x0 = 10.0;
    let y0 = 10.0;
    let x1 = (world.width - 11) as f32;
    let y1 = (world.height - 20) as f32;
    MapQuad {
        dest: [origin[0], origin[1], (x1 - x0) * scale, (y1 - y0) * scale],
        uv: [
            x0 / world.width as f32,
            y0 / world.height as f32,
            x1 / world.width as f32,
            y1 / world.height as f32,
        ],
    }
}

/// `Images/Map` frame rect (`Main.cs:55431-55484`); the asset is 928x248. Returns
/// `[x, y, w, h]` in screen pixels.
pub fn map_frame_rect(pos: [f32; 2], scale: f32, screen: Vec2, world: &World) -> [f32; 4] {
    let origin = map_origin(pos, scale, screen);
    // `(float)(Main.maxTilesX / 840)`: integer division first.
    let mut n = (world.width / 840) as f32 * scale;
    let (mut x, mut y) = (origin[0], origin[1]);
    let mut fw = 928.0;
    let mut fh = 248.0;
    match world.width {
        8400 => {
            n *= 0.999;
            x -= 40.6 * n;
            y -= 5.0 * n;
            fw = (fw - 8.045) * n;
            fh = (fh + 0.12) * n;
            if n < 1.2 {
                fh += 1.0;
            }
        }
        6400 => {
            n *= 1.09;
            x -= 38.8 * n;
            y -= 3.85 * n;
            fw = (fw - 13.6) * n;
            fh = (fh - 6.92) * n;
            if n < 1.2 {
                fh += 2.0;
            }
        }
        6300 => {
            n *= 1.09;
            x -= 39.8 * n;
            y -= 4.08 * n;
            fw = (fw - 26.69) * n;
            fh = (fh - 6.92) * n;
            if n < 1.2 {
                fh += 2.0;
            }
        }
        4200 => {
            n *= 0.998;
            x -= 37.3 * n;
            y -= 1.7 * n;
            fw = (fw - 16.0) * n;
            fh = (fh - 8.31) * n;
        }
        _ => {}
    }
    [x, y, fw, fh]
}

/// Minimap geometry (`Main.cs:55503-55546`): a 240x240 panel anchored at
/// `minimap::minimap_origin_raw` (`Main.cs:55508-55511`) showing `240 / scale`
/// tiles around the view centre. `scale` is `Main.mapMinimapScale` (the zoom
/// axis, clamped `0.2..3`, `Main.cs:55515-55522`) and `map_scale` is
/// `Main.MapScale` (clamped `[0.5,1]`, `Main.cs:4494`): the whole tile pass is
/// drawn inside `Matrix.CreateScale(Main.MapScale)` (`Main.cs:55309-55329`), so
/// the destination rect is the raw one multiplied by it. The full map is a
/// different scale (`Main.mapFullscreenScale`, `Main.cs:55390`) and does not
/// read `MapScale`.
pub fn map_minimap_quad(
    center_tile: [f32; 2],
    scale: f32,
    map_scale: f32,
    screen: Vec2,
    world: &World,
) -> MapQuad {
    let scale = scale.clamp(0.2, 3.0);
    // `Main.cs:55508-55511`, consolidated onto `minimap::minimap_origin_raw`
    // (identical to the former hardcode `[screen.x - 292.0, 90.0]` at
    // `MapScale = 1` — pinned by `minimap_quad_matches_the_panel` below).
    let panel = crate::ui::minimap::minimap_origin_raw(screen, map_scale);
    let win = 240.0 / scale;
    let a = center_tile[0].floor() - win * 0.5;
    let b = center_tile[1].floor() - win * 0.5;
    let x0 = a.max(10.0);
    let y0 = b.max(10.0);
    let x1 = (a + win).min((world.width - 10) as f32);
    let y1 = (b + win).min((world.height - 19) as f32);
    // `num10`/`num11`: the sub-tile offset of the (fractional) view centre.
    let sub = [
        -(center_tile[0] - center_tile[0].floor()) * scale,
        -(center_tile[1] - center_tile[1].floor()) * scale,
    ];
    MapQuad {
        // The raw rect, then the `MapScale` batch transform (`Main.cs:55309-55329`).
        dest: crate::ui::minimap::scale_rect(
            [
                panel[0] + (x0 - a) * scale + sub[0],
                panel[1] + (y0 - b) * scale + sub[1],
                (x1 - x0) * scale,
                (y1 - y0) * scale,
            ],
            map_scale,
        ),
        uv: [
            x0 / world.width as f32,
            y0 / world.height as f32,
            x1 / world.width as f32,
            y1 / world.height as f32,
        ],
    }
}

/// `Main.DrawMapFullscreenBackground` (`Main.cs:56342-56444`). `index` is the
/// `TextureAssets.MapBGs` index (`0` = `MapBG1`); `sky_tint` is true when the
/// original tinted the backdrop with `Main.ColorOfTheSkies` (`Main.cs:56395`),
/// i.e. the surface branch.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MapBg {
    pub index: i32,
    pub sky_tint: bool,
}

/// `Main.DrawMapFullscreenBackground` (`Main.cs:56342-56444`): which `MapBG`
/// backdrop the fullscreen map sits on.
///
/// `camera` is the view **centre** in px and `screen` the px size, so
/// `camera - screen/2` is vanilla's `Main.screenPosition` (the camera top-left).
/// `player_center_tile` is `Main.player[myPlayer].Center` in tiles; the caller
/// falls back to the view-centre tile when there is no player (`Main.player` is
/// never null in vanilla, but our engine has `TR_NOPLAYER`).
///
/// Returns `None` for the vanilla early return
/// (`!WorldGen.InWorld(playerTile, 10)`, `Main.cs:56347-56350`).
pub fn map_bg_index(
    world: &World,
    camera: Vec2,
    screen: Vec2,
    player_center_tile: (i32, i32),
) -> Option<MapBg> {
    let (tx, ty) = player_center_tile;
    // `WorldGen.InWorld(x, y, 10)` (`WorldGen.cs`): `10 <= x < w - 10`,
    // `10 <= y < h - 10`.
    if !(tx >= 10 && tx < world.width - 10 && ty >= 10 && ty < world.height - 10) {
        return None;
    }
    // `Main.cs:56347`: a null tile also returns. `InWorld` already implies
    // in-bounds, but keep the explicit check for fidelity.
    let t = world.tile(tx, ty)?;
    let wall = t.wall;
    // `Main.screenPosition` is the camera top-left.
    let sp = camera - screen * 0.5;
    let snow = crate::biome::is_snow_biome(world, camera);
    let desert = crate::biome::is_desert_biome(world, camera);
    // `SceneMetrics.cs:669`: `TileCenter.Y <= Main.UnderworldLayer &&
    // TileCenter.Y > Main.rockLayer`, `Main.UnderworldLayer = maxTilesY - 200`.
    let rock = ty > world.rock_layer as i32 && ty <= world.height - 200;
    // `SceneMetrics.cs:665`: `TileCenter.Y <= Main.worldSurface * 0.3499999940395355`
    // (i.e. `(double)0.35f`).
    let sky = (ty as f64) <= world.world_surface * (0.35_f32 as f64);
    // `Main.cs:56396`: `(int)((screenPosition.X + (float)(screenWidth / 2)) / 16f)`.
    // `screenWidth / 2` is **integer** division in C#.
    let num2 = ((sp.x + (screen.x as i32 / 2) as f32) / TILE_PX as f32) as i32;
    // `Main.cs:56413`: `(screenPosition.Y / 16f) < worldSurface + 10` and near an
    // ocean edge.
    let beach = (sp.y as f64 / TILE_PX as f64) < world.world_surface + 10.0
        && (num2 < 380 || num2 > world.width - 380);

    let mut index = -1;
    // `color` stays `Color.White` in every branch except the surface one.
    let mut sky_tint = false;
    if sp.y > ((world.height - 232) * 16) as f32 {
        // `Main.cs:56352-56354`.
        index = 2;
    } else if false {
        // not simulated: ZoneDungeon (`Main.cs:56356`)
        index = 4;
    } else if wall == 87 {
        // `Main.cs:56360-56362`.
        index = 13;
    } else if (sp.y as f64) > world.world_surface * TILE_PX as f64 {
        // `Main.cs:56364-56388`.
        index = match wall {
            86 | 108 => 15,  // `Main.cs:56368-56371`
            180 | 184 => 16, // `Main.cs:56372-56375`
            178 | 183 => 17, // `Main.cs:56376-56379`
            62 | 263 => 18,  // `Main.cs:56380-56383`
            _ => {
                if false {
                    // not simulated: ZoneGlowshroom (`Main.cs:56385`)
                    20
                } else if false {
                    // not simulated: ZoneCorrupt
                    if desert {
                        39
                    } else if snow {
                        33
                    } else {
                        22
                    }
                } else if false {
                    // not simulated: ZoneCrimson
                    if desert {
                        40
                    } else if snow {
                        34
                    } else {
                        23
                    }
                } else if false {
                    // not simulated: ZoneHallow
                    if desert {
                        41
                    } else if snow {
                        35
                    } else {
                        21
                    }
                } else if snow {
                    3
                } else if false {
                    // not simulated: ZoneJungle
                    12
                } else if desert {
                    14
                } else if rock {
                    31
                } else {
                    1
                }
            }
        };
    } else if false {
        // not simulated: ZoneGlowshroom (`Main.cs:56389-56391`)
        index = 19;
    } else {
        // `Main.cs:56393-56395`: the surface branch tints with
        // `Main.ColorOfTheSkies`.
        sky_tint = true;
        if sky {
            // `Main.cs:56397-56399`: ZoneSkyHeight.
            index = 32;
        } else if false {
            // not simulated: ZoneCorrupt (`Main.cs:56401-56403`)
            index = if desert { 36 } else { 5 };
        } else if false {
            // not simulated: ZoneCrimson (`Main.cs:56405-56407`)
            index = if desert { 37 } else { 6 };
        } else if false {
            // not simulated: ZoneHallow (`Main.cs:56409-56411`)
            index = if desert { 38 } else { 7 };
        } else if beach {
            // `Main.cs:56413-56415`.
            index = 10;
        } else if snow {
            // `Main.cs:56417-56419`.
            index = 11;
        } else if false {
            // not simulated: ZoneJungle (`Main.cs:56421-56423`)
            index = 8;
        } else if desert {
            // `Main.cs:56425-56427`.
            index = 9;
        } else if false {
            // not simulated: bloodMoon (`Main.cs:56429-56432`: color *= 2; num = 25)
        } else if false {
            // not simulated: ZoneGraveyard (`Main.cs:56434-56436`)
            index = 26;
        }
    }
    // `Main.cs:56439-56442`: `num == -1` keeps the `MapBGs[0]` default.
    Some(MapBg {
        index: if index < 0 { 0 } else { index },
        sky_tint,
    })
}

/// `Main.cs:56443`: stretches `MapBG{index+1}` (or the `sky`-tinted surface
/// variant) over the whole screen. `sky` is `[ColorOfTheSkies.r, .g, .b, 1.0]`;
/// unused unless `bg.sky_tint`.
pub fn draw_map_background(r: &mut SpriteRenderer, bg: MapBg, screen: Vec2, sky: [f32; 4]) {
    let name = format!("MapBG{}", bg.index + 1);
    let Some(uv) = r.sheets.get(&name).copied() else {
        return;
    };
    let color = if bg.sky_tint { sky } else { [1.0; 4] };
    r.push_map_back_sprite([0.0, 0.0], [screen.x, screen.y], uv, color);
}

pub struct WorldMap {
    /// Row-major `y * width + x`, matching `World::index`.
    pub tiles: Vec<MapTile>,
    /// The tile-grid dims (`tiles`' row-major shape); `WorldMap.QueueUpdate`
    /// bounds-checks against them (`WorldMap.cs:96`).
    tiles_w: usize,
    tiles_h: usize,
    pub open: bool,
    /// `Main.mapFullscreenScale` (px per tile) and `Main.mapFullscreenPos` (tiles).
    pub scale: f32,
    pub pos: [f32; 2],
    /// `Main.mapMinimapScale` (`Main.cs:66751`), fixed at 1.05 (no minimap-zoom UI yet).
    pub minimap_scale: f32,
    timer: f32,
    /// Tiles revealed/changed since the last `take_dirty` (Task 6 uploads these).
    dirty: Vec<(i32, i32)>,
    /// The world-sized map texture exists (`ensure_map_texture` has run).
    uploaded: bool,
    /// A `.map` was loaded into `tiles`: upload the whole texture on the next
    /// `prepare` (`MapRenderer.CheckMapTargets` uploads every revealed tile on
    /// the first frame).
    needs_full_upload: bool,
    /// `Main.resetMapFull`: recentre on the next `prepare`.
    reset_pos: bool,
    /// `PingMapLayer`'s pool (`PingMapLayer.cs:16-19`): the ping positions
    /// in world tiles + their age in seconds (the C# stamps `DateTime.Now`
    /// and compares `TotalSeconds > 15`).
    pub pings: Vec<([f32; 2], f32)>,
}

impl WorldMap {
    pub fn new(world: &World) -> Self {
        Self {
            tiles: vec![MapTile::default(); (world.width as usize) * (world.height as usize)],
            tiles_w: world.width as usize,
            tiles_h: world.height as usize,
            open: false,
            scale: MAP_FULLSCREEN_SCALE_DEFAULT,
            pos: [0.0, 0.0],
            minimap_scale: 1.05,
            timer: 1.0,
            dirty: Vec::new(),
            uploaded: false,
            needs_full_upload: false,
            reset_pos: true,
            pings: Vec::new(),
        }
    }

    /// Reveals 4x/second from `light` — the `LightingEngine.ExportToMiniMap`
    /// rect (`LightingEngine.cs:246-262`): the engine's processed area
    /// shrunk 28 px on every side, intersected with the world bounds inset
    /// 40 and the scene area (`MapHelper.sceneArea`,
    /// `MapHelper.cs:1819` = the player tile centre's
    /// `SceneMetrics.ZoneScanSize` box — `SceneMetrics.cs:917-919`'s
    /// 1920x1200 / 16 + the 25 px padding). The port used to reveal a fixed
    /// player-centred 75x47 box (the audit's D10) — a smaller, trail-prone
    /// window than the vanilla's view-derived one.
    pub fn update(&mut self, world: &World, player: Option<&Player>, light: &LightMap, dt: f32) {
        self.timer += dt;
        if self.timer < 0.25 {
            return;
        }
        self.timer = 0.0;
        // `:256`: `MapHelper.sceneArea` is the scene-metrics scan's
        // player-centred box — its C# static defaults to `Rectangle.Empty`,
        // so without a scan (no player) the intersect is empty and nothing
        // reveals.
        let Some(p) = player else {
            return;
        };
        // `LightingEngine.cs:252`: the processed area minus 28 px per side.
        let (ax, ay, aw, ah) = (light.x0, light.y0, light.w, light.h);
        let mut x0 = ax + 28;
        let mut y0 = ay + 28;
        let mut x1 = ax + aw - 28;
        let mut y1 = ay + ah - 28;
        // `:253-255`: the world bounds inset 40.
        x0 = x0.max(40);
        y0 = y0.max(40);
        x1 = x1.min(world.width - 40);
        y1 = y1.min(world.height - 40);
        // The player tile centre's 169x124 zone-scan box
        // (`SceneMetrics.cs:919`'s `ZoneScanSize`).
        let cx = (p.x / TILE_PX) as i32;
        let cy = (p.y / TILE_PX) as i32;
        let (hw, hh) = (169 / 2, 124 / 2);
        x0 = x0.max(cx - hw);
        y0 = y0.max(cy - hh);
        x1 = x1.min(cx + hw);
        y1 = y1.min(cy + hh);
        if x0 > x1 || y0 > y1 {
            return;
        }
        let changed = reveal_rect(world, &mut self.tiles, light, x0, y0, x1, y1);
        self.dirty.extend(changed);
    }

    /// `WorldMap.UpdateType` (`WorldMap.cs:65-93`) — see the tr-core free fn.
    pub fn update_type(&mut self, world: &World, x: i32, y: i32) -> bool {
        crate::map::update_type(world, &mut self.tiles, x, y)
    }

    /// `WorldMap.QueueUpdate` (`WorldMap.cs:95-105`) — see the tr-core free fn.
    pub fn queue_update(&mut self, x: i32, y: i32) -> bool {
        crate::map::queue_update(
            &mut self.tiles,
            x,
            y,
            self.tiles_w as i32,
            self.tiles_h as i32,
        )
    }

    /// `WorldMap.ClearEdges` (`WorldMap.cs:260-285`) — the map load's edge
    /// clear (`WorldMap.cs:210`).
    pub fn clear_edges(&mut self, world: &World) {
        crate::map::clear_edges(&mut self.tiles, world.width, world.height);
    }

    /// The edit-queue drain's push (the `MapRenderer.QueueChange` analog —
    /// the frame routes `MapUpdateQueue::update`'s changed tiles here).
    pub(crate) fn push_dirty(&mut self, tiles: Vec<(i32, i32)>) {
        self.dirty.extend(tiles);
    }

    /// `PingMapLayer.Add` (`PingMapLayer.cs:47-51`): up to 100 pings; the
    /// position is world tiles. The C# caller rides the ping net message —
    /// the port has no multiplayer, so the pool stays empty in-game (the
    /// seam exists for the net task).
    pub fn add_ping(&mut self, pos: [f32; 2]) {
        if self.pings.len() < 100 {
            self.pings.push((pos, 0.0));
        }
    }

    /// `PingMapLayer.Draw`'s expiry (`PingMapLayer.cs:37-40`): the pings
    /// older than 15 s drop.
    pub fn age_pings(&mut self, dt: f32) {
        for p in self.pings.iter_mut() {
            p.1 += dt;
        }
        self.pings.retain(|p| p.1 <= 15.0);
    }

    /// Tiles changed since the previous call (`prepare` uploads these).
    pub fn take_dirty(&mut self) -> Vec<(i32, i32)> {
        std::mem::take(&mut self.dirty)
    }

    /// Loads the explored tiles from `<world>.map` (`MapHelper.LoadMap`, invoked
    /// at world load). Returns `false` when the file is absent or does not match
    /// this world. The whole map texture is re-uploaded on the next `prepare`.
    pub fn load(&mut self, world: &World, path: &std::path::Path) -> bool {
        if !load_map_file(world, &mut self.tiles, path).unwrap_or(false) {
            return false;
        }
        // `WorldMap.Load`'s edge clear (`WorldMap.cs:210`): the 40-tile
        // world border bands come back unrevealed.
        self.clear_edges(world);
        // Drop any pending single-tile writes; the full upload replaces them.
        self.dirty.clear();
        self.needs_full_upload = true;
        true
    }

    /// `KeyM`: opening also arms the recentre (`Main.resetMapFull = true`) and
    /// resets the scale (`Main.mapFullscreenScale = 2.5`, `Main.cs:41101`).
    pub fn set_open(&mut self, open: bool) {
        self.open = open;
        if open {
            self.reset_pos = true;
            self.scale = MAP_FULLSCREEN_SCALE_DEFAULT;
        }
    }

    /// Per-frame prep: clamp `scale`/`pos`, recentre on the view centre if armed,
    /// then upload the tiles in `take_dirty()` into the map texture.
    ///
    /// `center_tile` is the **view-centre** tile (`[camera.x / TILE_PX,
    /// camera.y / TILE_PX]`, i.e. `(screenPosition + screenSize/2)/16`); it is only
    /// read when a recentre is armed.
    pub fn prepare(
        &mut self,
        r: &mut SpriteRenderer,
        gpu: &tr_render::Gpu,
        world: &World,
        screen: Vec2,
        center_tile: [f32; 2],
    ) {
        // `Main.cs:55387-55395`.
        self.scale = map_scale_clamp(screen.x, world.width, self.scale);
        // `Main.cs:55398-55413`.
        self.pos = map_clamp_pos(self.pos, world);
        // `Main.cs:55416-55424`: `resetMapFull` overrides the clamp with the unclamped
        // view centre `(screenPosition + screenSize/2)/16f`.
        if self.reset_pos {
            self.pos = center_tile;
            self.reset_pos = false;
        }

        // `MapRenderer.CheckMapTargets` (`MapRenderer.cs:357-378`).
        if !self.uploaded {
            r.ensure_map_texture(gpu, world.width as u32, world.height as u32);
            self.uploaded = true;
        }
        // A loaded `.map`: one bulk upload of every revealed tile, matching the
        // first-frame `MapRenderer.DrawChangesToMap` sweep.
        if self.needs_full_upload {
            self.needs_full_upload = false;
            let (w, h) = (world.width as u32, world.height as u32);
            let mut rgba = vec![[0u8; 4]; (w * h) as usize];
            for y in 0..world.height {
                for x in 0..world.width {
                    let t = self.tiles[world.index(x, y)];
                    if t.light > 0 {
                        let [cr, cg, cb] = get_map_tile_color(t, x, y);
                        rgba[(y as u32 * w + x as u32) as usize] = [cr, cg, cb, 255];
                    }
                }
            }
            r.write_map_region(gpu, 0, 0, w, h, &rgba);
        }
        // `MapRenderer.DrawChangesToMap`: one texel per revealed map tile.
        for (x, y) in self.take_dirty() {
            if !world.in_bounds(x, y) {
                continue;
            }
            let t = self.tiles[world.index(x, y)];
            let rgba = if t.light > 0 {
                let [r, g, b] = get_map_tile_color(t, x, y);
                [[r, g, b, 255]]
            } else {
                [[0, 0, 0, 0]]
            };
            r.write_map_region(gpu, x as u32, y as u32, 1, 1, &rgba);
        }
    }

    /// `Main.DrawMap` minimap branch (`Main.cs:55503-55546`): the minimap is centred
    /// on the **view centre** (`Main.cs:55537-55538`), so the caller passes that
    /// tile, and `MapRenderer.DrawMap` (`MapRenderer.cs:380`) draws the one tile
    /// quad from `map_minimap_quad`'s `MapScale`-scaled destination. `map_scale` is
    /// `Main.MapScale` (`Main.cs:4494`/`:67297`) — the zoom axis
    /// `self.minimap_scale` is a separate multiplier (`Main.cs:55515-55522`).
    pub fn draw_minimap(
        &self,
        r: &mut SpriteRenderer,
        world: &World,
        screen: Vec2,
        center_tile: [f32; 2],
        map_scale: f32,
    ) {
        let q = map_minimap_quad(center_tile, self.minimap_scale, map_scale, screen, world);
        r.push_map_sprite(
            [q.dest[0], q.dest[1]],
            [q.dest[2], q.dest[3]],
            q.uv,
            [1.0; 4],
        );
    }

    /// `Main.DrawMap` fullscreen branch (`Main.cs:55425-55609`): the `Images/Map`
    /// frame behind the tiles (Task 7 inserts the `MapBG` backdrop before the frame).
    pub fn draw_full(
        &self,
        r: &mut SpriteRenderer,
        _lookups: &assets::Lookups,
        world: &World,
        screen: Vec2,
    ) -> ([f32; 2], f32) {
        // The callers need the overlay context's offset/scale
        // (`Main.cs:55996`'s `MapIcons.Draw` arguments) — return the pair.
        // `Main.cs:55431-55485`: the wooden scroll frame (928x248).
        if let Some(uv) = r.uv("Map", 0.0, 0.0, 928.0, 248.0) {
            let rect = map_frame_rect(self.pos, self.scale, screen, world);
            r.push_map_back_sprite(
                [rect[0], rect[1]],
                [rect[2], rect[3]],
                uv,
                [1.0; 4],
            );
        }
        // `Main.DrawMap`'s `MapRenderer.DrawMap` call (`Main.cs:55609`).
        let q = map_full_quad(self.pos, self.scale, screen, world);
        r.push_map_sprite(
            [q.dest[0], q.dest[1]],
            [q.dest[2], q.dest[3]],
            q.uv,
            [1.0; 4],
        );
        // `Main.cs:55996`: the overlay offset is the map's top-left px minus
        // the 10-px frame inset, and the draw scale rides the map scale
        // (`num77`'s formula — `Main.cs:55975-55985`).
        (
            [q.dest[0] - 10.0 * self.scale, q.dest[1] - 10.0 * self.scale],
            self.scale,
        )
    }
}

// ---------------------------------------------------------------------------
// `.map` persistence (`MapHelper.InternalSaveMap` / `LoadMapVersion2`). The
// byte layout lives in `tr-core`; this crate owns the raw-deflate wrapping and
// the file IO, since `flate2` is a `tr-app`-only dependency.
// ---------------------------------------------------------------------------

/// Writes header + option tables + deflated body (`MapHelper.InternalSaveMap`).
pub fn save_map_file(
    world: &World,
    map: &[MapTile],
    path: &std::path::Path,
) -> std::io::Result<()> {
    use std::io::Write;
    let header = tr_core::map::write_map_header(world);
    let tables = tr_core::map::write_option_tables();
    let body = tr_core::map::encode_tiles(map, world.width as usize, world.height as usize);
    // The C# `DeflateStream` is raw deflate (no zlib/gzip wrapper); its
    // `CompressionMode.Compress` ctor default is `CompressionLevel.Optimal`
    // (zlib level 6) (`MapHelper.cs:3047`).
    let mut encoder = flate2::write::DeflateEncoder::new(Vec::new(), flate2::Compression::new(6));
    encoder.write_all(&body)?;
    let deflated = encoder.finish()?;

    let mut out = Vec::with_capacity(header.len() + tables.len() + deflated.len());
    out.extend_from_slice(&header);
    out.extend_from_slice(&tables);
    out.extend_from_slice(&deflated);
    std::fs::write(path, out)
}

/// Loads a `.map` sidecar into `map`.
///
/// `Ok(false)` when the file is absent, unreadable, truncated, has a foreign
/// version, or its world seed/size do not match `world` (`MapHelper.cs:3763`).
pub fn load_map_file(
    world: &World,
    map: &mut [MapTile],
    path: &std::path::Path,
) -> std::io::Result<bool> {
    use std::io::Read;
    let Ok(bytes) = std::fs::read(path) else {
        return Ok(false);
    };
    let Some(prefix) = tr_core::map::read_map_prefix(&bytes)? else {
        return Ok(false);
    };
    let header = &prefix.header;
    if header.seed != world.seed as i32
        || header.width != world.width
        || header.height != world.height
    {
        return Ok(false);
    }
    let Some(deflated) = bytes.get(prefix.body_offset..) else {
        return Ok(false);
    };
    let mut decoder = flate2::read::DeflateDecoder::new(deflated);
    let mut body = Vec::new();
    if decoder.read_to_end(&mut body).is_err() {
        return Ok(false);
    }
    let tiles = match tr_core::map::decode_tiles(
        &body,
        header,
        &prefix.tile_options,
        &prefix.wall_options,
        world.width as usize,
        world.height as usize,
        world.world_surface,
        world.rock_layer,
    ) {
        Ok(tiles) => tiles,
        Err(_) => return Ok(false),
    };
    if map.len() != tiles.len() {
        return Ok(false);
    }
    map.copy_from_slice(&tiles);
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tr_core::map::{color_table, read_map_prefix, MapTile};

    fn test_world() -> World {
        let mut w = World::new("Terrustaria".into(), 40, 40);
        w.seed = 0x1234_5678_9abc_def0;
        w.world_surface = 10.0;
        w.rock_layer = 18.0;
        w
    }

    fn put(map: &mut [MapTile], width: usize, x: usize, y: usize, tile: MapTile) {
        map[y * width + x] = tile;
    }

    /// The same 40x40 map as the `tr-core` round-trip test.
    fn test_tiles(width: usize, height: usize) -> Vec<MapTile> {
        let t = color_table();
        let mut map = vec![MapTile::default(); width * height];
        for x in 4..=6 {
            put(&mut map, width, x, 0, MapTile::create(t.tile_position + 2, 255, 0));
        }
        for (x, light) in [(7usize, 30u8), (8, 60), (9, 90)] {
            put(&mut map, width, x, 0, MapTile::create(t.tile_position + 3, light, 0));
        }
        put(&mut map, width, 10, 0, MapTile::create(t.tile_position + 4, 100, 7));
        put(&mut map, width, 11, 0, MapTile::create(t.tile_position + 300, 255, 0));
        put(&mut map, width, 12, 0, MapTile::create(t.wall_position, 200, 0));
        put(&mut map, width, 13, 0, MapTile::create(t.liquid_position, 200, 0));
        put(&mut map, width, 14, 0, MapTile::create(t.liquid_position + 3, 200, 0));
        map
    }

    fn unique_dir(name: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!("tr-map-test-{}-{name}", std::process::id()))
    }

    #[test]
    fn map_round_trips_through_a_file() {
        let world = test_world();
        let original = test_tiles(40, 40);
        let dir = unique_dir("roundtrip");
        std::fs::create_dir_all(&dir).unwrap();
        let path_a = dir.join("a.map");
        let path_b = dir.join("b.map");
        save_map_file(&world, &original, &path_a).unwrap();

        let mut loaded = vec![MapTile::default(); original.len()];
        assert!(load_map_file(&world, &mut loaded, &path_a).unwrap());
        assert_eq!(loaded, original);

        // Re-saving the loaded map must be byte-identical (deterministic header +
        // deflate).
        save_map_file(&world, &loaded, &path_b).unwrap();
        assert_eq!(
            std::fs::read(&path_a).unwrap(),
            std::fs::read(&path_b).unwrap()
        );
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn truncated_file_is_rejected() {
        let world = test_world();
        let original = test_tiles(40, 40);
        let dir = unique_dir("truncated");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("cut.map");
        save_map_file(&world, &original, &path).unwrap();
        let bytes = std::fs::read(&path).unwrap();
        std::fs::write(&path, &bytes[..10]).unwrap();

        let mut loaded = vec![MapTile::default(); original.len()];
        assert!(!load_map_file(&world, &mut loaded, &path).unwrap());
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn mismatched_world_is_rejected() {
        let world = test_world();
        let original = test_tiles(40, 40);
        let dir = unique_dir("mismatch");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("other.map");
        save_map_file(&world, &original, &path).unwrap();

        let mut other = World::new("Other".into(), 40, 40);
        other.seed = world.seed ^ 0xff;
        let mut loaded = vec![MapTile::default(); original.len()];
        assert!(!load_map_file(&other, &mut loaded, &path).unwrap());
        std::fs::remove_dir_all(&dir).ok();
    }

    // ---- Task 9: `.map` persistence (`WorldMap::load` + `persist::map_path`) ----

    /// A successful load copies the tiles and arms the whole-texture upload; a
    /// missing file changes nothing.
    #[test]
    fn loading_a_saved_map_arms_the_full_upload() {
        let world = test_world();
        let saved = test_tiles(40, 40);
        let dir = unique_dir("wmload");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("w.map");
        save_map_file(&world, &saved, &path).unwrap();

        let mut wm = WorldMap::new(&world);
        assert!(!wm.needs_full_upload);
        assert!(wm.load(&world, &path));
        // `WorldMap.cs:210`: the load clears the 40-tile edge bands after
        // the file body — the interior compares, the bands are wiped.
        let mut expected = saved.clone();
        crate::map::clear_edges(&mut expected, world.width, world.height);
        assert_eq!(wm.tiles, expected);
        assert!(wm.needs_full_upload);

        let mut fresh = WorldMap::new(&world);
        assert!(!fresh.load(&world, std::path::Path::new("does-not-exist.map")));
        assert!(!fresh.needs_full_upload);
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    #[ignore]
    fn loads_a_real_vanilla_map() {
        let Some(path) = crate::env::var_raw("TR_VANILLA_MAP") else {
            return;
        };
        let bytes = std::fs::read(&path).expect("read map");
        let prefix = read_map_prefix(&bytes)
            .expect("no error")
            .expect("vanilla prefix must parse");
        let mut world = World::new(
            prefix.world_name.clone(),
            prefix.header.width,
            prefix.header.height,
        );
        world.seed = prefix.header.seed as u32 as u64;
        // Only used for the sky/dirt/rock bands; irrelevant to the revealed count.
        world.world_surface = 300.0;
        world.rock_layer = 600.0;
        let mut map = vec![MapTile::default(); (world.width * world.height) as usize];
        assert!(load_map_file(&world, &mut map, std::path::Path::new(&path)).unwrap());
        assert!(
            map.iter().filter(|t| t.light > 0).count() > 0,
            "vanilla map decoded to no revealed tiles"
        );
    }

    // ---- Task 5: reveal from the light map ----

    use crate::player::Player;
    use tr_core::lighting::{LightMap, LightParams};
    use tr_core::tile::{FLAG_ACTIVE, Tile};

    /// The `ground_world` from `player.rs`: a 64x128 world with a solid band
    /// `y in 60..128`, spawn at `(10, 40)`.
    fn ground_world() -> World {
        // The world is 200 wide so the reveal rect's world-bounds inset 40
        // (`LightingEngine.cs:253-255`) leaves a live window around the
        // spawn column.
        let mut w = World::new("t".into(), 200, 128);
        for x in 0..200 {
            for y in 60..128 {
                let i = w.index(x, y);
                w.tiles[i] = Tile {
                    kind: 1,
                    ..Default::default()
                };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        w.spawn_x = 10;
        w.spawn_y = 40;
        w
    }

    #[test]
    fn update_type_refreshes_a_queued_tile_after_an_edit() {
        // `WorldMap.cs:65-93`: the queued flag arms a re-derive; the tile
        // keeps its light and picks the new world state.
        let mut world = ground_world();
        let px = 100usize;
        let py = 40usize;
        // A revealed water-free stone tile at (100, 40).
        let i = world.index(px as i32, py as i32);
        world.tiles[i] = Tile {
            kind: 1,
            ..Default::default()
        };
        world.tiles[i].flags |= FLAG_ACTIVE;
        let mut m = WorldMap::new(&world);
        // The map tile derives from the world (the colour included) — a
        // hand-built create(1, 200, 0) would mismatch on the colour.
        assert!(crate::map::update_map_lighting(&world, &mut m.tiles, 100, 40, 200));
        // Queue + drain: nothing changed yet (the world matches).
        let mut q = MapUpdateQueue::default();
        q.add_tile(&mut m, px as i32, py as i32);
        assert!(q.update(&world, &mut m).is_empty(), "no change yet");
        // The tile becomes air: the queued refresh re-derives to the
        // background (the type flips, the light stays). The first drain
        // consumed the entry, so the edit re-queues.
        world.tiles[i] = Tile::default();
        q.add_tile(&mut m, 100, 40);
        let changed = q.update(&world, &mut m);
        assert_eq!(changed, vec![(px as i32, py as i32)]);
        assert_ne!(m.tiles[i].ty, 1, "the type re-derived");
        assert_eq!(m.tiles[i].light, 200, "the light kept");
    }

    #[test]
    fn queue_update_refuses_dark_and_already_queued_tiles() {
        // `WorldMap.cs:99-102`.
        let world = ground_world();
        let mut m = WorldMap::new(&world);
        let i = world.index(100, 40);
        m.tiles[i] = MapTile::create(1, 0, 0); // dark
        assert!(!m.queue_update(100, 40));
        m.tiles[i] = MapTile::create(1, 200, 0);
        assert!(m.queue_update(100, 40));
        assert!(!m.queue_update(100, 40), "the dedupe flag");
    }

    #[test]
    fn unlock_map_tile_pretty_falls_off_and_skips_solids_and_lava() {
        // `WorldMap.cs:135-174`: the 11x11 Manhattan falloff, the solid and
        // the non-lava-liquid skips, the wall 0.8 / underground 0.7 tints.
        let mut world = ground_world();
        world.world_surface = 70.0; // the falloff rows sit above it
        let mut m = WorldMap::new(&world);
        // The air cell at (100, 60) reveals itself at 255 and its ring by
        // the Manhattan falloff — (100, 59) is one step out: 255*4/5 = 204.
        let (sx, sy) = (100, 60);
        let si = world.index(sx, sy);
        world.tiles[si] = Tile::default();
        unlock_map_tile_pretty(&world, &mut m.tiles, sx, sy);
        // The centre is a SKY-background air tile — `CreateMapTile`'s
        // sky arm forces the light to 255 (`MapHelper.cs`'s skyPosition
        // chain), so the falloff input 255 rides through. The ring reveals
        // (its sky tiles also clamp to 255); the exact per-tile number is
        // the sky override's, not the falloff's.
        assert_eq!(m.tiles[world.index(sx, sy)].light, 255, "the centre");
        assert!(
            m.tiles[world.index(100, 59)].light > 0,
            "the ring reveals"
        );
        assert!(m.tiles[world.index(100, 56)].light > 0, "the ring edge reveals");
        assert_eq!(m.tiles[world.index(100, 54)].light, 0, "beyond the radius stays dark");
        // The stone cell refuses the whole walk — itself and its ring stay
        // dark (`WorldGen.SolidTile`, `WorldMap.cs:136-139`).
        let (gx, gy) = (120, 60);
        unlock_map_tile_pretty(&world, &mut m.tiles, gx, gy);
        assert_eq!(m.tiles[world.index(gx, gy)].light, 0, "the solid skips");
        assert_eq!(m.tiles[world.index(gx, gy - 1)].light, 0, "the ring stays dark");
        // The lava cell refuses (`:144-147`).
        let si = world.index(140, 60);
        world.tiles[si] = Tile::default();
        world.tiles[si].liquid = 200;
        world.tiles[si].flags |= FLAG_ACTIVE | tr_core::tile::FLAG_LIQUID_LAVA;
        unlock_map_tile_pretty(&world, &mut m.tiles, 140, 60);
        assert_eq!(m.tiles[world.index(140, 60)].light, 0, "the lava skips");
    }

    #[test]
    fn clear_edges_wipes_the_40_tile_bands() {
        // `WorldMap.cs:260-285`.
        let world = ground_world();
        let mut m = WorldMap::new(&world);
        for t in m.tiles.iter_mut() {
            *t = MapTile::create(1, 255, 0);
        }
        m.clear_edges(&world);
        assert_eq!(m.tiles[world.index(0, 0)].light, 0);
        assert_eq!(m.tiles[world.index(39, 100)].light, 0);
        assert_eq!(m.tiles[world.index(100, 10)].light, 0);
        assert_eq!(m.tiles[world.index(100, 60)].light, 255, "the interior keeps");
        assert_eq!(m.tiles[world.index(100, 39)].light, 0, "the top band");
        assert_eq!(m.tiles[world.index(100, 40)].light, 255, "the first interior row");
        assert_eq!(m.tiles[world.index(100, 88)].light, 0, "the bottom band");
    }

    #[test]
    fn the_edit_queue_round_trips_into_the_map() {
        // `MapUpdateQueue.cs:43-71`: the per-tile add queues once (the
        // dedupe) and the drain reports the changed tiles once.
        let mut world = ground_world();
        let mut m = WorldMap::new(&world);
        let i = world.index(100, 40);
        m.tiles[i] = MapTile::create(1, 200, 0);
        let mut q = MapUpdateQueue::default();
        q.add_tile(&mut m, 100, 40);
        q.add_tile(&mut m, 100, 40);
        world.tiles[i] = Tile::default(); // the edit
        let changed = q.update(&world, &mut m);
        assert_eq!(changed.len(), 1, "the dedupe");
        assert_ne!(m.tiles[i].ty, 1);
    }

    #[test]
    fn reveal_window_marks_tiles_around_the_player() {
        let world = ground_world();
        let mut p = Player::spawn(&world);
        // The world's 40-tile border bands are un-revealable
        // (`LightingEngine.cs:253-255`'s inset + `ClearEdges`) — spawn mid-map.
        p.x = 100.0 * TILE_PX;
        let px = (p.x / TILE_PX) as i32;
        let py = (p.y / TILE_PX) as i32;
        let light = LightMap::compute(
            &world,
            px - 40,
            py - 30,
            px + 40,
            py + 30,
            &LightParams::default(),
        );
        let mut m = WorldMap::new(&world);
        m.update(&world, Some(&p), &light, 0.3);
        assert!(m.tiles[world.index(px, py)].light > 0);
    }

    #[test]
    fn update_without_a_player_is_a_no_op() {
        let world = ground_world();
        let light = LightMap::compute(&world, 0, 0, 63, 127, &LightParams::default());
        let mut m = WorldMap::new(&world);
        m.update(&world, None, &light, 1.0);
        assert!(m.tiles.iter().all(|t| t.light == 0));
    }

    // ---- Task 6b: fullscreen zoom / pan / minimap geometry ----

    /// The size the vanilla map-frame branches key on (`Main.maxTilesX == 4200`).
    fn map_world() -> World {
        World::new("t".into(), 4200, 1200)
    }

    fn assert_close(a: f32, b: f32, eps: f32) {
        assert!((a - b).abs() < eps, "{a} != {b} (eps {eps})");
    }

    #[test]
    fn scale_clamp_matches_vanilla() {
        // `Main.cs:55387-55395`.
        assert_eq!(
            map_scale_clamp(1600.0, 4200, 0.1),
            1600.0 / 4200.0 * 0.599
        );
        assert_eq!(map_scale_clamp(1600.0, 4200, 100.0), 31.18);
    }

    #[test]
    fn wheel_zoom_matches_vanilla() {
        // `Player.cs:25197`.
        assert_eq!(map_wheel_scale(2.0, 120.0), 2.0 * 1.3);
        assert_eq!(map_wheel_scale(2.0, -120.0), 2.0 * 0.7);
        assert_eq!(map_wheel_scale(2.0, 0.0), 2.0);
    }

    #[test]
    fn key_zoom_matches_vanilla() {
        // `Player.cs:24900-24907`.
        assert_eq!(map_key_scale(2.0, true), 2.1);
        assert_eq!(map_key_scale(2.0, false), 1.9);
    }

    #[test]
    fn pan_clamps_to_the_world() {
        // `Main.cs:55295-55296` + `55398-55413`.
        let w = map_world();
        assert_eq!(map_clamp_pos([0.0, 0.0], &w), [10.0, 10.0]);
        assert_eq!(
            map_clamp_pos([1e6, 1e6], &w),
            [(w.width - 10) as f32, (w.height - 19) as f32]
        );
    }

    #[test]
    fn drag_pan_is_cursor_pixels_over_scale() {
        // `Main.cs:55355-55358`: `0.06255 * 16` ≈ 1, so 100 px at scale 2.5 ≈ 40
        // tiles. The map moves opposite to the cursor.
        let d = map_drag_delta([100.0, 0.0], 2.5);
        assert!((d[0] + 40.032).abs() < 1e-3, "{d:?}");
        assert_eq!(d[1], 0.0);
    }

    #[test]
    fn full_quad_matches_drawmap() {
        // `Main.cs:55425-55430` + `55591-55607`.
        let w = map_world();
        let q = map_full_quad([2100.0, 355.0], 2.5, Vec2::new(1600.0, 900.0), &w);
        assert_eq!(
            q.dest,
            [
                800.0 - 2090.0 * 2.5,
                450.0 - 345.0 * 2.5,
                4179.0 * 2.5,
                1170.0 * 2.5
            ]
        );
        assert_eq!(
            q.uv,
            [10.0 / 4200.0, 10.0 / 1200.0, 4189.0 / 4200.0, 1180.0 / 1200.0]
        );
    }

    #[test]
    fn frame_rect_matches_the_4200_branch() {
        // `Main.cs:55431-55484`.
        let w = map_world();
        let rect = map_frame_rect([2100.0, 355.0], 2.5, Vec2::new(1600.0, 900.0), &w);
        let n = (4200 / 840) as f32 * 2.5 * 0.998;
        assert_eq!(
            rect,
            [
                800.0 - 2090.0 * 2.5 - 37.3 * n,
                450.0 - 345.0 * 2.5 - 1.7 * n,
                (928.0 - 16.0) * n,
                (248.0 - 8.31) * n
            ]
        );
    }

    #[test]
    fn minimap_clamps_to_the_world_edge() {
        // Main.cs:55295-55296 + 55591-55607: the minimap caps at maxTilesX-10 / maxTilesY-19.
        let w = map_world();
        let q = map_minimap_quad([4190.0, 1180.0], 1.05, 1.0, Vec2::new(1600.0, 900.0), &w);
        let win = 240.0 / 1.05;
        let a = 4190.0 - win / 2.0;
        let b = 1180.0 - win / 2.0;
        let eps = 1e-3;
        for (got, want) in q.dest.iter().zip([
            1308.0,
            90.0,
            (4190.0 - a) * 1.05,
            (1181.0 - b) * 1.05,
        ]) {
            assert_close(*got, want, eps);
        }
        for (got, want) in q.uv.iter().zip([
            a / 4200.0,
            b / 1200.0,
            4190.0 / 4200.0,
            1181.0 / 1200.0,
        ]) {
            assert_close(*got, want, eps);
        }
    }

    #[test]
    fn minimap_quad_matches_the_panel() {
        // `Main.cs:55503-55546`.
        let w = map_world();
        let q = map_minimap_quad([500.0, 300.0], 1.05, 1.0, Vec2::new(1600.0, 900.0), &w);
        let eps = 1e-3;
        for (got, want) in q.dest.iter().zip([1308.0, 90.0, 240.0, 240.0]) {
            assert_close(*got, want, eps);
        }
        for (got, want) in q.uv.iter().zip([
            385.7143 / 4200.0,
            185.7143 / 1200.0,
            614.2857 / 4200.0,
            414.2857 / 1200.0,
        ]) {
            assert_close(*got, want, eps);
        }
    }

    /// The panel top-left is exactly `minimap::minimap_origin_raw(screen,
    /// map_scale)` scaled back onto the screen at every interior view centre —
    /// the consolidation of the old hardcode `(screen.x - 292, 90)` onto the
    /// shared anchor (`Main.cs:55508-55511`).
    #[test]
    fn minimap_panel_uses_the_shared_anchor() {
        let w = map_world();
        for width in [1280.0f32, 1600.0, 1920.0] {
            let screen = Vec2::new(width, 900.0);
            let q = map_minimap_quad([2100.0, 355.0], 1.05, 1.0, screen, &w);
            let origin = crate::ui::minimap::minimap_origin_raw(screen, 1.0);
            assert_close(q.dest[0], origin[0], 1e-3);
            assert_close(q.dest[1], origin[1], 1e-3);
        }
    }

    /// DW-4.2: `Main.cs:55309-55329` — the tile pass is drawn inside
    /// `Matrix.CreateScale(Main.MapScale)`, so the destination rect is the raw
    /// one multiplied by the scale: the panel's visible left edge is
    /// `screenWidth - anchorLeft(MapScale)` (`Main.cs:55275-55276`, `:55508-55511`)
    /// and the drawn tile size and shown window both shrink with the scale.
    /// The zoom axis (`Main.mapMinimapScale`, clamped `0.2..3`,
    /// `Main.cs:55515-55522`) is a separate multiplier and is NOT scaled.
    #[test]
    fn minimap_quad_scales_the_tile_pass_by_map_scale() {
        let w = map_world();
        let screen = Vec2::new(1600.0, 900.0);
        let eps = 1e-3;
        // The clamp bounds (`Main.cs:4494`): `InitialMapScale = 1f` (`:67297`)
        // and the 0.5 floor.
        for map_scale in [1.0f32, 0.5] {
            let q = map_minimap_quad([500.0, 300.0], 1.05, map_scale, screen, &w);
            let raw = map_minimap_quad([500.0, 300.0], 1.05, 1.0, screen, &w);
            // The visible anchor moves with `MapScale` (`Main.cs:55275-55276`):
            // the raw origin divided by the scale times the scale back is exactly
            // `screenWidth - anchorLeft` (`Main.cs:55508-55511`).
            let anchor = crate::ui::minimap::minimap_anchor(map_scale);
            assert_close(q.dest[0], screen.x - anchor.0 as f32, eps);
            assert_close(q.dest[1], anchor.1 as f32, eps);
            // The tile size and the shown window follow the transform: a 240 px
            // raw panel covers `240 * map_scale` screen px.
            assert_close(q.dest[2], 240.0 * map_scale, eps);
            assert_close(q.dest[3], 240.0 * map_scale, eps);
            // The zoom axis is untouched: the same tiles are in view either way,
            // so the UV rect does not move with `MapScale`.
            assert_close(q.uv[0], raw.uv[0], eps);
            assert_close(q.uv[1], raw.uv[1], eps);
            assert_close(q.uv[2], raw.uv[2], eps);
            assert_close(q.uv[3], raw.uv[3], eps);
        }
        // The clamp bounds (`Main.cs:4494`): the 0.5 floor halves the panel.
        let half = map_minimap_quad([500.0, 300.0], 1.05, 0.5, screen, &w);
        assert_close(half.dest[2], 120.0, eps);
        // The zoom axis clamps at 0.2..3 on its own (`Main.cs:55515-55522`)
        // and is never scaled by `MapScale`: the shown tile window (the UV rect)
        // is identical at either scale, and only the drawn size follows the
        // transform. Zoomed all the way out the window hits the world edge, so
        // the quad's own clamp shifts the left edge — the invariant is the size.
        for zoom in [3.0f32, 0.1] {
            let wide = map_minimap_quad([500.0, 300.0], zoom, 1.0, screen, &w);
            let scaled = map_minimap_quad([500.0, 300.0], zoom, 0.5, screen, &w);
            for (a, b) in wide.uv.iter().zip(scaled.uv) {
                assert_close(*a, b, eps);
            }
            assert_close(scaled.dest[2], wide.dest[2] * 0.5, eps);
            assert_close(scaled.dest[3], wide.dest[3] * 0.5, eps);
        }
        // An interior view centre keeps the anchor assertion at every scale: the
        // panel's visible top-left is `screenWidth - anchorLeft(MapScale)`.
        for map_scale in [1.0f32, 0.5] {
            let q = map_minimap_quad([2100.0, 355.0], 3.0, map_scale, screen, &w);
            let anchor = crate::ui::minimap::minimap_anchor(map_scale);
            assert_close(q.dest[0], screen.x - anchor.0 as f32, eps);
            assert_close(q.dest[1], anchor.1 as f32, eps);
        }
    }

    /// DW-4.2's "full map unchanged": the fullscreen map is drawn at
    /// `Main.mapFullscreenScale` (default 4f, `Main.cs:66756`, read at
    /// `Main.cs:55390`) — a different axis from `Main.MapScale` (`:4494`), which
    /// only enters the minimap branch (`:55309-55329`). The full-map geometry at
    /// the vanilla default therefore keeps the source oracle and is untouched by
    /// the minimap anchor change.
    #[test]
    fn full_map_scale_is_its_own_axis() {
        let w = map_world();
        let q = map_full_quad([2100.0, 355.0], 4.0, Vec2::new(1600.0, 900.0), &w);
        assert_eq!(
            q.dest,
            [
                800.0 - 2090.0 * 4.0,
                450.0 - 345.0 * 4.0,
                4179.0 * 4.0,
                1170.0 * 4.0
            ]
        );
        assert_eq!(
            q.uv,
            [10.0 / 4200.0, 10.0 / 1200.0, 4189.0 / 4200.0, 1180.0 / 1200.0]
        );
    }

    // ---- Task 7: biome `MapBG` backdrop (`Main.DrawMapFullscreenBackground`) ----

    /// `Main.DrawMapFullscreenBackground` (`Main.cs:56347-56350`): the early return.
    #[test]
    fn bg_none_out_of_world() {
        // 4200x1200 -> `InWorld(..., 10)` needs 10 <= x < 4190, 10 <= y < 1190.
        let w = map_world();
        let screen = Vec2::new(1600.0, 900.0);
        assert!(map_bg_index(&w, Vec2::new(33600.0, 6000.0), screen, (5, 5)).is_none());
        assert!(map_bg_index(&w, Vec2::new(33600.0, 6000.0), screen, (2100, 5)).is_none());
    }

    /// `Main.cs:56352-56354`: `screenPosition.Y > (maxTilesY - 232) * 16`.
    #[test]
    fn bg_deepest_is_mapbg3() {
        let w = map_world();
        let screen = Vec2::new(1600.0, 900.0);
        // sp.y = 15939 - 450 = 15489 > (1200 - 232) * 16 = 15488.
        let bg = map_bg_index(&w, Vec2::new(33600.0, 15939.0), screen, (2100, 355)).unwrap();
        assert_eq!(bg.index, 2);
        assert!(!bg.sky_tint);
    }

    /// `Main.cs:56360-56362`: the Lihzahrd brick wall (WallID 87) forces `MapBG14`.
    #[test]
    fn bg_lihzahrd_wall_is_mapbg14() {
        let mut w = map_world();
        w.tile_mut(2100, 355).unwrap().wall = 87;
        let screen = Vec2::new(1600.0, 900.0);
        // sp.y = 2050 - 450 = 1600, above the deepest branch.
        let bg = map_bg_index(&w, Vec2::new(33600.0, 2050.0), screen, (2100, 355)).unwrap();
        assert_eq!(bg.index, 13);
        assert!(!bg.sky_tint);
    }

    /// `Main.cs:56364-56388`: the underground wall / biome chain.
    #[test]
    fn bg_underground_chain() {
        let screen = Vec2::new(1600.0, 900.0);
        // sp.y = 5251 - 450 = 4801 > worldSurface * 16 = 4800.
        let cam = Vec2::new(33600.0, 5251.0);
        let mut w = map_world();

        // No biome, tile y 355 <= rockLayer 540 -> cave default (num 1).
        let bg = map_bg_index(&w, cam, screen, (2100, 355)).unwrap();
        assert_eq!(bg.index, 1);
        assert!(!bg.sky_tint);

        for (wall, want) in [(86u16, 15), (180, 16), (178, 17), (62, 18)] {
            w.tile_mut(2100, 355).unwrap().wall = wall;
            let bg = map_bg_index(&w, cam, screen, (2100, 355)).unwrap();
            assert_eq!(bg.index, want, "wall {wall}");
        }
        w.tile_mut(2100, 355).unwrap().wall = 0;

        // Rock layer: 540 < 650 <= 1000 (height - 200).
        let bg = map_bg_index(&w, cam, screen, (2100, 650)).unwrap();
        assert_eq!(bg.index, 31);
        assert!(!bg.sky_tint);

        // Snow biome at the surface of the camera column.
        w.column_surface = vec![100; 4200];
        w.tile_mut(2100, 100).unwrap().kind = 147;
        let bg = map_bg_index(&w, cam, screen, (2100, 355)).unwrap();
        assert_eq!(bg.index, 3);
        assert!(!bg.sky_tint);

        // Desert biome.
        w.tile_mut(2100, 100).unwrap().kind = 53;
        let bg = map_bg_index(&w, cam, screen, (2100, 355)).unwrap();
        assert_eq!(bg.index, 14);
        assert!(!bg.sky_tint);
    }

    /// `Main.cs:56393-56438`: the surface chain, all tinted with
    /// `Main.ColorOfTheSkies` (`sky_tint = true`).
    #[test]
    fn bg_surface_chain() {
        let screen = Vec2::new(1600.0, 900.0);
        // sp.y = 4850 - 450 = 4400 <= worldSurface * 16 = 4800.
        let cam = Vec2::new(33600.0, 4850.0);

        // Plain forest falls through every branch: the `MapBGs[0]` default.
        let bg = map_bg_index(&map_world(), cam, screen, (2100, 355)).unwrap();
        assert_eq!(bg.index, 0);
        assert!(bg.sky_tint);

        let mut w = map_world();
        w.column_surface = vec![100; 4200];

        w.tile_mut(2100, 100).unwrap().kind = 53;
        let bg = map_bg_index(&w, cam, screen, (2100, 355)).unwrap();
        assert_eq!(bg.index, 9);
        assert!(bg.sky_tint);

        w.tile_mut(2100, 100).unwrap().kind = 147;
        let bg = map_bg_index(&w, cam, screen, (2100, 355)).unwrap();
        assert_eq!(bg.index, 11);
        assert!(bg.sky_tint);

        // Beach: sp.x = 4800 - 800 = 4000, num2 = (4000 + 800) / 16 = 300 < 380.
        let bg = map_bg_index(&map_world(), Vec2::new(4800.0, 4850.0), screen, (2100, 355))
            .unwrap();
        assert_eq!(bg.index, 10);
        assert!(bg.sky_tint);

        // Space: the player-centre tile is above `worldSurface * 0.35` = 105.
        let w = map_world();
        let bg = map_bg_index(&w, cam, screen, (2100, 50)).unwrap();
        assert_eq!(bg.index, 32);
        assert!(bg.sky_tint);
    }

    // ---- Task 8: fullscreen player head icon (`Main.cs:55931-55954`) ----

    /// `Main.DrawMap`'s fullscreen head-icon position (`Main.cs:55937-55944`).
    #[test]
    fn head_icon_pos_matches_vanilla() {
        // Hand-computed from `Main.cs:55425-55430` (origin) + `55937-55944`.
        //
        // origin = [800 - 2100*scale + 10*scale, 450 - 355*scale + 10*scale]
        // scale 2.5 -> origin = [800 - 5250 + 25, 450 - 887.5 + 25] = [-4425, -412.5]
        //   x = (33608/16)*2.5 + (-4425) - 6  - 10*2.5
        //     = 2100.5*2.5 - 4425 - 6 - 25 = 5251.25 - 4456 = 795.25
        //   y = (5688/16)*2.5 + (-412.5) - 2 - (2 - 2.5/5*2) - 25
        //     = 355.5*2.5 - 412.5 - 2 - 1 - 25 = 888.75 - 440.5 = 448.25
        // scale 4.0 -> origin = [800 - 8400 + 40, 450 - 1420 + 40] = [-7560, -930]
        //   x = 2100.5*4 - 7560 - 6 - 40 = 8402 - 7606 = 796
        //   y = 355.5*4 - 930 - 2 - (2 - 4/5*2) - 40 = 1422 - 930 - 2 - 0.4 - 40 = 449.6
        let screen = Vec2::new(1600.0, 900.0);
        let pos = [2100.0, 355.0];
        let center_px = [33608.0, 5688.0];
        let got = head_icon_pos(center_px, pos, 2.5, screen);
        assert_close(got[0], 795.25, 1e-3);
        assert_close(got[1], 448.25, 1e-3);
        let got = head_icon_pos(center_px, pos, 4.0, screen);
        assert_close(got[0], 796.0, 1e-3);
        assert_close(got[1], 449.6, 1e-3);
    }

    // ---- S1-L-5: the `.map` deflate parity (`MapHelper.InternalSaveMap`) ----
    //
    // The compressed layer is NOT byte-identical to vanilla: the vanilla
    // runtime's `clrcompression.dll` (.NET FW 4.8.1) embeds a Microsoft-patched
    // zlib whose output no stock zlib reproduces at any parameter combination
    // (the evidence and the parameter probe live in ANALYSIS.md, "The `.map`
    // deflate stream"). What IS 1:1 and frozen here: the uncompressed prefix,
    // the pre-deflate row body, and the wrapper semantics (raw deflate at the
    // `CompressionMode.Compress` ctor default = `CompressionLevel.Optimal`).

    /// The frozen pre-deflate row body for the synthetic map below — what the
    /// Rust encoder must emit and what vanilla's `DeflateStream` compressed
    /// byte-for-byte (`fw48-reference.deflate` inflates back onto it). Dump
    /// with `S1_L5_DUMP_BODY=<path>`.
    const S1_L5_BODY: &[u8] = include_bytes!("../../../tools/MapDeflateHarness/map-body.bin");
    /// The vanilla FW 4.8.1 `DeflateStream` bytes for that body — the
    /// reproducible divergence evidence, produced by `tools/MapDeflateHarness/fw48`.
    const S1_L5_FW_REFERENCE: &[u8] =
        include_bytes!("../../../tools/MapDeflateHarness/fw48-reference.deflate");
    /// Our own zlib-rs level-6 output for that body — the regression pin for
    /// the compressor choice. Dump with `S1_L5_DUMP_FLATE2=<path>`.
    const S1_L5_OUR_DEFLATE: &[u8] =
        include_bytes!("../../../tools/MapDeflateHarness/rust-deflate.bin");

    /// The S1-L-5 synthetic map: 320x64, `world_surface` 20, `rock_layer` 32.
    ///
    /// The rows walk every `encode_tiles` category and every run shape: dark
    /// skip-runs (single, mid-row and `> 255`), `light == 255` exact runs
    /// (`Equals`), variable-light runs (`EqualsWithoutLight`) including one
    /// that spans the full row (the row-end case), painted tiles/walls,
    /// 16-bit tile and wall types, all four liquid slots (the shimmer rides
    /// slot 0's `0x40` bit), the sky-gradient band, the dirt/rock depth bands
    /// and the hell band. The reader resolves the depth-bound slots by row:
    /// a dirt/rock colour is picked by `l < rockLayer` (`LoadMapVersion2:4002`)
    /// and the sky/hell slot by `l < worldSurface` (`:3999`), so sky rows sit
    /// above the surface, hell below it, dirt above `rock_layer`, rock below.
    fn s1_l5_synthetic_map() -> (Vec<MapTile>, usize, usize, f64, f64) {
        let t = color_table();
        let w = 320usize;
        let h = 64usize;
        let surface = 20.0f64;
        let rock = 32.0f64;
        assert!(
            t.wall_position as usize > t.tile_position as usize + 300,
            "the tile band must span a relative type > 255"
        );
        assert!(
            t.liquid_position as usize > t.wall_position as usize + 300,
            "the wall band must span a relative type > 255"
        );
        let mut map = vec![MapTile::default(); w * h];
        let mut rng = 0x5EED_1C55u32;
        let mut nxt = move || {
            rng ^= rng << 13;
            rng ^= rng >> 17;
            rng ^= rng << 5;
            rng
        };
        let put = |map: &mut Vec<MapTile>, x: usize, y: usize, tile: MapTile| {
            map[y * w + x] = tile;
        };

        // l=0: one dark skip-run across the whole row (run 319 > 255).
        // l=1: alternating dark tiles and lit tiles whose types vary.
        for x in (0..w).step_by(2) {
            let ty = t.tile_position + ((x / 2) % 7) as u16 + 1;
            put(&mut map, x + 1, 1, MapTile::create(ty, 25 + (nxt() % 60) as u8, 0));
        }
        // l=2: one variable-light run spanning the full row — the row-end case
        // with a two-byte run length and a light byte per run tile.
        for x in 0..w {
            put(&mut map, x, 2, MapTile::create(t.tile_position + 5, 20 + (x % 9) as u8, 0));
        }
        // l=3: two tile types in blocks of 5, one colour per block.
        for x in 0..w {
            let ty = if (x / 5) % 2 == 0 {
                t.tile_position + 1
            } else {
                t.tile_position + 2
            };
            put(&mut map, x, 3, MapTile::create(ty, 30 + (nxt() % 200) as u8, ((x / 5) % 6) as u8));
        }
        // l=4: a 16-bit tile type (relative 300): a 40-tile `light == 255`
        // exact run, then variable-light singles.
        for x in 0..40 {
            put(&mut map, x, 4, MapTile::create(t.tile_position + 300, 255, 0));
        }
        for x in 40..w {
            put(&mut map, x, 4, MapTile::create(t.tile_position + 300, 60 + (nxt() % 120) as u8, (x % 3) as u8));
        }
        // l=5: painted walls in blocks.
        for x in 0..w {
            let ty = t.wall_position + ((x / 8) % 4) as u16 * 2;
            put(&mut map, x, 5, MapTile::create(ty, 55 + (nxt() % 90) as u8, ((x / 8) % 6) as u8));
        }
        // l=6: a 16-bit wall type (relative 300).
        for x in 0..w {
            put(&mut map, x, 6, MapTile::create(t.wall_position + 300, 90, (x % 4) as u8));
        }
        // l=7: all four liquid slots; the shimmer rides slot 0's `0x40` bit.
        for x in 0..w {
            put(&mut map, x, 7, MapTile::create(t.liquid_position + (x % 4) as u16, 45 + (nxt() % 50) as u8, 0));
        }
        // l=8..12: sky rows (above the surface): no type or light is stored;
        // the reader rebuilds the gradient from the row (`:3999`).
        for l in 8..13usize {
            let gradient = ((255.0 * (l as f64 / surface)) as u8) as u16;
            for x in 0..w {
                put(&mut map, x, l, MapTile::create(t.sky_position + gradient, 255, (l % 4) as u8));
            }
        }
        // l=13: a long dark skip-run broken by a single lit tile mid-row.
        put(&mut map, 137, 13, MapTile::create(t.tile_position + 9, 40, 2));
        // l=14: dark.
        // l=15..19: dirt rows (above rock_layer; the dirt band is 256 wide, so
        // every relative type stays one byte).
        for l in 15..20usize {
            for x in 0..w {
                let ty = t.dirt_position + ((x / 3 + l) % 220) as u16;
                put(&mut map, x, l, MapTile::create(ty, 35 + (nxt() % 150) as u8, ((x + l) % 7) as u8));
            }
        }
        // l=20..23: between the surface and rock_layer — tiles, walls and
        // liquids only (the depth-bound slots would decode differently here).
        for l in 20..24usize {
            for x in 0..w {
                let ty = match x % 3 {
                    0 => t.tile_position + ((x / 3 + l) % 250) as u16 + 1,
                    1 => t.wall_position + ((x / 5 + l) % 40) as u16 * 3,
                    _ => t.liquid_position + (x % 3) as u16,
                };
                put(&mut map, x, l, MapTile::create(ty, 40 + (nxt() % 180) as u8, (x % 5) as u8));
            }
        }
        // l=24..25: `light == 255` whole-row exact runs.
        for l in 24..26usize {
            for x in 0..w {
                put(&mut map, x, l, MapTile::create(t.tile_position + 11, 255, 0));
            }
        }
        // l=26..27: walls whose type+colour repeat every 9 tiles with varying
        // lights (variable-light runs of 8 plus a mismatch).
        for l in 26..28usize {
            for x in 0..w {
                put(&mut map, x, l, MapTile::create(t.wall_position + (x % 9) as u16, 70 + (nxt() % 100) as u8, (x % 9) as u8));
            }
        }
        // l=28..31: walls and liquids, painted.
        for l in 28..32usize {
            for x in 0..w {
                let ty = if x % 2 == 0 {
                    t.wall_position + (x / 2 % 30) as u16
                } else {
                    t.liquid_position + (l % 3) as u16
                };
                put(&mut map, x, l, MapTile::create(ty, 50 + (nxt() % 140) as u8, (l % 9) as u8));
            }
        }
        // l=32..40: rock rows (at/below rock_layer).
        for l in 32..41usize {
            for x in 0..w {
                let ty = t.rock_position + ((x / 4 + l) % 200) as u16;
                put(&mut map, x, l, MapTile::create(ty, 45 + (nxt() % 120) as u8, ((x + l) % 8) as u8));
            }
        }
        // l=41..63: hell rows (below the surface): category 6, no type stored;
        // the reader maps them to the single hell entry by depth.
        for l in 41..64usize {
            for x in 0..w {
                put(&mut map, x, l, MapTile::create(t.hell_position, 60 + (nxt() % 130) as u8, (x % 9) as u8));
            }
        }
        (map, w, h, surface, rock)
    }

    #[test]
    fn s1_l_5_body_matches_vanilla_and_the_wrapper_is_faithful() {
        let (map, w, h, surface, rock) = s1_l5_synthetic_map();
        let body = tr_core::map::encode_tiles(&map, w, h);

        // The encoder is pinned byte-for-byte to the frozen body fixture — the
        // pre-deflate layer is the part of the format that is 1:1.
        assert_eq!(
            body,
            S1_L5_BODY.to_vec(),
            "encode_tiles drifted from the frozen body fixture"
        );

        // The vanilla FW 4.8.1 `DeflateStream` bytes inflate back onto exactly
        // our body: the compressed layer interoperates, its bytes do not match
        // any stock zlib (see ANALYSIS.md, "The `.map` deflate stream").
        use std::io::Read;
        let mut decoder = flate2::read::DeflateDecoder::new(S1_L5_FW_REFERENCE);
        let mut inflated = Vec::new();
        decoder.read_to_end(&mut inflated).unwrap();
        assert_eq!(
            inflated,
            body,
            "the vanilla reference stream inflates to a different body"
        );

        // Our encoder: raw deflate at the C# ctor default (Optimal, level 6),
        // pinned so the compressor choice cannot drift silently.
        use std::io::Write;
        let mut encoder =
            flate2::write::DeflateEncoder::new(Vec::new(), flate2::Compression::new(6));
        encoder.write_all(&body).unwrap();
        let deflated = encoder.finish().unwrap();
        assert_eq!(
            deflated,
            S1_L5_OUR_DEFLATE.to_vec(),
            "our deflate encoder drifted"
        );

        // And the body still round-trips through the reader.
        let t = color_table();
        let header = tr_core::map::MapHeader {
            version: tr_core::map::MAP_FORMAT_VERSION,
            seed: 0,
            height: h as i32,
            width: w as i32,
            tile_count: tr_core::mapdata::TILE_COUNT as i16,
            wall_count: tr_core::mapdata::WALL_COUNT as i16,
            liquid_count: 4,
            sky_count: 256,
            dirt_count: 256,
            rock_count: 256,
        };
        let decoded = tr_core::map::decode_tiles(
            &body,
            &header,
            &t.tile_option_counts,
            &t.wall_option_counts,
            w,
            h,
            surface,
            rock,
        )
        .expect("the S1-L-5 body must decode");
        assert_eq!(decoded, map);
    }

    #[test]
    fn s1_l_5_dump_body() {
        let Ok(path) = std::env::var("S1_L5_DUMP_BODY") else {
            return;
        };
        let (map, w, h, ..) = s1_l5_synthetic_map();
        std::fs::write(path, tr_core::map::encode_tiles(&map, w, h)).unwrap();
    }

    #[test]
    fn s1_l_5_dump_flate2() {
        let Ok(path) = std::env::var("S1_L5_DUMP_FLATE2") else {
            return;
        };
        use std::io::Write;
        let mut encoder =
            flate2::write::DeflateEncoder::new(Vec::new(), flate2::Compression::new(6));
        encoder.write_all(S1_L5_BODY).unwrap();
        let deflated = encoder.finish().unwrap();
        std::fs::write(path, &deflated).unwrap();
    }
}

/// `Terraria.Map.MapUpdateQueue` (`MapUpdateQueue.cs:21-108`): the edit-side
/// map refresh queues — the per-tile queue (`MapUpdateQueue.Add(int, int)`,
/// the `Main.Map.QueueUpdate` dedupe) and the area queue
/// (`MapUpdateQueue.Add(Rectangle)`). The `Main.dedServ`/`generatingWorld`/
/// `!mapEnabled` gates ride the callers. `Update()` (`Main.cs:61862`) drains
/// both through `WorldMap.UpdateType` and reports the changed tiles (the
/// C# feeds `MapRenderer.QueueChange`; the port re-uploads via `dirty`).
#[derive(Default)]
pub struct MapUpdateQueue {
    areas: Vec<[i32; 4]>,
    tiles: Vec<(i32, i32)>,
}

impl MapUpdateQueue {
    /// `MapUpdateQueue.Add(Rectangle)` (`MapUpdateQueue.cs:23-41`): the
    /// world-clamped rect queues every tile it covers.
    pub fn add_area(&mut self, map: &mut WorldMap, x: i32, y: i32, w: i32, h: i32) {
        let (mw, mh) = (map.tiles_w as i32, map.tiles_h as i32);
        let x0 = x.max(0);
        let y0 = y.max(0);
        let x1 = (x + w).min(mw);
        let y1 = (y + h).min(mh);
        self.areas.push([x0, y0, x1 - x0, y1 - y0]);
        for i in x0..x1 {
            for j in y0..y1 {
                map.queue_update(i, j);
            }
        }
    }

    /// `MapUpdateQueue.Add(int, int)` (`MapUpdateQueue.cs:43-60`): the
    /// per-tile queue behind `Main.Map.QueueUpdate`'s dedupe.
    pub fn add_tile(&mut self, map: &mut WorldMap, x: i32, y: i32) {
        if !map.queue_update(x, y) {
            return; // `:44`: the gate skips the enqueue
        }
        self.tiles.push((x, y));
    }

    /// `MapUpdateQueue.Update` (`MapUpdateQueue.cs:62-71`): both queues drain
    /// through `WorldMap.UpdateType`; the changed tiles return (the C#'s
    /// `MapRenderer.QueueChange`).
    pub fn update(&mut self, world: &World, map: &mut WorldMap) -> Vec<(i32, i32)> {
        let mut changed = Vec::new();
        for (x, y) in std::mem::take(&mut self.tiles) {
            if map.update_type(world, x, y) {
                changed.push((x, y));
            }
        }
        for [x, y, w, h] in std::mem::take(&mut self.areas) {
            for i in x..x + w {
                for j in y..y + h {
                    if map.update_type(world, i, j) {
                        changed.push((i, j));
                    }
                }
            }
        }
        changed
    }
}
