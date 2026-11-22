# W4-c4 merge part 4: render/entities.rs draw arms + rotation gate fold +
# anchor arms/witness; assets.rs sheet dedupe + slots.
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

def entities(rep, box):
    # STYLE44_SHEETS: drop 224 (d37's D37_SHEETS owns it)
    rep("""pub(crate) const STYLE44_SHEETS: &[u32] = &[
    224, // the Flying Fish
    509, // the Giant Antlion Swarmer
""", """pub(crate) const STYLE44_SHEETS: &[u32] = &[
    // E8 merge: 224 rides d37's D37_SHEETS (the FlyingFish dedupe).
    509, // the Giant Antlion Swarmer
""")
    # STYLE16_SHEETS: drop 57/241 (d34/d37 own), 688 (d43 lands later)
    rep("""pub(crate) const STYLE16_SHEETS: &[u32] = &[
    57,  // the Corrupt Goldfish
    157, // the Arapaima
    241, // the Blood Feeder
    465, // the Vicious Goldfish
    592, // the Gold Goldfish
    607, // the Pupfish
    615, // the Dolphin
    688, // the Pufferfish
];
""", """pub(crate) const STYLE16_SHEETS: &[u32] = &[
    // E8 merge: 57/241 ride d34/d37's sheet consts; 688 lands with d43
    // (the catchItem/anchor/module keepers stay wired on the vanilla id).
    157, // the Arapaima
    465, // the Vicious Goldfish
    592, // the Gold Goldfish
    607, // the Pupfish
    615, // the Dolphin
];
""")
    # draw arms: style-44 group drops FlyingFish (d37's group arm owns)
    rep("""            NpcKind::FlyingFish
            | NpcKind::GiantFlyingAntlion
            | NpcKind::FlyingAntlion
            | NpcKind::EyeballFlyingFish => {
                let ty = texture_net_id(n.kind);
                pumpkin_moon_draw(assets.npc_sheet(ty), npc_rows(ty), n, w, h)
            }
""", """            // E8 merge: FlyingFish's draw is d37's group arm above (the
            // identical texture_net_id shape - keep-one).
            NpcKind::GiantFlyingAntlion
            | NpcKind::FlyingAntlion
            | NpcKind::EyeballFlyingFish => {
                let ty = texture_net_id(n.kind);
                pumpkin_moon_draw(assets.npc_sheet(ty), npc_rows(ty), n, w, h)
            }
""")
    # style-16 draw group: drop CorruptGoldfish/BloodFeeder (d34/d37 own)
    # + Pufferfish/Orca (d43)
    rep("""            NpcKind::CorruptGoldfish
            | NpcKind::Arapaima
            | NpcKind::BloodFeeder
            | NpcKind::CrimsonGoldfish
            | NpcKind::GoldGoldfish
            | NpcKind::Pupfish
            | NpcKind::Dolphin
            | NpcKind::Pufferfish
            | NpcKind::Orca => {
                let ty = texture_net_id(n.kind);
                pumpkin_moon_draw(assets.npc_sheet(ty), npc_rows(ty), n, w, h)
            }
""", """            // E8 merge drops: CorruptGoldfish (d34's arm above),
            // BloodFeeder (d37's group arm above), Pufferfish/Orca (d43).
            NpcKind::Arapaima
            | NpcKind::CrimsonGoldfish
            | NpcKind::GoldGoldfish
            | NpcKind::Pupfish
            | NpcKind::Dolphin => {
                let ty = texture_net_id(n.kind);
                pumpkin_moon_draw(assets.npc_sheet(ty), npc_rows(ty), n, w, h)
            }
""")
    # anchor arms: Pufferfish/Orca variants gone - re-key onto the cs_id
    # guard? The nudge match is on n.kind. The 688 keeper: the DD2GoblinT1
    # arm already carries the same +=10 literal (the C#'s single 688 row
    # is type-keyed; the port's kind at 688 was c4's). With the kind gone,
    # drop the two arms; d43 re-adds them at its merge (carry-list note).
    rep("""        // W4-c4 - the SAME `:23115-23117` row for the vanilla type 688
        // (`NPCID.Pufferfish`): the C# keys the nudge on the type id, and
        // the port now carries a kind at 688 (the goblin T1's row above is
        // the historical mis-mapping - left untouched, the two kinds ride
        // the same literal exactly like the C#'s single 688 arm, the
        // FlyingAntlion 581 precedent).
        Pufferfish => num46 += 10.0,
        // W4-c4 - `:23072-23074`: the Orca's (692) wet-dependent anchor
        // `num46 = wet ? 6 : 10` (an absolute, not an add).
        Orca => num46 = if n.wet { 6.0 } else { 10.0 },
""", """        // W4-c4 merge (E8, research/148 carry-list): the Pufferfish 688
        // `num46 += 10` (`Main.cs:23115-23117`) + the Orca 692 wet anchor
        // `num46 = wet ? 6 : 10` (`:23072-23074`) arms RETIRE with the
        // kinds - d43's merge re-lands them verbatim (the keeper receipt
        // rides the chain-banner + research/148).
""")
    # anchor witness: drop the Pufferfish/Orca asserts
    rep("""        // W4-c4 (the style-16 swimmers): the Pufferfish's `num46 += 10`
        // (`Main.cs:23115-23117`) and the Orca's wet-dependent
        // `num46 = wet ? 6 : 10` (`:23072-23074`). The other seven fish
        // have no anchor row (the default 0).
        assert_eq!(npc_anchor_nudge(&bare(NpcKind::Pufferfish)), 10.0);
        let mut orca = bare(NpcKind::Orca);
        orca.wet = false;
        assert_eq!(npc_anchor_nudge(&orca), 10.0, "the dry Orca");
        orca.wet = true;
        assert_eq!(npc_anchor_nudge(&orca), 6.0, "the wet Orca");
        for kind in [
""", """        // W4-c4 merge (E8): the Pufferfish/Orca anchor asserts retire
        // with the kinds (d43 re-pins them - research/148's carry-list).
        // The remaining fish have no anchor row (the default 0).
        for kind in [
""")
    # rotation gate: fold c4's whitelist rows (the merged draw consumes
    # the ONE gate). FlyingFish/CorruptGoldfish/BloodFeeder join too (the
    # canonical kinds' machines write the same tilts - c4's module owns
    # them now). Pufferfish/Orca rows wait for d43.
    rep("""            | NpcKind::DD2LightningBug
            | NpcKind::GoldGoldfishWalker
""", """            // W4-c4 merge fold (the d40/d35 precedent): the style-44
            // swimmers' tilts (the antlions' AI tilt `NPC.cs:30963`/
            // `:30996`, the fish's FindFrame tilt `:74768`/`:74773`) +
            // the style-16 swimmers' `vy * direction * 0.1` (`:24008-
            // 24019`) / the Dolphin's leap tilt (`:23607-23615`). The
            // canonical FlyingFish/CorruptGoldfish/BloodFeeder ride c4's
            // machines now (the E8 keep-one) - their rows join here;
            // Pufferfish/Orca join with d43.
            | NpcKind::FlyingFish
            | NpcKind::GiantFlyingAntlion
            | NpcKind::FlyingAntlion
            | NpcKind::EyeballFlyingFish
            | NpcKind::CorruptGoldfish
            | NpcKind::Arapaima
            | NpcKind::BloodFeeder
            | NpcKind::CrimsonGoldfish
            | NpcKind::GoldGoldfish
            | NpcKind::Pupfish
            | NpcKind::Dolphin
            | NpcKind::DD2LightningBug
            | NpcKind::GoldGoldfishWalker
""")

def assets(rep, box):
    # STATIC_SHEET_NAMES: drop the c4 224/57/241 dups + the 688 row
    rep("""        // W4-c4 - the style-44 swimmers' sheets (`render/entities.rs`'s
        // STYLE44_SHEETS): 224 the Flying Fish, 509 the Giant Antlion
        // Swarmer, 581 the Antlion Swarmer, 587 the Wandering Eye Fish.
        // Skyline-measured headroom: +46 px on the main set (7757) and
        // +24 px on the full gore+font set (8143) - both under 8192.
        "NPC_224",
        "NPC_509",
        "NPC_581",
        "NPC_587",
""", """        // W4-c4 - the style-44 swimmers' sheets (STYLE44_SHEETS): 509
        // the Giant Antlion Swarmer, 581 the Antlion Swarmer, 587 the
        // Wandering Eye Fish. E8 merge: NPC_224 rides d37's row above.
        "NPC_509",
        "NPC_581",
        "NPC_587",
""")
    rep("""        "NPC_57",
        "NPC_157",
        "NPC_241",
        "NPC_465",
        "NPC_592",
        "NPC_607",
        "NPC_615",
        "NPC_688",
""", """        // E8 merge: NPC_57/NPC_241 ride d34/d37's rows above; NPC_688
        // lands with d43 (the sheet census survives, research/144/148).
        "NPC_157",
        "NPC_465",
        "NPC_592",
        "NPC_607",
        "NPC_615",
""")

edit_file("crates/tr-app/src/render/entities.rs", entities)
edit_file("crates/tr-app/src/assets.rs", assets)
