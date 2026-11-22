# W4-d40 sheets phase: extract the pure sheet-name string rows each pushed
# branch ADDS to assets.rs (vs the wave base a725acb0), for the cumulative
# atlas-headroom scenarios. Only `+  "Name",` rows count (const-list rows);
# `atlas.sheet(...)` code lines are lookups, not packs. Re-extracted fresh so
# it includes d35's sheets-phase D35_SHEETS and d38's batch additions.
import subprocess, re, json, os

BASE = "a725acb0"
def git(*a):
    return subprocess.run(["git", *a], capture_output=True, text=True,
                          encoding="utf-8", errors="replace").stdout

branches = [b for b in git("branch", "-r", "--format=%(refname:short)").split()
            if b.startswith("origin/task/")]
row = re.compile(r'^\+\s+"([^"{} ]+)",?\s*$')
out = {}
for b in branches:
    d = git("diff", BASE + ".." + b, "--", "crates/tr-app/src/assets.rs")
    if not d.strip():
        continue
    added = [m.group(1) for l in d.splitlines() if (m := row.match(l))]
    if added:
        out[b] = added
here = os.path.dirname(os.path.abspath(__file__))
with open(os.path.join(here, "_d40_pushed_sheets.json"), "w") as f:
    json.dump(out, f, indent=1)
total = sorted({n for v in out.values() for n in v})
print("branches with additions:", len(out), "| union:", len(total))
for b, v in out.items():
    print(b, len(v))
