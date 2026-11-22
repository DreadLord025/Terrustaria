# Find suspicious mid-pattern joins: a line starting with `| NpcKind` (or
# `| crate::npc::NpcKind`) whose previous non-comment/non-blank line ends
# with `{`, `}`, `;` or `=> {` (i.e. not a pattern continuation).
import sys, re, glob
sys.stdout.reconfigure(encoding="utf8", errors="replace")
for f in sys.argv[1:]:
    lines = open(f, encoding="utf8").read().split("\n")
    for i, l in enumerate(lines):
        t = l.strip()
        if not (t.startswith("| NpcKind") or t.startswith("| crate::npc::NpcKind")):
            continue
        j = i - 1
        while j >= 0:
            p = lines[j].strip()
            if p and not p.startswith("//"):
                break
            j -= 1
        p = lines[j].strip() if j >= 0 else ""
        if p.endswith("{") or p.endswith("}") or p.endswith(";"):
            print(f"{f}:{i+1}: prev `{p[:80]}` then `{t[:80]}`")
