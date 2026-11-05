"""M4-NPC-2 part C: the periphery — netglue, buffs, render, assets, UI,
bestiary, the spawner arms."""
import io
import os

os.chdir('C:/Users/DreadLord/Desktop/Terraria-Rust')

# ---------- 1) netglue ----------
p = 'crates/tr-app/src/app/netglue.rs'
s = io.open(p, encoding='utf-8').read()
old = '''        // The swimmers: the ctor plants the feet on the column surface —
        // the net state overwrites the position below.
        crate::npc::NpcKind::Goldfish | crate::npc::NpcKind::Piranha | crate::npc::NpcKind::Shark => {
            crate::npc::Npc::spawn_cave_walker(world, tx, kind)?
        }'''
assert old in s, 'netglue anchor'
s = s.replace(old, '''        // The swimmers: the ctor plants the feet on the column surface —
        // the net state overwrites the position below.
        crate::npc::NpcKind::Goldfish | crate::npc::NpcKind::Piranha | crate::npc::NpcKind::Shark => {
            crate::npc::Npc::spawn_cave_walker(world, tx, kind)?
        }
        // The worm segments: the net spawns each link standalone (the chain
        // links rebuild through the init — the head; a lone body/tail dies
        // finding no leader — the net-chain sync lands with the netglue
        // worm work).
        crate::npc::NpcKind::DevourerHead
        | crate::npc::NpcKind::DevourerBody
        | crate::npc::NpcKind::DevourerTail
        | crate::npc::NpcKind::GiantWormHead
        | crate::npc::NpcKind::GiantWormBody
        | crate::npc::NpcKind::GiantWormTail
        | crate::npc::NpcKind::EaterOfWorldsHead
        | crate::npc::NpcKind::EaterOfWorldsBody
        | crate::npc::NpcKind::EaterOfWorldsTail => {
            crate::npc::Npc::spawn_cave_walker(world, tx, kind)?
        }''')
io.open(p, 'w', encoding='utf-8', newline='').write(s)
print('netglue ok')

# ---------- 2) npc_buffs ----------
p = 'crates/tr-app/src/npc_buffs.rs'
s = io.open(p, encoding='utf-8').read()
old = '''        // The swimmers: no debuff-immunity rows.
        NpcKind::Goldfish | NpcKind::Piranha | NpcKind::Shark => &[],'''
assert old in s, 'buffs anchor'
s = s.replace(old, '''        // The swimmers: no debuff-immunity rows.
        NpcKind::Goldfish | NpcKind::Piranha | NpcKind::Shark => &[],
        // The worms: no debuff-immunity rows.
        NpcKind::DevourerHead
        | NpcKind::DevourerBody
        | NpcKind::DevourerTail
        | NpcKind::GiantWormHead
        | NpcKind::GiantWormBody
        | NpcKind::GiantWormTail
        | NpcKind::EaterOfWorldsHead
        | NpcKind::EaterOfWorldsBody
        | NpcKind::EaterOfWorldsTail => &[],''')
io.open(p, 'w', encoding='utf-8', newline='').write(s)
print('npc_buffs ok')

# ---------- 3) render/entities: the textures + the draw arms ----------
p = 'crates/tr-app/src/render/entities.rs'
s = io.open(p, encoding='utf-8').read()
old = '''    pub(crate) shark: Option<BgTex>,'''
assert old in s, 'struct anchor'
s = s.replace(old, '''    pub(crate) shark: Option<BgTex>,
    pub(crate) devourer_head: Option<BgTex>,
    pub(crate) devourer_body: Option<BgTex>,
    pub(crate) devourer_tail: Option<BgTex>,
    pub(crate) giant_worm_head: Option<BgTex>,
    pub(crate) giant_worm_body: Option<BgTex>,
    pub(crate) giant_worm_tail: Option<BgTex>,
    pub(crate) eow_head: Option<BgTex>,
    pub(crate) eow_body: Option<BgTex>,
    pub(crate) eow_tail: Option<BgTex>,''')

old = '''        shark: sky_tex(atlas, "NPC_65", None),'''
assert old in s, 'build anchor'
s = s.replace(old, '''        shark: sky_tex(atlas, "NPC_65", None),
        devourer_head: sky_tex(atlas, "NPC_7", None),
        devourer_body: sky_tex(atlas, "NPC_8", None),
        devourer_tail: sky_tex(atlas, "NPC_9", None),
        giant_worm_head: sky_tex(atlas, "NPC_10", None),
        giant_worm_body: sky_tex(atlas, "NPC_11", None),
        giant_worm_tail: sky_tex(atlas, "NPC_12", None),
        eow_head: sky_tex(atlas, "NPC_13", None),
        eow_body: sky_tex(atlas, "NPC_14", None),
        eow_tail: sky_tex(atlas, "NPC_15", None),''')

old = '''            NpcKind::EaterOfSouls | NpcKind::Hornet | NpcKind::MeteorHead => {
                let tex = match n.kind {'''
assert old in s, 'draw anchor'
s = s.replace(old, '''            // The worms: the single-frame segments rotated along the chain
            // (`rotation = atan2 + 1.57`), drawn only in the open — the
            // vanilla `behindTiles` hides them under the ground (the
            // behindTiles layer lands with the render-layers work).
            NpcKind::DevourerHead
            | NpcKind::DevourerBody
            | NpcKind::DevourerTail
            | NpcKind::GiantWormHead
            | NpcKind::GiantWormBody
            | NpcKind::GiantWormTail
            | NpcKind::EaterOfWorldsHead
            | NpcKind::EaterOfWorldsBody
            | NpcKind::EaterOfWorldsTail => {
                let tex = match n.kind {
                    NpcKind::DevourerHead => assets.devourer_head.as_ref(),
                    NpcKind::DevourerBody => assets.devourer_body.as_ref(),
                    NpcKind::DevourerTail => assets.devourer_tail.as_ref(),
                    NpcKind::GiantWormHead => assets.giant_worm_head.as_ref(),
                    NpcKind::GiantWormBody => assets.giant_worm_body.as_ref(),
                    NpcKind::GiantWormTail => assets.giant_worm_tail.as_ref(),
                    NpcKind::EaterOfWorldsHead => assets.eow_head.as_ref(),
                    NpcKind::EaterOfWorldsBody => assets.eow_body.as_ref(),
                    _ => assets.eow_tail.as_ref(),
                };
                let (bw, bh) = n.size();
                (
                    tex,
                    tex.map_or(bh, |t| t.h),
                    [bw + 4.0, bh + 4.0],
                    [n.x - 2.0, n.y - 2.0],
                )
            }
            NpcKind::EaterOfSouls | NpcKind::Hornet | NpcKind::MeteorHead => {
                let tex = match n.kind {''')
io.open(p, 'w', encoding='utf-8', newline='').write(s)
print('entities ok')
print('PART C DONE')
