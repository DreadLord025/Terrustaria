# W4-c10 merge npc.rs: enum dups drop (the 12-critter tail + the
# Seagull/Grebe pairs), the Duck quartet keeps. Then the stat groups.
import sys, re
sys.stdout.reconfigure(encoding="utf8", errors="replace")
p = "crates/tr-app/src/npc.rs"
src = open(p, encoding="utf8", newline="").read()
nl = "\r\n" if "\r\n" in src else "\n"
lines = src.split(nl)

# The c10 enum insert spans from the Duck doc to the Buggy variant (the
# second occurrence of each dup). Drop the Seagull/Grebe pairs + the
# 12-critter tail (Worm..Buggy), keep the Duck quartet.
# Find the c10 Seagull doc start (the second Seagull variant)
seagull_idxs = [k for k, l in enumerate(lines) if l.strip() == "Seagull,"]
i = seagull_idxs[-1]
# back up to the doc start
k = i
while not lines[k].strip().startswith("/// `NPCID.Seagull`"):
    k -= 1
# the c10 Buggy variant (the second one)
buggy_idxs = [k for k, l in enumerate(lines) if l.strip() == "Buggy,"]
j = buggy_idxs[-1]
# drop lines[k:j+1] (the Seagull doc through the Buggy variant)
repl = ["    // W4-c10 merge-side E8 drops (d37 tables): the Seagull/Grebe pairs",
        "    // (d42's variants above) + the 12-critter tail (Worm -> c8@737, the",
        "    // gold septet + bait tail -> d40's variants above) dropped; the Duck",
        "    // quartet keeps (renumbered 731-734 in to_u16)."]
lines = lines[:k] + repl + lines[j+1:]
src = nl.join(lines)
open(p, "w", encoding="utf8", newline="").write(src)
print("enum dups dropped")
