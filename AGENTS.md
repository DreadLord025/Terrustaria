# AGENTS.md — working rules for this repository

## ⚠️ READ THIS FIRST — Branch-per-task workflow

**Tasks are taken from `docs/ROADMAP.md` → section "Task catalog"** (or from a direct
user request). Coordination lives in **git branches**: the branch list *is* the live
record of who is working on what.

**Every task follows this loop:**

1. **Sync.** `git checkout main && git pull` (if a remote is configured) — start from
   the freshest main.
2. **Branch.** `git checkout -b <branch-name>` from main. Branch name = the task, in
   slug form: `M1.4-BG-11-cloud-parallax`, `M2-DRAW-1-player-layers`,
   `refactor-app-env-flags`, `fix-crafting-scrollbar`. One branch = one task; do not
   stack unrelated work.
3. **Work.** Commit early and often on the branch with imperative messages. Keep the
   rules below (golden rule, tests green, no fmt noise). The rule "never work on the
   same task as another agent" still applies — pick a catalog task nobody's branch
   covers (check `git branch -a` first).
4. **Re-sync before merging.** Bring the branch up to date with main
   (`git merge main` inside the branch, or rebase if you prefer linear history) and
   resolve everything there, not in the final merge.
5. **Verify the merge result.** `cargo test --workspace -- --skip
   migrates_version_2_kinds_to_item_ids` must be green **on the branch after the
   sync** (see the exit-code gotcha below). For visual work also re-take the
   deterministic screenshots on the merged state.
6. **Merge into main.** `git checkout main && git merge <branch-name>`. Conflicts
   must be resolved **thoughtfully**: a conflict means two tasks touched the same
   system — re-read the C# source for that system and decide what the *port* needs,
   rather than picking "mine" or "theirs" blindly. Never take `--ours`/`--theirs`
   wholesale; never use `git merge -X ours/theirs`. If a conflict is semantic (two
   agents changed the same behaviour differently), stop and reconcile the intent
   first — the merged result must still be 1:1 with the source.
7. **Clean up.** Delete the branch after the merge (`git branch -d`). The task is
   recorded by its merge commit, its branch name and its docs updates.

**Practical limits:** keep branches short-lived (merge within a day of work) — long-
lived branches are where painful conflicts come from. Two or three parallel branches
are comfortable; more invites overlap. Never push to a remote unless the user asked.

---

Terraria-Rust is a **1:1 reimplementation of Terraria 1.4.5.8** in Rust: same behaviour,
same constants, same pixels, different engine. These rules are mandatory for every change.

## The golden rule: port the C# first

Before writing or changing **any** game logic, function or constant:

1. Find the matching implementation in the decompiled sources under `decompiled/`
   (`Terraria.sln` namespaces: `Terraria`, `Terraria.GameContent.*`, `Terraria.ID`, …).
   Data tables (tile ids, frame counts, merge sets, colours) also come from there.
2. Read the whole relevant function and its callers. Note constants, update order,
   rounding (`Math.Round` is banker's rounding), RNG order, and special cases.
3. Port it **faithfully** to Rust first — same structure, same maths, same order.
   **Naming (updated 2026-11-01):** prefer a **readable, descriptive Rust name** over
   copying the C# identifier (`get_rock_transition_point`, `rain_frames`,
   `fill_terrain`), **but** keep the mapping discoverable: every ported block carries
   a one-line citation comment (`// Main.cs:53100-53343`, `// MapHelper.cs:2083-3005`)
   and non-obvious mappings get an explicit reference. Never drop the citations —
   they are how we re-verify the port against the source.
4. Verify the port against the source: unit-test the maths, compare screenshots with
   the original assets, compare an original `.wld` where possible.
5. Only after steps 1–4 may you optimise. Optimisations must produce identical output
   (pixel-identical screenshots, identical state) and be measured with `TR_BENCH` /
   `TR_STATS`. The same applies to refactors: **behaviour is frozen** — RNG streams,
   frame-cache keys and the fixed-step order must not change (see
   `docs/ANALYSIS.md` "Architecture refactor").

If you are unsure about a behaviour, **search the C# source again**; never guess, never
invent a plausible-looking shortcut. If something is deliberately approximate, write it
down in `docs/ANALYSIS.md` (and `docs/ROADMAP.md` if it is a roadmap item).

## Architecture — where things live (2026-11-01 split)

The god-files are gone. **Extend the matching module; never grow a new god-file.**
The full map with per-file line counts lives in `docs/ARCHITECTURE.md`; the short version:

```
crates/tr-app/src/
  main.rs            bootstrap only (~90 lines: world load, atlas, event loop run)
  app/               App state (118 fields), event loop (loop.rs), frame orchestration
                     (frame.rs), front-end flow (flow.rs: FrontScreen enum +
                     FrontScreenBehaviours trait ≈ C# UIState; next_flow_state table),
                     menu/inv/net clicks (clicks.rs), session, settings_ui, netglue,
                     saveglue
  render/            world rendering: sky, bg passes (dirt/rock/magma/underworld),
                     tiles, liquids, entities (player/npcs), foliage, debug overlay
  world_sim/         fixed-step simulation: step.rs (FixedStepper) + player/npcs/items/
                     projectiles/editing/clock/liquids/weather/camera
  menu/              legacy immediate-mode menu: mod.rs (machinery), pages.rs (one
                     rows-builder per Page), actions.rs (Action/Toggle/Act)
  ui/                HUD panels (slot, inventory, crafting, minimap, bars, tooltip,
                     chrome) + front-end screens (char_select, char_creation,
                     world_select, world_creation, keyboard) + shared machinery
                     (uistate = the UIElement.cs:306-322 layout pass, cards, sliders,
                     text, panel, color, mod.rs helpers)
  item.rs  biome.rs  inventory slot type / biome + water-style + bg cross-fade
  assets.rs player.rs settings.rs map.rs npc.rs …
crates/tr-core/src/
  tiledata/          GENERATED by tools/GenTileData — never hand-edit; regenerate
  map/               world map: mod.rs (MapTile/reveal), colors.rs, option.rs (the
                     MapHelper port), file.rs (.map IO)
  tile_light/        scanner.rs (apply_tile_light, split by theme — keep the arm
                     order), palette.rs (FastRandom — exact C# stream), wall/surface/hell
  lighting/          mod.rs (LightMap + LightEngine trait ≈ ILightingEngine), params, blur
  waterfall/ worldgen/ (passes) localization/ binio.rs (unified le-readers)
  framing.rs liquid.rs liquid_draw.rs wind.rs drawdata.rs torch.rs edit.rs tree.rs
  sky.rs save.rs wld.rs world.rs tile.rs
```

**C#-style seams already in place — use them, don't re-invent:**
`FrontScreenBehaviours` (draw/click/key per screen ≈ `UIState.DrawSelf`/`Click_*`),
`LightEngine` (≈ `ILightingEngine`), `FixedStepper` (the 60 Hz accumulator), the
shared `ui` helpers (`in_rect`, `hover_pair`, `push_whole`, `text::border_string`,
`ui::cards`). When a new system has peers with similar mechanics, extract a trait or
a shared helper instead of copying — that is the house style since the 2026-11-01
refactor.

## Knowledge graph (graphify) — use it before grepping

Three pre-built AST graphs (code-only, zero token cost) exist so you can find the exact
C# to port and the exact Rust to extend, instead of grepping raw files:

| Graph | File | Scope |
| --- | --- | --- |
| **C# game (source of truth)** | `decompiled/graphify-out/graph.json` | `decompiled/` (Terraria) |
| **C# library** | `decompiled-libs/graphify-out/graph.json` | `decompiled-libs/` (ReLogic) |
| **Rust (target)** | `graphify-out/graph.json` | `crates/` (~3 440 nodes) |
| **Merged (all three)** | `graphify-out/merged-graph.json` | Terraria + ReLogic + Rust |

**Rule:** for a codebase question ("where is X", "what calls Y", "what does Z touch"),
run a `graphify` query first. It returns a scoped subgraph with **file + line + community**,
which is far cheaper than grep and tells you what to read next. Grep only to confirm.

### Recipes (run from the repo root unless noted)

```powershell
# 1. Symbol + its callers/callees (use path::symbol to disambiguate; copy the id from
#    the "Ambiguous" output if needed). --all shows every connection, not just top 20.
cd decompiled
graphify explain "Terraria/Liquid.cs::Liquid"
graphify explain "terraria_liquid_terraria_liquid" --all      # by node id

# 2. Who calls a function / class — the port dependency order (reverse `calls` walk).
graphify affected "terraria_liquid_terraria_liquid" --relation calls --depth 2

# 3. Related code for a concept, filtered to real calls (not `contains`/`imports` noise).
graphify query "how is liquid settlement done" --context calls

# 4. Shortest call path between two symbols.
graphify path "Terraria/WorldGen.cs::SquareTileFrame" "Terraria/Liquid.cs::Liquid" --context calls

# 5. Architectural hubs (read these first when starting a system).
graphify god-nodes --top 20
```

**Gotchas**

- Label lookup is ambiguous across files. Prefer `path::symbol` (e.g.
  `Terraria/Liquid.cs::Liquid`) or the full node id (`terraria_liquid_terraria_liquid`).
- `calls` edges **are directional** in the JSON (the build restores `_src`/`_tgt`), so
  `affected … --relation calls` and `path … --context calls` give real caller/callee order.
  Without `--context`, shortest paths wander through `contains`/`imports` and are useless.
- `explain` shows only the 20 most connected neighbours by default — add `--all`.

### Avoid duplicating an existing implementation

Before creating a new module/type, check **once** that it is not already ported:

```powershell
graphify query "<concept or symbol>" --graph graphify-out/merged-graph.json
graphify explain "<RustSymbol>"        --graph graphify-out/merged-graph.json
```

`docs/PORT_MAP.md` is the C#→Rust cross-language map that the graphs cannot build
themselves — update it when you start or finish a system.

> In the merged graph every id is prefixed with its source — `decompiled::` (Terraria),
> `decompiled-libs::` (ReLogic), `repo::` (Rust) — and some labels (e.g. `ContentManager`)
> exist in more than one source, so always disambiguate with `path::symbol` or the full id.

### Keeping the graphs fresh

```powershell
graphify update .                       # refresh the Rust graph after touching crates/
# (C# side) /graphify decompiled --update  and  /graphify decompiled-libs --update
#           — only needed when the decompiled source changes
graphify merge-graphs decompiled/graphify-out/graph.json `
        decompiled-libs/graphify-out/graph.json graphify-out/graph.json `
        --out graphify-out/merged-graph.json   # re-merge after any side changes
```

Run the update/merge **on main after merging a branch** (or on your branch before the
merge if the change is code-shape relevant); avoid two agents racing on
`graphify-out/` at the same time.

## Assets

- `decompiled/`, `decompiled-libs/` and `assets/Content/` (the game's own `.xnb`
  content) are derived from the user's own retail copy.

## Rendering invariants

- Tiles are 16 px on an 18 px sheet grid (2 px padding); tree trunks are 20×20 on a 22 px
  grid; walls use 36 px cells with an 8 px inset. Keep source rects on those grids.
- The renderer draws one atlas through one instanced sprite pipeline. Sprites carry an
  optional rotation; tint is pre-linearised in WGSL to match XNA byte multiplies.
- Lighting uses the original `TileLightScanner` seeds and `LightMap` blur; the 9-slice
  block path (`GetColor9Slice`) is what makes Color/White mode smooth.
- Optimisation policy (Milestone 1.5): the in-game zoom range stays 1.0–2.0; **no LOD,
  no reduced detail, no skipped frames of world state**. Any cached range must key on
  every input that affects its pixels (the G6 `WorldFrameKey` — never widen or narrow
  what it covers without an A/B).

## Verification before merging

- `cargo test --workspace -- --skip migrates_version_2_kinds_to_item_ids` must pass —
  **`cargo` may exit 1 on success in this project; judge only by the `test result: ok`
  lines.** (`persist::tests::migrates_version_2_kinds_to_item_ids` is the one known
  broken test; the `--skip` is permanent until it is fixed.)
- For visual work, take deterministic screenshots (`TR_SHOT`, `TR_CAM`, `TR_DAY`,
  `TR_NOPLAYER`, `TR_NO_BG`, `TR_UI=<screen>`, `TR_DEBUG=0`) and compare with
  `tools/pixel_diff.py`; two identical runs must be SHA256-equal. Check zoom 1.0,
  1.5 and 2.0 when background/parallax changes.
- For performance work, A/B against the previous binary with `TR_BENCH=600` and
  `TR_STATS=1`; include before/after numbers.
- Update the docs when a feature lands: `docs/ARCHITECTURE.md` (if modules moved),
  `docs/UI.md`, `docs/ANALYSIS.md` (every deliberate deviation), `docs/PORT_MAP.md`,
  `docs/ROADMAP.md`. `README.md` is player-facing — update it only for visible
  features, not internal work. Commit one task per branch; imperative commit messages.
- Determinism is a hard invariant: RNG streams (`FastRandom`, worldgen xorshift), the
  G6 `WorldFrameKey` contents and the fixed-step order must not change. If your change
  touches them, prove byte-equality of the affected outputs (see the worldgen FNV
  proof and `deterministic_for_seed` for the pattern).

## Housekeeping

- Do not run `cargo fmt` on the generated `crates/tr-core/src/tiledata/` data files
  (`sets.rs`, `emissive.rs`, `merge.rs`; the hand-written `mod.rs` is fine) or `wld.rs`;
  use `rustfmt` on the touched files only. Never hand-edit `tiledata/` — fix
  `tools/GenTileData` and regenerate (it matches the current decompilation since
  2026-11-01; a regen must produce an empty diff).
- `assets/test/`, `target/`, `decompiled/` and `.qoder_tmp/` are local.
- Prefer extending the existing modules (see the architecture map) over adding parallel
  implementations; extract a shared helper or trait when a second copy of a mechanic
  appears.
