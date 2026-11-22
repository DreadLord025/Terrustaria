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
| 300 | W4-c31 jungle residuals census + gap-fill + the DUP-RECONCILE EXECUTED (renumbered 12k→300 per registry-150 §7.3; the g42 GROWTH SEAL keeper adoption + the jungle-family bible) | `300-c31-jungle-residuals.md` | ✅ |
| 246 | Temple-family SEAL (W4-g43: the FINAL trio census, the c36/d4/c31/f24 keep-one + disjointness receipts, the merger bible; code-only) | `246-temple-family-seal.md` | ✅ SEALED |
| 93 | Spider-cave biome gen census (W4-g41: web placement / egg sacs / wall coverage vs WorldGen.cs; landed M5-STRUCT-5 audit + the egg-sac validator chain port; FINAL: ambience NOTHING-TO-PORT census + the wall-62 spawn pool (163/164 cross-noted to d36) + the PlacePot remix veto residual + the re-pin deferral final status + merger checklist; code-only) | `93-spider-cave-census.md` | ✅ all-landed (re-pin deferred to gate) |
| 94 | NOT-THE-BEES host census (W4-g41 follow-up: FinishNotTheBees + NotTheBees vs WorldGen.cs; c31 co-lane claim check — zero pushed claims, host unclaimed/taken; ILied-family wiring census; spider_wall_remover wiring; code-only) | `94-not-the-bees-host-census.md` | ✅ landed (pins unchanged, flag-gated) |
| 95 | FinishNoTraps + FinishRemixWorld census (W4-g41 finish gaps: the two remaining classic-seed Finish tails vs WorldGen.cs; d107 co-lane claim check — remix host explicitly deferred, no-traps claims micro-seed side only, both unclaimed/taken; seam census + deviation pins; FINAL: the QuickWater g47d activation addendum (fdeb897c/d222af69 read); RETIRE: receipt re-verified at g47 5c09a68b8 + the g46 RE-FOLD union 9bc722e5f — activation targets the union; code-only) | `95-finish-no-traps-remix-census.md` | ✅ landed (pins unchanged; QuickWater activates with the g46 union + f25 graft) |
| 96 | BIOME RETIRE receipts + FINAL declaration (W4-g41 retire phase: §1 g47f liquid-retire — the 3-seam QuickWater contract re-verified at 5c09a68b8 + union 9bc722e5f, g46d re-base DONE; §2 d36e hand-off — zero kind code at bcf4ac551/0eb4fee95, activation hunk PUBLISHED id-agnostic (Bottom placement, corrects 93's spawn_cave_walker recipe), band R7 767-816; §3 research/88 stalactite keep-one — d106 e2c2609c5 half verified + the 6-point FINAL INSTRUCTION; §4 the SPIDER+NTB+FINISH all-landed declaration + the refreshed merger 6-step + anchors canary; code-only) | `96-biome-retire-receipts.md` | ✅ retire complete (no unclaimed g41 work; residuals owned cross-lane) |
| 222 | Biome-residual census + the desert entrance port (W4-g40 continuation: the gap table vs the landed port, the co-lane claims, the ConvertGrassToDirt correction; the 17-deviation entrance residual closed verbatim; code-only; renumbered 94→222 per the d54 research/150 registry key-94 row) | `222-w4g40-biome-residual-census.md` | ✅ |
| 248 | W4-g40 FINAL (renumbered 246→248 at merge): the desert/marble-family declaration + the hive-smooth slice + the f42/g17/g22/g23 compose maps (the marble/granite music/growth census = NONE in vanilla; the NPC.cs:5019-5042 spawn arms task-seeded to the d40 kind merge; research number per registry-150: 246 claimed first-free but g43 temple SEAL landed 246 earlier (15:10:56 vs 16:02:53) - renumbered 246 to 248 at the batch-B merge (248 free at every tip); code-only) | `248-w4g40-desert-marble-final.md` | ✅ |
| 12f | WorldGen passes, lane 6 (W4-d106: the d101–d105 remainder — Grass Wall/Settle Liquids Again/Tile Cleanup/Stalac/Remove Broken Traps/Final Cleanup, R1–R6 landed; witnesses written NOT run, pins stale-by-design) | `12f-worldgen-passes-6.md` | 🟡 R1–R6 COMPLETE |
| 312 | W4-d106 WORLDGEN-6 RETIRE FINAL (the d102/d107 overlap census O1–O9, the g36 §6.4 compose + g41 stalactite-6-point d106-side drop-hunks, the e20 research-148 fidelity receipts, the sequential re-pin schedule, the family declaration; RENUMBERED 240→281→312 per registry-150 §9 UNIFIED FINAL, §9.2 row 312) | `312-d106-worldgen-6-retire-final.md` | ✅ SEALED |

## Fog of war (not yet specified)
- Mod-era entity families beyond 1.4.5.8 (Calamity/Fargo scale) вЂ” enters when the
  counts question is ticketed.
- MP transport design details вЂ” blocked on the engine design doc (separate doc).
- tModLoader-style mod API surface вЂ” only if the destination is redrawn to include it.
