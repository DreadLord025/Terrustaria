# 117 — DOC-SWEEP: clouds / sky (lane 33, W4-g69) — SEALED

**Branch:** `task/W4-g69-weather-clouds-parallax` (continues the DONE `9717c0bb`
cloud-parallax work). **Mode:** code-only — no cargo/tests/graphify invoked;
witnesses WRITTEN NOT RUN. **Scope:** the five clouds/sky follow-ups the Lead
flagged after the g69 cloud lane landed. Companion lanes: `task/W4-g70-bg-layers-1`
(forest sets, tip `b86c8d69`), `task/W4-g71-bg-layers-2` (surface variants, tip
`2b57b362`; its DOC-SWEEP doc `92-doc-sweep-bg.md` owns the v18 layout contract),
`task/W4-g72-bg-layers-3` (cavern/hell styles).

**SEAL (CLOUDS RETIRE, 2026-11-21):** this doc is CLOSED. The retire sweep
added §6 (the g69d-star-visual dispatch census + the small ports), §7 (the
weather-family compose confirmations) and §8 (the 12-deferred-consumers
activation census + the GERunner live-flavour fix), and §9 (the clouds-family
declaration). §1's v18 cross-note is SUPERSEDED by g71's SEALED
`92-doc-sweep-bg.md` §1-SEAL at tip `b706737a1` (the 64a-stable [0..53] prefix
WINS; the crimson byte rides the f45 `.wld` flags dedupe — see §9). Retire
commits: `fd0edd6b6` (consumer 4), `99a67f9ee` (star visual), this seal.

Five items, five verdicts:

| # | Item | Verdict | Evidence |
| --- | --- | --- | --- |
| 1 | TRWLD v18 tail union | **CONFLICT — g71b owns the layout; crimson byte cross-noted below** | 5 distinct v18 blocks across 6 branches; g71b's 19-byte contract covers only g70+g71 |
| 2 | `World.crimson` consumers | **5 small feeds WIRED + drunk dawn flip; 9 large consumers census-recorded** | the field is live (g69 deviation 3); the constants-false stand-ins now read it |
| 3 | cloud SPAWN cadence `numCloudsTemp` | **VERBATIM — verified; one init deviation documented** | `update_num_clouds` re-read line-for-line vs `Main.cs:58952-59036` |
| 4 | rain-cloud darkening | **GLOBAL dim FIXED (per-cloud census: no tint)** | `Cloud.cloudColor` carries no rain tint; the `SetBackColor` dim was a divergent stand-in |
| 5 | star sky static / falling-star VISUAL | **CENSUS — static starfield; falling-star visual UNPORTED** | `StarSprite` has no falling/hidden/fadeIn state; g23b star-pair sound dormant, g70b owns the sky-sweep |

---

## 1. The TRWLD v18 tail union — crimson byte cross-note

### g71b's published contract (read from `origin/task/W4-g71-bg-layers-2`)

`docs/research/92-doc-sweep-bg.md` §1 on the g71 branch is the **layout owner**.
It defines a **19-byte** v18 UNION tail after the v17 `downedFishron` byte:

```
[0..12]  tree_x[0..3]          3×i32 LE   ┐ g70 forest-set block (16 bytes)
[12..16] forest_bg_styles[0..4] 4×u8      ┘
[16]     snow_bg_style          u8        ┐ g71 surface bytes (3 bytes)
[17]     desert_bg_style        u8        │
[18]     ocean_bg_style         u8        ┘
```

Forest-first matches the C# `.wld` field order (treeX/treeStyle at
`WorldFile.cs:1253-1259` precede the setBG bytes at `:1318-1325`). The contract
is **forward-only**: no released v18 exists, so a g70-only or g71-only v18 file
is not grandfathered (both are unmerged task work).

### The conflict is wider than g71b's 2-way — FIVE distinct v18 blocks

A sweep of every `origin/task/W4-*` branch's `save.rs` `VERSION` found **six
branches at v18**, carrying **five distinct tail blocks** (f45 inherits c43's):

| Lane | Branch(es) | v18 block | Size | Cite |
| --- | --- | --- | --- | --- |
| c43 / f45 | `W4-c43-save-header-plr`, `W4-f45-wld-header-2` | world-identity: `generator_version` u64 + 16 GUID bytes + `world_id` i32 + `creation_time` i64 + `last_played` i64 | **44** | `WorldFile.cs:1225-1227`, `:1250-1251` |
| d109 | `W4-d109-framing-genrand-a3` | packed framing variants (`World::variants`, `Tile.bTileHeader2` bits 4-7), dense row-major | **width×height** (variable) | `Tile.cs:464-482`; port-only persistence (the C# `.wld` never writes these) |
| **g69 (this lane)** | `W4-g69-weather-clouds-parallax` | **`WorldGen.crimson` byte** | **1** | `WorldFile.cs:1281` write / `:2080` read |
| g70 | `W4-g70-bg-layers-1` | forest-set: `tree_x` 3×i32 + `forest_bg_styles` 4×u8 | **16** | `WorldFile.cs:1255-1258`, `:1319`, `:1376-1379` |
| g71 | `W4-g71-bg-layers-2` | surface styles: snow/desert/ocean 3×u8 | **3** | `WorldFile.cs:1321`, `:1324-1325` |

g71b's 19-byte contract unions only **g70+g71**. The full merge must also carry
**c43's identity (44)**, **d109's variants (width×height)** and **g69's crimson
(1)**. The "4-way" framing undercounts — there are five blocks.

### Cross-note: the g69 crimson byte position

The crimson byte's natural slot follows the C# `.wld` relative order:
`WorldGen.crimson` (`:1281`) sits **after** treeStyle (`:1256-1259`) and
**before** the setBG bytes (`:1318-1325`). So in the forest-first union it lands
**between the forest block and the surface bytes**:

```
… v17 downedFishron byte (1)
v18 UNION tail (proposed, g71b to ratify):
  [0..16]  g70 forest-set block            (16 bytes, unchanged)
  [16]     WorldGen.crimson      u8        ← g69 (this lane), C# :1281 order
  [17]     snow_bg_style         u8        ┐ g71 surface bytes shift +1
  [18]     desert_bg_style       u8        │
  [19]     ocean_bg_style        u8        ┘
  [20..64] c43 world-identity block        (44 bytes)
  [64..]   d109 variants                   (width×height, LAST — variable size,
                                            self-delimiting from the header dims)
```

**Recommendation:** crimson at offset **16** (C# order), surface bytes shift to
17-19, identity 20-63, variants last. **Alternative** (least disruption to
g71b's published table): append crimson at offset **19** after the surface
bytes, keeping g71b's 16/17/18 stable. Either is correct — the order is
arbitrary but must be *fixed and shared*; g71b owns the ratification. The
variants block MUST be last (its size is `width×height`, known only from the
header, so it cannot precede fixed-size blocks without a length prefix).

**Pre-v18 fallback:** crimson loads `false` (the corruption default, the
vanilla's own `WorldFile.cs:3507` fallback) — g69's reader already does this
(`save.rs:824`, gated `if version >= 18`). The union preserves it.

**Merge instruction:** take the union writer+reader (NOT `--ours`/`--theirs`),
keep `VERSION = 18`, and move every back-compat tail-length constant to the
union size. g69's own constants count the crimson byte as **+1**:
`EMPTY_V16_TAIL = EMPTY_V13_TAIL + 9 + 1 + 5 + 1 + 1` (`save.rs:1325`), the v12
`let tail = … + 1` (`:1296`), the v14 `truncate(len - 10 - EMPTY_V16_TAIL)`
(`:1234`). These become the union total once g71b ratifies the order.

### .wld-side overlap (not the .trwld tail)

Both **g69** (`wld/sections/flags.rs:399`) and **f45** (`flags.rs:422` read /
`:70` write) claim the `.wld` `WorldGen.crimson` flags byte at `WorldFile.cs:2080`
read / `:1281` write. This is the **retail `.wld`** field, separate from the
`.trwld` v18 tail. Keep ONE at merge (both read the same byte into
`World.crimson`; f45 also writes it, g69 reads it — the union should write+read
once). f45 Ph3's "gen stamps" claim and g69's generation-entry stamps
(`worldgen/mod.rs:153/184`, `pipeline.rs:3001`) are the same field — no
behavioural conflict, just a dedupe.

---

## 2. `World.crimson` consumers — census + the small wirings

The field went live in g69 deviation 3 (`2c84e9b6`): set from `GenOptions.crimson`
at all three generation entries, persisted in the `.wld` flags byte and the
`.trwld` v18 tail, read by `CloudEnv::gather` for the `RollRareCloud` type-36
gate. This sweep censuses **every** `WorldGen.crimson` read in the decompile
against the port and wires the small constants-false stand-ins.

### WIRED this sweep (5 feeds + the drunk dawn flip)

| Consumer | C# | Port feed (was `false`) | Now |
| --- | --- | --- | --- |
| Dryad chat lines | `NPC.cs:94038-94080` (332/69 pre-boss2, 333/77 idle slot 0) | `npc_chat::ChatContext.crimson` (`world_sim/npcs.rs:260`) | `app.world.crimson` |
| Dryad shop blood-moon seeds | `Chest.cs:1571-1594` (67/59/4504 vs 2886/2171/4508) | `ShopFlags.crimson_world` (`world_sim/npcs.rs:965`) | `app.world.crimson`; new `BloodMoonCrimson` gate |
| Dryad shop painting split | `Chest.cs:1665-1672` (3217 vs 3218) | `row(3217, Always)` (`shop.rs:374`) | `CorruptionWorld`/`CrimsonWorld` gates |
| Angler quest re-roll | `Main.cs:3769-3817` (2454/2457/2463/2477/2485) | `QuestGates.crimson` (`world_sim/npcs.rs:724`) | `app.world.crimson` |
| EoC loot ore/seed split | `Conditions.IsCrimson/CorruptionAndNotExpert` (`loot.rs:1744-1745`) | `LootFlags.crimson_world` (`world_sim/npcs.rs:6859` default) | `app.world.crimson` |
| `WorldEvilStone` lang token | `Lang.cs:145` (Ebonstone vs Crimstone) | `text("Misc.Ebonstone")` (`lang.rs:225`) | `if world.crimson { Crimstone } else { Ebonstone }` |
| **Drunk dawn evil flip** | `Main.cs:65403-65406` (`crimson = !crimson` every `UpdateTime_StartDay`) | absent | `clock.rs` dawn branch, gated `SeedFlags::DRUNK`, no RNG draw |

`ShopFlags::from_world` also reads `world.crimson` now (was documented "stays
false"). 7 witnesses written not run: the Dryad chat both-halves, the Dryad shop
both-flavours (blood-moon + painting), the angler swap twin-replay both flavours,
the EoC loot engine-replay both flavours, the `WorldEvilStone` crimson flip, the
`from_world` carry, and the drunk dawn flip (flip / flip-back / classic-no-flip).

### Census-recorded, NOT wired (large consumers — their own lanes)

These read `WorldGen.crimson` but need whole subsystems the port lacks; they are
**not** small constants-false flips:

| Consumer | C# | Why deferred |
| --- | --- | --- |
| Powder tile conversion | `Projectile.cs:24191-24202` (proj 10→Convert 11, 11→Convert 1, 463→Convert 4) | proj 10/11/463 are **aiStyle 6** with **no `proj_data` row**; needs the powder AI + `Convert` types 1/4/11 (the port's `spread.rs` has `convert_cell_flavor` but no powder dispatch) |
| Powder NPC evil-transform | `Projectile.cs:14763-14768` (`AttemptToConvertNPCToEvil(type==463)`) | `AttemptToConvertNPCToEvil` unported (`NPC.cs:91927+`) |
| Blood-moon NPC evil-transform | `NPC.cs:91920` (`UpdateNPC_BloodMoonTransformations`) | same `AttemptToConvertNPCToEvil` gap |
| Evil biome spread runner | `WorldGen.cs:76304` (`GERunner`), `:79205` (`IslandHouse`) | `GERunner` unported; the runtime spread (`spread.rs`) reads tile-set membership, not the world flavour |
| Big Mimic spawn kind | `NPC.cs:77597` (473 corrupt / 474 crimson) | the chest-mimic spawn path is unported |
| WoF brick box | `NPC.cs:79047` (347 crimson / 140 corrupt) | `CreateBrickBoxForWallOfFlesh` unported |
| Altar drop | `NPC.cs:79603` (`NPCLoot_DropAltar`, 5533/5532) | the altar-smash loot path is unported |
| Boss-bag open | `Player.cs:7250-7251` (3319: 880/56 + 2171/59) | `Player.OpenBossBag` unported (the bag DROP is registered, `loot_db_gen.rs:1325`, but the OPEN is not) |
| Shimmer decraft variant preference | `ShimmerTransforms.cs:22-29` (`IsCraftedCrimson`/`IsCraftedCorruption`) | the port's recipes carry no crimson/corruption tags (`shimmer.rs:131-133` documents the deviation) |
| Painter shop paintings | `Chest.cs:2178-2185` (2193/4142), `:2211-2218` (784/782) | the Painter shop (case 11) is not rowed in `shop.rs` |
| Guide help-text lines | `Main.cs:39490-39682` (403/205, 402/206, 404/213, 401/222) | the help-text system is unported |
| `Recipe.GetRequiredTileStyle` | `Recipe.cs:16742` (altar 26 style 0/1) | the crafting UI's required-tile name/style is unported |

The audio crimson lead (`audio.rs:2759-2768`) and the ambience/fishing/loot
**zone** crimson reads are `SceneMetrics.ZoneCrimson` (the player's biome), NOT
`WorldGen.crimson` — correctly separate, untouched.

---

## 3. Cloud SPAWN cadence `numCloudsTemp` — VERBATIM

`CloudField::update_num_clouds` (`cloud.rs:610-673`) was re-read line-for-line
against `Main.cs:58952-59036`. **Every roll matches**, in order:

1. `:58952-58955` — `Next(60)==0` → `temp += Next(-1,2)`.
2. `:58956-58959` — `Next(1000) < 50·cloudBGAlpha` → `temp++`.
3. `:58960-58963` — `Next(1300) < 25·(1−cloudBGAlpha)` → `temp--`.
4. `:58964-58967` — `Next(1000) < 200·cloudAlpha && temp<100` → `temp++`.
5. `:58968-58971` — `Next(1000) < 50·cloudAlpha` → `temp++`.
6. `:58972-58975` — `temp>66 && Next(100)==0` → `temp -= Next(1,3)`.
7. `:58976-58979` — `temp<50 && Next(100)==0` → `temp += Next(1,3)`.
8. `:58980-58983` — `cloudBGActive≤0 && temp>100 && cloudAlpha==0` → `temp=100`.
9. `:58984-58987` — `temp<−20` → `temp=−20`.
10. `:58988-59006` — the `cloudAlpha>0 && numClouds<200·cloudAlpha` surge loop
    (`+= Next(30)`, cap 200, `temp` follows).
11. `:59007-59011` — `weatherCounter--`; `>0` returns.
12. `:59012-59022` — the jump: `Next(2)==0` → (`Next(2)==0` ? `+= Next(250)` :
    `+= Next(100)`).
13. `:59023-59030` — clamp `temp` to 0..200.
14. `:59031-59032` — `numClouds = temp`; `weatherCounter = Next(3600,10800)`.

**Wiring confirmed:** `world_sim/weather.rs:185-189` runs it once per `dayRate`
iteration, right after the wind walk (`wind.update_rain`), on `NamedRngs::global`
—the C# `UpdateWeather` order (`:58830-58950` wind, then `:58952` cloud tail),
inside the `for i in 0..dayRate` loop (`Main.cs:17228-17231`). The
`updateCloudLayer` state machine runs ONCE per tick before the loop
(`weather.rs:173`, C# `:17226`).

**One init deviation (documented, not a bug):** the C# ctor sets
`numClouds = 200` and copies it to `numCloudsTemp` (`Main.cs:66839-66840`), then
`PostContentLoadInitialize` re-rolls `numClouds = Main.rand.Next(200)`
(`:10876`). The port keeps the ctor 200 (`cloud.rs:231`). This is benign:
`weatherCounter` starts 0, so the **first** `update_num_clouds` tick fires the
jump (`:59007-59031`) and overwrites `numClouds` with the clamped temp before any
in-world draw reads it. Wiring the load-time `Next(200)` draw would shift the
bare global stream every sibling witness pins (the wind walk, the overcast
rolls), so it stays a documented init deviation.

**Witness added:** `w4_g69_the_weather_counter_jump_replays_the_full_cadence`
(`cloud.rs`) — a full draw-order state pin of the jump including the 50/50 arm,
the clamp, the `numClouds` copy and the `Next(3600,10800)` re-arm range.

---

## 4. Rain-cloud darkening — per-cloud census + the GLOBAL dim fix

### Census: `Cloud.cloudColor` carries NO per-cloud rain tint

`Cloud.cloudColor` (`Cloud.cs:227-235`) is `bgColor × min(scale·Alpha, 1)`,
byte-truncated per channel — **no `cloudAlpha`/rain term**. The rain-cloud
darkening is two separate mechanisms:

1. **Dark storm TYPES 18-21** — `addCloud` (`Cloud.cs:116-127`) spawns the dark
   sheet types while raining (`cloudAlpha>0`, 3-in-4) or overcast
   (`cloudBGActive≥1`, coin flip), with the scale≥1/≥1.15 Y-lifts. **Already
   landed + witnessed** (`cloud.rs` `rain_spawn_takes_the_dark_type_branch`).
2. **The GLOBAL sky dim** — `SetBackColor` (`Main.cs:63546-63552`) multiplies
   every sky channel by `1 − cloudAlpha·0.9·atmo`, byte-truncated, gated on
   `cloudAlpha>0 && !remixWorld`. The clouds read `Main.ColorOfTheSkies`
   (`NextHorizonRenderer.cs:245`), so they inherit this dim.

`BetterColorsForClouds` (`NextHorizonRenderer.cs:265-294`, the sunrise/sunset
gradient blend) has **no call site** in the 1.4.5.8 decompile — dead code, not a
gap (the g69 deviation-4 note already records it).

### The GLOBAL dim was a divergent stand-in — FIXED

The port's two sky-tint sites (`app/frame/draw.rs` and `world_sim/scene_scan.rs`)
ran `(1 − visual_alpha·0.9).max(0.1)`:
- **clamped `visual_alpha`** instead of RAW `cloudAlpha` (wrong for over-1 pins),
- **no `atmo` factor** (the C# multiplies by `Main.atmo`),
- **no remix exclusion** (`!Main.remixWorld`),
- a **0.1 floor** the C# lacks,
- **no byte truncation**.

New shared helper `render::sky::rain_dim_sky(sky, cloud_alpha, atmo, remix)`
ports the formula verbatim: the sky channel is truncated to its XNA byte first
(`(v·255) as i32 as u8`, the `draw_cloud_color` `ColorOfTheSkies` convention),
then `× num3` with the unchecked byte cast (`as i32 as u8` — a >1.11 cloudAlpha
pin wraps the negative product through the low byte; vanilla `maxRaining ≤ 1`
keeps `num3 ≥ 0.1`). Both sites read it and feed `modify_horizon_light` the RAW
alpha (`GetVisibilities` `:202` reads `Main.cloudAlpha` unclamped).

**Witness added:** `w4_g69_rain_dim_sky_is_the_global_byte_truncated_formula`
(`sky.rs`) — 7 cases: full dim, half dim, atmo-0 no-dim, alpha-0 gate skip, remix
skip, the over-1 wrap, and per-channel truncation on a non-white tint.

---

## 5. Star sky static / falling-star VISUAL — census

### The port's starfield is STATIC (twinkle/rotation only)

`make_stars` (`sky.rs:384-424`) ports `Star.SpawnStars` (`Star.cs:86-158`) in its
RNG order (count `Next(200,400)`, the 1920×1200 design-space positions, the
twinkle/rotation speeds with sign coins, the 1-in-40 giant). `StarSprite`
(`sky.rs:321-334`) carries **only** `x/y/scale/tex/phase/twinkle_speed/rotation/
rotation_speed` — **no `falling`, `hidden`, `fadeIn`, `fallSpeed`, `fallTime`**.
`draw_sky` (`sky.rs:704-740`) derives twinkle/rotation from the pinned
`anim_time` clock (sine-approximated), **not** integrated `Star.Update` state.

### The falling-star VISUAL is UNPORTED (census, not wired this sweep)

The C# falling-star visual chain:

| Piece | C# | Port status |
| --- | --- | --- |
| `Star.Fall()` | `Star.cs:160-169` (fallTime=0, falling=true, fallSpeed Y `Next(700,1001)·0.01`, X `Next(-400,401)·0.01`; gated `!anySecretSeedIsActive \|\| starGame`) | **absent** — no `Fall` |
| `Star.Update` falling branch | `Star.cs:173-205` (fallTime += dayRate, position += fallSpeed·(dayRate+99)/100, hide at Y>1500, twinkle ×3 speed, rotation += 0.5) | **absent** — static field |
| `DrawStar` falling trail | `Main.cs:63140-63171` (the `fallTime`-capped-30 trail loop, `fallSpeed·j·0.4` offsets, the `1−j/30` scale/colour fade, the starGame mouse-hit redirect) | **absent** — `draw_sky` draws only the static sprite |
| `Star.StarFall(x)` | `Star.cs:62-84` (the nearest-star pick by design-space X) | **absent** |
| `Star.NightSetup()` | `Star.cs:41-60` (the `starfallBoost` roll: 1-in-10 → `Next(300,501)·0.01`, else 1-in-3 → `Next(100,151)·0.01`; anniversary 5/2) | **absent** — no `starfallBoost` |
| `Star.UpdateStars()` | `Star.cs:237-260` (the dayCheck respawn at time≥27000, the per-star Update) | **absent** |
| Falling-star spawn trigger | `Main.cs:17259-17272` (the shimmer-arm `Next(90)` Fall + respawn), `:19865-19871` (the menu `Next(12)` Fall) | **absent** |
| Star-cannon / Starfury sky link | `Projectile.cs:53107` (`AI_148_StarSpawner` → `Star.StarFall`) | the proj AI is unported (aiStyle 148) |
| Enchanted Nightcrawler spawn | `NPC.cs:2406` (`numClouds≤55 && cloudBGActive==0 && starfallBoost>3`) | reads the LIVE cloud statics now, but `starfallBoost` is absent |

**Lane boundaries:** the falling-star **sound** pair (cases 37/38,
`LegacySoundPlayer.cs`) is **g23b's** (`task/W4-g23-sounds-main-3`, dormant —
`sound.rs:298/450-460` carries the case-38 `starGame` ×0.15 cut but no
`starGame` state). The **sky-sweep** (the `DrawStarsInBackground` visibility
gate, the celestial dims) is **g70b's** (`task/W4-g70-bg-layers-1`) — this lane
**skips** it per the Lead's instruction.

**Cross-note for g70b (read, not touched):** the port's star visibility gate
(`sky.rs:704`) reads `255·(1 − storm) − skyR − 25 > 0` where `storm` is
`rain.cloud_bg()` (**cloudBGAlpha**, passed at `draw.rs:1141`). The C#
(`Main.cs:63051`) reads `255·(1 − cloudAlpha·atmo) − skyR − 25 ≤ 0` —
**cloudAlpha·atmo**, not cloudBGAlpha. g70b's sky-sweep adds a `cloud_alpha`
param to `draw_sky` (its diff shows the intent); the gate should read that, not
the storm deck. Flagged here so the two lanes don't both rewrite it.

**Verdict:** the static starfield is a faithful `SpawnStars` + `DrawStar`
brightness port; the falling-star VISUAL (Fall/Update/trail/NightSetup/
starfallBoost) is a coherent unported subsystem — a follow-up lane, not a small
fix. No code change this sweep (census only).

---

## 6. g69d-star-visual — the dispatch census + the small ports (RETIRE)

The dispatch asked: any unclaimed star-twinkle/cloud-parallax arm? Census at
the sibling tips (read from origin, no checkouts):

### Claim map

| Arm | C# | Owner at its tip | Status |
| --- | --- | --- | --- |
| Static starfield spawn | `Star.cs:86-158` | **g69** (`make_stars`, `sky.rs:408`) | landed (seed-pinned deviation documented) |
| Celestial dims (`DrawSunAndMoon` num12/num13) | `Main.cs:62848-63028` | **g70** (`752655914`: `cloud_alpha` param + `sky_color` byte chain, its `sky.rs:903/:1019`) | landed on g70 |
| Star visibility GATE | `Main.cs:63051` | **UNCLAIMED at g70's tip** — g70 added the `cloud_alpha` param (its `sky.rs:722`) but its gate (its `sky.rs:815`) STILL reads `255·(1−storm)`; the param feeds only the celestial dims | **PORTED HERE** (`99a67f9e`) |
| Graveyard star-opacity head | `Main.cs:63037-63045` | unclaimed (g70's comment still says "not modelled", its `sky.rs:812`) | **PORTED HERE** |
| Shimmer star heads (sky dim + pass skip + i%3 tint) | `Main.cs:63047-63050`, `:62086-62088`, `:63087-63110` | unclaimed (no `shimmer` hit in g70's sky.rs) | **PORTED HERE** |
| `DrawStar` brightness/opacity chain | `Main.cs:63066-63086` | g69 (partial — no graveyard/shimmer terms) | **COMPLETED HERE** |
| `DrawStar` skyblock bgTopY zero | `Main.cs:63114-63117` | unclaimed | **PORTED HERE** |
| Falling-star SOUND pair 37/38 | `LegacySoundPlayer.cs:1329-1350` | **g23** (`945391e33`: `main_sounds_batch3.rs:328-377` — the `star_game`/`fall_speed_y`/70px hit gate coded, "DEFERRED — no live consumer") | stays g23's, dormant |
| Falling-star VISUAL subsystem | `Star.cs:41-84/:160-260`, `Main.cs:63140-63171`, `:17249-17274`, `:19865-19871` | **UNCLAIMED everywhere** (`starfall_boost`/`fall_time` grep across g70/g71/g72/d130/g68/g23: only `npc_ai_style67.rs`'s unrelated fall-timer hits) | **TASK-SEED below** |
| Cloud-parallax residual arms | `Main.cs:59644-59703` | g69 — the three cY formulas + bucket gates + byte chain all landed (`cloud_distant_cy`/`cloud_closer_cy`/`cloud_closest_cy`, witnesses `cloud_pass_cy_formulas_are_verbatim` + `draw_cloud_color_replays_the_byte_chain`) | **NONE unclaimed** |

### The small ports (`99a67f9e`)

Two shared helpers in `render/sky.rs`, both read by `draw_sky`:

* **`star_pass_heads`** (`Main.cs:63030-63058`): the graveyard opacity
  `1 − GraveyardVisualIntensity·1.4` with the ≤0 field kill; the shimmer sky
  dim `colorOfTheSkies *= 1 − shimmerAlpha` (XNA byte multiply) feeding BOTH
  the gate and `DrawStar`; the gate `remixWorld || 255·(1 − cloudAlpha·atmo)
  − skyR − 25 > 0` on the RAW cloudAlpha (the `storm`/cloudBGAlpha stand-in
  retired — the §5 cross-note is DISCHARGED); the `shimmerAlpha == 1`
  whole-pass skip (`:62086-62088`).
* **`draw_star_color`** (`Main.cs:63066-63110`): the per-channel `(int)`
  truncation BEFORE the /3 average, the `(double)·1.4` cap-255,
  `color *= starOpacity`, the shimmer i%3 channel tint (**0→G, 1→B, 2→R** —
  `:63090-63100`; the `(1f − num·0f)` term is the literal 1; the two
  sequential twinkle multiplies), `Color.Lerp` byte lerp, and the
  artificial-false `*= 1 − num`. The `artificial: true` second call site
  (`:62203`, the shimmer-surface star pass, `*= num` at `:63105`) is
  UNPORTED with its surface pass — noted, not faked.
* `DrawStar`'s `worldSurface <= 30` bgTopY zero (`:63114-63117`) inline in
  the star loop.

`draw_sky` gains `graveyard_intensity`/`shimmer_alpha`/`cloud_alpha` params,
fed from `scene_state.graveyard_visual_intensity`/`scene_state.shimmer_alpha`
/`rain.raw_alpha()` at `app/frame/draw.rs`. **Merge note for g70:** its tip
adds a `cloud_alpha: f32` param with the SAME name and semantics (RAW
`Main.cloudAlpha`) — ONE param at the merge, both bodies read it; g70's
celestial-dim uses and this gate fix compose without overlap.

### TASK-SEED — the falling-star visual lane (unclaimed)

A coherent follow-up (suggested slug `W4-hXX-star-fall-visual`), NOT a small
fix — it needs integrated per-star state the static field lacks:

1. `StarSprite` gains `falling/hidden/fadeIn/fallSpeed/fallTime/velocity`
   (`Star.cs:23-39`); `Star.Update`'s falling branch (`:173-205`: fallTime +=
   dayRate, position += fallSpeed·(dayRate+99)/100, hide at Y>1500, twinkle
   ×3 speed, rotation += 0.5) and the normal branch (`:207-234`: fadeIn
   decay 6.1728395E-05·dayRate·10, the 0.6..1 twinkle bounce, the 6.28
   rotation wrap) — replacing the sine approximation ONLY when the lane
   lands (the anim_time pin is a documented deviation until then).
2. `Star.Fall` (`:160-169`, the `!anySecretSeedIsActive || starGame` gate,
   Y `Next(700,1001)·0.01`, X `Next(-400,401)·0.01` on `Main.rand`),
   `Star.StarFall(x)` (`:62-84`, nearest non-hidden non-falling by
   design-space X), `NightSetup` (`:41-60`, starfallBoost 1-in-10
   `Next(300,501)·0.01` else 1-in-3 `Next(100,151)·0.01`, anniversary 5/2;
   called at `UpdateTime_StartDay`'s night arm `Main.cs:65223`).
3. `UpdateStars` (`:237-316`): the dayCheck respawn at day-time ≥27000
   (`SpawnStars(i)`'s 10-candidate max-min-distance pick `:100-126`), the
   per-star Update, and the menu-secret-seed gravity sim (`:261-316`).
4. The `DrawStar` falling trail (`Main.cs:63140-63157`: fallTime capped 30,
   `fallSpeed·j·0.4` offsets, `1−j/30` scale/colour fade) + the starGame
   mouse-hit redirect (`:63158-63170`: <70px, normalize·10, Y-flip, sounds
   38+37 — **g23's dormant pair activates HERE**, its
   `main_sounds_batch3.rs:375` gate is the ready seam).
5. Spawn triggers: the shimmer-arm `Next(90)` Fall + hidden respawn
   (`Main.cs:17256-17273`), the menu `Next(12)` Fall (`:19865-19871`), the
   star-cannon `AI_148_StarSpawner` link (`Projectile.cs:53107`, aiStyle 148
   unported), and the Enchanted Nightcrawler spawn gate (`NPC.cs:2406`:
   `numClouds≤55 && cloudBGActive==0 && starfallBoost>3` — reads the LIVE
   cloud statics already).

The `starGame` seed state itself (`WorldGen.cs:11212-11215`) is the lane's
prerequisite; g23's audio census already documents it.

---

## 7. Weather-family compose — CONFIRMED (g67f + g61)

### g67f-8b9a1ff4 rain-seal (research/146, the WEATHER-FAMILY bible)

Read at `origin/task/W4-g67-weather-rain-splash` tip `8b9a1ff46` (its
`docs/research/146-g67-post-union-folds.md`, SEAL paragraph): *"**g69**
(`task/W4-g69-weather-clouds-parallax`) owns the cloud draw/parallax lane
including `rain_dim_sky` (the XNA byte-truncate-FIRST convention,
`98067533`) — the `cloudAlpha`/`cloudBG` VALUES it reads come from d130's
rain state, g69 keeps its draw bodies."* **CONFIRMED — the compose is
clean:**

* g69's draw bodies (`cloud.rs`, the three `DrawClouds_*` passes,
  `draw_cloud_color`, `rain_dim_sky`, `cloud_bg_draw_step`) read only
  `Rain` accessors + `CloudField` statics — no rain-state writes outside
  `cloud_bg_draw_step` (the per-DRAW `cloudBGAlpha` ramp, `Main.cs:59358-59373`).
* d130's tip (`b8f655708`, research/83 s9 SEALED — the mutual receipt)
  carries the accessor surface g69 reads: `raw_alpha`→**`cloud_alpha()`**
  (d130 `rain.rs:303`, the unclamped `Main.cloudAlpha`), `visual_alpha()`
  (`:297`), `max_raining()` (`:292`), `is_raining()`/`is_it_raining()`
  (`:309/:321`), `cloud_bg()` (`:373`). **One rename at the merge:** g69's
  `rain.raw_alpha()` call sites (`draw.rs:312/:1092/:1160`,
  `scene_scan.rs:617`) become `rain.cloud_alpha()` — same value, d130's
  name wins (its s8 d130-canonical declaration).
* **One divergence to resolve at the merge (flagged, NOT touched here):**
  d130's `rain.rs:504-516` still runs its own per-TICK `cloud_bg` ramp
  (`CLOUD_BG_RAMP_PER_TICK` 0.005, the intensity-chasing stand-in g69's
  deviation fix retired on ITS branch). g69's `cloud_bg_draw_step` (per-DRAW
  ±0.0005·dayRate by `cloudBGActive`, the C# verbatim) is the canonical
  schedule per the 146 bible's "g69 keeps its draw bodies" — the merger
  drops d130's tick-side ramp block and keeps g69's draw-head step + the
  `weather.rs` gate chain. Both branches' `cloud_bg()` accessor stays.
* d130's `draw.rs:304-309` still carries the OLD `(1 − visual·0.9).max(0.1)`
  sky-dim stand-in — g69's `rain_dim_sky` (byte-truncate-first, `98067533`)
  supersedes it per the bible; keep g69's two call sites.
* The star-visual ports (§6) read `scene_state` + `rain.raw_alpha()` only —
  they compose with d130's state the same way (the rename applies).

### g61-NightKey-crimson (research/238 at its SEAL tip `6e922fc60`)

Read at `origin/task/W4-g61-npc-spawn-dust`: its §2 declares the Night Key
pick LIVE — `chest.rs::big_mimic_summon_check` (`:1370`, the `:1469-1472`
comment) reads `app.world.crimson`, the g69 field hunks carried VERBATIM
(its `git diff --no-index` against `20d0380f` EMPTY for both files' crimson
hunks). **CONFIRMED:** the field/ctor/worldgen hunks dedupe to one at the
merge; the `.trwld` v18 tail + `.wld` flags read stay g69's (g61 does not
duplicate them); loaded worlds keep the corruption default until g69's save
layer merges — g61's own note, still true. Its witness
`w4_g61_the_crimson_night_key_clears_and_reads_the_world_evil` rides the
same field. NOTE: g61's doc renumbered 237→**238** at its seal (registry-150
first-pushed-wins, e23 owns 237) — cite `238-g61-spawn-family-final.md`.
The big-mimic kinds 473-476 AUTO-activate at the d40 merge (`8fe1d6d31`,
own bestiary rows) per its §3 — consumer 5 below.

---

## 8. The 12 deferred consumers — activation census (g71e-92 §5's ask)

g71's SEALED 92 (`b706737a1`) §5 lists "g69's 12 deferred consumers" as a
merge-scoped gap. Re-censused at the CURRENT origin tips — **which activate:**

| # | Consumer (C#) | Verdict | Evidence |
| --- | --- | --- | --- |
| 1 | Powder tile conversion (`Projectile.cs:24191-24202`) | **ACTIVATED-BY-d1** — but NOT a crimson consumer: the Convert types are per-PROJECTILE constants (10→11, 11→1, 463→4), no `WorldGen.crimson` read in the arm (re-read `:24180-24209`). d1's `projectile_ai_powder.rs:187-189` ports all three verbatim through `convert_sized` | d1 tip `212516a82`; §2's row was over-broad — the powder TILE arm never reads the world evil |
| 2 | Powder NPC evil-transform (`Projectile.cs:14759-14770`) | **STILL-DEFERRED, flavour-corrected** — the crimson bool is `this.type == 463` (`:14763`), NOT `WorldGen.crimson`; the gap is `AttemptToConvertNPCToEvil` (`NPC.cs:91928-91963`: 46/303/337/443/540→464/47, 55/230/592/593→465/57, 148/149→470/168). No port on d1/g61/c10 | grep `convert_npc_to_evil|AttemptToConvertNPCToEvil` across d1/g61/c10: zero hits |
| 3 | Blood-moon NPC evil-transform (`NPC.cs:91911-91926`) | **STILL-DEFERRED** — this one DOES read `WorldGen.crimson` (`:91920`); same `AttemptToConvertNPCToEvil` gap + the `value==0` save/restore dance | no `blood_moon_transform` port on g61/c10/b3/d30 |
| 4 | Evil biome spread runner (`WorldGen.cs:76304` GERunner) | **ALREADY-WIRED-ON-BASE + FIXED THIS SWEEP** — §2's "GERunner unported" row was STALE: `spread.rs:605` `gerunner` + `:680` `initialize_hard_mode` port the V-strip on this branch's own base. The feed was the stale `(seed>>56)&1` stand-in; `fd0edd6b6` re-points it to `world.crimson` (the C# reads the static INSIDE the convert arm `:76304`, after the wobble roll — flavour changes no draw). The IslandHouse half (`:79205`) is g44's: its `islands.rs:1198` reads the `crimson` param fed `opts.crimson` at `pipeline.rs:2632` — ACTIVATED-BY-g44 (`9ce512b4f`, the islands-family keeper; d103's `island_houses.rs` is the DROP side per its matrix-88 row 73) | `growth.rs:27/48/66`, `micro_surface.rs:425`, witness `w4_g69_the_hardmode_v_reads_the_live_world_evil` |
| 5 | Big Mimic spawn kind (`NPC.cs:77594-77598`) | **ACTIVATED-BY-g61** — `chest.rs::big_mimic_summon_check` reads `app.world.crimson` (§7 above); kinds AUTO at d40 | g61 tip `6e922fc60`, research/238 §2-§3 |
| 6 | WoF brick box (`NPC.cs:79047`) | **STILL-DEFERRED** — no `brick_box`/`BrickBox` port on d35/g36-g39/f24/f25/d101-d106 | grep zero hits |
| 7 | Altar drop (`NPC.cs:79600-79611`, 5533/5532) | **STILL-DEFERRED (half-adjacent)** — g86 (`e6d02d2f0`) ports the altar SMASH (`ores.rs:163` + `editing.rs:1303-1311`) but not `NPCLoot_DropAltar` (the altar-NPC loot path); the item rows 5532/5533 exist (`itemdata.rs:5698-5699`) | grep `DropAltar` on g86: zero |
| 8 | Boss-bag open (`Player.cs:7250-7251`) | **STILL-DEFERRED** — the bag DROP rows are registered (`loot_db_gen.rs:1325` etc.) on g87-g90 but no `OpenBossBag`/`open_boss_bag` port anywhere | grep zero hits across g87/g88/g89/g90/f16/e19/f47 |
| 9 | Shimmer decraft variant preference (`ShimmerTransforms.cs:22-29`) | **ACTIVATED-BY-e19** — `shimmer.rs:149` `decrafting_recipe_index(world_crimson, …)` ports the preference; `ShimmerContext.world_crimson` fed `app.world.crimson` at `:418`; the recipe `crimson`/`corruption` tags landed in `recipes_gen.rs` (15 `crimson: true` rows) — §2's "no tags" note is STALE at e19's tip. e19 also carries g69's `World.crimson` field + v18 save hunks (its `save.rs:434-438/:824` cite W4-g69) — dedupe at merge | e19 tip `e7e466d46` |
| 10 | Painter shop paintings (`Chest.cs:2178-2185`, `:2211-2218`) | **ACTIVATED-BY-d122 (rows), feed OPEN** — d122's `shop.rs:1001-1002` rows 2193/4142 + `:1018-1019` 784/782 ride `StockWhen::CrimsonWorld`; BUT its `ShopFlags::from_world` (`:222`) still defaults `crimson_world: false` (`:189`, the "port persists no world-evil type" comment `:219-221` is STALE — g69's field exists). **Merge step:** d122's `from_world` gains `crimson_world: world.crimson` (g69's own `shop.rs:124` already does this on ITS superset rows — the keep-one is d122's shop per its 142 FINAL, so the ONE-LINE feed graft is the merger's) | d122 tip `962084663` |
| 11 | Guide help-text lines (`Main.cs:39490-39682`) | **STILL-DEFERRED** — no `help_text`/`guide_help` port on c24/d120/f38/c25 | grep zero hits |
| 12 | `Recipe.GetRequiredTileStyle` (`Recipe.cs:16742`) | **STILL-DEFERRED** — no `required_tile` port on g58/c24/d81/d82 or HEAD | grep zero hits |

**Scoreboard:** 3 ACTIVATED by sibling lanes (5 g61, 9 e19, 10 d122-rows),
1 ACTIVATED-BY-g44 (4's IslandHouse half), 1 WIRED-HERE (4's GERunner half,
`fd0edd6b6`), 1 flavour-corrected non-consumer (1), 6 STILL-DEFERRED
(2, 3, 6, 7, 8, 11, 12 — 2 and 3 share the one
`AttemptToConvertNPCToEvil` gap). The d122 `from_world` one-line feed graft
is the only NEW merge step this census adds; everything else composes as
the sibling lanes' own seals declare.

---

## 9. CLOUDS-FAMILY declaration (the merger bible, one paragraph)

**KEEP g69 wholesale for the cloud + star-static family:** `cloud.rs` (the
200-slot pool, `addCloud`/`RollRareCloud` incl. the type-36 crimson gate, the
`updateCloudLayer`/`update_num_clouds` cadence, `UpdateCloudParallax`, the
`CloudEnv::gather` feeds), the three `DrawClouds_*` passes + `draw_cloud_color`
+ `cloud_bg_draw_step` (the per-DRAW cloudBGAlpha ramp — d130's tick-side
stand-in ramp DROPS), `rain_dim_sky` (byte-truncate-first `98067533` — d130's
`(1−visual·0.9).max(0.1)` stand-in DROPS at both sites), `make_stars` +
`star_pass_heads` + `draw_star_color` (the §6 ports; g70's celestial-dim
`cloud_alpha` param dedupes to ONE with g69's gate param, same name/semantics),
the `World.crimson` field + `.wld` flags read + `.trwld` v18 tail byte (g61/e19
carry the field hunks verbatim — dedupe to one; the v18 tail offset follows
g71's SEALED 92 §1-SEAL layout authority, NOT this doc's §1 proposal; the
`.wld` flags byte dedupes onto f45's write per 92 §1-SEAL.2). **Values from
d130:** every `cloudAlpha`/`cloudBG`/`maxRaining` read goes through d130's
`Rain` accessors (`raw_alpha`→`cloud_alpha` rename at the merge). **Wind from
g68** (the single wind-flag owner). **Falling-star visual = the §6 task-seed,
unclaimed; falling-star sound = g23's dormant pair.** No two family branches
own the same fn body after the three named drops (d130 tick-ramp, d130
sky-dim stand-in, g70 star-gate stand-in) and two dedupes (cloud_alpha param,
crimson field hunks).

---

## Witnesses (WRITTEN NOT RUN — code-only)

| Witness | File | Mechanic |
| --- | --- | --- |
| `w4_g69_the_dryad_lines_follow_the_world_evil` | `npc_chat.rs` | 2 |
| `the_dryad_stock_rides_the_moon_and_the_blood_moon` (extended) | `shop.rs` | 2 |
| `from_world_carries_the_downed_flags` (extended) | `shop.rs` | 2 |
| `w4_g69_the_angler_swap_reads_the_live_world_evil` | `world_sim/npcs.rs` | 2 |
| `w4_g69_the_world_evil_splits_the_eye_loot` | `world_sim/npcs.rs` | 2 |
| `w4_g69_the_dryad_shop_open_reads_the_live_world_evil` | `world_sim/npcs.rs` | 2 |
| `evil_stone_and_armor_key_producers` (extended) | `lang.rs` | 2 |
| `w4_g69_the_drunk_dawn_flips_the_world_evil` | `world_sim/clock.rs` | 2 |
| `w4_g69_the_weather_counter_jump_replays_the_full_cadence` | `cloud.rs` | 3 |
| `w4_g69_rain_dim_sky_is_the_global_byte_truncated_formula` | `render/sky.rs` | 4 |
| `w4_g69_the_hardmode_v_reads_the_live_world_evil` | `world_sim/growth.rs` | 8 (retire) |
| `w4_g69_the_star_gate_reads_the_raw_cloud_alpha` | `render/sky.rs` | 6 (retire) |
| `w4_g69_draw_star_color_replays_the_byte_chain` | `render/sky.rs` | 6 (retire) |
| `initialize_hard_mode_runs_both_runners` (re-pinned onto `world.crimson`) | `spread.rs` | 8 (retire) |
| `test_dw_1_1_hardmode_init_draws_the_update_worldgen_stream` (re-pinned) | `world_sim/growth.rs` | 8 (retire) |

## Commits (this sweep)

- `5af3c78d` — mechanic 2: the 5 crimson feeds + drunk dawn flip + 7 witnesses.
- `bf7151b5` — mechanic 3: spawn-cadence verify + init-deviation doc + jump witness.
- `7f1581d1` — mechanic 4: rain-cloud darkening census + `rain_dim_sky` global dim.
- `c7e5476a` / `98067533` — mechanic 4 fixups: unchecked byte wrap + byte-first truncation.
- `20d0380f` — the doc-sweep publish (mechanic 1 cross-note + mechanic 5 census).

## Commits (the RETIRE sweep)

- `fd0edd6b6` — retire 1: the hardmode-V evil flavour reads the LIVE
  `World.crimson` (consumer 4's GERunner half; 3 callers re-pointed, 2
  witnesses re-pinned + 1 new).
- `99a67f9ee` — retire 2 (g69d-star-visual): `star_pass_heads` +
  `draw_star_color` + the skyblock bgTopY zero; 2 new witnesses.
- `16e0a9dff` — retire 2 fixup: the brightness witness re-derived on the f64
  1.4 (`(int)(155·1.4)` = 216, not 217 — the double representation truncates
  down; the graveyard case follows).
- (this seal) — §6 census/ports/task-seed + §7 compose confirmations + §8
  activation census + §9 clouds-family declaration; INDEX/ANALYSIS/PORT_MAP/
  anchors updated. **SEALED — no further g69 commits planned.**
