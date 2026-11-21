# W4-d37 LAGGARD-EXEC: exact ALL_KINDS row census (row LINES only, not
# comment mentions) + decl + s1_f_3 pin per laggard tip. Compile-vs-test
# failure classification.
import re, subprocess, json

REPO = r"C:\Users\DreadLord\Desktop\TR-W4-d37"

def show(rev, p):
    r = subprocess.run(["git", "-C", REPO, "show", f"{rev}:{p}"],
                       capture_output=True)
    return r.stdout.decode("utf8", "replace") if r.returncode == 0 else ""

def census(rev):
    n = show(rev, "crates/tr-app/src/npc.rs")
    m = re.search(r"const ALL_KINDS: \[NpcKind; (\d+)\] = \[(.*?)\n    \];", n, re.S)
    if not m:
        return {"error": "no array"}
    decl = int(m.group(1))
    body = m.group(2)
    rows = re.findall(r"^\s*NpcKind::(\w+),\s*$", body, re.M)
    i = n.find("covered + sentinel")
    pin = None
    if i >= 0:
        pin = int(re.search(r"(\d+),", n[i:]).group(1))
    return {"decl": decl, "rows": len(rows), "pin": pin,
            "decl_line": n[:m.start()].count("\n") + 1,
            "compiles": decl == len(rows),
            "dup_rows": len(rows) != len(set(rows))}

TIPS = {
 "c9-origin": "2042d77f1",
 "c9-local": "c830aec3d",
 "g8-new": "1752c70df",
 "d125": "origin/task/W4-d125-empress-r7-drain",
 "d42": "origin/task/W4-d42-npc-kinds-561-630",
 "c7": "origin/task/W4-c7-style64-fireflies",
 "c10": "origin/task/W4-c10-style68-waterfowl",
 "d35": "origin/task/W4-d35-npc-kinds-71-140",
 "d36": "origin/task/W4-d36-npc-kinds-141-210",
 "d39": "origin/task/W4-d39-npc-kinds-351-420",
 "d41": "origin/task/W4-d41-npc-kinds-491-560",
 "d43": "origin/task/W4-d43-npc-kinds-631-697",
 "f18": "origin/task/W4-f18-npc-kinds-verify-2",
 "f42": "origin/task/W4-f42-sandstorm-2",
 "g10": "origin/task/W4-g10-npc-kind-wire-3",
}
out = {k: census(v) for k, v in TIPS.items()}
json.dump(out, open(REPO + r"\tools\_d37_lag_akcensus.json", "w"), indent=1, sort_keys=True)
for k, v in out.items():
    print(f"{k:12s} {v}")
