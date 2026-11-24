# 259 — Doc-sweep lane 39: worldgen #37-45 residuals (W4-d105 sweep phase) — SEALED

> RENAME CHAIN: 129 → 259 (registry-150 §6.1/§6.3 round-2 assignment; 129 =
> W4-f28 `129-f28-deferred-families.md` wins). Executed at the
> integration/batch-O merge (merger dedupe per §9/§10); the
> 12e-worldgen-passes-5 ref re-pointed.

> Wave 4 · 2026-11-21 · code-only mode (no cargo/tests/rustfmt/graphify on
> this branch) · branch `task/W4-d105-worldgen-passes-5` (9/9 DONE at
> `241ffa39`, this sweep continues the SAME branch).
> **SEALED at the RETIRE phase** (see the RETIRE section at the tail:
> the g42/g58/g34/f35/f12 compose census, the corrected Chillet ownership,
> the two task-seeds, the wg-lane5-family declaration pointer).
> Scope: the six residuals the lane-39 dispatch named over the landed
> #37-45 passes — guide starter-NPC spawn, dye-plant variants, herb growth
> tick, ChilletEggs chain, GlowTulips glow, sunflower happiness. Each was
> re-censused against the C# source; the REAL gaps were ported (3 commits),
> the rest are recorded here as verified-present or cross-lane.

## Method

- C# read in ≤100-line windows, direct file paths only (the `decompiled/`
  junction is never recursed). Every verdict below carries a `file:line`
  citation on both sides.
- Co-lane claims read from the pushed branches: `W4-d102`/`W4-d106`
  (growth co-lanes), `W4-g45`/`W4-f6` (mount co-lanes), `W4-g18`/`W4-e8`
  (AddLight), `W4-g58`/`W4-f12` (happiness/buff-frame).
- Witnesses WRITTEN NOT RUN (code-only). The herb RNG fixtures were
  hand-derived through a Python replay of the .NET Knuth-subtractive
  `UnifiedRandom` (the exact algorithm `unified_random.rs` ports) — the
  three existing cue tests bloom at the same ticks (17/5/82) with the
  fixed arm, and the new seeds (35/102/1) pin the first-draw outcomes.

## 1. Guide starter-NPC spawn — VERIFIED PRESENT (no gap)

`WorldGen.cs:19685-19897` (`SpawnStarterNPCs`) vs
`worldgen/starter_npcs.rs::pass_starter_npcs` (landed `c499a767`):

- **Full arm census**: the anniversary first spawn (infected Dryad 20 /
  classic Guide 22 / bees SkeletonMerchant 228 / remix TaxCollector 441,
  `:19690-19725`), the +2 Mechanic 124 / Steampunker 178 (`:19729-19750`),
  the −2 Angler 17 / Princess 663 (`:19751-19774`), the +4 Painter 227 /
  PartyGirl 208 (`:19775-19797`), the −4 rainbow slime 681 / purple slime
  680 / white bunny 656 (`:19798-19820`), the non-anniversary infected
  Dryad (`:19823-19835`), vampire 633 (`:19836-19843`, no port flag —
  recorded deviation), remix TaxCollector (`:19844-19852`), drunk+starve
  Mechanic (`:19853-19861`), bees Angler (`:19862-19870`), good
  Demolitionist 38 (`:19871-19879`), drunk PartyGirl (`:19880-19888`),
  classic Guide (`:19889-19896`). All present, same order, same
  home/direction/homeless writes.
- **Starter-house frame check**: `GetAdjustedFloorPosition`
  (`:24175-24198`) + `Scan3By3` (`:24200-24229`) — the 3×3 clear box +
  solid floor row, the up-while-blocked / down-while-floorless scans with
  the ±10 spawn-row clamps, the `(num+1, num2+2)` return. Ported verbatim
  (`adjusted_floor_position`, starter_npcs.rs:75-121); the scan-at-current-
  row-then-step order and the boundary-on-stepped-row quirk are kept.
- **Guide name pick**: `NPC.NewNPC` → `GiveTownUniqueDataToNPCsThatNeedIt`
  (`NPC.cs:80839-80856`) → `getNewNPCNameInner` (`:7281-7325`) — one
  `Language.RandomFromCategory(<Kind>Names, genRand)` per town spawn. The
  ledger carries the raw draw word; `App::starter_npc_name`
  (app/mod.rs:1291-1338) resolves it through the kind's category with
  `roll % size` — the full 38-row category table matches `:7285-7322`
  (incl. the pet/slime rows 637/638/656/670/678-684). The anniversary
  fixed names (`GuideNames.Andrew` etc.) ride `given_name_key`.
- **Starter chest / copper tools**: NOT part of `SpawnStarterNPCs`. The
  copper set is the PLAYER start inventory (`Player.cs:54356-54361`:
  3507 CopperShortsword / 3509 CopperPickaxe / 3506 CopperAxe) — already
  ported (app/mod.rs:2957-2976, flow.rs:2519, saveglue.rs:1014). The
  buried-chest copper/iron bar roll (`WorldGen.cs:36394-36408`,
  `GenVars.copperBar`) is the chest-stock lane's, not the guide's.
- Recorded deviations (module header, unchanged): the unported
  `TownNPCProfiles` variation rolls, the vampire-seed flag, the
  `unlockedMerchantSpawn`/`unlockedDemolitionistSpawn`/`boughtBunny`/
  `unlockedSlimePurpleSpawn` World fields, the bestiary-decode skip for
  the unported kinds (178/656/680/681).

**Verdict: no gap.** The #37 port is faithful; the dispatch's "starter
chest items / copper tools" belong to other lanes (player start inventory
+ chest stock), both already present.

## 2. Dye-plant variants — variants PRESENT; runtime remix/veto gaps PORTED

`WorldGen.plantDye` (`WorldGen.cs:71286-71455`) + `PlaceDye`
(`:53988-54064`). The plantDye "table" is the soil→style dispatch, not a
named-variant list:

| soil / band | style | C# | gen port | runtime port |
| --- | --- | --- | --- | --- |
| grass 2/109 surface | 3 green / 4 orange (1/4) | :71327-71337 | ✓ | ✓ |
| jungle 60 surface | 2 teal / 4 orange (1/2) | :71338-71348 | ✓ | ✓ |
| sand 53 submerged surface | 2 teal | :71349-71352 | ✓ | ✓ |
| cactus 80 surface | 6 lime (evil-box gate) | :71353-71376 | ✓ | ✓ |
| jungle 60 underground | 2 / 0 / 1 cascade | :71406-71420 | ✓ | ✓ |
| dirt 0/stone 1/mud 59 underground | 0 / 1 (1/2) | :71421-71431 | ✓ | ✓ |
| submerged underground | 5 blue | :71432-71435 | ✓ | ✓ |
| hanging dirt 0 | 7 | :71437-71454 | ✓ | ✓ |
| exotic grass/moss/dirt | 8-11 (`Next(8,12)`, frameX 272..374) | :71314-71326 / :71398-71405 | n/a (gen never exotic) | ✓ |

The "MARIGOLD / sky-blue-flowers" the dispatch named are the style-4
orange (marigold) and style-5 blue dye plants — both present in both
twins. The strange plants (styles 8-11) are the exotic arm, already
ported in `growth/decorations.rs::plant_dye`/`place_dye` (the `8|9|10|11`
dry-floor arm, decorations.rs:426). **No missing variant.**

**REAL gaps found + ported (`52978903f`)** — the runtime growth twin
(`growth/decorations.rs::plant_dye`, the `UpdateWorld` `Main.rand` stream)
was missing two remix halves and mis-vetoed shimmer:

1. **Band-1 remix bypass** (`:71298`): C# `(j < worldSurface ||
   remixWorldGen) && freeAbove`. The port had only `j < world_surface`.
   On a remix world the surface arm runs at any depth. Fixed:
   `let remix = world.seeds.contains(SeedFlags::REMIX)` +
   `((j as f64) < world.world_surface || remix)`.
2. **Band-2 gate remix half** (`:71378-71381`): C#
   `if (!(j >= worldSurface) && !remixWorldGen || j >= UnderworldLayer)
   return;`. The port had `j < world_surface || j >= height-200` (no
   remix). Fixed: `((j as f64) < world.world_surface && !remix) || j >=
   world.height - 200`.
3. **Exotic liquid veto** (`:71318-71321`): C# `liquid > 0 && (lava() ||
   honey())` — water AND shimmer pass. The port had `!= Water`, which
   also vetoed shimmer. Fixed to `matches!(Lava | Honey)`.

The gen-time twin (`worldgen/surface.rs::plant_dye`, landed `dd43734a`)
already carried all three correctly — the gap was runtime-only. The
`free_above` cut-list (`3|51|61|73|74|184|697|703`) and the density-box
frame split (`<272` classic / `>=272` exotic, the unclamped hanging scan
counts ANY 227) were re-verified faithful in both twins.

Witnesses (2, written not run): `w4_d105_the_runtime_dye_carries_the_remix_bypass`
(below-surface grass plants on remix / refuses on classic; the exotic
draw-count parity above-surface) and
`w4_d105_the_exotic_veto_is_lava_and_honey_only` (shimmer passes,
honey/lava veto).

## 3. Herb GROWTH tick — inverted gate + missing tail PORTED

`WorldGen.GrowAlch` (`WorldGen.cs:45777-45899`) vs
`growth/herbs.rs::grow_alch` (the S1-K-3d split). The daybloom/moonglow
bloom-time rules (the cue arms) were present; TWO real gaps:

1. **The generic 1/50 arm's frame-108 gate was INVERTED**
   (`:45798-45811`). C#: `if (Next(50)==0) { if (frameX==108) { if
   (Next(2)==0) flag=true; } else flag=true; }` — only the fireblossom
   stem (frameX 108) needs the extra `Next(2)`; every other frame blooms
   outright. The port had `frame_x == 108 || rng.next_max(2) == 0`, which
   bloomed 108 unconditionally AND made every other frame draw the extra
   `Next(2)` (a stream divergence on every non-108 armed tick). Fixed to
   `frame_x != 108 || rng.next_max(2) == 0` — the `||` short-circuit keeps
   the `Next(2)` draw on frame 108 only, matching the source order.
2. **The fireblossom maturity tail was MISSING** (`:45890-45898`):
   `else if (frameX==108 && type==83 && Next(30)==0) type=84;`. Added as
   the else-if chain's final arm (a blooming fireblossom matures to 84 at
   1/30). The else-if shape is load-bearing: the stage-3 flash arm
   (`:45868`, frameX 36) draws `Next(3)` on EVERY frame-36 cell (a 0 falls
   out of the whole chain), so the port must NOT early-return on a failed
   `Next(3)` — the tail is the `else if`, keeping the stream identical.

The cue arms (day/frame0, night/frame18, rain/frame72, below-surface/
frame36, near-underworld/frame90, each `Next(50)`) and the liquid-kill
(`:45783-45795`, lava keeps stage 5 / water keeps stages 1+4) were
re-verified faithful. `MatureTheHerbPlants` (the GEN-time sweep,
`:45748-45775`) is a separate function already ported in
`worldgen/surface.rs` — not part of this runtime gap.

**Co-lane claims**: d102/d106's pushed docs (`12a`/`12f`) claim only their
own pass ranges (d102 #10-18, d106 #46+); neither claims the runtime
`GrowAlch`. The growth tick is the S1-K-3d/M7-GROWTH lane's, unclaimed by
the worldgen co-lanes — this sweep ports the residual.

Witnesses (2, written not run, hand-derived .NET seeds):
`w4_d105_the_generic_bloom_gate_is_frame_108_only` (seed 35 first
`Next(50)`=0 + `Next(2)`=0 → frame 0 and 108 both bloom; seed 102
`Next(2)`=1 → 108 holds; the third-raw-draw divergence pins the draw
order) and `w4_d105_the_fireblossom_tail_matures_at_one_in_thirty`
(seed 35 `Next(30)`=0 fires, seed 1 skips; the 82-sprout and frame-36
gates). The 3 existing cue tests replay identically (Python Knuth replay:
blooms at ticks 17/5/82, unchanged).

## 4. ChilletEggs — gen + mount + recipe PRESENT; bag-open chain CROSS-LANE

The full chain censused:

- **Egg tile (752) placement**: `PlaceChilletEggs` (`WorldGen.cs:45652-
  45678`, 6/9/12 by size, 10000 budget) → `TryPlacingChilletEgg`
  (`:45700-45723`, 5 jitter attempts) → `HasValidGroundForChilletEggBelowSpot`
  (`:45725-45746`) + `NoNearbyChilletEggs` (`:45680-45698`, ±20 box) →
  `PlaceTile(752)` → `Place2x2` (`:51330-51423`). Ported in
  `worldgen/surface.rs::place_chillet_eggs`/`try_placing_chillet_egg`/
  `place_2x2` (landed `dd43734a`). ✓
- **Egg tile → egg item (6142)**: `Check2x2` (`WorldGen.cs:48397-48590`)
  `case 752: num14 = 6142` (`:48576-48578`) → `Item.NewItem` (`:48750`).
  The tile-752 `TileObjectData` is `Style2x2` (`TileObjectData.cs:4016-
  4017`). The port's `check_candelabra_sink` (edit/kill.rs:1524) is the
  Check2x2 default arm but is dispatched only for 100/172 (kill.rs:667);
  752 is NOT in `check_object`'s match. **This is the egg-tile-break drop
  gap** — but it rides the `Check2x2` multi-tile-object framing family
  (the 335/411/490/564/565/594/94/95/... dispatch, `:85659/:85798/:85945`),
  which is the tile-framing lane's (W4-d110 / the Check* family), not a
  worldgen residual. Recorded as cross-lane, not ported here.
- **Egg item (6142) → pet**: `ItemID.Sets.OpenableBag` includes 6142
  (`ItemID.cs:1384`); `ItemSlot.TryOpenContainer_GrantItems`
  (`ItemSlot.cs:1832-1834`) → `Player.OpenChilletEgg` (`Player.cs:9491-
  9500`): `Next(2)==0` → 5666 (Ignis) else 5665 (Chillet), `QuickSpawnItem`.
  The port has NO `TryOpenContainer`/bag-opening system (clicks.rs:2603
  records "the OpenableBag arm has no container-opening state in the
  port"). **Cross-lane** (the UI container-opening lane), not ported here.
- **Pet item → mount**: 5665/5666 are `mountType 62/63` (`Item.cs:44519/
  44531`), 6150/6151 (Trusty) are `mountType 64/65` (`:47394+`). The
  mount data (`Mount.SetAsChillet`, `Mount.cs:2670-2744`, buffs 387/388/
  391/392) is ported (`mount.rs:1762-1796`, CHILLET/CHILLET_IGNIS/
  TRUSTY_CHILLET/TRUSTY_CHILLET_IGNIS). The item rows (itemdata.rs:5831/
  5832/6316/6317) and the kinship-peach recipes (`Recipe.cs:10522-10526`,
  5665+6191→6150, 5666+6191→6151) are ported (recipes_gen.rs:16873-16880). ✓

**Co-lane claims**: g45 (`11-liquid.md`) claims the liquid delay/AddWater
machinery only — no Chillet. f6 (`65-dust-world-sites-claims.md`) claims
the mount DUST (Chillet spawn/dismount `Mount.cs:6404/6413`, the 100×306
scale-2 burst) — the dust half, not the egg chain. Neither claims the
egg→item→pet chain.

**Verdict**: the worldgen-owned half (egg tile placement) is present and
faithful. The egg-tile-break drop (Check2x2-752) and the bag-open
(OpenChilletEgg) are real gaps but belong to the tile-framing and UI
container lanes respectively — recorded, not ported in this sweep.

## 5. GlowTulips glow — VERIFIED PRESENT (cross-note to g18/e8)

- **Tile**: GlowTulip = 656 (`TileID.cs:1941`), replica 701 (`:2031`).
  `GrowGlowTulips` (`WorldGen.cs:45622-45650`, 2/4/6 quota, right-band
  preference) → `TryGrowingGlowTulip` (`:73119-73142`) →
  `HasValidGroundForGlowTulipBelowSpot` (`:73096-73117`) +
  `NoNearbyGlowTulips` (`:73144-73162`, ±120 box) → `PlaceTile(656)`
  (`:59495-59506`). Ported in `worldgen/surface.rs::grow_glow_tulips`/
  `try_growing_glow_tulip`/`valid_ground_below`/`no_nearby_glow_tulips`
  (landed `dd43734a`). ✓
- **Light emission**: `TileLightScanner.cs:389-393` `case 656: R=0.2,
  G=0.55, B=0.5` (a constant emitter, no flicker/frame gate). The port's
  `tile_light/scanner.rs:751` `656 => *l = [0.2, 0.55, 0.5]` matches
  EXACTLY. ✓ **No gap.**
- **Cross-note**: the g18 lane (`W4-g18-addlight-main-1`) censused the
  Main.cs AddLight hook-chain sites (332/488) and found them claimed by
  the in-flight e8 lane (`W4-e8-addlight-tiles-2`, `tile_lights.rs`). The
  glow-tulip's emission is a TileLightScanner seed (the scanner.rs misc
  arm), NOT a Main.cs AddLight draw-site — it is outside both g18's and
  e8's AddLight scope and already landed in the scanner. No overlap.

## 6. Sunflower HAPPINESS buff — grant/effect PRESENT; spawn-flag feed PORTED

The sunflower area buff (id 146, "Happy!"):

- **Area grant**: `Player.UpdateBuffs` (`Player.cs:25548-25550`)
  `if (SceneMetrics.HasSunflower) AddBuff(146, 2)`. `HasSunflower` =
  `_tileCounts[27] > 0` minus the graveyard override
  (`SceneMetrics.cs:586-588/:617-624`). Ported: `scene.rs:583-588`
  (`has_sunflower = count(27) > 0`, graveyard cancels) →
  `auras().sunflower` → `player.rs:6992-6994` `if self.auras.sunflower {
  add_buff(146, 2) }`. ✓
- **Effect**: `Player.cs:10999-11004` `buffType==146: moveSpeed += 0.1;
  moveSpeed *= 1.1; sunflower = true`. Ported: the movement half
  (`player.rs:6609-6612`) + the flag half (`buff_effects_accessory.rs:216`
  `146 => p.sunflower = true`, cleared at `:171`). ✓
- **REAL gap found + ported (`7eb486f07`)**: the `player.sunflower` flag
  feeds `NPC.GetSpawnRate` (`NPC.cs:712-716`: `spawnRate ×1.2`,
  `maxSpawns ×0.8`) via `SpawnFlags.sunflower` — but the live spawn
  context (`world_sim/npcs.rs:6616`) only fed `invis`/`enemy_spawns`, so
  the sunflower modifier was **constants-off** since the spawner port.
  Fixed: `sunflower: player.sunflower` added to the SpawnFlags literal.
  - `calmed` (106, `NPC.cs:707-711`) stays constants-off on THIS base —
    its live `player.calmed` arm lands with the W4-f12 wiring lane
    (`buff_frame_wiring.rs:508` `p.calmed |= e.calmed`); no field exists
    here yet.
  - `angler_set` (`NPC.cs:717-721`) is **C# dead code**: `Player.cs:2808`
    declares `anglerSetSpawnReduction`, `:18729` clears it, and NO site
    sets it true (verified by full-file scan). The port's `angler_set`
    arm stays constants-off verbatim.

**Co-lane claims**: g58 (`W4-g58-ui-shop-happiness`) claims the NPC
shop-happiness UI/personality tables — NOT the sunflower tile buff. f12
(`W4-f12-buff-frame-wiring`) claims the buff-frame wiring; its `28-buffs-
debuffs.md` lists "Aura buffs (87/89/146/158/157/350/215) PORTED" (the
grant + effect) but does NOT claim the `GetSpawnRate` flag feed. The
spawn-flag wiring is unclaimed — this sweep ports it.

Witness (1, written not run): `w4_d105_the_sunflower_flag_feeds_the_spawn_modifiers`
(solo 432/4 from the 360/5 day-surface base; the invis-then-sunflower
stack 518/3, matching the C# `:702` then `:712` order).

## Commits (this sweep)

| hash | mechanic |
| --- | --- |
| `35ccf38ea` | herb growth: inverted frame-108 gate fix + the missing 1/30 fireblossom tail (+2 witnesses) |
| `52978903f` | runtime dye: the remix bypass halves + the lava/honey-only exotic veto (+2 witnesses) |
| `7eb486f07` | sunflower happiness: the `player.sunflower` → SpawnFlags feed (+1 witness) |

## Residual ledger (unported, cross-lane — NOT this sweep's scope)

- **Chillet egg-tile-break drop** (`Check2x2`-752 → 6142): the tile-framing
  Check* family lane (W4-d110 / the multi-tile-object framing).
- **Chillet bag-open** (`OpenChilletEgg`, `ItemSlot.TryOpenContainer`): the
  UI container-opening lane (no `TryOpenContainer` state in the port).
- **`calmed` (106) spawn-flag feed**: lands with W4-f12's `player.calmed`
  field + wiring.
- The guide's recorded deviations (vampire flag, the four `unlocked*`
  World fields, the TownNPCProfiles variation rolls) — unchanged, already
  in the starter_npcs.rs module header.

**Hash pins**: none of the three ports writes tiles at generation (they
are runtime growth / spawn-rate / flag-feed changes), so neither
`world_hash.rs` pin moves. The lane-5 gen-time hash pins stay
stale-by-design (deferred to the merge gate, per the lane discipline).

## RETIRE phase (2026-11-21, code-only) — SEALED

The three sweep ports are final on this branch. This section censuses
them against the co-lane tips (all reads are `git diff`/`git show` of the
pushed branches vs the common base `a725acb04`, plus direct ≤100-line C#
windows). Witnesses stay WRITTEN NOT RUN.

### R1. g42-growth compose (tip `fdedf9776`, GROWTH FINAL)

- **Disjoint, keep-one by construction**: g42's tree-diff vs base carries
  **ZERO `growth/herbs.rs` hunks** — its `grow_alch` body is the base copy
  (still the INVERTED `frame_x == 108` gate, no 1/30 tail). The keep-one
  survivor is **d105's herbs.rs** (`35ccf38ea`: `frame_x != 108 ||
  next_max(2) == 0` + the `:45890-45898` maturity tail, 2 witnesses). No
  g42-alch-arm duplicate exists: its dispatcher keeps the single
  `TILE_ALCH.contains(&kind) => grow_alch(...)` call per path (its
  growth.rs:777-778 overground / :2104-2105 underground) through the
  herbs.rs seam.
- **g42-D6-actuated-wall-arms**: F1 landed the C# wall arms OUTSIDE the
  active/inactive if-else (`:72499`/`:73809`) — the actuated path skips
  the growth chain (`:72216`/`:73252` nactive gates) but its evil wall
  spreads; the overground alch arm no longer returns, so a herb cell's
  walk reaches the wall arms. That is dispatcher-structure only — it
  changes WHICH cells reach `grow_alch`'s callers, not its body or draw
  order. The d105 fix composes under it unchanged.
- **decorations.rs**: g42's only hunk is `CAT_TAIL_DISTANCE` →
  `pub(super)` (line 16, the cattail export for its runtime PlaceCatTail
  copy). d105's dye hunks sit at `plant_dye` (~:560-640) + the two
  witnesses — no textual overlap, auto-merge clean, both halves keep.
- **Shared-file merge notes** (the true common set is anchors.txt,
  decorations.rs, world.rs, pipeline.rs, world_hash.rs): world.rs — g42
  adds `allowed_to_spread_infections` (:366/:722 regions), d105 adds the
  gen-NPC ledger + GenVars fields (:585/:624 regions), disjoint.
  pipeline.rs — g42's 3-line `step_mushroom` desert-veto arg vs d105's
  nine step insertions, disjoint regions. world_hash.rs — BOTH lanes
  prepend stale-by-design comments at the same anchor (the S1-P-i33
  comment above `SEED42_512`, :238); a textual conflict there resolves
  by KEEPING BOTH comment blocks (the pins stay stale until the merge
  gate re-pins; g42's mushroom-biome pin moves and d105's #37-45 pin
  moves are independent). anchors.txt — append-only rows, keep all.

### R2. Chillet egg chain — ownership census CORRECTED

The sweep's §4 recorded both runtime halves as unclaimed cross-lane. The
retire census re-reads the co-lane tips and corrects the ledger:

- **Egg tile 752 → item 6142 (tile-break drop) — UNCLAIMED; task-seed for
  the g34 Check\* continuation lane.** g34 (tip `cf85a1cf9`) generalized
  `check_candelabra_sink` → `check_2x2` (kill.rs:1596, the `:48397-48799`
  default branch incl. the `:48792-48798` TileFrame tail ring) and wired
  the 28 water-cage kinds + 100/172 (dispatch arm kill.rs:707-711); its
  research/68 explicitly parks the "boulder/ceiling/chandelier arms
  (95/126/132/138/484/664/665/711-716)" as separate lanes — 752 is in
  NEITHER list. g56 (tip `3ab631923`) is the void-vault UI lane: zero
  752/Check2x2/OpenChilletEgg hits in its tree-diff; it does NOT own the
  drop. C# re-read: the `TileFrame` inner switch lists 751/752 as plain
  `break` (`WorldGen.cs:85689-85691`) — no direct `Check2x2(i,j,752)`
  call site exists (the three call sites :85659/:85798/:85945 never
  list 752); the drop arm is `case 752: num14 = 6142` (`:48576-48578`,
  the sibling `case 751: num14 = 5667` :48573-48574 rides with it) and
  the reach rides the Style2x2 TileObjectData framing (the Check\*
  family lane). **Seed**: extend g34's `check_2x2` drop switch with
  752→6142 (+751→5667) and wire the dispatch through the object-data
  route — the tile-framing lane (W4-d110 / the g34 Check\* continuation),
  NOT the void lane, NOT worldgen.
- **Egg item 6142 → pet (bag-open) — CLAIMED + LANDED on W4-f35.** The
  sweep's "the port has NO TryOpenContainer system" was true at base but
  is stale at the f35 tip (`1abae9ade`): `misc_bags.rs::open_chillet_egg`
  ports `Player.OpenChilletEgg` (`Player.cs:9491-9500`, the `Next(2)==0`
  → 5666 Ignis else 5665 Chillet, `quick_spawn`) — commit `0fd244e79`;
  the full 58-kind OpenableBag dispatch + the `TryOpenContainer_GrantItems`
  tail (`ItemSlot.cs:1806-1865`) wired at clicks.rs:1601 — commit
  `00a5b27db`; research/66 row `type == 6142 | Chillet Egg |
  OpenChilletEgg`. Nothing left for d105 or g56 here; the chain closes
  when f35 + the tile-framing seed both land.
- The worldgen-owned half (egg placement, `dd43734a6`) stays verified
  present; item row 6142 exists on base (itemdata.rs:6308) and the mount
  halves remain as §4 recorded.

### R3. sunflower-happiness g58 compose (tip `2ff3b7062`, happiness final)

- **Disjoint**: g58's tree-diff touches NO spawner.rs, NO
  buff_effects_accessory.rs, NO player.rs and NO tr-core growth/worldgen
  files; its only world_sim/npcs.rs hunks are the shop/talk/angler-reward
  arms (:75-:1058 regions) + tests (:7536) — none near the live
  SpawnFlags literal. Its SpawnFlags literal (npcs.rs:6684-6713) keeps
  `invis`/`enemy_spawns` + `..default()` (sunflower constants-off there).
  **Keep-one: d105's `sunflower: player.sunflower` feed** (`7eb486f07`,
  the `:6638` hunk + the spawner.rs witness). g58's shop-mood system is
  the `shopping_settings.price_adjustment` TALK-OPEN LATCH
  (`Player.SetTalkNPC :4389`, buy/sell/reforge/nurse/tax/angler readers)
  — a different flag family entirely; no merge interaction.
- **calmed (106) stays off — f12-claimed, confirmed at its RETIRE tip
  `22ce2a41f`**: f12 carries `player.calmed` (player.rs:2091, `106,
  :10142`) + the `buff_frame_wiring.rs:508` `p.calmed |= e.calmed` arm,
  and its tree-diff has ZERO hunks in world_sim/npcs.rs or spawner.rs —
  the live `SpawnFlags.calmed` feed is claimed by f12's wiring but NOT
  yet landed there. d105 keeps the field constants-off verbatim (the
  spawner arm at spawner.rs:394 exists since M4-SPAWN-1). Merger note:
  when f12 lands the feed it adds one line beside d105's sunflower line
  in the same literal — keep both. `angler_set` stays off (C# dead code,
  §6).

### R4. Seal + family declaration

- **research/129 SEALED**: the six-residual census is final — guide and
  glow-tulip verified present, the three ports landed (`35ccf38ea` /
  `52978903f` / `7eb486f07`), the Chillet ledger corrected above (one
  task-seed to the Check\* lane, one half already landed on f35). No
  further d105 work on these systems.
- **wg-lane5-family declaration**: published in
  `12e-worldgen-passes-5.md` (the RETIRE section there) — the full
  commit family, the owned files, the co-lane boundaries and the
  hash-pin state, for the merge gate.
