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

## 2. d36 hand-off — NOT porting yet; activation hunk PUBLISHED anyway

Tips read: `origin/task/W4-d36-npc-kinds-141-210` **`bcf4ac551`** (09:30
supersede-note), `0eb4fee95` (08:20 census, research/109), `65e481f96`
(06:03 CLAIM). `origin/task/W4-d36x-npc-kinds-141-210-exec` tip ==
`0eb4fee95` (same census, zero code). Receipt: `git grep BlackRecluse`
over both tips' `crates/` hits ONLY the base tables (`lang_tables.rs`
`:240`/`:315`, itemdata, creative_sacrifices) — **zero kind code**. The
d36 lane is at CENSUS + PLAN; E4-762-band resume is ACTIVE (research/109
supersede-note: 21-kind skip list → 28 kinds to port).

**Band caveat (read before wiring ids):** d36's own supersede-note still
says E4 **762-811** (28 kinds at 762-789), but d37's later reconcile
`a341daa12` (10:02, research/108 **R7**) re-pins d36 to **767-816**
(R5 Arapaima → 762, R6 d120 towns → 763-766 took the low holes). The
activation hunk below keys on `NpcKind` NAMES, never protocol ids — it
survives either re-pin. Co-lane kind owners at the read tips:

| Kind | Owner tip | Receipt |
| --- | --- | --- |
| 164 WallCreeper (+165 wall form) | g8 `82c93cecf` (batch 2 `0a4b76e3f`) | `npc.rs:1163`, `data.rs:607` proto 366 (→745-761 per E8), aiStyle 3 `:2630`, 50×20 `:3202`, cs 164 `:3349`, hp 80 `:3944`, dmg 30 `:4358`, NPCHit29/NPCDeath32 |
| 163 BlackRecluse | d36 (unported) | research/109 supersede-note 28-kind list; SetDefaults `NPC.cs:11030-11043` re-read: 50×20 aiStyle 3 dmg 90 def 40 350 HP kb 0.25 value 500 slots 0.5 |
| 238 BlackRecluseWall | d37 `a341daa12` (band 354-399 kept) | `npc.rs:1167`, `data.rs:613` proto 370, hp 350 `:4009` |

### The activation hunk (publish — apply when BOTH kinds exist on main)

`crates/tr-app/src/spawner.rs`, the pool pick at `:1142-1154` (the
consume-and-None arms; the Stylist sub-arm above is already live). The
C# is TERMINAL either way (`NPC.cs:1671-1678`), so both replacements
keep the `return`:

```rust
        // `:1671-1678` — the pool pick: hardmode `Next(10) != 0` → the
        // Black Recluse 163, else the Wall Creeper 164 (d36 + g8 kinds).
        if ctx.flags.hard_mode && rng.next_max(10) != 0 {
            return spawn_spider_walker(ctx, x, y / 16, NpcKind::BlackRecluse);
        }
        return spawn_spider_walker(ctx, x, y / 16, NpcKind::WallCreeper);
```

with the placement helper beside `set_spawn_spider_flag` (the C#
`SpawnNPC(spawnTileX * 16 + 8, spawnTileY * 16, type)` is the **Bottom**
placement — `NPC.cs:5238` → `NewNPC` `:80750`, `nPC.Bottom = (X, Y)`
`:80770`; NOT `spawn_cave_walker` (`npc.rs:2705`), which re-scans
`world.surface_of` (the WORLD surface column — wrong row for a cavern
pocket) and adds a `liquid > 128` veto the C# arm does not have. The
`spawn_caster` body (`npc.rs:2731`) is the same Bottom math but is named
for casters; a dedicated helper keeps the aiStyle-3 semantics explicit):

```rust
/// `NPC.SpawnNPC`'s Bottom placement (`NPC.cs:5238`/`:80770`) for the
/// spider pool's aiStyle-3 walkers — feet on the found tile's top edge,
/// X centred; the rect test is the port's collision refusal (the
/// Stylist arm's `spawn_caster` precedent).
fn spawn_spider_walker(ctx: &SpawnContext, x: i32, y_tile: i32, kind: NpcKind) -> Option<Npc> {
    Npc::spawn_caster(ctx.world, x, y_tile, kind)
}
```

(The helper is a thin alias ON PURPOSE: if d36 lands a Bottom-placement
walker ctor with the aiStyle-3 facing/idle conventions, point the alias
at it — one line, no stream change; the placement draws NOTHING. This
corrects research/93 §"d36 cross-note"'s earlier `spawn_cave_walker`
recipe — that ctor's surface re-scan is wrong for this arm.)
Witness swap: the two consume-and-None pins in `spawner.rs`'s W4-g41
block (`:4777` region) flip to kind/position assertions; the draw-order
pins (flag draws before `GetZombieSettings`, the `Next(8)`/`Next(10)`
order) HOLD unchanged — the hunk adds no draws. The wall-stick transform
(`:58875-58905`), the expert web-shot (`:58855-58874`) and `FindFrame`
(`:73083-73111`/`:73227-73267`) ride the KINDS (d36/g8/d37), not this arm.

## 3. research/88 stalactite keep-one — d106 still owns its half

Tips read: matrix `origin/task/W4-g36-worldgen-missing-a` **`bf7b6bf5b`**
(09:42; §3.2 unchanged from `6d46f6a54` — the gap-fill touched rows
105/109 + the merge order, NOT §3.2), d106
`origin/task/W4-d106-worldgen-passes-6` **`e2c2609c5`** (R5 tip, R4 =
`29faa0995`).

**d106-tip still owns its half — VERIFIED:**
- `pass_stalac` (`spider_gem_moss.rs:774`) + the `Stalac` pipeline slot
  (`pipeline.rs:1152`/`:1428`, vanilla `:22038` after Water Plants) on
  ONE name-seeded `Streams::pass_unified("Stalac")` (`pipeline.rs:235`,
  called `:2800`).
- `place_tight` `pub(crate)`, generic over `chest_stock::WorldGenRand`
  (`:402-404`); `check_stalactite` `:528` / `update_stalagtite_style`
  `:703` same trait bound. d106's `place_tight` does NOT call the
  validator (caller-runs convention, `:396-401` doc) — its Stalac sites
  call `check_stalactite` explicitly (`:826-876`), its spider/webs sites
  don't (`:1062`/`:1592`), keeping ITS spider output byte-identical to
  the pre-validator base.
- Matrix §3.2 + rows 67/107 unchanged at `bf7b6bf5b`: KEEP g41's chain
  body + d106's generalisation + pass_unified; single home
  `spider_gem_moss.rs`; DROP g36's local shimmer chain (exception clause
  stands); g40's table arms dedupe onto g41's table.

**The g41 half (this branch) — receipt:** the chain at
`spider_gem_moss.rs:441-660` (`invalid_tile_for_piles_or_speleothems`
`:441`, `get_stalagtite_style` `:450`, `get_desired_stalagtite_style`
`:472` with the spider style-11 override `:498` + the hive small-sac arm
`:513`, `update_stalagtite_style` `:524`, `check_stalactite` `:566`),
wired INSIDE `place_tight`'s tail (`:422-424`); the moss set is the
GENERATED `tiledata::TILE_MOSS` (`:106`, `sets.rs:44` — 627 included,
`Main.cs:7031` re-read: 179/381/534/536/539/625/627/180/181/182/183)
vs d106's local `MOSS_TILES`+`NEON_MOSS`+381 lists (same 11 members —
dedupe onto the generated set).

### FINAL INSTRUCTION (the gate's keep-one, exact)

1. KEEP this branch's `spider_gem_moss.rs` chain body + frame table +
   the 14 egg-sac witnesses + the `place_tight`-internal validator call.
2. GRAFT d106 R4's signatures over it: `place_tight`/`check_stalactite`/
   `update_stalagtite_style` take `rng: &mut impl
   super::chest_stock::WorldGenRand` (the trait + both impls already
   exist on this branch, `chest_stock.rs:185-213` — Xorshift64 AND
   UnifiedRandom); swap the two `rng.range(0, n)` draws in `place_tight`
   for `rng.next(n)` (same semantics under both impls); make
   `place_tight`/`check_stalactite` `pub(crate)` for the Stalac pass.
   Do NOT adopt d106's caller-runs convention — keep the validator
   inside `place_tight` (g41's spider output already includes it; the
   d106 Stalac sites then DROP their explicit `check_stalactite` calls
   to avoid double-validation, which would double the `Next(3)` mismatch
   draws — d106's own sites were written against ITS unvalidated base).
3. KEEP d106's `pass_stalac` + slot + `pass_unified("Stalac")` stream
   wholesale; its gem-tree scatter calls g42's trees per matrix row 107.
4. DROP g36's local `shimmer.rs` chain; rewire its PlaceTight tail onto
   the canonical `place_tight` (draw-schedule exception clause: if the
   shimmer island arm's schedule differs, keep the local copy + record).
5. Moss reads: one set — `tiledata::TILE_MOSS` (generated, never
   hand-edit); delete d106's local lists at the graft.
6. Re-pin AFTER the graft, sequentially per research/93 §FINAL STATUS
   step 2 (d106 R1-R5 pins move by design; the g41 egg-sac pins move at
   the same gate). The keep-one is RNG-neutral for the spider pass by
   construction (same draws, same order — d106 proved byte-identity).

## 4. FINAL DECLARATION — SPIDER + NTB + FINISH family

**ALL G41 PASSES + HOSTS + POOLS LANDED** on
`task/W4-g41-biome-spider-cave` (closeout `2254f2e1` + this retire phase):

| Piece | Landing commit | State |
| --- | --- | --- |
| Spider-cave egg-sac validator chain + decor arms + webs/DST residuals | `e5e2d783..ee937411` | landed; 14 witnesses NOT run; pins stale-by-design |
| NTB host (`NotTheBees` + `FinishNotTheBees` + `Spread.Wall2` + `OverGrownDungeon` + `spider_wall_remover` WIRED) | `78b0cd152` + `5ca77ca8a` + `c1c527e12` | landed; 17 witnesses NOT run; flag-gated, pins UNCHANGED |
| FinishNoTraps host | `d267d43fd` + `74d3eb3d6` | landed; 7 witnesses NOT run; flag-gated |
| FinishRemixWorld host | `6300aab69` + `74d3eb3d6` | landed; 11 witnesses NOT run; flag-gated; QuickWater ×3 no-op seams marked |
| QuickWater wire-contract addendum | `3b626b83c` (+ research/95 §addendum) | contract pinned; §1 receipt: API holds at g47 `5c09a68b8` + union `9bc722e5f` |
| Ambience census | `64e1f9cad` | NOTHING TO PORT (no spider track/gate/cue in 1.4.5.8) |
| Spawn pool (wall-62 arm + spawnSpider latch + CheckToSpawnSpider + Stylist sub-arm; stand-in RETIRED) | `c55239723` | landed; 9 witnesses NOT run; 163/164 consume-and-None (hunk §2) |
| PlacePot remix centre veto (last research/93 MISSING row) | `d18923543` | landed; 1 witness NOT run; flag-gated |
| Closeout docs (deferral final status + 6-step merger checklist + INDEX) | `2254f2e14` | landed |

59 written-NOT-run witnesses total (14+17+7+11+1+9). Both classic pins
(`SEED42_512 = 124297949`, `DEFAULT_WORLD = 3647283075`) stale-by-design,
re-pin at the gate; NOTHING in the FINAL phase moved them.

### The merger 6-step (research/93 §FINAL STATUS, refreshed by these receipts)

1. **Stalactite keep-one** — §3's FINAL INSTRUCTION (g41 body + d106
   generalisation, matrix §3.2 at `bf7b6bf5b`; d106 half verified alive).
2. **Sequential re-pins** with the world-changing lanes (d106 R1-R5,
   g36/g37/g38/g40/g42/g43/g44, f24/f25, d103/d104); each lane's a==b
   witness green on the failing run before its re-pin is recorded.
3. **Run the 59 witnesses** (the full-pipeline flag-world a==b halves
   included); gate command `cargo test --workspace -- --skip
   migrates_version_2_kinds_to_item_ids`, judge by `test result:` lines.
4. **QuickWater activation** — now against the g46 RE-FOLD union
   (`9bc722e5f`+) per §1: arm the gen context ONCE at the driver head,
   swap the three seams for `quick_water(world, -1, -1)`, restore the
   spawn-box `while flag` shape, re-derive the REMIX pins ONLY; carry
   the g47-only closeout shimmer witness + the §11 dedupes.
5. **d36 kinds hand-off** — §2's published hunk when 163 (d36) + 164
   (g8) both exist on main; ids ride `NpcKind` names (R7 band 767-816
   vs d36's E4 note 762-811 — the hunk is id-agnostic).
6. **Anchors canary** — the five g41 rows (`anchors.txt:899-903`:
   `fn check_stalactite`, `finish_not_the_bees`, `finish_no_traps`,
   `finish_remix_world`, `fn set_spawn_spider_flag`) must all survive;
   a failed marker = lossy merge, recover the arm, never delete the row.

### research/95 addendum — the receipt cross-ref

The addendum's five items re-verified at the retire tips (research/96
§1's table is the symbol-level receipt): items 1-2 HOLD verbatim on
both g47 `5c09a68b8` and the union `9bc722e5f`; item 3's instance
question resolves at the f25 dedupe (research/70 §9 item 2 — DELETE the
gen-local copies); item 4's while-flag shape + item 5's REMIX-only
re-pin are unchanged obligations of the activation commit. The three
seam comments in `finish_remix.rs` (`:379`/`:405`/`:445`) stay as the
call-site markers until that commit.

**Retire verdict:** the g41 lane has NO remaining unclaimed work. Every
residual is a NAMED cross-lane obligation with an owner and a published
contract (liquid activation → g46-union/f25 graft; kinds → d36/g8/d37;
keep-one → the gate). Branch ready for the merge gate.
