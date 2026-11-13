# 20 — In-game UI (options / inventory / chest / craft)

> Wave 2 · researched 2026-11-12 · sources: `decompiled/Terraria/UI/ItemSlot.cs`
> (4 653 lines), `decompiled/Terraria/UI/CraftingUI.cs` (603),
> `decompiled/Terraria/GameContent/UI/NewCraftingUI.cs` (996),
> `decompiled/Terraria/UI/ChestUI.cs`, `decompiled/Terraria/Main.cs` +
> `crates/tr-app/src/ui/` + `crates/tr-app/src/app/clicks.rs` (3 084) + ROADMAP §23.

## Scope

This module is the *in-game* half of the UI: the HUD overlays the player drives
while a world is loaded — the inventory/hotbar grid with its item-slot protocol,
the chest and bank panels, the crafting window, the equipment/dye page, the trash
slot, the settings window and the tooltips. The framework machinery (retained
mode, `UserInterface`, layers) is module 18; the front-end screens are module 19;
the recipe *data* is module 21 and the chest *logic* module 22, so only their UI
surfaces are judged here.

The single highest-traffic surface is `ItemSlot.cs`: `LeftClick`
(`:914-1253`) is a six-arm switch on `PickItemMovementAction`
(`:1484-1634`) — action 0 (generic move), 1 (equip), 2 (dye), 3/4 (shop), 5
(Journey duplication) — with `MouseItemSwap` (`:1255-1263`) clearing favourites
unless `retainFavoriteOnMoveAt[context]`. `RightClick` (`:1636-1715`) first tries
the "alternate actions" (openable bag at `:1660-1667`, `HasItemSwap` at
`:1668-1675`, vanity swap `:1678-1684`, `SwapEquip` `:1685-1697`), then falls to
the paced one-per-transfer split `PickupItemIntoMouse` (`:1717-1772`) gated on
`Main.stackSplit <= 1` (`:1700`). Shift behaviour is not a handler but a cursor
promise: `GetAlternateClickAction` (`:629-914`) / `OverrideLeftClick`
(`:715-780`), consumed by `ItemSlot.Handle` (`:569-627`). `PickupItemIntoMouse`
moves **one** item per call; a held right button repeats because `RightClick`
tests `Main.mouseRight` (held, `:1654`), not an edge.

Rust: the whole click protocol lives in one file, `app/clicks.rs`, while the
draw/hit geometry lives in `ui/{slot,inventory,chest,crafting,ingame_options,
tooltip}.rs` and the front-end-style legacy immediate menu in `app/menu/`.
`ui/slot.rs` (771) is **draw-only** (sheet selection, stack numbers, hotbar
digits, equip glyphs) — all interaction is in `clicks.rs`.

## Key entities

- `decompiled/Terraria/UI/ItemSlot.cs::PickItemMovementAction` (`:1484-1634`) —
  the per-context gate; returns the action index (or `-1` = refused).
- `ItemSlot.cs::LeftClick` (`:914-1253`) / `MouseItemSwap` (`:1255-1263`) — the
  action dispatch and the favourite-clearing swap.
- `ItemSlot.cs::RightClick` (`:1636-1715`) / `PickupItemIntoMouse`
  (`:1717-1772`) / `RefreshStackSplitCooldown` (`:1774-1784`) — the right-click
  ladder and its 30/`stackDelay 7` pacing.
- `ItemSlot.cs::GetAlternateClickAction` (`:629-914`) +
  `OverrideLeftClick` (`:715-780`) — the Shift/Alt cursor contract (trash 6,
  favourite 3, transfer-to-chest 9, transfer-from-chest 8, unequip).
- `Main.cs::HoverOverCraftingItemButton` (`:42250-42306`) — the craft amount
  (`superFastStack + 1`, **×10 while Shift is held**, `:42271-42275`), the
  `stackSplit <= 1` gate and the focus-change branch.
- `NewCraftingUI.cs::HandleCraftSlot` (`:499-528`) — the recipe grid's
  select-vs-craft split (`_selectedRecipeIndex` vs `Main.focusRecipe`).
- `CraftingUI.cs::DrawRecipesList` (`:161-232`) / `DrawGridToggle` (`:267-334`) /
  `DrawCraftFromNearbyChestsToggle` (`:336-356`).
- `Main.cs::DrawTrashItemSlot` (`:42349-42366`) — the (448, 258) slot at scale
  0.85, or +5/+168 at 0.755 while a chest/shop is open.
- `Player.cs::DropItems` (`:54304` head) — death clears `trashItem`.
- `crates/tr-app/src/app/clicks.rs::handle_inventory_clicks` (`:557-943`) — the
  HUD click chain, in the C#'s consume-the-press order.
- `clicks.rs::left_click_movement` (`:67-89`) — the shared action-0 move
  (take / place / merge-capped / swap) used by the grid, the coin/ammo rows and
  the bank.
- `clicks.rs::context_accepts` (`:122-128`) / `coin_ammo_click` (`:207-277`) /
  `evaluate_cursor_override` (`:1063-1133`).
- `clicks.rs::handle_chest_clicks` (`:955-1060`) — ctx-3 plus the re-click and
  chest-switch arms; `chest_transfer_click` (`:1141-1201`);
  `chest_slot_click`/`chest_slot_stack_split` (`:1203-1296`).
- `clicks.rs::handle_crafting_clicks` (`:347-412`).
- `ui/inventory.rs::trash_rect`/`trash_slot_at` (`:729-753`), `coin_ammo_hit`
  (`:259`), `bank_slot_at` (`:707`); `app/mod.rs::trash_swap` (`:1192-1193`).

## Dependencies

- Calls into: `crate::registry` (max_stack, `ItemDef`), `crate::craft`
  (`available`, `try_craft`, `chests_in_range`, `scan_adjacent`), `crate::bank`
  (loot_all / deposit_all / quick_stack / restock / move_coins), `crate::chest`
  (`Source`, `try_placing_from`, `can_favorite_at`), `crate::inv` (slot
  numbering), `ui::*` geometry modules, `assets::Lookups` texture ids,
  `app/audio.rs` cues, `App::achievements` (`notify_craft`),
  `world_sim`/`player` state.
- Called by: `app/frame.rs` (the per-frame HUD click pass), `app/loop.rs`
  (`ui_left_press`/`ui_right_press` are set on press edges, `loop.rs:373/428/438`).
- Shared data structures: `item::ItemSlot { kind, name, count, prefix }`,
  `item::EquipSlots { armor, vanity, dye, coins, ammo, misc, trash, hook,
  mount, pet }`, `App::mouse_item`, `App::favorite_slots: Vec<usize>`,
  `App::stack_split`/`stack_delay`, `App::cursor_override`,
  `settings::{quick_trash, disable_left_shift_trash, craft_from_nearby_chests}`.

## Port status

| C# subsystem | Status | Rust location | Evidence / notes |
| --- | --- | --- | --- |
| `LeftClick` action 0 (generic move, merge-capped) | PORTED | `clicks.rs::left_click_movement:67-89` | merge gate is same kind **and** same prefix (`ItemSlot.cs:980-998`); the cap is `registry::max_stack` |
| `PickItemMovementAction` gates | PORTED | `clicks.rs::pick_item_movement_action`, `context_accepts` | the whole `:1484-1634` switch: ctx 0/3/6/30 accept (0), coin/ammo rows, the material guide gate (7), the armour/vanity/accessory rows by `slot % 10` (8-11), dye (12/25/27/33 → 2), display-doll armour/accessory (23/24), hat rack (26), creative (29 → 5) and shop (15 → 3/4). The arms reading unported data (ctx 5 `CanHavePrefixes`, 16 `projHook`, 17/18/39 `mountType`, 19/20 pet tables, 38 `TEDisplayDoll`) keep the C# `type == 0` shortcut (accept non-air) — docs/ANALYSIS.md; tests `pick_item_movement_action_ports_the_context_table` |
| `LeftClick` actions 1/2/3/4/5 | MISSING | — | equip (`:1019-1120`), dye (`:1121-1218`), shop (`:1219-1241`), Journey duplication (`:1242-1251`) have no port arm |
| `RightClick` alternate actions | MISSING | — | openable bag (`:1660-1667`), `HasItemSwap` (`:1668-1675`), `SwapVanityEquip` (`:1678-1684`), `SwapEquip` (`:1685-1697`) — the port's right click never equips |
| `RightClick` one-per-transfer split | PORTED | `clicks.rs::right_click_split`, `pickup_one_into_mouse` | one item per transfer (`PickupItemIntoMouse:1717-1772`), `superFastStack+1` per gated call; shared by the inventory grid, coin/ammo rows, bank and chest (S1-C-10) |
| `stackSplit` pacing on the grid | PORTED | `clicks.rs::right_click_split` | all four surfaces ride the `stackSplit > 1` gate + `RefreshStackSplitCooldown` (`:1700`, `:1774-1784`); chest tests `dw_4_3_*` plus the S1-C-10 grid/coin/bank tests |
| `GetAlternateClickAction` Shift arms | PORTED | `clicks.rs::evaluate_cursor_override:1088-1132` | ctx-3 from-chest cursor 8, to-chest cursor 9 (incl. the coin/ammo arms `:657-665`), quick-trash tail `:679-682`; the head guard (air/favorited) is in `chest_transfer_source:179-201` |
| `OverrideLeftClick` Alt/favourite + quick-trash | PORTED | `clicks.rs:882-898`, `coin_ammo_click:213-219` | favorite cursor 3 and trash cursor 6 consume the press; the Journey-research exception inside cursor 6 (`ItemSlot.cs:762`) is out |
| `MouseItemSwap` favourite clearing | PARTIAL | `clicks.rs::left_click_movement:85` | C# clears both sides unless `retainFavoriteOnMoveAt[context]` (`:1255-1263`); the port tracks favourites by **slot index** (`App::favorite_slots`), so a favourite follows the slot, not the item |
| Hotbar (closed) + stack icons | PORTED | `ui/slot.rs::closed_hotbar_metrics`, `hotbar_digit:165-188`, `stack_number_pos:229` | `GUIHotbarDrawInner` (`Main.cs:43614-43639`), 17 tests incl. `closed_hotbar_metrics_match_guihotbardrawinner` |
| Slot background / context textures | PORTED | `ui/slot.rs::context_texture_index:23-76` | the per-context `Inventory_Back*` table (incl. trash 6 → Back7, `:34`) and the equip glyph table (`:105-146`) |
| Chest panel (slots, scroll, buttons, rename) | PORTED | `ui/chest.rs:131-502`, `clicks.rs:1141-1296`, `item_sorting.rs`, `sortdata.rs` | LootAll/DepositAll/QuickStack/Restock dispatch + the keyboard rename + the **Sort** button (id 4 → `ItemSorting.SortChest`, the full 52-layer engine, S1-H-8) landed; the name field blinker/hover ramp are draw-only, and the `withFeedback` `ItemSlot.SetGlow` half is a no-glow-system no-op |
| Bank panel (ctx 4/32) | PARTIAL | `ui/inventory.rs::bank_slot_at`, `clicks.rs` bank block | uses the action-0 move + the paced one-item right-click split (S1-C-10); the ctx-4/32 shift route has no cursor-8/9 (`evaluate_cursor_override` admits it) |
| Crafting UI | PARTIAL | `ui/crafting.rs` (draw + `hit_test:328`), `clicks.rs:347-412` | left-click craft-one works; the recipe filter strip and search bar are **draw-only** (`draw_filter_strip:690`, `draw_search_bar:737` — no hit test); no `focusRecipe`/`_selectedRecipeIndex`, no guide item, no Shift ×10 craft-all, no held-repeat pacing |
| Equipment / dye page | PARTIAL | `ui/inventory.rs:282-676`, `ui/chrome.rs::draw_page_icons` | page-tab clicks landed (M7-UI-3a); the dye drag needs `Item.dye` (ROADMAP M7-UI-3 open) |
| Trash slot (BUG-20) | PORTED | `ui/inventory.rs::trash_rect:729-736`, `app/mod.rs::trash_swap:1192` | `(448,258)` @0.85 / `+5,+168` @0.755 (`Main.cs:42349-42366`), quick-trash cursor 6, death clears (`Player.cs:54304`) — 3 `bug20_*` tests |
| Settings window | PORTED | `ui/ingame_options.rs` (2 101 lines) | M7-UI-4/4b: two-column `IngameOptions.Draw` (`IngameOptions.cs:103-1441`), 61 rows, `TR_OPTIONS=1` |
| Tooltips | PARTIAL | `ui/tooltip.rs` (646) | equipable/range/bait rows landed (M7-UI-6); the `tip[60]` shop-only pair and the full per-item `Item.ToolTip` table are open |
| Legacy immediate menu (front-end) | PORTED | `app/menu/{mod,pages,actions}.rs` | out of this module's scope but shares the macro-style rows |

## Tests

Static counts in the module's files: `app/clicks.rs` **59** `#[test]`,
`ui/ingame_options.rs` **27**, `ui/slot.rs` **17**, `ui/chest.rs` **13**,
`ui/inventory.rs` **10**, `ui/crafting.rs` **10**, `ui/tooltip.rs` **9**,
`app/menu/mod.rs` **69** (front-end).

Representative names (`clicks.rs`): `dw_4_1_take_place_swap_stack_on_chest_slots`,
`dw_4_1_stack_merge_respects_max_stack_boundary`, `dw_4_2_shift_transfer_full_chest_is_noop`,
`dw_4_3_stack_split_timing_constants`, `dw_4_3_right_click_stack_one_takes_one`,
`dw_4_3_right_click_repeat_paced_by_stack_split`,
`right_click_split_respects_the_cursor_room_and_kind`,
`test_DW_2_1_stack_split_carries_the_prefix_and_refuses_cross_prefix`,
`shift_click_transfers_inventory_into_the_chest`,
`shift_click_from_chest_slot_transfers_to_the_inventory`,
`re_click_on_the_open_chest_tile_closes_it`,
`right_click_on_another_chest_switches_while_open`,
`test_DW_3_1_loot_all_drains_a_world_chest_coins_first_and_keeps_overflow`,
`test_DW_3_3_quick_stack_world_merges_only_held_types_without_move_coins`,
`test_DW_4_1_alt_left_click_toggles_the_favorite`,
`test_DW_4_2_can_favorite_gate_holds_and_toggle_plays_sound_12`,
`test_DW_4_3_shift_click_on_favorited_slot_is_noop`,
`test_DW_4_4_left_click_trash_swaps_held_and_trash`,
`bug20_shift_click_moves_the_stack_to_the_trash`, `bug20_death_clears_the_trash`,
`bug20_the_trash_shifts_with_the_chest_open`,
`ui_2b_the_rows_are_dead_while_the_inventory_is_closed`,
`test_DW_3_5_bank_values_dispatch_no_chest_panel_row`.
`ui/slot.rs`: `closed_hotbar_metrics_match_guihotbardrawinner`,
`stack_number_offset_uses_the_source_formula`, `hotbar_digit_renders_ten_as_zero`,
`equip_glyph_index_map`, `crafting_contexts_use_inventory_back18`,
`test_DW_4_5_favorited_background_gate_matches_the_csharp_arm`.
`ui/crafting.rs`: `layout_matches_the_source_table_at_1600x900`,
`difficulty_3_shifts_both_toggles_right_by_40`, `filter_buttons_and_icons_match_the_source_table`,
`search_bar_geometry_and_text_pos_match_the_source` — geometry only, no click tests.
`ui/ingame_options.rs`: 27 tests incl. the quick-trash three-state cycle.

## Deviations & risks

1. ~~**Right-click takes half the stack, not one item**~~ — **RESOLVED
   2026-11-12 (S1-C-10)**: `clicks.rs::right_click_split` /
   `pickup_one_into_mouse` move exactly one item per transfer on the
   inventory grid, coin/ammo rows, bank and chest (`ItemSlot.cs:1717-1784`).
2. ~~**The inventory right-click is not paced and does not repeat**~~ —
   **RESOLVED 2026-11-12 (S1-C-10)**: `handle_inventory_clicks` dispatches on
   `Main.mouseRight` (`place_down`), and the shared split reads the
   `stackSplit > 1` cooldown, so a held right-click repeats one item at the
   vanilla cadence.
3. **No right-click equip/dye/swap actions** — `SwapEquip` (`:1685-1697`),
   `SwapVanityEquip`, `HasItemSwap`, openable bags. Equipping an accessory or
   armour by right-click is impossible. The *gate* is now faithful (S1-H-6:
   `clicks.rs::pick_item_movement_action` ports `ItemSlot.cs:1484-1634`), so
   the equip/dye/mount/pet contexts no longer accept any held item — but the
   equip pages still draw without a left-click handler (`LeftClick` actions
   1/2 are the separate S1-H-5), so the gate is inert in-game until then. The
   arms whose data the port lacks (ctx 5/16/17/18/19/20/38/39) keep the C#
   `type == 0` shortcut and accept non-air (docs/ANALYSIS.md). **Behavioural**,
   not admitted (M7-UI-3 admits only the *dye drag*).
4. **Favourites follow the slot, not the item** — `MouseItemSwap` clears both
   sides (`:1255-1263`); the port's slot-index favourites survive a move and
   block the shift-transfer of the receiving slot. **Behavioural**, admitted in
   M3-STORE-3's wording but not measured against the C#.
5. **`MAX_STACK` (flat 999) in the right-click merge arm** — `clicks.rs:984` uses
   `item::MAX_STACK` while every sibling uses `registry::max_stack(kind)`; the C#
   gate is `Main.mouseItem.maxStack` (`ItemSlot.cs:683`, `:1707`). Diverges for
   any kind whose `maxStack` is not 999. The arm is now the port's **press-edge
   carry** only (the C# right click never deposits); the same-kind pickup wins
   when the cursor has room (S1-C-10). **Behavioural**, cosmetic-sized today.
6. **Crafting UI is one click deep** — no Shift ×10 craft-all, no `superFastStack`,
   no `stackSplit` pacing, no `focusRecipe`/`_selectedRecipeIndex` select step
   (`NewCraftingUI.cs:499-528`), no filter/search hit tests, no guide item, no
   `HoverOverCraftingItemButton` tooltip chain (`Main.cs:42250-42306`). Filters
   are drawn (`ui/crafting.rs:690`) but inert. **Behavioural**; the recipe *logic*
   itself (module 21) is complete.
7. **No `CoinSlot.ForceSlotState`** — the C# re-derives a merged coin slot's rank
   (`ItemSlot.cs:1016`, `:1119`); the port merges only identical kinds, so a gold
   into a copper slot stays a mixed/broken state. **Behavioural** (coins only).
8. **Achievement hooks on slot moves absent** — `NotifyItemPickup`
   (`ItemSlot.cs:960`) and `HandleOnEquip` (`:972`) never fire from the click
   path; only `notify_craft` does (`clicks.rs:409`). **Behavioural/cosmetic**.
9. **Quick-trash cue `TrashItem` unportable** — `clicks.rs:896` plays a cue that
   is not loaded; admitted. **Cosmetic**.
10. **Trash right-click (Journey sacrifice) unported** — `clicks.rs:850`;
    the cursor-6 arm skips `TryResearchingItem` (`ItemSlot.cs:940-943`).
    **Behavioural**, Journey-mode only, admitted.
11. **Shift-sell cursor stands down** — `GetSellOrTrash` with a shop open
    (`:198-209`) is not modelled (`clicks.rs:1120-1122`). **Behavioural**, shop
    (module 22) adjacency.
12. **UI scale pinned to 1.0** (module 18 deviation 6) — `UserInterface`'s scale
    spaces cannot be exercised; the port's grids hard-code 0.85 / 0.755.
    **Behavioural/pixel**, admitted.
13. **Chest Sort is ported; name-field editing is partial** — the rename keyboard
    path landed, and the Sort button (id 4) now dispatches `ItemSorting.SortChest`
    (S1-H-8, 2026-11-13); the name field blinker/hover ramp stays draw-only.
    **Behavioural**.
14. **BUG-20 is implemented but its catalog row is not closed** — the code and
    three `bug20_*` tests are on main, yet `ROADMAP.md:1696` still reads
    "trash slot: quick-trash … clear on death … the +5/+168 chest-shop offset"
    with no done text (BUG-21 sits on the next line *with* one). **Docs drift**.

## Task seeds

- **S — Pin the right-click amount.** Replace `(count + 1) / 2` in
  `clicks.rs:911`, `:238` and `:813` with a one-item `PickupItemIntoMouse`
  equivalent (`ItemSlot.cs:1717-1772`), add the `stack_split > 1` gate
  (`:1700`) and `RefreshStackSplitCooldown` (`:1774-1784`) on all three paths,
  then flip the existing `dw_4_3_*` tests onto the grid. Depends on nothing;
  unblocks task 2.
- **S — BUG-20 row closeout.** Move the ROADMAP line 1696 into the DONE shape
  quoting the three bug20 tests and `ui/inventory.rs::trash_rect` (docs only).
- **M — `PickItemMovementAction` as a real table.** Port `:1480-1634` as
  `clicks::movement_action(context, slot, kind, item_flags) -> Option<u8>` with
  the C#'s arm order (incl. the context-30 override at `:1629-1632`), then route
  the equip/dye/shop/Journey arms of `LeftClick` (`:1019-1251`) to it. Depends
  on `ItemDef` fields (`head_slot`, `dye`, `mount_type`, `buff_type`, `material`,
  `shoot`) — a `GenItemData` extension, so split the data part into its own task.
- **M — Right-click equip/swap arms.** `SwapEquip`/`SwapVanityEquip`/
  `TryItemSwap`/`TryOpenContainer` (`ItemSlot.cs:3263-3364`, `:1660-1697`) over
  the equip pages; needs the `movement_action` seed above.
- **M — Crafting UI depth.** `NewCraftingUI`'s select-vs-craft split
  (`_selectedRecipeIndex` vs `focusRecipe`, `:499-528`), Shift ×10 craft-all
  (`Main.cs:42271-42275`), the filter/search hit tests, and the guide item.
  Depends on the `ui/crafting.rs` geometry tests already present.
- **M — `retainFavoriteOnMoveAt` + item-level favourites.** Decide whether to
  carry `Item.favorited` on `ItemSlot` (a `.plr`/save change, BUG-03 neighbour)
  or keep slot-index favourites and document the divergence in ANALYSIS.
- **S — Coin-slot merge.** `CoinSlot.ForceSlotState`'s rank arithmetic
  (`ItemSlot.cs:1016`, `:1119`) so mixed coin stacks degrade correctly.
- **S — Max-stack audit.** Replace `MAX_STACK` with `registry::max_stack` at
  `clicks.rs:934` and add a test with a sub-999 kind.
- **L — Equipment/dye page parity (M7-UI-3 half).** The dye drag, capture mode
  and the housing `SetMouseNPC` click; blocked on `Item.dye` and `ItemDef`
  extras.
