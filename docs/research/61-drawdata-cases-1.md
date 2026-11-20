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
