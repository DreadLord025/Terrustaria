# 95 — Wiring MASTER reconcile (c42 × f11 × e20 × e21 × g4 × d6)

> Merger playbook · 2026-11-21 · code-only (no cargo/tests/graphify run) ·
> sources: `decompiled/Terraria/Wiring.cs`, `WorldGen.cs`, `Minecart.cs`,
> `NPC.cs` (1.4.5.8) + the branch tips below. Supersedes nothing; **adopts and
> extends research/71** (the e20×f11 device-arm reconcile, `8791f07f` on
> `task/W4-e20-wiring-device-arms-2` — pushed; no separate "e20b" branch exists).
>
> **EXECUTION STATUS (union-exec on `task/W4-c42-wiring-device-arms`, this
> branch is now the wiring canonical).** All six items below executed
> code-only on top of the reconcile tip `ec568b3e`; per-item hashes + what
> landed in §10. No `e20c` branch exists on origin (checked at exec time) —
> the PlaySound(7)/OnceLock wins came from f11 and were merged INTO the kept
> e20 bodies per research/71 §3b, exactly as item 4's fallback allows. No
> cargo/tests/graphify run; every witness WRITTEN NOT RUN.

**Tips compared** (all branched from `a725acb0`, wave-3 land):

| Lane | Tip | Branch | Wiring scope |
| --- | --- | --- | --- |
| c42 | `12b45b21` | `task/W4-c42-wiring-device-arms` | fix round `de176ccd` (144/421/422 wakes + actuator guard, 235/226 `downedPlantBoss`, 506 shared validator) + 8 missing arms (215/4/130/131/11/10/386+387/388+389) + NEW `tr-core/src/edit/shift.rs` + `door_cues` queue drained at EVERY switch-click site |
| f11 | `cf0737da` | `task/W4-f11-wiring-arms-4` | the SAME 8 arms (torch/campfire/slabs/doors/trapdoors/tall-gates) + 314/hoppers/extractinators + NEW `tr-core/src/minecart.rs` + NEW `tr-app/.../wiring_devices.rs` + `door_sounds` queue drained in `update_wiring` only |
| e20 | `2322af90` (+docs `8791f07f`) | `task/W4-e20-wiring-device-arms-2` | 314/21+467/219+642 request seams + app `minecart.rs` extension + hopper/extractinator app bodies + trap fidelity fixes (wall-trap vy=0, F3 cap anchor+Center) + light-family/429/b=0 witnesses |
| e21 | `12fb1a2e` | `task/W4-e21-wiring-device-arms-3` | pump `xfer_water` active-gate drop, monolith nine-kind gate, grate 546/557 wake drops, boulder-531 app drain → DeviceShot |
| g4 | `53063dd1` | `task/W4-g4-npc-style3-fighter` | P4 only: `edit::shift_tall_gate` (third copy) + the zombie door-bash gate-388 drain arm (NPC.cs 60072-60084) with a REAL entity-occupancy closure |
| d6 | `5ae2978c` (wiring at `5d81f2de`) | `task/W4-d6-proj-ai-51-60` | the `TryToggleLight` free-fn family (the 11 statics: 10 `Wiring.Toggle*` helpers + the `WorldGen.TryToggleLight` dispatch, WorldGen.cs 72970-73024) for the aiStyle-60 gun streams |

g31b: **not in scope** (no wiring content; per the task order).

**Base inventory** (what `a725acb0` already has — do NOT re-port): the
`Wiring`-method light family `toggle_holiday_light`/`toggle_hanging_lantern`/
`toggle_2x2_light`/`toggle_lamp_post`/`toggle_candle`/`toggle_lamp`/
`toggle_chandelier`/`toggle_fire_place` (base wiring.rs :2609-2775, SkipWire
built in, **no forced-state parameter**), the 244/565 frame-shift arms, the
531 `boulder_statue` core arm (:1344, `StatueSpawn::Boulder` queued; the app
drain DISCARDS it at :235), `edit::open_door`/`close_door` (4/5-arg, no skip
collection), `edit::can_kill_tile` (kill.rs:2783, full body) and the
`Wiring::can_kill_tile` slice (wiring.rs:3025, private), `validate_tile_square`
(private), `running` flag (:277, set/cleared around the flood :729/:741).

---

## 1. Doors 10/11 — c42 vs f11 (two-way dup)

C# (`Wiring.cs:1461-1488`): 11 → `CloseDoor(forced:true)`, `SendData(19,1)` on
success; 10 → ONE `Main.rand.Next(2)` (0→−1 else +1), `OpenDoor(num67)`, on
refusal the **unconditional** opposite retry (no kind-10 re-check — the player
path's re-check is its own shape), `SendData(19,0,…,dir)`.

| Item | c42 | f11 | KEEP |
| --- | --- | --- | --- |
| Edit-layer seam | NEW wrappers `open_door_wired`/`close_door_wired` + private `*_inner(collect_skips: bool, skips: &mut Vec<(i32,i32)>)`; the 4/5-arg `open_door`/`close_door` signatures UNCHANGED (zero caller churn) | `wire_skips: Option<&mut Vec<(i32,i32)>>` appended to `open_door`/`close_door` themselves; ALL callers updated (`interact_at` ×3, NPC bash drain, ~12 test sites) | **either body — identical skip cells** (open ×6 `WorldGen.cs:37717-37725`, close ×3 `:31895-31899`). Recommend **f11's Option shape**: it encodes the C# `Main.netMode != 1 && Wiring.running` gate at the call site (`if self.running { Some(...) } else { None }`); c42's wrappers collect unconditionally and rely on the arm being the only wired caller. If f11's shape wins, DROP c42's wrappers/inner split; if c42's wins, f11's caller updates revert. Do NOT ship both. |
| Arm 11 | `close_door_wired(forced:true, &|_,_| false, …)`; on success drops+skips+`(false, i*16, j*16)` cue | same via Option; cue `(false, i*16, j*16)` | tie — C# `PlaySound(9, i*16, j*16)` is at `WorldGen.cs:31908` (verified), hit-cell coords, success-only. Both correct. |
| Arm 10 | `rand_next(2)` roll, `open_door_wired` + unconditional `-num67` retry; skips+cues only on success | identical roll/retry/order | tie — the roll joins the wiring stand-in stream (S1-L-6c) in both. Keep ONE arm. |
| Cue queue | `door_cues: Vec<(bool,i32,i32)>` + `take_door_cues` | `door_sounds: Vec<(bool,i32,i32)>` + `take_door_sounds` | **one queue** (identical payload/semantics; name either). **c42's drain set is the superset**: netglue ×3, chest.rs, editing/doors.rs, npcs.rs, player.rs ×3, projectiles.rs ×2, world_sim/wiring.rs — each on ITS stage's stream (`play_door_sound(open, px, py, stage_rng)`). f11 drains ONLY in `update_wiring` (UpdateWorldGen) — cues queued by trips started from other stages would play on the wrong stream or linger. KEEP c42's drain-site union. |
| `SendData(19)` | not ported (Stage-2 note) | not ported | open seam §8.2 |

**Witness union (doors):** c42 `c42_the_open_door_closes_forced_and_skips_the_hinge`,
`c42_the_closed_door_opens_with_one_roll_and_skips_the_block`,
`c42_the_door_open_retries_the_opposite_direction`,
`c42_the_door_roll_joins_the_wiring_stream` × f11
`w4_f11_the_closed_door_opens_on_a_wire_hit`,
`w4_f11_the_open_door_closes_on_a_wire_hit`,
`w4_f11_the_door_direction_draw_is_deterministic`. Overlap: the open/close
pair (keep one of each — the c42 versions additionally pin the SkipWire sets);
f11-unique: the deterministic-draw pin (keep); c42-unique: the retry + the
stream-join + the hinge-skip pins (keep).

## 2. Trapdoors 386/387 — c42 `edit/shift.rs` vs f11 `edit.rs` (two-way dup)

C# `WorldGen.ShiftTrapdoor` (`WorldGen.cs:51447-51596`, re-read verbatim):
386-open has TWO styleX variants (0: gate/clear bottom row, write y+1, thud-less;
1: gate/clear top row unconditionally, write y); 387-close has the CanKillTile
guard on both open cells, the `playerAbove.ToDirectionInt()` destination row,
the cut/drip `KillTile` sweep, the per-column `BlockColorAndCoating` cache, and
the fallback `PlaySound(9, x*16, y*16)` at the ADJUSTED anchor on every refused
call (the wiring arm calls twice → two thuds on a full refusal).

| Item | c42 (`edit/shift.rs:214`) | f11 (`edit.rs:1041` region) | KEEP |
| --- | --- | --- | --- |
| Signature | `(world, x, y, player_above, cell_blocked, can_kill: &dyn Fn(&World,i32,i32)->bool, drops, skips: &mut Vec, cues: &mut Vec)` — no `only_close_or_open` (hardcodes the C# default −1) | `(world, x, y, player_above, only_close_or_open: i32, cell_blocked, drops, wire_skips: Option<&mut Vec>)` — no cues, no can_kill injection | **c42's body + f11's `only_close_or_open` param** (the C# default-arg is real API surface; a future hammer/interact caller needs it). Skip seam: whichever §1 shape wins. |
| Close-guard kill check | injected `can_kill` — the wiring arm passes `Wiring::can_kill_tile` (the flood's anchored-tile slice, i34 keep-one; `pub(crate)` bump) | direct `edit::can_kill_tile(world, x, y, \|_,_\| false)` (the FULL WorldGen body) | tie for 387 cells (the chest-inventory cases test kinds 21/467/88 — never 387). f11's is the more complete fn; c42's keeps the flood slice single-sourced. Either; if c42's body survives, keep the injection. |
| Paint on close | `block_color_cache` = paint **+ fullbright coating** (`BlockColorAndCoating`/`UseBlockColors`, both fields) | paint ONLY — **drops the echo-coating bit** | **c42** (f11 loses `UseBlockColors`' coating half, `WorldGen.cs:51574/:51581`). |
| Sound cues | in-body, per C# site: `(true, x*16+16, y*16+16)` styleX0 (:51470), `(true, x*16+16, y*16)` styleX1 (:51512) and close (:51563), `(false, x*16, y*16)` fallback (:51594) at the adjusted anchor — a full wiring refusal queues TWO thuds, exactly the C# | NONE in-body; the arm (`wire_trapdoor`) folds to ONE cue per outcome at the HIT cell: success `(true, i*16+16, j*16+16)` regardless of variant/anchor, refusal `(false, i*16, j*16)` once | **c42** — f11's fold is a documented deviation but wrong on position (anchor vs hit cell), on the styleX1/close Y, and on the double-thud count. |
| `touch()` bookkeeping | called per write | absent | **c42** (the port's dirty-cell convention). |
| Open-pair write | sets kind/frames only (C# :51482-51484 — the cells are already active from the validate) | additionally `\|= FLAG_ACTIVE` | tie (no observable difference post-validate). |
| Reframe | radius-2 supersets at the written block (open) / `(x, y+1−2·(!above))` (close) | radius-2 at `(x+1, write_row)` / radius-3 at `(x+1, y−above_offset)` | tie — both documented supersets of the C# walks (:51486-51492/:51524-51530/:51585-51591). |
| styleX fall-through | `if style_x==0 {…} if style_x==1 {…}` then 387/fallback | `if style_x==0 \|\| style_x==1 {…}` then 387/fallback | tie (identical outcomes incl. the styleX≥2 fallback). |
| Anchor helper | `get_top_left_and_styles` (private, shift.rs) | `top_left_and_styles` (`pub(crate)`, edit.rs; `.max(1)` divisor guard) | **one copy** — f11's pub(crate) placement is the reusable one; c42's is verbatim (C# divides by zero on inc=0 too — no current caller passes 0). Either; never both. |

**Witness union (trapdoors):** c42 `c42_the_trapdoor_opens_downward_and_skips_the_body`,
`c42_the_open_trapdoor_closes_upward_with_paint`,
`c42_the_trapdoor_close_sweeps_cut_tiles` (the `12b45b21` pin: the closed body
writes rows y..y+1 OVER the swept cell — the sweep's observable is the close
proceeding past a cut tile) × f11 `w4_f11_the_trapdoor_shifts_open`. Keep all
four (f11's overlaps c42's first — keep one of that pair; the paint/sweep pins
are c42-unique and MUST survive: f11's body has no coating and its cue fold is
unpinned). If f11's signature wins, re-point the c42 witnesses' call args only.

## 3. Tall gates 388/389 — THREE-way dup (c42 vs f11 vs g4)

C# `WorldGen.ShiftTallGate` (`WorldGen.cs:51723-51775`, re-read verbatim):
`GetTileData(388, frameX/18)` — the CLOSED kind's registry entry, **2-arg →
alternate 0**; the row walk subtracts only `CoordinateHeights[i]` (no padding);
validate → SkipWire column (running-gated) → per-row `EmptyTile` gate
(`forced` skips) → `PlaySound(8, x*16+16, y*16+16)` at the ADJUSTED top →
kind swap → `TileFrame` 3×(height+2) walk (x−1..x+1, y−1..y+height).

| Item | c42 (`edit/shift.rs:382`) | f11 (`edit.rs`) | g4 (`edit.rs`, `53063dd1`) | KEEP |
| --- | --- | --- | --- | --- |
| Signature | `(world,x,y,closing,forced,cell_blocked,drops,skips:&mut Vec,cues:&mut Vec)` | `(…,drops,wire_skips:Option<&mut Vec>)` | `(world,x,y,closing,forced,cell_blocked,drops)` — 6-arg, no skips/cues | **c42's body**; the NPC drain (g4) re-points at it passing a local skips vec (discarded — no flood running on the NPC stage, the C# gate is false there) and drains the cues vec on the UpdateNpcs stream instead of its own `play_door_sound` call. |
| Def decode | `get_tile_data(def_for_tile(388), style, 0)` — alternate **0**, matches the C# 2-arg call; `None → false` | `def_for_tile(388).expect(…)` — **no style/alternate lookup at all** (uses the base def; comment claims style only shifts the sheet); panics if unregistered | `get_tile_data(def, style, 1)` — alternate **1** (C# default is 0); `full_h <= 0 → false` guard (C# would div-by-0) | **c42** (alternate 0 + graceful refusal). g4's alternate-1 and f11's no-lookup are both deviations; harmless while the registry has one alternate, wrong the moment a style/alternate diverges. |
| Cue | in-body `(true, x*16+16, y*16+16)` at the adjusted top (:51762) | arm-level `(true, i*16+16, j*16+16)` at the HIT cell — wrong Y for non-top hits | drain plays `play_door_sound(true, tx*16, ty*16, UpdateNpcs)` — wrong position (no +16, hit cell) | **c42** |
| SkipWire | unconditional push into `skips` (arm inserts into `self.skip`) | Option-gated at the arm (`if self.running`) | none ("no wiring in the port" — correct for the NPC caller, the C# gate is `Wiring.running`) | c42's collection + the §1 gate convention |
| `touch()` | yes | no | yes | c42/g4 |
| Reframe | radius **4** at `(x, y+height/2)` — superset of the 7-row walk | radius **3** at `(x, y+2)` — exact row coverage for the 1×5 gate, superset in x | radius **2** at `(x, top+height/2)` — covers y..y+4 only: **MISSES rows y−1 and y+5** of the C# walk (:51767-51773) | **c42 or f11** (f11's radius 3 is the tight superset; c42's 4 is safe). **g4's radius 2 is a fidelity gap — must be lifted at merge.** |
| Entity gate | closure, `forced` skips; wiring arm passes always-free | same | same signature, but the DRAIN wires a REAL player/NPC box-overlap closure (the `close_door` convention) | c42's body; **g4's drain closure is the reference implementation for the open seam §8.1** |

**g4's non-dup half LANDS untouched**: the `wants_door_bash` gate-388 arm in
`update_npcs` (refusal `ai[3] = 60.0` + `net_update`, success `world_key=None`,
drops spawned on UpdateNpcs), the `npc_ai_fighter_scan.rs` doc retirement of
the S1-D-3z stand-in. Only its `edit::shift_tall_gate` copy is dropped and the
drain call re-pointed (add the two vec args; play the drained cues).

**Witness union (tall gates):** c42 `c42_the_tall_gate_opens_and_skips_the_column`,
`c42_the_tall_gate_validation_kills_a_broken_column`; f11
`w4_f11_the_tall_gate_swaps_its_column`; g4 `w4_g4_shift_tall_gate_opens_the_closed_gate`
(open/close round-trip + blocked refusal + forced bypass + non-gate refusal —
the richest unit pin) + `w4_g4_the_door_bash_drain_shifts_the_tall_gate` (the
only drain-level pin). Keep the c42 pair + both g4 witnesses (re-pointed at the
kept signature); f11's overlaps c42's first — drop or keep one.

## 4. Torch 4 / campfire 215 / slabs 130+131 — c42 vs f11 vs d6 (three-way)

C# call sites: `Wiring.cs:1418-1421` (Campfires set → `ToggleCampFire`),
`:1575-1579` (Torches set → `ToggleTorch`), `:1423-1439` (130/131). The wire
path passes `forcedStateWhereTrueIsOn = null`, `doSkipWires = true`. The SAME
`Toggle*` statics are ALSO called by `WorldGen.TryToggleLight`
(`WorldGen.cs:72970-73024`) with a forced state and `doSkipWires = false` —
that is d6's aiStyle-60 gun caller.

**The 11 statics** (d6 `5d81f2de`, free fns in tr-core/wiring.rs):
`try_toggle_light` (dispatch) + `toggle_torch` (:2903-2918, ±66),
`toggle_candle` (:2920-2933, ±18), `toggle_lamp` (:2935-2961, 1×3 ±18),
`toggle_chandelier` (:2963-2998, 3×3 ±54 mod-108),
`toggle_campfire` (:3000-3042, 3×2 frameY ±36 + validate-kill),
`toggle_fireplace` (:3044-3072, 3×2 frameX ±54),
`toggle_2x2_light` (:2843-2877, ±36), `toggle_lamp_post` (:2879-2901, 1×6 ±18),
`toggle_hanging_lantern` (:2817-2841, 1×2 ±18),
`toggle_holiday_light` (:2800-2815, ±54). All carry `forced: Option<bool>`
(the C# `!HasValue || !Value != off` gate, verified equivalent); **none does
SkipWire** (`skip_wires` param carried but ignored — documented skip);
campfire alone collects validate-kill `drops`.

| Item | c42 | f11 | d6 | KEEP |
| --- | --- | --- | --- | --- |
| Torch arm | `4 =>` literal, inline ±66 | `k if SET_TORCHES.has(k) =>` guard, identical ±66 body | `toggle_torch` free fn (forced gate; null ⇒ same flip) | **one arm** — f11's set-guard form is the C# shape (`TileID.Sets.Torches[type]`, set = {4}); body identical in all three. |
| Campfire | `toggle_camp_fire` method: set gate, %54/18 + %36/18 anchor, shared `validate_tile_square` (drops → `actuation_drops`), anchor-frameY ±36, SkipWire 3×2, per-cell active+kind write gate | `toggle_campfire` method — **line-for-line the same semantics** (adds the vacuous-forced comment) | `toggle_campfire` free fn: same anchor/validate/flip/write-gate, forced gate, drops out-param, **NO SkipWire** | **one body** serving both callers: the wiring method (c42/f11, either — identical) is the SkipWire-complete one; d6's forced gate must be ADDED to it (or the method delegates to a shared core with `forced` + `skips` out). Never three copies. |
| The other 8 Toggle* | untouched (base methods) | untouched | free-fn copies of the SAME statics the base `Wiring` methods already port (candle/lamp/chandelier/fireplace/2x2/lamp_post/lantern/holiday) | **unify at merge**: base methods (SkipWire-complete, no forced) × d6 free fns (forced-complete, no SkipWire) are two halves of each C# static. Recommended shape: one family with `forced: Option<bool>` + a skips out-collector; `Wiring` methods pass `None` + insert into `self.skip`; `try_toggle_light` passes the forced state + discards skips (its callers all pass `skip_wires=false` today). d6's `try_toggle_light` dispatch itself is UNIQUE (no other lane ports TryToggleLight) — KEEP, re-pointed at the unified family. |
| Slab 130 | above-cell `PreventsActuationUnder` gate (`None => false` = the C# null conjunct) + `self.can_kill_tile` slice, swap + `square_tile_frame_default` wake + drops | `is_some_and` (identical truth table), same slice/swap/wake/drops; comment pins the CanKillTile conjunct OUTSIDE the above-active disjunction | — | tie — keep ONE (f11's comment is the better doc). |
| Slab 131 | unconditional swap + wake + drops | identical | — | tie — keep ONE. |
| `validate_tile_square` visibility | `pub(crate)` (de176ccd) | `pub(crate)` | uses it (already visible on its branch? d6 calls `crate::edit::validate_tile_square` — needs the same bump) | the bump lands ONCE (identical edit, trivial conflict). |

**Witness union (light/slabs):** c42 `c42_the_campfire_flips_its_3x2_body_and_skips_it`,
`c42_the_campfire_validation_kills_a_broken_body`, `c42_the_torch_toggles_frame_x_by_66`,
`c42_the_active_stone_block_swaps_and_wakes`,
`c42_the_inactive_stone_block_swaps_back_unconditionally`; f11
`w4_f11_the_torch_toggles_its_frame_by_66`, `w4_f11_the_campfire_flips_the_whole_3x2_body`,
`w4_f11_a_broken_campfire_body_is_killed_not_flipped`, `w4_f11_the_stone_slab_swaps_to_its_pair`,
`w4_f11_a_chest_above_refuses_the_130_swap` (UNIQUE — the CanKillTile gate pin),
`w4_f11_an_actuated_torch_actuates_instead_of_toggling` (UNIQUE — the
ActuateForced-refusal interaction); d6 `w4_d6_the_torch_flip_is_66_and_the_forced_state_gates`
(UNIQUE — the forced gate + inactive/non-light refusal),
`w4_d6_the_lamp_flips_all_three_cells_from_the_mod3_anchor`,
`w4_d6_the_chandelier_flips_the_3x3_by_the_anchor_mod108`,
`w4_d6_the_campfire_flips_framey_and_a_broken_square_kills`. e20's light-family
witnesses (`the_hanging_lantern_flips_both_cells_and_skips_them` — the
`2322af90` corrected version, `the_lamp_flips_three_cells_and_skips_them`,
`the_2x2_light_family_flips_the_block_and_skips_it`,
`the_chandelier_flips_the_3x3_and_skips_it`,
`the_lamp_post_flips_six_cells_and_skips_them`, `the_holiday_light_flips_54`,
`the_pixel_box_frame_shift_flips_3x2_and_skips_it`,
`the_campfire_frame_shift_flips_2x2_and_skips_it`) pin the BASE methods'
SkipWire sets — they must survive the unification (they are the only skip-set
pins for the 8 non-campfire statics). Dedupe: the torch ±66 trio (c42/f11/d6)
→ keep d6's (superset: forced gate) + one wiring-path pin; the campfire
validate-kill trio → keep one wiring pin + d6's forced variant; everything
else is additive.

## 5. Hoppers 21/467 + extractinators 219/642 — e20 vs f11

**Delegated to research/71** (`8791f07f`, pushed on the e20 branch) — its
per-fn table is adopted WHOLESALE here. Summary of the verdict:

- **KEEP e20's bodies**: app `hopper_pull`/`extractinator_roll` +
  `try_put_item_in_chest` (tuple `(placed, remaining)`),
  `try_adding_to_stack`, `try_move_coins_in_chest`, `coins_count`/`coins_split`,
  `find_chest_for_extractinator`; the core request seams
  (`hopper_requests`/`extractinator_requests` carrying hit cell + kind, app
  resolves anchor + CheckMech(60)).
- **DROP f11's** `world_sim/wiring_devices.rs` + its core anchor+CheckMech arms,
  EXCEPT the must-merge wins: (1) `PlaySound(7)` + its wiring-stream pitch draw
  in the empty-slot fill (`Wiring.cs:2765` — e20 omits both, a determinism-
  visible gap); (2) the OnceLock 64-mask switch-options cache; (3) the f11
  witnesses re-pointed (coins_split 1234567 pin, grab-rect pin, locked-chest
  refusal, non-anchor-cell hits, cooldown-EXPIRY re-fires).
- e20-unique must-survive: wall-trap vy=0, F3 cap anchor+Center (+ the
  `TrapRequest::Spear` shape change — f11 constructors update to e20, not the
  reverse), partial-merge remainder witness, anchor-CENTRE extractinator drop
  (f11's tile-origin spawn is ~8px off), `WireState::any`.
- c42/e21/g4/d6 add NOTHING here (c42's claim doc explicitly defers 21/467/219/642
  to e20; verified — no hopper/extractinator code on any other tip).

## 6. Switch track 314 — e20 vs f11 (two-way; c42 deferred)

Also **research/71 §1-2** adopted: KEEP e20's `TRACK_FRAMES` (the M2-PHYS-7
superset table, already consumed by `track_collision`) + e20's
`flip_switch_track`/`frame_track_pound` (assert-on-front-absent ≈ the C#
IndexOutOfRange; f11's `.max(0)` clamp silently diverges) + the e20 core
`CheckMech(5)` → `switch_track_flips` request seam. DROP f11's
`tr-core/src/minecart.rs` (its four flat arrays re-transcribe a verified-identical
subset; its full `frame_track` non-pound branch + `flag4`/`KillTile(fail)` are
the future hammer-placement half — salvage as a follow-up note, NOT wiring
code). Merge in f11's OnceLock cache shape over the e20 table. c42's claim doc
deferred 314 to e21, but e21 did NOT port it (its three commits are pump/
monolith/grate/boulder only) — the actual dup pair is e20×f11; no third copy
exists.

**Witness union:** research/71 §3c (e20's swap/no-back/pressure/booster-30→31→30
+ `dw_e20_the_switch_track_flips_front_and_back` + the core request pin; f11's
table-sanity + cooldown-expiry pins re-pointed).

## 7. e21-unique fixes — KEEP all four, no dups anywhere

Verified against every other tip (research/71 §4 + this lane's re-check of
c42/f11/g4/d6): no other branch touches `xfer_water`, `switch_monolith`, the
546/557 arms, or the app `StatueSpawn::Boulder` drain.

| Fix | C# anchor | Content | Witness |
| --- | --- | --- | --- |
| Pump `xfer_water` | `Wiring.cs:481-485` | drops the invented `active()` gate — a liquid-bearing INACTIVE body corner transfers (the 142/143 arm registers all four corners) | `the_pump_moves_liquid_from_an_inactive_body_corner` |
| Monolith kind gate | `WorldGen.cs:51037-51134` | flips ONLY the nine kinds (410 ±56; 480/509/657/720/721/725/733 ±54; 658 +54-wrap-162); non-monolith cells in the rect keep frames (no final `else` in C#) | `the_monolith_leaves_non_monolith_cells_untouched`, `the_monolith_variants_flip_by_their_own_steps` |
| Grate 546/557 | `Wiring.cs:2541-2550` | the `SquareTileFrame` wake's object-cascade drops QUEUE into `actuation_drops` (were discarded); comment corrected (grate, not "water fountain stone block") | `the_grate_pair_swaps_types_and_queues_the_wake_drops` |
| Boulder 531 drain | `Wiring.cs:2002-2010` | app `StatueSpawn::Boulder{x,y}` → `DeviceShot{kind:99, damage:70, knock_back:10.0, v=(0,0)}` on the shared device-shot seam; auto-spawns once the aiStyle-25 family row lands, drops with the ANALYSIS note until then | (none new — the base core pin `dw_1_3_the_boulder_statue_requests_the_99_projectile` already covers the request; the spawn is family-gated) |

The core 531 arm (`boulder_statue`, CheckMech(900), anchor `(num89+1)*16,
num90*16+28`) is BASE code (S1-L-6) — no lane re-ports it; f11's app drain
still discards the Boulder variant, so e21's arm is the only copy. Textual
adjacency only: e21's Boulder arm sits ~18 lines above e20's drain extends in
`update_wiring`, and both append to `s1_l6_tests` — resolve by union.

## 8. Open seams (documented, NOT invented — the merger must not "fix" these silently)

1. **`Collision.EmptyTile(ignoreTiles: true)` entity half** — tr-core has no
   entity store; the c42/f11 wiring arms pass the always-free closure
   (`smart_cursor.rs:312` precedent). The app-side follow-up: the wiring-tick
   drain passes a real `cell_blocked` (the `doors.rs::cell_blocked_by_entity`
   shape). **g4's NPC-bash drain closure (player + NPC box overlap,
   `53063dd1`) is the reference implementation.** Door 11 is unaffected
   (`forced:true` skips the gate entirely).
2. **`NetMessage.SendData(19)`** door/gate/trapdoor echoes
   (`Wiring.cs:1450/:1458/:1465/:1480/:1485`) — Stage-2, single-player host;
   the trapdoor `num66` direction and the gate `4+flag4` subtypes are computed
   nowhere yet.
3. **`Wiring.SkipWire` inside the WorldGen bodies** — the C#
   `Main.netMode != 1 && Wiring.running` gate maps to the out-vec/Option
   collection (§1); the netMode conjunct is implicit (SP). Order-identical:
   the C# skips precede the trailing reframe, which reads no skip state.
4. **Door-cue stage streams** — the pitch draw (`Next(-20,21)*0.01`,
   LegacySoundPlayer.cs:728-749) rides the CALLER's stage stream, never the
   wiring stand-in; the c42 drain-site union (§1) is the contract.
5. **d6 `try_toggle_light` `skip_wires=true` path** — carried in the signature,
   unimplemented (every current caller passes false). The §4 unification
   closes it structurally; until then it stays a documented skip.
6. **`NetMessage.SendTileSquare` tails** in every Toggle* static — MP-only,
   unported (all lanes agree).
7. **Hopper `SyncItem`/`VisualizeChestTransfer`** (`Wiring.cs:2670-2671`) —
   net/draw seams (the particles task); both lanes document out.
8. **f11 `FrameTrack` non-pound branch + `flag4`/`KillTile(fail)`**
   (`Minecart.cs:976-1038/:1110-1166`) — the hammer-placement half
   (`edit/hammer.rs` `Consumed` seam), follow-up task; use the e20
   assert-on-front-absent convention when it lands (research/71 §3b.4).
9. **`WireState::any`** (`Tile.anyWire`) — e20-unique helper, unconsumed; the
   `IsHopperInRangeOf` (`Wiring.cs:2628`) follow-up reads it. Keep.
10. **Boulder aiStyle-25 `proj_data` row** — absent; e21's DeviceShot drops
    with the ANALYSIS note until the family lands (§7).
11. **`Wiring::can_kill_tile` slice completeness** — the chest-inventory cases
    (M7-TE-2) pass vacuously (port chests hold no inventories at the slice's
    call sites); c42's `pub(crate)` bump exposes it to `edit/shift.rs` — the
    trapdoor close guard is the second consumer.

## 9. Merge order (the playbook)

1. **c42 fix round first** (`de176ccd` content): 144/421/422 wakes + actuator
   guard, 235/226 `downed_plant_boss`, 506 shared validator. NO other lane
   touches these arms — f11's tip still carries the OLD 144/421/422/506 code;
   a merge that takes f11's arm region wholesale silently reverts the fixes.
2. **Doors/trapdoors/tall-gates**: land ONE edit-layer seam (§1-3): c42's
   `edit/shift.rs` bodies (paint+coating, in-body cues, touch, alternate-0 def
   decode, superset reframes) with f11's `only_close_or_open` param and the
   chosen skip-gate shape; drop f11's `edit.rs` shift copies and g4's
   `shift_tall_gate` copy; re-point g4's NPC drain (real entity closure stays,
   cues drain on UpdateNpcs). One cue queue + the c42 drain-site union.
3. **Light/slabs**: one torch arm + one campfire body (§4); unify the base
   `Wiring` Toggle* methods with d6's forced-state free fns; keep
   `try_toggle_light` (unique) re-pointed; keep the e20 skip-set witnesses.
4. **Devices**: research/71 §5 checklist verbatim (e20 bodies keep, f11
   minecart.rs + wiring_devices.rs drop, PlaySound(7) + OnceLock merge in,
   `TrapRequest::Spear` shape → e20).
5. **e21**: union-merge the four unique fixes (§7) — expect textual adjacency
   in the `update_wiring` drain block and the `s1_l6_tests` tail only.
6. **Docs**: PORT_MAP D8-residual rows → e20 modules; delete rows naming
   `tr-core::minecart`/`wiring_devices.rs`/f11-shift/g4-shift; ANALYSIS keeps
   the e20 device section + the c42 claim doc's census; this doc + research/71
   are the reconcile record.

**Never-ship-both list** (the actual duplicate symbols):
`shift_trapdoor` ×2, `shift_tall_gate` ×3, `get_top_left_and_styles`/
`top_left_and_styles` ×2, `open_door_wired`/`close_door_wired` vs the
Option-param `open_door`/`close_door`, `door_cues` vs `door_sounds`,
`toggle_camp_fire`/`toggle_campfire` ×3 (c42/f11 methods + d6 free fn),
torch ±66 arm ×2 (+d6's `toggle_torch`), 130/131 arms ×2, the 8 base Toggle*
methods vs d6's free-fn copies, `flip_switch_track` ×2, `frame_track`/
`frame_track_pound` ×2, the track tables ×2, `nearby_tiles_set_lookup_index`/
`nearby_track_mask` ×2, `track_switch_options` ×2, hopper/extractinator
bodies ×2, `coins_split` ×2, `try_find_chest_for_extractinator`/
`find_chest_for_extractinator` ×2, `FIRST_PRESSURE_FRAME` ×2.

*All C# anchors re-read at the cited lines in this worktree's
`decompiled/Terraria/` checkout; all port comparisons read from
`git diff a725acb0..<tip>` per file. No cargo/tests/graphify run (code-only
lane); every witness named above is WRITTEN NOT RUN on its source branch.*

---

## 10. EXECUTED — the union-exec record (this branch, on top of `ec568b3e`)

Executed code-only on `task/W4-c42-wiring-device-arms` (the wiring
canonical). Order: items 1→2→3→6→5→4→3-finish→4-finish (the light
unification ran BEFORE the e20/e21 cherry-picks so their hunks land on the
final method shapes; e21 before e20 per the §9.5 adjacency note — all
auto-merged except two additive field/drain conflicts in `264dc4b7`,
resolved by union, and the anchors.txt tail in `12cc46a2`, resolved by
union + re-pointing the c42 canary rows at the surviving symbols).

| Item | Status | Commits | What landed |
| --- | --- | --- | --- |
| 1 — doors Option shape | **EXECUTED** | `f982f1d4` | f11's `wire_skips: Option<&mut Vec<(i32,i32)>>` appended to `open_door`/`close_door` themselves; the c42 `open_door_wired`/`close_door_wired` wrappers + `*_inner` split DROPPED; the `Main.netMode != 1 && Wiring.running` gate lives at the arm call sites (`if self.running { Some(&mut skips) } else { None }`); all interaction/NPC/test callers pass `None` (edit.rs ×3 + ~14 test sites, npcs.rs bash drain). The c42 11-site `door_cues` drain union KEPT untouched. f11's unique `w4_f11_the_door_direction_draw_is_deterministic` landed beside the c42 stream-join pin; the open/close overlap pair kept as the c42 versions (they pin the SkipWire sets). |
| 2 — trapdoor param | **EXECUTED** | `ed88ec3d` | `only_close_or_open: i32` merged into the c42 `shift_trapdoor` (the `:51450`/`:51534` branch gates verbatim; the wiring arm passes the C# default −1); `shift_trapdoor`/`shift_tall_gate` skip seams adopt the §1 Option shape. The c42 body wins all kept: paint+fullbright coating, in-body per-site cues (incl. the double-thud), `touch()`, the injected `can_kill` slice, the radius-2/4 superset reframes. New gate witness `c42_the_trapdoor_only_close_or_open_gates_the_direction`. |
| 3 — tall gates + g4 re-point | **EXECUTED** | `552a0a44`, `23411fa7` | The c42 `shift_tall_gate` body untouched (already canonical: alternate-0 decode, padding-free row walk, radius-4 reframe); the §3 NPC-bash drain re-point contract documented ON the fn and LANDED in `world_sim/npcs.rs`: g4's real player/NPC box-overlap closure kept, `wire_skips: None`, the in-body cues drained on UpdateNpcs at the adjusted anchor centre (g4's hit-cell `play_door_sound` call replaced), refusal `ai[3]=60`+`net_update`, success `world_key=None`, drops on UpdateNpcs. g4's `edit.rs` shift copy NOT taken (alternate-1 + radius-2 gap dropped with it). S1-D-3z stand-in docs retired (npc.rs, npc_ai_fighter_scan.rs). Both g4 witnesses landed re-pointed (`w4_g4_the_door_bash_drain_shifts_the_tall_gate`, `w4_g4_shift_tall_gate_opens_the_closed_gate` — fixtures re-derived to the real placement grid frameY 0/20/38/56/74, verified against `tileobject/registry.rs` D_388 `coordinate_heights [18,16,16,16,18]` + padding 2). |
| 4 — e20 devices | **EXECUTED (cherry-picked, not attach-contract)** | `39b9d71c`, `6dc38f37`, `0df48370`, `264dc4b7`, `4e4f8a92`, `26ae3739`, `83ef35ff`, `05b6f940`, `12cc46a2`, `f1c24c38`, `e8e6b159` | The whole e20 code+docs slice cherry-picked (fix round, light/429 witnesses, b=0 pin, the three D8-residual arms + app minecart extension + hopper/extractinator bodies, grab-box + partial-merge + cap-fixture + lantern fixes, D8-closure docs, research/71). **No `e20c` branch exists on origin** (re-checked at exec time) — the PlaySound(7)/OnceLock wins are f11's and were merged INTO the kept e20 code per research/71 §3b (`e8e6b159`): `try_put_item_in_chest` returns `(placed, remaining, filled_empty)` and `hopper_pull` plays `play_pickup_sound(false, app.wiring.rand_mut())` on the empty-slot path (the wiring-stream draw, order-identical); `track_switch_options` wrapped in the OnceLock 64-mask cache over `TRACK_FRAMES` (returns `Option<&'static [i32]>`). The f11 §3b.3 witnesses re-pointed: coins_split 1234567, grab-rect 192, locked-chest refusal (fixture keeps frameX 72 — `chest_at` would reset it), non-anchor hopper/extractinator hits (wire re-routed around the anchor cell — discriminating under the e20 hit-cell request seam), the three cooldown-EXPIRY re-fires, the track-table sanity + lookup-mask pins (in app minecart.rs). f11's `tr-core::minecart`/`wiring_devices.rs` NOT taken (never existed on this branch). |
| 5 — e21 unique ×4 | **EXECUTED (cherry-picked clean)** | `7c9a67a2`, `8a3d7c7a`, `7ef9166b` | pump `xfer_water` active-gate drop, monolith nine-kind gate + grate 546/557 wake drops, boulder-531 app drain → DeviceShot{99,70,10.0}. All three auto-merged (the §7 adjacency predictions held; the Boulder arm sits above the e20 drain extends, both `s1_l6_tests` tails unioned). |
| 6 — d6 try_toggle_light unification | **EXECUTED** | `6381b52d` | ONE free-fn family (the 10 `Toggle*` statics + the unique `try_toggle_light` dispatch) with `forced: Option<bool>` AND a `skips: Option<&mut Vec>` out-collector; the base `Wiring` methods became thin wire-path wrappers (`None` + `Some(&mut skips)` → `self.skip`; campfire drops → `actuation_drops`); the torch arm adopts f11's `SET_TORCHES` guard form calling the unified `toggle_torch`. The forced gates follow the C# per-static form (early-return vs wrap; campfire's gate AFTER the validate). `skip_wires=true` now feeds the collector (the §8.5 seam structurally closed; the collected cells are discarded in `try_toggle_light` — no free-fn caller owns a flood skip set, documented). d6's 4 forced-gate witnesses + f11's unique actuated-torch witness landed; the e20 skip-set witnesses (`6dc38f37`/`05b6f940`) pin the unified family's wire path. **Attach contract**: `pub fn try_toggle_light(world, x, y, forced: Option<bool>, skip_wires: bool, drops: &mut Vec<ObjectDrop>) -> bool` is UNCHANGED — d6's `projectile_ai_guns.rs:183/:189` callers (on `task/W4-d6-proj-ai-51-60`, not merged here) attach verbatim when that lane lands. |

**Witness union state**: every KEEP/unique witness named in §1-7 is present
on this branch under its original name (re-pointed where the signature
changed); the dropped overlaps are f11's open/close/trapdoor/tall-gate
duplicates (the c42 versions pin strictly more) and f11's plain-rail/
no-back duplicates (the e20 versions ride the kept seam). The g4 unit
witness additionally pins the adjusted-top cue position. New witnesses
added by the exec: the `only_close_or_open` gate pin, the f11
deterministic-draw + actuated-torch pins, and the nine re-pointed f11
device pins. All WRITTEN NOT RUN (code-only).

**Canary**: the c42 anchors rows for `open_door_wired`/`close_door_wired`
were re-pointed at `pub fn open_door(`/`pub fn close_door(` (the wrappers
are gone — the rows would have failed the build); the four e20 rows landed
verbatim. The `fn toggle_camp_fire` row still matches (the wrapper kept its
name).

**Open seams after the exec**: §8.1 (the wiring-tick drain still passes
the always-free closure — the g4 NPC closure is landed and is the
reference), §8.2 SendData(19), §8.5 (collector landed, sink pending),
§8.6-8.10 unchanged; §8.11 unchanged (the trapdoor close guard still rides
the injected slice). The §9.6 doc rows (PORT_MAP D8-residual → e20
modules) landed with `12cc46a2`; no f11/g4-shift rows ever existed on this
branch to delete.
