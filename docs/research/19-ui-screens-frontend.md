# 19 — Front-end screens (char/world create + select)

> Wave 6 · researched 2026-11-12 · sources: decompiled + crates + ROADMAP/ANALYSIS/PORT_MAP

## Scope

The modern front end that sits in front of the game: the character/world select
screens and their cards, the character and world creation screens, the virtual
keyboard that names things, the delete/discard confirm pages, the credits page
and the save listing that feeds the cards. On the C# side this is
`Main.menuMode == 888`'s `MenuUI` states plus the legacy `menuMode` 5/9/40/3000
pages they park onto (`Main.cs:51059-51069`, `:49431-49467`, `:49674-49692`,
`:49693-49704`, `:52346-52369`), and the loading flow one hop further out
(`UIWorldLoad` + `UIGenProgressBar`).

C# under audit (line counts):

- `decompiled/Terraria/GameContent/UI/States/UICharacterSelect.cs` (311)
- `decompiled/Terraria/GameContent/UI/States/UIWorldSelect.cs` (347)
- `decompiled/Terraria/GameContent/UI/States/UICharacterCreation.cs` (2381)
- `decompiled/Terraria/GameContent/UI/States/UIWorldCreation.cs` (1382)
- `decompiled/Terraria/GameContent/UI/States/UIVirtualKeyboard.cs` (1087)
- `decompiled/Terraria/GameContent/UI/States/UIWorldLoad.cs` (137)
- `decompiled/Terraria/GameContent/UI/Elements/UICharacterListItem.cs` (381),
  `UIWorldListItem.cs` (533), `AWorldListItem.cs` (201), `UICharacter.cs` (163),
  `UIGenProgressBar.cs` (126)
- `decompiled/Terraria/Main.cs` (menuMode sites + `LoadPlayers`/`LoadWorlds`,
  `ErasePlayer`/`EraseWorld`, `SelectPlayer`, `OpenCharacterSelectUI`
  `:52346-52350`, `OpenWorldSelectUI` `:52365-52369`)

Rust under audit:

- `crates/tr-app/src/app/flow.rs` (2924) — the `Main.menuMode == 888` state
  machine, `FrontScreenBehaviours` (≈ `UIState`), the flow table
- `crates/tr-app/src/app/session.rs` (181) — `FinishCreatingCharacter`,
  `FinishCreatingWorld`, the session card entry, the erase handlers
- `crates/tr-app/src/ui/char_select.rs` (1306), `ui/char_creation.rs` (2445),
  `ui/world_select.rs` (1326), `ui/world_creation.rs` (1539),
  `ui/keyboard.rs` (1325), `ui/save_list.rs` (929)
- shared machinery the screens sit on: `ui/framework.rs` (177),
  `ui/select_hit.rs` (36), `ui/pointer.rs` (432)
- `crates/tr-app/src/app/mod.rs` (4301) — `enter_front` `:1350`,
  `open_keyboard*` `:1402/:1410`, `keyboard_submit` `:1465`,
  `keyboard_cancel` `:1548`, `swap_world` `:1630`, `latest_own_world` `:3746`;
  `app/frame.rs` — `tick_pointer_clock` call `:105`, the Play swap `:303-326`;
  `app/clicks.rs` — the lobby rows that open the select screens `:1307`
  (`Main.cs:48931`'s menuMode 888 hop)

Responsibility: turn the "pick a character / pick a world / make one" part of
the boot path into an immediate-mode port of the C# `UIState`s, keeping the
single-mode invariant (`menuMode` is one variable; exactly one of the front
screens or the legacy menu is live) and the 200 ms post-`SetState` click
lockout (`UserInterface.cs:178`, `:203-204`).

## Key entities

C#:

- `UICharacterSelect` (`UICharacterSelect.cs:16`) — the singleton `_characterSelectMenu`
  (`Main.cs:66797`): `OnInitialize` `:100-133`, `UpdatePlayersList` `:146-157`
  (LINQ order: `IsFavorite` desc, `LastPlayed` desc, `Name`, `GetFileName()`),
  `NewCharacterClick` `:108-114` (`Main.PendingPlayer = new Player()` +
  menuMode 888 + new `UICharacterCreation`), `GoBackClick` `:116-120`
  (`menuMode = 0`), `PlayGame` `:256-261` → `Main.SelectPlayer`
- `UIWorldSelect` (`UIWorldSelect.cs:16`) — `_worldSelectMenu` (`Main.cs:66798`):
  `UpdateWorldsList` `:153-166` (joinable desc, newly-generated desc, favourite
  desc, played desc, `LastPlayed` desc, `Name`, `GetFileName`),
  `NewWorldClick` `:110-116` (`Main.newWorldName` = `Lang.gen[57] + " " +
  (WorldList.Count + 1)`), `GoBackClick` `:118-122`,
  `CanWorldBeJoinedByActivePlayer` `:177-180`
  (`(active == 3) == (world GameMode == 3)`), `HasWorldBeenPlayedByActivePlayer`
  `:182-186` (`WorldMap.TryGetMapPath`)
- `UICharacterListItem` (`UICharacterListItem.cs:15`) — one card:
  `DrawSelf` `:300-372` (difficulty switch `:353-370`, life/mana panel `:339-347`),
  the button table `:140-194` (Play/Favorite/Rename/Delete + the favourited
  delete gate `:174-184`), `DeleteButtonClick` `:242-254` (menuMode 5),
  `RenameButtonClick` `:197-201` (the naming keyboard, `allowEmpty: true`,
  `maxLength` 20), `GetPlayerData`/`UpdateCursorPosition`
- `UIWorldListItem` (`UIWorldListItem.cs:15`) — one card: `DrawSelf` `:494-532`
  (name/divider/difficulty/size/created panels), the button table `:94-170`
  (play 4 / favorite 28 / rename 52 / delete `HAlign 1`; the cloud `:113-124`
  and seed `:125-136` slots), `PlayGame` `:360-385` (`IsValid` +
  `TryMovingToRejectionMenuIfNeeded` `:387-410` → menuMode 1000000),
  `DeleteButtonClick` `:346-358` (menuMode 9), `RenameButtonClick` `:416-421`
  (`maxLength` 27), `FavoriteButtonClick` `:455-458` (`UpdateOrder`)
- `UICharacterCreation` (`UICharacterCreation.cs:16`) — the creator:
  `dirty` `:154`, `GoBack` `:1205-1222` (dirty → park
  `BackupConfirmationState` `:1215` + menuMode 40), `MakeCategoriesBar`
  `:730-768`, `MakeInfoMenu` `:832-965` (name plate `:858`, the four difficulty
  rows `:889-910`, descriptions `:946-965`), `MakeHairStyleMenu`, `Click_Naming`
  `:1659-1666`, `Click_NamingAndCreating` `:1668-1682`,
  `OnFinishedNaming`/`OnFinishedNamingAndCreating` `:1684-1702`,
  `FinishCreatingCharacter` `:1704-1711`, `SetupPlayerStatsAndInventoryBasedOnDifficulty`
  `:1713-1762`, `Click_RandomizePlayer` `:1545-1657`,
  `GetPlayerTemplateValues` `:2006-2021`
- `UIWorldCreation` (`UIWorldCreation.cs:16`) — the creator: option fields
  `:51-143`, `BuildPage`/`MakeInfoMenu` `:163-301`, `SetDefaultOptions`
  `:313-326` (`Medium` `:318`, `Creative` iff active player is Journey `:319-322`,
  `Random` evil `:323`), the option builders `:358-495`, the click handlers
  `:497-575`, `Click_GoBack` `:665-668`, `Click_SetName` `:689-695`,
  `Click_OpenSeedInputMenu` `:697-708`, `Click_NamingAndCreating` `:710-724`,
  `OnFinishedSettingName` `:726-731`, `OnFinishedSettingSeed` `:817-831`,
  `TryApplyingCopiedSeed`, `FinishCreatingWorld` `:845-863`,
  `AssignRandomWorldName` `:1002-1025`
- `UIVirtualKeyboard` (`UIVirtualKeyboard.cs:16`) — `DrawSelf` `:680-746`
  (the two layers), the grid `:219-227`, shift/symbol/space/backspace
  `:228/:263/:452/:391`, Submit/Cancel `:291-326` (gamepad layer) and
  `:327-390` (desktop layer), `ValidateText` `:585-613`,
  `TextIsValidForSubmit` `:615-626`, `TypeText` `:517-533`, `CycleSymbols`
  `:933-950`
- `UIWorldLoad` (`UIWorldLoad.cs:16`) + `UIGenProgressBar`
  (`UIGenProgressBar.cs:8`) — the worldgen progress screen
  (`SetProgress` `:51-55`, `DrawSelf` `:57-86`, the 612x70 bar `:42-49`
  with the 570 px long / 508 px small fills)
- `Main.cs` — `LoadPlayers` `:4001-4030`, `LoadWorlds` `:3959-3981`,
  `ErasePlayer` `:4772-4791`, `EraseWorld` `:4796-4813`, `SelectPlayer`
  `:66210-66250` (menuMode 6 at `:66249`), the confirm pages 5/9/40
  `:49431-49467`/`:49674-49692`/`:49440-49467`

Rust:

- `crates/tr-app/src/app/flow.rs::FrontScreen` (`:25-47`) — the five screens
  (CharSelect/WorldSelect/CharCreation/WorldCreation/Keyboard) as
  `Main.menuMode`'s one variable
- `crates/tr-app/src/app/flow.rs::next_flow_state` (`:162-224`) — the pure
  `(front, backup, menu)` hop table, the 1:1 image of the menuMode transitions
- `crates/tr-app/src/app/flow.rs::FlowClick` (`:105-152`) — the click rows,
  carrying the card index where the App parks `Main.selectedPlayer`/`selectedWorld`
- `crates/tr-app/src/app/flow.rs::KeyboardSession`/`KeyboardTail` (`:59-97`) —
  the keyboard payload and its seven submit/cancel tails
- `crates/tr-app/src/app/flow.rs::FrontScreenBehaviours` (`:287-945`) —
  header/draw/click/key per screen ≈ `UIState.DrawSelf`/`Click_*`
- `crates/tr-app/src/app/flow.rs::App::handle_front_click` (`:988-1030`) — the
  `UserInterface` click gate + the BUG-17 cue, then `apply_front_flow`
  (`:1038-…`) which runs the table and performs the tail effects (erase, park,
  `play_world` at `:1047-1051`)
- `crates/tr-app/src/app/flow.rs::App::front_hit` (`:954-981`) — the element
  under the pointer for the pointer machine's double-click
- `crates/tr-app/src/app/session.rs::App::finish_creating_character` (`:18-41`),
  `::finish_creating_world` (`:50-62`), `::session_entry` (`:71-110`),
  `::erase_player_entry` (`:117-123`), `::erase_world_entry` (`:128-134`),
  `save_created_character` (`:141-176`)
- `crates/tr-app/src/app/mod.rs::App::enter_front` (`:1350-1393`) —
  `OpenCharacterSelectUI`/`OpenWorldSelectUI`'s `OnActivate` re-list +
  `ResetState`'s 200 ms lockout
- `crates/tr-app/src/app/mod.rs::App::keyboard_submit` (`:1465-1540`) and
  `::keyboard_cancel` (`:1548-1564`)
- `crates/tr-app/src/app/mod.rs::App::swap_world` (`:1630-…`) +
  `App.play_world` (`:355`) — the `PlayGame` → `Main.LoadWorld` tail
- `crates/tr-app/src/ui/save_list.rs::{PlayerEntry,WorldEntry,Appearance}`
  (`:21-133`), `list_players` `:255`, `list_worlds` `:167`,
  `sort_players` `:397`, `sort_worlds` `:415`, `erase_player` `:352`,
  `erase_world` `:340`, `size_bucket` `:148`
- `crates/tr-app/src/ui/char_select.rs::layout`/`resolved_layout`
  (`:176`/`:304`), `card_layout` `:366`, `difficulty_text` `:454`,
  `button_label` `:468`, `card_hit` `:541`, `elements` `:581`, `draw_card` `:643`
  (the BUG-02 preview call at `:749`)
- `crates/tr-app/src/ui/world_select.rs::card_layout` `:254`,
  `overlay_rects` `:353` (pinned, never drawn), `joinable` `:139`,
  `difficulty_text` `:151`, `elements` `:545`, `created_text` `:787`
- `crates/tr-app/src/ui/char_creation.rs::{NewPlayerDraft,CreationState}`
  (`:41`/`:160`), `HAIRSTYLES` `:261, `VALID_CLOTH_STYLES` `:230,
  `DIFFICULTY_IDS` `:322, `layout` `:476, `middle_for` `:1026,
  `hsl_sliders_update` `:1153, `click_at` `:1233, `tooltip_for` `:1308,
  `draw` `:1407
- `crates/tr-app/src/ui/world_creation.rs::NewWorldDraft` `:48,
  `size_dimensions` `:662, `seed_value` `:697, `seed_flags` `:718,
  `seed_identity` `:731, `create_world_in` `:746, `click_at` `:549,
  `preview_layers` `:596, `draw_preview` `:623
- `crates/tr-app/src/ui/keyboard.rs::{Request,KeyboardState,Context,KeyState}`
  (`:137`/`:172`/`:119`/`:160`), `layout` `:320, `click_at` `:605,
  `key_labels` `:253, `text_valid_for` `:562, `submit_colors` `:535,
  `draw` `:767

## Dependencies

- Calls into: `ui/uistate.rs` (the `UIElement` layout pass + the colour
  constants), `ui/panel.rs`, `ui/cards.rs`, `ui/text.rs`, `ui/color.rs`,
  `ui/sliders.rs` (the HSL bars), `ui/framework.rs`+`ui/select_hit.rs` (the
  `GetElementAt` walk), `ui/pointer.rs` (the `UserInterface` pointer machine),
  `crate::lang` (`UI.*`/`LegacyMenu.*`), `crate::assets::Lookups` (every sheet
  an `Option` — nothing panics on `Lookups::default()`),
  `crate::player_file` (the `.plr` read/write), `tr_core::save` (`.trwld`
  headers, the players/worlds dirs), `tr_core::binio`, `tr_core::seeds` (the
  seed codec), `tr_core::worldgen::generate` (creation), `crate::audio`
  (`Menu_Open`/`Menu_Close`/`Menu_Tick`), `crate::menu` (the confirm/credits
  pages)
- Called by: `app/loop.rs` (the front regime dispatch + Escape routing),
  `app/frame.rs` (`tick_pointer_clock` `:105`, the `play_world` swap `:303`),
  `app/clicks.rs` `:1307`/`:1374` (the lobby's Single Player/Player rows →
  `OpenCharacterSelectUI`, `Main.cs:48931`), `chest.rs` (the in-game rename
  keyboard, `KeyboardTail::ChestRename`)
- Shared data structures: `App.players`/`App.worlds` (the listings),
  `App.front`/`App.menu`/`App.front_backup`/`App.front_hint`,
  `App.session_draft`, `App.favorite_slots`, `App.play_world`,
  `tr_core::save::{PlayerFile → PlayerEntry, WorldEntry}`, `menu::Page`
  (`ConfirmDeletePlayer`/`ConfirmDeleteWorld`/`ConfirmDiscard`/`Credits`)

## Port status

| C# subsystem | Status | Rust location | Evidence / notes |
| --- | --- | --- | --- |
| The menuMode 888 state machine (one variable, exactly one side live, the 200 ms lockout) | PORTED | `app/flow.rs::FrontScreen` `:25-47`, `::next_flow_state` `:162-224`, `app/mod.rs::tick_pointer_clock` `:1281` + `frame.rs:105` | `Main.cs:51059-51069`'s single-mode invariant is asserted by `front_and_menu_are_never_both_some` `:1328`; the lockout decays on a live front screen exactly as `Main.UpdateUIStates` → `MenuUI.Update` → `UserInterface.cs:203-204` does |
| `UICharacterSelect` + `UICharacterListItem` cards | PORTED | `ui/char_select.rs` (layout `:176`, cards `:366`, draw `:643`, hit `:541`), `app/flow.rs` `:345-500` | `UICharacterSelect.cs:100-157`'s OnActivate re-list, card geometry, difficulty/life-mana rows, the hover labels `:140-194`, delete `:242-254`, rename `:197-201` |
| `UIWorldSelect` + `UIWorldListItem` cards | PORTED | `ui/world_select.rs` (cards `:254`, draw `:603`, hit `:545`), `app/flow.rs` `:505-820` | `UIWorldSelect.cs:137-166`'s re-list, `UIWorldListItem.cs:494-532`'s card draw, the button table `:94-170`, delete `:346-358`, rename `:416-421`, favourite `:455-458` |
| `UICharacterCreation` (categories, info/hairstyles/cloth/HSL menus, difficulty, naming) | PORTED (visual + flow) | `ui/char_creation.rs` (draft `:41`, layout `:476`, click `:1233`, draw `:1407`), `app/session.rs::finish_creating_character` `:18` | `UICharacterCreation.cs:730-768` categories, `:832-965` info, `:1659-1702` naming, `:1704-1711` finish. Gaps: the Randomize/Copy/Paste click arms and `GameTipsDisplay` (see deviations) |
| `UIWorldCreation` (name/seed/size/difficulty/evil, preview, create) | PORTED (visual + flow) | `ui/world_creation.rs` (draft `:48`, click `:549`, preview `:596/:623`, create `:746`), `app/session.rs::finish_creating_world` `:50` | `UIWorldCreation.cs:313-326` defaults, `:358-495` rows, `:665-745` name/seed/back, `:845-863` finish. The Advanced seed tab and its ring effect are out of scope (ANALYSIS `:2146-2149`) |
| `UIVirtualKeyboard` | PORTED | `ui/keyboard.rs` (layout `:320`, click `:605`, draw `:767`), `app/mod.rs::open_keyboard*` `:1402/:1410`, `::keyboard_submit` `:1465` | `UIVirtualKeyboard.cs:180-425` geometry, `:517-533`/`:933-950` typing, `:585-626` validation, both layers `:680-746` (the grid behind `TR_VK_GRID`, ANALYSIS `:2144-2145`) |
| Save listing (`LoadPlayers`/`LoadWorlds`, the two orderings, `ErasePlayer`/`EraseWorld`) | PORTED | `ui/save_list.rs` `:167`/`:255`/`:397`/`:415`/`:340`/`:352`, `app/session.rs` `:117`/`:128` | `Main.cs:4001-4030`/`:3959-3981`/`:4772-4791`/`:4796-4813`; real `.plr` and `.trwld` files list (BUG-02/03's reader). **PORT_MAP:180 is stale** ("partial (no `.plr` reader…)") |
| Journey/Classic/Expert/Master selection | PORTED both screens | `ui/char_creation.rs::DIFFICULTY_IDS` `:322`, `::difficulty_titles` `:327`, `::DIFFICULTY_COLORS` `:338`; `ui/world_creation.rs::DIFFICULTY_ROW_IDS` `:116`; `ui/char_select.rs::difficulty_text` `:454`; `ui/world_select.rs::difficulty_text` `:151` | `Player.difficulty` 0-3 = Classic/Medium/Hard/Journey; the char screen's row order is Journey/Classic/Medium/Hardcore (`UICharacterCreation.cs:889-910`), the world screen's is `Main.GameMode` `:34-40`; the `.plr` difficulty rides the card (`save_list.rs:296`) |
| Character creation `.plr` write | PORTED | `app/session.rs::save_created_character` `:141-176` → `crate::player_file::write_player_file` | `PlayerFileData.CreateAndSave` (`PlayerFileData.cs:50-63`) with the fresh defaults and the three starter items (`Player.cs:54356-54360`) — **ANALYSIS `:2120-2124` is stale** ("no `.plr` reader/writer") |
| Delete confirms (modes 5/9) and the discard confirm (mode 40) | PORTED | `menu/pages.rs::confirm_delete_player_items` `:1005`, `::confirm_delete_world_items` `:1023`, `::confirm_discard_items` `:1038`; `app/flow.rs::apply_front_flow` `:1056-…` | `Main.cs:49431-49467`/`:49674-49692`/`:49440-49467`; Yes erases and re-lists, No reopens the parked screen — the mode-40 No branch restores the exact `BackupConfirmationState` (`UICharacterCreation.cs:1215`, test `:1237`) |
| Credits (mode 3000) | PARTIAL | `menu/pages.rs::credits_items` `:1053`, `flow.rs::TrUiPreset::Credits` `:239` | world backdrop + the single Back row; `SkyManager.Activate("CreditsRoll")` and the rolling credits are unported (ANALYSIS `:2150-2151`) |
| World loading / generation progress screen | MISSING | — | `UIWorldLoad.cs:16-137` + `UIGenProgressBar.cs:8-126` have no counterpart; creation saves and the world select reopens instead (ANALYSIS `:2148-2149`, `app/session.rs:43-49`); Play swaps the world in-frame at a frame boundary (`frame.rs:303-326`) |
| Cloud / Steam cloud UI | MISSING (deliberate) | — | `SocialAPI.Cloud == null` is the port's state, the exact branch vanilla takes without an account (ANALYSIS `:2135-2137`); the cloud *slot's* `num += 24` still shifts rename to Left 52 (`world_select.rs:267-271` vs `UIWorldListItem.cs:123`) |
| Random world-name generation | PARTIAL (stand-in) | `ui/world_creation.rs::stand_in_name` `:103-105`, `::assign_random_world_name` `:97-99` | `AssignRandomWorldName`'s `RandomWorldName_*` composition and the 1-in-10000 `TheConstant` roll (`UIWorldCreation.cs:1002-1025`) are unported; the stand-in is `"World {n+1}"` (ANALYSIS `:2163-2169`) |
| Character preview on the select cards | PORTED | `ui/char_select.rs::draw_card` `:643` (the `draw_player_preview` call `:749`), `ui/save_list.rs::Appearance` `:59-105`, `render/entities.rs::draw_player_preview` `:1197` | BUG-02: the seven `.plr` colour triplets + hair/skin variant ride the card (`save_list.rs:294-317`); deviation: only the base body sheets decode, so every skin variant renders the base frame (ROADMAP `:1677`) |
| `.plr` favourites | PORTED | `ui/save_list.rs::PlayerEntry::inv_favorites` `:45`, `app/mod.rs::hydrate_favorites`/`favorite_bits`, `app/session.rs:36` | BUG-03 (`Player.cs:54820-54824` write, `:55340-55362` read); ROADMAP `:1678` |
| Play → load own world | PORTED | `app/mod.rs::App::play_world` `:355` + `::swap_world` `:1630-1704`, `frame.rs:303-326`, `::latest_own_world` `:3746` | BUG-04/05; `Main.LoadWorld`'s device/state split is mirrored (window/GPU/audio/music/shot survive, world state re-derives through `App::new`) |

### State-graph 1:1 verification vs `next_flow_state` (`flow.rs:162-224`)

| C# transition | Rust arm | Verdict |
| --- | --- | --- |
| `NewCharacterClick` → 888 `UICharacterCreation` (`UICharacterSelect.cs:108-114`) | `FlowClick::CharNew` `:173-179` | PASS — a fresh `CreationState::new()` draft, both sides cleared |
| `GoBackClick` → menuMode 0 (`UICharacterSelect.cs:116-120`) | `CharBack` `:170` | PASS — legacy main page, front cleared |
| `PlayGame` → `SelectPlayer` → menuMode 6 (`:256-261`, `Main.cs:66249`) | `CharPlay(_)` `:180` | PASS — world select opens |
| `DeleteButtonClick` → menuMode 5 (`UICharacterListItem.cs:242-254`) | `CharDelete(i)` `:183` | PASS — the confirm page parks the index/name |
| Mode 5 Yes/No → reopen char select (`Main.cs:49434-49439`) | `ConfirmYes/No` on `ConfirmDeletePlayer` `:206-208` | PASS |
| `GoBackClick` → menuMode 1/12 (`UIWorldSelect.cs:118-122`) | `WorldBack` `:184` | PASS (single-player arm only, as the port is single-player) |
| `PlayGame` → `LoadWorld` (`UIWorldListItem.cs:360-374`) | `WorldPlay(i)` `:186` + `apply_front_flow` `:1047-1051` + `world_play_click` (`IsValid` + `TryMovingToRejectionMenuIfNeeded`, `:362`/`:387-410`) | PORTED — the gates land in `world_select::rejection_key` + `flow.rs::world_play_click` (S1-H-10); the rejection surfaces `Main.statusText` via `App::front_hint` (no `menuMode == 1000000` page) |
| `DeleteButtonClick` → menuMode 9 (`:346-358`) | `WorldDelete(_)` `:187` | PASS |
| Mode 9 Yes/No → reopen world select (`Main.cs:49684-49692`) | `ConfirmYes/No` on `ConfirmDeleteWorld` `:209-211` | PASS |
| `NewWorldClick` → 888 `UIWorldCreation` (`UIWorldSelect.cs:110-116`) | `WorldNew(worlds, active)` `:191-197` | PASS — the stand-in name and the Journey default (`SetDefaultOptions` `:313-326`) ride in |
| `Click_GoBack` → `OpenWorldSelectUI` (`UIWorldCreation.cs:665-674`) | `WorldCreateBack` `:200` | PASS — no discard confirm (the draft is choice-only) |
| `FinishCreatingWorld` (`:845-863`) | `WorldCreate` `:203` (the App ran `create_world` first) | PASS — minus the `UIWorldLoad` screen |
| char creation Back while dirty → mode 40 (`UICharacterCreation.cs:1210-1222`) | `Click::Back` → `apply_front_flow`'s dirty park (`flow.rs:2569`) | PASS — `front_backup` is the parked screen |
| Mode 40 Yes → char select (`Main.cs:49454-49459`) | `ConfirmYes` on `ConfirmDiscard` `:214-216` | PASS |
| Mode 40 No → restore `BackupConfirmationState` (`Main.cs:49460-49467`) | `ConfirmNo` on `ConfirmDiscard` `:219` | PASS — the exact parked screen (test `:1237`) |
| The rename/naming keyboards' `_cancelAction` (`UICharacterListItem.cs:214-216`, `UIWorldListItem.cs:428-430`) | `keyboard_cancel` `:1548-1564` | PASS for the reopen; the rename *submit* only notices (no write-back) |

### ROADMAP DONE-claim re-verification

| Row | Claim | Verdict | Evidence |
| --- | --- | --- | --- |
| M7-UI-1 | pointer machine + `GetElementAt` walk + double-click, over both select screens | PASS | `ui/pointer.rs` (432 lines, 12 tests), `ui/framework.rs::get_element_at` `:64`, both screens' `elements`/`hit_element` (`char_select.rs:581/:619`, `world_select.rs:545/:583`), double-click test `flow.rs:1420` |
| BUG-02 | char-select preview draws the saved character | PASS | `char_select.rs:749` calls `draw_player_preview`; `save_list::Appearance` `:59-105` + the `.plr` read `:291-317`; test `a_disk_card_carries_its_plr_appearance` |
| BUG-03 | `.plr` favourites round-trip | PASS | `player_file`'s `inv_favorites` on both models, `PlayerEntry.inv_favorites` `:45`, `save_created_character`'s write `:166`; test `a_disk_card_carries_its_plr_appearance`'s star assertion |
| BUG-04 | Play loads the selected world | PASS | `App.play_world` `:355`, `apply_front_flow` `:1047-1051`, `frame.rs:303-326`, `swap_world` `:1630`; test `world_play_parks_the_selected_file` `:2809` |
| BUG-05 | boot loads the player's newest own world | PASS | `latest_own_world` `:3746-3767` |
| BUG-17 | front-end click/hover cues | PASS | `handle_front_click`'s cue `flow.rs:1017-1027`, the hover tick `:1295-1320`, the legacy menu's ticks `loop.rs:383/:466/:476`; tests `:1267`/`:1294` |
| fix-world-select-clicks | the 200 ms lockout decays on a live front screen | PASS | `tick_pointer_clock` `:1281-1284` called unconditionally at `frame.rs:105`; six regression tests `flow.rs:1534-1712` |

## Tests

Static `#[test]` counts (no cargo run per the research rules):
`app/flow.rs` 46, `ui/char_select.rs` 17, `ui/char_creation.rs` 23,
`ui/world_select.rs` 17, `ui/world_creation.rs` 13, `ui/keyboard.rs` 13,
`ui/save_list.rs` 15, `ui/pointer.rs` 12, `ui/framework.rs` 5,
`ui/select_hit.rs` 0; `app/session.rs` has no test module of its own (its
paths are covered by `flow.rs`'s click tests).

Notable coverage (file::test):

- Flow/state machine: `flow.rs::the_flow_table_routes_every_front_hop`
  `:1133`, `confirm_resolutions_reopen_their_screen` `:1206`,
  `the_mode_40_no_branch_restores_the_exact_parked_screen` `:1237`,
  `front_and_menu_are_never_both_some` `:1328`,
  `dw_2_3_a_double_click_press_routes_to_play` `:1420`,
  `tr_ui_names_map_the_hook_table` `:1446`
- Click-gate regressions (fix-world-select-clicks):
  `world_select_play_answers_after_the_entry_lockout_expires` `:1534`,
  `world_select_card_body_and_back_answer_after_the_lockout` `:1581`,
  `world_select_rename_row_opens_the_keyboard_after_the_lockout` `:1622`,
  `world_select_delete_row_parks_the_confirm_page_after_the_lockout` `:1640`,
  `world_select_new_and_favorite_rows_answer_after_the_lockout` `:1663`,
  `entry_lockout_expires_on_a_live_front_screen` `:1694`
- BUG-17: `bug17_a_front_click_plays_the_activation_cue` `:1267`,
  `bug17_the_hover_edge_plays_the_menu_tick_once` `:1294`
- Creation: `new_click_opens_a_fresh_creation_screen` `:2490`,
  `creation_clicks_mutate_the_draft` `:2520`,
  `clean_creation_back_reopens_the_char_select` `:2554`,
  `dirty_creation_back_parks_the_mode_40_round_trip` `:2567`,
  `naming_keyboard_round_trips_into_the_draft` `:2605`,
  `create_with_an_empty_name_opens_the_naming_keyboard` `:2639`,
  `create_with_a_name_records_the_draft_as_the_session_player` `:2666`
- World creation: `world_new_click_opens_the_creator_with_the_stand_in_name`
  `:2120`, `world_creation_option_clicks_update_the_draft` `:2162`,
  `world_creation_create_with_an_empty_name_opens_the_naming_keyboard` `:2252`,
  `world_creation.rs::create_world_writes_a_parsable_trwld` `:1423`,
  `two_created_seeds_generate_two_worlds` `:1453`
- Keyboard: `keyboard_submit_reopens_with_the_not_ported_notice` `:2354`,
  `keyboard_enter_gates_on_the_validation` `:2364`,
  `the_chest_keyboards_save_click_submits_the_empty_text` `:2432`
- Geometry/tables: `char_select.rs::layout_matches_the_source_table_at_1600x900`
  `:923`, `char_creation.rs::layout_matches_the_source_geometry_at_1600x900`
  `:2076`, `world_creation.rs::layout_matches_the_source_geometry` `:1258`,
  `keyboard.rs::layout_matches_the_source_table_at_1600x900` `:997`,
  `key_rect_matches_the_create_keyboard_button_oracle` `:1038`,
  `world_select.rs::card_layout_matches_the_source_rows` `:988`
- Listing/erase: `save_list.rs::list_players_reads_real_plr_files` `:634`,
  `erase_world_removes_the_world_and_its_sidecars` `:702`,
  `erase_player_removes_the_file_bak_and_sibling_dir` `:723`,
  `sort_players_favourites_first_then_name` `:746`,
  `sort_worlds_follows_the_vanilla_key_chain` `:769`

Screenshot hooks: `TR_UI=charsel|worldsel|charnew|worldnew|vk|credits|delplayer|delworld|discard`
(`TrUiPreset` `flow.rs:229-246`, test `:1446`) — the deterministic captures the
P6 screenshots pair against.

## Deviations & risks

Legend: cosmetic / behavioural / determinism / pixel. "Admitted" = already
written in ROADMAP/ANALYSIS/PORT_MAP.

1. ~~**`IsValid` and the creative/journey rejection menu are missing on Play.**~~
   **FIXED 2026-11-13** (`S1-H-10-world-select-gates`):
   `world_select::rejection_key` ports `TryMovingToRejectionMenuIfNeeded`
   (`UIWorldListItem.cs:387-413` — the invalid `GameModeID` arm, the
   Journey-player/non-Journey-world arm and its mirror) and
   `flow.rs::world_play_click` runs `IsValid` (already gated by
   `worlds[i].valid`, so that half of the claim was right) then the rejection
   before `WorldPlay`, playing `SoundEngine.PlaySound(10)` and surfacing
   `Main.statusText` through `App::front_hint` (the port has no
   `menuMode == 1000000` page). `card_hit`/`elements` still offer the Play
   button for an invalid/mismatched card, as the C# does — the click is what
   rejects.
2. **`ANALYSIS.md:2120-2124` is stale** — it still says "No `.plr`
   reader/writer… Create records the session draft instead of writing a
   character file", while `list_players` reads real `.plr`s
   (`save_list.rs:291`) and `save_created_character` writes one
   (`session.rs:141-176`). Severity: doc drift (behavioural risk: a future
   agent re-ports a reader). Not admitted. The same stale sentence survives in
   `flow.rs:73`'s `KeyboardTail::Rename` doc and in the user-visible rename
   notice `app/mod.rs:1482` ("no save-file writer") — the writer exists, what
   is missing is a *read-modify-write* of a loaded player.
3. **`PORT_MAP.md:180` is stale** — "partial (no `.plr` reader; metadata
   stand-ins)" while the reader is live and the port writes players.
   Severity: doc drift. Not admitted.
4. **Rename writes nothing.** `KeyboardTail::Rename` only sets `front_hint`
   (`mod.rs:1481-1484`), whereas `UICharacterListItem.RenameButtonClick` →
   `GameContents.PlayerRename` rewrites the `.plr` and `Main.LoadPlayers`
   re-lists (`Main.cs:49431`'s reopen). Severity: behavioural. Admitted
   (ANALYSIS's P6 list implies it via §9.4, but not stated verbatim).
5. **World metadata stand-ins**: no GameMode (every entry Classic →
   `CanWorldBeJoinedByActivePlayer`'s Journey gate is vacuous, `save_list.rs:426-433`),
   favourites are a `<world>.trfav` sidecar (`Main.LocalFavoriteData` has no
   port), creation time is the mtime, no play-time, the icon is a fixed
   `IconCrimson`, invalid worlds bucket to Small instead of "Unknown",
   `erase_world` deletes the orphan `.map` and there is no recycle bin. All
   admitted (ANALYSIS `:2125-2134`). Severity: behavioural (a Journey world
   created in-game is later listed and joined as Classic).
6. **The card status overlays are pinned but never drawn** —
   `IconCompletion`/`IconMixedSeed`/`IconNewlyGenerated`
   (`UIWorldListItem.cs:71-93`, `:186-199`); `world_select::overlay_rects`
   `:353` is `#[allow(dead_code)]` with tested geometry only. Severity:
   cosmetic/pixel. Admitted in code (`:343-350`), **not** in ANALYSIS.
7. **The creator's Randomize/Copy/Paste arms do nothing.** ✅ **Fixed
   2026-11-13 (`S1-H-11-randomize-copy-paste`).** `click_at` now routes all six
   buttons (`CopyHex`/`PasteHex`/`RandomizeColor`, `CopyTemplate`/
   `PasteTemplate`/`RandomizePlayer`), porting `Click_RandomizePlayer`
   (`UICharacterCreation.cs:1545-1657`), `Click_RandomizeSingleColor`
   (`:1779-1787`), the hex path (`Click_CopyHex`/`Click_PasteHex`/`GetHexColor`,
   `:1331-1347`, `:1764-1777`) and the template path
   (`GetPlayerTemplateValues`/`Click_CopyPlayerTemplate`/`Click_PastePlayerTemplate`,
   `:1349-1412`, `:1414-1510`); `ui/clipboard.rs` is the new
   `Platform.Get<IClipboard>()` bridge. The historical gap was: `HoverTarget`
   carried `CopyHex`/`PasteHex`/`RandomizeColor`/`CopyTemplate`/`PasteTemplate`/
   `RandomizePlayer` (`char_creation.rs:1286-1294`) but `click_at`
   (`:1233-1278`) routed none of them — `Click_RandomizePlayer` was unported and
   there was no clipboard bridge. Severity: behavioural (feature absent).
8. **`GameTipsDisplay` is not ported** (`UICharacterCreation.cs:170`,
   `:1998-2005`). Severity: cosmetic. Admitted (ANALYSIS `:2161-2162`).
9. **No loading screen.** `UIWorldLoad`/`UIGenProgressBar`/`UIWorldGenDebug`
   have no counterpart, so `FinishCreatingWorld` reopens the world select
   instead of showing progress (`session.rs:43-49`) and Play swaps the world
   in-frame (`frame.rs:303-326`, ~51 ms after BUG-21). Severity: cosmetic +
   behavioural (no cancel/abort path for a slow generation, and the abort cue
   `UIWorldLoad.cs:54-58` has no port). Admitted (ANALYSIS `:2148-2149`).
10. **Random world name and blank seed are deterministic stand-ins.**
    `stand_in_name` = `"World {n+1}"` (`world_creation.rs:103-105`) and a
    blank seed hashes the world *name* through CRC-32
    (`seed_value` `:697-702`) instead of `Main.rand`, so two worlds created
    from the same name generate the same terrain and the same file name
    collision surface. Severity: determinism (by design). Admitted (ANALYSIS
    `:2163-2169`).
11. **`AssignRandomWorldName` cannot re-roll a renamed draft.** The stand-in
    re-derives `"World {n+1}"` (`:97-99`), so the C#'s re-randomisation of an
    explicitly named world (`UIWorldCreation.cs:1002-1025`) is a no-op after
    an edit. Severity: cosmetic. Admitted in code, not in ANALYSIS.
12. **The world sort's active-player key is hardcoded false** —
    `enter_front` calls `sort_worlds(&mut self.worlds, false)`
    (`mod.rs:1368`) because no active player exists at that hop; the C# reads
    `Main.ActivePlayerFileData.Player.difficulty` (`UIWorldSelect.cs:177-180`).
    Consequence: once `.trwld` carries GameMode, the sort and the Play
    rejection stay wrong until this is wired. Severity: behavioural (latent).
    Admitted in code (`save_list.rs:426-433`) only.
13. **`sort_players`/`sort_worlds` have no `LastPlayed` key** — insertion
    order stands in (`save_list.rs:391-424`), and the filename tie-break
    compares the path component. Severity: behavioural. Admitted in code.
14. **Name/rename caps and the display trims**: the keyboards cap at 20/27
    (`char_creation.rs:1200`, `world_creation.rs:37`) and
    `TrimDisplayIfOverElementDimensions` is not ported, so a 27-char world
    name overflows its plate. Severity: pixel. Admitted (ANALYSIS `:2142-2143`).
15. **Keyboard: desktop layer only, plus the two font-metric trims.**
    `ShowGamepadHints` is pinned false (`keyboard.rs:1-31`; `TR_VK_GRID=1`
    restores the grid), the textbox/label `MinWidth` growth and the `HAlign`
    switch at >1600 px are documented off. Severity: cosmetic. Admitted.
16. **No gamepad snap points** (all `SetSnapPoint` sites, e.g.
    `UIWorldListItem.cs:205`), so Escape/no-key navigation on the front screens
    is absent; `FrontScreenBehaviours::key` (`flow.rs:915-944`) only routes the
    keyboard's text/Backspace/Enter/Escape. Severity: behavioural. Admitted
    (ANALYSIS `:2138-2139`).
17. **Legacy create flows (menuModes 2/3/7/8/16/17-24/222/−7/−71/5000) and the
    palette renderer are unported**; menuMode 10 is a pass-through. Severity:
    cosmetic (dead code from the modern menu). Admitted (ANALYSIS `:2152-2154`).
18. **Credits is a stub** — the world backdrop plus one Back row; the
    `SkyManager.Activate("CreditsRoll")` cinematic and the rolling credits are
    unported. Severity: cosmetic. Admitted (ANALYSIS `:2150-2151`).
19. **The confirms are legacy pages, not inline states** — the App runs the
    erase and the reopen (`flow.rs:1056-…`) instead of vanilla's inline
    `selectedMenu` branches, and the Yes/No rows are 2-row lists rather than
    the C#'s positioned confirm. Severity: cosmetic (the mode-40 round-trip is
    bit-faithful, test `:1237`). Admitted (ANALYSIS `:2155-2160`).
20. **The screen-switch cue is a family heuristic, not per-site.**
    `handle_front_click` plays `Menu_Close` for the four Back/No rows and
    `Menu_Open` for everything else (`flow.rs:1017-1027`), whereas vanilla
    plays the cue at each click site (`UICharacterSelect.cs:110/:118`,
    `UIWorldListItem.cs:365`, the confirms' `Main.cs:49434`/`:49681` — the
    confirms stay silent in the port because `menu/pages.rs` never calls
    `audio`). Severity: cosmetic. Partially admitted (ANALYSIS `:2139-2141`
    still says the front-end clicks are silent, which BUG-17 corrected for the
    front screens only).
21. **`Color` multiply wraps, not saturates** (`color.rs`'s pinned convention
    — the `DarkGray * 1.2` Master bunny). Severity: pixel, by design. Admitted
    (ANALYSIS `:2170-2172`).
22. **Session-entry hydration** — the session card always reports
    `inv_favorites = 0` and `mana_max = 20` and takes `life_max` from the live
    player (`session.rs:71-110`), while a same-named disk card replaces it
    (`save_list.rs:321-329`). Minor, undocumented in ANALYSIS (code comments
    cover it). Severity: cosmetic.

## Task seeds

S — small (single file / one table)

- **S1. Fix the three stale doc claims** — ANALYSIS `:2120-2124` (the `.plr`
  reader/writer), PORT_MAP `:180` ("partial, no `.plr` reader"), and
  `flow.rs:73`'s + `mod.rs:1482`'s "no save-file writer" notices (the writer
  exists; the gap is the rename read-modify-write). Docs-only, no behaviour.
- ~~**S2. Gate Play on validity.**~~ **DONE 2026-11-13** (`S1-H-10`):
  `world_select::rejection_key` + `flow.rs::world_play_click` reproduce
  `IsValid` + `TryMovingToRejectionMenuIfNeeded` (`UIWorldListItem.cs:362`,
  `:387-410`) and surface the rejection through `front_hint` (no
  `menuMode == 1000000` page). Tests `rejection_key_ports_try_moving_to_rejection_menu_if_needed`
  + `world_select_play_gates_validity_and_the_journey_mismatch` (invalid entry,
  both Journey mismatches, the agreeing pair).
- **S3. Wire the active player's difficulty into the world list** — pass the
  session/selected player's difficulty into `enter_front`'s `sort_worlds`
  (`mod.rs:1368`), matching `UIWorldSelect.cs:177-180`. Blocked by S6 (needs
  GameMode metadata) for observable effect; the plumbing itself is local.
- **S4. The rename display trims** — port
  `TrimDisplayIfOverElementDimensions(27)`/`(20)` for the world/character
  rename labels (`UIWorldListItem.cs:418`, `UICharacterListItem.cs:199`).

M — medium

- **M1. `.plr` rename read-modify-write** — read the card's `.plr` through
  `player_file::read_player_file`, rewrite the name, re-list; replaces the
  "Renaming is not ported yet" notice (`mod.rs:1481-1484`) and closes
  deviation 4. Depends on S1's wording.
- **M2. The creator's Randomize/Copy/Paste arms** — port
  `Click_RandomizePlayer` (`UICharacterCreation.cs:1545-1657`) and the
  clipboard pair (`GetPlayerTemplateValues` `:2006-2021`) behind a clipboard
  bridge (the SDL/winit clipboard); the hover targets already exist
  (`char_creation.rs:1286-1294`). Depends on M1's draft-copy plumbing.
- **M3. `GameTipsDisplay`** — the rotating hint line under the creator panel
  (`UICharacterCreation.cs:170`, `:1998-2005`) and the `UIWorldLoad` tip
  (`UIWorldLoad.cs:117-118`); needs the `GameTips` data table (the strings
  ride `Lang.tip`-adjacent tables) and a per-screen timer.
- **M4. Random world-name composition** — load the
  `RandomWorldName_Composition/Adjective/Location/Noun` tables and the
  1-in-10000 `SpecialWorldName.TheConstant` roll, replacing
  `stand_in_name` (`world_creation.rs:103-105`); keep the C# RNG order so a
  re-roll sequence matches.

L — large

- **L1. `UIWorldLoad` + `UIGenProgressBar` as a sixth `FrontScreen`** — the
  612x70 bar, the 570/508 px fills, the `WorldGenerator.CurrentController`
  progress feed and the Inventory-abort cue
  (`UIGenProgressBar.cs:42-86`, `UIWorldLoad.cs:46-120`); needs
  `tr_app`'s worldgen pipeline to expose total/current progress (the pass
  count) and to become resumable/cancellable. Then `finish_creating_world`
  hands off to it instead of reopening the world select (`session.rs:50-62`).
- **L2. `.trwld` metadata fields** — GameMode, evil, creation time, play
  time, Moon-Lord/seed flags in the port's `tr_core::save` header (a version
  bump, with the existing reader as the migration shape) plus the matching
  `WorldEntry` fields; unlocks deviations 5/6/12 (the Journey gate, the card
  overlays, the sort key) and S3. Depends on the save-format module's review.
- **L3. Cloud save UI** — the cloud favourite buttons and
  `IsCloudSave`/`MoveToCloud` arms (`UIWorldListItem.cs:113-124`, `:336-343`)
  behind a `SocialAPI.Cloud`-shaped port seam; currently the deliberate
  `Cloud == null` branch (ANALYSIS `:2135-2137`).
