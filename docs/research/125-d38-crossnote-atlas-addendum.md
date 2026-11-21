# 125 — d38 crossnote ADDENDUM to the atlas-repack plan (research/113)

Status: 📣 published INPUT for `task/W4-d35-npc-kinds-71-140`'s ATLAS-REPACK
phase (d35c, tips `519c8035`/`7d131be3` read 2026-11-21) and the d40b sheets
phase (`task/W4-d40-npc-kinds-421-490`, data-only tip `8fe1d6d3` read same
day). Branch: `task/W4-d38-npc-kinds-281-350` (crossnote phase). Base:
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
