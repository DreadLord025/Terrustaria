# W4-c4 merge part 3: bestiary.rs ENTRIES dedupe + join repairs +
# GoldGoldfishWalker fold re-point (148 checklist item 5); npc_buffs /
# netglue / ui portraits / entities draw+rotation / assets sheets.
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

def bestiary(rep, box):
    # 1. GoldGoldfishWalker mid-def join repair + fold re-point (148 item 5):
    # the walker's interim OWN entry retires; the 593 credit folds onto
    # c4's GoldGoldfish 592 entry in BOTH resolvers.
    rep("""    // W4-d42 - the GoldGoldfishWalker's interim OWN entry: its vanilla
    // credit folds onto the GoldGoldfish (592, `ContentSamples.cs:1049`),
    // but 592 is c4b's unported kind - the d40 BoneThrowingSkeleton2/3
    // unported-fold-target precedent. The merge folds it onto 592 once
    // c4b lands (the `entry_for_npc` note).
    BestiaryEntryDef {
        credit_id: "GoldGoldfishWalker",
        net_id: 593,
        kind: NpcKind::GoldGoldfishWalker,
    // W4-c4 (the style-44 swimmers): the four kinds' own entries - the
""", """    // W4-c4 merge (148 item 5): the GoldGoldfishWalker's interim OWN
    // entry RETIRED - its vanilla credit folds onto the GoldGoldfish
    // (592, `ContentSamples.cs:1049`) now that c4's 592 entry landed;
    // both resolvers carry the fold arm below.
    // W4-c4 (the style-44 swimmers): the four kinds' own entries - the
""")
    # 2. drop c4's FlyingFish dup entry (d37's one-line row owns 224)
    rep("""    BestiaryEntryDef {
        credit_id: "FlyingFish",
        net_id: 224,
        kind: NpcKind::FlyingFish,
        variant: None,
        quick_unlock: false,
        name: "Flying Fish",
    },
    BestiaryEntryDef {
        credit_id: "GiantFlyingAntlion",
""", """    // E8 merge: the FlyingFish 224 entry is d37's row above (dup drop).
    BestiaryEntryDef {
        credit_id: "GiantFlyingAntlion",
""")
    # 3. drop c4's CorruptGoldfish + BloodFeeder dup entries
    rep("""    BestiaryEntryDef {
        credit_id: "CorruptGoldfish",
        net_id: 57,
        kind: NpcKind::CorruptGoldfish,
        variant: None,
        quick_unlock: false,
        name: "Corrupt Goldfish",
    },
    BestiaryEntryDef {
        credit_id: "Arapaima",
""", """    // E8 merge: the CorruptGoldfish 57 entry is d34's row above.
    BestiaryEntryDef {
        credit_id: "Arapaima",
""")
    rep("""    BestiaryEntryDef {
        credit_id: "BloodFeeder",
        net_id: 241,
        kind: NpcKind::BloodFeeder,
        variant: None,
        quick_unlock: false,
        name: "Blood Feeder",
    },
    BestiaryEntryDef {
        credit_id: "CrimsonGoldfish",
""", """    // E8 merge: the BloodFeeder 241 entry is d37's row above.
    BestiaryEntryDef {
        credit_id: "CrimsonGoldfish",
""")
    # 4. PossessedArmor/WaterBoltMimic mid-def join repairs + drop the
    # Pufferfish/Orca entries (d43 lands later)
    rep("""        name: "Possessed Armor",
    // W4-c3 (the style-10 family): the WaterBoltMimic's own entry
""", """        name: "Possessed Armor",
    },
    // W4-c3 (the style-10 family): the WaterBoltMimic's own entry
""")
    rep("""        name: "Water Bolt Mimic",
    BestiaryEntryDef {
        credit_id: "Pupfish",
""", """        name: "Water Bolt Mimic",
    },
    BestiaryEntryDef {
        credit_id: "Pupfish",
""")
    rep("""    BestiaryEntryDef {
        credit_id: "Pufferfish",
        net_id: 688,
        kind: NpcKind::Pufferfish,
        variant: None,
        quick_unlock: false,
        name: "Pufferfish",
    },
    BestiaryEntryDef {
        credit_id: "Orca",
        net_id: 692,
        kind: NpcKind::Orca,
        variant: None,
        quick_unlock: false,
        name: "Orca",
    },
];
""", """    // E8 merge: the Pufferfish 688 / Orca 692 entries drop with the
    // kinds (d43 rows them later - research/148's TRIGGERED conditional).
];
""")
    # 5. the fold arms in BOTH resolvers
    rep("""        NpcKind::ArmedTorchZombie => NpcKind::TorchZombie,
        // (the fairy/balloon/dragonfly dups 583-585/594/595-601 dropped -
        // d33 owns them, d37 tables; their folds land with its splice.)
        NpcKind::Seagull2 => NpcKind::Seagull,
""", """        NpcKind::ArmedTorchZombie => NpcKind::TorchZombie,
        // W4-c4 merge (148 item 5): the walker's 593 credit folds onto
        // the GoldGoldfish 592 (`ContentSamples.cs:1049`).
        NpcKind::GoldGoldfishWalker => NpcKind::GoldGoldfish,
        // (the fairy/balloon/dragonfly dups 583-585/594/595-601 dropped -
        // d33 owns them, d37 tables; their folds land with its splice.)
        NpcKind::Seagull2 => NpcKind::Seagull,
""")
    rep("""        // The GoldGoldfishWalker (593) folds onto the GoldGoldfish (592,
        // `ContentSamples.cs:1049`); 592 is c4b's unported kind, so the
        // walker keeps its OWN entry for now (the d40 BoneThrowingSkeleton2/3
        // unported-fold-target precedent) - the merge re-points this arm at
        // `NpcKind::GoldGoldfish` and drops the walker's entry once c4b lands.
""", """        // W4-c4 merge (148 item 5): the walker's 593 credit folds onto
        // the GoldGoldfish 592 (`ContentSamples.cs:1049`) - c4's entry
        // landed, the interim own-row retired.
        NpcKind::GoldGoldfishWalker => NpcKind::GoldGoldfish,
""")

def buffs(rep, box):
    rep("""        NpcKind::CorruptGoldfish
        | NpcKind::Arapaima
        | NpcKind::BloodFeeder
        | NpcKind::CrimsonGoldfish
        | NpcKind::GoldGoldfish
        | NpcKind::Pupfish
        | NpcKind::Dolphin => &[31],
        NpcKind::Pufferfish | NpcKind::Orca => &[],
""", """        // E8 merge drops: CorruptGoldfish/BloodFeeder (d34/d37 rows
        // above own them), Pufferfish/Orca (d43 lands later - the null
        // else-branch already covers their absence).
        NpcKind::Arapaima
        | NpcKind::CrimsonGoldfish
        | NpcKind::GoldGoldfish
        | NpcKind::Pupfish
        | NpcKind::Dolphin => &[31],
""")

def netglue(rep, box):
    rep("""        | crate::npc::NpcKind::CorruptGoldfish
        | crate::npc::NpcKind::Arapaima
        | crate::npc::NpcKind::BloodFeeder
        | crate::npc::NpcKind::CrimsonGoldfish
        | crate::npc::NpcKind::GoldGoldfish
        | crate::npc::NpcKind::Pupfish
        | crate::npc::NpcKind::Dolphin
        | crate::npc::NpcKind::Pufferfish
        | crate::npc::NpcKind::Orca => crate::npc::Npc::spawn_cave_walker(world, tx, kind)?,
""", """        // E8 merge drops: CorruptGoldfish/BloodFeeder ride the canonical
        // seed arms (d34/d37's netglue rows), Pufferfish/Orca land with d43.
        | crate::npc::NpcKind::Arapaima
        | crate::npc::NpcKind::CrimsonGoldfish
        | crate::npc::NpcKind::GoldGoldfish
        | crate::npc::NpcKind::Pupfish
        | crate::npc::NpcKind::Dolphin => crate::npc::Npc::spawn_cave_walker(world, tx, kind)?,
""")

def ui(rep, box):
    # portraits: drop c4's FlyingFish half (d37's arm owns), the
    # CorruptGoldfish/BloodFeeder dups (d34/d37 arms own), Pufferfish/Orca
    rep("""        (NpcKind::FlyingFish | NpcKind::EyeballFlyingFish, _) => {
""", """        // E8 merge: FlyingFish's portrait is d37's arm below.
        (NpcKind::EyeballFlyingFish, _) => {
""")
    rep("""        (NpcKind::CorruptGoldfish | NpcKind::CrimsonGoldfish | NpcKind::GoldGoldfish | NpcKind::Pupfish, _) => {
""", """        // E8 merge: CorruptGoldfish's portrait is d34's arm below.
        (NpcKind::CrimsonGoldfish | NpcKind::GoldGoldfish | NpcKind::Pupfish, _) => {
""")
    rep("""        (NpcKind::BloodFeeder, _) => {
            let h = entry_sheet(lookups, entry)
                .map(|s| s.height as f32 / 6.0)
                .unwrap_or(30.0);
            (sheet_i, h, 36.0, h, white)
        }
        (NpcKind::Arapaima, _) => {
""", """        // E8 merge: BloodFeeder's portrait is d37's arm below.
        (NpcKind::Arapaima, _) => {
""")
    rep("""        (NpcKind::Pufferfish, _) => {
            let h = entry_sheet(lookups, entry)
                .map(|s| s.height as f32 / 6.0)
                .unwrap_or(42.0);
            (sheet_i, h, 46.0, h, white)
        }
        (NpcKind::Orca, _) => {
            let h = entry_sheet(lookups, entry)
                .map(|s| s.height as f32 / 16.0)
                .unwrap_or(62.0);
            (sheet_i, h, 136.0, h, white)
        }
""", """        // E8 merge: the Pufferfish/Orca portraits drop with the kinds
        // (d43 lands later; the NPC_692 census survives, research/144).
""")

edit_file("crates/tr-app/src/bestiary.rs", bestiary)
edit_file("crates/tr-app/src/npc_buffs.rs", buffs)
edit_file("crates/tr-app/src/app/netglue.rs", netglue)
edit_file("crates/tr-app/src/ui/bestiary.rs", ui)
