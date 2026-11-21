# 137 — WLD FINAL: the full SaveWorldFlags/LoadWorldFlags walk audit (W4-f45)

**Branch:** `task/W4-f45-wld-header-2` (final phase; base `6352d3840`, gate-census
`2e13cfafc`, boundary fixup `ad153feb3`). **Mode:** code-only — no
cargo/tests/rustfmt/graphify invoked; witnesses WRITTEN NOT RUN. **Scope:** the
Lead's four final items — (1) the FULL flags-section walk audit + complete
ownership table, (2) the g90 combatBookVolTwo/satchel co-claim adjudication,
(3) the g71 v18-union cross-read boundary, (4) the `.wld` VERSION-gate census.

**Method:** `decompiled/Terraria/IO/WorldFile.cs` `SaveWorldFlags:1238-1430`
(write) and `LoadWorldFlags:1981-2517` (read) walked line-by-line against
`crates/tr-core/src/wld/sections/flags.rs` (write `:33-295` + helpers, read
`:358-815`), plus the delegated bodies (`BannerSystem.Save/Load:172-212`,
`DD2Event.Save/Load:151-181`, `TreeTopsInfo.Save/Load:47-68`,
`ExtraSpawnPointManager.Write/Read:436-456`, `FixEndlessRainWorlds:3284-3298`,
`SetTempToOngoing:1040-1066` / `SetOngoingToTemps:152-179`,
`Main.FakeLoadInvasionStart:64488-64514`, `Main.UpdateTimeRate:6199-6224`).
Cross-branch reads: `origin/task/W4-g90-artisan-bread-use` tip `73367e0af`
(the `bb78974a9` lane-7 claim), `origin/task/W4-g71-bg-layers-2` tip `e2d085ae4`
(research/92 SEAL), `origin/task/W4-f39-weather-slimerain` tip `f1a168776`,
`origin/task/W4-c44-golden-wld-fixture` tip `a96fa7a5d`, g70 `752655914`,
g72 `1f5c2b1ee` (the g71 doc reads its newer `4a75715e` seat).

**Headline verdicts:**

| # | Item | Verdict |
| --- | --- | --- |
| 1 | Full walk audit | **FAITHFUL** — every field covered (owned or C#-default), every gate matched; 4 gate defects FOUND + FIXED this phase (§4); 7 recorded observations (§5), none a byte-walk defect |
| 2 | g90 booster co-claim | **.wld halves BYTE-IDENTICAL → trivial keep-one** (§2); the `.trwld` v18 tails DIVERGE (2 B books vs 44 B identity) → the g71 §1-FULL 107-byte union adjudicates, NOT a byte fight |
| 3 | g71 cross-read | **NO OVERLAP** on the disjoint setBG seats; the shared fields are byte-identical dedups; g71's research/92 §1-SEAL.2 already compose-audited THIS branch (§3) — cross-verified, agree |
| 4 | VERSION gates | **4 fixes landed** (`2e13cfafc` + `ad153feb3`): >326 reject, <88 lower boundary (off-by-one), flags_tail legacy-u32 window, FixEndlessRainWorlds (§4) |

---

## 1. The complete ownership table

Every `SaveWorldFlags` field in C# write order. **W** = `WorldFile.cs` write
line, **R** = read line, **gate** = the read version gate (— = unconditional).
**Owner**: `pre` = owned before f45 (the S1-J/M5 base), `PhN` = f45 phase N,
lane id = an in-flight lane's claim, `UNOWNED` = written at the C# default /
skip-read. "C# default" columns give the unowned write value.

### 1.1 The head (flavour, times, zone tables, spawn, boss flags)

| Field | W | R | Gate | Owner | Notes |
| --- | --- | --- | --- | --- | --- |
| `Main.GameMode` i32 | 1240 | 1985 | 209 | **Ph5** | pre-209: expert bool ≥112 (`:2031`), v208 master byte (`:2037`), classic below — all ported |
| 9 flavour bools (drunk/getGood/anniversary/dontStarve/notTheBees/remix/noTraps/zenith/skyblock) | 1241-1249 | 1986-2024 | 222/227/238/239/241/249/266/267/302 | pre (M5-SEED) | `World.seeds`; pre-267 zenith inference `:2020` — see §5.1 |
| `CreationTime` i64 | 1250 | 2044 | 141 | pre (c43) | `DateTime.ToBinary`; the clock seam's Utc stand-in recorded |
| `LastPlayed` i64 | 1251 | 2048 | 284 | pre (c43) | save-time `UtcNow` stamp |
| `moonType` u8 | 1252 | 2050 | — | pre | read `.min(8)` guard (§5.3) |
| `treeX[0..3]` 3×i32 | 1253-1255 | 2051-2053 | — | **Ph6** | g70/g71 co-claim the same wire (§3) |
| `treeStyle[0..4]` 4×i32 | 1256-1259 | 2054-2057 | — | **Ph6** | g71 co-claim, byte-identical; **g70's copy writes `forest_bg_styles` here — WRONG field (§3.3)** |
| `caveBackX[0..3]` 3×i32 | 1260-1262 | 2058-2060 | — | **Ph6** | g72/g71 co-claim; `[i32;3]` vs `[i32;4]` shape — wire identical (§3.2) |
| `caveBackStyle[0..4]` 4×i32 | 1263-1266 | 2061-2064 | — | **Ph6** | g72/g71 co-claim, byte-identical |
| `iceBackStyle` i32 | 1267 | 2065 | — | **Ph6** | g71/g72 u8-field co-claim — wire identical (§3.2) |
| `jungleBackStyle` i32 | 1268 | 2066 | — | **Ph6** | same |
| `hellBackStyle` i32 | 1269 | 2067 | — | pre | u8 field, i32 wire — all branches agree |
| `spawnTileX/Y` 2×i32 | 1270-1271 | 2068-2069 | — | pre | `spawnTileY` keeps the file value (the old `compute_spawn_y` override removed, S1-J-3b) |
| `worldSurface`/`rockLayer` 2×f64 | 1272-1273 | 2070-2071 | — | pre | |
| `_tempTime` f64 / `_tempDayTime` bool / `_tempMoonPhase` i32 / `_tempBloodMoon` / `_tempEclipse` | 1274-1278 | 2072-2077 | — | pre | `SetOngoingToTemps:154-158` restores; moonPhase read `&7` guard (§5.3) |
| `dungeonX/Y` 2×i32 | 1279-1280 | 2078-2079 | — | **Ph3** | gen stamps (`step_dungeon`) |
| `WorldGen.crimson` bool | 1281 | 2080 | — | **Ph3** | g69 co-claims the READ (its `RollRareCloud` type-36 gate); write+read ONCE at merge (g71 §1-SEAL.2, g69 research/117 §1) |
| 18 downed/saved bools (`downedBoss1/2/3`, queenBee, mech1/2/3, mechAny, plant, golem, slimeKing, savedGoblin/Wizard/Mech, downedGoblins/Clown/Frost/Pirates) | 1282-1299 | 2081-2101 | slimeKing 118 | pre + **Ph1** (clown) | `downedMechBossAny` DERIVED write / skip read (§5.2); Ph1 adds the clown latch (kind unported, round-trips) |

### 1.2 The middle (orbs, invasion, weather, ores, setBG, clouds, angler, banners)

| Field | W | R | Gate | Owner | Notes |
| --- | --- | --- | --- | --- | --- |
| `shadowOrbSmashed`/`spawnMeteor` bools, `shadowOrbCount` u8, `altarCount` i32, `hardMode` bool | 1300-1304 | 2102-2106 | — | pre | orb count byte-truncated both sides |
| `afterPartyOfDoom` bool | 1305 | 2109 | 257 | **Ph8** | gen Reset arm + the doom-party consumer (`Main.cs:17926-17943`) |
| `invasionDelay/Size/Type` 3×i32 + `invasionX` f64 | 1306-1309 | 2115-2118 | — | pre | |
| `slimeRainTime` f64 | 1310 | 2121 | 118 | pre | f39 re-ports the event machine; the field pre-exists |
| `sundialCooldown` u8 | 1311 | 2125 | 113 | pre | |
| `_tempRaining` bool / `_tempRainTime` i32 / `_tempMaxRain` f32 | 1312-1314 | 2127-2129 | — | pre (S1-J-7) | **`FixEndlessRainWorlds:2130` now ported (§4.3)**; `cloudAlpha := maxRaining` restore (`rain.rs::restore_saved` sets `intensity`) |
| `SavedOreTiers.Cobalt/Mythril/Adamantite` 3×i32 | 1315-1317 | 2131-2133 | — | pre | |
| `setBG(0)` treeBG1 u8 | 1318 | 2134 | — | UNOWNED → **g70** | write 0 / skip; g70 `forest_bg_styles[0]` |
| `setBG(1)` corruptBG u8 | 1319 | 2135 | — | pre | tail 89 |
| `setBG(2)` jungleBG u8 | 1320 | 2136 | — | pre | tail 90 |
| `setBG(3)` snowBG u8 | 1321 | 2137 | — | UNOWNED → **g71** | write 0 / skip; tail 91 |
| `setBG(4)` hallowBG u8 | 1322 | 2138 | — | pre | tail 92 |
| `setBG(5)` crimsonBG u8 | 1323 | 2139 | — | pre | tail 93 |
| `setBG(6)` desertBG u8 | 1324 | 2140 | — | UNOWNED → **g71** | tail 94 |
| `setBG(7)` oceanBG u8 | 1325 | 2141 | — | UNOWNED → **g71** | tail 95 |
| `cloudBGActive` i32 | 1326 | 2142 | — | **Ph7** | the C# RE-ROLLS at `:2144` (`-genRand.Next(8640,86400)`); the port keeps the stored mirror, the g69 live machine owns the re-roll (recorded deviation) |
| `numClouds` i16 | 1327 | 2145 | — | **Ph7** | short cast both sides; g69's `update_num_clouds` owns the live value |
| `windSpeedTarget` f32 | 1328 | 2146 | — | **Ph3** | `windSpeedCurrent := target` restore (`:2147`, `app/mod.rs:4130-4131`) |
| `anglerWhoFinishedToday` count + names | 1329-1333 | 2152-2156 | 95 (return) | **Ph4** | the real name list; C# `num>0` loop == port `max(0)` |
| `savedAngler` bool | 1334 | 2161 | 99 | pre | |
| `anglerQuest` i32 | 1335 | 2166 | 101 | pre | u8 field (the 0..40 pool, `anglerQuestItemNetIDs[41]`), i32 wire |
| `savedStylist` bool | 1336 | 2171 | 104 | pre | |
| `savedTaxCollector` bool | 1337 | 2174 | 129 | pre | |
| `savedGolfer` bool | 1338 | 2177 | 201 | pre | |
| `invasionSizeStart` i32 | 1339 | 2189 | 107 | pre | pre-107 `FakeLoadInvasionStart` re-derivation ported (Ph3 restored the gate; ladder 80/40, 120/60, 160/40 verified vs `Main.cs:64488-64514`) |
| `_tempCultistDelay` i32 | 1340 | 2197 | 108 | **Ph3** | pre-108 default 86400 (`CultistRitual.delayStart`) |
| `BannerSystem.Save`: killCount i16+293×i32 | 1341 | 2203 | 109 (return) | **Ph4** | densified from `App.banners`; C# reads ALL count entries, stores `i<293` — port identical |
| `BannerSystem.Save`: claimableBanners i16+293×u16 | 1341 | 2203 | 289 | **UNOWNED** (c44 fixture) | write 0 / skip; the port spawns the banner item at the death spot (`npcs.rs:5354-5361`), no claim queue (§1.5) |
| `fastForwardTimeToDawn` bool | 1342 | 2208 | 128 | pre | |

### 1.3 The event bosses, towers, party, sandstorm, DD2

| Field | W | R | Gate | Owner | Notes |
| --- | --- | --- | --- | --- | --- |
| `downedFishron/Martians/AncientCultist/Moonlord` 4 bools | 1343-1346 | 2213-2216 | 131 (return) | pre | fishron S1-D-3 W-9 |
| 5 seasonal downeds (halloweenKing/Tree, christmasIceQueen/Santank/Tree) | 1347-1351 | 2217-2221 | 131 | **Ph1** | the moon-gated death arms (`NPC.cs:79210-79239`) + the 5/20 lantern default arm; c39/f39 co-claim the `.trwld` bytes (union dedup) |
| 4 tower downeds + 4 tower actives + `LunarApocalypseIsUp` (9 bools) | 1352-1360 | 2226-2234 | 140 (return) | pre | active towers restore at `ShieldStrengthTowerMax` (`NPC.cs:6795-6806`: normal 100, halved post-Moon-Lord) — port `:604-623` faithful |
| `_tempPartyManual/Genuine` bools + `_tempPartyCooldown` i32 + celebrant count/list | 1361-1368 | 2260-2268 | 170 | **Ph3** | pre-170 reset arm == World defaults; the celebrant slots carry the C# stable `Main.npc` indices (the compacted-vec remap is the recorded `world_events.rs` deviation) |
| `_tempSandstorm*` (bool + i32 + 2×f32) | 1369-1372 | 2279-2282 | 174 | pre (S1-J-7) | pre-174 zero reset == defaults |
| `savedBartender` bool | 1373 | DD2Event.Load | 178 | **Ph3** | rescue path unported, the latch round-trips |
| `DD2Event.Save` 3 tier bools | 1374 | DD2Event.Load:158-181 | 178 | pre (S1-E-12c) | pre-178 `ResetProgressEntirely`'s Ongoing/ArenaHitbox are App-side, not flags bytes |

### 1.4 The tail (styles, books, lanterns, tree-tops, seasons, ores, pets, unlocks, slimes, seeds, starfall, spawns, manifest)

| Field | W | R | Gate | Owner | Notes |
| --- | --- | --- | --- | --- | --- |
| `setBG(8)` mushroomBG u8 | 1375 | 2287 | >194 | pre | else `setBG(8,0)` == default |
| `setBG(9)` underworldBG u8 | 1376 | 2295 | 215 | UNOWNED → **g71** | else `setBG(9,0)` == default |
| `setBG(10..12)` treeBG2-4 3×u8 | 1377-1379 | 2303-2305 | >195 | UNOWNED → **g70** | **pre-196 else copies treeBG1 into all three (`:2309-2311`); the port leaves 0 — §5.4, g70 lane** |
| `combatBookWasUsed` bool | 1380 | 2315 | 204 | pre (i35) | |
| `_tempLanternNight*` (i32 + 3 bools) | 1381-1384 | 2326-2329 | 207 | **Ph3** | pre-207 reset == defaults |
| `TreeTops.Save` count + 13×i32 | 1385 | TreeTopsInfo.Load:56-68 | 211 | **UNOWNED** (c44 fixture) | write 13 zeros / skip; the table is a DERIVED mirror of treeStyle[0..4]+setBG (`CopyExistingWorldInfo:222-237`) — the sources are owned; §5.5 |
| `forceHalloweenForToday`/`forceXMasForToday` 2 bools | 1386-1387 | 2334-2335 | 212 | UNOWNED → **f39** | f39's tip owns `world.force_*_for_today` (the `CheckForMoonEventsStartingTemporarySeasons` wave-15 arms, `Main.cs:10901-10937`); at merge f39's live write+read supersedes the false/skip |
| `SavedOreTiers.Copper/Iron/Silver/Gold` 4×i32 | 1388-1391 | 2344-2347 | 216 | pre | pre-216 −1 quartet + `CheckSavedOreTiers:794-849` post-load re-derivation ported |
| `boughtCat/Dog/Bunny` 3 bools | 1392-1394 | 2358-2360 | 217 | **Ph11** | f43 license consumers; g87 co-claims the `.trwld` bytes (union dedup) |
| `downedEmpressOfLight` bool | 1395 | 2370 | 223 | **Ph1** | d125 co-claims the live wiring + `.trwld` byte (union dedup) |
| `downedQueenSlime` bool | 1396 | 2371 | 223 | pre (S1-F-16) | |
| `downedDeerclops` bool | 1397 | 2380 | 240 | pre | |
| `unlockedSlimeBlueSpawn` bool | 1398 | 2388 | 250 | **Ph4** | King Slime death (`NPC.cs:79411`) |
| 8 arrival unlocks (merchant/demolitionist/partyGirl/dyeTrader/truffle/armsDealer/nurse/princess) | 1399-1406 | 2396-2403 | 251 | **Ph4** (6) + pre (partyGirl/princess) | arrival setters `WorldGen.cs:5577-5608` + `SpawnAllowed_*` overrides |
| `combatBookVolumeTwoWasUsed` bool | 1407 | 2418 | 259 | **Ph12** | **g90 co-claim — byte-identical (§2)** |
| `peddlersSatchelWasUsed` bool | 1408 | 2426 | 260 | **Ph12** | **g90 co-claim — byte-identical (§2)** |
| 7 slime unlocks (green/old/purple/rainbow/red/yellow/copper) | 1409-1415 | 2434-2440 | 261 | **Ph13** (5) + pre (rainbow/red) | town-slime kinds unported, latches round-trip; the doom party clears green (`Main.cs:17940`) |
| `fastForwardTimeToDusk` bool + `moondialCooldown` u8 | 1416-1417 | 2454-2455 | 264 | pre | |
| `forceHalloweenForever`/`forceXMasForever` 2 bools | 1418-1419 | 2464-2465 | 287 | **Ph2** | `micro_seeds` ENDLESS_HALLOWEEN/CHRISTMAS mirrors (`WorldGen.cs:571-578`) |
| `vampireSeed` bool | 1420 | 2474 | 288 | **Ph2** | VAMPIRISM mirror |
| `infectedSeed` bool | 1421 | 2482 | 296 | **Ph2** | WORLD_IS_INFECTED mirror; **the 296 gate sits BEFORE 291 in the C# — the port preserves the out-of-order walk** |
| `_tempMeteorShowerCount`/`_tempCoinRain` 2×i32 | 1422-1423 | 2490-2491 | 291 | **Ph9** | `StartMeteorShower` `Next(650,751)*4` + the failed-pick `Next(3)` arm; `StartRain`/`StopRain` store/clear coinRain |
| `teamBasedSpawnsSeed` bool | 1424 | 2500 | 297 | **Ph2** | TEAM_BASED_SPAWNS mirror |
| `ExtraSpawnPointManager.Write` count u8 + pairs (i16,i16) | 1425 | 2501 | 297 | **Ph10** | c46 respawn consumers; pre-297 `Clear()` == empty default |
| `dualDungeonsSeed` bool | 1426 | 2508 | 304 | **Ph2** | inline `version >= 304 && read` — port matches |
| `moreLightningSeed`/`noLightningSeed` 2 bools | 1427-1428 | 2509-2510 | 323 | **Ph2** | inline gates |
| *(no bytes)* `Main.UpdateTimeRate()` | — | 2511 | — | App clock seam | §5.6 — a side-effect call, not a field |
| *(legacy u32, read-only)* | — | 2512-2515 | 299..312 | skip | **the flags_tail table gap FIXED (§4.2)** |
| `WorldGen.Manifest.Serialize()` string | 1429 | 2516 | 299 | **UNOWNED** (c44 fixture) | write empty string / skip; §5.7 — the C# writes the null-field JSON even for an empty manifest |

### 1.5 The remaining-unowned set (the census answer)

**Five families, nothing else.** Every other `SaveWorldFlags` field is owned
(the table above). The unowned write values are the C# defaults except the
manifest (§5.7):

1. **`claimableBanners`** (293×u16, v289+) — the port spawns the banner item at
   the death spot (`world_sim/npcs.rs:5354-5361`, the M2-BUFF-8 milestone) and
   has no claim queue/UI (`BannerSystem.RequestBannerClaim:280-305` unported).
   The kill tally IS owned (Ph4). c44's golden fixture carries the real
   C# payload (50 AnglerFish kills → 1 claimable).
2. **`TreeTops._variations`** (13×i32, v211+) — a derived mirror
   (`CopyExistingWorldInfo:222-237`: [0..4]=treeStyle, [4..13]=the nine setBG
   styles); the sources are owned (Ph6 + g70/g71), the table itself has no port
   state (the foliage draw approximates style 0 — the recorded tree.rs
   deviation). c44 fixture owns the payload.
3. **`WorldGen.Manifest`** (string, v299+) — the port has no gen-pass metadata;
   writes the empty .NET string (deviation §5.7). c44 fixture pins the real
   `{"GenPassResults":[],"Version":null,"GitSHA":null,"FinalHash":null}` shape.
4. **`forceHalloweenForToday`/`forceXMasForToday`** (2 bools, v212+) — the f39
   season lane owns them at its tip (`world.force_*_for_today` + the wave-15
   temporary-season arms); f45 writes false / skip-reads until that merge.
5. **The eight setBG seats** — 0 (treeBG1), 3 (snow), 6 (desert), 7 (ocean),
   9 (underworld), 10-12 (treeBG2-4): g70 owns the forest four, g71 owns the
   surface three + underworld at their tips; f45 writes 0 / skip-reads until
   those merges (the compose is g71 research/92 §1-SEAL.2, cross-verified §3).

---

## 2. The g90 combatBookVolTwo/satchel co-claim — KEEP-ONE verdict

**Claimants:** f45 `b8ad4a469` (Ph12) vs g90 `bb78974a9` (lane 7 1/4, tip
`73367e0af`). Both land `NPC.combatBookVolumeTwoWasUsed` (`NPC.cs:6341`) +
`NPC.peddlersSatchelWasUsed` (`NPC.cs:6343`) as World latches.

### 2.1 The `.wld` halves — BYTE-IDENTICAL → trivial keep-one

| Axis | f45 `b8ad4a469` | g90 `bb78974a9` | Match |
| --- | --- | --- | --- |
| World fields | `combat_book_volume_two_was_used` / `peddlers_satchel_was_used` (bool, default false) | identical names/types/defaults | ✓ |
| write | `w.boolean(world.…)` at `:1407`/`:1408` seats (after princess, before green slime) | identical | ✓ |
| read | `world.… = r.bool()?` under `version >= 259` / `>= 260` | identical | ✓ |
| tail offsets | 2036 / 2037 (the `flags_tail` running widths) | 2036 / 2037 | ✓ |
| gate witness | v258 neither / v259 book only / v260 satchel; fixed pins `tail_at+2036` / `+2037` | v258/259/260; `flags_tail(v).len()-1` pins | ✓ same bytes (len−1 == 2036 at v259, == 2037 at v260) |
| round-trip sweep | `owned_world` sets both true; `assert_owned_fields` compares both | identical | ✓ |

**C# anchor check:** write `:1407-1408`; read gates `:2416-2418` (v259) /
`:2424-2426` (v260). g90's commit message cites "`WorldFile.cs:2404-2405`" —
those lines are the unlockedPrincess else-arm, NOT the book gates (a citation
slip, the code itself is correct). f45 cites `:2418`/`:2426` (the exact read
lines).

**One table difference, not a wire difference:** f45's `expected_flags_block`
pins `tail[2036+d]`/`tail[2037+d]` (d = 4×celebrant-count, the Ph3
variable-length splice); g90's pins `tail[2036]`/`tail[2037]` fixed (its base
predates Ph3 — the party snapshot was unowned, all lists empty, d=0). Both are
correct for their own `owned_world`; f45's variable-length table is the
superset (the C# block IS variable-length). **Merge: keep f45's flags.rs +
table wholesale; g90's flags.rs hunks are a strict subset — drop as duplicates.**

### 2.2 The `.trwld` v18 tails — DIVERGENT, but NOT a byte fight

g90 appends a 2-byte books tail after the v17 fishron byte; f45 (inheriting
c43 `7cbfa5a04`) appends the 44-byte identity block at the same seat. Both bump
`VERSION` 17→18. These are two of the **13 claimant branches** in g71's
research/92 §1-FULL registry (the SEALED union authority): the books ride union
offsets [51..53], the identity [63..107], inside the 107-byte fixed head +
d109's dense variants LAST. **No exact-byte adjudication is needed between f45
and g90** — the union layout already places both blocks; the merge takes the
union writer/reader (g71 §1-FULL.2), not either branch's tail. f45's
`EMPTY_V18_TAIL = 44` and g90's `+2` truncation constants both move per g71
§1-FULL.4.

### 2.3 The consumer boundary

g90 lane 7 (3/4) owns the USE paths (`use_combat_book` 4382/5336,
`use_peddlers_satchel` 5343, the travel-shop +1 slot, the TRAINING_DAY hook)
and lane 7 (4/4) the net sync (WorldInfo bitsByte9[0] + bitsByte11[6/7],
`NetMessage.cs:334/:360-361`). f45 Ph12 owns the `.wld` persistence + the
latches; the g89b boosters lane co-claims the town-stat consumer
(`npc_ai_town.rs` combat_book_two env). No overlap: persistence (f45/g90 dedup)
vs use/net (g90) vs stat consumer (g89b).

---

## 3. The g71 v18-union cross-read — boundary note

**Read:** `origin/task/W4-g71-bg-layers-2` tip `e2d085ae4`,
`docs/research/92-doc-sweep-bg.md` (SEALED; §1-FULL is THE `.trwld` v18
union-layout authority, receipted by g88 103 / f44 132 / g72 64a+64b).

### 3.1 No overlap on the flags-block seats — CONFIRMED

The setBG 0..7 block sits at tail 88..95. f45 owns 89 (corrupt) / 90 (jungle) /
92 (hallow) / 93 (crimson); g71 owns 88 (treeBG1, with g70) / 91 (snow) /
94 (desert) / 95 (ocean). **Disjoint seats, zero shared bytes.** The later
block: g71 owns setBG(9) underworld + setBG(10-12) treeBG2-4 (f45 writes 0 /
skips). g71's §1-SEAL.2 compose table already audited THIS branch at
`6352d3840` and noted the then-unpushed gate census `2e13cfafc` as orthogonal
("load-gate fixes, no bg-byte interaction") — **cross-verified, agree**.

### 3.2 The shared fields — byte-identical wire, shape dedups

| Field | f45 | g71/g72 | Wire | Merge (g71 §1-SEAL.2) |
| --- | --- | --- | --- | --- |
| `tree_x` | `[i32;3]`, full loop | `[i32;3]`, full loop | 3×i32 identical | dedup |
| `tree_style` | `[i32;4]` live | `[i32;4]` live | 4×i32 identical | dedup |
| `cave_back_x` | `[i32;3]`, full loop | `[i32;4]` + `[..3]` slice | 3×i32 identical (C# `new int[4]`, `[3]` never persisted — `Main.cs:66696`, no `caveBackX[3]` reader anywhere) | keep the `[i32;4]` shape + slice; f45's `[i32;3]` drops |
| `cave_back_style` | `[i32;4]` | `[i32;4]` | 4×i32 identical | dedup |
| ice/jungle | i32 fields, `w.i32` direct, full-i32 read | u8 fields, `as i32` casts, `as u8` trunc read | i32 identical (rolls 0..3 / 0..1 fit u8; the trunc differs only for foreign out-of-range values) | keep g72's u8 fields (the `.trwld` u8 widths) |
| hell | u8 field, i32 wire | u8 field, i32 wire | identical | dedup |
| setBG 1/2/4/5/8 | live | live | identical | dedup, keep one |
| crimson | live write+read (Ph3) | g71 false/skip stand-ins; g69 reads | identical seat | f45 write+read ONCE; g69's read folds onto it; g71 stand-ins drop |

### 3.3 The g70 treeStyle conflation — a REAL divergence to flag at merge

g70's flags.rs writes `world.forest_bg_styles` (the treeBG1..4 setBG style
bytes, 0..14 range) into the **`treeStyle` i32 slots** (`:1256-1259`) and reads
them back into `forest_bg_styles`. The C# keeps TWO separate arrays:
`Main.treeStyle[4]` (the tree-top variations, `Next(6)` rolls, copied into
`TreeTops._variations[0..4]`) and `WorldGen.treeBG1..4` (the forest-set styles,
`RollRandomForestBGStyle` 0..14/31/51/71-73). g70 has no `tree_style` field at
all. **f45 and g71 both carry the correct separate `tree_style: [i32;4]`**
(g71's world.rs doc explicitly notes "Distinct from `forest_bg_styles` … the
C# keeps them as two separate arrays"). At merge: f45/g71's `treeStyle`
write/read wins; g70's conflation must be corrected (its `forest_bg_styles`
still owns the setBG 0/10-12 seats — those hunks are fine).

### 3.4 The gen-roll keep-one — f45 Ph6's copies defer

g71 §2.1-FINAL (SEALED) adjudicated the roll duplicates: **keep g72's
`cave_bg.rs` cave driver** (the `[i32;4]` + previous-array carry + u8 tail +
the live draw/persist/net chain), **keep g71's `bg_rolls.rs`** for
tree/surface/boot (`randomize_tree_style`, `randomize_backgrounds` + helpers,
`is_background_considered_the_same`, `boot_reroll_backgrounds`). f45 Ph6 landed
its OWN `pipeline.rs::randomize_tree_style` + `randomize_cave_backgrounds`
(the frozen moon→hell order, draws on `streams.rng`) — a THIRD copy pair not in
the g71 registry (it predates their sweep). **At merge: drop f45's two roll
fns + their `step_world_tail` calls for the g71/g72 canonical seats** (tree on
`pass_rng("RandomizeTreeStyle")`, cave on shared `streams.rng` at g72's seat,
surface on `pass_rng("RandomizeBackgrounds")`, moon after — the §2.1-FINAL
order contract). The f45 Ph6 roll witness
(`w4_f45_the_tree_cave_rolls_follow_the_per_size_branches`) re-points at the
canonical fns; its per-size/distinctness assertions carry over (both copies
are draw-order-equivalent to `WorldGen.cs:8117-8249` — verified draw-by-draw:
bands → base styles in index order → rejection loops → 0→4 promotion (tree) /
ice-hell-jungle tail (cave); the batch-then-reject and interleave shapes
consume the identical sequence). The `.wld` flags-block fields (the arrays)
are unaffected — they dedup per §3.2.

### 3.5 The `.trwld` v18 boundary

f45's 44-byte identity tail is registry claim #12 (c43/f45 byte-identical,
one block at union [63..107]); g71's surface 3 B + underworld 1 B are #5/#6.
The union is forward-only (no released v18 exists); every branch's
truncation constants move per §1-FULL.4. f45's `EMPTY_V18_TAIL` separate
constant folds into the union's `EMPTY_V13_TAIL + 9 + 1 + 5 + 1 + 107`.

---

## 4. The VERSION-gate census — 4 defects found + fixed

The full read-gate list was walked against the C# (§1 tables): **every gate
matches** — 209/222/227/238/239/241/249/266/267/302 (flavour), 112/208 (pre-209
GameMode), 141/284 (times), 118 (slimeKing + slimeRain), 257 (afterParty), 113
(sundial), the 95/99/101/104/109/128/131/140 early returns, 129/201
(tax/golfer), 107/108 (invasionStart/cultistDelay + FakeLoadInvasionStart),
289 (claimable), 170/174/178 (party/sandstorm/DD2), >194/215/>195 (setBG
8/9/10-12), 204/207/211/212/216/217/223/240/250/251/259/260/261/264 (the tail
ladders), 287/288/296/291/297/304/323 (the out-of-order seed gates — the C#
reads 296 BEFORE 291; the port preserves it), 299..312 (legacy u32), 299
(manifest). The else-arm resets all match the World defaults (verified
field-by-field). **Four defects outside the gate list itself:**

### 4.1 The >326 later-version reject — FIXED (`2e13cfafc`)

C# `WorldFile.cs:712`: `num2 > 326 -> StatusID.LaterVersion` (the same guard
`GetAllMetadata:197`). The port walked a v327+ file as v326 and mis-parsed its
unknown tail. Now `SectionTable::open` bails `version > FORMAT_VERSION`.
Witness: `w4_f45_final_the_later_version_file_is_rejected` (v327 Err, v326 Ok).

### 4.2 The flags_tail legacy-u32 window — FIXED (`2e13cfafc`)

The reader skips the legacy u32 for 299..=312 (`:2512-2515`, port `:807-809`),
but the `flags_tail` TEST TABLE omitted the 4 bytes — a v299..312 synthetic
file mis-aligned at the manifest (the reader ate the manifest + 3 section
bytes as the u32). This broke the Ph2 v304 witness's own pin. Table now carries
`if (299..313).contains(&version) { z(&mut b, 4) }`; the v304 witness offset
re-pinned `len-1` → `len-6` (dual 1 + u32 4 + manifest 1). Witness:
`w4_f45_final_the_legacy_u32_window_walks` (v299 opens, v312 dual at len−6,
v313 dual at len−2 — the window closes).

### 4.3 FixEndlessRainWorlds — PORTED (`2e13cfafc`)

`WorldFile.cs:3284-3298`, called at `LoadWorldFlags:2130` right after the rain
triple: a v≤317 file whose saved rain ran ≥5184000 ticks (a year) gets its rain
snapshot cleared UNLESS the seed text carries the rainsForAYear secret
(`GetSecretSeedCodes` splits on '|', `WorldFileData.cs:199-206`; each code
through `SecretSeed.Check`'s normalise+hash, `WorldGen.cs:468-480`). The port
now retains the header seed text (`parse_table`) and runs
`fix_endless_rain_worlds(&mut world, version, &seed_text)` after the flags
walk — byte-equivalent placement (nothing between `:2130` and the walk's end
touches the triple). The secret check rides the existing
`seeds::check_input_for_secret_seed` (the golden-verified BCrypt walk).
**The rainsForAYear plaintext "bring a towel" was verified against the
registered code `xYBNU5Soje9VhQHNQXETDKbwlc+7XZau` (`WorldGen.cs:3032`)**
through an independent Python mirror of the game's non-NUL BCrypt walk
(tables parsed from `bcrypt.rs`, controls `pumpkinseason`→halloweenGen and
`arachnophobia`→noSpiderCaves both match). Witness:
`w4_f45_final_fix_endless_rain_worlds_clears_the_year_long_bug_rain` (v317
year-rain clears; 5183999 stays; v318 stays; the "bring a towel" seed-text
splice keeps its endless rain — pointers 1.. shifted +8).

### 4.4 The <87 lower-boundary off-by-one — FIXED (`ad153feb3`)

C# dispatch `:712`: `num2 > 87 -> LoadWorld_Version2`, else
`LoadWorld_Version1_Old_BeforeRelease88` (`:3423`, unported). **Version2 starts
at 88**; the port's `< 87` bail let a v87 file into the Version2 walk. Now
`< 88` bails the whole old-loader range (1..87); the mod.rs doc "87+" → "88+";
the `rejects_old_versions_and_broken_reads` witness gains the v87 boundary pin.

### 4.5 The GameMode pre-209 pattern — VERIFIED FAITHFUL

The brief's named pattern: `version >= 209` reads the i32; else `>= 112` reads
the expert bool (`:2031`), `< 112` classic (`:2035`), and `version == 208`
reads the master byte upgrading to 2 (`:2037-2040`). Port `flags.rs:360-401`
matches exactly, including the v208 byte being read ONLY at exactly 208 (not
≥208). The nine flavour bools each gate on their own introduction version
inside the 209 arm — matches. The pre-267 zenith inference is the one
widening (§5.1).

---

## 5. Recorded observations (not byte-walk defects)

### 5.1 The pre-267 zenith inference widens beyond the C# (pre-existing)

C# `:2020`: `Main.zenithWorld = Main.remixWorld && Main.drunkWorld` — a SINGLE
bool. The port (origin/main, inherited by Ph5) inserts
`ZENITH | EVERYTHING_DEPS` (8 bits: the 7 Everything deps incl FOR_THE_WORTHY).
For a pre-267 remix+drunk world whose getGood byte is FALSE, the C# applies the
zenith adjustments only (`NPC.cs:17839-17842`); the port also forces the
getGood/notTheBees/etc adjustments. A real getfixedboi file has all 7 dep bytes
set anyway (the inference is redundant there); the divergence bites only
partial remix+drunk mixes. **Pre-existing, not changed this phase** — narrowing
it would move the seeded-worldgen pins; recorded for the seed lane.

### 5.2 downedMechBossAny derived vs stored (equivalent for C# files)

The C# stores `NPC.downedMechBossAny` (set with each mech kill
`NPC.cs:79427/79470/79477`, reset only together with the three individual
flags `:95258-95261`). The port derives it on write (`b1||b2||b3`) and
skip-reads it. For any C#-written file `any == (b1||b2||b3)` holds (they are
never cleared individually), so the round-trip is exact; only a hand-edited
foreign file with any=1, b1=b2=b3=0 would diverge. Recorded, kept.

### 5.3 The moonType/moonPhase read guards (equivalent for valid files)

Port: `moon_type = r.u8()?.min(8)`, `moon_phase = (r.i32()? as u8) & 7`. C#
reads both unclamped (`Main.moonType` int, `Main.moonPhase` int). Valid files
carry 0..8 (`Next(9)`, `WorldGen.cs:7787`) and 0..7 (the dawn wrap
`Main.cs:65398-65401`), so the guards are no-ops; they only clamp foreign
out-of-range values (the C# would draw a garbage moon row / keep phase≥8 until
the first dawn). Pre-existing, kept as defensive reads.

### 5.4 The pre-196 treeBG2-4 else-arm (g70 lane)

C# `:2309-2311`: below v196, `setBG(10/11/12, WorldGen.treeBG1)` — the three
later forest sets COPY the first set's style. The port leaves them 0 (unowned,
g70's seats). For a pre-196 file with treeBG1≠0 the C# propagates it; the port
doesn't. g70's merge should carry the else-arm copy (its `forest_bg_styles`
ownership); recorded here so the compose doesn't lose it.

### 5.5 TreeTops.Load's min(count,13) vs the port's full-count skip

C# `TreeTopsInfo.Load:64-67` reads `min(num, 13)` entries (a num>13 foreign
file leaves the cursor after 13, not num). The port skips `num*4`. For any
C#-written file num==13 (Save writes `_variations.Length`), so the cursors
agree; the comment at `flags.rs:675-677` already records the nuance. Kept.

### 5.6 UpdateTimeRate's creative-fold ordering (clock seam, out of flags scope)

C# `UpdateTimeRate` (`Main.cs:6199-6224`) returns dayRate=60 during a dial
fast-forward BEFORE reading the creative powers — the creative multiplier is
ignored while fast-forwarding. The port splits the rate: `World::day_rate()`
(fast-forward 60 / asleep 5 / else 1) × `creative_powers.time_rate()` applied
in `frame/world.rs:357-367` — the creative multiplier still scales dt during a
fast-forward (and the asleep ×5 multiplies the slider rather than the C#'s
targetTimeRate base). A no-byte side-effect at `:2511`, so the flags walk is
unaffected; recorded for the clock/creative lane (the Journey powers'
fast-forward interaction).

### 5.7 The manifest empty-string write (the recorded deviation)

C# `SaveWorldFlags:1429` writes `WorldGen.Manifest.Serialize()` — even an EMPTY
manifest serializes to `{"GenPassResults":[],"Version":null,"GitSHA":null,
"FinalHash":null}` (c44's harness pins the exact JSON). The port writes the
empty .NET string (no gen metadata). The read side is cursor-faithful (skip
≥299, no read <299 == the C#'s `Deserialize("")`). The write deviation is
recorded in the flags.rs comment + ANALYSIS; c44's golden fixture owns the
payload comparison. Kept.

---

## 6. Witness offset-pins (written NOT run, code-only)

The final phase's witnesses, all in `crates/tr-core/src/wld/mod.rs`:

| Witness | Pins |
| --- | --- |
| `w4_f45_final_the_later_version_file_is_rejected` | v327 → Err; v326 → Ok |
| `w4_f45_final_the_legacy_u32_window_walks` | v299 loads; v312 dual byte at `flags_tail(312).len()-6`; v313 at `len-2`; below-323 lightning false |
| `w4_f45_final_fix_endless_rain_worlds_clears_the_year_long_bug_rain` | v317 rain triple at tail 67/68..72/72..76; 5184000 clears, 5183999 stays, v318 stays; the "bring a towel" header splice (seed string 6→14 bytes, pointers 1.. +8) keeps the rain |
| `rejects_old_versions_and_broken_reads` (extended) | v87 → Err (the new boundary), v86 → Err |
| `w4_f45_the_secret_seed_bytes_mirror_the_micro_seed_bits` (re-pinned) | the v304 dual byte at `flags_tail(304).len()-6` (was len−1, the table gap) |

The Ph1-13 offset pins (the `w4_f45_*` family) are unchanged — the gate fixes
touch the table's 299..312 window and the load entry, not the v326 owned
offsets. The booster pair pins (2036/2037 + d) match g90's (§2.1).

**Verification performed:** code-only — no cargo/tests/rustfmt/graphify run
(per the lane contract). The walk audit is a line-by-line C#-vs-Rust read
(every citation above re-read from the decompile at the cited lines); the
rainsForAYear plaintext was verified through the independent Python BCrypt
mirror (controls + target match); the cross-branch reads are `git show` at the
named tips. The compile window runs the witnesses.

## 7. Merge instructions (the f45 side)

1. **flags.rs is the `.wld` merge base** (g71 §1-SEAL.2, a6 138 §5, f44 132
   §3-L3 rule 3 all designate it). Apply g71's bg-ownership hunks (live setBG
   0/3/6/7/9/10-12 + the `[i32;4]` cave shape + u8 ice/jungle) on top; dedup
   the five shared live styles + the tree/cave arrays; g69's crimson read
   folds onto f45's write+read; f39's force-today supersedes the false/skip;
   g90's flags.rs hunks drop (strict subset).
2. **Correct g70's treeStyle conflation** (§3.3) — keep f45/g71's separate
   `tree_style` field; g70's `forest_bg_styles` keeps only the setBG seats.
3. **Drop f45 Ph6's two roll fns** for the g71/g72 canonical seats (§3.4);
   re-point the Ph6 roll witness.
4. **The `.trwld` v18 union** per g71 §1-FULL (107-byte head + dense LAST);
   f45's identity block is claim #12, the truncation constants move per
   §1-FULL.4.
5. **The gate fixes ride f45's file** (>326, <88, legacy-u32 table,
   FixEndlessRainWorlds) — no bg-byte interaction (g71 §1-SEAL.2 concurs).