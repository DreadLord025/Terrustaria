# 314 — W4-c25 RETIRE FINAL: the keyboard-text-field census, the f22 sign co-lane compose-verify and the KEYBOARD-FAMILY bible (SEALED)

> **Renumber chain: 281 → 314.** This seal originally took 281 (self-pick per
> §6.3 round 2); research/150 §8.1 ROUND 4 FINAL (W4-d54 `a15cb634b`) adjudicated
> the 281-block self-pick race — §7.2's owner keeps 281, the 10 self-pickers take
> 308–317 in claim order — and assigns c25 **314**. Executed per §8.3: `git mv`
> + header + all refs (INDEX/ANALYSIS/UI/anchors/keyboard.rs comments);
> `git grep` zero-stale at the pushed tip. The three `research/281` short-form
> citations inside `keyboard.rs`'s SEAL comment lines are left byte-identical
> (the keyboard blob `e72b3efe7` is pinned verbatim for the f22 keep-both
> compose, §2); they resolve through this chain 281→314.

Lane: `task/W4-c25-keyboard-text-field`, tip `e5daa96f7` at the seal (2 commits
over base `a725acb04`, the wave-3 landing). Code-only lane: no cargo / tests /
rustfmt / graphify invoked; all witnesses WRITTEN NOT RUN. No merge performed,
no other branch touched.

**Registry number: 314** (chain 281→314 per research/150 §8.1 ROUND 4 FINAL,
d54 `a15cb634b`; the original 281 pick below is the seal-time history).
Originally picked as 281 per `research/150-research-number-registry.md`
§6.3 round 2 (W4-d54 tip `72226e500`): the round-2 block 244–280 is fully
assigned (123 total assignments 151–236 + 244–280, machine-checked), **281+
FREE**, and new lanes must NOT pick "max-on-origin + 1" — a fresh all-tip
sweep was run for this seal (719 refs: every local branch + every
`origin/task/W4-*` and `origin/main` tip, `git ls-tree <tip>:docs/research`
number-key scan): the highest number present at ANY tip is 280; **nothing at
281+**; the brief's provisional "246" is double-taken at tips
(`246-projectile-ai-91-100-final.md` f19 + `246-temple-family-seal.md` g43)
and 247 is g67's executed weather rename — the seal takes the first free
number ≥281 per the registry rule. The registry's cutoff note applies: this
sweep adjudicates every push visible at seal time; a later collision is
renamed per §6.3 (first-add wins).

## 1. Census — what landed on keyboard-text-field

Two commits, both `crates/tr-app/src/ui/keyboard.rs` + docs/anchors only:

| Commit | Scope |
| --- | --- |
| `420fa3ecf` | +494 lines into `ui/keyboard.rs`: the text-entry bindings block after `write_str` (module comment "Text-entry bindings (W4-c25)") + 9 tests inside the existing `mod tests`. |
| `e5daa96f7` | docs+anchors: UI.md text-entry section, PORT_MAP row (`UITextBox/GetInputText/Utils/ReLogic Clipboard -> keyboard.rs`, the row at PORT_MAP:397), ANALYSIS deviations, 2 anchors rows (`pub struct TextField`, `pub fn get_input_text`), the `set_max_length` test + the dead_code allows. |

Symbols (all `pub`, all `#[allow(dead_code)]` until consumers attach):
`TextField` (new/write/set_text/set_max_length/backspace/cursor_left/
cursor_right — `UITextBox.cs:11-78`), `trim_user_string`/`trim_last_character`
(`Utils.cs:2839-2855`), `sanitize_clipboard_text` (ReLogic
`Clipboard.cs:21-32`), `InputTextResult`/`InputKeys`/`BackspaceRepeat`
(private `round_to_even`), `get_input_text` (`Main.GetInputText`,
`Main.cs:20010-20121` — Ctrl else-if chain `:20031-20047`, Shift two-ifs
`:20053-20061`, typed queue `:20067-20083`, append `:20086`, repeat machine
`:20089-20107` with banker's `Math.Round`, trim gate `:20108`, `PasteTextIn`
`:20117-20121`).

C# re-read at the seal (line-exact): `UITextBox.cs` whole (111 lines — the
`Write` insert-then-trim order `:40`→`:46`, the `DrawSelf` caret pin
`_cursor = Text.Length` `:86`), `Main.cs:20001-20125` whole (the `dedServ`/
`AllowInputProcessing` early returns are caller-side, the non-Windows
Escape edge `:20063-20066` folds into the queue flag on the winit path —
recorded), `Utils.cs:2839-2858`, `UISearchBar.cs:30-196` (the 50 cap `:63`,
`SetContents` `:67-89`, the `DrawSelf` GetInputText/flags/SetContents order
`:145-167`), `UIVirtualKeyboard.cs:100-204/:400-499/:700-799/:1025-1064`
(the `Text` setter `:110-121`, the sign context `:166`/`:176`, the
`SetTextMaxLength(maxLength)` `:428`, `BuildSpaceBarArea` `:444-489`, the
DrawSelf GetInputText call `:750` + sign Enter newline `:751-754` + the
`this.Text = text` write-back `:778-788`, `CacheCanceledInput`/
`RestoreCanceledInput`/`CopyTextToSign` `:1031-1059`), `IngameFancyUI.cs:86-185`
(the sign open `:136-145` — `allowEmpty: false`, **1200** cap the census pin).

Deviations STAND as recorded in ANALYSIS "Text-entry bindings (W4-c25)":
char-index caret (not UTF-16 code units; the dangling-surrogate guard
`Utils.cs:2845-2848` unreachable), no ranged selection anywhere in the
sources (copy/cut take the whole string), no `ChatManager.ParseMessage`
snippet-delete arm (`Main.cs:20110-20111` — no text-field consumer produces
chat markup), pure bindings (clipboard in/out via parameters, no OS, no IME
— `imeCompositionActive` caller-side, always false), consumers unwired.
**No `UITextField.cs` exists in the 1.4.5.8 decompilation** — the three
sources above ARE the text field.

## 2. The f22 sign co-lane FINAL compose-verify (keep-BOTH-halves-verbatim)

f22's tip at this seal: **`c727ebc02`** (advanced past the retire-4
compose-verify `a2fdf86fb` and retire-5 `8c1b40a46` by the registry rename
144→264 — `crates/tr-app/src/ui/keyboard.rs` blob **unchanged**
`e72b3efe76bd0551098408a761761255c8996ef8` at both `8c1b40a46` and
`c727ebc02`, `git diff 8c1b40a46 c727ebc02 -- crates/` empty; the rename is
docs-only, the retire-4 instruction stands verbatim).

Read-only `git merge-tree --write-tree c727ebc02 e5daa96f7` → tree
`176c2daabe2db7ad92b8096e43cecda858047f93` (the identical shape at
`8c1b40a46`: tree `287519d7733f53da25ea4f2feba7830bea38d258`):

* **`ui/keyboard.rs` AUTO-MERGES, ZERO conflicts.** Composed blob
  `2400d944b9030fc7326a7cfd406f9ac299f7170e`, 2060 lines. Byte-verified:
  * `git diff --no-index f22-blob composed-blob` = **+516 / −0** — the
    composed file is f22's 1544 lines PLUS c25's insertions, nothing else;
  * the c25 bindings block (HEAD 251–567, 317 lines) sits **verbatim** at
    composed 266–582 (line-by-line compare, zero diffs), inserted after
    `write_str` and before f22's label tables;
  * the c25 test block (HEAD 1639–1842, 204 lines) sits **verbatim** at
    composed 1752–1955, INSIDE the one `mod tests`, before f22's sign
    tests (f22's 101-line sign-test tail verbatim at 1958–2058);
  * stripping exactly those two blocks from the composed file reproduces
    the f22 blob **line-for-line** (1544 lines);
  * ONE `mod tests`, no duplicate top-level symbols, f22's `Context::Sign`/
    `bottom_row`/`layout(screen, hints, context)`/`click_at`/`draw`
    signature changes all present; the c25 bindings call NONE of the
    changed signatures (self-contained; dead_code-allowed).
* **The only real conflicts are the two append races** (both keep-BOTH):
  * `anchors.txt` — f22 adds 13 rows (rename ×3, sign open/tick/submit,
    bottom_row, interactions, SignEdit tail, hover/kill/bubble/tooltip),
    c25 adds 2 rows (`pub struct TextField`, `pub fn get_input_text`).
    Both sides are pure appends over base blob `e06a65905` — keep all 15.
  * `docs/PORT_MAP.md` — f22 edits 5 rows + adds the rename-chain row
    (NPCChatPanel DrawText range, Sign.cs editor chain, the
    **UIVirtualKeyboard superset row**, save_list, MessageBuffer); c25
    adds its ONE row (`UITextBox/GetInputText/Utils/Clipboard ->
    keyboard.rs::{TextField, get_input_text, …}`). Different rows — keep
    f22's superset UIVirtualKeyboard row AND c25's separate UITextBox row.
  * `docs/UI.md` + `docs/ANALYSIS.md` auto-merge (disjoint sections).

**MERGER INSTRUCTION (re-confirmed at the moved tip): keep BOTH keyboard.rs
halves verbatim — the auto-merge IS the correct resolution.** Do not
hand-edit the composed file; resolve only the anchors.txt / PORT_MAP.md
append races by keeping both sides' rows. The char-index caret stays c25's
documented deviation; the sign editor keeps the `KeyboardState` path.

## 3. The chat / sign / text-input compose (f22-NPCChatPanel / sign)

Three owners, zero overlap, one future wiring seam:

* **f22 owns the sign session end-to-end**: `Context::Sign` + the −5000
  textbox park (`UIVirtualKeyboard.cs:176`), the sign-aware NPCChatPanel
  (`ui/npc_chat.rs`: `interaction_lines = count.div_ceil(4)`
  `NPCChatPanel.cs:174` — the d120 patch, verified equivalent at f22
  retire-1b `f35111e43`; `panel_rect`'s live count `:68`; the
  `interaction_rects` `num/4*22` y-step `:106`; the sign caret blinker
  `DrawText :236-262` — the 20-frame `textBlinkerCount` toggle, the last
  line's measured width, IME half skipped), `signs.rs` open/tick/submit/
  cancel/restore + hover/break surfaces, the `KeyboardTail::SignEdit`
  tails. The sign's physical Enter appends '\n' via `write_str` on
  `KeyboardState` (`UIVirtualKeyboard.cs:751-754`), the submit reads the
  live text (`CopyTextToSign` `:1049-1059`), the cap is **1200**
  (`IngameFancyUI.cs:145`, NOT 500).
* **c25 owns the caret field + the GetInputText pass** (§1) — pure
  bindings, no field on App, no draw. Nothing in the c25 block references
  `Context::Sign`, `KeyboardState` mutation paths, or any f22 signature.
* **c24 owns the live search stand-in**: `app/loop.rs`'s
  `journey.search_writing` arm (c24 tip `ff99f1486`) — the chat-style
  `event.text` seam, Enter/Escape stop, Backspace pop, the 50-char cap
  (`UISearchBar.cs:63`); its commit message records "the c25
  keyboard-bindings branch is pure bindings/unwired/not on main — the
  event seam stands in". The chest rename keeps its M3-CHEST-UI keyboard
  path (`KeyboardState`).
* **The ONE future wiring seam** is `UIVirtualKeyboard.DrawSelf:750`'s
  `Main.GetInputText(this.Text, this._editingSign)` call (and
  `UISearchBar.DrawSelf:155`'s twin): when a consumer task attaches
  `get_input_text`, it feeds `InputKeys` from winit, applies the
  `InputTextResult` flags, and writes back through `SetText` semantics.
  Until then **no interaction**: the sign editor keeps `KeyboardState`,
  the search keeps the event seam. The three SEAL witnesses (§5) pin the
  contract at exactly these seams so the attach is mechanical.

The behavioural agreement at the seam (witnessed, §5): `UITextBox.DrawSelf`
pins `_cursor = Text.Length` every frame (`:86`), so at the pinned caret
`TextField` and `KeyboardState`/`write_str` agree char-for-char on append,
backspace, the Enter newline and the 1200/50 overflow trims — the keep-both
merge has no behavioural seam to reconcile.

## 4. The KEYBOARD-FAMILY bible (SEALED, one paragraph)

**f22-sign-CANONICAL** (`ui/keyboard.rs` layout/click_at/draw/Context
0/1/2 + `bottom_row`/`text_valid_for` + the sign session in `signs.rs` +
the sign-aware `NPCChatPanel` with the d120 dynamic interaction lines +
the hover/bubble/tooltip/kill-on-break surfaces + the rename tails
`player_file::rename_player_file`/`save_list::rename_world`, retire FINAL
`research/264`, family COMPLETE — all 28 UIState descendants accounted)
**+ c25-text-entry-CANONICAL** (`TextField`/`get_input_text`/
`BackspaceRepeat`/`trim_*`/`sanitize_clipboard_text`, this seal, pure
bindings, the char-index caret / no-selection / no-ParseMessage /
no-IME deviations standing) **+ c24-search-STAND-IN** (the
`journey.search_writing` event.text arm until a consumer attaches) **+
the M3 chest-rename keyboard path** — merged by keep-BOTH-halves-verbatim
(§2), the only conflicts the anchors/PORT_MAP append races (keep all
rows). The IME/clrInput/blockKey skip is f22-recorded FINAL
(input-system scope); the 1200 sign cap is the census pin; no scroll
exists in `UIVirtualKeyboard` (f22 census). **No further keyboard port
work on any lane**; the remaining moves are consumer attachments (the
search box onto `get_input_text`, any future field onto `TextField`),
each its own task — the keyboard family itself is CLOSED.

## 5. SEAL witnesses — WRITTEN NOT RUN (code-only)

Three new tests appended inside `mod tests` in `ui/keyboard.rs` (before
`panel_bg_truncates_to_the_xna_bytes`), calling ONLY c25-half symbols so
they compose verbatim into the f22 merge (§2's strip-proof extends to
them):

1. `seal_sign_enter_appends_the_newline_outside_get_input_text` — code 13
   sets the flag and never inserts a char (`Main.cs:20071-20074`); the
   sign consumer's post-pass `write("\n")` (`UIVirtualKeyboard.cs:751-754`)
   appends; a full 1200 box swallows the newline (insert-then-trim,
   `UITextBox.cs:40`→`:46`).
2. `seal_textfield_at_the_pinned_caret_matches_keyboardstate` — at the
   `DrawSelf` caret pin (`UITextBox.cs:86`) `TextField` and
   `KeyboardState`/`write_str` agree on append, ×3 backspace, the Enter
   newline and the 1300→1200 overflow (f22's signs.rs witness pins the
   write_str arm; this pins the TextField twin) — the compose has no
   behavioural seam.
3. `seal_search_box_seam_matches_the_50_cap_and_stop_flags` — the c24
   stand-in's contract: the 50 cap (`UISearchBar.cs:63`) trims on the
   Enter frame too (`SetContents` always runs, `:165`), Escape stops
   without reverting the text (`:160-164`).

Prior c25 witnesses (the 9 tests of `420fa3ecf`/`e5daa96f7`, including the
`set_max_length` arm) stand unchanged. 12 c25 tests total on the branch at
the seal commit. All WRITTEN NOT RUN per the lane constraint.

## 6. Receipts

* f22 retire-4 `a2fdf86fb` (the original compose-verify + merger
  instruction) — re-verified at the moved tip `c727ebc02`, keyboard blob
  unchanged, instruction stands (§2).
* f22 retire-5 `8c1b40a46` → registry rename `c727ebc02`
  (`research/144` → `research/264`) — the UI-screens-family COMPLETE
  declaration rides `research/264`; this seal cites it at its executed
  number.
* g58 `research/242` (the chat-portrait TASK-SEED) names f22 the
  sign-aware panel owner per d120's hand-off `2408233da` — consistent
  with §3's ownership split; the portrait remainder is f22's/successor's,
  NOT c25's.
* Registry: `research/150` §6.3 round 2 at d54 `72226e500` — 281 taken
  per the ≥281 rule after the fresh 719-ref sweep (§0); SUPERSEDED by
  §8.1 ROUND 4 FINAL (d54 `a15cb634b`): 281 → **314** (the 281-block
  self-pick race; chain-banner above).
* c24 tip `ff99f1486` — the search stand-in recorded in its own commit
  message; no c24 edit owed by this lane.

**SEALED. No c25 follow-up open.** The consumer attachments (search onto
`get_input_text`, any future caret field) are their own tasks; the merger
executes §2's keep-both instruction.