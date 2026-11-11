"""Generate `crates/tr-app/src/itemdata.rs` from the decompiled Terraria sources.

The vanilla item defaults are `Item.SetDefaults` (dispatcher `Item.cs:48663-48926`),
`ResetStats` (`Item.cs:48944-49051`, the stock defaults), the five case tables
`SetDefaults1-5` (~40 000 lines of case data), `SetFoodDefaults`
(`Item.cs:48040-48393`), the `DefaultTo*`/`SetShopValues` helpers
(`Item.cs:27651-48652`), the dispatcher epilogue (`Item.cs:48713-48925`),
`netDefaults` (`Item.cs:1211-1367`), `OnCreated` (`Item.cs:48936-48942`) and
`ItemVariants` (`GameContent/Items/ItemVariants.cs:109-146`). They are extracted
mechanically here so the Rust port can never drift from the decompiled source.
`itemdata.rs` is generated: never hand-edit it, re-run this script instead
(`python tools/GenItemData/gen.py` from the repo root); a second run must produce
an empty diff.

Supported statement forms (the parser fails loudly on anything it does not know,
naming `Item.cs:<line>`, so a source change cannot be silently dropped):

  * `this.FIELD = RHS;`                          field assignment
      RHS: int / float (`5f`, `0.9f`) / bool literal
           `Item.sellPrice(p, g, s, c)` / `Item.buyPrice(...)`   (`Item.cs:49417-49425`)
           `this.<namedPrice>` (readonly price fields, `Item.cs:71-79`)
           `this.OTHER_FIELD` (copy of the row's current value)
           `ItemID.X` / `AmmoID.X` / `ProjectileID.X` / `BuffID.X` /
             `TileID.X` / `WallID.X` id constants
           arithmetic over `type`, locals and the above, with C# casts
             `(int)/(ushort)/(short)/(byte)/(sbyte)` (`(int)` = trunc toward 0)
           `(int)((double)this.F * k)`                           relative multiply
           `(cond).ToInt()`                                      1/0
           chained `(this.F = RHS)`
  * `this.FIELD += / -= / *= / /= N;`, `this.FIELD++;`, `this.FIELD--;`
  * `this.DefaultTo*(args)` / `this.SetShopValues(...)` /
      `this.SetWeaponValues(...)` / `this.MakeUsableWithChlorophyteExtractinator()`
      — helper calls, ported below from their C# bodies
  * `this.SetDefaultsN(arg);`                    nested case-table run (no reset)
  * `int NAME = EXPR;` / `NAME = EXPR;`          locals (literals, `type`, loop vars)
  * `if (COND) { ... } else if (...) { ... } else { ... }`
      COND: comparisons (`== != < > <= >=`) over `type` / `this.FIELD` / locals,
      combined with `&&` / `||`; epilogue-specific conditions are dispatched
      through SPECIAL_CONDS by exact source text
  * `switch (type) { case N: ... case M: ... break; default: ... }`
      (nested switches allowed; several labels may share one body)
  * `break;` / `return;` / `goto case N;`
  * `if (this.Variant ==|!= ItemVariants.XVariant) { assignments }`
      recorded as `VARIANT_OVERRIDES` rows instead of applied (base rows are the
      no-variant state; `!=` bodies bake into the base row plus a restore row)

Skipped statements (fields not represented in `ItemDef`) are whitelisted
explicitly with a reason — see THIS_SKIP_FIELDS / SKIP_CALLS below.

External tables read: `ID/ItemID.cs` (names, `IsFood`/`IsDrill`/`IsChainsaw`/
`Deprecated`/`TextureCopyLoad` sets), `ID/AmmoID.cs`, `ID/ProjectileID.cs`,
`ID/BuffID.cs`, `ID/TileID.cs` (`Platforms`), `ID/WallID.cs`,
`GameContent/Items/ItemVariants.cs`, `Recipe.cs` (`SetupRecipeGroups` +
`SetupRecipes`, the runtime source of `ItemID.Sets.IsAMaterial`,
Recipe.cs:15205-15239), `Projectile.cs` (`aiStyle = 7` ⇒ `Main.projHook`,
Main.cs:6612-6624).
"""

from pathlib import Path
import re

ROOT = Path(__file__).resolve().parent.parent.parent
DEC = ROOT / "decompiled" / "Terraria"
OUT = ROOT / "crates" / "tr-app" / "src" / "itemdata.rs"

ITEM_CS = DEC / "Item.cs"
ITEMID_CS = DEC / "ID" / "ItemID.cs"
AMMOID_CS = DEC / "ID" / "AmmoID.cs"
PROJECTILEID_CS = DEC / "ID" / "ProjectileID.cs"
BUFFID_CS = DEC / "ID" / "BuffID.cs"
TILEID_CS = DEC / "ID" / "TileID.cs"
WALLID_CS = DEC / "ID" / "WallID.cs"
RARITY_CS = DEC / "Enums" / "ItemRarityColor.cs"


def parse_rarity_enum():
    """`AmberMinus11 = -11,` → {name: value} (decompiled/Terraria/Enums/ItemRarityColor.cs)."""
    consts = {}
    for i, line in enumerate(read_lines(RARITY_CS)):
        m = re.match(r"\t([A-Za-z0-9_]+) = (-?\d+),", line)
        if m:
            consts[m.group(1)] = int(m.group(2))
    if not consts:
        fail(str(RARITY_CS), 1, "no ItemRarityColor values found")
    return consts
PROJECTILE_CS = DEC / "Projectile.cs"
RECIPE_CS = DEC / "Recipe.cs"
ITEMVARIANTS_CS = DEC / "GameContent" / "Items" / "ItemVariants.cs"

ITEM_COUNT = 6196  # ItemID.cs:14861 — re-verified in main()


def fail(src, line, msg):
    raise SystemExit("GenItemData: %s:%d: %s" % (src, line, msg))


def read_lines(path):
    return path.read_text(encoding="utf-8").replace("\r\n", "\n").split("\n")


# ---------------------------------------------------------------------------
# Source tables
# ---------------------------------------------------------------------------

def parse_id_consts(path, cls):
    """`public const <int|short|ushort|byte> NAME = VALUE;` → {name: value}.

    AmmoID.cs declares plain static ints filled in its static constructor
    (`AmmoID.Arrow = 40;`) — that form is parsed too.
    """
    consts = {}
    pat = re.compile(r"public const (?:int|short|ushort|byte|sbyte) (\w+) = (-?\d+);")
    for i, line in enumerate(read_lines(path)):
        m = pat.match(line.strip())
        if m:
            consts[m.group(1)] = int(m.group(2))
    if not consts:
        pat2 = re.compile(r"%s\.(\w+) = (\d+);" % re.escape(cls))
        for i, line in enumerate(read_lines(path)):
            m = pat2.match(line.strip())
            if m:
                consts[m.group(1)] = int(m.group(2))
    if not consts:
        fail(str(path), 1, "no const ids found for " + cls)
    return consts


def parse_bool_set(itemid_text, expr, src):
    """`Sets.NAME = Sets.Factory.CreateBoolSet(a, b, ...);` → set of ids."""
    m = re.search(re.escape(expr) + r" = Sets\.Factory\.CreateBoolSet\(([0-9,\s]*)\);", itemid_text)
    if not m:
        fail(src, 1, "cannot find bool set " + expr)
    return {int(x) for x in m.group(1).split(",") if x.strip()}


def parse_int_set_values(itemid_text, expr, src):
    """`Sets.NAME = Sets.Factory.CreateIntSet(default, k1, v1, ...);` → values ≥ 0."""
    m = re.search(re.escape(expr) + r" = Sets\.Factory\.CreateIntSet\(\s*(-?\d+)\s*,([0-9,\s]*)\);",
                  itemid_text)
    if not m:
        fail(src, 1, "cannot find int set " + expr)
    nums = [int(x) for x in m.group(2).split(",") if x.strip()]
    if len(nums) % 2:
        fail(src, 1, "odd entry count in int set " + expr)
    return [v for (k, v) in zip(nums[0::2], nums[1::2]) if v >= 0]


def price_value(kind, args, src, line):
    """Port of `Item.sellPrice`/`Item.buyPrice` (Item.cs:49417-49425).

    buyPrice(p, g, s, c) = c + s*100 + g*10000 + p*1000000; sellPrice × 5.
    Trailing arguments default to 0.
    """
    args = list(args) + [0] * (4 - len(args))
    if len(args) > 4:
        fail(src, line, "price call with more than 4 args")
    p, g, s, c = args
    base = c + s * 100 + g * 10000 + p * 1000000
    return base * 5 if kind == "sell" else base


def parse_named_prices(item_lines):
    """`private readonly int NAME = Item.sellPrice(...);` (Item.cs:71-79) → values."""
    prices = {}
    pat = re.compile(r"private readonly int (\w+) = Item\.(sell|buy)Price\(([\d,\s]*)\);")
    for i, line in enumerate(item_lines[:200]):
        m = pat.match(line.strip())
        if m:
            args = [int(x) for x in m.group(3).split(",") if x.strip()]
            prices[m.group(1)] = price_value(m.group(2), args, "Item.cs", i + 1)
    if not prices:
        fail("Item.cs", 1, "no named price fields found")
    return prices


# ---------------------------------------------------------------------------
# C# statement parsing (line-based, brace-matched)
#
# Statement tuples:
#   ('assign', line, field, rhs)          this.F = rhs
#   ('compound', line, field, op, rhs)    this.F op= rhs  /  ++ / --
#   ('call', line, name, args, named)     helper call
#   ('sdcall', line, method, arg)         this.SetDefaultsN(arg)
#   ('local', line, name, rhs)            int name = rhs
#   ('localassign', line, name, rhs)      name = rhs
#   ('if', line, cond, body, else_body)
#   ('switch', line, casemap, default)
#   ('break', line) | ('return', line) | ('goto', line, target)
#   ('skip', line, what, why)
# ---------------------------------------------------------------------------

ASSIGN_RE = re.compile(r"this\.(\w+)\s*=\s*(.+)$")
COMPOUND_RE = re.compile(r"this\.(\w+)\s*(\+=|-=|\*=|/=)\s*(.+)$")
INCDEC_RE = re.compile(r"this\.(\w+)\s*(\+\+|--)$")
LOCAL_RE = re.compile(r"(?:int|short|ushort|byte|sbyte|float)\s+(\w+)\s*=\s*(.+)$")
LOCAL_ASSIGN_RE = re.compile(r"(\w+)\s*=\s*(.+)$")
CALL_RE = re.compile(r"this\.(\w+)\((.*)$")


def parse_call_args(argstr):
    """Split a C# argument list into positional strings + {name: value} named args."""
    args, named, depth, cur = [], {}, 0, ""
    for ch in argstr:
        if ch == "(":
            depth += 1
        elif ch == ")":
            depth -= 1
        if ch == "," and depth == 0:
            args.append(cur.strip())
            cur = ""
        else:
            cur += ch
    tail = cur.strip()
    if tail.endswith(");"):
        tail = tail[:-2]
    elif tail.endswith(")"):
        tail = tail[:-1]
    if tail:
        args.append(tail)
    positional = []
    for a in args:
        m = re.match(r"(\w+)\s*:\s*(.+)$", a)
        if m:
            named[m.group(1)] = m.group(2)
        else:
            positional.append(a)
    return positional, named


THIS_SKIP_FIELDS = {
    # field: reason (value not represented in ItemDef)
    "UseSound": "sound data not provisioned",
    "useSoundPitch": "sound data not provisioned",
    "color": "Color not provisioned",
    "glowMask": "glow-mask id not provisioned",
    "stringColor": "fishing-line colour not provisioned",
    "dye": "shader id not provisioned",
    "hairDye": "shader id not provisioned",
    "mountType": "mount id not provisioned",
    "tileWand": "tile-wand id not provisioned",
    "fishingPole": "fishing-pole power not provisioned",
    "makeNPC": "critter npc id not provisioned",
    "lifeRegen": "life regen not provisioned",
    "questItem": "quest flag not provisioned",
    "expert": "expert flag not provisioned",
    "expertOnly": "expert flag not provisioned",
    "isAShopItem": "shop flag not provisioned",
    "hasVanityEffects": "vanity-effects flag not provisioned",
    "sentry": "sentry flag not provisioned",
    "cartTrack": "cart-track flag not provisioned",
    "DD2Summon": "DD2 flag not provisioned",
    "mech": "mech flag not provisioned",
    "noWet": "no-wet flag not provisioned",
    "vanity": "vanity flag not provisioned",
    "noMelee": "contact-damage flag not provisioned",
    "noUseGraphic": "use-graphic flag not provisioned",
    "paint": "paint id not provisioned",
    "paintCoating": "coating id not provisioned",
    "flame": "flame flag not provisioned",
    "voiceSlot": "voice slot not provisioned",
    "chlorophyteExtractinatorConsumable": "extractinator flag not provisioned",
    "notAmmo": "ammo-slot flag not provisioned",
    "bonusTagDamage": "critter-tag damage not provisioned",
    "armorPenetration": "armor penetration not provisioned",
    "manaIncrease": "mana increase not provisioned",
    "alpha": "draw alpha not provisioned",
    "handOnSlot": "equip slot not provisioned",
    "handOffSlot": "equip slot not provisioned",
    "backSlot": "equip slot not provisioned",
    "frontSlot": "equip slot not provisioned",
    "shoeSlot": "equip slot not provisioned",
    "waistSlot": "equip slot not provisioned",
    "wingSlot": "equip slot not provisioned",
    "shieldSlot": "equip slot not provisioned",
    "neckSlot": "equip slot not provisioned",
    "faceSlot": "equip slot not provisioned",
    "balloonSlot": "equip slot not provisioned",
    "beardSlot": "equip slot not provisioned",
    "buy": "shop flag not provisioned",
}

SKIP_CALLS = {
    "RebuildTooltip": "tooltip rebuild — no tooltip data in ItemDef",
    # Item.cs:48919 — the 1803-1807 redirect runs generator-side
    # (`_cond_redirect` copies the final row of 1533+type-1803); the C# call
    # itself must not execute in the interpreter.
    "SetDefaults": "runtime re-default — redirect handled generator-side",
    # Item.cs:48924 — Deprecated ids become air; `_cond_deprecated` zeroes the
    # row generator-side, so the runtime call must not execute in the interpreter.
    "TurnToAir": "deprecated→air — zeroed generator-side",
}


def block_end(lines, i, src):
    """Given lines[i] opens a `{` block (possibly on an earlier line), return the
    index just past its matching `}`."""
    depth = 0
    opened = False
    j = i
    while j < len(lines):
        for ch in lines[j]:
            if ch == "{":
                depth += 1
                opened = True
            elif ch == "}":
                depth -= 1
        if opened and depth == 0:
            return j + 1
        j += 1
    fail(src, i + 1, "unbalanced braces")


def find_method_span(lines, signature, src):
    """Lines [start, end) of the method whose header matches `signature`."""
    for i, line in enumerate(lines):
        if re.match(r"\s*" + re.escape(signature), line):
            depth = 0
            started = False
            for j in range(i, len(lines)):
                depth += lines[j].count("{") - lines[j].count("}")
                if "{" in lines[j]:
                    started = True
                if started and depth == 0:
                    return i, j + 1
            fail(src, i + 1, "unbalanced braces after " + signature)
    fail(src, 1, "method not found: " + signature)


def parse_stmt_at(lines, i, src):
    """Parse one statement starting at lines[i]; returns (stmt or None, next_i).

    Pseudocode (parse one C# statement):
        skip blank / comment lines
        if the line opens a block `{` alone → parse its contents as a group
        classify by prefix:
            `break;` / `return;` / `goto case N;`  → leaf statements
            `case N:` / `default:` → error (switch parser owns labels)
            `switch (`  → match the braces, then walk case labels; consecutive
                          labels share the following body; `default:` collects
                          its own body; bodies parse recursively
            `if (` → take the condition (source text between the outer parens),
                     match the body block, then an optional `else if` chain
                     (recursive) or `else { ... }`
            `int/short/... NAME = RHS;` → local declaration
            `NAME = RHS;` (bare) → local assignment
            `this.SetDefaultsN(arg);` → nested case-table call
            `this.NAME(...)` → helper call / skip-listed call / unknown → fail
            `this.F = rhs` / compound / ++/-- → field writes (skip-listed fields
                          become ('skip', ...) with their reason)
            anything else → fail loud naming the line
    """
    while i < len(lines):
        s = lines[i].strip()
        if not s or s.startswith("//"):
            i += 1
            continue
        break
    else:
        return None, i
    line = i + 1
    src_name = src

    if s == "break;":
        return ("break", line), i + 1
    if s == "return;":
        return ("return", line), i + 1
    if s.startswith("goto case "):
        return ("goto", line, int(s[len("goto case "):].rstrip(";"))), i + 1
    if s.startswith("case ") or s == "default:":
        fail(src_name, line, "case label outside switch parser: " + s)
    if s.startswith("switch ("):
        return parse_switch(lines, i, src_name)
    if s.startswith("if (") or s.startswith("} else"):
        return parse_if(lines, i, src_name)
    if s == "{":
        end = block_end(lines, i, src_name)
        return ("group", line, parse_stmts(lines, i + 1, end - 1, src_name)), end
    if s.endswith(";"):
        body = s[:-1]
        m = LOCAL_RE.match(body)
        if m:
            return ("local", line, m.group(1), m.group(2)), i + 1
        m = re.match(r"this\.SetDefaults([1-5])\((.+)\)$", body)
        if m:
            return ("sdcall", line, "SetDefaults" + m.group(1), m.group(2)), i + 1
        if body.startswith("this."):
            m = ASSIGN_RE.match(body)
            if m:
                field, rhs = m.group(1), m.group(2)
                if field in THIS_SKIP_FIELDS:
                    return ("skip", line, "assign " + field, THIS_SKIP_FIELDS[field]), i + 1
                return ("assign", line, field, rhs), i + 1
            m = COMPOUND_RE.match(body)
            if m:
                field, op, rhs = m.group(1), m.group(2), m.group(3)
                if field in THIS_SKIP_FIELDS:
                    return ("skip", line, "compound " + field, THIS_SKIP_FIELDS[field]), i + 1
                return ("compound", line, field, op, rhs), i + 1
            m = INCDEC_RE.match(body)
            if m:
                field, op = m.group(1), m.group(2)
                if field in THIS_SKIP_FIELDS:
                    return ("skip", line, "incdec " + field, THIS_SKIP_FIELDS[field]), i + 1
                return ("compound", line, field, "+=" if op == "++" else "-=", "1"), i + 1
            m = CALL_RE.match(body)
            if m:
                name, argstr = m.group(1), m.group(2)
                if name in SKIP_CALLS:
                    return ("skip", line, "call " + name, SKIP_CALLS[name]), i + 1
                if name in HELPER_TABLE:
                    args, named = parse_call_args(argstr)
                    return ("call", line, name, args, named), i + 1
                fail(src_name, line, "unsupported method call: " + body[:100])
            fail(src_name, line, "unsupported this-statement: " + body[:100])
        m = LOCAL_ASSIGN_RE.match(body)
        if m:
            return ("localassign", line, m.group(1), m.group(2)), i + 1
        fail(src_name, line, "unsupported statement: " + body[:100])
    fail(src_name, line, "unsupported statement: " + s[:100])


def parse_stmts(lines, lo, hi, src):
    """Parse the statement sequence lines[lo:hi] into tuples (recursive)."""
    out = []
    i = lo
    while i < hi:
        stmt, nxt = parse_stmt_at(lines, i, src)
        if stmt is not None:
            out.append(stmt)
            if nxt <= i:
                fail(src, i + 1, "parser made no progress")
        i = max(nxt, i + 1)
    return out


def parse_switch(lines, i, src):
    """`switch (type) { ... }` → (('switch', line, casemap, default), end_index)."""
    header = lines[i].strip()
    if header not in ("switch (type)", "switch (this.type)"):
        fail(src, i + 1, "unsupported switch subject: " + header)
    end = block_end(lines, i, src)
    casemap = {}
    default = []
    labels = []
    j = i + 1
    if j < end and lines[j].strip() == "{":
        j += 1  # the switch's own opening brace — exactly one
    else:
        fail(src, i + 1, "switch body does not open with a brace")
    body_start = None
    while j < end - 1:
        s = lines[j].strip()
        line = j + 1
        if not s or s.startswith("//"):
            j += 1
            continue
        m = re.match(r"case (-?\d+):$", s)
        is_label = bool(m) or s == "default:"
        if is_label:
            if body_start is not None:
                # flush the previous label group's body
                body = parse_stmts(lines, body_start, j, src)
                for lab in labels:
                    if lab == "default":
                        default.extend(body)
                    else:
                        casemap[lab] = body
                labels = []
                body_start = None
            labels.append(int(m.group(1)) if m else "default")
            j += 1
            continue
        if body_start is None:
            # `case X:` may open an explicit scope block (`case 1994..2001:`,
            # Item.cs:21338) — consume it whole as the label group's body.
            if s == "{":
                scope_end = block_end(lines, j, src)
                body = parse_stmts(lines, j + 1, scope_end - 1, src)
                for lab in labels:
                    if lab == "default":
                        default.extend(body)
                    else:
                        casemap[lab] = body
                labels = []
                j = scope_end
                continue
            body_start = j
        # skip to the next label or the end, one statement at a time
        stmt, nxt = parse_stmt_at(lines, j, src)
        j = nxt
    if body_start is not None:
        body = parse_stmts(lines, body_start, end - 1, src)
        for lab in labels:
            if lab == "default":
                default.extend(body)
            else:
                casemap[lab] = body
        labels = []
    if labels:
        fail(src, i + 1, "switch labels with no body")
    return ("switch", i + 1, casemap, default), end


def parse_if(lines, i, src):
    """`if (COND) {...} [else if (...) ... | else {...}]` → (stmt, next_index)."""
    s = lines[i].strip()
    line = i + 1
    cond = _extract_if_cond(lines, i, src)
    j = i
    while not lines[j].strip().endswith("{"):
        j += 1
        if j - i > 3:
            fail(src, line, "cannot locate if body")
    end = block_end(lines, j, src)
    body = parse_stmts(lines, j + 1, end - 1, src)
    else_body = []
    k = end
    if k < len(lines):
        nxt = lines[k].strip()
        if nxt.startswith("else if ("):
            inner, k2 = parse_if(lines, k, src)
            else_body = [inner]
            end = k2
        elif nxt.startswith("else"):
            if not nxt.endswith("{"):
                if nxt != "else":
                    fail(src, k + 1, "unsupported else form: " + nxt[:60])
                k2 = k + 1
                while not lines[k2].strip().endswith("{"):
                    k2 += 1
            else:
                k2 = k
            end2 = block_end(lines, k2, src)
            else_body = parse_stmts(lines, k2 + 1, end2 - 1, src)
            end = end2
    return ("if", line, cond, body, else_body), end


def _extract_if_cond(lines, i, src):
    """Source text between the outer parens of `if (...)` (may span lines)."""
    joined = ""
    j = i
    while j < len(lines) and j - i <= 3:
        joined += lines[j].strip()
        if joined.endswith("{"):
            joined = joined[:-1].rstrip()
        m = re.search(r"if \((.*)\)$", joined)
        if m:
            return m.group(1)
        j += 1
    fail(src, i + 1, "cannot extract if condition: " + joined[:80])


# ---------------------------------------------------------------------------
# Expression evaluation (recursive descent over the whitelisted grammar)
# ---------------------------------------------------------------------------

TOKEN_RE = re.compile(r"""
    (?P<ws>\s+)
  | (?P<toint>\.ToInt\(\))
  | (?P<float>-?\d+\.\d+f|-?\d+f|-?\d+\.\d+)
  | (?P<int>-?\d+)
  | (?P<name>[A-Za-z_][A-Za-z0-9_]*(?:\.[A-Za-z_][A-Za-z0-9_]*)*)
  | (?P<op><=|>=|==|!=|&&|\|\||[-+*/()<>!,?:])
""", re.VERBOSE)


def tokenize(expr, src, line):
    toks = []
    pos = 0
    while pos < len(expr):
        m = TOKEN_RE.match(expr, pos)
        if not m:
            fail(src, line, "cannot tokenize expression: " + expr[pos:pos + 40])
        pos = m.end()
        if m.lastgroup != "ws":
            toks.append((m.lastgroup, m.group()))
    return toks


class ExprParser:
    """Grammar (whitelist), C# precedence:

    expr    := or
    or      := and ('||' and)*
    and     := cmp ('&&' cmp)*
    cmp     := add (('=='|'!='|'<'|'>'|'<='|'>=') add)?      → bool
    add     := term (('+'|'-') term)*
    term    := factor (('*'|'/') factor)*                    int/int truncates (C#)
    factor  := '-' factor | atom
    atom    := int | float | true | false | name | price-call
             | '(' expr ')' ['.ToInt()'] | cast
    cast    := '(' (int|ushort|short|byte|sbyte|double) ')' factor
    """

    def __init__(self, toks, env, src, line):
        self.toks = toks
        self.i = 0
        self.env = env
        self.src = src
        self.line = line

    def peek(self, ahead=0):
        j = self.i + ahead
        return self.toks[j] if j < len(self.toks) else (None, None)

    def take(self):
        tok = self.toks[self.i]
        self.i += 1
        return tok

    def expect(self, val):
        kind, got = self.take()
        if got != val:
            fail(self.src, self.line, "expected %r, got %r" % (val, got))

    def parse(self):
        v = self.ternary()
        if self.i != len(self.toks):
            fail(self.src, self.line, "trailing tokens: %s" % (self.toks[self.i:],))
        return v

    def expr_paren(self):
        # full expression inside parentheses — no trailing-token check (the
        # caller consumes the closing `)`)
        return self.ternary()

    def ternary(self):
        # C# `cond ? a : b` (lowest precedence; SetFoodDefaults tail,
        # Item.cs:48392)
        v = self.or_expr()
        if self.peek()[1] == "?":
            self.take()
            a = self.ternary()
            self.expect(":")
            b = self.ternary()
            return a if v else b
        return v

    def or_expr(self):
        v = self.and_expr()
        while self.peek()[1] == "||":
            self.take()
            r = self.and_expr()
            v = v or r
        return v

    def and_expr(self):
        v = self.cmp_expr()
        while self.peek()[1] == "&&":
            self.take()
            r = self.cmp_expr()
            v = v and r
        return v

    def cmp_expr(self):
        v = self.add_expr()
        if self.peek()[1] in ("==", "!=", "<", ">", "<=", ">="):
            op = self.take()[1]
            r = self.add_expr()
            return {"==": v == r, "!=": v != r, "<": v < r, ">": v > r,
                    "<=": v <= r, ">=": v >= r}[op]
        return v

    def add_expr(self):
        v = self.term()
        while self.peek()[1] in ("+", "-"):
            op = self.take()[1]
            r = self.term()
            v = v + r if op == "+" else v - r
        return v

    def term(self):
        v = self.factor()
        while self.peek()[1] in ("*", "/"):
            op = self.take()[1]
            r = self.factor()
            if op == "*":
                v = v * r
            else:
                if isinstance(v, bool) or isinstance(r, bool):
                    fail(self.src, self.line, "arithmetic on bool operand")
                if isinstance(v, int) and isinstance(r, int):
                    v = int(v / r)  # C# int division truncates toward zero
                else:
                    v = v / r
        return v

    def factor(self):
        if self.peek()[1] == "-":
            self.take()
            return -self.factor()
        return self.atom()

    def atom(self):
        kind, val = self.peek()
        if val is None:
            fail(self.src, self.line, "unexpected end of expression")
        if kind == "int":
            self.take()
            return int(val)
        if kind == "float":
            self.take()
            return float(val[:-1] if val.endswith("f") else val)
        if val == "(":
            self.take()
            if self.peek()[1] in ("int", "ushort", "short", "byte", "sbyte", "double", "float") and \
                    self.peek(1)[1] == ")":
                cast = self.take()[1]
                self.expect(")")
                return apply_cast(cast, self.factor(), self.src, self.line)
            v = self.expr_paren()
            self.expect(")")
            if self.peek()[0] == "toint":
                self.take()
                return 1 if v else 0
            return v
        if kind == "name":
            return self.name_atom()
        fail(self.src, self.line, "unexpected token %r" % (val,))

    def name_atom(self):
        name = self.take()[1]
        env = self.env
        if name == "true":
            return True
        if name == "false":
            return False
        if name in ("Item.sellPrice", "Item.buyPrice"):
            self.expect("(")
            args = self.call_args()
            self.expect(")")
            kind = "sell" if name.endswith("sellPrice") else "buy"
            if any(not isinstance(a, int) for a in args):
                fail(self.src, self.line, "non-int argument in price call")
            return price_value(kind, args, self.src, self.line)
        if name == "type":
            return env["type"]
        if name.startswith("this."):
            field = name[len("this."):]
            if field in env["prices"]:
                return env["prices"][field]
            if field in env["row"]:
                return env["row"][field]
            fail(self.src, self.line, "unknown field reference this." + field)
        for prefix, table in (("ItemID.", env["ids"]["ItemID"]),
                              ("AmmoID.", env["ids"]["AmmoID"]),
                              ("ProjectileID.", env["ids"]["ProjectileID"]),
                              ("BuffID.", env["ids"]["BuffID"]),
                              ("TileID.", env["ids"]["TileID"]),
                              ("WallID.", env["ids"]["WallID"]),
                              ("ItemRarityColor.", env["ids"]["ItemRarityColor"])):
            if name.startswith(prefix):
                const = name[len(prefix):]
                if const not in table:
                    fail(self.src, self.line, "unknown %s constant %s" % (prefix, const))
                return table[const]
        if name in env["locals"]:
            return env["locals"][name]
        if name in env["loopvars"]:
            return env["loopvars"][name]
        fail(self.src, self.line, "unknown identifier in expression: " + name)

    def call_args(self):
        args = []
        if self.peek()[1] == ")":
            return args
        while True:
            args.append(self.add_expr())
            if self.peek()[1] == ",":
                self.take()
                continue
            break
        return args


def apply_cast(cast, v, src, line):
    if cast == "double":
        return float(v)
    if cast == "float":
        return float(v)  # f32 rounding differences are irrelevant at these magnitudes
    if cast == "int":
        return int(v)  # C# (int) truncates toward zero
    if cast == "ushort":
        if not 0 <= int(v) <= 0xFFFF:
            fail(src, line, "ushort cast out of range: %s" % v)
        return int(v)
    if cast == "short":
        if not -0x8000 <= int(v) <= 0x7FFF:
            fail(src, line, "short cast out of range: %s" % v)
        return int(v)
    if cast == "byte":
        if not 0 <= int(v) <= 255:
            fail(src, line, "byte cast out of range: %s" % v)
        return int(v)
    if cast == "sbyte":
        if not -128 <= int(v) <= 127:
            fail(src, line, "sbyte cast out of range: %s" % v)
        return int(v)
    fail(src, line, "unsupported cast " + cast)


def eval_expr(expr, env, src, line):
    return ExprParser(tokenize(expr, src, line), env, src, line).parse()


COND_SPLIT_RE = re.compile(r"\s*(\|\||&&)\s*")


def eval_cond(cond, env, src, line):
    """Evaluate a whitelisted boolean condition (comparisons joined by && / ||)."""
    return bool(ExprParser(tokenize(cond, src, line), env, src, line).parse())


VARIANT_COND_RE = re.compile(r"this\.Variant\s*(==|!=)\s*ItemVariants\.(\w+)Variant$")


# ---------------------------------------------------------------------------
# Execution
# ---------------------------------------------------------------------------

class Ctx:
    """Interpreter context for one item id."""

    def __init__(self, ids, prices, tables):
        self.ids = ids
        self.prices = prices
        self.tables = tables  # projhook / is_food / is_drill / is_chainsaw / deprecated
        self.row = {}
        self.locals = {}
        self.loopvars = {}
        self.returned = False
        self.brk = False
        self.variant_rows = []       # (id, variant, [(op, field, value)], kind, line)
        self.deviation_sites = []    # (id, line, what)
        self.redirect_to = None
        self.deprecated_hit = False
        self.material_written = False  # the case body assigned `material` itself
        self.write_log = []            # fields written, in order (variant check)

    def env(self, type_value):
        return {"ids": self.ids, "prices": self.prices, "row": self.row,
                "locals": self.locals, "loopvars": self.loopvars,
                "type": type_value}


def exec_stmts(stmts, ctx, type_value, src, casemap=None, in_switch=False):
    for st in stmts:
        if ctx.returned:
            return
        if ctx.brk:
            # A `break` from an inner body exits the enclosing statement list
            # too — C# `if (type < 3315 || type > 3317) { break; } goto case
            # 3262;` (Item.cs:30037-30041) never reaches its goto; without this
            # the guard-broken id fell through into the goto's case body (the
            # yoyo template over every unmatched id of the switch).
            return
        kind = st[0]
        line = st[1]
        if kind == "break":
            if not in_switch:
                fail(src, line, "break outside switch")
            ctx.brk = True
            return
        if kind == "return":
            ctx.returned = True
            return
        if kind == "group":
            exec_stmts(st[2], ctx, type_value, src, casemap, in_switch)
            if ctx.brk:
                return
        elif kind == "assign":
            _, _, field, rhs = st
            env = ctx.env(type_value)
            # chained assignment `a = (b = RHS)`: the innermost field takes the
            # value first, then the outer ones (same value)
            fields = [field]
            while True:
                m = re.match(r"^\(this\.(\w+) = (.+)\)$", rhs)
                if not m:
                    break
                fields.append(m.group(1))
                rhs = m.group(2)
            value = eval_expr(rhs, env, src, line)
            for f in reversed(fields):
                ctx.row[f] = value
                ctx.write_log.append(f)
                if f == "material":
                    ctx.material_written = True
        elif kind == "compound":
            _, _, field, op, rhs = st
            env = ctx.env(type_value)
            if field not in ctx.row:
                fail(src, line, "compound op on unset field " + field)
            delta = eval_expr(rhs, env, src, line)
            ctx.row[field] = apply_compound(op, ctx.row[field], delta, src, line)
            ctx.write_log.append(field)
        elif kind == "call":
            _, _, name, args, named = st
            env = ctx.env(type_value)
            HELPER_TABLE[name](ctx, type_value, args, named, src, line, env)
        elif kind == "sdcall":
            _, _, method, arg = st
            env = ctx.env(type_value)
            target = eval_expr(arg, env, src, line)
            # `method` is the parsed-region NAME ("SetDefaults3"); the parsed
            # block list rides on ctx.methods so the inheritance runs the real
            # body (Item.cs:31271-style `SetDefaults1(1)` inheritances).
            run_method(ctx.methods[method], target, ctx, src)
        elif kind == "local":
            _, _, name, rhs = st
            env = ctx.env(type_value)
            ctx.locals[name] = eval_expr(rhs, env, src, line)
        elif kind == "localassign":
            _, _, name, rhs = st
            env = ctx.env(type_value)
            if name == "type":
                # `type = 52;` inside case 52 (Item.cs:2004) — self-assignment of
                # the switch parameter; only a true re-point stays unsupported.
                v = eval_expr(rhs, env, src, line)
                if v != type_value:
                    fail(src, line, "parameter re-point type = %d (type-merge path)" % v)
                return
            if name not in ctx.locals:
                fail(src, line, "assignment to unknown local " + name)
            ctx.locals[name] = eval_expr(rhs, env, src, line)
        elif kind == "if":
            _, _, cond, body, else_body = st
            vm = VARIANT_COND_RE.match(cond)
            if vm:
                handle_variant_branch(ctx, st, type_value, src, vm)
                continue
            special = SPECIAL_CONDS.get(cond)
            if special is not None:
                special(ctx, type_value, body, src, line)
                continue
            taken = eval_cond(cond, ctx.env(type_value), src, line)
            branch = body if taken else else_body
            if branch:
                exec_stmts(branch, ctx, type_value, src, casemap, in_switch)
                # a `break` here belongs to an enclosing switch — it propagates
        elif kind == "switch":
            _, _, cmap, default = st
            body = cmap.get(type_value)
            if body is None:
                body = default
            if body:
                exec_stmts(body, ctx, type_value, src, cmap, in_switch=True)
            if ctx.brk:
                ctx.brk = False
        elif kind == "goto":
            _, _, target = st
            if casemap is None or target not in casemap:
                fail(src, line, "goto case %s outside its switch" % target)
            exec_stmts(casemap[target], ctx, type_value, src, casemap, in_switch=True)
            return  # the goto replaces the remaining statements of this body
        elif kind == "skip":
            pass
        else:
            fail(src, line, "unsupported statement kind " + kind)


def apply_compound(op, cur, delta, src, line):
    if op == "+=":
        return cur + delta
    if op == "-=":
        return cur - delta
    if op == "*=":
        return cur * delta
    if op == "/=":
        if isinstance(cur, int) and isinstance(delta, int):
            return int(cur / delta)  # C# int division truncates toward zero
        return cur / delta
    fail(src, line, "unknown compound op " + op)


def handle_variant_branch(ctx, st, type_value, src, vm):
    """Record `if (this.Variant ==/!= XVariant) {...}` as VARIANT_OVERRIDES data.

    `==` branches: the base row keeps the no-variant state; the branch body's
    assignments are evaluated against the row snapshot and recorded as absolute
    `Set` ops (a relative `(int)(this.F * k)` resolves against the snapshot —
    valid because the post-run check forbids later writes to branch fields).

    `!=` branches: the body DOES apply to the no-variant state (baked into the
    base row by normal execution), and a restore row records the pre-branch
    values so `apply(id, X)` reproduces the skipped state exactly.
    """
    _, line, cond, body, else_body = st
    if else_body:
        fail(src, line, "Variant branch with else")
    negate = vm.group(1) == "!="
    variant = vm.group(2)
    if variant not in ("Stronger", "Weaker", "Rebalanced", "Enabled", "DisabledBossSummon"):
        fail(src, line, "unknown variant " + variant)
    for s in body:
        if s[0] != "assign":
            fail(src, line, "non-assignment inside Variant branch: %s" % (s[:2],))
    touched = [s[2] for s in body]
    before = {f: ctx.row.get(f) for f in touched}
    if negate:
        exec_stmts(body, ctx, type_value, src)  # base state includes the branch
        ops = [("Set", f, "__MATERIAL__" if f == "material" else before[f])
               for f in touched]
        ctx.variant_rows.append((ctx.row["type"], variant, ops, "restore", line,
                                 len(ctx.write_log)))
    else:
        snap = dict(ctx.row)
        env = ctx.env(type_value)
        env = dict(env, row=snap)
        ops = []
        for s in body:
            _, l2, field, rhs = s
            if field == "material":
                fail(src, l2, "material write inside a Variant == branch")
            ops.append(("Set", field, eval_expr(rhs, env, src, l2)))
        ctx.variant_rows.append((ctx.row["type"], variant, ops, "delta", line,
                                 len(ctx.write_log)))


# Special epilogue/interpreter conditions, dispatched by exact source text.
def _cond_redirect(ctx, type_value, body, src, line):
    # Item.cs:48917-48920 — full SetDefaults(1533 + type - 1803); resolved
    # generator-side by copying the already-final row of that id. SPECIAL_CONDS
    # dispatches by source text without evaluating, so the range gate repeats
    # here; the body's runtime call is a named parse-level skip.
    if 1803 <= type_value <= 1807:
        ctx.redirect_to = 1533 + type_value - 1803


def _cond_projhook(ctx, type_value, body, src, line):
    if ctx.row.get("shoot", 0) in ctx.tables["projhook"]:
        exec_stmts(body, ctx, type_value, src)


def _cond_drill(ctx, type_value, body, src, line):
    t = ctx.row["type"]
    if t in ctx.tables["is_drill"] or t in ctx.tables["is_chainsaw"] or t == 1262:
        exec_stmts(body, ctx, type_value, src)


def _cond_isfood(ctx, type_value, body, src, line):
    if ctx.row["type"] in ctx.tables["is_food"]:
        exec_stmts(body, ctx, type_value, src)


def _cond_deprecated(ctx, type_value, body, src, line):
    t = ctx.row["type"]
    if 0 < t < ITEM_COUNT and t in ctx.tables["deprecated"]:
        ctx.deprecated_hit = True  # TurnToAir ⇒ SetDefaults(0), Item.cs:49884-48990


def _cond_infected(ctx, type_value, body, src, line):
    # Deviation: `if (Main.infectedSeed)` (Item.cs:2180) — a world-seed conditional
    # outside ItemVariants; the base value is kept and the site is recorded.
    ctx.deviation_sites.append((ctx.row["type"], line, "Main.infectedSeed conditional"))


def _cond_hairdye(ctx, type_value, body, src, line):
    pass  # hairDye keeps the ResetStats value; shader ids not provisioned


SPECIAL_CONDS = {
    "this.type >= 1803 && this.type <= 1807": _cond_redirect,
    "Main.projHook[this.shoot]": _cond_projhook,
    "ItemID.Sets.IsDrill[this.type] || ItemID.Sets.IsChainsaw[this.type] || this.type == 1262": _cond_drill,
    "ItemID.Sets.IsFood[this.type]": _cond_isfood,
    "this.type > 0 && this.type < ItemID.Count && ItemID.Sets.Deprecated[this.type]": _cond_deprecated,
    "Main.infectedSeed": _cond_infected,
    "this.hairDye != 0": _cond_hairdye,
}


# ---------------------------------------------------------------------------
# Helper ports (each cites its C# body; skipped fields keep the reasons above)
# ---------------------------------------------------------------------------

def _arg(args, named, key, i, default, env, src, line):
    if key in named:
        return eval_expr(named[key], env, src, line)
    if i < len(args):
        return eval_expr(args[i], env, src, line)
    if default is not None:
        return default
    fail(src, line, "missing argument %d for helper" % i)


def h_SetShopValues(ctx, type_value, args, named, src, line, env):
    """`SetShopValues(ItemRarityColor rarity, int coinValue)` — Item.cs:48585-48589."""
    ctx.row["rare"] = eval_expr(args[0], env, src, line)
    ctx.row["value"] = eval_expr(args[1], env, src, line)


def h_DefaultToFood(ctx, type_value, args, named, src, line, env):
    """Item.cs:48541-48568 (UseSound Item3/Item2 not provisioned)."""
    gulp = bool(_arg(args, named, "useGulpSound", 4, False, env, src, line))
    anim = _arg(args, named, "animationTime", 5, 17, env, src, line)
    ctx.row["useStyle"] = 9 if gulp else 2
    ctx.row["useTurn"] = True
    ctx.row["useTime"] = anim
    ctx.row["useAnimation"] = anim
    ctx.row["consumable"] = True
    ctx.row["width"] = _arg(args, named, "newwidth", 0, None, env, src, line)
    ctx.row["height"] = _arg(args, named, "newheight", 1, None, env, src, line)
    ctx.row["buffType"] = _arg(args, named, "foodbuff", 2, None, env, src, line)
    ctx.row["buffTime"] = _arg(args, named, "foodbuffduration", 3, None, env, src, line)
    ctx.row["rare"] = 1
    ctx.row["value"] = price_value("buy", [0, 0, 20], src, line)


def h_DefaultToHealingPotion(ctx, type_value, args, named, src, line, env):
    """Item.cs:48570-48583."""
    anim = _arg(args, named, "animationTime", 3, 17, env, src, line)
    ctx.row["useStyle"] = 9
    ctx.row["useTurn"] = True
    ctx.row["useTime"] = anim
    ctx.row["useAnimation"] = anim
    ctx.row["consumable"] = True
    ctx.row["width"] = eval_expr(args[0], env, src, line)
    ctx.row["height"] = eval_expr(args[1], env, src, line)
    ctx.row["rare"] = 1
    ctx.row["value"] = price_value("buy", [0, 0, 20], src, line)
    ctx.row["potion"] = True
    ctx.row["healLife"] = eval_expr(args[2], env, src, line)


def h_DefaultToHeadgear(ctx, type_value, args, named, src, line, env):
    """Item.cs:48591-48596."""
    ctx.row["width"] = eval_expr(args[0], env, src, line)
    ctx.row["height"] = eval_expr(args[1], env, src, line)
    ctx.row["headSlot"] = eval_expr(args[2], env, src, line)


def h_DefaultToBody(ctx, type_value, args, named, src, line, env):
    """Item.cs:48598-48603."""
    ctx.row["width"] = eval_expr(args[0], env, src, line)
    ctx.row["height"] = eval_expr(args[1], env, src, line)
    ctx.row["bodySlot"] = eval_expr(args[2], env, src, line)


def h_DefaultToLegs(ctx, type_value, args, named, src, line, env):
    """Item.cs:48605-48610."""
    ctx.row["width"] = eval_expr(args[0], env, src, line)
    ctx.row["height"] = eval_expr(args[1], env, src, line)
    ctx.row["legSlot"] = eval_expr(args[2], env, src, line)


def h_DefaultToAccessory(ctx, type_value, args, named, src, line, env):
    """Item.cs:48612-48617."""
    ctx.row["width"] = _arg(args, named, "newwidth", 0, 24, env, src, line)
    ctx.row["height"] = _arg(args, named, "newheight", 1, 24, env, src, line)
    ctx.row["accessory"] = True


def h_DefaultToInfoAccessory(ctx, type_value, args, named, src, line, env):
    """Item.cs:48619-48623 (hasVanityEffects not provisioned)."""
    h_DefaultToAccessory(ctx, type_value, args, named, src, line, env)


def h_DefaultToGuitar(ctx, type_value, args, named, src, line, env):
    """Item.cs:48625-48633 (UseSound not provisioned)."""
    ctx.row["width"] = _arg(args, named, "newwidth", 0, 24, env, src, line)
    ctx.row["height"] = _arg(args, named, "newheight", 1, 24, env, src, line)
    ctx.row["autoReuse"] = True
    ctx.row["holdStyle"] = 5
    ctx.row["useStyle"] = 12
    ctx.row["useTime"] = 12
    ctx.row["useAnimation"] = 12


def h_DefaultToMusicBox(ctx, type_value, args, named, src, line, env):
    """Item.cs:48635-48652."""
    ctx.row["useStyle"] = 1
    ctx.row["useTurn"] = True
    ctx.row["useAnimation"] = 15
    ctx.row["useTime"] = 10
    ctx.row["autoReuse"] = True
    ctx.row["consumable"] = True
    ctx.row["createTile"] = 139
    ctx.row["placeStyle"] = eval_expr(args[0], env, src, line)
    ctx.row["width"] = 24
    ctx.row["height"] = 24
    ctx.row["rare"] = 4
    ctx.row["value"] = 100000
    ctx.row["accessory"] = True
    ctx.row["vanity"] = True


def h_DefaultToQuestFish(ctx, type_value, args, named, src, line, env):
    """Item.cs:27651-27657 (questItem not provisioned)."""
    ctx.row["width"] = 26
    ctx.row["height"] = 26
    ctx.row["rare"] = -11


def h_DefaultToGolfBall(ctx, type_value, args, named, src, line, env):
    """Item.cs:35444-35460 (noUseGraphic/noMelee/hasVanityEffects not provisioned)."""
    ctx.row["shoot"] = eval_expr(args[0], env, src, line)
    ctx.row["useStyle"] = 1
    ctx.row["shootSpeed"] = 12.0
    ctx.row["width"] = 18
    ctx.row["height"] = 20
    ctx.row["useAnimation"] = 15
    ctx.row["useTime"] = 15
    ctx.row["value"] = 0
    ctx.row["accessory"] = True
    h_SetShopValues(ctx, type_value, ["ItemRarityColor.Green2", "Item.buyPrice(0, 1)"],
                    {}, src, line, env)


def h_DefaultToBanner(ctx, type_value, args, named, src, line, env):
    """Item.cs:47753-47758."""
    h_SetShopValues(ctx, type_value, ["ItemRarityColor.Blue1", "1000"], {}, src, line, env)
    h_DefaultToPlaceableTile(ctx, type_value, ["(ushort)91", args[0] if args else "0"],
                             {}, src, line, env)
    ctx.row["width"] = 10
    ctx.row["height"] = 24


def h_DefaultToMonolith(ctx, type_value, args, named, src, line, env):
    """Item.cs:47761-47768 (vanity not provisioned)."""
    h_DefaultToPlaceableTile(ctx, type_value, [args[0]], {"tileStyleToPlace": args[1]}
                             if len(args) > 1 else {}, src, line, env)
    ctx.row["width"] = 22
    ctx.row["height"] = 32
    ctx.row["accessory"] = True


def h_DefaultToBomb(ctx, type_value, args, named, src, line, env):
    """Item.cs:47770-47783 (noUseGraphic/noMelee not provisioned)."""
    ctx.row["useStyle"] = 1
    ctx.row["shootSpeed"] = eval_expr(args[1], env, src, line)
    ctx.row["shoot"] = eval_expr(args[0], env, src, line)
    ctx.row["width"] = 8
    ctx.row["height"] = 28
    ctx.row["consumable"] = True
    ctx.row["useAnimation"] = 40
    ctx.row["useTime"] = 40


def h_DefaultToVoiceOverrideAccessory(ctx, type_value, args, named, src, line, env):
    """Item.cs:47785-47797 (UseSound/voiceSlot/vanity not provisioned)."""
    ctx.row["useStyle"] = 4
    ctx.row["useTurn"] = False
    ctx.row["useAnimation"] = 17
    ctx.row["useTime"] = 17
    ctx.row["width"] = 24
    ctx.row["height"] = 24
    ctx.row["accessory"] = True


def h_DefaultToSolution(ctx, type_value, args, named, src, line, env):
    """Item.cs:47799-47808."""
    ctx.row["shoot"] = eval_expr(args[0], env, src, line) - 145
    ctx.row["ammo"] = env["ids"]["AmmoID"]["Solution"]
    ctx.row["width"] = 10
    ctx.row["height"] = 12
    ctx.row["value"] = price_value("buy", [0, 0, 15], src, line)
    ctx.row["rare"] = 3
    ctx.row["consumable"] = True


def h_DefaultToWhip(ctx, type_value, args, named, src, line, env):
    """Item.cs:47810-47826 (noMelee/noUseGraphic not provisioned)."""
    anim = _arg(args, named, "animationTotalTime", 4, 30, env, src, line)
    ctx.row["autoReuse"] = False
    ctx.row["useStyle"] = 1
    ctx.row["useAnimation"] = anim
    ctx.row["useTime"] = anim
    ctx.row["width"] = 18
    ctx.row["height"] = 18
    ctx.row["shoot"] = eval_expr(args[0], env, src, line)
    ctx.row["summon"] = True
    ctx.row["damage"] = eval_expr(args[1], env, src, line)
    ctx.row["knockBack"] = eval_expr(args[2], env, src, line)
    ctx.row["shootSpeed"] = eval_expr(args[3], env, src, line)


def h_DefaultToKite(ctx, type_value, args, named, src, line, env):
    """Item.cs:47828-47838 (noUseGraphic not provisioned)."""
    ctx.row["width"] = 20
    ctx.row["height"] = 28
    h_DefaultToThrownWeapon(ctx, type_value, [args[0], "15", "8f"], {}, src, line, env)
    ctx.row["consumable"] = False
    ctx.row["ranged"] = False
    h_SetShopValues(ctx, type_value, ["ItemRarityColor.Blue1", "Item.buyPrice(0, 2)"], {},
                    src, line, env)
    ctx.row["createTile"] = 723


def h_DefaultToVanitypet(ctx, type_value, args, named, src, line, env):
    """Item.cs:47936-47949 (noMelee not provisioned)."""
    ctx.row["damage"] = 0
    ctx.row["useStyle"] = 1
    ctx.row["width"] = 16
    ctx.row["height"] = 30
    ctx.row["useAnimation"] = 20
    ctx.row["useTime"] = 20
    ctx.row["rare"] = 3
    ctx.row["value"] = price_value("sell", [0, 2], src, line)
    ctx.row["buffType"] = eval_expr(args[1], env, src, line)
    ctx.row["shoot"] = eval_expr(args[0], env, src, line)


def h_DefaultToSeaShell(ctx, type_value, args, named, src, line, env):
    """Item.cs:47966-47998 — includes the `switch (this.type)` placeStyle/value table."""
    ctx.row["useStyle"] = 1
    ctx.row["autoReuse"] = True
    ctx.row["useAnimation"] = 15
    ctx.row["useTime"] = 10
    ctx.row["consumable"] = True
    ctx.row["createTile"] = 324
    ctx.row["width"] = 22
    ctx.row["height"] = 22
    t = ctx.row["type"]
    if t == 4071:
        ctx.row["placeStyle"] = 12
        ctx.row["value"] = price_value("sell", [0, 1], src, line)
    elif t == 4073:
        ctx.row["placeStyle"] = 9
        ctx.row["value"] = price_value("sell", [0, 0, 20], src, line)
    elif t == 4072:
        ctx.row["placeStyle"] = 6
        ctx.row["value"] = price_value("sell", [0, 0, 20], src, line)
    elif t == 2626:
        ctx.row["placeStyle"] = 3
        ctx.row["value"] = price_value("sell", [0, 0, 10], src, line)
    else:
        ctx.row["value"] = price_value("sell", [0, 0, 5], src, line)


def h_DefaultToCapturedCritter(ctx, type_value, args, named, src, line, env):
    """Item.cs:48000-48013 (makeNPC/noUseGraphic not provisioned)."""
    ctx.row["useStyle"] = 1
    ctx.row["autoReuse"] = True
    ctx.row["useTurn"] = True
    ctx.row["useAnimation"] = 15
    ctx.row["useTime"] = 10
    ctx.row["consumable"] = True
    ctx.row["width"] = 12
    ctx.row["height"] = 12
    ctx.row["createTile"] = 724


def h_DefaultToStaff(ctx, type_value, args, named, src, line, env):
    """Item.cs:48015-48022 (UseSound not provisioned)."""
    h_DefaultToMagicWeapon(ctx, type_value, [args[0], args[2], args[1], "true"], {},
                           src, line, env)
    ctx.row["mana"] = eval_expr(args[3], env, src, line)
    ctx.row["width"] = 40
    ctx.row["height"] = 40


def h_DefaultToSpear(ctx, type_value, args, named, src, line, env):
    """Item.cs:48024-48038 (noMelee/noUseGraphic not provisioned)."""
    anim = eval_expr(args[2], env, src, line)
    ctx.row["useStyle"] = 5
    ctx.row["useAnimation"] = 31
    ctx.row["useTime"] = 31
    ctx.row["shootSpeed"] = eval_expr(args[1], env, src, line)
    ctx.row["width"] = 32
    ctx.row["height"] = 32
    ctx.row["shoot"] = eval_expr(args[0], env, src, line)
    ctx.row["melee"] = True
    ctx.row["useAnimation"] = anim
    ctx.row["useTime"] = anim


def h_DefaultToMinecart(ctx, type_value, args, named, src, line, env):
    """Item.cs:48395-48400 (mountType not provisioned)."""
    ctx.row["width"] = 36
    ctx.row["height"] = 26


def h_DefaultToPlaceableWall(ctx, type_value, args, named, src, line, env):
    """Item.cs:48402-48413."""
    ctx.row["useStyle"] = 1
    ctx.row["useTurn"] = True
    ctx.row["useAnimation"] = 15
    ctx.row["useTime"] = 7
    ctx.row["autoReuse"] = True
    ctx.row["consumable"] = True
    ctx.row["createWall"] = eval_expr(args[0], env, src, line)
    ctx.row["width"] = 12
    ctx.row["height"] = 12


def h_SetWeaponValues(ctx, type_value, args, named, src, line, env):
    """Item.cs:48415-48420."""
    ctx.row["damage"] = eval_expr(args[0], env, src, line)
    ctx.row["knockBack"] = eval_expr(args[1], env, src, line)
    ctx.row["crit"] = _arg(args, named, "bonusCritChance", 2, 0, env, src, line)


def h_DefaultToBow(ctx, type_value, args, named, src, line, env):
    """Item.cs:48422-48428 (UseSound not provisioned)."""
    auto = bool(_arg(args, named, "hasAutoReuse", 2, False, env, src, line))
    h_DefaultToRangedWeapon(ctx, type_value,
                            ["1", "AmmoID.Arrow", args[0], args[1], "true" if auto else "false"],
                            {}, src, line, env)
    ctx.row["width"] = 14
    ctx.row["height"] = 30


def h_DefaultToMagicWeapon(ctx, type_value, args, named, src, line, env):
    """Item.cs:48430-48440 (noMelee not provisioned)."""
    auto = bool(_arg(args, named, "hasAutoReuse", 3, False, env, src, line))
    ctx.row["autoReuse"] = auto
    ctx.row["useStyle"] = 5
    ctx.row["useAnimation"] = eval_expr(args[1], env, src, line)
    ctx.row["useTime"] = eval_expr(args[1], env, src, line)
    ctx.row["shoot"] = eval_expr(args[0], env, src, line)
    ctx.row["shootSpeed"] = eval_expr(args[2], env, src, line)
    ctx.row["magic"] = True


def h_DefaultToRangedWeapon(ctx, type_value, args, named, src, line, env):
    """Item.cs:48442-48453 (noMelee not provisioned)."""
    auto = bool(_arg(args, named, "hasAutoReuse", 4, False, env, src, line))
    ctx.row["autoReuse"] = auto
    ctx.row["useStyle"] = 5
    ctx.row["useAnimation"] = eval_expr(args[2], env, src, line)
    ctx.row["useTime"] = eval_expr(args[2], env, src, line)
    ctx.row["shoot"] = eval_expr(args[0], env, src, line)
    ctx.row["useAmmo"] = eval_expr(args[1], env, src, line)
    ctx.row["shootSpeed"] = eval_expr(args[3], env, src, line)
    ctx.row["ranged"] = True


def h_DefaultToThrownWeapon(ctx, type_value, args, named, src, line, env):
    """Item.cs:48455-48466 (noMelee not provisioned)."""
    auto = bool(_arg(args, named, "hasAutoReuse", 3, False, env, src, line))
    ctx.row["autoReuse"] = auto
    ctx.row["useStyle"] = 1
    ctx.row["useAnimation"] = eval_expr(args[1], env, src, line)
    ctx.row["useTime"] = eval_expr(args[1], env, src, line)
    ctx.row["shoot"] = eval_expr(args[0], env, src, line)
    ctx.row["shootSpeed"] = eval_expr(args[2], env, src, line)
    ctx.row["ranged"] = True
    ctx.row["consumable"] = True


def h_DefaultToTorch(ctx, type_value, args, named, src, line, env):
    """Item.cs:48468-48484 (flame/noWet not provisioned)."""
    ctx.row["holdStyle"] = 1
    ctx.row["autoReuse"] = True
    ctx.row["consumable"] = True
    ctx.row["createTile"] = 4
    ctx.row["placeStyle"] = eval_expr(args[0], env, src, line)
    ctx.row["width"] = 10
    ctx.row["height"] = 12
    ctx.row["value"] = 60
    ctx.row["useStyle"] = 1
    ctx.row["useTurn"] = True
    ctx.row["useAnimation"] = 15
    ctx.row["useTime"] = 10


def h_DefaultToPlaceableTile(ctx, type_value, args, named, src, line, env):
    """Item.cs:48486-48527 (the int overload delegates to the ushort one)."""
    ctx.row["createTile"] = eval_expr(args[0], env, src, line)
    ctx.row["placeStyle"] = _arg(args, named, "tileStyleToPlace", 1, 0, env, src, line)
    ctx.row["width"] = 14
    ctx.row["height"] = 14
    ctx.row["useStyle"] = 1
    ctx.row["useAnimation"] = 15
    ctx.row["useTime"] = 10
    ctx.row["useTurn"] = True
    ctx.row["autoReuse"] = True
    ctx.row["consumable"] = True


def h_MakeUsableWithChlorophyteExtractinator(ctx, type_value, args, named, src, line, env):
    """Item.cs:48505-48513 (chlorophyteExtractinatorConsumable not provisioned)."""
    ctx.row["useStyle"] = 1
    ctx.row["useAnimation"] = 15
    ctx.row["useTime"] = 10
    ctx.row["autoReuse"] = True
    ctx.row["consumable"] = True


def h_DefaultToGolfClub(ctx, type_value, args, named, src, line, env):
    """Item.cs:48515-48527 (UseSound not provisioned)."""
    ctx.row["width"] = eval_expr(args[0], env, src, line)
    ctx.row["height"] = eval_expr(args[1], env, src, line)
    ctx.row["channel"] = True
    ctx.row["useStyle"] = 8
    ctx.row["holdStyle"] = 4
    ctx.row["shootSpeed"] = 6.0
    ctx.row["shoot"] = 722
    ctx.row["useAnimation"] = 12
    ctx.row["useTime"] = 12


def h_DefaultToLawnMower(ctx, type_value, args, named, src, line, env):
    """Item.cs:48529-48539 (UseSound not provisioned)."""
    ctx.row["width"] = eval_expr(args[0], env, src, line)
    ctx.row["height"] = eval_expr(args[1], env, src, line)
    ctx.row["holdStyle"] = 1
    ctx.row["useStyle"] = 11
    ctx.row["useAnimation"] = 30
    ctx.row["useTime"] = 10
    ctx.row["autoReuse"] = True


HELPER_TABLE = {
    "SetShopValues": h_SetShopValues,
    "DefaultToFood": h_DefaultToFood,
    "DefaultToHealingPotion": h_DefaultToHealingPotion,
    "DefaultToHeadgear": h_DefaultToHeadgear,
    "DefaultToBody": h_DefaultToBody,
    "DefaultToLegs": h_DefaultToLegs,
    "DefaultToAccessory": h_DefaultToAccessory,
    "DefaultToInfoAccessory": h_DefaultToInfoAccessory,
    "DefaultToGuitar": h_DefaultToGuitar,
    "DefaultToMusicBox": h_DefaultToMusicBox,
    "DefaultToQuestFish": h_DefaultToQuestFish,
    "DefaultToGolfBall": h_DefaultToGolfBall,
    "DefaultToBanner": h_DefaultToBanner,
    "DefaultToMonolith": h_DefaultToMonolith,
    "DefaultToBomb": h_DefaultToBomb,
    "DefaultToVoiceOverrideAccessory": h_DefaultToVoiceOverrideAccessory,
    "DefaultToSolution": h_DefaultToSolution,
    "DefaultToWhip": h_DefaultToWhip,
    "DefaultToKite": h_DefaultToKite,
    "DefaultToVanitypet": h_DefaultToVanitypet,
    "DefaultToSeaShell": h_DefaultToSeaShell,
    "DefaultToCapturedCritter": h_DefaultToCapturedCritter,
    "DefaultToStaff": h_DefaultToStaff,
    "DefaultToSpear": h_DefaultToSpear,
    "DefaultToMinecart": h_DefaultToMinecart,
    "DefaultToPlaceableWall": h_DefaultToPlaceableWall,
    "SetWeaponValues": h_SetWeaponValues,
    "DefaultToBow": h_DefaultToBow,
    "DefaultToMagicWeapon": h_DefaultToMagicWeapon,
    "DefaultToRangedWeapon": h_DefaultToRangedWeapon,
    "DefaultToThrownWeapon": h_DefaultToThrownWeapon,
    "DefaultToTorch": h_DefaultToTorch,
    "DefaultToPlaceableTile": h_DefaultToPlaceableTile,
    "MakeUsableWithChlorophyteExtractinator": h_MakeUsableWithChlorophyteExtractinator,
    "DefaultToGolfClub": h_DefaultToGolfClub,
    "DefaultToLawnMower": h_DefaultToLawnMower,
}


# ---------------------------------------------------------------------------
# ResetStats (Item.cs:48944-49051) — the stock defaults every row starts from
# ---------------------------------------------------------------------------

def stock_row():
    return {
        "type": 0,
        "crit": 0,
        "reuseDelay": 0,
        "placeStyle": 0,
        "buffTime": 0,
        "buffType": 0,
        "material": False,
        "mana": 0,
        "channel": False,
        "shootSpeed": 0.0,
        "ammo": 0,          # AmmoID.None (ItemID.cs:554)
        "useAmmo": 0,
        "autoReuse": False,
        "healMana": 0,
        "bodySlot": -1,
        "legSlot": -1,
        "headSlot": -1,
        "potion": False,
        "consumable": False,
        "createTile": -1,
        "createWall": -1,
        "damage": -1,
        "defense": 0,
        "healLife": 0,
        "holdStyle": 0,
        "knockBack": 0.0,
        "maxStack": 9999,   # Item.CommonMaxStack (Item.cs:49966)
        "rare": 0,
        "scale": 1.0,
        "shoot": 0,
        "stack": 1,
        "tileBoost": 0,
        "useStyle": 0,
        "useTime": 100,
        "useAnimation": 100,
        "value": 0,
        "useTurn": False,
        "bait": 0,
        "pick": 0,
        "axe": 0,
        "hammer": 0,
        "accessory": False,
        "melee": False,
        "ranged": False,
        "magic": False,
        "summon": False,
        "thrown": False,
        "width": 0,         # C# field defaults (not reset by ResetStats)
        "height": 0,
    }


# ---------------------------------------------------------------------------
# Method routing (dispatcher Item.cs:48689-48712)
# ---------------------------------------------------------------------------

METHOD_SPANS = [
    ("SetDefaults1", 1387, 12929),
    ("SetDefaults2", 12931, 21378),
    ("SetDefaults3", 21380, 27649),
    ("SetDefaults4", 27659, 35442),
    ("SetDefaults5", 35462, 47751),
    ("SetFoodDefaults", 48040, 48393),
]


def route_method(item_id, is_food):
    """The dispatcher's routing chain (Item.cs:48685-48712)."""
    if item_id == 0:
        return None  # stack = 0 only; SetDefaults1(0) has no case
    if is_food:
        return "SetFoodDefaults"
    if item_id <= 1000:
        return "SetDefaults1"
    if item_id <= 2001:
        return "SetDefaults2"
    if item_id <= 3000:
        return "SetDefaults3"
    if item_id <= 3989:
        return "SetDefaults4"
    return "SetDefaults5"


def run_method(blocks, item_id, ctx, src):
    """Execute the routed method's top-level blocks for one id.

    Pseudocode (run one item through a SetDefaultsN/SetFoodDefaults body):
        for each top-level block, in source order (a `return` anywhere stops
        the whole method — C# exits the function):
            switch block: run the id's case body, else the default body;
            a `break` ends the case (cleared before the next block)
            if-range block: evaluate the condition, run the taken branch
    """
    for block in blocks:
        if ctx.returned:
            return
        kind = block[0]
        if kind == "switch":
            _, line, cmap, default = block
            body = cmap[item_id] if item_id in cmap else default
            if body:
                exec_stmts(body, ctx, item_id, src, cmap, in_switch=True)
            if ctx.brk:
                ctx.brk = False
        elif kind == "if":
            _, line, cond, body, else_body = block
            if eval_cond(cond, ctx.env(item_id), src, line):
                exec_stmts(body, ctx, item_id, src)
            elif else_body:
                exec_stmts(else_body, ctx, item_id, src)
            if ctx.brk:
                fail(src, line, "break escaped a top-level if-range")
        else:
            # `return;` between top-level switches (SD4, Item.cs:31270) and any
            # other leaf statement — delegate to the statement executor.
            exec_stmts([block], ctx, item_id, src)


# Pseudocode (build_rows — the whole dispatcher, Item.cs:48663-48926, per id):
#     row ← stock defaults (ResetStats)
#     row.material ← IsAMaterial[id]                      (:48684, placeholder pass)
#     if id == 0: row.stack ← 0                           (:48685-48688)
#     route: id==0 → nothing; IsFood → SetFoodDefaults;   (:48689-48712)
#            else SetDefaults1..5 by range — run the case body
#     run the epilogue statements                          (:48713-48925)
#       (the 1803-1807 redirect copies the already-final row of 1533+id-1803
#        in place, so later epilogue steps see the redirected state)
#     if Deprecated[id]: row ← row 0's data (TurnToAir ⇒ SetDefaults(0))
def build_rows(parsed, tables, ids, prices, is_food, names):
    rows = []
    all_variants = []
    deviations = []
    material_written = []
    epilogue = parsed["epilogue"]
    for item_id in range(ITEM_COUNT):
        ctx = Ctx(ids, prices, tables)
        ctx.methods = parsed
        ctx.row = stock_row()
        ctx.row["type"] = item_id
        if item_id == 0:
            # dispatcher `if (this.type == 0) this.stack = 0;` (Item.cs:48685-48688)
            ctx.row["stack"] = 0
        method = route_method(item_id, item_id in is_food)
        if method:
            run_method(parsed[method], item_id, ctx, "Item.cs")
        ctx.returned = False
        exec_stmts(epilogue, ctx, item_id, "Item.cs")
        if ctx.redirect_to is not None:
            if ctx.redirect_to >= len(rows):
                fail("Item.cs", 48919, "redirect target %d not built yet" % ctx.redirect_to)
            ctx.row = dict(rows[ctx.redirect_to])
        if ctx.deprecated_hit:
            ctx.row = dict(rows[0])
        # no field recorded in a Variant branch may be written again later for
        # this id — the recorded absolute values must stay exact (C# applies the
        # branch mid-case and later writes would win for base AND variant alike)
        for (_vid, _variant, ops, _kind, line, log_pos) in ctx.variant_rows:
            later = set(ctx.write_log[log_pos:])
            for (_op, field, _value) in ops:
                if field in later:
                    fail("Item.cs", line,
                         "field %s of a Variant branch is written later for id %d"
                         % (field, item_id))
        # `name` rides on the row so redirect copies keep the source item's name
        # and deprecated (TurnToAir) copies keep "None" (id 0's name)
        ctx.row.setdefault("name", names[item_id])
        rows.append(ctx.row)
        all_variants.extend(ctx.variant_rows)
        deviations.extend(ctx.deviation_sites)
        material_written.append(ctx.material_written)
    if not tables["deprecated"].isdisjoint(range(1803, 1808)):
        fail("Item.cs", 48922, "a redirected id is deprecated")
    return rows, all_variants, deviations, material_written


# ---------------------------------------------------------------------------
# ItemID.Sets.IsAMaterial — runtime-filled from the recipe database
# (Recipe.cs:15205-15239 UpdateWhichItemsAreMaterials)
# ---------------------------------------------------------------------------

def parse_recipe_groups(recipe_lines):
    """`RecipeGroups.NAME = new RecipeGroup(key, ids...).Register();`
    (Recipe.cs SetupRecipeGroups, :497-533) → {name: [member ids]}."""
    groups = {}
    pat = re.compile(r"RecipeGroups\.(\w+) = new RecipeGroup\([^;]*?\);")
    for i, line in enumerate(recipe_lines):
        m = pat.search(line)
        if m:
            ids = [int(x) for x in re.findall(r"(?<![\w.])(\d+)", line.split("RecipeGroup(")[1])]
            groups[m.group(1)] = ids
    if len(groups) < 30:
        fail("Recipe.cs", 497, "expected ~37 recipe groups, got %d" % len(groups))
    return groups


def scan_recipes(recipe_lines, groups, full_ids, texture_copy_load):
    """Linear scan of `SetupRecipes` (Recipe.cs:604-15160, no conditional guards).

    Pseudocode (collect recipe ingredients for the material set):
        walk the lines with a brace-depth stack:
            `for (int V = A; V <= B; V++)` → remember the loop var and its
                range; drop it when the depth rises above its body
            `int[,] NAME = ... { {a, b}, ... }` → remember the rows
            `NAME = EXPR;` / `int NAME = EXPR;` → locals (int arithmetic)
            `requiredItem[N].SetDefaults(ARG);` → resolve ARG (literal, ItemID
                const, local, loop-var expression, array column, or
                TextureCopyLoad values) — each id is an ingredient
            `RequireGroup(RecipeGroups.X);` → all group members are ingredients
            `createItem.SetDefaults(ARG);` → remember the crafted id
            `AddRecipe();` → close the recipe (create, [ingredients])
            anything else mentioning requiredItem/createItem/RequireGroup → fail
        returns (direct ids, used-group members, [(create, [ingredients])])
    """
    direct = set()
    used_group_members = set()
    recipes = []
    locals_ = {}
    loops = []   # (var, lo, hi_inclusive, depth_at_for)
    arrays = {}  # name -> list of row tuples
    collecting = None  # name of a multi-line int[,] being collected
    depth = 0
    cur_create = [0]
    cur_ings = [None]  # boxed list so inner helpers can reset it

    def resolve(arg, line):
        arg = arg.strip()
        if re.match(r"^\d+$", arg):
            return [int(arg)]
        if arg.startswith("ItemID.Sets.TextureCopyLoad["):
            return list(texture_copy_load)
        if arg.startswith("ItemID."):
            name = arg[len("ItemID."):]
            if name not in full_ids["ItemID"]:
                fail("Recipe.cs", line, "unknown ItemID const " + name)
            return [full_ids["ItemID"][name]]
        if arg in locals_:
            return [locals_[arg]]
        m = re.match(r"(\w+)\[\w+, (\d+)\]$", arg)
        if m and m.group(1) in arrays:
            col = int(m.group(2))
            return [row[col] for row in arrays[m.group(1)]]
        # loop-var arithmetic: expand over every active loop range
        active = [l for l in loops]
        names = set(re.findall(r"\b(\w+)\b", arg)) & {l[0] for l in active}
        if len(names) == 1:
            var = names.pop()
            rng = next(l for l in active if l[0] == var)
            out = []
            for v in range(rng[1], rng[2] + 1):
                env = dict(locals_)
                env[var] = v
                out.append(eval_expr(arg, {"ids": full_ids,
                                            "prices": {}, "row": {}, "locals": env,
                                            "loopvars": {}, "type": 0},
                                      "Recipe.cs", line))
            return out
        fail("Recipe.cs", line, "cannot resolve ingredient argument: " + arg)

    for i, raw in enumerate(recipe_lines):
        line = i + 1
        s = raw.strip()
        depth += raw.count("{") - raw.count("}")
        while loops and depth < loops[-1][3]:
            loops.pop()
        if not s or s.startswith("//") or s in ("{", "}"):
            continue
        m = re.match(r"for \(int (\w+) = (\d+); \1 (<=|<) (\d+); \1\+\+\)", s)
        if m:
            hi = int(m.group(4)) if m.group(3) == "<=" else int(m.group(4)) - 1
            loops.append((m.group(1), int(m.group(2)), hi, depth))
            continue
        m = re.match(r"(?:int\[,\] )?(\w+) = new int\[\d+, 2\]\s*\{(.*)\};", s)
        if m and re.search(r"\{\s*\d+\s*,\s*\d+\s*\}", s):
            # single-line rows (`int[,] name = new int[n, 2] { {a, b}, ... };`)
            rows_ = re.findall(r"\{\s*(\d+)\s*,\s*(\d+)\s*\}", s)
            arrays[m.group(1)] = [(int(a), int(b)) for a, b in rows_]
            collecting = None
            continue
        m = re.match(r"(?:int\[,\] )?(\w+) = new int\[\d+, 2\]\s*(?:\{)?$", s)
        if m:
            # multi-line rows (Recipe.cs:6533: `int[,] array = new int[36, 2]`,
            # a bare `{`, then `{ a, b },` lines until `};`)
            collecting = m.group(1)
            arrays[collecting] = []
            continue
        if collecting is not None:
            if s == "{":
                continue
            for a, b in re.findall(r"\{\s*(\d+)\s*,\s*(\d+)\s*\}", s):
                arrays[collecting].append((int(a), int(b)))
            if s.rstrip().endswith("};"):
                collecting = None
            continue
        m = re.match(r"(?:int )?(\w+) = ([^;]+);", s)
        if m and not s.startswith("Recipe") and "SetDefaults" not in s and "(" not in m.group(2):
            try:
                locals_[m.group(1)] = eval_expr(m.group(2), {
                    "ids": full_ids, "prices": {}, "row": {},
                    "locals": dict(locals_), "loopvars": {l[0]: l[1] for l in loops},
                    "type": 0}, "Recipe.cs", line)
            except SystemExit:
                pass  # not a simple int local — ignored (stacks of arrays etc.)
            continue
        m = re.match(r"(?:Recipe\.currentRecipe\.)?requiredItem\[\d+\]\.SetDefaults\((.*)\);", s)
        if m:
            for item in resolve(m.group(1), line):
                if item > 0:
                    direct.add(item)
                    cur_ings[0] = (cur_ings[0] or []) + [item]
            continue
        m = re.match(r"(?:Recipe\.currentRecipe\.)?RequireGroup\(RecipeGroups\.(\w+)\);", s)
        if m:
            if m.group(1) not in groups:
                fail("Recipe.cs", line, "unknown recipe group " + m.group(1))
            used_group_members.update(groups[m.group(1)])
            cur_ings[0] = (cur_ings[0] or []) + list(groups[m.group(1)])
            continue
        m = re.match(r"(?:Recipe\.currentRecipe\.)?createItem\.SetDefaults\((.*)\);", s)
        if m:
            ids = resolve(m.group(1), line)
            cur_create[0] = ids[0] if ids else 0
            continue
        if re.match(r"(?:Recipe\.)?AddRecipe\(\);", s):
            recipes.append((cur_create[0], cur_ings[0] or []))
            cur_create[0] = 0
            cur_ings[0] = None
            continue
        # field tweaks on the recipe's items (`createItem.stack = 3;`,
        # Recipe.cs SetupRecipes) do not change which ids are ingredients
        m = re.match(r"(?:Recipe\.currentRecipe\.)?(?:createItem|requiredItem\[\d+\])\.\w+ = ", s)
        if m:
            continue
        if re.search(r"requiredItem|createItem|RequireGroup|AddIngredient", s):
            fail("Recipe.cs", line, "unrecognized recipe line: " + s[:100])
    return direct, used_group_members, recipes


def compute_material(rows, itemid_text, ids):
    """The full IsAMaterial set (Recipe.cs:15205-15239 semantics)."""
    recipe_lines = read_lines(RECIPE_CS)
    groups = parse_recipe_groups(recipe_lines)
    texture_copy_load = parse_int_set_values(itemid_text, "Sets.TextureCopyLoad", "ItemID.cs")
    # scan ONLY the SetupRecipes body (Recipe.cs:604-15160) — the whole file
    # would feed class field declarations into the statement scan.
    lo, hi = find_method_span(recipe_lines, "public static void SetupRecipes()", "Recipe.cs")
    direct, group_members, recipes = scan_recipes(
        recipe_lines[lo + 1:hi - 1], groups, ids, texture_copy_load)
    tileid_text = "\n".join(read_lines(TILEID_CS))
    platforms = parse_bool_set(tileid_text, "Sets.Platforms", "TileID.cs")

    def create_tile(item_id):
        v = rows[item_id]["createTile"]
        return v if v >= 0 else None

    def create_wall(item_id):
        return rows[item_id]["createWall"]

    reverse_platform = set()
    reverse_wall = set()
    for create, ings in recipes:
        if create <= 0 or len(ings) > 1:
            continue
        if create_tile(create) in platforms:
            reverse_platform.add(create)
        if create_wall(create) > 0 and (not ings or create_wall(ings[0]) < 0):
            reverse_wall.add(create)
    material = direct | group_members | reverse_platform | reverse_wall
    return material


# ---------------------------------------------------------------------------
# ItemVariants (GameContent/Items/ItemVariants.cs:109-146)
# ---------------------------------------------------------------------------

def parse_variant_registrations():
    """`ItemVariants.AddVariant(id, ItemVariants.XVariant, ItemVariants.YCondition)`
    (:121-145) → [(id, variant, condition)] in registration order."""
    lines = read_lines(ITEMVARIANTS_CS)
    out = []
    pat = re.compile(r"ItemVariants\.AddVariant\((\d+), ItemVariants\.(\w+)Variant,"
                     r"\s*ItemVariants\.(\w+)\)")
    for i, line in enumerate(lines):
        m = pat.search(line)
        if m:
            out.append((int(m.group(1)), m.group(2), m.group(3)))
    if len(out) != 25:
        fail("ItemVariants.cs", 121, "expected 25 AddVariant registrations, got %d" % len(out))
    return out


def parse_net_defaults(item_lines):
    """netDefaults (Item.cs:1211-1367): case -N → SetDefaults(M)."""
    span = "\n".join(item_lines[1210:1367])
    pairs = re.findall(r"case (-\d+):\s*\n\s*this\.SetDefaults\((\d+)\);", span)
    out = sorted({(int(a), int(b)) for a, b in pairs})
    if len(out) != 48:
        fail("Item.cs", 1211, "expected 48 netDefaults cases, got %d" % len(out))
    return out


def parse_on_created(item_lines):
    """OnCreated (Item.cs:48936-48942): type 5437 → SetDefaults(5358)."""
    span = "\n".join(item_lines[48935:48942])
    pairs = re.findall(r"this\.type == (\d+)\)\s*\{\s*this\.SetDefaults\((\d+)\);", span)
    out = sorted({(int(a), int(b)) for a, b in pairs})
    if out != [(5437, 5358)]:
        fail("Item.cs", 48936, "unexpected OnCreated table: %s" % out)
    return out


def parse_projhook_set():
    """Projectile ids whose SetDefaults assigns `aiStyle = 7` — Main.cs:6612-6624
    sets Main.projHook[i] for exactly those (aiStyle 7 = grapple)."""
    lines = read_lines(PROJECTILE_CS)
    span_lo, span_hi = find_method_span(lines, "public void SetDefaults(int Type)", "Projectile.cs")
    ids = []
    cur = None
    for i in range(span_lo, span_hi):
        s = lines[i].strip()
        m = re.match(r"(?:else )?if \(this\.type == (\d+)\)", s)
        if m:
            cur = int(m.group(1))
        if s == "this.aiStyle = 7;" and cur is not None:
            ids.append(cur)
    if len(ids) != 20:
        fail("Projectile.cs", span_lo, "expected 20 aiStyle = 7 sites, got %d" % len(ids))
    return set(ids)



# ---------------------------------------------------------------------------
# Emission
# ---------------------------------------------------------------------------

# (rust_name, row_key, kind) — order is the struct's field order
RUST_FIELDS = [
    ("id", "id", "id"),
    ("name", None, "name"),
    ("max_stack", "maxStack", "u32"),
    ("tile", "createTile", "option_pos"),
    ("place_style", "placeStyle", "i32"),
    ("wall", "createWall", "option_pos"),
    ("projectile", "shoot", "option_pos1"),
    ("damage", "damage", "i32"),
    ("rare", "rare", "i32"),
    ("use_time", "useTime", "u8"),
    ("use_animation", "useAnimation", "u8"),
    ("use_style", "useStyle", "u8"),
    ("hold_style", "holdStyle", "u8"),
    ("auto_reuse", "autoReuse", "bool"),
    ("head_slot", "headSlot", "i16"),
    ("body_slot", "bodySlot", "i16"),
    ("leg_slot", "legSlot", "i16"),
    ("defense", "defense", "u8"),
    ("class", None, "class"),
    ("crit", "crit", "i32"),
    ("value", "value", "i32"),
    ("knock_back", "knockBack", "f32"),
    ("mana", "mana", "i32"),
    ("use_ammo", "useAmmo", "i32"),
    ("ammo", "ammo", "i32"),
    ("channel", "channel", "bool"),
    ("reuse_delay", "reuseDelay", "i32"),
    ("consumable", "consumable", "bool"),
    ("use_turn", "useTurn", "bool"),
    ("buff_type", "buffType", "i32"),
    ("buff_time", "buffTime", "i32"),
    ("shoot_speed", "shootSpeed", "f32"),
    ("width", "width", "i32"),
    ("height", "height", "i32"),
    ("material", "material", "bool"),
    ("potion", "potion", "bool"),
    ("heal_life", "healLife", "i32"),
    ("heal_mana", "healMana", "i32"),
    ("scale", "scale", "f32"),
    ("stack", "stack", "u32"),
    ("bait", "bait", "u16"),
    ("tile_boost", "tileBoost", "i32"),
    ("pick", "pick", "u16"),
    ("axe", "axe", "u16"),
    ("hammer", "hammer", "u16"),
    ("accessory", "accessory", "bool"),
]

TYPE_RANGES = {
    "u8": (0, 255), "u16": (0, 65535), "u32": (0, 2**32 - 1),
    "i16": (-32768, 32767), "i32": (-2**31, 2**31 - 1),
}

CLASS_BY_FLAG = [("melee", "Melee"), ("ranged", "Ranged"), ("magic", "Magic"),
                 ("summon", "Summon"), ("thrown", "Thrown")]


def derive_class(row, item_id, multi_flags):
    flags = [rust for key, rust in CLASS_BY_FLAG if row.get(key)]
    if len(flags) > 1:
        multi_flags.append((item_id, flags))
    return "DamageClass::" + (flags[0] if flags else "None")


def rust_float(v):
    s = repr(float(v))
    return s if ("." in s or "e" in s or "inf" in s) else s + ".0"


def emit_rows(rows, names, multi_flags):
    out = []
    for item_id, row in enumerate(rows):
        parts = []
        for rust, key, kind in RUST_FIELDS:
            if kind == "name":
                parts.append("name: %s" % rust_str(display_split(row["name"])))
                continue
            if kind == "id":
                parts.append("id: %d" % item_id)
                continue
            if kind == "class":
                parts.append("class: %s" % derive_class(row, item_id, multi_flags))
                continue
            if kind == "option_pos" or kind == "option_pos1":
                v = row[key]
                # createTile/createWall: < 0 = none; shoot: <= 0 = none
                none_at = 0 if kind == "option_pos1" else -1
                parts.append("%s: %s" % (rust, "None" if v <= none_at else "Some(%d)" % v))
                continue
            if kind == "bool":
                parts.append("%s: %s" % (rust, "true" if row[key] else "false"))
                continue
            if kind == "f32":
                parts.append("%s: %s" % (rust, rust_float(row[key])))
                continue
            lo, hi = TYPE_RANGES[kind]
            v = row[key]
            if not lo <= v <= hi:
                fail("itemdata.rs", item_id,
                     "field %s value %s out of %s range" % (rust, v, kind))
            parts.append("%s: %d" % (rust, v))
        out.append("    ItemDef { %s }," % ", ".join(parts))
    return out


def display_split(ident):
    """Raw `ItemID` identifier → display name ("DirtBlock" → "Dirt Block")."""
    t = ident.replace("_", " ")
    t = re.sub(r"(?<=[a-z0-9])(?=[A-Z])", " ", t)
    t = re.sub(r"(?<=[A-Z])(?=[A-Z][a-z])", " ", t)
    return t


def rust_str(s):
    return '"%s"' % s.replace("\\", "\\\\").replace('"', '\\"')


STRUCT_TEMPLATE = """/// One item's effective defaults — what `Item.SetDefaults(type)` leaves on a
/// fresh `Item` (`Item.cs:48663-48926`: the `ResetStats` stock (`:48944-49051`),
/// the case body, then the dispatcher epilogue `:48713-48925`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ItemDef {
    pub id: u16,
    /// The `ItemID` const name (`ID/ItemID.cs`) — the identifier, not the
    /// localized display name.
    pub name: &'static str,
    /// `Item.maxStack` (`Item.cs:49017`, `CommonMaxStack = 9999`).
    pub max_stack: u32,
    /// `Item.createTile` (`Item.cs:49009`, stock −1); `Some(tile id)` when the item places one.
    pub tile: Option<u16>,
    /// `Item.placeStyle` (`Item.cs:48979`, stock 0).
    pub place_style: i32,
    /// `Item.createWall` (`Item.cs:49010`, stock −1).
    pub wall: Option<u16>,
    /// `Item.shoot` (`Item.cs:49021`, stock 0): the vanilla projectile id fired.
    pub projectile: Option<u16>,
    /// `Item.damage` (`Item.cs:49011`, stock −1).
    pub damage: i32,
    /// `Item.rare` (`Item.cs:49019`, stock 0).
    pub rare: i32,
    /// `Item.useTime` (`Item.cs:49028`, stock 100).
    pub use_time: u8,
    /// `Item.useAnimation` (`Item.cs:49029`, stock 100).
    pub use_animation: u8,
    /// `Item.useStyle` (`Item.cs:49026`, stock 0).
    pub use_style: u8,
    /// `Item.holdStyle` (`Item.cs:49015`, stock 0; 1 = torch-style hold).
    pub hold_style: u8,
    /// `Item.autoReuse` (`Item.cs:48997`, stock false).
    pub auto_reuse: bool,
    /// `Item.headSlot` (`Item.cs:49003`, stock −1 = none).
    pub head_slot: i16,
    /// `Item.bodySlot` (`Item.cs:49001`, stock −1).
    pub body_slot: i16,
    /// `Item.legSlot` (`Item.cs:49002`, stock −1).
    pub leg_slot: i16,
    /// `Item.defense` (`Item.cs:49012`, stock 0).
    pub defense: u8,
    /// Derived from the `melee`/`ranged`/`magic`/`summon`/`thrown` flags
    /// (first of `Melee, Ranged, Magic, Summon, Thrown`; `None` when classless).
    pub class: DamageClass,
    /// `Item.crit` (`Item.cs:48969`, stock 0).
    pub crit: i32,
    /// `Item.value` in copper (`Item.cs:49030`, stock 0; `sellPrice` is ×5, `Item.cs:49422-49425`).
    pub value: i32,
    /// `Item.knockBack` (`Item.cs:49016`, stock 0).
    pub knock_back: f32,
    /// `Item.mana` (`Item.cs:48987`, stock 0).
    pub mana: i32,
    /// `Item.useAmmo` (`Item.cs:48996`, stock `AmmoID.None` = 0).
    pub use_ammo: i32,
    /// `Item.ammo` (`Item.cs:48995`, stock `AmmoID.None` = 0).
    pub ammo: i32,
    /// `Item.channel` (`Item.cs:48988`, stock false).
    pub channel: bool,
    /// `Item.reuseDelay` (`Item.cs:48974`, stock 0).
    pub reuse_delay: i32,
    /// `Item.consumable` (`Item.cs:49007`, stock false).
    pub consumable: bool,
    /// `Item.useTurn` (`Item.cs:49031`, stock false).
    pub use_turn: bool,
    /// `Item.buffType` (`Item.cs:48981`, stock 0).
    pub buff_type: i32,
    /// `Item.buffTime` (`Item.cs:48980`, stock 0).
    pub buff_time: i32,
    /// `Item.shootSpeed` (`Item.cs:48993`, stock 0).
    pub shoot_speed: f32,
    /// `Item.width` — C# instance-field default 0 (`ResetStats` never resets it).
    pub width: i32,
    /// `Item.height` — as `width`.
    pub height: i32,
    /// `Item.material` (`Item.cs:48684` reads `ItemID.Sets.IsAMaterial`; the
    /// static set is empty, `ItemID.cs:487` — the value is runtime-filled from
    /// the recipe database and derived generator-side, `Recipe.cs:15205-15239`).
    pub material: bool,
    /// `Item.potion` (`Item.cs:49004`, stock false).
    pub potion: bool,
    /// `Item.healLife` (`Item.cs:49014`, stock 0).
    pub heal_life: i32,
    /// `Item.healMana` (`Item.cs:49000`, stock 0).
    pub heal_mana: i32,
    /// `Item.scale` (`Item.cs:49020`, stock 1).
    pub scale: f32,
    /// `Item.stack` (`Item.cs:49022`, stock 1; 0 for id 0 / air).
    pub stack: u32,
    /// `Item.bait` (`Item.cs:48960`, stock 0; drives the epilogue rare ladder).
    pub bait: u16,
    /// `Item.tileBoost` (`Item.cs:49027`, stock 0) — the tool range bonus
    /// the tooltip prints through `Lang.tip[54]` (`Main.cs:20865-20875`).
    pub tile_boost: i32,
    /// `Item.pick` — pickaxe power (`Item.cs:49018`, stock 0).
    pub pick: u16,
    /// `Item.axe` — axe power (`Item.cs:48999`, stock 0).
    pub axe: u16,
    /// `Item.hammer` — hammer power (`Item.cs:49013`, stock 0).
    pub hammer: u16,
    /// `Item.accessory` (`Item.cs:48998`, stock false).
    pub accessory: bool,
}"""


def emit_variant_tables(registrations, variant_rows):
    """registrations: [(id, variant, condition)]; variant_rows: interpreter rows."""
    lines = []
    lines.append("""/// The five `ItemVariant`s (`GameContent/Items/ItemVariants.cs:112-116`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ItemVariant {
    Stronger,
    Weaker,
    Rebalanced,
    Enabled,
    DisabledBossSummon,
}

/// The four world-seed conditions (`ItemVariants.cs:117-120`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SeedCondition {
    RemixWorld,
    GetGoodWorld,
    Mechdusa,
    Skyblock,
}

/// Fields a `Variant` stat-delta branch touches (the union over
/// `Item.cs`'s 26 `this.Variant ==/!=` branches).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VField {
    Rare,
    Value,
    Damage,
    UseTime,
    UseAnimation,
    Mana,
    Defense,
    UseStyle,
    Consumable,
    Material,
    AutoReuse,
    KnockBack,
    Shoot,
    ShootSpeed,
    Scale,
    UseTurn,
    Width,
    Height,
    BuffType,
    Ammo,
    MaxStack,
    Melee,
    Ranged,
    Magic,
    Summon,
    Thrown,
    Hammer,
    Pick,
    Axe,
    TileBoost,
    Bait,
    Crit,
    UseAmmo,
    NoMelee,
    NoUseGraphic,
    Channel,
    ReuseDelay,
    BuffTime,
    HealLife,
    Potion,
    Accessory,
    Vanity,
    HeadSlot,
    BodySlot,
    LegSlot,
    PlaceStyle,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum VVal {
    Int(i64),
    Float(f32),
    Bool(bool),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum VOp {
    Set(VField, VVal),
}

/// One row of the ItemVariants data:
/// - `condition: Some(c)` — a registration row (`ItemVariants.AddVariant`,
///   `ItemVariants.cs:121-145`); `variants::select` gates on it.
/// - `condition: None` — a delta row from an in-case `Variant ==` branch, or a
///   restore row for a `Variant !=` branch (the branch body applies to the
///   no-variant state; the restore row reverses it when the variant IS
///   selected); `variants::apply` consumes `set`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VariantOverride {
    pub id: u16,
    pub variant: ItemVariant,
    pub condition: Option<SeedCondition>,
    pub set: &'static [VOp],
}""")
    cond_map = {"RemixWorld": "RemixWorld", "GetGoodWorld": "GetGoodWorld",
                "MechdusaWorld": "Mechdusa", "SkyblockWorld": "Skyblock"}
    lines.append("")
    lines.append("pub static VARIANT_OVERRIDES: &[VariantOverride] = &[")
    for item_id, variant, condition in registrations:
        lines.append("    VariantOverride { id: %d, variant: ItemVariant::%s, "
                     "condition: Some(SeedCondition::%s), set: &[] },"
                     % (item_id, variant, cond_map[condition]))
    field_map = {"rare": "Rare", "value": "Value", "damage": "Damage",
                 "useTime": "UseTime", "useAnimation": "UseAnimation",
                 "mana": "Mana", "defense": "Defense", "useStyle": "UseStyle",
                 "consumable": "Consumable", "material": "Material",
                 "autoReuse": "AutoReuse", "knockBack": "KnockBack",
                 "shoot": "Shoot", "shootSpeed": "ShootSpeed",
                 "scale": "Scale", "useTurn": "UseTurn", "width": "Width",
                 "height": "Height", "buffType": "BuffType", "ammo": "Ammo",
                 "maxStack": "MaxStack", "melee": "Melee", "ranged": "Ranged",
                 "magic": "Magic", "summon": "Summon", "thrown": "Thrown",
                 "hammer": "Hammer", "pick": "Pick", "axe": "Axe",
                 "tileBoost": "TileBoost", "bait": "Bait", "crit": "Crit",
                 "useAmmo": "UseAmmo", "noMelee": "NoMelee",
                 "noUseGraphic": "NoUseGraphic", "channel": "Channel",
                 "reuseDelay": "ReuseDelay", "buffTime": "BuffTime",
                 "healLife": "HealLife", "potion": "Potion",
                 "accessory": "Accessory", "vanity": "Vanity",
                 "headSlot": "HeadSlot", "bodySlot": "BodySlot",
                 "legSlot": "LegSlot", "placeStyle": "PlaceStyle"}
    for (item_id, variant, ops, kind, line_no, _pos) in variant_rows:
        op_parts = []
        for (op, field, value) in ops:
            if op != "Set":
                fail("Item.cs", line_no, "unknown variant op " + op)
            if isinstance(value, bool):
                val = "VVal::Bool(%s)" % ("true" if value else "false")
            elif isinstance(value, float):
                val = "VVal::Float(%s)" % (repr(value),)
            elif isinstance(value, int):
                val = "VVal::Int(%d)" % value
            else:
                fail("Item.cs", line_no, "non-int/bool variant op value %r" % (value,))
            op_parts.append("VOp::Set(VField::%s, %s)" % (field_map[field], val))
        lines.append("    // Item.cs:%d (%s)" % (line_no, kind))
        lines.append("    VariantOverride { id: %d, variant: ItemVariant::%s, "
                     "condition: None, set: &[%s] },"
                     % (item_id, variant, ", ".join(op_parts)))
    lines.append("];")
    return "\n".join(lines)


def emit_net_defaults(net_defaults):
    lines = ["/// `Item.netDefaults` negative-id table (`Item.cs:1211-1367`):",
             "/// net id −N → the vanilla item id it names.",
             "pub static NEGATIVE_DEFAULTS: [(i16, u16); %d] = [" % len(net_defaults)]
    chunk = ["(%d, %d)" % (neg, pos) for (neg, pos) in net_defaults]
    for i in range(0, len(chunk), 6):
        lines.append("    " + ", ".join(chunk[i:i + 6]) + ",")
    lines.append("];")
    return "\n".join(lines)


def emit_names(names):
    lines = ["/// `ItemID` const names by id (`ID/ItemID.cs`; `Count = 6196` at :14861).",
             "pub static NAMES: [&str; %d] = [" % len(names)]
    for i in range(0, len(names), 8):
        lines.append("    " + ", ".join(rust_str(n) for n in names[i:i + 8]) + ",")
    lines.append("];")
    return "\n".join(lines)


def emit_on_created(on_created):
    return ("/// `Item.OnCreated` special cases (`Item.cs:48936-48942`):\n"
            "/// crafting id A yields the defaults of id B.\n"
            "pub static ON_CREATED: &[(u16, u16)] = &[%s];"
            % ", ".join("(%d, %d)" % p for p in on_created))


# DW-1.2 spot sample — (id, [(rust field, expected, citation)]). The expected
# values are hand-verified against the cited Item.cs lines (Phase 1 build).
SPOT_CHECKS = [
    (2, [
        ("tile", ("Some", 0), "Item.cs:1413 createTile = 0 (case 2, :1406-1415)"),
        ("max_stack", 9999, "ResetStats CommonMaxStack = 9999 (Item.cs:49017, :49966)"),
        ("use_time", 10, "Item.cs:1410 useTime = 10 (case 2)"),
        ("use_animation", 15, "Item.cs:1409 useAnimation = 15 (case 2)"),
        ("use_style", 1, "Item.cs:1407 useStyle = 1 (case 2)"),
        ("auto_reuse", True, "Item.cs:1411 autoReuse = true (case 2)"),
        ("value", 0, "ResetStats stock (Item.cs:49030)"),
        ("material", True, "IsAMaterial via recipes (Recipe.cs:15224)"),
    ]),
    # Torch — case 8, Item.cs:1488-1500 (flame/noWet, holdStyle 1, tile 4).
    (8, [
        ("tile", ("Some", 4), "Item.cs:1498 createTile = 4"),
        ("hold_style", 1, "Item.cs:1495 holdStyle = 1"),
        ("use_time", 10, "Item.cs:1494 useTime = 10"),
        ("use_animation", 15, "Item.cs:1493 useAnimation = 15"),
    ]),
    # Cobalt drill — case 385, Item.cs:5869-5881 (SD1 range: id <= 1000
    # routes to SetDefaults1): useTime 13 (:5872), useAnimation 25 (:5871),
    # pick 110 (:5878), rare 4 (:5881); the drill epilogue x0.6 min 1
    # (Item.cs:48884-48895) turns 13->7, 25->15.
    (385, [
        ("use_time", 7, "Item.cs:5872 useTime = 13 -> x0.6 = 7"),
        ("use_animation", 15, "Item.cs:5871 useAnimation = 25 -> x0.6 = 15"),
        ("pick", 110, "Item.cs:5878 pick = 110"),
        ("rare", 4, "Item.cs:5881 rare = 4"),
    ]),
    # Architect Gizmo Pack — case 3061, Item.cs:27677-27683 (SD4 range:
    # 3001-3989 routes to SetDefaults4): width 30, height 30, rare 5,
    # value buyPrice(0,20) = 200000; backSlot 8 (:27683) is a named skip.
    (3061, [
        ("rare", 5, "Item.cs:27681 rare = 5"),
        ("value", 200000, "Item.cs:27682 buyPrice(0,20) = 20 gold"),
        ("width", 30, "Item.cs:27678 width = 30"),
        ("height", 30, "Item.cs:27679 height = 30"),
        ("accessory", True, "Item.cs:27680 accessory = true"),
    ]),
    (25, [
        ("tile", ("Some", 10), "Item.cs:1677-1687 createTile = 10"),
        ("use_time", 10, "Item.cs:1683 useTime = 10"),
        ("use_animation", 15, "Item.cs:1684 useAnimation = 15"),
        ("use_style", 1, "Item.cs:1681 useStyle = 1"),
        ("auto_reuse", False, "Item.cs:1677-1687 — no autoReuse in the case"),
        ("value", 200, "Item.cs:1684 value = 200"),
    ]),
    (42, [
        ("damage", 10, "Item.cs:1873 damage = 10 (case 42, :1869-1890)"),
        ("projectile", ("Some", 3), "Item.cs:1872 shoot = 3"),
        ("shoot_speed", 9.0, "Item.cs:1871 shootSpeed = 9f (case 42)"),
        ("use_time", 15, "Item.cs:1879 useTime = 15"),
        ("use_ammo", 0, "ResetStats stock (Item.cs:48996) — thrown, no ammo"),
        ("class", "DamageClass::Ranged", "Item.cs:1883 ranged = true"),
    ]),
    (99, [
        ("damage", 8, "Item.cs:2553 damage = 8 (case 99, Iron Bow)"),
        ("shoot_speed", 6.6, "Item.cs:2554 shootSpeed = 6.6f"),
        ("use_ammo", 40, "Item.cs:2549 useAmmo = AmmoID.Arrow (40)"),
        ("use_time", 28, "Item.cs:2547 useTime = 28"),
        ("use_animation", 28, "Item.cs:2546 useAnimation = 28"),
        ("auto_reuse", False, "Item.cs:1788-1796 — no autoReuse"),
    ]),
    # 112 is the Flower of Fire (Item.cs:2710-2723); Muramasa is 155 — the
    # original pins here carried Muramasa data and were replaced (M3-ITEM-1).
    (112, [
        ("damage", 48, "Item.cs:2711 damage = 48"),
        ("mana", 9, "Item.cs:2710 mana = 9"),
        ("use_animation", 16, "Item.cs:2720 useAnimation = 16"),
        ("use_time", 16, "Item.cs:2721 useTime = 16"),
        ("rare", 3, "Item.cs:2718 rare = 3"),
        ("value", 125000, "Item.cs:2714 hellPrice = sellPrice(0,2,50) = 125000"),
        ("knock_back", 5.5, "Item.cs:2719 knockBack = 5.5f"),
        ("width", 26, "Item.cs:2716 width = 26"),
        ("height", 28, "Item.cs:2717 height = 28"),
        ("class", "DamageClass::Magic", "Item.cs:2722 magic = true"),
        ("projectile", ("Some", 15), "Item.cs:2713 shoot = 15"),
    ]),
    # 1881 is the Red & Green Garland (the plan's "potion ~1881" guess was
    # wrong); the potion pin moved to id 28 (Item.cs:1711-1722).
    (1881, [
        ("value", 500, "Item.cs case 1881: value = 500 (garland)"),
        ("use_time", 10, "Item.cs case 1881: useTime = 10"),
        ("use_animation", 15, "Item.cs case 1881: useAnimation = 15"),
        ("auto_reuse", True, "Item.cs case 1881: autoReuse = true"),
    ]),
    # Lesser Healing Potion — Item.cs:1711-1722
    (28, [
        ("heal_life", 50, "Item.cs:1712 healLife = 50"),
        ("potion", True, "Item.cs:1720 potion = true"),
        ("consumable", True, "Item.cs:1717 consumable = true"),
        ("use_style", 9, "Item.cs:1713 useStyle = 9"),
        ("use_time", 17, "Item.cs:1716 useTime = 17"),
        ("use_animation", 17, "Item.cs:1715 useAnimation = 17"),
        ("value", 300, "Item.cs:1721 value = 300"),
        ("width", 14, "Item.cs:1718 width = 14"),
        ("height", 24, "Item.cs:1719 height = 24"),
    ]),
    # 4254 is a golf BALL, not a hook — DefaultToGolfBall (Item.cs:35444-35460)
    # sets its own timings; the projHook zeroing (Item.cs:48893-48898) applies
    # only to hook projectiles (aiStyle 7: 84, 185, 939, 1273, 1800).
    (4254, [
        ("projectile", ("Some", 751), "Item.cs case 4254: DefaultToGolfBall(751)"),
        ("use_style", 1, "DefaultToGolfBall — Item.cs:35448-35460"),
        ("use_time", 15, "DefaultToGolfBall — Item.cs:35458"),
        ("use_animation", 15, "DefaultToGolfBall — Item.cs:35457"),
    ]),
]

SPOT_NAMES = {}


def emit_test_module(spot_checks, food_ids, deprecated_ids, bait_rare3, variant_rows):
    """DW-1.2 tests + food/epilogue shape tests, emitted with citations."""
    lines = []
    lines.append("#[cfg(test)]")
    lines.append("mod tests {")
    lines.append("    #![allow(non_snake_case)] // DW-IDs stay in the test names")
    lines.append("")
    lines.append("    use super::*;")
    lines.append("")
    lines.append("    /// Rows describe the post-SetDefaults state; redirect/deprecated")
    lines.append("    /// rows differ from their source row only in `id`.")
    lines.append("    fn same_except_id(a: &ItemDef, b: &ItemDef) -> bool {")
    lines.append("        let mut x = *a;")
    lines.append("        let mut y = *b;")
    lines.append("        x.id = 0;")
    lines.append("        y.id = 0;")
    lines.append("        x == y")
    lines.append("    }")
    lines.append("")
    lines.append("    // DW-1.2 — spot sample hand-verified field-for-field against Item.cs.")
    lines.append("    #[test]")
    lines.append("    fn test_DW_1_2_spot_sample_matches_item_cs() {")
    for (item_id, checks) in spot_checks:
        lines.append("        // ItemID.%s" % SPOT_NAMES.get(item_id, item_id))
        for (field, expected, citation) in checks:
            lines.append("        // %s" % citation)
            if isinstance(expected, bool):
                exp = "true" if expected else "false"
            elif isinstance(expected, float):
                exp = rust_float(expected)
            elif expected is None:
                exp = "None"
            elif isinstance(expected, tuple) and expected and expected[0] == "Some":
                exp = "Some(%d)" % expected[1]
            elif isinstance(expected, str) and expected.startswith("DamageClass::"):
                exp = expected
            else:
                exp = str(expected)
            lines.append("        assert_eq!(ITEMS[%d].%s, %s);" % (item_id, field, exp))
    lines.append("    }")
    lines.append("")
    lines.append("    // DW-1.4 — food rows carry the DefaultToFood shape; the epilogue")
    lines.append("    // rules are visible in the data.")
    lines.append("    #[test]")
    lines.append("    fn test_DW_1_4_food_and_epilogue_rules_present() {")
    lines.append("        // IsFood ids (ItemID.cs:498, %d of them): SetFoodDefaults" % len(food_ids))
    lines.append("        // (DefaultToFood) or, for case-less 4010, the stock shape + holdStyle 1.")
    lines.append("        let food = [%s];" % ", ".join(str(i) for i in sorted(food_ids)))
    lines.append("        assert_eq!(food.len(), %d);" % len(food_ids))
    lines.append("        // 4010 is Deprecated (ItemID.cs:1316) — TurnToAir zeroes it")
    lines.append("        // (Item.cs:48921-48924); the epilogue holdStyle never applies.")
    lines.append("        assert!(same_except_id(&ITEMS[4010], &ITEMS[0]));")
    lines.append("        for &id in &food {")
    lines.append("            if id == 4010 { continue; }")
    lines.append("            let d = &ITEMS[id as usize];")
    lines.append("            assert_eq!(d.hold_style, 1, \"id {id}\"); // Item.cs:48913-48916")
    lines.append("            if id != 4010 {")
    lines.append("                // DefaultToFood — Item.cs:48541-48568")
    lines.append("                assert!(d.consumable, \"id {id}\");")
    lines.append("                assert!(d.use_turn, \"id {id}\");")
    lines.append("                assert_eq!((d.use_time, d.use_animation), (17, 17), \"id {id}\");")
    lines.append("                // rare/value come from the per-id SetShopValues call —")
    lines.append("                // the DefaultToFood stock (rare 1, buyPrice(0,0,20) = 2000) is")
    lines.append("                // overridden for most foods, so only the timings pin here.")
    lines.append("            }")
    lines.append("        }")
    lines.append("        // bait rare ladder — Item.cs:48838-48852: bait >= 50 → rare 3")
    lines.append("        for &id in &[%s] {" % ", ".join(str(i) for i in bait_rare3))
    lines.append("            assert_eq!(ITEMS[id as usize].rare, 3, \"id {id}\");")
    lines.append("        }")
    lines.append("        // 1803-1807 redirect — Item.cs:48917-48920: rows equal 1533-1537's")
    lines.append("        for k in 0..5 {")
    lines.append("            assert!(same_except_id(&ITEMS[1803 + k], &ITEMS[1533 + k]), \"redirect {k}\");")
    lines.append("        }")
    lines.append("        // deprecated ids → TurnToAir ⇒ the id-0 row — Item.cs:48921-48924, :49884-48990")
    lines.append("        for &id in &[%s] {" % ", ".join(str(i) for i in sorted(deprecated_ids)))
    lines.append("            assert!(same_except_id(&ITEMS[id as usize], &ITEMS[0]), \"deprecated {id}\");")
    lines.append("        }")
    lines.append("    }")
    lines.append("")
    lines.append("    // DW-1.2 — netDefaults table and OnCreated spot checks.")
    lines.append("    #[test]")
    lines.append("    fn test_DW_1_2_negative_defaults_and_on_created() {")
    lines.append("        assert_eq!(NEGATIVE_DEFAULTS.len(), 48);")
    lines.append("        assert_eq!(NEGATIVE_DEFAULTS[47], (-1, 3521)); // Item.cs:1217-1219")
    lines.append("        assert_eq!(NEGATIVE_DEFAULTS[0], (-48, 3480)); // Item.cs:1358-1360")
    lines.append("        assert_eq!(ON_CREATED, &[(5437, 5358)]); // Item.cs:48936-48942")
    lines.append("    }")
    lines.append("")
    lines.append("    // DW-1.2 — the ItemVariants tables (25 registrations + delta rows).")
    lines.append("    #[test]")
    lines.append("    fn test_DW_1_2_variant_tables() {")
    lines.append("        let regs: Vec<_> = VARIANT_OVERRIDES")
    lines.append("            .iter()")
    lines.append("            .filter(|r| r.condition.is_some())")
    lines.append("            .collect();")
    lines.append("        assert_eq!(regs.len(), 25); // ItemVariants.cs:121-145")
    lines.append("        let deltas: Vec<_> = VARIANT_OVERRIDES")
    lines.append("            .iter()")
    lines.append("            .filter(|r| r.condition.is_none())")
    lines.append("            .collect();")
    lines.append("        assert_eq!(deltas.len(), %d);" % len(variant_rows))
    lines.append("        // 112 Stronger — Item.cs:2716-2723")
    lines.append("        let r112 = deltas")
    lines.append("            .iter()")
    lines.append("            .find(|r| r.id == 112 && r.variant == ItemVariant::Stronger)")
    lines.append("            .unwrap();")
    lines.append("        assert!(r112.set.contains(&VOp::Set(VField::Rare, VVal::Int(6))));")
    lines.append("        assert!(r112.set.contains(&VOp::Set(VField::Value, VVal::Int(500000))));")
    lines.append("        assert!(r112.set.contains(&VOp::Set(VField::Damage, VVal::Int(85))));")
    lines.append("        assert!(r112.set.contains(&VOp::Set(VField::UseAnimation, VVal::Int(12))));")
    lines.append("        assert!(r112.set.contains(&VOp::Set(VField::UseTime, VVal::Int(12))));")
    lines.append("        // 544 DisabledBossSummon loses the use block — Item.cs:7873-7880")
    lines.append("        let r544 = deltas")
    lines.append("            .iter()")
    lines.append("            .find(|r| r.id == 544 && r.variant == ItemVariant::DisabledBossSummon)")
    lines.append("            .unwrap();")
    lines.append("        assert!(r544.set.contains(&VOp::Set(VField::UseStyle, VVal::Int(0))));")
    lines.append("        assert!(r544.set.contains(&VOp::Set(VField::Consumable, VVal::Bool(false))));")
    lines.append("        // the base row keeps the use block (the branch applies without the variant)")
    lines.append("        assert_eq!(ITEMS[544].use_style, 4);")
    lines.append("        assert!(ITEMS[544].consumable);")
    lines.append("    }")
    lines.append("")
    lines.append("    // DW-1.2 — names come from the ItemID consts.")
    lines.append("    #[test]")
    lines.append("    fn test_DW_1_2_names() {")
    lines.append("        assert_eq!(NAMES.len(), %d);" % ITEM_COUNT)
    lines.append("        assert_eq!(NAMES[0], \"None\");")
    lines.append("        assert_eq!(NAMES[2], \"DirtBlock\");")
    lines.append("        assert_eq!(NAMES[6195], \"OldStyleParkourBookInactive\");")
    lines.append("        assert_eq!(ITEMS[2].name, \"Dirt Block\");")
    lines.append("    }")
    lines.append("}")
    return "\n".join(lines)


def emit(rows, names, net_defaults, on_created, registrations, variant_rows,
         food_ids, deprecated_ids, bait_rare3, multi_flags):
    parts = []
    parts.append("// @generated by tools/GenItemData from the decompiled Terraria sources.")
    parts.append("// Do not edit by hand.")
    parts.append("")
    parts.append("//! Per-id item defaults — the effective result of `Item.SetDefaults(type)`")
    parts.append("//! on a fresh item (`Item.cs:48663-48926`): `ResetStats` stock")
    parts.append("//! (`:48944-49051`), the `SetDefaults1-5` / `SetFoodDefaults` case bodies,")
    parts.append("//! the `DefaultTo*` helpers and the dispatcher epilogue (`:48713-48925`),")
    parts.append("//! plus `netDefaults` (`:1211-1367`), `OnCreated` (`:48936-48942`) and the")
    parts.append("//! `ItemVariants` tables (`ItemVariants.cs:109-146`). Regenerate with")
    parts.append("//! `python tools/GenItemData/gen.py` — a second run must produce an empty")
    parts.append("//! diff. Never hand-edit.")
    parts.append("")
    parts.append("use crate::registry::DamageClass;")
    parts.append("")
    parts.append(STRUCT_TEMPLATE)
    parts.append("")
    parts.append("/// `ITEMS[id]` — the effective defaults for every vanilla item id")
    parts.append("/// 0..%d (`ItemID.Count = 6196`, `ItemID.cs:14861`)." % (ITEM_COUNT - 1))
    parts.append("pub static ITEMS: [ItemDef; %d] = [" % ITEM_COUNT)
    parts.extend(emit_rows(rows, names, multi_flags))
    parts.append("];")
    parts.append("")
    parts.append(emit_names(names))
    parts.append("")
    parts.append(emit_net_defaults(net_defaults))
    parts.append("")
    parts.append(emit_on_created(on_created))
    parts.append("")
    parts.append(emit_variant_tables(registrations, variant_rows))
    parts.append("")
    parts.append(emit_test_module(SPOT_CHECKS, food_ids, deprecated_ids,
                                  bait_rare3, variant_rows))
    parts.append("")
    return "\n".join(parts)


def run_pipeline(out_path):
    """Full parse+build+emit; returns the parsed rows (test entry point)."""
    global OUT
    saved = OUT
    OUT = out_path
    try:
        return _main_inner()
    finally:
        OUT = saved


def main():
    _main_inner()


def _main_inner():
    item_lines = read_lines(ITEM_CS)
    itemid_text = "\n".join(read_lines(ITEMID_CS))
    count = re.search(r"\bItemID\.Count = (\d+);", itemid_text)
    if not count or int(count.group(1)) != ITEM_COUNT:
        fail("ItemID.cs", 14861, "ItemID.Count mismatch")
    ids = {
        "ItemID": parse_id_consts(ITEMID_CS, "ItemID"),
        "AmmoID": parse_id_consts(AMMOID_CS, "AmmoID"),
        "ProjectileID": parse_id_consts(PROJECTILEID_CS, "ProjectileID"),
        "BuffID": parse_id_consts(BUFFID_CS, "BuffID"),
        "TileID": parse_id_consts(TILEID_CS, "TileID"),
        "WallID": parse_id_consts(WALLID_CS, "WallID"),
        "ItemRarityColor": parse_rarity_enum(),
    }
    prices = parse_named_prices(item_lines)
    is_food = parse_bool_set(itemid_text, "Sets.IsFood", "ItemID.cs")
    is_drill = parse_bool_set(itemid_text, "Sets.IsDrill", "ItemID.cs")
    is_chainsaw = parse_bool_set(itemid_text, "Sets.IsChainsaw", "ItemID.cs")
    deprecated = parse_bool_set(itemid_text, "Sets.Deprecated", "ItemID.cs")
    tables = {
        "projhook": parse_projhook_set(),
        "is_food": is_food,
        "is_drill": is_drill,
        "is_chainsaw": is_chainsaw,
        "deprecated": deprecated,
    }
    parsed = {}
    for name, lo, hi in METHOD_SPANS:
        # spans are 1-based header..closing-brace; body = the `{` line (0-based lo)
        # through the line before the closing `}` (0-based hi - 1)
        parsed[name] = parse_stmts(item_lines, lo, hi - 1, "Item.cs")
    parsed["epilogue"] = parse_stmts(item_lines, 48712, 48925, "Item.cs")
    registrations = parse_variant_registrations()
    by_value = {}
    for const_name, value in ids["ItemID"].items():
        if 0 <= value < ITEM_COUNT:
            by_value.setdefault(value, const_name)
    if len(by_value) != ITEM_COUNT:
        fail("ItemID.cs", 1, "expected %d in-range const names, got %d"
             % (ITEM_COUNT, len(by_value)))
    names = [by_value[i] for i in range(ITEM_COUNT)]
    rows, all_variants, deviations, material_written = build_rows(
        parsed, tables, ids, prices, is_food, names)
    material_set = compute_material(rows, itemid_text, ids)
    for i in range(ITEM_COUNT):
        if not material_written[i]:
            rows[i]["material"] = i in material_set
    # restore rows' material sentinel: the pre-branch value is IsAMaterial[id]
    for vr in all_variants:
        ops = vr[2]
        for j, (op, field, value) in enumerate(ops):
            if value == "__MATERIAL__":
                ops[j] = (op, field, vr[0] in material_set)
    for (item_id, line, what) in deviations:
        print("DEVIATION: Item.cs:%d id %d: %s (base value kept)" % (line, item_id, what))

    multi_flags = []
    for item_id, row in enumerate(rows):
        derive_class(row, item_id, multi_flags)  # range/range checks happen in emit
    if multi_flags:
        fail("Item.cs", 1, "items with multiple damage-class flags need a "
             "derivation order decision: %s" % multi_flags[:10])
    for (item_id, _checks) in SPOT_CHECKS:
        SPOT_NAMES[item_id] = names[item_id]
    # sanity: the bait ladder really landed (bait >= 50 → rare 3)
    bait_rare3 = [i for i in range(ITEM_COUNT) if rows[i]["bait"] >= 50]
    for i in bait_rare3:
        if rows[i]["rare"] != 3:
            fail("Item.cs", 48840, "bait ladder did not set rare 3 for id %d" % i)
    if not bait_rare3:
        fail("Item.cs", 48840, "no bait >= 50 rows — ladder broken?")

    text = emit(rows, names, parse_net_defaults(item_lines),
                parse_on_created(item_lines), registrations, all_variants,
                is_food, [d for d in deprecated if 0 <= d < ITEM_COUNT],
                bait_rare3, multi_flags)
    _last_variants = all_variants
    OUT.write_text(text, encoding="utf-8")
    print("wrote %s (%d bytes, %d rows, %d variant rows, %d material ids)"
          % (OUT, len(text), ITEM_COUNT, len(all_variants), len(material_set)))
    return rows, all_variants


if __name__ == "__main__":
    main()


