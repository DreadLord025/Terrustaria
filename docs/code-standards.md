<!-- base-commit: fd6a2da -->
<!-- generated: 2026-11-09 -->

# Code Standards — Terraria-Rust

A 1:1 port of Terraria 1.4.5.8. The governing document is `AGENTS.md` (golden rule,
branch-per-task, verification); this file captures the code conventions LLMs otherwise
guess wrong. Authoritative maps: `docs/ARCHITECTURE.md` (modules), `docs/PORT_MAP.md`
(C#→Rust), `docs/ANALYSIS.md` (deliberate deviations).

## 1. Forbidden Patterns

- **Never invent game behaviour.** Read the decompiled C# first and cite it. Don't add a
  plausible constant; do carry the citation — it is how the port is re-verified. If the
  C# is approximate for us, port it anyway and record the deviation in
  `docs/ANALYSIS.md`.
- **Never hand-edit generated data.** `crates/tr-core/src/tiledata/{sets,emissive,merge}.rs`,
  `crates/tr-core/src/wld.rs`, `crates/tr-app/src/itemdata.rs`, `recipes_gen.rs`: fix the
  `tools/` generator and regenerate (empty diff required). A hand-written `mod.rs` beside
  them is fine.
- **No god-files.** Extend the matching module (`projectile.rs` for projectile things),
  don't grow `main.rs` or a new catch-all. A second copy of a mechanic → extract a shared
  helper/trait. A self-contained subsystem is a crate-root leaf (`grapple.rs`), not a
  section inside a hub file.
- **No ad-hoc RNG in sim code.** Where the C# has a named stream, route through the stream
  the app owns (`item_rng`, `craft_rng`, …) or a pinned value documented as an ANALYSIS
  deviation. A pinned constant is the last resort.
- **No behaviour drift in "refactors".** RNG streams, frame-cache keys (`WorldFrameKey`),
  fixed-step order are frozen; optimisations must be pixel/state-identical and benchmarked
  (`TR_BENCH`/`TR_STATS`).
- **No `cargo fmt` on generated files**; rustfmt touched files only. **No blanket staging**
  (`git add -A`/`git add .`/`git commit -a`), no AI commit attribution, never commit secrets
  or deps. **No task work in the primary checkout**, no `../TR-*` scratch; scratch lives in
  the gitignored `.code-foundations/` or `target/`.

## 2. Code Examples

DO — port with a one-line C# citation and a descriptive Rust name (the golden rule in
miniature):

```rust
// `Projectile.cs:43824-43831`: the flail's per-kind speeds scale by 1/meleeSpeed.
let throw_speed = params.throw_speed / player.melee_speed();
```

DON'T — copying the C# identifier, dropping the citation, or a free-floating constant:

```rust
fn GetParallax(fScale: f32) -> f32 { /* no source cite, C# casing */ }
```

DO — per-type data is a table with the C# set name and default in the doc comment
(`projectile.rs:362-441`, the yoyo ladder):

```rust
/// `ProjectileID.Sets.YoyosTopSpeed` (`ProjectileID.cs:427`) — unlisted → 10f.
const YOYO_TOP_SPEED: &[(u16, f32)] = &[(541, 9.0), (548, 11.0), …];
fn yoyo_table(table: &[(u16, f32)], default: f32, kind: u16) -> f32 {
    table.iter().find(|(k, _)| *k == kind).map_or(default, |(_, v)| *v)
}
```

DO — match the C# structure even when Rust is terser; keep the arm order verbatim where
the C# iterates a set (the `keep the arm order` comments in `tile_light/`). Multi-tile
objects stay data: one `TileObjectDef` entry with the C# numbers in the doc comment
(`tr-core/src/tileobject.rs:516-543`), arms only for per-type *behaviour*.

## 3. Error Handling

- I/O and file parsing use `anyhow` with `bail!`/`Context`
  (`tr-core/src/save.rs:178,182`: `bail!("not a TRWLD save")`).
- `unwrap()`/`panic!` only where the invariant is local and provable (test code,
  run-length bookkeeping in `save.rs`) — never on user/network/hostile-data paths.
- Sim/core lookups return `Option`; an absent cell is a normal state, not an error —
  `let Some(tile) = world.tile(tx, ty) else { continue; };` (`projectile.rs:466`).
- Registry probes: `registry::get(kind).is_some_and(|d| d.channel)`. The pool's spawners
  return `Option<usize>` — a kind without a `proj_data` row spawns nothing; callers use
  `if let Some(slot) = …`.

## 4. Imports & Dependency Direction

- `tr-core` (world/tiles/lighting/save/loot) never depends on `tr-app`; `tr-app` (sim +
  render + UI) depends on `tr-core` + `tr-render`. No upward imports, no cycles.
- The seam: core returns *plans/requests* (`TileKill`, `DropRequest`), the app executes
  them (drops, sounds). Core never spawns app-side entities or reads `App` state.
- Inside a file: `use crate::…` for siblings, `glam::Vec2` for world vectors, `tr_core::…`
  for core types; crates stay flat, generated data behind a hand-written `mod.rs`.

## 5. Testing Patterns

- `#[cfg(test)] mod tests` at the bottom of the same file (`projectile.rs:2474+`); no
  separate test crate. Every test's doc comment cites the C# lines it verifies.
- Fixtures per area: `ground()`/`one_npc()`/`tick()` (`projectile.rs:2479-2556`),
  `test_world_helpers::ground()` (`crates/tr-app/src/test_world_helpers.rs:9`),
  `object_world()`/`mining_app()` (`world_sim/editing.rs` tests). Movement features drive
  the Player tick; the grapple tests (`player.rs:3247+`) are the model.
- RNG-dependent logic takes an explicit seeded stream: `FastRandom::new(seed)` in tests
  (the exact C# `UnifiedRandom` stream), `LootRng(&mut app.item_rng)` in the app (one
  item-stream draw per event, C# call order in the comment). Determinism-critical work
  proves state equality across seeds, never one lucky seed.
- In-development features get a `TR_<FEATURE>` env pin for screenshots/tests
  (`TR_HOOK`, `TR_SHOT`, `TR_CAM`, `TR_DAY`, `TR_UI`, `TR_DEBUG=0`, …) + `tools/pixel_diff.py`;
  two identical runs must be SHA256-equal.
- Full suite: `cargo test --workspace -- --skip migrates_version_2_kinds_to_item_ids` —
  judge only the `test result: ok` lines, cargo may exit 1 on success in this repo.

## 6. Naming Conventions

- Rust names describe behaviour, not the C# identifier: `get_rock_transition_point`,
  `fill_terrain`, `ai_flail`. C# shorthand (`num2`, `flag5`) never survives the port; the
  citation comment carries the mapping and the C# symbol name
  (`ai_boomerang` ↔ `AI_003_Boomerang`).
- Ported per-type check functions keep the C# verb so cascades read like the source
  (`check_door_closed`, `check_chest`). Types/enums `PascalCase`, functions `snake_case`,
  id constants `SCREAMING_SNAKE` with the verbatim vanilla `ProjectileID`/`ItemID` value.
- The pool mirrors C# fields minus camelCase (`time_left`, `knock_back`,
  `uses_local_immunity`, `ai`/`local_ai`). Test names are full sentences of the expected
  behaviour (`the_dropped_twin_passes_through_platforms`).

## 7. File Organization

- New system → extend the module map in `docs/ARCHITECTURE.md`; a new concern gets a module
  file next to its peers, registered as `pub(crate) mod`. `world_sim/` systems join as a
  module plus one fixed-step tick in the original C# call order (`ARCHITECTURE.md:314`).
- Projectile families: pool + per-family `ai_*` + `proj_data` in
  `crates/tr-app/src/projectile.rs`, the fixed-step pass in `world_sim/projectiles.rs`, the
  item-use wire in `world_sim/editing.rs`.
- App state lives as `App` fields (`crates/tr-app/src/app/mod.rs:77+`), sim state grouped
  with its system; per-frame hooks register in `app/frame.rs` next to their peers.

## 8. Technology Decisions

- RNG layering (order matters when choosing): (1) `FastRandom` (`tr-core/tile_light/
  palette.rs:20`) is the exact C# `UnifiedRandom` stream — replay C# streams with it;
  (2) the drop engine (`tr-core/src/loot.rs`) is RNG-generic (`&mut dyn DropRng`), driven
  from `App.item_rng`; (3) other per-consumer randomness uses named u64 streams through
  `next_rand`/`rand01` (`app/frame.rs:1413/:1421`, one stream per consumer); (4) worldgen +
  the wind walk share one xorshift core — `Xorshift64` (`tr-core/src/rng.rs`); (5) a pinned
  constant is the last resort, recorded in ANALYSIS. Determinism is a hard invariant.
- Rendering: one atlas, one instanced sprite pipeline; tiles 16 px on 18 px grid, trunks
  20×20 on 22 px, walls 36 px cell/8 px inset — keep source rects on those grids.
- Time: 60 Hz fixed step (`FixedStepper`), one update per draw, accumulator clamped 3×.
- `graphify` is the navigation tool; graphs are gitignored and refreshed at every workflow
  checkpoint (never two concurrent `graphify update` runs).
- Data assembly is generated by `tools/` from the decompiled sources — a regen must
  produce an empty diff, otherwise the decompile drifted.

## 9. Exemplar Files

- `crates/tr-app/src/projectile.rs` — the whole contract in one file: data table + citation
  comments + per-family `ai_*` + pool engine + seeded tests. Start a projectile-family port
  at `ai_boomerang` (`:1469`, the stateful-weapon peer) and `ai_yoyo` (`:992`, the
  channeled-weapon peer).
- `crates/tr-app/src/world_sim/projectiles.rs` — the fixed-step pass + inputs-snapshot
  seam.
- `crates/tr-app/src/grapple.rs` — self-contained movement subsystem (state struct, spec
  table, math, tests through the Player tick).
- `crates/tr-core/src/tileobject.rs` / `loot.rs` — data-only registry; the RNG-generic
  plan/request seam (`DropRequest`) done right.
- `docs/PORT_MAP.md` — what "done" looks like per system; `AGENTS.md` — the golden rule and
  the branch-per-task loop.
