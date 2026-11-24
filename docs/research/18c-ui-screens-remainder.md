# 18c — the missing `UIState` screens: the d86 remainder (census + ports)

> **LANDED (2026-11-21, code-only — witnesses written, not run):**
> `21fab727` UIManageControls (`ui/manage_controls.rs` + `input_profiles.rs`,
> 19 witnesses) · `66176102` the AWorkshopPublishInfoState pair data/state
> half (`ui/workshop_publish_info.rs`, 9 witnesses) · `19641e25` the
> UIWorldLoad re-verification note in code + ANALYSIS. Scope-outs
> (UIWorldGenDebug/UITextWrappingTest) recorded below + in ANALYSIS.

> Wave 4 · task/W4-d86-ui-screens-3 · base a725acb0 · researched 2026-11-20 ·
> sources: `decompiled/Terraria/GameContent/UI/States/*` (29 files),
> `decompiled/Terraria/GameContent/UI/Elements/*`, `crates/tr-app/src/ui/` +
> research/18's gap table (:155-177). Code-only mode: witnesses written, not run.

## Scope claim — RECONCILED with the pushed d84/d85 claims + the f22 census

The original phase-0 claim below was written while d84/d85 were still at base.
All three waves then pushed their splits in parallel and they disagreed; the
verify-only f22 census (`origin/task/W4-f22-ui-screens-4` @ `d9f9b60a`,
`docs/research/18d-ui-screens-4-census.md`, "Claim conflicts" 1-5) recorded the
conflicts and ruled by the lanes' own split records
(`origin/task/W4-d84-ui-screens-1` @ `e0f69499`
`docs/research/W4-UI-SCREENS-RANGES.md`, `origin/task/W4-d85-ui-screens-2` @
`b0ef93b0` research/18 split note). **This branch accepts those rulings** and
re-scopes to what f22 assigns d86 — no keep-one contention is left behind:

| Slice | Screens / classes | Owner (reconciled) |
| --- | --- | --- |
| d84 | Workshop hub/publish-select/import ×4 + ResourcePack selection/info ×2 (rows 1-6) | NOT touched here |
| d85 | `UIDebugCommandsList` + `UIDebugLoadoutsList` + their row elements, `UIReportsPage` (+ its reporters data family), `UIWorldCreationAdvanced` + `UIWorldCreationAdvancedSecretSeedsList` (rows 7-12) | NOT touched here — d86's original co-claim on reports/debug-lists and its advanced-seeds attribution to d84 are BOTH WITHDRAWN per f22 conflicts 1+3 |
| **d86 (this branch)** | `UIManageControls` (1061 raw ln) + the `PlayerInputProfile`/preset data it edits; the `AWorkshopPublishInfoState` pair owner decision (f22 conflict 4/5); the `UIWorldLoad` PARTIAL-row re-verification; the `UIWorldGenDebug`/`UITextWrappingTest` scope-outs | this doc + the ports below |
| skipped by directive | `UICreativePowersMenu` (597 ln, journey powers — c24 journey-ui lane) | NOT touched here |

The `UIKeybinding*ListItem` row elements are NOT re-ported here: d83 already
lands them (`origin/task/W4-d83-ui-elements-3`, `ui/keybinding_rows.rs`,
research/18b rows 40-44). d86's screen port cites them and stays self-contained
until the merge; the merger keeps d83's row module.

f22 conflict 5 is also accepted: the original doc described reports/debug-list
ports in past tense that never existed on any pushed tree (`c9855a4f` = doc
only). Those rows below are corrected to d85-owned MISSING; the only d86 code
claims are the ones this branch actually lands.

## Census — every `UIState` subclass in 1.4.5.8 vs the port (29 files)

Line counts: f22's raw-file counts (`(Get-Content).Count`) supersede both older
tables (research/18's "Lines" held class-declaration line numbers; this doc's
first pass counted differently). Verdicts re-checked on base a725acb0.

| C# state (file) | Lines | Rust counterpart | Verdict |
| --- | --- | --- | --- |
| `UICharacterSelect` | 311 | `ui/char_select.rs` + `flow.rs::FrontScreen::CharSelect` | PORTED |
| `UICharacterCreation` | 2381 | `ui/char_creation.rs` | PORTED |
| `UIWorldSelect` | 347 | `ui/world_select.rs` | PORTED |
| `UIWorldCreation` | 1382 | `ui/world_creation.rs` | PORTED (base tab; Advanced seed icon click documented no-op, `world_creation.rs:525-529` — d85's seed) |
| `UIVirtualKeyboard` | 1087 | `ui/keyboard.rs` | PORTED |
| `UIWorldLoad` | 137 | `ui/world_load.rs` (incl. `UIGenProgressBar` + `UIHeader`) | PORTED — see the re-verification below; research/18's "PARTIAL folded into world_select/save_list" row is STALE |
| `UIAchievementsMenu` | 223 | `ui/achievements.rs` | PORTED |
| `UIEmotesMenu` | 656 | `ui/emotes.rs` | PORTED |
| `UIBestiaryTest` | 856 | `ui/bestiary.rs` | PORTED (the file *is* the live bestiary screen despite the `Test` name; research/38:119's "no port" half is stale — c37 lane owns the residuals) |
| `NewCraftingUI` (outside `States/`) | — | `ui/crafting.rs` | PORTED (f22's 28th descendant) |
| `UICreativePowersMenu` | 597 | none (`creative.rs` is the powers DATA) | MISSING — journey (c24 lane), skipped by directive |
| `UIManageControls` | 1061 | `ui/manage_controls.rs` + `input_profiles.rs` (d86) | **PORTED (d86)** — screen state machine + the profile data it edits; rows cite d83's `ui/keybinding_rows.rs` |
| `UIWorldCreationAdvanced` | 532 | none | MISSING — d85 claim (f22 conflict 1 ruling) |
| `UIWorldCreationAdvancedSecretSeedsList` | 459 | none | MISSING — d85 claim |
| `UIReportsPage` | 248 | none | MISSING — d85 claim (d86 co-claim withdrawn, f22 conflict 3) |
| `UIDebugCommandsList` | 165 | none | MISSING — d85 claim (d86 co-claim withdrawn) |
| `UIDebugLoadoutsList` | 146 | none | MISSING — d85 claim (d86 co-claim withdrawn) |
| `UIWorldGenDebug` | 1702 | none | MISSING — d86 scope-out (debug-only; see below) |
| `UITextWrappingTest` | 336 | none | MISSING — d86 scope-out (test-only; research/38:119 UNDOCUMENTED-ABSENCE, now recorded) |
| `UIWorkshopHub` | 374 | none | d84 claim; standing Workshop exclusion (ANALYSIS:23673) is the fallback |
| `UIWorkshopSelectResourcePackToPublish` | 235 | none | d84 claim; same fallback |
| `UIWorkshopSelectWorldToPublish` | 233 | none | d84 claim; same fallback |
| `UIWorkshopWorldImport` | 251 | none | d84 claim; same fallback |
| `AWorkshopPublishInfoState` (+`ForResourcePack`/`ForWorld`) | 822+46+46 | `ui/workshop_publish_info.rs` (d86, data/state half) | **DECIDED (d86)** — see the pair decision below (f22 conflict 4/5) |
| `UIResourcePackSelectionMenu` | 591 | none | d84 port planned vs MISSING-with-dependency (no `AssetSourceController`/`ResourcePackList` port); d84's data layer decides at merge |
| `UIResourcePackInfoMenu` | 210 | none | same dependency |
| `UIGamepadHelper` | 367 | none | a `struct`, NOT a `UIState` (f22); gamepad snap helper — Stage-1 exclusion |
| `UISortableElement` | 22 | d83 `ui/sortable.rs` | element (d81-d83 lane), not a screen |

**Non-screen companions**: the `UIReportsPage` data sources
(`DataStructures/IssueReport.cs`, `GeneralIssueReporter.cs`, `IProvideReports.cs`,
`GameContent/UI/IssueReportsIndicator.cs`) follow the page to **d85** — the
original d86 claim on them is withdrawn. The indicator's only
`AttemptLettingPlayerKnow` caller is the resource-pack audio reload failure
(`Audio/LegacyAudioSystem.cs:116-118`) — with no resource packs a port's
reporter stays empty exactly as vanilla does on a pack-free install; recorded
here so d85's port keeps the `AddReport` seam.

## Scope-outs (recorded per research/18's task seed "record in module 38")

- **Workshop select/hub/import family (4 files)**: d84 owns the ports; the
  standing EXCLUDED-BY-DESIGN record (research/38:111, ANALYSIS:23673 —
  `SocialAPI.Workshop` has no port) is the fallback if d84 descopes. The
  `AWorkshopPublishInfoState` pair is decided separately below.
- **`UIWorldGenDebug` (1 702 ln)**: dev-only worldgen inspector; opens only via
  `DebugOptions.enableDebugCommands && F5` (`UIWorldLoad.cs:50-52`) or the
  worldgen thread's debug hook (`WorldBuilding/WorldGenerator.cs:477`); reads
  `Testing/*` + writes `dev-worldgen.json`. Excluded with the `Testing/**`
  subsystem (research/38:1260's exclusion row). NOT ported; the `UIWorldLoad`
  F5 arm is therefore absent in `world_load.rs` and recorded there (this
  branch's `UIWorldLoad` commit adds the note f22 conflict 5 asked for).
- **`UITextWrappingTest` (336 ln)**: dev-only text-wrap harness over 6 draw
  modes (`UITextWrappingTest.cs:16-24`); reached only from debug chat commands.
  Closes research/38-7's "undocumented absence" for this file (the
  `UIBestiaryTest` half of that finding is stale — the bestiary is ported).
- **`UIResourcePackSelectionMenu`/`UIResourcePackInfoMenu` (801 ln)**: the
  screens are pure views over `AssetSourceController` + `ResourcePackList`,
  neither of which exists in the port (`Main.cs:52371-52375` constructs them).
  d84 plans the ports plus a data layer; if that layer does not land, this
  MISSING-with-dependency verdict is the fallback record (f22 conflict 4).

## The `AWorkshopPublishInfoState` pair — d86's owner decision (f22 conflicts 4/5)

f22 left the pair without an owner: d86's first pass said EXCLUDED, d84's
publish ports "must cover or record it". Decision, checked against the source
in ≤100-line windows (`AWorkshopPublishInfoState.cs:1-822` +
both 46-line subclasses):

- The pair is the publish-info FORM: publicity radio (Private/FriendsOnly/
  Public, `:356-402`), the tag grid (3-per-row, `:404-432`), the preview-image
  plate (`:128-177`, `:680-757`), the Steam disclaimer (`:209-276`), the
  description strip (`:491-523`, `:553-581`), Back/Publish (`:587-653`).
- Its Steam-facing halves are dead on a no-Steam build EXACTLY as vanilla:
  `GetTagsToShow()` reads `SocialAPI.Workshop.SupportedTags` (null → the screen
  is unreachable, both constructors are only called from the Workshop select
  screens), `TryFindingTags` reads `SocialAPI.Workshop.TryGetInfoFor*`,
  `GoToPublishConfirmation` gates on `SocialAPI.Workshop != null` and then
  unconditionally returns to `_previousUIState` via menuMode 888
  (`WorkshopPublishInfoStateForWorld.cs:28-35`).
- **Verdict: the form's DATA/STATE half is portable and ported by d86**
  (`ui/workshop_publish_info.rs`): the publish-settings assembly
  (`GetPublishSettings` `:668-678`), the publicity/tag selection machines, the
  `PrettifyPath` 30-char truncation (`:693-713`), the >512px preview rejection
  (`:732-742`), the back-button routing (`:592-601`), the `FoundWorkshopEntryInfo`
  re-import (`:179-199`) — over the `Social/Base` data types
  (`WorkshopItemPublicSettingId`, `WorkshopTagOption`,
  `WorkshopItemPublishSettings`, `FoundWorkshopEntryInfo`). The Steam I/O
  (`FileBrowser.OpenFilePanel`, `Texture2D.FromStream`, `OpenURL`,
  `SocialAPI.Workshop.Publish*`) stays excluded with `Social/**`
  (ANALYSIS:23673); the port's seams take the dialog's RESULT as input, so a
  future native file picker plugs in without touching the state machine.
  d84's select-screen ports may consume this module or keep their own; the
  merger keeps one. This closes the pair's ownership without porting Steam.

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
- Deliberate deferrals already admitted in code (`world_load.rs:71-75`,
  `:300-303`) and ANALYSIS "The loading screen": the drunk-world evil flicker
  (`UIGenProgressBar.cs:63-67`) and drunk random-digit message
  (`UIWorldLoad.cs:93-103`) roll `Main.rand` — deferred with the draw seam to
  keep the RNG order; `Main.gameTips` (`:117-118`) is research/19 M3.
- Recorded by this branch's `UIWorldLoad` commit (f22 conflict 5): the F5
  `UIWorldGenDebug.Open()` arm (`:50-52`) is scope-out (debug), and
  `UpdateGamepadSquiggle` (`:122-127`) + `OnActivate`'s `ChangePoint(3000)`
  (`:37-44`) are gamepad (Stage-1 exclusion). Verdict: **PORTED with recorded
  deferrals**, row fixed in research/18.