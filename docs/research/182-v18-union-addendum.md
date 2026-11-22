> RENUMBERED 64a -> 182 per docs/research/150 (W4-g72 fork; 64 belongs to W4-d64's drawdata-cases-4). Content otherwise unchanged.

# 64-addendum — the W4-g72 31-byte v18 claim, input to the g71c UNION

> **SEALED (W4-g72 retire, 2026-11-21; D5-RE-PIN same day).** The claim (§1)
> landed and is carried verbatim into the FINAL union; the layout authority
> is **g88's research/103 §L3-DEFINITIVE D3/D5** (tip `fc5cf11ae`) via
> research/64b. Nothing owed. OUR cave block's union prefix [0..50)
> (forest 16 + cave 31 + surface 3) is UNAFFECTED by the D5 middle re-order.

> **SUPERSEDED for the final union by research/64b** (the CONSOLIDATED
> claimant registry, W4-g72 closeout; retire-updated to the **107-byte**
> head): the five-claimant 53-byte layout below grew to ELEVEN distinct
> blocks once g87's pet trio, the c39≡f39 seasonal five, d125's empress
> byte, the c43/f45 identity block AND g71's FIRED underworld byte (§3's
> flagged seat — rolled + `.wld`-persisted at g71 tip `f2d8c2c67`) were
> seated. The retire pass pinned the authority to g71's research/92
> §1-FULL.2 (books inside the 53-byte prefix); **the D5-RE-PIN supersedes
> that pin** — g88-103-D5 + g90g's independent C# direct check
> (`WorldFile.cs:1392-1394`→`:1395`→`:1407-1408`) confirm the strict-C#
> middle: §2's draft books tail [51..53] moves to [61..63] in the FINAL
> union (64b §2); OUR 31-byte claim (§1) and the [0..50) prefix rows are
> unchanged. The §3 truncation moves now target **107**, not 53/106.

**Branch:** `task/W4-g72-bg-layers-3` (final phase, continues DONE `2b02345c`).
**Mode:** code-only — witnesses WRITTEN NOT RUN. **Audience:** the g71c
bg-roll-wire lane extending `origin/task/W4-g71-bg-layers-2`'s research/92 §1
(tip `e2d085ae`, the 19-byte forest-first union). This note publishes OUR
exact byte layout and where it sits in the C# `.wld` field order, so the union
grows to five claimants without another round-trip.

## 1. The claim — 31 bytes, exact layout

Our v18 tail (`save.rs` writer `:439-450`, reader `:833-850`, commit
`715c60963`) appends, after the v17 `downedFishron` byte:

```
v18 g72 cave-bg block (31 bytes), offsets relative to the block start:
  [0..4]   cave_back_x[0]      i32 LE   ┐ WorldFile.cs:1260
  [4..8]   cave_back_x[1]      i32 LE   │ :1261
  [8..12]  cave_back_x[2]      i32 LE   ┘ :1262   (caveBackX[3] NEVER persisted)
  [12..16] cave_back_style[0]  i32 LE   ┐ WorldFile.cs:1263
  [16..20] cave_back_style[1]  i32 LE   │ :1264
  [20..24] cave_back_style[2]  i32 LE   │ :1265
  [24..28] cave_back_style[3]  i32 LE   ┘ :1266
  [28]     ice_back_style      u8         WorldFile.cs:1267 (C# writes i32)
  [29]     jungle_back_style   u8         WorldFile.cs:1268 (C# writes i32)
  [30]     hell_back_style     u8         WorldFile.cs:1269 (C# writes i32)
```

**Width deviations from the C# `.wld` (all values are small rolls):**
- `caveBackStyle[0..4]` are C# `int`s (`writer.Write(Main.caveBackStyle[i])`,
  no byte cast) rolled `Next(8)` (`WorldGen.cs:8194` `maxValue = 8`) — we keep
  **i32** for them, same as the C#.
- `iceBackStyle`/`jungleBackStyle`/`hellBackStyle` are C# `int`s rolled
  `Next(4)`/`Next(2)`/`Next(3)` (`WorldGen.cs:8246-8248`) — we compress to
  **u8**. The `.trwld` is our own format; the `.wld` flags half
  (`wld/sections/flags.rs:54-56`) keeps the C# i32 widths verbatim.
- `hellBackStyle` joins this block because the `.trwld` NEVER persisted it
  before (the v6 flags block has no slot — the old v6-block note was wrong,
  corrected in `715c60963`).
- `cave_back_x[3]` stays the `new int[4]` zero (`Main.cs:66696`): the C# never
  writes it (`:1260-1262` is three writes) and the net header never sends it
  (`NetMessage.cs:258-278`).

Pre-v18 loads keep the `World::new` zeros (`world.rs:714-718`) — NOT the
`WorldGen.Reset` band defaults, because the port's boot roll is a dead store
(research/64 §2 correction: `Main.cs:6603` fires once per process before any
load with `maxTilesX == 0`, the load overwrites it).

## 2. Where the block sits — the C# `.wld` field order (five claimants)

Read directly from `decompiled/Terraria/IO/WorldFile.cs` on this branch
(`SaveWorldFlags`):

```
:1252      (byte)moonType            — v6-era, already owned
:1253-1255 treeX[0..3]        i32×3  ┐ g70 forest block, part 1 (12 B)
:1256-1259 treeStyle[0..4]    i32×4  ┘ (g70 compresses to u8×4 — its claim)
:1260-1262 caveBackX[0..3]    i32×3  ┐
:1263-1266 caveBackStyle[0..4] i32×4 │ g72 CAVE BLOCK (28 B here;
:1267      iceBackStyle       i32    │ + hell below; u8 compression
:1268      jungleBackStyle    i32    ┘ makes our tail 31 B total)
:1269      hellBackStyle      i32      g72 (rides OUR block, see §1)
:1270-1280 spawn/layers/time/dungeon  — owned, earlier versions
:1281      WorldGen.crimson   bool   — g69 claim (1 B)
:1282-1317 the downed ladder / invasion / rain / ore tiers — owned
:1318      (byte)treeBG1             ┐ g70 forest block, part 2 (setBG0)
:1319-1320 corruptBG / jungleBG      │ v16-owned on main
:1321      (byte)snowBG              │ g71 surface bytes (3 B:
:1322-1323 hallowBG / crimsonBG      │ snow/desert/ocean)
:1324-1325 (byte)desertBG / oceanBG  ┘
:1326-1374 clouds / wind / angler / banners / events / towers — owned
:1375      (byte)mushroomBG          — v16-owned on main
:1376      (byte)underworldBG        — UNOWNED (g71c roll-wire candidate;
                                       flags.rs:188 zero-write/:573 skip-read)
:1377-1379 treeBG2 / treeBG3 / treeBG4 — g70 forest block, part 3 (3 B)
:1407-1408 combatBookVolumeTwoWasUsed / peddlersSatchelWasUsed — g90 (2 B)
```

**Answer to the g71c question:** the cave arrays sit **between g70's treeX
half and g69's crimson byte** — strictly AFTER all of g70's treeX/treeStyle
and strictly BEFORE g71's snow/desert/ocean, g69's crimson and g90's books.
The full C#-field-order union (forest-first, exactly research/92 §1's
rationale extended) is:

```
… v17 downedFishron byte (1)
v18 UNION tail (53 bytes), offsets relative to the v18 block start:
  [0..4]    tree_x[0]            i32 LE  ┐
  [4..8]    tree_x[1]            i32 LE  │ g70 forest-set block (16 B)
  [8..12]   tree_x[2]            i32 LE  │ (research/92 §1, unchanged)
  [12]      forest_bg_styles[0]  u8      │
  [13]      forest_bg_styles[1]  u8      │
  [14]      forest_bg_styles[2]  u8      │
  [15]      forest_bg_styles[3]  u8      ┘
  [16..20]  cave_back_x[0]       i32 LE  ┐
  [20..24]  cave_back_x[1]       i32 LE  │
  [24..28]  cave_back_x[2]       i32 LE  │ g72 cave block (31 B, THIS note §1)
  [28..32]  cave_back_style[0]   i32 LE  │
  [32..36]  cave_back_style[1]   i32 LE  │
  [36..40]  cave_back_style[2]   i32 LE  │
  [40..44]  cave_back_style[3]   i32 LE  │
  [44]      ice_back_style       u8      │
  [45]      jungle_back_style    u8      │
  [46]      hell_back_style      u8      ┘
  [47]      snow_bg_style        u8      ┐ g71 surface bytes (3 B)
  [48]      desert_bg_style      u8      │
  [49]      ocean_bg_style       u8      ┘
  [50]      crimson              u8      — g69 (WorldFile.cs:1281)
  [51]      combat_book_volume_two_was_used u8 ┐ g90 books (2 B,
  [52]      peddlers_satchel_was_used       u8 ┘  WorldFile.cs:1407-1408)
```

**Total 53 bytes: 16 (g70) + 31 (g72) + 3 (g71) + 1 (g69) + 2 (g90).**

**D5-RE-PIN note (historical draft):** this five-claimant layout is the
closeout-era DRAFT. In the FINAL 107-byte union (g88 103 §L3-DEFINITIVE D3
via 64b §2) the rows [0..50) stand VERBATIM but the books tail does NOT
close the prefix: books move to [61..63] (the LAST flags rows,
`WorldFile.cs:1407-1408`), with moon [51..56], underworld [56], pets
[57..60], empress [60] between crimson [50] and them. The draft's
books-at-[51..53] seat was carried into 92 §1-FULL.2 and retired by the
D5 correction (64b §5).

Ordering rationale: g70's forest block stays first (research/92 §1's fixed
contract — do NOT re-shuffle it); our cave block follows because the C#
writes caveBackX/Style immediately after treeX/treeStyle (`:1260` follows
`:1259`); g71's surface bytes follow because setBG3/6/7 (`:1321/:1324-1325`)
come after the cave fields; g69's crimson (`:1281`) technically precedes the
setBG bytes in C# order but is a single bool riding the flags half — placing
it after g71's group keeps every multi-byte claim contiguous and matches
"forest-first, then C# order within the bg family, then the flag singles".
g90's books are the last C# fields of the whole block (`:1407-1408`), so
they close the tail. If g71c prefers strict C# order (crimson at [16] before
the cave block), that is equally valid — the contract only demands ONE
fixed shared order; this note publishes the recommended one.

## 3. Merge instructions (extends research/92 §1)

- Writer: after the v17 fishron byte, write g70's 16, then OUR 31 verbatim
  (`save.rs:444-450` as written on this branch), then g71's 3, g69's 1,
  g90's 2. Reader: one `if version >= 18` arm reading all 53 in that order.
- Truncation constants: research/92 §1 moved g70's 16 / g71's 3 to 19. With
  all five claimants the constant becomes **53**. On OUR branch the eleven
  sites currently carry `+ 31` (`save.rs:1111/:1207/:1285/:1349/:1378/
  :1567/:1588/:1610/:1726/:1846/:1961/:2088/:2189` — the EMPTY_V16_TAIL
  family, the v12/v14 tail arithmetic and the six mid-record cuts re-armed
  in `2b02345c`); each moves 31 → 53 at the merge. g69/g90's own sites move
  1/2 → 53 likewise.
- Back-compat: unchanged from research/92 §1 — no released v18 exists, the
  union is forward-only from the merge; pre-v18 files keep each claimant's
  own defaults (ours: the `World::new` zeros, §1).
- `underworldBG` (`:1376`) — **SEAT FIRED (retire receipt, research/64b §0):**
  g71 DOES roll `RandomizeBackgrounds`' underworld `Next(3)`
  (`WorldGen.cs:8011`, `bg_rolls.rs:168`/`:311`) and `.wld`-persists it
  (`flags.rs:186`/`:577`) at tip `f2d8c2c67`. In the FINAL 107-byte union
  (g88 103 §L3-DEFINITIVE D3, D5-RE-PIN) the byte sits at **[56]** (after
  the moon-boss block [51..56], ahead of the pets [57..60] — strict C#
  row order `:1347-1351` < `:1376` < `:1392-1394`; the retire-era 92
  §1-FULL.2 seat [58] and this note's original "between surface and
  crimson" suggestion are both superseded). This flag is CLOSED.

## 4. Witnesses

Code-only: no new executable witness in this note (it is a merge
instruction). The existing `w4_g72_the_cave_background_roll_round_trips_in_v18`
(`save.rs:1264`) pins our 31-byte block end-to-end and stays green through
the merge with its truncation constant moved to 53 (then **107** in the
FINAL union — research/64b §3).