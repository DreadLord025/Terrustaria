//! W4-f35 — the NPC death HEAL feed: `NPC.NPCLoot_DropHeals`
//! (`NPC.cs:79098-79195`) + its `NPCLoot_DropCommonLifeAndMana`
//! preamble (`:79531-79557`) + the boss-only
//! `DoDeathEvents_DropBossPotionsAndHearts` (`:78945-79016`), the two
//! death-path arms g60's dust lane does NOT claim (its census covers
//! the death DUST only — the heal drops are item spawns).
//!
//! The C# call order inside `NPCLoot` (`:78830-78899`):
//! `NPCLoot_DropItems` → `DoDeathEvents` (whose `this.boss` tail at
//! `:79505-79507` runs the boss potions) → `NPCLoot_DropMoney` →
//! `NPCLoot_DropHeals`. The port's death retain runs `spawn_coins`
//! BEFORE `spawn_loot` (the recorded pre-existing order fold); the
//! heal feed lands AFTER `spawn_loot`, keeping the C#'s
//! loot-before-heals relative order on the shared `UpdateNPCs` stream.
//!
//! Every drop is an `Item.NewItem` at the NPC's `(int)position.X/Y`
//! box — the port's `spawn_loot` drop convention (the velocity triplet
//! + the `Prefix(-1)` roll + the heart/star seasonal remap,
//! `Item.cs:49668-49714`). The `RollLuck` gates ride
//! `tr_core::loot::roll_luck` (luck 0 = the plain `Next(range)`).
//!
//! ## The recorded quirks (kept verbatim)
//!
//! - The common preamble's `num` flag: netID 16/121 force 1 (NO
//!   life/mana roll at all), netID 81 forces 1 for the FIRST roll but
//!   still runs the SECOND (`RollLuck(2)`) mana roll (`:79534-79556`).
//! - The first roll's mana-vs-heart pick: `Next(2) == 0 && mana < max`
//!   → star; ELSE `Next(2) == 0 && life < max` → heart — the second
//!   `Next(2)` draws even when the first hit but mana was full
//!   (`:79539-79546`).
//! - The EoW segments 13/14/15 + the Creeper 267 gate the heart on
//!   `statLife < statLifeMax2` AFTER the chance draw (`:79166/:79174`).
//! - The WoF's Hungry 116-119: `!expertMode || Next(5) == 0` — the
//!   expert draw only happens in expert (`:79183`).
//! - The boss potion kind chain (`:78948-78996`): the `type > 113 &&
//!   type < 222` band maps the mechs to 499 BEFORE the 636/245/262/370
//!   /395/439 arms (which the band never reaches); 398 → 3544.
//! - The Badger's Hat 5004 (`:79003-79015`): the EoC 4 / WoF 113
//!   deaths latch the two KilledToday statics; when BOTH are up the
//!   hat drops and `ResetBadgerHatTime` clears both. The statics also
//!   reset at dusk (`Main.cs:65221`, `UpdateTime_StartNight`).
//!
//! Census: `docs/research/65-expert-drop-trees-census.md`.

#![allow(dead_code)] // the npcs.rs death-feed wiring lands with its commit

use tr_core::loot::{DropRng, roll_luck};

/// One heal-feed drop request: the item + the stack (the caller adds
/// the position/velocity/prefix — the `spawn_loot` convention).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HealDrop {
    pub item: u16,
    pub stack: u32,
}

/// The player scalars the heal gates read (`closestPlayer`).
#[derive(Clone, Copy, Debug)]
pub struct HealPlayer {
    pub luck: f32,
    /// `statLife < statLifeMax2` — the heart gates.
    pub life_below_max: bool,
    /// `statMana < statManaMax2` — the star gates.
    pub mana_below_max: bool,
}

/// The NPC scalars the heal gates read.
#[derive(Clone, Copy, Debug)]
pub struct HealNpc {
    /// `this.netID` — the common preamble's 16/121/81 arms.
    pub net_id: i32,
    /// `this.type` — the per-kind switch + the boss potion chain.
    pub kind: i32,
    pub life_max: i32,
    pub damage: i32,
}

/// The `NPC.boss` set (`SetDefaults`' 22 `this.boss = true` rows:
/// `NPC.cs:8716/9180/9389/10358/10535/10552/10570/10676/11868/12176/
/// 12393/12452/13716/14061/14097/14116/14136/14227/17202/17323/17401/
/// 17460`) + the EoW head's dynamic latch (`DropEoWLoot`, `:78414-
/// 78429`: the LAST living segment sets `boss = true` before its
/// `NPCLoot`). The port's `Npc.boss` field only carries King Slime +
/// Fishron today — this table is the death-feed's own gate.
pub fn is_boss_kind(kind: i32) -> bool {
    matches!(
        kind,
        4 | 13
            | 35
            | 50
            | 113
            | 125
            | 126
            | 127
            | 134
            | 222
            | 245
            | 262
            | 266
            | 370
            | 395
            | 396
            | 397
            | 398
            | 439
            | 636
            | 657
            | 664
            | 668
    )
}

/// The Badger's Hat latches (`NPC.EoCKilledToday` / `WoFKilledToday`,
/// `NPC.cs:6611-6613`): the wiring keeps them on the World, the dusk
/// transition clears both (`Main.cs:65221`).
#[derive(Clone, Copy, Debug, Default)]
pub struct BadgerLatches {
    pub eoc_killed_today: bool,
    pub wof_killed_today: bool,
}

/// `NPC.NPCLoot_DropCommonLifeAndMana` (`NPC.cs:79531-79557`): the
/// 1/6 life-or-star roll + the 1/2 mana roll, with the 16/121/81
/// netID exclusions.
pub fn drop_common_life_and_mana(
    npc: &HealNpc,
    player: &HealPlayer,
    rng: &mut dyn DropRng,
    out: &mut Vec<HealDrop>,
) {
    // `:79533-79552` — the num flag.
    let num = if npc.net_id == 16 || npc.net_id == 121 {
        1
    } else if npc.net_id == 81 {
        1
    } else {
        0
    };
    if num == 0
        && roll_luck(rng, player.luck, 6) == 0
        && npc.life_max > 1
        && npc.damage > 0
    {
        // `:79539-79546` — the mana-first pick; the heart's Next(2)
        // draws even when the first hit with full mana.
        if rng.next_range(0, 2) == 0 && player.mana_below_max {
            out.push(HealDrop { item: 184, stack: 1 });
        } else if rng.next_range(0, 2) == 0 && player.life_below_max {
            out.push(HealDrop { item: 58, stack: 1 });
        }
    }
    // `:79553-79556` — the second mana roll (81 still runs it).
    if num == 0
        && roll_luck(rng, player.luck, 2) == 0
        && npc.life_max > 1
        && npc.damage > 0
        && player.mana_below_max
    {
        out.push(HealDrop { item: 184, stack: 1 });
    }
}

/// `NPC.NPCLoot_DropHeals` (`NPC.cs:79098-79195`): the common preamble
/// then the per-kind heart switch.
pub fn drop_heals(
    npc: &HealNpc,
    player: &HealPlayer,
    expert: bool,
    rng: &mut dyn DropRng,
    out: &mut Vec<HealDrop>,
) {
    drop_common_life_and_mana(npc, player, rng, out); // `:79100`
    let heart = |out: &mut Vec<HealDrop>| out.push(HealDrop { item: 58, stack: 1 });
    match npc.kind {
        // `:79103-79119` — the scarecrow decade + the two moon mobs:
        // RollLuck(4).
        305..=314 | 329 | 330 => {
            if roll_luck(rng, player.luck, 4) == 0 {
                heart(out);
            }
        }
        // `:79120-79125` — the Poltergeist: RollLuck(6).
        326 => {
            if roll_luck(rng, player.luck, 6) == 0 {
                heart(out);
            }
        }
        // `:79126-79128` — the Headless Horseman: ALWAYS one heart.
        315 => heart(out),
        // `:79129-79137` — the Mourning Wood: Next(5,11) hearts.
        341 => {
            let num2 = rng.next_range(5, 11);
            for _ in 0..num2 {
                heart(out);
            }
        }
        // `:79138-79145` — the frost trio: RollLuck(5).
        338 | 339 | 340 => {
            if roll_luck(rng, player.luck, 5) == 0 {
                heart(out);
            }
        }
        // `:79146-79151` — the Gingerbread Man: Next(3) != 0 (2/3).
        342 => {
            if rng.next_range(0, 3) != 0 {
                heart(out);
            }
        }
        // `:79152-79164` — the moon bosses: Next(6)+6 hearts.
        325 | 327 | 344 | 345 | 346 => {
            let num = rng.next_range(0, 6) + 6;
            for _ in 0..num {
                heart(out);
            }
        }
        // `:79165-79170` — the Creeper: Next(2)==0 AND life below max.
        267 => {
            if rng.next_range(0, 2) == 0 && player.life_below_max {
                heart(out);
            }
        }
        // `:79171-79178` — the EoW segments: Next(4)==0 AND life below.
        13 | 14 | 15 => {
            if rng.next_range(0, 4) == 0 && player.life_below_max {
                heart(out);
            }
        }
        // `:79179-79187` — the Hungry: classic always, expert 1/5.
        116 | 117 | 118 | 119 => {
            if !expert || rng.next_range(0, 5) == 0 {
                heart(out);
            }
        }
        // `:79188-79193` — the Leech: Next(2)==0.
        139 => {
            if rng.next_range(0, 2) == 0 {
                heart(out);
            }
        }
        _ => {}
    }
}

/// `NPC.DoDeathEvents_DropBossPotionsAndHearts` (`NPC.cs:78945-
/// 79016`): the boss potion stack + the 5-9 hearts + the Badger's Hat
/// latch chain. Runs ONLY for `this.boss` deaths (`:79505-79507`).
pub fn drop_boss_potions_and_hearts(
    kind: i32,
    latches: &mut BadgerLatches,
    rng: &mut dyn DropRng,
    out: &mut Vec<HealDrop>,
) {
    let stack = rng.next_range(5, 16); // `:78947`
    // `:78948-78996` — the potion kind chain (the else-if order).
    let mut num = 28;
    if kind == 113 {
        num = 188;
    } else if kind == 35 {
        num = 188;
    } else if kind == 668 {
        num = 188;
    } else if kind == 222 {
        num = 1134;
    } else if kind == 657 {
        num = 499;
    } else if kind > 113 && kind < 222 {
        num = 499; // the mech band (`:78969-78972`)
    } else if kind == 636 {
        num = 499;
    } else if kind == 245 || kind == 262 {
        num = 499;
    } else if kind == 370 {
        num = 499;
    } else if kind == 395 {
        num = 499;
    } else if kind == 439 {
        num = 499;
    } else if kind == 398 {
        num = 3544;
    }
    out.push(HealDrop { item: num as u16, stack: stack as u32 }); // `:78997`
    let num2 = rng.next_range(0, 5) + 5; // `:78998`
    for _ in 0..num2 {
        out.push(HealDrop { item: 58, stack: 1 }); // `:78999-79002`
    }
    // `:79003-79015` — the Badger's Hat latches.
    if kind == 4 {
        latches.eoc_killed_today = true;
    } else if kind == 113 {
        latches.wof_killed_today = true;
    }
    if latches.eoc_killed_today && latches.wof_killed_today {
        // `ResetBadgerHatTime` (`:78939-78943`).
        latches.eoc_killed_today = false;
        latches.wof_killed_today = false;
        out.push(HealDrop { item: 5004, stack: 1 });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::VecDeque;
    use tr_core::UnifiedRandom;

    /// A scripted DropRng: forced range draws pop off the script
    /// (asserted inside `[min, max)`); an exhausted script returns the
    /// range minimum. The float draws (the luck gates) return 1.0 —
    /// luck 0 never reads them.
    struct ScriptRng {
        draws: VecDeque<i32>,
        ranges: Vec<(i32, i32)>,
    }
    impl ScriptRng {
        fn new(draws: &[i32]) -> Self {
            Self {
                draws: VecDeque::from(draws.to_vec()),
                ranges: Vec::new(),
            }
        }
    }
    impl DropRng for ScriptRng {
        fn next_range(&mut self, min: i32, max: i32) -> i32 {
            self.ranges.push((min, max));
            let v = self.draws.pop_front().unwrap_or(min);
            assert!((min..max).contains(&v), "draw {v} outside [{min}, {max})");
            v
        }
        fn next_float(&mut self) -> f32 {
            1.0
        }
    }

    fn player() -> HealPlayer {
        HealPlayer { luck: 0.0, life_below_max: true, mana_below_max: true }
    }

    fn npc(kind: i32) -> HealNpc {
        HealNpc { net_id: kind, kind, life_max: 100, damage: 10 }
    }

    fn items(out: &[HealDrop]) -> Vec<u16> {
        out.iter().map(|d| d.item).collect()
    }

    #[test]
    fn w4_f35_the_boss_kind_table_is_the_22_setdefaults_rows_plus_the_eow_head() {
        for k in [
            4, 13, 35, 50, 113, 125, 126, 127, 134, 222, 245, 262, 266, 370, 395, 396, 397,
            398, 439, 636, 657, 664, 668,
        ] {
            assert!(is_boss_kind(k), "{k}");
        }
        // The EoW body/tail are NOT bosses (only the head's dynamic
        // latch); the moon mobs + the DD2 minibosses are not either.
        for k in [0, 1, 14, 15, 12, 34, 114, 223, 267, 325, 327, 341, 344, 551, 564, 576, 669] {
            assert!(!is_boss_kind(k), "{k} not a boss");
        }
    }

    #[test]
    fn w4_f35_common_preamble_zero_script_grants_the_star_then_the_mana_star() {
        // RollLuck(6)=0 hits, Next(2)=0 + mana below → 184; the second
        // RollLuck(2)=0 + mana below → another 184.
        let mut rng = ScriptRng::new(&[]);
        let mut out = Vec::new();
        drop_common_life_and_mana(&npc(1), &player(), &mut rng, &mut out);
        assert_eq!(items(&out), vec![184, 184]);
        assert_eq!(rng.ranges, vec![(0, 6), (0, 2), (0, 2)]);
    }

    #[test]
    fn w4_f35_common_preamble_full_mana_falls_to_the_heart_draw() {
        // The first Next(2)=0 hits but mana FULL → the else-if draws
        // its OWN Next(2) (=0) + life below → 58. The second roll's
        // mana gate refuses (no drop, but the RollLuck(2) still drew).
        let mut p = player();
        p.mana_below_max = false;
        let mut rng = ScriptRng::new(&[]);
        let mut out = Vec::new();
        drop_common_life_and_mana(&npc(1), &p, &mut rng, &mut out);
        assert_eq!(items(&out), vec![58]);
        assert_eq!(rng.ranges, vec![(0, 6), (0, 2), (0, 2), (0, 2)]);
    }

    #[test]
    fn w4_f35_common_preamble_the_netid_exclusions() {
        // 16/121: NO draws at all. 81: the first roll skips, the
        // second still draws + grants.
        for id in [16, 121] {
            let mut rng = ScriptRng::new(&[]);
            let mut out = Vec::new();
            drop_common_life_and_mana(&npc(id), &player(), &mut rng, &mut out);
            assert!(out.is_empty(), "netID {id}");
            assert!(rng.ranges.is_empty(), "netID {id} draws nothing");
        }
        let mut rng = ScriptRng::new(&[]);
        let mut out = Vec::new();
        drop_common_life_and_mana(&npc(81), &player(), &mut rng, &mut out);
        assert_eq!(items(&out), vec![184]);
        assert_eq!(rng.ranges, vec![(0, 2)], "only the second RollLuck(2)");
    }

    #[test]
    fn w4_f35_common_preamble_the_life_and_damage_gates_draw_after_the_luck() {
        // lifeMax 1: the RollLuck(6) DRAWS (the C# order), then the
        // gate refuses; the second RollLuck(2) draws too.
        let mut n = npc(1);
        n.life_max = 1;
        let mut rng = ScriptRng::new(&[]);
        let mut out = Vec::new();
        drop_common_life_and_mana(&n, &player(), &mut rng, &mut out);
        assert!(out.is_empty());
        assert_eq!(rng.ranges, vec![(0, 6), (0, 2)]);
    }

    #[test]
    fn w4_f35_drop_heals_the_per_kind_switch_zero_script() {
        // The common preamble's rolls all hit at zero (2 stars), then
        // each kind's own arm. The script feeds the preamble 0s.
        let rows: &[(i32, &[u16])] = &[
            (305, &[184, 184, 58]),        // RollLuck(4) hit
            (326, &[184, 184, 58]),        // RollLuck(6) hit
            (315, &[184, 184, 58]),        // always
            (341, &[184, 184, 58, 58, 58, 58, 58]), // Next(5,11)=5
            (338, &[184, 184, 58]),        // RollLuck(5) hit
            (342, &[184, 184]),            // Next(3)!=0 FALSE at zero
            (325, &[184, 184, 58, 58, 58, 58, 58, 58]), // Next(6)+6=6
            (267, &[184, 184, 58]),        // Next(2)==0 + life below
            (13, &[184, 184, 58]),         // Next(4)==0 + life below
            (116, &[184, 184, 58]),        // classic always
            (139, &[184, 184, 58]),        // Next(2)==0
            (1, &[184, 184]),              // no switch arm
        ];
        for &(kind, want) in rows {
            let mut rng = ScriptRng::new(&[]);
            let mut out = Vec::new();
            drop_heals(&npc(kind), &player(), false, &mut rng, &mut out);
            assert_eq!(items(&out), want, "kind {kind}");
        }
    }

    #[test]
    fn w4_f35_drop_heals_the_gingerbread_two_thirds_and_the_hungry_expert_gate() {
        // 342: Next(3)=1 → heart; the expert Hungry: Next(5)=0 hits,
        // =1 misses. Script past the preamble (3 draws: 6, 2, 2).
        let mut g = ScriptRng::new(&[0, 0, 0, 1]);
        let mut out = Vec::new();
        drop_heals(&npc(342), &player(), false, &mut g, &mut out);
        assert_eq!(items(&out), vec![184, 184, 58]);
        let mut h = ScriptRng::new(&[0, 0, 0, 0]);
        let mut out2 = Vec::new();
        drop_heals(&npc(116), &player(), true, &mut h, &mut out2);
        assert_eq!(items(&out2), vec![184, 184, 58]);
        let mut m = ScriptRng::new(&[0, 0, 0, 1]);
        let mut out3 = Vec::new();
        drop_heals(&npc(116), &player(), true, &mut m, &mut out3);
        assert_eq!(items(&out3), vec![184, 184], "the expert 1/5 miss");
        // The classic Hungry draws NOTHING for the gate.
        let mut c = ScriptRng::new(&[0, 0, 0]);
        let mut out4 = Vec::new();
        drop_heals(&npc(116), &player(), false, &mut c, &mut out4);
        assert_eq!(items(&out4), vec![184, 184, 58]);
        assert_eq!(c.ranges, vec![(0, 6), (0, 2), (0, 2)], "no expert draw");
    }

    #[test]
    fn w4_f35_drop_heals_the_life_gate_refuses_after_the_draw() {
        // The Creeper at full life: the Next(2) DRAWS, the gate
        // refuses — no heart.
        let mut p = player();
        p.life_below_max = false;
        p.mana_below_max = false;
        let mut rng = ScriptRng::new(&[]);
        let mut out = Vec::new();
        drop_heals(&npc(267), &p, false, &mut rng, &mut out);
        assert!(out.is_empty());
        assert_eq!(rng.ranges, vec![(0, 6), (0, 2), (0, 2), (0, 2)]);
    }

    #[test]
    fn w4_f35_boss_potions_the_kind_chain_and_the_heart_count() {
        // The potion chain: 28 default, 188 for 113/35/668, 1134 for
        // 222, 499 for the mech band + 636/245/262/370/395/439/657,
        // 3544 for 398. The hearts: Next(5)+5 = 5 at zero.
        let rows: &[(i32, u16)] = &[
            (50, 28),
            (4, 28),
            (113, 188),
            (35, 188),
            (668, 188),
            (222, 1134),
            (657, 499),
            (125, 499),
            (134, 499),
            (636, 499),
            (245, 499),
            (262, 499),
            (370, 499),
            (395, 499),
            (439, 499),
            (398, 3544),
            (266, 28),
            (664, 28),
        ];
        for &(kind, potion) in rows {
            let mut rng = ScriptRng::new(&[]);
            let mut latches = BadgerLatches::default();
            let mut out = Vec::new();
            drop_boss_potions_and_hearts(kind, &mut latches, &mut rng, &mut out);
            assert_eq!(out[0], HealDrop { item: potion, stack: 5 }, "kind {kind}");
            assert_eq!(out.len(), 6, "the potion + 5 hearts");
            assert!(out[1..].iter().all(|d| *d == HealDrop { item: 58, stack: 1 }));
            assert_eq!(rng.ranges, vec![(5, 16), (0, 5)]);
        }
    }

    #[test]
    fn w4_f35_the_badgers_hat_needs_both_latches_and_resets_them() {
        // EoC alone: latch up, no hat. WoF next: BOTH up → the hat
        // drops + both clear. A third boss: no hat (the latches reset).
        let mut latches = BadgerLatches::default();
        let mut rng = ScriptRng::new(&[]);
        let mut out = Vec::new();
        drop_boss_potions_and_hearts(4, &mut latches, &mut rng, &mut out);
        assert!(latches.eoc_killed_today && !latches.wof_killed_today);
        assert!(!items(&out).contains(&5004));
        let mut out2 = Vec::new();
        drop_boss_potions_and_hearts(113, &mut latches, &mut rng, &mut out2);
        assert!(items(&out2).contains(&5004), "the hat on the second kill");
        assert!(!latches.eoc_killed_today && !latches.wof_killed_today, "the reset");
        let mut out3 = Vec::new();
        drop_boss_potions_and_hearts(50, &mut latches, &mut rng, &mut out3);
        assert!(!items(&out3).contains(&5004));
        // The hat is the LAST drop (after the hearts).
        assert_eq!(*out2.last().unwrap(), HealDrop { item: 5004, stack: 1 });
    }

    /// The C# heal-feed replay on a twin `UnifiedRandom` (luck 0 — the
    /// plain rolls), in the C# call order: the boss tail
    /// (`DoDeathEvents`, `:79505-79507`) BEFORE `NPCLoot_DropHeals`
    /// (`:78898`). The latches start clear (a single kill never drops
    /// the hat — both statics never up).
    fn heals_replay(seed: i32, kind: i32, expert: bool, boss: bool) -> (Vec<HealDrop>, UnifiedRandom) {
        let mut r = UnifiedRandom::new(seed);
        let mut w = Vec::new();
        let p = player();
        let n = npc(kind);
        // The boss tail (`:78945-79016`).
        if boss {
            let stack = r.next_range(5, 16);
            let potion = match kind {
                113 | 35 | 668 => 188,
                222 => 1134,
                657 => 499,
                k if k > 113 && k < 222 => 499,
                636 | 245 | 262 | 370 | 395 | 439 => 499,
                398 => 3544,
                _ => 28,
            };
            w.push(HealDrop { item: potion, stack: stack as u32 });
            for _ in 0..(r.next_range(0, 5) + 5) {
                w.push(HealDrop { item: 58, stack: 1 });
            }
        }
        // The common preamble (`:79531-79557`).
        let num = if n.net_id == 16 || n.net_id == 121 || n.net_id == 81 { 1 } else { 0 };
        if num == 0 && r.next_range(0, 6) == 0 && n.life_max > 1 && n.damage > 0 {
            if r.next_range(0, 2) == 0 && p.mana_below_max {
                w.push(HealDrop { item: 184, stack: 1 });
            } else if r.next_range(0, 2) == 0 && p.life_below_max {
                w.push(HealDrop { item: 58, stack: 1 });
            }
        }
        if num == 0 && r.next_range(0, 2) == 0 && n.life_max > 1 && n.damage > 0 && p.mana_below_max {
            w.push(HealDrop { item: 184, stack: 1 });
        }
        // The switch (`:79101-79194`).
        let heart = |w: &mut Vec<HealDrop>| w.push(HealDrop { item: 58, stack: 1 });
        match kind {
            305..=314 | 329 | 330 => {
                if r.next_range(0, 4) == 0 {
                    heart(&mut w);
                }
            }
            326 => {
                if r.next_range(0, 6) == 0 {
                    heart(&mut w);
                }
            }
            315 => heart(&mut w),
            341 => {
                for _ in 0..r.next_range(5, 11) {
                    heart(&mut w);
                }
            }
            338 | 339 | 340 => {
                if r.next_range(0, 5) == 0 {
                    heart(&mut w);
                }
            }
            342 => {
                if r.next_range(0, 3) != 0 {
                    heart(&mut w);
                }
            }
            325 | 327 | 344 | 345 | 346 => {
                for _ in 0..(r.next_range(0, 6) + 6) {
                    heart(&mut w);
                }
            }
            267 => {
                if r.next_range(0, 2) == 0 && p.life_below_max {
                    heart(&mut w);
                }
            }
            13 | 14 | 15 => {
                if r.next_range(0, 4) == 0 && p.life_below_max {
                    heart(&mut w);
                }
            }
            116 | 117 | 118 | 119 => {
                if !expert || r.next_range(0, 5) == 0 {
                    heart(&mut w);
                }
            }
            139 => {
                if r.next_range(0, 2) == 0 {
                    heart(&mut w);
                }
            }
            _ => {}
        }
        (w, r)
    }

    #[test]
    fn w4_f35_the_heal_feed_replays_the_csharp_draw_order() {
        let kinds = [
            1i32, 16, 81, 121, 305, 310, 314, 315, 325, 326, 327, 329, 330, 338, 341, 342, 344,
            267, 13, 14, 15, 116, 119, 139, 4, 50, 113, 125, 222, 245, 398, 657, 668,
        ];
        for seed in [1i32, 7, 42, 1234, 99999] {
            for &kind in &kinds {
                for expert in [false, true] {
                    let boss = is_boss_kind(kind);
                    let mut rng = UnifiedRandom::new(seed);
                    let mut out = Vec::new();
                    let mut latches = BadgerLatches::default();
                    if boss {
                        drop_boss_potions_and_hearts(kind, &mut latches, &mut rng, &mut out);
                    }
                    drop_heals(&npc(kind), &player(), expert, &mut rng, &mut out);
                    let (want, r2) = heals_replay(seed, kind, expert, boss);
                    assert_eq!(out, want, "seed {seed} kind {kind} expert {expert}");
                    assert_eq!(rng, r2, "seed {seed} kind {kind} expert {expert} stream");
                }
            }
        }
    }
}
