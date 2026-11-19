//! Plantera's hook-centroid leash anchor — `NPC.cs:31743-31762`.
//! S1-D-3-plantera-minions, RESCOPED per the Lead coordination note: the
//! three minion machines are ALREADY ported by sibling branches —
//!
//! - kind 263 PlanterasHook (aiStyle 52): `npc_ai_style52.rs`, commit
//!   61e1bd5c (`task/S1-D-3-aistyle52`) — `planteras_hook_step` over
//!   `&mut [Npc]` + a `type_of` closure (the Destroyer precedent).
//! - kind 264 PlanterasTentacle (aiStyle 53): `npc_ai_style53.rs`, commit
//!   89a9b808 (`task/S1-D-3-aistyle53`).
//! - kind 265 Spore (+261 FungiSpore, aiStyle 50): `npc_ai_style50.rs`,
//!   commit 613aac92 (`task/S1-D-3-aistyle50`).
//! - the boss core (kind 262, aiStyle 51): `npc_ai_plantera.rs`, commit
//!   ba8159fc (`task/S1-D-3-plantera-audit`) — its `step` takes
//!   `hook_anchor: (f32, f32)` and the sim currently passes Plantera's
//!   own centre (no hook kind exists yet).
//!
//! NONE of the four computes the centroid itself: the boss module only
//! RECEIVES the anchor. This module owns the missing computation — the
//! C# block that walks the NPC slots, takes the first three active
//! aiStyle-52 hooks, sums their centres in slot order and divides by the
//! count. It is the seam the boss module consumes at the merge union.
//!
//! Source (`NPC.cs:31743-31762`, inside the aiStyle-51 boss arm):
//! ```csharp
//! int[] array2 = new int[3];            // :31743 — written, NEVER read
//! float num743 = 0f;                    // :31744 — the X sum
//! float num744 = 0f;                    // :31745 — the Y sum
//! int num745 = 0;                       // :31746 — the count
//! for (int num746 = 0; num746 < Main.maxNPCs; num746++)   // :31747
//!     if (Main.npc[num746].active && Main.npc[num746].aiStyle == 52)
//!     {                                 // :31749
//!         num743 += Main.npc[num746].Center.X;   // :31751
//!         num744 += Main.npc[num746].Center.Y;   // :31752
//!         array2[num745] = num746;      // :31753 — dead store
//!         num745++;                     // :31754
//!         if (num745 > 2) break;        // :31755-31758 — first three
//!     }
//! num743 /= (float)num745;              // :31761
//! num744 /= (float)num745;              // :31762
//! ```
//!
//! Faithful details:
//! - `aiStyle == 52` has exactly ONE member in the C# (`SetDefaults` case
//!   263, `NPC.cs:12402`; id `ID/NPCID.cs:11203`), so the port filters on
//!   the vanilla type via the caller's `type_of` closure — the SAME
//!   convention `npc_ai_style52.rs` uses for its `:32129` same-type scan.
//! - `Center` is `Entity.Center` (`Entity.cs:50-55`, position + width/2f
//!   FLOAT division) — the port's `Npc::center()` (`npc.rs:2119-2122`).
//!   NOT the tentacle anchor's integer-width centre (`:32301-32302`).
//! - the sums are f32 accumulated in SLOT ORDER (`:31751-31752`), then
//!   divided by the count as f32 (`:31761-31762`) — the order is kept.
//! - `array2` is written at `:31753` and NEVER read anywhere in the arm
//!   (verified: the only two `array2` mentions in `:31700-32080` are the
//!   declaration and the store) — a dead store, not ported.
//! - NO RNG in this block.
//!
//! Documented deviations (ANALYSIS "The Plantera hook-centroid anchor"):
//! - `NPC.active` maps to the pool's `hp > 0` convention (the same
//!   reading `npc_ai_style52.rs` uses for the boss slot, `:32080-32085`);
//!   absent slots do not exist in the port's dense pool.
//! - zero hooks: the C# divides by a zero count → NaN, and the boss
//!   leash then reads NaN. The port returns `None` and the merge-union
//!   caller falls back to Plantera's own centre — the sibling boss
//!   module's documented stand-in until hooks spawn.
//!
//! THE MERGE UNION (four modules, no file overlap): at merge the sim's
//! kind-262 dispatch replaces the centre fallback with
//! `hook_anchor(&app.npcs, type_of, boss)` (or
//! `hook_centroid(..).unwrap_or(boss.center())`) and passes the result
//! into `npc_ai_plantera::step`'s `hook_anchor` parameter. No edit to
//! any of the four sibling modules is needed — this module is new.
//!
//! WIRED at the merge union (S1-D-3-plantera-minions): the sim's pre-pass
//! (`world_sim/npcs.rs`, before the tick closure) computes
//! `hook_centroid(&app.npcs, type_of)` per live Plantera and writes
//! `Npc::plantera_hook_anchor`; `update_plantera` consumes it with the
//! own-centre fallback. S1-D-3-wire-plantera-minions landed the hook
//! kind (263) and fixed the pre-pass's `type_of` to read `cs_id` (the
//! VANILLA type — `to_u16` is the protocol id 154 (gate-reassigned), which never matched
//! 263): the centroid is live whenever a hook lives.

use crate::npc::Npc;

/// `NPCID.PlanterasHook` (`ID/NPCID.cs:11203`) — the ONLY aiStyle-52
/// member (`NPC.cs:12402`), the type the centroid filters on.
pub const PLANTERAS_HOOK: i32 = 263;

/// The hook-centroid anchor — `NPC.cs:31743-31762`. Walk the pool in
/// SLOT ORDER, take the first three live aiStyle-52 hooks (the caller's
/// `type_of` resolves the vanilla type; `hp > 0` is the port's `active`),
/// sum their `Npc::center()`s as f32 in order and divide by the count.
/// `None` when no hook lives (the C#'s zero-count NaN; the caller falls
/// back to the boss centre — see the module doc).
pub fn hook_centroid(npcs: &[Npc], type_of: impl Fn(&Npc) -> i32) -> Option<(f32, f32)> {
    let mut sum_x = 0.0f32; // num743 (`:31744`)
    let mut sum_y = 0.0f32; // num744 (`:31745`)
    let mut count = 0i32; // num745 (`:31746`)
    for n in npcs {
        // `:31749` — active && aiStyle == 52 (the one-member family).
        if n.hp > 0 && type_of(n) == PLANTERAS_HOOK {
            let (cx, cy) = n.center(); // `Entity.Center` (`Entity.cs:50-55`)
            sum_x += cx; // `:31751`
            sum_y += cy; // `:31752`
            count += 1; // `:31754` (`array2`'s `:31753` store is dead)
            if count > 2 {
                break; // `:31755-31758` — the first three only
            }
        }
    }
    if count == 0 {
        return None; // the C# NaN (`:31761-31762` ÷ 0) — documented
    }
    Some((sum_x / count as f32, sum_y / count as f32)) // `:31761-31762`
}

/// The merge-union call site in one fn: the centroid when hooks live,
/// Plantera's own centre otherwise — exactly the parameter
/// `npc_ai_plantera::step` (`ba8159fc`) receives as `hook_anchor`. The
/// fallback is the sibling's documented stand-in (the C# NaN case).
/// The sim's pre-pass stores the `Option` instead (the fallback lives in
/// `update_plantera`), so this convenience wrapper is test-only.
#[allow(dead_code)]
pub fn hook_anchor(npcs: &[Npc], type_of: impl Fn(&Npc) -> i32, boss: &Npc) -> (f32, f32) {
    hook_centroid(npcs, type_of).unwrap_or_else(|| boss.center())
}

// ---- witness tests (WRITTEN, NOT RUN — code-only mode, no compiler) ----
#[cfg(test)]
mod tests {
    use super::*;
    use crate::npc::NpcKind;

    /// The `type_of` stand-in mapping (the `npc_ai_style52.rs` test
    /// convention): BurningSphere plays the hook (263), Plantera the
    /// boss (262); every other kind reads 0.
    fn type_of(n: &Npc) -> i32 {
        match n.kind {
            NpcKind::BurningSphere => PLANTERAS_HOOK,
            NpcKind::Plantera => 262,
            _ => 0,
        }
    }

    fn hook(x: f32, y: f32) -> Npc {
        Npc::new(NpcKind::BurningSphere, x, y, 1, 0.0)
    }

    fn boss(x: f32, y: f32) -> Npc {
        Npc::new(NpcKind::Plantera, x, y, 1, 0.0)
    }

    /// `NPC.cs:31749` + `:31761-31762` — an empty pool (or no hook) is
    /// the zero-count case: `None`, and the union fn falls back to the
    /// boss centre (the sibling's documented stand-in for the C# NaN).
    #[test]
    fn s1_d_3_minions_the_centroid_is_none_without_hooks() {
        assert_eq!(hook_centroid(&[], type_of), None);
        let b = boss(100.0, 200.0);
        assert_eq!(hook_centroid(&[b.clone()], type_of), None);
        assert_eq!(hook_anchor(&[b.clone()], type_of, &b), b.center());
        // A dead hook (hp = 0, the port's `active` reading) never rides.
        let mut dead = hook(10.0, 20.0);
        dead.hp = 0;
        assert_eq!(hook_centroid(&[dead], type_of), None);
    }

    /// `NPC.cs:31751-31752` + `:31761-31762` — one hook: the centroid
    /// IS its centre (the f32 sum of one divided by 1).
    #[test]
    fn s1_d_3_minions_one_hook_anchors_at_its_own_centre() {
        let h = hook(100.0, 200.0);
        let b = boss(0.0, 0.0);
        assert_eq!(hook_centroid(&[h.clone()], type_of), Some(h.center()));
        assert_eq!(hook_anchor(&[h.clone()], type_of, &b), h.center());
    }

    /// `NPC.cs:31747-31762` — three hooks: the slot-order f32 average;
    /// the boss slot in the pool never rides the sum (type 262 ≠ 263).
    #[test]
    fn s1_d_3_minions_three_hooks_average_in_slot_order() {
        let a = hook(0.0, 0.0);
        let b = hook(30.0, 60.0);
        let c = hook(60.0, 120.0);
        let boss_npc = boss(9000.0, 9000.0);
        let pool = [a.clone(), boss_npc, b.clone(), c.clone()];
        let (ax, ay) = a.center();
        let (bx, by) = b.center();
        let (cx, cy) = c.center();
        assert_eq!(
            hook_centroid(&pool, type_of),
            Some(((ax + bx + cx) / 3.0, (ay + by + cy) / 3.0))
        );
    }

    /// `NPC.cs:31755-31758` — the `num745 > 2` break: a FOURTH hook
    /// never rides the sum, even when it sits later in the pool.
    #[test]
    fn s1_d_3_minions_the_centroid_caps_at_the_first_three() {
        let a = hook(0.0, 0.0);
        let b = hook(30.0, 60.0);
        let c = hook(60.0, 120.0);
        let d = hook(9000.0, 9000.0);
        let (ax, ay) = a.center();
        let (bx, by) = b.center();
        let (cx, cy) = c.center();
        assert_eq!(
            hook_centroid(&[a.clone(), b.clone(), c.clone(), d.clone()], type_of),
            Some(((ax + bx + cx) / 3.0, (ay + by + cy) / 3.0))
        );
        // Slot order matters: the first three in the pool win, not the
        // three nearest the boss.
        let e = hook(-9000.0, -9000.0);
        let (ex, ey) = e.center();
        assert_eq!(
            hook_centroid(&[e, b, c, d], type_of),
            Some(((ex + bx + cx) / 3.0, (ey + by + cy) / 3.0))
        );
    }

    /// `NPC.cs:31749` — inactive hooks are skipped but do NOT consume a
    /// cap slot: a dead hook between live ones leaves the later live
    /// hook eligible.
    #[test]
    fn s1_d_3_minions_dead_hooks_skip_without_consuming_the_cap() {
        let a = hook(0.0, 0.0);
        let mut dead = hook(500.0, 500.0);
        dead.hp = 0;
        let b = hook(30.0, 60.0);
        let c = hook(60.0, 120.0);
        let d = hook(9000.0, 9000.0);
        let (ax, ay) = a.center();
        let (bx, by) = b.center();
        let (cx, cy) = c.center();
        // a, b, c are the first three LIVE hooks; d is the fourth.
        assert_eq!(
            hook_centroid(&[a, dead, b, c, d], type_of),
            Some(((ax + bx + cx) / 3.0, (ay + by + cy) / 3.0))
        );
    }
}
