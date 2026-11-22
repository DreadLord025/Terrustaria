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
| 66 | Projectile per-kind TRAIL dust claims + census (W4-g63, kinds ≤400 — batches 1-7, the row census + the corrected 60-style gun row per the e3 finding; registry-150 number winner) | `66-proj-trail-dust-claims.md` | 🟡 UNWIRED emitters await the merges |
| 199 | Projectile TRAIL dust activation checklist (W4-g63, for the merger — which merge unlocks which emitter + the RNG-seam reconciliations; renumbered 67→199 per registry-150; refreshed at retire: the 128/129/229 promotions + the 357 f20 seam at tip 31386266; spawn-dust seal: the gun Kill keep-one adjudicated d6-survives + the e3 exact fix-hunk pointer) | `199-proj-trail-dust-activation.md` | 🟡 merge-wave input |
| 284 | g52c net final-audit: the relay-host arms census (SEND side; the g48 FINAL-table feed, the 34 destroy keep-one, the 34/162 codec census; 139 stays RESERVED for g52c-net-master) | `284-g52c-relay-host-arms-census.md` | ✅ code-only |
| 298 | W4-g63 projectile TRAIL dust FINAL + the spawn-dust-family declaration (SEALED — the family boundaries, the e3 census-error consumption, the 12/955 anniversary EXACT two-draw fix-hunk (§3.2), the gun Kill keep-one ADJUDICATED d6-survives/g65-drops with EXACT drop-hunks (§3.3), the 128/129/229 rows re-probed current, the residual debt; renumbered 240→298 per registry-150 ROUND-3-FINAL §7.3 — g58 wins 240, the unpushed round-2 281 draft superseded (281 = f26's 7h); code-only, 50 witnesses WRITTEN NOT RUN) | `298-proj-trail-dust-family-final.md` | ✅ FINAL + SEALED |
| 338 | W4-d30 RETIRE/SEAL: NPC aiStyle band 65-72 FINAL census + keep-one bible (69-72 wired on main; 66/67/68 wiring on c8/c9/c10; 65 d30-canonical, f18/d39/d125-lacewing subsets drop; d31/d32/g65-309/d77-cues disjoint; code-only, witnesses written not run; renumbered 281→308→338 per research/150 §8.5, g13-308-executed-pre-§8 honored) | `338-band-65-72-retire.md` | ✅ SEALED |

| 303 | HeightOffset family seam + portableStool deferral (movement-lane task seed: the 5 offset getters, the fold-to-0 sites, the consumers-list, the stool machine's blast radius; the lane-owner recommendation + the 6-step task-seed for the merger/future-wave; W4-f3 retire + seal, code-only; renumber-chain 65→303, registry-150 §7.3 round-1 owed — 65 = f17's round-1 winner `65-kinds-coverage-1.md`) | `303-heightoffset-stool-seam.md` | seed |
| 66 | The f3 × g66 player-move mutual receipt (trail-final compose: WallClimb/carpet IDENTICAL keep-f3, the integer-half corner + the should_draw_wings mount-gate adopted into f3, g66's wing/boot/trail own-lanes kept, the wing-glide overlap dedupes to g66; g66 tip 248f0ff4d NOW PUSHED; W4-f3 seal, code-only) | `66-f3-g66-player-move-receipt.md` | receipt |
| 378 | The e1b-141 reconcile: e1 drop-hunks + keep-lanes (the 5-deviation wall copy + the vortex-arm-only machine + stealth:0.0 spawn dropped; the stoned/drown/splash/floor/double-jump duplicate drains dedupe to f3; the falling head capture + drain order + integer halves + sandstorm gore argument-draw adopted into f3; e1's pre-21654 own-lanes kept; e1 tip 3245d624, no e1b branch; W4-f3 seal, code-only; renumber-chain 67→378, registry-150 §10.2 — 67 = d54's dust-union lineage) | `378-e1b-141-e1-drop-hunks.md` | reconcile |

## Fog of war (not yet specified)
- Mod-era entity families beyond 1.4.5.8 (Calamity/Fargo scale) вЂ” enters when the
  counts question is ticketed.
- MP transport design details вЂ” blocked on the engine design doc (separate doc).
- tModLoader-style mod API surface вЂ” only if the destination is redrawn to include it.
