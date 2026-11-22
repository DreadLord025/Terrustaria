# Drop the c9 gated-64 axis arm (index-based, no quoting issues).
import sys
sys.stdout.reconfigure(encoding="utf8", errors="replace")
p = "crates/tr-app/src/npc.rs"
src = open(p, encoding="utf8", newline="").read()
nl = "\r\n" if "\r\n" in src else "\n"
bt = chr(96)
marker = "// " + bt + "else if (this.aiStyle == 64)" + bt
i = src.index(marker)
j = src.index("64 if self.kind == NpcKind::Shimmerfly => {", i)
k = src.index("}", src.index("self.ai_064_shimmerfly(world, player, rng)", j)) + 1
if src[k:k+len(nl)] == nl: k += len(nl)
repl = ("            // E8 merge: the c9 kind-gated 64 arm dropped - c7's plain 64" + nl +
        "            // arm below claims all four fireflies (the solo wrapper + the" + nl +
        "            // sim pool pass own the 677 threat scan; the c9 adapter retired)." + nl)
src = src[:i] + repl + src[k:]
open(p, "w", encoding="utf8", newline="").write(src)
print("gated-64 arm dropped")
