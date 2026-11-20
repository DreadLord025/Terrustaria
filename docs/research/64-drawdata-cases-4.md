# 64 — DrawData case census, slice 4 (W4-d64)

> Wave 4 · researched 2026-11-20 · branch `task/W4-d64-drawdata-cases-4` @ base a725acb0 ·
> code-only mode (no builds/tests run) · sources: `decompiled/Terraria/GameContent/Drawing/TileDrawing.cs`,
> `crates/tr-core/src/drawdata.rs`, `crates/tr-core/src/drawdata_furniture.rs`,
> `crates/tr-app/src/drawdata_cases_a.rs`, `crates/tr-app/src/drawdata_cases_b.rs`,
> `crates/tr-app/src/render/tiles.rs`.

## Range claim (documented early, per wave protocol)

The four W4 drawdata slices partition the **C#-order union of `GetTileDrawData` case ids**
(first switch `TileDrawing.cs:4540-6032`, then the second/glow switch `:6049-6510` ids not
already in the first; 361 + 10 = **371 distinct ids**, plus the 215 campfire epilogue
`:6033-6044` which is not a `case` label). Consecutive quarters by index:

| Slice | Index range | Ids |
| --- | --- | --- |
| d61 | [0, 93) | 93 |
| d62 | [93, 186) | 93 |
| d63 | [186, 279) | 93 |
| **d64 (this doc)** | **[279, 371)** | **92 + the 215 epilogue** |

**d64 owns these 92 ids** (C# order; `@` = first-switch label line, glow-only ids cite the
second switch):

```
298@5758 299@5759 310@5760 339@5761 361@5762 362@5763 363@5764 364@5765
391@5766 392@5767 393@5768 394@5769 532@5770 533@5771 538@5772 544@5773
555@5774 556@5775 582@5776 619@5777 629@5778          (small-animal cages tail)
282@5833 505@5834 543@5835 598@5843 568@5851 569@5852 570@5853
288@5861 289@5862 290@5863 291@5864 292@5865 293@5866 294@5867 295@5868
360@5869 580@5870 620@5871 521@5884 522@5885 523@5886 524@5887 525@5888
526@5889 527@5890 316@5898 317@5899 318@5900          (water/jellyfish cages)
207@5908 410@5930 480@5945 509@5946 657@5947 720@5948 721@5949 725@5950
658@5962 733@5982                                     (fountain + monoliths)
326@5991 327@5992 328@5993 329@5994 345@5995 351@5996 421@5997 422@5998
458@5999 459@6000 708@6001                            (fall blocks + conveyors)
541@6004 507@6007 508@6008 336@6014 340@6015 341@6016 342@6017 343@6018
344@6019 89@6023 102@6026 753@6029                    (echo/starry/fire/benches)
19@6157 34@6268 10@6312 11@6323 88@6343 42@6363 87@6377 93@6414 101@6446
104@6466                                              (second-switch glow rows)
215@6033 (campfire epilogue, not a case label)
```

Overlap note: ids 568/569/570/580/657/89 appear in BOTH switches; the first-switch body
is audited here (their index falls in d64's quarter) and the second-switch glow rows are
cited alongside. The 21/467/441/468 chest glow rows belong to d62/d63 quarters and are
NOT claimed here.

## Headline census result — the "missing cases" premise is stale

A brace-balanced extraction of every outer `case` label in both switches
(`TileDrawing.cs:4540-6032` + `:6049-6510`, depth-1 labels only — the nested cage/monolith
sub-switches are bodies, not new cases) yields **371 distinct ids**. Diffing against the
union of all four port modules' match arms / guards:

| Port module | Ids covered (∩ C# set) |
| --- | --- |
| `tr-core/src/drawdata.rs::draw_data` (:130-1031) + 215 epilogue (:1036) | 353 + 215 |
| `tr-app/src/drawdata_cases_a.rs` (518/529/530 probes + glow switch) | 41 |
| `tr-app/src/drawdata_cases_b.rs::case_draw_data` (:107) | 16 |
| `tr-core/src/drawdata_furniture.rs::cage_table` (:111) + `is_basic_chest` | 94 |
| **Union** | **371 / 371 — 0 missing** |

The 18 ids absent from `draw_data` alone are all delegated, not missing:
719/518/579/453/456/541/507/508 → `cases_b`/`cases_a` (engine-driven arms, wired at
`render/tiles.rs:700`/`:731`); 19/34/10/11/88/42/87/93/101/104 → `cases_a` glow switch
(wired at `tiles.rs:731`). ROADMAP S1-F-9's "352 of 361, only 9 engine-state ids stay
default (5/453/456/507/508/518/541/579/719)" is now 353 of 361 — case 5 gained its static
20×20 arm in wave A (`c654e020`), leaving exactly the 8 delegated ids above.

Partition convention note: at base a725acb0 no sibling slice (d61/d62/d63) had documented
a range (their branches sit at the same base; d61's worktree holds only untracked census
scripts). This doc claims the LAST quarter in C# order and states the convention so the
other slices can take [0,93)/[93,186)/[186,279) without overlap.

**Therefore research/09's "≈113 of ≈376 case ids" (09-tile-drawing.md:31/111/188) and
90-consolidated-findings.md:50 are STALE** — they predate S1-F-9 waves A-E
(`2eedb54d`/`b38a1cb3`/`4bfffc19`/`16c00bcc`). The per-case audit below verifies the
d64 quarter arm-by-arm against the C# verbatim and records what is *actually* still
missing on the draw side (wiring, not cases).

## Per-case audit

Filled in per batch below. Verdict key: **PORTED** (arm faithful, wired),
**PORTED-UNWIRED** (arm faithful, no live caller), **DEVIATION** (documented approximation).

## REMAINDER claim (documented early, per wave protocol — 2026-11-20 continuation)

In-flight check before claiming: d61/d62/d63 branches all sit at base `a725acb0` with
**zero commits**; d61's worktree holds untracked census scripts
(`tools/_census_drawdata*.py`), d62's holds an untracked classification census
(`tools/_d62_census_out.txt`, scratch convention #1-65/#66-130/#131-195 — never
committed, superseded by the quarter convention above), d63's worktree is clean.
No sibling slice has landed any per-case audit. Per the continuation instruction
("DrawData missing cases REMAINDER, d61-d63 before"), this branch now claims the
**entire first three quarters — index [0, 279), 279 ids** — as the remainder, audited
in C# order in batches of ≤15 ids (one commit+push per batch). The re-run census
(depth-2-tab labels, `:4540-6032` + `:6049-6510`, first-switch order then
second-only) reproduces exactly 361 + 10 = 371 union ids; remainder = union[:279]:

```
  0 752@4542   1 751@4547   2 739@4552   3 748@4553   4 726@4556   5 719@4562
  6 443@4571   7 571@4581   8 698@4588   9 136@4593  10 561@4599  11 518@4604
 12 330@4631  13 331@4632  14 332@4633  15 333@4634  16 129@4637  17 5@4645
 18 583@4650  19 584@4651  20 585@4652  21 586@4653  22 587@4654  23 588@4655
 24 589@4656  25 596@4657  26 616@4658  27 634@4659  28 476@4663  29 323@4667
 30 4@4672    31 78@4680   32 85@4681   33 133@4682  34 134@4683  35 173@4684
 36 210@4685  37 233@4686  38 254@4687  39 283@4688  40 378@4689  41 457@4690
 42 466@4691  43 520@4692  44 651@4693  45 652@4694  46 100@4697  47 530@4705
 48 705@4724  49 485@4727  50 489@4731  51 490@4742  52 33@4819   53 49@4820
 54 174@4821  55 372@4822  56 646@4823  57 529@4827  58 3@4850    59 24@4851
 60 61@4852   61 71@4853   62 110@4854  63 201@4855  64 637@4856  65 703@4857
 66 20@4864   67 590@4865  68 595@4866  69 615@4873  70 16@4880   71 17@4881
 72 26@4882   73 32@4883   74 69@4884   75 72@4885   76 77@4886   77 124@4887
 78 137@4888  79 138@4889  80 352@4890  81 462@4891  82 487@4892  83 488@4893
 84 574@4894  85 575@4895  86 576@4896  87 577@4897  88 578@4898  89 664@4899
 90 695@4900  91 704@4901  92 712@4902  93 713@4903  94 714@4904  95 715@4905
 96 716@4906  97 79@4909   98 90@4917   99 18@4924  100 711@4931 101 654@4940
102 14@4943  103 21@4944  104 411@4945 105 467@4946 106 469@4947 107 15@4953
108 497@4954 109 172@4960 110 376@4961 111 27@4967  112 132@4973 113 135@4974
114 82@4978  115 83@4979  116 84@4980  117 324@4988 118 494@4997 119 52@5000
120 62@5001  121 115@5002 122 205@5003 123 382@5004 124 528@5005 125 636@5006
126 638@5007 127 80@5014  128 142@5015 129 143@5016 130 139@5019 131 73@5027
132 74@5028  133 113@5029 134 388@5037 135 389@5038 136 227@5052 137 624@5084
138 700@5085 139 656@5094 140 701@5095 141 579@5104 142 567@5131 143 185@5143
144 186@5144 145 187@5145 146 650@5184 147 649@5187 148 647@5195 149 706@5196
150 648@5199 151 178@5207 152 184@5213 153 519@5224 154 493@5231 155 28@5258
156 105@5259 157 470@5260 158 475@5261 159 506@5262 160 547@5263 161 548@5264
162 552@5265 163 560@5266 164 597@5267 165 613@5268 166 621@5269 167 622@5270
168 623@5271 169 653@5272 170 699@5273 171 617@5276 172 614@5281 173 81@5286
174 272@5295 175 106@5298 176 300@5301 177 301@5302 178 302@5303 179 303@5304
180 304@5305 181 305@5306 182 306@5307 183 307@5308 184 308@5309 185 354@5310
186 355@5311 187 499@5312 188 377@5316 189 463@5320 190 464@5321 191 491@5325
192 379@5329 193 349@5332 194 441@5342 195 468@5343 196 390@5357 197 412@5360
198 36@5364  199 406@5367 200 452@5387 201 455@5397 202 454@5409 203 453@5412
204 456@5415 205 405@5418 206 12@5433  207 31@5434  208 96@5435  209 639@5436
210 665@5437 211 696@5438 212 238@5441 213 593@5445 214 594@5466 215 592@5488
216 228@5491 217 231@5492 218 243@5493 219 247@5494 220 244@5498 221 565@5509
222 235@5520 223 217@5523 224 218@5524 225 564@5525 226 219@5529 227 220@5530
228 642@5531 229 270@5535 230 271@5536 231 581@5537 232 660@5564 233 572@5591
234 428@5605 235 442@5612 236 275@5629 237 276@5630 238 277@5631 239 278@5632
240 279@5633 241 280@5634 242 281@5635 243 296@5636 244 297@5637 245 309@5638
246 358@5639 247 359@5640 248 413@5641 249 414@5642 250 542@5643 251 550@5644
252 551@5645 253 553@5646 254 554@5647 255 558@5648 256 559@5649 257 599@5650
258 600@5651 259 601@5652 260 602@5653 261 603@5654 262 604@5655 263 605@5656
264 606@5657 265 607@5658 266 608@5659 267 609@5660 268 610@5661 269 611@5662
270 612@5663 271 632@5664 272 640@5665 273 643@5666 274 644@5667 275 645@5668
276 710@5669 277 285@5756 278 286@5757
```

Overlap notes: ids 21/467/441/468 (chest glow rows :6106-6156 region) fall in this
remainder and their second-switch rows are cited alongside; 568-570/580/657/89 stay
with d64's quarter (batches 2-6 above). The 285/286 cage labels' shared body
(:5756-5831) was quoted verbatim in batch 2 — the remainder audit cites it once more
for the two ids without re-auditing d64's 21 tail ids. Draw-side scope only: case
arms, their frame/top deltas, and the glow records they produce; consumption gaps
already recorded above (`_glow` TILE-DRAW-2, `furniture_post_pass` W4-d57,
`martianLight` W4-d64 batch 5) are referenced, not re-litigated.

Remainder batches are numbered R1…R19 (≤15 ids each, C# order).

### R1 — switch head: 752, 751, 739, 748, 726, 719, 443, 571, 698, 136, 561, 518, 330, 331, 332 (15 ids, indices 0-14)

C# `TileDrawing.cs:4542-4633`. The method prologue (:4529-4538) seeds
`tileTop = 0; tileWidth = 16; tileHeight = 16; halfBrickHeight = 0;
addFrY = Main.tileFrame[typeCache] * 38; addFrX = 0; tileSpriteEffect = None;
glowTexture = null;` — the port's `TileDrawData` default + `drawdata.rs:121-124`
(`add_fr_y: (tile_frame * 38) as i16`) reproduce it exactly.

Verbatim (:4542-4561):

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
```

| Id | C# body (verbatim key lines) | C# line | Rust arm | Verdict |
| --- | --- | --- | --- | --- |
| 752 | `tileHeight = 38; tileWidth = 36; tileTop = 2;` | :4542-4546 | `drawdata.rs:133-137` (height 38, width 36, top 2) | PORTED |
| 751 | `tileFrameY += (short)((x + y * 2) % 7 * 46);` (56×46) | :4547-4551 | :140-144 `frame_y_override = Some((fy + (x + y*2) % 7 * 46) as i16)` — the `(short)` wrap is the `as i16` (pinned :2191) | PORTED |
| 739 | `addFrY = Main.tileFrame[typeCache] * 90;` | :4552-4554 | :147-149 (`739 \| 748`, `tile_frame * 90`) | PORTED |
| 748 | shares 739's arm | :4553 | :147 | PORTED |
| 726 | `tileFrameX = 0; tileFrameY = 0; tileWidth = 20; tileHeight = 20;` | :4556-4561 | :152-157 (both overrides `Some(0)`, 20×20) | PORTED |
| 719 | `int num24 = (x + y + (int)(Main.GlobalTimeWrappedHourly * 15f)) % 14;` `addFrX += 288 * num25; addFrY += 270 * num26;` (`num25 = num24/4`, `num26 = num24%4`) | :4562-4570 | `cases_b.rs:125-131` — term-for-term, `+=` onto the `tile_frame*38` base; clock fed `anim_time % 3600.0` (`tiles.rs:358`, `Main.cs:16836`) | PORTED (delegated, wired :700) |
| 443 | `if (tileFrameX / 36 >= 2) { tileTop = -2; } else { tileTop = 2; }` | :4571-4580 | `drawdata.rs:163-169` | PORTED |
| 571 | `if (x % 2 == 0) { tileSpriteEffect = SpriteEffects.FlipHorizontally; } tileTop = 2;` | :4581-4587 | :172-175 (`flip = flip_x`, `flip_x = x % 2 == 0` :128; top 2) | PORTED |
| 698 | `tileWidth = 36; tileFrameX = (short)(tileFrameX / 18 * 38); tileHeight = 44;` | :4588-4592 | :178-182 (same order; `as i16` cast) | PORTED |
| 136 | `if (tileFrameX == 0) { tileTop = 2; }` | :4593-4598 | :185-189 | PORTED |
| 561 | `tileTop -= 2; tileHeight = 20; addFrY = tileFrameY / 18 * 4;` | :4599-4603 | :192-196 | PORTED |
| 518 | liquid float — verbatim below | :4604-4630 | `cases_a.rs::lily_pad_top_delta` :158-177 + `apply_cases_a` :639-642 (`top.wrapping_add(delta)`) | PORTED (delegated, wired :731) |
| 330 | `tileTop += 2;` | :4631-4636 | `drawdata.rs:200-202` (`330..=333`, `top += 2`) | PORTED |
| 331 | shares the candy-cane arm | :4632 | :200 | PORTED |
| 332 | shares the candy-cane arm | :4633 | :200 | PORTED |

518 verbatim (:4604-4630):

```csharp
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
```

Audit notes (R1):
- 518's probes are faithful: `solid_tile` (`cases_a.rs:129-139`) is
  `WorldGen.SolidTile` incl. the null-tile-is-solid rule (`WorldGen.cs:70111-70132`);
  `block_type` (:144-151) is `Tile.blockType()` verbatim (`Tile.cs:202-214`:
  halfBrick→1, slope>0→slope+1); `Framing.GetTileSafely` out-of-world returns the
  inactive default tile, matching `world.tile(..) == None` skipping the switch.
  The `tileTop -= num19` becomes `wrapping_add(-num19)` on i16 — values bounded
  ±16, no wrap in range. Tests pin the levels/slopes/wiring
  (`cases_a.rs:711/:740/:775`).
- 719 is delegated, not missing: `drawdata.rs:158-160`'s comment records the
  static default while `cases_b.rs:125-131` carries the full body; the live path
  runs it (`tiles.rs:700-712`). Test `case_719_rainbow_cloud_clock_grid`
  (`cases_b.rs:439`) pins the 14-step grid and the `+=` base.
- 751/752 draw only on their (0,0) frame — that gate lives in
  `apply_world_corrections` (`drawdata.rs:1358-1374`, the `DrawSingleTile`
  :1022-1039 correction), outside the case-arm scope but verified present.
- No second-switch glow rows exist for any R1 id (checked :6049-6510).

### R2 — trees, palms and the 20×20 groups: 333, 129, 5, 583-589, 596, 616, 634, 476, 323 (15 ids, indices 15-29)

C# `TileDrawing.cs:4634-4671`. Verbatim (:4637-4671):

```csharp
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
		case 583:
		case 584:
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

| Id | C# body | C# line | Rust arm | Verdict |
| --- | --- | --- | --- | --- |
| 333 | `tileTop += 2;` (candy-cane tail) | :4634-4636 | `drawdata.rs:200-202` (`330..=333`) | PORTED |
| 129 | `addFrY = 0;` + the `((num40 + tileFrame) % 6 - num40) * 18` small-crystal walk | :4637-4644 | :206-212 — term-for-term incl. the `fx >= 324` gate | PORTED |
| 5 | 20×20 + `tileFrameX += 176 * (GetTreeBiome + 1)` | :4645-4649 | static :219-222; probe in `apply_world_corrections` :1304-1307 → `tree_frame_x_offset` :1183-1185 → `tree_biome` :1142-1177 (wired `tiles.rs:683`) | PORTED |
| 583-589 | 20×20, no biome shift (own trunk sheets) | :4650-4662 | :227-230 (`583..=589 \| 596 \| 616 \| 634`) | PORTED |
| 596 | shares the variant arm | :4657 | :227 | PORTED |
| 616 | shares the variant arm | :4658 | :227 | PORTED |
| 634 | shares the variant arm | :4659 | :227 | PORTED |
| 476 | `tileWidth = 20; tileHeight = 18;` (GolfHole) | :4663-4666 | :232-235 | PORTED |
| 323 | 20×20 + `tileFrameY = 22 * GetPalmTreeBiome(x, y)` (REPLACES the stored row) | :4667-4671 | static :239-242; replacement in `apply_world_corrections` :1310-1312 → `palm_frame_y` :1101-1103 (`22 * biome`) → `palm_tree_biome` :1059-1068 | PORTED |

Audit notes (R2):
- `tree_biome` (:1142-1177) is `GetTreeBiome` (`TileDrawing.cs:1735-1776`)
  verbatim: the START-cell type is read BEFORE the column shifts (:1739 ↔ :1148,
  regardless of `active()`); the four shift gates match order-for-order
  (`fx==66 && fy<=45` → +1; `fx==88 && 66<=fy<=110` → −1; `fy>=198` → 66:−1/44:+1;
  else `fy>=132` → 22:−1/44:+1); the down-walk stops on the first inactive or
  different-type cell. The C# walk indexes `Main.tile` without bounds checks;
  the port's out-of-world `None` stops the walk — the same convention
  `palm_tree_biome` documents (:1056-1058). Ground classification
  (`GetTreeVariant`) is the d44-d49 framing lane's port (`tree_variant`);
  tests pin the offsets (`drawdata.rs:2339` — `176 * 4` gem-tree ground).
- `palm_tree_biome` walks down while `active() && type == 323`
  (`TileDrawing.cs:1726-1733`) then classifies the ground
  (`GetPalmTreeVariant`); tests `test_DW_2_1_palm_biome` (:2219) and
  `test_DW_2_1_palm_oasis_offset` (:2250) pin sand/crimsand/pearlsand/oasis.
- The C# `(short)` casts on 5/323 are the port's `as i16` on the overrides;
  the additive 5 shift and the replacing 323 row keep their distinct shapes
  (`+=` vs `=`) — `frame_x_override` adds `frame_x`, `frame_y_override` replaces.
- Kind 5's live app draw routes through the dedicated tree path
  (`tree::trunk_sheet_offset_at`, documented :216-218); the DrawData arm itself
  is complete and the world correction is applied on the generic path too.
- Glow cross-reference (correction, found in R5): 634 DOES have a second-switch
  glow row — `GlowMask[315]`, `Color.White`, rect `(tileFrameX, tileFrameY +
  addFrY, w, h)` (:6070-6074) — ported at `cases_a.rs:320` (`rect_anim_y`),
  produced but unconsumed (`_glow`, TILE-DRAW-2). The other R2 ids have no
  second-switch rows.

### R3 — torch + the top-2 furniture group: 4, 78, 85, 133, 134, 173, 210, 233, 254, 283, 378, 457, 466, 520, 651 (15 ids, indices 30-44)

C# `TileDrawing.cs:4672-4696`. Verbatim:

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

| Id | C# body | C# line | Rust arm | Verdict |
| --- | --- | --- | --- | --- |
| 4 | 20×20; `tileTop = 4` iff `WorldGen.SolidTile(x, y-1)` | :4672-4679 | `drawdata.rs:245-251` (`solid_above` param) | PORTED (probe delta, see note) |
| 78 | `tileTop = 2;` (ClayPot) | :4680-4696 | :256-258 (the 15-id `\|` arm) | PORTED |
| 85 | shares the top-2 arm (Tombstones) | :4681 | :256 | PORTED |
| 133 | shares the top-2 arm | :4682 | :256 | PORTED |
| 134 | shares (Forge) | :4683 | :256 | PORTED |
| 173 | shares (Candelabra) | :4684 | :256 | PORTED |
| 210 | shares (LandMine) | :4685 | :256 | PORTED |
| 233 | shares (PlantDetritus) | :4686 | :256 | PORTED |
| 254 | shares (Pumpkins) | :4687 | :256 | PORTED |
| 283 | shares (HeavyWorkBench) | :4688 | :256 | PORTED |
| 378 | shares (TargetDummy) | :4689 | :256 | PORTED |
| 457 | shares (PartyPresent) | :4690 | :256 | PORTED |
| 466 | shares (ElderCrystalStand) | :4691 | :256 | PORTED |
| 520 | shares (FoodPlatter) | :4692 | :256 | PORTED |
| 651 | shares (detritus echo) | :4693 | :256 | PORTED |

(652, the arm's last id, is index 45 — audited in R4; the shared body is quoted
once here.)

Audit notes (R3):
- **Case 4's probe predicate delta (already-documented deviation, re-verified):**
  the C# gate is `WorldGen.SolidTile(x, y-1)` (`WorldGen.cs:70111-70132`):
  null-tile → **true**; else `active && tileSolid && !tileSolidTop && !halfBrick
  && slope == 0 && !inActive`. The live feed (`tiles.rs:654-656`) is only
  `is_some_and(active && TILE_SOLID && !TILE_SOLID_TOP)` — it drops the
  half-brick/slope/inactive exclusions and takes out-of-world as **false**
  (C# true). A torch under a sloped or half-brick ceiling, or an actuated
  ceiling, keeps top 0 in the port where vanilla raises it 4. This is the
  ANALYSIS.md:3954-3957 "predicate delta … follow-up (Phase 4)" row — NOT a
  missing case; the arm itself (`drawdata.rs:245-251`) is verbatim and tests
  `torch_uses_solid_above` (:1509) / `test_DW_4_2_solid_above_scope` (:2870)
  pin the arm's contract. The exact predicate already exists in the tree
  (`cases_a.rs::solid_tile` :129-139) — the fix is a one-line feed swap,
  left to the wiring lane per scope.
- The top-2 group's 15 labels are one shared `tileTop = 2;` body — the port's
  single `|` arm is label-for-label identical (order irrelevant, no fallthrough
  arithmetic).
- No second-switch glow rows exist for any R3 id.

### R4 — candelabra stride, oasis/sea-oats biome rows, wind trio, candles, plants: 652, 100, 530, 705, 485, 489, 490, 33, 49, 174, 372, 646, 529, 3, 24 (15 ids, indices 45-59)

C# `TileDrawing.cs:4694-4863`. Verbatim key bodies:

```csharp
		case 100:
		{
			tileTop = 2;
			int num41 = tileFrameY / 2016;
			addFrY -= 2016 * num41;
			addFrX += 72 * num41;
			break;
		}
```

```csharp
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
```

```csharp
		case 529:
		{
			int num28 = y + 1;
			WorldGen.GetBiomeInfluence(x, x, num28, num28, out var corruptCount, out var crimsonCount, out var hallowedCount);
			int num29 = corruptCount;
			if (num29 < crimsonCount)
			{
				num29 = crimsonCount;
			}
			if (num29 < hallowedCount)
			{
				num29 = hallowedCount;
			}
			int num30 = 0;
			addFrY += 34 * ((corruptCount == 0 && crimsonCount == 0 && hallowedCount == 0) ? ((x < WorldGen.beachDistance || x > Main.maxTilesX - WorldGen.beachDistance) ? 1 : 0) : ((hallowedCount == num29) ? 2 : ((crimsonCount != num29) ? 4 : 3))) - tileFrameY;
			tileHeight = 32;
			tileTop = -14;
			if (x % 2 == 0)
			{
				tileSpriteEffect = SpriteEffects.FlipHorizontally;
			}
			break;
		}
```

| Id | C# body | C# line | Rust arm | Verdict |
| --- | --- | --- | --- | --- |
| 652 | `tileTop = 2;` (top-2 group tail, R3's body) | :4694-4696 | `drawdata.rs:256-258` | PORTED |
| 100 | top 2; `num41 = fy / 2016; addFrY -= 2016*num41; addFrX += 72*num41;` | :4697-4704 | :261-266 — `add_fr_y = tile_frame*38 - 2016*n41` folds the `-=` onto the prologue default; `add_fr_x = 72*n41` (default 0) | PORTED |
| 530 | biome band under the anchor, `addFrY += 36 * pick`, top 2 | :4705-4722 | static top 2 :270-272; `cases_a.rs::oasis_plants_add_fr_y` :207-230 + `apply_cases_a` :644-648 (`wrapping_add`) | PORTED (delegated, wired :731) |
| 705 | `tileTop = 2;` (OasisPlantsEcho) | :4724-4726 | :274-276 | PORTED |
| 485 | top 2; `addFrY = (tileFrameCounter/5 + (y - fy/18 + (x - fx/18))) % 4 * 36` | :4727-4730 | static top 2 :280-282; `cases_b.rs:171-174` verbatim | PORTED (delegated, wired :700) |
| 489 | top 2; wind-gated `(counter/5 + (n32+n33)) % 16 * 54`, probe `(x - fx/18, y - fy/18, 2, 3)` | :4731-4741 | static :286-288; `cases_b.rs:178-185` — outside wind keeps the `tile_frame*38` base, matching | PORTED (delegated, wired) |
| 490 | top 2; the two bob tables + `%12*36` wrap | :4742-4818 | static :293-295; `cases_b.rs:189-242` — both 8-arm tables value-for-value (strong :4752-4778, calm :4782-4808), `n12<0 → +12`, `%12*36` | PORTED (delegated, wired) |
| 33 | `tileHeight = 20; tileTop = -4;` (candles) | :4819-4826 | `drawdata.rs:298-301` (`33 \| 49 \| 174 \| 372 \| 646`) | PORTED |
| 49 | shares the candle arm | :4820 | :298 | PORTED |
| 174 | shares | :4821 | :298 | PORTED |
| 372 | shares | :4822 | :298 | PORTED |
| 646 | shares | :4823 | :298 | PORTED |
| 529 | 32 tall, top −14, parity flip + the biome/beach `34*pick - fy` rewrite | :4827-4849 | static :306-310; `cases_a.rs::sea_oats_add_fr_y` :238-262 + :649-655 (beach 380) | PORTED (delegated, wired) |
| 3 | `tileHeight = 20;` + parity flip (plants) | :4850-4863 | `drawdata.rs:313-316` (`3 \| 24 \| 61 \| 71 \| 110 \| 201 \| 637 \| 703`) | PORTED |
| 24 | shares the plant arm | :4851 | :313 | PORTED |

Audit notes (R4):
- `biome_influence` (`cases_a.rs:182-200`) is `WorldGen.GetBiomeInfluence`
  (`WorldGen.cs:47750-47777`) verbatim: inclusive rect, null cells skipped,
  three independent set tests (a tile can count in more than one set).
  `beachDistance = 380` pinned (`WorldGen.cs:86914`); `Main.maxTilesX` ↔
  `world.width`. The 530 tie order (hallow → crimson → corrupt) and the 529
  beach-band pick are the exact nested-ternary shapes; the dead C# locals
  `num38`/`num30` (`= 0`, never read) have no port counterpart — no behaviour.
- 490's `Main.WindForVisuals` is `Main.windSpeedCurrent` (`Main.cs:2823`
  expression-bodied property) — the port reads `wind.speed_current`, matching.
  A bobframe outside 0..=7 leaves `num13 = 0` in C# (no default arm); the
  port's `_ => 0` is that same value. The animator's bobframe bump
  (`Main.cs:19325`, `++weatherVaneBobframe == 8` reset) is the S1-G-1f lane.
- 485/489/490's static arms keep only `top = 2`; the full bodies run in
  `cases_b` BEFORE slice A — the C# first-switch order. The 489 no-wind path
  leaves `addFrY` at the prologue default in both.
- Glow cross-references (correction, found via the full second-switch label
  census in R6): 100 has a glow row — `fx/36 == 0 && fy/36 == 27` →
  `GlowMask[68]`, `_martianGlow`, rect `(fx, fy % 36, w, h)` (:6228-6235) —
  ported at `cases_a.rs:414-418`; 33 has one — `fx/18 == 0 && fy/22 == 26` →
  `GlowMask[61]`, `_martianGlow`, rect `(fx, fy % 22, w, h)` (:6236-6243, the
  candle sheet's 22 px stride) — ported :421-425. Both produced, unconsumed
  (`_glow`). No other R4 id has a second-switch row.

### R5 — plant/sapling tails + the 18-px group head: 61, 71, 110, 201, 637, 703, 20, 590, 595, 615, 16, 17, 26, 32, 69 (15 ids, indices 60-74)

C# `TileDrawing.cs:4850-4908` (the plant/sapling bodies were quoted verbatim in
R4's window; the 18-px group head):

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

| Id | C# body | C# line | Rust arm | Verdict |
| --- | --- | --- | --- | --- |
| 61 | plant arm: `tileHeight = 20;` + parity flip | :4852 | `drawdata.rs:313-316` | PORTED |
| 71 | shares the plant arm | :4853 | :313 | PORTED |
| 110 | shares | :4854 | :313 | PORTED |
| 201 | shares | :4855 | :313 | PORTED |
| 637 | shares (MushroomBlock plant shape) + second-switch glow | :4856, :6075-6079 | :313; glow `cases_a.rs:324` | PORTED (glow unconsumed + texture deviation) |
| 703 | shares the plant arm | :4857 | :313 | PORTED |
| 20 | sapling arm: `tileHeight = 18;` + parity flip | :4864-4872 | `drawdata.rs:319-322` (`20 \| 590 \| 595 \| 615`) | PORTED |
| 590 | shares the sapling arm | :4865 | :319 | PORTED |
| 595 | shares | :4866 | :319 | PORTED |
| 615 | own label, identical body (`tileHeight = 18;` + flip) | :4873-4879 | :319 (folded into the same `\|` arm — same values) | PORTED |
| 16 | `tileHeight = 18;` (Anvils) | :4880-4908 | `drawdata.rs:327-346` (the 27-label `\|` arm) | PORTED |
| 17 | shares (Furnaces) | :4881 | :327 | PORTED |
| 26 | shares (DemonAltar) | :4882 | :327 | PORTED |
| 32 | shares (thorns) | :4883 | :327 | PORTED |
| 69 | shares (MushroomTrees) | :4884 | :327 | PORTED |

Audit notes (R5):
- The 18-px arm's label set is exactly the C#'s 27 labels: the port spells
  574-578 as `574..=578` and 712-716 as `712..=716` — same ids, same single
  `height = 18` assignment, no other state touched.
- 637's second-switch glow verbatim (:6075-6079):

```csharp
		case 637:
			glowTexture = this.GetTileDrawTexture(tileCache, x, y);
			glowSourceRect = new Rectangle(tileFrameX + addFrX, tileFrameY + addFrY, tileWidth, tileHeight);
			glowColor = Color.Lerp(Color.White, color, 0.75f);
			break;
```

  Port `cases_a.rs:324`: `rect_framed(height)` = `(fx + add_fr_x, fy + add_fr_y,
  w, h)` — faithful; `lerp_white_75` (:300-303) is the XNA per-byte float lerp
  with truncation (`lerp(255, 0, .75) = 63.75 → 63`, pinned :912). Two recorded
  deviations: the glow TEXTURE is the painted tile sheet
  (`GetTileDrawTexture`), not a GlowMask — the port emits `mask: 0` and does
  not bind the sheet (:46-48, :321-323); and the record is unconsumed
  (`tiles.rs:731` `_glow`, TILE-DRAW-2). The cell colour fed is the shaded
  light bytes (`tiles.rs:724-730`, alpha 255 — `Lighting.GetColor` has no
  alpha), matching the C# `color` local (:4539).
- 615's C# label is separate from the 20/590/595 group but the body is
  byte-identical; folding it into one Rust arm changes no behaviour.
- No other R5 id has a second-switch row (638's row :6080-6084 belongs to
  index 126, audited in R9).

### R6 — the 18-px group middle: 72, 77, 124, 137, 138, 352, 462, 487, 488, 574, 575, 576, 577, 578, 664 (15 ids, indices 75-89)

C# `TileDrawing.cs:4885-4908` — every id is a label of the single
`tileHeight = 18;` body quoted verbatim in R5. Port: `drawdata.rs:327-346`
(the same 27-label arm; 574-578 spelled `574..=578`).

| Id | C# label line | Kind | Rust arm | Verdict |
| --- | --- | --- | --- | --- |
| 72 | :4885 | Hellforge | :327 arm | PORTED |
| 77 | :4886 | WoodenBeam | :327 | PORTED |
| 124 | :4887 | Traps (dart/gas) | :327 | PORTED |
| 137 | :4888 | Boulder | :327 | PORTED |
| 138 | :4889 | DjinnLamp | :327 | PORTED |
| 352 | :4890 | PicnicTable | :327 | PORTED |
| 462 | :4891 | FallenLog | :327 | PORTED |
| 487 | :4892 | beams (echo) | :327 | PORTED |
| 488 | :4893 | BouncyBoulder | :327 | PORTED |
| 574-578 | :4894-4898 | altars/boulders family | `574..=578` :341 | PORTED |
| 664 | :4899 | (1.4.5 furniture) | :342 | PORTED |

Audit notes (R6):
- Full second-switch label census (38 labels, :6049-6510): none of the R6 ids
  appears — no glow rows to cross-reference.
- The arm sets only `height`; width stays 16, top stays 0, `add_fr_y` keeps
  the prologue `tileFrame * 38` — identical in C# and port for every id here.

### R7 — 18-px tail, beds/bathtubs/workbenches strides, boulder, chest-row head: 695, 704, 712-716, 79, 90, 18, 711, 654, 14, 21, 411 (15 ids, indices 90-104)

C# `TileDrawing.cs:4900-4952`. Verbatim (:4909-4952):

```csharp
		case 79:
		{
			tileHeight = 18;
			int num23 = tileFrameY / 2016;
			addFrY -= 2016 * num23;
			addFrX += 144 * num23;
			break;
		}
		case 90:
		{
			int num22 = tileFrameY / 2016;
			addFrY -= 2016 * num22;
			addFrX += 144 * num22;
			break;
		}
		case 18:
		{
			int num21 = tileFrameX / 2016;
			addFrX -= 2016 * num21;
			addFrY += 20 * num21;
			break;
		}
		case 711:
			if (tileFrameX > 0)
			{
				tileWidth = 18;
			}
			tileHeight = 20;
			glowTexture = TextureAssets.Tile[711].Value;
			glowSourceRect = new Rectangle(tileFrameX + addFrX, tileFrameY + addFrY, tileWidth, tileHeight);
			break;
		case 654:
			tileTop += 2;
			break;
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

| Id | C# body | C# line | Rust arm | Verdict |
| --- | --- | --- | --- | --- |
| 695, 704, 712-716 | `tileHeight = 18;` (18-px group tail) | :4900-4908 | `drawdata.rs:327-346` | PORTED |
| 79 | 18 tall + the 2016 row stride (`-2016*n`, `+144*n`) | :4909-4916 | :350-355 (fold onto `tile_frame*38`); glow :6191-6207 → `cases_a.rs:387-396` (rows 27/28, masks 53/114, rect `(fx, fy%36)`) | PORTED (glow unconsumed) |
| 90 | stride only, NO height change | :4917-4923 | :358-362; glow :6174-6190 → `cases_a.rs:376-385` (masks 52/113) | PORTED (glow unconsumed) |
| 18 | stride in frame X (`fx/2016`), `+20*n` rows | :4924-4930 | :365-369; glow :6429-6445 → `cases_a.rs:555-564` (cols 27/28, masks 69/125, rect `(fx%36, fy)`) | PORTED (glow unconsumed) |
| 711 | width 18 iff `fx > 0`; height 20; **glow in the FIRST switch** (`Tile[711]`, framed rect, colour stays `Color.Transparent`) | :4931-4939 | :372-377 (metrics only — glow NOT ported, see note) | DEVIATION (glow record missing) |
| 654 | `tileTop += 2;` (TNTBarrel) | :4940-4942 | :379-381 | PORTED |
| 14 | `if (tileFrameY == 18) tileHeight = 18;` | :4943-4952 | :385-389 (`14 \| 21 \| 411 \| 467 \| 469`); glow :6397-6413 → `cases_a.rs:532-541` (cols 31/32, masks 67/124, rect `(fx%54, fy)`, no stride) | PORTED (glow unconsumed) |
| 21 | shares the row-18 arm | :4944 | :385; glow :6276-6293 → `cases_a.rs:450-459` (cols 48/49, masks 56/117, rect `(fx%36, fy)`) | PORTED (glow unconsumed) |
| 411 | shares the row-18 arm (Detonator) | :4945 | :385 | PORTED |

Audit notes (R7):
- **711's first-switch glow is the one genuinely unported glow record found so
  far.** The C# sets `glowTexture = TextureAssets.Tile[711].Value` +
  `glowSourceRect = (tileFrameX + addFrX, tileFrameY + addFrY, tileWidth,
  tileHeight)` with `glowColor` left at the prologue's `Color.Transparent`
  (:4538). The port's arm carries only the metrics and cites the glow as
  "TILE-DRAW-2" (`drawdata.rs:370-371`), but unlike 637 (which emits a `mask: 0`
  record with the painted-sheet deviation, `cases_a.rs:46-48/:324`) NO record is
  produced for 711 anywhere — `glow_switch` has no 711 arm (711 is absent from
  the second-switch label census). Draw-side impact today: none (all glow
  records are dropped at `tiles.rs:731`), and the transparent colour makes the
  vanilla additive draw a no-op too; but the record shape should exist for the
  glow pass. Seed: emit `GlowData { mask: 0 /* Tile[711] sheet */, src:
  rect_framed, color: [0,0,0,0] }` from the first-switch port when TILE-DRAW-2
  lands. Recorded here as the R7 residual.
- 79/90/18's stride folds are the same prologue-default arithmetic as case 100
  (R4): `-=` becomes `tile_frame*38 - stride*n`, `+=` on X becomes `0 + step*n`.
  The second-switch gates read RAW fx/fy (never rewritten), matching
  `cases_a.rs:30-34`'s convention; 18's first-switch stride is in X while its
  glow gate is `fx/36` columns — both faithful.
- 21/467 and 441/468 share identical chest glow bodies in C# (two separate
  case groups, :6276-6293 / :6294-6311); the port keeps two arms with the same
  values — label-faithful. 467/441/468's first-switch indices fall in R8/R13.
- 14/21/411's `tileFrameY == 18` is an EXACT equality (not `%`): only the
  second sheet row shrinks; the port matches (`fy == 18`).

### R8 — chest-row tail, chairs/sinks strides, sunflower, plates, herbs, vines head: 467, 469, 15, 497, 172, 376, 27, 132, 135, 82, 83, 84, 324, 494, 52 (15 ids, indices 105-119)

C# `TileDrawing.cs:4946-5013`. Verbatim (:4953-4999):

```csharp
		case 15:
		case 497:
			if (tileFrameY % 40 == 18)
			{
				tileHeight = 18;
			}
			break;
		case 172:
		case 376:
			if (tileFrameY % 38 == 18)
			{
				tileHeight = 18;
			}
			break;
		case 27:
			if (tileFrameY % 74 == 54)
			{
				tileHeight = 18;
			}
			break;
		case 132:
		case 135:
			tileTop = 2;
			tileHeight = 18;
			break;
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
		case 324:
			tileWidth = 20;
			tileHeight = 20;
			tileTop = -2;
			if (x % 2 == 0)
			{
				tileSpriteEffect = SpriteEffects.FlipHorizontally;
			}
			break;
		case 494:
			tileTop = 2;
			break;
```

| Id | C# body | C# line | Rust arm | Verdict |
| --- | --- | --- | --- | --- |
| 467 | row-18 arm (Containers2) | :4946 | `drawdata.rs:385-389`; glow :6277 (shares 21's body, R7) → `cases_a.rs:450-459` | PORTED (glow unconsumed) |
| 469 | row-18 arm (Tables2) | :4947 | :385 | PORTED |
| 15 | `fy % 40 == 18` → height 18 | :4953-4959 | :392-396; second switch :6244-6267 → `cases_a.rs:428-442` (2040 stride BOTH kinds, glow only kind 15 rows 32/33, masks 54/116) | PORTED (glow unconsumed) |
| 497 | shares the `%40` arm (toilets) | :4954 | :392; stride applies, no glow (`kind == 15` gate) — faithful | PORTED |
| 172 | `fy % 38 == 18` → height 18 | :4960-4966 | :399-403; second switch :6487-6506 → `cases_a.rs:598-610` (2014 stride, rows 28/29, masks 88/122, rect `(fx, fy%38)`) | PORTED (glow unconsumed) |
| 376 | shares the `%38` arm (FishingCrate) | :4961 | :399 | PORTED |
| 27 | `fy % 74 == 54` → height 18 (Sunflower) | :4967-4972 | :406-410 | PORTED |
| 132 | `tileTop = 2; tileHeight = 18;` (Lever) | :4973-4977 | :413-416 | PORTED |
| 135 | shares (PressurePlates) | :4974 | :413 | PORTED |
| 82 | herbs: 20 tall, top −2, parity flip | :4978-4987 | :419-423 | PORTED |
| 83 | shares | :4979 | :419 | PORTED |
| 84 | shares | :4980 | :419 | PORTED |
| 324 | BeachPiles: 20×20, top −2, parity flip | :4988-4996 | :426-431 | PORTED |
| 494 | `tileTop = 2;` (GolfTee) | :4997-4999 | :433-435 | PORTED |
| 52 | vine arm: `tileTop = -2;` + parity flip | :5000-5013 | :438-441 (`52 \| 62 \| 115 \| 205 \| 382 \| 528 \| 636 \| 638`) | PORTED |

Audit notes (R8):
- 15/497's second-switch stride verbatim (:6247-6250): `num46 = fy/40;
  num53 = num46/51; addFrY -= 2040*num53; addFrX += 36*num53;` — the port's
  deltas match term-for-term; the `typeCache == 15` glow gate is the port's
  `kind == 15`. 172's 2014 = 53×38 stride likewise (:6489-6492).
- The `%`-row gates (40/38/74) are the sheet strides of the chair/sink/
  sunflower families — distinct from R7's exact `== 18`; both shapes ported.
- 638 shares the vine arm here (index 126 audits its glow row :6080-6084);
  62/115/205/382/528/636 follow in R9. No other R8 id has a second-switch row.

### R9 — vine tail, cacti, music boxes, jungle plants, tall gates: 62, 115, 205, 382, 528, 636, 638, 80, 142, 143, 139, 73, 74, 113, 388 (15 ids, indices 120-134)

C# `TileDrawing.cs:5001-5051`. Verbatim (:5019-5051):

```csharp
		case 139:
		{
			tileTop = 2;
			int num3 = tileFrameY / 2016;
			addFrY -= 2016 * num3;
			addFrX += 72 * num3;
			break;
		}
		case 73:
		case 74:
		case 113:
			tileTop = -12;
			tileHeight = 32;
			if (x % 2 == 0)
			{
				tileSpriteEffect = SpriteEffects.FlipHorizontally;
			}
			break;
		case 388:
		case 389:
		{
			int num42 = 94;
			tileTop = -2;
			if (tileFrameY == num42 - 20 || tileFrameY == num42 * 2 - 20 || tileFrameY == 0 || tileFrameY == num42)
			{
				tileHeight = 18;
			}
			if (tileFrameY != 0 && tileFrameY != num42)
			{
				tileTop = 0;
			}
			break;
		}
```

| Id | C# body | C# line | Rust arm | Verdict |
| --- | --- | --- | --- | --- |
| 62, 115, 205, 382, 528, 636 | vine arm: top −2 + parity flip | :5001-5013 | `drawdata.rs:438-441` | PORTED |
| 638 | shares the vine arm | :5007 | :438; glow :6080-6084 → `cases_a.rs:327` (GlowMask 327, `rect_framed(height − halfBrickHeight)`, `lerp_white_75(color)`) | PORTED (glow unconsumed) |
| 80 | `tileTop = 2;` (cacti) | :5014-5018 | :447-449 | PORTED |
| 142 | shares (Pumps) | :5015 | :447 | PORTED |
| 143 | shares | :5016 | :447 | PORTED |
| 139 | top 2 + 2016 stride (`−2016*n`, `+72*n`) | :5019-5026 | :452-457 | PORTED |
| 73 | jungle plants: top −12, 32 tall, parity flip | :5027-5036 | :460-464 | PORTED |
| 74 | shares | :5028 | :460 | PORTED |
| 113 | shares | :5029 | :460 | PORTED |
| 388 | TallGates: the 94-grid seam rows + top reset | :5037-5051 | :467-476 (`N42 = 94`, both gates in C# order) | PORTED |

Audit notes (R9):
- 638's glow verbatim (:6080-6084): `GlowMask[327]`, rect
  `(tileFrameX + addFrX, tileFrameY + addFrY, tileWidth, tileHeight -
  halfBrickHeight)`, `Color.Lerp(Color.White, color, 0.75f)` — the port's
  `rect_framed(height - half_brick_height)` + `lerp_white_75` match; the
  half-brick cut is fed `if tile.half_brick() { 8 } else { 0 }`
  (`cases_a.rs:658`), the C# `halfBrickHeight` out-param's only writer being
  the halfBrick epilogue (:6045-6048). Test :906-913 pins the cut + lerp.
- 388/389's two gates keep their C# ORDER: the height gate runs first, then
  the top reset (`fy != 0 && fy != 94` → top 0) — so rows 74/168 get height 18
  AND top 0, rows 0/94 keep top −2. The port reproduces both statements in
  sequence (:470-475).
- 80's `DrawSingleTile` ground-sand row shift (+54/108/162, :1132-1147) is a
  draw-local correction, ported in `apply_world_corrections` (BUG-19, pinned
  `drawdata.rs::bug19_the_cactus_shift_lands_on_the_flat_draw_rect`) — outside
  the case-arm scope, verified present.
- No other R9 id has a second-switch row.

### R10 — dye plants, Abigail/glow-tulip, golem head, gnome, piles: 389, 227, 624, 700, 656, 701, 579, 567, 185, 186, 187, 650, 649, 647, 706 (15 ids, indices 135-149)

C# `TileDrawing.cs:5038-5198`. Verbatim (:5104-5142):

```csharp
		case 579:
		{
			tileWidth = 20;
			tileHeight = 20;
			tileTop -= 2;
			bool flag2 = (float)(x * 16 + 8) > Main.LocalPlayer.Center.X;
			if (tileFrameX > 0)
			{
				if (flag2)
				{
					addFrY = 22;
				}
				else
				{
					addFrY = 0;
				}
			}
			else if (flag2)
			{
				addFrY = 0;
			}
			else
			{
				addFrY = 22;
			}
			break;
		}
		case 567:
			tileWidth = 26;
			tileHeight = 18;
			if (tileFrameY == 0)
			{
				tileTop = -2;
			}
			if (x % 2 == 0)
			{
				tileSpriteEffect = SpriteEffects.FlipHorizontally;
			}
			break;
```

The piles inner switch verbatim (:5146-5182):

```csharp
			tileTop = 2;
			switch (typeCache)
			{
			case 185:
				if (tileFrameY == 18 && tileFrameX >= 576 && tileFrameX <= 882)
				{
					Main.tileShine2[185] = true;
				}
				else
				{
					Main.tileShine2[185] = false;
				}
				if (tileFrameY == 18)
				{
					int num18 = tileFrameX / 1908;
					addFrX -= 1908 * num18;
					addFrY += 18 * num18;
				}
				break;
			case 186:
				if (tileFrameX >= 864 && tileFrameX <= 1170)
				{
					Main.tileShine2[186] = true;
				}
				else
				{
					Main.tileShine2[186] = false;
				}
				break;
			case 187:
			{
				int num17 = tileFrameX / 1890;
				addFrX -= 1890 * num17;
				addFrY += 36 * num17;
				break;
			}
			}
```

| Id | C# body | C# line | Rust arm | Verdict |
| --- | --- | --- | --- | --- |
| 389 | shares 388's tall-gate body (R9) | :5038 | `drawdata.rs:467-476` | PORTED |
| 227 | 32×38; top −6 iff `fx == 238` else −20; `fx == 204` → GetCactusType rewrite (+238 good/+204 evil/+272 crimson); parity flip | :5052-5083 | static :482-487; rewrite `apply_world_corrections` :1316-1320 → `cactus_type` :1193 (`WorldGen.cs:47687-47737`) + `dye_frame_x_offset` :1268-1270 | PORTED (wired :683) |
| 624 | 20×16, `top += 2`, parity flip (AbigailsFlower) | :5084-5093 | :490-495 | PORTED |
| 700 | shares | :5085 | :490 | PORTED |
| 656 | 24×34, `top -= 16`, parity flip (GlowTulip) | :5094-5103 | :498-503; glow :6064-6069 → `cases_a.rs:315-318` (GlowMask 329, white × text pulse, alpha 0) | PORTED (glow unconsumed; pulse fed 255) |
| 701 | shares + same glow | :5095 | :498; :315 | PORTED (glow unconsumed) |
| 579 | 20×20, `top -= 2`, eye row faces the player | :5104-5130 | `cases_b.rs:134-147` — full body incl. the 4-way row pick; `player_center_x` fed `p.center().0` (`tiles.rs:355`) | PORTED (delegated, wired :700) |
| 567 | 26×18, top −2 iff `fy == 0`, parity flip (GardenGnome) | :5131-5142 | `drawdata.rs:509-516` | PORTED |
| 185 | top 2; row-18 1908 stride; `tileShine2[185]` write | :5149-5164 | :523-529 (stride faithful; shine write NOT ported — documented :518) | PORTED (shine deviation) |
| 186 | top 2; ONLY the `tileShine2[186]` write | :5165-5174 | :520-521 top 2 (shine not ported) | PORTED (shine deviation) |
| 187 | top 2; 1890 stride (`−1890*n`, `+36*n`) | :5175-5181 | :530-534 | PORTED |
| 650 | `tileTop = 2;` | :5184-5186 | :539-541 | PORTED |
| 649 | top 2 + 1908 stride (echo piles) | :5187-5194 | :544-549 | PORTED |
| 647 | `tileTop = 2;` | :5195-5198 | :552-554 | PORTED |
| 706 | shares | :5196 | :552 | PORTED |

Audit notes (R10):
- **185/186's `Main.tileShine2` writes are draw-side side effects the port does
  not model** (documented `drawdata.rs:517-519`, `tiles.rs:1582` — the sparkle
  overlay `Main.shine` :4400 is unported). The frame maths around them is
  complete; 186's case body is ONLY the shine write + top 2, so its port arm is
  top-2-only by design. This is a lighting/sparkle lane residual, not a missing
  case arm.
- 185's stride gate is `fy == 18` ONLY (the shine gate additionally bounds
  `576 <= fx <= 882`); the port keeps the stride gate exact and drops the shine
  gate with the write. 187/648's 1890 stride is unconditional.
- 227's three independent `if`s sum to the same offset as the port's literal
  form (flags mutually exclusive by construction, :1266-1267); the rewrite
  fires only on the `fx == 204` column, matching the C# gate.
- 656/701's glow colour `new Color(255,255,255,0) * (mouseTextColor/255f)` —
  the port's `(255.0 * pulse) as u8` with alpha 0; `mouseTextColor` fed steady
  255 (`tiles.rs:732`, documented `cases_a.rs:51-52`).
- No other R10 id has a second-switch row.

### R11 — echo piles tail, gems, moss, cattail, golf flag, statue group head: 648, 178, 184, 519, 493, 28, 105, 470, 475, 506, 547, 548, 552, 560, 597 (15 ids, indices 150-164)

C# `TileDrawing.cs:5199-5274`. Verbatim (:5207-5230):

```csharp
		case 178:
			if (tileFrameY <= 36)
			{
				tileTop = 2;
			}
			break;
		case 184:
			tileWidth = 20;
			if (tileFrameY <= 36)
			{
				tileTop = 2;
			}
			else if (tileFrameY <= 108)
			{
				tileTop = -2;
			}
			break;
		case 519:
			tileTop = 2;
			if (x % 2 == 0)
			{
				tileSpriteEffect = SpriteEffects.FlipHorizontally;
			}
			break;
```

493 verbatim (:5231-5257):

```csharp
		case 493:
			if (tileFrameY == 0)
			{
				int num4 = Main.tileFrameCounter[typeCache];
				float num5 = Math.Abs(Main.WindForVisuals);
				int num6 = y - tileFrameY / 18;
				int num7 = x - tileFrameX / 18;
				if (!this.InAPlaceWithWind(x, num6, 1, 1))
				{
					num5 = 0f;
				}
				if (!(num5 < 0.1f))
				{
					if (num5 < 0.5f)
					{
						num4 = (num4 / 20 + (num6 + num7)) % 6;
						addFrY = ((!(Main.WindForVisuals < 0f)) ? (num4 + 1) : (6 - num4)) * 36;
					}
					else
					{
						num4 = (num4 / 10 + (num6 + num7)) % 6;
						addFrY = ((!(Main.WindForVisuals < 0f)) ? (num4 + 7) : (12 - num4)) * 36;
					}
				}
			}
			tileTop = 2;
			break;
```

| Id | C# body | C# line | Rust arm | Verdict |
| --- | --- | --- | --- | --- |
| 648 | top 2 + 1890 stride | :5199-5206 | `drawdata.rs:557-562` | PORTED |
| 178 | top 2 iff `fy <= 36` (ExposedGems) | :5207-5212 | :565-569 | PORTED |
| 184 | 20 wide; top 2 / −2 / 0 by row bands | :5213-5223 | :572-579; glow :6114-6151 → `cases_a.rs:346-360` (six columns, mask 127, RAW-frame rect) | PORTED (glow unconsumed; disco deviation) |
| 519 | top 2 + parity flip (Cattail) | :5224-5230 | :582-585 | PORTED |
| 493 | the wind-tier flag wave (verbatim above) | :5231-5257 | static top 2 :588-590; `cases_b.rs:247-277` full body | PORTED (delegated, wired :700) |
| 28, 105, 470, 475, 506, 547, 548, 552, 560, 597 | `tileTop = 2;` (16-label statue/pot group) | :5258-5275 | `drawdata.rs:593-596` (the 16-id `\|` arm) | PORTED |

Audit notes (R11):
- 493's quirks all carried: the wind probe uses the RAW `x` (not `num7`,
  :5238 — pinned in the port comment `cases_b.rs:253-254`); the calm tier
  keeps the prologue `addFrY` default; the sign tests are `!(wind < 0f)`
  (negative-zero-safe), ported as `!(env.wind.speed_current < 0.0)`; the
  `tileTop = 2` runs AFTER the `fy == 0` block regardless. Test
  `case_493_golf_flag_wind_tiers` (`cases_b.rs:606`) pins both tiers, both
  signs, the row gate and the below-surface calm path.
- 184's glow: six independent column `if`s (110/132/154/176/198/220), all
  GlowMask[127], rect `(tileFrameX, tileFrameY, w, h)` — RAW frame, no addFr*
  (the C# reads `tileCache.frameX` for the gates and `tileFrameX` for the
  rect; both are the stored frame). The five moss colours are the
  `TileDrawing.cs:132-140` constants (`cases_a.rs:114-123`); column 220's
  `new Color(Main.DiscoR, Main.DiscoG, Main.DiscoB)` is engine state not
  routed — the port emits `[0, 0, 0, 255]` (documented :47-48/:344-345; the
  `Color(byte,byte,byte)` ctor sets alpha 255). Rect + mask ported; colour
  pending the disco clock.
- The statue group's remaining six labels (613/621/622/623/653/699) are
  indices 165-170, audited in R12; the shared body is quoted once here.
- No other R11 id has a second-switch row.

### R12 — statue tail, trophy, tendrils, coral, cog, sawmill, crafting stations head: 613, 621, 622, 623, 653, 699, 617, 614, 81, 272, 106, 300, 301, 302, 303 (15 ids, indices 165-179)

C# `TileDrawing.cs:5268-5315`. Verbatim (:5276-5300):

```csharp
		case 617:
			tileTop = 2;
			tileFrameY %= 144;
			tileFrameX %= 54;
			break;
		case 614:
			addFrX = Main.tileFrame[typeCache] * 54;
			addFrY = 0;
			tileTop = 2;
			break;
		case 81:
			tileTop -= 8;
			tileHeight = 26;
			tileWidth = 24;
			if (x % 2 == 0)
			{
				tileSpriteEffect = SpriteEffects.FlipHorizontally;
			}
			break;
		case 272:
			addFrY = 0;
			break;
		case 106:
			addFrY = Main.tileFrame[typeCache] * 54;
			break;
```

| Id | C# body | C# line | Rust arm | Verdict |
| --- | --- | --- | --- | --- |
| 613, 621, 622, 623, 653, 699 | `tileTop = 2;` (statue group tail, R11's body) | :5268-5275 | `drawdata.rs:593-596` | PORTED |
| 617 | top 2; `tileFrameY %= 144; tileFrameX %= 54;` | :5276-5280 | :599-603 (both overrides, Y first like C#) | PORTED |
| 614 | `addFrX = tileFrame * 54; addFrY = 0; tileTop = 2;` | :5281-5285 | :606-610 | PORTED |
| 81 | `top -= 8;` 26 tall, 24 wide, parity flip (Coral) | :5286-5294 | :613-618 | PORTED |
| 272 | `addFrY = 0;` (Cog) | :5295-5297 | :622-624; the `DrawSingleTile` parity `*90` (:1125-1131) rides `apply_world_corrections` → `cog_source_y_offset` :1281-1283 | PORTED (wired :683) |
| 106 | `addFrY = tileFrame * 54;` (Sawmill, NO top change) | :5298-5300 | :626-628 | PORTED |
| 300-303 | `addFrY = tileFrame * 54; tileTop = 2;` (crafting stations) | :5301-5315 | :632-635 (`300..=308 \| 354 \| 355 \| 499`) | PORTED |

Audit notes (R12):
- 617's `%=` mutate the `ref tileFrameX/Y` out-params — the port carries them
  as `frame_*_override` consumed by `resolve_tile_frame`, the same fold as
  case 733 (d64 batch 4). Order Y-then-X kept.
- 614 is the one arm where the animation row rides addFrX (not Y) and addFrY
  is CLEARED — the port matches all three statements in order.
- 272's case body only clears the row; the cog parity shift
  `(tileFrame + x%2 + y%2 + x%3 + y%3) % 2 * 90` is a `DrawSingleTile`
  correction (:1125-1131), ported term-for-term (`cog_source_y_offset`) and
  applied on the flat draw rect — outside the case scope, verified present.
- 106 has NO `tileTop` statement (stays 0) — distinct from the 300-group;
  the port keeps them separate arms.
- No R12 id has a second-switch row.

### R13 — crafting tail, forge, void vault, bubble, mushroom statue, fake chest: 304-308, 354, 355, 499, 377, 463, 464, 491, 379, 349, 441 (15 ids, indices 180-194)

C# `TileDrawing.cs:5305-5356`. Verbatim (:5316-5356):

```csharp
		case 377:
			addFrY = Main.tileFrame[typeCache] * 38;
			tileTop = 2;
			break;
		case 463:
		case 464:
			addFrY = Main.tileFrame[typeCache] * 72;
			tileTop = 2;
			break;
		case 491:
			tileTop = 2;
			addFrX = 54;
			break;
		case 379:
			addFrY = Main.tileFrame[typeCache] * 90;
			break;
		case 349:
		{
			tileTop = 2;
			int num45 = tileFrameX % 36;
			if (Animation.GetTemporaryFrame(y: y - tileFrameY % 54 / 18, x: x - num45 / 18, frameData: out var frameData4))
			{
				tileFrameX = (short)(36 * frameData4 + num45);
			}
			break;
		}
		case 441:
		case 468:
		{
			if (tileFrameY == 18)
			{
				tileHeight = 18;
			}
			int num43 = tileFrameX % 36;
			int num44 = tileFrameY % 38;
			if (Animation.GetTemporaryFrame(x - num43 / 18, y - num44 / 18, out var frameData3))
			{
				tileFrameY = (short)(38 * frameData3 + num44);
			}
			break;
		}
```

| Id | C# body | C# line | Rust arm | Verdict |
| --- | --- | --- | --- | --- |
| 304-308, 354, 355, 499 | `addFrY = tileFrame * 54; tileTop = 2;` (R12's crafting body) | :5305-5315 | `drawdata.rs:632-635` | PORTED |
| 377 | `tileFrame * 38`, top 2 (SharpeningStation) | :5316-5319 | :638-641 | PORTED |
| 463 | `tileFrame * 72`, top 2 (DefendersForge) | :5320-5324 | :644-647; glow :6152-6156 → `cases_a.rs:362` (GlowMask 243, rect `(fx, fy+addFrY)`, `(127,127,127,0)`) | PORTED (glow unconsumed) |
| 464 | shares the forge arm (NO glow row) | :5321 | :644 | PORTED |
| 491 | top 2; `addFrX = 54;` (VoidVault) | :5325-5328 | :649-652 | PORTED |
| 379 | `tileFrame * 90` (Bubble, no top change) | :5329-5331 | :654-656 | PORTED |
| 349 | top 2 + the temp-frame X rewrite | :5332-5341 | static top 2 :659-661; full body `cases_b.rs:297-303` around `temporary_frame` (None) | PORTED (registry deviation) |
| 441 | row-18 height gate + the temp-frame Y rewrite | :5342-5356 | static :665-669 (height gate ONLY); glow :6294-6311 → `cases_a.rs:461-470` | DEVIATION (probe scaffolding absent, see note) |

Audit notes (R13):
- **441/468's first-switch temp-frame probe is not scaffolded anywhere.** The
  C# computes `num43 = fx % 36`, `num44 = fy % 38` and rewrites
  `tileFrameY = 38 * frameData + num44` when `Animation.GetTemporaryFrame`
  hits. The port's static arm carries only the height gate; unlike 349 (whose
  full body wraps the always-None `temporary_frame`, `cases_b.rs:297-303`),
  no 441/468 arm exists in `cases_b` — the module doc (:50-52) records
  "441/468's first-switch half is static-only with the registry empty".
  Behaviour TODAY is identical (the registry is unported, so the C# probe
  always takes the false path too); when the temporary-animation registry
  lands, 441/468 need the probe arm added beside 349/593/594. Recorded as the
  R13 residual (draw-side, registry-gated).
- 349's probe origin is sheet-relative with the 54-row grid
  (`y - fy % 54 / 18`, `x - (fx % 36) / 18`) and the rewrite keeps the
  within-block column (`36 * f + num45`) — the port matches term-for-term;
  `temp_animation_cases_match_the_static_fallback` (`cases_b.rs:673`) pins
  the empty-registry equality.
- 463's glow rect adds `addFrY` (the 72-stride animation row) — the port's
  `rect_anim_y` reads the post-first-switch `add_fr_y`, the C# order.
- No other R13 id has a second-switch row (468's glow is audited at its
  index 195 in R14).

### R14 — fake chest tail, lava lamp, lunar station, chimney, balloons, fireplace, crystal group: 468, 390, 412, 36, 406, 452, 455, 454, 453, 456, 405, 12, 31, 96, 639 (15 ids, indices 195-209)

C# `TileDrawing.cs:5343-5440`. Verbatim (:5367-5417):

```csharp
		case 406:
		{
			tileHeight = 16;
			if (tileFrameY % 54 >= 36)
			{
				tileHeight = 18;
			}
			int num39 = Main.tileFrame[typeCache];
			if (tileFrameY >= 108)
			{
				num39 = 6 - tileFrameY / 54;
			}
			else if (tileFrameY >= 54)
			{
				num39 = Main.tileFrame[typeCache] - 1;
			}
			addFrY = num39 * 56;
			addFrY += tileFrameY / 54 * 2;
			break;
		}
		case 452:
		{
			int num34 = Main.tileFrame[typeCache];
			if (tileFrameX >= 54)
			{
				num34 = 0;
			}
			addFrY = num34 * 54;
			break;
		}
		case 455:
		{
			addFrY = 0;
			tileTop = 2;
			int num31 = 1 + Main.tileFrame[typeCache];
			if (!BirthdayParty.PartyIsUp)
			{
				num31 = 0;
			}
			addFrY = num31 * 54;
			break;
		}
		case 454:
			addFrY = Main.tileFrame[typeCache] * 54;
			break;
		case 453:
			addFrY = (Main.tileFrameCounter[typeCache] / 20 + (y - tileFrameY / 18 + x)) % 3 * 54;
			break;
		case 456:
			addFrY = (Main.tileFrameCounter[typeCache] / 20 + (y - tileFrameY / 18 + (x - tileFrameX / 18))) % 4 * 54;
			break;
```

| Id | C# body | C# line | Rust arm | Verdict |
| --- | --- | --- | --- | --- |
| 468 | shares 441's body (R13) | :5343 | `drawdata.rs:665-669`; glow :6295 → `cases_a.rs:461-470` | DEVIATION (R13 probe residual) + glow ported |
| 390 | `tileFrame * 36` (LavaLamp) | :5357-5359 | :671-673 | PORTED |
| 412 | `addFrY = 0; tileTop = 2;` | :5360-5363 | :676-679; glow :6051-6055 → `cases_a.rs:307` (GlowMask 202, rect `(fx, fy+addFrY)`, opaque white) | PORTED (glow unconsumed) |
| 36 | `tileTop = 2;` (Presents) | :5364-5366 | :681-683 | PORTED |
| 406 | chimney: height bands + the 3-way row pick + `*56 + fy/54*2` | :5367-5386 | :686-698 — all gates and the two-step addFrY fold | PORTED |
| 452 | row dropped on `fx >= 54`, `*54` | :5387-5396 | :701-707 | PORTED |
| 455 | party-gated row | :5397-5408 | static :711-714 (addFrY 0, top 2); full body `cases_b.rs:287-294` (`party_is_up` fed) | PORTED (delegated, wired :700) |
| 454 | `tileFrame * 54` (Pigronata) | :5409-5411 | :716-718 | PORTED |
| 453 | counter cycle `%3*54`, x NOT sheet-relative | :5412-5414 | `cases_b.rs:150-152` verbatim | PORTED (delegated, wired) |
| 456 | counter cycle `%4*54`, both axes sheet-relative | :5415-5417 | `cases_b.rs:155-157` | PORTED (delegated, wired) |
| 405 | fireplace: height 16/18 + row drop + `*38` | :5418-5432 | :725-735 | PORTED |
| 12, 31, 96, 639 | `tileFrame * 36` (crystal/orb group) | :5433-5440 | :739-741 (`12 \| 31 \| 96 \| 639 \| 665 \| 696`) | PORTED |

Audit notes (R14):
- 406's row pick is the trickiest arithmetic in this stretch: `fy >= 108`
  REVERSES the row (`6 - fy/54`), `54 <= fy < 108` steps back one
  (`tileFrame - 1`), else the raw row; then `*56` plus the `fy/54*2` padding
  walk. The port keeps the if/else-if order and folds both adds into one
  expression — same value.
- 453 vs 456: 453's cycle uses the RAW `x` (`y - fy/18 + x`) while 456
  subtracts the frame column from BOTH axes — the port carries the asymmetry
  (tests `cases_b.rs:490-503` pin both).
- 455's C# `addFrY = 0` then reassignment is dead-store-folded in the port's
  static arm; the live `cases_b` body recomputes `num31` exactly (party down →
  row 0, up → `(1 + tileFrame) * 54`).
- 412's glow is UNCONDITIONAL (no row gate) with alpha 255 — distinct from
  the alpha-0 martian/meteor glows; the port's `[255,255,255,255]` matches.
- No other R14 id has a second-switch row.

### R15 — bulb, volcanoes, brazier, dye stations, machines, teleporter: 665, 696, 238, 593, 594, 592, 228, 231, 243, 247, 244, 565, 235, 217, 218 (15 ids, indices 210-224)

C# `TileDrawing.cs:5437-5528`. Verbatim (:5445-5490):

```csharp
		case 593:
		{
			if (tileFrameX >= 18)
			{
				addFrX = -18;
			}
			tileTop = 2;
			if (Animation.GetTemporaryFrame(x, y, out var frameData2))
			{
				addFrY = (short)(18 * frameData2);
			}
			else if (tileFrameX < 18)
			{
				addFrY = Main.tileFrame[typeCache] * 18;
			}
			else
			{
				addFrY = 0;
			}
			break;
		}
		case 594:
		{
			if (tileFrameX >= 36)
			{
				addFrX = -36;
			}
			tileTop = 2;
			int num20 = tileFrameX % 36;
			if (Animation.GetTemporaryFrame(y: y - tileFrameY % 36 / 18, x: x - num20 / 18, frameData: out var frameData))
			{
				addFrY = (short)(36 * frameData);
			}
			else if (tileFrameX < 36)
			{
				addFrY = Main.tileFrame[typeCache] * 36;
			}
			else
			{
				addFrY = 0;
			}
			break;
		}
		case 592:
			addFrY = Main.tileFrame[typeCache] * 54;
			break;
```

| Id | C# body | C# line | Rust arm | Verdict |
| --- | --- | --- | --- | --- |
| 665, 696 | `tileFrame * 36` (crystal group tail, R14) | :5437-5440 | `drawdata.rs:739-741` | PORTED |
| 238 | top 2 + `tileFrame * 36` (Plantera bulb) | :5441-5444 | :744-747 | PORTED |
| 593 | −18 column fold + temp-frame probe + 18-stride fallback | :5445-5465 | static :751-757; full body `cases_b.rs:305-317` | PORTED (registry deviation) |
| 594 | −36 fold + sheet-relative probe + 36-stride fallback | :5466-5487 | static :760-766; full body `cases_b.rs:319-332` | PORTED (registry deviation) |
| 592 | `tileFrame * 54` (BrazierSuspended) | :5488-5490 | :769-771 | PORTED |
| 228, 231, 243, 247 | top 2 + `tileFrame * 54` | :5491-5497 | :774-777 | PORTED |
| 244 | top 2; `tileFrame * 36` iff `fx < 54` else 0 | :5498-5508 | :780-783 | PORTED |
| 565 | top 2; `tileFrame * 36` iff `fx < 36` else 0 | :5509-5519 | :786-789 | PORTED |
| 235 | `tileFrame * 18` (Teleporter) | :5520-5522 | :791-793 | PORTED |
| 217, 218 | `tileFrame * 36`, top 2 | :5523-5528 | :796-799 | PORTED |

Audit notes (R15):
- 593/594's static arms carry the C# `else`-path fallbacks exactly
  (`fx < stride → tileFrame * stride`, else 0); the full probe bodies live in
  `cases_b` with `temporary_frame` = None (the registry deviation, R13's
  note). 593 probes the RAW cell `(x, y)`; 594 the sheet-relative origin —
  both shapes ported. The `(short)` casts on the probe hits are the port's
  `as i16`.
- **564 cross-reference (index 225, R16):** the C# shares the 217/218 label
  but the second switch (:6105-6113) unconditionally clears `addFrY = 0`
  AFTER building the glow rect with the pre-clear value. The port folds the
  net result into the static arm (`drawdata.rs:803-806`, top 2 + row 0) and
  the glow pass deliberately does NOT re-apply the clear
  (`cases_a.rs:49-50/:274-275/:334-342`). Consequence: the port's 564 glow
  rect uses `fy + 0` where the C# uses `fy + tileFrame*36` — a rect deviation
  on the (unconsumed) glow record, documented here for the TILE-DRAW-2 lane;
  the drawn tile pixels are identical.
- No other R15 id has a second-switch row.

### R16 — plasma lamp, extractinators, bottle parity chains, plates, big-cage head: 564, 219, 220, 642, 270, 271, 581, 660, 572, 428, 442, 275, 276, 277, 278 (15 ids, indices 225-239)

C# `TileDrawing.cs:5525-5711`. Verbatim (:5535-5563, the mod-6 chain):

```csharp
		case 270:
		case 271:
		case 581:
		{
			int num14 = Main.tileFrame[typeCache] + x % 6;
			if (x % 2 == 0)
			{
				num14 += 3;
			}
			if (x % 3 == 0)
			{
				num14 += 3;
			}
			if (x % 4 == 0)
			{
				num14 += 3;
			}
			while (num14 > 5)
			{
				num14 -= 6;
			}
			addFrX = num14 * 18;
			addFrY = 0;
			if (x % 2 == 0)
			{
				tileSpriteEffect = SpriteEffects.FlipHorizontally;
			}
			break;
		}
```

The big-cage group head verbatim (:5670-5673):

```csharp
		{
			tileTop = 2;
			Main.critterCage = true;
			int bigAnimalCageFrame = this.GetBigAnimalCageFrame(x, y, tileFrameX, tileFrameY);
```

| Id | C# body | C# line | Rust arm | Verdict |
| --- | --- | --- | --- | --- |
| 564 | shares 217/218's label; second switch clears addFrY + glow | :5525, :6105-6113 | `drawdata.rs:803-806` (net fold); glow `cases_a.rs:337-342` | PORTED (rect deviation, R15 note) |
| 219, 220, 642 | `tileFrame * 54`, top 2 (Extractinator family) | :5529-5534 | :809-812 | PORTED |
| 270, 271, 581 | mod-6 parity chain, `addFrX = n*18`, addFrY 0, flip | :5535-5563 | :816-833 — all three `+3` gates and the `while > 5` unwind | PORTED |
| 660 | mod-5 chain (`x % 5`, `while > 4`, `-= 5`) | :5564-5590 | :836-853 | PORTED |
| 572 | mod-4 chain, NO parity gates (for-loop unwind) | :5591-5604 | :856-864 | PORTED |
| 428 | `tileTop += 4;` + pressed-plate `addFrX += 18` | :5605-5611 | static :868-870; `cases_b.rs:280-284` (`wiring.plate_is_pressed`) | PORTED (delegated, wired :700) |
| 442 | 20×20; inner switch `fx/22`: 1→top −4, 2→top −2 + width 24, 3→top −2 | :5612-5628 | :873-885 | PORTED |
| 275 | big cage: bunny table `*54` | :5676-5686 | `drawdata_furniture.rs:117` (`275 \| 359 \| 599..=605`) | PORTED-UNWIRED |
| 276 | squirrel table `*54` | :5694-5705 | :120 (`276 \| 413 \| 414 \| 606..=612`) | PORTED-UNWIRED |
| 277 | mallard `*54` | :5706-5708 | :121 | PORTED-UNWIRED |
| 278 | duck `*54` | :5709-5711 | :122 | PORTED-UNWIRED |

Audit notes (R16):
- The three bottle chains differ ONLY in modulus and gates: 270/271/581 use
  `%6` with the three `+3` parity gates; 660 uses `%5` with the same gates;
  572 uses `%4` with NO gates (the C# for-loop `for (n = tf + x%4; n > 3;
  n -= 4)` is the port's `while`). All keep `addFrY = 0` + parity flip. The
  `while` unwind equals `%` for non-negative values; `tileFrame` and `x % k`
  are non-negative in range, and the port keeps the loop shape verbatim.
- 428's `PressurePlateHelper.PressurePlatesPressed.ContainsKey(Point(x,y))`
  is the wiring state — `cases_b` reads `wiring.plate_is_pressed(x, y)`
  (test `case_428_pressed_plate_shifts_a_column` pins press/release).
- Big-cage probe verbatim (:6542-6547): `num / 6 * (num2 / 4) %
  Main.cageFrames` — `big_animal_cage_frame` (`drawdata_furniture.rs:99-103`)
  term-for-term. The per-animal arms are faithful in `cage_table` but
  UNWIRED for the same reason as d64 batches 2-3: `furniture_post_pass` has
  zero live callers, the static fallback (`drawdata.rs:890-910`) sets top 2
  only, cages frozen at row 0 (pixel-identical to C# t=0). Wiring belongs to
  W4-d57 with `AnimateTiles_CritterCages` (`Main.cs:13511`).
- No R16 id has a second-switch row other than 564 (audited above).

### Batch 2 — small-animal cage tail: 298, 299, 310, 339, 361-364, 391-394, 532, 533, 538, 544, 555, 556, 582, 619, 629 (21 ids)

C# group `TileDrawing.cs:5756-5831` (labels 285/286 at :5756-5757 fall in d63's quarter;
the shared body is cited once, verbatim):

```csharp
		case 285:
		case 286:
		case 298:
		case 299:
		case 310:
		case 339:
		case 361:
		case 362:
		case 363:
		case 364:
		case 391:
		case 392:
		case 393:
		case 394:
		case 532:
		case 533:
		case 538:
		case 544:
		case 555:
		case 556:
		case 582:
		case 619:
		case 629:
		{
			tileTop = 2;
			Main.critterCage = true;
			int smallAnimalCageFrame2 = this.GetSmallAnimalCageFrame(x, y, tileFrameX, tileFrameY);
			switch (typeCache)
			{
```

The probe, verbatim (`TileDrawing.cs:6535-6540`):

```csharp
	private int GetSmallAnimalCageFrame(int x, int y, int tileFrameX, int tileFrameY)
	{
		int num = x - tileFrameX / 18;
		int num2 = y - tileFrameY / 18;
		return num / 3 * (num2 / 3) % Main.cageFrames;
	}
```

Per-id inner arms (verbatim, `TileDrawing.cs:5783-5830`) vs the port
(`drawdata_furniture.rs::cage_table` :134-144, stride 36, SMALL probe;
`cage_draw_data` :169-201 applies `addFrY = frame * stride`, `top = 2`):

| Id | C# inner arm (verbatim) | C# line | Rust arm | Verdict |
| --- | --- | --- | --- | --- |
| 298 | `case 298:` `case 361:` → `addFrY = Main.frogCageFrame[smallAnimalCageFrame2] * 36;` | :5792-5795 | `298 \| 361 => (CageAnim::Frog, 0, 36, SMALL)` :136 | PORTED-UNWIRED |
| 299 | `case 299:` `case 363:` → `addFrY = Main.mouseCageFrame[smallAnimalCageFrame2] * 36;` | :5800-5803 | `299 \| 363 => (CageAnim::Mouse, 0, 36, SMALL)` :138 | PORTED-UNWIRED |
| 310 | `case 310:` `case 364:` `case 391:` `case 619:` → `addFrY = Main.wormCageFrame[smallAnimalCageFrame2] * 36;` | :5804-5809 | `310 \| 364 \| 391 \| 619 => (CageAnim::Worm, 0, 36, SMALL)` :139 | PORTED-UNWIRED |
| 339 | `case 339:` `case 362:` → `addFrY = Main.grasshopperCageFrame[smallAnimalCageFrame2] * 36;` | :5796-5799 | `339 \| 362 => (CageAnim::Grasshopper, 0, 36, SMALL)` :137 | PORTED-UNWIRED |
| 361 | shares 298's frog arm | :5793 | :136 | PORTED-UNWIRED |
| 362 | shares 339's grasshopper arm | :5797 | :137 | PORTED-UNWIRED |
| 363 | shares 299's mouse arm | :5801 | :138 | PORTED-UNWIRED |
| 364 | shares 310's worm arm | :5805 | :139 | PORTED-UNWIRED |
| 391 | shares 310's worm arm | :5806 | :139 | PORTED-UNWIRED |
| 392 | `case 392:` `case 393:` `case 394:` → `addFrY = Main.slugCageFrame[typeCache - 392, smallAnimalCageFrame2] * 36;` | :5810-5814 | `392..=394 => (CageAnim::Slug, kind as i32 - 392, 36, SMALL)` :140 | PORTED-UNWIRED |
| 393 | shares the slug arm, row 1 | :5811 | :140 | PORTED-UNWIRED |
| 394 | shares the slug arm, row 2 | :5812 | :140 | PORTED-UNWIRED |
| 532 | `case 532:` → `addFrY = Main.maggotCageFrame[smallAnimalCageFrame2] * 36;` | :5815-5817 | `532 => (CageAnim::Maggot, 0, 36, SMALL)` :141 | PORTED-UNWIRED |
| 533 | `case 533:` → `addFrY = Main.ratCageFrame[smallAnimalCageFrame2] * 36;` | :5818-5820 | `533 => (CageAnim::Rat, 0, 36, SMALL)` :142 | PORTED-UNWIRED |
| 538 | `case 538:` `case 544:` `case 629:` → `addFrY = Main.ladybugCageFrame[smallAnimalCageFrame2] * 36;` | :5821-5825 | `538 \| 544 \| 629 => (CageAnim::Ladybug, 0, 36, SMALL)` :143 | PORTED-UNWIRED |
| 544 | shares the ladybug arm | :5822 | :143 | PORTED-UNWIRED |
| 555 | `case 555:` `case 556:` → `addFrY = Main.waterStriderCageFrame[smallAnimalCageFrame2] * 36;` | :5826-5829 | `555 \| 556 => (CageAnim::WaterStrider, 0, 36, SMALL)` :144 | PORTED-UNWIRED |
| 556 | shares the water-strider arm | :5827 | :144 | PORTED-UNWIRED |
| 582 | `case 286:` `case 582:` → `addFrY = Main.snail2CageFrame[smallAnimalCageFrame2] * 36;` | :5788-5791 | `286 \| 582 => (CageAnim::Snail2, 0, 36, SMALL)` :135 | PORTED-UNWIRED |
| 619 | shares 310's worm arm | :5807 | :139 | PORTED-UNWIRED |
| 629 | shares the ladybug arm | :5823 | :143 | PORTED-UNWIRED |

Probe port: `small_animal_cage_frame` (`drawdata_furniture.rs:91-95`) is term-for-term
`num / 3 * (num2 / 3) % cage_frames` with `CAGE_FRAMES = 25` (:32, `Main.cs:66901`
`Main.cageFrames = 25;`); C# truncating division and sign-keeping `%` match Rust i32.

**Why UNWIRED, and what the pixels do today:** the live draw path
(`render/tiles.rs:671-734`) runs `draw_data` → `apply_world_corrections` → `cases_b` →
`cases_a`; **`drawdata_furniture::furniture_post_pass` (:262) has zero callers outside its
own tests** (grep-verified across `crates/`). The static fallback arm in
`drawdata.rs:912-932` sets only `top = 2` and leaves `add_fr_y = tile_frame * 38`. Cage
kinds are absent from `Main.AnimateTiles` (`Main.cs:18516-19304` — they animate through
`AnimateTiles_CritterCages`, `Main.cs:13511`, which moves the `*CageFrame` tables, not
`tileFrame`), so `tile_frame ≡ 0` for them and the fallback draws row 0 — pixel-identical
to the C# t=0 state (tables zero-initialised, `Main.cs:66902-66972`) but **frozen**: the
critter never animates. The `Main.critterCage = true` side effect (:5781) gates that
animator and is likewise unported (documented in `drawdata_furniture.rs:167-168`).
Wiring belongs to the furniture lane (W4-d57); the seam is a `furniture_post_pass` call
after `cases_a` with a `frame_of` backed by a ported `AnimateTiles_CritterCages`.
Tests already pin the rewrite: `drawdata_furniture.rs` :319/:487 (per-kind sweep),
:559/:563 (post-pass).

### Batch 3 — water-animal + jellyfish cages: 282, 505, 543, 598, 568-570, 288-295, 360, 580, 620, 521-527, 316-318 (29 ids)

C# groups `TileDrawing.cs:5833-5907`. The water probe, verbatim (:6528-6533):

```csharp
	private int GetWaterAnimalCageFrame(int x, int y, int tileFrameX, int tileFrameY)
	{
		int num = x - tileFrameX / 18;
		int num2 = y - tileFrameY / 18;
		return num / 2 * (num2 / 3) % Main.cageFrames;
	}
```

Port: `water_animal_cage_frame` (`drawdata_furniture.rs:83-87`), term-for-term.

| Id(s) | C# body (verbatim key lines) | C# line | Rust arm (`cage_table`) | Verdict |
| --- | --- | --- | --- | --- |
| 282, 505, 543 | `tileTop = 2;` `Main.critterCage = true;` `addFrY = Main.fishBowlFrame[waterAnimalCageFrame5] * 36;` | :5837-5840 | `282 \| 505 \| 543 => (CageAnim::FishBowl, 0, 36, WATER)` :146 | PORTED-UNWIRED |
| 598 | `addFrY = Main.lavaFishBowlFrame[waterAnimalCageFrame4] * 36;` | :5845-5848 | `598 => (CageAnim::LavaFishBowl, 0, 36, WATER)` :147 | PORTED-UNWIRED |
| 568, 569, 570 | `addFrY = Main.fairyJarFrame[waterAnimalCageFrame3] * 36;` | :5855-5858 | `568..=570 => (CageAnim::FairyJar, 0, 36, WATER)` :148 | PORTED-UNWIRED |
| 288-295 | `int num8 = typeCache - 288;` … `addFrY = Main.butterflyCageFrame[num8, waterAnimalCageFrame2] * 36;` | :5873-5881 | `288..=295 => (CageAnim::Butterfly, kind as i32 - 288, 36, WATER)` :150 | PORTED-UNWIRED |
| 360, 580, 620 | `if (typeCache == 360 \|\| typeCache == 580 \|\| typeCache == 620)` `{ num8 = 8; }` (same butterfly body) | :5877-5880 | `360 \| 580 \| 620 => (CageAnim::Butterfly, 8, 36, WATER)` :151 | PORTED-UNWIRED |
| 521-527 | `addFrY = Main.dragonflyJarFrame[typeCache - 521, waterAnimalCageFrame] * 36;` | :5892-5895 | `521..=527 => (CageAnim::Dragonfly, kind as i32 - 521, 36, WATER)` :152 | PORTED-UNWIRED |
| 316-318 | `int smallAnimalCageFrame = this.GetSmallAnimalCageFrame(x, y, tileFrameX, tileFrameY);` `addFrY = Main.jellyfishCageFrame[typeCache - 316, smallAnimalCageFrame] * 36;` | :5902-5905 | `316..=318 => (CageAnim::Jellyfish, kind as i32 - 316, 36, SMALL)` :154 | PORTED-UNWIRED |

The butterfly verbatim (:5873-5882):

```csharp
			tileTop = 2;
			Main.critterCage = true;
			int waterAnimalCageFrame2 = this.GetWaterAnimalCageFrame(x, y, tileFrameX, tileFrameY);
			int num8 = typeCache - 288;
			if (typeCache == 360 || typeCache == 580 || typeCache == 620)
			{
				num8 = 8;
			}
			addFrY = Main.butterflyCageFrame[num8, waterAnimalCageFrame2] * 36;
```

Note the jellyfish group reads the **SMALL** probe (`/3 * /3`), not the water probe —
the port carries that (`SMALL` flag, :154). The 360/580/620 → row 8 override is the
"jarred butterfly variants share the eighth row" quirk, ported as the constant row.

Second-switch glow rows for the same ids (audited here since the first-switch index
falls in d64): 568/569/570 → `GlowMask[268/269/270]`, `Color.White`, rect
`(tileFrameX, tileFrameY + addFrY, w, h)` (:6085-6099) — ported in
`drawdata_cases_a.rs:329-331` (`rect_anim_y` = `fy + add_fr_y`); 580 →
`GlowMask[289]`, `new Color(225, 110, 110, 0)` (:6100-6104) — ported :333. These glow
records are produced but **consumed by no pass** (`tiles.rs:731` binds the result to
`_glow`; TILE-DRAW-2 owns the bind) — a draw-side deviation, not a case gap.

UNWIRED rationale identical to batch 2: static fallback `drawdata.rs:933-943`
(`282 | 505 | 543 | 598 | 568..=570 | 288..=295 | 360 | 580 | 620 | 521..=527` and
`316 | 317 | 318`, top 2 only); `furniture_post_pass` uncalled; frozen row 0.

### Batch 4 — fountain + monoliths: 207, 410, 480, 509, 657, 720, 721, 725, 658, 733 (10 ids)

All static+animation arms live in `drawdata.rs` and are wired (the animator feeds
`tile_frame` via `tiles.rs:661-678`, S1-G-1f).

**207 WaterFountain** — C# verbatim (:5908-5929):

```csharp
		case 207:
			tileTop = 2;
			if (tileFrameY >= 72)
			{
				addFrY = Main.tileFrame[typeCache];
				int num2 = x;
				if (tileFrameX % 36 != 0)
				{
					num2--;
				}
				addFrY += num2 % 6;
				if (addFrY >= 6)
				{
					addFrY -= 6;
				}
				addFrY *= 72;
			}
			else
			{
				addFrY = 0;
			}
			break;
```

Port `drawdata.rs:947-963`: same order — `n = tile_frame`, `n2 = x` minus 1 when
`fx % 36 != 0`, `n += n2 % 6`, wrap `>= 6`, `* 72`; else 0; `top = 2`. **PORTED.**

**410 LunarMonolith** — C# verbatim (:5930-5944): `if (tileFrameY == 36) tileHeight = 18;`
then `if (tileFrameY >= 56) { addFrY = Main.tileFrame[typeCache]; addFrY *= 56; } else
addFrY = 0;`. Port `drawdata.rs:966-975` — identical gates and stride. **PORTED.**

**480/509/657/720/721/725 monolith group** — C# verbatim (:5945-5961):

```csharp
		case 480:
		case 509:
		case 657:
		case 720:
		case 721:
		case 725:
			tileTop = 2;
			if (tileFrameY >= 54)
			{
				addFrY = Main.tileFrame[typeCache];
				addFrY *= 54;
			}
			else
			{
				addFrY = 0;
			}
			break;
```

Port `drawdata.rs:979-986` (`480 | 509 | 657 | 720 | 721 | 725`). **PORTED.** The 657
second-switch glow (`if (tileFrameY >= 54)` → `GlowMask[330]`, `Color.White`,
:6056-6063) is in `cases_a.rs:309-313` — produced, unconsumed (`_glow`).

**658 ShimmerMonolith** — C# verbatim (:5962-5981): `tileTop = 2;` then
`switch (tileFrameY / 54)`: default `addFrY = Main.tileFrame[typeCache] * 54;`,
case 1 adds `+ 486`, case 2 adds `+ 972`. Port `drawdata.rs:989-996` —
`match fy / 54 { 1 => …+486, 2 => …+972, _ => … }`. **PORTED.**

**733 RadioThingMonolith** — C# verbatim (:5982-5990):

```csharp
		case 733:
			tileTop = 2;
			if (tileFrameY < 54)
			{
				addFrX += 54;
			}
			tileFrameX %= 54;
			tileFrameY %= 54;
			break;
```

Port `drawdata.rs:1000-1007`: `top = 2`, `add_fr_x += 54` below row 54, then
`frame_x_override = Some(fx % 54)` / `frame_y_override = Some(fy % 54)` — the C# mutates
the `ref tileFrameX/Y` out-params, the port carries the same fold as overrides consumed
by `resolve_tile_frame`. **PORTED.**

### Batch 5 — fall blocks/conveyors, echo, starry glass, living fire, benches, campfire: 326-329, 345, 351, 421, 422, 458, 459, 708, 541, 507, 508, 336, 340-344, 89, 102, 753, 215 (23 ids)

**326/327/328/329/345/351/421/422/458/459/708** — C# verbatim (:5991-6003):
`addFrY = Main.tileFrame[typeCache] * 90;` (no top change). Port `drawdata.rs:1011-1013`
(`326 | 327 | 328 | 329 | 345 | 351 | 421 | 422 | 458 | 459 | 708`). **PORTED.**
(422's animator counts DOWN to 3 — `drawdata_anim.rs`, S1-F-9b — upstream of this arm.)

**541 EchoBlock** — C# verbatim (:6004-6006):
`addFrY = ((!this._shouldShowInvisibleBlocks) ? 90 : 0);`
Port `cases_b.rs:166-168`: `if !env.inputs.show_invisible_blocks { 90 } else { 0 }`.
The flag rides `Main.ShouldShowInvisibleBlocksAndWalls` (`TileDrawing.cs:379` →
`Main.cs:58023`), fed at `tiles.rs:356` from `frame/draw.rs:392`. **PORTED (wired).**

**507/508 StarryGlassBlocks** — C# verbatim (:6007-6013):

```csharp
		case 507:
		case 508:
		{
			int num = 20;
			addFrY = 90 * ((Main.tileFrameCounter[typeCache] + x * 11 + y * 27) % (num * 8) / num);
			break;
		}
```

Port `cases_b.rs:160-163`: `90 * ((counter + x * 11 + y * 27) % (num * 8) / num)` with
`num = 20`; `counter` is `tile_anim.tile_frame_counter[kind]` (`tiles.rs:666-670`).
**PORTED (wired).** Note the C# reads `tileFrameCounter`, NOT `tileFrame` — the port
passes the counter, matching.

**336/340/341/342/343/344 LivingFire family** — C# verbatim (:6014-6022):
`addFrY = Main.tileFrame[typeCache] * 90;` `tileTop = 2;`. Port `drawdata.rs:1019-1022`.
**PORTED.** (340..344's animator loop is `drawdata_anim.rs`.)

**89/102/753 Benches/Thrones/GiantTiki** — C# verbatim (:6023-6031): three one-statement
arms, each `tileTop = 2; break;`. Port `drawdata.rs:1025-1027` (`89 | 102 | 753`).
**PORTED.** The 89 second-switch stride+glow is batch 6's audit.

**215 campfire epilogue** — C# verbatim (:6033-6044), runs AFTER the switch for
`TileID.Sets.Campfires` (`TileID.cs:616`: `Sets.Campfires = Sets.Factory.CreateBoolSet(215);`):

```csharp
		if (TileID.Sets.Campfires[tileCache.type])
		{
			if (tileFrameY < 36)
			{
				addFrY = Main.tileFrame[typeCache] * 36;
			}
			else
			{
				addFrY = 252;
			}
			tileTop = 2;
		}
```

Port `drawdata.rs:1036-1043`: `if kind == 215` — `add_fr_y = tile_frame * 36` below row
36 else 252, `top = 2`, positioned after the match exactly like the C#. **PORTED.**

### Batch 6 — second-switch (glow) cases: 19, 34, 10, 11, 88, 42, 87, 93, 101, 104 (+ 89's glow half) (10 ids)

These ten ids appear ONLY in the second `switch (typeCache)` (`TileDrawing.cs:6049-6510`,
runs after the halfBrick epilogue :6045-6048); the first switch has no arm for them, so
the base metrics stay the 16×16 default — `drawdata.rs:1028-1029` documents exactly that
("19 (platforms), 25, 40, 50, 51, 75 and 76 have no branch in GetTileDrawData"). The port
delegates the whole second switch to `drawdata_cases_a.rs::glow_switch` (:277-614),
called via `apply_cases_a` (:625) at `tiles.rs:731` AFTER `draw_data` +
`apply_world_corrections` + `cases_b` — the C# order. The glow colours, verbatim:
`_meteorGlow = new Color(100, 100, 100, 0)` (:130) → `METEOR_GLOW` (`cases_a.rs:112`);
`_martianGlow` recomputed per frame `byte b = (byte)(100f + 150f * Main.martianLight);
new Color(b, b, b, 0)` (:498-499) → `martian_glow` (:107-109, truncating cast).

| Id | C# (verbatim key lines) | C# line | Rust (`glow_switch`) | Verdict |
| --- | --- | --- | --- | --- |
| 19 | `int num46 = tileFrameY / 18;` row 26 → `GlowMask[65]`, rect `(tileFrameX, tileFrameY % 18, w, h)`, `_martianGlow`; row 27 → `GlowMask[112]`, `_meteorGlow` | :6157-6173 | :365-374 (`mask: 65`/`112`, `rect_plain(fx, fy % 18, height)`) | PORTED (glow unconsumed) |
| 34 | `if (tileFrameX / 54 == 0 && tileFrameY / 54 == 33)` → `GlowMask[55]`, rect `(fx, fy % 54, w, h)`, `_martianGlow` | :6268-6275 | :444-448 | PORTED (glow unconsumed) |
| 10 | `if (tileFrameX < 54 && num46 == 32)` (`num46 = fy / 54`) → `GlowMask[57]`, `_martianGlow` | :6312-6322 | :472-477 | PORTED (glow unconsumed) |
| 11 | `fx < 54` gate; row 32 → `GlowMask[58]` martian, row 33 → `GlowMask[119]` meteor | :6323-6342 | :479-490 | PORTED (glow unconsumed) |
| 88 | `addFrX -= 1998 * num55; addFrY += 36 * num55;` (`num55 = fx / 1998`); col 24 → `GlowMask[59]` martian, col 25 → `GlowMask[120]` meteor, rect `(fx % 54, fy, w, h)` | :6343-6362 | :492-504 (`add_fr_x_delta`/`add_fr_y_delta`) | PORTED (glow unconsumed) |
| 42 | `addFrY -= 2016 * num54; addFrX += 36 * num54;` (`num54 = fy / 2016`); row 33 → `GlowMask[63]`, rect `(fx, fy % 36, w, h)` | :6363-6376 | :507-515 | PORTED (glow unconsumed) |
| 87 | 1998 stride unwind (as 88); col 26 → `GlowMask[64]` martian, col 27 → `GlowMask[121]` meteor | :6377-6396 | :517-529 | PORTED (glow unconsumed) |
| 93 | `addFrY -= 1998 * num51; addFrX += 36 * num51;` (`num51 = fy / 1998`); **`tileTop += 2;`** row 27 → `GlowMask[62]`, rect `(fx, fy % 54, w, h)` | :6414-6428 | :544-553 (`top_delta = 2`) | PORTED (glow unconsumed) |
| 101 | `addFrX -= 1998 * num50; addFrY += 72 * num50;` (**72** row step, not 36); col 28 → `GlowMask[60]` martian, col 29 → `GlowMask[115]` meteor | :6446-6465 | :567-579 | PORTED (glow unconsumed) |
| 104 | `addFrX -= 2016 * num48; addFrY += 90 * num48;` (**90** row step); **`tileTop = 2;`** col 24 → `GlowMask[51]` martian, col 25 → `GlowMask[118]` meteor, rect `(fx % 36, fy, w, h)` | :6466-6486 | :582-595 (`top_override = Some(2)`) | PORTED (glow unconsumed) |
| 89 (glow half) | `addFrX -= 1998 * num49; addFrY += 36 * num49;` col 29 → `GlowMask[66]` martian, col 30 → `GlowMask[123]` meteor | :6208-6227 | :400-412 | PORTED (glow unconsumed) |

Verbatim exemplar — case 104, the trickiest (stride + top override), :6466-6486:

```csharp
		case 104:
		{
			int num46 = tileFrameX / 36;
			int num48 = tileFrameX / 2016;
			addFrX -= 2016 * num48;
			addFrY += 90 * num48;
			tileTop = 2;
			if (num46 == 24)
			{
				glowTexture = TextureAssets.GlowMask[51].Value;
				glowSourceRect = new Rectangle(tileFrameX % 36, tileFrameY, tileWidth, tileHeight);
				glowColor = this._martianGlow;
			}
			if (num46 == 25)
			{
				glowTexture = TextureAssets.GlowMask[118].Value;
				glowSourceRect = new Rectangle(tileFrameX % 36, tileFrameY, tileWidth, tileHeight);
				glowColor = this._meteorGlow;
			}
			break;
		}
```

Audit notes:
- The C# glow gate reads the RAW `tileFrameX / 54` (or `/36`) BEFORE the stride unwind
  mutates `addFr*` — the port computes `num46` from raw `fx`/`fy` likewise
  (e.g. :401/:493/:568), and the rect uses `fx % 54` — faithful.
- The frame/top deltas ARE consumed: `DrawDataCasesA::apply` (:93-100) folds them into
  `TileDrawData` in place, so the martian/meteor furniture styles draw the right sheet
  section today. Only the `glow` record itself is dropped (`tiles.rs:731` `let _glow`),
  pending the TILE-DRAW-2 glow pass. `martian_light` is fed 0.0 (`tiles.rs:732`) —
  `Main.martianLight` is event state the port does not simulate (documented,
  `cases_a.rs:53-54`); meteor glow is the constant (100,100,100,0).
- Platform 19's glow rows are the research/09 deviation-6 "martian conduit rows 26/27"
  — now PORTED as data (this audit), still not drawn (the glow pass). The slope
  side-fill quads (`DrawBasicTile` :1511-1545) are a separate pass, out of DrawData
  case scope (W4-d59 owns them).
