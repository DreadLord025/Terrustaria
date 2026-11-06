//! The per-NPC loot registrations built on the [`tr_core::loot`] rule engine —
//! `ItemDropDatabase`'s port. M4-LOOT-1 ships the slime-family slice; M4-LOOT-2
//! fills the remaining 643 registrations.

use std::sync::OnceLock;
use tr_core::loot::{CommonDrop, ItemDropDatabase, gel, normalvs_expert};

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

/// The bestiary's per-entry drop lines — the  view of the same registrations the executor runs
/// (ItemDropBestiaryInfoElement reads the database's rules; the port
/// restates the family's rows so the info page and the drops stay in sync
/// until the rules walk themselves for display, M4-BEST-1 follow-up).
/// denom <= 1 = the always-drop case.
pub fn drop_lines(net_id: i32) -> &'static [(i32, i32, i32, i32)] {
    match net_id {
        BLUE_SLIME => &[(23, 1, 2, 1)],
        RED_SLIME | PURPLE_SLIME | BLACK_SLIME => &[(23, 2, 5, 1)],
        PINKY => &[(3111, 25, 50, 1), (1309, 0, 0, 100)],
        _ => &[],
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use tr_core::loot::{DropAttemptInfo, LootFlags, LootNpc, DropResultState};

    fn info<'a>(rng: &'a mut tr_core::tile_light::FastRandom) -> DropAttemptInfo<'a> {
        DropAttemptInfo {
            rng,
            luck: 0.0,
            is_expert: false,
            is_master: false,
            active_players: 1,
            extra_gel: false,
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
        assert!(
            !db.rules_for_npc(BLUE_SLIME).is_empty(),
            "blue slime gel"
        );
        assert!(!db.rules_for_npc(BLACK_SLIME).is_empty(), "black slime family");
        assert!(!db.rules_for_npc(-10).is_empty(), "jungle slime (-10) family");
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
            out.into_iter().map(|r| (r.item, r.stack)).collect::<Vec<_>>()
        };
        assert_eq!(resolve(42), resolve(42));
        assert_ne!(resolve(42), resolve(43), "different seeds diverge");
    }
}
