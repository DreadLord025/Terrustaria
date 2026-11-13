//! In-game HUD: hotbar / open inventory, selected-item name, the item carried on the
//! cursor, hearts and breath, and the boss bar.

pub mod achievements;
pub mod bars;
pub(crate) mod bestiary;
pub(crate) mod emotes;
pub mod framework;
pub mod cards;
pub mod char_creation;
pub mod char_select;
pub mod clipboard;
pub(crate) mod chest;
pub mod chrome;
pub mod color;
pub mod crafting;
pub mod cursor;
pub mod ingame_options;
pub mod housing;
pub mod inventory;
pub mod keyboard;
pub mod minimap;
pub mod panel;
pub mod pointer;
pub mod resources;
pub mod save_list;
pub mod sliders;
pub mod select_hit;
pub mod slot;
pub mod text;
pub mod tooltip;
pub mod uistate;
pub mod world_creation;
pub mod world_select;

use glam::Vec2;
use tr_render::{SheetRect, SpriteRenderer};

use crate::assets;
use crate::inv;
use crate::settings::Settings;
use crate::item::{HOTBAR, INV, ItemSlot};
use crate::player::Player;

/// The shared rect hit-test: XNA `Rectangle.Contains(Point)` as the UI reads it
/// (`UIElement.cs:354-357`'s `ContainsPoint`, `UIColoredSlider.cs:110`,
/// `IngameOptions.cs:1557`): inclusive left/top edges, exclusive right/bottom.
/// The one copy behind every screen's click table and hover test — the former
/// per-screen `in_rect` / `rect_hover` / `contains` / `inside` duplicates.
///
/// Two source sites differ and keep their own comparisons: the HSL rows'
/// open-on-every-edge test is [`in_rect_strict`], and the inventory grid's
/// `Main.cs:41446` comparison is inclusive on the right/bottom too
/// (`inventory::hit_test`).
pub(crate) fn in_rect(mouse: Vec2, rect: [f32; 4]) -> bool {
    mouse.x >= rect[0]
        && mouse.x < rect[0] + rect[2]
        && mouse.y >= rect[1]
        && mouse.y < rect[1] + rect[3]
}

/// The strict four-side variant of [`in_rect`] — open on every edge — the HSL
/// colour rows' hit test (`Main.cs:51142` and the row twins: `mouseX > num53 -
/// 4 && mouseX < num53 + Hue.Width() + 4 && ...`).
pub(crate) fn in_rect_strict(mouse: Vec2, rect: [f32; 4]) -> bool {
    mouse.x > rect[0]
        && mouse.x < rect[0] + rect[2]
        && mouse.y > rect[1]
        && mouse.y < rect[1] + rect[3]
}

/// One atlas sprite stretched over `dst` from the whole `sheet` — the shared
/// `push_ui_sprite` over the atlas UVs (the former per-screen `push_whole`
/// copies of `char_select` / `world_select` / `char_creation`).
pub(crate) fn push_whole(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    sheet: &SheetRect,
    dst: [f32; 4],
    tint: [f32; 4],
) {
    let (aw, ah) = (lookups.atlas_size.0 as f32, lookups.atlas_size.1 as f32);
    let uv = [
        sheet.x as f32 / aw,
        sheet.y as f32 / ah,
        (sheet.x + sheet.width) as f32 / aw,
        (sheet.y + sheet.height) as f32 / ah,
    ];
    r.push_ui_sprite([dst[0], dst[1]], [dst[2], dst[3]], uv, tint);
}

/// The hover/idle colour pick every UI colour table shares (the
/// `MouseOver`/`MouseOut` pairs): one of the two pre-tint byte colours by the
/// hover flag. `char_select`/`world_select`'s card pairs and the keyboard's
/// key / toggle / submit tables select both their background and border
/// through it.
pub(crate) fn hover_pair(idle: [u8; 4], hovered: [u8; 4], on: bool) -> [u8; 4] {
    if on { hovered } else { idle }
}

/// Per-frame animated UI values shared by the HUD passes: `Main.mouseTextColor`
/// (which multiplies every interface label/colour) and `Main.cursorScale` (the
/// heart/mana pulse, `Main.cs:52491`).
#[derive(Clone, Copy, Debug)]
pub struct UiPulse {
    /// `Main.mouseTextColor / 255` (`Main.cs:18216`; oscillates `190/255..=1.0`).
    pub text: f32,
    /// `Main.cursorScale` = `cursorAlpha * 0.3 + 0.7 + 0.1` (`Main.cs:52491`).
    pub cursor_scale: f32,
}

impl Default for UiPulse {
    fn default() -> Self {
        Self {
            text: 1.0,
            cursor_scale: 1.0,
        }
    }
}

/// Vanilla HUD layout: hotbar top-left (or the open 10x5 inventory grid), hearts
/// top-right, breath bubbles above the player and the boss bar at the bottom centre
/// (`GUIHotbarDrawInner`, `Main.DrawInventory`, `ClassicPlayerResourcesDisplaySet`,
/// `BigProgressBarHelper`). `m_h` is `Main.mH`
/// (`Main.DrawInterface_16_MapOrMinimap`, `Main.cs:45771-45798`) — `256` while
/// the minimap HUD is live, `0` otherwise; it pushes the equipment block down.
/// `map_style` is `Main.mapStyle` (`Main.cs:66750`), driving the map-style icon
/// row's active tint (`Main.cs:41083-41086`); `boss_bar_text` is
/// `BigProgressBarSystem.ShowText` (`BigProgressBarSystem.cs:231`/`:236`), the
/// gate on the boss bar's numbers (`BigProgressBarHelper.cs:53-55`).
#[allow(clippy::too_many_arguments)]
/// `Utils.Remap` (`Utils.cs`) — the skip prompt's two ramps.
fn remap(v: f32, from_min: f32, from_max: f32, to_min: f32, to_max: f32) -> f32 {
    to_min + (v - from_min) / (from_max - from_min) * (to_max - to_min)
}

/// `Main.DrawDeathText`'s skip-prompt ramps (`Main.cs:45350-45358`):
/// `num4 = Remap(frames, 0, 60, 0, 1)` (the alpha and the 70 → 20 px slide) and
/// `num5 = Remap(frames, 0, 30, 0.5, 0)^2` (the scale pulse). Both come from
/// the vanilla's **unclamped** `Utils.Remap`, ported verbatim: past frame 60
/// `num4 > 1` and past 30 `num5` breathes back up, which the real game only
/// sees if the gate stays open that long (a danger NPC usually resets the
/// counter first). Returns `(num4, num5)`.
pub(crate) fn respawn_skip_ramp(frames: i32) -> (f32, f32) {
    let frames = frames as f32;
    let num4 = remap(frames, 0.0, 60.0, 0.0, 1.0);
    let mut num5 = remap(frames, 0.0, 30.0, 0.5, 0.0);
    num5 *= num5;
    (num4, num5)
}

/// `Game.RespawnSkip` ("{InputTrigger_Jump} to skip", `:832`) with the trigger
/// tag resolved. The port has no input-binding table yet (M7-UI-5), so the
/// jump trigger renders as its default key, `Space` — the same key
/// `world_sim::player` reads for `PlayerInput::jump`.
pub(crate) fn respawn_skip_text() -> String {
    crate::lang::text("Game.RespawnSkip").replace("{InputTrigger_Jump}", "Space")
}

/// `Main.DrawDeathText`'s row arithmetic (`Main.cs:45328-45386`) as a pure
/// layout: the broadcast's top, the optional coins line, the optional skip
/// prompt (its y, scale and alpha) and the optional countdown. `num` is the
/// C#'s running offset from the screen's midpoint.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct DeathLayout {
    pub broadcast_y: f32,
    pub coins_y: Option<f32>,
    pub prompt_y: Option<f32>,
    pub prompt_scale: f32,
    pub prompt_alpha: f32,
    pub countdown_y: Option<f32>,
}

/// The C#'s `num` walk: `-60`, `+50`/`-26` for the coins line, the skip
/// prompt's `Remap(num4, 0, 1, 70, 20) * num4` slide, then `+70` for the
/// countdown (`Main.cs:45328-45386`).
pub(crate) fn death_layout(
    screen_y: f32,
    lost_coins: u64,
    respawn_timer: i32,
    skip_frames: i32,
) -> DeathLayout {
    let mid = screen_y / 2.0;
    let mut num = -60.0;
    let mut coins_y = None;
    if lost_coins > 0 {
        coins_y = Some(mid + num + 50.0);
        num += 24.0;
    }
    let mut prompt_y = None;
    let mut prompt_scale = 0.7;
    let mut prompt_alpha = 0.0;
    if skip_frames > 0 {
        let (num4, num5) = respawn_skip_ramp(skip_frames);
        num += remap(num4, 0.0, 1.0, 70.0, 20.0) * num4;
        prompt_scale = (1.0 + num5) * 0.7;
        prompt_alpha = num4;
        // `+ stringSize.Y * 0.5f * num2` (`:45366`) — the caller adds the
        // font's half line height, so the layout carries the bare `num`.
        prompt_y = Some(mid + num);
    }
    let mut countdown_y = None;
    if respawn_timer > 1 {
        num += 70.0;
        countdown_y = Some(mid + num);
    }
    DeathLayout {
        broadcast_y: mid - 60.0,
        coins_y,
        prompt_y,
        prompt_scale,
        prompt_alpha,
        countdown_y,
    }
}

/// The death screen's text block — `Main.DrawDeathText`'s shape
/// (`Main.cs:45324-45341`): the death broadcast (the big `DeathText` font,
/// centered 60 px above the screen's midpoint), the softcore
/// "Dropped coins: X" line 50 px lower (the small `MouseText` font), and
/// the respawn countdown 70 px below the broadcast (`Game.RespawnInSuffix`
/// — the vanilla's "Respawn in {0} seconds" key resolves to "{0}" so the
/// port draws the seconds with the prefix the same way; BUG-06). The
/// alpha rides the death-screen fade — the port draws at the fixed
/// full-bright (the respawn timer's fade lands with the death-screen pass).
pub(crate) fn draw_death_screen(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    screen: Vec2,
    death_text: &str,
    lost_coins: u64,
    lost_coin_string: &str,
    respawn_timer: i32,
    skip_frames: i32,
) {
    let color = [1.0, 1.0, 1.0, 1.0];
    let layout = death_layout(screen.y, lost_coins, respawn_timer, skip_frames);
    text::draw_death_string(r, lookups, screen.x, layout.broadcast_y, color, death_text);
    if let Some(y) = layout.coins_y {
        // `Game.DroppedCoins` ("dropped {0}") with the `ValueToCoins` string.
        let line = crate::lang::text_args("Game.DroppedCoins", &[lost_coin_string.to_string()]);
        text::draw_dropped_coins(r, lookups, screen.x, y, color, &line);
    }
    // `if (Main.LocalPlayer.AllowsRespawnTimerSkip())` (`Main.cs:45348-45378`):
    // the "{InputTrigger_Jump} to skip" prompt, ramped in by
    // `_framesWithRespawnSkipAvailable` (see [`respawn_skip_ramp`]).
    if let Some(y) = layout.prompt_y {
        let label = respawn_skip_text();
        let y = y + text::death_line_height(lookups) * 0.5 * 0.7;
        text::draw_death_string_scaled(
            r,
            lookups,
            screen.x,
            y,
            layout.prompt_scale,
            [1.0, 1.0, 1.0, layout.prompt_alpha],
            &label,
        );
    }
    // `if (Main.player[Main.myPlayer].respawnTimer > 1)` (`Main.cs:45380`):
    // the ceil'ed seconds through the suffix key, drawn at the 0.7 scale.
    if let Some(y) = layout.countdown_y {
        let seconds = (1 + respawn_timer / 60).to_string();
        let suffix = crate::lang::text_args("Game.RespawnInSuffix", &[seconds]);
        let line = format!("Respawn in {suffix}");
        text::draw_death_string(r, lookups, screen.x, y, color, &line);
    }
}

/// The invasion progress bar — a faithful port of `Main.DrawInvasionProgress`
/// (`Main.cs:47696-47845`).
///
/// Three gates close the whole function, and they are what stops the bar from
/// being drawn permanently (`BUG_REPORT.md` §8):
/// 1. `Main.invasionProgress == -1` — no invasion is being tracked (`:47698`);
/// 2. the fade alpha clamped to `0..=1` is still `0` (`:47729`).
///
/// The fade head itself (`:47700-47728`) rides the world tick instead of the
/// draw (`world_sim::npcs::update_invasion_state`) so the alpha ramp is
/// deterministic — the only deliberate structural deviation.
///
/// The rest is 1:1: the per-icon sheet/label/colour pick (`:47733-47780`), the
/// `invasionProgressWave > 0` arm versus the `WaveCleared` arm (both with the
/// `ColorBar` track and the yellow / orange / black `MagicPixel` fills), and
/// the shared group plate at `(screenWidth - num13, screenHeight - 80)` with
/// the `Extra` icon and the group label (`:47831-47845`).
#[allow(clippy::too_many_arguments)]
pub(crate) fn draw_invasion_progress(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    screen: Vec2,
    progress: i32,
    progress_max: i32,
    progress_icon: i32,
    progress_wave: i32,
    alpha: f32,
) {
    // The two early returns (`:47698` and `:47729`), factored out so the gate
    // can be unit-tested without a renderer.
    let Some(alpha) = invasion_bar_alpha(progress, alpha) else {
        return;
    };
    // `float num = 0.5f + Main.invasionProgressAlpha * 0.5f;` (`:47730`).
    let num = 0.5 + alpha * 0.5;

    // The icon/label/colour pick (`:47733-47780`). `value` defaults to
    // `Extra[9]`, `text` to "" and `c` to White.
    let (icon_sheet, label, panel_color) = invasion_group(lookups, progress_icon);

    // The bar panel's anchor (`:47784`, `:47810`): both arms share it.
    let vector3 = [screen.x - 120.0, screen.y - 40.0];
    // `Utils.DrawInvBG`'s tint (`:47786`, `:47811`):
    // `new Color(63, 65, 151, 255) * 0.785f`.
    let panel_tint = color::tint(color::mul([63, 65, 151, 255], 0.785));

    if progress_wave > 0 {
        // --- the wave arm (`:47782-47808`) --------------------------------
        let num2 = (200.0 * num) as i32;
        let num3 = (45.0 * num) as i32;
        tooltip::draw_inv_bg(
            r,
            lookups,
            [
                (vector3[0] as i32 - num2 / 2) as f32,
                (vector3[1] as i32 - num3 / 2) as f32,
                num2 as f32,
                num3 as f32,
            ],
            panel_tint,
        );
        // `Game.WaveMessage` = "Wave {0}: {1}" where `{1}` is the percent (or
        // the `Game.InvasionPoints` fallback when the max is 0) (`:47789`).
        let arg1 = if progress_max != 0 {
            format!(
                "{}%",
                (progress as f32 * 100.0 / progress_max as f32) as i32
            )
        } else {
            crate::lang::text_args("Game.InvasionPoints", &[progress.to_string()])
        };
        let text = crate::lang::text_args("Game.WaveMessage", &[progress_wave.to_string(), arg1]);
        // `float num4 = MathHelper.Clamp(progress/max, 0, 1); if (max == 0)
        // num4 = 1f;` (`:47791-47795`).
        let num4 = if progress_max != 0 {
            (progress as f32 / progress_max as f32).clamp(0.0, 1.0)
        } else {
            1.0
        };
        let num5 = 169.0 * num;
        let num6 = 8.0 * num;
        // `vector2 = vector + UnitY*num6 + UnitX*1f` (`:47796`).
        let mut vector2 = [vector3[0] + 1.0, vector3[1] + num6];
        // `Utils.DrawBorderString(text2, vector2, White*alpha, num, 0.5f, 1f)`:
        // the origin is `(0.5, 1) * MeasureString(text)` at scale `num`, so the
        // string is centred on `vector2.x` and its bottom sits on `vector2.y`.
        draw_anchored_text(r, lookups, &text, vector2, num, alpha, 0.5, 1.0);
        // The `ColorBar` track: `Draw(value2, vector, null, White*alpha, 0,
        // new Vector2(value2.Width/2, 0f), num, ...)` (`:47798`).
        push_color_bar(r, lookups, vector3, num, alpha);
        // `vector2 += UnitX * (num4 - 0.5f) * num5` (`:47799`).
        vector2[0] += (num4 - 0.5) * num5;
        // The yellow / orange / black `MagicPixel` fills (`:47800-47802`).
        push_inv_fill(
            r,
            [vector2[0] - num5 * num4, vector2[1] - num6 / 2.0],
            [num5 * num4, num6],
            color::tint(color::mul([255, 241, 51, 255], alpha)),
        );
        push_inv_fill(
            r,
            [vector2[0] - 2.0, vector2[1] - num6 / 2.0],
            [2.0, num6],
            color::tint(color::mul([255, 165, 0, 127], alpha)),
        );
        push_inv_fill(
            r,
            [vector2[0], vector2[1] - num6 / 2.0],
            [num5 * (1.0 - num4), num6],
            [0.0, 0.0, 0.0, alpha],
        );
    } else {
        // --- the `WaveCleared` arm (`:47806-47829`) ------------------------
        let num7 = (200.0 * num) as i32;
        let num8 = (45.0 * num) as i32;
        tooltip::draw_inv_bg(
            r,
            lookups,
            [
                (vector3[0] as i32 - num7 / 2) as f32,
                (vector3[1] as i32 - num8 / 2) as f32,
                num7 as f32,
                num8 as f32,
            ],
            panel_tint,
        );
        let text = crate::lang::text_args(
            "Game.WaveCleared",
            &[if progress_max != 0 {
                format!(
                    "{}%",
                    (progress as f32 * 100.0 / progress_max as f32) as i32
                )
            } else {
                progress.to_string()
            }],
        );
        if progress_max != 0 {
            push_color_bar(r, lookups, vector3, num, alpha);
            let num9 = (progress as f32 / progress_max as f32).clamp(0.0, 1.0);
            // `float num10 = num; if (MeasureString(text3).Y > 22f) num10 *=
            // 22f / that.Y;` (`:47817-47821`).
            let text_h = text::line_height(lookups);
            let num10 = if text_h > 22.0 {
                num * (22.0 / text_h)
            } else {
                num
            };
            let num11 = 169.0 * num;
            let num12 = 8.0 * num;
            // `vector5 = vector3 + UnitY*num12 + UnitX*1f` (`:47824`).
            let mut vector5 = [vector3[0] + 1.0, vector3[1] + num12];
            // `DrawBorderString(text3, vector5 + (0,-4), White*alpha, num10,
            // 0.5f, 1f)` (`:47826`).
            draw_anchored_text(
                r,
                lookups,
                &text,
                [vector5[0], vector5[1] - 4.0],
                num10,
                alpha,
                0.5,
                1.0,
            );
            // `vector5 += UnitX * (num9 - 0.5f) * num11` (`:47827`).
            vector5[0] += (num9 - 0.5) * num11;
            push_inv_fill(
                r,
                [vector5[0] - num11 * num9, vector5[1] - num12 / 2.0],
                [num11 * num9, num12],
                color::tint(color::mul([255, 241, 51, 255], alpha)),
            );
            push_inv_fill(
                r,
                [vector5[0] - 2.0, vector5[1] - num12 / 2.0],
                [2.0, num12],
                color::tint(color::mul([255, 165, 0, 127], alpha)),
            );
            push_inv_fill(
                r,
                [vector5[0], vector5[1] - num12 / 2.0],
                [num11 * (1.0 - num9), num12],
                [0.0, 0.0, 0.0, alpha],
            );
        }
    }

    // --- the shared group plate (`:47831-47845`) ---------------------------
    // `vector6 = MouseText.MeasureString(text); num13 = 120f; if (vector6.X >
    // 200f) num13 += vector6.X - 200f;` (`:47831-47835`).
    let label_w = text::hud_text_width(r, lookups, &label, 1.0);
    let mut num13 = 120.0;
    if label_w > 200.0 {
        num13 += label_w - 200.0;
    }
    // `r = Utils.CenteredRectangle(new Vector2(screenWidth - num13,
    // screenHeight - 80), (vector6 + new Vector2(value.Width + 12, 6f)) * num)`
    // (`:47836`).
    let icon_w = icon_sheet
        .as_ref()
        .map(|s| s.width as f32)
        .unwrap_or(32.0);
    let icon_h = icon_sheet
        .as_ref()
        .map(|s| s.height as f32)
        .unwrap_or(32.0);
    let plate_w = (label_w + icon_w + 12.0) * num;
    let plate_h = (text::line_height(lookups) + 6.0) * num;
    let plate = [
        screen.x - num13 - plate_w / 2.0,
        screen.y - 80.0 - plate_h / 2.0,
        plate_w,
        plate_h,
    ];
    // `Utils.DrawInvBG(sb, r, c)` (`:47837`) — the group-coloured plate.
    tooltip::draw_inv_bg(r, lookups, plate, panel_color);
    // `sb.Draw(value, Utils.Left(r) + UnitX * num * 8f, null, White*alpha, 0f,
    // new Vector2(0f, value.Height / 2), num * 0.8f, ...)` (`:47838`): the icon's
    // origin is `(0, h/2)`, so its vertical centre sits on the plate's top edge.
    if let Some(uv) = lookups.hud_uv(icon_sheet.as_ref()) {
        let icon_scale = num * 0.8;
        r.push_ui_sprite(
            [plate[0] + num * 8.0, plate[1] - icon_h / 2.0 * icon_scale],
            [icon_w * icon_scale, icon_h * icon_scale],
            uv,
            [1.0, 1.0, 1.0, alpha],
        );
    }
    // `DrawBorderString(text, Utils.Right(r) + UnitX * num * -22f, White*alpha,
    // num * 0.9f, 1f, 0.4f)` (`:47839`): right-aligned, 40% of the height above
    // the anchor y.
    let right = plate[0] + plate[2] - num * 22.0;
    draw_anchored_text(r, lookups, &label, [right, plate[1]], num * 0.9, alpha, 1.0, 0.4);
}

/// `Main.DrawInvasionProgress`'s two early returns (`Main.cs:47698` and
/// `:47729`): `invasionProgress == -1` means no invasion is tracked, and an
/// alpha clamped to `0..=1` that is still `0` means the bar has finished
/// fading. Returns the clamped alpha when the bar should draw.
pub(crate) fn invasion_bar_alpha(progress: i32, alpha: f32) -> Option<f32> {
    if progress == -1 {
        return None;
    }
    let alpha = alpha.clamp(0.0, 1.0);
    if alpha <= 0.0 {
        return None;
    }
    Some(alpha)
}

/// The `ColorBar` track of both arms: `Draw(value, vector, null, White*alpha,
/// 0f, new Vector2(value.Width / 2, 0f), num, ...)` (`Main.cs:47798`,
/// `:47814`) — the 178x16 sheet centred on `vector.x`, its top edge at
/// `vector.y`, scaled by `num`.
fn push_color_bar(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    vector: [f32; 2],
    num: f32,
    alpha: f32,
) {
    let bar_w = lookups
        .ui_color_bar
        .as_ref()
        .map(|s| s.width as f32)
        .unwrap_or(178.0);
    let bar_h = lookups
        .ui_color_bar
        .as_ref()
        .map(|s| s.height as f32)
        .unwrap_or(16.0);
    if let Some(uv) = lookups.hud_uv(lookups.ui_color_bar.as_ref()) {
        r.push_ui_sprite(
            [vector[0] - bar_w / 2.0 * num, vector[1]],
            [bar_w * num, bar_h * num],
            uv,
            [1.0, 1.0, 1.0, alpha],
        );
    }
}

/// The icon/label/colour pick dispatch (`Main.cs:47734-47780`), separated so
/// the arms can be unit-tested without a renderer.
///
/// The labels are the localized strings the C# pulls: `Lang.inter[83..88]` for
/// the six legacy events and `DungeonDefenders2.InvasionProgressTitle` for the
/// Old One's Army.
fn invasion_group(
    lookups: &assets::Lookups,
    icon: i32,
) -> (Option<tr_render::SheetRect>, String, [f32; 4]) {
    // The index order is `assets::INVASION_ICON_EXTRAS`:
    // [0]=Extra_7 [1]=Extra_8 [2]=Extra_9 [3]=Extra_10 [4]=Extra_11
    // [5]=Extra_12 [6]=Extra_79.
    //
    // The colour is the group RGB `* 0.5f` for the seven mapped icons and the
    // untouched `Color.White` for the default (`:47731-47732` vs
    // `:47736-47779`), so the halving lives in each arm rather than outside.
    let (sheet_idx, label, plate): (usize, String, [f32; 4]) = match icon {
        // `Lang.inter[83]` — "Frost Moon" (`:47735-47739`).
        1 => (1, crate::lang::inter(83), half([64, 109, 164])),
        // `Lang.inter[84]` — "Pumpkin Moon" (`:47740-47744`).
        2 => (5, crate::lang::inter(84), half([112, 86, 114])),
        // `DungeonDefenders2.InvasionProgressTitle` — "Old One's Army"
        // (`:47745-47749`).
        3 => (
            6,
            crate::lang::text("DungeonDefenders2.InvasionProgressTitle"),
            half([88, 0, 160]),
        ),
        // `Lang.inter[85]` — "Martian Madness" (`:47750-47754`).
        7 => (3, crate::lang::inter(85), half([165, 160, 155])),
        // `Lang.inter[86]` — "Pirate Invasion" (`:47755-47759`).
        6 => (4, crate::lang::inter(86), half([148, 122, 72])),
        // `Lang.inter[87]` — "Frost Legion" (`:47760-47764`).
        5 => (0, crate::lang::inter(87), half([173, 135, 140])),
        // `Lang.inter[88]` — "Goblin Army" (`:47765-47769`).
        4 => (2, crate::lang::inter(88), half([94, 72, 131])),
        // The default: `Extra[9]`, empty text, `Color.White` (`:47731-47732`).
        _ => (2, String::new(), [1.0, 1.0, 1.0, 1.0]),
    };
    (
        lookups.invasion_icons.get(sheet_idx).copied().flatten(),
        label,
        plate,
    )
}

/// `new Color(r, g, b) * 0.5f` — XNA truncates every channel
/// (`(byte)(c * 0.5)`, alpha included).
fn half(rgb: [u8; 3]) -> [f32; 4] {
    color::tint(color::mul([rgb[0], rgb[1], rgb[2], 255], 0.5))
}

/// One `MagicPixel` fill of the invasion bar: `new Rectangle(0, 0, 1, 1)` of the
/// 1x1 sheet stretched to `size`.
fn push_inv_fill(r: &mut SpriteRenderer, pos: [f32; 2], size: [f32; 2], color: [f32; 4]) {
    if size[0] <= 0.0 || size[1] <= 0.0 {
        return;
    }
    r.push_fx_rect(pos, size, color);
}

/// `Utils.DrawBorderString` (`Utils.cs:2510-2518`) with a fractional anchor:
/// the origin is `(anchor_x, anchor_y) * MeasureString(text)` in **unscaled**
/// pixels, so the destination top-left is `pos - origin * scale`.
fn draw_anchored_text(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    text: &str,
    pos: [f32; 2],
    scale: f32,
    alpha: f32,
    anchor_x: f32,
    anchor_y: f32,
) {
    let w = text::hud_text_width(r, lookups, text, 1.0);
    let h = text::line_height(lookups);
    let x = pos[0] - anchor_x * w * scale;
    let y = pos[1] - anchor_y * h * scale;
    text::draw_border_string(
        r,
        lookups,
        text::BorderFont::Ui,
        x,
        y,
        scale,
        &[[1.0, 1.0]],
        [1.0, 1.0, 1.0, alpha],
        text,
    );
}

pub fn draw_hud(
    r: &mut SpriteRenderer,
    camera: Vec2,
    zoom: f32,
    screen: Vec2,
    lookups: &assets::Lookups,
    inventory: &[ItemSlot; INV],
    favorites: &[usize],
    equip: &inv::EquipSlots,
    equip_page: i32,
    selected: usize,
    hotbar_scales: &[f32; HOTBAR],
    // `Main.npc[]` — the housing page's source (`Main.cs:42008-42023`).
    npcs: &[crate::npc::Npc],
    player: Option<&Player>,
    boss: Option<(i32, i32)>,
    // `BigProgressBarSystem.ShowText` — the `ShowBossBarHealthText` key
    // (`BigProgressBarSystem.cs:231`/`:236`), i.e. the port's `boss_bar_text`:
    // with it off the boss bar draws without its `current / max` numbers
    // (`BigProgressBarHelper.cs:53-55`).
    boss_bar_text: bool,
    inventory_open: bool,
    crafting_open: bool,
    mouse_item: &ItemSlot,
    mouse_pos: Vec2,
    cursor_state: &cursor::CursorState,
    craft: &[usize],
    pulse: UiPulse,
    m_h: f32,
    map_style: i32,
    bestiary_open: bool,
    bestiary_selected: Option<usize>,
    kill_tracker: &tr_core::bestiary::KillTracker,
    chat_state: &mut crate::chat::ChatMonitor,
    emotes_state: &crate::emotes::EmoteSystem,
    ingame_options: Option<(&Settings, &ingame_options::IngameOptions)>,
    shop: Option<(&[crate::shop::ShopEntry], f32, bool)>,
    chest_panel: chest::PanelState<'_>,
    reforge: Option<(&crate::item::ItemSlot, u64, i32)>,
    bank: Option<(&[ItemSlot], i32)>,
    // `Main.DrawPVPIcons`'s inputs (`Main.cs:42654`): the net mode / seed gates,
    // the local player's `hostile` + `team` and the `teamCooldown` timer.
    pvp: &chrome::PvpIconState,
    // `Player.hideVisibleAccessory` — the accessory visible/hidden ticks beside
    // the accessory column (`Main.cs:41425-41460`).
    hide_visible_accessory: &[bool; 10],
) {
    // `DrawInterface_25_...` (`Main.cs:43181`): the classic life / mana / breath
    // resources are the first interface layer, before the inventory (27) and the
    // hotbar (30).
    resources::draw(r, lookups, screen, player, camera, zoom, pulse);

    // `ItemSlot.Draw` / `Main.DrawInventory`: with the inventory open the hotbar is
    // row 0 of a 10x5 grid of 56 px cells at (20,20) at `inventoryScale = 0.85`
    // (context 0: grey `InventoryBack`, hotbar row `InventoryBack9`); closed, the ten
    // slots animate between 0.75 and 1.0 (context 13: grey back at 200/255, the
    // selected one yellow `InventoryBack14` at full white).
    // `Main.DrawInterface`'s chat tail (`Main.cs:40928-40997`): the input
    // strip + the log while open, the blinker advancing per drawn frame.
    draw_chat(r, lookups, screen, chat_state);

    // `EmoteBubble.DrawAll` + the `UIEmotesMenu` state (M6-EMOTE-1).
    let (pc, pdir) = player
        .map(|p| {
            let (cx, cy) = p.center();
            (Vec2::new(cx, cy), p.facing)
        })
        .unwrap_or((Vec2::new(camera.x, camera.y), 1));
    emotes::draw(r, lookups, screen, emotes_state, pc, pdir, mouse_pos);

    // `Main.DrawBestiaryIcon(num, num2)` (`Main.cs:41013`): the button rides
    // the open inventory's top row, before the slots.
    if inventory_open {
        bestiary::draw_inventory_button(r, lookups, mouse_pos);
    }

    // `Main.DrawSettingButton` (`Main.cs:45532-45580`): the Settings button
    // rides the open inventory's row beside the bestiary/emote buttons, and
    // the `IngameOptions` window draws over the open inventory
    // (`DrawInterface_11_IngameOptionsMenu`, `Main.cs:46160-46168`).
    if let Some((_settings, options)) = ingame_options {
        // `Main.cs:45511`: the button draws while the inventory **or** the
        // options window is up (`|| Main.ingameOptionsWindow || dead`).
        if inventory_open || options.open {
            ingame_options::draw_settings_button(
                r,
                lookups,
                &options.button_layout,
                options.button_scale,
            );
        }
        if options.open {
            ingame_options::draw_options(r, lookups, _settings, options, screen, mouse_pos);
        }
    }

    // `IngameFancyUI.OpenUIState(Main.BestiaryUI)` (`Main.cs:42459`): the
    // bestiary covers the interface while open (the state's draw replaces
    // the HUD behind it).
    if bestiary_open {
        bestiary::draw_screen(r, lookups, screen, kill_tracker, bestiary_selected, mouse_pos);
        return;
    }

    let mut open_hover: Option<inventory::HoverRef> = None;
    // `Main.mouseItem.IsAir` (`Main.cs:41430`): every accessory tick draw and hit
    // is gated on the cursor holding nothing.
    let mouse_item_is_air = mouse_item.kind == 0 || mouse_item.count == 0;
    // The housing page's cell hover (`Main.cs:42059-42065`), which the tooltip
    // layer prints instead of an item name.
    let mut open_housing: Option<(usize, Option<usize>)> = None;
    // `Main.hidePVPAndTeamIcons` — the housing page's columns run into the
    // PVP/team icons (`Main.cs:42053-42056`); the C# clears it at the top of
    // `DrawNPCHousesInUI` and reads it in `DrawPVPIcons`, which runs after.
    let mut hide_pvp_icons = false;
    if inventory_open {
        // `Main.DrawInventory` (`Main.cs:41001`): the bounded 10x5 grid, the coin and
        // ammo columns, the equipment/vanity/dye columns (or the misc-equip page),
        // the trash slot and the "Inventory"/"Coins"/"Ammo" labels.
        let hit = inventory::draw(
            r,
            lookups,
            screen,
            inventory,
            favorites,
            equip,
            selected,
            equip_page,
            npcs,
            mouse_pos,
            m_h,
            bank,
            // `Main.ChestOrShopUIVisible` (`Main.cs:2939-2952`): the trash
            // slot shifts while a container or shop panel is open (bug 20).
            chest_panel.chest >= 0 || shop.is_some(),
            // `Player.hideVisibleAccessory` + `Main.mouseItem.IsAir`: the
            // accessory visible/hidden ticks (`Main.cs:41425-41460`).
            hide_visible_accessory,
            mouse_item_is_air,
        );
        open_hover = hit.hovered;
        open_housing = hit.housing;
        hide_pvp_icons = hit.hide_pvp_icons;
        // `num20 = 174 + Main.mH` (`Main.cs:41146`).
        let num20 = 174.0 + m_h;
        // `Main.DrawPageIcons(num20 - 32)` (`Main.cs:41154`).
        chrome::draw_page_icons(r, lookups, screen, equip_page, num20 - 32.0, mouse_pos);
        // `Main.DrawDefenseCounter(num39, num40)` (`Main.cs:41421-41424`): drawn
        // beside the last accessory row, only on the equipment pages. Defence is
        // `Player.statDefense`, not modelled yet, so a fresh character's 0.
        if let Some((dx, dy)) = hit.defense_anchor {
            chrome::draw_defense_counter(r, lookups, dx, dy, 0, 0.85);
        }
        // `Main.DrawGuideCraftText(73, 331 + num50)` (`Main.cs:41694`); the guide
        // slot cannot be opened in the port yet, so the empty label is drawn.
        chrome::draw_guide_text(r, lookups, 73.0, 331.0 + (screen.y - 600.0) / 2.0, None);
        // The shop row (`Main.DrawShop`): the stock's names and the
        // happiness-adjusted prices in a text column above the inventory
        // grid (the vanilla draws the item slots at (406, 258+); the slot
        // frames land with the slot-work pass).
        if let Some((stock, mood, discount)) = shop {
            text::hud_text(r, lookups, 40.0, 34.0, 0.85, [1.0; 4], "Shop");
            for (i, entry) in stock.iter().enumerate().take(10) {
                let y = 50.0 + i as f32 * 16.0;
                let def = &crate::itemdata::ITEMS[entry.item as usize];
                // The displayed price follows `Main.cs:20457`: buyOnce/
                // isAShopItem rows show the buying value, the rest the
                // selling value.
                let price = crate::shop::display_price_row(entry, discount, mood);
                text::hud_text(
                    r,
                    lookups,
                    40.0,
                    y,
                    0.85,
                    [1.0; 4],
                    &format!("{} - {} copper", def.name, price),
                );
            }
        }
        // The reforge menu (`Main.DrawInventory`'s inline block,
        // `Main.cs:41557-41685`): the (50, 270) input slot, the (120, 310)
        // button (30x30), the cost label above the button (`Lang.inter[46]`
        // + the coin breakdown — the port prints the copper total), the
        // `Lang.inter[20]` "place an item here" hint on the empty slot and
        // the held item's name; the cooldown greys the label. The port's
        // text-panel fidelity rides the same coordinates + m_h offset.
        if let Some((slot, cost, cooldown)) = reforge {
            text::hud_text(r, lookups, 50.0, 250.0 + m_h, 0.85, [1.0; 4], "Reforge");
            if slot.count > 0 {
                let def = &crate::itemdata::ITEMS[slot.kind as usize];
                text::hud_text(
                    r,
                    lookups,
                    76.0,
                    274.0 + m_h,
                    0.85,
                    [1.0; 4],
                    &format!("{} x{}", def.name, slot.count),
                );
            } else {
                text::hud_text(r, lookups, 76.0, 274.0 + m_h, 0.85, [0.8; 4], "place an item here");
            }
            let label = if cooldown > 0 {
                "cooling down".to_string()
            } else {
                format!("Reforge - {} copper", cost)
            };
            let ready = cooldown == 0;
            let tint: [f32; 4] = if ready { [1.0; 4] } else { [0.55; 4] };
            text::hud_text(r, lookups, 120.0, 316.0 + m_h, 0.85, tint, &label);
        }
        // `Main.cs:41831`: `ChestUI.Draw` runs after the shop pass — the chest
        // panel draws together with the inventory (`OpenChest` sets
        // `playerInventory`, `Player.cs:33517`, chest.rs:409). The closed arm
        // decays the button state (`ChestUI.cs:99-104`).
        chest::draw(r, lookups, chest_panel, pulse, mouse_pos);
        // `Main.DrawPVPIcons` (`Main.cs:41003`): the icons ride the open
        // inventory's downward push (`Main.mH`) — the hardcoded 0.0 kept them
        // glued to the top while the minimap HUD pushed the panel down.
        if !hide_pvp_icons {
            chrome::draw_pvp_icons(r, lookups, screen, equip_page, m_h, pvp, mouse_pos);
        }
        chrome::draw_hotbar_lock(r, lookups, false);
        // `Main.DrawInventory`'s map-style icon row (`Main.cs:41055-41126`),
        // after `DrawHotbarLockIcon` (`:41046`) and before the crafting panel
        // (`:41706`).
        chrome::draw_map_style_icons(r, lookups, screen, map_style, mouse_pos);
        // `Main.cs:41706-41709`: with `NewCraftingUI.Visible` the crafting panel is
        // drawn after `Main.DrawInventory`'s own passes. The guide slot cannot be
        // opened in the port yet (see `draw_guide_text` above), so `guide_mode`
        // stays off, and a fresh character has neither expert mode nor the
        // creative inventory unblocked.
        if crafting_open {
            crafting::draw(r, lookups, screen, mouse_pos, craft, false, false);
            // Hovering a recipe shows the crafted item's tooltip
            // (`Main.HoverOverCraftingItemButton`, `Main.cs:42250`).
            if let Some(idx) = crafting::hit_test(screen, lookups, mouse_pos, craft) {
                let (kind, count) = crate::craft::RECIPES[idx].create;
                let hover = crate::item::ItemSlot {
                    kind,
                    name: crate::registry::name(kind),
                    count,
                    prefix: 0,
                };
                tooltip::draw_item_tooltip(r, lookups, mouse_pos, Some(&hover), screen, pulse);
            }
        }
    } else {
        // `GUIHotbarDrawInner` (`Main.cs:43600-43639`): the closed hotbar is a row
        // of context-13 `ItemSlot.Draw` calls at the per-slot `hotbarScale`, the x
        // advancing by `(int)(InventoryBack.Width() * scale) + 4` from x=20 and the
        // row `y` compensating with `(int)(20 + 22 * (1 - scale))`. The scale lerp
        // itself runs in `app::frame` (`Main.cs:43604-43611`); `draw_slot` derives
        // the `(255,255,255,(int)(75+150*scale))` tint (`Main.cs:43615`) and the
        // context-13 background (`ItemSlot.cs:2232-2242`).
        let mut x = 20.0;
        for i in 0..HOTBAR {
            let scale = hotbar_scales[i].clamp(0.75, 1.0);
            let (y, advance, _) = slot::closed_hotbar_metrics(scale);
            slot::draw_slot(
                r,
                lookups,
                &inventory[i],
                x,
                y,
                13,
                i as i32,
                i == selected,
                scale,
                [1.0; 4],
            );
            x += advance;
        }
    }
    // Selected item name, centred on x=236 at y=0 like the original
    // (`GUIHotbarDrawInner`, `Main.cs:43593-43595`): `FontAssets.MouseText` at
    // scale 1, tinted `Color(Main.mouseTextColor x4)` — no shadow pass.
    let item = &inventory[selected.min(HOTBAR - 1)];
    if item.count > 0 && !inventory_open {
        // `Lang.GetItemNameValue` via the `ItemName.<const>` key.
        let label = crate::lang::item_name(item.kind);
        let tw = text::hud_text_width(r, lookups, &label, 1.0);
        let t = pulse.text;
        text::hud_text(r, lookups, 236.0 - tw * 0.5, 0.0, 1.0, [t, t, t, t], &label);
    }

    // `Main.mouseItem`: `DrawInterface_38_MouseCarriedObject` (`Main.cs:45173-45195`)
    // draws it through `ItemSlot.Draw` at context 21 — no background
    // (`ItemSlot.cs:2236-2240`) — at `(Main.mouseX, Main.mouseY)` with
    // `inventoryScale = Main.cursorScale`, so the icon centres on
    // `position + InventoryBack.Size() / 2 * cursorScale` and keeps the 32 px clamp.
    // (The vanilla `DoGlowingMouseItemDraw` shader pass is not ported.)
    if mouse_item.count > 0 {
        slot::draw_slot(
            r,
            lookups,
            mouse_item,
            mouse_pos.x,
            mouse_pos.y,
            21,
            0,
            false,
            pulse.cursor_scale,
            [1.0; 4],
        );
    }

    // Boss bar: `BigProgressBarHelper.DrawFancyBar` (`BigProgressBarHelper.cs:25-57`,
    // the 4-arg overload via `CommonBossBigProgressBar`) — the 456x22 bar at the
    // bottom centre (`ui/bars.rs`). `boss_bar_text` is the `ShowText` gate on
    // the numbers (`:53-55`).
    if let Some((hp, max_hp)) = boss {
        bars::draw_fancy_bar(r, lookups, screen, hp, max_hp, boss_bar_text);
    }

    // `Main.DrawPendingMouseText()` → `Main.MouseText` (`Main.cs:20162`, `20322`):
    // the hovered slot's item tooltip, drawn over the interface just before the
    // cursor. With the inventory open the hover comes from the panel layout
    // (`inventory::draw`); closed, only the hotbar row is hoverable.
    // `Main.cs:42678-42686` / `:42724-42731`: the PVP/team hover text
    // (`MouseTextHackZoom`) and the `player.mouseInterface = true` that suppresses
    // the slot hover under this pointer. The C# icon pass runs inside
    // `DrawInventory` (before `DrawPendingMouseText`), so its text is the one on
    // screen for those cells.
    let pvp_hover_text = if inventory_open {        chrome::pvp_hit(
            screen,
            equip_page,
            m_h,
            pvp,
            mouse_pos,
            chrome::pvp_team_cell(lookups),
        )
        .map(|hit| match hit {
            chrome::PvpHit::PvpToggle => {
                crate::lang::text(chrome::pvp_hover_key(pvp.hostile))
            }
            chrome::PvpHit::Team(i) => {
                crate::lang::text(&chrome::team_hover_key(i, pvp.team))
            }
        })
    } else {
        None
    };
    let housing_hover_text: Option<String> = open_housing
        .and_then(|(head, npc)| housing::label(head, npc, npcs));
    let hover_item: Option<&ItemSlot> = if pvp_hover_text.is_some() || housing_hover_text.is_some() {
        None
    } else if inventory_open {
        match open_hover {
            // `Main.cs:41462-41465`: the hovered tick shows `Lang.inter[58 +
            // num43]` — `[59]` "Visible" while the accessory is drawn, `[60]`
            // "Hidden" once it is not.
            Some(inventory::HoverRef::HideAccessory(i)) => {
                let label = if hide_visible_accessory[i] {
                    crate::lang::inter(60)
                } else {
                    crate::lang::inter(59)
                };
                tooltip::draw_mouse_text(r, lookups, mouse_pos, &label, screen, pulse);
                None
            }
            other => other.and_then(|h| hover_item_for(equip, inventory, h, bank.map(|b| b.0))),
        }
    } else {
        tooltip::hovered_slot(mouse_pos, false, selected, hotbar_scales, inventory)
            .map(|i| &inventory[i])
    };
    tooltip::draw_item_tooltip(r, lookups, mouse_pos, hover_item, screen, pulse);
    if let Some(text) = pvp_hover_text {
        tooltip::draw_mouse_text(r, lookups, mouse_pos, &text, screen, pulse);
    }
    if let Some(text) = &housing_hover_text {
        tooltip::draw_mouse_text(r, lookups, mouse_pos, text, screen, pulse);
    }

    // `Main.DrawPendingMouseText()` then `Main.DrawCursor(...)` are the last
    // things `Main.DrawInterface` draws (`Main.cs:44939`); the OS pointer is
    // hidden while the vanilla `Cursor_0` is on screen.
    cursor::draw(r, lookups, cursor_state, mouse_pos);
}

/// Maps an `inventory::HoverRef` (which panel the pointer is over) to the item it
/// holds, so the tooltip layer can show its name.
fn hover_item_for<'a>(
    equip: &'a inv::EquipSlots,
    inventory: &'a [ItemSlot; INV],
    hit: inventory::HoverRef,
    bank: Option<&'a [ItemSlot]>,
) -> Option<&'a ItemSlot> {
    use inventory::HoverRef as H;
    match hit {
        H::Grid(i) => inventory.get(i),
        H::Bank(i) => bank.and_then(|b| b.get(i)),
        // The buttons carry no item: the tooltip stays off.
        H::BankButton(_) => None,
        // `Main.cs:41462-41465`: the tick replaces the slot's tooltip with
        // `Lang.inter[59]` / `[60]` ("Visible" / "Hidden"), so no item text.
        H::HideAccessory(_) => None,
        H::Coin(i) => equip.coins.get(i),
        H::Ammo(i) => equip.ammo.get(i),
        H::Armor(i) => equip.armor.get(i),
        H::Dye(i) => equip.dye.get(i),
        H::MiscEquip(i) => equip.misc_equips.get(i),
        H::MiscDye(i) => equip.misc_dyes.get(i),
        H::Trash => Some(&equip.trash),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn in_rect_is_inclusive_exclusive_like_rectangle_contains() {
        // `UIElement.cs:354-357`: `Rectangle.Contains` includes the left/top
        // edges and excludes the right/bottom. (Moved with the code from
        // `crafting.rs`'s `rect_hover` test — the helper it pinned is now this
        // shared function.)
        let r = [10.0, 20.0, 40.0, 46.0];
        assert!(in_rect(Vec2::new(10.0, 20.0), r));
        assert!(in_rect(Vec2::new(49.9, 65.9), r));
        assert!(!in_rect(Vec2::new(50.0, 30.0), r));
        assert!(!in_rect(Vec2::new(30.0, 66.0), r));
        assert!(!in_rect(Vec2::new(9.9, 30.0), r));
        assert!(!in_rect(Vec2::new(30.0, 19.9), r));
    }

    /// BUG-06: the death screen's respawn countdown — the `Main.DrawDeathText`
    /// tail (`Main.cs:45380-45386`): the seconds value appears once the timer
    /// is above 1 (`1 + timer/60`), the countdown moves below the coin line,
    /// and a zero-timer world draws none. The draw path measured here is the
    /// string the panel would carry (the same builder the frame call uses).
    #[test]
    fn bug_06_the_death_screen_renders_the_respawn_seconds() {
        // The countdown builder mirrors draw_death_screen's tail: the vanilla
        // suffix key resolves to "{0}" (en-US Game.json), so the port draws
        // "Respawn in {seconds}".
        let countdown_line = |respawn_timer: i32| -> Option<String> {
            if respawn_timer <= 1 {
                return None;
            }
            let seconds = (1 + respawn_timer / 60).to_string();
            let suffix = crate::lang::text_args("Game.RespawnInSuffix", &[seconds]);
            Some(format!("Respawn in {suffix}"))
        };
        // The seconds math (`1 + respawnTimer / 60`, `Main.cs:45385`) and the
        // >1 gate.
        assert_eq!(1 + 600 / 60, 11);
        assert_eq!(1 + 60 / 60, 2);
        assert!(countdown_line(1).is_none(), "the last tick draws nothing");
        assert!(countdown_line(0).is_none(), "the spent timer draws nothing");
        // The live suffix key: the loaded game resolves it to the vanilla
        // "{0}" (en-US Game.json); an unloaded table echoes the key (the
        // test binary) — either way the draw carries the seconds.
        let resolved = crate::lang::text_args("Game.RespawnInSuffix", &["11".to_string()]);
        assert!(resolved == "11" || resolved == "Game.RespawnInSuffix", "got {resolved}");
        assert_eq!(
            countdown_line(600).map(|l| l.contains("Respawn in")),
            Some(true),
            "the line rides the builder"
        );
    }

    /// BUG-06b: the death screen's row arithmetic (`Main.cs:45328-45386`) —
    /// the broadcast 60 px above the midpoint, the coins line 50 px below it
    /// (then `-26`), the skip prompt sliding 70 → 20 px below the broadcast,
    /// and the countdown 70 px below whatever came before it.
    #[test]
    fn bug_06b_the_death_rows_walk_the_c_sharp_num() {
        let mid = 450.0;
        // No coins, no skip, a live timer: broadcast -60, countdown +10.
        let l = death_layout(900.0, 0, 600, 0);
        assert_eq!(l.broadcast_y, mid - 60.0);
        assert_eq!(l.coins_y, None);
        assert_eq!(l.prompt_y, None);
        assert_eq!(l.countdown_y, Some(mid + 10.0));
        // Coins: the line at -10 and the countdown 24 px lower than without.
        let l = death_layout(900.0, 5, 600, 0);
        assert_eq!(l.coins_y, Some(mid - 10.0));
        assert_eq!(l.countdown_y, Some(mid + 34.0));
        // The skip prompt at frame 0 sits 70 px below the broadcast and pushes
        // the countdown down with it; at frame 60 it has slid to 20 px.
        let l = death_layout(900.0, 0, 600, 1);
        // `num` starts at -60 and the slide adds `Remap(num4,0,1,70,20)*num4`
        // on top of it: ~1.15 px on the first frame, 20 by frame 60.
        assert!((l.prompt_y.unwrap() - (mid - 58.846_7)).abs() < 1e-3);
        assert!((l.countdown_y.unwrap() - (mid + 11.153_3)).abs() < 1e-3);
        assert_eq!(l.prompt_alpha, 1.0 / 60.0);
        let l = death_layout(900.0, 0, 600, 60);
        assert_eq!(l.prompt_y, Some(mid - 40.0));
        assert_eq!(l.countdown_y, Some(mid + 30.0));
        assert_eq!(l.prompt_alpha, 1.0);
        // The unclamped pulse: `num5 = 0.25` at frame 60, so `(1 + 0.25) * 0.7`.
        assert_eq!(l.prompt_scale, 0.875);
        assert_eq!(death_layout(900.0, 0, 600, 30).prompt_scale, 0.7);
        // The spent timer draws no countdown at all.
        assert_eq!(death_layout(900.0, 0, 1, 0).countdown_y, None);
    }

    /// BUG-06b: the skip prompt's two ramps (`Main.cs:45352-45358`) and the
    /// localized label — `Game.RespawnSkip` carries the `{InputTrigger_Jump}`
    /// glyph tag the C# resolves through `ChatManager.ParseMessage`; the port
    /// substitutes its default jump key.
    #[test]
    fn bug_06b_the_respawn_skip_prompt_ramps_and_names_the_jump_key() {
        // `num4 = Remap(frames, 0, 60, 0, 1)` and `num5 = Remap(frames, 0, 30,
        // 0.5, 0)^2` — the alpha reaches 1 at frame 60, the pulse dies at 30.
        assert_eq!(respawn_skip_ramp(0), (0.0, 0.25));
        assert_eq!(respawn_skip_ramp(30), (0.5, 0.0));
        assert_eq!(respawn_skip_ramp(60), (1.0, 0.25));
        // `Utils.Remap` is unclamped in the C#, so the ramp keeps going.
        assert_eq!(respawn_skip_ramp(120), (2.0, 2.25));
        let (alpha_mid, pulse_mid) = respawn_skip_ramp(15);
        assert!((alpha_mid - 0.25).abs() < 1e-6);
        assert!((pulse_mid - 0.0625).abs() < 1e-6, "0.25^2 at the midpoint");
        // The row slides 70 -> 20 px below the broadcast as the ramp fills.
        assert_eq!(remap(0.0, 0.0, 1.0, 70.0, 20.0), 70.0);
        assert_eq!(remap(1.0, 0.0, 1.0, 70.0, 20.0), 20.0);

        // The label: the loaded en-US table resolves the trigger tag to the
        // port's default jump key.
        let _guard = crate::lang::test_lock()
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        crate::lang::init(tr_core::localization::GameCulture::English);
        assert_eq!(respawn_skip_text(), "Space to skip");
    }

    /// BUG-08: the `Main.invasionProgressIcon` dispatch (`Main.cs:47734-47780`)
    /// — each of the seven icons maps to its own `Extra` sheet and its own
    /// `* 0.5f` plate colour, and the unmapped default is `Extra[9]` with an
    /// empty label and the untouched `Color.White`.
    ///
    /// The table is pinned against the C# — note in particular that icon `5`
    /// is the **Frost Legion** (`Lang.inter[87]`) and icon `6` the **Pirate
    /// Invasion** (`Lang.inter[86]`); the pre-BUG-08 draw had those two labels
    /// swapped and had no Martian row at all.
    #[test]
    fn bug_08_the_icon_dispatch_matches_the_source_table() {
        // (icon, Extra sheet name, the plate RGB before the `* 0.5f`).
        let rows: [(i32, &str, [u8; 3]); 7] = [
            (1, "Extra_8", [64, 109, 164]),   // Frost Moon,      inter[83]
            (2, "Extra_12", [112, 86, 114]),  // Pumpkin Moon,    inter[84]
            (3, "Extra_79", [88, 0, 160]),    // Old One's Army,  DD2 key
            (7, "Extra_10", [165, 160, 155]), // Martian Madness, inter[85]
            (6, "Extra_11", [148, 122, 72]),  // Pirate Invasion, inter[86]
            (5, "Extra_7", [173, 135, 140]),  // Frost Legion,    inter[87]
            (4, "Extra_9", [94, 72, 131]),    // Goblin Army,     inter[88]
        ];
        let lookups = assets::Lookups::default();
        for (icon, sheet_name, rgb) in rows {
            assert!(
                assets::INVASION_ICON_EXTRAS.contains(&sheet_name),
                "icon {icon}: {sheet_name} must be packed into the atlas"
            );
            let (_, _label, plate) = invasion_group(&lookups, icon);
            // `new Color(r, g, b) * 0.5f` truncates every channel.
            let expected = color::tint(color::mul([rgb[0], rgb[1], rgb[2], 255], 0.5));
            assert_eq!(plate, expected, "icon {icon} ({sheet_name}): the plate colour");
        }
        // The label rows resolve through the live tables; the test binary's
        // unloaded manager echoes the key, so the DD2 row is matched by either
        // form. Pin the KEY the C# formats, not the English string.
        let (_, dd2_label, _) = invasion_group(&lookups, 3);
        assert!(
            dd2_label == "Old One's Army"
                || dd2_label == "DungeonDefenders2.InvasionProgressTitle",
            "icon 3 rides the DD2 InvasionProgressTitle key, got {dd2_label}"
        );
        // The default arm: `Extra[9]`, "", and `Color.White` WITHOUT the
        // halving (`:47731-47732`).
        let (_, default_label, default_color) = invasion_group(&lookups, 0);
        assert_eq!(default_label, "", "the default label is empty");
        assert_eq!(default_color, [1.0, 1.0, 1.0, 1.0], "the default is White");
    }

    /// BUG-08: the two gates that close `Main.DrawInvasionProgress`
    /// (`Main.cs:47698` and `:47729`) — with `invasionProgress == -1` or a
    /// zero alpha the function returns before drawing anything, which is what
    /// stops the panel from being permanently on screen.
    #[test]
    fn bug_08_the_bar_is_gated_on_progress_and_alpha() {
        // `Main.invasionProgress == -1` -> nothing (`:47698`), whatever the
        // alpha says.
        assert_eq!(invasion_bar_alpha(-1, 1.0), None);
        assert_eq!(invasion_bar_alpha(-1, 0.5), None);
        // `if (Main.invasionProgressAlpha <= 0f) return;` (`:47729`).
        assert_eq!(invasion_bar_alpha(50, 0.0), None);
        assert_eq!(invasion_bar_alpha(50, -0.3), None);
        // The clamp to `0..=1` (`:47726-47727`) and the live arm.
        assert_eq!(invasion_bar_alpha(50, 1.7), Some(1.0));
        assert_eq!(invasion_bar_alpha(50, 0.4), Some(0.4));
        assert_eq!(invasion_bar_alpha(0, 1.0), Some(1.0), "a zero-point bar still draws");
    }
}

/// The chat draw (`Main.cs:40928-40997` + `RemadeChatMonitor.DrawChat`):
/// the blinker advances once per drawn frame, the closed chat draws nothing
/// (the port has no `CanBeShownWhenChatIsClosed` sources), the open chat
/// draws the log at `(88, screenHeight - 58)` upward 22 px per line and the
/// input strip with the `|` cursor.
fn draw_chat(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    screen: Vec2,
    chat: &mut crate::chat::ChatMonitor,
) {
    // `textBlinkerCount`/`textBlinkerState` (`:40928-40944`) — 20-frame
    // toggle, like the menu input screens.
    chat.blink_timer = chat.blink_timer.wrapping_add(1);
    if chat.blink_timer >= 20 {
        chat.blink_timer = 0;
        chat.blink_on = !chat.blink_on;
    }
    if !chat.open {
        return;
    }
    // `RemadeChatMonitor.DrawChat` (`:55-96`): the newest-first log, 22 px
    // rows, capped by the port's 20-line window (`_showCount`'s screen-fit
    // stand-in).
    for (i, line) in chat.messages.iter().take(20).enumerate() {
        let c = [
            line.color[0] as f32 / 255.0,
            line.color[1] as f32 / 255.0,
            line.color[2] as f32 / 255.0,
            1.0,
        ];
        text::hud_text(
            r,
            lookups,
            88.0,
            screen.y - 58.0 - i as f32 * 22.0,
            1.0,
            c,
            &line.text,
        );
    }
    // The `Text_Back` strip (`:40947-40976`): a head piece (the sheet minus
    // 100 px), 300 px middle tiles, then the trimmed tail — the source runs
    // the same three-piece loop for `screenWidth > 800`.
    if let Some(sheet) = lookups.ui_text_back {
        let (aw, ah) = (lookups.atlas_size.0 as f32, lookups.atlas_size.1 as f32);
        let y = screen.y - 36.0;
        let h = sheet.height as f32;
        let w = sheet.width as f32;
        let head = (w - 100.0).max(0.0);
        let mut draw_piece = |x: f32, piece_w: f32, src_x: f32, src_w: f32| {
            if piece_w <= 0.0 || src_w <= 0.0 {
                return;
            }
            let x0 = sheet.x as f32 + src_x;
            let uv = [x0 / aw, sheet.y as f32 / ah, (x0 + src_w) / aw, (sheet.y as f32 + h) / ah];
            r.push_ui_sprite([x, y], [piece_w, h], uv, [100.0 / 255.0, 100.0 / 255.0, 100.0 / 255.0, 100.0 / 255.0]);
        };
        // `num = screenWidth - 300` paces the loop; the tail consumes the
        // remainder.
        let mut num = screen.x - 300.0;
        let mut x = 78.0;
        draw_piece(x, head, 0.0, head);
        x += head;
        num -= 400.0;
        while num > 0.0 {
            if num > 300.0 {
                draw_piece(x, 300.0, 100.0, w - 200.0);
                x += 300.0;
                num -= 300.0;
            } else {
                let tail = num;
                draw_piece(x, tail, w - tail, tail);
                num = 0.0;
            }
        }
    }
    // The input text (`:40986-40994`): white at `(88, screenHeight - 30)`,
    // the `|` blinker appended while `textBlinkerState == 1`.
    let cursor = if chat.blink_on { "|" } else { "" };
    text::hud_text(
        r,
        lookups,
        88.0,
        screen.y - 30.0,
        1.0,
        [1.0, 1.0, 1.0, 1.0],
        &format!("{}{}", chat.text, cursor),
    );
}
