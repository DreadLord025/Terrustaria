# W4-c10 merge npc.rs part 2: kb/def/value dedupe (index-based).
import sys, re
sys.stdout.reconfigure(encoding="utf8", errors="replace")
p = "crates/tr-app/src/npc.rs"
src = open(p, encoding="utf8", newline="").read()
nl = "\r\n" if "\r\n" in src else "\n"
lines = src.split(nl)

def find(sub, start=0):
    return next(k for k, l in enumerate(lines) if sub in l and k >= start)

# value: the 12-critter tail + the waterfowl Seagull/Grebe
i = find("// W4-c10 roster tail — the twelve critters write no value row")
j = find("| NpcKind::Buggy => 0.0,", i)
lines[i:j+1] = ["            // E8 merge: the c10 12-critter value tail dropped (d40/c8's rows)."]
i = find("| NpcKind::Grebe2 => 0.0,")
k = i
while "NpcKind::Duck" not in lines[k]:
    k -= 1
lines[k:i+1] = ["            // E8 merge: Seagull/Grebe's value rows are d42's above.",
                "            NpcKind::Duck",
                "            | NpcKind::Duck2",
                "            | NpcKind::DuckWhite",
                "            | NpcKind::DuckWhite2 => 0.0,"]

# defense: the 12-critter tail + the waterfowl Seagull/Grebe
i = find("// W4-c10 roster tail — the twelve critters' `defense = 0` rows")
j = find("| NpcKind::Buggy => 0,", i)
lines[i:j+1] = ["            // E8 merge: the c10 12-critter defense tail dropped (d40/c8's rows)."]
i = find("| NpcKind::Grebe2 => 0,")
k = i
while "NpcKind::Duck" not in lines[k]:
    k -= 1
lines[k:i+1] = ["            // E8 merge: Seagull/Grebe's defense rows are d42's above.",
                "            NpcKind::Duck",
                "            | NpcKind::Duck2",
                "            | NpcKind::DuckWhite",
                "            | NpcKind::DuckWhite2 => 0,"]

# kb: the 12-critter tail + the waterfowl Seagull/Grebe
i = find("// W4-c10 roster tail — the GoldBird's `knockBackResist = 0.8f`")
j = find("| NpcKind::Buggy => 1.0,", i)
lines[i:j+1] = ["            // E8 merge: the c10 12-critter kb tail dropped (d40/c8's rows)."]
i = find("| NpcKind::Grebe2 => 1.0,")
k = i
while "NpcKind::Duck" not in lines[k]:
    k -= 1
lines[k:i+1] = ["            // E8 merge: Seagull/Grebe's kb rows are d42's above.",
                "            NpcKind::Duck",
                "            | NpcKind::Duck2",
                "            | NpcKind::DuckWhite",
                "            | NpcKind::DuckWhite2 => 1.0,"]

src = nl.join(lines)
open(p, "w", encoding="utf8", newline="").write(src)
print("kb/def/value dedupe done")
