//! The aiStyle-47 family arm — `AI_047_GolemFist` (`NPC.cs:19231-19503`),
//! the Golem's two lunging fists (the left 247 / the right 248,
//! `SetDefaults` `NPC.cs:12193-12206`: 40x30, aiStyle 47, dmg 59, def 28,
//! 10000 HP, `noGravity`, `alpha = 255`, kb 0). S1-D-3 migrates the family
//! off the kind-keyed `update_golem` arm onto the aiStyle axis
//! (`npc.rs`'s `match self.kind.ai_style()`).
//!
//! The machine: the tether to the body's anchor (the follow, state 0), the
//! 60-contact cadence into the 30-tick wind-up (state 1), the dash
//! (state 2) and the dead state-3 hover. The anchor rides the World's
//! `golem_*` write-through (`world_sim/npcs.rs`'s pre-pass hoist — the
//! body's centre / velocity.X / life ratio), the one-tick shape recorded
//! in ANALYSIS "The Golem family's display".
//!
//! RNG-free by construction: the C#'s only `Main.rand` draws in this arm
//! are the dust spawns — the wind-up's two `NextVector2Circular`
//! (`:19341`/`:19343`) and the dash's `Next(2)` gate + `NextVector2Circular`
//! (`:19417`/`:19420`). The dust is a documented skip, so the arm draws no
//! random number and needs no stream (the signature carries none).
//!
//! Documented deviations (ANALYSIS "The Golem family's display"):
//! * `GetMyBalance` (`:19233`, `:18483-18491`) is 1.0 for the solo unscaled
//!   player; the good-world `+3` (`:19234-19237`) and the outside-the-
//!   temple/jungle-or-above-surface `*2` (`:19238-19241`) stay off.
//! * The dust arms (`:19337-19353`, `:19417-19422`) and the dash's Item14
//!   sound (`:19413-19416`) / good-world torch break (`:19394-19411`) are
//!   skipped.
//! * The fists integrate raw (`noTileCollide`), so `collideX`/`collideY`
//!   never set — the dash's `:19445` exit rides the 700-px anchor distance
//!   (the flags are reset/checked faithfully but stay false).
//! * The state-3 hover (`:19451-19502`) is dead in the vanilla AI (nothing
//!   sets `ai[0] = 3` for the fists) and is not carried.

use crate::npc::{Npc, NpcKind};
use crate::player::Player;
use tr_core::{World, TILE_PX};

/// `AI_047_GolemFist` (`NPC.cs:19231-19503`) for one fist, then the raw
/// `noTileCollide`/`noGravity` step. `left` is the type-247 arm (the
/// `this.type == 247` folds); the right fist is 248.
pub(crate) fn ai_047_golem_fist(npc: &mut Npc, world: &World, player: &Player) {
    // `:19233` — `num = GetMyBalance()`. Solo unscaled => 1.0
    // (`:18485-18487`); the good-world / zone multipliers stay off
    // (ANALYSIS). The `(num + 3) / 4` speed factor below is thus x1.
    let num = 1.0f32;
    // `:19242-19246` — `NPC.golemBoss < 0` => `StrikeNPCNoInteraction(9999)`.
    // The sim also kills the parts on the body's absence
    // (`world_sim/npcs.rs`); the guard keeps the standalone step faithful.
    if !world.golem_present {
        npc.hp = 0;
        return;
    }
    // `:19247-19255` — the spawn fade: `alpha -= 10` (clamp 0) holds the
    // cadence counter at 0 while the fist fades in.
    if npc.alpha > 0 {
        npc.alpha = npc.alpha.saturating_sub(10);
        npc.ai[1] = 0.0;
    }
    let (w, h) = npc.size();
    let (cx, cy) = npc.center();
    let (pcx, pcy) = player.center();
    // `this.type == 247` — the left fist (`NPCID.GolemFistLeft`, 247).
    let left = npc.kind == NpcKind::GolemFistLeft;
    // `:19258-19259` — the anchor: `body.Center + body.velocity +
    // (0, -9 * scale)`, then `X += (247 ? -84 : 78) * scale`. The body's
    // centre / velocity.X ride the World hoist (the one-tick shape). The
    // hoist carries no `golem_vy`, so the body's velocity.Y term of the
    // anchor folds out (the pre-existing seam shape, ANALYSIS).
    let scale = npc.scale;
    let anchor_x = world.golem_x + world.golem_vx + if left { -84.0 } else { 78.0 } * scale;
    let anchor_y = world.golem_y - 9.0 * scale;
    // `:19260-19261` — `vector2 = anchor - Center`, `num2 = Length()`,
    // computed once at the top of the tick (the state-2 exit reads it).
    let dx = anchor_x - cx;
    let dy = anchor_y - cy;
    let num2 = (dx * dx + dy * dy).sqrt();

    if npc.ai[0] == 0.0 {
        // The follow (`:19262-19330`). `:19264` sets `noTileCollide = true`
        // — implicit here: the fists integrate raw (see the tail).
        // `:19265-19282` — the approach speed: base 14, +3/+3 under the
        // own half/quarter life, +8 while the body is damaged, x(num+3)/4,
        // clamped at 32.
        let mut num3 = 14.0f32;
        if npc.hp < npc.max_hp / 2 {
            num3 += 3.0;
        }
        if npc.hp < npc.max_hp / 4 {
            num3 += 3.0;
        }
        if world.golem_hp_ratio < 1.0 {
            num3 += 8.0;
        }
        num3 *= (num + 3.0) / 4.0;
        if num3 > 32.0 {
            num3 = 32.0;
        }
        if num2 < 12.0 + num3 {
            // In contact (`:19286-19318`): the raw delta velocity, the
            // rotation reset, and the cadence accumulates ONLY here.
            npc.rotation = 0.0;
            npc.vx = dx;
            npc.vy = dy;
            // `:19291-19304` — `num5 = num`; +num5, +num5 under half life,
            // +num5 under quarter, +10*num5 while the body is damaged.
            let num5 = num;
            npc.ai[1] += num5;
            if npc.hp < npc.max_hp / 2 {
                npc.ai[1] += num5;
            }
            if npc.hp < npc.max_hp / 4 {
                npc.ai[1] += num5;
            }
            if world.golem_hp_ratio < 1.0 {
                npc.ai[1] += 10.0 * num5;
            }
            if npc.ai[1] >= 60.0 {
                // `:19307` — `TargetClosest()`. The shared selection
                // (`npc_target.rs`, S1-D-3p); the stock aggro/no-aggro/tank
                // values keep the C# metric's plain solo form.
                let cand = crate::npc_target::TargetCandidate {
                    active: true,
                    dead: player.dead,
                    ghost: false,
                    x: player.x,
                    y: player.y,
                    width: crate::player::WIDTH as i32,
                    height: crate::player::HEIGHT as i32,
                    aggro: 0,
                    item_animation: player.frame_inputs.swing_anim as i32,
                    npc_type_no_aggro: false,
                    tank_pet: None,
                };
                crate::npc_target::target_closest(npc, world, &[cand], true, 47, 0);
                // `:19308-19317` — the side gate: the left fist dashes when
                // the player stands left of `Center.X + 100`, the right
                // fist when they stand right of `Center.X - 100`; else the
                // cadence resets and the fist keeps hovering.
                let dashes = if left { cx + 100.0 > pcx } else { cx - 100.0 < pcx };
                npc.ai[1] = 0.0;
                if dashes {
                    npc.ai[0] = 1.0;
                }
            }
        } else {
            // Out of range (`:19319-19329`): move at num3 along the delta.
            let ratio = num3 / num2;
            npc.vx = dx * ratio;
            npc.vy = dy * ratio;
            // `:19324-19328` — the rotation: the default aims AGAINST the
            // velocity (`atan2(-vy, -vx)`); the type-247 (left) arm aims
            // ALONG it (`atan2(vy, vx)`). (The pre-axis arm had these two
            // swapped; the axis port restores the C# order.)
            npc.rotation = if left {
                npc.vy.atan2(npc.vx)
            } else {
                (-npc.vy).atan2(-npc.vx)
            };
        }
    } else if npc.ai[0] == 1.0 {
        // The wind-up (`:19331-19390`): the hard Center snap to the anchor
        // + the 30-tick hold; the launch sets the dash speed tiers.
        npc.ai[1] += 1.0;
        // `:19334` — `base.Center = vector` (the anchor snap).
        npc.x = anchor_x - w * 0.5;
        npc.y = anchor_y - h * 0.5;
        npc.rotation = 0.0;
        npc.vx = 0.0;
        npc.vy = 0.0;
        // `:19337-19353` — the wind-up dust (the two NextVector2Circular
        // draws) is a documented skip.
        if npc.ai[1] >= 30.0 {
            // `:19356-19358` — `noTileCollide = true`, `collideX/Y = false`.
            // The flags are reset faithfully; the raw integration keeps
            // them false (the dash exit below).
            npc.collide_x = false;
            npc.collide_y = false;
            npc.ai[0] = 2.0;
            npc.ai[1] = 0.0;
            // `:19361-19378` — the dash speed: base 12, +4/+4 under the own
            // half/quarter life, +10 while the body is damaged, x(num+3)/4,
            // clamped at 48.
            let mut num6 = 12.0f32;
            if npc.hp < npc.max_hp / 2 {
                num6 += 4.0;
            }
            if npc.hp < npc.max_hp / 4 {
                num6 += 4.0;
            }
            if world.golem_hp_ratio < 1.0 {
                num6 += 10.0;
            }
            num6 *= (num + 3.0) / 4.0;
            if num6 > 48.0 {
                num6 = 48.0;
            }
            // `:19379-19384` — the dash aims from the SNAPPED centre (the
            // anchor, after `:19334`) at the player, normalized to num6.
            let ddx = pcx - anchor_x;
            let ddy = pcy - anchor_y;
            let d = (ddx * ddx + ddy * ddy).sqrt().max(0.001);
            let num9 = num6 / d;
            npc.vx = ddx * num9;
            npc.vy = ddy * num9;
            // `:19385-19389` — the dash rotation: the default aims ALONG
            // the velocity (`atan2(vy, vx)`); the type-247 (left) arm aims
            // AGAINST it (`atan2(-vy, -vx)`) — the mirror of state 0.
            npc.rotation = if left {
                (-npc.vy).atan2(-npc.vx)
            } else {
                npc.vy.atan2(npc.vx)
            };
        }
    } else if npc.ai[0] == 2.0 {
        // The dash (`:19392-19449`). The good-world torch break
        // (`:19394-19411`) is skipped.
        npc.ai[1] += 1.0;
        // `:19413-19416` — the Item14 sound at `ai[1] == 1` is skipped;
        // `:19417-19422` — the `Next(2)`-gated dash dust is skipped (the
        // arm's only RNG draws, hence RNG-free).
        // `:19423-19444` — `noTileCollide = false` once past the player
        // along the dominant velocity axis: implicit/no-effect here (the
        // fists integrate raw, so no tile collision flags ever set).
        // `:19445-19449` — the exit: `num2 > 700 || collideX || collideY`.
        // The flags stay false under the raw integration, so the 700-px
        // anchor distance is the live exit (ANALYSIS deviation).
        if num2 > 700.0 || npc.collide_x || npc.collide_y {
            // `:19447` — `noTileCollide = true` (implicit).
            npc.ai[0] = 0.0;
        }
    }
    // `:19451-19502` — the state-3 hover is dead in the vanilla AI (no
    // code sets `ai[0] = 3` for the fists) and is not carried (ANALYSIS).

    // The raw `noTileCollide`/`noGravity` step + the off-world kill — the
    // `update_golem` shared-tail shape the fists rode pre-axis.
    npc.facing = if npc.vx >= 0.0 { 1 } else { -1 };
    npc.sprite_dir = npc.facing;
    npc.x += npc.vx;
    npc.y += npc.vy;
    if npc.y > world.height as f32 * TILE_PX + 400.0 {
        npc.hp = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// An air world with the Golem anchor hoisted (the fists integrate
    /// raw, so no tiles are needed). The body's centre sits at (1600,1600).
    fn anchor_world() -> World {
        let mut w = World::new("t".into(), 200, 200);
        w.golem_present = true;
        w.golem_x = 100.0 * 16.0;
        w.golem_y = 100.0 * 16.0;
        w.golem_vx = 0.0;
        w.golem_hp_ratio = 1.0;
        w
    }

    fn fist(kind: NpcKind, x: f32, y: f32) -> Npc {
        let mut n = Npc::new(kind, x, y, 1, 0.0);
        n.hp = n.max_hp;
        n
    }

    #[test]
    fn s1_d_3_the_fist_fades_in_over_26_ticks() {
        // `SetDefaults` `alpha = 255` (`:12204`) + the `alpha -= 10` fade
        // (`:19247-19255`): 26 ticks to fully visible.
        let w = anchor_world();
        let player = crate::player::Player::spawn(&w);
        let mut f = fist(NpcKind::GolemFistLeft, 100.0 * 16.0, 100.0 * 16.0);
        f.alpha = 255;
        for t in 1..=26u16 {
            ai_047_golem_fist(&mut f, &w, &player);
            let want = 255u8.saturating_sub((t * 10).min(255) as u8);
            assert_eq!(f.alpha, want, "the fade at tick {t}");
        }
        assert_eq!(f.alpha, 0, "the fade completes at 26 ticks");
    }

    #[test]
    fn s1_d_3_the_fist_dies_without_the_body() {
        // `:19242-19246` — `golemBoss < 0` => the 9999 self-strike.
        let mut w = anchor_world();
        w.golem_present = false;
        let player = crate::player::Player::spawn(&w);
        let mut f = fist(NpcKind::GolemFistLeft, 100.0 * 16.0, 100.0 * 16.0);
        ai_047_golem_fist(&mut f, &w, &player);
        assert_eq!(f.hp, 0, "the bodyless fist strikes itself");
    }

    #[test]
    fn s1_d_3_the_left_fist_side_gate_blocks_the_right_player() {
        // `:19308-19317` — the left fist only dashes when the player stands
        // left of `Center.X + 100`. A player far to the right keeps the
        // follow state forever (the cadence resets each 60).
        let w = anchor_world();
        let mut player = crate::player::Player::spawn(&w);
        player.x = 130.0 * 16.0;
        player.y = 100.0 * 16.0;
        // The left fist's anchor centre is 1600 - 84 = 1516.
        let mut f = fist(NpcKind::GolemFistLeft, 1490.0, 100.0 * 16.0);
        for _ in 0..400 {
            ai_047_golem_fist(&mut f, &w, &player);
        }
        assert_eq!(f.ai[0], 0.0, "the blocked gate keeps the follow state");
    }

    #[test]
    fn s1_d_3_the_fist_follows_wind_up_and_dash() {
        // `:19231-19503` — the follow to the anchor, the 60-contact into the
        // 30 wind-up, the 12-speed dash. The player stands LEFT of the left
        // fist's anchor so the side gate lets the transition through.
        let w = anchor_world();
        let mut player = crate::player::Player::spawn(&w);
        player.x = 60.0 * 16.0;
        player.y = 100.0 * 16.0;
        let mut f = fist(NpcKind::GolemFistLeft, 1490.0, 100.0 * 16.0);
        // The follow pulls toward the anchor (to the right).
        ai_047_golem_fist(&mut f, &w, &player);
        assert!(f.vx > 0.0, "the follow pulls toward the anchor");
        // 60 contact + 30 wind-up => the dash opens on the 92nd tick.
        for _ in 0..91 {
            ai_047_golem_fist(&mut f, &w, &player);
        }
        assert_eq!(f.ai[0], 2.0, "the dash state after the wind-up");
        let sp = (f.vx * f.vx + f.vy * f.vy).sqrt();
        assert!((sp - 12.0).abs() < 2.0, "the 12-speed lunge ({sp})");
    }

    #[test]
    fn s1_d_3_the_damaged_body_speeds_the_cadence_and_dash() {
        // `:19301-19304` / `:19370-19373` — the damaged body adds +10 to the
        // cadence (11x) and +10 to the dash speed (12 -> 22).
        let mut w = anchor_world();
        w.golem_hp_ratio = 0.5;
        let mut player = crate::player::Player::spawn(&w);
        player.x = 60.0 * 16.0;
        player.y = 100.0 * 16.0;
        let mut f = fist(NpcKind::GolemFistLeft, 1490.0, 100.0 * 16.0);
        let mut transition_tick = None;
        for t in 0..45 {
            ai_047_golem_fist(&mut f, &w, &player);
            if f.ai[0] == 1.0 && transition_tick.is_none() {
                transition_tick = Some(t);
            }
        }
        assert_eq!(f.ai[0], 2.0, "the dash state after the wind-up");
        let sp = (f.vx * f.vx + f.vy * f.vy).sqrt();
        assert!((sp - 22.0).abs() < 2.0, "the 12+10 damaged lunge ({sp})");
        assert!(
            transition_tick.expect("the transition fires") < 10,
            "the 11x cadence is fast"
        );
    }

    #[test]
    fn s1_d_3_the_state0_rotation_aims_per_type() {
        // `:19324-19328` — out of range, the left (247) rotation aims ALONG
        // the velocity, the right (248) AGAINST it. The fist starts well
        // below its anchor so the follow is out of the contact band.
        let w = anchor_world();
        let player = crate::player::Player::spawn(&w);
        // Left anchor centre (1516, 1591); the fist 109 px below it.
        let mut fl = fist(NpcKind::GolemFistLeft, 1516.0 - 20.0, 1700.0 - 15.0);
        ai_047_golem_fist(&mut fl, &w, &player);
        assert_eq!(fl.ai[0], 0.0);
        assert!(
            (fl.rotation - fl.vy.atan2(fl.vx)).abs() < 1e-4,
            "the left fist aims along the velocity"
        );
        // Right anchor centre (1678, 1591).
        let mut fr = fist(NpcKind::GolemFistRight, 1678.0 - 20.0, 1700.0 - 15.0);
        ai_047_golem_fist(&mut fr, &w, &player);
        assert!(
            (fr.rotation - (-fr.vy).atan2(-fr.vx)).abs() < 1e-4,
            "the right fist aims against the velocity"
        );
    }

    #[test]
    fn s1_d_3_the_dash_exits_past_700px() {
        // `:19445-19449` — the dash returns to the follow once the anchor
        // distance passes 700 (the collideX/Y terms stay false under the
        // raw integration).
        let w = anchor_world();
        let player = crate::player::Player::spawn(&w);
        // Park the fist 800 px from its anchor, already dashing.
        let mut f = fist(NpcKind::GolemFistLeft, 1516.0 - 20.0, 1591.0 + 800.0 - 15.0);
        f.ai[0] = 2.0;
        f.ai[1] = 5.0;
        ai_047_golem_fist(&mut f, &w, &player);
        assert_eq!(f.ai[0], 0.0, "the >700 anchor distance ends the dash");
    }

    #[test]
    fn s1_d_3_the_axis_dispatch_arms_the_fists() {
        // The wiring pin (the S1-D-2 axis shape): `Npc::update_rng`'s
        // `match self.kind.ai_style()` claims style 47 for both fists and
        // reaches this module — the bodyless guard's `hp = 0` proves the
        // arm ran (the pre-axis kind match no longer carries the fists).
        assert_eq!(NpcKind::GolemFistLeft.ai_style(), 47);
        assert_eq!(NpcKind::GolemFistRight.ai_style(), 47);
        let mut w = anchor_world();
        w.golem_present = false;
        let mut player = crate::player::Player::spawn(&w);
        let mut rng = tr_core::UnifiedRandom::new(0);
        for kind in [NpcKind::GolemFistLeft, NpcKind::GolemFistRight] {
            let mut f = fist(kind, 100.0 * 16.0, 100.0 * 16.0);
            f.update_rng(&w, &mut player, 1.0 / 60.0, &mut rng);
            assert_eq!(f.hp, 0, "{kind:?} rides the style-47 axis arm");
        }
    }
}
