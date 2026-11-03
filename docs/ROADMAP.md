# Roadmap & systems atlas — Terraria-Rust (1.4.5.8)

**Rewritten 2026-11-02** from a full audit of the decompiled sources: nine per-system
mechanics inventories with `File.cs:line` anchors (run on `decompiled/Terraria`, 1 550
files / ~642 000 lines). This file is both the **task catalog** (take tasks from the
bottom) and the **systems atlas** (use it to scope and order them). It supersedes the
2026-10-29 `docs/TERRARIA_SYSTEMS_CATALOG.md`, which is deleted — its content counts and
several entries were wrong (e.g. invented bosses, "550+ items" instead of ~6 020).

Anchors are the decompile's line numbers. They are **not** decorative: per the golden
rule (agents.md) every port re-reads the cited C# before writing Rust. Constants quoted
here are transcribed from those audits — re-verify against the source when porting.

## How tasks work (branch-per-task, unchanged)

- Tasks are taken from the **Task catalog** at the bottom (or a direct user request).
- `git fetch --prune`, check `git branch -a` — a catalog row is *taken* when a branch
  with its ID exists. Then `git checkout -b <Task-ID>-short-slug` from `main`.
- One branch = one task; merge back per the loop in agents.md (sync → verify → merge →
  delete → graphify refresh).
- Rows marked 🟡 list what is *still missing*; read the linked atlas section before
  starting. In-flight rows name the live branch.

## Legend

- ✅ done · 🟡 partial (missing items listed) · ⬜ not started · 🔄 in flight (branch exists).
- **Scale** = measured size in the decompile (lines / ID counts).
- **Hidden** bullets = behaviours easy to get wrong: thresholds, RNG streams, rounding,
  order dependence, one-off special cases. They are the fidelity traps.
- Task IDs are stable; branch name = `<Task-ID>-<slug>`.

## Content scale (measured 2026-11-02)

| Content | Count (from the decompiled ID classes) |
| --- | --- |
| Items | **6 016** const IDs (`ItemID.cs:1-14864`), `ItemID.Count`-gated ~6 200 |
| NPCs | **704** const IDs (`NPCID.cs`) — 1.4.5 renames BlueSlime=1, GreenSlime=-3 (`NPCID.cs:10671-10679`) |
| Projectiles | **1 043** const IDs (`ProjectileID.cs`) |
| Tiles / walls | **732** tile IDs (`TileID.cs`) / **293** wall IDs (`WallID.cs`) |
| Buffs | **373** IDs (`BuffID.cs`) |
| Mounts | **66** (`MountID.cs:189`) |
| Prefixes | **93** consts, `PrefixID.Count` = 98 (`PrefixID.cs:216`) |
| Emotes | **152** (`EmoteID.cs`) |
| Recipes | `Recipe.maxRecipes = 3700`, max 15 ingredients (`Recipe.cs:16776-16777`) |
| Chat/net | protocol **326**, messages 1-162 (`MessageBuffer.cs`), 12 NetModules |
| C# god files | NPC.cs 95 306 · WorldGen.cs 86 974 · Projectile.cs 79 791 · Main.cs 67 317 · Player.cs 57 151 · Item.cs 49 981 · Recipe.cs 16 784 · ItemID.cs 14 864 · NPCID.cs 12 584 · TileDrawing.cs 9 656 · Mount.cs 6 671 · TileObjectData.cs 5 328 · ItemSlot.cs 4 653 · Collision.cs 4 598 · MessageBuffer.cs 4 510 · PlayerDrawLayers.cs 4 309 |
| Namespaces | root `Terraria/*.cs` 78 files 509 k lines · `GameContent/` 788 files 54 k · `ID/` 61 files 47 k · `UI/` 49 files 14 k · `DataStructures/` 126 files 14 k · `Graphics/` 87 files 15 k |

The engine is the small part; **content behaviour is the project**. The estimates below
stay in agent-days (±50 %): a playable subset of M1–M3 ≈ 3–6 months of agent time,
broad parity 1.5–3 years, full 1:1 3+ years.

## Progress snapshot (2026-11-02)

| Area | Status | Notes |
| --- | --- | --- |
| Renderer (atlas, instancing, zoom, frame cache) | ✅ | 2 draw calls; G6 cache; zoom 1.0–2.0 |
| Autotiling / framing | ✅ (core) | generic chain + merge pairs + grass chain; TileFrameImportant multi-tile missing (M1.4-TO) |
| Lighting | ✅ (core) | scanner 147 cases + blur + 4 modes; sun/moon/star light missing |
| Liquids & waterfalls | ✅ (core) | sim + both draw passes + per-style waterfalls + shimmer draw; vertex distortion missing |
| Sky / weather visuals | 🟡 | clouds/stars/sun/rain/storm banks done; ambient creature skies, `HorizonClouds` tint, storm branch missing |
| World generation | 🟡 | compact native model (forest/snow/desert); vanilla pass pipeline not ported |
| Player | 🟡 | physics + survival + draw layers + item timings; no buffs/combat stats/dash/wings/hooks/mounts |
| Items | 🟡 | hotbar/mining/placing/drops + per-item use timings; no ammo/mana/equipment/recipes mechanics |
| Combat | 🟡 | contact damage, melee, hurt/respawn, King Slime; no defense/crit/damage-variance/banner rules |
| NPCs | 🟡 | slime colour family (8 variants + vanilla stats/tints/gel drops) + demon eye/Wandering Eye/zombie + King Slime; ~700 types, town system missing |
| Projectiles | ⬜ | shuriken first pass only; no engine (immunity/penetrate/minions) |
| Wiring / tile entities / shimmer / fishing / pylons | ⬜ | — |
| UI framework & screens | 🟡 | front-end char/world screens done; HUD partial; inventory UX partial; chest/creative/bestiary/emotes missing |
| Localization | 🟡 | core ported + tested, not wired into UI |
| Audio | 🟡 | 8 cues + XACT music engine + 3D attenuation; biome branches, ambience emitters missing |
| Networking | 🔄 | custom host/join flow; exact 1.4.5.8 protocol not started |
| Saves | 🟡 | `.trwld` round-trip, `.wld` header+tiles, `.map` v326; chests/signs/NPC sections skipped |

---

# Systems atlas

Every section: **C# ↔ Rust ↔ status**, what remains, and the hidden mechanics. Task IDs
referenced here exist in the catalog.

## 1. Engine loop, update order & determinism

**C#** `Main.cs` (67 317) · **Rust** `app/{loop,frame,mod}.rs` · 🟡 core done.

- Fixed timestep: `TARGET_FRAME_TIME = 1/60` (`Main.cs:2229`), accumulator clamped to
  `3.0×target`, **one update max per draw** — catch-up is discarded, not executed
  (`Main.cs:17005-17025`). `updateRate` counts updates/sec.
- Per-tick order (`DoUpdateInWorld_Inner` `Main.cs:17744`): particles → players →
  RevengeManager → `SpawnNPCs` → pressure plates → NPCs → gores → projectiles → items →
  dust → leashed → floating text → time (`Main.cs:18083`) → `WorldGen.UpdateWorld` +
  `UpdateInvasion` → chest frames → ambient wind (`Main.cs:17820`) → tile/wall renderers
  → `UpdateCameraPan` (`Main.cs:62664`) → `SceneState.Update` (`Main.cs:17825`).
- **RNG discipline:** every world stage runs under `Main.SwapRandom("<label>")`
  (`Main.cs:17747-17815`) — a *named RNG stream* per stage; `UnpausedUpdateSeed` per tick
  (`Main.cs:17246`); `TileFrameSeed` reseeded every 5 draws (`Main.cs:61727-61729`).
  Port implication: our single xorshift worldgen stream matches, but per-stage named
  streams (SpawnNPC, etc.) are a determinism-relevant design item when M4 spawns land.
- Draw order (`DoDraw` `Main.cs:61707`): sky capture → surface bg → stars/sun/moon →
  scene overlays → `DrawBG` skies → black fill → `Lighting.LightTiles` → back water →
  cave backgrounds → walls/NPCs/tiles sandwich (`Main.cs:63265`) → projectiles/players/
  items/rain/dust → front water + wires → ScreenDarkness/overlays → UI (`Main.cs:62484`).
- `dayRate` (time multiplier) can be 0 (FreezeTime) — all consumers divide by
  `(dayRate == 0) ? 1 : dayRate`. Weather ticks run **dayRate× per frame**
  (`Main.cs:17228`), so sundial fast-forward multiplies event chances 60×.
- `timeForVisualEffects` wraps at 216000 (`Main.cs:17218`); `GlobalTimeWrappedHourly` at
  3600 s (`Main.cs:16845`); `sunCircle += 0.01` wraps at 6.285 (`Main.cs:62082`).
- Camera: `screenPosition` from player bottom + `(0, gfxOffY−21)` + pan − half screen
  (`Main.cs:62590-62641`); zoom `GameZoomTarget` clamp 1–2 (`Main.cs:16801-16813`);
  `ForcedMinimumZoom = max(1, screenW/MaxWorldViewSize…)` (`Main.cs:61991`).
- **Hidden:** gfxQuality auto-scale caps `maxDustToDraw = 6000*(q*0.7+0.3)` min 1000 and
  `LegacyLighting.RenderPhases` 8→4 by quality (`Main.cs:17040-17131`); black-fade
  teleport when pan distance exceeds `BlackFadeDist` (`Main.cs:62472`).

**Tasks:** REFACTOR-env (env flags), and when M4 spawns land: add named RNG streams
(determinism proof required). Menu modes: `menuMode` values enumerated in `Main.cs:48201+`
(0 title, 888 fancy UI framework, 3000 credits, 272727 easter egg, ~50 others).

## 2. Tiles: framing, drawing, tile objects, paint

**C#** `WorldGen.cs` (TileFrame parts), `GameContent/Drawing/TileDrawing.cs` (9 656),
`ObjectData/TileObjectData.cs` (5 328), `TilePaintSystemV2.cs` · **Rust** `framing.rs`,
`tiledata/`, `render/tiles.rs`, `drawdata.rs` · 🟡.

- `TileFrame` (`WorldGen.cs:81328`): border guard ±5; inactive clears slope/paint;
  `tileFrameImportant` → per-type dispatch (`TileFrameImportant` `WorldGen.cs:84962`);
  falling tiles → `SpawnFallingBlockProjectile` (`WorldGen.cs:86039`); else
  `TileFrameCosmetic` (`WorldGen.cs:81390`, ported).
- FrameImportant dispatch includes: torches → `CheckTorch` (86095, ported), doors
  10/11 → `CheckDoorOpen/CheckDoorClosed` (86339/86432), chests → `CheckChest` (57923),
  chandeliers 34/454 → `CheckChand`, trees → `CheckTree*` (54402/54570), herbs →
  `CheckAlch` (45972), vines `CheckVines` (84858) — a vine dies unless the tile above is
  its family or base block (6 families listed), drip tiles die without solid above
  (85090), multi-directional 129/149/178 (60314), anchors via `CheckTileAnchors`
  (`WorldGen.cs:51808`, mode&1 columns / mode&2 rows, `AnchorValid` 51834).
- `TileObjectData` (`TileObjectData.cs`): Width/Height/Origin/CoordinateHeights/anchors
  (1174-1621), style wrap `StyleWrapLimit` (1009)/`StyleLineSkip` (1108)/`StyleMultiplier`
  (1141), alternates as cloned modules; `TileObject.Place` (`TileObject.cs:26-140`)
  computes style → pre-kills cut tiles → writes per-column frames → `FlattenAnchors`
  slope-flattening. **This is the M1.4-TO foundation** (doors, chests, furniture).
- Paint/coatings: `TilePaintSystemV2.cs` — done (TILE-DRAW-4): shader-side recolor, wall
  paint + coatings stored in `World.extras` on top of the E1 paint byte; echo/invisible
  coats remain.
- **Hidden:** `StopMergingByInvisibility` breaks merge chains across invisibility state
  (86023); slope masking kills specific neighbours (`WorldGen.cs:81390` chain); open-top
  tiles 477/492 block spawn 10% unless event (`NPC.cs:1022`); `SlowlyDiesInWater` tiles
  die at >32 liquid (`WorldGen.cs:72203`).

**Tasks:** M1.4-TO (umbrella) → M1.4-TO-1 doors, -2 chests, -3 furniture/Check3x2 family,
-4 herbs/alch/pots; TILE-DRAW-1/2/3/4/5 as before; TILE-DRAW-6 vines/moss/cactus arms.

## 3. Liquids & waterfalls

**C#** `Liquid.cs`, `LiquidBuffer.cs`, `GameContent/Liquid/*` · **Rust** `liquid.rs`,
`liquid_draw.rs`, `waterfall/`, `render/liquids.rs` · ✅ core (M1.5-LIQUID-2 done).

- Remaining: shimmer vertex distortion (`Main.DrawTileInWater` full effect),
  `HorizonClouds` fall tint, pumps (`Wiring.XferWater` `Wiring.cs:475`, in/out caps 20,
  type-locked, 255 max), actuator liquid control, buckets.
- **Hidden:** liquid updates every other tick (`Liquid.skipCount > 1`); `curMaxLiquid`
  scales with gfx quality (`Main.cs:17115`); waterfalls re-scan every 30 ticks (ported).

## 4. Lighting

**C#** `Graphics/Light/*`, `TileLightScanner.cs` (3 318) · **Rust** `lighting/`,
`tile_light/` · ✅ core (147-case scanner ported).

- Remaining: sun/moon/star light contribution, invisible blocks/walls (not stored in the
  packed tile), jellyfish-cage animation state, shader chromatic aberration (instances
  need a flag).
- **Hidden:** darkness debuff multiplies light ×0.95 / blackout ×0.85 in neg-light mode
  (`LegacyLighting.cs:944-958`); lightning flash relights tiles twice (`Main.cs:58761`);
  disco/shimmer dynamics and `mouseTextColor` flicker already ported (M1.5-LIGHT-1/2).

## 5. Sky, weather, ambient skies & screen filters

**C#** `Main.cs` weather/filters, `SceneState.cs`, `SceneMetrics.cs`, `Cloud.cs`,
`Star.cs`, `Rain.cs`, `GameContent/Skies/*` (15), `GameContent/Events/Sandstorm.cs`,
`GameContent/Ambience/*` · **Rust** `render/sky.rs`, `biome.rs`, `rain.rs`,
`world_sim/weather.rs` · 🟡.

- Time: `dayLength = 54000` ticks (15 min), `nightLength = 32400` (9 min)
  (`Main.cs:1173-1175`); `moonPhase` 0-7 advanced at every StartDay (`Main.cs:65398`);
  `Main.time = 0` at each dawn/dusk; sundial/moondial set `dayRate = 60`, cooldown 8 days,
  reset by blood moon/meteor nights (`Main.cs:6169-6203`, `65350`, `65422`).
- Rain: chance per day `rand.Next(86400/dayRate * 5.75) == 0` (×4.25 overcast, ×1
  ladybugRainBoost), needs a player with `statLifeMax >= 120` (`Main.cs:64898-64914`);
  `rainTime = rand(3600,86400)` + additive bonuses (`Main.cs:64659`); strength from
  cloudiness bands >150/>100/else clouds (`Main.cs:64728`); coin rain via luck
  `RollLuck(25)` (anniversary: 5) (`Main.cs:64666`); `IsRainingForever = rainTime >=
  5184000` (`Main.cs:3044`); `cloudAlpha` lerps at `0.0005*dayRate` per *draw*.
- Wind: `windCounter = rand(900,2701)`, extreme events `rand(10,31)`; drift steps
  `rand(±25|±50|±100|±200|±400|±850)*0.001`, clamp ±0.8, cap 0.35 without a 120-HP player
  (`Main.cs:58701-58902`); `windSpeedCurrent` approaches target ×(1+5/9·maxRaining) at
  `0.0003+|Δ|·0.0015`; direction reversal resisted 2/3 (`Main.cs:58902`); windy-day
  music window 10800<time<43200 (`Main.cs:13000`); storm music thresholds
  `_minWind 0.34 / _minRain 0.4` (`Main.cs:67225`).
- Thunder: `thunderDelay = rand(1,31)/rand(31,121)/rand(11,61)/1` variants; bolt offset
  `thunderDistance*15`; chance only when player above rock layer and `atmo == 1f`, never
  in remix (`Main.cs:58773-58820`); graveyard ≥0.9 → `rand(120)==0` (`Main.cs:58822`).
- Clouds: `rand(60)` drift; spawn `rand(1000) < 200*cloudAlpha` up to 100; rare cloud
  `rand(150)` (25 anniversary/DST) (`Cloud.cs:140`); overcast `cloudBGActive` ramps ±0.0005.
- Sandstorm: needs `|windSpeedCurrent| ≥ 0.6`; chance `rand(21600)` per dayRate tick
  (×3 pre-HM, ×2 HM); duration `rand(28800,86401)`; `Severity` moves 0.003/tick toward
  `IntendedSeverity`; dust needs ≥100 desert sand tiles (`Sandstorm.cs:22-147`).
- Blizzard: `snowing()` needs SnowTileCount>0 && surface atmospherics; intensity
  `pow(count/max, 4−3·cloudAlpha)`; Deerclops near screen ÷20 (`Main.cs:13032-13051`).
- Screen filters (`SceneState.ApplyVisuals` `SceneState.cs:64-253`): graveyard,
  Noir, water distortion (intensity from wind+rain+depth, clamp 2.5), sandstorm overlay
  (opacity `min(1, Severity*1.5)`), heat distortion (underworld within 320 tiles, desert
  surface, sunScorch), blizzard, monolith filters (Stardust/Nebula/Vortex/Solar/MoonLord),
  blood moon, aurora (`ZoneSnow && night && !storm`), sepia/noir/CRT, party overlay.
  Sky entities registered in `ScreenEffectInitializer.cs:56-74` (18 skies).
- Ambient creature skies (`AmbientSky.cs`): 20 entity classes (BirdsPack 432, Seagulls
  475, Gastropod 586, SlimeBalloon 691, HellBats 788, Bats 898, Wyvern 1003, BoneSerpent
  1054, Airship 1058, AirBalloon 1101, Crimera 1153, EoW 1177, EoC eyeball 1201, Meteor
  1291, Butterflies 179, LostKite 223, Pegasus 281, Vulture 329, PixiePosse 378).
- Ambience emitters: attempt `rand(600,7200)` ticks; birds/vulture/seagulls/pegasus
  sunny-day, wyvern sunny+HM, airships windy, eyeball night, hellbats always; biome gates
  per player (`AmbienceServer.cs:57-96`); waterfall/lava loops every 15 ticks
  (`Main.cs:13466`).
- **Hidden:** stars `rand(200,400)` per night, starfallBoost 1/10 nights ×(3-5) else 1/3
  ×(1-1.5) (`Star.cs:41-95`); Lantern Night force-stops rain and overcast
  (`Main.cs:64827-64856`); shimmer: stars only, `rand(90)==0` drops a star item
  (`Main.cs:17269`); meteor visible 15000-16200, lands `time > 16200` (`Main.cs:65466`).

**Tasks:** M1.4-BG-18-screen-filters (filters/monoliths), M1.4-BG-19-ambient-skies
(creature skies), M1.4-BG-20-ambience-emitters, M1.4-BG-21-sundial (sundial/moondial
fast-forward), M1.4-WEATHER-1-wind-gusts, M1.4-WEATHER-2-sandstorm.

## 6. World generation

**C#** `WorldGen.cs` gen half + `GameContent/Generation/*` + `GameContent/Biomes/*` +
`WorldBuilding/*` · **Rust** `worldgen/` · 🟡 compact model only.

- Pass pipeline is **data-driven** (`GenPassNameID` + embedded
  `WorldBuilding.Configuration.json`, `WorldGen.cs:11078`) — `AddPasses`
  (`WorldGen.cs:11502`) registers ~92 passes in order; `DisablePassesForSpecialSeeds`
  (22493) only handles dualDungeons; other seeds gate inside pass bodies.
  Key order: TerrainPass → Tunnels/MountainCaves/Caves → Ice/Grass/Jungle/Desert/
  Mushroom/Marble/Granite/FloatingIslands → Silt/OresAndShinies/Webs/Underworld →
  CorruptionAndCrimson → Lakes → Dungeon → Gems → Shimmer → Pyramids → LivingTrees →
  Altars → Temple → Beehives → Shrines → SettleLiquids → SmoothWorld → Waterfalls →
  LifeCrystals → Statues → UndergroundHouses/Chests → SpiderCaves/GemCaves/Moss →
  QuickCleanup → Pots/Traps/Piles → SpawnPoint → Trees/Herbs/Plants/Vines → MicroBiomes →
  CactusPalmCoral → TileCleanup → LihzahrdAltar → FinalCleanup (full ordered list with
  anchors at `WorldGen.cs:11502-22141`).
- Terrain: per-column feature enum {Plateau, Hill, Dale, Mountain, Valley}, runs
  `Next(5,40)` cols (`TerrainPass.cs:86`); mountains suppressed in mid-band
  X∈(0.45,0.55), forced Plateau X∈(0.48,0.52) (`TerrainPass.cs:98-101`); surface clamp
  `[maxTilesY*0.17, *0.23]` (`TerrainPass.cs:114`); `worldSurface = maxSurfaceY+25`
  (`TerrainPass.cs:203`); `rockLayer` snapped to a multiple of 6 (`TerrainPass.cs:208`);
  `waterLine = (rockLayer+maxTilesY)/2 + Next(-100,20)`, `lavaLine = waterLine +
  Next(50,80)` (`TerrainPass.cs:211-212`).
- Layout (Reset `WorldGen.cs:11340-11460`): dungeon side coin-flip; jungle at 15-30 %
  from center-opposite side (anniversary 25-35 %); snow band center constrained to
  (0.25-0.4 | 0.6-0.75) of map width, half-width `Next(50,90)+2·Next(20,40)·scale`;
  `beachBordersWidth = 275`; desert placed opposite the dungeon.
- Ores (`OresAndShinies` 13182): all via `TileRunner` random-walk blobs; densities
  per tile: copper 6e-5/8e-5/2e-4 by band, iron 3e-5/8e-5/2e-4, silver 2.6e-5+1.5e-5,
  gold 1.2e-5×2; alt tiers copper↔166, iron↔167, silver↔168, gold↔169 coin-flip
  (`WorldGen.cs:11206-11235`).
- Evil: count `maxTilesX*0.00045` (remix ×2); chasm `ChasmRunner` steps
  `Next(150)+150` (`WorldGen.cs:14066`); orbs `AddShadowOrb` (32095); beach avoidance
  `evilBiomeBeachAvoidance = beachSandRandomCenter+60`.
- Floating islands: count `maxTilesX*0.0008`; skyLakes 1 (+1 >6000, +1 >8000)
  (`WorldGen.cs:11171-11177`); Y band `Next(90, worldSurface/2+100−100)`.
- Shimmer/Aether: one per world, opposite the dungeon in the outer 11 % band, Y
  `(worldSurface+rockLayer)/2+50 … min(maxTilesY−460)`; 200×200 protected rect
  (`WorldGen.cs:15195`).
- Hardmode trigger `initializeHardMode` (31570): two `GERunner` hallow strips at
  `Next(300,400)*0.001` and mirrored, `strength 3, steps 5.0`; evil house walls 25×scale
  flood-fills; **`SmashAltar`** (48927) cycle `%3`: Cobalt(107)/Palladium(221) →
  Mythril(108)/Orichalcum(222) → Adamantite(111)/Titanium(223), blob count
  `(maxTilesX/4200*310 − 85*(count%3)) * 0.85 / (count/3+1)`, depth bands per tier
  (49044-49066). `Main.rand` is *re-seeded from `DateTime.Now`* in `initializeHardMode`
  (31572) — hardmode is **not** seed-deterministic in vanilla; document if we deviate.
- World metadata (`.wld` header, `IO/WorldFile.cs:1223-1459`): seed text, bounds,
  gameMode, all seed flags, moonType/background styles, spawn, worldSurface/rockLayer,
  time/bloodmoon/eclipse, dungeonX/Y, crimson, 24 downed flags, saved NPCs, invasion
  state, slimeRain, rain state, SavedOreTiers, angler quests, party, cultistDelay.
- **Hidden:** drunk world flips dungeon side *twice* in Reset (11368, 11390);
  `crimStone/ebonStone` (25/203) and walls 3/83 **swap** when drunk+FTW
  (`flipInfections` 11238-46); `Main.tileSolid[659] = false` during gen (11478);
  `slimeRainTime` initialised negative `-Next(86400*2, 86400*3)` (11264); hell chest
  pool `{274,220,112,218,3019}` (remix 112→683) (11272-11283); genRand IS Main.rand
  (`WorldGen.cs:4445`); config-driven counts live in the embedded Configuration.json
  (not in the decompile — mark when porting).

**Tasks:** M5-GEN-1 pass-pipeline scaffold (config-driven passes + GenVars, keep
`deterministic_for_seed` FNV proof), M5-BIOME (evil/jungle), M5-BIOME-2
mushroom/marble/granite, M5-OCEAN, M5-DESERT-structures, M5-DUNGEON, M5-TEMPLE,
M5-LIVING-trees, M5-STRUCT-1 shrines/cabins/pots/statues/traps, M5-STRUCT-2
beehives/honey, M5-STRUCT-3 pyramids/oasis, M5-STRUCT-4 floating islands, M5-STRUCT-5
spider/gem/moss caves, M5-ORES-1 TileRunner fidelity + alt tiers, M5-HARDFORCE
(initializeHardMode + altar ores), M5-SPREAD-1 grass/vine/moss growth ticks, M5-SPREAD-2
evil spread & chlorophyte defense, M5-CHEST-gen, M5-SEED-1 secret seed framework
(drunk/FTW/remix/anniversary/DST), M5-SEED-2 1.4.5 micro-seeds (38 modular seeds,
`WorldGen.cs:3022-3055`).

## 7. Player movement & physics

**C#** `Player.cs` (movement/collision), `Collision.cs` (4 598) · **Rust** `player.rs`,
`collision.rs`, `world_sim/player.rs` · 🟡 core done.

Ported: gravity 0.4, run 3.0/acc 0.08/slowdown 0.2, jump 15/5.01, hold-jump, 1-tile step,
axis AABB, water buoyancy, breath/drowning (2 HP per 7 frames, +3 breath/frame),
fall damage, lava flat 80 + OnFire 420, respawn, camera.

Remaining (audit-verified):
- Static constants (`Player.cs:57117-57121`): `originalRunSpeed = 3`, `jumpHeight = 15`,
  `jumpSpeed = 5.01`, `maxFallSpeed = 10`, `respawnTimerMax = 3600`; bare-walk accel is
  `runAcceleration*0.2` (`Player.cs:19963-19999`).
- Boost items: `accRunSpeed = 6` (Amphibian/Frog/Sand 3990-3996/4055) and `6.75`
  (Lightning 898, Frostspark 1862, Terraspark 5000 with waterWalk/fireWalk/lavaMax+420)
  (`Player.cs:14400-15399`); ice `slippy`: accel ×0.7, slowdown ×0.1; with iceSkate ×3.5
  and maxSpeed ×1.25 (`Player.cs:26533-26554`); desertBoots ×1.75 on sand; wet velocity
  multiplier **post-collision**: water 0.5, lavaWet 0.5, honey 0.25, shimmer 0.375
  (`Player.cs:28180-28192`) — not drag.
- Extra jumps (`JumpMovement` 20531): Sandstorm ×3, Blizzard ×1.5 (accel ×3), Fart ×2
  (accel ×3/speed ×1.75), Sail ×1.25, Unicorn/WoF-goat/Santank ×2, Basilisk/Cloud ×0.75;
  refreshed by `RefreshDoubleJumps` (22826); each sets accel ×3 while active (26566-26605).
- Jump mods (`UpdateJumpHeight` 19587): jumpBoost floor 20/6.51, frogLeg +2.4/+15 fall,
  moonLordLegs +1.8/+10/+1, wereWolf +2/+0.2, stool +5, sticky /10, dazed /5.
- Wings (`WingMovement` 22022, stats `WingStatsInitializer.cs:28-61`): flyTime tiers
  25-180; cap is a multiple of `jumpSpeed` (so jump items scale wing speed); hover wings
  4/44/45 thrust 0.2-0.4 toward `-jumpSpeed*3`; rocket boots `rocketTimeMax = 7`, only
  while falling (`canRocket` `Player.cs:26801`), +wings merge `wingTime += rocketTime*6`.
- Fall damage (`Player.cs:25296-25355`): threshold `25 + extraFall`; dmg
  `(tiles − threshold) * 10` ×mount multiplier; stoned `(tiles−2)*20`; resets from
  jump/rocket/wet/slowFall; cloud/hell-brick landing zeroes it; bunny mount nullifies.
- Dash (`DashMovement` 21034): Tabi/MNG dashType 1 `16.9f`, EoC shield 2 `14.5f`
  (`eocDash = 15`, 30×melee, cooldown 30 vs 20), Solar 3 `21.9f` (150×melee), Chillet
  mounts 6 `16f`; double-tap window `dashTime = ±15`; decay 0.992 then ×0.96.
- Grapple (player side, `GrappleMovement` 22414): velocity = avg hook anchor − center,
  cap 11 (overrides 12-16 per hook); jump clears hooks (`Player.cs:22686-22712`);
  fall damage disabled while grappled (22440).
- Minecart: `SetAsMinecart` runSpeed 13, acc 0.04 (`Mount.cs:2747`); track boost ±4;
  off-track friction 0.2 + dismount; wheel sparks at 33/66 % thresholds.
- Webbed (buff 149) zeroes gravity/jump/dash and deletes hooks (`Player.cs:11861-11879`).
- Shimmer: buff 353 freezes, `timeShimmering` clamp 0-7200, exit after 1200 ticks of
  input, `AllowShimmerDodge` short-circuits `Hurt` (`Player.cs:38199`).
- **Hidden:** `jumpHeight/jumpSpeed` are **static shared** fields recomputed per-player
  per-frame — treat as per-player derived; wet dive = full fall-damage reset (25371);
  lava damage has its own `ImmunityCooldownID.Lava = 4` counter, flat (not difficulty
  scaled); drowning is exactly 2 HP / 7 frames (~17 DPS); OnFire regen penalty −8
  (−100 remix); honey cancels −4 regen and adds +2.

**Tasks:** M2-PHYS-1 movement constants & boots, M2-PHYS-2 fall damage & extraFall,
M2-PHYS-3 fluid movement & breath fidelity, M2-PHYS-4 dash system, M2-PHYS-5 wings &
rocket boots, M2-PHYS-6 grapple movement, M2-PHYS-7 minecart riding, M2-PHYS-8 extra
jumps.

## 8. Player status: buffs, debuffs, regen, luck

**C#** `Player.cs` (buffs/regen), `BuffID.cs`, `GameContent/Luck.cs`,
`GameContent/BannerSystem.cs`, `SceneMetrics.cs` · **Rust** ⬜ (none yet) · ⬜.

- Buff slots: `maxBuffs = 44` (`Player.cs:57106`); `buffTime--` except
  `TimeLeftDoesNotDecrease` set {28,334,29,159,150,93,348,366} (`BuffID.cs:33`); debuffs
  displace positive buffs when full, full-debuff bar rejects new debuffs
  (`AddBuff_ActuallyTryToAddTheBuff` `Player.cs:5325-5359`); expert extends flagged
  debuffs ×2 (master ×2.5) (`Player.cs:5434`); Warmth cuts Chilled /4 and Frozen /2.
- Regen engine (`UpdateLifeRegen` `Player.cs:18942`): accumulator `lifeRegenCount`, +1 HP
  per 120; ramp tiers at 300/600/900/1200/1500/1800/2400/3000 → +1 each; moving ×0.5,
  still ×1.25, sitting ×1.3; campfire ×1.1; **expert without food ÷2** (×0.75 shinyStone);
  final `lifeRegen += round(ramp * (statLifeMax2/400*0.85 + 0.15))` (`Player.cs:19262`).
- Mana (`Player.cs:19398-19560`): delay `((1−mana/max)*240+45)*0.7`; regen
  `maxMana/3 + bonus`, doubled if still/grappling/buff; accumulator 120 → +1.
- DoT values (per tick, from regen): poison −4, venom −30, OnFire −8, cursed inferno
  −24, frostburn −16, hellfire −60, suffocation −40, electrified −8 (−40 moving);
  bleeding blocks natural regen; drain caps 4 HP per 120-chunk; starving hits
  `maxLife/50` (×2 desert/snow).
- Environmental auras (`SceneMetrics` scans a **169×124-tile** box
  `SceneMetrics.cs:913-920`): campfire +1 regen & ×1.1 ramp; heart lantern +2; star in
  bottle `manaRegenBonus += 10`; sunflower moveSpeed +0.1 then ×1.1 (cancelled in
  graveyard); catbast def+5; water/peace/shadow candles (spawn side, §15).
- Luck (`RecalculateLuck` `Player.cs:29137-29164`): `luck = ladybug*0.2 + torchLuck*0.2 +
  luckPotion*0.1 + kite/3*0.1 + 0.03·galaxyPearl + 0.3·lanternNight + 0.2·gnome − 0.25·
  stinky + equipment + coinLuck − 0.25·brokenMirror`. Torch luck per-biome table
  (`Player.cs:17921-18035`, 13 torch types, plain torch bad in snow/desert/evil/hallow);
  ladybug good 43200 / bad −10800 ticks (bad weighs 4×), radius 800/400 px weighted
  `NPC.cs:81570-81647`; coin luck cap 1e6, tiers >249000→0.2 … >0.249→0.05
  (`Player.cs:18115-18153`), decay ×0.9999^dayRate. **Never clamped** — the −0.7/1.0
  caps are UI-only (`Player.cs:4112`).
- `RollLuck` (`Luck.cs:5-16`): positive luck and `rand.NextFloat() < luck` →
  `rand.Next(rand.Next(range/2, range))`; negative mirrors to *worse*; consumers:
  drop rules, chest loot, money rolls (keep better/worse of two), fishing power ×(0.9-1.2
  to 1.1+0.3·rand), `Main.DamageVar` (contact damage uses **−luck**).
- Banner system (`BannerSystem.cs` + `ItemID.cs:11-46`): `KillsToBanner` default 50
  (`ItemID.cs:1303`), overrides 10-1000; banner **tile 91** in the SceneMetrics scan →
  `NPCBannerBuff`; strength 1: DamageDealt ×1.5 (expert 2.0), DamageReceived ×0.75
  (expert 0.5).
- WellFed tiers 26/206/207: def +2/+3/+4, all-class dmg +5/7.5/10 %, crit +2/3/4,
  moveSpeed +0.2/0.3/0.4; hungry 333 / starving 334 are the mirrors; all IsFedState
  buffs are deleted before adding a new food buff (`Player.cs:5277`).
- **Hidden:** `WellFedHelper.cs` is **dead code** in the decompile (unreferenced — do not
  port it as live logic); luck potions tier by *remaining time* (>36000 → 3); coin luck
  has a duplicated unreachable branch (`>24900 → 0.175` twice, `Player.cs:18125`); buff
  383 cap 43200 = exactly one day; mana sickness −25 % magic at 300 ticks fading.

**Tasks:** M2-BUFF-1 buff slot engine, M2-BUFF-2 regen engines (life/mana), M2-BUFF-3
DoT debuffs, M2-BUFF-4 stat/movement debuffs, M2-BUFF-5 environmental auras (SceneMetrics
scan), M2-BUFF-6 luck system, M2-BUFF-7 food/WellFed, M2-BUFF-8 banners.

## 9. Combat math & death consequences

**C#** `Main.cs` damage helpers, `Player.cs` Hurt, `NPC.cs` StrikeNPC · **Rust** ⬜ · ⬜.

- Variance `Main.DamageVar(dmg, luck)` (`Main.cs:66118`): ±15 %, positive luck rerolls
  keep higher, negative keep lower; `Math.Round` (banker's in C# — port carefully).
- Player→NPC: class multiplier (`GetWeaponDamageMultiplier` `Player.cs:53386`) → crit
  roll `rand.Next(1,101) <= crit` (base crit **4** for all classes, `Player.cs:2430`) →
  banner × → `DamageVar` → **armor penetration added after DamageVar** (`Player.cs:44400`):
  `checkArmorPenetration` = `min(armorPen, defense)/2` subtracted (`NPC.cs:81147`; ichor
  +15, brokenArmor +20, betsyCurse +40).
- `CalculateDamageNPCsTake = Damage − defense*0.5` min 1 (`Main.cs:66147`); crit ×2
  applied inside `StrikeNPC_Inner` (`NPC.cs:81226`).
- NPC→Player (`Player.cs:31353-31400`): `DamageVar(npc.damage·mult, −luck)` → banner
  DamageReceived × → thorns/parry → `Hurt` (`Player.cs:38197`): dodge rolls (sash 1/10,
  black belt 1/10, brain of confusion 1/6), defense per difficulty (`Main.cs:66167`:
  Classic `−def*0.5`, Expert `×0.75` **then** `−def*0.5`? — see source, Master `−def`
  full, min 1; PVP ×0.5), endurance `(1−endurance)`, Solar shell ×0.8, beetle orbs
  ×(1−0.15·orbs), Paladin shield 25 % share within 800 f.
- Iframes: pvp 8; else `longInvince ? 80 : 40` (damage≠1) or `?40:20` for exactly-1
  damage (`Player.cs:38367-38370`); cooldown counters General/TileContact/BossNoCheese/
  Lava/PaladinsShield (`ImmunityCooldownID.cs:76-83`).
- Knockback: NPC `vel += kb·knockBackResist` with soft caps 8→10→… (`NPC.cs:81388`);
  Master knockback ×0.8; player fixed `4.5·dir, −3.5` unless noKnockback/cart.
- Death: softcore/journey drop **only gems** (1522-1527, 3643; `Player.cs:38863`);
  mediumcore drops inventory except 3506/3507/3509; coins drop `stack/2` (expert /4,
  master none, PVP none) (`DropCoins` `Player.cs:54261`); respawn base 600, +600 boss
  near (4000 px), ×1.5 expert, cap 3600 (`GetRespawnTime` `Player.cs:39060`), skip after
  90 s dead with no danger (`AllowsRespawnTimerSkip` 38955); tombstone RNG: >100k coins →
  `rand(5)+527`, else 1/6 real grave (`Player.cs:39091-39115`).
- Coin-loss revenge (`CoinLossRevengeSystem.cs`): NPCs absorb dropped coins into
  `extraValue` (`WorldItem.cs:1189`), cached markers expire by coin tier 3600-432000
  ticks, respawn with `life = lifeMax × max(0.5, hp%)`, killing re-drops the money.
- **Hidden:** `Hurt` computes defense from **pre-crit** damage — crit ×2 never feeds the
  defense formula in Hurt (callers bake it); contact damage passes −luck (bad luck = more
  damage taken); expert defense formula reads `Damage − defense*0.5` after a ×0.75
  multiplier — verify order when porting; death drops gems even in softcore.

**Tasks:** M4-DAMAGE (damage pipeline), M2-COMBAT-1 iframes & knockback, M2-COMBAT-2
crit/damage classes, M2-DEATH-1 death drops & tombstones, M2-DEATH-2 respawn rules,
M4-REVENGE-1 coin-loss revenge.

## 10. Items & use logic

**C#** `Item.cs` (49 981), use sections of `Player.cs` · **Rust** `item.rs`, `registry.rs`,
`inv.rs` · 🟡 (timings + hotbar done — M2-ANIM-item-timings).

- Use styles (`ItemUseStyleID.cs:5-37`): 0 None, 1 Swing, 2 EatFood, 3 Thrust, 4 HoldUp,
  5 Shoot, 6 DrinkLong, 7 DrinkOld, 8 GolfPlay, 9 DrinkLiquid, 10 HiddenAnimation,
  11 MowTheLawn, 12 Guitar, 13 Rapier, 14 RaiseLamp, 15 HoldOrb, 16 PlaySound; body-frame
  dispatch per style (`Player.cs:36312-36430`; style 5 frames from
  `itemRotation·dir` thresholds −0.75/+0.6).
- Timing: `useAnimation` drives the visual, `useTime` gates the effect (`itemTime`),
  `reuseDelay` post-lockout, `autoReuse`; style-5 autofire requires
  `useTime <= useAnimation && reuseDelay == 0` and then adds **+1 frame**
  (`Player.cs:4466-4469`) — off-by-one trap.
- Channel: `StartChanneling` (52958), cleared on release; held-shoot gate
  `itemAnimation > 0 && ItemTimeIsZero` (43556); per-item shot suppression ×3/×4/×5
  (43502-43511).
- Mana (`ItemCheck_ActuallyPayMana` 52638): free-use exceptions (spaceGun items
  127/4347/4348/514; item 3269), doubled cost 3852 alt-fire, Mana Flower −8 %.
- Ammo (`PickAmmo` `Player.cs:53458-53760`): ammo slots = inventory 54-57; ammo adds
  its damage×multiplier + kb + speed; archery: speed <20 → ×1.2 capped 20; not-consume
  rolls per item (1/3, 1/2, 1/5…); ammo cycling cooldown `useAnimation + 120`.
- SetDefaults (`Item.cs:48663`): split into SetDefaults1..5 by type ranges; `ItemVariants`
  (`GameContent/Items/ItemVariants.cs:9-60`) pick per-seed variants; food via
  `SetFoodDefaults`.
- **Hidden:** wet player cannot fire torch-type projectiles (52110); paint brushes need
  paint in inventory; single-instance projectiles block re-shoot; swing rate display
  `60/(useAnimation+reuseDelay+(!autoReuse))` (19490).

**Tasks:** M3-ITEM-1 SetDefaults registry scale-up (+ItemVariants), M3-ITEM-2 full use
styles, M3-ITEM-3 ammo system, M3-ITEM-4 mana & mana sickness, M3-ITEM-5 channel/reuseDelay,
M3-TOOL (tiers/durability), M2-ANIM-item-timings ✅.

## 11. Crafting, prefixes, dyes, reforge

**C#** `Recipe.cs` (16 784), `GameContent/Prefixes/*`, `GameContent/Dyes/*` ·
**Rust** `ui/crafting.rs` (chrome), `registry.rs`, `craft.rs` · ✅ subset-first done 2026-11-03: 7-recipe table + can_craft/try_craft/adjacent_tiles + panel interaction (contexts **41/42/43**, `ItemSlot.cs:126-130` — the catalog's 4/33 was stale); full SetupRecipes generator is a follow-up.

- Recipe structure (`Recipe.cs:58-112`): 15-ingredient cap, `requiredTile`, groups
  (`RecipeGroup`, fake IDs ≥ `FakeItemIdOffset`), flags needHoney/Water/Lava/
  TorchGodsFavor/alchemy/needSnowBiome/needGraveyardBiome/needMechdusa/notDecraftable;
  environment check `PlayerMeetsEnvironmentConditions` (322-357).
- Alchemy: per-ingredient `rand.Next(3)==0` not-consumed, only with alchemyTable
  (`Recipe.cs:215-227`).
- Availability scan (`UpdateRecipeList` 231-264): counts from inventory (58 cap) + open
  container + bank4 (void bag) + **nearby chests within 600 f** if
  `CraftFromNearbyChests`; pending craft requests subtract.
- Reforge (`Main.cs:41586-41911`): cost `value·stack ×0.8(discount) ×PriceAdjustment /3`
  min 1; `Prefix(-2)` re-rolls; "best" = rolled prefix value == `BestPrefixValue()` of the
  legal set; top-tier feedback `reforgeCooldown = 60`.
- Prefixes (`Item.Prefix` `Item.cs:504-610`): natural spawn 1/4 **no prefix** first;
  `ReducedNaturalChance` set → 1/3 none; category lists in `PrefixLegacy.cs:24-90`
  (Swords 40, GunsBows 35, Magic 36, Summons 22, Accessories 19…); stats: damage/anim
  rounded after multiply, kb/speed/scale float; rarity shift by value multiplier ≥1.2 →
  +2 … ≤0.8 → −2 (clamp [-1,11]); price ×value²; accessory prefixes grant flat Player
  stats on equip (`GrantPrefixBenefits` `Player.cs:13953-14044`: def +1..4, mana +20,
  crit +2/4, dmg +1-4 %, speed +1-4 %).
- Dyes: draw-time `ArmorShaderData` passes (`Graphics/Shaders/ArmorShaderData.cs`), 5
  special shader classes in `GameContent/Dyes/`; `Player.dye[10]` mirrors slots.
- **Hidden:** invalid prefix rolls loop silently until valid; reforge rejects prefix 0;
  quest rarity −11 is preserved by the clamp; decraftItemId = lowest-value group member
  (`RecipeGroup.cs:53-56`).

**Tasks:** ✅ M3-CRAFT-recipes (subset-first, see above; contexts 41/42/43 — the old 4/33 numbering was stale), M3-CRAFT-2 recipe groups &
environment flags, M3-CRAFT-3 alchemy table, M3-PREFIX (roll engine + tables), M3-REFORGE-1
goblin reforge UI + cost, M3-DYES-1 dye slot plumbing (shader-pass architecture).

## 12. Equipment & armor sets

**C#** `Player.cs` equip sections, `DataStructures/ArmorSetBonuses.cs` · **Rust** ⬜ · ⬜.

- Slots: `armor[20]`, `dye[10]`, `inventory[59]` (50-57 = 4 coins + 4 ammo) — functional
  accessories are slots 3-9 (5 base + Demon Heart + master slot) gated by
  `IsItemSlotUnlockedAndUsable` (`Player.cs:12947`); benefits only when
  `!expertOnly || expertMode`.
- Set bonuses: `UpdateArmorSets` → `ArmorSetBonuses.GetCompleteSet(QueryContext)?.Effect`
  (`ArmorSetBonuses.cs:8`, `ArmorSetBonus.cs:19-45`); always-on partial effects for
  Vortex/Chlorophyte/Stardust/Solar/Beetle (16315-16441).
- **Hidden:** void bag (4131) in inventory enables bank4 info-accessories
  (`ItemID.Sets.WorksInVoidBag`); armor benefits scan uses `GetEffectiveArmor(k)`.

**Tasks:** M3-EQUIP (umbrella; also unlocks M2-DRAW-1 armor/accessories), M3-EQUIP-2 set
bonus engine, M3-EQUIP-3 accessory effect flags, M3-EQUIP-4 dye slots.

## 13. Storage & containers

**C#** `Chest.cs` (3 682), `GameContent/QuickStacking.cs`, `EmergencyStacking.cs`,
`Tile_Entities/*` (item side) · **Rust** ⬜ · ⬜.

- Banks: `bank(-2 piggy) / bank2(-3 safe) / bank3(-4 defender) / bank4(-5 void)`
  (`Player.cs:1580-1586`); `Chest.DefaultMaxItems = 40`; world chest = `chest >= 0`;
  `IsLockedOrInUse` (standing inside = in use).
- Pickup magnet: base range 42 px, speeds 0.45→4 (`Player.cs:57121`); coin 350 / mana
  300; direct-overlap pickup blocked while a use animation runs (34919); grab delay per
  item (`WorldItem.cs:201`).
- Quick stack: destinations = `NearbyChests` within **600 f**; matching by item then
  `ItemSorting` layer category (`QuickStacking.cs:34-40`); locked/in-use skipped.
- Emergency stacking: priority groups (FallenStars, coins, Equipment, RareCurrency,
  Default), 160 px stagger (`EmergencyStacking.cs:66`).
- Trash: single slot; auto-clears certain IDs on unload; research clones both the
  dropped item AND trash content before delete (`ItemSlot.cs:936-945`).
- **Hidden:** selling sets `stackSplit = 9999` (no repeat); void-bag pickups pull at 12 f
  within 5 ticks (`Player.cs:34947`).

**Tasks:** M3-CHEST-UI (world chest UI), M3-STORE-1 banks (piggy/safe/defender/void),
M3-STORE-2 quick stack/loot-all/restock, M3-STORE-3 trash & favorites.

## 14. NPC AI families

**C#** `NPC.cs` (95 306, `AI()` chain at 19819-41128) · **Rust** `npc.rs`,
`world_sim/npcs.rs` · 🟡 3 kinds.

- Top families (aiStyle → IDs): 1 slime hop (28+, `AI_001_Slimes` 60341) · 3 fighter
  walker (~157, `AI_003_Fighters` 55856) · 6 worm (47, `AI_006_Worms` 50963) · 7 town
  (67, `AI_007_TownEntities` 52965) · 2 floating eye · 5 flyer/charge · 8 caster ·
  10 cursed skull · 11 Skeletron head · 13 plant trap · 14 bat · 16 swimmer · 18
  jellyfish · 22 generic hover · 24 bird walk · 26 charger · 39 tortoise · 40 spider
  wall-crawl · 44 special flyer · 107 improved walkers (~20) · boss styles 4, 11, 61-68,
  86-89, 95-99, 109-127.
- Worm AI (`AI_006_Worms` 50963): head spawns chain, each `ai[1] = previous`, shared
  `ai[3]`/`realLife` = head; segment counts: Wyvern 14, CultistDragon 30, TombCrawler
  rand(6,10), DuneSplicer rand(12,21), BoneSerpent rand(14,23) (+3 FTW, +4 remix), EoW
  65/70 expert, Destroyer 80/100 FTW, Brain creepers 20/40 FTW; health sharing via
  realLife (EoW is the exception — per-segment); orphans kill themselves.
- **Hidden:** spawn RNG is the named `Main.SwapRandom("SpawnNPC")` stream
  (`Main.cs:17886`); `noSpawnCycle` latch suppresses the next spawn tick after
  catch/release/revenge (`NPC.cs:80171`).

**Tasks:** M4-NPC-1 (umbrella; per-family branches `M4-NPC-1-fighters`, `-slimes`,
`-worms`, `-flyers`, `-casters`, `-swimmers`, `-bats`, `-event`, …), M4-NPC-2 worm/segment
framework.

## 15. NPC spawning & town NPCs

**C#** `NPC.Spawner` (`NPC.cs:143`), `Main.UpdateTime_SpawnTownNPCs` (`Main.cs:65542`),
`GameContent/Personalities/*`, `ShopHelper.cs`, `Chest.SetupShop` · **Rust** `spawner.rs`
· 🟡 minimal.

- Tick: one candidate player per tick; `defaultSpawnRate = 600`, `defaultMaxSpawns = 5`,
  `activeTime = 750`, `maxNPCs = 200`, active range `2.1×screen`, safe range `0.52×screen/16`
  (`NPC.cs:95181-95188`, 935-936); 50-tile search; `CheckNotSpawningOnScreen`.
- Spawn-rate multiplier chain (`GetSpawnRate` `NPC.cs:473-810`, **sequential**): HM ×0.9,
  underworld cap ×2, deep ×0.4/×1.9, night ×0.6/×1.3, blood ×0.3/×1.8, moons ×0.2/×2,
  eclipse ×0.2/×1.9, dungeon ×0.3/×1.8, sandstorm ×0.4/×1.5, underground desert ×0.2/×3,
  jungle by town count ×0.4-0.85/×1.5-1.2, evil ×0.65/×1.3, meteor ×0.4/×1.1, temple
  ×0.8/×1.2, current-NPC ramp ×0.6-0.9, invis ×1.2/×0.8, calm ×1.65/×0.6, sunflower
  ×1.2/×0.8, angler armor ×1.3/×0.7, water candle ×0.75/×1.5 (+extra ×0.5 above 35 %
  depth), peace candle ×1.3/×0.7; floors rate ≥10 %, cap ≤3×; invasion override
  `maxSpawns = 5(2+0.3·players)`, `spawnRate = 20`; dungeon pre-Skeletron rate = 10.
- Town NPCs: housing flood fill (`WorldGen.MoveTownNPC` 4675 → `StartRoomCheck` 6041),
  too small <60 tiles, max 750, span <100 (`WorldGen.cs:6100-6158`); furniture via
  `TileID.Sets.RoomNeeds.CountsAs*` sets (`WorldGen.cs:5679`); move-in evaluation every
  7200/rate ticks (`Main.cs:65542`) with per-NPC conditions (Merchant ≥50 silver, Nurse
  lifeMax/20 > 5, Clothier post-Skeletron, Princess when all others present…).
- Happiness (`ShopHelper.cs`): like 0.94 / dislike 1.06 / love 0.88 / hate 1.12, clamp
  **0.75-1.5** then round to 2 decimals; homeless / >120 tiles from home / dangerous
  biome → ×1000 sentinel; crowding >3 within 25 tiles ×1.05 each; ≤2 in house & <4
  village ×0.95; Princess needs ≥2 nearby; biome preferences per NPC
  (`PersonalityDatabasePopulator.cs:13`).
- Shops: `Chest.SetupShop(shopIndex)` rebuilds on every open (`Main.cs:40459`); stock
  conditions (Pirate moon-phase items `Chest.cs:3452`, graveyard/beach, holidays);
  buy = value ×0.8 ×PriceAdjustment; sell = buy/5; sellback memory
  (`ItemShopSellbackHelper.cs`); Skeleton Merchant price curve by moonPhase 1.1-1.4.
- Pylons (sale): ≥2 housed NPCs within 100 tiles of home (`Chest.cs:3466-3495`); biome →
  pylon type table.
- **Hidden:** shadow candle zone forces `noWorms=false, townNPCs=0` (`NPC.cs:417`);
  AFK 300 ticks sets `noGroundWorms`; zombie variants: 7 styles, torchZombie 1/12
  (×2-6 by player count/HP), maggotZombie 1/20 (`NPC.cs:5583-5605`); gold critters via
  `RollLuck(goldCritterChance)`; gnome chance 10 base; happiness ×1000 is a sentinel, not
  a price; town NPCs' `npcSlots` suppress spawns.

**Tasks:** M4-SPAWN-1 spawn engine core (multiplier chain + caps + named RNG stream),
M4-SPAWN-2 biome spawn tables & special rules, M4-TOWN-1 housing & move-in, M4-TOWN-2
happiness & prices, M4-TOWN-3 shop stock (M4-TOWN umbrella), M7-PYLON-1 pylons
(placement + teleport rules, `TeleportPylonsSystem.cs`).

## 16. Drops, loot, bestiary & banners (data side)

**C#** `GameContent/ItemDropRules/*` (38 files), `ItemDropDatabase.cs` (643 registrations),
`FishDropRules/*` · **Rust** ⬜ · ⬜.

- Rule engine: `IItemDropRule` + chained attempts; `ItemDropResolver.TryDropping`
  (`ItemDropResolver.cs:14-44`); factories (`ItemDropRule.cs:5-102`): Common, OneFromOptions,
  NormalvsExpert, ByCondition, ExpertGetsRerolls, WithRerolls, MasterModeCommonDrop,
  BossBag, DropLocalPerClientAndResetsNPCMoneyTo0 (boss bags), DropOneByOne,
  FromOptionsWithoutRepeats; conditions (~71 classes in `Conditions.cs`).
- Money rolls (`NPCLoot_DropMoney` `NPC.cs:78953`): value ×midas ×rand(30-51) % then
  ×rand(−20..76) %, up to 6 bonus rolls (1/2:+5-10 % … 1/64:+50-100 %), blood moon
  ×(1+0-100 %), luck rolls twice; split platinum(74)/gold(73)/silver(72)/copper(71) with
  shrink rolls at >50 count.
- Statue-spawned drops gated by `StatueSpawnedDropRarity` + `AnyInteractions()`
  (`NPC.cs:78849`).
- Bestiary: unlock tiers 0-4 (`BestiaryEntryUnlockState.cs`); kill thresholds from
  `KillsToBanner` at full/½//5 (`CommonEnemyUICollectionInfoProvider.cs:49-64`); tracked
  via `Main.BestiaryTracker` (msg 51 case 5 remote credits).

**Tasks:** M4-LOOT-1 drop-rule engine, M4-LOOT-2 database population, M4-BEST-1 bestiary
data + UI, M4-LOOT-3 money roll chain.

## 17. Projectiles

**C#** `Projectile.cs` (79 791) · **Rust** `projectile.rs`, `world_sim/projectiles.rs`
· ⬜ (shuriken first pass only).

- Engine: pool `Main.projectile[1001]`; recycle = oldest lowest-`timeLeft` non-`netImportant`
  (`Projectile.cs:10815`); `timeLeft` default 3600; `SentryLifeTime = 36000`;
  `extraUpdates` runs AI 1+n times/tick; minion damage recompute adds `5E-06f` epsilon.
- Immunity (three systems): default per-owner `immune[owner] = 10` when penetrating;
  local `localNPCImmunity[npc] = localNPCHitCooldown`; ID-static
  `perIDStaticNPCImmunity[type, npc] = GameUpdateCount + cooldown` shared per **type**
  (`Projectile.cs:13764`); PVP `playerImmune = 40`.
- AI families (aiStyle → IDs): 1 arrows/bullets gravity (156) · 2 thrown spinners (40) ·
  3 boomerangs (20; return speed 12) · 7 grappling hooks (20) · 16 bombs (41; wet →
  `timeLeft = 1`) · 19 spears · 26 glow orbs (59) · 29 gem staffs (13; homing 500 f) ·
  48 beams (5) · 61 fishing bobber (2) · 62 flying minions (6) · 75 channel projections
  (12) · 99 yoyos + counterweights · 100-210 named one-offs (~90) · sentry helpers 53/197
  (`AI_053_HandleSentryNPCTargeting`, max 1000 f).
- Minions: `minionSlots` float (spiders 0.5); slot check on spawn kills oldest
  (`Projectile.cs:16704-16732`); whip hit sets `player.MinionAttackTargetNPC` (14261);
  tag damage: `bonusTagDamage + AddedBaseDamage` × per-projectile multiplier, applied on
  minion hit (`Projectile.cs:12924`); whips `TagDuration = 240` (`WhipTagEffect.cs:14`).
- Yoyos: string 200 f, speed 10, life ×`YoyosLifeTimeMultiplier` (×1.5 with string);
  duplicate yoyos accelerate retirement `Next(10,31)*0.1` (`Projectile.cs:67747`);
  glove duplicates at 16 speed (max 2).
- Flails: kill if >`900×(1/meleeSpeed)` or owner dead or map open (43735); damage
  ×0.35 flying, ×0.5 spinning.
- Hooks (aiStyle 7): kill >2500 f; latch rules per type; per-hook max (default 3,
  Lunar 8, 2/1/4 specials) enforced by killing oldest; hooks destroy cracked bricks 1/16
  per frame.
- Reflection: NPC `ReflectProjectile` — damage ÷4, hostile flip, penetrate 1
  (`NPC.cs:66427`); player parry `shieldParryTimeLeft`, cooldown 15.
- Golf: `GolfHelper.StepGolfBall` gravity 0.3 drag 0.99; clubs wedge/iron/driver
  strength ovals; `AbsoluteStrength = ovalLength·32`; ball rest on tee tile 494.
- Debuffs: `StatusNPC` per-type switch with RNG durations (poison 60·Next(5,10), ichor
  60·Next(10,20)…).
- **Hidden:** lucky coin 1/5 coin → 1/10 silver → 1/100 gold (13688); chlorophyte spore
  1/3 spawns type 228; kites need wind ≥ 0.2; chum fishing bonuses use strict `>`
  comparisons (3rd chum gives nothing more); reflected projectiles lose ¾ damage.

**Tasks:** M4-PROJ-1 engine (slots/immunity/penetrate), M4-PROJ-2 arrows & bullets
families, M4-PROJ-3 boomerangs & flails, M4-PROJ-4 yoyos, M4-PROJ-5 grappling hook AI,
M4-PROJ-6 magic/staff families, M4-PROJ-7 whips & tag damage, M4-PROJ-8 minion & sentry
system, M4-PROJ-9 bombs/explosives, M4-PROJ-10 golf balls & clubs, M4-PROJ-11 hostile
projectiles set 1.

## 18. Bosses & events

**C#** boss AI in `NPC.cs`, events `Main.cs` + `GameContent/Events/*` (11 files) ·
**Rust** King Slime only · 🟡.

- Summon items (`Player.cs:43755-44040`): 560→King Slime, 43→EoC (night), 70→EoW
  (corruption), 1331→BoC, 544→Twins, 556→Destroyer, 557→Prime, 5334→MechQueen (1.4.5 new),
  1133→Queen Bee, 4988→Empress, 5120→Deerclops; non-item: WoF (doll), Plantera/Golem
  (inline), Fishron (truffle worm), Moon Lord (countdown 3600 after 4 pillars,
  `NPC.MaxMoonLordCountdown`, quadratic music silence below 20 %).
- Downed flags (`NPC.cs:6345-6427`): 24 flags stored in the world header.
- Scaling (`ScaleStats` 18146): expert uplift to baseline 80 (+20 post-Plantera);
  per-player `ScaleStats_ByPlayerCount` (18560) soft-capped factors; difficulty curves
  (`GameDifficultyData.cs:83-103`): EnemyMaxLife Journey 0.5→Legendary 4, Damage →Master
  3→Legendary 5.333, Money ×2.5 expert, knockback ×0.8 master.
- Blood moon: `rand.Next(9)==0` (anniversary 6) with `statLifeMax > 120`, never on full
  moon, blocks sundial (`Main.cs:65336-65364`).
- Eclipse: HM + `downedMechBossAny` + `rand.Next(20)==0` at dawn (`Main.cs:65429`).
- Invasions: goblin (post-orb `rand(3)` or `rand(30)`, HM 60), pirate (HM+altars,
  `rand(30|60)`), frost (Snow Globe), martian (probe flee); sizes `80+40·players`
  (+pirate extra), march toward spawn, warning every 3600 ticks (`Main.cs:64414-64460`).
- Moon events: wave points `MoonEventRequiredPointsPerWaveLookup[21]`
  (`NPC.cs:95292`), kill points ×2 expert / ×2.5 master; wave advance
  `CheckProgressFrostMoon` 78445 / `CheckProgressPumpkinMoon` 78588.
- DD2 (`DD2Event.cs`): crystal summon, difficulty auto-pick, Betsy at wave 7 tier 3.
- Cultist: spawn check every 600 ticks (×6 if danger), needs HM + Golem + Skeletron + no
  LOS at dungeon; tablet destroyed → delay 43200 (`CultistRitual.cs:26-87`).
- Party: needs Party Girl, `rand(10)` (anniv 7), ≥5 town NPCs, cooldown `rand(5,11)`
  (`BirthdayParty.cs:81-130`); `afterPartyOfDoom` (FTW) kills townies when it ends
  (`Main.cs:17868`).
- Lantern night: `rand(14)==0` post-Moon-Lord, cooldown `rand(5,11)`, suppresses rain
  (`LanternNight.cs:62-101`).
- Slime rain: window 6:00-11:15, `450000/dayRate` (÷2 pre-King, ×1.5 HM, ×5 if nobody
  "ready"), duration `rand(32400,54000)` (`Main.cs:64927-64946`).
- Meteor: at nightfall `rand.Next(50)==0 && downedBoss2`; falls `time > 16200`;
  `WorldGen.dropMeteor()` sampling (`Main.cs:65238-65491`).
- **Hidden:** seasonal windows: XMas Dec ≥15 (`Main.cs:13081`), Halloween Oct 10-Nov 1
  (13089); anniversary world lowers every threshold (blood 1/6, meteor 1/10, party 1/7,
  rain luck 1/5); boss music selection scans NPCs in a screen+5000 px rect
  (`Main.cs:12215`); music priority chain at `Main.cs:12800-12995` (credits → Torch God →
  boss → eclipse → slime rain → shimmer → town day/night → sandstorm → underworld →
  depth surface → dungeon → snow → evil → jungle → hallow → ocean → mushroom).

**Tasks:** M4-BOSS (per-boss branches), M4-EVENT-1 invasion engine, M4-EVENT-2 moon
events, M4-EVENT-3 blood/eclipse/meteor/slime-rain, M4-EVENT-4 party/lantern/seasonal,
M4-EVENT-5 DD2, M4-EVENT-6 cultist→pillars→Moon Lord chain, M6-MUSIC-1 music selection
priority chain.

## 19. Wiring, traps, statues, tile entities

**C#** `Wiring.cs` (3 431), `GameContent/PressurePlateHelper.cs`, `Tile_Entities/*`
(16 files), `TileEntitiesManager.cs` · **Rust** ⬜ (M3-ACTUATOR only in catalog) · ⬜.

- 4 wire colors; `TripWire` collects 4 lists then `HitWire` BFS with `_wireSkip` and
  multi-trigger counting (`Wiring.cs:525-996`); timers tile 144 periods 60/180/300/30/15
  by `frameX/18`; `CheckMech` queue max 999 (`Wiring.cs:455`).
- Statue cooldowns: NPC statues 30, item 600, boulder 900, dart 200, spear 90, super-dart
  300 (`Wiring.cs:996+`); caps `MechSpawn` ≤3 within 300 px, ≤6 within 800, ≤10 global
  (`NPC.cs:7420`, `Item.cs:49385`).
- Logic gates: lamp tile 420 + gate tile 419 stack, frame 0-5 =
  AND/OR/NAND/NOR/XOR/XNOR; chained via `_GatesNext` (`Wiring.cs:682-780`).
- Teleporters: two 3×2 pads, offset rules, moves players + NPCs (not boss/noTileCollide)
  (`Wiring.cs:3120`); pumps `XferWater` cap 20 in/out.
- Tile entities (registration order = net IDs, `TileEntitiesManager.cs:26-37`):
  0 TrainingDummy · 1 ItemFrame · 2 LogicSensor · 3 DisplayDoll (armor 3 + acc 5 + mount,
  `_dyes[9]`) · 4 WeaponsRack · 5 HatRack (2 hats + dyes) · 6 FoodPlatter · 7
  TeleportationPylon · 8 DeadCellsJar · 9 KiteAnchor · 10 CritterAnchor; `PerformUpdates`
  called from UpdateWorld (`WorldGen.cs:71507`); **beds are pure tiles, not TEs**.
- **Hidden:** tile 226 (temple brick) actuator blocked pre-Plantera; gem-lock button
  frameX/36 == 4 mode; pixel boxes 424/445 with rotation filtering; mass wire cutter
  consumes item stacks.

**Tasks:** M7-WIRE-1 wiring engine, M7-WIRE-2 traps & statues, M7-WIRE-3 actuators
(fold M3-ACTUATOR), M7-WIRE-4 logic gates & sensors, M7-WIRE-5 teleporters & pumps,
M7-TE-1 TileEntity framework + 11 TEs.

## 20. Shimmer, extractinator, falling blocks & interactables

**C#** `GameContent/ShimmerHelper.cs` + `ShimmerTransforms.cs` + `ShimmerUnstuckHelper.cs`,
`ExtractinatorHelper.cs`, `WorldGen` falling/growth parts, `DoorOpeningHelper.cs`,
`PlayerSleepingHelper.cs`, `PlayerSittingHelper.cs` · **Rust** ⬜ · ⬜.

- Shimmer items: `shimmerTime += 0.01/tick` (`WorldItem.cs:518`); chain → coin
  transform → `ShimmerTransformToItem` (tile-139 trophies → 5538/5579/5638/5639/576;
  lunar brick by moon phase → 5401-5408) → critter release (≤50 attempts) → **decraft**
  via `GetDecraftingRecipeIndex` (prefers corruption/crimson variant recipes; locked
  until Skeletron if ingredient 154, until Golem if 1101, pre-ML locks)
  (`ShimmerTransforms.cs:13-52`).
- Shimmer NPCs (`NPC.GetShimmered` 91548): full transform map (`NPCID.cs:10470`):
  zombies→21/202, slimes/critters→676, skeleton variants→449-452, **all town NPCs→677**
  (shimmered variant, `shimmerTransparency = 0.89`).
- Shimmer player: §7 rules + coin luck on coin shimmer (`AddCoinLuck` 18090).
- Extractinator (`ExtractinatorHelper.cs:5`): base 1/5000 → item 1242; money 1/12000
  platinum / 1/800 gold / 1/60 silver; Desert Fossil mode ×3 rate; silt mode item pool
  {2674, 2006, 2002, 2675}.
- Falling blocks: set {53, 234, 112, 116, 224, 123, 330-333, 495}; SP velocity 0.41, MP
  dedupe 60 ticks; damage per type (sand→proj 56 dmg 10) (`WorldGen.cs:86514-86612`).
- Growth ticks (`UpdateWorld` 71495): overground `3e-5·tiles` per tick (×2 notTheBees),
  underground `1.5e-5`, ×1.5 in rain, rate cap 24; trees `Next(20)` per update (gem
  saplings `Next(5)`); cactus water-limit box; herbs stage frames by time-of-day/rain/
  depth (`GrowAlch` 45777); vine growth per family `Next(2..70)`; moss 50 % of updates;
  crystal shards `Next(5)`, density 2 per radius 6, big shard 1/50.
- Spread (`hardUpdateWorld` 69600): runs when hardmode (or remix+getGood); **halved after
  Plantera** via `Next(2)`; 50 % chain `Next(2)==0`; sunflower (tile 27) suppresses evil
  in radius 2; converts grass/stone/sand/ice, never walls; chlorophyte defense takes over
  near chlorophyte; `SpreadGrass` recursion cap 1000; wall spread `wallDist = 3`.
- Interactables: doors need player inside span (`DoorOpeningHelper.cs:71-109`); sleeping
  cancels on danger/blood moon/eclipse (`PlayerSleepingHelper.cs:85`); sitting max 2 per
  chair, counts as fishing +5 (`PlayerSittingHelper.cs:50`).
- **Hidden:** `worldUpdateRate` capped at 24 (sundial); `CountTiles` audit every 30
  ticks; `SlowlyDiesInWater` at >32 liquid; alch seeding `Next(num7*100)==0` with
  `num7 = Lerp(151, 151*2.8, maxTilesX/4200 − 1)`.

**Tasks:** M7-SHIMMER-1 shimmer transforms, M7-GROWTH-1 growth ticks (trees/herbs/vines/
moss/shards), M7-GROWTH-2 spread & chlorophyte, M7-EXTRACT-1 extractinator, M7-FALL-1
falling blocks, M7-INTERACT-1 doors/beds/sitting, M5-SPREAD-1/2 (worldgen side).

## 21. Fishing

**C#** `Projectile.cs` fishing part, `GameContent/FishDropRules/*` (10 files),
`ChumBucketProjectileHelper.cs` · **Rust** ⬜ · ⬜.

- Power: `FinalFishingLevel = (BaitPower + PolePower + fishingSkill + situational +5) ×
  multipliers` (`Player.cs:42200-42204`); multipliers (`Fishing_GetPowerMultiplier`
  42215): rain ×1.2, overcast ×1.1, dawn/dusk ×1.3, midday ×0.8, night ×0.8, moonPhase
  0 ×1.1 / 1|7 ×1.05 / 3|5 ×0.95 / 4 ×0.9, blood moon ×1.1; sitting/wet float +5.
- Bite: `rand.Next(100) <= (level+75)/2` (`Projectile.cs:20583`); pond ≥ **75** tiles,
  full quality **300** (`Projectile.cs:20614-20687`); honey ×1.5 effective water; chum
  +11/+6/+3 for >0/>1/>2 chums (strict `>`).
- Luck: negative cancels bite, positive forces instant (`Projectile.cs:20691-20699`);
  power scaled by luck ×(0.9-1.2 … 1.1+0.3·rand).
- Depth bands: `<worldSurface*0.5` sky / `<worldSurface` / `<rockLayer` /
  `<maxTilesY-300` / else; drop levels thresholds `150·{1,2,7,15,30}/level`
  (`Projectile.cs:21512`); crates `Next(100) < 10` (+15 cratePotion); quest fish
  2450-2487 + remix set.
- **Hidden:** ocean quest check `X < 380 || > maxTilesX−380` with >1000 water; catch
  selection XOR corruption/crimson `Next(2)`.

**Tasks:** M7-FISH-1 fishing engine, M7-FISH-2 drop rules & quest fish.

## 22. Mounts, hooks & riding

**C#** `Mount.cs` (6 671) · **Rust** ⬜ · ⬜.

- 66 mounts; categories: ground quadrupeds (Unicorn 4/12/0.3, horses 3/9/0.25, Scutlix
  8/8/0.4), slimes/bouncers (constantJump, slime-bounce damage 40×minion), hover fliers
  (flightTimeMax 320: Bee/UFO/Drill/Witch Broom/Pirate Ship/Spooky Wood 12/16/0.5),
  partial flight (Rudolph/Pigron 160), swimmers (Turtle cancels drowning), minecarts
  (base 6 + cosmetics), dash mounts 62-65 (Chillet, dashType 6), specials (Santank
  cannon, Drill `UpdateDrill`, roller skates lerp).
- Mount stat merge into player: replace / max-while-flying / additive
  (`MovementStatsAreAdditive`, `Player.cs:26694-26712`); wings suppressed when
  `!mount.CanUseWings` (26678); jump height scales with |xVel| for types 0/1
  (`Mount.cs:2810-2855`); charge damage per type (types 10/47: 60×minion, 44: 100,
  45: 120 — `Player.cs:20145-20200`).
- **Hidden:** bee mount bypasses maxFallSpeed clamp; wet slime bounce counts as ground;
  mount 44/45 charge thresholds differ from the others; `extraFall` per mount (3: +10,
  40: +20).

**Tasks:** M2-MOUNT-1 mount framework + ground mounts, M2-MOUNT-2 hover/flyer mounts,
M2-MOUNT-3 minecart system, M2-MOUNT-4 special-ability mounts.

## 23. UI framework, screens & HUD

**C#** `UI/*` (49 files), `GameContent/UI/*` (170 files, 34 k lines) · **Rust**
`ui/uistate.rs` (layout pass ported), front-end screens ✅, `menu/` ✅ · 🟡.

- UIElement: StyleDimension px/percent + H/VAlign; `Recalculate` outer→inner (margins →
  padding), parent `UIList` gives infinite height (`UIElement.cs:285-321`); hit tests
  iterate children **back-to-front** (324-352); events **bubble child→parent**; double
  click 500 ms, state-change click lockout 200 ms (`UserInterface.cs:41-47`); draw uses
  `Main.UIScaleMatrix`, `OverflowHidden` = scissor rect.
- Inventory UX: ItemSlot contexts 0-33+ (`ItemSlot.cs:44-102`); actions 0-5
  (`PickItemMovementAction` 1484-1633); cursorOverride special clicks: 3 favorite, 6
  quick-trash, 7 shift-transfer, 8 transfer+sync, 9 put-in-UI, 10 quick-sell
  (730-912); right-click stack-split: first delay 30, repeat `stackDelay = 7`, superFast
  after ~2 s (`Main.cs:61785-61803`); coin slots accept only 71-74; shop sell excludes
  coins; journey duplication ignores source decrement (ctx 29).
- Chest UI buttons 0-4 (`ChestUI.cs`): LootAll 393, DepositAll 540, QuickStack 661,
  Restock 871, MoveCoins first for banks.
- Crafting: legacy `CraftingUI` + `NewCraftingUI` (grid, filters station/water,
  `NewCraftingUI.cs:132-171`); craft-from-nearby-chests toggle.
- Screens (all in `GameContent/UI/States/`): CharacterSelect 17, CharacterCreation 23,
  WorldSelect 18, WorldLoad 16, WorldCreation 25 (+Advanced 16 + SecretSeeds 17),
  VirtualKeyboard 15 (contexts 0/1 sign/2 chest name), AchievementsMenu 16,
  BestiaryTest 17, CreativePowersMenu 13, EmotesMenu 16, ManageControls 20, Workshop hub
  13/16/17/18, ResourcePacks 17/15; `IngameFancyUI` open helpers (`IngameFancyUI.cs:35-86`).
- HUD extras: MinimapFrame `DEFAULT_ZOOM 1.05, ×0.975/×1.025` (`MinimapFrame.cs:59-63`);
  boss bars `BigProgressBarSystem` (per-boss registry, generic fallback); team overlay
  radius `teamNamePlateDistance*8`; `GameTipsDisplay`, `InGamePopups`.
- **Hidden:** favorited items rejected by item frame/rack/platter TEs; `Main.mouseItem`
  is a single global; dim-invalid fade 0.5 f; loadout share toggles
  (`RecordLoadoutChange` 2451-2496).

**Tasks:** M7-UI-1 UIElement framework completion (events/hit order), M7-UI-2 inventory
full UX (contexts + cursorOverride + stack-split), M3-CHEST-UI, M7-UI-3 equipment/dye
page, M7-UI-4 settings screens parity (M6-SETTINGS), M7-UI-5 keybinding UI, M4-BEST-1
bestiary UI, M7-UI-6 tooltips system.

## 24. Chat, emotes, achievements

**C#** `Chat/*` (22), `GameContent/UI/Chat/*`, `EmoteBubble.cs`, `Achievements/*` ·
**Rust** ⬜ · ⬜.

- Chat model: `ChatMessage = ChatCommandId + Text` (`ChatMessage.cs:35-60`); commands via
  `[ChatCommand]` attributes, localized prefixes with `_` aliases; incoming unknown
  command falls back to Say; commands: Say, Party, Roll, RPS, Help, ListPlayers, Emote,
  Emoji (bubble duration 360), Death/AllDeath/PVPDeath variants, BossDamage.
- Colors (`ChatColors.cs`): BossOrEvent (175,75,255), World (50,255,130), NPCTravel
  (50,125,255), ServerMessage (255,240,20), Death (255,25,25), Command (250,250,0),
  Error (250,0,0); tags: [c/r,g,b:], [i/s:N], [g:N], achievement/name/plain handlers;
  monitor keeps 500 lines (`RemadeChatMonitor.cs:13`).
- Emotes: `EmoteBubble` server-authoritative, frameSpeed 8, sheet 8×4; 152 emotes; menu
  groups (`UIEmotesMenu.cs:90-97`); NPC reactions `CheckForNPCsToReactToEmoteBubble`.
- Achievements: `Achievement` = conditions set (`Achievement.cs:11`); reporting hub
  `AchievementsHelper.cs` (tile destroyed, pickup, craft, NPC killed, progression,
  equip, nurse/angler service, running pixels, mining…).
- **Hidden:** wire format is `[cmd name][text]` strings, not message ids; msg 25 is the
  legacy chat path, 107 multiline.

**Tasks:** M6-CHAT-1 chat system, M6-EMOTE-1 emote bubbles & wheel, M6-ACH-1 achievements
framework.

## 25. Localization

**C#** `Localization/*`, `Lang.cs` · **Rust** `localization/` ✅ core — not wired · 🟡.

**Tasks:** M6-LOCALIZE-wire (HUD/item names/menus).

## 26. Audio & music

**C#** `Audio/*` (23), XACT banks, `Main.UpdateAudio` · **Rust** `audio.rs`, `sound.rs`,
`tr-content/xact.rs` ✅ engine · 🟡.

**Tasks:** M6-MUSIC-biome-tracks, M6-AUDIO-1 ambience emitters (§5), M6-AUDIO-2 music
box overrides (box 85 enables credits roll, `SceneMetrics.cs:349`).

## 27. Multiplayer protocol

**C#** `Netplay.cs`, `NetMessage.cs` (2 977), `MessageBuffer.cs` (4 510), `Net/*`,
`NetModules/*` (12) · **Rust** `net.rs`, `netglue.rs` (custom flow) · 🔄.

- Magic `"Terraria" + 326`; 2-byte LE length prefix, min 3 bytes; buffers 131 070; port
  7777; `MaxConnections = 256`; client state machine 0-6 (`Netplay.cs:469-543`).
- Message ids 1-162 with named bodies (`MessageBuffer.cs:181-4483`): full list audited —
  1 Connect, 2 Disconnect, 3 PlayerId, 4 PlayerInfo, 5 PlayerInventory, 7 WorldInfo,
  8 SpawnRequest, 10 TileBlock (GZip-ish `CompressTileBlock` `NetMessage.cs:1895-2240`),
  13 PlayerState, 16 PlayerHP, 17 CreateProjectile, 21 ItemDrop(+151 kill), 22
  ItemOwner (7-bit reservation expiry), 23 NPCInfo, 27 Projectile, 28 NPCStrike, 33
  ChestName, 35 KillPlayer, 38 Password, 48 LiquidSet, 51 MiscAction, 55 PlayerBuff,
  59 Wiring, 60 NPCHome, 61 NetSetInventory, 70 NPCKillCount, 73 Teleport, 74/75
  AnglerQuest, 82 NetModule envelope, 85 QuickStack, 86 TeleportPylon, 89/91 EmoteBubble,
  97 BestiaryKillDirect, 100 NPCKill, 103 MoonLordCountdown, 110 PlaceObject, 112
  Loadout, 117 PlayerHurtPVP, 121 TileEntityItem, 124 HatRack, 126/127 RevengeMarker,
  134 Luck, 139 CreditsRoll, 141 LucyAxe, 143 DD2Skip, 147 LoadoutIndex… (dead: 15, 24,
  25, 26, 44, 67, 83, 93).
- Join push (`SyncConnectedPlayer` `NetMessage.cs:2806`): SyncOnePlayer + full inventory
  (msg 5 ×59 + armor/dye/misc/banks/trash/loadouts) + NPC houses/travel shop + angler
  quest + revenge markers.
- NetModules: sequential ushort ids in registration order; packet = [module id][payload];
  modules: Text, Bestiary, CreativePowers(+Permissions/Unlocks/Report), Liquid, Particles,
  Ping, Ambience, TeleportPylon, Debug.
- **Hidden:** msg 21 auto-converts to 151 for air/blocked items; `netSpam` throttle;
  `netImportant` protects from recycling; tile sections via `SendSection`/`ResyncTiles`;
  `DoesPlayerSlotCountAsAHost`.

**Tasks:** M6-NET-protocol (umbrella), M6-NET-1 handshake & states, M6-NET-2 entity sync
cadence, M6-NET-3 tile sync & sections, M6-NET-4 NetModule system, M6-NET-5 parked MP
Back-targets (from P6 review).

## 28. Creative/journey & research

**C#** `GameContent/Creative/*` (17), `CreativeUI.cs` · **Rust** ⬜ · ⬜.

- Powers (`CreativePowerManager.cs:90-104`): time_setfrozen/setdawn/…/setspeed,
  godmode, wind/rain set+frozen, increaseplacementrange (default ON), setdifficulty
  (slider → multiplier quantized 1/20 steps; presets 0.5/1/2/3), biomespread_setfrozen,
  setspawnrate (asymmetric 0.1×-10×; 0 disables for that player).
- Research: sacrifice `SacrificeItem` (`CreativeUI.cs:288`), catalog
  `CreativeItemSacrificesCatalog`, infinite items + filters/sorts; persistence per-world
  (`IPersistentPerWorldContent` → world file section 11) and per-player bools.

**Tasks:** M6-CREATIVE-1 powers, M6-CREATIVE-2 research/sacrifice.

## 29. Save formats & world metadata

**C#** `IO/WorldFile.cs` (3 966), `Player` file IO · **Rust** `save.rs`, `wld.rs`,
`persist.rs` · 🟡 (header+tiles of .wld; .trwld full).

**Tasks:** M6-SAVE-1 remaining `.wld` sections (chests/signs/NPCs/TEs/TownManager/
Bestiary/CreativePowers — sections 4-11, `WorldFile.cs:1150`), M6-SAVE-2 `.plr` reader
(replaces save_list stand-ins).

## 30. Secret seeds & special worlds

**C#** `WorldBuilding/WorldSeedOption_*.cs`, `WorldGen.SecretSeed` (`WorldGen.cs:483,
3022-3055`), `SpecialSeedFeatures.cs`, `DontStarveSeed.cs` · **Rust** ⬜ · ⬜.

- Classic: Drunk 05162020, Celebrationmk10 05162021, ForTheWorthy, NoTraps, NotTheBees,
  Remix (upside-down layers: surface band rockLayerLow→maxTilesY−350), DontStarve
  (`theconstant`), Everything (getfixedboi = FTW+DST+remix+drunk+bees+notraps+anniversary),
  Skyblock (1.4.5 new — `GenerateSkyBlockWorld` 22524 wipes the world).
- 1.4.5 modular micro-seeds (38): paintEverythingGray/Negative, coatEverythingEcho/
  Illuminant, noSurface, extraLivingTrees, extraFloatingIslands, errorWorld,
  graveyardBloodmoonStart, surfaceIsInSpace, rainsForAYear, biggerAbandonedHouses,
  randomSpawn, addTeleporters, startInHardmode, noInfection, hallowOnTheSurface,
  worldIsInfected, surfaceIsMushrooms/Desert, pooEverywhere, noSpiderCaves,
  actuallyNoTraps, rainbowStuff, digExtraHoles, roundLandmasses, extraLiquid,
  portalGunInChests, worldIsFrozen, halloweenGen, endlessHalloween/Christmas, vampirism,
  teamBasedSpawns, dualDungeons, moreLightning, noLightning (+ Variations at ≥4 seeds).
- Cross-seed gates (`SpecialSeedFeatures.cs`): Mechdusa = remix+FTW,
  BossesKeepSpawning = FTW+DST, SpawnOnBeach = anniversary, NoDungeonGuardian /
  RainbowSandAndBlackSandWalls = shimmer-ocean worlds.

**Tasks:** M5-SEED-1, M5-SEED-2 (see §6).

## 31. Minor systems (parked until their milestone)

- **Golf** (`Golf/*`, 823): ball physics §17, scoring 500/1000/2000, clubs. → M7-GOLF-1.
- **Extractinator** → M7-EXTRACT-1. **Fossil** modes in §20.
- **FlexibleTileWand** (`FlexibleTileWand.cs`): placement wands for rubble/coral/
  seashell/present/etc. mapped from items (`Item.cs:49468`). → M7-TOOLWAND-1.
- **Smart cursor** (`SmartCursorHelper.cs`, 2 950): priority chain Axe→Force→Shinies→
  Solids→Wrenches→Hammers→Actuation→WireCutter→Platforms→Tracks→Walls→Seeds→Moss→…
  (`SmartCursorHelper.cs:131-160`); continuity lock; reach from
  `TileReachCheckSettings + tileBoost + blockRange`. → M7-SMARTCURSOR-1.
- **Leashed entities** (`LeashedEntities/*`, 23 files): pets on leash anchors. → M7-PETS-1.
- **Emote reactions**, **NPC petting** (`PlayerPettingInfo`), **Lucy the axe message**
  (`LucyAxeMessage.cs`), **Chum bucket**, **Item shop sellback**, **Pressure plates**,
  **Minecart digger** (`MinecartDiggerHelper.cs`). → fold into their sections.
- **RGB lighting** (`GameContent/RGB/*`, 53 files): peripheral lighting — **exclude**
  (platform-specific, like Social/*).

---

# Milestone 1.5 — performance & scalability (agreed, frozen)

**Product constraint:** this is a 1:1 reimplementation — same "body", different "engine".
The in-game zoom range is the original one (`Main.GameZoomTarget` clamped to **1.0–2.0**);
below that the original switches to the map, so **no LOD or reduced-detail mode is allowed**.
Settings, menu and all parameters stay identical to Terraria. Any optimisation must
produce the same pixels at the same camera/time, verified by screenshot diff.

The map view is *not* the tile renderer: in the original it is a precomputed world texture.
It will be implemented separately (M1.4) and is out of scope here.

### Measured baseline (RTX 4070 Laptop, Ryzen AI 9 365, 1600×900, overlay off)

| Scene | Instances | Frame | Notes |
| --- | --- | --- | --- |
| Surface, zoom 1.5 | 1 612 | 1.86 ms | real gameplay zoom |
| Surface, zoom 1.0 | 3 744 | 1.61 ms | minimum real zoom |
| Cave, zoom 1.0 | 5 795 | 1.89 ms | most cost is light + present |
| Surface, zoom 0.25 (debug only) | 40 630 | 6.16 ms | stress test |
| Cave, zoom 0.25 (debug only) | 77 439 | 8.75 ms | walls = 20 187 instances; most tiles are black |
| Empty sky, zoom 8 | 1 | 1.33 ms | present/compositor floor (p50 1.10) |

Frame times above are 1 200-frame runs of the pre-optimisation binary. The present floor is
bimodal on this laptop (roughly 1.1 ms or 1.85 ms p50 depending on compositor/power state),
so CPU-side phase timings from `TR_STATS` are the reliable signal at real zooms.

### Phase 1 results (done)

All items below were verified against the pre-optimisation binary: screenshots of the same
surface/cave/hell cameras are **pixel-identical (0 differing pixels, overlay off)**. The
debug overlay changed on purpose (more counters).

| Scene | Instances before → after | Frame before → after | CPU tile+light after |
| --- | --- | --- | --- |
| Surface, zoom 1.5 | 1 612 → 1 502 | 1.86 → 1.33 ms | 0.22 ms |
| Surface, zoom 1.0 | 3 744 → 2 954 | 1.61–1.93 → 1.81–1.89 ms* | ~0.3 ms |
| Cave, zoom 1.0 | 5 795 → 3 544 | 1.89 → 1.79–1.90 ms* | 0.48 ms |
| Surface, zoom 0.25 | 40 630 → 23 200 | 6.16 → 3.70 ms | 2.84 ms |
| Cave, zoom 0.25 | 77 439 → 44 356 | 8.75 → 5.58 ms | 4.34 ms |
| Empty sky, zoom 8 | 1 → 1 | 1.33 → 1.33 ms | floor |

\* Within the present-floor noise; GPU instance work is lower, but wall-clock at real zooms
is dominated by presentation.

### Phase 3 results (in progress)

| Scene | Change | Measured result |
| --- | --- | --- |
| D3 | Light cache keeps a 32-tile valid region around the view; pans inside it reuse the map | Panning at zoom 0.25: light **3.31 → 0.03 ms**, draw **4.13 → 0.74 ms** (release, `TR_PAN=0.25`, 600 frames; `TR_NOCACHE=1` gives the old path) |
| F1 (step 1) | Sky/parallax moved to an always-rebuilt back range; the world range is keyed by camera tile + light generation instead of exact camera bits | Panning at zoom 1.5: **71–74 %** world-range hits, draw 0.02–0.03 ms; zoom 0.25 panning: draw **0.74 → 0.28 ms**; static: **98 %** hits, 0.00 ms. Static screenshots pixel-identical (0 differing pixels) |

### Phase 2 results (done)

Every change was A/B-tested against the Phase 1 binary. Surface, cave and hell frames at
zoom 1.0–2.0 and the zoom-0.25 stress views are **pixel-identical (0 differing pixels,
overlay off)**; framed tile/wall counts and instance counts are unchanged.

| ID | Change | Measured result |
| --- | --- | --- |
| C1 + C3 | Parallel light on Rayon's persistent global pool | Seeding and the horizontal sweeps are parallel; vertical sweeps stay serial because row-major storage makes them strided. Cave at zoom 0.25: light **7.67 → 3.05 ms**, full draw **9.09 → 3.80 ms**. Rects below 96k cells keep the sequential path: task dispatch cost more than it saved at 1.0–2.0 zoom (light 0.20 → 0.22 ms). |
| D2 | Light-map cache keyed by rect, sky, glow, flicker and world revision | Static view: light compute **0**. Panning at 0.25 tiles/frame still hits ~50 % of frames (tile rect changes every ~4 frames): cave 0.25 light **3.07 → 1.54 ms**. |
| G6 | Reuse the world instance range when camera/zoom/screen/light/world are unchanged | Static stress: cave 0.25 frame **9.73 → 2.02 ms**, surface 0.25 **4.01 → 2.05 ms** (600-frame averages). At 1.0–2.0 zoom the frame is present-floor bound, so the gain is CPU headroom rather than wall-clock. Panning always misses (camera bits change), as intended. |
| C4 | Banded parallel autotiling (128 rows) | Framing **295 → ~70 ms** (5 911 174 tiles in ~53 ms + 2 691 213 walls in ~19 ms) on the reference 6400×1800 world; outputs identical. |
| C5 | Parallel PNG decode | 258 sheets **68 → ~20 ms**; packed atlas (4096×6454) and every UV unchanged. |
| C2 | Parallel instance build | **Evaluated, not implemented.** The tile loop is already 0.5–0.9 ms at zoom 0.25 and ~0.05 ms at real zoom; the greedy dark-rect merge is order-sensitive and `Lookups::frame_cache` is a lazy `HashMap`, so a parallel build risks pixel identity for a gain hidden by the present floor. |
| B2 | Compress 48 B instances to 20–24 B | **Evaluated, deferred.** 42k instances = 2.0 MB/frame at the 0.25 stress zoom and ~140 KB/frame at real zoom; the frame is present- and CPU-bound, not upload-bound, and RGBA8 tint quantisation would break the exact-pixel constraint. |
| B9 | Sort instances by sheet/page | **Evaluated, deferred.** Everything samples one atlas through one bind group, so sheet sorting cannot improve texture locality; it would only risk changing blend order. |

Startup (excluding window/GPU init) went from **~475 ms to ~210 ms** on the reference
machine. `TR_NOCACHE=1` reproduces the uncached per-frame path for future A/B tests and
`TR_PAN=<tiles/frame>` gives a deterministic moving-camera benchmark.

### Agreed items

| ID | Item | Phase | Status |
| --- | --- | --- | --- |
| A1 | Chunk occupancy (64×64) + skip empty chunks/rows; also used by light later | 1 | done |
| A2 | Skip walls fully covered by an opaque non-sloped block | 1 | done |
| A3 | Darkness culling: quantised light == 0 + opaque content → greedy black rectangles | 1 | done |
| A4 | Chunk-level dark shortcut (one black quad for fully dark chunks) | 1 | done (greedy rows merge full chunks) |
| B1 | Remove per-frame `collect::<Vec<_>>()`; upload world/UI as two ranges | 1 | done |
| B3 | Instance ring buffer (2–3 frames) to avoid GPU/CPU stalls | 1 | done (3 buffers) |
| B4 | Dense per-kind UV tables instead of per-tile `HashMap` | 1 | done |
| B5 | Precomputed wall UV table `[wall][col][row]` (col ≤ 12, row ≤ 4) | 1 | done |
| D1 | Reusable light-map scratch buffers (no 1.7 MB alloc/zero per frame) | 1 | done |
| E4 | Remove redundant bounds checks in the hot loop (row slices) | 1 | done |
| G1 | Measure the ~1.7 ms present/compositor floor, separate it from scene cost | 1 | done (1.10–1.33 ms) |
| G4 | Cache debug-overlay strings (only when the overlay is open) | 1 | done |
| H | `TR_STATS` per-phase timings + culling counters; screenshot-diff harness | 1 | done (`tools/pixel_diff.py`) |
| C1 | Parallel light: seeding + 4 sweeps via rayon (persistent pool) | 2 | done (seeding + horizontal sweeps; vertical serial by design) |
| C2 | Parallel instance build: count → prefix sum → fill, no atomics | 2 | evaluated — present-floor hidden, pixel-identity risk |
| C3 | Persistent rayon pool (no per-frame thread creation) | 2 | done (Rayon global pool) |
| C4 | Parallel world framing at load | 2 | done (~4.2× faster, identical frames) |
| C5 | Parallel PNG decode at load | 2 | done (~3.4× faster, identical atlas) |
| B2 | Compress instances 48 → 20–24 B (u16 tile positions/UV, RGBA8 tint) | 2 | evaluated — upload is not the bottleneck, tint quantisation risks pixels |
| B6 | Merge identical textured runs — only after B7; otherwise skipped | 3 | deferred |
| B9 | Sort instances by sheet/page for texture-cache locality | 2 | evaluated — one atlas and one bind, no locality win |
| D2 | Cache the light map by (rect, sky colour, flicker) when nothing changes | 2 | done (static: 0 ms; panning: ~50 % hits) |
| D3 | Incremental light on camera movement: keep a `CACHE_MARGIN = 32` tile valid region around the visible rect; a pan inside it reuses the map, and the visible rect always keeps the original 28-tile blur padding | 3 | done — surface/cave panning at zoom 0.25: light **3.31 → 0.03 ms**, draw **4.13 → 0.74 ms** (600-frame release runs, `TR_PAN=0.25`; `TR_NOCACHE=1` reproduces the old full path) |
| D5 | GPU light: masks/seeds as textures, blur as parallel compute dispatches | 3 | evaluated — with D3 light is **0.01–0.03 ms** at zoom 0.25–1.5 while the present floor is ~1.1 ms; moving the blur to the GPU cannot improve real-zoom wall-clock and risks float-order pixel drift. Deferred until larger worlds/zoom need it |
| D6 | Deferred lighting: unlit chunk meshes + screen-space light texture | 3 | evaluated — same reason as D5; the CPU light is already cached and the frame is present-bound at zoom 1.0–2.0. Deferred |
| F1 | Chunk meshes (32×32/64×64), rebuilt only on tile edits | 3 | partial — sky split into a back range and the world range cached by tile/light generation (72 % reuse while panning, 98 % static). Per-chunk baking evaluated: the remaining rebuild is 0.02–0.03 ms at zoom 1.5 and 0.28 ms in the 0.25 stress view, below the present floor; deferred |
| B7 | Texture array instead of atlas — internal GPU storage only, must be pixel-identical; enables B6 and cuts cache pressure | 3 | evaluated — one atlas/one bind and no texture-bound cost measured; variable sheet sizes make a pixel-identical array require a layered atlas (same sampling cost) and would touch every UV. Deferred |
| B8 | Manual `fract` UV wrapping for merged runs (nearest, no mips) | 3 | deferred — only matters after B6 merged runs, which depends on B7 |
| B10 | Crop/repack the atlas to frames actually used by the world | 4 | evaluated — a deterministic skyline repack fits the same 186 sheets into 8192×1984 (**113.3 → 65.0 MB**, −43 %, 23 ms), and cave/lake screenshots stay pixel-identical, but the normalised UV phase moves by <1 texel and flips 166 background pixels at the forest horizon (surface, zoom 2.0). Layout-invariant pixel-space sampling is a prerequisite; deferred |
| E1 | Pack `Tile` 16 → 8–10 bytes (bitfields) | 4 | partial — 14 → **12 bytes** by packing slope/paint and the wall frame cell; world memory **70.6 → 60.5 MB**. Going below 12 needs narrower frame fields (only safe for our asset range) — deferred |
| E2 | SoA/chunked tile storage for the light and draw scans | 4 | planned — the 12-byte row-major tile already scans sequentially; a full SoA refactor is a world-model change with no real-zoom frame gain over D3 + the world cache |
| F3 | GPU tilemap (tile metadata in a texture, static grid, vertex-shader culling) | 4 | optional — 2–80 k instances; not needed at the current scale |
| G2 | Present-mode / frame-latency tuning under `AutoVsync` | 1 | done (default latency 1, `TR_LATENCY` override) |
| G3 | Push constants instead of the two globals UBOs | 1 | evaluated — 2×32 B/frame is below noise |
| G5 | Avoid recreating per-frame descriptors where the API allows | 1 | evaluated — surface views have no stable handle |
| G6 | Skip instance rebuild when camera/light/world are unchanged | 2 | done (static camera reuses world range) |

### Explicitly excluded

- **A5 (zoom LOD)** — impossible by design: the original has no zoom below 1.0, and detail
  must never drop.
- **A6 (background/occlusion tricks)** — no measurable benefit in 2D; not worth the
  complexity or the risk to fidelity.
- **C6 (light one frame ahead)** — rejected: visible one-frame light lag and a source of
  entity/lighting bugs later.
- **D4 (light LOD)** — same reason as A5; zoom never goes below 1.0.
- **F2 (4-frame temporal cycle)** — vanilla needs it because it renders into two screen
  targets; our instanced path does not, so it would only add latency.
- **F5 (indirect/GPU culling)** — premature at 2–80 k instances.

### Entities (M4) — data-oriented plan

Entities will be **data-oriented**, not object-oriented: fixed 60 Hz simulation, arrays of
components (SoA) grouped by archetype (player, NPC, projectile, item drop, gore), spatial hash
for broad-phase queries, and deterministic update order. Parallelism only across independent
islands/archetypes with per-thread output buffers; no shared mutable state. This gives DOTS-
like cache behaviour without adopting a full ECS framework or rewriting the world model.
Terraria's own update order and RNG streams must be reproduced exactly, so "parallel by
default" is explicitly not a goal — determinism first.

# Task catalog

Rows marked 🔄 have a live branch; ✅ rows are merged (branch deleted). Read the atlas
section in brackets before starting a row.

## Milestone 1 — visual fidelity

| Task | Branch | Scope [atlas] |
| --- | --- | --- |
| ✅ M1.4-BG-11/16 | `M1.4-BG-11-cloud-parallax` | clouds/stars/horizon [5] |
| ✅ M1.4-BG-13 | `M1.4-BG-13-grass-transition` | grass framing [2] |
| ✅ TILE-DRAW-1 | `TILE-DRAW-1-drawdata-finish` | full static `GetTileDrawData` switch (`TileDrawing.cs:4540-6032`, ~60 cases) + Campfires/564 epilogues + palm/cactus/dye/cog corrections; animator/glow/wind remain TILE-DRAW-2/3 [2] |
| ✅ TILE-DRAW-2 | `TILE-DRAW-2-torch-flames` | torch flames |
| ✅ TILE-DRAW-3 | `TILE-DRAW-3-wind-finish` | wind sway + leaves — determinism verified 2026-11-02: the earlier leaf-gore divergence (84f08e3-era binaries) does not reproduce on 3470372 (6+ SHA256-equal pairs across wind on/off, frames 120/240; per-frame leaf/wind logs identical); standing rule — every screenshot verification is a SHA256 double-run |
| ✅ TILE-DRAW-4 | `TILE-DRAW-4-paint-coatings` | shader-side paint recolor (compiled HLSL not in decompile) + wall paint/coatings in `World.extras` on E1's `paint()`/`set_paint()`; echo/invisible coats remain [2] |
| ✅ TILE-DRAW-5 | `TILE-DRAW-5-grass-framing` | full grass framing |
| ✅ M1.4-BG-18 | `M1.4-BG-18-screen-filters` | SceneState.ApplyVisuals filters + monoliths [5] — done 2026-11-03: post-pass pipeline (offscreen target + ScreenShader.xnb pass table via the tr-content Effect reader), SceneMetrics/Sandstorm/SceneState sensors, WGSL ports (Graveyard/Noir/BloodMoon/Sandstorm fg+bg/Blizzard fg+bg/Heat/Water with mask target); towers/monoliths/Aurora/Sepia OUT (M4/sky/DST) |
| M1.4-BG-19 | `M1.4-BG-19-ambient-skies` | AmbientSky creature entities [5] |
| M1.4-BG-20 | `M1.4-BG-20-ambience-emitters` | ambience sounds + waterfall/lava loops [5] |
| M1.4-BG-21 | `M1.4-BG-21-sundial` | sundial/moondial fast-forward + dayRate plumbing [1][5] |
| M1.4-WEATHER-1 | `M1.4-WEATHER-1-wind-gusts` | wind gust events + extreme wind [5] |
| M1.4-WEATHER-2 | `M1.4-WEATHER-2-sandstorm` | sandstorm severity/visuals [5] |
| ✅ M1.5-LIQUID-2 | `M1.5-LIQUID-2-water-styles` | water styles + shimmer draw |
| M1.5-LIQUID-3 | `M1.5-LIQUID-3-shimmer-distortion` | shimmer vertex distortion + HorizonClouds tint [3] |
| M1.5-LIGHT-3 | `M1.5-LIGHT-3-sun-star-light` | sun/moon/star light, invisible blocks [4] |
| 🔄 M1.4-TO | `M1.4-TO-tile-objects` | TileObjectData umbrella (branch claimed) [2] |
| M1.4-TO-1 | `M1.4-TO-1-doors` | doors open/close + CheckDoor [2] |
| M1.4-TO-2 | `M1.4-TO-2-chests` | chest tiles + CheckChest + locking [2][13] |
| M1.4-TO-3 | `M1.4-TO-3-furniture` | Check3x2/3x3/6x3 family + anchors [2] |
| M1.4-TO-4 | `M1.4-TO-4-herbs-pots` | herbs/pots/alch/sunflower/piles [2][20] |

## Milestone 2 — player

| Task | Branch | Scope [atlas] |
| --- | --- | --- |
| 🟡 M2-DRAW-1 | `M2-DRAW-1-player-layers` | layering done (order + weapon slot + composite front arm + bob); armor/accessories go live with M3-EQUIP |
| ✅ M2-DRAW-2 | `M2-DRAW-2-arm-rotation` | composite arm rotation; the rotation source consumes M2-ANIM use fractions (swing-arc stand-in replaced in the M3-CRAFT merge) |
| ✅ M2-ANIM-foundation | `M2-ANIM-foundation` | anim primitives + faithful draw |
| ✅ M2-ANIM-item-timings | `M2-ANIM-item-timings` | useTime/useAnimation/useStyle |
| M2-PHYS-1 | `M2-PHYS-1-movement-constants` | boots/ice/sand/wet multipliers [7] |
| M2-PHYS-2 | `M2-PHYS-2-fall-damage` | thresholds/extraFall/stoned [7] |
| M2-PHYS-3 | `M2-PHYS-3-fluids-breath` | wet multipliers, drowning fidelity [7] |
| M2-PHYS-4 | `M2-PHYS-4-dash` | dashTypes 1/2/3/6 [7] |
| M2-PHYS-5 | `M2-PHYS-5-wings-rocket-boots` | flight/hover/rocket merge [7] |
| M2-PHYS-6 | `M2-PHYS-6-grapple-movement` | GrappleMovement + per-hook overrides [7] |
| M2-PHYS-7 | `M2-PHYS-7-minecart` | rails/boost/ramp/sparks [7][22] |
| M2-PHYS-8 | `M2-PHYS-8-extra-jumps` | the 8 extra jumps [7] |
| M2-BUFF-1 | `M2-BUFF-1-buff-slots` | 44-slot engine + add/evict rules [8] |
| M2-BUFF-2 | `M2-BUFF-2-regen` | life/mana regen engines [8] |
| M2-BUFF-3 | `M2-BUFF-3-dot-debuffs` | DoT set (poison/fire/ichor…) [8] |
| M2-BUFF-4 | `M2-BUFF-4-stat-debuffs` | slow/weak/webbed/frozen/darkness… [8] |
| M2-BUFF-5 | `M2-BUFF-5-auras` | SceneMetrics auras (campfire/lantern/star/sunflower/candles) [8] |
| M2-BUFF-6 | `M2-BUFF-6-luck` | full Luck.cs (ladybug/torch/coin/kite/potions) [8] |
| M2-BUFF-7 | `M2-BUFF-7-food` | WellFed tiers + foods [8] |
| M2-BUFF-8 | `M2-BUFF-8-banners` | kills + tile scan + BannerStrength [8] |
| M2-COMBAT-1 | `M2-COMBAT-1-iframes-knockback` | cooldown counters + KB caps [9] |
| M2-COMBAT-2 | `M2-COMBAT-2-crit-classes` | crit rolls + damage classes [9] |
| M2-DEATH-1 | `M2-DEATH-1-death-drops` | coin loss/gem drops/tombstones [9] |
| M2-DEATH-2 | `M2-DEATH-2-respawn` | timers + skip rules + bed spawn [9] |
| M2-MOUNT-1..4 | `M2-MOUNT-1-framework` … | mounts per category [22] |
| M2-BUFF | (umbrella retired into M2-BUFF-1..8) | — |

## Milestone 3 — items, inventory, crafting

| Task | Branch | Scope [atlas] |
| --- | --- | --- |
| 🔄 M3-CRAFT-recipes | `M3-CRAFT-recipes` | recipe data + contexts 4/33 |
| M3-CRAFT-2 | `M3-CRAFT-2-groups-environment` | recipe groups + env flags + nearby chests [11] |
| M3-CRAFT-3 | `M3-CRAFT-3-alchemy` | alchemy table 1/3 rule [11] |
| M3-ITEM-1 | `M3-ITEM-1-registry-scaleup` | SetDefaults 1-5 + ItemVariants [10] |
| M3-ITEM-2 | `M3-ITEM-2-use-styles` | 16 use styles + body dispatch [10] |
| M3-ITEM-3 | `M3-ITEM-3-ammo` | ammo system + not-consume rolls [10] |
| M3-ITEM-4 | `M3-ITEM-4-mana` | mana costs + sickness [10] |
| M3-ITEM-5 | `M3-ITEM-5-channel-reuse` | channel + reuseDelay [10] |
| M3-TOOL | `M3-TOOL-tiers` | pick/axe/hammer tiers + use logic |
| M3-EQUIP | `M3-EQUIP-equipment` | slots + armor pages [12] |
| M3-EQUIP-2 | `M3-EQUIP-2-set-bonuses` | ArmorSetBonuses engine [12] |
| M3-EQUIP-3 | `M3-EQUIP-3-accessory-flags` | accessory effect flags [12] |
| M3-EQUIP-4 | `M3-EQUIP-4-dye-slots` | dye plumbing [11][12] |
| M3-PREFIX | `M3-PREFIX` | roll engine + tables [11] |
| M3-REFORGE-1 | `M3-REFORGE-1-goblin` | reforge UI + cost + best-tier [11] |
| M3-CHEST-UI | `M3-CHEST-UI` | chest UI + world chests [13][23] |
| M3-STORE-1 | `M3-STORE-1-banks` | piggy/safe/defender/void [13] |
| M3-STORE-2 | `M3-STORE-2-quick-stack` | loot all/deposit/quick stack/restock [13] |
| M3-STORE-3 | `M3-STORE-3-trash-favorites` | trash + favorites rules [13] |

## Milestone 4 — NPCs, combat, events, projectiles

| Task | Branch | Scope [atlas] |
| --- | --- | --- |
| 🔄 M4-NPC-1 | `M4-NPC-1-slimes-eyes` ✅ (the slime colour family -3..-9 + vanilla stats/tints/gel drops, Wandering Eye 133, `GetBasicSlimeToSpawn` + the Pinky roll; done 2026-11-02) — fighters/cave/etc. to come | AI families [14] |
| M4-NPC-2 | `M4-NPC-2-worms` | multi-segment framework [14] |
| ✅ M4-SPAWN-1 | `M4-SPAWN-1-engine` | spawn engine + multiplier chain: `GetSpawnRate` (`NPC.cs:473-810`) in the vanilla order with the vanilla cast widths — hardmode, the depth ladder (underworld ×2 / deep ×0.4/×1.9 / below-surface ×0.5/×1.7), night ×0.6/×1.3 + blood ×0.3/×1.8 + moon events ×0.2/×2 + eclipse, the dungeon/sandstorm/UD/jungle-town-tiers/evil/meteor/temple/hallow-deep zone branches, the WoF override, both nearby-ramp passes, invis/calmed/sunflower/angler/enemySpawns, the water/peace candles + the >35%-depth candle cut, the rate floor (10 %) and the cap (3×), the moon-event raid pacing (rate 20, max 2+0.3·players) and the pre-Skeletron dungeon rate 10 [15] | done 2026-11-03: `SpawnFlags` carries the scene state (hardmode/blood/moon/zones/candles — all false until the owning systems land); remix/FTW/journey/skyblock/fairy branches constants-off (M5-SEED-1/M6-CREATIVE-1); 8 seeded tests |
| M4-SPAWN-2 | `M4-SPAWN-2-biome-tables` | spawn tables + special rules [15] |
| M4-TOWN-1 | `M4-TOWN-1-housing` | room check + move-in [15] |
| M4-TOWN-2 | `M4-TOWN-2-happiness` | happiness + prices [15] |
| M4-TOWN-3 | `M4-TOWN-3-shops` | shop stock + conditions [15] |
| ✅ M4-LOOT-1 | `M4-LOOT-1-rule-engine` | drop rule engine: `tr-core/src/loot.rs` (`IItemDropRule` + the three chain kinds, `ItemDropResolver.ResolveRule/TryDropping`, `CommonDrop`/`CommonDropWithRerolls`/`OneFromOptions`/`ItemDropWithCondition`/`DropBasedOnExpertMode`/`MasterMode`/`ExtraGel`/`DropLocalPerClientAndResetsMoney`/`DropOneByOne`/`DropNothing`, `Luck.cs`'s five rolls, `ItemDropDatabase` register/multiple/remove) [16] | done 2026-11-02: 13 engine tests on seeded FastRandom streams; the slime family's gel/pink-gel/glove registrations moved from hand rolls to `loot_db.rs` (M4-LOOT-2 grows the rest; the bestiary `ReportDroprates` side stays out until M4-BEST-1) |

| ✅ M4-LOOT-3 | `M4-LOOT-3-money` | the coin roll chain: `tr_core::loot::drop_money` (`NPC.cs:79619-80106` — midas, the ±20-75 % spread, the six bonus chains, the blood-moon bump, the luck double-roll best/worst, `NPC.extraValue`, the platinum/gold/silver/copper split with the >50-count shrink rolls and the truncating loop guard) [16] | done 2026-11-03: 4 seeded tests; `NpcKind::value()`/`SlimeStats.value` from the `SetDefaults` lines; `spawn_coins` rewired from the hand rolls — ⚠ separate find: the waterfall sparkle pass breaks pure-world screenshot determinism (`TR_NO_WATERFALL=1` restores SHA256 equality; bisected 2026-11-03 on f07f96d1+loot, desert-with-water pair) — follow-up `M1.5-LIQUID-2-det-fix` || M4-LOOT-2 | `M4-LOOT-2-database` | 643 registrations [16] |
| M4-LOOT-3 | `M4-LOOT-3-money` | coin roll chain [16] |
| M4-DAMAGE | `M4-DAMAGE-damage` | damage pipeline [9] |
| M4-PROJ-1..11 | `M4-PROJ-1-engine` … | projectiles per family [17] |
| M4-BOSS | `M4-BOSS-bosses` | per-boss branches [18] |
| M4-EVENT-1..6 | `M4-EVENT-1-invasions` … | events [18] |
| M4-BEST-1 | `M4-BEST-1-bestiary` | bestiary data + UI [16][23] |
| M4-REVENGE-1 | `M4-REVENGE-1-coin-loss` | revenge system [9] |
| M4-TOWN | (umbrella retired into M4-TOWN-1..3) | — |

## Milestone 5 — world generation

| Task | Branch | Scope [atlas] |
| --- | --- | --- |
| M5-GEN-1 | `M5-GEN-1-pass-pipeline` | config-driven pass scaffold [6] |
| M5-BIOME | `M5-BIOME-evil-jungle` | evil + jungle [6] |
| M5-BIOME-2 | `M5-BIOME-2-mushroom-marble-granite` | micro-biomes [6] |
| M5-OCEAN | `M5-OCEAN` | full ocean |
| M5-DESERT-structures | `M5-DESERT-structures` | desert + underground desert |
| M5-DUNGEON | `M5-DUNGEON` | dungeon crawler |
| M5-TEMPLE | `M5-TEMPLE` | jungle temple |
| ✅ M5-LIVING-trees | `M5-LIVING-trees` | living trees | ✅ done 2026-11-03 (the generation pass, the hollow passage + wall 244, the 191/192 framing merge, walk-through leaves; the room furniture needs M1.4-TO; deviations in `ANALYSIS.md`) |
| M5-STRUCT-1..5 | `M5-STRUCT-1-shrines-cabins` … | structures [6] |
| M5-ORES-1 | `M5-ORES-1-tilerunner` | ore fidelity + alt tiers [6] |
| M5-HARDFORCE | `M5-HARDFORCE` | initializeHardMode + altars [6] |
| M5-SPREAD-1 | `M5-SPREAD-1-growth` | growth ticks [20] |
| M5-SPREAD-2 | `M5-SPREAD-2-evil-chlorophyte` | spread + chlorophyte [20] |
| M5-CHEST-gen | `M5-CHEST-gen` | chest loot generation |
| M5-SEED-1 | `M5-SEED-1-classic-seeds` | drunk/FTW/remix/anniv/DST [30] |
| M5-SEED-2 | `M5-SEED-2-micro-seeds` | 38 modular seeds [30] |

## Milestone 6 — audio, localization, net, UI systems

| Task | Branch | Scope [atlas] |
| --- | --- | --- |
| M6-MUSIC-biome-tracks | `M6-MUSIC-biome-tracks` | biome branches + SceneMetrics |
| M6-MUSIC-1 | `M6-MUSIC-1-selection-chain` | full priority chain + boxes [18][26] |
| M6-AUDIO-1 | `M6-AUDIO-1-ambience` | ambience emitters [5] |
| M6-LOCALIZE-wire | `M6-LOCALIZE-wire` | wire core into HUD/menus |
| M6-NET-protocol | `M6-NET-protocol` | umbrella [27] |
| M6-NET-1..5 | `M6-NET-1-handshake` … | protocol parts [27] |
| M6-SETTINGS | `M6-SETTINGS` | settings parity |
| M6-SAVE-1 | `M6-SAVE-1-wld-sections` | remaining .wld sections [29] |
| M6-SAVE-2 | `M6-SAVE-2-plr-reader` | .plr reader [29] |
| M6-CHAT-1 | `M6-CHAT-1-chat` | chat system [24] |
| M6-EMOTE-1 | `M6-EMOTE-1-emotes` | emote bubbles [24] |
| M6-ACH-1 | `M6-ACH-1-achievements` | achievements framework [24] |
| M6-CREATIVE-1 | `M6-CREATIVE-1-powers` | journey powers [28] |
| M6-CREATIVE-2 | `M6-CREATIVE-2-research` | sacrifice/research [28] |

## Milestone 7 — world systems & UI framework

| Task | Branch | Scope [atlas] |
| --- | --- | --- |
| M7-WIRE-1..5 | `M7-WIRE-1-engine` … | wiring [19] |
| M7-TE-1 | `M7-TE-1-tile-entities` | TE framework + 11 TEs [19] |
| M7-SHIMMER-1 | `M7-SHIMMER-1-transforms` | shimmer [20] |
| M7-GROWTH-1 | `M7-GROWTH-1-ticks` | growth [20] |
| M7-EXTRACT-1 | `M7-EXTRACT-1-extractinator` | extractinator [20] |
| M7-FALL-1 | `M7-FALL-1-falling-blocks` | sand/silt physics [20] |
| M7-FISH-1..2 | `M7-FISH-1-engine` … | fishing [21] |
| M7-PYLON-1 | `M7-PYLON-1-pylons` | pylons [15] |
| M7-INTERACT-1 | `M7-INTERACT-1-doors-beds` | doors/beds/sitting [20] |
| M7-UI-1..6 | `M7-UI-1-uielement` … | UI framework + screens [23] |
| M7-SMARTCURSOR-1 | `M7-SMARTCURSOR-1` | smart cursor [31] |
| M7-GOLF-1 | `M7-GOLF-1-golf` | golf (parked) [31] |
| M7-PETS-1 | `M7-PETS-1-leashed` | leashed entities (parked) [31] |
| M3-ACTUATOR | (folded into M7-WIRE-3) | — |

## Engineering debt

| Task | Branch | Scope |
| --- | --- | --- |
| REFACTOR-env | `refactor-app-env-flags` | centralize ~65 TR_* flags into app/env.rs (OnceLock) |
| REFACTOR-rng | `refactor-xorshift-unify` | unify worldgen/wind xorshift cores byte-exactly |
| REFACTOR-saveheader | `refactor-save-list-reader` | absorb HeaderReader into binio |
| REFACTOR-cleanup | `refactor-allow-cleanup` | remove inert #[allow(dead_code)] in ui/uistate.rs |
| E1 (paint byte) | — | paint byte landed with TILE-DRAW-4; further Tile packing (12→8-10 B) stays deferred |

## Recommended order of attack (updated)

1. **Finish M1**: M1.4-TO-1/2 (doors/chests unlock M3-CHEST-UI) →
   M1.4-BG-18/19 (filters/ambient skies) → M1.5 leftovers.
2. **M2 combat floor**: M2-BUFF-1/2 (buff+regen engines) → M4-DAMAGE (pipeline) →
   M2-BUFF-3/4 (debuffs) → M2-COMBAT-1/2. These unblock every NPC/weapon task.
3. **M4 in parallel**: M4-SPAWN-1 (engine, with RNG-stream determinism proof) →
   M4-NPC-1 families → M4-PROJ-1 (projectile engine) → M4-LOOT-1.
4. **M5**: M5-GEN-1 pipeline scaffold first — everything else slots into it.
5. **M7**: M7-WIRE-1 + M7-TE-1 (wiring/TEs unlock builds and bosses' arenas).
6. Everything else per capacity; the two-agent rule (agents.md) applies — check
   `git branch -a` first.

## Non-goals (unchanged)

- Modding API / tModLoader compatibility.
- Mobile/console parity.
- Exact network compatibility *until* M6-NET — single-player first.
- RGB peripheral lighting (`GameContent/RGB/*`) — platform-specific, excluded like
  `Social/*`.
