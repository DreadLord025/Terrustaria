# W4-c9 merge npc.rs: enum dups + stat groups + dispatch + find_frame +
# render_key + ALL_KINDS + pins 505->508 (+3 snails).
import sys, os, re
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from _m1_rep import make_rep
p = "crates/tr-app/src/npc.rs"
src = open(p, encoding="utf8", newline="").read()
box = [src]
rep = make_rep(lambda: box[0], lambda v: box.__setitem__(0, v))
nl = "\r\n" if "\r\n" in src else "\n"

# enum: drop the c9 Shimmerfly + 5 gold dup variants (doc + variant)
for k, tid in [("Shimmerfly", "677"), ("GoldBird", "442"), ("GoldGrasshopper", "446"), ("GoldMouse", "447"), ("GoldBunny", "443"), ("GoldFrog", "445")]:
    marker = "/// `NPCID." + k + "` (" + tid
    i = box[0].index(marker)
    j = box[0].index("    " + k + ",", i) + len("    " + k + ",")
    if box[0][j:j+len(nl)] == nl: j += len(nl)
    box[0] = box[0][:i] + box[0][j:]
print("enum dups dropped")

# kb: snail group drops Shimmerfly; gold group drops whole
rep("""            NpcKind::Snail | NpcKind::GlowingSnail | NpcKind::MagmaSnail | NpcKind::Shimmerfly => 1.0,
            // The gold critters (W4-c9): the GoldBird writes kb 0.8
            // (`NPC.cs:14256`); the GoldGrasshopper/GoldMouse write none -
            // the head's 1f (`NPC.cs:8533`) stands.
            NpcKind::GoldBird => 0.8,
            NpcKind::GoldGrasshopper | NpcKind::GoldMouse | NpcKind::GoldBunny | NpcKind::GoldFrog => 1.0,
""", """            // E8 merge: Shimmerfly's kb is c7's above; the 5 gold rows
            // are d40's above (the dup drops).
            NpcKind::Snail | NpcKind::GlowingSnail | NpcKind::MagmaSnail => 1.0,
""")
# defense
rep("""            NpcKind::Snail | NpcKind::GlowingSnail | NpcKind::MagmaSnail | NpcKind::Shimmerfly => 0,
            // The gold critters (W4-c9): defense 0 each (`NPC.cs:14253` /
            // `:14309` / `:14324` / `:14268` / `:14296`).
            NpcKind::GoldBird | NpcKind::GoldGrasshopper | NpcKind::GoldMouse | NpcKind::GoldBunny | NpcKind::GoldFrog => 0,
""", """            // E8 merge: Shimmerfly's defense is c7's above; the 5 gold
            // rows are d40's above (the dup drops).
            NpcKind::Snail | NpcKind::GlowingSnail | NpcKind::MagmaSnail => 0,
""")
# value
rep("""            NpcKind::Snail | NpcKind::GlowingSnail | NpcKind::MagmaSnail | NpcKind::Shimmerfly => 0.0,
            // The gold critters (W4-c9): the rows write no value - the
            // head's 0f (`NPC.cs:8544`) stands.
            NpcKind::GoldBird | NpcKind::GoldGrasshopper | NpcKind::GoldMouse | NpcKind::GoldBunny | NpcKind::GoldFrog => 0.0,
""", """            // E8 merge: Shimmerfly's value is c7's above; the 5 gold
            // rows are d40's above (the dup drops).
            NpcKind::Snail | NpcKind::GlowingSnail | NpcKind::MagmaSnail => 0.0,
""")
open(p, "w", encoding="utf8", newline="").write(box[0])
print("c9 npc.rs part 1 done")
