import sys, os
sys.path.insert(0, os.path.dirname(__file__))
from _d35_gen import rd, NPC, DATA, BUFFS, BEST, NET, ENT, UIB, AUD

anchors = {
    NPC: [
        '    DungeonSpirit,\n}',
        '            NpcKind::WyvernTail => 228,',
        '            NpcKind::DungeonSpirit => 30,',
        '            NpcKind::DungeonSpirit => 500.0,',
        '            NpcKind::DungeonSpirit => 0.2,',
        '            NpcKind::Sharkron | NpcKind::Sharkron2 => self.ai_071_sharkron(world, player),',
        '        }\n    }\n\n    fn update_slime(&mut self, world: &World, player: &mut Player, _dt: f32) {',
        '                .unwrap_or(if kind == NpcKind::BlazingWheel { 1.2 } else { 1.0 }),',
        '        NpcKind::WyvernTail,\n    ];',
        '        (NpcKind::WyvernTail, 6),\n    ];',
        '                            NpcKind::DetonatingBubble | NpcKind::Sharkron | NpcKind::Sharkron2 => {\n                                255\n                            }\n                            _ => 0,',
        '        NpcKind::WyvernTail,\n    ];',
    ],
    DATA: [
        '            NpcKind::WyvernTail => 228,',
        '            228 => NpcKind::WyvernTail,',
        '            NpcKind::DungeonSpirit => 56,',
        '            NpcKind::DungeonSpirit => (20.0, 20.0),',
        '            NpcKind::DungeonSpirit => 200,',
        '            NpcKind::DungeonSpirit => 70,',
        '            | NpcKind::WyvernTail => "NPC_Hit_7",',
        '            | NpcKind::WyvernTail => "NPC_Killed_8",',
        '            NpcKind::BlazingWheel => 72,',
        '                | NpcKind::BlazingWheel\n',
        '            NpcKind::AngryNimbus => true,',
        '                | NpcKind::WyvernTail\n',
    ],
    BUFFS: ['        | NpcKind::DungeonSpirit => &[],\n    }\n}'],
    BEST: ['        name: "Force Bubble",\n    },\n];'],
    NET: ['    })\n}\n\nfn npc_from_net'],
    ENT: [
        '            NpcKind::DungeonSpirit => {\n                pumpkin_moon_draw(assets.npc_sheet(288), npc_rows(288), n, w, h)\n            }\n        };',
        '                    | NpcKind::WyvernTail\n',
    ],
    UIB: ['        (NpcKind::Drippler, _) => (sheet_i, 30.0, 28.0, 30.0, white),\n    }\n}'],
    AUD: ['    ("NPC_Killed_33", 4),\n'],
}
bad = 0
for f, aa in anchors.items():
    src = rd(f)
    for a in dict.fromkeys(aa):
        c = src.count(a)
        if c != 1:
            print('BAD', f, repr(a[:70]), 'count', c)
            bad += 1
print('anchor check done, bad =', bad)
