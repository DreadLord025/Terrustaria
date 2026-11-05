"""M4-NPC-2 part D: assets, ui/bestiary, bestiary ENTRIES, the spawner arms."""
import io
import os

os.chdir('C:/Users/DreadLord/Desktop/Terraria-Rust')

# ---------- 1) assets.rs ----------
p = 'crates/tr-app/src/assets.rs'
s = io.open(p, encoding='utf-8').read()
old = '''        "NPC_55",
        "NPC_58",
        "NPC_65",'''
assert old in s, 'atlas list anchor'
s = s.replace(old, '''        "NPC_55",
        "NPC_58",
        "NPC_65",
        "NPC_7",
        "NPC_8",
        "NPC_9",
        "NPC_10",
        "NPC_11",
        "NPC_12",
        "NPC_13",
        "NPC_14",
        "NPC_15",''')

old = '                atlas.sheet("NPC_65").copied(),'
assert old in s, 'sheets table anchor'
s = s.replace(old, '''                atlas.sheet("NPC_65").copied(),
                atlas.sheet("NPC_7").copied(),
                atlas.sheet("NPC_8").copied(),
                atlas.sheet("NPC_9").copied(),
                atlas.sheet("NPC_10").copied(),
                atlas.sheet("NPC_11").copied(),
                atlas.sheet("NPC_12").copied(),
                atlas.sheet("NPC_13").copied(),
                atlas.sheet("NPC_14").copied(),
                atlas.sheet("NPC_15").copied(),''')

old = 'pub bestiary_npc_sheets: [Option<tr_render::SheetRect>; 25],'
assert old in s, 'sheets len anchor'
s = s.replace(old, 'pub bestiary_npc_sheets: [Option<tr_render::SheetRect>; 34],')

old = '''    /// sphere, goldfish, piranha, shark]`.'''
assert old in s, 'sheets doc anchor'
s = s.replace(old, '''    /// sphere, goldfish, piranha, shark, the Devourer's three, the Giant
    /// Worm's three, the Eater of Worlds' three]`.''')
io.open(p, 'w', encoding='utf-8', newline='').write(s)
print('assets ok')

# ---------- 2) ui/bestiary ----------
p = 'crates/tr-app/src/ui/bestiary.rs'
s = io.open(p, encoding='utf-8').read()
old = '''        NpcKind::Shark => 24,
    };'''
assert old in s, 'sheet_i anchor'
s = s.replace(old, '''        NpcKind::Shark => 24,
        NpcKind::DevourerHead => 25,
        NpcKind::DevourerBody => 26,
        NpcKind::DevourerTail => 27,
        NpcKind::GiantWormHead => 28,
        NpcKind::GiantWormBody => 29,
        NpcKind::GiantWormTail => 30,
        NpcKind::EaterOfWorldsHead => 31,
        NpcKind::EaterOfWorldsBody => 32,
        NpcKind::EaterOfWorldsTail => 33,
    };''')

old = '        (NpcKind::Shark, _) => (sheet_i, 24.0, 100.0, 24.0, white),'
assert old in s, 'portrait anchor'
s = s.replace(old, '''        (NpcKind::Shark, _) => (sheet_i, 24.0, 100.0, 24.0, white),
        // The worm segments: the square sheets.
        (NpcKind::DevourerHead, _) => (sheet_i, 22.0, 22.0, 22.0, white),
        (NpcKind::DevourerBody, _) => (sheet_i, 22.0, 22.0, 22.0, white),
        (NpcKind::DevourerTail, _) => (sheet_i, 22.0, 22.0, 22.0, white),
        (NpcKind::GiantWormHead, _) => (sheet_i, 14.0, 14.0, 14.0, white),
        (NpcKind::GiantWormBody, _) => (sheet_i, 14.0, 14.0, 14.0, white),
        (NpcKind::GiantWormTail, _) => (sheet_i, 14.0, 14.0, 14.0, white),
        (NpcKind::EaterOfWorldsHead, _) => (sheet_i, 38.0, 38.0, 38.0, white),
        (NpcKind::EaterOfWorldsBody, _) => (sheet_i, 38.0, 38.0, 38.0, white),
        (NpcKind::EaterOfWorldsTail, _) => (sheet_i, 38.0, 38.0, 38.0, white),''')
io.open(p, 'w', encoding='utf-8', newline='').write(s)
print('ui/bestiary ok')

# ---------- 3) bestiary ENTRIES ----------
p = 'crates/tr-app/src/bestiary.rs'
s = io.open(p, encoding='utf-8').read()
old = '''    BestiaryEntryDef {
        credit_id: "Shark",
        net_id: 65,
        kind: NpcKind::Shark,
        variant: None,
        quick_unlock: false,
        name: "Shark",
    },
];'''
assert old in s, 'entries tail anchor'
s = s.replace(old, '''    BestiaryEntryDef {
        credit_id: "Shark",
        net_id: 65,
        kind: NpcKind::Shark,
        variant: None,
        quick_unlock: false,
        name: "Shark",
    },
    BestiaryEntryDef {
        credit_id: "DevourerHead",
        net_id: 7,
        kind: NpcKind::DevourerHead,
        variant: None,
        quick_unlock: false,
        name: "Devourer",
    },
    // The body/tail segments share the head's bestiary entry (the vanilla
    // credit rides the head id through `IsNPCValidForBestiaryKillCredit`'s
    // worm mapping; the port credits each segment's own id and the entries
    // fold at the UI level — the body/tail defs stay here so the kill
    // credits resolve).
    BestiaryEntryDef {
        credit_id: "DevourerHead",
        net_id: 7,
        kind: NpcKind::DevourerBody,
        variant: None,
        quick_unlock: false,
        name: "Devourer",
    },
    BestiaryEntryDef {
        credit_id: "DevourerHead",
        net_id: 7,
        kind: NpcKind::DevourerTail,
        variant: None,
        quick_unlock: false,
        name: "Devourer",
    },
    BestiaryEntryDef {
        credit_id: "GiantWormHead",
        net_id: 10,
        kind: NpcKind::GiantWormHead,
        variant: None,
        quick_unlock: false,
        name: "Giant Worm",
    },
    BestiaryEntryDef {
        credit_id: "GiantWormHead",
        net_id: 10,
        kind: NpcKind::GiantWormBody,
        variant: None,
        quick_unlock: false,
        name: "Giant Worm",
    },
    BestiaryEntryDef {
        credit_id: "GiantWormHead",
        net_id: 10,
        kind: NpcKind::GiantWormTail,
        variant: None,
        quick_unlock: false,
        name: "Giant Worm",
    },
    BestiaryEntryDef {
        credit_id: "EaterofWorldsHead",
        net_id: 13,
        kind: NpcKind::EaterOfWorldsHead,
        variant: None,
        quick_unlock: false,
        name: "Eater of Worlds",
    },
    BestiaryEntryDef {
        credit_id: "EaterofWorldsHead",
        net_id: 13,
        kind: NpcKind::EaterOfWorldsBody,
        variant: None,
        quick_unlock: false,
        name: "Eater of Worlds",
    },
    BestiaryEntryDef {
        credit_id: "EaterofWorldsHead",
        net_id: 13,
        kind: NpcKind::EaterOfWorldsTail,
        variant: None,
        quick_unlock: false,
        name: "Eater of Worlds",
    },
];''')
io.open(p, 'w', encoding='utf-8', newline='').write(s)
print('bestiary ok')

# ---------- 4) the spawner arms ----------
p = 'crates/tr-app/src/spawner.rs'
s = io.open(p, encoding='utf-8').read()
old = '''    // The Tim: the deep-cavern roll'''
assert old in s, 'spawner anchor'
s = s.replace(old, '''    // The Devourer: the corruption-cavern roll (`NPC.cs:3701-3710`:
    // `ZoneCorrupt && Next(maxValue) == 0` with the `maxValue = 65`
    // (`:1286`); `CanSpawnDevourer` is the custom-seed gate — true in the
    // port; the hardmode 98/99 split waits for the hardmode worm pass).
    if ctx.flags.zone_evil && rng.next(65) == 0 {
        return Some(Npc::spawn_worm_head(ctx.world, x, y / 16, NpcKind::DevourerHead));
    }
    // The Giant Worm: the cavern roll (`NPC.cs:4815-4832`:
    // `underGround && !noWorms && Next(50) == 0 && !ZoneSnow` — the port
    // approximates the snow gate with the snow-floor tiles 147/161).
    if !matches!(tile_kind, 147 | 161) && rng.next(50) == 0 {
        return Some(Npc::spawn_worm_head(ctx.world, x, y / 16, NpcKind::GiantWormHead));
    }
    // The Tim: the deep-cavern roll''')
io.open(p, 'w', encoding='utf-8', newline='').write(s)
print('spawner arms ok')
print('PART D DONE')
