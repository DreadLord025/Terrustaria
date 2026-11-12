# Research index — full C# Terraria audit

Map of the research effort backing the new ROADMAP. One file per module,
`docs/research/NN-slug.md`. Status: ✅ done · 🔄 in flight · ⬜ planned.

Destination: a complete picture of every Terraria 1.4.5.8 subsystem — what is
ported, what is partial, what is missing, what diverges — to compose a detailed
new ROADMAP (Stage-1 completion + DONE re-verification + Stage-2 optimisation
per `docs/superpowers` agreements).

## Wave 1 — god files & world core
| # | Module | File | Status |
| --- | --- | --- | --- |
| 01 | Main update loop / time / camera / input | `01-main-update-loop.md` | 🔄 |
| 02 | Main draw pipeline / scene layers / effects | `02-main-draw-pipeline.md` | 🔄 |
| 03 | Player core (movement, life/mana, death) | `03-player-core.md` | 🔄 |
| 04 | Player combat (hurt/damage, item use, buffs) | `04-player-combat.md` | 🔄 |
| 05 | NPC core (defaults, AI dispatch, net sync) | `05-npc-core.md` | 🔄 |
| 06 | Projectile core (defaults, AI dispatch, damage) | `06-projectile-core.md` | 🔄 |
| 07 | Item core (defaults, prefixes, tooltips) | `07-item-core.md` | 🔄 |
| 08 | Tile / Framing / TileObjectData / Modules | `08-tile-framing.md` | 🔄 |
| 09 | TileDrawing / DrawData / paint+colour | `09-tile-drawing.md` | 🔄 |
| 10 | Lighting / TileLightScanner / LightMap | `10-lighting.md` | 🔄 |
| 11 | Liquid / LiquidBuffer / modules | `11-liquid.md` | 🔄 |
| 12 | WorldGen / WorldBuilding passes | `12-worldgen.md` | 🔄 |

## Wave 2 — world & render systems
| # | Module | File | Status |
| --- | --- | --- | --- |
| 13 | Collision / HitTile / scans | `13-collision.md` | ⬜ |
| 14 | Map / MapHelper / minimap layers | `14-map-minimap.md` | ⬜ |
| 15 | Dust / Gore / UIDust | `15-dust-gore.md` | ⬜ |
| 16 | Weather (rain/cloud/wind/scene metrics) | `16-weather.md` | ⬜ |
| 17 | Netcode (NetMessage/sockets/Netplay) | `17-netcode.md` | ⬜ |
| 18 | UI framework (UIElement/UIState/layers) | `18-ui-framework.md` | ⬜ |
| 19 | Front-end screens (char/world create+select) | `19-ui-screens-frontend.md` | ⬜ |
| 20 | In-game UI (options/inventory/chest/craft) | `20-ui-ingame.md` | ⬜ |
| 21 | Recipes / crafting | `21-recipe-crafting.md` | ⬜ |
| 22 | Chests / loot / shops / shopping | `22-chest-loot-shops.md` | ⬜ |
| 23 | Wiring / TileEntities | `23-wiring-tile-entities.md` | ⬜ |
| 24 | Spawner / invasions / world events | `24-spawner-invasions.md` | ⬜ |

## Wave 3 — entity deep dives & DONE re-verification
| # | Module | File | Status |
| --- | --- | --- | --- |
| 25 | NPC AI families (aiStyle dispatch) | `25-npc-ai-audit.md` | ⬜ |
| 26 | Projectile AI families (aiStyle dispatch) | `26-projectile-ai-audit.md` | ⬜ |
| 27 | Mounts / pets / minecart / grapple | `27-mounts-pets-vehicles.md` | ⬜ |
| 28 | Buffs & debuffs (apply, UI, sync) | `28-buffs-debuffs.md` | ⬜ |
| 29 | Town NPCs (housing/happiness/pylon) | `29-town-npc-housing.md` | ⬜ |
| 30 | Achievements / bestiary | `30-achievements-bestiary.md` | ⬜ |
| 31 | Golf / ball physics | `31-golf-ball-physics.md` | ⬜ |
| 32 | Sound / audio engine | `32-sound-audio.md` | ⬜ |
| 33 | Localization | `33-localization.md` | ⬜ |
| 34 | ID sets completeness (Terraria.ID.*) | `34-id-sets-audit.md` | ⬜ |
| 35 | RNG & data-structure utils | `35-rng-utils.md` | ⬜ |
| 36 | Cinematics / films / Old One's Army | `36-cinematics-film.md` | ⬜ |

## Wave 4 — infrastructure & port-side audits
| # | Module | File | Status |
| --- | --- | --- | --- |
| 37 | ReLogic libs (content pipeline, XNA, JSON) | `37-relogic-libs.md` | ⬜ |
| 38 | Out-of-scope confirmation (Social/Steam/Testing) | `38-out-of-scope-audit.md` | ⬜ |
| 39 | Rust port architecture inventory | `39-port-architecture.md` | ⬜ |
| 40 | DONE hot-rows re-verification (npc/entities/projectile) | `40-port-done-verification.md` | ⬜ |
| 41 | Render pipeline vs C# graphics layer | `41-render-pipeline.md` | ⬜ |
| 42 | Save/load (IO) + Stage-2 compression hooks | `42-save-load-audit.md` | ⬜ |

## Fog of war (not yet specified)
- Mod-era entity families beyond 1.4.5.8 (Calamity/Fargo scale) — enters when the
  counts question is ticketed.
- MP transport design details — blocked on the engine design doc (separate doc).
- tModLoader-style mod API surface — only if the destination is redrawn to include it.
