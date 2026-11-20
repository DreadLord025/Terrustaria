# 61 — DrawData case census, slice 1 (W4-d61)

> Wave 4 · researched 2026-11-20 · branch `task/W4-d61-drawdata-cases-1` @ base a725acb0 ·
> code-only mode (no builds/tests run) · sources: `decompiled/Terraria/GameContent/Drawing/TileDrawing.cs`,
> `crates/tr-core/src/drawdata.rs`, `crates/tr-core/src/drawdata_anim.rs`,
> `crates/tr-app/src/drawdata_cases_a.rs`, `crates/tr-app/src/drawdata_cases_b.rs`,
> `crates/tr-app/src/render/tiles.rs`.

## Range claim (documented early, per wave protocol)

The four W4 drawdata slices partition the **C#-order union of `GetTileDrawData` case ids**
(first switch `TileDrawing.cs:4540-6032`, then the second/glow switch `:6049-6507` ids not
already in the first; 361 + 10 = **371 distinct ids**). d64
(`origin/task/W4-d64-drawdata-cases-4`, `docs/research/64-drawdata-cases-4.md`) stated the
convention and claimed the LAST quarter; d63
(`origin/task/W4-d63-drawdata-cases-3` @ `a2ad77ba`, 93/93 audited, 0 MISSING) took the
third. This doc takes the FIRST quarter:

| Slice | Index range | Ids |
| --- | --- | --- |
| **d61 (this doc)** | **[0, 93)** | **93** |
| d62 | [93, 186) | 93 |
| d63 | [186, 279) | 93 (done, 0 MISSING) |
| d64 | [279, 371) | 92 + the 215 epilogue |

Boundary cross-check (brace-balanced re-extraction, `tools/_d61_census.py` — the same
depth-1-label extraction as d63's `tools/_d63_census.py`, nested sub-switch bodies
excluded): index 92 = `712@4902` (last d61), index 93 = `713@4903` (first d62),
index 185 = `354@5310` (last d62), index 186 = `355@5311` (first d63 — matches d63's
claimed list verbatim), index 278 = `286@5757` (last d63), index 279 = `298@5758`
(first d64 — matches d64's claimed list verbatim). Union = 371, identical to
d63/d64. The 215 campfire epilogue (`:6033-6044`, not a case label) belongs to d64.

**d61 owns these 93 ids** (C# order; `@` = first-switch label line; all 93 are
first-switch ids — none of the 10 second-switch-only ids fall in this quarter):

```
752@4542 751@4547 739@4552 748@4553 726@4556 719@4562 443@4571
571@4581 698@4588 136@4593                                        (batch 2)
561@4599 518@4604 330@4631 331@4632 332@4633 333@4634 129@4637
5@4645 583@4650 584@4651                                          (batch 3)
585@4652 586@4653 587@4654 588@4655 589@4656 596@4657 616@4658
634@4659 476@4663 323@4667                                        (batch 4)
4@4672 78@4680 85@4681 133@4682 134@4683 173@4684 210@4685
233@4686 254@4687 283@4688                                        (batch 5)
378@4689 457@4690 466@4691 520@4692 651@4693 652@4694 100@4697
530@4705 705@4724 485@4727                                        (batch 6)
489@4731 490@4742 33@4819 49@4820 174@4821 372@4822 646@4823
529@4827 3@4850 24@4851                                           (batch 7)
61@4852 71@4853 110@4854 201@4855 637@4856 703@4857 20@4864
590@4865 595@4866 615@4873                                        (batch 8)
16@4880 17@4881 26@4882 32@4883 69@4884 72@4885 77@4886
124@4887 137@4888 138@4889                                        (batch 9)
352@4890 462@4891 487@4892 488@4893 574@4894 575@4895 576@4896
577@4897 578@4898 664@4899                                        (batch 10)
695@4900 704@4901 712@4902                                        (batch 11)
```

Overlap notes:
* Ids **634, 637, 100, 33** appear in BOTH switches; their first-switch index falls in
  d61's quarter, so their second-switch glow rows (`634@6070`, `637@6075`, `100@6228`,
  `33@6236`) are audited here alongside the first-switch body (d63 convention).
* The `16..712..716` height-18 group (`:4880-4908`) straddles the d61/d62 boundary:
  labels 713-716 (`:4903-4906`) are d62's; the shared group body is cited once here,
  d62 audits its own labels against the same arm.
* The `330..333` candy-cane group and the `583..589 | 596 | 616 | 634` tree-variant
  group are fully inside this quarter.

## Headline census result — 0 MISSING arms in this quarter

An independent brace-balanced port-side census (`tools/_census_drawdata3.py`, in-flight
scratch from this task's earlier pass, kept as evidence) matches every one of the 371
C# ids to an arm in `drawdata.rs` / `drawdata_cases_a.rs` / `drawdata_cases_b.rs` /
`drawdata_furniture.rs`: **covered 371/371, MISSING 0** — confirming d63/d64's count.
18 ids live outside `draw_data` alone; of those, d61 owns **719** (delegated to
`cases_b`) and **518** (delegated to `cases_a`); the other 16 are d62/d63/d64 ids.
The per-case audit below re-derives each verdict from the C# verbatim (≤80-line
windows) rather than trusting the port's deviation comments. research/09's
"≈113 of ≈376 case ids" (09-tile-drawing.md:31/111/188) is STALE, predating S1-F-9
waves A-E.

## Per-case audit

Filled in per batch below (10 ids per commit). Verdict key: **PORTED** (arm faithful,
wired), **PORTED-UNWIRED** (arm faithful, no live caller / unconsumed output),
**DEVIATION** (documented approximation — static part only), **MISSING** (no arm
anywhere). Wiring seams (all live in `render/tiles.rs`): the static table
`tr_core::drawdata::draw_data`, the world corrections
`TileDrawData::apply_world_corrections` (`tiles.rs:683-691`), the engine-driven pass
`drawdata_cases_b::case_draw_data` (`tiles.rs:700-712`, `CaseEnv` built at
`tiles.rs:349-360` with the WIRED animator per S1-G-1f), and slice A
`drawdata_cases_a::apply_cases_a` (`tiles.rs:731-733`) whose glow record rides
`_glow` — unconsumed until the glow pass binds it (TILE-DRAW-2).

### Batch 2 — palworld pets / jellyfish-water blocks / dummy / rainbow cloud / geyser / bamboo / display jar / switch: 752, 751, 739, 748, 726, 719, 443, 571, 698, 136 (10 ids)

C# `TileDrawing.cs:4542-4598` (57-line window), verbatim head:

```csharp
		case 752:
			tileHeight = 38;
			tileWidth = 36;
			tileTop = 2;
			break;
		case 751:
			tileHeight = 46;
			tileWidth = 56;
			tileFrameY += (short)((x + y * 2) % 7 * 46);
			break;
		case 739:
		case 748:
			addFrY = Main.tileFrame[typeCache] * 90;
			break;
		case 726:
			tileFrameX = 0;
			tileFrameY = 0;
			tileWidth = 20;
			tileHeight = 20;
			break;
		case 719:
		{
			int num24 = (x + y + (int)(Main.GlobalTimeWrappedHourly * 15f)) % 14;
			int num25 = num24 / 4;
			int num26 = num24 % 4;
			addFrX += 288 * num25;
			addFrY += 270 * num26;
			break;
		}
```

| Id | C# body (verbatim key lines) | C# line | Rust arm | Verdict |
| --- | --- | --- | --- | --- |
| 752 | `tileHeight = 38; tileWidth = 36; tileTop = 2;` | :4542-4546 | `drawdata.rs:133-137` | PORTED |
| 751 | `tileHeight = 46; tileWidth = 56; tileFrameY += (short)((x + y * 2) % 7 * 46);` | :4547-4551 | `drawdata.rs:140-144` (`frame_y_override`) | PORTED |
| 739 | `addFrY = Main.tileFrame[typeCache] * 90;` (absolute assign over the *38 default) | :4552-4555 | `drawdata.rs:147-149` (`739 \| 748`) | PORTED |
| 748 | shares 739's arm | :4553 | :147-149 | PORTED |
| 726 | frame reset to (0,0), 20x20 | :4556-4561 | `drawdata.rs:152-157` (both overrides Some(0)) | PORTED |
| 719 | `(x + y + (int)(GlobalTimeWrappedHourly * 15f)) % 14` → `/4`, `%4` grid; `addFrX += 288*n25; addFrY += 270*n26;` | :4562-4570 | static stub comment `drawdata.rs:158-160` + full body `cases_b.rs:125-131` (clock from `CaseEnv`, `tiles.rs:358` = `anim_time % 3600.0` ≈ `Main.cs:16836`) | PORTED |
| 443 | `if (tileFrameX / 36 >= 2) tileTop = -2; else tileTop = 2;` | :4571-4580 | `drawdata.rs:163-169` | PORTED |
| 571 | flip on `x % 2 == 0`, `tileTop = 2` | :4581-4587 | `drawdata.rs:172-175` (`flip_x` = same parity) | PORTED |
| 698 | `tileWidth = 36; tileFrameX = (short)(tileFrameX / 18 * 38); tileHeight = 44;` | :4588-4592 | `drawdata.rs:178-182` | PORTED |
| 136 | `if (tileFrameX == 0) tileTop = 2;` | :4593-4598 | `drawdata.rs:185-189` | PORTED |

DrawSingleTile companions audited with the ids (`TileDrawing.cs:1020-1073`, ported in
`apply_world_corrections`, `drawdata.rs:1357-1402`): 751/752 draw only at the (0,0)
frame (`:1022-1036` → `skip_draw` + `draw_dx/dy` nudges +11/-8 and +8), 136's column
nudge ±2 (`:1037-1047` → `drawdata.rs:1377-1381`), 726's -2 plus the
half-brick/slope ±6/+2 block (`:1054-1073` → `drawdata.rs:1387-1402`). One C# quirk
preserved: 751's skip gate reads `tileDrawInfo.tileCache.frameY` (the RAW stored
frame, `:1023`) while 752 reads the rewritten `tileDrawInfo.tileFrameY` (`:1031`) —
the port gates both on the raw `frame_x`/`frame_y` arguments (`drawdata.rs:1362`,
`:1370`), which matches 751 exactly and 752 whenever the raw frame is (0,0); for 752
the rewritten frame is only non-zero if the raw one is (the arm never rewrites it),
so the two gates coincide. Wired: `tiles.rs:683-691` calls the corrections for every
drawn cell.

### Batch 3 — marble column / lily pad / candy canes / crystals / trees / gem-tree variants: 561, 518, 330, 331, 332, 333, 129, 5, 583, 584 (10 ids)

C# `TileDrawing.cs:4599-4662` (64-line window), verbatim key bodies:

```csharp
		case 561:
			tileTop -= 2;
			tileHeight = 20;
			addFrY = tileFrameY / 18 * 4;
			break;
		case 518:
		{
			int num19 = tileCache.liquid / 16 - 3;
			if (WorldGen.SolidTile(x, y - 1) && num19 > 8)
			{
				num19 = 8;
			}
			if (tileCache.liquid == 0)
			{
				Tile tileSafely = Framing.GetTileSafely(x, y + 1);
				if (tileSafely.nactive())
				{
					switch (tileSafely.blockType())
					{
					case 1:
						num19 = -16 + Math.Max(8, tileSafely.liquid / 16);
						break;
					case 2:
					case 3:
						num19 -= 4;
						break;
					}
				}
			}
			tileTop -= num19;
			break;
		}
		case 330:
		case 331:
		case 332:
		case 333:
			tileTop += 2;
			break;
		case 129:
			addFrY = 0;
			if (tileFrameX >= 324)
			{
				int num40 = (tileFrameX - 324) / 18;
				addFrX = ((num40 + Main.tileFrame[typeCache]) % 6 - num40) * 18;
			}
			break;
		case 5:
			tileWidth = 20;
			tileHeight = 20;
			tileFrameX += (short)(176 * (TileDrawing.GetTreeBiome(x, y, tileFrameX, tileFrameY) + 1));
			break;
```

| Id | C# body (verbatim key lines) | C# line | Rust arm | Verdict |
| --- | --- | --- | --- | --- |
| 561 | `tileTop -= 2; tileHeight = 20; addFrY = tileFrameY / 18 * 4;` | :4599-4603 | `drawdata.rs:192-196` | PORTED |
| 518 | liquid float `num19` + solid-above clamp 8 + dry-pad half-brick/slope pick, `tileTop -= num19` | :4604-4630 | static stub comment `drawdata.rs:197-198` + full body `cases_a.rs:158-177` (`lily_pad_top_delta`, incl. `solid_tile` `:129-139` ≈ `WorldGen.cs:70111-70132` and `block_type` `:144-151` ≈ `Tile.cs:202-214`), folded at `cases_a.rs:639-643` | PORTED |
| 330 | `tileTop += 2` group | :4631-4636 | `drawdata.rs:200-202` (`330..=333`, `+=` preserved) | PORTED |
| 331 | shares the group | :4632 | :200-202 | PORTED |
| 332 | shares the group | :4633 | :200-202 | PORTED |
| 333 | shares the group | :4634 | :200-202 | PORTED |
| 129 | `addFrY = 0;` then `fx >= 324`: `((num40 + Main.tileFrame) % 6 - num40) * 18` | :4637-4644 | `drawdata.rs:206-212` (same mod-6 walk, absolute assign) | PORTED |
| 5 | 20x20 + `tileFrameX += 176 * (GetTreeBiome + 1)` | :4645-4649 | static `drawdata.rs:219-222` + biome shift `apply_world_corrections` `drawdata.rs:1304-1307` (`tree_frame_x_offset`); app also routes via `tree::trunk_sheet_offset_at` (`tiles.rs:742-756`) | PORTED |
| 583 | 20x20 tree-variant group (583..589/596/616/634) | :4650-4662 | `drawdata.rs:227-230` | PORTED |
| 584 | shares the group | :4651 | :227-230 | PORTED |

Notes: the `drawdata.rs:225-226` comment "Dead in the app for the same reason" is
STALE — `tree::is_tree` (`tree.rs:20-21`) covers `5 | 583..=589 | 596 | 616 | 634`
and `tiles.rs:641` routes every one of them through the tree path with per-variant
sheets (`tree.rs:321-327`); the metrics arm is live. 129's DrawSingleTile companion
(`:1088-1107`: alpha 100/255, transparent for `fx >= 324`, ±2 px nudge by frame row)
is ported at `drawdata.rs:1410-1418` — the `src_alpha` field feeds the flat draw.

### Batch 4 — tree variants tail / ash tree (+glow) / golf hole / palm: 585, 586, 587, 588, 589, 596, 616, 634, 476, 323 (10 ids)

C# `TileDrawing.cs:4652-4671` (20-line window), verbatim:

```csharp
		case 585:
		case 586:
		case 587:
		case 588:
		case 589:
		case 596:
		case 616:
		case 634:
			tileWidth = 20;
			tileHeight = 20;
			break;
		case 476:
			tileWidth = 20;
			tileHeight = 18;
			break;
		case 323:
			tileWidth = 20;
			tileHeight = 20;
			tileFrameY = (short)(22 * this.GetPalmTreeBiome(x, y));
			break;
```

634's second-switch glow row (`TileDrawing.cs:6070-6074`), verbatim:

```csharp
		case 634:
			glowTexture = TextureAssets.GlowMask[315].Value;
			glowSourceRect = new Rectangle(tileFrameX, tileFrameY + addFrY, tileWidth, tileHeight);
			glowColor = Color.White;
			break;
```

| Id | C# body (verbatim key lines) | C# line | Rust arm | Verdict |
| --- | --- | --- | --- | --- |
| 585 | 20x20 tree-variant group | :4652-4662 | `drawdata.rs:227-230` (`583..=589 \| 596 \| 616 \| 634`) | PORTED |
| 586 | shares the group | :4653 | :227-230 | PORTED |
| 587 | shares the group | :4654 | :227-230 | PORTED |
| 588 | shares the group | :4655 | :227-230 | PORTED |
| 589 | shares the group | :4656 | :227-230 | PORTED |
| 596 | shares the group; vanity sheet style 29 (`tree.rs:327`) | :4657 | :227-230 | PORTED |
| 616 | shares the group; vanity sheet style 30 | :4658 | :227-230 | PORTED |
| 634 | shares the group + glow row GlowMask 315, `Rectangle(tileFrameX, tileFrameY + addFrY, w, h)`, `Color.White` | :4659, :6070-6074 | metrics :227-230; glow `cases_a.rs:319-320` (`mask: 315, rect_anim_y(fx, height), [255,255,255,255]`) | PORTED; glow record PORTED-UNWIRED (`_glow`, TILE-DRAW-2) |
| 476 | `tileWidth = 20; tileHeight = 18;` | :4663-4666 | `drawdata.rs:232-235` | PORTED |
| 323 | 20x20 + `tileFrameY = 22 * GetPalmTreeBiome(x, y)` (replacement, not `+=`) | :4667-4671 | static `drawdata.rs:239-242` + `apply_world_corrections` `drawdata.rs:1310-1312` (`palm_frame_y(palm_tree_biome(...))`) | PORTED |

323's DrawSingleTile companion (`TileDrawing.cs:1152-1158`): the frond columns
`frameX 88..=132` `return` (canopy comes from the tree pass) and every other cell
shifts `vector.X += tileCache.frameY` (the RAW stored row). Ported at
`drawdata.rs:1432-1438` — the gate reads the raw `frame_x` argument and the nudge
adds raw `frame_y`, matching the C#'s `tileCache` reads exactly (the rewritten
`tileFrameY` is NOT used here). 634's glow rect uses raw `fx` + post-first-switch
`addFrY` (`rect_anim_y`), matching `:6072`; the first switch never rewrites 634's
frame, so raw == local. Same wiring-gap pattern as d63 batch 4's 564: the glow
record is computed faithfully but rides `_glow` (`tiles.rs:731`) until TILE-DRAW-2
binds the glow pass — a wiring gap, not a case gap.

### Batch 5 — torch + the top-2 furniture group head: 4, 78, 85, 133, 134, 173, 210, 233, 254, 283 (10 ids)

C# `TileDrawing.cs:4672-4696` (25-line window), verbatim:

```csharp
		case 4:
			tileWidth = 20;
			tileHeight = 20;
			if (WorldGen.SolidTile(x, y - 1))
			{
				tileTop = 4;
			}
			break;
		case 78:
		case 85:
		case 133:
		case 134:
		case 173:
		case 210:
		case 233:
		case 254:
		case 283:
		case 378:
		case 457:
		case 466:
		case 520:
		case 651:
		case 652:
			tileTop = 2;
			break;
```

`WorldGen.SolidTile` (`WorldGen.cs:70111-70132`), verbatim gate: null tile → true;
else `active() && tileSolid && !tileSolidTop && !halfBrick() && slope() == 0 &&
!inActive()` (the `noDoors` flag defaults false, unused here).

| Id | C# body (verbatim key lines) | C# line | Rust arm | Verdict |
| --- | --- | --- | --- | --- |
| 4 | 20x20, `tileTop = 4` on `WorldGen.SolidTile(x, y - 1)` | :4672-4679 | `drawdata.rs:245-251`; probe `tiles.rs:654-656` | PORTED (probe DEVIATION, below) |
| 78 | `tileTop = 2` group (absolute assign) | :4680-4696 | `drawdata.rs:256-258` (15-label alternation, exact set) | PORTED |
| 85 | shares the group | :4681 | :256-258 | PORTED |
| 133 | shares the group | :4682 | :256-258 | PORTED |
| 134 | shares the group | :4683 | :256-258 | PORTED |
| 173 | shares the group | :4684 | :256-258 | PORTED |
| 210 | shares the group | :4685 | :256-258 | PORTED |
| 233 | shares the group | :4686 | :256-258 | PORTED |
| 254 | shares the group | :4687 | :256-258 | PORTED |
| 283 | shares the group | :4688 | :256-258 | PORTED |

**Probe deviation (case 4, pre-existing, documented in the port):** the caller's
`solid_above` (`tiles.rs:654-656`) checks only `active && TILE_SOLID &&
!TILE_SOLID_TOP` — it omits the C#'s `!halfBrick() && slope() == 0 && !inActive()`
clauses and the null-cell → true edge (`WorldGen.cs:70115-70119`). A torch under a
half-brick, slope or actuated block therefore hangs (top 4) where vanilla stands it
(top 0). The full-fidelity probe already exists in the codebase —
`cases_a.rs:129-139` (`solid_tile`, ported for 518's clamp) — so this is a wiring
gap: point `tiles.rs:654` at the same predicate (needs the null-edge decision for
the top world row). The arm itself is verbatim; the group tail (378/457/466/520/
651/652) is audited in batch 6 against this same body.

### Batch 6 — top-2 group tail / candelabra (+glow) / oasis plants / echo / larva: 378, 457, 466, 520, 651, 652, 100, 530, 705, 485 (10 ids)

C# `TileDrawing.cs:4689-4730` (42-line window), verbatim key bodies:

```csharp
		case 100:
		{
			tileTop = 2;
			int num41 = tileFrameY / 2016;
			addFrY -= 2016 * num41;
			addFrX += 72 * num41;
			break;
		}
		case 530:
		{
			int num35 = y - tileFrameY % 36 / 18 + 2;
			int num36 = x - tileFrameX % 54 / 18;
			WorldGen.GetBiomeInfluence(num36, num36 + 3, num35, num35, out var corruptCount2, out var crimsonCount2, out var hallowedCount2);
			int num37 = corruptCount2;
			if (num37 < crimsonCount2)
			{
				num37 = crimsonCount2;
			}
			if (num37 < hallowedCount2)
			{
				num37 = hallowedCount2;
			}
			int num38 = 0;
			addFrY += 36 * ((corruptCount2 != 0 || crimsonCount2 != 0 || hallowedCount2 != 0) ? ((hallowedCount2 == num37) ? 1 : ((crimsonCount2 != num37) ? 3 : 2)) : 0);
			tileTop = 2;
			break;
		}
		case 705:
			tileTop = 2;
			break;
		case 485:
			tileTop = 2;
			addFrY = (Main.tileFrameCounter[typeCache] / 5 + (y - tileFrameY / 18 + (x - tileFrameX / 18))) % 4 * 36;
			break;
```

100's second-switch glow row (`TileDrawing.cs:6228-6235`), verbatim:

```csharp
		case 100:
			if (tileFrameX / 36 == 0 && tileFrameY / 36 == 27)
			{
				glowTexture = TextureAssets.GlowMask[68].Value;
				glowSourceRect = new Rectangle(tileFrameX, tileFrameY % 36, tileWidth, tileHeight);
				glowColor = this._martianGlow;
			}
			break;
```

| Id | C# body (verbatim key lines) | C# line | Rust arm | Verdict |
| --- | --- | --- | --- | --- |
| 378 | top-2 group (batch 5 head) | :4689-4696 | `drawdata.rs:256-258` | PORTED |
| 457 | shares the group | :4690 | :256-258 | PORTED |
| 466 | shares the group | :4691 | :256-258 | PORTED |
| 520 | shares the group | :4692 | :256-258 | PORTED |
| 651 | shares the group | :4693 | :256-258 | PORTED |
| 652 | shares the group | :4694 | :256-258 | PORTED |
| 100 | top 2 + 2016 stride unwind (`-= 2016*n41`, `+= 72*n41`); glow row 27 col 0 → mask 68, `_martianGlow` | :4697-4704, :6228-6235 | `drawdata.rs:261-266` (default `tile_frame*38`/0 makes `-=`/`+=` the same absolute form); glow `cases_a.rs:414-418` (`rect_plain(fx, fy % 36, height)`, martian) | PORTED; glow PORTED-UNWIRED (`_glow`) |
| 530 | 4-cell band influence probe + hallow/crimson/corrupt pick, `addFrY += 36*pick`, top 2 | :4705-4723 | static top `drawdata.rs:270-272` + full probe `cases_a.rs:207-230` (`oasis_plants_add_fr_y`, tie order hallow→crimson→corrupt preserved; `biome_influence` :182-200 ≈ `WorldGen.cs:47750-47777`), folded at `cases_a.rs:644-648` | PORTED |
| 705 | `tileTop = 2` | :4724-4726 | `drawdata.rs:274-276` | PORTED |
| 485 | top 2 + `(counter/5 + (y - fy/18 + (x - fx/18))) % 4 * 36` | :4727-4730 | static top `drawdata.rs:280-282` + full body `cases_b.rs:171-174` (counter = wired `tile_frame_counter`, `tiles.rs:666-670`) | PORTED |

Notes: the `drawdata.rs:277-279` (485) and `:267-269` (530) "documented deviation"
comments are STALE — the full bodies live in `cases_b`/`cases_a` and are wired
(`tiles.rs:700-712`, `:731-733`); same stale-comment pattern d63 found for 455/428.
C#'s dead `int num38 = 0` (:4719) has no effect and is correctly absent. The 530
probe rectangle is the single row `num35` across `num36..num36+3` inclusive — the
port's `biome_influence(world, num36, num36 + 3, num35, num35)` matches the
inclusive C# loop. 100's glow gate reads RAW fx/fy (`glow_switch` inputs are the
stored framing, `cases_a.rs:636-637`) — correct, the second switch never rewrites
frames; the rect's `fy % 36` is the raw row, not the unwound one, exactly as
`:6232`.
