# W4-d37 LAGGARD-EXEC probe 2: c9 local tip (S2 fix consumed? new dup ports?),
# g8 dirty WIP (moves in progress?), d42 dragonfly rows at tip, d125 arms.
import re, subprocess, json

REPO = r"C:\Users\DreadLord\Desktop\TR-W4-d37"
DATA = "crates/tr-app/src/npc/data.rs"
NPC = "crates/tr-app/src/npc.rs"

def git(repo, *a):
    r = subprocess.run(["git", "-C", repo] + list(a), capture_output=True)
    return r.returncode, r.stdout.decode("utf8", "replace")

def fn_body(src, sig):
    i = src.index(sig); j = src.index("{", i); d = 0; k = j
    while True:
        c = src[k]
        if c == "{": d += 1
        elif c == "}":
            d -= 1
            if d == 0: return src[j:k+1]
        k += 1

def arms_to(src):
    return {m[0]: int(m[1]) for m in
            re.findall(r"NpcKind::(\w+)\s*=>\s*(\d+)", fn_body(src, "pub fn to_u16"))}

def allkinds(src):
    m = re.search(r"const ALL_KINDS:\s*\[NpcKind;\s*(\d+)\]\s*=\s*\[(.*?)\];",
                  src, re.S)
    if not m: return None, []
    return int(m.group(1)), re.findall(r"NpcKind::(\w+)", m.group(2))

out = {}
C9 = r"C:\Users\DreadLord\Desktop\TR-W4-c9"
G8 = r"C:\Users\DreadLord\Desktop\TR-W4-g8"

# --- c9 local tip c830aec3d: ALL_KINDS state + snail/gold arms + s1_f_3 pin
rc, nsrc = git(C9, "show", f"c830aec3d:{NPC}")
rc2, dsrc = git(C9, "show", f"c830aec3d:{DATA}")
decl, rows = allkinds(nsrc)
to = arms_to(dsrc)
pin = re.search(r"s1_f_3[\s\S]{0,2000}?(\d{3})\s*,?\s*\)?;?\s*//", nsrc)
out["c9_local"] = {
    "decl": decl, "rows": len(rows), "to_arms": len(to),
    "array_set_eq_to_set": set(rows) == set(to),
    "snails_in_array": [k for k in ["Snail","GlowingSnail","MagmaSnail","GlowingSnail"] if k in rows],
    "kinds": {k: to.get(k) for k in ["Snail","GlowingSnail","MagmaSnail","Shimmerfly",
                                     "GoldBird","GoldGrasshopper","GoldMouse"]},
}
m = re.findall(r"ALL_KINDS\.len\(\)[\s\S]{0,120}", nsrc)
out["c9_local_len_uses"] = m[:4]
m2 = re.findall(r"assert_eq!\([^\n]*\b(27\d|28\d)\b[^\n]*\)", nsrc)
out["c9_local_pins"] = m2[:8]
# c9 dirty diff summary
out["c9_dirty_stat"] = git(C9, "diff", "--stat")[1].strip().splitlines()[-1] if git(C9,"diff","--stat")[1].strip() else ""
out["c9_unpushed_files"] = git(C9, "diff", "--name-only", "origin/task/W4-c9-style67-snails", "HEAD")[1].split()

# --- g8 dirty WIP: does the working tree already move ids / fix ALL_KINDS?
wdata = open(G8 + r"\crates\tr-app\src\npc\data.rs", encoding="utf8").read()
wnpc = open(G8 + r"\crates\tr-app\src\npc.rs", encoding="utf8").read()
wto = arms_to(wdata)
wdecl, wrows = allkinds(wnpc)
out["g8_worktree"] = {
    "decl": wdecl, "rows": len(wrows), "to_arms": len(wto),
    "kinds": {k: wto.get(k) for k in ["PinkJellyfish","Antlion","SpikeBall","WallCreeper",
              "WallCreeperWall","IceElemental","Herpling","Derpling","CrimsonAxe","FloatyGross",
              "LostGirl","Nymph","IceBat","Lavabat","GiantFlyingFox","VampireBat","RedDevil"]},
}
out["g8_dirty_stat"] = git(G8, "diff", "--stat")[1].strip().splitlines()

# --- d42 dragonflies at origin tip
rc, dsrc = git(REPO, "show", f"origin/task/W4-d42-npc-kinds-561-630:{DATA}")
to = arms_to(dsrc)
out["d42_dragonflies"] = {k: to.get(k) for k in
    ["BlackDragonfly","BlueDragonfly","GreenDragonfly","OrangeDragonfly","RedDragonfly",
     "YellowDragonfly","GoldDragonfly"]}
out["d42_waterfowl"] = {k: to.get(k) for k in ["Seagull","Seagull2","Grebe","Grebe2"]}
# research/239 exists at tip?
rc, doc = git(REPO, "show", "origin/task/W4-d42-npc-kinds-561-630:docs/research/239-d42-final-keep-one-and-band.md")
out["d42_239_lines"] = len(doc.splitlines()) if rc == 0 else None

# --- d125 arms at pushed tip: the two owed moves' actual kind names/ids
rc, dsrc = git(REPO, "show", f"origin/task/W4-d125-empress-r7-drain:{DATA}")
to = arms_to(dsrc)
rc, nsrc = git(REPO, "show", f"origin/task/W4-d125-empress-r7-drain:{NPC}")
decl, rows = allkinds(nsrc)
newk = {k: v for k, v in to.items() if v >= 229 and k not in ("AngryBones",)}
out["d125_tip"] = {"decl": decl, "rows": len(rows), "to_arms": len(to),
    "kinds": {k: to.get(k) for k in ["EmpressOfLight","PrismaticLacewing"]}}

json.dump(out, open(REPO + r"\tools\_d37_lag_probe2.json", "w"), indent=1, sort_keys=True)
print(json.dumps(out, indent=1, sort_keys=True))
