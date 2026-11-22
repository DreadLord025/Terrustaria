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
| 26i | g65 Kill-bodies keep-one receipt (the d78c blasts cross-lane publish: 234-census CLOSED 406=393+6+7, the d77/g3 census drop-hunks, the swept-618 inline KEEP) | `26i-g65-kill-keep-one.md` | done |
| 346 | g65 Kill death-dust-family bible FINAL (the 309→316-kinds census correction, the 406-site closure re-scan, the d78/g1 CURRENT-tip re-verification + the 29 merger drop-hunks: 14 census + 3 gun smoke-puff per g63-298 §3.3 + 12 g16 gore-compose per §2c (disjoint pools / shared stream, the per-kind 706/709/503/178 gates, the bomb-family no-double-feed, the 503 unconditional-draw fix), the deferred set task-seeded S1-P-i58/59/60, the family declaration; chain 245→308→346 per registry-150 §8.7 FROZEN assignment) | `346-g65-kill-death-dust-family-final.md` | ✅ SEALED |
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
| 351 | Gore-sites family bible + the e5-vs-f10 HitEffect keep-one (W4-e5: the 1578-site census re-verified 1345/165/36/32/0; the 551 flipped-twin overlap RESOLVED — e5 drops :82206-82215, f10 keeps its betsy_flipped_death; the 5th hit-half :82045 found, g62 emit-at-wiring; the d55 compose + the registry-150 free-number take; code-only) — renumbered 304→351 per FROZEN registry-150 §8.7 | `351-gore-sites-family-bible.md` | ✅ SEALED |
| 111 | Gore-family FINAL verification + completion receipt + SEAL (W4-f5: the 853-site wave-D recount 849 death/4 hit, 849/849 citation-covered; g60 §12.1 remaining-window closed b86-103; SEAL §7 the 4 hit-half bodies' exact wiring-assignment — g62 emit-at-wiring ×3 + wiring-caller 133 latch, d55 NOT extended; §8 the f10-overrun delete re-censused at f10 LIVE tip d197124d1 b157 — 131 fns 3672-8180 / 128 gates 8604-9179 / 138 witnesses 11527-14415 + the wyvern-twin/substring traps + re-census-at-merge; §9 g60f death-bible compose + the f5↔DeathFx keep-one = disjointness + the king_slime_spawns field fix; §10 the 1578-site bible FINAL one-paragraph + gate-command; f5 dust 43/43; code-only) | `111-gore-final-verification.md` | ✅ SEALED |
| 364 | Proj-gore-family FINAL + SEAL (W4-g16: the 165/165 coverage + the 1578 gate re-run, the g65-Kill compose — 316-kind drain DISJOINT bodies, the 12-hunk gore-stub drop-table, the 706/503 caveats, the family bible; number per registry-150 §9 row 364 — sealed as 360, chain 247-invalid-self→360→364, g10-360 first-add KEEPS) | `364-proj-gore-family-final.md` | ✅ SEALED |
| 339 | W4-g17 research FINAL + the GORE-WORLD-family bible SEALED (registry 339 per §9.2 — renamed from 304, §8.6: post-§7.4-contested, d50 KEEPS 304; chain 304→339; the 68-site census Main 0/WorldGen 36/Player 32 re-counted, the full-family 1627-site per-variant table with owner lanes; the i30-compose ONE-throw witness; DrawNPCChatButtons DEAD + the 6 unported chat buttons pinned; code-only) | `339-g17-gore-world-final.md` | ✅ SEALED |
| 334 | W4-g62 hit-dust FINAL + hit-dust-family declaration (number per registry-150 §9 UNIFIED FINAL, d54 tip b243c7086, row 334; chain 240→334 — 240 belongs to g58 WINS `cdfe3d108` 12:16:37): the FOUR hit-half gore bodies :86532/:86947/:86951/:88648 OWNED+ported (the f5 research/111 hand-off, batch 35 — GorePool param, the 133 eye-pop latch capture-gate + strike-entry localAI[0] write + drain body), the g60f-widening MUTUAL RECEIPT re-verified by g62 itself (g60 30fea0fe1 vs g62 32af98b44: fields 34/34 init 8/8 update_trails 40/40 Transform 7/7 witnesses 105/105 table 19/19 ALL BYTE-IDENTICAL), 629 live sites zero missed — FAMILY COMPLETE; 69 witnesses written not run; code-only | `334-g62-hit-dust-final.md` | ✅ |
| 110 | NPC death dust × gore keep-one merger playbook (W4-g60: per-kind 1-697 dust/gore single-owner matrix + the d55/e5/f10 strip-lists + the g59↔g60 boundary proof + §11 gore-remainder re-census: the 66 re-classified 24 gore-f10 / 32 gore-f5 / 10 gore-none, zero unclaimed bodies + §12 FINAL STATUS: the lane-tip re-probe, the f10 :85560 overrun, the g59∩f5 strip extension, the per-row mechanical strip checklist, the npc_death_dust.rs union / last_hit_dir ONE-declaration / update_trails ONE-fn contracts + §12.8 BIBLE PHASE / FINAL-FINAL: the TRAILCACHE WIDENING EXECUTED byte-identical to g62's 6-hunk contract (`19431e385`, the ONE-widening item CLOSED), f5 CHAIN COMPLETE 853/853 (remaining window 0), the f10-overrun drop-hunks EXACT per-fn line-ranges (88 fns / 85 gates / 97 witnesses at `91b082e3c`, three contiguous bands + re-census scripts, the 133 eye-pop latch hand-off), strip totals 204 actionable / 0 conditional (g59∩f5 = 93), the union-8-step re-pinned; code-only) | `110-death-dust-gore-keep-one.md` | ✅ |
| 273 | W4-g61 NPC spawn/teleport-in dust FINAL + SEALED — the spawn-family bible (the keep-one vs g11's EXECUTED research/116 union — g11 teleport_dust.rs canonical, the g61 teleport family DEPRECATED fold-source, the g11 mutual receipt at its tip 2e978b279; the NightKey crimson pick LIVE on the g69 World.crimson field; the roster activation census seal-re-read — 473-476 AUTO / 478-479 SEMI-AUTO at d40 tip 8fe1d6d31 with the 478 Hide-no-ENTRIES-row fix published for d40, 158 g8 flip-gated, 159 d36-claimed, 687 nowhere d43-bare-main; the c10 census-dup keep-g61-full-body contract vs its SEALED research/149; code-only; renumber chain 237→238→273 per the research/150 registry §6.3 round-2 refresh 72226e500 — 237=e23's earlier push, 238=d42's earlier push (c8's 238 →262), first-pushed-wins) | `273-g61-spawn-family-final.md` | ✅ FINAL + SEALED |

## Fog of war (not yet specified)
- Mod-era entity families beyond 1.4.5.8 (Calamity/Fargo scale) вЂ” enters when the
  counts question is ticketed.
- MP transport design details вЂ” blocked on the engine design doc (separate doc).
- tModLoader-style mod API surface вЂ” only if the destination is redrawn to include it.
