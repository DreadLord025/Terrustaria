# W4-d35 SEAL-VERIFY: show WHICH const-list each claimant's new rows land in
# (STATIC_SHEET_NAMES vs GLOW_SHEET_NAMES vs other), with hunk headers.
import subprocess, re, sys

BASE = "a725acb0"
def git(*a):
    return subprocess.run(["git", *a], capture_output=True, text=True,
                          encoding="utf-8", errors="replace").stdout

for slug in sys.argv[1:]:
    b = "origin/task/W4-" + slug
    d = git("diff", BASE + ".." + b, "--", "crates/tr-app/src/assets.rs")
    print("=" * 20, slug)
    for l in d.splitlines():
        if l.startswith("@@") or re.match(r'^\+\s+"[^"{} ]+",?\s*(//.*)?$', l) \
           or re.match(r'^\+.*(const|static)\s', l):
            print(l[:160])
