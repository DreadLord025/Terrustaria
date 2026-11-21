# 124 — B7 Hellforge graft-patch: g37's PlaceTile route → d104's bare place_3x2

> Wave 4 · produced on `task/W4-g37-worldgen-missing-b` (reconcile DONE
> `53a5b5b5`, gap-fill `4ced188f`). Consumer: **d104b** (the d104 follow-up).
> Decision record: `docs/research/75` §2.7 (REFINES matrix 88 §2.13 —
> 88 keeps d104's shell on a witness tie; 75 adds the placement-route
> evidence). This doc is the **exact graft patch**: the code to move, the
> attach point on d104's LATEST tip, the witness re-pins, and the C#
> citations for every gate the bare route skips.

## 1. The two routes, side by side

**C# truth** (`decompiled/Terraria/WorldGen.cs`): the Hellforge pass
(`:18187-18236`) calls `WorldGen.PlaceTile(num3, j, 77)` at `:18210` —
default `mute:false, forced:false, plr:-1, style:0`. `PlaceTile`
(`:59342-60295`) is NOT a bare `Place3x2`:

| stage | C# anchor | what it does for kind 77 |
| --- | --- | --- |
| 488 mannequin veto | `:59345-59348` | generating + active 488 at the cell → refuse |
| `num >= TileID.Count` | `:59349-59352` | bounds on the kind |
| cell bounds | `:59354` | `i/j` inside the map |
| **entry gate** | `:59373` | `forced \|\| Collision.EmptyTile(i,j) \|\| !Main.tileSolid[num] \|\| …` — generation-time `EmptyTile` reduces to `!tile.active()` (`Collision.cs:1359-1381`, no players/NPCs). **77 is non-solid** (`tileSolid[77]=false`), so the entry always passes for 77 — the gate matters for the *other* kinds the slice serves, and must stay verbatim |
| liquid gate | `:59454-59472` | the torch arm binds only kind 4; the non-torch veto list (`:59468`) does NOT contain 77 → a hellforge may seat over liquid (the `Place3x2` body scan still vetoes active cells) |
| **Clear arm** | `:59473-59476` | an **inactive** anchor cell is `Clear(Tile \| TilePaint \| Slope)` — type/active/frames zeroed, **paint + slope + halfBrick cleared**; wall/liquid/wiring/actuator survive (`Tile.Clear`, `Tile.cs:744-790`) |
| **ResetsHalfBrick arm** | `:59477-59482` | an **active** anchor whose existing type is non-frame-important and in `TileID.Sets.ResetsHalfBrickPlacementAttempt` (`TileID.cs:497`, default-true set) clears halfBrick + frameX/frameY |
| 77 dispatch | `:60079-60084` | `case 17: case 77: case 133:` → `Place3x2(i, j, num, style)` + `SquareTileFrame(i, j)` |
| tail re-frame | `:60255-60262` | if the anchor is active → `SquareTileFrame` again + `result = true` (the `TruncatesWalls` arm `:60257-60260` never fires for 77) |
| sound/dust | `:60263-60290` | skipped under mute; the pass's default `mute:false` would play — the port has no gen-time sound (documented no-op) |

**d104 today** (tip `8a229c3e`, `crates/tr-core/src/worldgen/structures.rs`):
`pass_hellforge` (`:703-737`) calls `super::evil::place_3x2(world, num3, j,
TILE_HELLFORGE, 0)` directly at **`structures.rs:722`** — the whole PlaceTile
preamble above is skipped, and no `SquareTileFrame` runs. d104's
`evil::place_3x2` (`evil.rs:1930-1985`) is itself a reduced slice: no
285-family solidTop floor arm, no 186/187 pile / 26 altar vetoes (the boulder
check is documented out, D20), and its floor predicate is evil.rs's local
`solid_tile2` (`evil.rs:2007-2017`) which reads **OOB → true** — the C#
`SolidTile2` *indexed* access throws OOB and the catch returns **false**
(`WorldGen.cs:70134-70151`; the null-Tile→true case cannot occur in the
port's dense array). `crate::edit::solid_tile2` (`edit/kill.rs:2158-2167`)
is the correct indexed port (OOB → false).

**g37 B7** (`passes_b.rs`, commit `454e43ba`): `pass_hellforge`
(`:2072-2114`) calls the full `place_tile(world, num3, j, 77, false, false,
0, rng)` slice (`:94-229`) → the 77 arm (`:182-186`) → g37's `place_3x2`
(`:416-492`, the full `:52066-52196` arm set) → `crate::edit::solid_tile2`
floor + `crate::edit::square_tile_frame_gen` re-frame.

**RNG-neutrality of the graft:** for kind 77 the `place_tile` slice draws
**nothing** — the only rng-consuming arms are 135 (`place_1x1`), 10
(`place_door`'s `Next(3)`), 20 (`place_1x2`'s `Next(3)`), none of which
fires. The Hellforge retry loop's draw schedule (`Next(1,w)` ×
`Next(h-250,h-30)` per attempt) is byte-identical before/after the graft;
the `pass_rng("Hellforge")` ledger does not move.

## 2. The exact code block to graft (g37's fn, verbatim)

Source: `crates/tr-core/src/worldgen/passes_b.rs` @ this branch. The full
slice (the preamble is kind-generic; d104b may keep it whole for the other
callers or trim the dispatch to the 77 + default arms — the preamble and
tail must stay verbatim either way):

```rust
/// `WorldGen.PlaceTile(i, j, Type, mute, forced, plr, style)`
/// (`WorldGen.cs:59342-60295`), the B-lane slice. The `plr` argument is −1 on
/// every B call (the 79/90 bed-direction arm `:60202-60212` reads it; no B
/// kind is a bed), so it is dropped from the signature. The preamble gates
/// that bind for the B kinds are ported verbatim (the 488 mannequin veto
/// `:59345`, the bounds `:59354`, the `forced || EmptyTile || !tileSolid[num]`
/// entry `:59373` - `Collision.EmptyTile` `Collision.cs:1359-1381` reduces to
/// `!tile.active()` during generation, no players/NPCs exist - the torch
/// liquid gate `:59454-59467`, the `Clear(Tile|TilePaint|Slope)` on an inactive
/// cell `:59473-59476` and the ResetsHalfBrick arm `:59477-59482`,
/// `Tile.cs:744-790`). The kind dispatch covers the B kinds; every other kind
/// falls to the C# default single-tile write (`:60234-60248`). The tail
/// (`:60255-60291`) re-frames and returns `result`; `mute: true` skips the
/// sound/dust block.
pub(crate) fn place_tile(
    world: &mut World,
    i: i32,
    j: i32,
    kind: u16,
    mute: bool,
    forced: bool,
    style: i32,
    rng: &mut Rng,
) -> bool {
    let num = kind;
    // `:59345-59348`: the mannequin 488 veto while generating.
    if world.tile(i, j).is_some_and(|t| t.active() && t.kind == 488) {
        return false;
    }
    // `:59349-59352`: `num >= TileID.Count`.
    if num as usize >= td::TILE_COUNT {
        return false;
    }
    let mut result = false;
    // `:59354`: the bounds gate.
    if i < 0 || j < 0 || i >= world.width || j >= world.height {
        return false;
    }
    let active_here = world.tile(i, j).is_some_and(|t| t.active());
    // `:59364-59372`: the grass-on-mud remaps (23/199 → 661/662) never fire
    // for the B kinds.
    // `:59373`: `forced || Collision.EmptyTile(i, j) || !Main.tileSolid[num] ||
    // …` — the grass-on-dirt / moss disjuncts never fire for the B kinds; the
    // generation-time EmptyTile is `!active` (no players/NPCs exist).
    let entry = forced || !active_here || !td::TILE_SOLID.has(num);
    if !entry {
        return false;
    }
    // `:59375-59430`: the grass/moss type guards — none of the B kinds is
    // 23/199/2/109/60/661/662/70/633 or a `tileMoss` kind, so they pass.
    // `:59431-59453`: the 81 coral floor arm and the IsADripTile veto — no B
    // kind is 81 or a drip tile {373, 374, 375, 461, 709}.
    // `:59454-59472`: the liquid gate. `checkingLiquid` is false during
    // generation; the torch arm binds only for kind 4 (the B5 house torches).
    if let Some(t) = world.tile(i, j) {
        if t.liquid > 0 {
            if num == 4 && style != 8 && style != 11 && style != 17 {
                return false;
            }
            // `:59468`: the non-torch liquid-veto list — no B kind is in it.
        }
    }
    // `:59473-59476`: an inactive cell is cleared (Tile | TilePaint | Slope) —
    // type/active/frames zeroed, paint + slope + halfBrick cleared; wall,
    // liquid, wiring and actuator survive (`Tile.Clear`, `Tile.cs:744-790`).
    if !active_here {
        if let Some(t) = world.tile_mut(i, j) {
            t.kind = 0;
            t.flags &= !FLAG_ACTIVE;
            t.frame_x = 0;
            t.frame_y = 0;
            t.set_paint(0);
            t.set_slope(0);
            t.flags &= !crate::tile::FLAG_HALF_BRICK;
        }
    } else if resets_half_brick(num)
        && !world
            .tile(i, j)
            .is_some_and(|t| crate::tileobject::frame_important(t.kind))
    {
        // `:59477-59482`: the ResetsHalfBrickPlacementAttempt arm — the grass
        // kinds clear halfBrick + frames on an active non-frame-important cell.
        if let Some(t) = world.tile_mut(i, j) {
            t.flags &= !crate::tile::FLAG_HALF_BRICK;
            t.frame_y = 0;
            t.frame_x = 0;
        }
    }
    // `:59483-59971`: the flower / vine / special-object arms — none of the B
    // kinds reaches them (the B kinds dispatch below).
    // The kind dispatch (the B arms of the `:59974-60249` switch).
    match num {
        // `:60065-60078`: the 2xX statue/fountain family.
        105 | 349 | 207 => {
            place_2xx(world, i, j, num, style);
            crate::edit::square_tile_frame_gen(world, i, j);
        }
        // `:60129-60140`: the 1x1 mechanism family (the statue trap plate 135).
        135 => {
            place_1x1(world, i, j, num as i32, style, rng);
            crate::edit::square_tile_frame_gen(world, i, j);
        }
        // `:59987-60003` + `:60079-60084`: the 3x2 family (hellforge 77 rides
        // the case-17/77/133 arm; piles 186/187 + table 14 the case-14 arm).
        77 | 186 | 187 | 14 => {
            place_3x2(world, i, j, num, style);
            crate::edit::square_tile_frame_gen(world, i, j);
        }
        // `:60027-60041`: the 1x2 chair family (15).
        15 => {
            place_1x2(world, i, j, num, style, rng);
            crate::edit::square_tile_frame_gen(world, i, j);
        }
        // `:59843-59883`: the door 10 branch.
        10 => {
            place_door_branch(world, i, j, style, rng);
        }
        // `:59798-59842`: the torch 4 support arm (the B5 house torches).
        4 => {
            if torch_support(world, i, j) {
                if let Some(t) = world.tile_mut(i, j) {
                    t.flags |= FLAG_ACTIVE;
                    t.kind = 4;
                    t.frame_y = (22 * style) as i16;
                }
                crate::edit::square_tile_frame_gen(world, i, j);
            }
        }
        // `:60116-60119`: the banner 91.
        91 => {
            place_banner(world, i, j, num, style);
            crate::edit::square_tile_frame_gen(world, i, j);
        }
        // `:60234-60248`: the default single-tile write.
        _ => {
            if let Some(t) = world.tile_mut(i, j) {
                t.flags |= FLAG_ACTIVE;
                t.kind = num;
            }
        }
    }
    // `:60255-60262`: the tail re-frame + result. The `TruncatesWalls`
    // SquareWallFrame arm (`:60257-60260`) never fires for the B kinds — the
    // set is {54, 328, 459, 748} (`TileID.cs:595`), none of them placed here.
    if world.tile(i, j).is_some_and(|t| t.active()) {
        crate::edit::square_tile_frame_gen(world, i, j);
        result = true;
    }
    let _ = mute; // `:60263-60290`: the sound/dust block — every B call mutes.
    result
}
```

**Companion pieces the graft must carry** (same file):

- `resets_half_brick` (`passes_b.rs:234-270`) — the
  `TileID.Sets.ResetsHalfBrickPlacementAttempt` default-true set
  (`TileID.cs:497`); 77 is NOT in the false-list, so the arm binds for 77.
- `place_3x2` (`passes_b.rs:416-492`) — the full `WorldGen.Place3x2`
  (`:52066-52196`): the picnic one-row arm, the campfire liquid veto, the
  285-family solidTop-frameY-0 floor arm (`:52125-52138`), the 186/187
  `InvalidTileForPilesOrSpeleothems` + 26 boulder floor vetoes
  (`:52107-52123`), the `crate::edit::solid_tile2` floor (OOB → false), the
  column-major frame writes. **Replace d104's `evil::place_3x2` for this
  call** — or re-point `evil::place_3x2`'s floor onto `edit::solid_tile2`
  and add the missing arms (88 §6.3 already flags the g39 Place3x2
  reconciliation; g39's H6 canonical is the eventual home, 75 §5).
- `invalid_tile_for_piles` (`passes_b.rs:496-503`, `:38979-38994`) — only
  needed if the 186/187 arms stay.
- Imports: `crate::tile::{Tile, FLAG_ACTIVE}`, `crate::tiledata as td`,
  `super::noise::Rng`; `crate::edit::{solid_tile2, square_tile_frame_gen}`
  (both already exist on d104 — `edit/kill.rs:2158` / `:222`).
- If d104b trims the dispatch to 77-only: keep the preamble + the
  `77 | 186 | 187 | 14` arm (or just `77`) + the default arm + the tail;
  drop `place_2xx`/`place_1x1`/`place_1x2`/`place_door*`/`torch_support`/
  `place_banner` (they serve other callers g39 owns). The `rng` parameter
  stays in the signature (draw-free for 77) so the call site reads like the
  C# `PlaceTile`.

## 3. The d104 attach point (read from origin/task/W4-d104-worldgen-passes-4 @ 8a229c3e)

`crates/tr-core/src/worldgen/structures.rs:703-737`, `pass_hellforge` —
the single line to replace is **`:722`**:

```rust
// BEFORE (d104 today):
                // `PlaceTile(num3, j, 77)` — the `Place3x2` route
                // (`:60079-60083`), style 0.
                super::evil::place_3x2(world, num3, j, TILE_HELLFORGE, 0);
// AFTER (the graft):
                // `PlaceTile(num3, j, 77)` — the full PlaceTile route
                // (`:59342-60295`), default mute/forced, style 0.
                place_tile(world, num3, j, TILE_HELLFORGE, false, false, 0, rng);
```

Everything around it stays: the wall-13/14 gate, the drop scan + `j--`, the
type-77 success check (`:723`), the 10000 valve, the wall-miss no-bump arm.
`rng` is already in scope (`pass_hellforge(world, rng: &mut super::Rng)`).
The step adapter (`pipeline.rs:2712-2724`, the skyblock
`denyAllGeneration` gate + `pass_rng("Hellforge")`) and the
`step!("hellforge", "Hellforge", step_hellforge)` row (`pipeline.rs:1423`,
between `quick cleanup` and `spreading grass`) are untouched. The merged
chain re-points the row to g37's pots-adjacent slot (75 §4) — the graft is
slot-independent.

Optional fidelity extras while d104b is in the file (g37 B7 carries them):
the sub-vanilla guard `if world.height <= 250 || num <= 0 { return 0; }`
(the C# `Next(maxY-250, maxY-30)` band collapses on a tiny world; g37
`passes_b.rs:2074-2078`). d104's 400×600 witness world is unaffected.

## 4. Witness re-pins

1. **d104 `w4_d104_30_the_hellforge_places_and_replays`**
   (`structures.rs:1406-1435`): the fixture (400×600, wall-14 block rows
   400-500, hell-brick floor at 500) seats every forge on an **inactive,
   zero-paint/slope** anchor — the Clear arm is a no-op there, and the
   `SquareTileFrame` Check-chain keeps the forges (solid floor). The
   assertions (`a.tiles == b.tiles`, `pa == pb`, `pa > 0`, any-77) are
   expected to **survive unchanged**, but the 3×3 re-frame walk may write
   merge frames onto the floor/wall cells — re-run at the gate; if a
   byte-level fixture assertion is added later, pin it on the grafted route.
2. **New distinguishing witness (add with the graft)** — the Clear arm is
   the observable difference: seat an **inactive anchor with paint + slope
   set** (e.g. `set_paint(5)`, `set_slope(2)` on the cell the drop scan
   lands on), run the pass, assert the placed forge's anchor cell has
   `paint() == 0 && slope() == 0 && frame_x/y` from the 3×2 write. The bare
   d104 route leaves the paint/slope on the anchor (place_3x2 writes only
   kind/frames/active) → the witness FAILS before the graft, PASSES after.
   A second arm: an active non-frame-important anchor (dirt 0) with
   halfBrick set → the ResetsHalfBrick arm clears it before the body scan
   refuses the placement (the retry then re-rolls — assert the draw ledger
   advanced by exactly one attempt pair).
3. **g37 B7 module witnesses: 0** (the lane's known gap, 75 §2.7) — the
   graft inherits d104's witness + the new one above; nothing to port from
   g37's file beyond the body itself.
4. **Hash pins**: the graft changes generated bytes (the Clear arm's
   paint/slope clears + the SquareTileFrame re-frames wherever a hellforge
   seats) → `SEED42_512` / `DEFAULT_WORLD` move BY DESIGN; both are already
   stale-by-design on every lane and re-pin ONCE at the gate (75 §6,
   88 §6.2). The `a == b` halves stay green — the graft adds no RNG draw.
5. **The 488 veto / entry gate are behaviourally inert for 77 on a
   generated world** (no mannequins exist during gen; 77 is non-solid so
   the entry always passes) — they are carried for slice fidelity, not for
   a witness; do not write a gen-time witness that pretends otherwise.

## 5. Entry-gate / Clear-arm C# citations (the quick card)

- Entry gate: `WorldGen.cs:59373` (`forced || Collision.EmptyTile || !tileSolid[num] || …`); `Collision.EmptyTile` `Collision.cs:1359-1381`.
- Clear arm: `WorldGen.cs:59473-59476`; `Tile.Clear(TileDataType.Tile | TilePaint | Slope)` semantics `Tile.cs:744-790`.
- ResetsHalfBrick arm: `WorldGen.cs:59477-59482`; the set `TileID.cs:497`.
- 488 veto: `WorldGen.cs:59345-59348`. Liquid gate: `:59454-59472`.
- 77 dispatch: `WorldGen.cs:60079-60084` (`case 17/77/133 → Place3x2 + SquareTileFrame`).
- Place3x2 body: `WorldGen.cs:52066-52196` (floor arms `:52104-52138`, writes `:52151-52195`).
- Tail: `WorldGen.cs:60255-60262`; SquareTileFrame gen entry `:80185-80196` (port: `edit/kill.rs:222`).
- SolidTile2 indexed: `WorldGen.cs:70134-70151` (OOB catch → false; port `edit/kill.rs:2158`).
- The pass itself: `WorldGen.cs:18187-18236` (the `:18210` PlaceTile call, the `:18211-18222` success/valve, the `:18225-18232` catch arm).

---
*Produced by the g37 lane (code-only; no cargo run). d104b consumes §§2-4;
the gate re-runs §4's witnesses and re-pins the hashes once on the merged
chain.*
