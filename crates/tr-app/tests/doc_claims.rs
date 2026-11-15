//! The docs' render-structure claims pinned against the renderer's own
//! constants (S1-G-12): the stale "2 draw calls" claims must not return and
//! the honest bound must agree with `tr_render::sprite::PASS_BUFFERS`.

use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .map(|p| p.to_path_buf())
        .expect("crates/tr-app sits two levels under the repo root")
}

#[test]
fn the_architecture_doc_carries_the_honest_draw_call_bound() {
    let doc = std::fs::read_to_string(repo_root().join("docs/ARCHITECTURE.md"))
        .expect("docs/ARCHITECTURE.md exists");
    assert!(
        !doc.contains("2 draw calls"),
        "the stale \"2 draw calls\" claim returned to docs/ARCHITECTURE.md"
    );
    assert!(
        doc.contains("up to 6 draw calls"),
        "the honest bound (research/41 D-1) must stay"
    );
}

#[test]
fn the_renderer_pass_buffers_match_the_documented_bound() {
    // `sprite.rs::draw_ranges` emits one `pass.draw` per non-empty instance
    // buffer: the background, world, world-fx, map-back, map and UI ranges.
    assert_eq!(
        tr_render::sprite::PASS_BUFFERS, 6,
        "a seventh buffer needs the docs' bound update too"
    );
}
