# 15 — Dust / Gore / UIDust

> Wave 2 · researched 2026-11-12 · sources: decompiled + crates + ROADMAP/ANALYSIS/PORT_MAP

## Scope

Two C# files carry the whole particle layer: `decompiled/Terraria/Dust.cs`
(2749 lines, `public class Dust` — a **reference type**, not a struct, so
`Main.dust` is an array of shared objects) and `decompiled/Terraria/Gore.cs`
(1477 lines, `public class Gore`). `Dust` owns the spawn API (`NewDust` family),
the per-tick simulation (`UpdateDust`, Dust.cs:420-2423) and the colour maths
(`GetAlpha`/`GetColor`/`GetVisualRotation`/`GetVisualScale`, Dust.cs:2425-2793).
`Gore` owns the blood/chip/tree-leaf particles (`Update`, Gore.cs:308-1272, incl.
the leaf arm `Gore_UpdateLeaf` Gore.cs:1085-1209). Both are driven from
`Main.Update`/`Main.DrawDust` (Main.cs:~38830-39013) and both gate on
`Main.maxDustToDraw` / the 600-slot gore array. Their consumers are everywhere:
3279 `Dust.NewDust*`-family call sites in 36 C# files and 1643 `Gore.NewGore*`
sites in 13 files.

## Key entities

C# (`decompiled/Terraria/`):

- `Dust.cs::Dust` (Dust.cs:10-52) — the particle record: `dCount` (static f32,
  pool-pressure), `lavaBubbles`, `SandStormCount`, `dustIndex`, `position`,
  `velocity`, `fadeIn`, `noGravity`, `scale`, `rotation`, `noLight`,
  `noLightEmittance`, `fullBright`, `active`, `type`, `color`, `alpha`, `frame`
  (Rectangle), `shader`, `customData` (Player/Projectile/NPC follow), `firstFrame`.
- `Dust.cs::NewDust` (75-256) — the only allocator. Guards (77-101):
  `gameMenu`/`gamePaused`/`netMode==2`/`isGeneratingOrLoadingWorld` → return
  `6000` (the "no dust" sentinel); an off-screen cull with a
  `400*(1-dCount)` px margin (`Intersects` a 10×10 box). Pool scan (103-141):
  first inactive slot in `0..6000`; `Main.NoPooling` reallocation; **pressure
  bands** — `i > 0.9*maxDustToDraw` → `rand.Next(4)!=0` refuse, `0.8` →
  `Next(3)!=0`, `0.7` → `Next(2)==0` refuse, `0.6` → `Next(4)!=0`, `0.5` →
  `Next(5)!=0`, else `dCount = 0`.
- `Dust.cs::NewDustDefault block` (153-213) — `fadeIn=0`, `active=true`,
  `noGravity=false`, `color=newColor`, `alpha=Alpha`, position jittered inside
  the (min-5) W×H box, `velocity = rand.Next(-20,21)*0.1 + Speed`, `frame.X =
  10*Type`, `frame.Y = 10*rand.Next(3)`, per-100 frame shift (`X -= 1000`,
  `Y += 30`), 8×8 frame, `rotation=0`, `scale = 1 + rand.Next(-20,21)*0.01`,
  then `*= Scale`, `firstFrame=true`, `ChildSafety.DangerousDust` → type 16.
  Per-type tweaks 214-252: velocity-damp family, 308, alpha-170 family, 41
  zero, 80 alpha 50, 34/35/152 (`0.1x`, `Y=-0.5`, the 34 `WetCollision` kill).
- `Dust.cs::NewDustPerfect` (54-63), `NewDustDirect` (65-73), `CloneDust`
  (258-286), `QuickDust` (288/293/298/367), `QuickDustSmall` (379),
  `QuickDustLine` (391), `QuickBox` (311), `QuickCircle` (335),
  `dustWater` (402). **There is no `NewDustMulti` in 1.4.5.8** — the family is
  exactly the eight names above plus `UpdateDust`.
- `Dust.cs::UpdateDust` (420-2423) — the order: `netMode==2` return → counters
  reset → loop `i<6000`: `i >= maxDustToDraw` ⇒ **`active=false`** (dust past the
  draw budget is killed, not skipped — 2393-2395) → `dCount++` → `scale>10` kill
  → ChildSafety re-type → per-type scale/velocity arms (299-301/305, 258, 309,
  325, 86-92/286, 262, 240) → `position += velocity` (477) → `Lighting.AddLight`
  arms → `customData` follow (position delta of the owner) → collision arms
  (`SolidCollision` ⇒ `velocity *= 0.25` when `fadeIn == 0`) → the big if-chain
  per type → `!noGravity` tail (2335-2342: `velocity *= 0.92`, `scale -= 0.04`
  while `fadeIn==0`) → screen-bottom kill (2343-2346) → `dCount` scale nibbles
  (2347-2387: 0.001/0.0025/0.005/0.01/0.02 at dCount 0.5..0.9) → `scale < num86`
  (0.1..0.25) kill → `dCount` recomputed from the live count (2398-2422).
- `Dust.cs::GetAlpha` (2425-2728) / `GetColor` (2729-2773) — the ~300-arm type
  switch (default at 2456) mapping type → alpha, tint and light overlays.
- `Gore.cs::Gore` (16-95) — `goreTime` (static, `600`, set at Gore.cs:1509),
  `position`, `velocity`, `rotation`, `scale`, `alpha`, `type`, `light`,
  `active`, `sticky = true`, `timeLeft = goreTime`, `behindTiles`,
  `frameCounter`, `Frame` (SpriteFrame), `Width`/`Height`, `AABBRectangle`,
  `frame`, `numFrames`.
- `Gore.cs::NewGore` (1286-1462) — `netMode==2`/`gamePaused`/`isGenerating`/
  `Type==-1` → `600`; first inactive slot in `0..600`, **no pressure bands — the
  pool simply drops at saturation** (1317-1320). Init 1321-1340 (velocity jitter
  `Y -= rand.Next(10,31)*0.1`, `X += rand.Next(-20,21)*0.1`), `ChildSafety`
  replacement (1334-1340), sticky rules (1341-1354), `GoreID.Sets.IsDrip`
  (1355-1360), 16/17, 570-572, the `SpriteFrame` rows for 1201/1208/1217/1218/
  1225, the `SpecialAI` arms 3/1/2 (1390-1435), alpha-254 tail (1443-1455),
  `DeactivateIfOutsideOfWorld` (1457).
- `Gore.cs::NewGorePerfect` (1273-1279), `NewGoreDirect` (1281-1284),
  `GetAlpha` (1464-1477).
- `Gore.cs::Update` (308-1272) — sticky/rest arms, `timeLeft -=
  GoreID.Sets.DisappearSpeed[type]` then `active=false` (112/158/205/250/1011/
  1149/1203), `Collision.TileCollision`/`SlopeCollision`/`WetCollision`, the
  leaf arm `Gore_UpdateLeaf` (1085-1209: falling phase with the sway/wind, rest
  phase with the wind drift, the 600-tick fade + 15-tick alpha lerp).
- `Main.cs` — the pool state: `maxDust = 6000` (Main.cs:1023), `dust = new
  Dust[6001]` (Main.cs:66985, slot 6000 = the sentinel),
  `maxDustToDraw = (int)(6000*(gfxQuality*0.7+0.3)) * gfxQuality`, clamped
  `≥ 1000` (Main.cs:17066-17073, reset to 6000 at 66794); the draw loop
  `for (i < maxDustToDraw)` (Main.cs:38836) with the type-213 draw-scale-1 arm
  (Main.cs:38974-38977).

Rust (`crates/`):

- No `Dust`/`Gore` module, no `DustID` table, no `new_dust`, no dust/gore pool.
  The only shared asset is the sheet: `assets.rs::Atlas::dust` (1374/1698) and
  `assets.rs::Lookups::dust_uv` (2352).
- `tr-app/src/sand.rs::Sand`/`SandP` (46-47) + `emit_dust` (82+) + `tick`
  (255-371) — the sandstorm's dust-268 stand-in.
- `tr-app/src/minecart.rs::SparkP` (974-985), `Sparks` (990-1113),
  `MAX_SPARKS = 500` (1116) — the dust-213 wheel-spark stand-in.
- `tr-app/src/render/foliage.rs::LeafGore` (25-43), `from_spawn`/`new` (47-67),
  `tick_leaves` (76-150), `draw_leaves` (162+) — the tree-leaf gore stand-in.
- `tr-app/src/app/mod.rs::App::leaves` (625-626) +
  `tr-app/src/app/frame.rs` (1151-1179) — the App-owned leaf vector, capped at
  **400** (frame.rs:1159).
- `tr-app/src/rain.rs` — `Snowflake` (98) and `Sparkle` (91): the dust-76 /
  dust-43 rain emitters, `MAX_SNOW = 600` (48).

## Dependencies

- Calls into: `Lighting.AddLight` (dust is a light source for ~60 types),
  `Collision.SolidCollision`/`WetCollision`/`TileCollision`/`SlopeCollision`,
  `Framing.GetTileSafely`/`WorldGen.SolidTile`, `Main.rand` (UnifiedRandom — the
  shared global stream), `ChildSafety.DangerousDust`/`DangerousGore`,
  `TextureAssets.Gore[]`, `GoreID.Sets.{SpecialAI,IsDrip,DisappearSpeed}`,
  `ArmorShaderData`, `SpriteFrame`, `Player`/`Projectile`/`NPC` (the `customData`
  follow).
- Called by: everything — `Projectile.cs` (1195 sites), `NPC.cs` (927),
  `Player.cs` (215), `ParticleOrchestrator.cs` (61), `TileDrawing.cs` (53),
  `WorldGen.cs` (29), `Mount.cs` (25), `DelegateMethods.cs` (18), `Main.cs`
  (23), plus 27 more files.
- Shared data: `Main.dust[6001]`, `Main.gore[600]`, `Main.maxDustToDraw`,
  `Main.gfxQuality`, `Dust.dCount`, `Gore.goreTime`, `Main.snowDust`,
  `Main.NoPooling`.

## Port status

| C# subsystem | Status | Rust location | Evidence / notes |
| --- | --- | --- | --- |
| `Dust` record (18 fields) | MISSING | — | No `Dust` type; three unrelated per-consumer structs (`SandP`, `SparkP`, `LeafGore`) each carry a different field subset |
| `Dust.NewDust` / `NewDustPerfect` / `NewDustDirect` / `CloneDust` / `QuickDust*` | MISSING | — | No spawn API anywhere in `crates/`; only `Lookups::dust_uv` (assets.rs:2352) reads the sheet |
| `Main.dust[6001]` + `maxDust`/`maxDustToDraw` + pressure bands | MISSING | minecart.rs:1116 `MAX_SPARKS`, frame.rs:1159 leaf cap 400 | Stand-in caps are *not* the C# model: `Sparks::spawn` (1002-1008) drops overflow at 500 and says so in a comment ("pinned to no pressure"); no `gfxQuality`/`dCount` |
| `UpdateDust` main loop order | PARTIAL | minecart.rs:1010-1072, sand.rs:255-371, foliage.rs:76-150 | Three type-specific re-implementations of the tail; none has the generic loop, the per-type if-chain or the `dCount` nibbles |
| Per-type dust table (~300 arms, `GetAlpha`/`GetColor`) | MISSING | minecart.rs:1099-1103 (the 213 `GetAlpha` arm only) | `Dust.cs:2425-2773` has no counterpart; even the ported 213/268 arms hard-code their arms |
| Dust → `Lighting.AddLight` | MISSING | — | minecart.rs:1025-1027 states "the port writes no dust light" |
| `Main.DrawDust` generic loop + 213 arm | PARTIAL | minecart.rs:1079-1107, sand.rs:344+, foliage.rs:162+ | Three private draw routines; no `maxDustToDraw`-scaled loop |
| `Gore` record + `NewGore`/`NewGorePerfect`/`NewGoreDirect` | MISSING | — | No gore allocator; `Gore_<id>` sheets exist (assets.rs:240-243, 1389-1392) but only for 910/911 |
| `Main.gore[600]` pool + saturation drop | MISSING | frame.rs:1159 cap 400 (leaves only) | No gore pool; nothing else can exist |
| `Gore.Update` (sticky / `timeLeft` / fade) | PARTIAL | foliage.rs:76-150 | Only the leaf arm `Gore_UpdateLeaf`; sticky/`DisappearSpeed`/alpha-fade of every other gore absent |
| `GoreID.Sets.{SpecialAI,IsDrip,DisappearSpeed}` | MISSING | — | Not generated into `tiledata` or elsewhere |
| `ChildSafety.DangerousDust`/`DangerousGore` gate | MISSING | settings.rs:79-81 (`Settings::blood`) | The toggle exists and round-trips, but **no sim or render code reads it** (grep: only `menu/pages.rs:364`, `ui/ingame_options.rs:769/798` write/read it) |
| Ambient tree leaves (Gore 910/911) | PORTED | foliage.rs + frame.rs:1151-1179 | Full `Gore_UpdateLeaf` falling+rest phases minus the honey/lava rest arms; App-owned, capped 400 |
| Sandstorm dust (268) | PORTED | sand.rs | `Sandstorm.EmitDust` (`Main.cs:17247`) + the 268 AI |
| Minecart wheel sparks (213) | PORTED | minecart.rs | `DelegateMethods.cs:223-242` + the 213 `UpdateDust`/`DrawDust` arms |

### Caller-coverage table (the centrepiece)

`Dust.NewDust*`-family call sites: **3279** in **36** files
(`NewDust` 2648 + `NewDustPerfect` 207 + `NewDustDirect` 349 + `QuickDust` 18 +
`CloneDust` 51 + `QuickBox` 5 + `QuickDustLine` 1).
`Gore.NewGore*`: **1643** in 13 files (`NewGore` 1627 + `NewGorePerfect` 16).

| C# caller file | `Dust.NewDust*` | `Gore.NewGore*` | Rust counterpart | Verdict |
| --- | --- | --- | --- | --- |
| `Projectile.cs` | 1195 | 165 | none — `projectile.rs:5512` "the dust/gore spawns are SKIPPED" | MISSING |
| `NPC.cs` | 927 | 1345 | none — `npc.rs:9349`/`:10008` list the dust/gore bursts as skipped | MISSING |
| `Player.cs` | 215 | 32 | none (dash/wing/jump trails = ROADMAP "Out") | MISSING |
| `GameContent/Drawing/ParticleOrchestrator.cs` | 61 | 12 | none — no particle renderer exists | MISSING |
| `GameContent/Drawing/TileDrawing.cs` | 53 | 20 | leaf emit only (`foliage.rs`, 910/911) | PARTIAL |
| `WorldGen.cs` | 29 | 36 | none — but `NewDust` returns 6000 while `isGeneratingOrLoadingWorld` (Dust.cs:89-92), so C# emits nothing here at runtime either | N/A (latent) |
| `Mount.cs` | 25 | 3 | minecart sparks only (213) | PARTIAL |
| `DataStructures/PlayerDrawSet.cs` | 23 | 3 | none | MISSING |
| `Main.cs` | 23 | 0 | `sand.rs` covers `Sandstorm.EmitDust` (Main.cs:17247) only | PARTIAL |
| `DelegateMethods.cs` | 18 | 3 | minecart sparks (223-242) | PARTIAL |
| `WorldItem.cs` | 17 | 1 | none | MISSING |
| `GameContent/Golf/GolfHelper.cs` | 8 | 0 | none | MISSING |
| `Dust.cs` (self, QuickDust helpers) | 8 | 4 | none | MISSING |
| `Graphics/Renderers/FakeFishParticle.cs` | 4 | 0 | none | MISSING |
| `Graphics/Renderers/StormLightningParticle.cs` | 4 | 0 | none | MISSING |
| `Gore.cs` (self) | 4 | 4 | none | MISSING |
| `GameContent/PotionOfReturnGateHelper.cs` | 4 | 0 | none | MISSING |
| `DataStructures/PlayerDrawLayers.cs` | 3 | 0 | none | MISSING |
| `Collision.cs` | 2 | 0 | none | MISSING |
| `GameContent/Liquid/LiquidRenderer.cs` | 2 | 0 | none | MISSING |
| `UIWorldCreationAdvancedSecretSeedsList.cs` | 2 | 0 | none | MISSING |
| `PopupText.cs` | 2 | 0 | none | MISSING |
| `GameContent/VoidLensHelper.cs` | 2 | 0 | none | MISSING |
| `LeashedEntities/LeashedCritter.cs` | 2 | 0 | none | MISSING |
| `GameContent/Events/Sandstorm.cs` | 1 | 0 | `sand.rs::emit_dust` | PORTED |
| 15 single-site files (`WaterfallManager`, `Rain.cs`, `Chest.cs`, `DD2Film`, `Utils`, `MessageBuffer`, `FinalFractalHelper`, `Fairy`/`EmpressButterfly`/`HellButterfly` leashed critters, `BloodyExplosionParticle`, `SpelunkerProjectileHelper`, `TeleportPylonsSystem`, `UIWorldCreation`, `FakeFish`) | 15 | 0 | `rain.rs` covers the rain/snow + waterfall sparkle emitters (dust 76/43) | PARTIAL |

Net Rust coverage: **3 of 36 dust-calling files** (8 %) and **1 of 13
gore-calling files** (TileDrawing's leaves). The three ported arms (213, 268,
leaf) are exactly the ones whose *visibility* the screenshot tests noticed.

## Tests

- `crates/tr-app/src/sand.rs` — 7 tests (the `sand_world` helper + the
  emit gates: visuals off, dead calm, `<100` sand tiles, the orphan-fade, the
  scale hold) — the dust-268 AI.
- `crates/tr-app/src/minecart.rs` — 32 tests in the file, incl.
  `world_sim/player.rs:1094` `assert_eq!(app.minecart_sparks.count(), 3, "the
  pool holds the sparks")` — the dust-213 pool.
- `crates/tr-app/src/rain.rs` — 24 tests; the snow/sparkle pools are pinned by
  the `set_raining_for_test` seam (:287).
- `crates/tr-app/src/render/foliage.rs` — **0 tests**; the leaf gore is covered
  only indirectly by the TILE-DRAW-3 SHA256 screenshot determinism check.
- `crates/tr-core/src/edit.rs::test_DW_2_3_no_gore_items_spawn` (edit.rs:6125,
  with the rationale at :2053-2054)
  — asserts the Gore-shard block (Main.cs:56878-56953) spawns no items, i.e.
  pins the *absence* of the gore mechanism.
- `crates/tr-app/src/settings.rs` — the `GoreVisualsAllowed`/`BloodAndGore`
  round-trip tests (891/992/1554/1598/1618/1623): the toggle persists but is
  never consumed by a sim/render path.

## Deviations & risks

1. **No general dust/gore layer at all** — 3161 of 3279 dust sites and 1642 of
   1643 gore sites have no port. Severity: **behavioural + pixel** (every hit,
   step, spell, death, liquid splash loses its particles). ROADMAP/ANALYSIS admit
   it per-feature ("the dust/gore spawns are SKIPPED", `npc.rs:9349`,
   `projectile.rs:5512`) but never as a module gap. Not in PORT_MAP as a row.
2. **Three unrelated pools instead of one** — `Sparks` (500), `Sand::parts`
   (budget `1000*screenW/1920`), `App::leaves` (400). Severity: **behavioural**
   for spawn-rate fidelity, **cosmetic** for the cap values themselves.
3. **Pool pressure model absent** — C# `dCount` bands (Dust.cs:97-152, 2347-2422)
   both gate spawns *and* shrink particles; the Rust comment at
   `minecart.rs:999-1001` says "pinned to no pressure". Since the port has no
   global pool, `dCount` can never be reproduced. Severity: **behavioural**.
4. **Per-consumer RNG streams** — each stand-in carries its own `Main.rand`
   stand-in, so the *global* RNG order diverges from C# even where a consumer is
   "ported". Severity: **determinism** (port-internal determinism is kept and
   tested; cross-consumer order is not).
5. **Dust light absent** — ~60 dust types feed `Lighting.AddLight`; the ported
   213 arm documents "the port writes no dust light" (minecart.rs:1025-1027).
   Severity: **pixel** (lighting/glows).
6. **`maxDustToDraw` / `gfxQuality` unmodelled** — the C# quality-scaled budget
   (Main.cs:17066-17073) is pinned to 1.0/0 (admitted in ROADMAP M1.4-WEATHER-2
   "Out"). Severity: **behavioural** at non-max quality; **cosmetic** for the
   shipped pins.
7. **Leaf gore deviation inside a DONE row** — `foliage.rs:74-75` documents that
   the honey-freeze and lava-burn-up rest arms are not ported; leaves rest on
   liquid surfaces. Severity: **pixel**.
8. **`ChildSafety` gate inert** — `Settings::blood` is parsed, saved and toggled
   but never read by any dust/gore code (there is none); BUG_REPORT's truth pass
   already records "`blood` is the `ChildSafety` gate — nothing to gate yet".
   Severity: **cosmetic** today, **behavioural** the moment a gore/dust layer
   lands (the gate must be wired at spawn, not at draw).
9. **Gore shards deferred with no mechanism** — `edit.rs:2053-2054` (the pot
   shards, Main.cs:56878-56953) and ROADMAP M1.4-TO-4's "the pot gore shards
   deferred (no gore mechanism)". Severity: **cosmetic** (no item is lost; the
   test asserts the block spawns nothing).

### ROADMAP DONE-row re-verification

| Row | Claim | Verdict |
| --- | --- | --- |
| M1.4-WEATHER-2 sandstorm (DONE 2026-11-05) | `Sandstorm.EmitDust` + the dust-268 AI in `sand.rs`, with `Out: gfxQuality/dCount pinned 1.0/0` | **PASS** for the claimed slice; the row is honest that the shared pool/pressure is out. Re-read of `Dust.cs:420-2423` confirms no other dust arm is claimed. |
| M2-PHYS-7 minecart (DONE 2026-11-06) | "the App-owned capped-500 spark pool ticked/drawn in `frame.rs`" | **PASS** — matches `minecart.rs:990-1116` + `frame.rs:1177`. Deviation: 500 vs the 6000-slot C# pool (risk 2/3). |
| M4-PROJ-6b leaf blower (DONE 2026-11-11) | "the dust calls SKIPPED (no dust layer in the pool)" | **PASS** — verified: `projectile.rs:5512` and no dust API exists. |
| M4-PROJ-3b flails / M4-PROJ-7 whips / M4-PROJ-8 minions | list "dust/sound/impact" as decor follow-ups | **PASS** as honest scoping; the module-level gap is risk 1. |
| M2-PHYS-4 dash / M2-PHYS-5 wings / M2-PHYS-8 jumps | "the dust/gore trails (visual seams)" out | **PASS** as honest scoping. |
| M1.4-TO-4 herbs/pots (DONE 2026-11-05) | "the pot gore shards deferred (no gore mechanism — ANALYSIS)" | **PARTIAL** — feature DONE, the gore arm knowingly absent; acceptable only because nothing else ports gore. |
| TILE-DRAW-3 wind + leaves (DONE, determinism verified) | leaf-gore behaviour | **PASS** — leaf arm is the most complete gore port; deviations in ANALYSIS (risk 7). |

## Task seeds

- **S** — Add a `Dust`/`Gore` gap row to `PORT_MAP.md` + a `docs/ANALYSIS.md`
  section "Dust & gore (module 15)" listing the 3 stand-ins, the 4 pool caps and
  the `dCount`/`maxDustToDraw` exclusions. No code. (Prerequisite for the rest.)
- **M** — Port the `Dust` record + `NewDust` family as one `tr-core::particles`
  module (fields verbatim, the guard chain 77-101, the pressure bands 103-141,
  the default block 153-213) behind a `DustPool` with `spawn/next_index`, and
  make `minecart::Sparks` and `sand::Sand` clients of it. Keeps the existing
  per-consumer RNG streams, so no determinism change. Depends on: the S task.
- **M** — Port `UpdateDust`'s generic tail (position integration, the
  `!noGravity` arm, the screen-bottom kill, the `dCount` nibbles, the scale-floor
  kill) into the shared pool and delete the three duplicated tails. Depends on
  the pool.
- **L** — Port the per-type table: `DustID` constants, the `GetAlpha`/`GetColor`
  switches, `GetVisualRotation/Scale`, and the `UpdateDust` if-chain arms by
  family (light emitters, followers, collisions). Depends on the pool + the
  generic tail; needs a screenshot A/B per family.
- **L** — Port `Gore` + `NewGore` + `Gore.Update` + `GoreID.Sets` (600-slot
  pool, `SpecialAI`/`IsDrip`/`DisappearSpeed`, `Gore_UpdateLeaf` factored out of
  `foliage.rs`). Depends on the dust pool (shared `Main.rand` and light).
- **M** — Wire `ChildSafety`/`Settings::blood` at the spawn site of both pools
  (not at draw), and add tests that the gate blocks blood/gore types. Must land
  with or before the Gore port.
- **S** — Add unit tests for `foliage.rs` (the leaf falling/rest transition, the
  600-tick fade) — the only ported arm with 0 tests.
- **M** — Re-point the 3 stand-in consumers (NPC/Projectile death bursts, player
  step/dash dust, TileDrawing hit dust) at the shared pool once it exists, one
  file at a time, each with a screenshot pair. Depends on the per-type table.
