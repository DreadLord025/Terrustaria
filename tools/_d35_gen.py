import sys, os, re
sys.path.insert(0, os.path.dirname(__file__))
from _d35_model import KINDS, BATCHES
ROOT = r"C:\Users\DreadLord\Desktop\TR-W4-d35"
def rd(p): return open(os.path.join(ROOT,p),encoding='utf-8',newline='').read().replace('\r\n','\n')
def wr(p,s):
    # match the working-tree convention (CRLF, core.autocrlf=true)
    s=s.replace('\r\n','\n').replace('\n','\r\n')
    open(os.path.join(ROOT,p),'w',encoding='utf-8',newline='').write(s)
def ins(path, anchor, block):
    src=rd(path); assert src.count(anchor)==1, f"{path}: anchor x{src.count(anchor)}: {anchor[:70]}"
    wr(path, src.replace(anchor, block+anchor))
def rep(path, old, new, n=1):
    src=rd(path); assert src.count(old)==n, f"{path}: old x{src.count(old)} (want {n}): {old[:70]}"
    wr(path, src.replace(old,new))

NPC=r'crates\tr-app\src\npc.rs'
DATA=r'crates\tr-app\src\npc\data.rs'
BUFFS=r'crates\tr-app\src\npc_buffs.rs'
BEST=r'crates\tr-app\src\bestiary.rs'
NET=r'crates\tr-app\src\app\netglue.rs'
ENT=r'crates\tr-app\src\render\entities.rs'
UIB=r'crates\tr-app\src\ui\bestiary.rs'
AUD=r'crates\tr-app\src\audio.rs'

def grp_ind(ks, ind="            "):
    return ("\n"+ind+"| ").join(f"NpcKind::{k['v']}" for k in ks)
def ids(ks): return "/".join(str(k['id']) for k in ks)

def enum_doc(k):
    kb = k['kb'] if k['kb'] is not None else 1.0
    flags=[]
    if k['noGrav']: flags.append('noGravity')
    if k['noTile']: flags.append('noTileCollide')
    if k['behind']: flags.append('behindTiles')
    if k['netAlways']: flags.append('netAlways')
    fl = (", "+", ".join(flags)) if flags else ""
    extra=[]
    if k.get('npcslots'): extra.append(f"npcSlots {k['npcslots']}")
    if k.get('rarity',0)>1: extra.append(f"rarity {k['rarity']}")
    if k['id']==74: extra.append("catchItem 2015")
    ex = ("; "+"; ".join(extra)) if extra else ""
    fold = f" Bestiary folds onto {k['hide']} (Hide=true)." if k['hide'] else ""
    return (f"    /// `NPCID.{k['v']}` ({k['id']}) - W4-d35 data wave (`NPC.cs:{k['cite']}`):\n"
            f"    /// {k['w']}x{k['h']}, aiStyle {k['ai']}, dmg {k['dmg']}, def {k['dfn']}, {k['life']} HP,\n"
            f"    /// kb {kb}, value {k['value']}{fl}{ex}. UNWIRED (data-only; the AI lands with its wave).{fold}\n"
            f"    {k['v']},\n")

def last_landed(table):
    # The last W4-d35 kind already present in a table (for append-anchors).
    # Walk KINDS in order; the last one whose variant appears is the tail.
    src=rd(table)
    last=None
    for k in KINDS:
        if f'NpcKind::{k["v"]}' in src or (table==NPC and f'    {k["v"]},\n' in src):
            last=k
    return last

def apply_batch(ks, label):
    hidden=[k for k in ks if k['hide']]
    visible=[k for k in ks if not k['hide']]
    # Append-anchors: insert AFTER the last landed kind (the previous batch's
    # tail), not before the closing brace (which shifts as batches land).
    prev_enum=last_landed(NPC)
    enum_anchor=f"    {prev_enum['v']},\n" if prev_enum else "    DungeonSpirit,\n"
    # 1. enum variants (append AFTER the last landed variant)
    rep(NPC, enum_anchor, enum_anchor + "".join(enum_doc(k) for k in ks))
    # 2. to_u16 / from_u16 / render_key (port-local band 429..461)
    ins(DATA, "            NpcKind::WyvernTail => 228,\n",
        f"            // W4-d35 ({label}): the {ids(ks)} band - port-local protocol ids\n"
        f"            // 429.. (the vanilla ids collide with the port-local space;\n"
        f"            // the pirate/wyvern renumber precedent). The vanilla NPC.type\n"
        f"            // rides cs_id/net_id, NOT these protocol ids.\n"
        + "".join(f"            NpcKind::{k['v']} => {k['proto']},\n" for k in ks))
    ins(DATA, "            228 => NpcKind::WyvernTail,\n",
        "".join(f"            {k['proto']} => NpcKind::{k['v']},\n" for k in ks))
    ins(NPC, "            NpcKind::WyvernTail => 228,\n",
        f"            // W4-d35 ({label}): the port-local band (to_u16's ids).\n"
        + "".join(f"            NpcKind::{k['v']} => {k['proto']},\n" for k in ks))
    # 3. ai_style
    by={}
    for k in ks: by.setdefault(k['ai'],[]).append(k)
    ai_block=""
    for ai,g in sorted(by.items()):
        cites=", ".join(f":{k['cite'].split('-')[0]}" for k in g)
        ai_block+=f"            // aiStyle {ai} - W4-d35 ({label}; NPC.cs {cites}).\n"
        if len(g)==1:
            ai_block+=f"            NpcKind::{g[0]['v']} => {ai},\n"
        else:
            ai_block+="            " + grp_ind(g) + f" => {ai},\n"
    ins(DATA, "            NpcKind::DungeonSpirit => 56,\n", ai_block)
    # 4. size / max_hp / contact_damage
    ins(DATA, "            NpcKind::DungeonSpirit => (20.0, 20.0),\n",
        f"            // W4-d35 ({label}): the {ids(ks)} SetDefaults boxes.\n"
        + "".join(f"            NpcKind::{k['v']} => ({k['w']}.0, {k['h']}.0),\n" for k in ks))
    ins(DATA, "            NpcKind::DungeonSpirit => 200,\n",
        f"            // W4-d35 ({label}): the {ids(ks)} lifeMax rows.\n"
        + "".join(f"            NpcKind::{k['v']} => {k['life']},\n" for k in ks))
    ins(DATA, "            NpcKind::DungeonSpirit => 70,\n",
        f"            // W4-d35 ({label}): the {ids(ks)} damage rows.\n"
        + "".join(f"            NpcKind::{k['v']} => {k['dmg']},\n" for k in ks))
    # 5. defense / value / kb (npc.rs)
    ins(NPC, "            NpcKind::DungeonSpirit => 30,\n",
        f"            // W4-d35 ({label}): the {ids(ks)} defense rows.\n"
        + "".join(f"            NpcKind::{k['v']} => {k['dfn']},\n" for k in ks))
    ins(NPC, "            NpcKind::DungeonSpirit => 500.0,\n",
        f"            // W4-d35 ({label}): the {ids(ks)} value rows (copper).\n"
        + "".join(f"            NpcKind::{k['v']} => {k['value']}.0,\n" for k in ks))
    ins(NPC, "            NpcKind::DungeonSpirit => 0.2,\n",
        f"            // W4-d35 ({label}): the {ids(ks)} knockBackResist rows\n"
        f"            // (the no-row kinds keep the head's 1f, NPC.cs:8533).\n"
        + "".join(f"            NpcKind::{k['v']} => {k['kb'] if k['kb'] is not None else 1.0},\n" for k in ks))
    # 6. hit/death sound
    hb="".join(f"            NpcKind::{k['v']} => \"NPC_Hit_{k['hit']}\",\n" for k in ks if k['hit'] is not None)
    db="".join(f"            NpcKind::{k['v']} => \"NPC_Killed_{k['death']}\",\n" for k in ks if k['death'] is not None)
    if hb:
        rep(DATA, "            | NpcKind::WyvernTail => \"NPC_Hit_7\",\n",
            "            | NpcKind::WyvernTail => \"NPC_Hit_7\",\n"
            + f"            // W4-d35 ({label}): the {ids(ks)} HitSound rows.\n"+hb)
    if db:
        rep(DATA, "            | NpcKind::WyvernTail => \"NPC_Killed_8\",\n",
            "            | NpcKind::WyvernTail => \"NPC_Killed_8\",\n"
            + f"            // W4-d35 ({label}): the {ids(ks)} DeathSound rows.\n"+db)
    # 7. cs_id hand rows - VISIBLE kinds only. The Hide kinds keep the
    #    fallback (try_entry folds to the head -> head's net id; the
    #    Wyvern-segment precedent). A hand row here would disagree with the
    #    folded entry's net id (the s1_f_3 pin).
    if visible:
        ins(DATA, "            NpcKind::BlazingWheel => 72,\n",
            f"            // W4-d35 ({label}): the {ids(visible)} vanilla NPC.type rows.\n"
            + "".join(f"            NpcKind::{k['v']} => {k['id']},\n" for k in visible))
    # 8. behind_tiles
    bh=[k for k in ks if k['behind']]
    if bh:
        ins(DATA, "                | NpcKind::BlazingWheel\n",
            f"                // W4-d35 ({label}): the {ids(bh)} behindTiles rows.\n"
            + "".join(f"                | NpcKind::{k['v']}\n" for k in bh))
    # 9. no_gravity (kb>0 flyers only; the kb-0 worms keep false - the
    #    knockback gate refuses resist 0, the WyvernHead-absent precedent)
    ng=[k for k in ks if k['noGrav'] and (k['kb'] is None or k['kb']>0)]
    if ng:
        ins(DATA, "            NpcKind::AngryNimbus => true,\n",
            f"            // W4-d35 ({label}): the {ids(ng)} noGravity flyers (kb>0).\n"
            + "            " + grp_ind(ng) + " => true,\n")
    # 10. shares_carrier_life (worms, head included - the WyvernHead precedent)
    worms=[k for k in ks if k['ai']==6]
    if worms:
        ins(DATA, "                | NpcKind::WyvernTail\n",
            f"                // W4-d35 ({label}): the {ids(worms)} worm chain shares the\n"
            f"                // head's life pool (NPC.cs:51305-51363). Inert until wiring.\n"
            + "".join(f"                | NpcKind::{k['v']}\n" for k in worms))
    # 11. debuff - append after the last landed buffs arm (or DungeonSpirit)
    db_block=""
    for k in ks:
        if k.get('allnonwhip'):
            db_block+=f"// {k['id']}: ImmuneToAllBuffsThatAreNotWhips - unmodelled mask, &[] (Poltergeist precedent).\n"
            db_block+=f"NpcKind::{k['v']} => &[],\n"
        elif k['debuff']:
            db_block+=f"NpcKind::{k['v']} => &[{', '.join(map(str,k['debuff']))}],\n"
        else:
            db_block+=f"NpcKind::{k['v']} => &[],\n"
    prev_buffs=last_landed(BUFFS)
    if prev_buffs:
        # the last landed arm line (find its exact text)
        bsrc=rd(BUFFS)
        m=re.search(r'^        NpcKind::'+prev_buffs['v']+r' => &[^\n]*\n', bsrc, re.M)
        buffs_anchor=m.group(0)
    else:
        buffs_anchor="        | NpcKind::DungeonSpirit => &[],\n"
    ins(BUFFS, buffs_anchor,
        f"        // W4-d35 ({label}): the {ids(ks)} DebuffImmunitySets rows (NPCID.cs).\n"
        + "".join("        "+line+"\n" for line in db_block.rstrip("\n").split("\n")))
    # 12. update_kind_ai no-op + find_frame no-op
    ins(NPC, "            NpcKind::Sharkron | NpcKind::Sharkron2 => self.ai_071_sharkron(world, player),\n",
        f"            // W4-d35 ({label}): the {ids(ks)} kinds are UNWIRED (data-only);\n"
        f"            // the AI lands with its wave (the wyvern no-op precedent).\n"
        + "            " + grp_ind(ks) + " => {}\n")
    ins(NPC, "        }\n    }\n\n    fn update_slime(&mut self, world: &World, player: &mut Player, _dt: f32) {",
        f"            // W4-d35 ({label}): the {ids(ks)} kinds' FindFrame arms are UNWIRED\n"
        f"            // (data-only); the frame rests row 0 until the AI wave.\n"
        + "            " + grp_ind(ks) + " => {}\n")
    # 12b. style-axis guards (data-only): arms 5/9 are claimed unconditionally
    #      and would run AI (or panic) for the new kinds. Guard them so the
    #      batch's style-5/9 kinds fall through to the update_kind_ai no-op
    #      (the MourningWood/Everscream 57-guard precedent).
    g5=[k for k in ks if k['ai']==5]
    g9=[k for k in ks if k['ai']==9]
    if g5:
        names=" | ".join(f"NpcKind::{k['v']}" for k in g5)
        rep(NPC, "            5 => self.ai_005_hover_family(world, player),\n",
            f"            // W4-d35 ({label}): the {ids(g5)} data-only kinds skip the\n"
            f"            // aiStyle-5 family (it would run AI); they fall through to\n"
            f"            // the update_kind_ai no-op until the AI wave.\n"
            f"            5 if !matches!(self.kind, {names}) => self.ai_005_hover_family(world, player),\n")
        # keep ai_005_hover_family's inner match exhaustive (Probe precedent)
        rep(NPC, "            NpcKind::Probe => {}\n            _ => unreachable!(\"only aiStyle-5 kinds reach the family arm\"),",
            "            NpcKind::Probe => {}\n"
            + f"            // W4-d35 ({label}): the {ids(g5)} data-only no-op (the Probe\n"
            f"            // precedent); the axis guard keeps it off the live path.\n"
            + "".join(f"            NpcKind::{k['v']} => {{}}\n" for k in g5)
            + "            _ => unreachable!(\"only aiStyle-5 kinds reach the family arm\"),")
    if g9:
        names=" | ".join(f"NpcKind::{k['v']}" for k in g9)
        rep(NPC, "            9 => self.ai_009_sphere_family(world, player, rng),\n",
            f"            // W4-d35 ({label}): the {ids(g9)} data-only kinds skip the\n"
            f"            // aiStyle-9 family (it would run AI); they fall through to\n"
            f"            // the update_kind_ai no-op until the AI wave.\n"
            f"            9 if !matches!(self.kind, {names}) => self.ai_009_sphere_family(world, player, rng),\n")
    # 13. ctor scale + alpha
    sc=[k for k in ks if k['scale']!=1.0]
    if sc:
        old="                .unwrap_or(if kind == NpcKind::BlazingWheel { 1.2 } else { 1.0 }),"
        new=("                .or_else(|| match kind {\n"
             + "".join(f"                    NpcKind::{k['v']} => Some({k['scale']}),\n" for k in sc)
             + "                    _ => None,\n"
             + "                })\n"
             + "                .unwrap_or(if kind == NpcKind::BlazingWheel { 1.2 } else { 1.0 }),")
        rep(NPC, old, new)
    al=[k for k in ks if k['alpha']!=0]
    if al:
        # Insert before the innermost alpha-match wildcard (28-space `_ => 0,`,
        # unique to the ctor alpha match; robust to prior batches' alpha rows).
        old="                            _ => 0,\n                        }\n                    }),"
        new=("".join(f"                            NpcKind::{k['v']} => {k['alpha']},\n" for k in al)
             + "                            _ => 0,\n                        }\n                    }),")
        rep(NPC, old, new)
    # 14. ALL_KINDS + count
    ins(NPC, "        NpcKind::WyvernTail,\n    ];",
        f"        // W4-d35 ({label}): the {ids(ks)} data-wave kinds.\n"
        + "".join(f"        NpcKind::{k['v']},\n" for k in ks))
    # 15. CS_STYLES
    ins(NPC, "        (NpcKind::WyvernTail, 6),\n    ];",
        f"        // W4-d35 ({label}).\n"
        + "".join(f"        (NpcKind::{k['v']}, {k['ai']}),\n" for k in ks))
    # 15b. CS_STYLES style-5 roster pin (npc.rs s1_d_2 test): the `five` vec
    #      lists every style-5 kind in CS_STYLES order; append the batch's.
    if g5:
        old="                NpcKind::Probe\n            ]\n        );"
        lines=[f"                NpcKind::{k['v']}," for k in g5]
        lines[-1]=lines[-1].rstrip(",")
        new="                NpcKind::Probe,\n" + "\n".join(lines) + "\n            ]\n        );"
        rep(NPC, old, new)
    # 16. bestiary ENTRIES - VISIBLE kinds only (the Hide kinds fold onto the
    #     head's entry, the Wyvern-segment precedent). Append after the last
    #     landed visible entry (or Force Bubble).
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
        if prev_vis:
            best_anchor=f'        name: "{prev_vis[-1]["name"]}",\n    }},\n'
        else:
            best_anchor='        name: "Force Bubble",\n    },\n'
        ins(BEST, best_anchor,
            f"    // W4-d35 ({label}): the {ids(visible)} data-wave entries.\n"+eb)
    # 16b. bestiary folds (BOTH try_entry_for_npc + entry_for_npc): the Hide
    #      kinds fold onto the head (C# GetExclusions, the Wyvern precedent).
    if hidden:
        fold_block=(f"        // W4-d35 ({label}): the {ids(hidden)} Hide=true kinds fold onto\n"
                    f"        // the head's entry (NPCID.cs DrawOffset Hide; the populator's\n"
                    f"        // GetExclusions skips them - the Wyvern-segment precedent).\n"
                    + "".join(f"        NpcKind::{k['v']} => NpcKind::{k['hide']},\n" for k in hidden))
        rep(BEST, "        | NpcKind::WyvernTail => NpcKind::WyvernHead,\n",
            "        | NpcKind::WyvernTail => NpcKind::WyvernHead,\n" + fold_block, n=2)
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
    # 18. render draw_npcs arms (sheets deferred - the wyvern precedent)
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
    # 18b. texture_net_id: the Hide kinds need explicit VANILLA rows because
    #      net_id() folds to the head (the DetonatingBubble/Sharkron2 precedent).
    if hidden:
        ins(ENT, "        NpcKind::ForceBubble => 384,\n        _ => kind.net_id() as u16,",
            f"        // W4-d35 ({label}): the {ids(hidden)} Hide kinds' DRAW sheets are their\n"
            f"        // own NPC_<id> (net_id() folds to the head; the fishron-hazard precedent).\n"
            + "".join(f"        NpcKind::{k['v']} => {k['id']},\n" for k in hidden))
    # 19. rotation whitelist (worms + the spinning weapons + clinger)
    rot=[k for k in ks if k['ai']==6 or k.get('rotate') or k['id']==101]
    if rot:
        rep(ENT, "                    | NpcKind::WyvernTail\n",
            "                    | NpcKind::WyvernTail\n"
            + f"                    // W4-d35 ({label}): the {ids(rot)} chain/spin rotations.\n"
            + "".join(f"                    | NpcKind::{k['v']}\n" for k in rot))
    # 20. ui/bestiary portrait_data (the SetDefaults box; sheet deferred).
    #     ALL kinds get an arm (the Wyvern-segment precedent: WyvernLegs etc.
    #     carry arms even though their entry folds to the head). The match is
    #     exhaustive; the hidden kinds' arms are reached only if a caller
    #     passes their kind directly (the fold target's entry is the norm).
    pb=""
    for k in ks:
        pb+=f"        (NpcKind::{k['v']}, _) => (sheet_i, {k['pf']}.0, {k['w']}.0, {k['pf']}.0, white),\n"
    ins(UIB, "        (NpcKind::Drippler, _) => (sheet_i, 30.0, 28.0, 30.0, white),\n    }\n}",
        f"        // W4-d35 ({label}): the {ids(ks)} portraits rest the SetDefaults box.\n"+pb)
    # 21. ALL_KINDS array length + s1_f_3 decodable-count bump
    src=rd(NPC)
    m=re.search(r'pub\(super\) const ALL_KINDS: \[NpcKind; (\d+)\]', src)
    n=int(m.group(1)); wr(NPC, src.replace(m.group(0), f'pub(super) const ALL_KINDS: [NpcKind; {n+len(ks)}]'))
    src=rd(NPC)
    m=re.search(r'covered \+ sentinel,\n            (\d+),', src)
    n=int(m.group(1)); wr(NPC, src.replace(m.group(0), f'covered + sentinel,\n            {n+len(ks)},'))
    # 22. audio bank rows for the new cues (the assets ship; S1-E-21 wave C)
    hits=sorted({k['hit'] for k in ks if k['hit'] is not None})
    deaths=sorted({k['death'] for k in ks if k['death'] is not None})
    ab=""
    for h in hits:
        if f'("NPC_Hit_{h}",' not in rd(AUD):
            ab+=f'    ("NPC_Hit_{h}", 3),\n'
    for d in deaths:
        if f'("NPC_Killed_{d}",' not in rd(AUD):
            ab+=f'    ("NPC_Killed_{d}", 4),\n'
    if ab:
        ins(AUD, '    ("NPC_Killed_33", 4),\n',
            f"    // W4-d35 ({label}): the {ids(ks)} hit/death cues (the Sounds/NPC_*.xnb\n"
            f"    // assets ship; the S1-E-21 wave-C bank-gap close).\n"+ab)
    # 23. stat-pin witness module (appended at npc.rs end; the smallkinds
    #     witness precedent - every SetDefaults row through the port tables).
    append_witness(ks, label, hidden, visible)
    print(f"batch {label}: core tables done ({len(ks)} kinds: {ids(ks)}; hidden={ids(hidden) if hidden else 'none'})")

def append_witness(ks, label, hidden, visible):
    rows=""
    for k in ks:
        kb = k['kb'] if k['kb'] is not None else 1.0
        rows+=(f"            (NpcKind::{k['v']}, {k['id']}u16, {k['ai']}u8, "
               f"{k['w']}.0, {k['h']}.0, {k['dmg']}, {k['dfn']}, {k['life']}, "
               f"{kb}f32, {k['value']}.0f32, {k['scale']}f32, {k['alpha']}u8),\n")
    # sound pins
    snd=""
    for k in ks:
        h = f'Some("NPC_Hit_{k["hit"]}")' if k['hit'] is not None else 'None'
        d = f'Some("NPC_Killed_{k["death"]}")' if k['death'] is not None else 'None'
        snd+=(f'        assert_eq!(NpcKind::{k["v"]}.hit_sound(), {h}, "{k["v"]} hit");\n'
              f'        assert_eq!(NpcKind::{k["v"]}.death_sound(), {d}, "{k["v"]} death");\n')
    # protocol round-trip pins
    proto=""
    for k in ks:
        proto+=(f'        assert_eq!(NpcKind::{k["v"]}.to_u16(), {k["proto"]}, "{k["v"]} proto");\n'
                f'        assert_eq!(NpcKind::from_u16({k["proto"]}), Some(NpcKind::{k["v"]}), "{k["v"]} decode");\n')
    # debuff pins (for_kind -> buff_immune; the derived 20=>30+375 rule)
    deb=""
    for k in ks:
        if k.get('allnonwhip'):
            deb+=(f'        // {k["v"]}: ImmuneToAllBuffsThatAreNotWhips is unmodelled (&[]).\n'
                  f'        let b = NpcBuffs::for_kind(NpcKind::{k["v"]});\n'
                  f'        assert!(b.buff_immune.iter().all(|&i| !i), "{k["v"]} unmodelled mask");\n')
        elif k['debuff']:
            deb+=f'        let b = NpcBuffs::for_kind(NpcKind::{k["v"]});\n'
            for id in k['debuff']:
                deb+=f'        assert!(b.buff_immune[{id}], "{k["v"]} immune {id}");\n'
            if 20 in k['debuff']:
                deb+=f'        assert!(b.buff_immune[30] && b.buff_immune[375], "{k["v"]} derived 20");\n'
        else:
            deb+=(f'        let b = NpcBuffs::for_kind(NpcKind::{k["v"]});\n'
                  f'        assert!(b.buff_immune.iter().all(|&i| !i), "{k["v"]} null entry");\n')
    # flag pins (behind_tiles / no_gravity static tables)
    flags=""
    for k in ks:
        flags+=(f'        assert_eq!(NpcKind::{k["v"]}.behind_tiles(), {str(k["behind"]).lower()}, "{k["v"]} behindTiles");\n')
        ng = k['noGrav'] and (k['kb'] is None or k['kb']>0)
        flags+=f'        assert_eq!(NpcKind::{k["v"]}.no_gravity(), {str(ng).lower()}, "{k["v"]} noGravity");\n'
    # bestiary pins: visible own entry; hidden fold to head
    bes=""
    for k in visible:
        bes+=(f'        let e = crate::bestiary::entry_for_npc(NpcKind::{k["v"]}, SlimeVariant::Blue);\n'
              f'        assert_eq!(e.net_id, {k["id"]}, "{k["v"]} net");\n'
              f'        assert_eq!(e.credit_id, "{k["v"]}", "{k["v"]} credit");\n'
              f'        assert_eq!(e.name, "{k["name"]}", "{k["v"]} name");\n'
              f'        assert_eq!(NpcKind::{k["v"]}.cs_id(), {k["id"]}, "{k["v"]} cs_id");\n')
    for k in hidden:
        head=k['hide']
        bes+=(f'        // {k["v"]} is Hide=true: folds onto {head} (no own entry/cs_id hand row).\n'
              f'        let e = crate::bestiary::entry_for_npc(NpcKind::{k["v"]}, SlimeVariant::Blue);\n'
              f'        assert_eq!(e.kind, NpcKind::{head}, "{k["v"]} folds to {head}");\n'
              f'        assert_eq!(NpcKind::{k["v"]}.net_id(), e.net_id, "{k["v"]} net reads the fold");\n'
              f'        assert_eq!(crate::render::entities::texture_net_id(NpcKind::{k["v"]}), {k["id"]}, "{k["v"]} draw sheet stays own");\n')
    cites=", ".join(f"`NPC.cs:{k['cite']}`" for k in ks)
    mod=(f"\n#[cfg(test)]\nmod w4_d35_{label.lower()}_tests {{\n"
         f"    //! W4-d35 batch {label} ({ids(ks)}) - the data-only SetDefaults wave.\n"
         f"    //! Every row pins verbatim to the C# ({cites}). WRITTEN, NOT RUN\n"
         f"    //! (code-only mode); the smallkinds-witness precedent.\n"
         f"    use super::*;\n\n"
         f"    #[test]\n"
         f"    fn w4_d35_{label.lower()}_the_setdefaults_rows_match_the_csharp() {{\n"
         f"        // (kind, vanilla, aiStyle, w, h, dmg, def, life, kb, value, scale, alpha)\n"
         f"        let rows = [\n{rows}        ];\n"
         f"        for (kind, ty, ai, w, h, dmg, def, life, kb, value, scale, alpha) in rows {{\n"
         f"            let shell = Npc::new(kind, 0.0, 0.0, 1, 0.0);\n"
         f"            // cs_id: visible kinds ride their own vanilla id; the Hide\n"
         f"            // kinds fold to the head (the fallback reads the head entry).\n"
         f"            if crate::bestiary::has_entry(kind, SlimeVariant::Blue) {{\n"
         f"                assert_eq!(kind.cs_id(), ty, \"{{kind:?}} vanilla type\");\n"
         f"            }}\n"
         f"            assert_eq!(kind.ai_style(), ai, \"{{kind:?}} aiStyle\");\n"
         f"            assert_eq!(kind.size(), (w, h), \"{{kind:?}} size\");\n"
         f"            assert_eq!(shell.size(), (w, h), \"{{kind:?}} npc size\");\n"
         f"            assert_eq!(kind.contact_damage(), dmg, \"{{kind:?}} damage\");\n"
         f"            assert_eq!(shell.defense(), def, \"{{kind:?}} defense\");\n"
         f"            assert_eq!(kind.max_hp(), life, \"{{kind:?}} lifeMax\");\n"
         f"            assert_eq!(shell.hp, life, \"{{kind:?}} spawn hp\");\n"
         f"            assert_eq!(shell.knock_back_resist(), kb, \"{{kind:?}} kb\");\n"
         f"            assert_eq!(shell.value(), value, \"{{kind:?}} value\");\n"
         f"            assert_eq!(shell.scale, scale, \"{{kind:?}} scale\");\n"
         f"            assert_eq!(shell.alpha, alpha, \"{{kind:?}} alpha\");\n"
         f"        }}\n"
         f"        // sounds\n{snd}"
         f"        // protocol band (port-local 429..; the vanilla ids collide)\n{proto}"
         f"        // flags\n{flags}"
         f"        // debuff immunities (NPCID.cs DebuffImmunitySets)\n{deb}"
         f"        // bestiary entries / Hide folds\n{bes}"
         f"    }}\n}}\n")
    src=rd(NPC)
    wr(NPC, src+mod)

if __name__ == "__main__":
    import argparse
    ap=argparse.ArgumentParser(); ap.add_argument('batch',type=int); a=ap.parse_args()
    apply_batch(BATCHES[a.batch], chr(ord('A')+a.batch))
