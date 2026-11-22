# Move the c9 walk countdown+exit to the HEAD of the walk block (the C#
# :53815-53831 order: countdown + exit BEFORE the ladder/hop/stuck-flip).
import sys, re
sys.stdout.reconfigure(encoding="utf8", errors="replace")
p = "crates/tr-app/src/npc_ai_critter.rs"
src = open(p, encoding="utf8", newline="").read()
nl = "\r\n" if "\r\n" in src else "\n"
# extract the countdown+exit block (from the :53815 comment to the exit close)
start_marker = "        // `:53815` - the walk countdown (W4-c9 merge"
i = src.index(start_marker)
# the block ends at the exit's closing "        }" before the :54226 comment
j = src.index("        // `:54226-54242`", i)
block = src[i:j]
# remove it from its current position
src = src[:i] + src[j:]
# insert at the head of the walk block (right after the `} else if npc.ai[0] == 1.0 {`)
k = src.index("} else if npc.ai[0] == 1.0 {") + len("} else if npc.ai[0] == 1.0 {") + len(nl)
src = src[:k] + block + src[k:]
open(p, "w", encoding="utf8", newline="").write(src)
print("countdown moved to head")
