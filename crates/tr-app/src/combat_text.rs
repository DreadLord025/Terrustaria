//! `Terraria.CombatText` (`CombatText.cs:1-232`) — the floating damage numbers
//! and their `Draw` half (`Main.cs:62340-62396`).
//!
//! The spawn and update rules are pure functions so they test without a GPU:
//! [`spawn`] is `CombatText.NewText` (`CombatText.cs:59-121`), [`update`] is
//! `CombatText.Update` (`:131-206`), and the palette is the static ctor
//! (`:219-231`). The draw helpers at the bottom mirror the vanilla passes.

use tr_core::UnifiedRandom;

/// `for (int i = 0; i < 100; i++)` — the pool size (`CombatText.cs:65`,
/// `:125`, `:210`).
pub const MAX_TEXTS: usize = 100;
/// `CombatText.TargetScale => 1f` (`CombatText.cs:52`).
pub const TARGET_SCALE: f32 = 1.0;
/// `Main.combatText[i].lifeTime = 60` (`CombatText.cs:98`).
pub const LIFE_TICKS: i32 = 60;
/// `lifeTime *= 2` (`CombatText.cs:104`).
pub const CRIT_LIFE_TICKS: i32 = LIFE_TICKS * 2;
/// `lifeTime = 40` (`CombatText.cs:116`).
pub const DOT_LIFE_TICKS: i32 = 40;

/// `CombatText.DamagedHostile` — `new Color(255, 160, 80, 255)`
/// (`CombatText.cs:223`).
pub const DAMAGED_HOSTILE: [u8; 4] = [255, 160, 80, 255];
/// `CombatText.DamagedHostileCrit` — `new Color(255, 100, 30, 255)`
/// (`CombatText.cs:224`).
pub const DAMAGED_HOSTILE_CRIT: [u8; 4] = [255, 100, 30, 255];
/// `CombatText.DamagedFriendly` — `new Color(255, 80, 90, 255)`
/// (`CombatText.cs:221`).
pub const DAMAGED_FRIENDLY: [u8; 4] = [255, 80, 90, 255];
/// `CombatText.DamagedFriendlyCrit` — `new Color(255, 100, 30, 255)`
/// (`CombatText.cs:222`).
pub const DAMAGED_FRIENDLY_CRIT: [u8; 4] = [255, 100, 30, 255];
/// `CombatText.OthersDamagedHostile = DamagedHostile * 0.4f`
/// (`CombatText.cs:225`).
pub const OTHERS_DAMAGED_HOSTILE: [u8; 4] = [102, 64, 32, 102];
/// `CombatText.OthersDamagedHostileCrit = DamagedHostileCrit * 0.4f`
/// (`CombatText.cs:226`).
pub const OTHERS_DAMAGED_HOSTILE_CRIT: [u8; 4] = [102, 40, 12, 102];
/// `CombatText.HealMana` — `new Color(100, 100, 255, 255)`
/// (`CombatText.cs:228`); `Player.ManaEffect` spawns with it
/// (`Player.cs:4533`).
pub const HEAL_MANA: [u8; 4] = [100, 100, 255, 255];
/// `CombatText.HealLife` — `new Color(100, 255, 100, 255)`
/// (`CombatText.cs:227`); `Player.HealEffect` spawns with it
/// (`Player.cs:4524`, S1-P-i37's potion heals + S1-P-i52's heart pickup text
/// + S1-P-i54's QuickHeal).
pub const HEAL_LIFE: [u8; 4] = [100, 255, 100, 255];

/// The bar sheets decode at their retail 36x12 size
/// (`TextureAssets.Hb1`/`Hb2` = `Images/HealthBar1|2`, `AssetInitializer.cs:404-405`).
pub const BAR_WIDTH: f32 = 36.0;
/// `TextureAssets.Hb1.Height()` (`Main.cs:21988`).
pub const BAR_HEIGHT: f32 = 12.0;
/// `float num3 = 10f` (`Main.cs:45913`) — the gap under the hitbox for
/// `HealthBarDrawSettings == 1` (`Main.cs:46021-46024`).
pub const BAR_GAP: f32 = 10.0;

/// `Rectangle` as the C# passes it (`location` — the NPC hitbox).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

/// `CombatText` (`CombatText.cs:28-50`).
#[derive(Clone, Debug, PartialEq)]
pub struct CombatText {
    pub position: [f32; 2],
    pub velocity: [f32; 2],
    pub alpha: f32,
    pub alpha_dir: i32,
    pub text: String,
    pub scale: f32,
    pub rotation: f32,
    pub color: [u8; 4],
    pub active: bool,
    pub lifetime: i32,
    pub crit: bool,
    pub dot: bool,
}

impl Default for CombatText {
    fn default() -> Self {
        Self {
            position: [0.0; 2],
            velocity: [0.0; 2],
            alpha: 1.0,
            alpha_dir: 1,
            text: String::new(),
            scale: 1.0,
            rotation: 0.0,
            color: [255; 4],
            active: false,
            lifetime: 0,
            crit: false,
            dot: false,
        }
    }
}

/// `CombatText.NewText(Rectangle, Color, int, bool, bool)`
/// (`CombatText.cs:54-57`).
pub fn spawn_amount(
    pool: &mut Vec<CombatText>,
    location: Rect,
    color: [u8; 4],
    amount: i32,
    dramatic: bool,
    dot: bool,
    text_size: [f32; 2],
    grav_dir: f32,
    rng: &mut UnifiedRandom,
) -> Option<usize> {
    spawn(
        pool,
        location,
        color,
        amount.to_string(),
        dramatic,
        dot,
        text_size,
        grav_dir,
        rng,
    )
}

/// `CombatText.NewText(Rectangle, Color, string, bool, bool)`
/// (`CombatText.cs:59-121`).
pub fn spawn(
    pool: &mut Vec<CombatText>,
    location: Rect,
    color: [u8; 4],
    text: String,
    dramatic: bool,
    dot: bool,
    text_size: [f32; 2],
    grav_dir: f32,
    rng: &mut UnifiedRandom,
) -> Option<usize> {
    // `for (int i = 0; i < 100; i++) { if (Main.combatText[i].active) continue;`
    // (`CombatText.cs:65-70`) — the first free slot, `100` when full
    // (`:120`).
    let mut slot = None;
    for i in 0..MAX_TEXTS {
        match pool.get(i) {
            Some(t) if t.active => continue,
            _ => {
                slot = Some(i);
                break;
            }
        }
    }
    let i = slot?;
    if pool.len() <= i {
        pool.resize_with(i + 1, CombatText::default);
    }
    let t = &mut pool[i];
    // `MeasureString` (`CombatText.cs:76`): the caller passes the measured
    // size (the app measures with the combat font it will draw with).
    let vector = text_size;
    t.alpha = 1.0; // `CombatText.cs:81`
    t.alpha_dir = -1; // `:82`
    t.active = true; // `:83`
    t.scale = 0.0; // `:84`
    t.rotation = 0.0; // `:85`
    t.position[0] = location.x + location.width * 0.5 - vector[0] * 0.5; // `:86`
    t.position[1] = location.y + location.height * 0.25 - vector[1] * 0.5; // `:87`
    // `Main.rand.Next(-(int)(Width * 0.5), (int)(Width * 0.5) + 1)` (`:88-89`).
    t.position[0] += rng.next_range(
        -((location.width * 0.5) as i32),
        (location.width * 0.5) as i32 + 1,
    ) as f32;
    t.position[1] += rng.next_range(
        -((location.height * 0.5) as i32),
        (location.height * 0.5) as i32 + 1,
    ) as f32;
    t.color = color; // `:90`
    t.text = text; // `:91`
    t.velocity = [0.0, -7.0]; // `:92`
    if grav_dir == -1.0 {
        // `:93-97` — the flipped-gravity mirror.
        t.velocity[1] *= -1.0;
        t.position[1] = location.y + location.height * 0.75 + vector[1] * 0.5;
    }
    t.lifetime = LIFE_TICKS; // `:98`
    t.crit = dramatic; // `:99`
    t.dot = dot; // `:100`
    if dramatic {
        // `:101-112`.
        t.lifetime *= 2;
        t.velocity[1] *= 2.0;
        t.velocity[0] = rng.next_range(-25, 26) as f32 * 0.05;
        t.rotation = (t.lifetime / 2) as f32 * 0.002;
        if t.velocity[0] < 0.0 {
            t.rotation *= -1.0;
        }
    }
    if dot {
        // `:113-117`.
        t.velocity[1] = -4.0;
        t.lifetime = DOT_LIFE_TICKS;
    }
    Some(i)
}

/// `CombatText.Update` (`CombatText.cs:131-206`).
pub fn update(t: &mut CombatText) {
    if !t.active {
        return; // `:133-136`
    }
    let target_scale = TARGET_SCALE;
    // `:138-147` — the ±0.05 alpha oscillation in [0.6, 1.0].
    t.alpha += t.alpha_dir as f32 * 0.05;
    if t.alpha <= 0.6 {
        t.alpha_dir = 1;
    }
    if t.alpha >= 1.0 {
        t.alpha = 1.0;
        t.alpha_dir = -1;
    }
    // `:148-159` — DoT rises; everything else decelerates (crit twice).
    if t.dot {
        t.velocity[1] += 0.15;
    } else {
        t.velocity[1] *= 0.92;
        if t.crit {
            t.velocity[1] *= 0.92;
        }
    }
    t.velocity[0] *= 0.93; // `:160`
    t.position[0] += t.velocity[0]; // `:161`
    t.position[1] += t.velocity[1];
    t.lifetime -= 1; // `:162`
    if t.lifetime <= 0 {
        // `:163-177` — the shrink-out, with the crit's +0.07 pull-back.
        t.scale -= 0.1 * target_scale;
        if t.scale < 0.1 {
            t.active = false;
        }
        t.lifetime = 0;
        if t.crit {
            t.alpha_dir = -1;
            t.scale += 0.07 * target_scale;
        }
        return;
    }
    if t.crit {
        // `:178-188` — the dramatic spin.
        if t.velocity[0] < 0.0 {
            t.rotation += 0.001;
        } else {
            t.rotation -= 0.001;
        }
    }
    if t.dot {
        // `:189-197`.
        t.scale += 0.5 * target_scale;
        if t.scale > 0.8 * target_scale {
            t.scale = 0.8 * target_scale;
        }
        return;
    }
    if t.scale < target_scale {
        t.scale += 0.1 * target_scale;
    }
    if t.scale > target_scale {
        t.scale = target_scale;
    }
}

/// `CombatText.UpdateCombatText` (`CombatText.cs:208-217`).
pub fn update_all(pool: &mut [CombatText]) {
    for t in pool.iter_mut().take(MAX_TEXTS) {
        if t.active {
            update(t);
        }
    }
}

/// `CombatText.clearAll` (`CombatText.cs:123-129`).
pub fn clear_all(pool: &mut [CombatText]) {
    for t in pool.iter_mut().take(MAX_TEXTS) {
        t.active = false;
    }
}

/// The colour rule of `NPC.StrikeNPC_Inner` (`NPC.cs:81252-81263`): friendly
/// NPCs use the friendly pair (`:81254`), hostile ones the hostile pair
/// (`:81258`), and a network-received strike dims to `Others…` at 0.4
/// (`:81261`).
pub fn damage_text_color(crit: bool, friendly: bool, from_net: bool) -> [u8; 4] {
    if friendly {
        // `crit ? CombatText.DamagedFriendlyCrit : CombatText.DamagedFriendly`
        // (`NPC.cs:81254`).
        return if crit { DAMAGED_FRIENDLY_CRIT } else { DAMAGED_FRIENDLY };
    }
    if from_net {
        // `crit ? OthersDamagedHostileCrit : OthersDamagedHostile`
        // (`NPC.cs:81261`).
        return if crit { OTHERS_DAMAGED_HOSTILE_CRIT } else { OTHERS_DAMAGED_HOSTILE };
    }
    // `crit ? DamagedHostileCrit : DamagedHostile` (`NPC.cs:81258`).
    if crit { DAMAGED_HOSTILE_CRIT } else { DAMAGED_HOSTILE }
}

/// `Main.DrawHealthBar`'s fill geometry (`Main.cs:21915-21921`, `:21972-21975`):
/// width `(int)(36f * ratio)` clamped to at least 3, height
/// `TextureAssets.Hb1.Height()`, anchored `X - 18 * scale` under the hitbox
/// (`Main.cs:46024`).
pub fn bar_rect(hitbox: Rect, ratio: f32) -> Rect {
    let ratio = ratio.clamp(0.0, 1.0); // `Main.cs:21916-21919`
    // `int num2 = (int)(36f * num)` (`Main.cs:21920`) — the truncating cast.
    let mut width = (BAR_WIDTH * ratio) as i32;
    // `if (num2 < 3) num2 = 3` (`Main.cs:21972-21975`).
    if width < 3 {
        width = 3;
    }
    Rect {
        // `float num3 = X - 18f * scale` (`Main.cs:21921`), `X` is the box
        // centre-X and `Y` its bottom + 10 (`Main.cs:46024`).
        x: hitbox.x + hitbox.width * 0.5 - 18.0,
        y: hitbox.y + hitbox.height + BAR_GAP,
        width: width as f32,
        height: BAR_HEIGHT,
    }
}

/// The `life != lifeMax && ShowLife` gate (`Main.cs:45835`) with the
/// `Health <= 0` early return (`Main.cs:21911-21914`). `ShowLife` defaults to
/// true (`NPC.cs:6832-6842`; the pass-level gate lives in
/// `render::entities::npc_shows_health_bar`).
pub fn health_bar_rect(hitbox: Rect, life: i32, life_max: i32) -> Option<Rect> {
    // `if (Health <= 0) return;` (`Main.cs:21911-21914`) and
    // `life != lifeMax` (`Main.cs:45835`).
    if life <= 0 || life_max <= 0 || life >= life_max {
        return None;
    }
    Some(bar_rect(hitbox, life as f32 / life_max as f32))
}

/// One `Main.DrawHealthBar` slice (`Main.cs:21976-21998`), in the C# draw
/// order: `(hb1, src_x, dst_x, width)` — `hb1` selects the filled `Hb1`
/// sheet over the depleted `Hb2`, `src_x` is the source-rect X inside the
/// 36x12 sheet, `dst_x` the offset from the bar's left edge in unscaled
/// pixels, `width` the slice width. `filled` is `num2`, already floored at
/// 3 (`Main.cs:21972-21975`).
///
/// Below 34 the C# keeps a 2 px seam: `Hb2`'s columns 2..4 ride the fill
/// edge and `Hb1`'s columns 32..34 cap it (`:21980-21990`); at 34+ the two
/// sheets simply butt (`:21994-21998`).
pub fn bar_slices(filled: i32) -> Vec<(bool, i32, i32, u32)> {
    let n = filled.max(3); // `Main.cs:21972-21975`
    if n < 34 {
        // `:21976-21990` — four slices (`n >= 3` makes `num2 < 36` and
        // `num2 > 2` always true here).
        vec![
            (false, 2, n, 2),                    // `:21980` Hb2 (2,0,2,H)
            (false, n + 2, n + 2, (36 - n - 2) as u32), // `:21984`
            (true, 0, 0, (n - 2) as u32),        // `:21988` Hb1 (0,0,n-2,H)
            (true, 32, n - 2, 2),                // `:21990` Hb1 (32,0,2,H)
        ]
    } else {
        // `:21992-21998` — the depleted strip only while `num2 < 36`.
        let mut v = Vec::with_capacity(2);
        if n < 36 {
            v.push((false, n, n, (36 - n) as u32)); // `:21996`
        }
        v.push((true, 0, 0, n as u32)); // `:21998`
        v
    }
}

/// The green-to-red fill colour (`Main.cs:21932-21942`), before the
/// `alpha * 0.95f` light multiply (`:21943-21944`).
pub fn bar_color(ratio: f32) -> [u8; 3] {
    let mut num = ratio.clamp(0.0, 1.0); // `Main.cs:21915-21919`
    num -= 0.1; // `Main.cs:21932`
    let (r, g) = if num > 0.5 {
        // `:21933-21937` — green with the red channel ramping down.
        (255.0 * (1.0 - num) * 2.0, 255.0)
    } else {
        // `:21938-21942` — red with the green channel ramping down.
        (255.0, 255.0 * num * 2.0)
    };
    // `:21947-21969` clamps each channel into [0, 255] before the byte cast.
    [r.clamp(0.0, 255.0) as u8, g.clamp(0.0, 255.0) as u8, 0]
}


// --- Draw -------------------------------------------------------------------

use crate::assets;
use crate::ui::text::push_font_text;
use glam::Vec2;
use tr_render::SpriteRenderer;

/// `Main.DoDraw`'s combat-text pass (`Main.cs:62340-62396`): the four 1 px
/// shadow passes (`:62360-62384`, colour × `scale·alpha·0.3`) then the fill
/// (`:62381-62382`), drawn with `FontAssets.CombatText[crit ? 1 : 0]`
/// (`:62348-62352`). The port draws in screen space (as the C# does after
/// `position - Main.screenPosition`) and maps the world position through the
/// render camera; the C# rotation (`:62389`) has no glyph-rotation primitive
/// in this renderer and stays unapplied.
pub fn draw(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    texts: &[CombatText],
    camera: Vec2,
    zoom: f32,
    screen: Vec2,
) {
    let target = TARGET_SCALE;
    for t in texts.iter().take(MAX_TEXTS) {
        if !t.active {
            continue;
        }
        // `int num14 = 0; if (crit) num14 = 1;` (`Main.cs:62348-62352`).
        let font = if t.crit {
            lookups.combat_crit_font.as_ref()
        } else {
            lookups.combat_text_font.as_ref()
        };
        let Some(font) = font else { continue };
        // `MeasureString` (`Main.cs:62353`) fed only the spawn placement; the
        // draw reads the stored position.
        // `float num15 = scale / targetScale` (`Main.cs:62355`).
        let num15 = t.scale / target;
        // Shadow colour: `num16 *= num15 * alpha * 0.3f` with the alpha byte
        // scaled by `num15 * alpha` (`Main.cs:62360-62361`).
        let k = num15 * t.alpha;
        let shadow = [
            t.color[0] as f32 / 255.0 * k * 0.3,
            t.color[1] as f32 / 255.0 * k * 0.3,
            t.color[2] as f32 / 255.0 * k * 0.3,
            t.color[3] as f32 / 255.0 * k,
        ];
        // Fill colour (`Main.cs:62381-62382`).
        let fill = [
            t.color[0] as f32 / 255.0 * k,
            t.color[1] as f32 / 255.0 * k,
            t.color[2] as f32 / 255.0 * k,
            t.color[3] as f32 / 255.0 * k,
        ];
        // `position - Main.screenPosition` (`Main.cs:62389`), through the
        // port's camera transform.
        let base = [
            (t.position[0] - camera.x) * zoom + screen.x * 0.5,
            (t.position[1] - camera.y) * zoom + screen.y * 0.5,
        ];
        // `for (int num20 = 0; num20 < 5; num20++)` (`Main.cs:62362`): four
        // ±targetScale offsets (left/right/up/down) then the fill pass.
        for pass in 0..5 {
            let (dx, dy) = match pass {
                0 => (-target, 0.0),
                1 => (target, 0.0),
                2 => (0.0, -target),
                3 => (0.0, target),
                _ => (0.0, 0.0),
            };
            let color = if pass == 4 { fill } else { shadow };
            push_font_text(r, font, base[0] + dx, base[1] + dy, t.scale, color, &t.text);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tr_core::damage::strike_npc_damage;

    fn rng(seed: i32) -> UnifiedRandom {
        UnifiedRandom::new(seed)
    }

    /// `CombatText.cs:101-112`: `lifeTime *= 2`, `velocity.Y *= 2f`,
    /// `velocity.X = Main.rand.Next(-25, 26) * 0.05f`.
    #[test]
    fn a_crit_text_spawns_with_double_lifetime_and_dramatic_velocity() {
        let mut pool = Vec::new();
        let mut r = rng(12345);
        let loc = Rect { x: 100.0, y: 100.0, width: 32.0, height: 48.0 };
        let i = spawn(
            &mut pool,
            loc,
            DAMAGED_HOSTILE_CRIT,
            "48".into(),
            true,
            false,
            [12.0, 10.0],
            1.0,
            &mut r,
        )
        .expect("spawn");
        let t = &pool[i];
        assert_eq!(t.lifetime, 120, "lifeTime *= 2 (CombatText.cs:104)");
        assert!(t.crit);
        assert_eq!(t.velocity[1], -14.0, "velocity.Y = -7 then *2 (:92,:105)");
        // Pinned draw for seed 12345 — the three Main.rand.Next calls of the
        // spawn path (:88, :89, :106) run through the C# Knuth generator.
        assert_eq!(t.velocity[0], CRIT_VX_SEED_12345);
        assert_eq!(t.rotation, 60.0 * 0.002, "rotation = lifeTime/2 * 0.002 (:107)");
    }

    // Main.rand.Next(-25, 26) = 14 on seed 12345 after the two position
    // jitter draws (:88, :89) — 14 * 0.05f (:106).
    const CRIT_VX_SEED_12345: f32 = 0.7;

    /// `CombatText.cs:221-226` / `NPC.cs:81254,81258`.
    #[test]
    fn a_hostile_text_uses_the_damaged_hostile_palette() {
        assert_eq!(damage_text_color(false, false, false), [255, 160, 80, 255]);
        assert_eq!(damage_text_color(true, false, false), [255, 100, 30, 255]);
        assert_eq!(damage_text_color(false, true, false), [255, 80, 90, 255]);
        assert_eq!(damage_text_color(true, true, false), [255, 100, 30, 255]);
        assert_eq!(damage_text_color(false, false, true), OTHERS_DAMAGED_HOSTILE);
    }

    /// `CombatText.cs:138-147`: ±0.05 per tick in `[0.6, 1.0]`, reversing at
    /// both ends. The first 20 alphas are pinned.
    #[test]
    fn text_alpha_oscillates_between_0_6_and_1_0_in_0_05_steps() {
        let mut t = CombatText {
            active: true,
            lifetime: 1000,
            alpha: 1.0,
            alpha_dir: -1,
            ..Default::default()
        };
        let mut got = Vec::new();
        for _ in 0..20 {
            update(&mut t);
            got.push(t.alpha);
        }
        let want: Vec<f32> = vec![
            0.95, 0.90, 0.85, 0.80, 0.75, 0.70, 0.65, 0.60, 0.65, 0.70, 0.75, 0.80, 0.85, 0.90,
            0.95, 1.00, 0.95, 0.90, 0.85, 0.80,
        ];
        for (i, (g, w)) in got.iter().zip(want.iter()).enumerate() {
            assert!((g - w).abs() < 1e-5, "tick {i}: got {g}, want {w}");
        }
        assert_eq!(t.alpha_dir, -1);
    }

    /// `CombatText.cs:152-160`: `Y *= 0.92` (twice for crit), `X *= 0.93`,
    /// then `position += velocity`.
    #[test]
    fn text_velocity_decays_per_the_csharp_curve() {
        let mut t = CombatText {
            active: true,
            lifetime: 1000,
            velocity: [1.0, -7.0],
            ..Default::default()
        };
        let mut ys = Vec::new();
        for _ in 0..5 {
            update(&mut t);
            ys.push(t.velocity[1]);
        }
        let want = [-6.44f32, -5.9248, -5.450816, -5.014_750_7, -4.613_570_7];
        for (i, (g, w)) in ys.iter().zip(want.iter()).enumerate() {
            assert!((g - w).abs() < 1e-5, "tick {i}: got {g}, want {w}");
        }
        // `velocity.X *= 0.93f` once per tick (`CombatText.cs:160`).
        let vx_want = 0.93f32.powi(5);
        assert!((t.velocity[0] - vx_want).abs() < 1e-6);
    }

    /// `CombatText.cs:98`, `:162-176`, `:198-205`: 60 ticks alive, the scale
    /// ramp 0.1 -> 1.0, then the shrink `scale -= 0.1` and deactivation.
    #[test]
    fn a_non_crit_text_lives_60_ticks_and_dies_with_the_scale_shrink() {
        let mut t = CombatText {
            active: true,
            lifetime: LIFE_TICKS,
            scale: 0.0,
            ..Default::default()
        };
        for _ in 0..10 {
            update(&mut t);
        }
        assert!((t.scale - 1.0).abs() < 1e-5, "ramp 0.1/tick to 1.0 (:198-200)");
        assert_eq!(t.lifetime, LIFE_TICKS - 10);
        for _ in 0..(LIFE_TICKS - 10) {
            update(&mut t);
        }
        assert_eq!(t.lifetime, 0, "lifeTime hits 0 after 60 ticks (:162-163)");
        assert!(t.active, "the shrink starts, the text is not gone yet");
        let mut shrink = 0;
        while t.active {
            update(&mut t);
            shrink += 1;
            assert!(shrink < 20);
        }
        // 1.0 -> 0.9 -> ... -> below 0.1 (`:165-169`), float-accumulation safe.
        assert!((8..=10).contains(&shrink), "shrink ticks: {shrink}");
    }

    /// `NPC.cs:81239` — the number drawn is the post-defence strike value:
    /// `25` vs `defense 1` -> `24`, crit -> `48`.
    #[test]
    fn the_damage_shown_is_the_post_defense_value() {
        let dmg = strike_npc_damage(25, 1, false);
        assert_eq!(dmg, 24);
        let mut pool = Vec::new();
        let mut r = rng(7);
        let loc = Rect { x: 0.0, y: 0.0, width: 32.0, height: 48.0 };
        spawn_amount(&mut pool, loc, DAMAGED_HOSTILE, dmg, false, false, [12.0, 10.0], 1.0, &mut r);
        assert_eq!(pool[0].text, "24");
        assert!(!pool[0].crit);
        // `CalculateDamageNPCsTake` is 24.5 (double) and the crit doubles
        // BEFORE the `(int)` cast (`NPC.cs:81239`), so 24.5 * 2 = 49.
        let crit = strike_npc_damage(25, 1, true);
        assert_eq!(crit, 49);
        spawn_amount(&mut pool, loc, DAMAGED_HOSTILE_CRIT, crit, true, false, [12.0, 10.0], 1.0, &mut r);
        assert_eq!(pool[1].text, "49");
        assert!(pool[1].crit);
    }

    /// `Main.cs:21915-21921`, `:21972-21975`, `:46024`: width
    /// `(int)(36 * ratio)` floored at 3, height 12, `X - 18`, under the box.
    #[test]
    fn an_ordinary_mob_health_bar_rect_matches_the_csharp_geometry() {
        let box_ = Rect { x: 100.0, y: 200.0, width: 32.0, height: 48.0 };
        let r = bar_rect(box_, 0.5);
        assert_eq!(r, Rect { x: 100.0 + 16.0 - 18.0, y: 200.0 + 48.0 + 10.0, width: 18.0, height: 12.0 });
        // A sliver still draws the 3 px minimum (`Main.cs:21972-21975`).
        assert_eq!(bar_rect(box_, 0.01).width, 3.0);
        assert_eq!(bar_rect(box_, 0.0).width, 3.0);
        assert_eq!(bar_rect(box_, 1.0).width, 36.0);
    }

    /// `Main.cs:21932-21942`: green at full, red near empty, with the
    /// `num -= 0.1f` head start.
    #[test]
    fn the_bar_color_goes_green_to_red_per_the_ratio() {
        // ratio 1.0 -> num 0.9 -> R = 255*(1-0.9)*2 = 51, G = 255.
        assert_eq!(bar_color(1.0), [51, 255, 0]);
        // ratio 0.5 -> num 0.4 -> R = 255, G = 255*0.4*2 = 204.
        assert_eq!(bar_color(0.5), [255, 204, 0]);
        // ratio 0.1 -> num 0.0 -> R = 255, G = 0.
        assert_eq!(bar_color(0.1), [255, 0, 0]);
    }

    /// `Main.cs:45835` / `:21911-21914`: a full-health mob draws nothing.
    #[test]
    fn full_health_mobs_draw_no_bar() {
        let box_ = Rect { x: 0.0, y: 0.0, width: 32.0, height: 48.0 };
        assert_eq!(health_bar_rect(box_, 100, 100), None);
        assert_eq!(health_bar_rect(box_, 101, 100), None);
        assert_eq!(health_bar_rect(box_, 0, 100), None);
        assert!(health_bar_rect(box_, 50, 100).is_some());
    }

    /// `Main.cs:21976-21998`: below 34 filled pixels the four-slice seam
    /// layout, at 34/35 the two-sheet butt with the depleted remainder, at
    /// 36 the single full Hb1 strip. `(hb1, src_x, dst_x, width)` in the C#
    /// draw order.
    #[test]
    fn the_bar_slices_match_the_csharp_seam_layout() {
        // num2 = 18 (ratio 0.5): Hb2 (2,0,2) at 18, Hb2 (20,0,16) at 20,
        // Hb1 (0,0,16) at 0, Hb1 (32,0,2) at 16 (`:21980-21990`).
        assert_eq!(
            bar_slices(18),
            vec![
                (false, 2, 18, 2),
                (false, 20, 20, 16),
                (true, 0, 0, 16),
                (true, 32, 16, 2),
            ]
        );
        // The 3 px minimum still keeps the seam (`:21972-21975`):
        // Hb1's body is 1 px, the cap rides at dst 1.
        assert_eq!(
            bar_slices(3),
            vec![
                (false, 2, 3, 2),
                (false, 5, 5, 31),
                (true, 0, 0, 1),
                (true, 32, 1, 2),
            ]
        );
        // num2 = 34: the else arm — Hb2 (34,0,2) at 34, Hb1 (0,0,34)
        // (`:21994-21998`).
        assert_eq!(bar_slices(34), vec![(false, 34, 34, 2), (true, 0, 0, 34)]);
        // num2 = 35: one depleted pixel.
        assert_eq!(bar_slices(35), vec![(false, 35, 35, 1), (true, 0, 0, 35)]);
        // num2 = 36: the full strip, no Hb2 (`:21994`'s `num2 < 36` fails).
        assert_eq!(bar_slices(36), vec![(true, 0, 0, 36)]);
        // Every slice set tiles [0, 36) exactly once — no gap, no overlap.
        for n in 3..=36 {
            let mut covered = [false; 36];
            for &(_, _, dst, width) in &bar_slices(n) {
                for px in dst..dst + width as i32 {
                    assert!(!covered[px as usize], "num2 {n}: px {px} painted twice");
                    covered[px as usize] = true;
                }
            }
            assert!(covered.iter().all(|&c| c), "num2 {n}: a gap in the bar");
        }
    }
}
