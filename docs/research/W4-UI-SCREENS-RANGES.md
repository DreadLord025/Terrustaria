# W4 UI screens lane — range split (research/18 gap table)

> Branch lane: `task/W4-d84-ui-screens-1` (d84), `…-d85` (d85), `…-d86` (d86).
> Source of the split: `docs/research/18-ui-framework.md` § "UIState subclasses
> with no Rust screen at all" (the 17-row census). Rows already ported
> (`UIWorldLoad` S1-E-17, `NewCraftingUI`, the select/creation/keyboard/
> achievements/bestiary/emotes eight) are not in the lane. `UICreativePowersMenu`
> (Journey powers) is **c24's** — excluded from every d8x range.

## d84 — rows 1–6 (the Workshop/ResourcePack cluster) — THIS BRANCH

| # | C# state | Rust screen | Module |
| --- | --- | --- | --- |
| 1 | `UIWorkshopHub` (337 lines) | `FrontScreen::WorkshopHub` | `ui/workshop_hub.rs` |
| 2 | `UIWorkshopSelectResourcePackToPublish` (215) | `FrontScreen::WorkshopPublishPacks` | `ui/workshop_publish.rs` |
| 3 | `UIWorkshopSelectWorldToPublish` (214) | `FrontScreen::WorkshopPublishWorlds` | `ui/workshop_publish.rs` |
| 4 | `UIWorkshopWorldImport` (228) | `FrontScreen::WorkshopImportWorlds` | `ui/workshop_publish.rs` |
| 5 | `UIResourcePackSelectionMenu` (548) | `FrontScreen::ResourcePackSelection` | `ui/resource_packs.rs` |
| 6 | `UIResourcePackInfoMenu` (193) | `FrontScreen::ResourcePackInfo` | `ui/resource_pack_info.rs` |

Data layer: `tr-app/src/resource_pack.rs` — `ResourcePack` /
`ResourcePackList` (`IO/ResourcePack.cs`, `IO/ResourcePackList.cs`): the
`ResourcePacks` config key (`Main.cs:6519-6536`), the
`EnabledPacks`/`DisabledPacks`/`AllPacks` sorts, directory-pack discovery
(`pack.json`), the `SortingOrder` priority maths the selection menu edits.

## d85 — rows 7–12 (the debug cluster + reports)

7 `UIDebugCommandsList` (152), 8 `UIDebugLoadoutsList` (133),
9 the `UIDebugLoadoutItem`-family screens, 10 `UIReportsPage` (227),
11 `UIWorldCreationAdvanced` (485, ROADMAP:953), 12
`UIWorldCreationAdvancedSecretSeedsList` (424).

## d86 — the rest

13 `UIWorldGenDebug` (1581), 14 `UITextWrappingTest` (311, test-only),
15 `UIManageControls` (979, `input_map.rs` data behind it),
16 `UIWorldLoad` re-verification (PARTIAL row — ported S1-E-17; verify no
screen behaviour lost), 17 the census re-audit (`NewCraftingUI` + the eight
ported rows stay PORTED; `UICreativePowersMenu` handed to c24, not ported here).
Also in scope if rows shift: `UIGamepadHelper`/snap points stay MISSING
(gamepad unsupported, §J.3) — not a screen.

## Shared conventions (landed UIState ports)

- One `FrontScreen` variant per screen; payload = the screen's own state
  (`world_load.rs`'s `WorldLoadState` pattern); `FrontScreenBehaviours`
  draw/click/key arms; hops ride `FlowClick` + `next_flow_state`
  (`flow.rs`), `enter_front` re-lists per `OnActivate`.
- Layout through `uistate::resolve` with C# citations per block; draws via
  `panel::draw_panel` / `uistate::draw_text_panel` / `draw_spliced_panel`;
  lists reuse `char_select`'s scrollbar maths (`panel::scrollbar_*`,
  `scroll_wheel`/`clamp_scroll`) with per-screen row pitch.
- Deviations recorded in `docs/ANALYSIS.md` ("The Workshop/ResourcePack
  screens"); anchors.txt rows per landed screen; PORT_MAP + research/18
  gap-table statuses updated in the same commits.
