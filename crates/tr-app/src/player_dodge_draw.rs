//! W4-d115 lane 19 — the dodge family's DRAW-side maths
//! (`LegacyPlayerRenderer.DrawPlayers` `:433-467` + the shield layer
//! `PlayerDrawLayers.DrawPlayer_25_Shield` `:2954-3024`).
//!
//! ## Census (the dust side is NOT here)
//!
//! The dodge handlers' dust/gore bursts (`Player.cs:38060-38112` sash
//! 50x type-267 + CloneDust pairs / 5-gore fan, `:38122-38160` ninja
//! 100x type-31 jitter dusts / 5-gore fan) are owned by **W4-g17 wave C**
//! (`player_gore_c.rs`, branch task/W4-g17-gore-world-sites, commit
//! 3ff14a0a) — this module never spawns particles. What vanilla draws
//! AROUND the dodging player is the `LegacyPlayerRenderer` afterimage
//! block, and it has no port consumer: this module ports its maths as
//! pure functions (the g17 code-only precedent — the `entities.rs`
//! draw wiring is the follow-up round, the renderer runs per-frame
//! outside the fixed step).
//!
//! ## The two afterimages (`LegacyPlayerRenderer.cs:433-467`)
//!
//! * **Shadow dodge** (`:433-454`): `shadowDodgeCount` (a `Player`
//!   float, `Player.cs:1166`) climbs +1/frame while `shadowDodge` is
//!   armed (cap 30), decays -1/frame otherwise (floor 0) — the counter
//!   is DRAW-owned (the renderer mutates it; `ResetEffects` never
//!   touches it). While > 0, TWO ghost copies draw at ±count on X,
//!   each with `shadow = 0.5 + Next(-10,11) * 0.005` — one `Main.rand`
//!   draw PER GHOST, left (+count) first (`:452-453`).
//! * **Brain of Confusion** (`:455-467`): while the 300-tick animation
//!   counter (`Player.cs:1158`, set by `BrainOfConfusionDodge`
//!   `:38049`) is > 0, `lerp = GetLerpValue(300, 270, counter)`
//!   (UNclamped, `Utils.cs:315-343` — the default `clamped = false`),
//!   `y = Lerp(2, 120, lerp)`; only while `0 <= lerp <= 1` (the first
//!   30 ticks, counter 300..=270) SIX ghosts ring the player: offset
//!   `(0, y).RotatedBy(2π·lerp·0.5 + k·π/3)`, `shadow = lerp`. The
//!   float accumulator steps `π/3` six times (0, π/3, … 5π/3).
//!
//! `shadow` is `DrawPlayer`'s ghost-alpha parameter
//! (`LegacyPlayerRenderer.cs:132`, fed to `BoringSetup` `:142` →
//! `GetImmuneAlphaPure`-family tinting) — the port's layer alpha.
//!
//! ## Wiring (the W4-d115 wiring round)
//!
//! The fns are LIVE: `render::entities::draw_dodge_afterimages` draws the
//! ghost copies before the main player copy (`render::tiles`' player
//! pass), and `Player::step_shadow_dodge_draw_counter` runs the
//! `:433-448` step once per rendered frame in `App::draw_region` (the
//! C# mutates the live Player inside the renderer — the port steps the
//! live field before the render clone). Writer census: `Player.cs:1166`
//! is the field's ONLY Player.cs mention — no dodge proc sets the count
//! (Holy Protection/Titanium arm the buff-59 flag `:6938-6941`/`:10076`,
//! the brain arms the animation counter `:38049`); the renderer is the
//! sole writer.
//!
//! ## The shield layer (`PlayerDrawLayers.cs:2954-3024`)
//!
//! The Cross Necklace's parry visual rides the Squire/Brand shield
//! layer, NOT the necklace itself (the necklace only arms
//! `longInvince`, `Player.cs:15587-15590`). The parry MACHINE
//! (`CanParryAgainst` / `TryTogglingShield` / `shieldParryTimeLeft`)
//! is **W4-f48**-owned (branch task/W4-f48-dodge-2, 64ca5fad…69d499c6);
//! this module ports only the layer's pulse/ghost maths:
//!
//! * raise lift: `zero.Y -= 4 * gravDir` (`:2961-2964`);
//! * the orbit pulse (`:2980-2990`): `num = sin(GlobalTimeWrappedHourly
//!   · 2π)`, orbit radius `x = 2.5 + 1.5·num`, FOUR ghosts at
//!   `(x, 0).RotatedBy(k/4 · 2π)`, colour `A = 0` then `*= 0.45 −
//!   num·0.15`;
//! * the breathe pulse (`:2997-3001`): `num3 = sin(t·π)`, the whole
//!   colour (A included) `*= 0.5 + 0.5·num3`;
//! * the parry ghost (`:3004-3018`, gated `shieldRaised &&
//!   shieldParryTimeLeft > 0`): `num4 = timeLeft/20`, `num5 = 1.5·num4`,
//!   position pushed AWAY from the player centre by `num5`, scale
//!   `1 + num5`, colour `*= 1 − num4`.
//!
//! No draws in any of the three shield arms (pure `sin` of the global
//! clock); the RNG contract lives only in the shadow ghosts. The shield
//! layer's own draw wiring stays with the f48 parry machine's follow-up
//! (the Acc_Shield sheets are unported); the afterimage fns above are
//! wired (the W4-d115 wiring round).
#![allow(dead_code)] // the shield-layer fns await the f48 draw follow-up; the afterimage fns are wired

use glam::Vec2;
use tr_core::unified_random::UnifiedRandom;

/// One afterimage copy: the offset from the player's draw position
/// (`position + netOffset + (0, gfxOffY)`) and the `DrawPlayer` shadow
/// parameter (the ghost alpha).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DodgeGhost {
    pub offset: Vec2,
    pub shadow: f32,
}

/// `LegacyPlayerRenderer.cs:433-448` — the per-frame counter step:
/// +1 while `shadowDodge` (cap 30), −1 otherwise (floor 0). The C#
/// mutates `drawPlayer.shadowDodgeCount` in the renderer; the port
/// keeps the field on `Player` and steps it here at draw time.
pub fn step_shadow_dodge_count(count: f32, shadow_dodge: bool) -> f32 {
    if shadow_dodge {
        (count + 1.0).min(30.0)
    } else {
        (count - 1.0).max(0.0)
    }
}

/// `LegacyPlayerRenderer.cs:449-454` — the two shadow-dodge ghosts at
/// ±count on X, `shadow = 0.5 + Next(-10,11) * 0.005` each (the +count
/// copy draws FIRST, so its roll rides the stream first). `None` while
/// the counter is 0.
pub fn shadow_dodge_ghosts(count: f32, rng: &mut UnifiedRandom) -> Option<[DodgeGhost; 2]> {
    if count <= 0.0 {
        return None;
    }
    let alpha = |rng: &mut UnifiedRandom| 0.5 + rng.next_range(-10, 11) as f32 * 0.005;
    Some([
        DodgeGhost {
            offset: Vec2::new(count, 0.0),
            shadow: alpha(rng),
        },
        DodgeGhost {
            offset: Vec2::new(-count, 0.0),
            shadow: alpha(rng),
        },
    ])
}

/// `Utils.GetLerpValue(300, 270, counter)` UNclamped
/// (`Utils.cs:315-343`, `clamped = false`): `(t − from)/(to − from)`.
fn brain_lerp(counter: i32) -> f32 {
    (counter as f32 - 300.0) / (270.0 - 300.0)
}

/// `LegacyPlayerRenderer.cs:455-467` — the brain dodge's six-ghost
/// ring. Empty while the counter is ≤ 0 or the lerp leaves [0, 1]
/// (counter < 270 — the aura only shows for the window's first 30
/// ticks). Offsets `(0, y).RotatedBy(2π·lerp·0.5 + k·π/3)` with
/// `y = Lerp(2, 120, lerp)`; every ghost's shadow IS the lerp.
/// RNG-free.
pub fn brain_dodge_ghosts(counter: i32) -> Vec<DodgeGhost> {
    if counter <= 0 {
        return Vec::new();
    }
    let lerp = brain_lerp(counter);
    if !(0.0..=1.0).contains(&lerp) {
        return Vec::new();
    }
    let y = 2.0 + (120.0 - 2.0) * lerp; // `MathHelper.Lerp(2f, 120f, lerp)`
    let mut out = Vec::with_capacity(6);
    // The C# float accumulator: `for (num9 = 0f; num9 < 2π; num9 += π/3)`.
    let mut angle = 0.0f32;
    let step = std::f32::consts::PI / 3.0;
    while angle < std::f32::consts::PI * 2.0 {
        let rot = std::f32::consts::PI * 2.0 * lerp * 0.5 + angle;
        // `Utils.RotatedBy` (`Utils.cs:1675-1684`): the double-precision
        // rotation the port's `bow_launch::rotated_by` mirrors.
        let offset = crate::bow_launch::rotated_by(Vec2::new(0.0, y), rot as f64);
        out.push(DodgeGhost {
            offset,
            shadow: lerp,
        });
        angle += step;
    }
    out
}

/// `PlayerDrawLayers.cs:2961-2964` — the raised shield's draw lift.
pub fn shield_raise_lift(grav_dir: f32) -> f32 {
    -4.0 * grav_dir
}

/// `PlayerDrawLayers.cs:2980-3001` — the raised shield's two pulses:
/// the orbit radius `2.5 + 1.5·sin(t·2π)` with ghost colour factor
/// `0.45 − sin(t·2π)·0.15` (A zeroed first), and the breathe factor
/// `0.5 + 0.5·sin(t·π)` on the main copy. Returns
/// `(orbit_x, orbit_color_factor, breathe_factor)`; the four orbit
/// ghosts sit at `(orbit_x, 0).RotatedBy(k/4 · 2π)`, k = 0..4.
pub fn shield_raise_pulses(global_time: f32) -> (f32, f32, f32) {
    let num = (global_time * std::f32::consts::PI * 2.0).sin();
    let num3 = (global_time * std::f32::consts::PI).sin();
    (2.5 + 1.5 * num, 0.45 - num * 0.15, 0.5 + 0.5 * num3)
}

/// `PlayerDrawLayers.cs:3004-3018` — the parry flash ghost, gated
/// `shieldRaised && shieldParryTimeLeft > 0` (the raise half rides the
/// caller). Returns `(pos_push, scale, color_factor)`: the copy's
/// position moves `(pos − playerCentre) · pos_push` further out, its
/// scale is `1 + pos_push`, its colour `·= color_factor`.
/// `num4 = timeLeft / 20` — the 20-tick parry window
/// (`TryTogglingShield`, `Player.cs:31741-31764`, W4-f48).
pub fn shield_parry_ghost(parry_time_left: i32) -> Option<(f32, f32, f32)> {
    if parry_time_left <= 0 {
        return None;
    }
    let num4 = parry_time_left as f32 / 20.0;
    let num5 = 1.5 * num4;
    Some((num5, 1.0 + num5, 1.0 - num4))
}

/// `Player.UpdateVisibleAccessories` (`Player.cs:36770-36777`) — the
/// raised-shield slot pick: while `shieldRaised` and no shield is
/// drawn yet (`shield == -1`), the FIRST accessory (slots 3..10 in
/// order) with a `shieldSlot != -1` becomes the drawn shield. The
/// `eocDash > 0` twin (`:36762-36769`) shares the arm; the port folds
/// both gates into one predicate (the caller ORs them). Returns the
/// slot id to draw, `None` keeps the current one.
///
/// W4-d115 lane 19 census: the Cross Necklace 554 itself draws NO
/// shield — its `neckSlot = 2` (`Item.cs:7992`) rides the Acc_Neck
/// layer (the whole visible-accessory layer family is unported, no
/// `Acc_*` sheets in `assets.rs`); the necklace's only combat write is
/// `longInvince` (`:15587-15590`, landed 41323c7c on the f48 branch).
/// The "shield-raise sprite" is the SQUIRE/BRAND shield the parry
/// machine raises (`hasRaisableShield`, the eleven `shieldSlot` rows
/// `:13123-13126` + held 4760 — W4-f48-owned); this fn is its draw-side
/// slot selection.
pub fn raised_shield_slot(
    current_shield: i32,
    raise_or_eoc_dash: bool,
    accessory_shield_slots: &[i32],
) -> Option<i32> {
    if !raise_or_eoc_dash || current_shield != -1 {
        return None;
    }
    accessory_shield_slots
        .iter()
        .find(|&&s| s != -1)
        .copied()
}

/// `PlayerDrawLayers.cs:2965-2976` — the shield layer's frame/origin
/// adjust: the body frame's WIDTH swaps to the Acc_Shield texture's
/// width (the row/height stay the player's), and on a horizontal flip
/// the origin X mirrors inside the new width. Returns
/// `(frame_width, origin_x)`.
///
/// The decompiled `bodyVect.X += bodyFrame.Width - tex.Width`
/// (`:2971`) runs AFTER the width assignment, so the add is always
/// zero — a decompilation artifact of the original's old-width
/// capture; the port keeps the effective semantics (the swap + the
/// flip mirror) and records the artifact here.
pub fn shield_layer_frame(
    body_frame_width: i32,
    tex_width: i32,
    origin_x: f32,
    flip_horizontally: bool,
) -> (i32, f32) {
    if body_frame_width == tex_width {
        return (body_frame_width, origin_x);
    }
    let width = tex_width;
    let mut ox = origin_x + (width - tex_width) as f32; // the += 0 artifact
    if flip_horizontally {
        ox = width as f32 - ox;
    }
    (width, ox)
}

/// `PlayerDrawLayers.cs:2992` — the shield layer's draw anchor (the
/// pre-`zero`/pulse base every copy shares): the integer-truncated
/// `(Position.X − screenX − frameWidth/2 + playerWidth/2, Position.Y −
/// screenY + playerHeight − bodyFrameHeight + 4)` plus `bodyPosition`
/// plus the frame centre `(frameWidth/2, bodyFrameHeight/2)`. The
/// port's layers anchor on the 40×56 composite box, so the caller
/// feeds the box origin as `position` and the live body-frame height.
pub fn shield_layer_anchor(
    position: Vec2,
    screen_position: Vec2,
    player_width: i32,
    player_height: i32,
    frame_width: i32,
    body_frame_height: i32,
    body_position: Vec2,
) -> Vec2 {
    Vec2::new(
        (position.x - screen_position.x - (frame_width / 2) as f32
            + (player_width / 2) as f32) as i32 as f32,
        (position.y - screen_position.y + player_height as f32
            - body_frame_height as f32
            + 4.0) as i32 as f32,
    ) + body_position
        + Vec2::new((frame_width / 2) as f32, (body_frame_height / 2) as f32)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rng() -> UnifiedRandom {
        UnifiedRandom::new(12345)
    }

    /// `:433-448` — the counter climbs to 30 and decays to 0, never
    /// past either rail; the decay runs even from a mid value.
    #[test]
    fn w4_d115_lane19_the_shadow_counter_clamps_at_30_and_0() {
        let mut c = 0.0;
        for _ in 0..40 {
            c = step_shadow_dodge_count(c, true);
        }
        assert_eq!(c, 30.0, "the cap holds");
        for _ in 0..40 {
            c = step_shadow_dodge_count(c, false);
        }
        assert_eq!(c, 0.0, "the floor holds");
        assert_eq!(step_shadow_dodge_count(10.0, true), 11.0);
        assert_eq!(step_shadow_dodge_count(10.0, false), 9.0);
    }

    /// `:449-454` — two ghosts at ±count, one Next(-10,11) roll each,
    /// the +count copy first; alpha stays in [0.45, 0.55].
    #[test]
    fn w4_d115_lane19_the_shadow_ghosts_mirror_and_roll_in_order() {
        assert!(shadow_dodge_ghosts(0.0, &mut rng()).is_none());
        let mut r = rng();
        let g = shadow_dodge_ghosts(12.0, &mut r).unwrap();
        assert_eq!(g[0].offset, Vec2::new(12.0, 0.0), "+count first (:452)");
        assert_eq!(g[1].offset, Vec2::new(-12.0, 0.0));
        for ghost in g {
            assert!(
                (0.45..=0.55).contains(&ghost.shadow),
                "0.5 ± 10·0.005 = [0.45, 0.55]"
            );
        }
        // Determinism: the same seed replays the same two alphas.
        let g2 = shadow_dodge_ghosts(12.0, &mut rng()).unwrap();
        assert_eq!(g, g2);
    }

    /// `:455-467` — the ring shows only for counter 300..=270 (the
    /// unclamped lerp gate), six ghosts, shadow = lerp; at 300 the
    /// radius is 2 and the spin 0, at 270 the radius 120 and spin π.
    #[test]
    fn w4_d115_lane19_the_brain_ring_gates_on_the_first_30_ticks() {
        assert!(brain_dodge_ghosts(0).is_empty());
        assert!(brain_dodge_ghosts(269).is_empty(), "lerp > 1 gates off");
        assert!(brain_dodge_ghosts(-5).is_empty());
        let start = brain_dodge_ghosts(300);
        // `LegacyPlayerRenderer.cs:462` accumulates π/3 in f32: the sixth sum
        // is 6.2831853 < the f32 2π (6.2831855), so the loop takes a seventh
        // step — the source's float quirk, not six exact iterations.
        assert_eq!(start.len(), 7, "the f32 π/3 accumulator takes seven steps");
        for g in &start {
            assert_eq!(g.shadow, 0.0, "lerp(300) = 0");
            assert!((g.offset.length() - 2.0).abs() < 1e-4, "y = Lerp(2,120,0)");
        }
        // At lerp 0 the first ghost sits at (0, 2) — no spin yet.
        assert!((start[0].offset.x).abs() < 1e-4);
        assert!((start[0].offset.y - 2.0).abs() < 1e-4);
        let end = brain_dodge_ghosts(270);
        assert_eq!(end.len(), 7, "the f32 accumulator's seventh step");
        for g in &end {
            assert_eq!(g.shadow, 1.0, "lerp(270) = 1");
            assert!((g.offset.length() - 120.0).abs() < 1e-3);
        }
        // The spin at lerp 1 is π: the first ghost flips to (0, −120).
        assert!((end[0].offset.x).abs() < 1e-3);
        assert!((end[0].offset.y + 120.0).abs() < 1e-3);
        // Mid-window: lerp 0.5 → radius 61, every ghost equidistant.
        let mid = brain_dodge_ghosts(285);
        assert_eq!(mid.len(), 7);
        for g in &mid {
            assert!((g.shadow - 0.5).abs() < 1e-6);
            assert!((g.offset.length() - 61.0).abs() < 1e-3);
        }
    }

    /// `:2980-3001` — at t = 0 both sins are 0: orbit 2.5, orbit colour
    /// 0.45, breathe 0.5; at t = 0.25 the orbit sin peaks (radius 4,
    /// colour 0.30) while the breathe sin is √2/2.
    #[test]
    fn w4_d115_lane19_the_shield_pulses_ride_the_global_clock() {
        let (x, c, b) = shield_raise_pulses(0.0);
        assert!((x - 2.5).abs() < 1e-6);
        assert!((c - 0.45).abs() < 1e-6);
        assert!((b - 0.5).abs() < 1e-6);
        let (x, c, b) = shield_raise_pulses(0.25);
        assert!((x - 4.0).abs() < 1e-5);
        assert!((c - 0.30).abs() < 1e-5);
        assert!((b - (0.5 + 0.5 * std::f32::consts::FRAC_1_SQRT_2)).abs() < 1e-5);
        assert_eq!(shield_raise_lift(1.0), -4.0, ":2963 — up 4 px");
    }

    /// `:3004-3018` — the parry ghost fades over the 20-tick window:
    /// at 20 the copy is fully bright, scale 2.5, pushed 1.5 out; at 1
    /// it is nearly gone; at 0 the gate closes.
    #[test]
    fn w4_d115_lane19_the_parry_ghost_fades_over_20_ticks() {
        assert!(shield_parry_ghost(0).is_none());
        assert!(shield_parry_ghost(-1).is_none());
        let (push, scale, color) = shield_parry_ghost(20).unwrap();
        assert!((push - 1.5).abs() < 1e-6);
        assert!((scale - 2.5).abs() < 1e-6);
        assert!((color - 0.0).abs() < 1e-6, "num4 = 1 → fully faded");
        let (push, scale, color) = shield_parry_ghost(10).unwrap();
        assert!((push - 0.75).abs() < 1e-6);
        assert!((scale - 1.75).abs() < 1e-6);
        assert!((color - 0.5).abs() < 1e-6);
    }

    /// `Player.cs:36770-36777` — the first shieldSlot accessory wins,
    /// only while raised (or eoc-dashing) and nothing drawn yet. The
    /// Cross Necklace's neckSlot never feeds this arm.
    #[test]
    fn w4_d115_lane19_the_raised_shield_picks_the_first_shield_slot() {
        // No raise: nothing picked.
        assert_eq!(raised_shield_slot(-1, false, &[6, 2]), None);
        // Already drawn: the pick never re-fires.
        assert_eq!(raised_shield_slot(6, true, &[6, 2]), None);
        // Raised, slots in order: the first non-(-1) wins.
        assert_eq!(raised_shield_slot(-1, true, &[-1, -1, 6, 2]), Some(6));
        // No shield accessory: nothing to draw.
        assert_eq!(raised_shield_slot(-1, true, &[-1, -1]), None);
        assert_eq!(raised_shield_slot(-1, true, &[]), None);
    }

    /// `PlayerDrawLayers.cs:2965-2976` — the width swap + the flip
    /// mirror (the += 0 decompilation artifact keeps the origin on the
    /// no-flip path).
    #[test]
    fn w4_d115_lane19_the_shield_frame_swaps_width_and_mirrors_on_flip() {
        // Equal widths: untouched.
        assert_eq!(shield_layer_frame(40, 40, 20.0, false), (40, 20.0));
        assert_eq!(shield_layer_frame(40, 40, 20.0, true), (40, 20.0));
        // The swap: width becomes the texture's, origin rides (+= 0).
        assert_eq!(shield_layer_frame(40, 44, 20.0, false), (44, 20.0));
        // The flip mirrors inside the NEW width.
        assert_eq!(shield_layer_frame(40, 44, 20.0, true), (44, 24.0));
    }

    /// `PlayerDrawLayers.cs:2992` — the anchor's integer truncation +
    /// the frame-centre add.
    #[test]
    fn w4_d115_lane19_the_shield_anchor_truncates_then_centres() {
        let a = shield_layer_anchor(
            Vec2::new(100.7, 200.9),
            Vec2::new(10.0, 20.0),
            20,
            42,
            40,
            56,
            Vec2::ZERO,
        );
        // x: (int)(100.7 − 10 − 20 + 10) = 80, + 20 = 100.
        // y: (int)(200.9 − 20 + 42 − 56 + 4) = 170, + 28 = 198.
        assert_eq!(a, Vec2::new(100.0, 198.0));
        // bodyPosition rides the whole anchor.
        let b = shield_layer_anchor(
            Vec2::new(100.7, 200.9),
            Vec2::new(10.0, 20.0),
            20,
            42,
            40,
            56,
            Vec2::new(1.0, 2.0),
        );
        assert_eq!(b, a + Vec2::new(1.0, 2.0));
    }
}
