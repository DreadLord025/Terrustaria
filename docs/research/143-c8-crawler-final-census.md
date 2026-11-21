# 143 — W4-c8 crawler FINAL census: the cross-lane provider contract, the gold set, the d22 compose, the sheets union

Branch: `task/W4-c8-style66-crawling-critters` (final phase, 2026-11-21,
code-only — no cargo/tests/rustfmt run; every witness WRITTEN NOT RUN).
Base: `3705670d3` (the sweep docs commit). Final-phase commits:
`af2a06880` (item 1+2), `72e05facf` (item 3), `b7fd55a43` (item 4),
this doc (item 5).

## 1. The SIGHT-unlock provider is GENERIC — the other critter lanes are auto-covered

`scan_world_for_critter_sights` (`bestiary.rs`, run at the `update_npcs`
head — `Main.cs:17914` before the `UpdateNPC` loop) never had a kind
list: its gate is the live `max_hp <= 5 && contact_damage() == 0` (the
`CountsAsACritter` stand-in, `NPC.cs:6887-6897`). The FINAL phase
generalized the **UI dispatch** to match: `is_critter_entry` now reads
the new `counts_as_a_critter(kind)` (the kind's `SetDefaults` rows +
the `cs_id != 594/686` exclusions) instead of the nine-kind match.

**The contract for the other lanes**: a critter kind rowed by any lane
is scanned AND unlocked with **zero provider wiring** the moment its
`SetDefaults` rows land, because every lane's kinds are 5-HP/0-dmg in
the C#:

| lane | kinds (net ids) | C# rows (lifeMax 5 / damage 0) | auto-covered? |
|---|---|---|---|
| c7-fireflies | Firefly 355, LightningBug 358, Lavafly 654, Shimmerfly 677 | `NPC.cs` SetDefaults rows (the c7 branch's data) | YES — scan + critter provider |
| c9-snails | Snail 359, GlowingSnail 360, MagmaSnail 655 (+677 shared) | same shape | YES |
| c10-waterfowl | Duck 362/363, DuckWhite 364/365, Seagull 602/603, Grebe 608/609 | `NPC.cs:13613-13655` (the duck block, verbatim 5/0) | YES |
| e26-ladybugs | Frog 361, LadyBug 604, GoldLadyBug 605 | `NPC.cs:13601-13612` (frog), `:16763+` (ladybugs) | YES — 605 additionally rides the GOLD arm (§2) |
| c4-fish | Goldfish 55, GoldGoldfish 592, GoldGoldfishWalker 593 | `NPC.cs:9460-9473` (55) | YES — 592 the GOLD arm, 593 the plain critter provider |

Two deltas the generalization landed on THIS branch (the only ported
kinds the raw gate newly reaches):

- **Goldfish 55**: the sweep's recorded c4 follow-up is CLOSED — its C#
  provider IS `CritterUICollectionInfoProvider` (the populator's
  automated `CountsAsACritter` registration,
  `BestiaryDatabaseNPCsPopulator.cs:851-868`), so the port entry now
  unlocks on sight, not kills. The c4 lane's own `unlock_state` (kill)
  call sites are its branch's code — at merge the world-aware dispatch
  here covers them (the c4 UI rows read `unlock_state(entry, tracker)`
  on their branch; the merge reconciliation points them at
  `unlock_state_world`, the same fold the c8 UI already did).
- **DD2LanePortal 549**: raw gate passes (lifeMax 5 / damage 0,
  `NPC.cs:16035-16037`) but `Hide = true` (`NPCID.cs:4910-4915`) — the
  populator's `GetExclusions` (`:870-886`) never registers an entry, so
  the port's fold row keeps the kill provider (the ForceBubble
  precedent). `is_critter_entry` carries the one exclusion.

The 594 WindyBalloon / 686 BoundTownSlimePurple exclusions are in the
gate as `cs_id` tests (both unported kinds today — the e26/luck lane's
686-adjacent town slimes must keep them when rowed).

**Cross-note to c7/c9/c10/e26/c4**: do NOT add per-lane sight scans or
provider arms. Row the kinds (`max_hp`/`contact_damage`/`cs_id` +
`ENTRIES`), and the scan + dispatch cover them. The witness
`w4_c8_final_the_sight_scan_covers_the_other_lanes_critters`
(`world_sim/npcs.rs`) pins the Goldfish as the stand-in for every
lane's shape.

## 2. The GoldCritter set census — SET-COMPLETE

C# `NPCID.Sets.GoldCrittersCollection` (`NPCID.cs:10121-10125`):
**14 members** — 443, 442, 592, 593, 444, 601, 445, 446, 605, 447, 627,
613, 448, 539. The port's `GOLD_COLLECTION` (the gate inside
`gold_critter_unlock_state`) matches verbatim, credit ids =
`NPCID.Search.GetName` (`ContentSamples.cs:868-870`). The
`IsGoldCritter` bool set (`:10126`) is the SAME 14 ids — the sparkle
arm's family (only 448 ported here; d40's gold band extends it).

The **provider registrations** are 13, NOT 14
(`BestiaryDatabaseNPCsPopulator.cs:703-716`): 593 GoldGoldfishWalker is
collection-only (its own entry — `NPCID.cs:5227` no Hide — rides the
plain critter provider). The FINAL phase replaced the hardcoded
448→{357} pair with the full `GOLD_PEER_TABLE`:

| gold id | peers | lane |
|---|---|---|
| 443 GoldBunny | {46 Bunny} | d40 |
| 442 GoldBird | {74 Bird} | d40 |
| 592 GoldGoldfish | {55 Goldfish} | c4 |
| 444 GoldButterfly | {356 Butterfly} | d40 |
| 601 GoldDragonfly | {599 RedDragonfly} | unported (d40 forecast) |
| 445 GoldFrog | {361 Frog} | d40 + e26 (peer) |
| 446 GoldGrasshopper | {377 Grasshopper} | d40 |
| 605 GoldLadyBug | {604 LadyBug} | e26 |
| 447 GoldMouse | {300 Mouse} | d40 |
| 627 GoldSeahorse | {626 Seahorse} | unported |
| 613 GoldWaterStrider | {612 WaterStrider} | unported |
| 448 GoldWorm | {357 Worm} | c8 (ported, live) |
| 539 SquirrelGold | {299 Squirrel, 538 SquirrelRed} | unported — the ONE two-peer row |

`unlock_state_world` dispatches on `net_id ∈ GOLD_PEER_TABLE` — a
merged lane's gold row needs no dispatch edit. Witness
`w4_c8_final_the_gold_collection_is_the_full_fourteen` pins all 14
gate-openers + the 13-row table + the 593 absence + the squirrel's two
peers.

## 3. The digger rides the UPGRADED worm-head machine — compose-verified

W4-d22 `b24977d99` landed the faithful AI_006 head kinematics (audit
T1b). The c8 digger (375) rides `worm_head_step` — the compose
transplanted the upgrade and re-pinned the witnesses:

- **Air stumble** (`:51925-51973`): velocity-sign kicks (not facing),
  the snapped aim num48 (not the raw centre delta), vy>4 decel toward
  zero.
- **Dig steer** (`:52112-52199`): the three-arm accel approach (NOT the
  velocity snap) — same-sign num46 steer + two cross-axis ×2 kicks,
  num65>num66 dominant-axis ×1.1 else arms with the num45*0.5 slow
  nudge.
- **Dig-mode latch** (`:52202-52224`): `localAI[0]` tracks flag3 on the
  7/10/13/39/95/98/117/510/513/621 list — **375 is NOT on it** (verified
  against the C# type list; the ported subset 7/10/13/117). The
  velocity-flip netUpdate is inert (noTileCollide skips
  `UpdateCollision`'s oldVelocity capture).
- **The 375 FLEE inversion** (`:51861-51865`) MOVED to the C# position:
  after the tile snap, BEFORE the length and BOTH branches. The
  pre-compose port flipped inside the dig branch only; the C# flips
  num48/num49 once, so the air stumble's terminal-fall steer reads the
  away-aim too. This is the compose's one semantic delta — the digger
  now flees in the air arm as well (faithful).

Witness re-pins (all WRITTEN NOT RUN): the digger's first-tick flee
accel is **−0.165** (num46*1.1, X-dominant else arm from rest), not the
−6 snap; the air kick from rest is +0.165/0.11 with NO latch write; the
corruption-escape / far-player-override / digs-and-stumbles tests
re-derived per d22 (accel from rest −0.1/+0.11, 150-tick saturation,
2000-tile world). Two d22 witnesses carried as `w4_c8_compose_*` so the
crawler branch keeps its own kinematics pins after the merge (the d22
originals keep their names on their branch — the merge dedupes by
content, keep-one).

**Numeric re-derivation delta (the JS mirror of the composed machine,
f32-rounded)**: every single-tick pin verified exactly (−0.165/+0.165/
0.11 digger, −0.1/+0.11 escape, +0.055/−0.05 dig accel, −1.11 air
sign). The `the_worm_head_digs_and_stumbles` saturation loop is
**110 ticks, NOT d22's 150**: on the `ground_air` fixture (the surface
hole x 960..2240 / y 2000..2160) the head reaches the hole at ~tick 118
and the air arm flips vy positive — at 150 ticks speed 5.50 / vy +1.02
(the d22 pin would FAIL on this fixture). At 110 ticks: speed 5.955,
vy −2.5, y 2165 — still digging, saturated within 0.1. The far-player
override's 150-tick loop is fine (empty world, no hole — speed 6.005).
**Cross-note to d22**: re-check your 150-tick `ground_air` pin on the
merged base — the composed machine + this fixture saturates at ~109
and overshoots by 150.

**Cross-note to d22**: the c8 compose is line-for-line your
`worm_head_step` body plus the 375 inversion at `:51861` and the 375
head-claim/override rows (already on c8). At merge the two
`worm_head_step` bodies are identical except those c8 rows — keep c8's
(the superset).

## 4. The crawler sheets in the union inventory — the d35 lazy-load census

The d35 FINAL (`e5558b0a9`, research/113 §10-12) census: **ALL
`TextureAssets.Npc` register `DoNotLoad`** (`AssetInitializer.cs:127-129`);
`Main.LoadNPC` (`Main.cs:59084-59090`) fires per DRAWN NPC —
`NPC.SetFrameSize` (`NPC.cs:66661-66663`) ← `DrawNPCDirect`
(`Main.cs:22516`), the bestiary hover (`UnlockableNPCEntryIcon.cs:185`),
the leashed critters (`LeashedCritter.cs:199`,
`ShimmerFlyLeashedCritter.cs:60`), + 8 cross-draw sites. **NOT
load-on-spawn** — the working set is what DRAWS, the lifetime set what
the world spawns + the bestiary hovers.

The nine crawler sheets ARE in the d35 FINAL 490-name union inventory
(§11/Appendix B): `NPC_357`/`NPC_606` rows 82/83 and `NPC_374`/`NPC_375`
rows 50/51 owned **c8**; `NPC_448`/`NPC_484-487` ride the d40 forecast
(the same files). Real retail dims (measured with the pinned d35 LZX
first-frame harness, `tools/_d35_atlas_sim_pinned.js` head):

| sheet | dims | padded px (PAD 2) | rows (`Main.cs:67082`) | row h |
|---|---|---|---|---|
| NPC_357 | 14×16 | 288 | 2 | 8 |
| NPC_374 | 38×144 | 5840 | 4 | 36 |
| NPC_375 | 38×144 | 5840 | 4 | 36 |
| NPC_448 | 14×16 | 288 | 2 | 8 |
| NPC_484 | 18×20 | 440 | 2 | 10 |
| NPC_485 | 18×72 | 1480 | 4 | 18 |
| NPC_486 | 18×72 | 1480 | 4 | 18 |
| NPC_487 | 18×72 | 1480 | 4 | 18 |
| NPC_606 | 14×16 | 288 | 2 | 8 |

Total 17,424 padded px; every height divides evenly by its row count
(the `SetFrameSize` integer division loses nothing). The port packs
them EAGERLY today (`STATIC_SHEET_NAMES` + the `npc_sheets` chain) —
the documented superset deviation (the E-4b precedent); under the §10.6
step-7 amendment the rows leave the eager union and join the per-world
RESIDENT+SPAWNABLE NPC page. The bestiary-opened superset covers 8 of
the 9 (`BESTIARY_NPC_SHEET_IDS`; 375 is Hide — folds onto the
TruffleWorm card, draws its own sheet via `texture_net_id`). Witness
`w4_c8_final_the_crawler_sheets_are_in_the_union_inventory` pins the
packing rows, the kind→sheet mapping, the dims/rows and the superset.

**Cross-note to d35**: the c8 rows need no inventory edit — they are
already in the FINAL union with c8 ownership; the step-7 NPC-page
census (`_d35_npc_census.json`) predates the c8 landing (its entryNets
lack 357/374/375/448/484-487/606) — re-run the extraction on the merged
base so the per-world NPC page includes the crawler family (the
bestiary superset argument covers them regardless: all 8 visible ids
are ENTRIES rows).

## 5. Witness register (all WRITTEN NOT RUN — code-only mode)

| witness | file | pins |
|---|---|---|
| `w4_c8_final_the_generic_critter_gate_covers_the_goldfish` | bestiary.rs | the generic gate, the Goldfish sight unlock, the 549 Hide exclusion |
| `w4_c8_final_the_gold_collection_is_the_full_fourteen` | bestiary.rs | all 14 gate-openers, the 13-row peer table, 593 absent, the squirrel two-peer row |
| `w4_c8_final_the_sight_scan_covers_the_other_lanes_critters` | world_sim/npcs.rs | the scan-level Goldfish + the cross-lane gold gate |
| `w4_c8_compose_the_dig_steer_accelerates_instead_of_snapping` | world_sim/npcs.rs | the d22 accel + latch (compose carry) |
| `w4_c8_compose_the_air_stumble_keys_on_the_velocity_sign` | world_sim/npcs.rs | the d22 velocity-sign kick + air latch (compose carry) |
| `w4_c8_the_digger_rides_the_worm_pass_and_flees` (re-pinned) | world_sim/npcs.rs | the −0.165 flee accel, the +0.165/0.11 air kick, no 375 latch |
| `w4_c8_final_the_crawler_sheets_are_in_the_union_inventory` | render/entities.rs | the packing rows, texture_net_id, dims/rows, the bestiary superset |
| re-derived: `the_corruption_escape_digs_down_and_despawns_deep`, `the_far_player_forces_the_dig_override`, `the_worm_head_digs_and_stumbles` | world_sim/npcs.rs, npc.rs | the d22 accel semantics (from-rest nudges, saturation — 110 ticks on ground_air, the numeric mirror's delta vs d22's 150) |

## 6. Merge notes

- **d22**: `worm_head_step` — keep c8's body (the superset: identical
  kinematics + the 375 inversion at the C# position + the 375 claim
  rows). The `w4_c8_compose_*` witnesses duplicate d22's
  `w4_d22_the_dig_steer_*`/`w4_d22_the_air_stumble_*` by content —
  keep-one at merge (either name; the bodies are the same). The
  `the_worm_head_digs_and_stumbles` saturation loop: keep c8's 110-tick
  pin (the numeric mirror shows d22's 150 overshoots the ground_air
  hole — §3).
- **c4**: the Goldfish provider freeze is lifted HERE; c4's UI
  `unlock_state` call sites reconcile onto `unlock_state_world` (the
  c8 fold). c4's GoldGoldfish 592 entry auto-rides the gold arm.
- **d40/e26/c7/c9/c10**: no provider wiring needed (§1/§2) — row the
  kinds and the generic gates cover them.
- **d35**: re-run `_d35_npc_census.py` on the merged base (§4).
