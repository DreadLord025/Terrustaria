//! The full-screen scene effects — `ScreenObstruction`, `ScreenDarkness` and
//! `MoonlordDeathDrama` (`decompiled/Terraria/GameContent/Events/`).
//!
//! The update halves live in `tr_core::scene::SceneState` (pure state
//! machines); these are the draw halves. Sources:
//! `ScreenObstruction.cs:48-68`, `ScreenDarkness.cs:36-50`,
//! `MoonlordDeathDrama.cs:195-225`, plus the `Main.Draw` order at
//! `Main.cs:62217` (`DrawBack`, right after the scene overlays) and
//! `Main.cs:62315-62324` (`DrawFront`, `DrawWhite`, `Draw` — after the
//! projectiles and the wires).
//!
//! Deviations (recorded in `docs/ANALYSIS.md`): `TextureAssets.MagicPixel` is
//! the renderer's built-in `white_uv` quad; the `MoonlordPiece` light average
//! reads the port's `LightMap` rather than `Lighting.GetColor`;
//! `DrawPaladinsShield`/`DrawInfernoRings` are gated on `Player` fields the
//! port does not carry (`hasPaladinShield`, `defendedByPaladin`, `inferno`,
//! `flameRingScale`/`flameRingRot`), so their draws stay unwritten.

use glam::Vec2;

use tr_core::lighting::LightMap;
use tr_core::scene::SceneState;
use tr_core::TILE_PX;
use tr_render::sprite::SpriteRenderer;

use crate::assets::{self, Lookups};

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
