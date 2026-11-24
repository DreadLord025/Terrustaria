# 101 — W4-g80 task-seeds: the snow evenActuated arm + the housing nactive residue

> W4-g80 lane 21 follow-up round — 2026-11-21 — code-only (no cargo/tests/graphify)
> Branch `task/W4-g80-tile-actuator-draw`, continuing 9bcaa86f (DOC-SWEEP) +
> 1284dc63 (the autoActuator build-pass port).
> Sources: `decompiled/Terraria/{Player,Projectile,Collision,WorldGen,Wiring}.cs`,
> `decompiled/Terraria/GameContent/Drawing/TileDrawing.cs` vs `crates/` +
> the pushed `origin/task/W4-d1*`, `W4-d118*`, `W4-d119*` tips.

This round: (1) the autoActuator build-pass consumer PORTED (1284dc63,
`world_sim/editing/auto_actuator.rs`); (2) the actuator-DRAW lane re-verified
(all arms landed, no port needed); (3) actuators-in-walls censused (block
behind wall actuates fine; wall-only cells are a C# quirk, recorded below);
(4) this seed for the AI_010 falling-tile lane; (5) this cross-note for the
d118/d119 housing lanes.

## Seed A — the snowfall evenActuated second pass (AI_010 lane)

### Ownership (read from the pushed claims)

Style 10 is **W4-d1**'s: its census commit `e271b5bc` ("the aiStyle-1..10
census", ANALYSIS "The aiStyle-1..10 projectile census") rows style 10 as
"PORTED for the rowed kinds (verify-only) — the sand family 42/65/68/354
rides the falling-tile tail; the 42 row's full AI_010 arm is the documented
S1-D-5 gap". **d3 owns aiStyles 21-30, d7 owns 61-70 — neither owns style
10.** The falling-tile flight itself is the generic `falling_tile` tail
(`tr-app/projectile.rs:19820-19829` + the landing `:19887-19907`,
`tr-core/falling.rs`), unclaimed by any per-style lane. This seed is for
whoever takes the AI_010 falling-tile completion (d1's census row is the
natural claim).

### The C# contract (precise seam)

`Projectile.HandleMovement`'s aiStyle-10 arm (`Projectile.cs:17592-17611`),
the snowfall variant (`:17598-17602`):

```csharp
else if (this.type == 411 || this.type == 412 || this.type == 413 || this.type == 414)
{
    base.velocity = Collision.TileCollision(base.position, base.velocity, base.width, base.height, flag6, flag6);
    base.velocity = Collision.AnyCollisionWithSpecificTiles(base.position, base.velocity, base.width, base.height, TileID.Sets.Falling, evenActuated: true);
}
```

- Runs per sub-step BEFORE the position integrates (`position += velocity`
  is `Update`'s generic tail). `flag6` = the fallThrough flag (the
  `decidesManualFallThrough`/`shouldFallThrough` override at `:17583-17586`);
  the snow projectiles' SetDefaults rows carry `tileCollide` true.
- Pass 1 (`Collision.TileCollision`) is the normal solid collision — an
  ACTUATED tile is air to it (the `nactive` gates, `Collision.cs:3675-3707`).
- Pass 2 (`AnyCollisionWithSpecificTiles`, `Collision.cs:2813-2913`) collides
  ONLY with `TileID.Sets.Falling` kinds (53, 234, 112, 116, 224, 123,
  330-333, 495 — the port's `tr_core::falling::FALLING` /
  `tiledata::SET_FALLING`, verbatim) and `evenActuated: true` SKIPS the
  `inActive()` refusal (`:2849`: `(!evenActuated && tile.inActive())`).
  Net effect: **snow falls through actuated sand but LANDS on it** — the
  actuated Falling tile still stops the snowball. This is the ONLY
  `evenActuated: true` call in the decompile; the `AnyCollision` overload's
  parameter (`:2915`, gate `:2950`) has no true caller.
- Pass 2's geometry: the scan box is `(int)(pos/16) - 1 ..= (int)((pos+size)/16) + 2`
  clamped to `[0, maxTilesX] × [0, maxTilesY - 40]`; half-brick cells shrink
  to 8 px from the top (`:2856-2860`); the four-edge resolution keeps the
  last-blocked-row/column dedup (`num5..num8`) and the `+0.01f` nudges
  (`:2904-2907`); `tileSolidTop` kinds block ONLY from above (`:2874/:2887/:2900`).
  The `AnyCollision` twin (`:2915-3010`) is the same walk without the
  kind-set gate — port ONE helper with an `Option<&TileSet>` if both land.
- The snow AI arm itself (`AI_010`, `Projectile.cs:57853-57869`): the
  `Main.rand.Next(3) == 0` dust roll (type 9; 412/414 → 11; 413 → 19,
  `noGravity`, `velocity -= base.velocity * 0.5f`) — a per-tick draw on the
  UpdateProjectiles stream, keep the order when the AI arm lands.

### The port gap (the seam to extend)

The port's falling-tile flight has NO per-tick tile collision at all:
`projectile.rs:19824-19829` is gravity (`+0.41`, cap 10) + `pos += vel`
only; the landing is the post-loop `falling::landed` probe
(`falling.rs:129-137` — `active && TILE_SOLID`, **no `!inactive()` term**)
+ `can_place_back` (`:141-153`). Two distinct deltas:

1. The flight needs the per-sub-step `TileCollision` (pass 1) for ALL
   falling kinds — the wider AI_010 gap d1's census already rows.
2. The snowfall kinds 411-414 additionally need pass 2 with the
   `even_actuated` parameter — this seed. `falling::landed`'s missing
   `!inactive()` is a THIRD delta: in C# the actuated tile stops the flight
   via pass 2 (velocity zeroed at the surface), not via the landing probe;
   a port that only adds `!inactive()` to `landed` without pass 2 would let
   the snow sink INTO the actuated tile before refusing to place back.

Witness shape (when ported): an actuated sand tile (53, FLAG_INACTIVE) at
the flight path — the snowball's velocity.Y zeroes at its top edge (pass 2
blocks); a NON-Falling actuated stone does NOT block (pass 2's set gate);
an active sand blocks via pass 1 already.

## Seed B — housing cross-note for W4-d118/d119 (the nactive residue)

Read from the pushed tips: `origin/task/W4-d118-housing-scoreroom`
(`91db266e`, phase 1) and `origin/task/W4-d119-housing-relocation`
(`315a9a7a`, stacked on d118).

**FIXED by d118 phase 1 (verified in the diff):**
`housing.rs::solid_tiles_in_box` now cites `Collision.cs:3455-3488` and
carries the four edge guards, the null-cell arm and the full
`active && !inactive && TILE_SOLID && !TILE_SOLID_TOP` test. The
research/100 gap row for this helper is CLOSED on their branch.

**STILL MISSING on the d118 tip (the residue — line numbers are THEIR
branch's `crates/tr-core/src/housing.rs`):**

| Site | d118 tip line | C# | Missing term |
| --- | --- | --- | --- |
| The home-spot candidate | `:300` (`if !tile.active()`) | `WorldGen.cs:5903` `!Main.tile[i, j].nactive()` | `&& !tile.inactive()` |
| The side-neighbour `framed` closure | `:307-311` (`t.active() && TILE_SOLID.has`) | `:5903` `!Main.tile[i-1, j].nactive()` / `[i+1, j]` | `&& !t.inactive()` |
| The furniture scan | `:324` (`if !t.active() \|\| IGNORED_IN_HOUSE_SCORE...`) | `:5915` `tile.nactive()` | `&& !t.inactive()` (i.e. `!t.active() \|\| t.inactive() \|\| ...`) |

Effect: an actuated floor still scores as a home spot / its actuated
neighbours still count as framing / actuated furniture still scores. The
three-above probe (`:365-382`) reads `active()` + `tileSolid` only — that
one is CORRECT, C# `:5965` has no `nactive` there (d118's comment pins it).

Witness shape: a room whose floor row is actuated (`FLAG_INACTIVE`) — C#
`ScoreRoom` finds NO home spot on it (`hiScore` stays 0); the d118-tip port
still picks one. d119's relocation chain inherits the fix through
`score_room`/`QuickFindHome` — no separate d119 edit needed.

## Census C — actuators in walls (this lane's question 3)

- **Walls themselves: NONE.** `WallDrawing.cs` zero `inActive`/`actColor`
  refs (research/100 §5, re-verified). No wall actuation exists in C#.
- **A BLOCK behind a wall: actuates normally.** The actuation chain never
  reads `tile.wall`: `Wiring.DeActive` (`Wiring.cs:3195-3223`) gates on
  active/226-bulb/tileSolid/NotReallySolid/exceptions/above-cell only; the
  port's `de_active` (`wiring.rs:2968-3004`) matches. Draw: the wall layer
  draws first, the ghosted block (×0.4) draws over it — no wall interaction
  in `TileDrawing`'s ghost arms. PORTED, no gap.
- **A wall-only cell (no active tile): the C# quirk.** `PlaceActuator`
  (`WorldGen.cs:58011-58020`) has NO active/solid gate — it sets the 0x800
  bit on ANY tile object (Main.tile cells always exist), and
  `CanDoWireStuffHere` (`Player.cs:46921-46944`) gates only the wall TYPES
  350/87, not tile activity. So the Actuator item CAN mark a wall-only cell
  (the icon overlay draws, g81's lane). But `HitWireSingle`'s
  `if (tile.actuator()) ActuateForced(...)` runs BEFORE the `!active()`
  return, and `DeActive`'s `!active()` guard then refuses — the bit is
  inert. The port matches bit-for-bit: `edit::place_actuator`
  (`edit.rs:120-130`) has no active gate, `use_wiring_tool`'s Actuator arm
  places on any in-bounds cell, `de_active` refuses inactive cells.
  VERIFIED, no change.

## Census D — the actuator-DRAW lane (question 2, re-verified)

All arms landed on this branch (af6cd44f + the earlier draw waves); nothing
to port:

- The flat/sliced ×0.4 ghost: `render/tiles.rs:614-624` (the per-cell
  `FLAG_INACTIVE` → RGB ×0.4, `GetFinalLight` `TileDrawing.cs:4380-4397`),
  the sliced twin `sliced_light` (anchors rows 187-188).
- The disco-arm ghost: `disco_final_color` (`tiles.rs:1518-1521` →
  `drawdata::act_color_bytes`, the double-0.4 byte truncation).
- The behind-liquid refusal: `render/liquids.rs:418/:453`
  (`liquid_behind_refused`, `TileDrawing.cs:3858`'s `inActive()` term,
  witness `:753`).
- The actuator icon overlay: g81's pushed lane (6f046998/0017d32a) — not
  duplicated. The shine branch: d56's unwired port, order pinned there.

Code-only: no cargo/rustc/rustfmt/tests/graphify invoked this round. The
autoActuator port's 8 witnesses are WRITTEN, NOT RUN (1284dc63). RNG
untouched (the build pass draws no random numbers — `PlaceActuator`'s sound
0 is the fixed Dig cue, no `Next` variation on the wiring feedback seam).
