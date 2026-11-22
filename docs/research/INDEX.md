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
| 341 | NPC.cs dust-site window coverage (W4-e2 SEALED; renumbered 64→285→341 per registry-150 §8.7 — d64 wins key 64, the interim 284 pick lost to g52's executed push, the post-§8 285 self-pick re-issued to 341 by the FROZEN table; the AI/HitEffect/spawn lane table + the f4 keep-one overlap register, mutual seal receipt) | `341-npc-dust-window-coverage.md` | ✅ |
| 342 | NPC dust family bible (W4-e2 seal; renumbered 286→342 per registry-150 §8.7: the 927+5 census re-count, the per-window ownership map d52/e2/f4/g61/g62/g59/g60, the keep-one symbol list, the staged-emitter wiring obligations) | `342-npc-dust-family-bible.md` | ✅ |
| 231 | PROJ-DUST master coverage census (W4-e3: the 1195-site Projectile.cs scan, the Kill-tail 15 + the AI-INLINE-40 + the AI_001 edges ported, the FINAL activation-order table + the three-disjoint-KEEP family bible; was 125 — the registry-150 rename, d125 wins the 125 key) | `231-proj-dust-coverage.md` | ✅ SEALED |
| 66 | World dust sites SECOND batch (W4-e4: orch 41-61 + mount 1-19, 40 sites; wiring wave + RainbowRodHit keep-one + orch-family bible; code-only) | `66-dust-world-sites-2-claims.md` | ✅ SEALED |

| 303 | HeightOffset family seam + portableStool deferral (movement-lane task seed: the 5 offset getters, the fold-to-0 sites, the consumers-list, the stool machine's blast radius; the lane-owner recommendation + the 6-step task-seed for the merger/future-wave; W4-f3 retire + seal, code-only; renumber-chain 65→303, registry-150 §7.3 round-1 owed — 65 = f17's round-1 winner `65-kinds-coverage-1.md`) | `303-heightoffset-stool-seam.md` | seed |
| 66 | The f3 × g66 player-move mutual receipt (trail-final compose: WallClimb/carpet IDENTICAL keep-f3, the integer-half corner + the should_draw_wings mount-gate adopted into f3, g66's wing/boot/trail own-lanes kept, the wing-glide overlap dedupes to g66; g66 tip 248f0ff4d NOW PUSHED; W4-f3 seal, code-only) | `66-f3-g66-player-move-receipt.md` | receipt |
| 378 | The e1b-141 reconcile: e1 drop-hunks + keep-lanes (the 5-deviation wall copy + the vortex-arm-only machine + stealth:0.0 spawn dropped; the stoned/drown/splash/floor/double-jump duplicate drains dedupe to f3; the falling head capture + drain order + integer halves + sandstorm gore argument-draw adopted into f3; e1's pre-21654 own-lanes kept; e1 tip 3245d624, no e1b branch; W4-f3 seal, code-only; renumber-chain 67→378, registry-150 §10.2 — 67 = d54's dust-union lineage) | `378-e1b-141-e1-drop-hunks.md` | reconcile |
| 131 | Dust cross-lane MASTER STATUS — **BIBLE-FINAL §8** (W4-d54: all-lane tips + per-fold VERIFIED/OWED table, Dust.cs FAMILY COMPLETE declaration + per-family spawn censuses, merger keep-one symbol checklist, witness recount 318→492 gross/≈461 net; F1-F7/g75d/116/G1-G6 closed, R5-R7+G2+e3-contracts+f6-push owed) | `131-dust-master-status.md` | ✅ consolidated final |

| 150 | Research-number COLLISION REGISTRY (W4-d54 doc-integrity: **§9 UNIFIED FINAL TABLE (Lead, 308–370, THE SINGLE AUTHORITY for 240+) + §10 ROUND-5 full-sweep mini-refresh (371–381, the LOW-key residue)** — rounds 3/3-ext/4/§8-FROZEN consolidated into §9: 281/282/283=f26, 284=g52, 244/245=g48, 246=e23, 247=d10-EXECUTED, **248=g82-reservation (e26→357, NOT 333)**, 249=d15, 250=d37, 304=d50, 305=f4, 308=g13; §9.2 rows 308–368 EXECUTED (49/49 §9.4) + 369=d34 DONE + 370=d54j owed; **§10 round-5 FULL 362-tip sweep (21 368 rows) found 11 ZERO-coverage losers on OLD winner keys**: new keys 75=g37 (f32→380) / 78=c40 (g78→375), late landings g46-71→374 / g1-26d→373 / c26-64→376 / g2-26d→377 / f3-67→378 / f47-144v2→379, pre-round-2 misses c14-83→371 / d78-26f→372, **f18's §6.3 249 assignment VOID (249=d15 §9.1)→381 RE-ISSUE**; §10 block 371–381 machine-checked unique/no-gaps/no-overlap; the dispatched post-cutoff brief (g63-240/d10-246/g48-244+245/f39-244/e10/f6-67) ALL already adjudicated+mostly EXECUTED; **free floor 382+; NO SELF-PICKS — execute §9/§10 ONLY**; mid-file §8-DRAFT SUPERSEDED, do not execute) | `150-research-number-registry.md` | registry |
| 367 | Merger order DAG (W4-d54, Lead-assigned per registry-150 §9.2 — 4 merge waves: roster spine per 108 §X0-X5 FINAL (d34→d38→d37→d40→d42→d35-LAST→small lanes; 395 kinds/395 ids/max 827; pins 665/583/593/slots 440→517; 59 moves/336 keeps/153 drops; f18 ZERO-rows/d39@817-823 splice; 593-sheets lazy-load MANDATORY), core (persist 103§L3-ARBITER 107-byte/liquid/atlas 113§17.5-LAST), families, UI/small; §2 merge-gate flags incl. world-hash sequential re-pins SEED42_512=124297949/DEFAULT=3647283075 stale-by-design; §3 single gate cargo test --workspace --skip migrates_version_2_kinds_to_item_ids baseline 051600da8, never force-push, ancestry+symbol verify; §4 fix-sub-branches per error cluster; §5 cleanup checklist (junctions rmdir FIRST, TR-* dirs incl TR-W4-d34seal, only-main audit); §6 doc dedupe = registry §9/§10) | `367-merger-order-dag.md` | ✅ final plan |

## ⚠️ Collision-aware index note (2026-11-21, W4-d54) — READ BEFORE MERGING

**REFRESHED round 5 (§10): the registry's §9 UNIFIED FINAL TABLE (Lead) is THE
SINGLE AUTHORITY for keys 240+ (308–370, §9.4 EXECUTION COMPLETE 49/49); §10
adds the round-5 FULL-sweep LOW-key residue (371–381). The mid-file §8-DRAFT
is superseded; the FROZEN §8/§8.5/§8.6/§8.7 are consolidated into §9. Where
any earlier section disagrees with §9/§10, §9/§10 win.**

**§10 round-5 (full 362-tip sweep, 21 368 rows): the dispatched post-cutoff
brief — g63-240 (→298 EXECUTED), d10-246 (→247 EXECUTED), g48-244/245
(WINS; docs →310/318 DONE), f39-244 (→316 DONE), e10-voice-seal (lane-prefixed,
no number), f6-67 (→198 owed) — is ALL already adjudicated by rounds 3/3-ext/
4/§9. The sweep additionally found 11 losers with ZERO prior coverage on OLD
winner keys: 75=g37 KEEPS (f32→380), 78=c40 KEEPS (g78→375), 64=d64 (c26→376),
67=d54 (f3→378), 71=e20 (g46→374), 83=d130 (c14→371), 26d=d12 (g1→373,
g2→377), 26f=f20 (d78→372), 144=e19 (f47-v2→379), and f18's §6.3 249
assignment VOID (249=d15 KEEPS per §9.1) → 381 RE-ISSUE. Block 371–381
machine-checked unique/no-gaps/no-overlap. Free floor 382+.**
The number space `docs/research/NN-*.md` is **collided across the task/W4-\*
branches**: the round-3 re-scan of all **360** remote tips (21 172 rows; +2
lanes: f12-buff-effects base-carry, g11-teleport-dust successor) finds **53
colliding number keys**, extended by §7.4's lean post-cutoff re-check of the
240/243/244/246/247 claimant tips. The authoritative registry is
**`150-research-number-registry.md`** — per-number winner (earliest
first-pushed timestamp wins; a self-renamed doc contests the NEW number at
its rename-commit time; an EXECUTED rename is honored only if UNCONTESTED),
loser → FREE-number assignments **151–236 +
246–280 + 281–303 + 304–307 + 308–337 + 338–351** (FROZEN 191 live; **237–245 are winner-consumed: e23 237 / d42
238+239 / g58 240–243 / g48 244+245 [pre-publish pushes beat the round-2
draft reservations]**), per-lane rename instructions with refs-to-update,
and same-path fork rules. **Round-3 winner corrections: 125 = d125's empress
ledger (d38 →293), 144 = e19's splash-keep-one (c4 orca →294), 145 = e19's
16box (g89 rod →295)** — earlier claimants the first two rounds missed.
**§7.4 post-cutoff extension: 243 = g58 (d50 →304), 240 = g58 (+f4 →305),
244 = g48 (+g86 →306; d53's EXECUTED 240→244 rename contested → 275
re-issued), 246 = e23's reservation (+g43 →307), 247 = d10's reservation
(g67's EXECUTED 146→247 rename contested → 267 re-issued).**
**§8 ROUND-4 FINAL (Lead direct-append, FROZEN + §8.5/§8.6/§8.7 amendments):
the post-§7.4 SELF-PICK wave on the ASSIGNED keys 281–284 and the winner
keys 244–248 renumbers per the FROZEN table at the END of the registry —
281 = f26 KEEPS (10 self-pickers → 308–317; d30 shifted → 338 by g13's
honored pre-§8 executed 308), 282 = f26 KEEPS (g48/d125/f39-seal →
318/319/320), 283 = f26 KEEPS (g59 → 321; f4 → 305 EXECUTED; c7 → 350),
284 = g52 executed-pre-§8 KEEPS (d32 → 348); 244 = g48 WINS (d43/g5 →
322/323; d43's later 304 self-pick → 343), 245 = g48 WINS (g65 → 324,
its executed 308 → 346 §8.7), 246 = e23 reservation (g40/d16 → 325/326),
247 = d10 EXECUTED KEEPS (d16-bible/g68/c3/d6 → 327–330), 248 = **g82's
published round-2 reservation WINS per §9 (e26 → 357 CORRECTION, d3/c27 →
331/332 DONE)**, 249 = d15 KEEPS (f18's §6.3 249 assignment VOID → 381
§10 re-issue), 304 = d50 KEEPS
(d119/g17/d43/e5 → 333/339/343/351), 240 (g62/f30 → 334/335), 243
(d13/f24 → 336/337); §8.7 FROZEN batch: e2 → 341+342, c24 → 344, d40 → 345,
g15 → 347, c9's 312 self-pick → 349. f39's §7.3 299 row superseded → 316
(chain 244→281→316).**
**Free floor 382+ (§10 round-5; §9 consumed 308–370) — NO SELF-PICKS: execute
§9/§10 assignments ONLY; new docs require a registry check + Lead assignment;
do NOT pick max-on-origin+1 or first-free-at-your-last-fetch** (f11's interim
151–153, g50's 154, g48's 237/239, g56+f21's 239, d53+c42+g63+f4's 240,
d50's 243, f39+d53+g86's 244, d10+g43's 246, g67's 247, the c8/d42/g61
triple-238, the 281×10/282×3/360×4 self-pick waves and the §10 low-key
landings all collided that way). §9.4 enforcement: future self-picks
INVALID; execute the §9/§10 assignment or request a round-6 refresh.

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
   (registry §1 fork rows / §4 / §6.1 / §7.1). Eleven fork families now:
   18b (d82 keeps; d83 DIVERGED →302; d81+g53 identical pair →301; e23 →246),
   65, 66, 69, 70-liquid-merge-order, 71, 83, 84, 109, 122; plus 116 =
   SAME-LANE successor fork (g11-dust-main → g11-teleport-dust): NO renumber,
   merger keep-one the successor SEAL blob.
3. Lanes rename on their OWN branches (registry §3 + §6.3 + §7.3 + §7.4.3
   + the FROZEN §8/§8.7 tables at the END of the registry; §7.3/§7.4.3
   RE-ISSUE rows replace their §6.3 originals; the FROZEN §8 supersedes
   §7.3's f39→299 row and the mid-file §8-DRAFT); until a lane has
   renamed, the merger applies the registry mapping at merge time.
   The registry's §3 d54 rows (65→186, 66→194) are applied at/after the dust
   union so sibling-lane citations into `131-dust-master-status.md` stay live.
   **Rename chains** (record the full chain in the doc header): f11
   134/135/136→151/152/153→**233/234/235 EXECUTED**; g67
   134→146→247 EXECUTED-CONTESTED→**267 re-issued §7.4.3**;
   c10 145→149→**270**; g61 237→238→**273**; c8 143→238→**262**;
   d42 143→**238 EXECUTED+CONFIRMED** (+ **239 KEEPS** vs g56/f21/g48); g58
   143→**240 EXECUTED+CONFIRMED** (+ **241/242/243 KEEP** vs d53/c42/g63/f4/d50);
   g64 66-fork→**196 EXECUTED**; g63 67-activation→**199 EXECUTED**; d10
   26d→**165/166 EXECUTED** + 26d→246→**247 re-issued** (246=e23's fork
   reservation); g24 107→**146 KEEPS**; e23 **237 KEEPS** (now sole
   claimant); g84 **137 KEEPS**; c4 **143/148 KEEP**, **144 REVOKED→294**
   (e19 earlier), 145-addendum→**265**; g89 **145 REVOKED→295** (e19
   earlier; chain 139→145→295), 122 KEEPS; d38 **125 REVOKED→293** (d125
   earlier); e19 **144/145 KEEP**, 146→**296**, 147→**268**, 148→**297**;
   e20 **149/147/71 KEEP**, 146→**266**, 148→**269**; g50 154→**271**;
   g48 237→239→**272 EXECUTED**, **244/245 key WINS stand**; its executed
   244→281/245→282 renames contested → **310/318** (FROZEN §8); c31
   12k→**300 re-issued**; d81+g53 18b-fork→**301 re-issued**; d83
   18b-fork→**302**; g56 239→**274**; d53 240→244 EXECUTED-CONTESTED→**275
   re-issued §7.4.3**; f21 239→**276**;
   f47 144×2→**277/278**; f45 137→**279**; c42 240→**280**; f26
   7h/7i/7j→**281/282/283**; g15 77→~~284~~ (**284 = g52 KEEPS per FROZEN
   §8.7, executed-pre-§8; g15's 77-doc renumber owed — round 5**; g15's 309
   self-pick→**347**); d22/g4 91×7→**285–291**;
   g8 108→**292**; g63 240→**298 EXECUTED**; f39 244→281→**316 FROZEN-§8**;
   f3 65→**303**;
   d50 243→**304**; f4 240→283→**305 EXECUTED**; g86 244→**306**; g43 246→**307**;
   **FROZEN §8 round-4 wave**: d30/d125/g48/c9/d106/d9/c25/c32/f39/g8 281→**308–317**
   (d30→**338** §8.5; c9's later 312 self-pick→**349** §8.7);
   g48/d125/f39 282→**318/319/320**; g59 283→**321**; d43/g5 244→**322/323**;
   g65 245→**324** (its 308 self-pick→**346** §8.7); g40/d16 246→**325/326**;
   d16/g68/c3/d6 247→**327–330**; d3/c27 248→**331/332** (e26 **248 KEEPS**);
   d119 304→**333**; g62/f30 240→**334/335**; d13/f24 243→**336/337**;
   g17 304→**339**; d21 308→**340**; §8.7 batch e2→**341+342**, d43→**343**,
   c24→**344**, d40→**345**, g15→**347**, d32→**348**, c7→**350**, e5→**351**.
4. 107 and 139 are now EMPTY at the tips (g24/g89 renamed away). 139 stays
   RESERVED for g52c-net-master; 143 is a FULL collision won by c4; 144/145
   are won by e19 (round-3 corrections) — the round-1 "one-sided" note is
   superseded; the brief's f21b doc finally pushed (on 239, loses → 276).
   Unpushed sub-round docs take free numbers **≥360** (free floor 352+;
   round-5 re-scan first).
5. This INDEX's row set above is the base+d54 view; rows for other lanes'
   docs enter with their merges, renumbered per the registry. The merger
   dedupes INDEX rows by (number, filename) using the registry mapping.

## Fog of war (not yet specified)
- Mod-era entity families beyond 1.4.5.8 (Calamity/Fargo scale) вЂ” enters when the
  counts question is ticketed.
- MP transport design details вЂ” blocked on the engine design doc (separate doc).
- tModLoader-style mod API surface вЂ” only if the destination is redrawn to include it.
