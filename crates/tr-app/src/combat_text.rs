//! `CombatText` (`CombatText.cs`) — the floating damage numbers.
//!
//! The C# keeps 100 slots on `Main.combatText` (`Main.cs:1631`, `:66993`),
//! fills a free slot in `CombatText.NewText` (`:59-121`), updates them in
//! `CombatText.UpdateCombatText` (`:208-217`, called from `Main.cs:18077`)
//! and draws them in the UI batch with the world zoom
//! (`Main.cs:62341-62396`) — five passes per number (the four shadows plus
//! the bright text).
//!
//! The port keeps the same 100-slot pool, the same per-slot state and the
//! same update maths; the position jitter and the crit's horizontal kick use
//! the stage's own RNG stream (the C# draws them from `Main.rand`, which is
//! the running stage's stream — `Main.SwapRandom`, `Main.cs:3204-3215`).

use glam::Vec2;
use tr_core::unified_random::UnifiedRandom;

/// `Main.maxCombatText` (`Main.cs:1027`): the pool's slot count.
pub(crate) const MAX_COMBAT_TEXT: usize = 100;

/// The C#'s palette (`CombatText.cs:219-231`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CombatColor {
    DamagedFriendly,
    DamagedFriendlyCrit,
    DamagedHostile,
    DamagedHostileCrit,
    OthersDamagedHostile,
    OthersDamagedHostileCrit,
    HealLife,
    HealMana,
    LifeRegen,
    LifeRegenNegative,
}

impl CombatColor {
    /// The RGB bytes the C#'s static colours carry.
    pub(crate) fn rgb(self) -> [u8; 4] {
        match self {
            // `DamagedFriendly = new Color(255, 80, 90, 255)`.
            CombatColor::DamagedFriendly => [255, 80, 90, 255],
            // `DamagedFriendlyCrit = new Color(255, 100, 30, 255)`.
            CombatColor::DamagedFriendlyCrit => [255, 100, 30, 255],
            // `DamagedHostile = new Color(255, 160, 80, 255)`.
            CombatColor::DamagedHostile => [255, 160, 80, 255],
            // `DamagedHostileCrit = new Color(255, 100, 30, 255)`.
            CombatColor::DamagedHostileCrit => [255, 100, 30, 255],
            // `OthersDamagedHostile = DamagedHostile * 0.4f` — the byte
            // multiply of the XNA colour (255, 160, 80) by 0.4.
            CombatColor::OthersDamagedHostile => [102, 64, 32, 255],
            // `OthersDamagedHostileCrit = DamagedHostileCrit * 0.4f`.
            CombatColor::OthersDamagedHostileCrit => [102, 40, 12, 255],
            // `HealLife = new Color(100, 255, 100, 255)`.
            CombatColor::HealLife => [100, 255, 100, 255],
            // `HealMana = new Color(100, 100, 255, 255)`.
            CombatColor::HealMana => [100, 100, 255, 255],
            // `LifeRegen = new Color(255, 60, 70, 255)`.
            CombatColor::LifeRegen => [255, 60, 70, 255],
            // `LifeRegenNegative = new Color(255, 140, 40, 255)`.
            CombatColor::LifeRegenNegative => [255, 140, 40, 255],
        }
    }
}

/// One floating number (`CombatText.cs:28-50`).
#[derive(Clone, Debug)]
pub(crate) struct CombatText {
    pub(crate) position: Vec2,
    pub(crate) velocity: Vec2,
    pub(crate) alpha: f32,
    /// `alphaDir` — `-1` fades the text in, `1` fades it out.
    pub(crate) alpha_dir: i32,
    pub(crate) text: String,
    pub(crate) scale: f32,
    pub(crate) rotation: f32,
    pub(crate) color: [u8; 4],
    pub(crate) active: bool,
    pub(crate) life_time: i32,
    pub(crate) crit: bool,
    pub(crate) dot: bool,
}

impl Default for CombatText {
    fn default() -> Self {
        Self {
            position: Vec2::ZERO,
            velocity: Vec2::ZERO,
            alpha: 1.0,
            alpha_dir: 1,
            text: String::new(),
            scale: 1.0,
            rotation: 0.0,
            color: [255, 255, 255, 255],
            active: false,
            life_time: 0,
            crit: false,
            dot: false,
        }
    }
}

/// The box the C#'s `NewText` places the number in — the hitbox rectangle
/// (`CombatText.NewText`'s `location`).
#[derive(Clone, Copy, Debug)]
pub(crate) struct CombatTextBox {
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) w: f32,
    pub(crate) h: f32,
}

/// `CombatText.NewText` (`CombatText.cs:59-121`): fills the first free slot.
///
/// `text_width` is the measured string width (`FontAssets.CombatText`'s
/// `MeasureString`), `text_height` its height — the C# uses the measured
/// vector for the centring and the gravity-flip offset.
pub(crate) fn new_text(
    pool: &mut [CombatText],
    box_: CombatTextBox,
    color: CombatColor,
    text: &str,
    text_width: f32,
    text_height: f32,
    dramatic: bool,
    dot: bool,
    rng: &mut UnifiedRandom,
) -> Option<usize> {
    let slot = pool.iter().position(|t| !t.active)?;
    let t = &mut pool[slot];
    t.alpha = 1.0;
    t.alpha_dir = -1;
    t.active = true;
    t.scale = 0.0;
    t.rotation = 0.0;
    t.position.x = box_.x + box_.w * 0.5 - text_width * 0.5;
    t.position.y = box_.y + box_.h * 0.25 - text_height * 0.5;
    // `Main.rand.Next(-(int)(width*0.5), (int)(width*0.5) + 1)` — the C#'s
    // exclusive upper bound, so the jitter spans `[-w/2, w/2]`.
    let half_w = (box_.w * 0.5) as i32;
    let half_h = (box_.h * 0.5) as i32;
    t.position.x += rng.next_range(-half_w, half_w + 1) as f32;
    t.position.y += rng.next_range(-half_h, half_h + 1) as f32;
    t.color = color.rgb();
    t.text = text.to_string();
    t.velocity.y = -7.0;
    t.life_time = 60;
    t.crit = dramatic;
    t.dot = dot;
    if dramatic {
        t.life_time *= 2;
        t.velocity.y *= 2.0;
        t.velocity.x = rng.next_range(-25, 26) as f32 * 0.05;
        t.rotation = (t.life_time / 2) as f32 * 0.002;
        if t.velocity.x < 0.0 {
            t.rotation *= -1.0;
        }
    }
    if dot {
        t.velocity.y = -4.0;
        t.life_time = 40;
    }
    Some(slot)
}

/// `CombatText.NewText(location, color, amount)` (`CombatText.cs:54-57`):
/// the amount's own text, measured with the combat font.
pub(crate) fn new_amount(
    pool: &mut [CombatText],
    box_: CombatTextBox,
    color: CombatColor,
    amount: i32,
    dramatic: bool,
    dot: bool,
    font: Option<&crate::assets::UiFont>,
    rng: &mut UnifiedRandom,
) -> Option<usize> {
    let text = amount.to_string();
    let measured = match font {
        Some(f) => crate::ui::text::font_size(f, &text, 1.0),
        None => [0.0, 0.0],
    };
    new_text(
        pool,
        box_,
        color,
        &text,
        measured[0],
        measured[1],
        dramatic,
        dot,
        rng,
    )
}

/// `CombatText.Update` (`CombatText.cs:131-206`): the alpha fade, the rise
/// and the scale-in.
pub(crate) fn update(t: &mut CombatText) {
    if !t.active {
        return;
    }
    let target_scale = 1.0f32;
    t.alpha += t.alpha_dir as f32 * 0.05;
    if t.alpha <= 0.6 {
        t.alpha_dir = 1;
    }
    if t.alpha >= 1.0 {
        t.alpha = 1.0;
        t.alpha_dir = -1;
    }
    if t.dot {
        t.velocity.y += 0.15;
    } else {
        t.velocity.y *= 0.92;
        if t.crit {
            t.velocity.y *= 0.92;
        }
    }
    t.velocity.x *= 0.93;
    t.position += t.velocity;
    t.life_time -= 1;
    if t.life_time <= 0 {
        t.scale -= 0.1 * target_scale;
        if t.scale < 0.1 {
            t.active = false;
        }
        t.life_time = 0;
        if t.crit {
            t.alpha_dir = -1;
            t.scale += 0.07 * target_scale;
        }
        return;
    }
    if t.crit {
        if t.velocity.x < 0.0 {
            t.rotation += 0.001;
        } else {
            t.rotation -= 0.001;
        }
    }
    if t.dot {
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

/// `Main.cs:62341-62396`: the combat text is drawn in the UI batch with the
/// world zoom — five passes per number (the four shadows at ±1 px and the
/// bright text), the shadows at 30 % of the colour's brightness.
///
/// The port draws into the always-rebuilt `fx` range (the world effect
/// sprites: the same "world content outside the cached world range" slot the
/// rain and the dust use) and converts the world position with the same
/// transform the rain uses.
pub(crate) fn draw(
    r: &mut tr_render::SpriteRenderer,
    lookups: &crate::assets::Lookups,
    pool: &[CombatText],
    camera: Vec2,
    zoom: f32,
    screen: Vec2,
) {
    for t in pool.iter().filter(|t| t.active) {
        let font = if t.crit {
            lookups.combat_crit_font.as_ref()
        } else {
            lookups.combat_text_font.as_ref()
        };
        let Some(font) = font else { continue };
        let measured = crate::ui::text::font_size(font, &t.text, 1.0);
        let origin = [measured[0] * 0.5, measured[1] * 0.5];
        // `num15 = scale / targetScale` — the scale relative to the target.
        let num15 = t.scale;
        let alpha = t.alpha;
        // The shadow colour: the RGB at 30 % brightness, the alpha as is.
        let shadow = [
            (t.color[0] as f32 * num15 * alpha * 0.3) as i32 as f32 / 255.0,
            (t.color[1] as f32 * num15 * alpha * 0.3) as i32 as f32 / 255.0,
            (t.color[2] as f32 * num15 * alpha * 0.3) as i32 as f32 / 255.0,
            (t.color[3] as f32 * num15 * alpha) as i32 as f32 / 255.0,
        ];
        let bright = [
            (t.color[0] as f32 * num15 * alpha) as i32 as f32 / 255.0,
            (t.color[1] as f32 * num15 * alpha) as i32 as f32 / 255.0,
            (t.color[2] as f32 * num15 * alpha) as i32 as f32 / 255.0,
            (t.color[3] as f32 * num15 * alpha) as i32 as f32 / 255.0,
        ];
        let to_screen = |p: Vec2| {
            [
                (p.x - camera.x) * zoom + screen.x * 0.5,
                (p.y - camera.y) * zoom + screen.y * 0.5,
            ]
        };
        let base = to_screen(t.position);
        for pass in 0..5 {
            let (dx, dy, color) = match pass {
                0 => (-1.0, 0.0, shadow),
                1 => (1.0, 0.0, shadow),
                2 => (0.0, -1.0, shadow),
                3 => (0.0, 1.0, shadow),
                _ => (0.0, 0.0, bright),
            };
            let anchor = [base[0] + dx, base[1] + dy];
            crate::ui::text::push_font_text_rot(
                r,
                font,
                anchor,
                origin,
                t.scale,
                t.rotation,
                color,
                &t.text,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pool() -> Vec<CombatText> {
        vec![CombatText::default(); MAX_COMBAT_TEXT]
    }

    #[test]
    fn the_palette_matches_the_csharp_colours() {
        assert_eq!(CombatColor::DamagedFriendly.rgb(), [255, 80, 90, 255]);
        assert_eq!(CombatColor::DamagedHostile.rgb(), [255, 160, 80, 255]);
        assert_eq!(CombatColor::DamagedHostileCrit.rgb(), [255, 100, 30, 255]);
        // `OthersDamagedHostile = DamagedHostile * 0.4f` — the byte multiply.
        assert_eq!(CombatColor::OthersDamagedHostile.rgb(), [102, 64, 32, 255]);
        assert_eq!(CombatColor::OthersDamagedHostileCrit.rgb(), [102, 40, 12, 255]);
        assert_eq!(CombatColor::HealLife.rgb(), [100, 255, 100, 255]);
        assert_eq!(CombatColor::HealMana.rgb(), [100, 100, 255, 255]);
    }

    #[test]
    fn a_new_number_centres_on_its_box_and_rises() {
        // `CombatText.cs:86-92`: the position is the box's centre minus half
        // the measured text, the jitter comes from `Main.rand`, and the rise
        // is `velocity.Y = -7`.
        let mut pool = pool();
        let mut rng = UnifiedRandom::new(1234);
        let slot = new_text(
            &mut pool,
            CombatTextBox {
                x: 100.0,
                y: 200.0,
                w: 40.0,
                h: 42.0,
            },
            CombatColor::DamagedHostile,
            "12",
            20.0,
            10.0,
            false,
            false,
            &mut rng,
        )
        .expect("a free slot");
        let t = &pool[slot];
        assert!(t.active);
        assert_eq!(t.text, "12");
        assert_eq!(t.color, [255, 160, 80, 255]);
        assert_eq!(t.life_time, 60);
        assert_eq!(t.velocity.y, -7.0);
        assert_eq!(t.scale, 0.0);
        assert_eq!(t.alpha_dir, -1);
        // The un-jittered centre: `x + w*0.5 - text_w*0.5` = 100 + 20 - 10,
        // then the jitter spans the box's half-width/half-height
        // (`Main.rand.Next(-w/2, w/2 + 1)`).
        assert!(t.position.x > 90.0 && t.position.x < 130.0, "{:?}", t.position);
        assert!(t.position.y > 184.0 && t.position.y < 227.0, "{:?}", t.position);
    }

    #[test]
    fn a_crit_doubles_the_life_and_kicks_sideways() {
        // `CombatText.cs:101-112`: the crit doubles the life and the rise,
        // the horizontal kick is `Main.rand.Next(-25, 26) * 0.05f` and the
        // rotation follows the kick's sign.
        let mut pool = pool();
        let mut rng = UnifiedRandom::new(7);
        let slot = new_text(
            &mut pool,
            CombatTextBox {
                x: 0.0,
                y: 0.0,
                w: 20.0,
                h: 20.0,
            },
            CombatColor::DamagedHostileCrit,
            "99",
            10.0,
            10.0,
            true,
            false,
            &mut rng,
        )
        .unwrap();
        let t = &pool[slot];
        assert!(t.crit);
        assert_eq!(t.life_time, 120);
        assert_eq!(t.velocity.y, -14.0);
        assert!(t.velocity.x.abs() <= 25.0 * 0.05);
        if t.velocity.x < 0.0 {
            assert!(t.rotation < 0.0);
        } else {
            assert!(t.rotation >= 0.0);
        }
    }

    #[test]
    fn the_update_scales_in_and_expires() {
        // `CombatText.cs:131-206`: the scale grows by 0.1 per tick up to 1,
        // the life runs out at 60 and the text then shrinks away.
        let mut pool = pool();
        let mut rng = UnifiedRandom::new(3);
        let slot = new_text(
            &mut pool,
            CombatTextBox {
                x: 0.0,
                y: 0.0,
                w: 20.0,
                h: 20.0,
            },
            CombatColor::DamagedHostile,
            "5",
            10.0,
            10.0,
            false,
            false,
            &mut rng,
        )
        .unwrap();
        for _ in 0..10 {
            update(&mut pool[slot]);
        }
        assert!((pool[slot].scale - 1.0).abs() < 1e-6, "{}", pool[slot].scale);
        assert!(pool[slot].position.y < 0.0, "the number rises");
        // The life runs out: the scale shrinks and the slot frees itself.
        for _ in 0..60 {
            update(&mut pool[slot]);
        }
        assert!(!pool[slot].active, "the number expires");
    }

    #[test]
    fn the_pool_holds_a_hundred_slots() {
        let mut pool = pool();
        let mut rng = UnifiedRandom::new(1);
        let box_ = CombatTextBox {
            x: 0.0,
            y: 0.0,
            w: 20.0,
            h: 20.0,
        };
        for i in 0..MAX_COMBAT_TEXT {
            assert!(
                new_text(&mut pool, box_, CombatColor::DamagedHostile, "1", 5.0, 5.0, false, false, &mut rng)
                    .is_some(),
                "slot {i}"
            );
        }
        assert!(
            new_text(&mut pool, box_, CombatColor::DamagedHostile, "1", 5.0, 5.0, false, false, &mut rng)
                .is_none(),
            "the pool is full"
        );
    }
}
