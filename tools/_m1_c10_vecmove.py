import sys, re
sys.stdout.reconfigure(encoding="utf8", errors="replace")
p = "crates/tr-app/src/assets.rs"
src = open(p, encoding="utf8", newline="").read()
nl = "\r\n" if "\r\n" in src else "\n"
bt = chr(96)
# extract the c10 waterfowl vec block (comment + 8 rows)
marker = "// W4-c10 — the waterfowl family's eight 15-row strips."
i = src.index(marker)
end_row = 'atlas.sheet("NPC_609").copied(),'
j = src.index(end_row, i) + len(end_row)
if src[j:j+len(nl)] == nl: j += len(nl)
block = src[i:j]
src = src[:i] + src[j:]
# append after the c9 snails (655)
anchor = 'atlas.sheet("NPC_655").copied(),'
k = src.index(anchor) + len(anchor)
if src[k:k+len(nl)] == nl: k += len(nl)
src = src[:k] + block + src[k:]
open(p, "w", encoding="utf8", newline="").write(src)
print("waterfowl vec rows moved to end")
