//! The scenario-test loadouts (`Testing/ScenarioTestLoadout.cs:12-279`) —
//! the `/saveloadout` / `/loadout` / `/loadouts` debug commands' data layer
//! (W4-d85 row 8's backend): the JSON record shape, the `Loadouts` directory
//! under `Main.SavePath`, and `Apply`'s player write.
//!
//! Deviations (docs/ANALYSIS.md "Debug lists (W4-d85)"): the JSON reader is
//! `serde_json::Value` walks rather than Newtonsoft's
//! `EasyDeserializationJsonContractResolver` (the shipped files carry the
//! plain PascalCase field names); `MinionRespawner.RestoreMinionsFor` has no
//! port yet, so `apply` kills the live minions (`ScenarioTestLoadout.cs:
//! 126-133`) but does not respawn the recorded ones.

use std::path::{Path, PathBuf};

use crate::inv;
use crate::item::ItemSlot;
use crate::itemdata;
use crate::lang_tables;
use crate::player::Player;

/// `ScenarioTestLoadout.ItemRecord` (`ScenarioTestLoadout.cs:14-23`).
#[derive(Clone, Debug, PartialEq)]
pub struct ItemRecord {
    pub index: usize,
    /// `ItemID.Search.GetName(item.type)` — the const name.
    pub kind_name: String,
    pub prefix: u16,
    pub stack: u32,
}

/// `ScenarioTestLoadout.BuffRecord` (`:25-32`).
#[derive(Clone, Debug, PartialEq)]
pub struct BuffRecord {
    pub index: usize,
    pub kind_name: String,
    pub time: i32,
}

/// `ScenarioTestLoadout.MinionRecord` (`:34-39`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MinionRecord {
    pub item_type: u16,
    pub item_prefix: u16,
}

/// `ScenarioTestLoadout` (`:41-65`) — every field Newtonsoft serialises
/// (`DefaultValueHandling.Ignore` drops the defaults on write; the reader
/// tolerates their absence).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ScenarioTestLoadout {
    pub day_time: bool,
    pub time_of_day: i32,
    pub selected_item: usize,
    pub life: i32,
    pub life_max: i32,
    pub mana: i32,
    pub mana_max: i32,
    pub minions: Vec<MinionRecord>,
    pub items_inventory: Vec<ItemRecord>,
    pub items_ammo: Vec<ItemRecord>,
    pub items_armor: Vec<ItemRecord>,
    pub items_equipment: Vec<ItemRecord>,
    pub buffs: Vec<BuffRecord>,
}

/// `ScenarioTestLoadout.LoadoutsPath` (`:271`): `Main.SavePath/Loadouts` —
/// the port's `Terrustaria/Loadouts` (`save.rs`'s mirror).
pub fn loadouts_dir() -> Option<PathBuf> {
    Some(tr_core::save::default_world_dir()?.parent()?.join("Loadouts"))
}

/// `ItemID.Search.TryGetId` (`ID/ItemID.cs`'s name→id reverse) over the
/// port's const-name table (`itemdata::NAMES`).
pub fn item_id_for_name(name: &str) -> Option<u16> {
    itemdata::NAMES
        .iter()
        .position(|&n| n == name)
        .map(|i| i as u16)
}

/// `BuffID.Search.GetId` over the port's const-name table
/// (`lang_tables::BUFF_ID`); an unknown name answers 0 (the C# `GetId`
/// throws — the loadout files only carry registered names).
pub fn buff_id_for_name(name: &str) -> i32 {
    lang_tables::BUFF_ID
        .iter()
        .find(|(_, n)| *n == name)
        .map(|(id, _)| *id)
        .unwrap_or(0)
}

fn read_items(v: &serde_json::Value, key: &str) -> Vec<ItemRecord> {
    let mut out = Vec::new();
    if let Some(arr) = v.get(key).and_then(|x| x.as_array()) {
        for e in arr {
            let Some(kind_name) = e.get("Type").and_then(|x| x.as_str()) else {
                continue;
            };
            out.push(ItemRecord {
                index: e.get("Index").and_then(|x| x.as_u64()).unwrap_or(0) as usize,
                kind_name: kind_name.to_string(),
                prefix: e.get("Prefix").and_then(|x| x.as_u64()).unwrap_or(0) as u16,
                stack: e.get("Stack").and_then(|x| x.as_u64()).unwrap_or(0) as u32,
            });
        }
    }
    out
}

/// `ScenarioTestLoadout.Deserialize` (`:193-196`): the JSON object's fields.
/// A non-object or malformed JSON answers `None` (the `TryReadFromFilePath`
/// catch, `:234-237`).
pub fn deserialize(json: &str) -> Option<ScenarioTestLoadout> {
    let v: serde_json::Value = serde_json::from_str(json).ok()?;
    if !v.is_object() {
        return None;
    }
    let mut minions = Vec::new();
    if let Some(arr) = v.get("Minions").and_then(|x| x.as_array()) {
        for e in arr {
            minions.push(MinionRecord {
                item_type: e.get("ItemType").and_then(|x| x.as_u64()).unwrap_or(0) as u16,
                item_prefix: e.get("ItemPrefix").and_then(|x| x.as_u64()).unwrap_or(0) as u16,
            });
        }
    }
    let mut buffs = Vec::new();
    if let Some(arr) = v.get("Buffs").and_then(|x| x.as_array()) {
        for e in arr {
            let Some(kind_name) = e.get("Type").and_then(|x| x.as_str()) else {
                continue;
            };
            buffs.push(BuffRecord {
                index: e.get("Index").and_then(|x| x.as_u64()).unwrap_or(0) as usize,
                kind_name: kind_name.to_string(),
                time: e.get("Time").and_then(|x| x.as_i64()).unwrap_or(0) as i32,
            });
        }
    }
    Some(ScenarioTestLoadout {
        day_time: v.get("DayTime").and_then(|x| x.as_bool()).unwrap_or(false),
        time_of_day: v.get("TimeOfDay").and_then(|x| x.as_i64()).unwrap_or(0) as i32,
        selected_item: v.get("SelectedItem").and_then(|x| x.as_u64()).unwrap_or(0) as usize,
        life: v.get("Life").and_then(|x| x.as_i64()).unwrap_or(0) as i32,
        life_max: v.get("LifeMax").and_then(|x| x.as_i64()).unwrap_or(0) as i32,
        mana: v.get("Mana").and_then(|x| x.as_i64()).unwrap_or(0) as i32,
        mana_max: v.get("ManaMax").and_then(|x| x.as_i64()).unwrap_or(0) as i32,
        minions,
        items_inventory: read_items(&v, "Items_Inventory"),
        items_ammo: read_items(&v, "Items_Ammo"),
        items_armor: read_items(&v, "Items_Armor"),
        items_equipment: read_items(&v, "Items_Equipment"),
        buffs,
    })
}

/// `ScenarioTestLoadout.TryReadFromFilePath` (`:222-238`): missing file or
/// any read/parse failure answers `None`.
pub fn try_read_from_file_path(path: &Path) -> Option<ScenarioTestLoadout> {
    let text = std::fs::read_to_string(path).ok()?;
    deserialize(&text)
}

/// `ScenarioTestLoadout.FindAll` (`:260-267`): `Directory.GetFiles(path,
/// "*json")` — every `*json` file, in the OS directory order.
pub fn find_all(dir: &Path) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut out: Vec<PathBuf> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            p.is_file()
                && p.file_name()
                    .and_then(|n| n.to_str())
                    .is_some_and(|n| n.ends_with("json"))
        })
        .collect();
    out.sort();
    out
}

/// `Apply(Item[] items, List<ItemRecord>)` (`:179-191`): a record whose
/// type name resolves writes a fresh item at its index (unknown names are
/// skipped, `TryGetId`'s false arm). The port's split arrays: inventory
/// 0..49, ammo records target 54..57 (`inv::ammo_slot_index`), armor and
/// equipment their own arrays.
pub fn apply_items(slots: &mut [ItemSlot], records: &[ItemRecord], index_base: usize) {
    for record in records {
        let Some(id) = item_id_for_name(&record.kind_name) else {
            continue; // `TryGetId` false (`:183`)
        };
        let Some(local) = record.index.checked_sub(index_base) else {
            continue;
        };
        if local < slots.len() {
            slots[local] = ItemSlot {
                kind: id,
                name: crate::registry::name(id),
                count: record.stack,
                prefix: record.prefix,
            };
        }
    }
}

/// The row-preview twin of [`apply_items`] over a sparse `Option` array —
/// the `UIDebugLoadoutItem` ctor's `Item[]` starts all-null and only the
/// records' indices fill (`UIDebugLoadoutItem.cs:47-51`); a null cell draws
/// no icon (`:56`). The live [`apply_items`] writes onto the player's real
/// (non-null) slots instead.
pub fn apply_items_sparse(
    slots: &mut [Option<ItemSlot>],
    records: &[ItemRecord],
    index_base: usize,
) {
    for record in records {
        let Some(id) = item_id_for_name(&record.kind_name) else {
            continue;
        };
        let Some(local) = record.index.checked_sub(index_base) else {
            continue;
        };
        if local < slots.len() {
            slots[local] = Some(ItemSlot {
                kind: id,
                name: crate::registry::name(id),
                count: record.stack,
                prefix: record.prefix,
            });
        }
    }
}

/// `ScenarioTestLoadout.Apply(Player)` (`:88-106`) over the port's state:
/// the clock (single-player arm — `Main.netMode != 1`), the life/mana
/// stats, the selection request, then the four item groups, the minion
/// kill and the buffs. `inventory`/`equip` are the App's arrays (the
/// player's own `Player.inventory` in the C#); `world` carries the clock.
pub fn apply(
    loadout: &ScenarioTestLoadout,
    player: &mut Player,
    inventory: &mut [ItemSlot; crate::item::INV],
    equip: &mut inv::EquipSlots,
    world: &mut tr_core::World,
    projectiles: &mut crate::projectile::ProjectilePool,
    my_player: u8,
    selection: &mut crate::hotbar::SelectionState,
    selected: usize,
    is_client: bool,
) {
    // `:90-94`: the clock writes only off-client.
    if !is_client {
        world.day_time = loadout.day_time;
        world.time = loadout.time_of_day as f64;
    }
    player.hp = loadout.life;
    player.max_hp = loadout.life_max;
    player.mana = loadout.mana;
    player.mana_max = loadout.mana_max;
    // `selectedItemState.Select(SelectedItem)` (`:99`).
    let air = inventory
        .get(loadout.selected_item)
        .is_none_or(|s| s.kind == 0);
    selection.select(selected, loadout.selected_item, air);
    // `:100-103`: inventory, ammo (indices 54..57), armor, misc equips.
    apply_items(inventory, &loadout.items_inventory, 0);
    apply_items(&mut equip.ammo, &loadout.items_ammo, 54);
    apply_items(&mut equip.armor, &loadout.items_armor, 0);
    apply_items(&mut equip.misc_equips, &loadout.items_equipment, 0);
    // `Apply(player, Minions)` (`:124-137`): every owned minion dies;
    // `MinionRespawner.RestoreMinionsFor` has no port (deviation).
    for i in 0..projectiles.slots().len() {
        let p = &projectiles.slots()[i];
        if p.active && p.owner == my_player && p.minion_slots > 0.0 {
            projectiles.kill(i);
        }
    }
    // `Apply(player, Buffs)` (`:155-162`): direct slot writes, no AddBuff.
    for record in &loadout.buffs {
        if record.index < player.buffs.buff_type.len() {
            player.buffs.buff_type[record.index] = buff_id_for_name(&record.kind_name);
            player.buffs.buff_time[record.index] = record.time;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The JSON shape round-trips the record fields (`Deserialize`,
    /// `ScenarioTestLoadout.cs:193-196`).
    #[test]
    fn the_json_fields_deserialize() {
        let json = r#"{
            "DayTime": true, "TimeOfDay": 27000, "SelectedItem": 2,
            "Life": 300, "LifeMax": 400, "Mana": 100, "ManaMax": 200,
            "Items_Inventory": [ { "Index": 0, "Type": "CopperShortsword", "Prefix": 81, "Stack": 1 } ],
            "Items_Ammo": [ { "Index": 54, "Type": "WoodenArrow", "Stack": 250 } ],
            "Items_Armor": [ { "Index": 0, "Type": "CopperHelmet", "Stack": 1 } ],
            "Buffs": [ { "Index": 0, "Type": "WellFed", "Time": 3600 } ],
            "Minions": [ { "ItemType": 3611, "ItemPrefix": 0 } ]
        }"#;
        let l = deserialize(json).unwrap();
        assert!(l.day_time);
        assert_eq!(l.time_of_day, 27000);
        assert_eq!(l.selected_item, 2);
        assert_eq!((l.life, l.life_max, l.mana, l.mana_max), (300, 400, 100, 200));
        assert_eq!(l.items_inventory[0].kind_name, "CopperShortsword");
        assert_eq!(l.items_inventory[0].prefix, 81);
        assert_eq!(l.items_ammo[0].index, 54);
        assert_eq!(l.buffs[0].time, 3600);
        assert_eq!(l.minions[0].item_type, 3611);
        // Absent fields keep the defaults (`DefaultValueHandling.Ignore`).
        let bare = deserialize("{}").unwrap();
        assert_eq!(bare, ScenarioTestLoadout::default());
        assert!(deserialize("not json").is_none());
        assert!(deserialize("[1,2]").is_none());
    }

    /// `Apply(items, records)` writes only resolvable names at their index
    /// (`:179-191`); the ammo group's indices are the vanilla 54..57.
    #[test]
    fn apply_items_resolves_names_and_skips_unknown() {
        let mut inv = [crate::item::EMPTY_SLOT; crate::item::INV];
        let records = vec![
            ItemRecord { index: 3, kind_name: "CopperShortsword".into(), prefix: 0, stack: 1 },
            ItemRecord { index: 4, kind_name: "NoSuchItem".into(), prefix: 0, stack: 1 },
        ];
        apply_items(&mut inv, &records, 0);
        assert_eq!(inv[3].kind, item_id_for_name("CopperShortsword").unwrap());
        assert_eq!(inv[4], crate::item::EMPTY_SLOT);
        let mut ammo = [crate::item::EMPTY_SLOT; inv::AMMO];
        let ammo_records = vec![ItemRecord { index: 55, kind_name: "WoodenArrow".into(), prefix: 0, stack: 99 }];
        apply_items(&mut ammo, &ammo_records, 54);
        assert_eq!(ammo[1].count, 99, "index 55 is ammo cell 1");
        assert_eq!(ammo[0], crate::item::EMPTY_SLOT);
    }

    /// `buff_id_for_name` reverses the const table (`BuffID.Search.GetId`).
    #[test]
    fn the_buff_name_reverse_matches_the_table() {
        assert_eq!(buff_id_for_name("WellFed"), 26);
        assert_eq!(buff_id_for_name("NotABuff"), 0);
    }
}
