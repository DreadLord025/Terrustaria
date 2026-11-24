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
| 181 | `RandomizeCaveBackgrounds` worldgen RNG-shift census (W4-g72 lane 3: the exact draw list/order/rejection loops/width-arm bounds + the moon-ordering constraint; phases 7-8 WIRED it — `worldgen/cave_bg.rs` + the `.wld`/`.trwld` v18/net persistence; §2 boot-roll caveat CORRECTED: fires once per process before any load, the load overwrites it; §6 the :11340 ORDER-CONTRACT for g71c — RandomizeBackgrounds inserts BETWEEN the cave block and the moon draw; §7 the keep-one RECEIPT (retire): g71 fc76a48b3 adopted OUR cave_bg.rs VERBATIM + dropped ITS bg_rolls cave copy — ours canonical, no banners on our side; the underworld seat FIRED (g71 rolls + .wld-persists it); code-only) | `181-cave-bg-rng-shift-census.md` | 🔒 SEALED (retire) |
| 182 | The v18 UNION addendum (W4-g72 final: OUR exact 31-byte claim — 3×i32 caveBackX + 4×i32 caveBackStyle + ice/jungle/hell u8 — seated BETWEEN g70's treeX half and g71's surface bytes in the C# .wld field order; the five-claimant 53-byte union layout + the 31→53 truncation-constant moves, input to g71c extending research/92 §1; SUPERSEDED by 64b → the FINAL 107-byte union; D5-RE-PIN: the [0..50) prefix rows stand verbatim, the draft's books tail [51..53] moves to [61..63] per g88 103 §L3-DEFINITIVE D3; §3's underworld flag CLOSED — seat FIRED at [56], was [58]) | `182-v18-union-addendum.md` | 🔒 SEALED (superseded by 64b; authority g88 103 D3/D5) |
| 64b | The CONSOLIDATED v18 claimant registry (W4-g72 closeout, RETIRE-UPDATED to the **107-byte head**, D5-RE-PINNED: twelve rows / eleven distinct blocks — g70 16 + g72 31 + g71 surface 3 + g71 underworld 1 (the FIRED seat, §0's receipt: rolled bg_rolls.rs:168/:311 + .wld flags.rs:186/:577 at g71 tip f2d8c2c67) + g69 1 (e19 dup drops) + g90 2 + seasonal 5 (c39≡f39 dedup) + g87 3 + d125 1 + c43/f45 identity 44 = 107 + d109 W×H dense LAST; **layout AUTHORITY = g88 research/103 §L3-DEFINITIVE D3/D5 @ fc5cf11ae** — the flag middle in STRICT C# row order: crimson [50], moon [51..56], underworld [56], pets [57..60], empress [60], books [61..63], identity [63..107]; the retire-era 92 §1-FULL.2 pin (books [51..53], underworld [58]) RETIRED — 64a-stable-prefix is not C# write order, g90g's independent direct check confirms; OUR cave prefix [0..50) unaffected; f44 132 §3-L3 agrees on bytes (its underworld-before-crimson inversion fixed by D3); the truncation moves → 107; the keep-one ADJUDICATED (our cave_bg canonical); §5 the g71f mutual receipt — 92 must re-pin books to D5; the historical 97→98→106→107 corrections kept as the audit trail) | `64b-v18-claimant-registry.md` | 🔒 SEALED (D5-authority: g88 103 §L3-DEFINITIVE D3) |
| 64c | The surface-bg draw ownership (W4-g72 closeout verification: the five v16-owned variants corrupt/jungle/hallow/crimson/mushroom READ the g71-shape setBG style→ids expansion via world.*_bg() → surface_bg_texture_ids slots 1/2/4/5/8 — LIVE, not style-0 pins; the forest/desert/snow/ocean chains are style-0 PINNED here, their style tables SetForestBGSet 0/10/11/12 = g70 + setBG 3/6/7 = g71 are those lanes' to port, the pins lift to g70's forest_bg(n)/g71's snow_bg()/desert_bg_set()[0]/ocean_bg() at the merge; the C# dispatch table Main.cs:59529-59597 + the setBG slot map WorldGen.cs:7215-7768; the underground-desert cavern style stays ours) | `64c-surface-bg-ownership.md` | 🔒 SEALED (retire) |
| 64d | The d69 capture-compose DELTA (W4-g72 closeout: the seven bg_target.rs seams STILL MATCH re-verified at d69 tip 79f914fdb — LayerCamera/offscreen_range/layer_composite_position 6-arg/sky_target_active 6-bool/SceneLayer/WorldSceneLayerTarget position-size/BgTex-push_screen; the draw_background CALL SIGNATURE IS BROKEN 10→14 args — d69's capture_render.rs:393 + layer_targets.rs:426 use the merge-base 10-arg shape, our HEAD appends show_invisible_walls/metrics/ug_back/shimmer_alpha; the mechanical 4-arg delta + the CaptureWorld/LayerFrame field threading published; bg_style_pick_input is called INSIDE draw_background so transparent to d69; draw_sky gained the same 3 tail args but d69 calls its OWN draw_surface_bg extraction — not a break; RETIRE MONITOR: re-probed at d69 tip 79f914fdb — the 4-arg append is NOT consumed, both call sites still 10-arg, the delta stays OPEN for the d69 lane) | `64d-d69-capture-delta.md` | 🔒 SEALED (retire; delta OPEN on d69) |
| 79 | Doc-sweep lane 8: background/sky census & port (W4-g70: SetBackColor byte-space chain, DrawSunAndMoon event bodies + flares, the live Star.cs pool + starfall chain; code-only) | `79-doc-sweep-sky.md` | ✅ |
| 140 | Map-overlay reconcile + MAP-FAMILY merger instruction + SEAL (f29×e17 adjudication, F1-F6 executed, f46-union-first compose, D18 re-fold receipt, d35/d42 atlas receipts) | `140-map-overlay-reconcile.md` | ✅ SEALED |
| 218 | **SEALED + ORDER CORRECTED - THE .trwld v18 union-layout DOC (byte ORDER authority = 103 §L3-ARBITER A3, d09d7e10a - the BG-SEAL order verdict OVERRULED)** + DOC-SWEEP bg variants + WIRING + V18 FINAL CONSOLIDATION + BG SEAL (W4-g71 lane 16: §1-FULL the 13-branch v18 claimant registry — g70 forest 16 + g72 cave 31 + g71 surface 3 + g69 crimson 1 (e19 dup) + g90 books 2 + c39 moon-bosses 5 (f39 dup) + g71 underworld 1 + g87 pets 3 + d125 empress 1 + c43/f45 identity 44 = 107-byte fixed head + d109 dense W×H LAST; supersedes the 21/53/65/97/98/100-byte drafts; §1-SEAL receipts the three external 107 re-derivations at their latest tips (g88 103 L3-DEFINITIVE fc5cf11ae / f44 132 RETIRE §3-L3 d35876af9 / g72 64a+64b SEALED b069439eb — the latter defers layout authority here per the Lead) + the ORDER adjudication OVERRULED by the PERSIST-ARBITER (g88 103 §L3-ARBITER FINAL d09d7e10a ruled D5 strict-C#-row order DEFINITIVE - the books-before-moon middle inverted WorldFile.cs:1407-1408 vs :1347-1351; 1-FULL.2 CORRECTED to A3: crimson [50]/moon [51..56]/underworld [56]/pets [57..60]/empress [60]/books [61..63]/identity [63..107]; registry/census/dedups + 1-FULL.4 site table STAND; A4 mutual receipt closed; the net fix d2fcf9f2 UNAFFECTED - wire order, not byte layout) + the BG-SEAL RECEIPT phase (§1-SEAL.3 f45-flags compose receipt — exact supersession hunks re-read, f45 has NOT published a read, standing input seated; §1-SEAL.4 WorldInfo [u8;13] census keep-one — 10 net-lane tips + main + f45 + g72 carry the inherited f6b736a4c [u8;11] untouched; ONE other editor g52 856edb86c (five-style partial fill on the old width — strict subset, g71 d2fcf9f2 width+full seam WINS, g52 keeps its re-push cadence scope); f45 field-ownership compose field-disjoint; §1-SEAL.5 cross-receipt 103 d09d7e10a / 132 d35876af9 (owes its one-move re-pin) / 64b aa1645481 D5-RE-PIN + g71f mutual receipt CLOSED / 138 9a4acd729 D5-RE-PIN agrees / 135 43fee29a4 §10g agrees — FOUR of five L3 dependents on the identical strict-C#-row middle, the persist-order dispute RESOLVED on the published record; registry tips refreshed e19 c7a38bea / d125 7a048230 / g90 43fee29a, every claimant save.rs byte-unchanged diff-verified) + the three registry hash slips fixed (e19/d125/c43) + the f45 flags-block compose audit (f45 = the .wld base, g71's live setBG hunks supersede its zeros); §2.1-FINAL keep-one: g72 2184ba07 DID land cave wiring — cave_bg.rs adopted VERBATIM, bg_rolls cave dup DROPPED, :11338-tree→:11339-cave→:11340-surface→:11341-moon compose, double-draw risk resolved fc76a48b; §5 the final roll→persist→net→draw witness-chain COMPLETE on the logic side (the net hop closed d2fcf9f2: WorldInfo backgrounds [u8;11]→[u8;13] NetMessage.cs:245-257 + world_info emit + client setBG wire-order apply MessageBuffer.cs:493-505); boot re-roll Main:6594-6603 menu seam; atlas budget guard fit_bg_styles_to_atlas; ocean bg STATIC no-op; oasis row-250 verified + underground-desert = g72's; remaining gaps = sheet supply (d35) + g72 draw modules + the forest pin, all merge-scoped; code-only) | `218-doc-sweep-bg.md` | ✅ SEALED |
| 279 | WLD FINAL walk audit (W4-f45: the complete SaveWorldFlags/LoadWorldFlags ownership table, the g90 booster keep-one, the g71 v18-union cross-read, the 4 VERSION-gate fixes; code-only) — **SEALED**; the g71 `b27474a44` §1-SEAL.3 compose receipt consumed here (f45's owed read = this doc), the g70 treeStyle conflation pinned to one fix-hunk pair — renumbered 137→279 per the research/150 registry §6.3 (137 belongs to g84) | `279-wld-final-audit.md` | ✅ |

> **land-MD supersede (Lead arbitration v615 / f43b):** the rows above that
> seal the **107-byte** v18 fixed head (263, 64b, 182, 218, and the BG-family
> bible in ANALYSIS) are superseded at `integration/land-MD` by a **115-byte**
> head — the 6-byte crimson+moon quintet tail is unchanged; f43b adds `blue`
> [61] and the 7 slime unlocks [64..71], books move [61..63] → [62..64], and
> the identity block moves [63..107] → [71..115]. The research rows stand as
> the historical audit trail; the merged `save.rs` is the authority.

## Fog of war (not yet specified)
- Mod-era entity families beyond 1.4.5.8 (Calamity/Fargo scale) вЂ” enters when the
  counts question is ticketed.
- MP transport design details вЂ” blocked on the engine design doc (separate doc).
- tModLoader-style mod API surface вЂ” only if the destination is redrawn to include it.
