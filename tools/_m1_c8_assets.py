# W4-c8 merge assets.rs: slots dedupe (the 5 d40 dups drop), decl 314->317,
# parallel vec += 357/374/606; STATIC_SHEET_NAMES NPC_448 dup drop + the
# c8 sheet rows keep (357/374/375/606 new; 448/484-487 d40 owns).
import sys, os, re
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from _m1_rep import make_rep
p = "crates/tr-app/src/assets.rs"
src = open(p, encoding="utf8", newline="").read()
box = [src]
rep = make_rep(lambda: box[0], lambda v: box.__setitem__(0, v))

# STATIC_SHEET_NAMES: the c8 insert's 448/484-487 dups drop (d40 owns);
# 357/374/375 keep + the 606 row (hunk 1) keeps.
rep("""        // W4-c8 - the crawling-critter family's own sheets
        // (`CRAWLER_SHEETS`), keyed by the vanilla net id in
        // `NpcAssets::npc_sheets` (`render/entities.rs`).
        "NPC_357",
        "NPC_374",
        "NPC_375",
        "NPC_448",
        "NPC_484",
        "NPC_485",
        "NPC_486",
        "NPC_487",
""", """        // W4-c8 - the crawling-critter family's own sheets
        // (`CRAWLER_SHEETS`), keyed by the vanilla net id in
        // `NpcAssets::npc_sheets` (`render/entities.rs`). E8 merge: the
        // 448/484-487 dups ride d40's D40_SHEETS rows above.
        "NPC_357",
        "NPC_374",
        "NPC_375",
""")
# slots decl join: 314 + 3 = 317
rep("""pub const BESTIARY_NPC_SHEET_IDS: [u16; 314] = [
pub const BESTIARY_NPC_SHEET_IDS: [u16; 189] = [
""", """// + W4-c8's 3 visible crawlers (357/374/606; the digger 375 is
// Hide=true - folds onto the TruffleWorm; the 448/484-487 dups dropped,
// d40's slots above own them) = 317.
pub const BESTIARY_NPC_SHEET_IDS: [u16; 317] = [
""")
# slots rows: drop the 5 dups
rep("""        // W4-c8 - the crawling-critter family, appended last for the same
        // reason. The digger 375 is Hide=true (no grid card of its own -
        // it folds onto the TruffleWorm), so only the eight visible
        // crawlers take sheet slots.
        357, 374, 448, 484, 485, 486, 487, 606,
""", """        // W4-c8 - the crawling-critter family, appended last for the same
        // reason. The digger 375 is Hide=true (no grid card of its own -
        // it folds onto the TruffleWorm); the 448/484-487 dups dropped
        // (d40's slots above own them) - 3 new slots.
        357, 374, 606,
""")
# parallel vec: append the 3 new strips after the 694 row
rep("""                atlas.sheet("NPC_694").copied(),
            ],
""", """                atlas.sheet("NPC_694").copied(),
                // W4-c8 - the 3 new crawler strips, parallel to
                // BESTIARY_NPC_SHEET_IDS (the 448/484-487 dups dropped).
                atlas.sheet("NPC_357").copied(),
                atlas.sheet("NPC_374").copied(),
                atlas.sheet("NPC_606").copied(),
            ],
""")
open(p, "w", encoding="utf8", newline="").write(box[0])
print("c8 assets.rs done")
