# W4-c9 merge data.rs: full drops + renumber (anchored blocks).
import sys, os, re
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from _m1_rep import make_rep
p = "crates/tr-app/src/npc/data.rs"
src = open(p, encoding="utf8", newline="").read()
box = [src]
rep = make_rep(lambda: box[0], lambda v: box.__setitem__(0, v))

# to_u16 (anchor: the gold block follows)
rep("""            NpcKind::Snail => 359,
            NpcKind::GlowingSnail => 360,
            NpcKind::MagmaSnail => 655,
            NpcKind::Shimmerfly => 677,
            // The gold critters (W4-c9): the vanilla ids 442/446/447 are
            // free protocol slots (the to_u16 scan: the 442-447 band is
            // unclaimed), so they keep their vanilla ids - the snail
            // precedent.
            NpcKind::GoldBird => 442,
            NpcKind::GoldGrasshopper => 446,
            NpcKind::GoldMouse => 447,
            NpcKind::GoldBunny => 443,
            NpcKind::GoldFrog => 445,
""", """            // E8 merge (d37 tables): Snail/GlowingSnail ride 740/741
            // (d37's FlyingFish/FlyingSnake own the vanilla slots;
            // cs_id/net_id stay 359/360); MagmaSnail 655 keeps; the
            // Shimmerfly 677 dup + the 5 gold dups drop (c7/d40's rows
            // above own them - c9's wiring composes, 349 s6).
            NpcKind::Snail => 740,
            NpcKind::GlowingSnail => 741,
            NpcKind::MagmaSnail => 655,
""")
# from_u16
rep("""            359 => NpcKind::Snail,
            360 => NpcKind::GlowingSnail,
            655 => NpcKind::MagmaSnail,
            677 => NpcKind::Shimmerfly,
            // The gold critters (W4-c9): the vanilla ids kept (`to_u16`
            // above).
            442 => NpcKind::GoldBird,
            446 => NpcKind::GoldGrasshopper,
            447 => NpcKind::GoldMouse,
            443 => NpcKind::GoldBunny,
            445 => NpcKind::GoldFrog,
""", """            // E8 merge: 740/741 (the 359/360 slots are d37's); the
            // 677 + 442/443/445/446/447 dups dropped (c7/d40's arms above).
            740 => NpcKind::Snail,
            741 => NpcKind::GlowingSnail,
            655 => NpcKind::MagmaSnail,
""")
# hit_sound: snail group drops Shimmerfly; gold group drops whole
rep("""            NpcKind::Snail | NpcKind::GlowingSnail | NpcKind::MagmaSnail | NpcKind::Shimmerfly => {
                "NPC_Hit_1"
            }
            // The gold critters (W4-c9): NPCHit1 each (`NPC.cs:14255` /
            // `:14311` / `:14326` / `:14270` / `:14298`).
            NpcKind::GoldBird | NpcKind::GoldGrasshopper | NpcKind::GoldMouse | NpcKind::GoldBunny | NpcKind::GoldFrog => "NPC_Hit_1",
""", """            // E8 merge: Shimmerfly's row is c7's above; the 5 gold rows
            // are d40's above (the dup drops).
            NpcKind::Snail | NpcKind::GlowingSnail | NpcKind::MagmaSnail => {
                "NPC_Hit_1"
            }
""")
# death_sound
rep("""            NpcKind::Snail | NpcKind::GlowingSnail | NpcKind::MagmaSnail | NpcKind::Shimmerfly => {
                "NPC_Killed_1"
            }
            // The gold critters (W4-c9): the GoldBird/GoldGrasshopper/
            // GoldBunny/GoldFrog NPCDeath1 (`NPC.cs:14257` / `:14312` /
            // `:14271` / `:14299`), the GoldMouse NPCDeath4 (`:14327`).
            NpcKind::GoldBird | NpcKind::GoldGrasshopper | NpcKind::GoldBunny | NpcKind::GoldFrog => "NPC_Killed_1",
            NpcKind::GoldMouse => "NPC_Killed_4",
""", """            // E8 merge: Shimmerfly's row is c7's above; the 5 gold rows
            // are d40's above (the dup drops).
            NpcKind::Snail | NpcKind::GlowingSnail | NpcKind::MagmaSnail => {
                "NPC_Killed_1"
            }
""")
# no_gravity
rep("""            NpcKind::Snail | NpcKind::GlowingSnail | NpcKind::MagmaSnail | NpcKind::Shimmerfly => true,
""", """            // E8 merge: Shimmerfly's row is c7's above.
            NpcKind::Snail | NpcKind::GlowingSnail | NpcKind::MagmaSnail => true,
""")
# ai_style: the snail group + the gold group
rep("""            NpcKind::GoldBird => 24,
            NpcKind::GoldGrasshopper => 1,
            NpcKind::GoldMouse => 7,
            // The GoldBunny/GoldFrog's aiStyle 7 (`NPC.cs:14266` / `:14294`).
            NpcKind::GoldBunny | NpcKind::GoldFrog => 7,
""", """            // E8 merge: the 5 gold aiStyle rows are d40's above (24/1/7/
            // 7/7 - the dup drops; c9's wiring composes onto them).
""")
open(p, "w", encoding="utf8", newline="").write(box[0])
print("c9 data.rs part 1 done")
