# W4-d37 LAGGARD-EXEC: locate ALL_KINDS.len() literal pins (multi-line
# assert shape) on the lanes whose s1_f_3 literal is stale but decl fresh.
import re, subprocess

REPO = r"C:\Users\DreadLord\Desktop\TR-W4-d37"

def show(rev, p):
    r = subprocess.run(["git", "-C", REPO, "show", f"{rev}:{p}"],
                       capture_output=True)
    return r.stdout.decode("utf8", "replace")

for lane, rev in [("c7", "origin/task/W4-c7-style64-fireflies"),
                  ("c10", "origin/task/W4-c10-style68-waterfowl"),
                  ("g10", "origin/task/W4-g10-npc-kind-wire-3"),
                  ("d125", "origin/task/W4-d125-empress-r7-drain"),
                  ("g8", "1752c70df")]:
    n = show(rev, "crates/tr-app/src/npc.rs")
    hits = []
    for m in re.finditer(r"ALL_KINDS\.len\(\)", n):
        seg = n[m.start():m.start()+120]
        ln = n[:m.start()].count("\n") + 1
        num = re.search(r"ALL_KINDS\.len\(\),\s*(\d+)", seg)
        if num:
            hits.append((ln, num.group(1)))
    print(lane, "literal len pins:", hits)
