# 18c — the missing `UIState` screens: the d86 remainder (census + ports)

> Wave 4 · task/W4-d86-ui-screens-3 · base a725acb0 · researched 2026-11-20 ·
> sources: `decompiled/Terraria/GameContent/UI/States/*` (29 files),
> `decompiled/Terraria/GameContent/UI/Elements/*`, `crates/tr-app/src/ui/` +
> research/18's gap table (:155-177). Code-only mode: witnesses written, not run.

## Scope claim (committed FIRST — the d84/d85/d86 split)

The wave-4 UI lane is d81-d88 (elements 1-3, screens 1-3, margins/snap, list
scissor). d84 (`ui-screens-1`) and d85 (`ui-screens-2`) were still at base when
this branch started, so the split below is **this branch's claim**, recorded
before any port landed; if d84/d85 land overlapping files, the merger reconciles
keep-one per file.

| Slice | Screens / classes | Owner |
| --- | --- | --- |
| d84 (claimed elsewhere) | the two L seeds research/18 names first: `UIWorldCreationAdvanced` (485 ln) + `UIWorldCreationAdvancedSecretSeedsList` (424 ln) | NOT touched here |
| d85 (claimed elsewhere) | `UIManageControls` (979 ln) + its `UIKeybinding*ListItem` element family | NOT touched here |
| **d86 (this branch)** | **the remainder**: `UIReportsPage` + the `IProvideReports` data family + `IssueReportsIndicator`; `UIDebugCommandsList`; `UIDebugLoadoutsList`; the `UIWorldLoad` PARTIAL-row re-verification; the scope-out census for Workshop/ResourcePack/Debug/Test screens | this doc + the ports below |
| skipped by directive | `UICreativePowersMenu` (539 ln, journey powers — c24 journey-ui lane) | NOT touched here |

## Census — every `UIState` subclass in 1.4.5.8 vs the port (29 files)

Line counts re-measured on the base tree (research/18's counts were stale).

| C# state (file) | Lines | Rust counterpart | Verdict |
| --- | --- | --- | --- |
| `UICharacterSelect` | 291 | `ui/char_select.rs` + `flow.rs::FrontScreen::CharSelect` | PORTED |
| `UICharacterCreation` | 2232 | `ui/char_creation.rs` | PORTED |
| `UIWorldSelect` | 322 | `ui/world_select.rs` | PORTED |
| `UIWorldCreation` | 1280 | `ui/world_creation.rs` | PORTED (base tab; Advanced seed icon click documented no-op, `world_creation.rs:527-528` — d84's seed) |
| `UIVirtualKeyboard` | 1004 | `ui/keyboard.rs` | PORTED |
| `UIWorldLoad` | 137 (was "16") | `ui/world_load.rs` (730 ln incl. `UIGenProgressBar` + `UIHeader`) | PORTED — see the re-verification below; research/18's "PARTIAL folded into world_select/save_list" row is STALE |
| `UIAchievementsMenu` | 208 | `ui/achievements.rs` | PORTED |
| `UIEmotesMenu` | 628 | `ui/emotes.rs` | PORTED |
| `UIBestiaryTest` | 791 | `ui/bestiary.rs` | PORTED (the file *is* the live bestiary screen despite the `Test` name; research/38:119's "no port" half is stale — c37 lane owns the residuals) |
| `UICreativePowersMenu` | 539 | none | MISSING — journey (c24 lane), skipped by directive |
| `UIManageControls` | 979 (was "20") | none (`input_map.rs` holds the bind data, not the screen) | MISSING — d85 claim |
| `UIWorldCreationAdvanced` | 485 | none | MISSING — d84 claim (research/18 M seed) |
| `UIWorldCreationAdvancedSecretSeedsList` | 424 | none | MISSING — d84 claim |
| `UIReportsPage` | 248 (was "16") | `ui/reports.rs` | **PORTED (d86)** — the page + the reporters data family |
| `UIDebugCommandsList` | 165 (was "15") | `ui/debug_lists.rs` | **PORTED (d86)** — state machine + search filter, unwired (dev-only) |
| `UIDebugLoadoutsList` | 146 (was "14") | `ui/debug_lists.rs` | **PORTED (d86)** — same shell, `_lastSearch` static latch |
| `UIWorldGenDebug` | 1581 (was "27") | none | MISSING — scope-out (debug-only; see below) |
| `UITextWrappingTest` | 311 (was "14") | none | MISSING — scope-out (test-only; research/38:119 UNDOCUMENTED-ABSENCE, now recorded) |
| `UIWorkshopHub` | 337 | none | EXCLUDED — Steam Workshop (research/38:111); research/18's "13 lines" was the class header, not the file |
| `UIWorkshopSelectResourcePackToPublish` | 215 | none | EXCLUDED — Workshop publish |
| `UIWorkshopSelectWorldToPublish` | 214 | none | EXCLUDED — Workshop publish |
| `UIWorkshopWorldImport` | 228 | none | EXCLUDED — Workshop import |
| `AWorkshopPublishInfoState` (+`ForResourcePack`/`ForWorld`, 40+40) | 756 | none | EXCLUDED — Workshop publish base; its `FileBrowser` call is the dialog's only caller (research/38:88) |
| `UIResourcePackSelectionMenu` | 548 | none | MISSING — resource packs; the pack-loading system itself is unported, so the screen has no data source; recorded, not ported |
| `UIResourcePackInfoMenu` | 193 | none | MISSING — same dependency |
| `UIGamepadHelper` | 345 | none | framework helper (gamepad — research/18 deviation 10, Stage-1 exclusion); not a screen |
| `UISortableElement` | 18 | none | element (d81-d83 lane), not a screen |

**Non-screen companions ported by d86** (the `UIReportsPage` data sources):
`DataStructures/IssueReport.cs` (16), `GeneralIssueReporter.cs` (18),
`IProvideReports.cs` (interface), `GameContent/UI/IssueReportsIndicator.cs`
(80) — all in `ui/reports.rs`. The indicator's only `AttemptLettingPlayerKnow`
caller is the resource-pack audio reload failure
(`Audio/LegacyAudioSystem.cs:116-118`) — with no resource packs the port's
reporter stays empty exactly as vanilla does on a pack-free install; the
`AddReport` seam exists for future callers.

## Scope-outs (recorded per research/18's task seed "record in module 38")

- **Workshop family (7 files, ~1 830 ln)**: EXCLUDED-BY-DESIGN, already covered
  by research/38:111 (`SocialAPI.Workshop` has no port). research/18:159-162's
  rows said "none / Steam Workshop hub" without an exclusion marker — this doc
  is that marker.
- **`UIWorldGenDebug` (1 581 ln)**: dev-only worldgen inspector; opens only via
  `DebugOptions.enableDebugCommands && F5` (`UIWorldLoad.cs:50-52`) or the
  worldgen thread's debug hook (`WorldBuilding/WorldGenerator.cs:477`); reads
  `Testing/*` + writes `dev-worldgen.json`. Excluded with the `Testing/**`
  subsystem (research/38:1260's exclusion row). NOT ported; the `UIWorldLoad`
  F5 arm is therefore absent in `world_load.rs` and recorded there.
- **`UITextWrappingTest` (311 ln)**: dev-only text-wrap harness over 6 draw
  modes (`UITextWrappingTest.cs:16-24`); reached only from debug chat commands.
  Closes research/38-7's "undocumented absence" for this file (the
  `UIBestiaryTest` half of that finding is stale — the bestiary is ported).
- **`UIResourcePackSelectionMenu`/`UIResourcePackInfoMenu` (741 ln)**: the
  screens are pure views over `AssetSourceController` + `ResourcePackList`,
  neither of which exists in the port (`Main.cs:52371-52375` constructs them).
  Porting the view without the pack system would be a dead screen; recorded as
  MISSING-with-dependency, not scheduled here.

## `UIWorldLoad` PARTIAL-row re-verification (research/18:175)

research/18's row read "`UIWorldLoad` | 16 | `world_select.rs`/`save_list.rs`
(folded in) | PARTIAL — verify no screen is lost". Verified against the tree:

- The screen **is** a first-class state: `flow.rs::FrontScreen::WorldLoad`
  (S1-E-17), payload `ui::world_load::WorldLoadState`, drawn from
  `frame.rs:355`'s per-frame update seam; nothing was folded into
  `world_select`/`save_list`. No screen is lost.
- Ported 1:1: ctor 270/-70 layout (`:24-35`), the 600→700 screen-height lerp
  (`:60-61`), Inventory-queues-abort-once + sound 11 (`:54-58`), the null
  progress draw gate (`:69-73`), `UI.Canceling` override (`:86-89`), the bees
  + get-good-reversal message overrides (`:104-116`), `GetStatusText`'s
  `"0.0%"` half-away-from-zero formatting (`:129-136`), `UIGenProgressBar`'s
  570/508 widths + packed colours + even round-down, `UIHeader`'s four-corner
  draw + tenth-anniversary HotPink arm.
- Deliberate deferrals already admitted in code (`world_load.rs:73-75`,
  `:227-229`) and ANALYSIS "The loading screen": the drunk-world evil flicker
  (`UIGenProgressBar.cs:63-67`) and drunk random-digit message
  (`UIWorldLoad.cs:93-103`) roll `Main.rand` — deferred with the draw seam to
  keep the RNG order; `Main.gameTips` (`:117-118`) is research/19 M3.
- Newly recorded here: the F5 `UIWorldGenDebug.Open()` arm (`:50-52`) is
  scope-out (debug), and `UpdateGamepadSquiggle` (`:122-127`) is gamepad
  (Stage-1 exclusion). Verdict: **PORTED with recorded deferrals**, row fixed
  in research/18.