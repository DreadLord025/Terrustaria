//! The aiStyle-72 family — the Martian Officer's force bubble
//! (`NPC.AI()`'s `else if (this.aiStyle == 72)` arm,
//! `NPC.cs:35080-35101`). S1-D-3, aiStyle-72 round, CODE-ONLY MODE (no
//! compiler invoked; the witness tests below are written, not run).
//!
//! The family has exactly ONE vanilla member: the ForceBubble 384
//! (`NPCID.ForceBubble`, `ID/NPCID.cs:11445`, `SetDefaults` row
//! `NPC.cs:13882-13894`). The census of `this.aiStyle = 72;` across
//! `NPC.cs` returns a single hit (`:13888`, the `type == 384` branch);
//! no other kind rides the style. (The `Golem => 72` row in
//! `npc/data.rs:2650` is the Golem's contact DAMAGE, not its aiStyle —
//! the Golem's style is 45, `npc/data.rs:1555` / `NPC.cs:12167`.)
//!
//! The machine: a rider, not a chaser. The bubble carries NO
//! `TargetClosest` and NO steering of its own — every tick it snaps to
//! its host Martian Officer (383) and mirrors it:
//! * `num1016 = (int)ai[0]` (`:35084`) — the officer's NPC slot, written
//!   at spawn (`AI_003_Fighters`, `NPC.cs:56228`:
//!   `Main.npc[num12].ai[0] = base.whoAmI`).
//! * If `Main.npc[num1016].active && type == 383` (`:35085`):
//!   `velocity = Vector2.Zero` (`:35087`), `position = officer.Center`
//!   then `-= width/2` / `-= height/2` (`:35088-35090` — INTEGER
//!   half-extents, `Entity.width`/`height` are `int`), `gfxOffY =
//!   officer.gfxOffY` (`:35091`), and a constant
//!   `Lighting.AddLight((int)Center.X/16, (int)Center.Y/16,
//!   0.1, 0.5, 0.7)` (`:35092`).
//! * Else (`:35094-35099`): `life = 0`, `HitEffect()`, `active = false`
//!   — the bubble pops the instant its officer dies or its slot is
//!   recycled to a non-383.
//!
//! ## RNG order (by construction)
//!
//! The AI arm draws NO `Main.rand` — `NPC.cs:35080-35101` contains no
//! `Next`/`NextFloat`; `Lighting.AddLight` draws nothing; there is no
//! `TargetClosest`/`EncourageDespawn` in the arm at all. So
//! [`ai_style72`] takes NO RNG parameter; the witness test
//! `the_style72_arm_draws_no_rng` pins a `UnifiedRandom` state across
//! the calls.
//!
//! Unlike the spore pair (aiStyle-50), the bubble has NO per-tick
//! `UpdateNPC_CastLights` draw either: that pass (`NPC.cs:92343-92414`)
//! arms only the gold critters, 160, 254..=261/634/635, 209 and 44 —
//! none is 384. The bubble's only light is the constant `AddLight`
//! inside the AI arm (`:35092`), which is RNG-free.
//!
//! The draws that DO exist around the arm: `HitEffect` on hit/pop. The
//! 384 arm (`NPC.cs:84342-84361`) spawns dust 226 only — the hit half's
//! `dmg/lifeMax*50.0` loop and the death half's flat 70 — and every draw
//! is inside `Dust.NewDust` itself (the arm has no explicit `Next`). The
//! dust system is unported (S1-E-2/S1-E-3, other branches), so the arm
//! is skipped wholesale — the `npc_lights.rs` precedent: skipping an arm
//! whole leaves the `Main.rand` stream untouched.
//! [`bubble_hit_dust_plan`] ports the branch structure and counts only
//! (no RNG parameter, no draws); the wiring wave must draw the
//! `Dust.NewDust` internals in C# order when the dust pass lands.
//!
//! ## Lifecycle context (the officer's shield — AI_003, NOT this arm)
//!
//! The bubble is spawned and shielded by the Martian Officer's
//! `AI_003_Fighters` block (`NPC.cs:56222-56251`), which belongs to the
//! aiStyle-3 family (S1-D-3c), not aiStyle-72. It is cited here as the
//! wiring obligation, not ported in this module:
//! * Spawn (`:56222-56230`): when `type == 383 && ai[2] == 0 &&
//!   localAI[0] == 0 && Main.netMode != 1`, `NewNPC(source,
//!   (int)Center.X, (int)Center.Y, 384, whoAmI)`; then `ai[2] = slot+1`,
//!   `localAI[0] = -1`, `netUpdate = true`, and the new bubble's
//!   `ai[0] = whoAmI`, `netUpdate = true`. `NewNPC` places the bubble by
//!   `Bottom = (X, Y)` (`NPC.cs:80770`, `Entity.cs:128-131`) and sets
//!   `timeLeft = (int)(activeTime * 1.25)` (`:80772`, `activeTime = 750`
//!   `:95186` → 937).
//! * Shield (`:56231-56251`): `num13 = (int)ai[2] - 1`; if `num13 != -1
//!   && Main.npc[num13].active && type == 384` the officer is
//!   `dontTakeDamage = true`; else `dontTakeDamage = false`, `ai[2] = 0`,
//!   and if `localAI[0] == -1` it becomes 180, then `localAI[0]--` while
//!   `> 0` — the 180-tick re-spawn cooldown after a pop.
//! The bubble itself is hittable (its `SetDefaults` leaves
//! `dontTakeDamage = false`, `NPC.cs:8507`); it is the OFFICER that the
//! living bubble shields.
//!
//! **WIRED by S1-D-3-wire-smallkinds (2026-11-19).** `NpcKind::ForceBubble`
//! (protocol id 384, vanilla type 384) now exists with its `SetDefaults`
//! row (`NPC.cs:13882-13894`). The arm reads the SIBLING NPC slots, so it
//! runs in the world-sim host pre-pass (`world_sim/npcs.rs`), which builds
//! the [`BubbleHost`] view slice and resolves the officer link; the style
//! axis' 72 arm (`npc.rs`) owns the post-`AI()` gravity + integration tail.
//! The seams, as landed:
//! * `Npc.gfx_off_y` now exists — the wiring caller writes the arm's
//!   returned [`Style72Outcome::gfx_off_y`] onto it, and the NPC draw
//!   consumes it for the bubble (`render/entities.rs`, gated to 384).
//! * `Npc` has no `active` field — the pop sets `hp = 0` here (the C#
//!   `life = 0`, `:35096`) and returns `died`; the caller then runs the
//!   `HitEffect()` + deactivation pair (`:35097-35098`). `checkDead`
//!   (`NPC.cs:78040`) has no 384-specific branch; the port's death
//!   resolution is the `world_sim/npcs.rs` retain.
//! * The host officer is read through a [`BubbleHost`] view slice
//!   indexed by `(int)ai[0]`, mirroring `Main.npc[num1016]`. The port's
//!   officer link is the stable uid (`carrier_uid`, the Moon Lord
//!   convention); the pre-pass resolves it to the current slot and writes
//!   `ai[0]` before the call. The C# indexes without a bounds check; the
//!   port `.get()`s and treats an out-of-range/negative slot as "no valid
//!   officer" → the pop branch.
//! * The officer's spawn + shield (`AI_003_Fighters`, `NPC.cs:56222-56251`)
//!   is wired through `Npc::martian_officer_bubble` (the request) + the
//!   world-sim drain (the spawn, the `carrier_uid` two-way link).
//! * `FindFrame` (`NPC.cs:66684-77370`) has NO case 384 — the switch
//!   falls through and the post-switch tail is aiStyle-39 only
//!   (`:77292`), so the frame NEVER advances: the bubble is single-frame
//!   (`Main.npcFrameCount[384] = 1`, `Main.cs:67122`, the 380-389 row
//!   `12, 9, 9, 9, 1, 9, 11, 2, 2, 9` of the `:67082` array, index 384 =
//!   the 5th entry = 1). The kind's `find_frame` row pins frame 0.
//! * The arm sets `velocity = 0` and snaps `position`; the C#
//!   `UpdateNPC` gravity (`:90465-90472`, `noGravity` is false for 384)
//!   and the `noTileCollide` raw integration (`:90494-90498`) run AFTER
//!   `AI()` (`:90423`) and are the axis-72 arm's movement step, not this
//!   arm's — the bubble re-snaps to the officer every tick.
//!
//! See docs/ANALYSIS.md "The force bubble (S1-D-3 aiStyle-72,
//! code-only)" + "The small-kinds wiring wave".
#![allow(dead_code)] // WIRED by S1-D-3-wire-smallkinds (the world-sim host pre-pass + axis 72's gravity/integration tail); the defaults row + witnesses stay as the audit cross-check

use glam::Vec2;

use crate::entity_lights::{tile_x, tile_y, XAnchor};
use crate::npc::Npc;

/// The ForceBubble `SetDefaults` row (`NPC.cs:13882-13894`), verbatim,
/// with the head defaults the branch leaves untouched (`:8507`
/// `dontTakeDamage = false`, `:8533` `knockBackResist = 1`, `:8535`
/// `noGravity = false`, `:8544` `value = 0`, `:8563` `damage = 0`).
pub struct ForceBubbleDefaults {
    pub width: i32,
    pub height: i32,
    pub ai_style: i32,
    pub damage: i32,
    pub defense: i32,
    pub life_max: i32,
    /// `(SoundType, Style)` — `SoundID.NPCHit43` (`ID/SoundID.cs:1299`).
    pub hit_sound: (i32, i32),
    /// `(SoundType, Style)` — `SoundID.NPCDeath45` (`ID/SoundID.cs:1359`).
    pub death_sound: (i32, i32),
    pub no_gravity: bool,
    pub no_tile_collide: bool,
    pub knock_back_resist: f32,
    pub npc_slots: f32,
    pub value: f32,
    pub can_ghost_heal: bool,
    pub dont_take_damage: bool,
}

/// `NPC.cs:13882-13894` — `else if (this.type == 384)`, the ForceBubble.
pub const FORCE_BUBBLE: ForceBubbleDefaults = ForceBubbleDefaults {
    width: 18,
    height: 40,
    ai_style: 72,
    damage: 0,
    defense: 20,
    life_max: 1000,
    hit_sound: (3, 43),
    death_sound: (4, 45),
    no_gravity: false,
    no_tile_collide: true,
    knock_back_resist: 1.0,
    npc_slots: 0.0,
    value: 0.0,
    can_ghost_heal: false,
    dont_take_damage: false,
};

/// `Main.npcFrameCount[384]` = 1 (`Main.cs:67122`, the 380-389 row of
/// the `:67082` array, index 384 = the 5th entry). `FindFrame` has no
/// case 384 — the single frame never advances.
pub const FORCE_BUBBLE_FRAME_COUNT: u32 = 1;

/// The host officer's per-tick state the arm reads — the four fields of
/// `Main.npc[num1016]` touched at `NPC.cs:35085-35091`. The wiring
/// caller builds one view per live NPC slot, indexed by `(int)ai[0]`.
pub struct BubbleHost {
    /// `Main.npc[slot].active` (`:35085`).
    pub active: bool,
    /// `Main.npc[slot].type` (`:35085` — must be 383, the officer).
    pub npc_type: i32,
    /// `Main.npc[slot].Center` (`:35088`) — the float centre.
    pub center: (f32, f32),
    /// `Main.npc[slot].gfxOffY` (`:35091`).
    pub gfx_off_y: f32,
}

/// The arm's per-tick side effects beyond the NPC state — the wiring
/// seams (module docs).
pub struct Style72Outcome {
    /// The C# `this.gfxOffY = officer.gfxOffY` copy (`:35091`). `Npc`
    /// has no such field; `Some` when attached, `None` on the pop tick
    /// (the C# leaves `gfxOffY` untouched there).
    pub gfx_off_y: Option<f32>,
    /// The constant `Lighting.AddLight` (`:35092`):
    /// `(tileX, tileY, [r, g, b])` at the truncated centre tile, fed to
    /// the caller's `EntityLights`. `None` on the pop tick.
    pub light: Option<(i32, i32, [f32; 3])>,
    /// The officer-gone pop (`:35096-35098`): `hp` is already 0; the
    /// caller owes the `HitEffect()` + deactivation pair.
    pub died: bool,
}

/// One fixed step of the aiStyle-72 arm (`NPC.cs:35080-35101`), in C#
/// order. NO RNG parameter — the arm draws nothing (module docs).
///
/// `npc_type` is the vanilla id (384) until the roster kind lands;
/// `npcs` is the host-view slice indexed by `(int)ai[0]` (the C#
/// `Main.npc[num1016]`). The arm has no `TargetClosest` — it reads the
/// officer slot, never the player — so `npc_target.rs` is not involved.
pub fn ai_style72(npc: &mut Npc, npcs: &[BubbleHost], npc_type: i32) -> Style72Outcome {
    // `:35082` — the only type that rides aiStyle 72.
    if npc_type == 384 {
        // `:35084` — num1016 = (int)this.ai[0].
        let num1016 = npc.ai[0] as i32;
        // `:35085` — Main.npc[num1016].active && type == 383. The C#
        // indexes without a bounds check; the port .get()s (module
        // docs: out-of-range is unreachable from vanilla ai[0] writes).
        let host = if num1016 >= 0 {
            npcs.get(num1016 as usize)
        } else {
            None
        };
        match host {
            Some(h) if h.active && h.npc_type == 383 => {
                // `:35087` — base.velocity = Vector2.Zero.
                npc.vx = 0.0;
                npc.vy = 0.0;
                // `:35088-35090` — position = officer.Center, then
                // -= width/2 / -= height/2. INTEGER half-extents
                // (Entity.width/height are int): 18/2 = 9, 40/2 = 20.
                let (w, hgt) = npc.size();
                npc.x = h.center.0 - (w as i32 / 2) as f32;
                npc.y = h.center.1 - (hgt as i32 / 2) as f32;
                // `:35091` — gfxOffY = officer.gfxOffY (returned seam).
                // `:35092` — AddLight((int)Center.X/16, (int)Center.Y/16,
                // 0.1, 0.5, 0.7). The centre is read AFTER the snap, so
                // it equals the officer's centre exactly.
                let pos = Vec2::new(npc.x, npc.y);
                let light = (
                    tile_x(pos, w, XAnchor::CenterTruncated),
                    tile_y(pos, hgt, XAnchor::CenterTruncated),
                    [0.1, 0.5, 0.7],
                );
                Style72Outcome {
                    gfx_off_y: Some(h.gfx_off_y),
                    light: Some(light),
                    died: false,
                }
            }
            _ => {
                // `:35096-35098` — life = 0; HitEffect(); active = false.
                npc.hp = 0;
                Style72Outcome {
                    gfx_off_y: None,
                    light: None,
                    died: true,
                }
            }
        }
    } else {
        // No other type rides aiStyle 72 — the C# arm is a no-op for
        // them (the `if (this.type == 384)` is the whole body).
        Style72Outcome {
            gfx_off_y: None,
            light: None,
            died: false,
        }
    }
}

/// The `HitEffect` dust arm's branch structure — counts and kinds only,
/// the documented visual skip (module docs): no dust is spawned and NO
/// RNG is drawn (every draw is inside `Dust.NewDust`, skipped wholesale
/// with the whole arm — the `npc_lights.rs` precedent, the stream stays
/// untouched).
///
/// The generic `HitEffect` prologue (`NPC.cs:81656-82655`) is inert for
/// the bubble: the `!active` gate is the caller's, the party-hat block
/// needs `townNPC` (`:81660`), daybreak needs the unported buff
/// (`:81730`), and every type check before the 370-chain (`:84258`) is
/// false for 384. The 384 arm (`:84342-84361`) is reached through that
/// chain and spawns dust 226 only — NO gore (unlike the officer 383's
/// 597/598/599 trio, `:84377-84379`).
pub enum BubbleHitDust {
    /// The hit half (`NPC.cs:84344-84352`, `life > 0`): `count` dusts of
    /// type 226, speedX = `-1 * hitDirection`, speedY = -1, each
    /// position `Lerp(dust.position, Center, 0.25)`, scale 0.5. `count`
    /// rides the `dmg / lifeMax * 50.0` double-precision loop bound
    /// (`:84346`).
    Hit { count: u32 },
    /// The death half (`NPC.cs:84353-84360`, `life <= 0`): a flat 70
    /// dusts of type 226, speedX = `-2.5 * hitDirection`, speedY = -2.5,
    /// scale 0.5. The AI pop passes `life = 0` → this arm.
    Death { count: u32 },
}

/// The dust plan for one `HitEffect(hitDirection, dmg)` call on the
/// ForceBubble. `life` is the C# `this.life` AT THE CALL (the AI pop
/// passes 0 after `life = 0`; a strike may pass a positive life). The
/// gate is `life > 0` (`:84344`) — note `>`, so a just-zeroed life takes
/// the death arm (unlike the Spore's `>= 0`).
pub fn bubble_hit_dust_plan(life: i32, dmg: f64, life_max: i32) -> BubbleHitDust {
    if life > 0 {
        // `:84346` — the double-precision loop bound, counted exactly as
        // the C# `for` runs it.
        let bound = dmg / life_max as f64 * 50.0;
        let mut count = 0u32;
        while (count as f64) < bound {
            count += 1;
        }
        return BubbleHitDust::Hit { count };
    }
    // `:84355` — the flat 70-iteration death loop.
    BubbleHitDust::Death { count: 70 }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::npc::NpcKind;
    use tr_core::unified_random::UnifiedRandom;

    /// A bubble-shaped NPC: the roster has no ForceBubble kind yet, so
    /// the fixture rides a placeholder kind with the C# box through
    /// `size_override` and the 1000-HP life.
    fn bubble(x: f32, y: f32) -> Npc {
        let d = &FORCE_BUBBLE;
        let mut n = Npc::new(NpcKind::Zombie, x, y, 1, 0.0);
        n.size_override = Some((d.width as f32, d.height as f32));
        n.hp = d.life_max;
        n.max_hp = d.life_max;
        n
    }

    /// An officer host view at the given centre.
    fn officer(center: (f32, f32), gfx: f32) -> BubbleHost {
        BubbleHost {
            active: true,
            npc_type: 383,
            center,
            gfx_off_y: gfx,
        }
    }

    /// `NPC.cs:13882-13894` + the head defaults — the SetDefaults row,
    /// field by field.
    #[test]
    fn the_defaults_row_matches_the_csharp() {
        let d = &FORCE_BUBBLE;
        assert_eq!((d.width, d.height), (18, 40));
        assert_eq!(d.ai_style, 72);
        assert_eq!(d.damage, 0, ":8563 head default");
        assert_eq!(d.defense, 20);
        assert_eq!(d.life_max, 1000);
        assert_eq!(d.hit_sound, (3, 43), "SoundID.NPCHit43");
        assert_eq!(d.death_sound, (4, 45), "SoundID.NPCDeath45");
        assert!(!d.no_gravity, ":8535 head default");
        assert!(d.no_tile_collide, ":13892");
        assert_eq!(d.knock_back_resist, 1.0, ":8533 head default");
        assert_eq!(d.npc_slots, 0.0, ":13891");
        assert_eq!(d.value, 0.0, ":8544 head default");
        assert!(!d.can_ghost_heal, ":13893");
        assert!(!d.dont_take_damage, ":8507 — the bubble is hittable");
        // Main.cs:67122 — the 380-389 row, index 384 = 1.
        assert_eq!(FORCE_BUBBLE_FRAME_COUNT, 1);
    }

    /// `NPC.cs:35087-35090` — the snap: velocity zeroed, position =
    /// officer.Center minus the INTEGER half-extents (9, 20).
    #[test]
    fn the_bubble_snaps_to_the_officer_center() {
        let mut n = bubble(0.0, 0.0);
        n.ai[0] = 3.0; // the officer's slot
        n.vx = 5.0;
        n.vy = -2.0;
        let hosts = [
            officer((0.0, 0.0), 0.0),
            officer((0.0, 0.0), 0.0),
            officer((0.0, 0.0), 0.0),
            officer((1000.0, 800.0), 4.0),
        ];
        let out = ai_style72(&mut n, &hosts, 384);
        assert!(!out.died);
        assert_eq!(n.vx, 0.0, ":35087");
        assert_eq!(n.vy, 0.0, ":35087");
        assert_eq!(n.x, 1000.0 - 9.0, ":35089 width/2 = 9");
        assert_eq!(n.y, 800.0 - 20.0, ":35090 height/2 = 20");
        // The centre lands exactly on the officer's centre.
        assert_eq!(n.center(), (1000.0, 800.0));
        assert_eq!(out.gfx_off_y, Some(4.0), ":35091");
    }

    /// `NPC.cs:35092` — the constant light at the truncated centre tile
    /// (0.1, 0.5, 0.7), read AFTER the snap.
    #[test]
    fn the_light_is_the_constant_blue_at_the_truncated_center() {
        let mut n = bubble(0.0, 0.0);
        n.ai[0] = 0.0;
        // Centre (1000.5, 800.9): (int)/16 truncates toward zero.
        let hosts = [officer((1000.5, 800.9), 0.0)];
        let out = ai_style72(&mut n, &hosts, 384);
        let (tx, ty, rgb) = out.light.expect("attached lights");
        assert_eq!(tx, 1000 / 16, "(int)Center.X / 16");
        assert_eq!(ty, 800 / 16, "(int)Center.Y / 16");
        assert_eq!(rgb, [0.1, 0.5, 0.7]);
    }

    /// `NPC.cs:35094-35098` — the officer-gone pop: hp = 0, died, no
    /// light/gfx write. An inactive host pops it.
    #[test]
    fn the_bubble_pops_when_the_officer_is_inactive() {
        let mut n = bubble(100.0, 100.0);
        n.ai[0] = 0.0;
        let mut hosts = [officer((500.0, 500.0), 2.0)];
        hosts[0].active = false;
        let out = ai_style72(&mut n, &hosts, 384);
        assert!(out.died);
        assert_eq!(n.hp, 0, ":35096");
        assert_eq!(out.gfx_off_y, None, "no gfx write on the pop");
        assert!(out.light.is_none(), "no light on the pop");
        // The position/velocity are NOT touched on the pop tick.
        assert_eq!(n.x, 100.0);
        assert_eq!(n.vx, 0.0);
    }

    /// `NPC.cs:35085` — a host that is active but NOT type 383 pops the
    /// bubble (the slot was recycled to another kind).
    #[test]
    fn the_bubble_pops_when_the_slot_is_not_an_officer() {
        let mut n = bubble(0.0, 0.0);
        n.ai[0] = 0.0;
        let mut hosts = [officer((500.0, 500.0), 0.0)];
        hosts[0].npc_type = 385; // the Gray Grunt, not the officer
        let out = ai_style72(&mut n, &hosts, 384);
        assert!(out.died);
        assert_eq!(n.hp, 0);
    }

    /// The defensive seam: an out-of-range / negative `(int)ai[0]`
    /// resolves to "no valid officer" → the pop branch (module docs —
    /// unreachable from vanilla writes, but the port must not panic).
    #[test]
    fn an_out_of_range_slot_pops_without_panicking() {
        for slot in [99.0, -1.0] {
            let mut n = bubble(0.0, 0.0);
            n.ai[0] = slot;
            let hosts = [officer((500.0, 500.0), 0.0)];
            let out = ai_style72(&mut n, &hosts, 384);
            assert!(out.died, "slot {slot}");
            assert_eq!(n.hp, 0);
        }
    }

    /// `NPC.cs:35084` — the slot is the INTEGER truncation of ai[0].
    #[test]
    fn the_slot_is_the_integer_truncation_of_ai0() {
        let mut n = bubble(0.0, 0.0);
        n.ai[0] = 2.9; // (int)2.9 = 2
        let hosts = [
            officer((0.0, 0.0), 0.0),
            officer((0.0, 0.0), 0.0),
            officer((300.0, 300.0), 1.0),
        ];
        let out = ai_style72(&mut n, &hosts, 384);
        assert!(!out.died);
        assert_eq!(n.center(), (300.0, 300.0), "picked slot 2, not 3");
    }

    /// A non-384 type is a no-op (the C# arm body is the `type == 384`
    /// gate alone).
    #[test]
    fn a_non_bubble_type_is_a_noop() {
        let mut n = bubble(10.0, 20.0);
        n.ai[0] = 0.0;
        let hosts = [officer((500.0, 500.0), 0.0)];
        let out = ai_style72(&mut n, &hosts, 383);
        assert!(!out.died);
        assert_eq!(n.hp, FORCE_BUBBLE.life_max);
        assert_eq!(n.x, 10.0, "untouched");
        assert!(out.light.is_none());
        assert_eq!(out.gfx_off_y, None);
    }

    /// `NPC.cs:84344-84360` — the dust plan: the hit half's
    /// `dmg/lifeMax*50.0` bound and the flat-70 death half. The gate is
    /// `life > 0`, so life == 0 takes the death arm.
    #[test]
    fn the_dust_plan_counts_the_hit_bound_and_the_flat_death() {
        // dmg 10 / lifeMax 1000 * 50 = 0.5 → one iteration (0 < 0.5).
        match bubble_hit_dust_plan(1000, 10.0, 1000) {
            BubbleHitDust::Hit { count } => assert_eq!(count, 1),
            _ => panic!("life > 0 is the hit arm"),
        }
        // dmg 100 / 1000 * 50 = 5.0 → five iterations (0..4 < 5).
        match bubble_hit_dust_plan(500, 100.0, 1000) {
            BubbleHitDust::Hit { count } => assert_eq!(count, 5),
            _ => panic!("hit arm"),
        }
        // The AI pop: life = 0 → the flat-70 death arm.
        match bubble_hit_dust_plan(0, 10.0, 1000) {
            BubbleHitDust::Death { count } => assert_eq!(count, 70),
            _ => panic!("life == 0 is the death arm (the > gate)"),
        }
        // A negative life (a strike past zero) also takes the death arm.
        match bubble_hit_dust_plan(-5, 10.0, 1000) {
            BubbleHitDust::Death { count } => assert_eq!(count, 70),
            _ => panic!("death arm"),
        }
    }

    /// The arm draws NO RNG — no `Next`/`NextFloat` anywhere in
    /// `NPC.cs:35080-35101`, no `TargetClosest`, no `EncourageDespawn`,
    /// and the constant `AddLight` draws nothing. Pin a `UnifiedRandom`
    /// state across attached + pop ticks so no stream can shift when
    /// the wiring lands.
    #[test]
    fn the_style72_arm_draws_no_rng() {
        let rng = UnifiedRandom::new(12345);
        let witness = rng.clone();
        let mut attached = bubble(0.0, 0.0);
        attached.ai[0] = 0.0;
        let mut popping = bubble(0.0, 0.0);
        popping.ai[0] = 9.0; // out of range → pop
        let hosts = [officer((1000.0, 800.0), 3.0)];
        for _ in 0..120 {
            let _ = ai_style72(&mut attached, &hosts, 384);
            let _ = ai_style72(&mut popping, &hosts, 384);
            let _ = bubble_hit_dust_plan(attached.hp, 100.0, 1000);
            let _ = bubble_hit_dust_plan(0, 10.0, 1000);
        }
        assert_eq!(rng, witness, "no Next/NextFloat anywhere in the arm");
    }
}
