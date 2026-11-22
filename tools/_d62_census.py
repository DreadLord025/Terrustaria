"""d62 census: GetTileDrawData case ids in C# order vs the port.

Classifies each C# case id:
  FULL   - a draw_data arm with no deviation comment (static part complete
           at t=0; dynamic rows ride the wired animator)
  STATIC - a draw_data arm carrying a deviation comment (only the
           unconditional static part is ported)
  DELEG  - covered by a delegated pass (cases_a / cases_b / furniture)
  NONE   - no arm anywhere (falls to the 16x16 default)

C# order = first-switch label order (:4540-6032), then second-switch-only
ids (:6049-6509), then the campfire epilogue 215 (:6033-6044).
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


def arm_spans(body):
    """Yield (pattern_text, body_text) for each top-level match arm."""
    depth = 0
    pat = []
    buf = []
    in_arm = False
    for ln in body.splitlines():
        s = ln.strip()
        if depth == 0 and not in_arm:
            if "=>" in s:
                pat.append(s.split("=>")[0])
                buf = [s.split("=>", 1)[1]]
                if "{" in s:
                    in_arm = True
                    depth = s.count("{") - s.count("}")
                    if depth == 0:
                        yield " ".join(pat), " ".join(buf)
                        pat, buf, in_arm = [], [], False
                else:
                    # single-line arm
                    yield " ".join(pat), " ".join(buf)
                    pat, buf = [], []
            elif re.match(r"^[\d|.\s=]+$", s) or s.startswith("|"):
                pat.append(s)
            else:
                pat = []
        elif in_arm:
            buf.append(s)
            depth += ln.count("{") - ln.count("}")
            if depth <= 0:
                yield " ".join(pat), " ".join(buf)
                pat, buf, in_arm, depth = [], [], False, 0
    if pat and buf:
        yield " ".join(pat), " ".join(buf)


def pat_ids(text):
    ids = set()
    for m in re.finditer(r"(\d+)\.\.=(\d+)", text):
        ids.update(range(int(m.group(1)), int(m.group(2)) + 1))
    t2 = re.sub(r"\d+\.\.=\d+", " ", text)
    for m in re.finditer(r"\b(\d+)\b", t2):
        ids.add(int(m.group(1)))
    return ids


def main():
    cs1 = cs_cases(4540, 6032)
    cs2 = cs_cases(6049, 6509)
    first = {}
    for t, l in cs1 + cs2:
        first.setdefault(t, l)
    order = [t for t, _ in cs1] + [t for t, _ in cs2 if t not in {x for x, _ in cs1}]
    order.append(215)
    first.setdefault(215, 6033)

    src = open(ROOT + r"\crates\tr-core\src\drawdata.rs", encoding="utf-8").read()
    body = src[src.index("match kind {"):src.index("// Campfire epilogue")]
    full, static = set(), set()
    for pat, abody in arm_spans(body):
        ids = pat_ids(pat)
        # deviation markers live in the arm's leading comment lines too;
        # re-scan with comments kept for the marker check
        if re.search(r"deviation|MISSING|not ported", abody):
            static |= ids
        else:
            full |= ids
    # comments precede arms; redo with raw body to attach leading comments
    raw = body
    full, static = set(), set()
    chunks = re.split(r"\n(?=\s*(?:\d|//))", raw)
    # simpler: walk arms on the RAW body (comments kept inside arm spans)
    for pat, abody in arm_spans(raw):
        ids = pat_ids(pat)
        if re.search(r"deviation|MISSING|not ported", abody):
            static |= ids
        else:
            full |= ids
    # leading comments: attach each comment block to the following arm
    lines = raw.splitlines()
    pending = []
    depth = 0
    in_arm = False
    arm_lines = []
    pat = []
    results = []
    for ln in lines:
        s = ln.strip()
        if not in_arm:
            if s.startswith("//"):
                pending.append(s)
                continue
            if depth == 0:
                if "=>" in s:
                    pat.append(s.split("=>")[0])
                    arm_lines = pending + [s]
                    pending = []
                    if "{" in s:
                        in_arm = True
                        depth = s.count("{") - s.count("}")
                        if depth == 0:
                            results.append((" ".join(pat), "\n".join(arm_lines)))
                            pat, arm_lines, in_arm = [], [], False
                    else:
                        results.append((" ".join(pat), "\n".join(arm_lines)))
                        pat, arm_lines = [], []
                elif re.match(r"^[\d|.\s=]+$", s) or s.startswith("|"):
                    pat.append(s)
                else:
                    pat = []
                    pending = []
        else:
            arm_lines.append(s)
            depth += ln.count("{") - ln.count("}")
            if depth <= 0:
                results.append((" ".join(pat), "\n".join(arm_lines)))
                pat, arm_lines, in_arm, depth = [], [], False, 0
    full, static = set(), set()
    for pat, text in results:
        ids = pat_ids(pat)
        if re.search(r"deviation|MISSING|not ported", text):
            static |= ids
        else:
            full |= ids
    full -= static  # deviation wins if both (range overlap)

    def module_ids(path):
        s = open(ROOT + "\\" + path, encoding="utf-8").read()
        s = re.sub(r"//[^\n]*", "", s)
        ids = set()
        for m in re.finditer(r"\bkind == (\d+)", s):
            ids.add(int(m.group(1)))
        for m in re.finditer(r"matches!\(\s*(?:kind|k)\s*,([^)]*)\)", s):
            ids |= pat_ids(m.group(1))
        for m in re.finditer(r"match\s+(?:kind|base_kind|k)\b[^{;]*\{", s):
            # crude balanced take
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

    a = module_ids(r"crates\tr-app\src\drawdata_cases_a.rs")
    b = module_ids(r"crates\tr-app\src\drawdata_cases_b.rs")
    f = module_ids(r"crates\tr-core\src\drawdata_furniture.rs")
    deleg = (a | b | f) - full - static

    cs_set = set(order)
    print(f"C# union ids: {len(order)} (first {len(cs1)}, second-only "
          f"{len(order) - len(cs1) - 1}, +215 epilogue)")
    print(f"FULL {len(full & cs_set)}  STATIC {len(static & cs_set)}  "
          f"DELEG {len(deleg & cs_set)}  NONE {len(cs_set - full - static - deleg)}")
    print("\nSTATIC (deviation) ids in C# order:")
    st = [t for t in order if t in static]
    for i, t in enumerate(st, 1):
        print(f"{i:3d}. {t}@{first[t]}")
    print("\nDELEG ids in C# order:")
    dg = [t for t in order if t in deleg]
    for i, t in enumerate(dg, 1):
        print(f"{i:3d}. {t}@{first[t]}")
    nn = [t for t in order if t not in full | static | deleg]
    print("\nNONE ids:", nn)
    gaps = [t for t in order if t in static or t in deleg or t not in full]
    print(f"\nTOTAL non-FULL (STATIC+DELEG+NONE): {len(gaps)}")
    for i, t in enumerate(gaps, 1):
        cls = "STATIC" if t in static else ("DELEG" if t in deleg else "NONE")
        print(f"{i:3d}. {t}@{first[t]} {cls}")


if __name__ == "__main__":
    main()
