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
//! `EquipmentLoadout` (`decompiled/Terraria/EquipmentLoadout.cs`) — the three
//! swappable armor/dye sets behind the loadout buttons (`Main.DrawLoadoutButtons`,
//! `Main.cs:41935-41991`).
//!
//! The port keeps the live equipment arrays on `App.equip` (`inv::EquipSlots`);
//! a loadout stores the *other two* sets plus their share flags. `Swap`
//! exchanges the live arrays with the stored ones exactly as the C# does
//! (`EquipmentLoadout.cs:63-80`), so `TrySwitchingLoadout`'s two-swap shape
//! (`Player.cs:5665-5666`) moves the current set into its slot and the target
//! set out. The per-item `favorited` flag doubles as the loadout-share flag in
//! the equip contexts (`ItemSlot.canLoadoutShareAt`, `ItemSlot.cs:395-399`);
//! the port's grid favorites ride `App.favorite_slots`, so the equip-side
//! flags get their own 30-bit word per loadout here.
use std::path::{Path, PathBuf};

use crate::inv;
use crate::item::ItemSlot;
use crate::itemdata;
use crate::lang_tables;
use crate::player::Player;
use crate::inv::{ARMOR, DYE};
use crate::item::EMPTY_SLOT;

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
/// NO minion-slot-cap check runs here — `SpawnMinionOnCursor` has none,
/// and `FreeUpPetsAndMinions` rides the USE START (`Player.cs:42780`),
/// which `SilentlyShootItem` bypasses; an over-cap respawn dies on the
/// first pool tick (`Projectile.cs:16712`, the port's
/// `summon_tick_block`), exactly as the C#.
/// The Stardust Dragon chain (`:48498-48554`): the Apply kill sweep only
/// kills `minionSlots > 0` (`ScenarioTestLoadout.cs:129`) and the dragon
/// HEAD (625) / TAIL (628) carry ZERO slots (`Projectile.cs:6731-6738`),
/// so a live head+tail SURVIVES the apply — the grow-existing-chain arm
/// (`:48537-48553`) is reachable and ported below; the fresh-chain arm
/// (`:48520-48536`) runs only when neither lives, and a half-chain
/// (head xor tail) spawns NOTHING (the C# has no third arm).
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
        // The chain scan (`:48500-48519`): the FIRST owned 625 head and the
        // FIRST owned 628 tail in pool order. The Apply kill sweep only
        // kills `minionSlots > 0` (`ScenarioTestLoadout.cs:129`); the head
        // and tail carry ZERO slots (`Projectile.cs:6731-6738` — only the
        // 626/627 bodies get 0.5), so a live chain's head+tail SURVIVE the
        // apply and the grow arm below is the reachable one.
        let mut head = None;
        let mut tail = None;
        for (i, s) in projectiles.slots().iter().enumerate() {
            if s.active && s.owner == my_player {
                if head.is_none() && s.kind == 625 {
                    head = Some(i);
                }
                if tail.is_none() && s.kind == 628 {
                    tail = Some(i);
                }
            }
        }
        if let (Some(_), Some(tail_slot)) = (head, tail) {
            // The grow-existing-chain arm (`:48537-48553`): TWO new bodies
            // (626 + 627) at the PLAYER-CENTRE point (`pointPosition` —
            // `RotatedRelativePoint(MountedCenter)`, `:47196`; no mount /
            // rotation in the port, the use path's precedent) with the AIM
            // velocity (`num4/num5` — the aim normalized to `shootSpeed`,
            // `:47277-47307`; the zero aim falls back to the facing), NOT
            // the cursor/zero of the fresh arm. The first body's ai0 is the
            // TAIL's current ai0 (its previous link's key), the second's is
            // the first's key; `localAI[1]` links body1→body2→tail; the
            // tail's ai0 re-keys to body2; all three carry `ai[1] = 1` +
            // `netUpdate`; `originalDamage` on all three. Spawn damage is
            // `Damage` = 0 (the silent shot's weaponDamage).
            let aim = cursor_world - glam::Vec2::new(pcx, pcy);
            let speed = fx.map_or(def.shoot_speed, |s| s.shoot_speed);
            // `:47213-47224` — the aim-dot `ChangeDir` (flag true for 3531,
            // `:47197-47201`): Dot(UnitX, aim) > 0 → 1 else −1; the zero aim
            // takes the −1 arm too. The port models the facing write only
            // (the use path's boomerang precedent — the pulley/itemRotation
            // halves of `ChangeDir` stay unported); the grow arm's zero-aim
            // fallback below is the one reader.
            player.facing = if aim.x > 0.0 { 1 } else { -1 };
            let (vx, vy) = if aim == glam::Vec2::ZERO {
                // `:47291-47296` — num4 = direction, num5 = 0, then ×speed.
                (player.facing as f32 * speed, 0.0)
            } else {
                let n = aim.normalize_or_zero() * speed;
                (n.x, n.y)
            };
            let tail_ai0 = projectiles.slots()[tail_slot].ai[0];
            let body_row = crate::projectile::proj_data(proj + 1);
            let Some(body_row) = body_row else {
                return;
            };
            let Some(body1) = projectiles.spawn_with_data_ai_owned(
                proj + 1,
                my_player,
                pcx - body_row.w * 0.5,
                pcy - body_row.h * 0.5,
                vx,
                vy,
                0,
                fx_kb,
                tail_ai0,
                0.0,
            ) else {
                return;
            };
            let body2_row = crate::projectile::proj_data(proj + 2);
            let Some(body2_row) = body2_row else {
                return;
            };
            let Some(body2) = projectiles.spawn_with_data_ai_owned(
                proj + 2,
                my_player,
                pcx - body2_row.w * 0.5,
                pcy - body2_row.h * 0.5,
                vx,
                vy,
                0,
                fx_kb,
                f32::from_bits(projectiles.key_of(body1) as u32),
                0.0,
            ) else {
                return;
            };
            // `:48541-48549` — the links + the ai[1] = 1 / netUpdate trio.
            projectiles.set_local_ai1(body1, body2 as f32);
            projectiles.set_ai1(body1, 1.0);
            projectiles.set_local_ai1(body2, tail_slot as f32);
            projectiles.set_ai1(body2, 1.0);
            projectiles.set_ai0(tail_slot, f32::from_bits(projectiles.key_of(body2) as u32));
            projectiles.set_ai1(tail_slot, 1.0);
            for slot in [body1, body2, tail_slot] {
                projectiles.slots_mut()[slot].net_update = true;
                // `:48550-48552` — `originalDamage = damage` (UNSCALED).
                projectiles.set_original_damage(slot, unscaled);
            }
            return;
        }
        if head.is_some() || tail.is_some() {
            // A HALF chain (head xor tail): the C# has no third arm —
            // neither `:48520` nor `:48537` matches, nothing spawns.
            return;
        }
        // The Stardust Dragon's fresh-chain arm (`:48520-48536`): head
        // 625 + bodies 626/627 + tail 628 at the RAW cursor point (no reach
        // clamp — `:48524-48525` reads `Main.MouseWorld` directly), zero
        // velocity, the key chain through `localAI[1]`, `originalDamage`
        // on all four. The spawn damage is `Damage` — the weaponDamage
        // parameter, 0 for the silent shot (`:47053`); only
        // `originalDamage` carries the item damage.
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

// --- W4-g55: the EquipmentLoadout model (the three swappable armor/dye sets) ---

/// `EquipmentLoadout.Hide` (`EquipmentLoadout.cs:12`, `new bool[10]` `:18`):
/// the per-accessory visual-hide flags stored with the set.
pub const HIDE: usize = 10;

/// One stored equipment set — `EquipmentLoadout`'s three arrays.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EquipmentLoadout {
    /// `EquipmentLoadout.Armor` — `CreateItemArray(20)` (`:16`).
    pub armor: [ItemSlot; ARMOR],
    /// `EquipmentLoadout.Dye` — `CreateItemArray(10)` (`:17`).
    pub dye: [ItemSlot; DYE],
    /// `EquipmentLoadout.Hide` — `new bool[10]` (`:18`).
    pub hide: [bool; HIDE],
    /// The per-slot `Item.favorited` bits over `armor[0..20]` then
    /// `dye[0..10]` (bit `i` = armor slot `i`, bit `20 + j` = dye slot `j`).
    /// In the equip contexts the flag is the loadout-share toggle
    /// (`ItemSlot.ToggleLoadoutShare`, `ItemSlot.cs:879-889`), read by
    /// `GetEffectiveArmor`/`GetEffectiveDye` (`Player.cs:5690-5764`).
    pub favorited: u32,
}

impl Default for EquipmentLoadout {
    /// The ctor (`EquipmentLoadout.cs:14-19`): 20 + 10 fresh `Item()`s (all
    /// air, `favorited` false) and `Hide[10]` false.
    fn default() -> Self {
        Self {
            armor: [EMPTY_SLOT; ARMOR],
            dye: [EMPTY_SLOT; DYE],
            hide: [false; HIDE],
            favorited: 0,
        }
    }
}

impl EquipmentLoadout {
    /// The share flag of one stored slot: `slot < 20` indexes `Armor`, else
    /// `Dye[slot - 20]` (the `PlayerItemSlotID.Loadout*_Armor_0/Dye_0` split).
    pub fn is_shared(&self, slot: usize) -> bool {
        slot < 30 && self.favorited & (1 << slot) != 0
    }

    /// `Item.favorited = value` on one stored slot (the toggle's write half).
    pub fn set_shared(&mut self, slot: usize, value: bool) {
        if slot >= 30 {
            return;
        }
        if value {
            self.favorited |= 1 << slot;
        } else {
            self.favorited &= !(1 << slot);
        }
    }

    /// The stored item at `slot` (`< 20` armor, else dye) — the
    /// `GetEffectiveArmor`/`GetEffectiveDye` scan reads this.
    pub fn item_at(&self, slot: usize) -> ItemSlot {
        if slot < ARMOR {
            self.armor[slot]
        } else if slot < ARMOR + DYE {
            self.dye[slot - ARMOR]
        } else {
            EMPTY_SLOT
        }
    }

    /// `EquipmentLoadout.Swap` (`EquipmentLoadout.cs:63-80`): `Utils.Swap`
    /// every armor slot, every dye slot and every `hideVisibleAccessory` flag
    /// with the player's live arrays. The share flags swap with their items
    /// (the C# `favorited` rides the `Item` instance through the swap).
    pub fn swap(
        &mut self,
        armor: &mut [ItemSlot; ARMOR],
        dye: &mut [ItemSlot; DYE],
        hide: &mut [bool; HIDE],
        live_favorited: &mut u32,
    ) {
        for i in 0..ARMOR {
            std::mem::swap(&mut armor[i], &mut self.armor[i]);
        }
        for j in 0..DYE {
            std::mem::swap(&mut dye[j], &mut self.dye[j]);
        }
        for k in 0..HIDE {
            std::mem::swap(&mut hide[k], &mut self.hide[k]);
        }
        std::mem::swap(live_favorited, &mut self.favorited);
    }
}

/// One item's net-state snapshot — `Player.clientCloneItem`
/// (`Player.cs:54514-54520`): `type`, `stack`, `prefix`, `favorited` (the four
/// fields `Item.IsNetStateDifferent` compares). The port's `ItemSlot` carries
/// kind/count/prefix; the share flag rides the loadout's 30-bit word, so the
/// clone takes it as a parameter (the c28 `item_net_state` convention).
pub type ClonedItem = (u16, u32, u16, bool);

/// `Player.clientCloneItemArray` (`Player.cs:54506-54512`): clone every slot's
/// net state, reading the per-slot `favorited` bit out of `share_word`.
fn clone_item_array(items: &[ItemSlot], share_word: u32, bit_base: usize) -> Vec<ClonedItem> {
    items
        .iter()
        .enumerate()
        .map(|(i, s)| {
            (
                s.kind,
                s.count,
                s.prefix,
                share_word & (1 << (bit_base + i)) != 0,
            )
        })
        .collect()
}

/// The `clientCloneLoadouts` snapshot (`Player.cs:54522-54531`) — the item-array
/// half of `Player.clientClone` (`:54461-54504`, the c28 `ClientClone` carries
/// the scalar half). It clones the LIVE armor + dye arrays and each of the
/// three stored loadouts' Armor + Dye (`clientCloneItemArray`, `:54524-54530`).
/// `Hide`/`CurrentLoadoutIndex` are NOT cloned by the C# (the stored sets'
/// `favorited` rides each item). The `TrySwitchingLoadout` tail calls it on
/// `Main.clientPlayer` (`:5670`) so the next `TrySyncingMyPlayer` array diff
/// (`Main.cs:17557-17575`) sees the swapped sets as already-synced.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct LoadoutClone {
    /// `clientCloneItemArray(this.armor, clonePlayer.armor)` (`:54524`).
    pub armor: Vec<ClonedItem>,
    /// `clientCloneItemArray(this.dye, clonePlayer.dye)` (`:54525`).
    pub dye: Vec<ClonedItem>,
    /// `Loadouts[i].Armor` then `Loadouts[i].Dye` per set (`:54526-54530`),
    /// indexed `[loadout]` -> `(armor, dye)`.
    pub loadouts: [(Vec<ClonedItem>, Vec<ClonedItem>); 3],
}

impl LoadoutClone {
    /// `Player.clientCloneLoadouts` (`Player.cs:54522-54531`): clone the live
    /// armor/dye (the live share word supplies their `favorited` bits) and the
    /// three stored sets (each set's own share word). `live_favorited` is
    /// `App.equip_share_flags`; bit `i` = armor[i], bit `20 + j` = dye[j].
    pub fn clone_from(
        armor: &[ItemSlot; ARMOR],
        dye: &[ItemSlot; DYE],
        live_favorited: u32,
        loadouts: &[EquipmentLoadout; 3],
    ) -> Self {
        Self {
            armor: clone_item_array(armor, live_favorited, 0),
            dye: clone_item_array(dye, live_favorited, ARMOR),
            loadouts: std::array::from_fn(|i| {
                (
                    clone_item_array(&loadouts[i].armor, loadouts[i].favorited, 0),
                    clone_item_array(&loadouts[i].dye, loadouts[i].favorited, ARMOR),
                )
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn slot(kind: u16) -> ItemSlot {
        ItemSlot {
            kind,
            name: crate::registry::name(kind),
            count: 1,
            prefix: 0,
        }
    }

    /// W4-g55 — the ctor is 20 + 10 air items, `Hide[10]` false, no shares
    /// (`EquipmentLoadout.cs:14-19`; the `.plr` fresh-default block is 310
    /// zero bytes per loadout, `player_file.rs`'s golden layout).
    #[test]
    fn w4_g55_the_fresh_loadout_is_all_air() {
        let l = EquipmentLoadout::default();
        assert!(l.armor.iter().all(|s| s.count == 0));
        assert!(l.dye.iter().all(|s| s.count == 0));
        assert_eq!(l.hide, [false; HIDE]);
        assert_eq!(l.favorited, 0);
    }

    /// W4-g55 — `Swap` exchanges all three arrays plus the share word, and a
    /// second swap restores the original state (`Utils.Swap` is symmetric).
    #[test]
    fn w4_g55_swap_exchanges_every_array_and_is_symmetric() {
        let mut live_armor = [EMPTY_SLOT; ARMOR];
        let mut live_dye = [EMPTY_SLOT; DYE];
        let mut live_hide = [false; HIDE];
        let mut live_fav = 0u32;
        live_armor[0] = slot(90); // Iron Helmet (`ItemID.cs:1863`)
        live_dye[0] = slot(1007); // Red Dye (`ItemID.cs:3697`)
        live_hide[3] = true;
        live_fav |= 1 << 0;

        let mut stored = EquipmentLoadout::default();
        stored.armor[1] = slot(954); // Ancient Iron Helmet (`ItemID.cs:3591`)
        stored.set_shared(21, true); // dye slot 1 shared

        let before = (live_armor, live_dye, live_hide, live_fav);
        stored.swap(&mut live_armor, &mut live_dye, &mut live_hide, &mut live_fav);
        assert_eq!(live_armor[0], EMPTY_SLOT, "the live head moved into storage");
        assert_eq!(stored.armor[0].kind, 90);
        assert_eq!(live_armor[1].kind, 954, "the stored head came out");
        assert_eq!(live_dye[0], EMPTY_SLOT);
        assert_eq!(stored.dye[0].kind, 1007);
        assert!(!live_hide[3], "the hide flags swapped");
        assert!(stored.hide[3]);
        assert_eq!(live_fav, 1 << 21, "the share word swapped");
        assert_eq!(stored.favorited, 1 << 0);

        stored.swap(&mut live_armor, &mut live_dye, &mut live_hide, &mut live_fav);
        assert_eq!((live_armor, live_dye, live_hide, live_fav), before);
    }

    /// W4-g55 — `clientCloneLoadouts` (`Player.cs:54522-54531`) clones the
    /// live armor/dye + the three stored sets' Armor/Dye, per-item
    /// type/stack/prefix/favorited (`clientCloneItem`, `:54514-54520`). The
    /// live share word supplies the live arrays' favorited bits; each stored
    /// set's own word supplies its own.
    #[test]
    fn w4_g55_the_loadout_clone_snapshots_every_array() {
        let mut armor = [EMPTY_SLOT; ARMOR];
        let mut dye = [EMPTY_SLOT; DYE];
        armor[0] = slot(90); // Iron Helmet, live + shared
        dye[1] = slot(1007); // Red Dye, live + unshared
        let live_fav = 1u32 << 0; // armor[0] shared, dye[1] not

        let mut loadouts = [EquipmentLoadout::default(); 3];
        loadouts[1].armor[2] = slot(954); // Ancient Iron Helmet, stored
        loadouts[1].set_shared(2, true); // armor[2] shared
        loadouts[2].dye[0] = slot(1007); // stored dye, unshared

        let c = LoadoutClone::clone_from(&armor, &dye, live_fav, &loadouts);
        // The live armor[0]: (90, 1, 0, true).
        assert_eq!(c.armor[0], (90, 1, 0, true));
        assert_eq!(c.armor[1], (0, 0, 0, false), "an air slot clones air");
        // The live dye[1]: (1007, 1, 0, false) — bit 20+1 not set.
        assert_eq!(c.dye[1], (1007, 1, 0, false));
        // Loadout 1's armor[2]: (954, 1, 0, true).
        assert_eq!(c.loadouts[1].0[2], (954, 1, 0, true));
        // Loadout 2's dye[0]: (1007, 1, 0, false).
        assert_eq!(c.loadouts[2].1[0], (1007, 1, 0, false));
        // Loadout 0 is all air.
        assert!(c.loadouts[0].0.iter().all(|&(k, _, _, _)| k == 0));
        // The array lengths match the C# (20 armor + 10 dye per set).
        assert_eq!(c.armor.len(), ARMOR);
        assert_eq!(c.dye.len(), DYE);
        assert_eq!(c.loadouts[1].0.len(), ARMOR);
        assert_eq!(c.loadouts[1].1.len(), DYE);
    }

    // --- W4-d85: the ScenarioTestLoadout debug-list witnesses ---


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

    /// The Stardust Dragon's GROW arm (`Player.cs:48537-48553`) — the Apply
    /// kill sweep only kills `minionSlots > 0` (`ScenarioTestLoadout.cs:129`)
    /// and the head 625 / tail 628 carry ZERO slots (`Projectile.cs:
    /// 6731-6738`), so a live head+tail survives and the respawn GROWS the
    /// chain: two bodies at the player centre with the aim velocity, the
    /// key relink, `ai[1] = 1` + netUpdate + originalDamage on all three.
    #[test]
    fn the_dragon_grow_arm_relinks_the_surviving_chain() {
        let (mut player, inventory, w) = respawn_fixture(3531, 0);
        let mut projectiles = crate::projectile::ProjectilePool::new();
        let mut rng = tr_core::UnifiedRandom::new(7);
        let (pcx, pcy) = player.center();
        // A surviving head + tail (the Apply kill leaves them — 0 slots).
        let head = projectiles
            .spawn_with_data_ai_owned(625, 0, 100.0, 100.0, 0.0, 0.0, 0, 2.0, 0.0, 0.0)
            .expect("the head row");
        let tail = projectiles
            .spawn_with_data_ai_owned(628, 0, 124.0, 100.0, 0.0, 0.0, 0, 2.0, 0.0, 0.0)
            .expect("the tail row");
        let tail_key_before = projectiles.key_of(head);
        projectiles.set_ai0(tail, f32::from_bits(tail_key_before as u32));
        let cursor = glam::Vec2::new(pcx + 32.0, pcy);
        respawn_minions(
            &[MinionRecord { item_type: 3531, item_prefix: 0 }],
            &mut player,
            &inventory,
            &w,
            &mut projectiles,
            0,
            cursor,
            &mut rng,
        );
        let kinds: Vec<u16> = projectiles.slots().iter().filter(|s| s.active).map(|s| s.kind).collect();
        assert_eq!(kinds, vec![625, 628, 626, 627], "the grow arm adds ONLY two bodies");
        let (b1, b2) = (2, 3);
        // The player-centre spawn (24x24 rows, `:6739-6740`).
        for slot in [b1, b2] {
            let p = &projectiles.slots()[slot];
            assert_eq!(p.pos.x, pcx - 12.0, "the centre X minus half the box");
            assert_eq!(p.pos.y, pcy - 12.0, "the centre Y minus half the box");
            // The aim velocity at shootSpeed 10 (`:47277-47307`), NOT zero.
            assert_eq!((p.vel.x, p.vel.y), (10.0, 0.0), "the aim normalized to shootSpeed");
            assert_eq!(p.damage, 0, "Damage = weaponDamage = 0 (the silent shot)");
            assert_eq!(p.ai[1], 1.0, "the grow arm's ai[1] = 1");
            assert!(p.net_update, "the grow arm's netUpdate");
            assert_eq!(p.original_damage, 40, "the UNSCALED item damage (`:48550-48551`)");
        }
        // The key chain: body1.ai0 = the tail's OLD ai0 (the head's key),
        // body2.ai0 = body1's key, tail.ai0 = body2's key (`:48539-48547`).
        assert_eq!(projectiles.slots()[b1].ai[0], f32::from_bits(tail_key_before as u32));
        assert_eq!(
            projectiles.slots()[b2].ai[0],
            f32::from_bits(projectiles.key_of(b1) as u32)
        );
        assert_eq!(
            projectiles.slots()[tail].ai[0],
            f32::from_bits(projectiles.key_of(b2) as u32)
        );
        // The localAI[1] backlinks: body1→body2, body2→tail (`:48541/:48544`).
        assert_eq!(projectiles.slots()[b1].local_ai[1], b2 as f32);
        assert_eq!(projectiles.slots()[b2].local_ai[1], tail as f32);
        assert_eq!(projectiles.slots()[tail].ai[1], 1.0, "the tail's ai[1] = 1");
        assert!(projectiles.slots()[tail].net_update, "the tail's netUpdate");
        assert_eq!(projectiles.slots()[tail].original_damage, 40, "`:48552`");
        // The head is untouched by the grow arm (no writes in the C#).
        assert_eq!(projectiles.slots()[head].ai[1], 0.0);
        assert!(!projectiles.slots()[head].net_update);
    }

    /// The fresh-chain arm runs only when NEITHER head nor tail lives; a
    /// HALF chain (head xor tail) spawns nothing — the C# has no third arm
    /// (`Player.cs:48520`/`:48537` are the only two).
    #[test]
    fn the_dragon_half_chain_spawns_nothing_and_the_fresh_arm_needs_both_dead() {
        // Half chain: a lone head.
        let (mut player, inventory, w) = respawn_fixture(3531, 0);
        let mut projectiles = crate::projectile::ProjectilePool::new();
        let mut rng = tr_core::UnifiedRandom::new(7);
        let (pcx, pcy) = player.center();
        projectiles
            .spawn_with_data_ai_owned(625, 0, 100.0, 100.0, 0.0, 0.0, 0, 2.0, 0.0, 0.0)
            .expect("the head row");
        let cursor = glam::Vec2::new(pcx + 32.0, pcy);
        respawn_minions(
            &[MinionRecord { item_type: 3531, item_prefix: 0 }],
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
            1,
            "the half chain spawns nothing"
        );
        // Fresh chain: no head, no tail — head+bodies+tail at the RAW cursor.
        let (mut player, inventory, w) = respawn_fixture(3531, 0);
        let mut projectiles = crate::projectile::ProjectilePool::new();
        respawn_minions(
            &[MinionRecord { item_type: 3531, item_prefix: 0 }],
            &mut player,
            &inventory,
            &w,
            &mut projectiles,
            0,
            cursor,
            &mut rng,
        );
        let kinds: Vec<u16> = projectiles.slots().iter().filter(|s| s.active).map(|s| s.kind).collect();
        assert_eq!(kinds, vec![625, 626, 627, 628], "the fresh four-part chain");
        for s in projectiles.slots().iter().filter(|s| s.active) {
            assert_eq!((s.vel.x, s.vel.y), (0.0, 0.0), "the fresh arm's zero velocity");
            assert_eq!(s.damage, 0, "the fresh arm spawns Damage = 0");
            assert_eq!(s.original_damage, 40, "originalDamage carries the item damage");
        }
    }
}
