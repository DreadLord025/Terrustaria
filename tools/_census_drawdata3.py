"""Census v3: full GetTileDrawData (both switches) vs all port modules.

Brace-balanced extraction of every `match kind`/`match k` body in each port
module, plus `kind == N` and `matches!(kind, ...)` guards.
"""
import re

ROOT = r"C:\Users\DreadLord\Desktop\TR-W4-d61"

def cs_case_ids(lo, hi, indent):
    lines = open(ROOT + r"\decompiled\Terraria\GameContent\Drawing\TileDrawing.cs",
                 encoding="utf-8-sig").read().splitlines()
    ids, seen = [], set()
    for i in range(lo - 1, hi):
        m = re.match(r"^(\t+)case (\d+):", lines[i])
        if m and len(m.group(1)) == indent and int(m.group(2)) not in seen:
            seen.add(int(m.group(2)))
            ids.append((int(m.group(2)), i + 1))
    return ids

def strip_comments(src):
    src = re.sub(r"//[^\n]*", "", src)
    src = re.sub(r"/\*.*?\*/", "", src, flags=re.S)
    return src

def balanced(src, open_idx):
    """src[open_idx] == '{'; return body up to matching '}'"""
    depth = 0
    for j in range(open_idx, len(src)):
        if src[j] == '{':
            depth += 1
        elif src[j] == '}':
            depth -= 1
            if depth == 0:
                return src[open_idx + 1:j]
    raise ValueError("unbalanced")

def pattern_ids(text):
    """ids in match-arm pattern position; nested matches excluded by
    only taking patterns at the TOP brace depth of this body."""
    ids = set()
    depth = 0
    pat = []
    for ln in text.splitlines():
        s = ln.strip()
        # track depth roughly per line for pattern collection
        if depth == 0 and s:
            if "=>" in s:
                pat.append(s.split("=>")[0])
                t = " ".join(pat)
                for m in re.finditer(r"(\d+)\.\.=(\d+)", t):
                    ids.update(range(int(m.group(1)), int(m.group(2)) + 1))
                t2 = re.sub(r"\d+\.\.=\d+", " ", t)
                for m in re.finditer(r"\b(\d+)\b", t2):
                    ids.add(int(m.group(1)))
                pat = []
            elif re.match(r"^[\d|.\s=]+$", s):
                pat.append(s)
            else:
                pat = []
        depth += ln.count('{') - ln.count('}')
        if depth < 0:
            depth = 0
    return ids

def module_ids(path):
    src = strip_comments(open(ROOT + "\\" + path, encoding="utf-8").read())
    ids = set()
    for m in re.finditer(r"match\s+(?:kind|base_kind|k|tile_type|t)\b[^{;]*\{", src):
        ids |= pattern_ids(balanced(src, m.end() - 1))
    for m in re.finditer(r"\bkind == (\d+)", src):
        ids.add(int(m.group(1)))
    for m in re.finditer(r"matches!\(\s*(?:kind|k)\s*,([^)]*)\)", src):
        t = m.group(1)
        for r in re.finditer(r"(\d+)\.\.=(\d+)", t):
            ids.update(range(int(r.group(1)), int(r.group(2)) + 1))
        t2 = re.sub(r"\d+\.\.=\d+", " ", t)
        for r in re.finditer(r"\b(\d+)\b", t2):
            ids.add(int(r.group(1)))
    return ids

def main():
    cs1 = cs_case_ids(4540, 6032, 2)
    cs2 = cs_case_ids(6049, 6510, 2)
    all_cs = {}
    for t, l in cs1 + cs2:
        all_cs.setdefault(t, l)
    order = [t for t, _ in cs1] + [t for t, _ in cs2 if t not in {x for x, _ in cs1}]
    print(f"first switch: {len(cs1)}; second switch: {len(cs2)}; union: {len(all_cs)} (+215 = {len(all_cs)+1})")

    dd = module_ids(r"crates\tr-core\src\drawdata.rs") | {215}
    a = module_ids(r"crates\tr-app\src\drawdata_cases_a.rs")
    b = module_ids(r"crates\tr-app\src\drawdata_cases_b.rs")
    f = module_ids(r"crates\tr-core\src\drawdata_furniture.rs")
    cs_set = set(order)
    print("dd ids in cs:", len(dd & cs_set), " a:", sorted(a & cs_set))
    print("b:", sorted(b & cs_set))
    print("f count in cs:", len(f & cs_set))
    covered = (dd | a | b | f) & cs_set
    missing = [t for t in order if t not in covered]
    print(f"\ncovered: {len(covered)}/{len(order)}")
    print(f"MISSING ({len(missing)}), C# order:")
    for i, t in enumerate(missing, 1):
        print(f"{i:3d}. {t}@{all_cs[t]}")
    dd_miss = [t for t in order if t not in dd]
    print(f"\nnot in draw_data alone ({len(dd_miss)}):")
    print(dd_miss)

if __name__ == "__main__":
    main()
