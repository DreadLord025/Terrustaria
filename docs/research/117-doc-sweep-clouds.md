# 117 — DOC-SWEEP: clouds / sky (lane 33, W4-g69)

**Branch:** `task/W4-g69-weather-clouds-parallax` (continues the DONE `9717c0bb`
cloud-parallax work). **Mode:** code-only — no cargo/tests/graphify invoked;
witnesses WRITTEN NOT RUN. **Scope:** the five clouds/sky follow-ups the Lead
flagged after the g69 cloud lane landed. Companion lanes: `task/W4-g70-bg-layers-1`
(forest sets, tip `b86c8d69`), `task/W4-g71-bg-layers-2` (surface variants, tip
`2b57b362`; its DOC-SWEEP doc `92-doc-sweep-bg.md` owns the v18 layout contract),
`task/W4-g72-bg-layers-3` (cavern/hell styles).

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

## Commits (this sweep)

- `5af3c78d` — mechanic 2: the 5 crimson feeds + drunk dawn flip + 7 witnesses.
- `bf7151b5` — mechanic 3: spawn-cadence verify + init-deviation doc + jump witness.
- `7f1581d1` — mechanic 4: rain-cloud darkening census + `rain_dim_sky` global dim.
- `c7e5476a` / `98067533` — mechanic 4 fixups: unchecked byte wrap + byte-first truncation.
- (this doc) — mechanic 1 cross-note + mechanic 5 census + INDEX/ANALYSIS/PORT_MAP.
