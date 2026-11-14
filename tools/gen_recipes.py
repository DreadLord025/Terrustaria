#!/usr/bin/env python
"""Generate `crates/tr-app/src/recipes_gen.rs` from the decompiled
`Terraria/Recipe.cs` `SetupRecipes` (the faithful 1.4.5.8 recipe table).

Sources (paths relative to the repo root):
  decompiled/Terraria/Recipe.cs      - SetupRecipes + the Add*Furniture helpers
  decompiled/Terraria/ID/ItemID.cs   - item names + `Sets.TextureCopyLoad`
  decompiled/Terraria/ID/TileID.cs   - tile constants (`SetCraftingStation(TileID.X)`)
                                       + `Sets.Platforms` (the reverse recipes)
  decompiled/Terraria/Item.cs        - the C#'s own item registry: the per-type
                                       placed tile/wall (`SetDefaults1..5`) that the
                                       reverse recipes derive from (`Item.cs:169/171`,
                                       `:48663`, `:49010`)
  crates/tr-app/src/itemdata.rs      - the port's item registry (id validation only)
  crates/tr-app/src/craft.rs         - the port's RecipeGroups table (validation)

Usage: python tools/gen_recipes.py [--check]
  --check verifies the generated file is up to date (exits 1 on drift).

Handled C# forms (everything else in the body is tail bookkeeping and skipped):
  createItem.SetDefaults(N) / createItem.stack = N|local
  requiredItem[i].SetDefaults(N) / .stack = N  (i = num | ++num | literal)
  SetIngredients(id, stack, ...)               - (id, stack) pairs
  RequireGroup(RecipeGroups.X)                 - bound to the slot whose authored
                                                 item is a group member
  requiredTile = N | SetCraftingStation(N | TileID.X)
  needWater/needHoney/needLava/needSnowBiome/needGraveyardBiome = true
  notDecraftable / corruption / crimson / AddCustomShimmerResult - ignored
    (decrafting/shimmer bookkeeping; corruption|crimson do NOT gate crafting -
    they only feed ItemID.Sets.IsCraftedCorruption/Crimson, `Recipe.cs:15252-15257`)
  needTorchGodsFavor / needMechdusa - the row is SKIPPED (no port system)
  for (int i = A; i <= B; i++) { ... }         - textual expansion
  int[,] array = new int[N, 2] { ... } + for (j < array.GetLength(0)) - expanded
  Recipe.AddCritterStatueRecipe(a[, b[, RecipeGroups.X]]) - synthesized rows
  Recipe.Add*Furniture()                       - the helper body spliced in place
  CreateReversePlatformRecipes/CreateReverseWallRecipes - emitted by this script
    (the C# derives them from the finished table + the per-item `createTile`/
    `createWall` of `Item.SetDefaults`, and inserts each reverse right after its
    source row, `Recipe.cs:15148-15149`, `:16648-16694`)
"""
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
# The decompiled tree is a local (untracked) reference — reached through the
# checkout's own `decompiled` junction (a fresh worktree creates it per
# AGENTS.md; a junction-less checkout fails loudly instead of silently
# reading another checkout's tree). S1-L-11e / research-21 D8.
DECOMP = ROOT / "decompiled" / "Terraria"
OUT = ROOT / "crates" / "tr-app" / "src" / "recipes_gen.rs"

FLAG_MAP = {
    "needWater": "need_water",
    "needHoney": "need_honey",
    "needLava": "need_lava",
    "needSnowBiome": "need_snow",
    "needGraveyardBiome": "need_graveyard",
}
SKIP_FLAGS = {"needTorchGodsFavor", "needMechdusa"}


def method_body(src, signature_re):
    """The `{...}` inner span of the first method matching `signature_re`."""
    m = re.search(signature_re, src)
    if not m:
        return None
    i = src.index("{", m.start())
    depth, j = 0, i
    while j < len(src):
        if src[j] == "{":
            depth += 1
        elif src[j] == "}":
            depth -= 1
            if depth == 0:
                return src[i + 1 : j]
        j += 1
    return None


def span_of(body, open_brace):
    """The index of the `}` closing the `{` at `open_brace`."""
    depth = 0
    for j in range(open_brace, len(body)):
        if body[j] == "{":
            depth += 1
        elif body[j] == "}":
            depth -= 1
            if depth == 0:
                return j
    return None


def snake_upper(name):
    return re.sub(r"(?<!^)(?=[A-Z])", "_", name).upper()


class Ctx:
    def __init__(self):
        self.warnings = []
        self.skipped = []   # (origin, reason)
        recipe_cs = (DECOMP / "Recipe.cs").read_text(encoding="utf-8")
        item_cs = (DECOMP / "ID" / "ItemID.cs").read_text(encoding="utf-8")
        tile_cs = (DECOMP / "ID" / "TileID.cs").read_text(encoding="utf-8")
        craft_rs = (ROOT / "crates" / "tr-app" / "src" / "craft.rs").read_text(encoding="utf-8")
        itemdata_rs = (ROOT / "crates" / "tr-app" / "src" / "itemdata.rs").read_text(encoding="utf-8")

        self.item_names = {}
        for m in re.finditer(r"public const (?:sbyte|short|ushort|int) (\w+) = (\d+);", item_cs):
            self.item_names.setdefault(int(m.group(2)), m.group(1))

        m = re.search(r"Sets\.TextureCopyLoad = Sets\.Factory\.CreateIntSet\((-?\d+), ([^)]*)\);", item_cs)
        self.tex_copy = {int(a): int(b) for a, b in re.findall(r"(\d+), (\d+)", m.group(2))} if m else {}

        self.tile_ids = {"None": -1}
        for m in re.finditer(r"public const ushort (\w+) = (\d+);", tile_cs):
            self.tile_ids[m.group(1)] = int(m.group(2))

        m = re.search(r"Sets\.Platforms = Sets\.Factory\.CreateBoolSet\(([^)]*)\)", tile_cs)
        self.platforms = {int(x) for x in re.findall(r"\d+", m.group(1))} if m else set()

        # The port's group table: c# field name -> (rust const, members).
        self.group_defs = {}     # c# name -> members
        self.group_consts = {}   # c# name -> rust const name
        rust_by_id = {}
        rust_syms = {}
        for m in re.finditer(r"pub(?:\(crate\))? const ([A-Z0-9_]+): u16 = (\d+);", craft_rs):
            rust_by_id[int(m.group(2))] = m.group(1)
            rust_syms[m.group(1)] = int(m.group(2))
        # The def rows come in two shapes: single-line with a numeric id and
        # multi-line with a symbolic id (`id: RecipeGroups::WOOD`).
        for m in re.finditer(r"RecipeGroupDef \{(.{0,400}?)\},\n", craft_rs, re.S):
            body = m.group(1)
            mid = re.search(r"id: (?:(\d+)|RecipeGroups::([A-Z0-9_]+))", body)
            mname = re.search(r'name: "([^"]+)"', body)
            mmembers = re.search(r"members: &\[([^\]]*)\]", body)
            if not (mid and mname and mmembers):
                continue
            gid = int(mid.group(1)) if mid.group(1) else rust_syms.get(mid.group(2), -1)
            members = [int(x) for x in re.findall(r"-?\d+", mmembers.group(1))]
            self.group_defs[mname.group(1)] = members
            if gid in rust_by_id:
                self.group_consts[mname.group(1)] = rust_by_id[gid]

        # The port's item registry: the id set only (emission validation).
        self.port_ids = {int(x) for x in re.findall(r"\bid: (\d+),", itemdata_rs)}

        # The C#'s own item registry: the per-type placed tile/wall the reverse
        # recipes derive from (`Item.SetDefaults`, `Item.cs:48663`). `ResetStats`
        # resets both to -1 (`:49010`) and dispatches to `SetDefaults1..5` by
        # type range; every assignment is a literal inside the per-type switch —
        # the only other `createTile`/`createWall` references in Item.cs are the
        # field declarations (`:169`/`:171`) and the reset.
        self.cs_tiles, self.cs_walls = {}, {}
        item_body_src = (DECOMP / "Item.cs").read_text(encoding="utf-8")
        # Deliberate parse gap (S1-L-11): `default:` switch sections are not
        # interpreted — emulating them needs the whole control flow of
        # SetDefaults (which types reach which nested switch). The only
        # uncovered ids are the monolith series 2192-2198
        # (`default: this.createTile = 300 + type - 2192;`, `Item.cs:27635`);
        # none of them is a single-ingredient recipe result, so the reverse
        # derivation is unaffected (checked by tools/test_gen_recipes.py).

        # The `DefaultTo*` helpers that place a tile/wall
        # (`DefaultToPlaceableTile`/`DefaultToPlaceableWall` take the id as a
        # parameter, `Item.cs:48402`/`:48492`; the rest assign literals). Their
        # case-side calls are resolved by interpreting the helper body with the
        # call's arguments bound to the parameters.
        self.default_helpers = {}
        for hm in re.finditer(
            r"(?:public|private) void (DefaultTo\w+)\(([^)]*)\)\s*\n\t\{", item_body_src
        ):
            hbody = method_body(item_body_src, re.escape(hm.group(0)))
            if hbody is None:
                continue
            if not re.search(r"this\.(createTile|createWall)\s*=", hbody) and not re.search(
                r"this\.(?:DefaultTo)\w+\(", hbody
            ):
                continue
            params = []
            for p in hm.group(2).split(","):
                pm = re.search(r"(\w+)\s*(?:=\s*([^,]+))?$", p.strip())
                if pm:
                    params.append((pm.group(1), pm.group(2)))
            self.default_helpers[hm.group(1)] = (params, hbody)

        for n in (1, 2, 3, 4, 5):
            body = method_body(
                item_body_src, rf"private void SetDefaults{n}\(int type\)\s*\n\t\{{"
            )
            if body is None:
                # A missing body would silently gut the reverse derivation.
                raise SystemExit(f"gen_recipes: Item.cs SetDefaults{n} not found")

            def apply_tile_call(line, iids, depth=0, parent_env=None):
                """Resolve a `this.DefaultTo*(...)` case-side call for every
                stacked case id; returns False for non-calls. Helper bodies may
                delegate (`DefaultToMonolith` -> `DefaultToPlaceableTile`,
                `Item.cs:47761-47768`), so the body is interpreted line by line
                with the call's arguments bound to the parameters."""
                mc = re.match(r"\t+this\.(DefaultTo\w+)\(", line)
                if not (mc and mc.group(1) in self.default_helpers):
                    return False
                if not line.rstrip().endswith(");"):
                    return False
                params, hbody = self.default_helpers[mc.group(1)]
                inner = line[line.index("(") + 1 : line.rindex(")")].strip()
                raw_args = [a.strip() for a in inner.split(",")] if inner else []
                if depth > 8:
                    raise SystemExit("gen_recipes: DefaultTo* delegation too deep")
                for iid in iids:
                    env = {"type": iid}
                    if parent_env:
                        env.update(parent_env)
                    for k, (pname, _dflt) in enumerate(params):
                        if k >= len(raw_args):
                            break
                        # `DefaultToTorch(0, allowWaterPlacement: true)` — drop
                        # the C# named-argument label.
                        raw = re.sub(r"^\w+:\s*", "", raw_args[k])
                        if raw in ("true", "false"):
                            env[pname] = raw   # bool parameters never reach the tile/wall math
                        else:
                            env[pname] = eval_case_expr(raw, iid, self, f"Item.cs case {iid} arg", env)
                    # C# parameter defaults fill the omitted trailing args
                    # (`DefaultToPlaceableTile(ushort tile, int style = 0)`).
                    for k, (pname, dflt) in enumerate(params):
                        if pname not in env and dflt is not None:
                            if dflt in ("true", "false"):
                                env[pname] = dflt
                            else:
                                env[pname] = eval_case_expr(dflt, iid, self, f"{mc.group(1)} default", env)
                    for hline in hbody.splitlines():
                        ma = re.fullmatch(r"\t+this\.(createTile|createWall) = (.+);", hline)
                        if ma:
                            v = eval_expr(ma.group(2), env, self, f"Item.cs case {iid} {mc.group(1)}")
                            if v is not None:
                                table = self.cs_tiles if ma.group(1) == "createTile" else self.cs_walls
                                table[iid] = int(v)
                            continue
                        if apply_tile_call(hline, [iid], depth + 1, parent_env=env):
                            continue
                return True

            # `if (type >= A && type <= B) { ... this.createTile = ...;
            # return; }` blocks ahead of the switch — one assignment covers
            # the whole range (`Item.cs:18299`, `:22786`).
            for im in re.finditer(r"if \(type >= (\d+) && type <= (\d+)\)\s*\n\t+\{", body):
                ob = body.index("{", im.start())
                cb = span_of(body, ob)
                ibody = body[ob + 1 : cb]
                lo_i, hi_i = int(im.group(1)), int(im.group(2))
                rng = range(lo_i, hi_i + 1)
                for line in ibody.splitlines():
                    if apply_tile_call(line, rng):
                        continue
                    ma = re.fullmatch(r"\t+this\.(createTile|createWall) = (.+);", line)
                    if ma:
                        table = self.cs_tiles if ma.group(1) == "createTile" else self.cs_walls
                        for iid in rng:
                            v = eval_case_expr(ma.group(2), iid, self, f"Item.cs if-range {iid}")
                            if v is not None:
                                table[iid] = int(v)

            case_ids = []   # the current stacked `case a: case b:` label group
            for line in body.splitlines():
                m = re.fullmatch(r"\t*case (-?\d+):", line)
                if m:
                    case_ids.append(int(m.group(1)))
                    continue
                if re.fullmatch(r"\t*default:", line):
                    case_ids = []   # applies to every unmatched type; skip
                    continue
                # A C# switch section cannot fall through into the next one:
                # every non-empty section ends with `break` (or `return`),
                # which closes the stacked label group.
                if re.fullmatch(r"\t*(break|return);", line):
                    case_ids = []
                    continue
                # `this.DefaultToPlaceableWall(27);` and friends.
                if apply_tile_call(line, case_ids):
                    continue
                m = re.fullmatch(r"\t+this\.(createTile|createWall) = (.+);", line)
                if m and case_ids:
                    table = self.cs_tiles if m.group(1) == "createTile" else self.cs_walls
                    for iid in case_ids:
                        # Series items assign from `type`
                        # (`this.createWall = 116 + type - 1948;`), so the
                        # expression is evaluated per stacked case id.
                        v = eval_case_expr(m.group(2), iid, self, f"Item.cs case {iid}")
                        if v is not None:
                            table[iid] = int(v)

        self.helper_bodies = {}
        for name in set(re.findall(r"private static void (Add\w+)\(\)", recipe_cs)):
            self.helper_bodies[name] = method_body(
                recipe_cs, rf"private static void {name}\(\)\s*\n\t\{{"
            )
        self.recipe_cs = recipe_cs


# ---------------------------------------------------------------- row parsing

class Row:
    __slots__ = ("create", "required", "tile", "flags", "origin")

    def __init__(self, origin):
        self.create = None        # [id, stack]
        self.required = []        # [ [item, stack, group|None] | None, ... ]
        self.tile = None
        self.flags = {}           # rust field name -> True
        self.origin = origin


def eval_expr(expr, env, ctx, origin):
    """A restricted C# int expression: literals, locals, `+`/`-`/`*`, casts,
    the `2.5f` float literals, the `ItemID.Sets.TextureCopyLoad[i]` lookup."""
    expr = expr.strip()
    for _ in range(4):
        m = re.fullmatch(r"\(int\)\((.*)\)", expr)
        if not m:
            break
        expr = m.group(1).strip()
    # Peel one balanced pair of outer parentheses per round
    # (`430 + (type - 3633)` parses its right side through here).
    for _ in range(4):
        if expr.startswith("(") and expr.endswith(")"):
            depth = 0
            balanced = True
            for i, ch in enumerate(expr):
                if ch == "(":
                    depth += 1
                elif ch == ")":
                    depth -= 1
                    if depth == 0 and i != len(expr) - 1:
                        balanced = False
                        break
            if balanced and depth == 0:
                expr = expr[1:-1].strip()
                continue
        break
    m = re.fullmatch(r"\((?:float|int)\)\(?([^()]*)\)?", expr)
    if m:
        return eval_expr(m.group(1).strip(), env, ctx, origin)
    m = re.fullmatch(r"([\d.]+)f", expr)
    if m:
        return float(m.group(1))
    if re.fullmatch(r"-?\d+", expr):
        return int(expr)
    m = re.fullmatch(r"ItemID\.Sets\.TextureCopyLoad\[(\w+)\]", expr)
    if m:
        key = eval_expr(m.group(1), env, ctx, origin)
        return ctx.tex_copy.get(key, -1)
    m = re.fullmatch(r"(.+?) \* (.+)", expr)
    if m:
        a = eval_expr(m.group(1), env, ctx, origin)
        b = eval_expr(m.group(2), env, ctx, origin)
        if a is None or b is None:
            return None
        v = a * b
        return int(v) if isinstance(v, float) else v
    m = re.fullmatch(r"(.+) ([+-]) (.+?)", expr)
    if m:
        a = eval_expr(m.group(1), env, ctx, origin)
        b = eval_expr(m.group(3), env, ctx, origin)
        return None if a is None or b is None else (a + b if m.group(2) == "+" else a - b)
    if expr in env and isinstance(env[expr], int):
        return env[expr]
    ctx.warnings.append(f"{origin}: cannot evaluate `{expr}`")
    return None


def peel_numeric_casts(s):
    """`(ushort)726` -> `726` — the helper-call arguments carry explicit casts."""
    return re.sub(r"\((?:sbyte|byte|short|ushort|int|uint)\)\s*", "", s).strip()


def eval_case_expr(expr, iid, ctx, origin, base_env=None):
    """Evaluate one `createTile`/`createWall` right-hand side for case id
    `iid`: casts are peeled and `(type - N)` is rewritten `type - N` (the
    series forms in Item.cs); `+`/`-` are left-associative, so dropping the
    parentheses is identity. `base_env` carries the enclosing helper's
    parameters through a delegated call."""
    expr = peel_numeric_casts(expr)
    expr = re.sub(r"\(\s*type([^()]*)\)", r"type\1", expr)
    env = {"type": iid}
    if base_env:
        env.update(base_env)
    return eval_expr(expr, env, ctx, origin)


def slot_index(spec, env, ctx, origin):
    """`requiredItem[<spec>]`: a literal, a local, or `++local`."""
    spec = spec.strip()
    m = re.fullmatch(r"\+\+(\w+)", spec)
    if m:
        env[m.group(1)] = env.get(m.group(1), -1) + 1
        return env[m.group(1)]
    if re.fullmatch(r"\d+", spec):
        return int(spec)
    if spec in env and isinstance(env[spec], int):
        return env[spec]
    ctx.warnings.append(f"{origin}: cannot index requiredItem[{spec}]")
    return None


def resolve_tile(expr, env, ctx, origin):
    m = re.fullmatch(r"TileID\.(\w+)", expr.strip())
    if m:
        return ctx.tile_ids.get(m.group(1), None)
    return eval_expr(expr, env, ctx, origin)


def parse_block(text, env, ctx, origin):
    """Parse one AddRecipe block into a Row (or None)."""
    row = Row(origin)
    for raw in text.splitlines():
        line = raw.strip()
        if not line or line.startswith("//"):
            continue
        m = re.match(r"(?:int )?(\w+) = (-?\d+);$", line)
        if m:
            env[m.group(1)] = int(m.group(2))
            continue
        m = re.match(r"Recipe\.currentRecipe\.createItem\.SetDefaults\((.+)\);$", line)
        if m:
            v = eval_expr(m.group(1), env, ctx, origin)
            row.create = [v, 1]
            continue
        m = re.match(r"Recipe\.currentRecipe\.createItem\.stack = (.+);$", line)
        if m:
            if row.create:
                row.create[1] = eval_expr(m.group(1), env, ctx, origin)
            continue
        m = re.match(r"Recipe\.currentRecipe\.requiredItem\[(\w+)\]\.SetDefaults\((.+)\);$", line)
        if m:
            i = slot_index(m.group(1), env, ctx, origin)
            if i is not None:
                while len(row.required) <= i:
                    row.required.append(None)
                row.required[i] = [eval_expr(m.group(2), env, ctx, origin), 1, None]
            continue
        m = re.match(r"Recipe\.currentRecipe\.requiredItem\[(\w+)\]\.stack = (.+);$", line)
        if m:
            i = slot_index(m.group(1), env, ctx, origin)
            if i is not None and i < len(row.required) and row.required[i]:
                row.required[i][1] = eval_expr(m.group(2), env, ctx, origin)
            continue
        m = re.match(r"Recipe\.currentRecipe\.SetIngredients\(([^)]*)\);$", line)
        if m:
            vals = [eval_expr(p, env, ctx, origin) for p in m.group(1).split(",")]
            if None in vals:
                continue
            # `SetIngredients(params)`: a lone id means (id, 1)
            # (`Recipe.cs:16696-16702`).
            if len(vals) == 1:
                vals = [vals[0], 1]
            if len(vals) % 2:
                ctx.warnings.append(f"{origin}: odd SetIngredients {vals}")
                continue
            for k in range(0, len(vals), 2):
                while len(row.required) <= k // 2:
                    row.required.append(None)
                row.required[k // 2] = [vals[k], vals[k + 1], None]
            continue
        m = re.match(r"Recipe\.currentRecipe\.RequireGroup\(RecipeGroups\.(\w+)\);$", line)
        if m:
            row.flags.setdefault("_groups", []).append(m.group(1))
            continue
        m = re.match(r"Recipe\.currentRecipe\.SetCraftingStation\(([^)]*)\);$", line)
        if m:
            row.tile = resolve_tile(m.group(1), env, ctx, origin)
            continue
        m = re.match(r"Recipe\.currentRecipe\.requiredTile = (.+);$", line)
        if m:
            row.tile = resolve_tile(m.group(1), env, ctx, origin)
            continue
        if re.match(r"Recipe\.currentRecipe\.alchemy = true;$", line):
            row.flags["alchemy"] = True
            continue
        m = re.match(r"Recipe\.currentRecipe\.(needWater|needHoney|needLava|needSnowBiome|needGraveyardBiome) = true;$", line)
        if m:
            row.flags[FLAG_MAP[m.group(1)]] = True
            continue
        m = re.match(r"Recipe\.currentRecipe\.(needTorchGodsFavor|needMechdusa) = true;$", line)
        if m:
            ctx.skipped.append((origin, m.group(1)))
            return None
        if re.match(
            r"Recipe\.currentRecipe\.(notDecraftable|corruption|crimson|acceptedGroups)"
            r"|Recipe\.currentRecipe\.AddCustomShimmerResult|Recipe\.currentRecipe\.Shimmer",
            line,
        ):
            continue
        if re.match(r"Recipe\.currentRecipe\.", line):
            ctx.warnings.append(f"{origin}: unparsed `{line[:90]}`")
    # Bind each group to the first unclaimed slot whose authored item is a
    # member (`CreateRequiredItemQuickLookups`, `Recipe.cs:15162-15191`).
    for gname in row.flags.pop("_groups", []):
        rust = ctx.group_consts.get(gname)
        members = ctx.group_defs.get(gname, [])
        if rust is None or not members:
            ctx.warnings.append(f"{origin}: unknown group `{gname}`")
            continue
        placed = False
        for slot in row.required:
            if slot and slot[2] is None and slot[0] in members:
                slot[2] = rust
                placed = True
                break
        if not placed:
            # An unauthored slot (the statue placeholder 0) takes the group's
            # placeholder item (`RecipeGroup.GetPlaceholderItemType`).
            for slot in row.required:
                if slot and slot[2] is None and slot[0] in (0, None):
                    slot[0] = members[0]
                    slot[2] = rust
                    placed = True
                    break
        # An unbound group is the C# semantics (`CreateRequiredItemQuickLookups`
        # binds an acceptedGroup only when the entry's type is a member) - the
        # requirement stays the exact authored item.
    return row


def statue_row(statue, critter, group, ctx):
    """`AddCritterStatueRecipe(statueItem[, critterItem[, RecipeGroups.X]])`
    (`Recipe.cs:16630-16646`): 50 Stone + 5 of the critter, at tile 283
    (the graveyard statuary), needGraveyardBiome."""
    row = Row(f"AddCritterStatueRecipe({statue}, {critter})")
    row.create = [statue, 1]
    row.required = [[3, 50, None]]
    if group:
        gname = group.strip().replace("RecipeGroups.", "")
        rust = ctx.group_consts.get(gname)
        members = ctx.group_defs.get(gname, [])
        if rust and members:
            row.required.append([members[0], 5, rust])
        else:
            ctx.warnings.append(f"statue {statue}: unknown group {group}")
            row.required.append([critter, 5, None])
    else:
        row.required.append([critter, 5, None])
    row.tile = 283
    row.flags["need_graveyard"] = True
    return row


# ------------------------------------------------------------ body expansion

FOR_RE = re.compile(r"for \(int (\w+) = (-?\d+); \1 <= (\d+); \1\+\+\)\s*\n\s*\{")
FOR_LT_RE = re.compile(r"for \(int (\w+) = (-?\d+); \1 < (\d+); \1\+\+\)\s*\n\s*\{")
FOR_ARR_RE = re.compile(r"for \(int (\w+) = 0; \1 < array\.GetLength\(0\); \1\+\+\)\s*\n\s*\{")
ARRAY_RE = re.compile(r"int\[,\] array = new int\[\d+, ?2\]\s*\n\s*\{(.*?)\n\s*\};", re.S)
STATUE_RE = re.compile(r"Recipe\.AddCritterStatueRecipe\(([^)]*)\);")
HELPER_RE = re.compile(r"Recipe\.(Add(?!Recipe)\w+)\(\);")
TAIL_RE = re.compile(r"Recipe\.CreateReverseWallRecipes\(\);")


def expand_body(body, ctx, origin):
    """Expand loops, the int[,] table, the statue calls and the helper calls
    into flat AddRecipe block text. Returns the expanded text."""
    arr = ARRAY_RE.search(body)
    array_rows = []
    if arr:
        for pair in re.findall(r"\{ ?(-?\d+), ?(-?\d+) ?\}", arr.group(1)):
            array_rows.append((int(pair[0]), int(pair[1])))

    out = []
    i = 0
    while i < len(body):
        tail = TAIL_RE.search(body, i)
        f = FOR_RE.search(body, i)
        flt = FOR_LT_RE.search(body, i)
        if flt and (not f or flt.start() < f.start()):
            f = flt
        fa = FOR_ARR_RE.search(body, i)
        st = STATUE_RE.search(body, i)
        hp = HELPER_RE.search(body, i)
        marks = [(m.start(), kind) for m, kind in ((tail, "tail"), (f, "for"), (fa, "forarr"), (st, "statue"), (hp, "helper")) if m]
        if not marks:
            out.append(body[i:])
            break
        pos, kind = min(marks)
        if kind != "tail":
            out.append(body[i:pos])
        if kind == "tail":
            break  # the C# tail derives the reverse recipes (this script emits them itself)
        if kind == "for":
            var, lo, hi = f.group(1), int(f.group(2)), int(f.group(3))
            close = span_of(body, f.end() - 1)
            if close is None or hi - lo > 200:
                ctx.warnings.append(f"{origin}: unexpanded loop {var} {lo}..{hi}")
                i = f.end()
                continue
            inner = body[f.end() + 1 : close]
            for v in range(lo, hi + 1):
                out.append(f"int {var} = {v};\n{inner}\n")
            i = close + 1
        elif kind == "forarr":
            var = fa.group(1)
            close = span_of(body, fa.end() - 1)
            if close is None:
                ctx.warnings.append(f"{origin}: unexpanded array loop")
                i = fa.end()
                continue
            inner = body[fa.end() + 1 : close]
            for r, (a0, a1) in enumerate(array_rows):
                rep = re.sub(rf"array\[{var}, 0\]", str(a0), inner)
                rep = re.sub(rf"array\[{var}, 1\]", str(a1), rep)
                out.append(f"int {var} = {r};\n{rep}\n")
            i = close + 1
        elif kind == "statue":
            args = [x.strip() for x in st.group(1).split(",")]
            statue, critter = int(args[0]), int(args[1])
            group = args[2] if len(args) > 2 else None
            out.append(statue_text(statue, critter, group))
            i = st.end()
        else:  # helper
            name = hp.group(1)
            hbody = ctx.helper_bodies.get(name)
            if hbody:
                out.append(expand_body(hbody, ctx, name))
            else:
                ctx.warnings.append(f"{origin}: helper `{name}` body not found")
            i = hp.end()
    return "".join(out)


def statue_text(statue, critter, group):
    lines = [
        f"Recipe.currentRecipe.createItem.SetDefaults({statue});",
        "Recipe.currentRecipe.requiredItem[0].SetDefaults(3);",
        "Recipe.currentRecipe.requiredItem[0].stack = 50;",
    ]
    if group:
        lines.append("Recipe.currentRecipe.requiredItem[1].SetDefaults(0);")
        lines.append(f"Recipe.currentRecipe.RequireGroup({group});")
    if not group:
        lines.append(f"Recipe.currentRecipe.requiredItem[1].SetDefaults({critter});")
    lines.append("Recipe.currentRecipe.requiredItem[1].stack = 5;")
    lines.append("Recipe.currentRecipe.requiredTile = 283;")
    lines.append("Recipe.currentRecipe.needGraveyardBiome = true;")
    lines.append("Recipe.AddRecipe();")
    return "\n".join(lines) + "\n"


def parse_all(ctx):
    """The full ordered row list (the C# registration order)."""
    start = ctx.recipe_cs.index("public static void SetupRecipes()")
    m = re.search(r"public static void SetupRecipes\(\)\s*\n\t\{", ctx.recipe_cs[start:])
    body = method_body(ctx.recipe_cs[start:], r"public static void SetupRecipes\(\)\s*\n\t\{")
    expanded = expand_body(body, ctx, "SetupRecipes")

    rows = []
    env = {}
    for chunk in expanded.split("Recipe.AddRecipe();"):
        if "Recipe.currentRecipe.createItem" not in chunk:
            # bookkeeping or empty chunk
            if re.search(r"Recipe\.currentRecipe\.", chunk):
                ctx.warnings.append(f"SetupRecipes: chunk without createItem: {chunk.strip()[:80]}")
            continue
        # The env is SHARED across chunks - the C# locals (`int type = 3234;`)
        # are declared between the AddRecipe calls and used by later blocks.
        row = parse_block(chunk, env, ctx, "SetupRecipes")
        if row is not None and row.create and row.create[0] is not None:
            rows.append(row)
    return rows


def validate(rows, ctx):
    out = []
    for row in rows:
        cid, stack = row.create
        if cid not in ctx.port_ids:
            ctx.skipped.append((f"row create {cid} ({ctx.item_names.get(cid, '?')}) from {row.origin}", "id not in itemdata"))
            continue
        ok = True
        for slot in row.required:
            if slot is None:
                continue
            item, _stack, group = slot
            if item not in ctx.port_ids:
                ctx.skipped.append((f"row create {cid} ingredient {item} from {row.origin}", "id not in itemdata"))
                ok = False
                break
        if not ok:
            continue
        if row.tile is None or row.tile < 0:
            row.tile = None
        out.append(row)
    return out


def reverse_rows(rows, ctx):
    """`CreateReverseWallRecipes` then `CreateReversePlatformRecipes` — the C#
    call order (`Recipe.cs:15148-15149`). Each pass captures the table at its
    entry (`int num = Recipe.numRecipes`) and inserts the flipped recipe right
    after its source (`Main.recipe[i + 1]`, `notDecraftable` — irrelevant to the
    craft direction). The conditions are the C#'s own, over the C# item data:
    walls  `createItem.createWall > 0 && requiredItem[1].type == 0 &&
            requiredItem[0].createWall == -1`   (`Recipe.cs:16681-16683`),
    platforms `createItem.createTile >= 0 && TileID.Sets.Platforms[createTile] &&
            requiredItem[1].type == 0`          (`Recipe.cs:16655-16657`).
    A reverse row never re-triggers either pass: its createItem is the source's
    ingredient (`createWall == -1`, and `createTile` not a platform by the
    source row's own condition)."""
    def reverse_pass(rows, want_walls):
        out = []
        for row in rows:
            out.append(row)
            # `requiredItem[1].type == 0` - exactly one ingredient slot.
            if len([s for s in row.required if s]) != 1:
                continue
            src_slot = next(s for s in row.required if s)
            cid = row.create[0]
            hit = False
            if want_walls:
                hit = (
                    ctx.cs_walls.get(cid, -1) > 0
                    and ctx.cs_walls.get(src_slot[0], -1) == -1
                )
            else:
                tile = ctx.cs_tiles.get(cid, -1)
                hit = tile >= 0 and tile in ctx.platforms
            if hit:
                rev = Row(row.origin + " reverse")
                rev.create = [src_slot[0], src_slot[1]]
                rev.required = [[cid, row.create[1], None]]
                rev.tile = row.tile
                # `Main.recipe[i + 1].notDecraftable = true` (`Recipe.cs:16665`).
                rev.flags["not_decraftable"] = True
                out.append(rev)
        return out

    rows = reverse_pass(rows, want_walls=True)      # Recipe.cs:15148
    rows = reverse_pass(rows, want_walls=False)     # Recipe.cs:15149
    return rows


def emit(rows, ctx):
    lines = [
        "// GENERATED by `tools/gen_recipes.py` from the decompiled",
        "// `Terraria/Recipe.cs` `SetupRecipes` (1.4.5.8) - do not edit by hand;",
        "// rerun `python tools/gen_recipes.py`. The rows are in the C#",
        "// registration order (the UI listing order). The file is ONE",
        "// expression (`&[Recipe, ...]`) so `include!` can splice it into the",
        "// `RECIPES` static.",
        "&[",
    ]
    for n, row in enumerate(rows):
        cid, stack = row.create
        name = ctx.item_names.get(cid, f"Item{cid}")
        src = f", {row.origin}" if row.origin != "SetupRecipes" else ""
        lines.append(f"// {n}: {name} x{stack}{src}")
        req = []
        for slot in row.required:
            if slot is None:
                continue
            item, istack, group = slot
            if group:
                req.append(f"Requirement::group({item}, RecipeGroups::{group}, {istack})")
            else:
                req.append(f"Requirement::item({item}, {istack})")
        req_s = ", ".join(req) if req else ""
        lines.append("Recipe {")
        lines.append(f"    create: ({cid}, {stack}),")
        lines.append(f"    required: &[{req_s}],")
        if row.tile is not None:
            lines.append(f"    stations: &[{row.tile}],")
        for f in ("need_water", "need_honey", "need_lava", "need_snow", "need_graveyard"):
            if row.flags.get(f):
                lines.append(f"    {f}: true,")
        # `AddRecipe` auto-sets alchemy iff the station is the placed bottle
        # tile 13 (`Recipe.cs:16720-16724`); some rows set it explicitly.
        if row.tile == 13:
            row.flags["alchemy"] = True
        if row.flags.get("alchemy"):
            lines.append("    alchemy: true,")
        if row.flags.get("not_decraftable"):
            lines.append("    not_decraftable: true,")
        lines.append("    ..BASE")
        lines.append("},")
    lines.append("]")
    return "\n".join(lines) + "\n"


def main():
    check = "--check" in sys.argv
    ctx = Ctx()
    rows = parse_all(ctx)
    rows = validate(rows, ctx)
    rows = reverse_rows(rows, ctx)
    text = emit(rows, ctx)

    if check:
        current = OUT.read_text(encoding="utf-8") if OUT.exists() else ""
        if current != text:
            print("recipes_gen.rs is out of date - rerun tools/gen_recipes.py")
            return 1
        print(f"check ok ({len(rows)} rows)")
        return 0

    OUT.write_text(text, encoding="utf-8", newline="\n")
    print(f"wrote {OUT} ({len(rows)} rows)")
    print(f"skipped: {len(ctx.skipped)}")
    for origin, why in ctx.skipped[:25]:
        print(f"  - {why}: {origin}")
    if len(ctx.skipped) > 25:
        print(f"  ... and {len(ctx.skipped) - 25} more")
    print(f"warnings: {len(ctx.warnings)}")
    for w in ctx.warnings[:25]:
        print(f"  ! {w}")
    if len(ctx.warnings) > 25:
        print(f"  ... and {len(ctx.warnings) - 25} more")
    return 0


if __name__ == "__main__":
    sys.exit(main())
