import sys, os
sys.path.insert(0, os.path.dirname(__file__))
from _d35_gen import rd, NPC, DATA, BUFFS, BEST, NET, ENT, UIB, AUD
anchors={
 DATA:['            NpcKind::WyvernTail => 228,\n','            228 => NpcKind::WyvernTail,\n','            NpcKind::DungeonSpirit => 56,\n','            NpcKind::DungeonSpirit => (20.0, 20.0),\n','            NpcKind::DungeonSpirit => 200,\n','            NpcKind::DungeonSpirit => 70,\n','            | NpcKind::WyvernTail => "NPC_Hit_7",\n','            | NpcKind::WyvernTail => "NPC_Killed_8",\n','            NpcKind::BlazingWheel => 72,\n','                | NpcKind::BlazingWheel\n','            NpcKind::AngryNimbus => true,\n','                | NpcKind::WyvernTail\n'],
 NPC:['            NpcKind::WyvernTail => 228,\n','            NpcKind::DungeonSpirit => 30,\n','            NpcKind::DungeonSpirit => 500.0,\n','            NpcKind::DungeonSpirit => 0.2,\n','            NpcKind::Sharkron | NpcKind::Sharkron2 => self.ai_071_sharkron(world, player),\n','        }\n    }\n\n    fn update_slime(&mut self, world: &World, player: &mut Player, _dt: f32) {','                .unwrap_or(if kind == NpcKind::BlazingWheel { 1.2 } else { 1.0 }),','        NpcKind::WyvernTail,\n    ];','        (NpcKind::WyvernTail, 6),\n    ];','                            NpcKind::DetonatingBubble | NpcKind::Sharkron | NpcKind::Sharkron2 => {\n                                255\n                            }\n                            _ => 0,'],
 NET:['    })\n}\n\nfn npc_from_net'],
 ENT:['            NpcKind::DungeonSpirit => {\n                pumpkin_moon_draw(assets.npc_sheet(288), npc_rows(288), n, w, h)\n            }\n        };','                    | NpcKind::WyvernTail\n','        NpcKind::ForceBubble => 384,\n        _ => kind.net_id() as u16,'],
 UIB:['        (NpcKind::Drippler, _) => (sheet_i, 30.0, 28.0, 30.0, white),\n    }\n}'],
 AUD:['    ("NPC_Killed_33", 4),\n'],
 BEST:['        | NpcKind::WyvernTail => NpcKind::WyvernHead,\n'],
}
bad=0
for f,aa in anchors.items():
    src=rd(f)
    for a in dict.fromkeys(aa):
        c=src.count(a)
        if c!=1: print('BAD',f,repr(a[:60]),'count',c); bad+=1
print('insert-anchor bad =',bad)
