//! The achievements framework — `Achievement` = a named condition set
//! (`Achievement.cs:11-120`), the manager registry (`AchievementManager`) and
//! the `AchievementsHelper` reporting surface
//! (`GameContent/Achievements/AchievementsHelper.cs`).
//!
//! The registry is the **full 137-row** `AchievementInitializer.Load`
//! (`Initializers/AchievementInitializer.cs:13-441`): every
//! `new Achievement("NAME")` block with its `AddCondition` calls, then the
//! `RegisterIconIndex` pass (`:445-582`, the icon frame each row draws from
//! `Images/UI/Achievements`), the `RegisterAchievementCategory` pass
//! (`:584-736`, the filter buttons) and the `UseTracker*` flags. The table
//! itself lives in [`crate::achievements_data`]; this module is the runtime.
//!
//! `ui::achievements` draws the menu (`UIAchievementsMenu` + its
//! `UIAchievementListItem` rows) on top of this registry.
//!
//! The vanilla save is Rijndael-CBC+BSON with the "RELOGIC-TERRARIA" key
//! (`AchievementManager.cs:40-79`); the port writes its own text format at
//! `achievements.trdat` (documented deviation: writing the vanilla format
//! would touch a real install's `achievements.dat`).

use std::path::{Path, PathBuf};

pub use crate::achievements_data::{Cond, Row, Tracker, ROWS};
use crate::achievements_data::ROWS as REGISTRY;

/// `AchievementCategory.cs` — `None = -1` is the secret row
/// (`GAIN_TORCH_GODS_FAVOR`, which `AchievementInitializer` never passes to
/// `RegisterAchievementCategory`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Category {
    None,
    Slayer,
    Collector,
    Explorer,
    Challenger,
}

impl Category {
    /// The four toggle buttons, in `UIToggleImage` order
    /// (`UIAchievementsMenu.cs`'s `_categoryButtons` loop, `j = 0..3`):
    /// Slayer, Collector, Explorer, Challenger.
    pub const FILTERS: [Category; 4] = [
        Category::Slayer,
        Category::Collector,
        Category::Explorer,
        Category::Challenger,
    ];

    /// The tooltip key of the category button
    /// (`UIAchievementsMenu.Draw`'s switch): `Achievements.SlayerCategory` …
    pub fn tooltip_key(self) -> &'static str {
        match self {
            Category::None => "Achievements.NoCategory",
            Category::Slayer => "Achievements.SlayerCategory",
            Category::Collector => "Achievements.CollectorCategory",
            Category::Explorer => "Achievements.ExplorerCategory",
            Category::Challenger => "Achievements.ChallengerCategory",
        }
    }

    /// `(int)Category` — the icon frame column of
    /// `Achievement_Categories.Frame(4, 2, …)` and the filter-button index.
    pub fn index(self) -> i32 {
        match self {
            Category::None => -1,
            Category::Slayer => 0,
            Category::Collector => 1,
            Category::Explorer => 2,
            Category::Challenger => 3,
        }
    }

    /// `(AchievementCategory)index` (the `FilterList` cast, clamped to the
    /// enum's range so a stray id reads as `None`).
    pub fn from_index(index: usize) -> Category {
        match index {
            0 => Category::Slayer,
            1 => Category::Collector,
            2 => Category::Explorer,
            3 => Category::Challenger,
            _ => Category::None,
        }
    }
}

/// What one condition listens for — the `AchievementsHelper.Notify*` family
/// narrowed to the ported systems.
///
/// A single vanilla condition can list **many** ids and completes on any of
/// them (`NPCKilledCondition.Create(13, 14, 15)` is one condition named
/// `NPC_KILLED_13`, `NPCKilledCondition.cs:20-30`), so the single-id variants
/// are the event shape and the `…Any` variants carry the initializer's
/// multi-id rows.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CondKind {
    /// `NPCKilledCondition` — the NPC net id.
    NpcKilled(i32),
    /// The multi-id `NPCKilledCondition(params short[])`.
    NpcKilledAny(&'static [i32]),
    /// `ItemPickupCondition` — the item id.
    ItemPickup(u16),
    /// The multi-id `ItemPickupCondition(params short[])`.
    ItemPickupAny(&'static [u16]),
    /// `TileDestroyedCondition` — the tile kind.
    TileDestroyed(u16),
    /// The multi-id `TileDestroyedCondition(params short[])`.
    TileDestroyedAny(&'static [u16]),
    /// `ItemCraftCondition` — the crafted id (the port's craft event shape;
    /// vanilla's condition class reuses the `ITEM_PICKUP_` name prefix,
    /// `ItemCraftCondition.cs:8`).
    ItemCraft(u16),
    /// The multi-id `ItemCraftCondition(params short[])` — e.g. the
    /// `ItemID.Sets.Workbenches` set of `BENCHED`.
    ItemCraftAny(&'static [u16]),
    /// `ProgressionEventCondition` — the `Main._progressionEvent` id.
    ProgressionEvent(u32),
    /// `CustomFlagCondition` — the named flag (`Peek`/`Use`/`Equip`/…).
    Custom(&'static str),
    /// `CustomIntCondition` — a counted condition with its max.
    CustomInt(&'static str, i32),
    /// `CustomFloatCondition` — the float tracker, stored as its whole value
    /// (every shipped row's max is integral: 10000f, 1106688f).
    CustomFloat(&'static str, i32),
}

/// One `AchievementCondition`: the listener kind plus the tracker state
/// (`Completed` + the int tracker's value, `AchievementCondition.cs:9-45`).
#[derive(Clone, Debug)]
pub struct Condition {
    /// The condition's dictionary key (`NPC_KILLED_4`, `ITEM_PICKUP_9`,
    /// `TILE_DESTROYED_12`, `PROGRESSION_EVENT_8`, `Pick`, `Use`, …) — the
    /// vanilla `AchievementCondition.Name`, which the save file keys on.
    pub name: String,
    pub kind: CondKind,
    /// The int tracker's progress (`ConditionIntTracker.Value`).
    pub value: i32,
    /// The int tracker's target (`CustomIntCondition`'s max); 1 = the plain
    /// complete-on-event conditions.
    pub max: i32,
    /// `IsCompleted`.
    pub completed: bool,
}

impl Condition {
    fn new(name: String, kind: CondKind) -> Self {
        let max = match kind {
            CondKind::CustomInt(_, max) | CondKind::CustomFloat(_, max) => max.max(1),
            _ => 1,
        };
        Self {
            name,
            kind,
            value: 0,
            max,
            completed: false,
        }
    }
}

/// `Achievement` (`Achievement.cs:11-120`): complete when every condition is
/// (`IsCompleted => _completedCount == _conditions.Count` — so the zero-condition
/// `STAR_DESTROYER` row is complete from the start, exactly as vanilla).
#[derive(Clone, Debug)]
pub struct Achievement {
    pub name: &'static str,
    /// `Achievement.Id` — the construction-order index the list sorts by
    /// (`UIAchievementListItem.CompareTo`).
    pub id: u16,
    pub category: Category,
    /// `RegisterIconIndex`'s value: the 64 px icon's index on the
    /// 8-per-row `Images/UI/Achievements` sheet (the locked copy rides
    /// +528 px, `UIAchievementListItem`'s `_iconFrameLocked`).
    pub icon: u16,
    pub conditions: Vec<Condition>,
    /// `Achievement._tracker` — `UseTrackerFromCondition` /
    /// `UseConditionsCompletedTracker`.
    pub tracker: Option<Tracker>,
    pub completed: bool,
}

impl Achievement {
    fn from_row(row: &Row, id: u16) -> Self {
        Self {
            name: row.name,
            id,
            category: row.category,
            icon: row.icon,
            conditions: row.conditions.iter().map(Condition::from).collect(),
            tracker: row.tracker,
            completed: false,
        }
    }

    fn is_completed(&self) -> bool {
        self.conditions.iter().all(|c| c.completed)
    }

    /// `UIAchievementListItem.GetTrackerValues` — `(value, max)` when the row
    /// carries a tracker, else the UI's `(0, 0)` (no progress bar).
    pub fn tracker_values(&self) -> Option<(i32, i32)> {
        match self.tracker {
            Some(Tracker::ConditionsCompleted) => {
                let done = self.conditions.iter().filter(|c| c.completed).count() as i32;
                Some((done, self.conditions.len() as i32))
            }
            Some(Tracker::FromCondition(name)) => self
                .conditions
                .iter()
                .find(|c| c.name == name)
                .map(|c| (c.value, c.max)),
            None => None,
        }
    }
}

impl Condition {
    /// The initializer's `AddCondition` shape into the runtime condition,
    /// keeping vanilla's `AchievementCondition.Name` keys.
    fn from(cond: &Cond) -> Self {
        let (kind, name) = match *cond {
            Cond::Npc(id) => (CondKind::NpcKilled(id), format!("NPC_KILLED_{id}")),
            Cond::Npcs(ids) => (
                CondKind::NpcKilledAny(ids),
                format!("NPC_KILLED_{}", ids[0]),
            ),
            Cond::Item(id) => (CondKind::ItemPickup(id), format!("ITEM_PICKUP_{id}")),
            Cond::Items(ids) => (
                CondKind::ItemPickupAny(ids),
                format!("ITEM_PICKUP_{}", ids[0]),
            ),
            Cond::Tile(id) => (
                CondKind::TileDestroyed(id),
                format!("TILE_DESTROYED_{id}"),
            ),
            Cond::Tiles(ids) => (
                CondKind::TileDestroyedAny(ids),
                format!("TILE_DESTROYED_{}", ids[0]),
            ),
            // `ItemCraftCondition` names its conditions with the
            // `ITEM_PICKUP_` prefix (`ItemCraftCondition.cs:8`).
            Cond::Craft(id) => (
                CondKind::ItemCraft(id),
                format!("ITEM_PICKUP_{id}"),
            ),
            Cond::Crafts(ids) => (
                CondKind::ItemCraftAny(ids),
                format!("ITEM_PICKUP_{}", ids[0]),
            ),
            Cond::Progression(id) => (
                CondKind::ProgressionEvent(id),
                format!("PROGRESSION_EVENT_{id}"),
            ),
            Cond::Flag(flag) => (CondKind::Custom(flag), flag.to_string()),
            Cond::Int(name, max) => (CondKind::CustomInt(name, max), name.to_string()),
            Cond::Float(name, max) => (CondKind::CustomFloat(name, max), name.to_string()),
        };
        Condition::new(name, kind)
    }
}

/// `AchievementManager`: the registry + the notification surface.
#[derive(Clone, Debug, Default)]
pub struct AchievementManager {
    pub list: Vec<Achievement>,
    /// `AchievementManager._savePath` (`AchievementManager.cs:25`, bound in the
    /// constructor `:41-55`). The port binds it at bootstrap
    /// (`App::bind_achievements_save_path`) so a headless test App never
    /// touches the real `achievements.trdat`.
    pub save_path: Option<PathBuf>,
}

impl AchievementManager {
    /// `AchievementInitializer.Load` — the full registry in construction
    /// order (`CreateAchievementsList`'s `_achievements.Values`, which
    /// preserves insertion).
    pub fn registry() -> Self {
        Self {
            list: REGISTRY
                .iter()
                .enumerate()
                .map(|(id, row)| Achievement::from_row(row, id as u16))
                .collect(),
            save_path: None,
        }
    }

    /// `AchievementManager.Register`'s completion event shape — the caller
    /// learns which achievement just finished (`OnAchievementCompleted`).
    pub fn notify(&mut self, kind: CondKind) -> Option<&'static str> {
        let mut done = None;
        for a in &mut self.list {
            if a.completed {
                continue;
            }
            let mut touched = false;
            for c in &mut a.conditions {
                if c.completed || !matches_kind(c.kind, kind) {
                    continue;
                }
                touched = true;
                c.value += 1;
                if c.value >= c.max {
                    c.completed = true;
                }
            }
            if touched && a.is_completed() {
                a.completed = true;
                done = Some(a.name);
            }
        }
        // `AchievementCompleted` (`AchievementManager.cs:180-187`): the manager
        // saves the moment an achievement completes.
        if done.is_some() {
            self.save();
        }
        done
    }

    /// `AchievementManager.Save()` (`AchievementManager.cs:57-63`): write the
    /// bound `_savePath`. A manager with no path (a headless test App) is a
    /// no-op — the port binds the path at bootstrap, its stand-in for the C#
    /// constructor's `Main.SavePath/achievements.dat` (`:51`).
    pub fn save(&self) {
        let Some(path) = &self.save_path else { return };
        if let Err(e) = save(path, self) {
            log::warn!("achievements save failed: {e}");
        }
    }

    /// `AchievementsHelper.NotifyItemCraft(Recipe)` (`:73-79`) — the crafted
    /// item's id rides the event, so `BENCHED`'s workbench set matches on the
    /// crafted item rather than on any craft.
    pub fn notify_craft(&mut self, item: u16) -> Option<&'static str> {
        self.notify(CondKind::ItemCraft(item))
    }

    /// `RegisterIconIndex`'s table (`:445-582`) by name.
    pub fn icon_index(name: &str) -> Option<u16> {
        REGISTRY
            .iter()
            .find(|r| r.name == name)
            .map(|r| r.icon)
    }

    /// `CreateAchievementsList` + `UIAchievementListItem.CompareTo`: the
    /// completed rows first (registration order), then the open ones.
    pub fn sorted_indices(&self) -> Vec<usize> {
        let mut idx: Vec<usize> = (0..self.list.len()).collect();
        idx.sort_by(|&a, &b| {
            let (x, y) = (&self.list[a], &self.list[b]);
            y.completed
                .cmp(&x.completed)
                .then_with(|| x.id.cmp(&y.id))
        });
        idx
    }

    /// `Load` (`AchievementManager.cs:86-108` shape): the saved condition
    /// states overwrite, the completed flags follow the conditions.
    pub fn load_pairs(&mut self, rows: impl Iterator<Item = (String, Vec<(String, i32, bool)>)>) {
        for (name, conds) in rows {
            let Some(a) = self.list.iter_mut().find(|a| a.name == name) else {
                continue;
            };
            for (cname, value, completed) in conds {
                if let Some(c) = a.conditions.iter_mut().find(|c| c.name == cname) {
                    c.value = value;
                    c.completed = completed;
                }
            }
            a.completed = a.is_completed();
        }
    }

    /// The save rows — `(achievement name, [(condition name, value, completed)])`.
    pub fn save_rows(&self) -> Vec<(&'static str, Vec<(&str, i32, bool)>)> {
        self.list
            .iter()
            .map(|a| {
                (
                    a.name,
                    a.conditions
                        .iter()
                        .map(|c| (c.name.as_str(), c.value, c.completed))
                        .collect(),
                )
            })
            .collect()
    }
}

fn matches_kind(stored: CondKind, event: CondKind) -> bool {
    use CondKind::*;
    match (stored, event) {
        (NpcKilled(a), NpcKilled(b)) => a == b,
        (NpcKilledAny(ids), NpcKilled(b)) => ids.contains(&b),
        (ItemPickup(a), ItemPickup(b)) => a == b,
        (ItemPickupAny(ids), ItemPickup(b)) => ids.contains(&b),
        (TileDestroyed(a), TileDestroyed(b)) => a == b,
        (TileDestroyedAny(ids), TileDestroyed(b)) => ids.contains(&b),
        (ItemCraft(a), ItemCraft(b)) => a == b,
        (ItemCraftAny(ids), ItemCraft(b)) => ids.contains(&b),
        (ProgressionEvent(a), ProgressionEvent(b)) => a == b,
        (Custom(a), Custom(b)) => a == b,
        (CustomInt(a, _), CustomInt(b, _)) => a == b,
        (CustomFloat(a, _), CustomFloat(b, _)) => a == b,
        _ => false,
    }
}

/// `AchievementManager.Save`/`Load` narrowed to the port's text format — the
/// vanilla file is Rijndael-CBC+BSON with the "RELOGIC-TERRARIA" key
/// (`AchievementManager.cs:40-79`); the port writes
/// `Terrustaria/achievements.trdat` (the deviation keeps the vanilla
/// install's real file untouched).
pub fn save(path: &Path, manager: &AchievementManager) -> std::io::Result<()> {
    let mut s = String::from("1\n");
    for (name, conds) in manager.save_rows() {
        s.push_str(&format!("{name} {}\n", conds.len()));
        for (cname, value, completed) in conds {
            s.push_str(&format!("  {cname} {value} {}\n", completed as i32));
        }
    }
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    std::fs::write(path, s)
}

pub fn load(path: &Path, manager: &mut AchievementManager) {
    let Ok(s) = std::fs::read_to_string(path) else {
        return;
    };
    let mut lines = s.lines().skip(1); // the version head
    let mut rows = Vec::new();
    while let Some(head) = lines.next() {
        let mut parts = head.split_whitespace();
        let Some(name) = parts.next() else { break };
        let count: usize = parts.next().and_then(|c| c.parse().ok()).unwrap_or(0);
        let mut conds = Vec::new();
        for _ in 0..count {
            let Some(line) = lines.next() else { break };
            let mut p = line.split_whitespace();
            let Some(cname) = p.next() else { break };
            let value = p.next().and_then(|v| v.parse().ok()).unwrap_or(0);
            let completed = p.next().map(|v| v == "1").unwrap_or(false);
            conds.push((cname.to_string(), value, completed));
        }
        rows.push((name.to_string(), conds));
    }
    manager.load_pairs(rows.into_iter());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_registry_is_the_initializers_full_table() {
        let m = AchievementManager::registry();
        // 137 `new Achievement` blocks (`AchievementInitializer.cs:13-441`).
        assert_eq!(m.list.len(), 137);
        // `RegisterIconIndex` covers 0..136, one per row.
        let mut icons: Vec<u16> = m.list.iter().map(|a| a.icon).collect();
        icons.sort_unstable();
        assert_eq!(icons, (0..137).collect::<Vec<u16>>());
        // The category pass leaves exactly one row uncategorised.
        assert_eq!(
            m.list
                .iter()
                .filter(|a| a.category == Category::None)
                .count(),
            1
        );
        assert_eq!(
            m.list
                .iter()
                .find(|a| a.category == Category::None)
                .map(|a| a.name),
            Some("GAIN_TORCH_GODS_FAVOR")
        );
    }

    #[test]
    fn every_row_carries_a_distinct_condition_key() {
        let m = AchievementManager::registry();
        for a in &m.list {
            let mut names: Vec<&str> = a.conditions.iter().map(|c| c.name.as_str()).collect();
            let total = names.len();
            names.sort_unstable();
            names.dedup();
            // `Achievement._conditions` is a dictionary keyed by Name — a
            // duplicate would drop a condition.
            assert_eq!(names.len(), total, "{}", a.name);
        }
    }

    #[test]
    fn the_trackers_report_the_ports_progress() {
        let mut m = AchievementManager::registry();
        // BULLDOZER rides `UseTrackerFromCondition("Pick")` — the counter is
        // the condition's own value.
        m.notify(CondKind::CustomInt("Pick", 10000));
        let b = m.list.iter().find(|a| a.name == "BULLDOZER").unwrap();
        assert_eq!(b.tracker_values(), Some((1, 10000)));
        // SLAYER_OF_WORLDS rides the conditions-completed tracker: three
        // `Any` conditions, one kill closes the first.
        m.notify(CondKind::NpcKilled(13));
        let s = m.list.iter().find(|a| a.name == "SLAYER_OF_WORLDS").unwrap();
        assert_eq!(s.tracker_values(), Some((1, 3)));
    }

    #[test]
    fn the_multi_id_condition_completes_on_any_listed_id() {
        let mut m = AchievementManager::registry();
        // `NPCKilledCondition.Create(13, 14, 15)` is ONE condition
        // (`NPCKilledCondition.cs:20-30`) — the Eater of Souls kill closes it.
        assert_eq!(m.notify(CondKind::NpcKilled(14)), Some("WORM_FODDER"));
        // TIMBER's ten-wood set likewise opens on any listed wood.
        assert_eq!(m.notify(CondKind::ItemPickup(619)), Some("TIMBER"));
        // OOO_SHINY's tile set: any listed ore.
        assert_eq!(m.notify(CondKind::TileDestroyed(169)), Some("OOO_SHINY"));
        // BENCHED only opens on a workbench craft, not on any craft.
        assert_eq!(m.notify_craft(35), None);
        assert_eq!(m.notify_craft(36), Some("BENCHED"));
    }

    #[test]
    fn the_sorted_list_puts_completed_rows_first() {
        let mut m = AchievementManager::registry();
        m.notify(CondKind::NpcKilled(4)); // EYE_ON_YOU, id 9, icon 9
        let order = m.sorted_indices();
        let names: Vec<&str> = order.iter().map(|&i| m.list[i].name).collect();
        assert_eq!(names[0], "EYE_ON_YOU");
        // The rest keep registration order (`CompareTo`'s id tiebreak).
        assert_eq!(names[1], "TIMBER");
        assert_eq!(names[2], "BENCHED");
    }

    #[test]
    fn the_ported_hooks_still_complete_their_rows() {
        // GELATIN_WORLD_TOUR's tracker shape from the initializer: the
        // three gel net ids with the conditions-completed tracker.
        let mut m = AchievementManager::registry();
        assert_eq!(m.notify(CondKind::NpcKilled(-25)), Some("GELATIN_WORLD_TOUR"));
        // DUNGEON_HEIST needs both of its conditions.
        let mut d = m.list.iter_mut().find(|a| a.name == "DUNGEON_HEIST").unwrap();
        assert!(!d.completed);
        m.notify(CondKind::ItemPickup(327));
        assert!(!m.list.iter().any(|a| a.name == "DUNGEON_HEIST" && a.completed));
        m.notify(CondKind::ProgressionEvent(19));
        assert!(m.list.iter().any(|a| a.name == "DUNGEON_HEIST" && a.completed));
    }

    #[test]
    fn the_save_round_trips_the_condition_state() {
        let mut m = AchievementManager::registry();
        // An Eater of Worlds head closes one of SLAYER_OF_WORLDS' three
        // conditions, a wood pickup closes TIMBER's ten-item row.
        m.notify(CondKind::NpcKilled(13));
        m.notify(CondKind::ItemPickup(9));
        let path = std::env::temp_dir().join("tr-ach-test.trdat");
        save(&path, &m).unwrap();
        let mut fresh = AchievementManager::registry();
        load(&path, &mut fresh);
        let timber = fresh.list.iter().find(|a| a.name == "TIMBER").unwrap();
        assert!(timber.completed);
        let slayer = fresh
            .list
            .iter()
            .find(|a| a.name == "SLAYER_OF_WORLDS")
            .unwrap();
        // One of three conditions rode the save, so the row stays open.
        assert!(!slayer.completed);
        assert_eq!(slayer.conditions[0].completed, true);
        assert_eq!(slayer.conditions[1].completed, false);
        // Big Booty's `PROGRESSION_EVENT_20` row round-tripped untouched.
        assert_eq!(
            fresh
                .list
                .iter()
                .find(|a| a.name == "BIG_BOOTY")
                .unwrap()
                .conditions[0]
                .completed,
            false
        );
        std::fs::remove_file(&path).unwrap();
    }
}
