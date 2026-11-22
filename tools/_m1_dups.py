# W4 merger: dup-arm detector across match functions in a file (kind-keyed).
# Finds `NpcKind::X ... =>` arms appearing 2+ times within the same fn body.
import sys, re, collections
sys.stdout.reconfigure(encoding="utf8", errors="replace")
path = sys.argv[1]
kinds = sys.argv[2:] if len(sys.argv) > 2 else None
src = open(path, encoding="utf8").read()
lines = src.split("\n")
# locate fn boundaries (top-level `fn name` at any indent, track braces)
fns = []
for m in re.finditer(r"^(\s*)(?:pub(?:\([^)]*\))?\s+)?(?:const\s+)?fn\s+(\w+)", src, re.M):
    fns.append((m.start(), m.group(2)))
fns.append((len(src), "<eof>"))
for i in range(len(fns)-1):
    a, name = fns[i]; b = fns[i+1][0]
    body = src[a:b]
    arms = collections.defaultdict(list)
    for m in re.finditer(r"NpcKind::(\w+)(?:\s*\|\s*NpcKind::(\w+))*\s*=>", body):
        for k in re.findall(r"NpcKind::(\w+)", m.group(0)):
            ln = src[:a+m.start()].count("\n") + 1
            arms[k].append(ln)
    for k, lns in sorted(arms.items()):
        if len(lns) > 1 and (kinds is None or k in kinds):
            print(f"{name}: {k} x{len(lns)} at lines {lns}")
