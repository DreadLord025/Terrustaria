//! `Terraria.PopupText` (`PopupText.cs:10-991`) — the floating item-name
//! popups, ported for the reforge reveal (W4-g57): the two contexts
//! `ItemReforge` / `ItemReforge_Best` that `Main.ReforgeItemInReforgeSlot`
//! spawns (`Main.cs:41906`).
//!
//! Scope: the reforge contexts carry `effectStyle == None`
//! (`PrepareEffects`, `PopupText.cs:395-412` — the fancy styles are
//! `ItemCraft`-only), so the `EmitFancyFlashDust` arm (`:674-676`), the
//! per-char offsets/colours (`:828-916`) and the side textures (`:967-980`,
//! `texture2D` stays null for these contexts) never run and are not ported.
//! The spawn path draws NO `Main.rand` (`PopupText.cs:194-412`; the i45
//! presentation tail owns the reforge's draws — this module adds none).
//!
//! The pool is the C#'s fixed 20 (`maxItemText`, `PopupText.cs:12`); the
//! update is `Update` (`:644-741`) driven by `UpdateItemText` (`:752-764`)
//! beside `CombatText.UpdateCombatText` (`Main.cs:18077-18078`), and the
//! draw is `DrawItemTextPopups`' non-effect path (`:775-983`) right after
//! the combat-text pass (`Main.cs:62397-62402`).

use glam::Vec2;
use tr_render::SpriteRenderer;

use crate::assets;
use crate::ui::text;

/// `PopupText.maxItemText = 20` (`PopupText.cs:12`).
pub const MAX_POPUPS: usize = 20;
/// `PopupText.activeTime = 60` (the static ctor, `PopupText.cs:988`).
pub const ACTIVE_TIME: i32 = 60;
/// `popupText2.lifeTime = 60` (`PopupText.cs:326`).
pub const LIFE_TICKS: i32 = 60;
/// `popupText2.velocity.Y = -7f` (`PopupText.cs:325`).
pub const SPAWN_VY: f32 = -7.0;

/// The two reforge contexts (`PopupTextContext.ItemReforge` /
/// `ItemReforge_Best`, selected at `Main.cs:41906`). The other contexts
/// spawn from unported sites; the pool only ever holds these two.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Context {
    #[default]
    ItemReforge,
    ItemReforgeBest,
}

/// `PopupText` (`PopupText.cs:16-72`), reduced to the fields the reforge
/// contexts read (no coin/sonar/npc/advanced/effect state — those spawn
/// paths are unported). Field defaults mirror the C# initializers
/// (`alphaDir = 1` `:22`, `scale = 1f` `:30`).
#[derive(Clone, Debug, PartialEq)]
pub struct PopupText {
    pub position: [f32; 2],
    pub velocity: [f32; 2],
    pub alpha: f32,
    pub alpha_dir: i32,
    /// `name` = `Item.AffixName()` (`PopupText.cs:323`).
    pub name: String,
    /// `displayText` = name + " (stack)" when stack > 1
    /// (`PrepareDisplayText`, `:414-425`).
    pub display_text: String,
    pub stack: i32,
    pub scale: f32,
    pub rotation: f32,
    pub color: [u8; 4],
    pub active: bool,
    pub life_time: i32,
    pub frames_since_spawn: i32,
    /// `NoStack` (`:46`) — the reforge spawn passes `noStack: true`
    /// (`Main.cs:41906`), so the merge loop (`:209-308`) always skips.
    pub no_stack: bool,
    /// `expert` (`:54`) — `Item.expert`; the port's item table carries no
    /// expert flag (no prefixable item is expert-only), so spawns pass
    /// false (documented in ANALYSIS).
    pub expert: bool,
    /// `master` (`:56`) — `newItem.rare == -13` (`PrepareEffects`, `:397-400`).
    pub master: bool,
    pub context: Context,
}

impl Default for PopupText {
    fn default() -> Self {
        Self {
            position: [0.0; 2],
            velocity: [0.0; 2],
            alpha: 0.0,
            alpha_dir: 1, // `PopupText.cs:22`
            name: String::new(),
            display_text: String::new(),
            stack: 0,
            scale: 1.0, // `PopupText.cs:30`
            rotation: 0.0,
            color: [0; 4],
            active: false,
            life_time: 0,
            frames_since_spawn: 0,
            no_stack: false,
            expert: false,
            master: false,
            context: Context::ItemReforge,
        }
    }
}

/// The 20-slot pool + `PopupText.numActive` (`PopupText.cs:44`) — the count
/// is refreshed AFTER the update loop (`UpdateItemText`, `:752-764`), so an
/// `Update` reads the previous tick's count (`:689`).
#[derive(Clone, Debug, Default)]
pub struct Pool {
    pub texts: Vec<PopupText>,
    pub num_active: usize,
}

/// `PopupText.PrepareDisplayText` (`PopupText.cs:414-420`): the name plus
/// the " (stack)" suffix when the stack exceeds 1.
pub fn display_text(name: &str, stack: i32) -> String {
    if stack > 1 {
        format!("{name} ({stack})")
    } else {
        name.to_string()
    }
}

/// `Item.GetPopupRarityColor(int)` (`Item.cs:652-707`) — the popup tint
/// `PrepareEffects` assigns through `CraftingEffects.GetEffectDetails`'
/// `Rarity = newItem.rare` (`CraftingEffects.cs:65-69`, `PopupText.cs:405`).
/// Distinct from the tooltip's `MouseText` table (`tooltip::item_name_color`).
pub fn popup_rarity_color(rare: i32) -> [u8; 4] {
    let rgb = match rare {
        1 => [150, 150, 255],           // `Item.cs:654-657`
        2 => [150, 255, 150],           // `:658-661`
        3 => [255, 200, 150],           // `:662-665`
        4 => [255, 150, 150],           // `:666-669`
        5 => [255, 150, 255],           // `:670-673`
        -11 => [255, 175, 0],           // `:674-677`
        -1 => [130, 130, 130],          // `:678-681`
        6 => [210, 160, 255],           // `:682-685`
        7 => [150, 255, 10],            // `:686-689`
        8 => [255, 255, 10],            // `:690-693`
        9 => [5, 200, 255],             // `:694-697`
        10 => [255, 40, 100],           // `:698-701`
        r if r >= 11 => [180, 40, 255], // `:702-705`
        _ => [255, 255, 255],           // `Color.White` (`:706`)
    };
    [rgb[0], rgb[1], rgb[2], 255]
}

/// `PopupText.FindNextItemTextSlot` (`PopupText.cs:501-525`): the first
/// inactive slot, else the topmost active one (the smallest `position.Y`
/// under the `bottomWorld` ceiling).
fn find_next_slot(pool: &Pool, bottom_world: f32) -> Option<usize> {
    for i in 0..MAX_POPUPS {
        if !pool.texts.get(i).is_some_and(|t| t.active) {
            return Some(i);
        }
    }
    // `:512-523` — every slot active: replace the highest-on-screen one.
    let mut num = None;
    let mut num2 = bottom_world;
    for j in 0..MAX_POPUPS {
        if let Some(t) = pool.texts.get(j) {
            if num2 > t.position[1] {
                num = Some(j);
                num2 = t.position[1];
            }
        }
    }
    num
}

/// The reforge spawn — `PopupText.NewText(PopupTextContext, Item, Vector2,
/// int, noStack: true)` (`PopupText.cs:194-393`) as `Main.cs:41906` calls
/// it. The merge loop (`:209-308`) always `continue`s on `noStack` (`:212`),
/// so the spawn lands straight in `FindNextItemTextSlot` (`:309`). The
/// `Main.showItemText` gate (`:196-199`) rides the caller
/// (`settings.pickup_text`); `netMode == 2` (`:204-207`) never holds in the
/// single-player port. `text_size` is the caller's
/// `FontAssets.MouseText.MeasureString(displayText)` (`:317`);
/// `color` is `popup_rarity_color(item.rare)` (`PrepareEffects`, `:405`).
/// No RNG parameter: the path draws no `Main.rand` (the i45 tail owns the
/// reforge's draws).
#[allow(clippy::too_many_arguments)]
pub fn spawn_reforge(
    pool: &mut Pool,
    best: bool,
    name: &str,
    stack: i32,
    position: [f32; 2],
    color: [u8; 4],
    master: bool,
    expert: bool,
    text_size: [f32; 2],
    bottom_world: f32,
) -> Option<usize> {
    let i = find_next_slot(pool, bottom_world)?;
    if pool.texts.len() <= i {
        pool.texts.resize_with(i + 1, PopupText::default);
    }
    let t = &mut pool.texts[i];
    // `ResetText` (`PopupText.cs:99-116`).
    t.no_stack = true; // the reforge call's `noStack: true` (`Main.cs:41906`)
    t.scale = 0.0; // `:109`
    t.rotation = 0.0; // `:110`
    t.alpha = 1.0; // `:111`
    t.alpha_dir = -1; // `:112`
    t.frames_since_spawn = 0; // `:113`
    t.expert = expert; // `PrepareEffects` (`:401`)
    t.master = master; // `:397-400`
    t.active = true; // `:320`
                     // `:321-322` — centred on the spawn point by the measured display text.
    t.position = [
        position[0] - text_size[0] * 0.5,
        position[1] - text_size[1] * 0.5,
    ];
    t.name = name.to_string(); // `:323` — `Item.AffixName()`
    t.stack = stack; // `:324`
    t.velocity = [0.0, SPAWN_VY]; // `:325`
    t.life_time = LIFE_TICKS; // `:326` (longText is false)
    t.context = if best {
        Context::ItemReforgeBest
    } else {
        Context::ItemReforge
    }; // `:327`
    t.color = color; // `PrepareEffects` (`:405`), coinText false
    t.display_text = display_text(name, stack); // `PrepareDisplayText` (`:390`)
    Some(i)
}

/// The per-frame colour clocks `Update`/`DrawItemTextPopups` read off
/// `Main` — all visual-only, no RNG.
#[derive(Clone, Copy, Debug)]
pub struct Env {
    /// `Main.DiscoR/G/B` (`PopupText.cs:665`).
    pub disco: [u8; 3],
    /// `Main.masterColor` (`:669`; the 0..1 ±0.05 cycle,
    /// `Main.cs:18225-18235`).
    pub master_color: f32,
    /// `Main.mouseTextColor` (`:665`/`:669`).
    pub mouse_text_color: u8,
    /// `Main.GlobalTimeWrappedHourly` (`:821`).
    pub global_time_wrapped_hourly: f32,
    /// `Main.player[myPlayer].gravDir` (`:952-955`) — unported (always 1).
    pub grav_dir: f32,
}

/// The renderer-free hitbox twin of `GetTextHitbox` (the collision pass
/// runs in the update, before any GPU exists): the caller supplies the
/// measured `MeasureString(displayText)` size.
fn hitbox_from(t: &PopupText, size: [f32; 2]) -> [f32; 2] {
    [size[0] * t.scale, size[1] * t.scale * 0.8]
}

/// XNA `Rectangle.Intersects` — strict on all four edges.
fn rects_intersect(a: [i32; 4], b: [i32; 4]) -> bool {
    b[0] < a[0] + a[2] && a[0] < b[0] + b[2] && b[1] < a[1] + a[3] && a[1] < b[1] + b[3]
}

/// `PopupText.Update(whoAmI)` (`PopupText.cs:644-741`). `target_scale` is
/// `PopupText.TargetScale` (`:88`, `Main.UIScale / RenderZoom.X`);
/// `sizes[i]` is the caller's `MeasureString(displayText)` per slot (the
/// update runs renderer-free). The `AnyEffect` dust arm (`:674-676`) never
/// fires for the reforge contexts (effectStyle None) — no RNG.
pub fn update(
    pool: &mut Pool,
    who_am_i: usize,
    target_scale: f32,
    env: Env,
    sizes: &[[f32; 2]; MAX_POPUPS],
) {
    if !pool.texts.get(who_am_i).is_some_and(|t| t.active) {
        return; // `:646-649`
    }
    // `:650` — the frame counter.
    pool.texts[who_am_i].frames_since_spawn += 1;
    // `:652-662` — the ±0.01 alpha oscillation in [0.7, 1.0].
    {
        let t = &mut pool.texts[who_am_i];
        t.alpha += t.alpha_dir as f32 * 0.01;
        if t.alpha <= 0.7 {
            t.alpha = 0.7;
            t.alpha_dir = 1;
        }
        if t.alpha >= 1.0 {
            t.alpha = 1.0;
            t.alpha_dir = -1;
        }
        // `:663-670` — the expert disco / master pulse recolour.
        if t.expert {
            t.color = [
                env.disco[0],
                env.disco[1],
                env.disco[2],
                env.mouse_text_color,
            ];
        } else if t.master {
            t.color = [
                255,
                (env.master_color * 200.0) as u8,
                0,
                env.mouse_text_color,
            ];
        }
    }
    // `:672-673` — this popup's hitbox rect (truncating casts).
    let hb = hitbox_from(&pool.texts[who_am_i], sizes[who_am_i]);
    let pos = pool.texts[who_am_i].position;
    let rect = [
        (pos[0] - hb[0] / 2.0) as i32,
        (pos[1] - hb[1] / 2.0) as i32,
        hb[0] as i32,
        hb[1] as i32,
    ];
    // `:678-697` — the overlap push: an intersecting popup below this one
    // (or same-Y with a higher index) extends BOTH lifetimes.
    let mut flag = false;
    for i in 0..MAX_POPUPS {
        if i == who_am_i || !pool.texts.get(i).is_some_and(|t| t.active) {
            continue;
        }
        let hb2 = hitbox_from(&pool.texts[i], sizes[i]);
        let p2 = pool.texts[i].position;
        let rect2 = [
            (p2[0] - hb2[0] / 2.0) as i32,
            (p2[1] - hb2[1] / 2.0) as i32,
            hb2[0] as i32,
            hb2[1] as i32,
        ];
        if rects_intersect(rect, rect2) && (pos[1] < p2[1] || (pos[1] == p2[1] && who_am_i < i)) {
            flag = true;
            let mut num = pool.num_active as i32; // `:689` — last tick's count
            if num > 3 {
                num = 3; // `:690-693`
            }
            pool.texts[i].life_time = ACTIVE_TIME + 15 * num; // `:694`
            pool.texts[who_am_i].life_time = ACTIVE_TIME + 15 * num; // `:695`
        }
    }
    let t = &mut pool.texts[who_am_i];
    // `:698-713` — the velocity curve.
    if !flag {
        t.velocity[1] *= 0.86;
        if t.scale == target_scale {
            t.velocity[1] *= 0.4;
        }
    } else if t.velocity[1] > -6.0 {
        t.velocity[1] -= 0.2;
    } else {
        t.velocity[1] *= 0.86;
    }
    t.velocity[0] *= 0.93; // `:714`
    t.position[0] += t.velocity[0]; // `:715`
    t.position[1] += t.velocity[1];
    t.life_time -= 1; // `:716`
    if t.life_time <= 0 {
        // `:717-729` — the shrink-out (0.03/tick, gone below 0.1×target).
        t.scale -= 0.03 * target_scale;
        if t.scale < 0.1 * target_scale {
            t.active = false;
        }
        t.life_time = 0;
    } else {
        // `:730-740` — the 0.1/tick ramp up to the target.
        if t.scale < target_scale {
            t.scale += 0.1 * target_scale;
        }
        if t.scale > target_scale {
            t.scale = target_scale;
        }
    }
}

/// `PopupText.UpdateItemText` (`PopupText.cs:752-764`): tick every active
/// slot in index order, then refresh `numActive`.
pub fn update_all(pool: &mut Pool, target_scale: f32, env: Env, sizes: &[[f32; 2]; MAX_POPUPS]) {
    for i in 0..MAX_POPUPS {
        if pool.texts.get(i).is_some_and(|t| t.active) {
            update(pool, i, target_scale, env, sizes);
        }
    }
    pool.num_active = pool
        .texts
        .iter()
        .take(MAX_POPUPS)
        .filter(|t| t.active)
        .count();
}

/// The best-tier popup tint (`DrawItemTextPopups`, `PopupText.cs:820-824`):
/// `Main.hslToRgb(GlobalTimeWrappedHourly * 0.6f % 1f, 1f, 0.6f) * 0.6f` —
/// the cycling rainbow, XNA-truncated by 0.6.
pub fn best_popup_color(global_time_wrapped_hourly: f32) -> [u8; 4] {
    let hue = (global_time_wrapped_hourly * 0.6) % 1.0;
    let rgb = crate::ui::color::hsl_to_rgb(hue as f64, 1.0, 0.6, 255);
    crate::ui::color::mul(rgb, 0.6)
}

/// XNA `Color.Lerp` at 0.5 — per-channel `(byte)(a + (b - a) * 0.5)`
/// truncation (`PopupText.cs:960`'s `Color.Lerp(value3, value4, 0.5f)`).
fn lerp_half(a: [u8; 4], b: [u8; 4]) -> [f32; 4] {
    let c = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * 0.5) as u8 as f32 / 255.0;
    [c(a[0], b[0]), c(a[1], b[1]), c(a[2], b[2]), c(a[3], b[3])]
}

/// `PopupText.DrawItemTextPopups(scaleTarget)` (`PopupText.cs:775-983`),
/// non-effect path. The C# draws inside the world-zoom batch with
/// `TargetScale = UIScale / RenderZoom` (`:88`), so the effective glyph
/// size is `popup.scale * zoom` — a constant UI size at any world zoom;
/// the port multiplies the font scale and the shadow offsets by `zoom`
/// explicitly (the `combat_text::draw` screen-space convention). The
/// reforge contexts keep `rotation == 0` (`ResetText` `:110`; no effect
/// style ever spins them) and `charColors == null`, so the plain
/// `DrawString` arm (`:965`) is the only one ported.
pub fn draw(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    pool: &Pool,
    camera: Vec2,
    zoom: f32,
    screen: Vec2,
    target_scale: f32,
    env: Env,
) {
    for t in pool.texts.iter().take(MAX_POPUPS) {
        if !t.active {
            continue; // `:783-786`
        }
        // `:787-789` — the measured size and its half (the draw origin).
        let w = text::ui_font_width(lookups, &t.display_text, 1.0)
            .unwrap_or_else(|| r.text_width(&t.display_text, 1.0));
        let h = text::ui_font_height(lookups, 1.0).unwrap_or(8.0);
        let half = [w * 0.5, h * 0.5];
        // `:790-800` — the scale ratio, the shadow byte and the alpha.
        let mut num = target_scale;
        let num2 = t.scale / num;
        let num3 = (255.0 - 255.0 * num2) as i32;
        let num5 = t.color[3] as f32 * num2 * t.alpha;
        // `:801-825` — the context override: Best cycles the rainbow,
        // halves the shadow spread and dims the fill to 0.8.
        let mut color = [0u8; 4]; // `Color.Black`
        let mut num6 = 1.0f32;
        if t.context == Context::ItemReforgeBest {
            color = best_popup_color(env.global_time_wrapped_hourly); // `:821`
            num *= 0.5; // `:822`
            num6 = 0.8; // `:823`
        }
        let num21 = num3 as f32 / 255.0; // `:917`
                                         // `:918-981` — the four shadow passes then the fill.
        for n in 0..5 {
            let (mut num22, mut num23) = (0.0f32, 0.0f32);
            let fill;
            match n {
                0 => {
                    num22 -= num * 2.0; // `:925-927`
                    fill = false;
                }
                1 => {
                    num22 += num * 2.0; // `:928-930`
                    fill = false;
                }
                2 => {
                    num23 -= num * 2.0; // `:931-933`
                    fill = false;
                }
                3 => {
                    num23 += num * 2.0; // `:934-936`
                    fill = false;
                }
                _ => fill = true,
            }
            // `:938` — the fill colour: `color * num2 * alpha * num6`.
            let fill_color =
                crate::ui::color::tint(crate::ui::color::mul(t.color, num2 * t.alpha * num6));
            // `:941-945` — the shadow is black with the scaled alpha byte.
            let shadow = [0.0, 0.0, 0.0, num5 / 255.0];
            let colored = color != [0, 0, 0, 0];
            if colored && !fill {
                // `:946-950` — the coloured shadow spreads wider as the
                // popup shrinks (num3 grows).
                num22 *= 1.3 + 1.3 * num21;
                num23 *= 1.3 + 1.3 * num21;
            }
            // `:951-955` — the screen position; the gravDir mirror flips
            // the screen-space Y before the zoom (the C# runs inside the
            // zoom batch). The port's player gravDir is unported (always 1).
            let mut sy = t.position[1] - camera.y + num23;
            if env.grav_dir == -1.0 {
                sy = screen.y - sy;
            }
            let x = (t.position[0] - camera.x + num22) * zoom + screen.x * 0.5;
            let y = sy * zoom + screen.y * 0.5;
            // The C# origin is the text's half-size (`:960`/`:965`), the
            // scale `popup.scale` inside the zoom batch.
            let gx = x + half[0] * zoom - half[0] * t.scale * zoom;
            let gy = y + half[1] * zoom - half[1] * t.scale * zoom;
            let scale = t.scale * zoom;
            if colored && !fill {
                // `:956-962` — the two-pass coloured shadow: the black
                // base lerped halfway to the tint, then the tint itself
                // with the 60..127 alpha ramp.
                let a = 60.0 + (127.0 - 60.0) * (num5 / 255.0).clamp(0.0, 1.0);
                let value4 = [
                    color[0] as f32 / 255.0,
                    color[1] as f32 / 255.0,
                    color[2] as f32 / 255.0,
                    a as u8 as f32 / 255.0,
                ];
                let shadow_rgba = [0, 0, 0, num5 as u8];
                let mid = lerp_half(shadow_rgba, [color[0], color[1], color[2], a as u8]);
                push_line(r, lookups, gx, gy, scale, mid, &t.display_text);
                push_line(r, lookups, gx, gy, scale, value4, &t.display_text);
            } else {
                // `:963-966` — the plain pass (black shadow or the fill).
                let c = if fill { fill_color } else { shadow };
                push_line(r, lookups, gx, gy, scale, c, &t.display_text);
            }
        }
    }
}

/// One `DrawString(FontAssets.MouseText, …)` through the port's ui font
/// (the bitmap fallback mirrors `text::hud_text`).
fn push_line(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    x: f32,
    y: f32,
    scale: f32,
    color: [f32; 4],
    s: &str,
) {
    text::hud_text(r, lookups, x, y, scale, color, s);
}

// --- witnesses (W4-g57) — WRITTEN NOT RUN (code-only swarm, no compiler) ---
#[cfg(test)]
mod tests {
    use super::*;

    fn env() -> Env {
        Env {
            disco: [255, 0, 0],
            master_color: 1.0,
            mouse_text_color: 225,
            global_time_wrapped_hourly: 0.0,
            grav_dir: 1.0,
        }
    }

    fn sizes() -> [[f32; 2]; MAX_POPUPS] {
        [[40.0, 20.0]; MAX_POPUPS]
    }

    /// `PopupText.cs:317-327`: centred on the spawn point, velocity −7,
    /// 60-tick life, scale 0, alpha 1/dir −1, the stack suffix.
    #[test]
    fn the_reforge_spawn_matches_the_csharp_reset() {
        let mut pool = Pool::default();
        let i = spawn_reforge(
            &mut pool,
            false,
            "Legendary Iron Broadsword",
            3,
            [100.0, 200.0],
            popup_rarity_color(0),
            false,
            false,
            [40.0, 20.0],
            800.0,
        )
        .expect("slot");
        let t = &pool.texts[i];
        assert_eq!(t.position, [100.0 - 20.0, 200.0 - 10.0]);
        assert_eq!(t.velocity, [0.0, -7.0]);
        assert_eq!(t.life_time, 60);
        assert_eq!(t.scale, 0.0);
        assert_eq!(t.alpha, 1.0);
        assert_eq!(t.alpha_dir, -1);
        assert!(t.no_stack);
        assert_eq!(t.display_text, "Legendary Iron Broadsword (3)");
        assert_eq!(t.context, Context::ItemReforge);
        // stack 1 carries no suffix (`PrepareDisplayText`, `:417-420`).
        assert_eq!(
            display_text("Keen Iron Broadsword", 1),
            "Keen Iron Broadsword"
        );
    }

    /// `FindNextItemTextSlot` (`PopupText.cs:501-525`): first free slot,
    /// then the topmost active one when all 20 are live.
    #[test]
    fn a_full_pool_replaces_the_topmost_popup() {
        let mut pool = Pool::default();
        for k in 0..MAX_POPUPS {
            let i = spawn_reforge(
                &mut pool,
                false,
                "x",
                1,
                [0.0, 100.0 + k as f32 * 10.0],
                [255; 4],
                false,
                false,
                [10.0, 10.0],
                10_000.0,
            )
            .expect("slot");
            assert_eq!(i, k);
        }
        // All active; the smallest Y is slot 0 (the spawn centring shifts
        // every Y by the same −5, preserving the order).
        let i = spawn_reforge(
            &mut pool,
            true,
            "y",
            1,
            [0.0, 500.0],
            [255; 4],
            false,
            false,
            [10.0, 10.0],
            10_000.0,
        )
        .expect("replacement");
        assert_eq!(i, 0, "the topmost (smallest Y) slot is recycled");
        assert_eq!(pool.texts[0].context, Context::ItemReforgeBest);
    }

    /// `PopupText.cs:652-662`: ±0.01 per tick in [0.7, 1.0].
    #[test]
    fn the_alpha_oscillates_between_0_7_and_1_0() {
        let mut pool = Pool::default();
        spawn_reforge(
            &mut pool,
            false,
            "x",
            1,
            [0.0, 0.0],
            [255; 4],
            false,
            false,
            [10.0, 10.0],
            10_000.0,
        );
        let mut got = Vec::new();
        for _ in 0..60 {
            update_all(&mut pool, 1.0, env(), &sizes());
            got.push(pool.texts[0].alpha);
        }
        // 1.0 → 0.99 … 0.7000003 (tick 30, f32 accumulation stays a hair
        // above the floor), clamped 0.7 at tick 31, back up toward 1.
        assert!((got[0] - 0.99).abs() < 1e-5);
        assert!((got[29] - 0.70).abs() < 1e-4);
        assert_eq!(pool.texts[0].alpha_dir, 1, "reversed at the 0.7 floor");
        assert!(
            (got[59] - 0.99).abs() < 1e-4,
            "tick 60: 0.9899997 (f32 drift)"
        );
    }

    /// `PopupText.cs:698-705`, `:730-740`: ×0.86 while rising, the extra
    /// ×0.4 once the scale hits the target (tick 10 ramps 0.9 → 1.0, so
    /// tick 11 is the first ×0.4 tick), and the 0.1/tick scale ramp.
    #[test]
    fn the_velocity_decays_per_the_csharp_curve() {
        let mut pool = Pool::default();
        spawn_reforge(
            &mut pool,
            false,
            "x",
            1,
            [0.0, 0.0],
            [255; 4],
            false,
            false,
            [10.0, 10.0],
            10_000.0,
        );
        let mut ys = Vec::new();
        for _ in 0..12 {
            update_all(&mut pool, 1.0, env(), &sizes());
            ys.push(pool.texts[0].velocity[1]);
        }
        for (i, y) in ys.iter().enumerate().take(10) {
            let want = -7.0 * 0.86f32.powi(i as i32 + 1);
            assert!((y - want).abs() < 1e-4, "tick {i}: {y} vs {want}");
        }
        assert!(
            (pool.texts[0].scale - 1.0).abs() < 1e-5,
            "ramp done at tick 10"
        );
        let want11 = -7.0 * 0.86f32.powi(11) * 0.4;
        assert!((ys[10] - want11).abs() < 1e-4, "the settled ×0.4 drag");
    }

    /// `PopupText.cs:716-729`: 60 live ticks, then −0.03/tick until the
    /// scale drops below 0.1 (31 shrink ticks from 1.0).
    #[test]
    fn the_popup_lives_60_ticks_then_shrinks_out() {
        let mut pool = Pool::default();
        spawn_reforge(
            &mut pool,
            false,
            "x",
            1,
            [0.0, 0.0],
            [255; 4],
            false,
            false,
            [10.0, 10.0],
            10_000.0,
        );
        for _ in 0..60 {
            update_all(&mut pool, 1.0, env(), &sizes());
        }
        assert!(pool.texts[0].active);
        assert_eq!(pool.texts[0].life_time, 0);
        let mut shrink = 0;
        while pool.texts[0].active {
            update_all(&mut pool, 1.0, env(), &sizes());
            shrink += 1;
            assert!(shrink < 40);
        }
        assert_eq!(shrink, 31, "1.0 − 0.03×31 = 0.07 < 0.1");
    }

    /// `PopupText.cs:678-697`: overlapping popups extend both lifetimes to
    /// `activeTime + 15×min(numActive, 3)` and the upper one accelerates
    /// upward (−0.2/tick while Y > −6).
    #[test]
    fn overlapping_popups_extend_both_lifetimes() {
        let mut pool = Pool::default();
        // Two popups at the same spot: slot 0 is "above" by index order
        // (equal Y, whoAmI < i) when slot 0 updates. Pre-ramp the scales so
        // the hitboxes are non-zero (a fresh spawn's scale-0 hitbox cannot
        // intersect — the vanilla overlaps only once the texts have grown).
        spawn_reforge(
            &mut pool,
            false,
            "a",
            1,
            [100.0, 100.0],
            [255; 4],
            false,
            false,
            [40.0, 20.0],
            10_000.0,
        );
        spawn_reforge(
            &mut pool,
            false,
            "b",
            1,
            [100.0, 100.0],
            [255; 4],
            false,
            false,
            [40.0, 20.0],
            10_000.0,
        );
        pool.texts[0].scale = 1.0;
        pool.texts[1].scale = 1.0;
        pool.num_active = 2; // the previous tick's count (`:689`)
        update_all(&mut pool, 1.0, env(), &sizes());
        // Slot 0's update extends both to 60 + 15×2 = 90; each slot's own
        // update then decrements its life once.
        assert_eq!(pool.texts[0].life_time, 89, "60 + 15×min(2,3) − 1");
        assert_eq!(pool.texts[1].life_time, 89);
        // Slot 0 took the overlap arm (`:706-713`): vy −7 is NOT > −6, so
        // the else ×0.86 fires (the −0.2 push only starts once vy > −6).
        assert!((pool.texts[0].velocity[1] - (-7.0 * 0.86)).abs() < 1e-4);
    }

    /// `Item.cs:652-707` — the popup rarity table (NOT the tooltip table).
    #[test]
    fn the_popup_rarity_table_matches_the_csharp() {
        assert_eq!(popup_rarity_color(0), [255, 255, 255, 255]);
        assert_eq!(popup_rarity_color(1), [150, 150, 255, 255]);
        assert_eq!(popup_rarity_color(2), [150, 255, 150, 255]);
        assert_eq!(popup_rarity_color(3), [255, 200, 150, 255]);
        assert_eq!(popup_rarity_color(4), [255, 150, 150, 255]);
        assert_eq!(popup_rarity_color(5), [255, 150, 255, 255]);
        assert_eq!(popup_rarity_color(-11), [255, 175, 0, 255]);
        assert_eq!(popup_rarity_color(-1), [130, 130, 130, 255]);
        assert_eq!(popup_rarity_color(6), [210, 160, 255, 255]);
        assert_eq!(popup_rarity_color(7), [150, 255, 10, 255]);
        assert_eq!(popup_rarity_color(8), [255, 255, 10, 255]);
        assert_eq!(popup_rarity_color(9), [5, 200, 255, 255]);
        assert_eq!(popup_rarity_color(10), [255, 40, 100, 255]);
        assert_eq!(popup_rarity_color(11), [180, 40, 255, 255]);
        assert_eq!(popup_rarity_color(15), [180, 40, 255, 255]);
        // -13 (master) falls to White here; the master pulse recolours in
        // Update (`PopupText.cs:667-670`).
        assert_eq!(popup_rarity_color(-13), [255, 255, 255, 255]);
    }

    /// `PopupText.cs:821` + `Main.hslToRgb` (`Main.cs:47890-47917`):
    /// hue 0 at l=0.6 is (255, 51, 51); ×0.6 truncates to (153, 30, 30),
    /// alpha 255×0.6 = 153.
    #[test]
    fn the_best_tier_colour_cycles_the_rainbow() {
        assert_eq!(best_popup_color(0.0), [153, 30, 30, 153]);
        // The hue wraps: t and t + 1/0.6 land on the same colour.
        let a = best_popup_color(0.25);
        let b = best_popup_color(0.25 + 1.0 / 0.6);
        assert_eq!(a, b, "GlobalTimeWrappedHourly × 0.6 % 1 wraps");
        assert_ne!(a, best_popup_color(0.0), "the hue actually moves");
    }

    /// `PopupText.cs:663-670`: the expert disco / master pulse recolour.
    #[test]
    fn expert_and_master_popups_recolour_every_tick() {
        let mut pool = Pool::default();
        spawn_reforge(
            &mut pool,
            false,
            "x",
            1,
            [0.0, 0.0],
            [255; 4],
            true,
            false,
            [10.0, 10.0],
            10_000.0,
        );
        let mut e = env();
        e.disco = [10, 20, 30];
        e.master_color = 0.5;
        update_all(&mut pool, 1.0, e, &sizes());
        assert_eq!(
            pool.texts[0].color,
            [255, 100, 0, 225],
            "master: (byte)(0.5×200)"
        );
        pool.texts[0].master = false;
        pool.texts[0].expert = true;
        update_all(&mut pool, 1.0, e, &sizes());
        assert_eq!(pool.texts[0].color, [10, 20, 30, 225], "expert disco");
    }
}
