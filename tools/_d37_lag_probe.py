# W4-d37 LAGGARD-EXEC phase: probe the flagged lanes at CURRENT tips.
# Read-only git (rev-parse/merge-base/show/log). Direct paths, no junctions.
import re, subprocess, json

REPO = r"C:\Users\DreadLord\Desktop\TR-W4-d37"
DATA = "crates/tr-app/src/npc/data.rs"
NPC = "crates/tr-app/src/npc.rs"

def git(repo, *a):
    r = subprocess.run(["git", "-C", repo] + list(a), capture_output=True)
    return r.returncode, r.stdout.decode("utf8", "replace")

def show(rev, path, repo=REPO):
    rc, out = git(repo, "show", f"{rev}:{path}")
    return out if rc == 0 else None

def fn_body(src, sig):
    i = src.index(sig); j = src.index("{", i); d = 0; k = j
    while True:
        c = src[k]
        if c == "{": d += 1
        elif c == "}":
            d -= 1
            if d == 0: return src[j:k+1]
        k += 1

def arms(src):
    to = {m[0]: int(m[1]) for m in
          re.findall(r"NpcKind::(\w+)\s*=>\s*(\d+)", fn_body(src, "pub fn to_u16"))}
    fr = {int(m[0]): m[1] for m in
          re.findall(r"(\d+)\s*=>\s*NpcKind::(\w+)", fn_body(src, "pub fn from_u16"))}
    return to, fr

def allkinds(src):
    m = re.search(r"const ALL_KINDS:\s*\[NpcKind;\s*(\d+)\]\s*=\s*\[(.*?)\];",
                  src, re.S)
    if not m: return None, []
    return int(m.group(1)), re.findall(r"NpcKind::(\w+)", m.group(2))

out = {}

# --- c9: origin tip vs local tip; is session alive (tip age)? patch applies?
for lane, br, wt in [
    ("c9", "task/W4-c9-style67-snails", r"C:\Users\DreadLord\Desktop\TR-W4-c9"),
    ("g8", "task/W4-g8-npc-kind-wire-1", r"C:\Users\DreadLord\Desktop\TR-W4-g8"),
    ("d125", "task/W4-d125-empress-r7-drain", r"C:\Users\DreadLord\Desktop\TR-W4-d125"),
    ("d42", "task/W4-d42-npc-kinds-561-630", r"C:\Users\DreadLord\Desktop\TR-W4-d42"),
    ("c4", "task/W4-c4-style44-flying-fish", r"C:\Users\DreadLord\Desktop\TR-W4-c4"),
]:
    rc, o_tip = git(REPO, "rev-parse", f"origin/{br}")
    rc2, l_tip = git(wt, "rev-parse", "HEAD")
    rc3, age = git(REPO, "log", "-1", "--format=%ci", f"origin/{br}")
    anc = git(wt, "merge-base", "--is-ancestor", o_tip.strip(), l_tip.strip())[0] if rc == 0 and rc2 == 0 else None
    ab = git(wt, "rev-list", "--left-right", "--count",
             f"origin/{br}...HEAD")[1].strip() if anc is not None else "?"
    st = git(wt, "status", "--porcelain=v1")[1]
    ent = {"origin_tip": o_tip.strip()[:9], "origin_tip_date": age.strip(),
           "local_tip": l_tip.strip()[:9] if rc2 == 0 else None,
           "origin_is_ancestor_of_local": anc == 0,
           "behind_ahead": ab, "dirty_files": len([l for l in st.splitlines() if l.strip()])}
    out[lane] = ent

# --- per-lane kind state at ORIGIN tip (and d125 local tip)
def lane_state(rev, keys):
    src = show(rev, DATA)
    if src is None:
        return {"error": "no data.rs"}
    to, fr = arms(src)
    nsrc = show(rev, NPC)
    decl, rows = allkinds(nsrc) if nsrc else (None, [])
    r = {"decl": decl, "rows": len(rows), "to_arms": len(to), "from_arms": len(fr),
         "bijection_ok": set(rows) == set(to) and {to[k] for k in to} == set(fr)}
    r["kinds"] = {k: to.get(k) for k in keys}
    return r

final = json.load(open(REPO + r"\tools\_d37_seal_final.json"))
owners, fmap = final["owners"], final["final"]
by_lane = {}
for k, (lane, old) in owners.items():
    by_lane.setdefault(lane, []).append(k)

snails = ["Snail", "GlowingSnail", "MagmaSnail", "Shimmerfly"]
out["c9_state_origin"] = lane_state("origin/task/W4-c9-style67-snails", snails + by_lane.get("c9", []))
out["g8_state_origin"] = lane_state("origin/task/W4-g8-npc-kind-wire-1", by_lane.get("g8", []))
out["d42_state_origin"] = lane_state("origin/task/W4-d42-npc-kinds-561-630",
    ["Dragonfly", "Dragonfly2", "Dragonfly3", "Dragonfly4", "Dragonfly5", "Dragonfly6", "Dragonfly7"])
out["c4_state_origin"] = lane_state("origin/task/W4-c4-style44-flying-fish",
    ["BloodFeeder", "CorruptGoldfish", "FlyingFish"] + by_lane.get("c4", []))
out["d125_state_local"] = lane_state("7a0482302", ["Empress", "EmpressButterfly", "Lacewing", "LightningBug"])

# d42 movement since the seal-referenced 49346dba6
rc, log = git(REPO, "log", "--oneline", "49346dba6..origin/task/W4-d42-npc-kinds-561-630")
out["d42_new_commits"] = log.strip().splitlines()
rc, df = git(REPO, "diff", "--stat", "49346dba6", "origin/task/W4-d42-npc-kinds-561-630")
out["d42_diff_stat"] = df.strip()

# c9 patch still applies at current origin tip? blob-hash comparison only
import hashlib
blob = subprocess.run(["git", "-C", REPO, "show",
                       f"origin/task/W4-c9-style67-snails:{NPC}"],
                      capture_output=True).stdout
h = hashlib.sha1(b"blob %d\0" % len(blob) + blob).hexdigest()
out["c9_tip_npc_blob"] = h
out["c9_local_npc_blob"] = hashlib.sha1(
    b"blob %d\0" % len(open(r"C:\Users\DreadLord\Desktop\TR-W4-c9\crates\tr-app\src\npc.rs", "rb").read())
    + open(r"C:\Users\DreadLord\Desktop\TR-W4-c9\crates\tr-app\src\npc.rs", "rb").read()).hexdigest()

json.dump(out, open(REPO + r"\tools\_d37_lag_probe.json", "w"), indent=1, sort_keys=True)
print(json.dumps(out, indent=1, sort_keys=True))
