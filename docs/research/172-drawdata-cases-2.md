# 172 — DrawData case census, slice 2 (W4-d62; renumbered 62->172 per research/150 registry)

> Wave 4 · researched 2026-11-20 · branch `task/W4-d172-drawdata-cases-2` @ base a725acb0 ·
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
   79/90/18/14/21/467/15/172 and 638's lerp are produced but not drawn (same
   finding as d64 batch 5; 497 shares 15's arm but the `typeCache == 15` gate
   keeps it dark).

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

### Batch 3 — strides, the rainbow boulder and the chest seam: #96-110 (15 ids)

Ids: 715, 716, 79, 90, 18, 711, 654, 14, 21, 411, 467, 469, 15, 497, 172.

**715 / 716** — last two labels of the 18-px group (batch 1 quote,
`TileDrawing.cs:4905-4908`): Ghoulder / BoulderThatSpawnsPet. Rust
`712..=716` run. **PORTED**.

**79 Beds** (`TileDrawing.cs:4909-4916`):

```csharp
		case 79:
		{
			tileHeight = 18;
			int num23 = tileFrameY / 2016;
			addFrY -= 2016 * num23;
			addFrX += 144 * num23;
			break;
		}
```

Rust (`drawdata.rs:348-355`): `data.height = 18; let n23 = fy / 2016;
data.add_fr_y = (tile_frame * 38 - 2016 * n23) as i16; data.add_fr_x = (144 * n23) as i16;`
— the `-=` folds onto the `:4533` starting value `tileFrame * 38` exactly.
Second switch (`:6191-6207`): rows 27/28 of the 36 grid → GlowMask 53 (martian) /
114 (meteor), rect `(tileFrameX, tileFrameY % 36, w, h)` — ported verbatim in
`drawdata_cases_a.rs:387-396` (gate reads the RAW fy, not the unwound row).
**PORTED** (glow record unconsumed, `_glow` TILE-DRAW-2).

**90 Bathtubs** (`TileDrawing.cs:4917-4923`):

```csharp
		case 90:
		{
			int num22 = tileFrameY / 2016;
			addFrY -= 2016 * num22;
			addFrX += 144 * num22;
			break;
		}
```

Rust (`drawdata.rs:356-362`): same stride, NO height write (the C# has none —
bathtubs keep 16). Second switch (`:6174-6190`): rows 27/28 → masks 52/113,
`cases_a.rs:376-385`. **PORTED**.

**18 WorkBenches** (`TileDrawing.cs:4924-4930`):

```csharp
		case 18:
		{
			int num21 = tileFrameX / 2016;
			addFrX -= 2016 * num21;
			addFrY += 20 * num21;
			break;
		}
```

Rust (`drawdata.rs:363-369`): the stride lives in frame X here; `add_fr_y =
(tile_frame * 38 + 20 * n21)` — the `+=` folds onto the same start. Second
switch (`:6429-6445`): columns 27/28 of the 36 grid → masks 69/125, rect
`(tileFrameX % 36, tileFrameY, w, h)`, `cases_a.rs:555-564`. **PORTED**.

**711 RainbowBoulder** (`TileDrawing.cs:4931-4939`):

```csharp
		case 711:
			if (tileFrameX > 0)
			{
				tileWidth = 18;
			}
			tileHeight = 20;
			glowTexture = TextureAssets.Tile[711].Value;
			glowSourceRect = new Rectangle(tileFrameX + addFrX, tileFrameY + addFrY, tileWidth, tileHeight);
			break;
```

Rust static arm (`drawdata.rs:370-379`): the width/height half was faithful, but
the two glow lines were parked ("the glow out-params are TILE-DRAW-2") — the
FIRST-switch glow record was **the one genuinely unported draw-side piece in
this range** (no second-switch 711 case exists; nothing else produced it).
**FIXED in this batch**: `drawdata_cases_a::glow_switch` gains the 711 arm —
`GlowData { mask: TILE_SHEET_GLOW, src: rect_framed(height), color: [255, 255, 255, 0] }`.
`TILE_SHEET_GLOW` (u16::MAX) is the new sentinel for "the glow texture is
`TextureAssets.Tile[kind]`, not a GlowMask entry"; the colour is the `:4538`
default `Color.Transparent` = (255,255,255,0) — the case never writes glowColor.
The rect reads the POST-width locals (`tileWidth` 18 on later columns), which
the seam already carries (`apply_cases_a` passes `data.width/height` after the
first switch). The `DoNotAdjustDrawPositionBasedOnTileWidth` centring skip
(`TileID.cs` `CreateBoolSet(711)`, `TileDrawing.cs:1009-1012`) was already ported
(`drawdata.rs:1449-1451` `no_width_adjust`, consumed `tiles.rs:1001`).
Test `test_DW_d62_rainbow_boulder_first_switch_glow` pins fx-0/fx-18 rects and
the animator-row fold. **PORTED** (record now produced; binding stays TILE-DRAW-2).

**654 TNTBarrel** (`TileDrawing.cs:4940-4942`): `tileTop += 2; break;` —
Rust `data.top += 2` (`drawdata.rs:380-383`). **PORTED**.

**14 / 21 / 411 / 467 / 469** (`TileDrawing.cs:4943-4952`):

```csharp
		case 14:
		case 21:
		case 411:
		case 467:
		case 469:
			if (tileFrameY == 18)
			{
				tileHeight = 18;
			}
			break;
```

Rust (`drawdata.rs:384-391`): `14 | 21 | 411 | 467 | 469 => { if fy == 18 { data.height = 18; } }`.
**PORTED** — Tables / Containers / Detonator / Containers2 / Tables2.
Second switch: 14 (`:6397-6413`, columns 31/32 of the 54 grid, masks 67/124,
`cases_a.rs:532-541`) and 21/467 (`:6276-6293`, columns 48/49 of the 36 grid,
masks 56/117, `cases_a.rs:450-459`) — verbatim, raw-fx gates.

**Chest draw-side seam (21/467)** — `DrawSingleTile` runs
`CacheSpecialDraws_Part2` AFTER `GetTileDrawData` (`:993` → `:4289-4330`): the
sub-cell frame fold (`addFrX = num*18 - fx`, `addFrY = num4 - fy`), the open
animation +38/+76 rows, the bottom-row 18-px clamp and the style-48/49 glow
rect. This is ported TWICE — `drawdata_furniture::chest_draw_rewrite`
(`tr-core/src/drawdata_furniture.rs:222-250`, minus the glow half) and
`render/special_tiles.rs::chest_frame_adjust` (`:101-145`, with the glow rect) —
and **UNWIRED both**: `furniture_post_pass` has zero callers in tr-app;
`special_tiles.rs` is the S1-G-1e UNWIRED module whose doc defers the wiring to
the d57/d58 special-tile merge step (contested `resolve_tile_frame` seam,
`special_tiles.rs:7-16`). d62 does NOT wire it (scope: the seam belongs to
d57 `tile-draw-te-furniture` / d58 `tile-draw-chest-frames`); recorded as
**PORTED-UNWIRED**. Consequence today: chests draw their stored sub-cell frames
without the open-animation row shift — the closed-chest t=0 draw is unaffected
(the rewrite is the identity at frame 0 on the anchor grid).

**15 / 497** (`TileDrawing.cs:4953-4959`):

```csharp
		case 15:
		case 497:
			if (tileFrameY % 40 == 18)
			{
				tileHeight = 18;
			}
			break;
```

Rust (`drawdata.rs:392-398`): `15 | 497 => { if fy % 40 == 18 { data.height = 18; } }`.
Second switch (`:6244-6267`): the 2040 stride (51 rows of 40) unwinds for BOTH
kinds, but only `typeCache == 15` glows (rows 32/33, masks 54/116) —
`cases_a.rs:428-442` keeps the `kind == 15` gate verbatim. **PORTED** —
Chairs / Toilets.

**172 Sinks** (`TileDrawing.cs:4960-4966`, shared label with 376 — batch 4):
`if (tileFrameY % 38 == 18) tileHeight = 18;` — Rust `172 | 376` arm
(`drawdata.rs:399-405`). Second switch (`:6487-6506`): the 2014 stride (53 rows
of 38) + rows 28/29 glow (masks 88/122), `cases_a.rs:598-610`. **PORTED**.

Batch 3 verdict: **15/15 static arms PORTED**; 711's first-switch glow record
FIXED (produced, unconsumed); 21/467 chest rewrite PORTED-UNWIRED (d57/d58 seam).

### Batch 4 — sunflower, levers, herbs and the vine head: #111-125 (15 ids)

Ids: 376, 27, 132, 135, 82, 83, 84, 324, 494, 52, 62, 115, 205, 382, 528.

**376 FishingCrate** — second label of the 172 arm (batch 3 quote,
`TileDrawing.cs:4960-4966`): `if (tileFrameY % 38 == 18) tileHeight = 18;`.
Rust `172 | 376` (`drawdata.rs:399-405`). No second-switch case (the 172 glow
row is kind-gated by the switch label itself — 376 never enters it).
**PORTED**.

**27 Sunflower** (`TileDrawing.cs:4967-4972`):

```csharp
		case 27:
			if (tileFrameY % 74 == 54)
			{
				tileHeight = 18;
			}
			break;
```

Rust (`drawdata.rs:406-412`): `27 => { if fy % 74 == 54 { data.height = 18; } }`.
**PORTED**, and the draw side is WIRED beyond the table: the sunflower never
draws through `DrawSingleTile` in C# (MultiTileGrass special, `:600-606`); the
port draws each cell inline with the same per-cell transform
(`tiles.rs:915-977`, `sunflower_sway` over `_sunflowerWindCounter`, the face
glow 74 px below the cell, `:8924-8928`/`:9004-9008`) — the documented z-order
approximation. The `% 74 == 54` row is the flower's face row.

**132 / 135** (`TileDrawing.cs:4973-4977`):

```csharp
		case 132:
		case 135:
			tileTop = 2;
			tileHeight = 18;
			break;
```

Rust (`drawdata.rs:413-418`): `132 | 135 => { data.top = 2; data.height = 18; }`.
**PORTED** — Lever / PressurePlates. (The `_d62_range.py` "+cases_a" tag on 132
is the moss-glow nested match `132 => Some(KRYPTON_MOSS_GLOW)` inside case 184's
arm, `cases_a.rs:350` — a false positive, not a lever arm.)

**82 / 83 / 84** (`TileDrawing.cs:4978-4987`):

```csharp
		case 82:
		case 83:
		case 84:
			tileHeight = 20;
			tileTop = -2;
			if (x % 2 == 0)
			{
				tileSpriteEffect = SpriteEffects.FlipHorizontally;
			}
			break;
```

Rust (`drawdata.rs:419-425`): `82 | 83 | 84 => { data.height = 20; data.top = -2; data.flip = flip_x; }`.
**PORTED** — ImmatureHerbs / MatureHerbs / BloomingHerbs. Draw-side notes:
83/84 are `SwaysInWindBasic` members (`wind.rs:1062` set; the grass-sway path
`tiles.rs:882-896` + `:978-994` draws them with the wind transform); 82 is not
(the immature herb does not sway in vanilla's set either). **83's
`DrawSingleTile` texture override** (`TileDrawing.cs:1149-1151`):

```csharp
		case 83:
			tileDrawInfo.drawTexture = this.GetTileDrawTexture(tileDrawInfo.tileCache, tileX, tileY);
			break;
```

`GetTileDrawTexture` resolves the per-tile PAINT-coat sheet; the port parks it
(`drawdata.rs:1344-1345` comment: "the texture override (83) stay parked … the
per-tile sheet override"). The paint tint itself rides `push_lit`'s
`tile.paint()` argument, so unpainted mature herbs draw identically; painted
ones miss the sheet swap. **DEVIATION** (documented, paint-sheet resolve scope).

**324 BeachPiles** (`TileDrawing.cs:4988-4996`):

```csharp
		case 324:
			tileWidth = 20;
			tileHeight = 20;
			tileTop = -2;
			if (x % 2 == 0)
			{
				tileSpriteEffect = SpriteEffects.FlipHorizontally;
			}
			break;
```

Rust (`drawdata.rs:426-433`): all four writes verbatim. **PORTED**.

**494 GolfTee** (`TileDrawing.cs:4997-4999`): `tileTop = 2; break;` — Rust
`data.top = 2` (`drawdata.rs:434-437`). **PORTED**.

**52 / 62 / 115 / 205 / 382 / 528** — first six labels of the vine group
(`TileDrawing.cs:5000-5013`; 636/638 are batch 5):

```csharp
		case 52:
		case 62:
		case 115:
		case 205:
		case 382:
		case 528:
		case 636:
		case 638:
			tileTop = -2;
			if (x % 2 == 0)
			{
				tileSpriteEffect = SpriteEffects.FlipHorizontally;
			}
			break;
```

Rust (`drawdata.rs:438-443`): the full 8-id pattern, `data.top = -2; data.flip = flip_x;`.
**PORTED** — Vines / JungleVines / HallowedVines / CrimsonVines / VineFlowers /
MushroomVines. The vine THREAD growth animation (`Main.tileFrame` rows) rides
the wired animator's global `addFrY = tileFrame * 38` default; the dedicated
vine-tip pass is S1-G-1a (out of this slice's scope).

Batch 4 verdict: **14/15 PORTED**, 83's painted-sheet override DEVIATION
(documented, pre-existing).

### Batch 5 — vine tail and cacti/pumps: #126-130 (5 ids)

Ids: 636, 638, 80, 142, 143.

**636 CorruptVines** — seventh label of the vine group (batch 4 quote,
`TileDrawing.cs:5000-5013`). Rust `52 | 62 | 115 | 205 | 382 | 528 | 636 | 638`.
No second-switch case. **PORTED**.

**638 AshVines** — last label of the vine group, PLUS a second-switch glow
(`TileDrawing.cs:6080-6084`):

```csharp
		case 638:
			glowTexture = TextureAssets.GlowMask[327].Value;
			glowSourceRect = new Rectangle(tileFrameX + addFrX, tileFrameY + addFrY, tileWidth, tileHeight - halfBrickHeight);
			glowColor = Color.Lerp(Color.White, color, 0.75f);
			break;
```

Rust (`cases_a.rs:327`): `638 => out.glow = Some(GlowData { mask: 327, src:
rect_framed(height - half_brick_height), color: lerp_white_75(color) })` — the
framed rect, the half-brick height cut and the per-channel byte-truncating
`Color.Lerp` (`cases_a.rs:300-303`, `(a + (b - a) * 0.75) as u8`) verbatim; the
cell light rides in as `color` (`tiles.rs:724-732`, XNA bytes alpha 255).
**PORTED** (record unconsumed, `_glow` TILE-DRAW-2).

**80 / 142 / 143** (`TileDrawing.cs:5014-5018`):

```csharp
		case 80:
		case 142:
		case 143:
			tileTop = 2;
			break;
```

Rust (`drawdata.rs:444-451`): `80 | 142 | 143 => { data.top = 2; }`.
**PORTED** — Cactus / InletPump / OutletPump. The cactus ground shift is
`DrawSingleTile`'s `case 80:` ONLY (`TileDrawing.cs:1132-1147`,
`WorldGen.GetCactusType` → `rectangle.Y += 54/108/162`): ported as
`cactus_type` + `cactus_source_y_offset` (`drawdata.rs:1275-1277`) applied by
`apply_world_corrections` (`:1328-1331`) into `src_y_shift`, which the flat
draw consumes (`tiles.rs:1440-1449`) while the half-brick/slope arms rebuild
their rects without it — exactly the C# `rectangle`-local semantics
(`:1007` vs `:1298`). 142/143 (pumps) correctly get NO ground shift.
**PORTED + WIRED**.

Batch 5 verdict: **5/5 PORTED**.

## Slice summary

| Verdict | Count | Ids |
| --- | --- | --- |
| PORTED (static arm faithful, wired) | 65 | all |
| glow record produced, unconsumed (`_glow`, TILE-DRAW-2) | 10 | 79, 90, 18, 711, 14, 21, 467, 15, 172, 638 (497 shares 15's arm but never glows — the `typeCache == 15` gate) |
| PORTED-UNWIRED (chest rewrite, d57/d58 seam) | 2 | 21, 467 |
| DEVIATION (pre-existing, documented) | 1 | 83 (painted-sheet override) |

Code changes in this slice (the only draw-side gap that was d62's to fix):
* `drawdata_cases_a.rs`: `TILE_SHEET_GLOW` sentinel + the 711 first-switch glow
  arm + `test_DW_d62_rainbow_boulder_first_switch_glow` (written, NOT run —
  code-only mode).
* `drawdata.rs`: the 711 arm comment repointed at the new record (no behaviour
  change).

Not touched (other slices' seams): the chest rewrite wiring (d57/d58), the glow
pass binding (TILE-DRAW-2), 83's paint-sheet resolve, the vine-tip pass
(S1-G-1a). No RNG stream, frame-cache key or fixed-step order changes — the
711 record is produced at the existing `apply_cases_a` seam and rides unused
until the glow pass lands, so the current pixel output is unchanged.
