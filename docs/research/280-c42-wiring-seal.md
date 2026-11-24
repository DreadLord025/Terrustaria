# 280 — W4-c42 WIRING SEAL: the research/146 re-pick verdict + the g78/g48 compose receipts + the wiring-family bible

> **RENUMBERED 240 → 280 (REGISTRY-RENAME-EXEC, research/150 §6.3 at
> `72226e500`, origin/task/W4-d54).** The original pick 240 collided with
> W4-g58's `240-happiness-final.md` (12:16:37, executed push `cdfe3d108`)
> and W4-d53's 240 (→275); c42's 240 was pushed last (14:17:05,
> `2910a7209`) → the registry assigns **280**. The number-240 provenance
> notes below are kept as the historical pick record; every live
> reference to this doc is now research/280.

> **STATUS: SEAL — 2026-11-21, branch `task/W4-c42-wiring-device-arms`
> (the seal phase, on top of the final tip `15810df4`).** Consumes the
> e20 retire escalation research/146 (`46fdcfd8` origin tip, publication
> `e47e1399f`), the g78 compose request (research/77 §3, `82bc71ea`),
> and the g48 NET-APPLY-1 seal (`9460c85d9`). Code-only lane: no
> cargo/tests/rustfmt/graphify run; all proofs are git-object reads and
> greps quoted below. Number 240 (HISTORICAL PICK — renumbered to 280,
> see the banner): the taken-set scan across ALL remote
> tips ends at 239 (`239-d42-final-keep-one-and-band.md`; 237/238 are
> contested on other lanes — 240 was the first free number at pick time,
> the research/150 registry's sequential-block convention; the round-2
> re-scan found the g58/d53 racing 240s and assigned 280).

## 1. research/146 RE-PICK — verdict: compile blocker FIXED HERE, census remainder re-picked merger-side

### 1.1 The defect was STILL present at the pre-seal tip (re-verification)

`git grep -n "slots: &mut \[crate::item::ItemSlot\]\|slots.resize" 15810df4 -- crates/`:

```text
crates/tr-app/src/world_sim/wiring.rs:674:    slots: &mut [crate::item::ItemSlot],   ← try_move_coins_in_chest
crates/tr-app/src/world_sim/wiring.rs:721:            slots.resize(t + 1, crate::item::EMPTY_SLOT);
crates/tr-app/src/world_sim/wiring.rs:773:    slots: &mut [crate::item::ItemSlot],   ← try_put_item_in_chest
crates/tr-app/src/world_sim/wiring.rs:795:                slots.resize(idx + 1, crate::item::EMPTY_SLOT);
```

plus the routing proof: `chest.rs:228-232` `chest_slots_mut` returns
`Option<&mut [ItemSlot]>` via `.map(|c| c.slots.as_mut_slice())` — the
slice erasure baked into the accessor, exactly research/146 §2. No
resize extension trait exists (`fn resize` hits only the wgpu surface
methods in tr-render — unrelated). **E0599 latent, twice — confirmed.**

### 1.2 The port (this branch, the seal commit): e20 `a2d6cdad0` verbatim

The census ruled the fix SMALL (2 signatures + 1 routing site, zero
behaviour change), so the seal ported it instead of leaving the merge
gate blocked:

1. `try_move_coins_in_chest` — `slots: &mut Vec<crate::item::ItemSlot>`;
2. `try_put_item_in_chest` — `slots: &mut Vec<crate::item::ItemSlot>`;
3. `hopper_pull`'s grab loop — `let slots = &mut app.world_chests[idx].slots;`
   directly (`find_chest` proved the index live), replacing the
   `chest_slots_mut(...).expect(...)` slice route.

The two `resize` bodies are UNCHANGED — they are the C# `Chest.Resize`
growth (`Chest.cs:118-130`) and now legal on the Vec. The read-only sort
path and the extractinator slot-consume path KEEP `chest_slots_mut`
(no growth there; e20 keeps them too — verified at `46fdcfd8`'s
`wiring.rs`). `WorldChest.slots` is `Vec<ItemSlot>` (`chest.rs:80`) —
the direct route needs no accessor change.

**Witness** (WRITTEN NOT RUN, code-only):
`w4_c42_the_hopper_slot_arms_grow_the_chest_vec`
(`world_sim/wiring.rs`, the tests module) pins BOTH growth sites
directly: the empty-slot fill past `slots.len()` (a full dirt stack +
a fresh drop → `resize(2)` + write) and the coin write-back into an
EMPTY Vec (100 copper → 1 silver at target 0 → `resize(1)`). The
whole-app path rides the existing
`dw_e20_the_hopper_pulls_a_ground_item_into_the_chest` (its chest
starts `vec![]` — the growth runs through `hopper_pull`'s new route).
Dedicated-module witness total: **282 → 283**.

### 1.3 The census remainder — the FINAL merger-side re-pick instruction

What the port does NOT import (re-grepped at the seal tip — all still
absent): `TrackHammer`/`frame_track_hammer`/`frame_track_place`/
`refit_tracks_around` (zero hits), `track_item`/`is_pressure_plate`
(zero non-test hits), the `edit/hammer.rs:413` 314 arm (still
`PoundOutcome::Consumed`), the `cell_drops` 314 arm
(`world_sim/editing.rs:6207` — no 314 case). **The merger takes these
from e20, per research/146 §1's exact 25-commit `6d1754719` chain**
(base `a725acb04`, state `02ac64ab6`, docs tail `46fdcfd8`); the
census-bearing commits:

| Commit | Half |
| --- | --- |
| `0cea13365` | §3b.4 hammer-placement: the FULL `frame_track(pound, mute)` + `PoundOutcome::TrackHammer` + the Item52 cue/wake drain |
| `865e9d83f` | track-piece census: `edit::place_track`, `minecart::track_item`, the `cell_drops` 314 arm, the placement wake |
| `29d9dc224` | mechanical: the `cell_drops` 314 arm yields `Vec<(u16,i32)>` |
| `f74384e97` | hopper INPUT-side census (grab box, no chest-above arm; IsHopperInRangeOf MP-only) |
| `6d1754719` | census close: `minecart::is_pressure_plate` (`Minecart.cs:549-560`) |
| `e38aef2f3` | placement-wake witness fix (outer supports) |

**Do NOT re-pick** (already on this tree, re-verified at the seal tip):
the hopper/extractinator bodies (the Vec signatures now per §1.2; the
strict-`<` grab box, the `(placed, remaining)` remainder, the
PlaySound(7) wiring-stream cue, the anchor-centre extractinator drop —
the `extractinator_roll` body byte-matches e20's), `flip_switch_track`
(byte-equal to e20's, 27 lines), the OnceLock `track_switch_options`
cache (the `f167082ed`/`6336e3b44` merge-in content), the 36-row
`TRACK_FRAMES` superset, the `cb14ba4fb` trap-fix round (this tree's
`TrapRequest::Spear` carries `cap_x/cap_y`; the wall-trap `vy: 0.0`
arms present). The stale lineage `8791f07fe`/`f1c24c381`/`e8e6b159c`
never re-lands.

**Union overlap resolutions:**

- e20's FULL `frame_track(world, i, j, pound, mute) -> (bool, bool)`
  SUPERSEDES this tree's wiring-only `frame_track_pound` slice
  (`minecart.rs:1385`): the slice's pound path is behaviour-identical
  (the non-pound branch and the `!mute` fail-kill sound were documented
  unreachable and not ported); e20's wrapper
  `frame_track_pound = frame_track(…, true, true).0` keeps the wiring
  call sites unchanged. Take e20's shape, keep this tree's call sites.
- `IsPressurePlate` keep-one: research/148 §3c (e20's app-side fn vs
  d106's inline tr-core predicate in `is_it_a_trigger` — ONE truth at
  the merge). d106's `BrokenTrapCleanup` pass is ALREADY LANDED
  (`e2c2609c5` R5) — do NOT re-port (research/148).
- f11's `tr-core/src/minecart.rs` duplicate never lands (the 133
  receipt §4b ruling stands).
- This branch's research-71 copy is the stale v1 fork — now carrying a
  SEAL BANNER pointing here; the registry (research/150 §3) assigns it
  →212 at the merge. The FINAL research-71 is e20's (`46fdcfd8`).

**Re-pick status: the compile blocker is CLOSED on this branch; the
census remainder is a merger-side take from the `6d1754719` chain —
this branch does NOT carry it and does NOT claim it.**

## 2. g78 compose receipt (research/77 §3, `82bc71ea`)

- **§3.1 the 657 use arm — c42's landed arm is the KEEP-ONE.** g78's
  branch carries the hunks as DOCS ONLY (zero `MonolithSwitch` code
  hits in `crates/` at `82bc71ea`). This tree landed the arm at
  `f173fb88`: `TileInteraction::MonolithToggle` (`tr-core/edit.rs:211`,
  the nine-kind literal gate at `:305` BEFORE the registry lookup) +
  the `doors.rs:248` arm (Mech-28 on the UpdatePlayers stream +
  `gamepad_enable_grapple_cooldown` + `use_switch_monolith`) +
  `Wiring::use_switch_monolith` (`tr-core/wiring.rs:3251`, over the
  kept e21 nine-kind `switch_monolith` body, `running == false` — the
  SkipWire tail inert) + `Player.quick_grapple_cooldown`
  (field/setter/Update decrement + the map-open re-arm,
  `Player.cs:24439-24448`). Naming delta vs the request
  (`MonolithToggle`/`use_switch_monolith` vs
  `MonolithSwitch`/`switch_monolith_use`): SAME shape; c42's arm ports
  `:32275` for real where g78's hunk recorded a no-gamepad deviation —
  c42's is the richer port. **The merger does NOT apply g78's §3.1
  code hunks on top** (they would duplicate the arm under a second
  name). Witness:
  `c42_the_monolith_right_click_flips_the_body_and_arms_the_grapple_cooldown`
  (`editing/doors.rs`) — the flip ±54, the 3-tick cooldown, the
  second-use flip-back, the inert skip set.
- **§3.2 the Check2xX drop-5345 break check — stays c31/g17/g47/f41.**
  The break-lane contract per research/77's own co-lane list
  (`edit/kill.rs::check_object` is contested by the break lanes, not
  this one). The contract detail (the `num4 = 3` height, the 410
  lit-row +2 quirk, the per-kind drop table, the ORIGIN-cell 32×32
  spawn at `(num*16, j*16)`) is recorded in research/77 §3.2 for those
  lanes; this branch ports nothing there.
- §1 (spelunker dispatch), §2 (Hunter floor), §4 (IsTileSpelunkable
  keep-one: d9 canonical, g78 re-export, d13 delete, d56
  crate-boundary exception) are g78's own lanes — no c42 hunks.

## 3. g48 keep-one receipt (NET-APPLY-1, tip `9460c85d9`)

- **The `19` apply arm consumes this tree's canonical bodies.**
  `net_apply.rs`'s `apply_door_action` calls
  `edit::shift_trapdoor`/`edit::shift_tall_gate` with the FULL post-union
  shapes — the trailing `None` (wire_skips) + the `cues` vec ALREADY
  passed (`:698`/`:734` at the g48 tip; signatures matched against this
  tree's `edit/shift.rs:86`/`:278`). Compatible — no change.
- **The ONLY signature gap: `open_door`/`close_door`.** g48's
  `net_apply.rs:635` calls `edit::open_door(&mut app.world, x, y, dir,
  &mut drops)` (5 args) and `:650` `edit::close_door(&mut app.world, x,
  y, true, &|_, _| false, &mut drops)` (6 args) — the PRE-union shapes.
  This tree's canonical fns take the trailing
  `wire_skips: Option<&mut Vec<(i32, i32)>>` (`edit.rs:732`/`:868`, the
  f11 §1 Option seam adopted by the union-exec). **Merge note (the
  merger executes at the g48 union): append `None` at both call sites**
  — the net reader runs outside `Wiring.running`, so `None` is the
  faithful value (no skip collection). Nothing else changes.
- **Keep-one feed (g48's own receipts):** the 34 destroy-arm refinement
  takes g52's `e2a5968a4` (`can_kill_tile` + the chest-store gate +
  store removal — absent from g48's `net_apply.rs:920-957`); the
  34/162 codecs: g48==g52 byte-verbatim 34, g51's pending-damage 162
  (g48 `d3200ef3a`). The 19-echo SEND half is this tree's
  `door_echoes` queue (95 §8.2); the MP relay attaches with the g48b
  codec lane. The f11-tail-fix (`34029437a`, research/237: the
  `kill_tile(no_item:true)` pound-fail tail) targets f11's minecart
  copy — this tree's wiring-only pound slice never had that tail; the
  fix rides e20's FULL `frame_track` shape at the §1.3 re-pick (e20's
  `frame_track` fail-kill returns the bool; the app runs the
  `KillTile(fail: true)` slice — never a removal).

## 4. The WIRING-FAMILY bible (one paragraph)

**c42-canonical / f11-unique-folds / e20-devices / g48-net-apply:**
`task/W4-c42-wiring-device-arms` is the wiring canonical — the
doors/trapdoors/tall-gates union, the light unification, the `de176ccdd`
fix round, the fold-133 lifecycle/mass-wire/3625 uniques (f11's entire
unique estate, EXECUTED per the 133 receipt), the §8 seam closures
(the stamped entity gate, the SendData(19) echo queue, the drain union,
`is_hopper_in_range_of`), the monolith right-click use arm (the g78
request), and the compile-fixed hopper/extractinator/flip-switch-track
device arms (the e20-unioned bodies + this seal's `a2d6cdad0` Vec port)
— **283 dedicated-module witnesses** (core wiring 143, app world_sim
wiring 34, app minecart 43, core edit 34, doors 14, editing/wiring 14,
tile_entities 1; plus the cross-module pins), ALL WRITTEN NOT RUN;
**e20** contributes ONLY the census remainder (the `6d1754719` chain's
hammer/placement/track-item/`is_pressure_plate` halves, §1.3) and the
one-truth seam resolutions (research/148 §3c); **f11** and **e21**
contribute nothing further (folded/unioned; their deprecated halves
never re-land); **g48** owns NET-APPLY and consumes this tree's edit
bodies verbatim — the merger appends the trailing `None` at its
`open_door`/`close_door` pair (§3) and takes g52's 34-destroy
refinement. **Gate-command** (the merger runs it; this lane never
did): `cargo test --workspace -- --skip
migrates_version_2_kinds_to_item_ids` — judge by the `test result: ok`
lines (cargo may exit 1 on success); canary `anchors.txt` (the seal
added the `w4_c42_the_hopper_slot_arms_grow_the_chest_vec` row) via
`tr-core/build.rs` + `cargo test -p tr-app --test main_anchors`.

## 5. Verification record (this seal, code-only)

- Defect re-verification: `git grep` at `15810df4` (the four sites +
  the `as_mut_slice` accessor) — quoted §1.1.
- The port: `git diff` shows ONLY the 2 signatures + the routing +
  the witness in `world_sim/wiring.rs`; no other file's code changed.
- Caller census: `try_put_item_in_chest`/`try_move_coins_in_chest` have
  exactly the internal callers (`wiring.rs:781`/`:873` + the witness) —
  no external slice-typed caller exists.
- Body-equality proofs: `flip_switch_track` 27-line byte-equal;
  `extractinator_roll`/`find_chest_for_extractinator` byte-match e20's
  at `46fdcfd8`; the OnceLock cache + `TRACK_FRAMES` 36-row table
  present on both trees.
- Absence proofs (the census remainder): zero `TrackHammer`/
  `frame_track_hammer`/`track_item`/`is_pressure_plate` non-test hits;
  `hammer.rs:413` still `Consumed`; `cell_drops` no 314 arm.
- g78/g48 receipts: git-object reads at `82bc71ea`/`9460c85d9` (the
  docs-only §3.1 proof, the two call-site signatures, the shift-body
  compatibility).
- Number 240 (HISTORICAL PICK — renumbered to **280** per research/150
  §6.3, `72226e500`): the full remote-tip scan (`git ls-tree docs/research/`
  per tip) — max taken 239, 240 free at pick time; g58's executed 240
  (`cdfe3d108`) wins the collision, c42 → 280.
- No cargo/tests/rustfmt/graphify run (lane rule). Every witness
  WRITTEN NOT RUN.