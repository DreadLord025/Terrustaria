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
