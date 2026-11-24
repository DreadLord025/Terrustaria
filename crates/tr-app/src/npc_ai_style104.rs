//! The aiStyle-104 family — the DD2 Attacker Test (`NPC.AI()`'s
//! `else if (this.aiStyle == 104)` arm, `NPC.cs:40724-40727`). W4-d33,
//! research/25 row 104, CODE-ONLY MODE (no compiler invoked; the
//! witness tests below are written, not run).
//!
//! ## Census
//!
//! The family has exactly ONE vanilla member: the DD2AttackerTest 547
//! (`NPCID.DD2AttackerTest`, `ID/NPCID.cs:11771`; `SetDefaults` row
//! `NPC.cs:15985-15997`). The census of `this.aiStyle = 104;` across the
//! decompiled tree returns two hits: `NPC.cs:15989` (the `type == 547`
//! branch) and `Projectile.cs:6165` (the projectile-space 565 row — a
//! different machine, out of NPC scope). The port's roster has no
//! DD2AttackerTest kind — the module lands UNWIRED (the
//! `npc_ai_style73.rs` precedent), keyed on the vanilla `npc_type`
//! until the roster wave.
//!
//! ## The machine
//!
//! The WHOLE arm is one line (`:40726`): `this.active = false;` — the
//! developer's DD2 attacker test dummy despawns on its first AI tick.
//! No movement, no draws, no sounds, no netUpdate. The port's `Npc`
//! has no `active` field; the deactivation rides the
//! [`Style104Outcome::died`] seam (the style100 convention — the caller
//! owes the pool removal).
//!
//! ## RNG order (by construction)
//!
//! ZERO draws — the arm touches nothing but `active`.
//!
//! ## FindFrame
//!
//! No case 547 exists in `NPC.FindFrame` (the `case 547:` at
//! `:78377` is `GetNPCInvasionGroup`, a different switch);
//! `Main.npcFrameCount[547] = 1` (`Main.cs:67138`, the 540-549 row
//! `7, 13, 4, 4, 4, 4, 1, 1, 1, 8` — index 547 is the 8th entry).
//! Nothing to port.
#![allow(dead_code)] // wiring deferred — the kind + dispatch arm land with the roster wave

use crate::npc::Npc;

/// The type-547 `SetDefaults` row (`NPC.cs:15985-15997`), verbatim.
/// The row writes NO damage (the `SetDefaults` head default 0 stands).
pub struct Dd2AttackerTestDefaults {
    pub width: i32,
    pub height: i32,
    pub ai_style: i32,
    pub defense: i32,
    pub life_max: i32,
    /// `(SoundType, Style)` — `SoundID.NPCHit11` (`SoundID.cs:1267`).
    pub hit_sound: (i32, i32),
    /// `(SoundType, Style)` — `SoundID.NPCDeath15` (`SoundID.cs:1329`).
    pub death_sound: (i32, i32),
    pub knock_back_resist: f32,
    pub no_gravity: bool,
    pub no_tile_collide: bool,
}

/// `NPC.cs:15985-15997` — `else if (this.type == 547)`.
pub const DD2_ATTACKER_TEST: Dd2AttackerTestDefaults = Dd2AttackerTestDefaults {
    width: 16,
    height: 16,
    ai_style: 104,
    defense: 10,
    life_max: 10,
    hit_sound: (3, 11),
    death_sound: (4, 15),
    knock_back_resist: 1.0,
    no_gravity: true,
    no_tile_collide: true,
};

/// `Main.npcFrameCount[547]` = 1 (`Main.cs:67138`, the 540-549 row).
pub const DD2_ATTACKER_TEST_FRAME_COUNT: u32 = 1;

/// The arm's per-tick side effects beyond the NPC state.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Style104Outcome {
    /// The `active = false` deactivation (`:40726`) — ALWAYS true; the
    /// caller owes the pool removal (the style100 `died` seam).
    pub died: bool,
}

/// One fixed step of the aiStyle-104 arm (`NPC.cs:40724-40727`): the
/// whole body is `this.active = false;`. The NPC state is untouched
/// otherwise — no velocity, no ai, no netUpdate.
pub fn style104_step(_npc: &mut Npc) -> Style104Outcome {
    // `:40726` — this.active = false;
    Style104Outcome { died: true }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::npc::NpcKind;
    use tr_core::UnifiedRandom;

    /// A 547-shaped NPC (the style73 placeholder convention).
    fn dummy(x: f32, y: f32) -> Npc {
        let d = &DD2_ATTACKER_TEST;
        let mut n = Npc::new(NpcKind::Zombie, x, y, 1, 0.0);
        n.size_override = Some((d.width as f32, d.height as f32));
        n.hp = d.life_max;
        n.max_hp = d.life_max;
        n
    }

    /// `NPC.cs:15985-15997` — the SetDefaults row, field by field.
    #[test]
    fn w4_d33_s104_the_defaults_row() {
        let d = &DD2_ATTACKER_TEST;
        assert_eq!((d.width, d.height), (16, 16));
        assert_eq!(d.ai_style, 104);
        assert_eq!((d.defense, d.life_max), (10, 10));
        assert_eq!(d.hit_sound, (3, 11));
        assert_eq!(d.death_sound, (4, 15));
        assert_eq!(d.knock_back_resist, 1.0);
        assert!(d.no_gravity && d.no_tile_collide);
        assert_eq!(DD2_ATTACKER_TEST_FRAME_COUNT, 1);
    }

    /// `:40726` — the whole arm: the death flag fires and NOTHING else
    /// moves (the state snapshot is bit-identical, the stream untouched).
    #[test]
    fn w4_d33_s104_the_one_line_despawn() {
        let mut rng = UnifiedRandom::new(104);
        let mut n = dummy(100.0, 100.0);
        n.vx = 3.0;
        n.vy = -2.0;
        n.ai[0] = 7.0;
        let before = (n.x, n.y, n.vx, n.vy, n.ai[0], n.net_update);
        let out = style104_step(&mut n);
        assert!(out.died, "active = false");
        assert_eq!(
            (n.x, n.y, n.vx, n.vy, n.ai[0], n.net_update),
            before,
            "nothing else moves"
        );
        assert_eq!(
            rng.peek(),
            UnifiedRandom::new(104).peek(),
            "the 104 arm is draw-free"
        );
    }
}
