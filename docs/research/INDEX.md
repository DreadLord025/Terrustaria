# Research index вЂ” full C# Terraria audit

Map of the research effort backing the new ROADMAP. One file per module,
`docs/research/NN-slug.md`. Status: вњ… done В· рџ”„ in flight В· в¬њ planned.

Destination: a complete picture of every Terraria 1.4.5.8 subsystem вЂ” what is
ported, what is partial, what is missing, what diverges вЂ” to compose a detailed
new ROADMAP (Stage-1 completion + DONE re-verification + Stage-2 optimisation
per `docs/superpowers` agreements).

## Wave 1 вЂ” god files & world core
| # | Module | File | Status |
| --- | --- | --- | --- |
| 01 | Main update loop / time / camera / input | `01-main-update-loop.md` | рџ”„ |
| 02 | Main draw pipeline / scene layers / effects | `02-main-draw-pipeline.md` | рџ”„ |
| 03 | Player core (movement, life/mana, death) | `03-player-core.md` | рџ”„ |
| 04 | Player combat (hurt/damage, item use, buffs) | `04-player-combat.md` | рџ”„ |
| 05 | NPC core (defaults, AI dispatch, net sync) | `05-npc-core.md` | рџ”„ |
| 06 | Projectile core (defaults, AI dispatch, damage) | `06-projectile-core.md` | рџ”„ |
| 07 | Item core (defaults, prefixes, tooltips) | `07-item-core.md` | рџ”„ |
| 08 | Tile / Framing / TileObjectData / Modules | `08-tile-framing.md` | рџ”„ |
| 09 | TileDrawing / DrawData / paint+colour | `09-tile-drawing.md` | рџ”„ |
| 10 | Lighting / TileLightScanner / LightMap | `10-lighting.md` | рџ”„ |
| 11 | Liquid / LiquidBuffer / modules | `11-liquid.md` | рџ”„ |
| 12 | WorldGen / WorldBuilding passes | `12-worldgen.md` | рџ”„ |

## Wave 2 вЂ” world & render systems
| # | Module | File | Status |
| --- | --- | --- | --- |
| 13 | Collision / HitTile / scans | `13-collision.md` | в¬њ |
| 14 | Map / MapHelper / minimap layers | `14-map-minimap.md` | в¬њ |
| 15 | Dust / Gore / UIDust | `15-dust-gore.md` | в¬њ |
| 16 | Weather (rain/cloud/wind/scene metrics) | `16-weather.md` | в¬њ |
| 17 | Netcode (NetMessage/sockets/Netplay) | `17-netcode.md` | в¬њ |
| 18 | UI framework (UIElement/UIState/layers) | `18-ui-framework.md` | в¬њ |
| 19 | Front-end screens (char/world create+select) | `19-ui-screens-frontend.md` | в¬њ |
| 20 | In-game UI (options/inventory/chest/craft) | `20-ui-ingame.md` | в¬њ |
| 21 | Recipes / crafting | `21-recipe-crafting.md` | в¬њ |
| 22 | Chests / loot / shops / shopping | `22-chest-loot-shops.md` | в¬њ |
| 23 | Wiring / TileEntities | `23-wiring-tile-entities.md` | в¬њ |
| 24 | Spawner / invasions / world events | `24-spawner-invasions.md` | в¬њ |

## Wave 3 вЂ” entity deep dives & DONE re-verification
| # | Module | File | Status |
| --- | --- | --- | --- |
| 25 | NPC AI families (aiStyle dispatch) | `25-npc-ai-audit.md` | в¬њ |
| 26 | Projectile AI families (aiStyle dispatch) | `26-projectile-ai-audit.md` | в¬њ |
| 27 | Mounts / pets / minecart / grapple | `27-mounts-pets-vehicles.md` | в¬њ |
| 28 | Buffs & debuffs (apply, UI, sync) | `28-buffs-debuffs.md` | в¬њ |
| 29 | Town NPCs (housing/happiness/pylon) | `29-town-npc-housing.md` | в¬њ |
| 30 | Achievements / bestiary | `30-achievements-bestiary.md` | в¬њ |
| 31 | Golf / ball physics | `31-golf-ball-physics.md` | в¬њ |
| 32 | Sound / audio engine | `32-sound-audio.md` | в¬њ |
| 33 | Localization | `33-localization.md` | в¬њ |
| 34 | ID sets completeness (Terraria.ID.*) | `34-id-sets-audit.md` | в¬њ |
| 35 | RNG & data-structure utils | `35-rng-utils.md` | в¬њ |
| 36 | Cinematics / films / Old One's Army | `36-cinematics-film.md` | в¬њ |

## Wave 4 вЂ” infrastructure & port-side audits
| # | Module | File | Status |
| --- | --- | --- | --- |
| 37 | ReLogic libs (content pipeline, XNA, JSON) | `37-relogic-libs.md` | в¬њ |
| 38 | Out-of-scope confirmation (Social/Steam/Testing) | `38-out-of-scope-audit.md` | в¬њ |
| 39 | Rust port architecture inventory | `39-port-architecture.md` | в¬њ |
| 40 | DONE hot-rows re-verification (npc/entities/projectile) | `40-port-done-verification.md` | в¬њ |
| 41 | Render pipeline vs C# graphics layer | `41-render-pipeline.md` | в¬њ |
| 42 | Save/load (IO) + Stage-2 compression hooks | `42-save-load-audit.md` | в¬њ |
| 50 | Entity-AI census & re-audit baseline (full roster, wiring wave worklist) | `50-entity-ai-census.md` | в¬њ |
| 51 | Twins/Skeletron boss audit (census slice B-1: Retinazer mislabel, Skeletron head/hands, Prime head+limbs) | `51-audit-twins-skeletron.md` | ✅ |
| 53 | Celestial Pillars + DD2 lanes audit (census slice B-6; code-only) | `53-audit-pillars.md` | в¬њ |
| 42 | Save/load (IO) + Stage-2 compression hooks | `42-save-load-audit.md` |
| 54 | Census A-1: most-played early-game AIs (slime/eye/fighter/KingSlime) | `54-audit-slime-eye.md` | 🟡 WRONG×3 + KS defects | в¬њ |
| 55 | Worm SIM-PASS fidelity audit (census slice A-2: AI_006 whole vs update_worms_in; Wyvern/EoW specials; Destroyer boundary notes for R55) | `55-audit-worms.md` | 🟡 |
| 60 | Item-use census (S1-P-i01: Player.ItemCheck vs world_sim/editing.rs use path; U1-U39 rows + 11 fix slices) | `60-item-use-census.md` | 🟡 |
| 61 | Weapon data census (S1-P-i02: all 624 weapon rows, Item.SetDefaults vs itemdata.rs; 18 fields + 25 seed variants; 0 WRONG / 0 MISSING) | `61-weapon-data-census.md` | ✅ |
| 62 | Ammo & Potion census (S1-P-i03: PickAmmo order/rolls vs ammo.rs; PotionSickness 21 / ManaSickness 94, per-item delays, quick-use keys; code-only) | `62-ammo-potion-census.md` | 🔴 P1 heal-gate + A1/A2 defects |
| 63 | Special-bullet effects census & port (S1-P-i18: the 11 aiStyle-1 bullet specials + Luminite; all MISSING at base — no proj_data rows; code-only) | `63-special-bullets-census.md` | ✅ |
| 69 | Liquid 4-way reconcile (W4-g46 merger assist: the g45/g46/g47/f25 per-fn KEEP table + the graft-prep UNION record + the W4-g45 ADDENDUM; A7 superseded by A8 — g46d LANDED `9bc722e5`; **A9 LIQUID SEAL — the ONE authoritative fold-list for g46e** reconciled vs g47 `ca76d68ee` §15: `2bbadd9bd` hunk verbatim, 10-row fold table, 102→109+2 pin matrix, g15 push-owed probe; **A10 monitor pass 2** — g15 push-rescue LANDED `7981aa5bf` (A9.4 discharged), union `9bc722e5f` still pre-graft) | `69-liquid-reconcile.md` | 🟡 A9 fold-list pending g46e (seal not done until the down-arm hunk is in) |
| 104 | DOC-SWEEP lane 24 liquid census + the bucket port (W4-g45: QuickShimmer phantom / no pressure constant / the flow-rate table; the down-arm live reads + ItemCheck_UseBuckets + the shimmer-flow & fall-arm pins; §5-final owner table; §6c-compose g15 contract; **§7 SEALED — the liquid-family receipt → g46e-bible**; code-only) | `104-doc-sweep-liquid.md` | ✅ SEALED |
| 121 | Shimmer doc-sweep FINAL (W4-f47 lane 36: the splash arms + faithful wet probes ported, the town state-25 seam wired with the b3 contract, the ShimmerSlime 676 row + light/dust arm wired, the transform-table census clean, the g2 Shimmer() cross-note; the e19 keep-one fold — the 16 box + the compile-break fixes; code-only) | `121-doc-sweep-shimmer.md` | ✅ |
| 277 | Item splash KEEP-ONE (W4-f47 × W4-e19 2-way: the arm-by-arm verdict — the f47 Splash latch kept, the e19 16-box fix + box_is_wet + pass-level witnesses folded, the wet-probe 10x8 census aligned, the 676 aiStyle-1 light/dust wiring; RENUMBERED 144 → 277 per registry §6.3 — 144 belongs to e19 §7.1; code-only) | `277-item-splash-keep-one.md` | ✅ |
| 278 | Splash keep-one ARBITER FINAL v1 (RENUMBERED 144-F → 278 per registry §6.3 — 144 belongs to e19 §7.1; SUPERSEDED by 379, formerly 144-F2 — its verbatim-gate tiebreak mooting by the e19 seal's fix; kept as the lane record: the sticky-vs-local lemma, the 145 repin reconciliation, the census-trio folds) | `278-ARBITER-FINAL.md` | ⛔ |
| 379 | Splash keep-one ARBITER FINAL v2 — THE definitive verdict (RENUMBERED 144-F2 → 379 per registry §10 — 144 belongs to e19 §7.1; BOTH prior verdicts superseded; adjudicated at f47 18fe75f2e / e19 0fb0ac4ad with BOTH entry gates verbatim: the keep reduces to witness SUPERSET COVERAGE — KEEPER = f47 Splash/splash_due/ItemDrop::splash, the latch-lifecycle + draw-count twins e19 lacks, the e19 quartet is the budget-broken copy, the e19 gate witness folded as w4_f47k2; the exact e19 drop-hunks re-pinned at 0fb0ac4ad, the e19 seal's 8-row table corrected; code-only) | `379-ARBITER-FINAL-v2.md` | ✅ |
| 151 | Shimmer-family SEAL (W4-f47 seal phase: the five cross-lane compositions — the e19 lights/AddLight mutual-receipt DISJOINT + the lunar-brick RAW-enum fold, the 676 HitEffect census g62-hit/g60-death + the death-half seed, the b1 activation-contract, the d6 ShimmerImmunity-131 compose disjoint, the d43 676-row keep-one; the family bible single-owner map + merge checklist; code-only) | `151-shimmer-family-seal.md` | ✅ |
| 144 | Item liquid-splash keep-one verdict (W4-e19 final + SHIMMER SEAL: the f47 keep-f47 counter-verdict consumed, the e19 keep UPHELD — the entry gate made verbatim (lava_now), the 8-row f47-drop-hunks table published, 2 f47 probe twins folded + 1 gate witness; TR_WATER_STYLE pin adopted) | `144-splash-keep-one.md` | ✅ SEALED (arbiter fallback) |
| 145 | The 16-box cross-lane repin list (W4-e19 final + SHIMMER SEAL: SIZE 14→16 per WorldItem.cs:138; per-lane OWN-BRANCH execution instructions at every lane's current tip — grab edge 42/43, magnet 292/192, floor−SIZE, f49 nebula +92/−8; the g15 line drift corrected) | `145-16box-cross-lane-repins.md` | ✅ execution published |
| 146 | The AETHER biome census (W4-e19 final + SHIMMER SEAL: ZoneShimmer metric/music faithful; bg-skip + star-dim + monolith ramp ported; liquidAlpha[13] dead-code; the g69d-star-visual boundary CONFIRMED disjoint at the four C# windows, g69e-clouds-retire reads) | `146-aether-biome-census.md` | ✅ census + boundary sealed |
| 147 | The quicksell SHIFT-click UI-hint census (W4-e19 final + SHIMMER SEAL: the C# mouse hint IS the cursor-10 icon; the override draw ported; the g86e-cursor-170-sites COMPOSE published — cursorOverride layer 36 vs cursorItemIcon layer 40, keep BOTH, the two reset clocks separate) | `147-quicksell-ui-hint-census.md` | ✅ compose published |
| 148 | W4-e19 research FINAL (SEALED: the full arc + the SHIMMER-seal round — the arbiter consume, the repin execution, the aether boundary, the cursor compose, the shimmer-family bible in ANALYSIS; the witness ledger, the five merge contracts, the standing deviations) | `148-e19-research-final.md` | ✅ SEALED |

## Fog of war (not yet specified)
- Mod-era entity families beyond 1.4.5.8 (Calamity/Fargo scale) вЂ” enters when the
  counts question is ticketed.
- MP transport design details вЂ” blocked on the engine design doc (separate doc).
- tModLoader-style mod API surface вЂ” only if the destination is redrawn to include it.
