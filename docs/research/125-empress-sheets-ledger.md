# 125 — W4-d125 sheet additions: the empress overflow-ledger entry

Status: ✅ measured + packed SOLO; the cumulative reconciliation belongs to
the atlas-repack wave. Branch: `task/W4-d125-empress-r7-drain` (remainder 1).
Code-only mode: no cargo/tests run; every number below is measured by the
pinned sim harness over the REAL retail `.xnb` dims (LZX first-frame
decoder, `tools/_d125_atlas_sim_pinned.js` — a byte copy of d35's tip
`tools/_d35_atlas_sim_pinned.js`), with `verify()` (no overlap incl. PAD,
in-bounds) and double-run determinism on every layout.

Read FIRST: `docs/research/113-atlas-repack-plan.md` on
`origin/task/W4-d35-npc-kinds-71-140` (commit `519c8035`, the ATLAS-REPACK
phase) — the crisis census, the packer verdict (keep skyline) and the
two-page merger contract. This note is d125's entry in that wave's
overflow ledger.

## The two sheets (real dims, `tools/_d125_sheets_measure.js`)

| sheet | w×h | padded | rows (`Main.npcFrameCount`) | row h |
|---|---|---|---|---|
| `NPC_636` (Empress of Light) | 166×352 | 168×354 | 2 (`Main.cs:67147`) | 176 |
| `NPC_661` (Prismatic Lacewing) | 24×72 | 26×74 | 3 (`Main.cs:67150`) | 24 |

Both divide into whole rows. Padded area 0.061 Mpx ≈ 8 px of bin width.

## Measured heights (width 8192, PAD 2, strict-GPU bound 8192)

`tools/_d125_sheets_measure.js` → `tools/_d125_sheets_measure.json`
(skyline waste tie-break, the landed packer; every scenario verified +
double-run deterministic):

| scenario | sheets | skyline | verdict |
|---|---|---|---|
| base (d125 HEAD `d8047a10`, no sheet adds) | 9497 | **8115** | headroom 77 |
| base + boot font | 9498 | **8119** | headroom 73 |
| base + NPC_661 solo | 9498 | 8113 | headroom 79 (valley fill: −2) |
| base + font + NPC_661 solo | 9499 | 8118 | headroom 74 |
| base + NPC_636 solo | 9498 | 8118 | headroom 74 (+3) |
| base + font + NPC_636 solo | 9499 | 8125 | headroom 67 (+6) |
| **base + both (636 then 661)** | 9499 | **8116** | **headroom 76 (+1)** |
| **base + font + both** | 9500 | **8117** | **headroom 75 (−2)** |
| base + d35's pushed union (198) + both | 9697 | 8541 | OVERFLOW +349 |
| base + font + pushed union + both | 9698 | 8541 | OVERFLOW +349 |

**Verdict: PACK both** (not a None-skip). The pair costs +1 px main / −2 px
boot-font on this branch's base — the 661 strip fills a skyline valley the
636 strip opens. The d125 base (8115/8119) matches d35's sheets-phase HEAD
re-measurement exactly (same wave base `a725acb0` + no sheet adds on either
side before the packs).

## Overflow-ledger context (the repack wave owns the union)

- The union scenario above uses d35's OLD 17-branch/198-name extraction
  (`tools/_d125_pushed_sheets.json`, copied from d35's sheets phase). d35's
  ATLAS-REPACK phase re-extracted **26 branches / 339 names** and measured
  the union at skyline **9382** (+1190), area bound **8835** — already
  un-fittable on one page WITHOUT d125 (research/113 §1).
- d125's two names are NEW to that union (no other pushed branch packs
  NPC_636/NPC_661 — checked `git grep` over the pushed tips; d43
  (631-697 band) is data-only at the wave base with no `assets.rs` diff).
  **Ledger addition: +2 names, +0.061 Mpx padded (≈ +8 px area bound),
  owner `task/W4-d125-empress-r7-drain`.**
- Priority note for the merger: LOW area pressure (rank ~300 of 409 by
  padded area — both are small strips), but HIGH visibility (a major boss
  + its summon critter; the draw arms are live and currently rest on
  `npc_sheet == None` only in content-less builds). Under research/113's
  incremental two-page form they append to page 1's input list; page-1
  height impact ≈ +0-8 px against the measured 1924 px (23.5% of a page).
  They may merge BEFORE the page plumbing lands only while the merged
  main stays ≤ 8192 solo — after the d35/d37/d38 sheet waves merge, the
  page plumbing must land with or before them (the union is over the
  bound regardless of d125).

## What landed on this branch

- `assets.rs`: `NPC_636`/`NPC_661` in `STATIC_SHEET_NAMES` (id order);
  `BESTIARY_NPC_SHEET_IDS` 181 → 183 (+ the parallel `bestiary_npc_sheets`
  vec rows, appended last — the earlier slots keep their indexes).
- `render/entities.rs`: `D125_SHEETS = [636, 661]` chained into
  `npc_sheets`; the two draw arms' deferral comments retired; the packing
  witness `the_d125_strips_divide_into_their_vanilla_rows` (real dims,
  whole rows, sheet-width frames, both tables carry the slots).
- `ui/bestiary.rs`: the two portrait arms ride the sheets' own row heights
  AND widths (`SetFrameSize`'s `frame.Width = tex.Width()`, the d35
  shape); the witness `the_d125_portraits_ride_the_real_sheet_dims`.
- Rotation whitelist: 661 already rides it (the `FindFrame` case-661
  `vx * 0.3` tilt, `NPC.cs:72747`). 636 stays OUT — the pure core
  (`npc_ai_empress.rs`) ports no `this.rotation` write (the C# machine's
  rotation writes at `NPC.cs:47082+` belong to AI_119 Dandelion, past
  `AI_120`'s `:46982` end), so there is nothing to whitelist yet.

Witnesses WRITTEN NOT RUN (code-only mode).

## 2. The draw phase's overlay census (the HallowBoss arm's six strips)

`Main.DrawNPCDirect_HallowBoss` (`Main.cs:26471-26652`) reads six
`TextureAssets.Extra` strips (`AssetInitializer.cs:363-365`,
`Images/Extra_<id>`). Real dims + the greedy fit on the both-packed base
(the harness's `overlay PICK/SKIP` rows):

| sheet | role (C# line) | w×h | solo height main/font | verdict |
|---|---|---|---|---|
| `Extra_157` | the shader-pass body copy (`:26482`/`:26602`) | 195×1221 | 8330/8330 (+138) | DEFER |
| `Extra_158` | the left arm, 7 frames (`:26480`) | 174×1204 | 8256/8256 (+64) | DEFER |
| `Extra_159` | the wing strip, 11 frames (`:26477`) | 195×1221 | 8330/8330 (+138) | DEFER |
| `Extra_160` | the right arm, 7 frames (`:26481`) | 174×1204 | 8256/8256 (+64) | DEFER |
| `Extra_187` | the phase-2 wings, 8 frames (`:26624`) | 178×1488 | 8208/8208 (+16) | DEFER |
| `Extra_188` | the halo ring (`:26635`) | 166×174 | **8141/8141 (+25)** | **PACK** |

(The solo heights above were measured on the pre-Extra_188 base at
8116/8117; the final three-sheet set re-measures 8141/8141 — 51 px
headroom, verify() + determinism. The harness's `base` scenarios now
read the landed assets.rs, so they re-measure the final state.)

**Ledger addition (draw phase): +1 packed name (`Extra_188`, +0.030 Mpx),
+5 deferred names (`Extra_157/158/159/160/187`, +0.985 Mpx — the five
tall strips are ~1.0 Mpx of page-1 pressure, rank ~40-60 by area).**
The deferred five are the arm's wings/arms/shader-body — HIGH visibility
(the boss draws body-only until they land); they admit on page 1 under
research/113's form. The pure draw geometry (the arm-frame picker, the
state factors, the ghost rings, the halo offsets) is ported NOW in
`render/empress_draw.rs` so the repack wave's wiring is sheet-only.
