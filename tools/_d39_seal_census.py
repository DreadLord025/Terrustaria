# W4-d39 SEAL phase: re-derive every count pin at the CURRENT worktree tip
# (a0d49dc1d + the seal's comment corrections), and adjudicate the f18
# overlap (the 20-kind 355-430 lane) against this lane's 7 kinds. Scratch
# tool, committed as the seal's witness. File reads + git-show only;
# direct paths, no junction recursion; NO cargo/rustc/tests invoked.
# Run: python tools/_d39_seal_census.py   (from the repo root)
import re, subprocess, json, collections

REPO = r"C:\Users\DreadLord\Desktop\TR-W4-d39"
F18 = "origin/task/W4-f18-npc-kinds-verify-2"

def read(path):
    return open(REPO + "\\" + path.replace("/", "\\"),
                encoding="utf-8", errors="replace").read()

def show(rev, path):
    r = subprocess.run(["git", "-C", REPO, "show", f"{rev}:{path}"],
                       capture_output=True)
    assert r.returncode == 0, f"git show {rev}:{path} failed"
    return r.stdout.decode("utf-8", "replace")

def fn_body(src, sig):
    i = src.index(sig)
    j = src.index("{", i)
    depth, k = 0, j
    while True:
        c = src[k]
        if c == "{": depth += 1
        elif c == "}":
            depth -= 1
            if depth == 0: return src[j:k+1]
        k += 1

def arms_to(src):
    body = fn_body(src, "pub fn to_u16")
    return {m[0]: int(m[1]) for m in
            re.findall(r"NpcKind::(\w+)\s*=>\s*(\d+)", body)}

def arms_from(src):
    body = fn_body(src, "pub fn from_u16")
    return {int(m[0]): m[1] for m in
            re.findall(r"(\d+)\s*=>\s*NpcKind::(\w+)", body)}

npc = read("crates/tr-app/src/npc.rs")
data = read("crates/tr-app/src/npc/data.rs")
best = read("crates/tr-app/src/bestiary.rs")
assets = read("crates/tr-app/src/assets.rs")
out = {}

# --- pin 1: enum variants == ALL_KINDS rows == to_u16 == from_u16, bijection
enum_src = npc[npc.index("pub enum NpcKind"):]
enum_src = enum_src[:enum_src.index("\n}")]
variants = re.findall(r"^\s{4}(\w+),\s*$", enum_src, re.M)
ak = npc[npc.index("const ALL_KINDS:"):]
ak = ak[:ak.index("];")]
ak_rows = re.findall(r"NpcKind::(\w+),", ak)
decl = int(re.search(r"const ALL_KINDS: \[NpcKind; (\d+)\]", npc).group(1))
to = arms_to(data); fr = arms_from(data)
out["enum_variants"] = len(variants)
out["all_kinds_decl"] = decl
out["all_kinds_rows"] = len(ak_rows)
out["to_u16_arms"] = len(to)
out["from_u16_arms"] = len(fr)
out["all_kinds_dups"] = len(ak_rows) - len(set(ak_rows))
out["enum_set_eq_allkinds_set"] = set(variants) == set(ak_rows)
out["to_from_bijection"] = all(fr.get(v) == k for k, v in to.items()) \
    and len(to) == len(fr)
out["arm_set_eq_allkinds_set"] = set(to) == set(ak_rows)
out["max_proto"] = max(to.values())
out["i16_safe"] = max(to.values()) <= 32767
out["d39_band"] = {k: to[k] for k in
    ["Butterfly", "ScorpionBlack", "Scorpion", "Grasshopper",
     "ChatteringTeethBomb", "CultistArcherBlue", "CultistArcherWhite"]}

# --- pin 2: s1_f_3 literal + the dead-id absence
m = re.search(r"covered \+ sentinel,\s*(\d+),", npc)
out["s1_f_3_literal"] = int(m.group(1))
out["dead_ids_absent"] = not any(
    v in variants for v in
    ["StardustWormBody", "StardustWormTail", "StardustJellyfishSmall"])

# --- pin 3: bestiary ENTRIES rows, unique credits, positive-net distinct
eb = best[best.index("pub const ENTRIES"):]
eb = eb[:eb.index("\n];")]
credits = re.findall(r'credit_id: "(\w+)"', eb)
nets = [int(x) for x in re.findall(r"net_id: (-?\d+)", eb)]
out["entries_rows"] = len(credits)
out["entries_credit_dups"] = len(credits) - len(set(credits))
pos = [n for n in nets if n > 0]
out["entries_positive_net_dups"] = len(pos) - len(set(pos))

# --- pin 4: BESTIARY_NPC_SHEET_IDS decl == element count, unique
sid = assets[assets.index("pub const BESTIARY_NPC_SHEET_IDS"):]
sdecl = int(re.search(r"BESTIARY_NPC_SHEET_IDS: \[u16; (\d+)\]", sid).group(1))
sid = sid[sid.index("= [") + 3:]
sid = sid[:sid.index("];")]
sid = "\n".join(l.split("//")[0] for l in sid.splitlines())
ids = [int(x) for x in re.findall(r"\d+", sid)]
out["sheet_ids_decl"] = sdecl
out["sheet_ids_elems"] = len(ids)
out["sheet_ids_dups"] = len(ids) - len(set(ids))

# --- pin 5: s1_d_4c — one arm per kind in update_kind_ai (the test's scan)
start = npc.index("fn update_kind_ai")
rest = npc[start:]
body = rest[:rest.index("\n    }")]
counts = collections.Counter()
for line in body.splitlines():
    t = line.lstrip()
    if not (t.startswith("NpcKind::") or t.startswith("| NpcKind::")):
        continue
    for name in re.findall(r"NpcKind::(\w+)", t):
        counts[name] += 1
out["s1_d_4c_kinds"] = len(counts)
out["s1_d_4c_multi_arm"] = {k: n for k, n in counts.items() if n != 1}
out["s1_d_4c_set_eq_enum"] = set(counts) == set(variants)

# --- pin 6: the d39 witnesses (WRITTEN NOT RUN — count only)
out["npc_rs_w4_d39_tests"] = len(re.findall(r"fn w4_d39_\w+", npc))
s65 = read("crates/tr-app/src/npc_ai_style65.rs")
out["style65_tests"] = s65.count("#[test]")

# --- f18 overlap adjudication: the 20-kind lane's to_u16 arms
f18_to = arms_to(show(F18, "crates/tr-app/src/npc/data.rs"))
band = ["Firefly", "Butterfly", "Worm", "LightningBug", "Snail",
        "GlowingSnail", "Frog", "Duck", "Duck2", "DuckWhite",
        "DuckWhite2", "ScorpionBlack", "Scorpion", "TruffleWorm",
        "TruffleWormDigger", "Grasshopper", "ChatteringTeethBomb",
        "CultistArcherBlue", "CultistArcherWhite", "ArmedZombie"]
out["f18_band"] = {k: f18_to.get(k) for k in band}
out["overlap_7"] = {k: {"d39": to[k], "f18": f18_to[k]} for k in
    ["Butterfly", "ScorpionBlack", "Scorpion", "Grasshopper",
     "ChatteringTeethBomb", "CultistArcherBlue", "CultistArcherWhite"]}
# f18's 13 non-overlap kinds vs the E8/SEAL canonical homes (108 S1/S4)
canonical = {"Firefly": 735, "LightningBug": 736, "Worm": 737,
             "TruffleWorm": 738, "TruffleWormDigger": 739,
             "Snail": 740, "GlowingSnail": 741, "Frog": 744,
             "Duck": 731, "Duck2": 732, "DuckWhite": 733,
             "DuckWhite2": 734, "ArmedZombie": 430}
out["f18_nonoverlap_vs_canonical"] = {
    k: {"f18": f18_to.get(k), "canonical": v}
    for k, v in canonical.items()}

# --- the grasshopper correction: AI_001 DOES carry 377 branches
cs = read("decompiled/Terraria/NPC.cs").splitlines()
out["cs_ai001_377_branches"] = [
    i + 1 for i, l in enumerate(cs[60340:61723], start=60340)
    if "this.type == 377" in l]

# --- pin 7: the f18 overlap stat spot-check (sizes/HP/dmg/def/kb/value)
f18_data = show(F18, "crates/tr-app/src/npc/data.rs")
f18_npc = show(F18, "crates/tr-app/src/npc.rs")

def row_values(src, sig, kinds, pat):
    body = fn_body(src, sig)
    got = {}
    pending = []
    for line in body.splitlines():
        t = line.split("//")[0].strip()
        if not t:
            continue
        names = re.findall(r"NpcKind::(\w+)", t)
        m = re.search(pat, t)
        if m:
            for n in pending + names:
                if n in kinds:
                    got[n] = m.group(1)
            pending = []
        elif t.startswith("NpcKind::") or t.startswith("| NpcKind::"):
            # the or-pattern continuation lines (d39's multi-line arms)
            pending.extend(names)
        else:
            pending = []
    return got

seven = ["Butterfly", "ScorpionBlack", "Scorpion", "Grasshopper",
         "ChatteringTeethBomb", "CultistArcherBlue", "CultistArcherWhite"]
checks = {}
for label, dsrc, nsrc, sig, owner, pat in [
    ("size", data, npc, "pub fn size", "data", r"=>\s*(\([\d., ]+\))"),
    ("ai_style", data, npc, "pub fn ai_style", "data", r"=>\s*(\d+)"),
    ("max_hp", data, npc, "fn max_hp", "data", r"=>\s*(\d+)"),
    ("contact_damage", data, npc, "fn contact_damage", "data", r"=>\s*(\d+)"),
    ("defense", npc, npc, "pub fn defense", "npc", r"=>\s*(\d+)"),
    ("value", npc, npc, "pub fn value", "npc", r"=>\s*([\d.]+)"),
    ("knock_back_resist", npc, npc, "pub fn knock_back_resist", "npc",
     r"=>\s*([\d.]+)"),
]:
    a = row_values(dsrc if owner == "data" else nsrc, sig, seven, pat)
    b = row_values(f18_data if owner == "data" else f18_npc, sig, seven, pat)
    checks[label] = {"d39": a, "f18": b, "agree": a == b}
out["f18_stat_spotcheck"] = checks

# --- pin 8: the >=788 corridor scan over EVERY pushed origin tip
# (the §6d merge-clean claim: only d36's 791-804 sub-band + d39 itself)
r = subprocess.run(["git", "-C", REPO, "branch", "-r",
                    "--format=%(refname:short)"], capture_output=True)
branches = [b.strip() for b in r.stdout.decode().splitlines()
            if b.strip() and "HEAD" not in b]
hi = {}
for b in branches:
    src = show(b, "crates/tr-app/src/npc/data.rs")
    if src is None or "pub fn to_u16" not in src:
        src = show(b, "crates/tr-app/src/npc.rs")
    if src is None or "pub fn to_u16" not in src:
        continue
    arms = arms_to(src)
    over = {k: v for k, v in arms.items() if v >= 788}
    if over:
        hi[b] = over
out["hi_corridor_scan"] = hi
out["hi_corridor_branches_scanned"] = len(branches)

print(json.dumps(out, indent=1))
with open(REPO + r"\tools\_d39_seal_census.json", "w",
          encoding="utf-8") as f:
    json.dump(out, f, indent=1)
    f.write("\n")
ok = (out["enum_variants"] == out["all_kinds_decl"] == out["all_kinds_rows"]
      == out["to_u16_arms"] == out["from_u16_arms"] == out["s1_f_3_literal"]
      == 277
      and out["all_kinds_dups"] == 0 and out["to_from_bijection"]
      and out["enum_set_eq_allkinds_set"] and out["arm_set_eq_allkinds_set"]
      and out["max_proto"] == 823 and out["i16_safe"]
      and out["dead_ids_absent"]
      and out["entries_rows"] == 255 and out["entries_credit_dups"] == 0
      and out["entries_positive_net_dups"] == 0
      and out["sheet_ids_decl"] == out["sheet_ids_elems"] == 187
      and out["sheet_ids_dups"] == 0
      and out["s1_d_4c_kinds"] == 277 and not out["s1_d_4c_multi_arm"]
      and out["s1_d_4c_set_eq_enum"]
      and out["npc_rs_w4_d39_tests"] == 16 and out["style65_tests"] == 8
      and len(out["cs_ai001_377_branches"]) >= 4
      and all(c["agree"] for c in out["f18_stat_spotcheck"].values())
      and set(out["hi_corridor_scan"]) == {
          "origin/task/W4-d36-npc-kinds-141-210",
          "origin/task/W4-d39-npc-kinds-351-420"})
print("SEAL PINS:", "ALL GREEN" if ok else "FAIL")
assert ok
