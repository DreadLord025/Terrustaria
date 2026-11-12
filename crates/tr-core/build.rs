//! Landed-feature anchor check, run at build time.
//!
//! Several agents merge into `main` at once and some of those merges are
//! wholesale (a tree rebuilt from one side), which silently deletes another
//! agent's landed work — this has eaten BUG-21 twice and the whole
//! `worldgen/dungeon_features.rs` twice. `anchors.txt` at the repo root lists
//! one (file, marker, feature) row per landed feature; this script fails the
//! build with the exact names when one disappears, so the loss cannot ship.
//!
//! The same rows are checked by `crates/tr-app/tests/main_anchors.rs`. To skip
//! the check for a one-off experiment set `TR_SKIP_ANCHORS=1`.
//!
//! When an anchor legitimately disappears because the feature was replaced,
//! update the row in `anchors.txt` in the same commit. Never just delete rows.

use std::path::{Path, PathBuf};

fn main() {
    let manifest = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let root = manifest
        .parent()
        .and_then(Path::parent)
        .expect("crates/<crate> sits two levels under the repo root")
        .to_path_buf();
    let list_path = root.join("anchors.txt");
    println!("cargo:rerun-if-changed={}", list_path.display());
    println!("cargo:rerun-if-env-changed=TR_SKIP_ANCHORS");
    if std::env::var_os("TR_SKIP_ANCHORS").is_some() {
        return;
    }
    let Ok(list) = std::fs::read_to_string(&list_path) else {
        println!(
            "cargo:warning=anchors.txt is missing at {} — the lossy-merge tripwire is not armed",
            list_path.display()
        );
        return;
    };
    // Watch every file the list mentions, so editing one re-runs the check.
    let mut missing = Vec::new();
    for line in list.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut parts = line.splitn(3, '|');
        let (Some(path), Some(marker), Some(feature)) =
            (parts.next(), parts.next(), parts.next())
        else {
            missing.push(format!("malformed anchors.txt row: {line}"));
            continue;
        };
        let file = root.join(path.trim());
        println!("cargo:rerun-if-changed={}", file.display());
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
    if !missing.is_empty() {
        panic!(
            "a merge dropped landed work (the lossy-merge failure mode).\n\
             If this was you: re-land the feature (see AGENTS.md step 6) — do not \
             delete the row from anchors.txt.\n  - {}",
            missing.join("\n  - ")
        );
    }
}
