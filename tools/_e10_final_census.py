# W4-e10 FINAL census re-sweep: every SoundEngine.PlaySound site in Player.cs
# vs the port's citation comments (crates/**/*.rs). Code-only, no build.
# Usage: python tools/_e10_final_census.py [--json out.json]
import io, json, os, re, sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
PLAYER_CS = os.path.join(ROOT, "decompiled", "Terraria", "Player.cs")
CRATES = os.path.join(ROOT, "crates")

# --- 1. the sites -----------------------------------------------------------
sites = []  # (line, call text)
with io.open(PLAYER_CS, "r", encoding="utf-8-sig", errors="replace") as f:
    for n, line in enumerate(f, 1):
        if "SoundEngine.PlaySound(" in line or "PlayTrackedSound(" in line:
            sites.append((n, line.strip()))
print("Player.cs PlaySound sites:", len(sites))

# --- 2. the citations -------------------------------------------------------
# Full form: Player.cs:NNNN or Player.cs:NNNN-NNNN.
# Short form: `:NNNN` / `:NNNN-NNNN` resolved against the last *.cs file named
# in the same comment neighbourhood (a rolling last-file state per source file;
# a different .cs name resets it). Only Player.cs-resolved citations count.
full_re = re.compile(r"([A-Za-z0-9_]+\.cs):(\d+)(?:-(\d+))?")
short_re = re.compile(r"(?<![A-Za-z0-9_.]):(\d{3,5})(?:-(\d{3,5}))?")

claims = {}  # site line -> list of (rust file, rust line, cite text)
def add_claim(a, b, src, ln, text):
    for (site, _) in sites:
        if a <= site <= b:
            claims.setdefault(site, []).append((src, ln, text))

for dirpath, dirnames, filenames in os.walk(CRATES):
    dirnames[:] = [d for d in dirnames if d not in ("target",)]
    for fn in filenames:
        if not fn.endswith(".rs"):
            continue
        path = os.path.join(dirpath, fn)
        rel = os.path.relpath(path, ROOT)
        last_cs = None
        with io.open(path, "r", encoding="utf-8", errors="replace") as f:
            raw = f.read().splitlines()
        # Group consecutive comment lines into blocks so wrapped citations
        # (`Player.cs:25225-` / `// 25232`) resolve as one span. Inline
        # trailing comments join the block stream too (their code prefix is
        # stripped; a code line between two comment lines breaks the block).
        blocks = []  # (start line, joined text)
        cur, cur_start = [], None
        for ln, line in enumerate(raw, 1):
            stripped = line.lstrip()
            is_comment = (stripped.startswith("//") or stripped.startswith("///")
                          or stripped.startswith("*") or stripped.startswith("/*"))
            if not is_comment and "//" in line:
                line = line[line.index("//"):]
                is_comment = True
            if is_comment:
                if cur_start is None:
                    cur_start = ln
                cur.append(line)
            else:
                if cur:
                    blocks.append((cur_start, " ".join(cur)))
                    cur, cur_start = [], None
        if cur:
            blocks.append((cur_start, " ".join(cur)))
        for ln, text in blocks:
            # a wrapped range tail: `:25225- // 25232` -> `:25225- 25232`;
            # normalise `- <digits>` to `-<digits>` before matching
            text = re.sub(r"-\s+(\d)", r"-\1", text)
            # Per-block context: a short cite resolves against the most recent
            # full *.cs name INSIDE the same comment block. Cross-block
            # inheritance is a false-positive source (a `Projectile.Update
            # (:16668-...)` doc after a `Player.cs:134-140` mention), so a
            # block with no full cite claims nothing; those sites surface in
            # the unclaimed list for manual verification instead.
            block_cs = None
            events = []
            for m in full_re.finditer(text):
                events.append((m.start(), "full", m))
            for m in short_re.finditer(text):
                # skip short matches inside a full match's span
                if any(f.start() <= m.start() < f.end() for f in full_re.finditer(text)):
                    continue
                events.append((m.start(), "short", m))
            events.sort(key=lambda e: e[0])
            for _, kind, m in events:
                if kind == "full":
                    block_cs = m.group(1)
                    if m.group(1) == "Player.cs":
                        a = int(m.group(2)); b = int(m.group(3) or m.group(2))
                        add_claim(a, b, rel, ln, m.group(0))
                elif block_cs == "Player.cs":
                    a = int(m.group(1)); b = int(m.group(2) or m.group(1))
                    if 1 <= a <= 60000:
                        add_claim(a, b, rel, ln, ":" + m.group(1))

claimed = sorted(claims)
unclaimed = [(n, t) for (n, t) in sites if n not in claims]
print("claimed sites:", len(claimed), " unclaimed:", len(unclaimed))
print()
for n, t in unclaimed:
    print(f"{n}: {t}")

if "--json" in sys.argv:
    out = sys.argv[sys.argv.index("--json") + 1]
    with io.open(out, "w", encoding="utf-8") as f:
        json.dump({"sites": sites, "claims": claims,
                   "unclaimed": unclaimed}, f, indent=1)
    print("wrote", out)
