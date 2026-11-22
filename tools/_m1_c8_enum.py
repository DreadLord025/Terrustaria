# W4-c8 merge: drop the 5 dup enum variants + the GoldWorm ALL_KINDS dup row.
import sys, re
sys.stdout.reconfigure(encoding="utf8", errors="replace")
p = "crates/tr-app/src/npc.rs"
src = open(p, encoding="utf8", newline="").read()
nl = "\r\n" if "\r\n" in src else "\n"
for k, tid in [("GoldWorm", "448"), ("EnchantedNightcrawler", "484"), ("Grubby", "485"), ("Sluggy", "486"), ("Buggy", "487")]:
    marker = "/// `NPCID." + k + "` (" + tid
    i = src.index(marker)
    j = src.index("    " + k + ",", i) + len("    " + k + ",")
    if src[j:j+len(nl)] == nl: j += len(nl)
    src = src[:i] + src[j:]
# ALL_KINDS: the c8 insert's GoldWorm row is a dup of d40's row - drop the
# c8 one (the one right after TruffleWormDigger in the c8 block)
i = src.index("// The crawling-critter family (W4-c8")
j = src.index("NpcKind::GoldWorm,", i)
# verify it's within the c8 block (before the d40 comment)
k = src.index("// W4-d40:", i)
assert j < k, "GoldWorm row not in the c8 block"
src = src[:j] + src[j+len("NpcKind::GoldWorm,")+len(nl):]
open(p, "w", encoding="utf8", newline="").write(src)
print("enum + ALL_KINDS dups dropped")
