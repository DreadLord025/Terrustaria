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
| 372 | d78 FINAL Kill dedupe (g1-dup / g3 keep-one / the 620 d77 co-claim / the drop-vs-keep tables) — SEALED §6: g1-drop-hunks refreshed (602ln), the d77-617-drop mutual receipt, the g3-cue-position-fix hunks, the blasts-family bible (renumber 26f→372 per registry §10.2; 26f belongs to f20) | `372-d78-final-kill-dedupe.md` | sealed |
| 160 | g1 Kill-hook census (SEALED — g1 keeps 459/521/522/80, the 452/454/448 EVP trio folded to d78 per 26f §1) — renumbered 26d → 160-round-1 per registry §10.4 | `160-projectile-kill-hooks-census.md` | done |
| 373 | the proj-missing-a family bible (lane ledger + the g1×d78/f20/d15/g65/d77 merge instructions) — renumbered 26d-g1 → 373 per registry §10 | `373-g1-proj-missing-a-family-bible.md` | done |
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
| 106 | DOC-SWEEP lane 26: the WorldGen.cs sound deferrals (W4-g24: the quit-path 34/35 stops + JustQuit, the serverLoad dedServ skip, the AppTownDoors b3 seam, the c14 keep-one; code-only) | `106-doc-sweep-wg-sounds.md` | ✅ |
| 146 | WG-SOUND FINAL (W4-g24: the b3 adoption contract + keep-one, the 79-site census re-sweep ledger, the secret-seed cues vs the g23 union + the two WorldSeed-arm remainders ported, the gen-progress census vs d85/f23; code-only) — renumbered 107 → 146 in the retire phase (f21b owns 107; 145 double-claimed by e19/g89); §5 RETIRE: the g73 quit-stop cross-receipt + the b3 monitor + FINAL-FINAL + the wg-sound-family declaration | `146-wg-sounds-final.md` | ✅ |

| W4-e10 | Voice-family SEAL receipts (the f44g hydration closure + the d79 rewire hunks + the g21 disjointness + the f3 65-seam compose-note + the voice bible; docs-only over 8e0d951fc) | `W4-e10-voice-seal.md` | ✅ SEALED 2026-11-21 |
| W4-e10 gap | The 51-unclaimed gap census: 4 PORTED (pet grants 17306/17338 + sign tails 33537/33547) → 211/164/47; the d79 cross-check (16 strict instrument sites, 63/64/65/66 zero Player.cs) + the owner-contract task seeds (instruments/Journey/QueenSlime/type-75 + the 23-site backlog); code-only, 10 witnesses written not run | `W4-e10-gap-census.md` | 🟡 CODE-ONLY 2026-11-21 |

| 141 | NPC-sound FINAL census (W4-e9): the 27-tail activation census (all UNWIRED — d33b/d41r/d42r/d43r do not exist), the second 147-quirk + the critter cs_id key fixes, the f2/f47/d53/g2 co-lane keep-ones, the 32/32 idle-loop census; §8 TAIL-ACTIVATION-2 re-probe after the roster pushes (d42 vanilla-free 43-kind row, d33's wired-114-only axis activates ZERO tail cues, :24449 machine-owned-LIVE on f42/g8, the per-kind activation contract + the ten-tip critter-rekey scan with the exact f2 hunk); **§9 SEALED — the npc-sound-tail-family bible (e9 wholesale DROP / f2-canonical `b754bdcd`, the exit-LAVA hunk MOOT under the DROP, the twelve-tip rekey re-scan + d43-rowed/c10-fixed deltas, the Frog/Duck arm-label correction, the style-114 ZERO-tail-cues re-verify)** | `141-npc-sound-final-census.md` | SEALED (code-only) |
| 358 | DOC-SWEEP lane 5: WorldItem/items entity census + port (W4-g15: merge/draw/lava/magnet verdicts + MakeAWish/CheckInWorld/TryFillBucket/ItemNoGravity/copper-slime/enemy-delay/hopper/hold-expiry ports; witnesses written not run) — SEALED: the 24-commit lane pushed (af7d97534), the g45-compose debt discharged, gaps seeded (39→d98 per 245, the 684-transform→post-merge), e19b/g52b skips receipted, the 145 16-box repin executed (8f04df01b); number per registry-150 §9.2 UNIFIED FINAL — chain 77→284(§7.3 target VOIDED: 284=g52)→358 | `358-doc-sweep-items.md` | ✅ SEALED code-only |
| 347 | The item-dust-family bible (W4-g15 SEAL round: the WorldItem/Item dust+entity fold-table — g15 the 17-site DustSink keeper, e19 the splash/repulsion/16-box/wet-gravity keeper per 144/145, the f47+g15 splash dup-halves retire, g15==d98 msg-21 byte keep-one, the merge order + the post-merge seeds; number per registry-150 §8.7 FROZEN assignment — the 309 self-pick at `131961a73` contested, chain 309→347, d125-309 stands) | `347-item-dust-family-bible.md` | ✅ code-only |
| 184 | Expert/master boss drop trees census + port (W4-f35, chain 65→184 per registry-150 §3 executed at batch-O merge: the bag OPEN side — OpenBossBag's 19 grant switches + TryGettingDevArmor + the money tail + the ItemSlot right-click open arm; the widened scope — the full 58-kind OpenableBag set: the 26 fishing crates + the presents + the misc bags + the death-heal feed; the death side already landed S1-E-14a; code-only) | `184-expert-drop-trees-census.md` | ✅ |
| 195 | Bag-open mechanics + master-mode bag drop-rate census (W4-f35 DOC-SWEEP, chain 66→195 per registry-150 §3 executed at batch-O merge: the right-click open dispatch, the luck census — NO luck at open time, the death-side luck-free bag gate + the master pet/mount rule fixes, the master-mode relic/pet rates, the e25 value_multiplier boundary) | `195-bag-open-master-census.md` | ✅ |
| 144 | Item liquid-splash keep-one verdict (W4-e19 final + SHIMMER SEAL: the f47 keep-f47 counter-verdict consumed, the e19 keep UPHELD — the entry gate made verbatim (lava_now), the 8-row f47-drop-hunks table published, 2 f47 probe twins folded + 1 gate witness; TR_WATER_STYLE pin adopted) | `144-splash-keep-one.md` | ✅ SEALED (arbiter fallback) |
| 145 | The 16-box cross-lane repin list (W4-e19 final + SHIMMER SEAL: SIZE 14→16 per WorldItem.cs:138; per-lane OWN-BRANCH execution instructions at every lane's current tip — grab edge 42/43, magnet 292/192, floor−SIZE, f49 nebula +92/−8; the g15 line drift corrected) | `145-16box-cross-lane-repins.md` | ✅ execution published |
| 146 | The AETHER biome census (W4-e19 final + SHIMMER SEAL: ZoneShimmer metric/music faithful; bg-skip + star-dim + monolith ramp ported; liquidAlpha[13] dead-code; the g69d-star-visual boundary CONFIRMED disjoint at the four C# windows, g69e-clouds-retire reads) | `146-aether-biome-census.md` | ✅ census + boundary sealed |
| 147 | The quicksell SHIFT-click UI-hint census (W4-e19 final + SHIMMER SEAL: the C# mouse hint IS the cursor-10 icon; the override draw ported; the g86e-cursor-170-sites COMPOSE published — cursorOverride layer 36 vs cursorItemIcon layer 40, keep BOTH, the two reset clocks separate) | `147-quicksell-ui-hint-census.md` | ✅ compose published |
| 148 | W4-e19 research FINAL (SEALED: the full arc + the SHIMMER-seal round — the arbiter consume, the repin execution, the aether boundary, the cursor compose, the shimmer-family bible in ANALYSIS; the witness ledger, the five merge contracts, the standing deviations) | `148-e19-research-final.md` | ✅ SEALED |
| 224 | Gem-moss biome gen census + fidelity port (W4-c29: Spread.Gem duplicate-visit redraw / the 178 PlaceTile anchor gate / neonMossBiome clamps / moss arm A ledger + arm F live sweep vs WorldGen.cs; g41 co-lane claim check; code-only) — renumbered 95 → 224 per registry 150 (c42 keeps 95) | `224-gem-moss-census.md` | 🟡 |
| 260 | DOC-SWEEP lane 40 gem/moss residuals (W4-c29 sweep: gemsparkle DRAW d60-claimed / moss PAINT f25-C8-wired / extraction faithful / neon CONVERT d103 / 627 no-def refuted; the REAL gap = the gem/moss per-cell break-drop slice ported 55555fb9b; code-only) — SEALED + the gem-family bible + the dead-man's-chest re-census (FAITHFUL); renumbered 130 → 260 per registry 150 (f28 keeps 130) | `260-doc-sweep-gem-moss.md` | ✅ SEALED |
| 219 | The 949 Torch God keep-one (W4-f33 vs W4-d19, chain 93→219 per registry-150 §3 executed at batch-O merge): the arm-by-arm verdict — KEEP f33's `projectile_ai_torch_god.rs` (the 32-draw census, the Kill arm, the live hitbox, the XNA float forms) | `219-torch-god-949-keep-one.md` | ✅ recommendation |
| 221 | The f33 closeout fold-input + RETIRE FINAL (chain 93-addendum→221 per registry-150 §3 backfill, executed at batch-O merge: the v7 pair's fold-input CONSUMED by g88 v11 `a9f1c29d` — §1-receipt ↔ 103 §2b; the d19b-tip exact 949 drop-hunks; the c36 row-chain verification; the music-101 census; §3b the a6b `d7de8bd5` 5043 re-compare — KEEP the f33 full body; §7 the whole-window event final-verify — no arm missed; §8 the FINAL merger instruction) | `221-torch-god-fold-input-addendum.md` | ✅ FINAL (retire) |
| 284 | g52c net final-audit: the relay-host arms census (SEND side; the g48 FINAL-table feed, the 34 destroy keep-one, the 34/162 codec census; 139 stays RESERVED for g52c-net-master; number per registry-150 §9.1 — 284=g52 pre-§8 KEEP) | `284-g52c-relay-host-arms-census.md` | ✅ code-only |
| 156 | W4-d5 projectile aiStyles 41-50 dispatch census (41/42/43/46/49 MISSING ported UNWIRED, 44/45/47/48/50 verify-only, gaps F1-F6; renumbered 26d→156 per registry-150 §3, d12 wins the key) | `156-projectile-ai-41-50-census.md` | ✅ census |
| 161 | Projectile aiStyles 31-40 COVERAGE census (W4-d4: 5 MISSING arms 32/35/37/38/39; renumbered 26d->161 per registry-150) | `161-projectile-ai-31-40-census.md` | census |
| 167 | Projectile aiStyles 61-70 dispatch census (W4-d7: 63/69 MISSING ported UNWIRED; renumbered 26e->167 per registry-150) | `167-projectile-ai-61-70-census.md` | census |
| 169 | Rope-coil DUPLICATE keep-one (W4-d4 DOC-SWEEP: the g82 union FINAL-FINAL re-adjudication; renumbered 26e->169 per registry-150) | `169-rope-coil-keep-one.md` | keep-one |
| 170 | Projectile aiStyles 21-30 band status (W4-d3: every style armed, 25 the ported boulder family, the other nine verify-only + the effect-layer gap map; renumbered 26f→170 per registry-150) | `170-projectile-ai-21-30-band-status.md` | ✅ |
| 226 | FINAL-phase proj AI 71-80 (W4-d8: the Drill ability-half port + mining-write split-contract, the 434 legacy-keep verdict, the band 71-80 all-armed census, the g64 fountain-dust + e6 606-light activation cross-notes; SEALED with the retire addendum: Tile.SmoothSlope + the Player HeightOffset family ported, the 434 keep-one FINAL, the smooth-slope-family declaration; renumbered 97→226 per registry-150, d115 keeps 97; code-only) | `226-final-phase-proj-ai-71-80.md` | ✅ SEALED |
| 313 | W4-d9 FINAL: the projectile aiStyles 81-90 band table (the five gap-styles 81/86/87/89/90 ported + wired, the five even styles main's census; the e3 dust-activation contracts + the g78 spelunker-canonical verdict; registry-number 313 per §8.1 row 6, chain 281→313, code-only) | `313-projectile-ai-81-90-final.md` | ✅ SEALED |
| 330 | Projectile aiStyles 51-60 band bible + RETIRE seal (W4-d6: 10/10 styles armed, 16/16 kinds rowed; the g65 gun-splash late-dup keep-one DROP-HUNKS (mutual at g65 346 §2b), the f47 ShimmerImmunity COMPOSE + the c9 third-writer KEEP-ONE, the d53/g64 disjoint line-range receipt, the dust-lane trail activation contract, the 601 PortalGunBolt row fix + the portal-gun teleport TASK-SEED; registry-150 §8 FINAL assignment, chain 247→330) | `330-projectile-ai-51-60-band-bible.md` | ✅ SEALED |
| 331 | W4-d3 SEAL (the boulder-family bible + the five receipts: the landing wave, the d75 SIBLING-CUED sound layer, the g65 Kill-309 keep-one, the 1023/1081 stopped-short re-probe + seed S1-P-i58; renumbered 248→331 per registry-150 §9 — g82's reservation wins 248, code-only) | `331-d3-boulder-family-seal.md` | ✅ SEAL |
| 352 | W4-d1 falling-block FINAL census + RETIRE declaration + the SEAL round (number per registry-150 §9 UNIFIED FINAL, chain 246→352 — 246 = e23's published reservation KEEPS; the three open follow-ups censused whole + ported — the natural-fall spawner's data-path move (the frozen-tail carve-out RETIRED), the forced-PlaceTile bypass set, the tile-cut bodies; the f25/d18 spray-row DUP keep-one (the d18 verbatim shape) + the ai_spray types-1/4 activation + the f25/g42 compose table; the 502 Meowmere row census (d75 part 5 owns it, no d1 action); FALLING-BLOCK FAMILY COMPLETE (the family bible §4); §6 the SEAL — the 491 FlyingKnife row + the LIVE `AI_009_MagicMissiles_Old` machine (the e3 task seed) + the 981 SilverBullet as the special-bullets module's kind 12 (the Next(6) drip gate + the Werewolf ×3), the 246→352 owed code-cites PAID; code-only) | `352-d1-falling-block-final.md` | ✅ |
| 354 | Projectile aiStyles 11-20 RETIRE FINAL (W4-d2: the git-log census, the 261 flail-15-NOT-14 resolution + S1-P-i61 seed, the 10/10-armed + 169/196-rowed census with the 27 named remainders, the d77c/f1 kill-tail DISJOINT compose receipt, the band declaration; code-only, witnesses written not run; chain 283→310 (invalid self-pick — 310 is g48's §9.2 assignment, DONE 09364c008)→354 per registry §9.2 UNIFIED FINAL, d54 tip 4b0d5c246) | `354-proj-ai-11-20-retire.md` | ✅ SEALED |
| 361 | W4-d5 projectile aiStyles 41-50 FINAL + SEALED — the landing census (6 commits, 5 UNWIRED machines), the e3 STAGED style43 emitter ACTIVATED (the 8-kept-NextFloat census REPLACED by crystal_leaf_burst_227, pool internals + POST-spawn blend; the Kill 15-float census dropped to the g65 drain), the all-styles-armed/kinds-rowed sweep (F1-F6 closed on g2's pushed tip; the only residue = d5's own 6 rows + 5 dispatch arms + the crystalLeaf player fields), the g65-Kill-309 compose (227/281 single-roller drop instructions + the g63 tick-3/g16 gore seams), the band declaration; number per registry-150 §9.2 (was 360 per §8.7 new-doc rule; g10-360 first-add KEEPS, renamed 360→361); 31 witnesses WRITTEN NOT RUN | `361-projectile-ai-41-50-final.md` | ✅ FINAL + SEALED |

## Fog of war (not yet specified)
- Mod-era entity families beyond 1.4.5.8 (Calamity/Fargo scale) вЂ” enters when the
  counts question is ticketed.
- MP transport design details вЂ” blocked on the engine design doc (separate doc).
- tModLoader-style mod API surface вЂ” only if the destination is redrawn to include it.
