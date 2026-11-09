//! Minimap and full map (`Main.DrawMap`, `Terraria.Map.WorldMap`): reveals
//! `MapTile`s from the light map (`WorldMap.UpdateLighting`,
//! `LightingEngine.ExportToMiniMap`) and draws them from a world-sized texture
//! (one texel per map tile) at `Main.DrawMap`'s geometry.

use glam::Vec2;
use tr_core::lighting::LightMap;
use tr_core::map::{MapTile, get_map_tile_color, reveal_rect};
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
}

impl WorldMap {
    pub fn new(world: &World) -> Self {
        Self {
            tiles: vec![MapTile::default(); (world.width as usize) * (world.height as usize)],
            open: false,
            scale: MAP_FULLSCREEN_SCALE_DEFAULT,
            pos: [0.0, 0.0],
            minimap_scale: 1.05,
            timer: 1.0,
            dirty: Vec::new(),
            uploaded: false,
            needs_full_upload: false,
            reset_pos: true,
        }
    }

    /// Reveals a `75x47` window around the player 4x/second from `light`.
    pub fn update(&mut self, world: &World, player: Option<&Player>, light: &LightMap, dt: f32) {
        let Some(p) = player else { return };
        self.timer += dt;
        if self.timer < 0.25 {
            return;
        }
        self.timer = 0.0;
        let px = (p.x / TILE_PX) as i32;
        let py = (p.y / TILE_PX) as i32;
        let changed = reveal_rect(
            world,
            &mut self.tiles,
            light,
            px - 37,
            py - 23,
            px + 37,
            py + 23,
        );
        self.dirty.extend(changed);
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
    ) {
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
    // The C# `DeflateStream` is raw deflate (no zlib/gzip wrapper).
    let mut encoder = flate2::write::DeflateEncoder::new(Vec::new(), flate2::Compression::fast());
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
        assert_eq!(wm.tiles, saved);
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
        let mut w = World::new("t".into(), 64, 128);
        for x in 0..64 {
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
    fn reveal_window_marks_tiles_around_the_player() {
        let world = ground_world();
        let p = Player::spawn(&world);
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
}
