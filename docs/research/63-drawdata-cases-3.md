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

### Batch 4 — dye vat / machines / teleporter / extractinators / firefly bottles: 228, 231, 243, 247, 244, 565, 235, 217, 218, 564, 219, 220, 642, 270, 271 (15 ids)

C# `TileDrawing.cs:5491-5536` (the 270/271 body continues to :5563, shared with
581 — batch 5 audits 581's index, the body is cited here once).

| Id | C# body (verbatim key lines) | C# line | Rust arm | Verdict |
| --- | --- | --- | --- | --- |
| 228 | shares the `tileTop = 2; addFrY = Main.tileFrame[typeCache] * 54;` group | :5491-5497 | `drawdata.rs:774-777` (`228 \| 231 \| 243 \| 247`) | PORTED |
| 231 | shares the group | :5492 | :774-777 | PORTED |
| 243 | shares the group | :5493 | :774-777 | PORTED |
| 247 | shares the group | :5494 | :774-777 | PORTED |
| 244 | top 2; `tileFrame * 36` below fx 54 else 0 | :5498-5508 | `drawdata.rs:780-783` | PORTED |
| 565 | top 2; `tileFrame * 36` below fx 36 else 0 | :5509-5519 | `drawdata.rs:786-789` | PORTED |
| 235 | `addFrY = Main.tileFrame[typeCache] * 18;` | :5520-5522 | `drawdata.rs:791-793` | PORTED |
| 217 | shares `* 36` + top 2 | :5523-5528 | `drawdata.rs:796-799` (`217 \| 218`) | PORTED |
| 218 | shares 217's arm | :5524 | :796-799 | PORTED |
| 564 | shares 217's arm; second switch clears addFrY | :5525 + :6105-6113 | `drawdata.rs:803-806` (folded) + glow `cases_a.rs:337-342` | DEVIATION (glow rect, see below) |
| 219 | shares `* 54` + top 2 | :5529-5534 | `drawdata.rs:809-812` (`219 \| 220 \| 642`) | PORTED |
| 220 | shares 219's arm | :5530 | :809-812 | PORTED |
| 642 | shares 219's arm | :5531 | :809-812 | PORTED |
| 270 | parity-chain mod 6 (below) | :5535-5563 | `drawdata.rs:816-833` (`270 \| 271 \| 581`) | PORTED |
| 271 | shares 270's arm | :5536 | :816-833 | PORTED |

Animator coverage (all wired via `tiles.rs:661-678`): 228 `step_ge_ge(228,5,3)`
(`drawdata_anim.rs:354`, `Main.cs:19053-19060`), 231 `(16,7)` (:301), 243 `(4,6)`
(:304), 247 the `step_gt_ge` idiom (:228 doc, `Main.cs:18739-18748`), 244 `(4,6)`
(:305), 565 `(4,5)` (:433, `Main.cs:19267-19275`), 235 `(20,4)` (:302), 217 `(4,5)`
(:296), 218 `(4,2)` (:297), 564 `(5,36)` (:420, `Main.cs:19231-19239`), 219 `(4,10)`
(:298), 220 `(4,4)` (:300), 642 `(4,6)` (:299), 270 `step_gt_gt(270,8,5)` (:309,
`Main.cs:18769-18778`) with 271/581 mirroring 270's frame (:311-313,
`Main.cs:18779-18781`; test :645-654).

**244/565 machines** — C# verbatim, 244 (:5498-5508):

```csharp
		case 244:
			tileTop = 2;
			if (tileFrameX < 54)
			{
				addFrY = Main.tileFrame[typeCache] * 36;
			}
			else
			{
				addFrY = 0;
			}
			break;
```

565 (:5509-5519) is the same with the gate at 36. Ports `drawdata.rs:780-783` /
:786-789 fold the if/else into one expression — same value. **PORTED.**

**564 PlasmaLamp — the fold is net-correct but shifts the glow rect.** C# first
switch: 564 shares the 217/218 arm (`addFrY = Main.tileFrame[564] * 36`, top 2).
Second switch, verbatim (:6105-6113):

```csharp
		case 564:
			if (tileCache.frameX < 36)
			{
				glowTexture = TextureAssets.GlowMask[267].Value;
				glowSourceRect = new Rectangle(tileFrameX, tileFrameY + addFrY, tileWidth, tileHeight);
				glowColor = new Color(200, 200, 200, 0) * ((float)(int)Main.mouseTextColor / 255f);
			}
			addFrY = 0;
			break;
```

The port folds the trailing `addFrY = 0` into the static arm
(`drawdata.rs:803-806`, documented at `cases_a.rs:274-275`), so the FINAL tile
draw matches C# exactly (top 2, no animation offset). But the C# builds the glow
rect BEFORE the clear — its y is `tileFrameY + tileFrame[564] * 36` — while the
port's `glow_switch` receives the already-zeroed `add_fr_y` and emits
`rect_anim_y(fx, height)` = `fy + 0` (`cases_a.rs:340`). While
`tileFrame[564] != 0` (it cycles 0..35 every 5 ticks) the glow source rect is one
animation row off. Latent today — the glow record is unconsumed (`tiles.rs:731`
`_glow`, TILE-DRAW-2) — but the bind must either un-fold the clear (apply it in
`glow_switch` after the rect, as C# orders it) or pass the pre-clear `addFrY` for
564. The colour port is faithful: `(200 * pulse) as u8` per channel with
`pulse = mouse_text_color / 255` (:304/:339), fed steady 255 (`tiles.rs:732` —
the UI text pulse deviation). The gate reads `tileCache.frameX` (raw stored
frame); the first switch never rewrites `tileFrameX` for 564, so the port's `fx`
is the same value. **DEVIATION (glow rect y), final draw PORTED.**

**270/271 Firefly/Lavafly bottles** — C# verbatim (:5535-5563, shared with 581):

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

Port `drawdata.rs:816-833`: the three `+3` parity gates in the same order, the
`while > 5` unwind (not `% 6` — C# keeps the loop, so does the port), absolute
`add_fr_x = n14 * 18`, `add_fr_y = 0`, and `data.flip = flip_x` with
`flip_x = x % 2 == 0` (:128) — the C# only SETS the flip on even x and leaves the
default `None` otherwise, which is the same boolean. **PORTED.** (581's own index
is batch 5; the arm is shared.)

### Batch 5 — shimmerfly/soul bottles / pressure plates / big-cage group head: 581, 660, 572, 428, 442, 275, 276, 277, 278, 279, 280, 281, 296, 297, 309 (15 ids)

C# `TileDrawing.cs:5537-5638` (581's body shared with 270/271, cited in batch 4;
the big-cage group body continues to :5755 — batches 6-7 audit the remaining ids).

| Id | C# body (verbatim key lines) | C# line | Rust arm | Verdict |
| --- | --- | --- | --- | --- |
| 581 | shares the 270/271 parity-chain mod 6 + flip | :5537-5563 | `drawdata.rs:816-833` | PORTED |
| 660 | parity chain mod 5 (below) | :5564-5590 | `drawdata.rs:836-853` | PORTED |
| 572 | `for (num10 = Main.tileFrame[typeCache] + x % 4; num10 > 3; num10 -= 4) {}` | :5591-5604 | `drawdata.rs:856-864` | PORTED |
| 428 | `tileTop += 4;` + pressed-plate `addFrX += 18` | :5605-5611 | static `drawdata.rs:868-870` + `cases_b.rs:280-284` | PORTED (wired) |
| 442 | 20×20, top/width by `tileFrameX / 22` sub-switch | :5612-5628 | `drawdata.rs:873-885` | PORTED |
| 275 | big-cage group: top 2 + `bunnyCageFrame[idx] * 54` | :5629-5685 | static `drawdata.rs:890-911` + `drawdata_furniture.rs:117` | PORTED-UNWIRED |
| 276 | `squirrelCageFrame[idx] * 54` | :5694-5704 | :120 | PORTED-UNWIRED |
| 277 | `mallardCageFrame[idx] * 54` | :5706-5708 | :121 | PORTED-UNWIRED |
| 278 | `duckCageFrame[idx] * 54` | :5709-5711 | :122 | PORTED-UNWIRED |
| 279 | `birdCageFrame[idx] * 54` | :5718-5721 | :125 | PORTED-UNWIRED |
| 280 | `blueBirdCageFrame[idx] * 54` | :5722-5724 | :126 | PORTED-UNWIRED |
| 281 | `redBirdCageFrame[idx] * 54` | :5725-5727 | :127 | PORTED-UNWIRED |
| 296 | `scorpionCageFrame[0, idx] * 54` | :5735-5738 | :129 | PORTED-UNWIRED |
| 297 | shares 296's arm (row 0) | :5736 | :129 | PORTED-UNWIRED |
| 309 | `penguinCageFrame[idx] * 54` | :5739-5741 | :130 | PORTED-UNWIRED |

**660 ShimmerflyinaBottle** — C# verbatim (:5564-5590): the 270 shape with mod 5 —
`num11 = Main.tileFrame[typeCache] + x % 5`, the same three `+3` parity gates
(x%2/x%3/x%4), `while (num11 > 4) num11 -= 5;`, `addFrX = num11 * 18; addFrY = 0;`,
flip on even x. Port `drawdata.rs:836-853` term-for-term. Animator:
`step_gt_gt(660, 8, 4)` (`drawdata_anim.rs:314`, `Main.cs:18782-18791`).
**PORTED.**

**572 SoulBottles** — C# verbatim (:5591-5604):

```csharp
		case 572:
		{
			int num10;
			for (num10 = Main.tileFrame[typeCache] + x % 4; num10 > 3; num10 -= 4)
			{
			}
			addFrX = num10 * 18;
			addFrY = 0;
			if (x % 2 == 0)
			{
				tileSpriteEffect = SpriteEffects.FlipHorizontally;
			}
			break;
		}
```

Port `drawdata.rs:856-864`: the empty-body `for` becomes a `while` — same unwind,
no parity gates here (the 4-column bottle has none in C#). Animator:
`step_ge_ge(572, 6, 4)` (`drawdata_anim.rs:434`, `Main.cs:19276-19283`).
**PORTED.**

**428 WeightedPressurePlate** — C# verbatim (:5605-5611):

```csharp
		case 428:
			tileTop += 4;
			if (PressurePlateHelper.PressurePlatesPressed.ContainsKey(new Point(x, y)))
			{
				addFrX += 18;
			}
			break;
```

Static arm `drawdata.rs:868-870` carries `top += 4` (the `+=` matters — the
default top is 0, but the C# adds); its comment "treats plates as not pressed" is
STALE as a behaviour claim: `cases_b.rs:280-284` adds `add_fr_x += 18` when
`env.wiring.plate_is_pressed(x, y)` (`wiring.rs:571`, the ported
`PressurePlatesPressed` map), wired at `tiles.rs:700-712`. **PORTED, wired.**

**442 ProjectilePressurePad** — C# verbatim (:5612-5628):

```csharp
		case 442:
			tileWidth = 20;
			tileHeight = 20;
			switch (tileFrameX / 22)
			{
			case 1:
				tileTop = -4;
				break;
			case 2:
				tileTop = -2;
				tileWidth = 24;
				break;
			case 3:
				tileTop = -2;
				break;
			}
			break;
```

Port `drawdata.rs:873-885`: same sub-switch on `fx / 22` (the 20×20 tree-style
22 px grid), variant 2 widens to 24, default keeps top 0. **PORTED.** (The pad's
projectile-trigger state is wiring, not DrawData.)

**Big-animal cage group** — C# group head, verbatim (:5670-5674):

```csharp
		{
			tileTop = 2;
			Main.critterCage = true;
			int bigAnimalCageFrame = this.GetBigAnimalCageFrame(x, y, tileFrameX, tileFrameY);
			switch (typeCache)
```

The probe, verbatim (:6542-6547):

```csharp
	private int GetBigAnimalCageFrame(int x, int y, int tileFrameX, int tileFrameY)
	{
		int num = x - tileFrameX / 18;
		int num2 = y - tileFrameY / 18;
		return num / 6 * (num2 / 4) % Main.cageFrames;
	}
```

Port `big_animal_cage_frame` (`drawdata_furniture.rs:99-103`) is term-for-term;
`CAGE_FRAMES = 25` (:32, `Main.cs:66901`). The per-id inner arms map to
`cage_table` (:117-132) exactly as the table above lists; `cage_draw_data`
(:169-201) applies `top = 2` and `add_fr_y = frame * 54`. The scorpion pair reads
row 0 of the 2-D table (`scorpionCageFrame[0, idx]`, :5737) — the port's
`CageAnim::Scorpion` row-0 convention (:51-53/:129) carries it.

**Why UNWIRED (same finding as d64's cage batches):** the live draw path
(`tiles.rs:671-734`) runs `draw_data` → `apply_world_corrections` → `cases_b` →
`cases_a`; `furniture_post_pass` (`drawdata_furniture.rs:262`) has zero callers
outside its own tests (grep-verified across `crates/`). The static fallback
(`drawdata.rs:890-911`) sets only `top = 2`. Cage kinds animate through
`AnimateTiles_CritterCages` (`Main.cs:13511`, called at `Main.cs:19303`), which
moves the `*CageFrame` tables — the port defers it (`drawdata_anim.rs:21/:455`),
so the tables stay zero-initialised (`Main.cs:66902-66972`) and row 0 is the
exact C# t=0 state: pixel-identical today, **frozen** until the animator +
post-pass wiring land (furniture lane W4-d57). Tests pin the rewrite:
per-kind sweep `drawdata_furniture.rs:323-370`, zero-table byte-equality :471-493.
The `Main.critterCage = true` side effect (:5672) gates that animator and is
likewise deferred (documented :167-168).

### Batch 6 — big-cage group middle: 358, 359, 413, 414, 542, 550, 551, 553, 554, 558, 559, 599, 600, 601, 602 (15 ids)

All fifteen share the group head cited in batch 5 (`tileTop = 2;
Main.critterCage = true;` + the BIG probe, :5670-5674). Inner arms verbatim
(`TileDrawing.cs:5676-5745`) vs `cage_table` (`drawdata_furniture.rs`):

| Id | C# inner arm (verbatim) | C# line | Rust arm | Verdict |
| --- | --- | --- | --- | --- |
| 358 | `case 279:` `case 358:` → `addFrY = Main.birdCageFrame[bigAnimalCageFrame] * 54;` | :5718-5721 | `279 \| 358 => (CageAnim::Bird, 0, 54, BIG)` :125 | PORTED-UNWIRED |
| 359 | `case 275:` `case 359:` `case 599:`…`case 605:` → `addFrY = Main.bunnyCageFrame[bigAnimalCageFrame] * 54;` | :5676-5686 | `275 \| 359 \| 599..=605 => (CageAnim::Bunny, 0, 54, BIG)` :117 | PORTED-UNWIRED |
| 413 | `case 276:` `case 413:` `case 414:` `case 606:`…`case 612:` → `addFrY = Main.squirrelCageFrame[bigAnimalCageFrame] * 54;` | :5694-5705 | `276 \| 413 \| 414 \| 606..=612 => (CageAnim::Squirrel, 0, 54, BIG)` :120 | PORTED-UNWIRED |
| 414 | shares the squirrel arm | :5696 | :120 | PORTED-UNWIRED |
| 542 | `case 542:` → `addFrY = Main.owlCageFrame[bigAnimalCageFrame] * 54;` | :5691-5693 | `542 => (CageAnim::Owl, 0, 54, BIG)` :119 | PORTED-UNWIRED |
| 550 | `case 550:` `case 551:` → `addFrY = Main.turtleCageFrame[bigAnimalCageFrame] * 54;` | :5687-5690 | `550 \| 551 => (CageAnim::Turtle, 0, 54, BIG)` :118 | PORTED-UNWIRED |
| 551 | shares the turtle arm | :5688 | :118 | PORTED-UNWIRED |
| 553 | `case 553:` → `addFrY = Main.grebeCageFrame[bigAnimalCageFrame] * 54;` | :5712-5714 | `553 => (CageAnim::Grebe, 0, 54, BIG)` :123 | PORTED-UNWIRED |
| 554 | `case 554:` → `addFrY = Main.seagullCageFrame[bigAnimalCageFrame] * 54;` | :5715-5717 | `554 => (CageAnim::Seagull, 0, 54, BIG)` :124 | PORTED-UNWIRED |
| 558 | `case 558:` `case 559:` → `addFrY = Main.seahorseCageFrame[bigAnimalCageFrame] * 54;` | :5742-5745 | `558 \| 559 => (CageAnim::Seahorse, 0, 54, BIG)` :131 | PORTED-UNWIRED |
| 559 | shares the seahorse arm | :5743 | :131 | PORTED-UNWIRED |
| 599 | shares the bunny arm | :5678 | :117 | PORTED-UNWIRED |
| 600 | shares the bunny arm | :5679 | :117 | PORTED-UNWIRED |
| 601 | shares the bunny arm | :5680 | :117 | PORTED-UNWIRED |
| 602 | shares the bunny arm | :5681 | :117 | PORTED-UNWIRED |

Verbatim exemplar — the bunny group, the widest label run (:5676-5686):

```csharp
			case 275:
			case 359:
			case 599:
			case 600:
			case 601:
			case 602:
			case 603:
			case 604:
			case 605:
				addFrY = Main.bunnyCageFrame[bigAnimalCageFrame] * 54;
				break;
```

The port's `599..=605` range arm covers exactly the C# label run 599-605 (the
remaining 603-605 are batch 7's ids). The squirrel run is split in C# — 276/413/414
then 606-612 — and the port keeps it as one alternation with the same membership.
UNWIRED rationale identical to batch 5: static fallback `drawdata.rs:890-911`
(top 2 only), `furniture_post_pass` uncalled, critter animator deferred, frozen
row 0 = the C# t=0 pixels. The per-kind sweep test
(`drawdata_furniture.rs:330-369`) pins every one of these picks against a
distinct-valued fake table, so a wrong table/row/probe would fail loudly once
wired.
