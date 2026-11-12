//! Main-branch anchors — a canary against lossy merges.
//!
//! This repo is worked on by several agents at once, and their merges are
//! sometimes wholesale (`git commit-tree` over a tree built from one side), so
//! a merge can silently DELETE another agent's landed work while leaving no
//! conflict and no deletion commit of its own. That has happened repeatedly:
//! BUG-21's `enter_loaded_world` vanished twice, and the whole
//! `worldgen/dungeon_features.rs` (spikes + lights) plus `touch_damage.rs` were
//! dropped by `9f89c99` before being re-landed.
//!
//! Each entry below is a (source file, marker) pair naming one landed feature
//! that must stay in the tree. The test fails with a message that names the
//! feature, so a dropped merge is caught by the normal `cargo test` run
//! instead of by a screenshot or a player report a day later.
//!
//! How to maintain it: add an anchor when a feature lands (one per major
//! module/entry point), and when a refactor legitimately renames something,
//! update the marker in the same commit. Never remove an anchor to make a test
//! pass — that is exactly the failure the file exists to catch.

/// (file, marker, feature name) — the marker is the smallest string that only
/// exists while the feature does.
const ANCHORS: &[(&str, &str, &str)] = &[
    (
        "src/app/mod.rs",
        "fn enter_loaded_world",
        "BUG-21 — single-player world entry (Play must leave the lobby)",
    ),
    (
        "src/app/mod.rs",
        "static_sheets",
        "BUG-21 — the cached world-independent atlas half",
    ),
    (
        "src/npc.rs",
        "fn walker_frame",
        "BUG-22/24 — the walker NPC row band",
    ),
    (
        "src/npc.rs",
        "fn extra_frames_count",
        "BUG-22 — NPCID.Sets.ExtraFramesCount",
    ),
    (
        "src/town_arrival.rs",
        "fn update_time_spawn_town_npcs",
        "M4-TOWN-1b — the town NPC arrival gates",
    ),
    (
        "src/touch_damage.rs",
        "fn apply_touch_damage",
        "M4-HAZARD-1 — tile contact damage (dungeon spikes)",
    ),
    (
        "src/touch_damage.rs",
        "TOUCH_DAMAGE_IMMEDIATE",
        "M4-HAZARD-1 — the TileID.Sets.TouchDamageImmediate table",
    ),
    (
        "../tr-core/src/worldgen/dungeon_features.rs",
        "fn place_spikes",
        "M5-DUNGEON — the DungeonGlobalSpikes pass",
    ),
    (
        "../tr-core/src/worldgen/dungeon_features.rs",
        "fn place_lights",
        "M5-DUNGEON — the DungeonGlobalLights pass",
    ),
    (
        "../tr-core/src/worldgen/dungeon_features.rs",
        "fn wall_variants",
        "M5-DUNGEON — the DungeonGlobalWallVariants pass",
    ),
    (
        "../tr-core/src/worldgen/dungeon.rs",
        "fn generate_dungeon",
        "M5-DUNGEON — the faithful dungeon crawler",
    ),
    (
        "../tr-core/src/worldgen/world_hash.rs",
        "fn world_hash",
        "the world-gen determinism pins",
    ),
    (
        "src/ui/housing.rs",
        "fn ",
        "M7-UI-3c — the housing hover UI module",
    ),
];

/// Every anchor's marker must be present in its file.
#[test]
fn every_landed_feature_is_still_in_the_tree() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut missing = Vec::new();
    for (file, marker, feature) in ANCHORS {
        let path = root.join(file);
        let text = match std::fs::read_to_string(&path) {
            Ok(t) => t,
            Err(_) => {
                missing.push(format!("{feature}: {file} is gone entirely"));
                continue;
            }
        };
        if !text.contains(marker) {
            missing.push(format!(
                "{feature}: {file} no longer contains {marker:?}"
            ));
        }
    }
    assert!(
        missing.is_empty(),
        "a merge dropped landed work (the lossy-merge failure mode):\n  - {}",
        missing.join("\n  - ")
    );
}
