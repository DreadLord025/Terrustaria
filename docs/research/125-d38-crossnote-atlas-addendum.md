# 125 — d38 crossnote ADDENDUM to the atlas-repack plan (research/113)

Status: 🔒 SEALED (2026-11-21, RETIRE phase — §§11-13) — the d38 lane is
COMPLETE: 35 kinds + 35 sheets + 35 portraits, protocol==vanilla in the
281-337 corridor; the merger hand-off is §13. Previously ✅ FINAL
(2026-11-21) — published INPUT for
`task/W4-d35-npc-kinds-71-140`'s ATLAS-REPACK phase, read through d35's
FINAL tip `e5558b0a9` (research/113 §10-12 + Appendix B). §§1-5 below
stand as the eager-form record (both-agree supersession, §6 aligns them
with the lazy-load FINAL contract); §§6-10 are the FINAL phase (the
lazy-load re-projection, the FindFrame status census, the bestiary
unlock census, the 108-extend roster cross-ref, the FINAL witnesses);
§§11-13 are the RETIRE phase (the stand-in drop-hunks + the residual
list re-probed at d44 `c863a7a5`, the 328 extra-card exact drop-hunk,
the GAP-1 closure receipt against c8's generic critter gate, the SEAL
declaration). Branch:
`task/W4-d38-npc-kinds-281-350` (FINAL phase base `6f9dd02d`, FINAL tip
`1ba0395c7`; RETIRE continues the SAME branch).
Original crossnote status: 📣 published INPUT (d35c, tips
`519c8035`/`7d131be3` read 2026-11-21) and the d40b sheets
phase (`task/W4-d40-npc-kinds-421-490`, data-only tip `8fe1d6d3` read same
day). Base:
`a725acb0` + d38's 35 packed sheets (`c096bac9`). Code-only mode: no
cargo/tests run; every number measured by `tools/_d38_crossnote.js` (the
`_d38_atlas_cumulative.js` harness: `tools/atlas_sim.js`'s real-dims LZX
census + the skyline/maxrects twins, the purged L-8b git-show stubbed in
memory, `verify()` + skyline double-run determinism on EVERY layout;
outputs `tools/_d38_crossnote.txt` / `.json`).

## 1. The critical correction: BAF-alone dies WITHIN ONE wave, not two

d38's s11 ANALYSIS proposal 1 ("maxrects BAF promotion … Headroom is THIN
(22 px) — the next wave (~35 sheets ≈ +120-220 px) overflows again") and
proposal 3 ("even maxrects dies within two waves") UNDERESTIMATED the
crisis. Re-measured with d40's REAL 56-sheet set (not a ~35 estimate):

| set (runtime = landed E-4b gore, width 8192, PAD 2) | sheets | padded area | area bound | skyline | maxrects BAF |
|---|---|---|---|---|---|
| HEAD (d38's 35 packed) | 9532 | 64.96 Mpx | 7934 | 8236 **OVER +44** | 8058 (134 headroom) |
| + d35's 21 packed (d35-tip state) | 9553 | 65.24 Mpx | 7969 | 8232 **OVER +40** | 8070 (122) |
| + d35 21 + **d40 56** (d40b lands) | 9609 | 66.95 Mpx | **8177 (15 left)** | 8688 **OVER +496** | 8278 **OVER +86** |
| + d35 33 + d40 56 (full three-wave) | 9621 | 67.61 Mpx | **8258 OVER +66** | 8621 **OVER +429** | 8371 **OVER +179** |
| + c4's 4 on top (four-wave) | 9625 | 67.74 Mpx | **8273 OVER +81** | 8640 OVER +448 | 8373 OVER +181 |

("area bound" = ⌈padded area / 8190⌉ + 2 — research/113's formula, the
height NO single-page packer can beat. Boot-font twins differ ≤ 2 px, all
in `_d38_crossnote.json`.)

**The d40b wave ALONE kills maxrects BAF**: 8070 → 8278 (+208 px for the
56 sheets), over the 8192 bound by 86. The 22-px-headroom finding from the
d38+d35-deferred set (8170) was already one wave from death; with d40's
actual id list the death is CONFIRMED, not projected. And the full
three-wave set's AREA BOUND alone (8258) exceeds 8192 — **no packer
whatsoever fits it on one page**. This independently reproduces
research/113 §1's decisive fact on a SMALLER set (113's 26-branch union:
74.71 Mpx → 9124 bound; ours: three waves only, 67.61 Mpx → 8258 bound).
**A second atlas page is REQUIRED, not optional — d38 withdraws its
proposal 1 (BAF promotion) as a standalone fix.** It remains useful only
as page-internal packing; 113's own measurement (BAF 9998 vs skyline 9382
on the union, chaotic ordering) already rejected the swap — agreed.

d38 endorses 113's chosen fix as amended by `7d131be3`: the
**boundary-split skyline 2-page form** (pack unbounded, keep rects
entirely under 8192 on page 0, re-pack the tail into page 1; 113's
measured page-0 8163-8192 / page-1 1232-1298 across its 10 scenarios).
The incremental form's UV stability is attractive but its pinned pre-W4
name set cannot survive 26 interleaving branches (113's own rejection
rationale) — the boundary form is the maintainable choice.

## 2. Ledger reconciliation (d35's 8188 vs d38's 8232 — both correct)

d35's sheets phase measured "HEAD base 8115/8119" and packed its 21 at
**8188** — that base is `a725acb0` WITHOUT d38's 35 (d35 branched before
d38's sheets landed; its base predates them). d38's harness measures the
same 21 over OUR d38-carrying HEAD at **8232** — the +44 delta is exactly
d38's own overflow contribution (8115 → 8236 with d38 alone, s10's
measurement). Neither number is wrong; they are different bases. The
merger's page-0 input must be the UNION, and only the repack wave's
2-page form reconciles them (both single-page numbers are over).

The c4 quotes still reconcile exactly: 7757/8143 reproduced by
`tools/_d38_c4_recheck.js` (s10); the g72 "7530" remains STALE (pre-gore
enumeration). d35's sheets-phase "17-branch 198-name union 8530" is
superseded by 113's 26-branch 339-name re-extraction (9382 skyline) —
d38's numbers here are the THREE-WAVE subset, not the full union; 113's
union table remains the authoritative merger input.

## 3. d40 sheet-count input for the cumulative estimate

d40 tip `8fe1d6d3` is data-only (12 batches, ALL_KINDS 270 → 326, +56
kinds). Its sheets phase (d40b) is in flight, NOT pushed — no `assets.rs`
diff on the branch. The 56 ids (verified against d40's batch commits AND
verbatim-equal to 113's `D40_IDS` forecast list in
`_d35_repack_measure.js`):

```
430-438, 440, 442-470, 472-488   (56 ids; 439/441/471 are base-owned,
                                  489/490 already packed by S1-D-3z)
```

All 56 `NPC_<id>.xnb` files exist and decode (the harness asserts this).
**Every one draws its own sheet in vanilla** (`TextureAssets.Npc[type]`;
the Hide kinds — 437/438/440 cultist fixtures, 455/456 dragon bodies,
478 MothronEgg, 488 TargetDummy — fold their bestiary CREDIT but still
draw their own texture in-world, the 371/373/384 precedent). So d40b's
sheet count is **56, not ~35** — the "~35 sheets/wave" heuristic
under-counts this band by 21. 113's forecast already used 56; this
section CONFIRMS it against the pushed tip and adds the per-wave cost:
**+208 px maxrects / +456 px skyline** over the d35-21 state (R vs P
rows above). The d35 deferred 12 add a further +0.66 Mpx padded area
(Q vs R: skyline actually DROPS 8688 → 8621 — the packer's chaotic
ordering again,
113's finding; the area bound rises monotonically 8177 → 8258, which is
the honest metric).

**Updated overflow projection for the merger** (d38's input to 113's
union table): after d34/d37/d39/d43 also pack (~35-46 sheets each, still
unpushed), the union grows past 113's 385-addition worst case; page 1 at
1924 px (incremental) / 1232-1298 px (boundary) has room for ~4× that
before a page 3 is needed — the 2-page form unblocks the whole remaining
wave-4 roster, as 113 §1 concluded.

## 4. d38 draw-side audit results (the crossnote's verify half)

**(a) All 35 in-world draw arms wired — no None-skip left in-world.**
Every kind's arm reads `assets.npc_sheet(texture_net_id(kind))`, all 35
ids ride `D38_B1..B7_SHEETS` → `npc_sheets` (entities.rs:517-523), all 35
names in `STATIC_SHEET_NAMES` (the `w4_d38_every_draw_sheet_is_packed`
audit), and the G6 `render_key` rows landed per batch (npc.rs:19518-19559,
vanilla types, injective). The one deliberate non-own-sheet draw: 328's
vanilla arm draws the Pumpking's ARMS (`Main.cs:22669-22699`,
`TextureAssets.PumpkingArm`) — that overlay is c6's lane
(`pumpking_arm` field + the two-link chain draw on
`task/W4-c6-style59-pumpking-blade`, NOT on d38/main); d38 draws the
blade's own single-frame body sheet, the documented s6 witness. No
action for d38; the merger keeps c6's overlay arm.

**(b) The bestiary portraits WERE None-skipping — FIXED this phase.**
The 35 kinds had ENTRIES rows + `portrait_data` arms but no
`BESTIARY_NPC_SHEET_IDS` slots (the table stood at 181), so
`draw_portrait`'s `entry_sheet` gate returned early — 35 empty bestiary
cards. This is research/108 §5's d38 half of the portrait-slot gap; d38
closed it HERE (commit `d0094d8a`): +35 slots/vec rows in ENTRIES order
(181 → 216), the portrait arms ride the sheet row height over
`Main.npcFrameCount` (the c4/d35 shape). **Merger note**: 108's final pin
"BESTIARY_NPC_SHEET_IDS 250 → 323 after the gap close" now reads
**216 (d38 closed) + 10 (d34) + 16 (d35) + 43 (d37) = 285 as-pushed-after-
merge, + 38 (d40's entries) = 323 final** — the 323 target is unchanged,
d38's +35 simply moved from the merge-wave worklist into this branch.
d40's 38-entry half stays OPEN on d40b (its entries exist, its sheets
don't yet).

**(c) The rotation whitelist 8-IN is consumed by the draw.** The gate
(`n.rotation != 0.0 && matches!(…)`, entities.rs) was extracted verbatim
into `npc_rotation_allowed` (55 kinds, membership + order proven
identical vs HEAD by `tools/_d38_crossnote_check.py`). The 8 IN kinds'
writers all exist and are reachable: 289 via
`update_giant_cursed_skull` → `npc_ai_style10::style10_ai` (npc.rs:12297,
rotation at style10.rs:368/:372), 328 via the world-sim pool arm
(world_sim/npcs.rs:4467-4493 → `npc_ai_style59::ai_style59`, rotation at
style59.rs:355/:364/:398), the six FindFrame writers live in
`Npc::find_frame` (npc.rs:9317/:9379/:9388/:9401/:9415/:9469). New
witness `w4_d38_crossnote_the_rotation_gate_consumes_the_writers` runs
the six FindFrame machines with exact tilt values, pins the 27 OUT kinds
against a forced rotation, and counts 8 IN. The 27 OUT kinds are a
deliberate C# match (zero rotation writes in their machines — the s8/s9
censuses), not a deferral.

## 5. Witnesses (written, NOT run — code-only)

- `crates/tr-app/src/ui/bestiary.rs`
  `w4_d38_crossnote_the_portrait_slots_ride_the_real_strips` — 35 kinds,
  real dims, slot resolution, SetFrameSize row heights (the 286 integer
  division, the 328 single frame), one-row frame-0 UVs.
- `crates/tr-app/src/render/entities.rs`
  `w4_d38_crossnote_the_rotation_gate_consumes_the_writers` — the 6
  FindFrame writers live + exact values, the 2 AI writers cited, 27 OUT,
  8 IN.
- `tools/_d38_crossnote_check.py` (RUN, passed): gate membership/order
  identical to the pre-edit inline list; brace balance on all 3 edited
  files; 216 declared == 216 ids == 216 vec rows; first 181 rows
  byte-identical; tail in ENTRIES order.
- `tools/_d38_crossnote.js` (RUN, passed): 8 scenarios × verify() +
  determinism, outputs committed.

## 6. FINAL alignment: the lazy-load census (research/113 §10-12, d35 tip `e5558b0a9`)

d35's ATLAS FINAL extension re-read this addendum at `6f9dd02d` (its §11:
"d38c … Inventory-current: YES … no delta to the union list") and then
measured the NPC-sheet lazy-load census (§10). **Both lanes agree on the
eager-form verdicts above** — the BAF withdrawal, the boundary-split
endorsement, the 8188-vs-8232 ledger reconciliation — and the lazy form
does not contradict them, it re-partitions the INPUT:

- **The C# contract** (113 §10.1-10.2, re-verified here at the cited
  lines): ALL 697 `TextureAssets.Npc` register `DoNotLoad`
  (`AssetInitializer.cs:127-129`); `Main.LoadNPC` (`Main.cs:59084-59090`)
  fires per DRAWN NPC from `NPC.SetFrameSize` (`NPC.cs:66661-66663`) and
  per bestiary-icon hover (`UnlockableNPCEntryIcon.cs:185`). The port's
  eager `STATIC_SHEET_NAMES` NPC rows are a documented superset deviation.
- **The union shrinks**: pulling the NPC corpus out of the merged union
  drops it from 9478 skyline (2 pages) to **7059 (ONE page, 1133 px
  headroom)**; the FINAL 424-name re-extraction's lazy form is 7162
  (113 §10.3). d38's 35 sheets are among the 441 NPC rows that move.
- **d38's own re-projection** (`tools/_d38_final_lazy.js`, RUN, 4
  scenarios × verify() + skyline double-run determinism, outputs
  `tools/_d38_final_lazy.json` / `.txt` committed):

| scenario | sheets | padded area | area bound | skyline | maxrects BAF |
|---|---|---|---|---|---|
| E35: HEAD eager MINUS d38's 35 NPC rows | 9497 | 64.24 Mpx | 7847 | **8115 fits (77 headroom)** | 7956 (236) |
| E35 + boot font | 9498 | 64.25 Mpx | 7848 | 8119 (73) | 7956 (236) |
| L91: d38's 35 + d40's 56 forecast ALONE (the NPC-page slice) | 91 | 2.43 Mpx | 299 | **1554 fits (6638 headroom)** | 1554 (6638) |
| L91 + boot font | 92 | 2.43 Mpx | 300 | 1554 (6638) | 1554 (6638) |

  Reading: (a) E35 reproduces s10's pre-d38 baseline **8115 exactly** —
  the 35 sheets' eager contribution was precisely the +121 that pushed
  HEAD to 8236; under the lazy contract that overflow evaporates from
  the union page. (b) The 35+56 two-wave NPC slice packs **1554 px on
  its own page** — inside 113 §10.4's per-world budgets (fresh 1668 /
  long-lived 2152 / bestiary 2566), i.e. d38's wave adds ~0.9 Mpx of the
  ~15.5 Mpx long-lived set and d40's forecast ~1.5 Mpx; neither wave
  threatens the one-page NPC budget. (c) The §1-3 overflow tables above
  remain the correct EAGER-form record (what the merger sees if step 7
  is skipped); under the FINAL §10.6 step-7 contract the d38 rows leave
  `STATIC_SHEET_NAMES` and ride the per-world NPC page via
  `texture_net_id` — d38's draw path already resolves every sheet by
  name (§4a), so no draw-side change is needed on this branch.
- **The bestiary superset keeps the atlas immutable** (113 §10.6): d38's
  35 portrait slots (§4b, the 181→216 close) are exactly the ids the
  bestiary-hover lazy load can reach; packing them at world load is the
  documented superset, still one page. The 323 slot merger pin (§4b)
  is unchanged by the lazy form — slots are a LOOKUP table, not atlas
  input.
- **d40b stays data-only** at `8fe1d6d3` (113 §11 re-confirms): the
  56-id forecast stands verbatim; L91 above measures it as the NPC-page
  slice, the dagger rows in 113's union table retire when d40b pushes.

## 7. FINAL: the 35 kinds' FindFrame status (script-check vs the findframe lanes)

Mechanical census (`tools/_d38_final_census.py` + `tools/_d38_final_ff.py`,
RUN, outputs committed): the C# `NPC.FindFrame` switch
(`NPC.cs:66705-77370`) carries a case for **34 of the 35**; the one
NO-CASE kind is 328 PumpkingBlade (`npcFrameCount[328]` = 1, the row-0
pin correct by construction — the `case 328` at `:77828` is
`DoesntDespawnToInactivity`, NOT FindFrame). The port's `find_frame`
(`npc.rs:6874-9705`) has an arm for **all 35** — zero coverage gaps.
The sibling-lane claims at the CURRENT pushed tips (f26 `7f38fd345`,
f27 `1a24a133c`, d44 `7c430def6` — whose research/68 refresh 2 rebuilt
the master 1-697 table; the f26 7f / f27 slice-10 waves are all 531+
ids, ZERO new claims inside 281-337, re-probed by
`tools/_d38_final_ff.py`):

| d38 ids | C# case | port arm today | lane state (d44 master table) | wiring-round verdict |
|---|---|---|---|---|
| 281-286 casters | `:77023-77028` | inline pose group (faithful: flip + row0/air2/windup1, counter untouched) | **WIRED(d44)** `extra6` (`59b798dc`) | keep-one: the extra6 machine is the same pose group; either side faithful, hook precedes the body match |
| 287 BoneLee | `:68590` humanoid group | `humanoid_town_frame()` **stand-in** | **WIRED(d44)** `extra6` (wave-B `humanoid_fighter_frame` delegate) | **module wins** — the stand-in lacks the grounded flip + air tail (the f27 Nailhead three-way gap class) |
| 289 GiantCursedSkull | `:72652` | inline bands + ai[3] dash (faithful; the dead `>12` re-zero not carried, C# itself resets to 0 first) | INLINE-ARM wave-B claim | keep the port arm (faithful twin) |
| 290 Paladin | `:75963` | inline hammer ladder + \|vx\| walk + strict >8 wrap (faithful) | **WIRED(d44)** `extra6` | keep-one, both faithful |
| 291/292 Sniper/Tactical | `:76139-76140` | inline aim row (int)ai[2]-1 + \|vx\|*3.3 walk rows 7.. + air 6 (faithful) | **WIRED(d44)** `extra6` | keep-one, both faithful |
| 293 Commando | `:76021` archer group | inline aim row (int)ai[2] + \|vx\|*3 walk rows 6.. + air 0 (faithful) | **WIRED(d44)** `extra6` (SkeletonArcher delegate) | keep-one, both faithful |
| 294/296 AngryBonesBig/Helmet | `:68591/:68593` | `humanoid_town_frame()` **stand-in** | INLINE-ARM wave-B (`npc_findframe_b` fighter machine, air-row-0 pin) | **wave-B module wins** — same stand-in gap as 287 |
| 295 AngryBonesBigMuscle | `:68592` | `humanoid_town_frame()` **stand-in** | **WIRED(d44)** `extra6` (fighter delegate, air pin 0) | **module wins** |
| 297/298 birds | `:73629-73630` | inline perched row 4 + 4-tick flap + vx*0.1 tilt (faithful) | INLINE-ARM d38 (68 §2 rows cite npc.rs:9399/:9413) | keep d38 arms |
| 299 Squirrel | `:76458` | inline \|vx\|*0.5+1 walk + air 2/4 (faithful; the aiStyle-3 turn gate unreachable for aiStyle 7) | INLINE-ARM d38 | keep d38 arm |
| 300 Mouse | `:76513` | inline idle 2-row blink + walk/air same (faithful) | INLINE-ARM d38 | keep d38 arm |
| 301 Raven | `:73559` | inline -direction flip + >4 flap rows 1..4 (faithful) | INLINE-ARM d38 | keep d38 arm |
| 302/304 SlimeMasked/HoppinJack | `:70812-70813` | inline slime strip + the 302/304 sprite snap `:70824` (faithful) | INLINE-ARM d38 | keep d38 arms |
| 303/337 bunnies | `:76853/:76854` | inline \|vx\|+1 hop + air 4/6 (faithful) | MODULE-LANDED(f27) `extra7` | keep-one: faithful twins (68 §5 wire note), module wins at merge per the hook order |
| 316 Ghost | `:74834` floater group | inline vx*0.1 tilt + 6-tick flap over 4 rows (faithful) | MODULE-LANDED(f27) `extra7` | keep-one: faithful twin |
| 317/318 eyes | `:74003-74004` | inline atan2 (+3.14 left) + 8-tick flap over 2 rows (faithful) | MODULE-LANDED(f27) `extra7` | keep-one: faithful twins |
| 322/323/324 costume skeletons | `:68599-68601` | `humanoid_town_frame()` **stand-in** | MODULE-LANDED(f27) `extra7` (fighter delegate) | **module wins** (68 §3: "the d38 287/295/322/323/324 town stand-ins are superseded") |
| 328 PumpkingBlade | NO CASE | row-0 pin (correct; count 1) | NO-CASE (68 §2) | nothing to wire |
| 333-336 ribbon slimes | `:70814-70817` | inline slime strip (faithful) | MODULE-LANDED(f27) `extra7` | keep-one: faithful twins |

**The gap list for the wiring round is exactly 7 stand-ins**: 287/295
(extra6, WIRED(d44)), 294/296 (wave-B module, unwired on any tip —
`npc_findframe_b` has NO cs_id hook anywhere yet), 322/323/324 (extra7,
MODULE-LANDED unwired). The stand-in gap is the f27 Nailhead class:
`humanoid_town_frame` has no grounded sprite flip, no air tail, and its
walk wraps at `count - extra_frames_count` (extra=0 for these kinds, so
the wrap band is right) — but the C# fighter group (`NPC.cs:68647-69655`)
flips spriteDirection grounded, pins air row 1 (then the `:69647` pin
list snaps 294/295/296 to row 0), and idles row 0 with the counter
zeroed. The 27 dedicated arms are faithful twins of their C# cases
(line-for-line read this phase; the two dead-code branches — the 289
`>12` re-zero and the 299 aiStyle-3 turn gate — are unreachable for
these kinds and documented, not carried). **No d38 code change**: the
68 §3 keep-one plan resolves every stand-in at the merge (the module
hook precedes the body match); d38's arms stay as the parity references.
The 8-IN rotation whitelist (s9/crossnote) is unaffected — none of the
7 stand-in kinds writes rotation in either form.

## 8. FINAL: the bestiary UNLOCK census (kill-counts / rarity, per kind)

Mechanical census (`tools/_d38_final_unlock.py`, RUN, output committed):
the C# chain `CommonEnemyUICollectionInfoProvider.GetKillCountNeeded`
(`:21-34`) = `KillsToBanner[BannerToItem(NPCtoBanner(BannerID()))]`
(`BannerSystem.cs:341/:403`, `ItemID.cs:1303-1304`), and the port's
`kills_needed_for` (`tr-core/bestiary.rs:979`) rides the same three
tables. **All 35 chains AGREE** (banner id, item id, full kill count,
tier-2/3 thresholds — zero disagreements):

- **29 CommonEnemy kinds**: full 50 kills (tiers 1/10/25/50), EXCEPT
  **290 Paladin: full 25** (banner 149 → item 2958 → `KillsToBanner`
  25; tiers 1/5/12/25) — the one non-default ladder in the band, and
  the port reproduces it exactly.
- **6 critters** (297/298/299/300/303/337 — `CountsAsACritter`,
  `NPC.cs:6887-6897`: lifeMax 5 + damage 0): vanilla uses
  `CritterUICollectionInfoProvider` (`:14-20`) — **one SIGHTING jumps
  straight to tier 4**, no kill ladder at all. The port's
  `unlock_state` (`bestiary.rs:2529`) applies the kill ladder to ALL
  entries: **GAP-1, PRE-EXISTING base-wide** (the base roster already
  carries the Goldfish 55 critter entry, `bestiary.rs:657-664`, on the
  same kill ladder; d38's b4/b5/b7 added six more critter rows that
  inherit the gap — d38 did not introduce it). A caught bunny unlocks
  its card in vanilla; in the port it needs 50 kills. The sightings
  tracker EXISTS (`tr-core/bestiary.rs:143-173`, persisted) but no
  production writer feeds it and `unlock_state` never reads it.
  Wiring-round item: a critter branch in `unlock_state` (sight → tier 4)
  + the `NPCWasNearPlayerTracker.ScanWorldForFinds` port (`:102-130`).
- **5 rare spawns** (287 BoneLee rarity 1, 290 Paladin rarity 2,
  291/292/293 rarity 1 — the SetDefaults `rarity` rows): vanilla adds
  `RareSpawnBestiaryInfoElement` (`BestiaryEntry.cs:32-35`) + the
  portrait rarity stars (`NpcBestiaryRarityStars`). The port's
  `BestiaryEntryDef` has NO rarity field and the UI draws no stars:
  **GAP-2, display-only** (unlock tiers unaffected — rarity is not an
  unlock condition in either lane). Wiring-round item: a rarity field
  + the star row (the `NPCPortraitInfoElement` port).
- **328 PumpkingBlade**: vanilla `Hide = true`
  (`NPCID.cs` draw-offset row, ported verbatim at
  `bestiary_draw_offsets.rs:384`) → the populator's `GetExclusions`
  (`BestiaryDatabaseNPCsPopulator.cs:870-886`) gives it **NO entry**;
  its kill credit stays "PumpkingBlade" (`ContentSamples` name row, no
  fold) but no card exists. The port gave it an ENTRIES row + a
  portrait slot (b6/crossnote B): **GAP-3, one extra card** vs vanilla.
  The kill credit itself is faithful (the port registers
  "PumpkingBlade" kills, vanilla does too — they just never display).
  Merger note: the 216-slot pin and the 323 union pin count this row;
  dropping it is a 1-row ENTRIES/slot/vec edit + the witness re-pin,
  deferred to the wiring round to keep this phase's pins stable.
- **quick_unlock**: zero of the 35 (no boss, no
  `ShouldBeCountedAsBossForBestiary` member — the set is
  {517,422,507,493,13,664}, `NPCID.cs:10481`); all 35 rows
  `quick_unlock: false` — agrees with `BestiaryEntry.Enemy`'s
  non-boss branch (`:44`).
- **The kill-credit path** (`world_sim/npcs.rs:5283-5291`) registers
  `entry.credit_id` per death + broadcasts the count — the
  `NPCKillsTracker.RegisterKill` shape (`:23-33`), faithful for all 35
  (the 328 credit included, per the vanilla name-row census above).

## 9. FINAL: the research/108 cross-ref (the extend census, d37 tip `bdd59afba`)

The 108 EXTEND re-census (18 pushed claimants + 4 local-only) re-confirms
d38's corridor at the CURRENT tip `6f9dd02d`:

- **Protocol band: 281-337 vanilla-free, CONFIRMED** (E0 row: "+35 |
  281-337 | portrait gap CLOSED"). The task's proto-band question
  ("429-461?") is **NO** — 429-461 is d35's port-local band (renumbered
  698-730 in the canonical plan); d38 is protocol==vanilla per 108 §1/§3
  and the extend E3 ("d34/d38/d40 keep pushed … pairwise collision-free").
  Mechanically re-verified here (`tools/_d38_final_census.py`): all 35
  `to_u16` ids == their `cs_id` resolutions (hand rows + the S1-F-3
  bestiary net-id fallback, `data.rs:3363-3370`), band [281,337], the 22
  holes exactly the base-owned ids 108 §2b lists (288/305-315/319-321/
  325-327/329-332).
- **The two d38 dedupes** (E3): GiantCursedSkull keep d38@289 (drop
  c3@289), PumpkingBlade keep d38@328 (drop c6@328) — both are d38's
  vanilla-free ids; the dropped lanes keep their AI/draw/sound work.
  d38's 328 entry question (§8 GAP-3) is independent of the dedupe: c6's
  dropped port also drew the blade, the bestiary row is d38's alone.
- **Count pins** (E4 supersedes §6): the union is 550 kinds / 489
  ENTRIES / ≤357 slots / 446 static NPC sheets / 103 cues / max proto
  761. d38's contributions to those pins: +35 kinds, +35 entries (the
  328 row is the one vanilla-exclusion candidate, §8), +35 slots
  (already landed, 181→216), +35 static sheets (which the 113 §10.6
  step-7 lazy form moves to the per-world half — the 446 pin is the
  EAGER-form count, the lazy union's NPC rows leave it).
- **The findframe cross-ref** (E5): 108's checklist cites f26/f27 at
  their older tips (`63417edd`/`0b8e225d`); the CURRENT tips
  (`7f38fd345`/`1a24a133c`) plus d44's `7c430def6` master table are the
  authoritative wiring-round input — §7 above is the d38-band slice of
  that table, re-derived mechanically, and its 7-stand-in gap list is
  the d38 half of the 68 §5 worklist.

## 10. FINAL witnesses

- `tools/_d38_final_census.py` (RUN, passed): the 35-kind extraction
  (to_u16 batch blocks), cs_id==proto proof, the SetDefaults census
  (lifeMax/damage/rarity/boss/catchItem per kind, the OR-chain and
  range-block parsing), the FindFrame case census (34 covered + 328
  NO-CASE), the npcFrameCount literal rows (all 35, `Main.cs:67082`'s
  697-entry initializer).
- `tools/_d38_final_ff.py` (RUN, passed): the port find_frame arm census
  (35/35 arms, zero missing) + the sibling-tip claim scan.
- `tools/_d38_final_unlock.py` (RUN, passed): the C# vs port banner/
  item/kills chains — all 35 agree; the 6-critter provider gap, the
  5 rarity rows, the 328 Hide exclusion.
- `tools/_d38_final_lazy.js` (RUN, passed): E35/L91 × boot-font twins,
  verify() + double-run determinism on every layout (§6 table).
- `crates/tr-app/src/bestiary.rs`
  `w4_d38_final_the_unlock_ladders_match_the_banner_chain` — all 35
  kinds' `kills_needed_for` values pinned (290 Paladin=25, the other 34
  =50 — the 6 critters' ladder values are what the port computes today,
  the GAP-1 provider difference is documented not asserted away), the
  6 critter rows + the 328 Hide row cited; WRITTEN, NOT RUN (code-only).

## 11. RETIRE: the 7-town-stand-in drop-hunks + the residual at d44 `c863a7a5`

The FINAL §7 gap list (7 `humanoid_town_frame` stand-ins) re-probed
against d44's CURRENT tip **`c863a7a5`** (wire round 3 — past the §7
census's `7c430def6`): the extra7 hook is NOW WIRED (`5568fabf`, the
68 refresh-3 cells), so 5 of the 7 resolve on the d44 tip itself and
the **residual stand-in list is exactly 2 ids: 294/296** (wave-B
module — `npc_findframe_b.rs` has NO `find_frame_kind_*` mapper and NO
cs_id hook on ANY pushed tip, mechanically re-verified by
`tools/_d38_retire_check.py` R1, RUN, ALL PASS).

| id | kind | our stand-in arm (`npc.rs` @ `1ba0395c`) | d44-tip state | wiring-round action |
|---|---|---|---|---|
| 287 | BoneLee | `:9531-9532` (comment + arm) | **WIRED** extra6 (`c863a7a5` hook `:6987`, mapper 287→BoneLee) | DROP the arm — the extra6 hook claims cs_id 287 before the body match; the fighter delegate carries the grounded flip + air row 1 (NOT in the `:69647` pin list) |
| 294 | AngryBonesBig | `:9514-9515` | **RESIDUAL** — wave-B module arm exists (`npc_findframe_b.rs:239/:252` fighter group, air pin 0), NO hook anywhere | KEEP the stand-in until the wave-B wiring batch lands a cs_id hook (or the wiring round delegates the arm directly to `humanoid_fighter_frame`, air_row0=true) |
| 295 | AngryBonesBigMuscle | `:9516-9517` | **WIRED** extra6 (mapper 295; `let air_row0 = kind == FindFrameExtra6Kind::AngryBonesBigMuscle;`) | DROP — extra6 fighter delegate, air row 0 (the `:69647` pin) |
| 296 | AngryBonesBigHelmet | `:9518-9519` | **RESIDUAL** — same as 294 (`npc_findframe_b.rs:240/:253`, air pin 0) | KEEP until wave-B wiring |
| 322 | SkeletonTopHat | `:9326-9327` | **WIRED** extra7 (`c863a7a5` hook `:7003`, mapper 322) | DROP — extra7 fighter delegate (`humanoid_fighter_frame(…, false, false)`: air row 1, idle row 0) |
| 323 | SkeletonAstonaut | `:9328-9329` | **WIRED** extra7 (mapper 323) | DROP — same delegate; the ONE 15-row sheet (walk wraps at 14) |
| 324 | SkeletonAlien | `:9330-9331` | **WIRED** extra7 (mapper 324) | DROP — same delegate, 16 rows |

Mechanics of the drop (the merger's edit, NOT d38's — d38's branch has
no extra6/extra7 modules; the arms stay as the parity references until
the merge): the d44 `find_frame` hook order (`find_frame_kind_c` →
`extra5` → `extra6` → `extra7` → `extra7b`, all `return`-early) precedes
the body match, so once both branches merge the five WIRED ids never
reach the stand-in arms. The keep-one edit deletes the five
comment+arm pairs above; the `humanoid_town_frame` helper itself STAYS
(the town kinds + 294/296 still ride it until wave-B wires). The
FINAL witness `w4_d38_final_the_seven_town_standins_are_the_wiring_gap`
(`npc.rs:35137-35178`) re-pins at the same edit: the five dropped kinds
leave its list (their no-flip/no-air assertions would fail against the
module machines — deliberately), 294/296 keep the stand-in assertions,
the 328 row-0 pin stays. The b2/b3 walk witnesses
(`w4_d38_b2_the_bone_lee_humanoid_frame` `:34747`,
`w4_d38_b3_the_frames` `:34832`) keep passing either way (the walk band
is identical in both forms — the module adds the flip/air tail the
witnesses don't assert against).

**The §7 table's 303 correction**: the FINAL §7 row "303/337 bunnies …
MODULE-LANDED(f27) extra7" was half-stale — at `c863a7a5` the extra7
mapper carries **337 but NOT 303** (the 15-id set is
316/317/318/322/323/324/333/334/335/336/337/437/438/440/442, R1
mechanical). The 68 master table at `c863a7a5` §2 agrees: 303 is
`INLINE-ARM d38:BunnySlimed(npc.rs:9349)` — **303 keeps the d38 inline
arm** (faithful twin of `NPC.cs:76853`, no module claim to resolve).
337 rides extra7 (keep-one at merge, both faithful).

The keep-one dedupes for the 10 WIRED faithful twins (the §7 "either
side faithful" rows — no behavior change, the merger picks one):
281-286 casters (our `:9520-9530` + `:9579-9633` pose groups vs
extra6's caster group), 290 Paladin (`:9542-9560`), 291/292
Sniper/Tactical (`:9561-9578`/`:9478-9495`), 293 Commando
(`:9496-9513`), 316/317/318/333-336/337 (the floater/eye/slime/bunny
inline arms vs extra7). 289 GiantCursedSkull keeps the d38 inline arm
(`:9533-9541`) per the E3 dedupe (the wave-B module arm is the dropped
twin). None of these arms draws RNG in either form — the dedupe cannot
move a stream.

## 12. RETIRE: the 328 extra-card drop-hunk (GAP-3, exact)

Vanilla facts re-verified at source this phase:
`BestiaryDatabaseNPCsPopulator.GetExclusions`
(`BestiaryDatabaseNPCsPopulator.cs:870-886`) collects every
`NPCBestiaryDrawOffset` row with `Hide = true`; 328's row is Hide
(our verbatim port `bestiary_draw_offsets.rs:384`), so
`AddEmptyEntries_CrittersAndEnemies_Automated` (`:851-868`) never
registers a 328 entry. The kill credit itself is faithful:
`IsNPCValidForBestiaryKillCredit` (`NPC.cs:78901-78912`) excludes only
121 + the non-boss EoW heads — 328 passes, and
`NPCKillsTracker.RegisterKill` (`NPCKillsTracker.cs:23-33`) counts the
"PumpkingBlade" credit id with no entry requirement. **The card is the
only delta.** The wiring-round merger applies this exact hunk set
(all anchors verified unique at `1ba0395c` by
`tools/_d38_retire_check.py` R2, RUN):

1. **`crates/tr-app/src/bestiary.rs:2256-2265`** — DELETE the ENTRIES
   row (the `// W4-d38 b6 — BestiaryEntry.Enemy(328)` comment + the
   `BestiaryEntryDef { credit_id: "PumpkingBlade", net_id: 328, … }`
   block). 284→283 entries on this branch (the union E4 pin 489→488).
2. **`crates/tr-app/src/bestiary.rs` `entry_for_npc` (`:2656-2728`)** —
   ADD the credit-preserving guard: the kill-credit loop
   (`world_sim/npcs.rs:5283`) calls `entry_for_npc` for EVERY dead NPC
   and the blade's silent despawn sets `hp = 0`
   (`npc_ai_style59.rs:255`), so a bare delete PANICS the death pass.
   The faithful shape is the `try_entry_for_npc` split: the sim's
   kill-credit + module-4 broadcast keep registering the
   "PumpkingBlade" credit (vanilla does), so EITHER (a) the death loop
   moves to `try_entry_for_npc` with a name-row fallback for the
   credit (the `ContentSamples` idiom), OR (b) `entry_for_npc` keeps a
   static stand-in row for 328 outside ENTRIES. Option (a) is the
   cleaner seam — `netmodules.rs:457-460` already tolerates a missing
   kind via `kind_from_net_id` (which reads ENTRIES — after the drop
   it returns None for 328 and the client's kill-count arm
   early-returns; the vanilla client also has no 328 entry, but its
   tracker still counts — the (a) fallback must cover this arm too).
   **The merger owns this choice; both are witnessed edits.**
3. **`crates/tr-app/src/assets.rs:1943`** — drop `328,` from the
   `BESTIARY_NPC_SHEET_IDS` d38 append row; `:1905` declared
   `[u16; 216]` → `[u16; 215]`.
4. **`crates/tr-app/src/assets.rs:2720`** — drop the parallel
   `atlas.sheet("NPC_328").copied(),` vec row (216→215 rows; the
   first 181 + the other 34 d38 rows keep their indexes — 328 is the
   10th of the 35-row append, so the 25 later d38 slots shift by one;
   no literal slot reads exist, every read rides
   `Lookups::bestiary_sheet_index`).
5. **`crates/tr-app/src/ui/bestiary.rs:1647-1652`** — DELETE the
   `(NpcKind::PumpkingBlade, _)` portrait arm (no entry can reach it
   after 1; the exhaustive-match wildcard absorbs the kind).
6. **Witness re-pins (same commit)**:
   `bestiary.rs:2851` — remove `K::PumpkingBlade` from the 35-kind
   ladder list (`:2855` `assert_eq!(d38.len(), 35)` → 34) and
   `:2889-2896` — the GAP-3 pin flips: keep the
   `bestiary_draw_offsets::get(328).unwrap().hide` assertion, replace
   the `ENTRIES.iter().any(|e| e.net_id == 328)` positive pin with
   `!ENTRIES.iter().any(…)` (the drop landed);
   `ui/bestiary.rs:2255` — remove the `(328, 106, 86, 1)` strips row
   (`:2259` len 35→34).
7. **NOT dropped** (the in-world kind stays — only the bestiary card
   goes): the `STATIC_SHEET_NAMES` "NPC_328" row (`assets.rs:1289`),
   `D38_B6_SHEETS` (`render/entities.rs:1363`), the draw arm
   (`entities.rs:3371`), the rotation-whitelist membership (8-IN
   unchanged — the blade's style-59 rotation writer is in-world), the
   `w4_d38_b6_*` SetDefaults/frame witnesses, the kill-credit census
   (§8's "credit faithful" finding stands), the 108-E3 dedupe (keep
   d38@328 drop c6@328 — the KIND ownership, independent of the card).

Post-drop arithmetic (R4 verified): this branch 283 entries / 215
slots; the merger pins move 216→215 and the §4b union 323→322 (**the
108-E "323 final" pin is SUPERSEDED by this drop — the wiring-round
merger must re-pin 108's §5 target to 322**, or equivalently record
the -1 against the E4 count pins: 489→488 entries, ≤357→≤356 slots).
The 35-kinds/35-sheets pins are UNCHANGED (the kind + its in-world
sheet stay).

## 13. RETIRE: GAP-1 closure receipt + the d38-band SEAL declaration

### 13a. GAP-1 (the critter sight provider) — CLOSED by c8, d38's six auto-covered

The §8 GAP-1 wiring-round item is ALREADY LANDED on
`task/W4-c8-style66-crawling-critters`: the sweep's
`scan_world_for_critter_sights` (`a17cd75b8`, at the `update_npcs`
head — `Main.cs:17914` before the UpdateNPC loop, the 300×200-inflated
player box) and the FINAL generalization (`af2a06880`, local FINAL
tip **`af1f8ba1d`** — the pushed origin tip `3705670d3` carries the
nine-kind-list form; the generic gate is on the local final commits,
the merger reads the branch tip). Receipt
(`tools/_d38_retire_check.py` R3, RUN, ALL PASS):

- The c8 FINAL dispatch is the GENERIC C# gate, not a kind list:
  `counts_as_a_critter(kind)` = `kind.max_hp() <= 5 &&
  kind.contact_damage() == 0 && kind.cs_id() != 594 && kind.cs_id() !=
  686` (`bestiary.rs:2280-2285` @ `af1f8ba1d`, the
  `NPC.CountsAsACritter` port `NPC.cs:6887-6897`);
  `is_critter_entry` = the gate + the one 549 Hide exclusion
  (`:2303-2305`); `unlock_state_world` dispatches critter entries to
  `critter_unlock_state` (sight → tier 4,
  `CritterUICollectionInfoProvider.cs:14-20`).
- **All six d38 critters pass the gate on their SetDefaults rows**
  (mechanically extracted from `npc/data.rs` `max_hp`/`contact_damage`):
  297 BirdBlue 5/0, 298 BirdRed 5/0, 299 Squirrel 5/0, 300 Mouse 5/0,
  303 BunnySlimed 5/0, 337 BunnyXmas 5/0 — none is 594/686. **No other
  d38 kind passes** (the 29 non-critters' rows are all >5 HP or >0
  damage; the full 35-row table in the R3 output). The live-NPC scan
  gate (`n.max_hp > 5 || n.kind.contact_damage() != 0`) is the same
  predicate on the spawned rows — `Npc::new` copies the kind's
  `max_hp` (`npc.rs:2998-2999`), no d38 kind overrides damage.
- The six credit ids ("BirdBlue"/"BirdRed"/"Squirrel"/"Mouse"/
  "BunnySlimed"/"BunnyXmas") are ENTRIES rows on this branch with no
  `entry_for_npc` fold — the scan's
  `entry_for_npc(n.kind, n.slime).credit_id` lands in the same sights
  set the dispatch reads. None of the six is a `GOLD_PEER_TABLE` key
  (the gold arm is irrelevant to them; the vanilla peer registrations
  for 442 GoldBird→"Bird" / 443 GoldBunny→"Bunny" / 447 GoldMouse→
  "Mouse" / 539 SquirrelGold→{"Squirrel","SquirrelRed"} reference the
  BASE Bird/Bunny kinds + our 299 Squirrel as PEERS — d40's gold rows
  read our sightings for free, the cross-lane contract works in both
  directions).
- **d38 wiring-round action: NONE.** The c8 branch's own FINAL census
  (research/143 §1) states the contract: "do NOT add per-lane sight
  scans or provider arms". At merge, the six entries auto-ride the
  sight provider. The merge-time edit is on the UI side only: this
  branch's three kill-only `unlock_state(entry, tracker)` call sites
  (`ui/bestiary.rs:510/:537/:1803`) fold onto c8's
  `unlock_state_world` (the same fold c8 did for its own UI rows —
  the 143 §1 merge note); the §8 witness's critter block
  (`bestiary.rs:2879-2888`, the kill-ladder pins) re-pins to the
  sight form at that edit. GAP-1's "pre-existing base-wide" scope
  also closes for the base Goldfish 55 (the c8 FINAL lifted the
  freeze).
- GAP-2 (rarity stars, 5 kinds) stays OPEN — display-only, no lane
  has claimed it; the wiring-round item stands as §8 recorded.

### 13b. The SEAL declaration

**The d38 band is COMPLETE and SEALED** (this doc's status row; the
RETIRE receipts above close every d38-owned wiring item):

- **35 kinds** — `NpcKind` + SetDefaults + AI + cs_id/to_u16 rows,
  protocol==vanilla in the 281-337 corridor (§9, the 108-E
  confirmation; the 22 holes are base-owned, the two E3 dedupes
  recorded).
- **35 sheets** — the real-dims strips packed (`c096bac9`), the
  draw arms wired with no None-skip (§4a), the eager→lazy migration
  is d35's 113 §10.6 step-7 (no d38 code change, §6).
- **35 portraits** — ENTRIES rows + `portrait_data` arms + the
  181→216 slot close (§4b); the ONE vanilla-exclusion delta (328) is
  the §12 exact drop-hunk, owned by the wiring-round merger.
- **Merger hand-off** (the complete list, per 108-E + 113-lazy):
  1. the §11 five stand-in drop-hunks + the witness re-pin (the
     residual 294/296 wait on the wave-B wiring batch — NOT d38's
     work);
  2. the §11 keep-one dedupes (10 faithful twins + the 303
     correction: 303 KEEPS the d38 inline arm);
  3. the §12 328 card drop (7-part hunk, the credit-preserving guard
     choice owned by the merger, the 323→322 / 489→488 re-pins);
  4. the §13a UI fold onto `unlock_state_world` (GAP-1 closed by c8,
     zero d38 provider work);
  5. the 113 §10.6 step-7 lazy migration (d35's seam, d38's rows ride
     `texture_net_id` unchanged);
  6. GAP-2 rarity stars (unclaimed, display-only).
- **Verification record (RETIRE)**: `tools/_d38_retire_check.py` RUN,
  **44/44 PASS** (R1 the claim sets/hooks/residual at `c863a7a5`, R2
  the six unique drop anchors + the panic-path proof, R3 the generic
  gate + the six-critter coverage + the credit rows, R4 the census
  pins 284/216/216/216 + the post-drop arithmetic); output
  `tools/_d38_retire_check.txt` committed. No cargo/tests/rustfmt
  (code-only); no game-code change this phase — the drop-hunks are
  the MERGER's edits, published exact so they are mechanical.
