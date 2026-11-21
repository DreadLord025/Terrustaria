# 91 — g23 re-fold list: g22's post-e0f81135 doc-sweep (57 sites)

> **MENU RETIRE supersession (2026-11-21):** g23d never pushed (tip still
> `945391e3`). The fold DIRECTION flipped — g22 is now the canonical base
> and g23's 4 commits fold into it. Read
> `91-g23-branch-deprecated.md` FIRST; this list's per-commit hunks and
> compose-check map still apply, in reverse (g22 = base, g23 = incoming).

For the **g23d merger**. g23's union `945391e3` merged g22's `e0f81135`
(the 90/90 window). g22 then advanced to **`09761544`** (+16 commits, the
lane-14 doc-sweep, 57 sites, `docs/research/90-doc-sweep-menu-sounds.md`).
`git merge-base --is-ancestor 097615441 origin/task/W4-g23-sounds-main-3`
→ exit 1: the re-fold has NOT happened as of this note. Fold target:
merge `task/W4-g22-sounds-main-2` tip `097615441` (or cherry-pick the
range `e0f811358..097615441`) onto the g23d tip.

## Per-commit fold list (oldest first, `e0f811358..097615441`)

| Commit | Files | What lands |
| --- | --- | --- |
| `1b4761f79` | flow.rs, ui/char_creation.rs, anchors | `Click` category/cloth arms + `click_cue` (sites 1-5, `:1239/:1252/:1261/:1270/:1288` → 12); flow.rs CharCreation click arm plays via `click_cue` |
| `36b2f60ac` | same | `click_cue` Name/Create 10, Back 11, CopyHex/PasteHex 12 (sites 6-10, `:1661/:1670/:1212/:1333/:1339`) |
| `37ae78581` | same | CopyTemplate/PasteTemplate/RandomizePlayer/RandomizeColor/Difficulty 12 (sites 11-15, `:1410/:1416/:1547/:1781`, UIDifficultyButton.cs:84) |
| `81d81eaa3` | same | Hair click 12 + `hover_element` table (ids 0-9 categories / 10-11 Back-Create / 12-15 difficulty / 20+ hair / 100 name / 300+ cloth / 320+ template trio / 330+ hex trio); flow.rs `front_hit` CharCreation arm |
| `e65cdd8dd` | flow.rs, ui/world_creation.rs, anchors | world-creator `click_cue` (RandomName/SeedIcon/Size/Difficulty/Evil 12, Name/Seed 10; the `SeedIcon` split) + flow.rs WorldCreation click arm |
| `ff8a57283` | flow.rs, app/mod.rs, ui/world_creation.rs, anchors | world-creator `hover_element` (ids 0-1 icons / 2-3 plates / 10-32 rows / 40-41 Back-Create); `keyboard_submit`'s `KeyboardTail::WorldSeed` plays `"Chat"` when `seed_flags != none` (`:834`) — **app/mod.rs hunk** |
| `fe2336a3d` | app/clicks.rs, menu/actions.rs, menu/mod.rs, anchors | `Action::CancelConnect` + the connecting guard in `activate`/`key` Escape + `App::cancel_connect`; `pending_cue = Some("Menu_Close")` (`:48621/:48822`) |
| `a65f2a088` | app/clicks.rs, app/frame.rs, menu/actions.rs, menu/mod.rs, menu/pages.rs, anchors | `Page::LoadFailedBackup/NoBackup` (enum tail + layout + `from_tr_page` "200"/"201" + `open_load_failed`), `Menu::load_failed_path` field + init, `Action::LoadBackup/CancelLoadFailed`, `parent` arms, pages.rs rows, `App::load_backup`, frame.rs swap-Err probe arm |
| `d85c8d68b` | flow.rs, anchors | keyboard-key clicks play 12 in the `FrontScreen::Keyboard` click arm (`:252/:286/:525/:499/:394`) |
| `a4daef8fa` | flow.rs, app/loop.rs, ui/keyboard.rs, anchors | `front_hit` Keyboard arm (Submit/Cancel only), `App::front_hit_sounds` gate (card bodies silent), **loop.rs Bug-17 edge** `hit.is_some_and(\|h\| self.front_hit_sounds(h))` |
| `31f622f07` | flow.rs, app/frame/draw.rs, ui/char_creation.rs, anchors | HSL latches (`hsl_hovered`/`hsl_sounded` on `CreationState`, `hsl_slider_cues`); `FrontDraw` gains `audio: &Audio`; **frame/draw.rs** `audio: &self.audio` field in the `FrontDraw` literal (front arm, ~:1548); flow.rs CharCreation draw arm plays the cues after `draw` returns its highlight |
| `7354e58f7` | app/frame.rs, ui/char_creation.rs | witness fixes only (frame.rs menu path qualified; two-bar fixture pre-latch) |
| `ba5355c19` | menu/actions.rs | the connecting guard MOVES before the header gate in `activate` (final position: after the `items.get(index)` let-else, before `if item.header`) |
| `a0e858d4c` | ui/keyboard.rs | keyboard hover census witness |
| `aed3cc05e` | ui/char_creation.rs, anchors | census-only helpers `armor_assignment`/`armor_click_cue`/`voice_pitch_click_cue`/`back_button_usage_cue` + witness (sites 49-57) |
| `097615441` | docs/research/90-doc-sweep-menu-sounds.md | the census doc |

## The pending_cue shape the new sites use

All g22 menu-side sites queue on the SAME single-slot activation queue
g23's union already standardized:

* `Menu::pending_cue: Option<&'static str>` — ids are cue NAMES:
  `"Menu_Open"` (10), `"Menu_Close"` (11), `"Menu_Tick"` (12); drained
  take-once by `Menu::take_cue()` in the event loop after
  click/key/character.
* New activation sites added by the doc-sweep: the connecting guard
  (`activate` BEFORE the header gate + `key` Escape first arm) and the
  200/201 rows (`Act` arms: `CancelLoadFailed` queues `Menu_Close`;
  `LoadBackup` queues NOTHING — `App::load_backup` owns the per-branch
  10/11 after the `.bak` probe; `CancelConnect` queues in the guard).
* The front-end 888 sites do NOT use `pending_cue`: they play directly
  through `App.audio`/`FrontDraw.audio` (the UI states own their audio,
  like vanilla `SoundEngine.PlaySound` inside the handlers). Draw-time
  front cues (the HSL latches) play in `FrontDraw`'s CharCreation arm
  AFTER `draw` returns — the same statement order as g23's chrome drain.

## Disjoint-population contract — compose-check

g23's union contract (`945391e3`): `pending_cue` = ACTIVATION only,
`pending_ticks: Vec<u32>` = DRAW-TIME legacy-menu chrome only
(`:51276` colour-focus, `:51575` volume-hover), drained after
`Menu::draw`. The doc-sweep holds the contract:

1. **No new `pending_ticks` writers.** Every doc-sweep menu cue rides
   `pending_cue`; the draw-time front-end HSL cues never enter either
   menu queue (they are UI-state cues, not `DrawMenu` chrome).
2. **One activation drain stays one drain.** The connecting guard and
   the 200/201 arms write `pending_cue` exactly like g23's `ip_submit`
   arm; `take_cue` semantics unchanged.
3. **Hot-file overlap map** (g22 doc-sweep hunks vs g23-union hunks,
   both against `e0f81135`):
   * `menu/actions.rs`: g22 touches the `Action` enum tail (:32),
     `parent` Credits arm (:146), the `activate` head (:157 — the
     connecting guard), the `Act::Nav` match-tail after ReturnToWorld
     (:294), Escape head (:582) and the 888-page Escape arm (:607).
     g23 touches the `Act::Nav` cue block (:168-218), `Act::ResetKeys`
     (:310), `Act::Hint` (:323) and `submit_join_ip` (:643). The
     :294/:310 pair are 2 lines apart — expect an adjacent-hunk conflict
     in `Act::Nav`'s match tail; keep BOTH (g22's three Action arms +
     g23's ResetKeys/Hint cue writes). The connecting guard must stay
     BEFORE `if item.header` (commit `ba5355c19` moved it deliberately).
   * `menu/mod.rs`: g22's field `load_failed_path` inserts after
     `connecting` (:322); g23's `pending_ticks` field inserts at :332 —
     adjacent struct-field hunks, keep both. g22's test insert (:3297,
     between the connecting-text test and `g22_escape_parent_walk_...`)
     vs g23's inserts (:3191 after the hint test, :3514 after the
     `g22_escape_...` test at EOF) — same tests-module tail region,
     keep all three groups.
     g22's enum/layout/from_tr_page/open_load_failed hunks (:101/:179/
     :507/:558) do not overlap g23's (:607+ machinery, :1003+ chrome).
   * `app/frame/draw.rs`: g22 adds `audio: &self.audio` to the
     `FrontDraw` literal (~:1548, front arm); g23 adds the
     `take_pending_ticks` drain after `menu.draw` (~:1830, menu arm).
     Disjoint arms — no conflict expected.
   * `app/frame.rs`: g22's swap-Err probe (:398) vs g23's
     `drain_new_text_cues` (:478) — disjoint.
   * `app/loop.rs`: g22 edits the Bug-17 front-hover edge (:611); g23's
     union did not touch loop.rs — clean.
   * `app/clicks.rs`, `menu/pages.rs`, `ui/*`, `app/mod.rs`: g23-union
     untouched — clean.
   * `anchors.txt`: both append at EOF (:899+) — keep both blocks
     (g23's 4 rows + g22's 12 rows).
4. **g23's two g22-expectation fixes** (JoinIp-Accept `Menu_Tick`,
   password_back `JoinIp` tuple) live in the `Act::Nav` block g22's
   doc-sweep did NOT touch — take g23's side there wholesale.
