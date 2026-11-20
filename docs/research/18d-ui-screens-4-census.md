# 18d — UIState screens coverage census (W4-f22, verify-only)

> Wave 4 · task/W4-f22-ui-screens-4 · base a725acb0 · researched 2026-11-20 ·
> sources: `decompiled/Terraria/GameContent/UI/States/*` (29 files) +
> `NewCraftingUI.cs`, `crates/tr-app/src/ui/` + `app/flow.rs`, and the pushed
> phase-0 claims of the parallel screens waves:
> `origin/task/W4-d84-ui-screens-1` @ `e0f69499`
> (`docs/research/W4-UI-SCREENS-RANGES.md`),
> `origin/task/W4-d85-ui-screens-2` @ `b0ef93b0` (research/18 split note),
> `origin/task/W4-d86-ui-screens-3` @ `c9855a4f`
> (`docs/research/18c-ui-screens-remainder.md`).
> Code-only mode: no cargo/tests/graphify run; every Rust verdict below was
> re-checked by reading the base tree directly.

## Verdict: verify-only — zero UNCLAIMED missing screens

An exhaustive walk of every `UIState` descendant in 1.4.5.8 (grep
`:\s*UIState\b` over all of `decompiled/`, not just `States/`) finds **28
classes**: 24 concrete files in `States/`, the abstract
`AWorkshopPublishInfoState<T>` + its two closed subclasses, and
`NewCraftingUI` outside `States/`. Ten are PORTED on this base (evidence
below); the other 18 (17 rows in research/18's gap table + the
`AWorkshopPublishInfoState` pair d86 folds into the Workshop family) are each
covered by a pushed wave claim or a standing exclusion:

- d84 (`e0f69499`): Workshop hub/publish/import ×4 + ResourcePack
  selection/info ×2 (rows 1-6).
- d85 (`b0ef93b0`): both debug lists + their row elements, `UIReportsPage`,
  `UIWorldCreationAdvanced`, `UIWorldCreationAdvancedSecretSeedsList`
  (rows 7-12).
- d86 (`c9855a4f`): the remainder — `UIWorldGenDebug` /
  `UITextWrappingTest` scope-outs, `UIManageControls`, the
  `AWorkshopPublishInfoState` pair, the `UIWorldLoad` PARTIAL-row
  re-verification.
- `UICreativePowersMenu`: c24 journey lane, skipped by directive in all
  three waves and here.

Nothing is left UNCLAIMED, so per the task rule this branch lands **no
ports** — only this census. No files outside this doc are touched.

## Claim conflicts recorded for the merger (not resolved here)

The three waves claimed in parallel from the same base and their phase-0
docs disagree; f22 takes no side, only records:

1. **Advanced seeds**: d84's split doc and d85's research/18 note both give
   `UIWorldCreationAdvanced` + `…SecretSeedsList` to **d85**; d86's scope
   table gives them to d84. The d84 doc is the lane's own split record —
   treat d85 as owner unless the waves re-negotiate.
2. **`UIManageControls`**: d86's table says "d85 claim"; d84/d85 give it to
   **d86** ("the rest"). Same ruling: d86 owns it.
3. **Reports + debug lists**: claimed by **both d85** (`b0ef93b0`) and
   **d86** (`c9855a4f`, which announces `ui/reports.rs` + `ui/debug_lists.rs`
   ports). Keep-one per file at merge; both cite the same C#.
4. **ResourcePack screens**: d84 plans ports (`ui/resource_packs.rs`,
   `ui/resource_pack_info.rs` + a `resource_pack.rs` data layer); d86 marks
   them MISSING-with-dependency (no `AssetSourceController`/`ResourcePackList`
   port) and excludes them. If d84 lands the data layer its ports stand;
   otherwise d86's dependency verdict is the fallback record.
5. d86's doc describes its ports and research/18 row fixes in past tense,
   but at `c9855a4f` only the doc exists (`git diff a725acb0..c9855a4f
   --stat` = 1 file). Likewise its claim that the `UIWorldLoad` F5
   `UIWorldGenDebug.Open()` scope-out is "recorded in `world_load.rs`" is
   not yet true on any pushed tree — `world_load.rs` on base has no F5/debug
   mention (checked). That record belongs to d86's port commits.

## Census — every `UIState` descendant vs the port (base a725acb0)

Line counts are **raw file lines** (`(Get-Content).Count`). research/18's
"Lines" column held the class-declaration line numbers, not lengths (d86
noted this too); this table supersedes both older counts.

| C# state (file) | Lines | Rust counterpart | Verdict / owner |
| --- | --- | --- | --- |
| `UICharacterSelect` | 311 | `ui/char_select.rs`, `flow.rs:28` `FrontScreen::CharSelect` | PORTED (verified below) |
| `UICharacterCreation` | 2381 | `ui/char_creation.rs`, `flow.rs:36` `CharCreation(CreationState)` | PORTED |
| `UIWorldSelect` | 347 | `ui/world_select.rs`, `flow.rs:30` `WorldSelect` | PORTED |
| `UIWorldCreation` | 1382 | `ui/world_creation.rs`, `flow.rs:42` `WorldCreation(NewWorldDraft)` | PORTED (base tab; Advanced-seed icon click degrades to plain seed keyboard — documented deviation `world_creation.rs:525-529`; d85 owns the Advanced tab) |
| `UIVirtualKeyboard` | 1087 | `ui/keyboard.rs`, `flow.rs:46` `Keyboard(Box<KeyboardSession>)` | PORTED |
| `UIWorldLoad` | 137 | `ui/world_load.rs` (`WorldLoadState` :134), `flow.rs:55` `WorldLoad(Box<WorldLoadState>)` | PORTED — first-class screen, nothing folded into world_select/save_list; research/18's PARTIAL row is STALE (d86 re-verifies in depth). Deliberate deferrals admitted in ANALYSIS "The loading screen + capture mode (S1-E-17)" (:24792). F5 debug arm absent (scope-out, d86 to record). |
| `UIAchievementsMenu` | 223 | `ui/achievements.rs` (draw seam `app/frame/draw.rs:1837`) | PORTED |
| `UIEmotesMenu` | 656 | `ui/emotes.rs` (wheel: panel + 8-column `EmoteButton` grid) | PORTED |
| `UIBestiaryTest` | 856 | `ui/bestiary.rs` (the file IS the live bestiary despite the `Test` name) | PORTED (residual elements: c37/e23 lanes) |
| `NewCraftingUI` (`UI/`, not `States/`) | — | `ui/crafting.rs` (immediate port, ctor `:300-306` cited) | PORTED |
| `UIWorkshopHub` | 374 | none | MISSING — **d84** (`ui/workshop_hub.rs` planned); standing Workshop exclusion ANALYSIS:23673 is the fallback if d84 descopes |
| `UIWorkshopSelectResourcePackToPublish` | 235 | none | MISSING — **d84** (`ui/workshop_publish.rs`) |
| `UIWorkshopSelectWorldToPublish` | 233 | none | MISSING — **d84** (same module) |
| `UIWorkshopWorldImport` | 251 | none | MISSING — **d84** (same module) |
| `AWorkshopPublishInfoState<T>` (abstract) | 822 | none | MISSING — Workshop publish base; d86 EXCLUDED, d84's publish ports must cover or record it |
| `WorkshopPublishInfoStateForResourcePack` | 46 | none | MISSING — same pair (d86 census row "40+40") |
| `WorkshopPublishInfoStateForWorld` | 46 | none | MISSING — same pair |
| `UIResourcePackSelectionMenu` | 591 | none | MISSING — **d84** port planned vs d86 MISSING-with-dependency (conflict 4 above) |
| `UIResourcePackInfoMenu` | 210 | none | MISSING — same |
| `UIDebugCommandsList` | 165 | none | MISSING — **d85** (d86 co-claims, conflict 3) |
| `UIDebugLoadoutsList` | 146 | none | MISSING — **d85** (d86 co-claims) |
| `UIReportsPage` | 248 | none | MISSING — **d85** (d86 co-claims) |
| `UIWorldCreationAdvanced` | 532 | none | MISSING — **d85** (d86 table misattributes to d84, conflict 1) |
| `UIWorldCreationAdvancedSecretSeedsList` | 459 | none | MISSING — **d85** (same) |
| `UIWorldGenDebug` | 1702 | none | MISSING — **d86** scope-out (dev-only; `UIWorldLoad.cs:50-52` F5 + worldgen debug hook) |
| `UITextWrappingTest` | 336 | none | MISSING — **d86** scope-out (test-only harness) |
| `UIManageControls` | 1061 | none (`input_map.rs` is the bind DATA, header cites `Main.cs:67163-67195`, not the screen) | MISSING — **d86** (conflict 2) |
| `UICreativePowersMenu` | 597 | none (`creative.rs` is the powers DATA, `CreativePowers.cs`, not the screen) | MISSING — **c24** journey lane, skipped by directive |

Non-screen files in `States/` (not gap-table rows):

- `UIGamepadHelper.cs` (367) — a `struct` (`[StructLayout] … public struct
  UIGamepadHelper`, :14-15), NOT a `UIState`; gamepad snap-point helper,
  Stage-1 gamepad exclusion (research/18 deviation 10, §J.3).
- `UISortableElement.cs` (22) — `UIElement` subclass (:5); d81-d83 elements
  lane (f23 continuation), not a screen.
- Row elements `UIDebugCommandItem`/`UIDebugLoadoutItem`
  (`Elements/*.cs`) are `UIPanel` subclasses, not states — inside d85's
  row-9 claim ("the `UIDebugLoadoutItem`-family screens" row is these two).

## PORTED-row verification evidence (checked on this base, not quoted from docs)

- `flow.rs:28,30,36,42,46,55` — the six `FrontScreen` variants exist with
  the screen-state payloads (`CreationState`, `NewWorldDraft`,
  `KeyboardSession`, `WorldLoadState`); `FlowClick` targets at `:239-248`.
- `ui/char_select.rs:35-63` — `UI.SelectPlayer`/`UI.Back`/`UI.New` keys and
  `ListPadding = 5f` cited to `UICharacterSelect.cs:52-76`.
- `ui/char_creation.rs`, `ui/world_select.rs`, `ui/keyboard.rs:3,11` —
  module headers cite their C# states (`UIVirtualKeyboard.cs:680-746`
  DrawSelf two-layer note).
- `ui/world_creation.rs:1-10` — 500x452 root, name/seed plates, three
  option rows, 84x84 preview; `:525-529` documents the Advanced-tab
  degradation; `:710-711` notes `SecretSeed.CheckInputForSecretSeed` flags
  resolve from the draft text.
- `ui/world_load.rs:134,177-192` — `WorldLoadState`, the 600→700
  `GetLerpValue` bar-top lerp; witness tests `:598-680` cover the 270/320/370
  layout, the Inventory-queues-abort-once latch, the null-progress draw
  gate, the `UI.Canceling` override and the secret-seed message overrides.
- `ui/achievements.rs:1-2,83,282` — `UIAchievementsMenu` + list-item port,
  panel colour `(33,43,79)*0.8f`; live draw call `app/frame/draw.rs:1837`.
- `ui/emotes.rs:1-2` — `UIEmotesMenu` wheel + `EmoteBubble.Draw:209-251`.
- `ui/bestiary.rs:1,44-49,156-179` — `UIBestiaryTest.cs` BuildPage/progress
  bar citations throughout.
- `ui/crafting.rs:1-2,29-35,73` — `NewCraftingUI.cs:300-306` ctor geometry
  (20, 312, 490, -350) ported as static chrome.
- Data-not-screen confirmations: `creative.rs:1` cites
  `GameContent/Creative/CreativePowers.cs` (no `UICreativePowersMenu`
  reference anywhere in `crates/`); `input_map.rs:1-8` cites the
  `Main.cUp..cMapAlphaDown` bind strings (no `UIManageControls` reference).

## Follow-ups (owned elsewhere, listed so nothing dangles)

- research/18 gap table (:155-177): stale line counts + the `UIWorldLoad`
  PARTIAL row + missing exclusion markers — d85's note and d86's port
  commits own the edits; this doc is the corrected count source meanwhile.
- research/38-7 "undocumented absence": `UIBestiaryTest` half is stale
  (ported); `UITextWrappingTest` half closes with d86's scope-out.
- The `AWorkshopPublishInfoState` pair needs an explicit owner decision at
  merge (d84 ports vs d86 exclusion) — conflict 4's ruling extends to it.