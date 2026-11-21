# W4-d35 RETIRE probe: the pushed-extract regex misses `+  "Name",  // comment`
# rows (e23's MapBG set). Count them across all branches; compare vs HEAD's
# committed extraction.
import json, subprocess, re

def git(*a):
    return subprocess.run(["git", *a], capture_output=True, text=True,
                          encoding="utf-8", errors="replace").stdout

BASE = "a725acb0"
old = json.loads(git("show", "HEAD:tools/_d35_pushed_sheets.json"))
new = json.load(open(r"tools\_d35_pushed_sheets.json"))
on = {n for v in old.values() for n in v}
nn = {n for v in new.values() for n in v}
print("MapBG in HEAD union:", sorted(n for n in on if n.startswith("MapBG")))
print("MapBG in new union:", sorted(n for n in nn if n.startswith("MapBG")))
print("e23 HEAD:", old.get("origin/task/W4-e23-bestiary-ui-grid"))
print("e23 new:", new.get("origin/task/W4-e23-bestiary-ui-grid"))

# the comment-tolerant row regex
row_c = re.compile(r'^\+\s+"([^"{} ]+)",?\s*(//.*)?$')
row_s = re.compile(r'^\+\s+"([^"{} ]+)",?\s*$')
branches = [b for b in git("branch", "-r", "--format=%(refname:short)").split()
            if b.startswith("origin/task/")]
extra = {}
for b in branches:
    d = git("diff", BASE + ".." + b, "--", "crates/tr-app/src/assets.rs")
    if not d.strip():
        continue
    strict = {m.group(1) for l in d.splitlines() if (m := row_s.match(l))}
    loose = {m.group(1) for l in d.splitlines() if (m := row_c.match(l))}
    miss = loose - strict
    if miss:
        extra[b] = sorted(miss)
print("\ncomment-row misses per branch:")
tot = set()
for b, v in extra.items():
    print(" ", b.replace("origin/task/", ""), len(v), v)
    tot |= set(v)
print("total missed names:", len(tot), "| not in new union:", sorted(tot - nn))
