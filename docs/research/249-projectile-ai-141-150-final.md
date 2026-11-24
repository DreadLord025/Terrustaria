# 249 — W4-d15 RETIRE/FINAL: the projectile aiStyles 141-150 band ledger

> Numbered **249** per the research-number collision registry
> (`150-research-number-registry.md`, W4-d54 `38bfd1d22`): a fresh
> all-remote-tip `ls-tree` scan (2026-11-21, every `origin/task/W4-*`
> tip + every local branch) shows 244 FOUR-way contested
> (d53-proj-dust-retire / f39-v18-claim-input / g48-c28-18-send-switch /
> g86-cursor-item-icon-seed), 245 g48's, 246 TWO-way (d10 FINAL /
> g43-temple-family-seal), 247 g67's, 248 claimed by the local
> `task/W4-e26-luck-system` (`248-e26-luck-final-census.md`) — **249 is
> the first free number**. The d10 FINAL precedent
> (`246-projectile-ai-91-100-final.md`) is this doc's shape.
>
> Branch `task/W4-d15-proj-ai-141-150`, base `a725acb04` (wave-3
> complete), tip at census `5b7f028d3`, 2026-11-21. CODE-ONLY round:
> **every witness WRITTEN NOT RUN** — no cargo/rustc/rustfmt/tests/
> graphify invoked. 27 commits before this retire round
> (`git log origin/main..HEAD` count), 22 files, +4020/−361 (the −361
> craft.rs/ROADMAP delta is the base-vs-main artifact: main advanced 2
> commits, `03711f456`+`051600da8` the S1-L-11g freeze guards — the
> branch touches NEITHER file; `git diff a725acb04..HEAD --stat` lists
> only task files).

## 1. The band table (all ten styles ARMED — NONE unarmed)

| style | kind(s) | C# arm | port | landing commit | notes |
|-------|---------|--------|------|----------------|-------|
| 141 | SleepyOctopod blast 698 | `AI_141_MonkStaffT1Explosion` `Projectile.cs:69602-69620` + impacts `:69680-69772` | `projectile_ai_dd2::ai_monk_staff_t1_explosion`, dispatch `projectile.rs:17380` | `2cea74544` | 20-tick counter kill; tick-1 tile-walk dust census (2 floats first row / 1 second) + 25-point rings (50/75 floats); row `:7385-7401` — FIELD-IDENTICAL to d14's copy (§4) |
| 142 | MonkStaff swings 699/708 | `AI_142_MonkStaffT2And3` `:69817-69968` | `ai_monk_staff_t2_and_3`, `:17399` | `79576fbf3` | Centre pin + heldProj claim, π+num·2π arc, the 708 animation-2/6/10 709 volley (2 doubles + Next(4)); reuseDelay=2 write drain; rows `:7402-7418`/`:7491-7507` |
| 143 | MonkStaff ghast 700 | `AI_143_MonkStaffT2Ghast` `:69970-70036` | `ai_monk_staff_t2_ghast`, `:17439` | `9db16b3d3` | tracked-sound fold to SlotId.Invalid bits, ±25 alpha ramps, ×0.98 decay, 60-tick kill, [10,34) trail; row `:7419-7432`, projFrames 4 |
| 144 | DD2 pets 701/702/703/764/1050 | `AI_144_DD2Pet` `:70038-70324` | `ai_dd2_pet`, `:17454` | `a56339c69` | per-kind hover block, ghost sparkle chain, 764 spin, the 1050 12-step IsShakeable tree scan + 100/20/5 shake census; rows `:7433-7446`/`:7866-7878`/`:9953-9966`, projFrames 3/4/8/16 |
| 145 | BookStaff twister 704 | `AI_145_BookStaffStorm` `:70326-70468` | `ai_book_staff_storm`, `:17479` | `3198932d4` | soundDelay=−1 fold, ≥16 fast-forward to 285 + 300-tick kill, 16×160 column probe ±2/±4 ladder, three 5/7/7 gore gates (G3 Y-first kicks); row `:7447-7460` |
| 146 | DD2 victory banner 713 | `AI_146_DD2Victory` `:53960-53962` — EMPTY C# body | `ai_dd2_victory` no-op, `:17494` | `38ddbe539` | free integrate + 900-tick life are the whole behaviour; row `:7560-7571` |
| 147 | Celebration Mk2 715-718 | `AI_147_Celeb2Rocket` `:52881-53092` | PRE-EXISTING `projectile_ai_misc::ai_celeb2_rocket`, `:17541` | (base) | the death resize/Damage/explosion ride `celeb2_death` (`projectile.rs:20559`) — the d77 `715..=718` cue composes there (§3) |
| 148 | StarSpawner 720 (+ the 12 shot) | `AI_148_StarSpawner` `:53094-53128` | `ai_star_spawner`, `:17507` | `c5e24faba` | daylight kill (remix exempt), day-rate age, one-shot Star.StarFall census, 180-age steer + FallingStar 12 spawn (damage 1000, kb 10, ai1 10); NO free integrate (`:20435` skip list); rows `:7617-7625` + the 12 `:924-938` (light 1f) |
| 149 | golf balls 721+739-752 | `AI_149_GolfBall` `:20331-20374` | PRE-EXISTING `projectile_ai_golf`, `:19867` | (base, S1-P-i55) | the 14 dyed twins share the 149 row shape |
| 150 | golf club helper 722 | `AI_150_GolfClubHelper` `:52417-52548` | PRE-EXISTING `projectile_ai_golf`, `:19933` | (base, S1-P-i55) | |

The coverage direction is guarded twice: the `s1_d_4c` scan
(`projectile.rs:24002`, exactly-one-live-arm per rowed style +
`UNARMED_ROWED = [140, 156, 164, 169, 206]` carries NO band style) and
the residual witness `w4_d15_the_141_150_band_has_no_unarmed_residual`
(`projectile.rs:24115` — roster + deferral exclusion + ten-style
reachability, `053a1118d`).

## 2. The kind-row census (every band kind ROWED)

698→141, 699/708→142, 700→143, 701/702/703/764/1050→144, 704→145,
713→146, 715-718→147, 720→148, 721+739-752→149, 722→150 — plus the two
adjacent rows the slice needed: **707** (aiStyle 140, `:7474-7490`, the
3858 normal-use shot — the row lands verbatim, the ARM is d14's, the
row-keep deferral `90b368a43`) and **12** (aiStyle 5, `:924-938`, the
148 spawner's shot — joins the i22 `ai_falling_star` machine,
`projectile.rs:17012-17017`, the 955 precedent + the `:23841` daylight
kill + `:23982` light 0.9). projFrames: 700→4, 709→3, 701→3, 702→4,
703→8, 1050→16 (`Main.cs:8473`/`:8554`/`:8626-8628`/`:8719`; 764 keeps
stock 1). The C# `SetDefaults` citations are pinned in the residual
witness.

## 3. The d77c cue-12 compose (cross-lane, KEEP d77's arm)

The d77 lane (`task/W4-d77-proj-sounds-3`, tip `5f10560b2`, Kill-host
wave `0573b70f2`) armed 15 landed-host kill cues on its `kill_tail_cue`
map — among them **`12 d15`**: our 12 row + daylight-kill arm is the
host their cue rides. The arm (`projectile.rs:20422` on their tip):

```rust
// `:75683` — the Starfury / natural Falling Star / Star Cannon
// star's Item10 (the 12's row + aiStyle-5 daylight-kill arm
// live on d15 `60731d50`; the unrowed 503 sibling stays out;
// the 12/955 dust rings + the screen-gated gores are the
// g65/g16 lanes').
9 | 12 | 955 => Some((2, 10, pos.x, pos.y, 1.0)),
```

C# verified (`Projectile.cs:75681-75683`): the
`type == 9 || 12 || 503 || 955` arm plays `SoundID.Item10` at
`base.position` (the pre-hook top-left — no resize precedes the play);
the 503 sibling stays unrowed on both branches. The dust/gore tail
(`:75684-75699`, the 7×58 ring + the 8-step 278 NewDustPerfect ring
with the tenth-anniversary HotPink/2 tint) is the g65/g16 lanes'
(their census, our arm's recorded layer skip).

**Compose verdict — CLEAN, no keep-one conflict:**

1. `kill_tail_cue` does NOT exist on this branch (git grep empty) —
   d77's map is the ONLY copy; nothing to dedupe.
2. Their branch has NO 12 row and NO 12 dispatch (their aiStyle-5 gate
   is `STARFURY | STAR_CANNON_STAR` only, their comment names
   "12/503/723-726/1037/1049/1133 have no rows yet";
   `git merge-base --is-ancestor` confirms NO d15 commit is in their
   history — merge-base `a725acb04`). Their cue arm is INERT on their
   branch (kind 12 can never spawn there) and ARMS at the merge when
   our row + 148 spawner land. Complementary halves, one behaviour.
3. The call site composes mechanically: their hook is
   `self.kill_tail_cue(i)` inside the `!died_out_of_world` block right
   after `try_cancel_channel` (their `projectile.rs:20025`); our
   death hook has the identical shape at `projectile.rs:20188-20192`
   — the merge inserts their one call line there. Our 12's kill paths
   (the daylight kill `projectile.rs:6875`, the blocked-move
   `kill_step` `:6931`, the 720 spawner's kill) all funnel through
   `Projectile::kill` (`:13044`) → the once-per-death hook, so every
   Kill()-shaped death cues exactly once; the out-of-world direct
   deactivate never cues (the `:70644-70648` `!active` head).
4. Citation nit for the merger: their comment's `d15 60731d50` names
   our record commit; the 12 arm itself landed at `c5e24faba` (both
   hashes are on this branch; repoint at the merge if desired).
5. RNG: the cue is draw-free (a queue push, no `Main.rand` term) — the
   stream order is untouched by the compose.

**The rest of the band's Kill()-tail census (retire completeness):**
the 704 twister's tracked-loop stop (`:71643-71650`,
`GetActiveSound(SlotId.FromFloat(localAI[1]))` → Volume 0 + Stop) is
the d80 tracked-sound layer's — our `localAI[1]` slot latch keeps the
state for it (zero draws, the fold in ANALYSIS). The 709 volley shot's
Kill host (`:73338-73388`, shared with the 459 orb: the scale≥1
144-box resize + Damage, the 3/10→7/30 dust rings + 2 gores, and the
owner-gated 1110 Sky Dragon's Fury circle sweep + spawn with the
`30*Next(2,6)` life roll + tracked DD2_SkyDragonsFuryCircle) is
UNCLAIMED by every pushed lane checked (no 1110 row on d15/d16/d17/
g1/g2/g3) — recorded here as the band's ONE unported Kill remainder,
the host-state slice a future lane owns (the SoundID row exists,
`soundid.rs:453`). No other band kind has a Kill()-tail arm in
`Projectile.cs:70640-78700` (grep census: 704/709 are the only hits;
the 715-718 cue is d77's `celeb2_death` compose above).

## 4. Cross-lane keep-one notes (d14 / d16 adjacent bands)

### d14 (`task/W4-d14-proj-ai-131-140`, tip `eaa05ae9e`)

Already cross-noted at `b1d2a416e` + the ANALYSIS final phase; the
retire re-verification stands:

- **697/707/698 rows**: d14's style-140 commit `6d7b1b9bb` landed
  field-identical copies (their 707: 16×16/140/pen −1/local immunity
  6/alpha 255/scale 1; their 698: 75×200/141/pen −1/life 5). KEEP-ONE
  row dedupe at the integrate; the compose witness
  `the_707_and_698_rows_match_the_d14_landed_values`
  (`world_sim/editing.rs`) pins our side.
- **The wiring composes**: our 3858 use arm's normal spawn
  (`editing.rs`, `the_t3_left_use_fires_the_707`) feeds their
  `ai_monk_staff` 707 machine (`:69505-69591`), whose 697 impact
  spawns the 698 our 141 arm serves. No behaviour gap.
- **Pin reconciliation**: ours `UNARMED_ROWED [140,156,164,169,206]`
  vs theirs `[141,156,164,169,206]` (verified at their tip
  `projectile.rs:24557`) → merged `[156,164,169,206]`. Each branch's
  list names the OTHER's band style; the merge drops both deferrals.

### d16 (`task/W4-d16-proj-ai-151-160`, tip `41080e159`)

The 151-160 band is disjoint from ours (their arms 151-155/157-160 +
158; their `UNARMED_ROWED [156,164,169,206]` already assumes 140/141
armed — their base predates d14/d15, so at the three-way merge the
list converges to `[156,164,169,206]` with NO further edit on their
side). NO 12/starfall/698 overlap (git grep on their tip: zero hits).
The ONE shared system is the **afkCounter small-port** — both lanes
ported `Player.cs:25497-25506` independently:

| piece | d15 (ours, `811838360`) | d16 (theirs, `6d324abb7`) | keep-one verdict |
|-------|------------------------|---------------------------|------------------|
| `Player.afk_counter` field | `player.rs:2032` | `player.rs:949` | ONE field, either copy (identical semantics) |
| `afk_counter_for_kiting` | NOT ported (the kite machine has no port state here) | ported (`player.rs:950`) | KEEP THEIRS — the kite lane owns the consumer |
| the tick block | `player.rs:5809-5823`, inside `Player::update` at the controls-landed site (PRE-movement velocity — the C# position: `:25497` runs long before `HorizontalMovement` `:26688`) | `world_sim/editing/kites.rs:158` `afk_counter_tick`, called from `world_sim/player.rs:582` AFTER the movement tick (their doc says post-movement) | **SEMANTIC — reconcile to the C#**: the counter must read the PRE-movement velocity. Keep ONE tick site at our position; their fn body is the richer one (both counters + the mouse-item/sitting/sleeping kiting zeros `:25507-25514`) — the merge folds their kiting tail into the pre-movement site and drops the second call. Their `is_considered_standing_still` helper is the shared shape of our inline `afk_still` |
| idle gate | `!input.item_in_use \|\| IS_A_KITE.contains(&self.held_item_kind)` (the world-sim fills item_in_use from app.swing) | `item_animation == 0 \|\| held_is_kite` (the ticks_of convention + inventory read) | equivalent intent; keep the merged site's existing input convention |
| `IS_A_KITE` 24-set | `player.rs:67` `IS_A_KITE` | `kites.rs:44` `KITE_ITEMS` + `is_kite` | BYTE-IDENTICAL 24 ids (verified both literals) — keep ONE (their named helper, our const dropped or re-exported) |
| `control_torch` | `Player.control_torch` field (`player.rs:2021`), fed at `world_sim/player.rs:508` | fn param from the same Torch-key read (`world_sim/player.rs:502`) | keep the field (ours) OR the param (theirs) — one feed, the key read is identical |
| AFK constants | `AFK_TIME_NEEDED_FOR_NO_LUCKY_STARS` 10800 + `..._NO_WORM_SPAWNS` 300 (`player.rs:59/63`) | `AFK_TIME_NEEDED_FOR_AUTO_KITING` 3600 (`kites.rs:141`) | DISJOINT — keep all three |
| Select `:515` resets | 3 sites: hotbar click `clicks.rs:1851`, wheel `loop.rs:422`, digit `loop.rs:686` (zeroes afk_counter only) | the SAME 3 sites at the SAME lines + `afk_counter_for_kiting = 0` (the C# `:515-516` pair) | keep-one per site, take THEIR two-field form (superset) |
| the starfall consumer | `growth.rs:228` reads `afk_counter < 10800` | none (no starfall on their branch) | ours, untouched |
| extra kite resets | none (documented unported) | reeled-kill `:48597` + live-bobber `:53685` + `projectile_ai_kites.rs:352` | theirs, kite-lane-only — no conflict |

No RNG term in either afk port (pure state) — the keep-one is
behaviour-safe once the ONE tick site sits pre-movement.

## 5. The band-141-150 declaration

**DECLARED COMPLETE (code-only).** All ten aiStyles 141-150 have
exactly one live dispatch arm; all band kinds rowed with verbatim
SetDefaults; the four wiring seams landed (the heldProj per-tick reset
`1724a40d9`, the 3836/3858 use arms + alt latch `b7fb33b55`, the 1050
tree-shake DropItems drain `5d4868ed8`, the natural starfall spawner
`ff8d2126f`); the three final-phase folds landed (starfallBoost
NightSetup `869e26d93`, afkCounter small-port `811838360`, the DD2
shake dust-body ownership cross-note `e0286df9d`); the residual census
is NONE (`053a1118d`). The static-review tail
(`f08dcbf0a`/`4058a3bf5`/`8f4dc415a`/`57aa7763a`/`2f5addde3`/`b68a228d9`/
`5b7f028d3`) closed the fixture/helper/gate nits.

**Recorded remainders (NOT band gaps — other lanes' documented
folds):** the 140 arm (d14's), the 704 tracked-loop stop (d80's), the
709/459 Kill host + the 1110 circle (unclaimed, §3), the 1050 shake
dust BODY (d54/g14's `kill_tile_make_tile_dust`, the integrate recipe
cross-noted at the drain), the starfall visual fall + `starFallCount`
(the sky layer), the 484 boosted-night critter (`NPC.cs:2406`, an
unported kind), the meteor-shower 1078 spawner (f45's documented
consumer), `afkCounterForKiting` + the spectate clamp (d16's, §4), the
`SummonMonkGhast` hit-hook spawner (`Projectile.cs:15155-15184`, a
Damage_PVE seam follow-up).

**Witness ledger — 49 WRITTEN NOT RUN** (counted mechanically from
`git diff a725acb04..HEAD -- crates/` `+#[test]` lines):
`projectile_ai_dd2.rs` 28 (s141 ×4, s142 ×6, s143 ×3, s144 ×7 incl. the
shakeable-ownership pin, s145 ×4, s146 ×1, s148 ×3), `player.rs` 1
(afk tick), `projectile.rs` 1 (band residual), `world_sim/clock.rs` 1
(dusk order), `world_sim/editing.rs` 5 (4 volley/latch + the d14
compose), `world_sim/growth.rs` 7 (5 spawner + boost threshold + afk
ceiling), `world_sim/projectiles.rs` 3 (drain/reset),
`tr-core/world.rs` 3 (NightSetup rolls). This retire round adds 2
compose witnesses (§3 cue-12 + §4 afk keep-one pins) → **51 total**.

**Merge order note:** d14 → d15 compose is row-dedupe + pin-list;
d15 → d16 compose is the afk keep-one table (§4); d15 → d77 compose is
the one-line cue-hook insert (§3). None of the three needs a behaviour
decision beyond the afk tick-site position, which this doc pins to
the C# pre-movement site.