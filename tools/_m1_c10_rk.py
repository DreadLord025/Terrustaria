# W4-c10 merge npc.rs render_key + ALL_KINDS + pins + retire update_gold_bird.
import sys, re
sys.stdout.reconfigure(encoding="utf8", errors="replace")
p = "crates/tr-app/src/npc.rs"
src = open(p, encoding="utf8", newline="").read()
nl = "\r\n" if "\r\n" in src else "\n"
lines = src.split(nl)

def find(sub, start=0):
    return next(k for k, l in enumerate(lines) if sub in l and k >= start)

# render_key: the Duck quartet renumbers 731-734; the Seagull/Grebe + the
# 12-critter tail drop (d42/d40/c8's keys above own them).
i = find("// W4-c10 — the waterfowl family's vanilla ids (362-365/602/")
j = find("NpcKind::Grebe2 => 609,", i)
lines[i:j+1] = ["            // E8 merge renumber: the Duck quartet rides 731-734 (the",
                "            // vanilla 362-365 keys are d37's above); the Seagull/Grebe",
                "            // keys are d42's above (the dup drops).",
                "            NpcKind::Duck => 731,",
                "            NpcKind::Duck2 => 732,",
                "            NpcKind::DuckWhite => 733,",
                "            NpcKind::DuckWhite2 => 734,"]
i = find("// W4-c10 roster tail — the twelve critters' vanilla ids (357/")
j = find("NpcKind::Buggy => 487,", i)
lines[i:j+1] = ["            // E8 merge: the c10 12-critter render_key tail dropped (d40/c8's keys)."]

src = nl.join(lines)

# ALL_KINDS decl join + rows + pins
# find the decl
m = re.search(r"pub\(super\) const ALL_KINDS: \[NpcKind; 508\] = \[" + re.escape(nl) + r"\s*pub\(super\) const ALL_KINDS: \[NpcKind; \d+\] = \[", src)
if m:
    src = src[:m.start()] + "    // + W4-c10's 4 waterfowl (Duck 731 / Duck2 732 / DuckWhite 733 /" + nl + "    // DuckWhite2 734 - the d37-table renumber; the Seagull/Grebe pairs +" + nl + "    // the 12-critter tail dropped - d42/d40/c8's rows own them) = 512." + nl + "    pub(super) const ALL_KINDS: [NpcKind; 512] = [" + src[m.end():]
    print("ALL_KINDS decl joined")
else:
    print("DECL JOIN NOT FOUND - check")

# ALL_KINDS rows: the c10 insert
lines = src.split(nl)
i = find("// W4-c10")
# find the c10 ALL_KINDS rows block
idxs = [k for k, l in enumerate(lines) if "W4-c10" in l and k > 30000]
for k in idxs:
    print("ALL_KINDS c10 comment at", k+1, lines[k].strip()[:80])
src = nl.join(lines)
open(p, "w", encoding="utf8", newline="").write(src)
print("render_key done")
