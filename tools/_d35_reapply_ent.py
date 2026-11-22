import sys, os
sys.path.insert(0, os.path.dirname(__file__))
from _d35_model import BATCHES
from _d35_gen import rd, wr, ins, rep, ids, ENT
ks = BATCHES[0]; label='A'
# draw arms
rb=""
for k in ks:
    if k['draw']=='humanoid':
        rb+=f"            NpcKind::{k['v']} => humanoid_draw(assets.npc_sheet({k['id']}), {k['rows']}, n, w, h),\n"
    elif k['draw']=='worm':
        rb+=f"            NpcKind::{k['v']} => generic_anchor(assets.npc_sheet({k['id']}), 1, n, w, h),\n"
    else:
        rb+=f"            NpcKind::{k['v']} => generic_anchor(assets.npc_sheet({k['id']}), {k['rows']}, n, w, h),\n"
ins(ENT, "            NpcKind::DungeonSpirit => {\n                pumpkin_moon_draw(assets.npc_sheet(288), npc_rows(288), n, w, h)\n            }\n        };",
    f"            // W4-d35 ({label}): the {ids(ks)} draw arms. **Sheet deferral**:\n"
    f"            // the NPC sheets are NOT packed (the atlas stands at the 8192\n"
    f"            // strict-GPU bound - the wyvern precedent); npc_sheet returns\n"
    f"            // None and the draw skips. The packing rides the atlas-repack wave.\n"+rb)
rot=[k for k in ks if k['ai']==6 or k.get('rotate') or k['id']==101]
if rot:
    rep(ENT, "                    | NpcKind::WyvernTail\n",
        "                    | NpcKind::WyvernTail\n"
        + f"                    // W4-d35 ({label}): the {ids(rot)} chain/spin rotations.\n"
        + "".join(f"                    | NpcKind::{k['v']}\n" for k in rot))
print("entities re-applied")
