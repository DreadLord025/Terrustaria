# 90 — DOC-SWEEP lane 14: menu/UI sounds (W4-g22 continuation)

Branch `task/W4-g22-sounds-main-2`, base `e0f81135` (the 90/90 window). This
pass ports the **recorded unported surfaces** of the menu/UI sound lane: the
888 inline cues of `UICharacterCreation`/`UIWorldCreation`, the
connecting-cancel cue, the corrupt-world recovery pages 200/201, and the
generic hover/click census across the front screens. **57 sites**, commits
`1b4761f7`…`aed3cc05` (5 sites per commit; census-only tail bundled). All
witnesses are **written, not run** (code-only lane — no cargo in this phase).

## 1. The 888 inline-cue census (UICharacterCreation / UIWorldCreation)

The g22 window (`Main.cs` sites 91-180) explicitly deferred these to a
separate census ("the 888 inline cues stay a separate census", `56017102`).
Every `SoundEngine.PlaySound` site in the two UI files is now accounted for:

### UICharacterCreation.cs — 24 sites

| C# site | Handler | Cue | Port |
| --- | --- | --- | --- |
| `:532/:538/:549/:560/:571/:582/:599` | the seven `EquipArmor*` | 12 | CENSUS-ONLY — `char_creation::armor_assignment` + `armor_click_cue` (the armor preview state is unported; the Formal/Swimming gender split kept). MENU-FINAL correction: only FIVE are subscribed (`:355/:362/:369/:376/:383`) — layout order `[None, Hallowed, Silver, Formal, Swimming]` = `ARMOR_LAYOUT_HANDLERS [0,4,2,5,6]`; Gold/Funeral are dead handlers; Swimming female carries `Accessory1Item 208` (`:611`, `armor_assignment_accessory`) |
| `:1042` | `Click_VoicePitch` | 12 | CENSUS-ONLY here — SUPERSEDED: the voice state landed on `task/W4-e10-player-sounds-wave` (`b3a82c9d`), whose flow.rs `Click::VoicePitch` arm already plays `Menu_Tick` at the C# site; `voice_pitch_click_cue` stays as this lane's census row |
| `:1212` | `GoBack` | 11 | WIRED — `click_cue(Click::Back)`, played before the dirty-check/mode-40 park |
| `:1226` | `FadedMouseOver` (Back/Create) | 12 | WIRED — `hover_element` ids 10/11 through `front_hit` + the loop's Bug-17 edge |
| `:1239/:1252/:1261/:1270` | `Click_ColorPicker`/`ClothStyles`/`HairStyles`/`CharInfo` | 12 | WIRED — `click_cue(Click::Category(_))` |
| `:1288` | `Click_CharClothStyle` | 12 | WIRED — `click_cue(Click::ClothStyle(_))` |
| `:1333/:1339` | `Click_CopyHex`/`Click_PasteHex` | 12 | WIRED (a failed paste still played — the cue precedes the parse) |
| `:1410/:1416` | `Click_CopyPlayerTemplate`/`PastePlayerTemplate` | 12 | WIRED |
| `:1547` | `Click_RandomizePlayer` | 12 | WIRED (before the `Main.rand` rolls) |
| `:1661/:1670` | `Click_Naming`/`Click_NamingAndCreating` | 10 | WIRED — played before the keyboard opens / the create finishes |
| `:1781` | `Click_RandomizeSingleColor` | 12 | WIRED |
| `:2370` | `HandleBackButtonUsage` (gamepad) | 12 / 11 via `GoBack` | CENSUS-ONLY — `back_button_usage_cue` (gamepad snap points out of scope §J.3) |

Element-level sites feeding the same screen (ported with it):

| C# site | Element | Cue | Port |
| --- | --- | --- | --- |
| `UIDifficultyButton.cs:53/:84` | hover latch / click | 12 / 12 | WIRED — `hover_element` ids 12-15 + `click_cue(Click::Difficulty)` |
| `UIHairStyleButton.cs:47/:82` | hover latch / click | 12 / 12 | WIRED — `hover_element` ids 20+ (clipped to the list view, the DrawSelf-only-drawn rule) + `click_cue(Click::Hair)` |
| `UIClothStyleButton.cs:71` | hover latch | 12 | WIRED — `hover_element` ids 300+ |
| `UIColoredImageButton.cs:95` | category + clipboard-trio MouseOver | 12 | WIRED — `hover_element` ids 0-9 / 320+ / 330+ |
| `UICharacterNameButton.cs:69` | name-plate hover latch | 12 | WIRED — `hover_element` id 100 |
| `UIColoredSlider.cs:136/:85` | `_alreadyHovered` / `_soundedUsage` | 12 / 12 | WIRED — `CreationState::hsl_hovered/hsl_sounded` + `hsl_slider_cues`, driven by the draw's highlight answer through the new `FrontDraw::audio` seam |

### UIWorldCreation.cs — 7 sites (+ the element 12s)

| C# site | Handler | Cue | Port |
| --- | --- | --- | --- |
| `:672` | `GoBack` | 11 | ALREADY WIRED (Bug 17's `FlowClick::WorldCreateBack` hop) — `click_cue(Click::Back)` answers `None` so the cue is never doubled |
| `:678` | `FadedMouseOver` (Back/Create) | 12 | WIRED — `hover_element` ids 40/41 |
| `:691` | `Click_SetName` | 10 | WIRED — `click_cue(Click::Name)` |
| `:704` | `OpenSeedInputMenu` | 10 | WIRED — `click_cue(Click::Seed)` |
| `:712` | `Click_NamingAndCreating` | 10 | WIRED — `click_cue(Click::Create)` |
| `:834` | `OnFinishedSettingSeed` special-option hit | 24 (Chat) | WIRED — the `KeyboardTail::WorldSeed` submit tail plays `"Chat"` when `seed_flags(text) != none` (`tr_core::seeds` is the `GetOptionFromSeedText` resolution) |
| `GroupOptionButton.cs:255/:216` over `:231/:258/:393/:439/:484` | the two plate icons + ten option rows, click/hover | 12 / 12 | WIRED — `click_cue(RandomName/SeedIcon/Size/Difficulty/Evil)` + `hover_element` ids 0-1/10-32. The `SeedIcon` split keeps the icon's button-12 vs the plate's handler-10 distinction (the Advanced tab itself stays the documented deviation) |
| `UICharacterNameButton.cs:69` over `:236/:264` | the two plates' hover | 12 | WIRED — `hover_element` ids 2/3 |

The secret-seed `Enable` trackable sound (`WorldGen.cs:539`,
`SoundID.MenuAccept`) and `TryApplyingCopiedSeed`'s copied-seed path
(`WorldFileData.cs:270`) stay with the unported Advanced tab — recorded, not
silently dropped.

## 2. Connecting-cancel (Main.cs:48621 / :48822)

The menuMode 14 / 15 overlay's Cancel (click or gamepad Back) plays 11,
disconnects and returns to menuMode 0 / netMode 0. The port had **no cancel
path at all** — the overlay only ate text. Now:

- `Menu::activate`'s connecting guard runs BEFORE the header gate (the
  overlay's rows replace the page's, `:48604-48612`); every click answers
  `Action::CancelConnect` with `Menu_Close` queued.
- `Menu::key`'s Escape arm answers the same while connecting.
- `App::cancel_connect` drops the client role (the socket close), clears
  `join_target`, returns `Page::Main`.

## 3. Corrupt-world recovery 200/201 (Main.cs:48843/:48849/:48857/:48876)

The world-load failure path was a bare `front_hint`; the vanilla opens a
page (`WorldGen.playWorldCallBack`, `WorldGen.cs:6766-6790`). Now:

- `Page::LoadFailedBackup` (200) / `Page::LoadFailedNoBackup` (201) with the
  exact rows (`Lang.menu[9]/[10]/[6]` and `[9]/[11]/[5]`, the 201 middle row
  a non-clickable heading `array[1]=true`), layouts `num2 = 220` (`-= 30`),
  `array4` 70/-30/50; `TR_PAGE=200|201` screenshotable.
- The frame's swap-Err arm probes the `.bak` sidecar and opens the page
  (`Menu::open_load_failed` parks the path). The port's single `load_world`
  attempt stands in for the C#'s double retry (recorded deviation).
- Load Backup (`App::load_backup`): `.bak` exists → `.wld`→`.bad`,
  `.bak`→`.wld`, play 10, re-park `play_world` (`WorldGen.playWorld` +
  menuMode 10); absent → play 11, Main. Cancel/Back/Escape play 11, Main.
- The rolling-backup WRITER is still absent (research-90 42-3) — the probe
  reads whatever `.bak` exists, exactly the C#'s `FileUtilities.Exists`.

## 4. g23 claims — read, skipped

`origin/task/W4-g23-sounds-main-3` (`72879996`) owns `Main.cs:49734-66248`
(the 75-site DrawMenu tail census + 14 pure tail halves in
`main_sounds_batch3.rs`). No overlap with this lane's UI-file sites; nothing
taken.

## 5. Generic hover/click census (UIElement.MouseOver sound-12)

d81's `GroupOptionButtonState::hover_sound_tick` lives on
`origin/task/W4-d81-ui-elements-1` (unmerged); this branch's coverage runs
through the existing Bug-17 edge (`app/loop.rs`) fed by `App::front_hit`:

- **CharSelect/WorldSelect**: Back/New (`FadedMouseOver`) and the card
  buttons (`UIImageButton.MouseOver`, `UIImageButton.cs:61`) tick —
  **the card BODIES no longer do**. The census found the port ticking on
  body enter, but `UICharacterListItem.MouseOver` (`:294-300`) and
  `UIWorldListItem.MouseOver` (`:475-479`) are colour-only. The new
  `App::front_hit_sounds` gate keeps the ids for the pointer machine
  (double-click needs them) while silencing the body's tick.
- **CharCreation/WorldCreation/Keyboard**: `front_hit` now answers the
  per-screen `hover_element` tables (bases +1000/+2000/+3000 so a screen
  switch never reads a stale latch as the same element). The keyboard's
  table lists ONLY Submit/Cancel (`FadedMouseOver` `:1071`) — the grid keys'
  `CreateKeyboardButton` OnMouseOver (`:637-644`) and shift/symbol handlers
  are colour-only.
- **On-screen keyboard keys**: Shift/Symbol/letters/Space/Backspace play 12
  before the edit (`:252/:286/:525/:499/:394`) — wired in the
  `FrontScreen::Keyboard` click arm. Submit/Cancel keep their gated 10/11
  inside `keyboard_submit`/`keyboard_cancel` (already landed).
- The legacy menu's row hover (`DoStatefulTickSound`) and the emote wheel's
  cell hover were already wired (g21/Bug 17) — verified, untouched.

## Witnesses (written, not run)

`char_creation.rs`: `doc_sweep_category_and_cloth_clicks_play_12`,
`doc_sweep_hover_elements_answer_stable_ids`,
`doc_sweep_hsl_hover_and_usage_latches`,
`doc_sweep_armor_voice_and_gamepad_back_census`;
`world_creation.rs`: `doc_sweep_world_creator_click_cues`,
`doc_sweep_world_creator_hover_elements`;
`keyboard.rs`: `doc_sweep_only_submit_cancel_hover_sounds`;
`app/flow.rs`: `doc_sweep_a_category_click_plays_one_tick`,
`doc_sweep_naming_and_back_play_their_own_cues_once`,
`doc_sweep_difficulty_and_randomize_play_one_tick`,
`doc_sweep_creator_hover_edge_ticks_once_per_enter`,
`doc_sweep_world_creator_rows_and_plates_play_once`,
`doc_sweep_special_seed_submit_plays_the_chat_cue`,
`doc_sweep_keyboard_keys_tick_once`,
`doc_sweep_card_bodies_do_not_tick_but_buttons_do`;
`menu/mod.rs`: `doc_sweep_connecting_cancel_queues_eleven`,
`doc_sweep_load_failed_pages_queue_their_own_cues`;
`app/clicks.rs`: `doc_sweep_cancel_connect_drops_the_role_and_returns_main`,
`doc_sweep_load_backup_probe_moves_and_plays`.

## MENU-FINAL — the armor handlers' activation contract (co-lane read)

Read `origin/task/W4-d85-ui-screens-2` (`aeadb43a`) and local
`task/W4-e10-player-sounds-wave` (`b3a82c9d`, unpushed): d85 draws the
armor row (`Layout::armor_buttons`, the five rects) but keeps NO equip
state — the draft has no `miscEquips`; e10 landed the VOICE half
(`draft.voice_variant/voice_pitch_offset`, `Click::VoicePitch` plays 12
in its flow.rs arm, the preview seam `play_voice_preview`) and its
`Click::ClothStyle` arm documents "the armor guard is out of scope (no
armor preview)". **Equip state has not landed on any lane** — the seven
handlers stay census-only. The contract for whoever ports the preview:

1. Hit-test the five live rects in `Layout::armor_buttons` order; map
   through `ARMOR_LAYOUT_HANDLERS` (`[0,4,2,5,6]`) — never index the
   census by layout position directly (Gold/Funeral are dead).
2. Play `armor_click_cue(handler)` = 12 BEFORE the state write (the C#
   plays first in every handler).
3. Write BOTH genders from `armor_assignment(handler, male)` plus
   `armor_assignment_accessory` — the five shared handlers assign
   `_femaleArmor = _maleArmor`; Formal/Swimming split; Swimming female
   adds accessory 208.
4. `Click_CharClothStyle`'s guard (`:1279-1283`): if the male assignment
   is non-zero, re-enter through `EquipArmorNone` (its OWN 12, `:532`)
   and RETURN — the variant does NOT land and the handler's `:1288` 12
   does NOT play (one cue total, not two).
5. `Click_RandomizePlayer`'s tail resets the armor (`:1653-1656`, the
   e10 `randomize_player_voice_tail` doc comment) — wire the reset there.

## Deviations recorded

- The armor/gamepad 888 sites are census-only helpers (state unported);
  the voice-pitch census row is superseded by e10's live wiring.
- The 200/201 entry runs after ONE load attempt (the C# retries twice).
- The seed-icon click degrades to the plain seed keyboard (Advanced tab
  unported) but keeps the button's own 12.
- The secret-seed trackable `MenuAccept` sound stays with the Advanced tab.