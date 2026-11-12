# 39 — Rust port architecture inventory

> Wave 4 · researched 2026-11-12 · sources: decompiled (C# 1.4.5.8) + crates + docs/ARCHITECTURE.md + docs/PORT_MAP.md + docs/ROADMAP.md + docs/ANALYSIS.md + docs/research/01..29,40

## Scope

This module is not a subsystem — it is the **map every other module's gaps plug into**.
It inventories the four crates, maps each C# god class onto its Rust seams, and
extracts the structural problems that the other research files keep hitting (god
modules, cross-module mutable state, borrow-split workarounds, duplicated logic) so
the ROADMAP Stage-2 section can be written against reality.

Measured on the worktree root (`crates/`, 266 `.rs` files, ~328 500 lines):

| Crate | Files | Lines | Top-level modules | `#[test]` | Role |
| --- | --- | --- | --- | --- | --- |
| `tr-core` | 105 | 104 851 | 48 (`lib.rs`) | 1 252 | data & formats: `World`/`Tile`, worldgen, lighting, liquids, wiring, IO codecs |
| `tr-app` | 141 | 217 549 | 70 (`main.rs`) | 2 169 | the game: `App` state, fixed-step sim, rendering passes, HUD, net |
| `tr-content` | 12 | 2 705 | 9 | 19 | the retail `.xnb` pipeline (XNB v5, LZX, DXT, fonts) |
| `tr-render` | 8 | 3 423 | 6 | 22 | wgpu 30: atlas packer, one instanced sprite pipeline, post passes |

`tr-app` is **66 % of the codebase** and 63 % of the tests. `tr-core` is the only
crate that may be touched by a "read-only world" phase; `tr-app` is where every
simultaneous-mutation problem lives.

### `tr-core` — data & formats (the read-only half)

| Module (lines) | Role |
| --- | --- |
| `world.rs` (948) | `World`: 132 public fields — tiles, `column_surface`, `chunk_has`, seeds, **and every boss/event flag** |
| `tile.rs` (154) | `Tile`, 12 B, slope+paint and wall-frame bit packing |
| `framing.rs` (1 357) | autotiling — band-parallel on Rayon (the only parallel code in the sim-facing core) |
| `lighting/` (1 284) | `LightMap` + the `LightEngine` trait seam, 9-slice, blur |
| `tile_light/` (1 824) | the 147-case `TileLightScanner` port + `FastRandom` palette |
| `liquid.rs` (542) + `liquid_draw.rs` (522) + `waterfall/` (1 591) | flow sim + draw cache + waterfall manager |
| `edit.rs` (6 791) | place/break, the KillTile cascade, `TileObjectData` calls |
| `tileobject.rs` (3 613) | the `TileObjectData` registry (36 defs of the full set) |
| `growth.rs` (5 305) / `spread.rs` (817) / `ores.rs` (338) | the world-growth ticks |
| `smart_cursor.rs` (5 197) | the 40-step `SmartCursorHelper` port |
| `wiring.rs` (4 622) | the wire engine + traps + statues |
| `worldgen/` (31 files, 34 476) | the pass pipeline: `pipeline.rs` (2 958) the runner, then one file per biome family |
| `wld/` (10 files) + `save.rs` (1 740) + `binio.rs` (450) + `map/` | the save formats over one little-endian cursor |
| `scene.rs` (2 488) | `SceneState`, filters, `SceneMetrics`, `Sandstorm` |
| `loot.rs` (2 035) | the 22 drop-rule classes |
| `localization/` (1 050) | `LocalizedText`, `LanguageManager`, cultures, format |

### `tr-app` — the game

| Directory (lines) | Role |
| --- | --- |
| `main.rs` (163) | bootstrap only: world path, content dir, settings, atlas, event loop |
| `app/` (18 371) | `App` (257 fields), `loop.rs` (winit), `frame.rs` (the `Main.Draw` orchestrator), `flow.rs` (front-end FSM), `clicks.rs`, `netglue.rs`, `saveglue.rs`, `session.rs`, `filters.rs`, `settings_ui.rs` |
| `world_sim/` (23 048) | the 60 Hz simulation: `step.rs` (the shared `FixedStepper`), `npcs.rs`, `editing.rs`, `player.rs`, `items.rs`, `projectiles.rs`, `weather.rs`, `wiring.rs`, `scene_scan.rs`, `growth.rs`, `liquids.rs`, `clock.rs`, `camera.rs`, `extract.rs`, `lunar.rs`, `smart_cursor.rs`, `tile_entities.rs` |
| `render/` (11 301) | `sky`, `bg`, `tiles`, `liquids`, `entities`, `player_layers`, `foliage`, `wires`, `debug` |
| `ui/` (24 424) | HUD panels + the UIState front-end screens + the shared `uistate`/`pointer`/`framework` kit |
| `menu/` (4 396) | the legacy immediate-mode `Main.DrawMenu` port |
| leaf | `npc.rs` (17 134), `projectile.rs` (17 411), `player.rs` (10 098), `itemdata.rs` (7 527, generated), `recipes_gen.rs` (19 513, generated), `chest.rs`, `minecart.rs`, `mount.rs`, `spawner.rs`, `shop.rs`, `net.rs`, `assets.rs`, `settings.rs` |

## Key entities

- `crates/tr-app/src/app/mod.rs::App` (L82–~700, **257 fields**) — the port's `Terraria.Main`. Holds the world, the entity stores, the RNG streams, the UI state, the audio, the atlas and the frame caches. The single mutable global of the port.
- `crates/tr-core/src/world.rs::World` (L10, **132 fields**) — `Terraria.World`'s tile data *plus* `Main`'s boss/event statics (`downed_*` ×20, `invasion_*`, `tower_shield_*`, `slime_rain*`, `moon_*`, `ore_tiers`, `seeds`, `micro_seeds`).
- `crates/tr-app/src/npc.rs::Npc` (L2446, **84 fields**) — the NPC entity, AoS, with `immune: [i32; 256]` and `buffs: NpcBuffs` inline.
- `crates/tr-app/src/projectile.rs::Projectile` (L2426, 51 fields) and `::ProjectilePool` (L7309) — the 1000-slot pool: `slots: Vec<Projectile>`, `local_immunity: Vec<i32>` (flattened MAX_PROJECTILES×256 — the one already-SoA structure), `id_static: HashMap<(u16,u16),u64>`, `hook_reg: [i16;10]`.
- `crates/tr-app/src/player.rs::Player` (L237, **239 fields**) — the player entity; `pending_sparks` / `pending_cart_sounds` / `pending_switch_trip` (L805–815) are the borrow-split escape hatches.
- `crates/tr-app/src/world_sim/step.rs::FixedStepper` (L9) — the shared 60 Hz accumulator; **the fixed-step order lives in the call order of `app/frame.rs`, not in the steppers**.
- `crates/tr-core/src/lighting/mod.rs::LightEngine` — trait seam for `ILightingEngine`.
- `crates/tr-app/src/app/flow.rs::FrontScreenBehaviours` — trait seam for `Terraria.UI.UIState`.
- `crates/tr-app/src/render/bg.rs::PassPrologue` — the shared `Main.DrawBackground*` prologue.
- `crates/tr-core/src/worldgen/pipeline.rs::VANILLA_CHAIN` (L370) / `EXECUTED_PIPELINE` / `Streams::pass_rng` — the worldgen pass model + per-pass RNG streams.
- `crates/tr-core/src/tile.rs::Tile` (12 B) — the row-major tile record every scan reads.

## Dependencies

- **Calls into (tr-app → tr-core):** `World`/`Tile` mutation (`edit::place_tile`, `kill_tile`, `break_wall`), `framing::reframe_area`, `liquid::wake_around`, `wiring` traps/actuation, `growth::update_world`, `spread::hard_update_world`, `tile_light` seeding, `loot::drop_money`, `worldgen::*` (generation only), the `wld`/`save` codecs, `map::*`, `localization::*`.
- **Calls into (tr-core → tr-app):** *none* — `tr-core` has no dependency on `tr-app` (verified in the four `Cargo.toml`s: `tr-core` depends only on anyhow/glam/log/rayon/serde_json). This is the crate boundary that makes a read-only world phase possible at all.
- **Called by:** `tr-app` (everything), `tr-render` (nothing — it only draws instance buffers), `tr-content` (nothing — it is a leaf decoder; `tr-app/build.rs` is its only consumer).
- **Shared data structures:** `World` (tiles + event flags), `Tile`, `ItemSlot` (`tr-app/src/item.rs`), `Lookups` (`tr-app/src/assets.rs`, 235 graph edges — the atlas/texture lookup god node), `NpcKind` (182 variants, 286 graph edges), `GenVars`/`Streams` (worldgen), `WireState`, `TileEntities`, `WorldBestiary`.
- **Graph hubs (graphify `god-nodes`, Rust graph):** `World` 1 403 edges · `App` 322 · `Player` 305 · `NpcKind` 286 · `Lookups` 235 · `Tile` 167 · `ItemSlot` 108 · `Projectile` 79. `World` and `App` together are the two poles of the port's coupling.

## Port status — the god-class seams

| C# god class (file, lines) | Rust crate::module | Lines | Fidelity verdict (cross-read) |
| --- | --- | --- | --- |
| `Main.cs` (65 265) | `tr-app::app/{mod,frame,loop,flow,clicks}.rs` | 18 371 | **PARTIAL** (research 01) — `DoUpdate` order diverges in 3 places (items/projectiles swapped, `time` before entities, growth/wiring/TE after NPCs); adaptive `gfxQuality` MISSING; frame-skip clamp PARTIAL (no draw suppression); `gamePaused` MISSING; `ShouldUpdateEntities` gate MISSING; camera follow moved from draw-time to sim-step |
| `Player.cs` (54 877) | `tr-app::player.rs` + `world_sim/player.rs` + `collision.rs` + `mount.rs` | 10 098 + 1 488 + 751 + 2 466 | **PARTIAL** (research 03) — movement/jump/dash/wings/grapple/fall-damage/regen PORTED; the wet/honey/merman/shimmer gravity branches are collapsed to one `GRAVITY/3`; `maxFallSpeed += 0.01` MISSING; vortex gravity MISSING; `CheckDrowning` PARTIAL (centre-tile gate vs the 10×12 head box) |
| `NPC.cs` (94 430) | `tr-app::npc.rs` + `npc_ai.rs` + `npc_buffs.rs` + `world_sim/npcs.rs` | 17 134 + 741 + 1 086 + 9 057 | **PARTIAL** (research 05) — the **central structural deviation**: AI dispatches on `NpcKind` (182) instead of `aiStyle` (128), so 74 aiStyles are unreferenced; `ScaleStats` MISSING; `GetAvailableNPCSlot` protocol absent (a `Vec` free list capped at 200); `behindTiles` MISSING; `UpdateNPC` phase order partial (18 named phases dropped) |
| `Item.cs` (49 703) | `tr-app::itemdata.rs` (generated) + `registry.rs` + `prefix.rs` + `item.rs` + `shop.rs` | 7 527 + 505 + 811 + 441 + 1 535 | **PORTED (data) / PARTIAL (behaviour)** (research 07) — all 6 196 ids generated from `SetDefaults`; the prefix tables/order/rounding are arm-for-arm; `Clone`/`Refresh` MISSING; the per-item `ToolTip` table and the instance-level fields (`favorited`, `buyOnce`, `questItem`) MISSING |
| `Tile.cs` (805) | `tr-core::tile.rs` + `tiledata/` | 154 + 3 394 | **PORTED** (research 08) — the field set, the slope/paint packing and the generated `TileID.Sets.*`; `checkingLiquid`/`skipLiquid` NOT-FOUND (the liquid sim uses its own path) |
| `Projectile.cs` (79 278) | `tr-app::projectile.rs` + `world_sim/projectiles.rs` | 17 411 + 311 | **PARTIAL** (research 06) — the pool, ownership, sub-step count and damage pipeline are ported; the `AI()` dispatch covers **32 of 210** arms; five duplicated dead arms (1, 3, 14, 99, 115); `Projectile.light` MISSING; the `oldPos`/`oldRot` trail cache MISSING; `timeLeft` burns at half rate for `extraUpdates ≥ 1` |
| `WorldGen.cs` (85 820) | `tr-core::worldgen/` (31 files) | 34 476 | **PARTIAL** (research 12) — the biome families (jungle/evil/ocean/desert/ores/temple) are faithful; `VANILLA_CHAIN` is a **model** (nothing walks it); `GenVars` carries ~25 of the C#'s ~100 statics; cave fields collapse five C# passes into one; the dungeon is a faithful crawler but the feature passes are out |
| `Collision.cs` (4 501) | `tr-app::collision.rs` + `tr-core::smart_cursor.rs::collision_empty_tile` | 751 + 5 197 | **PARTIAL** (research 13) — the app-side port and the core-side LOS helper are two homes for one mechanic (PORT_MAP itself flags the move as pending) |
| `Chest.cs` (3 600) | `tr-app::chest.rs` + `bank.rs` + `chest_gen.rs` + `tr-core::loot.rs` | 3 277 + 977 + 2 673 + 2 035 | **PORTED** (research 22) — the store, the lock decode, the `.wld` codec, the drop-rule engine; the chest-type icon table and the rigged-chest arm are MISSING |
| `Wiring.cs` (3 351) | `tr-core::wiring.rs` + `tr-app::world_sim::wiring.rs` | 4 622 + 711 | **PORTED (engine)** (research 23 pending, PORT_MAP) — the four colours, the trip/hit/switch, the traps, the statues; the entity-effect half is the app-side drain |

## Architecture problems (the centrepiece)

### 1. God modules — the "god-files are gone" claim is only half true

`AGENTS.md` and `docs/ARCHITECTURE.md` both assert the 2026-11-01 refactor removed the
god files. The *bootstrap* half is true: `main.rs` is 163 lines of pure bootstrap
(the claim says 86 — stale, see Deviations). The *entity* half is not: **eight
hand-written files are over 5 000 lines**, and they are exactly the files the other
research modules keep hitting.

| File | Lines | vs the C# it replaces | Generated? |
| --- | --- | --- | --- |
| `tr-app/src/projectile.rs` | 17 411 | `Projectile.cs` 79 278 (22 %) | hand |
| `tr-app/src/npc.rs` | 17 134 | `NPC.cs` 94 430 (18 %) | hand |
| `tr-app/src/player.rs` | 10 098 | `Player.cs` 54 877 (18 %) | hand |
| `tr-app/src/world_sim/npcs.rs` | 9 057 | (the NPC sim half of `NPC.cs`) | hand |
| `tr-app/src/world_sim/editing.rs` | 7 738 | (the edit half of `WorldGen.cs`) | hand |
| `tr-core/src/edit.rs` | 6 791 | `WorldGen.cs` edit/kill cascade | hand |
| `tr-core/src/growth.rs` | 5 305 | `WorldGen.UpdateWorld` | hand |
| `tr-core/src/smart_cursor.rs` | 5 197 | `SmartCursorHelper.cs` | hand |
| `tr-app/src/recipes_gen.rs` | 19 513 | `Recipe.cs` 16 668 | **generated** |
| `tr-app/src/itemdata.rs` | 7 527 | `Item.cs` 49 703 | **generated** |
| `tr-app/src/creative_sacrifices.rs` | 6 151 | `CreativePowers.cs` | **generated** |

Three of the top ten are generated data tables and are *not* god modules in the
architectural sense. The other eight are hand-written monoliths that mix the entity
model, the AI machines, the data tables and the tests in one file — exactly the shape
the refactor was supposed to kill. `npc.rs` at 17 k lines holds 84-field entity
structs, 182 `NpcKind` variants, the per-kind stat tables and ~1 000 test lines.

### 2. Cross-module mutable state — `World` and `App` are two god objects

- `App` (**257 fields**, `app/mod.rs` L82) is the `Main` equivalent, but it is *also*
  the entity store (`npcs: Vec<Npc>` L144, `player: Option<Player>` L140,
  `projectiles: ProjectilePool` L662, `items: Vec<ItemDrop>` L633), the RNG bank
  (`npc_rng`, `craft_rng`, `item_rng`, `clock_rng`, `ambience_rng`, `growth_rng`,
  `light_rng_seed`), the UI state and the render caches. Every `world_sim` function
  takes `&mut App`, so **no two systems can ever run concurrently**.
- `World` (**132 fields**, `tr-core/src/world.rs`) mixes immutable tile geometry with
  **runtime event state**: 20 `downed_*` boss flags, `invasion_*` (5 fields),
  `tower_shield_*` (4), `tower_active_*` (4), `slime_rain*` (3), `moon_*`,
  `ore_tiers`, `seeds`, `micro_seeds`, `altars_smashed`. Those are `Main.cs` statics
  in C#; here they live in the *tile container*. `tr-core` is therefore **not**
  read-only during a tick — `world_sim/clock.rs`, `world_sim/npcs.rs` and
  `world_sim/weather.rs` all write into it.
- The graph shows the same thing structurally: `World` 1 403 edges and `App` 322 edges
  are the two poles; `NpcKind` (286) and `Lookups` (235) are the third tier.

### 3. Borrow-split workarounds — no event bus, ad-hoc queues instead

Spawning / firing / breaking during an iteration over the entity store is the
recurring problem. The port has no event buffer; each site invented a private one:

| Workaround | Site | What it defers |
| --- | --- | --- |
| `Npc.wants_sphere: Vec<(NpcKind,f32,f32)>` | `npc.rs` L2568; drained at `world_sim/npcs.rs` L2601 | the Golem part spawns (the pre-fix version overwrote itself — a fixed bug, per ROADMAP M4-BOSS-19b) |
| `Npc.wants_minion` / `wants_minion_batch` | `npc.rs` L2508 / L2511 | the `AI_015` minion batch |
| `Player.pending_sparks` / `pending_cart_sounds` / `pending_switch_trip` | `player.rs` L805–815 | the minecart sparks/sounds/trip that need `&mut App` |
| `Wiring::take_trap_requests()` | `world_sim/wiring.rs` L27, L143 | the trap shots that need the projectile pool |
| `App.net_pending_sections` | `app/mod.rs` L369 | the net tile-block queue |
| `Npc.carrier_x/y` + `moon_core_alive` pre-pass | `world_sim/npcs.rs` L1933–1994 (the O(n) `position` lookup at L1947) | the Moon Lord part linkage (the C#'s `realLife` index links) |

The pattern is always the same: *the AI wants to mutate another entity; the borrow
checker forbids it; the mutation becomes a flag or a queue drained later.* Any Stage-2
"per-thread event buffer" design is really the formalisation of this ad-hoc pattern.

### 4. Duplicated logic across modules — concrete pairs

| # | Pair | Evidence |
| --- | --- | --- |
| 1 | `tr-core::edit::place_tile` (L2563) / `kill_tile` (L161) ↔ `tr-app::world_sim::editing::place_tile` (L3194) / `break_tile` (L1768) / `place_wall_item` (L1745) | two layers over the same mechanic (core = tile mutation, app = drop/sound/net glue) — the split is by *layer*, not by *seam* |
| 2 | `tr-core::wiring` (4 622) ↔ `tr-app::world_sim::wiring` (711) | engine vs 60 Hz driver, with the driver re-reading `app.wiring` state each tick |
| 3 | `tr-core::growth` (5 305) ↔ `tr-app::world_sim::growth` (50) | a 50-line app stub over a 5 305-line core module — the app side is vestigial |
| 4 | `tr-core::smart_cursor` (5 197) ↔ `tr-app::world_sim::smart_cursor` (280) ↔ `tr-app::app::clicks` (2 971) | one mechanic in three files; PORT_MAP already notes the core copy's "home is in-module until the app-side Collision port lands" |
| 5 | `tr-core::loot` (2 035) ↔ `tr-app::loot_db` (154) ↔ `tr-app::loot_db_gen` (1 439) | the rule engine in core, the database in app, bridged by `tr_core::loot::DropRng` over the app's `item_rng` |
| 6 | `tr-core::scene` (2 488) ↔ `tr-app::world_sim::scene_scan` (779) | the **read-only** world scan lives in the app while the state machine lives in core — this inversion is exactly what blocks the read-only phase |
| 7 | `tr-core::tile_light::palette::hsl_to_rgb` ↔ `tr-app::ui::color` | the same colour maths written twice |
| 8 | `tr-core::scene::move_towards` ↔ `tr-app::ambient_sound::move_towards` | duplicated helper |
| 9 | `tr-core::wind::update_rain` ↔ `tr-app::app::mod::update_rain` | same name, two crates, two behaviours |
| 10 | `tr-core::map::option` (802, the `GetTileBaseOption` port) ↔ `tr-app::map` (1 161) | the map model split across the crate boundary |

Cross-crate name collisions found mechanically: `break_wall`, `place_tile`, `place_wall`,
`can_place`, `hit_switch`, `update_rain`, `update_time`, `move_towards`, `hsl_to_rgb`,
`label`, `reset`, `active_culture`, `set_language`, `from_wld`, `to_wld`, `is_empty`.

### 5. No broad-phase structure exists

There is **no spatial hash, quadtree, bucket grid or archetype index** anywhere in
`crates/` (a search for `spatial|quadtree|bucket|grid_hash|hash_grid` returns only
game-content false positives such as `EmptyBucket`). Every broad-phase query is a
linear scan:

- `tr-app/src/projectile.rs` — 10 sites of `for (idx, n) in npcs.iter().enumerate()`
  (L2886, 3352, 3534, 4364, 5107, 5422, 6492, 6596, 7188, …)
- `tr-app/src/npc.rs` — 5 sites (L12565, 12665, 13697, 14521, …)
- `tr-app/src/world_sim/npcs.rs` — `app.npcs.iter().position(|m| m.kind == want && m.hp > 0)`
  inside the Moon-Lord pre-pass (L1948) — an O(n) search *per Moon-Lord part*.

Entity data is AoS (`Vec<Npc>` of an 84-field struct with an inline `[i32; 256]`
immunity array; `Vec<Projectile>` of a 51-field struct). The only SoA-ish structure in
the port is `ProjectilePool::local_immunity: Vec<i32>` (flattened MAX_PROJECTILES×256).

### 6. Determinism is currently protected by serialisation, not by design

The C# update order is reproduced by the *call order in `app/frame.rs`* (L328–557) plus
one `FixedStepper` per system. Research 01 already found three order inversions against
`Main.DoUpdateInWorld_Inner`. The RNG discipline is one `u64`/counter stream per
consumer, all living as flat fields on `App` — the streams are per-*consumer*, not
per-*archetype*, so a parallel phase must either re-home them or serialise them.

## Concurrency readiness (Stage-2 assessment)

The owner's Stage-2 plan — *phase A: read-only world + spatial hash; phase B: sequential
in C# order; per-thread event buffers; SoA by archetype* — is sound in principle but
each of its four pillars hits a specific, different obstacle in the current tree.

**Phase A — read-only world + spatial hash.**
The good news: `tr-core` has **zero** dependency on `tr-app`, and the read-only tile
pipeline is already proven parallel — `tr-core/src/framing.rs` frames the world in
128-row bands on Rayon with identical output, `tr-core/src/lighting/{mod,blur}.rs` seeds
and sweeps horizontally on the persistent pool. A read-only world phase is therefore
mechanically available for *tiles*.
The obstacle: `World` is not read-only during a tick. Its 132 fields mix tile data
with ~50 runtime event flags (`downed_*`, `invasion_*`, `tower_*`, `slime_rain*`,
`ore_tiers`, `seeds`) that `world_sim/{clock,npcs,weather}.rs` write. A phase-A
snapshot needs `World` split into `WorldTiles` (immutable) + `WorldState` (mutable), or
an explicit "phase A may not touch these fields" discipline that the compiler does not
enforce. Secondary hazard: `World.extras: HashMap<u32,u8>` — a `HashMap` iterates in a
nondeterministic order, so any phase-A pass that iterates it is a determinism hazard.
The spatial hash itself does not exist yet (see §5): it must be built, and it must be
built over **stable slots**, because `App.npcs` is a `Vec` whose indices shift on
compaction (`world_sim/npcs.rs` `retain`s dead NPCs) — the projectile immunity table
already documents this hazard ("the NPC's position in the app's `Vec<Npc>`: stable
between death compactions", `projectile.rs` L7312).

**Phase B — sequential in C# order.**
The order is currently a property of `app/frame.rs`'s call sequence, not of a data
structure, and it is already **wrong in three places** (research 01: items/projectiles
swapped vs `Main.cs:17977/18031`; `Main.time` runs before entities vs `:18083-18087`;
growth/wiring/tile-entities run after NPCs vs `Main.cs:18105-18109`). Phase B must
first *fix* the order (a behaviour change needing a determinism proof) and then
*protect* it — meaning the order must move out of the frame body into an explicit,
data-driven tick plan (an array of `(system, phase)` entries) so a parallel phase can
read the plan rather than the code.

**Per-thread event buffers.**
The mechanism exists in embryonic form only — `Npc.wants_sphere` (a typed
`Vec<(kind,sx,sy)>` drained at `world_sim/npcs.rs` L2601), `Wiring::take_trap_requests()`
and the `Player.pending_*` queues. Stage 2 needs one typed event enum
(spawn-npc / spawn-projectile / damage / drop-item / break-tile / sound / net-sync)
with per-thread buffers merged in a **deterministic order** (thread index, then
sequence number) so the merge reproduces the serial order exactly. The current
per-site queues have no ordering contract at all — `wants_sphere` is drained in
`Vec` push order, which happens to match the serial order only because there is one
thread today.

**SoA by archetype.**
Blocked by three facts. (a) `NpcKind` is a 182-variant enum and the AI dispatches on
it (research 05) — the natural archetype boundary (`aiStyle`) does not exist in the
Rust tree, so "SoA by archetype" has no key to group by until the `aiStyle` dispatch
is ported (74 aiStyles are currently unreferenced). (b) The AoS structs are enormous
(`Npc` 84 fields with an inline `[i32;256]`, `Player` 239 fields), so a SoA split
touches every field access across `npc.rs` (17 k), `world_sim/npcs.rs` (9 k),
`projectile.rs` (17 k) and `render/entities.rs` (2.7 k) — an L-sized refactor with a
pixel-identity risk at every step. (c) The RNG is per-consumer-flat (`App.npc_rng`,
`item_rng`, `craft_rng`, `clock_rng`, `ambience_rng`, `growth_rng` + the `FastRandom`
in `tr-core`), so parallel archetypes need one stream *per archetype* to keep the
deterministic draw order.

**What is already parallel-safe and reusable:** `tr-core/src/framing.rs` (band
parallel, identical output), `tr-core/src/lighting/{mod,blur}.rs` (seeding +
horizontal sweeps, persistent Rayon pool), `tr-app/src/assets.rs` (parallel PNG
decode). What is *not* parallel-safe: everything under `tr-app/src/world_sim/` — every
entry point takes `&mut App`, and `App` is 257 fields wide.

## Tests

Architecture-specific coverage (no cargo run performed — static inspection only):

- `tr-app/src/world_sim/step.rs::tests::tick_runs_at_most_max_steps_per_frame` — the
  shared accumulator's cap/clamp/reset semantics.
- `tr-core/src/rng.rs` — the unified Xorshift64 core replays the pre-refactor streams
  verbatim (the REFACTOR-rng proof).
- `tr-core/src/binio.rs` — 3 unit tests for the unified LE cursor.
- `tr-core/src/worldgen/world_hash.rs` — the four pinned FNV-1a state hashes (the
  worldgen byte-proof).
- `tr-core/src/worldgen/pipeline.rs::the_vanilla_chain_declares_106_classic_passes` —
  the pass-model contract.
- `tr-app/src/npc.rs` — `slime_variants_match_the_setdefaults_lines` (L13197),
  `spawned_npcs_carry_their_debuff_immunity_table` (L13181);
  `tr-app/src/npc_buffs.rs` — `dw_3_5_immune_kind_rejects_buffs_and_derives_30_375`.
- `tr-app/src/projectile.rs` — `spawn_takes_the_first_free_slot_and_recycles_the_dead`
  (L10004), `full_pool_recycles_the_lowest_time_left` (L10018),
  `time_left_expires_the_slot` (L10198), `extra_updates_runs_the_ai_twice_per_tick`
  (L10224), `test_dw_1_1_every_row_carries_the_set_defaults_reset` (L14069),
  `test_dw_1_2_an_unported_kind_spawns_nothing` (L14190).
- `tr-app/src/world_sim/npcs.rs` — `the_worm_chain_assembles_and_trails`,
  `the_destroyer_chain_assembles_same_kind`, `the_pillar_worms_round_trip_their_rows`,
  `the_crawltipede_assembles_its_thirty_segments`.
- `tr-app/src/itemdata.rs::tests::test_DW_1_2_negative_defaults_and_on_created` (L7498),
  `tr-app/src/registry.rs::tests::test_dw_2_2_net_defaults_and_on_created` (L385),
  `tr-app/src/prefix.rs::tests::tables_are_sorted_and_prefix_lists_verbatim` (L605).
- `tr-core/src/scene.rs` — 58 tests over the filter board and parameter functions.
- `tr-app/src/ui/pointer.rs` — 12 tests (`dw_1_1..dw_1_3`) over the pointer machine.
- Coverage gap: **no test asserts the fixed-step order**, the App/World field
  partitioning, or the absence of cross-module writes — the three things Stage 2
  changes.

## Deviations & risks

| # | Claim / state | Verdict | Severity |
| --- | --- | --- | --- |
| 1 | AGENTS.md: "**The god-files are gone.** Extend the matching module; never grow a new god-file." | **FALSE as stated** — 8 hand-written files > 5 000 lines (`projectile.rs` 17 411, `npc.rs` 17 134, `player.rs` 10 098, `world_sim/npcs.rs` 9 057, `world_sim/editing.rs` 7 738, `edit.rs` 6 791, `growth.rs` 5 305, `smart_cursor.rs` 5 197). True only for the bootstrap (`main.rs` 163). | **structural** (this file *is* the refactor's undo bill) |
| 2 | AGENTS.md: "App state (118 fields)" | Stale — `App` has **257** direct fields (`app/mod.rs` L82). | structural |
| 3 | ARCHITECTURE.md: `app/` ~5.5 k lines, `render/` ~4.5 k, `ui/` ~15.5 k, `main.rs` 86 lines | Stale — actual **18 371 / 11 301 / 24 424 / 163**. The map is directionally right, the sizes are ~2× low. | cosmetic (docs) |
| 4 | ARCHITECTURE.md: "the graph is rebuilt (graphify: 3 440 nodes / 7 933 edges Rust-side)" | Stale — the current Rust graph reports **13 769 nodes**. | cosmetic (docs) |
| 5 | ARCHITECTURE.md/ANALYSIS: "the 2026-11-01 refactor split the two god files" | Partly true: `main.rs` and `menu.rs` were split; the *entity* god files (NPC/Projectile/Player/WorldGen) were never split — they were re-homed into `tr-app`/`tr-core` leaves. | structural |
| 6 | ARCHITECTURE.md "C# seams" table (3 seams: `FrontScreenBehaviours`, `LightEngine`, `PassPrologue`) | Accurate, but the seams cover *UI and rendering* only — there is **no seam at the entity/sim boundary**, which is precisely where Stage 2 needs one. | structural |
| 7 | ROADMAP "Entities (M4) — data-oriented plan" (SoA by archetype, spatial hash, per-thread output buffers, no shared mutable state) | Aspiration only: **no** spatial hash, **no** SoA, **no** event buffers, and the sim is `&mut App` end-to-end. The plan is not started. | structural |
| 8 | ROADMAP §Optimisation "C1/C3 parallel light, C4 framing, C5 decode: done" | Verified — the only parallelism in the tree is `tr-core/{framing,lighting/{mod,blur}}.rs` + `tr-app/assets.rs`. | none (accurate) |
| 9 | ROADMAP perf "E2 SoA/chunked tile storage — planned" | Still "planned"; `Tile` stays 12 B row-major (`tr-core/src/tile.rs`). Consistent with the Stage-2 plan but note the ROADMAP's own conclusion that it yields no real-zoom gain over D3 + the world cache. | none (consistent) |
| 10 | `World` holds runtime event flags (`downed_*`, `invasion_*`, `tower_*`) | Not in any doc; makes `tr-core` mutable during a tick and blocks the read-only phase A. The C# keeps these on `Main`. | structural |
| 11 | `App.npcs` index instability vs `ProjectilePool::local_immunity` | Documented in-code ("stable between death compactions"), but the `Vec` free list (no `GetAvailableNPCSlot` protocol, research 05) means a compaction *does* shift indices. A spatial hash keyed on `Vec` index would inherit the bug. | behavioural / determinism |
| 12 | `World.extras: HashMap<u32,u8>` | `HashMap` iteration order is nondeterministic → a determinism hazard for any future phase-A pass over it. | determinism |
| 13 | `tr-core::scene::move_towards` vs `tr-app::ambient_sound::move_towards`; `hsl_to_rgb` twice | Duplicated helpers (house-style violation: "extract a shared helper or trait when a second copy of a mechanic appears"). | cosmetic |
| 14 | `tr-core::smart_cursor::collision_empty_tile` vs `tr-app::collision.rs` | PORT_MAP already records the move as pending; two homes for `Collision.EmptyTile`. | behavioural (latent divergence) |
| 15 | `tr-app/src/main.rs` module list (70 `mod` lines) vs the file's own doc | `main.rs` is 163 lines of bootstrap but still declares 70 modules — cosmetic only; the bootstrap body is genuinely thin. | cosmetic |

## Task seeds

| # | Task | Size | Depends on |
| --- | --- | --- | --- |
| 1 | **Split `World` into `WorldTiles` (immutable geometry) and `WorldState` (the ~50 runtime/event flags)**; move the event flags to the app side or into a `tr-core::state` module that the sim owns exclusively. Unblocks Stage-2 phase A. | **L** | — |
| 2 | **Extract the fixed-step order out of `app/frame.rs` into a data-driven tick plan** (`[(system, phase)]`), fix the three order inversions research 01 found, and add a test that pins the C# order (`Main.DoUpdateInWorld_Inner` `Main.cs:17749-17821`). Unblocks phase B. | **M** | 1 |
| 3 | **Introduce a typed event buffer** (`enum SimEvent { SpawnNpc, SpawnProjectile, Damage, DropItem, BreakTile, Sound, NetSync }`) with per-thread buffers + a deterministic merge (thread index, then sequence), then migrate `Npc.wants_sphere`, `Wiring::take_trap_requests` and `Player.pending_*` onto it. | **L** | 2 |
| 4 | **Build the spatial hash** over stable entity ids (introduce a monotonic `entity_id` so `Vec` compaction cannot move an entity), with a per-tick rebuild and a `Vec`→`HashMap<cell, Vec<id>>` layout; replace the 15 linear NPC scans in `projectile.rs`/`npc.rs`. | **L** | 3 |
| 5 | **Split `tr-app/src/npc.rs` (17 134) into `npc/{entity,stats,kinds,ai,buffs}.rs`**, mirroring the C#'s own split (`NPC.cs` = entity + `SetDefaults` + `AI` + loot + net). Mechanical move first, behaviour frozen. | **L** | — |
| 6 | **Split `tr-app/src/projectile.rs` (17 411) into `projectile/{entity,data,ai,damage}.rs`** — the `proj_data` table (L1046–2400) and the AI match (L8617–9026) are separable from the pool. | **L** | — |
| 7 | **Split `tr-app/src/player.rs` (10 098) into `player/{entity,physics,buffs,inventory,draw_state}.rs`**; `Player` has 239 fields, so the struct itself is the god object. | **L** | — |
| 8 | **Split `tr-core/src/edit.rs` (6 791) into `edit/{kill,place,actuation,objects}.rs`** and merge the app-side `world_sim/editing.rs` (7 738) placement/break wrappers onto the core's seam instead of duplicating it. | **M** | 5 |
| 9 | **Port the `aiStyle` dispatch** (research 05: 74 of 128 styles unreferenced) and introduce an `ai_style: u8` field on `Npc`, so "SoA by archetype" has a grouping key. | **L** | 5 |
| 10 | **Re-home the duplicated helpers**: `hsl_to_rgb`, `move_towards`, `update_rain` → one owner each in `tr-core`; `collision_empty_tile` → the app-side `collision.rs`. | **S** | — |
| 11 | **Move the read-only `scene_scan` scan from `tr-app/world_sim/scene_scan.rs` into `tr-core::scene`** so the read-only half of the world query lives in the read-only crate (mirrors `tr-core::framing`'s band-parallel split). | **M** | 1 |
| 12 | **Replace `World.extras: HashMap<u32,u8>` with an ordered structure** (or forbid iteration) to remove the nondeterministic-iteration hazard. | **S** | 1 |
| 13 | **Refresh `docs/ARCHITECTURE.md` + `AGENTS.md`**: the crate line counts (2× low), the `App` field count (118 → 257), the `main.rs` size (86 → 163), the graph node count (3 440 → 13 769), and the "god-files are gone" sentence (replace with the measured table above). | **S** | 5, 6, 7 |
| 14 | **Add an architecture lint**: fail the build when a hand-written file exceeds 5 000 lines or when a `pub fn` name exists in both `tr-core` and `tr-app` (the mechanical duplicate detector used here, 15 hits today). | **S** | — |
