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
