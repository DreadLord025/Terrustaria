# 68 — FindFrame master wire-census (W4-d44 census refresh, code-only)

Audit date: 2026-11-21 (refresh 5 FINAL — the wave-B residual wire
round + the full-band FindFrame-family declaration, §10; the §2
table is the refresh-2 mechanical scan with the refresh-3/4/5 status
cells updated, see the census-vintage note below §0). **UNCLAIMED = 0
at the refresh-5 probe-time tips — every one of the 602 case ids
carries an inline arm or a module machine on some pushed tip (§10).**
Branch `task/W4-d44-findframe-low-1`,
worktree `TR-W4-d44`. **Code-only mode: no cargo/rustc/tests/rustfmt/
graphify invoked** — every claim below is git/file-read evidence against
the PUSHED remote tips (`git ls-remote origin` + `git show
origin/<branch>:<file>` + `git cat-file blob`), re-probed at census time.
The per-kind table (§2) is derived MECHANICALLY: a Python transcription
of the C# switch-label scan (`NPC.cs:66705-77370`, two-tab `case` labels
only — the nested switches excluded) joined against the port's
`find_frame` arm scan (this tree + every pushed roster lane's own
`npc.rs`, mapped through each lane's `cs_id` hand rows + bestiary net-id
fallback + credit folds) and every pushed module's mapper/vanilla_type
table. No hand-counted rows.

Purpose: the merger's worklist for the `NPC.FindFrame` pool — the
per-kind 1-697 status, the per-module bridge shapes, the remaining
UNCLAIMED kind list with C# case-line anchors, and the remaining-work
estimate.

## 0. The census arithmetic (refresh 2 mechanically re-derived; refresh 3/4/5 wire-round deltas applied)

| step | count |
| --- | --- |
| C# top-level `case` ids (`NPC.cs:66705`, two-tab labels) | **602** |
| ids 1-697 with NO case label (the NO-CASE rows, §2) | 95 |
| port-covered: an inline `find_frame` arm on main OR a pushed roster lane (d34/d35/d37/d38/d40/d41/d42/d43) OR the wave-A/B module rosters | **271** (refresh-2: 273; the wave-B 294/296 moved to module-claimed-WIRED at refresh 5) |
| module-claimed ids (deduped across the pushed module mappers) | **201** (refresh-2: 199 + the 2 moved ids) |
| — of those, WIRED (a pushed cs_id hook claims the id: g6/g7/d44/d45/d46/e13) | **198** |
| — of those, MODULE-LANDED but UNWIRED (the extra7b keep-inline trio ONLY — RESOLVED, not wiring work) | **3** |
| **UNCLAIMED** (no arm, no module claim on ANY pushed tip — refresh-2 vintage; the refresh-5 re-probe shrinks it to ZERO true no-coverage, see the vintage note) | **130** |

Check: 271 + 201 + 130 = 602 (198 + 3 = 201). The f26 slice-7c addendum
(`68-findframe-census-addendum-7c.md`, tip `a2e47596`) published **127**;
the +3 delta is **435/436/591** — the d49 armed-zombie case-group tail
(`NPC.cs:70884-70886` ride the `case 430..436/591` group body) which the
d49 ANALYSIS explicitly defers ("435/436/591 ride the next wave") and
the d49/g7 mapper does NOT list. They are UNCLAIMED here, not covered.

The wire-round delta (refresh 3, THIS branch): +27 WIRED — extra7's 15
(316-318/322-324/333-337/437/438/440/442, `5568fabf`) + extra7b's 12
d40-stub kinds (473-477/479/483-488, `d1d20450`).

The wire-round delta (refresh 4, THIS branch): +30 WIRED — extra7c's 15
(494-506/508/509, `2a4ddf32e`) + extra8's 15 (443-454/460-462,
`44da5dc19`). 166 + 30 = **196 WIRED**.

The wire-round delta (refresh 5, THIS branch): +2 WIRED — the wave-B
residual pair **294 AngryBonesBig / 296 AngryBonesBigHelmet**
(`f4a7074bc`, the d38 SEAL-RECEIPT research/293 §14 exact hunks — the
`find_frame_kind_b` mapper + the four-field `find_frame_wave_b` bridge
+ the cs_id hook at the END of the world-free chain; 289 EXCLUDED per
the E3 dedupe). 196 + 2 = **198 WIRED**. These 2 ids were counted in
the 273 INLINE-ARM bucket at refresh 2 (the waveB module roster) —
they MOVE to module-claimed-WIRED here, so the buckets re-derive as
271 + 201 + 130 = 602 at the refresh-5 vintage (the table above).
The remaining 3
MODULE-LANDED-unwired = the extra7b keep-inline trio 480/481/482 (the
f26e verdict: the d40 INLINE arms win at merge — RESOLVED, not wiring
work). **The wave-B residual was the LAST findframe stand-in pair —
the d38 §11 seven-stand-in gap list is now FULLY hook-wired on this
branch (5 via extra6/extra7 + these 2).**

The roster-lane coverage note: d35's arms for 73-140 are `{}` row-0
STUBS ("UNWIRED (data-only)", its own comment) — they count as
INLINE-ARM coverage (the kind exists + an arm exists) but the faithful
machines for those ids are the d45/e14 modules (already WIRED). d38's
287/295/322/323/324 + d40's 453/460/462 ride the `humanoid_town_frame`
stand-in (the port's documented town-walk shape); d40's 443-448/473-477/
479/483-488 are row-0 stubs. The MODULE-LANDED machines are the faithful
upgrades for those — the keep-one verdicts are in §3.

Census-vintage note (refresh 5): the §2 per-kind table below is the
refresh-2 mechanical scan's output; the refresh-3/4/5 updates change
ONLY the status cells this branch's wire rounds touch (the extra7 15 +
the extra7b 12 + the extra7c 15 + the extra8 15 + the wave-B 2 →
WIRED(d44)). The f26/f27/d41/d42/d43 tips moved since refresh 4 —
re-probed at refresh-5 time (the lanes are IN FLIGHT; the tips below
are the probe-time snapshots, they may have advanced again by merge):
* f26 `7f38fd34`→**`a2e43ef77`** adds extra7g (**435/436/631/632/
  635**), extra7h (**636/639-645**), extra7i (**646-649**), extra7j
  (**650-655** + the 661 butterfly-arm claim `2ba235a17`), extra7k
  (**657-662**, the 661 twin + the count-table FIX `808f57f9f`/
  `a2e43ef77`);
* f27 `1a24a133`→**`01888909a`** adds extra11 (**435/436/550/
  552-563** — the 435/436 twin with f26 7g; the 552-563 DD2 v1-arm
  FIXES self-wired on f27's own npc.rs `9db8ed0f9`) + extra12
  (**551** Betsy, `70762233d`);
* d41 `5d036721`→**`065efdf97`** (the 527-531 hallowed-ghoul/lamia/
  sand-poacher batch rides it — no new findframe claim relevant to
  the UNCLAIMED list);
* d42 `bac408be0`→**`adb304ab8`** (the SEAL + SEAL-VERIFY docs; the
  578-630 roster arms stand — 38 of the refresh-2 UNCLAIMED ids carry
  real inline arms there, the 581/587/592/604/605/606/607/615 eight
  are c4/e26/c8-owned per the d42 research/239 §3 re-adjudication,
  mechanically re-verified: all 8 carry arms on their owner tips);
* d43 `6fbe06250`→**`31310b652`** (the retire + SEAL-VERIFY docs; the
  631-697 roster arms stand — ALL 31 remaining-band ids carry inline
  arms there, 13 as `{}` row-0 stubs: 637/638/656/670/678-684).
The refresh-4 stale-24 note RESOLVED + WIDENED: the mechanical re-join
(the refresh-2 UNCLAIMED 130-id list minus every pushed module mapper
at the probe-time tips) claims **53** of the 130 (the 24 refresh-4
stale ids + 435/436 + 631/632/635/636/639-655/657-662 the f26 7g-7k
waves + the f27 extra11/12 550-563 band was already roster-covered) —
**77 UNCLAIMED at the probe-time tips** by module-claim arithmetic.
BUT the roster-arm sweep covers ALL 77 with inline arms on their
owner tips: **38 on d42** (578/580/582-586/590/591/593-603/608-614/
616-620/624-629), **31 on d43** (634/637/638/656/664/667/669-684/
687-689/691-696), **8 on c4/e26/c8** (581/587/592/607/615 c4
style-44/16, 604/605 e26, 606 c8 — the d42 research/239 §3
re-adjudication, each arm mechanically re-verified on its owner tip).
38 + 31 + 8 = 77: the TRUE no-coverage residual is **ZERO ids with
neither an arm nor a module claim** — every one of the refresh-2 130
is covered on SOME pushed tip at refresh-5 time. The §6 worklist is
therefore a KEEP-ONE/faithfulness worklist now (the arm quality
varies: 13 d43 `{}` row-0 stubs — 637/638/656/670/678-684 — plus the
d42 stand-ins need the faithful-machine verdicts), not a
port-from-scratch list. The §2/§6 UNCLAIMED cells stay refresh-2
vintage until the next full mechanical refresh re-derives them with
the arm-quality column.

## 1. Branch census (all pushed tips, re-probed 2026-11-21 refresh 2)

| lane | branch (`task/W4-…`) | pushed tip | module(s) | kinds | hook status |
| --- | --- | --- | --- | --- | --- |
| d44 | `d44-findframe-low-1` | `925423511` | `npc_findframe_low1.rs` + the wire rounds (`c`/`extra5`/`extra6`/`extra7`/`extra7b`/`extra7c`/`extra8`/`wave-b`) | 12 + 11 + 15 + 15 + 15 + 12 + 15 + 15 + 2 | low1 wired by g6 (`5113ff18`); c/extra5/extra6/extra7/extra7b/extra7c/extra8/wave-b wired HERE (§7/§7b/§7c/§7d) |
| d45 | `d45-findframe-low-2` | `aea0aebd` | `npc_findframe_low2.rs` | 12 | YES (self-wired) |
| d46 | `d46-findframe-mid-1` | `055fcf8c` | `npc_findframe_mid1.rs` | 12 | YES (self-wired) |
| d47 | `d47-findframe-mid-2` | `3d36f76c` | `npc_findframe_mid2.rs` | 12 | NO — wired by g6 (`2fe567f9`) |
| d48 | `d48-findframe-high-1` | `3eb2ee08` | `npc_findframe_c.rs` | 12 | NO on its own branch — **WIRED(d44)** (§7, 355-365 only) |
| d49 | `d49-findframe-high-2` | `dadfd828` | `npc_findframe_high2.rs` | 12 / 13 ids | NO — wired by g7 (`b6c9c05c`+`1b273055`) |
| e13 | `e13-findframe-extra-1` | `9fbca588` | `npc_findframe_extra1.rs` | 15 | YES (self-wired) — fully twinned, DROP candidate (§3) |
| e14 | `e14-findframe-extra-2` | `63bc2c9f` | `npc_findframe_extra2.rs` | 15 | NO — wired by g6 (`b58d8d2f`) AND g7 (`fbee59b3`, trimmed 7-id) |
| f26 | `f26-findframe-extra-3` | `a2e43ef77` (refresh-5 probe; IN FLIGHT) | `extra3` (s3) + `extra5` (s5) + `extra6` (s6) + `extra7b` (s7b) + `extra7c` (s7c) + `extra7d`/`extra7e`/`extra7f`/`extra7g`/`extra7h`/`extra7i`/`extra7j`/`extra7k` | 15×5 + 5×3 + 5 + 8 + 4 + 7 + 6 | s3: g6+g7; s5/s6: **WIRED(d44)**; s7b: **WIRED(d44 `d1d20450`, 12 stub kinds — 480/481/482 keep d40 inline)**; s7c: **WIRED(d44 `2a4ddf32e`, all 15 — 494-506 faithful-inline twins on d41, behaviour-identical either way)**; 7d/7e/7f (516/521/523-530 + 531/532/534/535/536) + 7g/7h/7i/7j/7k (435/436/631/632/635/636/639-655/657-662, the 661 7j/7k twin) UNWIRED |
| f27 | `f27-findframe-extra-4` | `01888909a` (refresh-5 probe; IN FLIGHT) | `extra4` (s4) + `extra7` (s7) + `extra8` (s8) + `extra9` (s9) + `extra10` (s10) + `extra11` (s11) + `extra12` (s12) | 15×3 + 5 + 15 + 15 + 1 | s4: g7; s7: **WIRED(d44 `5568fabf`)**; s8: **WIRED(d44 `44da5dc19`, all 15 — NO exclusions, the d40-faithful six 464/465/466/468/469/470 were never in the mapper)**; s9: the d40-arm FIX set (463/467/471/472/492), self-wired on f27's own npc.rs (the 439 stub-fix rides it too); s10: 531/532/534-545 (the 531/532/534/535/536 five TWIN f26 7f) + the 549 lane-portal FIX, UNWIRED; s11: 435/436 (TWIN f26 7g) + 550/552-563 (the DD2 v1-arm FIXES, self-wired `9db8ed0f9`); s12: 551 Betsy, UNWIRED |
| g6 | `g6-npc-findframe-wire-1` | `bf99e983` | wires low1 + mid2 + extra2 + extra3 | 54 ids / 4 hooks | the wire lane itself |
| g7 | `g7-npc-findframe-wire-2` | `6ca3e668` | wires extra2(trim) + extra3 + extra4 + high2 | 50 ids / 4 hooks | the wire lane itself |

Roster lanes with landed `find_frame` arms (the INLINE-ARM coverage):
d34 `5f3435a8` (37/43/46/47/48/56/57/61/62/63), d35 `7d131be3` (the
73-140 row-0 stubs + the faithful 121/137 etc.), d37 `08f7d25c` (the
217-280 band incl. the faithful 278-280 armored-bones group), d38
**`b888a9575`** (refresh-5 probe — the 281-337 band + the SEAL-RECEIPT
research/293 §§14-18; the 294/296 stand-in arms `npc.rs:9514-9515`/
`:9518-9519` STILL STAND there — the drop is the d38-merge edit, §7d),
d40 `8fe1d6d3` (the 437-488 band), d41 **`065efdf97`** (refresh-5
probe — the 494-506 FAITHFUL inline arms + the 527-531 batch),
**d42 `adb304ab8` (NEW at refresh 5 — the 578-630 band, 38 real arms
over the refresh-2 UNCLAIMED list; the 581/587/592/604-607/615 eight
are c4/e26/c8-owned per its research/239 §3)**, **d43 `31310b652`
(NEW at refresh 5 — the 631-697 band, 31 arms over the list, 13 as
row-0 stubs)**. d36/d39 carry NO new `find_frame` arms. The
d34/d35/d37/d38/d41/d42/d43 tips moved since their last census scan —
the refresh-5 rows cite the probe-time tips.

Main base for both wire lanes: `a725acb0` (wave-3 land). All hooks are
`cs_id`-keyed at the `npc.rs::find_frame` head, ahead of the body match,
each `return`ing after one machine tick; every hook is stream-inert on
its own branch base (zero collisions over the `npc/data.rs` hand cs_id
rows + the bestiary net_id fallback). **At merge the roster lanes add
the kinds — the hook then precedes the roster's inline arm and wins
(first-match-wins); that is the intended keep-one for the faithful
module over the roster stub/stand-in (§3).**

## 2. Per-kind master table (1-697, mechanically derived)

Status legend:
* **WIRED(lane)** — a pushed cs_id hook claims the id (g6/g7 wire lanes,
  the d44 wire round, or the d45/d46/e13 self-wired branches).
* **MODULE-LANDED(lane)** — a pushed module carries the faithful machine
  but NO hook claims the id yet (the wiring worklist, §5). The
  **KEEP-INLINE** suffix (refresh 3) marks the f26e keep-one verdict:
  the d40 INLINE arm is faithful and wins at merge — the module arm
  is a parity reference and NO hook will ever claim the id.
* **INLINE-ARM** — a `find_frame` match arm exists on main or a pushed
  roster lane (faithful inline, the `humanoid_town_frame` stand-in, or a
  row-0 stub — the arm's source is cited).
* **UNCLAIMED** — no arm and no module claim on ANY pushed tip (the
  port-round worklist, §6).
* **NO-CASE** — the id has no top-level `case` label in the C# switch
  (the single-frame kinds / the folded segment families' own labels).

The `module` column names the `npc_findframe_*` module (sans prefix);
`twin claim` marks ids claimed by 2+ modules (the keep-one dedupe, §3).
The anchor is the FIRST top-level `case` line of the id's group.

| id | kind | status | module / arm source | C# anchor |
| --- | --- | --- | --- | --- |
| 1 | BlueSlime | INLINE-ARM | npc.rs:Slime | NPC.cs:70801 |
| 2 | DemonEye | INLINE-ARM | npc.rs:DemonEye | NPC.cs:73994 |
| 3 | Zombie | INLINE-ARM | npc.rs:Zombie | NPC.cs:76324 |
| 4 | EyeofCthulhu | INLINE-ARM | npc.rs:EyeOfCthulhu | NPC.cs:76905 |
| 5 | ServantofCthulhu | INLINE-ARM | npc.rs:ServantOfCthulhu | NPC.cs:76931 |
| 6 | EaterofSouls | INLINE-ARM | npc.rs:EaterOfSouls | NPC.cs:76966 |
| 7 | DevourerHead | NO-CASE | — | — |
| 8 | DevourerBody | NO-CASE | — | — |
| 9 | DevourerTail | NO-CASE | — | — |
| 10 | GiantWormHead | NO-CASE | — | — |
| 11 | GiantWormBody | NO-CASE | — | — |
| 12 | GiantWormTail | NO-CASE | — | — |
| 13 | EaterofWorldsHead | NO-CASE | — | — |
| 14 | EaterofWorldsBody | NO-CASE | — | — |
| 15 | EaterofWorldsTail | NO-CASE | — | — |
| 16 | MotherSlime | INLINE-ARM | npc.rs:MotherSlime | NPC.cs:70802 |
| 17 | Merchant | INLINE-ARM | npc.rs:Merchant | NPC.cs:68543 |
| 18 | Nurse | INLINE-ARM | npc.rs:Nurse | NPC.cs:68544 |
| 19 | ArmsDealer | INLINE-ARM | npc.rs:ArmsDealer | NPC.cs:68545 |
| 20 | Poisoned | INLINE-ARM | npc.rs:Dryad | NPC.cs:68546 |
| 21 | Skeleton | INLINE-ARM | npc.rs:Skeleton | NPC.cs:68547 |
| 22 | Guide | INLINE-ARM | npc.rs:Guide | NPC.cs:68548 |
| 23 | MeteorHead | INLINE-ARM | npc.rs:MeteorHead | NPC.cs:73995 |
| 24 | OnFire | INLINE-ARM | npc.rs:FireImp | NPC.cs:76979 |
| 25 | BurningSphere | NO-CASE | — | — |
| 26 | GoblinPeon | INLINE-ARM | npc.rs:GoblinPeon | NPC.cs:68549 |
| 27 | GoblinThief | INLINE-ARM | npc.rs:GoblinThief | NPC.cs:68550 |
| 28 | GoblinWarrior | INLINE-ARM | npc.rs:GoblinWarrior | NPC.cs:68551 |
| 29 | GoblinSorcerer | INLINE-ARM | npc.rs:GoblinSorcerer | NPC.cs:77019 |
| 30 | Bleeding | NO-CASE | — | — |
| 31 | Confused | INLINE-ARM | waveB:AngryBones | NPC.cs:68552 |
| 32 | DarkCaster | INLINE-ARM | npc.rs:DarkCaster | NPC.cs:77020 |
| 33 | WaterSphere | NO-CASE | — | — |
| 34 | CursedSkull | INLINE-ARM | waveB:CursedSkull | NPC.cs:77055 |
| 35 | SkeletronHead | INLINE-ARM | npc.rs:SkeletronHead | NPC.cs:66767 |
| 36 | SkeletronHand | INLINE-ARM | npc.rs:SkeletronHand | NPC.cs:66773 |
| 37 | OldMan | WIRED(g6) | `low1` | NPC.cs:68553 |
| 38 | Demolitionist | INLINE-ARM | npc.rs:Demolitionist | NPC.cs:68554 |
| 39 | CursedInferno | NO-CASE | — | — |
| 40 | BoneSerpentBody | NO-CASE | — | — |
| 41 | BoneSerpentTail | NO-CASE | — | — |
| 42 | Hornet | INLINE-ARM | npc.rs:Hornet | NPC.cs:74907 |
| 43 | ManEater | WIRED(g6) | `low1` | NPC.cs:74970 |
| 44 | Frostburn | INLINE-ARM | npc.rs:UndeadMiner | NPC.cs:68555 |
| 45 | Tim | INLINE-ARM | npc.rs:Tim | NPC.cs:77021 |
| 46 | Bunny | WIRED(g6) | `low1` | NPC.cs:76851 |
| 47 | CorruptBunny | WIRED(g6) | `low1` | NPC.cs:76852 |
| 48 | Harpy | WIRED(g6) | `low1` | NPC.cs:74823 |
| 49 | CaveBat | INLINE-ARM | npc.rs:CaveBat | NPC.cs:74824 |
| 50 | KingSlime | INLINE-ARM | npc.rs:KingSlime | NPC.cs:73380 |
| 51 | JungleBat | INLINE-ARM | npc.rs:JungleBat | NPC.cs:74825 |
| 52 | DoctorBones | INLINE-ARM | npc.rs:DoctorBones | NPC.cs:76325 |
| 53 | TheGroom | INLINE-ARM | npc.rs:TheGroom | NPC.cs:76326 |
| 54 | Clothier | INLINE-ARM | npc.rs:Clothier | NPC.cs:68556 |
| 55 | Goldfish | INLINE-ARM | npc.rs:Goldfish | NPC.cs:74181 |
| 56 | Snatcher | WIRED(g6) | `low1` | NPC.cs:74971 |
| 57 | CorruptGoldfish | WIRED(g6) | `low1` | NPC.cs:74182 |
| 58 | Piranha | INLINE-ARM | npc.rs:Piranha | NPC.cs:74183 |
| 59 | LavaSlime | INLINE-ARM | npc.rs:LavaSlime | NPC.cs:70803 |
| 60 | Hellbat | INLINE-ARM | npc.rs:Hellbat | NPC.cs:74826 |
| 61 | Vulture | WIRED(g6) | `low1` | NPC.cs:73537 |
| 62 | Demon | WIRED(g6) | `low1` | NPC.cs:73887 |
| 63 | BlueJellyfish | WIRED(g6) | `low1` | NPC.cs:73916 |
| 64 | PinkJellyfish | WIRED(g6) | `low1` | NPC.cs:73917 |
| 65 | Shark | INLINE-ARM | npc.rs:Shark | NPC.cs:74654 |
| 66 | VoodooDemon | INLINE-ARM | npc.rs:VoodooDemon | NPC.cs:73888 |
| 67 | Crab | WIRED(g6) | `low1` | NPC.cs:74571 |
| 68 | DungeonGuardian | NO-CASE | — | — |
| 69 | Ichor | WIRED(d45-self) | `low2` | NPC.cs:74291 |
| 70 | Venom | NO-CASE | — | — |
| 71 | DungeonSlime | INLINE-ARM | npc.rs:DungeonSlime | NPC.cs:70804 |
| 72 | BlazingWheel | INLINE-ARM | npc.rs:BlazingWheel | NPC.cs:74642 |
| 73 | GoblinScout | WIRED(d45-self) | `low2` | NPC.cs:68557 |
| 74 | Bird | WIRED(d45-self) | `low2` | NPC.cs:73628 |
| 75 | Pixie | INLINE-ARM | waveB:Pixie | NPC.cs:74113 |
| 76 | None2 | NO-CASE | — | — |
| 77 | ArmoredSkeleton | INLINE-ARM | waveB:ArmoredSkeleton | NPC.cs:68558 |
| 78 | Mummy | INLINE-ARM | waveB:Mummy | NPC.cs:68559 |
| 79 | DarkMummy | INLINE-ARM | waveB:DarkMummy | NPC.cs:68560 |
| 80 | LightMummy | INLINE-ARM | waveB:LightMummy | NPC.cs:68561 |
| 81 | CorruptSlime | INLINE-ARM | npc.rs:CorruptSlime | NPC.cs:70805 |
| 82 | Wraith | INLINE-ARM | waveB:Wraith | NPC.cs:74827 |
| 83 | CursedHammer | WIRED(d45-self) | `low2` | NPC.cs:74622 |
| 84 | EnchantedSword | WIRED(g6+g7) | `extra2` (+1 twin claim) | NPC.cs:74623 |
| 85 | Mimic | INLINE-ARM | waveB:Mimic | NPC.cs:73443 |
| 86 | Unicorn | INLINE-ARM | waveB:Unicorn | NPC.cs:74404 |
| 87 | WyvernHead | NO-CASE | — | — |
| 88 | WyvernLegs | NO-CASE | — | — |
| 89 | WyvernBody | NO-CASE | — | — |
| 90 | WyvernBody2 | NO-CASE | — | — |
| 91 | WyvernBody3 | NO-CASE | — | — |
| 92 | WyvernTail | NO-CASE | — | — |
| 93 | GiantBat | WIRED(g6+g7) | `extra2` (+1 twin claim) | NPC.cs:74828 |
| 94 | Corruptor | WIRED(g6+g7) | `extra2` (+1 twin claim) | NPC.cs:76943 |
| 95 | DiggerHead | NO-CASE | — | — |
| 96 | DiggerBody | NO-CASE | — | — |
| 97 | DiggerTail | NO-CASE | — | — |
| 98 | SeekerHead | NO-CASE | — | — |
| 99 | SeekerBody | NO-CASE | — | — |
| 100 | SeekerTail | NO-CASE | — | — |
| 101 | Clinger | WIRED(g6+g7) | `extra2` (+1 twin claim) | NPC.cs:75018 |
| 102 | AnglerFish | WIRED(g6+g7) | `extra2` (+1 twin claim) | NPC.cs:74184 |
| 103 | GreenJellyfish | WIRED(g6+g7) | `extra2` (+1 twin claim) | NPC.cs:73918 |
| 104 | Werewolf | INLINE-ARM | waveB:Werewolf | NPC.cs:68562 |
| 105 | BoundGoblin | NO-CASE | — | — |
| 106 | BoundWizard | NO-CASE | — | — |
| 107 | GoblinTinkerer | INLINE-ARM | npc.rs:GoblinTinkerer | NPC.cs:68563 |
| 108 | Wizard | INLINE-ARM | npc.rs:Wizard | NPC.cs:68564 |
| 109 | Clown | INLINE-ARM | waveB:Clown | NPC.cs:74606 |
| 110 | SkeletonArcher | INLINE-ARM | waveB:SkeletonArcher | NPC.cs:76017 |
| 111 | GoblinArcher | INLINE-ARM | npc.rs:GoblinArcher | NPC.cs:76138 |
| 112 | VileSpit | NO-CASE | — | — |
| 113 | WallofFlesh | INLINE-ARM | npc.rs:WallOfFlesh | NPC.cs:73516 |
| 114 | WallofFleshEye | INLINE-ARM | npc.rs:WallOfFleshEye | NPC.cs:73517 |
| 115 | TheHungry | INLINE-ARM | npc.rs:Hungry | NPC.cs:74995 |
| 116 | TheHungryII | WIRED(g6+g7) | `extra2` (+1 twin claim) | NPC.cs:74091 |
| 117 | LeechHead | NO-CASE | — | — |
| 118 | LeechBody | NO-CASE | — | — |
| 119 | LeechTail | NO-CASE | — | — |
| 120 | ChaosElemental | WIRED(g6+g7) | `extra2` (+1 twin claim) | NPC.cs:68565 |
| 121 | Slimer | WIRED(g6+g7) | `extra2` (+1 twin claim) | NPC.cs:73996 |
| 122 | Gastropod | INLINE-ARM | waveB:Gastropod | NPC.cs:73607 |
| 123 | BoundMechanic | NO-CASE | — | — |
| 124 | Mechanic | INLINE-ARM | npc.rs:Mechanic | NPC.cs:68566 |
| 125 | Retinazer | INLINE-ARM | npc.rs:Retinazer | NPC.cs:76906 |
| 126 | Spazmatism | INLINE-ARM | npc.rs:Spazmatism | NPC.cs:76907 |
| 127 | SkeletronPrime | INLINE-ARM | npc.rs:SkeletronPrime | NPC.cs:74498 |
| 128 | PrimeCannon | NO-CASE | — | — |
| 129 | PrimeSaw | INLINE-ARM | npc.rs:PrimeSaw | NPC.cs:74539 |
| 130 | PrimeVice | INLINE-ARM | npc.rs:PrimeVice | NPC.cs:74555 |
| 131 | PrimeLaser | NO-CASE | — | — |
| 132 | BaldZombie | INLINE-ARM | npc.rs:BaldZombie | NPC.cs:76327 |
| 133 | WanderingEye | INLINE-ARM | npc.rs:WanderingEye | NPC.cs:74061 |
| 134 | TheDestroyer | NO-CASE | — | — |
| 135 | TheDestroyerBody | INLINE-ARM | npc.rs:TheDestroyerBody(fold) | NPC.cs:73433 |
| 136 | TheDestroyerTail | NO-CASE | — | — |
| 137 | IlluminantBat | WIRED(g6+g7) | `extra2` (+1 twin claim) | NPC.cs:74829 |
| 138 | IlluminantSlime | INLINE-ARM | npc.rs:IlluminantSlime | NPC.cs:70806 |
| 139 | Probe | NO-CASE | — | — |
| 140 | PossessedArmor | INLINE-ARM | waveB:PossessedArmor | NPC.cs:68567 |
| 141 | ToxicSludge | WIRED(g6+g7) | `extra2` (+1 twin claim) | NPC.cs:72977 |
| 142 | SantaClaus | WIRED(g6+g7) | `extra2` (+1 twin claim) | NPC.cs:68568 |
| 143 | SnowmanGangsta | INLINE-ARM | npc.rs:SnowmanGangsta | NPC.cs:73268 |
| 144 | MisterStabby | INLINE-ARM | npc.rs:SnowBalla | NPC.cs:73298 |
| 145 | SnowBalla | INLINE-ARM | npc.rs:MysterySnowman | NPC.cs:73345 |
| 146 | None3 | NO-CASE | — | — |
| 147 | IceSlime | INLINE-ARM | npc.rs:IceSlime | NPC.cs:70807 |
| 148 | Penguin | WIRED(g6+g7) | `extra2` (+1 twin claim) | NPC.cs:76392 |
| 149 | PenguinBlack | WIRED(g6+g7) | `extra2` (+1 twin claim) | NPC.cs:76393 |
| 150 | IceBat | WIRED(d46-self) | `mid1` | NPC.cs:74786 |
| 151 | Lavabat | WIRED(d46-self) | `mid1` | NPC.cs:74787 |
| 152 | GiantFlyingFox | WIRED(d46-self) | `mid1` | NPC.cs:74788 |
| 153 | ShadowFlame | NO-CASE | — | — |
| 154 | IceTortoise | NO-CASE | — | — |
| 155 | Wolf | INLINE-ARM | waveB:Wolf | NPC.cs:74311 |
| 156 | RedDevil | WIRED(d46-self) | `mid1` | NPC.cs:73903 |
| 157 | Arapaima | WIRED(d46-self) | `mid1` | NPC.cs:74134 |
| 158 | VampireBat | WIRED(d46-self) | `mid1` | NPC.cs:74789 |
| 159 | Vampire | WIRED(d46-self) | `mid1` | NPC.cs:68569 |
| 160 | Truffle | WIRED(d46-self) | `mid1` | NPC.cs:68570 |
| 161 | ZombieEskimo | INLINE-ARM | npc.rs:ZombieEskimo | NPC.cs:76328 |
| 162 | Frankenstein | WIRED(d46-self) | `mid1` | NPC.cs:68571 |
| 163 | BlackRecluse | WIRED(d46-self) | `mid1` | NPC.cs:73227 |
| 164 | WallCreeper | WIRED(d46-self) | `mid1` | NPC.cs:73083 |
| 165 | WallCreeperWall | WIRED(d46-self) | `mid1` | NPC.cs:73112 |
| 166 | SwampThing | WIRED(g6) | `mid2` | NPC.cs:75618 |
| 167 | UndeadViking | INLINE-ARM | waveB:UndeadViking | NPC.cs:68572 |
| 168 | CorruptPenguin | WIRED(g6) | `mid2` | NPC.cs:76394 |
| 169 | BoneJavelin | WIRED(g6) | `mid2` | NPC.cs:73997 |
| 170 | PigronCorruption | WIRED(g6) | `mid2` | NPC.cs:73418 |
| 171 | PigronHallow | WIRED(g6) | `mid2` | NPC.cs:73419 |
| 172 | RuneWizard | WIRED(g6) | `mid2` | NPC.cs:77022 |
| 173 | Crimera | WIRED(g6) | `mid2` | NPC.cs:76967 |
| 174 | Herpling | INLINE-ARM | waveB:Herpling | NPC.cs:73172 |
| 175 | AngryTrapper | WIRED(g6) | `mid2` | NPC.cs:74972 |
| 176 | MossHornet | WIRED(g6) | `mid2` | NPC.cs:74958 |
| 177 | Derpling | INLINE-ARM | waveB:Derpling | NPC.cs:73203 |
| 178 | Steampunker | WIRED(g6) | `mid2` | NPC.cs:68573 |
| 179 | CrimsonAxe | WIRED(g6) | `mid2` | NPC.cs:74624 |
| 180 | PigronCrimson | WIRED(g6) | `mid2` | NPC.cs:73420 |
| 181 | FaceMonster | WIRED(g6+g7) | `extra2` (+1 twin claim) | NPC.cs:68574 |
| 182 | FloatyGross | WIRED(g6+g7) | `extra3` (+1 twin claim) | NPC.cs:74830 |
| 183 | Crimslime | INLINE-ARM | npc.rs:Crimslime | NPC.cs:70808 |
| 184 | SpikedIceSlime | INLINE-ARM | npc.rs:SpikedIceSlime | NPC.cs:70809 |
| 185 | SnowFlinx | WIRED(g6+g7) | `extra3` (+1 twin claim) | NPC.cs:68575 |
| 186 | PincushionZombie | INLINE-ARM | npc.rs:PincushionZombie | NPC.cs:76329 |
| 187 | SlimedZombie | INLINE-ARM | npc.rs:SlimedZombie | NPC.cs:76330 |
| 188 | SwampZombie | INLINE-ARM | npc.rs:SwampZombie | NPC.cs:76331 |
| 189 | Daybreak | INLINE-ARM | npc.rs:TwiggyZombie | NPC.cs:76332 |
| 190 | CataractEye | INLINE-ARM | npc.rs:CataractEye | NPC.cs:73998 |
| 191 | SleepyEye | INLINE-ARM | npc.rs:SleepyEye | NPC.cs:73999 |
| 192 | DialatedEye | INLINE-ARM | npc.rs:DialatedEye | NPC.cs:74000 |
| 193 | GreenEye | INLINE-ARM | npc.rs:GreenEye | NPC.cs:74001 |
| 194 | PurpleEye | INLINE-ARM | npc.rs:PurpleEye | NPC.cs:74002 |
| 195 | LostGirl | WIRED(g6+g7) | `extra3` (+1 twin claim) | NPC.cs:73146 |
| 196 | Nymph | WIRED(g6+g7) | `extra3` (+1 twin claim) | NPC.cs:68576 |
| 197 | ArmoredViking | INLINE-ARM | waveB:ArmoredViking | NPC.cs:68577 |
| 198 | Lihzahrd | WIRED(g6+g7) | `extra3` (+1 twin claim) | NPC.cs:68578 |
| 199 | LihzahrdCrawler | WIRED(g6+g7) | `extra3` (+1 twin claim) | NPC.cs:75030 |
| 200 | FemaleZombie | INLINE-ARM | npc.rs:FemaleZombie | NPC.cs:76333 |
| 201 | HeadacheSkeleton | WIRED(g6+g7) | `extra3` (+1 twin claim) | NPC.cs:68579 |
| 202 | MisassembledSkeleton | WIRED(g6+g7) | `extra3` (+1 twin claim) | NPC.cs:68580 |
| 203 | BetsysCurse | WIRED(g6+g7) | `extra3` | NPC.cs:68581 |
| 204 | Oiled | WIRED(g6+g7) | `extra3` | NPC.cs:70810 |
| 205 | Moth | WIRED(g6+g7) | `extra3` | NPC.cs:74935 |
| 206 | IcyMerman | INLINE-ARM | waveB:IcyMerman | NPC.cs:76060 |
| 207 | DyeTrader | INLINE-ARM | npc.rs:DyeTrader | NPC.cs:68582 |
| 208 | PartyGirl | INLINE-ARM | npc.rs:PartyGirl | NPC.cs:68583 |
| 209 | Cyborg | WIRED(g6+g7) | `extra3` | NPC.cs:68584 |
| 210 | Bee | INLINE-ARM | npc.rs:Bee | NPC.cs:74831 |
| 211 | BeeSmall | INLINE-ARM | npc.rs:BeeSmall | NPC.cs:74832 |
| 212 | PirateDeckhand | INLINE-ARM | npc.rs:PirateDeckhand | NPC.cs:68585 |
| 213 | PirateCorsair | INLINE-ARM | npc.rs:PirateCaptainsMate | NPC.cs:68586 |
| 214 | PirateDeadeye | INLINE-ARM | npc.rs:PirateArcher | NPC.cs:76018 |
| 215 | PirateCrossbower | INLINE-ARM | npc.rs:PirateCorsair | NPC.cs:76019 |
| 216 | PirateCaptain | INLINE-ARM | npc.rs:PirateCaptain | NPC.cs:76020 |
| 217 | CochinealBeetle | WIRED(g6+g7) | `extra3` | NPC.cs:74572 |
| 218 | CyanBeetle | WIRED(g6+g7) | `extra3` | NPC.cs:74573 |
| 219 | LacBeetle | WIRED(g6+g7) | `extra3` | NPC.cs:74574 |
| 220 | SeaSnail | INLINE-ARM | d37:SeaSnail(npc.rs:9482) | NPC.cs:74590 |
| 221 | Squid | INLINE-ARM | d37:Squid(npc.rs:9494) | NPC.cs:73961 |
| 222 | QueenBee | INLINE-ARM | npc.rs:QueenBee | NPC.cs:73054 |
| 223 | ZombieRaincoat | INLINE-ARM | npc.rs:ZombieRaincoat | NPC.cs:76334 |
| 224 | FlyingFish | INLINE-ARM | d37:FlyingFish(npc.rs:9521) | NPC.cs:74763 |
| 225 | UmbrellaSlime | INLINE-ARM | npc.rs:UmbrellaSlime | NPC.cs:70811 |
| 226 | FlyingSnake | INLINE-ARM | d37:FlyingSnake(npc.rs:9538) | NPC.cs:74790 |
| 227 | Painter | INLINE-ARM | npc.rs:Painter | NPC.cs:68587 |
| 228 | WitchDoctor | INLINE-ARM | npc.rs:WitchDoctor | NPC.cs:68588 |
| 229 | Pirate | INLINE-ARM | d37:Pirate(npc.rs:9557) | NPC.cs:68589 |
| 230 | GoldfishWalker | INLINE-ARM | d37:GoldfishWalker(npc.rs:9562) | NPC.cs:75883 |
| 231 | HornetFatty | INLINE-ARM | d37:HornetFatty(npc.rs:9595) | NPC.cs:74908 |
| 232 | HornetHoney | INLINE-ARM | d37:HornetHoney(npc.rs:9596) | NPC.cs:74909 |
| 233 | HornetLeafy | WIRED(g7) | `extra4` | NPC.cs:74910 |
| 234 | HornetSpikey | WIRED(g7) | `extra4` | NPC.cs:74911 |
| 235 | HornetStingy | WIRED(g7) | `extra4` | NPC.cs:74912 |
| 236 | JungleCreeper | WIRED(g7) | `extra4` | NPC.cs:73084 |
| 237 | JungleCreeperWall | WIRED(g7) | `extra4` | NPC.cs:73113 |
| 238 | BlackRecluseWall | WIRED(g7) | `extra4` | NPC.cs:73114 |
| 239 | BloodCrawler | WIRED(g7) | `extra4` | NPC.cs:73085 |
| 240 | BloodCrawlerWall | WIRED(g7) | `extra4` | NPC.cs:73115 |
| 241 | BloodFeeder | WIRED(g7) | `extra4` | NPC.cs:74185 |
| 242 | BloodJelly | WIRED(g7) | `extra4` | NPC.cs:73919 |
| 243 | IceGolem | WIRED(g7) | `extra4` | NPC.cs:72995 |
| 244 | RainbowSlime | WIRED(g7) | `extra4` | NPC.cs:72632 |
| 245 | Golem | INLINE-ARM | npc.rs:GolemFistLeft | NPC.cs:72898 |
| 246 | GolemHead | INLINE-ARM | npc.rs:GolemHead(fold) | NPC.cs:72931 |
| 247 | GolemFistLeft | NO-CASE | — | — |
| 248 | GolemFistRight | NO-CASE | — | — |
| 249 | GolemHeadFree | INLINE-ARM | npc.rs:GolemHeadFree(fold) | NPC.cs:72963 |
| 250 | AngryNimbus | INLINE-ARM | npc.rs:AngryNimbus | NPC.cs:72884 |
| 251 | Eyezor | WIRED(g7) | `extra4` | NPC.cs:76335 |
| 252 | Parrot | WIRED(g7) | `extra4` | NPC.cs:73558 |
| 253 | Reaper | WIRED(g7) | `extra4` | NPC.cs:74833 |
| 254 | ZombieMushroom | WIRED(d44) | `extra5` | NPC.cs:76336 |
| 255 | ZombieMushroomHat | WIRED(d44) | `extra5` | NPC.cs:76337 |
| 256 | FungoFish | WIRED(d44) | `extra5` | NPC.cs:73920 |
| 257 | AnomuraFungus | WIRED(d44) | `extra5` | NPC.cs:76179 |
| 258 | MushiLadybug | WIRED(d44) | `extra5` | NPC.cs:76225 |
| 259 | FungiBulb | NO-CASE | — | — |
| 260 | GiantFungiBulb | NO-CASE | — | — |
| 261 | FungiSpore | NO-CASE | — | — |
| 262 | Plantera | INLINE-ARM | npc.rs:Plantera | NPC.cs:72558 |
| 263 | PlanterasHook | INLINE-ARM | npc.rs:PlanterasHook | NPC.cs:72609 |
| 264 | PlanterasTentacle | INLINE-ARM | npc.rs:PlanterasTentacle | NPC.cs:72885 |
| 265 | Spore | NO-CASE | — | — |
| 266 | BrainofCthulhu | INLINE-ARM | npc.rs:BrainOfCthulhu | NPC.cs:72534 |
| 267 | Creeper | NO-CASE | — | — |
| 268 | IchorSticker | WIRED(d44) | `extra5` | NPC.cs:72886 |
| 269 | RustyArmoredBonesAxe | WIRED(d44) | `extra5` | NPC.cs:75825 |
| 270 | RustyArmoredBonesFlail | WIRED(d44) | `extra5` | NPC.cs:75826 |
| 271 | RustyArmoredBonesSword | WIRED(d44) | `extra5` | NPC.cs:75827 |
| 272 | RustyArmoredBonesSwordNoArmor | WIRED(d44) | `extra5` | NPC.cs:75828 |
| 273 | BlueArmoredBones | WIRED(d44) | `extra5` | NPC.cs:75829 |
| 274 | BlueArmoredBonesMace | WIRED(d44) | `extra5` | NPC.cs:75830 |
| 275 | BlueArmoredBonesNoPants | WIRED(d44) | `extra5` | NPC.cs:75831 |
| 276 | BlueArmoredBonesSword | WIRED(d44) | `extra5` | NPC.cs:75832 |
| 277 | HellArmoredBones | WIRED(d44) | `extra5` | NPC.cs:75833 |
| 278 | HellArmoredBonesSpikeShield | WIRED(d44 59b798dc) | `extra6` | NPC.cs:75834 |
| 279 | HellArmoredBonesMace | WIRED(d44 59b798dc) | `extra6` | NPC.cs:75835 |
| 280 | HellArmoredBonesSword | WIRED(d44 59b798dc) | `extra6` | NPC.cs:75836 |
| 281 | RaggedCaster | WIRED(d44 59b798dc) | `extra6` | NPC.cs:77023 |
| 282 | RaggedCasterOpenCoat | WIRED(d44 59b798dc) | `extra6` | NPC.cs:77024 |
| 283 | Necromancer | WIRED(d44 59b798dc) | `extra6` | NPC.cs:77025 |
| 284 | NecromancerArmored | WIRED(d44 59b798dc) | `extra6` | NPC.cs:77026 |
| 285 | DiabolistRed | WIRED(d44 59b798dc) | `extra6` | NPC.cs:77027 |
| 286 | DiabolistWhite | WIRED(d44 59b798dc) | `extra6` | NPC.cs:77028 |
| 287 | BoneLee | WIRED(d44 59b798dc) | `extra6` | NPC.cs:68590 |
| 288 | DungeonSpirit | INLINE-ARM | npc.rs:DungeonSpirit | NPC.cs:72676 |
| 289 | GiantCursedSkull | INLINE-ARM | d38:GiantCursedSkull (the E3 keep — the waveB arm is the dropped twin, NEVER claimed by `find_frame_kind_b`) | NPC.cs:72652 |
| 290 | Paladin | WIRED(d44 59b798dc) | `extra6` | NPC.cs:75963 |
| 291 | SkeletonSniper | WIRED(d44 59b798dc) | `extra6` | NPC.cs:76139 |
| 292 | TacticalSkeleton | WIRED(d44 59b798dc) | `extra6` | NPC.cs:76140 |
| 293 | SkeletonCommando | WIRED(d44 59b798dc) | `extra6` | NPC.cs:76021 |
| 294 | AngryBonesBig | WIRED(d44 f4a7074b) | `waveB` (the residual mapper `find_frame_kind_b`) | NPC.cs:68591 |
| 295 | AngryBonesBigMuscle | WIRED(d44 59b798dc) | `extra6` | NPC.cs:68592 |
| 296 | AngryBonesBigHelmet | WIRED(d44 f4a7074b) | `waveB` (the residual mapper `find_frame_kind_b`) | NPC.cs:68593 |
| 297 | BirdBlue | INLINE-ARM | d38:BirdBlue(npc.rs:9399) | NPC.cs:73629 |
| 298 | BirdRed | INLINE-ARM | d38:BirdRed(npc.rs:9413) | NPC.cs:73630 |
| 299 | Squirrel | INLINE-ARM | d38:Squirrel(npc.rs:9427) | NPC.cs:76458 |
| 300 | Mouse | INLINE-ARM | d38:Mouse(npc.rs:9442) | NPC.cs:76513 |
| 301 | Raven | INLINE-ARM | d38:Raven(npc.rs:9463) | NPC.cs:73559 |
| 302 | SlimeMasked | INLINE-ARM | d38:SlimeMasked(npc.rs:9337) | NPC.cs:70812 |
| 303 | BunnySlimed | INLINE-ARM | d38:BunnySlimed(npc.rs:9349) | NPC.cs:76853 |
| 304 | HoppinJack | INLINE-ARM | d38:HoppinJack(npc.rs:9364) | NPC.cs:70813 |
| 305 | Scarecrow1 | INLINE-ARM | npc.rs:Scarecrow1 | NPC.cs:72582 |
| 306 | Scarecrow2 | INLINE-ARM | npc.rs:Scarecrow2 | NPC.cs:72583 |
| 307 | Scarecrow3 | INLINE-ARM | npc.rs:Scarecrow3 | NPC.cs:72584 |
| 308 | Scarecrow4 | INLINE-ARM | npc.rs:Scarecrow4 | NPC.cs:72585 |
| 309 | Scarecrow5 | INLINE-ARM | npc.rs:Scarecrow5 | NPC.cs:72586 |
| 310 | Scarecrow6 | INLINE-ARM | npc.rs:Scarecrow6 | NPC.cs:68594 |
| 311 | Scarecrow7 | INLINE-ARM | npc.rs:Scarecrow7 | NPC.cs:68595 |
| 312 | Scarecrow8 | INLINE-ARM | npc.rs:Scarecrow8 | NPC.cs:68596 |
| 313 | Scarecrow9 | INLINE-ARM | npc.rs:Scarecrow9 | NPC.cs:68597 |
| 314 | Scarecrow10 | INLINE-ARM | npc.rs:Scarecrow10 | NPC.cs:68598 |
| 315 | HeadlessHorseman | INLINE-ARM | npc.rs:HeadlessHorseman | NPC.cs:74465 |
| 316 | Ghost | WIRED(d44 5568fabf) | `extra7` | NPC.cs:74834 |
| 317 | DemonEyeOwl | WIRED(d44 5568fabf) | `extra7` | NPC.cs:74003 |
| 318 | DemonEyeSpaceship | WIRED(d44 5568fabf) | `extra7` | NPC.cs:74004 |
| 319 | ZombieDoctor | INLINE-ARM | npc.rs:ZombieDoctor | NPC.cs:76338 |
| 320 | ZombieSuperman | INLINE-ARM | npc.rs:ZombieSuperman | NPC.cs:76339 |
| 321 | ZombiePixie | INLINE-ARM | npc.rs:ZombiePixie | NPC.cs:76340 |
| 322 | SkeletonTopHat | WIRED(d44 5568fabf) | `extra7` | NPC.cs:68599 |
| 323 | OnFire3 | WIRED(d44 5568fabf) | `extra7` | NPC.cs:68600 |
| 324 | Frostburn2 | WIRED(d44 5568fabf) | `extra7` | NPC.cs:68601 |
| 325 | MourningWood | INLINE-ARM | npc.rs:MourningWood | NPC.cs:72491 |
| 326 | Splinterling | INLINE-ARM | npc.rs:Splinterling | NPC.cs:68602 |
| 327 | Pumpking | INLINE-ARM | npc.rs:Pumpking | NPC.cs:72296 |
| 328 | PumpkingBlade | NO-CASE | — | — |
| 329 | Hellhound | INLINE-ARM | npc.rs:Hellhound | NPC.cs:74375 |
| 330 | Poltergeist | INLINE-ARM | npc.rs:Poltergeist | NPC.cs:74886 |
| 331 | ZombieXmas | INLINE-ARM | npc.rs:ZombieXmas | NPC.cs:76341 |
| 332 | ZombieSweater | INLINE-ARM | npc.rs:ZombieSweater | NPC.cs:76342 |
| 333 | SlimeRibbonWhite | WIRED(d44 5568fabf) | `extra7` | NPC.cs:70814 |
| 334 | SlimeRibbonYellow | WIRED(d44 5568fabf) | `extra7` | NPC.cs:70815 |
| 335 | SlimeRibbonGreen | WIRED(d44 5568fabf) | `extra7` | NPC.cs:70816 |
| 336 | SlimeRibbonRed | WIRED(d44 5568fabf) | `extra7` | NPC.cs:70817 |
| 337 | TentacleSpike | WIRED(d44 5568fabf) | `extra7` | NPC.cs:76854 |
| 338 | ZombieElf | INLINE-ARM | npc.rs:ZombieElf | NPC.cs:68603 |
| 339 | ZombieElfBeard | INLINE-ARM | npc.rs:ZombieElfBeard | NPC.cs:68604 |
| 340 | ZombieElfGirl | INLINE-ARM | npc.rs:ZombieElfGirl | NPC.cs:68605 |
| 341 | PresentMimic | INLINE-ARM | npc.rs:PresentMimic | NPC.cs:73444 |
| 342 | GingerbreadMan | INLINE-ARM | npc.rs:GingerbreadMan | NPC.cs:76343 |
| 343 | Yeti | INLINE-ARM | npc.rs:Yeti | NPC.cs:75779 |
| 344 | BloodButcherer | INLINE-ARM | npc.rs:Everscream | NPC.cs:72463 |
| 345 | IceQueen | INLINE-ARM | npc.rs:IceQueen | NPC.cs:72425 |
| 346 | SantaNK1 | INLINE-ARM | npc.rs:SantaNK1 | NPC.cs:72369 |
| 347 | ElfCopter | INLINE-ARM | npc.rs:ElfCopter | NPC.cs:72357 |
| 348 | Nutcracker | INLINE-ARM | npc.rs:Nutcracker | NPC.cs:76276 |
| 349 | NutcrackerSpinning | INLINE-ARM | npc.rs:NutcrackerSpinning | NPC.cs:76312 |
| 350 | ElfArcher | INLINE-ARM | npc.rs:ElfArcher | NPC.cs:76099 |
| 351 | Krampus | INLINE-ARM | npc.rs:Krampus | NPC.cs:75733 |
| 352 | Flocko | INLINE-ARM | npc.rs:Flocko | NPC.cs:72413 |
| 353 | Stylist | INLINE-ARM | npc.rs:Stylist | NPC.cs:68606 |
| 354 | WebbedStylist | NO-CASE | — | — |
| 355 | Firefly | WIRED(d44) | `c` | NPC.cs:72688 |
| 356 | Butterfly | WIRED(d44) | `c` | NPC.cs:72739 |
| 357 | Worm | WIRED(d44) | `c` | NPC.cs:72774 |
| 358 | LightningBug | WIRED(d44) | `c` | NPC.cs:72689 |
| 359 | Snail | WIRED(d44) | `c` | NPC.cs:72238 |
| 360 | GlowingSnail | WIRED(d44) | `c` | NPC.cs:72239 |
| 361 | Frog | WIRED(d44) | `c` | NPC.cs:76618 |
| 362 | Duck | WIRED(d44) | `c` | NPC.cs:73747 |
| 363 | Duck2 | WIRED(d44) | `c` | NPC.cs:73772 |
| 364 | DuckWhite | WIRED(d44) | `c` | NPC.cs:73748 |
| 365 | DuckWhite2 | WIRED(d44) | `c` | NPC.cs:73773 |
| 366 | ScorpionBlack | WIRED(g7 366|367 fold) | `high2` | NPC.cs:76820 |
| 367 | Scorpion | WIRED(g7) | `high2` | NPC.cs:76821 |
| 368 | TravellingMerchant | INLINE-ARM | npc.rs:TravellingMerchant | NPC.cs:68607 |
| 369 | Angler | INLINE-ARM | npc.rs:Angler | NPC.cs:68608 |
| 370 | DukeFishron | INLINE-ARM | npc.rs:DukeFishron | NPC.cs:72143 |
| 371 | DetonatingBubble | INLINE-ARM | npc.rs:DetonatingBubble(fold) | NPC.cs:72138 |
| 372 | Sharkron | INLINE-ARM | npc.rs:Sharkron | NPC.cs:72139 |
| 373 | Sharkron2 | INLINE-ARM | npc.rs:Sharkron2(fold) | NPC.cs:72140 |
| 374 | TruffleWorm | WIRED(g7) | `high2` | NPC.cs:71121 |
| 375 | Hemorrhage | WIRED(g7) | `high2` | NPC.cs:71109 |
| 376 | SleepingAngler | INLINE-ARM | npc.rs:SleepingAngler | NPC.cs:71007 |
| 377 | Grasshopper | WIRED(g7) | `high2` | NPC.cs:70958 |
| 378 | ChatteringTeethBomb | WIRED(g7) | `high2` | NPC.cs:70983 |
| 379 | CultistArcherBlue | WIRED(g7) | `high2` | NPC.cs:71578 |
| 380 | CultistArcherWhite | WIRED(g7) | `high2` | NPC.cs:71579 |
| 381 | BrainScrambler | INLINE-ARM | npc.rs:BrainScrambler | NPC.cs:71623 |
| 382 | RayGunner | INLINE-ARM | npc.rs:RayGunner | NPC.cs:71624 |
| 383 | MartianOfficer | INLINE-ARM | npc.rs:MartianOfficer | NPC.cs:71625 |
| 384 | ForceBubble | NO-CASE | — | — |
| 385 | GrayGrunt | INLINE-ARM | npc.rs:GrayGrunt | NPC.cs:71626 |
| 386 | MartianEngineer | INLINE-ARM | npc.rs:MartianEngineer | NPC.cs:71697 |
| 387 | MartianTurret | INLINE-ARM | npc.rs:MartianTurret | NPC.cs:71663 |
| 388 | MartianDrone | INLINE-ARM | npc.rs:MartianDrone | NPC.cs:71731 |
| 389 | GigaZapper | INLINE-ARM | npc.rs:GigaZapper | NPC.cs:71627 |
| 390 | ScutlixRider | INLINE-ARM | npc.rs:ScutlixRider | NPC.cs:71779 |
| 391 | Scutlix | INLINE-ARM | npc.rs:Scutlix | NPC.cs:71743 |
| 392 | MartianSaucer | INLINE-ARM | npc.rs:MartianSaucer | NPC.cs:71867 |
| 393 | MartianSaucerTurret | INLINE-ARM | npc.rs:MartianSaucerTurret | NPC.cs:71806 |
| 394 | MartianSaucerCannon | INLINE-ARM | npc.rs:MartianSaucerCannon | NPC.cs:71820 |
| 395 | PotentAcid | INLINE-ARM | npc.rs:MartianSaucerCore | NPC.cs:71845 |
| 396 | MoonLordHead | INLINE-ARM | npc.rs:MoonLordHead | NPC.cs:71950 |
| 397 | MoonLordHand | INLINE-ARM | npc.rs:MoonLordHand | NPC.cs:71966 |
| 398 | MoonLordCore | INLINE-ARM | npc.rs:MoonLordCore | NPC.cs:71922 |
| 399 | MartianProbe | INLINE-ARM | npc.rs:MartianProbe | NPC.cs:71885 |
| 400 | MoonLordFreeEye | INLINE-ARM | npc.rs:MoonLordFreeEye | NPC.cs:71972 |
| 401 | MoonLordLeechBlob | NO-CASE | — | — |
| 402 | StardustWormHead | NO-CASE | — | — |
| 403 | StardustWormBody | NO-CASE | — | — |
| 404 | StardustWormTail | NO-CASE | — | — |
| 405 | StardustCellBig | INLINE-ARM | npc.rs:StardustCellBig | NPC.cs:70728 |
| 406 | StardustCellSmall | INLINE-ARM | npc.rs:StardustCellSmall | NPC.cs:70729 |
| 407 | StardustJellyfishBig | INLINE-ARM | npc.rs:StardustJellyfishBig | NPC.cs:70717 |
| 408 | StardustJellyfishSmall | NO-CASE | — | — |
| 409 | StardustSpiderBig | INLINE-ARM | npc.rs:StardustSpiderBig | NPC.cs:70684 |
| 410 | StardustSpiderSmall | INLINE-ARM | npc.rs:StardustSpiderSmall | NPC.cs:70705 |
| 411 | StardustSoldier | INLINE-ARM | npc.rs:StardustSoldier | NPC.cs:70643 |
| 412 | SolarCrawltipedeHead | NO-CASE | — | — |
| 413 | SolarCrawltipedeBody | NO-CASE | — | — |
| 414 | SolarCrawltipedeTail | NO-CASE | — | — |
| 415 | SolarDrakomire | INLINE-ARM | npc.rs:SolarDrakomire | NPC.cs:70120 |
| 416 | SolarDrakomireRider | INLINE-ARM | npc.rs:SolarDrakomireRider | NPC.cs:70173 |
| 417 | SolarSroller | INLINE-ARM | npc.rs:SolarSroller | NPC.cs:70196 |
| 418 | SolarCorite | INLINE-ARM | npc.rs:SolarCorite | NPC.cs:70758 |
| 419 | SolarSolenian | INLINE-ARM | npc.rs:SolarSelenian | NPC.cs:70266 |
| 420 | NebulaBrain | INLINE-ARM | npc.rs:NebulaFloater | NPC.cs:70632 |
| 421 | NebulaHeadcrab | INLINE-ARM | npc.rs:BrainSuckler | NPC.cs:70578 |
| 422 | LunarTowerVortex | NO-CASE | — | — |
| 423 | NebulaBeast | INLINE-ARM | npc.rs:NebulaBeast | NPC.cs:70538 |
| 424 | NebulaSoldier | INLINE-ARM | npc.rs:Predictor | NPC.cs:70593 |
| 425 | VortexRifleman | INLINE-ARM | npc.rs:VortexRifleman | NPC.cs:70332 |
| 426 | VortexHornetQueen | INLINE-ARM | npc.rs:VortexHunter | NPC.cs:70411 |
| 427 | VortexHornet | INLINE-ARM | npc.rs:VortexAlien | NPC.cs:70374 |
| 428 | VortexLarva | INLINE-ARM | npc.rs:VortexHeadcrab | NPC.cs:70501 |
| 429 | VortexSoldier | INLINE-ARM | npc.rs:VortexSoldier | NPC.cs:70466 |
| 430 | ArmedZombie | WIRED(g7) | `high2` | NPC.cs:70879 |
| 431 | ArmedZombieEskimo | WIRED(g7) | `high2` | NPC.cs:70880 |
| 432 | ArmedZombiePincussion | WIRED(g7) | `high2` | NPC.cs:70881 |
| 433 | ArmedZombieSlimed | WIRED(g7) | `high2` | NPC.cs:70882 |
| 434 | ArmedZombieSwamp | WIRED(g7) | `high2` | NPC.cs:70883 |
| 435 | ArmedZombieTwiggy | **UNCLAIMED** | — | NPC.cs:70884 |
| 436 | ArmedZombieCenx | **UNCLAIMED** | — | NPC.cs:70885 |
| 437 | CultistTablet | WIRED(d44 5568fabf) | `extra7` | NPC.cs:71987 |
| 438 | CultistDevote | WIRED(d44 5568fabf) | `extra7` | NPC.cs:71998 |
| 439 | CultistBoss | INLINE-ARM | npc.rs:AncientCultist | NPC.cs:72017 |
| 440 | CultistBossClone | WIRED(d44 5568fabf) | `extra7` | NPC.cs:72018 |
| 441 | TaxCollector | INLINE-ARM | npc.rs:TaxCollector | NPC.cs:68609 |
| 442 | GoldBird | WIRED(d44 5568fabf) | `extra7` | NPC.cs:73631 |
| 443 | GoldBunny | WIRED(d44 44da5dc1) | `extra8` | NPC.cs:76855 |
| 444 | GoldButterfly | WIRED(d44 44da5dc1) | `extra8` | NPC.cs:72740 |
| 445 | GoldFrog | WIRED(d44 44da5dc1) | `extra8` | NPC.cs:76619 |
| 446 | GoldGrasshopper | WIRED(d44 44da5dc1) | `extra8` | NPC.cs:70959 |
| 447 | GoldMouse | WIRED(d44 44da5dc1) | `extra8` | NPC.cs:76514 |
| 448 | GoldWorm | WIRED(d44 44da5dc1) | `extra8` | NPC.cs:72775 |
| 449 | BoneThrowingSkeleton | WIRED(d44 44da5dc1) | `extra8` (d40 faithful twin) | NPC.cs:71398 |
| 450 | BoneThrowingSkeleton2 | WIRED(d44 44da5dc1) | `extra8` (d40 faithful twin) | NPC.cs:71399 |
| 451 | BoneThrowingSkeleton3 | WIRED(d44 44da5dc1) | `extra8` (d40 faithful twin) | NPC.cs:71400 |
| 452 | BoneThrowingSkeleton4 | WIRED(d44 44da5dc1) | `extra8` (d40 faithful twin) | NPC.cs:71401 |
| 453 | SkeletonMerchant | WIRED(d44 44da5dc1) | `extra8` (d40 town stand-in superseded) | NPC.cs:68610 |
| 454 | CultistDragonHead | WIRED(d44 44da5dc1) | `extra8` (d40 faithful twin) | NPC.cs:70952 |
| 455 | CultistDragonBody1 | NO-CASE | — | — |
| 456 | CultistDragonBody2 | NO-CASE | — | — |
| 457 | CultistDragonBody3 | NO-CASE | — | — |
| 458 | CultistDragonBody4 | NO-CASE | — | — |
| 459 | CultistDragonTail | NO-CASE | — | — |
| 460 | Butcher | WIRED(d44 44da5dc1) | `extra8` (d40 stand-in WRONG — no air tail) | NPC.cs:68611 |
| 461 | CreatureFromTheDeep | WIRED(d44 44da5dc1) | `extra8` (d40 faithful twin) | NPC.cs:75315 |
| 462 | Fritz | WIRED(d44 44da5dc1) | `extra8` (d40 stand-in WRONG — no air tail) | NPC.cs:68612 |
| 463 | Nailhead | INLINE-ARM | d40:Nailhead(npc.rs:9685) | NPC.cs:68613 |
| 464 | CrimsonBunny | INLINE-ARM | d40:CrimsonBunny(npc.rs:9691) | NPC.cs:76856 |
| 465 | CrimsonGoldfish | INLINE-ARM | d40:CrimsonGoldfish(npc.rs:9716) | NPC.cs:74186 |
| 466 | Psycho | INLINE-ARM | d40:Psycho(npc.rs:9732) | NPC.cs:75466 |
| 467 | DeadlySphere | INLINE-ARM | d40:DeadlySphere(npc.rs:9760) | NPC.cs:75069 |
| 468 | DrManFly | INLINE-ARM | d40:DrManFly(npc.rs:9764) | NPC.cs:71516 |
| 469 | ThePossessed | INLINE-ARM | d40:ThePossessed(npc.rs:9795) | NPC.cs:75665 |
| 470 | CrimsonPenguin | INLINE-ARM | d40:CrimsonPenguin(npc.rs:9832) | NPC.cs:76395 |
| 471 | GoblinSummoner | INLINE-ARM | npc.rs:GoblinSummoner | NPC.cs:75532 |
| 472 | ShadowFlameApparition | INLINE-ARM | d40:ShadowFlameApparition(npc.rs:9761) | NPC.cs:71369 |
| 473 | BigMimicCorruption | WIRED(d44 d1d20450) | `extra7b` | NPC.cs:71159 |
| 474 | BigMimicCrimson | WIRED(d44 d1d20450) | `extra7b` | NPC.cs:71160 |
| 475 | BigMimicHallow | WIRED(d44 d1d20450) | `extra7b` | NPC.cs:71161 |
| 476 | BigMimicJungle | WIRED(d44 d1d20450) | `extra7b` | NPC.cs:71162 |
| 477 | Mothron | WIRED(d44 d1d20450) | `extra7b` | NPC.cs:71330 |
| 478 | MothronEgg | NO-CASE | — | — |
| 479 | MothronSpawn | WIRED(d44 d1d20450) | `extra7b` | NPC.cs:71343 |
| 480 | Medusa | MODULE-LANDED(f26) KEEP-INLINE | `extra7b` parity / d40 inline wins | NPC.cs:75103 |
| 481 | GreekSkeleton | MODULE-LANDED(f26) KEEP-INLINE | `extra7b` parity / d40 inline wins | NPC.cs:71454 |
| 482 | GraniteGolem | MODULE-LANDED(f26) KEEP-INLINE | `extra7b` parity / d40 inline wins | NPC.cs:75202 |
| 483 | GraniteFlyer | WIRED(d44 d1d20450) | `extra7b` | NPC.cs:75168 |
| 484 | EnchantedNightcrawler | WIRED(d44 d1d20450) | `extra7b` | NPC.cs:72776 |
| 485 | Grubby | WIRED(d44 d1d20450) | `extra7b` | NPC.cs:72829 |
| 486 | Sluggy | WIRED(d44 d1d20450) | `extra7b` | NPC.cs:72830 |
| 487 | Buggy | WIRED(d44 d1d20450) | `extra7b` | NPC.cs:72831 |
| 488 | TargetDummy | WIRED(d44 d1d20450) | `extra7b` | NPC.cs:70847 |
| 489 | BloodZombie | INLINE-ARM | npc.rs:BloodZombie | NPC.cs:68614 |
| 490 | Drippler | INLINE-ARM | npc.rs:Drippler | NPC.cs:72521 |
| 491 | PirateShip | NO-CASE | — | — |
| 492 | PirateShipCannon | INLINE-ARM | npc.rs:FlyingDutchmanCannon | NPC.cs:71156 |
| 493 | LunarTowerStardust | NO-CASE | — | — |
| 494 | Crawdad | WIRED(d44 2a4ddf32) | `extra7c` (d41 faithful twin) | NPC.cs:69657 |
| 495 | Crawdad2 | WIRED(d44 2a4ddf32) | `extra7c` (d41 faithful twin) | NPC.cs:69658 |
| 496 | GiantShelly | WIRED(d44 2a4ddf32) | `extra7c` (d41 faithful twin) | NPC.cs:69994 |
| 497 | GiantShelly2 | WIRED(d44 2a4ddf32) | `extra7c` (d41 faithful twin) | NPC.cs:69995 |
| 498 | Salamander | WIRED(d44 2a4ddf32) | `extra7c` (d41 faithful twin) | NPC.cs:69849 |
| 499 | Salamander2 | WIRED(d44 2a4ddf32) | `extra7c` (d41 faithful twin) | NPC.cs:69850 |
| 500 | Salamander3 | WIRED(d44 2a4ddf32) | `extra7c` (d41 faithful twin) | NPC.cs:69851 |
| 501 | Salamander4 | WIRED(d44 2a4ddf32) | `extra7c` (d41 faithful twin) | NPC.cs:69852 |
| 502 | Salamander5 | WIRED(d44 2a4ddf32) | `extra7c` (d41 faithful twin) | NPC.cs:69853 |
| 503 | Salamander6 | WIRED(d44 2a4ddf32) | `extra7c` (d41 faithful twin) | NPC.cs:69854 |
| 504 | Salamander7 | WIRED(d44 2a4ddf32) | `extra7c` (d41 faithful twin) | NPC.cs:69855 |
| 505 | Salamander8 | WIRED(d44 2a4ddf32) | `extra7c` (d41 faithful twin) | NPC.cs:69856 |
| 506 | Salamander9 | WIRED(d44 2a4ddf32) | `extra7c` (d41 faithful twin) | NPC.cs:69857 |
| 507 | LunarTowerNebula | NO-CASE | — | — |
| 508 | GiantWalkingAntlion | WIRED(d44 2a4ddf32) | `extra7c` | NPC.cs:70067 |
| 509 | GiantFlyingAntlion | WIRED(d44 2a4ddf32) | `extra7c` | NPC.cs:70104 |
| 510 | DuneSplicerHead | NO-CASE | — | — |
| 511 | DuneSplicerBody | NO-CASE | — | — |
| 512 | DuneSplicerTail | NO-CASE | — | — |
| 513 | TombCrawlerHead | NO-CASE | — | — |
| 514 | TombCrawlerBody | NO-CASE | — | — |
| 515 | TombCrawlerTail | NO-CASE | — | — |
| 516 | SolarFlare | MODULE-LANDED(f26) — stale cell | `extra7d` (tip 7f38fd34) | NPC.cs:70740 |
| 517 | LunarTowerSolar | NO-CASE | — | — |
| 518 | SolarSpearman | INLINE-ARM | npc.rs:SolarSpearman | NPC.cs:70231 |
| 519 | SolarGoop | NO-CASE | — | — |
| 520 | MartianWalker | INLINE-ARM | npc.rs:MartianWalker | NPC.cs:71889 |
| 521 | AncientCultistSquidhead | MODULE-LANDED(f26) — stale cell | `extra7d` (tip 7f38fd34) | NPC.cs:72103 |
| 522 | AncientLight | NO-CASE | — | — |
| 523 | AncientDoom | MODULE-LANDED(f26) — stale cell | `extra7d` (tip 7f38fd34) | NPC.cs:72127 |
| 524 | DesertGhoul | MODULE-LANDED(f26) — stale cell | `extra7d` (tip 7f38fd34) | NPC.cs:69908 |
| 525 | DesertGhoulCorruption | MODULE-LANDED(f26) — stale cell | `extra7d` (tip 7f38fd34) | NPC.cs:69909 |
| 526 | DesertGhoulCrimson | MODULE-LANDED(f26) — stale cell | `extra7e` (tip 7f38fd34) | NPC.cs:69910 |
| 527 | DesertGhoulHallow | MODULE-LANDED(f26) — stale cell | `extra7e` (tip 7f38fd34) | NPC.cs:69911 |
| 528 | DesertLamiaLight | MODULE-LANDED(f26) — stale cell | `extra7e` (tip 7f38fd34) | NPC.cs:69946 |
| 529 | DesertLamiaDark | MODULE-LANDED(f26) — stale cell | `extra7e` (tip 7f38fd34) | NPC.cs:69947 |
| 530 | DesertScorpionWalk | MODULE-LANDED(f26) — stale cell | `extra7e` (tip 7f38fd34) | NPC.cs:73086 |
| 531 | DesertScorpionWall | MODULE-LANDED(f26+f27) TWIN — stale cell | `extra7f` / `extra10` | NPC.cs:73116 |
| 532 | DesertBeast | MODULE-LANDED(f26+f27) TWIN — stale cell | `extra7f` / `extra10` | NPC.cs:70068 |
| 533 | DesertDjinn | INLINE-ARM | npc.rs:DesertDjinn | NPC.cs:69978 |
| 534 | DemonTaxCollector | MODULE-LANDED(f26+f27) TWIN — stale cell | `extra7f` / `extra10` | NPC.cs:68615 |
| 535 | SlimeSpiked | MODULE-LANDED(f26+f27) TWIN — stale cell | `extra7f` / `extra10` | NPC.cs:70818 |
| 536 | TheBride | MODULE-LANDED(f26+f27) TWIN — stale cell | `extra7f` / `extra10` | NPC.cs:76344 |
| 537 | SandSlime | MODULE-LANDED(f27) — stale cell | `extra10` (tip 1a24a133) | NPC.cs:70819 |
| 538 | SquirrelRed | MODULE-LANDED(f27) — stale cell | `extra10` (tip 1a24a133) | NPC.cs:76459 |
| 539 | SquirrelGold | MODULE-LANDED(f27) — stale cell | `extra10` (tip 1a24a133) | NPC.cs:76460 |
| 540 | PartyBunny | MODULE-LANDED(f27) — stale cell | `extra10` (tip 1a24a133) | NPC.cs:76857 |
| 541 | SandElemental | MODULE-LANDED(f27) — stale cell | `extra10` (tip 1a24a133) | NPC.cs:69734 |
| 542 | SandShark | MODULE-LANDED(f27) — stale cell | `extra10` (tip 1a24a133) | NPC.cs:74681 |
| 543 | SandsharkCorrupt | MODULE-LANDED(f27) — stale cell | `extra10` (tip 1a24a133) | NPC.cs:74682 |
| 544 | SandsharkCrimson | MODULE-LANDED(f27) — stale cell | `extra10` (tip 1a24a133) | NPC.cs:74683 |
| 545 | SandsharkHallow | MODULE-LANDED(f27) — stale cell | `extra10` (tip 1a24a133) | NPC.cs:74684 |
| 546 | Tumbleweed | NO-CASE | — | — |
| 547 | DD2AttackerTest | NO-CASE | — | — |
| 548 | DD2EterniaCrystal | NO-CASE | — | — |
| 549 | DD2LanePortal | INLINE-ARM | npc.rs:DD2LanePortal | NPC.cs:67980 |
| 550 | DD2Bartender | INLINE-ARM | npc.rs:DD2Bartender | NPC.cs:68616 |
| 551 | DD2Betsy | INLINE-ARM | npc.rs:DD2Betsy | NPC.cs:67258 |
| 552 | DD2GoblinT1 | INLINE-ARM | npc.rs:DD2GoblinT1 | NPC.cs:67452 |
| 553 | DD2GoblinT2 | INLINE-ARM | npc.rs:DD2GoblinT2 | NPC.cs:67453 |
| 554 | DD2GoblinT3 | INLINE-ARM | npc.rs:DD2GoblinT3 | NPC.cs:67454 |
| 555 | DD2GoblinBomberT1 | INLINE-ARM | npc.rs:DD2GoblinBomberT1 | NPC.cs:67797 |
| 556 | DD2GoblinBomberT2 | INLINE-ARM | npc.rs:DD2GoblinBomberT2 | NPC.cs:67798 |
| 557 | DD2GoblinBomberT3 | INLINE-ARM | npc.rs:DD2GoblinBomberT3 | NPC.cs:67799 |
| 558 | DD2WyvernT1 | INLINE-ARM | npc.rs:DD2WyvernT1 | NPC.cs:67838 |
| 559 | DD2WyvernT2 | INLINE-ARM | npc.rs:DD2WyvernT2 | NPC.cs:67839 |
| 560 | DD2WyvernT3 | INLINE-ARM | npc.rs:DD2WyvernT3 | NPC.cs:67840 |
| 561 | DD2JavelinstT1 | INLINE-ARM | npc.rs:DD2JavelinT1 | NPC.cs:67756 |
| 562 | DD2JavelinstT2 | INLINE-ARM | npc.rs:DD2JavelinT2 | NPC.cs:67757 |
| 563 | DD2JavelinstT3 | INLINE-ARM | npc.rs:DD2JavelinT3 | NPC.cs:67758 |
| 564 | DD2DarkMageT1 | INLINE-ARM | npc.rs:DD2DarkMage | NPC.cs:67998 |
| 565 | DD2DarkMageT3 | INLINE-ARM | npc.rs:DD2DarkMageT3 | NPC.cs:67999 |
| 566 | DD2SkeletonT1 | INLINE-ARM | npc.rs:DD2SkeletonT1 | NPC.cs:67559 |
| 567 | DD2SkeletonT3 | INLINE-ARM | npc.rs:DD2SkeletonT3 | NPC.cs:67560 |
| 568 | DD2WitherBeastT2 | INLINE-ARM | npc.rs:DD2WitherBeast | NPC.cs:67025 |
| 569 | DD2WitherBeastT3 | INLINE-ARM | npc.rs:DD2WitherBeastT3 | NPC.cs:67026 |
| 570 | DD2DrakinT2 | INLINE-ARM | npc.rs:DD2DrakinT2 | NPC.cs:67711 |
| 571 | DD2DrakinT3 | INLINE-ARM | npc.rs:DD2DrakinT3 | NPC.cs:67712 |
| 572 | DD2KoboldWalkerT2 | INLINE-ARM | npc.rs:DD2KoboldWalkerT2 | NPC.cs:67493 |
| 573 | DD2KoboldWalkerT3 | INLINE-ARM | npc.rs:DD2KoboldWalkerT3 | NPC.cs:67494 |
| 574 | DD2KoboldFlyerT2 | INLINE-ARM | npc.rs:DD2KoboldFlyerT2 | NPC.cs:67887 |
| 575 | DD2KoboldFlyerT3 | INLINE-ARM | npc.rs:DD2KoboldFlyerT3 | NPC.cs:67888 |
| 576 | DD2OgreT2 | INLINE-ARM | npc.rs:DD2OgreT2 | NPC.cs:67584 |
| 577 | DD2OgreT3 | INLINE-ARM | npc.rs:DD2OgreT3 | NPC.cs:67585 |
| 578 | DD2LightningBugT3 | **UNCLAIMED** | — | NPC.cs:67875 |
| 579 | BartenderUnconscious | NO-CASE | — | — |
| 580 | WalkingAntlion | **UNCLAIMED** | — | NPC.cs:70069 |
| 581 | FlyingAntlion | **UNCLAIMED** | — | NPC.cs:70105 |
| 582 | LarvaeAntlion | **UNCLAIMED** | — | NPC.cs:70070 |
| 583 | FairyCritterPink | **UNCLAIMED** | — | NPC.cs:69835 |
| 584 | FairyCritterGreen | **UNCLAIMED** | — | NPC.cs:69836 |
| 585 | FairyCritterBlue | **UNCLAIMED** | — | NPC.cs:69837 |
| 586 | ZombieMerman | **UNCLAIMED** | — | NPC.cs:75411 |
| 587 | EyeballFlyingFish | **UNCLAIMED** | — | NPC.cs:74764 |
| 588 | Golfer | INLINE-ARM | npc.rs:Golfer | NPC.cs:68617 |
| 589 | GolferRescue | INLINE-ARM | npc.rs:SleepingGolfer | NPC.cs:71019 |
| 590 | TorchZombie | **UNCLAIMED** | — | NPC.cs:76345 |
| 591 | ArmedTorchZombie | **UNCLAIMED** | — | NPC.cs:70886 |
| 592 | GoldGoldfish | **UNCLAIMED** | — | NPC.cs:74187 |
| 593 | GoldGoldfishWalker | **UNCLAIMED** | — | NPC.cs:75923 |
| 594 | WindyBalloon | **UNCLAIMED** | — | NPC.cs:67993 |
| 595 | BlackDragonfly | **UNCLAIMED** | — | NPC.cs:72722 |
| 596 | BlueDragonfly | **UNCLAIMED** | — | NPC.cs:72723 |
| 597 | GreenDragonfly | **UNCLAIMED** | — | NPC.cs:72724 |
| 598 | OrangeDragonfly | **UNCLAIMED** | — | NPC.cs:72725 |
| 599 | RedDragonfly | **UNCLAIMED** | — | NPC.cs:72726 |
| 600 | YellowDragonfly | **UNCLAIMED** | — | NPC.cs:72727 |
| 601 | GoldDragonfly | **UNCLAIMED** | — | NPC.cs:72728 |
| 602 | Seagull | **UNCLAIMED** | — | NPC.cs:73819 |
| 603 | Seagull2 | **UNCLAIMED** | — | NPC.cs:73842 |
| 604 | LadyBug | **UNCLAIMED** | — | NPC.cs:66961 |
| 605 | GoldLadyBug | **UNCLAIMED** | — | NPC.cs:66962 |
| 606 | Maggot | **UNCLAIMED** | — | NPC.cs:72777 |
| 607 | Pupfish | **UNCLAIMED** | — | NPC.cs:74188 |
| 608 | Grebe | **UNCLAIMED** | — | NPC.cs:73749 |
| 609 | Grebe2 | **UNCLAIMED** | — | NPC.cs:73774 |
| 610 | Rat | **UNCLAIMED** | — | NPC.cs:76566 |
| 611 | Owl | **UNCLAIMED** | — | NPC.cs:73681 |
| 612 | WaterStrider | **UNCLAIMED** | — | NPC.cs:69773 |
| 613 | GoldWaterStrider | **UNCLAIMED** | — | NPC.cs:69774 |
| 614 | ExplosiveBunny | **UNCLAIMED** | — | NPC.cs:76858 |
| 615 | Dolphin | **UNCLAIMED** | — | NPC.cs:74227 |
| 616 | Turtle | **UNCLAIMED** | — | NPC.cs:76686 |
| 617 | TurtleJungle | **UNCLAIMED** | — | NPC.cs:76687 |
| 618 | BloodNautilus | **UNCLAIMED** | — | NPC.cs:67000 |
| 619 | BloodSquid | **UNCLAIMED** | — | NPC.cs:67011 |
| 620 | GoblinShark | **UNCLAIMED** | — | NPC.cs:68423 |
| 621 | BloodEelHead | NO-CASE | — | — |
| 622 | BloodEelBody | NO-CASE | — | — |
| 623 | BloodEelTail | NO-CASE | — | — |
| 624 | Gnome | **UNCLAIMED** | — | NPC.cs:77067 |
| 625 | SeaTurtle | **UNCLAIMED** | — | NPC.cs:76732 |
| 626 | Seahorse | **UNCLAIMED** | — | NPC.cs:76780 |
| 627 | GoldSeahorse | **UNCLAIMED** | — | NPC.cs:76781 |
| 628 | Dandelion | **UNCLAIMED** | — | NPC.cs:77103 |
| 629 | IceMimic | **UNCLAIMED** | — | NPC.cs:73445 |
| 630 | BloodMummy | INLINE-ARM | waveB:BloodMummy | NPC.cs:68618 |
| 631 | RockGolem | **UNCLAIMED** | — | NPC.cs:75259 |
| 632 | MaggotZombie | **UNCLAIMED** | — | NPC.cs:68619 |
| 633 | BestiaryGirl | INLINE-ARM | npc.rs:Zoologist | NPC.cs:68620 |
| 634 | SporeBat | **UNCLAIMED** | — | NPC.cs:74835 |
| 635 | SporeSkeleton | **UNCLAIMED** | — | NPC.cs:68621 |
| 636 | HallowBoss | **UNCLAIMED** | — | NPC.cs:67022 |
| 637 | TownCat | **UNCLAIMED** | — | NPC.cs:68622 |
| 638 | TownDog | **UNCLAIMED** | — | NPC.cs:68623 |
| 639 | GemSquirrelAmethyst | **UNCLAIMED** | — | NPC.cs:76461 |
| 640 | GemSquirrelTopaz | **UNCLAIMED** | — | NPC.cs:76462 |
| 641 | GemSquirrelSapphire | **UNCLAIMED** | — | NPC.cs:76463 |
| 642 | GemSquirrelEmerald | **UNCLAIMED** | — | NPC.cs:76464 |
| 643 | GemSquirrelRuby | **UNCLAIMED** | — | NPC.cs:76465 |
| 644 | GemSquirrelDiamond | **UNCLAIMED** | — | NPC.cs:76466 |
| 645 | GemSquirrelAmber | **UNCLAIMED** | — | NPC.cs:76467 |
| 646 | GemBunnyAmethyst | **UNCLAIMED** | — | NPC.cs:76859 |
| 647 | GemBunnyTopaz | **UNCLAIMED** | — | NPC.cs:76860 |
| 648 | GemBunnySapphire | **UNCLAIMED** | — | NPC.cs:76861 |
| 649 | GemBunnyEmerald | **UNCLAIMED** | — | NPC.cs:76862 |
| 650 | GemBunnyRuby | **UNCLAIMED** | — | NPC.cs:76863 |
| 651 | GemBunnyDiamond | **UNCLAIMED** | — | NPC.cs:76864 |
| 652 | GemBunnyAmber | **UNCLAIMED** | — | NPC.cs:76865 |
| 653 | HellButterfly | **UNCLAIMED** | — | NPC.cs:72741 |
| 654 | Lavafly | **UNCLAIMED** | — | NPC.cs:72690 |
| 655 | MagmaSnail | **UNCLAIMED** | — | NPC.cs:72240 |
| 656 | TownBunny | **UNCLAIMED** | — | NPC.cs:68624 |
| 657 | QueenSlimeBoss | **UNCLAIMED** | — | NPC.cs:66848 |
| 658 | QueenSlimeMinionBlue | **UNCLAIMED** | — | NPC.cs:70820 |
| 659 | QueenSlimeMinionPink | **UNCLAIMED** | — | NPC.cs:70821 |
| 660 | QueenSlimeMinionPurple | **UNCLAIMED** | — | NPC.cs:74005 |
| 661 | EmpressButterfly | **UNCLAIMED** | — | NPC.cs:72742 |
| 662 | PirateGhost | **UNCLAIMED** | — | NPC.cs:74836 |
| 663 | Princess | INLINE-ARM | npc.rs:Princess | NPC.cs:68625 |
| 664 | TorchGod | **UNCLAIMED** | — | NPC.cs:74837 |
| 665 | ChaosBallTim | NO-CASE | — | — |
| 666 | VileSpitEaterOfWorlds | NO-CASE | — | — |
| 667 | GoldenSlime | **UNCLAIMED** | — | NPC.cs:70822 |
| 668 | Deerclops | INLINE-ARM | npc.rs:Deerclops | NPC.cs:66779 |
| 669 | Stinkbug | **UNCLAIMED** | — | NPC.cs:66981 |
| 670 | TownSlimeBlue | **UNCLAIMED** | — | NPC.cs:68626 |
| 671 | ScarletMacaw | **UNCLAIMED** | — | NPC.cs:73654 |
| 672 | BlueMacaw | **UNCLAIMED** | — | NPC.cs:73655 |
| 673 | Toucan | **UNCLAIMED** | — | NPC.cs:73656 |
| 674 | YellowCockatiel | **UNCLAIMED** | — | NPC.cs:73657 |
| 675 | GrayCockatiel | **UNCLAIMED** | — | NPC.cs:73658 |
| 676 | ShimmerSlime | **UNCLAIMED** | — | NPC.cs:70823 |
| 677 | Shimmerfly | **UNCLAIMED** | — | NPC.cs:72710 |
| 678 | TownSlimeGreen | **UNCLAIMED** | — | NPC.cs:68627 |
| 679 | TownSlimeOld | **UNCLAIMED** | — | NPC.cs:68628 |
| 680 | TownSlimePurple | **UNCLAIMED** | — | NPC.cs:68629 |
| 681 | TownSlimeRainbow | **UNCLAIMED** | — | NPC.cs:68630 |
| 682 | TownSlimeRed | **UNCLAIMED** | — | NPC.cs:68631 |
| 683 | TownSlimeYellow | **UNCLAIMED** | — | NPC.cs:68632 |
| 684 | TownSlimeCopper | **UNCLAIMED** | — | NPC.cs:68633 |
| 685 | BoundTownSlimeOld | NO-CASE | — | — |
| 686 | BoundTownSlimePurple | NO-CASE | — | — |
| 687 | BoundTownSlimeYellow | **UNCLAIMED** | — | NPC.cs:76620 |
| 688 | Pufferfish | **UNCLAIMED** | — | NPC.cs:74244 |
| 689 | OwlMimic | **UNCLAIMED** | — | NPC.cs:73682 |
| 690 | StatueMimic | NO-CASE | — | — |
| 691 | MossZombie | **UNCLAIMED** | — | NPC.cs:76346 |
| 692 | Orca | **UNCLAIMED** | — | NPC.cs:74693 |
| 693 | LibrarianSkeleton | **UNCLAIMED** | — | NPC.cs:77029 |
| 694 | WaterBoltMimic | **UNCLAIMED** | — | NPC.cs:77202 |
| 695 | PalworldCattivaDistressed | **UNCLAIMED** | — | NPC.cs:66707 |
| 696 | PalworldFoxsparksDistressed | **UNCLAIMED** | — | NPC.cs:66721 |
| 697 | (NPCID.Count sentinel, no kind) | NO-CASE | — | — |

## 3. Per-module bridge shapes + the keep-one union plan

### 3.0 The FindFrame-family declaration (refresh 5, the census master)

The full module family at the probe-time pushed tips (the merger's
compose list; every seam fn mechanically re-grepped at the cited tip):

| lane / tip | modules | seam / hook status |
| --- | --- | --- |
| d44 `f4a7074bc` (origin probe) | `low1` + `c` + `extra5`/`extra6`/`extra7`/`extra7b`/`extra7c`/`extra8` + the wave-B residual (`npc_findframe_b.rs`) | the EIGHT wire blocks + the residual hook — the cs_id chain c → 5 → 6 → 7 → 7b → 7c → 8 → **wave-b** (`find_frame_kind_b` claims {294,296} ONLY, `npc.rs:7008`/`:7249`) |
| d45 `aea0aebd` / d46 `055fcf8c` | `low2` / `mid1` | self-wired |
| d47 `3d36f76c` / d48 `3eb2ee08` / d49 `dadfd828` | `mid2` / `c` / `high2` | g6 / **d44** / g7-wired |
| e13 `9fbca588` / e14 `63bc2c9f` | `extra1` / `extra2` | extra1 DROP (all twins); extra2 g6+g7 |
| f26 `a2e43ef77` | `extra3`/`extra5`/`extra6`/`extra7b`/`extra7c` + `extra7d`/`7e`/`7f`/`7g`/`7h`/`7i`/`7j`/`7k` | the 7g-7k seams `find_frame_kind_extra7g` (`:160`) / `7h` (`:198`) / `7i` (`:128`) / `7j` (`:184`) / `7k` (`:182`), each paired `find_frame_extraNN(kind, &mut FrameState, &FrameInput)` — UNWIRED, the next wire rounds |
| f27 `af5d54b61` | `extra4`/`extra7`/`extra8`/`extra9` + `extra10`/`extra11`/`extra12`/`extra13` | the seams `find_frame_kind_extra10` (`:209`) / `11` (`:175`) / `12` (`:83`) / `13` (`:105`); the extra11/12/13 bridges `find_frame_extra11` (`npc.rs:6602`) / `12` (`:6630`) / `13` (`:6659`) SELF-WIRED on f27's own npc.rs (the v1-arm FIX sets) |
| g10 `5dee2865` | `npc_findframe_pet.rs` | `town_pet_find_frame` — the 11 town pets 637/638/656/670/678-684, wired on g10's own npc.rs |

**Merge-compose: the d38 kinds + the d44 hooks.** The d38 roster
(`b888a9575`) brings the 281-350 `NpcKind` variants + their bestiary
ENTRIES rows; the d44 cs_id hooks precede the roster arms at merge
(first-match-wins, §1). For 294/296 the S1-F-3 `net_id` fallback
resolves through the d38 bestiary rows — NO hand cs_id rows needed;
the d38-side stand-in drops (`npc.rs:9514-9515`/`:9518-9519` at
`b888a9575`) + the FINAL-witness re-pin are the d38-merge edits (§7d).

The bridges differ per module — carry each module's own bridge verbatim
at the union (the `find_frame_wave_a` copy-out/tick/copy-back precedent):

| module | FrameState (copy both ways) | FrameInput (feed in) | world read |
| --- | --- | --- | --- |
| `low1` | row/counter/sprite_dir/rotation | vx/vy/facing/wet/ai | no |
| `low2` | row/counter/sprite_dir/rotation | vx/vy/facing/wet | no |
| `mid1` | row/counter/sprite_dir/rotation | vx/vy/facing/wet | no |
| `mid2` | row/counter/sprite_dir/rotation | vx/vy/facing + local_ai0 INPUT-only | no |
| `c` | row/counter/sprite_dir/rotation/local_ai0 | vx/vy/facing/wet/ai2/local_ai2 + **slope** | YES (worm probe) |
| `high2` | row/counter/sprite_dir/rotation/local_ai0 | vx/vy/facing/wet/ai2 | no |
| `extra1` | row/counter/sprite_dir/rotation/local_ai0 | vx/vy/facing/wet/ai | no — DROP (all twins) |
| `extra2` | row/counter/sprite_dir/rotation/local_ai0 (both ways) | vx/vy/facing/wet/ai | no |
| `extra3` | row/counter/sprite_dir/rotation | vx/vy/facing/wet/ai + ai_action | no |
| `extra4` | row/counter/sprite_dir/rotation | vx/vy/facing/wet/ai/ai_action | no |
| `extra5` | row/counter/sprite_dir (MINIMAL) | vx/vy/facing ONLY | no |
| `extra6` | row/counter/sprite_dir | vx/vy/facing/ai[4] | no |
| `extra7` | row/counter/sprite_dir/rotation | vx/vy/facing/ai/ai_action/local_ai2 | no |
| `extra7b` | row/counter/sprite_dir/rotation/local_ai[0..1] | vx/vy/facing/ai + **slope/frame_x36** | YES (crawler/grub/dummy) |
| `extra7c` | row/counter/sprite_dir/rotation | vx/vy/facing/ai[4] | no — **WIRED(d44 refresh 4)** |
| `extra8` | row/counter/sprite_dir/rotation/local_ai0 | vx/vy/facing/wet/ai + **slope** | YES (GoldWorm probe) — **WIRED(d44 refresh 4)** |
| `waveB` (the residual claim) | row/counter/sprite_dir/rotation | vx/vy/facing/wet/ai[4] (the fighter pair reads vx/vy/facing only) | no — **WIRED(d44 refresh 5, 294/296 ONLY; 289 + the other 22 kinds NEVER)** |

The g6-vs-g7 twin-hook union (unchanged from refresh 1, still valid):
1. **extra2 twin: KEEP g7's trimmed 7-id mapper** (`86505082`) — g6's
   15-id mapper double-claims the 8 d45-owned ids; the arms are verbatim
   identical (kept in the module as parity references).
2. **extra3 twin: identical machines** (comments-only 9-line diff); keep
   g7's later doc state.
3. **e13 extra1: DROP the module + hook** — all 15 ids twinned onto
   extra2/extra3, arm-by-arm verified (g7's keep-one ruling).
4. **366 scorpion fold: g7 owns it** (`find_frame_kind_high2` maps
   `366 | 367`); the d44 c-module hook claims 355-365 ONLY.
5. **The roster-lane keep-one (NEW, refresh 2):** where a MODULE-LANDED
   machine and a roster inline arm both exist, the faithful module wins
   at merge (the hook precedes the body match). The f26 slice-7c
   addendum's arm-by-arm verdict: **480 Medusa / 481 GreekSkeleton /
   482 GraniteGolem are FAITHFUL on BOTH sides — keep the d40 INLINE
   arms** (the module arms drop to parity references); the twelve d40
   stub kinds 473-477/479/483-488 keep the extra7b MODULE arms. The
   f27 slice-8 verdict: the d40 443-448 row-0 stubs are superseded by
   the extra8 machines; the d40 453/460/462 `humanoid_town_frame`
   stand-in is WRONG for the fighters (460/462) — the extra8 fighter
   delegates win; 453's town wrap is right but the module is the
   faithful town machine. The d38 287/295/322/323/324 town stand-ins
   are superseded by the extra6/extra7 fighter delegates.
   **Refresh 4 (the extra7c/extra8 wire):** the d41 tip `5d036721`
   lands FAITHFUL inline arms for 494-506 (arm-by-arm equal to the
   extra7c machines — the crawdad `npc.rs:7515`, the shelly `:7418`,
   the salamander `:7465`): the 13 ids are faithful-inline TWIN
   claims, the f26e 480/481/482 precedent — behaviour-identical
   either way, the d44 hook precedes at merge (no exclusion needed;
   the merger may keep the d41 inline arms and drop the extra7c
   claims for those 13, the 508/509 module claims stand alone). The
   extra8 claim needs NO exclusions: the d40-faithful SIX
   464/465/466/468/469/470 (the f27 slice-9 FAITHFUL-EXCLUDE
   verdicts) are NOT in the extra8 mapper — verified id-for-id
   against the pushed blob `9b526e3a`; the d40 449-452/454/461
   faithful inline twins keep one either way.
6. **The 439 stub-fix** rides f27's own `npc.rs` (the AncientCultist
   1-frame pin → the extra7 CultistBossClone ladder delegate, the
   `case 439/440` shared pose ladder `NPC.cs:72017-72101`); the towers
   422/493/507/517 keep their row-0 pin (no FindFrame case).

## 4. d45/d46/d48 branch-wired status (explicit)

* **d45**: SELF-WIRED on its own pushed branch (`aea0aebd`). No action.
* **d46**: SELF-WIRED on its own pushed branch (`055fcf8c`). No action.
* **d48**: NOT branch-wired; **WIRED(d44)** on this branch (§7, the
  355-365-only hook + the worm slope world-read bridge). RESOLVED.

## 5. UNWIRED list — the MODULE-LANDED worklist (the wire rounds)

**TRULY EMPTY at refresh 5.** The only MODULE-LANDED-unwired ids on
the d44-landed modules are the extra7b keep-inline trio — ALL
RESOLVED keep-inline, ZERO real wire work left on ANY module landed
on this branch (the wave-B residual 294/296 was the last pair, and
the refresh-4 "ZERO real wire work" verdict silently omitted it —
the d38 research/125 §11 task seed caught the gap, now closed):

| module | ids | count | bridge shape | wire note |
| --- | --- | --- | --- | --- |
| `extra7b` keep-inline trio | 480/481/482 | 3 | — | RESOLVED, NOT wiring work: the f26e verdict keeps the d40 INLINE arms (faithful both sides); the d44 hook NEVER claims them (`d1d20450`), the module arms ride as parity references |

**DONE refresh 2 (§7):** `extra6` (278-287/290-293/295, 15 ids) —
WIRED(d44 `59b798dc`).

**DONE refresh 3 (§7b):** `extra7` (316-318/322-324/333-337/437/438/
440/442, 15 ids) — WIRED(d44 `5568fabf`); `extra7b`'s 12 d40-stub kinds
(473-477/479/483-488) — WIRED(d44 `d1d20450`, the world-read
slope/frame_x36 extension landed with it).

**DONE refresh 4 (§7c):** `extra7c` (494-506/508/509, 15 ids) —
WIRED(d44 `2a4ddf32e`, the four-field world-free bridge; the d41
494-506 faithful-inline twins noted §3.5); `extra8` (443-454/460-462,
15 ids) — WIRED(d44 `44da5dc19`, the five-field/local_ai0 bridge + the
GoldWorm slope probe reusing `find_frame_worm_slope` through the
ALREADY-EXTENDED `find_frame_world` entry; NO exclusions — the
d40-faithful six were never in the mapper).

**DONE refresh 5 (§7d):** the wave-B residual pair (294/296, 2 ids) —
WIRED(d44 `f4a7074bc`, the d38 SEAL-RECEIPT research/293 §14 exact
hunks: the `find_frame_kind_b` in-module mapper + the four-field
`find_frame_wave_b` bridge + the cs_id hook at the END of the
world-free chain; NO `find_frame_world` arm — the fall-through rides;
289 EXCLUDED per E3; the verification batch `925423511` adds the
frame-count diff + 3 exclusion/inertness witnesses).

The NEXT wire rounds (when their rounds come): the f26 7d/7e/7f/7g/
7h/7i/7j/7k modules + the f27 extra10/extra12 modules (the f26/f27
tips are IN FLIGHT — the refresh-5 probe-time claim sets are in the
vintage note; the extra11 552-563 FIXES are already self-wired on
f27's own npc.rs). The 7d SolarFlare needs the two RESOLVED draw
seams (the f26f-516 precedent this lane followed for slope/frame_x36).

## 6. UNCLAIMED worklist (the port rounds) — 130 kinds (refresh-2 vintage; ALL covered on some pushed tip at refresh 5)

No arm and no module claim on ANY pushed tip **at refresh-2 scan
time**. The refresh-5 re-probe (the vintage note): 53 of the 130 are
module-claimed at the probe-time f26/f27 tips (77 remain by
module-claim arithmetic) and ALL 77 carry inline roster arms (38 d42
+ 31 d43 + 8 c4/e26/c8) — **ZERO true no-coverage ids**. The rows
below are stale until the next full mechanical refresh; the work
they name is now KEEP-ONE/faithfulness adjudication (the 13 d43 row-0
stubs 637/638/656/670/678-684 + the d42 arm-quality verdicts), not
port-from-scratch. The C# anchor is the first top-level `case` line
of the id's group (the group body serves every listed sibling).
Contiguous runs (refresh-2 vintage):

435-436 (2), 516 (1), 521 (1), 523-532 (10), 534-545 (12), 578 (1),
580-587 (8), 590-620 (31), 624-629 (6), 631-632 (2), 634-662 (29),
664 (1), 667 (1), 669-684 (16), 687-689 (3), 691-696 (6).

| id | kind | C# anchor |
| --- | --- | --- |
| 435 | ArmedZombieTwiggy | `NPC.cs:70884` |
| 436 | ArmedZombieCenx | `NPC.cs:70885` |
| 516 | SolarFlare | `NPC.cs:70740` |
| 521 | AncientCultistSquidhead | `NPC.cs:72103` |
| 523 | AncientDoom | `NPC.cs:72127` |
| 524 | DesertGhoul | `NPC.cs:69908` |
| 525 | DesertGhoulCorruption | `NPC.cs:69909` |
| 526 | DesertGhoulCrimson | `NPC.cs:69910` |
| 527 | DesertGhoulHallow | `NPC.cs:69911` |
| 528 | DesertLamiaLight | `NPC.cs:69946` |
| 529 | DesertLamiaDark | `NPC.cs:69947` |
| 530 | DesertScorpionWalk | `NPC.cs:73086` |
| 531 | DesertScorpionWall | `NPC.cs:73116` |
| 532 | DesertBeast | `NPC.cs:70068` |
| 534 | DemonTaxCollector | `NPC.cs:68615` |
| 535 | SlimeSpiked | `NPC.cs:70818` |
| 536 | TheBride | `NPC.cs:76344` |
| 537 | SandSlime | `NPC.cs:70819` |
| 538 | SquirrelRed | `NPC.cs:76459` |
| 539 | SquirrelGold | `NPC.cs:76460` |
| 540 | PartyBunny | `NPC.cs:76857` |
| 541 | SandElemental | `NPC.cs:69734` |
| 542 | SandShark | `NPC.cs:74681` |
| 543 | SandsharkCorrupt | `NPC.cs:74682` |
| 544 | SandsharkCrimson | `NPC.cs:74683` |
| 545 | SandsharkHallow | `NPC.cs:74684` |
| 578 | DD2LightningBugT3 | `NPC.cs:67875` |
| 580 | WalkingAntlion | `NPC.cs:70069` |
| 581 | FlyingAntlion | `NPC.cs:70105` |
| 582 | LarvaeAntlion | `NPC.cs:70070` |
| 583 | FairyCritterPink | `NPC.cs:69835` |
| 584 | FairyCritterGreen | `NPC.cs:69836` |
| 585 | FairyCritterBlue | `NPC.cs:69837` |
| 586 | ZombieMerman | `NPC.cs:75411` |
| 587 | EyeballFlyingFish | `NPC.cs:74764` |
| 590 | TorchZombie | `NPC.cs:76345` |
| 591 | ArmedTorchZombie | `NPC.cs:70886` |
| 592 | GoldGoldfish | `NPC.cs:74187` |
| 593 | GoldGoldfishWalker | `NPC.cs:75923` |
| 594 | WindyBalloon | `NPC.cs:67993` |
| 595 | BlackDragonfly | `NPC.cs:72722` |
| 596 | BlueDragonfly | `NPC.cs:72723` |
| 597 | GreenDragonfly | `NPC.cs:72724` |
| 598 | OrangeDragonfly | `NPC.cs:72725` |
| 599 | RedDragonfly | `NPC.cs:72726` |
| 600 | YellowDragonfly | `NPC.cs:72727` |
| 601 | GoldDragonfly | `NPC.cs:72728` |
| 602 | Seagull | `NPC.cs:73819` |
| 603 | Seagull2 | `NPC.cs:73842` |
| 604 | LadyBug | `NPC.cs:66961` |
| 605 | GoldLadyBug | `NPC.cs:66962` |
| 606 | Maggot | `NPC.cs:72777` |
| 607 | Pupfish | `NPC.cs:74188` |
| 608 | Grebe | `NPC.cs:73749` |
| 609 | Grebe2 | `NPC.cs:73774` |
| 610 | Rat | `NPC.cs:76566` |
| 611 | Owl | `NPC.cs:73681` |
| 612 | WaterStrider | `NPC.cs:69773` |
| 613 | GoldWaterStrider | `NPC.cs:69774` |
| 614 | ExplosiveBunny | `NPC.cs:76858` |
| 615 | Dolphin | `NPC.cs:74227` |
| 616 | Turtle | `NPC.cs:76686` |
| 617 | TurtleJungle | `NPC.cs:76687` |
| 618 | BloodNautilus | `NPC.cs:67000` |
| 619 | BloodSquid | `NPC.cs:67011` |
| 620 | GoblinShark | `NPC.cs:68423` |
| 624 | Gnome | `NPC.cs:77067` |
| 625 | SeaTurtle | `NPC.cs:76732` |
| 626 | Seahorse | `NPC.cs:76780` |
| 627 | GoldSeahorse | `NPC.cs:76781` |
| 628 | Dandelion | `NPC.cs:77103` |
| 629 | IceMimic | `NPC.cs:73445` |
| 631 | RockGolem | `NPC.cs:75259` |
| 632 | MaggotZombie | `NPC.cs:68619` |
| 634 | SporeBat | `NPC.cs:74835` |
| 635 | SporeSkeleton | `NPC.cs:68621` |
| 636 | HallowBoss | `NPC.cs:67022` |
| 637 | TownCat | `NPC.cs:68622` |
| 638 | TownDog | `NPC.cs:68623` |
| 639 | GemSquirrelAmethyst | `NPC.cs:76461` |
| 640 | GemSquirrelTopaz | `NPC.cs:76462` |
| 641 | GemSquirrelSapphire | `NPC.cs:76463` |
| 642 | GemSquirrelEmerald | `NPC.cs:76464` |
| 643 | GemSquirrelRuby | `NPC.cs:76465` |
| 644 | GemSquirrelDiamond | `NPC.cs:76466` |
| 645 | GemSquirrelAmber | `NPC.cs:76467` |
| 646 | GemBunnyAmethyst | `NPC.cs:76859` |
| 647 | GemBunnyTopaz | `NPC.cs:76860` |
| 648 | GemBunnySapphire | `NPC.cs:76861` |
| 649 | GemBunnyEmerald | `NPC.cs:76862` |
| 650 | GemBunnyRuby | `NPC.cs:76863` |
| 651 | GemBunnyDiamond | `NPC.cs:76864` |
| 652 | GemBunnyAmber | `NPC.cs:76865` |
| 653 | HellButterfly | `NPC.cs:72741` |
| 654 | Lavafly | `NPC.cs:72690` |
| 655 | MagmaSnail | `NPC.cs:72240` |
| 656 | TownBunny | `NPC.cs:68624` |
| 657 | QueenSlimeBoss | `NPC.cs:66848` |
| 658 | QueenSlimeMinionBlue | `NPC.cs:70820` |
| 659 | QueenSlimeMinionPink | `NPC.cs:70821` |
| 660 | QueenSlimeMinionPurple | `NPC.cs:74005` |
| 661 | EmpressButterfly | `NPC.cs:72742` |
| 662 | PirateGhost | `NPC.cs:74836` |
| 664 | TorchGod | `NPC.cs:74837` |
| 667 | GoldenSlime | `NPC.cs:70822` |
| 669 | Stinkbug | `NPC.cs:66981` |
| 670 | TownSlimeBlue | `NPC.cs:68626` |
| 671 | ScarletMacaw | `NPC.cs:73654` |
| 672 | BlueMacaw | `NPC.cs:73655` |
| 673 | Toucan | `NPC.cs:73656` |
| 674 | YellowCockatiel | `NPC.cs:73657` |
| 675 | GrayCockatiel | `NPC.cs:73658` |
| 676 | ShimmerSlime | `NPC.cs:70823` |
| 677 | Shimmerfly | `NPC.cs:72710` |
| 678 | TownSlimeGreen | `NPC.cs:68627` |
| 679 | TownSlimeOld | `NPC.cs:68628` |
| 680 | TownSlimePurple | `NPC.cs:68629` |
| 681 | TownSlimeRainbow | `NPC.cs:68630` |
| 682 | TownSlimeRed | `NPC.cs:68631` |
| 683 | TownSlimeYellow | `NPC.cs:68632` |
| 684 | TownSlimeCopper | `NPC.cs:68633` |
| 687 | BoundTownSlimeYellow | `NPC.cs:76620` |
| 688 | Pufferfish | `NPC.cs:74244` |
| 689 | OwlMimic | `NPC.cs:73682` |
| 691 | MossZombie | `NPC.cs:76346` |
| 692 | Orca | `NPC.cs:74693` |
| 693 | LibrarianSkeleton | `NPC.cs:77029` |
| 694 | WaterBoltMimic | `NPC.cs:77202` |
| 695 | PalworldCattivaDistressed | `NPC.cs:66707` |
| 696 | PalworldFoxsparksDistressed | `NPC.cs:66721` |

Group notes (the keep-one families that shrink the port count):
* **435/436/591** ride the d49 armed-zombie group body (`:70879-70951`)
  — the d49 machine serves them; a mapper-widen, not a new machine.
* **524-527** the four Desert Ghouls share one body (`:69908-69945`);
  **528/529** the two Lamias (`:69946-70066`); **530/531** the desert
  scorpion walk/wall pair (the 164/165 crawler groups).
* **538/539 + 639-645** the squirrel family (`:76459-76512`); **540 +
  646-652 + 614** the bunny hop group (`:76851-76904`); **595-601** the
  seven dragonflies share one body (`:72722-72738`); **602/603** the
  seagull pair (the duck walk/swimmer groups); **604/605** the ladybug
  pair; **606** Maggot joins the 357/448/484 crawler group; **608/609**
  the grebe pair (the duck group); **616/617** the turtle pair (the
  frog group `:76686-76731`); **626/627** the seahorse pair; **637/638/
  656/670/678-684** the town pets + town slimes (the humanoid/slime
  groups); **653/661** the butterfly flap group (356/444); **654** the
  firefly group (355/358); **655** the snail group (359/360); **658-660**
  the Queen Slime minions (the slime/eye groups); **662/664** the
  floater group (316/253); **667/676** the slime hop group; **671-675**
  the bird group (74/297/298/442); **691** joins the 590 TorchZombie
  walker group (`:76345`); **693** the caster pose group (281-286).
* The Palworld crossover kinds **695/696** (`:66707`/`:66721`) are the
  switch's first two cases — small dedicated arms.

## 7. Wire-round addendum (2026-11-21 refresh 2, this branch)

**f26 slice 6 (`extra6`) — WIRED(d44)** on `task/W4-d44-findframe-low-1`:
* `npc_findframe_extra6.rs` copied byte-exact from `a2e47596` (blob
  `38ee6c0f` hash-verified, `git hash-object` == the pushed tip's blob).
  The 15 machines + the in-module `find_frame_kind_extra6` lookup + 11
  frame witnesses ride unchanged. Commit `9d6baa89`.
* The wave-B `humanoid_fighter_frame` `pub(crate)` lift (the f26/f27
  keep-one delegate precedent — the BoneLee/AngryBonesBigMuscle fighter
  walk + the SkeletonCommando archer delegate ride it). Commit `9d6baa89`.
* The hook `find_frame_kind_extra6_claimed` = the in-module lookup
  verbatim (all 15 ids 278-287/290-293/295). The bridge
  `find_frame_wave_extra6` copies row/counter/sprite_dir both ways and
  feeds vx/vy/facing/ai[4] — no slice-6 arm reads wet/aiAction or writes
  rotation (the f26 wave-6 census), so no world read is needed and the
  hook sits in the world-free `find_frame` directly. Batches `21628a0d`
  (the first 11: 278-287+290) + `59b798dc` (the last 4: 291/292/293/295).
* Stream-inert: no current kind's cs_id lands in 278-287/290-293/295
  (hand rows + bestiary net ids swept over ALL_KINDS), no arm draws RNG
  (the f26 mechanical grep over all eight cited ranges = ZERO).
* **Witnesses**: 3 seam tests in `npc.rs::w4_d44_wire_tests` (the
  inertness sweep over ALL_KINDS, the full 15-id claim set with the
  288/289/294/296 port-covered exclusions, the bridge field copy-backs
  with per-machine frame pins — the armored-bones walk/idle/air, the
  caster pose group, the BoneLee fighter delegate, the Paladin hammer
  ladder + strict->8 wrap, the sniper draw/walk/air, the commando
  delegate, the big-muscle air pin) + the module's own 11 frame
  witnesses ride verbatim. Written not run.

The refresh-1 wire round (d48 wave C + f26 slice 5) is unchanged — see
the §7 history in the prior tip `7cb3f8d0` (the worm slope world-read
bridge at both production call sites, the minimal extra5 bridge).

## 7b. Wire-round addendum (2026-11-21 refresh 3, this branch)

**f27 slice 7 (`extra7`) — WIRED(d44)** on `task/W4-d44-findframe-low-1`:
* `npc_findframe_extra7.rs` copied byte-exact from `0e187428` (the f27
  tip moved past the census's `72448094` — extra9 rides it; blob
  `a41abca3` hash-verified, `git hash-object` == the pushed tip's blob,
  the committed blob re-verified via `git ls-tree`). The 15 machines +
  the in-module `find_frame_kind_extra7` lookup + 11 frame witnesses
  ride unchanged. Commit `86791530`. The wave-B
  `humanoid_fighter_frame` was ALREADY `pub(crate)` (the `9d6baa89`
  extra6 lift) — the 322/323/324 costume-skeleton delegate rides it,
  no new lift.
* The hook `find_frame_kind_extra7_claimed` = the in-module lookup
  verbatim (all 15 ids). The bridge `find_frame_wave_extra7` copies
  row/counter/sprite_dir/rotation both ways and feeds
  vx/vy/facing/ai[4]/ai_action/local_ai[2] — no slice-7 arm reads wet
  or the world (the floater draw `:74844` is 634-gated, the eye dust
  `:74007` is 23-gated), so the hook sits in the world-free
  `find_frame` directly. Batches `a72b09ee` (the first 11: the
  316-337 band) + `5568fabf` (the last 4: 437/438/440/442).
* The 439 boss keeps its own inline arm on THIS branch — the f27
  `d959c234` stub-fix (the 439/440 shared ladder delegate) rides f27's
  own `npc.rs`, not this wire round; at merge the f27 fix + this
  hook's 440 claim are the same machine (keep-one trivially).

**f26 slice 7b (`extra7b`) — WIRED(d44), the 12 stub kinds ONLY**:
* `npc_findframe_extra7b.rs` copied byte-exact from `f1a0eaa9` (the
  f26 tip moved past the census's `a2e47596` — extra7d/7e ride it;
  blob `3729faa4` hash-verified equal). The 15 machines + lookup + 12
  frame witnesses ride unchanged. Commit `96877ed2`.
* The hook `find_frame_kind_extra7b_claimed` = the module mapper
  MINUS the f26e keep-inline trio **480 Medusa / 481 GreekSkeleton /
  482 GraniteGolem — NEVER claimed** (the d40 INLINE arms are
  FAITHFUL both sides; the module arms ride as parity references).
  The 12 d40-stub kinds 473-477/479/483-488 all claimed. Batches
  `0a19a13a` (the first 11) + `d1d20450` (the last 1: 488).
* The bridge `find_frame_wave_extra7b` copies
  row/counter/sprite_dir/rotation/local_ai[0..1] both ways and feeds
  vx/vy/facing/ai[4] + the two RESOLVED world reads. The
  `find_frame_world` entry EXTENDED (the f26f-516 resolved-seams
  precedent): the crawler/grub kinds 484-487 reuse
  `find_frame_worm_slope` (the SAME two-tile probe,
  `NPC.cs:72807-72815`/`:72862-72869`); the NEW
  `find_frame_dummy_frame_x36` resolves
  `Framing.GetTileSafely((int)ai[0], (int)ai[1]).frameX >= 36`
  (`:70850`; out-of-world → `new Tile()` frameX 0 → false,
  `Framing.cs:426-439` + `Tile.cs:101`). The world-free `find_frame`
  lane passes slope 0 / frame_x36 false (the wave-C convention).
  Both production call sites already ride `find_frame_world`
  (`npc.rs` update_rng tail + `world_sim/npcs.rs` slot loop) — no
  call-site change needed.
* Stream-inert: a mechanical Python scan of the cs_id hand rows +
  the bestiary net-id literals + the loot_db constants over both
  claim bands = ZERO hits (the lone 475 token in loot_db is a
  comment cite). No arm draws RNG (the f26/f27 mechanical greps).
* **Witnesses**: 8 seam tests added to `npc.rs::w4_d44_wire_tests`
  (the extra7/extra7b inertness sweeps over ALL_KINDS, the two
  claim sets — the extra7b one pinning the 480/481/482 NEVER-claim —
  the two bridge field copy-backs with per-machine frame pins, the
  dummy frameX probe, the extra7b world-entry probe resolution) +
  the modules' own 11 + 12 frame witnesses ride verbatim. Written
  not run.

## 7c. Wire-round addendum (2026-11-21 refresh 4, this branch)

**f26 slice 7c (`extra7c`) — WIRED(d44), all 15 ids**:
* `npc_findframe_extra7c.rs` copied byte-exact from `7f38fd34` (the
  f26 tip moved past the census's `f1a0eaa9` — extra7f rides it; blob
  `658e5366` hash-verified, `git hash-object` == the pushed tip's blob,
  the committed blob re-verified via `git ls-tree HEAD`). The 15
  machines + the in-module `find_frame_kind_extra7c` lookup + 10 frame
  witnesses ride unchanged. Commit `ca88aaf0`.
* The hook `find_frame_kind_extra7c_claimed` = the in-module lookup
  verbatim (all 15 ids 494-506/508/509). The bridge
  `find_frame_wave_extra7c` copies row/counter/sprite_dir/rotation
  both ways and feeds vx/vy/facing/ai[4] — NO world read this wave
  (the mechanical grep over the five cited C# ranges
  `:69657-69733`/`:69994-70066`/`:69849-69907`/`:70067-70103`/
  `:70104-70119` = ZERO Framing/Main.tile/rand), so the hook rides
  the world-free `find_frame` directly; the 7b slope/frame_x36 seams
  are NOT carried. Batches `532e234a` (the first 11: 494-504) +
  `2a4ddf32` (the last 4: 505/506/508/509).
* **The d41 twin note**: the d41 tip ADVANCED `4ed22f0b`→`5d036721` —
  its batches 1-2 land FAITHFUL inline arms for 494-506 (the crawdad
  charge bands `npc.rs:7515`, the shelly 12-row state machine
  `:7418`, the salamander walk cycle `:7465` — arm-by-arm equal to
  the extra7c machines, read via `git show`). The 13 ids are
  faithful-inline TWIN claims (the f26e 480/481/482 precedent); the
  hook precedes the roster arm at merge and the machines are
  behaviour-identical — no exclusion needed (§3.5). 508/509 have NO
  roster kind on any tip.

**f27 slice 8 (`extra8`) — WIRED(d44), all 15 ids, NO exclusions**:
* `npc_findframe_extra8.rs` copied byte-exact from `1a24a133` (the
  f27 tip moved past the census's `0e187428` — extra10 + the 549 fix
  ride it; blob `9b526e3a` hash-verified equal). The 15 machines +
  lookup + 15 frame witnesses ride unchanged. Commit `4ba3c815`.
* **The claim-set verification** (the brief's exclusion question):
  the extra8 mapper carries EXACTLY 443-454/460/461/462 — the
  d40-inline-faithful SIX **464/465/466/468/469/470 are NOT in it**
  (the f27 slice-9 re-census FAITHFUL-EXCLUDE verdicts, re-read from
  the pushed ANALYSIS; verified id-for-id against the blob). The
  claim = the in-module lookup verbatim, NO exclusions. The d40
  443-448 row-0 stubs keep-MODULE (superseded); the 449-452/454/461
  arms are FAITHFUL inline twins (keep-one either way, the hook
  precedes at merge); the 453/460/462 `humanoid_town_frame`
  stand-ins keep-MODULE (460/462 WRONG — no air tail; 453 right only
  with the 26/9 count rows the module carries internally).
* The hook `find_frame_kind_extra8_claimed` = the in-module lookup
  verbatim. The bridge `find_frame_wave_extra8` copies
  row/counter/sprite_dir/rotation/local_ai0 both ways and feeds
  vx/vy/facing/wet/ai[4] + the RESOLVED slope. The `find_frame_world`
  entry EXTENDED (the extra7b precedent): the GoldWorm (448) REUSES
  `find_frame_worm_slope` (the SAME two-tile probe,
  `NPC.cs:72807-72826` — NOT the dummy frame_x36 probe, that is
  TargetDummy-only); the world-free lane passes slope 0. Both
  production call sites already ride `find_frame_world` — no
  call-site change. Batches `9a20da59` (the first 11: 443-453) +
  `44da5dc1` (the last 4: 454/460/461/462).
* Stream-inert: a mechanical Python scan of the cs_id hand-row match
  arms (57 ids, the `cs_id` fn `data.rs:2992-3108`) + the bestiary
  net-id literals (241 ids) over both claim bands = ZERO hits. No
  arm draws RNG (the f26 7c / f27 slice-8 mechanical greps; the town
  pose zones are the documented skip).
* **Witnesses**: 9 seam tests added to `npc.rs::w4_d44_wire_tests`
  (the extra7c/extra8 inertness sweeps over ALL_KINDS, the two claim
  sets — the extra8 one pinning the d40-faithful-six NEVER-reach —
  the two bridge field copy-backs with per-machine frame pins, the
  extra7c tail-batch bridge, the extra8 tail-batch bridge with the
  creature aim-ease values re-derived mechanically, the GoldWorm
  world-entry probe resolution) + the modules' own 10 + 15 frame
  witnesses ride verbatim. Written not run.

## 7d. Wire-round addendum (2026-11-21 refresh 5, this branch)

**The wave-B residual pair — the LAST 2 findframe stand-ins.** The
d38 RETIRE census (research/125 §7, now renumbered 293) named the
7-town-stand-in gap list; refresh 3/4 hook-wired 5 of 7 (287/295
extra6 + 322/323/324 extra7) and the refresh-4 §5 "ZERO real wire
work" verdict did NOT cover the residual 294/296 — the wave-B module
(`npc_findframe_b.rs`, on this branch since S1-D-3) had NO cs_id
mapper and NO hook on ANY pushed tip. The d38 SEAL-RECEIPT
(research/293 §14, tip `72b1092e7`) published the exact 3-hunk seed;
the d44g-wire-final dispatch executed it at `ce2c3a24a` →
**`f4a7074bc`** (pushed):

1. `npc_findframe_b.rs` — the `find_frame_kind_b` in-module mapper
   {294 → AngryBonesBig, 296 → AngryBonesBigHelmet}, appended after
   `find_frame_b` (the `find_frame_kind_extra6` lookup shape). 289
   EXCLUDED per the E3 dedupe (the d38 inline arm is the kept twin);
   the other 22 module kinds await their roster lanes.
2. `npc.rs` — the `find_frame_wave_b` FOUR-field bridge beside
   `find_frame_wave_extra6` (row/counter/sprite_dir/rotation both
   ways — rotation carried though the fighter delegate never writes
   it, the 75/82/122 arms do; inputs vx/vy/facing/wet/ai[4], NO
   ai_action — the module header's skip).
3. `npc.rs` — the `find_frame_kind_b_claimed` wrapper beside
   `extra8_claimed` + the cs_id hook at the END of the world-free
   chain (c → 5 → 6 → 7 → 7b → 7c → 8 → **wave-b**, before the num2
   body). NO `find_frame_world` arm — the fall-through rides (the
   extra5/6/7 precedent).

**The cs_id hand-row answer (the §14 receipt): NO new hand rows.**
`NpcKind::AngryBonesBig`/`BigHelmet` do NOT exist on this tip — they
arrive with the d38 merge, where the S1-F-3 fallback
(`npc/data.rs`'s `try_entry_for_npc(...).net_id`) resolves both
through the d38 bestiary ENTRIES rows (net_id 294 `bestiary.rs:2390`
/ 296 `:2410` at d38 `b888a9575`, mechanically re-read). Hand rows
referencing the absent variants would not compile; the optional
freeze-pin is the merger's choice, not required.

**Faithfulness (re-read at source this round):** the case labels
`NPC.cs:68591/:68593`; the fighter group `:68647-69655` delegates
with `air_row0 = true` for BOTH (the `:69647` pin list carries
294/296), `idle_row1 = false` (the `:69510-69514` quirk is 140/489
only), `num57 = 0` (neither `isLikeATownNPC`, `:6856-6866` — 453 +
the town flag), the |vx|*2 THEN ++ accumulator (`:69550-69553`), the
>6 stride (`:69576`), the wrap at count back to num109 = row 2
(`:69581-69584`). Frame counts: the mechanical 697-slot
`Main.cs:67082-67153` text-parse vs the port `NPC_FRAME_COUNT`
(`render/entities.rs:1002`) = **ZERO mismatches** (294 = 16 `:67113`
slot 4, 296 = 15 slot 6; the module `frame_count` rows agree). The
d38 `humanoid_town_frame` stand-in has the SAME walk band but NO
grounded flip and NO air tail — the gap this hook closes at merge.

**The stand-in drop (the d38-merge edit, re-derived at the CURRENT
d38 tip `b888a9575`):** the two comment+arm pairs
`npc.rs:9514-9515` (294) / `:9518-9519` (296) — UNCHANGED line
numbers vs the §11 recipe (the SEAL-RECEIPT/registry rounds touched
docs only). With the five §11 drops (`:9326-9331` the 322/323/324
trio, `:9516-9517` the 295, `:9531-9532` the 287) that is ALL SEVEN
stand-ins gone; the `humanoid_town_frame` helper STAYS (the town
kinds ride it). The FINAL witness
`w4_d38_final_the_seven_town_standins_are_the_wiring_gap`
(`npc.rs:35137-35178` at `b888a9575`) re-pins at the same edit: the
seven-kind list EMPTIES (every stand-in assertion would fail against
the module machines — deliberately); it either retires or flips to
pin the MODULE behavior through the hook. NOTHING to drop on THIS
tip — the d38 kinds (and their stand-in arms) do not exist here
(mechanically verified: zero `NpcKind::AngryBonesBig*` hits in this
branch's npc.rs).

**Stream-inertness:** the mechanical sweep of the cs_id hand rows
(57 ids) + the bestiary net-id literals (241) + the 8 loot_db consts
= ZERO 294/296 hits on this tip; the fighter delegate draws no RNG
(the town `ai[0]` pose zones `:68662-69488` — the group's only
rand/dust sites — are the documented skip, unreachable for these
aiStyle-3 fighters) and reads no world.

**Witnesses (WRITTEN NOT RUN):** the `f4a7074bc` seam witness
`w4_d44_the_wave_b_residual_pair_bridge` (the {294,296} claim set +
the 289/293/295/297 exclusions, the grounded flip + the 7>6 step, the
FULL-16/FULL-15 wraps, the air pin 0 BOTH, the rotation copy-back,
the idle row 0) + the verification batch `925423511`'s 3:
`the_wiring_seam_maps_only_the_two_residual_fighter_ids` (the module
mapper's full 24-id exclusion sweep),
`w4_d44_the_wave_b_hook_is_inert_for_every_current_kind` (the
ALL_KINDS inertness sweep) and
`w4_d44_the_wave_b_claim_excludes_the_other_module_kinds` (the 23-id
complement). Every vector re-derived by a mechanical f32 Python
replay of `humanoid_fighter_frame` (all 7 pinned vectors match).

**The refresh-5 census side-notes (mechanically re-verified this
round):**
* **The RESOLVED trio 480/481/482 needs NO wire — FAITHFUL inline
  confirmed at the current d40 tip:** the d40 `8fe1d6d3` npc.rs
  carries real per-kind arms `NpcKind::Medusa` (`:9865`),
  `GreekSkeleton` (`:9902`), `GraniteGolem` (`:9935`) — NOT
  `humanoid_town_frame` stand-ins. The f26e keep-inline verdict
  stands: the extra7b module arms ride as parity references, the
  d44 hook NEVER claims the trio. ZERO wire work, ever.
* **The 634 SporeBat RNG-seam contract is PUBLISHED** in the f26
  extra7h module header (`npc_findframe_extra7h.rs:100-133` at
  `a2e43ef77`, re-read this round): the bridge draws `Next(10)` EVERY
  tick for type 634 ONLY (`NPC.cs:74844`, the `&&` short-circuits the
  other floater members), feeds the RESOLVED `spore_dust_draw_is_zero`
  boolean (the 7d resolved-draw precedent), the `Dust.NewDust` 165
  spawn + velocity*0.1 + noGravity (`:74846-74848`) a bridge-side
  effect on the same boolean; the post-seam machine is zero-RNG (the
  vx-sign gates, rotation = vx*0.1, the >=6 step, the count-1 flap
  wrap `:74867-74870`). 634 stays module-UNCLAIMED until that
  seam-fed wave (the d44f lane); its d43 inline arm (`npc.rs:9636`
  at `6fbe0625`) is the interim coverage — arm-quality adjudication
  pending, the RNG draw is NOT in it.
* **The unclaimed count at the CURRENT tips: ZERO true no-coverage**
  (the vintage note arithmetic). The in-flight subtractions all
  verified: f26 7g-7k claim 435/436/631/632/635/636/639-655/657-662
  (the 661 7j/7k twin, `2ba235a17`); f27 s11/s12/s13 fix-or-claim
  550-563/551/566/567/570/571 (the extra13 `af5d54b61` round landed
  mid-census — the s13 seams + bridges grep-verified); d36 E4
  (`8c5839b03`) lands FindFrame cases 173/176/205 — all three were
  ALREADY WIRED(g6), zero count change; d41 batches 3-7
  (`065efdf97`) add the 508-537 roster arms INSIDE the f26 7c/7d/7e/
  7f claim bands (twin coverage, zero new ids); g10 (`5dee2865`,
  unchanged) owns the 11 town pets. The 20-id high-band residual
  (634/664/667/669/671-677/687-689/691-696) ALL carry d43 inline
  arms (`npc.rs:9636-10999` at `6fbe0625`, arm-by-arm grep-verified:
  SporeBat :9636, TorchGod :10432, GoldenSlime :10465, Stinkbug
  :10487, the macaws/cockatiels :10510-10598, ShimmerSlime :10620,
  Shimmerfly :10642, BoundTownSlimeYellow :10705, Pufferfish
  :10751, OwlMimic :10788, MossZombie :10846, Orca :10847,
  LibrarianSkeleton :10899, WaterBoltMimic :10919, the Palworld
  pair :10988/:10999).

## 8. Remaining-work estimate

* **UNCLAIMED kinds: ZERO true no-coverage ids at refresh 5** (the
  vintage note — the refresh-2 130 are ALL covered on some pushed
  tip: 53 module-claimed at the probe-time f26/f27 tips + 77 roster
  arms [38 d42 + 31 d43 + 8 c4/e26/c8]). The remaining PORT work is
  faithfulness adjudication: the 13 d43 `{}` row-0 stubs
  (637/638/656/670/678-684) + the d42 arm-quality verdicts — NOT
  port-from-scratch slices. The next full mechanical refresh
  re-derives §2/§6 with the arm-quality column.
* **MODULE-LANDED-but-UNWIRED on THIS branch: ZERO** (§5 — the
  wave-B residual closed the last pair; the extra7b keep-inline trio
  480/481/482 is RESOLVED, no hook ever). The NEXT wire rounds are
  the f26 7d/7e/7f/7g/7h/7i/7j/7k + f27 extra10/extra12 modules (the
  lanes are IN FLIGHT — their claim sets grow; the extra11 552-563
  FIXES are already self-wired on f27's own npc.rs) — at the ≤11-kind
  cap when their rounds come (the 7d SolarFlare needs the two
  RESOLVED draw seams — the f26f-516 precedent this lane followed
  for slope/frame_x36).
* **Total remaining: the faithfulness adjudication + the in-flight
  modules' wire batches.** The merger lands g6 ∪ g7 (with §3's
  keep-one resolutions) + the d45/d46 self-wired branches + THIS
  branch's EIGHT wire blocks (d48 c + extra5 + extra6 + extra7 +
  extra7b + extra7c + extra8 + wave-b) + the roster lanes
  d34/d35/d37/d38/d40/d41/d42/d43; the d38 merge applies the §7d
  seven-stand-in drop + the FINAL-witness re-pin.

## 9. Verification performed

Code-only census — no build/test/lint (per lane rules). Evidence
commands: `git ls-remote origin` (all tips re-probed), `git fetch origin
--prune`, `git show origin/<branch>:crates/tr-app/src/npc_findframe_*.rs`
+ `npc.rs` + `npc/data.rs` + `bestiary.rs` (every per-kind row read from
the pushed mappers/arms/cs_id tables), `git cat-file blob 38ee6c0f`
(the extra6 verbatim landing, hash-verified), a Python mechanical scan
(the C# two-tab case labels + the per-lane find_frame arm scan + the
cs_id/bestiary/fold mapping + the module mapper join — the §2 table is
its output, not hand-counted). The extra6 wire witnesses are WRITTEN NOT
RUN. All reads were direct file paths / git object reads; no junction
recursion, ≤100-line windows.

Refresh 3 (the extra7/extra7b wire round): `git cat-file blob
a41abca3`/`3729faa4` extracted via cmd redirection (byte-exact, no
PowerShell encoding mangling) + `git hash-object` == the pushed tips'
blobs + `git ls-tree HEAD` re-verification of the COMMITTED blobs; a
Python mechanical scan of the cs_id hand-row match arms + the
bestiary/loot_db literals over both claim bands (ZERO hits — the lone
475 token is a comment cite); the C# probe sites re-read
(`NPC.cs:70847-70878` the dummy, `:72805-72826`/`:72856-72883` the
crawler/grub slope, `Framing.cs:426-439` + `Tile.cs:101` the
new-Tile defaults); every seam-witness expectation hand-re-derived
against the module arms (the mimic state ladder, the mothron length
stepper, the granite tilt modes, the grub slope latch, the dummy
recoil banks, the cultist pose ladder, the devote mirror). The 8 new
seam witnesses are WRITTEN NOT RUN — **no cargo/rustc/tests/rustfmt/
graphify invoked** (code-only mode).

Refresh 4 (the extra7c/extra8 wire round): `git cat-file blob
658e5366`/`9b526e3a` extracted via cmd redirection (byte-exact, no
PowerShell encoding mangling) + `git hash-object` == the pushed tips'
blobs (`7f38fd34`/`1a24a133`) + `git ls-tree HEAD` re-verification of
the COMMITTED blobs; a Python mechanical scan of the cs_id hand-row
match arms (the `cs_id` fn `data.rs:2992-3108`, 57 ids) + the bestiary
net-id literals (241 ids) over both claim bands (ZERO hits — an
earlier naive whole-file scan's 450/500 tokens were `max_hp` rows,
not cs_id); the C# arms re-read line-for-line
(`NPC.cs:69657-69733`/`:69994-70066`/`:69849-69907`/`:70067-70119`
the 7c five ranges — ZERO Framing/Main.tile/rand;
`:72774-72828` the worm-group slope probe — the ONLY world read in
the extra8 ranges, the two `Framing.GetTileSafely` lines;
`:75315-75360` the creature wet arm); the d41 tip's 494-506 inline
arms read via `git show` (arm-by-arm equal to the extra7c machines);
the f27 slice-8/9 ANALYSIS verdicts re-read from the pushed tip (the
464/465/466/468/469/470 FAITHFUL-EXCLUDE table — none in the extra8
mapper); every seam-witness expectation hand-re-derived against the
module arms (the crawdad 39 reset, the shelly 24 reset-tick hold, the
salamander >7 wrap, the antlion sign gates, the bunny full-7 wrap,
the butterfly no-bank pin, the frog deadbands, the bone-toss
literal-15 wrap + the 452 count variant, the merchant town-17 wrap,
the creature entry quirk + the aim-ease values atan2(4,3)/5 =
0.18545904 / the mirror tick −0.21454096 re-derived in Python, the
fritz >9 stride + air pair). The 9 new seam witnesses are WRITTEN NOT
RUN — **no cargo/rustc/tests/rustfmt/graphify invoked** (code-only
mode).

Refresh 5 (the wave-B residual wire round): the d38 SEAL-RECEIPT
research/293 §14 exact hunks re-read from the pushed tip `b888a9575`
(the §14 seed + the bestiary ENTRIES rows :2390/:2410 + the stand-in
arm windows :9514-9515/:9518-9519 + the FINAL witness :35137-35178
all re-verified via `git show`); the C# fighter premises re-read at
source (`NPC.cs:68591/:68593` the case labels, `:69647` the air-pin-0
list, `:69510-69519` the idle branches, `:6856-6866` isLikeATownNPC,
`:69550-69584` the walk accumulator/stride/wrap); the mechanical
697-slot frame-count diff (`Main.cs:67082-67153` text-parse vs
`NPC_FRAME_COUNT` = ZERO mismatches); the stream-inertness sweep (57
cs_id hand rows + 241 bestiary net-id literals + 8 loot_db consts =
ZERO 294/296 hits); the census re-probe (a Python join of the
refresh-2 UNCLAIMED 130-id list against every pushed module mapper at
the probe-time tips f26 `a2e43ef77` / f27 `01888909a` — 53 claimed,
77 remaining — + the d42 `adb304ab8` / d43 `31310b652` / c4
`8abf0f1d8` / e26 `a23c64933` / c8 `10192602a` roster-arm sweeps
covering all 77, the 13 d43 row-0 stubs flagged); every seam-witness
vector re-derived by a mechanical f32 Python replay of
`humanoid_fighter_frame` (7/7 match). The 4 wave-B witnesses (1 at
`f4a7074bc` + 3 at `925423511`) are WRITTEN NOT RUN — **no
cargo/rustc/tests/rustfmt/graphify invoked** (code-only mode).

## 10. Refresh-5 FINAL — the FindFrame-family declaration (2026-11-21, this branch)

The d44 wire-final dispatch's closing census: a FULL-BAND mechanical
re-scan of all **602** top-level `case` ids (`NPC.cs:66705-77370`,
two-tab labels) against EVERY pushed lane's `npc.rs::find_frame`
body + every pushed `npc_findframe_*` mapper + the `npc_ai_style*`
find_frame fns, joined through each lane's own `cs_id` hand rows +
bestiary net-id fallback (a Python scan, the §2 method, re-run at the
refresh-5 probe-time tips — NOT the refresh-2 vintage). The headline:

**UNCLAIMED = 0.** Every one of the 602 case ids carries an inline
`find_frame` arm OR a module machine on SOME pushed tip. The
refresh-2 130-id UNCLAIMED worklist (§6) is fully absorbed: 53
module-claimed at the probe-time f26/f27 tips + 77 roster arms
(38 d42 + 31 d43 + 8 c4/e26/c8). The remaining work is KEEP-ONE /
faithfulness adjudication + the in-flight modules' wire batches, NOT
port-from-scratch.

### 10.1 The 480/481/482 trio — CONFIRMED RESOLVED, no wire needed

The census's "MODULE-LANDED-unwired 3" (the extra7b keep-inline trio)
is re-verified arm-by-arm against BOTH the d40 inline arms and the C#
windows — the f26e keep-inline verdict stands, **no hook will ever
claim them**:

| id | kind | C# window | d40 inline | extra7b module | count |
| --- | --- | --- | --- | --- | --- |
| 480 | Medusa | `NPC.cs:75103-75167` | `npc.rs:9865` (d40 tip) | `extra7b.rs:459` | 25 |
| 481 | GreekSkeleton | `NPC.cs:71454-71515` | `npc.rs:9902` | `extra7b.rs:522` | 19 |
| 482 | GraniteGolem | `NPC.cs:75202-75258` | `npc.rs:9935` | `extra7b.rs:582` | 20 |

A mechanical constant-diff (the gaze/idle/walk clamps, the stride
thresholds, the accumulator forms, the air pins) finds the d40 inline
arm and the extra7b module arm **identical for all three** (480: the
ai[2]<0 gaze rows 21..24 both-clamps-21 + the >4 idle rows 0..7 + the
1+|vx| >6 walk rows 10..20 both-clamps-10 + air 8; 481: the ai[2]>0
ladder 15+{-14,3,2,1,0} off ai[1] <22/<28/<34/<40 + the idle row-1
counter-HELD + the |vx|*2 NO-+1 >6 walk rows 2..14 wrap-15→2 + air 0;
482: the ai[2]<0 gaze rows 11..19 both-clamps-11 + the idle row-0
counter-TICKS + the 0.2+|vx| >8 walk rows 2..9 both-clamps-2 + air 1).
The frame counts 25/19/20 match `NPC_FRAME_COUNT` (the d40 inline
reads `npc_rows(cs_id)`, the module `kind.frame_count()` — same
table). The d44 hook `find_frame_kind_extra7b_claimed` returns `None`
for all three (`npc.rs:6924`, the `Medusa | GreekSkeleton |
GraniteGolem` arm) — **RESOLVED, zero wire work**. The module arms
ride as parity references; the merger keeps the d40 inline arms.

### 10.2 The 634 SporeBat RNG-seam contract (published, not wired)

634 rides the floater group `case 48/49/51/60/82/93/137/182/210/211/
253/316/634/662/664` (`NPC.cs:74823-74884`, label `:74835`). Its ONLY
RNG draw is `:74844`: `if (this.type == 634 && Main.rand.Next(10) == 0)`
— a per-tick 1-in-10 spore-dust spawn (Dust 165 `:74846`, velocity
`*= 0.1f` + `noGravity` `:74847-74848`). The zero-RNG rule excludes it
from the f26 7g/7h waves; the f26 7h module header
(`npc_findframe_extra7h.rs:100-133`, tip `a2e43ef77`) publishes the
seam contract for a future seam-fed wave. Restated + extended here
for the d44 wire lane:

* **The draw**: the wiring bridge draws `Main.rand.Next(10)` EVERY
  tick for type 634 ONLY (the `&&` short-circuits for every other
  group member — 182/210/211/253/316/662/664 draw NOTHING on this
  arm). The draw order rides `UpdateNPC`'s AI → FindFrame sequence
  (`NPC.cs:90423` → `:90511`, after the AI's own draws). In the port
  the draw comes from the caller's `npcs_rng` stream
  (`world_sim/npcs.rs:4196`, `RngKey::UpdateNpcs`) — the SAME stream
  the AI pass uses, so the seam preserves the C# draw order.
* **The seam param**: feed the RESOLVED boolean
  `spore_dust_draw_is_zero` into `FrameInput` (the 7d SolarFlare
  resolved-draw precedent — there the latch draws row-first/
  counter-second; here ONE draw per tick). The frame machine never
  draws RNG.
* **The dust effect**: the `Dust.NewDust` 165 spawn + the two field
  writes is a bridge-side effect keyed on the same boolean (the
  port's `wants_*` dust-request seam, the `update_rng_dust` pattern
  `world_sim/npcs.rs:4467`) — the frame machine never touches dust.
* **The post-seam frame machine (zero-RNG)**: the vx-SIGN sprite
  gates (`:74850-74857`, NOT direction), `rotation = vx * 0.1f`
  (`:74858`), the bare `++frameCounter` (`:74864` — the 210/211
  second ++ does NOT apply), the `>= 6` step (`:74875-74879`,
  num291 = 6 — the type-48 num291=5 override does NOT apply), and the
  wrap at `num292 = count − 1` (`:74867-74870` — the 49/51/60/634
  decrement DOES apply: rows 0..3 flap on the 5-row sheet, row 4
  never; `:74880-74883` wraps to 0). `Main.npcFrameCount[634]` = 5
  (`Main.cs:67147` entry 4). The `type == 60` dust block
  (`:74839-74843`) is NOT 634's.
* **Current coverage**: 634 ALREADY carries an inline arm on d43
  (`npc.rs:9636`, tip `6fbe06250`) — the frame machine (vx-sign flip
  + vx*0.1 tilt + the 6-tick count−1 wrap) WITHOUT the dust draw
  (the d43 arm is the zero-RNG frame subset). The seam-fed wave adds
  the dust draw + spawn on top; the frame machine is unchanged. So
  634 is COVERED (inline), and the seam contract is the dust-half
  upgrade path, not a from-scratch port.

### 10.3 The UNCLAIMED re-derivation (the f26-lane worklist)

The full-band scan's residual after inline + module + style coverage
is **9 scanner false-negatives**, all verified covered:

* **5 cs_id-fold inline arms on main** (the scanner's cs_id map could
  not resolve the fold): 135 TheDestroyerBody (`npc.rs:9084`), 246
  GolemHead (`:8926`), 249 GolemHeadFree (`:9228`), 371
  DetonatingBubble + 373 Sharkron2 (`:9629`, the shared arm). These
  are real inline arms — the cs_id hand rows map the kinds to OTHER
  protocol ids (the destroyer/golem/bubble/sharkron folds), so the
  vanilla-id join missed them. COVERED (inline, main).
* **4 wave-B module machines** (no `NpcKind` on any tip yet): 155
  Wolf (`npc_findframe_b.rs:497`), 167 UndeadViking + 197
  ArmoredViking (the fighter group `:240-241`), 206 IcyMerman (the
  archer shape `:437`). The machines are landed on main; the kinds
  await their roster lanes (d36's 141-210 band). COVERED (module).

So the TRUE no-coverage residual is **ZERO**. The f26-lane worklist
is therefore the KEEP-ONE/faithfulness adjudication:
* the **13 d43 `{}` row-0 stubs** (637/638/656/670/678-684 — the town
  pets/slimes; g10's `npc_findframe_pet.rs` carries the faithful
  pet-aware town machine, the d43 retire `6fbe06250` adjudicates
  11 of them onto g10);
* the **d42 arm-quality verdicts** (the 578-630 band's stand-ins vs
  the faithful machines — the d42 research/239 §3 re-adjudication
  cedes 581/587/592/604-607/615 to c4/e26/c8);
* the **in-flight f26/f27 module wire batches** (7d-7k + extra10/12,
  the claim sets grow as the lanes push).

### 10.4 The FindFrame-family declaration (the merger's keep-one map)

The family is COMPLETE on the pushed tips — every case id has a
machine. The merger's keep-one resolutions (the §3 union plan,
extended):
* **d40-faithful-six** 464/465/466/468/469/470: the d40 inline arms
  are FAITHFUL (the f27 slice-9 verdicts) — keep d40, the extra8
  mapper never carried them (NO exclusions needed).
* **d41-inline-twins** 494-506: the d41 faithful inline arms are
  arm-by-arm equal to the extra7c machines — keep one either way
  (the d44 hook precedes at merge; the f26e 480/481/482 precedent).
  508/509 have NO roster kind on any tip — the extra7c module claims
  stand alone.
* **480/481/482**: keep the d40 inline arms (§10.1), the extra7b
  module arms are parity references, the hook NEVER claims them.
* **the d43 roster** (631-697): the retire `6fbe06250` adjudicates 18
  kinds onto wired siblings (693→g5, 636/661→d125, 11 town pets→g10,
  694→c3, 654/677→c7, 655→c9); the rest keep the d43 inline arms.
* **the f26/f27 modules**: the in-flight wire batches land the
  faithful machines over the roster stand-ins (the hook precedes the
  body match, first-match-wins).

**WIRED count: 198** (the refresh-5 §0 — 196 at refresh 4 + the
wave-B residual 2). **MODULE-LANDED-unwired on THIS branch: 0 real
wire ids** (the extra7b trio RESOLVED keep-inline). **UNCLAIMED: 0.**
The family declaration is FINAL at the refresh-5 probe-time tips;
the in-flight lanes (f26/f27/d41/d42/d43) may push more claims before
merge — the merger re-probes at merge time (the §1 vintage note).
