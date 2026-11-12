//! Main-branch anchors — a canary against lossy merges.
//!
//! This repo is worked on by several agents at once, and their merges are
//! sometimes wholesale (`git commit-tree` over a tree built from one side), so
//! a merge can silently DELETE another agent's landed work while leaving no
//! conflict and no deletion commit of its own. That has happened repeatedly:
//! BUG-21's `enter_loaded_world` vanished twice, `touch_damage.rs` once and the
//! whole `worldgen/dungeon_features.rs` twice.
//!
//! The rows live in `anchors.txt` at the repo root (one `path|marker|feature`
//! line each) and are checked in two places: `crates/tr-core/build.rs` (so a
//! dropped feature stops the build) and this test (so `cargo test` names it).
//! The build script is the one that cannot be forgotten; this test exists to
//! give a readable failure inside the test run, and reads the shared list so
//! the two can never drift.
//!
//! Maintain the list, never the assertions: add a row when a feature lands and
//! update a marker when a refactor legitimately renames it.

use std::path::{Path, PathBuf};

#[test]
fn every_landed_feature_is_still_in_the_tree() {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let root = manifest
        .parent()
        .and_then(Path::parent)
        .expect("crates/<crate> sits two levels under the repo root")
        .to_path_buf();
    let list_path = root.join("anchors.txt");
    let list = std::fs::read_to_string(&list_path).unwrap_or_else(|e| {
        panic!(
            "{} could not be read ({e}) — the anchor list is part of the tree, \
             so its absence means a merge dropped it",
            list_path.display()
        )
    });
    let mut missing = Vec::new();
    let mut rows = 0usize;
    for line in list.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        rows += 1;
        let mut parts = line.splitn(3, '|');
        let (Some(path), Some(marker), Some(feature)) =
            (parts.next(), parts.next(), parts.next())
        else {
            missing.push(format!("malformed anchors.txt row: {line}"));
            continue;
        };
        let file = root.join(path.trim());
        match std::fs::read_to_string(&file) {
            Ok(text) => {
                if !text.contains(marker) {
                    missing.push(format!(
                        "{}: {} no longer contains {:?}",
                        feature.trim(),
                        path.trim(),
                        marker
                    ));
                }
            }
            Err(_) => missing.push(format!(
                "{}: {} is gone entirely",
                feature.trim(),
                path.trim()
            )),
        }
    }
    assert!(rows >= 10, "the anchor list carries the landed features ({rows})");
    assert!(
        missing.is_empty(),
        "a merge dropped landed work (the lossy-merge failure mode) — \
         re-land it (AGENTS.md step 6) instead of editing the list:\n  - {}",
        missing.join("\n  - ")
    );
}
