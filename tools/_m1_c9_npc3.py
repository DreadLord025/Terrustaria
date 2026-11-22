# W4-c9 merge npc.rs part 3: render_key + ALL_KINDS + pins 505->508.
import sys, os, re
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from _m1_rep import make_rep
p = "crates/tr-app/src/npc.rs"
src = open(p, encoding="utf8", newline="").read()
box = [src]
rep = make_rep(lambda: box[0], lambda v: box.__setitem__(0, v))

# render_key: snail trio renumber, Shimmerfly + golds drop
rep("""            NpcKind::Snail => 359,
            NpcKind::GlowingSnail => 360,
            NpcKind::MagmaSnail => 655,
            NpcKind::Shimmerfly => 677,
            // The gold critters (W4-c9): the vanilla ids - unused by every
            // arm above (the G6 key stays injective).
            NpcKind::GoldBird => 442,
            NpcKind::GoldGrasshopper => 446,
            NpcKind::GoldMouse => 447,
            NpcKind::GoldBunny => 443,
            NpcKind::GoldFrog => 445,
""", """            // E8 merge renumber: the snail pair rides 740/741 (the
            // vanilla 359/360 keys are d37's above); MagmaSnail 655 free;
            // the Shimmerfly 677 key is c7's above; the 5 gold keys are
            // d40's above (the dup drops).
            NpcKind::Snail => 740,
            NpcKind::GlowingSnail => 741,
            NpcKind::MagmaSnail => 655,
""")
# ALL_KINDS decl join: 505 + 3 = 508
rep("""    pub(super) const ALL_KINDS: [NpcKind; 505] = [
    pub(super) const ALL_KINDS: [NpcKind; 279] = [
""", """    // + W4-c9's 3 snails (Snail 740 / GlowingSnail 741 - the d37-table
    // renumber, MagmaSnail 655 keep; the Shimmerfly + 5 gold dups dropped
    // - c7/d40's rows own them) = 508.
    pub(super) const ALL_KINDS: [NpcKind; 508] = [
""")
# ALL_KINDS rows: drop the c9 Shimmerfly + 5 gold dups
rep("""        // The snail wall-crawler family + its shimmer target (W4-c9 phase
        // 1 - the latent ALL_KINDS gap: the four kinds landed in the enum
        // + from_u16 but were missing from this roster array).
        NpcKind::Snail,
        NpcKind::GlowingSnail,
        NpcKind::MagmaSnail,
        NpcKind::Shimmerfly,
        // The gold critters (W4-c9): the first three of the 442-447 band.
        NpcKind::GoldBird,
        NpcKind::GoldGrasshopper,
        NpcKind::GoldMouse,
        // The gold critters (W4-c9): the bunny + the frog.
        NpcKind::GoldBunny,
        NpcKind::GoldFrog,
""", """        // The snail wall-crawler family (W4-c9; the Shimmerfly + 5 gold
        // dups dropped - c7/d40's rows above own them).
        NpcKind::Snail,
        NpcKind::GlowingSnail,
        NpcKind::MagmaSnail,
""")
# pins 505 -> 508
n = box[0].count("assert_eq!(ALL_KINDS.len(), 505);")
box[0] = box[0].replace("assert_eq!(ALL_KINDS.len(), 505);", "assert_eq!(ALL_KINDS.len(), 508);")
print(f"ALL_KINDS.len pins re-pinned: {n}")
# s1_f_3 pin join (the 3-way stray)
rep("""            505,
            "every decodable kind resolves (the wire roster's size - 501 (spine 488 + c3's 694 + c4's 8 + c7's 4) + W4-c8's 4 crawlers (Worm 737 / TruffleWorm 738 / TruffleWormDigger 739 renumbered per the d37 tables, Maggot 606 keep; the 5 d40-owned dups dropped), L-B gate)"
            "every decodable kind resolves (the wire roster's size - 279 after S1-D-3ff wave A's 24 kinds + S1-F-13c slices A+B's frost fifteen + S1-D-3 W-3's two volley bees + S1-D-3-wire-plantera-minions' four + S1-D-3-wire-destroyer's three + W-9's fishron four + W-A2-3's LeechTail + S1-F-13d's ForceBubble/MartianTurret + S1-D-3 wyvern's six + S1-D-3z's seven zombies + S1-D-3-wire-smallkinds' three new kinds, L-B gate + W4-c8's nine crawling critters)"
            279,
            "every decodable kind resolves (the wire roster's size - 270 after S1-D-3ff wave A's 24 kinds + S1-F-13c slices A+B's frost fifteen + S1-D-3 W-3's two volley bees + S1-D-3-wire-plantera-minions' four + S1-D-3-wire-destroyer's three + W-9's fishron four + W-A2-3's LeechTail + S1-F-13d's ForceBubble/MartianTurret + S1-D-3 wyvern's six + S1-D-3z's seven zombies + S1-D-3-wire-smallkinds' three new kinds, L-B gate; + W4-c9's snail family four + the five gold critters 442/443/445/446/447)"
""", """            508,
            "every decodable kind resolves (the wire roster's size - 505 (spine 488 + c3's 694 + c4's 8 + c7's 4 + c8's 4) + W4-c9's 3 snails (Snail 740 / GlowingSnail 741 renumbered per the d37 tables, MagmaSnail 655 keep; the Shimmerfly + 5 gold dups dropped - c7/d40 own them), L-B gate)"
""")
open(p, "w", encoding="utf8", newline="").write(box[0])
print("c9 npc.rs part 3 done")
