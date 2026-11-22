# W4-c10 merge data.rs: the Duck quartet 362-365->731-734 (d37 tables);
# the 16 dups DROP (Worm->c8@737, the gold septet + bait tail->d40, the
# waterfowl Seagull/Grebe pairs->d42). The c10 to_u16/from_u16 inserts
# carry the waterfowl + the 12-critter tail + the d42 band re-push.
import sys, os, re
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from _m1_rep import make_rep
p = "crates/tr-app/src/npc/data.rs"
src = open(p, encoding="utf8", newline="").read()
box = [src]
rep = make_rep(lambda: box[0], lambda v: box.__setitem__(0, v))

# to_u16 waterfowl: the Duck quartet renumbers; the Seagull/Grebe pairs
# are d42's rows above (dup drop); the Rat..BloodMummy band is d42's too.
rep("""            NpcKind::Duck => 362,
            NpcKind::Duck2 => 363,
            NpcKind::DuckWhite => 364,
            NpcKind::DuckWhite2 => 365,
            NpcKind::Seagull => 602,
            NpcKind::Seagull2 => 603,
            NpcKind::Grebe => 608,
            NpcKind::Grebe2 => 609,
            NpcKind::Rat => 610,
            NpcKind::Owl => 611,
            NpcKind::WaterStrider => 612,
            NpcKind::GoldWaterStrider => 613,
            NpcKind::ExplosiveBunny => 614,
            NpcKind::Turtle => 616,
            NpcKind::TurtleJungle => 617,
            NpcKind::BloodNautilus => 618,
            NpcKind::BloodSquid => 619,
            NpcKind::GoblinShark => 620,
            NpcKind::BloodEelHead => 621,
            NpcKind::BloodEelBody => 622,
            NpcKind::BloodEelTail => 623,
            NpcKind::Gnome => 624,
            NpcKind::SeaTurtle => 625,
            NpcKind::Seahorse => 626,
            NpcKind::GoldSeahorse => 627,
            NpcKind::Dandelion => 628,
            NpcKind::IceMimic => 629,
            NpcKind::BloodMummy => 630,
""", """            // E8 merge (d37 tables): the Duck quartet rides 731-734
            // (d37's GoldfishWalker/HornetFatty/HornetHoney/HornetLeafy
            // own the vanilla slots; cs_id/net_id stay 362-365); the
            // Seagull/Grebe pairs + the Rat..BloodMummy band are d42's
            // rows above (the dup drops - c10's waterfowl wiring composes).
            NpcKind::Duck => 731,
            NpcKind::Duck2 => 732,
            NpcKind::DuckWhite => 733,
            NpcKind::DuckWhite2 => 734,
""")
# to_u16 the 12-critter tail: all dups drop
rep("""            NpcKind::Worm => 357,
            NpcKind::GoldBird => 442,
            NpcKind::GoldBunny => 443,
            NpcKind::GoldButterfly => 444,
            NpcKind::GoldFrog => 445,
            NpcKind::GoldGrasshopper => 446,
            NpcKind::GoldMouse => 447,
            NpcKind::GoldWorm => 448,
            NpcKind::EnchantedNightcrawler => 484,
            NpcKind::Grubby => 485,
            NpcKind::Sluggy => 486,
            NpcKind::Buggy => 487,
        }
    }
""", """            // E8 merge: the 12-critter tail dropped - Worm rides c8@737,
            // the gold septet + the bait tail are d40's rows above (the
            // dup drops; c10's wiring composes onto the canonical kinds).
        }
    }
""")
open(p, "w", encoding="utf8", newline="").write(box[0])
print("c10 data.rs to_u16 done")
