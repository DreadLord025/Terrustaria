#!/usr/bin/env python3
"""W4-d36 witness: the NpcKind protocol-id bijection + the exhaustive-match
site walk, scripted over the working tree (code-only mode — no cargo).

Checks (per research/109 §7 + the 108-E E5 checklist):
 1. `to_u16` / `from_u16` arm counts agree, ids bijective, 0 dups, i16-safe.
 2. The foreign stash's generator bug (`NpcKind::X => NpcKind::X` inside
    `from_u16`) is absent — every from arm is `<int> => NpcKind::<name>`.
 3. The enum variant set == the to-arm set == the from-arm set == ALL_KINDS.
 4. ALL_KINDS decl length == its row count; s1_f_3 pin == the same count.
 5. Every hard exhaustive-match site carries an arm for every new d36 kind
    (or an explicit fold), and the wildcard sites' new rows parse.
Run: python tools/_d36_bijection.py   (exit 0 = all pins hold)
"""
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SRC = ROOT / "crates/tr-app/src"


def read(p):
    return (SRC / p).read_text(encoding="utf-8")


def fn_body(src, sig):
    """The brace-balanced body of the first fn whose line contains sig."""
    i = src.index(sig)
    j = src.index("{", i)
    depth, k = 0, j
    while True:
        c = src[k]
        if c == "{":
            depth += 1
        elif c == "}":
            depth -= 1
            if depth == 0:
                return src[j : k + 1]
        k += 1


def arms(body):
    """(lhs, rhs) of every `lhs => rhs,` arm, comments stripped."""
    body = re.sub(r"//[^\n]*", "", body)
    out = []
    for m in re.finditer(r"([^=>{},]+?)=>\s*([^,{}]+?)(?:,|\n\s*\})", body):
        lhs = " ".join(m.group(1).split())
        rhs = " ".join(m.group(2).split())
        out.append((lhs, rhs))
    return out


def kinds_in(pat):
    return set(re.findall(r"NpcKind::(\w+)", pat))


fail = []


def check(name, ok, detail=""):
    print(("PASS " if ok else "FAIL ") + name + ("  " + detail if detail else ""))
    if not ok:
        fail.append(name)


data = read("npc/data.rs")
npc = read("npc.rs")

# --- 1/2/3: the protocol pair -------------------------------------------
to_body = fn_body(data, "pub fn to_u16(")
from_body = fn_body(data, "pub fn from_u16(")
to_arms = {}
for lhs, rhs in arms(to_body):
    ks = kinds_in(lhs)
    if not ks or not rhs.isdigit():
        continue
    for k in ks:
        if k in to_arms:
            fail.append(f"to_u16 dup kind {k}")
        to_arms[k] = int(rhs)
from_arms = {}
for lhs, rhs in arms(from_body):
    ks = kinds_in(rhs)
    if not lhs.isdigit() or len(ks) != 1:
        if "NpcKind::" in lhs:
            fail.append(f"from_u16 generator-bug arm: {lhs} => {rhs}")
        continue
    i = int(lhs)
    if i in from_arms:
        fail.append(f"from_u16 dup id {i}")
    from_arms[i] = next(iter(ks))
check("from_u16 has no NpcKind=>NpcKind arms", not any("generator-bug" in f for f in fail))
check("to/from arm counts equal", len(to_arms) == len(from_arms), f"{len(to_arms)}/{len(from_arms)}")
inv = {v: k for k, v in to_arms.items()}
check("to ids unique", len(inv) == len(to_arms))
check("bijection to->from", all(from_arms.get(i) == k for k, i in to_arms.items()))
check("bijection from->to", all(to_arms.get(k) == i for i, k in from_arms.items()))
check("i16-safe", max(to_arms.values()) <= 32767, f"max {max(to_arms.values())}")

# --- enum / ALL_KINDS ----------------------------------------------------
enum_body = fn_body(npc, "pub enum NpcKind")
enum_names = set(re.findall(r"^\s{4}(\w+),\s*$", enum_body, re.M))
check("enum == to-arm kinds", enum_names == set(to_arms), f"enum {len(enum_names)}")
m = re.search(r"const ALL_KINDS: \[NpcKind; (\d+)\] = \[(.*?)\];", npc, re.S)
decl = int(m.group(1))
rows = re.findall(r"NpcKind::(\w+)", m.group(2))
check("ALL_KINDS decl == rows == roster", decl == len(rows) == len(to_arms), f"{decl}/{len(rows)}/{len(to_arms)}")
check("ALL_KINDS set == roster", set(rows) == set(to_arms))
m = re.search(r"covered \+ sentinel,\s*(\d+),", npc)
check("s1_f_3 pin == roster", m and int(m.group(1)) == len(to_arms), m.group(1) if m else "missing")

# --- 5: hard sites carry every d36 kind ----------------------------------
D36 = [
    "ToxicSludge", "Penguin", "PenguinBlack", "GiantTortoise", "IceTortoise",
    "Wolf", "Vampire", "Frankenstein", "BlackRecluse", "SwampThing",
    "UndeadViking", "CorruptPenguin", "PigronCorruption", "PigronHallow",
    "Crimera", "MossHornet", "PigronCrimson", "FaceMonster", "SnowFlinx",
    "ArmoredViking", "Lihzahrd", "LihzahrdCrawler", "HeadacheSkeleton",
    "MisassembledSkeleton", "PantlessSkeleton", "SpikedJungleSlime", "Moth",
    "IcyMerman",
]
present = [k for k in D36 if k in enum_names]
print(f"d36 kinds present: {len(present)}/28 -> {present}")
hard = [
    ("npc/data.rs", "fn ai_style("), ("npc/data.rs", "fn size("),
    ("npc/data.rs", "fn max_hp("), ("npc/data.rs", "fn contact_damage("),
    ("npc.rs", "fn defense("), ("npc.rs", "fn value("),
    ("npc.rs", "fn update_kind_ai("), ("npc.rs", "pub fn render_key(&self)"),
    ("npc_buffs.rs", "fn debuff_immune_ids("),
    ("app/netglue.rs", "fn npc_seed("),
    ("ui/bestiary.rs", "fn portrait_data("),
    ("render/entities.rs", "pub(crate) fn draw_npcs("),
]
for path, sig in hard:
    src = read(path)
    try:
        body = fn_body(src, sig)
    except ValueError:
        check(f"{path}::{sig} found", False)
        continue
    have = kinds_in(body)
    missing = [k for k in present if k not in have]
    check(f"{path}::{sig}", not missing, f"missing {missing}" if missing else f"{len(present)} kinds")

# bestiary: every kind resolves through ENTRIES (own row or fold)
best = read("bestiary.rs")
entry_kinds = kinds_in(best[best.index("pub const ENTRIES"):best.index("pub fn unlock_state")])
fold_body = fn_body(best, "pub fn entry_for_npc(")
fold_src = kinds_in(fold_body[: fold_body.index("_ =>") if "_ =>" in fold_body else len(fold_body)])
unresolved = [k for k in present if k not in entry_kinds and k not in fold_src]
check("bestiary entry/fold coverage", not unresolved, f"missing {unresolved}" if unresolved else "")

print()
if fail:
    print(f"{len(fail)} FAILURES: {fail}")
    sys.exit(1)
print(f"ALL PINS HOLD — roster {len(to_arms)}, d36 {len(present)}/28, max id {max(to_arms.values())}")
