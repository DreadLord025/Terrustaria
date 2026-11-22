# W4-c8 merge npc.rs: ALL_KINDS decl join + rows dedupe + the stat-group
# drops + render_key renumber + pins 501->505 (+4: Worm/TruffleWorm/
# TruffleWormDigger/Maggot; the 5 d40 dups drop).
import sys, os, re
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from _m1_rep import make_rep
p = "crates/tr-app/src/npc.rs"
src = open(p, encoding="utf8", newline="").read()
box = [src]
rep = make_rep(lambda: box[0], lambda v: box.__setitem__(0, v))

# ALL_KINDS decl join: 501 + 4 = 505
rep("""    // + W4-c7's 4 (Firefly 735 / LightningBug 736 - the d37-table
    // renumber, Lavafly 654 / Shimmerfly 677 keep) = 501.
    pub(super) const ALL_KINDS: [NpcKind; 501] = [
    pub(super) const ALL_KINDS: [NpcKind; 279] = [
""", """    // + W4-c7's 4 (Firefly 735 / LightningBug 736 - the d37-table
    // renumber, Lavafly 654 / Shimmerfly 677 keep) = 501; + W4-c8's 4
    // (Worm 737 / TruffleWorm 738 / TruffleWormDigger 739 - the d37-table
    // renumber, Maggot 606 keep; the GoldWorm 448 + 484-487 grub dups
    // dropped - d40's rows own them) = 505.
    pub(super) const ALL_KINDS: [NpcKind; 505] = [
""")
# ALL_KINDS rows: drop the 5 dups from the c8 insert
rep("""        // The crawling-critter family (W4-c8).
        NpcKind::Worm,
        NpcKind::TruffleWorm,
        NpcKind::TruffleWormDigger,
        NpcKind::GoldWorm,
        NpcKind::EnchantedNightcrawler,
        NpcKind::Grubby,
        NpcKind::Sluggy,
        NpcKind::Buggy,
""", """        // The crawling-critter family (W4-c8; the GoldWorm/Enchanted-
        // Nightcrawler/Grubby/Sluggy/Buggy dups dropped - d40's rows above).
        NpcKind::Worm,
        NpcKind::TruffleWorm,
        NpcKind::TruffleWormDigger,
""")
# the Maggot row (hunk 12's separate insert) stays.
# pins 501 -> 505
n = box[0].count("assert_eq!(ALL_KINDS.len(), 501);")
box[0] = box[0].replace("assert_eq!(ALL_KINDS.len(), 501);", "assert_eq!(ALL_KINDS.len(), 505);")
print(f"ALL_KINDS.len pins re-pinned: {n}")
# s1_f_3 pin join
rep("""            501,
            "every decodable kind resolves (the wire roster's size - 497 (spine 488 + c3's 694 + c4's 8) + W4-c7's 4 fireflies (Firefly 735 / LightningBug 736 renumbered per the d37 tables, Lavafly 654 / Shimmerfly 677 keep), L-B gate)"
            279,
""", """            505,
            "every decodable kind resolves (the wire roster's size - 501 (spine 488 + c3's 694 + c4's 8 + c7's 4) + W4-c8's 4 crawlers (Worm 737 / TruffleWorm 738 / TruffleWormDigger 739 renumbered per the d37 tables, Maggot 606 keep; the 5 d40-owned dups dropped), L-B gate)"
""")
# knock_back_resist group: drop the 5 dups
rep("""            NpcKind::Worm
            | NpcKind::TruffleWorm
            | NpcKind::GoldWorm
            | NpcKind::EnchantedNightcrawler
            | NpcKind::Grubby
            | NpcKind::Sluggy
            | NpcKind::Buggy
            | NpcKind::Maggot => 1.0,
""", """            // E8 merge: the 5 d40-owned dups dropped (d40's kb rows above).
            NpcKind::Worm
            | NpcKind::TruffleWorm
            | NpcKind::Maggot => 1.0,
""")
# defense group
rep("""            NpcKind::Worm
            | NpcKind::TruffleWorm
            | NpcKind::TruffleWormDigger
            | NpcKind::GoldWorm
            | NpcKind::EnchantedNightcrawler
            | NpcKind::Grubby
            | NpcKind::Sluggy
            | NpcKind::Buggy
            | NpcKind::Maggot => 0,
""", """            // E8 merge: the 5 d40-owned dups dropped (d40's rows above).
            NpcKind::Worm
            | NpcKind::TruffleWorm
            | NpcKind::TruffleWormDigger
            | NpcKind::Maggot => 0,
""")
# value group
rep("""            NpcKind::Worm
            | NpcKind::TruffleWorm
            | NpcKind::TruffleWormDigger
            | NpcKind::GoldWorm
            | NpcKind::EnchantedNightcrawler
            | NpcKind::Grubby
            | NpcKind::Sluggy
            | NpcKind::Buggy
            | NpcKind::Maggot => 0.0,
""", """            // E8 merge: the 5 d40-owned dups dropped (d40's rows above).
            NpcKind::Worm
            | NpcKind::TruffleWorm
            | NpcKind::TruffleWormDigger
            | NpcKind::Maggot => 0.0,
""")
# update_kind_ai unreachable group: drop the 5 dups (d40's arms own them)
rep("""            NpcKind::Worm
            | NpcKind::TruffleWorm
            | NpcKind::GoldWorm
            | NpcKind::EnchantedNightcrawler
            | NpcKind::Grubby
            | NpcKind::Sluggy
            | NpcKind::Buggy
            | NpcKind::Maggot => {
                unreachable!("aiStyle 66 rides the style axis (W4-c8)")
            }
""", """            // E8 merge: the 5 d40-owned dups ride d40's unreachable arms
            // above (the same axis-66 claim - keep-one).
            NpcKind::Worm
            | NpcKind::TruffleWorm
            | NpcKind::Maggot => {
                unreachable!("aiStyle 66 rides the style axis (W4-c8)")
            }
""")
# find_frame crawler group: KEEP all 9 (the c8 machine is the keeper for
# the canonical d40 kinds too - the superset-module rule; d40's own
# find_frame arms for them must drop instead). Check d40's arms below.
# render_key: renumber the trio, drop the 5 dups
rep("""            NpcKind::Worm => 357,
            NpcKind::TruffleWorm => 374,
            NpcKind::TruffleWormDigger => 375,
            NpcKind::GoldWorm => 448,
            NpcKind::EnchantedNightcrawler => 484,
            NpcKind::Grubby => 485,
            NpcKind::Sluggy => 486,
            NpcKind::Buggy => 487,
            NpcKind::Maggot => 606,
""", """            // E8 merge renumber: the trio rides 737/738/739 (the vanilla
            // 357/374/375 keys are d37's above); the 5 d40-owned dups
            // dropped (d40's keys above); Maggot 606 free.
            NpcKind::Worm => 737,
            NpcKind::TruffleWorm => 738,
            NpcKind::TruffleWormDigger => 739,
            NpcKind::Maggot => 606,
""")
open(p, "w", encoding="utf8", newline="").write(box[0])
print("c8 npc.rs done")
