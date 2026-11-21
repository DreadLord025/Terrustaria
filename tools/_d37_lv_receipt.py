# W4-d37 ROSTER-LAGGARD-VERIFY (successor probe, read-only): current tips,
# laggard statuses, d36-R7 receipt, d125-regression resolution, d37 zero-move,
# and the free-id check for d36's new hover batch vs the lag-union witness.
# Direct paths only, git-show only, no junction recursion. Writes ONLY
# tools/_d37_lv_receipt.json (does not touch the live executor's _d37_lag_*).
import re, subprocess, json, hashlib, datetime

REPO = r"C:\Users\DreadLord\Desktop\TR-W4-d37"
DATA = "crates/tr-app/src/npc/data.rs"
NPC = "crates/tr-app/src/npc.rs"

def git(*a):
    r = subprocess.run(["git", "-C", REPO] + list(a), capture_output=True)
    return r.returncode, r.stdout.decode("utf8", "replace")

def show(rev, path):
    rc, out = git("show", f"{rev}:{path}")
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

def to_arms(src):
    return {m[0]: int(m[1]) for m in
            re.findall(r"NpcKind::(\w+)\s*=>\s*(\d+)", fn_body(src, "pub fn to_u16"))}

def allkinds(src):
    m = re.search(r"const ALL_KINDS:\s*\[NpcKind;\s*(\d+)\]\s*=\s*\[(.*?)\];",
                  src, re.S)
    if not m: return None, []
    return int(m.group(1)), re.findall(r"NpcKind::(\w+)", m.group(2))

out = {"utc_local_time": datetime.datetime.now().isoformat(timespec="seconds")}

# --- tips (origin, post-fetch)
BR = {"c9": "task/W4-c9-style67-snails", "g8": "task/W4-g8-npc-kind-wire-1",
      "d36": "task/W4-d36-npc-kinds-141-210",
      "d36x": "task/W4-d36x-npc-kinds-141-210-exec",
      "d125": "task/W4-d125-empress-r7-drain",
      "d42": "task/W4-d42-npc-kinds-561-630",
      "c4": "task/W4-c4-style44-flying-fish",
      "d39": "task/W4-d39-npc-kinds-351-420",
      "d41": "task/W4-d41-npc-kinds-491-560",
      "d43": "task/W4-d43-npc-kinds-631-697",
      "g10": "task/W4-g10-npc-kind-wire-3"}
tips = {}
for lane, br in BR.items():
    rc, h = git("rev-parse", "--short=9", f"origin/{br}")
    rc2, d = git("log", "-1", "--format=%ci", f"origin/{br}")
    tips[lane] = {"origin": h.strip(), "date": d.strip()}
out["tips"] = tips

# --- c9: origin still stale; local tip carries the latent ALL_KINDS fix?
src = show("origin/" + BR["c9"], NPC)
decl, rows = allkinds(src)
to = to_arms(show("origin/" + BR["c9"], DATA))
out["c9_origin"] = {"decl": decl, "rows": len(rows), "to_arms": len(to),
                    "snails_in_array": [k for k in
                        ("Snail", "GlowingSnail", "MagmaSnail", "Shimmerfly")
                        if k in rows]}
rc, lh = git("-C", r"C:\Users\DreadLord\Desktop\TR-W4-c9", "rev-parse", "--short=9", "HEAD")
lsrc = show(lh.strip(), NPC)
ldecl, lrows = allkinds(lsrc)
lto = to_arms(show(lh.strip(), DATA))
out["c9_local"] = {"tip": lh.strip(), "decl": ldecl, "rows": len(lrows),
                   "to_arms": len(lto),
                   "array_set_eq_to_set": set(lrows) == set(lto),
                   "snails_in_array": [k for k in
                       ("Snail", "GlowingSnail", "MagmaSnail", "Shimmerfly")
                       if k in lrows],
                   "gold_critters": {k: lto.get(k) for k in
                       ("GoldBird", "GoldGrasshopper", "GoldMouse")},
                   "unpushed": git("rev-list", "--count",
                       f"origin/{BR['c9']}..{lh.strip()}")[1].strip()}

# --- g8: new pushed tip state (machine rounds landed; E8 table still owed)
gsrc = show("origin/" + BR["g8"], NPC)
gdecl, grows = allkinds(gsrc)
gto = to_arms(show("origin/" + BR["g8"], DATA))
out["g8_origin"] = {"decl": gdecl, "rows": len(grows), "to_arms": len(gto),
                    "vampire": gto.get("Vampire"),
                    "flying_snake": gto.get("FlyingSnake"),
                    "e8_moves_executed": gto.get("PinkJellyfish") != 356}

# --- d36: R7 re-pin receipt + the new hover batch ids
d36to = to_arms(show("origin/" + BR["d36"], DATA))
out["d36_receipt"] = {k: d36to.get(k) for k in
    ("ToxicSludge", "Penguin", "PenguinBlack", "GiantTortoise", "IceTortoise",
     "Crimera", "MossHornet", "Moth")}
out["d36_r7_executed"] = all(d36to.get(k) == v for k, v in
    [("ToxicSludge", 767), ("Penguin", 768), ("PenguinBlack", 769),
     ("GiantTortoise", 770), ("IceTortoise", 771)])

# --- d39: R7 self-consumed 817-823?
d39to = to_arms(show("origin/" + BR["d39"], DATA))
out["d39_r7"] = {k: d39to.get(k) for k in
    ("Butterfly", "ScorpionBlack", "Scorpion", "Grasshopper",
     "ChatteringTeethBomb", "CultistArcherBlue", "CultistArcherWhite")}

# --- d125: regression resolved? (sheets commit ancestor of origin tip)
out["d125_sheets_ancestor"] = git("merge-base", "--is-ancestor",
    "cd2b1b1e3", "origin/" + BR["d125"])[0] == 0
d125to = to_arms(show("origin/" + BR["d125"], DATA))
d125decl, d125rows = allkinds(show("origin/" + BR["d125"], NPC))
out["d125_state"] = {"decl": d125decl, "rows": len(d125rows),
                     "EmpressOfLight": d125to.get("EmpressOfLight"),
                     "PrismaticLacewing": d125to.get("PrismaticLacewing")}

# --- d42: the 7 dragonfly drops still owed?
d42to = to_arms(show("origin/" + BR["d42"], DATA))
out["d42_dragonflies_present"] = {k: d42to.get(k) for k in
    ("BlackDragonfly", "BlueDragonfly", "GreenDragonfly", "OrangeDragonfly",
     "RedDragonfly", "YellowDragonfly", "GoldDragonfly")}

# --- d37 zero-move re-verify
rc, df = git("diff", "--stat", "750d8e44", "HEAD", "--", "crates", "anchors.txt")
out["d37_zero_code_diff"] = df.strip() == ""

# --- lag-union witness (read-only cite) + d36 hover-batch free-id check
u = json.load(open(REPO + r"\tools\_d37_lag_union.json"))
out["lag_union"] = {k: u[k] for k in
    ("core_kinds", "grand_kinds", "max_id", "core_max")}
used = set(u["grand"].values())
out["d36_hover_batch_vs_union"] = {
    k: {"id": d36to.get(k), "in_union": d36to.get(k) in used,
        "kind_in_union": k in u["grand"]}
    for k in ("Crimera", "MossHornet", "Moth")}

json.dump(out, open(REPO + r"\tools\_d37_lv_receipt.json", "w"),
          indent=1, sort_keys=True)
print(json.dumps(out, indent=1, sort_keys=True))
