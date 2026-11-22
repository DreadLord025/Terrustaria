# c10 critter hunk: keep the c9 countdown receipt, drop the stale :54257
# tail, keep the c10 frog wet tail (outside the walk block).
import sys
sys.stdout.reconfigure(encoding="utf8", errors="replace")
p = "crates/tr-app/src/npc_ai_critter.rs"
src = open(p, encoding="utf8", newline="").read()
nl = "\r\n" if "\r\n" in src else "\n"
i = src.index("<<<<<<< HEAD")
mid = src.index("=======", i)
end = src.index(">>>>>>> origin/task/W4-c10-style68-waterfowl", mid)
head_side = src[i+len("<<<<<<< HEAD")+len(nl):mid]
theirs_side = src[mid+len("=======")+len(nl):end]
# theirs: the stale countdown block + the frog wet tail. Split at the
# walk-block close: the countdown ends with "        }\n    }\n" then the
# frog tail comment starts.
marker = "    // W4-c10"
j = theirs_side.index(marker)
frog_tail = theirs_side[j:]
# head_side is the c9 receipt comment (3 lines). Keep it + close the walk
# block + the frog tail.
resolved = head_side + "    }" + nl + frog_tail
src = src[:i] + resolved + src[end+len(">>>>>>> origin/task/W4-c10-style68-waterfowl")+len(nl):]
open(p, "w", encoding="utf8", newline="").write(src)
print("critter hunk resolved")
