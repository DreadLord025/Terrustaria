# W4 merger: conflict-hunk inspector/union resolver.
# Usage:
#   python tools/_m1_conf.py list <file>        -- print hunks with line numbers
#   python tools/_m1_conf.py show <file> <n>    -- print hunk n fully
#   python tools/_m1_conf.py union <file> [n..] -- keep BOTH sides (HEAD then theirs) for given hunks (default all)
#   python tools/_m1_conf.py head <file> <n..>  -- keep HEAD side for given hunks
#   python tools/_m1_conf.py theirs <file> <n..>-- keep theirs side for given hunks
import sys, re, io
sys.stdout.reconfigure(encoding="utf8", errors="replace")
path = sys.argv[2]
src = open(path, encoding="utf8", newline="").read()
lines = src.split("\n")
hunks = []  # (start, mid, end) 0-based indices of marker lines
i = 0
while i < len(lines):
    if lines[i].startswith("<<<<<<<"):
        s = i; m = None
        j = i + 1
        while not lines[j].startswith(">>>>>>>"):
            if lines[j].startswith("=======") and m is None: m = j
            j += 1
        hunks.append((s, m, j))
        i = j + 1
    else:
        i += 1
cmd = sys.argv[1]
if cmd == "list":
    for n, (s, m, e) in enumerate(hunks):
        print(f"hunk {n}: lines {s+1}-{e+1} (HEAD {m-s-1} ln, theirs {e-m-1} ln)")
        print("  HEAD  first:", lines[s+1].strip()[:110] if m-s>1 else "(empty)")
        print("  HEAD  last :", lines[m-1].strip()[:110] if m-s>1 else "")
        print("  THEIR first:", lines[m+1].strip()[:110] if e-m>1 else "(empty)")
        print("  THEIR last :", lines[e-1].strip()[:110] if e-m>1 else "")
elif cmd == "show":
    n = int(sys.argv[3]); s, m, e = hunks[n]
    lo = max(0, s-4); hi = min(len(lines), e+5)
    for k in range(lo, hi): print(f"{k+1}: {lines[k]}")
elif cmd in ("union", "head", "theirs"):
    sel = [int(x) for x in sys.argv[3:]] or list(range(len(hunks)))
    out = []
    i = 0; n = 0
    while i < len(lines):
        if n < len(hunks) and i == hunks[n][0]:
            s, m, e = hunks[n]
            if n in sel:
                if cmd == "union": out.extend(lines[s+1:m]); out.extend(lines[m+1:e])
                elif cmd == "head": out.extend(lines[s+1:m])
                else: out.extend(lines[m+1:e])
            else:
                out.extend(lines[s:e+1])
            i = e + 1; n += 1
        else:
            out.append(lines[i]); i += 1
    open(path, "w", encoding="utf8", newline="").write("\n".join(out))
    print(f"{cmd} applied to hunks {sel} of {len(hunks)}; remaining markers:",
          sum(1 for l in out if l.startswith("<<<<<<<")))
