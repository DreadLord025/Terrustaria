import re, sys
sys.stdout.reconfigure(encoding="utf8", errors="replace")
path = sys.argv[1]
src = open(path, encoding='utf8').read()
fns = [(src[:m.start()].count('\n')+1, m.group(1)) for m in re.finditer(r'^\s*(?:pub(?:\([^)]*\))?\s+)?fn\s+(\w+)', src, re.M)]
for ln in [int(x) for x in sys.argv[2:]]:
    prev = [f for f in fns if f[0] <= ln][-1]
    print(ln, '->', prev)
