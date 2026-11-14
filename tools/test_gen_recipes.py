#!/usr/bin/env python3
"""gen_recipes test suite — S1-L-11 acceptance checks.

Run:  python tools/test_gen_recipes.py   (exit 0 = all pass)

The reverse-recipe rule is re-derived here independently of
`gen_recipes.reverse_pass` (a straight transcription of
`Recipe.CreateReverseWallRecipes` / `CreateReversePlatformRecipes`,
`Recipe.cs:16678-16702` / `:16652-16676`), so a regression in the two-pass
insertion logic or in the C# item-data parse shows up as a mismatch against
the committed `recipes_gen.rs`.
"""

import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import gen_recipes  # noqa: E402

GOLDEN_AUTHORED = 2480   # 2 481 executed Add* sites - 1 skipped needTorchGodsFavor
GOLDEN_REVERSE = 311


def fail(msg):
    print("FAIL:", msg)
    sys.exit(1)


def check(cond, msg):
    if not cond:
        fail(msg)


def csharp_reverse_plan(rows, ctx):
    """Straight transcription of the C# derivation: walls first
    (`Recipe.cs:15148`), platforms second (`:15149`); each pass captures the
    table at entry and the flip lands right after its source."""
    plan = []
    cur = list(rows)
    for want_walls in (True, False):
        nxt = []
        for row in cur:
            nxt.append(row)
            if row is None or isinstance(row, tuple):
                continue
            slots = [s for s in row.required if s]
            if len(slots) != 1:
                continue   # `requiredItem[1].type == 0`
            src = slots[0]
            cid = row.create[0]
            if want_walls:
                # `createItem.createWall > 0 && requiredItem[0].createWall == -1`
                hit = ctx.cs_walls.get(cid, -1) > 0 and ctx.cs_walls.get(src[0], -1) == -1
            else:
                # `createItem.createTile >= 0 && TileID.Sets.Platforms[createTile]`
                t = ctx.cs_tiles.get(cid, -1)
                hit = t >= 0 and t in ctx.platforms
            if hit:
                nxt.append(("rev", cid, row.create[1], src[0], src[1], row.tile))
        cur = nxt
    plan = [x[1:] for x in cur if isinstance(x, tuple)]
    return plan


def main():
    ctx = gen_recipes.Ctx()
    check(not ctx.warnings, f"generator warnings: {ctx.warnings[:5]}")

    rows = gen_recipes.validate(gen_recipes.parse_all(ctx), ctx)
    check(len(rows) == GOLDEN_AUTHORED, f"authored rows {len(rows)} != {GOLDEN_AUTHORED}")

    out = gen_recipes.reverse_rows(rows, ctx)
    reverse_rows = [r for r in out if r.origin.endswith(" reverse")]
    check(len(reverse_rows) == GOLDEN_REVERSE, f"reverse rows {len(reverse_rows)} != {GOLDEN_REVERSE}")

    # The emitted reverse rows must equal the independently transcribed rule.
    plan = csharp_reverse_plan(rows, ctx)
    check(len(plan) == GOLDEN_REVERSE, f"rule-derived reverse {len(plan)} != {GOLDEN_REVERSE}")
    for row, (cid, cstack, item, istack, tile) in zip(reverse_rows, plan):
        check(
            row.create == [item, istack]
            and row.required == [[cid, cstack, None]]
            and row.tile == tile,
            f"reverse row mismatch: emitted {row.create}/{row.required} vs rule {item}x{istack}<-{cid}x{cstack}",
        )

    # Spot checks with the C# citation (a platform and a wall source).
    by_item = {(r.create[0], r.required[0][0]): r for r in reverse_rows}
    # FrozenPlatform (tile 19, a Sets.Platforms member) -> Ice Block.
    check((664, 3908) in by_item, "missing reverse: Ice Block <- 2 FrozenPlatform (tile 19)")
    # LivingLeafWall (createWall 60) -> Wood; the ingredient has no wall.
    check((9, 3584) in by_item, "missing reverse: Wood <- 4 LivingLeafWall (wall 60)")

    # The committed table must match a fresh run (the `--check` contract).
    text = gen_recipes.emit(out, ctx)
    committed = gen_recipes.OUT.read_text(encoding="utf-8")
    check(text == committed, "recipes_gen.rs is out of date - rerun tools/gen_recipes.py")

    print(f"ok: {GOLDEN_AUTHORED} authored + {GOLDEN_REVERSE} reverse rows")


if __name__ == "__main__":
    main()
