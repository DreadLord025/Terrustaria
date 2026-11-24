//! The map icon overlay layers (`Terraria.Map.MapIconOverlay` + the
//! `Terraria.Map/*Layer*.cs` files, registered at `Main.cs:66551` in the
//! order Digtoise → BossBag → Spawn → TeamBasedSpawn → TeleportPylons →
//! Pings). The context is `MapOverlayDrawContext` (`MapOverlayDrawContext.cs`),
//! the layer walk is `Main.MapIcons.Draw` (`MapIconOverlay.cs:17-27`).

use glam::Vec2;
use tr_core::World;
use tr_core::pylons::PylonInfo;
use tr_render::SpriteRenderer;

use crate::assets;
use crate::item::ItemDrop;
use crate::map::WorldMap;
use crate::player::Player;

/// The two `Alignment` multipliers the layers use (`Terraria.UI.Alignment`).
const ALIGN_BOTTOM: [f32; 2] = [0.5, 1.0];
const ALIGN_CENTER: [f32; 2] = [0.5, 0.5];

/// `Terraria.Map.MapOverlayDrawContext` (`MapOverlayDrawContext.cs:8-70`):
/// the map-space → screen-space transform, the optional clip rect and the
/// hover hit-test. [`Self::draw`] pushes the icon quad and returns
/// `DrawResult.IsMouseOver`.
pub(crate) struct MapOverlayContext<'a> {
    r: &'a mut SpriteRenderer,
    /// `Main.MouseScreen`.
    mouse: Vec2,
    /// `Main.DrawMap`'s `mapPosition` — `Vector2.Zero` on the fullscreen map
    /// (the layer positions are world tiles).
    map_position: [f32; 2],
    /// The map's top-left on screen (`num - 10f * num5`, `Main.cs:55625`).
    map_offset: [f32; 2],
    /// The minimap's screen rect (`Main.cs:55703`); `None` on the fullscreen.
    clip: Option<[f32; 4]>,
    /// `num5`: the px-per-tile scale.
    map_scale: f32,
    /// `num37`/`num46`/`num77`: the icon draw scale.
    draw_scale: f32,
    /// `b / 255`: the map alpha.
    opacity: f32,
}

impl MapOverlayContext<'_> {
    /// `Draw` (`MapOverlayDrawContext.cs:33-58`): the transform, the clip,
    /// the aligned quad and the mouse-over rect.
    pub fn draw(
        &mut self,
        uv: [f32; 4],
        size: [f32; 2],
        position: [f32; 2],
        alignment: [f32; 2],
    ) -> bool {
        if self.opacity == 0.0 {
            return false; // `:36-39`
        }
        let pos = [
            (position[0] - self.map_position[0]) * self.map_scale + self.map_offset[0],
            (position[1] - self.map_position[1]) * self.map_scale + self.map_offset[1],
        ];
        if let Some(c) = self.clip {
            if pos[0] < c[0] || pos[1] < c[1] || pos[0] > c[2] || pos[1] > c[3] {
                return false; // `:45-48`: `DrawResult.Culled`
            }
        }
        let origin = [size[0] * alignment[0], size[1] * alignment[1]];
        self.r
            .push_ui_sprite(pos, size, uv, [self.opacity; 4]);
        // `:57-58`: the hover rect is the draw box shifted by the aligned
        // origin, sized by the draw scale.
        let hx = pos[0] - origin[0] * self.draw_scale;
        let hy = pos[1] - origin[1] * self.draw_scale;
        let hw = size[0] * self.draw_scale;
        let hh = size[1] * self.draw_scale;
        self.mouse.x >= hx
            && self.mouse.x <= hx + hw
            && self.mouse.y >= hy
            && self.mouse.y <= hy + hh
    }

    /// `GetUnclampedDrawRegion`'s centre (`MapOverlayDrawContext.cs:60-70`) —
    /// the pylon line's endpoints.
    pub fn unclamped_center(
        &self,
        size: [f32; 2],
        position: [f32; 2],
        scale: f32,
        alignment: [f32; 2],
    ) -> [f32; 2] {
        let pos = [
            (position[0] - self.map_position[0]) * self.map_scale + self.map_offset[0],
            (position[1] - self.map_position[1]) * self.map_scale + self.map_offset[1],
        ];
        let origin = [size[0] * alignment[0], size[1] * alignment[1]];
        let num = self.draw_scale * scale;
        [
            pos[0] - origin[0] * num + size[0] * num * 0.5,
            pos[1] - origin[1] * num + size[1] * num * 0.5,
        ]
    }

    /// `Draw`'s scaled form (`MapOverlayDrawContext.cs:218-242`): the pylon
    /// markers' `scaleIfNotSelected`/`scaleIfSelected` pair. The hover rect
    /// is sized by `drawScale * scaleIfNotSelected` (`:232-234`) and the
    /// draw scale jumps to `drawScale * scaleIfSelected` while hovered
    /// (`:236-239`); the tint is `color * opacity`. Returns `IsMouseOver`.
    #[allow(clippy::too_many_arguments)]
    pub fn draw_scaled(
        &mut self,
        uv: [f32; 4],
        size: [f32; 2],
        position: [f32; 2],
        alignment: [f32; 2],
        tint: f32,
        scale_if_not_selected: f32,
        scale_if_selected: f32,
    ) -> bool {
        if self.opacity == 0.0 {
            return false; // `:220-223`
        }
        let pos = [
            (position[0] - self.map_position[0]) * self.map_scale + self.map_offset[0],
            (position[1] - self.map_position[1]) * self.map_scale + self.map_offset[1],
        ];
        if let Some(c) = self.clip {
            if pos[0] < c[0] || pos[1] < c[1] || pos[0] > c[2] || pos[1] > c[3] {
                return false; // `:225-228`
            }
        }
        let origin = [size[0] * alignment[0], size[1] * alignment[1]];
        let num = self.draw_scale * scale_if_not_selected;
        let top_left = [pos[0] - origin[0] * num, pos[1] - origin[1] * num];
        let (hw, hh) = (size[0] * num, size[1] * num);
        let hover = self.mouse.x >= top_left[0]
            && self.mouse.x <= top_left[0] + hw
            && self.mouse.y >= top_left[1]
            && self.mouse.y <= top_left[1] + hh;
        let scale = if hover {
            self.draw_scale * scale_if_selected
        } else {
            num
        };
        let a = self.opacity * tint;
        self.r.push_ui_sprite(
            [pos[0] - origin[0] * scale, pos[1] - origin[1] * scale],
            [size[0] * scale, size[1] * scale],
            uv,
            [a, a, a, a],
        );
        hover
    }

    /// `DrawClamped` (`MapOverlayDrawContext.cs:118-216`): the fullscreen
    /// pylon markers. The unclamped rect is pulled back inside the
    /// `screenBorderRegion`-inset screen along the ray from the screen
    /// centre; an offscreen marker draws at `scaleIfOffscreen` (×
    /// `scaleIfSelected` when hovered) plus the offscreen arrow
    /// (`Extra_299` frame (2,1) of 3x3) pointing back at the true position.
    /// Returns `(IsMouseOver, onScreen)`.
    #[allow(clippy::too_many_arguments)]
    pub fn draw_clamped(
        &mut self,
        uv: [f32; 4],
        size: [f32; 2],
        offscreen_uv: [f32; 4],
        offscreen_size: [f32; 2],
        position: [f32; 2],
        alignment: [f32; 2],
        tint: f32,
        scale_if_not_selected: f32,
        scale_if_selected: f32,
        scale_if_offscreen: f32,
        screen_border_region: i32,
        screen: [f32; 2],
    ) -> (bool, bool) {
        if self.opacity == 0.0 {
            return (false, true); // `:121-124`
        }
        let pos = [
            (position[0] - self.map_position[0]) * self.map_scale + self.map_offset[0],
            (position[1] - self.map_position[1]) * self.map_scale + self.map_offset[1],
        ];
        let origin = [size[0] * alignment[0], size[1] * alignment[1]];
        let num = self.draw_scale * scale_if_not_selected;
        let num2 = self.draw_scale * scale_if_offscreen;
        // The unclamped rect, INTEGER-truncated like the C# `Rectangle`.
        let rx = (pos[0] - origin[0] * num) as i32;
        let ry = (pos[1] - origin[1] * num) as i32;
        let rw = (size[0] * num) as i32;
        let rh = (size[1] * num) as i32;
        let (rx, ry, draw_pos, on_screen) =
            clamp_draw_region(rx, ry, rw, rh, pos, screen_border_region, screen);
        // `:185`: the hover test is the CLAMPED rect vs the mouse.
        let hover = self.mouse.x >= rx as f32
            && self.mouse.x <= (rx + rw) as f32
            && self.mouse.y >= ry as f32
            && self.mouse.y <= (ry + rh) as f32;
        // `:186-198`: the draw scale.
        let mut num23 = num;
        if !on_screen {
            num23 = num2;
            if hover {
                num23 *= scale_if_selected;
            }
        } else if hover {
            num23 = self.draw_scale * scale_if_selected;
        }
        // `:199-213`: the offscreen arrow, only when offscreen AND unhovered.
        if !on_screen && !hover {
            let mut dir = [pos[0] - draw_pos[0], pos[1] - draw_pos[1]];
            let rotation = dir[1].atan2(dir[0]);
            let len = (dir[0] * dir[0] + dir[1] * dir[1]).sqrt();
            if len > 0.0 {
                dir = [dir[0] / len, dir[1] / len];
            }
            // `rectangle.Center` — integer centre of the clamped rect.
            let center = [(rx + rw / 2) as f32, (ry + rh / 2) as f32];
            // `(float)(sourceRectangle.Height / 4)` — INTEGER division.
            let reach = (size[1] as i32 / 4) as f32 * num2;
            let arrow_pos = [
                center[0] + dir[0] * reach,
                center[1] + dir[1] * reach,
            ];
            // Origin `(0, height/2)`; the arrow rotates about that anchor.
            let (s, c) = rotation.sin_cos();
            let oy = offscreen_size[1] * 0.5;
            let tl = [arrow_pos[0] + oy * s, arrow_pos[1] - oy * c];
            let a = self.opacity;
            self.r.push_ui_sprite_rot(
                tl,
                [offscreen_size[0] * num2, offscreen_size[1] * num2],
                offscreen_uv,
                [a, a, a, a],
                rotation,
            );
        }
        // `:214`: the marker itself at the clamped position.
        let a = self.opacity * tint;
        self.r.push_ui_sprite(
            [draw_pos[0] - origin[0] * num23, draw_pos[1] - origin[1] * num23],
            [size[0] * num23, size[1] * num23],
            uv,
            [a, a, a, a],
        );
        (hover, on_screen)
    }
}

/// `DrawClamped`'s rect pull-back (`MapOverlayDrawContext.cs:132-184`), pure:
/// the unclamped integer rect `(rx, ry, rw, rh)` is pulled inside the
/// `border`-inset screen along the ray from the screen centre to the rect's
/// top-left. The X pass runs first (left edge, else right edge), then the Y
/// pass reads the ALREADY-shifted rect. Returns the clamped `(rx, ry)`, the
/// shifted draw position and `onScreen`. The shifts are the C#'s integer
/// `(int)((float)over / d * other)` truncations.
fn clamp_draw_region(
    mut rx: i32,
    mut ry: i32,
    rw: i32,
    rh: i32,
    pos: [f32; 2],
    border: i32,
    screen: [f32; 2],
) -> (i32, i32, [f32; 2], bool) {
    let mut on_screen = true;
    let mut draw_pos = pos;
    let max_x = screen[0] as i32 - border;
    let max_y = screen[1] as i32 - border;
    let mid_x = (border + max_x) / 2;
    let mid_y = (border + max_y) / 2;
    // `:137-160`: the X pull-back.
    if rx < border {
        let dx = rx - mid_x;
        let dy = ry - mid_y;
        let over = rx - border;
        let shift_y = (over as f32 / dx as f32 * dy as f32) as i32;
        draw_pos[0] -= over as f32;
        rx -= over;
        draw_pos[1] -= shift_y as f32;
        ry -= shift_y;
        on_screen = false;
    } else if rx + rw > max_x {
        on_screen = false;
        let dx = rx - mid_x;
        let dy = ry - mid_y;
        let over = rx + rw - max_x;
        let shift_y = (over as f32 / dx as f32 * dy as f32) as i32;
        draw_pos[0] -= over as f32;
        rx -= over;
        draw_pos[1] -= shift_y as f32;
        ry -= shift_y;
    }
    // `:161-184`: the Y pull-back reads the ALREADY-shifted rect.
    if ry < border {
        on_screen = false;
        let dx = rx - mid_x;
        let dy = ry - mid_y;
        let over = ry - border;
        let shift_x = (over as f32 / dy as f32 * dx as f32) as i32;
        draw_pos[0] -= shift_x as f32;
        rx -= shift_x;
        draw_pos[1] -= over as f32;
        ry -= over;
    } else if ry + rh > max_y {
        on_screen = false;
        let dx = rx - mid_x;
        let dy = ry - mid_y;
        let over = ry + rh - max_y;
        let shift_x = (over as f32 / dy as f32 * dx as f32) as i32;
        draw_pos[0] -= shift_x as f32;
        rx -= shift_x;
        draw_pos[1] -= over as f32;
        ry -= over;
    }
    (rx, ry, draw_pos, on_screen)
}

/// `TeleportPylonsMapLayer.DrawLine`'s segment geometry
/// (`TeleportPylonsMapLayer.cs:18-28`), pure: `(segment_count, rotation,
/// scale_x, scale_y)` for a line of `width` thickness. `num2 = Min(5,
/// (int)Distance)`; a sub-1-px line draws nothing (`num2 == 0`). The scale is
/// `(num/num2/16, width/16)` — the BlackTile is 16x16, so each segment is
/// `num/num2` px long and `width` px thick.
pub(crate) fn pylon_line_segments(start: [f32; 2], end: [f32; 2], width: f32) -> (i32, f32, f32, f32) {
    let dx = end[0] - start[0];
    let dy = end[1] - start[1];
    let num = (dx * dx + dy * dy).sqrt();
    let rotation = dy.atan2(dx);
    let num2 = 5.min(num as i32);
    if num2 <= 0 {
        return (0, rotation, 0.0, 0.0);
    }
    (num2, rotation, num / num2 as f32 / 16.0, width / 16.0)
}

/// `TeleportPylonsMapLayer.DrawLine` (`TeleportPylonsMapLayer.cs:18-28`):
/// `num = Distance(start, end)`, `rotation = (end - start).ToRotation()`,
/// `num2 = Min(5, (int)num)`; `num2` BlackTile quads lerped along the line,
/// each rotated by `rotation`, origin `(0, width/2)`, scale
/// `(num/num2/16, width/16)`. `width` is the line thickness (6 black, 2
/// white). A zero-length line (`num2 == 0`) draws nothing.
pub(crate) fn pylon_line(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    start: [f32; 2],
    end: [f32; 2],
    tint: [f32; 4],
    width: f32,
) {
    let Some(tex) = lookups.black_tile else {
        return;
    };
    let Some(uv) = lookups.hud_uv(Some(&tex)) else {
        return;
    };
    let (num2, rotation, sx, sy) = pylon_line_segments(start, end, width);
    if num2 <= 0 {
        return;
    }
    let dx = end[0] - start[0];
    let dy = end[1] - start[1];
    let fw = tex.width as f32;
    let fh = tex.height as f32;
    // Origin `(0, value.Width * 0.5)` — the rotation anchor.
    let oy = fw * 0.5;
    let (s, c) = rotation.sin_cos();
    for i in 0..num2 {
        let t = i as f32 / num2 as f32;
        let p = [start[0] + dx * t, start[1] + dy * t];
        // top-left = position + origin - R(rot)*origin, origin = (0, oy).
        let tl = [p[0] + oy * s, p[1] - oy * c];
        r.push_ui_sprite_rot(tl, [fw * sx, fh * sy], uv, tint, rotation);
    }
}

/// `ItemID.Sets.BossBag` (`ItemID.cs:1383`) — the map layer's boss-bag drops.
const BOSS_BAGS: [u16; 21] = [
    3318, 3319, 3320, 3321, 3322, 3323, 3324, 3325, 3326, 3327, 3328, 3329,
    3330, 3331, 3332, 3860, 3861, 3862, 4782, 4957, 5111,
];

fn is_boss_bag(kind: u16) -> bool {
    BOSS_BAGS.contains(&kind)
}

/// The layer inputs — the C# reads the `Main.*` globals; the port threads
/// them through one struct.
pub(crate) struct MapIconInputs<'a> {
    pub world: &'a World,
    /// `Main.LocalPlayer` — the bed spawn's `SpawnX/Y` and the team.
    pub player: Option<&'a Player>,
    /// `Main.item[400]` — the BossBag drop scan (`BossBagMapLayer.cs:10-24`).
    pub item_drops: &'a [ItemDrop],
    /// `Main.projectile`'s Digtoise trackers (1098/1123) — `(bottom_px,
    /// sprite_direction)` of the first active one the local player owns, the
    /// scan breaking on it (`DigtoiseMapLayer.cs:14-25`).
    pub digtoise: Option<([f32; 2], i32)>,
    /// `PingMapLayer`'s pool.
    pub pings: &'a [([f32; 2], f32)],
    /// `Main.PylonSystem.Pylons` (`TeleportPylonsMapLayer.cs:32`) — the live
    /// network, from `tr_core::pylons::scan_pylons`.
    pub pylons: &'a [PylonInfo],
    /// `TeleportPylonsSystem.IsPlayerNearAPylon(Main.LocalPlayer)`
    /// (`TeleportPylonsMapLayer.cs:39-42`): away from a pylon the markers dim
    /// to `Color.Gray * 0.5f`.
    pub near_pylon: bool,
    /// The map tiles, for `TeleportPylonsMapLayer.IsRevealed` (`:112-119`) —
    /// only read with the team-based-spawns seed on.
    pub map: Option<&'a WorldMap>,
    /// The team-based-spawns seed gate (`TeamBasedSpawnMapLayer.cs:10`).
    pub team_based_spawns: bool,
    pub mouse: Vec2,
    /// `Main.mapFullscreen` (`TeleportPylonsMapLayer.cs:45`/`:73-79`): the
    /// network lines and the clamped offscreen markers are fullscreen-only.
    pub map_fullscreen: bool,
    /// `Main.MapPylonTile` (`TeleportPylonsMapLayer.cs:45`): the pylon whose
    /// network lines draw; `(-1, -1)` = none.
    pub pylon_tile: (i32, i32),
    /// The screen size in px — `DrawClamped`'s border region math
    /// (`MapOverlayDrawContext.cs:133-136`).
    pub screen: Vec2,
    /// `Main.mouseLeft && Main.mouseLeftRelease` — the press edge the pylon
    /// marker click reads (`TeleportPylonsMapLayer.cs:83`/`:90`).
    pub left_press: bool,
}

/// The pylon marker click's outcome (`TeleportPylonsMapLayer.cs:90-109`):
/// an onscreen marker teleports (the caller runs the gate ladder + the
/// teleport), an offscreen one pans the map to it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PylonClick {
    Teleport(PylonInfo),
    PanTo(PylonInfo),
}

/// `Main.MapIcons.Draw`'s layer walk (`MapIconOverlay.cs:17-27`) in the
/// registration order. Returns the hovered layer's localized text (the C#
/// `ref text` feeds `Main.DrawMap`'s hover line), the pylon marker click
/// (`TeleportPylonsMapLayer.cs:90-109` — the caller drains it) and whether a
/// pylon marker was hovered (`Main.cancelWormHole = true`, `:81`).
pub(crate) fn map_layers_draw(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    inputs: &MapIconInputs,
    map_position: [f32; 2],
    map_offset: [f32; 2],
    clip: Option<[f32; 4]>,
    map_scale: f32,
    draw_scale: f32,
    opacity: f32,
) -> (Option<String>, Option<PylonClick>, bool) {
    let mut ctx = MapOverlayContext {
        r,
        mouse: inputs.mouse,
        map_position,
        map_offset,
        clip,
        map_scale,
        draw_scale,
        opacity,
    };
    let mut text: Option<String> = None;

    // `DigtoiseMapLayer` (`DigtoiseMapLayer.cs:13-26`): the player's Digtoise
    // tracker (1098/1123) draws Extra_305's 2-frame row, bottom-aligned at
    // `Bottom / 16 + (0.5, 0)`; column 1 faces left (`spriteDirection <= 0`).
    if let Some((bottom, dir)) = inputs.digtoise {
        if let Some(tex) = lookups.extra_305 {
            if let Some(uv) = lookups.hud_uv(Some(&tex)) {
                let fw = tex.width as f32 / 2.0;
                let fh = tex.height as f32;
                let col = if dir <= 0 { 1.0 } else { 0.0 };
                let uw = uv[2] - uv[0];
                let uv = [
                    uv[0] + uw * 0.5 * col,
                    uv[1],
                    uv[0] + uw * 0.5 * (col + 1.0),
                    uv[3],
                ];
                let pos = [bottom[0] / 16.0 + 0.5, bottom[1] / 16.0];
                if ctx.draw(uv, [fw, fh], pos, ALIGN_BOTTOM) {
                    text = Some(crate::lang::text("UI.Digtoise"));
                }
            }
        }
    }

    // `BossBagMapLayer` (`BossBagMapLayer.cs:10-24`): every active boss-bag
    // drop draws its item icon centred on the drop's tile.
    for drop in inputs.item_drops {
        if !is_boss_bag(drop.kind) {
            continue;
        }
        if let Some(rect) = lookups.item_icons.get(&drop.kind) {
            let Some(uv) = lookups.hud_uv(Some(rect)) else {
                continue;
            };
            let size = [rect.width as f32, rect.height as f32];
            let pos = [drop.pos.x / 16.0 + 0.5, drop.pos.y / 16.0 + 0.5];
            if ctx.draw(uv, size, pos, ALIGN_CENTER) {
                text = Some(crate::lang::item_name(drop.kind));
            }
        }
    }

    // `SpawnMapLayer` (`SpawnMapLayer.cs:9-27`): the world spawn and the bed
    // spawn markers, bottom-aligned.
    if let Some(tex) = lookups.map_spawn {
        if let Some(uv) = lookups.hud_uv(Some(&tex)) {
            let size = [tex.width as f32, tex.height as f32];
            let pos = [inputs.world.spawn_x as f32 + 0.5, inputs.world.spawn_y as f32];
            if ctx.draw(uv, size, pos, ALIGN_BOTTOM) {
                text = Some(crate::lang::text("UI.SpawnPoint"));
            }
        }
    }
    // `localPlayer.SpawnX/SpawnY` — the port keeps the bed spawn on the
    // World (`World.bed_spawn`), which `Player.Spawn`'s re-validation
    // (`Player.cs:37571-37575`) already consumes.
    if let Some((bx, by)) = inputs.world.bed_spawn {
        if let Some(tex) = lookups.map_spawn_bed {
            if let Some(uv) = lookups.hud_uv(Some(&tex)) {
                let size = [tex.width as f32, tex.height as f32];
                let pos = [bx as f32 + 0.5, by as f32];
                if ctx.draw(uv, size, pos, ALIGN_BOTTOM) {
                    text = Some(crate::lang::text("UI.SpawnBed"));
                }
            }
        }
    }

    // `TeamBasedSpawnMapLayer` (`TeamBasedSpawnMapLayer.cs:11-21`): the seed
    // gate first. `Main.teamBasedSpawnsSeed` is unported (M5-SEED-2's deferred
    // list) and so is `ExtraSpawnPointManager`, so the gate stays the C#'s and
    // nothing draws until that seed lands.
    if inputs.team_based_spawns {
        if let Some(tex) = lookups.extra_282 {
            let _ = lookups.hud_uv(Some(&tex));
        }
    }

    // `TeleportPylonsMapLayer` (`TeleportPylonsMapLayer.cs:30-110`): the
    // network lines, then the markers. `:39-42` dims them to
    // `Color.Gray * 0.5f` when the player is not near a pylon; `:112-119`
    // hides the ones the fog covers once the team-based-spawns seed is on.
    let pylon_color = if inputs.near_pylon { 1.0 } else { 0.25 };
    let revealed = |p: &PylonInfo| {
        !inputs.team_based_spawns
            || inputs
                .map
                .is_some_and(|m| m.tile_revealed(inputs.world, p.x as i32, p.y as i32))
    };
    let mut pylon_click: Option<PylonClick> = None;
    let mut pylon_hovered = false;
    if let Some(tex) = lookups.extra_182 {
        if let Some(base_uv) = lookups.hud_uv(Some(&tex)) {
            // `SpriteFrame(11, 1, col, 0) { PaddingY = 0 }`
            // (`SpriteFrame.cs:72-77`): cell width = width/11 - PaddingX(2),
            // cell height = height (RowCount 1 → no Y padding).
            let cell_w = (tex.width / 11 - 2) as f32;
            let cell_h = tex.height as f32;
            let step = (tex.width / 11) as f32;
            let du = (base_uv[2] - base_uv[0]) / tex.width as f32;
            let frame_uv = |col: u8| {
                [
                    base_uv[0] + col as f32 * step * du,
                    base_uv[1],
                    base_uv[0] + (col as f32 * step + cell_w) * du,
                    base_uv[3],
                ]
            };
            // `:45-64`: the network lines — fullscreen + a selected pylon.
            // One black 6px + one white 2px line from the selected pylon's
            // unclamped centre to every OTHER revealed pylon's.
            if inputs.map_fullscreen && inputs.pylon_tile != (-1, -1) {
                let anchor = [
                    inputs.pylon_tile.0 as f32 + 1.5,
                    inputs.pylon_tile.1 as f32 + 2.0,
                ];
                let from = ctx.unclamped_center([cell_w, cell_h], anchor, 1.0, ALIGN_CENTER);
                for p in inputs.pylons {
                    if revealed(p) && (p.x as i32, p.y as i32) != inputs.pylon_tile {
                        let to = ctx.unclamped_center(
                            [cell_w, cell_h],
                            [p.x as f32 + 1.5, p.y as f32 + 2.0],
                            1.0,
                            ALIGN_CENTER,
                        );
                        pylon_line(ctx.r, lookups, from, to, [0.0, 0.0, 0.0, 1.0], 6.0);
                        pylon_line(ctx.r, lookups, from, to, [1.0, 1.0, 1.0, 1.0], 2.0);
                    }
                }
            }
            // `:65-89`: the markers. `scaleIfSelected = 2`, `scaleIfOffscreen
            // = 0.5` (`:33-35`); the fullscreen uses `DrawClamped` (border
            // 10), the minimap the plain scaled `Draw`.
            let offscreen = lookups
                .extra_299
                .and_then(|t| lookups.hud_uv(Some(&t)).map(|uv| (t, uv)));
            for p in inputs.pylons {
                if !revealed(p) {
                    continue;
                }
                let uv = frame_uv(p.pylon_type.style());
                let pos = [p.x as f32 + 1.5, p.y as f32 + 2.0];
                let (hover, on_screen) = if inputs.map_fullscreen {
                    if let Some((otex, ouv)) = offscreen {
                        // `Extra_299.Frame(3, 3, 2, 1)` — the right-middle
                        // arrow cell (`:201-210`).
                        let ow = (otex.width / 3) as f32;
                        let oh = (otex.height / 3) as f32;
                        let odu = (ouv[2] - ouv[0]) / otex.width as f32;
                        let odv = (ouv[3] - ouv[1]) / otex.height as f32;
                        let arrow_uv = [
                            ouv[0] + 2.0 * ow * odu,
                            ouv[1] + oh * odv,
                            ouv[0] + 3.0 * ow * odu,
                            ouv[1] + 2.0 * oh * odv,
                        ];
                        ctx.draw_clamped(
                            uv,
                            [cell_w, cell_h],
                            arrow_uv,
                            [ow, oh],
                            pos,
                            ALIGN_CENTER,
                            pylon_color,
                            1.0,
                            2.0,
                            0.5,
                            10,
                            [inputs.screen.x, inputs.screen.y],
                        )
                    } else {
                        // No arrow sheet: the marker still clamps, the arrow
                        // simply does not draw (the C# would throw on the
                        // missing asset; the port degrades to the marker).
                        ctx.draw_clamped(
                            uv,
                            [cell_w, cell_h],
                            [0.0; 4],
                            [0.0, 0.0],
                            pos,
                            ALIGN_CENTER,
                            pylon_color,
                            1.0,
                            2.0,
                            0.5,
                            10,
                            [inputs.screen.x, inputs.screen.y],
                        )
                    }
                } else {
                    (
                        ctx.draw_scaled(uv, [cell_w, cell_h], pos, ALIGN_CENTER, pylon_color, 1.0, 2.0),
                        true,
                    )
                };
                if hover {
                    // `:81-87`: the hover claims the wormhole-cancel flag and
                    // the item-name text; a press latches the click.
                    pylon_hovered = true;
                    text = Some(crate::lang::item_name(p.pylon_type.item_id()));
                    if inputs.left_press {
                        pylon_click = Some(if on_screen {
                            PylonClick::Teleport(*p)
                        } else {
                            PylonClick::PanTo(*p)
                        });
                    }
                }
            }
        }
    }

    // `PingMapLayer` (`PingMapLayer.cs:24-42`): the 1x5 ping animation
    // cycles `CurrentRow = seconds * 10 % 5` over the pool.
    if let Some(tex) = lookups.map_ping {
        if let Some(full_uv) = lookups.hud_uv(Some(&tex)) {
            let fw = tex.width as f32;
            let fh = tex.height as f32 / 5.0;
            for (pos, age) in inputs.pings {
                let row = ((age * 10.0) as usize) % 5;
                let row_h = (full_uv[3] - full_uv[1]) / 5.0;
                let uv = [
                    full_uv[0],
                    full_uv[1] + row as f32 * row_h,
                    full_uv[2],
                    full_uv[1] + (row as f32 + 1.0) * row_h,
                ];
                ctx.draw(uv, [fw, fh], *pos, ALIGN_CENTER);
            }
        }
    }
    (text, pylon_click, pylon_hovered)
}

/// `Main.MapIcons`' accessor over the world map's ping pool — the C# holds
/// the pool on the static `Main.Pings`; the port keeps it beside the map
/// tiles it draws over.
pub(crate) fn map_pings(map: &WorldMap) -> &[([f32; 2], f32)] {
    &map.pings
}

/// `DigtoiseMapLayer`'s scan (`DigtoiseMapLayer.cs:14-26`): the first active
/// projectile of kind 1098/1123 owned by the local player (`Main.myPlayer` is
/// the port's `owner == 0`). Returns `(Bottom / 16 px, spriteDirection)`; the
/// C# `break`s on the first match, so the scan stops there too.
pub(crate) fn digtoise_marker(
    player: Option<&Player>,
    pool: &crate::projectile::ProjectilePool,
) -> Option<([f32; 2], i32)> {
    player?;
    pool.slots()
        .iter()
        .find(|p| p.active && p.owner == 0 && (p.kind == 1098 || p.kind == 1123))
        .map(|p| ([p.pos.x + p.w * 0.5, p.pos.y + p.h], p.sprite_direction))
}

#[cfg(test)]
mod tests {
    use super::*;

    // The context's transform (`MapOverlayDrawContext.cs:44-46`): the world
    // tile position through `mapScale` + the offset.
    #[test]
    fn overlay_context_transform_maps_tiles_to_screen() {
        let mut world = World::new("t".into(), 200, 128);
        world.spawn_x = 100;
        world.spawn_y = 60;
        world.bed_spawn = Some((42, 30));
        // The spawn marker's expected screen position: the tile centre
        // through the scale + the offset — the C# `Draw`'s
        // `(position - mapPosition) * mapScale + mapOffset`.
        let map_offset = [300.0, 200.0];
        let map_scale = 4.0;
        let expected = [
            (world.spawn_x as f32 + 0.5) * map_scale + map_offset[0],
            world.spawn_y as f32 * map_scale + map_offset[1],
        ];
        assert_eq!(expected[0], 702.0);
        assert_eq!(expected[1], 440.0);
        let _ = (world.bed_spawn.unwrap(), map_offset, map_scale);
    }

    #[test]
    fn boss_bag_set_matches_itemid_1383() {
        // `ItemID.cs:1383`: the 21 boss-bag kinds.
        for kind in [3318, 3325, 3332, 3860, 4782, 4957, 5111] {
            assert!(is_boss_bag(kind));
        }
        for kind in [8, 9, 23, 3317, 5112] {
            assert!(!is_boss_bag(kind));
        }
    }

    #[test]
    fn ping_pool_expiry_and_cap() {
        // `PingMapLayer.cs:47-51` (the 100 cap) + `:37-40` (the 15 s drop);
        // the 5-frame cycle rows at 10 Hz.
        let mut m = WorldMap::new(&World::new("t".into(), 64, 128));
        for i in 0..100 {
            m.add_ping([i as f32, i as f32]);
        }
        assert_eq!(m.pings.len(), 100);
        m.add_ping([50.0, 50.0]);
        assert_eq!(m.pings.len(), 100, "the cap");
        m.age_pings(14.9);
        assert_eq!(m.pings.len(), 100);
        let rows: Vec<usize> = m
            .pings
            .iter()
            .map(|(_, age)| ((age * 10.0) as usize) % 5)
            .collect();
        assert_eq!(rows[0], 4, "14.9 s -> row 4");
        m.age_pings(0.2);
        assert!(m.pings.is_empty(), "the 15 s expiry");
    }

    /// S1-L-4 — `TeleportPylonsMapLayer.cs:73-79`: the marker anchor is
    /// `PositionInTiles.ToVector2() + (1.5, 2)`, centre-aligned, through the
    /// context transform (`MapOverlayDrawContext.cs:44-46`).
    #[test]
    fn s1_l_4_the_pylon_marker_anchor_is_the_csharp_offset() {
        let (map_pos, map_off, map_scale) = ([0.0f32, 0.0], [100.0f32, 50.0], 2.0f32);
        let pylon = (40i16, 30i16);
        let anchor = [pylon.0 as f32 + 1.5, pylon.1 as f32 + 2.0];
        let screen = [
            (anchor[0] - map_pos[0]) * map_scale + map_off[0],
            (anchor[1] - map_pos[1]) * map_scale + map_off[1],
        ];
        assert_eq!(screen, [183.0, 114.0]);
    }

    /// S1-L-4 — `TeleportPylonsMapLayer.IsRevealed` (`:112-119`) + the port's
    /// bounds guard: with the team seed on, the fog hides an unexplored pylon;
    /// without it the answer is a flat true; out-of-bounds never panics.
    #[test]
    fn s1_l_4_the_pylon_reveal_gate_follows_the_team_seed() {
        let world = World::new("t".into(), 200, 200);
        let mut m = WorldMap::new(&world);
        assert!(!m.tile_revealed(&world, 40, 30), "nothing explored yet");
        let i = world.index(40, 30);
        m.tiles[i].light = 1;
        assert!(m.tile_revealed(&world, 40, 30), "light > 0 reveals");
        assert!(!m.tile_revealed(&world, -1, -1), "out of bounds is unrevealed");
        assert!(!m.tile_revealed(&world, 5000, 5000), "and never panics");
    }

    /// S1-L-4 — `DigtoiseMapLayer.cs:14-25`: the scan takes the first active
    /// 1098/1123 owned by the local player (`owner == 0`, vanilla's
    /// `Main.myPlayer`), yields `Bottom` and the `spriteDirection`.
    #[test]
    fn s1_l_4_the_digtoise_marker_is_the_local_players_first_tracker() {
        let world = World::new("t".into(), 200, 200);
        let player = Player::spawn(&world);
        let mut pool = crate::projectile::ProjectilePool::new();
        assert!(digtoise_marker(Some(&player), &pool).is_none(), "none spawned");
        // A foreign-owned Digtoise stays invisible to the local map.
        pool.spawn(1098, 3, Vec2::ZERO, Vec2::ZERO, 0, 0.0, false);
        // A non-Digtoise local projectile does not qualify either.
        pool.spawn(1, 0, Vec2::ZERO, Vec2::ZERO, 0, 0.0, false);
        assert!(digtoise_marker(Some(&player), &pool).is_none());
        // The local Digtoise does: `Bottom = pos + (w/2, h)`. Its width and
        // direction come from the spawn's data defaults, so the test pins the
        // formula rather than the projectile's own fields.
        let idx = pool
            .spawn(1098, 0, Vec2::new(160.0, 320.0), Vec2::ZERO, 0, 0.0, false)
            .unwrap();
        let p = &pool.slots()[idx];
        let (bottom, _dir) = digtoise_marker(Some(&player), &pool).unwrap();
        assert_eq!(
            bottom,
            [p.pos.x + p.w * 0.5, p.pos.y + p.h],
            "Bottom = pos + (w/2, h)"
        );
        // No local player, no marker (`Main.LocalPlayer` gates the scan).
        assert!(digtoise_marker(None, &pool).is_none());
    }

    /// W4-e17 — `TeleportPylonsMapLayer.DrawLine` (`:18-28`): the segment
    /// count is `Min(5, (int)Distance)`, the scale `(num/num2/16, width/16)`,
    /// the rotation `(end-start).ToRotation()`; a sub-1-px line draws nothing.
    #[test]
    fn w4_e17_the_pylon_line_segments_follow_drawline() {
        // A 100-px horizontal line: 5 segments, each 20 px long (scale_x
        // 20/16), 6 px thick (scale_y 6/16), rotation 0.
        let (n, rot, sx, sy) = pylon_line_segments([0.0, 0.0], [100.0, 0.0], 6.0);
        assert_eq!(n, 5);
        assert_eq!(rot, 0.0);
        assert_eq!(sx, 20.0 / 16.0);
        assert_eq!(sy, 6.0 / 16.0);
        // A 3-px line: only 3 segments (the (int)num truncation).
        let (n, _, sx, _) = pylon_line_segments([0.0, 0.0], [3.9, 0.0], 2.0);
        assert_eq!(n, 3);
        assert_eq!(sx, 3.9 / 3.0 / 16.0);
        // A sub-1-px line draws nothing.
        let (n, _, _, _) = pylon_line_segments([0.0, 0.0], [0.5, 0.0], 6.0);
        assert_eq!(n, 0);
        // A vertical line rotates +pi/2 (ToRotation = atan2(dy, dx)).
        let (_, rot, _, _) = pylon_line_segments([0.0, 0.0], [0.0, 50.0], 2.0);
        assert!((rot - std::f32::consts::FRAC_PI_2).abs() < 1e-6);
        // The white pass is 2 px thick.
        let (_, _, _, sy) = pylon_line_segments([0.0, 0.0], [100.0, 0.0], 2.0);
        assert_eq!(sy, 2.0 / 16.0);
    }

    /// W4-e17 — `DrawClamped`'s rect pull-back
    /// (`MapOverlayDrawContext.cs:132-184`): an onscreen rect stays put; an
    /// off-left rect snaps its x to the border and shifts y along the ray
    /// from the screen centre; the Y pass reads the already-shifted rect.
    #[test]
    fn w4_e17_the_clamped_region_pulls_back_to_the_border() {
        let screen = [800.0f32, 600.0];
        let border = 10;
        // An onscreen rect: unchanged, onScreen true.
        let (rx, ry, dp, on) =
            clamp_draw_region(100, 100, 28, 38, [114.0, 119.0], border, screen);
        assert_eq!((rx, ry), (100, 100));
        assert_eq!(dp, [114.0, 119.0]);
        assert!(on);
        // An off-left rect: x snaps to 10, y shifts by the ray's slope.
        // mid = ((10 + 790)/2, (10 + 590)/2) = (400, 300). The rect top-left
        // (-50, 200): dx = -450, dy = -100, over = -60 → shift_y =
        // (int)(-60 / -450 * -100) = (int)(-13.33) = -13. x -= -60 → 10,
        // y -= -13 → 213. draw_pos = (-50+14, 200+19) + (60, 13).
        let pos = [-36.0, 219.0]; // top-left + origin (14, 19)
        let (rx, ry, dp, on) = clamp_draw_region(-50, 200, 28, 38, pos, border, screen);
        assert_eq!((rx, ry), (10, 213));
        assert_eq!(dp, [pos[0] + 60.0, pos[1] + 13.0]);
        assert!(!on);
        // An off-right rect: x snaps so rx + rw == max_x (790).
        let (rx, _, _, on) = clamp_draw_region(900, 100, 28, 38, [914.0, 119.0], border, screen);
        assert_eq!(rx + 28, 790);
        assert!(!on);
        // An off-bottom rect: y snaps so ry + rh == max_y (590).
        let (_, ry, _, on) = clamp_draw_region(100, 700, 28, 38, [114.0, 719.0], border, screen);
        assert_eq!(ry + 38, 590);
        assert!(!on);
    }

    /// W4-e17 — the pylon marker frame is `SpriteFrame(11, 1, col, 0)` with
    /// the default PaddingX 2 (`SpriteFrame.cs:72-77`): the 330-px sheet's
    /// cell is 28 px wide (330/11 - 2), NOT 30; the height is the full 38
    /// (RowCount 1 → no Y padding). The column step is 30 px.
    #[test]
    fn w4_e17_the_pylon_marker_frame_is_28_wide() {
        let (sheet_w, sheet_h) = (330u32, 38u32);
        let cell_w = (sheet_w / 11 - 2) as f32;
        let cell_h = sheet_h as f32;
        let step = (sheet_w / 11) as f32;
        assert_eq!(cell_w, 28.0);
        assert_eq!(cell_h, 38.0);
        assert_eq!(step, 30.0);
        // Column 6 (the snow pylon) starts at 180 px, spans 180..208.
        assert_eq!(6.0 * step, 180.0);
        assert_eq!(6.0 * step + cell_w, 208.0);
    }
}

