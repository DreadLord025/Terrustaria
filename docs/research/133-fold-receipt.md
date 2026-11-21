# 133-fold-receipt — the f11 unique folds EXECUTED on the c42 union

> Fold receipt · 2026-11-21 · code-only (no cargo/tests/rustfmt/graphify
> run) · branch `task/W4-c42-wiring-device-arms`, base `fadaa4bd` (the
> union EXECUTED tip). Consumes **research/133** (`dda240ee` on
> `task/W4-f11-wiring-arms-4`, compared state `6bc1547d`) — read FULL;
> every §1/§2 hunk grep-verified ABSENT on this tree before landing, then
> applied per-hunk from `git show ad80daaa` (core, +554/−10) /
> `git show 24fa39f1` (app, +285/−5), adapted to the union shapes.
> Companion record: research/95 §10.1 (this branch).

## 1. Per-hunk landing table (f11 source → union target)

| 133 § | f11 hunk (at `6bc1547d`) | Union landing (at `e6169bd1`) | Commit |
| --- | --- | --- | --- |
| 1a | `wiring.rs:465-474` `clear_all` | `tr-core/src/wiring.rs:479-489` (after `check_mech`, before `update` — same slot) | `e5569c2a` |
| 2e | `app/mod.rs:2323-2327` hook | `app/mod.rs:2322-2327` `enter_loaded_world` tail, after `reset_map_delay` | `e5569c2a` |
| 1b | `wiring.rs:492-508` pop + fluff-1 | `wiring.rs:505-523` inside `update` (`in_world` `:515`, `pop` `:520`) — the out-of-world branch ONLY; the expiry-branch `mechs.remove` (`:578`) stays (f11 kept it too, `f11-wiring.rs:564`) | `90c4fb12` |
| 1c | `wiring.rs:2015-2024` unlatch | `wiring.rs:2041-2049` at the `logic_gate_pass` tail (after `gates_done.clear()`) | `2458b596` |
| 1c re-pin | `wiring.rs:3709` plate witness | `wiring.rs:4050` `destroy_plate_releases_a_pressed_plate_and_pokes_the_switch` — block FALSE after the poke; no-poke arm sets TRUE manually | `2458b596` |
| 1d hunk 1 | `wiring.rs:2157-2168` entry clears | `hit_wire` entry (`wiring.rs:2182-2192`) — both clears DELETED, the census comment in their place; the exit clears (`:2294-2295`) stay | `4e23f31e` |
| 1d hunk 2 | `wiring.rs:2257-2265` Add pin | the `_toProcess.Add` site comment (`wiring.rs:2282-2289`) — provably-absent key, `insert` ≡ `Add` | `4e23f31e` |
| 1e | `wiring.rs:1684-1686` + doc `:1648-1652` | `launch_rocket` tail `skip.insert((i, j))` (`wiring.rs:1707-1709`) + the doc-comment correction | `b048c1b5` |
| 1f | `wiring.rs:1697-1703` box 2×2 | `launch_rocket_small` — the two right-column inserts added (`wiring.rs:1724-1725`); fountain comment-only fix (`:1762-1764`), the 1×2 code untouched | `b048c1b5` |
| 1g | `wiring.rs:3483-3663` mass core | `wiring.rs:3383-3563` — `MassStepOutcome` `:3387`, `mass_wire_operation_step` `:3406`, `mass_wire_operation_inner` `:3480`; after `dx_y_to_dir`, before the light-toggle family (the union's post-`dx_y_to_dir` region is the §10 item-6 unified family — the insert sits ahead of it, `mod tests` further down) | `05ae5cc6` |
| 2a | `app/mod.rs:878-882`/`:4002` + `flow.rs:2420` | `app/mod.rs:878-882` field + `:4002` ctor RED + `flow.rs:2420` test-ctor RED (both App literals covered — grep-verified only two exist) | `05ae5cc6` |
| 2b | `player.rs:1000-1007`/`:2638` + `world_sim/player.rs:574-578` | `player.rs:1000-1007` field + `:2638` spawn 0; `world_sim/player.rs:574-578` ResetEffects decrement beside the rod lock | `05ae5cc6` |
| 2c | `editing/wiring.rs:24-29`/`:40-41`/`:56`/`:187-209` | doc rewrite `:23-29`, `MulticolorWrench` variant `:40-41`, `3625 =>` decode `:56`, use arm `:187-210` (cooldown gate → Actuator-strip → single-cell op) | `05ae5cc6` |
| 2d | `wiring_devices.rs:351-471` | **MODULE MOVE**: f11's `world_sim/wiring_devices.rs` never existed on this branch (95 §10 item 4 — the e20 hopper/extractinator halves live in `world_sim/wiring.rs`); the append lands THERE after `spawn_device_kind`: `can_do_wire_stuff_here_world` `:991`, `consume_one` `:1010`, `mass_wire_operation` `:1042`. The 3625 arm calls `crate::world_sim::wiring::mass_wire_operation` | `05ae5cc6` |
| 1h | anchors rows (2) | core row verbatim; app row RE-POINTED at `crates/tr-app/src/world_sim/wiring.rs\|pub(crate) fn mass_wire_operation` (the §1h move clause — same commit as the code) | `05ae5cc6` |
| §3 item-6 | `edit.rs:953-972` pub(crate) helper | `edit/shift.rs:33` `pub(crate) fn top_left_and_styles` (renamed from the private `get_top_left_and_styles`, `.max(1)` guard added; both call sites re-pointed; both pass inc 18 — behaviour unchanged) | `a4b8a602` |

## 2. §3 f11-KEEP-shapes verification (the four rulings)

| Item | Ruling | Union state | Action |
| --- | --- | --- | --- |
| Doors Option skip | §1 f11 Option shape | PRESENT — `edit.rs:721`/`:858` `wire_skips: Option<&mut Vec<(i32,i32)>>`, the `if self.running { Some(...) } else { None }` arm gates; wrappers gone (item-1 `f982f1d4`) | none (verified) |
| `only_close_or_open` | §2 c42 body + f11 param | PRESENT — `edit/shift.rs:91` param, the `!= 1`/`!= 0` gates `:103`/`:179` (item-2 `ed88ec3d`) | none (verified) |
| Anchor helper | §2 f11 pub(crate) placement, one copy | WAS c42-private `get_top_left_and_styles` — the reusable placement + guard were the missing half | landed `a4b8a602` |
| Torch set-guard | §4 f11 `SET_TORCHES.has(k)` form | PRESENT — `wiring.rs:2782` `k if crate::tiledata::SET_TORCHES.has(k) =>` (item-6 `6381b52d`) | none (verified) |
| Slab-130 comment | §4 f11 comment tie-break | The union's 130 arm carries the c42 comment WITH the CanKillTile conjunct documented outside the above-disjunction (`wiring.rs` 130 arm: `above_ok` match + the separate `can_kill_tile` conjunct) — semantically the f11 doc shape | none (verified) |
| Door-cue queue | §1 c42 drain-site union superset | c42 `door_cues` ×11-site drain set untouched | none (verified) |

## 3. §5 cross-note compliance (c42 fix round FIRST)

No `hit_wire_single`/`de_active` arm was touched by this fold: the 144
timer wake, the `421 | 422 if !actuator` guard + wake, the 235/226
`downed_plant_boss` conjuncts and the 506 `validate_tile_square`
kill-cascade all stay c42-canonical. The f11 multi-tile witness flips a
NON-actuated 422 lamp — identical behaviour under the c42 guard; landed
unchanged (adapted fixture only: the module's own `world()` helper).

## 4. Witness ledger (133 §6 adapted to this tree; ALL WRITTEN NOT RUN)

**KEEP-unique** (133 §6 says "10 core"; the f11 diff actually adds **11**
tests to `w4_f11_residual_tests` — the doc's count missed one, the same
off-by-one class `cf0737da` fixed on the f11 lane) — 11 core
(`tr-core/src/wiring.rs`, `mod w4_f11_residual_tests`):

| Witness | f11 line | Union line |
| --- | --- | --- |
| `w4_f11_clear_all_empties_the_mech_and_pump_queues` | 7045 | 7747 |
| `w4_f11_the_gate_pass_clears_the_teleport_block` | 7064 | 7765 |
| `w4_f11_the_out_of_world_mech_drops_the_tail_not_the_dead_slot` | 7086 | 7787 |
| `w4_f11_the_mech_gate_is_fluff_one` | 7105 | 7806 |
| `w4_f11_a_multi_tile_trip_seeds_every_wired_cell_of_the_rect` | 7119 | 7846 |
| `w4_f11_the_mass_step_places_in_the_csharp_channel_order` | 7147 | 7874 |
| `w4_f11_the_mass_step_halts_on_a_dry_stock` | 7177 | 7904 |
| `w4_f11_the_mass_cut_half_caches_the_kills_and_never_bills` | 7198 | 7925 |
| `w4_f11_the_mass_inner_walks_the_l_not_the_filled_rect` | 7226 | 7953 |
| `w4_f11_the_mass_single_cell_click_places_once` | 7267 | 7993 |
| `w4_f11_the_mass_step_skips_a_refused_cell` | 7293 | 8019 |

(the union module holds 12 tests — the 11 residual pins above + the §3
chest-130 pin below; on f11 the chest-130 pin lived in the dup-arm
module, hence its absence from the residual count).

3 use-arm (`tr-app/src/world_sim/editing/wiring.rs`):
`the_multicolor_wrench_places_the_tool_mode_channels` (f11 493 → 493),
`the_multicolor_wrench_cut_drops_the_cached_kills` (520 → 520),
`the_wire_operations_cooldown_refuses_the_wrench` (539 → 540).

2 mass-wire unit (`tr-app/src/world_sim/wiring.rs` — the module move):
`w4_f11_consume_one_takes_the_first_matching_cell` (f11
`wiring_devices.rs:518` → `wiring.rs:1941`),
`w4_f11_the_mass_operation_bills_the_placed_channels` (539 → 1964).
Adapted imports: `crate::item::{EMPTY_SLOT, ItemSlot}` +
`crate::item::INV` (the f11 file's local `use` set does not exist here).

**Re-pinned base**: `destroy_plate_releases_a_pressed_plate_and_pokes_the_switch`
(f11 3709 → union 4050).

**KEEP-in-dup 3** (§3): `w4_f11_the_door_direction_draw_is_deterministic`
(union 7463, item-1 `f982f1d4` — verified), 
`w4_f11_an_actuated_torch_actuates_instead_of_toggling` (union 7414,
item-6 `6381b52d` — verified),
`w4_f11_a_chest_above_refuses_the_130_swap` (f11 6695 → union 7822,
landed `e6169bd1` — the c42 combined pin
`c42_the_active_stone_block_swaps_and_wakes` covers the same ground and
stays; the 95 §4 KEEP list names the f11 pin explicitly).

**Already-rebased on e20 / nothing to do**: the device/track witness set
(95 §10 item 4, `e8e6b159`); f11's §4 deprecated halves never existed on
this branch (no `tr-core::minecart`, no `wiring_devices.rs`) — nothing to
drop.

## 5. Commit hashes (this fold, base `fadaa4bd`)

1. `e5569c2a` — fix-family 1: clear_all + enter_loaded_world hook
2. `90c4fb12` — fix-family 2: UpdateMech pop + fluff-1
3. `2458b596` — fix-family 3: gate-pass teleport unlatch + plate re-pin
4. `4e23f31e` — fix-family 4: HitWire entry clears + Add pin
5. `b048c1b5` — fix-family 5: 216/fireworks SkipWire
6. `a4b8a602` — item-6 finish: pub(crate) top_left_and_styles + guard
7. `05ae5cc6` — fix-family 6: mass-wire core + app + 3625 + fields + canary
8. `e6169bd1` — witness-union finish: the chest-130 f11 pin

Verification performed: code-only lane — per-hunk absence greps before
landing, shape/caller/import greps after (both App literals, all Player
constructors ride `Player::spawn`, `ItemSlot`/`EquipSlots`/`max_stack`/
`spawn_item_drop_px` signatures matched, `multi_tool_mode` bits verified
against `WiresUI.cs:13-21` order). No cargo/tests/rustfmt/graphify run
(lane rule); every witness WRITTEN NOT RUN.

## 6. FINAL addendum (the wiring final phase, on top of `79b81ae7`)

The 133 fold is COMPLETE and stays canonical — the final phase touched no
fold-133 region (the mass-wire core/app, the lifecycle folds, the 3625 use
arm all untouched). What landed after this receipt:

1. `c6f40d0a` — the research/95 §8 seam closures: §8.1 the stamped-boxes
   entity gate (the wired trapdoor/tall-gate arms + the app stamp at every
   trip entry), §8.2 the `SendData(19)` echo queue (the five sites'
   subtype/direction computed verbatim; SP-discard drains at every cue
   site), §8.4 the two missing door-cue drains (logic sensors, plate
   break) + the `DestroyPlate` poke moved BEFORE the kill (the C#
   `KillTile_GetItemDrops` order — the release trip saw a dead tile),
   §8.9 `is_hopper_in_range_of` (PORTED-UNWIRED; the app `HOPPER_BANNED`
   aliases the new core `ITEMS_THAT_SHOULD_NOT_BE_IN_INVENTORY` — one
   copy). 6 witnesses.
2. `f173fb88` — the g78 research/76 contested-seam request: the monolith
   right-click use arm (`TileInteraction::MonolithToggle`,
   `use_switch_monolith` over this fold's untouched e21 body,
   `Player.quick_grapple_cooldown` + the `Update` decrement/map re-arm).
   1 witness. The 657 `Check2xX` break check stays the break-lane
   contract (research/76's co-lane list).
3. The census + final merger-checklist: research/95 §11-12 (the 52-member
   full-file diff — no gameplay gap; the single-branch-keep instruction,
   the 282-witness dedicated-module total, the gate-command).

**Single-branch keep**: `task/W4-c42-wiring-device-arms` — merge this
branch only; the f11 deprecated halves (this receipt's source lane) never
re-land. Gate: `cargo test --workspace --skip
migrates_version_2_kinds_to_item_ids` (the merger runs it; this lane is
code-only, every witness WRITTEN NOT RUN).