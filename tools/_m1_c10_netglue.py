# W4-c10 merge netglue: the waterfowl group keeps the Duck quartet (the
# Seagull/Grebe ride the d42 cave_walker/hover arms above); the 12-critter
# seed tail drops (d40's seeds own them).
import sys, re
sys.stdout.reconfigure(encoding="utf8", errors="replace")
p = "crates/tr-app/src/app/netglue.rs"
src = open(p, encoding="utf8", newline="").read()
nl = "\r\n" if "\r\n" in src else "\n"
lines = src.split(nl)

def find(sub, start=0):
    return next(k for k, l in enumerate(lines) if sub in l and k >= start)

# the c10 waterfowl group: the Duck quartet + Seagull/Grebe. The d42
# Seagull/Grebe arms above are the live ones (first-match). Drop the
# Seagull/Grebe members from the c10 group.
i = find("// W4-c10 — the waterfowl family (362-365/602/603/608/609): the")
j = find("| crate::npc::NpcKind::Grebe2 => crate::npc::Npc::new(kind, _x, y, 1, 0.0),", i)
lines[i:j+1] = [
    "        // W4-c10 - the waterfowl family: the Duck quartet's bare-stats",
    "        // seed (the net state overwrites the position below; the",
    "        // machines take over on the first tick). E8 merge: the",
    "        // Seagull/Grebe seeds ride the d42 cave_walker/hover arms above.",
    "        crate::npc::NpcKind::Duck",
    "        | crate::npc::NpcKind::Duck2",
    "        | crate::npc::NpcKind::DuckWhite",
    "        | crate::npc::NpcKind::DuckWhite2 => crate::npc::Npc::new(kind, _x, y, 1, 0.0),",
]
# the c10 12-critter seed tail drops
i = find("// W4-c10 roster tail — the twelve critters (357/442-448/484-487):")
j = find("| crate::npc::NpcKind::Buggy => crate::npc::Npc::new(kind, _x, y, 1, 0.0),", i)
lines[i:j+1] = ["        // E8 merge: the c10 12-critter seed tail dropped (d40/c8's seeds above)."]

src = nl.join(lines)
open(p, "w", encoding="utf8", newline="").write(src)
print("netglue dedupe done")
