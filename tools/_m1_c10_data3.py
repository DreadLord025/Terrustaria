# W4-c10 merge data.rs part 3: index-based dedupe of the c10 dup blocks.
import sys, re
sys.stdout.reconfigure(encoding="utf8", errors="replace")
p = "crates/tr-app/src/npc/data.rs"
src = open(p, encoding="utf8", newline="").read()
nl = "\r\n" if "\r\n" in src else "\n"
lines = src.split(nl)

def drop_block(start_idx, end_idx, replacement):
    """Drop lines[start_idx:end_idx] (0-based), insert replacement lines."""
    global lines
    lines = lines[:start_idx] + replacement + lines[end_idx:]

# Work bottom-up so indices stay valid.
# 1. ai_style 68 waterfowl swimmers: drop Seagull2/Grebe2 (d42's above)
i = next(k for k, l in enumerate(lines) if "NpcKind::Duck2 | NpcKind::DuckWhite2 | NpcKind::Seagull2 | NpcKind::Grebe2 => 68," in l)
lines[i] = "            // E8 merge: Seagull2/Grebe2's 68 rows are d42's above." + nl + "            NpcKind::Duck2 | NpcKind::DuckWhite2 => 68,"

# 2. ai_style the 12-critter tail (the c10 block at ~3768-3784)
i = next(k for k, l in enumerate(lines) if "// aiStyle 7 — W4-c10 roster tail, the TownCritter golds:" in l)
j = next(k for k, l in enumerate(lines) if k > i and "| NpcKind::Buggy => 66," in l)
drop_block(i, j+1, ["            // E8 merge: the c10 12-critter aiStyle tail dropped - d40/c8's", "            // rows above own them (the dup drops; the wiring composes)."])

# 3. ai_style 7 waterfowl walkers: drop Seagull/Grebe (d42's above)
i = next(k for k, l in enumerate(lines) if "NpcKind::Duck | NpcKind::DuckWhite | NpcKind::Seagull | NpcKind::Grebe => 7," in l)
lines[i] = "            // E8 merge: Seagull/Grebe's 7 rows are d42's above." + nl + "            NpcKind::Duck | NpcKind::DuckWhite => 7,"

# 4. ai_style 1 GoldGrasshopper dup
i = next(k for k, l in enumerate(lines) if "// aiStyle 1 — W4-c10 roster tail: the GoldGrasshopper 446" in l)
drop_block(i, i+3, ["            // E8 merge: the c10 GoldGrasshopper aiStyle-1 dup dropped (d40's)."])

src = nl.join(lines)
open(p, "w", encoding="utf8", newline="").write(src)
print("ai_style dedupe done")
