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
| 63 | Special-bullet effects census & port (S1-P-i18: the 11 aiStyle-1 bullet specials + Luminite; all MISSING at base — no proj_data rows; code-only) | `63-special-bullets-census.md` | ✅ |
| 293 | d38 crossnote ADDENDUM to research/113 — FINAL: the lazy-load alignment (d35 e5558b0a9 §10-12: E35 reproduces the 8115 pre-d38 baseline, the 35+56 NPC-page slice packs 1554 px ONE page), the 35-kind FindFrame status (34 C# cases + the 328 no-case pin, 35/35 port arms, the 7 town stand-ins the extra6/wave-B/extra7 modules supersede), the bestiary unlock census (all 35 banner chains agree, Paladin 25 the one non-default; GAP-1 critter sight-provider pre-existing, GAP-2 rarity stars display-only, GAP-3 the 328 Hide extra card), the 108-extend cross-ref (281-337 vanilla-free CONFIRMED, not 429-461). RETIRE §§11-13: the stand-in drop-hunks at d44 c863a7a5 (5 hook-wired, residual exactly 294/296 wave-B; the 303 correction — keeps the d38 inline arm), the 328 card EXACT 7-part drop-hunk (the panic-path proof + the credit guard, 323→322/489→488 re-pins), GAP-1 CLOSED by c8's generic gate (all six d38 critters auto-covered, 41/41 receipt), the SEAL declaration (35 kinds+35 sheets+35 portraits COMPLETE, the 6-item merger hand-off). SEAL-RECEIPT §§14-18: the {294,296} mapper-hook EXACT 3-hunk task-seed for d44f at ce2c3a24a (no cs_id hand rows needed — the S1-F-3 fallback resolves), the 328 guard exact hunk + recommendation (b) (the wider panic surface: spawn_loot + net_id() + the cs_id sentinel; the lossy try-table asymmetry), the c8 receipt at the PUSHED 7c2de9c06 (the 3 UI sites + plumbing fold table, the e23 static-list compose hazard), the d35 562-union sheets-present receipt at 1afd327b9 (35/35, dims row-for-row, 717824 px), the §13b re-receipt + the b1-b7 sheets-family (75/75 receipt). **RENUMBERED 125→293** (registry round 3, research/150 §7: d125's empress-sheets-ledger 06:07:29 predates this doc 07:00:21 — round-1 win REVOKED, d125 WINS 125) | `293-d38-crossnote-atlas-addendum.md` | 🔒 SEALED |
| 345 | W4-d40 SEAL: band-421-490 census (12 batches, 56 kinds) + the g61d spawn-seal fixes (478 own-ENTRIES-row + 479 drain arm) + the c10e/d37e merge receipts + the band bible (renumbered 285→345 per §8.7 FROZEN table, registry 150 @ d54 `9dddb32b4`; code-only, witnesses written not run) | `345-d40-band-421-490-seal.md` | ✅ SEALED |
| 238 | NPCID 578-630 roster sheets (W4-d42: real-dims inventory 43 kinds / 0.807 Mpx, pack verdict None-skip — full set skyline 8382 > 8192; the 39-name union delta + the d33/c10 keep-one inputs; research/113 §10 lazy-load input; RENAMED from 143 per research/150's 237+ late-push rule — c4's 143 wins; the TRIPLE-238 collision resolved d42-first per registry round-2/3 §6.2/§7.4: c8→262, g61→273, d42 KEEPS 238 CONFIRMED) | `238-d42-roster-sheets.md` | ✅ |
| 239 | W4-d42 FINAL (the 11-kind keep-one drop-hunks for d33 dragonflies + c10 waterfowl — exact line ranges at 9eadc3043, merger-mechanical; the GoldGoldfishWalker-593 fold re-point onto c4's ported 592; the 592/607/615 re-adjudication c4-STANDS + the 405/406 CorruptGoldfish-keep-d34@277/Arapaima-accept-c4@406 verdicts; the 32-kind vanilla-free band declaration; the BloodSquid hover gap-closure cross-ref; the FOUR-way-239 fork resolved d42-first per registry §6.2: g56→274, f21→276, g48→272, d42 KEEPS 239 CONFIRMED) | `239-d42-final-keep-one-and-band.md` | ✅ |
| 113 | Atlas repack plan SEALED (W4 union overflow: 2-page boundary contract + BG-STYLE extension + NPC lazy-load census + step-7 IMPLEMENTATION contract; RETIRE 562-union + cross-receipts d38d/g72e/g31g at 1afd327b re-verified; SEAL §17: NEW claimants d42 NO-PACK-8382-forecast-29 / d43 +63-band / f29 109-equal-both-ways / e23 +9-MapBGOverlay / g84 unmoved / c8-c9 in-union / g10 +11-pets / g11 zero — the **707-name census union** (+145 since 562, 0 gone), lazy ONE page **7256/7423/+Glow362 7447**, eager worst 8191/2377, per-world NPC own pages ≤3286 px post-merge, realistic 3-page ~537-595 MB / VRAM ~932-990.5 MB (adversarial 4-page ~650-688 / ~1046-1083) incl. the 268.4 MB font array + 127.1 MB targets, min-spec 2 GB STANDS; §17.5 the FINAL atlas-family bible — the 7-step lazy-load merger contract, no further census owed; code-only) | `113-atlas-repack-plan.md` | ✅ SEALED |
| 329 | W4-c3 RESEARCH FINAL: the cursed-skull-family bible SEALED (renumbered 247→329 per registry §9 UNIFIED FINAL, d54 tip `b243c7086` — d10's executed 247 reservation wins; style-10 34/289/694 machine + drains + 299 bolt + GetAlpha tints + dungeon spawn band + the 299 census; the 355-collision E8 receipt — keep d34@267/d38@289, c3 keeps 694 with the d43 research/244 mutual receipt; tip-verified merge hazards: d34/d38 discard Style10Out + no 10 axis arm + d38 no 299 row; 24 witnesses WRITTEN NOT RUN; code-only) | `329-cursed-skull-family-bible.md` | ✅ SEALED |

## Wave 4 - fish lane contracts (W4-c4, retire phase)
| # | Module | File | Status |
| --- | --- | --- | --- |
| 143 | d40 CrimsonGoldfish-465 retire hunk (exact drop-lines for merger/d40b; keep-c4 kb-1.0 + vanilla_type fix) | `143-d40-crimsongoldfish-retire-hunk.md` | MERGE CONTRACT |
| 144 | Orca NPC_692 census-input for d35e (dims-row + corrected FINAL rank #16 + the lazy-NPC-page admission form) | `144-orca-npc692-census-input.md` | CENSUS INPUT |
| 145 | Claims-table addendum for d37d-E3: the five keep-d42 verdicts SUPERSEDED (d42 `49346dba6` dropped the kinds, ceded to c4) + the CorruptGoldfish d34@277 dup flag + the CrimsonGoldfish implementation-conflict note | `145-d42-keep-d42-verdicts-addendum.md` | MERGE-CONTRACT ADDENDUM |
| 148 | FISH-FAMILY declaration: style-44 (4 kinds) + style-16 (13 kinds) ALL LANDED - the lane-close merger contract per 108-E7 + research/143/144/145; the d43-band 688/692 conditional retire note | `148-fish-family-declaration.md` | LANE CLOSE |

Number-collision verdict (2026-11-21 retire phase, first-pushed-wins):
143/144 KEPT - no rename. Evidence: all 357 pushed refs swept
(`git ls-tree <ref> docs/research/`); `origin/task/W4-g87-demon-heart-use`
tip `dfbcaf94b` carries NO research/143 file and NO 143/144 INDEX row;
`origin/task/W4-f47-liquid-shimmer-2` tip `a2561d6d9` carries only 121
(its own INDEX row) and NO 144 claim; no `g87b`/`f47c` refs exist on
origin (`git ls-remote`). c4's 143 (`90fd242d9`) / 144 (`479d70ad2`) are
the only pushed claims. The lead's fallback numbers 146/147 stay
UNASSIGNED (reserved); this lane's new retire-phase docs take 145/148.
| 262 | W4-c8 crawler FINAL census + RETIRE declaration + SEAL (renumber-chain 143→238→262 per registry-150 §6.3 round 2 — 143 belongs to c4; the interim 238 lost to d42's earlier `9eadc3043` 12:15:55 rename, c8 assigned 262; the generic CountsAsACritter provider contract — c7/c9/c10/e26/c4 auto-covered at their tips (§7a receipt, the c9 row-cite corrected :3720/:4090 at seal), Goldfish freeze lifted + LIVE (§1a — 55 is base), 549 Hide excluded; the 14-member gold set + 13-row peer table SET-COMPLETE; the d22 b24977d9 worm-head compose — 375 flee at the C# position, accel not snap, the 110-tick pin CONSUMED by d22's seal `3ec231f98` (§8 item 2 — keep-one now); the 9 sheets in the d35 FINAL union + lazy-load census; CRAWLER-FAMILY COMPLETE §7 bible FINAL; the four seal items receipted §8 — retire 7c2de9c06 landed, the 238 triple collision resolved d42-238/c8-262/g61-273; code-only) | `262-c8-crawler-final-census.md` | ✅ SEALED |
| 63 | Special-bullet effects census & port (S1-P-i18: the 11 aiStyle-1 bullet specials + Luminite; all MISSING at base - no proj_data rows; code-only) | `63-special-bullets-census.md` | ✅ |
| 109 | The snail band claims (W4-c9: protocol ids, the 354-399 corridor collision matrix, the catch/lights/Transform residuals) | `109-snail-band-claims.md` | 🟡 claims register (109 = c9's registry win) |
| 349 | The snail-family FINAL bible + the W4-c9 seal census (the 279-roster bijection, the d37 ALL_KINDS hunk EXECUTED-VERIFIED superseded, the c8 sight-gate auto-cover, the f26-7j 655 delegation, the merger contract + the five-gold-critter c10 drop-hunks) — rename chain 281→312→349 per registry-150 §7 round-3 then §8.7 AMENDMENT-3 BATCH FROZEN (`9dddb32b4`: d106 keeps 312, c9's 312 self-pick → 349; NO-SELF-PICKS enforced) | `349-snail-family-final.md` | ✅ FINAL |
| 270 | W4-c10 waterfowl band FINAL (aiStyle-68 + walkers + 1080 drop: the one-paragraph bible, band-complete ledger, the b3/d20/d76/d36/g8/d40 cross-lane contracts incl the roster-tail + d76 keep-ones, 37-witness index, merge checklist; code-only; renumber chain 145→149→270 — g89's rod doc pushed first at 145, e20's devices bible pushed first at 149, see the doc's renumber notes + d54's 150 registry §6.3) | `270-waterfowl-band-final.md` | ✅ SEALED |
| 263 | W4-g87 closeout fold-input + RETIRE (RENUMBERED 143 → 263 per research/150 §6.3 — c4 keeps 143): §1-§5 the closeout record (the a6 dup adjudication, a6's 4 fidelity deltas folded, the 5289 flag-gate hypothesis REFUTED — both ALWAYS-spawn; the slot-8 UI/draw census + port); §6 the g87c retire receipts SUPERSEDE the fold tables — the dh line g88h DEFAULT-DROP (138 §10.4 / 132 §2f, DD2 keeps v12), the pets seat [57..60] in the SEALED 107-byte v18 union (103 L3-ARBITER A3, the 24/68-byte tables retired), the use-arm survivor seats on a6's machine (138 §12.3), the pet-license Success leg activation-contracted (g10 DONE → f43b splice → the one-line dispatch, the g87-side hunk in code comment), the FINAL demonheart-family bible | `263-g87-fold-input.md` | ✅ SEALED |
| 91-D22 | The 123 unrowed C# aiStyle-3 types — the d34-d43 roster-gap table (W4-d22 retire R2, the d37d-E6 input): per-type SetDefaults cite + corridor lane + pushed-tip claim status (75 ROWED / 48 open) + the AI_003 extras each faithful port must carry; the fighter_walker_kind extension list; the 590/591 + 368 census corrections | `91-d22-roster-gap-table.md` | ✅ |
| 91-G4 | G4 DEPRECATED — the fighter-family canonical record (d22) + the final KEEP/DROP symbol table (W4-d22 retire R3): every g4 commit f384797c4..a9369ec38 adjudicated with d22 landing sites; the P6 generic-walker flips verified landed; the merger instruction (merge d22, never g4-after) | `91-g4-deprecated-keep-drop.md` | ✅ |
| 91-FR | The d22r keep-both fold-list FINAL RECEIPT (W4-d22 retire R4): the g4 addendum §7 checklist all 9 boxes CLOSED with evidence, the retire deltas (R1 moan-position closure / R2 roster table / R3 deprecation), the fighter-family merger instruction, the branch hash receipt | `91-addendum-final-receipt.md` | ✅ |
| 91-FWK | The fighter_walker_kind extension obligation EXACT HUNK (W4-d22 seal S2, the d37e-E8-seal input): the append point + ai[3]-free precondition + per-lane variant names probed at pushed tips + the witness obligation; the 294-296 seal correction to the R2 §2.1 list | `91-fighter-walker-kind-obligation.md` | ✅ |
| 91-BIB | The FIGHTER-FAMILY BIBLE (W4-d22 seal S4, SEALED — FINAL, the FIGHTER-CORRECTION consumed): d22 canonical styles 3-6 (the landing table), **f35 owns the DropHeals heal seam** (ported FIRST superset b72cdfbfc/00a5b27db, g4's duplicate retired by the g4 seal e5915bfab — the S3 a9369ec38 no-delta probe was STALE), merge d22+f35 NEVER g4-wholesale + cherry g4's 2 unique hunks (spawn_coins midas/blood-moon LIVE inputs d22:7218+f35:6954 still false,false + the Thief/Skeleton scan witnesses), DROP g4's scan type-bonus (d22:498-513 identical) + pick-path Peon smash (KEEP d22 kill_tile), **the 2 cherry hunks LANDED IN-BRANCH by the g4d fold** (spawn_coins LIVE inputs + witness, the Thief/Skeleton scan pair — verify by symbol, do NOT re-cherry), g8/g9/g10 compose disjoint, d34-d43 take the R2 table + the S2 obligation + the 520 MartianWalker shooter = d41 task seed (NOT d43); the c8 110-tick pin consumed (S1) | `91-fighter-family-bible.md` | ✅ |
| 91-FC | The FIGHTER-CORRECTION receipt (W4-d22, the g4 seal e5915bfab consumed): the stale-probe timeline, the keep-one INVERTED (f35-owns-DropHeals, the census :39 disclaimer a BEFORE-this-task snapshot), the corrected merger instruction + the 2 cherry hunks with d22/f35 re-verified stand-ins, the Peon kill_tile keep, the 520 d41 task seed with the C# params re-read; the g4-side record `91-seal-keep-one.md` lives on origin/task/W4-g4-npc-style3-fighter | `91-fighter-family-bible.md` §0/§2/§6 + ANALYSIS 'The SEAL-receipt CORRECTION' | ✅ |
| 322 | Caster-family declaration (W4-g5 lane close): aiStyle-8 ALL CLAIMED (13 kinds), the 693 g5-vs-d43 keep-one + the 665 R4 fold FINAL, the RuneWizard 354→827 E8 move, the d33-102/103 routing note; §9 seal verification (the 244→322 registry-150 §8.1 row-15 renumber — g48 wins 244; the strike-restart + spawn-roll census addendum) | `322-caster-family-declaration.md` | ✅ |
| 108 | W4-g8 proto-band claims table (354-387) for the d37c/d37d matrix (the 34 <=230 d22-d33 machine kinds + the Vampire 159 census; code-only; E8 EXECUTED — the live ids are 745-761 + Vampire 762, see 317; the 108 number collides with d37's roster-master 108, merger keeps d37's) | `108-input-g8.md` | ✅ |
| 317 | W4-g8 kind-wire FINAL + SEALED (chain 281→317, registry §9.2 row 317) — the E8 execution receipt (the 17 owned kinds 745-761, the 17 drop-pending dups keep-one d34/d35/d37 incl Harpy d34@275 per g44e SS6, the Vampire 388→762 collision fix (788-790 stay reserved for d39), ALL_KINDS 270→305 + the s1_f_3/s1_f_13d stale pins re-derived), the git-log census (4 batches + 4 machine rounds), the g59-activations + 158-flip-callers CLOSED receipts, the feather-shot ACTIVE supersession of g44e's NOT-active sweep, the kind-wire-family bible; code-only, witnesses written not run | `317-g8-kind-wire-final.md` | ✅ FINAL |
| 365 | W4-e21 wiring-tail family SEAL (the five receipts at the pushed tips: the HitWireSingle census 66/66 ZERO-unclaimed + the bunny-statue pair 46/540 ported; the c42 wiring seal 2910a720 re-pick-compile-blocker-FIXED + the e20/c42/e21 ownership split; the bunny keep-one vs d34@270 (d37-E8 KEEP CONSUMED) + the 229/237 collision-free proof (f18's 237 drops per d39 §6c); the boulder aiStyle-25 rows d3-owned PUSHED + the two-arm keep-one with the seam's latent 15.5px anchor defect; the research-149 devices bible consumed; code-only, witnesses WRITTEN NOT RUN; chain 285→365 per registry §9.2 — 285 = §7.2 owner KEEPS) | `365-e21-wiring-tail-seal.md` | ✅ SEALED |
| 313 | W4-d9 FINAL: the projectile aiStyles 81-90 band table (the five gap-styles 81/86/87/89/90 ported + wired, the five even styles main's census; the e3 dust-activation contracts + the g78 spelunker-canonical verdict; registry-number 313 per §8.1 row 6, chain 281→313, code-only) | `313-projectile-ai-81-90-final.md` | ✅ SEALED |
| 357 | W4-e26 luck-system FINAL census + RETIRE declaration (number 357 per registry-150 §9 UNIFIED FINAL — chain 248→357: g82's §6.3 reservation wins key 248, the round-4 draft's 333 collided with d119, §9.2 row 357 is FINAL; drafted at first-free 248, 237-247 claimed at tips; the c8 generic SIGHT-gate auto-cover receipt — Frog/LadyBug/GoldLadyBug zero provider wiring, 605 pre-tabled in GOLD_PEER_TABLE; the ladybug-rain-boost g67-drop/d130-canonical merge note; the factor-activation table — f33 torch-god ports the scan head/tail NOT the galaxy-pearl/mirror/kite neutral factors, gnome +0.2 already live; the Frog361/Duck362 DEDUPE claims-table — d37-E8 keep-one EXECUTED 04dbdb0d2: Duck→c10, Frog wire 361→744 cs/net 361, ladybug pair stays e26 via the d42 flip; bijection 273/273, ALL_KINDS 273, sheets 184/184, ENTRIES 252; LUCK-FAMILY COMPLETE §4; code-only) | `357-e26-luck-final-census.md` | ✅ |

## Wave 4 lane FINALs (registry-150 §8.7 new-doc rule, ≥360)
| # | Module | File | Status |
| --- | --- | --- | --- |
| 360 | W4-g10 lane 3 pets FINAL — the SEAL-VERIFY receipt (g10b phases landed, ALL-11 wired census, the f43b splice contract receipt, the d43 keep-one) + the pets-family bible | `360-g10-pets-final.md` | ✅ SEALED (code-only; 26 witnesses written not run) |
| 121 | Shimmer doc-sweep FINAL (W4-f47 lane 36: the splash arms + faithful wet probes ported, the town state-25 seam wired with the b3 contract, the ShimmerSlime 676 row + light/dust arm wired, the transform-table census clean, the g2 Shimmer() cross-note; the e19 keep-one fold — the 16 box + the compile-break fixes; code-only) | `121-doc-sweep-shimmer.md` | ✅ |
| 277 | Item splash KEEP-ONE (W4-f47 × W4-e19 2-way: the arm-by-arm verdict — the f47 Splash latch kept, the e19 16-box fix + box_is_wet + pass-level witnesses folded, the wet-probe 10x8 census aligned, the 676 aiStyle-1 light/dust wiring; RENUMBERED 144 → 277 per registry §6.3 — 144 belongs to e19 §7.1; code-only) | `277-item-splash-keep-one.md` | ✅ |
| 278 | Splash keep-one ARBITER FINAL v1 (RENUMBERED 144-F → 278 per registry §6.3 — 144 belongs to e19 §7.1; SUPERSEDED by 379, formerly 144-F2 — its verbatim-gate tiebreak mooting by the e19 seal's fix; kept as the lane record: the sticky-vs-local lemma, the 145 repin reconciliation, the census-trio folds) | `278-ARBITER-FINAL.md` | ⛔ |
| 379 | Splash keep-one ARBITER FINAL v2 — THE definitive verdict (RENUMBERED 144-F2 → 379 per registry §10 — 144 belongs to e19 §7.1; BOTH prior verdicts superseded; adjudicated at f47 18fe75f2e / e19 0fb0ac4ad with BOTH entry gates verbatim: the keep reduces to witness SUPERSET COVERAGE — KEEPER = f47 Splash/splash_due/ItemDrop::splash, the latch-lifecycle + draw-count twins e19 lacks, the e19 quartet is the budget-broken copy, the e19 gate witness folded as w4_f47k2; the exact e19 drop-hunks re-pinned at 0fb0ac4ad, the e19 seal's 8-row table corrected; code-only) | `379-ARBITER-FINAL-v2.md` | ✅ |
| 151 | Shimmer-family SEAL (W4-f47 seal phase: the five cross-lane compositions — the e19 lights/AddLight mutual-receipt DISJOINT + the lunar-brick RAW-enum fold, the 676 HitEffect census g62-hit/g60-death + the death-half seed, the b1 activation-contract, the d6 ShimmerImmunity-131 compose disjoint, the d43 676-row keep-one; the family bible single-owner map + merge checklist; code-only) | `151-shimmer-family-seal.md` | ✅ |

## Fog of war (not yet specified)
- Mod-era entity families beyond 1.4.5.8 (Calamity/Fargo scale) вЂ” enters when the
  counts question is ticketed.
- MP transport design details вЂ” blocked on the engine design doc (separate doc).
- tModLoader-style mod API surface вЂ” only if the destination is redrawn to include it.
