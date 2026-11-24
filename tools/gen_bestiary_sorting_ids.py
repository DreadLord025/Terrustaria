#!/usr/bin/env python3
"""Generate crates/tr-app/src/bestiary_sorting_ids.rs from the decompiled
Terraria sources (Terraria 1.4.5.8) — W4-e23 remainder 5.

`ContentSamples.NpcBestiarySortingId` numbers the FULL vanilla NPC set, not
the port's narrowed roster. This generator replays the C# pipeline exactly:

  * `ContentSamples.Initialize` (:857-873): one `NPC.SetDefaults(i)` sample
    per net id -65..NPCID.Count-1 (the fresh `Main`, Classic difficulty, no
    world flags — `Main.Difficulty` reads Classic with no ActiveWorldFileData,
    `Main.cs:2702-2724`). The stat fields are interpreted from the
    `SetDefaults` type arms (`NPC.cs:8426-17858`) + the negative-id
    `SetDefaultsFromNetId` switch (`:7653-7843`); `ScaleStats` is the identity
    at Classic (every multiplier samples 1, the <6-HP bump only fires inside
    the scaling gate).
  * `BestiaryHelper.GetSortedBestiaryEntriesList` (:765-773): the nine-key
    stable orderby — town priority, !isLikeATownNPC, gold-critter priority,
    !CountsAsACritter, boss priority, lowest biome group index, aiStyle,
    power level (damage+defense+lifeMax/4), rarity stars — then RemoveAll of
    the `NPCBestiaryDrawOffset.Hide` rows (:771, :788-795).
  * `CreateBestiarySortingIds` (:1083-1092): number the survivors from 1.

The tag/biome half reuses the `gen_bestiary_tags.py` replay (the common-filter
order + the per-entry AddRange/Remove walk); the priority lists come from
`NPCID.Sets.*BestiaryPriority`; the stars from `FillNpcRarities`' formula +
override table (`ContentSamples.cs:1094-1306`).

Run from the repo root:  python tools/gen_bestiary_sorting_ids.py [--check]
The output is a cited static table; never hand-edit it.
"""
import re
import struct
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
DEC = ROOT / "decompiled" / "Terraria"
NPC = (DEC / "NPC.cs").read_text(encoding="utf-8")
NPCID = (DEC / "ID" / "NPCID.cs").read_text(encoding="utf-8")
POP = (DEC / "GameContent" / "Bestiary" / "BestiaryDatabaseNPCsPopulator.cs").read_text(encoding="utf-8")
CS = (DEC / "ID" / "ContentSamples.cs").read_text(encoding="utf-8")
OUT = ROOT / "crates" / "tr-app" / "src" / "bestiary_sorting_ids.rs"

NPC_COUNT = 697  # NPCID.cs:12581 `NPCID.Count = 697`
INT_MAX = 2**31 - 1


def f32(x):
    """Round a Python float to IEEE-754 single precision (the C# `float`)."""
    return struct.unpack("f", struct.pack("f", x))[0]


def match_brace(s, i):
    d = 0
    while True:
        if s[i] == "{":
            d += 1
        elif s[i] == "}":
            d -= 1
            if d == 0:
                return i
        i += 1


def match_paren(s, i):
    d = 0
    while True:
        if s[i] == "(":
            d += 1
        elif s[i] == ")":
            d -= 1
            if d == 0:
                return i
        i += 1


def body_of(sig, src):
    m = re.search(sig, src, re.S)
    assert m, sig
    j = src.index("{", m.start())
    return src[j + 1 : match_brace(src, j)]


# --- the SetDefaults type arms -------------------------------------------------
SD = body_of(r"public void SetDefaults\(int Type.*?\)\s*\{", NPC)


def cond_ids(cond):
    ids = set(int(x) for x in re.findall(r"==\s*(-?\d+)", cond))
    for a in re.findall(r">=\s*(-?\d+)", cond):
        for b in re.findall(r"<=\s*(-?\d+)", cond):
            if int(a) <= int(b):
                ids.update(range(int(a), int(b) + 1))
    return ids


arm_by_id = {}
i = 0
pat = re.compile(r"(?:else )?if\s*\(")
while True:
    t = pat.search(SD, i)
    if not t:
        break
    p = SD.index("(", t.start())
    q = match_paren(SD, p)
    cond = SD[p + 1 : q]
    bs = SD.index("{", q)
    e = match_brace(SD, bs)
    # Only the `this.type` arms set stats; the world-flag / spawnparams arms
    # are false in the fresh sample.
    if "this.type" in cond and "Main." not in cond and "spawnparams" not in cond:
        for tid in cond_ids(cond):
            arm_by_id.setdefault(tid, SD[bs + 1 : e])
    i = e + 1

FIELDS = {
    "damage", "defense", "lifeMax", "aiStyle", "townNPC", "boss", "rarity",
    "knockBackResist", "value", "friendly", "life", "scale",
}
# The class defaults + the SetDefaults pre-arm resets (NPC.cs:8437-8545).
DEFAULTS = {
    "damage": 0, "defense": 0, "lifeMax": 0, "aiStyle": 0, "townNPC": False,
    "boss": False, "rarity": 0, "knockBackResist": f32(1.0), "value": f32(0.0),
    "friendly": False, "life": 0, "scale": f32(1.0),
}


def buy_price(expr):
    m = re.fullmatch(r"Item\.buyPrice\(([^)]*)\)", expr)
    args = [int(a.strip()) for a in m.group(1).split(",")]
    while len(args) < 4:
        args.append(0)
    p, g, s, c = args
    return c + s * 100 + g * 10000 + p * 1000000  # Item.cs:49417-49420


def eval_rhs(expr, st):
    expr = expr.strip()
    if expr == "true":
        return True
    if expr == "false":
        return False
    if re.fullmatch(r"-?\d+", expr):
        return int(expr)
    if re.fullmatch(r"-?[\d.]+f", expr):
        return f32(float(expr[:-1]))
    if re.fullmatch(r"-?[\d.]+", expr):
        return f32(float(expr))
    if expr.startswith("Item.buyPrice"):
        return buy_price(expr)
    # The negative-id scale-stat writes (SetDefaultsFromNetId):
    m = re.fullmatch(r"\(int\)\(\(float\)this\.(\w+) \* this\.scale\)", expr)
    if m:
        return int(f32(st.get(m.group(1), 0) * st["scale"]))
    m = re.fullmatch(r"\(int\)\(this\.(\w+) \* this\.scale\)", expr)
    if m:
        return int(f32(st.get(m.group(1), 0) * st["scale"]))
    m = re.fullmatch(r"\(int\)\(\(double\)\(\(float\)this\.(\w+) \* this\.scale\) \* ([\d.]+)\)", expr)
    if m:
        return int(f32(f32(st.get(m.group(1), 0) * st["scale"]) * float(m.group(2))))
    m = re.fullmatch(r"\(int\)\(\(double\)\(this\.(\w+) \* this\.scale\) \* ([\d.]+)\)", expr)
    if m:
        return int(f32(f32(st.get(m.group(1), 0) * st["scale"]) * float(m.group(2))))
    m = re.fullmatch(r"([\d.]+)f - this\.scale", expr)
    if m:
        return f32(float(m.group(1)) - st["scale"])
    raise ValueError("unhandled RHS: " + expr)


def apply_stmt(stmt, st):
    m = re.match(r"this\.(\w+)\s*(=|\+=|-=|\*=)\s*(.*);$", stmt.strip(), re.S)
    if not m:
        return
    f, op, expr = m.group(1), m.group(2), m.group(3).strip()
    if f not in FIELDS:
        return
    val = eval_rhs(expr, st)
    if op == "=":
        st[f] = val
    elif op == "+=":
        st[f] = st.get(f, 0) + val
    elif op == "-=":
        st[f] = st.get(f, 0) - val
    elif op == "*=":
        cur = st.get(f, 0)
        st[f] = f32(cur * val) if isinstance(val, float) or isinstance(cur, float) else cur * val


def eval_guard(cond, tid):
    cond = cond.strip()
    # Every world flag / progression gate is false in the fresh Classic sample.
    if "Main." in cond or "NPC.downed" in cond or "spawnparams" in cond or "IsABestiaryIconDummy" in cond:
        return False

    def atom(a):
        a = a.strip()
        for op, fn in [
            ("==", lambda x, y: x == y), ("!=", lambda x, y: x != y),
            (">=", lambda x, y: x >= y), ("<=", lambda x, y: x <= y),
            (">", lambda x, y: x > y), ("<", lambda x, y: x < y),
        ]:
            m = re.fullmatch(r"this\.type\s*" + re.escape(op) + r"\s*(-?\d+)", a)
            if m:
                return fn(tid, int(m.group(1)))
        raise ValueError("guard atom: " + a)

    for or_part in cond.split("||"):
        if all(atom(a) for a in or_part.split("&&")):
            return True
    return False


def walk_block(blk, st, tid):
    i = 0
    while i < len(blk):
        ch = blk[i]
        if ch in " \t\r\n":
            i += 1
            continue
        mm = re.match(r"(?:else )?if\s*\(", blk[i:])
        if mm:
            p = i + mm.end() - 1
            q = match_paren(blk, p)
            cond = blk[p + 1 : q]
            bs = blk.index("{", q)
            e = match_brace(blk, bs)
            taken = eval_guard(cond, tid)
            if taken:
                walk_block(blk[bs + 1 : e], st, tid)
            rest = blk[e + 1 :]
            em = re.match(r"\s*else\s*\{", rest)
            if em:
                bs2 = e + 1 + em.end() - 1
                e2 = match_brace(blk, bs2)
                if not taken:
                    walk_block(blk[bs2 + 1 : e2], st, tid)
                i = e2 + 1
            else:
                i = e + 1
            continue
        mm = re.match(r"switch\s*\(", blk[i:])
        if mm:
            p = i + mm.end() - 1
            q = match_paren(blk, p)
            bs = blk.index("{", q)
            e = match_brace(blk, bs)
            parts = re.split(r"case (\d+):", blk[bs + 1 : e])
            for ci in range(1, len(parts), 2):
                if int(parts[ci]) == tid:
                    walk_block(parts[ci + 1].split("break;")[0], st, tid)
            i = e + 1
            continue
        sc = blk.find(";", i)
        if sc < 0:
            break
        apply_stmt(blk[i : sc + 1], st)
        i = sc + 1


def set_defaults_positive(tid):
    st = dict(DEFAULTS)
    blk = arm_by_id.get(tid)
    if blk:
        walk_block(blk, st, tid)
    # ScaleStats is the identity at Classic; the tail `this.life = this.lifeMax`
    # (NPC.cs:17857) keeps life synced.
    st["life"] = st["lifeMax"]
    return st


# --- the negative-id switch ----------------------------------------------------
NEG = body_of(r"private void SetDefaultsFromNetId\(int id.*?\)\s*\{", NPC)
# NPCID.NetIdMap (NPCID.cs:12095-12104): FromNetId(-n) = NetIdMap[n-1].
NETIDMAP = [
    81, 81, 1, 1, 1, 1, 1, 1, 1, 1, 6, 6, 31, 31, 77, 42, 42, 176, 176, 176,
    176, 173, 173, 183, 183, 3, 3, 132, 132, 186, 186, 187, 187, 188, 188, 189,
    189, 190, 191, 192, 193, 194, 2, 200, 200, 21, 21, 201, 201, 202, 202, 203,
    203, 223, 223, 231, 231, 232, 232, 233, 233, 234, 234, 235, 235,
]
neg_cases = {}
parts = re.split(r"case (-?\d+):", NEG)
for ci in range(1, len(parts), 2):
    neg_cases[int(parts[ci])] = parts[ci + 1].split("break;")[0]


def set_defaults_negative(nid):
    base = NETIDMAP[-nid - 1]
    st = set_defaults_positive(base)
    blk = neg_cases.get(nid, "")
    # SetDefaults_ForNetId(num, spawnparams, X) sets scale = X (the size
    # override, NPC.cs:17792-17807); the world-flag scale bumps are false.
    sm = re.search(r"SetDefaults_ForNetId\(num, spawnparams, ([\d.]+f)\)", blk)
    if sm:
        st["scale"] = f32(float(sm.group(1)[:-1]))
    walk_block(blk, st, base)
    # The tail `this.lifeMax = this.life` (SetDefaultsFromNetId).
    st["lifeMax"] = st.get("life", st["lifeMax"])
    return st


# --- the priority lists --------------------------------------------------------
def prio_list(name):
    m = re.search(r"Sets\." + name + r" = new List<int>\s*\{(.*?)\};", NPCID, re.S)
    assert m, name
    return [int(x) for x in re.findall(r"-?\d+", m.group(1))]


TOWN = prio_list("TownNPCBestiaryPriority")
GOLD = prio_list("NormalGoldCritterBestiaryPriority")
BOSS = prio_list("BossBestiaryPriority")


# --- the tag/biome half (the gen_bestiary_tags replay) -------------------------
m = re.search(
    r"GetCommonInfoElementsForFilters\(\)\s*\{.*?return new List<IBestiaryInfoElement>\s*\{(.*?)\};",
    POP, re.S,
)
filter_order = [f"{a}.{b}" for a, b in re.findall(r"SpawnConditions\.(\w+)\.(\w+)", m.group(1))]


def lineno(pos):
    return POP.count("\n", 0, pos) + 1


per_entry = {}
events = []
pat_add = re.compile(
    r"this\.FindEntryByNPCID\((-?\d+)\)\.Info\.AddRange\(new IBestiaryInfoElement\[\d+\]\s*\{(.*?)\}\)",
    re.S,
)
pat_remove = re.compile(
    r"this\.FindEntryByNPCID\((-?\d+)\)\.Info\.Remove\(CommonTags\.SpawnConditions\.(\w+)\.(\w+)\)"
)
for mm in pat_add.finditer(POP):
    ln = lineno(mm.start())
    if not (888 <= ln <= 2027):
        continue
    net = int(mm.group(1))
    for a, b in re.findall(r"CommonTags\.SpawnConditions\.(\w+)\.(\w+)", mm.group(2)):
        events.append((0, ln, net, "add", f"{a}.{b}"))
for mm in pat_remove.finditer(POP):
    ln = lineno(mm.start())
    if not (888 <= ln <= 2027 or 601 <= ln <= 748):
        continue
    phase = 1 if 601 <= ln <= 748 else 0
    events.append((phase, ln, int(mm.group(1)), "remove", f"{mm.group(2)}.{mm.group(3)}"))
events.sort(key=lambda e: (e[0], e[1]))
for _, ln, net, op, tag in events:
    lst = per_entry.setdefault(net, [])
    if op == "add":
        lst.append(tag)
    elif tag in lst:
        lst.remove(tag)  # List.Remove: first occurrence


def lowest_biome(net):
    tags = per_entry.get(net, [])
    for i in range(len(filter_order) - 1, -1, -1):
        if filter_order[i] in tags:
            return i
    return INT_MAX


# --- the Hide set --------------------------------------------------------------
hide = set()
for mm in re.finditer(
    r"\{\s*(-?\d+),\s*new NPCBestiaryDrawModifiers\(0\)\s*(\{[^{}]*\})?\s*\}", NPCID, re.S
):
    if mm.group(2) and "Hide = true" in mm.group(2):
        hide.add(int(mm.group(1)))


# --- the rarity stars ----------------------------------------------------------
overrides = {}
for mm in re.finditer(r"ContentSamples\.NpcBestiaryRarityStars\[(-?\d+)\] = (\d+);", CS):
    overrides[int(mm.group(1))] = int(mm.group(2))


def stars_formula(st):
    rarity, boss = st["rarity"], st["boss"]
    num = f32(1.0 + rarity)
    if rarity == 1:
        num = f32(num + 1)
    elif rarity == 2:
        num = f32(num + 1.5)
    elif rarity == 3:
        num = f32(num + 2)
    elif rarity == 4:
        num = f32(num + 2.5)
    elif rarity == 5:
        num = f32(num + 3)
    elif rarity > 0:
        num = f32(num + 3.5)
    if boss:
        num = f32(num + 0.5)
    num2 = st["damage"] + st["defense"] + st["lifeMax"] // 4
    if num2 > 10000:
        num = f32(num + 3.5)
    elif num2 > 5000:
        num = f32(num + 3)
    elif num2 > 1000:
        num = f32(num + 2.5)
    elif num2 > 500:
        num = f32(num + 2)
    elif num2 > 150:
        num = f32(num + 1.5)
    elif num2 > 50:
        num = f32(num + 1)
    if num > 5:
        num = f32(5.0)
    return int(num)


def stars(net, st):
    return overrides.get(net, stars_formula(st))


# --- the nine-key sort + numbering ---------------------------------------------
def town_prio(net):
    return TOWN.index(net) if net in TOWN else INT_MAX


def gold_prio(net):
    return GOLD.index(net) if net in GOLD else INT_MAX


def boss_prio(net):
    return BOSS.index(net) if net in BOSS else -1


def is_critter(st, net):
    return st["lifeMax"] <= 5 and st["damage"] == 0 and net != 594 and net != 686


rows = []
for net in range(-65, NPC_COUNT):
    st = set_defaults_negative(net) if net < 0 else set_defaults_positive(net)
    if net in hide:
        continue  # RemoveAll(ShouldHideBestiaryEntry) (:771)
    key = (
        town_prio(net),
        not st["townNPC"],
        gold_prio(net),
        not is_critter(st, net),
        boss_prio(net),
        lowest_biome(net),
        st["aiStyle"],
        st["damage"] + st["defense"] + st["lifeMax"] // 4,
        stars(net, st),
    )
    rows.append((key, net))
# LINQ orderby is a stable sort; the net-id ascending iteration breaks ties.
rows.sort(key=lambda r: r[0])
ids = {net: i + 1 for i, (k, net) in enumerate(rows)}


# --- emit ----------------------------------------------------------------------
out = []
w = out.append
w("//! GENERATED by tools/gen_bestiary_sorting_ids.py — do not hand-edit.")
w("//!")
w("//! `ContentSamples.NpcBestiarySortingId` over the FULL vanilla NPC set")
w("//! (Terraria 1.4.5.8): the nine-key stable orderby of")
w("//! `BestiaryHelper.GetSortedBestiaryEntriesList` (`ContentSamples.cs:765-773`)")
w("//! with the `NPCBestiaryDrawOffset.Hide` rows removed (`:771`), numbered")
w("//! from 1 by `CreateBestiarySortingIds` (`:1083-1092`). The corner number")
w("//! (`NPCNetIdBestiaryInfoElement.cs:10`) and the `ByBestiarySortingId` step")
w("//! both read this table — the absolute display numbers match the vanilla,")
w("//! not the port's narrowed roster (W4-e23 remainder 5).")
w("//!")
w("//! The stats are the fresh Classic `SetDefaults` sample (`ContentSamples.Initialize`,")
w("//! `:857-873`); the biome index reuses the `gen_bestiary_tags.py` tag replay.")
w("")
w(f"/// The full-vanilla sorting-id table, net-id ascending ({len(ids)} entries).")
w("pub static SORTING_IDS: &[(i32, i32)] = &[")
for net in sorted(ids):
    w(f"    ({net}, {ids[net]}),")
w("];")
w("")
w("/// `ContentSamples.NpcBestiarySortingId[netId]` — the entry's display index")
w("/// (`NPCNetIdBestiaryInfoElement.cs:10`). An id outside the table reads 0")
w("/// (the C# dictionary would throw; the port's callers only pass roster ids).")
w("pub fn sorting_id(net_id: i32) -> i32 {")
w("    SORTING_IDS")
w("        .binary_search_by_key(&net_id, |&(id, _)| id)")
w("        .map(|i| SORTING_IDS[i].1)")
w("        .unwrap_or(0)")
w("}")
w("")

rendered = "\n".join(out) + "\n"
if "--check" in sys.argv:
    old = OUT.read_text(encoding="utf-8") if OUT.exists() else ""
    if old.replace("\r\n", "\n") != rendered:
        sys.exit(f"{OUT} drifted — regenerate with tools/gen_bestiary_sorting_ids.py")
    print("bestiary_sorting_ids.rs is up to date")
else:
    OUT.write_text(rendered, encoding="utf-8", newline="\n")
    print(f"wrote {OUT}: {len(ids)} sorting ids (town-first: 22={ids[22]}, 17={ids[17]}, 18={ids[18]})")
