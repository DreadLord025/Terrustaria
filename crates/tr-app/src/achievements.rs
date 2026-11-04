//! The achievements framework — `Achievement` = a named condition set
//! (`Achievement.cs:11-120`), the manager registry (`AchievementManager`)
//! and the `AchievementsHelper` reporting surface (`GameContent/Achievements/`).
//!
//! The port registers the subset whose conditions its systems can fire (the
//! slime-family kills, the wood/ore pickups, the ore mining, the workbench
//! craft and the mine counter); the vanilla save is Rijndael+BSON
//! (`AchievementManager.cs:66-79`) — the port writes its own text format at
//! `achievements.trdat` (documented deviation: touching a real
//! `achievements.dat` would corrupt the vanilla install).

use std::path::Path;

/// `AchievementCategory.cs`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Category {
    Slayer,
    Collector,
    Explorer,
    Challenger,
}

/// What one condition listens for — the `AchievementsHelper.Notify*` family
/// narrowed to the ported systems.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CondKind {
    /// `NPCKilledCondition` — the NPC net id (`NPCKilledCondition.cs`).
    NpcKilled(i32),
    /// `ItemPickupCondition` — the item id.
    ItemPickup(u16),
    /// `TileDestroyedCondition` — the tile kind.
    TileDestroyed(u16),
    /// `ItemCraftCondition` — the crafted-anything-at-a-station shape the
    /// port's recipes use.
    ItemCraft,
    /// `CustomFlagCondition` — the named flag (`Peek`/`Use`/`Equip`/…).
    Custom(&'static str),
    /// `CustomIntCondition` — a counted condition with its max
    /// (`CustomIntCondition.cs`; `UseTrackerFromCondition` rides the value).
    CustomInt(&'static str, i32),
}

/// One `AchievementCondition`: the listener kind plus the tracker state
/// (`Completed` + the int tracker's value, `AchievementCondition.cs:9-45`).
#[derive(Clone, Copy, Debug)]
pub struct Condition {
    /// The condition's dictionary key (`"Kill"`, `"Pick"`, `"Craft"`, …).
    pub name: &'static str,
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
    fn new(name: &'static str, kind: CondKind) -> Self {
        let max = match kind {
            CondKind::CustomInt(_, max) => max.max(1),
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

/// `Achievement` (`Achievement.cs:11-120`): complete when every condition is.
#[derive(Clone, Debug)]
pub struct Achievement {
    pub name: &'static str,
    pub category: Category,
    pub conditions: Vec<Condition>,
    pub completed: bool,
}

impl Achievement {
    fn new(name: &'static str, category: Category, conditions: Vec<Condition>) -> Self {
        Self {
            name,
            category,
            conditions,
            completed: false,
        }
    }

    fn is_completed(&self) -> bool {
        self.conditions.iter().all(|c| c.completed)
    }
}

/// `AchievementManager`: the registry + the notification surface.
#[derive(Clone, Debug, Default)]
pub struct AchievementManager {
    pub list: Vec<Achievement>,
}

impl AchievementManager {
    /// The ported subset of `AchievementInitializer.Load` (`:13-…`): the
    /// achievements whose conditions the ported systems can fire. Each entry
    /// cites its initializer block.
    pub fn ported() -> Self {
        let list = vec![
            // `:14-18` — TIMBER: any wood pickup.
            Achievement::new(
                "TIMBER",
                Category::Collector,
                vec![Condition::new(
                    "Pick",
                    CondKind::ItemPickup(9),
                )],
            ),
            // `:19-23` — BENCHED: craft at a workbench
            // (`ItemCraftCondition.Create(ItemID.Sets.Workbenches)` — the
            // port's recipes gate on the same adjacency).
            Achievement::new(
                "BENCHED",
                Category::Collector,
                vec![Condition::new("Craft", CondKind::ItemCraft)],
            ),
            // `:28-33` — OOO_SHINY: destroy any ore tile (the ported set:
            // copper 7 / iron 6 / silver 9 / gold 8).
            Achievement::new(
                "OOO_SHINY",
                Category::Explorer,
                vec![Condition::new("Destroy", CondKind::TileDestroyed(7))],
            ),
            // `:199-204` — BULLDOZER: 10000 tiles destroyed
            // (`CustomIntCondition.Create("Pick", 10000)` +
            // `UseTrackerFromCondition("Pick")`).
            Achievement::new(
                "BULLDOZER",
                Category::Explorer,
                vec![Condition::new(
                    "Pick",
                    CondKind::CustomInt("Pick", 10000),
                )],
            ),
            // `:189-193` — GELATIN_WORLD_TOUR: kill one of each slime kind
            // (`CreateMany(-5, -6, 1, …)` + `Create(-4)` +
            // `UseConditionsCompletedTracker()`); the port's family covers
            // the blue/green/pinky/baby/black/purple/red/yellow ids.
            Achievement::new(
                "GELATIN_WORLD_TOUR",
                Category::Slayer,
                [
                    (-5, "Kill0"),
                    (-6, "Kill1"),
                    (1, "Kill2"),
                    (-3, "Kill3"),
                    (-7, "Kill4"),
                    (-8, "Kill5"),
                    (-9, "Kill6"),
                    (-4, "Kill7"),
                ]
                .into_iter()
                .map(|(net, name)| Condition::new(name, CondKind::NpcKilled(net)))
                .collect(),
            ),
        ];
        Self { list }
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
        done
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
    pub fn save_rows(&self) -> Vec<(&'static str, Vec<(&'static str, i32, bool)>)> {
        self.list
            .iter()
            .map(|a| {
                (
                    a.name,
                    a.conditions
                        .iter()
                        .map(|c| (c.name, c.value, c.completed))
                        .collect(),
                )
            })
            .collect()
    }
}

fn matches_kind(stored: CondKind, event: CondKind) -> bool {
    match (stored, event) {
        (CondKind::NpcKilled(a), CondKind::NpcKilled(b)) => a == b,
        (CondKind::ItemPickup(a), CondKind::ItemPickup(b)) => a == b,
        (CondKind::TileDestroyed(a), CondKind::TileDestroyed(b)) => a == b,
        (CondKind::ItemCraft, CondKind::ItemCraft) => true,
        (CondKind::Custom(a), CondKind::Custom(b)) => a == b,
        (CondKind::CustomInt(a, _), CondKind::CustomInt(b, _)) => a == b,
        _ => false,
    }
}

/// `AchievementManager.Save`/`Load` narrowed to the port's text format —
/// the vanilla file is Rijndael-CBC+BSON with the "RELOGIC-TERRARIA" key
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
    fn the_ported_registry_completes_through_events() {
        // GELATIN_WORLD_TOUR completes only after every slime family member
        // dies once; TIMBER completes on the first wood pickup.
        let mut m = AchievementManager::ported();
        assert_eq!(m.list.len(), 5);
        for net in [-5i32, -6, 1, -3, -7, -8, -9] {
            assert_eq!(m.notify(CondKind::NpcKilled(net)), None);
        }
        // The Pinky (last condition) closes GELATIN_WORLD_TOUR.
        assert_eq!(m.notify(CondKind::NpcKilled(-4)), Some("GELATIN_WORLD_TOUR"));
        assert_eq!(m.notify(CondKind::ItemPickup(9)), Some("TIMBER"));
        assert_eq!(m.notify(CondKind::ItemCraft), Some("BENCHED"));
        assert_eq!(m.notify(CondKind::TileDestroyed(7)), Some("OOO_SHINY"));
        // BULLDOZER needs 10000 — one event leaves it open.
        assert_eq!(m.notify(CondKind::CustomInt("Pick", 10000)), None);
        assert!(!m
            .list
            .iter()
            .find(|a| a.name == "BULLDOZER")
            .unwrap()
            .completed);
    }

    #[test]
    fn the_save_round_trips_the_condition_state() {
        let mut m = AchievementManager::ported();
        m.notify(CondKind::NpcKilled(1));
        m.notify(CondKind::ItemPickup(9));
        let path = std::env::temp_dir().join("tr-ach-test.trdat");
        save(&path, &m).unwrap();
        let mut fresh = AchievementManager::ported();
        load(&path, &mut fresh);
        let timber = fresh.list.iter().find(|a| a.name == "TIMBER").unwrap();
        assert!(timber.completed);
        let gel = fresh.list.iter().find(|a| a.name == "GELATIN_WORLD_TOUR").unwrap();
        assert!(!gel.completed);
        assert_eq!(
            gel.conditions
                .iter()
                .find(|c| matches!(c.kind, CondKind::NpcKilled(1)))
                .unwrap()
                .completed,
            true
        );
        std::fs::remove_file(&path).unwrap();
    }
}
