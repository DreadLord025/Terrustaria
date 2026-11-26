//! The map's player pins, death markers, misc icons and revenge markers —
//! the `Main.DrawMap` passes the icon layers (`map_layers.rs`) and the NPC
//! heads (`map_npc_icons.rs`) sit between:
//!
//! - `Main.DrawPlayerMapIcon_CanBeSeen` (`Main.cs:56076-56097`) + the
//!   255-player head loops (fullscreen `:55931-55976`, minimap
//!   `:55712-55766`) — the local head rides
//!   `render/entities.rs::draw_player_map_icon`; the remote peers draw the
//!   same composite through their `PlayerNet` snapshots (the
//!   `render/tiles.rs:1218` convention).
//! - `Main.DrawPlayerDeathMarker` (`Main.cs:56446-56464`) + the
//!   `showLastDeath` loops (`:55977-55983`, `:55742-55765`) — the
//!   `MapDeath` cross and the `Game.PlayerDeathTime` hover.
//! - `Main.DrawMiscMapIcons` (`Main.cs:56466-56539`): the Potion-of-Return
//!   home/exit portals (`Extra_175`/`Extra_173`) and the last-hit golf
//!   ball (`Extra_176` + the ball's own projectile frame).
//! - `CoinLossRevengeSystem.RevengeMarker.DrawMapIcon`
//!   (`CoinLossRevengeSystem.cs:252-266`) + `DrawMapIcons` (`:482-496`) —
//!   the coin-tier marker over the cached carrier's despawn spot.
//!
//! W4-e17 draw-side lane. The `MapOverlayDrawContext` transform
//! (`(position - mapPosition) * mapScale + mapOffset`) is shared with
//! `map_layers.rs`; the misc/revenge passes floor the transformed position
//! (`Main.cs:56480`/`:56501`/`:56522`) and clip through
//! `Rectangle.Contains(point)` — the half-open `[x, x+w)` integer test.

use glam::Vec2;
use tr_render::SpriteRenderer;

use crate::assets;
use crate::net::PlayerNet;
use crate::player::Player;
use crate::revenge::RevengeMarker;

/// `Main.teamColor` (`Main.cs:6657-6662`): the six team border colours.
pub(crate) const TEAM_COLORS: [[f32; 4]; 6] = [
    [1.0, 1.0, 1.0, 1.0],
    [218.0 / 255.0, 59.0 / 255.0, 59.0 / 255.0, 1.0],
    [59.0 / 255.0, 218.0 / 255.0, 85.0 / 255.0, 1.0],
    [59.0 / 255.0, 149.0 / 255.0, 218.0 / 255.0, 1.0],
    [242.0 / 255.0, 221.0 / 255.0, 100.0 / 255.0, 1.0],
    [224.0 / 255.0, 100.0 / 255.0, 242.0 / 255.0, 1.0],
];

/// `Main.GetPlayerHeadBordersColor` (`Main.cs:56610-56629`). `net_mode` is
/// 0 singleplayer / 1 client / 2 host; the port's remote peers carry no
/// team/hostile sync (`PlayerNet`), so they read as team 0, not hostile —
/// the same values the world draw gives them.
pub(crate) fn player_head_borders_color(
    dead: bool,
    team: i32,
    hostile: bool,
    is_local: bool,
    local_team: i32,
    net_mode: i32,
) -> [f32; 4] {
    // `:56612-56615`: ghost/dead → Transparent (the port has no ghosts).
    if dead {
        return [0.0; 4];
    }
    // `:56616-56619`: singleplayer → teamColor[0] (White).
    if net_mode == 0 {
        return TEAM_COLORS[0];
    }
    let t = team.clamp(0, 5) as usize;
    if is_local {
        return TEAM_COLORS[t];
    }
    // `:56624-56627`: a hostile peer outside the local team → Transparent.
    if hostile && (team != local_team || team == 0) {
        return [0.0; 4];
    }
    TEAM_COLORS[t]
}

/// `Main.DrawPlayerMapIcon_CanBeSeen` (`Main.cs:56076-56097`): the local
/// player always; a peer needs `active && !ShouldNotDraw` (the port's
/// snapshots are active by construction and carry no `ShouldNotDraw`),
/// then the hostile/team gates.
pub(crate) fn player_map_icon_can_be_seen(
    is_local: bool,
    hostile: bool,
    team: i32,
    local_hostile: bool,
    local_team: i32,
) -> bool {
    if is_local {
        return true;
    }
    if !local_hostile && !hostile {
        return true;
    }
    team == local_team && team != 0
}

/// The fullscreen head-loop position (`Main.cs:55937-55944`): the C#
/// left-to-right order over `num`/`num2` (the map origin), `num12/13 = 0`.
/// `center_px` is `position + (width/2, gfxOffY + height/2)`.
pub(crate) fn fullscreen_pin_pos(center_px: [f32; 2], origin: [f32; 2], scale: f32) -> [f32; 2] {
    let x = center_px[0] / 16.0 * scale + origin[0] - 6.0 - 10.0 * scale;
    let y = center_px[1] / 16.0 * scale + origin[1] - 2.0 - (2.0 - scale / 5.0 * 2.0)
        - 10.0 * scale;
    [x, y]
}

/// The fullscreen head hover rect (`Main.cs:55945-55948`): the 28×28 box
/// at `(pos + (4, 2)) - 14 * headScale`.
pub(crate) fn head_hover_rect(pos: [f32; 2], head_scale: f32) -> [f32; 4] {
    [
        pos[0] + 4.0 - 14.0 * head_scale,
        pos[1] + 2.0 - 14.0 * head_scale,
        28.0 * head_scale,
        28.0 * head_scale,
    ]
}

/// The inclusive hover test the head/death loops use (`Main.cs:55955`,
/// `:56459-56462`): `mouse >= min && mouse <= max` on the float rect.
pub(crate) fn hover_inclusive(rect: [f32; 4], mouse: Vec2) -> bool {
    mouse.x >= rect[0]
        && mouse.x <= rect[0] + rect[2]
        && mouse.y >= rect[1]
        && mouse.y <= rect[1] + rect[3]
}

/// `Rectangle.Contains(Point)` (XNA): the half-open integer test —
/// `(int)point` inside `[x, x+w) × [y, y+h)`. The misc/revenge passes clip
/// and hover through it (`Main.cs:56481`/`:56486`,
/// `CoinLossRevengeSystem.cs:257`/`:265`).
pub(crate) fn rect_contains_point(rect: [f32; 4], p: [f32; 2]) -> bool {
    let (x, y) = (p[0] as i32, p[1] as i32);
    let (rx, ry) = (rect[0] as i32, rect[1] as i32);
    x >= rx && x < rx + rect[2] as i32 && y >= ry && y < ry + rect[3] as i32
}

/// `Utils.Floor(Vector2)` (`Utils.cs:1702-1707`): `vec.X = (int)vec.X` —
/// TRUNCATION toward zero, NOT `Math.Floor`; the two differ only for
/// negative coordinates. The misc portal/golf positions ride it
/// (`Main.cs:56480`/`:56501`/`:56522`). W4-f29 fix F3 (research/140 §3).
pub(crate) fn utils_floor(v: [f32; 2]) -> [f32; 2] {
    [v[0] as i32 as f32, v[1] as i32 as f32]
}

/// `Utils.CenteredRectangle` (`Utils.cs:947-950`): the `(int)` truncations
/// on the corner AND the size.
pub(crate) fn centered_rect(center: [f32; 2], size: [f32; 2]) -> [f32; 4] {
    [
        (center[0] - size[0] / 2.0) as i32 as f32,
        (center[1] - size[1] / 2.0) as i32 as f32,
        size[0] as i32 as f32,
        size[1] as i32 as f32,
    ]
}

/// The minimap head-loop position (`Main.cs:55718-55725`), in the
/// `MapScale` transform space: `((center)/16 - left/top) * scale +
/// origin - (6, 6 + (2 - scale/5*2)) + sub`. The caller multiplies by
/// `MapScale` on the way to the screen (the `minimap_origin_raw`
/// convention).
pub(crate) fn minimap_pin_pos(
    center_px: [f32; 2],
    left_top: [f32; 2],
    origin: [f32; 2],
    sub: [f32; 2],
    scale: f32,
) -> [f32; 2] {
    let x = (center_px[0] / 16.0 - left_top[0]) * scale + origin[0] - 6.0 + sub[0];
    let y = (center_px[1] / 16.0 - left_top[1]) * scale + origin[1] - 6.0
        - (2.0 - scale / 5.0 * 2.0)
        + sub[1];
    [x, y]
}

/// The minimap head gate (`Main.cs:55726`): strictly inside the panel
/// inset `(6, 6)`..`(width-16, height-14)`.
pub(crate) fn minimap_head_gate(pos: [f32; 2], panel: [f32; 4]) -> bool {
    pos[0] > panel[0] + 6.0
        && pos[0] < panel[0] + panel[2] - 16.0
        && pos[1] > panel[1] + 6.0
        && pos[1] < panel[1] + panel[3] - 14.0
}

/// The minimap death gate (`Main.cs:55752`): the `(8, 8)`..`(w-18, h-16)`
/// inset — different from the head gate's.
pub(crate) fn minimap_death_gate(pos: [f32; 2], panel: [f32; 4]) -> bool {
    pos[0] > panel[0] + 8.0
        && pos[0] < panel[0] + panel[2] - 18.0
        && pos[1] > panel[1] + 8.0
        && pos[1] < panel[1] + panel[3] - 16.0
}

/// The death-marker position (`Main.DrawPlayerDeathMarker`,
/// `Main.cs:56448-56453`): `(lastDeath/16 - left/top) * scale + origin -
/// (0, 2 - scale/5*2) - 10*scale`. The minimap loop's inline copy
/// (`:55746-55751`) adds `sub` instead of the `-10*scale` inset pair —
/// `sub` already carries it there (`num3/num4` are the raw origin).
pub(crate) fn death_marker_pos(
    death_px: [f32; 2],
    left_top: [f32; 2],
    origin: [f32; 2],
    scale: f32,
    inset: [f32; 2],
) -> [f32; 2] {
    let x = (death_px[0] / 16.0 - left_top[0]) * scale + origin[0] - inset[0];
    let y = (death_px[1] / 16.0 - left_top[1]) * scale + origin[1]
        - (2.0 - scale / 5.0 * 2.0)
        - inset[1];
    [x, y]
}

/// `Main.DrawPlayerDeathMarker`'s draw (`Main.cs:56454`): the whole
/// `MapDeath` sheet centred on `pos` at `head_scale`, `White * alpha`.
/// Returns the hover answer — the 28×28 box at `(4, 2) - 14*headScale`
/// (`:56455-56462`, NO further shift). The minimap's INLINE copy
/// (`:55755-55758`) additionally subtracts 4 on both axes (net `(0, -2)`);
/// `minimap` selects that arm. W4-f29 fix F5 (research/140 §3): the
/// fullscreen box must NOT ride the minimap's −4/−4.
/// Returns the hover answer (`:56455-56463` — the 28×28 box shifted
/// `(4, 2) - 14*headScale`, then `-4` on both axes: net `(0, -2)`).
pub(crate) fn draw_death_marker(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    pos: [f32; 2],
    head_scale: f32,
    alpha: f32,
    mouse: Vec2,
    minimap: bool,
) -> bool {
    let Some(sheet) = lookups.map_death else {
        return false;
    };
    let Some(uv) = lookups.hud_uv(Some(&sheet)) else {
        return false;
    };
    let size = [sheet.width as f32 * head_scale, sheet.height as f32 * head_scale];
    r.push_ui_sprite(
        [pos[0] - size[0] * 0.5, pos[1] - size[1] * 0.5],
        size,
        uv,
        [1.0, 1.0, 1.0, alpha],
    );
    // `:56455-56456`: num3 = num + 4 - 14*s, num4 = num2 + 2 - 14*s. The
    // `num3 -= 4; num4 -= 4` pair exists ONLY in the minimap inline copy
    // (`:55757-55758`) — the fullscreen fn has no such lines (W4-f29 F5).
    hover_inclusive(death_hover_rect(pos, head_scale, minimap), mouse)
}

/// The death-marker hover box: fullscreen `pos + (4, 2) - 14*s`
/// (`Main.cs:56455-56456`), minimap the same minus 4 on both axes
/// (`:55757-55758`). W4-f29 fix F5.
pub(crate) fn death_hover_rect(pos: [f32; 2], head_scale: f32, minimap: bool) -> [f32; 4] {
    let shift = if minimap { 4.0 } else { 0.0 };
    [
        pos[0] + 4.0 - 14.0 * head_scale - shift,
        pos[1] + 2.0 - 14.0 * head_scale - shift,
        28.0 * head_scale,
        28.0 * head_scale,
    ]
}

/// `Game.PlayerDeathTime` ("{0} died {1} ago", `Main.cs:55986`): the
/// non-abbreviated `Lang.LocalizedDuration(DateTime.Now - lastDeathTime,
/// abbreviated: false, showAllAvailableUnits: false)`.
pub(crate) fn death_hover_text(name: &str, last_death: std::time::SystemTime) -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(last_death)
        .map(|d| d.as_secs() as i32)
        .unwrap_or(0);
    crate::lang::text_args(
        "Game.PlayerDeathTime",
        &[
            name.to_string(),
            crate::ui::resources::localized_duration_full(secs, false),
        ],
    )
}

/// Build a peer's display player from its snapshot — the map twin of
/// `render/tiles.rs`'s remote draw (position + netOffset, facing, frames).
pub(crate) fn remote_pin_player(world: &tr_core::World, rp: &PlayerNet) -> Player {
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
    p
}

/// `Main.DrawMiscMapIcons` (`Main.cs:56466-56471`): the three passes in
/// order — the Potion-of-Return home portal, the exit portal, the last-hit
/// golf ball. `map_top_left`/`map_offset` are the C#'s
/// `mapTopLeft`/`mapX2Y2AndOff` pair (fullscreen: `Vector2.Zero` / the
/// `num - 10*scale` origin; minimap: `(num12, num13)` / `(num3+num10,
/// num4+num11)`). Returns the hovered marker's text.
pub(crate) struct MiscIconInputs<'a> {
    pub player: Option<&'a Player>,
    /// `Main.LocalGolfState.GetLastHitBall()` — the re-validated snapshot
    /// (`GolfState.cs:102-109`), resolved by the caller against the pool.
    pub golf_ball: Option<tr_core::golf_state::GolfBall>,
    pub mouse: Vec2,
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn draw_misc_map_icons(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    inputs: &MiscIconInputs,
    map_top_left: [f32; 2],
    map_offset: [f32; 2],
    clip: Option<[f32; 4]>,
    map_scale: f32,
    draw_scale: f32,
) -> Option<String> {
    let mut text = None;
    // `DrawMapIcons_PotionOfReturnHomePosition` (`:56494-56513`).
    if let Some(p) = inputs.player {
        if let Some(home) = p.potion_of_return_home_position {
            if let Some(t) = draw_return_portal(
                r, lookups, lookups.extra_175, home, p.height,
                map_top_left, map_offset, clip, map_scale, draw_scale,
                inputs.mouse, "GameUI.PotionOfReturnHomePortal",
            ) {
                text = Some(t);
            }
        }
        // `DrawMapIcons_PotionOfReturnAppearAfterUsePosition` (`:56473-56492`).
        if let Some(exit) = p.potion_of_return_original_use_position {
            if let Some(t) = draw_return_portal(
                r, lookups, lookups.extra_173, exit, p.height,
                map_top_left, map_offset, clip, map_scale, draw_scale,
                inputs.mouse, "GameUI.PotionOfReturnExitPortal",
            ) {
                text = Some(t);
            }
        }
    }
    // `DrawMapIcons_LastGolfballHit` (`:56515-56539`).
    if let Some(ball) = inputs.golf_ball {
        if let Some(t) = draw_golf_marker(
            r, lookups, &ball, map_top_left, map_offset, clip, map_scale,
            draw_scale, inputs.mouse,
        ) {
            text = Some(t);
        }
    }
    text
}

/// One Potion-of-Return portal (`Main.cs:56480-56490` / `:56501-56511`):
/// `((pos + (0, -height/2)) / 16 - mapTopLeft) * mapScale + offset`,
/// FLOORED; the clip is `Rectangle.Contains(point)`; the sheet draws
/// centred (`origin = size/2`) at `drawScale`, White; the hover is
/// `Utils.CenteredRectangle(vector, size * drawScale)`.
#[allow(clippy::too_many_arguments)]
fn draw_return_portal(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    sheet: Option<tr_render::SheetRect>,
    pos: (f32, f32),
    player_height: f32,
    map_top_left: [f32; 2],
    map_offset: [f32; 2],
    clip: Option<[f32; 4]>,
    map_scale: f32,
    draw_scale: f32,
    mouse: Vec2,
    hover_key: &str,
) -> Option<String> {
    let sheet = sheet?;
    let uv = lookups.hud_uv(Some(&sheet))?;
    // `:56480`/`:56501`: `Utils.Floor` is `(int)` TRUNCATION toward zero
    // (`Utils.cs:1702-1707` `vec.X = (int)vec.X`), NOT `Math.Floor` — the
    // two differ for negative coordinates. W4-f29 fix F3 (research/140 §3).
    let v = utils_floor([
        ((pos.0) / 16.0 - map_top_left[0]) * map_scale + map_offset[0],
        ((pos.1 - player_height / 2.0) / 16.0 - map_top_left[1]) * map_scale + map_offset[1],
    ]);
    let v = [
        (((pos.0) / 16.0 - map_top_left[0]) * map_scale + map_offset[0]).floor(),
        (((pos.1 - player_height / 2.0) / 16.0 - map_top_left[1]) * map_scale
            + map_offset[1])
            .floor(),
    ];
    if let Some(c) = clip {
        if !rect_contains_point(c, v) {
            return None;
        }
    }
    let size = [sheet.width as f32, sheet.height as f32];
    let drawn = [size[0] * draw_scale, size[1] * draw_scale];
    r.push_ui_sprite(
        [v[0] - drawn[0] * 0.5, v[1] - drawn[1] * 0.5],
        drawn,
        uv,
        [1.0; 4],
    );
    if rect_contains_point(centered_rect(v, drawn), [mouse.x, mouse.y]) {
        return Some(crate::lang::text(hover_key));
    }
    None
}

/// The last-hit golf ball (`Main.cs:56515-56539`): `Extra_176` (the flag
/// marker) then the ball's own `Projectile[type]` frame 0, both centred on
/// the floored transform of `ball.Center`; the hover rect is the MARKER's
/// (`rectangle2` is captured before the ball draw, `:56528`), the text is
/// `Projectile.Name` (`Projectile.cs:526` → `Lang.GetProjectileName`).
#[allow(clippy::too_many_arguments)]
fn draw_golf_marker(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    ball: &tr_core::golf_state::GolfBall,
    map_top_left: [f32; 2],
    map_offset: [f32; 2],
    clip: Option<[f32; 4]>,
    map_scale: f32,
    draw_scale: f32,
    mouse: Vec2,
) -> Option<String> {
    // `:56522`: `Utils.Floor` = `(int)` truncation, not `Math.Floor`
    // (W4-f29 fix F3, research/140 §3).
    let v = utils_floor([
        (ball.center.x / 16.0 - map_top_left[0]) * map_scale + map_offset[0],
        (ball.center.y / 16.0 - map_top_left[1]) * map_scale + map_offset[1],
    ]);
    let v = [
        ((ball.center.x / 16.0 - map_top_left[0]) * map_scale + map_offset[0]).floor(),
        ((ball.center.y / 16.0 - map_top_left[1]) * map_scale + map_offset[1]).floor(),
    ];
    if let Some(c) = clip {
        if !rect_contains_point(c, v) {
            return None;
        }
    }
    let sheet = lookups.extra_176?;
    let uv = lookups.hud_uv(Some(&sheet))?;
    let size = [sheet.width as f32, sheet.height as f32];
    let drawn = [size[0] * draw_scale, size[1] * draw_scale];
    r.push_ui_sprite(
        [v[0] - drawn[0] * 0.5, v[1] - drawn[1] * 0.5],
        drawn,
        uv,
        [1.0; 4],
    );
    let hover = centered_rect(v, drawn);
    // `:56529-56532`: the ball's own texture, `value.Frame()` = frame 0.
    if let Some((buv, bw, bh)) = lookups.proj_frame(ball.kind as u16, 0) {
        let bs = [bw * draw_scale, bh * draw_scale];
        r.push_ui_sprite(
            [v[0] - bs[0] * 0.5, v[1] - bs[1] * 0.5],
            bs,
            buv,
            [1.0; 4],
        );
    }
    if rect_contains_point(hover, [mouse.x, mouse.y]) {
        return Some(crate::lang::projectile_name(ball.kind));
    }
    None
}

/// `RevengeMarker.DrawMapIcon` (`CoinLossRevengeSystem.cs:252-266`): the
/// coin tier picks the sheet — `< 100` copper, `< 10000` silver,
/// `>= 1000000` platinum, else gold (`:262`; the `MapDeath` assignment at
/// `:261` is dead) — frame `(1, 8)` row 0, centred at `drawScale`, White.
/// The sheet is `TextureAssets.Coin[tier]`, the 1x8 ANIMATION strip
/// (`AssetInitializer.cs:369`), NOT the `Item_71..74` inventory icon —
/// W4-f29 fix F4 (research/140 §3). The clip is the transformed
/// (UNFLOORED) position's `ToPoint`; the hover is `Utils.CenteredRectangle`.
/// Returns the hover answer.
/// The clip is the transformed (UNFLOORED) position's `ToPoint`; the
/// hover is `Utils.CenteredRectangle`. Returns the hover answer.
pub(crate) fn draw_revenge_map_icon(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    marker: &RevengeMarker,
    map_top_left: [f32; 2],
    map_offset: [f32; 2],
    clip: Option<[f32; 4]>,
    map_scale: f32,
    draw_scale: f32,
    mouse: Vec2,
) -> bool {
    let v = [
        (marker.location.x / 16.0 - map_top_left[0]) * map_scale + map_offset[0],
        (marker.location.y / 16.0 - map_top_left[1]) * map_scale + map_offset[1],
    ];
    if let Some(c) = clip {
        if !rect_contains_point(c, v) {
            return false;
        }
    }
    let tier = if marker.coins_value < 100 {
        0
    } else if marker.coins_value < 10_000 {
        1
    } else if marker.coins_value >= 1_000_000 {
        3
    } else {
        2
    };
    // `TextureAssets.Coin[tier]` — the animation strip, not Item_71..74
    // (`CoinLossRevengeSystem.cs:262`, W4-f29 fix F4).
    let Some(sheet) = lookups.coin_anim[tier] else {
        return false;
    };
    // `value.Frame(1, 8)` — the first of the 8 animation rows.
    let fh = sheet.height / 8;
    let uv = lookups.health_bar_uv(&sheet, 0, 0, sheet.width, fh);
    let size = [sheet.width as f32, fh as f32];
    let drawn = [size[0] * draw_scale, size[1] * draw_scale];
    r.push_ui_sprite(
        [v[0] - drawn[0] * 0.5, v[1] - drawn[1] * 0.5],
        drawn,
        uv,
        [1.0; 4],
    );
    rect_contains_point(centered_rect(v, drawn), [mouse.x, mouse.y])
}

/// `CoinLossRevengeSystem.DrawMapIcons` (`:482-496`): every marker draws;
/// the LAST hovered one wins (the C# `result` overwrite). Returns its
/// index into `markers`.
#[allow(clippy::too_many_arguments)]
pub(crate) fn draw_revenge_map_icons(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    markers: &[RevengeMarker],
    map_top_left: [f32; 2],
    map_offset: [f32; 2],
    clip: Option<[f32; 4]>,
    map_scale: f32,
    draw_scale: f32,
    mouse: Vec2,
) -> Option<usize> {
    let mut hovered = None;
    for (i, m) in markers.iter().enumerate() {
        if draw_revenge_map_icon(
            r, lookups, m, map_top_left, map_offset, clip, map_scale, draw_scale, mouse,
        ) {
            hovered = Some(i);
        }
    }
    hovered
}

/// `Utils.CoinsSplit` (`Utils.cs:1048-1060`): the four denominations,
/// platinum first in the walk (index 3 down to 0).
pub(crate) fn coins_split(count: u64) -> [i64; 4] {
    let mut out = [0i64; 4];
    let mut acc = 0i64;
    let mut unit = 1_000_000i64;
    let count = count as i64;
    for i in (0..4).rev() {
        out[i] = (count - acc) / unit;
        acc += out[i] * unit;
        unit /= 100;
    }
    out
}

/// `RevengeMarker.UseMouseOver` (`CoinLossRevengeSystem.cs:268-273`): the
/// hover clears the text and draws `ItemSlot.DrawMoney(sb, "", mouse/drawScale
/// + (-28) + (4, 0), CoinsSplit(coins), horizontal: true)`
/// (`ItemSlot.cs:3652-3684`): four coin icons at `+ (24*i + 45, 50)` with
/// the stack counts under them (the 999 cap, `:3672-3675`). The empty
/// label's `DrawBorderStringFourWay` draws nothing.
pub(crate) fn draw_revenge_mouse_over(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    coins_value: u64,
    mouse: Vec2,
    draw_scale: f32,
) {
    let base = [mouse.x / draw_scale - 28.0 + 4.0, mouse.y / draw_scale];
    let split = coins_split(coins_value);
    let line = match &lookups.item_font {
        Some(f) => f.line_spacing,
        None => 8.0,
    };
    for i in 0..4usize {
        let num = split[3 - i].min(999);
        let v = [base[0] + (24 * i) as f32 + 45.0, base[1] + 50.0];
        // `CoinSlot.DrawItemCoin` (`CoinSlot.cs:240-253`):
        // `TextureAssets.Coin[coinType - 71]`, frame 0 of the 8-row strip,
        // centred, scale 1 (the 1024 size limit never bites a 16px coin).
        // W4-f29 fix F4: the animation strip, not the Item_71..74 icon.
        if let Some(sheet) = lookups.coin_anim[3 - i] {
            let fh = sheet.height / 8;
            let uv = lookups.health_bar_uv(&sheet, 0, 0, sheet.width, fh);
            r.push_ui_sprite(
                [v[0] - sheet.width as f32 * 0.5, v[1] - fh as f32 * 0.5],
                [sheet.width as f32, fh as f32],
                uv,
                [1.0; 4],
            );
        }
        // `:3682`: the count at `(v.X - 11, v.Y + lineSpacing*0.75)`,
        // origin `(0, lineSpacing)`, scale 0.75 — the origin shifts the
        // draw up by `lineSpacing * 0.75`, netting `v.Y`.
        let tx = v[0] - 11.0;
        let ty = v[1];
        let text = num.to_string();
        crate::ui::text::draw_border_string(
            r,
            lookups,
            crate::ui::text::BorderFont::Item,
            tx,
            ty,
            0.75,
            &[[-2.0, 0.0], [2.0, 0.0], [0.0, -2.0], [0.0, 2.0]],
            [1.0; 4],
            &text,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `Main.cs:56610-56629`: the border colour chain.
    #[test]
    fn w4_e17_head_borders_follow_the_team_chain() {
        // Singleplayer: White regardless of team.
        assert_eq!(
            player_head_borders_color(false, 3, false, true, 0, 0),
            TEAM_COLORS[0]
        );
        // Dead → Transparent.
        assert_eq!(player_head_borders_color(true, 1, false, true, 1, 1), [0.0; 4]);
        // MP local: own team colour.
        assert_eq!(
            player_head_borders_color(false, 2, false, true, 2, 1),
            TEAM_COLORS[2]
        );
        // MP hostile peer outside the team → Transparent.
        assert_eq!(player_head_borders_color(false, 0, true, false, 2, 1), [0.0; 4]);
        // MP same-team peer keeps the colour even when hostile.
        assert_eq!(
            player_head_borders_color(false, 2, true, false, 2, 1),
            TEAM_COLORS[2]
        );
    }

    /// `Main.cs:56076-56097`: the visibility gates.
    #[test]
    fn w4_e17_can_be_seen_gates() {
        assert!(player_map_icon_can_be_seen(true, true, 0, true, 0));
        assert!(player_map_icon_can_be_seen(false, false, 0, false, 0));
        // One side hostile, no team → hidden.
        assert!(!player_map_icon_can_be_seen(false, true, 0, false, 0));
        assert!(!player_map_icon_can_be_seen(false, false, 0, true, 0));
        // Same non-zero team → visible despite hostility.
        assert!(player_map_icon_can_be_seen(false, true, 3, true, 3));
        // Team 0 never qualifies.
        assert!(!player_map_icon_can_be_seen(false, true, 0, true, 0));
    }

    /// `Main.cs:55937-55944` at scale 4, origin (100, 50), centre
    /// (320, 640) px = tile (20, 40): x = 20*4 + 100 - 6 - 40 = 134;
    /// y = 40*4 + 50 - 2 - (2 - 1.6) - 40 = 167.6.
    #[test]
    fn w4_e17_fullscreen_pin_matches_the_csharp_order() {
        let p = fullscreen_pin_pos([320.0, 640.0], [100.0, 50.0], 4.0);
        assert_eq!(p[0], 134.0);
        assert!((p[1] - 167.6).abs() < 1e-4, "{}", p[1]);
    }

    /// W4-f29 — `Main.cs:55937-55944` vs `:55718-55725`: the two pin
    /// transforms, adapted onto e17's `fullscreen_pin_pos`/`minimap_pin_pos`
    /// (e17 pins the fullscreen order; this adds the minimap transform —
    /// the `-6` inset pair, the `-(2 - scale/5*2)` y-offset and the sub).
    #[test]
    fn w4_f29_head_pos_matches_the_two_modes() {
        let center = [160.0, 320.0];
        // Fullscreen: origin (200,300), scale 4.
        // x = 160/16*4 + 200 - 6 - 40 = 194;
        // y = 320/16*4 + 300 - 2 - (2 - 1.6) - 40 = 337.6.
        let p = fullscreen_pin_pos(center, [200.0, 300.0], 4.0);
        assert_eq!(p[0], 194.0);
        assert!((p[1] - 337.6).abs() < 1e-4, "{}", p[1]);
        // Minimap: left_top (5,10), origin (500,90), sub (-0.5,-0.25),
        // scale 2. x = (10-5)*2 + 500 - 6 - 0.5 = 503.5;
        // y = (20-10)*2 + 90 - 6 - (2 - 0.8) - 0.25 = 102.55.
        let p = minimap_pin_pos(center, [5.0, 10.0], [500.0, 90.0], [-0.5, -0.25], 2.0);
        assert_eq!(p[0], 503.5);
        assert!((p[1] - 102.55).abs() < 1e-4, "{}", p[1]);
    }

    /// `Main.cs:55726` vs `:55752`: the two insets differ.
    #[test]
    fn w4_e17_minimap_gates_use_their_own_insets() {
        let panel = [100.0, 100.0, 240.0, 240.0];
        // x = 107: head gate (>106) passes, death gate (>108) fails.
        assert!(minimap_head_gate([107.0, 200.0], panel));
        assert!(!minimap_death_gate([107.0, 200.0], panel));
        // The right edges: head < 324, death < 322.
        assert!(minimap_head_gate([323.0, 200.0], panel));
        assert!(!minimap_death_gate([323.0, 200.0], panel));
        // Strict: the boundary itself fails both.
        assert!(!minimap_head_gate([106.0, 200.0], panel));
        assert!(!minimap_death_gate([108.0, 200.0], panel));
    }

    /// `Utils.cs:947-950` + XNA `Rectangle.Contains`: the truncations and
    /// the half-open edge.
    #[test]
    fn w4_e17_centered_rect_truncates_and_contains_is_half_open() {
        let r = centered_rect([10.9, 20.9], [5.9, 7.9]);
        assert_eq!(r, [8.0, 17.0, 5.0, 7.0]);
        assert!(rect_contains_point(r, [8.0, 17.0]));
        assert!(rect_contains_point(r, [12.9, 23.9]));
        assert!(!rect_contains_point(r, [13.0, 23.0]), "x+w is outside");
        assert!(!rect_contains_point(r, [7.9, 17.0]));
    }

    /// W4-f29 fix F5 — `Main.cs:56455-56462` (`DrawPlayerDeathMarker`, read
    /// in full): the FULLSCREEN hover box is the head box `(4, 2) - 14*s`
    /// with NO further shift. The `-4/-4` pair exists only in the minimap
    /// inline copy (`:55757-55758`) — the e17 first pass wrongly applied
    /// the minimap's net `(0, -2)` box to both branches.
    /// `Main.cs:56455-56458`: the death hover box is the head box shifted
    /// (-4, -4) — net origin pos + (0, -2) - 14*s.
    #[test]
    fn w4_e17_death_hover_box_is_the_shifted_head_box() {
        let pos = [200.0, 300.0];
        let s = 0.5;
        let head = head_hover_rect(pos, s);
        // Fullscreen: identical to the head box (no -4).
        let full = death_hover_rect(pos, s, false);
        assert_eq!(full, head);
        assert_eq!(full, [pos[0] + 4.0 - 14.0 * s, pos[1] + 2.0 - 14.0 * s, 28.0 * s, 28.0 * s]);
        // Minimap: the extra -4 on both axes — net pos + (0, -2) - 14*s.
        let mini = death_hover_rect(pos, s, true);
        assert_eq!(mini[0], head[0] - 4.0);
        assert_eq!(mini[1], head[1] - 4.0);
        assert_eq!(mini[2], 28.0 * s);
        assert_eq!(mini[3], 28.0 * s);
    }

    /// W4-f29 fix F5 — `Main.cs:56448-56453` (fullscreen transform) vs
    /// `:55746-55751` (the minimap inline copy): the sub-tile offsets add
    /// and there is no `-10*scale` inset; the y-offset is
    /// `-(2 - scale/5*2)` = -1.2 at scale 2, NOT the head's `-2*scale/5`.
    #[test]
    fn w4_f29_death_marker_transform() {
        // Fullscreen: origin (200,300), left_top 0, scale 4, inset 10*scale.
        // x = 160/16*4 + 200 - 40 = 200; y = 320/16*4 + 300 - (2-1.6) - 40.
        let last = [160.0, 320.0];
        let p = death_marker_pos(last, [0.0, 0.0], [200.0, 300.0], 4.0, [40.0, 40.0]);
        assert_eq!(p[0], 200.0);
        assert!((p[1] - 339.6).abs() < 1e-4, "{}", p[1]);
        // Minimap: origin (500,90), left_top (5,10), sub (-0.5,-0.25),
        // scale 2 — the inset pair is the NEGATED sub (the caller passes
        // `[-sub.x, -sub.y]`, draw.rs). y = (320/16-10)*2 + 90 - 1.2 - 0.25.
        let got = death_marker_pos(last, [5.0, 10.0], [500.0, 90.0], 2.0, [0.5, 0.25]);
        assert_eq!(got[0], 509.5);
        assert_eq!(got[1], 108.55);
    }

    /// W4-f29 fix F3 — `Main.cs:56480`: the misc icon's `Utils.Floor`
    /// transform is `(int)` truncation (`Utils.cs:1702-1707`), not
    /// `Math.Floor`.
    #[test]
    fn w4_f29_misc_icon_position_is_floored() {
        // anchor (161.7, 325.9), height_offset -21, scale 2.5, offset (100,50).
        // x = (161.7/16)*2.5 + 100 = 25.265.. + 100 = 125.265 -> 125.
        // y = ((325.9-21)/16)*2.5 + 50 = 47.64.. + 50 = 97.64 -> 97.
        let anchor = [161.7f32, 325.9];
        let h_off = -21.0f32;
        let scale = 2.5f32;
        let off = [100.0f32, 50.0];
        let v = utils_floor([
            (anchor[0] / 16.0) * scale + off[0],
            ((anchor[1] + h_off) / 16.0) * scale + off[1],
        ]);
        assert_eq!(v, [125.0, 97.0]);
        // A negative coordinate truncates toward zero (not Math.Floor,
        // which would answer -126).
        assert_eq!(utils_floor([-125.265, -97.64]), [-125.0, -97.0]);
        // The death box: (pos.x + 4 - 14s, pos.y + 2 - 14s, 28s, 28s) — the
        // fullscreen box minus 4 on both axes is the minimap net `(0, -2)`.
        let pos = [100.0f32, 200.0];
        let s = 1.5f32;
        let head = death_hover_rect(pos, s, false);
        assert_eq!(head[0] - 4.0, pos[0] - 14.0 * s);
        assert_eq!(head[1] - 4.0, pos[1] - 2.0 - 14.0 * s);
        assert_eq!(head[2], 28.0 * s);
    }

    /// `CoinLossRevengeSystem.cs:262`: the coin tiers.
    #[test]
    fn w4_e17_revenge_coin_tiers() {
        let tier = |v: u64| {
            if v < 100 { 0 } else if v < 10_000 { 1 } else if v >= 1_000_000 { 3 } else { 2 }
        };
        assert_eq!(tier(99), 0);
        assert_eq!(tier(100), 1);
        assert_eq!(tier(9_999), 1);
        assert_eq!(tier(10_000), 2);
        assert_eq!(tier(999_999), 2);
        assert_eq!(tier(1_000_000), 3);
    }

    /// `Utils.cs:1048-1060`: the split walk.
    #[test]
    fn w4_e17_coins_split_matches_the_walk() {
        assert_eq!(coins_split(0), [0, 0, 0, 0]);
        assert_eq!(coins_split(1_234_567), [67, 45, 23, 1]);
        assert_eq!(coins_split(99), [99, 0, 0, 0]);
        assert_eq!(coins_split(100), [0, 1, 0, 0]);
    }

    /// `Main.cs:56448-56453`: the death position at scale 2, origin
    /// (50, 60), death (160, 320) px = tile (10, 20), inset 10*scale:
    /// x = 10*2 + 50 - 20 = 50; y = 20*2 + 60 - (2 - 0.8) - 20 = 78.8.
    #[test]
    fn w4_e17_death_marker_position() {
        let p = death_marker_pos([160.0, 320.0], [0.0, 0.0], [50.0, 60.0], 2.0, [20.0, 20.0]);
        assert_eq!(p[0], 50.0);
        assert!((p[1] - 78.8).abs() < 1e-4, "{}", p[1]);
    }
}
