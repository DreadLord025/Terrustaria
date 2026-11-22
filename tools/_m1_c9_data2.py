# W4-c9 merge data.rs part 2: size/cs_id/max_hp/contact_damage + the
# snail ai_style group (find it first).
import sys, os, re
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from _m1_rep import make_rep
p = "crates/tr-app/src/npc/data.rs"
src = open(p, encoding="utf8", newline="").read()
box = [src]
rep = make_rep(lambda: box[0], lambda v: box.__setitem__(0, v))

# size: snail group drops Shimmerfly; gold group drops whole
rep("""            NpcKind::Snail => (12.0, 12.0),
            NpcKind::GlowingSnail | NpcKind::MagmaSnail => (14.0, 14.0),
            NpcKind::Shimmerfly => (10.0, 10.0),
            // The gold critters (W4-c9): 442 14x14 (`NPC.cs:14249-14250`),
            // 446 14x10 (`:14305-14306`), 447 14x12 (`:14320-14321`).
            NpcKind::GoldBird => (14.0, 14.0),
            NpcKind::GoldGrasshopper => (14.0, 10.0),
            NpcKind::GoldMouse => (14.0, 12.0),
            // The GoldBunny 18x20 (`NPC.cs:14264-14265`), the GoldFrog
            // 12x10 (`:14292-14293`).
            NpcKind::GoldBunny => (18.0, 20.0),
            NpcKind::GoldFrog => (12.0, 10.0),
""", """            // E8 merge: Shimmerfly's size is c7's above; the 5 gold
            // sizes are d40's above (the dup drops).
            NpcKind::Snail => (12.0, 12.0),
            NpcKind::GlowingSnail | NpcKind::MagmaSnail => (14.0, 14.0),
""")
# cs_id: snail trio keeps (vanilla hand rows), Shimmerfly + golds drop
rep("""            NpcKind::Snail => 359,
            NpcKind::GlowingSnail => 360,
            NpcKind::MagmaSnail => 655,
            NpcKind::Shimmerfly => 677,
            // The gold critters (W4-c9): the vanilla `NPC.type` ids - the
            // catch/convert tables + the IsGoldCritter sparkle gate read
            // them (`NPCID.cs:11557`/`:11565`/`:11567`). All three have
            // their own bestiary entries (the populator's
            // `AddEmptyEntries_CrittersAndEnemies_Automated` rows).
            NpcKind::GoldBird => 442,
            NpcKind::GoldGrasshopper => 446,
            NpcKind::GoldMouse => 447,
            NpcKind::GoldBunny => 443,
            NpcKind::GoldFrog => 445,
""", """            // E8 merge: the snail trio keeps its vanilla cs hand rows
            // (the protocol ids renumbered, the type gates unchanged);
            // Shimmerfly's 677 is c7's above; the 5 gold rows are d40's
            // above (the dup drops).
            NpcKind::Snail => 359,
            NpcKind::GlowingSnail => 360,
            NpcKind::MagmaSnail => 655,
""")
# max_hp
rep("""            NpcKind::Snail | NpcKind::GlowingSnail | NpcKind::MagmaSnail | NpcKind::Shimmerfly => 5,
            // The gold critters (W4-c9): lifeMax 5 each (`NPC.cs:14254` /
            // `:14310` / `:14325` / `:14269` / `:14297`).
            NpcKind::GoldBird | NpcKind::GoldGrasshopper | NpcKind::GoldMouse | NpcKind::GoldBunny | NpcKind::GoldFrog => 5,
""", """            // E8 merge: Shimmerfly's row is c7's above; the 5 gold rows
            // are d40's above (the dup drops).
            NpcKind::Snail | NpcKind::GlowingSnail | NpcKind::MagmaSnail => 5,
""")
# contact_damage
rep("""            NpcKind::Snail | NpcKind::GlowingSnail | NpcKind::MagmaSnail | NpcKind::Shimmerfly => 0,
            // The gold critters (W4-c9): damage 0 each (`NPC.cs:14253` /
            // `:14309` / `:14324` / `:14268` / `:14296`).
            NpcKind::GoldBird | NpcKind::GoldGrasshopper | NpcKind::GoldMouse | NpcKind::GoldBunny | NpcKind::GoldFrog => 0,
""", """            // E8 merge: Shimmerfly's row is c7's above; the 5 gold rows
            // are d40's above (the dup drops).
            NpcKind::Snail | NpcKind::GlowingSnail | NpcKind::MagmaSnail => 0,
""")
open(p, "w", encoding="utf8", newline="").write(box[0])
print("c9 data.rs part 2 done")
