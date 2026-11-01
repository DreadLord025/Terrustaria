# AGENTS.md — working rules for this repository

## ⚠️ READ THIS FIRST — Multi-agent coordination protocol

**Before starting ANY work, you MUST read `docs/TASK_STATUS.md` to check:**
- What tasks are currently assigned/in progress
- What tasks are completed (so you don't duplicate work)
- The current project state and recent changes

**All agents must follow this protocol:**
1. Read `agents.md`, `docs/TASK_STATUS.md`, and relevant implementation docs
2. Check `docs/TASK_STATUS.md` for available tasks and current assignments
3. When taking a task: mark it as "in_progress" in `docs/TASK_STATUS.md` WITH YOUR NAME/ID
4. Update the task when complete: mark as "done", add commit reference
5. Never work on the same task as another agent — avoid conflicts

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
   Name it after the original (`Liquid.Update`, `DrawBasicTile`, `AI_003_Fighters`, …)
   and reference it in a one-line comment when the mapping is not obvious.
4. Verify the port against the source: unit-test the maths, compare screenshots with the
   original assets, compare an original `.wld` where possible.
5. Only after steps 1–4 may you optimise. Optimisations must produce identical output
   (pixel-identical screenshots, identical state) and be measured with `TR_BENCH` /
   `TR_STATS`.

If you are unsure about a behaviour, **search the C# source again**; never guess, never
invent a plausible-looking shortcut. If something is deliberately approximate, write it
down in `docs/ANALYSIS.md` and `docs/ROADMAP.md`.

## Knowledge graph (graphify) — use it before grepping

Three pre-built AST graphs (code-only, zero token cost) exist so you can find the exact
C# to port and the exact Rust to extend, instead of grepping raw files:

| Graph | File | Scope | Size |
| --- | --- | --- | --- |
| **C# game (source of truth)** | `decompiled/graphify-out/graph.json` | `decompiled/` (Terraria; ~40 BCrypt/csproj noise nodes) | 20 871 nodes · 68 495 edges · 447 communities |
| **C# library** | `decompiled-libs/graphify-out/graph.json` | `decompiled-libs/` (ReLogic: Content, IO, OS, Utilities, RGB, Threading) | 1 992 nodes · 3 659 edges · 102 communities |
| **Rust (target)** | `graphify-out/graph.json` | `crates/` | 2 101 nodes · 4 518 edges |
| **Merged (all three)** | `graphify-out/merged-graph.json` | Terraria + ReLogic + Rust | 24 928 nodes · 76 909 edges |

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
  every input that affects its pixels.

## Verification before committing

- `cargo test --workspace` must pass.
- For visual work, take deterministic screenshots (`TR_SHOT`, `TR_CAM`, `TR_DAY`,
  `TR_NOPLAYER`, `TR_DEBUG=0`) and compare with `tools/pixel_diff.py`. Check zoom 1.0,
  1.5 and 2.0 when background/parallax changes.
- For performance work, A/B against the previous binary with `TR_BENCH=600` and
  `TR_STATS=1`; include before/after numbers.
- Update `README.md`, `docs/ARCHITECTURE.md`, `docs/ANALYSIS.md` and `docs/ROADMAP.md`
  when a feature lands. Commit one roadmap item per commit with an imperative message.

## Housekeeping

- Do not run `cargo fmt` on the generated `crates/tr-core/src/tiledata/` data files
  (`sets.rs`, `emissive.rs`, `merge.rs`; the hand-written `mod.rs` is fine) or `wld.rs`;
  use `rustfmt` on the touched files only.
- `assets/test/`, `target/`, `decompiled/` and `.qoder_tmp/` are local.
- Prefer extending the existing systems (world model, `framing`, `lighting`, `LiquidSystem`,
  `Rain`, `Projectile`, `Npc`) over adding parallel implementations.
