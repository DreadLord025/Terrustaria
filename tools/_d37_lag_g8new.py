# W4-d37 LAGGARD-EXEC: g8 NEW pushed tip 1752c70df (machine rounds 3+4 +
# claims-table landed mid-phase). Re-derive all 35 move/drop arm sites.
import re, subprocess, json

REPO = r"C:\Users\DreadLord\Desktop\TR-W4-d37"
REV = "1752c70df"

def show(p):
    r = subprocess.run(["git", "-C", REPO, "show", f"{REV}:{p}"],
                       capture_output=True)
    return r.stdout.decode("utf8", "replace")

def fnspan(src, sig):
    i = src.index(sig); j = src.index("{", i); depth = 0; k = j
    while True:
        c = src[k]
        if c == "{": depth += 1
        elif c == "}":
            depth -= 1
            if depth == 0: break
        k += 1
    return src[:i].count("\n"), src[i:k+1]

n = show("crates/tr-app/src/npc.rs")
d = show("crates/tr-app/src/npc/data.rs")
m = re.search(r"const ALL_KINDS: \[NpcKind; (\d+)\] = \[(.*?)\];", n, re.S)
rows = re.findall(r"NpcKind::(\w+)", m.group(2))
out = {"tip": REV, "decl": [n[:m.start()].count("\n") + 1, int(m.group(1))],
       "rows": len(rows)}
i = n.find("covered + sentinel")
out["pin"] = [n[:i].count("\n") + 2, n[i:i+60].split("\n")[1].strip()]

bt, bo = fnspan(d, "pub fn to_u16")
bf, fr = fnspan(d, "pub fn from_u16")
kinds = ["PinkJellyfish","Antlion","SpikeBall","WallCreeper","WallCreeperWall",
         "IceElemental","Herpling","Derpling","CrimsonAxe","FloatyGross",
         "LostGirl","Nymph","IceBat","Lavabat","GiantFlyingFox","VampireBat",
         "RedDevil","Vampire","Vulture","BlueJellyfish","Pixie","Wraith",
         "CursedHammer","EnchantedSword","Mimic","GreenJellyfish","Gastropod",
         "CursedSkull","Demon","GiantBat","Slimer","IlluminantBat",
         "FlyingSnake","Squid","Harpy"]
nl = n.split("\n")
for k in kinds:
    t = [bt+i+1 for i, l in enumerate(bo.split("\n"))
         if re.search(rf"NpcKind::{k} => (\d+),", l)]
    f = [bf+i+1 for i, l in enumerate(fr.split("\n"))
         if re.search(rf"(\d+) => NpcKind::{k},", l)]
    tid = re.search(rf"NpcKind::{k} => (\d+),", bo)
    ak = [j+1 for j, l in enumerate(nl) if l.strip() == f"NpcKind::{k},"]
    out[k] = {"id": int(tid.group(1)) if tid else None,
              "to": t, "from": f, "ak": ak}

json.dump(out, open(REPO + r"\tools\_d37_lag_g8_newtip.json", "w"),
          indent=1, sort_keys=True)
print("decl", out["decl"], "rows", out["rows"], "pin", out["pin"])
for k in ["PinkJellyfish", "RedDevil", "Vampire", "Vulture", "Harpy"]:
    print(k, out[k])
