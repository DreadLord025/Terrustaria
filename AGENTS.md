# AGENTS.md — working rules for this repository

## ⚠️ READ THIS FIRST — Branch-per-task workflow

**Tasks are taken from `docs/ROADMAP.md` → section "Task catalog"** (or from a direct
user request). Coordination lives in **git branches**: the branch list *is* the live
record of who is working on what.

**Every task follows this loop:**

1. **Sync first — always, in every scenario.** Before ANY other action
   (taking a task, resuming, finishing, merging, after a pull): bring the code
   and the graph to the same state. **Do this from your own worktree, never by
   checking out in the primary checkout:**
   ```powershell
   git fetch origin --prune               # refresh the branch list + origin/main
   git merge --ff-only origin/main        # inside YOUR worktree's branch
   graphify update .                      # rebuild the Rust graph from crates/
   graphify merge-graphs decompiled/graphify-out/graph.json `
           decompiled-libs/graphify-out/graph.json graphify-out/graph.json `
           --out graphify-out/merged-graph.json
   ```
   `graphify-out/` is **gitignored** — the graph never travels with git, so a stale
   local graph after a pull/merge gives queries answered from pre-refactor locations.
   When the update reports no changes, it is a no-op and cheap.
2. **Branch + worktree, from main.** `git worktree add ../<task-dir> -b <branch-name>
   origin/main` (the primary checkout is NEVER used for task work). Branch name = the
   task, in slug form: `M1.4-BG-11-cloud-parallax`, `M2-DRAW-1-player-layers`,
   `refactor-app-env-flags`, `fix-crafting-scrollbar`. One branch = one task; do not
   stack unrelated work. A FRESH worktree lacks the gitignored trees — create the
   symlinks before the first build or the **localization** tests fail with false
   "unknown key" errors: the language JSON lives in the **`decompiled/`** junction
   (there is none under `assets/`, which supplies fonts and content):
   ```powershell
   cd ..\{task-dir}
   cmd /c mklink /J assets ..\Terraria-Rust\assets
   cmd /c mklink /J decompiled ..\Terraria-Rust\decompiled
   cmd /c mklink /J decompiled-libs ..\Terraria-Rust\decompiled-libs
   ```
3. **Work.** Commit early and often on the branch with imperative messages. Keep the
   rules below (golden rule, tests green, no fmt noise). The rule "never work on the
   same task as another agent" still applies — pick a catalog task nobody's branch
   covers (check `git branch -a` first). If your change moves or renames code (a
   refactor-shaped task), run `graphify update .` again before committing so the
   branch's last state is graphed.
4. **Re-sync before merging.** Bring the branch up to date with the remote main
   (`git merge origin/main` inside the branch, or rebase if you prefer linear
   history) and resolve everything there, not in the final merge. Then repeat
   step 1's graphify update on the synced branch.
5. **Verify the merge result.** `cargo test --workspace -- --skip
   migrates_version_2_kinds_to_item_ids` must be green **on the branch after the
   sync** (see the exit-code gotcha below). For visual work also re-take the
   deterministic screenshots on the merged state.
6. **Merge into main — by ref update, never by checkout.** The primary checkout
   stays on `main`, clean, and unoccupied. From YOUR worktree, after step 4's sync:
   ```powershell
   # set your identity first if needed (git config user.name / user.email)
   $m = git commit-tree "$(git rev-parse HEAD)^{tree}" -p origin/main -p HEAD `
        -m "Merge branch '<branch-name>'
   <one-line scope>"
   # the ref race: if another agent merged in the meantime the update-ref
   # (or the push) fails — re-fetch, re-sync your branch onto the new main
   # (step 4) and retry. Never force main.
   git update-ref refs/heads/main $m $(git rev-parse main)
   git push origin main
   ```
   **Then verify the merge actually landed** — a successful push is not proof that
   your work survives. Several agents rewrite `main` by ref update, and a later
   merge made from a tree that predates yours silently reverts it (no merge base is
   recorded for a `commit-tree` merge, so git cannot even show it as a conflict).
   This has already eaten BUG-21 twice and M4-NPC-3m once. So, immediately after
   the push:
   ```powershell
   git fetch origin
   git merge-base --is-ancestor $m origin/main   # exit 0 = your merge is in
   # and check a SYMBOL from your change, not just the commit:
   git show origin/main:crates/tr-app/src/app/mod.rs | Select-String 'enter_loaded_world'
   ```
   If either check fails, your merge was overwritten: re-fetch, rebuild the merge on
   top of the new `origin/main` (`git merge origin/main` in your worktree, then the
   `commit-tree` above again) and push again. Re-run these two checks once more after
   `graphify` step 7 — and re-check them if you are still around when another agent
   pushes main.

   **The tree-wide canary**: `anchors.txt` at the repo root lists one row per
   landed feature, `path|marker|feature name`. Two checks read it:
   * `crates/tr-core/build.rs` — **the build fails** when a marker disappears
     (escape hatch: `TR_SKIP_ANCHORS=1`), so a lossy merge cannot ship, and
   * `cargo test -p tr-app --test main_anchors` — the same rows with a readable
     failure inside the test run.

   The failure names the lost feature, e.g.
   `M5-DUNGEON — the DungeonGlobalWallVariants pass: ... no longer contains "fn wall_variants"`.
   That means *some other agent's* merge dropped it — it has happened to BUG-21
   twice and to the whole `dungeon_features.rs` twice (`9f89c99`, `cc6298a`).
   Do not "fix" it by deleting rows from `anchors.txt`.

   Recovery, without disturbing whoever is working in the primary checkout:
   ```powershell
   git log --oneline --all -- <dropped-file>            # find the commit that had it
   git worktree add ../TR-RELAND -b RELAND origin/main
   cd ../TR-RELAND                                     # (+ the two mklink junctions)
   git cherry-pick <original-commit>                   # or -m 1 <merge-commit>
   cargo test --workspace -- --skip migrates_version_2_kinds_to_item_ids
   # push through the merge procedure above, then re-run the canary
   ```
   Add a row to `anchors.txt` for every feature you land (one line: path,
   marker, name).
   Conflicts (when the merge is done as `git merge origin/main` inside the branch)
   must be resolved **thoughtfully**: a conflict means two tasks touched the same
   system — re-read the C# source for that system and decide what the *port* needs,
   rather than picking "mine" or "theirs" blindly. Never take `--ours`/`--theirs`
   wholesale; never use `git merge -X ours/theirs`. If a conflict is semantic (two
   agents changed the same behaviour differently), stop and reconcile the intent
   first — the merged result must still be 1:1 with the source.
   **Before you land anything, read `docs/MAIN-RULES.md`** — the short version of
   steps 1-6 above plus the four documented work-losses, the banned merge forms
   (`commit-tree`, `read-tree`, `-s ours`, `-X ours/theirs`, force-push) and a
   copy-paste prompt for an AI agent.
7. **Merge graphify, then clean up.** Immediately after the merge, on main, re-run
   step 1's graphify update + merge-graphs so main's graph includes the branch's
   code — this is the step that "pulls not only the code but the graph". Then delete
   the branch (`git branch -d`). The task is recorded by its merge commit, its
   branch name and its docs updates.

**Practical limits:** keep branches short-lived (merge within a day of work) — long-
lived branches are where painful conflicts come from. Two or three parallel branches
are comfortable; more invites overlap. Two people work from this catalog (2026-11-01
agreement): a branch with the task's id **is** the claim — push the task branch to
`origin` as soon as you create it so the other collaborator sees it (run
`git fetch --prune` before taking a row). Merge into `main` only after verification;
push `main` by agreement, never force-push.

### Branch & worktree lifecycle — after a merge, clean up (2026-11-02 rules)

A merged branch and its worktree are garbage. Leaving them behind makes the branch
list lie about who is working on what. After step 7 of the loop, always:

1. **Verify before deleting a branch.** The tip must be fully in main:
   `git log main..<branch> --oneline` shows 0 commits (or `git branch --merged main`
   lists the branch). Never delete a branch with unique commits — merge it first.
2. **Delete the branch both locally and on the remote:**
   `git branch -d <branch>` + `git push origin --delete <branch>`, then
   `git fetch --prune` (delete one branch per command — a batch fails whole if any
   ref is already gone).
3. **Worktrees are temporary, never permanent.** Each worktree is created for one
   task (`git worktree add <path> <branch>` from `origin/main`) and removed as soon
   as its branch is merged: `git worktree remove <path>` (add `--force` only after
   inspecting it), then `git worktree prune`. The removal must run from another
   checkout — you cannot remove the worktree you are standing in. Before removing,
   run `git -C <path> status --porcelain` — if there are uncommitted or untracked
   files, resolve them first: commit what is real, and for an untracked file that
   duplicates a committed one (compare `sha256sum` with the tracked copy) discard it.
   A `crash.log`/log file is junk.
4. **Nobody occupies the primary checkout (2026-11-08).** The primary checkout
   (`Terraria-Rust`) is a shared, always-clean, always-on-`main` repository — it is
   NOT anyone's workspace and no agent checks out a task branch in it, stages, or
   commits there. Task work happens ONLY in per-task worktrees; merges are ref
   updates (step 6). If the primary's `git status --porcelain` is non-empty,
   something bypassed this rule — recover per the 2026-11-08 rules below and NEVER
   commit the foreign state. Before `git worktree add`, `git status --porcelain` in
   the primary must be empty.
5. **End state after a cleanup pass:** `git branch -a` shows only `main` (local +
   `origin/main`) plus branches that are genuinely in flight, `git worktree list`
   shows only the primary checkout (on `main`, pushed), and no `../TR-*` task
   directories or stray scratch files (`*.rs`, `*.patch`, screenshots outside
   `docs/screenshots/`) remain on the Desktop.

### Mandatory safety checks — the 2026-11-08 incidents

Three failures on 2026-11-08 shape these: a **stale index** in the primary checkout
that staged a full revert of merged work (`D ambient_sound.rs`, deleted screenshots,
~1200 lines cut from `npc.rs`/`render/entities.rs` — the index matched the old
`f0c2476d` tree while HEAD had already absorbed later merges); a **lossy merge** that
silently dropped an entire merged task (another branch's merge resolution overwrote
the Golem's regions in the hot files); and task agents checking out inside the shared
primary.

1. **Before every git write, check what you are standing on.** `git status
   --porcelain` non-empty in a checkout you do not own = STOP — never commit,
   reset or "clean" it. Report it and work in your own worktree.
2. **Verify the task diff before pushing.** `git diff origin/main..<branch> --stat`
   must list ONLY your task's files. Unrelated files or deletions of files you never
   touched mean the branch base or the merge is wrong — fix it before pushing.
3. **Verify the merge after every merge.** Run the catalog command
   (`cargo test --workspace -- --skip migrates_version_2_kinds_to_item_ids`) after
   merging `origin/main` into your branch or merging your branch into main. A
   previously-green test that now fails = the auto-resolution is wrong: `git merge
   --abort`, redo the merge, resolve the hot files (`npc.rs`, `render/entities.rs`,
   `world_sim/npcs.rs`, `world.rs`, `app/mod.rs`) by re-reading the C# — a semantic
   conflict there means two tasks touched one system.
4. **Stale-index recovery (primary only).** If the primary's index/worktree drifted
   from HEAD through merges, preserve it BEFORE resetting:
   `git branch backup-stale-index-<date> "$(git commit-tree "$(git write-tree)" -p
   HEAD -m "backup: stale staged index")"` → `git reset --hard HEAD` → confirm the
   stale tree was a subset of HEAD (no unique content) → run the tests → delete the
   backup branch once confirmed. This is a recovery path, not a workflow: with rule 4
   of the lifecycle section it should never be needed again.
5. **Never delete another agent's branch.** Delete only your own, only after
   `git log main..<branch>` is empty, and delete local + remote + prune in that
   order.

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

The refresh is **mandatory at every workflow checkpoint** — see step 1 / step 7 of the
branch-per-task loop above (before taking a task, after a pull, after a merge, when
pausing or finishing work). The commands live there; this section is the rationale:

- `graphify-out/` is gitignored, so the graph is local-only: after any `pull`/`merge`
  the code can be new while the graph is old — queries then answer from stale
  locations (the pre-refactor map, moved functions), which silently misroutes work.
- `graphify update .` is AST-only and cheap (no API cost); the merge-graphs step only
  re-stitches the three graphs.
- **Never run two graphify updates concurrently** (two agents racing on
  `graphify-out/` corrupt the output); refresh on main after merging a branch, or on
  your branch before the merge if the change is code-shape relevant.

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
  touches them, prove byte-equality of the affected outputs (see the frozen-input
  golden test in `worldgen/world_hash.rs` and `generation_is_reproducible_for_a_seed`
  for the pattern).

## Working with docs (find/read/edit efficiently)

The doc set is large; read **targeted** slices, not whole files.

| Doc | Role (source of truth for…) | How to use |
| --- | --- | --- |
| `docs/ROADMAP.md` | **Audited task catalog & status claims** (S1/S2 rows, severities, evidence, verification). Where it disagrees with ATLAS, it wins. | Start here for "what is broken/missing". Grep the task id (`S1-*`) or subsystem; read the row + its cited report, not the whole file. |
| `docs/ATLAS.md` | Legacy per-subsystem atlas & historical task rows (descriptive). | Use for background/legacy context; do not treat its marks as verified truth. |
| `docs/research/90-consolidated-findings.md` | Cross-module register: scoreboard, task seeds, risks, unadmitted claims, priority list. | Fastest overview of audit findings and what to tackle next. |
| `docs/research/NN-*.md`, `40/41/42`, `91/92` | Per-module audits and re-verification passes (evidence, deviations, seeds). | Open the specific report named by a ROADMAP row; prefer over re-deriving claims. |
| `docs/ARCHITECTURE.md` | Where code lives (module map). | "Where do I extend this?" before creating files. |
| `docs/PORT_MAP.md` | C# → Rust symbol/map (cross-language). | Update when you start/finish a system; check before duplicating. |
| `docs/ANALYSIS.md` | Deliberate deviations, invariants, proofs (e.g. determinism notes). | Record any intentional divergence here (and ROADMAP if it is a task). |
| `docs/UI.md`, `docs/BUG_REPORT.md`, `docs/MAIN-RULES.md` | UI/front-end map, known bugs, merge-safety rules. | Consult the one that matches the work; MAIN-RULES is mandatory before landing. |
| `README.md` | Player-facing only. | Update only for visible features. |
| `anchors.txt` | Landed-feature canary (path\|marker\|name). | Add a row for every landed feature; never delete rows to silence failures. |

**Rules of thumb**
- **Find before reading:** `grep`/`Select-String` for the task id, symbol, or C# anchor (`file.cs:line`) to jump to the exact row/section; use `graphify query/explain/path` for codebase questions (see the graphify section) instead of broad greps.
- **Read narrowly:** these files are large (ROADMAP/ATLAS/ANALYSIS). Read the relevant section/line range; follow citations into the named research report or source file rather than scrolling the whole doc.
- **Edit surgically:** make the smallest change that records the fact (status/evidence/deviation). Keep rows consistent (size/severity/evidence/deps/verify). When you land or re-verify work, update `ROADMAP.md` (and the matching research note if adding evidence), `PORT_MAP.md`, and `ANALYSIS.md` as applicable; add an `anchors.txt` row.
- **Don't guess:** if a claim lacks C#/Rust citations, treat it as unverified and re-check against source before acting or marking done. Prefer `docs/research/` evidence over memory.
- **Stay in sync:** after a pull/merge, refresh the graph (step 1) so doc/code navigation reflects current state; never run concurrent graphify updates.


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
