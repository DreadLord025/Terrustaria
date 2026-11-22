# W4-c10 merge data.rs part 2: from_u16 + the stat groups.
import sys, os, re
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from _m1_rep import make_rep
p = "crates/tr-app/src/npc/data.rs"
src = open(p, encoding="utf8", newline="").read()
box = [src]
rep = make_rep(lambda: box[0], lambda v: box.__setitem__(0, v))

# from_u16 waterfowl + the d42 band re-push
rep("""            362 => NpcKind::Duck,
            363 => NpcKind::Duck2,
            364 => NpcKind::DuckWhite,
            365 => NpcKind::DuckWhite2,
            602 => NpcKind::Seagull,
            603 => NpcKind::Seagull2,
            608 => NpcKind::Grebe,
            609 => NpcKind::Grebe2,
            610 => NpcKind::Rat,
            611 => NpcKind::Owl,
            612 => NpcKind::WaterStrider,
            613 => NpcKind::GoldWaterStrider,
            614 => NpcKind::ExplosiveBunny,
            616 => NpcKind::Turtle,
            617 => NpcKind::TurtleJungle,
            618 => NpcKind::BloodNautilus,
            619 => NpcKind::BloodSquid,
            620 => NpcKind::GoblinShark,
            621 => NpcKind::BloodEelHead,
            622 => NpcKind::BloodEelBody,
            623 => NpcKind::BloodEelTail,
            624 => NpcKind::Gnome,
            625 => NpcKind::SeaTurtle,
            626 => NpcKind::Seahorse,
            627 => NpcKind::GoldSeahorse,
            628 => NpcKind::Dandelion,
            629 => NpcKind::IceMimic,
            630 => NpcKind::BloodMummy,
""", """            // E8 merge: 731-734 (the 362-365 slots are d37's); the
            // Seagull/Grebe + Rat..BloodMummy band are d42's arms above.
            731 => NpcKind::Duck,
            732 => NpcKind::Duck2,
            733 => NpcKind::DuckWhite,
            734 => NpcKind::DuckWhite2,
""")
# from_u16 the 12-critter tail
rep("""            357 => NpcKind::Worm,
            442 => NpcKind::GoldBird,
            443 => NpcKind::GoldBunny,
            444 => NpcKind::GoldButterfly,
            445 => NpcKind::GoldFrog,
            446 => NpcKind::GoldGrasshopper,
            447 => NpcKind::GoldMouse,
            448 => NpcKind::GoldWorm,
            484 => NpcKind::EnchantedNightcrawler,
            485 => NpcKind::Grubby,
            486 => NpcKind::Sluggy,
            487 => NpcKind::Buggy,
            _ => return None,
""", """            // E8 merge: the 12-critter tail dropped (c8/d40's arms above).
            _ => return None,
""")
# hit_sound waterfowl: keep the Duck quartet, drop the Seagull/Grebe
rep("""            NpcKind::Duck
            | NpcKind::Duck2
            | NpcKind::DuckWhite
            | NpcKind::DuckWhite2
            | NpcKind::Seagull
            | NpcKind::Seagull2
            | NpcKind::Grebe
            | NpcKind::Grebe2 => "NPC_Hit_1",
""", """            // E8 merge: the Seagull/Grebe rows are d42's above.
            NpcKind::Duck
            | NpcKind::Duck2
            | NpcKind::DuckWhite
            | NpcKind::DuckWhite2 => "NPC_Hit_1",
""")
# hit_sound the 12-critter tail: drop whole
rep("""            NpcKind::Worm
            | NpcKind::GoldBird
            | NpcKind::GoldBunny
            | NpcKind::GoldButterfly
            | NpcKind::GoldFrog
            | NpcKind::GoldGrasshopper
            | NpcKind::GoldMouse
            | NpcKind::GoldWorm
            | NpcKind::EnchantedNightcrawler
            | NpcKind::Grubby
            | NpcKind::Sluggy
            | NpcKind::Buggy => "NPC_Hit_1",
""", """            // E8 merge: the 12-critter hit rows are c8/d40's above.
""")
# death_sound waterfowl
rep("""            NpcKind::Duck
            | NpcKind::Duck2
            | NpcKind::DuckWhite
            | NpcKind::DuckWhite2
            | NpcKind::Seagull
            | NpcKind::Seagull2
            | NpcKind::Grebe
            | NpcKind::Grebe2 => "NPC_Killed_1",
""", """            // E8 merge: the Seagull/Grebe rows are d42's above.
            NpcKind::Duck
            | NpcKind::Duck2
            | NpcKind::DuckWhite
            | NpcKind::DuckWhite2 => "NPC_Killed_1",
""")
# death_sound the GoldMouse + the tail
rep("""            NpcKind::GoldMouse => "NPC_Killed_4",
            // `NPC.cs:13554`, `:14257`/`:14271`/`:14284`/`:14299`/`:14312`/
            // `:14341`, the `:14585` range row.
            NpcKind::Worm
            | NpcKind::GoldBird
            | NpcKind::GoldBunny
            | NpcKind::GoldButterfly
            | NpcKind::GoldFrog
            | NpcKind::GoldGrasshopper
            | NpcKind::GoldWorm
            | NpcKind::EnchantedNightcrawler
            | NpcKind::Grubby
            | NpcKind::Sluggy
            | NpcKind::Buggy => "NPC_Killed_1",
""", """            // E8 merge: the GoldMouse NPCDeath4 + the 12-critter death
            // rows are d40/c8's above (the dup drops).
""")
# no_gravity: the c10 GoldButterfly|GoldBird row - GoldButterfly is d40's
# above; GoldBird is NEW (the aiStyle-24 head write, the superset addition)
rep("""            NpcKind::GoldButterfly | NpcKind::GoldBird => true,
            _ => false,
""", """            // E8 merge: GoldButterfly's row is d40's above; the GoldBird's
            // noGravity is the aiStyle-24 head's per-tick write (`:25325`,
            // the AngryNimbus precedent - the superset addition).
            NpcKind::GoldBird => true,
            _ => false,
""")
open(p, "w", encoding="utf8", newline="").write(box[0])
print("c10 data.rs part 2 done")
