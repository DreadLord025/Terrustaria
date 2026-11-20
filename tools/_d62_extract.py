"""d62 extraction: verbatim C# case bodies + Rust arm text for ids #66-130."""
import re

ROOT = r"C:\Users\DreadLord\Desktop\TR-W4-d62"
import importlib.util
spec = importlib.util.spec_from_file_location("r", ROOT + r"\tools\_d62_range.py")
m = importlib.util.module_from_spec(spec)
spec.loader.exec_module(m)

cs1 = m.cs_cases(4540, 6032)
cs2 = m.cs_cases(6049, 6509)
order = [t for t, _ in cs1] + [t for t, _ in cs2 if t not in {x for x, _ in cs1}]
first = {}
for t, l in cs1 + cs2:
    first.setdefault(t, l)
mine = order[65:130]

lines = open(ROOT + r"\decompiled\Terraria\GameContent\Drawing\TileDrawing.cs",
             encoding="utf-8-sig").read().splitlines()


def case_body(start_line):
    """From a `case N:` label, collect consecutive labels + body to break/return."""
    out = []
    i = start_line - 1
    depth = 0
    started = False
    while i < len(lines):
        ln = lines[i]
        s = ln.strip()
        out.append(f"{i+1}: {ln}")
        depth += ln.count("{") - ln.count("}")
        if re.match(r"case \d+:", s):
            i += 1
            continue
        started = True
        if depth <= 0 and (s == "break;" or s.startswith("return")):
            break
        if depth <= 0 and started and s == "}":
            break
        i += 1
    return "\n".join(out)


arms = m.drawdata_arms()
a_src = open(ROOT + r"\crates\tr-app\src\drawdata_cases_a.rs", encoding="utf-8").read()

with open(ROOT + r"\tools\_d62_extract.txt", "w", encoding="utf-8") as f:
    for idx, t in enumerate(mine, 66):
        f.write(f"\n===== #{idx} id {t} first@{first[t]} =====\n")
        f.write("--- C# first switch ---\n")
        f.write(case_body(first[t]) + "\n")
        # second-switch occurrence
        second = [l for tt, l in cs2 if tt == t]
        if second:
            f.write(f"--- C# second switch @{second[0]} ---\n")
            f.write(case_body(second[0]) + "\n")
        f.write("--- Rust drawdata.rs arms ---\n")
        for ids, txt in arms:
            if t in ids:
                f.write(txt + "\n")
        # cases_a glow arm: find `t =>` or lists containing t in glow_switch
        ga = re.search(r"fn glow_switch.*", a_src)
        for mm in re.finditer(r"(?:^|\n)\s*([\d| ]+)=>", a_src):
            if t in m.pat_ids(mm.group(1)):
                st = mm.start()
                en = a_src.find("\n        }", st)
                f.write("--- cases_a arm ---\n" + a_src[st:en + 10] + "\n")
print("done", len(mine))
