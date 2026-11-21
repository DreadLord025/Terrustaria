# 67 — The e1b-141 reconcile: e1 drop-hunks + keep-lanes (f3 × g66 × e1)

**Lane:** the player-move family (pulley / carpet / wallclimb / stealth — W4-f3
canonical) + the player-dust site lanes. **Status:** three-way reconcile input,
code-only (no compiler invoked). **Producer:** W4-f3 seal pass,
`task/W4-f3-dust-player-3`, 2026-11-21. **Consumer:** the e1b-141 merger.

This publishes the e1 drop-hunks (the exact `player.rs` / `player_dust2.rs` /
`world_sim/player.rs` ranges the merge removes) and the e1 keep-lanes (the disjoint
site lanes that land as-is). It is the e1 side of the three-way reconcile; the f3
side is the PLAYER-MOVE-FAMILY declaration (ANALYSIS retire item 4) and the g66
side is research/66 (the mutual receipt).

## 0. Is e1b active? — the branch probe

**There is NO `task/W4-e1b*` branch** (`git branch -a | grep e1b` → empty). The
active player-dust lane is **e1 itself**: `origin/task/W4-e1-dust-sites-player-2`,
tip **`3245d624`** ("W4-e1 fidelity fixes + docs", 2026-11-21 07:08 +0200 — the
40/40-site completion + the fidelity-fix round). "e1b-141" is the **merge lane's
label** (the unclaimed three-way compose of f3 × g66 × e1), not a separate worker
branch. The reconcile below ran against e1's exact pushed tree
(`git show 3245d624:crates/...`), not commit messages. e1's tip is the latest on
its branch; no newer e1 commit exists to re-reconcile against.

## 1. The e1 wall-machine copy — DROP (five verified C# deviations)

e1 `player.rs:10041-10197` (`wall_climb_movement` `:10041`, `climb_probe_solid`
`:10125`, `wallslide_movement` `:10137`) + `player_dust2.rs::wall_cling_puff`
(`:546`) + `WallClingDust` (`:536`). The five deviations from the gravDir-1 C#
(re-verified against `Player.cs:21607-21840` + `Utils.cs:1807-1810`):

1. **`:10079`** keeps `&& !(self.vy < gravity)` in the climb gate. The C# `:21639`
   clause `(!(vy < gravity) || gravDir != -1)` is VACUOUS at gravDir 1 (the
   `gravDir != -1` disjunct is always true, so the whole `&&` term is always true
   and drops out). e1's extra AND blocks the early return: a resting Rat
   (vy ≤ 0, no keys) wrongly clings + sets `sliding` where the C# returns.
2. **`:10105`** the same vacuous-clause bug on the level-1 gate (`:21709`'s
   `!(vy < -0.5) || gravDir != -1`): e1 proceeds while rising fast.
3. **`:10171`** the wallslide `:21777` gate, same bug (clings at rest).
4. **`:10186`** `else if (wall && vy > 0.5) || vy < -0.5` — the second disjunct is
   the C# `:21814` gravDir −1 arm (`(vy < -0.5 && gravDir == -1)`), UNGUARDED in
   e1: a rising player slides with NO wall.
5. **`:10130`** `climb_probe_solid` divides (`(probe.x / TILE_PX) as i32`) where
   `ToTileCoordinates` is `(int)v >> 4` (`Utils.cs:1807-1810`) — the arithmetic
   shift differs on negative probe coordinates (division truncates toward zero,
   the shift floors).

f3's copy has none of these (the folds are cited per-arm: `wall_dust_suppressed`
`player.rs:10021` uses `(p.x as i32) >> 4`). **DROP wholesale:** e1's
`wall_climb_movement` / `wallslide_movement` / `climb_probe_solid` + the
`pending_wall_cling_dust` field (`:2063`) + its spawn (`:3031`) +
`take_wall_cling_dust` (`:3657`) + the `WallClingDust` struct + `wall_cling_puff`
drain + the `world_sim/player.rs:1481-1489` drain call. f3's
`WallDust`/`drain_wall_dust` survive.

**The e1 wall host call** (`player.rs:7630-7634`, the Rat-55 / else wallslide
dispatch) is dropped with the machines; f3's call site (`:8097-8102`, the
`!grappling && !tongued && !pulley_ride` gate) survives.

## 2. The e1 vortex-stealth machine — DROP (vortex-arm-only + wrong spawn)

e1 `player.rs:10290-10331` (`update_vortex_stealth`) + `player_dust2.rs::
vortex_stealth_rings` (`:1117`) + `VortexRingDust` (`:1106`).

* **The machine is the vortex arm ONLY** — the C# `:25733-25899` stealth chain is
  a Psycho-Knife / Shroomite / Vortex if-else-if with a final `else stealth = 1`
  reset. e1's `update_vortex_stealth` ports just the setVortex arm and DEFERS the
  reset to "the shroomite lane" (its own comment `:10293-10297`). **f3 IS that
  lane, already landed** (`update_stealth` `player.rs:9856`, the full chain).
* **The spawn default is WRONG:** C# `Player.cs:1562` `public float stealth = 1f;`.
  e1 spawns `stealth: 0.0` (`:3063`); f3 spawns `stealth: 1.0` (`:3311`). A fresh
  e1 player is fully stealthed (invisible, +ranged) until the first ramp — wrong.
* **The ring drains are chain-identical** (f3 `drain_vortex_stealth`
  `player_dust.rs:991` × e1 `vortex_stealth_rings` `player_dust2.rs:1117`, both
  replay `:25874-25893` draw-for-draw: two Next(2) gates, the RotatedByRandom(2π)
  TWO-NextDouble spins, Next(5,11) radius / NextFloat scale / fadeIn 0.5). KEEP
  f3's queue + drain.
* **e1's set-loss disarm** (`3245d624`, `:16315-16321`) is already f3's
  (`player.rs:7064`, the Always_Vortex `!set_vortex → vortex_stealth_active=false`).

**DROP wholesale:** e1's `update_vortex_stealth` + the `pending_vortex_ring_dust`
field (`:2075`) + spawn (`:3038`) + `take_vortex_ring_dust` (`:3680`) + the
`VortexRingDust` struct + `vortex_stealth_rings` drain + the
`world_sim/player.rs:1536-1545` drain call + the `stealth: 0.0` spawn row (f3's
`1.0` survives). The e1 host call (`player.rs:7203`) is dropped; f3's
`update_stealth` call survives.

## 3. The shared-site overlaps — KEEP f3's drains, e1's copies DROP

The retire(2) bible UNDERSTATED these: it listed stoned/drowning/splash as
"pre-21654, no f3/g66 counterpart", but **f3's batch 1/2/4 ALSO ported them**
(`drain_stoned_dust` `:1033`, `drain_drown_bubble` `:1065`, `drain_splash_dust`
`:1374`, `drain_floor_dust` `:1114`, `drain_double_jump_dust` `:748`). The seal
re-compared each pair against the C# and the verdicts are:

| Site | C# | f3 drain | e1 drain | Verdict |
| --- | --- | --- | --- | --- |
| Stoned burst | `:25226-25233` | `drain_stoned_dust` `:1033` | `stoned_burst` `:1080` | KEEP f3 (identical: 20 type-1 + per-puff Next(2) after the spawn internals); DROP e1's `StonedBurstDust`+`stoned_burst`+drain call `:1527` |
| Drowning bubble | `:23283-23316` | `drain_drown_bubble` `:1065` | `drown_bubble` `:771` | KEEP f3 (the Next(20) gate on the tick, `:12856`); DROP e1's `DrownBubbleDust`+`drown_bubble`+drain call `:1548` |
| Liquid splash | `:27783-27940` | `drain_splash_dust` `:1374` | `liquid_splash` `:1188` | KEEP f3; **f3 ADOPTED e1's integer half** (`(float)(height/2)`, the seal); DROP e1's `LiquidSplashDust`+`liquid_splash`+drain call `:1668` |
| MakeFloorDust | `:23790-24043` | `drain_floor_dust` `:1114` | `make_floor_dust` `:816` | KEEP f3; **f3 ADOPTED e1's integer half** (`:24026`) + the **falling head capture**; DROP e1's `FloorDust`+`make_floor_dust`+drain call `:1697` |
| DoubleJumpVisuals | `:21892-22021` | `drain_double_jump_dust` `:748` | `double_jump_visuals` `:599` | KEEP f3 (the d51 performing-jump lifecycle + the Cloud sandstorm-spent gate `:9596`); **f3 ADOPTED e1's sandstorm gore argument-draw** (`:21922` Next(220,223), the seal); DROP e1's `DoubleJumpVisualDust`+`double_jump_visuals`+drain call `:1490` |

**The e1 host machines for these sites** (`double_jump_visuals` `:10204`,
`check_drowning` `:11923`, `floor_visuals` `:11469`, the stoned edge `:7873`, the
splash host `:6008`) are dropped with their drains — f3's hosts survive
(`double_jump_visuals` `:9580`, `check_drowning` `:12811`, `floor_visuals`
`:12327`, the stoned edge `:8353`, the splash host `:6401`). The e1
`cancel_all_jump_visual_effects` (`:10032`) is f3's too (the `:26135` pulley cancel
host fires on f3).

### 3a. The falling-flag head capture — KEEP-E1 (ADOPTED into f3)

e1 `3245d624` captures `falling` at the movement head (`:7854`, the
pre-integration velocity, `Player.cs:28073-28081`) so a landing tick reaches
`FloorVisuals` with vy == 0 but falling == true and fires the 20-dust burst. f3's
OLD capture sat on the REST path (`vy == 0` gate) where `vy > gravity || vy <
-gravity` is always false — f3's landing burst never fired. **f3 ADOPTED e1's head
capture in the seal** (`player.rs:8334`, before `move_vanilla`); the
`w4_f3_the_landing_tick_captures_falling_at_the_head` witness pins it. g66
`248f0ff4d` also carries the head capture (`:8278`) — all three agree; the merge
keeps one. This is a ONE-LINE re-site, not a machine rewrite.

### 3b. The drain execution order — KEEP-E1 (ADOPTED into f3)

e1's `3245d624` fidelity round reordered the drains to the C# execution order:
hair-dye / manaHeat before stoned / Vortex / drowning / splash / floor; floor dust
after the splash sound. The C# `Player.Update` order (verified):
`UpdateHairDyeDust` `:24593` → manaHeat `:24630` → stoned `:25219` →
`UpdateArmorSets`/stealth `:25685`/`:25826` → `HorizontalMovement` `:26688`
(fast-run) → dash `:26757` → wall `:26760` → carpet `:26766` → DoubleJumpVisuals
`:26767` → `WingMovement`/glide `:26818`/`:27034` → `CheckDrowning` `:27655` →
splash `:27783` → `FloorVisuals` `:28271`. **f3 ADOPTED the wing-glide-before-drown/
splash + drown-before-splash reorder in the seal** (`world_sim/player.rs`): the
old f3 order drained wing-glide after the floor dust and the drown bubble after
the splash — cross-lane RNG-order deviations. The merged `world_sim/player.rs`
drain sequence follows e1's order with f3's wall/double-jump slots between dash
and wing-glide.

## 4. The e1 OWN lanes — KEPT (no f3/g66 overlap, land as-is)

These are e1's pre-21654 equip/vanity dust lanes + the cosmetic/ambient sites. They
have NO f3/g66 counterpart and survive the merge intact (`player_dust2.rs`):

* **Yoraiz0r eye trail** (`:12569`, `yoraiz_eye_trail` `:83`) — the eye grant +
  social-shadow facing history + headgear/mount/sitting geometry.
* **GlassSlipper sparkles** (`:14291`/`:14300`, `glass_slipper_sparkles` `:146`) —
  the per-equipped-slot 5077 gates.
* **Solar shield-gain burst** (`:16389`, `solar_shield_burst` `:207`) — the setSolar
  row + 180-tick counter + buff-170 shield derivation.
* **Teleport visuals** (`:16623`/`:16633`/`:16652`, `teleport_visuals` `:254`) — the
  teleportTime/Style countdown + portal colour.
* **Regen sparkles** (`:19273`/`:19296`/`:19555`, `shiny_stone_sparkle` `:337` /
  `crimson_heal_bursts` `:385` / `mana_full_burst` `:431`) — the regen-engine event
  counters drained to queues.
* **MowTheLawn swing puff** (`:20320`, `mow_lawn_puff` `:502`) — the Lawn Mower 4049
  lerp anchor + NewDustDirect.
* **Hair-dye puff** (`:24274`, `hair_dye_puff` `:965`) — the hairDye shader-index
  grant + TeamDyeShaderIndex=9 gate + the gore argument draws for d55. The Hair Dye
  Remover keeps 0 (`Item.cs:48714`, the `3245d624` fix).
* **manaHeat ambient** (`:24641`/`:24654`, `mana_heat_ambient` `:1025`) — the Remap
  gate + NewDustDirect + CloneDust white twin.

The e1 host machines for these (`yoraiz0r_eye`, `update_social_shadow`,
`update_teleport_visuals`, `mow_the_lawn` `:10243`, `update_hair_dye_dust` `:10337`,
`mana_heat_ambient`) + their `take_*` accessors + the `world_sim/player.rs` drain
calls (`:1405-1479`, `:1505-1524`) all survive. The e1 `rotate_by`/`rotate_about`
helpers (`player_dust2.rs:464`/`:472`) survive (the vortex rings use them, but f3's
`drain_vortex_stealth` has its own rotation — the helpers stay for e1's other lanes
if any; if unused after the drop, the merge removes them as dead code).

## 5. The net e1b-141 merge recipe (player-move family + dust lanes)

**f3's machines + grants + queues survive** (pulley / carpet / wallclimb / stealth
+ the stoned/drown/splash/floor/double-jump drains). **g66 contributes** the
integer-half corner (ALREADY adopted into f3), the `should_draw_wings` mount-gate
fix (ALREADY adopted), and its wing/boot/trail lanes (no f3 overlap; f3's partial
wing-glide copy DROPS to g66's). **e1 contributes** its disjoint pre-21654 site
lanes + the falling-flag head capture (ALREADY adopted) + the drain execution order
(ALREADY adopted) + the floor/splash integer halves (ALREADY adopted) + the
sandstorm gore argument-draw (ALREADY adopted). **e1's wall machines, wall-cling
drain, vortex machine/rings, `stealth: 0.0` spawn, and the stoned/drown/splash/
floor/double-jump duplicate drains are DROPPED** (the ranges above).

**RNG contract:** the adopted arms are C#-correctness fixes (integer halves change
a position expression, not a draw count; the falling capture / drain order / gore
argument-draw restore draws the C# spends). The sandstorm gore argument-draw
ADOPTION adds one `Next(220,223)` per gate-firing sandstorm tick that f3 previously
skipped — this RESTORES the C# stream (a determinism fix, not a deviation). The
dropped e1 copies are chain-identical or lane-disjoint. No net stream shift beyond
the restored C# draws.

**Witness ledger:** f3's 54 `w4_f3_*` fns + the seal's 2 new (the falling-capture +
the sandstorm-gore-argument) survive; e1's `w4_e1_*` witnesses for the KEPT lanes
survive, the dropped-lane witnesses drop with their machines. All WRITTEN NOT RUN
(code-only).
