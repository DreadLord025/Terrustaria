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
| 131 | Dust cross-lane MASTER STATUS — **BIBLE-FINAL §8** (W4-d54: all-lane tips + per-fold VERIFIED/OWED table, Dust.cs FAMILY COMPLETE declaration + per-family spawn censuses, merger keep-one symbol checklist, witness recount 318→492 gross/≈461 net; F1-F7/g75d/116/G1-G6 closed, R5-R7+G2+e3-contracts+f6-push owed) | `131-dust-master-status.md` | ✅ consolidated final |

| 150 | Research-number COLLISION REGISTRY (W4-d54 doc-integrity: **§4-REFRESH round 2** — full 358-tip re-scan ×6 fetches, 47 colliding number keys, 123 rename assignments 151–236 + 244–280, first-pushed-timestamp winners, self-rename reconciliation f11/g24/g67/c10/e23/g61/g89/c4/g50/d42/c8/g48/g58/g56/f47/f21/d53/c42/f45, per-lane rename instructions + refs-to-update, same-path fork dedupe rules; 237–243 taken by e23/d42/d42/g58×4; 281+ FREE) | `150-research-number-registry.md` | registry |

## ⚠️ Collision-aware index note (2026-11-21, W4-d54) — READ BEFORE MERGING

**REFRESHED round 2 (registry §4-REFRESH banner + §6).** The number space
`docs/research/NN-*.md` is **collided across the task/W4-\* branches**: the
refresh re-scan of all **358** remote tips (21 038 rows, six fetches — lanes
pushed mid-round) finds **47 colliding number keys** (round 1 found 32 of 355
tips; the lane wave added 12k, 96, 108, 111, 129, 130, 137, 143–149, 237–240
and extended 26d/26e/64/68). The authoritative registry is
**`150-research-number-registry.md`** — per-number winner (earliest
first-pushed timestamp wins), loser → FREE-number assignments **151–236 +
244–280** (123 total; **237–243 are consumed by winner pushes: e23
bestiary-ui-final 237 / d42 roster-sheets 238 / d42 final-keep-one 239 / g58
happiness-final 240 + latch-hunks 241 + chat-portrait-seed 242 +
seal-verification 243**), per-lane rename instructions with refs-to-update,
and same-path fork rules. **Numbers 281+ are FREE — do NOT pick
max-on-origin+1 or first-free-at-your-last-fetch without checking the
registry AND a fresh all-tip sweep** (f11's interim 151–153, g50's 154,
g48's 237, g56+f21's 239, d53+c42's 240 and the c8/d42/g61 triple-238 all
collided that way). Pushes after the registry's final sweep self-adjudicate
per §6.3's cutoff note.

**Merger rules (dedupe per registry, not per filename guess):**
1. Numbers 08, 61–63, 90–92 belong to the inherited base census docs
   (`08-tile-framing`, `61-weapon-data-census`, `62-ammo-potion-census`,
   `63-special-bullets-census`, `90-consolidated-findings`,
   `91-post-audit-landed`, `92-roadmap-reverification`). Any W4-lane doc
   reusing those numbers is a loser and renames per registry §3 — do NOT
   index two different docs under one number.
2. When two branches carry the SAME filename: identical blob → keep one
   (clean merge); diverged blob → same-path fork, keep the earlier-first-add
   copy under the number, the later copy takes its assigned free number
   (registry §1 fork rows / §4 / §6.1). Ten fork families now: 18b, 65, 66,
   69, 70-liquid-merge-order, 71, 83, 84, 109, 122.
3. Lanes rename on their OWN branches (registry §3 + §6.3 tables); until a
   lane has renamed, the merger applies the registry mapping at merge time.
   The registry's §3 d54 rows (65→186, 66→194) are applied at/after the dust
   union so sibling-lane citations into `131-dust-master-status.md` stay live.
   **Rename chains** (record the full chain in the doc header): f11
   134/135/136→151/152/153→**233/234/235 EXECUTED**; g67 134→146→**267**;
   c10 145→149→**270**; g61 237→238→**273**; c8 143→238→**262**;
   d42 143→**238 EXECUTED+CONFIRMED** (+ **239 KEEPS** vs g56/f21); g58
   143→**240 EXECUTED+CONFIRMED** (+ **241/242/243 KEEP**); g64 66-fork→**196
   EXECUTED**; g89 139→**145 KEEPS**; g24 107→**146 KEEPS**; e23 **237
   KEEPS**; g84 **137 KEEPS**; c4 **143/144/148 KEEP**, 145-addendum→**265**;
   e20 **149/147/71 KEEP**, 146→**266**, 148→**269**; g50 154→**271**;
   g48 237→**272**; g56 239→**274**; d53 240→**275**; f21 239→**276**;
   f47 144×2→**277/278**; f45 137→**279**; c42 240→**280**.
4. 107 and 139 are now EMPTY at the tips (g24/g89 renamed away). 139 stays
   RESERVED for g52c-net-master; 143/144 are now FULL collisions won by c4
   (losers renumber per §6) — the round-1 "one-sided" note is superseded;
   the brief's f21b doc finally pushed (on 239, loses → 276). Unpushed
   sub-round docs take free numbers **281+**.
5. This INDEX's row set above is the base+d54 view; rows for other lanes'
   docs enter with their merges, renumbered per the registry. The merger
   dedupes INDEX rows by (number, filename) using the registry mapping.

## Fog of war (not yet specified)
- Mod-era entity families beyond 1.4.5.8 (Calamity/Fargo scale) вЂ” enters when the
  counts question is ticketed.
- MP transport design details вЂ” blocked on the engine design doc (separate doc).
- tModLoader-style mod API surface вЂ” only if the destination is redrawn to include it.
