# 90 — Consolidated findings (module research → ROADMAP raw material)

> Consolidated 2026-11-12 · sources: every completed file in `docs/research/` except
> `INDEX.md`, `MODULE-TEMPLATE.md` and this file · read-only extraction, no new research.
>
> **Input set:** **39 module files** — 01, 02, 03, 04, 05, 06, 07, 08, 09, 10, 11, 12, 13,
> 14, 15, 16, 17, 18, 19, 20, 22, 24, 25, **26, 27**, 28, 29, **30**, 31, 32, **36, 37, 38,
> 39**, 40, 41, 42. Modules **21** (recipes/crafting), **23** (wiring/tile-entities) and
> **33** (localization) have **no research file** yet (the new `docs/ROADMAP.md` links to
> them anyway — see §4.3, U-60). `INDEX.md`, `MODULE-TEMPLATE.md` and this file are excluded.
>
> **Doc reorganisation (2026-11-12, after the research):** the old `docs/ROADMAP.md` is now
> **`docs/ATLAS.md`** (the per-subsystem atlas + the legacy task rows, kept verbatim). A
> **new `docs/ROADMAP.md`** was written from this audit (the scoreboard, the Stage-1/Stage-2
> task catalog and a "Documentation corrections" table). Every `ROADMAP.md:NN` citation in
> the research files therefore resolves against **`ATLAS.md:NN+7`** from line 64 onward.
> §4 says where each claim now lives and whether the new ROADMAP already covers it.
>
> **Conventions used below**
> * Task-seed ids are renumbered `R-<module#><n>` (`R-09-1` = module 09, seed 1) and are
>   stable: they never change once published.
> * Size: `S` / `M` / `L` as written in the source file.
> * Severity: taken from the source file's own severity label for the deviation the seed
>   fixes; where the seed has no stated severity, the closest matching deviation's
>   severity is used; doc-only seeds are marked `doc`.
> * Sort order inside every theme: **size descending (L → M → S), then severity
>   descending** (critical → high → medium → low → cosmetic → doc).
> * "Unadmitted" means the research file records the gap and states that
>   `docs/ROADMAP.md` / `docs/ANALYSIS.md` / `docs/PORT_MAP.md` do **not** admit it
>   (or assert the opposite). "Admitted" means one of those three docs already records it.
> * Citations are reproduced **exactly** as written in the source research files.

---

## 1. Module scoreboard

Overall verdict is derived from the file's own status table (the `Status` column of
"Port status" / "Coverage table" / "Port status" blocks), not guessed.

| # | Topic | Overall verdict | Headline gap | Deviations | Seeds | File size (lines / bytes) |
| --- | --- | --- | --- | --- | --- | --- |
| 01 | Main update loop / time / camera / input | **PARTIAL** | No pause semantics — the world keeps simulating with the inventory open | 8 | 7 | 148 / 14 643 |
| 02 | Main draw pipeline / scene layers / effects | **PARTIAL** | No off-screen layer-target family; the full-screen effect set (`ScreenDarkness`, lens flare) is absent | 8 | 6 | 128 / 16 586 |
| 03 | Player core (movement, life/mana, death) | **MOSTLY PORTED** | In-liquid statics are the wrong formula port-wide (2× jump-height error in liquids) | 12 | 12 | 226 / 23 111 |
| 04 | Player combat (hurt/damage, item use, buffs) | **PARTIAL** | The projectile→NPC hit skips `DamageVar` and the defence step entirely | 10 | 7 | 209 / 20 102 |
| 05 | NPC core (defaults, AI dispatch, net sync) | **PARTIAL** | `aiStyle` dispatch coverage is by-kind, not by-style — 74 of 128 styles unreferenced | 12 | 10 | 265 / 24 451 |
| 06 | Projectile core (defaults, AI dispatch, damage) | **PARTIAL** | `AI()` dispatch covers 32 of 210 aiStyles; the rest silently no-op | 11 | 8 | 132 / 15 415 |
| 07 | Item core (defaults, prefixes, tooltips) | **PARTIAL** | The pickup path adds the transfer twice — every drop is doubled | 8 | 7 | 300 / 25 489 |
| 08 | Tile data & framing | **MOSTLY PORTED** | `frameNumber` is a position hash, not the stored/RNG value — reframing is byte-different from vanilla | 8 | 8 | 161 / 14 112 |
| 09 | TileDrawing / DrawData / paint+colour | **PARTIAL** | `GetTileDrawData` covers ≈113 of ≈376 case ids; the tile animator is absent | 12 | 10 | 239 / 22 981 |
| 10 | Lighting | **MOSTLY PORTED** | Entity `AddLight` sources are missing — glowing entities/explosions are dark at night | 10 | 8 | 162 / 16 109 |
| 11 | Liquid / LiquidBuffer / modules | **PARTIAL** | Liquid type reactions absent — lava+honey never become obsidian / honey block / shimmer | 12 | 8 | 190 / 16 243 |
| 12 | WorldGen / WorldBuilding passes | **PARTIAL** | 52 of 106 classic passes have no executed step | 15 | 8 | 348 / 31 192 |
| 13 | Collision / HitTile / scans | **PARTIAL** | `HitTiles`, `StickyTiles`, `AnyHurtingTiles`, `SwitchTiles` are all absent | 10 | 7 | 151 / 15 636 |
| 14 | Map / MapHelper / minimap layers | **MOSTLY PORTED** | No `MapUpdateQueue` — tile changes outside the reveal window are silently lost | 22 | 9 | 325 / 30 178 |
| 15 | Dust / Gore / UIDust | **MOSTLY MISSING** | No general dust/gore layer — 3161 of 3279 dust sites and 1642 of 1643 gore sites have no port | 9 | 8 | 258 / 20 458 |
| 16 | Weather | **MOSTLY PORTED** | `any_player_120_life` has no writer — the 0.35 wind cap is dead code | 14 | 10 | 299 / 27 227 |
| 17 | Netcode (NetMessage / sockets / Netplay) | **MOSTLY MISSING** | Only 25 of 163 message ids are ported; the wire is not vanilla-compatible | 18 | 9 | 290 / 24 913 |
| 18 | UI framework (UIElement / UIState / layers) | **PARTIAL** | No child→parent event bubble; no `OverflowHidden` scissor | 10 | 9 | 262 / 23 245 |
| 19 | Front-end screens (char/world create+select) | **MOSTLY PORTED** | `IsValid` + the creative/journey rejection menu are missing on Play | 22 | 11 | 457 / 37 681 |
| 20 | In-game UI (options/inventory/chest/craft) | **PARTIAL** | Right-click takes half the stack instead of one item, and never repeats | 14 | 9 | 221 / 18 082 |
| 22 | Chests / loot / shops / shopping | **PARTIAL** | The buy-price formula is incomplete — every `buyOnce` row costs 5× the vanilla | 10 | 8 | 241 / 17 999 |
| 24 | Spawner, invasions & world events | **PARTIAL** | No `invaders` spawn-rate override — ~20× the vanilla invasion density | 14 | 10 | 283 / 27 958 |
| 25 | NPC AI families (aiStyle dispatch) | **MOSTLY MISSING** | 69 of the 128 `aiStyle` arms have no Rust kind; aiStyle 3 and 7 are archetype approximations | 11 | 10 | 385 / 29 792 |
| 26 | Projectile AI families (aiStyle dispatch) | **MOSTLY MISSING** | 21 spawnable styles have a `proj_data` row but no dispatch arm — those projectiles hold position forever | 12 | 9 | 431 / 32 166 |
| 27 | Mounts / pets / minecart / grapple | **MOSTLY PORTED** | Three C# activation rules are missing (the Cart/`CanUseHooks` grapple exemption, the `m == 56` wet rule, the Fishron halving) | 17 | 13 | 319 / 26 232 |
| 28 | Buffs & debuffs (ids, rules, effects, UI, sync) | **MOSTLY PORTED** | The buff HUD is entirely missing — no icon, alpha, timer text or removal | 9 | 9 | 258 / 24 252 |
| 29 | Town NPCs: housing, happiness, pylons | **PARTIAL** | `WorldGen.ScoreRoom` is not ported — a town NPC can be homed on a spot vanilla rejects | 13 | 10 | 308 / 27 339 |
| 30 | Achievements / bestiary | **PARTIAL** | 87 of the 137 achievement rows carry a condition kind with no notify site; `achievements::save` has no call site | 18 | 12 | 265 / 24 264 |
| 31 | Golf / ball physics | **MOSTLY MISSING** | The entire golf subsystem is absent from the Rust port | 8 | 11 | 208 / 18 532 |
| 32 | Sound / audio engine | **PARTIAL** | No pitch, no per-style volume, no variation families, no instance pool | 18 | 9 | 249 / 27 499 |
| 34 | ID sets completeness (`Terraria.ID.*`) | **MOSTLY MISSING** | The `NpcKind` protocol id space collides with itself on 7 ids (78..84) | 13 | 12 | 272 / 29 542 |
| 35 | RNG & data-structure utils | **PARTIAL** | `UnifiedRandom` itself is not ported; the `Main.rand` call order is not modelled | 13 | 10 | 246 / 27 037 |
| 36 | Cinematics / film / Old One's Army | **PARTIAL** | The OOA's loss path, victory scene, medals, crystal drops and tier persistence are missing (the film framework is dead code in vanilla) | 13 | 15 | 211 / 21 047 |
| 37 | ReLogic libs (content pipeline, XNA, JSON) | **MOSTLY PORTED** | Text wrapping is not ported; the `.fx` effects are only name-parsed | 9 | 8 | 291 / 27 833 |
| 38 | Out-of-scope confirmation (Social/Steam/Testing) | **PARTIAL** | Five absences are undocumented: `Terraria/Testing/**` (34 files), `TimeLogger`, the crash handlers, the dev screens and dedicated-server mode | 13 | 9 | 255 / 22 470 |
| 39 | Rust port architecture inventory | **PARTIAL** | "The god-files are gone" is false for the entity half — 8 hand-written files exceed 5 000 lines and `App` has 257 fields (docs say 118) | 15 | 14 | 342 / 33 093 |
| 40 | DONE hot-rows re-verification | **MOSTLY PORTED** | The Hornet's hover constants are wrong (the port ships the type-205 values) | 10 | 10 | 181 / 22 959 |
| 41 | Render pipeline vs the C# graphics layer | **PARTIAL** | One blend state for the whole frame; the atlas merges textures vanilla keeps separate | 15 | 9 | 278 / 27 583 |
| 42 | Save/load (IO) + Stage-2 compression hooks | **PARTIAL** | The `.wld` tile section drops the four wire bits — data loss on a `.wld` round trip | 8 | 12 | 281 / 25 256 |

**Totals:** 39 modules reviewed · 483 deviation/risks rows · 371 task seeds.

---

## 2. Consolidated task-seed register

Every task seed from every research file, renumbered `R-<module#><n>` and grouped by
theme. `—` in *dependencies* means the source file states no dependency. Module numbers
refer to the research file (`docs/research/NN-*.md`).

### (a) determinism / RNG — 22 seeds

| id | One-line title | Module | Size | Severity | Dependencies | Source |
| --- | --- | --- | --- | --- | --- | --- |
| R-35-1 | Port `Terraria.Utilities.UnifiedRandom` (56-int `SeedArray`, `SetSeed`, `InternalSample`, `Sample`, `Next(max)`, `Next(min,max)`, `NextDouble`, `NextBytes`) | 35 | L | determinism (critical) | — | 35-rng-utils.md:268 |
| R-35-3 | `Main.SwapRandom` named-stream registry in `App` (mirrors `Main.cs:3204-3215`, the 13 stage names of `Main.cs:17749-17809`) | 35 | L | determinism (critical) | R-35-1, R-35-2 | 35-rng-utils.md:270 |
| R-35-4 | Migrate the runtime consumers (spawn, prefix, loot, chest, sandstorm, wind, item drops) off the splitmix stand-ins onto the `UnifiedRandom` streams | 35 | L | determinism (high) | R-35-1, R-35-3 | 35-rng-utils.md:271 |
| R-35-7 | Add a `UnifiedRandom`-vs-port differential harness that replays the C# draw order | 35 | L | determinism (medium) | R-35-1, R-35-2 | 35-rng-utils.md:274 |
| R-34-1 | Delete the dead `to_u16`/`from_u16` arms and fix the 78..84 id collision | 34 | S | behavioural/determinism (high) | — | 34-id-sets-audit.md:255 |
| R-34-2 | Add the exhaustive id-round-trip test over all 182 kinds | 34 | S | determinism (high) | R-34-1 | 34-id-sets-audit.md:258 |
| R-34-6 | Unify the three parallel NPC id spaces (protocol `u16`, vanilla `net_id`, partial `cs_id`) | 34 | M | structural/determinism (high) | R-34-1 | 34-id-sets-audit.md:271 |
| R-01-2 | Reorder `frame.rs:481-551` to the C# `DoUpdateInWorld_Inner` order (spawn NPCs → NPCs → gores → projectiles → items → dust → time → worldgen) | 01 | M | determinism (high) | modules 05/15 research rows | 01-main-update-loop.md:143 |
| R-26-1 | Delete the six duplicated match arms (10615/10616/10617/10618/10826/10884) and replace `_ => {}` with an assert-on-unknown-aiStyle arm | 26 | S | determinism (maintenance) | — | 26-projectile-ai-audit.md:155 |
| R-35-2 | Golden-vector test for `UnifiedRandom` (first 64 `InternalSample()` values for seeds 0/1/42/-1) | 35 | M | determinism (high) | R-35-1 | 35-rng-utils.md:269 |
| R-35-5 | Re-key the per-consumer streams to the C# seed rule and delete the arbitrary XOR constants + `craft_rng: 1` | 35 | M | determinism (medium) | R-35-3 | 35-rng-utils.md:272 |
| R-35-8 | Audit `Utils.Clamp` / `Math.Round` / `NextFloat` numerics across the Rust tree | 35 | M | behavioural/pixel (medium) | — | 35-rng-utils.md:275 |
| R-22-1 | Chest-gen determinism study: decide whether `AddBuriedChest` moves onto the worldgen stream; pin a golden fixture | 22 | L | determinism (medium) | — | 22-chest-loot-shops.md:225 |
| R-11-4 | Port `skipLiquid` (tile flag + pass gate) and switch the pass window to the C# 2500-slot stride | 11 | M | determinism (medium) | — | 11-liquid.md:192 |
| R-06-2 | Delete the duplicate match arms (aiStyles 1/3/14/99/115) and replace `_ => {}` with an assert arm | 06 | S | determinism (maintenance) | — | 06-projectile-core.md:135 |
| R-05-2 | Delete the dead `to_u16`/`from_u16` arms and correct the "the unreachable dead copies are gone" comment | 05 | S | cosmetic (id-space trap) | R-34-1 | 05-npc-core.md:250 |
| R-11-3 | Port the `genRand` 254→255 nudge (`Liquid.cs:892-895`) on the ported RNG seam | 11 | S | determinism (low-med) | R-35-1 | 11-liquid.md:190 |
| R-24-8 | Align the invasion pick's RNG with the named `SpawnNPC` stream (or document why the item stream is deliberate) | 24 | S | determinism (medium) | R-35-3 | 24-spawner-invasions.md:298 |
| R-07-7 | Decide the prefix RNG contract: document the splitmix stand-in in `ANALYSIS.md` or add an `UnifiedRandom`-compatible stream | 07 | S | determinism (medium) | R-35-1 | 07-item-core.md:324 |
| R-08-4 | Decide + document the `frameNumber` policy (decode the stored byte or keep the hash) | 08 | S | determinism/pixel (medium) | a shipped test `.wld` | 08-tile-framing.md:161 |
| R-10-4 | Exact water jitter — replace the multiplicative hash with `FastRandom::with_modifier` semantics in `blur_line_*` | 10 | S | pixel (low-med) | — | 10-lighting.md:169 |
| R-16-10 | Verify against a vanilla `.wld`/video for one full in-game day of `windCounter` walks | 16 | S | determinism (verification) | — | 16-weather.md:320 |
| R-35-6 | Fix the determinism proof: rename `deterministic_for_seed` → `generation_is_reproducible_for_a_seed` and split the two tests | 35 | S | doc (determinism claim) | — | 35-rng-utils.md:273 |

### (b) pixel / visual identity — 26 seeds

| id | One-line title | Module | Size | Severity | Dependencies | Source |
| --- | --- | --- | --- | --- | --- | --- |
| R-41-7 | Reproduce the vanilla's per-texture batching semantics on top of the atlas (group instances by source sheet) | 41 | L | pixel (medium) | R-41-2, R-41-6 | 41-render-pipeline.md:284 |
| R-41-8 | Colour-space audit of the whole sprite path (exact sRGB EOTF, paint/dye space, vanilla byte-multiply diff) | 41 | L | pixel (medium) | R-41-2 | 41-render-pipeline.md:290 |
| R-41-4 | Implement a second blend state (additive + `ReverseSubtract` multiply) and port `Main.DrawProj_LightsBane` | 41 | M | pixel/behavioural (medium) | R-41-2 | 41-render-pipeline.md:271 |
| R-41-5 | Add scissor support (`set_scissor_rect` per range) and port the two vanilla sites + the `UIList.OverflowHidden` skip | 41 | M | pixel/behavioural (medium) | — | 41-render-pipeline.md:276 |
| R-41-3 | Verify the premultiplied-alpha claim end-to-end over the decoded atlas and one DXT3 sheet | 41 | M | pixel (medium) | R-41-2 | 41-render-pipeline.md:266 |
| R-10-3 | Legacy-engine golden test for White/Retro/Trippy (`doColors_Mode*` byte output vs `LightStyle::shade`) | 10 | M | behavioural (medium) | — | 10-lighting.md:166 |
| R-10-7 | Per-column vs global surface cutoff A/B (`TR_SHOT` pair) — keep `surface_cutoff` only if the diff is intentional | 10 | M | pixel (medium) | — | 10-lighting.md:175 |
| R-09-8 | Verify or replace the paint recolor (pixel A/B against a reference frame) — until then TILE-DRAW-4 reads PARTIAL | 09 | M | pixel/determinism (medium) | — | 09-tile-drawing.md:253 |
| R-40-5 | Port `behindTiles` (WoF `NPC.cs:10355`, EoW/Destroyer segments, boss parts) and add the behind-tiles draw pass | 40 | L | pixel/behavioural (medium) | the render-layers work | 40-port-done-verification.md:141 |
| R-37-6 | Effect parity: decode the `.xnb` effect into a real pass table, or hand-port the remaining `ScreenShader` passes (22 today) | 37 | L | pixel (medium) | the renderer/post pipeline work | 37-relogic-libs.md:312 |
| R-02-5 | Re-introduce a layer-target equivalent (wall/black/background/backWater/water caches with partial-offscreen refresh) | 02 | L | pixel (medium) | R-02-1, R-02-2 | 02-main-draw-pipeline.md:138 |
| R-18-6 | Clipping: implement a scissor/rect-clip in `tr_render` and wire `OverflowHidden` into the list screens | 18 | L | pixel (medium) | R-41-5 | 18-ui-framework.md:274 |
| R-09-5 | Vine/rope family (`DrawVines`/`DrawReverseVines`/`DrawVineStrip`/`DrawRisingVineStrip`/`DrawMultiTileVinesInWind` + `DrawTile_BackRope`) | 09 | M | behavioural/pixel (medium) | R-09-6 | 09-tile-drawing.md:244 |
| R-06-3 | Add the `oldPos`/`oldRot`/`oldSpriteDirection` ring buffer (`TrailCacheLength`) + the trail draw pass | 06 | M | pixel (medium) | — | 06-projectile-core.md:138 |
| R-06-4 | Port `Projectile.light` into the slot and the draw's light emission for the ~40 kinds whose `SetDefaults` sets it | 06 | M | pixel (low-med) | R-10-1 | 06-projectile-core.md:141 |
| R-09-2 | Port `CacheSpecialDraws_Part2`'s chest rewrite (`+38/+76` by `Main.chest[i].frame`, `tileHeight = 18`) | 09 | M | behavioural (medium) | — | 09-tile-drawing.md:235 |
| R-14-4 | `MapBG` zone branches (`ZoneDungeon`, `ZoneGlowshroom`, `ZoneCorrupt`/`Crimson`/`Hallow`/`Jungle`, `bloodMoon`, `ZoneGraveyard`) | 14 | M | cosmetic (medium) | module 16 (SceneMetrics) | 14-map-minimap.md:327 |
| R-10-5 | `slotGetFinalLight` in the sliced path (actuation ×0.4 / `light_scale` / paint tint per slice) | 10 | S | pixel (medium) | — | 10-lighting.md:172 |
| R-10-6 | `NON_VISIBLE_PADDING` fixed 18 — drop the `/3` clamp in `blur.rs:43` | 10 | S | pixel (low) | — | 10-lighting.md:174 |
| R-16-9 | Lightning bolt geometry — the `ParticleOrchestrator` bolt spawner + draw and the `thunderSkipSound` writer | 16 | L | pixel (medium) | — | 16-weather.md:319 |
| R-09-1 | Port the platform arms (martian-conduit glow rows 26/27 + the slope side-fill quads) | 09 | S | pixel (medium) | — | 09-tile-drawing.md:232 |
| R-09-7 | `ShouldTileShine` / `Main.shine` into `push_lit` | 09 | S | pixel (medium) | — | 09-tile-drawing.md:252 |
| R-02-2 | Locate and cite the Rust zoom clamp equivalent of `Main.cs:61990-61999` (`ForcedMinimumZoom` × `clamp(1,2)`), or add it | 01 | S | behavioural (medium) | — | 02-main-draw-pipeline.md:129 |
| R-05-10 | `behindTiles` + the render layer (flag on `Npc`, populate from the kind table, behind-tiles pass in `render/entities.rs`) | 05 | L | pixel (medium) | R-40-5 | 05-npc-core.md:283 |
| R-19-4 | The rename display trims (`TrimDisplayIfOverElementDimensions(27)` / `(20)`) | 19 | S | pixel (low) | — | 19-ui-screens-frontend.md:451 |
| R-01-4 | Port `UpdateViewZoomKeys` (±0.02) and clamp `self.zoom` to 1.0–2.0 except under `TR_ZOOM` | 01 | S | cosmetic (low) | — | 01-main-update-loop.md:152 |

### (c) behavioural parity (order / formula) — 97 seeds

| id | One-line title | Module | Size | Severity | Dependencies | Source |
| --- | --- | --- | --- | --- | --- | --- |
| R-04-2 | Projectile hit chain: thread `Projectile.Damage`'s `DamageVar` + owner luck + `strike_npc_damage` into the PVE hit | 04 | L | behavioural + determinism (high) | R-35-1 (RNG decision) | 04-player-combat.md:184 |
| R-39-9 | Port the `aiStyle` dispatch and introduce an `ai_style: u8` field on `Npc`, so "SoA by archetype" has a grouping key (74 of 128 styles unreferenced) | 39 | L | structural (high) | R-25-5 | 39-port-architecture.md:337 |
| R-14-1 | Port `MapUpdateQueue` + `WorldMap.UpdateType` and feed them from the edit/net glue | 14 | S | behavioural (high) | — | 14-map-minimap.md:317 |
| R-14-2 | Wire `ClearEdges` (`WorldMap.cs:260`, `BlackEdgeWidth = 40`) after load and after worldgen | 14 | S | behavioural (low-med) | — | 14-map-minimap.md:323 |
| R-29-1 | Port `WorldGen.ScoreRoom` + the `hiScore > 0` gate and make `scan_spot` return the score-best `bestX/bestY` | 29 | M | behavioural (medium) | — | 29-town-npc-housing.md:297 |
| R-27-9 | Hook fire from the item — port `QuickGrapple_GetItemToUse` + `ItemCheck_TryStartUse`, keeping `TR_HOOK` as the test pin | 27 | M | behavioural (medium) | — | 27-mounts-pets-vehicles.md:308 |
| R-30-5 | Replace the `quick_unlock` bool with a per-entry provider enum (`CommonEnemy`/`TownNPC`/`Critter`/`HighestOfMultiple`/`GoldCritter`/`Salamander`) and record sightings and chats | 30 | M | behavioural (medium) | R-30-4 | 30-achievements-bestiary.md:244 |
| R-25-5 | A per-type speed table for the aiStyle-3 family (the vanilla's per-type `velocity.X` rows) | 25 | M | behavioural (portfolio-wide) | — | 25-npc-ai-audit.md:399 |
| R-05-8 | Port `ScaleStats` with a difficulty enum + `NPCID.Sets.NeedsExpertScaling`/`ProjectileNPC` tables | 05 | L | behavioural (high) | difficulty plumbing (module 24) | 05-npc-core.md:277 |
| R-03-1 | Move the wet/shimmer statics into `update_with_grapple`: add `wet_gravity_and_cap()` from `Player.cs:24382-24437` | 03 | M | behavioural (critical) | — | 03-player-core.md:253 |
| R-05-4 | Wire `strike_npc_damage` + `check_armor_penetration` into the projectile path (`projectile.rs::damage_pve`) | 05 | M | behavioural (high) | — | 05-npc-core.md:257 |
| R-03-4 | Port `Collision.DrownCollision` + `CheckDrowning` as the drowning gate; stop `lose_health` forcing `hurt_cd = 0.8` | 03 | M | behavioural (high) | — | 03-player-core.md:256 |
| R-03-5 | Respawn fidelity: `statLife = max(statLifeMax, statLifeMax2)/2`, `immuneTime = 180`/`60`, plus the moon/DD2 refusal | 03 | M | behavioural (high) | — | 03-player-core.md:257 |
| R-03-6 | Remove the `expert = false` hardcode at `world_sim/npcs.rs:3180` and thread expert through the respawn path | 03 | M | behavioural (medium) | world-gen/difficulty plumbing | 03-player-core.md:258 |
| R-03-7 | Feed `RegenInputs.grappling` from the live hook state so `Player.cs:19241`/`:19426`/`:19449` apply | 03 | M | behavioural (medium) | — | 03-player-core.md:259 |
| R-03-8 | Finish `UpdateDead`: non-persistent buff purge, `chest = -1`, `headPosition`/`headVelocity` | 03 | M | behavioural (medium) | buffs module boundary | 03-player-core.md:260 |
| R-24-3 | Port `GetSpawnRate`'s town-NPC distribution block (`NPC.cs:799-923`) with the graveyard arms | 24 | M | behavioural (high) | the graveyard scene work | 24-spawner-invasions.md:280 |
| R-24-1 | Port the `invaders` spawn-rate override (`NPC.cs:781-785`) and gate the invasion pick behind `rng.next(spawn_rate)` | 24 | S | behavioural (high) | — | 24-spawner-invasions.md:274 |
| R-24-9 | Faithful invasion spawn placement through `FindSpawnTile` + `HasTileSpawnSpace`/`CheckNotSpawningOnScreen` | 24 | L | behavioural (medium) | R-24-1 | 24-spawner-invasions.md:300 |
| R-25-1 | Fix the Frost Legion speeds (`num551`/`num552` from `NPC.cs:28997-29012`) + the type-143 throw and the type-145 jump | 25 | S | behavioural (high) | — | 25-npc-ai-audit.md:387 |
| R-28-4 | The remaining `UpdateBuffs` arms, batch 1 (ids 1–18: ObsidianSkin … Gravitation) | 28 | M | behavioural (high) | R-28-1 | 28-buffs-debuffs.md:254 |
| R-28-5 | The remaining `UpdateBuffs` arms, batch 2 (43, 48, 60-63, 87-90, 93-99, 104-117, 121-124, …) | 28 | M | behavioural (high) | R-28-1 | 28-buffs-debuffs.md:259 |
| R-40-3 | Restore the player damage floor: clamp `amount` to ≥ 1 after the endurance multiply; fix the wrong comment | 40 | M | behavioural (medium) | — | 40-port-done-verification.md:139 |
| R-40-4 | Port the arrow/bow arm of `GetWeaponDamageMultiplier` (`Player.cs:53394-53398`, `bowEffectiveDamage`) | 40 | M | behavioural (medium) | M3-ITEM-3 (ammo) | 40-port-done-verification.md:140 |
| R-40-1 | Fix the hover presets (`HORNET` → 3.5 / 0.021) + the `num *= 2f - scale` factor and the surface-gated damping | 40 | M | behavioural (high) | — | 40-port-done-verification.md:137 |
| R-34-4 | Close the `from_u16` gap for ids 85-88 and 162-168 (11 kinds currently undecodable) | 34 | M | behavioural (medium) | R-34-1 | 34-id-sets-audit.md:264 |
| R-34-5 | Extend `cs_id` to the full roster (43 → 182 kinds) so the statue spawns and `MechSpawn` caps can match | 34 | M | behavioural (medium) | R-34-6 | 34-id-sets-audit.md:268 |
| R-34-7 | Port the missing prefix ids 62..80 into `stat_switch` with the C#'s melee-branch values | 34 | M | behavioural (medium) | module 07 | 34-id-sets-audit.md:277 |
| R-12-7 | Replace the FBM surface model with `TerrainPass`'s actual height/rock lines (including the ≥20 recentering) | 12 | L | behavioural/pixel (medium) | R-12-6 | 12-worldgen.md:361 |
| R-11-7 | Port the force-settle family (`QuickWater`/`SettleWaterAt`/`AttemptToMoveHoney`/`Lava`/`Shimmer`, load settle, stuck, panic) | 11 | L | behavioural (medium) | R-11-5, module 42 | 11-liquid.md:201 |
| R-05-6 | Port `UpdateNPC`'s tail phases (`CheckActive`+`timeLeft`, `CheckDrowning`, `CheckLifeRegen`, `GetHurtByOtherNPCs`, the velocity.X snap, `UpdateNetworkCode`) | 05 | M | behavioural (medium) | R-05-5 | 05-npc-core.md:268 |
| R-05-7 | Port `Transform` as a function (reposition, `SetDefaults(newType)`, life-ratio carry-over, town-home switch, `TransformVisuals`) | 05 | M | behavioural (medium) | — | 05-npc-core.md:273 |
| R-22-2 | Port `GetItemExpectedPrice` fully (the 0.8 discount, `buyOnce` `/5` + min-1 clamp, `GetStoreValue`, `isAShopItem` aliasing) | 22 | M | behavioural (medium) | R-22-6 | 22-chest-loot-shops.md:230 |
| R-06-5 | Replace the point hit test with `Damage_GetHitbox()`'s rectangle for the flight-size boxes | 06 | M | behavioural (medium) | — | 06-projectile-core.md:143 |
| R-06-6 | Complete the spawn protocol: `FindOldestProjectile` with the `netImportant` filter, the golf-ball 10-cap, `CopiesOwnerAttackCDToLocalImmunityOnSpawn` | 06 | M | behavioural (medium) | — | 06-projectile-core.md:145 |
| R-06-1 | Hoist the `timeLeft`/`penetrate` bookkeeping and the `Damage()` call back inside the sub-step loop | 06 | S | behavioural (medium) | — | 06-projectile-core.md:132 |
| R-03-2 | `maxFallSpeed += 0.01f` + `PortalPhysicsEnabled = 35` + `vortexDebuff` gravity = 0 | 03 | S | behavioural (high) | — | 03-player-core.md:254 |
| R-03-3 | Port `breathCDMax` as a property (7 base, ×2 Breathing Reed, ×6 Diving Helm) | 03 | S | behavioural (medium) | item registry access | 03-player-core.md:255 |
| R-04-1 | Floor and iframe consistency: `if (num2 < 1.0) num2 = 1` before the `statLife` subtraction, feed the floored value to `iframes_for` | 04 | S | behavioural (medium) | — | 04-player-combat.md:180 |
| R-04-3 | Port `Player.Hurt`'s dodge block + the four accessory flags (`blackBelt`, `shadowDodge`, `mysticSashDodge`, `brainOfConfusionItem`) | 04 | M | behavioural (medium) | M3-EQUIP-3 flag table | 04-player-combat.md:190 |
| R-04-4 | Add `bow_effective_damage` / `gun_effective_damage` / `specialist_effective_damage` + the AmmoID routing; move the truncation to `:53428` | 04 | S | behavioural (medium) | — | 04-player-combat.md:195 |
| R-04-6 | Apply `DamageVar(npc.damage * multiplier, -luck)` at `Player.cs:31387` in `npc.rs::contact` | 04 | S | behavioural (medium) | — | 04-player-combat.md:203 |
| R-20-3 | `PickItemMovementAction` as a real table (`:1480-1634`) routed to the equip/dye/shop/Journey arms | 20 | M | behavioural (medium) | R-07-6 (ItemDef columns) | 20-ui-ingame.md:217 |
| R-20-4 | Right-click equip/swap arms (`SwapEquip`/`SwapVanityEquip`/`TryItemSwap`/`TryOpenContainer`) | 20 | M | behavioural (medium) | R-20-3 | 20-ui-ingame.md:220 |
| R-20-6 | Decide `retainFavoriteOnMoveAt` + item-level favourites vs slot-index favourites | 20 | M | behavioural (medium) | R-19-5 | 20-ui-ingame.md:226 |
| R-25-6 | Deduplicate `aiStyle 2` — make `update_eye` call `npc_ai::flyer_step` and add the vanilla's alpha phase | 25 | M | behavioural (medium) | — | 25-npc-ai-audit.md:404 |
| R-25-4 | Re-point `GoblinSummoner` from `update_caster` to an `AI_003` summon arm (or document the substitute) | 25 | S | behavioural (medium) | R-05-3 | 25-npc-ai-audit.md:396 |
| R-29-2 | Add the housing occupancy rule (`housing_category` on `Npc`, `can_npcs_live_with_each_other`, the two occupancy gates) | 29 | M | behavioural (medium) | R-29-1 | 29-town-npc-housing.md:301 |
| R-29-3 | Port the mid-game relocation (`WorldGen.MoveTownNPC` 4685 / `moveRoom` 4843) + the `lookForHomeTimeout` gate | 29 | M | behavioural (medium) | R-29-1 | 29-town-npc-housing.md:306 |
| R-29-7 | Add the pylon duplicate-type gate (`TETeleportationPylon.cs:13-27`, `:153-161`) and the `Framing_CheckTile` item drop | 29 | S | behavioural (medium) | M7-TE-2 | 29-town-npc-housing.md:320 |
| R-29-8 | Port the remix arms of `DoesPylonAcceptTeleportation` (`:252-254`, `:276-279`, `:284-285`) | 29 | S | behavioural (low-med) | — | 29-town-npc-housing.md:323 |
| R-29-4 | Fix the mood exclusion block (`ShopHelper.cs:103-106`): the `Main.remixWorld` early return + the 37/453 exclusions | 29 | S | behavioural (medium) | — | 29-town-npc-housing.md:310 |
| R-16-1 | Wire `Wind::any_player_120_life` from the world tick + assert the `abs(target) ≤ 0.35` clamp | 16 | M | behavioural (medium) | — | 16-weather.md:287 |
| R-16-2 | Take `Sandstorm.cs:58-59`'s hardMode branch in `Sandstorm::update_time`'s start denominator | 16 | M | behavioural (medium) | — | 16-weather.md:291 |
| R-16-3 | Port `Main.snowing()` (`Main.cs:13032-13051`) as the snow weather's single source | 16 | M | behavioural/pixel (medium) | a Deerclops-proximity query | 16-weather.md:294 |
| R-16-5 | Feed the live wind into the rain field (`wind.speed_current * 18` per drop) | 16 | M | behavioural/pixel (medium) | — | 16-weather.md:305 |
| R-16-6 | Port `NewRain`'s spawn factors (`gfxQuality`, the snow suppression, the `atmo⁹`/`< 0.4` gates, `waterStyle*3` frames) | 16 | S | behavioural/pixel (medium) | — | 16-weather.md:309 |
| R-28-9 | Fix the Nurse debuff rule (honour `NurseCannotRemoveDebuff` in the heal and in `nurse_heal_cost_for`) | 28 | S | behavioural (medium) | R-28-2 | 28-buffs-debuffs.md:276 |
| R-15-6 | Wire `ChildSafety`/`Settings::blood` at the spawn site of both pools (not at draw) | 15 | M | behavioural (medium) | R-15-5 | 15-dust-gore.md:275 |
| R-15-8 | Re-point the 3 stand-in consumers (NPC/Projectile death bursts, player step/dash dust, TileDrawing hit dust) at the shared pool | 15 | M | behavioural (medium) | R-15-4 | 15-dust-gore.md:280 |
| R-19-5 | `.plr` rename read-modify-write (read → rewrite the name → re-list) | 19 | M | behavioural (medium) | R-19-1 | 19-ui-screens-frontend.md:457 |
| R-19-2 | Gate Play on validity (`IsValid` + `TryMovingToRejectionMenuIfNeeded`) | 19 | S | behavioural (medium) | — | 19-ui-screens-frontend.md:442 |
| R-19-3 | Wire the active player's difficulty into the world list (`sort_worlds` in `enter_front`) | 19 | S | behavioural (low) | R-19-8 | 19-ui-screens-frontend.md:448 |
| R-11-1 | Add the `delay` field to `Entry` + the lava 5 / honey 10 gate and the `quickFall`-sets-10 pre-pass flag | 11 | S | behavioural (med-high) | — | 11-liquid.md:183 |
| R-11-2 | Port underworld water evaporation (`Liquid.cs:466-474`) and update the mass-conservation test | 11 | S | behavioural (medium) | — | 11-liquid.md:187 |
| R-11-6 | Port `AddWater`'s missing guards (5-tile border, `type != 546`, `CheckWaterDeath`/`CheckLavaDeath` → kill) | 11 | M | behavioural (low-med) | — | 11-liquid.md:199 |
| R-05-5 | Port the `GetAvailableNPCSlot` protocol (200-slot array, `SearchSpawnSlotsInReverse`, `CannotSpawnInSlot0`, the two-pass search) | 05 | M | behavioural/determinism (medium) | — | 05-npc-core.md:263 |
| R-05-3 | Remove the shadowed goblin arm so `GoblinThief/Warrior/Peon` reach `update_skeleton` | 05 | S | behavioural (low-moderate) | — | 05-npc-core.md:253 |
| R-24-2 | Port `ShouldSpawnInvasionEnemies`'s town fallback (`NPC.cs:455-468`) with its `Next(3)` | 24 | S | behavioural (low-med) | — | 24-spawner-invasions.md:278 |
| R-24-6 | Despawn encouragement: port `EncourageDespawn` + the two `DespawnEncouragement_*` helpers | 24 | M | behavioural (medium) | — | 24-spawner-invasions.md:292 |
| R-12-4 | Skyblock gate — read `VANILLA_CHAIN`'s `gate` in `run` and implement the skyblock shape | 12 | M | behavioural (medium) | R-12-2 | 12-worldgen.md:350 |
| R-07-1 | Fix the pickup double-add (`world_sim\items.rs:191-194`) and add a regression test | 07 | S | behavioural (high) | — | 07-item-core.md:298 |
| R-07-3 | Price the instance, not the kind — `GetStoreValue` semantics + the `buyOnce`/`discountAvailable` arms | 07 | M | behavioural (medium) | — | 07-item-core.md:306 |
| R-13-1 | Port `WetCollision`'s box scan (surface offset + slope support arm) and feed the four flags from it | 13 | S | behavioural (medium) | — | 13-collision.md:143 |
| R-13-4 | Switch the AI to the faithful LOS walks (`can_hit_line`/`can_hit` instead of `line_clear`) | 13 | S | behavioural (medium) | — | 13-collision.md:155 |
| R-08-2 | Finish `wall_frame_of`: `TruncatesWalls` gate, `invisibleWall` gate, the wall-21 roll, the `wall == 0` reset | 08 | S | behavioural/pixel (medium) | — | 08-tile-framing.md:156 |
| R-08-3 | Port the `wallLargeFrames == 1/2` phlebas/lazure lookups into `wall_frame_of` | 08 | S | behavioural (low-med) | R-08-2 | 08-tile-framing.md:159 |
| R-09-3 | `WindGrid` push wave + `EnsureWindGridSize` as a `tr_core::wind` extension | 09 | M | behavioural (medium) | module 16 (WEATHER-1 row) | 09-tile-drawing.md:238 |
| R-09-9 | Quality-tier constants reconciliation (`:479-486` vs `tiles.rs:362-366`) | 09 | S | behavioural/pixel (medium) | — | 09-tile-drawing.md:256 |
| R-10-2 | Water-style decay + buff/debuff decay multipliers in `LightParams`/`LightKey` | 10 | M | behavioural (medium) | — | 10-lighting.md:163 |
| R-18-3 | Port margins into `uistate::resolve` (add then subtract, `:293-297`/`:317-318`) | 18 | M | behavioural (latent) | — | 18-ui-framework.md:264 |
| R-18-4 | Model the `UIList` ∞-height parent (`parent[3] = f32::MAX`) and test a percent-height child | 18 | M | behavioural (latent) | — | 18-ui-framework.md:267 |
| R-18-5 | Ancestor bubble — an optional parent chain in `pointer.rs` + ancestor `IsMouseHovering` | 18 | M | behavioural (medium) | — | 18-ui-framework.md:270 |
| R-20-1 | Pin the right-click amount — a one-item `PickupItemIntoMouse` equivalent + the `stack_split > 1` gate | 20 | S | behavioural (medium) | — | 20-ui-ingame.md:209 |
| R-20-7 | Coin-slot merge — `CoinSlot.ForceSlotState`'s rank arithmetic | 20 | S | behavioural (medium) | — | 20-ui-ingame.md:230 |
| R-20-8 | Max-stack audit — replace `MAX_STACK` with `registry::max_stack` at `clicks.rs:934` | 20 | S | behavioural (low) | R-07-1 | 20-ui-ingame.md:232 |
| R-22-6 | The two dead stock gates (`HardModeMoonlord`, `DownedBothSlimes`) once the boss flags reach `World` | 22 | S | behavioural (small) | — | 22-chest-loot-shops.md:246 |
| R-22-8 | Price rounding audit (f64 vs f32) if a half-value price shows up in play | 22 | S | behavioural (cosmetic) | R-22-2 | 22-chest-loot-shops.md:251 |
| R-41-9 | Locate or add the `Main.ForcedMinimumZoom` equivalent (`Main.cs:243`, `:61991`) and the runtime `GameZoomTarget` step | 41 | M | behavioural/pixel (medium) | module 01's camera work | 41-render-pipeline.md:294 |
| R-42-1 | Write the four wire bits in `sections/tiles.rs::write_tile_record` and add them to `encoded_same` | 42 | S | behavioural (high) | — | 42-save-load-audit.md:270 |
| R-42-3 | Own the world id / GUID / generator version / `LastPlayed` in `World` and write them | 42 | S | behavioural (medium) | — | 42-save-load-audit.md:278 |
| R-42-4 | `.plr` byte parity — extend `write_player_bytes` to the full `Player.Serialize` field list | 42 | S | behavioural (medium) | — | 42-save-load-audit.md:280 |
| R-42-5 | Rolling backups + the post-write `ValidateWorld` rollback | 42 | S | behavioural (medium) | — | 42-save-load-audit.md:283 |
| R-42-7 | Verify the autosave tick against `Main.cs:19737-19765` (the 600 s world save **and** the 300 s player save) | 42 | S | behavioural (medium) | — | 42-save-load-audit.md:288 |
| R-01-1 | Move `advance_time` (frame.rs:160-189) inside the `menu.is_none() && front.is_none()` world branch | 01 | S | behavioural (medium) | — | 01-main-update-loop.md:139 |
| R-01-3 | Port `gamePaused`/`CanPauseGame` semantics + `DoUpdate_WhilePaused`'s inventory refresh/scroll/drop-check | 01 | M | behavioural (medium) | — | 01-main-update-loop.md:148 |
| R-01-5 | Port the C# draw-time camera (`PlayerFocusedScreenPosition`, `cameraX/cameraY` decay, `cameraLerp`, `ClampScreenPositionToWorld`) | 01 | M | behavioural/sub-pixel (medium) | — | 01-main-update-loop.md:155 |
| R-01-6 | Add `camera.rs` unit tests + a frame-level integration test asserting the branch order fixed by LOOP-2 | 01 | S | verification (medium) | R-01-2 | 01-main-update-loop.md:159 |
| R-02-1 | Write a test asserting the `frame.rs` phase order against `Main.cs:62229-62326` | 02 | S | verification (medium) | — | 02-main-draw-pipeline.md:126 |
| R-03-9 | Verify the in-water ascent: decide whether C# supports `self.vy = -2.0` (marked **UNVERIFIED** deliberately) | 03 | S | behavioural (medium) | R-03-1 | 03-player-core.md:261 |
| R-26-2 | Wire `ai_vortex_ritual`/`ai_vortex_bolt` into the dispatch (aiStyle 88/108) and fix the M4-EVENT-17 + PORT_MAP 236 "done" claims | 26 | S | behavioural (medium) | — | 26-projectile-ai-audit.md:158 |
| R-26-3 | Add the aiStyle-70 arm and `ai_bubble` (the Flairon's bubble homing, C# 30304-30352) or correct PORT_MAP 157 | 26 | S | behavioural (medium) | — | 26-projectile-ai-audit.md:160 |
| R-27-2 | Restore the `CanMount` arms — the `m == 56` wet rule and the Cart/`CanUseHooks` grapple exemption (C# `Mount.cs:6595-6602`) | 27 | S | behavioural (medium) | R-27-1 | 27-mounts-pets-vehicles.md:287 |
| R-27-3 | Fishron halving — port `Player.cs:26655-26659` into `player.rs::update_mount` | 27 | S | behavioural (medium) | — | 27-mounts-pets-vehicles.md:290 |
| R-27-4 | `SetMount`/`Dismount` extras — the type-44 velocity/dash reset, the 7/8 `fullRotationOrigin`, the `fullRotation` reset, the `noItems` clear for 55/56 | 27 | S | behavioural (low) | — | 27-mounts-pets-vehicles.md:293 |
| R-27-6 | SuperCart constants — replace the `0.0` placeholders with 20/20/0.1/15/5.15 (`Mount.cs:6660-6664`) and add the `should_super_cart` writer | 27 | S | behavioural (latent) | — | 27-mounts-pets-vehicles.md:304 |
| R-30-1 | Model `NPCKilledCondition.CreateMany` faithfully — a "one condition per id" variant + re-transcribe the 4 rows (46 conditions, not 5) | 30 | S | behavioural (high) | — | 30-achievements-bestiary.md:232 |
| R-30-2 | Call `achievements::save` from the autosave/F9 pass + a write→read round-trip test through the app path | 30 | S | behavioural (medium) | — | 30-achievements-bestiary.md:235 |
| R-30-4 | Make `kills_needed_for` fall back to `ItemID_Sets_DefaultKillsForBannerNeeded` (50) instead of 0 | 30 | S | behavioural (high) | — | 30-achievements-bestiary.md:240 |
| R-30-7 | Fire `TryGrantingBestiary100PercentAchievement` from `KillTracker::set_kill_count_directly` | 30 | S | behavioural (low) | R-30-3 | 30-achievements-bestiary.md:251 |
| R-36-6 | The per-kind/per-tier kill worth — port `GetMonsterPointsWorth` (`DD2Event.cs:546-554` → `:1106`, `:1199`, `:1509`) and drop the constant `KILL_WORTH_T1` | 36 | S | behavioural (medium) | — | 36-cinematics-film.md:186 |
| R-36-7 | The D1 wave-6 win transition — the `waveNumber`/`currentKillCount` write-back + the victory-scene call (`DD2Event.cs:998-1005`) | 36 | S | behavioural (medium) | — | 36-cinematics-film.md:190 |
| R-36-9 | The wave-break skip — `IsStandActive` + `RequestToSkipWaitTime` + `AttemptToSkipWaitTime` (`DD2Event.cs:1765-1801`) | 36 | S | behavioural (medium) | — | 36-cinematics-film.md:193 |
| R-36-10 | `DropMedals` (`DD2Event.cs:802-813`, called at `:405-420`) → the Defender Medal item 3817 | 36 | S | behavioural (loot) | — | 36-cinematics-film.md:196 |
| R-36-13 | The zone metric — `Player.SceneMetrics.CloseEnoughToDD2LanePortal` → `Player.ZoneOldOneArmy` + `NPC.ZoneOldOneArmy` + the `NPC.cs:799` despawn gate | 36 | S | behavioural (low) | — | 36-cinematics-film.md:204 |
| R-36-14 | The OOA's invasion id — set `invasion_type = 3` (`DD2Event.cs:47`) and route the progress through `invasion_progress_icon = 3` | 36 | S | behavioural (UI) | — | 36-cinematics-film.md:207 |
| R-36-15 | Replace the `DD2Event.Ongoing` stub (`dd2.rs:17-19`) with the world's real flag and re-enable the sentry-persistence gate | 36 | S | behavioural (low) | — | 36-cinematics-film.md:210 |

### (d) missing subsystems — 51 seeds

| id | One-line title | Module | Size | Severity | Dependencies | Source |
| --- | --- | --- | --- | --- | --- | --- |
| R-26-8 | Port `AI_099_1_Counterweights` (67380) + the counterweight rows 556-561/1079 | 26 | L | behavioural (medium) | — | 26-projectile-ai-audit.md:171 |
| R-26-9 | Port the remaining ~150 vanilla aiStyles (the 1.4 boss/event families 130-210 and the pre-1.3 groups 30-99), one family per row | 26 | L | behavioural (high) | R-26-5 | 26-projectile-ai-audit.md:172 |
| R-12-3 | Port the cheap missing passes (Life Crystals, Statues/Traps/Piles, Sunflowers, Flowers, Mushrooms, Herbs/Dye Plants) | 12 | M | behavioural (medium) | — | 12-worldgen.md:338 |
| R-12-5 | Liquid settlement pass — a minimal `Liquid` update that satisfies `WorldGenerator.cs:508`'s invariant, then the two `Settle Liquids` passes | 12 | M | behavioural (high) | R-11-5 | 12-worldgen.md:353 |
| R-11-5 | Port the liquid reactions (`LiquidCheck`, `GetLiquidMergeTypes`, `CreateLiquidMergeTile`, `LiquidOverwriteStrip`; tiles 56/229/230/659) | 11 | M | behavioural (high) | a tr-core `edit.rs` seam | 11-liquid.md:195 |
| R-15-2 | Port the `Dust` record + the `NewDust` family behind a `DustPool` with `spawn/next_index` | 15 | M | behavioural+pixel (high) | R-15-1 | 15-dust-gore.md:259 |
| R-26-4 | Port the aiStyle-53 / 197 sentry shot machines on top of the existing `store_sentry_aim` seam | 26 | M | behavioural (medium) | — | 26-projectile-ai-audit.md:162 |
| R-26-5 | Port the aiStyle-62 minion attack-shot families (the six kinds' projectiles need `proj_data` rows + arms) | 26 | M | behavioural (medium) | — | 26-projectile-ai-audit.md:164 |
| R-26-6 | Port `AI_001`'s remaining per-type arms (142 `type ==` checks) or split the style-1 arm into a per-kind table | 26 | M | behavioural (high) | — | 26-projectile-ai-audit.md:166 |
| R-26-7 | Port the 21 gap-list styles (the sentry/turret group 120/123/130/134/137/138, the pillar group 88/108/133, the small ones 2/23/26/66/67/70/79/80/157/158/187/189) | 26 | M | behavioural (high) | R-26-2 | 26-projectile-ai-audit.md:168 |
| R-27-7 | `Minecart.FrameTrack` — port the track-tile placement used by the item use (`Player.cs:46058`) | 27 | S | behavioural (low) | — | 27-mounts-pets-vehicles.md:306 |
| R-27-8 | The missing 16 mount rows (ids 4, 17, 37, 45, 46, 52, 54, 55, 57-60, 62-65) with their C# constants and item→mount rows | 27 | M | behavioural (medium) | — | 27-mounts-pets-vehicles.md:296 |
| R-27-10 | Petting — `Player.petting` state machine + `Mount.TryPettingMount` for the Chillets | 27 | M | behavioural (feature gap) | R-27-8 | 27-mounts-pets-vehicles.md:310 |
| R-27-12 | Mount abilities — `UseAbility`/`StartAbilityCharge`/`AimAbility`/`UpdateDrill` + the dash mounts 62-65 | 27 | L | behavioural (medium) | R-27-8 | 27-mounts-pets-vehicles.md:316 |
| R-30-3 | Add the missing notify sites: `NotifyProgressionEvent` (43 rows), the custom-flag `Complete()` calls (37 rows) and the `CustomInt`/`CustomFloat` increments | 30 | M | behavioural (high) | R-30-2 | 30-achievements-bestiary.md:237 |
| R-30-6 | Port `ReportDroprates` — add a `report_droprates` method to the drop-rule traits and feed the rows into the bestiary entries | 30 | L | behavioural (medium) | — | 30-achievements-bestiary.md:248 |
| R-36-4 | Port `DD2Film` (`DD2Film.cs:1-396`): the dryad/ogre/portal staging, the emote bubbles, `CombatText.clearAll`, the fade/dust, the friendly flip | 36 | M | behavioural (dead-code parity) | R-36-2 | 36-cinematics-film.md:180 |
| R-36-5 | The OOA's loss path — `DD2Event.ReportLoss` + `StopInvasion(false)` → `update_dd2_event` must clear `dd2_event_ongoing` | 36 | M | behavioural (medium) | — | 36-cinematics-film.md:183 |
| R-36-8 | `DD2Event.StartVictoryScene` (`DD2Event.cs:481-505`): the crystal's victory animation and the 549 portals' close | 36 | M | behavioural/visual (medium) | R-36-7 | 36-cinematics-film.md:193 |
| R-36-12 | The event's cleanups — `WipeEntities`/`ClearAllTowersInGame`/`ClearAllDD2HostilesInGame`/`ClearAllDD2EnergyCrystalsInChests`, the arena hitbox and `ShouldBlockBuilding` | 36 | M | behavioural (medium) | — | 36-cinematics-film.md:201 |
| R-15-3 | Port `UpdateDust`'s generic tail (position integration, the `!noGravity` arm, the screen-bottom kill, the `dCount` nibbles) | 15 | M | behavioural (high) | R-15-2 | 15-dust-gore.md:264 |
| R-15-5 | Port `Gore` + `NewGore` + `Gore.Update` + `GoreID.Sets` (600-slot pool, `SpecialAI`/`IsDrip`/`DisappearSpeed`) | 15 | L | behavioural+pixel (high) | R-15-2 | 15-dust-gore.md:268 |
| R-25-10 | The 68 unported `aiStyle` arms, grouped by content wave (1.4.4 critters, 1.4.5 bosses, the lunar leftovers) | 25 | L | behavioural (high) | R-25-5 | 25-npc-ai-audit.md:416 |
| R-25-9 | The town NPC family (`aiStyle 7`, 52 types): the homes, the sitting, the idle animations, the shimmer | 25 | L | behavioural (high) | module 29 | 25-npc-ai-audit.md:413 |
| R-05-9 | Close the `aiStyle` gap by porting the missing archetypes as `npc_ai` steps rather than per-kind machines | 05 | L | behavioural (high) | R-25-10 | 05-npc-core.md:280 |
| R-31-2 | Port the generic ball engine (`Terraria/Physics/BallCollision.cs:24-89`) to `crates/tr-core/src/ball.rs` | 31 | M | behavioural (feature-level) | — | 31-golf-ball-physics.md:201 |
| R-31-3 | Port `GolfHelper` (shot strength + club table + `HitGolfBall` + the contact listener) to `crates/tr-app/src/golf.rs` | 31 | M | behavioural (feature-level) | R-31-1, R-31-2 | 31-golf-ball-physics.md:205 |
| R-31-4 | Port `GolfState` + `GolfBallTrackRecord` and wire `Player.golfer_score` ← `accumulate_golfing_score` | 31 | M | behavioural (feature-level) | R-31-2 | 31-golf-ball-physics.md:208 |
| R-31-5 | Port the two projectile AIs (721 ball `AI_149_GolfBall`, 722 club helper `AI_150_GolfClubHelper`) + the `proj_data` rows | 31 | M | behavioural (feature-level) | R-31-2 | 31-golf-ball-physics.md:211 |
| R-10-1 | Feed entity/projectile/item `AddLight` sources (Projectile 107 / NPC 80 / Dust 94 / Player 46 / WorldItem 69) | 10 | M | behavioural (high) | the fixed-step ordering in `world_sim/step.rs` | 10-lighting.md:158 |
| R-24-5 | Lantern Night + Birthday Party + Sandstorm event + the cultist ritual delay (four schedulers on `World`) | 24 | M | behavioural (medium) | — | 24-spawner-invasions.md:287 |
| R-24-4 | Moon-event wave rosters (`SpawnAnNPC`'s pumpkin/snow-moon arms `NPC.cs:2560-2960`) | 24 | M | behavioural (high) | — | 24-spawner-invasions.md:284 |
| R-13-3 | Port `StickyTiles`, `AnyHurtingTiles`/`CanTileHurt` and `SwitchTiles` | 13 | M | behavioural (medium) | R-08-6 (the hurting-id table) | 13-collision.md:152 |
| R-13-6 | Port `BuildTileContacts`, `noSlopeCollision`, `AdvancedTileCollision`, `DrownCollision`/`GetWaterLine` | 13 | M | behavioural (medium) | R-03-4 | 13-collision.md:163 |
| R-09-4 | Tile animator — `Main.tileFrame[type]` advance (`TileDrawing.Update` :367-388 + the per-type counter rules) | 09 | L | behavioural (high) | — | 09-tile-drawing.md:241 |
| R-02-3 | Port the full-screen effect set (`ScreenDarkness`, `ScreenObstruction`, `DrawPaladinsShield`, `DrawInfernoRings`, lens flare) | 02 | M | cosmetic/pixel (medium) | R-02-2 | 02-main-draw-pipeline.md:131 |
| R-02-6 | Particle-system parity (`ParticleSystem_World_BehindPlayers/OverPlayers`) + the C# draw-cache buckets | 02 | M | behavioural/pixel (medium) | R-02-1 | 02-main-draw-pipeline.md:142 |
| R-02-4 | Capture mode — a minimal `CaptureManager` (selection rect, `IsCapturing` suppression, scene-metrics skip, write-out) | 02 | M | behavioural (medium) | — | 02-main-draw-pipeline.md:134 |
| R-04-7 | The remaining `Hurt` arms (`magicCuffs`, retaliate, paladin share, beetle orbs, solar flare, combat text) | 04 | M | behavioural (medium) | R-04-3 | 04-player-combat.md:206 |
| R-16-4 | Port the live cloud system (`Cloud.cs`: `addCloud`'s ladder, `UpdateClouds`, `RollRareCloud`, `cloudColor`) | 16 | L | behavioural (medium) | R-16-3 | 16-weather.md:300 |
| R-16-8 | Port the Lantern Night arms (`cloudBGActive = 0`/`numClouds = 30`, the `!slimeRain && !LanternsUp` gate, the wind-walk freeze) | 16 | M | behavioural (low priority) | the Lantern Night event (M4) | 16-weather.md:314 |
| R-29-6 | Port the town chatter (`NPC.GetChat`, `ConditionalDialogue`, the ten per-NPC chatter tables) | 29 | M | behavioural (feature gap) | M6-LOCALIZE | 29-town-npc-housing.md:317 |
| R-31-6 | Port `Player.TryPlacingAGolfBallNearANearbyTee` + `GetPreferredGolfBallToUse` (tee scan ±2 tiles, nearest-to-mouse tee, kill old balls) | 31 | S | behavioural (medium) | R-31-5 | 31-golf-ball-physics.md:215 |
| R-25-7 | Port the `aiStyle 84` cultist boss (`AI_084_LunaticCultist`) and fix the `AncientCultist` identity (439 vs 422) | 25 | M | behavioural (medium) | R-34-1, module 34 | 25-npc-ai-audit.md:407 |
| R-06-8 | Port `Kill()`'s full arm set (channel cancel, self-damage, per-type kill arms, gore/dust/sound) + `CutTiles`/`SwitchTiles` | 06 | L | behavioural (medium) | R-06-7 | 06-projectile-core.md:150 |
| R-08-1 | Port `Framing.SelfFrame8Way` + `BlockStyle` + `FindBlockStyle` + `WillItBlend` and call it from the 255-268 arm | 08 | S | behavioural (medium) | — | 08-tile-framing.md:153 |
| R-13-2 | Port `HitTiles`/`HitTilesInACircle` + the tile-hit resolution (kill/drop path) | 13 | M | behavioural (medium-high) | the edit/kill events | 13-collision.md:148 |
| R-01-7 | `ShouldUpdateEntities`-equivalent gate + the world-loading state machine (`WorldPreparationState`) | 01 | L | behavioural (medium) | module 12 (worldgen) | 01-main-update-loop.md:162 |
| R-14-9 | Legacy `.map` migration — `DecompressChunk` + `LoadMapVersion1` / `LoadMapVersionCompressed` | 14 | L | behavioural (medium) | — | 14-map-minimap.md:347 |
| R-36-2 | Port the film framework verbatim into `crates/tr-core/src/cinematics.rs` (`Film.cs:1-144` + `FrameEvent.cs` + `FrameEventData.cs` + `CinematicManager.cs:1-41`) + a `Main.cs:16789`-equivalent hook | 36 | S | behavioural (dead-code parity) | — | 36-cinematics-film.md:173 |
| R-36-3 | Port `DSTFilm` (`DSTFilm.cs:1-177`): the `Main.hideUI` toggle, the `Player.isControlledByFilm` flag + the `Player.cs:24765` gate, the Deerclops/Chester spawns, the DST shader item 5113 | 36 | S | behavioural (dead-code parity) | R-36-1 | 36-cinematics-film.md:177 |

### (e) data / coverage gaps (includes doc-truth and verification work) — 63 seeds

| id | One-line title | Module | Size | Severity | Dependencies | Source |
| --- | --- | --- | --- | --- | --- | --- |
| R-34-11 | Port the remaining "no Rust counterpart" sets (`SoundID`, `PlayerItemSlotID`, `ExtrasID`, `GlowMaskID`, `GoreID`, `NPCHeadID`, `MessageID`, `CloudID`, `ChainID`, `ProjectileSourceID`, `ImmunityCooldownID`, `SurfaceBackgroundID`, `PlayerTextureID`, `PaintID`/`PaintCoatingID`, `ItemRarity`) | 34 | L | data coverage (medium) | — | 34-id-sets-audit.md:294 |
| R-34-10 | Port the `ArmorIDs` per-slot id spaces (15 slots, Head 293 … Beard 5) and the `ArmorIDs.Sets.*` families | 34 | L | data coverage (medium) | modules 07/20/29 | 34-id-sets-audit.md:289 |
| R-34-8 | Generate the `ProjectileID` id-constant table (all 1136 ids) and give `proj_data` a documented default arm | 34 | M | data coverage (medium-high) | modules 06, M4-PROJ | 34-id-sets-audit.md:281 |
| R-34-9 | Port the `DustID` id table (332 ids) with the dust-type → texture/frame mapping | 34 | M | data coverage (medium) | R-15-2 | 34-id-sets-audit.md:285 |
| R-32-1 | Port the `SoundID` style table (506 fields) and wire `Audio::play_style` to it | 32 | S | data coverage (scope) | — | 32-sound-audio.md:243 |
| R-28-6 | The NPC buff completion — the full `NPCID.Sets.DebuffImmunitySets` table + the unported NPC flags + `UpdateNPC_SoulDrainDebuff` | 28 | L | data coverage (medium) | R-28-1 | 28-buffs-debuffs.md:263 |
| R-40-10 | Re-verify the untested `✅` rows (M1 visual, M3 items/equipment, M5 worldgen, M6 audio/net/save, M7 UI/wiring — ~180 rows) | 40 | L | verification (medium) | — | 40-port-done-verification.md:146 |
| R-28-2 | `BuffID.Sets` completion (`IsAFlaskBuff`, `NurseCannotRemoveDebuff`, `IsAnNPCWhipDebuff`, `SortingPriorityFoodBuffs`, `BuffTextHandlers`) | 28 | M | data coverage (medium) | R-28-1 | 28-buffs-debuffs.md:245 |
| R-28-1 | Add the `BuffId` id module (401 named constants from `BuffID.cs:173-989`) and replace the literals | 28 | S | data coverage (maintainability) | — | 28-buffs-debuffs.md:241 |
| R-15-4 | Port the per-type dust table (`DustID` constants, the `GetAlpha`/`GetColor` switches, `GetVisualRotation/Scale`, the `UpdateDust` if-chain) | 15 | L | data coverage (medium) | R-15-3, R-34-9 | 15-dust-gore.md:268 |
| R-22-3 | The 13 skipped loot rows (`SlimeBody`, `StatueMimic`, `MechBossSpawners`, `FromOptionsWithoutRepeats` + `NamedNPC`/`RedHatSkeletron`) | 22 | M | data coverage (medium) | the rule-engine work | 22-chest-loot-shops.md:234 |
| R-22-4 | Shop-index coverage audit — enumerate every `case` of `Chest.SetupShop` against `shop.rs::setup_shop` | 22 | M | data coverage (medium) | — | 22-chest-loot-shops.md:239 |
| R-22-5 | Chest icon tables (`chestTypeToIcon` / `chestTypeToIcon2` / `dresserTypeToIcon` + `Initialize`'s overrides) | 22 | S | data coverage (cosmetic) | `GetItemDrop_Dressers` | 22-chest-loot-shops.md:244 |
| R-22-7 | Bestiary drop rates — `DropRateInfo` + `ReportDroprates` (`DropRateInfoChainFeed`) | 22 | S | data coverage (cosmetic) | M4-BEST-1 | 22-chest-loot-shops.md:248 |
| R-12-8 | Assign the 20 orphan passes (`Quick Cleanup`, `Tile Cleanup`, `Final Cleanup`, `Wall Variety`, `Cave Walls`, `Grass Wall`, `Guide`, `Herbs`, …) to real task ids | 12 | M | coverage/roadmap (medium) | — | 12-worldgen.md:365 |
| R-37-8 | Cover the 23 unsupported retail textures (multi-mip + the remaining surface formats) | 37 | M | data coverage (medium) | — | 37-relogic-libs.md:319 |
| R-31-1 | Extract the golf material data (5 floats × N materials) into a Rust table | 31 | S | data coverage (blocks everything) | — | 31-golf-ball-physics.md:198 |
| R-07-6 | Verify the generated table row-by-row against a decompile-wide extract of `SetDefaults1..5` | 07 | L | data coverage (medium) | — | 07-item-core.md:319 |
| R-08-5 | Write a `TileObjectData` generator that parses `TileObjectData.Initialize` (`:2110`) into the `TABLE` consts | 08 | M | data coverage (medium) | — | 08-tile-framing.md:164 |
| R-08-6 | Audit `tiledata` coverage: list the `TileID.Sets.*` consumed across `crates/` and diff against `tiledata/sets.rs` | 08 | S | data coverage (medium) | — | 08-tile-framing.md:170 |
| R-08-7 | Golden framing test — load a reference `.wld`, run `frame_world`/`frame_walls`, pin the frames | 08 | M | verification (medium) | R-08-4 | 08-tile-framing.md:172 |
| R-09-10 | Sweep `draw_data`'s uncovered ids: emit the C#-side id set from `TileDrawing.cs:4540-6044` and diff | 09 | S | data coverage (medium) | — | 09-tile-drawing.md:258 |
| R-03-12 | Derive the `leg_frame`/`body_frame` → C# `PlayerFrame` slot-id table | 03 | M | data coverage (medium) | framing module | 03-player-core.md:264 |
| R-05-1 | Fix `NpcKind::cs_id` (`Skeleton => 21`, `UndeadMiner => 44`, `MeteorHead => 23`) + round-trip test rows | 05 | S | behavioural (medium) | — | 05-npc-core.md:246 |
| R-10-8 | Sliced-path render test — unit-test `push_sliced` quad count/geometry against the C# `sourceRectangle` table | 10 | S | verification (medium) | R-10-5 | 10-lighting.md:178 |
| R-13-7 | LOS/HitTile unit tests — a synthetic world compared against the C# ranges | 13 | M | verification (medium) | R-13-4 | 13-collision.md:166 |
| R-14-7 | Replace the four substituted `GetTileBaseOption` cases (461, 80, 529, 530) | 14 | L | data coverage (cosmetic) | R-12-7, module 16 | 14-map-minimap.md:340 |
| R-14-8 | Byte-exact `.map` writer verification against a captured vanilla `InternalSaveMap` byte stream | 14 | L | verification (medium) | — | 14-map-minimap.md:343 |
| R-15-1 | Add a `Dust`/`Gore` gap row to `PORT_MAP.md` + an `ANALYSIS.md` "Dust & gore (module 15)" section | 15 | S | doc (prerequisite) | — | 15-dust-gore.md:256 |
| R-15-7 | Add unit tests for `foliage.rs` (the leaf falling/rest transition, the 600-tick fade) | 15 | S | verification (medium) | — | 15-dust-gore.md:279 |
| R-16-7 | Fix the docs — mark `M1.4-BG-23`, correct `M1.4-BG-21`'s "Out (M4)" note, record deviations 1/2/5/6/10 in ANALYSIS | 16 | S | doc (medium) | — | 16-weather.md:311 |
| R-18-1 | Pin the event order — a `ui/pointer.rs` test reproducing `UserInterface.cs:190-228` | 18 | S | verification (medium) | — | 18-ui-framework.md:258 |
| R-18-2 | Verify the `GoBack` dead-code claim (grep `decompiled` for `UserInterface.GoBack` call sites) | 18 | S | doc (low) | — | 18-ui-framework.md:262 |
| R-19-1 | Fix the three stale doc claims (`ANALYSIS :2120-2124`, `PORT_MAP :180`, `flow.rs:73` + `mod.rs:1482`) | 19 | S | doc (medium) | — | 19-ui-screens-frontend.md:438 |
| R-20-2 | BUG-20 row closeout (move `ROADMAP.md:1705` into the DONE shape quoting the three `bug20_*` tests) | 20 | S | doc (low) | — | 20-ui-ingame.md:215 |
| R-24-7 | Re-verify + re-word the M4-SPAWN-1/2 and M4-EVENT-1a/2 ROADMAP rows (the unported `GetSpawnRate` tail, the missing moon-event rosters, the "2-second spawn pass") | 24 | S | doc (medium) | — | 24-spawner-invasions.md:295 |
| R-24-10 | `is_ices_snow` from the generated set — move `TileID.Sets.IcesSnow` into `tools/GenTileData`'s output | 24 | S | data coverage (cosmetic) | R-08-6 | 24-spawner-invasions.md:303 |
| R-25-2 | Retire the misleading "folds into the walker params" comments (`npc.rs:4175-4179`, `:4184-4186`, `:4190-4192`, `:4364-4366`) | 25 | S | doc (medium) | — | 25-npc-ai-audit.md:391 |
| R-25-3 | Fix `docs/NPC_AI_TAXONOMY.md` (the five stale `TODO` rows) and ROADMAP §14's "🟡 3 kinds" header | 25 | S | doc (cosmetic) | — | 25-npc-ai-audit.md:394 |
| R-25-8 | A family-identity test — assert that each `NpcKind` maps to its vanilla `aiStyle` | 25 | M | verification (medium) | — | 25-npc-ai-audit.md:410 |
| R-28-8 | Re-verify the DONE rows M2-BUFF-1..8 and mark PASS/PARTIAL/FAIL per row | 28 | S | doc (medium) | — | 28-buffs-debuffs.md:269 |
| R-29-9 | Re-read `tr-core/src/housing.rs` against `WorldGen.cs:6041-6237` and record the room-check fidelity in ANALYSIS | 29 | S | verification (medium) | — | 29-town-npc-housing.md:326 |
| R-29-10 | Fix the ROADMAP citation drift on the M4-TOWN-1 row (`RoomNeeds` 5679-5730, `MoveTownNPC` 4685) | 29 | S | doc (cosmetic) | — | 29-town-npc-housing.md:329 |
| R-31-7 | Add the golf tiles (476/477/492/493/494) to `tiledata` + the `TileID.Conversion.GolfGrass` chain | 31 | S | data coverage (medium) | R-08-6 | 31-golf-ball-physics.md:217 |
| R-32-9 | Docs pass — refresh `docs/ROADMAP.md:78` and `docs/PORT_MAP.md:241` with the quantified audio coverage | 32 | S | doc (low) | — | 32-sound-audio.md:273 |
| R-34-3 | Correct the ROADMAP/ANALYSIS claims ("the duplicated dead arms removed") and the `M4-EVENT-4b` status | 34 | S | doc (high) | R-34-1 | 34-id-sets-audit.md:261 |
| R-34-12 | Add the "id census" test harness — parse the C# `Count` constants and compare them against the Rust constants | 34 | L | verification (medium) | R-34-8 | 34-id-sets-audit.md:301 |
| R-38-1 | Refresh `README.md` "What is *not* implemented yet" (drop the multiplayer + in-game-options claims) and add a "Not included by design" list | 38 | S | doc (medium, player-facing) | — | 38-out-of-scope-audit.md:215 |
| R-38-2 | Correct the ROADMAP non-goals: replace "until M6-NET is complete" with the real gap (the Steam transport/lobby/social layer), drop the "single-player-first" line, narrow the RGB path, name WeGame | 38 | S | doc (medium) | — | 38-out-of-scope-audit.md:219 |
| R-38-3 | Add an ANALYSIS subsection "Excluded subsystems (platform & services)" enumerating the eight `SocialAPI` modules one by one | 38 | S | doc (medium) | — | 38-out-of-scope-audit.md:224 |
| R-38-4 | Add PORT_MAP rows for the excluded C# paths (`Terraria/Social/**`, `Terraria/Testing/**`, `nativefiledialog.cs`, `TimeLogger.cs`, `CrashDump.cs`, `RGB/*Shader*.cs`) | 38 | S | doc (medium) | — | 38-out-of-scope-audit.md:229 |
| R-38-7 | Verify the achievements split — confirm the vanilla's *local* unlock is inside `AchievementManager`, so only the Steam push is missing | 38 | M | doc (medium) | — | 38-out-of-scope-audit.md:244 |
| R-38-9 | Do NOT schedule: Steam Workshop hub / mod browser, Steam Cloud, WeGame IPC, peripheral RGB, `Terraria.Mono`, the GameBanana browser — record them as closed non-goals | 38 | L | doc (low) | — | 38-out-of-scope-audit.md:252 |
| R-27-1 | Mount-id sets as data — add the 13 `MountID.Sets` tables and consume them in `can_mount`, `can_dismount` and `quick_mount` | 27 | S | data coverage (medium) | — | 27-mounts-pets-vehicles.md:283 |
| R-27-5 | Refresh the stale documentation — ANALYSIS "Mounts framework", ATLAS:1364 and PORT_MAP:96 from "12 rows" to the current 50, and delete the three stale in-code comments | 27 | S | doc (medium) | — | 27-mounts-pets-vehicles.md:300 |
| R-27-11 | Replace `the_twelve_ground_rows_carry_the_exact_c_constants` with a 50-row table test (C# values pasted from `Mount.cs:696-2567`) | 27 | M | verification (medium) | — | 27-mounts-pets-vehicles.md:313 |
| R-30-10 | Reconcile `ENTRIES` (172) with `BESTIARY_NPC_SHEET_IDS` (174) and add a test that every sheet slot has an entry and vice versa | 30 | S | data coverage (cosmetic) | — | 30-achievements-bestiary.md:259 |
| R-30-12 | Add a data test that pins every Rust row's condition count against `AchievementInitializer.cs` (137 rows / 180 conditions) | 30 | S | verification (medium) | R-30-1 | 30-achievements-bestiary.md:263 |
| R-39-13 | Refresh `docs/ARCHITECTURE.md` + `AGENTS.md`: the crate line counts, the `App` field count (118 → 257), `main.rs` (86 → 163), the graph node count (3 440 → 13 769) and the "god-files are gone" sentence | 39 | S | doc (high) | R-39-5, R-39-6, R-39-7 | 39-port-architecture.md:341 |
| R-35-9 | Correct the premise in the audit docs (`Item.seed`, `NPC.seed`, `NPC.netSeed`, `Utils.GetPointHash` do not exist in 1.4.5.8) | 35 | S | doc (medium) | — | 35-rng-utils.md:276 |
| R-35-10 | Fix the `spawner.rs` doc comment ("xorshift64\*" → splitmix64) and cross-link the three `next_u64` copies | 35 | S | doc (cosmetic) | R-35-1 | 35-rng-utils.md:277 |
| R-37-1 | Fix the ReLogic citations in `docs/PORT_MAP.md` (`:250`) and `INDEX.md:62` ("content pipeline, XNA, JSON") | 37 | S | doc (cosmetic) | — | 37-relogic-libs.md:288 |
| R-37-2 | Add a `tr-content` test that pins the XNB *header* of one compressed and one plain retail `.xnb` | 37 | S | verification (medium) | — | 37-relogic-libs.md:293 |
| R-37-3 | Document the ReLogic ↔ FNA ↔ Rust boundary in `docs/ANALYSIS.md` | 37 | S | doc (medium) | — | 37-relogic-libs.md:296 |
| R-40-2 | Repair the hover citations in `npc_ai.rs` (162-191) and in `docs/ROADMAP.md:1409` + a per-type table | 40 | S | doc (high) | R-40-1 | 40-port-done-verification.md:138 |
| R-40-6 | Reconcile the ROADMAP snapshot with the catalog for the M4-NPC-3c/3h..3r rows (`ROADMAP.md:73` vs `:1534-1550`) | 40 | S | doc (medium) | — | 40-port-done-verification.md:142 |
| R-40-7 | Stamp the M1.5 performance tables (`ROADMAP.md:1181-1226`) with the commit they were measured on, or mark them "historical" | 40 | S | doc (low-med) | needs a benchmark run | 40-port-done-verification.md:143 |
| R-40-9 | Cover the hover/caster/sphere/swim machines with unit tests in `crates/tr-app/src/npc_ai.rs` (today: 2) | 40 | M | verification (medium) | R-40-1 | 40-port-done-verification.md:145 |
| R-41-1 | Fix `ROADMAP.md:64`'s "2 draw calls" to the measured range-draw count + a `TR_STATS` per-range draw line | 41 | S | doc (cosmetic) | — | 41-render-pipeline.md:258 |
| R-41-2 | Add a headless GPU test that renders one quad through the sprite pipeline and asserts the exact blended output | 41 | S | verification (medium) | — | 41-render-pipeline.md:261 |
| R-42-2 | A golden-byte harness — one C#-written `.wld` + a test asserting the Rust writer's bytes are identical | 42 | M | verification (medium) | — | 42-save-load-audit.md:273 |
| R-42-6 | Close the cloud-save row as an explicit non-goal (module 38) and drop it from the Saves status line | 42 | S | doc (low) | — | 42-save-load-audit.md:285 |
| R-42-12 | Documentation — the ROADMAP "Saves" status line (`.trwld v10` → v13, the BUG-03 note) + the M6-SAVE-1 wire-bit caveat | 42 | S | doc (low) | — | 42-save-load-audit.md:305 |
| R-03-11 | Correct ROADMAP §8's stale "Rust ⬜ (none yet)" header (`ROADMAP.md:401`) | 03 | S | doc (medium) | — | 03-player-core.md:263 |
| R-12-1 | Fix the doc-level lies — the misnamed test, ANALYSIS's M5-STRUCT-5 pins, ROADMAP.md:69's world-gen row | 12 | S | doc (medium) | — | 12-worldgen.md:328 |

### (f) netcode / multiplayer — 16 seeds

| id | One-line title | Module | Size | Severity | Dependencies | Source |
| --- | --- | --- | --- | --- | --- | --- |
| R-17-8 | The per-system passes ROADMAP.md:1067-1071 parks: peer inventories, creative powers' per-player split, pylon/bestiary module bodies, the exact clock path | 17 | L | behavioural (medium) | R-17-4, R-17-5 | 17-netcode.md:318 |
| R-17-9 | Dedicated-server mode (`Main.dedServ`, `ServerMode.cs`, `NetManager`) — the port has no headless host path | 17 | L | behavioural (medium) | R-17-4 | 17-netcode.md:321 |
| R-17-4 | Port the join bundle (`SyncConnectedPlayer`/`SyncOnePlayer`/`SyncOnePlayer_ItemArray`, the 59-slot inventory push) | 17 | M | behavioural (medium) | R-17-1, a peer-inventory store | 17-netcode.md:309 |
| R-17-5 | Chest/sign/wiring round trip — ids `31-33`, `69`, `19`, `20`, `59`, `85` | 17 | M | behavioural (medium) | R-17-1 | 17-netcode.md:312 |
| R-17-6 | `18 SetTime` 1:1 clock sync (drop the internal `205`) | 17 | M | behavioural (medium) | R-17-1, the world flags in `7 WorldInfo` | 17-netcode.md:314 |
| R-17-7 | Remote-player smoothing parity — document or port the C#/ReLogic interpolation | 17 | M | pixel/behavioural (medium) | — | 17-netcode.md:316 |
| R-11-8 | MP parity — `curMaxLiquid`/`cycles` from the active-player count, the `kill >= 10 + players/3` threshold, `_netChangeSet` | 11 | L | behavioural (N/A for SP) | module 17 | 11-liquid.md:204 |
| R-06-7 | Port the `ProjectileKey` table (`slotGenerations` + `keyToIndex`, `Projectile.cs:10852-10858`) | 06 | L | determinism (MP) | — | 06-projectile-core.md:149 |
| R-28-7 | Buff net sync — the buff rows in the player-state message and the `55 PlayerBuff` message | 28 | M | behavioural (medium) | R-17-1, R-28-1 | 28-buffs-debuffs.md:266 |
| R-31-10 | Golf netcode — `NetMessage` 59 / 128 / 27 and the `Wiring.HitSwitch` cup press | 31 | S | behavioural (medium) | R-17-1 | 31-golf-ball-physics.md:224 |
| R-27-13 | Mount net sync — msg 13 (`Mount.cs:6174`, `:6224`) | 27 | L | behavioural (MP) | the MP design | 27-mounts-pets-vehicles.md:318 |
| R-30-11 | Port `NetBestiaryModule` — broadcast `SerializeKillCount` on a kill and sync the counts on player join | 30 | M | behavioural (MP) | the netcode module's shape | 30-achievements-bestiary.md:261 |
| R-17-1 | Map the ported client onto real ids (replace `201-205` with the C# ids they stand in for) | 17 | S | behavioural (wire-incompatible) | — | 17-netcode.md:300 |
| R-17-2 | Rename `WORLD_COMPLETE` (49) — confirm the semantic against `MessageBuffer.cs:2447-2453` | 17 | S | behavioural (medium) | — | 17-netcode.md:303 |
| R-17-3 | Port ping as id 154 (`Net/Ping.cs:23-35`) alongside module 2 and expose `CurrentPing` to the HUD | 17 | S | behavioural (medium) | — | 17-netcode.md:306 |
| R-03-10 | Replace the `whoAmI = 0` stand-in (`player.rs:2001-2003`) with the real index once multi-player netsession ids land | 03 | M | determinism/behavioural (medium) | R-17-1 | 03-player-core.md:262 |

### (g) UI — 23 seeds

| id | One-line title | Module | Size | Severity | Dependencies | Source |
| --- | --- | --- | --- | --- | --- | --- |
| R-19-9 | `UIWorldLoad` + `UIGenProgressBar` as a sixth `FrontScreen` (the 612×70 bar, the 570/508 px fills, the progress feed, the abort cue) | 19 | L | cosmetic+behavioural (medium) | R-12-6 | 19-ui-screens-frontend.md:478 |
| R-19-6 | The creator's Randomize/Copy/Paste arms (`Click_RandomizePlayer` + the clipboard pair) behind a clipboard bridge | 19 | M | behavioural (feature absent) | R-19-5 | 19-ui-screens-frontend.md:461 |
| R-14-5 | The eight map overlay layers (`Map/*Layer*.cs`, `MapIconOverlay.cs`, `Main.DrawNPCMapIcons`, `Lang.BuildMapAtlas`) | 14 | M | behavioural (medium) | R-17-1 | 14-map-minimap.md:332 |
| R-14-6 | Fullscreen map input parity — `mapStyle` cycling (0/1/2), the `mapOverlayScale` map, zoom key/wheel, `DebugOptions.unlockMap` | 14 | M | behavioural (medium) | module 18 | 14-map-minimap.md:336 |
| R-18-8 | Screens with no counterpart — `UIManageControls` and `UICreativePowersMenu` (the `UIWorkshop*`/`UIResourcePack*` set stays scoped out) | 18 | L | UI (medium) | — | 18-ui-framework.md:281 |
| R-07-5 | Finish the tooltip pass to parity with `GetLinesInfo` (per-item `ToolTip`, the player-side layers, `GetHoverName`'s stack suffix) | 07 | L | cosmetic/pixel (medium) | R-07-2 | 07-item-core.md:315 |
| R-18-7 | `UIWorldCreationAdvanced` + the secret-seeds list as a `ui/world_creation_advanced.rs` immediate port | 18 | M | UI (medium) | R-18-4 | 18-ui-framework.md:278 |
| R-31-9 | Port `FancyGolfPredictionLine` (20 simulated steps, tile 379 temporarily non-solid) | 31 | M | UI (medium) | R-31-1, R-31-2 | 31-golf-ball-physics.md:222 |
| R-20-9 | Equipment/dye page parity (M7-UI-3 half): the dye drag, capture mode, the housing `SetMouseNPC` click | 20 | L | UI (medium) | R-20-3 | 20-ui-ingame.md:235 |
| R-20-5 | Crafting UI depth — the select-vs-craft split, Shift ×10 craft-all, the filter/search hit tests, the guide item | 20 | M | behavioural (medium) | — | 20-ui-ingame.md:224 |
| R-29-5 | Port the happiness report — the `TownNPCMood_*` localisation keys, `NPCInteractions.ReportHappiness`, the shop-hud icon/offsets | 29 | M | behavioural/cosmetic (medium) | M6-LOCALIZE | 29-town-npc-housing.md:313 |
| R-28-3 | The buff HUD — `Main.DrawBuffIcon` + `Main.TryRemovingBuff` (the atlas rows, the `buffAlpha` fade, the duration text, the right-click removal) | 28 | M | behavioural+pixel (high) | R-28-1, R-28-2 | 28-buffs-debuffs.md:249 |
| R-37-4 | Port `ReLogic.Text.WrappedTextBuilder` + `StringReaderWrapExtension` into the Rust UI text path | 37 | M | behavioural (UI) | modules 18/20 final shape | 37-relogic-libs.md:301 |
| R-31-8 | Port the golf UI — `DrawInterface_Resources_GolfPower` and `DrawInterface_GolfBallIndicator` | 31 | M | UI (medium) | R-31-4 | 31-golf-ball-physics.md:219 |
| R-19-7 | `GameTipsDisplay` — the rotating hint line under the creator panel and the `UIWorldLoad` tip | 19 | M | cosmetic (low) | — | 19-ui-screens-frontend.md:466 |
| R-19-8 | Random world-name composition — the `RandomWorldName_*` tables + the 1-in-10000 `TheConstant` roll | 19 | M | cosmetic (low) | R-35-1 | 19-ui-screens-frontend.md:470 |
| R-04-5 | `PlayerDeathReason` — a compact death-cause enum routed through `hurt`/`kill` so the message table covers every cause | 04 | M | cosmetic (UI) | — | 04-player-combat.md:200 |
| R-14-3 | Minimap alpha config — a `MapMinimapAlpha` key in `settings.rs`, clamped 0.01..1 | 14 | S | cosmetic (low) | — | 14-map-minimap.md:325 |
| R-07-2 | Add the tooltip's missing gates (`heal_life`/`heal_mana`, `quest_item`, `vanity`, `tile_wand`, the fishing-pole pair) | 07 | S | cosmetic (medium) | R-07-6 | 07-item-core.md:302 |
| R-19-11 | Cloud save UI — the cloud favourite buttons and the `IsCloudSave`/`MoveToCloud` arms | 19 | L | UI (low) | — | 19-ui-screens-frontend.md:490 |
| R-30-8 | Bestiary UI parity — the filter list (`Filters.cs`), the sort steps (`SortingSteps.cs`), the flavour-text/rarity-star/spawn-condition info elements | 30 | M | UI (medium) | R-30-6 | 30-achievements-bestiary.md:254 |
| R-30-9 | Add the achievement unlock toast and fire `OnCompleted` for every completed achievement (not only the last) | 30 | S | UI (medium) | R-30-2 | 30-achievements-bestiary.md:257 |
| R-36-1 | Verify/port the F11 UI-hide toggle (`Main.cs:2085`, `:18402-18424`) and the `Main.hideUI` state the films need | 36 | S | UI (cosmetic) | — | 36-cinematics-film.md:171 |

### (h) audio — 8 seeds

| id | One-line title | Module | Size | Severity | Dependencies | Source |
| --- | --- | --- | --- | --- | --- | --- |
| R-32-6 | Port the tracked-sound layer (`ActiveSound`/`SlotId`/`SoundPlayer`): the 4096-slot pool, the 10⁷-px² cull, `MaxTrackedInstances`, the `SoundType` slider routing | 32 | M | behavioural (medium) | — | 32-sound-audio.md:261 |
| R-32-4 | Extend the cue coverage in waves by C# call-site count (footsteps/run, tink, NPC-hit/killed, splash/drip, grass/shatter, chat, camera, drown/max-mana) | 32 | M | scope (medium) | R-32-1 | 32-sound-audio.md:254 |
| R-32-3 | Model the sound instance pool — a per-family "stop the previous instance" map with the stop-then-recreate semantics | 32 | M | behavioural (medium) | — | 32-sound-audio.md:251 |
| R-32-7 | Music polish — the `PlayCallbacks`/`TrackLoopCounts` hook, the `IsPlaying or not IsStopped` nuance, the boss→slot map, the `.mp3`/`.ogg`/`.wav` loader | 32 | M | behavioural (medium) | — | 32-sound-audio.md:266 |
| R-32-5 | Add the `PlaySound` guards (`isGeneratingOrLoadingWorld`, `netMode == 2`) + focus pause/resume + `StopAmbientSounds` | 32 | M | behavioural (medium) | — | 32-sound-audio.md:258 |
| R-32-2 | Add pitch to the playback path (a `Pitch` source or a per-cue pre-shift), starting with the `case 43` thunder family | 32 | S | behavioural (medium) | — | 32-sound-audio.md:248 |
| R-32-8 | Wire the two orphaned emitters (the critter cue table, the liquid-merge cue table) | 32 | S | scope (low) | module 25, R-11-5 | 32-sound-audio.md:270 |
| R-31-11 | Golf chat/effects — `Game.BallBounceResultGolf_Single/Plural` text and `EmitGolfballExplosion` | 31 | S | audio (low) | the localization port | 31-golf-ball-physics.md:226 |

### (i) architecture & concurrency prep — 32 seeds

| id | One-line title | Module | Size | Severity | Dependencies | Source |
| --- | --- | --- | --- | --- | --- | --- |
| R-39-1 | Split `World` into `WorldTiles` (immutable geometry) and `WorldState` (the ~50 runtime/event flags) | 39 | L | structural (high) | — | 39-port-architecture.md:329 |
| R-39-3 | Introduce a typed event buffer (`enum SimEvent { SpawnNpc, SpawnProjectile, Damage, DropItem, BreakTile, Sound, NetSync }`) with per-thread buffers + a deterministic merge | 39 | L | structural (high) | R-39-2 | 39-port-architecture.md:331 |
| R-39-4 | Build the spatial hash over stable entity ids (a monotonic `entity_id` so `Vec` compaction cannot move an entity) + replace the 15 linear NPC scans | 39 | L | structural (high) | R-39-3 | 39-port-architecture.md:332 |
| R-39-5 | Split `tr-app/src/npc.rs` (17 134) into `npc/{entity,stats,kinds,ai,buffs}.rs` | 39 | L | structural (medium) | — | 39-port-architecture.md:333 |
| R-39-6 | Split `tr-app/src/projectile.rs` (17 411) into `projectile/{entity,data,ai,damage}.rs` | 39 | L | structural (medium) | — | 39-port-architecture.md:334 |
| R-39-7 | Split `tr-app/src/player.rs` (10 098) into `player/{entity,physics,buffs,inventory,draw_state}.rs` (`Player` has 239 fields) | 39 | L | structural (medium) | — | 39-port-architecture.md:335 |
| R-12-6 | Verifiable world parity harness — re-implement `WorldGenSnapshot`/`WorldManifest` end-to-end + a `--worldgen-audit` dev flag | 12 | L | architecture (high) | R-12-2 | 12-worldgen.md:355 |
| R-42-8 | The compressor seam — a `Compression` switch in `wld::to_bytes` that deflates the tile-section body and bumps `FORMAT_VERSION` to 327 | 42 | M | architecture (Stage-2) | R-42-2 | 42-save-load-audit.md:291 |
| R-09-6 | Reintroduce the deferred `_specialTilesX/Y` registry (`TileCounterType`, `AddSpecialPoint`) so tiles draw in C#'s order | 09 | L | architecture/z-order (high) | — | 09-tile-drawing.md:247 |
| R-37-7 | An `AssetRepository` equivalent in Rust (name → lazy load, `AssetRequestMode`, resource-pack overlays) | 37 | L | architecture (feature gap) | a roadmap decision (no mod support) | 37-relogic-libs.md:314 |
| R-39-2 | Extract the fixed-step order out of `app/frame.rs` into a data-driven tick plan (`[(system, phase)]`) + a test that pins the C# order (`Main.DoUpdateInWorld_Inner` `Main.cs:17749-17821`) | 39 | M | structural (high) | R-39-1 | 39-port-architecture.md:330 |
| R-39-8 | Split `tr-core/src/edit.rs` (6 791) into `edit/{kill,place,actuation,objects}.rs` and merge the app-side `world_sim/editing.rs` wrappers onto the core seam | 39 | M | structural (medium) | R-39-5 | 39-port-architecture.md:336 |
| R-39-11 | Move the read-only `scene_scan` scan from `tr-app/world_sim/scene_scan.rs` into `tr-core::scene` | 39 | M | structural (medium) | R-39-1 | 39-port-architecture.md:339 |
| R-36-11 | The OOA's persistence — `DD2Event.Save`/`Load` (`DD2Event.cs:151-170`), the three downed-tier flags in the world save | 36 | M | determinism/persistence (medium) | module 42 | 36-cinematics-film.md:199 |
| R-38-5 | Decide the `Terraria/Testing` harness: record the exclusion in ROADMAP non-goals + ANALYSIS, or port a Rust equivalent of `RecordReplay`/`StateSnapshot` | 38 | M | structural (medium) | — | 38-out-of-scope-audit.md:233 |
| R-38-6 | Decide and document the dedicated-server story (host-in-window only vs a headless `-server` mode) | 38 | M | structural (medium) | R-38-1 | 38-out-of-scope-audit.md:239 |
| R-39-10 | Re-home the duplicated helpers (`hsl_to_rgb`, `move_towards`, `update_rain` → one owner each in `tr-core`; `collision_empty_tile` → the app-side `collision.rs`) | 39 | S | structural (low) | — | 39-port-architecture.md:338 |
| R-39-12 | Replace `World.extras: HashMap<u32,u8>` with an ordered structure (or forbid iteration) to remove the nondeterministic-iteration hazard | 39 | S | determinism (medium) | R-39-1 | 39-port-architecture.md:340 |
| R-39-14 | Add an architecture lint: fail the build when a hand-written file exceeds 5 000 lines or when a `pub fn` name exists in both `tr-core` and `tr-app` | 39 | S | structural (medium) | — | 39-port-architecture.md:342 |
| R-38-8 | Optional: a trait-shaped `SocialApi` seam in Rust (a `NoSocial` impl behind the C# method names) | 38 | L | structural (low) | the owner's choice | 38-out-of-scope-audit.md:248 |
| R-19-10 | `.trwld` metadata fields — GameMode, evil, creation time, play time, Moon-Lord/seed flags in the `tr_core::save` header | 19 | L | architecture (medium) | module 42 | 19-ui-screens-frontend.md:485 |
| R-12-2 | Make the config view real for one pass — thread `pass_config` into `caves.rs::ores_and_shinies` and delete the hard-coded constants | 12 | S | architecture (medium) | — | 12-worldgen.md:333 |
| R-18-9 | Layer stack — a named ordered layer list (`GameInterfaceLayer` analogue) in `app/frame.rs` with `InterfaceScaleType` spaces | 18 | L | architecture (medium) | — | 18-ui-framework.md:284 |
| R-42-9 | The RLE/VLQ predictor stage over the already-RLE'd tile stream (column-wise delta + VLQ) before deflate | 42 | S | architecture (Stage-2) | R-42-8 | 42-save-load-audit.md:296 |
| R-42-10 | The `.plr` hook — compress the plaintext before `cbc_encrypt`, decompress after `cbc_decrypt` | 42 | S | architecture (Stage-2) | R-42-4 | 42-save-load-audit.md:298 |
| R-42-11 | Background save — move `wld::save`'s `std::fs::write` onto a worker thread (serialization stays on the main thread) | 42 | S | architecture/concurrency (low) | — | 42-save-load-audit.md:302 |
| R-40-8 | Add a "cited-range" lint to the doc toolchain (parse `File.cs:NNNN-MMM` anchors and assert the range contains the quoted symbol) | 40 | M | architecture/tooling (medium) | — | 40-port-done-verification.md:144 |
| R-41-6 | Write down the draw-order contract — a golden list of the ranges and the vanilla buckets they correspond to | 41 | M | architecture (medium) | — | 41-render-pipeline.md:280 |
| R-13-5 | Generalise `tile_collision`/`slope_collision` signatures — expose `gravDir`, `ignore_doors`, `ignore_aetherium_platforms`, `special_checks_mode` | 13 | S | architecture (low) | — | 13-collision.md:160 |
| R-37-5 | Make the font parser's structural discovery explicit — document the page-header offsets and pin them with a test | 37 | M | architecture (medium) | — | 37-relogic-libs.md:309 |
| R-07-4 | Port `Clone`/`Refresh` as an instance snapshot (if and when a system needs in-place item mutation) | 07 | M | architecture (structural) | — | 07-item-core.md:311 |
| R-08-8 | Perf hardening (behaviour-frozen) — replace `WireState`'s four `HashSet`s with an index-keyed bitset | 08 | S | architecture/perf (low) | — | 08-tile-framing.md:175 |

---

## 3. Consolidated risk register

Every deviation/risks row that is **not already admitted** in `docs/ANALYSIS.md` or
`docs/ROADMAP.md`, honouring the admission flags written in the source research files.
276 rows.

Legend for *status*: **CLAIM-OTHERWISE** = the docs assert the opposite (or mark the
row `✅`) → this is also a §4 row. **SILENT** = the docs do not mention the gap at all.

### 01 — main update loop (8)

| Source | Severity | One-line risk | Claim vs reality | Status |
| --- | --- | --- | --- | --- |
| 01-1 | behavioural | The clock advances while a menu/front screen is open | Claim: (silent) · Reality: `advance_time` runs at `frame.rs:160-189`, *outside* the `menu.is_none() && front.is_none()` gate (328); C# reaches `UpdateTime` only through `DoUpdateInWorld` (17278-17281), which the `gameMenu` early return skips (17166-17175) | SILENT |
| 01-2 | behavioural/sub-pixel | The camera follow runs in the fixed step, not the draw pass | Claim: ANALYSIS admits the interpolation intent (frame.rs:572-575) · Reality: the port lerps toward `p.center()` with `1 - exp(-dt*12)` inside the player fixed step; C# positions the camera in `DoDraw_UpdateCameraPosition` (`Main.cs:62590-62649`) with `cameraLerp`/`cameraX`/`cameraY`/`ClampScreenPositionToWorld` (63948) absent | SILENT |
| 01-3 | determinism | Items are updated after projectiles | Claim: (silent) · Reality: C# `17977` before `18031`; the port runs items after ambience, projectiles after items | SILENT |
| 01-4 | behavioural | `UpdateTime` runs before the entity pass | Claim: (silent) · Reality: C# 18083-18097 after dust/floating text; the port advances the clock at the top of `frame` → anything reading `Main.time` sees a value one phase ahead | SILENT |
| 01-5 | behavioural | No `max_steps` bottleneck and no frame suppression | Claim: (silent) · Reality: the dt clamp `[0, 0.1]` silently turns C# `17006-17020`'s `UpdateTimeAccumulator`/draw-skip policy into "drop at most 6 steps" | SILENT |
| 01-6 | behavioural | No pause semantics | Claim: (silent) · Reality: with `inventory_open` the world keeps simulating; C# sets `gamePaused` (17194-17201) and runs only `DoUpdate_WhilePaused` — single-player diverges from the default vanilla behaviour | SILENT |
| 01-7 | cosmetic | Zoom accelerators and the 1.0-2.0 range | Claim: (silent) · Reality: `UpdateViewZoomKeys` (16801-16815) is not ported and `TR_ZOOM` allows 0.25-8.0 vs the vanilla clamp at 61999 | SILENT |
| 01-8 | cosmetic/performance | `gfxQuality` budgets absent | Claim: (silent) · Reality: C# 17054-17113 (`maxDustToDraw`, `Liquid.cycles`, `Liquid.curMaxLiquid`, `LegacyLighting.RenderPhases`) has no port; `settings.quality` is a menu toggle only | SILENT |

### 02 — main draw pipeline (8)

| Source | Severity | One-line risk | Claim vs reality | Status |
| --- | --- | --- | --- | --- |
| 02-1 | behavioural, pixel | No off-screen target family | Claim: (silent) · Reality: the vanilla caches wall/tile/tile2/background/backWater/water in `WorldSceneLayerTarget`s (`Main.cs:63333`, `62213`, `62196`, `62308`); the port has one world range + immediate liquids → likely divergence at zoom ≠ 1.0 and on the partial-offscreen double-buffer swap (`Main.rs:57311-57325`) | SILENT |
| 02-2 | pixel | `skyTarget` + `Filters.Scene.CanCapture` gating absent | Claim: (silent) · Reality: vanilla clears and renders `skyTarget` only when a scene filter can capture (62021-62027, 62329-62332); the port always draws sky into the main range | SILENT |
| 02-3 | pixel, medium-high | The phase order is not verified 1:1 | Claim: the ROADMAP row for entity ordering "cannot be marked DONE on this evidence" · Reality: C# 62249-62326 interleaves projectiles/players/particles/NPC buckets; the Rust order is `draw_tiles` → rain/sand/sparks → leaves → wires → player, with no test or doc asserting equality | CLAIM-OTHERWISE |
| 02-4 | cosmetic, user-visible | The full-screen effect set is missing | Claim: (silent) · Reality: `ScreenDarkness`, `ScreenObstruction`, `MoonlordDeathDrama`, `PaladinsShield`, `InfernoRings`, lens flare and `DrawLensFlare` (`Main.cs:62316-62326`) are absent → blood-moon/hell/eclipse darkness and boss drama never appear | SILENT |
| 02-5 | behavioural | Capture mode is missing | Claim: (silent) · Reality: no `CaptureManager`; anything keyed on `IsCapturing`/`SceneMetrics` suppression (`Main.cs:62518`) cannot work | SILENT |
| 02-6 | behavioural | The zoom clamp is unverified | Claim: policy requires 1.0-2.0 and `Main.ForcedMinimumZoom` semantics (`Main.cs:61990-61999`) · Reality: only `self.zoom` consumers were found, no clamp site → the M2-DRAW zoom row cannot be marked DONE | CLAIM-OTHERWISE |
| 02-7 | behavioural, minor | The `drawToScreen`/`UpdateEveryFrame` branch is missing | Claim: (silent) · Reality: vanilla can bypass the target cache entirely (`Main.cs:61851-61862`, `63326`); the port always uses the cache — equivalent only while the light mode is static | SILENT |
| 02-8 | pixel | `DrawData`-driven tile overlay gaps | Claim: (silent) · Reality: hit-tile/hit-replace animations (`Main.cs:63348-63349`), tile cracks (61533) and `DrawTileInWater` (57536) are not represented | SILENT |

### 03 — player core (9)

| Source | Severity | One-line risk | Claim vs reality | Status |
| --- | --- | --- | --- | --- |
| 03-1 (critical) | behavioural | In-liquid statics use the wrong formula port-wide | Claim: (silent) · Reality: `player.rs:3190-3198` applies `GRAVITY/3 = 0.1333` and `MAX_FALL/3 = 3.333` to *every* liquid; C# rebases gravity/maxFallSpeed/jumpHeight/jumpSpeed per liquid at `Player.cs:24382-24437` (water 0.2/5.01/30/6.01, honey 0.1/3.01, merman 0.3/7.01, trident 0.25/6.01, shimmer 0.15) → a 2× jump-height error in liquids | SILENT |
| 03-2 | behavioural | `MAX_FALL` misses `+0.01f` | Claim: (silent) · Reality: `player.rs:30` `MAX_FALL = 10.0` vs `Player.cs:24437` `this.maxFallSpeed += 0.01f` (and `PortalPhysicsEnabled` → 35 at :24380) — per-tick drift on every fall | SILENT |
| 03-3 | behavioural | Drowning damage ignores iframes | Claim: (silent) · Reality: `player.rs:2375` forces `hurt_cd = max(…, 0.8)` (48 ticks); C# drowning is `statLife -= 2` on its own `breathCD` clock (`Player.cs:23263`) with **no** `immuneTime` set | SILENT |
| 03-4 | behavioural | Respawn life & spawn iframes are wrong | Claim: (silent) · Reality: `player.rs:1820` `self.hp = self.max_hp` vs C# `statLife = max(statLifeMax, statLifeMax2)/2` (`Player.cs:37596-37611`); `player.rs:1821` `hurt_cd = 0.0` vs `immuneTime = 180`/60 (`Player.cs:37661-37668`) → players spawn at full health and can be hit on respawn frame 1 | SILENT |
| 03-5 | behavioural | Expert death scaling is unreachable | Claim: (silent) · Reality: `world_sim/npcs.rs:3180` hardcodes `expert = false`, so `GetRespawnTime`'s ×1.5 (`Player.cs:39060-39082`) never runs | SILENT |
| 03-6 | behavioural | The respawn-skip refusal list is incomplete | Claim: (silent) · Reality: `allows_respawn_timer_skip` (`player.rs:1901-1930`) omits the `Main.snowMoon ‖ Main.pumpkinMoon ‖ DD2Event.Ongoing` refusal at `Player.cs:38984-38987` → the skip is granted during invasions where C# forbids it | SILENT |
| 03-7 | determinism/pixel | `whoAmI` is a stand-in | Claim: (silent) · Reality: `player.rs:2001-2003` uses `0`; `leg_frame`/`body_frame` (`player.rs:504`, :507) are frame-row indices rather than C#'s `PlayerFrame` slot ids | SILENT |
| 03-8 | doc | ROADMAP §8 is stale | Claim: `ROADMAP.md:401` reads "**Rust** ⬜ (none yet) · ⬜" · Reality: `regen.rs` (18 tests), `buffs.rs` (~40 tests) and `luck.rs` exist; the task rows below the header (`✅ M2-BUFF-2 regen engines`) contradict it | CLAIM-OTHERWISE |
| 03-9 (risk) | behavioural | The phase order is 1:1 today but two moves are load-bearing | Claim: (silent) · Reality: the wet-flag derivation happens *before* the movement defaults, so the port's wet statics are one tick fresher than C#'s; any fix to the statics must keep that ordering property | SILENT |

### 04 — player combat (9)

| Source | Severity | One-line risk | Claim vs reality | Status |
| --- | --- | --- | --- | --- |
| 04-1 | behavioural + determinism | The projectile→NPC hit skips `DamageVar` and defence | Claim: the M4-DAMAGE row excuses only the *contact* hit · Reality: every bow/gun/magic shot does its raw fused damage with no ±15 % variance and no defence halving (`Projectile.cs:12938` vs `projectile.rs:9703`/`npc.rs:4062`); the crit is carried but the `strike_npc_damage` multiply happens nowhere on this path | CLAIM-OTHERWISE |
| 04-2 | behavioural | The `num2 < 1 → 1` floor is missing | Claim: `player.rs:6093-6097` asserts the divergent behaviour as intended · Reality: `Player.cs:38378-38381` floors it to 1 and the iframes are then 20, not 40 — the port's own docs disagree with the source | CLAIM-OTHERWISE |
| 04-3 | behavioural | `GetWeaponDamageMultiplier`'s ranged split is missing | Claim: M2-COMBAT-2 is `✅` · Reality: arrows never roll `bowEffectiveDamage`, bullets never `gunEffectiveDamage`, specialists never `specialistEffectiveDamage` (`Player.cs:53396-53407`) → all ranged damage is off | CLAIM-OTHERWISE |
| 04-4 | behavioural | There is no dodge at all | Claim: (silent) · Reality: blackBelt/shadowDodge/brain/mystic sash are a 10 %-or-always miss that never happens (`:38214-38236`); the whole accessory wiring is absent | SILENT |
| 04-5 | cosmetic (UI) | There is no `PlayerDeathReason` | Claim: (silent) · Reality: the death text only exists for `DotDeath` (`death.rs:250`) → `Hurt`-caused deaths lose their cause string | SILENT |
| 04-6 | behavioural | No retaliate / paladin / beetle / solar arms in `Hurt` | Claim: (silent) · Reality: all six defensive set families are inert on the inbound path (`Player.cs:38305-38516`) | SILENT |
| 04-7 | cosmetic | No combat text, no damage tracker, no hurt sound/eye blink/dust | Claim: (silent) · Reality: `Player.cs:38369-38377`, `:38382-38384`, `:38396-38400`, `:38525-38553` are absent | SILENT |
| 04-8 | behavioural | NPC→player contact has no `DamageVar(npcdmg, -luck)` | Claim: (silent) · Reality: `npc.rs:12326-12334` passes a flat `contact_damage()`; no difficulty `damageMultiplier`, and the town-NPC gate is a kind list rather than the friendly-AI check | SILENT |
| 04-9 | behavioural (narrow) | The `Crit` parameter is dropped end-to-end | Claim: (silent) · Reality: the C# doubles only the raw `num` (fed to `magicCuffs` and PVP); the port cannot express it at all | SILENT |

### 05 — NPC core (9)

| Source | Severity | One-line risk | Claim vs reality | Status |
| --- | --- | --- | --- | --- |
| 05-1 | behavioural | `NpcKind::cs_id` has three wrong vanilla ids | Claim: (silent) · Reality: `npc.rs:1823/1824/1831` give `Skeleton => 77`, `UndeadMiner => 23`, `MeteorHead => 5`; the C# values are 21/44/23 (`NPCID.cs:10719`, `:10765`, `:10723`) — 77 is `ArmoredSkeleton` and 5 is `ServantofCthulhu`; the file's own doc comment (`npc.rs:1840-1847`) says "21 Skeleton / 44 UndeadMiner" | SILENT |
| 05-2 | behavioural | Projectile damage ignores NPC defense | Claim: (silent) · Reality: `projectile.rs::damage_pve` (9460) → `hurt_from_projectile` (9599) with the raw damage; `strike_npc_damage` is never reached and `checkArmorPenetration` only runs for the melee Flymeal arm — C# `Projectile.cs:13630-13649` does both → every ranged hit over-damages by roughly `defense/2` | SILENT |
| 05-3 | cosmetic (id-space trap) | `to_u16`/`from_u16` carry 25 unreachable dead arms | Claim: the code comment claims "the unreachable dead copies are gone" · Reality: `npc.rs:881-886` is false — `to_u16` still carries 25 dead arms (953-977, 987-990) and `from_u16` repeats the same 25 (1207-1226) | CLAIM-OTHERWISE |
| 05-4 | behavioural | The goblin fighters' AI arm is shadowed | Claim: (silent) · Reality: `update_rng` runs `GoblinPeon/GoblinThief/GoblinWarrior => update_goblin` at `npc.rs:4163-4165` *before* the intended `update_skeleton` at 4180-4183 → the goblin army walks at the zombie's speed (1.0 vs 0.8) | SILENT |
| 05-5 | structural/behavioural | `aiStyle` dispatch coverage is by-kind, not by-style | Claim: documented only implicitly by the per-kind `npc.rs` comments · Reality: 74 of the 128 `aiStyle` arms have no Rust reference; a kind added later gets a hand-written machine instead of its family default | SILENT |
| 05-6 | behavioural/determinism | The `GetAvailableNPCSlot` slot protocol is missing | Claim: (silent) · Reality: no `spawn_slot_protected`, no `SearchSpawnSlotsInReverse`, no `CannotSpawnInSlot0`; the port uses a grow-only vec capped at 200 → net sync indexes by vec position rather than by the stable slot | SILENT |
| 05-7 | behavioural | `UpdateNPC` phases are absent | Claim: (silent) · Reality: `CheckActive`/`timeLeft` despawn (78000), `CheckDrowning` (94897), `CheckLifeRegen`, `GetHurtByOtherNPCs`, `UpdateCollision`/`SwitchTiles`, `UpdateNetworkCode` are all missing; the port compensates with ad-hoc `retain` rules | SILENT |
| 05-8 | behavioural/drift risk | `NewNPC`'s spawn capsule is not a single code path | Claim: (silent) · Reality: `spawnSlotProtected`, `timeLeft`, the `wet` check, `ai[0..3]`, `target` and `OnSpawn` are re-derived (or dropped) at each of the ~20 spawn sites | SILENT |
| 05-9 | behavioural | `Npc::hurt` uses `hurt_cd = 0.18` as the melee cooldown stand-in | Claim: (silent) · Reality: C# uses a 10-tick `NPC.immune` window; `hurt_from_projectile` bypasses the cooldown entirely and the per-owner `immune[256]` array is read only by `projectile.rs` (9536) | SILENT |

### 06 — projectile core (5)

| Source | Severity | One-line risk | Claim vs reality | Status |
| --- | --- | --- | --- | --- |
| 06-1 | behavioural | `timeLeft--`/`penetrate == 0` run once per update, not once per sub-step | Claim: (silent) · Reality: `projectile.rs:9079-9090` vs `Projectile.cs:17201/17206` inside the `while` at 16676 → every `extraUpdates > 0` kind lives ~2× longer than vanilla | SILENT |
| 06-2 | determinism (maintenance) | Duplicate unreachable match arms for aiStyles 1/3/14/99/115 | Claim: (silent) · Reality: 8624+8728, 8631+8729, 8695+8730, 8777+8983, 8732+8925 — dead code, two copies of the yoyo/flail dispatch bodies to keep in sync | SILENT |
| 06-3 | behavioural | Plain `spawn` recycles by lowest `timeLeft` without the `!netImportant` filter | Claim: (silent) · Reality: `FindOldestProjectile` (`Projectile.cs:10814`) refuses net-important slots; the port can evict a net-important shot | SILENT |
| 06-4 | pixel | There is no `oldPos`/`oldRot`/`oldSpriteDirection` | Claim: (silent) · Reality: `SetDefaults` sizes them from `TrailCacheLength` (718-735) and `Update` shifts them (17170-17195) → no trails, no `TrailingMode[5]` | SILENT |
| 06-5 | pixel | No `Projectile.light` propagation | Claim: (silent) · Reality: the vanilla's per-shot `light` (853/856/944) never reaches the renderer; `draw` takes `&LightMap` for tinting only (6839) | SILENT |

### 07 — item core (3)

| Source | Severity | One-line risk | Claim vs reality | Status |
| --- | --- | --- | --- | --- |
| 07-1 | behavioural, high | The pickup path adds the transfer twice | Claim: (silent; PORT_MAP's `Item.cs` row is "partial" without the reason) · Reality: `world_sim\items.rs:191-194` adds `remaining` **and** `drop.count` to the slot → a pickup of N grants 2N; every tile drop, mob drop and shop grant is doubled | SILENT |
| 07-2 | determinism, medium | The prefix RNG is a stand-in | Claim: (silent) · Reality: `prefix.rs::next_u64`/`next` `:434-453` is a splitmix stream shared with `rain.rs`, not `UnifiedRandom`; the draw order matches but the concrete stream differs → a vanilla-replay of prefix rolls cannot match | SILENT |
| 07-3 | behavioural, medium (structural) | There is no instance-level item record | Claim: (silent) · Reality: no `Clone`/`Refresh`/`favorited`/`uniqueStack` state beyond `ItemSlot`/`ItemDrop`; any system that mutates an instance in place (painted items, the Angler's quest state) has no home | SILENT |

### 08 — tile data & framing (7)

| Source | Severity | One-line risk | Claim vs reality | Status |
| --- | --- | --- | --- | --- |
| 08-1 | determinism/pixel | `frameNumber` is a hash, not the stored/RNG value | Claim: the code comment only · Reality: C# stores per-tile `frameNumber`/`wallFrameNumber` in the save (`Tile.cs:464-482`) and re-rolls them from `WorldGen.genRand` (`Framing.cs:319`, `:390-395`); the port substitutes an avalanche hash → a port of an original world reframes with a different variant pattern than the original binary | SILENT |
| 08-2 | behavioural | `Framing.SelfFrame8Way` is not ported | Claim: (silent) · Reality: its only caller is `WorldGen.cs:84833` for ids 255-268 → those tile ids get the generic merge path in the port | SILENT |
| 08-3 | behavioural/pixel | `WallFrame` is incomplete | Claim: (silent) · Reality: the `TruncatesWalls[tile.type]` condition (`Framing.cs:348/356/364/372`), the `invisibleWall()` gate (`:344`), the `wallLargeFrames == 1/2` phlebas/lazure arms (`:378-387`), the wall-21 third-variant roll (`:391-394`) and the `wall == 0` reset (`:337-342`) are all missing | SILENT |
| 08-4 | cosmetic/structural | Wiring + coatings moved out of `Tile` | Claim: (silent) · Reality: `WireState` uses four `HashSet<(i32,i32)>` (`wiring.rs:91-96`) and `World::extras` is a sparse `HashMap<u32,u8>` (`world.rs:20`) vs the C#'s in-tile bits | SILENT |
| 08-5 | cosmetic | `reframe_area` is a square, not the C# 3×3 ring | Claim: (silent) · Reality: `WorldGen.SquareTileFrame` frames exactly the 3×3 block (`WorldGen.cs:80185-80209`); `reframe_area` clamps the radius to 1..4 and callers pass 2/3 → superset behaviour, extra work per edit | SILENT |
| 08-6 | data gap | `tiledata` set coverage is 46 of 100+ | Claim: (silent) · Reality: 46 generated `TileSet`s vs 152 `Factory.CreateBoolSet(` call sites in `TileID.cs`; sets needed outside framing are hand-copied (`tiledata/mod.rs` names `wind.rs`'s `SwaysInWindBasic`) → duplication drift risk | SILENT |
| 08-7 | verification gap | The parallel-framing "outputs identical" claim is unproven | Claim: `ROADMAP.md:1217` "outputs identical" · Reality: the comment's premise holds for the merge path, but the hash `variant()` is position-only and **no golden test** proves equality against the C# frames | CLAIM-OTHERWISE |

### 09 — TileDrawing / DrawData (8)

| Source | Severity | One-line risk | Claim vs reality | Status |
| --- | --- | --- | --- | --- |
| 09-1 | behavioural/z-order | Row-major fusion of the two layer passes | Claim: ROADMAP acknowledges nothing for the tile layer split · Reality: C# runs `Draw(solid:true)` then `Draw(solid:false)`, each with its own `SetLayer` stream; the port interleaves (`tiles.rs:427-435` documents the waterfall-order approximation) | SILENT |
| 09-2 | pixel | `GetTileDrawData` covers ≈113 of ≈376 case ids | Claim: documented per-arm in `drawdata.rs` only · Reality: every uncovered kind with a non-16×16 frame or an animation-row stride silently draws the default rect (e.g. tiles whose `addFrY = tileFrame*38`) → a lamp/candle/banner family can be panned or squashed; the uncovered set is not enumerated in the docs | SILENT |
| 09-3 | behavioural | The chest frame rewrite is missing | Claim: (silent) · Reality: `Main.chest[i].frame` (+0/+38/+76, `tileHeight=18`) never reaches the source rect → opening chests do not animate and the bottom row is 16 px tall instead of 18 | SILENT |
| 09-4 | pixel | Platforms: the martian-conduit glow rows and the slope side-fill quads are missing | Claim: (silent) · Reality: `SET_PLATFORMS` is generated but unused by the renderer (`tiles.rs:1156` only uses it inside `solid_full`) | SILENT |
| 09-5 | pixel/determinism | Paint is shader-side, the C# bakes a render-target cache | Claim: `TILE-DRAW-4` is marked `✅` · Reality: `TilePaintSystemV2.RequestTile`/`PrepareAllRequests`/`TryGetTileAndRequestIfNotReady` (`:369-541`) produce a recolored *copy* per `(type, style, paint)`; the port recolours per pixel in `sprite.wgsl::paint_recolor` `:158-169` from the `PaintID` sRGB table `:222-226`, and `sprite.wgsl:151-158` admits "this port shades in the sRGB domain" — the DONE rests on an unprovable equivalence | CLAIM-OTHERWISE |
| 09-6 | pixel | The tile colour lookup is incomplete | Claim: (silent) · Reality: C# `GetFinalLight` (`TileDrawing.cs:4358-4402`) → `ShouldTileShine` (`:4404-4433`) → `Main.shine`; the port's `push_lit` covers only `actColor` (`tiles.rs:526-528`) | SILENT |
| 09-7 | behavioural/pixel | The quality-light tier constants differ | Claim: (silent) · Reality: C# `_highQualityLightingRequirement = (255*(1-q)+30q, ×1.1, ×1.2)` and `_medium… = (50*(1-q)+2q, …)` (`:479-486`) vs the port's `HIGH_LIGHT`/`MEDIUM_LIGHT` (`tiles.rs:362-366`) | SILENT |
| 09-8 | cosmetic | `DrawTileCracks` is missing | Claim: (silent) · Reality: the mining/hit overlay (`:732-733`) has no counterpart | SILENT |

### 09b — tile drawing, additional admitted-only notes (not in this register)

The `deferred special points → inline drawing` deviation is admitted at `tiles.rs:671-678`,
`:727-730`; the missing animator is admitted by ROADMAP ("animator not ported"); the
`WindGrid` push wave is admitted by ROADMAP M1.4-WEATHER-1 ("Out: the WindGrid push wave").

### 10 — lighting (9)

| Source | Severity | One-line risk | Claim vs reality | Status |
| --- | --- | --- | --- | --- |
| 10-1 | behavioural, high | Entity `AddLight` sources are missing | Claim: (silent) · Reality: 300+ C# call sites project light from NPCs, projectiles, dust and items; the Rust map only accepts waterfall lights → glowing entities/explosions are dark at night | SILENT |
| 10-2 | pixel, medium | Water decay is style-0 only | Claim: (silent) · Reality: `blur.rs:33` hardcodes `(0.88,0.96,1.015)*0.91`; C# `LightingEngine.cs:147-182` has 13 arms (blue/red water styles etc.) → any non-default `waterStyle` diverges | SILENT |
| 10-3 | pixel, medium | The water jitter is an approximation | Claim: (silent) · Reality: C# `_random.WithModifier((ulong)i).Next(98,100)/100` ∈ {0.98, 0.99} (`LightMap.cs:219`); the port uses `(i*2654435761)%15 → 0.985..0.999` (`blur.rs:95-98`, `:136-139`) | SILENT |
| 10-4 | pixel, low | `NON_VISIBLE_PADDING` is clamped | Claim: (silent) · Reality: `blur.rs:43` uses `min(18, min(w,h)/3)`; C# is a fixed 18 (`LightMap.cs:31`, `:99-108`) | SILENT |
| 10-5 | behavioural, medium | Per-column surface cutoff instead of the global `Main.worldSurface` | Claim: the comment admits the intent (hills/valleys) · Reality: C# `TileLightScanner.cs:92` uses the single global `Main.worldSurface` → it *is* a pixel change and needs a deterministic A/B | SILENT |
| 10-6 | pixel, medium | The sliced path misses the per-slice `GetFinalLight` | Claim: (silent) · Reality: C# applies `GetFinalLight` on every slice (`TileDrawing.cs:3581`, `:4391-4397`); the port's `push_sliced` (`tiles.rs:1367`) only calls `LightStyle::shade`, so paint tint / `light_scale` / actuated ×0.4 never reach the 9 slices | SILENT |
| 10-7 | behavioural, medium | White/Retro/Trippy are shader approximations | Claim: `ROADMAP.md:66` marks Lighting `✅` with "4 modes" · Reality: the 1442-line `LegacyLighting` engine (sky-colour swipe, data shifting, `doColors_Mode*`) is unported; only the style descriptors exist (`params.rs::LightStyle::shade` `:116-148`) | CLAIM-OTHERWISE |
| 10-8 | behavioural, low | The 4-phase engine split is collapsed | Claim: (silent) · Reality: minimap light export and scene metrics run in separate phases in C# (`LightingEngine.cs:76-108`); the port computes light on demand per drawn rect | SILENT |
| 10-9 | cosmetic, low | `GetSubLight` (bilinear sub-tile sample) is missing | Claim: (silent) · Reality: `Lighting.cs:124-133` is not found in the renderer | SILENT |

### 11 — liquid (10)

| Source | Severity | One-line risk | Claim vs reality | Status |
| --- | --- | --- | --- | --- |
| 11-1 | behavioural, high | Liquid type reactions are absent | Claim: ROADMAP §3 lists only the render leftovers · Reality: `LiquidUpdate` never calls `LavaCheck/HoneyCheck/ShimmerCheck`, so water meeting lava never becomes obsidian (`Liquid.cs:1261-1265`, tile 56), honey+water never becomes honey block/crisp honey (229/230, `:1399-1420`), and nothing becomes 659 → pools of different liquids simply refuse to mix | SILENT |
| 11-2 | behavioural, med-high | The `delay` cadence is missing | Claim: (silent) · Reality: lava moves every 6th visit (`delay < 5`) and honey every 11th (`delay < 10`) — C# `Liquid.cs:485-491/516-522` → lava/honey propagate ~6-11× faster and settle differently | SILENT |
| 11-3 | behavioural, med | Underworld evaporation is missing | Claim: (silent) · Reality: `Liquid.cs:466-474` removes 2 units per update from water below `UnderworldLayer`; the port conserves mass there — and the existing mass-conservation assertion (`liquid.rs:445`) encodes the *opposite* invariant | SILENT |
| 11-4 | determinism, med | The skip/window mechanics differ | Claim: ROADMAP's "Hidden" line mentions `skipCount > 1`, a *different* mechanism · Reality: `skipLiquid` is not ported at all and the C# window is a fixed 2500-slot stride with a clamp that re-walks the entire active list every update (`Liquid.cs:1070-1082`), while the port walks `ceil(len/10)` entries (`liquid.rs:135`) | SILENT |
| 11-5 | determinism, low-med | The `genRand` nudge is missing | Claim: (silent) · Reality: `Liquid.cs:892-895` (`num == 254 && WorldGen.genRand.Next(30) == 0 → 255`) is absent; the C# call also consumes a shared RNG draw → settled-surface levels differ by 1 unit on ~1/30 of cells | SILENT |
| 11-6 | determinism, low | The extra wake calls are missing | Claim: (silent) · Reality: in the 7- and 5-neighbour arms the original re-queues ±1/±2/±3 whenever the neighbour *or the centre* changed (`Liquid.cs:748-771`, `:821-836`); the port only queues on an actual write → queue order (and hence update order) differs | SILENT |
| 11-7 | behavioural, low-med | `AddWater`'s guards are absent | Claim: (silent) · Reality: the 5-tile world-border exclusion (`Liquid.cs:1163`), the `tile.type != 546` carve-out and the `CheckWaterDeath/CheckLavaDeath` → `KillTile` of submerged plants (`:1183-1206`) are all missing | SILENT |
| 11-8 | behavioural, low | `tilesIgnoreWater` is missing | Claim: (silent) · Reality: `Liquid.cs:69-73/986/1157` flips `Main.tileSolid[546]` (+ boulder solidity) for the whole pass, and `Liquid.Update:451` forces `tileSolid[379] = true` per cell | SILENT |
| 11-9 | behavioural, med | `quickSettle`/panic/stuck detection is absent | Claim: (silent) · Reality: the world-load settle (`WorldFile.cs:735-760`), panic mode and the stuck self-clear are absent → a pathological pool never force-settles and never empties itself | SILENT |
| 11-10 | behavioural | The ROADMAP "core ✅" claim over-states the simulation | Claim: §3 marks the liquid core `✅` (M1.5-LIQUID-2/3) · Reality: rows 1-11 above show the simulation is PARTIAL; the DONE rows cover only the cellular water flow / the draw side | CLAIM-OTHERWISE |

### 12 — worldgen (8)

| Source | Severity | One-line risk | Claim vs reality | Status |
| --- | --- | --- | --- | --- |
| 12-1 | behavioural | The config layer is parsed but not consumed | Claim: only in the `run` doc comment (`:2493-2496`) · Reality: `run` (`:2524`) builds `pass_config` per step and every adapter takes `_pass_config` unused → every C# `Configuration.json` value is still a hard-coded constant, although ROADMAP M5-GEN-1 claims "config-driven passes" | CLAIM-OTHERWISE |
| 12-2 | cosmetic | The chain weights are `_weightMap` values, not the ctor weights, and are unused | Claim: only in the code's own notes · Reality: `Terrain` 507.352 vs `TerrainPass.cs:51-54`'s 449.3721923828125; `Jungle` 11205.83 vs `:14-17`'s 10154.65234375; `Cactus & Coral` 1.0 because the map key is `"Cactus & Coral"` | SILENT |
| 12-3 | behavioural | `Settle Liquids` has no equivalent, so the C# post-pass liquid invariant cannot be honoured | Claim: (silent) · Reality: `WorldGenerator.cs:508` asserts `Liquid.numLiquid == 0` after every pass; the port has no liquid-flux simulation, so generated water is whatever the passes wrote | SILENT |
| 12-4 | behavioural | The spawn point is a compact approximation | Claim: the remix arm's doc comment cites the C#; the classic deviation is not in the docs · Reality: `step_world_tail` takes `spawn_center` and `heights[center]-6`; C# `SpawnPoint` (`WorldGen.cs:19460-19588`) scans upward with flood checks and rejects occupied/liquid cells → a spawn inside terrain is possible | SILENT |
| 12-5 | documentation | Pinned constants in the docs lag the code | Claim: `world_hash.rs:173-174` pins `SEED42_512 = 3075125982` / `DEFAULT_WORLD = 366022756`; ANALYSIS records `3074238937`/`4238218118` and ROADMAP M5-GEN-1's `3510339375`/`3248003836` · Reality: three published numbers are stale (the chain of custody is in `world_hash.rs:64-172`) | SILENT |
| 12-6 | cosmetic | A test name lies about its assertion | Claim: (silent) · Reality: `pipeline::the_executed_pipeline_is_59_named_steps` asserts `names.len() == 62`; the count grew 39→44→50→52→56/61→62 and the name was never updated | SILENT |
| 12-7 | documentation | ROADMAP's status table contradicts its own catalog | Claim: `ROADMAP.md:69` marks world generation 🟡 with "parked LIVING-trees" · Reality: `living_trees.rs` (1 113 lines, 12 tests) is a faithful port whose catalog row (`ROADMAP.md:1571`) says done; M5-DUNGEON/M5-TEMPLE are 🔄/🟡 partial, not parked | CLAIM-OTHERWISE |
| 12-8 | behavioural | `StructureMap` protection is absent | Claim: only the honey-patch section names a `protected` vec "stand-in" · Reality: the C#'s `StructureMap` (`WorldBuilding/StructureMap.cs`, 94 lines) has no Rust counterpart → passes that would register rects just write tiles and can overlap | SILENT |

### 13 — collision (9)

| Source | Severity | One-line risk | Claim vs reality | Status |
| --- | --- | --- | --- | --- |
| 13-1 | behavioural | The wet flags read one tile, not a box | Claim: M2-PHYS-3 claims the four flags are done · Reality: they are done in *kind*, not in *shape* — C# `WetCollision` scans the hitbox against every liquid tile using the surface offset and accepts a sloped tile above the liquid (`Collision.cs:1610-1612`, `:1628-1646`); the port samples the mid tile only (`player.rs:2487`) → feet-in-shallow-water and slope-over-water do not wet the player | CLAIM-OTHERWISE |
| 13-2 | behavioural | There is no `StickyTiles` | Claim: (silent) · Reality: the C# adhesion (tiles 51/229, the `rand.Next(30)` dust-30 trail) has no Rust counterpart; `Player.sticky` is a different mechanism (`FloorVisuals`) | SILENT |
| 13-3 | behavioural | No hurting-tile damage | Claim: the M3-EQUIP-3 note mentions the hot-tile touch is unported · Reality: `AnyHurtingTiles`/`CanTileHurt` are absent → spikes, geysers and lava-tile contact damage are missing | SILENT |
| 13-4 | behavioural | `tile_collision` is gravDir-1-only | Claim: (silent) · Reality: `gravDir = −1` bodies cannot use the port and the `−0.01` landing branch (`Collision.cs:2394`) is unreachable; `ignoreDoors`/`ignoreAetheriumPlatforms` are not exposed | SILENT |
| 13-5 | determinism, low | The `scan_bottom` heuristic differs | Claim: (silent) · Reality: C# always clamps to `maxTilesY − 40`; the port only applies the margin when `world.height > 400` (`collision.rs:54-60`) → tiny test worlds scan 40 rows the vanilla would skip | SILENT |
| 13-6 | cosmetic/behavioural | The hoik edge at `i == 0` | Claim: (silent) · Reality: C# skips the left arm when `i < 1` (`:2401-2405`); the port reads `world.tile(i − 1, j)` → `None` → slope 0 → the collision still applies | SILENT |
| 13-7 | behavioural, low | `step_up` ignores `specialChecksMode` | Claim: (silent) · Reality: the `IgnoredByNpcStepUp` set is not plumbed; only the player calls `step_up`, so no live divergence today, but the NPC reuse path inherits it | SILENT |
| 13-8 | behavioural | The AI line-of-sight is an approximation | Claim: (silent) · Reality: `npc_ai.rs::line_clear` samples the segment every 8 px instead of running `CanHitLine`/`CanHit`'s tile-corner walk → NPCs can see/aggro through geometry the vanilla blocks; the faithful walks ship as dead code | SILENT |
| 13-9 | behavioural, low | `line_clear` uses `TILE_SOLID` only | Claim: (silent) · Reality: no `!inActive`/`!active` handling of actuated tiles and no `tileSolidTop` exemption (`projectile.rs:6989-6992` shows the correct conjunction exists in the other helper) | SILENT |

### 14 — map / minimap (8)

| Source | Severity | One-line risk | Claim vs reality | Status |
| --- | --- | --- | --- | --- |
| 14-D10 | behavioural | The reveal window is a fixed 75×47 tile box around the player | Claim: `ANALYSIS.md:1560-1561` states it without flagging it · Reality: C# `ExportToMiniMap` reveals the lighting engine's processed area shrunk by 28 px and intersected with the world bounds inset 40 (`LightingEngine.cs:252-256`); the port reveals `px±37`, `py±23` at 4 Hz centred on the **player** rather than the view centre → less area revealed, and fast travel leaves a trail of unrevealed tiles | SILENT |
| 14-D13 | behavioural | Pre-2 `.map` formats never open | Claim: (silent) · Reality: `LoadMapVersion1`/`LoadMapVersionCompressed`/`DecompressChunk` have no counterpart (`MapHelper.cs:3347-3585`, `:3587-3747`) → a legacy sidecar is rejected as a foreign version | SILENT |
| 14-D14 | behavioural | There is no `MapUpdateQueue` | Claim: (silent) · Reality: tile edits made outside the player's reveal window (worldgen after load, net-received tile changes `MessageBuffer.cs:1153`, `NetMessage.cs:2456`, `WorldGen.cs:80347/80402/81334`) never refresh their map pixel until the reveal window revisits them with ≥ light | SILENT |
| 14-D15 | behavioural | No `UnlockMapSection` / `UnlockMapTilePretty` / `DebugOptions.unlockMap` | Claim: (silent) · Reality: the debug "unlock map" reveal (`Main.cs:61873-61882`) and its near/full-bright fade (`WorldMap.cs:140-174`) are absent | SILENT |
| 14-D17 | cosmetic | The minimap has no alpha config | Claim: (silent) · Reality: `Main.mapMinimapAlpha` (`Main.cs:55523-55532`) has no config key or clamp; the panel always draws at full alpha | SILENT |
| 14-D18 | behavioural | The `mapStyle` switcher is absent | Claim: (silent) · Reality: only `mapStyle == 1` is modelled; the `mapStyle == 2` screen-overlay map and its `mapOverlayScale` (`Main.cs:55548-55578`) and `mapStyle == 0`'s retro layout have no counterpart | SILENT |
| 14-D19 | behavioural | The map icon overlays are entirely missing | Claim: (silent) · Reality: pings (`PingMapLayer.cs`), team spawn, pylon links, boss bags, the Digtoise marker, `MapIconOverlay`'s NPC/player/chest text (`Main.cs:55782-55926`, `:56631`) and the golf/potion-of-return icons have no Rust code → multiplayer map callouts and single-player NPC/chest labels simply do not exist | SILENT |
| 14-D22 | behavioural | `Main.mapDelay` and `Main.refreshMap`/`Main.clearMap` are not modelled | Claim: (silent) · Reality: the first-frames skip (`LightingEngine.cs:82-89`) and the content-lost-triggered full refresh (`MapRenderer.cs:363-377`) are replaced by the port's `uploaded`/`needs_full_upload` pair → never a full sweep after a GPU reset | SILENT |

### 15 — dust / gore (7)

| Source | Severity | One-line risk | Claim vs reality | Status |
| --- | --- | --- | --- | --- |
| 15-1 | behavioural+pixel | There is no general dust/gore layer | Claim: ROADMAP/ANALYSIS admit it per-feature ("the dust/gore spawns are SKIPPED", `npc.rs:9349`, `projectile.rs:5512`) but never as a module gap; PORT_MAP has no row · Reality: 3161 of 3279 dust sites and 1642 of 1643 gore sites have no port → every hit, step, spell, death and liquid splash loses its particles | SILENT |
| 15-2 | behavioural | Three unrelated pools instead of one | Claim: (silent) · Reality: `Sparks` (500), `Sand::parts` (budget `1000*screenW/1920`), `App::leaves` (400) — not the C# model (`Main.dust[6001]` + `Main.gore[600]`) | SILENT |
| 15-3 | behavioural | The pool pressure model is absent | Claim: (silent) · Reality: the C# `dCount` bands (Dust.cs:97-152, 2347-2422) both gate spawns *and* shrink particles; the port's comment at `minecart.rs:999-1001` says "pinned to no pressure" | SILENT |
| 15-4 | determinism | Per-consumer RNG streams | Claim: (silent) · Reality: each stand-in carries its own `Main.rand` stand-in → the *global* RNG order diverges from C# even where a consumer is "ported" | SILENT |
| 15-5 | pixel | Dust light is absent | Claim: (silent) · Reality: ~60 dust types feed `Lighting.AddLight`; the ported 213 arm documents "the port writes no dust light" (`minecart.rs:1025-1027`) | SILENT |
| 15-6 | cosmetic/behavioural | The `ChildSafety` gate is inert | Claim: BUG_REPORT's truth pass records "`blood` is the `ChildSafety` gate — nothing to gate yet" · Reality: the toggle is parsed and saved but no sim or render code reads it — it must be wired at spawn, not at draw | SILENT |
| 15-7 | cosmetic | The gore shards are deferred with no mechanism | Claim: ROADMAP M1.4-TO-4 says "the pot gore shards deferred (no gore mechanism)" · Reality: `edit.rs:2053-2054` asserts the absence, so nothing is lost yet — but the mechanism does not exist for anything else | SILENT |

### 16 — weather (9)

| Source | Severity | One-line risk | Claim vs reality | Status |
| --- | --- | --- | --- | --- |
| 16-1 | behavioural | `any_player_120_life` has no writer — the 0.35 wind cap is dead code | Claim: `ROADMAP.md:1312` (M1.4-WEATHER-1) says "Already ported and verified: … the 0.35 walk cap" · Reality: `wind.rs:75` is set `true` in `Wind::new` `:108` and **never written anywhere in `crates/`** → `Main.cs:58863-58871`'s low-wind limit is dead code and untested | CLAIM-OTHERWISE |
| 16-2 | behavioural | `Sandstorm.cs:58-59`'s hardMode denominator is not taken | Claim: only in a stale code comment · Reality: `scene.rs:591` pins `21600 * 3` while `World::hard_mode` exists → sandstorms start 1.5× too often in hardmode | SILENT |
| 16-3 | behavioural/pixel | Rain drops use a frozen wind snapshot | Claim: only in a code comment · Reality: vanilla's per-tick `Main.windSpeedCurrent` (`Rain.cs:56`, `Cloud.cs:391`) vs `Rain::wind` (`rain.rs:124-128`, `:811`) → sheeting rain does not bend with a gust | SILENT |
| 16-4 | behavioural/pixel | `NewRain`'s spawn factors are missing | Claim: (silent) · Reality: `gfxQuality` (`Rain.cs:176-180`), the snow-tile suppression `(1-(snow/max)²)` (`:181-182`), the `atmo⁹`/`atmo<0.4` gates (`:183-186`) and the `waterStyle*3` frame type (`:205`) are absent → drop density underwater/deep/at low quality differs | SILENT |
| 16-5 | behavioural | The Lantern Night force-stop is missing | Claim: only in code · Reality: `rain.rs:609-610` folds `!slimeRain && !LanternsUp && !NextNightIsLanternNight` (`Main.cs:64890`) to "always open" and skips `:64825-64831`'s `cloudBGActive = 0`/`numClouds = 30`; the wind walk's `LanternsUp` early-out (`:58856-58859`) collapses into `Wind::frozen` | SILENT |
| 16-6 | behavioural | The music-box / visual-scan window is substituted | Claim: only in a code comment · Reality: `scene_scan.rs:141-151` scans the 169×124 world window for tile 139 instead of the visual screen rect (`SceneMetrics.cs:4489-4501`) → a music box just off-screen can silence/override the track | SILENT |
| 16-7 | behavioural | The blood-moon/eclipse rolls are partially folded | Claim: the BG-21 row still claims these are "Out (M4)" · Reality: they are landed (`clock.rs:37-38`, `:72` → `world.rs:565`/`:594`); the *real* gaps (`tenth_anniversary` hardcoded false, `spawn_eye` false, the `stopEvents` gate, the pumpkin/snow-moon clearings `Main.cs:64837-64845`) are unstated — doc drift | CLAIM-OTHERWISE |
| 16-8 | doc | `M1.4-BG-23` has no catalog row at all | Claim: (silent) · Reality: its whole scope (the blood-moon/eclipse rolls + the dial-cooldown force-resets) is merged via `clock.rs`/`world.rs` | SILENT |
| 16-9 | behavioural | The `UpdateTime` rain gate's `!slimeRain` arm | Claim: (silent) · Reality: the port's start rolls run while the slime rain is not modelled in `Rain::update_time` (`rain.rs:609-610` passes `slimeRain` as always false) → a vanilla Lantern-less slime-rain night cannot start a rain event | SILENT |

### 17 — netcode (6)

| Source | Severity | One-line risk | Claim vs reality | Status |
| --- | --- | --- | --- | --- |
| 17-3 | behavioural | `49 InitialSpawn` is reused as "world complete" | Claim: (silent) · Reality: `MessageBuffer.cs:2447-2453` uses 49 to move the *client* to State 10 and call `Spawn`; the Rust name is `WORLD_COMPLETE` (`net.rs:77`) → any later port of the real 49 arm collides | SILENT |
| 17-4 | behavioural | Ping is id 154 in C#, module 2 in Rust | Claim: (silent) · Reality: `Net/Ping.cs:31` sends a bare `154`; the port sends NetModule 2 → ping is unportable and the vanilla-ping UI has no source | SILENT |
| 17-9 | pixel/behavioural | Client-side smoothing is invented, not ported | Claim: (silent) · Reality: `net_interp_tick`'s exponential decay has no C# counterpart; vanilla relies on ReLogic socket interpolation → any determinism/screenshot comparison of remote-entity pixels is port-specific | SILENT |
| 17-10 | behavioural | The NPC stale timeout (200 ms) is port-only | Claim: (silent) · Reality: vanilla despawns only on an explicit server `23` with `active = false` | SILENT |
| 17-11 | cosmetic | The port's `12 PlayerSpawn` arm is gate-whitelisted | Claim: (silent) · Reality: C# allows `b > 12` for the 9 whitelisted ids only (`MessageBuffer.cs:165`); the port's `GATE_WHITELIST` contains 12 → the gate no longer matches the source literally | SILENT |
| 17-18 | behavioural | The `netSpam` throttle, `netImportant` protection and `DoesPlayerSlotCountAsAHost` are unported | Claim: `ROADMAP.md:1063-1065` lists them under "Hidden" · Reality: the send-throttle behaviour of the C# server has no Rust equivalent at all | CLAIM-OTHERWISE |

### 18 — UI framework (7)

| Source | Severity | One-line risk | Claim vs reality | Status |
| --- | --- | --- | --- | --- |
| 18-1 | behavioural (latent) | Margins are silently dropped | Claim: admitted in code only (`uistate.rs:54-58,105`) · Reality: all current callers pass 0 so behaviour matches today, but any new element with a margin diverges invisibly | SILENT |
| 18-2 | behavioural (latent) | The `MaxHeight` clamp uses `f32::MAX` | Claim: (silent) · Reality: it should be `MaxHeight.GetValue(parentH)` (`UIElement.cs:314,316`) → a percent-valued `MaxHeight` cannot be expressed | SILENT |
| 18-3 | behavioural | There is no child→parent event bubble | Claim: `ROADMAP.md:938` — "events **bubble child→parent**" · Reality: `pointer.rs:154-224` raises events on the hit element only; ancestor `On_*` handlers never fire and ancestor `IsMouseHovering` is never set | CLAIM-OTHERWISE |
| 18-4 | pixel+behavioural | There is no scissor / `OverflowHidden` | Claim: admitted per-screen only · Reality: rows past a list bottom are *skipped* rather than clipped (`char_select.rs:856`, `world_select.rs:818`, `char_creation.rs:1859`); a row straddling the edge draws whole, and a fully past-edge row that C# still hit-tests is unhittable in the port | SILENT |
| 18-5 | behavioural, low | The scroll wheel is not in the pointer machine | Claim: (silent) · Reality: `UserInterface.cs:219-223` runs scroll after the two buttons with the delta consumed once; the port has per-screen counters and no shared `UIScrollWheelEvent` → order across screens differs | SILENT |
| 18-6 | determinism-relevant | The frame ordering is not pinned | Claim: (silent) · Reality: C# `Update` is hover → left click → right click → scroll → state update (`UserInterface.cs:190-228`); the front-end screens are event-driven from `loop.rs:390-403`, so the press dispatch may run before or after a hover refresh in the same frame — double-click edges are affected and no test pins it | SILENT |
| 18-7 | behavioural | The `GoBack`/history "dead code" claim is unverified | Claim: `pointer.rs:8-10` states zero call sites in 1.4.5.8 · Reality: `UserInterface.GoBack` (`:305-313`) *is* called from the back-button paths of `IHaveBackButtonCommand` screens; the port reimplements back as a `next_flow_state` table entry per screen → the 2-deep history pop is not modelled | SILENT |

### 19 — front-end screens (8)

| Source | Severity | One-line risk | Claim vs reality | Status |
| --- | --- | --- | --- | --- |
| 19-1 | behavioural | `IsValid` and the creative/journey rejection menu are missing on Play | Claim: the ROADMAP fix-world-select-clicks row says "the `valid`/path gates on Play/Delete are faithful to `UIWorldListItem.cs:362`/`:346-358`" · Reality: only half is true — the path is parked, the validity gate is not; `card_hit`/`elements` (`world_select.rs:545`) offer the Play button for an invalid card | CLAIM-OTHERWISE |
| 19-2 | doc | `ANALYSIS.md:2120-2124` is stale | Claim: "**No `.plr` reader/writer** … Create records the session draft instead of writing a character file" · Reality: `save_list.rs:291` reads real `.plr` files and `app/session.rs:141-176` writes one via `player_file::write_player_file` (`PlayerFileData.CreateAndSave` `PlayerFileData.cs:50-63`) — the same stale sentence survives in `flow.rs:73` and in the user-visible notice `app/mod.rs:1482` | CLAIM-OTHERWISE |
| 19-3 | doc | `PORT_MAP.md:180` is stale | Claim: "partial (no `.plr` reader; metadata stand-ins)" · Reality: the reader is live and the port writes players; only the metadata stand-ins remain | CLAIM-OTHERWISE |
| 19-4 | cosmetic/pixel | The card status overlays are pinned but never drawn | Claim: admitted in code only (`world_select::overlay_rects` `:343-350`) · Reality: `IconCompletion`/`IconMixedSeed`/`IconNewlyGenerated` (`UIWorldListItem.cs:71-93`, `:186-199`) are `#[allow(dead_code)]` with tested geometry only | SILENT |
| 19-5 | cosmetic | `AssignRandomWorldName` cannot re-roll a renamed draft | Claim: admitted in code only · Reality: the stand-in re-derives `"World {n+1}"` (`:97-99`) → the C#'s re-randomisation of an explicitly named world (`UIWorldCreation.cs:1002-1025`) is a no-op after an edit | SILENT |
| 19-6 | behavioural (latent) | The world sort's active-player key is hardcoded false | Claim: admitted in code only (`save_list.rs:426-433`) · Reality: `enter_front` calls `sort_worlds(&mut self.worlds, false)` (`mod.rs:1368`) because no active player exists at that hop, while C# reads `Main.ActivePlayerFileData.Player.difficulty` (`UIWorldSelect.cs:177-180`) → the sort and the Play rejection stay wrong until this is wired | SILENT |
| 19-7 | behavioural | `sort_players`/`sort_worlds` have no `LastPlayed` key | Claim: admitted in code only · Reality: insertion order stands in (`save_list.rs:391-424`) and the filename tie-break compares the path component | SILENT |
| 19-8 | cosmetic | The session-entry hydration is undocumented | Claim: only in code comments · Reality: the session card always reports `inv_favorites = 0` and `mana_max = 20` and takes `life_max` from the live player (`session.rs:71-110`) while a same-named disk card replaces it (`save_list.rs:321-329`) | SILENT |

### 20 — in-game UI (10)

| Source | Severity | One-line risk | Claim vs reality | Status |
| --- | --- | --- | --- | --- |
| 20-1 | behavioural | Right-click takes half the stack, not one item | Claim: (silent) · Reality: `clicks.rs:911-922` uses `(count + 1) / 2` against `PickupItemIntoMouse` (`ItemSlot.cs:1717-1772`), which does `mouseItem.stack++`; the same shape is in the coin/ammo rows (`clicks.rs:238`) and the bank panel (`:813`) | SILENT |
| 20-2 | behavioural | The inventory right-click is not paced and does not repeat | Claim: (silent) · Reality: C# gates on `Main.stackSplit > 1` (`:1700`) and re-fires while the button is *held* (`:1654`); the port consumes an edge (`loop.rs:438`) → a held right-click yields exactly one half-stack | SILENT |
| 20-3 | behavioural | There are no right-click equip/dye/swap actions | Claim: M7-UI-3 admits only the *dye drag* · Reality: `SwapEquip` (`:1685-1697`), `SwapVanityEquip`, `HasItemSwap` and openable bags are absent → equipping by right-click is impossible, and the dye/mount/pet pages accept any held item because `context_accepts` only gates contexts 1/2 | CLAIM-OTHERWISE |
| 20-4 | behavioural | `MAX_STACK` (flat 999) in the right-click merge arm | Claim: (silent) · Reality: `clicks.rs:934` uses `item::MAX_STACK` while every sibling uses `registry::max_stack(kind)`; the C# gate is `Main.mouseItem.maxStack` (`ItemSlot.cs:683`, `:1707`) → diverges for any kind whose `maxStack` is not 999 | SILENT |
| 20-5 | behavioural | The crafting UI is one click deep | Claim: (silent) · Reality: no Shift ×10 craft-all, no `superFastStack`, no `stackSplit` pacing, no `focusRecipe`/`_selectedRecipeIndex` select step (`NewCraftingUI.cs:499-528`), no filter/search hit tests, no guide item, no `HoverOverCraftingItemButton` tooltip chain (`Main.cs:42250-42306`) → the filters are drawn (`ui/crafting.rs:690`) but inert | SILENT |
| 20-6 | behavioural | There is no `CoinSlot.ForceSlotState` | Claim: (silent) · Reality: the C# re-derives a merged coin slot's rank (`ItemSlot.cs:1016`, `:1119`); the port merges only identical kinds → a gold into a copper slot stays a mixed/broken state | SILENT |
| 20-7 | behavioural/cosmetic | The achievement hooks on slot moves are absent | Claim: (silent) · Reality: `NotifyItemPickup` (`ItemSlot.cs:960`) and `HandleOnEquip` (`:972`) never fire from the click path; only `notify_craft` does (`clicks.rs:409`) | SILENT |
| 20-8 | behavioural | The shift-sell cursor stands down | Claim: (silent) · Reality: `GetSellOrTrash` with a shop open (`:198-209`) is not modelled (`clicks.rs:1120-1122`) | SILENT |
| 20-9 | behavioural | The Sort button and name-field editing are partial | Claim: (silent) · Reality: `ChestUI`'s Sort (id 4) has no click arm | SILENT |
| 20-10 | doc | BUG-20 is implemented but its catalog row is not closed | Claim: `ROADMAP.md:1705` still reads "trash slot: quick-trash (`cursorOverride == 6`), clear on death, the +5/+168 chest-shop offset" with no done text · Reality: the code and three `bug20_*` tests are on main | CLAIM-OTHERWISE |

### 22 — chests / loot / shops (6)

| Source | Severity | One-line risk | Claim vs reality | Status |
| --- | --- | --- | --- | --- |
| 22-3 | behavioural | The buy-price formula is incomplete | Claim: (silent; only the "no discount accessory yet" note) · Reality: `GetItemExpectedPrice` `:35380-35397` divides *both* prices by `PriceAdjustment` and then `/5` with a `storeValue != 0 && < 1 → 1` clamp; `shop.rs::buy_price` (`L1257-1259`) only multiplies → every `buyOnce` row costs **5× the vanilla**; `sell_price` (`L1282-1284`) conversely divides every item by 5 | SILENT |
| 22-4 | behavioural | `discountAvailable` and `shopSpecialCurrency` are missing | Claim: (silent) · Reality: the 0.8 Discount Card factor (`:35369-35374`) and the `calcForSelling = calcForBuying` aliasing for `isAShopItem` are not modelled at all | SILENT |
| 22-5 | behavioural (small) | Two stock conditions are hard-false | Claim: (silent) · Reality: `shop.rs:1268-1270` returns `false` for `StockWhen::HardModeMoonlord` and `DownedBothSlimes` → those rows can never appear | SILENT |
| 22-6 | cosmetic/UI | The bestiary drop-rate reporting is unported | Claim: (silent) · Reality: `ReportDroprates` / `DropRateInfoChainFeed` are absent (waits on M4-BEST-1) | SILENT |
| 22-7 | cosmetic | The chest icon tables are unported | Claim: (silent) · Reality: `chestTypeToIcon`, `chestTypeToIcon2`, `dresserTypeToIcon` have no Rust twin → any UI showing a chest style's item icon must build its own table | SILENT |
| 22-8 | behavioural | The price rounding uses `round_half_to_even` on f32 | Claim: (silent) · Reality: the C# casts through `(int)Math.Round(float)` on *longs* in some branches → a `f64` audit is cheap insurance | SILENT |

### 24 — spawner / invasions (14)

| Source | Severity | One-line risk | Claim vs reality | Status |
| --- | --- | --- | --- | --- |
| 24-1 | behavioural, high | `GetSpawnRate`'s town-NPC distribution block is unported | Claim: M4-SPAWN-1 claims the chain "ported in the vanilla order"; the doc comment at `spawner.rs:193-196` even claims the random town-NPC sub-branches are kept · Reality: C# draws `Main.rand` at `:807`/`:811`/`:824`/`:828`/`:840`/`:844`/`:860-916` to set `noWorms`/`spawnFriendly` and re-scale `spawnRate` by ×1.25…×3 (and the graveyard ×1.66/×2.33/×3); the port's function *discards its `rng`* (`:203`) → any world with town NPCs over-spawns relative to C# | CLAIM-OTHERWISE |
| 24-2 | behavioural, high | There is no `invaders` spawn-rate override | Claim: `ROADMAP.md:1513` (M4-EVENT-1a) describes "the 2-second spawn pass near the player while the invasion lasts" · Reality: C# forces `spawnRate = 20` / `maxSpawns = 5·(2+0.3·players)` while an invasion window is open (`NPC.cs:781-785`); the port fires `invasion_spawn_pick` **once per fixed step** with no rate roll and no nearby cap (`world_sim/npcs.rs:3386-3423`) → ~20× the vanilla density | CLAIM-OTHERWISE |
| 24-3 | behavioural | Invasion enemies are not placed through `FindSpawnTile` | Claim: (silent) · Reality: the port spawns them at `(playerX - 600, playerY - 400)` clamped to a tile row (`world_sim/npcs.rs:3429-3443`); C# picks a validated tile through the 50-candidate scan with `HasTileSpawnSpace`, the safe-area and on-screen checks (`NPC.cs:969-1048`) → invasion NPCs can materialise inside blocks or in mid-air | SILENT |
| 24-4 | behavioural | `ShouldSpawnInvasionEnemies`'s town fallback is missing | Claim: only in the port's doc comment (`npcs.rs:1745-1748`) · Reality: the `invasionX` ±5-tile-of-`maxTilesX/2` + resident town NPC arm (with its `Next(3)`) is unported (`NPC.cs:455-468`) | SILENT |
| 24-5 | determinism | `invasion_spawn_pick` draws from `app.item_rng`, not the named `SpawnNPC` stream | Claim: (silent) · Reality: C# draws every one of these from `Main.rand` inside `SpawnAnNPC`; the port's own stream discipline cannot be compared tick-for-tick and the natural-spawn stream is left unadvanced on invasion ticks (`world_sim/npcs.rs:3407`) | SILENT |
| 24-6 | behavioural | Kind coverage gaps in the invasion tables | Claim: only in the `invasion_spawn_pick` doc (`:1775-1779`, `:1800-1804`) · Reality: the type-4 RayGunner 382 / BrainScrambler 381 / ScutlixRider 390 arms consume their rolls and land on the port's nearest kind, and the `!AnyNPCs(395)` Saucer gate is pinned `false` (`:3400`) → the Saucer branch can never fire | SILENT |
| 24-7 | behavioural | The blood-moon roll inputs are pinned | Claim: (silent) · Reality: `roll_blood_moon` receives `tenth_anniversary = false` and the eye-spawn gate `= false` (`clock.rs:38`) → the anniversary 1/6 variant and the `:65335-65341` eye gate never apply | SILENT |
| 24-8 | behavioural | The eclipse roll has no `stopEvents` gate | Claim: only at `clock.rs:69-71` · Reality: the port always evaluates the hardmode + mechs roll → an eclipse can start while another stop-event would have suppressed it (unreachable today, real the moment those events land) | SILENT |
| 24-9 | behavioural, high | Pumpkin/Snow Moon are pacing-only | Claim: ROADMAP's M4-EVENT-2 row has no DONE entry · Reality: `World.pumpkin_moon`/`snow_moon` exist (`world.rs:203-205`) and the raid pacing applies, but `spawn_npc_at` never reads `moon_event` — the C# wave tables inside `SpawnAnNPC` (`NPC.cs:2560-2960`) are entirely absent → a moon event floods the world with ordinary surface enemies instead of the wave roster | SILENT |
| 24-10 | behavioural | Lantern Night, Birthday Party, the Sandstorm event and the cultist ritual delay are missing | Claim: (silent) · Reality: no state exists in `World`; `Luck`'s lantern-night term, `ShopHelper`'s party prices, `Sandstorm.StartSandstorm`'s gates and the `SpawnAnNPC` arms gated on `BirthdayParty.PartyIsUp` (`NPC.cs:1643`, `:2593`) and graveyard flags are inert | SILENT |
| 24-11 | behavioural | Despawn encouragement is a single ad-hoc cull | Claim: (silent) · Reality: only the Eye of Cthulhu's day-time climb-away is modelled (`world_sim/npcs.rs:2262-2270`); `EncourageDespawn` (`NPC.cs:7249`) and the `DespawnEncouragement_*` family (`:52752`, `:60300`) are unported → off-screen enemies persist forever instead of being recycled | SILENT |
| 24-12 | structural | Dual invasion flags | Claim: (silent) · Reality: `World.invasion_goblins` and `World.invasion_type` both encode "an invasion is running"; `update_invasion_state` re-derives the former from the latter every tick (`npcs.rs:1362`) but `can_start_invasion`/`start_invasion` gate and write `invasion_goblins` → any future writer that updates one without the other silently diverges | SILENT |
| 24-13 | documentation | M4-SPAWN-1/2's DONE rows over-claim | Claim: the rows read as complete · Reality: the ported chain is faithful only through `NPC.cs:789`; `:790-928` (skyblock, the town block, the luck tail) is not ported and no test covers it | CLAIM-OTHERWISE |
| 24-14 | cosmetic/data | `is_ices_snow` is a hand-written `matches!` | Claim: noted in the source comment · Reality: it should read the generated `TileID.Sets.IcesSnow` (`TileID.cs:561`) | SILENT |

### 25 — NPC AI families (11)

| Source | Severity | One-line risk | Claim vs reality | Status |
| --- | --- | --- | --- | --- |
| 25-1 | behavioural | The Frost Legion trio runs the wrong speeds | Claim: the comment at `npc.rs:4184-4186` claims "the per-type speeds 3/3.5/4 fold into the walker params" · Reality: they do not — all three share `speed: 0.8`; C# `aiStyle 38` (`NPC.cs:28995-29145`) uses `num551 = 4f` (`3f` for type 143, `3.5f` for type 145) and `num552 = 1f` (`0.7`/`0.8`), with the throw/kick/jump arms absent → the whole invasion walks 4-5× too slowly and never attacks at range | SILENT |
| 25-2 | behavioural (portfolio-wide) | The `aiStyle 3` family collapses onto one walker | Claim: the comments at `npc.rs:4175-4179`/`:4184-4186`/`:4190-4192`/`:4364-4366` claim the per-type speeds fold in · Reality: only the goblins differ (1.0, via the shadowed `update_goblin`); the vanilla `AI_003_Fighters` carries per-type velocity tables and 260 type branches (`NPC.cs:59069+`) | SILENT |
| 25-3 | behavioural | The goblin arm is still shadowed | Claim: (silent) · Reality: `npc.rs:4163-4165` runs `update_goblin` before the intended `update_skeleton` at 4180-4183 → the second arm is dead | SILENT |
| 25-4 | behavioural | `GoblinSummoner` (C# type 471) is routed to the aiStyle-8 caster archetype | Claim: (silent) · Reality: the vanilla's type 471 sits in the `aiStyle 3` group (its summon is an `AI_003` arm); the port runs `update_caster` (`npc.rs:4377`) → the teleport-and-shoot cycle | SILENT |
| 25-5 | behavioural + drift risk | `aiStyle 2` is duplicated instead of shared | Claim: (silent) · Reality: `npc_ai::flyer_step` is documented as `AI_002_FloatingEye`'s core (`npc_ai.rs:125-128`) but its only caller is `update_cave_bat`; `update_eye` (5916-5948) re-implements the same maths and drops the vanilla's alpha phase (`ai[0] >= 300 → alpha 200`, `NPC.cs:52341-52355`) and its ±4 velocity ramps | SILENT |
| 25-6 | behavioural/determinism | The bats' velocity reroll is a positional hash, not `Main.rand` | Claim: (silent) · Reality: `update_cave_bat` (6472-6477) derives the jitter from `self.x` instead of drawing from the shared stream every ~30 ticks as in `AI_014` (`NPC.cs:22849+`) | SILENT |
| 25-7 | behavioural | `aiStyle 7` (52 vanilla types) is reduced to a wander | Claim: (silent; the family is not listed as ported) · Reality: `update_guide` (6389-6427) walks and flips every 240 ticks; the vanilla's `AI_007_TownEntities` (52965-55562, 69 type branches) owns the homes, the sitting, the idle animations, the shimmer and the teleport-to-home | SILENT |
| 25-8 | behavioural + identity | The aiStyle-84 cultist boss is missing and mislabelled | Claim: the Rust `AncientCultist` doc (`npc.rs:367-374`) claims `NPCID.CultistBoss` (439) but cites `NPC.cs:15172-15189` — the **type 422** row (aiStyle 94) · Reality: the vanilla 439/440 are a 24×50 boss on `aiStyle 84` (`AI_084_LunaticCultist`, `NPC.cs:64414`), with no Rust equivalent | SILENT |
| 25-9 | structural | The `aiStyle` dispatch itself does not exist in Rust | Claim: (silent) · Reality: the port keys on `NpcKind` (182 variants) and has no `ai_style` field; the 128-arm family structure is implicit in the per-kind machines | SILENT |
| 25-10 | cosmetic | `docs/NPC_AI_TAXONOMY.md` is stale | Claim: (silent) · Reality: its archetype table still lists aiStyle 5/6/8/14/16 as `TODO` — all five exist now in `npc_ai.rs`/`world_sim/npcs.rs` | SILENT |
| 25-11 | documentary | ROADMAP §14's header and the summary table over-claim | Claim: `ROADMAP.md:604-605` "**Rust** `npc.rs`, `world_sim/npcs.rs` · 🟡 3 kinds" and `ROADMAP.md:73` "NPC families, town, bosses, events ✅" · Reality: the roster is 182 kinds and the AI family layer covers ~60 of 128 vanilla families, with the two biggest (3 and 7) as approximations | CLAIM-OTHERWISE |

### 28 — buffs & debuffs (8)

| Source | Severity | One-line risk | Claim vs reality | Status |
| --- | --- | --- | --- | --- |
| 28-D-1 | behavioural+pixel, high | The buff HUD is entirely missing | Claim: ROADMAP §8 does not list a UI task for the buff bar · Reality: `Main.DrawBuffIcon` (`Main.cs:43307-43368`) draws the icon with the per-slot `buffAlpha`, the remaining-time text and the right-click removal → `Main.TryRemovingBuff` (:43370); the Rust has no icon, no alpha, no timer text, no removal — the only "buff" mentions under `ui/` and `render` are a `vanityPet`/`lightPet` comment (`ui/chrome.rs:238`) | SILENT |
| 28-D-2 | cosmetic | `IsAFlaskBuff` / `Main.meleeBuff` id 72 (Midas) | Claim: (silent) · Reality: `BuffID.cs:49` lists 9 flask ids (`71,72,73..79`); the Rust `MELEE_BUFF_IDS` (`buffs.rs:47`) has 8 → the Midas buff would not be flushed when another flask is drunk (which table `AddBuff_RemoveOldMeleeBuffsOfMatchingType` (`Player.cs:5347`) consults must be settled first) | SILENT |
| 28-D-3 | behavioural | The Nurse removes protected debuffs | Claim: (silent) · Reality: `BuffID.Sets.NurseCannotRemoveDebuff` (`BuffID.cs:52`, 20 ids) is not modelled; the Rust nurse heal clears every debuff (`app/mod.rs:1251-1260`) and `nurse_heal_cost_for` (:4293) counts *all* debuffs → the price is also wrong | SILENT |
| 28-D-4 | behavioural, high | The effect table is ~15 % ported | Claim: M2-BUFF-3/4/5/7/8 are `✅` for the *families* they cover · Reality: the C# `UpdateBuffs` chain is 251 arms over ~290 ids and the Rust covers ≈40 ids; unported examples that change play: 3 Swiftness / 5 Ironskin / 7 MagicPower / 16 Archery / 113 Lifeforce / 115 Rage / 117 Wrath / 159 Sharpened / 165 Dryad's Ward / 199 NoBuilding / 332-334 hunger / 396 Mana Heat — the ROADMAP overstates coverage by reading "M2-BUFF-4 stat debuffs" as "the effect table" | CLAIM-OTHERWISE |
| 28-D-6 | maintainability | The id space is magic numbers | Claim: (silent) · Reality: `BUFF_ID_COUNT = 401` is the only representation of the id table; every use site writes a literal (`player.rs:2844`, `npc_buffs.rs:502-518`) → the coverage table is not measurable and id drift is silent | SILENT |
| 28-D-7 | robustness | Out-of-range ids panic | Claim: (silent) · Reality: `BuffSlots::add_buff` (`buffs.rs:155`) indexes `buff_immune[buff_type as usize]` without a guard, while `find_buff_index` (:338) *does* guard → the two entry points disagree | SILENT |
| 28-D-8 | behavioural | The expert/master debuff extension is hard-wired to `expert_mode` | Claim: ROADMAP's "expert ×2 (master ×2.5)" · Reality: `add_buff_determine_buff_time_to_add` (`buffs.rs:175`) applies ×2 and never ×2.5; the field has no writer in the port (no difficulty modes yet) → the ×2 branch is currently dead and the claim is not observable | CLAIM-OTHERWISE |
| 28-D-9 | correction | `NPC.maxBuffs` is 20, not 10 | Claim: module 05's note about a "5-slot debuff pool" · Reality: `NPC.maxBuffs = 20` (`NPC.cs:95189`) and `NPC_MAX_BUFFS = 20` (`npc_buffs.rs:29`) agree; the "5-slot debuff pool" does not exist in the code | SILENT |

### 29 — town NPCs / housing (12)

| Source | Severity | One-line risk | Claim vs reality | Status |
| --- | --- | --- | --- | --- |
| 29-1 | behavioural | The room score (`ScoreRoom`) is not ported | Claim: (silent) · Reality: the C# only accepts a home when `hiScore > 0` and picks the **best-scoring** standing spot (`bestX/bestY`, `WorldGen.cs:5983-5988`); the Rust's `town.rs::scan_spot` (50-57) returns the **scan origin** and ignores the score → a town NPC can be homed on a spot vanilla would reject (no standing space / an evil room) and always homes on a different tile | SILENT |
| 29-2 | behavioural | The remix-world mood exclusion is missing | Claim: (silent) · Reality: the C# returns early for `Main.remixWorld` (`ShopHelper.cs:103`) so every remix-world NPC sells at `PriceAdjustment = 1`; `happiness.rs:318-328` documents the exclusion but only implements the Travelling Merchant arm → remix worlds get real mood prices | SILENT |
| 29-3 | behavioural | The 37/453 mood exclusions are missing | Claim: (silent) · Reality: `IsNotReallyTownNPC` (`ShopHelper.cs:363-371`) covers the Old Man (37) and type 453; the Rust checks only the Travelling Merchant | SILENT |
| 29-4 | behavioural | The "one tenant per house" rule is missing | Claim: (silent) · Reality: `CanNPCsLiveWithEachOther` / `IsRoomConsideredAlreadyOccupied` (the `housingCategory` comparison) is absent and `Npc` has no `housing_category` field → two incompatible NPCs can be assigned the same home spot | SILENT |
| 29-5 | behavioural | The mid-game relocation is missing | Claim: (silent) · Reality: `MoveTownNPC` (`WorldGen.cs:4685`) / `moveRoom` (4843) re-home an NPC when its house is destroyed; the Rust only has the **load-time** healing (`town.rs::fix_town_npc_homes`) → a player who breaks a house mid-session keeps a stale home | SILENT |
| 29-6 | behavioural | `lookForHomeTimeout` is missing | Claim: (silent) · Reality: the C# only lets a homeless NPC move in when its timeout has expired (`WorldGen.cs:5621`); the Rust's `quick_find_home` has no cooldown → the move-in cadence differs | SILENT |
| 29-7 | behavioural | The pylon duplicate-type gate is missing | Claim: (silent) · Reality: the vanilla refuses a second pylon of the same type at placement and kills a net-placed duplicate (`TETeleportationPylon.cs:13-27`, `153-161`) | SILENT |
| 29-8 | behavioural | The chatter/dialogue layer is missing | Claim: (silent; research/05 lists `CheckDialogue` as an un-ported `UpdateNPC` phase) · Reality: `NPC.GetChat` (93766), `ConditionalDialogue` (6539/90519) and the ten chatter tables (93976-94794) have no Rust counterpart → the town NPCs are silent | SILENT |
| 29-9 | behavioural/cosmetic | The happiness report text is missing | Claim: (silent) · Reality: the C# shows a localised `TownNPCMood_<Name>.<key>` string plus the shop-hud icon (`Main.cs:41792-41797`); the Rust carries `MoodFlag`s but no report text → the "why is this NPC expensive" feedback is absent | SILENT |
| 29-10 | behavioural | The Princess's mood branch is missing | Claim: (silent) · Reality: `ShopHelper.cs:122-131` gives the Princess a hard gate (≥2 house & ≥2 village neighbours, else the 1000 sentinel + `HateLonely`); the port has no Princess kind | SILENT |
| 29-11 | cosmetic/determinism | The `home`/`homeless` collapse | Claim: (silent) · Reality: the C# keeps `homeTileX/homeTileY` **and** `homeless` **and** the legacy `oldHomeTile*` pair (`NPC.cs:6551-6561`); the Rust models the first two as one `Option<(i32,i32)>` → a 1.4.3 world loaded into the port keeps no fallback home | SILENT |
| 29-12 | cosmetic | Citation drift in the ROADMAP | Claim: the M4-TOWN-1 row cites `RoomNeeds` as `:5717-5756` and `MoveTownNPC 4675` · Reality: in the current decompilation `RoomNeeds` is at `WorldGen.cs:5679-5730` and `MoveTownNPC` at 4685 | CLAIM-OTHERWISE |

### 31 — golf (4)

| Source | Severity | One-line risk | Claim vs reality | Status |
| --- | --- | --- | --- | --- |
| 31-3 | behavioural | Holding a club does nothing | Claim: (silent) · Reality: the clubs are in `itemdata.rs` with `use_style: 8` (GolfPlay) and `projectile: Some(722)`, but there is no projectile 722 → the swing never fires | SILENT |
| 31-4 | behavioural + data | There is no `TileMaterial`/`TileGolfPhysics` in Rust | Claim: (silent) · Reality: even a faithful port of `BallCollision` needs the per-material dampening constants; `assets/` has no `*TileMaterial*` JSON → the port cannot be verified 1:1 without that data | SILENT |
| 31-5 | behavioural | The golf tiles are absent from `tiledata` | Claim: (silent) · Reality: tiles 476/477/492/493/494 do not appear in `crates/tr-core/src/tiledata/**` → the tee/cup/grass cannot be placed or drawn even though the *items* that place them are registered | SILENT |
| 31-7 | medium risk | The course geometry has no identified generator | Claim: (silent) · Reality: `WorldGen.cs` contains only `CheckGolf1x1` (`:45260`), `Main.LocalGolfState.WorldClear` (`:7001`) and the `TileID.Conversion.GolfGrass` conversion arms — no pass builds a course → a port must ship the course as data or find the placement path | SILENT |

### 32 — sound / audio (7)

| Source | Severity | One-line risk | Claim vs reality | Status |
| --- | --- | --- | --- | --- |
| 32-D2 | behavioural | There is no per-style volume | Claim: (silent) · Reality: `LegacySoundStyle.Volume` / `WithVolume` / `WithVolumePerceived` (`LegacySoundStyle.cs:65-73`, the `pow(v,1.6609)` curve) are folded into `SoundEngine.cs:99`; the port has a flat `volume_scale` and the two blizzard loops are the only styles with a non-1 volume, carried in a special table (`audio.rs:84-85`) | SILENT |
| 32-D3 | behavioural | There are no variation families | Claim: (silent) · Reality: `LegacySoundStyle.cs:20-30` picks `[style, style+Variations)` per play (e.g. `NPC_Hit_1..3`, `Dig_0..2`, `Splash_0..3`); the port pins one asset per cue | SILENT |
| 32-D4 | behavioural | There is no sound instance pool | Claim: (silent) · Reality: the C# pre-creates one instance per family and *stops then re-creates* it per play (`:445-447`), so a second dig cuts the first off; the port spawns a fresh rodio source per play → overlapping cues stack | SILENT |
| 32-D8 | behavioural | No `isGeneratingOrLoadingWorld` / `netMode == 2` guard | Claim: (silent) · Reality: `LegacySoundPlayer.cs:382-389` blocks playback in both cases; `sound.rs::play_sound` has neither → a positional cue can fire while the world generates | SILENT |
| 32-D9 | behavioural (minor) | The music `PlayCallbacks` / `TrackLoopCounts` hook has no port | Claim: (silent) · Reality: `LegacyAudioSystem.cs:308-311`, `:341` is absent and `update_common_track_toward_stopping` uses "a sink exists" instead of the C#'s `IsPlaying || !IsStopped` (`:326`) | SILENT |
| 32-D11 | behavioural | Custom music packs (`.mp3`/`.ogg`/`.wav`) are unsupported | Claim: `PORT_MAP.md:241` says only "partial" · Reality: `LegacyAudioSystem.FindReplacementTrack` (`:82-122`) plus the three track classes load `Music/<name>.{mp3,ogg,wav}`; the port reads only the XACT banks and never reads the `.xgs` engine file | SILENT |
| 32-D17 | cosmetic (docs) | The ROADMAP/PORT_MAP summary rows are stale | Claim: `ROADMAP.md:78` "front-end music and UI sounds missing (BUG-01/17)" and "the 8-cue bank" · Reality: both were fixed on 2026-11-10 and the table now holds 26 entries (13 played of 527 shipped `.xnb`); `PORT_MAP.md:241` likewise predates BUG-01/17 | CLAIM-OTHERWISE |

### 34 — ID sets (13)

| Source | Severity | One-line risk | Claim vs reality | Status |
| --- | --- | --- | --- | --- |
| 34-1 | behavioural/determinism, high | The `NpcKind` protocol id space collides with itself on 7 ids (78..84) | Claim: (silent) · Reality: `to_u16` (`npc.rs:850`) maps **two** kinds onto each of 78..84 (the pirate band at `:887-893` vs the Deerclops/Skeletron-Prime band at `:1044-1050`); `from_u16` resolves each id to its first arm → 7 round-trip failures. `to_u16`/`from_u16` are on the live **save** path (`app/saveglue.rs:107` writes, `app/mod.rs:1980` reads) and the live **netcode** path (`app/netglue.rs:385`, `:1044`, `:1504`, `:2060`) → a saved Deerclops comes back as a Pirate Deckhand | SILENT |
| 34-2 | cosmetic (id-space trap) | `to_u16` still carries 31 conflicting arms; `from_u16` 7 | Claim: the code comment claims the cleanup landed · Reality: 29 dead duplicate arms in `to_u16` (`SolarCrawltipedeHead` ×4 at `:953`/`:958`/`:963`/`:968`, `SolarSpearman` ×4, `StardustWormHead` ×4, …) plus 2 contradictory ones (`DD2LanePortal ⇒ 549` at `:990` vs `⇒ 147` at `:204`); `from_u16` has 21 dead literal arms and 7 contradictory ones. Rust resolves first-arm-wins, so behaviour is currently the "early" value — but the *id-collision* it hides (deviation 1) is behavioural | CLAIM-OTHERWISE |
| 34-3 | behavioural | The ROADMAP/ANALYSIS claim that the dead arms were removed is false | Claim: `ROADMAP.md:1469` "the duplicated dead arms removed" and `ANALYSIS.md:3024-3027` "the duplicated dead arms removed" · Reality: the renumber landed but **31 conflicting / 29 dead arms remain in `to_u16` and 7 contradictory / 21 dead arms remain in `from_u16`**, and the renumber *created* the 78..84 collision — the DONE row's verdict is **FAIL** | CLAIM-OTHERWISE |
| 34-4 | behavioural | `cs_id` covers 43 of 182 kinds | Claim: (silent) · Reality: `npc.rs:1789` maps Slime…Shark and returns `u16::MAX` for the remaining 139; the doc comment (`:1784-1788`) says the map is what the statue spawns' ids, the `MechSpawn` caps (`NPC.cs:7420`) and the teleporting statues' rosters (`Wiring.cs:2424`/`:2452`) read → for 139 kinds those vanilla-id consumers can never match | SILENT |
| 34-5 | behavioural | 11 kinds are unreachable through `from_u16` | Claim: (silent) · Reality: the protocol ids 85-88 (`BrainScrambler`, `RayGunner`, `ScutlixRider`, `Scutlix`) and 162-168 (`TaxCollector`, `Golfer`, `SleepingGolfer`, `Zoologist`, `Princess`, `DyeTrader`, `PartyGirl`) have no `from_u16` arm → a net message or save record carrying one of those ids decodes to `None` and the NPC is dropped | SILENT |
| 34-6 | structural, high | There are three parallel NPC id spaces | Claim: (silent) · Reality: the vanilla `NPC.type` is reachable through `net_id` (`npc.rs:1512`), the port's protocol id through `to_u16`, and a third partial map through `cs_id`; the `.wld` save keys on `net_id` and the app save on the protocol `u16` → any future consumer that picks the wrong one silently addresses the wrong id (this is the root cause of deviation 1) | SILENT |
| 34-7 | behavioural | The bestiary table (174 entries) is smaller than the kind roster (182) | Claim: (silent) · Reality: `net_id` is resolved through `bestiary::entry_for_npc` (`bestiary.rs:1482`), so at least 8 kinds have no entry row and their `net_id` resolution path is not covered by the port's own tests | SILENT |
| 34-8 | behavioural | Prefix ids 62..80 are unmodelled | Claim: (silent) · Reality: `stat_switch` (`prefix.rs:196-275`) covers 1..61 and 81..97; ids 62..80 (`Hard` … `Violent`, `PrefixID.cs:130-200`) hit the `_ =>` identity arm → a melee weapon rolled with, say, `Hard` (62) gets the vanilla melee damage bonus in C# and **nothing** in Rust | SILENT |
| 34-9 | behavioural, medium-high | `ProjectileID` is sparsely populated | Claim: ROADMAP tracks the *families* (M4-PROJ-*), not the id table · Reality: 213 id constants for 1136 vanilla ids and 174 `proj_data` arms; an unlisted projectile id spawns with no defaults (no size, no `aiStyle`, no `timeLeft`, no `penetrate`) — a silent behavioural difference for ~950 ids | SILENT |
| 34-10 | behavioural | `DustID` (332) has no Rust carrier | Claim: (silent) · Reality: only two literals (`rain.rs:96`, `tr-core/src/waterfall/mod.rs:54`) | SILENT |
| 34-11 | behavioural | The `TileID.Sets.*` family is only partly generated | Claim: (silent) · Reality: ~45 sets in `tiledata/sets.rs` against 100+ in `TileID.cs`; the bitset bounds-checks against `TILE_COUNT` so a missing set degrades to "not in the set" — safe, but wrong | SILENT |
| 34-12 | behavioural, high | `GameModeID` (4) and the whole difficulty system are absent | Claim: (silent) · Reality: no expert/hardmode flags exist in the sim, so every `GameModeID`-keyed vanilla table has no Rust counterpart (cross-cutting; tracked by module 05) | SILENT |
| 34-13 | cosmetic/pixel | `ItemRarity`'s 12 colours and `SurfaceBackgroundID`'s 16 backgrounds have no Rust tables | Claim: (silent) · Reality: the Rust carries `ItemDef.rare` but no rarity→colour map | SILENT |

### 35 — RNG utils (7)

| Source | Severity | One-line risk | Claim vs reality | Status |
| --- | --- | --- | --- | --- |
| 35-2 | determinism | The `Main.rand` call ORDER is not modelled at all | Claim: (silent) · Reality: in the C# a single stream serves the whole game and the order of the draws is the contract; the Rust splits it into seven fields, so there is no "order" to preserve and no way to align with vanilla without the type | SILENT |
| 35-3 | determinism | The `Main.SwapRandom` named streams are not ported | Claim: (silent; ANALYSIS.md:4420-4430 admits only the LCG) · Reality: the C#'s per-stage streams are all seeded with the *same* world seed and diverge only by draw count; the Rust's per-consumer streams use *different* seeds (`world_seed ^ 0x5DEECE66D1A49B5F`, `^ 0x0A5CE17A1C1E5EED`, `^ 0x1DA91750E5C10C0C`, `^ 0x2545F4914F6CDD1D`, `craft_rng: 1`) — the `craft_rng: 1` constant is not seed-derived at all (`app/mod.rs:2700`) | SILENT |
| 35-4 | behavioural | The `WorldGen.genRand` alias is not modelled | Claim: (silent) · Reality: `WorldGen.cs:4445` returns `Main.rand`, so `Item.Prefix` during world generation draws from the *same* stream as the worldgen; the Rust's `prefix.rs` uses an isolated splitmix → the prefix roll cannot drift the world (a divergence, but a *safe* one) | SILENT |
| 35-5 | determinism (documentation) | `deterministic_for_seed` is a self-consistency test, not a fidelity proof | Claim: `ROADMAP.md:335` asks to "keep `deterministic_for_seed` FNV proof" · Reality: `deterministic_for_seed` (`worldgen/mod.rs:259-276`) generates the same world twice and asserts element-wise equality — it contains **no** FNV and **no** vanilla comparison; the FNV-ish fold in `world_hash.rs` is the port's own mix (`world_hash.rs:9-13`) re-pinned ~20 times | CLAIM-OTHERWISE |
| 35-6 | cosmetic | The `spawner.rs` doc comment says "xorshift64\*" but the code is the splitmix64 finaliser | Claim: (silent) · Reality: `spawner.rs:28-43` vs `prefix.rs:432-440`, which calls the same shape "the splitmix step" → misleading for the next reader | SILENT |
| 35-7 | behavioural (rare) | `Utils.Clamp`'s argument-order semantics | Claim: (silent) · Reality: C# `Clamp` returns `max` when `value > max` even if `min > max` (`Utils.cs:223-234`); Rust's `Ord::clamp` **panics** when `min > max` | SILENT |
| 35-8 | pixel | `Math.Round` is banker's rounding | Claim: (silent) · Reality: `Utils.cs:112-115` is banker's; Rust's `f32::round` is half-away-from-zero | SILENT |

### 37 — ReLogic (6)

| Source | Severity | One-line risk | Claim vs reality | Status |
| --- | --- | --- | --- | --- |
| 37-1 | behavioural (accepted by design) | There is no asset repository in the port | Claim: ROADMAP/ANALYSIS does not spell it out for this subsystem · Reality: the C# loads every asset lazily through `AssetRepository.Request<T>` (`AssetRepository.cs:82`) with resource-pack overlays and hot reload; the Rust loads a fixed sheet list at boot → no mods/resource packs, no lazy loading, different memory profile | SILENT |
| 37-2 | behavioural | The XNB/LZX/DXT stack is a reimplementation, not a port | Claim: mitigated by pixel-hash tests only · Reality: since ReLogic delegates to FNA (`XnbReader.cs:10-33`) there is no C# in `decompiled-libs` to port 1:1; formats not exercised by a test (multi-mip, unknown surface formats — the 23 failures in `ANALYSIS.md:1408-1411`) can regress silently | SILENT |
| 37-5 | behavioural/visual | `.fx` effects are only name-parsed | Claim: (silent) · Reality: the Rust screen filter mirrors `ScreenShader.fx`'s named techniques with a WGSL module and a 22-entry `PASS_TABLE` (`ANALYSIS.md:1106`); it does not execute the compiled effect → visual parity is not guaranteed per pass | SILENT |
| 37-6 | behavioural (UI) | Text wrapping is not ported | Claim: not mentioned in ANALYSIS · Reality: `ReLogic.Text` has no Rust counterpart (no `wrap` helper in `crates/`) → long labels/tooltips will not break like the original | SILENT |
| 37-8 | cosmetic | Stale/incorrect doc citations | Claim: `PORT_MAP.md:250` cites `ReLogic/Content/Lzx*` · Reality: no such file exists (the decoder came from MonoGame/Xbox); `ANALYSIS.md:1406-1411` still describes the C# PNG-extraction pipeline and the "15 100 / 15 123" count; `INDEX.md:62` labels the module "(content pipeline, XNA, JSON)" although ReLogic contains neither a JSON serializer nor the XNA types | CLAIM-OTHERWISE |
| 37-9 | cosmetic | Asymmetric evidence in this checkout | Claim: the Rust `xnb.rs` header comment cites `tools/XnbExtract/Program.cs` · Reality: `tools/` here contains only `GenTileData/Program.cs` → the citation cannot be re-verified inside this audit tree | SILENT |

### 40 — DONE re-verification (10)

| Source | Severity | One-line risk | Claim vs reality | Status |
| --- | --- | --- | --- | --- |
| 40-1 | behavioural, high | The Hornet's hover constants are wrong | Claim: `ROADMAP.md:1409` (M4-NPC-4) — "the Hornet (3.25/0.018, `:50322-50323`)" · Reality: `npc_ai.rs::HoverParams::HORNET` (177-181) = 3.25/0.018 but C# `NPC.cs:50341-50344` = 3.5/0.021 for type 42 — the port ships the **type-205** values (`NPC.cs:50366-50367`) | CLAIM-OTHERWISE |
| 40-2 | behavioural, high | The hover presets' C# citations are all off | Claim: the same ROADMAP row cites `:50304-50305` / `:50322-50323` / `:50328-50329` · Reality: `:50322-50323` = type **619** arm (6/0.1); `:50328-50329` = type **252**; the Meteor Head is at `:50374-50377` (1/0.03 ✓) → the roadmap row is self-invalidating: it cannot be re-verified by following its own anchors | CLAIM-OTHERWISE |
| 40-3 | behavioural, medium | The player-side damage floor is missing | Claim: `ROADMAP.md:1353` (M2-COMBAT-1) is `✅` · Reality: `Player.cs:38378-38381` floors `num2` to ≥ 1 after the endurance multiply; `player.rs::hurt_inner` subtracts without the clamp and its comment asserts the opposite of the source → a 1-damage hit with enough endurance deals 0 in the port and 1 in vanilla | CLAIM-OTHERWISE |
| 40-4 | behavioural, medium | The bow/arrow branch of `GetWeaponDamageMultiplier` is missing | Claim: `ROADMAP.md:1353`/`M2-COMBAT-2` is `✅` · Reality: `Player.cs:53396-53398` → `bowEffectiveDamage` is absent (`AmmoID.Sets.IsArrow` → `bowEffectiveDamage`) | CLAIM-OTHERWISE |
| 40-5 | pixel/behavioural | `behindTiles` is not ported at all | Claim: the ROADMAP's M4-BOSS-6 `SetDefaults` citation lists the flag as if it were ported · Reality: C# `NPC.cs:10355` (the WoF) sets it, and the EoW / Destroyer segments and the boss parts rely on it; the port has no `behind_tiles` field → the Wall of Flesh draws in front of the world | CLAIM-OTHERWISE |
| 40-6 | doc | The ROADMAP contradicts itself on the display rows | Claim: `ROADMAP.md:73` says "🟡 the display rows M4-NPC-3c/3h..3r" while `ROADMAP.md:1534`/`:1540-1550` mark the same ids `✅` · Reality: nothing states which is current | CLAIM-OTHERWISE |
| 40-7 | doc | Citation drift in the rows | Claim: `Main.cs:64482-64511` (the method starts at `:64435`) and `Player.cs:53386` (the method starts at `:53388`) · Reality: a "cited range" lint would have caught both, and the M4-NPC-4 class of error | SILENT |
| 40-8 | doc | The M1.5 performance tables are measurement claims with no artefact | Claim: `ROADMAP.md:1181-1226` ("pixel-identical (0 differing pixels)" + the timing tables) · Reality: no code artefact to re-read; the neighbouring Phase 3 table (`:1199-1205`) is still "in progress", so the section is a mixed state | SILENT |
| 40-9 | cosmetic | `DamageVar`/`ItemDropResolver` are types in C# and functions in Rust | Claim: the roadmap text reads as if a struct exists · Reality: harmless naming divergence, but the text overstates | SILENT |
| 40-10 | cosmetic | The `M4-PROJ-4c`/`M4-PROJ-6b` naming | Claim: (silent) · Reality: `render` vs `leafblower` vs `magic-followups` makes the catalog harder to scan (the rows are consistent inside themselves) | SILENT |

### 41 — render pipeline (13)

| Source | Severity | One-line risk | Claim vs reality | Status |
| --- | --- | --- | --- | --- |
| 41-D1 | documentation | "2 draw calls" in the ROADMAP is wrong | Claim: `ROADMAP.md:64` "2 draw calls; G6 cache; zoom 1.0–2.0" · Reality: `sprite.rs::draw_ranges` issues **one `pass.draw` per non-empty range** (1195-1239) → up to 6 per frame, and with `TR_POST=1` the frame is 3 + 3 plus 1..4 filter passes (`post.rs::MAX_CHAIN`) | CLAIM-OTHERWISE |
| 41-D2 | behavioural+pixel, medium | One blend state for the whole frame | Claim: (silent) · Reality: the vanilla uses at least four blend configurations (`AlphaBlend` `Main.cs:22193`, `Additive` `:27490`, the custom `ReverseSubtract` multiply `:27462-27473`, and `null`→default `:40415`); the port has exactly one (`sprite.rs:327-338`) → Lightsbane's additive glow and the multiply pass cannot be reproduced, and the *premultiplied* convention it relies on is asserted in `sprite.wgsl:234-238` but the XNA `BlendState.AlphaBlend` definition is not in this repo's decompile | SILENT |
| 41-D3 | pixel, medium | The atlas merges textures the vanilla keeps separate | Claim: (silent) · Reality: vanilla batches per texture (`TileBatch.cs:485`) and `SpriteBatch` in `Deferred` mode additionally sorts by texture, while the port preserves push order → overlapping translucent sprites from different sheets can composite in a different order; the project's own `ROADMAP.md:1259` (B10) quantifies "flips 166 background pixels at the forest horizon" | SILENT |
| 41-D4 | pixel, low | Nearest sampling across a packed atlas vs per-texture PointClamp | Claim: (silent) · Reality: the atlas has `PAD = 2` transparent padding (`atlas.rs:9`) and the shader insets by half a texel; vanilla samples the exact texel of its own sheet → any sprite whose source rect abuts a mip/pad boundary can bleed one texel of a neighbour | SILENT |
| 41-D5 | pixel, medium | The tint and recolour maths happen in the wrong colour space | Claim: (silent) · Reality: `sprite.wgsl:231` uses `pow(tint, 2.2)` — an approximation of the sRGB EOTF — so `srgb_decode(tex) * pow(tint,2.2)` ≠ `srgb_decode(tex_srgb * tint_srgb)`; the vanilla multiplies **bytes** in sRGB space with integer division, which truncates (the project already records a 1-LSB divergence at `ANALYSIS.md:424-425`). `paint_recolor` (`:158-170`) and `dye_recolor` (`:181-195`) wash **linear** samples with sRGB-authored constants | SILENT |
| 41-D6 | pixel, low | The rasteriser culling is not reproduced | Claim: (silent) · Reality: the vanilla flips `Main.Rasterizer` between `CullCounterClockwise` (56690) and `CullClockwise` (62006) and uses `RasterizerState.CullNone` for the immediate batches; the port is `cull_mode: None` everywhere (`sprite.rs:345`) | SILENT |
| 41-D7 | behavioural, medium | The `MoonLordShake` filter is inert | Claim: (silent; the ROADMAP row admits "towers/monoliths/Aurora/Sepia OUT") · Reality: `Main.cs:64976-64984` activates the filter during the Moonlord fight and `post.rs:147` maps it to pass index 0 (passthrough) → the screen shake never happens; `Main.UseScreenShake` is ported as a settings flag that gates nothing | SILENT |
| 41-D8 | pixel, medium | The zoom model differs from `GameViewMatrix` | Claim: `ROADMAP.md:64` "zoom 1.0–2.0" · Reality: the port snaps the camera to the zoom grid (`frame.rs:817`) and clamps zoom in the settings loader only (`settings.rs:446`); `Main.ForcedMinimumZoom` (`Main.cs:243`, assigned at `:61991`/`:61994`) has **no located equivalent** → the port renders a wider world view than vanilla on wide screens | CLAIM-OTHERWISE |
| 41-D9 | behavioural+pixel, medium | Scissor clipping is absent | Claim: (silent) · Reality: the vanilla saves/restores `GraphicsDevice.ScissorRectangle` around scissored batches (`Main.cs:25403-25424`, `:40406-40414`, `Main.ScissorState` at `:2329`); the port has no scissor anywhere (`grep set_scissor` over `crates/` returns nothing) → clipped UI regions bleed or are skipped | SILENT |
| 41-D10 | pixel, low | The linear-clamp sampling paths are unavailable | Claim: (silent) · Reality: `SamplerState.LinearClamp` (`Main.cs:45025`, `48204`, `55613`) and `Main.SamplerStateForCursor` (`:45257`) have no equivalent; the port's single nearest sampler is used everywhere | SILENT |
| 41-D11 | pixel, medium | Order-only batching ignores XNA's `Deferred` sort | Claim: (silent) · Reality: the vanilla's `SpriteSortMode.Deferred` batches group sprites by texture and sort by depth within a batch; the port preserves push order into one buffer → for overlapping translucent sprites the composite order can differ | SILENT |
| 41-D13 | behavioural, medium | Twenty of 29 scene filters are inert | Claim: the ROADMAP row admits "towers/monoliths/Aurora/Sepia OUT" · Reality: `post.rs::PASS_TABLE` (136-175) gives pass index 0 to Nebula/Solar/Stardust/Vortex, the four Monolith rows, MoonLord(+Shake), testInvert, Sepia, the three Crystal rows, Test/Test2/Test3 and CRT — the **MoonLord shake** is not in the admitted-out list | SILENT |
| 41-D14 | behavioural, low | `alpha_mode` is taken blindly from the adapter | Claim: (silent) · Reality: `gpu.rs:105` takes `caps.alpha_modes[0]`, which may be `PreMultiplied` on some platforms; the vanilla's backbuffer is opaque | SILENT |

### 42 — save/load (8)

| Source | Severity | One-line risk | Claim vs reality | Status |
| --- | --- | --- | --- | --- |
| 42-1 | behavioural | The `.wld` tile section drops the four wire bits | Claim: `ROADMAP.md:1608` (M6-SAVE-1) reads "all 11 `.wld` sections read + written" · Reality: the port keeps wires in a side table (`World.wires`) and its `.wld` reader never restores them (`wld/sections/tiles.rs:103-105`); the C#'s batching equality (`Tile.isTheSameAs:155-197`) compares the wire bits → the **run boundaries** differ too, so the byte divergence is not only in the header bits. *The row's own tail lists "wires-not-written", so the gap is admitted in the row body but the headline over-claims.* | CLAIM-OTHERWISE |
| 42-2 | behavioural | The `.wld` world header and flags write placeholder values | Claim: the ROADMAP row does not mention them · Reality: generator version 0, GUID zeros, world id 0, `LastPlayed` = min date, the secret-seed booleans `false` (`sections/header.rs:49-60`, `sections/flags.rs:220-274`) → a vanilla client reads a different seed/GUID/world id | SILENT |
| 42-3 | behavioural | There are no rolling backups, no `.bak` and no post-write validation | Claim: (silent) · Reality: the C# keeps up to 9 numbered backups (`DoRollingBackups:966-1004`) and re-reads the file to validate it, restoring the `.bak` on failure (`InternalSaveWorld:941-962`); the port writes tmp+rename only (`wld::save:319-321`) → a corrupted write is not rolled back | SILENT |
| 42-4 | behavioural (interop) | The `.plr` writer is a subset mirror of `Player.Serialize` | Claim: (silent) · Reality: only the sanitized name + starter items are written → the port's `.plr` is not byte-comparable with a vanilla one; a vanilla client would read a partial character | SILENT |
| 42-5 | behavioural | Cloud save is missing | Claim: (silent) · Reality: the C# writes `Main.CloudWorldPath`/`Main.CloudPlayerPath` through `FileUtilities.Write(..., useCloudSaving)` → `SocialAPI.Cloud.Write` (`WorldFile.cs:940`, `Main.cs:4820-4870`); the port writes a local path only — this should be an explicit ROADMAP non-goal, not an open row | SILENT |
| 42-6 | behavioural | Autosave cadence parity is unverified | Claim: the module doc claims the C#'s 600 s branch (`Main.cs:19751-19765`) · Reality: the 300 s single-player `saveToonWhilePlaying` branch (the **player-only** save, `Main.cs:19743-19749`) was not verified → the player may be saved less often than the vanilla | SILENT |
| 42-7 | cosmetic/architectural | The port has two world formats | Claim: (silent) · Reality: `.wld` vanilla-shaped and `.trwld` native, chosen by file extension at save time (`saveglue.rs:27-28`) → the save surface doubles for Stage-2 (two compressors, two readers) | SILENT |
| 42-8 | unknown | The C#'s "temp" save snapshot is not verified | Claim: (silent) · Reality: the C# snapshots day/time/weather into `_temp*` and restores it on load (`SetTempToOngoing` 1040-1066 / `SetOngoingToTemps` 152-179); the Rust flags writer emits the world's own values and the exact temp mechanism was not checked | SILENT |

### 26 — projectile AI families (8)

| Source | Severity | One-line risk | Claim vs reality | Status |
| --- | --- | --- | --- | --- |
| 26-2 | determinism (maintenance) | Six duplicated match arms | Claim: (silent) · Reality: aiStyle 1 (10432 wins / 10615 dead), 3 (10439 / 10616), 14 (10503 / 10617), 16 (10504 / 10618), 115 (10619 / 10826), 99 (10678 / 10884) — Rust matches the **first** arm, so the later copies are dead code; the 99 pair is byte-identical | SILENT |
| 26-3 | behavioural | The Vortex ritual is never dispatched | Claim: M4-EVENT-17 and PORT_MAP 236 both claim "done" · Reality: `ai_vortex_ritual`/`ai_vortex_bolt` exist and are unit-tested but **no dispatch arm calls them** — only the test module does (15771-15839) → the Vortex ritual projectiles never animate in the live sim | CLAIM-OTHERWISE |
| 26-4 | behavioural | The Flairon's bubble has no arm and no `ai_bubble` function | Claim: PORT_MAP 157 lists `ai_bubble` as ported · Reality: no arm and no `ai_bubble` anywhere in `crates/`; the style-70 row (FLAIRON_BUBBLE 405) has data but no behaviour | CLAIM-OTHERWISE |
| 26-7 | behavioural | `AI_001`'s ~137 remaining per-type arms are absent | Claim: (silent) · Reality: C# `AI_001` has **142 distinct per-type arms** (197 `type ==` checks at 54002-57826); the port covers only the 4 M4-PROJ-6 riders + the Book of Skulls → one generic arrow flight covers 32 arm groups / 36 kinds | SILENT |
| 26-9 | behavioural | aiStyle 99: the counterweights are absent | Claim: (silent) · Reality: `AI_099_1_Counterweights` (types 556-561/1079, `Projectile.cs:33153`) has neither a `proj_data` row nor a dispatch arm | SILENT |
| 26-10 | behavioural/structural | `ai_shuriken` is a Rust-invented approximation | Claim: (silent) · Reality: the port's shuriken (3610) is a 2 half-step / 0.6-bounce stand-in for the C# aiStyle-2 block (23205-23689), a 480-line per-type machine covering ~40 vanilla types; it is gated by **kind** (10385) and has no `proj_data` row, so it bypasses the aiStyle table entirely | SILENT |
| 26-11 | behavioural | 21 spawnable styles have a data row but no dispatch arm | Claim: (silent) · Reality: styles 2, 23, 26, 66, 67, 70, 79, 80, 88, 108, 120, 123, 130, 133, 134, 137, 138, 157, 158, 187, 189 hold position and never animate, damage or expire | SILENT |
| 26-12 | behavioural | The aiStyle-84 arm is kind-gated | Claim: (silent) · Reality: one vanilla style now serves two unrelated C# families — LAST_PRISM_LASER vs the Phantasmal Deathray (10635-10643) | SILENT |

### 27 — mounts / pets / minecart / grapple (12)

| Source | Severity | One-line risk | Claim vs reality | Status |
| --- | --- | --- | --- | --- |
| 27-D1 | documentation | The "12 rows" story is stale | Claim: ANALYSIS "Mounts framework (M2-MOUNT-1)" says *"a 12-row ground table"* / *"Partial table (12 of `MountID.Count` 66)"`, ATLAS:1364 says *"the 12-row `MountData` table"*, PORT_MAP:96 says *"the 12-row ground table"* · Reality: the code now carries **50 rows** (`mount.rs:1305-1356` = 12 ground + 11 flyer + 27 cart) — the deviation list is right in substance but its count is wrong | CLAIM-OTHERWISE |
| 27-D2 | behavioural | `can_mount` drops the Cart/`CanUseHooks` grapple exemption | Claim: (silent) · Reality: C# `Mount.cs:6599-6602` returns false only when `!Cart[m] && !CanUseHooks[m] && grappling[0] >= 0`; the Rust (`player.rs:4665-4667`) returns false for **any** grappling mount, and its comment ("no row in this table is in the CanUseHooks set") is true for `CanUseHooks {54,57,58,59,60}` but **false for `MountID.Sets.Cart`** — 27 cart rows are present, so vanilla allows mounting a minecart while a hook is out and the port refuses it | SILENT |
| 27-D3 | behavioural | `can_mount` drops the bat-wet rule | Claim: (silent) · Reality: C# `Mount.cs:6595-6598` (`m == 56 && (wet \|\| dripping \|\| WetCollision(...))` → cannot mount); the Rust has no such arm and its comment is **false** since row 56 (Bat) is in the table (`mount.rs:1022`) | SILENT |
| 27-D4 | behavioural | The Fishron halving is missing | Claim: (silent) · Reality: C# `Player.cs:26655-26659` halves `runAcceleration`/`maxRunSpeed` for `mount.Type == 12`; the Rust (`player.rs:4776-4779`) skips it with the comment *"have no rows in this table yet"* — row 12 (`CUTE_FISHRON`) **is** in the table (`mount.rs:865`) → the CuteFishron runs at full speed | SILENT |
| 27-D5 | behavioural | `SetMount` per-type extras are missing | Claim: (silent) · Reality: C# `Mount.cs:6228-6235` (type 44 Pirate Ship: `velocity *= 0.2f` + dash reset), `:6279-6282` (types 7/8: `fullRotationOrigin = centre`), `:6287-6298` (`_mountSpecificData` for 8/35/54), `:6283-6286` (the PalChillet sound for 62-65), `:6301-6304` (the type-38 achievement); the Rust `set_mount` covers only the guards, the fly-time reset, the fatigue max, `_flipDraw` and the 9/46 ability cooldowns. Rows 44, 7, 8, 35 exist → the first three are reachable today | SILENT |
| 27-D6 | behavioural | `Dismount` housekeeping is missing | Claim: (silent) · Reality: C# `Mount.cs:6152-6155` clears `noItems` for types 55/56 and `:6161-6162` resets `fullRotation`/`fullRotationOrigin`; the Rust restores the hitbox and the buff only. Row 56 (Bat) exists | SILENT |
| 27-D7 | behavioural | `CanDismountWithResult`'s CC gate is missing | Claim: (silent) · Reality: C# `Mount.cs:6615-6618` returns `FailedCCed` for `DontDismountWhenCCed {55,56,61}`; the Rust only probes the space. 55 is un-equippable but 56 (Bat) and 61 (Pixie) are not | SILENT |
| 27-D8 | behavioural | `QuickMount` diverges | Claim: (silent) · Reality: C# `Player.cs:6190` skips `MountID.Sets.Cart` items and falls through to `QuickMinecart()` (:6097), and gates on `Mount.DismountsOnItemUse(item.mountType) && TryingToUseItem()` (:6082-6085); the Rust takes *any* inventory slot with a known `mount_type` — the 26 cart items are in `MOUNT_ITEMS`, so a minecart item is mounted through the normal path. The Rust comment ("the cart items are skipped by the table itself (no cart rows)") is stale | SILENT |
| 27-D9 | behavioural | 16 mount ids are absent | Claim: admitted in ANALYSIS ("partial table") but with a stale count · Reality: missing rows 4 (Turtle), 17 (Golf Cart), 37 (Flamingo), 45 (Spooky Wood), 46 (Santank), 52 (Wolf), 54 (Velociraptor), 55 (Rat), 57-60 (roller skates), 62-65 (Chillet); `set_mount` returns `None` for them (`mount.rs:1814`) → the saddles do nothing | SILENT |
| 27-D12 | cosmetic | `MountState::check_mount_buff` is dead code | Claim: (silent) · Reality: `mount.rs:1861-1863` has no caller; the equivalent check lives inline at `player.rs:2550` | SILENT |
| 27-D13 | determinism | The minecart ride block is relocated | Claim: (silent) · Reality: C# puts the cart/`OnTrack` branch *inside* the mount merge block (`Player.cs:26665-26682`, before `HorizontalMovement`); the Rust runs it in the separate cart path (`player.rs:3116-3500`) → the `runSlowdown = runAcceleration` assignment is ordered differently relative to the movement merge | SILENT |
| 27-D14 | behavioural | `FrameTrack` is missing | Claim: (silent) · Reality: C# places track tiles from the item use (`Player.cs:46058` → `Minecart.FrameTrack`); no `FrameTrack` symbol exists in `minecart.rs` | SILENT |

### 30 — achievements / bestiary (9)

| Source | Severity | One-line risk | Claim vs reality | Status |
| --- | --- | --- | --- | --- |
| 30-1 | behavioural | `CreateMany` is collapsed | Claim: (silent) · Reality: `STAR_DESTROYER` 4→0 conditions (so it can never unlock and its `is_completed()` is vacuously true), `GELATIN_WORLD_TOUR` 24→1, `SLAYER_OF_WORLDS` 15→3, `BUCKETS_OF_BOLTS` 3→1; the port's own comment at `achievements.rs:162-163` calls the zero-condition `STAR_DESTROYER` "complete from the start, exactly as vanilla" — vanilla needs all four pillars | SILENT |
| 30-2 | behavioural | `achievements::save` has no call site | Claim: ROADMAP M6-ACH-1 says "the save rides the F9/autosave pass" · Reality: only `load` is wired (`app/mod.rs:1807`) → nothing is ever persisted | CLAIM-OTHERWISE |
| 30-4 | behavioural | `kills_needed_for` returns 0 outside the banner table | Claim: (silent) · Reality: 0 means "≥ full", so **164 of 172** bestiary entries report tier 4 with zero kills; the C# default is 50 (`ItemID.Sets.DefaultKillsForBannerNeeded`) | SILENT |
| 30-5 | behavioural | `quick_unlock` covers 5 entries instead of every boss | Claim: (silent) · Reality: the C# sets it on **every** boss (`BestiaryEntry.cs:37-41`) plus the salamander and `HighestOfMultiple` overrides; the town NPC `Guide` wrongly carries it (the C# uses the chat provider) | SILENT |
| 30-10 | cosmetic/behavioural | The unlock popup, advisor and chat tag handler are missing | Claim: (silent) · Reality: `AchievementUnlockedPopup` (`UI/InGamePopups.cs:13`), `AchievementAdvisor` and `AchievementTagHandler` have no port | SILENT |
| 30-11 | behavioural | `notify` returns only the last completed achievement | Claim: (silent) · Reality: the C# fires `OnCompleted` (and a save) for every one (`Achievement.cs:93-108`) | SILENT |
| 30-12 | behavioural | `completion_percent` divides by the port's 172-entry roster | Claim: (silent) · Reality: combined with #4 it reads ~95 % at world start, which gates the Zoologist's shop (`Chest.cs:3265`) | SILENT |
| 30-16 | behavioural (MP) | There is no MP sync of the bestiary kill counts | Claim: (silent) · Reality: `NetBestiaryModule.cs` has no port | SILENT |
| 30-18 | behavioural | `Achievement.completed` vs `is_completed()` disagree | Claim: (silent) · Reality: the Rust `completed` flag is initialised `false` (188) while `is_completed()` on a zero-condition row is `true` → the two disagree after a save/load (`load_pairs` 350) | SILENT |

### 36 — cinematics / film / Old One's Army (12)

| Source | Severity | One-line risk | Claim vs reality | Status |
| --- | --- | --- | --- | --- |
| 36-D1 | cosmetic | The whole `Terraria/Cinematics/` framework is absent | Claim: no ROADMAP row and no ANALYSIS section cover the cinematics — a **documentation gap**, not an acknowledged deviation | SILENT |
| 36-D2 | cosmetic (scope risk) | The films are dead code in vanilla | Claim: not recorded anywhere · Reality: `PlayFilm` has **0 call sites** → porting them buys no player-visible behaviour | SILENT |
| 36-D3 | behavioural | The kill worth is a constant 1.0 | Claim: (silent) · Reality: `dd2_report_kill` uses `KILL_WORTH_T1 = 1.0` instead of the per-kind/per-tier `GetMonsterPointsWorth` (`DD2Event.cs:546-554`, `:1106`, `:1199`, `:1509`) → the wave budget burns at the wrong rate for every tier ≥ 2 | SILENT |
| 36-D4 | behavioural | Kills outside the current wave's roster are dropped | Claim: (silent) · Reality: `npcs.rs:1481-1484` filters by roster; the C# scores every OOA kill | SILENT |
| 36-D6 | behavioural (narrow) | The 139 kill-cap is unconditional | Claim: (silent) · Reality: the C# caps only when the T1 Dark Mage is *not* downed (`DD2Event.cs:993-996`) | SILENT |
| 36-D7 | behavioural | The event's **loss** path is absent | Claim: (silent) · Reality: a destroyed Etherian Crystal never ends the event (`DD2Event.cs:507-515` vs `npcs.rs:1690-1715`) | SILENT |
| 36-D8 | behavioural | The victory scene, wave-break skip, medals, crystal drops, arena/building block, entity wipes and tier persistence are all missing | Claim: the ROADMAP rows stop at "the wave engine" · Reality: `StartVictoryScene` (`:481-505`), `RequestToSkipWaitTime` (`:1779-1801`), `DropMedals` (`:802`), `ShouldDropCrystals` (`:815`), `FindArenaHitbox` (`:753`), `WipeEntities` (`:629-693`) and `DD2Event.Save/Load` (`:151-170`) are all absent | SILENT |
| 36-D10 | behavioural (UI) | The OOA never enters the invasion framework | Claim: (silent) · Reality: `invasion_type` stays 0 instead of `INVASION_ID = 3` (`DD2Event.cs:47`) → the OOA's wave bar is outside the vanilla's invasion-progress path (`Main.cs:64344`, `:47687-47694`) | SILENT |
| 36-D11 | behavioural | `ZoneOldOneArmy` is absent | Claim: (silent) · Reality: the OOA enemies' aggro/despawn gate (`NPC.cs:799`) and the "close to a lane portal" metric (`Player.cs:16720`) do not exist | SILENT |
| 36-D13 | cosmetic | The `DD2Event.DamageTracker` is missing | Claim: (silent) · Reality: `DD2Event.cs:17-39` (started at `:277`) — the kill-time/boss-health display has no port | SILENT |
| 36-D5 | behavioural (the win transition) | The D1 wave-6 branch is a bare `1` | Claim: ROADMAP M4-EVENT-5o claims "the wave-6 sentinel is the win (`WonThisRun`)" · Reality: the sentinel is there, the `waveNumber = 5` / `currentKillCount = 1` write-back and the `StartVictoryScene` call (`DD2Event.cs:998-1005`) are not | CLAIM-OTHERWISE |

### 38 — out-of-scope confirmation (13)

| Source | Severity | One-line risk | Claim vs reality | Status |
| --- | --- | --- | --- | --- |
| 38-1 | cosmetic (doc), player-facing | README is stale on multiplayer | Claim: `README.md:171` lists "**Networking** (multiplayer)" under "What is *not* implemented yet" · Reality: `net.rs` (3 392 lines) + `netglue.rs` (2 897) + `netmodules.rs` (528) and the M6-NET-1..5 rows (done 2026-11-09) contradict it — **FALSE** | CLAIM-OTHERWISE |
| 38-2 | cosmetic (doc) | README is stale on the in-game options window | Claim: `README.md:166-168` says "only the in-game options window vanilla opens over the inventory is still missing" · Reality: M7-UI-4 is **done 2026-11-11** — **FALSE** | CLAIM-OTHERWISE |
| 38-3 | cosmetic (doc), planning-relevant | The "until M6-NET is complete" non-goal is stale | Claim: ATLAS:1747-1751 frames the network gap as "until M6-NET is complete" / "the entity/tile/module sync is still single-player-first" · Reality: M6-NET-1..5 all landed (2026-11-09); the *real* remaining gap is the excluded Steam transport/lobby layer — **FALSE/STALE** | CLAIM-OTHERWISE |
| 38-4 | cosmetic | The RGB non-goal is over-broad by path | Claim: ATLAS:1157-1158 and ATLAS:1759-1760 exclude "RGB peripheral lighting (`GameContent/RGB/*`)" · Reality: that folder also holds `VirtualKeyboard.cs` (450 — **ported**, PORT_MAP:180), `DebugKeyboard.cs` (48), `CommonConditions.cs` (399) and `NoiseHelper.cs` (82) → the exclusion must name the `*Shader*.cs` files plus the peripheral packages | CLAIM-OTHERWISE |
| 38-5 | behavioural (doc) | `Terraria.Social` is excluded as a *path*, never as modules | Claim: (silent) · Reality: no doc enumerates the eight modules; a reader sees a **fully ported** achievement UI and can only guess that the Steam *unlock* is absent | SILENT |
| 38-6 | medium | `Terraria/Testing/**` (34 files, ≈6 000 lines) is an undocumented absence | Claim: nothing in ROADMAP/ANALYSIS/PORT_MAP admits it · Reality: it contains the vanilla's deterministic replay/snapshot harness (`RecordReplay.cs` 177, `StateSnapshot.cs` 473, `LockstepDebug.cs` 258, `QuickLoad.cs` 406) — precisely the tool a "1:1 reimplementation" project would use to prove parity | SILENT |
| 38-7 | cosmetic | The in-game dev screens are an undocumented absence | Claim: ATLAS:962 lists "BestiaryTest 17" as a C# screen with no port/exclusion marker · Reality: `UIBestiaryTest.cs` + `UITextWrappingTest.cs` have no port | SILENT |
| 38-8 | low | `TimeLogger.cs` (839) and the crash handlers are undocumented absences | Claim: (silent) · Reality: `Terraria/Utilities/CrashDump.cs` (119) + `CrashWatcher.cs` (84) — the port has no crash handler at all | SILENT |
| 38-9 | medium (behavioural) | Dedicated-server mode is an undocumented absence | Claim: the non-goals only promise a "port-internal" handshake · Reality: the vanilla's headless `-server` mode (`Main.dedServ`, **169 sites**) is not ported; the port hosts in-window on TCP 7777 | SILENT |
| 38-10 | cosmetic | `Terraria.Social.WeGame.*` (26 files, ≈2 850 lines) is never named | Claim: only the path `Social/*` covers it | SILENT |
| 38-11 | cosmetic | README never mentions the platform exclusions at all | Claim: (silent) · Reality: `README.md:142-172` lists gameplay gaps but says nothing about Steam/WeGame, cloud saves, the Workshop/mod browser, peripheral RGB or dedicated server, while the README's pitch is "the same game with a different engine underneath" (`README.md:3-8`) | SILENT |
| 38-12 | cosmetic | Dead user-facing row | Claim: documented in ANALYSIS:1942-1951 · Reality: the Multiplayer's "Join via Steam" row is a hint only (`menu/pages.rs:~404-412`) — a visible dead end | SILENT |
| 38-13 | behavioural | The Steam P2P transport divergence is undocumented | Claim: (silent) · Reality: the port is **TCP-only**; the vanilla's Steam P2P and the lobby browser have no analogue — "not stated anywhere in ROADMAP/ANALYSIS" | SILENT |

### 39 — port architecture inventory (15)

| Source | Severity | One-line risk | Claim vs reality | Status |
| --- | --- | --- | --- | --- |
| 39-1 | structural | "The god-files are gone" is false as stated | Claim: AGENTS.md: "**The god-files are gone.** Extend the matching module; never grow a new god-file." · Reality: **8 hand-written files exceed 5 000 lines** (`projectile.rs` 17 411, `npc.rs` 17 134, `player.rs` 10 098, `world_sim/npcs.rs` 9 057, `world_sim/editing.rs` 7 738, `edit.rs` 6 791, `growth.rs` 5 305, `smart_cursor.rs` 5 197) — true only for the bootstrap (`main.rs` 163) | CLAIM-OTHERWISE |
| 39-2 | structural | "App state (118 fields)" is stale | Claim: AGENTS.md + ARCHITECTURE.md both say 118 · Reality: `App` has **257** direct fields (`app/mod.rs` L82) | CLAIM-OTHERWISE |
| 39-3 | cosmetic (docs) | ARCHITECTURE.md's crate/module line counts are ~2× low | Claim: `app/` ~5.5 k lines, `render/` ~4.5 k, `ui/` ~15.5 k, `main.rs` 86 lines · Reality: actual **18 371 / 11 301 / 24 424 / 163** | CLAIM-OTHERWISE |
| 39-4 | cosmetic (docs) | ARCHITECTURE.md's graph node count is stale | Claim: "the graph is rebuilt (graphify: 3 440 nodes / 7 933 edges Rust-side)" · Reality: the current `graphify-out/graph.json` holds **13 769 nodes** (12 712 from `crates/`, 409 `docs`, 293 `tools`) and **35 496 links** | CLAIM-OTHERWISE |
| 39-5 | structural | "the 2026-11-01 refactor split the two god files" is only partly true | Claim: ARCHITECTURE.md/ANALYSIS · Reality: `main.rs` and `menu.rs` were split; the *entity* god files (NPC/Projectile/Player/WorldGen) were never split — they were re-homed into `tr-app`/`tr-core` leaves | CLAIM-OTHERWISE |
| 39-6 | structural | The "C# seams" table covers UI and rendering only | Claim: ARCHITECTURE.md lists 3 seams (`FrontScreenBehaviours`, `LightEngine`, `PassPrologue`) · Reality: there is **no seam at the entity/sim boundary**, which is precisely where Stage 2 needs one | SILENT |
| 39-7 | structural | The SoA/spatial-hash plan is aspiration only | Claim: ATLAS "Entities (M4) — data-oriented plan" · Reality: **no** spatial hash, **no** SoA, **no** event buffers, and the sim is `&mut App` end-to-end — the plan is not started | SILENT |
| 39-10 | structural | `World` holds runtime event flags | Claim: not in any doc · Reality: `World`'s 132 fields mix tile data with ~50 runtime event flags (`downed_*`, `invasion_*`, `tower_*`, `slime_rain*`, `ore_tiers`, `seeds`) that `world_sim/{clock,npcs,weather}.rs` write → `tr-core` is **not** read-only during a tick, which blocks the read-only phase A | SILENT |
| 39-11 | behavioural/determinism | `App.npcs` index instability vs `ProjectilePool::local_immunity` | Claim: documented in-code ("stable between death compactions") · Reality: the `Vec` free list (no `GetAvailableNPCSlot` protocol, research 05) means a compaction **does** shift indices → a spatial hash keyed on the `Vec` index would inherit the bug | SILENT |
| 39-12 | determinism | `World.extras: HashMap<u32,u8>` | Claim: (silent) · Reality: `HashMap` iteration order is nondeterministic → a determinism hazard for any future phase-A pass over it | SILENT |
| 39-13 | cosmetic | Duplicated helpers across crates | Claim: (silent) · Reality: `tr_core::scene::move_towards` vs `tr_app::ambient_sound::move_towards`; `hsl_to_rgb` twice; 15 cross-crate name collisions (`break_wall`, `place_tile`, `update_rain`, `move_towards`, `hsl_to_rgb`, …) — a house-style violation | SILENT |
| 39-14 | behavioural (latent) | Two homes for `Collision.EmptyTile` | Claim: PORT_MAP already records the move as pending · Reality: `tr-core::smart_cursor::collision_empty_tile` vs `tr-app::collision.rs` | SILENT |
| 39-15 | cosmetic | `main.rs` still declares 70 modules | Claim: (silent) · Reality: 163 lines of bootstrap but 70 `mod` lines — cosmetic only; the bootstrap body is genuinely thin | SILENT |
| 39-8 | none (accurate) | The parallel-light/framing/decode rows | Claim: ATLAS "C1/C3 parallel light, C4 framing, C5 decode: done" · Reality: verified — the only parallelism in the tree is `tr-core/{framing,lighting/{mod,blur}}.rs` + `tr-app/assets.rs` | ADMITTED |

**Register totals:** 483 deviation/risks rows surveyed → **345 unadmitted** (of which **62** are
CLAIM-OTHERWISE, i.e. the docs assert the opposite) · 79 admitted · 59 partially admitted
(admitted in code comments or in one doc only).

---

## 4. Unadmitted-claims list

Rows where `docs/ROADMAP.md` / `docs/ANALYSIS.md` / `docs/PORT_MAP.md` **actively assert
something the research disproved** — a `✅` DONE row, a status line, or prose that states
the opposite of the code. **44 rows.** This is the section that must drive the new ROADMAP's
"correct the record" pass: each entry is a *false* statement in the current docs.

| # | Doc claim (file:line) | What the research found | C#/Rust citations | Source |
| --- | --- | --- | --- | --- |
| U-01 | `ROADMAP.md:1469` (M4-EVENT-4b, `✅`): "the pirate/martian id collision 68-72 renumbered to 78-82/83 (plus the render_key + from_u16 tables), **the duplicated dead arms removed**" | The renumber landed but **31 conflicting / 29 dead arms remain in `to_u16`** and **7 contradictory / 21 dead arms remain in `from_u16`**, and the renumber *created* a 78..84 collision that yields **7 round-trip failures** on the live save path. Verdict on the DONE row: **FAIL** | `npc.rs:850` `to_u16`, `:1105` `from_u16`, `:887-893` vs `:1044-1050`; `app/saveglue.rs:107` (write), `app/mod.rs:1980` (read); `app/netglue.rs:385`, `:1044`, `:1504`, `:2060` | 34-id-sets-audit.md:183-194 |
| U-02 | `ANALYSIS.md:3024-3027`: "the `to_u16` id map's pirate/martian collision … renumbered to 78-82/83 through all three tables, **the duplicated dead arms removed**" | Same as U-01 — the claim is false and the cleanup was never done | as U-01 | 34-id-sets-audit.md:183-194 |
| U-03 | `ROADMAP.md:64`: "Renderer (atlas, instancing, zoom, frame cache) | ✅ | **2 draw calls**; G6 cache; zoom 1.0–2.0" | The port issues **one `pass.draw` per non-empty range** → up to **6** per frame, and with `TR_POST=1` the frame is 3 + 3 plus 1..4 filter passes. The "2 draw calls" claim is wrong | `tr-render/src/sprite.rs::draw_ranges` (1195-1239), `tr-render/src/post.rs::MAX_CHAIN`, `frame.rs:1774` (`BENCH draws=`) | 41-render-pipeline.md:161-166 |
| U-04 | `ROADMAP.md:66`: "Lighting | ✅ | scanner + blur + **4 modes** + sun/moon light (M1.5-LIGHT-3)" | The 1442-line `LegacyLighting` engine is unported; White/Retro/Trippy are shader approximations of the *style descriptors*, not the engine. The "4 modes" claim is true only of the descriptors | `params.rs::LightStyle::shade` (`:116-148`); `LegacyLighting.cs:202`, `:851`, `doColors_Mode0..3_Swipe` `:1087/1192/1273/1331` | 10-lighting.md:147-149 |
| U-05 | `ROADMAP.md:73`: "NPC families, town, bosses, events | ✅ | 124 M4 rows through `M4-EVENT-19`" | The AI family layer covers **~60 of 128** vanilla families; **68 of the 128 `aiStyle` arms have no Rust kind**, and the two biggest (aiStyle 3, the fighter family of 155 vanilla types, and aiStyle 7, the 52 town types) are archetype approximations. Module 40's verdict on this summary row is **FAIL** | `npc.rs::update_rng` match (4114-4413); `npc_ai.rs` (6 archetypes, 769 lines); 74 styles unreferenced (10, 15, 17–25, 31, 37, 41–44, 49, 50, 52, 53, 55–73, 83, 84, 86–92, 98–104, 111–122, 124–127) | 25-npc-ai-audit.md:247-251, :367-382 |
| U-06 | `ROADMAP.md:604-605` (§14 header): "**Rust** `npc.rs`, `world_sim/npcs.rs` · 🟡 **3 kinds**" | The roster is **182 kinds** (`NpcKind` 31-686), not 3; the header misdirects the next agent | `crates/tr-app/src/npc.rs:31-686`; `npc.rs:694-701` (the 8 slime-colour variants) | 25-npc-ai-audit.md:367-371 |
| U-07 | `ROADMAP.md:1409` (M4-NPC-4, `✅`): "the Hornet (**3.25/0.018**, `:50322-50323`)" | The C# Hornet (type 42) is **3.5/0.021**; the port ships the **type-205** values. The cited ranges are wrong too (`:50322-50323` = type 619, `:50328-50329` = type 252, the Meteor Head is at `:50374-50377`). The row is self-invalidating | `npc_ai.rs::HoverParams::HORNET` (177-181), bound at `npc.rs:4382`; `NPC.cs:50341-50344`, `:50366-50367`, `:50374-50377` | 40-port-done-verification.md:39-40, :84-95 |
| U-08 | `ROADMAP.md:1353` (M2-COMBAT-1, `✅`): "`Player::hurt` iframe 40 ticks" | The C# floors `num2` to ≥ 1 **after** the endurance multiply; the port does not, and `hurt_inner`'s comment asserts the opposite of the source → a 1-damage hit with enough endurance deals 0 in the port and 1 in vanilla | `Player.cs:38304`, `:38378-38381`; `player.rs::hurt_inner` (2223-2243); `damage.rs::iframes_for` (172) | 40-port-done-verification.md:42, :98-103 |
| U-09 | `ROADMAP.md:1436` (M4-DAMAGE, `✅`): "wired into the melee hit" | The **projectile**→NPC path never calls `strike_npc_damage`/`checkArmorPenetration`: every ranged hit skips defence halving and the ±15 % `DamageVar` roll. The row's own note excuses only the *contact* hit | `projectile.rs::damage_pve` (9460) → `hurt_from_projectile` (9599); `Projectile.cs:13630-13649` | 04-player-combat.md:100-101, :141-146 |
| U-10 | `ROADMAP.md:1513` (M4-EVENT-1a, `✅`): "the **2-second spawn pass** near the player while the invasion lasts" | The port fires `invasion_spawn_pick` **once per fixed step** with no rate roll and no nearby cap → ~20× the vanilla invasion density; C# forces `spawnRate = 20` / `maxSpawns = 5·(2+0.3·players)` | `world_sim/npcs.rs:3386-3423` (and `:3418` skips the natural spawn entirely); `NPC.cs:781-785` | 24-spawner-invasions.md:210-216 |
| U-11 | `ROADMAP.md:1410` (M4-SPAWN-1, `✅`): "`GetSpawnRate` (`NPC.cs:473-810`) in the vanilla order … remix/FTW/journey/skyblock/fairy branches constants-off" | The chain is faithful only through `NPC.cs:789`; **`:790-928`** (the skyblock branch, the **town-NPC distribution block** and the luck tail) is not ported — and the port's own doc comment at `spawner.rs:193-196` claims the random town-NPC sub-branches are kept while the function **discards its `rng`** (`:203`) | `spawner.rs::get_spawn_rate` (202-368), `:203`; `NPC.cs:473-810`, `:799-928` | 24-spawner-invasions.md:202-210, :262-265 |
| U-12 | `ROADMAP.md:1339` (M2-PHYS-3, `✅`): "the four wet flags (`wet`/`lavaWet`/`honeyWet`/`shimmerWet` **from the mid-tile liquid kind**, `Player.cs:27728-27910`), the in-liquid gravity **/ 3** with the both-ways fall cap at `maxFallSpeed / 3` (`Player.cs:27008-27017`)" | C# rebases gravity/maxFallSpeed/jumpHeight/jumpSpeed **per liquid** at `Player.cs:24382-24437` (water 0.2/5.01/30/6.01, honey 0.1/3.01, merman 0.3/7.01, trident 0.25/6.01, shimmer 0.15). The cited `:27008-27017` is the mount/wings arm, **not** the plain wet path. The port's `GRAVITY/3` + `jump_height 15` gives a **2× jump-height error** in liquids, and `maxFallSpeed += 0.01f` is missing | `Player.cs:24382-24437`, `:26962-27020`, `:27339`, `:27354`, `:27365`; `player.rs:3190-3198`, `:2516-2517`, `:30` | 03-player-core.md:154-183 |
| U-13 | `ROADMAP.md:938` (§23): "events **bubble child→parent**" | There is **no parent walk** in the Rust pointer machine: events are raised on the hit element only and ancestor `IsMouseHovering` is never set | `UIElement.cs:414-547` vs `pointer.rs:154-224` | 18-ui-framework.md:221-226 |
| U-14 | `ROADMAP.md:937` (§23): "parent `UIList` gives infinite height (`UIElement.cs:285-321`)" | The ∞-height parent special case is **not modelled** (`uistate.rs:73-75`); percent-height children of a list resolve against the list rect | `UIElement.cs:288-291` vs `uistate.rs:73-75` | 18-ui-framework.md:215-220 |
| U-15 | `ROADMAP.md:1217` (C4): "Banded parallel autotiling (128 rows) | Framing **295 → ~70 ms** … ; **outputs identical**" | No golden test proves equality against the C# frames; the `variant()` hash is position-only, so the band-parallel scheme is *sound* but *unproven* | `framing.rs::frame_world` (1010-1063), `variant()` (134-146); `Framing.cs:319`, `:390-395` | 08-tile-framing.md:146-149 |
| U-16 | `ROADMAP.md:1305` (TILE-DRAW-4, `✅`): "shader-side paint recolor (compiled HLSL not in decompile) + wall paint/coatings in `World.extras`" + `ROADMAP.md:149` "Paint/coatings: `TilePaintSystemV2.cs` — done (TILE-DRAW-4)" | The C# **bakes a recolored copy** per `(type, style, paint)` in `TilePaintSystemV2.RequestTile`/`PrepareAllRequests`; the port recolours **per pixel** in the shader and admits it "shades in the sRGB domain". The DONE rests on an unprovable equivalence (the compiled HLSL is not in the decompile) and the per-tile cost now scales with painted pixels | `TilePaintSystemV2.cs:369-541`; `sprite.wgsl::paint_recolor` (158-169), the `PaintID` sRGB table (222-226), `sprite.wgsl:151-158` | 09-tile-drawing.md:209-218 |
| U-17 | `ROADMAP.md:1312` (M1.4-WEATHER-1, `✅`): "Already ported and verified: the walk + extreme re-rolls + the ±0.8 clamp, **the 0.35 walk cap**, …" | `Wind::any_player_120_life` is set `true` in `Wind::new` and **never written anywhere in `crates/`** → the low-wind limit is dead code and untested; wind never damps to ≤ 0.35 for a low-life player | `wind.rs:75`, `:108`, `:199`, `:211`, `:232`; `Main.cs:58863-58871`, `:58889` | 16-weather.md:220-224 |
| U-18 | `ROADMAP.md:1300` (M1.4-BG-11/16, `✅`): "clouds/stars/horizon" | `make_clouds` is a deterministic **stand-in** (`NUM_CLOUDS_STAND_IN = 100`), there is no live `Cloud.addCloud`/`UpdateClouds`, and the drift is a constant light wind instead of `windSpeedCurrent * 9 * GetParallax` → `change_rain`'s `numClouds > 150` / `> 100` bands are unreachable, so rain strength only ever reaches the storm band | `render/sky.rs::make_clouds` (270-342); `rain.rs:57-63`, `:713-722`; `Cloud.cs:140-147`, `:170-177`, `:391` | 16-weather.md:119, :235-239 |
| U-19 | `ROADMAP.md:1311` (M1.4-BG-21, `✅`): "Out: … transition event rolls + the blood-moon/eclipse cooldown force-resets (**M4** — `Main.cs:65350`, `65422`)" | They are **landed**: `clock.rs::advance_time` `:37-38` (blood moon) and `:72` (eclipse) call `world.rs::roll_blood_moon` `:565` / `roll_eclipse` `:594`. The row's "Out (M4)" note is stale, and the *real* gaps (`tenth_anniversary` hardcoded false, `spawn_eye` false, the `stopEvents` gate, the pumpkin/snow-moon clearings `Main.cs:64837-64845`) are unstated | `Main.cs:65333-65352`, `:65421-65430`, `:64837-64845` | 16-weather.md:145, :266-272 |
| U-20 | `ROADMAP.md:401` (§8 header): "**Rust** ⬜ (none yet) · ⬜" | `regen.rs` (706 lines, 18 tests), `buffs.rs` (599 lines, 17 tests) and `luck.rs` (1 026 lines) all exist; the task rows *below* the header (`✅ M2-BUFF-2 regen engines`) contradict it | `crates/tr-app/src/{regen,buffs,luck}.rs`; `ROADMAP.md:1345-1346` | 03-player-core.md:233-236 |
| U-21 | `ANALYSIS.md:2120-2124`: "**No `.plr` reader/writer** (spec §9.4). Characters are the implicit session entry plus optional `TR_FAKE_CHARS` fakes; Create records the session draft instead of writing a character file. … a scanned `.plr` is skipped like `Player.GetFileData`'s null return." | The reader is live and the port **writes** players: `save_list.rs:291` reads real `.plr` files and `session.rs:141-176` writes one via `player_file::write_player_file`. The same stale sentence survives in `flow.rs:73` and in the user-visible notice `app/mod.rs:1482` ("no save-file writer") | `PlayerFileData.CreateAndSave` (`PlayerFileData.cs:50-63`); `Player.cs:54356-54360` (the three starter items) | 19-ui-screens-frontend.md:326-334 |
| U-22 | `PORT_MAP.md:180`: "`Main.LoadPlayers`/`LoadWorlds`, … | `crates/tr-app/src/ui/save_list.rs` | **partial (no `.plr` reader; metadata stand-ins)**" | The `.plr` reader is live and the port writes players; only the metadata stand-ins remain | `save_list.rs:167`/`:255`/`:397`/`:415`; `Main.cs:4001-4030`/`:3959-3981`/`:4772-4791`/`:4796-4813` | 19-ui-screens-frontend.md:335-337 |
| U-23 | `ROADMAP.md:78`: "Audio | 🟡 | … + the **8-cue bank**; **front-end music and UI sounds missing (BUG-01/17)**" | Both BUG-01/BUG-17 were fixed on 2026-11-10 and the cue table now holds **26** entries (13 names actually played of **527** shipped `.xnb` assets) | `audio.rs::SFX_CUES` (22-75); `app/mod.rs::update_menu_music` (937-953); `frame.rs:295-297` | 32-sound-audio.md:238 |
| U-24 | `PORT_MAP.md:241`: "`Audio/*`, `SoundEngine` | … | **partial** (2026-11-08, M6-AUDIO-1: …)" | The row predates BUG-01/BUG-17 and does not quantify the coverage: 64 Rust play sites against 1 411 C# `PlaySound` + 64 `PlayTrackedSound` sites; missing families include footsteps/run, tink, research, grass, shatter, camera, chat, drip, zombie, splash, drown, max-mana, double-jump and coins | `LegacySoundPlayer.cs:360-1382`; `SoundEngine.cs:83-149` | 32-sound-audio.md:227 |
| U-25 | `ANALYSIS.md:81`: "| Audio | `Audio/*`, XACT banks | ~4 000 | 🟡 | Runtime `SoundEffectReader`, **eight PCM cues** through `rodio`, music selector; **XACT/WMA playback pending** |" | The XACT/ADPCM music path **is** ported (`tr-content/src/xact.rs`, `adpcm.rs`; `audio.rs::MusicSystem`); the row understates the port and overstates the gap | `tr-content/src/xact.rs::parse_sound_bank` (332), `parse_wave_bank_entries` (337), `get_cue` (318), `music_track` (327); `tr-content/src/adpcm.rs::decode_msadpcm` (31) | 32-sound-audio.md:129-135, :221-227 |
| U-26 | `ROADMAP.md:250`/`PORT_MAP.md:250`: "LZX decompression (`ReLogic/Content/Lzx*`) | `crates/tr-content/src/lzx.rs` | **done**" | **No such file exists** in `decompiled-libs` — ReLogic's `XnbReader` is a 55-line shim over FNA's `ContentManager` and delegates the whole format parse to FNA; the decoder came from MonoGame/Xbox. The citation is wrong | `XnbReader.cs:10` (`InternalContentManager : ContentManager`), `:24` (`ReadAsset<T>("XnaAsset", null)`), `:29` (`OpenStream`) | 37-relogic-libs.md:131, :276-280 |
| U-27 | `ROADMAP.md:1305`/`INDEX.md:62`: the module label "(content pipeline, **XNA, JSON**)" | ReLogic contains **no** JSON serializer (Terraria uses Newtonsoft.Json on the game side) and **no** XNA types (those are FNA dependencies) | `ReLogic.Content.Readers.XnbReader.cs`; `Terraria.Initializers.AssetInitializer.cs:59`; `Terraria.Preferences`; `Terraria.Localization.LanguageManager` | 37-relogic-libs.md:40-48, :201 |
| U-28 | `ROADMAP.md:335`: "keep `deterministic_for_seed` **FNV proof**" | `deterministic_for_seed` has **no** FNV in it — it generates the same world twice and asserts element-wise equality (self-consistency); the FNV-ish fold lives in `world_hash.rs`, is the port's own mix, and is re-pinned ~20 times. The pin cannot detect a C#-divergent stream | `worldgen/mod.rs:259-276`; `world_hash.rs:9-13`, `:18-25`, `:64-172`; `WorldGenerator.cs:517-545` | 35-rng-utils.md:236-242, :171-196 |
| U-29 | `ROADMAP.md:103`: "(`Main.cs:17747-17815`) — a *named RNG stream* per stage" | The Rust has **no** named-stream registry: `App` holds seven independent `u64` streams seeded with arbitrary XOR constants, one of which (`craft_rng: 1`) is not seed-derived at all | `Main.cs:3204-3215`, `:17749-17809`; `app/mod.rs:180`, `:303`, `:609`, `:612`, `:634`, `:2700` | 35-rng-utils.md:221-226 |
| U-30 | `ROADMAP.md:1608` (M6-SAVE-1, `✅`): "**all 11 `.wld` sections read + written**" | The tile section drops the four wire bits (`b2` 0x02/0x04/0x08, `b3` 0x20) → the wire state is lost on a `.wld` round trip and the run boundaries differ from C#'s `isTheSameAs`. *The row's own tail does list "wires-not-written", so the gap is admitted in the body while the headline over-claims.* | `wld/sections/tiles.rs:103-105`, `encoded_same` (219-259); `Tile.cs:155-197`; `WorldFile.cs:1432-1604` | 42-save-load-audit.md:228-238 |
| U-31 | `ROADMAP.md:80`: "Saves | 🟡 | `.wld` 11 sections + `.trwld` **v10**, `.plr` read/write (M6-SAVE-2); **favourites round-trip broken (BUG-03)**; M6-SAVE-1b parked" | `.trwld` is now **v13** (ROADMAP.md:1609 records it landing 2026-11-11) and BUG-03 is **closed** — `player_file.rs` round-trips the favourited bits (test `the_favorited_bits_survive_a_save_and_a_load`) | `save.rs::VERSION` 12 → 13; `player_file.rs::write_player_bytes` (339) / `read_player_file` (135) | 42-save-load-audit.md:305-307 |
| U-32 | `ROADMAP.md:69`: "World generation | 🟡 | … 🟡 DUNGEON, TEMPLE, STRUCT-5 … ; **parked LIVING-trees**" | `living_trees.rs` (1 113 lines, 12 tests) is a faithful port — its own catalog row (`ROADMAP.md:1571`) says done — and M5-DUNGEON/M5-TEMPLE are 🔄/🟡 partial, not parked | `worldgen/living_trees.rs`; `WorldGen.cs:15476-15715`, `:27973` | 12-worldgen.md:311-315 |
| U-33 | `ROADMAP.md:73` vs `ROADMAP.md:1534`/`:1540-1550`: the snapshot says "🟡 the display rows M4-NPC-3c/3h..3r" while the catalog marks the same ids `✅` | The ROADMAP **contradicts itself** about the same rows and nothing states which is current | `ROADMAP.md:73`, `:1534`, `:1540-1550` | 40-port-done-verification.md:45, :113 |
| U-34 | `ROADMAP.md:1062` (§27): "NetModules: … modules: Text, Bestiary, CreativePowers(+Permissions/Unlocks/Report), Liquid, Particles, Ping, Ambience, TeleportPylon, **Debug**" | The port carries **11 of 12** NetModules — **`NetDebugModule` has no Rust counterpart** (`module_id` lists 11) | `netmodules.rs::module_id` (34-45) | 17-netcode.md:164-167 |
| U-35 | `ROADMAP.md:1067-1071` (§27 Tasks): "✅ done 2026-11-09 — M6-NET-protocol (umbrella), M6-NET-1 … M6-NET-5 …" | Only **25 of the 163** message ids are ported and the 774 C# `SendData` sites funnel into ~10 Rust send paths; the join bundle (`SyncConnectedPlayer`/`SyncOnePlayer`/`SyncOnePlayer_ItemArray`) is not reproduced and `SendNPCHousesAndTravelShop`/`SyncChestContentsForSection`/`SyncNPCsForSection` are absent | `net.rs::id` (56-95); `NetMessage.cs:2806-2853`, `:2724`, `:2741` | 17-netcode.md:104-108, :295-297 |
| U-36 | `ROADMAP.md:1608`/`:1609` (M6-SAVE-1 / M6-SAVE-1b) and `ROADMAP.md:80`: the `.wld`/`.trwld` twin set | The `.trwld` still has no native twin for the **TownManager rooms, the bestiary sights/chats and the CreativePowers table** (they live on the App side) and the `.wld` writes placeholder header values | `save.rs::VERSION` 12 → 13 (the `VERSION` doc records the gap); `sections/header.rs:49-60`, `sections/flags.rs:220-274` | 42-save-load-audit.md:99, :239-247 |
| U-37 | `ROADMAP.md:1412` (M4-TOWN-1, `✅`): "the move-in: `town.rs` ports `QuickFindHome`'s three-pass scan … each through `check_room` + `room_needs`" | The scan passes and the furniture gate match, but the **score gate (`hiScore > 0`) and the score-best home tile do not** — `town.rs::scan_spot` returns the scan origin, so a town NPC can be homed on a spot vanilla would reject. The row also cites `RoomNeeds` as `:5717-5756` and `QuickFindHome` as `:5759-5805`, while the current decompilation has them at `:5679-5730` and `:5732-5794` | `WorldGen.cs:5679-5730`, `:5732-5794`, `:5858-5992`, `:5983-5988`; `town.rs::scan_spot` (50-57) | 29-town-npc-housing.md:239-244, :288-293 |
| U-38 | `ROADMAP.md:1414` (M4-TOWN-2, `✅`): "`process_mood` ports `ShopHelper.ProcessMood` (`ShopHelper.cs:95-174`)" | The **exclusion block** at `ShopHelper.cs:103-106` is absent: only the Travelling Merchant (368) returns early, while the comment claims the remix-world and the 37/453 exclusions are implemented → remix worlds get real mood prices | `happiness.rs:318-328`; `ShopHelper.cs:103-106`, `:363-371` | 29-town-npc-housing.md:245-251 |
| U-39 | `ROADMAP.md:1705` (BUG-20): "trash slot: quick-trash (`cursorOverride == 6`), clear on death, the +5/+168 chest-shop offset" with no done text | The code and three `bug20_*` tests are on main (`ui/inventory.rs::trash_rect` (729-736), `app/mod.rs::trash_swap` (1192-1193), `bug20_shift_click_moves_the_stack_to_the_trash`, `bug20_death_clears_the_trash`, `bug20_the_trash_shifts_with_the_chest_open`) | `Main.cs:42349-42366`; `Player.cs:54304` | 20-ui-ingame.md:200-206 |
| U-40 | `ROADMAP.md:1305`/`:149` (TILE-DRAW-4) and `ROADMAP.md:149`: "Paint/coatings: `TilePaintSystemV2.cs` — **done**" (see U-16) + `ROADMAP.md:1247`'s parallel-framing claim (see U-15) | (duplicates of U-15/U-16 — listed once in §3) | — | 09-tile-drawing.md:209-218 |
| U-41 | `ROADMAP.md:1319` (M1.4-TO, `✅`): "remaining: **the full-registry generator**" | This one is *admitted*; it is listed here only because the registry is 36 hand-written consts against C#'s `TileObjectData.Initialize` (`:2110`) — every added object style is a manual transcription | `TileObjectData.cs:2110`; `tileobject.rs::TABLE` (293-330) | 08-tile-framing.md:143-145 |
| U-42 | `ROADMAP.md:1404` (M4-NPC-4's own cited C# anchors `:50304-50305` / `:50322-50323` / `:50328-50329`) | The row cites the wrong `if` branches (type 619 / type 252 / the `targetData` block), so it cannot be re-verified by following its own anchors — the same class of error as U-07 | `NPC.cs:50299-50301`, `:50322-50323`, `:50328-50329` | 40-port-done-verification.md:91-95 |
| U-43 | `ROADMAP.md:1290`-area §12 "M2-BUFF-4 stat/movement debuffs ✅" and the M2-BUFF-3/5/7/8 rows | The **effect table** behind them is ~15 % ported: the C# `UpdateBuffs` chain is 251 arms over ~290 ids and the Rust covers ≈40 ids. Unported ids that change play include 3 Swiftness / 5 Ironskin / 7 MagicPower / 16 Archery / 113 Lifeforce / 115 Rage / 117 Wrath / 159 Sharpened / 165 Dryad's Ward / 199 NoBuilding / 332-334 hunger / 396 Mana Heat | `Player.cs:9855-12069`; `player.rs` inline effect `match` (2839-2930); `derive_dot_inputs` (4260) | 28-buffs-debuffs.md:206-214 |
| U-44 | `ROADMAP.md:1181-1226` (M1.5 Phase 1 + Phase 2): "**pixel-identical (0 differing pixels)**" + the timing tables | Measurement-only claims with **no code artefact** to re-read; the neighbouring Phase 3 table (`:1199-1205`) is still "in progress" → the section is a mixed state that cannot be re-verified statically | `ROADMAP.md:1181-1226`, `:1199-1205` | 40-port-done-verification.md:46, :120-123 |

---

## 5. Counts

### 5.1 Totals

| Metric | Count |
| --- | --- |
| Modules reviewed (completed research files) | **33** |
| Modules planned but not researched (21, 23, 26, 27, 30, 33, 36, 38, 39) | 9 |
| Deviation / risks rows surveyed | **395** |
| Deviations **not admitted** in ROADMAP / ANALYSIS / PORT_MAP | **276** (44 CLAIM-OTHERWISE + 232 SILENT) |
| Deviations admitted | 70 |
| Deviations partially admitted (code comment or one doc only) | 49 |
| Task seeds consolidated | **299** |
| Unadmitted-claims rows (§4) | **44** |
| Seed ids | `R-01-1` … `R-42-12` (stable, one per source seed) |

### 5.2 Task seeds by size

| Size | Count | Share |
| --- | --- | --- |
| **L** | 50 | 17 % |
| **M** | 132 | 44 % |
| **S** | 117 | 39 % |
| total | **299** | 100 % |

### 5.3 Task seeds by theme

| Theme | Seeds |
| --- | --- |
| (a) determinism / RNG | 22 |
| (b) pixel / visual identity | 26 |
| (c) behavioural parity (order / formula) | 97 |
| (d) missing subsystems | 33 |
| (e) data / coverage gaps (incl. doc-truth + verification) | 63 |
| (f) netcode / multiplayer | 14 |
| (g) UI | 20 |
| (h) audio | 8 |
| (i) architecture & concurrency prep | 16 |
| **total** | **299** |

### 5.4 Top 15 task seeds by (severity × size) — land these first

Ranked by severity first (critical → high → medium → low → cosmetic → doc), then by size
(L → M → S), then by blast radius (how many other seeds depend on them).

| Rank | id | Task | Size | Severity | Why first |
| --- | --- | --- | --- | --- | --- |
| 1 | **R-03-1** | Move the wet/shimmer statics into `update_with_grapple` (the per-liquid `(gravity, maxFallSpeed, jumpHeight, jumpSpeed)` tuple from `Player.cs:24382-24437`) | M | behavioural (critical) | A **2× jump-height error in every liquid**; the row `ROADMAP.md:1339` currently claims the wrong formula |
| 2 | **R-34-1** | Delete the dead `to_u16`/`from_u16` arms and fix the 78..84 id collision | S | behavioural/determinism (high) | A saved or remote **Deerclops comes back as a Pirate Deckhand**; touches the save + netcode paths |
| 3 | **R-40-3** | Restore the player damage floor (`num2 < 1 → 1`) after the endurance multiply | M | behavioural (medium-high) | The `✅` M2-COMBAT-1 row is wrong; 0-damage hits land as 0 |
| 4 | **R-04-2** | Projectile hit chain: `DamageVar` + owner luck + `strike_npc_damage` into the PVE hit | L | behavioural + determinism (high) | **Every ranged hit** skips defence and the ±15 % roll; also changes RNG draw counts |
| 5 | **R-05-4** | Wire `strike_npc_damage` + `check_armor_penetration` into the projectile path | M | behavioural (high) | Same defect, from the NPC side; two rows, one fix |
| 6 | **R-35-1** | Port `Terraria.Utilities.UnifiedRandom` | L | determinism (critical) | The substrate for 8 other seeds (R-35-2/3/4/5/7, R-24-8, R-07-7, R-11-3) and for every "vanilla replay" claim |
| 7 | **R-35-3** | `Main.SwapRandom` named-stream registry | L | determinism (critical) | 13 per-stage streams are 7 ad-hoc `u64` fields today; blocks R-35-4/5 |
| 8 | **R-12-7** | Replace the FBM surface model with `TerrainPass`'s height/rock lines | L | behavioural/pixel (medium) | The worldgen model is the largest single divergence; also the anchor for R-12-6 |
| 9 | **R-11-5** | Port the liquid reactions (obsidian / honey block / crisp honey / shimmer) | M | behavioural (high) | Lava+honey never interact; blocks R-11-7, R-12-5, R-32-8 |
| 10 | **R-05-9** | Close the `aiStyle` gap — port the missing archetypes (74 styles) | L | behavioural (high) | The headline AI gap; blocks R-25-10 |
| 11 | **R-28-3** | The buff HUD (`Main.DrawBuffIcon` + `Main.TryRemovingBuff`) | M | behavioural+pixel (high) | An entire visible subsystem is missing and the ROADMAP has no task for it |
| 12 | **R-15-2** | Port the `Dust` record + the `NewDust` family behind a `DustPool` | M | behavioural+pixel (high) | 3161 of 3279 dust sites have no port; blocks R-15-3/4/5/8 |
| 13 | **R-25-5** | A per-type speed table for the aiStyle-3 family | M | behavioural (portfolio-wide) | 155 vanilla types collapse onto one walker speed; the port's own comments claim otherwise |
| 14 | **R-40-1** | Fix the hover presets (`HORNET` → 3.5 / 0.021, the scale factor, the damping gate) | M | behavioural (high) | The only *wrong constant* in the hot area; invisible to the suite (2 tests in `npc_ai.rs`) |
| 15 | **R-07-1** | Fix the pickup double-add in `world_sim\items.rs:191-194` | S | behavioural (high) | A one-line fix that doubles **every** item the player picks up |

### 5.5 Recommended order (derived from §5.4 + the dependency edges)

1. **Correct the record first** — the 44 §4 rows plus the 44 CLAIM-OTHERWISE rows in §3. Nothing
   else can be planned against a ROADMAP that marks `✅` on rows the code disproves.
   Cheapest first: R-03-11, R-12-1, R-16-7, R-18-2, R-19-1, R-20-2, R-24-7, R-25-3, R-28-8,
   R-29-10, R-32-9, R-34-3, R-35-9, R-35-10, R-37-1, R-37-3, R-40-2, R-40-6, R-41-1, R-42-6, R-42-12.
2. **Behavioural landmines with a tiny blast radius** — R-07-1 (pickup double-add), R-40-3
   (damage floor), R-25-1 (Frost speeds), R-05-3 (goblin arm), R-03-2 (`maxFallSpeed`), R-34-1
   (id collision), R-16-1 (wind cap), R-40-1 (hover presets).
3. **The formula families** — R-03-1 (wet statics) → R-03-4/S4 (drowning) → R-03-5 (respawn);
   R-04-2 + R-05-4 (projectile damage) → R-04-1 (floor) → R-04-4 (ranged split);
   R-05-8 (`ScaleStats`) → R-24-3 (town-NPC spawn block) → R-24-1 (invaders override).
4. **The determinism substrate** — R-35-1 → R-35-2 → R-35-3 → R-35-4/R-35-5, then the
   per-consumer migrations (R-24-8, R-07-7, R-11-3).
5. **The missing subsystems with visible surface area** — R-15-2..R-15-5 (dust/gore),
   R-28-3 (buff HUD), R-10-1 (entity light), R-09-4 (tile animator), R-09-5 (vines/ropes),
   R-05-10 + R-40-5 (`behindTiles`), R-31-2..R-31-5 (golf), R-02-3 (full-screen effects).
6. **The architecture prep** — R-12-6 (world parity harness), R-41-6 (draw-order contract),
   R-40-8 (cited-range lint), R-42-8..R-42-11 (Stage-2 save), R-07-4 (item instances),
   R-18-9 (layer stack).

<!--PART3-->


