# 120 — Water Chests keep-one ADDENDUM (arm-by-arm, W4-c33 sweep)

> Wave 4 reconcile · produced on `task/W4-c33-underwater-chests-oasis` @
> 3ecb8b08 · 2026-11-21 · code-only (no cargo/tests run; witnesses written
> not run). Reads: `origin/task/W4-g36-worldgen-missing-a` @ **6d46f6a5**
> `docs/research/88-worldgen-master-matrix.md` §2.8/§2.9 (the matrix tip did
> not advance past 6d46f6a5 at fetch time), `origin/task/W4-d103-worldgen-
> passes-3` @ **fe9a565f** (`water_chests.rs` 263 lines / `buried_chest.rs` /
> `chest_gen.rs` / `chest_stock.rs`), this branch @ **fd7fae0e → 3ecb8b08**
> (`structures.rs::pass_underwater_chests` + the `chest_gen.rs` stocking
> hook), and the C# `WorldGen.cs:17245-17355` read in ≤100-line windows.
> **g36c reads this doc.** It CONFIRMS matrix §2.9's KEEP with one new
> d103 deviation the matrix missed, and re-scopes the "carry c33's
> stocking-hook additions" instruction: after this sweep the carry is
> **c33's whole `chest_gen.rs` stocking block**, not just the 4425/4460 pair.

## 1. Recommendation (one line)

**KEEP d103 `9a935127`'s `water_chests.rs` pass + `f3929904`'s
`buried_chest.rs` core; DROP c33's `structures.rs::pass_underwater_chests`
and g37 B3; KEEP c33's `chest_gen.rs` stocking block wholesale (it is now
the superset — d103's copy still carries the phantom rows this sweep
deleted); fix d103's extraLiquid double-draw at the merge (§3 row A2).**
c33's Pyramids stay (matrix §2.16) — re-pointed onto d103's core.

## 2. The C# ground truth (`WorldGen.cs:17245-17355`)

The pass, verbatim structure:

- `:17247` `!Skyblock.denyAllGeneration` gate; `:17249` `Lang.gen[33]`.
- `:17250-17270` the `numOceanCaveTreasure` loop:
  - `:17252` `mainItemInChest = NextFromList({863,186,277,187,4404})`;
  - `:17253-17256` **if `SecretSeed.extraLiquid.Enabled` a SECOND draw
    `NextFromList({863,186,277,187,4404,268})` OVERWRITES it** — on an
    extraLiquid world BOTH draws consume the stream;
  - `:17258-17264` `num = 2.0; while (!flag && num < 50.0) { num += 0.1; }`
    widening box: `Next(X-(int)num, X+(int)num+1)`,
    `Next(Y-(int)num/2, Y+(int)num/2+1)`, the x-fold
    `num2 >= maxTilesX ? (int)(num2+num/2) : (int)(num2-num/2)`;
  - `:17265` `InWorld && liquid > 250 && (liquidType()==0 || notTheBees ||
    remixWorldGen)`;
  - `:17267` `AddBuriedChest(num2, num3, main, false, 17, trySlope:true, 0)`.
- `:17271-17277` scatter count `num5 = maxTilesX/4200.0`, extraLiquid ×3;
  `for (j = 0; j < 9.0*num5; j++)`.
- `:17287-17316` the main cycle: `maxValue = 10` (anniversary → **7**);
  `Next(maxValue)==0 → 863`; else `num4` 1→186 / 2→4404 / 3→277 / default
  (extraLiquid && num4==4 → 268, else 187 + `num4=0`).
- `:17317-17334` scatter arm 1: `Next(50, w-50)`, **`Next(1, UnderworldLayer)`
  first draw**; the inner re-draw loop is **UNBOUNDED** and re-draws
  `Next(50, w-50)`, **`Next(50, UnderworldLayer)`** (the 1→50 y-floor
  asymmetry); gate `liquid >= 250 && (water || notTheBees || remix)`;
  `AddBuriedChest(..., 17, trySlope: num8 < beachDistance || num8 >
  w - beachDistance, 0)`; the **outer** attempt counter `num7 > 10000`
  breaks (the cap is on placement attempts, NOT the water hunt).
- `:17335-17352` scatter arm 2: `Next(worldSurface, UnderworldLayer)` both
  draws; gate `liquid >= 250 && (water || notTheBees)` — **NO remix
  disjunct** (the C#'s own asymmetry); same beach-band trySlope + 10000 cap.

## 3. Arm-by-arm: d103 vs c33 vs C#

| # | arm (C# anchor) | d103 `water_chests.rs` | c33 `pass_underwater_chests` | verdict |
| --- | --- | --- | --- | --- |
| A1 | skyblock gate + Gen(33) (`:17247/:17249`) | step adapter + `progress.rs` Gen(33) | step adapter + Gen(33) | tie |
| A2 | treasure main: 5-list draw, extraLiquid **second** 6-list draw (`:17252-17256`) | **DEVIATION**: `if extra_liquid {6-list} else {5-list}` — ONE draw; the extraLiquid world loses the 5-list draw (stream shift) | 5-list only; extraLiquid deferred to d107 | d103 closer but **must fix at merge**: draw the 5-list, then overwrite with the 6-list when the flag is set |
| A3 | widening box + x-fold + `num<50` bound (`:17258-17264`) | verbatim (truncating casts, +0.1, fold) | verbatim | tie |
| A4 | treasure gate `liquid>250 && (water‖bees‖remix)` + InWorld (`:17265`) | verbatim (bounds after the tile read — `world.tile` None ≡ !InWorld) | classic water-only (`in_world` first); bees/remix deferred | d103 |
| A5 | style 17, trySlope:true place (`:17267`) | via full `buried_chest.rs` core | via the temple-generalised half | d103 core (matrix §2.8) |
| A6 | scatter count 9·w/4200, extraLiquid ×3 (`:17271-17277`) | verbatim | classic only | d103 |
| A7 | anniversary maxValue 7 (`:17289-17292`) | `flags.anniversary` | deferred d107 | d103 |
| A8 | 863 override + 186/4404/277/187 cycle + extraLiquid 268 slot (`:17293-17316`) | verbatim incl. the `num4==4` 268 arm | verbatim classic cycle | d103 |
| A9 | arm-1 first draw `Next(1,uw)`, retry `Next(50,uw)` (the 1→50 asymmetry, `:17319-17327`) | verbatim | verbatim | tie — **both keep it** (the matrix's "c33 1/50 asymmetry" is not a c33 exclusive) |
| A10 | arm-1 gate `liquid>=250 && (water‖bees‖remix)` (`:17323`) | verbatim | classic water-only | d103 |
| A11 | arm-2 gate **no remix** (`:17341`) | verbatim (comment cites the asymmetry) | classic water-only | d103 |
| A12 | beach-band trySlope (`:17328/:17346`) | verbatim (`BEACH_DISTANCE` 380) | verbatim (`ocean::BEACH_DISTANCE`) | tie |
| A13 | outer 10000-attempt cap (`:17330-17333/:17348-17351`) | **present** (`num7 > 10000` both arms) | present | tie — **matrix line 616 is stale**: d103's OUTER loops are capped exactly like the C#; only the INNER water re-draw is unbounded, and that is the C#'s own shape (both ports keep it verbatim; d103's doc notes the waterless-fixture hazard). g37's cap adds nothing d103 lacks |
| A14 | inner water hunt unbounded (`:17323-17327/:17341-17345`) | verbatim + doc note | verbatim | tie (keep verbatim; do NOT cap the inner loop — it would change the stream on sparse-water worlds) |
| A15 | sub-vanilla size guard | port-safety addition (`w<=100 ‖ uw<=50 ‖ surface>=uw` → return, no draws; W:1) | none (the C# `Next` would throw) | d103 (keep the guard) |
| A16 | GenVars threading (desertHiveHigh/Low, hellChest) into the core | yes | no (the half takes none) | d103 |
| A17 | per-place stocking seam | `stock_placed_chest(... p.style, p.kind, main ...)` | same seam | tie — **the seam is shared**; c33's hook arms fire on d103's events unchanged (style 17 / kind 21 / preset main) |
| A18 | vanilla slot (after `:17211` chests, before `:17356` spider) | pipeline `:1469` + slot witnesses | pipeline `:1309` + slot witnesses | tie |

**Score: d103 wins A2/A4/A6/A7/A8/A10/A11/A15/A16; ties elsewhere; c33 wins
nothing in the pass body.** Matrix §2.9's KEEP d103 stands.

## 4. The stocking hook — why the carry is now the WHOLE c33 block

The matrix (line 162) says "carry c33's stocking-hook additions (4425/4460
pair, style-17 main)". At fd7fae0e that was accurate. This sweep
(**3ecb8b08**) re-derived the surrounding block against
`WorldGen.cs:35871-37580` and found d103's `chest_gen.rs` copy (identical
to c33's pre-sweep base) carries **phantom rows with no C# counterpart**:

1. **flag5 is TWO INDEPENDENT 1/2 rolls** (`:36515-36525` mid,
   `:36845-36855` deep): `Next(2)==0 → 4425` then `Next(2)==0 → 4460` —
   both/either/neither. The old `ctx.mahogany`/`ctx.sky` "1/2 either-or
   4425/4460" rows in the mid/deep bands exist NOWHERE in the C# (the only
   4425/4460 writes in the whole stocking block are the two flag5 pairs).
   Deleted; the style-17-gated pair (c33 321c9b66) is the faithful arm.
2. **flag11 is MAIN-based** (`:36172-36175`: kind 21 && num9 != 0 && main
   ∈ {848, 857, 934}) — not a wall probe. The faithful Pyramid chamber
   walls are SandstoneBrick **34** (`Pyramid :27726-27747`), so the old
   `ctx.pyramid` (wall 187) gate missed the real chamber entirely. The
   `| flag11` disjunct (`:36227`) forces the **surface** band at any depth.
3. **preset mains skip the weapon ladders**: every band's weapon roll is
   the `num10 == 0` ELSE branch (`:36275/:36580/:36902/:37193`). The old
   port rolled a weapon for EVERY chest incl. water/jungle/sky/pyramid.
4. The mid band's common rows were re-derived to `:36668-36801` (the old
   copy had a phantom silver-bar 3-6 row, wrong gates/stacks on 52/bars/
   arrows/herbs/torches, and lacked the 166/4423 + 2350 rows); the flag12
   1/40 CreativeWings 4978 (`:36526-36532`) was missing.

d103's `chest_gen.rs` @ fe9a565f still has all of the above defects (its
grep shows no `chest_style == 17` arm, no flag5 pair, the phantom
mahogany/sky rows at :401-414/:488-493). **Merger: take c33's
`chest_gen.rs` stocking block wholesale; take d103's `water_chests.rs` +
`buried_chest.rs` + pipeline wiring.** The two compose through the shared
`ChestSiteEvent` seam with no signature changes (A17).

## 5. Witness union (the merge gate runs these)

| witness | source | covers |
| --- | --- | --- |
| `water_chests_size_guard_draws_nothing` | d103 W:1 | A15 (no draws under the guard) |
| `treasure_anchor_places_a_style_17_chest` | d103 W:1 | A3/A4/A5 |
| `water_chests_pass_is_deterministic_for_a_seed` | d103 W:1 | a==b replay |
| `the_water_chests_pass_stocks_the_basin` | c33 `structures.rs` W:1 | both hunts + style-17 frames + a==b — **port onto d103's signature** (the basin rig: 1200×600, water rows 100-140, sand 140+, anchors (300,120)/(900,120)) |
| `the_flag5_water_pair_rolls_two_independent_halves` | c33 3ecb8b08 | §4.1 — both/either/neither over 400 seeds × mid+deep, style-0 negative |
| `every_water_chest_main_stocks_with_the_flag5_pair` | c33 3ecb8b08 | all six mains 863/186/277/187/4404/268 stock slot 0 + a flag5 row |
| `the_flag11_pyramid_chest_takes_the_surface_rows_at_depth` | c33 3ecb8b08 | §4.2/§4.3 — surface consumables at y=340, no weapon, style-0 negative |
| `the_pyramid_chest_stocks_the_pharaoh_roll_and_the_robe` (updated: style 1, wall 34) | c33 3ecb8b08 | 848+866 pair, 857/934 arms |
| `the_sky_mid_band_rolls_the_creative_wings` | c33 3ecb8b08 | §4.4 flag12 odds |
| pipeline slot witnesses (`chests`+1, `spider caves`−1) | both | A18 |

**New witnesses the merger should add on the kept d103 body** (arms no
current witness pins): the A2 extraLiquid double-draw (stream-position
assertion: 5-list draw THEN 6-list draw), the A7 anniversary `Next(7)`
override, the A11 arm-2 no-remix gate (a remix+bees-less honey cell
satisfies arm 1 but not arm 2), the A8 268 cycle slot under extraLiquid.

## 6. Loot census closing notes (items 2-4 of the sweep)

- **Water-chest mains** — all six exist in `itemdata.rs`: 863
  WaterWalkingBoots, 186 BreathingReed, 187 Flipper, 277 Trident (proj
  47), 4404 FloatingTube, 268 DivingHelmet. The flag5 pair: 4425 SharkBait
  (pet, proj 774, buff 219), 4460 SandcastleBucket (tile 552). **No trident
  gap** — 277 is in both the NextFromList and the scatter cycle. The
  breathing-reed 186 leads the cycle. Table-complete.
- **Pyramid** — the site double-roll (`structures.rs :27852-27872`,
  `Next(3)` + 0→`Next(3)`) is faithful; the tenthAnniversary 0→1 arm and
  the flag11 anniversary 848+866 bonus pair (`:36267-36271`) stay with
  d107 (documented). Pharaoh'sMask 848 / Robe 866 / SandstorminaBottle 857
  / FlyingCarpet 934 all in `itemdata.rs`. The flag3 4346/4066 rows belong
  to the underground-desert family, NOT the pyramid (fixed 3ecb8b08).
- **Oasis remainder** — the Oasis pass (`:16243`) + `PlaceOasis`
  (`:10707-10932`) + the cacti-pass oasis block (`:21326-21363`) are d103's
  (90b7eee6, ledger live); c33's lily/cattail/bamboo deferral closed
  (fd7fae0e). **No unclaimed oasis worldgen arm remains.** There is no
  "mirage background" or "oasis music" in the C#: the mirage is the
  **HeatDistortion** filter (`SceneState.cs:79-80/:98/:145-160` — ported,
  `filters.rs` + `scene.rs::heat_zone_flags`), the oasis water falls ride
  `WaterfallID.DesertOasisWater` 23 ← `liquidAlpha[12]` (ported,
  `waterfall/mod.rs` (12,23)), and the oasis plays the **Desert 21 /
  UndergroundDesert 61** tracks. The 61 arm was constants-off in the port
  — the sweep's one real audio gap, landed **c7393225** (`MusicEnv.
  underground_desert` fed from the S1-L-1 wall zone, 61 ahead of 21).

## 7. Sweep commits (this branch)

| commit | item |
| --- | --- |
| c7393225 | (3) the underground-desert music 61 arm (the oasis family's real audio gap) |
| 3ecb8b08 | (2)+(4) flag11 pyramid loot shape + flag5 two-halves verify + phantom-row deletions + mid-band re-derivation + CreativeWings 4978 + 5 witnesses |
| (this commit) | (1) this ADDENDUM + the 12i ledger |
