# 75 — Worldgen pass reconcile: g37 B1–B9 vs d103/d104 (merger assist)

> Wave 4 · produced on `task/W4-g37-worldgen-missing-b` @ 1975f559 (reconcile
> phase, code-only — no cargo/tests run; every claim below is a source read of
> the pushed tips + the C# anchors, read in ≤100-line windows).
> Companion doc: `docs/research/88-worldgen-master-matrix.md` on
> `task/W4-g36-worldgen-missing-a` @ 6d46f6a5 (the master keep-one matrix).
> This doc is the **arm-by-arm fidelity layer** under that matrix for the nine
> g37 passes: which duplicate is more faithful against the C#, with line-level
> evidence, plus the executed-step-order union, the g39 helper resolution and
> the witness union. Where this doc's evidence agrees with 88 it says so; the
> two places it REFINES 88 are flagged (§3.7 Hellforge, §3.9 Surface Ore).

## 1. Tips read (fetched 2026-11-21, second fetch after the lanes moved)

| lane | tip read | content relevant here |
| --- | --- | --- |
| g37 (this) | `1975f559` (02:51) | B1–B9 in `passes_b.rs` (2634 lines, **0 module tests**, 9 pipeline positional witnesses), `islands.rs` sites ledger, 75-step pipeline |
| d103 | `8be04cb0` (02:18) + tail-census `fe9a565f` (04:14, docs only) | passes 19–27: `statues.rs` 424 / `buried_chest.rs` 655 / `water_chests.rs` 253 / `cave_walls.rs` 463 / `island_houses.rs` 559 lines; W:7/7/3/7/4 |
| d104 | `8a229c3e` (03:15) | passes 28–36: `island_houses.rs` 596, `micro_finish.rs::pass_quick_cleanup`, `structures.rs::pass_hellforge/pass_piles`, `surface.rs::pass_spreading_grass/pass_surface_ore_and_stone/pass_place_fallen_log/pass_spawn_point`, `traps.rs` |
| d105 | `d590d99a` (03:59) | #37–44 landed (Guide…Moss Grass); Larva NOT landed (c31 owns it) |
| d106 | `e2c2609c` (03:39) | R1–R5 landed (Grass Wall…Remove Broken Traps); R6 Final Cleanup NOT landed |
| f24 | `9f2276d5` (02:36) | gated Skyblock + Dither Snake (68 steps) |
| f25 | `f9c3dba7` (04:18) | Settle Liquids pass `0ed1f3cf` + NEW C1–C7 Convert/growth helper lane (no pipeline change) |
| g36 | `6d46f6a5` (04:00) | A1–A7 (A5 yielded to f25) + the master matrix doc 88 |
| g38 | `8c9b7b60` (00:49) | C1 Place Fallen Log only |
| g39 | `bab25558` (23:41) | H1–H15 `place_helpers.rs` (unwired) |
| g44 | `a6412835` (03:13) | verbatim FloatingIslands + IslandHouse + skyLakes (`islands.rs` 1143 lines) |
| c33 | `321c9b66` (02:59) | faithful Pyramids + a THIRD Water Chests (`structures.rs::pass_underwater_chests`) |

**Correction to the dispatch brief:** B6–B9 are NOT unique to g37 — d104
landed all four (Quick Cleanup `df067118` 00:38, Hellforge `2ac7c298` 00:44,
Spreading Grass `3a101031` 00:55, Surface Ore `19bc132b` 01:15), each BEFORE
g37's corresponding pass (B6 01:43, B7 01:55, B8 02:09, B9 02:21). B5 is
FOUR-way (d103/d104/g37/g44), B3 is THREE-way (d103/g37/c33). The g37 census
re-probe (`30f605c7`, 22:41) was correct at its timestamp — d103's
B-duplicates landed 00:23–02:01 and d104's 00:24–01:15, all after it. No lane
violated a pushed claim; the claims were doc-only at probe time.

## 2. Per-pass arm-by-arm fidelity (g37 slice vs the sibling port)

Legend: **C#** = `decompiled/Terraria/WorldGen.cs` unless noted. "verbatim"
means same structure/draw-order/constants; "deferred" means the arm is cited
but not ported (special-seed deferral, the lane convention).

### 2.1 B1 Statues (`:16853-16970`) — g37 `passes_b.rs::pass_statues` vs d103 `statues.rs`

| arm (C# anchor) | g37 B1 | d103 `bec7d67a` | more faithful |
| --- | --- | --- | --- |
| count `(73*2)*(w/4200)` (`:16859-16860`) | verbatim | verbatim | tie |
| noTraps ×15, /5 anniversary/bees (`:16861-16868`) | deferred | ported (`no_traps`/`anniversary_or_bees` params) | **d103** |
| starGame ×0.2 (`:16869-16872`), skyblock num4=10/count÷2 (`:16874-16878`) | deferred | deferred (adapter gate) | tie |
| remix extra draw consumed-and-discarded (`:16894-16897`, `:16908-16911`) | deferred | **verbatim quirk kept** (both draw sites) | **d103** |
| oceanDepths budget-100 re-roll (`:16898-16912`) | verbatim (dualDungeons disjunct deferred) | verbatim (same deferral) | tie |
| fall-to-first-active + `num7--` (`:16919-16933`) | verbatim | verbatim | tie |
| frameImportant/!tileCut + anyShimmer vetoes (`:16934-16943`) | verbatim (`tileobject::frame_important`) | verbatim (`world.frame_important` vec + boot statics 105/349/135) | tie |
| PlaceTile forced (`:16946`) | full `place_tile` slice incl. the **488 peace veto** (`:59345-59348`) inside the slice | `place_2xx_statue` + the 488 peace veto hoisted to the pass body | tie (same behaviour, different seam) |
| success/trap/`num++`/10000 give-up (`:16948-16964`) | verbatim | verbatim | tie |
| PlaceStatueTrap 21×21 plate scan + WireLine (`:4490-4513`) | verbatim (`solid_tile2_of` Tile-overload, null→true note) | verbatim (`solid_tile2` indexed; `actually_no_traps` gate) | tie |
| sub-vanilla guard | `y_hi<=y_lo \|\| w<=40 \|\| num3<=0` | `w<=40 \|\| band collapse \|\| remix band collapse` | tie (d103 also guards the remix band) |
| module witnesses | 0 (1 pipeline positional) | **7** (list table, 3×2 frames, jug swap, style-55 wrap, trap plate+wire, a==b, size guard) | **d103** |

**Verdict: d103 more faithful** (2 extra seed arms + the remix quirk + 7
witnesses). g37's unique value: the SEED42_512-unchanged beach-band proof
(the 512-wide world sits inside the ±380 oceanDepths margin → every candidate
rejected) — carry into the gate hash notes. **KEEP d103, DROP g37 B1** —
agrees with 88 §2.7.

### 2.2 B2 Surface Chests (`:17130-17210`) + AddBuriedChest core (`:35877-37568`)

Pass body: both verbatim on the classic arm (count `w*0.005`, 2000 budget,
unbounded oceanDepths re-roll with the 300-margin re-draw, wall-2/59/244 gate,
±50 `j+=2`/`k+=2` wall-244 scan with `Next(num8)==0` + `num8++`, style 12/−1).
d103 adds the remix y-band re-draw (`:17150-17153`) and the remix wall-gate
bypass; g37 defers both. The core is where they diverge:

| core arm (C# anchor) | g37 `add_buried_chest` | d103 `buried_chest.rs` | more faithful |
| --- | --- | --- | --- |
| floor scan `j..h-10` + shimmer/231 aborts (`:35889-35913`) | verbatim | verbatim | tie |
| trySlope style-17 ±30 chest box + flatten/restore (`:35914-35944`, `:37554-37564`) | verbatim | verbatim | tie |
| remix centre-bottom abort (`:35945-35948`) | deferred | ported | **d103** |
| ±2 boulder/26/237 box fluff-100 (`:35949-35959`) | verbatim | verbatim | tie |
| flag15 default + caller style (`:35966-35985`) | classic only | + remix flag15 (`num7 < h-400`) | **d103** |
| errorWorld style shuffle (`:35986-36014`, placement draws) | deferred | **ported** (1/2 gate, 6/5 remap, 1/3 `Next(52)` + 18-27 re-roll) | **d103** |
| desert arm flag3 (`:36023-36048`) | probe kept (467/style 10) | + the 467-style-10 / getGood-42 entry disjuncts + hive-depth flag20 read | **d103** |
| ice arm flag2 (`:36064-36088`) | probe kept (147/161/162/197) | + style-11/24 entry disjuncts + drunk-34 arm | **d103** |
| jungle arm flag4 (`:36089-36114`) | **absent** (documented "never reached") | ported (styles 10/8/29, main 211/212/213/753, bees arms) | **d103** |
| hell arm flag8/flag14 (`:36115-36125`) | style 4 set, **no counter advance** | + `GenVars.hellChest++` wrap (`:36210-36217`) + the hellChestItem Reset shuffle | **d103** |
| living-tree wall-244 gate (`:36134-36150`) | verbatim | verbatim | tie |
| water arm flag12 + remix evil swap (`:36176-36200`) | absent | ported (13/159/65/158/2219 → 43 or 467/3, drunk half-split) | **d103** |
| noTraps gold→mahogany swap (`:36201-36205`, placement draw) | deferred | ported | **d103** |
| PlaceChest route (`:36206`, `:57813-57846`) | `place_chest_at`: boulder veto + NearOtherChests + **TileObject.CanPlace/Place** (the C# route; place_rng take/restore, RandomStyleRange-0 note) | compact `structures::place_chest` 2×2 frame write + boulder/NearOtherChests hoisted into the core | **g37** (route fidelity; d103's compact write skips CanPlace's support checks) |
| NearOtherChests set (`Chest.cs:543-557`) | `{21,467}` literal | `SET_BASIC_CHEST` (the C# set) | **d103** |
| stocking | inside the core, layer-anchor save/restore | at the pass level via `BuriedChestPlace` | tie (seam choice) |
| GenVars | none added | `hell_chest`/`hell_chest_item` Reset shuffle + `desert_hive_high/low` | **d103** |
| module witnesses | 0 | **7** (floor place, gold-style pad, shimmer abort, boulder abort, near-chest block, a==b, size guard) | **d103** |

**Verdict: d103 core more faithful** (9 extra arms incl. two placement-draw
arms that shift the stream on seed worlds). g37's `place_chest_at`
TileObject route is the one arm where g37 is closer to C# — the merger should
graft that route under d103's core (88 §2.8 already keeps d103; this is the
graft note). **KEEP d103, DROP g37 B2** — agrees with 88 §2.8.

### 2.3 B3 Water Chests (`:17245-17355`) — THREE-way (d103 / g37 / c33)

| arm (C# anchor) | g37 B3 | d103 `9a935127` | c33 `321c9b66` |
| --- | --- | --- | --- |
| treasure NextFromList 5-item (`:17252`) | verbatim | verbatim | verbatim |
| extraLiquid 6-item 268 (`:17253-17256`) | deferred | **ported** | deferred |
| widening 2.0+0.1 box, `(int)num/2` bands, edge fold (`:17258-17264`) | verbatim | verbatim | verbatim |
| InWorld + liquid>250 + water (`:17265`) | water only | + notTheBees/remix disjuncts | water only |
| style-17 trySlope place (`:17267`) | verbatim | verbatim | verbatim |
| scatter count `9*w/4200` (`:17272-17277`) | `(9.0*num5) as i32` for-loop | **`while (j as f64) < 9.0*num5`** (the C# float compare — differs at fractional boundaries) | same float while |
| extraLiquid ×3 (`:17273-17276`) | deferred | **ported** | deferred |
| anniversary 1/7 (`:17282-17286`) | deferred | **ported** | deferred |
| 186/4404/277/187 cycle + extraLiquid-268 (`:17293-17316`) | classic cycle | + extraLiquid num4==4 arm | classic cycle |
| arm-1 y-floor 1→50 quirk (`:17322` vs `:17326`) | verbatim | verbatim | verbatim |
| arm-1 remix disjunct (`:17323`) | deferred | **ported** | deferred |
| arm-2 NO-remix gate (`:17341` — notTheBees only) | deferred (water only) | **verbatim asymmetry kept** | deferred |
| inner re-draw loops | **10000-cap degenerate-world guard** (documented deviation) | unbounded (verbatim; module note says waterless fixtures must not call) | unbounded |
| outer 10000 place budget (`:17330`, `:17348`) | verbatim | verbatim | verbatim |
| beach-band trySlope (`:17328`) | verbatim | verbatim | verbatim |
| module witnesses | 0 | **3** (size guard, treasure style-17, a==b) | 1 (+stocking-hook pair 4425/4460) |

**Verdict: d103 most faithful** (5 extra arms + the float-loop compare).
g37's inner 10000-cap is a deliberate anti-hang deviation, NOT more faithful —
but it is the right guard to graft onto d103's unbounded loops at the gate
(88 §6.3 already lists this). c33's stocking-hook additions (flag5 4425/4460
pair, style-17 main) are loot-side and ride the kept core's chest_stock seam.
**KEEP d103, DROP g37 B3 + c33 `pass_underwater_chests`** — agrees with 88
§2.9. c33's Pyramids + lily/cattail/bamboo stay (no g37 overlap).

### 2.4 B4 Cave Walls (`:17711-17839`) — g37 vs d103 `463cab95` (+`8be04cb0`)

| arm (C# anchor) | g37 B4 | d103 | more faithful |
| --- | --- | --- | --- |
| tileSolid[162]=false/[226]=true/[232]=false window (`:17713-17715`) | **unmodelled** (static `td::TILE_SOLID`; `edit::solid_tile` reads) | **modelled** (`solid_cave_walls`: 162/232→false, 226→true, else solid&&!solidTop) | **d103** |
| sweep-1 band `(int)(surface+rock)/2` (`:17725`) | `((surface+rock)/2.0) as i32` (float-divide-then-truncate) | `((surface+rock) as i32)/2` (truncate-then-divide, the C# cast order) | tie — `floor(floor(S)/2)==floor(S/2)` for S≥0, so both match the C#; d103 mirrors the cast order literally |
| 4% count, 500-budget search, 10≤tiles<1500 (`:17720-17741`) | verbatim | verbatim | tie |
| countTiles bails/counters (`:9546-9635`) | verbatim incl. BASELINE_WALLS {1,2,187,216} adaptation | verbatim, same adaptation | tie |
| wall pick shroom>rock*0.75→80 / ice→40/71 / lava→79 / Next(4)→59/61/170/171 (`:17744-17783`) | verbatim (both Next(2)+Next(4) draws) | verbatim | tie |
| remix sweep-2 clean-pocket dirt/grass (`:17787-17809`) | deferred | **ported** (ice/lava/sand==0 gate, Next(2)!=0→63 else 2) | **d103** |
| jungle sweep 2%, wall-64 gate, wallHouse/244 stale-count (`:17811-17837`) | verbatim | verbatim | tie |
| Spread.Wall wave BFS (`:3356-3414`) | verbatim (baseline adaptation) | verbatim | tie |
| Spread.Wall2 5000 budget + protected/stops-at-air sets (`:3416-3524`) | verbatim | verbatim | tie |
| sub-vanilla guard | per-sweep `if` guards (draws nothing when collapsed) | whole-pass guard incl. remix bands + lavaLine | tie (d103 stricter) |
| module witnesses | 0 | **7** (count pocket, real-wall bail, Wall rim, Wall2 convert, protected walls, a==b, size guard) — fixture corrected to baseline-wall convention by `8be04cb0` | **d103** |

**Verdict: d103 more faithful** (the solid window + the remix arm + 7
witnesses). The band expressions are equivalent (see table). **KEEP d103,
DROP g37 B4** — agrees with 88 §2.10.

### 2.5 B5 Floating Island Houses (`:17859-17874` + IslandHouse `:79203-79484`) — FOUR-way

All four ports do the faithful two-pass split (islands records sites, a later
pass builds). House-body comparison (classic arm unless noted):

| arm (C# anchor) | g37 B5 | d103 `eb018138` | d104 `93b1938e` | g44 `ae4472e6`+`a6412835` |
| --- | --- | --- | --- | --- |
| ±30 dungeon/87/3/83 wall veto (`:79206-79216`) | verbatim | verbatim | verbatim | verbatim |
| drunk crimson half-split (`:79217-79220`) | deferred | **ported** | ported (flags) | deferred (module doc) |
| Next(2) dir + Next(7,12)/Next(5,7) (`:79224-79230`) | verbatim | verbatim | verbatim | verbatim |
| ground scan j-15..j+30, 192/191/384 exclusions (`:79238-79247`) | verbatim; fallback `vec_y=j` (deviation, cited) | verbatim; fallback `floor_y=j` | verbatim | verbatim; **skip on no-hit** (the C# builds at row 0 — g44's valve is a deviation too) |
| j>worldSurface → j-5 (`:79232-79235`) | verbatim | verbatim | verbatim | verbatim |
| sunplate-202 shell, top-corner skip (`:79269-79283`) | verbatim | verbatim | verbatim | verbatim |
| skyware-82 interior, wall-0 only (`:79304-79314`) | verbatim | verbatim | verbatim | verbatim (disc wall 82) |
| door carve 5×3 + style-9 door (`:79315-79337`) | verbatim (`place_tile` door arm) | + remix 5/38 dispatch | + remix dispatch | classic 9; **`place_tile_door` up/down-hang dispatch** (`:59843-59883`) ported |
| far wall column (`:79338-79347`) | verbatim | verbatim | verbatim | verbatim |
| skyIslandHouseCount cycle 159/65/158/2219, >3→Next(4) (`:79348-79368`) | verbatim | verbatim | verbatim | verbatim |
| chest style 13 (getGood 2) via AddBuriedChest (`:79369-79376`) | full core call | full core call | **compact `add_sky_chest`** (larva/altar/floor only — no shimmer abort, no boulder set, no trySlope) | compact place_chest + stock seam (floor-scan stand-in, cited) |
| fountain search islandStyle>0 (`:79377-79393`) | verbatim (100000 budget, 50000 relax) | verbatim | verbatim | deferred (classic style 0) |
| counter advance (`:79394`) | verbatim | verbatim | verbatim | verbatim |
| window-21 walls (`:79395-79419`) | verbatim | + remix gate | + remix gate | verbatim |
| table 14/7 + chairs 15/10 + left-chair +18 (`:79420-79443`) | verbatim | + remix 469/2 + 8/38 arms | verbatim | verbatim |
| banners 91 ×3 Next(7,10) (`:79444-79461`) | verbatim | + remix gate | verbatim | verbatim |
| palms islandStyle==1 (`:79462-79483`) | verbatim (`ocean::grow_palm_tree`) | verbatim | verbatim | deferred |
| **producer** (FloatingIslands `:12927-13122`) | shape-level islands + sites ledger (count multipliers kept) | shape-level + ledger arrays | shape-level + slot ledger | **verbatim rewrite**: 0.0008·w count, skyLakes tail, errorWorld triple, spawn re-draw 150/100, spacing, y-band + surfaceLow clamp, num3·2 cap, disc-top-row ledger fix |
| module witnesses | 0 (+2 islands ledger tests) | **4** | 2 | **10** |

**Verdict: g44 wins on the producer** (the only verbatim FloatingIslands —
the sites every house builds on come from it; the other three keep the
shape-level disc). On the house body alone d103 is the most arm-complete
(drunk split + remix furniture + full chest core), g37 second (full core,
fountain/palms), d104/g44 use compact chest stand-ins. **KEEP g44 for both
passes** (agrees with 88 §2.11), with two grafts at the gate: (a) d103's
drunk-crimson split + remix door/furniture arms onto g44's house, (b) the
chest call re-pointed onto d103's `buried_chest.rs` core (88 §2.8). g37's
`island_house_sites` tuple ledger and d103's array ledger both DROP in favour
of g44's `island_house_x/y` + `sky_lake` + `island_style` Vecs. **DROP g37 B5,
d103 `island_houses.rs`, d104 `island_houses.rs`.**

### 2.6 B6 Quick Cleanup (`:17875-17995`) — g37 vs d104 `df067118`

| arm (C# anchor) | g37 B6 | d104 | more faithful |
| --- | --- | --- | --- |
| ocean non-water→water (`:17893-17896`) | verbatim | verbatim | tie |
| beach sand drip: `Next(4,7)` **inside the for-condition** (`:17903`) | **drawn ONCE** (`let fill = rng.range(4,7)` before the loop) — deviation | **re-rolled per condition check** (`let limit = j + rng.range(4,7)` inside the loop) — the C# semantics | **d104** (RNG-order divergence on g37) |
| 397/53/495 stop conditions k/k+1/k+2 (`:17903`) | verbatim | verbatim | tie |
| desert-wall 187/216 retype + rockLayer liquid clear / lava-255 (`:17911-17933`) | verbatim (dualDungeons gate param) | verbatim (micro flag) | tie |
| surface-sand ±3 wall bleed (`:17934-17957`) | **`break 'bleed`** — exits BOTH loops on the first walled cell | inner `break` only — **last-column-wins** (the C# quirk, cited) | **d104** (g37 picks the first wall, C# the last column's) |
| slope clear family SaveSlopes/Platforms/CanBeCleared (`:17958-17989`) | verbatim | verbatim | tie |
| 225 half-brick exemption + 137 dart deactivation (`:17966-17987`) | verbatim | verbatim | tie |
| tileSolid[137]/[130]=false window (`:17885-17886`, `:17992-17993`) | **unmodelled** | **modelled** (`solid_tile_qc`: 137/130→false) | **d104** |
| notTheBees arm (`:17881-17884`) | deferred | deferred (biomes.rs scope) | tie |
| module witnesses | 0 | **2** (sweep replay+tidy, ocean lava→water; 10 tests in file) | **d104** |

**Verdict: d104 more faithful on three behavioural arms** (the per-check
Next(4,7) draw, the last-column-wins bleed, the solid window). g37's B6 has
two real divergences, not just deferrals. **KEEP d104, DROP g37 B6** —
agrees with 88 §2.12.

### 2.7 B7 Hellforge (`:18187-18236`) — g37 vs d104 `2ac7c298` — REFINES 88

| arm (C# anchor) | g37 B7 | d104 | more faithful |
| --- | --- | --- | --- |
| count `w/200` (`:18192`) | verbatim | verbatim | tie |
| Next(1,w) × Next(h-250,h-30) (`:18200-18201`) | verbatim | verbatim | tie |
| wall 13/14 gate (`:18204`) | verbatim | verbatim | tie |
| drop scan bounded h-20 + `j--` (`:18206-18209`) | verbatim | verbatim | tie |
| **PlaceTile(num3,j,77) default mute/forced** (`:18210`) | **`place_tile(...,77,false,false,0,rng)`** — the full PlaceTile slice: the 488 veto, the `forced \|\| EmptyTile \|\| !tileSolid` entry gate (`:59373`), the Clear arm, the 77 dispatch → Place3x2 | **`evil::place_3x2(...,77,0)` directly** — skips the PlaceTile preamble (entry gate + Clear + the type-77 success semantics ride the raw write) | **g37** |
| type-77 success / 10000 give-up (`:18211-18222`) | verbatim | verbatim | tie |
| catch arm OOB-only increment (`:18225-18232`) | documented deviation (dense array never throws; wall-miss retries without bump) | same behaviour, stated as the wall-miss no-bump arm | tie |
| skyblock denyAllGeneration (`:18189`) | deferred | adapter gate | d104 (gate only) |
| sub-vanilla guard | `h<=250 \|\| num<=0` | none cited | g37 |
| module witnesses | 0 | 1 (places+replays) | d104 |

**Verdict: g37's body is more faithful on the placement route.** The C#
`PlaceTile(77)` is NOT a bare Place3x2: it runs the entry gate
(`forced:false` → the cell must be empty or non-solid), the Clear arm and the
halfBrick reset before the 77 dispatch. d104's direct `place_3x2` writes over
an occupied cell the C# would refuse. 88 §2.13 keeps d104 on a witness tie +
co-location; **this doc recommends KEEP d104's pass shell (gate, slot,
witness) but GRAFT g37's `place_tile` route for the 77 write** — or re-point
d104 onto g39's canonical PlaceTile when it lands. If the merger prefers a
single body, g37's is the more faithful one; the decision is the Lead's.

### 2.8 B8 Spreading Grass (`:18237-18446`, classic arm) — g37 vs d104 `3a101031`

Sweep 1 (±50 surface retypes) and sweep 2 (±10 first-surface grass) are
**line-for-line identical** between the two ports (jungle-grass-60 3×3 spread,
the 7×7 flavour scan with the 53-sticky latch, the open-air flag, the 23/199
cover veto + jungle-band 59/60 split; the flag2 latch + worldSurfaceHigh
break). Both draw no RNG. The only difference is the SpreadGrass seam:

| arm | g37 B8 | d104 | more faithful |
| --- | --- | --- | --- |
| sweep 1 (`:18242-18320`) | verbatim | verbatim | tie |
| sweep 2 (`:18321-18353`) | verbatim | verbatim | tie |
| SpreadGrass(0,2) generation slice (`:75146-75268`) | local `spread_surface_grass` (InWorld(10), dirt-0 guard, surface rule, 3×3 enclosure + lava exception, CanBeCleared gate, grassSpread<1000 recursion) | **`evil::spread_evil_grass(...,repeat:false)`** — the shared Convert-family port, gains the C# `repeat` parameter (evil callers pass true, unchanged) | **d104** (the shared seam is the C# signature; g37's local slice duplicates it) |
| remix arms (`:18354-18443`) | deferred | deferred (d107) | tie |
| module witnesses | 0 | **2** (dirt takes grass, stone takes flavour) | **d104** |

**Verdict: d104 more faithful on the seam** (one shared SpreadGrass with the
C# repeat parameter vs a second local copy). Behaviour on the classic arm is
identical. **KEEP d104, DROP g37 B8** — agrees with 88 §2.14.

### 2.9 B9 Surface Ore and Stone (`:18447-18512`) — g37 vs d104 `19bc132b` — REFINES 88

Pass shell (counts, budgets, bands, spacing ledger) is **identical** in both:
ore `Next(5w/4200,10w/4200)`, `w/420` budget, beach band + [0.48,0.52]
re-roll, ±200 spacing + `maxOrePatch-1` cap; stone `Next(1,7w/4200)`, ±100,
[0.47,0.53]. Differences:

| arm (C# anchor) | g37 B9 | d104 | more faithful |
| --- | --- | --- | --- |
| ore tier pick (`:10581-10585`) | **hardcoded 6/7** (`if Next(3)==0 {6} else {7}`) | **`SavedOreTiers.Copper/Iron` params** (`copper`/`iron` from `world.ore_tiers`, the Shinies-rolled tiers — tin 166/lead 167 worlds) | **d104** (g37 writes copper/iron even on a tin/lead world) |
| floor scan + grass triplet + wall veto + body scan (`:10588-10612`) | verbatim (`surface_patch_floor`, Conversion.Grass {2,23,199,109,477,492}) | verbatim (`SET_GRASS`) | tie |
| Next(2) sink + seed + OreHelper (`:10614-10617`) | verbatim | verbatim | tie |
| Next(8,13) re-roll-per-check walk (`:10622-10650`) | verbatim | verbatim | tie |
| drifting blob (`:10651-10703`) | verbatim (Next(5,9)/Next(9,14), 1/3 bumps, per-cell radii, 1/8 double, drift clamps) | verbatim | tie |
| StonePatch (`:10242-10330`) | verbatim (199/23 veto, Next(13,18)/Next(13,19), 1/7 pile arm) | verbatim + **186 Place3x2 + two 185 PlaceSmallPile decor** | **d104** (the decor arm) |
| place_small_pile | local slice (`:46815-46873`) | **gains the C# `type` parameter** (spider callers unchanged) | **d104** |
| GenVars slots | `num_ore_patch: i32` + `ore_patch_x: [i32;50]` (the C# fixed array) | `ore_patch_x: Vec<i32>` (len<49 cap) | tie (g37's shape is the C# array; d104's Vec is the port idiom — keep ONE, 88 §3.4 picks d104's) |
| sub-vanilla guard | `w<=760 \|\| y-band collapse` | `w*7/4200<=1` (≤600) | tie (both keep the 512 pin) |
| module witnesses | 0 | 1 (patches place+replay) | d104 |

**Verdict: d104 more faithful** — the ore-tier parameterisation is a real
world-state arm g37 hardcodes away, plus the StonePatch decor. 88 §2.15 keeps
d104 on "decor + type-param"; this doc adds the **tier-roll evidence** as the
stronger reason. **KEEP d104, DROP g37 B9.** GenVars: keep d104's
`ore_patch_x` Vec (88 §3.4); g37's `[i32;50]` + `num_ore_patch` DROP.

## 3. KEEP/DROP summary (the nine g37 passes)

| g37 pass | KEEP | DROP | reason (strongest arm) |
| --- | --- | --- | --- |
| B1 Statues | d103 `bec7d67a` | g37 B1 | remix quirk + noTraps arms + W:7 |
| B2 Surface Chests | d103 `f3929904` | g37 B2 | full core (9 extra arms); graft g37's TileObject PlaceChest route |
| B3 Water Chests | d103 `9a935127` | g37 B3 + c33 underwater | anniversary/extraLiquid/no-remix arms; graft g37's 10000-cap + c33 stocking pair |
| B4 Cave Walls | d103 `463cab95` | g37 B4 | solid window + remix arm + W:7 |
| B5 Island Houses | **g44** `ae4472e6`+`a6412835` | g37 B5 + d103 + d104 house bodies | verbatim FloatingIslands producer + skyLakes; graft d103 drunk/remix arms + chest core |
| B6 Quick Cleanup | d104 `df067118` | g37 B6 | per-check Next(4,7) + last-column bleed + solid window |
| B7 Hellforge | d104 `2ac7c298` shell | g37 B7 body | **REFINES 88**: graft g37's PlaceTile route (d104's bare Place3x2 skips the entry gate) |
| B8 Spreading Grass | d104 `3a101031` | g37 B8 | shared SpreadGrass repeat-param seam |
| B9 Surface Ore | d104 `19bc132b` | g37 B9 | **REFINES 88**: SavedOreTiers params (g37 hardcodes 6/7) + StonePatch decor |

**Net: g37 contributes ZERO pass bodies to the merged chain** (agrees with 88
§6.2). Its surviving value: (1) the census split + re-probe discipline, (2)
the SEED42_512 beach-band statue proof, (3) the B2/B3 chest-family slot fix
(`809a61a8`) — the executed-order grouping the merged chain keeps (§4), (4)
the two graft bodies (B7 PlaceTile route, B3 inner cap), (5) the 9 positional
witnesses the gate may port onto the kept owners' slots.

## 4. Pipeline step-order union (the executed chain, both insert)

The port's `EXECUTED_PIPELINE` is a grouped chain, not the raw C# order. g37
(75 steps) and d103 (75) / d104 (75) each insert their passes at different
group points. The union below is the **merged executed order** — g37's spine
with every sibling's new step inserted at its local predecessor, the
`granite and marble` combined step replaced by d101's `marble`+`granite`
split, and the KEEP owners from §3. **119 steps.** (The C# `AddPasses`
declaration order — the 109-registration table — is 88 §5; this is the
EXECUTED order the port actually runs, which is what the count/positional
witnesses assert.)

| # | step | owner | # | step | owner |
| --- | --- | --- | --- | --- | --- |
| 1 | terrain height map | base | 61 | spreading grass | d104 |
| 2 | terrain fill | base | 62 | surface ore and stone | d104 |
| 3 | plan biome spans | base | 63 | place fallen log | d104 |
| 4 | skyblock (gated) | f24 | 64 | traps | d104 |
| 5 | dunes | base | 65 | piles | d104 |
| 6 | ocean sand | base | 66 | spawn point | d104 |
| 7 | micro grass | base | 67 | beehives | c31 |
| 8 | jungle | base | 68 | hell forts | base |
| 9 | mud caves to grass | base | 69 | lihzahrd temple | base |
| 10 | full desert | base | 70 | temple part 2 | base |
| 11 | mushroom patches | base+g42 | 71 | cave walls | d103 |
| 12 | sand patches | d101 | 72 | statues | d103 |
| 13 | cave fields | base | 73 | cave houses | base |
| 14 | mount caves | d101 | 74 | dead man's chests | c29 |
| 15 | dirt wall backgrounds | d101 | 75 | thin ice | base |
| 16 | marble | d101+g40 | 76 | campsites | base |
| 17 | granite | d101+g40 | 77 | spike pits | base |
| 18 | stone-in-dirt blobs | base | 78 | sword shrines | base |
| 19 | dirt-in-stone blobs | base | 79 | mining explosives | base |
| 20 | clay | d101 | 80 | mahogany trees | base |
| 21 | small holes | d101 | 81 | webs and honey | base |
| 22 | dirt layer caves | d101 | 82 | mushroom and jungle plants | base+g42 |
| 23 | rock layer caves | d101 | 83 | jungle plants | base |
| 24 | surface caves | base | 84 | vines | base |
| 25 | wavy caves | base | 85 | flowers | d105 |
| 26 | generate ice biome | d101 | 86 | mushrooms | d105 |
| 27 | dirt to mud | base | 87 | random gems | d105 |
| 28 | silt | d102 | 88 | moss grass | d105 |
| 29 | ash blobs | base | 89 | muds walls in jungle | base |
| 30 | ores and gems | base | 90 | larva | c31 |
| 31 | webs | d102 | 91 | floating islands | g44 |
| 32 | evil | base | 92 | settle liquids | f25 |
| 33 | liquids | base | 93 | floating island houses | g44 |
| 34 | dither snake (gated) | f24 | 94 | quick cleanup | d104 |
| 35 | slush | d102 | 95 | remove water | base |
| 36 | dungeon | base | 96 | oasis | d103 |
| 37 | constant lava layer | base | 97 | shell piles | base |
| 38 | mountain caves | d102 | 98 | slopes | base |
| 39 | beaches | base | 99 | living trees | base |
| 40 | gravitating sand | base | 100 | grass wall | d106 |
| 41 | gems | d102 | 101 | guide | d105 |
| 42 | ocean caves | base | 102 | sunflowers | d105 |
| 43 | shimmer | g36 A1 | 103 | waterfalls | d103 |
| 44 | clean up dirt | d102 | 104 | wall variety | d103 |
| 45 | fragile ice | base | 105 | life crystals | d103 |
| 46 | underground jungle trees | base | 106 | wood tree walls | d102 |
| 47 | exposed gems | base | 107 | trees | base+g42 |
| 48 | jungle shrines | base | 108 | herbs | d105 |
| 49 | chests | base | 109 | dye plants | d105 |
| 50 | surface chests | d103 | 110 | surface flora | base |
| 51 | water chests | d103 | 111 | settle liquids again | d106 |
| 52 | spider caves | base+g41 | 112 | cacti | base |
| 53 | gem caves | base | 113 | tile cleanup | d106 |
| 54 | moss | base | 114 | lihzahrd altar | base |
| 55 | altars | base | 115 | water plants | base+c33 |
| 56 | wet jungle | base | 116 | stalac | d106 |
| 57 | pyramids | c33 | 117 | remove broken traps | d106 |
| 58 | dirt rock wall runner | d102 | 118 | world tail | base |
| 59 | pots | base | 119 | seed finish | base |
| 60 | hellforge | d104 | | | |

Ordering notes (the deviations the witnesses must encode):
- **Chest family group** (g37 `809a61a8`): `chests` (Jungle Chests Placement)
  → `surface chests` → `water chests` → `spider caves`. The port's `chests`
  step sits early; the isolated per-pass streams make the grouping
  RNG-neutral. d103's raw-order insert (statues→surface chests→water
  chests→cave walls) is the C# order but breaks the port's DW-5-6 spider-caves
  adjacency; the merged chain keeps g37's grouping. `test_DW_5_6` reads
  `spider caves == water chests + 1`.
- **Statues/cave-walls group**: g37 puts `cave walls`+`statues` after
  `temple part 2`, before `cave houses`; d103 puts them after `life crystals`.
  The merged chain keeps the temple-family group (the isolated streams make
  it RNG-neutral); `statues + 1 == cave houses` holds.
- **Island houses + quick cleanup**: after `floating islands` (g44 producer),
  before `remove water` — g37's slot. d104's between-living-trees-and-trees
  slot is the non-vanilla one 88 §2.11 flags; the merged chain uses g37's.
- **Hellforge/spreading grass/surface ore**: g37's after-`pots` group (before
  `beehives`) vs d104's after-`quick cleanup`. Both are RNG-neutral (own
  streams); the merged chain keeps g37's pots-adjacent group so the
  `hellforge follows pots` / `spreading grass follows hellforge` /
  `surface ore follows spreading grass` witnesses hold. d104's fallen
  log/traps/piles/spawn point follow `surface ore and stone` (the C# order).
- **Post-living-trees tail group** (grass wall / guide / sunflowers /
  waterfalls / wall variety / life crystals / wood tree walls / herbs / dye
  plants): every one of these draws its own `pass_rng` stream or no RNG, so
  their internal order is RNG-neutral — the table shows one valid insertion
  order (the union algorithm's), and the Lead may reorder within the group.
  The load-bearing constraints are the chest-family, statues/cave-walls and
  island-houses/quick-cleanup groups above, plus `trees` after `wood tree
  walls` and `herbs`/`dye plants` after `trees` (the d105 plant-pass order).
- **B-pass scope (the "75+4" reconcile):** g37's 75-step chain already carries
  B1–B9. Against d103 the swap is B1–B5 → d103's statues/buried_chest/
  water_chests/cave_walls (+ g44 houses), and d103 adds oasis/waterfalls/
  wall variety/life crystals (4 steps g37 lacks). Against d104 the swap is
  B6–B9 → d104's quick cleanup/hellforge/spreading grass/surface ore, and d104
  adds place fallen log/traps/piles/spawn point (the 4 steps past g37's
  range). The full 119-step union above folds in every other lane too.
- **Gaps** (88 §6.1, still open at these tips): Random Gems + Moss Grass
  landed on d105 `9f7b1a24`/`d590d99a` AFTER 88 was written — 88's "GAP" rows
  97/98 are now CLOSED (d105 #43/#44). **Final Cleanup** (d106 R6) remains the
  only unlanded classic pass.

## 5. g39 helper-dup resolution

g39 `place_helpers.rs` (H1–H15, unwired) is the canonical PlaceTile-wrapper
module (88 §3.1). The g37/d103/d104 pass-local slices resolve as:

| helper | g37 local | d103 local | d104 local | g39 canonical | resolution |
| --- | --- | --- | --- | --- | --- |
| Place2xX (`:39663-39722`) | `place_2xx` (105 jug/55-wrap/207-4tall) | `place_2xx_statue` | `place_2x_x` (207 arm) | `place_2x_x` (105/34→349, 55-wrap, 104-5tall, 207-4tall) | **g39** — all three slices DROP; g39 carries every arm |
| Place1x1 (`:45188-45235`) | `place_1x1` (135 plate) | `place_1x1_plate` | — | `place_1x1` | **g39** |
| Place3x2 (`:52066-52196`) | `place_3x2` | `place_3x2` | `evil::place_3x2` generalised | `place_3x2` (H6: picnic/campfire/285-family/186-187/26 vetoes) | **g39** — d104's type-param generalisation must survive (88 §6.3 check) |
| PlaceSmallPile (`:46815-46873`) | `place_small_pile` | — | `place_small_pile` (+type param) | (spider_gem_moss.rs existing) | keep the **type-parameterised** version (d104's); g37's DROP |
| Place1x2 (`:46652-46679`) | `place_1x2` | — | `place_1x2` | `place_1x2` (H1) | **g39** |
| PlaceDoor / PlaceTile-door | `place_door`/`place_door_branch` | `place_door`/`place_tile_door` | — | (not in H-list) | keep d103's `place_tile_door` up/down dispatch (g44 has it too); reconcile at the gate |
| AddBuriedChest core | `add_buried_chest` (half) | `buried_chest.rs` (full) | `add_sky_chest` (compact) | — | **d103 full core** (§2.2); g37/c33/g44/d104 calls re-point |
| PlaceChest | `place_chest_at` (TileObject route) | `structures::place_chest` (compact) | `structures::place_chest` | — | **graft g37's TileObject route** under d103's core (§2.2) |
| SolidTile2 variants | `solid_tile2_of` | `solid_tile2` | `solid_tile2` | H15 five floor variants | **g39** for the floor variants; the Tile-overload `solid_tile2_of` stays with its caller |

g39 is unwired by design; the merger wires each kept pass onto it. Where a
kept pass's local slice has a draw-schedule difference, the pass keeps its
slice and g39 serves the other callers (record the exception in PORT_MAP).
**f25's NEW C1–C7 helper lane** (`f9c3dba7`: Convert types 2/5/6/7/8/9/10,
TileIsExposedToAir, BlockBelowMakesSandConvertIntoHardenedSand,
IsTileATreeBranch/Root, ConvertTreeAndGround) is a DIFFERENT helper family
(the Convert/growth side, not PlaceTile) — no overlap with g39's H-list; both
keep. d104's `spread_evil_grass` repeat-param (§2.8) composes with f25's
Convert ports (the grass conversion rides `spread::convert_*`).

## 6. Witness union (what the gate runs)

Every lane wrote witnesses NOT run (code-only). The gate runs the union;
duplicate a==b halves collapse to one per kept body.

| kept body | witnesses to run | source |
| --- | --- | --- |
| d103 statues | 7 module + the pipeline positional | d103 `statues.rs` |
| d103 buried_chest core + surface chests | 7 module + positional | d103 `buried_chest.rs` |
| d103 water chests | 3 module + positional | d103 `water_chests.rs` |
| d103 cave walls | 7 module (baseline-wall fixture `8be04cb0`) + positional | d103 `cave_walls.rs` |
| g44 islands + houses | 10 module + positional | g44 `islands.rs` |
| d104 quick cleanup | 2 module (10 in file) + positional | d104 `micro_finish.rs` |
| d104 hellforge | 1 module + positional | d104 `structures.rs` |
| d104 spreading grass | 2 module + positional | d104 `surface.rs` |
| d104 surface ore | 1 module + positional | d104 `surface.rs` |
| g37 grafts | B7 PlaceTile-route witness (port onto d104 hellforge), B3 inner-cap witness (port onto d103 water chests) | g37 `passes_b.rs` (written, the gate ports) |
| g37 slot witnesses | the 9 `w4_g37_*` positional asserts — re-point onto the kept owners' slots (the §4 order keeps them valid) | g37 `pipeline.rs` |
| pipeline count/order | `the_executed_pipeline_is_59_named_steps` → **119** (the §4 union); `test_DW_5_6` spider-caves adjacency; the VANILLA_CHAIN 106-classic test | merged `pipeline.rs` |
| hash pins | `SEED42_512` + `DEFAULT_WORLD` re-measured ONCE on the merged chain; the `a==b` halves stay green throughout | `world_hash.rs` |

**Hash-pin discipline:** every lane's "pins unchanged/stale-by-design" note is
an argument about an ISOLATED insertion — they do not compose. g37 B1's
512-beach-band proof, d104's sub-vanilla width guards, d103's size guards are
all isolated-insertion arguments; only the gate's measured re-pin on the full
119-step chain is authoritative (88 §6.2). The `a == b` determinism halves of
both pin tests are untouched on every lane and must stay green.

## 7. Cross-dup census (the other lanes' pushed tips)

Beyond the nine g37 passes, the pushed tips carry these cross-dups (all
resolved in 88; confirmed here against the tips):

- **SkyblockIsland (f24) vs g44 islands** — NOT a dup. f24's `SkyblockIsland`
  (`:22818-23556`) is the skyblock-seed generator (14-style palette, gated
  under SKYBLOCK); g44's `FloatingIslands` (`:12927-13122`) is the classic
  floating-islands pass. Different passes, different seeds — both keep.
- **Hellforge (g37 B7 / d104) vs d106** — NOT a dup. d106 ports Grass Wall /
  Settle Liquids Again / Tile Cleanup / Stalac / Remove Broken Traps; no
  Hellforge. The only Hellforge pair is g37/d104 (§2.7).
- **Water Chests THREE-way** (d103/g37/c33) — §2.3, KEEP d103.
- **Island Houses FOUR-way** (d103/d104/g37/g44) — §2.5, KEEP g44.
- **Shimmer / Clean Up Dirt / Dirt Runner / Wood Walls** (d102 vs g36 A1–A4)
  — 88 §2.1–2.4: KEEP g36 A1 Shimmer, d102 for the other three.
- **Wall Variety / Life Crystals** (d103 vs g36 A6/A7) — 88 §2.5–2.6: KEEP
  d103 (rewire shimmerPosition onto g36 A1's published centre).
- **Place Fallen Log** (d104 vs g38 C1) — 88 §2.17: KEEP d104 (port g38's 3
  witnesses).
- **Dither Snake** (f24 vs c30) — 88 §2.18: KEEP f24 (verify 3-seam gate).
- **Settle Liquids machinery** (f25 vs d106 R2) — 88 §3.3: KEEP f25 core,
  d106 R2 pass re-based.
- **Stalactite validator chain** (g36/g40/g41/d106 R4) — 88 §3.2: KEEP g41
  body + d106 generalisation.
- **Liquid-settle / Convert helpers** (f25 C1–C7) — new, no dup with g39's
  PlaceTile family (§5).

**No code changes were made to any other lane.** This doc is the g37
reconcile-phase assist: the arm-level evidence under 88's keep-one decisions,
the executed-step union, and the two refinements (B7 PlaceTile route, B9
ore-tier params) the Lead should weigh at the gate.
