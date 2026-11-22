# W4-c3 merge-side E8 drops: bestiary.rs / assets.rs / npc_buffs.rs.
import sys, os
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
    # drop c3's two dup ENTRIES defs; keep the 694 def
    rep("""    // W4-c3 (the style-10 family): the three kinds' own entries
    // (`BestiaryDatabaseNPCsPopulator.cs:1112`/`:1559`/`:1561` - the
    // automated enemy registration, `AddEmptyEntries_CrittersAndEnemies_Automated`
    // `:851-868`; none is in the Hide exclusions). The credit ids are the
    // vanilla `NPCID.Search.GetName` rows (`NPCID.cs:10745`/`:11255`/`:12065`).
    BestiaryEntryDef {
        credit_id: "CursedSkull",
        net_id: 34,
        kind: NpcKind::CursedSkull,
        variant: None,
        quick_unlock: false,
        name: "Cursed Skull",
    },
    BestiaryEntryDef {
        credit_id: "GiantCursedSkull",
        net_id: 289,
        kind: NpcKind::GiantCursedSkull,
        variant: None,
        quick_unlock: false,
        name: "Giant Cursed Skull",
    },
    BestiaryEntryDef {
        credit_id: "WaterBoltMimic",
""", """    // W4-c3 (the style-10 family): the WaterBoltMimic's own entry
    // (`BestiaryDatabaseNPCsPopulator.cs:1561` - the automated enemy
    // registration `:851-868`, not in the Hide exclusions; the credit id
    // is the vanilla `NPCID.Search.GetName` row, `NPCID.cs:12065`). The
    // CursedSkull 34 / GiantCursedSkull 289 defs dropped per the d37 E8
    // tables - d34/d38's rows above own them.
    BestiaryEntryDef {
        credit_id: "WaterBoltMimic",
""")

def assets(rep, box):
    # STATIC_SHEET_NAMES: drop the 34/289 dup rows (d34/d38 own), keep 694
    rep("""        // W4-c3 (the style-10 family): the three kinds' own sheets
""", """        // W4-c3: the WaterBoltMimic's own sheet (the 34/289 rows are
        // d34/d38's above - E8 drop; `render/entities.rs`'s STYLE10_SHEETS
        // keeps only NPC_694).
""")
    # find and read what surrounds to drop rows precisely
    src = box[0]
    i = src.index('"NPC_694",')
    # drop the two lines before it if they are the c3 dups
    # handled below via explicit rep of the trio (comment lines vary)

def buffs(rep, box):
    rep("""        // W4-c3 (the style-10 family): 34/289/694 all carry
        // `SpecificallyImmuneTo = { 20, 31 }` (`NPCID.cs:6286-6292`/
        // `:7787-7793`/`:10066-10072`) - Poisoned + On Fire!.
        NpcKind::CursedSkull | NpcKind::GiantCursedSkull | NpcKind::WaterBoltMimic => &[20, 31],
""", """        // W4-c3: the WaterBoltMimic carries `SpecificallyImmuneTo =
        // { 20, 31 }` (`NPCID.cs:10066-10072`) - Poisoned + On Fire!;
        // the 34/289 rows are d34/d38's above (E8 keep-both).
        NpcKind::WaterBoltMimic => &[20, 31],
""")

edit_file("crates/tr-app/src/bestiary.rs", bestiary)
edit_file("crates/tr-app/src/npc_buffs.rs", buffs)
