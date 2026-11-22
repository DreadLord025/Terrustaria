# W4-c4 merge-side E8 drops (d37 tables + research/148 E7 checklist):
# DROP c4's BloodFeeder@241 (-> d37@373), CorruptGoldfish@405 (-> d34@277),
# FlyingFish@359 dup stat arms (-> d37@359 same-id dedupe), Pufferfish@688 +
# Orca@692 (-> d43, lands later). KEEP: Arapaima 406, CrimsonGoldfish 465,
# GoldGoldfish 592, Pupfish 607, Dolphin 615 + the style-44 trio 509/581/587.
# Wiring composes onto canonical kinds (superset-module: c4's style16/44
# machines + FindFrame helpers stay keyed on the canonical variants).
import sys, os, re
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from _m1_rep import make_rep

def edit_file(p, fn):
    src = open(p, encoding="utf8", newline="").read()
    box = [src]
    rep = make_rep(lambda: box[0], lambda v: box.__setitem__(0, v))
    fn(rep, box)
    open(p, "w", encoding="utf8", newline="").write(box[0])
    print(p, "done")

def data(rep, box):
    # to_u16: drop FlyingFish dup arm + the 5 drop identities' arms
    rep("""            NpcKind::FlyingFish => 359,
            NpcKind::GiantFlyingAntlion => 509,
""", """            // E8 merge: FlyingFish 359 is d37's arm above (identical-id
            // dedupe, c4's wiring keeps); GiantFlyingAntlion 509 c4's own.
            NpcKind::GiantFlyingAntlion => 509,
""")
    rep("""            NpcKind::CorruptGoldfish => 405,
            NpcKind::Arapaima => 406,
            NpcKind::BloodFeeder => 241,
            NpcKind::CrimsonGoldfish => 465,
            NpcKind::GoldGoldfish => 592,
            NpcKind::Pupfish => 607,
            NpcKind::Dolphin => 615,
            NpcKind::Pufferfish => 688,
            NpcKind::Orca => 692,
""", """            // E8 merge drops: CorruptGoldfish 405 (d34@277 owns),
            // BloodFeeder 241 (d37@373 owns), Pufferfish 688 / Orca 692
            // (d43 rows them, lands later - the c4 impl retires per
            // research/148's TRIGGERED conditional).
            NpcKind::Arapaima => 406,
            NpcKind::CrimsonGoldfish => 465,
            NpcKind::GoldGoldfish => 592,
            NpcKind::Pupfish => 607,
            NpcKind::Dolphin => 615,
""")
    # from_u16 mirrors
    rep("""            359 => NpcKind::FlyingFish,
            509 => NpcKind::GiantFlyingAntlion,
""", """            // E8 merge: 359 decodes at d37's arm above (identical-id).
            509 => NpcKind::GiantFlyingAntlion,
""")
    rep("""            405 => NpcKind::CorruptGoldfish,
            406 => NpcKind::Arapaima,
            241 => NpcKind::BloodFeeder,
            465 => NpcKind::CrimsonGoldfish,
            592 => NpcKind::GoldGoldfish,
            607 => NpcKind::Pupfish,
            615 => NpcKind::Dolphin,
            688 => NpcKind::Pufferfish,
            692 => NpcKind::Orca,
""", """            // E8 merge drops: 405/241/688/692 (d34/d37/d43 own).
            406 => NpcKind::Arapaima,
            465 => NpcKind::CrimsonGoldfish,
            592 => NpcKind::GoldGoldfish,
            607 => NpcKind::Pupfish,
            615 => NpcKind::Dolphin,
""")
    # hit_sound: drop the FlyingFish half of the c4 pair (d37 group owns)
    rep("""            NpcKind::FlyingFish | NpcKind::EyeballFlyingFish => "NPC_Hit_1",
            NpcKind::GiantFlyingAntlion | NpcKind::FlyingAntlion => "NPC_Hit_32",
""", """            NpcKind::EyeballFlyingFish => "NPC_Hit_1",
            NpcKind::GiantFlyingAntlion | NpcKind::FlyingAntlion => "NPC_Hit_32",
""")
    # hit_sound style-16 group: drop CorruptGoldfish/BloodFeeder/Pufferfish/Orca
    rep("""            NpcKind::CorruptGoldfish
            | NpcKind::Arapaima
            | NpcKind::CrimsonGoldfish
            | NpcKind::GoldGoldfish
            | NpcKind::Pupfish
            | NpcKind::Dolphin
            | NpcKind::Pufferfish
            | NpcKind::Orca => "NPC_Hit_1",
            NpcKind::BloodFeeder => "NPC_Hit_13",
""", """            // E8 merge drops: CorruptGoldfish/BloodFeeder (d34/d37 rows
            // above), Pufferfish/Orca (d43 lands later).
            NpcKind::Arapaima
            | NpcKind::CrimsonGoldfish
            | NpcKind::GoldGoldfish
            | NpcKind::Pupfish
            | NpcKind::Dolphin => "NPC_Hit_1",
""")
    # death_sound mirrors
    rep("""            NpcKind::FlyingFish | NpcKind::EyeballFlyingFish => "NPC_Killed_1",
            NpcKind::GiantFlyingAntlion | NpcKind::FlyingAntlion => "NPC_Killed_35",
""", """            NpcKind::EyeballFlyingFish => "NPC_Killed_1",
            NpcKind::GiantFlyingAntlion | NpcKind::FlyingAntlion => "NPC_Killed_35",
""")
    rep("""            NpcKind::CorruptGoldfish
            | NpcKind::Arapaima
            | NpcKind::CrimsonGoldfish
            | NpcKind::GoldGoldfish
            | NpcKind::Pupfish
            | NpcKind::Dolphin
            | NpcKind::Pufferfish
            | NpcKind::Orca => "NPC_Killed_1",
            NpcKind::BloodFeeder => "NPC_Killed_19",
""", """            // E8 merge drops: CorruptGoldfish/BloodFeeder (d34/d37 rows
            // above), Pufferfish/Orca (d43 lands later).
            NpcKind::Arapaima
            | NpcKind::CrimsonGoldfish
            | NpcKind::GoldGoldfish
            | NpcKind::Pupfish
            | NpcKind::Dolphin => "NPC_Killed_1",
""")
    # no_gravity: drop FlyingFish from the c4 style-44 group (d37 owns)
    rep("""            NpcKind::FlyingFish
            | NpcKind::GiantFlyingAntlion
            | NpcKind::FlyingAntlion
            | NpcKind::EyeballFlyingFish => true,
""", """            // E8 merge: FlyingFish's row is d37's group above.
            NpcKind::GiantFlyingAntlion
            | NpcKind::FlyingAntlion
            | NpcKind::EyeballFlyingFish => true,
""")
    # ai_style: drop FlyingFish from the c4 group (d37 owns 44)
    rep("""            NpcKind::FlyingFish
            | NpcKind::GiantFlyingAntlion
            | NpcKind::FlyingAntlion
            | NpcKind::EyeballFlyingFish => 44,
""", """            // E8 merge: FlyingFish's 44 row is d37's above.
            NpcKind::GiantFlyingAntlion
            | NpcKind::FlyingAntlion
            | NpcKind::EyeballFlyingFish => 44,
""")
    # size: drop FlyingFish from pair; the style-16 size rows live in npc.rs? no - data.rs has c4 swimmer sizes? check: c4 size block only style-44 (the style-16 sizes ride the enum ctor?). We saw size dups only FlyingFish. Fix pair:
    rep("""            NpcKind::FlyingFish | NpcKind::EyeballFlyingFish => (32.0, 18.0),
            NpcKind::GiantFlyingAntlion => (40.0, 31.0),
""", """            NpcKind::EyeballFlyingFish => (32.0, 18.0),
            NpcKind::GiantFlyingAntlion => (40.0, 31.0),
""")
    # cs_id: drop the four; keep 5 survivors (CorruptGoldfish/BloodFeeder
    # canonical cs rows: d34/d37 have them? d34 cs_id CorruptGoldfish=>57
    # exists (dup found earlier), d37 BloodFeeder=>241 exists via fallback
    # net id - check below; keep the arms if canonical lacks them).
    rep("""            NpcKind::CorruptGoldfish => 57,
            NpcKind::Arapaima => 157,
            NpcKind::BloodFeeder => 241,
            NpcKind::CrimsonGoldfish => 465,
            NpcKind::GoldGoldfish => 592,
            NpcKind::Pupfish => 607,
            NpcKind::Dolphin => 615,
            NpcKind::Pufferfish => 688,
            NpcKind::Orca => 692,
""", """            // E8 merge drops: CorruptGoldfish (d34's 57 row above),
            // BloodFeeder (d37's 241 - the ENTRIES fallback resolves),
            // Pufferfish/Orca (d43 lands later).
            NpcKind::Arapaima => 157,
            NpcKind::CrimsonGoldfish => 465,
            NpcKind::GoldGoldfish => 592,
            NpcKind::Pupfish => 607,
            NpcKind::Dolphin => 615,
""")
    # max_hp: drop FlyingFish + the four
    rep("""            NpcKind::FlyingFish => 20,
            NpcKind::GiantFlyingAntlion => 90,
""", """            // E8 merge: FlyingFish's 20 is d37's row above.
            NpcKind::GiantFlyingAntlion => 90,
""")
    # contact_damage: drop FlyingFish
    rep("""            NpcKind::FlyingFish => 9,
            NpcKind::GiantFlyingAntlion => 34,
""", """            // E8 merge: FlyingFish's 9 is d37's row above.
            NpcKind::GiantFlyingAntlion => 34,
""")

edit_file("crates/tr-app/src/npc/data.rs", data)
