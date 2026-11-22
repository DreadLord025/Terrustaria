# W4-c4 merge: npc_ai_style16.rs vanilla_type superset (the 5 new kinds
# route through the family arm) + witness dedupe.
import sys, os
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from _m1_rep import make_rep
p = "crates/tr-app/src/npc_ai_style16.rs"
src = open(p, encoding="utf8", newline="").read()
box = [src]
rep = make_rep(lambda: box[0], lambda v: box.__setitem__(0, v))
rep("""        // W4-d37 - the Blood Feeder (241) rides the generic aiStyle-16
        // wet/dry machine (`NPC.cs:23824-23961`, the 0.1/0.1 chase + bob).
        NpcKind::BloodFeeder => 241,
        _ => unreachable!("only aiStyle-16 kinds reach the family arm"),
""", """        // W4-d37 - the Blood Feeder (241) rides the generic aiStyle-16
        // wet/dry machine (`NPC.cs:23824-23961`, the 0.1/0.1 chase + bob).
        NpcKind::BloodFeeder => 241,
        // W4-c4 merge (the superset-module fold): the five kept kinds
        // route through the SAME family machine (the 157 asymmetric
        // chase caps `:23774-23802`, the 615 idle-timer/leap machine
        // `:23572-23697`, the critter exclusion `:23701`).
        NpcKind::Arapaima => 157,
        NpcKind::CrimsonGoldfish => 465,
        NpcKind::GoldGoldfish => 592,
        NpcKind::Pupfish => 607,
        NpcKind::Dolphin => 615,
        _ => unreachable!("only aiStyle-16 kinds reach the family arm"),
""")
rep("""            (NpcKind::Dolphin, 615),
            (NpcKind::Pufferfish, 688),
            (NpcKind::Orca, 692),
        ] {
""", """            (NpcKind::Dolphin, 615),
            // E8 merge: Pufferfish/Orca drop with the kinds (d43 lands
            // later; the module's 688/692 machines stay for its merge).
        ] {
""")
open(p, "w", encoding="utf8", newline="").write(box[0])
print("style16 done")
