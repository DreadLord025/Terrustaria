import sys, re
sys.stdout.reconfigure(encoding="utf8", errors="replace")
p = sys.argv[1]
marks = set(int(x) for x in sys.argv[2:])
src = open(p, encoding="utf8").read()
depth = 0; line = 1; n = len(src); j = 0
while j < n:
    c = src[j]
    if c == "\n":
        if line in marks: print("line", line, "depth", depth)
        line += 1; j += 1; continue
    if src.startswith("//", j):
        k = src.find("\n", j); j = n if k < 0 else k; continue
    if src.startswith("/*", j):
        k = src.find("*/", j+2); line += src[j:k].count("\n"); j = n if k < 0 else k+2; continue
    if c == '"':
        j += 1
        while j < n and src[j] != '"':
            if src[j] == "\\": j += 1
            if src[j] == "\n": line += 1
            j += 1
        j += 1; continue
    if c == "'":
        m = re.match(r"'(\\.|[^\\'])'", src[j:])
        if m: j += len(m.group(0)); continue
        j += 1; continue
    if c in "{([": depth += 1
    elif c in "})]": depth -= 1
    j += 1
print("final depth", depth)
