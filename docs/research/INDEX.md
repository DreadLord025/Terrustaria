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
| 271 | g52c cross-input: research/40's 34 DONE statuses × the g50 net lane 81-120 (zero claim conflicts; the hurt-floor inheritance ×2 + the 21/34/20 union keep-one notes ×3; 139 stays RESERVED for g52c-net-master) — renumbered 154→271 per the research/150 registry §6.3 (`72226e500`; 154 was registry-reserved for c33) | `271-g52c-cross-input-40-statuses.md` | ✅ |
| 272 | The f11 minecart fail-tile tail-fix upstream hunk (W4-g48 NET-APPLY-1: the kill_tile no_item→KillTile(fail) no-op fix published for f11f-seal/the merger; RENAMED chain 237→239→272 per research/150 round 2 §6.3 — 237 belongs to e23's earlier push, 239 to d42's 12:55:30 push (g61's chain →273); the f11e-retire consumption receipt: tip 14ca0a8d5 still carries the defect, blob 5efef353 unchanged; the ids 1-40 FINAL census + the g52c-139 receipt + the tail SEAL live in research/17; code-only) | `272-f11-tail-fix.md` | ✅ |
| 310 | The 18 SetTime send-switch instruction for the c28 lane (W4-g48 NET-APPLY-1 correction: the C# has ZERO SendData(18) sites — the clock rides 7 WorldData, event edges + the 3600-tick UpdateServer heartbeat; the exact 5-step internal-205→id-7 switch, the c27 18 reader kept wire-compat; RENAMED chain 244→281→310 per research/150 UNIFIED FINAL §9.2 (d54 `b243c7086`) — the 281 self-rename collided with f26's §7.3 assignment, §9.2 assigns 310; g48's 244 KEY WIN stands, only the doc filename renumbers; code-only) | `310-c28-18-send-switch.md` | ✅ |
| 318 | The 39 ReleaseItemOwnership item-lane task-seed (W4-g48 NET-APPLY-1 correction: the lane adjudication — not g49/g50/g51, not the landed M6-NET-5 27/28/29 row, the ITEM lane/d98 region owns it; the WorldItem.cs:188-285 FindOwner/ReserveFor scope pin + the unrowed reservation-fields census + the peer-inventory dependency; RENAMED chain 245→282→318 per research/150 UNIFIED FINAL §9.2 (d54 `b243c7086`) — the 282 self-rename collided with f26's §7.3 assignment (7i), §9.2 assigns 318; g48's 245 KEY WIN stands, only the doc filename renumbers; code-only) | `318-39-item-ownership-seed.md` | ✅ |

W4-g48 NET-APPLY-1 (2026-11-21): research/17's tail is SEALED — the ids
1-40 apply-status table is FINAL (the declaration + the g52c-139 receipt
live there); row 17 stays the netcode master file. CORRECTION phase:
the lane's 237 doc self-renamed 239, then the REGISTRY-RENAME round
(research/150 round 2, d54 `72226e500` §6.3) landed the final chain
237→239→**272** (237 = e23, 239 = d42, both earlier pushes); the
f11e-retire consumption receipt (the defect still live at f11 tip
14ca0a8d5) + the c28 send-switch publish (chain 244→281→**310**) + the 39
item-lane seed (chain 245→282→**318**) — the 244/245 pre-publish KEY
WINS stand (§7.1/§7.4); the round-2 self-renames to 281/282 collided
with f26's §7.3-assigned block and the UNIFIED FINAL (research/150
§9.2, d54 `b243c7086`) re-assigns **310/318** — only the doc filenames
renumber; the correction receipt appended AFTER
the seal in research/17 (re-probe citations only, the seal stands).
| 359 | W4-g51 SEAL: the net TE-family FINAL verification (the 8 BIG apply arms + 139 + 145/148; the g50 TE-store superset keep-one + the f21 byte-equal codec re-pin + the f30 dryad compose/995-row-dup; the net-121-family bible; registry §9.2 assignment 309→359, d125-KEEPS-309) | `359-g51-net-te-family-seal.md` | ✅ SEALED (code-only, witnesses WRITTEN NOT RUN) |
| 63 | Special-bullet effects census & port (S1-P-i18: the 11 aiStyle-1 bullet specials + Luminite; all MISSING at base - no proj_data rows; code-only) | `63-special-bullets-census.md` | ✅ |
| 284 | g52c net final-audit: the relay-host arms census (SEND side; the g48 FINAL-table feed, the 34 destroy keep-one, the 34/162 codec census; 139 stays RESERVED for g52c-net-master) | `284-g52c-relay-host-arms-census.md` | ✅ code-only |
| 276 | Net APPLY audit (W4-f21b: the g48-g52 tip coverage census, the remaining-gap list 39/41/43/48/52/79/58-client/TE/144/150, the union keep-one notes; 57/76/58-host small arms WIRED; code-only; renumbered 239 → 276 per research/150 round-2 §6.3 — 239 belongs to d42's earlier push) | `276-f21b-net-apply-audit.md` | ✅ |
| 332 | W4-c27 SEAL: the 18 SetTime lane FINAL (reader/codec/apply landed 6fb7bcb25/412b6bc0d) + the clock-family bible (one owner per body; 205→id-7 send switch = c28 PENDING per g48 244; sunModY/moonModY parked with the full title-screen-drag scope pin; code-only, 3 witnesses WRITTEN NOT RUN) | `332-c27-set-time-final.md` | ✅ SEALED |

## Fog of war (not yet specified)
- Mod-era entity families beyond 1.4.5.8 (Calamity/Fargo scale) вЂ” enters when the
  counts question is ticketed.
- MP transport design details вЂ” blocked on the engine design doc (separate doc).
- tModLoader-style mod API surface вЂ” only if the destination is redrawn to include it.
