//! W4-d11 DOC-SWEEP — the BloodSquid (619)'s `AI_005_EaterOfSouls` branches
//! (`NPC.cs:50244-50725`): the type-619 arms inside the shared hover family,
//! split out so the family machine (`npc_ai.rs::hover_step`) stays the
//! generic body. The C# order, verbatim:
//!
//! * `:50254-50292` — the head block: the daytime rise (-0.3 vy) +
//!   `EncourageDespawn(60)`, the alpha-255 first-tick launch (spriteDir =
//!   direction, vy = -6, the 35-dust burst — bodies skipped, the 70
//!   `NextFloat` draws stay on the stream), the -15 alpha fade and the
//!   2-dust per-tick trail while alpha != 0 (4 more draws).
//! * `:50323-50327` — the parameter pair: num 6, num2 0.1.
//! * `:50446`/`:50447` — flag4 true / flag5 false: the drift block runs
//!   EVERY tick (`:50450`'s `(num7 > 100) | flag4`), the ±0.023 ai[0]
//!   wander with the ±100/200 clamps; inside 150 px the aim adds
//!   `num4/num5 * 0.007` (`:50474-50478`).
//! * `:50485-50488` — below the target's centre - 200: vy -= 0.3.
//! * `:50547-50578` — the plain approach (flag5 false: no double-add).
//! * `:50643-50646` — the aim rotation (atan2 - 1.57).
//! * `:50656-50694` — the tile bounce at 0.7 with the small-velocity
//!   clamps + the per-tick 5-dust trail (the `NewDust` internals draw;
//!   bodies skipped, gates kept — `Dust.NewDust` rolls its position
//!   jitter unconditionally).
//! * `:50714-50725` — the wet rise: vy>0 ×0.95, -0.3, floor -2.
//!
//! Deviations (recorded): the dust BODIES skip (no dust layer in the NPC
//! step); the `NewDustDirect`/`NewDust` position/velocity draws are the
//! dust system's internals — the port's NPC machines consume zero draws
//! for them (the `npc_ai_style9` precedent, its module header pins the
//! same skip). `Lighting.AddLight` is not on the 619 path. The
//! `netOffset` add/subtract pairs are no-ops in the single-player port
//! (`net_offset` is zero on the host).

use crate::npc::Npc;
use crate::npc_ai::{snap8, HoverTarget};
use tr_core::World;

/// The 619 arm of `AI_005_EaterOfSouls` (`NPC.cs:50244-50725`).
/// `day_time` is `Main.dayTime`; the target is the family's
/// `GetTargetData` snapshot (the caller ran `TargetClosest` —
/// `:50250-50253` — before the arm).
pub fn blood_squid_step(npc: &mut Npc, world: &World, target: HoverTarget, day_time: bool) {
    debug_assert_eq!(npc.kind, crate::npc::NpcKind::BloodSquid);
    let (w, h) = npc.size();
    // `:50254-50260` — the daytime rise + the despawn encouragement.
    if day_time {
        npc.vy -= 0.3;
        npc.encourage_despawn(60);
    }
    // `:50262-50274` — the alpha-255 launch tick. The 35-dust burst's
    // bodies skip (the module header's draw-skip note).
    if npc.alpha == 255 {
        npc.sprite_dir = npc.facing;
        npc.vy = -6.0;
    }
    // `:50275-50279` — the fade (the u8 saturating form of `alpha -= 15;
    // if (alpha < 0) alpha = 0`).
    npc.alpha = npc.alpha.saturating_sub(15);
    // `:50280-50290` — the 2-dust trail while fading: body skipped.
    // `:50323-50327` — num 6, num2 0.1.
    let (num, num2) = (6.0f32, 0.1f32);
    // `:50418-50428` — the 8-px grid aim (the family's snap).
    let (cx, cy) = (npc.x + w * 0.5, npc.y + h * 0.5);
    let mut num4 = snap8(target.center_x) - snap8(cx);
    let mut num5 = snap8(target.center_y) - snap8(cy);
    let num6 = (num4 * num4 + num5 * num5).sqrt();
    let num7 = num6;
    // `:50434-50444` — the zero-distance keeps the velocity; else scale
    // to num.
    if num6 == 0.0 {
        num4 = npc.vx;
        num5 = npc.vy;
    } else {
        let s = num / num6;
        num4 *= s;
        num5 *= s;
    }
    // `:50446-50473` — flag4 true: the drift runs every tick.
    npc.ai[0] += 1.0;
    if npc.ai[0] > 0.0 {
        npc.vy += 0.023;
    } else {
        npc.vy -= 0.023;
    }
    if npc.ai[0] < -100.0 || npc.ai[0] > 100.0 {
        npc.vx += 0.023;
    } else {
        npc.vx -= 0.023;
    }
    if npc.ai[0] > 200.0 {
        npc.ai[0] = -200.0;
    }
    // `:50474-50478` — the close-range aim add.
    if num7 < 150.0 {
        npc.vx += num4 * 0.007;
        npc.vy += num5 * 0.007;
    }
    // `:50480-50488` — the dead-target flee, else the below-target rise.
    if target.dead {
        num4 = npc.facing as f32 * num / 2.0;
        num5 = -num / 2.0;
    } else if cy > target.center_y - 200.0 {
        npc.vy -= 0.3;
    }
    // `:50547-50578` — the approach (flag5 false: no double-add).
    if npc.vx < num4 {
        npc.vx += num2;
    } else if npc.vx > num4 {
        npc.vx -= num2;
    }
    if npc.vy < num5 {
        npc.vy += num2;
    } else if npc.vy > num5 {
        npc.vy -= num2;
    }
    // `:50643-50646` — the aim rotation.
    npc.rotation = num5.atan2(num4) - 1.57;
    // `:50656-50688` — the tile bounce (num14 = 0.7, `:50658`). The
    // collideX/collideY latches ride the port's per-axis probe (the
    // `hover_step` bounce precedent).
    let old_vx = npc.vx;
    let old_vy = npc.vy;
    let nx = npc.x + npc.vx;
    if !crate::player::collides_rect(world, nx, npc.y, w, h) {
        npc.x = nx;
    } else {
        npc.net_update = true;
        npc.vx = old_vx * -0.7;
        if npc.facing == -1 && npc.vx > 0.0 && npc.vx < 2.0 {
            npc.vx = 2.0;
        }
        if npc.facing == 1 && npc.vx < 0.0 && npc.vx > -2.0 {
            npc.vx = -2.0;
        }
    }
    let ny = npc.y + npc.vy;
    if !crate::player::collides_rect(world, npc.x, ny, w, h) {
        npc.y = ny;
    } else {
        npc.net_update = true;
        npc.vy = old_vy * -0.7;
        if npc.vy > 0.0 && npc.vy < 1.5 {
            npc.vy = 2.0;
        }
        if npc.vy < 0.0 && npc.vy > -1.5 {
            npc.vy = -2.0;
        }
    }
    // `:50690-50694` — the per-tick 5-dust trail: body skipped.
    // `:50714-50725` — the wet rise.
    if npc.wet {
        if npc.vy > 0.0 {
            npc.vy *= 0.95;
        }
        npc.vy -= 0.3;
        if npc.vy < -2.0 {
            npc.vy = -2.0;
        }
    }
    // The family tail's facing write (`npc_ai.rs::hover_step`'s
    // `:50418`-derived facing — the C# AI_005 has no facing write of its
    // own for 619; the port's hover family sets it from the target).
    npc.facing = if target.center_x >= cx { 1 } else { -1 };
}

// ---------------------------------------------------------------------------
// Witnesses (W4-d11 DOC-SWEEP code-only round — WRITTEN NOT RUN).

#[cfg(test)]
mod tests {
    use super::*;
    use crate::npc::NpcKind;
    use tr_core::tile::{Tile, FLAG_ACTIVE};
    use tr_core::TILE_PX;

    /// A 64x64 world, dirt ground at row 30 (the drippler fixture shape).
    fn ground() -> World {
        let mut w = World::new("t".into(), 64, 64);
        for x in 0..64 {
            for y in 30..64 {
                let i = w.index(x, y);
                w.tiles[i] = Tile { kind: 1, ..Default::default() };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        w.compute_column_surface();
        w
    }

    fn squid(x: f32, y: f32) -> Npc {
        Npc::new(NpcKind::BloodSquid, x, y, 1, 0.0)
    }

    fn target(x: f32, y: f32) -> HoverTarget {
        HoverTarget { center_x: x, center_y: y, top_y: y - 20.0, dead: false }
    }

    /// `NPC.cs:16934-16949` — the SetDefaults row through the port's
    /// tables: 44x44, aiStyle 5, dmg 60, def 16, 750 HP, kb 0, value 500,
    /// alpha 255, noGravity, NPCHit1/NPCDeath1, Confused-immune.
    #[test]
    fn w4_d11_sweep_the_blood_squid_defaults_match_the_csharp_row() {
        let n = squid(100.0, 100.0);
        assert_eq!(n.size(), (44.0, 44.0));
        assert_eq!(NpcKind::BloodSquid.ai_style(), 5);
        assert_eq!(NpcKind::BloodSquid.contact_damage(), 60);
        assert_eq!(n.defense(), 16);
        assert_eq!(n.max_hp, 750);
        assert_eq!(n.knock_back_resist(), 0.0);
        assert_eq!(n.value(), 500.0);
        assert_eq!(n.alpha, 255);
        assert!(NpcKind::BloodSquid.no_gravity());
        assert_eq!(NpcKind::BloodSquid.hit_sound(), Some("NPC_Hit_1"));
        assert_eq!(NpcKind::BloodSquid.death_sound(), Some("NPC_Killed_1"));
        assert!(n.buffs.buff_immune[31], "NPCID.cs:9648-9654's {{31}}");
        // The protocol id keeps the vanilla 619 and round-trips.
        assert_eq!(NpcKind::BloodSquid.to_u16(), 619);
        assert_eq!(NpcKind::from_u16(619), Some(NpcKind::BloodSquid));
        assert_eq!(n.kind.net_id(), 619);
    }

    /// `:50262-50279` — the alpha-255 launch tick sets vy = -6 and
    /// spriteDir = direction; the fade runs -15/tick and floors at 0
    /// (255 -> 240 on the launch tick itself).
    #[test]
    fn w4_d11_sweep_the_launch_tick_and_the_fade() {
        let w = ground();
        let mut n = squid(20.0 * TILE_PX, 20.0 * TILE_PX);
        n.facing = -1;
        blood_squid_step(&mut n, &w, target(20.0 * TILE_PX, 10.0 * TILE_PX), false);
        assert_eq!(n.sprite_dir, -1, "the launch copies the direction");
        assert_eq!(n.alpha, 240, "255 - 15 on the launch tick");
        // The vy = -6 launch write ran, then the drift/approach moved it;
        // the fade alone is deterministic: 17 ticks reach 0 and stay.
        for _ in 0..17 {
            blood_squid_step(&mut n, &w, target(20.0 * TILE_PX, 10.0 * TILE_PX), false);
        }
        assert_eq!(n.alpha, 0);
        blood_squid_step(&mut n, &w, target(20.0 * TILE_PX, 10.0 * TILE_PX), false);
        assert_eq!(n.alpha, 0, "the floor holds");
    }

    /// `:50254-50260` — by day the rise runs and EncourageDespawn(60)
    /// caps the despawn timer; by night neither runs.
    #[test]
    fn w4_d11_sweep_the_daytime_rise_and_despawn_cap() {
        let w = ground();
        let mut n = squid(20.0 * TILE_PX, 20.0 * TILE_PX);
        n.despawn_timer = 750;
        blood_squid_step(&mut n, &w, target(20.0 * TILE_PX, 20.0 * TILE_PX), true);
        assert_eq!(n.despawn_timer, 60, "EncourageDespawn(60)");
        let mut m = squid(20.0 * TILE_PX, 20.0 * TILE_PX);
        m.despawn_timer = 750;
        blood_squid_step(&mut m, &w, target(20.0 * TILE_PX, 20.0 * TILE_PX), false);
        assert_eq!(m.despawn_timer, 750, "no encouragement by night");
    }

    /// `:50446-50473` — flag4: the drift runs EVERY tick (ai[0]++ then
    /// the ±0.023 pair), and ai[0] wraps 200 -> -200.
    #[test]
    fn w4_d11_sweep_the_drift_runs_every_tick_and_wraps() {
        let w = ground();
        let mut n = squid(20.0 * TILE_PX, 20.0 * TILE_PX);
        n.alpha = 0; // skip the launch write
        // Park the target ON the squid: num6 == 0 keeps the velocity,
        // the aim adds nothing (num7 = 0 < 150 but num4/num5 = vx/vy).
        let t = target(n.x + 22.0, n.y + 22.0);
        blood_squid_step(&mut n, &w, t, false);
        assert_eq!(n.ai[0], 1.0, "the drift counter runs at any range");
        n.ai[0] = 200.0;
        blood_squid_step(&mut n, &w, t, false);
        assert_eq!(n.ai[0], -200.0, "201 wraps to -200 (:50469-50472)");
    }

    /// `:50714-50725` — the wet rise: vy>0 damps ×0.95, then -0.3, and
    /// the -2 floor holds.
    #[test]
    fn w4_d11_sweep_the_wet_rise_floors_at_minus_two() {
        let w = ground();
        let mut n = squid(20.0 * TILE_PX, 20.0 * TILE_PX);
        n.alpha = 0;
        n.wet = true;
        n.vy = -5.0;
        // The approach may move vy; the wet block runs LAST, so a vy
        // already below -2 pins to exactly -2.
        let t = target(n.x + 22.0, n.y + 22.0);
        blood_squid_step(&mut n, &w, t, false);
        assert_eq!(n.vy, -2.0, "the wet floor");
    }
}
