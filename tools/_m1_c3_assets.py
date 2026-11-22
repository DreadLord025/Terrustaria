# W4-c3 merge-side E8 drops in assets.rs: 34/289 sheet dups drop, 694 keeps;
# BESTIARY_NPC_SHEET_IDS 313 -> 314 (one new slot), parallel vec matched.
import sys, os
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from _m1_rep import make_rep
p = "crates/tr-app/src/assets.rs"
src = open(p, encoding="utf8", newline="").read()
box = [src]
rep = make_rep(lambda: box[0], lambda v: box.__setitem__(0, v))

# STATIC_SHEET_NAMES trio -> 694 only
rep("""        // W4-c3 (the style-10 family): the three kinds' own sheets
        // (`render/entities.rs`'s STYLE10_SHEETS).
        "NPC_34",
        "NPC_289",
        "NPC_694",
""", """        // W4-c3: the WaterBoltMimic's own sheet (the 34/289 rows are
        // d34/d38's above - E8 drop; STYLE10_SHEETS keeps only NPC_694).
        "NPC_694",
""")
# slot decl join: 313 + 1 = 314
rep("""// W4-d34+d38+d37+d40+d35 wave-1 union (367 s1): 181 base + 17 d34 + 35 d38
// + 43 d37 + 21 d40 (minus the dropped CrimsonGoldfish 465 dup - c4 owns
// @465) + 16 d35 (the 71-140 band's visible packed kinds; the 96/97/99/100/
// 112 Hide folds ride their heads) = 313 slots.
pub const BESTIARY_NPC_SHEET_IDS: [u16; 313] = [
pub const BESTIARY_NPC_SHEET_IDS: [u16; 184] = [
""", """// W4-d34+d38+d37+d40+d35 wave-1 union (367 s1): 181 base + 17 d34 + 35 d38
// + 43 d37 + 21 d40 (minus the dropped CrimsonGoldfish 465 dup - c4 owns
// @465) + 16 d35 (the 71-140 band's visible packed kinds; the 96/97/99/100/
// 112 Hide folds ride their heads) = 313 slots; + W4-c3's WaterBoltMimic
// 694 (the 34/289 dups dropped per the d37 E8 tables) = 314.
pub const BESTIARY_NPC_SHEET_IDS: [u16; 314] = [
""")
# slot rows trio -> 694
rep("""        // W4-c3 (the style-10 family) - appended last so the earlier slots
        // keep their indexes.
        34, 289, 694,
""", """        // W4-c3 - appended last so the earlier slots keep their indexes
        // (the 34/289 dups dropped per E8; d34/d38's slots above own them).
        694,
""")
# parallel vec trio -> 694
rep("""                // W4-c3 (the style-10 family) - the three new strips
                // (34/289/694), parallel to BESTIARY_NPC_SHEET_IDS.
                atlas.sheet("NPC_34").copied(),
                atlas.sheet("NPC_289").copied(),
                atlas.sheet("NPC_694").copied(),
""", """                // W4-c3 - the WaterBoltMimic strip, parallel to
                // BESTIARY_NPC_SHEET_IDS (the 34/289 dups dropped per E8).
                atlas.sheet("NPC_694").copied(),
""")
open(p, "w", encoding="utf8", newline="").write(box[0])
print("assets.rs done")
