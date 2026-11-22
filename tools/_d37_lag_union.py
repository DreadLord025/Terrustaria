# W4-d37 LAGGARD-EXEC: the FINAL union re-derive. SEAL final (282 core) +
# post-SEAL movers adjudicated by the E3 kinds-lane / keep-wired-superset /
# cs==proto rules. Scripted uniqueness + base-clash + count gates.
import re, subprocess, json

REPO = r"C:\Users\DreadLord\Desktop\TR-W4-d37"
DATA = "crates/tr-app/src/npc/data.rs"

def show(rev, path):
    r = subprocess.run(["git", "-C", REPO, "show", f"{rev}:{path}"],
                       capture_output=True)
    return r.stdout.decode("utf8", "replace") if r.returncode == 0 else ""

def fn_body(src, sig):
    i = src.index(sig); j = src.index("{", i); d = 0; k = j
    while True:
        c = src[k]
        if c == "{": d += 1
        elif c == "}":
            d -= 1
            if d == 0: return src[j:k+1]
        k += 1

def arms(rev):
    src = show(rev, DATA)
    return {m[0]: int(m[1]) for m in
            re.findall(r"NpcKind::(\w+) => (\d+),", fn_body(src, "pub fn to_u16"))}

seal = json.load(open(REPO + r"\tools\_d37_seal_final.json"))
final = dict(seal["final"])          # 282 core, post-move ids
owners = {k: v[0] for k, v in seal["owners"].items()}
base_ids = set(arms("a725acb04").values())

# ---- post-SEAL additions (kind -> (id, owner-lane, verdict))
add = {}
def put(k, i, lane, why):
    assert k not in add, k
    add[k] = (i, lane, why)

# d39 EXECUTED re-pin 817-823 (supersedes SEAL's 781-787 acceptance; the
# lane consumed R7 itself, collision-free, branch-only ids)
for k, v in arms("origin/task/W4-d39-npc-kinds-351-420").items():
    if v >= 817: put(k, v, "d39", "ACCEPT as-pushed R7 re-pin 817-823")
# d41 EXECUTED +9 on top of the SEAL-accepted 494-506; GiantFlyingAntlion
# 509 is c4's pushed kind (same id) -> keep c4, d41 drops its dup row;
# SolarFlare 516 is d23's SEAL-owned kind (grand 408) -> keep d23, d41 drops
for k, v in arms("origin/task/W4-d41-npc-kinds-491-560").items():
    if v >= 494 and k not in final and k not in ("GiantFlyingAntlion", "SolarFlare"):
        put(k, v, "d41", "ACCEPT as-pushed 494-519 band")
# g10 PUSHED (SEAL local-accept now on origin)
for k, v in arms("origin/task/W4-g10-npc-kind-wire-3").items():
    if v >= 637 and k not in final: put(k, v, "g10", "ACCEPT pushed pets cs==proto")
# d36 EXECUTED five at 762-766 -> RE-PIN 767-771 (763-766 are d120's
# owed town moves per R6; 762 reserved for g8's Vampire renumber)
for k, old, new in [("ToxicSludge",762,767),("Penguin",763,768),
                    ("PenguinBlack",764,769),("GiantTortoise",765,770),
                    ("IceTortoise",766,771)]:
    put(k, new, "d36", f"RE-PIN {old}->{new} (R6 towns 763-766 + Vampire 762)")
# g8 WIP Vampire@388 squats d37's RustyArmoredBonesAxe -> 762 (cs 159)
put("Vampire", 762, "g8", "RENUMBER WIP 388->762 (d37 owns 388)")
# d43 EXECUTED 63 kinds: 11 g10 dups + HallowBoss/EmpressButterfly/
# VileSpitEaterOfWorlds aliases + 6 grand-owned dups drop; the rest ACCEPT
ALIAS = {"HallowBoss": "EmpressOfLight", "EmpressButterfly": "PrismaticLacewing",
         "VileSpitEaterOfWorlds": "VileSpitEoW"}
g10k = {k for k, (i, l, w) in add.items() if l == "g10"}
d43_drop, d43_new = [], {}
for k, v in arms("origin/task/W4-d43-npc-kinds-631-697").items():
    if v < 631: continue
    if k in ALIAS: d43_drop.append((k, v, "alias of " + ALIAS[k])); continue
    if k in g10k: d43_drop.append((k, v, "dup g10")); continue
    if k in final: d43_drop.append((k, v, "dup " + owners[k])); continue
    d43_new[k] = v
    put(k, v, "d43", "ACCEPT as-pushed cs==proto 631-697")
# the three stranded uniques from claim-only lanes -> the free 788-790.
# cs_id falls through to the bestiary net_id on ALL THREE lanes (scripted
# read): the E8 convention keeps cs/net/texture VANILLA (546/172/521) and
# moves ONLY the to_u16/from_u16 protocol arms; cs_id==entry.net_id holds
# (the Lacewing 742/cs-661 precedent). Entry rows + sheets DO NOT move.
put("Tumbleweed", 788, "f42", "RENUMBER proto 354->788 ONLY (cs/net/texture stay 546)")
put("RuneWizard", 789, "g5", "RENUMBER proto 354->789 ONLY (cs/net/texture stay 172)")
put("AncientCultistSquidhead", 790, "d9", "RENUMBER proto 521->790 ONLY (cs/net/texture stay 521)")

# ---- gates
union = dict(final)
for k, (i, l, w) in add.items():
    assert k not in union, ("dup kind", k)
    union[k] = i
ids = list(union.values())
assert len(ids) == len(set(ids)), "id collision: " + str(
    [(i, [k for k in union if union[k] == i]) for i in set(ids) if ids.count(i) > 1])
clash = {k: v for k, v in union.items() if v in base_ids}
assert not clash, ("base clash", clash)
assert all(0 < v <= 32767 for v in ids)

out = {
  "core_kinds": len(final), "grand_kinds": len(union),
  "max_id": max(ids), "core_max": max(final.values()),
  "additions": {k: {"id": i, "lane": l, "verdict": w} for k, (i, l, w) in add.items()},
  "d43_drops": d43_drop,
  "grand": union,
  "aliases": ALIAS,
}
json.dump(out, open(REPO + r"\tools\_d37_lag_union.json", "w"), indent=1, sort_keys=True)
print("core", len(final), "grand", len(union), "max", max(ids), "core-max", max(final.values()))
print("additions:", len(add), "d43 drops:", len(d43_drop))
print("d43 accepted:", len(d43_new))
print("ALL UNION GATES PASSED (unique ids, zero base clash, i16-safe)")
