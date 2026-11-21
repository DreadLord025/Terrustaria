# 64b — the CONSOLIDATED v18 claimant registry (the final-union input)

**Branch:** `task/W4-g72-bg-layers-3` (closeout, continues DONE `4a75715e` +
item 1 `ba6da402e`). **Mode:** code-only — no cargo/tests/graphify.
**Audience:** the g71c union lane and the f44e persist-merge lane — this note
is the MECHANICAL final-union input: every lane's exact bytes, offsets and
C# anchors in ONE table, so the union computes without another round-trip.

## 0. The g71c union-final status (read at this probe, 2026-11-21)

`origin/task/W4-g71-bg-layers-2` tip **`f08e8d41e`** ("W4-g71 WIRING docs:
research/92 §1 five-branch v18 UNION extension") — research/92
(`92-doc-sweep-bg.md`) §1 WAS extended, but only to **FIVE** claimants
(g70/g71/g69/d125/d109): a **21-byte fixed head + W×H dense variants**.
Our 31-byte cave claim (research/64a, `108cc0934`) and g90's 2-byte book
pair are **NOT seated** in it. The rolls are WIRED on that branch
(`worldgen/bg_rolls.rs` → `step_world_tail`, `c13de6ee9`).

`origin/task/W4-f44-save-plr-2` tip **`bdd1cc79a`** — research/132
(`132-persist-master-map.md`, the persist MASTER INDEX) §3-L3 reconciles
**SEVEN** claimants (adds g72 31 + g90 2 + c43 44) as a "97-byte head".
**Two errors this registry corrects:**

1. **Arithmetic:** 16+31+3+1+1+2+44 = **98**, not 97 (the §3-L3 offset table
   `[54..97]` for the 44-byte identity block is internally inconsistent —
   54+44 = 98). Every "97" in 132 §3-L3 (head size, `v18_tail_len`, the
   truncation moves) must read **98** for that seven-claimant set.
2. **Missing claimants:** 132 §3-L3 omits **g87's 3-byte pet-license trio**
   and the **5-byte seasonal moon-boss block** (claimed TWICE — c39 AND f39,
   identical bytes). With those seated the head is **106**, not 97/98.

## 1. The claimant registry (eleven branches, ten distinct blocks)

Every row read directly off the branch's own `save.rs` writer + reader at
the tip named. `W:`/`R:` are the `// v18:` writer comment / `if version >= 18`
reader gate line numbers on that branch. All blocks append after the v17
`downedFishron` byte; all readers gate `version >= 18`; VERSION stays 18.

| # | Lane / branch | Tip | Bytes | Fields (order = write order) | C# anchor | W: / R: | Pre-v18 default |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | g70 `task/W4-g70-bg-layers-1` | `752655914` | **16** | `tree_x[0..3]` 3×i32 LE + `forest_bg_styles[0..4]` 4×u8 | `WorldFile.cs:1253-1259` (treeX/treeStyle), `:1318`/`:1377-1379` (setBG0/10/11/12) | 431 / 824 | `tree_x=[width;3]`, styles `[0;4]` |
| 2 | **g72 (OURS)** `task/W4-g72-bg-layers-3` | `ba6da402e` | **31** | `cave_back_x[0..3]` 3×i32 + `cave_back_style[0..4]` 4×i32 + ice/jungle/hell u8 | `WorldFile.cs:1260-1269` (hell `:1269` rides our block; `caveBackX[3]` never persisted) | 439 / 838 | `World::new` zeros (NOT the Reset band defaults — the boot roll is a dead store, research/64 §2) |
| 3 | g71 `task/W4-g71-bg-layers-2` | `f08e8d41e` | **3** | snow/desert/ocean_bg_style 3×u8 | `WorldFile.cs:1321`/`:1324-1325` (setBG3/6/7) | 434 / 826 | all 0 |
| 4 | g69 `task/W4-g69-weather-clouds-parallax` | `20d0380f9` | **1** | `crimson` u8 bool | `WorldFile.cs:1281`; `:3507` corruption fallback | 434 / 823 | false |
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

## 2. The FINAL union layout (ten distinct blocks, 106-byte head + dense)

One fixed shared order, extending research/92 §1's forest-first rationale
and 64a §2's C#-field-order seating: the bg family first (forest → cave →
surface, the C# `:1253→:1260→:1321` order), then the flag singles in C#
field order (crimson `:1281` → seasonal `:1347-1351` → pets `:1392-1394` →
empress `:1395` → books `:1407-1408`), then the identity block (header
fields, not bg-family — closes the fixed head), then d109's variable-length
dense block LAST (self-delimiting by `tiles.len()`):

```
… v17 downedFishron byte (1)
v18 UNION tail — fixed head (106 bytes) + dense variants (W×H):
  [0..16]    g70 forest block       3×i32 tree_x + 4×u8 styles      (#1)
  [16..47]   g72 cave block         3×i32 bands + 4×i32 styles
                                    + ice/jungle/hell u8            (#2, 64a §1)
  [47..50]   g71 surface bytes      snow/desert/ocean u8            (#3)
  [50]       g69 crimson            u8                              (#4)
  [51..56]   seasonal moon-boss 5   u8×5 (c39≡f39, keep one)        (#8/#9)
  [56..59]   g87 pet licenses       u8×3                            (#7)
  [59]       d125 downedEmpress     u8                              (#5)
  [60..62]   g90 book-two + satchel u8×2                            (#6)
  [62..106]  c43/f45 identity       u64 + 16B GUID + i32 + 2×i64    (#10)
  [106..106+W*H]  d109 dense variants (LAST, self-delimiting)       (#11)
```

**Fixed head = 106 bytes: 16+31+3+1+5+3+1+2+44. Total = 106 + W×H.**

Ordering notes:
- The flag singles sit in strict C# field order (`:1281 < :1347 < :1392 <
  :1395 < :1407`) — this DIFFERS from 64a §2's recommended five-claimant
  order (which put crimson after the surface group and books last) only by
  the insertion of the seasonal/pet/empress singles between them; the
  multi-byte bg blocks keep their 64a seats verbatim.
- 132 §3-L3's order (crimson → empress → books → identity) is preserved as
  a subsequence; the seasonal + pet blocks insert at their C# seats.
- The `underworldBG` seat (64a §3's flag, C# `:1376`) stays OPEN: if g71c's
  roll-wire persists it, insert 1 byte between the surface group and
  crimson → **107**-byte head, re-pin once (the same stale-by-design
  pattern as the moon roll).

## 3. Merge instructions (mechanical)

- **Writer:** after the v17 fishron byte, write the ten blocks in §2's
  order; d109's dense block is the LAST thing `write_world` emits.
  **Reader:** one `if version >= 18` arm reading all 106 + `tiles.len()`.
- **Truncation constants → 106:** OUR eleven sites currently carry `+ 31`
  (`save.rs:1111/:1207/:1285/:1349/:1378/:1567/:1588/:1610/:1726/:1846/
  :1961/:2088/:2189` — the EMPTY_V16_TAIL family, the v12/v14 tail
  arithmetic and the six mid-record cuts re-armed in `2b02345c`); each
  moves 31 → 106. g70/g71's 16/3 → 106 (92 §1.5's table), g69/d125's 1 →
  106, g90's 2 → 106 (135 §4), g87's 3 → 106, c39/f39's 5 → 106 (ONE set of
  sites after the dedup), c43/f45's 44 → 106. d109's end-relative sites
  need only `v18_tail_len`'s new return value (`106 + width*height`).
- **Back-compat:** unchanged from 92 §1 / 64a §3 — no released v18 exists,
  the union is forward-only from the merge; pre-v18 files keep each
  claimant's own defaults (§1's last column).
- **The roll keep-one (132 §3-L3 item 2) is UNAFFECTED by this registry:**
  g71c's `bg_rolls.rs` vs our `cave_bg.rs` adjudication stays owed at the
  merge; the persistence trio is our pushed shape either way.

## 4. Witnesses

Code-only: no new executable witness in this note (it is a merge
instruction). Our `w4_g72_the_cave_background_roll_round_trips_in_v18`
(`save.rs:1264`) pins the 31-byte block end-to-end and stays green through
the merge with its truncation constant moved to 106. The c39≡f39 dedup is
byte-identical by construction (§1 #8/#9 — same fields, same order, same
C# lines), so either lane's seasonal round-trip witness pins the kept block.