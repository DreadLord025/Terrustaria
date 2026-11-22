# W4-d37 E8-SEAL phase: sweep ALL origin task/W4-* refs for kind-claimants
# at CURRENT tips (post-SEAL movers), then census every claimant + local
# probes. Git-show / git-diff only, direct paths, no junction recursion.
import re, subprocess, json

REPO = r"C:\Users\DreadLord\Desktop\TR-W4-d37"

def git(*a):
    r = subprocess.run(["git", "-C", REPO] + list(a), capture_output=True)
    return r.returncode, r.stdout.decode("utf8", "replace")

def show(rev, path):
    rc, out = git("show", f"{rev}:{path}")
    return out if rc == 0 else None

DATA = "crates/tr-app/src/npc/data.rs"
NPC = "crates/tr-app/src/npc.rs"

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

def arms_from(src):
    return {int(m[0]): m[1] for m in
            re.findall(r"(\d+)\s*=>\s*NpcKind::(\w+)", fn_body(src, "pub fn from_u16"))}

base_src = show("a725acb04", DATA)
base_names = set(arms_to(base_src))

# --- sweep every origin task/W4-* ref: does its data.rs carry NEW kind arms?
_, refs = git("for-each-ref", "--format=%(refname:short)", "refs/remotes/origin/task/W4-*")
claimants = {}
changed = []
for ref in refs.split():
    rc, out = git("diff", "--name-only", "a725acb04", ref, "--", DATA)
    if not out.strip():
        continue
    changed.append(ref)
    src = show(ref, DATA)
    if src is None: continue
    try:
        to = arms_to(src)
    except ValueError:
        continue
    new = {k: v for k, v in to.items() if k not in base_names}
    if new:
        claimants[ref] = new
print(f"refs swept: {len(refs.split())}, data.rs-changed: {len(changed)}, "
      f"kind-claimants: {len(claimants)}")
for ref, new in sorted(claimants.items()):
    ids = sorted(new.values())
    print(f"{ref.replace('origin/task/W4-',''):42s} +{len(new):3d} "
          f"ids {ids[0]}-{ids[-1]}")
json.dump(claimants, open(REPO + r"\tools\_d37_e8s_sweep.json", "w"),
          indent=1, sort_keys=True)
