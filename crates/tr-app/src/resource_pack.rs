//! Resource packs — `Terraria.IO.ResourcePack` (`IO/ResourcePack.cs:13-152`)
//! and `ResourcePackList` (`IO/ResourcePackList.cs:12-197`), the data layer
//! behind `UIResourcePackSelectionMenu` / `UIResourcePackInfoMenu` /
//! `UIWorkshopSelectResourcePackToPublish` (W4-d84, research/18 rows 2/5/6).
//!
//! Deviations (docs/ANALYSIS.md "The Workshop/ResourcePack screens"):
//! - Zip packs are not enumerated (`CreatePacksFromZips`, `:136-154`): the
//!   port has no zip reader; directory packs and saved-json entries that
//!   point at directories are the whole list. `is_compressed` stays false.
//! - `SocialAPI.Workshop == null` (no Steam): the workshop-folder pass
//!   (`CreatePacksFromWorkshopFolders`, `:156-180`) and the saved-entry
//!   workshop fallback (`:94-99`) never run, so `Branding` is always `None`.
//! - The icon (`CreateIcon`, `:103-111`) is not decoded here; the screens
//!   draw the atlas fallback icon (`Images/UI/DefaultResourcePackIcon`).

use std::path::{Path, PathBuf};

/// `ResourcePackVersion` (`IO/ResourcePackVersion.cs:8-88`): the
/// `pack.json` `"Version": {"major": M, "minor": m}` pair.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct PackVersion {
    pub major: i32,
    pub minor: i32,
}

impl PackVersion {
    /// `GetFormattedVersion` (`ResourcePackVersion.cs:84-87`): `M.m`.
    pub(crate) fn formatted(&self) -> String {
        format!("{}.{}", self.major, self.minor)
    }
}

/// `ResourcePack` (`IO/ResourcePack.cs:13-152`) — the manifest fields
/// `LoadManifest` reads (`:113-129`) plus the two mutable UI fields
/// (`IsEnabled`/`SortingOrder`, `:60-62`). `Branding` is always `None`
/// (no Steam, module-header deviation).
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ResourcePack {
    /// `FileName` (`:69`) — the folder name under `ResourcePacks/`.
    pub file_name: String,
    /// `FullPath` (`:71`).
    pub full_path: PathBuf,
    /// `Name` (`pack.json["Name"]`, `:125`).
    pub name: String,
    /// `Description` (`:126`).
    pub description: String,
    /// `Author` (`:127`).
    pub author: String,
    /// `Version` (`:128`).
    pub version: PackVersion,
    /// `IsEnabled` (`:60`) — the selection menu's toggle writes it.
    pub is_enabled: bool,
    /// `SortingOrder` (`:62`) — the enabled-list priority.
    pub sorting_order: i32,
    /// `IsCompressed` (`:64-68`) — always false (zip deviation).
    pub is_compressed: bool,
}

impl ResourcePack {
    /// The ctor + `LoadManifest` (`ResourcePack.cs:64-84`, `:113-129`) for a
    /// directory pack: `pack.json` must exist and parse, else the pack is
    /// rejected (the C# throws; the list catches and skips, `:129-132`).
    pub(crate) fn from_dir(path: &Path) -> Option<Self> {
        let text = std::fs::read_to_string(path.join("pack.json")).ok()?;
        let v: serde_json::Value = serde_json::from_str(&text).ok()?;
        // `jObject["Name"].Value<string>()` etc. (`:125-128`) — a missing
        // key throws in the C# (NullReferenceException), caught by the
        // list's try/catch: the pack is skipped here too.
        let name = v.get("Name")?.as_str()?.to_string();
        let description = v.get("Description")?.as_str()?.to_string();
        let author = v.get("Author")?.as_str()?.to_string();
        let ver = v.get("Version")?;
        let version = PackVersion {
            major: ver.get("major")?.as_i64()? as i32,
            minor: ver.get("minor")?.as_i64()? as i32,
        };
        Some(Self {
            file_name: path.file_name()?.to_string_lossy().into_owned(),
            full_path: path.to_path_buf(),
            name,
            description,
            author,
            version,
            is_enabled: false,
            sorting_order: 0,
            is_compressed: false,
        })
    }
}

/// `ResourcePackList` (`IO/ResourcePackList.cs:12-197`) — the flat
/// `_resourcePacks` store (`:22`) with the three sorted views.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct ResourcePackList {
    pub packs: Vec<ResourcePack>,
}

/// The `Name, Version, FileName` tiebreak every view shares
/// (`ResourcePackList.cs:26-37`).
fn by_name_version_file(a: &ResourcePack, b: &ResourcePack) -> std::cmp::Ordering {
    a.name
        .cmp(&b.name)
        .then(a.version.cmp(&b.version))
        .then(a.file_name.cmp(&b.file_name))
}

impl ResourcePackList {
    /// `EnabledPacks` (`ResourcePackList.cs:24-27`): `IsEnabled`,
    /// `orderby SortingOrder, Name, Version, FileName`. Indices into `packs`.
    pub(crate) fn enabled_packs(&self) -> Vec<usize> {
        let mut idx: Vec<usize> = (0..self.packs.len())
            .filter(|&i| self.packs[i].is_enabled)
            .collect();
        idx.sort_by(|&a, &b| {
            self.packs[a]
                .sorting_order
                .cmp(&self.packs[b].sorting_order)
                .then(by_name_version_file(&self.packs[a], &self.packs[b]))
        });
        idx
    }

    /// `DisabledPacks` (`:29-32`): `!IsEnabled`, `orderby Name, Version,
    /// FileName`.
    pub(crate) fn disabled_packs(&self) -> Vec<usize> {
        let mut idx: Vec<usize> = (0..self.packs.len())
            .filter(|&i| !self.packs[i].is_enabled)
            .collect();
        idx.sort_by(|&a, &b| by_name_version_file(&self.packs[a], &self.packs[b]));
        idx
    }

    /// `AllPacks` (`:34-37`): `orderby Name, Version, FileName`.
    pub(crate) fn all_packs(&self) -> Vec<usize> {
        let mut idx: Vec<usize> = (0..self.packs.len()).collect();
        idx.sort_by(|&a, &b| by_name_version_file(&self.packs[a], &self.packs[b]));
        idx
    }

    /// `ToJson` (`:47-53`): one `{FileName, Enabled, SortingOrder}` entry
    /// per pack, in store order (the C# `Select` keeps `_resourcePacks`
    /// order — the entry struct's fields, `:14-20`).
    pub(crate) fn to_json(&self) -> serde_json::Value {
        serde_json::Value::Array(
            self.packs
                .iter()
                .map(|p| {
                    serde_json::json!({
                        "FileName": p.file_name,
                        "Enabled": p.is_enabled,
                        "SortingOrder": p.sorting_order,
                    })
                })
                .collect(),
        )
    }

    /// `FromJson` (`:55-66`) minus the zip/workshop passes (module-header
    /// deviations): the saved entries first (`CreatePacksFromSavedJson`,
    /// `:81-114` — only entries whose path exists, carrying their
    /// `Enabled`/`SortingOrder`), then the directory pass
    /// (`CreatePacksFromDirectories`, `:116-134` — every folder not already
    /// listed, default state). A missing dir is the empty list (`:57-60`).
    pub(crate) fn from_config(saved: &serde_json::Value, search_path: &Path) -> Self {
        let mut out = Self::default();
        if !search_path.is_dir() {
            return out;
        }
        // `CreatePackEntryListFromJson` (`:182-196`): a non-array or empty
        // value is the empty entry list.
        if let Some(entries) = saved.as_array() {
            for e in entries {
                let Some(file_name) = e.get("FileName").and_then(|v| v.as_str()) else {
                    continue; // `item.FileName == null` (`:86-89`)
                };
                let path = search_path.join(file_name);
                // `File.Exists || Directory.Exists` (`:92`) — directories
                // only in the port (zip deviation).
                if !path.is_dir() {
                    continue;
                }
                if let Some(mut pack) = ResourcePack::from_dir(&path) {
                    pack.is_enabled = e.get("Enabled").and_then(|v| v.as_bool()).unwrap_or(false);
                    pack.sorting_order =
                        e.get("SortingOrder").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                    out.packs.push(pack);
                }
            }
        }
        // `CreatePacksFromDirectories` (`:116-134`): `Directory.GetDirectories`
        // order (alphabetical on Windows), deduped by `FileName` (`:124`).
        let Ok(entries) = std::fs::read_dir(search_path) else {
            return out;
        };
        let mut dirs: Vec<PathBuf> = entries
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.is_dir())
            .collect();
        dirs.sort();
        for dir in dirs {
            let folder = dir.file_name().map(|n| n.to_string_lossy().into_owned());
            let Some(folder) = folder else { continue };
            if out.packs.iter().any(|p| p.file_name == folder) {
                continue;
            }
            if let Some(pack) = ResourcePack::from_dir(&dir) {
                out.packs.push(pack);
            }
        }
        out
    }

    /// `Publishable` (`:68-79`) — the publish screen's list: zips +
    /// directories, no saved state. The port's zip deviation leaves the
    /// directory pass only; every pack keeps the default disabled state.
    pub(crate) fn publishable(search_path: &Path) -> Self {
        Self::from_config(&serde_json::Value::Array(vec![]), search_path)
    }

    /// `CleanUpResourcePackPriority` (`UIResourcePackSelectionMenu.cs:277-285`):
    /// the enabled packs renumber 0..n in `SortingOrder` order.
    pub(crate) fn clean_up_priority(&mut self) {
        for (n, i) in self.enabled_packs().into_iter().enumerate() {
            self.packs[i].sorting_order = n as i32;
        }
    }

    /// `SetResourcePackAsTopPriority` (`UIResourcePackSelectionMenu.cs:209-224`):
    /// a disabled pack is untouched; an enabled one lands at
    /// `max(other enabled SortingOrder) + 1` (the `-1` seed makes an empty
    /// neighbourhood order 0).
    pub(crate) fn set_top_priority(&mut self, index: usize) {
        if !self.packs[index].is_enabled {
            return;
        }
        let mut max = -1;
        for i in self.enabled_packs() {
            if i != index && max < self.packs[i].sorting_order {
                max = self.packs[i].sorting_order;
            }
        }
        self.packs[index].sorting_order = max + 1;
    }

    /// `OffsetResourcePackPriority` (`UIResourcePackSelectionMenu.cs:226-240`):
    /// swap `SortingOrder` with the neighbour at `clamp(pos + offset, 0,
    /// count - 1)` in the enabled view; a disabled pack or a clamped-to-self
    /// offset does nothing.
    pub(crate) fn offset_priority(&mut self, index: usize, offset: i32) {
        if !self.packs[index].is_enabled {
            return;
        }
        let enabled = self.enabled_packs();
        let Some(pos) = enabled.iter().position(|&i| i == index) else {
            return;
        };
        let target = (pos as i32 + offset).clamp(0, enabled.len() as i32 - 1) as usize;
        if target != pos {
            let (a, b) = (enabled[pos], enabled[target]);
            let order_a = self.packs[a].sorting_order;
            self.packs[a].sorting_order = self.packs[b].sorting_order;
            self.packs[b].sorting_order = order_a;
        }
    }
}

/// `AssetInitializer.GetResourcePacksFolderPathAndConfirmItExists`
/// (`Initializers/AssetInitializer.cs:58-63`): `Main.SavePath/ResourcePacks`
/// — the port's `Terrustaria/ResourcePacks` beside `Worlds/`
/// (`settings::path()`'s root), created on demand (`TryCreatingDirectory`).
pub(crate) fn resource_packs_dir() -> Option<PathBuf> {
    let dir = tr_core::save::default_world_dir()?
        .parent()?
        .join("ResourcePacks");
    let _ = std::fs::create_dir_all(&dir);
    Some(dir)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pack(name: &str, file: &str, enabled: bool, order: i32) -> ResourcePack {
        ResourcePack {
            file_name: file.into(),
            full_path: PathBuf::from(file),
            name: name.into(),
            description: String::new(),
            author: String::new(),
            version: PackVersion { major: 1, minor: 0 },
            is_enabled: enabled,
            sorting_order: order,
            is_compressed: false,
        }
    }

    /// `EnabledPacks`/`DisabledPacks`/`AllPacks` (`ResourcePackList.cs:24-37`):
    /// the enabled view sorts by order first, the other two by name; the
    /// version tiebreak precedes the file name.
    #[test]
    fn the_three_views_sort_like_the_source() {
        let mut list = ResourcePackList::default();
        list.packs = vec![
            pack("Beta", "b", true, 5),
            pack("Alpha", "a", true, 9),
            pack("Alpha", "a2", false, 0),
            pack("Gamma", "g", true, 1),
        ];
        // Enabled: order 1 (Gamma), 5 (Beta), 9 (Alpha).
        assert_eq!(list.enabled_packs(), vec![3, 0, 1]);
        // Disabled: only Alpha/a2.
        assert_eq!(list.disabled_packs(), vec![2]);
        // All: Alpha/a (name tie -> file name), Alpha/a2, Beta, Gamma.
        assert_eq!(list.all_packs(), vec![1, 2, 0, 3]);
        // The version tiebreak: same name, higher version sorts later.
        list.packs[2].version = PackVersion { major: 2, minor: 0 };
        assert_eq!(list.all_packs(), vec![1, 2, 0, 3]);
        list.packs[2].version = PackVersion { major: 0, minor: 5 };
        assert_eq!(list.all_packs(), vec![2, 1, 0, 3]);
    }

    /// `CleanUpResourcePackPriority` (`:277-285`): renumber in order.
    #[test]
    fn clean_up_renumbers_the_enabled_orders() {
        let mut list = ResourcePackList::default();
        list.packs = vec![
            pack("A", "a", true, 40),
            pack("B", "b", false, 7),
            pack("C", "c", true, 10),
        ];
        list.clean_up_priority();
        assert_eq!(list.packs[0].sorting_order, 1);
        assert_eq!(list.packs[1].sorting_order, 7, "disabled untouched");
        assert_eq!(list.packs[2].sorting_order, 0);
    }

    /// `SetResourcePackAsTopPriority` (`:209-224`): max-of-others + 1; the
    /// disabled early return; the empty-neighbourhood 0.
    #[test]
    fn top_priority_lands_above_every_other_enabled_pack() {
        let mut list = ResourcePackList::default();
        list.packs = vec![
            pack("A", "a", true, 3),
            pack("B", "b", true, 8),
            pack("C", "c", false, 100),
        ];
        list.set_top_priority(0);
        assert_eq!(list.packs[0].sorting_order, 9);
        list.set_top_priority(2);
        assert_eq!(list.packs[2].sorting_order, 100, "disabled untouched");
        let mut solo = ResourcePackList::default();
        solo.packs = vec![pack("S", "s", true, 50)];
        solo.set_top_priority(0);
        assert_eq!(solo.packs[0].sorting_order, 0, "the -1 seed + 1");
    }

    /// `OffsetResourcePackPriority` (`:226-240`): the neighbour swap, the
    /// clamp at both ends, the disabled no-op.
    #[test]
    fn offset_swaps_with_the_clamped_neighbour() {
        let mut list = ResourcePackList::default();
        list.packs = vec![
            pack("A", "a", true, 0),
            pack("B", "b", true, 1),
            pack("C", "c", true, 2),
            pack("D", "d", false, 3),
        ];
        list.offset_priority(1, 1); // B <-> C
        assert_eq!(list.packs[1].sorting_order, 2);
        assert_eq!(list.packs[2].sorting_order, 1);
        list.offset_priority(0, -1); // clamped to self: no change
        assert_eq!(list.packs[0].sorting_order, 0);
        list.offset_priority(3, -1); // disabled: no change
        assert_eq!(list.packs[3].sorting_order, 3);
    }

    /// `ToJson` (`:47-53`): store order, the three entry fields.
    #[test]
    fn to_json_keeps_store_order_and_the_entry_fields() {
        let mut list = ResourcePackList::default();
        list.packs = vec![pack("Z", "z", true, 2), pack("A", "a", false, 0)];
        let j = list.to_json();
        assert_eq!(j[0]["FileName"], "z");
        assert_eq!(j[0]["Enabled"], true);
        assert_eq!(j[0]["SortingOrder"], 2);
        assert_eq!(j[1]["FileName"], "a");
        assert_eq!(j[1]["Enabled"], false);
    }

    /// `FromJson` (`:55-66`): saved entries carry their state, the directory
    /// pass appends the rest deduped, a missing dir is empty, a broken
    /// manifest is skipped (the try/catch, `:109-112`).
    #[test]
    fn from_config_merges_saved_entries_and_directories() {
        let root = std::env::temp_dir().join(format!("tr_packs_test_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        for (dir, name) in [("saved", "Saved Pack"), ("fresh", "Fresh Pack")] {
            std::fs::create_dir_all(root.join(dir)).unwrap();
            std::fs::write(
                root.join(dir).join("pack.json"),
                format!(
                    "{{\"Name\":\"{name}\",\"Description\":\"d\",\"Author\":\"a\",\
                     \"Version\":{{\"major\":1,\"minor\":2}}}}"
                ),
            )
            .unwrap();
        }
        std::fs::create_dir_all(root.join("broken")).unwrap(); // no pack.json
        let saved = serde_json::json!([
            {"FileName": "saved", "Enabled": true, "SortingOrder": 4},
            {"FileName": "ghost", "Enabled": true, "SortingOrder": 0},
        ]);
        let list = ResourcePackList::from_config(&saved, &root);
        assert_eq!(list.packs.len(), 2, "ghost missing, broken skipped");
        assert_eq!(list.packs[0].file_name, "saved");
        assert!(list.packs[0].is_enabled);
        assert_eq!(list.packs[0].sorting_order, 4);
        assert_eq!(list.packs[0].version.formatted(), "1.2");
        assert_eq!(list.packs[1].file_name, "fresh");
        assert!(!list.packs[1].is_enabled);
        // A missing directory is the empty list (`:57-60`).
        assert!(ResourcePackList::from_config(&saved, &root.join("nope")).packs.is_empty());
        let _ = std::fs::remove_dir_all(&root);
    }
}
