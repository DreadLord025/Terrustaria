# W4-c10 merge assets.rs: slots + vec + STATIC_SHEET_NAMES dedupe.
import sys, re
sys.stdout.reconfigure(encoding="utf8", errors="replace")
p = "crates/tr-app/src/assets.rs"
src = open(p, encoding="utf8", newline="").read()
nl = "\r\n" if "\r\n" in src else "\n"
lines = src.split(nl)

def find(sub, start=0):
    return next(k for k, l in enumerate(lines) if sub in l and k >= start)

# 1. STATIC_SHEET_NAMES: the c10 waterfowl 8 keep (362-365 new; 602/603/
#    608/609 new - d42's sheets were deferred, NOT in STATIC_SHEET_NAMES).
#    The c10 12-critter tail: 357 dup drops (c8's row), 442-448/454/462/
#    463/464/469/470/472/482/484-487 are d40's rows above (dup drop).
i = find("// W4-c10 roster tail — the twelve critters' own sheets")
j = find('"NPC_482",', i)
lines[i:j+1] = ["        // E8 merge: the c10 12-critter sheet tail dropped (d40/c8's rows above)."]

# 2. slots: the waterfowl 8 keep, the 12-critter tail drops
i = find("// W4-c10 roster tail — the twelve critters' sheets, appended")
j = find("357, 442, 443, 444, 445, 446, 447, 448, 484, 485, 486, 487,", i)
lines[i:j+1] = ["        // E8 merge: the c10 12-critter slot tail dropped (d40/c8's slots above)."]
# slots decl join: 320 + 8 = 328
i = find("pub const BESTIARY_NPC_SHEET_IDS: [u16; 320] = [")
j = find("pub const BESTIARY_NPC_SHEET_IDS: [u16;", i+1)
lines[i:j+1] = ["// + W4-c10's 8 waterfowl sheets (362-365/602/603/608/609; the",
                "// 12-critter tail dropped - d40/c8's slots above own them) = 328.",
                "pub const BESTIARY_NPC_SHEET_IDS: [u16; 328] = ["]

# 3. vec: the waterfowl 8 keep, the 12-critter tail drops (the d40 band
#    re-push), the 357 dup drops.
i = find("// W4-c10 roster tail — the twelve critters' strips")
j = find('atlas.sheet("NPC_487").copied(),', i)
lines[i:j+1] = ["                // E8 merge: the c10 12-critter vec tail dropped (d40/c8's rows above)."]

src = nl.join(lines)
open(p, "w", encoding="utf8", newline="").write(src)
print("assets dedupe done")
