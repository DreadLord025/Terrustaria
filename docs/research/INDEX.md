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
| 78 | DOC-SWEEP lane 6: weather/events census (W4-c40: severity ramp / wind→projectiles / rain start-stop / lightning strike / wind tiles — all landed or sibling-owned; the FALSE "SandstormSky dead code" claim corrected; calm-intent bands witnessed; code-only) | `78-doc-sweep-weather.md` | ✅ |
| 63 | Special-bullet effects census & port (S1-P-i18: the 11 aiStyle-1 bullet specials + Luminite; all MISSING at base — no proj_data rows; code-only) | `63-special-bullets-census.md` | ✅ |
| 270 | W4-c10 waterfowl band FINAL (aiStyle-68 + walkers + 1080 drop: the one-paragraph bible, band-complete ledger, the b3/d20/d76/d36/g8/d40 cross-lane contracts incl the roster-tail + d76 keep-ones, 37-witness index, merge checklist; code-only; renumber chain 145→149→270 — g89's rod doc pushed first at 145, e20's devices bible pushed first at 149, see the doc's renumber notes + d54's 150 registry §6.3) | `270-waterfowl-band-final.md` | ✅ SEALED |

| 315 | The convert-tables family bible (W4-c32-convert FINAL: the c32 tables-only census, BiomeDungeonRoom→DungeonUtils.ChangeTileType NOT Convert, f25's zero-call-site 15/16/18/19 cells, g42's FOUR Convert(3) sites, the whole-switch ownership map + tip hashes + merger contracts; number 315 per registry 150 §9 UNIFIED FINAL, d54 b243c7086, chain 281→315 — the original 281 self-pick per the ≥281 free block, fresh 362-tip sweep) | `315-convert-tables-family-bible.md` | ✅ |

| 68 | FindFrame wire-census (W4-d44 merger assist, refresh 5: per-kind ported/wired tables, g6/g7 twin-hook union plan, the FindFrame-family declaration §3.0/§10.4 [f26 7g-7k + f27 e10-e13 seams, the d38-kinds+d44-hooks merge-compose]; 198 WIRED [the wave-B residual 294/296 the LAST stand-in pair], the 480/481/482 trio RESOLVED keep-inline, the 634 SporeBat RNG-seam contract published, ZERO true no-coverage UNCLAIMED at the probe-time tips [53 module-claimed + 77 roster-arm-covered]; code-only) | `68-findframe-wire-status.md` | ✅ |

| 198 | World-lane dust WIRING keep-one + seam census + mount-dust-family bible (W4-f6: e4 keep-one, g68 tile_draw_rand ONE-field, UseDrill activation contract, minecart 4 delegates + staged fart cloud, SEAL receipts - d54 131 s8.1 push-owed DISCHARGED; rename chain 67→198 round-1, 67 stays d54's until the dust-union merge) | `198-dust-world-wiring-keep-one.md` | SEALED |

| 65 | Main.cs dust sites 1-8 (g11: snowing + TeleportEffect 0-5/7; wiring + union-exec pointers; DUST-RETIRE final sweep: every Main.cs dust-family window vs the union-table — ZERO unclaimed sites) | `65-dust-main-sites-g11.md` | FINAL (union-table closed) |
| 83 | Weather reconcile, g11 addendum (snowing keep-one: d130 base + 7 exact g11 folds; §-patch F1-F7 as EXACT patch-hunks; FINAL: the §-adjudication re-walks the :17238 SwapRandom scope — the g11 pre-swap/global claim STANDS; d130 tip 560a02ea executed F2/F3 + kept its splitmix by its own §2 merger-delegation, F1 stays the recommendation; g11's snowing marked DEPRECATED in-code for the merger drop; DUST-RETIRE §-receipt: d130e APPLIED F1+F5+F7 at 099df89e2 — splitmix retired, global UnifiedRandom, g11 mirrors ported; cull-kill at local tip 16a792d56, origin still 560a02ea — push-rescue flag; merger delta = the DROP only) | `83-weather-reconcile.md` | FINAL + RECEIPT (F1 APPLIED on d130) |
| 116 | TeleportEffect 4-way union (g11 wired 0-5/7 vs g61 widest-11 vs d54 latent-10 vs g12 staged-10/11/13: per-style table, Entry-OLD->Exit-NEW queue canonical, 6 wiring gaps, keep-one = g61 body + g11 seams) — EXECUTED 56afc0c0+8540a8ad + FINAL §5/§6: the g61b co-lane cross-verify (no spawn dup; the fade PRE-AI re-slot + the statue style-14 queue latch folded), the main_dust.rs g11×g13 filename union-plan (ONE-file per-fn keep table for d54f-131); 33+3+2 witnesses; g12/d54 drops are the merger's; DUST-RETIRE §7: g61b probe ALIVE at b5ff1803d (+9 local, seam-6 key-summon NO DUP) + the ONE-paragraph merger instruction for dust-master 131); SEAL §8 (2026-11-21): the 4 receipts — the union-chain census 9a9ac1fd/98283df0/2e978b279 (6 gaps re-verified in code, seam-6 NO-DUP), the g61d spawn-seal MUTUAL at pushed 6e922fc6 (the §7.1 push-owed flag RETIRED; 7 DEPRECATED banners re-read g11-canonical; poof/mimic/transform/play_teleport_cue STAY g61), the d54 bible 131 §8.3 item-4 main_dust THREE-WAY per §6's per-fn table, the teleport-family bible (ONE body + ONE seam set, fold-sources DROP, witness census ALL WRITTEN NOT RUN); no further g11 commits planned) | `116-teleport-reconcile.md` | SEALED (EXECUTED + FINAL + RETIRE) |

| 366 | THE ADDLIGHT-FAMILY BIBLE (W4-g18 SEAL): the git-log census (every S1-E-5 wave-4 lane at its pushed tip), the d54-bible reconcile (Dust 94/94 + proj-107 86-covered/21-absent-rand-gated + NPC 80 + Main 5), the c29 gems-NEVER-emit-light premise REFUTED (no tileLighted write / dust-43 sparkle / no scanner case / g18-negative-census-agrees), the e6-addlight-proj-2 compose mutual receipt, the family declaration; number 366 per registry-150 §9 UNIFIED FINAL row 366 (chain 304→338→341→344→360→366; g10-360 KEEPS first-add, g18 fourth claimant); 6 family-census witnesses WRITTEN NOT RUN | `366-addlight-family-bible.md` | ✅ SEAL |

## Fog of war (not yet specified)
- Mod-era entity families beyond 1.4.5.8 (Calamity/Fargo scale) вЂ” enters when the
  counts question is ticketed.
- MP transport design details вЂ” blocked on the engine design doc (separate doc).
- tModLoader-style mod API surface вЂ” only if the destination is redrawn to include it.
