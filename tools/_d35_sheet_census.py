# W4-d35 sheets phase: cumulative pushed sheet-addition census.
# For every origin/task/* branch, diff assets.rs vs the wave base a725acb0 and
# report added/removed quoted sheet-name rows. Also flags atlas.rs / harness
# changes on any branch (packer drift would invalidate the sim).
import subprocess

BASE = "a725acb0"

def git(*args):
    return subprocess.run(["git", *args], capture_output=True, text=True,
                          encoding="utf-8", errors="replace").stdout

branches = [b for b in git("branch", "-r", "--format=%(refname:short)").split()
            if b.startswith("origin/task/")]
print("branches scanned:", len(branches))
for b in branches:
    d = git("diff", BASE + ".." + b, "--", "crates/tr-app/src/assets.rs")
    if d.strip():
        added = [l.strip() for l in d.splitlines()
                 if l.startswith("+") and '"' in l]
        removed = [l.strip() for l in d.splitlines()
                   if l.startswith("-") and '"' in l and not l.startswith("---")]
        print("===", b)
        for l in added:
            print("  +", l)
        for l in removed:
            print("  -", l)
    p = git("diff", "--stat", BASE + ".." + b, "--",
            "crates/tr-render/src/atlas.rs", "tools/atlas_sim.js")
    if p.strip():
        print("### PACKER/HARNESS CHANGE on", b)
        print(p)
