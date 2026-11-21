# 66 — The f3 × g66 player-move mutual receipt (trail-final compose)

**Lane:** the player-move family (pulley / carpet / wallclimb / stealth — W4-f3
canonical). **Status:** mutual keep-one receipt, code-only (no compiler invoked).
**Producer:** W4-f3 seal pass, `task/W4-f3-dust-player-3`, 2026-11-21.
**Consumer:** the g66 lane (`task/W4-g66-player-trail-dust`) + the e1b-141 merger.

This is the two-sided receipt for the f3 × g66 keep-one compose. f3's side is the
PLAYER-MOVE-FAMILY declaration (ANALYSIS "the retire pass — item 4"); g66's side is
its `248f0ff4d` "final phase 1: the f3 wall/carpet keep-one inputs" commit. Both
sides agree on the verdicts below. The e1b-141 merger reads this receipt + the
declaration as the single lookup table; no re-compare is owed at the merge.

## 0. The compared tips (exact trees, not commit messages)

| Side | Ref | Tip | Verified |
| --- | --- | --- | --- |
| f3 | `task/W4-f3-dust-player-3` | `75f3f734` (pre-seal) → this seal | `git show <ref>:crates/...` |
| g66 | `origin/task/W4-g66-player-trail-dust` | **`248f0ff4d`** (NOW PUSHED; the prior `7213f3c7` is its parent) | `git show <ref>:crates/...` |

g66's tip moved `7213f3c7` → `248f0ff4d` since the f3 close-out (`0e332c61` /
`100730dd` unioned against `7213f3c7`). The seal re-verified BOTH g66 tips; the
`248f0ff4d` delta is exactly the "f3 keep-one inputs" (the tongue gate + the two
carpet consumers + the arm-by-arm verdicts), so the keep-one equality holds at the
new tip. **g66 is now pushed** — the receipt no longer depends on an unpushed local
tip (the close-out's caveat is retired).

## 1. WallClimb / Wallslide — IDENTICAL, KEEP f3

f3 `1a719d30` × g66 `b2dcd063` byte-compare after normalisation (comments,
`TILE_PX`→`16.0`, `1E-05`→`1e-5`, `flag3/flag4`→`up/down`, `!(vy>0)`↔`vy<=0`, the
`wall_probe`/`wall_dust_suppressed` helpers ↔ g66's inline forms, the `num3`
slow-fall block + the level-1 ternary canonicalised):

* `wall_climb_movement` SHA256
  `93ea3c817ad711b3f5e435841c51cdfa4a5c6ec607da7d2d188a816b37064866` (both sides)
* `wallslide_movement` SHA256
  `dcf476cc4049bd22f6e148d66967c4c245d2add13f2218d8f563de0bff428f11` (both sides)

**KEEP f3** (`player.rs:10057`/`:10129` + the `wall_probe` `:10003` /
`wall_dust_suppressed` `:10021` / `queue_wall_dust` `:10038` helpers): the helpers
are the readable form, the call site carries the `!pulley_ride` gate g66 lacks
(g66 never ported the pulley), and the `:20670`/`:20778`/`:20632`/`:26057` sliding
consumers are f3's. **DROP g66's** `wall_climb_movement` (`:10574`) /
`wallslide_movement` (`:10650`) / `queue_wall_dust` (`:10548`) +
`player_trail_dust::drain_wall_dust` (`:33`) at the merge.

**The ONE g66 arm ADOPTED into f3 (retire item 2, re-confirmed at `248f0ff4d`):**
the wall-puff corner uses INTEGER halves (`(float)(base.width / 2)`, `:21654`).
f3's float halves drifted 0.5 px on odd hitboxes. `player_dust::drain_wall_dust`
now computes `hw = width as i32 / 2` / `hh = height as i32 / 2` (`:957-958`); the
odd-hitbox (21×43) witness `w4_f3_the_odd_hitbox_corner_uses_integer_halves` pins
it. g66's `drain_wall_dust` (`:34-35`) already uses the int form — the two agree.

## 2. Carpet — IDENTICAL, KEEP f3

The `carpet_movement` bodies f3 `player.rs:10590` × g66 origin `7213f3c7`
`:10693` × g66 local→pushed `248f0ff4d` `:10705` are byte-identical after
normalisation (comments, `flag`↔`glide`, `TILE_PX`, `1E-06`): SHA256
`0a5410776a2b155bd0f383833feb991a777a8e52edef15c2dd601fab795f7a67` on all three.
The union shadow-read (`:26745-26752` pre-zero emulation) is in all three.

**KEEP f3's copy:** the call site `player.rs:8097` gates
`!grappling && !self.tongued && !pulley_ride` (the C# `:26494` else-if + the ride
bypass). g66 `248f0ff4d` adopted the tongue gate (`:8028`) but still lacks
`!pulley_ride` — its carpet would fire on a ride tick. **DROP g66's**
`carpet_movement` (`:10705`) + the grant + the local consumer copies at the merge.

**g66 `248f0ff4d`'s two consumer ports are DEDUPED to f3's** (same behaviour, f3's
citations): the `:26613-26617` glide run boost (g66 `:7727`, f3 `:7748`) and the
`:28066`/`:23537` step-up gate (g66 `:12895`, f3 `:13180`). g66 ported them as "f3
keep-one inputs"; f3 already carries them, so the merge keeps f3's and drops g66's.

## 3. The 934 grant — NOT f3-unique, NOT g66-misattributed: BOTH carry it

C# `Player.cs:15034-15037`: `if (currentItem.type == 934) { this.carpet = true; }`.
f3 `player.rs:5946` and g66 `player.rs:5941` (both tips) read
`934 => self.carpet = true,` — the same line. g66's `b2dcd063` message lists 934
among its grants and that is accurate. **One grant survives the merge — f3's row.**
The climbing grants census: f3 953|975 `+= 1` (`:5938`), 976 `+= 2` (`:5939`),
984 `= 2` (`:5921`), frog 3995/3996 `+= 2` (`:5663`/`:5670`), Rat +1 (`:5453`,
`Mount.cs:3002`); g66's rows (`:5942`/`:5943`/`:5987`/`:5723-5740` + the Rat tail
`:5512`) are semantically identical. KEEP f3's grant block.

## 4. g66's OWN lanes — KEPT (no f3 overlap, NOT dropped)

These are g66's trail-dust lane and survive the merge intact:

* **Wing dust** — `player_wing_dust.rs`: `drain_wing_air` (`:50`, `WingAirVisuals`
  `:30750-31013`), `drain_wing_glide` (`:263`, `:27040-27315`), `drain_wing_frame`
  (`:422`, `WingFrame` `:29468-30527`). The wings-33/38/47/49/50 arms + the
  `wing_glide_frames` tick machine (`player.rs:11419`) + the wings-4 Item13 cadence.
* **Boot flames + cues** — `player_boot_dust.rs`: `drain_rocket_flames` (`:40`),
  `drain_walk_effects` (`:192`); the Item13/Item24 boot cue cadence (`:26884-26896`).
* **Rudolph trail** — `player_trail_dust::drain_rudolph_trail` (`:500`,
  `Player.cs:36164-36198`, the mount-0 flight arm).
* **The `should_draw_wings_that_are_always_animated` mount-gate fix** — g66
  `248f0ff4d` corrected `data().is_some_and` → `mount.can_use_wings()`
  (`Mount.cs:570-580` returns TRUE when `_data == null || !_active`). **f3 ADOPTED
  this fix in the seal** (`player.rs:9213`): the stale form starved the
  Rainbow/Spooky/Bat leg arms unmounted (the
  `the_moon_lord_legs_overrun_the_body_rows` witness already expects row 0 for
  wings 22 unmounted). This is a g66-kept arm adopted INTO f3, like the wall corner.

### 4a. The wing-glide OVERLAP — f3's copy is a partial duplicate, DROP at merge

f3's batch 3 ALSO ported a wing-glide drain (`player_dust::drain_wing_glide`
`:1223`, wings 10/34/51/9/29/6/5/4/26/37). This OVERLAPS g66's
`player_wing_dust::drain_wing_glide`. The retire(2) bible missed this overlap (it
listed wing-glide as f3-only). The seal verdict: **KEEP g66's, DROP f3's** at the
merge — g66's is the complete wing lane (it carries the `always_animated` snapshot
field + the `wing_glide_frames` tick machine + the wings-4 Item13 cadence + the
`noLightEmittance` per-arm writes that f3 folds). f3's copy lacks the
`always_animated` field (its wings-34 gate reads `vy != 0` instead of the C#
`ShouldDrawWingsThatAreAlwaysAnimated()` predicate) and the frame machine. The
f3 wing-glide witnesses (`w4_f3_the_wings_10/4/26_*`) are dropped with the copy;
g66's wing witnesses survive. **The `should_draw_wings` predicate fix (above) is
still adopted into f3** because the leg-frame consumer (`:8715`) is f3's and lives
outside the wing-glide drain.

### 4b. The DoubleJumpVisuals / FloorDust drains — dedupe to f3 (verdict (f))

g66's `player_trail_dust::drain_double_jump_visuals` (`:58`) and `drain_floor_dust`
(`:267`) are the same six-arm / per-tile chains as f3's
`player_dust::drain_double_jump_dust` (`:748`) / `drain_floor_dust` (`:1114`).
KEEP f3's (the d51-merged performing-jump lifecycle + the cancel sites are f3's);
g66's drains dedupe to f3's. **The falling-flag head capture** (`:28073-28081`):
g66 `248f0ff4d` ALREADY carries it (`player.rs:8278` pre-integration capture), as
does e1 (`3245d624`). f3 ADOPTED it in the seal (`player.rs:8334` head capture) —
all three now agree; the merge keeps one. **The floor-dust integer half**
(`:24026` `(float)(base.width/2)`): f3 ADOPTED e1/g66's int form in the seal
(`player_dust.rs:1203`); g66's `drain_floor_dust` (`:486`) still uses the float
half — at the merge f3's int form survives (the g66 drain is dropped anyway).

## 5. The mutual agreement (both sides sign)

| Machine / lane | f3 side | g66 side | Merge verdict |
| --- | --- | --- | --- |
| WallClimb/Wallslide | KEEP (helpers + `!pulley_ride` gate + sliding consumers) | DROP machines + drain | f3 canonical |
| Wall-puff integer corner | ADOPTED g66's int form | KEPT (origin form) | f3 (g66 form adopted) |
| Carpet | KEEP (`!tongued && !pulley_ride` gate) | DROP machine + consumers | f3 canonical |
| 934 / climbing grants | KEEP block | DROP rows | f3 canonical |
| Wing dust (air/glide/frame) | DROP f3's partial glide copy | KEEP (own lane) | g66 canonical |
| `should_draw_wings` mount gate | ADOPTED g66's fix | KEPT (origin fix) | f3 (g66 form adopted) |
| Boot flames / Item13/24 / Rudolph | no overlap | KEEP (own lane) | g66 canonical |
| DoubleJumpVisuals / FloorDust drains | KEEP (d51 lifecycle) | DROP drains | f3 canonical |
| Falling-flag head capture | ADOPTED (seal) | KEPT (`248f0ff4d`) | one (all agree) |

**RNG contract:** the keep-one is behaviour-identical — the dropped copies are
byte-equal (wall/carpet) or lane-disjoint (wing/boot/trail). The adopted arms
(integer corner, `should_draw_wings` gate, falling capture) are C#-correctness
fixes, not stream changes; the wall/floor/splash integer halves change a POSITION
expression (0.5 px on odd hitboxes), not a draw count. No RNG stream shifts.

**Witness ledger:** f3's wall/carpet/grant witnesses survive; the odd-hitbox corner
+ the falling-capture witnesses pin the adopted arms. g66's wing/boot/trail
witnesses survive its own lane. All WRITTEN NOT RUN (code-only).
