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
| 284 | g52c net final-audit: the relay-host arms census (SEND side; the g48 FINAL-table feed, the 34 destroy keep-one, the 34/162 codec census; 139 stays RESERVED for g52c-net-master) | `284-g52c-relay-host-arms-census.md` | ✅ code-only |
| 276 | Net APPLY audit (W4-f21b: the g48-g52 tip coverage census, the remaining-gap list 39/41/43/48/52/79/58-client/TE/144/150, the union keep-one notes; 57/76/58-host small arms WIRED; code-only; renumbered 239 → 276 per research/150 round-2 §6.3 — 239 belongs to d42's earlier push) | `276-f21b-net-apply-audit.md` | ✅ |
| 332 | W4-c27 SEAL: the 18 SetTime lane FINAL (reader/codec/apply landed 6fb7bcb25/412b6bc0d) + the clock-family bible (one owner per body; 205→id-7 send switch = c28 PENDING per g48 244; sunModY/moonModY parked with the full title-screen-drag scope pin; code-only, 3 witnesses WRITTEN NOT RUN) | `332-c27-set-time-final.md` | ✅ SEALED |
| 237 | Bestiary UI FINAL close (W4-e23 final phase: the overlays + biome-flavor fallback + Misc rows, the NOTES writer compose contract, the c8 sight-unlock gold-gate fold, the d42 sorting-table coverage; code-only) | `237-bestiary-ui-final.md` | ✅ FINAL |
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
| 244 | PROJ-DUST RETIRE, SEALED (W4-d53: the wetVelocity 96-site movement-gap census + seed S1-P-i57/R-244-1, the g2 staged-copy drop-hunk + mutual receipt, the d53×e3 DISJOINT merge-note at moved tips, the insurance-patch SHA256 pins DELETE-AT-CLEANUP, the family bible FINAL; renumbered 240→244 — g58 won 240 first-pushed, 241-243 g58's, code-only) | `244-proj-dust-retire.md` | ✅ SEALED |
| 187 | Main.cs dust-site batch 3 (W4-g13): census completion + witnesses (renumbered 65→187 per registry-150) | `187-dust-main-sites-3.md` | ✅ |
| 202 | Main dust residual sweep (W4-g13): sibling re-probe, ZERO unclaimed spawn windows, the Dust.cs internals audit (3 d50 wiring flags), both lavaBubbles readers (renumbered 68→202 per registry-150) | `202-dust-main-residual-sweep.md` | ✅ |
| 206 | The 52 second-chain dust lights (W4-g13): census + collector + wiring + the d50 strip-contract (s4) + decrement retire (s5) + the FINAL receipt (s6) (renumbered 69→206 per registry-150) | `206-dust-second-chain-lights.md` | ✅ FINAL |
| 210 | Dust DRAW-tint census FINAL + the GetAlpha 157/327 drop closures (W4-g13; d54 s7.2 G3/G4): the dead-raise quirk, the draw-pass RNG placement contract (renumbered 70→210 per registry-150) | `210-dust-draw-tint-final.md` | ✅ |
| 308 | DUST-MAIN-FAMILY BIBLE (W4-g13 SEAL): the git-log census, the d54×g13 dust_table keep-one (both-ported mid-flight, g13 retire blob wins), the 94/94 AddLight census, the d50 resume-EXECUTED receipt, the merger contract set | `308-dust-main-family-bible.md` | ✅ SEAL |
| 66 | Projectile per-kind TRAIL dust claims + census (W4-g63, kinds ≤400 — batches 1-7, the row census + the corrected 60-style gun row per the e3 finding; registry-150 number winner) | `66-proj-trail-dust-claims.md` | 🟡 UNWIRED emitters await the merges |
| 199 | Projectile TRAIL dust activation checklist (W4-g63, for the merger — which merge unlocks which emitter + the RNG-seam reconciliations; renumbered 67→199 per registry-150; refreshed at retire: the 128/129/229 promotions + the 357 f20 seam at tip 31386266; spawn-dust seal: the gun Kill keep-one adjudicated d6-survives + the e3 exact fix-hunk pointer) | `199-proj-trail-dust-activation.md` | 🟡 merge-wave input |
| 298 | W4-g63 projectile TRAIL dust FINAL + the spawn-dust-family declaration (SEALED — the family boundaries, the e3 census-error consumption, the 12/955 anniversary EXACT two-draw fix-hunk (§3.2), the gun Kill keep-one ADJUDICATED d6-survives/g65-drops with EXACT drop-hunks (§3.3), the 128/129/229 rows re-probed current, the residual debt; renumbered 240→298 per registry-150 ROUND-3-FINAL §7.3 — g58 wins 240, the unpushed round-2 281 draft superseded (281 = f26's 7h); code-only, 50 witnesses WRITTEN NOT RUN) | `298-proj-trail-dust-family-final.md` | ✅ FINAL + SEALED |
| 243 | W4-d50 dust-type-table RETIRE FINAL (number per registry-150: 237-242 taken at the origin tips, 243 first free; the wiring census pre_move-strip/type_chain-wire CONFIRMED-LANDED, the d54×g13 G3/G4 keep-one exact drop-hunks, flag-b/flag-c receipts + the lights_b replay-retire keys, the dust-type-table family declaration; code-only, witnesses written not run) | `243-d50-dust-type-table-final.md` | ✅ |
| 68 | Mood reconcile (W4-f37 ProcessMood core × W4-g58 happiness UI: arm table, KEEP/DROP lists, capture-once integration contract, g58b deviation census; merger assist, code-only) | `68-mood-reconcile.md` | ✅ (EXECUTED at batch-L: f37 core + g58 id-tables/report/UI, the §3 drops + §4 one-capture seam) |
| 73 | Doc-sweep town lane 2 (W4-f37: ANALYSIS + research/23/11/22/29 + shop/town docs mined for unported town mechanics; CollectTaxes anniversary pair + head-index payer filter, nurse heal presentation tail, tax payout dialog tail, Stylist hair-window state machine — each with C# cite + lane verdict; code-only) | `73-doc-sweep-town.md` | ✅ |
| 64 | Town-spawn census (W4-f43: UpdateTime_SpawnTownNPCs tail arms - pet licenses 637/638/656, town slimes 670/678-684, Santa 142, Princess flag9; the 701-707 directive ids corrected to the real NPCID set; code-only, unwired) | `64-town-spawn-census.md` | ✅ (SPLICED by 368) |
| 368 | Pet-arrival family FINAL + the f43b SPLICE (W4-f43, a6f Success-leg contract leg 2+3; §9 rename: the post-§9 352 self-pick was contested — 352 = d1's EXECUTED `194c66ce6` 352-d1-falling-block-final — row 368 per Lead amendment): the arrival gates wire to the g10 kinds (LADDER 21->32, condition_holds pet/slime arms + full Princess flag9), spawn_town_tail_unique_data (GiveTownUniqueDataToNPCsThatNeedIt name/roll/name on the UpdateTime genRand-alias), exchange_resident_pet (the live-Npc RerollVariationForNPCType adapter) + the a6 else-arm one-line Success/Fail dispatch on UpdatePlayers; the flags.rs:608 load-gap fix (6 slime World fields + .wld read/write + .trwld v18 11-byte tail + the per-record variation byte); d43-town-pets DUP keep-one g10-canonical; the genRand=Main.rand alias census correction; the pet-arrival-family declaration; code-only, witnesses WRITTEN NOT RUN | `368-pet-arrival-family-final.md` | ✅ FINAL |
| 138 | POWERUP master reconcile SEALED + ARBITER receipt (a6 ⨉ g87/g88/g89/g90 ⨉ c36/f33 ⨉ f44/f45: per-item keep-one adjudication incl. the pushed g87 dfbcaf94b arm-by-arm §10 + the closeout 26620538e cross-receipt §11.3, persist-layer split, .wld byte-compare, the .trwld v18 union re-aligned §11.2's 101 → the LANDED 107-byte head + W×H dense §12.1 — **g88 d09d7e10a research/103 §L3-ARBITER FINAL A3 authority** (THE ONE table, D3 restated verbatim — A2 OVERRULES 92's books-in-prefix order under the C#-definitive brief, the fc5cf11a SEAL's D5 CONFIRMED; the seal's g71d pick corrected by the 9a4acd729 D5-RE-PIN), a6's seats pets 51-53 → 57-59 / books 55-56 → 61-62 STAND, the 103↔138 ARBITER mutual receipt §12.4 (A4's 138-row closed) + the SEALED-receipt §12.5, .trplayer v11 receipt — the g88 six-line v7 rule + torch line LANDED §11.1, a6's five fidelity-fix receipts §10.7/§11.4 + the exchange-fail fold, the Success-leg ACTIVATION CONTRACT §12.2 (**g10 5dee28650 kinds DONE** → f43-splice (f43b) → a6's one-line dispatch, the exact hunk published), the powerup-family bible §12.3, witness union, merge order; code-only) | `138-powerup-master-reconcile.md` | ✅ SEALED + ARBITER |

## Wave 4 lane FINALs (registry-150 §8.7 new-doc rule, ≥360)
| 85 | DOC-SWEEP lane 12 mana/magic (W4-f49: the nebula/4143 consume arms + NebulaLevelup PORTED; drink-while-sick / magic-cuffs / buff-6 ladder censused + witnessed; crystals skipped — g88b; code-only) + MANA RESIDUALS R1-R5 (the blaster CheckMana gap PORTED, the R3 one-writer contract) + FINAL-STATUS (R3 EXECUTED: the scan arms deleted, the fold_orb bridge stub + the update_survival lift; the five CheckMana sites' d-lane census — d10/d19/d21 landed 3 of 5; the prism flower end-to-end pass-tail witnessed; the f12d/f49-contract receipt) + RETIRE SEALED (RT1 the f12 mutual bridge receipt — their R3-NOT-landed monitor STALE, 8eb20d8b landed — + the merger-remainder-list FINAL; RT2 the d79 three-way per-fn keep-one table, the THIRD nebula copy ruled; RT3 the MANA-FAMILY merger-bible declaration) | `85-doc-sweep-mana.md` | ✅ SEALED |
| 360 | W4-g10 lane 3 pets FINAL — the SEAL-VERIFY receipt (g10b phases landed, ALL-11 wired census, the f43b splice contract receipt, the d43 keep-one) + the pets-family bible | `360-g10-pets-final.md` | ✅ SEALED (code-only; 26 witnesses written not run) |
| 263 | W4-g87 closeout fold-input + RETIRE (RENUMBERED 143 → 263 per research/150 §6.3 — c4 keeps 143): §1-§5 the closeout record (the a6 dup adjudication, a6's 4 fidelity deltas folded, the 5289 flag-gate hypothesis REFUTED — both ALWAYS-spawn; the slot-8 UI/draw census + port); §6 the g87c retire receipts SUPERSEDE the fold tables — the dh line g88h DEFAULT-DROP (138 §10.4 / 132 §2f, DD2 keeps v12), the pets seat [57..60] in the SEALED 107-byte v18 union (103 L3-ARBITER A3, the 24/68-byte tables retired), the use-arm survivor seats on a6's machine (138 §12.3), the pet-license Success leg activation-contracted (g10 DONE → f43b splice → the one-line dispatch, the g87-side hunk in code comment), the FINAL demonheart-family bible | `263-g87-fold-input.md` | ✅ SEALED (the use-arm keep-one + the Success leg EXECUTED at batch-L; the dh DROP deferred — g88/f44 not in this batch) |

| W4-d79-final | PITCH FINAL + **SEALED**: the 3-way Nebula keep-one owner-table (d79/f49/f12) + the f49 mutual receipt (§S1 — AGREED vs RT2, four identical-twin survivorships resolved to f49, the bridge-ASSIGN-with-deletions one-commit contract), the LegacySoundPlayer ALL-CASE census (0-42 **43/43 PORTED** — case-39 Drip landed at the seal `b2c9aa1d2`, the GoreSound seam), the d80 compose-note (re-censused at `a6bcf736a`, zero bare overlap holds), the per-lane rewire-table at the CURRENT tips (§S2 — g21 `7596fb262` +5, e10 `8e0d951fc` +0-bare, the ledger **48 targets + 3 deletes**), the six E21 task-seeds with owner-lanes (§S4), the pitch-family bible (§S5); code-only | `W4-d79-pitch-final.md` | ✅ SEALED (the d79-side deletions + the cue rewire EXECUTED at batch-L; the f12 bridge half stays its lane's) |
| 240 | Happiness final round (W4-g58: the buy/sell latch repoint + the report-UI census COMPLETE + the savings 3-way call-site census 142-input + the portrait-hop state port; code-only; renumbered 143→240 per registry-150, 143 stays with c4) | `240-happiness-final.md` | ✅ |
| 241 | f37 latch-repoint hunks (W4-g58 retire: the five exact `shop_mood` → `shopping_settings.price_adjustment` hunks against f37 tip `908482d52` for the f37 lane/merger; prerequisite + apply-order + not-repointed census; code-only) | `241-f37-latch-repoint-hunks.md` | ✅ |
| 242 | Chat-portrait system task-seed → f22 (the CloseUp/FullBodyRetro dummy-NPC contract + the unported provider rows) + the happiness-family SEALED bible (g58-latch canonical / f37-price-math / d122-shops, one paragraph; code-only) | `242-chat-portrait-seed-happiness-sealed.md` | ✅ |
| 243 | Seal verification round (W4-g58: the 4 retire commits PUSHED `d0f6275a5`; fresh 358-tip 237–249 scan — 240 SOLE claimant, NO collision, the sibling 237/238 assignments SUPERSEDED (c8/d42/g61 TRIPLE-238, e23/g48 237, d120 still-143); no registry-150b at any tip; f37 hunks re-verified line-exact at still-parked `908482d52`; f22 retired `8c1b40a46` BEFORE the seed reached origin — the dummy-mode remainder UNCLAIMED for a successor; code-only) | `243-seal-verification.md` | ✅ |

## Fog of war (not yet specified)
- Mod-era entity families beyond 1.4.5.8 (Calamity/Fargo scale) вЂ” enters when the
  counts question is ticketed.
- MP transport design details вЂ” blocked on the engine design doc (separate doc).
- tModLoader-style mod API surface вЂ” only if the destination is redrawn to include it.
