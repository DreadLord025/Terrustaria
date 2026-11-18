//! The per-NPC loot registrations built on the [`tr_core::loot`] rule engine —
//! `ItemDropDatabase`'s port. M4-LOOT-1 ships the slime-family slice; M4-LOOT-2
//! fills the remaining 643 registrations.

use std::sync::OnceLock;
use tr_core::loot::{gel, normalvs_expert, CommonDrop, ItemDropDatabase};

/// The slime colour family's vanilla net ids (`Lang.cs:520-533`,
/// `NPC.SetDefaultsFromNetId`).
pub const BLUE_SLIME: i32 = 1;
pub const GREEN_SLIME: i32 = -3;
pub const PINKY: i32 = -4;
pub const BABY_SLIME: i32 = -5;
pub const BLACK_SLIME: i32 = -6;
pub const PURPLE_SLIME: i32 = -7;
pub const RED_SLIME: i32 = -8;
pub const YELLOW_SLIME: i32 = -9;

/// `ItemDropDatabase.RegisterToNPC` for the ported enemy slice
/// (`ItemDropDatabase.cs:1072-1082`):
/// - the standard slimes (blue 1 and the green-side family) drop
///   `ItemDropRule.Gel(1, 1, 2)`,
/// - red/purple/black drop `Gel(1, 2, 5)`,
/// - pinky drops 25-50 Pink Gel (3111) plus the Bladed Glove
///   `NormalvsExpert(1309, 100, 70)` (the executor skips items the registry
///   does not model yet),
/// - green and baby slimes register nothing (no gel).
pub fn slime_database() -> &'static ItemDropDatabase {
    static DB: OnceLock<ItemDropDatabase> = OnceLock::new();
    DB.get_or_init(|| {
        let mut db = ItemDropDatabase::new();
        let standard = [BLUE_SLIME];
        let tough = [RED_SLIME, PURPLE_SLIME, BLACK_SLIME];
        db.register_to_multiple(&standard, &|| gel(1, 1, 2));
        db.register_to_multiple(&tough, &|| gel(1, 2, 5));
        db.register_to_npc(PINKY, CommonDrop::new(3111, 1, 25, 50));
        db.register_to_npc(PINKY, normalvs_expert(1309, 100, 70));
        db
    })
}

/// The full registration table - `ItemDropDatabase.Populate()` ported by
/// `tools/gen_loot.py` into [`crate::loot_db_gen`] (1405 net-id registrations
/// over the 643 C# register sites, the families expanded per
/// `NPCID.NetIdMap`). The executor resolves a dead NPC's net id against this.
pub fn database() -> &'static ItemDropDatabase {
    static DB: OnceLock<ItemDropDatabase> = OnceLock::new();
    DB.get_or_init(|| {
        let mut db = ItemDropDatabase::new();
        crate::loot_db_gen::populate(&mut db);
        db
    })
}

/// The bestiary's drop rows for one NPC — `BestiaryDatabase.ExtractDropsForNPC`
/// (`BestiaryDatabase.cs:70-88`): the NPC's rules report their rows with a
/// fresh feed (the rate 1.0). The C# keeps them on the bestiary entry; the
/// port derives them on demand from the same database the executor runs.
pub fn drop_rows(net_id: i32) -> Vec<tr_core::loot::DropRateInfo> {
    database().report_droprates(net_id)
}

#[cfg(test)]
mod s1_e_15_tests {
    use super::*;

    #[test]
    fn the_generated_database_reports_the_blue_slimes_gel() {
        // `BestiaryDatabase.ExtractDroprates` over the generated
        // registrations: the blue slime (1) drops the gel (23) 1-2 — the
        // C#'s `ItemDropRule.Gel(1, 1, 2)` row.
        let rows = drop_rows(BLUE_SLIME);
        assert!(!rows.is_empty(), "the blue slime's drop rows");
        // The gel rows: the normal one (1-2) and the extra-gel one (2-4 —
        // `DropBasedOnExtraGel` reports both).
        let gel = rows
            .iter()
            .find(|r| r.item_id == 23 && (r.stack_min, r.stack_max) == (1, 2))
            .expect("the gel row");
        assert!((gel.drop_rate - 1.0).abs() < 1e-6, "always");
        assert!(
            rows.iter()
                .any(|r| r.item_id == 23 && (r.stack_min, r.stack_max) == (2, 4)),
            "the extra-gel row: {rows:?}"
        );
        // The bladed glove (1309) at the normal/expert split.
        assert!(rows.iter().any(|r| r.item_id == 1309), "{rows:?}");
    }

    #[test]
    fn the_pinky_reports_both_of_its_rows() {
        // The pinky (-4): 25-50 Pink Gel (3111) plus the Bladed Glove
        // (1309) at the expert-normal split.
        let rows = drop_rows(PINKY);
        assert!(rows.iter().any(|r| r.item_id == 3111), "{rows:?}");
        assert!(rows.iter().any(|r| r.item_id == 1309), "{rows:?}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tr_core::loot::{DropAttemptInfo, DropResultState, LootFlags, LootNpc};

    fn info<'a>(rng: &'a mut tr_core::tile_light::FastRandom) -> DropAttemptInfo<'a> {
        // No localization resolver: the tests here touch no `NamedNPC` row
        // (the named-hat tests build their own info with a stub resolver).
        DropAttemptInfo {
            rng,
            luck: 0.0,
            is_expert: false,
            is_master: false,
            active_players: 1,
            extra_gel: false,
            is_in_simulation: false,
            get_text_value: None,
            flags: LootFlags::default(),
            npc: LootNpc::default(),
        }
    }

    #[test]
    fn the_database_registers_the_slime_families_and_the_zombies() {
        // `RegisterToNPC(1, Gel)` expands to the negative net-id family
        // (`PrepareNPCNetIDsByTypeDictionary` + `RegisterToNPC`'s loop,
        // `ItemDropDatabase.cs:61-72`).
        let db = database();
        assert!(!db.rules_for_npc(BLUE_SLIME).is_empty(), "blue slime gel");
        assert!(
            !db.rules_for_npc(BLACK_SLIME).is_empty(),
            "black slime family"
        );
        assert!(
            !db.rules_for_npc(-10).is_empty(),
            "jungle slime (-10) family"
        );
        // The zombie (3) carries the full generator table's rows.
        assert!(!db.rules_for_npc(3).is_empty(), "zombie rows");
        // An unregistered id resolves empty.
        assert!(db.rules_for_npc(9999).is_empty());
    }

    #[test]
    fn the_zombie_row_resolves_the_statue_gate() {
        // Zombie 3: `RegisterToNPC(3, ItemDropRule.Common(59, 150))`-style
        // rows; a seeded resolve yields at most the registered items and
        // never an unregistered one.
        let db = database();
        let mut rng = tr_core::tile_light::FastRandom::new(7);
        let mut out = Vec::new();
        tr_core::loot::try_dropping(db, 3, &mut info(&mut rng), &mut out);
        for req in &out {
            assert!(req.item > 0);
            assert!(req.stack > 0);
        }
    }

    #[test]
    fn the_leading_condition_gate_blocks_without_the_flag() {
        // `RegisterToNPC(59, new LeadingConditionRule(new
        // Conditions.RemixSeed())).OnSuccess(Gel)` (the slime family's remix
        // rows): with remix_seed off the chain does not fire.
        let db = database();
        let rules = db.rules_for_npc(59);
        assert!(!rules.is_empty(), "the slime type rows exist");
        let mut rng = tr_core::tile_light::FastRandom::new(11);
        let mut i = info(&mut rng);
        let mut out = Vec::new();
        for rule in rules {
            let r = tr_core::loot::resolve_rule(rule.as_ref(), &mut i, &mut out);
            // No remix seed: the leading gates report DoesntFillConditions.
            assert_ne!(r.state, DropResultState::Success, "no remix world");
        }
        assert!(out.is_empty());
        // With the flag on the chain may fire (the seeded roll decides).
        let mut rng = tr_core::tile_light::FastRandom::new(11);
        let mut i = info(&mut rng);
        i.flags.remix_seed = true;
        let mut out = Vec::new();
        for rule in rules {
            tr_core::loot::resolve_rule(rule.as_ref(), &mut i, &mut out);
        }
        for req in &out {
            assert_eq!(req.item, 23, "gel only from the remix chain");
        }
    }

    #[test]
    fn the_deterministic_streams_stay_seed_pinned() {
        // The same seed resolves the same drops (the executor consumes one
        // stream - the SHA stability the screenshot protocol relies on).
        let db = database();
        let resolve = |seed: u64| {
            let mut rng = tr_core::tile_light::FastRandom::new(seed);
            let mut i = info(&mut rng);
            let mut out = Vec::new();
            tr_core::loot::try_dropping(db, 1, &mut i, &mut out);
            out.into_iter()
                .map(|r| (r.item, r.stack))
                .collect::<Vec<_>>()
        };
        assert_eq!(resolve(42), resolve(42));
        assert_ne!(resolve(42), resolve(43), "different seeds diverge");
    }
}

#[cfg(test)]
mod s1_e_14_tests {
    use super::*;
    use std::collections::VecDeque;
    use tr_core::loot::{
        resolve_rule, statue_style_to_item, try_dropping, DropAttemptInfo, DropResultState,
        DropRng, LootFlags, LootNpc,
    };

    /// The Empress's without-repeats options (`ItemDropDatabase.cs:610` — the
    /// C# ctor passes dropCount 2 and ten options; the plan's "11" miscounted).
    const EMPRESS_BUNDLE: [i32; 10] = [3063, 3389, 3065, 1553, 3930, 3541, 3570, 3571, 3569, 5480];
    const MECH_SPAWNERS: [i32; 3] = [556, 544, 557];
    /// Skeletron's five RedHat items (`ItemDropDatabase.cs:570-574`).
    const RED_HAT_ITEMS: [i32; 5] = [5624, 5625, 5626, 5737, 5628];

    /// A scripted rng: the forced draws pop off a script (each asserted
    /// inside its requested range); once the script runs out every draw
    /// returns the range's minimum — the neutral pass-through that keeps a
    /// whole-database pass running (every `Next(0, N)` chance roll hits, the
    /// stack rolls return their minimum). The short-circuit / registration
    /// pins that seed hunting cannot force.
    struct ScriptRng {
        draws: VecDeque<i32>,
        ranges: VecDeque<(i32, i32)>,
    }
    impl ScriptRng {
        fn with_forced(draw: i32) -> Self {
            Self {
                draws: VecDeque::from(vec![draw]),
                ranges: VecDeque::new(),
            }
        }
    }
    impl DropRng for ScriptRng {
        fn next_range(&mut self, min: i32, max: i32) -> i32 {
            self.ranges.push_back((min, max));
            let v = self.draws.pop_front().unwrap_or(min);
            assert!((min..max).contains(&v), "draw {v} outside [{min}, {max})");
            v
        }
        fn next_float(&mut self) -> f32 {
            panic!("no scripted float for this test")
        }
    }

    fn s1_info<'a>(
        rng: &'a mut dyn DropRng,
        npc: LootNpc<'a>,
        flags: LootFlags,
        get_text_value: Option<&'a dyn Fn(&str) -> String>,
    ) -> DropAttemptInfo<'a> {
        DropAttemptInfo {
            rng,
            luck: 0.0,
            is_expert: false,
            is_master: false,
            active_players: 1,
            extra_gel: false,
            is_in_simulation: false,
            get_text_value,
            flags,
            npc,
        }
    }

    /// The named-NPC resolver stub — `Language.GetTextValue` with the
    /// unknown-keys-keep-the-key rule (`localization/mod.rs:418`).
    fn names(key: &str) -> String {
        match key {
            "GuideNames.Andrew" => "Andrew".into(),
            "SteampunkerNames.Whitney" => "Whitney".into(),
            "PainterNames.Jim" => "Jim".into(),
            _ => key.to_string(),
        }
    }

    #[test]
    fn s1_e_14_empress_drops_two_distinct_options_behind_not_expert() {
        // DW-2.1 — the registered bundle (the last of 398's rows, the C#
        // `RegisterBoss_MoonLord` order) drops exactly 2 distinct options of
        // the ten, per a same-seed replay of the Next(count)/RemoveAt order.
        let db = database();
        let rules = db.rules_for_npc(398);
        assert!(rules.len() >= 8, "the spliced Empress rows");
        let bundle = rules.last().expect("the bundle row");
        let mut rng = tr_core::tile_light::FastRandom::new(7);
        let mut i = s1_info(
            &mut rng,
            LootNpc {
                value: 1.0,
                is_boss: true,
                ..Default::default()
            },
            LootFlags::default(),
            None,
        );
        let mut out = Vec::new();
        let r = resolve_rule(bundle.as_ref(), &mut i, &mut out);
        assert_eq!(r.state, DropResultState::Success);
        assert_eq!(out.len(), 2, "dropCount 2: {out:?}");
        assert_ne!(out[0].item, out[1].item, "without repeats");
        for req in &out {
            assert!(EMPRESS_BUNDLE.contains(&req.item), "an option: {req:?}");
            assert_eq!(req.stack, 1);
        }
        let mut r2 = tr_core::tile_light::FastRandom::new(7);
        let mut avail: Vec<i32> = EMPRESS_BUNDLE.to_vec();
        let mut expect = Vec::new();
        for _ in 0..2 {
            let idx = r2.next_range(0, avail.len() as i32);
            expect.push(avail.remove(idx as usize));
        }
        assert_eq!(
            [out[0].item, out[1].item],
            expect.as_slice(),
            "the pinned pair"
        );

        // The full executor pass: normal mode yields the same two (the other
        // rows drop none of the options); the NotExpert leader refuses in
        // expert mode.
        let options_dropped = |expert: bool, seed: u64| {
            let mut rng = tr_core::tile_light::FastRandom::new(seed);
            let mut i = s1_info(
                &mut rng,
                LootNpc {
                    value: 1.0,
                    is_boss: true,
                    ..Default::default()
                },
                LootFlags::default(),
                None,
            );
            i.is_expert = expert;
            let mut out = Vec::new();
            try_dropping(db, 398, &mut i, &mut out);
            out.into_iter()
                .filter(|r| EMPRESS_BUNDLE.contains(&r.item))
                .map(|r| r.item)
                .collect::<Vec<_>>()
        };
        let normal = options_dropped(false, 7);
        assert_eq!(normal.len(), 2);
        assert!(normal[0] != normal[1]);
        assert!(options_dropped(true, 7).is_empty(), "expert: no bundle");
    }

    #[test]
    fn s1_e_14_global_rules_register_and_stay_inert_outside_their_gates() {
        // DW-2.2 — the mech spawner global fires for a valuable hardmode npc
        // with a mech standing (arm 1, short-circuiting the other arms), and
        // refuses pre-hardmode / all-downed; the slime-body global refuses a
        // slime without a carried item.
        let db = database();
        let zombie = |hard_mode: bool, d1: bool, d2: bool, d3: bool| {
            let mut rng = ScriptRng::with_forced(0);
            let mut i = s1_info(
                &mut rng,
                LootNpc {
                    value: 1.0,
                    life_max: 45,
                    has_player_target: true,
                    ..Default::default()
                },
                LootFlags {
                    hard_mode,
                    downed_mech1: d1,
                    downed_mech2: d2,
                    downed_mech3: d3,
                    ..Default::default()
                },
                None,
            );
            let mut out = Vec::new();
            try_dropping(db, 3, &mut i, &mut out);
            out.into_iter().map(|r| r.item).collect::<Vec<i32>>()
        };
        let hit = zombie(true, false, true, true);
        assert!(hit.contains(&556), "arm 1 fired: {hit:?}");
        assert!(
            !hit.contains(&544) && !hit.contains(&557),
            "short-circuit: {hit:?}"
        );
        assert!(
            zombie(false, false, true, true)
                .iter()
                .all(|i| !MECH_SPAWNERS.contains(i)),
            "pre-hardmode: inert"
        );
        assert!(
            zombie(true, true, true, true)
                .iter()
                .all(|i| !MECH_SPAWNERS.contains(i)),
            "all mechs downed: inert"
        );

        // The slime-body global: a blue slime with no carried item drops
        // nothing from it; ai[1] = 8 rides the FIRST global draw (the
        // globals run before the per-npc gel rows) with the 5..10 stack.
        let slime = |ai1: f32, seed: u64| {
            let mut rng = tr_core::tile_light::FastRandom::new(seed);
            let mut i = s1_info(
                &mut rng,
                LootNpc {
                    kind: 1,
                    value: 1.0,
                    ai: [0.0, ai1, 0.0, 0.0],
                    ..Default::default()
                },
                LootFlags::default(),
                None,
            );
            let mut out = Vec::new();
            try_dropping(db, 1, &mut i, &mut out);
            out
        };
        assert!(
            slime(0.0, 7).iter().all(|r| r.item != 8),
            "no carried item: the global refuses"
        );
        let carried = slime(8.0, 7);
        assert_eq!(carried[0].item, 8, "the global's drop leads the output");
        let mut r2 = tr_core::tile_light::FastRandom::new(7);
        let stack = r2.next_range(5, 11);
        assert_eq!(carried[0].stack, stack, "the Next(min, max+1) family roll");
    }

    #[test]
    fn s1_e_14_statue_rule_on_690_and_the_named_npc_hat_rows_register() {
        // DW-2.3 — NPC 690's statue rule maps ai[1] through
        // StatueStyleToItem; the three hat rows drop on the localized
        // given-name match and refuse on a mismatch / an unwired resolver.
        let db = database();
        let mut rng = tr_core::tile_light::FastRandom::new(7);
        let mut i = s1_info(
            &mut rng,
            LootNpc {
                kind: 690,
                value: 1.0,
                ai: [0.0, 42.0, 0.0, 0.0],
                ..Default::default()
            },
            LootFlags::default(),
            None,
        );
        let mut out = Vec::new();
        try_dropping(db, 690, &mut i, &mut out);
        assert_eq!(out.len(), 1, "690's only row: {out:?}");
        assert_eq!(out[0].item, statue_style_to_item(42));
        assert_eq!(out[0].stack, 1);
        // No spawn style: the row refuses.
        let mut rng = tr_core::tile_light::FastRandom::new(7);
        let mut i = s1_info(
            &mut rng,
            LootNpc {
                kind: 690,
                value: 1.0,
                ..Default::default()
            },
            LootFlags::default(),
            None,
        );
        let mut out = Vec::new();
        try_dropping(db, 690, &mut i, &mut out);
        assert!(out.is_empty(), "ai[1] = 0: inert");

        // The Guide's only row: "Andrew" (localized from the key) drops the
        // green cap (867) stack 1; "Aaron" and an unwired resolver refuse.
        let guide = |given: Option<&str>, wired: bool| {
            let mut rng = tr_core::tile_light::FastRandom::new(7);
            let resolver: Option<&dyn Fn(&str) -> String> = if wired { Some(&names) } else { None };
            let mut i = s1_info(
                &mut rng,
                LootNpc {
                    kind: 22,
                    value: 1.0,
                    given_name: given,
                    ..Default::default()
                },
                LootFlags::default(),
                resolver,
            );
            let mut out = Vec::new();
            try_dropping(db, 22, &mut i, &mut out);
            out.into_iter().map(|r| r.item).collect::<Vec<i32>>()
        };
        assert_eq!(guide(Some("Andrew"), true), vec![867], "the named Guide");
        assert!(
            guide(Some("Aaron"), true).is_empty(),
            "a different name refuses"
        );
        assert!(guide(None, true).is_empty(), "unnamed refuses");
        assert!(
            guide(Some("Andrew"), false).is_empty(),
            "an unwired resolver refuses"
        );
        // The Steampunker's row (4372 / Whitney) through the same seam.
        let mut rng = tr_core::tile_light::FastRandom::new(7);
        let mut i = s1_info(
            &mut rng,
            LootNpc {
                kind: 178,
                value: 1.0,
                given_name: Some("Whitney"),
                ..Default::default()
            },
            LootFlags::default(),
            Some(&names),
        );
        let mut out = Vec::new();
        try_dropping(db, 178, &mut i, &mut out);
        assert_eq!(out[0].item, 4372, "the Steampunker's item");
    }

    #[test]
    fn s1_e_14_skeletron_red_hat_rows_gate_on_the_state_machine() {
        // DW-2.3 — the five RedHat rows fire together when the state machine
        // passes (35/33 by ai[3], 36/32 by localAI[3]) and never when it
        // fails; the zero-script's hits also pin the spliced boss rows.
        let db = database();
        let skeletron = |kind: i32, ai3: f32, local3: f32| {
            let mut rng = ScriptRng::with_forced(0);
            let mut i = s1_info(
                &mut rng,
                LootNpc {
                    kind,
                    value: 1.0,
                    ai: [0.0, 0.0, 0.0, ai3],
                    local_ai: [0.0, 0.0, 0.0, local3],
                    ..Default::default()
                },
                LootFlags::default(),
                None,
            );
            let mut out = Vec::new();
            try_dropping(db, 35, &mut i, &mut out);
            out.into_iter()
                .map(|r| (r.item, r.stack))
                .collect::<Vec<_>>()
        };
        let fire = skeletron(35, 1.0, 0.0);
        for id in RED_HAT_ITEMS {
            assert!(
                fire.contains(&(id, 1)),
                "Skeletron with the red hat drops {id}: {fire:?}"
            );
        }
        assert!(
            fire.contains(&(1281, 1)),
            "the spliced NotExpert row: {fire:?}"
        );
        assert!(
            fire.contains(&(4993, 1)),
            "the spliced common row: {fire:?}"
        );
        assert!(
            skeletron(35, 0.0, 0.0)
                .iter()
                .all(|(i, _)| !RED_HAT_ITEMS.contains(i)),
            "no hat: the machine refuses"
        );
        for id in RED_HAT_ITEMS {
            assert!(skeletron(33, 1.0, 0.0).contains(&(id, 1)), "the head: {id}");
            assert!(
                skeletron(36, 0.0, 1.0).contains(&(id, 1)),
                "arm by localAI[3]: {id}"
            );
            assert!(
                !skeletron(32, 1.0, 0.0).iter().any(|(i, _)| *i == id),
                "type 32 reads localAI[3], not ai[3]: {id}"
            );
        }
    }
}
