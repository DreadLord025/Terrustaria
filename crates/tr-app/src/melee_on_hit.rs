//! S1-P-i10 — the melee weapon on-hit debuff infliction:
//! `Player.StatusToNPC` (`Player.cs:6502-6617`), called from
//! `ProcessHitAgainstNPC` at `Player.cs:44391` — AFTER the crit roll
//! (`:44298`) and `Main.DamageVar` (`:44342`), BEFORE the nano-flask
//! damage boost (`:44396-44399`), the armour-penetration add (`:44400`)
//! and `NPC.StrikeNPC` (`:44403`). `NPC.StrikeNPC` itself carries NO
//! weapon-debuff application (its only `AddBuff` is the Daybreak
//! death-spread, `NPC.cs:81730-81751` — a projectile-debuff death
//! effect, the projectile lane's scope), so the port calls this module
//! at the same pre-strike site in `world_sim/editing.rs`.
//!
//! The C# block order (and therefore the `Main.rand` draw order) is:
//! 1. the `meleeEnchant` if-chain in the SOURCE order 1,2,3,5,6,8,4
//!    (`:6504-6535`) — NOT numeric order; enchant 7 (Confetti) has no
//!    debuff arm (its projectile 289 spawns in `ApplyNPCOnHitEffects`
//!    `:44465-44468`, the projectile lane);
//! 2. `catalystBand` → AcceleratePoisons 398 (`:6536-6539`, no roll);
//! 3. item 2330 Purple Clubberfish's Confused gate (`:6540-6543`);
//! 4. item 3352 the Stylist's scissors → Bleeding (`:6544-6547`);
//! 5. `frostBurn` (the Frost armor set) → Frostburn2 324 (`:6548-6551`);
//! 6. `magmaStone` → OnFire3 323, ALWAYS applied, the 1/4-then-1/2
//!    ladder picks 360/240/120 (`:6552-6566`);
//! 7. item 3211 Bladetongue → Ichor (`:6567-6570`);
//! 8. item 5129 Flymeal → Stinky 120 (`:6571-6574`, no roll);
//! 9. the per-weapon switch (`:6575-6616`): 121 Fiery Greatsword,
//!    3823 Brand of the Inferno, 5382 Waffle's Iron, 122 Molten Pickaxe,
//!    190 Blade of Grass, 217 Molten Hamaxe, 1123 Bee Keeper.
//!
//! Every duration `Next` roll is drawn BEFORE `NPC.AddBuff`'s immunity
//! guard runs — an immune NPC still consumes the draw (the C# evaluates
//! the argument first). `NPC.AddBuff` (`NPC.cs:89926-89983`) is already
//! ported (`npc_buffs.rs::NpcBuffs::add_buff`); this module only feeds it.
//!
//! Scope (S1-P-i10): the DIRECT melee swing path only. The projectile /
//! whip enchant consumers (`Projectile.cs:11192-11224`, `:11755-11782`,
//! `:17362-17470`) and `StatusToPlayerPvP` (`Player.cs:6619-6713`) are
//! other lanes. Deviations recorded in docs/ANALYSIS.md "The melee
//! on-hit debuffs (S1-P-i10)".

use tr_core::loot::DropRng;

use crate::npc_buffs::NpcBuffs;
use crate::player::Player;

/// `Player.GetNanoFlaskDamageBoost` (`Player.cs:44423-44426`): the
/// Flask of Nanites' melee damage multiplier, applied to the rolled
/// `num2` right after `StatusToNPC` (`:44396-44399`).
pub fn nano_flask_damage_boost() -> f32 {
    1.05
}

/// The four `Player` fields `StatusToNPC` reads — a Copy snapshot so
/// the hit site can read them before the mutable NPC-borrow (the
/// `RetaliateRequest` snapshot convention, `hurt_modifiers.rs`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MeleeOnHitFlags {
    /// `Player.meleeEnchant` (`Player.cs:1068`): the imbue id 0..8.
    pub melee_enchant: u8,
    /// `Player.catalystBand` (`Player.cs:14618`).
    pub catalyst_band: bool,
    /// `Player.frostBurn` (`ArmorSetBonuses.cs:271`).
    pub frost_burn: bool,
    /// `Player.magmaStone` (`Player.cs:14581`/`:15313`/`:15338`).
    pub magma_stone: bool,
}

impl Player {
    /// The `StatusToNPC` input snapshot (`Player.cs:6502-6617` reads
    /// exactly these four fields off `this`).
    pub fn melee_on_hit_flags(&self) -> MeleeOnHitFlags {
        MeleeOnHitFlags {
            melee_enchant: self.melee_enchant,
            catalyst_band: self.catalyst_band,
            frost_burn: self.frost_burn,
            magma_stone: self.magma_stone,
        }
    }
}

/// `Player.StatusToNPC` (`Player.cs:6502-6617`) — the melee on-hit
/// debuff infliction, verbatim block order. `item_type` is `sItem.type`
/// (the held item's kind), `npc` the struck NPC's buff bar, `rng` the
/// hitting lane's `Main.rand` (the port's `UpdatePlayers` stream at the
/// `world_sim/editing.rs` melee site, `Main.cs:17754`). The hit site
/// passes the [`MeleeOnHitFlags`] snapshot (the player borrow must end
/// before the mutable NPC-buff borrow); tests use the `&Player` helper
/// below.
pub fn status_to_npc_flags(
    flags: MeleeOnHitFlags,
    item_type: u16,
    npc: &mut NpcBuffs,
    rng: &mut dyn DropRng,
) {
    let p = flags;
    // The weapon imbues (`Player.cs:6504-6535`). The C# writes seven
    // independent `if`s in the order 1,2,3,5,6,8,4 — only one enchant
    // can be armed (`UpdateBuffs` `:12013-12044` overwrites), so the
    // order is observationally a dispatch, but the port keeps the
    // source order for the citation walk.
    if p.melee_enchant > 0 {
        if p.melee_enchant == 1 {
            // Flask of Venom (`:6506-6510`): Venom 70 + PotentAcid 395.
            npc.add_buff(70, 60 * rng.next_range(5, 10));
            npc.add_buff(395, 60);
        }
        if p.melee_enchant == 2 {
            // Flask of Cursed Flames (`:6511-6514`): CursedInferno 39.
            npc.add_buff(39, 60 * rng.next_range(3, 7));
        }
        if p.melee_enchant == 3 {
            // Flask of Fire (`:6515-6518`): OnFire 24.
            npc.add_buff(24, 60 * rng.next_range(3, 7));
        }
        if p.melee_enchant == 5 {
            // Flask of Ichor (`:6519-6522`): Ichor 69.
            npc.add_buff(69, 60 * rng.next_range(10, 20));
        }
        if p.melee_enchant == 6 {
            // Flask of Nanites (`:6523-6526`): Confused 31.
            npc.add_buff(31, 60 * rng.next_range(1, 4));
        }
        if p.melee_enchant == 8 {
            // Flask of Poison (`:6527-6530`): Poisoned 20.
            npc.add_buff(20, 60 * rng.next_range(5, 10));
        }
        if p.melee_enchant == 4 {
            // Flask of Gold (`:6531-6534`): Midas 72, fixed 120 — no roll.
            npc.add_buff(72, 120);
        }
        // Enchant 7 (Flask of Party) has NO arm here — the confetti
        // projectile 289 is `ApplyNPCOnHitEffects` `:44465-44468`.
    }
    // The Catalyst Band 6185 (`:6536-6539`): AcceleratePoisons 398,
    // fixed 300 — no roll.
    if p.catalyst_band {
        npc.add_buff(398, 300);
    }
    // Purple Clubberfish 2330 (`:6540-6543`): a 1/2 gate, then Confused
    // 31 for 60 * Next(1, 4). The duration draws ONLY when the gate passes.
    if item_type == 2330 && rng.next_range(0, 2) == 0 {
        npc.add_buff(31, 60 * rng.next_range(1, 4));
    }
    // The Stylist's scissors 3352 (`:6544-6547`): Bleeding 30 for
    // 60 * Next(4, 7) — unconditional.
    if item_type == 3352 {
        npc.add_buff(30, 60 * rng.next_range(4, 7));
    }
    // The Frost armor set's frostBurn (`:6548-6551`): Frostburn2 324
    // for 60 * Next(5, 15). (The PvP variant rolls Next(1, 8) —
    // `StatusToPlayerPvP` `:6650`, not this lane.)
    if p.frost_burn {
        npc.add_buff(324, 60 * rng.next_range(5, 15));
    }
    // The Magma Stone family (`:6552-6566`): OnFire3 323 ALWAYS — the
    // 1/4 roll picks 360, else a SECOND 1/2 roll picks 240 vs 120.
    // Draw count: 1 on the 1/4 pass, 2 otherwise. (PvP differs:
    // Next(7)/Next(3) and 360/120/60, `:6652-6666` — not this lane.)
    if p.magma_stone {
        if rng.next_range(0, 4) == 0 {
            npc.add_buff(323, 360);
        } else if rng.next_range(0, 2) == 0 {
            npc.add_buff(323, 240);
        } else {
            npc.add_buff(323, 120);
        }
    }
    // Bladetongue 3211 (`:6567-6570`): Ichor 69 for 60 * Next(5, 10).
    if item_type == 3211 {
        npc.add_buff(69, 60 * rng.next_range(5, 10));
    }
    // Flymeal 5129 (`:6571-6574`): Stinky 120, fixed 300 — no roll.
    if item_type == 5129 {
        npc.add_buff(120, 300);
    }
    // The per-weapon switch (`:6575-6616`), source case order.
    match item_type {
        // Fiery Greatsword / Volcano 121 (`:6577-6582`): 1/2 → OnFire 24, 180.
        121 => {
            if rng.next_range(0, 2) == 0 {
                npc.add_buff(24, 180);
            }
        }
        // Brand of the Inferno 3823 (`:6583-6588`): 1/4 → OnFire3 323, 300.
        3823 => {
            if rng.next_range(0, 4) == 0 {
                npc.add_buff(323, 300);
            }
        }
        // Waffle's Iron 5382 (`:6589-6594`): 1/3 → OnFire3 323, 300.
        5382 => {
            if rng.next_range(0, 3) == 0 {
                npc.add_buff(323, 300);
            }
        }
        // Molten Pickaxe 122 (`:6595-6600`): 1/10 → OnFire 24, 180.
        122 => {
            if rng.next_range(0, 10) == 0 {
                npc.add_buff(24, 180);
            }
        }
        // Blade of Grass 190 (`:6601-6606`): 1/4 → Poisoned 20, 420.
        190 => {
            if rng.next_range(0, 4) == 0 {
                npc.add_buff(20, 420);
            }
        }
        // Molten Hamaxe 217 (`:6607-6612`): 1/5 → OnFire 24, 180.
        217 => {
            if rng.next_range(0, 5) == 0 {
                npc.add_buff(24, 180);
            }
        }
        // Bee Keeper 1123 (`:6613-6615`): Confused 31, fixed 120 — no roll.
        1123 => {
            npc.add_buff(31, 120);
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::npc::NpcKind;
    use tr_core::world::World;

    fn player() -> Player {
        Player::spawn(&World::new("t".into(), 64, 800))
    }

    /// The `&Player` test convenience over the flags core — the hit
    /// site itself passes `p.melee_on_hit_flags()`.
    fn status_to_npc(p: &Player, item_type: u16, npc: &mut NpcBuffs, rng: &mut dyn DropRng) {
        status_to_npc_flags(p.melee_on_hit_flags(), item_type, npc, rng);
    }

    /// A scripted `DropRng` recording every `(min, max)` draw — the
    /// draw-order contract mirror (the `hurt_modifiers` pattern).
    struct ScriptRng {
        rolls: Vec<i32>,
        draws: Vec<(i32, i32)>,
    }
    impl ScriptRng {
        fn with_rolls(rolls: &[i32]) -> Self {
            Self {
                rolls: rolls.to_vec(),
                draws: Vec::new(),
            }
        }
    }
    impl DropRng for ScriptRng {
        fn next_range(&mut self, min: i32, max: i32) -> i32 {
            self.draws.push((min, max));
            self.rolls.remove(0)
        }
        fn next_float(&mut self) -> f32 {
            0.0
        }
    }

    /// The buff bar with no immunities (a plain `Default`, the DemonEye
    /// null-row shape — `NPC.cs:17826-17832`).
    fn bar() -> NpcBuffs {
        NpcBuffs::default()
    }

    fn time_of(npc: &NpcBuffs, id: i32) -> Option<i32> {
        npc.find_buff_index(id).map(|i| npc.buff_time[i])
    }

    #[test]
    fn s1_p_i10_a_plain_hit_inflicts_nothing_and_draws_nothing() {
        // No flags, a plain wooden sword (24): every gate fails closed.
        let p = player();
        let mut npc = bar();
        let mut rng = ScriptRng::with_rolls(&[]);
        status_to_npc(&p, 24, &mut npc, &mut rng);
        assert!(rng.draws.is_empty(), "no source armed draws nothing");
        assert!(npc.buff_type.iter().all(|&t| t == 0));
    }

    #[test]
    fn s1_p_i10_the_venom_flask_adds_venom_then_potent_acid() {
        // `Player.cs:6506-6510`: Venom 70 for 60 * Next(5, 10), then
        // PotentAcid 395 for a fixed 60 — ONE draw, in that order.
        let mut p = player();
        p.melee_enchant = 1;
        let mut npc = bar();
        let mut rng = ScriptRng::with_rolls(&[7]);
        status_to_npc(&p, 24, &mut npc, &mut rng);
        assert_eq!(rng.draws, vec![(5, 10)]);
        assert_eq!(time_of(&npc, 70), Some(420));
        assert_eq!(time_of(&npc, 395), Some(60));
    }

    #[test]
    fn s1_p_i10_the_flask_arms_map_to_their_debuffs_and_rolls() {
        // `Player.cs:6511-6534`: (enchant, buff, roll bounds, rolled
        // value, expected time). Gold (4) is the fixed-120 no-roll arm;
        // Party (7) has no StatusToNPC arm at all.
        let cases: &[(u8, i32, (i32, i32), i32, i32)] = &[
            (2, 39, (3, 7), 5, 300),  // Cursed Flames → CursedInferno
            (3, 24, (3, 7), 3, 180),  // Fire → OnFire
            (5, 69, (10, 20), 12, 720), // Ichor
            (6, 31, (1, 4), 2, 120),  // Nanites → Confused
            (8, 20, (5, 10), 9, 540), // Poison → Poisoned
        ];
        for &(enchant, buff, bounds, roll, time) in cases {
            let mut p = player();
            p.melee_enchant = enchant;
            let mut npc = bar();
            let mut rng = ScriptRng::with_rolls(&[roll]);
            status_to_npc(&p, 24, &mut npc, &mut rng);
            assert_eq!(rng.draws, vec![bounds], "enchant {enchant}");
            assert_eq!(time_of(&npc, buff), Some(time), "enchant {enchant}");
        }
        // Gold: Midas 72 at a fixed 120, zero draws (`:6531-6534`).
        let mut p = player();
        p.melee_enchant = 4;
        let mut npc = bar();
        let mut rng = ScriptRng::with_rolls(&[]);
        status_to_npc(&p, 24, &mut npc, &mut rng);
        assert!(rng.draws.is_empty());
        assert_eq!(time_of(&npc, 72), Some(120));
        // Party: nothing, zero draws (the confetti 289 is the proj lane).
        let mut p = player();
        p.melee_enchant = 7;
        let mut npc = bar();
        let mut rng = ScriptRng::with_rolls(&[]);
        status_to_npc(&p, 24, &mut npc, &mut rng);
        assert!(rng.draws.is_empty());
        assert!(npc.buff_type.iter().all(|&t| t == 0));
    }

    #[test]
    fn s1_p_i10_the_catalyst_band_accelerates_poisons_without_a_roll() {
        // `Player.cs:6536-6539`: 398 for a fixed 300.
        let mut p = player();
        p.catalyst_band = true;
        let mut npc = bar();
        let mut rng = ScriptRng::with_rolls(&[]);
        status_to_npc(&p, 24, &mut npc, &mut rng);
        assert!(rng.draws.is_empty());
        assert_eq!(time_of(&npc, 398), Some(300));
    }

    #[test]
    fn s1_p_i10_the_clubberfish_gates_confused_behind_a_coin_flip() {
        // `Player.cs:6540-6543`: Next(2) == 0 gates the Next(1, 4)
        // duration — a failed gate draws ONCE, a pass draws TWICE.
        let mut p = player();
        let mut npc = bar();
        let mut rng = ScriptRng::with_rolls(&[0, 3]);
        status_to_npc(&p, 2330, &mut npc, &mut rng);
        assert_eq!(rng.draws, vec![(0, 2), (1, 4)]);
        assert_eq!(time_of(&npc, 31), Some(180));
        // The failed gate: no duration draw, no buff.
        let mut npc = bar();
        let mut rng = ScriptRng::with_rolls(&[1]);
        status_to_npc(&p, 2330, &mut npc, &mut rng);
        assert_eq!(rng.draws, vec![(0, 2)]);
        assert!(time_of(&npc, 31).is_none());
    }

    #[test]
    fn s1_p_i10_the_scissors_bleed_and_the_frost_set_burns() {
        // `Player.cs:6544-6547`: 3352 → Bleeding 30, 60 * Next(4, 7).
        let p = player();
        let mut npc = bar();
        let mut rng = ScriptRng::with_rolls(&[6]);
        status_to_npc(&p, 3352, &mut npc, &mut rng);
        assert_eq!(rng.draws, vec![(4, 7)]);
        assert_eq!(time_of(&npc, 30), Some(360));
        // `Player.cs:6548-6551`: frostBurn → Frostburn2 324,
        // 60 * Next(5, 15) — the NPC roll, NOT the PvP Next(1, 8).
        let mut p = player();
        p.frost_burn = true;
        let mut npc = bar();
        let mut rng = ScriptRng::with_rolls(&[14]);
        status_to_npc(&p, 24, &mut npc, &mut rng);
        assert_eq!(rng.draws, vec![(5, 15)]);
        assert_eq!(time_of(&npc, 324), Some(840));
    }

    #[test]
    fn s1_p_i10_the_magma_stone_always_burns_on_the_three_arm_ladder() {
        // `Player.cs:6552-6566`: Next(4) == 0 → 360 (ONE draw); else
        // Next(2) == 0 → 240 (TWO draws); else 120 (TWO draws). The
        // buff lands on every arm — there is no no-buff outcome.
        let mut p = player();
        p.magma_stone = true;
        let mut npc = bar();
        let mut rng = ScriptRng::with_rolls(&[0]);
        status_to_npc(&p, 24, &mut npc, &mut rng);
        assert_eq!(rng.draws, vec![(0, 4)]);
        assert_eq!(time_of(&npc, 323), Some(360));
        let mut npc = bar();
        let mut rng = ScriptRng::with_rolls(&[3, 0]);
        status_to_npc(&p, 24, &mut npc, &mut rng);
        assert_eq!(rng.draws, vec![(0, 4), (0, 2)]);
        assert_eq!(time_of(&npc, 323), Some(240));
        let mut npc = bar();
        let mut rng = ScriptRng::with_rolls(&[1, 1]);
        status_to_npc(&p, 24, &mut npc, &mut rng);
        assert_eq!(rng.draws, vec![(0, 4), (0, 2)]);
        assert_eq!(time_of(&npc, 323), Some(120));
    }

    #[test]
    fn s1_p_i10_the_bladetongue_ichors_and_the_flymeal_stinks() {
        // `Player.cs:6567-6570`: 3211 → Ichor 69, 60 * Next(5, 10).
        let p = player();
        let mut npc = bar();
        let mut rng = ScriptRng::with_rolls(&[5]);
        status_to_npc(&p, 3211, &mut npc, &mut rng);
        assert_eq!(rng.draws, vec![(5, 10)]);
        assert_eq!(time_of(&npc, 69), Some(300));
        // `Player.cs:6571-6574`: 5129 → Stinky 120, fixed 300, no roll.
        let mut npc = bar();
        let mut rng = ScriptRng::with_rolls(&[]);
        status_to_npc(&p, 5129, &mut npc, &mut rng);
        assert!(rng.draws.is_empty());
        assert_eq!(time_of(&npc, 120), Some(300));
    }

    #[test]
    fn s1_p_i10_the_weapon_switch_gates_and_times_match_the_csharp() {
        // `Player.cs:6575-6616`: (item, gate bound, buff, fixed time).
        // A passing gate roll of 0 lands the buff; a failing roll of
        // bound-1 draws exactly once and lands nothing.
        let cases: &[(u16, i32, i32, i32)] = &[
            (121, 2, 24, 180),  // Fiery Greatsword
            (3823, 4, 323, 300), // Brand of the Inferno
            (5382, 3, 323, 300), // Waffle's Iron
            (122, 10, 24, 180), // Molten Pickaxe
            (190, 4, 20, 420),  // Blade of Grass
            (217, 5, 24, 180),  // Molten Hamaxe
        ];
        let p = player();
        for &(item, bound, buff, time) in cases {
            let mut npc = bar();
            let mut rng = ScriptRng::with_rolls(&[0]);
            status_to_npc(&p, item, &mut npc, &mut rng);
            assert_eq!(rng.draws, vec![(0, bound)], "item {item} pass");
            assert_eq!(time_of(&npc, buff), Some(time), "item {item} pass");
            let mut npc = bar();
            let mut rng = ScriptRng::with_rolls(&[bound - 1]);
            status_to_npc(&p, item, &mut npc, &mut rng);
            assert_eq!(rng.draws, vec![(0, bound)], "item {item} fail");
            assert!(time_of(&npc, buff).is_none(), "item {item} fail");
        }
        // Bee Keeper 1123 (`:6613-6615`): Confused 31 at a fixed 120,
        // unconditional, zero draws.
        let mut npc = bar();
        let mut rng = ScriptRng::with_rolls(&[]);
        status_to_npc(&p, 1123, &mut npc, &mut rng);
        assert!(rng.draws.is_empty());
        assert_eq!(time_of(&npc, 31), Some(120));
    }

    #[test]
    fn s1_p_i10_the_full_block_draws_in_the_csharp_order() {
        // Fire flask (3) + catalyst + Frost set + magma stone, swinging
        // the Blade of Grass (190). The C# order (`:6504-6616`):
        // enchant Next(3,7) → catalyst (no draw) → 2330/3352 gates
        // (item is 190, skipped) → frost Next(5,15) → magma Next(4)
        // [+ Next(2) on the fail] → 3211/5129 (skipped) → the switch's
        // 190 gate Next(4) LAST. Rolls: 4, 9, 2, 0, 0.
        let mut p = player();
        p.melee_enchant = 3;
        p.catalyst_band = true;
        p.frost_burn = true;
        p.magma_stone = true;
        let mut npc = bar();
        let mut rng = ScriptRng::with_rolls(&[4, 9, 2, 0, 0]);
        status_to_npc(&p, 190, &mut npc, &mut rng);
        assert_eq!(
            rng.draws,
            vec![(3, 7), (5, 15), (0, 4), (0, 2), (0, 4)],
            "enchant → frost → magma ladder → the weapon switch"
        );
        assert_eq!(time_of(&npc, 24), Some(240));
        assert_eq!(time_of(&npc, 398), Some(300));
        assert_eq!(time_of(&npc, 324), Some(540));
        assert_eq!(time_of(&npc, 323), Some(240));
        assert_eq!(time_of(&npc, 20), Some(420));
    }

    #[test]
    fn s1_p_i10_an_immune_npc_still_consumes_the_duration_draw() {
        // The C# evaluates `60 * Main.rand.Next(5, 10)` BEFORE
        // `AddBuff`'s `buffImmune` guard (`NPC.cs:89928-89931`) — a
        // slime (immune 20, `NPCID.cs:6079-6084`) eats the Poison
        // flask's draw and stores nothing.
        let mut p = player();
        p.melee_enchant = 8;
        let mut npc = NpcBuffs::for_kind(NpcKind::Slime);
        let mut rng = ScriptRng::with_rolls(&[6]);
        status_to_npc(&p, 24, &mut npc, &mut rng);
        assert_eq!(rng.draws, vec![(5, 10)], "the draw happens regardless");
        assert!(time_of(&npc, 20).is_none(), "the immunity rejects the buff");
        // The Blade of Grass gate draws too, pass or fail, on the slime.
        let p = player();
        let mut npc = NpcBuffs::for_kind(NpcKind::Slime);
        let mut rng = ScriptRng::with_rolls(&[0]);
        status_to_npc(&p, 190, &mut npc, &mut rng);
        assert_eq!(rng.draws, vec![(0, 4)]);
        assert!(time_of(&npc, 20).is_none());
    }

    #[test]
    fn s1_p_i10_a_longer_existing_time_wins_and_a_shorter_refreshes() {
        // `NPC.AddBuff` (`NPC.cs:89941-89944`): the same-id early-out.
        // The roll is still consumed either way.
        let mut p = player();
        p.melee_enchant = 3;
        let mut npc = bar();
        npc.add_buff(24, 600);
        let mut rng = ScriptRng::with_rolls(&[3]);
        status_to_npc(&p, 24, &mut npc, &mut rng);
        assert_eq!(rng.draws, vec![(3, 7)]);
        assert_eq!(time_of(&npc, 24), Some(600), "600 >= 180 keeps the old");
        let mut rng = ScriptRng::with_rolls(&[6]);
        status_to_npc(&p, 24, &mut npc, &mut rng);
        assert_eq!(time_of(&npc, 24), Some(600), "600 >= 360 keeps the old");
        npc.buff_time[npc.find_buff_index(24).unwrap()] = 60;
        let mut rng = ScriptRng::with_rolls(&[5]);
        status_to_npc(&p, 24, &mut npc, &mut rng);
        assert_eq!(time_of(&npc, 24), Some(300), "60 < 300 refreshes");
    }

    #[test]
    fn s1_p_i10_the_nano_flask_boost_is_the_csharp_constant() {
        // `Player.GetNanoFlaskDamageBoost` (`Player.cs:44423-44426`).
        assert_eq!(nano_flask_damage_boost(), 1.05);
    }
}
