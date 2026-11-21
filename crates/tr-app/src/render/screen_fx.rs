//! The full-screen scene effects — `ScreenObstruction`, `ScreenDarkness`,
//! `MoonlordDeathDrama`, the Paladin's Shield boundary, the Inferno rings and
//! the lens flare (`decompiled/Terraria/GameContent/Events/`,
//! `Main.DrawPaladinsShield`/`DrawInfernoRings`, `NextHorizonRenderer`).
//!
//! The update halves live in `tr_core::scene::SceneState` (pure state
//! machines); these are the draw halves. Sources:
//! `ScreenObstruction.cs:48-68`, `ScreenDarkness.cs:36-50`,
//! `MoonlordDeathDrama.cs:195-225`, `Main.cs:57196-57275` (the shield),
//! `Main.cs:35297-35313` (the sparkle), `Main.cs:64275-64324` (the rings),
//! `NextHorizonRenderer.cs:363-533` + `LensFlareElement.cs:28-50` (the flare),
//! plus the `Main.Draw` order at `Main.cs:62217` (`DrawBack`, right after the
//! scene overlays) and `Main.cs:62316-62324` (the rings, the shield, the flare,
//! `DrawFront`, `DrawWhite`, `Draw` — after the projectiles and the wires).
//!
//! Deviations (recorded in `docs/ANALYSIS.md`): `TextureAssets.MagicPixel` is
//! the renderer's built-in `white_uv` quad; the `MoonlordPiece` light average
//! reads the port's `LightMap` rather than `Lighting.GetColor`; the lens
//! flare's `GameShaders.Misc["LensFlare"]` sun-occlusion pixel mask and the
//! `LensFlareElement` advanced-shadow offset have no port (the flare draws at
//! full temporal intensity, shadow offset 0).

use glam::Vec2;

use tr_core::lighting::LightMap;
use tr_core::scene::SceneState;
use tr_core::TILE_PX;
use tr_render::sprite::SpriteRenderer;

use crate::assets::{self, Lookups};
use crate::player::Player;

/// `Images/Misc/MoonExplosion/*` in [`Lookups::moon_explosion`] order
/// (`MoonlordDeathDrama.cs:231-242`).
const PIECE_SHEETS: [&str; 4] = [
    "Misc/MoonExplosion/Spine",
    "Misc/MoonExplosion/Shoulder",
    "Misc/MoonExplosion/Torso",
    "Misc/MoonExplosion/Head",
];

/// `Color.Black * a` (XNA's byte product → floats): rgb stays 0, alpha is `a`.
fn black(a: f32) -> [f32; 4] {
    [0.0, 0.0, 0.0, a]
}

/// XNA `Color operator*(Color, float)` (`MonoGame Color.cs`): every channel
/// scales then truncates toward zero (`(int)(c * s)`), returned as bytes.
fn scale_color(c: [u8; 4], s: f32) -> [u8; 4] {
    let f = |x: u8| (x as f32 * s) as i32 as u8;
    [f(c[0]), f(c[1]), f(c[2]), f(c[3])]
}

/// Renderer tint from sRGB bytes (`[r/255, g/255, b/255, a/255]`).
fn tint(c: [u8; 4]) -> [f32; 4] {
    [
        c[0] as f32 / 255.0,
        c[1] as f32 / 255.0,
        c[2] as f32 / 255.0,
        c[3] as f32 / 255.0,
    ]
}

/// `Main.OurFavoriteColor` (`Main.cs:66748`): `new Color(255, 231, 69)`.
const OUR_FAVORITE_COLOR: [u8; 4] = [255, 231, 69, 255];

/// `Player.PaladinsShieldRange` (`Player.cs:844`, set `:57100`): 800 px.
const PALADINS_SHIELD_RANGE: f32 = 800.0;

/// `Utils.Remap` (`Utils.cs:345-348`), always clamped: `Lerp(toMin, toMax,
/// GetLerpValue(fromMin, fromMax, value, true))`.
fn remap(value: f32, from_min: f32, from_max: f32, to_min: f32, to_max: f32) -> f32 {
    let t = tr_core::scene::get_lerp_value(from_min, from_max, value, true);
    to_min + (to_max - to_min) * t
}

/// `MathHelper.Lerp` (XNA): the linear interpolation the ring alpha uses.
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// The `TextureAssets.MagicPixel` full-screen quad every effect uses:
/// `new Rectangle(-2, -2, screenWidth + 4, screenHeight + 4)`
/// (`ScreenDarkness.cs:40`, `:48`; `MoonlordDeathDrama.cs:223`).
fn full_screen(r: &mut SpriteRenderer, screen: Vec2, color: [f32; 4]) {
    r.push_fx_rect([-2.0, -2.0], [screen.x + 4.0, screen.y + 4.0], color);
}

/// `ScreenDarkness.DrawBack` (`ScreenDarkness.cs:36-42`): a plain black wash
/// drawn before the world (`Main.cs:62217`).
pub(crate) fn draw_darkness_back(r: &mut SpriteRenderer, screen: Vec2, state: &SceneState) {
    if state.screen_darkness != 0.0 {
        full_screen(r, screen, black(state.screen_darkness));
    }
}

/// `ScreenDarkness.DrawFront` (`ScreenDarkness.cs:44-50`): the same quad in
/// `frontColor` (Wall of Flesh black, Duke Fishron navy), after the entities.
pub(crate) fn draw_darkness_front(r: &mut SpriteRenderer, screen: Vec2, state: &SceneState) {
    if state.screen_darkness != 0.0 {
        let c = state.screen_darkness_front;
        full_screen(r, screen, [c[0], c[1], c[2], c[3] * state.screen_darkness]);
    }
}

/// `MoonlordDeathDrama.DrawWhite` (`MoonlordDeathDrama.cs:219-225`): the
/// death-drama flash, a white quad at `whitening` alpha.
pub(crate) fn draw_white(r: &mut SpriteRenderer, screen: Vec2, state: &SceneState) {
    if state.moonlord_whitening != 0.0 {
        full_screen(r, screen, [1.0, 1.0, 1.0, state.moonlord_whitening]);
    }
}

/// `ScreenObstruction.Draw` (`ScreenObstruction.cs:48-68`): the `Extra[49]`
/// diamond around the player plus four `Rectangle.Union` border bars, all
/// tinted `Color.Black * screenObstruction`.
///
/// `player_rect` is `Main.SceneMetrics.PerspectivePlayer.getRect()` in
/// framebuffer pixels (`x, y, w, h`) — the caller converts from world space.
pub(crate) fn draw_obstruction(
    r: &mut SpriteRenderer,
    lookups: &Lookups,
    screen: Vec2,
    player_rect: [f32; 4],
    amount: f32,
) {
    if amount == 0.0 {
        return;
    }
    let Some(sheet) = lookups.extra_49 else {
        return;
    };
    let color = black(amount);
    let num = sheet.width as f32;
    let num2 = 10.0f32;
    // `:55-57`: inflate to the diamond's width, then shift up by `num2`.
    let (mut w, mut h) = (player_rect[2], player_rect[3]);
    let (mut x, mut y) = (player_rect[0], player_rect[1]);
    let inflate_x = (num - w) / 2.0;
    let inflate_y = (num - h) / 2.0 + num2 / 2.0;
    x -= inflate_x;
    y -= inflate_y;
    w += inflate_x * 2.0;
    h += inflate_y * 2.0;
    y -= num2;
    let (left, top, right, bottom) = (x, y, x + w, y + h);
    // `:58-61` — `Rectangle.Union` of a 1×1 screen corner with a 1×1 player-edge
    // pixel: the axis-aligned bar spanning the two points.
    for b in [
        [0.0, 0.0, right - 1.0, top - 1.0],
        [screen.x - 1.0, 0.0, right, bottom - 1.0],
        [screen.x - 1.0, screen.y - 1.0, left, bottom],
        [0.0, screen.y - 1.0, left - 1.0, top],
    ] {
        let bx = b[0].min(b[2]);
        let by = b[1].min(b[3]);
        let bw = (b[0].max(b[2]) - bx).max(1.0);
        let bh = (b[1].max(b[3]) - by).max(1.0);
        r.push_fx_rect([bx, by], [bw, bh], color);
    }
    // `:66` — `spriteBatch.Draw(Extra[49], rect, color)`.
    let uv = assets::uv_rect(&sheet, r.atlas_size, 0, 0, sheet.width, sheet.height);
    r.push_fx_sprite_rot([left, top], [w, h], uv, color, 0.0);
}

/// `MoonlordPiece.GetLight` (`MoonlordDeathDrama.cs:65-84`): the mean of the
/// `Lighting.GetColor` samples over the ±5-tile box, white when it is empty.
fn piece_light(light: &LightMap, p: [f32; 2]) -> [f32; 3] {
    let (tx, ty) = ((p[0] / TILE_PX) as i32, (p[1] / TILE_PX) as i32);
    let mut sum = [0.0f32; 3];
    let mut num = 0.0f32;
    for i in tx - 5..=tx + 5 {
        for j in ty - 5..=ty + 5 {
            let c = light.get(i, j);
            for k in 0..3 {
                sum[k] += c[k];
            }
            num += 1.0;
        }
    }
    if num == 0.0 {
        return [1.0; 3];
    }
    [sum[0] / num, sum[1] / num, sum[2] / num]
}

/// `MoonlordDeathDrama.DrawPieces` (`MoonlordDeathDrama.cs:195-205`): the flung
/// body parts, filtered by `InDrawRange`.
pub(crate) fn draw_moonlord_pieces(
    r: &mut SpriteRenderer,
    lookups: &Lookups,
    light: &LightMap,
    camera: Vec2,
    zoom: f32,
    screen: Vec2,
    state: &SceneState,
) {
    let centre = [camera.x, camera.y];
    for piece in &state.moonlord_pieces {
        if !piece.in_draw_range(centre, [screen.x, screen.y], zoom) {
            continue;
        }
        let Some(idx) = PIECE_SHEETS.iter().position(|s| *s == piece.sheet) else {
            continue;
        };
        let Some(sheet) = lookups.moon_explosion[idx] else {
            continue;
        };
        let (w, h) = (sheet.width as f32, sheet.height as f32);
        let uv = assets::uv_rect(&sheet, r.atlas_size, 0, 0, sheet.width, sheet.height);
        let c = piece_light(light, piece.position);
        let snapped =
            crate::render::entities::snap_to_screen_pub(piece.position, camera, zoom, screen);
        // `MoonlordPiece.Draw` (`:55-58`): the sheet's own size, the piece's
        // rotation about `_origin`.
        r.push_sprite_origin_rot(
            snapped,
            [w * zoom, h * zoom],
            uv,
            [c[0], c[1], c[2], 1.0],
            piece.rotation,
            [piece.origin[0] * zoom, piece.origin[1] * zoom],
        );
    }
}

/// `MoonlordDeathDrama.DrawExplosions` (`MoonlordDeathDrama.cs:207-217`): the
/// 7-frame blasts at the flat `(255,255,255,127)` colour, `Frame(1, 7)` grid.
pub(crate) fn draw_moonlord_explosions(
    r: &mut SpriteRenderer,
    lookups: &Lookups,
    camera: Vec2,
    zoom: f32,
    screen: Vec2,
    state: &SceneState,
) {
    let Some(sheet) = lookups.moon_explosion[4] else {
        return;
    };
    let centre = [camera.x, camera.y];
    let fw = sheet.width;
    let fh = sheet.height / 7;
    for blast in &state.moonlord_explosions {
        if !blast.in_draw_range(centre, [screen.x, screen.y], zoom) {
            continue;
        }
        let uv = assets::uv_rect(
            &sheet,
            r.atlas_size,
            0,
            (blast.frame_index() * fh) as i32,
            fw,
            fh,
        );
        let snapped =
            crate::render::entities::snap_to_screen_pub(blast.position, camera, zoom, screen);
        r.push_sprite_origin_rot(
            snapped,
            [fw as f32 * zoom, fh as f32 * zoom],
            uv,
            blast.light(),
            0.0,
            [fw as f32 * 0.5 * zoom, fh as f32 * 0.5 * zoom],
        );
    }
}

/// `Main.DrawInfernoRings`' animation step (`Main.cs:64288-64312`): advance
/// the scale/rotation counters (only while `allow_emit`), pick the breathing
/// scale `num` (reset to 0.8 at 1) and wrap the rotation at ±2π. Returns
/// `(num, rot)`; the counters are mutated in place like the C# fields.
fn inferno_ring_step(scale: &mut f32, rot: &mut f32, allow_emit: bool) -> (f32, f32) {
    if allow_emit {
        *scale += 0.004;
    }
    let num = if *scale < 1.0 {
        *scale
    } else {
        *scale = 0.8;
        *scale
    };
    if allow_emit {
        *rot += 0.05;
    }
    let tau = std::f32::consts::TAU;
    if *rot > tau {
        *rot -= tau;
    }
    if *rot < -tau {
        *rot += tau;
    }
    (num, *rot)
}

/// `Main.DrawInfernoRings`' per-ring terms (`Main.cs:64313-64321`): the ring
/// scale `num4` (wrapped down by 0.2 above 1) and the brightness `num5`
/// (`Lerp(0.8, 0, |num4 - 0.9| * 10)`).
fn inferno_ring_terms(num: f32, j: i32) -> (f32, f32) {
    let num2 = 0.1f32;
    let num3 = 0.9f32;
    let mut num4 = num + num2 * j as f32;
    if num4 > 1.0 {
        num4 -= num2 * 2.0;
    }
    let num5 = lerp(0.8, 0.0, (num4 - num3).abs() * 10.0);
    (num4, num5)
}

/// `Main.DrawInfernoRings` (`Main.cs:64275-64324`): the Inferno Potion's three
/// breathing halo rings around the player. The animation counters
/// (`flameRingScale`/`flameRingRot`) advance only while
/// `FocusHelper.AllowPlayerToEmitEffects` (`FocusHelper.cs:29`,
/// `!WorldUpdateStepper.DrawnThisUpdate`) — the port passes that flag in.
///
/// The C# walks all 255 player slots; the port draws the local player (the
/// single simulated one). `outOfRange` has no port (the local player is always
/// in range). The ring sheet is `FlameRing`, 400×1200 — three 400×400 rows.
pub(crate) fn draw_inferno_rings(
    r: &mut SpriteRenderer,
    lookups: &Lookups,
    player: &mut Player,
    camera: Vec2,
    zoom: f32,
    screen: Vec2,
    allow_emit: bool,
) {
    // `:64280`: active, in range, inferno, alive.
    if !player.inferno || player.dead {
        return;
    }
    let Some(sheet) = lookups.flame_ring else {
        return;
    };
    let (num, rot) = inferno_ring_step(
        &mut player.flame_ring_scale,
        &mut player.flame_ring_rot,
        allow_emit,
    );
    let (cx, cy) = player.center();
    let pos = [
        (cx - camera.x) * zoom + screen.x * 0.5,
        (cy - camera.y) * zoom + screen.y * 0.5,
    ];
    // `:64313-64322`: three rings, each a 400×400 row of the sheet.
    for j in 0..3 {
        let (num4, num5) = inferno_ring_terms(num, j);
        // `new Color(num5, num5, num5, num5 / 2)` — the float ctor normalises,
        // so the tint is `(num5, num5, num5, num5/2)` directly (no byte step).
        let color = [num5, num5, num5, num5 / 2.0];
        let uv = assets::uv_rect(&sheet, r.atlas_size, 0, 400 * j, 400, 400);
        let ring_rot = rot + std::f32::consts::FRAC_PI_3 * j as f32;
        // `:64321`: origin (200, 200), scale num4, about the player's centre.
        r.push_fx_sprite_origin_rot(
            pos,
            [400.0 * num4 * zoom, 400.0 * num4 * zoom],
            uv,
            color,
            ring_rot,
            [200.0 * num4 * zoom, 200.0 * num4 * zoom],
        );
    }
}

/// `Main.DrawPrettyStarSparkle` (`Main.cs:35297-35313`): the four-point star
/// glint the paladin-shield boundary plants on the defended player. Four
/// `Extra[98]` quads — two at `drawColor * 0.5` and two at `shineColor *
/// opacity * 0.5` (alpha forced 0, additive), crossed at 0 and π/2, the second
/// pair at 0.6× scale. All sizes/origins are in screen pixels (the caller
/// passes the zoom-scaled sheet).
#[allow(clippy::too_many_arguments)]
fn draw_pretty_star_sparkle(
    r: &mut SpriteRenderer,
    lookups: &Lookups,
    opacity: f32,
    drawpos: [f32; 2],
    draw_color: [u8; 4],
    shine_color: [u8; 4],
    flare_counter: f32,
    fade_in_start: f32,
    fade_in_end: f32,
    fade_out_start: f32,
    fade_out_end: f32,
    rotation: f32,
    scale: [f32; 2],
    fatness: [f32; 2],
    zoom: f32,
) {
    let Some(sheet) = lookups.extra_98 else {
        return;
    };
    let (sw, sh) = (sheet.width as f32 * zoom, sheet.height as f32 * zoom);
    let uv = assets::uv_rect(&sheet, r.atlas_size, 0, 0, sheet.width, sheet.height);
    let origin = [sw * 0.5, sh * 0.5];
    // `:35300-35301`: shineColor * opacity * 0.5, alpha forced 0 (additive).
    let mut color = scale_color(scale_color(shine_color, opacity), 0.5);
    color[3] = 0;
    // `:35303`: drawColor * 0.5.
    let color2 = scale_color(draw_color, 0.5);
    // `:35304`: the fade window product.
    let num = tr_core::scene::get_lerp_value(fade_in_start, fade_in_end, flare_counter, true)
        * tr_core::scene::get_lerp_value(fade_out_end, fade_out_start, flare_counter, true);
    // `:35305-35306`: the two axis scales.
    let vector = [fatness[0] * 0.5 * num, scale[0] * num];
    let vector2 = [fatness[1] * 0.5 * num, scale[1] * num];
    // `:35307-35308`: both colours fade by num.
    let color = tint(scale_color(color, num));
    let color2 = tint(scale_color(color2, num));
    let half_pi = std::f32::consts::FRAC_PI_2;
    // `:35309-35312`: the four crossed quads.
    let quad = |r: &mut SpriteRenderer, c: [f32; 4], rot: f32, s: [f32; 2]| {
        r.push_fx_sprite_origin_rot(drawpos, [sw * s[0], sh * s[1]], uv, c, rot, origin);
    };
    quad(r, color, half_pi + rotation, vector);
    quad(r, color, rotation, vector2);
    quad(r, color2, half_pi + rotation, [vector[0] * 0.6, vector[1] * 0.6]);
    quad(r, color2, rotation, [vector2[0] * 0.6, vector2[1] * 0.6]);
}

/// One peer the `DrawPaladinsShield` loops read (`Main.cs:57205-57231`): the
/// fields `CanDefendWithPaladinsShield` + `MountedCenter` need. W4-d70
/// DOC-SWEEP: the peers are the `PlayerNet` snapshots (team/dead ride the
/// `13`/`12`/`45` wires, the shield flag re-derives from the synced
/// inventory — `PlayerNet::paladin_shield`); single-player passes an empty
/// slice (the C# skips `Main.myPlayer`).
#[derive(Clone, Copy)]
pub(crate) struct PaladinPeer {
    pub mounted_center: [f32; 2],
    pub team: i32,
    pub has_paladin_shield: bool,
    pub dead: bool,
    pub hp: i32,
    pub max_hp: i32,
}

impl PaladinPeer {
    /// `Player.CanDefendWithPaladinsShield` (`Player.cs:38188-38195`) for a
    /// peer: active (the snapshot exists), alive, shield-bearing, matching
    /// non-zero team, above 25% life.
    fn can_defend(&self, other_team: i32) -> bool {
        !self.dead
            && self.has_paladin_shield
            && self.team > 0
            && self.team == other_team
            && self.hp as f32 > self.max_hp as f32 * 0.25
    }
}

/// `Main.DrawPaladinsShield` (`Main.cs:57196-57232`): the two boundary loops.
/// The local player's shield reaches out to same-team shield-bearing peers
/// (`hasPaladinShield`), and a `defendedByPaladin` local player is reached by
/// peers who can defend. Single-player has no peers, so both loops no-op.
pub(crate) fn draw_paladins_shield(
    r: &mut SpriteRenderer,
    lookups: &Lookups,
    local: &Player,
    peers: &[PaladinPeer],
    camera: Vec2,
    zoom: f32,
    screen: Vec2,
) {
    // `:57199-57202`: a dead local player draws nothing.
    if local.dead {
        return;
    }
    let (lcx, lcy) = local.mounted_center();
    let local_center = [lcx, lcy];
    let to_screen = |p: [f32; 2]| {
        [
            (p[0] - camera.x) * zoom + screen.x * 0.5,
            (p[1] - camera.y) * zoom + screen.y * 0.5,
        ]
    };
    // `:57203-57216`: the local shield defends each qualifying peer.
    if local.has_paladin_shield {
        for peer in peers {
            if !peer.dead && local.can_defend_with_paladins_shield(peer.team) {
                draw_paladins_shield_boundary(
                    r,
                    lookups,
                    to_screen(local_center),
                    to_screen(peer.mounted_center),
                    zoom,
                );
            }
        }
    }
    // `:57217-57231`: a defended local player is reached by defending peers.
    if !local.defended_by_paladin {
        return;
    }
    for peer in peers {
        if peer.can_defend(local.team) {
            draw_paladins_shield_boundary(
                r,
                lookups,
                to_screen(peer.mounted_center),
                to_screen(local_center),
                zoom,
            );
        }
    }
}

/// `Main.DrawPaladinsShieldBoundary` (`Main.cs:57234-57275`): the arc of
/// `Extra[98]` sprites around the defender, the sparkle on the defended, and
/// the Paladin's Shield item (938) glint. `from`/`to` are screen pixels.
fn draw_paladins_shield_boundary(
    r: &mut SpriteRenderer,
    lookups: &Lookups,
    from: [f32; 2],
    to: [f32; 2],
    zoom: f32,
) {
    let Some(sheet) = lookups.extra_98 else {
        return;
    };
    // `:57236-57241`: the distance remap gates the whole boundary.
    let dx = to[0] - from[0];
    let dy = to[1] - from[1];
    let num = (dx * dx + dy * dy).sqrt();
    let range = PALADINS_SHIELD_RANGE * zoom;
    let from_value = (num - range).abs();
    let flag = num < range;
    let num2 = 1.0f32;
    let num3 = remap(from_value, range * 0.15, 0.0, 0.0, 1.0);
    if num3 <= 0.0 {
        return;
    }
    let num4 = num3;
    let num5 = dy.atan2(dx); // `(to - from).ToRotation()`
    let (sw, sh) = (sheet.width as f32 * zoom, sheet.height as f32 * zoom);
    let uv = assets::uv_rect(&sheet, r.atlas_size, 0, 0, sheet.width, sheet.height);
    let origin = [sw * 0.5, sh * 0.5];
    let num6 = 0.33f32;
    let num7 = if flag { 0.5 } else { 0.25 };
    // `:57251-57258`: the favourite-coloured arc, scale (0.5, 1.5).
    let arc_color = tint(scale_color(scale_color(OUR_FAVORITE_COLOR, num6), num3));
    let mut num8 = -0.01f32;
    while num8 <= 0.01 {
        let num9 = num5 + std::f32::consts::TAU * num8 * 0.5;
        let (s, c) = num9.sin_cos();
        let vector = [from[0] + c * range, from[1] + s * range];
        r.push_fx_sprite_origin_rot(
            vector,
            [sw * 0.5, sh * 1.5],
            uv,
            arc_color,
            num9,
            origin,
        );
        num8 += 0.005;
    }
    // `:57259-57263`: the white arc, scale (0.5, 1.5) * 0.5.
    let white = [255u8, 255, 255, 255];
    let white_color = tint(scale_color(scale_color(scale_color(white, num3), num6), num7));
    let mut num10 = -0.01f32;
    while num10 <= 0.01 {
        let num11 = num5 + std::f32::consts::TAU * num10;
        let (s, c) = num11.sin_cos();
        let vector = [from[0] + c * range, from[1] + s * range];
        r.push_fx_sprite_origin_rot(
            vector,
            [sw * 0.25, sh * 0.75],
            uv,
            white_color,
            num11,
            origin,
        );
        num10 += 0.005;
    }
    // `:57264`: the sparkle on the defended player.
    let draw_color = scale_color(scale_color(white, num2), num7);
    draw_pretty_star_sparkle(
        r,
        lookups,
        num2,
        to,
        draw_color,
        OUR_FAVORITE_COLOR,
        num4,
        0.0,
        1.0,
        1.0,
        2.0,
        0.0,
        [1.0, 1.5],
        [1.0, 1.0],
        zoom,
    );
    // `:57265-57273`: the Paladin's Shield item (938) glint.
    if let Some(item) = lookups.item_icons.get(&938) {
        let mut color3 = OUR_FAVORITE_COLOR;
        color3[3] = 200;
        let color3 = tint(scale_color(color3, num4));
        // `num5.ToRotationVector2() * Min(range - num, 0)` — non-zero only
        // when the defended player stands beyond the shield range.
        let (s, c) = num5.sin_cos();
        let push = (range - num).min(0.0);
        let vector2 = [c * push, s * push];
        let iuv = assets::uv_rect(item, r.atlas_size, 0, 0, item.width, item.height);
        let iw = item.width as f32 * zoom;
        let ih = item.height as f32 * zoom;
        let k = if flag { 1.5 } else { 1.0 };
        r.push_fx_sprite_origin_rot(
            [to[0] + vector2[0], to[1] + vector2[1]],
            [iw * k, ih * k],
            iuv,
            color3,
            0.0,
            [iw * 0.5, ih * 0.5],
        );
    }
}

/// One `LensFlareElement` row (`LensFlareElement.cs:8-26`): the texture index
/// into [`Lookups::sunflare_lens`] plus the repeat/scale/distance/colour
/// ladder. `Default` is the C#'s `default(LensFlareElement)` (all zero).
#[derive(Clone, Copy)]
struct LensElement {
    /// Index into `lookups.sunflare_lens` (0 blurry, 1 sharp, 2 bokeh, 3 spectra).
    tex: usize,
    repeat_times: i32,
    scale_start: f32,
    scale_over_index: f32,
    distance_start: f32,
    distance_along_index: f32,
    /// The element's sRGB colour, alpha included (the C# `Color` ctor).
    color: [u8; 4],
    intensity_over_index: f32,
    rotation: f32,
}

/// `Vector2.SafeNormalize` (`Vector2.cs`): the unit vector, or `default`
/// when the length is zero.
fn safe_normalize(v: [f32; 2], default: [f32; 2]) -> [f32; 2] {
    let len = (v[0] * v[0] + v[1] * v[1]).sqrt();
    if len == 0.0 {
        default
    } else {
        [v[0] / len, v[1] / len]
    }
}

/// `LensFlareElement.Draw`'s shadow offset (`LensFlareElement.cs:34-37`):
/// `v = GetAdvancedShadow(0).Position - GetAdvancedShadow(min(4, count-1))
/// .Position` — the player's last ~4 ticks of movement — then
/// `Dot(v.SafeNormalize(UnitX), (sun - center).SafeNormalize(-UnitY)) *
/// v.Length()`. W4-d70 DOC-SWEEP: the trail is ported
/// (`Player::update_advanced_shadows`); the offset shifts the flare chain
/// along the sun axis by `-0.0002 * num` of the lerp fraction (`:42`).
fn lens_shadow_offset(player: &Player, sun: [f32; 2], center: [f32; 2]) -> f32 {
    let newest = player.get_advanced_shadow(0);
    let oldest = player.get_advanced_shadow(4.min(player.available_advanced_shadows_count - 1));
    let v = [newest[0] - oldest[0], newest[1] - oldest[1]];
    let v_len = (v[0] * v[0] + v[1] * v[1]).sqrt();
    let v_unit = safe_normalize(v, [1.0, 0.0]);
    let sun_dir = safe_normalize([sun[0] - center[0], sun[1] - center[1]], [0.0, -1.0]);
    (v_unit[0] * sun_dir[0] + v_unit[1] * sun_dir[1]) * v_len
}

/// `LensFlareElement.Draw` (`LensFlareElement.cs:28-50`): the repeated flare
/// sprites lerped from the sun toward the screen centre, shifted by the
/// advanced-shadow offset (`:34-37`, `lens_shadow_offset`).
#[allow(clippy::too_many_arguments)]
fn draw_lens_element(
    r: &mut SpriteRenderer,
    lookups: &Lookups,
    e: &LensElement,
    sun: [f32; 2],
    center: [f32; 2],
    intensity: f32,
    screen_position_y: f32,
    zoom: f32,
    shadow_offset: f32,
) {
    if intensity == 0.0 {
        return;
    }
    let Some(sheet) = lookups.sunflare_lens[e.tex] else {
        return;
    };
    let (sw, sh) = (sheet.width as f32 * zoom, sheet.height as f32 * zoom);
    let uv = assets::uv_rect(&sheet, r.atlas_size, 0, 0, sheet.width, sheet.height);
    let origin = [sw * 0.5, sh * 0.5];
    // `:43`: `(screenCenter - sunPosition).ToRotation()`.
    let base_rot = (center[1] - sun[1]).atan2(center[0] - sun[0]);
    for i in 0..e.repeat_times {
        let fi = i as f32;
        let scale = e.scale_start + e.scale_over_index * fi;
        // `:41`: `Color * (1 + IntensityOverIndex * i) * intensity`.
        let c = scale_color(scale_color(e.color, 1.0 + e.intensity_over_index * fi), intensity);
        // `:42`: the lerp fraction, `(d + num * -0.0002f) % 1f * 2f`.
        let frac =
            (e.distance_start + e.distance_along_index * fi + shadow_offset * -0.0002) % 1.0 * 2.0;
        let position = [
            sun[0] + (center[0] - sun[0]) * frac,
            sun[1] + (center[1] - sun[1]) * frac,
        ];
        // `:43-47`: the rotation, plus the screen-Y drift when unrotated.
        let mut rot = base_rot + e.rotation;
        if e.rotation == 0.0 {
            rot += screen_position_y * 0.001;
        }
        r.push_fx_sprite_origin_rot(
            position,
            [sw * scale, sh * scale],
            uv,
            tint(c),
            rot,
            origin,
        );
    }
}

/// `NextHorizonRenderer.DrawSunsetFlare` (`NextHorizonRenderer.cs:401-465`):
/// the seven sunset elements, in order.
#[allow(clippy::too_many_arguments)]
fn draw_sunset_flare(
    r: &mut SpriteRenderer,
    lookups: &Lookups,
    sun: [f32; 2],
    center: [f32; 2],
    intensity: f32,
    screen_position_y: f32,
    zoom: f32,
    shadow_offset: f32,
) {
    if intensity <= 0.01 {
        return;
    }
    let amber = scale_color([43, 32, 0, 0], 120.0 / 255.0);
    let elements = [
        // `:407-415`: blurry point ×3.
        LensElement { tex: 0, repeat_times: 3, scale_start: 0.3, scale_over_index: -0.04, distance_start: 0.33, distance_along_index: 0.05, color: amber, intensity_over_index: -0.125, rotation: 0.0 },
        // `:417-425`: sharp point ×3.
        LensElement { tex: 1, repeat_times: 3, scale_start: 0.3, scale_over_index: 0.04, distance_start: 0.03, distance_along_index: 0.05, color: amber, intensity_over_index: -0.125, rotation: 0.0 },
        // `:427-432`: blurry point ×1, red.
        LensElement { tex: 0, repeat_times: 1, scale_start: 0.3, scale_over_index: 0.0, distance_start: 0.41, distance_along_index: 0.0, color: scale_color([255, 0, 65, 0], 30.0 / 255.0), intensity_over_index: 0.0, rotation: 0.0 },
        // `:434-439`: bokeh ×1, white.
        LensElement { tex: 2, repeat_times: 1, scale_start: 0.3, scale_over_index: 0.0, distance_start: 0.475, distance_along_index: 0.0, color: scale_color([255, 255, 255, 0], 40.0 / 255.0), intensity_over_index: 0.0, rotation: 0.0 },
        // `:441-448`: bokeh ×6.
        LensElement { tex: 2, repeat_times: 6, scale_start: 0.24, scale_over_index: -0.04, distance_start: 0.225, distance_along_index: 0.04, color: scale_color([255, 255, 255, 0], 20.0 / 255.0), intensity_over_index: 0.0, rotation: 0.0 },
        // `:450-455`: blurry point ×1, orange.
        LensElement { tex: 0, repeat_times: 1, scale_start: 1.0, scale_over_index: 0.0, distance_start: 0.6, distance_along_index: 0.0, color: scale_color([255, 157, 0, 0], 40.0 / 255.0), intensity_over_index: 0.0, rotation: 0.0 },
        // `:457-463`: spectra ×1, rotated π.
        LensElement { tex: 3, repeat_times: 1, scale_start: 0.4, scale_over_index: 0.0, distance_start: 0.65, distance_along_index: 0.0, color: scale_color([255, 255, 255, 0], 10.0 / 255.0), intensity_over_index: 0.0, rotation: std::f32::consts::PI },
    ];
    for e in &elements {
        draw_lens_element(r, lookups, e, sun, center, intensity, screen_position_y, zoom, shadow_offset);
    }
}

/// `NextHorizonRenderer.DrawSunriseFlare` (`NextHorizonRenderer.cs:467-532`):
/// the seven sunrise elements, in order.
#[allow(clippy::too_many_arguments)]
fn draw_sunrise_flare(
    r: &mut SpriteRenderer,
    lookups: &Lookups,
    sun: [f32; 2],
    center: [f32; 2],
    intensity: f32,
    screen_position_y: f32,
    zoom: f32,
    shadow_offset: f32,
) {
    if intensity <= 0.01 {
        return;
    }
    let teal = scale_color([0, 32, 43, 0], 120.0 / 255.0);
    let elements = [
        // `:473-481`: sharp point ×3.
        LensElement { tex: 1, repeat_times: 3, scale_start: 0.3, scale_over_index: -0.04, distance_start: 0.33, distance_along_index: 0.05, color: teal, intensity_over_index: -0.125, rotation: 0.0 },
        // `:483-491`: sharp point ×3.
        LensElement { tex: 1, repeat_times: 3, scale_start: 0.3, scale_over_index: 0.04, distance_start: 0.03, distance_along_index: 0.05, color: teal, intensity_over_index: -0.125, rotation: 0.0 },
        // `:493-498`: blurry point ×1, violet.
        LensElement { tex: 0, repeat_times: 1, scale_start: 0.3, scale_over_index: 0.0, distance_start: 0.41, distance_along_index: 0.0, color: scale_color([65, 0, 255, 0], 30.0 / 255.0), intensity_over_index: 0.0, rotation: 0.0 },
        // `:500-506`: bokeh ×1, rotated 0.01.
        LensElement { tex: 2, repeat_times: 1, scale_start: 0.3, scale_over_index: 0.0, distance_start: 0.525, distance_along_index: 0.0, color: scale_color([255, 255, 255, 0], 40.0 / 255.0), intensity_over_index: 0.0, rotation: 0.01 },
        // `:508-515`: bokeh ×6.
        LensElement { tex: 2, repeat_times: 6, scale_start: 0.24, scale_over_index: -0.04, distance_start: 0.225, distance_along_index: 0.04, color: scale_color([255, 255, 255, 0], 20.0 / 255.0), intensity_over_index: 0.0, rotation: 0.0 },
        // `:517-522`: blurry point ×1, blue.
        LensElement { tex: 0, repeat_times: 1, scale_start: 1.0, scale_over_index: 0.0, distance_start: 0.6, distance_along_index: 0.0, color: scale_color([0, 157, 255, 0], 40.0 / 255.0), intensity_over_index: 0.0, rotation: 0.0 },
        // `:524-530`: spectra ×1, rotated π.
        LensElement { tex: 3, repeat_times: 1, scale_start: 0.38, scale_over_index: 0.0, distance_start: 0.65, distance_along_index: 0.0, color: scale_color([255, 255, 255, 0], 10.0 / 255.0), intensity_over_index: 0.0, rotation: std::f32::consts::PI },
    ];
    for e in &elements {
        draw_lens_element(r, lookups, e, sun, center, intensity, screen_position_y, zoom, shadow_offset);
    }
}

/// `NextHorizonRenderer.AdjustIntensity` (`NextHorizonRenderer.cs:388-399`):
/// the temporal × celestial intensity, cubed, with the sun-scorch override.
/// `sunScorchCounter` has no port (0), so the override never fires.
fn adjust_flare_intensity(temporal: f32, celestial: f32, sun_scorch_counter: i32) -> f32 {
    let mut num = temporal * celestial;
    num *= num * num;
    if sun_scorch_counter > 0 {
        let num2 = 1.0 - tr_core::scene::get_lerp_value(0.0, 300.0, sun_scorch_counter as f32, true);
        num = (1.0 - num2 * num2) * celestial * 5.0;
    }
    num
}

/// `NextHorizonRenderer.GetVisibilities` (`NextHorizonRenderer.cs:196-234`):
/// `(sunset, sunrise, celestial)`. `SmoothedMushroomLightInfluence` has no
/// port (0); the eclipse zeroes both temporal visibilities.
fn flare_visibilities(
    time: f32,
    day_time: bool,
    atmo: f32,
    cloud_alpha: f32,
    eclipse: bool,
) -> (f32, f32, f32) {
    let celestial = if day_time {
        remap(time, 0.0, 3600.0, 0.0, 1.0) * remap(time, 52200.0, 54000.0, 1.0, 0.0)
    } else {
        remap(time, 0.0, 3600.0, 0.0, 1.0) * remap(time, 30600.0, 32400.0, 1.0, 0.0)
    };
    let num2 = 1.0 - cloud_alpha;
    let num = atmo * (num2 * num2);
    let mut sunrise = num;
    let mut sunset = num;
    if day_time {
        sunrise *= remap(time, 0.0, 2700.0, 0.0, 1.0) * remap(time, 3600.0, 10800.0, 1.0, 0.0);
        let num7 = remap(time, 43200.0, 50400.0, 0.0, 1.0);
        let num8 = remap(time, 50400.0, 54000.0, 1.0, 0.0);
        sunset *= num7 * num8 * num8;
        if eclipse {
            sunset = 0.0;
            sunrise = 0.0;
        }
    } else {
        sunrise = 0.0;
        sunset = 0.0;
    }
    (sunset, sunrise, celestial)
}

/// `Main.DrawLensFlare` (`Main.cs:62509-62514`) →
/// `NextHorizonRenderer.DrawLensFlare` (`NextHorizonRenderer.cs:363-386`): the
/// sunrise/sunset lens-flare chains, drawn under the background zoom
/// (`PlayerInput.SetZoom_Background`). The `ShouldDrawSurfaceBackground` gate
/// (remix/drunk seeds, fullscreen map) and `HorizonHelper.SunVisibilityEnabled`
/// (the sun-occlusion pixel target) are passed in by the caller; the occlusion
/// mask itself has no port, so the flare draws at full temporal intensity.
/// W4-d70 DOC-SWEEP: `player` feeds the advanced-shadow offset
/// (`LensFlareElement.cs:34-37`) and the sun-scorch override
/// (`AdjustIntensity` `:392-397`); `None` (TR_NOPLAYER) keeps both at 0.
#[allow(clippy::too_many_arguments)]
pub(crate) fn draw_lens_flare(
    r: &mut SpriteRenderer,
    lookups: &Lookups,
    world: &tr_core::World,
    player: Option<&Player>,
    camera: Vec2,
    bg_zoom: f32,
    screen: Vec2,
    atmo: f32,
    cloud_alpha: f32,
    should_draw_surface_bg: bool,
    sun_visibility_enabled: bool,
) {
    if !should_draw_surface_bg || !sun_visibility_enabled {
        return;
    }
    let (sunset_vis, sunrise_vis, celestial_vis) = flare_visibilities(
        world.time as f32,
        world.day_time,
        atmo,
        cloud_alpha,
        world.eclipse,
    );
    let sun_scorch = player.map_or(0, |p| p.sun_scorch_counter);
    let sunrise_intensity = adjust_flare_intensity(sunrise_vis, celestial_vis, sun_scorch);
    let sunset_intensity = adjust_flare_intensity(sunset_vis, celestial_vis, sun_scorch);
    // `:373`: both below 0.01 → nothing.
    if sunrise_intensity <= 0.01 && sunset_intensity <= 0.01 {
        return;
    }
    // `GetCelestialBodyPosition` (`:358-361`): `LastCelestialBodyPosition *
    // ScreenSize` — the sun/moon centre the sky pass drew, in background-zoom
    // screen pixels. `sunModY`/`moonModY` are 0 outside the title screen.
    let view_top = camera.y - screen.y * 0.5 / bg_zoom;
    let bg_top_y = tr_core::sky::sky_bg_top_y(view_top, world.world_surface);
    let (period, tex_w) = if world.day_time {
        (tr_core::sky::DAY_LENGTH, 114.0)
    } else {
        (tr_core::sky::NIGHT_LENGTH, 50.0)
    };
    let body = tr_core::sky::celestial_body(world.time, period, bg_top_y, tex_w, screen.x);
    let sun = [body.x, body.y];
    let center = [screen.x * 0.5, screen.y * 0.5];
    // `:46`: the screen-Y drift uses `Main.screenPosition.Y` (world px).
    let screen_position_y = view_top;
    // `LensFlareElement.cs:34-37`: the advanced-shadow offset (0 without a
    // local player — the C# reads `Main.LocalPlayer`, always present in game).
    let shadow_offset = player.map_or(0.0, |p| lens_shadow_offset(p, sun, center));
    draw_sunrise_flare(r, lookups, sun, center, sunrise_intensity, screen_position_y, bg_zoom, shadow_offset);
    draw_sunset_flare(r, lookups, sun, center, sunset_intensity, screen_position_y, bg_zoom, shadow_offset);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `:55-57` widens the player's 20x42 box to the sheet's width and shifts
    /// it up by `num2`. The vertical inflate adds `num2/2` per side, so the
    /// rect ends up `num2` taller than the sheet — that is the C#'s `Inflate`.
    #[test]
    fn the_obstruction_rect_is_sheet_wide_and_shifted_up_by_10() {
        let (px, py, pw, ph) = (100.0f32, 200.0f32, 20.0f32, 42.0f32);
        let (num, num2) = (200.0f32, 10.0f32);
        let inflate_x = (num - pw) / 2.0;
        let inflate_y = (num - ph) / 2.0 + num2 / 2.0;
        let (x, y) = (px - inflate_x, py - inflate_y - num2);
        assert_eq!(x, 10.0, "centred on the player");
        assert_eq!(pw + inflate_x * 2.0, num);
        assert_eq!(ph + inflate_y * 2.0, num + num2);
        assert_eq!(y, 106.0, "200 - 84 - 10");
    }
}

/// W4-d70: the five screen-effect draw rows' pure math — the inferno ring
/// animation, the paladin-shield distance gate, the sparkle fade and the lens
/// flare visibilities/intensity. Written, not run (code-only lane).
#[cfg(test)]
mod w4_d70_screen_effect_tests {
    use super::*;

    /// `Main.cs:64290-64300`: the scale advances 0.004/draw and resets to 0.8
    /// once it reaches 1; below 1 it is the scale itself.
    #[test]
    fn the_inferno_scale_breathes_and_resets_at_one() {
        let mut scale = 0.999f32;
        let mut rot = 0.0f32;
        let (num, _) = inferno_ring_step(&mut scale, &mut rot, true);
        // 0.999 + 0.004 = 1.003 >= 1 → reset to 0.8.
        assert!((scale - 0.8).abs() < 1e-6);
        assert!((num - 0.8).abs() < 1e-6);
        // Below 1 the scale is itself, advanced by 0.004.
        let mut scale = 0.5f32;
        let (num, _) = inferno_ring_step(&mut scale, &mut rot, true);
        assert!((num - 0.504).abs() < 1e-6);
    }

    /// `Main.cs:64288/64301`: the counters only advance while `allow_emit`
    /// (`FocusHelper.AllowPlayerToEmitEffects`); a blocked frame holds them.
    #[test]
    fn the_inferno_counters_freeze_when_emit_is_blocked() {
        let mut scale = 0.5f32;
        let mut rot = 1.0f32;
        let (num, r) = inferno_ring_step(&mut scale, &mut rot, false);
        assert!((scale - 0.5).abs() < 1e-6, "scale held");
        assert!((rot - 1.0).abs() < 1e-6, "rot held");
        assert!((num - 0.5).abs() < 1e-6);
        assert!((r - 1.0).abs() < 1e-6);
    }

    /// `Main.cs:64305-64312`: the rotation wraps at ±2π.
    #[test]
    fn the_inferno_rotation_wraps_at_tau() {
        let tau = std::f32::consts::TAU;
        let mut scale = 0.5f32;
        let mut rot = tau - 0.01;
        let (_, r) = inferno_ring_step(&mut scale, &mut rot, true);
        // tau - 0.01 + 0.05 = tau + 0.04 → wraps to 0.04.
        assert!((r - 0.04).abs() < 1e-4, "{r}");
        let mut rot = -tau + 0.01;
        let (_, r) = inferno_ring_step(&mut scale, &mut rot, true);
        // -tau + 0.01 + 0.05 = -tau + 0.06, still > -tau → no wrap.
        assert!((r - (-tau + 0.06)).abs() < 1e-4, "{r}");
    }

    /// `Main.cs:64315-64320`: ring j's scale wraps down by 0.2 above 1, and the
    /// brightness peaks at num4 = 0.9 (`Lerp(0.8, 0, |num4-0.9|*10)`).
    #[test]
    fn the_inferno_ring_terms_wrap_and_peak_at_nine_tenths() {
        // num = 0.95: j0 = 0.95, j1 = 1.05 → 0.85, j2 = 1.15 → 0.95.
        let (n0, _) = inferno_ring_terms(0.95, 0);
        let (n1, _) = inferno_ring_terms(0.95, 1);
        let (n2, _) = inferno_ring_terms(0.95, 2);
        assert!((n0 - 0.95).abs() < 1e-6);
        assert!((n1 - 0.85).abs() < 1e-6);
        assert!((n2 - 0.95).abs() < 1e-6);
        // Brightness: at num4 = 0.9 → 0.8; at 0.82 → |−0.08|*10 = 0.8 → 0.16.
        let (_, b_peak) = inferno_ring_terms(0.9, 0);
        assert!((b_peak - 0.8).abs() < 1e-6);
        let (_, b_off) = inferno_ring_terms(0.82, 0);
        assert!((b_off - 0.16).abs() < 1e-5, "{b_off}");
    }

    /// `Main.cs:57236-57242`: the boundary gate —
    /// `Remap(|dist - range|, range*0.15, 0, 0, 1)`. At the range itself the
    /// fade is 1; at 15% of the range away it is 0 (the boundary vanishes).
    #[test]
    fn the_paladin_boundary_fades_with_distance_from_the_range() {
        let range = PALADINS_SHIELD_RANGE;
        // dist == range → fromValue 0 → Remap(0, 120, 0, 0, 1) = 1.
        let at = remap(0.0, range * 0.15, 0.0, 0.0, 1.0);
        assert!((at - 1.0).abs() < 1e-6);
        // dist = range ± 120 → fromValue 120 → 0.
        let edge = remap(range * 0.15, range * 0.15, 0.0, 0.0, 1.0);
        assert_eq!(edge, 0.0);
        // Halfway (60 px off) → 0.5.
        let half = remap(range * 0.075, range * 0.15, 0.0, 0.0, 1.0);
        assert!((half - 0.5).abs() < 1e-6);
    }

    /// `Player.cs:38188-38195`: the defend gate — alive, shield, matching
    /// non-zero team, above 25% life (strict >).
    #[test]
    fn the_paladin_peer_defend_gate_matches_can_defend() {
        let peer = PaladinPeer {
            mounted_center: [0.0, 0.0],
            team: 1,
            has_paladin_shield: true,
            dead: false,
            hp: 100,
            max_hp: 400,
        };
        assert!(peer.can_defend(1), "matching team, full life");
        assert!(!peer.can_defend(2), "team mismatch");
        assert!(!peer.can_defend(0), "team 0 never defends");
        let mut low = peer;
        low.hp = 100;
        low.max_hp = 400;
        assert!(!low.can_defend(1), "exactly 25% is out (strict >)");
        low.hp = 101;
        assert!(low.can_defend(1), "just above 25% defends");
        let mut dead = peer;
        dead.dead = true;
        assert!(!dead.can_defend(1));
        let mut no_shield = peer;
        no_shield.has_paladin_shield = false;
        assert!(!no_shield.can_defend(1));
    }

    /// `Main.cs:35304`: the sparkle fade —
    /// `GetLerpValue(0,1,t,clamped) * GetLerpValue(2,1,t,clamped)`. At t=0 the
    /// fade-in is 0; at t=1 both are 1; at t=2 the fade-out is 0.
    #[test]
    fn the_sparkle_fade_window_peaks_at_one() {
        let fade = |t: f32| {
            tr_core::scene::get_lerp_value(0.0, 1.0, t, true)
                * tr_core::scene::get_lerp_value(2.0, 1.0, t, true)
        };
        assert_eq!(fade(0.0), 0.0);
        assert!((fade(1.0) - 1.0).abs() < 1e-6);
        assert_eq!(fade(2.0), 0.0);
        assert!((fade(0.5) - 0.5).abs() < 1e-6, "fade-in half, fade-out full");
    }

    /// `NextHorizonRenderer.cs:196-234`: at night both temporal visibilities
    /// are 0; an eclipse zeroes the day ones too.
    #[test]
    fn the_flare_visibilities_are_zero_at_night_and_in_eclipse() {
        let (sunset, sunrise, _) = flare_visibilities(16000.0, false, 1.0, 0.0, false);
        assert_eq!(sunset, 0.0);
        assert_eq!(sunrise, 0.0);
        let (sunset, sunrise, _) = flare_visibilities(5000.0, true, 1.0, 0.0, true);
        assert_eq!(sunset, 0.0);
        assert_eq!(sunrise, 0.0);
    }

    /// `NextHorizonRenderer.cs:215`: the sunrise visibility ramps in over
    /// 0..2700 and out over 3600..10800. At t=1350 (half the rise) with full
    /// atmo and no cloud it is 0.5; past 10800 it is 0.
    #[test]
    fn the_sunrise_visibility_ramps_in_and_out() {
        let (_, sunrise, _) = flare_visibilities(1350.0, true, 1.0, 0.0, false);
        assert!((sunrise - 0.5).abs() < 1e-4, "{sunrise}");
        let (_, sunrise, _) = flare_visibilities(12000.0, true, 1.0, 0.0, false);
        assert_eq!(sunrise, 0.0);
    }

    /// `NextHorizonRenderer.cs:216-218`: the sunset visibility ramps in over
    /// 43200..50400 and out (squared) over 50400..54000. At t=52200 (half the
    /// fall) it is `1 * 0.5 * 0.5 = 0.25`.
    #[test]
    fn the_sunset_visibility_falls_squared() {
        let (sunset, _, _) = flare_visibilities(52200.0, true, 1.0, 0.0, false);
        assert!((sunset - 0.25).abs() < 1e-4, "{sunset}");
        let (sunset, _, _) = flare_visibilities(46800.0, true, 1.0, 0.0, false);
        // Half the rise (0.5) × full fall (1) = 0.5.
        assert!((sunset - 0.5).abs() < 1e-4, "{sunset}");
    }

    /// `NextHorizonRenderer.cs:388-399`: the intensity is `(temporal *
    /// celestial)^3`; the sun-scorch override (counter > 0) replaces it.
    #[test]
    fn the_flare_intensity_cubes_the_product() {
        let i = adjust_flare_intensity(0.5, 1.0, 0);
        assert!((i - 0.125).abs() < 1e-6, "{i}");
        // Sun scorch 150: num2 = 1 - lerp(0,300,150) = 0.5;
        // (1 - 0.25) * 1 * 5 = 3.75.
        let i = adjust_flare_intensity(0.5, 1.0, 150);
        assert!((i - 3.75).abs() < 1e-5, "{i}");
    }

    /// `NextHorizonRenderer.cs:311-321`: the celestial power ramps in over
    /// 0..3600 and out over 52200..54000 (day). At noon (27000) it is 1.
    #[test]
    fn the_celestial_power_is_full_at_midday() {
        let (_, _, celestial) = flare_visibilities(27000.0, true, 1.0, 0.0, false);
        assert!((celestial - 1.0).abs() < 1e-6, "{celestial}");
        let (_, _, celestial) = flare_visibilities(0.0, true, 1.0, 0.0, false);
        assert_eq!(celestial, 0.0);
    }

    /// `Main.cs:57250/57272`: the inside-range flag picks the brighter arc
    /// (0.5 vs 0.25) and the larger item glint (1.5 vs 1.0).
    #[test]
    fn the_paladin_inside_flag_picks_the_brighter_terms() {
        let range = PALADINS_SHIELD_RANGE;
        let inside = 700.0 < range;
        let outside = 900.0 < range;
        assert!(inside);
        assert!(!outside);
        assert_eq!(if inside { 0.5 } else { 0.25 }, 0.5);
        assert_eq!(if outside { 0.5 } else { 0.25 }, 0.25);
    }

    /// W4-d70 DOC-SWEEP — `LensFlareElement.cs:34-37`: the shadow offset is
    /// the dot of the trail's unit vector with the sun direction, times the
    /// trail length. A player moving toward the sun gets the full length;
    /// perpendicular motion gets 0; a still player gets 0.
    #[test]
    fn the_lens_shadow_offset_projects_the_trail_onto_the_sun_axis() {
        let w = tr_core::World::new("t".into(), 64, 800);
        let mut p = Player::spawn(&w);
        // The sun sits to the right of the screen centre: dir = (1, 0).
        let sun = [900.0, 360.0];
        let center = [640.0, 360.0];
        // A still player: the trail is all one point → offset 0.
        for _ in 0..10 {
            p.update_advanced_shadows();
        }
        assert_eq!(lens_shadow_offset(&p, sun, center), 0.0);
        // Move +x 5 px/tick for 5 ticks: the trail spans 20 px (index 0 vs
        // index 4), aligned with the sun → offset = +20.
        for _ in 0..5 {
            p.x += 5.0;
            p.update_advanced_shadows();
        }
        let off = lens_shadow_offset(&p, sun, center);
        assert!((off - 20.0).abs() < 1e-4, "{off}");
        // Perpendicular motion (+y): the dot with (1,0) is 0.
        let mut q = Player::spawn(&w);
        for _ in 0..10 {
            q.update_advanced_shadows();
        }
        for _ in 0..5 {
            q.y += 5.0;
            q.update_advanced_shadows();
        }
        assert!(lens_shadow_offset(&q, sun, center).abs() < 1e-4);
    }

    /// W4-d70 DOC-SWEEP — `Player.cs:4328-4345`: the ring caps at 60, the
    /// index clamps at the count, and the reset clears only the count.
    #[test]
    fn the_advanced_shadow_ring_caps_and_resets() {
        let w = tr_core::World::new("t".into(), 64, 800);
        let mut p = Player::spawn(&w);
        assert_eq!(p.available_advanced_shadows_count, 0);
        // One push: index 0 is the current position.
        p.x = 100.0;
        p.update_advanced_shadows();
        assert_eq!(p.available_advanced_shadows_count, 1);
        assert_eq!(p.get_advanced_shadow(0), [100.0, p.y]);
        // 100 pushes: the count caps at 60.
        for i in 0..100 {
            p.x = i as f32;
            p.update_advanced_shadows();
        }
        assert_eq!(p.available_advanced_shadows_count, 60);
        // The freshest slot is the last push (x = 99).
        assert_eq!(p.get_advanced_shadow(0)[0], 99.0);
        // Index 4 walks back 4 ticks (x = 95).
        assert_eq!(p.get_advanced_shadow(4)[0], 95.0);
        // The reset clears the count; index 0 then clamps to the stale head.
        p.reset_advanced_shadows();
        assert_eq!(p.available_advanced_shadows_count, 0);
    }
}
