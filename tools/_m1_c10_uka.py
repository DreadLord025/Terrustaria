# W4-c10 merge npc.rs update_kind_ai: drop the d42 Seagull/Grebe unwired
# stand-ins (the c10 waterfowl superset arms own them), the c10 12-critter
# tail dups (c8/c9/d40 own), the c9 GoldBird dispatch (the c10 axis-24
# unreachable arm is correct now). Keep the c9 GoldGrasshopper faithful
# machine over the c10 update_slime stand-in.
import sys, re
sys.stdout.reconfigure(encoding="utf8", errors="replace")
p = "crates/tr-app/src/npc.rs"
src = open(p, encoding="utf8", newline="").read()
nl = "\r\n" if "\r\n" in src else "\n"
lines = src.split(nl)

def find(sub, start=0):
    return next(k for k, l in enumerate(lines) if sub in l and k >= start)

# 1. The c9 GoldBird dispatch arm -> the c10 axis-24 unreachable arm is
#    the keeper. Drop the c9 arm (update_gold_bird wrapper becomes dead).
i = find("NpcKind::GoldBird => self.update_gold_bird(world, player, rng),")
lines[i] = "            // E8 merge: the GoldBird rides the c10 axis-24 unreachable arm" + nl + "            // above (ai_024_bird owns it - the c9 update_gold_bird wrapper retired)."

# 2. The c10 GoldGrasshopper update_slime stand-in -> keep the c9
#    update_gold_grasshopper faithful machine. Drop the c10 arm.
i = find("NpcKind::GoldGrasshopper => self.update_slime(world, player, dt),")
# find the c10 comment above it
k = i
while not lines[k].strip().startswith("// W4-c10 roster tail — the GoldGrasshopper"):
    k -= 1
lines[k:i+1] = ["            // E8 merge: the c10 GoldGrasshopper update_slime stand-in dropped",
                "            // - the c9 update_gold_grasshopper faithful machine below owns it."]

# 3. The c10 TownCritter golds arm (GoldBunny/GoldFrog/GoldMouse) - the c9
#    arm below is the keeper (same critter_ai call). Drop the c10 dup.
i = find("// W4-c10 roster tail — the TownCritter golds (443/445/447,")
j = find("crate::npc_ai_critter::critter_ai(self, world, player, rng);", i)
j = find("}", j)  # the arm close
lines[i:j+1] = ["            // E8 merge: the c10 TownCritter golds dup dropped - the c9 arm",
                "            // below owns it (the same critter_ai call)."]

# 4. The d40 GoldButterfly unwired stand-in drops (the c10 axis-65
#    unreachable arm owns it).
i = find("NpcKind::GoldButterfly => {}")
lines[i] = "            // E8 merge: the GoldButterfly rides the c10 axis-65 unreachable arm above."

# 5. The c10 12-critter tail unreachable arms (Worm/GoldWorm/EnchantedNightcrawler/Grubby/Sluggy/Buggy)
i = find("// W4-c10 roster tail — the aiStyle-66 crawlers (357/448/484-487):")
j = find('unreachable!("aiStyle 66 rides the style axis (W4-c10 roster tail)")', i)
j = find("}", j)
lines[i:j+1] = ["            // E8 merge: the c10 crawler unreachable dups dropped - c8's axis-66",
                "            // unreachable arm above owns them."]

# 6. The d42 Seagull/Grebe unwired stand-ins drop (the c10 waterfowl arms own them)
for kind in ["Grebe", "Grebe2", "Seagull", "Seagull2"]:
    i = find("NpcKind::" + kind + " => {}")
    # back up to the comment
    k = i
    while not lines[k].strip().startswith("//"):
        k -= 1
    lines[k:i+1] = ["            // E8 merge: the d42 " + kind + " unwired stand-in dropped - the",
                    "            // c10 waterfowl arm above owns it."]

src = nl.join(lines)
open(p, "w", encoding="utf8", newline="").write(src)
print("update_kind_ai dedupe done")
