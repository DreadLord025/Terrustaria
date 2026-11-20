import sys, os, re
sys.path.insert(0, os.path.dirname(__file__))
from _d35_model import KINDS, BATCHES
from _d35_gen import (rd, wr, ins, rep, ids, grp_ind, NPC, DATA, BUFFS, BEST,
                      NET, ENT, UIB, AUD, append_witness)
ks = BATCHES[4]; label='E'
hidden=[k for k in ks if k['hide']]; visible=[k for k in ks if not k['hide']]
g5=[k for k in ks if k['ai']==5]; g9=[k for k in ks if k['ai']==9]

# 13. ctor scale + alpha (scale: none !=1.0 in E; alpha: GreenJellyfish 20)
sc=[k for k in ks if k['scale']!=1.0]
assert not sc, 'E has no scale rows'
al=[k for k in ks if k['alpha']!=0]
if al:
    old="                            _ => 0,\n                        }\n                    }),"
    new=("".join(f"                            NpcKind::{k['v']} => {k['alpha']},\n" for k in al)
         + "                            _ => 0,\n                        }\n                    }),")
    rep(NPC, old, new)
# 14. ALL_KINDS
ins(NPC, "        NpcKind::WyvernTail,\n    ];",
    f"        // W4-d35 ({label}): the {ids(ks)} data-wave kinds.\n"
    + "".join(f"        NpcKind::{k['v']},\n" for k in ks))
# 15. CS_STYLES
ins(NPC, "        (NpcKind::WyvernTail, 6),\n    ];",
    f"        // W4-d35 ({label}).\n"
    + "".join(f"        (NpcKind::{k['v']}, {k['ai']}),\n" for k in ks))
# 16. bestiary ENTRIES (visible only)
if visible:
    eb=""
    for k in visible:
        eb+=("    BestiaryEntryDef {\n"
             f"        credit_id: \"{k['v']}\",\n"
             f"        net_id: {k['id']},\n"
             f"        kind: NpcKind::{k['v']},\n"
             "        variant: None,\n"
             "        quick_unlock: false,\n"
             f"        name: \"{k['name']}\",\n"
             "    },\n")
    prev_vis=[k for k in KINDS if not k['hide'] and f'kind: NpcKind::{k["v"]},' in rd(BEST)]
    best_anchor=f'        name: "{prev_vis[-1]["name"]}",\n    }},\n' if prev_vis else '        name: "Force Bubble",\n    },\n'
    ins(BEST, best_anchor, f"    // W4-d35 ({label}): the {ids(visible)} data-wave entries.\n"+eb)
# 17. netglue npc_seed
walkers=[k for k in ks if k['seed']=='walker']
flyers=[k for k in ks if k['seed']=='hover']
nb=""
if walkers:
    nb+=("        // W4-d35 ("+label+"): the "+ids(walkers)+" walkers plant feet on the\n"
         "        // column surface (the generic surface ctor; the family machine\n"
         "        // takes over on the first tick).\n"
         + "        " + "\n        | ".join(f"crate::npc::NpcKind::{k['v']}" for k in walkers)
         + " => crate::npc::Npc::spawn_cave_walker(world, tx, kind)?,\n")
if flyers:
    nb+=("        // W4-d35 ("+label+"): the "+ids(flyers)+" flyers ride the hover ctor\n"
         "        // (the net state overwrites the position below).\n"
         + "        " + "\n        | ".join(f"crate::npc::NpcKind::{k['v']}" for k in flyers)
         + " => {\n            let y = (y / TILE_PX) as i32;\n"
         "            crate::npc::Npc::spawn_hover(world, tx, y, kind)?\n        }\n")
ins(NET, "    })\n}\n\nfn npc_from_net", nb)
# 18. draw arms
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
    f"            // NPC_{ids(ks)} are NOT packed (the atlas stands at the 8192\n"
    f"            // strict-GPU bound - the wyvern precedent); npc_sheet returns\n"
    f"            // None and the draw skips. The packing rides the atlas-repack wave.\n"+rb)
# 19. rotation whitelist (Clinger 101)
rot=[k for k in ks if k['ai']==6 or k.get('rotate') or k['id']==101]
if rot:
    rep(ENT, "                    | NpcKind::WyvernTail\n",
        "                    | NpcKind::WyvernTail\n"
        + f"                    // W4-d35 ({label}): the {ids(rot)} chain/spin rotations.\n"
        + "".join(f"                    | NpcKind::{k['v']}\n" for k in rot))
# 20. portrait_data
pb=""
for k in ks:
    pb+=f"        (NpcKind::{k['v']}, _) => (sheet_i, {k['pf']}.0, {k['w']}.0, {k['pf']}.0, white),\n"
ins(UIB, "        (NpcKind::Drippler, _) => (sheet_i, 30.0, 28.0, 30.0, white),\n    }\n}",
    f"        // W4-d35 ({label}): the {ids(ks)} portraits rest the SetDefaults box.\n"+pb)
# 21. counts
src=rd(NPC)
m=re.search(r'pub\(super\) const ALL_KINDS: \[NpcKind; (\d+)\]', src)
n=int(m.group(1)); wr(NPC, src.replace(m.group(0), f'pub(super) const ALL_KINDS: [NpcKind; {n+len(ks)}]'))
src=rd(NPC)
m=re.search(r'covered \+ sentinel,\n            (\d+),', src)
n=int(m.group(1)); wr(NPC, src.replace(m.group(0), f'covered + sentinel,\n            {n+len(ks)},'))
# 22. audio
hits=sorted({k['hit'] for k in ks if k['hit'] is not None})
deaths=sorted({k['death'] for k in ks if k['death'] is not None})
ab=""
for h in hits:
    if f'("NPC_Hit_{h}",' not in rd(AUD): ab+=f'    ("NPC_Hit_{h}", 3),\n'
for d in deaths:
    if f'("NPC_Killed_{d}",' not in rd(AUD): ab+=f'    ("NPC_Killed_{d}", 4),\n'
if ab:
    ins(AUD, '    ("NPC_Killed_33", 4),\n',
        f"    // W4-d35 ({label}): the {ids(ks)} hit/death cues (the Sounds/NPC_*.xnb\n"
        f"    // assets ship; the S1-E-21 wave-C bank-gap close).\n"+ab)
# 23. witness
append_witness(ks, label, hidden, visible)
print(f"batch {label} continuation done (steps 13-23)")
