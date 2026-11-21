# 96 — W4-g41 BIOME RETIRE: cross-lane receipts + the FINAL declaration

Status: RECEIPTS + FINAL DECLARATION (retire phase; code-only, no compiler).
Branch: `task/W4-g41-biome-spider-cave`, base closeout `2254f2e1`.
Every hash below was re-read from `origin/*` after `git fetch --prune` on
2026-11-21 (this session); every symbol claim is a `git grep`/`git show`
receipt against the named tip, not a doc-quote.

## 1. g47 liquid retire — the QuickWater contract receipt

Tips read: `origin/task/W4-g47-liquid-skip-settle` **`5c09a68b8`**
(R-12-5 closeout 4, research/70 §§9-12), closeout 2+3 **`4c81731bc`**
(+3 `liquid_sim` witnesses, code), closeout 1 **`e420e351e`**
(research/70 §8, the g46d re-fold obligation). Code base for the
QuickWater surfaces: `d222af697` + `d3a30ebf2` (unchanged by the
closeouts — `git diff dd66fa3a..5c09a68b8 -- crates` touches ONLY
`liquid.rs` `mod liquid_sim` @@ -2956 +94 lines: the three witness fns).

**Is the g41 3-seam-markers contract consumed?** PARTIALLY — the API
contract HOLDS verbatim; the ACTIVATION is still open, and its home has
moved from "g47d" to the g46 RE-FOLD union + the f25 gen-driver graft:

| Contract item (research/95 §addendum) | g47 `5c09a68b8` receipt | g46 union `9bc722e5f` receipt |
| --- | --- | --- |
| `quick_water(world, -1, -1)` call shape (`Liquid.cs:104`, `-2` = verbose) | `liquid.rs:1742` — same signature, `-1` → `3..=height-3` defaults, shimmer arms pre/post sweep | `liquid.rs:1882` — byte-identical body (re-read both) |
| `set_gen_settle_context(water_line, lava_line, remix)` | `:1582` | `:1722` |
| `set_gen_shimmer_position(Option<(i32,i32)>)` | `:1597` | `:1737` |
| `set_gen_dungeon_context(Option<GenDungeonContext>)` | `:1606` (struct `:45`, ORIGIN-wall read) | `:1746` |
| gen settle driver + f25 export surface | `gen_settle_loop` `:1698` + `update_world_tick` `:581` (`skip_count`, ReInit reset) | `:1838` + `:662` (`skip_count` `:114`, reset `:323`) |
| boulder window subsumed by `tiles_ignore_water(true)` | `BOULDER_SOLIDS` `:1571` (nine kinds, `:2141` fold) | carried (RE-FOLD step 3/4) |

- The three seam markers stand UNTOUCHED in `finish_remix.rs` (`:379`
  SEAM 1/3, `:405` SEAM 2/3 + the while-flag note, `:445` SEAM 3/3).
  g47's research/70 names NO g41/finish_remix consumer (grep: zero hits)
  — the contract is one-directional: g41 waits on the driver, correctly.
- **The activation owner is now the g46 union**: research/70 §9 item 1
  moves step-1's base to `dd66fa3a`; the union's RE-FOLD (below) has
  already re-based onto the g47 machinery. research/95 §addendum item 3
  (gen-local vs `LiquidSystem` instance) resolves at the f25 dedupe
  (§9 item 2: DELETE f25's private copies, call the LiquidSystem methods).
- Witness note: the closeout trio (`honey_falls_once_every_eleven_visits`,
  `shimmer_falls_every_visit_ungated`, `quick_fall_bypasses_both_delay_gates`)
  exists ONLY on g47 — the union carries the honey + quickFall twins
  (`liquid.rs:4470`/`:4489`) but NOT `shimmer_falls_every_visit_ungated`
  (zero grep hits). The merger must carry the g47-only shimmer witness
  per research/70 §11 (20 g47-only pins + 3 closeout, −2 pair dedupes).

### g46d re-base status — DONE (supersedes the §8 stale-union read)

`origin/task/W4-g46-liquid-evaporation` tip **`9bc722e5f`** (10:00). The
RE-FOLD landed as four code steps + docs, AFTER g47's §8 obligation was
written (08:36): `e7626b72d` (step 1: base swap to g47 `95e9c1ef`
WHOLESALE + evap/nudge/extra-wake re-graft), `f588d53c5` (step 2: g47c
`d3a30ebf` — reduced_max_liquids, GenDungeonContext + ORIGIN-wall re-type,
ShimmerRemoveWater both arms, the loot seam), `16fd6ad78` (step 3: g45b
AddWater guards + the `&mut World` ripple), `d95911ce1` (step 4: the four
R-12-5 drivers — `update_world_tick`, buffer pins, the f25 export surface
+ `gen_settle_loop`, the e21 pump-wake compose), `9bc722e5f` (docs:
research/69 SUPERSEDED-BY-70, the FINAL UNION-CONTRACT, 102
written-not-run witnesses, anchors reconciled). Receipt: every §8
missing-surface symbol now greps PRESENT on the union tip (table above).
Residual for the liquid merger: the closeout trio carry + the §11 pair
dedupes; the g41 side owes NOTHING further until activation.

<!-- NEXT-PHASES -->
