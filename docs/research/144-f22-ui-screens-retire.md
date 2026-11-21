# 144 — W4-f22 UI screens RETIRE: research FINAL + the UI-screens-family COMPLETE declaration

Status: RETIRE (research FINAL; code-only mode — no cargo/tests/rustfmt/graphify
invoked; every witness written not run, verified by re-reading the C#).
Branch: `task/W4-f22-ui-screens-4` (DONE be73b87b5, the retire phase continues
the SAME branch). Retire commits: `1c63c2166` (the d120 INTERACTION_LINES
dynamic-lines fix), `b5162c8ec` (the sign-TILE placement census + the
announcement-box/style text-store verify + the wiring-425 exact-origin fix),
`a2fdf86fb` (the c25 co-lane FINAL compose-verify + merger-instruction),
`f35111e43` (the research/143 §1 patch-equivalence reconciliation), and this
doc's commit.

Lane tips re-read 2026-11-21 from `origin` at retire time:
d120 `2408233da` (RETIRE, research/143), c25 `e5daa96f7`,
d84 `ba7e8f94a`, d85 `aeadb43a6`, d86 `6b1186c1b`, c24 `ff99f1486`,
g56 `3a07ce683`.

## 1. The five retire items — what landed

### Item 1 — the d120 INTERACTION_LINES fix (research/143 §1)

The brief's 2-hunk patch had NOT arrived when the retire phase opened
(d120's tip was `c5e68a803`, the ANALYSIS compose note only), so the fix was
ported directly from the census (`1c63c2166`): `interaction_lines(count) =
count.div_ceil(4)` (`NPCChatPanel.cs:174`), `panel_rect`'s live
`interaction_count` parameter feeding `chat_box_height` (`:68`'s
`30·lines + max(0, 2·(lines−1))` growth), and `interaction_rects`'
`num % 4 == 0` reset stepping y by `num/4·22` (`:106`). The patch then
arrived in d120's RETIRE (`2408233da`, research/143 §1) and was verified
EQUIVALENT hunk-by-hunk (`f35111e43`): the same `div_ceil(4)` fn (the
patch's `pub(crate)` visibility adopted), the same `panel_rect` parameter,
the same y-step (the patch's `out_y` == the port's `y`, both step only on
the reset); the call-site counts match (`interaction_count(sign_open)` ==
`interactions(...).len()`). Count ≤ 4 keeps the landed pixels exactly; the
new witness `more_than_four_buttons_wrap_to_a_second_row` pins count 5 →
h 122 + row-2 y +22 + the x reset, and the ceil table 0/4/5/8/9. The merge
lane (g58/f38/d120's shop/happiness/housing buttons) only grows
`interactions` / `interaction_count` — the layout is already dynamic.

### Item 2 — the sign-TILE placement census (the place-arm)

**The sign item is 171, NOT 177** (`ItemID.cs:2025`; 177 is Sapphire
`:2037` — the brief's "177?" resolved). The full place-item family, every
itemdata row verified against `Item.cs`:

| Item | Name | createTile | placeStyle | C# |
| --- | --- | --- | --- | --- |
| 171 | Sign | 55 | 0 | `Item.cs:3477-3487` |
| 321 | Tombstone | 85 | 0 | `:5186-5195` |
| 1173-1177 | Grave Marker/Cross/Headstone/Gravestone/Obelisk | 85 | 1-5 | `:14405-14459` |
| 3229-3233 | Rich Gravestone1-5 (the GOLD variants) | 85 | 6-10 (`6 + type − 3229`) | `:29791-29805` |
| 3617 | Announcement Box (`mech: true`) | 425 | 0 | `:32574-32585` |
| 4319/4320 | Arrow Sign / Painted Arrow Sign | 510/511 | 0 | `:36997-37020` |
| 4710 | Tattered Wood Sign | 573 | 0 | `:38711-38721` |

There is NO wood/stone/metal sign-item family and NO "gold sign" item —
the one Sign item places the one tile 55; the "variants" are the tombstone
styles and the mounting alternates. **The place-route is COMPLETE**: all
six kinds (55/85/425/573/510/511) carry `UsesCustomCanPlace` defs
(`TileObjectData.cs:3822-3947`) and ride the port's `place_item_tile`
registry arm (`TileObject.CanPlace`/`Place`, `Player.cs:39771-39773`). The
legacy `WorldGen.PlaceSign` (`WorldGen.cs:43781-43861`) is reached ONLY
from the `PlaceTile` dispatcher (`:60166-60171`) — the item-use path never
enters it, so no `PlaceSign` port is needed. The generated
`D_55/D_85/D_425/D_510/D_511/D_573` defs match the C# field-for-field
(2x2 `Style2x2` base, origin (0,1), AnchorBottom ×2; the four mounting
alternates for 55/425/510/511/573; 85 floor-only with `LavaDeath false` +
`DrawYOffset 2`). Witnesses: `the_sign_family_places_through_the_registry_route`,
`the_tombstone_style_shifts_the_frame_column`,
`the_sign_mounts_ride_the_frame_x_alternates` (tr-core/edit/place.rs),
`the_sign_item_rows_place_the_sign_tiles` (signs.rs).

### Item 3 — the announcement-box/gold-sign styles: ONE text-store (VERIFIED)

The frame census: `TileObject.Place` writes `frameX = 36 ·
CalculatePlacementStyle + 18·col`, `frameY = 18·row` (`TileObject.cs:60-109`;
`CalculatePlacementStyle = style·StyleMultiplier + Style`,
`TileObjectData.cs:1990-1998` — the alternates carry Style 1-4, the
tombstone styles 0-10 ride the base Style 0). **The mount AND the style
ride frameX; frameY is always 0/18** — so `Sign.ReadSign`'s origin decode
(`frameX/18 % 2`, `frameY/18`, `Sign.cs:26-29`) resolves every mount and
every style to the same 2x2 top-left. All sign styles read/write the ONE
`Main.sign[0..31999]` store keyed by that origin — `ReadSign`/`TextSign`/
`KillSign` never read the style or the mount; the tombstone epitaph write
(`Projectile.cs:25358-25363`, the d2 sweep), the editor, the hover tooltip
and the wiring announcement all ride it. No per-style store exists.

The announcement box's wiring arm (`Wiring.cs:1177-1233`) reads the SAME
store via `Sign.ReadSign(num7, num8, CreateIfMissing: false)`. The port's
425 arm carried a 2x2-BOX record scan — **FIXED** to ReadSign's EXACT-origin
match + the `!tileSign` origin kill arm (`Sign.cs:30-34`), with the
off-origin-record and stale-origin witnesses
(`the_announcement_box_reads_only_the_exact_origin_record`). The two
`is_sign_tile` doc copies' wrong tile names corrected (85 Tombstones /
425 Announcement Box / 573 Tattered Wood Sign); the sets {55, 85, 425, 573}
were already right (`Main.cs:10081-10084`). The arrow signs 510/511 place
through the same route but are NOT `tileSign` kinds — no record, no editor.

Recorded follow-up (the break lane's, NOT ported here): the port's
`check_object` has no `CheckSign` cascade arm (`WorldGen.cs:43548-43779`,
dispatch `:85761-85768`), so a broken sign drops NO item today (the
per-cell drops are suppressed by `KILL_TILE_NO_CELL_DROP`, all six kinds
true). The C# drop arms (`:43690-43764`): 85 → 321 / 1173+style−1 /
3229+style−6 by the frameX style + the ghost roll (`genRand.Next(8/4/2)`,
NPC 316, `:43708-43742`), 425 → 3617, 573 → 4710, 511 → 4320, 510 → 4319,
default → 171. The lazy `KillSign` sweep IS landed (`kill_broken_sign_record`).

### Item 4 — the c25-keyboard co-lane FINAL (keep-both, compose-verified)

A read-only `git merge-tree --write-tree` of the f22 tip against
`origin/task/W4-c25-keyboard-text-field` (`e5daa96f7`) auto-merges
`crates/tr-app/src/ui/keyboard.rs` with ZERO conflicts: c25's +516-line
bindings insert after `write_str` (before the label tables) and its 9
tests inside `mod tests`; f22's `layout`/`click_at`/`draw` signature
changes + `Context::Sign`/`bottom_row` touch disjoint regions. The
composed file was read back: both halves present, no duplicate top-level
symbols, one `mod tests`, c25's bindings self-contained (no call into
f22's changed signatures; all `#[allow(dead_code)]` until the
search/rename consumers attach).

**Merger instruction (FINAL):** keep BOTH `ui/keyboard.rs` halves verbatim
— the auto-merge is correct. The only real conflicts are the `anchors.txt`
+ `docs/PORT_MAP.md` append races: keep BOTH sides' rows (f22's
sign/rename rows AND c25's `TextField`/`get_input_text` rows; the PORT_MAP
`UIVirtualKeyboard.cs` row takes f22's superset description, c25's separate
`UITextBox.cs`/`GetInputText` row stays). `docs/UI.md` and
`docs/ANALYSIS.md` auto-merge. The c25 `char`-index caret vs the C# UTF-16
code-unit index is c25's documented mapping; f22's sign editor keeps the
`KeyboardState` text path — the two layers do not interact until a
consumer wires `get_input_text` into a front screen.

### Item 5 — this declaration

## 2. The UI-screens family — COMPLETE

The family = the 28 `UIState` descendants (the 18d census) + the card
renames + the sign editor + the d85 six-screen wave + the g56 void vault
+ the c24 Journey menu + the c25 text-entry bindings. Every member is
landed on a pushed branch or carries a recorded standing exclusion:

| Member | Lane | Tip | State |
| --- | --- | --- | --- |
| The 10 PORTED screens re-verified (char/world select+creation, keyboard, world load, achievements, emotes, bestiary, crafting) | base + f22 verify | `a725acb04` base | PORTED (18d evidence) |
| The 28-screen census + the claim-conflict rulings | **f22** | this branch | research/18d (verify-only, zero unclaimed) |
| The card renames (`.plr` name splice + `.bak`, `.trwld` header splice, the keyboard tails) | **f22** | `0da19e92a`/`6916265d5` | LANDED (research/19 M1, 11 witnesses) |
| The sign editor end-to-end (ctx-1 keyboard, the dialog sign half, submit/cancel/restore, the closes) | **f22** | `8cb88f4f5`..`e7756fa70` | LANDED (the 1200 cap pin, 14+9 witnesses) |
| The sign hover/break surfaces (signHover tooltip, SignTileBubble, the latch, the lazy kill-on-break, the epitaph display) | **f22** | `19916bbe2`..`be73b87b5` | LANDED (7 witnesses) |
| The retire round (dynamic lines, placement census, text-store verify, c25 compose) | **f22** | `1c63c2166`..this | LANDED (8 witnesses) |
| Workshop hub/publish×2/import + ResourcePack selection/info (rows 1-6) + the data layer + the shared kit | d84 | `ba7e8f94a` | LANDED (9 commits) |
| Debug commands/loadouts lists + row elements, reports page, Advanced world creation + secret seeds (rows 7-12) + the doc-sweep | d85 | `aeadb43a6` | LANDED (14 commits) |
| UIManageControls + the AWorkshopPublishInfoState pair + the UIWorldLoad PORTED re-verification | d86 | `6b1186c1b` | LANDED (6 commits) |
| UICreativePowersMenu (the Journey strips/categories/sliders/search/duplication) | c24 | `ff99f1486` | LANDED |
| The void vault (context-32 grid, Inventory_Back2/19 frames, the favorite semantics, the Void Lens) | g56 | `3a07ce683` | LANDED (5 commits) |
| The text-entry field bindings (TextField/GetInputText/repeat/clipboard) | c25 | `e5daa96f7` | LANDED (keep-both compose verified) |
| The town-chatter panel's interaction layout (the >4-button dynamic lines) | d120→f22 | `2408233da`→`f35111e43` | LANDED (the patch equivalence) |

**Standing exclusions (recorded, not gaps):** `UIWorldGenDebug` (1702 ln,
dev-only — d86 scope-out, the `UIWorldLoad.cs:50-52` F5 arm) and
`UITextWrappingTest` (336 ln, test-only harness — d86 scope-out,
research/38's UNDOCUMENTED-ABSENCE now recorded). The gamepad snap points
and the `UIGamepadHelper` struct are the Stage-1 gamepad exclusion
(research/18 deviation 10). `UISortableElement` is an element, not a
screen (the d81-d83/f23 elements lane).

**The 18d claim conflicts — resolved by the landed code:** (1) Advanced
seeds → d85 landed them (`e0775f00c`); (2) UIManageControls → d86 landed
it (`21fab7274`); (3) reports + debug lists → d85 landed them
(`555eabfa5`/`3d4b894b4`/`9daf3133b`), d86 re-scoped away (`4a1d7a037`) —
no keep-one contention; (4) ResourcePack screens → d84 landed the data
layer + both screens (`2d178bcb4`/`856a1c74d`/`ff68eb9d9`), so d86's
dependency fallback is moot; (5) the UIWorldLoad F5 scope-out record →
d86 landed it in code (`19641e25f`). The AWorkshopPublishInfoState pair
owner decision → d86 ported the data/state half (`661761026`).

## 3. Declaration

**The UI-screens family is COMPLETE.** All 28 `UIState` descendants are
accounted for: 26 ported (10 on base re-verified + 16 across the
d84/d85/d86/c24 waves) and 2 standing scope-outs (UIWorldGenDebug,
UITextWrappingTest — dev/test-only, recorded). The f22 lane's own ports
(the card renames, the sign editor + hover/break surfaces, the retire
round) are landed with their witnesses written not run (code-only mode);
the cross-lane hand-offs (the d120 dynamic-lines patch, the c25 keep-both
compose, the d2 epitaph write, the break-lane CheckSign cascade) are
recorded with their owners. No UNCLAIMED screen remains; no f22 follow-up
is open. The merger inherits: the keyboard.rs keep-both instruction
(§1 item 4), the anchors/PORT_MAP append-race keep-both rows, and the
recorded break-lane sign-drop follow-up (§1 item 3).

Witness tally (this retire phase, all WRITTEN NOT RUN): 6 new — 1
dynamic-lines (`more_than_four_buttons_wrap_to_a_second_row`, npc_chat.rs)
+ 3 placement (`the_sign_family_places_through_the_registry_route`,
`the_tombstone_style_shifts_the_frame_column`,
`the_sign_mounts_ride_the_frame_x_alternates`, edit/place.rs) + 1 itemdata
(`the_sign_item_rows_place_the_sign_tiles`, signs.rs) + 1 announcement
exact-origin (`the_announcement_box_reads_only_the_exact_origin_record`,
wiring.rs); the c25 compose is a read-only merge-tree verify (no witness).
Plus 2 existing witnesses updated (the `panel_rect` call in
`the_close_row_rides_the_panel_bottom`, the full-box fixture in
`the_announcement_box_prints_the_sign_text`). On top of the lane's 40+
earlier witnesses. No cargo/tests/rustfmt/graphify invoked; verification
is the C# re-read + the read-only merge-tree compose.