# W4-c4 merge-side E8 drops, part 2: the style-16 stat groups in data.rs.
import sys, os
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from _m1_rep import make_rep
p = "crates/tr-app/src/npc/data.rs"
src = open(p, encoding="utf8", newline="").read()
box = [src]
rep = make_rep(lambda: box[0], lambda v: box.__setitem__(0, v))

# no_gravity group (find the tail with Pufferfish/Orca)
rep("""            NpcKind::CorruptGoldfish
            | NpcKind::Arapaima
            | NpcKind::BloodFeeder
            | NpcKind::CrimsonGoldfish
            | NpcKind::GoldGoldfish
            | NpcKind::Pupfish
            | NpcKind::Dolphin
            | NpcKind::Pufferfish
            | NpcKind::Orca => true,
""", """            // E8 merge drops: CorruptGoldfish/BloodFeeder (d34/d37 rows
            // above), Pufferfish/Orca (d43 lands later).
            NpcKind::Arapaima
            | NpcKind::CrimsonGoldfish
            | NpcKind::GoldGoldfish
            | NpcKind::Pupfish
            | NpcKind::Dolphin => true,
""")
# ai_style group
rep("""            NpcKind::CorruptGoldfish
            | NpcKind::Arapaima
            | NpcKind::BloodFeeder
            | NpcKind::CrimsonGoldfish
            | NpcKind::GoldGoldfish
            | NpcKind::Pupfish
            | NpcKind::Dolphin
            | NpcKind::Pufferfish
            | NpcKind::Orca => 16,
""", """            // E8 merge drops: CorruptGoldfish/BloodFeeder (d34/d37 rows
            // above), Pufferfish/Orca (d43 lands later).
            NpcKind::Arapaima
            | NpcKind::CrimsonGoldfish
            | NpcKind::GoldGoldfish
            | NpcKind::Pupfish
            | NpcKind::Dolphin => 16,
""")
# size group
rep("""            NpcKind::CorruptGoldfish | NpcKind::BloodFeeder | NpcKind::CrimsonGoldfish => {
                (18.0, 20.0)
            }
            NpcKind::Arapaima => (74.0, 20.0),
            NpcKind::GoldGoldfish | NpcKind::Pupfish | NpcKind::Dolphin => (20.0, 18.0),
            NpcKind::Pufferfish => (32.0, 16.0),
            NpcKind::Orca => (120.0, 34.0),
""", """            // E8 merge drops: CorruptGoldfish/BloodFeeder (d34/d37 rows
            // above), Pufferfish/Orca (d43 lands later).
            NpcKind::CrimsonGoldfish => (18.0, 20.0),
            NpcKind::Arapaima => (74.0, 20.0),
            NpcKind::GoldGoldfish | NpcKind::Pupfish | NpcKind::Dolphin => (20.0, 18.0),
""")
# max_hp group
rep("""            NpcKind::CorruptGoldfish => 100,
            NpcKind::Arapaima => 200,
            NpcKind::BloodFeeder => 150,
            NpcKind::CrimsonGoldfish => 110,
            NpcKind::GoldGoldfish | NpcKind::Pupfish | NpcKind::Dolphin | NpcKind::Pufferfish => 5,
            NpcKind::Orca => 400,
""", """            // E8 merge drops: CorruptGoldfish/BloodFeeder (d34/d37 rows
            // above), Pufferfish/Orca (d43 lands later).
            NpcKind::Arapaima => 200,
            NpcKind::CrimsonGoldfish => 110,
            NpcKind::GoldGoldfish | NpcKind::Pupfish | NpcKind::Dolphin => 5,
""")
# contact_damage group
rep("""            NpcKind::CorruptGoldfish => 30,
            NpcKind::Arapaima => 75,
            NpcKind::BloodFeeder => 50,
            NpcKind::CrimsonGoldfish => 31,
            NpcKind::GoldGoldfish | NpcKind::Pupfish | NpcKind::Dolphin | NpcKind::Pufferfish => 0,
            NpcKind::Orca => 50,
""", """            // E8 merge drops: CorruptGoldfish/BloodFeeder (d34/d37 rows
            // above), Pufferfish/Orca (d43 lands later).
            NpcKind::Arapaima => 75,
            NpcKind::CrimsonGoldfish => 31,
            NpcKind::GoldGoldfish | NpcKind::Pupfish | NpcKind::Dolphin => 0,
""")
open(p, "w", encoding="utf8", newline="").write(box[0])
print("data.rs part 2 done")
