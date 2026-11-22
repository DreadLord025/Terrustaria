# W4-d63 census: depth-1 outer case labels for both GetTileDrawData switches.
import re, io

SRC = r"C:\Users\DreadLord\Desktop\TR-W4-d63\decompiled\Terraria\GameContent\Drawing\TileDrawing.cs"
lines = io.open(SRC, encoding="utf-8-sig").read().splitlines()

def outer_cases(hdr_line, hard_end):
    depth = 0
    out = []
    for i in range(hdr_line - 1, hard_end):
        ln = lines[i]
        stripped = ln.strip()
        m = re.match(r"case (\d+):", stripped)
        if m and depth == 1:
            out.append((int(m.group(1)), i + 1))
        depth += ln.count("{") - ln.count("}")
        if depth == 0 and i > hdr_line - 1:
            return out, i + 1
    return out, hard_end

first, end1 = outer_cases(4540, 6048)
second, end2 = outer_cases(6049, 6600)
print("first switch: %d labels, ends line %d" % (len(first), end1))
print("second switch: %d labels, ends line %d" % (len(second), end2))

seen = set()
union = []
for cid, ln in first:
    if cid not in seen:
        seen.add(cid)
        union.append((cid, ln, 1))
first_ids = set(c for c, _ in first)
second_only = []
for cid, ln in second:
    if cid not in seen:
        seen.add(cid)
        union.append((cid, ln, 2))
        second_only.append((cid, ln))
print("distinct first ids:", len(first_ids))
print("second-only ids:", second_only)
print("union:", len(union))

# d63 quarter [186, 279)
q = union[186:279]
print("\nd63 quarter [186,279): %d ids" % len(q))
for idx, (cid, ln, sw) in enumerate(q, start=186):
    print("%3d. %d@%d%s" % (idx, cid, ln, " (2nd)" if sw == 2 else ""))

# boundaries for cross-check with d64 doc
print("\nindex 278 (last d63):", union[278])
print("index 279 (first d64):", union[279])
print("index 185 (last d62):", union[185])
print("index 186 (first d63):", union[186])
print("index 92/93:", union[92], union[93])

with io.open(r"C:\Users\DreadLord\Desktop\TR-W4-d63\tools\_d63_union.txt", "w", encoding="utf-8") as f:
    for idx, (cid, ln, sw) in enumerate(union):
        f.write("%d %d %d %d\n" % (idx, cid, ln, sw))
