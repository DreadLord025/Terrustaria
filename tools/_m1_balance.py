# Proper-ish Rust brace balance: skips // and /* */ comments, "strings",
# 'c' chars and lifetimes; reports the first line where depth goes wrong.
import sys, re
sys.stdout.reconfigure(encoding="utf8", errors="replace")
p = sys.argv[1]
src = open(p, encoding="utf8").read()
i = 0; n = len(src); depth = 0; line = 1
stack = []
while i < n:
    c = src[i]
    if c == "\n": line += 1; i += 1; continue
    if src.startswith("//", i):
        j = src.find("\n", i); i = n if j < 0 else j; continue
    if src.startswith("/*", i):
        j = src.find("*/", i+2)
        line += src[i:j].count("\n")
        i = n if j < 0 else j+2; continue
    if c == '"':
        i += 1
        while i < n and src[i] != '"':
            if src[i] == "\\": i += 1
            if src[i] == "\n": line += 1
            i += 1
        i += 1; continue
    if c == "'":
        # char literal vs lifetime: 'x' or '\x'
        m = re.match(r"'(\\.|[^\\'])'", src[i:])
        if m: i += len(m.group(0)); continue
        i += 1; continue
    if c in "{([":
        stack.append((c, line)); depth += 1
    elif c in "})]":
        if not stack:
            print(f"EXTRA CLOSE {c} at line {line}"); sys.exit(1)
        o, ol = stack.pop()
        if "({[".index(o) != ")}]".index(c):
            print(f"MISMATCH {o} (line {ol}) closed by {c} at line {line}"); sys.exit(1)
        depth -= 1
    i += 1
if stack:
    print("UNCLOSED:", stack[-5:])
else:
    print("BALANCED")
