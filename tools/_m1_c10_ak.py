# W4-c10 merge npc.rs ALL_KINDS rows dedupe + pins 508->512.
import sys, re
sys.stdout.reconfigure(encoding="utf8", errors="replace")
p = "crates/tr-app/src/npc.rs"
src = open(p, encoding="utf8", newline="").read()
nl = "\r\n" if "\r\n" in src else "\n"
lines = src.split(nl)

def find(sub, start=0):
    return next(k for k, l in enumerate(lines) if sub in l and k >= start)

# 1. The c10 waterfowl ALL_KINDS rows: keep the Duck quartet, drop the
#    Seagull/Grebe pairs (d42's rows above).
i = find("// W4-c10 — the waterfowl family (the enum's declaration order).")
j = find("NpcKind::Grebe2,", i)
lines[i:j+1] = ["        // W4-c10 - the waterfowl family (the Duck quartet; the Seagull/",
                "        // Grebe pairs dropped - d42's rows above own them).",
                "        NpcKind::Duck,",
                "        NpcKind::Duck2,",
                "        NpcKind::DuckWhite,",
                "        NpcKind::DuckWhite2,"]

# 2. The c10 12-critter ALL_KINDS rows (the first block: Worm..GoldGrasshopper)
i = find("// W4-c10 roster tail — the twelve critters (declaration order).")
j = find("NpcKind::GoldGrasshopper,", i)
lines[i:j+1] = ["        // E8 merge: the c10 12-critter ALL_KINDS tail dropped (d40/c8's rows)."]

# 3. The c10 12-critter ALL_KINDS rows (the second block: GoldMouse..Buggy
#    at the array tail)
i = find("NpcKind::GoldMouse,", find("NpcKind::MagmaSnail,"))
j = find("NpcKind::Buggy,", i)
lines[i:j+1] = []

src = nl.join(lines)
# pins 508 -> 512
n = src.count("assert_eq!(ALL_KINDS.len(), 508);")
src = src.replace("assert_eq!(ALL_KINDS.len(), 508);", "assert_eq!(ALL_KINDS.len(), 512);")
print(f"ALL_KINDS.len pins re-pinned: {n}")
# s1_f_3 pin
src = src.replace("            508," + nl + '            "every decodable kind resolves (the wire roster\'s size - 505 (spine 488 + c3\'s 694 + c4\'s 8 + c7\'s 4 + c8\'s 4) + W4-c9\'s 3 snails (Snail 740 / GlowingSnail 741 renumbered per the d37 tables, MagmaSnail 655 keep; the Shimmerfly + 5 gold dups dropped - c7/d40 own them), L-B gate)"',
                  "            512," + nl + '            "every decodable kind resolves (the wire roster\'s size - 508 (spine 488 + c3\'s 694 + c4\'s 8 + c7\'s 4 + c8\'s 4 + c9\'s 3) + W4-c10\'s 4 waterfowl (Duck 731 / Duck2 732 / DuckWhite 733 / DuckWhite2 734 renumbered per the d37 tables; the Seagull/Grebe pairs + the 12-critter tail dropped - d42/d40/c8 own them), L-B gate)"')
open(p, "w", encoding="utf8", newline="").write(src)
print("ALL_KINDS rows + pins done")
