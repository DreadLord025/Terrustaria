# W4-c10 merge npc.rs find_frame: repair the d42 Seagull/Seagull2 mid-arm
# joins (the c10 waterfowl arms are the superset - they cover the Duck
# quartet + the Seagull/Grebe pairs with identical bodies). Keep the c10
# superset arms, drop the d42 standalone Seagull/Seagull2 arms (their
# bodies are identical to the c10 group's). Also drop the c10 12-critter
# find_frame dups (the c8/c9 keepers own them).
import sys, re
sys.stdout.reconfigure(encoding="utf8", errors="replace")
p = "crates/tr-app/src/npc.rs"
src = open(p, encoding="utf8", newline="").read()
nl = "\r\n" if "\r\n" in src else "\n"
lines = src.split(nl)

def find(sub, start=0):
    return next(k for k, l in enumerate(lines) if sub in l and k >= start)

# 1. The d42 Seagull standalone arm: the c10 group arm follows it (the
#    mid-arm join). Drop the d42 arm header (the comment + `NpcKind::Seagull => {`)
#    so the c10 group arm stands alone.
i = find("// W4-d42 - `FindFrame` case 602/608")
j = find("NpcKind::Duck | NpcKind::DuckWhite | NpcKind::Grebe | NpcKind::Seagull => {", i)
# drop lines[i:j] (the d42 comment + the orphan Seagull header) but keep
# the c10 comment that starts at i+3
# Actually the c10 comment is between. Find the c10 comment start.
c10_start = find("// W4-c10 — `case 362 : case 364 : case 608`", i)
lines[i:c10_start] = []  # drop the d42 comment + orphan header

# 2. The d42 Seagull2 standalone arm: same.
i = find("// W4-d42 - `FindFrame` case 603/609")
c10_start = find("// W4-c10 — `case 363 : case 365 : case 609`", i)
lines[i:c10_start] = []

# 3. The c10 waterfowl walker group: drop Grebe/Seagull? NO - the c10
#    group IS the superset (the Duck quartet + the Seagull/Grebe pairs,
#    identical bodies). Keep it whole. The d42 standalone arms are gone.

# 4. The c10 12-critter find_frame dups: drop them (the c8/c9 keepers own).
#    The crawler arms (Worm/GoldWorm/EnchantedNightcrawler + Grubby/Sluggy/Buggy)
i = find("// W4-c10 roster tail — `case 357 : case 448 : case 484 :")
j = find("// W4-c10 roster tail — `case 74 : case 297 : case 298 :", i)
lines[i:j] = ["            // E8 merge: the c10 crawler find_frame dups dropped - c8's",
              "            // crawler groups above own them (the superset module)."]

# 5. The GoldBird find_frame dup (c9's keeper owns it)
i = find("// W4-c10 roster tail — `case 74 : case 297 : case 298 :")
j = find("// W4-c10 roster tail — `case 443` (the bunny group", i)
lines[i:j] = ["            // E8 merge: the c10 GoldBird find_frame dup dropped - c9's",
              "            // case-442 arm above owns it."]

# 6. The GoldBunny find_frame dup (c9's keeper)
i = find("// W4-c10 roster tail — `case 443` (the bunny group")
j = find("// W4-c10 roster tail — `case 356 : case 444 : case 653 :", i)
lines[i:j] = ["            // E8 merge: the c10 GoldBunny find_frame dup dropped - c9's",
              "            // case-443 arm above owns it."]

# 7. The GoldButterfly find_frame: c10's is the ONLY one (d40's was a
#    row-0 stand-in, dropped at the c9 merge). KEEP c10's.
# 8. The GoldFrog find_frame dup (c9's keeper)
i = find("// W4-c10 roster tail — `case 445 : case 687`")
j = find("// W4-c10 roster tail — `case 446` (`NPC.cs:70959-70980`)", i)
lines[i:j] = ["            // E8 merge: the c10 GoldFrog find_frame dup dropped - c9's",
              "            // case-445 arm above owns it."]

# 9. The GoldGrasshopper + GoldMouse find_frame dups (c9's keepers)
i = find("// W4-c10 roster tail — `case 446` (`NPC.cs:70959-70980`)")
# find the end of the GoldMouse arm (the next non-c10 comment or arm)
j = i
# the GoldMouse arm ends before the next W4 comment or the match close
# find the closing of the GoldMouse arm
k = find("NpcKind::GoldMouse => {", i)
# find its closing brace
depth = 0
m = k
while True:
    if "{" in lines[m]: depth += lines[m].count("{")
    if "}" in lines[m]: depth -= lines[m].count("}")
    if depth == 0 and m > k:
        break
    m += 1
lines[i:m+1] = ["            // E8 merge: the c10 GoldGrasshopper/GoldMouse find_frame dups",
                "            // dropped - c9's case-446/447 arms above own them."]

src = nl.join(lines)
open(p, "w", encoding="utf8", newline="").write(src)
print("find_frame dedupe done")
