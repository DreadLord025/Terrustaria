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
| 64 | Town-spawn census (W4-f43: UpdateTime_SpawnTownNPCs tail arms - pet licenses 637/638/656, town slimes 670/678-684, Santa 142, Princess flag9; the 701-707 directive ids corrected to the real NPCID set; code-only, unwired) | `64-town-spawn-census.md` | ✅ (SPLICED by 368) |
| 368 | Pet-arrival family FINAL + the f43b SPLICE (W4-f43, a6f Success-leg contract leg 2+3; §9 rename: the post-§9 352 self-pick was contested — 352 = d1's EXECUTED `194c66ce6` 352-d1-falling-block-final — row 368 per Lead amendment): the arrival gates wire to the g10 kinds (LADDER 21->32, condition_holds pet/slime arms + full Princess flag9), spawn_town_tail_unique_data (GiveTownUniqueDataToNPCsThatNeedIt name/roll/name on the UpdateTime genRand-alias), exchange_resident_pet (the live-Npc RerollVariationForNPCType adapter) + the a6 else-arm one-line Success/Fail dispatch on UpdatePlayers; the flags.rs:608 load-gap fix (6 slime World fields + .wld read/write + .trwld v18 11-byte tail + the per-record variation byte); d43-town-pets DUP keep-one g10-canonical; the genRand=Main.rand alias census correction; the pet-arrival-family declaration; code-only, witnesses WRITTEN NOT RUN | `368-pet-arrival-family-final.md` | ✅ FINAL |
| 138 | POWERUP master reconcile SEALED + ARBITER receipt (a6 ⨉ g87/g88/g89/g90 ⨉ c36/f33 ⨉ f44/f45: per-item keep-one adjudication incl. the pushed g87 dfbcaf94b arm-by-arm §10 + the closeout 26620538e cross-receipt §11.3, persist-layer split, .wld byte-compare, the .trwld v18 union re-aligned §11.2's 101 → the LANDED 107-byte head + W×H dense §12.1 — **g88 d09d7e10a research/103 §L3-ARBITER FINAL A3 authority** (THE ONE table, D3 restated verbatim — A2 OVERRULES 92's books-in-prefix order under the C#-definitive brief, the fc5cf11a SEAL's D5 CONFIRMED; the seal's g71d pick corrected by the 9a4acd729 D5-RE-PIN), a6's seats pets 51-53 → 57-59 / books 55-56 → 61-62 STAND, the 103↔138 ARBITER mutual receipt §12.4 (A4's 138-row closed) + the SEALED-receipt §12.5, .trplayer v11 receipt — the g88 six-line v7 rule + torch line LANDED §11.1, a6's five fidelity-fix receipts §10.7/§11.4 + the exchange-fail fold, the Success-leg ACTIVATION CONTRACT §12.2 (**g10 5dee28650 kinds DONE** → f43-splice (f43b) → a6's one-line dispatch, the exact hunk published), the powerup-family bible §12.3, witness union, merge order; code-only) | `138-powerup-master-reconcile.md` | ✅ SEALED + ARBITER |

## Wave 4 lane FINALs (registry-150 §8.7 new-doc rule, ≥360)
| # | Module | File | Status |
| --- | --- | --- | --- |
| 85 | DOC-SWEEP lane 12 mana/magic (W4-f49: the nebula/4143 consume arms + NebulaLevelup PORTED; drink-while-sick / magic-cuffs / buff-6 ladder censused + witnessed; crystals skipped — g88b; code-only) + MANA RESIDUALS R1-R5 (the blaster CheckMana gap PORTED, the R3 one-writer contract) + FINAL-STATUS (R3 EXECUTED: the scan arms deleted, the fold_orb bridge stub + the update_survival lift; the five CheckMana sites' d-lane census — d10/d19/d21 landed 3 of 5; the prism flower end-to-end pass-tail witnessed; the f12d/f49-contract receipt) + RETIRE SEALED (RT1 the f12 mutual bridge receipt — their R3-NOT-landed monitor STALE, 8eb20d8b landed — + the merger-remainder-list FINAL; RT2 the d79 three-way per-fn keep-one table, the THIRD nebula copy ruled; RT3 the MANA-FAMILY merger-bible declaration) | `85-doc-sweep-mana.md` | ✅ SEALED |
| 360 | W4-g10 lane 3 pets FINAL — the SEAL-VERIFY receipt (g10b phases landed, ALL-11 wired census, the f43b splice contract receipt, the d43 keep-one) + the pets-family bible | `360-g10-pets-final.md` | ✅ SEALED (code-only; 26 witnesses written not run) |
| 263 | W4-g87 closeout fold-input + RETIRE (RENUMBERED 143 → 263 per research/150 §6.3 — c4 keeps 143): §1-§5 the closeout record (the a6 dup adjudication, a6's 4 fidelity deltas folded, the 5289 flag-gate hypothesis REFUTED — both ALWAYS-spawn; the slot-8 UI/draw census + port); §6 the g87c retire receipts SUPERSEDE the fold tables — the dh line g88h DEFAULT-DROP (138 §10.4 / 132 §2f, DD2 keeps v12), the pets seat [57..60] in the SEALED 107-byte v18 union (103 L3-ARBITER A3, the 24/68-byte tables retired), the use-arm survivor seats on a6's machine (138 §12.3), the pet-license Success leg activation-contracted (g10 DONE → f43b splice → the one-line dispatch, the g87-side hunk in code comment), the FINAL demonheart-family bible | `263-g87-fold-input.md` | ✅ SEALED (the use-arm keep-one + the Success leg EXECUTED at batch-L; the dh DROP deferred — g88/f44 not in this batch) |

| W4-d79-final | PITCH FINAL + **SEALED**: the 3-way Nebula keep-one owner-table (d79/f49/f12) + the f49 mutual receipt (§S1 — AGREED vs RT2, four identical-twin survivorships resolved to f49, the bridge-ASSIGN-with-deletions one-commit contract), the LegacySoundPlayer ALL-CASE census (0-42 **43/43 PORTED** — case-39 Drip landed at the seal `b2c9aa1d2`, the GoreSound seam), the d80 compose-note (re-censused at `a6bcf736a`, zero bare overlap holds), the per-lane rewire-table at the CURRENT tips (§S2 — g21 `7596fb262` +5, e10 `8e0d951fc` +0-bare, the ledger **48 targets + 3 deletes**), the six E21 task-seeds with owner-lanes (§S4), the pitch-family bible (§S5); code-only | `W4-d79-pitch-final.md` | ✅ SEALED (the d79-side deletions + the cue rewire EXECUTED at batch-L; the f12 bridge half stays its lane's) |

## Fog of war (not yet specified)
- Mod-era entity families beyond 1.4.5.8 (Calamity/Fargo scale) вЂ” enters when the
  counts question is ticketed.
- MP transport design details вЂ” blocked on the engine design doc (separate doc).
- tModLoader-style mod API surface вЂ” only if the destination is redrawn to include it.
