<!-- base-commit: 958096d -->
<!-- generated: 2026-11-06 -->

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
  `itemdata.rs` is regenerated the same way — item fixes go through the generator, not the file.
- **No god-files.** Extend the matching module (`render/sky.rs` for sky things), don't grow
  `main.rs` or a new catch-all. A second copy of a mechanic → extract a shared helper/trait.
  A self-contained movement subsystem is a crate-root leaf module (`grapple.rs`), not a
  section inside `player.rs`.
- **No ad-hoc RNG in sim code.** Don't call `rand`/`rand01` inline where the C# has a named
  stream; route through the streams the app already owns (`item_rng`, per-consumer named
  u64 streams, `frame.rs:1413/:1421`) or document a pinned value as an ANALYSIS deviation
  (the `CloseDoor genRand.Next(3)` → 0 precedent).
- **No behaviour drift in "refactors".** RNG streams, frame-cache keys (`WorldFrameKey`),
  fixed-step order are frozen; optimisations must be pixel/state-identical and benchmarked
  (`TR_BENCH`/`TR_STATS`).
- **No `cargo fmt` on generated files** (`tiledata/`, `wld.rs`); rustfmt only touched files.
- **No blanket staging** (`git add -A`/`git add .`) and no AI attribution in commits.

## Code Examples

DO — port with citation + descriptive Rust name (`crates/tr-app/src/render/sky.rs:226-238`):
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

DO — multi-tile objects are data, not code: a new object is one `TileObjectDef` entry in
`TABLE` with the C# numbers in the doc comment (`crates/tr-core/src/tileobject.rs:516-543`):
```rust
/// `Style2x2` preset (`:3451-3460`) with the chest overrides (`:3461-3469`): ...
const CHEST: TileObjectDef = TileObjectDef {
    kind: 21, width: 2, height: 2, origin: Point16::new(0, 1),
    anchor_bottom: AnchorData::new(ANCHOR_SWTSS, 2, 0),
    coordinate_heights: &[16, 18], frame_important: true, // `Main.cs:7656`
    ..base_def()
};
```
`can_place`/`place` read only the table; per-type *behaviour* (kill cascade, interaction)
is the exception and gets a named arm.

DO — a deterministic stand-in is allowed, but must be documented as a deviation where it
deviates (`sky.rs:153-164`): the cloud field is generated from the world seed with the
deviations (wind drift, overlap rejection) listed in `docs/ANALYSIS.md`.

## Error Handling

- I/O and file parsing use `anyhow` with `bail!`/`Context` (`crates/tr-core/src/save.rs:178,182`):
  `bail!("not a TRWLD save")`.
- `unwrap()`/`panic!` only where the invariant is local and provable (test code,
  run-length bookkeeping in `save.rs`) — never on user/network input paths.
- Sim/core lookups return `Option` (`world.tile(x, y)?` in `edit.rs` arms); an absent cell
  is a normal state, not an error.
- Render code returns plain values; `Option` guards stand in for missing sheets and an
  empty field draws nothing (`sky.rs` `Option<BgTex>` pattern).

## Imports & Dependency Direction

- `tr-core` (world/tiles/lighting/save) never depends on `tr-app`; `tr-app` holds
  rendering/sim/UI and depends on `tr-core` + `tr-render`. No upward imports, no cycles.
- The seam between them: core returns *plans/requests* (`TileKill { object_drops }`,
  `DropRequest`), the app executes them (spawn item drops, sounds). Core never spawns
  app-side entities or reads `App` state.
- Crates stay flat: one `lib.rs`, modules in `src/`; generated data in its own
  `tiledata/` module behind a hand-written `mod.rs`.

## Testing Patterns

- `#[cfg(test)] mod tests` at the bottom of the same file (`framing.rs:1243`,
  `loot.rs:1123`, `sky.rs:1275`); no separate test crate.
- Tests port the C# recipe and say so in the name/body: `stars_follow_the_spawn_recipe`,
  `common_drop_gate_and_stack_range` (`sky.rs`, `loot.rs` tests).
- Movement features are tested through the Player tick: build a world with
  `test_world_helpers::ground()` (`crates/tr-app/src/test_world_helpers.rs:9`) or
  `ground_world()` (`player.rs` tests), `Player::spawn(&w)`, warm up ~120 ticks, then
  drive `p.update(&w, PlayerInput { .. })` loops and assert velocities/latches — the
  grapple tests (`player.rs:3247+`) are the model; every test's doc comment cites the C# lines.
- Editing/cascade tests share the fixtures in `world_sim/editing.rs`: `object_world()`
  (40×40, stone floor), `mining_app()`, `mine_frames()`.
- RNG-dependent logic is tested on a seeded `FastRandom` with the C# call order in the
  comment (`loot.rs` tests — "both consume the same FastRandom stream").
- In-development features get a deterministic `TR_<FEATURE>` env pin for screenshots and
  tests: `TR_HOOK` parse (`world_sim/player.rs:51-56`), `TR_DASH`/`TR_BOOTS`/`TR_WINGS`
  on Player fields (`player.rs` ~341-384), `TR_JUMPS` on the ExtraJump type.
- Visual work is verified with deterministic env pins (`TR_SHOT`, `TR_CAM`, `TR_DAY`,
  `TR_NOPLAYER`, `TR_NO_BG`, `TR_UI`, `TR_DEBUG=0`, `TR_STORM`, `TR_XY`) +
  `tools/pixel_diff.py`; two identical runs must be SHA256-equal.
- Full suite: `cargo test --workspace -- --skip migrates_version_2_kinds_to_item_ids`;
  judge only the `test result: ok` lines — cargo may exit 1 on success in this repo.

## Naming Conventions

- Rust names describe behaviour, not the C# identifier: `get_rock_transition_point`,
  `fill_terrain`, `advance_bg_alphas`; keep the mapping discoverable via the citation
  comment instead. C#-shorthand like `num2` never survives the port.
- Types/enums `PascalCase`, functions `snake_case`, constants `SCREAMING_SNAKE`.
- Ported per-type check functions keep the C# verb so the cascade reads like the source:
  `check_door_closed`, `check_chest` → new ones as `check_sunflower`, `check_pot`.
- Modules: one concept per file under `render/` (draw passes), `world_sim/`
  (fixed-step systems), `ui/` (screens/panels).

## File Organization

- New system → extend the module map in `docs/ARCHITECTURE.md`; a genuinely new concern
  gets a new module file next to its peers (`render/ambience.rs` beside `render/sky.rs`,
  `falling.rs`/`loot.rs`/`tileobject.rs` beside their tr-core peers), registered as
  `pub(crate) mod` in the folder's `mod.rs`. New `world_sim/` systems join as a module
  plus one `FixedStepper` tick in the original C# call order (`ARCHITECTURE.md:314`).
- A movement subsystem that owns state + spec tables + its own AI is a crate-root leaf
  beside its peers: `grapple.rs` (state struct, spec table, latch/pull math, app driver
  in `world_sim/player.rs:142-167`); rain/sand show the same shape for particle systems.
- Multi-tile tile objects: geometry/styles in `tr-core/src/tileobject.rs` (`TABLE`),
  kill-behaviour arms in `tr-core/src/edit.rs` (`check_object`), the app-side drop map in
  `tr-app/src/registry.rs` (`tile_drop`), hooks in `tr-app/src/world_sim/editing.rs`.
- App state lives as `App` fields (`crates/tr-app/src/app/mod.rs:77+`), sim state grouped
  with its system; per-frame hooks are registered in `app/frame.rs` next to
  `update_rain`/`update_npcs`.
- Scratch files only under `.code-foundations/` or `target/`; never the repo root.

## Technology Decisions

- Rendering: one atlas, one instanced sprite pipeline (`SpriteRenderer`,
  `push_back_sprite`/`push_front_sprite`); tiles 16 px on 18 px grid, trunks 20×20 on
  22 px, walls 36 px cell/8 px inset — keep source rects on those grids.
- RNG layering (order matters when choosing): (1) `FastRandom`
  (`tr-core/tile_light/palette.rs:20`) is the **exact C# `UnifiedRandom` stream** — use it
  when the C# stream must be replayed; (2) the drop engine (`tr-core/src/loot.rs`) is
  RNG-generic — it takes `&mut dyn DropRng` and returns `DropRequest`s, never touching
  world state; the app drives it from `App.item_rng`; (3) other per-consumer visual/sim
  randomness uses named u64 streams through `next_rand`/`rand01`
  (`crates/tr-app/src/app/frame.rs:1413/:1421`) — one stream per consumer (crafting,
  growth, scheduler; `rain.rs` owns its own with `StreamReplay` for replay); (4) worldgen
  and the wind walk share one xorshift core — `Xorshift64` (`tr-core/src/rng.rs`), the
  13/7/17 triple carrying both draw conventions (worldgen's high-32-bit `next_u32` +
  `range`/`chance`/`next_f64`, re-exported as `Rng` from `worldgen/noise.rs`; the wind
  walk's `from_seed` + `range`), byte-exact to the pre-refactor cores; (5) a pinned
  constant is the last resort and must be recorded in ANALYSIS. Determinism is a hard
  invariant.
- Time: 60 Hz fixed step (`FixedStepper`), one update per draw, accumulator clamped 3×.

## Exemplar Files

- `crates/tr-app/src/grapple.rs` — self-contained movement subsystem: state struct, spec
  table, latch/pull math, integration flag on Player, tests through the Player tick.
- `crates/tr-core/src/tileobject.rs` — data-only registry (new objects = one entry),
  `can_place`/`place` engine reading only the table, geometry tests.
- `crates/tr-core/src/loot.rs` — RNG-generic engine, C#-ordered rolls, seeded tests;
  the plan/request seam (`DropRequest`) done right.
- `crates/tr-app/src/world_sim/editing.rs` — app-side hooks (`break_drop_events`,
  `interaction_events`), fixture-driven cascade tests.
- `crates/tr-app/src/render/sky.rs` — port style, citations, deterministic stand-ins,
  visual test pins. `crates/tr-core/src/save.rs` — anyhow error style, binary format porting.
