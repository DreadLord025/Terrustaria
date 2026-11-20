# 62 — DrawData case census, slice 2 (W4-d62)

> Wave 4 · researched 2026-11-20 · branch `task/W4-d62-drawdata-cases-2` @ base a725acb0 ·
> code-only mode (no builds/tests run) · sources: `decompiled/Terraria/GameContent/Drawing/TileDrawing.cs`,
> `crates/tr-core/src/drawdata.rs`, `crates/tr-core/src/drawdata_furniture.rs`,
> `crates/tr-app/src/drawdata_cases_a.rs`, `crates/tr-app/src/drawdata_cases_b.rs`,
> `crates/tr-app/src/render/tiles.rs`, `crates/tr-app/src/render/special_tiles.rs`.

## Range claim (documented early, per wave protocol)

The four W4 drawdata slices partition the **C#-order union of `GetTileDrawData` case ids**
(first switch `TileDrawing.cs:4540-6032`, then the second/glow switch `:6049-6510` ids not
already in the first; 361 + 10 = **371 distinct ids**). d64's pushed doc
(`64-drawdata-cases-4.md`) claims the LAST quarter `[279, 371)` (+ the 215 campfire
epilogue) and states the quarter convention; d61's in-flight census script
(`TR-W4-d61/tools/_d61_cases.py`) claims `union[0, 65)`. This slice takes the SECOND
~65 in C# order:

| Slice | Index range (0-based) | Count |
| --- | --- | --- |
| d61 | [0, 65) | 65 |
| **d62 (this doc)** | **[65, 130)** | **65** |
| d63 | [130, 279) | 149 (d63's own split) |
| d64 | [279, 371) + 215 epilogue | 92 + 1 |

**d62 owns these 65 ids** (C# order; `@` = first-switch label line; `#` = 1-based index):

```
#66  703@4857  #67  20@4864  #68  590@4865  #69  595@4866  #70  615@4873
#71  16@4880   #72  17@4881  #73  26@4882   #74  32@4883   #75  69@4884
#76  72@4885   #77  77@4886  #78  124@4887  #79  137@4888  #80  138@4889
#81  352@4890  #82  462@4891 #83  487@4892  #84  488@4893  #85  574@4894
#86  575@4895  #87  576@4896 #88  577@4897  #89  578@4898  #90  664@4899
#91  695@4900  #92  704@4901 #93  712@4902  #94  713@4903  #95  714@4904
#96  715@4905  #97  716@4906 #98  79@4909   #99  90@4917   #100 18@4924
#101 711@4931  #102 654@4940 #103 14@4943   #104 21@4944   #105 411@4945
#106 467@4946  #107 469@4947 #108 15@4953   #109 497@4954  #110 172@4960
#111 376@4961  #112 27@4967  #113 132@4973  #114 135@4974  #115 82@4978
#116 83@4979   #117 84@4980  #118 324@4988  #119 494@4997  #120 52@5000
#121 62@5001   #122 115@5002 #123 205@5003  #124 382@5004  #125 528@5005
#126 636@5006  #127 638@5007 #128 80@5014   #129 142@5015  #130 143@5016
```
```

Both-switch ids in this range (first-switch body audited here, second-switch glow row
cited alongside): 79 (`:6191`), 90 (`:6174`), 18 (`:6429`), 14 (`:6397`), 21/467
(`:6276-6277`), 15/497 (`:6244-6245`), 172 (`:6487`), 638 (`:6080`). The cacti/pumps
group (80/142/143) is d62's tail; 139 (MusicBoxes, `@5019`) is index #131 — d63's.
Reproduce with `python tools/_d62_range.py` (order + classification) and
`python tools/_d62_extract.py` (verbatim C# bodies + Rust arm text per id,
`tools/_d62_extract.txt`).

## Headline census result

All 65 ids have a faithful static arm in `tr_core::drawdata::draw_data`
(`tools/_d62_range.py` classifies every one FULL — no deviation marker). The draw-side
gaps in this range are NOT missing static cases:

1. **711 RainbowBoulder's FIRST-switch glow record was unported** —
   `TileDrawing.cs:4937-4938` sets `glowTexture = TextureAssets.Tile[711]` +
   `glowSourceRect`; `drawdata.rs`'s 711 arm parked it as "TILE-DRAW-2". FIXED in this
   slice: `drawdata_cases_a::glow_switch` now produces the record verbatim (batch 3).
2. **21/467 chest frame rewrite is ported twice but UNWIRED** —
   `drawdata_furniture::chest_draw_rewrite` (`:222`) and
   `render/special_tiles.rs::chest_frame_adjust` (`:101`, S1-G-1e UNWIRED module) both
   port `CacheSpecialDraws_Part2` (`TileDrawing.cs:4289-4330`); neither has a live
   caller. The wiring belongs to the d57/d58 special-tile merge step (contested seam,
   `special_tiles.rs:7-16`) — documented here, NOT wired (batch 3).
3. **83's `GetTileDrawTexture` painted-sheet override** (`TileDrawing.cs:1149-1151`)
   stays parked (`drawdata.rs:1344-1345` comment) — the per-tile paint sheet override
   needs the paint-coat texture resolve (batch 4).
4. **All glow records ride unused** — `render/tiles.rs:731` binds `apply_cases_a`'s
   return to `_glow` (TILE-DRAW-2 glow pass); the martian/meteor conduit rows of
   79/90/18/14/21/467/15/497/172 and 638's lerp are produced but not drawn (same
   finding as d64 batch 5).

`cases_b` owns NONE of the 65 (its dispatch arms are 719/579/453/456/507/508/541/485/
489/490/493/428/455/349/593/594 — the `_d62_range.py` "+cases_b" tags on 20/16/90/18/
14/15/27 are nested-literal false positives of the crude module scan). The furniture
module touches only 21/467 (the `BasicChest` set). The "+cases_a" tag on 132 is the
moss-glow nested match inside case 184 (`drawdata_cases_a.rs:350`), not a lever arm.

## Per-case audit

Verdict key: **PORTED** (arm faithful, wired), **PORTED-UNWIRED** (arm faithful, no
live caller), **DEVIATION** (documented approximation). C# quoted verbatim (tab
indentation normalised); shared case-label groups cite the body once.

### Batch 1 — plants / saplings / the 18-px furniture head: #66-80 (15 ids)

Ids: 703, 20, 590, 595, 615, 16, 17, 26, 32, 69, 72, 77, 124, 137, 138.

**703** — last label of the `case 3` plant group (`TileDrawing.cs:4850-4863`):

```csharp
		case 3:
		case 24:
		case 61:
		case 71:
		case 110:
		case 201:
		case 637:
		case 703:
			tileHeight = 20;
			if (x % 2 == 0)
			{
				tileSpriteEffect = SpriteEffects.FlipHorizontally;
			}
			break;
```

Rust (`drawdata.rs:311-316`): `3 | 24 | 61 | 71 | 110 | 201 | 637 | 703 => { data.height = 20; data.flip = flip_x; }`
with `flip_x = x % 2 == 0` (`:128`). **PORTED** — 703 (JunglePlantsEcho) rides
the shared arm; the group's other labels are d61/d63 ids. 637's second-switch painted-
sheet glow (`:6075-6079`) is cases_a's (mask-0 deviation, d63's index).

**20 / 590 / 595** (`TileDrawing.cs:4864-4872`) and **615** (`:4873-4879`, a separate
label with the identical body):

```csharp
		case 20:
		case 590:
		case 595:
			tileHeight = 18;
			if (x % 2 == 0)
			{
				tileSpriteEffect = SpriteEffects.FlipHorizontally;
			}
			break;
		case 615:
			tileHeight = 18;
			if (x % 2 == 0)
			{
				tileSpriteEffect = SpriteEffects.FlipHorizontally;
			}
			break;
```

Rust (`drawdata.rs:317-322`): `20 | 590 | 595 | 615 => { data.height = 18; data.flip = flip_x; }`.
**PORTED** — the two C# label groups fold into one arm; same metrics, same parity flip.
Saplings 20 / GemSaplings 590 / VanityTreeSakuraSaplings 595 / VanityTreeWillowSaplings 615.

**16 / 17 / 26 / 32 / 69 / 72 / 77 / 124 / 137 / 138** — first ten labels of the 27-label
18-px group (`TileDrawing.cs:4880-4908`; the remaining 17 labels are batch 2):

```csharp
		case 16:
		case 17:
		case 26:
		case 32:
		case 69:
		case 72:
		case 77:
		case 124:
		case 137:
		case 138:
		case 352:
		case 462:
		case 487:
		case 488:
		case 574:
		case 575:
		case 576:
		case 577:
		case 578:
		case 664:
		case 695:
		case 704:
		case 712:
		case 713:
		case 714:
		case 715:
		case 716:
			tileHeight = 18;
			break;
```

Rust (`drawdata.rs:323-347`): the full 27-id pattern
`16 | 17 | 26 | 32 | 69 | 72 | 77 | 124 | 137 | 138 | 352 | 462 | 487 | 488 | 574..=578 | 664 | 695 | 704 | 712..=716 => { data.height = 18; }`.
**PORTED** — `574..=578` and `712..=716` are the exact label runs; no other field moves
(the C# body is the single assignment). Anvils 16 / Furnaces 17 / DemonAltar 26 /
CrimsonAltar 32 / thorns 69 / MushroomTrees 72 / Hellforge 77 / WoodenBeam 124 /
Traps 137 / Boulder 138. None of the ten has a second-switch case or a
`DrawSingleTile` nudge — the static arm is the whole draw-data story.

Batch 1 verdict: **15/15 PORTED**, no draw-side gap.

### Batch 2 — the 18-px furniture tail: #81-95 (15 ids)

Ids: 352, 462, 487, 488, 574, 575, 576, 577, 578, 664, 695, 704, 712, 713, 714.

All fifteen are labels of the SAME 27-label group quoted verbatim in batch 1
(`TileDrawing.cs:4880-4908`); the body is the single `tileHeight = 18; break;`. The
labels in this batch, verbatim from the group:

```csharp
		case 352:
		case 462:
		case 487:
		case 488:
		case 574:
		case 575:
		case 576:
		case 577:
		case 578:
		case 664:
		case 695:
		case 704:
		case 712:
		case 713:
		case 714:
```

Rust (`drawdata.rs:337-345`): `352 | 462 | 487 | 488 | 574..=578 | 664 | 695 | 704 | 712..=716`
inside the batch-1 arm — `data.height = 18` only. **PORTED** for all fifteen
(TileID.cs names): CrimsonThorns 352 / DjinnLamp 462 / PicnicTable 487 /
FallenLog 488 / BorealBeam 574 / RichMahoganyBeam 575 / GraniteColumn 576 /
SandstoneColumn 577 / MushroomBeam 578 / BouncyBoulder 664 /
DemonAltarReplica 695 / FallenLogEcho 704 / Poulder 712 / LavaBoulder 713 /
SpiderBoulder 714 (Ghoulder 715 / BoulderThatSpawnsPet 716 are batch 3).
None has a second-switch case,
a `DrawSingleTile` nudge, or an animator row beyond the global
`addFrY = Main.tileFrame[type] * 38` default (which the port carries for every
kind, `drawdata.rs:121-125`). The boulder kinds' 18-px height is what lets the
20-px-tall boulder sprite overhang the cell by 2 px like vanilla.

Batch 2 verdict: **15/15 PORTED**, no draw-side gap.
