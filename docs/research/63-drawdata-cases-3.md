# 63 — DrawData case census, slice 3 (W4-d63)

> Wave 4 · researched 2026-11-20 · branch `task/W4-d63-drawdata-cases-3` @ base a725acb0 ·
> code-only mode (no builds/tests run) · sources: `decompiled/Terraria/GameContent/Drawing/TileDrawing.cs`,
> `crates/tr-core/src/drawdata.rs`, `crates/tr-core/src/drawdata_furniture.rs`,
> `crates/tr-app/src/drawdata_cases_a.rs`, `crates/tr-app/src/drawdata_cases_b.rs`,
> `crates/tr-app/src/render/tiles.rs`.

## Range claim (documented early, per wave protocol)

The four W4 drawdata slices partition the **C#-order union of `GetTileDrawData` case ids**
(first switch `TileDrawing.cs:4540-6032`, then the second/glow switch `:6049-6507` ids not
already in the first; 361 + 10 = **371 distinct ids**). The in-flight sibling d64
(`origin/task/W4-d64-drawdata-cases-4`, `docs/research/64-drawdata-cases-4.md` @ `161c6389`)
claimed the LAST quarter and stated the convention; this doc takes the THIRD quarter:

| Slice | Index range | Ids |
| --- | --- | --- |
| d61 | [0, 93) | 93 |
| d62 | [93, 186) | 93 |
| **d63 (this doc)** | **[186, 279)** | **93** |
| d64 | [279, 371) | 92 + the 215 epilogue |

Boundary cross-check (brace-balanced re-extraction, `tools/_d63_census.py`, depth-1
labels only — the nested cage/monolith sub-switches at :5147/:5674/:5783 are bodies):
index 185 = `354@5310` (last d62), index 186 = `355@5311` (first d63), index 278 =
`286@5757` (last d63), index 279 = `298@5758` (first d64 — matches d64's claimed list
verbatim). The 215 campfire epilogue (`:6033-6044`, not a case label) belongs to d64.

**d63 owns these 93 ids** (C# order; `@` = first-switch label line; all 93 are
first-switch ids — none of the 10 second-switch-only ids fall in this quarter):

```
355@5311 499@5312 377@5316 463@5320 464@5321 491@5325 379@5329 349@5332
441@5342 468@5343 390@5357 412@5360 36@5364 406@5367 452@5387      (batch 2)
455@5397 454@5409 453@5412 456@5415 405@5418 12@5433 31@5434 96@5435
639@5436 665@5437 696@5438 238@5441 593@5445 594@5466 592@5488      (batch 3)
228@5491 231@5492 243@5493 247@5494 244@5498 565@5509 235@5520
217@5523 218@5524 564@5525 219@5529 220@5530 642@5531 270@5535
271@5536                                                          (batch 4)
581@5537 660@5564 572@5591 428@5605 442@5612 275@5629 276@5630
277@5631 278@5632 279@5633 280@5634 281@5635 296@5636 297@5637
309@5638                                                          (batch 5)
358@5639 359@5640 413@5641 414@5642 542@5643 550@5644 551@5645
553@5646 554@5647 558@5648 559@5649 599@5650 600@5651 601@5652
602@5653                                                          (batch 6)
603@5654 604@5655 605@5656 606@5657 607@5658 608@5659 609@5660
610@5661 611@5662 612@5663 632@5664 640@5665 643@5666 644@5667
645@5668                                                          (batch 7)
710@5669 285@5756 286@5757                                        (batch 8)
```

Overlap note: ids 441/468 (and 349's dresser group) appear in BOTH switches; their
first-switch index falls in d63's quarter, so the second-switch chest-glow rows
(`:6110-6156` region) are audited here alongside the first-switch body — d64's doc
explicitly defers "the 21/467/441/468 chest glow rows" to the d62/d63 quarters.
Ids 285/286 share the small-animal cage body with d64's 298..629 tail; the shared
group header is cited once per doc, the inner arms audited by whoever owns the id.

## Headline census result — the "missing cases" premise is stale

d64's brace-balanced census (`161c6389`) shows **371/371 ids covered** across
`drawdata.rs` / `drawdata_cases_a.rs` / `drawdata_cases_b.rs` / `drawdata_furniture.rs`
— 0 missing case arms. This doc independently re-extracted the same 371-id union
(`tools/_d63_census.py` → `tools/_d63_union.txt`) and confirms the count. The d63
quarter's per-case audit below verifies each arm against the C# verbatim and records
what is *actually* still missing on the draw side (static-only deviations, unwired
furniture passes, unconsumed glow records — not case gaps). research/09's
"≈113 of ≈376 case ids" (09-tile-drawing.md:31/111/188) is STALE, predating S1-F-9
waves A-E.

d62's in-flight classification (`TR-W4-d62/tools/_d62_census_out.txt`, untracked)
flags these d63-quarter ids as STATIC (deviation-comment arms in `draw_data`) or
DELEG (delegated pass): 349, 441, 468, 405, 275-281, 296, 297, 309, 358, 359, 413,
414, 542, 550, 551, 553, 554, 558, 559, 599-612, 632, 640, 643-645, 710, 285, 286
(STATIC) and 453, 456 (DELEG → `cases_b`). The audit below re-derives each verdict
from the C# verbatim rather than trusting the comment tags.

## Per-case audit

Filled in per batch below (15 ids per commit). Verdict key: **PORTED** (arm faithful,
wired), **PORTED-UNWIRED** (arm faithful, no live caller), **DEVIATION** (documented
approximation — static part only), **MISSING** (no arm anywhere).

### Batch 2 — crafting stations / statue / fake chests / chimney / balloon machine: 355, 499, 377, 463, 464, 491, 379, 349, 441, 468, 390, 412, 36, 406, 452 (15 ids)

C# `TileDrawing.cs:5311-5396`. The static-station group head, verbatim (:5311-5331 —
labels 300-308/354 at :5301-5310 fall in d62's quarter; the shared body cited once):

```csharp
		case 355:
		case 499:
			addFrY = Main.tileFrame[typeCache] * 54;
			tileTop = 2;
			break;
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
```

| Id | C# body (verbatim key lines) | C# line | Rust arm | Verdict |
| --- | --- | --- | --- | --- |
| 355 | shares the `* 54` + top 2 station body | :5311-5315 | `drawdata.rs:632-635` (`300..=308 \| 354 \| 355 \| 499`) | PORTED |
| 499 | shares the `* 54` + top 2 station body | :5312-5315 | :632-635 | PORTED |
| 377 | `addFrY = Main.tileFrame[typeCache] * 38; tileTop = 2;` | :5316-5319 | `drawdata.rs:638-641` | PORTED |
| 463 | `* 72` + top 2 | :5320-5324 | `drawdata.rs:644-647` (`463 \| 464`) | PORTED |
| 464 | shares 463's arm | :5321 | :644-647 | PORTED |
| 491 | `tileTop = 2; addFrX = 54;` (absolute assign, not `+=`) | :5325-5328 | `drawdata.rs:649-652` (`data.add_fr_x = 54`) | PORTED |
| 379 | `addFrY = Main.tileFrame[typeCache] * 90;` (no top change) | :5329-5331 | `drawdata.rs:654-656` | PORTED |
| 349 | top 2 + `GetTemporaryFrame` tileFrameX rewrite | :5332-5341 | static `drawdata.rs:659-661` + full body `cases_b.rs:297-303` | DEVIATION (probe stub) |
| 441 | height 18 on `tileFrameY == 18` + temp-frame tileFrameY rewrite | :5342-5356 | static `drawdata.rs:665-669`; rewrite unported | DEVIATION (static half) |
| 468 | shares 441's arm | :5343 | :665-669 | DEVIATION (static half) |
| 390 | `addFrY = Main.tileFrame[typeCache] * 36;` | :5357-5359 | `drawdata.rs:671-673` | PORTED |
| 412 | `addFrY = 0; tileTop = 2;` | :5360-5363 | `drawdata.rs:676-679` | PORTED |
| 36 | `tileTop = 2;` | :5364-5366 | `drawdata.rs:681-683` | PORTED |
| 406 | chimney band fold (below) | :5367-5386 | `drawdata.rs:686-698` | PORTED |
| 452 | `num34 = Main.tileFrame[typeCache]; if (tileFrameX >= 54) num34 = 0; addFrY = num34 * 54;` | :5387-5396 | `drawdata.rs:701-707` | PORTED |

All static arms are wired: `render/tiles.rs:671-679` calls `draw_data` with
`tile_anim.animation_row(kind)`; the animator steps these kinds
(`drawdata_anim.rs:350-359/:409-410` — 355 mirrors 354 per `Main.cs:19024`, 377/379/390/
406/452/499/463/464 each `step_ge_ge` with the C# frame counts).

**349 MushroomStatue** — C# verbatim (:5332-5341):

```csharp
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
```

Port `cases_b.rs:297-303` carries the full body — `n45 = fx % 36`, probe
`temporary_frame(x - n45 / 18, y - fy % 54 / 18)` (the C# named args reordered to
positional, same coordinates), `frame_x_override = Some(36 * f + n45)`. The probe
(`cases_b.rs:97-99`) returns `None` — `Animation._temporaryAnimations`
(`Animation.cs:186-196`, verbatim below) is not ported, so the C# `false` path is taken:

```csharp
	public static bool GetTemporaryFrame(int x, int y, out int frameData)
	{
		Point16 key = new Point16(x, y);
		if (!Animation._temporaryAnimations.TryGetValue(key, out var value))
		{
			frameData = 0;
			return false;
		}
		frameData = value._frameData[value._frame];
		return true;
	}
```

Pixel-identical while the registry is empty (statues only animate after a player
activates them); the deviation is documented at `cases_b.rs:47-52`. Wired at
`tiles.rs:700-712`.

**441/468 FakeContainers** — C# verbatim (:5342-5356):

```csharp
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

The static arm (`drawdata.rs:665-669`) ports only the height gate. Unlike 349, the
temp-frame rewrite has NO stubbed body in `cases_b` — the merge dedupe gave 441/468 to
slice A (`cases_b.rs:9-10/:50-52`, `OWNED_KINDS` :343-345 excludes them) and slice A
owns only their SECOND-switch glow rows. With the registry empty the rewrite never
fires in C# either, so today's pixels match; when the temporary-animation registry
lands, 441/468 need the `38 * f + num44` `frame_y_override` added (the 349 pattern at
`cases_b.rs:300-302`). Tests pin the static half: `drawdata.rs:1947-1952`.

Second-switch glow halves (d63 owns these ids' index; d64's doc defers them here):
- **441/468** (:6294-6311): `num46 = tileFrameX / 36`; col 48 → `GlowMask[56]`
  `_martianGlow`, col 49 → `GlowMask[117]` `_meteorGlow`, rect
  `(tileFrameX % 36, tileFrameY, w, h)` — ported `cases_a.rs:461-470`, term-for-term
  (identical to the 21/467 chest rows :6276-6293 → `cases_a.rs:450-459`).
- **463** (:6152-6156): unconditional `GlowMask[243]`, rect
  `(tileFrameX, tileFrameY + addFrY, w, h)`, `new Color(127, 127, 127, 0)` — ported
  `cases_a.rs:362` (`rect_anim_y` = `fy + add_fr_y`).
- **412** (:6051-6055): unconditional `GlowMask[202]`, rect `(fx, fy + addFrY, w, h)`,
  `new Color(255, 255, 255, 255)` (opaque white — the one glow with alpha 255) —
  ported `cases_a.rs:307`.

All three glow records are produced but unconsumed (`tiles.rs:731` binds `_glow`;
TILE-DRAW-2 owns the bind) — draw-side deviation, not a case gap. Martian glow rides
`martian_light = 0.0` (`tiles.rs:732`, unported event state).

**406 Chimney** — C# verbatim (:5367-5386):

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
```

Port `drawdata.rs:686-698`: same gate order (height 16 → 18 on `fy % 54 >= 36`;
`n39` band select `>= 108` first, `>= 54` second), the two `addFrY` statements fused
into `n39 * 56 + fy / 54 * 2` — same value, no intermediate observable. Tests pin all
five bands (`drawdata.rs:2125-2134`). **PORTED.**

### Batch 3 — party monolith / pigronata / balloons / fireplace / crystals / volcanoes: 455, 454, 453, 456, 405, 12, 31, 96, 639, 665, 696, 238, 593, 594, 592 (15 ids)

C# `TileDrawing.cs:5397-5490`.

| Id | C# body (verbatim key lines) | C# line | Rust arm | Verdict |
| --- | --- | --- | --- | --- |
| 455 | `num31 = 1 + Main.tileFrame[typeCache]; if (!BirthdayParty.PartyIsUp) num31 = 0; addFrY = num31 * 54;` + top 2 | :5397-5408 | static `drawdata.rs:711-714` + full body `cases_b.rs:287-294` | PORTED (wired) |
| 454 | `addFrY = Main.tileFrame[typeCache] * 54;` | :5409-5411 | `drawdata.rs:716-718` | PORTED (row frozen — see note) |
| 453 | `addFrY = (Main.tileFrameCounter[typeCache] / 20 + (y - tileFrameY / 18 + x)) % 3 * 54;` | :5412-5414 | `cases_b.rs:150-152` | PORTED (wired) |
| 456 | `addFrY = (Main.tileFrameCounter[typeCache] / 20 + (y - tileFrameY / 18 + (x - tileFrameX / 18))) % 4 * 54;` | :5415-5417 | `cases_b.rs:155-157` | PORTED (wired) |
| 405 | fireplace height gate + column drop (below) | :5418-5432 | `drawdata.rs:725-735` | PORTED |
| 12 | shares the `* 36` crystal/orb group | :5433-5440 | `drawdata.rs:739-741` (`12 \| 31 \| 96 \| 639 \| 665 \| 696`) | PORTED |
| 31 | shares the group | :5434 | :739-741 | PORTED |
| 96 | shares the group | :5435 | :739-741 | PORTED |
| 639 | shares the group | :5436 | :739-741 | PORTED |
| 665 | shares the group | :5437 | :739-741 | PORTED |
| 696 | shares the group | :5438 | :739-741 | PORTED |
| 238 | `tileTop = 2; addFrY = Main.tileFrame[typeCache] * 36;` | :5441-5444 | `drawdata.rs:744-747` | PORTED |
| 593 | volcano small: addFrX -18 / top 2 / temp-frame probe + fallbacks | :5445-5465 | static `drawdata.rs:751-757` + full body `cases_b.rs:304-317` | DEVIATION (probe stub) |
| 594 | volcano large: addFrX -36 / top 2 / probe + fallbacks | :5466-5487 | static `drawdata.rs:760-766` + full body `cases_b.rs:318-332` | DEVIATION (probe stub) |
| 592 | `addFrY = Main.tileFrame[typeCache] * 54;` | :5488-5490 | `drawdata.rs:769-771` | PORTED |

**455 PartyMonolith** — C# verbatim (:5397-5408):

```csharp
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
```

Port `cases_b.rs:287-294` is term-for-term (`n31 = 1 + tile_frame`, zeroed unless
`env.inputs.party_is_up`, `* 54`, top 2). The static arm's comment
(`drawdata.rs:708-710`, "treats as always false") is STALE as a behaviour claim: the
live path runs `cases_b` after the static table (`tiles.rs:700-712`) and
`party_is_up` is fed from the ported event state (`frame/draw.rs:1181` →
`world_events::party_is_up`, `tiles.rs:320/:357`). The animator steps 455
(`drawdata_anim.rs:358`, `Main.cs:19085-19092`). **PORTED, wired.**

**454 Pigronata — frozen row is faithful, not a gap.** The arm reads
`Main.tileFrame[454]`, but a whole-decompilation search finds NO writer of
`tileFrame[454]` outside the zero-init (`Main.cs:8393`): `AnimateTiles` has no 454
block (453/455/456 each have one, `Main.cs:19109-19126`/`:19085-19092`), and no
variable-index write covers it. So C# draws row 0 forever; the port's
`is_animated_kind` gate set (`drawdata_anim.rs:103-203`) correctly excludes 454 and
`animation_row(454) ≡ 0` — pixel-identical. (The pigronata's visible sway in-game is
the tile's own framing, not this array.)

**453/456 balloons** — the counter-driven bodies are ported verbatim in `cases_b`
(above table); `counter` rides `tile_anim.tile_frame_counter[kind]`
(`tiles.rs:666-670`), and the animator replays the C# random drift draw-for-draw
(`drawdata_anim.rs:361-389`, `Main.cs:19109-19113`/`:19118-19126`; the replay test
:725-748 pins both counters against a cloned `Main.rand` stream, including the
`isGeneratingOrLoadingWorld` gate on the `Next(3)` draw). Note 453 uses RAW `x`
(not sheet-relative) while 456 subtracts `tileFrameX / 18` — the port keeps that
asymmetry (`cases_b.rs:151` vs :156). The static table has no arm for either
(`drawdata.rs:719-721` documents the delegation). **PORTED, wired.**

**405 Fireplace** — C# verbatim (:5418-5432):

```csharp
		case 405:
		{
			tileHeight = 16;
			if (tileFrameY > 0)
			{
				tileHeight = 18;
			}
			int num27 = Main.tileFrame[typeCache];
			if (tileFrameX >= 54)
			{
				num27 = 0;
			}
			addFrY = num27 * 38;
			break;
		}
```

Port `drawdata.rs:725-735`: same gate order (`fy > 0` — strictly greater, unlike
406's `% 54 >= 36`), column drop at `fx >= 54`, stride 38. Animator steps 405
(`drawdata_anim.rs:355`, `Main.cs:19061-19068`). **PORTED.**

**12/31/96/639/665/696 group** — C# verbatim (:5433-5440): six labels, one body
`addFrY = Main.tileFrame[typeCache] * 36;` (no top change). Port
`drawdata.rs:739-741`. Animator coverage: 665 mirrors BOTH arrays of 12
(`drawdata_anim.rs:282-283`, `Main.cs:18528-18529`), 696 mirrors 31's frame only
(:290, `Main.cs:18588` — counter stays 0, test :620-631), 639 steps
`step_gt_ge(639, 5, 10)` (:284, `Main.cs:18530-18539`); 12/31/96 step in the
heart/orb/pot blocks. All six in the gate set. **PORTED.**

**593/594 volcanoes** — C# verbatim, 593 (:5445-5465):

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
```

594 (:5466-5487) is the same shape with 36: `addFrX = -36` at `fx >= 36`,
`num20 = tileFrameX % 36`, probe at `(x - num20 / 18, y - tileFrameY % 36 / 18)`
(named args reordered), `36 * frameData`, fallback `tileFrame * 36` below 36 else 0.
Ports: static fallbacks `drawdata.rs:751-757`/:760-766 (the C# `false` path), full
bodies around the stubbed probe `cases_b.rs:304-317`/:319-332 — including the
absolute `add_fr_x = -18/-36` assigns (not `+=`). Same deviation class as 349
(batch 2): the temporary-animation registry is unported, probe returns `None`,
pixel-identical while it is empty. The animator's 593/594 wrap-to-row-5 quirk
(`Main.cs:19240-19257`) is ported and tested (`drawdata_anim.rs:421-422`, test
:690-701). **DEVIATION (probe stub), wired.**
