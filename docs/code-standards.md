<!-- base-commit: 4dd6f0c -->
<!-- generated: 2026-11-03 -->

# Code Standards — Terraria-Rust

A 1:1 port of Terraria 1.4.5.8. The governing document is `AGENTS.md` (golden rule,
branch-per-task, verification); this file captures the code conventions LLMs otherwise
guess wrong. Authoritative maps: `docs/ARCHITECTURE.md` (modules), `docs/PORT_MAP.md`
(C#→Rust), `docs/ANALYSIS.md` (deliberate deviations).

## Forbidden Patterns

- **Never invent game behaviour.** Every game-logic change is read from the decompiled C#
  first (`decompiled/Terraria/**`) and cited. Don't: adding a plausible constant. Do:
  `// AmbientSky.cs:1319-1320` above the ported line, `SlotVector<SkyEntity>(500)`
  → `entities: Vec<SkyEntity>` (capacity noted in a comment).
- **Never hand-edit generated data.** `crates/tr-core/src/tiledata/{sets,emissive,merge}.rs`
  come from `tools/GenTileData` — fix the generator and regenerate (empty diff required).
- **No god-files.** Extend the matching module (`render/sky.rs` for sky things), don't grow
  `main.rs` or a new catch-all. A second copy of a mechanic → extract a shared helper/trait.
- **No behaviour drift in "refactors".** RNG streams, frame-cache keys (`WorldFrameKey`),
  fixed-step order are frozen; optimisations must be pixel/state-identical and benchmarked
  (`TR_BENCH`/`TR_STATS`).
- **No `cargo fmt` on generated files** (`tiledata/`, `wld.rs`); rustfmt only touched files.
- **No blanket staging** (`git add -A`/`git add .`) and no AI attribution in commits.

## Code Examples

DO — port with citation + descriptive Rust name (`crates/tr-app/src/render/sky.rs:119-130`):
```rust
/// `Cloud.GetParallax` (`Cloud.cs:318-338`): the per-cloud fraction of the
/// camera's X movement, from its scale bucket.
fn cloud_parallax(scale: f32) -> f32 {
    let (a, b) = if scale < 1.0 { (0.07, (scale + 0.15 + 1.0) / 2.0) } ...
```

DON'T — copying C# identifiers verbatim or dropping the citation:
```rust
fn GetParallax(fScale: f32) -> f32 { // no source cite, C# casing
```

DO — a deterministic stand-in is allowed, but must be documented as a deviation where it
deviates (`sky.rs:153-164`): the cloud field is generated from the world seed with the
deviations (wind drift, overlap rejection) listed in `docs/ANALYSIS.md`.

## Error Handling

- I/O and file parsing use `anyhow` with `bail!`/`Context` (`crates/tr-core/src/save.rs:11,58,139`):
  `bail!("not a TRWLD save")`.
- `unwrap()`/`panic!` only where the invariant is local and provable (test code,
  `save.rs:88-89` run-length bookkeeping) — never on user/network input paths.
- Render code returns plain values; `Option` guards stand in for missing sheets and an
  empty field draws nothing (`sky.rs` `Option<BgTex>` pattern, `sky.rs:239-289`).

## Imports & Dependency Direction

- `tr-core` (world/tiles/lighting/save) never depends on `tr-app`; `tr-app` holds
  rendering/sim/UI and depends on `tr-core` + `tr-render`. No upward imports, no cycles.
- Crates stay flat: one `lib.rs`, modules in `src/`; generated data in its own
  `tiledata/` module behind a hand-written `mod.rs`.

## Testing Patterns

- `#[cfg(test)] mod tests` at the bottom of the same file (`framing.rs:1243`,
  `biome.rs:137`, `sky.rs:1086`); no separate test crate.
- Tests port the C# recipe and say so in the name/body: `stars_follow_the_spawn_recipe`,
  `clouds_stay_within_the_vanilla_scale_clamps` (`sky.rs:1140,1157`).
- Visual work is verified with deterministic env pins (`TR_SHOT`, `TR_CAM`, `TR_DAY`,
  `TR_NOPLAYER`, `TR_NO_BG`, `TR_UI`, `TR_DEBUG=0`, `TR_STORM`, `TR_XY`) +
  `tools/pixel_diff.py`; two identical runs must be SHA256-equal.
- Full suite: `cargo test --workspace -- --skip migrates_version_2_kinds_to_item_ids`;
  judge only the `test result: ok` lines — cargo may exit 1 on success in this repo.

## Naming Conventions

- Rust names describe behaviour, not the C# identifier: `get_rock_transition_point`,
  `fill_terrain`, `advance_bg_alphas`; keep the mapping discoverable via the citation
  comment instead.
- Types/enums `PascalCase`, functions `snake_case`, constants `SCREAMING_SNAKE`;
  C#-shorthand like `num2` never survives the port.
- Modules: one concept per file under `render/` (draw passes), `world_sim/`
  (fixed-step systems), `ui/` (screens/panels).

## File Organization

- New system → extend the module map in `docs/ARCHITECTURE.md`; a genuinely new concern
  gets a new module file next to its peers (`render/ambience.rs` beside `render/sky.rs`),
  registered as `pub(crate) mod` in the folder's `mod.rs`.
- App state lives as `App` fields (`crates/tr-app/src/app/mod.rs:77+`), sim state grouped
  with its system; per-frame hooks are registered in `app/frame.rs` next to
  `update_rain`/`update_npcs`.
- Scratch files only under `.code-foundations/` or `target/`; never the repo root.

## Technology Decisions

- Rendering: one atlas, one instanced sprite pipeline (`SpriteRenderer`,
  `push_back_sprite`/`push_front_sprite`); tiles 16 px on 18 px grid, trunks 20×20 on
  22 px, walls 36 px cell/8 px inset — keep source rects on those grids.
- RNG: `tr-core` `FastRandom` (`tile_light/palette.rs:20`) is the **exact C# stream** —
  use it when the C# stream matters; `spawner.rs:30` documents the stand-in for places
  that only need uniformity. Worldgen uses the xorshift stream; determinism is a hard
  invariant.
- Time: 60 Hz fixed step (`FixedStepper`), one update per draw, accumulator clamped 3×.

## Exemplar Files

- `crates/tr-app/src/render/sky.rs` — port style, citations, deterministic stand-ins,
  tests. `crates/tr-core/src/save.rs` — anyhow error style, binary format porting.
- `crates/tr-app/src/world_sim/scene_scan.rs` — sensor/update module shape
  (M1.4-BG-18 pattern). `crates/tr-app/src/biome.rs` — small focused module + tests.
