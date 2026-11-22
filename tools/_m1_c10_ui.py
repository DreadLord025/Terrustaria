# W4-c10 merge ui/bestiary.rs: the waterfowl group keeps the Duck quartet
# (the Seagull/Grebe ride the d42 portrait arms above); the 12-critter
# portrait tail drops (d40/c8's arms own them).
import sys, re
sys.stdout.reconfigure(encoding="utf8", errors="replace")
p = "crates/tr-app/src/ui/bestiary.rs"
src = open(p, encoding="utf8", newline="").read()
nl = "\r\n" if "\r\n" in src else "\n"
lines = src.split(nl)

def find(sub, start=0):
    return next(k for k, l in enumerate(lines) if sub in l and k >= start)

i = find("(NpcKind::Duck | NpcKind::DuckWhite | NpcKind::Seagull | NpcKind::Grebe, _) => {")
lines[i] = "        // E8 merge: Seagull/Grebe ride the d42 portrait arms above." + nl + "        (NpcKind::Duck | NpcKind::DuckWhite, _) => {"
i = find("(NpcKind::Duck2 | NpcKind::DuckWhite2 | NpcKind::Seagull2 | NpcKind::Grebe2, _) => {")
lines[i] = "        (NpcKind::Duck2 | NpcKind::DuckWhite2, _) => {"
i = find("// W4-c10 roster tail — the twelve critters' portraits:")
j = find("(NpcKind::GoldMouse, _) => {", i)
j = find("}", j+1)
j = find("}", j)
lines[i:j+1] = ["        // E8 merge: the c10 12-critter portrait tail dropped (d40/c8's arms above)."]
src = nl.join(lines)
open(p, "w", encoding="utf8", newline="").write(src)
print("ui dedupe done")
