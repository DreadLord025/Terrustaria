//! W4-c10 sweep — `NPC.TransformVisuals(int oldType, int newType)`
//! (`NPC.cs:80963-81033`), the visuals half of `NPC.Transform`
//! (`:81136` calls it after the SetDefaults swap + the ai overrides).
//! The body censused WHOLE; the port keeps every `Main.rand` draw in
//! the C#'s order (the dust/gore BODIES are the documented visual skip,
//! the `npc_ai_empress.rs::new_dust_draws` convention) and returns the
//! one sound cue as an intent (the `wants_sound` pattern).
//!
//! THE WATERFOWL VERDICT (the W4-c10 census): NONE of the eight
//! waterfowl transform pairs — the dry landing's 363→362 / 365→364 /
//! 603→602 / 609→608 (`NPC.cs:34760`) and the takeoff's 362→363 /
//! 364→365 / 602→603 / 608→609 (`:53182`) — matches ANY arm below.
//! The takeoff/landing transforms are VISUALLY SILENT: zero draws,
//! zero cues. The `netOffset` add/subtract pair (`:80965`/`:81032`)
//! brackets the whole body but is zero on the host/single-player
//! (the port's `net_offset` doc, `npc.rs`), so it folds to a no-op.
//!
//! The five arms (the C#'s exact structure — the first three are an
//! if/else-if chain, the last two are INDEPENDENT ifs, `:81008`/
//! `:81020`; no pair matches two arms, so the distinction is
//! unreachable but kept):
//! * 158↔159 (VampireBat↔Vampire, `:80966-80975`): `SoundID.Item8`
//!   at `base.position` + THREE gore-99 spawns at (x, y−10),
//!   (x, y+h/2−10), (x, y+h−10) — the INTEGER `base.height / 2`
//!   (`(float)(base.height / 2)`), each `velocity *= 0.3`. Per gore
//!   `Gore.NewGore` draws its two kicks: `Next(10, 31)` Y then
//!   `Next(-20, 21)` X (`Gore.cs:1327-1328`; `ChildSafety.Disabled`
//!   is true, `ChildSafety.cs:33`, so the dangerous-gore re-roll
//!   never fires). SIX draws + the cue. Neither kind is ported.
//! * 478→479 (MothronEgg→MothronSpawn, `:80976-80990`): THIRTY
//!   dust-238 (`NewDust(position, w, h, 238, 0, -2)` — six internal
//!   draws each, `Dust.cs:170-175`/`:190`; 238 matches no type arm
//!   `:214-252`) each followed by the `Next(2)` noGravity gate; then
//!   FOUR gores 684/685/686/684+`Next(3)` — the last's `Next(3)` is
//!   an ARGUMENT, drawn BEFORE that gore's two kicks. 30×7 + 3×2 +
//!   (1+2) = 219 draws. Neither kind is ported.
//! * 406→405 (StardustCellSmall→Big, `:80991-81007`): the count
//!   `Next(10, 21)` (`:80993`) then the angle `NextFloat` (`:80994`);
//!   per dust: `NewDust(Center, 0, 0, 229)` (w/h clamp to 5 → six
//!   internals) THEN the scale `NextFloat` (`:81003`) THEN the
//!   fadeIn `NextFloat` (`:81005`) — the position/velocity writes
//!   between them draw nothing. 2 + count×8 draws. Both kinds ARE
//!   ported; WIRED (W4-c10 final): `update_stardust_cell`'s growth
//!   flip calls the census at the C# position (after the kind swap).
//! * 428→427 (VortexLarva→VortexHornet, `:81008-81019`): TWENTY
//!   dust, each `Utils.SelectRandom(Main.rand, 229, 240)` =
//!   `Next(2)` (`Utils.cs:2459-2462`, drawn as the NewDust type
//!   ARGUMENT, before the six internals) then the scale `NextFloat`
//!   (`:81015`). 20×8 = 160 draws. Ported as VortexHeadcrab→
//!   VortexAlien; WIRED (W4-c10 final): `update_vortex_headcrab`'s
//!   metamorphosis flip calls the census.
//! * 427→426 (VortexHornet→VortexHornetQueen, `:81020-81031`):
//!   THIRTY dust, each `SelectRandom(Main.rand, 229, 240, 240)` =
//!   `Next(3)` then six internals then the scale `NextFloat`
//!   (`:81027`). 30×8 = 240 draws. Ported as VortexAlien→
//!   VortexHunter; WIRED (W4-c10 final): `update_vortex_alien`'s
//!   metamorphosis flip calls the census.
//!
//! ACTIVATION CONTRACT (W4-c10 final, the per-kind flip-site census):
//! the three PORTED pairs above are wired at their in-place kind
//! flips (`npc.rs`); the two UNPORTED pairs stay module-ready —
//! 158↔159 (VampireBat/Vampire) is claimed by W4-d36's band
//! (research/109-d36: 159 Vampire ports at proto 762+; 158 VampireBat
//! skipped to g8 per 108-E2; the flip callers `NPC.cs:23353`/`:58850`
//! ride `npc_ai_style14::bat_ai`'s cited unported branch + the
//! aiStyle-3 vampire arm), and 478→479 (MothronEgg/MothronSpawn) is
//! claimed by W4-d40's 421-490 band (both kinds rostered on their tip,
//! the aiStyle-89 egg timer's `Transform(479)` at `NPC.cs:38380` is
//! their wiring step). When those lanes land their flips they call
//! `transform_visuals(old, new, w, h, rng)` with the NEW type's
//! SetDefaults box — the vampire arm's cue returns `Item8` for the
//! caller's `wants_sound` queue.
//!
//! CODE-ONLY MODE (W4-c10 sweep): the witnesses at the bottom are
//! WRITTEN NOT RUN; every expected count is hand-derived from the
//! cited lines.
#![allow(dead_code)] // the vampire/mothron arms await their lanes' kinds (the activation contract above)

use tr_core::UnifiedRandom;

/// The cue `TransformVisuals` plays: `SoundID.Item8` =
/// `LegacySoundStyle(2, 8)` at `base.position` (the 158↔159 arm,
/// `:80968`). The caller queues it on `wants_sound`
/// (`NpcSoundCue::Item8`, the caster-puff precedent).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransformCue {
    Item8,
}

/// `Dust.NewDust`'s six internal draws (`Dust.cs:170-175`, `:190`)
/// under the documented pool assumption (no skip draws, no
/// early-out) — the `npc_ai_empress.rs::new_dust_draws` shape, kept
/// local so that file stays byte-identical this round.
fn new_dust_draws(rng: &mut UnifiedRandom, width: i32, height: i32) {
    let w = width.max(5); // `Dust.cs:155-162`
    let h = height.max(5);
    let _ = rng.next_max(w - 4); // :170 position.X
    let _ = rng.next_max(h - 4); // :171 position.Y
    let _ = rng.next_range(-20, 21); // :172 velocity.X
    let _ = rng.next_range(-20, 21); // :173 velocity.Y
    let _ = rng.next_max(3); // :175 frame.Y
    let _ = rng.next_range(-20, 21); // :190 scale
}

/// `Gore.NewGore`'s two velocity kicks (`Gore.cs:1327-1328`): the Y
/// lift `Next(10, 31)` FIRST, then the X kick `Next(-20, 21)`. The
/// body (frame/sticky/scale) draws nothing else —
/// `ChildSafety.Disabled` is true (`ChildSafety.cs:33`).
fn new_gore_kicks(rng: &mut UnifiedRandom) {
    let _ = rng.next_range(10, 31);
    let _ = rng.next_range(-20, 21);
}

/// `NPC.TransformVisuals` (`NPC.cs:80963-81033`) — the draw census +
/// the cue intent. `old_type`/`new_type` are the VANILLA ids (the
/// C#'s `this.type`; the port's `cs_id`), `width`/`height` the NEW
/// type's `SetDefaults` box (the body runs after the swap). Returns
/// the one cue the matched arm plays, `None` for every silent pair
/// (including ALL EIGHT waterfowl pairs — the census verdict).
pub fn transform_visuals(
    old_type: i32,
    new_type: i32,
    width: i32,
    height: i32,
    rng: &mut UnifiedRandom,
) -> Option<TransformCue> {
    // `:80966-80975` — the vampire flip: Item8 + three gore-99.
    if (old_type == 158 && new_type == 159) || (old_type == 159 && new_type == 158) {
        for _ in 0..3 {
            new_gore_kicks(rng);
        }
        return Some(TransformCue::Item8);
    } else if old_type == 478 && new_type == 479 {
        // `:80978-80985` — thirty dust-238 + the Next(2) gate each.
        for _ in 0..30 {
            new_dust_draws(rng, width, height);
            let _ = rng.next_max(2); // :80981 noGravity gate
        }
        // `:80986-80989` — gores 684/685/686 then 684 + Next(3):
        // the type roll is the ARGUMENT, drawn before the kicks.
        for _ in 0..3 {
            new_gore_kicks(rng);
        }
        let _ = rng.next_max(3); // :80989 the 684+Next(3) pick
        new_gore_kicks(rng);
        return None;
    } else if old_type == 406 && new_type == 405 {
        // `:80993-80994` — the count then the angle.
        let count = rng.next_range(10, 21);
        let _ = rng.next_float();
        for _ in 0..count {
            // `:80998` — NewDust(Center, 0, 0, 229): w/h clamp to 5.
            new_dust_draws(rng, 0, 0);
            let _ = rng.next_float(); // :81003 scale
            let _ = rng.next_float(); // :81005 fadeIn
        }
        return None;
    }
    // `:81008-81019` — the larva→hornet dust ring (an INDEPENDENT if
    // in the C#, kept as its own test).
    if old_type == 428 && new_type == 427 {
        for _ in 0..20 {
            let _ = rng.next_max(2); // :81012 SelectRandom(229, 240)
            new_dust_draws(rng, width, height);
            let _ = rng.next_float(); // :81015 scale
        }
    }
    // `:81020-81031` — the hornet→queen dust ring.
    if old_type == 427 && new_type == 426 {
        for _ in 0..30 {
            let _ = rng.next_max(3); // :81024 SelectRandom(229, 240, 240)
            new_dust_draws(rng, width, height);
            let _ = rng.next_float(); // :81027 scale
        }
    }
    None
}

// =====================================================================
// Witness tests — WRITTEN, NOT RUN (code-only mode: no cargo/rustc).
// =====================================================================
#[cfg(test)]
mod tests {
    use super::*;

    /// THE WATERFOWL SILENCE — all eight takeoff/landing pairs
    /// (`NPC.cs:34760` / `:53182`) match no arm: zero draws, no cue.
    #[test]
    fn the_waterfowl_transforms_are_visually_silent() {
        for (old, new) in [
            (363, 362), (365, 364), (603, 602), (609, 608), // the dry landing
            (362, 363), (364, 365), (602, 603), (608, 609), // the takeoff
        ] {
            let mut rng = UnifiedRandom::new(42);
            let before = rng.clone();
            let cue = transform_visuals(old, new, 22, 26, &mut rng);
            assert_eq!(cue, None, "{old}→{new}: no cue");
            assert_eq!(rng, before, "{old}→{new}: zero draws");
        }
    }

    /// `:80966-80975` — the vampire flip: Item8 + 3×2 gore kicks,
    /// both directions.
    #[test]
    fn the_vampire_flip_plays_item8_and_six_kick_draws() {
        for (old, new) in [(158, 159), (159, 158)] {
            let mut a = UnifiedRandom::new(7);
            let mut b = UnifiedRandom::new(7);
            let cue = transform_visuals(old, new, 24, 26, &mut a);
            assert_eq!(cue, Some(TransformCue::Item8));
            for _ in 0..3 {
                let _ = b.next_range(10, 31);
                let _ = b.next_range(-20, 21);
            }
            assert_eq!(a, b, "{old}→{new}: exactly the six kicks");
        }
    }

    /// `:80976-80990` — the mothron hatch: 30×(6 dust + Next(2)) +
    /// 3×2 kicks + Next(3) + 2 kicks = 219 draws, no cue.
    #[test]
    fn the_mothron_hatch_censuses_219_draws() {
        let mut a = UnifiedRandom::new(9);
        let mut b = UnifiedRandom::new(9);
        assert_eq!(transform_visuals(478, 479, 30, 30, &mut a), None);
        for _ in 0..30 {
            for _ in 0..2 {
                let _ = b.next_max(26); // max(30,5)-4
            }
            let _ = b.next_range(-20, 21);
            let _ = b.next_range(-20, 21);
            let _ = b.next_max(3);
            let _ = b.next_range(-20, 21);
            let _ = b.next_max(2);
        }
        for _ in 0..3 {
            let _ = b.next_range(10, 31);
            let _ = b.next_range(-20, 21);
        }
        let _ = b.next_max(3);
        let _ = b.next_range(10, 31);
        let _ = b.next_range(-20, 21);
        assert_eq!(a, b, "the 219-draw order");
    }

    /// `:80991-81007` — the stardust growth: Next(10,21) count +
    /// NextFloat angle + count×(6 internals + 2 NextFloat), the
    /// count drawn from the SAME stream (the mirror replays it).
    #[test]
    fn the_stardust_growth_censuses_the_drawn_count() {
        for seed in 0..16 {
            let mut a = UnifiedRandom::new(seed);
            let mut b = UnifiedRandom::new(seed);
            assert_eq!(transform_visuals(406, 405, 44, 44, &mut a), None);
            let count = b.next_range(10, 21);
            let _ = b.next_float();
            for _ in 0..count {
                // NewDust(Center, 0, 0, 229): the 5-clamp.
                let _ = b.next_max(1);
                let _ = b.next_max(1);
                let _ = b.next_range(-20, 21);
                let _ = b.next_range(-20, 21);
                let _ = b.next_max(3);
                let _ = b.next_range(-20, 21);
                let _ = b.next_float(); // scale
                let _ = b.next_float(); // fadeIn
            }
            assert_eq!(a, b, "seed {seed}: 2 + count×8 draws");
        }
    }

    /// `:81008-81031` — the two vortex rings: 20×(Next(2)+6+float)
    /// and 30×(Next(3)+6+float); the SelectRandom draw precedes the
    /// NewDust internals (the type argument).
    #[test]
    fn the_vortex_rings_census_the_select_random_first() {
        let mut a = UnifiedRandom::new(3);
        let mut b = UnifiedRandom::new(3);
        assert_eq!(transform_visuals(428, 427, 40, 28, &mut a), None);
        for _ in 0..20 {
            let _ = b.next_max(2);
            let _ = b.next_max(36); // max(40,5)-4
            let _ = b.next_max(24); // max(28,5)-4
            let _ = b.next_range(-20, 21);
            let _ = b.next_range(-20, 21);
            let _ = b.next_max(3);
            let _ = b.next_range(-20, 21);
            let _ = b.next_float();
        }
        assert_eq!(a, b, "the larva ring: 160 draws");
        let mut a = UnifiedRandom::new(4);
        let mut b = UnifiedRandom::new(4);
        assert_eq!(transform_visuals(427, 426, 50, 62, &mut a), None);
        for _ in 0..30 {
            let _ = b.next_max(3);
            let _ = b.next_max(46);
            let _ = b.next_max(58);
            let _ = b.next_range(-20, 21);
            let _ = b.next_range(-20, 21);
            let _ = b.next_max(3);
            let _ = b.next_range(-20, 21);
            let _ = b.next_float();
        }
        assert_eq!(a, b, "the queen ring: 240 draws");
    }
}
