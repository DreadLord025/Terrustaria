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
| 69 | Liquid 4-way reconcile (W4-g46 merger assist: the g45/g46/g47/f25 per-fn KEEP table + the graft-prep UNION record + the W4-g45 ADDENDUM; A7 superseded by A8 — g46d LANDED `9bc722e5`; **A9 LIQUID SEAL — the ONE authoritative fold-list for g46e** reconciled vs g47 `ca76d68ee` §15: `2bbadd9bd` hunk verbatim, 10-row fold table, 102→109+2 pin matrix, g15 push-owed probe; **A10 monitor pass 2** — g15 push-rescue LANDED `7981aa5bf` (A9.4 discharged), union `9bc722e5f` still pre-graft) | `69-liquid-reconcile.md` | 🟡 A9 fold-list pending g46e (seal not done until the down-arm hunk is in) |
| 104 | DOC-SWEEP lane 24 liquid census + the bucket port (W4-g45: QuickShimmer phantom / no pressure constant / the flow-rate table; the down-arm live reads + ItemCheck_UseBuckets + the shimmer-flow & fall-arm pins; §5-final owner table; §6c-compose g15 contract; **§7 SEALED — the liquid-family receipt → g46e-bible**; code-only) | `104-doc-sweep-liquid.md` | ✅ SEALED |
| 121 | Shimmer doc-sweep FINAL (W4-f47 lane 36: the splash arms + faithful wet probes ported, the town state-25 seam wired with the b3 contract, the ShimmerSlime 676 row + light/dust arm wired, the transform-table census clean, the g2 Shimmer() cross-note; the e19 keep-one fold — the 16 box + the compile-break fixes; code-only) | `121-doc-sweep-shimmer.md` | ✅ |
| 277 | Item splash KEEP-ONE (W4-f47 × W4-e19 2-way: the arm-by-arm verdict — the f47 Splash latch kept, the e19 16-box fix + box_is_wet + pass-level witnesses folded, the wet-probe 10x8 census aligned, the 676 aiStyle-1 light/dust wiring; RENUMBERED 144 → 277 per registry §6.3 — 144 belongs to e19 §7.1; code-only) | `277-item-splash-keep-one.md` | ✅ |
| 278 | Splash keep-one ARBITER FINAL v1 (RENUMBERED 144-F → 278 per registry §6.3 — 144 belongs to e19 §7.1; SUPERSEDED by 379, formerly 144-F2 — its verbatim-gate tiebreak mooting by the e19 seal's fix; kept as the lane record: the sticky-vs-local lemma, the 145 repin reconciliation, the census-trio folds) | `278-ARBITER-FINAL.md` | ⛔ |
| 379 | Splash keep-one ARBITER FINAL v2 — THE definitive verdict (RENUMBERED 144-F2 → 379 per registry §10 — 144 belongs to e19 §7.1; BOTH prior verdicts superseded; adjudicated at f47 18fe75f2e / e19 0fb0ac4ad with BOTH entry gates verbatim: the keep reduces to witness SUPERSET COVERAGE — KEEPER = f47 Splash/splash_due/ItemDrop::splash, the latch-lifecycle + draw-count twins e19 lacks, the e19 quartet is the budget-broken copy, the e19 gate witness folded as w4_f47k2; the exact e19 drop-hunks re-pinned at 0fb0ac4ad, the e19 seal's 8-row table corrected; code-only) | `379-ARBITER-FINAL-v2.md` | ✅ |
| 151 | Shimmer-family SEAL (W4-f47 seal phase: the five cross-lane compositions — the e19 lights/AddLight mutual-receipt DISJOINT + the lunar-brick RAW-enum fold, the 676 HitEffect census g62-hit/g60-death + the death-half seed, the b1 activation-contract, the d6 ShimmerImmunity-131 compose disjoint, the d43 676-row keep-one; the family bible single-owner map + merge checklist; code-only) | `151-shimmer-family-seal.md` | ✅ |
| 144 | Item liquid-splash keep-one verdict (W4-e19 final + SHIMMER SEAL: the f47 keep-f47 counter-verdict consumed, the e19 keep UPHELD — the entry gate made verbatim (lava_now), the 8-row f47-drop-hunks table published, 2 f47 probe twins folded + 1 gate witness; TR_WATER_STYLE pin adopted) | `144-splash-keep-one.md` | ✅ SEALED (arbiter fallback) |
| 145 | The 16-box cross-lane repin list (W4-e19 final + SHIMMER SEAL: SIZE 14→16 per WorldItem.cs:138; per-lane OWN-BRANCH execution instructions at every lane's current tip — grab edge 42/43, magnet 292/192, floor−SIZE, f49 nebula +92/−8; the g15 line drift corrected) | `145-16box-cross-lane-repins.md` | ✅ execution published |
| 146 | The AETHER biome census (W4-e19 final + SHIMMER SEAL: ZoneShimmer metric/music faithful; bg-skip + star-dim + monolith ramp ported; liquidAlpha[13] dead-code; the g69d-star-visual boundary CONFIRMED disjoint at the four C# windows, g69e-clouds-retire reads) | `146-aether-biome-census.md` | ✅ census + boundary sealed |
| 147 | The quicksell SHIFT-click UI-hint census (W4-e19 final + SHIMMER SEAL: the C# mouse hint IS the cursor-10 icon; the override draw ported; the g86e-cursor-170-sites COMPOSE published — cursorOverride layer 36 vs cursorItemIcon layer 40, keep BOTH, the two reset clocks separate) | `147-quicksell-ui-hint-census.md` | ✅ compose published |
| 148 | W4-e19 research FINAL (SEALED: the full arc + the SHIMMER-seal round — the arbiter consume, the repin execution, the aether boundary, the cursor compose, the shimmer-family bible in ANALYSIS; the witness ledger, the five merge contracts, the standing deviations) | `148-e19-research-final.md` | ✅ SEALED |
| 123 | Vine wire contract (W4-g83 wiring phase: the 4 groups' per-fn attach seams vs the g31-union step map + d102/d106/g42/d109/d110/g48 census; the fused-pass rising-vine wire LANDED; the ROPE-interplay census; **§9 wire phase 2 EXECUTED** — the checks+genRand threading, the grow-tops/jungle-thorn tick arms, the thorn down-merge, the g31 keep-one final hunk-pair + the stream pins; **§10 RETIRE SEALED** — the latest-tip re-verify [g42 dispatcher rewrite + g31 retire receipts], the g42 merge script, the grow_more_vines duplicate retired, the hunk-B re-pin, the rng-rebind census, the vine-family merger bible + the 74-witness acceptance checklist) | `123-vine-wire-contract.md` | ✅ SEALED |
| 83 | Weather reconcile (W4-d130 merger assist: Rain.Update impact-gate truth vs g67, the snowing 3-way KEEP, the splash union) — RETIRED FINAL-FINAL (§8): the g67 splash lane transplanted onto the d130 core in-branch (`88cfbf38`), merger keeps THIS branch for rain+snow; final phase landed the g11 snowing fold (§4: in-body gamePaused + HiDef 4096 budget), the faithful rain/wind AMBIENT cue lifecycle (tracks 28/45, the case-33 frog census), the two-arm cloudAlpha ramp, the one-way sandstorm→rain zone cross-note (§5) and the f39 UpdateTime slime-rain fold (§6); §7 adjudication: g11 F1 CONFIRMED (snowing rides the GLOBAL bare-Main.rand — :17238 precedes every SwapRandom, call-chain proof; the private splitmix retired, `099df89e`) + the g67b-post-union fold (`16a792d5`); §8 RETIRE: research/134 now EXISTS (g67 tip `e777b3e2c`) — the per-item R1–R7 receipt (R1–R4 already folded by 16a792d5, R5 skyblock `10ee539da`, R6 Eyebrella + R7 umbrella `df28ea1ac`), the g67d DrawRain/blizzard censuses composed (rain_light_tile extract + witness, the snow lane confirmed canonical), the f39 co-lane compose `22371b4fe` (music track 48, the anniversary range/halving gates, the menu-return StopSlimeRain, the lowTiles /5 roll arm) — NO weather fold remains open, the d130-canonical declaration FINAL; §9 SEAL: the g67 mutual receipt (146-rename `8b9a1ff46`, BOTH-sides drop-wholesale agreement, the R5/R6/R7/A2 gifts folded), the c39 EXACT 2-hunk graft verified-OPEN-then-FOLDED (the `:64825-64831` storm-deck head clamp + the full `LanternsCanPersist` gate, spec c39 research/114 §13 `8e0946ff`, c39 tip `7dd785dc5`), the f39 season/SlimeSky keep-one receipt (tip `bd5a5a24a`, zero d130 overlap), the WEATHER-FAMILY bible FINAL + gate command | `83-weather-reconcile.md` | ✅ SEALED (§9) |
| 316 | W4-f39 v18-claim INPUT for g71c (the closeout receipt: registry row #8 re-receipted at `da058b714` — the five seasonal downeds byte-identical to c39's claim, keep-one drops f39's hunks, union seat [51..56] per 92 §1-FULL.2 A3; the seven truncation-site families re-pinned at the closeout tip (+5→+107, the six mid-record cuts re-arm); the `.wld` force-today deference DISCHARGED (flags.rs:209-210 live); the four closeout commits add ZERO persist bytes; the g54 EmoteGates five-pin hand-off; code-only. RENUMBERED 244→281→316: first leg registry-150 first-pushed-wins (g48's `244-c28-18-send-switch.md` first-add 14:30:16 beats this doc's 14:32:01); second leg registry §9 UNIFIED FINAL `b243c7086` FROZEN §8.1 — 281 = f26's §7.3 assignment, f39's self-rename contested → 316, chain supersedes the unexecuted 299 row, 299 FREE again) | `316-f39-v18-claim-input.md` | ✅ |
| 320 | W4-f39 SEAL (the closeout receipts + the SLIMERAIN-FAMILY bible: the five closeout items re-verified at tip `8704691a3`; the research/244 collision EXECUTED →281 (g48 first-add 14:30:16 wins, registry-150 §6.3 cutoff, fresh 386-tip sweep), later re-issued →316 by §9 round 4; RENUMBERED 282→320 — registry §9 UNIFIED FINAL `b243c7086` FROZEN §8.2, 282 = f26's §7.3 assignment (68-7i), self-pick `1c120b550` 16:06:54 → 320; the d130 weather-seal `b8f65570` MUTUAL receipt — the five boundary gates composed both-ways, the lowTiles `/5` roll takes d130's ACTIVATED form, d130 zero-edits to npcs.rs/sky.rs/season.rs re-verified (keep-f39); the c39 lantern seal `7dd785dc5` receipt — the real-date gates f39b DONE, c39 has no calendar, f39's resolved-flag reads win the shared constructions, c39's `pub(crate) lanterns_can_start` export wins; the family bible for the merger; code-only) | `320-f39-seal.md` | ✅ |
| 267 | W4-g67 post-union fold-list — FINAL-FINAL/SEALED + RECEIPT-CLOSED (FULL chain 134 → 146 → 247 → 267 FINAL: f11 owns 134 per d54 registry-150; 146 FOUR-way claimed g24d 10:02:44 WINS/e20f 10:22:27/g67 10:32:09/e19 12:08:42 — registry-150b refresh provisionally took 247, lowest unclaimed ≥237; d54 REGISTRY-ROUND-3-FINAL research/150 §7 `51a28d09e` re-issued 247 to d10 and pinned the g67 chain 134 → 146 → **267**): d130 tip b8f655708 SEALED — the mutual receipt CLOSED both sides, every d130f work-list row closed (R5 10ee539da / R6+R7 df28ea1ac / A2 af3ebc1a+s9 / A1 no-op / c39 2-hunk graft ec27d8cfd); FINAL fold-source hash-list f8c4b659c/356a1ac0e/01ba6c7a9/7aaf66f00(A2-only) CONSUMED; g67-drop-wholesale FINAL instruction + WEATHER-FAMILY bible FINAL; code-only | `267-g67-post-union-folds.md` | SEALED — receipt-closed, merger owes nothing from g67 |
| 328 | W4-g68 WIND SEAL (the wind-flag owner's close: the git-log census wind-flag-owner + leaf-cadence, the final-phase three missing arms — remix UpdateWindyDayState / WindyEnoughForKiteDrops FED / InAPlaceWithWind edge-skip; the d130 research/83 §9.4 + g67 research/146 mutual receipts, the c40 wind-only severity + ONE-WAY ZoneSandstorm→ZoneRain receipt + the push keep-one c40-CANONICAL with the G1-G3 graft ledger, the co-lane probe verdict, the WIND-FAMILY bible FINAL; registry 328 FINAL per §9.2 — renumbered from provisional 247, d10 EXECUTED keeps 247) | `328-g68-wind-seal.md` | ✅ SEALED |
| 117 | DOC-SWEEP clouds/sky (lane 33, W4-g69: v18 tail union crimson cross-note, World.crimson consumers census + 5 wirings, numCloudsTemp cadence verify, rain-cloud darkening global-dim fix, falling-star visual census; RETIRE: §6 g69d-star-visual ports + falling-star task-seed, §7 g67f/g61 compose confirmed, §8 12-consumer activation census + GERunner live-flavour fix, §9 clouds-family declaration; code-only) | `117-doc-sweep-clouds.md` | ✅ SEALED |

## Fog of war (not yet specified)
- Mod-era entity families beyond 1.4.5.8 (Calamity/Fargo scale) вЂ” enters when the
  counts question is ticketed.
- MP transport design details вЂ” blocked on the engine design doc (separate doc).
- tModLoader-style mod API surface вЂ” only if the destination is redrawn to include it.
