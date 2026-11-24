# 237 — The bestiary UI FINAL close (W4-e23 final phase)

STATUS: FINAL (2026-11-21, code-only mode — witnesses WRITTEN NOT RUN).
Branch: `task/W4-e23-bestiary-ui-grid`, base a4b972bd (the five remainders
landed). This report closes the last deferred items of the bestiary screen
and records the three co-lane compose verdicts (c8 / f28 / d42).

Number: 237 — the first free number after the research/150 collision
registry's 151–236 block (38bfd1d22).

## 1. The deferred overlays + biome-flavor fallback (LANDED, final 1 — 1750cc158)

The ANALYSIS deviation 7's two deferred pieces close, plus the Misc info
rows they share data with:

- **The decorations pass** (`BestiaryDatabaseNPCsPopulator.cs:2028-2071`,
  `AddNPCBiomeRelationships_AddDecorations_Automated`) replays in
  `tools/gen_bestiary_tags.py` BETWEEN the relationship passes and
  `ModifyEntriesThatNeedIt` — the `Populate:544-555` order. Per sampled net
  id (-65..696, the `ContentSamples.NpcsByNetId` iteration): the Rain arm
  picks `Visuals.Blizzard` when the entry carries Snow else `Visuals.Rain`
  and CONTINUES (`:2043`); Eclipse/NightTime/DayTime/PumpkinMoon/FrostMoon
  are independent ifs; the Meteor biome adds Moon AND Meteor (`:2067-2068`).
  The membership tests are reference checks — the preference wraps land
  later, so the plain-row set is the post-relationship Info.
- **The overlay catalog**: the eight `Visuals.*` rows
  (`SpawnConditionDecorativeOverlayInfoElement`, `:291-322`) join `TAGS`
  (ids 60-67, key ""/frame 0 — the decorative ctor carries neither,
  `SpawnConditionDecorativeOverlayInfoElement.cs:16`), plus the three
  `SpawnConditionBestiaryOverlayInfoElement` events (Rain 40 / Blizzard 42
  / Sandstorm 43 — only Sandstorm carries an overlay image,
  `MapBGOverlay1`+White, `:112`). `TagDef` gains `overlay`/`overlay_color`/
  `text_priority` (`DisplayTextPriority`)/`hide_in_portrait_info`
  (`HideInPortraitInfo`, Blizzard only `:109`)/`display_priority` (the
  overlay classes' own `DisplayPriority`, all Visuals 1)/`order_priority`
  (`OrderPriority`: biomes 0, Times -1, Events -2).
- **The portrait overlay draw** (`NPCPortraitInfoElement.cs:77-83` →
  `UIBestiaryNPCEntryPortrait.cs:42-55`): `ui/bestiary.rs::portrait_overlays`
  collects the overlay providers ONLY when no preference-matched provider
  won the background image (`!flag2`, `:77` — the same pass-1 pick
  `portrait_backdrop` runs, the world-evil rows included), then the stable
  `OrderBy DisplayPriority` (`:82-83`, `GetSortingValueForElement:97-104`).
  The draw pushes each image `ScaleToFit` over the portrait inner box
  BETWEEN the backdrop and the icon, tinted by its own color or
  `Color.Lerp(White, portraitColor, 0.5)` (`:53` —
  `npc_portrait_overlay_color`, its `allow(dead_code)` lifted).
- **The biome-flavor fallback** (`TryGivingEntryFlavorTextIfItIsMissing`,
  `:773-800`, run at `:637` after the preference wraps): the generated
  `FLAVOR_FALLBACK` table (118 rows) records, per entry whose factory found
  no `Bestiary_FlavorText.npc_<Const>` row (the en-US JSON's
  `Bestiary_FlavorText` section is the Exists oracle), the first
  preference-wrapped spawn-condition tag's `Bestiary_BiomeText.biome_<Tail>`
  key (`:783-787` — the world-evil wrap is the OTHER class, so 533/529 fall
  through to the scan), else the plain `SpawnConditionBestiaryInfoElement`
  with the highest `DisplayTextPriority` (`>=` — ties to the LATER row,
  `:788-792`; the overlay class never feeds the scan). `flavor_text` falls
  back to it from tier 2; the element is added WITHOUT an Exists check
  (`:798`) — `lang::text` prints the key itself when the row is missing,
  the same `Language.GetText` contract.
- **The Misc spawn-condition rows** (`FilterProviderInfoElement.ProvideUIElement`,
  `:55-95`): the info page's sixth group — the plain tag rows from tier 1
  as the (-14,1)x34 (43,56,101) panels, the framed `Icon_Tags_Shadow` icon
  at Left 5 VAlign 0.5, the 0.8 display name at PaddingLeft 38 with the
  `DynamicallyScaleDownToWidth` shrink, the `ShowButtonName` hover
  (`:105-112`); `HideInPortraitInfo` suppressed, the group ordered by
  `OrderPriority` descending (stable — `UIBestiaryEntryInfoPage.cs:142`).
  The `bestiary_info_category` Misc arm now has live rows.
- **Atlas**: +9 `MapBGOverlay1..9` sheets; measured 7753→7759 px skyline
  (`tools/atlas_sim.js`, waste tie-break, deterministic double-run
  verified), under the 8192 bound. The harness's deleted-L-8b-ref read now
  tolerates the pruned branch (the mount scenario collapses to the current
  STATIC list).
- **Invariant checks**: `gen_bestiary_tags.py --check` and
  `gen_bestiary_sorting_ids.py --check` both green; the plain-tag
  membership diff vs the pre-final table is ZERO (the decorations only
  APPEND ids ≥ 60; the filter list / `lowest_biome_group_index` /
  `has_tag` read ids < 57 only).

## 2. The bestiary NOTES writer (LANDED, final 2 — 715a69844)

The e16 closeout (1dc8520f3, research/128 sec.7) found the e23
`bestiary_notes` writer EXISTS (`ui/bestiary.rs:534` pre-final) but its
hover gate was dead code; the f28 tooltip salvage (85165b8be) carries the
consumer (`tooltip_lines2.rs:91` `HoverFlags.bestiary_notes: Option<&'static str>`
+ `:543` the `Main.cs:21334-21342` print block). The e23 side closes:

- `info_item_shows_tooltip`'s `allow(dead_code)` LIFTS — the drop-row draw
  feeds it (`UIBestiaryInfoItemLine.DrawSelf:112-119`: hovered AND
  `!_hideMouseOver`).
- The hovered non-locked line builds its display item (a fresh
  `SetDefaults(itemId)` count 1, `:23-24`) and calls `draw_item_tooltip`
  (`DrawMouseOver:121-126`'s `Main.HoverItem` + `MouseText("", 0, 0)`).
- `TooltipContext` gains `bestiary_notes: Option<&str>` — the f28
  `HoverFlags.bestiary_notes` twin. `item_tooltip_lines`' tail prints the
  rows via `tooltip::bestiary_note_rows` (the `IsNullOrWhiteSpace` gate +
  the `'\n'` split, white — `Main.cs:21334-21342` verbatim).
- The `Drop` InfoRow carries `item_id` + `notes`
  (`SetBestiaryNotesOnItemCache:137-156` joins the non-whitespace condition
  descriptions). The port's `DropRateInfo` carries no condition objects
  (the documented `loot.rs:292-294` gap), so the cache joins the empty
  list today — the writer seam stands live for the conditions port.

**Compose contract (the f28c merger's receipt):** the f28
`tooltip_lines2` tail block and the e23 `bestiary_note_rows` are the SAME
`Main.cs:21334-21342` port — keep ONE at the union (the f28 module is
canonical per research/128; the e23 `bestiary_note_rows` retires onto it,
the `TooltipContext.bestiary_notes` field folds onto
`HoverFlags.bestiary_notes`). The e23 WRITER (the drop-line hover +
the notes cache) is unique — no f28 twin. The compose witness
(`the_bestiary_notes_compose_contract`) pins the writer join → reader
split → gate → full `item_tooltip_lines` tail pipeline.

## 3. The c8 SIGHT-unlock compose (LANDED, final 3 — 57cdff0c6)

The c8 sweep 5 (a17cd75b8, tip 3705670d3) landed the sight census on ITS
branch against the OLD kill-only shape (`unlock_state_world` /
`completion_percent_world` + a kind-based `is_critter_entry`). The e23
base already carries the c37 gate-table dispatch (3e138525, an ancestor —
verified `merge-base --is-ancestor` exit 0): `unlock_state(entry,
&WorldBestiary)` evaluates `bestiary_data::gate(net_id)` — the C#'s
provider polymorphism as DATA. The compose verdict:

- **No parallel dispatch needed.** The c8 `unlock_state_world` semantics
  ARE the port's `unlock_state` shape; the UI draw chain, the completion
  bar, the Zoologist shop gate and the BOOK_WORM hook
  (`achievements.rs:417` / `netglue.rs:2986` / the kill path
  `world_sim/npcs.rs:5241`) already ride the WorldBestiary. The c36c
  re-probe's residual "c37 provider fold" closes here.
- **The gold provider folds onto the gate table**: `Gate::GoldCritter
  { credit_net, peers }` + the populator's 13-row peer dictionary verbatim
  (`:703-720`, the 539 row's TWO peers {299, 538}) + the 14-id
  `GoldCrittersCollection` (`NPCID.cs:10121-10125`) + the four critter-twin
  `Highest` folds (`:721-730`). `eval_gate`'s gold arm ports
  `GoldCritterUICollectionInfoProvider.cs:21-75`: max(own, peers) sights
  (`:23-37`), the tier-0 early return (`:42-45`), the any-gold-seen gate
  (`:46-52`/`:56-66`).
- **The proximity scan lands**: `scan_world_for_critter_sights` at the
  `update_npcs` head (`Main.cs:17914` — before the UpdateNPC loop, so a
  despawning critter still earns its sight; `netMode != 1` folded true).
  The player box inflated 300×200 (`Player.cs:3313-3321`),
  `CountsAsACritter` = live `max_hp <= 5` + the kind's SetDefaults contact
  damage 0 (`NPC.cs:6887-6897`; the 594/686 exclusions are unported kinds),
  the half-open XNA `Intersects`, the idempotent BTreeSet insert + the
  `try_grant_bestiary_100_net` hook (`SetWasSeenDirectly:43-52`).
- **Roster note**: the c8 nine crawler kinds (357/374/375/448/484-486/
  606/661) have no ENTRIES rows on this base — the c8 branch's own
  `is_critter_entry` set is recorded in `CRITTER_NET_IDS`' doc for the
  union; the gold gate's peer credits resolve through `credit_id_for_net`
  once those rows land (today the 448 own-sight arm is the live path).
  The Goldfish 55 / Etherian Portal 549 critter rows already ride the
  sight provider (the c37 census).
- **The c36c BOOK_WORM ladder co-lane read**: the c36 re-probe
  (4a8d56d99/774902cfc) verified progression 29 = BOOK_WORM (no bullseye
  row), the S1-F-15 tier-4 denominator fixed (`kills_needed_for` default
  50), the residual gate = the town/critter provider fold — that fold is
  the c37 table this branch carries, now gold-complete. The
  `notify_progression(29)` witness (`achievements.rs:711`) stands.

## 4. The sorting-ids table vs the d42 roster join (VERIFIED, final 4 — 0a3450781)

The d42 wave (tip 9b22c091a, NPCID 578-630, 53 kinds incl. the twelve Hide
folds) touches `npc.rs`/`npc/data.rs`/`npc_ai.rs`/`npc_buffs.rs`/
`render/entities.rs` + 65 lines of `ui/bestiary.rs` portrait arms — it does
NOT touch `bestiary.rs` ENTRIES, the gate table, the tag table or the
sorting generator. The 546-table is generated from the DECOMPILED sources
(`NPC.cs` SetDefaults arms + `NPCID.cs` Hide rows + the populator tag
walk), never the port roster — a roster wave cannot drift it. Verified:

- Both `--check` runs green (no regen needed).
- The 578-630 band: 40 visible + 13 Hide folds
  (579/589/591/593/595-598/600/603/609/622/623), ZERO missing.
- The Hide-set invariant: the 216-row `NPCBestiaryDrawOffset.Hide`
  derivation, the band's folds read 0 (`sorting_id`'s absent-key arm), the
  numbering is the 1..=546 permutation, Blue Slime 132 / Moon Lord 546 /
  Guide 1 stand.
- The d42 portrait arms compose cleanly with the final-phase
  `portrait_data` (they append match arms; the e23 final touches
  `draw_info_row`/`build_info_rows`, not the portrait arm table).

## 5. Witness ledger (this final phase — all WRITTEN NOT RUN, code-only)

| # | Witness | File | Pins |
| --- | --- | --- | --- |
| 1 | `the_decorations_pass_appends_the_visuals_rows` | ui/bestiary.rs | the Rain/Eclipse/Night/Day/Meteor arms + the Meteor double |
| 2 | `the_portrait_overlays_ride_the_fallback_arm` | ui/bestiary.rs | the !flag2 gate, the stable priority order, the Rain tint |
| 3 | `the_overlay_color_lerps_toward_the_portrait_color` | ui/bestiary.rs | the :53 Lerp fallback + the own-color win |
| 4 | `the_biome_flavor_fallback_fills_the_missing_rows` | ui/bestiary.rs | the 25/30 fallback keys, the 22/23 nulls, the tier-2 gate, the 128 night scan |
| 5 | `the_spawn_condition_rows_order_and_gate` | ui/bestiary.rs | the Hide flag, the OrderPriority trio, the Blue Slime row order |
| 6 | `the_bestiary_notes_compose_contract` | ui/bestiary.rs | the writer join → reader split → gate → tooltip tail |
| 7 | `w4_e23_the_gold_worm_gate_gates_on_any_gold_critter_seen` | bestiary.rs | the gold gate's three arms + the collection table + the 539 peers + the twin folds |
| 8 | `w4_e23_the_completion_counts_critter_sights` | bestiary.rs | the sight moves the bar, 50 kills do not |
| 9 | `w4_e23_the_critter_sight_scan_registers_proximity_once` | world_sim/npcs.rs | near/far/zombie/dead/idempotent + the tier-4 read |
| 10 | `the_sorting_table_covers_the_d42_band_and_holds_the_hide_invariant` | bestiary_filter.rs | the 546-permutation + the band coverage + the spot rows |

## 6. Residual deferrals (unchanged, documented)

- The gamepad link points (`SetupGamepadPoints`, ids 3000+) — the whole
  gamepad-navigation layer is out of scope (ANALYSIS deviation 8).
- The `Alphabetical` culture-aware collation + the `BySearch` non-ASCII
  case-fold (deviations 2/3).
- The raw 72 px CELL lattice (deviation 6, the S1-E-26 simplification).
- The drop-condition objects (`loot.rs`'s `DropRateInfo` carries none) —
  the notes writer joins the empty list until the conditions port; the
  seam is live (final 2).
- The c8 crawler ENTRIES rows + the d42 band's ENTRIES rows — the roster
  waves own them; the gate/tag/sorting tables already carry their ids.
