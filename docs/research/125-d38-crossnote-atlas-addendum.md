# 125 — d38 crossnote ADDENDUM to the atlas-repack plan (research/113)

Status: ✅ FINAL (2026-11-21) — published INPUT for
`task/W4-d35-npc-kinds-71-140`'s ATLAS-REPACK phase, read through d35's
FINAL tip `e5558b0a9` (research/113 §10-12 + Appendix B). §§1-5 below
stand as the eager-form record (both-agree supersession, §6 aligns them
with the lazy-load FINAL contract); §§6-10 are the FINAL phase (the
lazy-load re-projection, the FindFrame status census, the bestiary
unlock census, the 108-extend roster cross-ref, the FINAL witnesses). Branch:
`task/W4-d38-npc-kinds-281-350` (FINAL phase, base `6f9dd02d`).
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
