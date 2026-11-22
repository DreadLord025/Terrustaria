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
| 26h | Projectile aiStyles 171-180 FINAL band census (W4-d18) - SEALED (PROJ-171 retire: rowless+projPet ported, 978 dup keep-d12c, 601 cross-note d6; seal continuation: the 601 d6 seed R-26-13 EXACT spec, the f25 spray DUP keep-one, the d43/d33 projPet compose, the d12c superset confirm) | `26h-projectile-ai-171-180-census.md` | sealed |
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
| 63 | Special-bullet effects census & port (S1-P-i18: the 11 aiStyle-1 bullet specials + Luminite; all MISSING at base - no proj_data rows; code-only) | `63-special-bullets-census.md` | ✅ |
| 164 | Projectile aiStyles 101-110 census + port (W4-d11; renumbered 26d per registry-150: the ten-style coverage table, the DOC-SWEEP wave, the nebula-sweep producer + the sibling dedupe notes, the RETIRE closeout) | `164-projectile-ai-101-110-census.md` | ✅ |
| 355 | W4-d11 RETIRE FINAL (the band-101-110 git-log census - nebula-sweep NOT on main, branch-sole; the d77c 539/576 cue compose; the 813 three-way keep-one g64-superset + d11-row-byte-equal EXECUTED; the all-armed/rowed census + gaps; the nebula producer-unique/lifecycle-fold keep-one table + the can_ghost_heal d6-field ruling; the family declaration) — chain 245→355 (registry-150 §9.2 row 355, d54 `e9ac39459`: g48 WINS 245 pre-publish `2b61530b4`) | `355-w4-d11-retire-final.md` | ✅ SEALED |
| 127 | proj 111-120 spawns DOC-SWEEP + FINAL + RETIRE (W4-d12: the 113 six-kind sticker census, the 971/975/121 melee procs, the 5461 card cycle + recall no-active-gate, the 614 ai1 seating, the 978 row keep-d12c; SEALED at c98bd8937 — §R1–R5 receipts + the proj-111-family declaration) | `127-doc-sweep-proj-111.md` | ✅ SEALED |
| 336 | W4-d13 PET-LIGHT RETIRE FINAL (the c19 zero-site compose + the f11 retire-235 drain seal + the g10/d43 no-consume census + the classic pet single-item 3600 gap FIXED + the pet-light-family declaration; CHAIN 243→304→336 per registry-150 §9.2 UNIFIED FINAL — g58 owns 243, d50 owns 304; code-only) | `336-d13-pet-light-retire-final.md` | ✅ |
| 353 | W4-d13 band-121-130 BIBLE FINAL (the ten-arm band table: 121-123/129/130 pre-existing + 124-128 this lane, the satellite lanes 813/geyser-proxy/pointer/grant-gap, the SEAL receipts g10/d43 + c19 + f11-235 + g64-dup, the witness ledger; CHAIN 305→353 per registry-150 §9.2 UNIFIED FINAL — f4 KEEPS 305; code-only) | `353-projectile-ai-121-130-band-final.md` | ✅ |
| 249 | W4-d15 RETIRE/FINAL: the aiStyles 141-150 band ledger (all ten armed, kinds rowed, the d77c cue-12 compose + the d14/d16 keep-one tables, 51 witnesses written not run; numbered per registry-150 — 244-248 consumed) | `249-projectile-ai-141-150-final.md` | ✅ |
| 326 | W4-d16 RESEARCH FINAL (SEALED): the aiStyles 151-160 lane's 17-commit arc, the 69-witness ledger (WRITTEN NOT RUN), the five SEAL checks (the f20 F8 moonlord fix-list survival, the wind InWorld SKIP fix in BOTH copies, the afk/anchor landed re-verify, the d77 728/729 compose, this FINAL), the standing deviations; renumbered 246→326 per the 150 registry §9.2 (246 = e23's published reservation wins) | `326-d16-research-final.md` | ✅ |
| 327 | The KITE-FAMILY BIBLE (W4-d16 SEAL, the merger reads FIRST): d16 canonical 151-160 landing table, the compose map (f20's four-arg moonlord fix-list keep-BOTH, d77's 728/729 cue keep-one + d16's self-armed 761/762, d17's 161 census input, g65/g16 dust/gore), the keep-one rules (two wind copies by design, one ring-depth table, one cue arm per kind), the do-NOT-fix deviations; renumbered 247→327 per the 150 registry §9.2 (247 = d10's reservation wins, EXECUTED) | `327-kite-family-bible.md` | ✅ |
| 26g | The aiStyle-92 cloud keep-one + the W4-d19 SWORD RETIRE closeout (the d10 drop-hunks §4 at 732d3480b; the f33 949 keep-one §3 + the 93a §2 re-pin receipt §3a; the 1129/d21 contract §5; the 181–190 band FINAL table §6 + the 978 keep-d12c receipt §6a; the terra-chain compose-contract §7 + the d20c-T5 monitor §7a; the SWORD-FAMILY merger-bible §8; the SEAL-VERIFY monitor §9 — d10 D1–D13 re-pinned to e1519df75, the d18 V1–V5 drop verified EXECUTED f53de9c71, f33 bffee350f A–L/I+1 re-confirmed; the T5-LANDED receipt §9.7 — d20 77334040b, the four use-arms 23199067d feed the §7 envelope exactly, keep-d19 drop-d20 four slash-copy hunks, §9.2 NOT-landed STALE) | `26g-clouds-keep-one.md` | ✅ SEALED (retire + seal-verify + T5-landed receipt) |
| 109 | DOC-SWEEP lane 28 + SEAL: proj aiStyle 191-200 skip census + FINAL band census + sword-family declaration (W4-d20: 7 arms landed, T5 terra-use-arms + slash-row copies + ai2 drain fix landed, T4b + T4a pet-shoot-arm + 881/934/994/956 rows landed; slashes keep-d19 dedupe published; code-only) | `109-doc-sweep-proj-191.md` | 🟢 SEALED — all ten styles ARMED+ROWED; T4a-c chain landed (994 keep-alive = pet-AI lane); witnesses written not run |
| 162 | Projectile aiStyles 201-210 COVERAGE census (W4-d21: 7 MISSING ported, 206 verify-only a9, 207 empty/no kinds, 210 pre-ported; renumbered 26d→162 per registry §6) | `162-projectile-ai-201-210-census.md` | 🟢 SEALED — band CLOSED (research/340) |
| 340 | W4-d21 FINAL: the 201-210 band table + the d77c cue-1115 compose receipt + the all-styles-armed/kinds-rowed census + the proj-AI-1-210-all-decades-CLOSED ledger (renamed 308→340 per registry §9: g13 KEEPS 308 §8.5; code-only) | `340-projectile-ai-201-210-final.md` | 🟢 SEALED — 24 witnesses written not run; dirty-9 fold-2 co-lane idle NOT staged |

## Fog of war (not yet specified)
- Mod-era entity families beyond 1.4.5.8 (Calamity/Fargo scale) вЂ” enters when the
  counts question is ticketed.
- MP transport design details вЂ” blocked on the engine design doc (separate doc).
- tModLoader-style mod API surface вЂ” only if the destination is redrawn to include it.
