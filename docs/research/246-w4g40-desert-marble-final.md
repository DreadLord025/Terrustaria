# 246 — W4-g40 FINAL: the desert/marble-family declaration + compose maps (code-only)

Lane: `task/W4-g40-biome-marble-granite`, worktree TR-W4-g40, base
`a725acb04` (origin/main at branch time). This is the lane-closing record
after the marble/granite residual (P0-P4, closed 79361f00), the desert
entrance residual (research 222, renumbered from 94) and the desert chain
P1/P2/P3 (fossil-shatter 5d3f9c886, underground-desert MUSIC 1180f5b6d,
runtime desert-surface growth 3e6eddea1).

**Research number per registry-150:** the d54 registry
(`origin/task/W4-d54-dust-sites-world:docs/research/150-research-number-registry.md`)
consumed 151-236; 237+ free for late lanes. A fresh `git ls-tree
docs/research/` sweep of ALL 358 remote `task/W4-*` tips + all 358 local
tips (2026-11-21, this commit's fetch) finds 237-245 claimed (237 e23/g48,
238 c8/d42/g61 TRIPLE, 239 d42/f21/g48/g56 QUADRUPLE, 240 c42/d53/g58
TRIPLE, 241/242/243 g58, 244 f39/g48, 245 g48) and **246-255 free at every
tip** — this doc takes **246**, the first free number. The census doc's
registry-assigned renumber **94 → 222** is executed on this branch (§5).

## 1. The g40d-desert-chain scope census — what remains unclaimed

Method: full-tree `Select-String` over `decompiled/Terraria/**/*.cs` for
`Marble|Granite|ZoneMarble|ZoneGranite|nearMarble|nearGranite` (findstr
proved unreliable on the 2.3 MB Main.cs — it silently drops hits; every
negative below is a PowerShell re-run), cross-checked against the port
(`crates/`) and every pushed branch tip (`git grep` per tip for the
residual symbols).

### 1a. Marble/granite MUSIC — **NONE exists in vanilla**

`MusicID.cs` has NO Marble/Granite/UndergroundMarble/UndergroundGranite
constant (the full id list: Desert 21 :60, UndergroundDesert 61 :140,
OtherworldDesert 66 :150 are the only desert-family tracks).
`UpdateAudio_DecideOnNewMusic` (`Main.cs:12175-12988`) reads
`ZoneGranite`/`ZoneMarble` NOWHERE — the biome ladder's marble/granite
caves play the generic underground alternation (the `ugMusic` 4/31
random switch, `Main.cs:12830-12856`; the hallow 11/9 and the remix
underground bands are the only zone overrides there). `DrawBackground_PickUndergroundBackgroundStyle`
(`Main.cs:54033-54095`) likewise has NO marble/granite arm (the g40-P2
commit already recorded the desert half of this). **Verdict: no music/bg
arm to port — the marble/granite "biome music" question resolves to
vanilla-has-none, not a port gap.** The port's `scene.rs:817-818`
(`zone_granite` walls 184/180, `zone_marble` walls 183/178,
`SceneMetrics.cs:687-688`) already carries the zone flags verbatim; their
only C# consumers are `Player.UpdateBiomes` (`Player.cs:16695-16696`) and
the RGB `CommonConditions.UndergroundBiome.Marble/Granite`
(`CommonConditions.cs:157-158` — the RGB chroma system, out of scope per
research/38).

### 1b. Marble/granite runtime GROWTH/spread — **NONE exists in vanilla**

`WorldGen.Convert` (`:55085-55673`), `hardUpdateWorld` (`:69600`),
`UpdateWorld_GrassGrowth` (`:72000+`) and the `Grow*` family never read
tile 367/368 or walls 178/180/183/184 as a conversion SOURCE or growth
substrate. The 367/368 runtime reads are: `KillTile_GetItemDrops`
(`:64108-64116` — the granite 3086/marble 3081 drop rows, landed),
`GetTileMergeStyle` (`:38802-38807` — styles 6/5, tiledata), the
QuickCleanup 397-retype under sandstone walls (`:17917`, worldgen),
the pot-style rolls (`:18153/:18881-18889`, worldgen), the stalactite
tables (`:38448/:38477/:38582/:38611` — landed g40-P0), the
`tileSolid`/lighting sets (tiledata). **Verdict: no runtime growth arm —
marble/granite are static worldgen biomes in vanilla.**

### 1c. Marble/granite SPAWN — the `nearMarble`/`nearGranite` arms, UNCLAIMED, task-seeded

The one live runtime consumer family: `Spawner.SetSpawnFlagsForChosenTile`
(`NPC.cs:1050-1145`) sets `nearMarble`/`nearGranite` from the spawn tile /
player tile being 367/368 (`:1058-1073`), else two stride-sampled box
scans (the `Next(20,31)`/`Next(1,4)` spawn-box `:1076-1108` + the
`Next(30,61)`/`Next(3,7)` player-box `:1109-1141` — **8+ RNG draws per
scan**, on the spawn stream). The consumers are the deep-cavern ladder's
`:5019-5042` arms:

* `nearMarble && Next(4) != 0` → `Next(6) != 0 && !AnyNPCs(480) &&
  hardMode` → **480 Medusa**, else **481 GreekSkeleton (Hoplite)**;
* `nearGranite && Next(5) != 0` → `Next(6) != 0 && !AnyNPCs(483)` →
  **483 GraniteFlyer (Granite Elemental)**, else **482 GraniteGolem**.

Port status: the arms are ABSENT from `spawner.rs` (the deep-cavern
section jumps Tim `:5014` → the skeleton family; no `near_marble`/
`near_granite` symbol exists in any crate at any tip — `git grep` over
all 358 tips clean). The KINDS exist only on **d40**
(`origin/task/W4-d40-npc-kinds-421-490` tip 8fe1d6d31: Medusa/GreekSkeleton/
GraniteGolem cfed253d0, GraniteFlyer 55191f1a8 — SetDefaults + FindFrame +
bestiary, aiStyle machines UNWIRED per the style73 precedent). The
`SetSpawnFlagsForChosenTile` port itself is partial (the ANALYSIS
"scene-metrics zones" scope note, `docs/ANALYSIS.md:20021-20023`: the
per-candidate marble/granite proximity + dual-dungeons rescan "rides the
per-candidate work").

**Verdict: UNCLAIMED but kind-gated — TASK-SEED, not this slice.** Wiring
the arms before d40 merges would spawn unported kinds (the f42
consume-and-return-None shape would burn the `Next(4)`/`Next(6)` draws
with no spawn — a stream-visible deviation for zero behaviour). The
task-seed (for the catalog): *"the marble/granite spawn arms —
`SetSpawnFlagsForChosenTile`'s 367/368 proximity (`NPC.cs:1058-1141`, the
two stride-sampled box scans with their 8-draw ledgers) + the cavern
ladder's `:5019-5042` Medusa/Hoplite/GraniteFlyer/GraniteGolem arms;
prerequisite: d40's kinds merged (480-483) + the d32 aiStyle-3/91
machines; the arms slot between Tim `:5014` and the hardmode skeleton
family `:5043` in `spawn_npc_at`'s deep-cavern section."* The
underground-desert spawn ladder (`NPC.cs:1680-1762` — the
`AllowsUndergroundDesertEnemiesToSpawn` wall set + 510/513/524-533/537/
580/581/589 arms) is the same shape: kinds live on d41
(727578f2d/db12805be/c7159cf0b), f42 owns the sandstorm-surface half;
the UG ladder wiring stays with the d41-kind merge wave.

### 1d. The hive-smooth residual — UNCLAIMED → ported THIS round (§2)

Research 222's row (c): `DesertHive.PostPlacementEffect.Smooth`
(`DesertHive.cs:197,216-218,284,315,331,358`). `git grep
tile_smooth_slope_one` over all 358 tips in `desert.rs`: ZERO claimants
(the c31 hive-rewrite lane's tip carries the same deferred comment).
Ported verbatim below.

## 2. The hive-smooth slice (this round's port, WRITTEN NOT RUN)

`DesertHive.PlaceClusters` (`DesertHive.cs:206-222`): the carve fills a
`PostPlacementEffect[hive.Width, hive.Height]` grid (the hive inflated
20/20, `:209-210`), then a post-carve x-major walk runs
`Tile.SmoothSlope(i, j, applyToNeighbors: false)` on every Smooth-marked
cell (`:212-221`). The Smooth flag is set at exactly four sites:

* `:284` — the `num7 > 3.5` core clear, UNCONDITIONALLY (even where the
  cell was inactive);
* `:315` — the `num7 > 1.8` sandstone band, only where the
  `!flag || tile.active()` ResetToType(396) fires;
* `:331` — the `num7 > 0.7 || !flag` outer band, same gate, ResetToType(type);
* `:358` — the `num7 > 0.25` speckle arm, only on the
  `NextDouble() < num8` hit AND `tile.active()`.

The map write `:370` runs on EVERY visited cell (the `InWorld` continue
`:235` skips it — the cell keeps None). Port: `place_clusters_area` takes
a `&mut [bool]` (row-major `x * h + y`, the `[Width, Height]` flatten),
marks the four arms, and `full_desert` runs the walk on the g40-P1
`biomes::tile_smooth_slope_one` seam BEFORE `AddTileVariance` (the C#
`Place` order `:202-203`). **RNG-free** — no `Rng`/`FastRandom` draw added
or moved; the carve's ledgers are untouched. The tile bytes move (the
shell edges gain slope/half-brick states), so both hash pins are
stale-by-design where the hive fires; re-pin DEFERRED to the merge gate
(`world_hash.rs` notes updated). 3 witnesses WRITTEN NOT RUN
(`desert.rs`): the map's arm-exactness (every marked cell is a carved
shell tile or a Smooth-marked core clear), the slope delta (the flat
fixture starts unsloped, the hive edges end sloped), and the RNG-neutral
a==b replay (same-seed stream-end equality + the walk retypes nothing /
repaints no wall / moves no liquid).

## 3. Compose maps (the Lead's items 2+3) — measured hunk-by-hunk

All diffs measured `a725acb04..<tip>` (the shared base). Tips re-read at
this round's fetch: f42 `425d4ff9d`, g17 `f0fe1bbdd`, g22 `250488c34`,
g23 `945391e33`.

### 3a. f42-desert compose (f42 tip 425d4ff9d) — DISJOINT from g40's growth

f42's files: `spawner.rs` (the sandstorm spawn arm `NPC.cs:3949-4018` +
the post-sandstorm mummy tail `:4020-4035`, the `downed_boss1`/
`sandstorm_happening` ctx fields, `spawn_desert_ground`/`_flyer`),
`npc.rs`/`npc/data.rs` (the nine kinds 354-362), `npc_ai_desert.rs`,
`npc_ai_unicorn.rs` (546), `player.rs`/`world_sim/player.rs` (the
sandstorm run boost + DoubleJumpVisuals hold), `projectile*` (the Djinn
curse + sand ball), `audio.rs` (SFX_CUES +6 at `:139` — the desert-family
hit/death bank rows ONLY), `ambient_sound.rs`, `bestiary.rs`,
`render/entities.rs`, `assets.rs`, `main.rs`, `app/netglue.rs`.

g40's files: `worldgen/{biomes,desert,desert_entrances,mini_biomes,ocean,
pipeline,spider_gem_moss,world_hash}.rs`, `growth.rs` (the P3 runtime
desert-surface arm), `edit.rs`/`edit/kill.rs` (P1 fossil-shatter),
`world_sim/editing.rs` (the break_tile shatter drain), `audio.rs` (the
P2 MUSIC arm), `app/frame/world.rs` (the MusicEnv feed).

**Overlap: `audio.rs` ONLY, and DISJOINT hunks** — f42 touches `SFX_CUES`
(`@@ -139`), g40 touches `MusicEnv` (`@@ -2457`), `decide_new_music`
(`@@ -2778`) and the tests (`@@ -3334/:3504`). No shared line. The
runtime desert-surface GROWTH (g40 P3, `growth.rs:72227-72298` port) vs
the sandstorm SPAWN/event (f42) are different C# units
(`UpdateWorld_GrassGrowth` vs `SpawnAnNPC`/`Sandstorm`) — **disjoint by
construction**. Merger instruction: keep both; no adjudication needed.
The f42 mummy-tail kinds (78/79/630/80) resolve through f42's own
research/108-input dedupe table (d35 owns the mummies at 698-730) — g40
claims no kinds.

### 3b. g17-ShakeTree compose (g17 tip f0fe1bbdd) — three shared files, disjoint hunks

g17's palm/ash loot (`world_sim/editing/shake_loot.rs`, the
`WorldGen.cs:63163-63403` ladder) + the tree-FX dispatch
(`tree.rs`/`growth.rs`/`growth/decorations.rs`) vs g40's P1/P3:

* `world_sim/editing.rs`: g17 hunks at base `:23/:500/:1614/:1801/:2356/
  :5648/:5764` + tests; g40's single hunk at base `:5714` (the
  fossil-shatter drain inside `break_tile`). g17's `:5648` (+41 lines →
  ends ~5689) and `:5764` bracket g40's `:5714` WITHOUT touching it —
  **disjoint**; git's 3-way merge places all three.
* `edit/kill.rs`: g17 hunks at base `:659/:1639/:2360` (check_object /
  check_3x3_bottom / kill_tile_sound); g40 hunks at base `:160/:274/:282/
  :569/:971/:5065` (the TileKill fossil_shatters field + the cascade
  merges). **Disjoint.**
* `growth.rs`: g17 hunks at base `:699/:733/:826/:904/:922/:953/:972/
  :1672/:1739` + test renames (the GrowthFxCtx threading through
  update_world/overground/underground); g40 hunks at base `:110/:136/:148`
  (grow_cactus), `:775` (the overground desert arm), `:981` (the
  update_world helpers), `:1586` (tests). The `:775` vs `:733-748` and
  `:981` vs `:953-980` pairs are adjacent-but-disjoint (g17's `:972` hunk
  is +2 lines inside update_world's underground call; g40's `:981` hunk
  appends the runtime helpers after the function's close). **Textual
  compose clean, ONE semantic compose step:** g17 adds the
  `fx_ctx: &GrowthFxCtx` + `fx: &mut TreeFxRequests` parameters to
  `update_world_overground_tile`/`update_world` and rewrites every call
  site (its tests pass `&GrowthFxCtx::default(), &mut Vec::new()`); g40's
  new P3 witnesses call the OLD 9-arg signature — the merger appends the
  two FX arguments to g40's four new test call sites (the g17-test
  pattern) and threads `fx_ctx` through any g40 helper that calls
  `update_world_overground_tile` (none — the P3 helpers call the
  `grow_*`/`place_*` leaf fns directly). Both lanes' witnesses are
  written-not-run, so the merge gate is the first execution either way.
* The g17 `is_palm_oasis_tree` (beachDistance 380) reads the same
  `WorldGen.beachDistance` constant g40's P3 sea-oat/coral arms use —
  same value, no conflict.

### 3c. underground-music g22/g23 compose — WHICH LANE owns biome music

Census: g22 (`task/W4-g22-sounds-main-2`) and g23
(`task/W4-g23-sounds-main-3`) are the MENU-sounds lanes — g22's R3
close-out (250488c34) retires lane 14 with the direction-B merger
instruction (g22 base + g23's 4 commits, keep-g22/take-g23-unique). Both
tips carry `MusicEnv`/`decide_new_music` — but their DIFFS against the
base touch only: g22 = `SFX_CUES :66`, `Audio :1729`, `MusicEnv :2485`
(the `menu_micro_flags`/`menu_credits` fields), `decide_new_music :2671`
(the MENU title-track ladder `:12463-12489` — the shimmer-ocean 72 arm),
tests; g23 = `SFX_CUES :66/:278`, `Audio :1729/:1934`, tests (the star
pair + the menu cue seams). **NEITHER touches the biome ladder below
`:12644`** — g22's tip still carries the stale "UG-desert arm
constants-off" comment at its `:12750` equivalent.

**Adjudication: the BIOME-music ladder is g40's lane** (the P2 commit
1180f5b6d ports `Main.cs:12743-12746` — the UG-desert 61 arm between
graveyard 53 and surface desert 21, `MusicEnv.underground_desert` fed
from `scene_metrics.zone_underground_desert`). g22/g23 own the MENU
title-track ladder (`:12455-12494`) and the cue plumbing. **Keep-one:
g40's biome arm + g22's menu arms — no duplicate exists** (the lanes'
hunks are disjoint: g40 `MusicEnv @@ -2457` inserts `underground_desert`
between `graveyard` and `desert`; g22 `MusicEnv @@ -2485` appends
`menu_micro_flags`/`menu_credits` at the struct tail; g40
`decide_new_music @@ -2778` edits the biome ladder, g22 `@@ -2671` edits
the menu ladder ABOVE it). The one shared file region is
`app/frame/world.rs`'s `MusicEnv` literal: g40 adds
`underground_desert:` after `graveyard:` (`@@ -236`), g22 adds
`menu_micro_flags:`/`menu_credits:` after `menu_flags:` (`@@ -250`) —
disjoint lines, both keep. g23's audio.rs hunks are all in `Audio`
(the SFX player), not `MusicSystem` — disjoint from both. Merger
instruction: **take all three lanes' audio.rs/frame-world.rs hunks
verbatim; the g22-R3 direction-B fold applies WITHIN the menu family
only and does not touch g40's biome arm.** The remix-world underground
band (`Main.cs:12064/12089`, also 66/61) stays with the remix seed lane
(recorded in the P2 commit; no lane claims it yet).

## 4. The desert/marble-family declaration (lane close)

| C# unit | Port | State |
| --- | --- | --- |
| `MarbleBiome.cs` whole | `biomes.rs` P0-P4 (79361f00 + d05a082c8..) | **DONE** — closed, witnesses pinned |
| `GraniteBiome.cs` whole | `biomes.rs` P2 (64bcbe164) | **DONE** |
| `DesertBiome.cs` entrance family + gate | `desert_entrances.rs` + `desert.rs` (7bb5332e5/a8150064b) | **DONE** — research 222 |
| `DesertHive` PostPlacementEffect.Smooth | `desert.rs` (this round, §2) | **DONE** — the last unclaimed worldgen residual |
| `AttemptFossilShattering` (`WorldGen.cs:63484-63515`) | `edit/kill.rs` + `world_sim/editing.rs` (5d3f9c886) | **DONE** — desert chain P1 |
| UG-desert MUSIC (`Main.cs:12743-12746`) | `audio.rs` + `frame/world.rs` (1180f5b6d) | **DONE** — desert chain P2; the biome-music lane is g40's (§3c) |
| Runtime desert-surface growth (`WorldGen.cs:72227-72298`) + GrowCactus sand arm | `growth.rs` (3e6eddea1) | **DONE** — desert chain P3 |
| Marble/granite music + bg + runtime growth | — | **NONE IN VANILLA** (§1a/1b) — no port needed, census-closed |
| Marble/granite spawn arms (`NPC.cs:1058-1141/:5019-5042`) | — | **TASK-SEED** (§1c) — unclaimed, kind-gated on d40 (480-483) + d32 (aiStyle 3/91); NOT ported here (the consume-without-kind deviation) |
| UG-desert spawn ladder (`NPC.cs:1680-1762`) | — | **co-lane** — kinds on d41, the sandstorm-surface half on f42; wiring rides the d41 merge wave |
| Desert special-seed arms (remix/surfaceIsDesert/dualDungeons/notTheBees/skyblock) | — | documented deviations (desert.rs header) — the d107 seed lane |
| `MarblePileWithStatues` (`WorldGen.cs:10435-10540`, the DST marble piles `:16271-16302`) | `ocean.rs` header records the seed-gated-off half | **co-lane** — the dontStarve seed arm, d107 lane |
| CaveHouse granite/marble variants (`GraniteHouseBuilder.cs`/`MarbleHouseBuilder.cs`) | `mini_biomes.rs` shape-level (the documented M1.4-WGEN-8 approximation) | **follow-up, unclaimed** — research 222 row 30; the chest-chance variants resolve to the plain gold chest |
| Waterleaf PLANTING (`PlantAlch`/`PlaceSuitableHerbHere`) | — | the K-3d alch-seed skip-gate deviation (P3 commit records; not this lane) |
| d103 Oasis worldgen (`PlaceOasis :10707-10932`) | — | **d103 co-lane** (READ, not touched) |

**The lane is RETIRED with this doc**: every marble/granite/desert C#
unit is either landed, vanilla-absent, co-lane-claimed, or task-seeded
with its prerequisite named. No unclaimed in-scope residual remains.

## 5. The registry-150 renumber receipt (94 → 222)

Executed on this branch per the registry §3 g40 row: `git mv
94-w4g40-biome-residual-census.md 222-w4g40-biome-residual-census.md`,
the `# 222` header + a RENUMBERED provenance block, and every ref the
registry lists updated: `docs/research/INDEX.md` (the row),
`docs/ANALYSIS.md` (the census citation + the research-94 inline), plus
the three code-comment refs the registry's docs-only sweep missed
(`desert_entrances.rs:11/:797`, `world_hash.rs:245/:314`) and the
`anchors.txt:902` row tail. `git grep "research 94\|research/94\|94-w4g40"`
over crates/+docs/+anchors returns ZERO hits after (the renumber note's
own "from 94" mentions excepted). The key-94 winner (g41's
not-the-bees-host census) is untouched — this lane cannot touch other
branches.

## 6. Verification (code-only mode — no cargo/tests/rustfmt/graphify)

* C# re-reads: every claim above cites the decompiled line ranges read
  THIS round (MusicID.cs full list, Main.cs:12175-12988 +
  :54033-54095, NPC.cs:1050-1145/:4917-5116/:1680-1762,
  SceneMetrics.cs:670-714, DesertHive.cs:194-388, Tile.cs:792-865).
* Tip reads: f42/g17/g22/g23/d40/d41/c31/d129 re-fetched and `git
  grep`/`git diff`-measured at the tips named in §3; the 358-tip
  research-number sweep re-run at this round's fetch (§0).
* The port compiles-plausibly by inspection (the seam signatures
  re-read: `tile_smooth_slope_one(world, x, y)` pub(crate) in
  `biomes.rs:1532`, `Rect::left/right/top/bottom/w/h` pub(crate),
  `FastRandom` Copy); witnesses WRITTEN NOT RUN per the lane constraint.
* Hash pins: both stale-by-design notes updated in `world_hash.rs`
  (RNG-free slice — the ledgers do not move, the tile bytes do);
  re-pin DEFERRED to the merge gate.