# 64b — the CONSOLIDATED v18 claimant registry (the final-union input)

> **SEALED (W4-g72 retire, 2026-11-21).** The registry's job is done: both
> consumers (g71 92 §1-FULL, f44 132 §3-L3 FINAL-FINAL) published the
> 107-byte union. The layout AUTHORITY is **g71's research/92 §1-FULL.2**;
> this note is the per-lane byte/anchor input table + the audit trail.

> **RETIRED-UPDATE (W4-g72 retire, 2026-11-21): the head is 107, not 106.**
> The g71 underworld seat FIRED — g71 rolls AND `.wld`-persists
> `underworld_bg_style` (§0's receipt) — so the 1-byte conditional below is
> seated and every 106 in this note moves to **107**. **AUTHORITY for the
> final layout is g71's research/92 §1-FULL** (`92-doc-sweep-bg.md` at tip
> `f2d8c2c67`): §2 below is re-pinned to its offsets (the 64a 53-byte prefix
> kept VERBATIM, books inside it). This note stays the per-lane
> bytes/C#-anchor/W:-R: input table; the layout it points at is 92 §1-FULL.2.

**Branch:** `task/W4-g72-bg-layers-3` (closeout, continues DONE `4a75715e` +
item 1 `ba6da402e`; retire update on `fafd0659`). **Mode:** code-only — no
cargo/tests/graphify.
**Audience:** the g71c union lane and the f44e persist-merge lane — this note
is the MECHANICAL final-union input: every lane's exact bytes, offsets and
C# anchors in ONE table, so the union computes without another round-trip.

## 0. The union-final status — the RETIRE receipt (re-probed 2026-11-21)

**The underworld-1B receipt (the Lead's retire item 1).** Our closeout probe
read g71 at `f08e8d41e` and left the `underworldBG` seat OPEN (§2's old
conditional). At the CURRENT tip **`f2d8c2c67`** ("research/92 section
1-FULL 13-branch claimant registry + 107-byte union") the seat is CLAIMED
and FIRED, verified on g71's own code:

- **Roll:** `bg_rolls.rs:168` — `underworld: rng.roll_range(0, 3)` (the
  `RandomizeBackgrounds` last draw, `WorldGen.cs:8011`), applied at `:311`
  via `world.set_surface_bg_style(9, styles.underworld)` →
  `world.rs:1590` sets `underworld_bg_style` (field `world.rs:154`,
  `World::new` default 0 at `:754`).
- **`.wld` persistence:** `wld/sections/flags.rs:186` writes
  `w.u8(world.underworld_bg_style)` and `:577` reads it back
  (`WorldFile.cs:1376`). OUR branch still carries the stand-ins —
  `flags.rs:188` zero-write / `:573` skip-read — g71's pair supersedes them
  at the merge (its lane owns the byte).
- **`.trwld`:** g71's own `save.rs` still writes only the 3 surface bytes
  (W:434/R:826) — the underworld `.trwld` byte materialises at the union
  merge per 92 §1-FULL.2 offset **[58]** (a rolled-and-`.wld`-persisted
  style with no `.trwld` byte is exactly the loss class the union prevents,
  f44 research/132 §3-L3's FIRED-seat note).

`origin/task/W4-f44-save-plr-2` tip **`d35876af9`** — research/132 §3-L3 is
now the FINAL-FINAL **107-byte** ten-claimant union (it seats the same
underworld byte and confirms our 98-not-97 arithmetic correction). Its
flag-single ORDER differs from 92 §1-FULL.2 (it puts underworld at [50] and
books at [61..63]); both are 107 + W×H. **Per the Lead, 92 §1-FULL is the
layout authority** (it keeps our published 64a prefix offsets stable);
132 §3-L3 agrees on every byte count and the truncation arithmetic.

**Historical probe (the closeout read, superseded by the above):** g71 at
`f08e8d41e` carried only FIVE claimants / a 21-byte head (our 31 + g90's 2
unseated); f44 at `bdd1cc79a` reconciled SEVEN as a "97-byte head" — this
registry corrected it twice: (1) 16+31+3+1+1+2+44 = **98**, not 97;
(2) g87's 3-byte pet trio + the c39≡f39 5-byte seasonal block were unseated
(→ 106). Both corrections were absorbed upstream (92 §1-FULL, 132
FINAL-FINAL); with the underworld byte the final head is **107**.

## 1. The claimant registry (twelve rows, eleven distinct blocks)

Every row read directly off the branch's own `save.rs` writer + reader at
the tip named (row 3a off g71's worldgen/wld code — its `.trwld` byte seats
at the union merge, §0's receipt). `W:`/`R:` are the `// v18:` writer
comment / `if version >= 18` reader gate line numbers on that branch. All
blocks append after the v17 `downedFishron` byte; all readers gate
`version >= 18`; VERSION stays 18. The tips are the closeout-probe reads;
where a tip has since moved with the SAME bytes (g90 `740650d75`, c39
`7dd785dc5`, g71 `f2d8c2c67` — re-checked at the retire probe) the claim is
unchanged. 92 §1-FULL.1 additionally seats **e19**
(`task/W4-e19-liquid-quicksell-shimmer`, tip `e7e466d46`) as a THIRD dedup:
its `save.rs:438` writes the same `crimson` byte as #4 (its comment itself
cites W4-g69) — one byte, g69 the claimant of record.

| # | Lane / branch | Tip | Bytes | Fields (order = write order) | C# anchor | W: / R: | Pre-v18 default |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | g70 `task/W4-g70-bg-layers-1` | `752655914` | **16** | `tree_x[0..3]` 3×i32 LE + `forest_bg_styles[0..4]` 4×u8 | `WorldFile.cs:1253-1259` (treeX/treeStyle), `:1318`/`:1377-1379` (setBG0/10/11/12) | 431 / 824 | `tree_x=[width;3]`, styles `[0;4]` |
| 2 | **g72 (OURS)** `task/W4-g72-bg-layers-3` | `ba6da402e` | **31** | `cave_back_x[0..3]` 3×i32 + `cave_back_style[0..4]` 4×i32 + ice/jungle/hell u8 | `WorldFile.cs:1260-1269` (hell `:1269` rides our block; `caveBackX[3]` never persisted) | 439 / 838 | `World::new` zeros (NOT the Reset band defaults — the boot roll is a dead store, research/64 §2) |
| 3 | g71 `task/W4-g71-bg-layers-2` | `f08e8d41e` | **3** | snow/desert/ocean_bg_style 3×u8 | `WorldFile.cs:1321`/`:1324-1325` (setBG3/6/7) | 434 / 826 | all 0 |
| 3a | **g71 underworld seat (FIRED — §0)** | `f2d8c2c67` | **1** | `underworld_bg_style` u8 (setBG 9) | `WorldFile.cs:1376`; roll `WorldGen.cs:8011` | rolled `bg_rolls.rs:168`/`:311`; `.wld` flags.rs:186/:577; `.trwld` seats at the union (§0) | 0 (`world.rs:754`) |
| 4 | g69 `task/W4-g69-weather-clouds-parallax` | `20d0380f9` | **1** | `crimson` u8 bool (e19 `e7e466d46` DUP — keep g69) | `WorldFile.cs:1281`; `:3507` corruption fallback | 434 / 823 | false |
| 5 | d125 `task/W4-d125-empress-r7-drain` | `d8047a109` | **1** | `downed_empress_of_light` u8 bool | `NPC.cs:6393`, case-636 `:79351-79353`; `WorldFile.cs:1395` | 431 / 817 | false |
| 6 | g90 `task/W4-g90-artisan-bread-use` | `94d33f2ca` | **2** | combat_book_volume_two_was_used + peddlers_satchel_was_used u8×2 | `WorldFile.cs:1407-1408` | 435 / 832 | false×2 |
| 7 | g87 `task/W4-g87-demon-heart-use` | `dfbcaf94b` | **3** | bought_cat/bought_dog/bought_bunny u8×3 | `WorldFile.cs:1392-1394` (the 4829/4830/4910 Pet License unlocks) | 434 / 826 | false×3 |
| 8 | c39 `task/W4-c39-lantern-birthday` | `8ecb288aa` | **5** | downed_halloween_king/tree + christmas_ice_queen/santank/tree u8×5 | `WorldFile.cs:1347-1351`; `NPC.cs:79210-79238` death arms | 437 / 831 | false×5 |
| 9 | f39 `task/W4-f39-weather-slimerain` | `bd5a5a24a` | **5** | **IDENTICAL to #8** — same five fields, same order, same C# anchor | `WorldFile.cs:1347-1351` | 438 / 832 | false×5 |
| 10 | f45 `task/W4-f45-wld-header-2` (carries c43 `076970809`) | `6352d3840` | **44** | generator_version u64 + unique_id 16×u8 + world_id i32 + creation_time i64 + last_played i64 | `WorldFile.cs:1225-1227` (header) + `:1250-1251` (SaveWorldFlags stamps) | 438 / 832 | `World::new` zeros |
| 11 | d109 `task/W4-d109-framing-genrand-a3` | `81894095e` | **W×H** | `variants` dense u8, row-major (port-only: `Tile.bTileHeader2` bits 4-7, `Tile.cs:464-482`; the C# `.wld` never writes them) | port-only | 440 / 836 | zeros + `variants_from_save=false` (re-roll) |

**Dedup verdicts (mechanical, no adjudication needed):**
- **#8 ≡ #9:** c39 and f39 write the SAME five bytes in the SAME order from
  the SAME C# lines. Keep ONE block (5 bytes), drop the other writer/reader
  arm at the merge; both lanes' round-trip witnesses keep their assertions
  against the single block. (f39's Phase-B commit also wires the season
  gates — that logic is orthogonal to the persist block and survives.)
- **#10 carries c43:** f45's lineage includes c43's identity block
  (`076970809`); the f45 tip's writer/reader IS the c43 claim — do not seat
  it twice. 132 §3-L3 already treats it as one (the "seventh claimant").

## 2. The FINAL union layout (eleven distinct blocks, 107-byte head + dense)

**AUTHORITY: g71's research/92 §1-FULL.2** (tip `f2d8c2c67`). Its order rule
keeps OUR published 64a 53-byte prefix VERBATIM at [0..53] (forest 16 →
cave 31 → surface 3 → crimson 1 → books 2 — every offset 64a published
stays stable), then appends the remaining claims in C# `.wld` relative
order among themselves: moon bosses (`:1347-1351`) → underworld (`:1376`,
the FIRED seat, §0) → pets (`:1392-1394`) → empress (`:1395`); the identity
block (header fields, no flags position) closes the fixed head; d109's
variable-length dense block LAST (self-delimiting by `tiles.len()`):

```
… v17 downedFishron byte (1)
v18 UNION tail — fixed head (107 bytes) + dense variants (W×H):
  [0..16]    g70 forest block       3×i32 tree_x + 4×u8 styles      (#1)
  [16..47]   g72 cave block         3×i32 bands + 4×i32 styles
                                    + ice/jungle/hell u8            (#2, 64a §1)
  [47..50]   g71 surface bytes      snow/desert/ocean u8            (#3)
  [50]       g69 crimson            u8 (e19 dup drops)              (#4)
  [51..53]   g90 book-two + satchel u8×2 — closes 64a's prefix       (#6)
  [53..58]   seasonal moon-boss 5   u8×5 (c39≡f39, keep one)        (#8/#9)
  [58]       g71 underworld_bg_style u8 — the FIRED seat (:1376)    (#3a)
  [59..62]   g87 pet licenses       u8×3                            (#7)
  [62]       d125 downedEmpress     u8                              (#5)
  [63..107]  c43/f45 identity       u64 + 16B GUID + i32 + 2×i64    (#10)
  [107..107+W*H]  d109 dense variants (LAST, self-delimiting)       (#11)
```

**Fixed head = 107 bytes: 16+31+3+1+2+5+1+3+1+44. Total = 107 + W×H.**

Ordering notes:
- This SUPERSEDES the closeout draft's strict-C#-flag-order layout (books
  after empress): 92 §1-FULL keeps books INSIDE the 64a prefix so our
  published [51..53] offsets never move — the empress byte rides the
  extension group instead (a free move under the forward-only rule, no
  released v18 exists).
- f44 research/132 §3-L3's FINAL-FINAL layout is byte-count-identical
  (107 + W×H) but seats underworld at [50] and books at [61..63]; per the
  Lead, **92 §1-FULL.2 is the layout of record** — the merger follows it.
- The `underworldBG` conditional (64a §3's flag, C# `:1376`) is CLOSED:
  g71 rolls + `.wld`-persists it (§0's receipt), the byte is seated at
  [58], no further re-pin owed.

## 3. Merge instructions (mechanical)

- **Writer:** after the v17 fishron byte, write the eleven blocks in §2's
  order (92 §1-FULL.2's writer snippet is the reference); d109's dense
  block is the LAST thing `write_world` emits.
  **Reader:** one `if version >= 18` arm reading all 107 + `tiles.len()`.
- **Truncation constants → 107:** OUR eleven sites currently carry `+ 31`
  (`save.rs:1111/:1207/:1285/:1349/:1378/:1567/:1588/:1610/:1726/:1846/
  :1961/:2088/:2189` — the EMPTY_V16_TAIL family, the v12/v14 tail
  arithmetic and the six mid-record cuts re-armed in `2b02345c`); each
  moves 31 → 107. g70/g71's 16/3(+1) → 107 (92 §1-FULL.4's table), g69/e19/
  d125's 1 → 107, g90's 2 → 107 (135 §4), g87's 3 → 107, c39/f39's 5 → 107
  (ONE set of sites after the dedup), c43/f45's 44 → 107. d109's
  end-relative sites need only `v18_tail_len`'s new return value
  (`107 + width*height`).
- **Back-compat:** unchanged from 92 §1-FULL.3 / 64a §3 — no released v18
  exists, the union is forward-only from the merge; pre-v18 files keep each
  claimant's own defaults (§1's last column, underworld style 0 included).
- **The roll keep-one is ADJUDICATED (retire receipt, research/64 §7):**
  g71's `fc76a48b3` adopted OUR `cave_bg.rs` VERBATIM and DROPPED its
  `bg_rolls` cave copy — our driver + persistence trio are canonical; no
  double-draw survives the merge.
- **The `.wld` underworld stand-ins retire:** our `flags.rs:188` zero-write
  / `:573` skip-read are superseded by g71's `flags.rs:186`/`:577` pair at
  the merge (that lane owns the byte, §0).

## 4. Witnesses

Code-only: no new executable witness in this note (it is a merge
instruction). Our `w4_g72_the_cave_background_roll_round_trips_in_v18`
(`save.rs:1264`) pins the 31-byte block end-to-end and stays green through
the merge with its truncation constant moved to 107. The c39≡f39 dedup is
byte-identical by construction (§1 #8/#9 — same fields, same order, same
C# lines), so either lane's seasonal round-trip witness pins the kept
block; the e19≡g69 crimson dedup likewise (§1 #4). g71's underworld
round-trip witness (`wld/mod.rs:1754`) pins the `.wld` byte; the `.trwld`
seat [58] is pinned by the union writer/reader at the merge.