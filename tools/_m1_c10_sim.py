import sys, re
sys.stdout.reconfigure(encoding="utf8", errors="replace")
p = "crates/tr-app/src/world_sim/npcs.rs"
src = open(p, encoding="utf8", newline="").read()
nl = "\r\n" if "\r\n" in src else "\n"
# fix the interleaved for-loop (the Shimmerfly if missing its close)
old = ("                    for fi in 0..npcs.len() {" + nl +
       "                        if npcs[fi].kind == NpcKind::Shimmerfly && npcs[fi].hp > 0 {" + nl +
       "                            npcs[fi].shimmerfly_threat_views = views.clone();" + nl +
       "                        if npcs[fi].kind == NpcKind::GoldButterfly && npcs[fi].hp > 0 {" + nl +
       "                            npcs[fi].butterfly_threat_views = views.clone();" + nl +
       "                        }" + nl +
       "                    }")
new = ("                    for fi in 0..npcs.len() {" + nl +
       "                        if npcs[fi].kind == NpcKind::Shimmerfly && npcs[fi].hp > 0 {" + nl +
       "                            npcs[fi].shimmerfly_threat_views = views.clone();" + nl +
       "                        }" + nl +
       "                        if npcs[fi].kind == NpcKind::GoldButterfly && npcs[fi].hp > 0 {" + nl +
       "                            npcs[fi].butterfly_threat_views = views.clone();" + nl +
       "                        }" + nl +
       "                    }")
assert old in src, "for-loop not found"
src = src.replace(old, new)
# the c10 block's opening '{' - check the region before it
i = src.index("// W4-c10 roster tail — the GoldButterfly's threat-scan")
print(repr(src[i-350:i]))
open(p, "w", encoding="utf8", newline="").write(src)
print("for-loop fixed")
