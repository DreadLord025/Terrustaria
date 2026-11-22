# Restructure the interleaved c9/c10 threat-scan pre-passes: the c9
# Shimmerfly bare block + the c10 GoldButterfly if-block were merged into
# one. Separate them (the c9 pre-pass stamps the Shimmerfly views, the
# c10 pre-pass stamps the GoldButterfly views - both read the same pool).
import sys, re, subprocess
sys.stdout.reconfigure(encoding="utf8", errors="replace")
p = "crates/tr-app/src/world_sim/npcs.rs"
src = open(p, encoding="utf8", newline="").read()
nl = "\r\n" if "\r\n" in src else "\n"
head = subprocess.run(["git","show","HEAD:crates/tr-app/src/world_sim/npcs.rs"],capture_output=True).stdout.decode("utf8","replace")
# extract the c9 Shimmerfly pre-pass block from HEAD (the bare block)
i = head.index("// W4-c9")
# find the shimmerfly pre-pass comment
m = re.search(r"[^\n]*shimmerfly_threat_views[^\n]*", head)
# the block: from the comment before the '{' to the closing '}'
k = head.index("shimmerfly_threat_views = views.clone()")
# back up to the block start '{'
bs = head.rfind("{", 0, head.rfind("let views", 0, k))
# find the block comment start
cs = head.rfind("//", 0, bs)
cs = head.rfind("\n", 0, cs) + 1
# the block end: the '}' after the for-loop
be = head.index("}", head.index("}", k) + 1) + 1
c9_block = head[cs:be].replace("\r\n", "\n").replace("\n", nl)
print("c9 block extracted:", len(c9_block), "chars")
# the current merged block: from the '{' before the c10 comment to the '}' at 4678
i2 = src.index("// W4-c10 roster tail — the GoldButterfly's threat-scan")
bs2 = src.rfind("{", 0, i2)
# the block comment start (the c9 comment)
cs2 = src.rfind("//", 0, bs2)
cs2 = src.rfind("\n", 0, cs2) + 1
# back up further to the full c9 comment block
while True:
    prev = src.rfind("\n", 0, cs2-1)
    line = src[prev+1:cs2-1]
    if line.strip().startswith("//"):
        cs2 = prev+1
    else:
        break
be2 = src.index("                }" + nl + "                // The single session player", i2)
be2 += len("                }")
# replace the merged block with: c9 block + c10 block
# the c10 block: the if-gated GoldButterfly pre-pass
c10_block = ("                // W4-c10 roster tail — the GoldButterfly's threat-scan" + nl +
             "                // pool (`NPC.cs:45071-45079`): the same tick-start pool" + nl +
             "                // read (the c9 Shimmerfly pre-pass's shape)." + nl +
             "                if npcs.iter().any(|n| n.kind == NpcKind::GoldButterfly && n.hp > 0) {" + nl +
             "                    let views: Vec<crate::npc_ai_style64::Ai64NpcView> = npcs" + nl +
             "                        .iter()" + nl +
             "                        .map(|m| {" + nl +
             "                            let (mw, mh) = m.size();" + nl +
             "                            crate::npc_ai_style64::Ai64NpcView {" + nl +
             "                                active: m.hp > 0," + nl +
             "                                damage: m.damage_override.unwrap_or(m.kind.contact_damage())," + nl +
             "                                friendly: m.friendly," + nl +
             "                                x: m.x," + nl +
             "                                y: m.y," + nl +
             "                                width: mw as i32," + nl +
             "                                height: mh as i32," + nl +
             "                            }" + nl +
             "                        })" + nl +
             "                        .collect();" + nl +
             "                    for fi in 0..npcs.len() {" + nl +
             "                        if npcs[fi].kind == NpcKind::GoldButterfly && npcs[fi].hp > 0 {" + nl +
             "                            npcs[fi].butterfly_threat_views = views.clone();" + nl +
             "                        }" + nl +
             "                    }" + nl +
             "                }")
src = src[:cs2] + c9_block + nl + c10_block + src[be2:]
open(p, "w", encoding="utf8", newline="").write(src)
print("pre-passes restructured")
