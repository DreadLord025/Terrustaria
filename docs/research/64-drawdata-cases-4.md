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
