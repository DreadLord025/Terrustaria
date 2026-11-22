# W4-c8 merge data.rs: the trio renumber + the 5-dup group drops.
import sys, os, re
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from _m1_rep import make_rep
p = "crates/tr-app/src/npc/data.rs"
src = open(p, encoding="utf8", newline="").read()
box = [src]
rep = make_rep(lambda: box[0], lambda v: box.__setitem__(0, v))

# to_u16 (anchor: the closing brace)
rep("""            NpcKind::Worm => 357,
            NpcKind::TruffleWorm => 374,
            NpcKind::TruffleWormDigger => 375,
            NpcKind::GoldWorm => 448,
            NpcKind::EnchantedNightcrawler => 484,
            NpcKind::Grubby => 485,
            NpcKind::Sluggy => 486,
            NpcKind::Buggy => 487,
            NpcKind::Maggot => 606,
        }
    }
""", """            // E8 merge (d37 tables): Worm/TruffleWorm/TruffleWormDigger
            // ride 737/738/739 (d37's SeaSnail/BloodJelly/IceGolem own
            // the vanilla slots; cs_id/net_id stay 357/374/375); the
            // GoldWorm 448 + EnchantedNightcrawler/Grubby/Sluggy/Buggy
            // 484-487 dups DROP (d40's rows above own them - c8's
            // style-66 wiring composes onto the canonical kinds).
            NpcKind::Worm => 737,
            NpcKind::TruffleWorm => 738,
            NpcKind::TruffleWormDigger => 739,
            NpcKind::Maggot => 606,
        }
    }
""")
# from_u16
rep("""            357 => NpcKind::Worm,
            374 => NpcKind::TruffleWorm,
            375 => NpcKind::TruffleWormDigger,
            448 => NpcKind::GoldWorm,
            484 => NpcKind::EnchantedNightcrawler,
            485 => NpcKind::Grubby,
            486 => NpcKind::Sluggy,
            487 => NpcKind::Buggy,
            606 => NpcKind::Maggot,
""", """            // E8 merge: 737/738/739 (the 357/374/375 slots are d37's);
            // the 448/484-487 dups dropped (d40's decode arms above).
            737 => NpcKind::Worm,
            738 => NpcKind::TruffleWorm,
            739 => NpcKind::TruffleWormDigger,
            606 => NpcKind::Maggot,
""")
# hit_sound group: drop the 5 dups (d40 rows own them)
rep("""            NpcKind::Worm
            | NpcKind::TruffleWorm
            | NpcKind::TruffleWormDigger
            | NpcKind::GoldWorm
            | NpcKind::EnchantedNightcrawler
            | NpcKind::Grubby
            | NpcKind::Sluggy
            | NpcKind::Buggy
            | NpcKind::Maggot => "NPC_Hit_1",
""", """            // E8 merge: the GoldWorm/EnchantedNightcrawler/Grubby/
            // Sluggy/Buggy rows are d40's above (the dup drop).
            NpcKind::Worm
            | NpcKind::TruffleWorm
            | NpcKind::TruffleWormDigger
            | NpcKind::Maggot => "NPC_Hit_1",
""")
# death_sound group
rep("""            NpcKind::Worm
            | NpcKind::TruffleWorm
            | NpcKind::TruffleWormDigger
            | NpcKind::GoldWorm
            | NpcKind::EnchantedNightcrawler
            | NpcKind::Grubby
            | NpcKind::Sluggy
            | NpcKind::Buggy
            | NpcKind::Maggot => "NPC_Killed_1",
""", """            // E8 merge: the 5 d40-owned dups dropped (rows above).
            NpcKind::Worm
            | NpcKind::TruffleWorm
            | NpcKind::TruffleWormDigger
            | NpcKind::Maggot => "NPC_Killed_1",
""")
# ai_style group
rep("""            NpcKind::Worm
            | NpcKind::TruffleWorm
            | NpcKind::GoldWorm
            | NpcKind::EnchantedNightcrawler
            | NpcKind::Grubby
            | NpcKind::Sluggy
            | NpcKind::Buggy
            | NpcKind::Maggot => 66,
""", """            // E8 merge: the 5 d40-owned dups dropped - d40's aiStyle-66
            // rows above own them (the superset wiring composes).
            NpcKind::Worm
            | NpcKind::TruffleWorm
            | NpcKind::Maggot => 66,
""")
# size group
rep("""            NpcKind::Worm
            | NpcKind::GoldWorm
            | NpcKind::EnchantedNightcrawler
            | NpcKind::Grubby
            | NpcKind::Sluggy
            | NpcKind::Buggy
            | NpcKind::Maggot => (10.0, 4.0),
""", """            // E8 merge: the 5 d40-owned dups dropped (rows above).
            NpcKind::Worm
            | NpcKind::Maggot => (10.0, 4.0),
""")
# cs_id group: keep the trio's vanilla hand rows + Maggot; drop the 5
rep("""            NpcKind::Worm => 357,
            NpcKind::TruffleWorm => 374,
            NpcKind::TruffleWormDigger => 375,
            NpcKind::GoldWorm => 448,
            NpcKind::EnchantedNightcrawler => 484,
            NpcKind::Grubby => 485,
            NpcKind::Sluggy => 486,
            NpcKind::Buggy => 487,
            NpcKind::Maggot => 606,
""", """            // E8 merge: the trio keeps its vanilla cs hand rows (the
            // protocol ids renumbered, the C#-type gates unchanged); the
            // 5 d40-owned dups dropped (d40's rows/fallback above).
            NpcKind::Worm => 357,
            NpcKind::TruffleWorm => 374,
            NpcKind::TruffleWormDigger => 375,
            NpcKind::Maggot => 606,
""")
# max_hp group
rep("""            NpcKind::Worm
            | NpcKind::TruffleWorm
            | NpcKind::TruffleWormDigger
            | NpcKind::GoldWorm
            | NpcKind::EnchantedNightcrawler
            | NpcKind::Grubby
            | NpcKind::Sluggy
            | NpcKind::Buggy
            | NpcKind::Maggot => 5,
""", """            // E8 merge: the 5 d40-owned dups dropped (rows above).
            NpcKind::Worm
            | NpcKind::TruffleWorm
            | NpcKind::TruffleWormDigger
            | NpcKind::Maggot => 5,
""")
# contact_damage group
rep("""            NpcKind::Worm
            | NpcKind::TruffleWorm
            | NpcKind::TruffleWormDigger
            | NpcKind::GoldWorm
            | NpcKind::EnchantedNightcrawler
            | NpcKind::Grubby
            | NpcKind::Sluggy
            | NpcKind::Buggy
            | NpcKind::Maggot => 0,
""", """            // E8 merge: the 5 d40-owned dups dropped (rows above).
            NpcKind::Worm
            | NpcKind::TruffleWorm
            | NpcKind::TruffleWormDigger
            | NpcKind::Maggot => 0,
""")
open(p, "w", encoding="utf8", newline="").write(box[0])
print("c8 data.rs done")
