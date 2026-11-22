# W4-c10 merge data.rs part 4: size/cs_id/max_hp/contact_damage dedupe.
import sys, re
sys.stdout.reconfigure(encoding="utf8", errors="replace")
p = "crates/tr-app/src/npc/data.rs"
src = open(p, encoding="utf8", newline="").read()
nl = "\r\n" if "\r\n" in src else "\n"
lines = src.split(nl)

def find(sub, start=0):
    return next(k for k, l in enumerate(lines) if sub in l and k >= start)

# work bottom-up
# contact_damage: the 12-critter tail + the waterfowl Seagull/Grebe
i = find("// W4-c10 roster tail — the twelve critters' `damage = 0` rows")
j = find("| NpcKind::Buggy => 0,", i)
lines[i:j+1] = ["            // E8 merge: the c10 12-critter damage tail dropped (d40/c8's rows)."]
i = find("NpcKind::Duck")
# the waterfowl damage group
i = find("| NpcKind::Grebe2 => 0,")
# back up to the NpcKind::Duck start
k = i
while "NpcKind::Duck" not in lines[k]:
    k -= 1
lines[k:i+1] = ["            // E8 merge: Seagull/Grebe's damage rows are d42's above.",
                "            NpcKind::Duck",
                "            | NpcKind::Duck2",
                "            | NpcKind::DuckWhite",
                "            | NpcKind::DuckWhite2 => 0,"]

# max_hp: the 12-critter tail + the waterfowl Seagull/Grebe
i = find("// W4-c10 roster tail — the twelve critters' `lifeMax = 5` rows")
j = find("| NpcKind::Buggy => 5,", i)
lines[i:j+1] = ["            // E8 merge: the c10 12-critter lifeMax tail dropped (d40/c8's rows)."]
i = find("| NpcKind::Grebe2 => 5,")
k = i
while "NpcKind::Duck" not in lines[k]:
    k -= 1
lines[k:i+1] = ["            // E8 merge: Seagull/Grebe's lifeMax rows are d42's above.",
                "            NpcKind::Duck",
                "            | NpcKind::Duck2",
                "            | NpcKind::DuckWhite",
                "            | NpcKind::DuckWhite2 => 5,"]

# cs_id: the 12-critter tail + the waterfowl Seagull/Grebe (the Duck quartet keeps its vanilla hand rows)
i = find("// W4-c10 roster tail — the twelve critters' own bestiary")
j = find("NpcKind::Buggy => 487,", i)
lines[i:j+1] = ["            // E8 merge: the c10 12-critter cs_id tail dropped (d40/c8's rows)."]
i = find("NpcKind::Seagull => 602,")
j = find("NpcKind::Grebe2 => 609,", i)
lines[i:j+1] = ["            // E8 merge: Seagull/Grebe's cs_id rows are d42's above."]

# size: the 12-critter tail + the waterfowl Seagull/Grebe
i = find("// W4-c10 roster tail — the critter boxes:")
j = find("NpcKind::GoldMouse => (14.0, 12.0),", i)
lines[i:j+1] = ["            // E8 merge: the c10 12-critter size tail dropped (d40/c8's rows)."]
i = find("NpcKind::Duck | NpcKind::DuckWhite | NpcKind::Seagull | NpcKind::Grebe => {")
lines[i] = "            // E8 merge: Seagull/Grebe's size rows are d42's above." + nl + "            NpcKind::Duck | NpcKind::DuckWhite => {"
i = find("NpcKind::Duck2 | NpcKind::DuckWhite2 | NpcKind::Seagull2 | NpcKind::Grebe2 => {")
lines[i] = "            NpcKind::Duck2 | NpcKind::DuckWhite2 => {"

src = nl.join(lines)
open(p, "w", encoding="utf8", newline="").write(src)
print("size/cs_id/max_hp/contact_damage dedupe done")
