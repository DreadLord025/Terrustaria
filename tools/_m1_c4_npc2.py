# W4-c4 merge npc.rs part 2: witnesses re-key/drop + CS_STYLES dedupe.
import sys, os, re
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from _m1_rep import make_rep
p = "crates/tr-app/src/npc.rs"
src = open(p, encoding="utf8", newline="").read()
box = [src]
rep = make_rep(lambda: box[0], lambda v: box.__setitem__(0, v))

# CS_STYLES: drop the Pufferfish/Orca rows (kinds gone; d43 re-adds)
rep("""        (NpcKind::Dolphin, 16),
        (NpcKind::Pufferfish, 16),
        (NpcKind::Orca, 16),
""", """        (NpcKind::Dolphin, 16),
        // E8 merge: Pufferfish/Orca rows drop with the kinds (d43).
""")

# style-16 defaults witness: re-key CorruptGoldfish 405->277, BloodFeeder
# proto 241->373; drop the Pufferfish/Orca rows + their asserts.
rep("""            (NpcKind::CorruptGoldfish, 57u16, 405u16, 18.0, 20.0, 30, 6, 100, 1.0, 500.0, 6u32),
            (NpcKind::Arapaima, 157, 406, 74.0, 20.0, 75, 30, 200, 1.0, 500.0, 6),
            (NpcKind::BloodFeeder, 241, 241, 18.0, 20.0, 50, 20, 150, 1.0, 500.0, 6),
""", """            // E8 merge re-keys: CorruptGoldfish rides d34@277,
            // BloodFeeder rides d37@373 (the canonical protocol ids).
            (NpcKind::CorruptGoldfish, 57u16, 277u16, 18.0, 20.0, 30, 6, 100, 1.0, 500.0, 6u32),
            (NpcKind::Arapaima, 157, 406, 74.0, 20.0, 75, 30, 200, 1.0, 500.0, 6),
            (NpcKind::BloodFeeder, 241, 373, 18.0, 20.0, 50, 20, 150, 1.0, 500.0, 6),
""")
rep("""            (NpcKind::Dolphin, 615, 615, 20.0, 18.0, 0, 0, 5, 0.5, 0.0, 5),
            (NpcKind::Pufferfish, 688, 688, 32.0, 16.0, 0, 0, 5, 0.5, 0.0, 6),
            (NpcKind::Orca, 692, 692, 120.0, 34.0, 50, 20, 400, 0.7, 10000.0, 16),
        ] {
""", """            (NpcKind::Dolphin, 615, 615, 20.0, 18.0, 0, 0, 5, 0.5, 0.0, 5),
            // E8 merge: Pufferfish/Orca rows drop with the kinds (d43).
        ] {
""")
rep("""        assert_eq!(NpcKind::BloodFeeder.hit_sound(), Some("NPC_Hit_13"));
        assert_eq!(NpcKind::BloodFeeder.death_sound(), Some("NPC_Killed_19"));
        assert_eq!(NpcKind::Orca.hit_sound(), Some("NPC_Hit_1"));
        assert_eq!(NpcKind::Orca.death_sound(), Some("NPC_Killed_1"));
""", """        assert_eq!(NpcKind::BloodFeeder.hit_sound(), Some("NPC_Hit_13"));
        assert_eq!(NpcKind::BloodFeeder.death_sound(), Some("NPC_Killed_19"));
        // E8 merge: the Orca sound asserts drop with the kind (d43).
""")
rep("""        assert!(crate::npc_buffs::NpcBuffs::for_kind(NpcKind::CorruptGoldfish).buff_immune[31]);
        assert!(!crate::npc_buffs::NpcBuffs::for_kind(NpcKind::Pufferfish).buff_immune[31]);
        assert!(!crate::npc_buffs::NpcBuffs::for_kind(NpcKind::Orca).buff_immune[31]);
""", """        assert!(crate::npc_buffs::NpcBuffs::for_kind(NpcKind::CorruptGoldfish).buff_immune[31]);
        // E8 merge: the Pufferfish/Orca null-immunity asserts drop with
        // the kinds (d43 re-pins them at its merge).
""")
# swimmers-step witness: drop Pufferfish/Orca from the loop
rep("""            NpcKind::Pupfish,
            NpcKind::Dolphin,
            NpcKind::Pufferfish,
            NpcKind::Orca,
        ] {
            let rows = crate::render::entities::npc_rows(
""", """            NpcKind::Pupfish,
            NpcKind::Dolphin,
            // E8 merge: Pufferfish/Orca drop with the kinds (d43).
        ] {
            let rows = crate::render::entities::npc_rows(
""")
# the puff-flag witness: the KIND is gone but the module machine stays;
# re-key the witness onto the module direct-call shape (no Npc::new of a
# dropped variant). The witness body drives update_rng - which needs a
# kind. Retire the whole test with a receipt comment (d43 re-lands it).
i = box[0].index("    /// W4-c4 follow-up (the puff-flag fix):")
j = box[0].index('assert!(n.hurt_cd > 0.0, "the cooldown outlives the flag (the defect\'s window)");', i)
j = box[0].index("    }", j) + len("    }")
nl = "\r\n" if "\r\n" in box[0] else "\n"
replacement = (
"    // W4-c4 merge (E8, research/148): the puff-flag witness RETIRED with" + nl +
"    // the Pufferfish kind (d43 rows 688 later). The ONE-TICK just_hit fix" + nl +
"    // it pinned lives in `update_swimmer`'s Ai16Ctx feed (kept) + the" + nl +
"    // npc_ai_style16 module's own puff witnesses (kept) - d43's merge" + nl +
"    // re-lands the kind-level regression pin (the 17a241f6a receipt)."
)
box[0] = box[0][:i] + replacement + box[0][j:]

open(p, "w", encoding="utf8", newline="").write(box[0])
print("npc.rs part 2 done")
