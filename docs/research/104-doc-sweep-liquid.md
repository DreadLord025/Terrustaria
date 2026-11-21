# 104 — DOC-SWEEP lane 24: the liquid census + the bucket port

Date: 2026-11-21. Author: W4-g45 (DOC-SWEEP phase, code-only — no cargo/tests
run, no compiler invoked; every witness WRITTEN NOT RUN). Branch
`task/W4-g45-liquid-lava-honey-delays`, continuing the guards-DONE tip
`8e81814e`. This note is the lane's census verdicts + the four real gaps it
ported, with C#/Rust citations. The cross-lane reconcile addendum lives in
[research/69](69-liquid-reconcile.md) (the ADDENDUM section, g46c active reads).

## 1. The census questions and their verdicts

### 1a. QuickShimmer — DOES NOT EXIST (no gap)

A grep for `QuickShimmer` over the whole 1.4.5.8 decompilation
(`decompiled/Terraria/**.cs`, incl. `GameContent/`) returns **zero** matches.
The C# has no such function. The shimmer bulk-settle is
`Liquid.AttemptToMoveShimmer` (`Liquid.cs:387-433`, the L/R/U/D
first-different-neighbour scan calling `ShimmerCheck` at the origin or the
neighbour) + `Liquid.ShimmerCheck` (`:1465-1468` → `LiquidCheck(x, y, 3)`),
both inside the `QuickWater`/`SettleWaterAt` family the g47 lane ports (the
UNION keeps them, research/69 rows 4-6). The runtime shimmer cell rides
`update_tile`'s shimmer arm (`Liquid.cs:542-545`). **Verdict: nothing to
port; the question's premise is a phantom symbol.**

### 1b. BucketEmpty water placement + wet-bucket fill — THE REAL GAP, PORTED

The bucket/sponge use arms are NOT in Liquid.cs — they are
`Player.ItemCheck_UseBuckets` (`Player.cs:46318-46504`), called from
`ItemCheck_OwnerOnlyCode` at `:43575`. The twelve kinds
(`ID/ItemID.cs:2093-2097/:3939/:7745-7747/:11323/:11427/:12287-12291/:12411`):

| id | name | arm |
| --- | --- | --- |
| 205 | Empty Bucket | pickup → converts to 206/207/1128 (shimmer refused) |
| 206 | Water Bucket | place water 255, hand back 205 |
| 207 | Lava Bucket | place lava 255, hand back 205 |
| 1128 | Honey Bucket | place honey 255, hand back 205 |
| 3031 | Bottomless Bucket | place water 255, no consume |
| 3032 | Super Absorbant Sponge | pickup water/shimmer, no box-min, no consume |
| 4820 | Bottomless Lava Bucket | place lava 255, no consume |
| 4872 | Lava Absorbant Sponge | pickup lava only |
| 5302 | Bottomless Honey Bucket | place honey 255, no consume |
| 5303 | Honey Absorbant Sponge | pickup honey only |
| 5304 | Ultra Absorbant Sponge | pickup ANY kind |
| 5364 | Bottomless Shimmer Bucket | place shimmer 255, no consume |

**Pre-sweep status: MISSING everywhere.** No lane claimed it (a
`git log --all --grep=bucket -i` returns only the draw-bucket lanes g28-g32
and the chum-bucket fishing rows, unrelated); research/11 listed "buckets"
as an open ROADMAP §3 item; the
S1-E-21f player-sounds census deferred the five bucket splashes
(`:46374-46493`) as "the buckets … deferred" (ANALYSIS "The player-sounds
wave", deviation 8). The smart-cursor `Step_EmptyBuckets` re-point
(`SmartCursorHelper.cs:622-676`) WAS ported (`smart_cursor.rs:1178`), so the
held 205 already re-points at the deep-liquid box — but the use arm that
consumes it did not exist.

**Ported: `fa96cf32a`** — new `crates/tr-app/src/world_sim/editing/buckets.rs`
(the `use_buckets` entry + the pickup/placement arms +
`put_item_from_usage` + the splash/frame seams), wired into `update_editing`
before the melee arm (the C# `:43575` precedes the melee-hitbox block
`:43644`; the buckets carry `damage = -1`, no pick/axe/hammer, no
`createTile`, so they never mine/melee/place — the arm consumes the tick, the
bug-net precedent). The shape:

- **The swing start** is the generic autoReuse use-start (`Player.cs:42733`
  → `ItemCheck_StartActualUse` `:51657-51698` → `ApplyItemAnimation`), NOT
  reach-gated; all twelve kinds are `autoReuse`
  (`Item.cs:3958-3985/:13878-13885/:28072-28084`), so the swing restarts the
  moment it ends. `start_use_timers` is the port's ApplyItemAnimation seam.
- **The action gate** (`:46320/:46329`): the Simple reach rect
  (`TileReachCheckSettings.cs:81-85`, multiplier 1 / limit 20) with the
  item's `tileBoost` (the bottomless/sponge family +2, the Ultra +3), then
  `ItemTimeIsZero && itemAnimation > 0 && controlUseItem`. A refusal leaves
  `itemTime` at zero → retry every tick while held; a success re-arms
  `ApplyItemTime(useTime)` (10 for the plain buckets, 5 for the
  bottomless/sponges, 3 for the Ultra).
- **The pickup arm** (`:46333-46424`): the 3×3 same-type box sum
  (`:46337-46346`), the `num2 <= 100` minimum with the four sponge
  exemptions (`:46347`), the Empty-Bucket conversion (honey → 1128
  `:46354-46357`, lava → 207 `:46359-46362`, else water → 206
  `:46370-46371`; shimmer refused for 205 `:46366-46369`), the target wipe
  clearing ONLY the lava/honey bits (`:46377-46379` — the shimmer bit stays
  on a 3032-drawn dry cell), `SquareTileFrame(resetFrame: false)` +
  `AddWater` (`:46380/:46387`), the `num3 >= 255` early return
  (`:46389-46392`), and the partial-cell top-up loop capping each transfer
  at `255 - num3` (`:46393-46423`, the loop runs even once num3 hits 255 —
  a zero transfer still re-frames + re-wakes).
- **The placement arm** (`:46425-46503`): the `>= 200 || non-top solid`
  refusal with the grate-546 carve-out (`:46427`), the per-kind
  empty-or-same-type gate (`:46433/:46453/:46473/:46491`), the pour to
  `byte.MaxValue`, `SquareTileFrame` (default resetFrame), the non-bottomless
  consume + Empty-Bucket hand-back (`:46439-46443/:46459-46463/:46479-46483`;
  5364 consumes nothing), `ApplyItemTime`. The SP path wakes the liquid
  through SquareTileFrame's own `TileFrame` AddWater
  (`WorldGen.cs:81344-81347`) — the C# placement arms call only
  `NetMessage.sendWater` (MP), never AddWater directly.
- **`PutItemInInventoryFromItemUsage`** (`Player.cs:42141-42160`): the
  58-slot partial-stack scan (inventory 50 + coins 4 + ammo 4, type + stack
  only — the buckets are not prefixable), the IsAir selected-slot arm
  (`Item.cs:385-395`), else `GetOrDropItem` with
  `ItemCreatedFromItemUsage` (`GetItemSettings.cs:61` = default — no void
  vault, sound on): the `deposit_pickup` walk (the chime per accepted slot)
  + the overflow spawned at `base.Center` (`GetOrDropItem` `:7193-7200`).
- **The five splashes** (`:46374/:46435/:46455/:46475/:46493`):
  `PlaySound(19, (int)position.X, (int)position.Y)` — Style defaults to 1,
  the position the player's top-left truncated to int; the case-19 playing
  guard + the one `Next(-10, 11) * 0.01` pitch draw ride
  `Audio::play_splash_sound` on the `UpdatePlayers` stream (the S1-E-21f
  seam, `LegacySoundPlayer.cs:871-888`).
- **The TurnToAir clear** (`:42824-42828`): the emptied bucket clears at the
  swing end (the port keeps the kind through the swing, like the C#).

10 witnesses (`bucket_use_tests`, WRITTEN NOT RUN): the water draw +
hand-back, the lava/honey conversion, the shimmer refusal, the box-min +
sponge exemption, the partial top-up (200 + 100 → 55 moved, 45 left), the
pour + hand-back, the >=200/solid pour refusals, the bottomless-shimmer
no-consume, the empty-or-same-type gate, the typed sponges.

### 1c. Liquid pressure / flow-rate constants — VERIFIED MATCHING (no gap)

There is NO "pressure" constant in Liquid.cs (a grep for `pressure|Pressure`
returns nothing — Terraria's liquid model is a cellular mass-transfer, not a
pressure sim). The flow-rate constants, each verified against the port:

| C# | value | port | verdict |
| --- | --- | --- | --- |
| `maxLiquid` (`:1672`) | 25000 | `MAX_ACTIVE` (`liquid.rs:16`) | ✅ |
| `maxLiquidBuffer` (`:1671`) | 50000 | `MAX_BUFFER` (`:18`) | ✅ |
| `cycles` (`:88/:1673`) | 10 | `CYCLES` (`:24`) | ✅ |
| kill threshold (`:985/:1111`) | 8 (MP `10+players/3`) | `KILL_PASSES` (`:26`) | ✅ SP; MP N/A |
| `KILL_DEAD` (`:461/:477`) | 999 | `KILL_DEAD` (`:27`) | ✅ |
| lava delay (`:485`) | `< 5` → 1-in-6 | `delay < 5` (`:657`) | ✅ |
| honey delay (`:516`) | `< 10` → 1-in-11 | `delay < 10` (`:674`) | ✅ |
| quickFall delay (`:1087`) | force 10 | `delay = 10` (`:352`) | ✅ |
| stride (`:1070`) | `curMaxLiquid/cycles` | same (`:327`) | ✅ |
| sub-3 bias (`:644-648`) | `num2 = -1` | `bias = -1.0` (`:763`) | ✅ |
| merge gate (`:1261/:1290`) | `num >= 24` / `< 24` | same (`:963/:996`) | ✅ |
| evap shed (`:468-473`) | `min(2, liquid)` | same (`:636`) | ✅ |
| runtime cycles/curMax (`Main.cs:17078-17091`) | `17-10q` / `maxLiquid*(0.25+0.75q)` | `gfx_quality.rs:117-120` | ✅ |
| the averaging divisors (`:686/:780/:845/:868/:891/:913/:927`) | 7/5/4/4/3/2/2 | `round_level(…, 7/5/4/4/3/2/2)` | ✅ |
| banker's rounding (`Math.Round`) | ties-to-even | `round_ties_even` (`:1251`) | ✅ |

The `wetCounter` window (`:1069-1108`), the `numLiquid` clamp re-walk
(`:1077-1082`), the `skipLiquid` gate (`:1096-1104`) and the quickFall
skip-clear (`:1089`) all match. **Verdict: no constant gap.**

### 1d. Honey-block / crispy-honey reaction (c34 owns the merge) — VERIFIED FED

c34 (`a7f63d95` phase 1 + `37b0b702` round 2, branch
`task/W4-c34-liquid-merge-types`) landed the cadence/merge witness suite
(delay 5/10, quickFall bypass, sub-24 down-arm drain, shimmer 659 arms) +
the underworld evaporation. Its merge reads the SAME `liquid_check` /
`get_liquid_merge_types` the g45 delay counters feed: the reactions run
EVERY visit (`LavaCheck`/`HoneyCheck`/`ShimmerCheck` before the delay gate,
`Liquid.cs:480-490/:511-522`), the delay throttles only the flow. The g45
`the_delay_gate_never_throttles_the_merge` pin and c34's
`honey_and_lava_merge_into_crisp_honey_once` (229/230) agree — the honey
cell checks first, the merge tile is not `tileObsidianKill`, so the lava
cell's own check afterwards refuses (one merge). **The delay counters feed
c34's merge correctly.** The one merge-side correction c34's suite did not
catch is the down-arm stale-read (§2a below) — its plant-3 witness is
`tileObsidianKill` and cannot discriminate.

### 1e. Shimmer liquid special physics (e19/f47 own item-side) — LIQUID-SIDE CENSUSED

e19 (`task/W4-e19-liquid-quicksell-shimmer`) and f47
(`task/W4-f47-liquid-shimmer-2`) own the ITEM-side shimmer physics
(`WorldItem.MoveInWorld`/`UpdateShimmer`, the shimmerWet gravity ladder, the
shimmered-repulsion scan, the transform/decraft spawns — `shimmer.rs`,
`item.rs`, `world_sim/items.rs`). Neither touches `tr-core/src/liquid.rs`
(their diff stats confirm: e19 = audio/item/shimmer/items/npcs; f47 =
collision/item/npc/shimmer/editing-gate/extract). The LIQUID-side shimmer is
the engine's own: the shimmer arm (`Liquid.cs:542-545`) runs `ShimmerCheck`
with **NO delay gate** (only lava `:483-491` / honey `:514-522` throttle), so
shimmer falls every visit like water at the one-tile-per-two-passes
`skipLiquid` cadence (`:586-587/:1096-1104`). The cadence suite had the
shimmer MERGE arms (`shimmer_merges_with_lava_and_honey_into_659`) but NO
shimmer FLOW pin. **Ported: `99a4f8ccf`** —
`shimmer_falls_every_visit_like_water` (a 255-shimmer column falls five tiles
in ten visits, the shimmer type rides the fall; the lava column would still
sit at the source after ten, honey's first fall is visit 11). The
`AttemptToMoveShimmer` bulk-settle half rides the g47 family (research/69
row 6); the `ShimmerRemoveWater` gen arm rides f25 (row 11).

## 2. The four real gaps ported this sweep

| # | gap | C# | commit | witnesses |
| --- | --- | --- | --- | --- |
| a | `liquid_check` down-arm stale-read | `Liquid.cs:1278-1306` | `c8908a55d` | `liquid_downarm_live` ×4 |
| b | `ItemCheck_UseBuckets` unported | `Player.cs:46318-46504` | `fa96cf32a` | `bucket_use_tests` ×10 |
| c | shimmer liquid-side flow pin absent | `Liquid.cs:542-545` | `99a4f8ccf` | `shimmer_falls_every_visit_like_water` |
| d | fall-arm pre-evap read | `Liquid.cs:570-575` | `8d581953b` | `the_fall_arm_moves_the_post_evaporation_level` |

### 2a. The down-arm LIVE post-burn reads (`c8908a55d`)

The C# `tile4` is a LIVE reference: the merge gate (`Liquid.cs:1286`) and the
merge-type reads (`:1303`) run AFTER the `tileCut` burn (`:1280`), so a cut
kind the kill just cleared passes on `!tile4.active()` even when it is not
`tileObsidianKill`. The port read `down` ONCE before the burn and re-used
that copy — wrongly refusing the drain/merge for the **seven cut kinds
OUTSIDE `tileObsidianKill`**: 444 BeeHive, 518 LilyPad, 519 Cattail,
549 Seaweed, 637 AshPlants, 638 AshVines, 711 (the generated-bitset
membership hand-computed from `tiledata/sets.rs:92-96`; the other 34 cut
kinds ARE obsidianKill and pass either way). Fix: re-read `down` after
`kill_tile` (`remove_tile` keeps the liquid + type bits,
`WorldGen.cs:63885`); the `flag` container read stays PRE-kill (`:1274`
precedes `:1278`). The existing `lava_burns_the_submerged_cut_plant_below`
witness uses plant 3 (IS obsidianKill) and cannot discriminate. 4 new
witnesses: sub-24 drain over seaweed-549, full merge over lily-pad-518,
honey arm over cattail-519, water arm burns nothing. **The g46 UNION carries
the SAME bug** (research/69 ADDENDUM A3) — the graft must take this fix.

### 2d. The fall-arm LIVE post-evaporation read (`8d581953b`)

The down transfer read the PRE-evaporation `start.liquid`
(`Liquid.cs:570-575` reads the LIVE `tile5.liquid`), resurrecting the shed
units on every fall inside the underworld band. Fix: read `own_now` after
the evaporation block (the `:464` capture stays the kill-bookkeeping value).
**The g46 UNION ALREADY carries this form** (`own_now`, `f57d7bacd:832`) —
g45 catches up; the discriminating witness (a 100-water cell in the band
falls 98, not 100, then keeps shedding to 96) is the g45 addition.

## 3. Cross-lane reconcile (research/69 ADDENDUM)

The g46 branch advanced past the original 4-way table to a graft-prep UNION
(`f57d7bacd`, the g47 skeleton + g46 grafts + g45 extra-wakes, 72 witnesses).
**The UNION dropped the g45 round-2 AddWater guards + the widened fixtures**
(research/69 ADDENDUM A1/A2): its `add_water` is g47's queue-only `&World`
form (no 5-tile border, no grate-546 carve-out, no inline death check), and
its `basin`/`floor_world` fixtures are un-widened (they go inert under the
border guard). The graft-union MUST carry the g45 guarded `add_water`
(+ the `&mut World` ripple + the e11 death-check seam) and the (+6,+6)
widened fixtures, plus the down-arm fix (A3), the bucket module + the
shimmer-flow witness + the fall-arm witness (A4). The full graft instruction
+ the g46c active reads are in the ADDENDUM.

## 4. Verification

Code-only mode: **no cargo/tests/graphify run, no compiler invoked.** Every
witness is WRITTEN NOT RUN. Verified by re-reading the C# line-by-line and
hand-arithmetic: the down-arm bitset membership (the seven non-obsidianKill
cut kinds), the bucket box-sum/top-up traces (200 + 100 → 55 moved / 45
left), the shimmer shaft cadence (five falls in ten visits), the fall-arm
evap trace (100 → 98 fall → 96 shed). The four commits are on
`task/W4-g45-liquid-lava-honey-delays`, pushed unmerged for the verification
pass. The bucket port's `&mut World` `add_water`/`wake_around` calls depend
on the g45 guarded signature (already on this branch); the e11
`check_water_death`/`check_lava_death` seam is a merge-order dependency
(e11 not in main yet).

## 5-final. The FINAL per-fn owner table (LIQUID FINAL, 2026-11-21) — the single source of truth for the merger

Written against the LIVE tips: g45 `6f5bd2a2` (this branch, + this phase's
commits), g46 `f57d7bacd` (the graft-prep UNION, g46d NOT landed —
research/69 A7 receipt), g47 `5c09a68b8` (closeout 1-4, the re-base target
per research/70 §8), f25 `task/W4-f25-worldgen-passes-8` (gen module, stays
on its branch). g47's ANALYSIS "The Liquid.cs FULL-FN final census"
(`5c09a68b8`) script-diffed all 24 members and found the sim gaps-list
EMPTY; this table AGREES with that census and adds the one axis it does not
carry: **which lane's form the merger keeps per fn, and which g45 deltas
are graft OBLIGATIONS on the new base**. Where this table and research/69's
KEEP summary or A6 disagree, THIS table + research/70 §8-12 win.

### 5a. Liquid.cs — every fn, one owner

| C# fn (`Liquid.cs`) | FINAL owner (keep whose form) | g45 graft obligations on that form |
| --- | --- | --- |
| statics + ctor (`:15-55`/`:1669-1677`) | **g47** (`LiquidSystem` fields + consts; instance-per-world) | none — g45's constants verified matching (§1c) |
| `NetSendLiquid` (`:57-67`) + broadcast tails | **out of scope** (MP, all lanes agree) | none |
| `tilesIgnoreWater` (`:69-73`) | **g47** (`4288657c` export + pass-scoped fold) | none |
| `worldGenTilesIgnoreWater` (`:75-81`) | **g47** (`4288657c` + `gen_ignore_water` fold) | none |
| `ReInit` (`:83-102`) | **g47** (incl. the reduced-liquids 5000 cap) | none |
| `QuickWater` (`:104-148`) | **g47** runtime + **f25** gen driver (verbose/noTraps/cleanup) | none |
| `SettleWaterAt` (`:150-289`) | **g47** (gen re-type + dungeon arm + ORIGIN wall, `d3a30ebf`) | none |
| `AttemptToMoveHoney/Lava/Shimmer` (`:291-433`) | **g47** | none |
| `UpdateProgressDisplay` (`:435-447`) | **out of scope** (gen loading-bar UI) | none |
| `Update` (`:449-964`) | **g47** skeleton + **g46** evap (`:466-474`) + **g46** `surface_rng` nudge (`:892-895`) + **g45** 7/5-neighbour extra wakes (`:748-771`/`:821-836`, `ccfe0b42` — the UNION carries them at :920/:941, the g47 tip does NOT) | **g45 fall-arm LIVE read** (`8d581953b`): the g47 tip STILL reads `start.liquid` (`5c09a68b8:liquid.rs:1015`) — the UNION's `own_now` form (:832) must survive the re-base; carry the g45 witness `the_fall_arm_moves_the_post_evaporation_level` |
| `StartPanic` (`:966-981`) | **g47** | none |
| `UpdateLiquid` (`:983-1158`) | **g47** (`update` + `end_pass` + the `update_world_tick` driver `3dbbe3ef` + the reduced-liquids OR-arm `d3a30ebf` + the buffer-arm pins `d9445fc4`) | the g45 pre-loop num8 promotion pin (`daa1cdb0`, `:1120-1129`) rides step 5 |
| `AddWater` (`:1160-1207`) | **g47 skeleton + g45 GUARDED BODY** (research/70 step 5): the g45 `daa1cdb0` form (`&mut World`, 5-tile border, `kind != 546` carve-out, inline `check_water_death`/`check_lava_death` → `kill_tile`) REPLACES the g47/union queue-only form, with the `edit_kills` loot upgrade (push the `TileKill`, don't discard) | **the whole g45 body IS the obligation** + the (+6,+6) widened fixtures (A2) + the `liquid_addwater_guards` ×10 module + the `&mut World` ripple through `wake_around`/promotion/app callers; e11 seam merge-order dependency |
| `UndergroundDesertCheck` (`:1209-1223`) | **g47** | none |
| `LiquidCheck` (`:1225-1309`) | **g47** skeleton | **g45 down-arm LIVE re-read** (`c8908a55d`): BOTH the union (:1131/:1143) and the g47 tip (:1274) still read `down` pre-kill — re-read after `kill_tile` (`flag` stays pre-kill); carry `liquid_downarm_live` ×4 (the seven non-obsidianKill cut kinds 444/518/519/549/637/638/711) |
| `LiquidOverwriteStrip` (`:1311-1323`) | **g47** | none |
| `CreateLiquidMergeTile` (`:1325-1373`) | **g47** (cited `ReplaceTile` fallback) | none |
| `GetLiquidMergeTypes` (`:1375-1443`) | **g47** | none |
| `LavaCheck`/`HoneyCheck`/`ShimmerCheck` (`:1445-1468`) | **g47** | the g45 delay counters feed them (§1d verified); carry `shimmer_falls_every_visit_like_water` (`99a4f8ccf`) — dedupe against g47's closeout twin `shimmer_falls_every_visit_ungated` (same assertion: no delay gate, water cadence; keep ONE name, g47's fixture form + the g45 witness's cadence-trace assertions merged) |
| `DelWater` (`:1470-1599`) | **g47** canonical wholesale (`d3a30ebf` loot seam, research/70 base verdict; the union re-derived the same seam under `retire_kills` — rename to `edit_kills`, not a behaviour merge) | the g45 sharper assertions (delay-inheritance, kill-0 survivor, mud-60 ring) per research/70 §11 |
| `AddGameplaySnapshotComponents` (`:1601-1667`) | **out of scope** (dev tooling) | none |
| instance fields (`:45-51`) | **g47** (`Entry`) | none |
| `LiquidBuffer.cs` both fns (`:11-27`) | **g47** (`add_water` overflow + `end_pass` promotion, driver-2 census `d9445fc4`) | none |

### 5b. The g45-owned surfaces OUTSIDE Liquid.cs (no other lane claims them)

| Surface | Owner | Evidence |
| --- | --- | --- |
| `Player.ItemCheck_UseBuckets` (`Player.cs:46318-46504`) + `PutItemInInventoryFromItemUsage` (`:42141-42160`) + the five splashes | **g45** — `editing/buckets.rs` (`fa96cf32a`), wired before the melee arm; the union AND the g47 tip have NO buckets module (tree-verified both) | research/104 §1b; `bucket_use_tests` ×10 (+ the LIQUID-FINAL stack-consume additions §6) |
| The bucket item ROWS (all twelve kinds) | **already in main** — `itemdata.rs` generated rows verified against `Item.cs` (§6a census) | no graft needed |
| `WorldItem.TryFillBucket` (rain-fill, `WorldItem.cs:548-571`) | **g15** (`d43b490f3`, `task/W4-g15-dust-item-sites`, unmerged) — NOT g45's; do not duplicate | §6c |
| The down-arm/fall-arm witness modules + the shimmer-flow pin | **g45** (graft obligations, 5a rows) | `liquid_downarm_live`, `the_fall_arm_moves_the_post_evaporation_level`, `shimmer_falls_every_visit_like_water` |
| The widened (+6,+6) fixture helpers | **g45** (A2; research/70 step 5) | `basin`/`floor_world`/`shaft`/`put_as` on this branch |

### 5c. Coordination verdicts (g47-drivers / g46-evap vs g45)

- **No fn is double-owned.** g47 owns the machinery, g46 owns exactly two
  `Update` arms (evap + nudge) and the DelWater-tail record refinements the
  g47 canonical already absorbed, g45 owns the buckets module + four
  witness modules + three sim deltas (guards / down-arm re-read / fall-arm
  live read) + the extra wakes. f25 owns the gen driver only.
- **The g47 closeout trio dedupes**: `honey_falls_once_every_eleven_visits`
  / `quick_fall_bypasses_both_delay_gates` overlap the union's cadence pair
  (research/70 §11: keep one name each); the g45
  `shimmer_falls_every_visit_like_water` overlaps
  `shimmer_falls_every_visit_ungated` — same keep-one rule, the merger picks
  the g47 fixture + the merged assertions.
- **Both stale reads exist on the NEW base** (5a `Update`/`LiquidCheck`
  rows) — the g45 fixes are NOT union-only obligations; a re-base that
  takes the g47 tip verbatim re-introduces them. The A7 receipt re-flags
  this for g46d.