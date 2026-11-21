# W4-e1b — the player-dust 3-way reconcile (e1 × f3 × g66), e1 side

Task: `task/W4-e1-dust-sites-player-2` (branch tip carries this doc).
The merge lane is **e1b-141**; the canonical-owner declaration is f3's
`75f3f7347` (W4-f3 retire item 4, the PLAYER-MOVE-FAMILY table in
`docs/ANALYSIS.md` on `task/W4-f3-dust-player-3`). This doc publishes the
**e1 keep/drop verdicts** per that f3-reconcile, re-verified against the
decompiled C# windows (≤80-line reads), and records what the e1 branch
landed to make its own copy honest until the merge.

Code-only mode: no cargo/tests/rustfmt/graphify. All witnesses WRITTEN
NOT RUN.

## Verdict table (e1 side)

| Family | C# | Verdict | e1 action this pass |
| --- | --- | --- | --- |
| WallClimb / Wallslide machines + the 5 type-31 sites | `Player.cs:21607-21840` | **DROP e1's copy at the merge** — f3-canonical (`75f3f734` verdict (d): five verified deviations). e1's `wall_climb_movement` / `wallslide_movement` / `climb_probe_solid` / `player_dust2::wall_cling_puff` / `WallClingDust` are dropped wholesale; f3's `WallDust` / `drain_wall_dust` survive. | All five deviations FIXED in e1's copy (below) so the branch is C#-honest pre-merge; 4 host witnesses pin the folds. |
| Stealth machine (Shroomite + Vortex chain + rings + toggle + set-loss) | `:25733-25899` / `:56775-56778` / `:16315-16321` | **DROP e1's vortex-arm-only machine + rings at the merge** — f3's full `update_stealth` chain is canonical (verdict (e)). The ring drains are chain-identical; dedupe to f3's. | e1's WRONG `stealth: 0.0` spawn fixed to **1.0** (`:1562` `public float stealth = 1f;`); the final-else `stealth = 1f` (`:25896-25899`) now lands in e1's fallback too; the hurt reveal `stealth = 1f` (`:38245`) landed with a keep-one cross-note vs f50. |
| MakeFloorDust / FloorVisuals | `:23708-24043` / `:28271` | **KEEP-E1's falling-flag head capture** (verdict (f)): e1 captures `falling` at the movement head (`:28073-28081`, \|vy\| > gravity BEFORE the collision resolve) so landing ticks fire the 20-dust burst. f3's rest-path capture (`vy == 0` gate) makes \|vy\| > gravity always false — **f3's landing burst never fires**. At the merge the f3/g66 floor drains survive but the `falling` input MUST come from e1's head capture (a one-line re-site). | Kept + pinned by a witness; the any-wet/cart gate (`:23740`), the Stinky buff-120 grant (`:23732`, pre-carve-out) and the integer-half (`:24026`) re-verified. |
| Drain execution order | `Player.Update` call graph | **KEEP-E1's order** (verdict (f)): the merged `world_sim/player.rs` drain sequence follows e1's EXECUTION order with f3's wall/double-jump slots between dash and splash. | Two order bugs found + fixed this pass (below). |
| e1's disjoint lanes (eye 12569, glass slipper 14291/14300, Solar 16389, teleport 16623/16633/16652, regen 19273/19296/19555, MowTheLawn 20320, hair-dye 24274, manaHeat 24641/24654, stoned 25228, drowning 23316, splash 27785-27933) | various | **KEEP** — no f3/g66 counterpart (verdict (f) last bullet); they land as-is. | Re-verified; the drown-bubble gate order fixed (below). |
| DoubleJumpVisuals | `:21892-22021` | **DROP e1's drain at the merge** — KEEP f3's (verdict (f): three copies, the d51-merged performing-jump lifecycle + cancel sites are f3's). | e1's Cloud arm gained the missing Sandstorm-suppression conjunct (`:21894`) so the copy is honest pre-merge. |
| Pulley / Carpet | `:19644-19749` / `:21842-21890` | e1 never ported either — nothing to drop. | — |

## The five wall-copy deviations (f3 verdict (d)) — fixed in e1

Each was re-read against the C# and corrected; the gravDir-1 fold is the
key: a clause `X || gravDir != -1` is VACUOUSLY TRUE at gravDir 1, and
`X || gravDir != 1` is vacuously true at gravDir -1 (unreachable here).

1. **Climb gate `:21639`** — e1 kept `&& !(vy < gravity)`; the C# clause
   `(!(vy < gravity) || gravDir != -1)` is vacuous at gravDir 1, so the
   gate reduces to `wall + (a direction key or vy > 0)`. A resting Rat
   (vy ≤ 0, no keys) now RETURNS (the old AND wrongly clung + set
   `sliding`). Fixed `player.rs` climb gate; witness
   `w4_e1b_climb_gate_resting_rat_returns`.
2. **Level-1 slide gate `:21709`** — same vacuous-clause bug
   (`!(vy < -0.5) || gravDir != -1`); only `vy > 0.5` survives. A
   fast-rising Rat no longer clings. Witness
   `w4_e1b_slide_gate_rising_rat_returns`.
3. **Wallslide `:21777` gate** — same bug (`vy < gravity` disjunct needs
   gravDir -1); reduces to `wall && vy > 0`. Witness
   `w4_e1b_wallslide_gates_need_wall_and_downward_vy`.
4. **Single-boot slide `:21814`** — e1's `else if (wall && vy > 0.5) ||
   vy < -0.5` carried the gravDir -1 arm UNGUARDED: a rising player slid
   with NO wall. Now `wall && vy > 0.5` only. Same witness as (3).
5. **`climb_probe_solid` divide-not-shift** — `ToTileCoordinates` is
   `(int)v >> 4` (`Utils.cs:1807-1810`), the truncation-then-arithmetic-
   shift, NOT `/16` (which rounds differently for negatives: -15 → shift
   -1 vs divide 0). Fixed to `(probe.x as i32) >> 4`. Witness
   `w4_e1b_climb_probe_shifts_not_divides`.

The wall/visual block also gained the `!grappling && !tongued` movement-
arm guard (`:26494` — the C# block lives inside that arm; a grappled or
tongued player never wall-clings or runs the jump==0 visual cancel).
Witness `w4_e1b_wall_block_gated_on_grapple_and_tongue`.

## Stealth rows (f3 verdict (e)) — e1 made honest

- **Spawn `stealth: 1.0`** (`Player.cs:1562`). e1's old 0.0 made the
  Vortex recovery ramp fire 25 ring ticks on the first equip. Witness
  `w4_e1b_stealth_spawns_at_one`.
- **Final else `stealth = 1f`** (`:25896-25899`) now lands in e1's
  no-set fallback (was deferred to "the shroomite lane" — f3 IS that
  lane). The Psycho-Knife arm (`:25733`, held 3106) stays f3's; when it
  lands its check goes ahead of this fallback. Witness
  `w4_e1b_vortex_final_else_and_set_loss_disarm` (also pins the set-loss
  disarm `:16315-16321`, the -0.04 fade and the Vortex ranged kb scale
  `:53364-53367`).
- **Hurt reveal `stealth = 1f`** (`:38245`) — after the dodge early-
  returns, before the defense reduction. CROSS-NOTE (keep-one): f50's
  `hurt_inner` carries the SAME write (f3-canonical per its own retire
  receipt `51eefdf84`); at merge keep ONE copy.
- **ResetEffects `aggro = 0`** (`:18377`) — without it the Vortex writer's
  `aggro -= (int)((1-stealth)*1200)` (`:25847`) accumulates across ticks.
  Witness `w4_e1b_aggro_resets_each_tick`.
- **Climbing/frog grants** (`:15038-15066` / `:14426-14444`): 953/975 +1,
  976 +2, 984 =2 (the ASSIGNMENT), 3994 none / 3995/3996 +2 plus the
  auto-jump + 1.6 jump-speed + 10 safe-fall (+accFlipper 3994/3995)
  bundle. Witness `w4_e1b_climbing_and_frog_grants_arm_spiked_boots`.
- **Dash stealth cancel** (`:21379`) — an active dash disarms
  `vortexStealthActive`. Witness `w4_e1b_active_dash_cancels_vortex_stealth`.
- **Cloud/Sandstorm suppression** (`:21894`) — the Cloud visual ALSO
  needs `canJumpAgain_Sandstorm || !hasJumpOption_Sandstorm`; an unspent
  Sandstorm suppresses the Cloud puff. Witness
  `w4_e1b_cloud_visual_suppressed_by_unspent_sandstorm`.

## KEEP-E1: the falling capture + the drain execution order

**Falling head capture** (`:28073-28081`): `bool falling = false; if
(vy > gravity) falling = true; if (vy < -gravity) falling = true;` —
captured BEFORE `vector4 = velocity` / the collision resolve, read by
`FloorVisuals(falling)` at `:28271` (gated `vy == 0 && grappling[0] ==
-1`). A landing tick arrives with vy==0 but falling=true → the 20-dust
burst (`:23825-23827`). f3's rest-path capture can never fire it.
Witness `w4_e1b_falling_head_capture_fires_on_landing`.

**Floor-dust gate re-verifications** (witness
`w4_e1b_floor_dust_any_wet_gate_and_stinky_buff` +
`w4_e1b_floor_dust_integer_half_width`):
- `:23732-23735` — the Stinky tiles 666/712 grant buff 120 for 180 ticks
  BEFORE the slope carve-out (the grant reads the REAL tile).
- `:23736-23739` — the three-tile slope window suppresses the dust KIND.
- `:23740` — `!base.wet && !mount.Cart`: `base.wet` is WetCollision (ANY
  liquid) → the port's four-flag union; the cart check covers
  `cart_mounted` + the active cart mount.
- `:24026` — `(float)(base.width / 2)`: the INTEGER half (21 → 10).

**Drown bubble gate order** (`:23283`): `if (!flag || Main.rand.Next(20)
!= 0 || base.lavaWet || base.honeyWet) return;` — the Next(20) draw rides
whenever drowning, BEFORE the wet flags refuse. The host now gates ONLY
`flag`; `lava_or_honey_wet` rides the `DrownBubbleDust` snapshot and the
drain refuses after its draw (the old host-side wet gate skipped the
draw). Host witness `w4_e1b_drown_bubble_host_gates_flag_only`; the
draw-order mirror is `player_dust2.rs::w4_e1_drown_bubble_gate_and_shape`
(extended this pass with the wet-refuse-after-draw arm).

**Drain execution order — two bugs fixed this pass** (the f3 verdict (f)
order is e1's; these make it exact):
1. **MowTheLawn before the jump sounds.** `MowTheLawn` is the TAIL of
   `HorizontalMovement` (`:20176-20179`, called at `:26688`);
   `JumpMovement` runs at `:26744`. The drain had the Lawn Mower puff
   AFTER the double-jump/Fart sound drains — its NewDustDirect draw spent
   the stream out of order. Moved before them (after the step sound,
   which is `SpawnFastRunParticles` inside HorizontalMovement
   `:19990/:20020`).
2. **The drown gasp inside CheckDrowning.** `PlaySound(23)` (`:23257`)
   fires on the breath 1→0 edge inside `CheckDrowning` (`:27655`),
   BEFORE the bubble gate (`:23283`) and before the splash block
   (`:27728`). The drain had it after the splash sound; moved ahead of
   the bubble drain. (Case 23 draws no stream — the fix is positional
   fidelity, not draw order.)

The full e1 drain order (kept at the merge, f3's wall/double-jump slots
between dash and splash): hair-dye `:24593` → manaHeat `:24630` →
social-shadow/teleport `:24691-24692` → stoned `:25219` → eye `:25487` →
glass-slipper (UpdateEquips `:25633`) → Solar + Vortex rings
(UpdateArmorSets `:25685`) → regen sparkle/crimson `:25977` → full-mana
`:25995` → movement (step sound → MowTheLawn → jump sounds → dash →
wall → DoubleJumpVisuals, `:26688-26767`) → CheckDrowning gasp + bubble
`:27655` → splash dust-then-sound `:27772` → FloorVisuals `:28271`.

## Witness ledger (this pass)

15 new `w4_e1b_*` host witnesses in `player.rs` (the wall folds ×4, the
falling capture, the floor gates ×2, the drown host gate, the stealth
rows ×6, the dash cancel, the grapple/tongue guard) + the extended
drown-bubble draw mirror in `player_dust2.rs`. All WRITTEN NOT RUN
(code-only). The 26 batch draw-mirror witnesses (`w4_e1_*` in
`player_dust2.rs`) stand as landed.

## Merge instructions (e1b-141, player-move family)

f3's machines + grants + queues survive; g66 contributes the integer-
half corner (already adopted into f3) + its wing/boot/trail lanes; e1
contributes its disjoint site lanes + the falling-flag head capture +
the drain execution order. DROP from e1: the wall machines + wall-cling
drain, the vortex machine + rings, the DoubleJumpVisuals drain (f3's
survive). The `stealth` spawn is 1.0 on both sides now (one definition,
f3's field list). The hurt `stealth = 1f` write: keep ONE copy (f3/f50
canonical per the f50 retire receipt). The shared-field list is f3's
declaration (`75f3f7347`).
