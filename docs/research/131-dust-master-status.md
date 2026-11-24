# 131 — DUST CROSS-LANE MASTER STATUS (W4-d54 crosslane phase)

**BIBLE-FINAL (bible-final round, 2026-11-21, §8): the consolidated
final record.** ALL dust-lane tips re-fetched (three fetches mid-round —
g61 pushed its 9-commit wiring wave `b5ff1803d`, d130 landed the F1-F7
adjudication + the g67b fold `97ed44f5d`, g11 landed its union-exec +
final docs `98283df08`, g13 declared the Main family COMPLETE
`4444e1011`, g73 completed the falls closeout + the full g75d re-fold
`5d444f15a`, g67 published research/134 FINAL `e777b3e2c`, g60 consumed
g62's trailcache contract `19431e385`). Per-fold VERIFIED/OWED status
table with exact tip hashes: §8.1. The Dust.cs-family COMPLETE
declaration + the per-family spawn-site census: §8.2. The merger
dust-checklist (keep-one symbol list): §8.3. The witness-union recount
at current tips: §8.4 — **the 318 figure is SUPERSEDED: 492 gross /
≈461 net** at the final tips (the lanes landed the lights, the wiring
waves and the folds the 318 census still counted as owed).

**FINAL-STATUS (final-audit round, 2026-11-21): d54 residuals ZERO**
(§6 — anchors/diff/prose re-verified); the owed-fold ledger §5 stands
with row 3 now carrying the EXACT patch hunk (§7.1) and the 0-340
coverage census + d50-lane gap-table published (§7.2). Tips re-read
this round: e4/f6/d130/g73 UNCHANGED from §1 (verdicts stand); **g75
ADVANCED** `2b9ed81f` → `41b64b04` (+4 commits: the :3858 spout wiring
`a3093a66`, DELEGATED-STATUS docs `d441713d`, fold-list extension
`17161648`/`41b64b04`) — §1/§2/§4/§5 g73e rows updated below.

**GAP-CLOSURE ROUND (2026-11-21, post-final-audit):** the §7.2 gap-table
rows closed on this branch — **G3/G4 PORTED** (`dust_table.rs`
`get_alpha_with_globals`, commit `641561c2`), **G5 dead-claim CORRECTED**
(:2275 LIVE via ParticleOrchestrator, `d.color`-reconstructible) +
**G6 CLOSED-EXACT** (sole 235 producer is single-player index 0,
`c5a0e646`), **G2 ONE-EDIT hunk published §7.2a** (`84eb739b`), and the
**f6 atan2 probe: f6 ALIVE** (origin `0f9fd642f`, +8 wiring commits;
§7.1 hunk CONSUMED by its unpushed `76aeaf3c3` — §5 row 3
RESOLVED-LOCAL). Remaining port-wide owed: G1 (the 52-light feed,
S1-E-5) + G2's merge-time landing.

Code-only crosslane round, 2026-11-21, on `task/W4-d54-dust-sites-world`
(union EXECUTED `c792cf37`, research/66 §6). This doc is the **per-file
keep-one FINAL table** for the whole dust family, the merge-order
recommendation, the witness-union census and the outstanding-fold
ledger. Every keep-one decision below was re-confirmed against the
FETCHED lane tips (not the earlier probes); the decisions themselves
are owned by research/66 (d54/g14 seams), research/83 + its g11
addendum (snowing), research/116 (teleport), and the g73 waterfall
union contract — this doc records their FINAL state and the residual
work each lane still owes.

## 1. Tips read (git fetch, this round)

| Lane | Branch | Tip read | State |
| ---- | ------ | -------- | ----- |
| d54 (this) | task/W4-d54-dust-sites-world | `c792cf37` | union EXECUTED: g14 tr-core canonical + LIVE seams re-pointed (research/66 §6 ledger ce1276ba/8f8471cb/74ba4a2e/462a21ff/e41a90f0) |
| g14 | (folded into d54) | `a35efec2` via cherry-picks | tr-core worldgen_dust{,_wall,_tile}.rs, all 29 WorldGen sites, arm-11 torch round-trip + paint alpha-150 fixes applied on d54 |
| g11 | task/W4-g11-dust-main-sites-1 | `d892d8d4` (wiring tip `13a1c48e`) | snowing WIRED at the :17238 scene_scan slot + TeleportEffect 0-7 WIRED (player tick + wiring pads); tip adds research/83 addendum + research/116 union plan (docs-only) |
| g12 | task/W4-g12-dust-main-sites-2 | `665cd1e8` | main_dust_b.rs sites 9-16 (teleport cases 10/11/13 STAGED + bone-arm puffs 22744/22820 + Moon Lord flag8 trio 24736/24748/24752), 7 witnesses, UNWIRED |
| g13 | task/W4-g13-dust-main-sites-3 | `15fd99c9` | main_dust.rs sites 17-23 WIRED (draw-call feeds + DrawnThisUpdate relocation + item-sparkle 16px light-box fix + retro/Moon-Lord witness fixes), 18 witnesses |
| g61 | task/W4-g61-npc-spawn-dust | `a9628928` | npc_spawn_visuals.rs — widest teleport (11 styles + pylon 9 + NPC driver + per-tick visuals + alt-texture) + spawn family, UNWIRED, 18 witnesses; particles.rs DustRng generalisation |
| e4 | task/W4-e4-dust-sites-world-2 | `2a79551a` → SEAL `c38af30f3` | orchestrator_dust.rs (orch 41-61, 9 witnesses) + mount_dust.rs (Mount 1-19, 10 witnesses); **orch WIRED** (wiring batches 5-8 `ed7fff13e`/`1ac20a4c7`/`95d1a036d`/`e7bb2ff1d` — spawn_orchestra dispatch + pending_orchestra queue/drain + hit switch + post-strike chain + ClassyCane inline + FlameWaders/mount seams; the mount half DROPS to f6 per §8.3); lane complete 40/40 |
| f6 | task/W4-f6-dust-world-3 | `0f9fd642f` (gap-closure re-read; was `5daa1d70`) | mount_dust.rs ALL 27 Mount sites + batch-4 world_dust.rs 13 sites + **the wiring wave 1-6 LANDED** (spawn/dismount channel, frame extras, UpdateEffects, chest/poof, pylon/void-lens, lava bubbles + the borrow fix) — **ALIVE, 3 unpushed local commits** incl `76aeaf3c3` consuming this doc's §7.1 atan2 hunk (§5 row 3 RESOLVED-LOCAL) |
| g63 | task/W4-g63-proj-trail-dust-1 | `b1e78b19` | projectile_trail_dust.rs — rowed inline-machine kinds ≤400 re-armed end-to-end, WIRED via update_dust from world_sim/projectiles.rs, 19 witnesses |
| g64 | task/W4-g64-proj-trail-dust-2 | `77616a7d` | proj_trail_dust.rs — kinds >400 superset, 42 emitters + 16 re-armed call sites on the UpdateProjectiles stream (WIRED), 44 witnesses; particles.rs DustRng generalisation (second copy) |
| e3 | task/W4-e3-dust-sites-proj-2 | `8023aa4d` | projectile_dust.rs — aiStyle-generic window :22297-24318 (41 sites + the 24328 edge pair, all 42 ported), 21 witnesses + 3 keep-one contracts vs d16/d1/f48 (doc sweep tip) |
| d130 | task/W4-d130-weather-ambience | `ec1d73c7` | research/83 EXECUTED (`88cfbf38`): rain+splash union live on d130; snow.rs base kept — **the g11 addendum folds F1-F7 NOT yet applied** (still splitmix/1920/no pause gate at tip) |
| g73 | task/W4-g73-waterfall-draw | `82ae8b96` | waterfall family FINAL UNION (`eee87f98` folds g75 `e30ad7ba`); contract rewritten: canonical = g73 tip, 43-witness inventory — **but g75 pushed 3 commits AFTER the fold base** (§5) |
| g75 | task/W4-g75-honeyfall-draw | `41b64b04` (final-audit re-read; was `2b9ed81f`) | post-`e30ad7ba` closeout, now SEVEN commits: `d118f7ba` disco colour-source fix + `f4e3071f` 717 lava-fountain witnesses + `2b9ed81f` sweep docs + `17161648` retire-contract fold-list + **`a3093a66` the :3858 spout WIRING** (`liquid_behind_spout_refused` — TileDrawing.cs:3858 halfBrick && (num3>160 || num>160) && CheckForWaterfall, the :3835-3844 type-379 exclusion; manager handle through the draw_tiles seam; 3 witnesses) + `d441713d` sound.rs DELEGATED-STATUS (two-tier split: g73 decision tier delegates its volume walk to g75's pure `liquid_ambience` — merger keeps BOTH fns) + `41b64b04` fold-list extension to **5 commits + the FINAL 51-witness inventory** — ALL verified absent from g73 by ancestry (exit 1 each), OWED the g73e re-fold (§5) |

## 2. Per-file keep-one FINAL table

### WorldGen lane (RESOLVED on d54 — research/66 §6 EXECUTED)

| File | Keep-one FINAL | Status |
| ---- | -------------- | ------ |
| `tr-core/worldgen_dust.rs` | **g14** (ash-shake consumes the gore argument draws :29669) | LANDED on d54 (`8f8471cb`), paint color-30 alpha-150 fix `74ba4a2e` |
| `tr-core/worldgen_dust_wall.rs` | **g14** (ladder multiset identical; slot return) | LANDED on d54 |
| `tr-core/worldgen_dust_tile.rs` | **g14** 8-chunk ladder + **d54 arm-11 torch `*255` round-trip** + `edit::solid_tile` OOB-false | LANDED on d54, fixes `74ba4a2e`, wrong-semantics witness rewritten |
| `tr-app/world_dust.rs` (d54) | bodies DROPPED; **7 supplement witnesses** re-pointed (ash oracle adapted to the gore draws) | LANDED `e41a90f0`; LIVE cutter seams call tr-core directly |
| `tr-app/world_dust_tile.rs` (d54) | **`tile_break_burst` wrapper only** (amount = `collision::tile_dust_amount`, kind-first) | LANDED `e41a90f0`; duplicate amount/solid_tile copies dropped |
| `tr-app/world_dust_wall.rs` (d54) | **`wall_break_burst` + `place_tile_torch_dust` wrappers** (caller-side 22/140 gate) | LANDED `e41a90f0` |
| `tr-app/world_dust.rs` (**f6**, batch 4) | **KEEP f6's file** — DISJOINT from d54's supplement-only file in content but SAME filename: chest unlock / Utils.PoofOfSmoke / pylon per-style / void-lens / popup flash / lava bubbles / rain splash / nebula level-up (13 sites) | NOT on d54; merge collision resolve below |
| `collision.rs tile_dust_amount` | main's copy (kind-first) — the ONE table | LANDED; d54's third copy dropped |

**f6-vs-g61-vs-d54 small-file overlap (NEW at f6 `680ac9d8`):** f6's
batch-4 `poof_of_smoke` (Utils.cs:1089) and `pylon_spawn_dust`
(TeleportPylonsSystem.cs:390) DUPLICATE g61's landed `poof_of_smoke` /
`pylon_spawn_in_world_dust` (npc_spawn_visuals.rs). Research/66 §2
already ruled: **g61's bodies win** (its pylon port is the style-9
union body per research/116 §1; its poof carries the gore-arg draws).
f6's `rain_splash` (Rain.cs:122) duplicates d130's EXECUTED splash
union (`spawn_splash`/`water_splash_arm`, research/83 §3) — **d130's
wins** (wired + the impact-gate fix). f6's UNIQUE six (chest unlock,
void-lens, popup flash, lava bubbles' private `_random` stream, nebula
level-up, and its `dust_water` helper if not already on main) keep.
Merge shape: f6's world_dust.rs unique emitters fold into d54's
supplement-witness module (or a sibling `world_dust_small.rs` — the
merger's naming call), the three duplicate bodies drop, f6's 8
witnesses supplement minus the poof/pylon/splash duplicates.

### Main lane

| File | Keep-one FINAL | Status |
| ---- | -------------- | ------ |
| `tr-app/main_dust.rs` | **ONE file = g13's six bodies + feeds (sites 17-23, WIRED) + g11's teleport SEAMS dispatcher** (research/116 §3: `player_teleport`/`npc_teleport` replays, the Entry-OLD→Exit-NEW queue, the `Npc` teleport fields). g11's snowing body RETIRES (research/83: d130 canonical); g11's seven teleport arm bodies RETIRE (g61's body wins) | filename collision g11-vs-g13 unresolved on the branches — merger-wave resolve |
| `tr-app/main_dust_b.rs` (g12) | **KEEP the bone-arm tip puffs + Moon Lord flag8 trio halves**; the three `teleport_effect_case10/11/13` fns RETIRE into g61's body (research/116 §3 — draw-equivalent, g61 canonical hsl + slot-read clone guard) | g12 `665cd1e8` stands; case witnesses supplement |
| `tr-app/npc_spawn_visuals.rs` (g61) | **KEEP whole** — the widest teleport_effect (11 styles incl. pylon 9), poof/mimic/transform, alt-texture transitions; its env-level `npc_teleport` driver RETIRES (g11's App-level replay supersedes, + `other_position` per 116 gap 2) | UNWIRED; wiring per research/116 §3 |
| `tr-app/world_dust_main.rs` (d54) | **DROP both bodies at the main-lane resolve** (snowing → d130 per 83; teleport → g61 per 116); the 9 witnesses supplement (2 snowing → d130 re-pin, 7 teleport + colour oracle → g61 re-pin) | LATENT on d54 today — the DROP is merger-wave (this branch does not carry the surviving files) |
| `tr-app/snow.rs` (d130) | **KEEP as the snowing canonical** (research/83 EXECUTED on d130: wired + real `DustPool.snow_dust` counter + type-76 AI + draw) **+ the g11 addendum folds F1-F7** | F1-F7 OUTSTANDING (§5) |
| `tr-core/particles.rs` DustRng delta | **KEEP one generalisation** — g61 `a9628928` vs g64 `853039bf` diffs are DOC-COMMENT-ONLY (8 lines, verified this round: same trait, same generic signatures, same bodies); either lands, take g61's (first) and g64's comment wording is cosmetic | merge conflict is trivial-comment-level |

### Mount lane

| File | Keep-one FINAL | Status |
| ---- | -------------- | ------ |
| `tr-app/mount_dust.rs` | **f6's superset (ALL 27 sites)** — CONFIRMED from tips: f6 `5daa1d70` carries batches 1-3 (10+10+7) incl. the lava-cart 5099 / pixie 5149 / failed-dismount 6339 / DoSpawnDust wolf-chillet-generic 6382-6493 + FairyQueen colour helper; e4's 19 are a subset (bodies spot-equivalent, research/66 §2). e4's mount-name correction stands (44=PirateShip, 45=SpookyWood — f6's names right) | e4's file DROPS at merge; its 10 witnesses supplement where f6's 24 don't pin the draws |
| f6 `drill_beam_dust` atan2 | **STILL F32 at f6 tip** (`d.y.atan2(d.x)`, verified `5daa1d70`) — the double-atan2 fix (e4's caller-passes-aim shape, `Utils.ToRotation` = `(float)Math.Atan2` DOUBLE) is OUTSTANDING, owed at the mount resolve | one-edit fix + witness re-pin |
| `tr-app/orchestrator_dust.rs` (e4) | **KEEP whole** — orch 41-61 DISJOINT from d54's 1-40 (boundary 2582/2653 verified, research/66 §2); the 272-draw RainbowRodHit pin is the prefix.rs count-only fold authority | **WIRED on e4** (wiring batches 5-8: spawn_orchestra + queue/drain + hit/post-strike/kill/tag/ClassyCane/FlameWaders/mount seams, `ed7fff13e`..`e7bb2ff1d`); the prefix.rs fold LANDED as the reforge keep-one (count-only stays the reforge tail's ONE path — never both, double-draw risk). d54's 1-40 dispatch = `orch_dispatch.rs::spawn_orchestra_d54` (the `_ => {}` arm's d54 half, research/370) — LATENT until the queue-drain wiring |

### Projectile-trail lane (g63 vs g64 vs e3 — three DISJOINT scopes, one seam family)

| File | Keep-one FINAL | Status |
| ---- | -------------- | ------ |
| `tr-app/projectile_trail_dust.rs` (g63) | **KEEP whole** — rowed inline-machine kinds ≤400 (35 emitters incl. the molotov 399 + the no-dust rowed census closure), WIRED via `ProjectilePool::update_dust` from world_sim/projectiles.rs | 19 witnesses |
| `tr-app/proj_trail_dust.rs` (g64) | **KEEP whole** — kinds >400 superset (42 emitters, 16 call sites re-armed on the UpdateProjectiles stream, the 533 frame-table fix, the 509 behaviour-feeding jitter, the 596 bool seam, the 644 prism-hue orbit), WIRED via the same update_dust seam shape | 44 witnesses; the 26 unrowed kinds stay cited (S1-F-9b) |
| `tr-app/projectile_dust.rs` (e3) | **KEEP whole** — the aiStyle-GENERIC window :22297-24318 (styles 2/4/5/6/8/152 arms; 42 sites, zero missed per `8023aa4d` census) — a DIFFERENT axis from g63/g64's per-kind arms; no body overlap | 21 witnesses + the 3 keep-one contracts below |
| g63-vs-g64 kind boundary | g63 ≤400 rowed / g64 >400 — the 387/388/533 twins + 669 flag35 block are g64's (the type-533 gate); g63's 399 arm and g64's 669 Betsy pair are complementary, both keep | verified from the batch docs |
| e3's STAGED 152 emitters | `star_slash_dust`/`star_slash_976` drop-in over **d16 `e122a6b2`**'s count-only skips (3-edit contract, e3 `ab75231f`: thread DustPool+SpawnCtx, REPLACE the Next(8) block — keeping d16's circular pair DOUBLE-DRAWS — REPLACE the Next(3)+localAI[1] blocks, latch AFTER the call) | merger-wave, d16 still count-only at its tip |
| e3's 1051 style6_burst | feed through **d1 `e271b5bc`**'s machine+rows (count-only powder twin), retire e3's ai_style6 duplicate; pick-first order pinned by e3's keep-one witness | merger-wave |
| e3's cloak-five rows (723-726/1133) | ROWED on **f48 `bba610bb`** (`ai_cloak_star`) — f48's rowed arm survives + takes the cloak dust half; **g64's trail_503/723/1133 twins** (proj_trail_dust.rs:765/:1485/:1859, verified at tip) are the unwired counterparts — keep-one at merge per e3's contract; 503/1037/1049 unrowed everywhere, gates stay no-op | merger-wave |

### Waterfall family (falls-adjacent — the dust-feeding liquid loops)

| File | Keep-one FINAL | Status |
| ---- | -------------- | ------ |
| waterfall family (draw.rs/mod.rs/ambient_sound.rs/sound.rs/audio.rs + consumers) | **ONE branch: g73 tip** (`82ae8b96` contract FINAL; `eee87f98` already folds g75 `e30ad7ba` + g74 `cd462c1a`) — g73's `liquid_loop_update` superset (ambientLava Manhattan pick+sum, create-arm-no-pitch, live re-pan) delegates to g75's pure `sound::liquid_ambience`; g74/g75 branches DROP wholesale | **g75's SEVEN post-fold commits OWED a re-fold** (§5, final-audit re-read `41b64b04`): `d118f7ba` removes the manager-local disco cycle g73 still carries (`update_disco_rgb` at mod.rs:300/:303, re-verified at g73 tip) for the shared-statics `set_disco_rgb` latch; `f4e3071f`'s 2 lava-fountain witnesses (the 717 SPAWN arm itself is already in g73's mod.rs:457, re-verified); **NEW `a3093a66` :3858 spout wiring** (`liquid_behind_spout_refused` + the manager-handle parameter/call-site hunks in liquids.rs/tiles.rs + anchor row); `d441713d` DELEGATED-STATUS + doc syncs; `17161648`/`41b64b04` the per-hunk fold-list + 51-witness inventory. g75's `d441713d` records the two-tier keep-BOTH-fns contract (g73 decision tier + g75 pure volume walk) — the merger keeps both, no body contest |

## 3. Merge-order recommendation

Ordered so each step's inputs are already canonical when it runs; no
step re-litigates a resolved decision.

1. **d54 (this branch)** — already carries the WorldGen resolve
   (research/66 §6). Lands first: the tr-core worldgen_dust trio +
   the LIVE editing/wiring seams + the orch 1-40 lane. Everything
   else re-points at these markers.
2. **g61** — npc_spawn_visuals.rs + the particles.rs DustRng
   generalisation (the comment-only g64 twin resolves trivially).
   The teleport body every other Main-lane step needs.
3. **g12** — main_dust_b.rs (drop the 3 staged teleport cases into
   g61's body per 116 §3; keep the bone-arm/Moon-Lord halves + all 7
   witnesses as supplements).
4. **g13 + g11 main_dust.rs resolve** — ONE file: g13's bodies/feeds +
   g11's seams dispatcher calling g61's body (116 §3), the 6 wiring
   gaps fixed (10/11/13 fall-through, other_position, style-9
   extraInfo2, water_style param, Player.Teleport head/tail arms,
   double grants). g11's arm bodies + snowing retire here.
5. **d130** — rain+splash+snow canonical (83 EXECUTED); apply the g11
   addendum folds F1-F7 (§5) in the same step; d54's
   world_dust_main.rs snowing half + g11's Snowflake-retirement notes
   re-point. The `sand.rs`/`rain.rs` 1920-pin deviation family stays
   flagged for their own lanes.
6. **d54 world_dust_main.rs DROP** — the teleport half retires once
   step 4's dispatcher is live; the 9 witnesses re-point (2 → d130,
   7 → g61). Anchors row re-points at the surviving markers (never
   delete without the replacement landing).
7. **e4 + f6 mount resolve** — f6's superset file + the double-atan2
   fix; e4's orchestrator_dust.rs lands alongside (disjoint); the
   prefix.rs RainbowRodHit count-only→emitter fold records here.
8. **f6 batch-4 small files** — the unique six fold per §2 (g61's
   poof/pylon + d130's splash win the three duplicates).
9. **g63 → g64 → e3** — the trail trio in scope order (≤400 rowed,
   >400 rowed, aiStyle-generic); then e3's three keep-one contracts
   execute against d16/d1/f48 (those branches merge AFTER their
   count-only skips are replaced, or the contracts ride their merge
   commits). The g64 unwired cloak twins resolve with f48.
10. **g73 waterfall family** — after re-folding g75's 3 post-fold
    commits (§5); g74/g75 drop wholesale.
11. **Merge-verify wave** — compile + run the witness UNION (§4);
    the anchors canary + `main_anchors` test guard every step.

## 4. Witness-union census (all WRITTEN NOT RUN — code-only lanes)

Counts re-tallied from the fetched tips (`#[test]` grep per file):

| Lane / file | Tip count | Union disposition |
| ----------- | --------- | ----------------- |
| d54 batch-1/2/3 supplements (world_dust{,_wall,_tile}.rs) | 18 (7+5+6) | KEEP, re-pointed at tr-core (landed `e41a90f0`); ash oracle adapted to the gore draws |
| d54 batch-4 (world_dust_main.rs) | 9 | 2 snowing → d130 re-pin (F7), 7 teleport + colour oracle → g61 re-pin (116 §3) |
| d54 orch batches 5-8 | 21 (7+3+5+6) | KEEP whole (sites 1-40) |
| g14 tr-core trio | 25 (24 + the rewritten torch witness counted once + 1 new paint alpha-150) | KEEP (landed on d54 `74ba4a2e`) |
| g11 main_dust.rs | 15 (7 style + 4 snowing + 4 wiring) | 4 snowing → d130 F7 re-pins; 7 style + 4 wiring keep pinning the dispatcher; `the_g12_styles_stay_untouched` INVERTS at the union (116 §3); the pad integration witness (wiring.rs) keeps |
| g12 main_dust_b.rs | 7 | 4 teleport-case witnesses supplement g61 (same oracles); 3 bone-arm/Moon-Lord keep |
| g13 main_dust.rs | 18 (14 body + 4 feed/wiring) | KEEP whole (no overlap) |
| g61 npc_spawn_visuals.rs | 18 | 9 teleport witnesses transfer with rig adaptation (draw-equivalent per 116 §1); 8 spawn-family + alt-texture keep; style-12 gate unconditional (`a9628928`) |
| e4 orchestrator_dust.rs | 9 | KEEP (the 272-draw RainbowRodHit pin is the prefix fold authority) |
| e4 mount_dust.rs | 10 | supplement f6 where unique (basilisk-charge-fold, witch-puff-type31 pins); exact-shape duplicates drop |
| f6 mount_dust.rs | 24 | KEEP base (fairy-color generic-path expectation derived through hsl_to_rgb, `5daa1d70`) |
| f6 world_dust.rs (batch 4) | 8 | KEEP minus the poof/pylon/splash duplicate oracles (g61/d130 bodies win) |
| g63 projectile_trail_dust.rs | 19 | KEEP whole |
| g64 proj_trail_dust.rs | 44 (40 batch + 4 wiring-wave) | KEEP whole (incl. the DropRng/concrete seam parity + the 642/509 stream-parity pair) |
| e3 projectile_dust.rs | 21 (incl. the 1051 pick-first keep-one witness, `ab75231f`) | KEEP whole |
| g73 waterfall family | 43 (contract inventory) | + g75's post-fold 8 (3 disco + 2 lava-fountain + 3 spout, `a3093a66`) = **51** after the re-fold (g75 `41b64b04`'s pinned FINAL inventory supersedes the earlier 48) |

Union total (dust lanes, pre-dedup): **309 witnesses** (the table
sums to 309, + g11's pad integration witness in wiring.rs = 310;
**318** after the g75 post-fold re-fold — final-audit update: the
earlier 315 assumed 5 post-fold witnesses, g75 `41b64b04` pins 8:
3 disco + 2 lava-fountain + 3 spout); the exact
post-dedup tally is the merge-verify wave's census (checklist 6 of
research/66). No cargo/tests invoked this round.

## 5. Outstanding folds (the crosslane ledger)

| # | Fold | Owed by | Detail |
| - | ---- | ------- | ------ |
| 1 | **g73e — the g75 post-fold falls sweep** | g73 (or the family merger) | **FINAL-AUDIT UPDATE: the fold-list is now FIVE commits** (`git log e30ad7ba..origin/task/W4-g75-honeyfall-draw` = 7 commits; `17161648`/`41b64b04` are the fold-list docs themselves). Per g75's own `41b64b04` per-hunk list: `d118f7ba` disco FIX critical (g73's mod.rs:300/:303 still carries the manager-local `update_disco_rgb` cycle, re-verified — out of phase with the app's `advance_disco`, init 0 vs Main.Initialize's DiscoR=255 :66780; replace with g75's `set_disco_rgb` latch fed from params.disco beside `set_show_invisible`, frame/draw.rs) + `f4e3071f` 2 lava-fountain witnesses (717 spawn arm already in g73 mod.rs:457) + `2b9ed81f` sweep docs + **`a3093a66` the :3858 spout wiring** (liquids.rs `liquid_behind_spout_refused` + manager-handle parameter/call-site hunks + tiles.rs call feed + anchor row; composes by OR with g80's `af6cd44f` inActive half — documented, not contested) + `d441713d` sound.rs DELEGATED-STATUS + 4 census-doc syncs (the two-tier keep-BOTH-fns contract). All 7 ancestry-checked ABSENT from g73 tip `82ae8b96` this round. Witness inventory: 43 + 5 sweep + 3 spout = **51** (g75's pinned final count supersedes this doc's earlier 48). |
| 2 | **g11c — the snowing details F1-F7** | d130 (the canonical branch) or the weather merger | research/83's g11 addendum (`d892d8d4`) lists 7 exact folds onto d130's snow.rs: F1 g11's body + the `global` stream (retire the Snow splitmix + App.snow field), F2 MAX_SCREEN_W 4096 HiDef (re-derive every budget expectation), F3 restore the in-body gamePaused gate, F4 counter budget once-per-call (d130's shape — g11's per-attempt live_type census retires; this SUPERSEDES research/66 §6's "g11b hoist" note: the hoist target is d130's counter param, not g11's loop), F5 int attempt-loop shape, F6 env re-point (g11's scene_scan SnowingEnv block → SnowInputs + game_paused), F7 the 4 g11 snow witnesses re-pinned. VERIFIED at d130 tip `ec1d73c7`: snow.rs still pins 1920/splitmix/no pause gate — NONE of F1-F7 applied yet. |
| 3 | **f6 double-atan2 drill fix** | ~~the mount-resolve merger (step 7)~~ **RESOLVED-LOCAL by f6** | **GAP-CLOSURE PROBE (2026-11-21 08:28): f6 ALIVE and the hunk CONSUMED.** f6 advanced `5daa1d70` → origin `0f9fd642f` (+8: wiring 1-6 `bebd0c55f`/`ac0c7edcf`/`65ec64a29`/`bcd63051c`/`7dfcd0981`/`7c9165da7`, wiring docs `ac45e0d5f`, borrow fix `0f9fd642f`) and its LOCAL worktree carries 3 unpushed commits — **`76aeaf3c3` applies this doc's §7.1 EXACT hunks verbatim** ("apply d54 research/131 7.1 EXACT hunks (c89aea9f)"): `mount_dust.rs:88` now `(d.y as f64).atan2(d.x as f64) as f32` + the witness mirror :1849, signature kept. Also local: `8a34f6642` tile_draw_rand g68 keep-one contract + `a67d13638` mount-abilities activation contract (cites the atan2 fix as landed). **Merger note:** take f6's tip AFTER its push (the fix rides `76aeaf3c3`); if f6 stalls unpushed, the §7.1 hunk still applies mechanically at origin `0f9fd642f` — body :85 unchanged, witness mirror moved :1598 → **:1846** (the wiring wave grew the file; re-grep `let num = d.y.atan2(d.x);` — exactly 2 hits). |
| 4 | **research/116's 6 wiring gaps** | the main-lane merger (step 4) | 10/11/13 fall-through on reachable callers (Queen-Slime hook, Shellphone), other_position on the reqs/queue, style-9 pylon extraInfo2, style-5 waterStyle param, Player.Teleport head/tail arms, the double-grant cleanup. |
| 5 | **e3's 3 keep-one contracts** | the proj-lane mergers (step 9) | 152 drop-in vs d16 `e122a6b2`, 1051 burst vs d1 `e271b5bc`, cloak-five vs f48 `bba610bb` + the g64 unwired twins. |
| 6 | **prefix.rs RainbowRodHit fold** | the wiring wave (step 7) | count-only drains → e4's full emitter, never both. |
| 7 | **The witness UNION compile+run** | the merge-verify wave (step 11) | every lane was code-only; ~330 witnesses WRITTEN NOT RUN. |

## 6. This branch's residuals (research/66 checklist-7 audit)

**FINAL-STATUS: ZERO residuals on d54** (final-audit round re-verify,
2026-11-21): all 11 anchors markers grep-verified present at the same
lines as the crosslane round (world_dust.rs:256, world_dust_wall.rs:27,
world_dust_tile.rs:32, world_dust_main.rs:238, orch_dust.rs:99,
orch_dust_lightning.rs:198, orch_dust_boulder.rs:163,
orch_dust_town.rs:419, worldgen_dust.rs:98, worldgen_dust_wall.rs:466,
worldgen_dust_tile.rs:1511); no stale `make_tile_dust`-delegation prose
left in anchors.txt/ANALYSIS.md (grep empty); `git diff
origin/main..HEAD --name-only` = 24 files, ALL d54/g14 task scope (the
8 dust modules + tr-core trio + editing/wiring seams + main.rs/lib.rs
mod rows + docs/anchors). Checklist 1-3/7 EXECUTED, 4-5 cross-doc, 6
merge-verify wave. Nothing owed on d54-owned files.

Re-audited this round on `c792cf37`: all 11 d54/g14 anchors rows
(899-909) carry markers that EXIST in the surviving files (verified
by grep: world_dust.rs:256, world_dust_wall.rs:27,
world_dust_tile.rs:32, world_dust_main.rs:238, orch_dust.rs:99,
orch_dust_lightning.rs:198, orch_dust_boulder.rs:163,
orch_dust_town.rs:419, worldgen_dust.rs:98, worldgen_dust_wall.rs:466,
worldgen_dust_tile.rs:1511). No dropped-body row lacks its
replacement marker. Two stale prose references found and fixed in
this round's residual commit: the anchors orch_dust row's
"delegating make_tile_dust" tail and ANALYSIS.md's tile-sweep
paragraph (:39276) still named the pre-union delegation — both now
name the landed `collision::tile_dust_amount` +
`kill_tile_make_tile_dust` re-point (`e41a90f0`). Checklist items
1-3/7 are EXECUTED on this branch; 4-5 are cross-doc (the decisions
stand, the code lives on the other lanes); 6 is the merge-verify
wave's. No unclaimed checklist item remains on d54-owned files.

## 7. FINAL AUDIT ADDENDUM (final-audit round, 2026-11-21)

### 7.1 f6 drill-atan2 — the EXACT patch hunk (owed fold §5 row 3)

**STATUS (gap-closure probe, 2026-11-21 08:28): CONSUMED by f6 locally**
— f6's unpushed `76aeaf3c3` applies both hunks below verbatim (its
commit message cites this doc + `c89aea9f`); at f6 local HEAD
`a67d13638` the body reads `(d.y as f64).atan2(d.x as f64) as f32` at
:88 and the witness mirror at :1849. The hunk text below is kept as the
mechanical fallback for origin tip `0f9fd642f` (where :85 is still f32
and the mirror sits at :1846, NOT the :1598 quoted below — the wiring
wave moved it). See §5 row 3.

Re-verified at the fetched f6 tip `5daa1d70`:
`crates/tr-app/src/mount_dust.rs::drill_beam_dust` (fn at :75) still
computes the aim in F32 at :85. The C# is
`Mount.cs:3107` `float num = (vector - mountedPlayer.Center).ToRotation();`
and `Utils.ToRotation` is **`Utils.cs:1655-1658`**:

```csharp
public static float ToRotation(this Vector2 v)
{
    return (float)Math.Atan2(v.Y, v.X);
}
```

`Math.Atan2(double, double)` — the floats are WIDENED, atan2 runs in
DOUBLE, the result is cast back to f32. `f32::atan2` is a different
libm path and can differ in the last ulp; the cos/sin at :93-95 then
amplify the angle delta into the velocity bytes. e4's port already
has the correct shape — `orchestrator_dust.rs:74-78` at e4 tip
`2a79551a`:

```rust
/// `Utils.ToRotation` (`Utils.cs:1655-1658`): `Math.Atan2(Y, X)` in
/// double of the widened floats, cast back.
pub(crate) fn to_rotation(v: Vec2) -> f32 {
    (v.y as f64).atan2(v.x as f64) as f32
}
```

**Hunk 1 — the body fix** (`mount_dust.rs:83-85` at f6 `5daa1d70`),
old:

```rust
    // `:3107` — `(vector - mountedPlayer.Center).ToRotation()` (atan2).
    let d = target - player_center;
    let num = d.y.atan2(d.x);
```

new:

```rust
    // `:3107` — `(vector - mountedPlayer.Center).ToRotation()`:
    // Utils.cs:1655-1658 `(float)Math.Atan2(v.Y, v.X)` — Math.Atan2 is
    // DOUBLE of the widened floats, cast back (e4 to_rotation precedent,
    // orchestrator_dust.rs:76-78).
    let d = target - player_center;
    let num = (d.y as f64).atan2(d.x as f64) as f32;
```

**Hunk 2 — the witness re-pin** (the mirror inside
`w4_f6_drill_beam_spawns_two_and_mirrors_the_draw_order`,
`mount_dust.rs:1598`), old:

```rust
        let num = d.y.atan2(d.x);
```

new:

```rust
        let num = (d.y as f64).atan2(d.x as f64) as f32;
```

Two one-line edits (+ the citation comment). The signature KEEPS
`(target, player_center)` — f6's file is the keep-one survivor (§2
Mount lane), so the minimal in-place fix beats reshaping to e4's
caller-passes-aim `drill_sparks(tile_centre, aim, …)`; the second
call-shape witness `w4_f6_culled_spawn_writes_the_sentinel` (:1965)
passes positions only and needs NO edit. No RNG draw order changes
(the atan2 is draw-free); the streams stay aligned. Owner: f6-owner
or the mount-resolve merger (merge-order step 7).

### 7.2 World-side Dust.cs coverage FINAL — the 0-340 type sweep (d50-lane gap-table)

Census basis: C# `Dust.cs` full read (`UpdateDust` :420-2423, spawn
tweaks :214-252, `GetAlpha` :2425-2727, `GetColor` :2729-2772, visual
accessors :2774-2790) vs the fetched d50 tip `2e243a2d`
(`dust_physics.rs` 2154 ln, `dust_table.rs` 658 ln, `particles.rs`
2181 ln) + main's `dust_lights.rs`/`minecart.rs` tick + d130 tip
`ec1d73c7` (`snow.rs`, its `particles.rs`/`minecart.rs` deltas).
Mechanical checks: the C# `this.type` set of GetAlpha (123 types)
diffed against the Rust arm set (only 327 absent — the documented
drop); every `dust.type` condition line of :420-2400 extracted and
matched arm-for-arm against `type_chain`/`pre_move`/`update_dust`.

**Per-column verdict (types 0-340; unnamed types take the generic
paths — gravity :1819, drag :2071, rotation rebuild :2079, fadeIn
:2119, noGravity tail :2335, GetAlpha default — ALL ported):**

| Column | File (owner) | Verdict |
| ------ | ------------ | ------- |
| Spawn-time tweaks `:214-252` | particles.rs `new_dust` (main) | COMPLETE — torch family (incl 269), 127/187, 308, 33/52/266/98-105, 41, 80, 34/35/152 + the 34 wet-probe deactivate; ChildSafety re-type both copies (:194/:445) |
| Pre-move `:468-476` | dust_physics.rs `pre_move` (d50) | COMPLETE — 299-301/305 shrink+lift, 35 lavaBubbles++ |
| Physics chain `:478-2334` | dust_physics.rs `type_chain` (d50) | COMPLETE per-type — every arm present incl. the dead 235 vector3 draws, the 159/164/173/160/162 child-spawn sink, 304 sine/ladder, 43 pre-ramp dark-kill, 27 burn-down, 76/268/270/271, 41/44 clamps, second chain 130-134/278/219-223/226/331/272 with the tail-suppression bool. UNWIRED (the live `minecart.rs::tick` dispatches only 213/268; d130 adds 76+splash) — wiring is the d51-54/e1-e4 wave per the module doc |
| Generic head/tail `:426-447`, `:2335-2422` | particles.rs `update_dust` (main) | COMPLETE — incl. the verbatim dead dCount nibbles + recompute ladder |
| GetAlpha/GetColor/visual accessors | dust_table.rs (d50 = main, unchanged) | COMPLETE — the 2 former drops (157/327) PORTED at `get_alpha_with_globals` in the gap-closure round (G3/G4 below) |
| Light `Lighting.AddLight` (94 sites) | dust_lights.rs (S1-E-5, main; **UNCHANGED on d50 tip** — empty diff vs main) | **42/94 ported — the 52-site remainder is the ONE real port-wide gap** (below) |

**GAP-TABLE for the d50-lane (or the S1-E-5 follow-up it hands to):**

| # | Gap | Types / sites | Detail |
| - | --- | ------------- | ------ |
| G1 | **The 52 second-chain/RNG-adjacent AddLight sites** (all 52 reachable — the former "1 dead" :2275 corrected LIVE in the gap-closure round, G5 row) | 159 `:1061`, 164 `:1099`, 173 `:1137`, 160/162 `:1208/:1212`, 168 `:1241`, 244-247 `:1399`, 43 `:1441`, 15/57/58 `:1470/:1474/:1478`, 110-114 `:1501-1537`, 311/312/313 `:1546/:1555/:1564`, 66 `:1588`, 267 `:1619`, 20/21/231 `:1635/:1640/:1644`, 27/45 `:1692`, 55/56/73/74 `:1706/:1714/:1722/:1731`, 71/72 `:1743`, 35 `:1896`, 67/92 `:1926`, 185 `:1939`, 107 `:1950`, 35-wet `:1994`, 68 `:2004`, 70 `:2013`, 41 `:2041`, 44 `:2069`, 130-134 `:2139/:2143/:2147/:2151`, 278 `:2182`, 219-223 `:2214/:2218/:2222/:2226`, 226 `:2254`, 331 `:2299`, 272 `:2312`; DEAD only at the noLightEmittance producers: 226's customData-Color `:2275` — **CORRECTED in the gap-closure round (G5 row): LIVE via ParticleOrchestrator.cs:1171/:1184/:1199 (no noLightEmittance), reconstructible from `d.color`** | dust_lights.rs's own module doc promises these "land with S1-E-3's chain port" — d50's chain port (`cb07d2b6`) cites each skipped AddLight but did NOT extend dust_lights.rs (verified: identical to main at d50 tip). The physics mutations they interleave with ARE ported, so the feed can be added as post-update reads exactly like the landed 206/174/230 compensation retire note. This is the ONLY type-arm coverage gap port-wide. |
| G2 | **76-arm `Main.snowDust++` merge seam** (`:1747`) | 76 | d50's `type_chain` 76 arm (dust_physics.rs:650-665 at tip `2e243a2d`) cites+skips the increment ("the pool does not carry it") — but d130's particles.rs NOW carries `pool.snow_dust` (field :233, head reset :329 inside `update_dust` per `Dust.cs:428`, fed by d130's `snow::type_76_arm` through the live minecart tick :1069/:1106/:1126). **ONE-EDIT exact hunk published §7.2a below** — the `sand_storm_count` env-delta pattern (NOT a pool write: d50's tr-core cannot see the app pool, and d130's tick already lands the delta post-pass). Cross-ref §5 row 2 fold F4 (the counter is also the F4 hoist target). |
| G3 | GetAlpha type 157 (`:2627-2651`) | 157 | **CLOSED (gap-closure round)** — ported at `dust_table::get_alpha_with_globals` (this branch): `DustDrawGlobals.mouse_text_color` feeds the `(m/100f − 1.6f)` scale/+50/clamp arm verbatim; the f32-floor alpha (m=255 → 94, not 95) pinned by `w4_d54_g3_get_alpha_157_mouse_text_pulse`. The pure `get_alpha` keeps its shape (no global-free caller reaches 157). |
| G4 | GetAlpha type 327 (`:2658-2676`) | 327 | **CLOSED (gap-closure round)** — ported at the same seam: `day_time` gate + `disco_rgb` + THREE draw-pass `NextFloat` draws. VERBATIM QUIRK verified: `(int)(sum) * 255` truncates BEFORE the multiply (sum always < 1) → every channel computes 0, the colour passes through unchanged, but the draws still advance the stream — ported structurally, pinned by `w4_d54_g4_get_alpha_327_night_draws_and_zero_quirk`. Reachable port-wide: tile-718 break → dust 327 (`WorldGen.cs:68516-68518` = `worldgen_dust_tile.rs:1013-1015` on d54). The disco feed precedent is g75's `set_disco_rgb` latch; the draw-pass caller wiring (which `Main.rand` stream) is the draw-seam wave's — the seam takes `impl DustRng`. |
| G5 | customData follower halves (dropped field) | 325, 86-92/286, 240, 259/6/158/135-int, 263/264, 230, 182, 261, 229/228/279, 269, 304, 31-float/NPC, 267, 219-223, 226-Player/Color, 272-Player/NPC | **CENSUSED FINAL (gap-closure round)** — the follower halves stay dropped (the `Dust` record has no customData), each cited at its arm in dust_physics.rs; the 31 `else if (noGravity)` `:1297` reachability flip is ported. **The G1 ":2275 dead" sub-claim is WRONG — corrected:** the full C# `customData =` producer census found FOUR Color-valued producers: Projectile.cs:39844 + StormLightningParticle.cs:215 BOTH set `noLight = (noLightEmittance = true)` (`:39851`/`:222`) so `:2275` never fires THERE, but **ParticleOrchestrator.cs:1171/:1184/:1199 (`SpawnLightningExplosionDust`, 24 type-226 dusts) sets customData = color with NO noLightEmittance** → `:2275` IS LIVE, and d54 ports that producer (`orch_dust_lightning.rs::lightning_explosion_dust`, colour written at :211/:223/:238). **Reconstruction precedent for the S1-E-5 feed:** at EVERY Color producer `customData == dust.color` (assigned from the same colour, never mutated after — CloneDust copies both, Dust.cs:280/:284), so the light feed can read `d.color` directly gated on `!d.no_light_emittance` — no customData field needed for the Color half. The Player-follow halves (`:2265`/`:2239`) stay genuinely dropped (Projectile.cs:68978/:68999 lightning-aura sentry-backpack Player customData). |
| G6 | 235 fadeIn homing non-zero player index | 235 | **CLOSED-EXACT (gap-closure round)** — the full-tree producer census found ONE 235 spawn: `NPC.cs:90977` (`UpdateNPC_SoulDrainDebuff` :90957-90984), `fadeIn = i + 1` (:90980) over the ACTIVE-player loop (:90964-90966); single-player has only index 0 active (`Main.myPlayer = 0`, Main.cs:66214) → num78 = 0 always → d50's `env.player_center` arm is EXACT single-player, not an approximation. The (10,21) else-branch is unreachable single-player (default-ctor Player.Center = position(0,0) + 20/2, 42/2 — Player.cs:1846/:1848 defaults, Entity.Center). The producer itself is UNPORTED (`npc_buffs.rs:619-620` records SoulDrainDebuff has no port system) — when the buff-VFX wave lands it, the 235 arm needs no change. The dead `:817` vector3 two-draw keep is already pinned (`w4_d50_the_235_dead_vector_keeps_its_two_draws`). |

Types 0-340 with NO named arm anywhere (e.g. 0-4, 7-13, 17-19, 22-26,
28, 30, 36-40, 42, 47-51, 53-54, 69, 77-79*, 81-85, 93-97, 108-109,
115-122, 125-126, 128-129, 136-138*, 143-151, 153, 155, 161, 165-167,
171, 176-179, 189-196, 198-203, 207-210, 212, 214-217, 224-225, 227,
232-233, 236-239, 241, 243, 248-253, 256-257, 265, 275-277, 280-283,
285, 287, 289-291, 314-321, 330, 332-340; *79 rotation-exempt,
138/197/180/181/183 GetAlpha-only — all named exceptions checked)
ride the generic paths, which are ported in full. **No physics arm is
missing port-wide; the owed work is G1 (lights) + G2 (counter seam).**

### 7.2a G2 — the ONE-EDIT `snowDust++` hunk (d50×d130 resolve, mechanical)

The physics file (`dust_physics.rs`) is d50's — this branch does not
carry it, so the edit is published here for the merger. **Pattern: the
`sand_storm_count` env-delta** (d50 dust_physics.rs:140-143), NOT a pool
write — d50's tr-core cannot see the app-side pool, and the C# resets
the static at the pass head (`Dust.cs:428`) so a post-pass delta lands
the exact count. Three mechanical pieces, ONE semantic edit (piece 2);
pieces 1/3 are its plumbing:

**Piece 1 — the env field** (`DustPhysicsEnv`, after
`sand_storm_count: &'a mut i32,` at d50 tip :143), add:

```rust
    /// `Main.snowDust++` (76 `:1747`) accumulated as a delta; the caller
    /// lands it on `pool.snow_dust` after the pass (the head reset `:428`
    /// rides `update_dust`, so the post-pass write lands the exact count).
    pub snow_dust: &'a mut i32,
```

**Piece 2 — THE one edit** (the `type_chain` 76 arm, d50 tip
dust_physics.rs:650-653), old:

```rust
    } else if ty == 76 {
        // `:1745-1763` — `Main.snowDust++` (`:1747`) is a Main static, dropped
        // (module docs). The grow, the player-fall lerp + push, the solid damp.
        d.scale += 0.009;
```

new:

```rust
    } else if ty == 76 {
        // `:1745-1763` — `Main.snowDust++` (`:1747`) feeds the env delta
        // (the sand_storm_count pattern; the caller lands pool.snow_dust).
        *env.snow_dust += 1;
        // The grow, the player-fall lerp + push, the solid damp.
        d.scale += 0.009;
```

**Piece 3 — the caller landing** (the d50×d130 tick resolve): d130's
`minecart::tick` already carries the exact shape for its own
`snow::type_76_arm` — `let mut snow_dust = 0;` (:1069) …
`pool.snow_dust = snow_dust;` (:1126). At the resolve the tick keeps ONE
76 dispatcher: **d50's `type_chain` wins the arm body** (it is the
canonical chain port; d130's `snow::type_76_arm` + `type_76_tails`
retire into it — same maths, verified line-for-line this round: grow
0.009, lerp 0.04 fall-drag, noLight push 0.2, solid damp 0.9/0.25, the
second-chain drag/rotation/fade tails), the env construction gains
`snow_dust: &mut snow_dust,` beside `sand_storm_count`, and the existing
:1126 post-pass landing feeds the budget (`Main.cs:13057`) unchanged.
The module-doc drop note (d50 :29-31) retires with piece 2. F4 (§5 row
2) hoists the budget READ of this counter — orthogonal, no conflict.

### 7.2b G5/G6 closure census (gap-closure round, this branch)

Evidence for the corrected rows above (all reads this round, direct
file paths): the full-tree C# `customData =` producer sweep (every
`.cs` under decompiled/Terraria, non-entity assignments filtered) found
exactly FOUR Color-valued writers — Projectile.cs:39844 (`noLight =
(noLightEmittance = true)` at :39851), StormLightningParticle.cs:215
(same double-set at :222), ParticleOrchestrator.cs:1171/:1184/:1199
(`SpawnLightningExplosionDust` — NO emittance gate anywhere in
:1164-1239). d54's port of that producer
(`orch_dust_lightning.rs::lightning_explosion_dust`) writes
`d.color = custom_color` at :211/:223/:238 and never sets
`no_light_emittance` on the 226s — matching the C#, so the S1-E-5 light
feed reconstructs `:2275` as `num83 * d.color.rgb / 255` gated
`!d.no_light_emittance` with NO record field. The 235 sweep
(`NewDust*` + `.type =` + `SelectRandom` forms) found exactly ONE
producer: NPC.cs:90977, `fadeIn = i + 1` at :90980 inside the
active-player loop :90964-90966 — single-player index 0 only, so d50's
`env.player_center` arm is exact; the (10,21) fallback stays as
documented unreachable insurance.

## 8. BIBLE-FINAL (bible-final round, 2026-11-21) — the consolidated record

Three `git fetch origin --prune` passes this round; the lanes moved
between them (g61 pushed its wiring wave, d130 landed the adjudication,
g13/g11/g67/g73/g60 all advanced). Every status below is against the
FINAL fetched tips in §8.1, verified against blobs — not commit
messages. This section SUPERSEDES §1-§5 wherever they disagree (the
earlier sections stay as the historical record).

### 8.1 All dust-lane tips + the per-fold VERIFIED/OWED table

| Lane | Branch suffix | FINAL tip | Δ vs §1 |
| ---- | ------------- | --------- | ------- |
| d50 | dust-type-table | `2e243a2d2` | unchanged |
| d51 | dust-sites-player | `0d0cf7c42` | unchanged (26 sites, 3 batches) |
| d52 | dust-sites-npc | `231b31555` | unchanged (5 batches) |
| d53 | dust-sites-projectile | `6d09fe617` | unchanged (sites 1-40 + wet machine + batch-3 re-census) |
| d54 (this) | dust-sites-world | `1d16ef4ea` + this round | gap-closure tip; bible-final docs land on top |
| e1 | dust-sites-player-2 | `3245d624c` | unchanged (40/40 + fidelity fixes) |
| e2 | dust-sites-npc-2 | `6e304b960` | unchanged (batch 5 + f32-order fixes) |
| e3 | dust-sites-proj-2 | `1d26efa94` | ADVANCED `8023aa4d`→ (+4: research/125 census `300355bba`, 503/1037/1049 rows `80d47a568`, Kill_Bombs 6 wired `7ee870200` + 9 staged `1bf9db051` + clone fix) |
| e4 | dust-sites-world-2 | `2a79551ab` | unchanged |
| f3 | dust-player-3 | `0b11548fb` | pulley+carpet hosts (ZERO dust sites — host census) |
| f4 | dust-npc-3 | `0b7b14e29` | KingSlime trio + Destroyer fade (wave E complete) |
| f5 | dust-proj-3 | `d0619d8e9` | ADVANCED `55dd8f9f5`→ (wave D batches 97-103: the classic-mob gore chain 853/853 COMPLETE — NPC.cs gore, not Projectile sites) |
| f6 | dust-world-3 | `0f9fd642f` origin / `a67d13638` LOCAL | origin unchanged; 3 unpushed local commits incl `76aeaf3c3` (atan2 consumed) |
| g11 | dust-main-sites-1 | `98283df08` | ADVANCED `d892d8d4`→ (+5: union 1-3 `56afc0c01`/`8540a8ad5`/`9a9ac1fd0` = research/116 EXECUTED + F1-F7 exact hunks published; final `cc8c767b0`/`98283df08` = seam-4 folds + snowing DEPRECATED + 116 §5/§6) |
| g12 | dust-main-sites-2 | `f24dd76ac` | ADVANCED `665cd1e8`→ (+8 doc-sweep commits: blood-moon sky/shroom cap/martianLight/DrawGoreBehind/DrawSavings/DrawTileInWater/slime scales — NO new dust sites; main_dust_b.rs unchanged) |
| g13 | dust-main-sites-3 | `4444e1011` | ADVANCED `15fd99c9`→ (+6: DrawCapture cull `1af798478`, sweep docs `5ea44967b`, **52 lights `a234107ac` + census `441f6120f`**, receipt `6a2f68304`, GetAlpha 157/327 `daf7b7804`, **Main-family COMPLETE `4444e1011`**) |
| g14 | dust-worldgen-sites | `a35efec26` | unchanged (folded into d54) |
| g15 | dust-item-sites | `f8f35d799` | unchanged origin (17 WorldItem sites); local worktree ahead on doc-sweep lane 5 (item mechanics, no new dust) |
| g59 | npc-death-effects-1 | `29c13d103` | unchanged (death halves kinds 1-230) |
| g60 | npc-death-effects-2 | `19431e385` | ADVANCED `402b32f27`→ (+1: g62's TRAILCACHE 6-hunk contract CONSUMED byte-identical) |
| g61 | npc-spawn-dust | `b5ff1803d` | ADVANCED `a96289284`→ (+9: the wiring wave seams 1-6 `1eedb9a33`…`8f5450077` + docs `7bed9a1c4` + borrow fixes — NOW PUSHED) |
| g62 | npc-hit-dust | `32af98b44` | unchanged (32 batches, 629-live-site census closed, TRAILCACHE widening) |
| g63 | proj-trail-dust-1 | `2ea3e7110` | ADVANCED `b1e78b19`→ (+7: batches 5-6 `95bc7c012`/`dcf4d75a8`, claim `ed8d4bf5c`, 281 correction `eea7156fe`, batch 7 `395aab688`, activation checklist `036901a5e`, f20b cross-note) |
| g64 | proj-trail-dust-2 | `625c449e0` | ADVANCED `77616a7d`→ (+4 remainders: pets-124 `80644e7f9`, 813 four-phase `5be4452ce`, 596 activated `060d4e36b`, keep-one refresh `625c449e0`) |
| g65 | proj-death-dust | `7f0dc8649` | unchanged (10 batches ≈147 of Kill()'s 399) |
| g66 | player-trail-dust | `7213f3c71` | unchanged origin; local `248f0ff4d` f3 keep-one inputs unpushed |
| g67 | weather-rain-splash | `e777b3e2c` | ADVANCED `30b39fceb`→ (+5 final-phase: fold-monitor/Rain.fn-census/DrawRain/visual-overlap/**research/134 FINAL**) |
| g68 | weather-wind-leaves | `d247a1c9f` | unchanged (wiring 1-4, 28 witnesses) |
| g73 | waterfall-draw | `5d444f15a` | ADVANCED `82ae8b96`→ (+10: the falls CLOSEOUT `ea335744a`…`58b5a833f` + **the g75d re-fold 1-4 `05535ca23`/`7330a21b6`/`7e2ccce18`/`f9cdb54c4` + keep-one sync `5d444f15a`**) |
| g74 | lavafall-draw | `cd462c1ae` | unchanged (ancestor of g73 — verified exit 0) |
| g75 | honeyfall-draw | `41b64b042` | unchanged; CONTENT fully folded into g73 (per-hunk below) |
| d130 | weather-ambience | `97ed44f5d` | ADVANCED `ec1d73c7`→ (+7: the final wave `852507123`…`560a02eac` (F2/F3/F4-verify + ambience + intensity + sandstorm + f39 + docs) then **`099df89e2` F1/F5/F7 EXECUTED + `16a792d56` g67b fold + `97ed44f5d` adjudication docs**) |

**Per-fold status (the owed-list-current):**

| Fold | Owed by (§5 #) | FINAL status | Evidence at the exact tip |
| ---- | -------------- | ------------ | ------------------------- |
| d130e F1 (global stream, splitmix retirement) | 2 | **VERIFIED-EXECUTED** `099df89e2` | d130 `97ed44f5d` snow.rs: free `fn snowing(rng: &mut UnifiedRandom, …)`, no `Snow` struct/splitmix, `scene_scan` passes `app.rngs.global()`; call-chain proof in research/83 §7 (:17238 precedes every SwapRandom) |
| d130e F2 (HiDef 4096) | 2 | **VERIFIED-EXECUTED** `852507123` | `const MAX_SCREEN_W: f32 = 4096.0` + budget witnesses re-derived (195/214.5/165.75/71.5) |
| d130e F3 (in-body pause gate) | 2 | **VERIFIED-EXECUTED** `852507123` | `SnowInputs.game_paused` + first-disjunct check + the pause witness arm |
| d130e F4 (counter once-per-call) | 2 | **VERIFIED (d130's own shape)** | `snow_dust: i32` param read once; g11's per-attempt live_type census retired with the body; the g11b budget-hoist flag CLOSED (g11 `98283df08` research/83 FINAL: "no g11b fix owed") |
| d130e F5 (int attempt loop) | 2 | **VERIFIED-EXECUTED** `099df89e2` | `let mut attempt = 0; while (attempt as f32) < num3` |
| d130e F6 (env re-point) | 2 | **VERIFIED-EXECUTED** `852507123` | `update_scene` feeds `SnowInputs` + `game_paused: app.game_paused` |
| d130e F7 (witness re-pins) | 2 | **VERIFIED-EXECUTED** `099df89e2` | 8 d130 witnesses re-pinned to UnifiedRandom + g11's 3 draw-mirrors ported (snow.rs = 11 tests) |
| g67-134 R1 (IsItRaining consumers) | new | **VERIFIED-EXECUTED** d130 `16a792d56` | `Rain::is_it_raining()` feeds frame/world.rs ×3 + netmodules + weather.rs thunder gate (blob-verified); fountain scanner + Lang keep Main.raining |
| g67-134 R2 (ClearRain on quit) | new | **VERIFIED-EXECUTED** d130 `16a792d56` | `self.rain.clear_rain()` in save_and_quit (blob-verified) |
| g67-134 R3 (DrawDust cull + else-kill) | new | **VERIFIED-EXECUTED** d130 `16a792d56` | `splash_cull_rect`/`dust_in_cull_rect` in draw_splashes AND snow::draw (blob-verified) |
| g67-134 R4 (NewRainForced) | new | **VERIFIED-EXECUTED** d130 `16a792d56` | `new_rain_forced` fixed-slot scan + maxRain sentinel (blob-verified); the Eyebrella CALLER stays R6 |
| g67-134 R5 (skyblock snow threshold) | new | **OWED** (note folded, field not) | d130 rain.rs still reads `SNOW_TILE_NORMAL` unconditionally + the cited gap note; g67's `World.skyblock_low_tiles` + `snow_tile_threshold` arm awaits the f24 coordination |
| g67-134 R6 (Eyebrella caller) | new | **OWED** | no `eyebrella` arm in d130 player.rs (grep empty); g67 `356a1ac0e` owns it; R4 seam ready |
| g67-134 R7 (umbrella held-item) | new | **OWED** | no `umbrella_hold` in d130 player.rs; g67 `01ba6c7a9` owns it (player/render lane, independent) |
| g67 menu +2000 kill arm | new | **VERIFIED-EXECUTED** d130 `16a792d56` | `RainInputs.game_menu` move-first arm (the earlier "folded out" divergence RETIRED) |
| g73e g75d re-fold (5 commits) | 1 | **VERIFIED-EXECUTED** g73 `05535ca23`/`7330a21b6`/`7e2ccce18`/`f9cdb54c4` + sync `5d444f15a` | disco: `set_disco_rgb` latch in waterfall/mod.rs, manager-local cycle GONE (grep); 717: both lava-fountain witnesses present; spout: `liquid_behind_spout_refused` + the manager-handle call in render/liquids.rs; sound.rs DELEGATED-STATUS banner present. NOTE: `git merge-base --is-ancestor 41b64b042 g73` = exit 1 (the fold is per-hunk cherry-picks, not ancestry) — the CONTENT check is the authority; g75 branch DROPS wholesale per the contract |
| g73 falls closeout uniques | 1 | **VERIFIED-EXECUTED** `ea335744a`…`58b5a833f` | frame-table witnesses, quit-stop (`Audio::stop_waterfall_ambience` wired in saveglue), retro `ambient_lava_scan` (tr-core retro_water.rs, 3 witnesses), fountain wire-trip witness (wiring.rs) |
| g11 teleport union (116 §3) | 4 | **VERIFIED-EXECUTED** g11 `56afc0c01`/`8540a8ad5` | teleport_dust.rs = g61's widest-11 body adapted (8 witnesses); App dispatcher + Entry/Exit replays; the 6 wiring gaps closed in code (Queen-Slime 10 hook, Shellphone 11, other_position, pylon extraInfo2, water_style, head/tail + double-grant dedupe); g12's boundary witness INVERTED |
| g11 seam-4 folds (fade PRE-AI + statue-14) | 4 | **VERIFIED-EXECUTED** g11 `cc8c767b0` | fade at the UpdateNPC preamble (:90322 before AI :90423), statue style-14 rides the NpcTeleportReq queue, 2 witnesses |
| g11×g13 main_dust.rs filename union | 4 | **OWED (merger)** — plan FINAL | research/116 §6 per-fn keep table (§8.3); both files still exist on their branches |
| g61 wiring wave seams 1-6 | new | **VERIFIED-EXECUTED + PUSHED** `1eedb9a33`…`b5ff1803d` | bug-net poof, statue/lamp poofs, NPC.Teleport sites + fade driver, Transform tail, key-summon mimic; 12 wiring witnesses; keep-one vs g11: the pad-loop dup resolves keep-g11-queue/drop-g61b-helper (116 §5) |
| f6 double-atan2 | 3 | **RESOLVED-LOCAL, PUSH OWED** | f6 local `a67d13638` blob: `(d.y as f64).atan2(d.x as f64) as f32` ×2 (body + mirror); origin `0f9fd642f` still f32 ×2 — §7.1 fallback hunk stands until the push |
| f6 tile_draw_rand g68 keep-one | new | **VERIFIED-PUBLISHED** f6 local `8a34f6642` | ONE App.tile_draw_rand BOTH consumers (drips-before-pylons order); merge-order pin, no code owed |
| f6 mount-abilities activation contract | new | **VERIFIED-PUBLISHED** f6 local `a67d13638` | the UseDrill seam for the mining-machine lane (Player.cs:28288-28291), signature kept |
| e3 152 drop-in vs d16 | 5 | **OWED** — d16 advanced `e122a6b2`→`d414afbca` (+5 kite/159/161 commits) but `ai_super_star_slash` STILL count-only (grep: no star_slash_dust call) — the 3-edit contract unchanged |
| e3 1051 vs d1 | 5 | **OWED** — d1 `e271b5bc8` unchanged, count-only powder twin (grep: no style6_burst) |
| e3 cloak-five vs f48 + g64 twins | 5 | **OWED** — f48 advanced `bba610bb`→`69d499c6b` (+6 parry-wave commits, ai_cloak_star untouched); g64's keep-one refresh `625c449e0` pins the survivors + 5 parity witnesses (§8.3) |
| prefix.rs RainbowRodHit fold | 6 | **OWED** (wiring wave) — e4 `2a79551ab` unchanged |
| G1 52 lights | §7.2 | **CLOSED** g13 `a234107ac` | dust_lights_b.rs 52-arm collector WIRED via feed_dust_chain_lights in draw.rs; 23 witnesses; 94/94 family (research/69 §7) |
| G2 snowDust++ one-edit | §7.2a | **OWED (d50×d130 resolve)** — d50 `2e243a2d2` unchanged (76 arm still cites+skips); d130's tick landing shape verified unchanged (`let mut snow_dust = 0` … `pool.snow_dust = snow_dust`); the §7.2a hunk applies mechanically |
| G3/G4 GetAlpha 157/327 | §7.2 | **CLOSED ×2 — KEEP-ONE NOW OWED** | d54 `641561c2` (`get_alpha_with_globals`, 2 witnesses) AND g13 `daf7b7804` (`get_alpha_mouse_text`/`get_alpha_disco`, 3 witnesses incl. the tint-census pin) BOTH ported the same arms — g13's probe found the drops unclaimed at its read (d54's closure was mid-flight). Same maths, different seams: merger keeps ONE (§8.3) |
| G5/G6 | §7.2b | **CLOSED** (d54 `c5a0e646`, unchanged) |
| g13 flag-b pre_move 35 strip | new | **OWED (d50 wiring wave)** | d50 `2e243a2d2` pre_move :170-172 still carries the 35 arm (g13 receipt `6a2f68304` re-verified); g13's inline increment canonical |
| g13 flag-c dust_lights decrement retire | new | **OWED (same wave)** | 206/174/230 + 309/261 compensations retire when type_chain wires; dust_lights_b's replays retire arm-by-arm |
| g62 TRAILCACHE 6-hunk adoption | new | **VERIFIED-CONSUMED** g60 `19431e385` | byte-identical widening (grep-verified witness adoption); g62 `32af98b44` unchanged |
| witness UNION compile+run | 7 | **OWED (merge-verify wave)** | every lane code-only; §8.4 census |

### 8.2 Dust.cs FAMILY COMPLETE — the per-family spawn-site census

**Declaration basis:** the C# `Dust.NewDust*`-family counts re-derived
THIS round by direct `Select-String` on the decompiled files (no
junction recursion), cross-checked against the lane censuses
(research/15 caller table, e1's research/64, e3's research/125, g13's
research/68 §2, g62's ANALYSIS 629-census). Family =
`NewDust`+`NewDustPerfect`+`NewDustDirect` (the research/15 convention);
`CloneDust` counted separately where a lane counts it.

| C# family | Total (re-counted) | Owner lanes (FINAL tips) | Remaining unclaimed |
| --------- | ------------------ | ------------------------ | ------------------- |
| **Main.cs — 23 spawn + 1 CloneDust** | 17 ND + 1 NDP + 5 NDD = 23 (+1 Clone :13410 rides g12's case-10) | g11 sites 1-8 (`98283df08`, snowing DEPRECATED→d130, teleport→dispatcher) + g12 sites 9-16 (`f24dd76ac`) + g13 sites 17-23 (`4444e1011`) | **ZERO** — g13 research/68 §2 + the COMPLETE declaration `4444e1011`; the DrawCapture cull (:56735-56744, non-spawn) ported `1af798478`; DrawDust generic draw = the DRAW lane's remainder, cull+else-kill landed d130 `16a792d56` |
| **NPC.cs — 927 (+5 Clone = 932)** | 836 ND + 16 NDP + 75 NDD | AI/misc <50000: 159 → d52 (`231b31555`) + e2 (`6e304b960`) + f4 (`0b7b14e29`) per-kind batches; 50000-69999: 87 → e2 batch 5 + d52; spawn/Transform 70000-81653: 12 → g61 (`b5ff1803d` WIRED); **HitEffect :81654-89834: 629** → g62 live halves (`32af98b44`, "0 uncovered live sites" census) + g59/g60 death halves (`29c13d103`/`19431e385`); >89834: 40 → g61 teleport-visuals + buff-VFX lanes (the 235 soul-drain producer UNPORTED — npc_buffs.rs:619 note, G6) | the buff-VFX producers (>90322 remainder) ride their buff lanes (c16-c18/d71-d74); the 2 CloneDust in HitEffect ride g62's ledger |
| **Player.cs — 215** | 182 ND + 16 NDP + 17 NDD (+4 Clone; e1 counts :24654 in its 40) | d51 26 (`0d0cf7c42`) + e1 40 (`3245d624c`) + g66 trails/wings/boots (`7213f3c71`) + main's recall/mirror 6 + sticky 6 + f48 dodge/parry (`69d499c6b`) + g15 item-use halves + f3 host census (pulley/carpet ZERO sites, `0b11548fb`) | e1's no-host remainder: 36005-36193 SetArmorEffectVisuals (no slot fields), 38873 anniversary death, 39038-39053 KillMe_DustExplosion (armor flags), 43242 rod-of-discord (item lane), 56850/56860/57020 minion rest — each cited with its missing host |
| **Projectile.cs — 1195** | 890 ND + 112 NDP + 193 NDD (+21 Clone +1 QuickDust) | d53 :16851-22297 43 sites incl. the wet 8 (`6d09fe617`); e3 :22297-24328 42 + Kill_Bombs 15 (`1d26efa94` — 6 wired + 9 staged); f5 :24406-26501 43 (`d0619d8e9`); g63 per-kind ≤400 (`2ea3e7110`, batches 1-7 + the AI_001 remainder emitters); g64 kinds ≥401 + the remainder machines (`625c449e0` — pets-124/813/596 now WIRED); g65 Kill() ≈147 (`7f0dc8649`); main's stubs (rocket_kill_census etc.) | **e3 research/125's unclaimed windows, re-verified at the final tips (d54j corrections applied):** Kill() remainder ~~234~~ **CLOSED** — g65's independent re-scan `f6e07bb90`/`c356cb99c`: 406 sites = 399 NewDust + 7 CloneDust; 393 drain-covered (the FINAL census 316 distinct kinds, the 309/317 counts corrected) + 6 other-lane (442-i16/608-d12) + 7 documented-deferred cross-seam/App-frame (S1-P-i58/59/60); **0 unclaimed**; AI() inline **40** minus g64's remainder closures (the pets-124 :35601/:35681 + the 813 :33993/:34005/:34020 window now cited ⇒ ~35 remain, g63/g64 continuation); named machines ~~87+36~~ **79+36** (the named-machine dust wave — unassigned; the 8-site overcount corrected); AI_001 **2** (:56049/:56748, g63 continuation — its batch 7 covered 55/174/176/299/374/376) |
| **WorldGen.cs — 29 (KillTile et al.)** | 29 ND | g14 trio folded into d54 (`a35efec26` → d54 `8f8471cb`/`74ba4a2e`/`e41a90f0`); latent at runtime (isGeneratingOrLoadingWorld returns 6000) but ported + witnessed | **ZERO** |
| **ParticleOrchestrator.cs — 61 sites (+7 Clone = 68)** | 16 ND + 37 NDP + 8 NDD | d54 orch 1-40 (4 modules) + e4 orch 41-61 (`2a79551ab`) — DISJOINT, boundary 2582/2653 | **ZERO** (the prefix.rs RainbowRodHit count-only→emitter fold is wiring, not coverage) |
| **Mount.cs — 27** | 27 | f6 ALL 27 (`0f9fd642f` + local wiring wave); e4's 19 subset DROPS | **ZERO** |
| **WorldItem.cs — 17** | 17 | g15 (`f8f35d799`) — Item.cs itself has ZERO dust sites (g15's census correction) | **ZERO** |
| **TileDrawing.cs — 53** | 50 ND + 3 NDD | g68 wind/leaves/drips (`d247a1c9f`, EmitLiquidDrops WIRED per-draw), d60 tile_draw_particles (22 tests, UNWIRED — the cadence contract g68 wiring-4 verified), g31 grass-pass seam | the d60 wiring wave (per-frame pass move + persistent _rand + DrawnThisUpdate) |
| **Rain.cs — 1 (+ the Main snowing producer)** | 1 ND :122 splash | d130 canonical (`97ed44f5d`): splash union + snow.rs (F1-F7 done) + the g67b residuals R1-R4; g67 (`e777b3e2c`) keeps R5/R6/R7 | R5 skyblock field (f24 coordination), R6 Eyebrella caller, R7 umbrella (g67's own player-lane commits — fold or keep at merge) |
| **WaterfallManager.cs — 1** | 1 ND (the 717 lava-fountain spawn) | g73 (`5d444f15a`) — the falls family FINAL, g74/g75 content folded | **ZERO** |
| **Dust.cs internals (0-340 type sweep)** | 94 AddLight + the physics chain | d50 chain (`2e243a2d2`) + d54 table incl. G3/G4 + g13 lights_b (52) + main dust_lights (42) = **94/94**; generic head/tail on main | G2 one-edit + the flag-b strip + flag-c retire (all d50-wiring-wave, §8.1) |

**FAMILY COMPLETE verdict:** every spawn-site family above is either
fully claimed-and-ported or carries an explicit owner-lane remainder
(the Projectile Kill()/named-machine windows, the Player no-host rows,
the NPC buff-VFX producers, the TileDrawing wiring). NO unowned dust
window remains in Main.cs, WorldGen.cs, ParticleOrchestrator.cs,
Mount.cs, WorldItem.cs, Rain.cs, WaterfallManager.cs or the Dust.cs
internals. The dust-lane family is COMPLETE as a coverage claim; the
owed work is wiring seams + the named-machine/continuation waves with
named owners (§8.1 fold table + §8.3 checklist).

### 8.3 The merger dust-checklist (keep-one symbol list, FINAL)

Ordered per §3 (unchanged except where noted). Each row: the surviving
symbol/file, the retire list, the exact tips.

1. **d54 first** (WorldGen resolve already landed): tr-core
   `worldgen_dust{,_wall,_tile}.rs` + the LIVE editing/wiring seams +
   orch 1-40. Nothing owed.
2. **g61 `b5ff1803d`**: `npc_spawn_visuals.rs` KEEP whole +
   `particles.rs` DustRng generalisation (g64's twin is
   COMMENT-ONLY-different — re-verified this round, take g61's wording)
   + the 6 wiring seams. At the g11 resolve: g61's env-level
   `npc_teleport` driver + pad-loop helper RETIRE into g11's App-level
   replay queue (116 §5 keep-g11-queue); g61's statue-14/fade folds
   already mirrored on g11 `cc8c767b0`.
3. **g12 `f24dd76ac`**: `main_dust_b.rs` bone-arm puffs + Moon-Lord
   flag8 trio KEEP; `teleport_effect_case10/11/13` RETIRE (g61 body via
   g11 dispatcher); the doc-sweep additions (sky/savings/gore-behind)
   are disjoint files, no dust conflict.
4. **main_dust.rs THREE-WAY resolve (g11 `98283df08` × g13 `4444e1011`
   × d54)** — ONE file per research/116 §6's per-fn table:
   KEEP g11: `teleport_effect` dispatcher + `CellRng`/`EffectRect` +
   `player_teleport`/`npc_teleport` + the Req types;
   KEEP g13: `to_rotation`/`math_sign`/`tentacle_tip_position`/
   `moon_lord_tentacle_tip_puff`, `AlePlayerView`/`tavernkeep_ale_sparkle`,
   `world_item_sparkle`/`light_color_bytes`/`item_sparkle_feed`,
   `liquid_surface_sparkle`/`lava_bubble_pair`/`retro_surface_rect`/
   `retro_liquid_dust_feed`, the two feeds (Moon-Lord feed FUSES with
   g12's `moon_lord_core_dust` — ONE caller pass), `capture_snow_cull`;
   DROP: g11 `snowing`+`SnowingEnv` (DEPRECATED in-code already), d54
   `world_dust_main.rs` both bodies (9 witnesses re-point: 2→d130,
   7→g61); g13 test helpers RENAME `rig`→`pool_rig`/`live`→`pool_live`;
   ONE `mod main_dust;` (g11's spot). `teleport_dust.rs` (g11) = the
   g61-adapted body, KEEP. g13's `dust_lights_b.rs` KEEP + the draw.rs
   feed (G1 closed).
5. **d130 `97ed44f5d`**: `snow.rs` canonical (F1-F7 ALL EXECUTED —
   free fn on the global stream) + `rain.rs` splash union + the g67b
   residuals. g67 `e777b3e2c`: R5/R6/R7 fold or keep g67's player-lane
   bodies at the player-file resolve (they are disjoint from d130's
   files); the `spawn_acc` cadence + compacted-Vec storage NOT taken
   (d130's shapes canonical). **G2 landing (§7.2a) executes at the
   d50×d130 tick resolve**: d50 `type_chain` wins the 76 arm, the env
   gains `snow_dust`, d130's :1069/:1126 landing feeds it;
   `snow::type_76_arm`/`type_76_tails` retire.
6. **d54 `world_dust_main.rs` DROP** (with step 4).
7. **mount resolve**: f6 tip AFTER its push (local `a67d13638` carries
   the atan2 fix `76aeaf3c3`); fallback = §7.1 hunk at origin
   `0f9fd642f` (:85 body, :1846 mirror). e4 `mount_dust.rs` DROPS (10
   witnesses supplement); e4 `orchestrator_dust.rs` KEEPS (disjoint);
   prefix.rs RainbowRodHit fold rides here. `App.tile_draw_rand` ONE
   field both consumers (f6 `8a34f6642` contract, drips-before-pylons).
8. **f6 batch-4 small files**: unique six fold; g61's poof/pylon +
   d130's splash win the three duplicates (unchanged).
9. **trail trio g63 `2ea3e7110` → g64 `625c449e0` → e3 `1d26efa94`**:
   all three files KEEP (disjoint axes). NEW keep-ones from g64's
   remainder wave (its ANALYSIS table is the authority):
   ONE `ai_pets_style124` = g64's dust-wired port (d13's gates-only
   retires, `tentacle_pointer_pose` joins); ONE `ai_vortex_ritual` =
   g64's (d11/d13 census arms retire, witnesses RE-DERIVE); 596: d11's
   `ai_nebula_sphere` keeps the 575, the 596 half takes g64's
   `trail_596` bool seam (ONE machine either way); 497/504/503 → e3's
   arms survive, g64's twins retire (parity witnesses pin); 723/1133 →
   f48's `ai_cloak_star` survives + takes the dust half, e3+g64 twins
   retire. e3's contracts: 152 3-edit drop-in over d16 `d414afbca`
   (still count-only), 1051 `style6_burst` over d1 `e271b5bc8` (still
   count-only), the 503/1037/1049 rows e3 landed (`80d47a568`) make
   g64's trail_503 wireable — same keep-one. **projectile_dust.rs
   FILENAME collision d53×e3**: fn-name sets are DISJOINT (verified
   this round — only the test helper `ctx` shared); union into one
   file or rename e3's module (d53's ANALYSIS note) — merger's naming
   call, no body contest. g63's activation checklist (research/67
   `036901a5e`) lists which merge unlocks which UNWIRED emitter
   (12←d15, 87←d2, 298/305/358←d6, 55←d22, 299←c3, 357←f20 slice D,
   281←row+d5+call).
10. **g73 `5d444f15a` waterfall family**: TAKE THIS BRANCH ONLY —
    g74 `cd462c1ae` + g75 `41b64b042` DROP wholesale (g74 ancestry-in
    exit 0; g75 content folded per-hunk, ancestry exit 1 is EXPECTED
    for cherry-pick folds — the blob checks in §8.1 are the authority).
    Two-tier sound contract: keep BOTH `ambient_sound::liquid_loop_update`
    (decision) + `sound::liquid_ambience` (volume). The g80 spout halves
    compose by OR.
11. **dust_table.rs G3/G4 keep-one (NEW)**: d54's
    `get_alpha_with_globals` (globals struct seam, 2 witnesses) vs
    g13's `get_alpha_mouse_text`+`get_alpha_disco` (split seams, 3
    witnesses incl. the 8-consumer tint census). Same verbatim maths
    (both pin m=255→94 + the 327 zero-quirk). Recommended: keep ONE
    seam shape — g13's split fns are the purer shape (no struct), d54's
    witnesses re-pin trivially; either way the loser's fns DROP and
    its witnesses supplement. The draw-pass wiring contract is g13
    research/70 §3 (Main.cs:38973 between the gore draw and the
    DrawnThisUpdate set).
12. **d50 wiring wave**: `type_chain` into the `type_arm` hook +
    the flag-b strip (pre_move 35 arm) + the flag-c decrement retire
    (dust_lights 206/174/230+309/261 and dust_lights_b's replays
    arm-by-arm) + the 299-305 shrink/lift the skeleton lacks.
13. **Merge-verify wave**: compile + run the witness union (§8.4);
    anchors canary + main_anchors guard every step.

### 8.4 Witness-union recount at the FINAL tips (supersedes §4's 318)

All counts = `#[test]` greps of the exact tip blobs (this round),
delta-vs-main where the file pre-exists. All WRITTEN NOT RUN.

| Lane / file | Tip | Count | Disposition |
| ----------- | --- | ----- | ----------- |
| d54 supplements world_dust{,_wall,_tile} | `1d16ef4ea` | 18 (7+5+6) | KEEP |
| d54 world_dust_main | `1d16ef4ea` | 9 | re-point (2→d130, 7→g61) |
| d54 orch ×4 | `1d16ef4ea` | 21 (7+3+5+6) | KEEP |
| d54 dust_table G3/G4 | `1d16ef4ea` | +2 | keep-one vs g13 (§8.3.11) — count once |
| g14 trio (on d54) | `74ba4a2e` | 25 (8+7+10) | KEEP |
| g11 main_dust | `98283df08` | 30 | 26 survive (4 snow DROP→d130 F7 already re-pinned there) |
| g11 teleport_dust | `98283df08` | 8 | KEEP |
| g11 wiring/npcs | `98283df08` | 2 (pad + fade-pre-AI) | KEEP |
| g12 main_dust_b | `f24dd76ac` | 7 | 4 supplement g61, 3 KEEP |
| g13 main_dust | `4444e1011` | 20 | KEEP (incl. the 2 cull witnesses) |
| g13 dust_lights_b | `4444e1011` | 23 | KEEP (G1) |
| g13 dust_table + particles | `4444e1011` | 3+1 | 3 = the G3/G4 keep-one pool (+2 d54 = 5 gross, 3 net if g13's shape wins); 1 lava-bubbles KEEP |
| g61 npc_spawn_visuals | `b5ff1803d` | 18 | KEEP (9 teleport transfer-adapted already on g11) |
| g61 wiring wave (bug_net/chest/npcs/wiring×2) | `b5ff1803d` | 12 (1+6+1+3+1) | KEEP minus the pad-helper dup at the g11 resolve |
| e4 orchestrator_dust | `2a79551ab` | 9 | KEEP |
| e4 mount_dust | `2a79551ab` | 10 | supplement f6 |
| f6 mount_dust | `0f9fd642f` | 28 (24+4 wiring) | KEEP base |
| f6 world_dust + pylons + liquid_draw | `0f9fd642f` | 8+1+1 | KEEP minus poof/pylon/splash dup oracles |
| g63 projectile_trail_dust | `2ea3e7110` | 50 (19+11+14+6) | KEEP |
| g64 proj_trail_dust | `625c449e0` | 45 (+45 vs main; was 44 at `77616a7d`) | KEEP minus the 5 retired twins' bodies (witnesses stay as parity pins) |
| g64 projectile.rs + pets124 | `625c449e0` | 10+6 | KEEP (wiring + machine witnesses) |
| e3 projectile_dust + data.rs | `1d26efa94` | 27+1 | KEEP |
| d130 snow | `97ed44f5d` | 11 | KEEP (F7 re-pins incl. g11's 3 mirrors) |
| d130 rain (g67 union + g67b) | `97ed44f5d` | +22 | KEEP |
| d130 weather/wind/audio | `97ed44f5d` | 5+1+4 | KEEP |
| g73 family (mod 36 + draw 2 + ambient 5 + sound 5 + audio 2 + retro 3 + liquids 2 + tiles 1 + frame/draw 1 + wiring 1) | `5d444f15a` | **58** (43+7+8 per the contract's pinned inventory — grep-sum matches) | KEEP; g74/g75 drop |

**Union total (the §4 scope — the crosslane dust lanes):**
the table re-summed at the final tips:
18+9+21+2+25 (d54 75) + 26+8+2 (g11 36) + 7 (g12) + 20+23+3+1 (g13 47)
+ 18+12 (g61 30) + 9+10 (e4 19) + 28+8+1+1 (f6 38) + 50 (g63)
+ 45+10+6 (g64 61) + 27+1 (e3 28) + 11+22+5+1+4 (d130 43) + 58 (g73)
= **492 gross lane-witnesses**. Dedup estimate (the keep-one drops):
d54's 2 G3/G4 vs g13's 3 → −2; g61's 9 teleport witnesses superseded
by g11's union-adapted mirrors → −9; g12's 4 case witnesses superseded
by g11's `w4_g11u_g12_*` → −4; d54 world_dust_main's 7 teleport oracles
superseded by g11's `w4_g11u_d54_*` (the 2 snowing already re-pinned on
d130) → −7; e4's ~6 exact-shape mount dups → −6; f6's 3 poof/pylon/
splash dup oracles → −3 ⇒ **≈461 net** in the §4 scope. (g64's 5
retired-twin bodies keep their parity witnesses — no deduction.)

**The 318→recount bridge (exact):** 318
+ g73 51→58 (+7) + g63 19→50 (+31) + g64 44→61 (+17)
+ e3 21→28 (+7) + g11 16→36 (+20) + g13 18→47 (+29)
+ f6 32→38 (+6) + g61 18→30 (+12) + d54 +2 (G3/G4)
+ d130 joins the scope (+43: snow 11 + rain 22 + weather/wind/audio 10)
= **492 gross**, matching the table sum. **The 318 figure is
SUPERSEDED**; the merge-verify wave's compile+run is still the exact
post-dedup authority (checklist 13).

**Outside the §4 scope (the site-batch lanes, for the family-wide
union):** d51 24, e1 25, g66 76 (7+13+24+11+21), f3 50 (27+23),
d52 15 (5+4+5+1), e2 30 (11+2+2+15), f4 26 (2+3+3+1+10+7),
g62 66 (61+5), g59 53, g60 23, g65 26, d53 29 (20+9), f5 21 (+142
wave-D gore witnesses — the gore lane's), g15 20, g68 28. These ride
their own merges; the dust-family compile wave runs the whole tree.

## Verification

Code-only per the task brief: no cargo/tests/rustfmt/graphify. Every
tip row was read via `git fetch` + `git log`/`git show` this round;
the witness counts are `#[test]` greps of the tip blobs; the
particles.rs g61-vs-g64 delta was diffed in full (8 comment lines);
the f6 atan2, d130 snow.rs pin state, g73 disco-cycle state and the
g75 post-fold commit list were verified against the fetched blobs,
not commit messages alone.

**Final-audit round (2026-11-21, this addendum):** tips re-fetched —
e4 `2a79551a`, f6 `5daa1d70`, d130 `ec1d73c7`, g73 `82ae8b96`
UNCHANGED (verdicts stand: f6 :85 still `d.y.atan2(d.x)` f32; d130
snow.rs still splitmix/1920/no pause gate — F1-F7 unapplied; g73
mod.rs:300/:303 still `update_disco_rgb`, :457 still the 717 arm);
g75 ADVANCED to `41b64b04` (+4 commits, all 7 post-fold commits
ancestry-checked absent from g73, exit 1 each). The §7.1 hunk quotes
are verbatim from the f6/e4 tip blobs + Utils.cs:1655-1658 read
in-tree; the §7.2 census matched every extracted `dust.type`
condition line of Dust.cs:420-2400 against the d50 tip's
`type_chain`/`pre_move` and diffed the GetAlpha type sets mechanically
(123 C# types, only 327 absent from the Rust arms); dust_lights.rs
d50-vs-main diff EMPTY (the G1 gap evidence); d130's particles.rs
`snow_dust` field + minecart tick feed read at tip (the G2 seam).
d54 anchors: 11/11 markers grep-present; branch diff 24 files, all
task scope. No cargo/tests/rustfmt/graphify invoked.

**Gap-closure round (2026-11-21, this addendum):** G3/G4 — the
157/327 C# arms re-read verbatim (Dust.cs:2627-2676), the f32-floor
expectations computed with f32-emulated arithmetic before pinning
(m=255 → alpha 94), the 327 `(int)(sum)*255` operator precedence
verified against the C# text (cast binds to the sum, max sum 0.45875
< 1); `dust_table.rs` blob-swept across ALL remote branches (only e5
differs — a one-line `lerp_color` pub, orthogonal). G5 — full-tree
`customData =` producer census (every .cs, direct paths, no junction
recursion); the four Color writers read with their noLightEmittance
context; d54's `lightning_explosion_dust` re-read (no emittance set on
the 226s). G6 — full-tree 235-producer census (three grep forms);
NPC.cs:90957-90984 read whole; Player default 20×42 (Player.cs:1846/
:1848) + Entity.Center + Main.myPlayer=0 (:66214) verified. G2 — d50
tip 76 arm (:650-665) + env tail (:143) and d130 tip tick
(:1069/:1106/:1126) + particles.rs (:233/:329) re-read; the two 76
bodies diffed line-for-line (identical maths). f6 probe — origin
re-fetched TWICE (the lane moved mid-round: `ac45e0d5f` →
`0f9fd642f`), local worktree HEAD `a67d13638` read via `git -C`
(3 unpushed commits, `76aeaf3c3` = the §7.1 hunk consumed, verified in
the blob at :88/:1849); origin :85/:1846 still f32 (the fallback hunk
targets). 2 new witnesses WRITTEN NOT RUN (code-only). Commits:
`641561c2` (G3/G4), `c5a0e646` (G5/G6), `84eb739b` (G2 hunk), this
commit (f6 probe + header). No cargo/tests/rustfmt/graphify invoked.

**Bible-final round (2026-11-21, §8):** THREE `git fetch origin
--prune` passes (the lanes moved between passes 1→2→3: g61's wiring
wave pushed, d130 landed `099df89e2`/`16a792d56`/`97ed44f5d`, g11
`98283df08`, g13 `4444e1011`, g67 `e777b3e2c`, g60 `19431e385`, f5
`d0619d8e9`); every §8.1 tip is from the FINAL pass. Fold verdicts
blob-verified, not message-trusted: d130 snow.rs (free fn /
UnifiedRandom / int loop / 4096 / game_paused / counter param — F1-F7
all present), d130 rain.rs (`is_it_raining`/`clear_rain`/
`new_rain_forced`/`splash_cull_rect`/`dust_in_cull_rect` present; the
skyblock threshold still `SNOW_TILE_NORMAL` — R5 note-only), d130
saveglue (`clear_rain()` call), d130 player.rs (eyebrella/umbrella
greps EMPTY — R6/R7 owed), g73 waterfall/mod.rs (`set_disco_rgb`
present, manager-local cycle absent, both 717 witnesses), g73
render/liquids.rs (`liquid_behind_spout_refused` + call), g73 sound.rs
(DELEGATED banner), g73 wiring.rs (fountain-trip witness), g73
saveglue (`stop_waterfall_ambience` ×2); g75-tip ancestry in g73
exit 1 (cherry-pick fold — content is the authority), g74/e30ad7ba
ancestry exit 0. f6 local `a67d13638` atan2 blob re-read (double ×2),
origin `0f9fd642f` still f32 ×2. d16 `d414afbca` / d1 `e271b5bc8` /
f48 `69d499c6b` re-probed for the e3 contracts (all still count-only /
cloak-owner — contracts unchanged). The §8.2 C# family counts are
fresh `Select-String` censuses of the decompiled files (direct paths,
no junction recursion): Main 23+1, NPC 927+5 (HitEffect window 629+2),
Player 215+4, Projectile 1195+21+1 (Kill() 399, Kill_Bombs 15),
WorldGen 29, Orchestrator 61+7, Mount 27, WorldItem 17, TileDrawing
53, Rain 1, WaterfallManager 1, Dust.cs AddLight 94. The e3-125
40-site inline list re-verified line-by-line against the actual
NewDust lines (6 of the listed line numbers are loop-body lines, the
8 real sites in :34507-34588 enumerated); g64's remainder closures
(:33993/:34005/:34020/:35601/:35681 + the 813 window) confirmed in its
blob. The §8.4 witness counts are `#[test]` greps / delta-vs-main per
file at the exact tips; the 492 gross sum machine-checked. The
d53×e3 projectile_dust.rs fn-name sets diffed (disjoint; only the test
helper `ctx` shared). No cargo/tests/rustfmt/graphify invoked.

## Addendum — d54j dispatch-router round (§8.2 corrections, 2026-11-21)

Code-only round on `task/W4-d54-dust-sites-world`: the d54 orchestrator
dispatch router `orch_dispatch.rs::spawn_orchestra_d54` landed (the
`SpawnParticlesDirect` port for the d54j catalog's 24 routed arms / 28
kind bytes — `ParticleOrchestrator.cs:162-416`, the `_ => {}` arm of
e4's `spawn_orchestra`; full route table + contracts in research/370 —
registry-150 §9 assignment, the 355 provisional self-pick renumbered
355→370 at this landing, d11 keeps 355). Three §8.2/§2 corrections
applied in place above:

1. **Projectile named machines 87+36 → 79+36** (§8.2 row): the 87 was an
   8-site overcount of the named-machine window; the corrected count is
   79+36, still the unassigned named-machine dust wave.
2. **Kill() remainder 234 → CLOSED** (§8.2 row): g65's independent
   re-scan (`f6e07bb90` keep-one receipt, `c356cb99c` FINAL seal,
   renumbered bible 346 per registry s8.7) closed the whole Kill span —
   406 sites = 399 NewDust + 7 CloneDust; 393 drain-covered (316 distinct
   kinds FINAL; the 309 batch-28 census and 317 +1 miscount corrected),
   6 other-lane (442-i16/608-d12), 7 documented-deferred
   (S1-P-i58/59/60); 0 unclaimed. The §5 g65 probe row (`7f0dc8649`,
   ≈147 of 399) is a HISTORICAL tip record, left as-was.
3. **e4 UNWIRED → WIRED** (§2 lane table + §2 orch file row): e4's
   wiring batches 5-8 (`ed7fff13e` dispatch + reforge keep-one,
   `1ac20a4c7` projectile queue/drain + hit/post-strike/kill/tag seams,
   `95d1a036d` ClassyCane inline, `e7bb2ff1d` mount/FlameWaders) landed
   through SEAL `c38af30f3`. The mount half still DROPS to f6 (§8.3
   step 7); the reforge tail keeps prefix.rs count-only as its ONE path.
   d54's own 1-40 dispatch is LATENT (the router body + 7 witnesses
   WRITTEN NOT RUN) until the queue-drain wiring folds it into the same
   `SpawnParticlesDirect` port — never both routers for one kind.

### 8.5 The Orch-family declaration (d54j round)

**61/61 ParticleOrchestrator dust sites PORTED** (the §8.2 census row's
61 ND/NDP/NDD sites, +7 CloneDust siblings riding their parents' arms):
**e4's 15 kinds WIRED** (orch sites 41-61, `orchestrator_dust.rs` —
`spawn_orchestra` dispatch + the queue/drain/hit/post-strike seams,
wiring batches 5-8 through SEAL `c38af30f3`; its mount half drops to f6)
+ **d54's 24 routed arms / 28 kind bytes in the router**
(`orch_dispatch.rs::spawn_orchestra_d54`, the `_ => {}` arm's d54 half,
this round `97c8c5180`, research/370 — LATENT until the queue-drain
wiring; the bodies were the four batch modules 5-8, sites 1-40). The
**renderer/net remainders stay seeded, NOT ported**: the
`StormLightningParticle`/`LightningGenerator` bolt generation (the
`:1038` EndPosition seam — caller-resolved), the FadingParticle /
PrettySparkle / RandomizedFrame visual requests (RNG-free; their
ARGUMENT draws are consumed in order at each arm), and the
`netmodules.rs:447` Particles-carrier drain (the net lane's wiring
step). No orchestrator dust site is unclaimed.