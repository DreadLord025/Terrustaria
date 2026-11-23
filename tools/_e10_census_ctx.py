# context dump for unclaimed census sites
import io, json, os, sys
ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
data = json.load(io.open(sys.argv[1], encoding="utf-8"))
lines = io.open(os.path.join(ROOT, "decompiled", "Terraria", "Player.cs"),
                encoding="utf-8-sig", errors="replace").read().splitlines()
# find enclosing method: walk back to nearest method signature at indent 1
import re
sig = re.compile(r"^\t(public|private|protected|internal).*\(")
for n, t in data["unclaimed"]:
    # enclosing method
    m = "?"
    for i in range(n - 1, max(0, n - 3000), -1):
        if sig.match(lines[i - 1]):
            m = lines[i - 1].strip()
            break
    print(f"=== {n}  [{m}]")
    for i in range(max(1, n - 6), min(len(lines), n + 3)):
        print(f"{i}: {lines[i-1]}")
    print()
