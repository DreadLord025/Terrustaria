//! The scenario-test loadouts (`Testing/ScenarioTestLoadout.cs:12-279`) —
//! the `/saveloadout` / `/loadout` / `/loadouts` debug commands' data layer
//! (W4-d85 row 8's backend): the JSON record shape, the `Loadouts` directory
//! under `Main.SavePath`, and `Apply`'s player write.
//!
//! Deviations (docs/ANALYSIS.md "Debug lists (W4-d85)"): the JSON reader is
//! `serde_json::Value` walks rather than Newtonsoft's
//! `EasyDeserializationJsonContractResolver` (the shipped files carry the
//! plain PascalCase field names); the minion-respawn half's silent shot
//! ports the summon-staff spawn arms only (`respawn_minions`) — the full
//! `ItemCheck_Shoot` per-item tail stays on the use path.

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
/// kill + respawn and the buffs. `inventory`/`equip` are the App's arrays
/// (the player's own `Player.inventory` in the C#); `world` carries the
/// clock. `cursor_world` is the screen-centre world point
/// (`MinionRespawner.RestoreMinionsFor` parks the mouse at
/// `screenWidth/2, screenHeight/2`, `MinionRespawner.cs:36-39` — the App
/// passes its camera, which IS that point); `rng` feeds the staff arms'
/// kind picks (the use path's `UpdatePlayers` stream convention).
#[allow(clippy::too_many_arguments)]
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
    cursor_world: glam::Vec2,
    rng: &mut tr_core::UnifiedRandom,
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
    // `Apply(player, Minions)` (`:124-137`): every owned minion dies, then
    // `MinionRespawner.Replace` + `RestoreMinionsFor` respawns the recorded
    // ones from the matching inventory items.
    for i in 0..projectiles.slots().len() {
        let p = &projectiles.slots()[i];
        if p.active && p.owner == my_player && p.minion_slots > 0.0 {
            projectiles.kill(i);
        }
    }
    respawn_minions(
        &loadout.minions,
        player,
        inventory,
        world,
        projectiles,
        my_player,
        cursor_world,
        rng,
    );
    // `Apply(player, Buffs)` (`:155-162`): direct slot writes, no AddBuff.
    for record in &loadout.buffs {
        if record.index < player.buffs.buff_type.len() {
            player.buffs.buff_type[record.index] = buff_id_for_name(&record.kind_name);
            player.buffs.buff_time[record.index] = record.time;
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn respawn_minions(
    records: &[MinionRecord],
    player: &mut Player,
    inventory: &[ItemSlot; crate::item::INV],
    world: &tr_core::World,
    projectiles: &mut crate::projectile::ProjectilePool,
    my_player: u8,
    cursor_world: glam::Vec2,
    rng: &mut tr_core::UnifiedRandom,
) {
    for record in records {
        try_respawn(
            record,
            player,
            inventory,
            world,
            projectiles,
            my_player,
            cursor_world,
            rng,
        );
    }
}

/// `MinionSpawnFromInventoryItem.TryRespawn` (`MinionSpawnFromInventoryItem.
/// cs:30-47`): the FIRST inventory 0..49 slot whose type AND prefix match
/// (`FindMatchingItem` `:49-61`, `ItemMatches` `:21-28`); no match returns
/// silently. A matching item with `buffType > 0` grants `AddBuff(buffType,
/// buffTime == 0 ? 3600 : buffTime)` (`:37-45`), then `SilentlyShootItem`
/// (`Player.cs:47051-47054` — `ItemCheck_Shoot(whoAmI, item, 0,
/// withAudioVisualFeedback: false)`). The port routes the shot through
/// [`silently_shoot_summon`]; the non-summon per-item tail of
/// `ItemCheck_Shoot` stays on the use path (a loadout's `Minions` records
/// are `TrackMinionSpawnFromItemUse` spawns — summon staffs only).
#[allow(clippy::too_many_arguments)]
fn try_respawn(
    record: &MinionRecord,
    player: &mut Player,
    inventory: &[ItemSlot; crate::item::INV],
    world: &tr_core::World,
    projectiles: &mut crate::projectile::ProjectilePool,
    my_player: u8,
    cursor_world: glam::Vec2,
    rng: &mut tr_core::UnifiedRandom,
) {
    // `FindMatchingItem` (`:49-61`): inventory[0..50], type + prefix.
    let Some(item) = inventory[..50]
        .iter()
        .find(|s| s.kind == record.item_type && s.prefix == record.item_prefix)
        .copied()
    else {
        return;
    };
    let Some(def) = crate::registry::item(item.kind) else {
        return;
    };
    // `:37-45`: the buff grant — `num = item.buffTime; if (num == 0) num =
    // 3600; player.AddBuff(item.buffType, num)`.
    if def.buff_type > 0 {
        let time = if def.buff_time == 0 { 3600 } else { def.buff_time };
        player.buffs.add_buff(def.buff_type, time);
    }
    silently_shoot_summon(
        &item,
        def,
        player,
        world,
        projectiles,
        my_player,
        cursor_world,
        rng,
    );
}

/// `Player.SilentlyShootItem`'s summon-staff half — `ItemCheck_Shoot(i,
/// sItem, 0, withAudioVisualFeedback: false)` (`Player.cs:47051-47054`)
/// over the `SpawnMinionOnCursor` arms (`:48436-48559`, the i25 use-path
/// census): weaponDamage 0 (`Damage` stays 0 through the arms — the
/// minion's own damage is the UNSCALED prefixed `sItem.damage`, the C#
/// local `damage` `:47060`), `GetWeaponKnockback`'s minionKB fold
/// (`:47151`), no `ApplyItemTime` (the false flag, `:47185`), the cursor
/// point + reach clamp (`SpawnMinionOnCursor` `:49575-49577`) and the
/// per-staff ai seeds (`:49582-49598`). The `Main.rand` kind picks draw
/// from the caller's stream (the use path's `UpdatePlayers` convention).
/// The Stardust Dragon chain (`:48498-48554`) respawns head+body+body+tail
/// fresh at the cursor when no chain lives (the Apply kill just emptied
/// it) — the grow-existing-chain arm is unreachable from a loadout apply.
#[allow(clippy::too_many_arguments)]
fn silently_shoot_summon(
    item: &ItemSlot,
    def: &'static crate::registry::ItemDef,
    player: &mut Player,
    world: &tr_core::World,
    projectiles: &mut crate::projectile::ProjectilePool,
    my_player: u8,
    cursor_world: glam::Vec2,
    rng: &mut tr_core::UnifiedRandom,
) {
    let Some(proj) = def.projectile else {
        return; // `canShoot` false arm (`:47147`) — nothing to shoot
    };
    if !def.summon || item.kind == 6161 {
        return; // the Clay Pot rides the generic tail (the i25 gate)
    }
    let Some(row) = crate::projectile::proj_data(proj) else {
        return;
    };
    if !row.minion {
        return; // sentries/whips are not `Minions` records
    }
    let fx = crate::prefix::prefixed_stats(item.kind, item.prefix);
    let unscaled = fx.map_or(def.damage, |s| s.damage); // `damage` (`:47060`)
    let kb = fx.map_or(def.knock_back, |s| s.knock_back);
    let fx_kb = player.weapon_knockback(def, kb); // `:47151`
    let (pcx, pcy) = player.center();
    let cursor = crate::world_sim::editing::limit_point_to_player_reachable_area(
        glam::Vec2::new(pcx, pcy),
        cursor_world,
        world,
    );
    // `NewProjectile` centres the box; the port's seam takes the top-left.
    let spawn_minion = |projectiles: &mut crate::projectile::ProjectilePool,
                        kind: u16,
                        ai0: f32,
                        ai1: f32|
     -> Option<usize> {
        let row = crate::projectile::proj_data(kind)?;
        projectiles.spawn_with_data_ai_owned(
            kind,
            my_player,
            cursor.x - row.w * 0.5,
            cursor.y - row.h * 0.5,
            0.0,
            0.0,
            unscaled,
            fx_kb,
            ai0,
            ai1,
        )
    };
    if item.kind == 3531 {
        // The Stardust Dragon's fresh-chain arm (`:48520-48536`): head
        // 625 + bodies 626/627 + tail 628 at the RAW cursor point (no reach
        // clamp — `:48524-48525` reads `Main.MouseWorld` directly), zero
        // velocity, the key chain through `localAI[1]`, `originalDamage`
        // on all four. The Apply kill guarantees no live chain. The spawn
        // damage is `Damage` — the weaponDamage parameter, 0 for the silent
        // shot (`:47053`); only `originalDamage` carries the item damage.
        let mut slots = [usize::MAX; 4];
        for (k, kind) in [proj, proj + 1, proj + 2, proj + 3].iter().enumerate() {
            let Some(row) = crate::projectile::proj_data(*kind) else {
                return;
            };
            let Some(slot) = projectiles.spawn_with_data_ai_owned(
                *kind,
                my_player,
                cursor_world.x - row.w * 0.5,
                cursor_world.y - row.h * 0.5,
                0.0,
                0.0,
                0,
                fx_kb,
                0.0,
                0.0,
            ) else {
                return;
            };
            slots[k] = slot;
            // `ai0` carries the PARENT's key (`:48527-48529` — the head's
            // ai0 stays 0).
            if k > 0 {
                let key = projectiles.key_of(slots[k - 1]);
                projectiles.set_ai0(slot, f32::from_bits(key as u32));
            }
        }
        // `:48530-48531` — the backlinks are raw SLOT INDICES.
        projectiles.set_local_ai1(slots[1], slots[2] as f32);
        projectiles.set_local_ai1(slots[2], slots[3] as f32);
        for slot in slots {
            projectiles.set_original_damage(slot, unscaled);
        }
    } else if item.kind == 1157 {
        // `:48436-48440` — the Pygmy Staff: one `Main.rand.Next(191, 195)`
        // kind pick, ai1 = 60 (`:49584-49585`), then `localAI[0] = 30`.
        let kind = rng.next_range(191, 195) as u16;
        if let Some(slot) = spawn_minion(projectiles, kind, 0.0, 60.0) {
            projectiles.set_local_ai0(slot, 30.0);
        }
    } else if item.kind == 2535 {
        // `:48468-48474` — the Optic Staff spawns BOTH twins (387 + 388),
        // zero velocity (the `RotatedBy` of a zero vector), ai1 = 1.
        for k in [proj, proj + 1] {
            let _ = spawn_minion(projectiles, k, 0.0, 1.0);
        }
    } else if item.kind == 2551 {
        // `:48476-48481` — the Spider Staff cycles 390/391/392 via
        // `nextCycledSpiderMinionType` (never reset), then increments mod 3.
        let kind = (proj as i32 + player.next_cycled_spider_minion_type) as u16;
        let _ = spawn_minion(projectiles, kind, 0.0, 0.0);
        player.next_cycled_spider_minion_type += 1;
        player.next_cycled_spider_minion_type %= 3;
    } else if item.kind == 2584 {
        // `:48483-48486` — the Pirate Staff: `Main.rand.Next(3)` onto
        // 393/394/395.
        let kind = proj + rng.next_range(0, 3) as u16;
        let _ = spawn_minion(projectiles, kind, 0.0, 0.0);
    } else if item.kind == 5664 || item.kind == 6149 {
        // `:48452-48456` — the Foxsparks staves: spawn, `localAI[0] = 30`.
        if let Some(slot) = spawn_minion(projectiles, proj, 0.0, 0.0) {
            projectiles.set_local_ai0(slot, 30.0);
        }
    } else {
        // The plain `SpawnMinionOnCursor` staves (`:49587-49596`): ai1 = 1
        // for 2364/2365/2535/2621/2749/3474, ai0 = 45 for 6164.
        let (ai0, ai1) = match item.kind {
            2364 | 2365 | 2535 | 2621 | 2749 | 3474 => (0.0, 1.0),
            6164 => (45.0, 0.0),
            _ => (0.0, 0.0),
        };
        let _ = spawn_minion(projectiles, proj, ai0, ai1);
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

    /// A staged player + world for the respawn witnesses — the
    /// `use_loop_tests::app_with_staff` shape without the App. The cursor
    /// rides through the reach clamp like the use path (the witnesses
    /// assert kinds/damage/ai, not the clamped position).
    fn respawn_fixture(kind: u16, prefix: u16) -> (Player, [ItemSlot; crate::item::INV], tr_core::World) {
        let mut w = tr_core::World::new("t".into(), 64, 800);
        for x in 0..64 {
            let i = w.index(x, 300);
            w.tiles[i] = tr_core::tile::Tile { kind: 1, ..Default::default() };
            w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
        }
        let mut player = Player::spawn(&w);
        player.x = 33.0 * 16.0;
        player.y = 297.0 * 16.0;
        player.max_minions = 5;
        let mut inventory = [crate::item::EMPTY_SLOT; crate::item::INV];
        inventory[0] = ItemSlot { kind, name: crate::registry::name(kind), count: 1, prefix };
        (player, inventory, w)
    }

    /// `TryRespawn` (`MinionSpawnFromInventoryItem.cs:30-47`): the matching
    /// inventory item's buff grants (`buffTime == 0` → 3600) and the
    /// `SpawnMinionOnCursor` spawn lands at the cursor with the UNSCALED
    /// prefixed damage + the minionKB knockback fold; no matching item
    /// answers silently.
    #[test]
    fn the_respawn_grants_the_buff_and_spawns_at_the_cursor() {
        // The Slime Staff 1309 — shoot 266, buff 64, buffTime 0.
        let (mut player, inventory, w) = respawn_fixture(1309, 0);
        let mut projectiles = crate::projectile::ProjectilePool::new();
        let mut rng = tr_core::UnifiedRandom::new(7);
        let (pcx, pcy) = player.center();
        let cursor = glam::Vec2::new(pcx + 32.0, pcy);
        respawn_minions(
            &[MinionRecord { item_type: 1309, item_prefix: 0 }],
            &mut player,
            &inventory,
            &w,
            &mut projectiles,
            0,
            cursor,
            &mut rng,
        );
        let slot = projectiles
            .slots()
            .iter()
            .position(|s| s.active && s.kind == 266)
            .expect("the Slime minion respawned");
        let p = &projectiles.slots()[slot];
        let def = crate::registry::item(1309).unwrap();
        assert_eq!(p.damage, def.damage, "the UNSCALED item damage");
        assert_eq!((p.vel.x, p.vel.y), (0.0, 0.0), "zero spawn velocity");
        // The buff grant: buffTime 0 → 3600 (`:40-43`).
        let b = player.buffs.find_buff_index(64).expect("buff 64 granted");
        assert!(player.buffs.buff_time[b] >= 3600, "the 3600 default grant");
        // A record with no matching item answers silently (`:33-36`).
        let before = projectiles.slots().iter().filter(|s| s.active).count();
        respawn_minions(
            &[MinionRecord { item_type: 2364, item_prefix: 0 }],
            &mut player,
            &inventory,
            &w,
            &mut projectiles,
            0,
            cursor,
            &mut rng,
        );
        assert_eq!(
            projectiles.slots().iter().filter(|s| s.active).count(),
            before,
            "no Hornet Staff in the inventory — no spawn"
        );
        // A prefix mismatch also fails `ItemMatches` (`:21-28`).
        respawn_minions(
            &[MinionRecord { item_type: 1309, item_prefix: 81 }],
            &mut player,
            &inventory,
            &w,
            &mut projectiles,
            0,
            cursor,
            &mut rng,
        );
        assert_eq!(
            projectiles.slots().iter().filter(|s| s.active).count(),
            before,
            "the prefix must match too"
        );
    }

    /// The Optic Staff's record respawns BOTH twins (`:48468-48474`), the
    /// Spider Staff's cycle advances mod 3 (`:48476-48481`), and the
    /// Pygmy's kind pick draws the caller's stream (`:48436-48440`).
    #[test]
    fn the_staff_arms_respawn_their_kinds() {
        // Optic 2535 — shoot 387, both twins.
        let (mut player, inventory, w) = respawn_fixture(2535, 0);
        let mut projectiles = crate::projectile::ProjectilePool::new();
        let mut rng = tr_core::UnifiedRandom::new(7);
        let (pcx, pcy) = player.center();
        let cursor = glam::Vec2::new(pcx + 32.0, pcy);
        respawn_minions(
            &[MinionRecord { item_type: 2535, item_prefix: 0 }],
            &mut player,
            &inventory,
            &w,
            &mut projectiles,
            0,
            cursor,
            &mut rng,
        );
        let kinds: Vec<u16> = projectiles.slots().iter().filter(|s| s.active).map(|s| s.kind).collect();
        assert_eq!(kinds, vec![387, 388], "both twins, ai1 = 1");
        assert!(projectiles.slots().iter().filter(|s| s.active).all(|s| s.ai[1] == 1.0));
        // Spider 2551 — the cycle counter advances on every respawn.
        let (mut player, inventory, w) = respawn_fixture(2551, 0);
        let mut projectiles = crate::projectile::ProjectilePool::new();
        respawn_minions(
            &[
                MinionRecord { item_type: 2551, item_prefix: 0 },
                MinionRecord { item_type: 2551, item_prefix: 0 },
                MinionRecord { item_type: 2551, item_prefix: 0 },
                MinionRecord { item_type: 2551, item_prefix: 0 },
            ],
            &mut player,
            &inventory,
            &w,
            &mut projectiles,
            0,
            cursor,
            &mut rng,
        );
        let kinds: Vec<u16> = projectiles.slots().iter().filter(|s| s.active).map(|s| s.kind).collect();
        assert_eq!(kinds, vec![390, 391, 392, 390], "the mod-3 cycle");
        assert_eq!(player.next_cycled_spider_minion_type, 1);
        // Pygmy 1157 — the kind pick is a stream draw in [191, 195).
        let (mut player, inventory, w) = respawn_fixture(1157, 0);
        let mut projectiles = crate::projectile::ProjectilePool::new();
        let mut probe = tr_core::UnifiedRandom::new(7);
        let want = probe.next_range(191, 195) as u16;
        respawn_minions(
            &[MinionRecord { item_type: 1157, item_prefix: 0 }],
            &mut player,
            &inventory,
            &w,
            &mut projectiles,
            0,
            cursor,
            &mut rng,
        );
        let p = projectiles.slots().iter().find(|s| s.active).expect("a pygmy");
        assert_eq!(p.kind, want, "the stream's kind pick");
        assert_eq!(p.ai[1], 60.0, "the Pygmy ai1 seed");
        assert_eq!(p.local_ai[0], 30.0, "the localAI[0] = 30 payload");
    }
}
