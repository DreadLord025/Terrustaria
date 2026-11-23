# 08-d113 — TileObjectData third slice: TileID 240–359 census + witnesses

> W4-d113 (`task/W4-d113-tileobjectdata-3`) · 2026-11-20 · code-only mode
> (no cargo/tests run) · sources: `decompiled/Terraria/ObjectData/TileObjectData.cs`
> (5 328 lines, Terraria 1.4.5.8), `crates/tr-core/src/tileobject/registry.rs`

## Range claim (documented early, per the lane brief)

The TileObjectData lane is d111–d114 + e11–e12. **d113 owns the THIRD ~120
TileID block: kinds 240–359 inclusive** (d111: 0–119, d112: 120–239,
d114: 360–479, e11: 480–599, e12: 600–753). Sibling claims are written in
their own worktrees/branches; this range is disjoint from all of them.

## Premise correction (research/08 row is stale)

research/08-tile-framing.md:84/146 still says "36 defs registered / hand-
maintained, no generator". That was superseded by **S1-F-7 wave A
(2026-11-18, landed on main a725acb0)**: `tools/GenTileObjectData/gen.py`
INTERPRETS `TileObjectData.Initialize` (`TileObjectData.cs:2110-5014`)
statement-by-statement and emits **all 389 registered tiles (604 defs)** into
`crates/tr-core/src/tileobject/registry.rs` (ANALYSIS "S1-F-7 wave A").
Verified this task: `python tools/GenTileObjectData/gen.py --check` →
`check: registry.rs is current (389 tiles, 604 defs)`, exit 0.

**There are NO missing static defs.** Every `addTile` of `Initialize` —
including all 64 registered kinds of this range — already resolves through
`BY_KIND`. Re-transcribing them would violate the never-hand-edit rule for
the generated table. The d113 contribution is therefore the **per-kind
acceptance census + witnesses** for the third block (data-only; the
generator remains the single source of truth), plus the honest gap list
below.

## Census — the 64 registered kinds of 240–359

Every row's def is generated; the C# column cites the `addTile` line (the
def chain runs from the previous `addTile`/`addBaseTile` to it). Presets
(`addBaseTile`): Style1x2Top :2419-2441, StyleSmallCage :2655-2667,
Style6x3 :2696-2706, Style2xX :2939-2954 (approx — the chain start),
Style2x2 :3451-3460 region, Style3x2/Style3x3 presets per the generator doc.

| TileID | Name (TileID.cs) | C# chain → addTile | Preset / notes |
|---|---|---|---|
| 240 | Painting3X3 | :4799-4802 | wall painting |
| 241 | Painting4X3 | :4839-4843 | wall painting |
| 242 | Painting6X4 | :4844-4850 | wall painting |
| 243 | ImbuingStation | :4056-4058 | Style3x3 + DrawYOffset 2 |
| 244 | BubbleMachine | :3214-3216 | Style3x2 + DrawYOffset 2 |
| 245 | Painting2X3 | :4824-4832 | wall painting |
| 246 | Painting3X2 | :4833-4838 | wall painting |
| 247 | Autohammer | :4059-4061 | Style3x3 + DrawYOffset 2 |
| 254 | Pumpkins | :3497-3504 | Style2x2, wrap/mult/random 6, AnchorValidTiles {2,477,109,492} |
| 269 | Womannequin | :3053-3060 | Style2xX, origin (0,2), PlaceLeft + PlaceRight alt, LavaDeath true AFTER the alternate (base-only write) |
| 270 | FireflyinaBottle | :2442-2448 | Style1x2Top, DrawYOffset −2, platform alt −10 |
| 271 | LightningBuginaBottle | :2449-2455 | same chain as 270 |
| 275 | BunnyCage | :2707-2708 | Style6x3 |
| 276 | SquirrelCage | :2709-2710 | Style6x3 |
| 277 | MallardDuckCage | :2715-2716 | Style6x3 |
| 278 | DuckCage | :2717-2718 | Style6x3 |
| 279 | BirdCage | :2719-2720 | Style6x3 |
| 280 | BlueJay | :2721-2722 | Style6x3 |
| 281 | CardinalCage | :2723-2724 | Style6x3 |
| 282 | FishBowl | :3671-3673 | Style2x2 group |
| 283 | HeavyWorkBench | :4062-4064 | Style3x3 + DrawYOffset 2 |
| 285 | SnailCage | :2668-2669 | StyleSmallCage |
| 286 | GlowingSnailCage | :2670-2671 | StyleSmallCage |
| 287 | AmmoBox | :3617-3618, :3621-3622, :3782-3783 | **registered THREE times — last addTile wins (`_data[287]` overwrite, :2011)** |
| 288–295 | the 8 butterfly jars | :3689-3712 | one StyleSmallCage-family chain per jar |
| 296 | ScorpionCage | :2737-2738 | Style6x3 |
| 297 | BlackScorpionCage | :2739-2740 | Style6x3 |
| 298 | FrogCage | :2676-2677 | StyleSmallCage |
| 299 | MouseCage | :2678-2679 | StyleSmallCage |
| 300 | BoneWelder | :4065-4067 | Style3x3 + DrawYOffset 2 |
| 301 | FleshCloningVat | :4068-4070 | Style3x3 + DrawYOffset 2 |
| 302 | GlassKiln | :4071-4073 | Style3x3 + DrawYOffset 2 |
| 303 | LihzahrdFurnace | :4074-4076 | Style3x3 + DrawYOffset 2 |
| 304 | LivingLoom | :4077-4079 | Style3x3 + DrawYOffset 2 |
| 305 | SkyMill | :4080-4082 | Style3x3 + DrawYOffset 2 |
| 306 | IceMachine | :4083-4085 | Style3x3 + DrawYOffset 2 |
| 307 | SteampunkBoiler | :4086-4088 | Style3x3 + DrawYOffset 2 |
| 308 | HoneyDispenser | :4089-4091 | Style3x3 + DrawYOffset 2 |
| 309 | PenguinCage | :2741-2742 | Style6x3 |
| 310 | WormCage | :2680-2681 | StyleSmallCage |
| 316 | BlueJellyfishBowl | :3713-3715 | water-animal chain |
| 317 | GreenJellyfishBowl | :3716-3718 | water-animal chain |
| 318 | PinkJellyfishBowl | :3719-3721 | water-animal chain |
| 319 | ShipInABottle | :3619-3620 | Style2x2 |
| 320 | SeaweedPlanter | :2993-2994 | Style2xX |
| 324 | BeachPiles | :4430-4439 | Style1x1 + 20-px cell, wrap 3 horizontal, DrawFlipHorizontal; **NO GetStyleOverride** (correction: only 233/185/165 are tagged, :4968/:4979/:4993) |
| 334 | WeaponsRack | :4807-4814 | item-frame rack (TE consumer) |
| 335 | FireworksBox | :3784-3785 | Style2x2 |
| 337 | AlphabetStatues | :2968-2969 | Style2xX |
| 338 | FireworkFountain | :4213-4214 | wiring consumer |
| 339 | GrasshopperCage | :2686-2687 | StyleSmallCage |
| 349 | MushroomStatue | :2960-2967 | Style2xX, wrap 7, PlaceLeft + PlaceRight alt(7) |
| 354 | BewitchingTable | :3792-3794 | Style3x3 + DrawYOffset 2 |
| 355 | AlchemyTable | :3795-3797 | Style3x3 + DrawYOffset 2 |
| 356 | Sundial | :3801-3802 | Style2xX; NOT a TileObject placement route consumer here — the dial interaction rides edit.rs kind literals (:290) |
| 358 | GoldBirdCage | :3758-3759 | Style6x3-family gold chain |
| 359 | GoldBunnyCage | :3760-3761 | Style6x3-family gold chain |

64 registered + 56 unregistered = 120 kinds of the block. The unregistered
kinds are the plain blocks/woods/platings (248-253, 272-274, 284, 311-313,
315, 321-322, 325, 346-348, 350, 357...), the gemspark tiles 255-268 (the
documented M1.4-TO exclusion: wall/light render work), the falls 326/327/345,
coin piles 330-333, living fires 336/340-344, thorns 352, vine rope 353 —
all legacy `WorldGen.PlaceTile` single-tile routes in vanilla too
(`CustomPlace` false/absent), NOT missing defs.

## Honest gaps in this range (NOT static-def gaps)

1. **Interaction overlay**: gen.py `INTERACTION` (:913-926) carries 12 tags;
   the range's C# `TileInteractionsCheck` arms — 335 fireworks box
   (`Player.cs:32331`), 338 firework fountain (`:32355`), 334 weapons rack
   (`:32448`), 354 bewitching table (`:32624`), 287 ammo box (`:32630`),
   356 sundial (`:32636`, ported as kind-literal dial arm edit.rs:290),
   314 minecart (`:33262`, unregistered kind) — have no registry tags. The
   runtime bodies are sim scope; recorded, not fabricated.
2. **Draw-only fields** (DrawYOffset −2/−10/2 everywhere above) are parsed by
   gen.py but NOT emitted — S1-F-9 DrawData scope (wave C furniture module
   already covers the cage frame tables).
3. **Hook bodies** (`TEWeaponRack`/`TEItemFrame` post-place hooks on 334 etc.)
   stay documented callbacks (`hook: None` + name), the wave-A seam.

## Re-scope (registry generated-complete) — the coordinate-walk consumer

The Lead re-scoped d113 after the registry proved complete: port a
TileObjectData logic consumer NOT claimed by d112. d112's in-flight work
(`TR-W4-d112` worktree, uncommitted) claims the `GetTileData(Tile)`
overload (:5196-5261) as `resolve_tile_def` + `apply_style_override` — the
GetStyleOverride resolution. d113 therefore takes the NEXT logic gap per
research/08 row 86 ("`TryGetTileBounds` / coordinate pickup"):
**`TileObjectData.TryGetTileBounds` (:5163-5194)**, ported as
`tileobject::try_get_tile_bounds(world, x, y, resolve)`.

### Coordinate ANALYSIS (the walk, :5163-5194)

- `bounds = (x, y, 1, 1)`; `GetTileData(tile)` (:5167) gates — a null
  resolution returns false with the 1×1 bounds (the C# out-param default).
- `num3 = frameX % CoordinateFullWidth`, `num4 = frameY %
  CoordinateFullHeight` (:5172-5175, written as `f - (f/c)*c` — truncating
  division, so negative frames keep a negative remainder and step nothing).
- The column walk (:5176-5181) is a `while num3 >= CoordinateWidth` loop:
  one tile left per column, **padding included** (a padding column's
  remainder 18 ≥ 16 still steps once).
- The row walk (:5182-5190) is a `for` over EVERY `CoordinateHeights[i]`
  with `if num4 >= h { num4 -= h; bounds.Y--; }` — no early exit. For
  uneven heights this has a vanilla quirk: the chest's row-1 PADDING
  (frameY 36, heights [16,18], CFH 38) subtracts 16 then 18 → TWO row
  steps, landing one tile above the true top. The port keeps it 1:1
  (witness `w4_d113_walk_chest_21_uneven_heights`); real chest cells never
  carry padding frames, so it is unreachable in practice.
- `bounds.Width/Height = tileData.Width/Height` (:5191-5192) — the
  RESOLVED def's extents (sub-tile/alternate-aware through the seam).

### The seam (no duplicate port)

The walk's only dependency on the tile-based resolver is :5167. d113's
first draft ported `GetTileData(Tile)` too; on finding d112's claim that
duplicate was DELETED and the walk now takes
`resolve: fn(&Tile) -> Option<&'static TileObjectDef>` as a parameter.
The merge wires `try_get_tile_bounds(world, x, y, resolve_tile_def)` —
the C# call shape, no logic duplicated across branches. The Rust witnesses
use a base-def slice of the seam (exact for the sub-tile-free kinds they
exercise) plus a synthetic resolved-def test proving the walk reads the
RESOLVED geometry.

### Wall-frame consumers — analysis result: NOT TileObjectData consumers

research/08's wall-frame gaps (`Framing.WallFrame`, :337-396) consume
`Main.wallLargeFrames`/`TruncatesWalls`/`invisibleWall`, NOT
TileObjectData — no `TileObjectData.` reference exists in `Framing.cs`
(verified by search). The only TileObjectData consumers in the framing
family are `PlatformFrameWidth()` (:5121-5124, a constant read of
`_data[19].CoordinateFullWidth` — already reused from `torch.rs`,
`WorldGen.cs:51858`/:70155`) and `CheckWaterDeath`/`CheckLavaDeath(Tile)`
(:5073-5119, `WorldGen.cs:72930`/:80439-80444) — the latter DO call
`GetTileData(Tile)` and are d112-seam consumers, recorded here as the
next wiring step, not re-ported.

### Sole C# consumers of TryGetTileBounds

`RichRoomCheckFeedback.cs:228` (the housing-quality particle pass consumes
the other furniture cells inside an object's bounds so a 3×3 table counts
once) and `:298` (centres the room-check icon particle on the bounds:
`((Left+Right)/2, Min(Top/2+Bottom/2, Top+1)) * 16`). Housing feedback is
unported (d118/d119 lane); the walk lands as the registry's generic
coordinate pickup, ready for that wiring.

## Witnesses (WRITTEN NOT RUN — code-only mode)

- Rust: `tileobject.rs` tests `w4_d113_walk_*` (5) — the 283 bench 3×3
  walk incl. padding + style-column reset, the 275 cage 6-wide column
  walk, the 21 chest uneven-height quirk, the 270 bottle + the
  inactive/unregistered/out-of-bounds/negative-frame gates, and the
  resolved-def geometry proof.
- Python: `tools/GenTileObjectData/test_gen.py` gains
  `test_d113_third_block_census` + `test_d113_third_block_families`:
  per-family pins hand-derived from the C# chains above, run through the
  same interpreter the generator uses (the S1-F-7.4 pattern). The Rust-side
  freeze (`tileobject_registry_freeze.rs`) already pins the whole table's
  FNV — this slice adds the per-kind semantic layer for 240–359.