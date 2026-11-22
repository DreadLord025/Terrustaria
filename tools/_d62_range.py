"""d62 range census: case ids #66-130 in C# order, classified vs the port.

Order = first-switch label order (TileDrawing.cs:4540-6032), then
second-switch-only ids (:6049-6509). d61 owns #1-65, d62 #66-130,
d63 #131-195, d64 the rest (#196-371 + the 215 epilogue).
"""
import re

ROOT = r"C:\Users\DreadLord\Desktop\TR-W4-d62"


def cs_cases(lo, hi):
    lines = open(ROOT + r"\decompiled\Terraria\GameContent\Drawing\TileDrawing.cs",
                 encoding="utf-8-sig").read().splitlines()
    ids, seen = [], set()
    for i in range(lo - 1, hi):
        m = re.match(r"^(\t+)case (\d+):", lines[i])
        if m and len(m.group(1)) == 2 and int(m.group(2)) not in seen:
            seen.add(int(m.group(2)))
            ids.append((int(m.group(2)), i + 1))
    return ids


def pat_ids(text):
    ids = set()
    for m in re.finditer(r"(\d+)\.\.=(\d+)", text):
        ids.update(range(int(m.group(1)), int(m.group(2)) + 1))
    t2 = re.sub(r"\d+\.\.=\d+", " ", text)
    for m in re.finditer(r"\b(\d+)\b", t2):
        ids.add(int(m.group(1)))
    return ids


def drawdata_arms():
    """(pattern_ids, has_deviation_marker, arm_text) per top-level arm,
    with leading comment blocks attached."""
    src = open(ROOT + r"\crates\tr-core\src\drawdata.rs", encoding="utf-8").read()
    raw = src[src.index("match kind {"):src.index("// Campfire epilogue")]
    lines = raw.splitlines()
    pending, pat, arm_lines = [], [], []
    depth = 0
    in_arm = False
    out = []
    for ln in lines:
        s = ln.strip()
        if not in_arm:
            if s.startswith("//"):
                pending.append(s)
                continue
            if "=>" in s:
                pat.append(s.split("=>")[0])
                arm_lines = pending + [s]
                pending = []
                if "{" in s:
                    in_arm = True
                    depth = s.count("{") - s.count("}")
                    if depth == 0:
                        out.append((pat_ids(" ".join(pat)), "\n".join(arm_lines)))
                        pat, arm_lines, in_arm = [], [], False
                else:
                    out.append((pat_ids(" ".join(pat)), "\n".join(arm_lines)))
                    pat, arm_lines = [], []
            elif re.match(r"^[\d|.\s=]+$", s) or s.startswith("|"):
                pat.append(s)
            else:
                pat, pending = [], []
        else:
            arm_lines.append(s)
            depth += ln.count("{") - ln.count("}")
            if depth <= 0:
                out.append((pat_ids(" ".join(pat)), "\n".join(arm_lines)))
                pat, arm_lines, in_arm, depth = [], [], False, 0
    return out


def module_ids(path):
    s = open(ROOT + "\\" + path, encoding="utf-8").read()
    s = re.sub(r"//[^\n]*", "", s)
    ids = set()
    for m in re.finditer(r"\bkind == (\d+)", s):
        ids.add(int(m.group(1)))
    for m in re.finditer(r"matches!\(\s*(?:kind|k)\s*,([^)]*)\)", s):
        ids |= pat_ids(m.group(1))
    for m in re.finditer(r"match\s+(?:kind|base_kind|k)\b[^{;]*\{", s):
        depth = 0
        j = m.end() - 1
        while j < len(s):
            if s[j] == "{":
                depth += 1
            elif s[j] == "}":
                depth -= 1
                if depth == 0:
                    break
            j += 1
        ids |= pat_ids(s[m.end():j])
    return ids


def main():
    cs1 = cs_cases(4540, 6032)
    cs2 = cs_cases(6049, 6509)
    first = {}
    for t, l in cs1 + cs2:
        first.setdefault(t, l)
    order = [t for t, _ in cs1] + [t for t, _ in cs2 if t not in {x for x, _ in cs1}]
    arms = drawdata_arms()
    a = module_ids(r"crates\tr-app\src\drawdata_cases_a.rs")
    b = module_ids(r"crates\tr-app\src\drawdata_cases_b.rs")
    f = module_ids(r"crates\tr-core\src\drawdata_furniture.rs")
    mine = order[65:130]
    print(f"total {len(order)}; d62 range #66-130: {mine[0]}@{first[mine[0]]} .. {mine[-1]}@{first[mine[-1]]}")
    for i, t in enumerate(mine, 66):
        hits = [(ids, txt) for ids, txt in arms if t in ids]
        cls = []
        for ids, txt in hits:
            dev = bool(re.search(r"deviation|MISSING|not ported", txt))
            cls.append("STATIC-DEV" if dev else "FULL")
        if t in a:
            cls.append("cases_a")
        if t in b:
            cls.append("cases_b")
        if t in f:
            cls.append("furniture")
        if not cls:
            cls = ["NONE"]
        print(f"{i:3d}. {t:3d}@{first[t]}  {'+'.join(cls)}")


if __name__ == "__main__":
    main()
