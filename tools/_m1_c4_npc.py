# W4-c4 merge-side E8 drops in npc.rs: enum variants (5), kb/def/value
# groups, update_kind_ai arms, find_frame arms, render_key arms, ALL_KINDS
# decl+rows, pins 489 -> 497 (+8: GiantFlyingAntlion/FlyingAntlion/
# EyeballFlyingFish/Arapaima/CrimsonGoldfish/GoldGoldfish/Pupfish/Dolphin).
# Wait - CrimsonGoldfish was dropped from main at d40's splice, so c4
# re-adds it: +9 kinds total (509/581/587/406/465/592/607/615 = 8).
import sys, os, re
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from _m1_rep import make_rep
p = "crates/tr-app/src/npc.rs"
src = open(p, encoding="utf8", newline="").read()
box = [src]
rep = make_rep(lambda: box[0], lambda v: box.__setitem__(0, v))

# --- enum: drop FlyingFish/CorruptGoldfish/BloodFeeder/Pufferfish/Orca ---
rep("""    /// `NPCID.FlyingFish` (224, `NPCID.cs:11125`) - the rain swimmer
    /// (`aiStyle 44`): 32x18, dmg 9, def 4, 20 HP, kb 0.8,
    /// NPCHit1/NPCDeath1, value 300 (`NPC.cs:11885-11897`). WIRED
    /// (W4-c4): the axis' 44 arm owns the flyer integration tail; the
    /// steering machine runs in the world-sim pre-pass
    /// (`npc_ai_style44::style44_step`, the style-56 seam). The protocol
    /// id is 359 - d37's frozen-band FlyingFish row ADOPTED (the original
    /// port-local 354 collided d37's CochinealBeetle; the vanilla 224
    /// slot is the WyvernLegs' port-local band, `to_u16`'s guard).
    FlyingFish,
    /// `NPCID.GiantFlyingAntlion`""",
"""    // W4-c4 merge-side E8 dedupe (d37 tables): the FlyingFish variant is
    // d37's above (same id 359); c4's style-44 wiring (the world-sim
    // pre-pass + FindFrame + draw) composes onto it.
    /// `NPCID.GiantFlyingAntlion`""")
rep("""    /// `NPCID.CorruptGoldfish` (57, `NPCID.cs:10791`) - the corruption's
    /// goldfish (`aiStyle 16`): 18x20, dmg 30, def 6, 100 HP, kb 1 (the
    /// `SetDefaults` head's reset, `NPC.cs:8533` - no row), NPCHit1/
    /// NPCDeath1, value 500 (`NPC.cs:9489-9501`). WIRED (W4-c4): the
    /// `update_swimmer` family arm (`npc_ai_style16::ai_016_step`, the
    /// style-44-adjacent `integrate_flyer_collision` + `is_wet` seam).
    /// The protocol id is port-local 405 - the free-port band (the
    /// original 229 squatted the Pirate's reserved cs hole; the vanilla
    /// 57 slot is the GoblinThief's, `to_u16`'s bijection guard).
    CorruptGoldfish,
    /// `NPCID.Arapaima`""",
"""    // W4-c4 merge-side E8 drop: the CorruptGoldfish variant is d34's
    // above (@277); c4's style-16 machine + FindFrame compose onto it.
    /// `NPCID.Arapaima`""")
rep("""    /// `NPCID.BloodFeeder` (241, `NPCID.cs:11159`) - the crimson's fish
    /// (`aiStyle 16`): 18x20, dmg 50, def 20, 150 HP, kb 1 (no row),
    /// NPCHit13/NPCDeath19, value 500, npcSlots 0.5 (`NPC.cs:12105-12118`).
    /// WIRED (W4-c4), the 57 seam. The protocol id keeps the vanilla 241
    /// (free).
    BloodFeeder,
    /// `NPCID.CrimsonGoldfish`""",
"""    // W4-c4 merge-side E8 drop: the BloodFeeder variant is d37's above
    // (@373); c4's style-16 machine + FindFrame compose onto it.
    /// `NPCID.CrimsonGoldfish`""")
rep("""    /// `NPCID.Pufferfish` (688, `NPCID.cs:12053`) - the ocean's puffer
    /// (`aiStyle 16`): 32x16, dmg 0, def 0, 5 HP, kb 0.5, NPCHit1/
    /// NPCDeath1, catchItem 5511 (`NPC.cs:17653-17666`). WIRED (W4-c4),
    /// the 57 seam; the module's 688 puff latch (`:23498-23571`) reads
    /// `justHit`. The protocol id keeps the vanilla 688 (free).
    Pufferfish,
    /// `NPCID.Orca` (692, `NPCID.cs:12061`) - the ocean's apex swimmer
    /// (`aiStyle 16`): 120x34, dmg 50, def 20, 400 HP, kb 0.7, NPCHit1/
    /// NPCDeath1, value 10000 (`NPC.cs:17709-17722`). WIRED (W4-c4), the
    /// 57 seam; the module's 692 aim rotation (`:23990-24007`) + the
    /// Shark-group chase caps (`:23803`). The protocol id keeps the
    /// vanilla 692 (free). The DRAW sheet is atlas-deferred (the 136x992
    /// strip overflows the 8192 bound - the wyvern precedent, ANALYSIS).
    Orca,
}""",
"""    // W4-c4 merge-side E8 drops (research/148 TRIGGERED conditional):
    // Pufferfish 688 / Orca 692 retire onto d43's rows (its lane lands
    // later). The keeper carry-list transplants HERE at this splice:
    // the style16 module's 688 puff one-tick just_hit machine + the
    // 688/692 FindFrame arms + catchItem 5511 (bug_net) + the num46
    // anchors (npc_anchor_nudge) + the NPC_692 sheet census
    // (research/144, atlas-deferred) all stay keyed on the vanilla ids;
    // d43's kind rows re-activate them at its merge.
}""")

# --- knock_back_resist: drop the c4 style-16 group's four, keep 5 ---
rep("""            NpcKind::CorruptGoldfish | NpcKind::Arapaima | NpcKind::BloodFeeder | NpcKind::CrimsonGoldfish => 1.0,
            NpcKind::GoldGoldfish | NpcKind::Pupfish | NpcKind::Dolphin | NpcKind::Pufferfish => 0.5,
            NpcKind::Orca => 0.7,
""", """            // E8 merge drops: CorruptGoldfish/BloodFeeder (d34/d37 rows
            // above), Pufferfish/Orca (d43 lands later).
            NpcKind::Arapaima | NpcKind::CrimsonGoldfish => 1.0,
            NpcKind::GoldGoldfish | NpcKind::Pupfish | NpcKind::Dolphin => 0.5,
""")
# style-44 kb: drop FlyingFish (d37 owns 0.8)
rep("""            NpcKind::FlyingFish => 0.8,
            NpcKind::GiantFlyingAntlion => 0.3,
""", """            // E8 merge: FlyingFish's 0.8 is d37's row above.
            NpcKind::GiantFlyingAntlion => 0.3,
""")
# --- defense ---
rep("""            NpcKind::CorruptGoldfish => 6,
            NpcKind::Arapaima => 30,
            NpcKind::BloodFeeder => 20,
            NpcKind::CrimsonGoldfish => 7,
            NpcKind::GoldGoldfish | NpcKind::Pupfish | NpcKind::Dolphin | NpcKind::Pufferfish => 0,
            NpcKind::Orca => 20,
""", """            // E8 merge drops: CorruptGoldfish/BloodFeeder (d34/d37 rows
            // above), Pufferfish/Orca (d43 lands later).
            NpcKind::Arapaima => 30,
            NpcKind::CrimsonGoldfish => 7,
            NpcKind::GoldGoldfish | NpcKind::Pupfish | NpcKind::Dolphin => 0,
""")
rep("""            NpcKind::FlyingFish => 4,
            NpcKind::GiantFlyingAntlion => 12,
""", """            // E8 merge: FlyingFish's 4 is d37's row above.
            NpcKind::GiantFlyingAntlion => 12,
""")
# --- value ---
rep("""            NpcKind::CorruptGoldfish | NpcKind::Arapaima | NpcKind::BloodFeeder | NpcKind::CrimsonGoldfish => 500.0,
            NpcKind::GoldGoldfish | NpcKind::Pupfish | NpcKind::Dolphin | NpcKind::Pufferfish => 0.0,
            NpcKind::Orca => 10000.0,
""", """            // E8 merge drops: CorruptGoldfish/BloodFeeder (d34/d37 rows
            // above), Pufferfish/Orca (d43 lands later).
            NpcKind::Arapaima | NpcKind::CrimsonGoldfish => 500.0,
            NpcKind::GoldGoldfish | NpcKind::Pupfish | NpcKind::Dolphin => 0.0,
""")
rep("""            NpcKind::FlyingFish => 300.0,
            NpcKind::GiantFlyingAntlion => 195.0,
""", """            // E8 merge: FlyingFish's 300 is d37's row above.
            NpcKind::GiantFlyingAntlion => 195.0,
""")
# --- update_kind_ai: style-44 unreachable group drops FlyingFish ---
rep("""            NpcKind::FlyingFish
            | NpcKind::GiantFlyingAntlion
            | NpcKind::FlyingAntlion
            | NpcKind::EyeballFlyingFish => {
                unreachable!("aiStyle 44 rides the style axis (W4-c4)")
            }
""", """            // E8 merge: FlyingFish rides d37's update_flying_fish arm
            // below (its solo style-44 wrapper - the world-sim pre-pass
            // supersedes it at the slot turn; keep-one per research/148:
            // the pre-pass RUNS FIRST, the kind arm never fires for it).
            NpcKind::GiantFlyingAntlion
            | NpcKind::FlyingAntlion
            | NpcKind::EyeballFlyingFish => {
                unreachable!("aiStyle 44 rides the style axis (W4-c4)")
            }
""")
# --- update_kind_ai: style-16 group drops four, keeps five ---
rep("""            NpcKind::CorruptGoldfish
            | NpcKind::Arapaima
            | NpcKind::BloodFeeder
            | NpcKind::CrimsonGoldfish
            | NpcKind::GoldGoldfish
            | NpcKind::Pupfish
            | NpcKind::Dolphin
            | NpcKind::Pufferfish
            | NpcKind::Orca => {
                self.update_swimmer(world, player, rng)
            }
""", """            // E8 merge drops: Pufferfish/Orca (d43 lands later);
            // CorruptGoldfish/BloodFeeder ride the canonical arms below
            // (d34/d38's update_swimmer - the SAME family machine).
            NpcKind::Arapaima
            | NpcKind::CrimsonGoldfish
            | NpcKind::GoldGoldfish
            | NpcKind::Pupfish
            | NpcKind::Dolphin => {
                self.update_swimmer(world, player, rng)
            }
""")
# --- find_frame: style-44 group drops FlyingFish ---
rep("""            NpcKind::FlyingFish
            | NpcKind::GiantFlyingAntlion
            | NpcKind::FlyingAntlion
            | NpcKind::EyeballFlyingFish => {
                crate::npc_ai_style44::style44_find_frame(
""", """            // E8 merge: FlyingFish's FindFrame is d37's style44 arm
            // below (the identical module call - keep-one).
            NpcKind::GiantFlyingAntlion
            | NpcKind::FlyingAntlion
            | NpcKind::EyeballFlyingFish => {
                crate::npc_ai_style44::style44_find_frame(
""")
# --- find_frame: style-16 group drops Pufferfish/Orca ONLY (the c4
# helper is the keeper for CorruptGoldfish/BloodFeeder per 148 - drop
# d34's inline case-57 copy + d37's inline case-241 copy instead) ---
rep("""            NpcKind::CorruptGoldfish
            | NpcKind::Arapaima
            | NpcKind::BloodFeeder
            | NpcKind::CrimsonGoldfish
            | NpcKind::GoldGoldfish
            | NpcKind::Pupfish
            | NpcKind::Dolphin
            | NpcKind::Pufferfish
            | NpcKind::Orca => {
                crate::npc_ai_style16::style16_find_frame(
""", """            // E8 merge: Pufferfish/Orca drop (d43 lands later); the
            // CorruptGoldfish/BloodFeeder cases KEEP c4's helper (148's
            // keep-one: d34's inline case-57 + d37's inline case-241
            // copies dropped below - behaviour-identical bands).
            NpcKind::CorruptGoldfish
            | NpcKind::Arapaima
            | NpcKind::BloodFeeder
            | NpcKind::CrimsonGoldfish
            | NpcKind::GoldGoldfish
            | NpcKind::Pupfish
            | NpcKind::Dolphin => {
                crate::npc_ai_style16::style16_find_frame(
""")
# drop d34's inline CorruptGoldfish case-57 copy
rep("""            NpcKind::CorruptGoldfish => {
                self.sprite_dir = self.facing;
                self.frame_counter += 1.0;
                if self.wet {
                    if self.frame_counter < 6.0 {
                        self.frame = 0;
                    } else if self.frame_counter < 12.0 {
                        self.frame = 1;
                    } else if self.frame_counter < 18.0 {
                        self.frame = 2;
                    } else if self.frame_counter < 24.0 {
                        self.frame = 3;
                    } else {
                        self.frame_counter = 0.0;
                    }
                } else if self.frame_counter < 6.0 {
                    self.frame = 4;
                } else if self.frame_counter < 12.0 {
                    self.frame = 5;
                } else {
                    self.frame_counter = 0.0;
                }
            }
            NpcKind::Vulture => {
""", """            // W4-c4 merge keep-one (148): d34's inline case-57 copy
            // dropped - c4's style16_find_frame group above owns the
            // CorruptGoldfish (behaviour-identical bands, NPC.cs:74182-
            // 74226 re-read).
            NpcKind::Vulture => {
""")
# drop d37's inline BloodFeeder case-241 copy
rep("""            // The blood feeder (241) rides the case-55 fish group (`NPC.cs:
            // 74181-74226`): the flip, the wet 6/12/18/24 bands over rows
            // 0..3, the dry 6/12 bands over rows 4/5.
            NpcKind::BloodFeeder => {
                self.sprite_dir = self.facing;
                self.frame_counter += 1.0;
                if self.wet {
                    self.frame = if self.frame_counter < 6.0 {
                        0
                    } else if self.frame_counter < 12.0 {
                        1
                    } else if self.frame_counter < 18.0 {
                        2
                    } else if self.frame_counter < 24.0 {
                        3
                    } else {
                        self.frame_counter = 0.0;
                        self.frame
                    };
                } else if self.frame_counter < 6.0 {
                    self.frame = 4;
                } else if self.frame_counter < 12.0 {
                    self.frame = 5;
                } else {
                    self.frame_counter = 0.0;
                }
            }
""", """            // W4-c4 merge keep-one (148): d37's inline case-241 copy
            // dropped - c4's style16_find_frame group above owns the
            // BloodFeeder (behaviour-identical bands).
""")
# drop d37's FlyingFish find_frame arm? NO - keep it (the canonical kind
# is d37's; its arm calls the same module - keep-one satisfied).
# --- render_key ---
rep("""            NpcKind::FlyingFish => 359,
            NpcKind::GiantFlyingAntlion => 509,
""", """            // E8 merge: FlyingFish's 359 key is d37's arm above.
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
""", """            // E8 merge drops: CorruptGoldfish 405 (d34's 277 key above),
            // BloodFeeder 241 (d37's key above), Pufferfish/Orca (d43).
            NpcKind::Arapaima => 406,
            NpcKind::CrimsonGoldfish => 465,
            NpcKind::GoldGoldfish => 592,
            NpcKind::Pupfish => 607,
            NpcKind::Dolphin => 615,
""")
# --- ALL_KINDS decl join: 489 + 8 = 497 ---
rep("""    // Wave-1 splice pin (367 s1): 489 = the spine-end 488 + W4-c3's 1
    // (WaterBoltMimic 694; its CursedSkull 355 / GiantCursedSkull 289 dup
    // ports dropped per the d37 E8 tables - d34@267/d38@289 own them).
    pub(super) const ALL_KINDS: [NpcKind; 489] = [
    pub(super) const ALL_KINDS: [NpcKind; 283] = [
""", """    // Wave-1 splice pin (367 s1): 497 = 489 (spine 488 + c3's 694) +
    // W4-c4's 8 (GiantFlyingAntlion 509 / FlyingAntlion 581 /
    // EyeballFlyingFish 587 / Arapaima 406 / CrimsonGoldfish 465 (c4's
    // kept impl - d40's dup dropped at its splice) / GoldGoldfish 592 /
    // Pupfish 607 / Dolphin 615; the 5 E8 drops: FlyingFish -> d37@359,
    // CorruptGoldfish -> d34@277, BloodFeeder -> d37@373, Pufferfish 688
    // + Orca 692 -> d43 (lands later), d37 tables + research/148).
    pub(super) const ALL_KINDS: [NpcKind; 497] = [
""")
# --- ALL_KINDS rows ---
rep("""        // W4-c4 - the style-44 swimmers.
        NpcKind::FlyingFish,
        NpcKind::GiantFlyingAntlion,
        NpcKind::FlyingAntlion,
        NpcKind::EyeballFlyingFish,
        // W4-c4 - the style-16 swimmers.
        NpcKind::CorruptGoldfish,
        NpcKind::Arapaima,
        NpcKind::BloodFeeder,
        NpcKind::CrimsonGoldfish,
        NpcKind::GoldGoldfish,
        NpcKind::Pupfish,
        NpcKind::Dolphin,
        NpcKind::Pufferfish,
        NpcKind::Orca,
""", """        // W4-c4 - the style-44 swimmers (FlyingFish is d37's row above).
        NpcKind::GiantFlyingAntlion,
        NpcKind::FlyingAntlion,
        NpcKind::EyeballFlyingFish,
        // W4-c4 - the style-16 swimmers (CorruptGoldfish/BloodFeeder are
        // d34/d37's rows above; Pufferfish/Orca land with d43).
        NpcKind::Arapaima,
        NpcKind::CrimsonGoldfish,
        NpcKind::GoldGoldfish,
        NpcKind::Pupfish,
        NpcKind::Dolphin,
""")
# --- pins 489 -> 497 ---
n = box[0].count("assert_eq!(ALL_KINDS.len(), 489);")
box[0] = box[0].replace("assert_eq!(ALL_KINDS.len(), 489);", "assert_eq!(ALL_KINDS.len(), 497);")
print(f"ALL_KINDS.len pins re-pinned: {n}")
# s1_f_3 pin
rep("""            489,
            "every decodable kind resolves (the wire roster's size - the spine-end 488 (367 s1: d34 17 + d38 35 + d37 46 + d40 55 minus the 465 dup + d42 32 minus the 11 d33 dups + d35 33 at 698-730) + W4-c3's WaterBoltMimic 694 (the 355/289 dups dropped per the d37 E8 tables), L-B gate)"
""", """            497,
            "every decodable kind resolves (the wire roster's size - 489 (spine 488 + c3's 694) + W4-c4's 8 (509/581/587/406/465/592/607/615; the FlyingFish/CorruptGoldfish/BloodFeeder dups deduped onto d37/d34, Pufferfish/Orca retire to d43 per research/148), L-B gate)"
""")
open(p, "w", encoding="utf8", newline="").write(box[0])
print("npc.rs c4 drops done")
