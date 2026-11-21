# 133 — The f11 unique folds (union-sync → c42c canonical) — FINAL-FINAL

> **FINAL-FINAL (retire phase, 2026-11-21)**: §1-2 EXECUTED on c42
> (`e5569c2a`…`e6169bd1`, receipt `133-fold-receipt.md` on
> `task/W4-c42-wiring-device-arms` tip `79b81ae77` = c42e-final);
> §3 KEEP-shapes verified present; §4 DEPRECATED inventory stands;
> §5 cross-note honoured (the c42 fix round untouched by the fold).
> **§7 is the NEW post-95 fold-list** — the five seam-wave uniques
> (`b1b140979`/`11788ce9d5`/`9ae0edc97`/`809d9a2cf`/`84ed6342b`) that
> landed on f11 AFTER the 133 comparison state and are ABSENT on
> c42e-final: entity-gate / 19-codec / forced-half / radial-logic /
> kite-param. The 95-supersede receipts are §7.0. Line numbers: f11 at
> `3da456c3c` (the code state — the retire-phase commits are docs-only),
> c42 at `79b81ae77`, g48 at its fetched tip.
>
> Union-sync deliverable · 2026-11-21 · code-only (no cargo/tests/graphify
> run) · branch `task/W4-f11-wiring-arms-4`, tip `6bc1547d` (the compared
> state). Consumes **research/95** (`ec568b3e` on
> `task/W4-c42-wiring-device-arms` — the wiring MASTER reconcile) and
> **research/71** (`cf0b8f9c` on `task/W4-e20-wiring-device-arms-2` —
> **STATUS: EXECUTED (merge-IN phase)**: e20 is the full canonical device
> superset; the f11 must-merge wins are ALREADY merged there — see §4).
>
> c42c union-exec is ACTIVE on the c42 branch; c42 is becoming canonical:
> doors f11-shape adopted, trapdoors c42-wins, tall gates c42, devices
> e20-`cf0b8f9c`-superset. This doc is the f11 side of that contract:
> §1-3 = the exact hunks c42c must cherry-pick (none exists on any other
> tip); §4 = the f11 duplicate halves marked DEPRECATED-BY-UNION (kept,
> not deleted — the merger drops them per the 95 playbook); §5 = the old
> 144/421/422/506 validator cross-note (c42 fix round FIRST).

**Verified tips** (`git ls-remote`/fetch, 2026-11-21):
c42 `ec568b3e` (= `12b45b21` code + the 95 doc), e20 `cf0b8f9c`, f11
`6bc1547d`. All line numbers below are at f11 `6bc1547d` unless prefixed
`c42@ec568b3e`/`e20@cf0b8f9c`. Every witness named is WRITTEN NOT RUN on
its source branch (code-only lanes).

---

## 1. UNIQUE — core lifecycle fixes + mass-wire core (`ad80daaa`)

**None of these exists on c42@`ec568b3e`** — verified by grep over its
whole `crates/` tree: no `clear_all` on `Wiring` (only the unrelated
`combat_text::clear_all`/`binio::clear_all`), no `mechs.pop`, no
`mass_wire_operation*`, no gate-pass teleport clear. Cherry-pick source:
`git cherry-pick ad80daaa` (single file, `crates/tr-core/src/wiring.rs`,
+554/−10) — or the per-fix hunks below if the union has already reshaped
the file. All eight fixes are independent of the 95 §1-4 dup arms.

### 1a. `Wiring::clear_all` — the world-load reset
- C#: `Wiring.ClearAll` (`Wiring.cs:125-143`), called by
  `WorldGen.clearWorld` (`WorldGen.cs:6959`; the `WorldFile.cs:1977`/
  `:3455` load sites).
- Hunk: NEW `pub fn clear_all(&mut self)` at `wiring.rs:465-474` (after
  `check_mech`, before `update`): clears `in_pumps`, `out_pumps`, `mechs`.
  The C# zeroes the fixed 999-slot arrays; the port's `Vec::clear` is the
  same observable (the zero rows are dead slots either way).
- App consumer (KEEP, §2): `app/mod.rs:2327` inside `enter_loaded_world`.
- Witness: `w4_f11_clear_all_empties_the_mech_and_pump_queues`
  (`wiring.rs:7045`).

### 1b. `UpdateMech` out-of-world: pop-not-remove + fluff-1 (research-23 D16)
- C#: `Wiring.cs:165-175` — the out-of-world/null branches do
  `_numMechs--` WITHOUT shifting; the gate is `InWorld(x, y, 1)` (fluff 1).
- Hunk: `wiring.rs:492-508` inside `update` — replaces
  `if !world.in_bounds(mech.x, mech.y) { self.mechs.remove(num); … }`
  with the fluff-1 `in_world` test + `world.tile(...).is_none()` disjunct
  and `self.mechs.pop()` (`:506`). The descending loop never revisits
  index `num`; the dead entry survives until the tail drains past it —
  `Vec::remove` (the old fold) shifted the tail and kept the LAST entry.
- Witnesses: `w4_f11_the_out_of_world_mech_drops_the_tail_not_the_dead_slot`
  (`:7086`), `w4_f11_the_mech_gate_is_fluff_one` (`:7105`).

### 1c. `LogicGatePass` tail clears the one-iteration teleport block
- C#: `Wiring.cs:714-717` — the plate poke
  (`PressurePlateHelper.PokeLocation`) and the PlayerAbove sensor arm
  `blockPlayerTeleportation`; the pass drops it after the chained trips.
- Hunk: `wiring.rs:2015-2024` at the `logic_gate_pass` tail (after
  `self.gates_done.clear()`): `if self.block_player_teleportation { self.block_player_teleportation = false; }`.
  Without it the flag latched forever and every later teleporter pad
  refused to move the player.
- Re-pin (same commit): the base witness
  `destroy_plate_releases_a_pressed_plate_and_pokes_the_switch`
  (`:3709`) now asserts the block reads FALSE after the poke, and the
  no-poke arm arms the flag manually (`set_block_player_teleportation(true)`)
  to prove the unpressed plate leaves it untouched. If c42c's tree still
  carries the old assertions, take the re-pin with the fix.
- Witness: `w4_f11_the_gate_pass_clears_the_teleport_block` (`:7064`).

### 1d. `HitWire` entry/exit map lifecycle (research-23 D1/D2 tail)
- C#: `Wiring.cs:839` clears ONLY `_wireDirectionList` at entry;
  `_wireSkip`/`_toProcess` clear at the EXIT (`:977-978`).
- Hunk 1: `wiring.rs:2157-2168` in `hit_wire` — DELETE the entry
  `self.skip.clear(); self.to_process.clear();` (redundant on every
  standalone path; the exit clears at `:2269-2270` stay). The seed loop
  re-inserts every seed unchanged.
- Hunk 2: `wiring.rs:2257-2265` — comment-only pin at the
  `_toProcess.Add` site (`:971-974`): the key is provably absent (an entry
  exists ⇔ the `TryGetValue` arm `continue`d; removed exactly at zero),
  so `insert` cannot diverge from the C# `Add`'s throw-on-duplicate.
- Recorded quirk (ANALYSIS "The Wiring.cs device residuals (W4-f11)"):
  the C#'s nested gate-trip shares the static maps and wipes the outer
  flood's state on its own exit; the port's per-call structure does not
  reproduce it. Do not "fix" at merge.
- Witness: `w4_f11_a_multi_tile_trip_seeds_every_wired_cell_of_the_rect`
  (`:7119`).

### 1e. case-216 `LaunchRocket` — SkipWire AFTER the launch
- C#: `Wiring.cs:1492` — `SkipWire(i, j)` after the launch, hit cell only
  (the C# does not skip `j + 1`).
- Hunk: `wiring.rs:1684-1686` at the `launch_rocket` tail:
  `self.skip.insert((i, j));` (+ the doc-comment correction at
  `:1648-1652`). The port used to drop the skip entirely.
- No dedicated witness (the arm is exercised by the base rocket pins);
  the hunk is one line — verify by eye against `:1492`.

### 1f. fireworks box — the full 2×2 SkipWire
- C#: `Wiring.cs:1512-1515` — the box skips all FOUR cells.
- Hunk: `wiring.rs:1697-1703` in `launch_rocket_small`: ADD
  `self.skip.insert((num73 + 1, num72));` and
  `self.skip.insert((num73 + 1, num72 + 1));` (the port skipped only the
  left column). The fountain's 1×2 two-cell skip (`:1526-1527`) is
  CORRECT as-is — comment-only correction at `:1739-1742`, no code change.

### 1g. mass-wire core — `MassWireOperationStep`/`Inner` (research-23 D17)
- C#: `Wiring.MassWireOperationStep` (`Wiring.cs:3329-3421`) +
  `MassWireOperationInner` (`:3235-3327`).
- Hunk: NEW free fns at `wiring.rs:3483-3663` (after `dx_y_to_dir`,
  before `mod tests`): `pub type MassStepOutcome = Option<bool>` (`:3488`),
  `pub fn mass_wire_operation_step` (`:3507`) — fluff-1 `InWorld` gate,
  null-tile/`can_place` refusals → `None`; placement half (Cutter bit OFF)
  in the C# Red → Green → Blue → Yellow → Actuator order, one 530 per
  missing channel / one 849 per missing actuator, dry stock →
  `Some(false)` halt; cut half (Cutter ON) never bills, kills accumulate
  into `cut_cache` (`Item.StartCachingType`/`DropCache`, `:3244-3245`/
  `:3325-3326`). `pub fn mass_wire_operation_inner` (`:3581`) — the
  L-shaped walk (dir-true: ps column then pe row; half-open legs, the ps
  corner never re-walked), the endpoint runs once more unless halted;
  `ps == pe` (the Multicolor Wrench click) = one endpoint step.
- Consumers (KEEP, §2): `wiring_devices::mass_wire_operation` + the 3625
  use arm.
- Witnesses (6): `w4_f11_the_mass_step_places_in_the_csharp_channel_order`
  (`:7147`), `…_halts_on_a_dry_stock` (`:7177`),
  `…_cut_half_caches_the_kills_and_never_bills` (`:7198`),
  `w4_f11_the_mass_inner_walks_the_l_not_the_filled_rect` (`:7226`),
  `…_single_cell_click_places_once` (`:7267`),
  `…_the_mass_step_skips_a_refused_cell` (`:7293`).

### 1h. Canary rows (take with the code)
`anchors.txt` rows added by `6bc1547d`:
`crates/tr-core/src/wiring.rs|pub fn mass_wire_operation_inner|…` and
`crates/tr-app/src/world_sim/wiring_devices.rs|pub(crate) fn mass_wire_operation|…`.
The build canary reads them — cherry-picking the code WITHOUT the rows is
safe; dropping the code while keeping the rows FAILS the build. The
second row's file is the §4a deprecated one: when the merger drops the
hopper/extractinator halves it must KEEP `mass_wire_operation` in that
file (or move it and re-point the row in the same commit).

## 2. UNIQUE — app mass-wire + Multicolor Wrench + lifecycle seams (`24fa39f1`)

None of this exists on c42@`ec568b3e` (grep: no `wire_operations_cooldown`,
no `wire_tool_mode`, no 3625 arm). Cherry-pick source: `24fa39f1` — but
see the file-level caveat in §4a: four of its six files are clean keeps,
the fifth (`world_sim/wiring.rs`... actually untouched by this commit) —
the sixth, `wiring_devices.rs`, receives ONLY the mass-wire append.

### 2a. `App.wire_tool_mode` (WiresUI.Settings.ToolMode)
- C#: `WiresUI.cs:23` (the field), `:94` (the static ctor default Red).
- Hunks: `app/mod.rs:878-882` (field + doc), `:4002` (ctor init
  `multi_tool_mode::RED`), `app/flow.rs:2420` (the test-ctor init).
  The WiresRadial UI owns the setting — its LOGIC half landed W4-f11
  (`ui/wires_radial.rs`, research/234; the draw passes are the contract
  remainder).

### 2b. `Player.wire_operations_cooldown` + ResetEffects decrement
- C#: `Player.cs:2868` (field), `:18798-18800` (decrement),
  `:52075-52078` (the CanUseItem 3611/3625 refusal),
  `MessageBuffer.cs:3805` (the msg-110 clear — parked, net lane). No
  positive setter exists in 1.4.5.8; the field + gate keep the shape.
- Hunks: `player.rs:1000-1007` (field + doc), `:2638` (spawn init 0);
  `world_sim/player.rs:574-578` (the `update_player` ResetEffects-block
  decrement beside the rod lock).

### 2c. The Multicolor Wrench (3625) use arm
- C#: `Player.cs:46903-46918` — the ToolMode Actuator-strip (`:46908`,
  restored `:46917` — the port passes the masked mode straight through),
  `ApplyItemTime` (`:46906`) rides the caller's plain wiring-tool cadence
  (`editing.rs:5262-5273`, the 5-tick useTime `Item.cs:32673`); the
  single-cell `ps == pe` operation. The Grand Design 3611 drag stays with
  projectile 651 (`Projectile.cs:72053-72066`).
- Hunks: `world_sim/editing/wiring.rs:24-29` (doc rewrite), `:40-41`
  (`WiringTool::MulticolorWrench` variant), `:56` (the `3625 =>` decode),
  `:187-209` (the use arm: cooldown gate → Actuator-strip →
  `wiring_devices::mass_wire_operation(app, (tx,ty), (tx,ty), mode)`).
- Witnesses (3): `the_multicolor_wrench_places_the_tool_mode_channels`
  (`:493`), `the_multicolor_wrench_cut_drops_the_cached_kills` (`:520`),
  `the_wire_operations_cooldown_refuses_the_wrench` (`:539`).

### 2d. The mass-wire app half (in the §4a file — KEEP this append)
- C#: `Wiring.MassWireOperation` (`Wiring.cs:419-453`) +
  `Player.CanDoWireStuffHere` (`Player.cs:46921-46943`) +
  `Player.ConsumeItem` (`:9269-9308`).
- Hunks in `world_sim/wiring_devices.rs` (all AFTER the deprecated
  hopper/extractinator region — line numbers at `6bc1547d`):
  `can_do_wire_stuff_here_world` (`:351-368`, the wall-350 refusal + the
  wall-87 Golem gate; the dual-dungeons seed arm unported),
  `consume_one` (`:370-391`, the forward 58-slot scan —
  inventory → coins → ammo chained in the C# index order; a drained cell
  turns to air), `pub(crate) fn mass_wire_operation` (`:393-471`, the
  stock census `:423-433`, `master.Center`/`direction` `:436`, the core
  L-walk call, the bill `:437-452` as direct ConsumeItem loops — the i51
  netMode-0 fold of the two msg-110 bills — and the DropCache spawns in
  `maxStack` chunks on `UpdatePlayers`, the cutter precedent).
- Witnesses (2): `w4_f11_consume_one_takes_the_first_matching_cell`
  (`:518`), `w4_f11_the_mass_operation_bills_the_placed_channels` (`:539`).
- **Merger note**: when the hopper/extractinator halves drop (§4a), these
  three fns + two witnesses + the `use crate::registry`/`Vec2`/`ItemSlot`
  imports they need SURVIVE — either the file shrinks to the mass-wire
  module or the append moves; re-point the anchors.txt row (§1h) if it
  moves.

### 2e. The `ClearAll` world-load hook
- C#: `WorldGen.cs:6959` inside `clearWorld`.
- Hunk: `app/mod.rs:2323-2327` at the `enter_loaded_world` tail:
  `self.wiring.clear_all();`. Depends on §1a.

## 3. UNIQUE f11 halves inside the 95 dup arms (already ruled KEEP by 95)

The 95 playbook adopts these f11 shapes into the canonical bodies; they
are listed here so c42c's union has the exact source hunks:

| Item | 95 ruling | f11 hunk (at `6bc1547d`) |
| --- | --- | --- |
| Doors Option skip seam | §1 "Recommend f11's Option shape" — encodes the C# `Main.netMode != 1 && Wiring.running` gate at the call site | `edit.rs:714` (`open_door`'s `wire_skips: Option<&mut Vec<(i32,i32)>>`), `:797` collection; `:847`/`:936` (`close_door`); the arms pass `if self.running { Some(&mut skips) } else { None }` (`wiring.rs:2745`/`:2772`/`:2779`). ALL f11 callers already updated (`interact_at` ×3, NPC bash drain, test sites). If adopted, DROP c42's `open_door_wired`/`close_door_wired` wrappers (c42 `edit.rs:724`/`:886`). |
| Trapdoor `only_close_or_open` param | §2 "c42's body + f11's `only_close_or_open` param" (the C# default-arg is real API surface) | `edit.rs:987` (`only_close_or_open: i32`), the `!= 1` / `!= 0` gates at `:996` and the 387 branch. c42's `shift.rs` body hardcodes −1 — add the param when adopting. |
| Anchor helper placement | §2 "one copy — f11's pub(crate) placement is the reusable one" (either body; never both) | `edit.rs:953-972` `pub(crate) fn top_left_and_styles` (with the `.max(1)` divisor guard; c42's private `get_top_left_and_styles` is verbatim C# — no current caller passes inc 0). |
| Torch set-guard arm | §4 "f11's set-guard form is the C# shape" (`TileID.Sets.Torches[type]`) | `wiring.rs:2676` `k if crate::tiledata::SET_TORCHES.has(k) =>` (body identical to c42's `4 =>` literal ±66). |
| Slab-130 comment | §4 tie — "f11's comment is the better doc" (the CanKillTile conjunct OUTSIDE the above-active disjunction) | `wiring.rs:2695-2702` comment + `:2703-2712` arm (`is_some_and` truth table = c42's `None => false`). |
| Door-cue queue payload | §1 "one queue (identical payload/semantics; name either)" — but **c42's drain-site union is the superset, KEEP it** | f11 `door_sounds` (`wiring.rs:308`, `take_door_sounds` `:829`) drains ONLY at `world_sim/wiring.rs:349` (UpdateWorldGen). f11's single drain site is DEPRECATED-BY-UNION (§4b) — c42's ×11-site drain set wins. |
| f11-unique witnesses in dup arms | §1/§4 KEEP lists | `w4_f11_the_door_direction_draw_is_deterministic` (`wiring.rs:6850`), `w4_f11_a_chest_above_refuses_the_130_swap` (`:6695` — the CanKillTile gate pin), `w4_f11_an_actuated_torch_actuates_instead_of_toggling` (`:6717` — the ActuateForced-refusal interaction). These three MUST survive the dedupe. |

## 4. DEPRECATED-BY-UNION — the f11 duplicate halves (kept, NOT deleted)

Per the 95 §9 playbook the MERGER drops these; this lane only marks them
(deleting breaks the pushed-history consumers). The deprecation banners
live in the module docs + the ANALYSIS union-sync note; the code is
untouched and still compiles/behaves as pushed.

### 4a. `world_sim/wiring_devices.rs` — hopper/extractinator halves → e20
95 §5 delegates to research/71 (EXECUTED on e20@`cf0b8f9c`): KEEP e20's
`hopper_pull`/`extractinator_roll` + chest helpers; DROP f11's bodies.
Deprecated region (at `6bc1547d`): `run_hopper` (`:43`),
`try_to_put_item_in_chest` (`:110`), `try_adding_to_stack` (`:133`),
`try_adding_to_empty_slot` (`:164`), `try_move_coins_in_chest` (`:198`),
`coins_split` (`:268`), `run_extractinator` (`:287`),
`try_find_chest_for_extractinator` (`:339`), the `test_chest` helper
(`:476`) + witnesses `w4_f11_coins_split_matches_the_csharp_denominations`
(`:493`), `w4_f11_the_hopper_grab_rect_is_192_centred` (`:503`) and the
module header's hopper/extractinator RNG note.
**Already merged into e20 — nothing left to cherry-pick** (verified in
`cf0b8f9c`'s log): PlaySound(7) + the wiring-stream pitch draw
(`f167082e`), the OnceLock 64-mask cache (`6336e3b4`), the six f11
witnesses rebased (coins_split 1234567, grab-rect edges, locked-chest,
non-anchor hits, cooldown-EXPIRY re-fires — `b0d0836f` + `5272cd52`),
plus e20's own slot-Vec fix (`a2d6cdad`). The `TrapRequest::Spear` shape
change is e20's — f11 constructors update to e20 at merge, not reverse.
**KEEP in this file**: the §2d mass-wire append (unique).
The core request arms (`wiring.rs:2825-2850`, the `%36`/`%54` anchors +
`CheckMech(60)` + `hopper_requests`/`extractinator_requests` queues
`:315`/`:322`/`:836`/`:843`) and the app drains
(`world_sim/wiring.rs:362-367`) are the same seam shape e20 keeps —
the merger resolves them textually onto e20's versions; the f11
witnesses `w4_f11_the_hopper_resolves_its_2x2_anchor` (`:6951`) …
`w4_f11_the_extractinator_cooldown_gates_the_rehit` (`:7003`) and the
three app drains witnesses (`world_sim/wiring.rs:1086`/`:1117`/`:1143`)
were already rebased on e20 (`5272cd52`/`b0d0836f`).

### 4b. `tr-core/src/minecart.rs` — whole module → e20 TRACK_FRAMES
95 §6 / research/71 §1-2: KEEP e20's `TRACK_FRAMES` (the M2-PHYS-7
superset, consumed by `track_collision`) + `flip_switch_track`/
`frame_track_pound` (assert-on-front-absent ≈ the C# IndexOutOfRange;
f11's `.max(0)` clamp silently diverges). DROP the whole f11 module
(`lib.rs:34` `pub mod minecart;` goes with it): the four flat tables
(`:22-54`), `FIRST_PRESSURE_FRAME` (`:56`),
`nearby_tiles_set_lookup_index` (`:63`), `track_switch_options` (`:92` —
the OnceLock shape ALREADY merged into e20 `6336e3b4` over the e20
table), `frame_track` (`:146`), `flip_switch_track` (`:357`), the four
witnesses (`:406`/`:423`/`:435`/`:445` — the table pin already re-pointed
on e20 `5272cd52`). The core 314 arm (`wiring.rs:2726-2730`) re-points
at e20's `CheckMech(5)` → `switch_track_flips` request seam; the f11
cooldown witness (`:6738`) was rebased on e20.
**Salvage note (95 §8.8)**: `frame_track`'s non-pound branch + `flag4`/
`KillTile(fail)` (`Minecart.cs:976-1038`/`:1110-1166`) is the future
hammer-placement half (`edit/hammer.rs` `Consumed` seam) — a follow-up
task, NOT wiring code; use e20's assert convention when it lands.

### 4c. `edit.rs` shift copies → c42 `edit/shift.rs`
95 §2-3: c42's bodies win (paint + fullbright coating, in-body cues at
the C# sites incl. the double-thud refusal, `touch()`, alternate-0 def
decode). DEPRECATED: f11 `shift_trapdoor` (`edit.rs:982-1108` — paint
only, no cues, no touch) and `shift_tall_gate` (`:1118-1178` — no
style/alternate def lookup, arm-level cue at the HIT cell). The
`wire_trapdoor` fold (`wiring.rs:3062-3091` — ONE cue per outcome at the
hit cell) and the tall-gate arm's cue push (`:2816`) are wrong on
position/count per 95 and drop with the bodies; c42's in-body cues +
drain-site union replace them. **KEEP from f11**: the §3 param/placement
rows (`only_close_or_open`, `top_left_and_styles` pub(crate), the Option
skip seam if adopted).

### 4d. The dup light/slab arms + door arms → the 95 §1/§4 keep-one
f11 `toggle_campfire` (`wiring.rs:3098`) ≡ c42 `toggle_camp_fire`
(line-for-line semantics — keep ONE, add d6's forced gate per 95 §4);
the 130/131 arms (`:2703-2719`) tie with c42 (keep one + the f11 comment
and the two unique witnesses §3); the door arms 11/10 (`:2734-2786`)
tie with c42 (keep one; the f11 deterministic-draw witness survives).
The torch arm's set-guard FORM is the keeper (§3) — the body is a tie.

## 5. Cross-note — f11's OLD 144/421/422/506 (+235/226) vs the c42 fix round

95 §9.1: **c42 fix round FIRST** — f11's tip still carries the OLD code
in these arms; a merge taking f11's arm region wholesale silently
reverts the fixes. Verified both sides (grep at each tip):

| Arm | f11@`6bc1547d` (OLD — flagged) | c42@`ec568b3e` (fix landed, `de176ccd`) |
| --- | --- | --- |
| 144 timer | `wiring.rs:2304` — bare `hit_switch`, NO `SquareTileFrame` wake | `hit_switch` + `square_tile_frame_default` drops → `actuation_drops` (C# `:1012-1016`) |
| 421/422 lamp | `:2306-2307` — unconditional swaps, no actuator guard, no wake | `421 \| 422 if !actuator` (c42 `:2226`) + the wake (C# `:1017-1032`) |
| 235 teleporter | `:2347` — `wall == 87 && j > surface` only; the `downedPlantBoss` conjunct pending | `&& !world.downed_plant_boss` (c42 `:2270`; C# `:1550-1553`) |
| 226 bulb (DeActive) | `:3346` — always guarded below the surface | `&& !world.downed_plant_boss` (c42 `:3223`; C# `:3197`) |
| 506 plate | `:2863-2869` — inline active+kind loop, mismatch just refuses | shared `edit::validate_tile_square` (c42 `:2781`) — mismatch KILLS every matching cell, drops collected (C# `:2517`) |
| validator visibility | f11 `edit.rs:1185` already `pub(crate)` (own batch-1 bump) | c42 `edit.rs:1014` `pub(crate)` — identical edit, lands ONCE (trivial conflict) |

**Union instruction**: in every `hit_wire_single`/`de_active` conflict
hunk covering these arms take **c42's side**; f11's neighbouring unique
hunks (§1) are independent and cherry-pick cleanly around them. The f11
witnesses over these arms were written against the OLD behaviour — none
pins the 144 wake / 421 actuator guard / 506 kill-cascade (the f11
multi-tile witness `:7119` flips a NON-actuated 422 lamp, which behaves
identically under the c42 fix; it survives).

## 6. Witness ledger (this doc's claims)

All WRITTEN NOT RUN (code-only). KEEP-unique (15, `ad80daaa`+`24fa39f1`):
the 10 `w4_f11_residual_tests` (`wiring.rs:7015-7308`) + 3 use-arm
(`editing/wiring.rs:493`/`:520`/`:539`) + 2 mass-wire unit
(`wiring_devices.rs:518`/`:539`); plus the re-pinned base plate witness
(`wiring.rs:3709`). KEEP-in-dup (3, §3): the deterministic door draw, the
chest-above-130 refusal, the actuated-torch interaction. Already-rebased
on e20 (nothing to do): the device/track witness set (`b0d0836f`,
`5272cd52`). Dropped-with-body per 95: f11's trapdoor/tall-gate/open-close
overlap witnesses (the c42 versions pin the skip sets; 95 §1-3 union
tables name the keepers).

*(Count correction, c42 receipt §4: the residual module actually holds
11 tests — the "15" above undercounts by one; the c42 ledger has the
row-for-row table. The §6 line numbers are at `6bc1547d`, superseded by
the receipt's union lines.)*

---

## 7. POST-95 FOLD-LIST — the five seam-wave uniques for c42e/the merger

The five f11 commits AFTER `6bc1547d` (the §1-6 comparison state) closed
95 §8.1/§8.2-send/§8.5 on this branch and landed the radial/kite halves.
**None of these hunks exists on c42e-final `79b81ae77`** — grep-verified
at that tip: no `cell_blocked`/`install_entity_gate`/`EntitySnapshot`,
no `door_toggles`/`DoorToggle` in `tr-core`, no `DOOR_TOGGLE` in
`net.rs`, no `forced:` on the light methods (the forced state lives in
the c42 free-fn family), no `wires_radial`, no `drop_rng` param on
`mass_wire_operation`. The KEEP-list grows by the five families below;
the §4 DEPRECATED inventory is UNCHANGED (the f11 branch is deprecated
per 95, but these uniques must survive the dedupe).

### 7.0 The 95-supersede receipts (what the c42 exec already absorbed)

| f11 item (this doc) | Superseded by (c42e-final) | Receipt |
| --- | --- | --- |
| §1a-1h + §2a-2e (the 8 lifecycle fixes + mass-wire core/app + 3625 + fields) | fold-133 EXECUTED `e5569c2a`/`90c4fb12`/`2458b596`/`4e23f31e`/`b048c1b5`/`05ae5cc6` — per-hunk, adapted (the app mass-wire half MOVED to `world_sim/wiring.rs:991-1103`, the canary row re-pointed) | `133-fold-receipt.md` §1/§5 |
| §3 doors Option seam | union-exec item 1 `f982f1d4` (wrappers dropped, all callers `None`) | 95 §10 |
| §3 `only_close_or_open` | item 2 `ed88ec3d` (c42 body + the param) | 95 §10 |
| §3 anchor helper | item 6 finish `a4b8a602` (`pub(crate) top_left_and_styles` + `.max(1)` in `edit/shift.rs:33`) | receipt §1 |
| §3 torch set-guard / slab comment / door-cue union | items 6/4 — verified PRESENT, no edit | receipt §2 |
| §3 KEEP-in-dup witnesses ×3 | items 1/6 + `e6169bd1` (the chest-130 pin landed last) | receipt §4 |
| §4a/4b/4c/4d DEPRECATED halves | never existed on c42 (no `tr-core::minecart`, no `wiring_devices.rs`) — nothing to drop there; the f11-side banners stand | receipt §4 |
| §5 OLD 144/421/422/506 cross-note | honoured — the fold touched no `hit_wire_single`/`de_active` arm | receipt §3 |
| research/233 §1a (the 8 skips collectors) + §1b (the free-fn drop/re-point) | **SUPERSEDED by union-exec item 6 `6381b52d`**: c42 chose the FREE-FN family as the one copy (the methods are thin wrappers), not f11's method-centric shape. Do NOT apply 233 §1a/§1b — the surviving f11 unique is the forced-gate WITNESS (§7.3) | 95 §10 item 6 |

### 7.1 entity-gate (`b1b140979`, 95 §8.1 CLOSED) — 14 install sites

**Core** (`tr-core/src/wiring.rs`, f11 lines):
- `cell_blocked: Option<CellBlocked>` field `:323-333` (after
  `extractinator_requests`) + the `CellBlocked` Debug-transparent box
  `:358-366`. c42 slot: after its `extractinator_requests` (`:329`).
- `install_entity_gate`/`clear_entity_gate`/`entity_blocks` `:494-518`
  (after `rand_mut`, before `check_mech`; c42 slot `:462-468`).
- The tall-gate arm `:2916-2927`: `let blocked = |x,y| self.entity_blocks(x,y);`
  replaces the always-free closure. c42 target: `wiring.rs:2656`
  (`&|_, _| false` in the 388/389 arm).
- The trapdoor arm: f11's `wire_trapdoor` fold is DEPRECATED (§4c) —
  the gate lands on **c42's inline 386/387 arm** instead: BOTH
  `&|_, _| false` args (`:2617` first try, `:2630` negated retry)
  become `&blocked` with the same one-line closure above the arm.
  The c42 `shift_trapdoor` signature already takes `cell_blocked`
  (`edit/shift.rs:92`, read at `:116`/`:153`); `shift_tall_gate` at
  `:284`/`:326` — no edit-layer change needed.
- Door 11 (`:2684` on c42) keeps `&|_, _| false` — `forced:true` skips
  the gate (C# `Wiring.cs:1463`); door 10 has no occupancy gate.
- Witnesses ×2 (f11 `w4_f11_door_gate_tests`, `:7232`/`:7270`):
  `w4_f11_the_entity_snapshot_gates_the_tall_gate`,
  `w4_f11_the_entity_snapshot_gates_the_trapdoor_not_the_forced_door`.
  Re-point: the trapdoor rig calls `hit_wire_single` (unchanged API);
  the door-close leg uses the c42 `door_cues` name if it asserts cues
  (it does not — kind assertions only). KEEP both.

**App** (`tr-app/src/world_sim/wiring.rs`, f11 lines):
- `EntitySnapshot` + `capture`/`into_closure` `:16-54` (XNA
  `Rectangle.Intersects` strict-overlap math; the player box uses
  `crate::player::WIDTH/HEIGHT`, NPCs `kind.size()`),
  `install_entity_snapshot`/`clear_entity_snapshot` `:56-70`,
  `install_entity_snapshot_parts` `:72-83` (the split-borrow form).
  c42 slot: the module head (its `world_sim/wiring.rs` already holds
  the e20 device halves + the mass-wire append — prepend before
  `update_pressure_plates`).
- The wiring-tick pair `:180-182` around `app.wiring.update` — c42
  target `world_sim/wiring.rs:112`.
- Witness `w4_f11_the_npc_box_blocks_the_wired_tall_gate` (`:769`) —
  the real-flood NPC-box pin + the touching-edges-free rectangle math.
  KEEP (fixture adapts to the c42 module's `app_with`/`npc_of` helpers
  if named differently).

**The 14 production install/clear pairs** (f11 file:lines → c42 target):

| # | Site | f11 | c42e-final target |
| --- | --- | --- | --- |
| 1 | wiring tick (`update`) | `world_sim/wiring.rs:180/182` | `world_sim/wiring.rs:112` |
| 2 | player switch scan (`switch_tiles`, objType 5) | `world_sim/player.rs:1128/1144` (parts) | `:1125` scan |
| 3 | weighted plates (`update_pressure_plates`) | `player.rs:1174/1189` (parts) | `:1173` call |
| 4 | track-switch trip (`take_switch_trip`) | `player.rs:1346/1352` (parts) | `:1346-1347` |
| 5 | NPC switch scan (objType 2, one capture per pass) | `world_sim/npcs.rs:4655/4691` (parts) | `:4654` scan |
| 6 | golf cup (`GolfEffect::CupScored`) | `world_sim/projectiles.rs:637/639` | `:616` `hit_switch` |
| 7 | projectile pressure pad (`switch_tiles`, objType 4) | `projectiles.rs:669/681` | `:653` scan |
| 8 | plate destroy (`break_tile` 428) | `world_sim/editing.rs:5742/5744` | `:5740` `destroy_plate` |
| 9 | lever/switch/timer tile use | `editing/doors.rs:221/223` | `:220` `hit_switch` |
| 10 | logic sensors (`update_logic_sensors`) | `world_sim/tile_entities.rs:32/36` | `:31` call |
| 11 | rigged chest (cursor open) | `chest.rs:1164/1166` | `:1162` `hit_switch` |
| 12 | rigged chest (net load trip) | `app/netglue.rs:646/648` | `:644` `hit_switch` |
| 13 | msg-59 host apply (`C2s::HitSwitch`) | `netglue.rs:790/793` | `:789` arm |
| 14 | msg-59 client replay (`S2c::HitSwitch`) | `netglue.rs:1788/1791` | `:1787` arm |

The g4 NPC-bash drain keeps its OWN real closure (landed `23411fa7`) —
it is not a snapshot site (no flood runs there); do not double-install.
g48's `apply_door_action` 2/3 reader likewise keeps its own
`entity_cell_blocked` (the message handler is not a flood entry).
Canary: the `pub fn install_entity_gate` anchors row (f11 tail) lands
verbatim with the core hunk.

### 7.2 19-codec (`11788ce9d5`, 95 §8.2 CLOSED send-half) — g48 keep-one

**Core** (`tr-core/src/wiring.rs`, f11 lines):
- `door_toggles: Vec<DoorToggle>` field `:334-342` + `pub struct
  DoorToggle` `:344-356` (`action: u8, x/y/direction: i32` — the ENGINE
  request shape; the WIRE shape is `net.rs`'s i16/u8) +
  `take_door_toggles` `:900-909`. c42 slots: after `door_cues`
  (`:315`), the drain beside `take_door_cues` (`:842`).
- Door 11 arm: the success-only `action: 1` push (f11 `:2844-2846`) —
  c42 target: inside the success block `:2688-2693` beside the
  `door_cues.push((false, …))` (`:2692`).
- Door 10 arm: the two attempts SEPARATE so the echo carries the
  succeeded lean (f11 `:2863-2891`: `let mut opened` + `dir` flip on
  retry, the `action: 0, direction: dir` push on success). c42 target:
  its `:2705-2739` arm currently folds `opened || retry` — split it
  the f11 way (the retry stays UNCONDITIONAL, the validate-kill
  semantics unchanged).
- Trapdoor arm: c42's inline arm (`:2606-2642`) gains the `num66`
  computation (f11 `wire_trapdoor:3272-3297`: `+1` on the first
  success, `−1` on the negated retry's success, 0 on full refusal) +
  the success-only `action: 3 − (kind == 387)` push with `num66`
  (f11 `:3300-3309`). The f11 `wire_trapdoor` fn itself DROPS (§4c) —
  only the payload maths folds.
- Tall-gate arm: the UNCONDITIONAL `action: 4 + (kind == 389)` push
  (f11 `:2933-2941`, C# `Wiring.cs:1458` sends whether or not the shift
  ran) — c42 target after its `:2660` shift call, before the cue join
  (`:2665`).
- Witness `w4_f11_the_door_arms_queue_the_19_echo_payloads` (f11
  `:7311`) — re-point `take_door_sounds` → `take_door_cues` where
  asserted (it asserts toggles only; the entity-gate leg needs §7.1).

**App**:
- `net.rs`: `id::DOOR_TOGGLE` (`:104`), `struct DoorToggle` (`:1402`),
  the `C2s`/`S2c` variants (`:1660`/`:1760`), the id/write/read arms
  (`:1885`/`:1953`/`:2053`, `:2239`/`:2363`/`:2521`), the 4 round-trip
  rows (`:4099-4100`/`:4400-4401`). **KEEP-ONE with g48**: its `net.rs`
  carries the byte-identical id (`:179`), struct (`:2008`), variants
  and codec arms — verified field-for-field (`u8 action, i16 x, i16 y,
  u8 direction`; `NetMessage.cs:537-542`). The merger keeps g48's copy
  (its branch owns the apply wave) and drops f11's; the round-trip
  rows union (identical payloads, dedupe by value).
- `netglue.rs::send_door_toggle` (f11 `:2975-2998`): host broadcast /
  client send / `Off` = the `SendData` netMode-0 return
  (`NetMessage.cs:83-86`). g48 has NO send-half (grep: its sends are
  test-only) — this fn is f11-UNIQUE, KEEP.
- The wiring-tick drain (f11 `world_sim/wiring.rs:439-446`):
  `for t in app.wiring.take_door_toggles() { app.send_door_toggle(t); }`
  — c42 slot: after its `take_door_cues` drain (`:375-382`), same tick.
- The netglue no-op arms `C2s::DoorToggle(_)`/`S2c::DoorToggle(_)`
  (f11 `:829`/`:1879`) DROP at the g48 union — g48's dispatch
  (`netglue.rs:923-924`/`:2124-2125` → `net_apply.rs::host_door_toggle`/
  `client_door_toggle`) replaces them. The apply/relay bodies are
  g48's, NOT re-ported here.
- Canary: the `pub fn take_door_toggles` anchors row lands with the
  core hunk.

### 7.3 forced-half (`9ae0edc97`, 95 §4/§8.5) — SUPERSEDED shape, witness KEEP

The c42 exec item 6 (`6381b52d`) already unified the family as FREE FNS
with `forced: Option<bool>` + `skips: Option<&mut Vec>` (the campfire
also takes `drops`), the methods thin wrappers passing `None`. f11's
method-side gates (`wiring.rs:3023-3380`, the 9 `forced.is_some() &&
forced != Some(off)` early returns) are the SAME truth table (De
Morgan twins of the free fns' `forced.is_none() || forced == Some(off)`
wrap) — **no code hunk folds**; shipping both would be the ×3 dup 95
bans. The campfire validate-kill-BEFORE-forced-gate order is pinned on
both sides (c42 free fn `:3691-3698`; f11 method `:3349-3363`).
**KEEP**: `w4_f11_the_unified_light_family_honours_the_forced_state`
(f11 `:6893`) — re-pointed at the c42 free fns (the campfire leg adds
the `&mut drops` arg; the holiday/candle/chandelier legs map 1:1). It
pins what the d6 four do not: the forced-REFUSAL legs (`Some(off)`
returns without a flip), the null-flip leg, and the anchor-keyed gate
reading the ANCHOR not the clicked cell. The e20 skip-set witnesses +
d6's four stay (95 §10 item 6's ledger).

### 7.4 radial-logic (`809d9a2cf`) — whole-module KEEP

Absent on c42 (no `wires_radial` anywhere). The LOGIC half is
self-contained and lands verbatim:
- `ui/wires_radial.rs` (457 lines: the Settings gates, the
  flower/line state machines, the six-slot geometry + XOR toggles,
  5 witnesses `:359-457`) + the `ui/mod.rs` module row.
- `App.wires_radial` field (`app/mod.rs:885-887`) + BOTH ctor inits
  (`:4008`, `flow.rs:2421` — the same two-literal census the fold-133
  receipt verified for `wire_tool_mode`).
- `App::handle_wires_radial` (`app/clicks.rs:820-866`) + the
  click-cluster call (`app/frame/world.rs:44`, after the inventory
  handlers — the DrawInterface_9 order).
- Depends on `App.wire_tool_mode` — ALREADY on c42 (fold-133 `05ae5cc6`).
- The ~450-line DRAW passes stay the texture-wave contract
  (research/234 §2); nothing draw-side folds now.

### 7.5 kite-param (`84ed6342b`) — the stream param + the drain contract

- `mass_wire_operation` gains `drop_rng: crate::app::RngKey` (f11
  `wiring_devices.rs:418-423`, the `spawn_item_drop_px` stream at
  `:481`). c42 target: its MOVED copy `world_sim/wiring.rs:1042` (the
  fold-133 module move) — add the param, re-point the hard-coded
  `RngKey::UpdatePlayers` at `:1096` to `drop_rng`.
- The 3625 use arm passes `RngKey::UpdatePlayers` (f11
  `editing/wiring.rs:207-213`) — c42 target `:207` (behaviour
  unchanged, the same stream as before).
- The kite drain is the d13-union hunk (research/235 §2, published at
  f11 `projectiles.rs:423-442`; c42's pass tail is `:418-421` after
  `take_spear_hold`) — executes at the d13×c42 merge, NOT on c42 alone
  (the pool seam is d13's). One drain shape, never both.
- Witnesses: `w4_f11_the_kite_apply_keeps_the_actuator_bit_and_the_proj_stream`
  (f11 `wiring_devices.rs:582`) re-points at the moved fn; the two
  existing mass-wire unit pins gain the `UpdatePlayers` arg (the c42
  receipt's `:1941`/`:1964` sites).

### 7.6 Fold order + verification

1. §7.1 core+app (the gate is independent of everything else);
2. §7.2 core payloads + the tick drain (the codec dedupe waits for
   g48's union — f11's `net.rs` copy is the fallback if g48 slips);
3. §7.5 param (one-line signature + two call sites);
4. §7.4 module (additive, no conflicts expected);
5. §7.3 witness re-point only.
The §5 cross-note still binds: none of these hunks touches the
144/421/422/235/226/506 fix-round arms. Every witness named is WRITTEN
NOT RUN (code-only lanes); the merger runs the catalog command after
the union per AGENTS.md.