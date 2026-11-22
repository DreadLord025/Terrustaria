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
| 28 | Buffs & debuffs (apply, UI, sync) | `28-buffs-debuffs.md` | 🔒 SEALED → 28g |
| 28g | The BUFF-FAMILY SEAL (registry FINAL: 270-id zero-dup + 400 icons landed; the f49-mana-seal 3-ASSIGN stub bridge + extra_regen-sum:11492 + items.rs-wholesale; Panic-63 canonical vs the f50/f48 THIRD copies with re-pinned drop-hunks; the owed merger-actions checklist; the buff-family bible) | `28g-buff-family-seal.md` | 🔒 SEALED, 5 witnesses written not run |
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
| 78 | DOC-SWEEP lane 6: weather/events census (W4-c40: severity ramp / wind→projectiles / rain start-stop / lightning strike / wind tiles — all landed or sibling-owned; the FALSE "SandstormSky dead code" claim corrected; calm-intent bands witnessed; code-only) | `78-doc-sweep-weather.md` | ✅ |
| 63 | Special-bullet effects census & port (S1-P-i18: the 11 aiStyle-1 bullet specials + Luminite; all MISSING at base — no proj_data rows; code-only) | `63-special-bullets-census.md` | ✅ |
| 270 | W4-c10 waterfowl band FINAL (aiStyle-68 + walkers + 1080 drop: the one-paragraph bible, band-complete ledger, the b3/d20/d76/d36/g8/d40 cross-lane contracts incl the roster-tail + d76 keep-ones, 37-witness index, merge checklist; code-only; renumber chain 145→149→270 — g89's rod doc pushed first at 145, e20's devices bible pushed first at 149, see the doc's renumber notes + d54's 150 registry §6.3) | `270-waterfowl-band-final.md` | ✅ SEALED |

| 315 | The convert-tables family bible (W4-c32-convert FINAL: the c32 tables-only census, BiomeDungeonRoom→DungeonUtils.ChangeTileType NOT Convert, f25's zero-call-site 15/16/18/19 cells, g42's FOUR Convert(3) sites, the whole-switch ownership map + tip hashes + merger contracts; number 315 per registry 150 §9 UNIFIED FINAL, d54 b243c7086, chain 281→315 — the original 281 self-pick per the ≥281 free block, fresh 362-tip sweep) | `315-convert-tables-family-bible.md` | ✅ |

| 68 | FindFrame wire-census (W4-d44 merger assist, refresh 5: per-kind ported/wired tables, g6/g7 twin-hook union plan, the FindFrame-family declaration §3.0/§10.4 [f26 7g-7k + f27 e10-e13 seams, the d38-kinds+d44-hooks merge-compose]; 198 WIRED [the wave-B residual 294/296 the LAST stand-in pair], the 480/481/482 trio RESOLVED keep-inline, the 634 SporeBat RNG-seam contract published, ZERO true no-coverage UNCLAIMED at the probe-time tips [53 module-claimed + 77 roster-arm-covered]; code-only) | `68-findframe-wire-status.md` | ✅ |

| 198 | World-lane dust WIRING keep-one + seam census + mount-dust-family bible (W4-f6: e4 keep-one, g68 tile_draw_rand ONE-field, UseDrill activation contract, minecart 4 delegates + staged fart cloud, SEAL receipts - d54 131 s8.1 push-owed DISCHARGED; rename chain 67→198 round-1, 67 stays d54's until the dust-union merge) | `198-dust-world-wiring-keep-one.md` | SEALED |

| 65 | Main.cs dust sites 1-8 (g11: snowing + TeleportEffect 0-5/7; wiring + union-exec pointers; DUST-RETIRE final sweep: every Main.cs dust-family window vs the union-table — ZERO unclaimed sites) | `65-dust-main-sites-g11.md` | FINAL (union-table closed) |
| 83 | Weather reconcile, g11 addendum (snowing keep-one: d130 base + 7 exact g11 folds; §-patch F1-F7 as EXACT patch-hunks; FINAL: the §-adjudication re-walks the :17238 SwapRandom scope — the g11 pre-swap/global claim STANDS; d130 tip 560a02ea executed F2/F3 + kept its splitmix by its own §2 merger-delegation, F1 stays the recommendation; g11's snowing marked DEPRECATED in-code for the merger drop; DUST-RETIRE §-receipt: d130e APPLIED F1+F5+F7 at 099df89e2 — splitmix retired, global UnifiedRandom, g11 mirrors ported; cull-kill at local tip 16a792d56, origin still 560a02ea — push-rescue flag; merger delta = the DROP only) | `83-weather-reconcile.md` | FINAL + RECEIPT (F1 APPLIED on d130) |
| 116 | TeleportEffect 4-way union (g11 wired 0-5/7 vs g61 widest-11 vs d54 latent-10 vs g12 staged-10/11/13: per-style table, Entry-OLD->Exit-NEW queue canonical, 6 wiring gaps, keep-one = g61 body + g11 seams) — EXECUTED 56afc0c0+8540a8ad + FINAL §5/§6: the g61b co-lane cross-verify (no spawn dup; the fade PRE-AI re-slot + the statue style-14 queue latch folded), the main_dust.rs g11×g13 filename union-plan (ONE-file per-fn keep table for d54f-131); 33+3+2 witnesses; g12/d54 drops are the merger's; DUST-RETIRE §7: g61b probe ALIVE at b5ff1803d (+9 local, seam-6 key-summon NO DUP) + the ONE-paragraph merger instruction for dust-master 131); SEAL §8 (2026-11-21): the 4 receipts — the union-chain census 9a9ac1fd/98283df0/2e978b279 (6 gaps re-verified in code, seam-6 NO-DUP), the g61d spawn-seal MUTUAL at pushed 6e922fc6 (the §7.1 push-owed flag RETIRED; 7 DEPRECATED banners re-read g11-canonical; poof/mimic/transform/play_teleport_cue STAY g61), the d54 bible 131 §8.3 item-4 main_dust THREE-WAY per §6's per-fn table, the teleport-family bible (ONE body + ONE seam set, fold-sources DROP, witness census ALL WRITTEN NOT RUN); no further g11 commits planned) | `116-teleport-reconcile.md` | SEALED (EXECUTED + FINAL + RETIRE) |

| 366 | THE ADDLIGHT-FAMILY BIBLE (W4-g18 SEAL): the git-log census (every S1-E-5 wave-4 lane at its pushed tip), the d54-bible reconcile (Dust 94/94 + proj-107 86-covered/21-absent-rand-gated + NPC 80 + Main 5), the c29 gems-NEVER-emit-light premise REFUTED (no tileLighted write / dust-43 sparkle / no scanner case / g18-negative-census-agrees), the e6-addlight-proj-2 compose mutual receipt, the family declaration; number 366 per registry-150 §9 UNIFIED FINAL row 366 (chain 304→338→341→344→360→366; g10-360 KEEPS first-add, g18 fourth claimant); 6 family-census witnesses WRITTEN NOT RUN | `366-addlight-family-bible.md` | ✅ SEAL |

| 321 | Death-effects-family bible FINAL + the W4-g59 SEAL (research FINAL; renumber 283→321 per registry-150 §9.2 — f26's §7.3 reservation KEEPS 283, g59 → 321; original round-2 pick: 283 first-free at 360 tips): SEAL 1 ACTIVATIONS-2 landed (17 newly-rowed + 15 base-folds + 129 match types / 43 row-gated remainder); SEAL 2 g60f compose consumed + refreshed at g59 d277f782b × f5 b68eac297 — g59∩f5 = 110 kinds / 56 fn-rows (+17 = the activations), strip total 221 actionable / 0 conditional (supersedes 204); SEAL 3 f5f gore-seal consumed + f10 afea4c0fc delete EXECUTED (152 fns/147 gates/157 witnesses cross-verified) + g62 f5ce54743 the 4 hit-half bodies PORTED; SEAL 4 boundary CLEAN max cs_id 229 ≤ 230 (mechanical 129-id re-check); SEAL 5 the family bible one-paragraph + gate-commands re-run (1578 Σ / 853 window) + witness union 418 distinct (was 404, +14); code-only | `321-death-effects-family-bible.md` | ✅ SEALED |

| 137 | g84 inline arms vs g31 drains keep-one seam-mapping (W4-g84 final phase: the per-kind MultiTileGrass/MultiTileVines arm↔bucket table, the duplicate helpers, the animator-row feed the merger must bind) | `137-g84-g31-keep-one.md` | ✅ |

| 91 | Fighter (aiStyle 3) doc-sweep lane 15: d22 core probe, graveyard bash gate, door-open census, confused generic arm | `91-doc-sweep-fighter.md` | 🟡 |
| 91-A | Fighter residuals: the d22r RESUME keep-both fold-list (graveyard/confused hunks), the blood-moon door-break census correction, the confused-immunity table fix, the coin/star cross-notes | `91-addendum-d22r-fold-list.md` | 🟡 |
| 91-F | Fighter FINAL: the heart/star seam ported (f35 disclaimed it), the d22r reconcile receipt (fold-list CONSUMED at bcd42ddd1), the fighter-variant census (the MartianWalker 520 shooter remainder), the banner-carrier/goblin-peasant resolution | `91-final-fighter-census.md` | ✅ |
| 91-S | Fighter SEAL: the corrected keep-one — f35 ported the heal seam FIRST (b72cdfbfc/00a5b27db, a superset; the census :39 disclaimer was a BEFORE snapshot), g4's duplicate RETIRED; the g4-unique spawn_coins LIVE-inputs hunk + the Thief/Skeleton witnesses cherry-route; the 520 shooter remainder = d41 (band 491-560, the salamander precedent); 91-F SEALED + the fighter-family bible | `91-seal-keep-one.md` | ✅ |

| 145 | RoD teleport_style(1) ⨉ TeleportEffect-union composition (W4-g89h, RENAMED from 139 at g89j — g52c-net-master owns 139; the rod call COMPOSES with g11's executed union — style-1 Entry/Exit dust now-live via the queue; NEW gap 7: the editing-phase caller drains a tick late so the chaos Next(2) swaps ahead of the dust draws on UpdatePlayers [determinism break] — the g89j §2a EXACT drain-move hunks re-pinned at g11 FINAL 98283df08 [drain INSIDE run_rod_arm between the teleport and the chaos tax]; the gap-5 camera dedup; merger actions WRITTEN NOT APPLIED — seam unreachable on g89's base) | `145-rod-teleport-union-composition.md` | cross-note for the g11/g89 merger |
| 122 | .trplayer v9 addendum + the FINAL-FINAL receipts (W4-g89d/j: the v9 lineage contract, §6 the g89i v10 token-4 receipt, §7 the g88g-v10-token-4 SAME-slot double-writer check [same line/token/flag — g88 v11 superset keep, g89 v10 writer/reader DROP; the pre-merge "11"-reject incompatibility recorded] + the aegis-family declaration 5338/5341/5342/5337 ALL LANDED, merger per 138/103 §9) | `122-persist-v9-addendum.md` | FINAL — the live layout authority is g88's 103 |

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

| 91-D22 | The 123 unrowed C# aiStyle-3 types — the d34-d43 roster-gap table (W4-d22 retire R2, the d37d-E6 input): per-type SetDefaults cite + corridor lane + pushed-tip claim status (75 ROWED / 48 open) + the AI_003 extras each faithful port must carry; the fighter_walker_kind extension list; the 590/591 + 368 census corrections | `91-d22-roster-gap-table.md` | ✅ |
| 91-G4 | G4 DEPRECATED — the fighter-family canonical record (d22) + the final KEEP/DROP symbol table (W4-d22 retire R3): every g4 commit f384797c4..a9369ec38 adjudicated with d22 landing sites; the P6 generic-walker flips verified landed; the merger instruction (merge d22, never g4-after) | `91-g4-deprecated-keep-drop.md` | ✅ |
| 91-FR | The d22r keep-both fold-list FINAL RECEIPT (W4-d22 retire R4): the g4 addendum §7 checklist all 9 boxes CLOSED with evidence, the retire deltas (R1 moan-position closure / R2 roster table / R3 deprecation), the fighter-family merger instruction, the branch hash receipt | `91-addendum-final-receipt.md` | ✅ |
| 91-FWK | The fighter_walker_kind extension obligation EXACT HUNK (W4-d22 seal S2, the d37e-E8-seal input): the append point + ai[3]-free precondition + per-lane variant names probed at pushed tips + the witness obligation; the 294-296 seal correction to the R2 §2.1 list | `91-fighter-walker-kind-obligation.md` | ✅ |
| 91-BIB | The FIGHTER-FAMILY BIBLE (W4-d22 seal S4, SEALED — FINAL, the FIGHTER-CORRECTION consumed): d22 canonical styles 3-6 (the landing table), **f35 owns the DropHeals heal seam** (ported FIRST superset b72cdfbfc/00a5b27db, g4's duplicate retired by the g4 seal e5915bfab — the S3 a9369ec38 no-delta probe was STALE), merge d22+f35 NEVER g4-wholesale + cherry g4's 2 unique hunks (spawn_coins midas/blood-moon LIVE inputs d22:7218+f35:6954 still false,false + the Thief/Skeleton scan witnesses), DROP g4's scan type-bonus (d22:498-513 identical) + pick-path Peon smash (KEEP d22 kill_tile), **the 2 cherry hunks LANDED IN-BRANCH by the g4d fold** (spawn_coins LIVE inputs + witness, the Thief/Skeleton scan pair — verify by symbol, do NOT re-cherry), g8/g9/g10 compose disjoint, d34-d43 take the R2 table + the S2 obligation + the 520 MartianWalker shooter = d41 task seed (NOT d43); the c8 110-tick pin consumed (S1) | `91-fighter-family-bible.md` | ✅ |
| 91-FC | The FIGHTER-CORRECTION receipt (W4-d22, the g4 seal e5915bfab consumed): the stale-probe timeline, the keep-one INVERTED (f35-owns-DropHeals, the census :39 disclaimer a BEFORE-this-task snapshot), the corrected merger instruction + the 2 cherry hunks with d22/f35 re-verified stand-ins, the Peon kill_tile keep, the 520 d41 task seed with the C# params re-read; the g4-side record `91-seal-keep-one.md` lives on origin/task/W4-g4-npc-style3-fighter | `91-fighter-family-bible.md` §0/§2/§6 + ANALYSIS 'The SEAL-receipt CORRECTION' | ✅ |

| 111 | DOC-SWEEP screen-fx lane (W4-d70: the paladin peer loops, the lens-flare shadow offset + sun-scorch, the MoonlordDeathDrama producers, the screen-shader census; code-only) | `111-doc-sweep-screenfx.md` | ✅ |

| W4-f12-sweep | The FULL 270-id UpdateBuffs stat-arm census (C# case-list vs every ownership set; 10 gaps closed inline; the nebula-twin f49 one-writer contract + the buff-321 d115 keep-one note; the final-phase g27 cross-verify appended) | `W4-f12-sweep-census.md` | ✅ set-diff EMPTY |
| W4-f12-final | The buff-HUD final census (icon draw / tooltip lines / time display / debuff cancel / g27 canonical; the 94/396 tooltip arms restored, the icon-text font/scale/tint fixed; ANALYSIS "The buff-HUD final census") | `W4-f12-sweep-census.md` + ANALYSIS | ✅ 5 witnesses written not run |
| W4-f12-closeout | The 5-item buff closeout (f49d status NOT-landed → the exact fold_orb bridge hunk published; the animation census — no Main.buffFrame exists, 400 static icons, Ice Barrier 62 the one wired frame anim; the stack-source census COMPLETE — xN is projectile counts, re-grant semantics per-buff; the FINAL keep-one re-sweep script — set-diff EMPTY + TWO NEW f50 cross-tree dups ruled (Panic-63 inline, the beetle/solar ladder re-derivation); ANALYSIS "The buff closeout") | `W4-f12-closeout.md` + `tools/w4_f12_closeout_sweep.py` | ✅ 4 witnesses written not run, script run green |
| W4-f12-retire | The retire receipts (f50 tip ff56e6e51 retired NOTHING → both dup rulings RE-FLAGGED with exact drop-hunks :7010-7013 / :4606-4708+:4582+:7451-lift+:6203-6205-flag-fix + the two witness retargets; f49 6c0ee2aa2 R3-exec still NOT landed → both-halves bridge/deletion ranges pinned; d73 33c38000a ancestor + blob-identical boosters, sweep re-run set-diff EMPTY; the ONE-paragraph BUFF merger-instruction — f12-canonical + g27-orbs + d115-321-folds + f50-drops + f49-bridge) | `W4-f12-closeout.md` "RETIRE phase" | ✅ receipts recorded, script re-run green — the f49/f50 monitor notes SUPERSEDED by 28g |
| W4-f12-seal | The BUFF SEAL on `task/W4-f12-buff-effects` (the five seal items: 270-ids-zero-dup + 400-icons landed; the f49-mana-seal — R3 EXECUTED at their SEALED 0d1f941af, the 3-ASSIGN stub bridge + the extra_regen SUM :11492 + items.rs wholesale; Panic-63 canonical — the f50 51eefdf84 / f48 e5b66bb06 THIRD copies drop with re-pinned hunks; the owed merger-actions; research FINAL registry 28g + the buff-family bible) | `28g-buff-family-seal.md` | 🔒 SEALED, 5 witnesses written not run, sweep re-run green |

| 381 | NPC kinds 351–697 SetDefaults census + first-20 port (W4-f18; 343 arm-bearing ids, 105→125 ported, the value() merge-scar repair, the d37-E8 roster-collision keep-one + exact drop-hunks + the kinds-verify-2 family bible; SEALED, code-only; number chain 64→249-voided→381, registry-150 §10) | `381-npc-kinds-351-697-census.md` | ✅ SEALED |

| 335 | W4-f30 SYSTEMS RETIRE (keep-one drop-hunks c46/d70 vs f30 + the c24/f47/d2 compose meeting points + the FINAL notify-band census; renamed 240→335 per the d54 registry 150 §9.2 — g58's earlier 143→240 rename wins 240; code-only) | `335-f30-systems-retire.md` | ✅ |

| 105 | DOC-SWEEP lane 25 lantern/events (W4-f40: ReleaseLantern 4702 USE arm + noUseGraphic draw gate, the 4703/5613 census correction, the c39 gates union contract, d130 wind-drift verify, f45 lantern-quad persistence-order fix; FINAL s8: the c39 114-exec cross-receipt + world-seed graft, the music ambient-only census, the 4702-recipe-less/5613-bench craft census, the f45 compose-contract witness, the doves-volley correction; RETIRE s9: the doves-dup adjudication d17-form keep-one + brace fix, the f41 shop-row dedupe receipt, the 114 FINAL merge-scope receipts, the LANTERN-FAMILY unified merger order; code-only) | `105-doc-sweep-lantern.md` | ✅ |

| 274 | VOID-FAMILY BIBLE + the g56 RETIRE verdicts (the `.trvault` two-seam retire gate vs f44 bank4/g88 v11, e19 disjoint + the 16-box repin receipt, g88 persist zero-touch, SetGlowForChest keep-one g56; renumbered 239→274 per registry-150 §6.3 `72226e500` — 239 belongs to d42's earlier push 12:55:30) | `274-void-family-bible.md` | ✅ |

| 244 | The cursorItemIcon system census + task-seed + owner contract (W4-g86: 357 writes / 169 ID-writes in Player.cs; g81 owns the held-kind draw + cache + enabled latch; the altar hover + ItemCheck-head batches ported-unwired; the g34/g43 altar keep-one = disjoint layers) | `244-cursor-item-icon-seed.md` | seeded |

| 63 | Special-bullet effects census & port (S1-P-i18: the 11 aiStyle-1 bullet specials + Luminite; all MISSING at base - no proj_data rows; code-only) | `63-special-bullets-census.md` | ✅ |
| 135 | W4-g90 persist fold-input — **SEALED, CONSUMED by g88 fold 7 a9f1c29d8 (v11), re-pinned at g88 SEAL fc5cf11ae** (the v7 six-line block's exact lines/gates; §2's six-line recognition rule LANDED on g88 verbatim — witness g90bs_six_line_v7_loads_positionally re-verified present at fc5cf11ae, §12a; the .plr-bool ↔ unlock-token mapping — §3's pearl-first recommendation SUPERSEDED by the written crystal=4/arcane=5/pearl=6 order per 103 §8/138 §6; the .trwld pair SUPERSEDED THREE TIMES: 19/20→22/23 (138 §5a's 68) → 104/105 (c39 114 §8's 106, §10) → **[61]/[62] the LAST flags block in g88 103 §L3-DEFINITIVE D3's 107-byte head** (§10g — the SEAL receipt: D1's 13-claimant/11-block census locates g90 as the books-2 row, the STRICT-C#-row middle crimson/moon/underworld/pets/empress/books + identity-44 LAST, the six truncation sites → +107 re-pinned comments-only, the empress-order nuance RESOLVED after-pets); the combat-book consumer census — town ladder FED 8521139d, fishing/TRAINING_DAY/satchel armed; §8 the persist-merge receipts + the stale-flag census; §9 the 138 keep-one receipts — g90 KEEPS the 5326 loaf + the skeleton-merchant gate + all carriers/consumers/Player-field anchors, the shimmer arms dedup onto a6's machine; §11 the BOOSTER-FAMILY declaration — bread/books/satchel/shimmer-6 all-landed carriers+consumers; §11a the b3 feeder receipt (b3 tip UNCHANGED 96a5d88f0 at the seal re-probe, phase 1 ONLY — the feeder-contract unconsumed receipt stands); §11b the retire anchor census + the ordered merger checklist + the 26-witness census; §12a the g88-v11 torch-line + six-line-rule lossless receipt (guard :463-465 / reads :470-481 / witness :1198 / writer :230 all present at fc5cf11ae; the 132-RETIRE §2e dh-fold moves DD2's bump v13, orthogonal); §12b the booster-family bible (a6-machine / g90-carriers / g88-v11 / f44-plr / f45-wld / D3-107) — NO open g90 obligations, the lane SEALED; code-only) | `135-g90-persist-folds.md` | ✅ SEALED |

## Fog of war (not yet specified)
- Mod-era entity families beyond 1.4.5.8 (Calamity/Fargo scale) вЂ” enters when the
  counts question is ticketed.
- MP transport design details вЂ” blocked on the engine design doc (separate doc).
- tModLoader-style mod API surface вЂ” only if the destination is redrawn to include it.
