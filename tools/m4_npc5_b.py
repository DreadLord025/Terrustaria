"""M4-NPC-5 part B: the periphery — the world-sim sphere resolution, the
spawner arms (Tim + Fire Imp) with the live y-unit fix, the netglue, the
buff immunity rows, the render textures/draw arms, the atlas, the bestiary,
the UI sheet index."""
import io
import os

os.chdir('C:/Users/DreadLord/Desktop/Terraria-Rust')

# ---------- 1) world_sim/npcs.rs: the sphere resolution ----------
p = 'crates/tr-app/src/world_sim/npcs.rs'
s = io.open(p, encoding='utf-8').read()

old = '''    let mut minions = app.npcs.iter().filter(|n| n.kind == NpcKind::Slime).count();
    let mut spawned = Vec::new();
    for n in app.npcs.iter_mut() {
        if n.wants_minion {'''
assert old in s, 'minion block anchor'
s = s.replace(old, '''    // The casters' sphere spawn requests (`AI_008`'s `NPC.NewNPC` at the
    // shoot tick, `NPC.cs:21238`+): the update pass set `wants_sphere`,
    // this pass resolves it — the same borrow-safe shape as the minion
    // block below.
    let mut spheres = Vec::new();
    for n in app.npcs.iter_mut() {
        if let Some((kind, sx, sy)) = n.wants_sphere.take() {
            // `NPC.NewNPC`'s `Bottom = (X, Y)` (`NPC.cs:80771`): the payload
            // is the sphere's bottom-centre.
            let (w, h) = kind.size();
            let mut ball = Npc::new(kind, sx - w * 0.5, sy - h, 1, 0.0);
            ball.hp = ball.max_hp;
            spheres.push(ball);
        }
    }
    app.npcs.append(&mut spheres);

    let mut minions = app.npcs.iter().filter(|n| n.kind == NpcKind::Slime).count();
    let mut spawned = Vec::new();
    for n in app.npcs.iter_mut() {
        if n.wants_minion {''')
io.open(p, 'w', encoding='utf-8', newline='').write(s)
print('world_sim/npcs.rs ok')

# ---------- 2) spawner.rs: the y-unit fix + the arms ----------
p = 'crates/tr-app/src/spawner.rs'
s = io.open(p, encoding='utf-8').read()

old = '''        // `SpawnAnNPC` default selection for the implemented kinds.
        return spawn_npc_at(ctx, rng, num, j);'''
assert old in s, 'live call anchor'
s = s.replace(old, '''        // `SpawnAnNPC` default selection for the implemented kinds. `y`
        // converts to pixels at the boundary (`spawn_npc_at`'s `y` is
        // pixels — the `Main.worldSurface` comparisons run on the tile
        // coordinate through `y / 16`).
        return spawn_npc_at(ctx, rng, num, j * 16);''')

old = '''    // The Undead Miner: `Next(20) == 0 → 44` (`:5060` region — the rare
    // cave miner).'''
assert old in s, 'miner arm anchor'
s = s.replace(old, '''    // The Tim: the deep-cavern roll (`NPC.cs:5014`): `(remixWorld ||
    // spawnTileY > (rockLayer + maxTilesY) / 2.0) && (Next(200) == 0 ||
    // (offensiveToTim && Next(50) == 0)) → 45`. The port gates the depth
    // alone — the offensiveToTim accessory rows (`NPC.cs:418`, the
    // M3-EQUIP-3 work) stay constants-off, so the Next(50) never runs.
    if (y as f64 / 16.0) > (ctx.world.rock_layer + ctx.world.height as f64) / 2.0
        && rng.next(200) == 0
    {
        return Npc::spawn_caster(ctx.world, x, y / 16, NpcKind::Tim);
    }
    // The underworld ladder (`NPC.cs:4869-4913`, the `spawnTileY >
    // maxTilesY - 190` branch that precedes the cavern arms): the
    // lava-critter Next(8) and Demon Next(40) arms are unmodelled
    // (constants-off, consume nothing); the Fire Imp `Next(14) == 0 → 24`
    // (`:4891-4893`) is the port's first underworld caster arm. The gate
    // rides the scene-centre flag the Hellbat arm uses.
    if ctx.underworld_for_bats && rng.next(14) == 0 {
        return Npc::spawn_caster(ctx.world, x, y / 16, NpcKind::FireImp);
    }
    // The Undead Miner: `Next(20) == 0 → 44` (`:5060` region — the rare
    // cave miner).''')
io.open(p, 'w', encoding='utf-8', newline='').write(s)
print('spawner.rs ok')

# ---------- 3) netglue.rs ----------
p = 'crates/tr-app/src/app/netglue.rs'
s = io.open(p, encoding='utf-8').read()

old = '''        crate::npc::NpcKind::EaterOfSouls | crate::npc::NpcKind::Hornet | crate::npc::NpcKind::MeteorHead => {
            let y = (n.y / TILE_PX) as i32;
            crate::npc::Npc::spawn_hover(world, tx, y, kind)?
        }'''
assert old in s, 'netglue hover arm'
s = s.replace(old, '''        crate::npc::NpcKind::EaterOfSouls | crate::npc::NpcKind::Hornet | crate::npc::NpcKind::MeteorHead => {
            let y = (n.y / TILE_PX) as i32;
            crate::npc::Npc::spawn_hover(world, tx, y, kind)?
        }
        // The casters/spheres: the ctor only seeds the stats — the net
        // state overwrites the position below (the hover arm's shape).
        crate::npc::NpcKind::GoblinSorcerer
        | crate::npc::NpcKind::Tim
        | crate::npc::NpcKind::DarkCaster
        | crate::npc::NpcKind::FireImp
        | crate::npc::NpcKind::ChaosBall
        | crate::npc::NpcKind::BurningSphere
        | crate::npc::NpcKind::WaterSphere => {
            let y = (n.y / TILE_PX) as i32;
            crate::npc::Npc::spawn_hover(world, tx, y, kind)?
        }''')
io.open(p, 'w', encoding='utf-8', newline='').write(s)
print('netglue.rs ok')

# ---------- 4) npc_buffs.rs ----------
p = 'crates/tr-app/src/npc_buffs.rs'
s = io.open(p, encoding='utf-8').read()

old = '        NpcKind::Hornet => &[20],'
assert old in s, 'buffs anchor'
s = s.replace(old, '''        NpcKind::Hornet => &[20],
        // The casters/spheres: no debuff-immunity rows in the data (the
        // Fire Imp's `lavaImmune` rides the liquid system).
        NpcKind::GoblinSorcerer | NpcKind::Tim | NpcKind::DarkCaster | NpcKind::FireImp => &[],
        NpcKind::ChaosBall | NpcKind::BurningSphere | NpcKind::WaterSphere => &[],''')
io.open(p, 'w', encoding='utf-8', newline='').write(s)
print('npc_buffs.rs ok')

# ---------- 5) render/entities.rs ----------
p = 'crates/tr-app/src/render/entities.rs'
s = io.open(p, encoding='utf-8').read()

old = '    pub(crate) meteor_head: Option<BgTex>,'
assert old in s, 'entities struct anchor'
s = s.replace(old, '''    pub(crate) meteor_head: Option<BgTex>,
    pub(crate) goblin_sorcerer: Option<BgTex>,
    pub(crate) tim: Option<BgTex>,
    pub(crate) dark_caster: Option<BgTex>,
    pub(crate) fire_imp: Option<BgTex>,
    pub(crate) chaos_ball: Option<BgTex>,
    pub(crate) burning_sphere: Option<BgTex>,
    pub(crate) water_sphere: Option<BgTex>,''')

old = '        meteor_head: sky_tex(atlas, "NPC_23", None),'
assert old in s, 'entities build anchor'
s = s.replace(old, '''        meteor_head: sky_tex(atlas, "NPC_23", None),
        goblin_sorcerer: sky_tex(atlas, "NPC_29", None),
        tim: sky_tex(atlas, "NPC_45", None),
        dark_caster: sky_tex(atlas, "NPC_32", None),
        fire_imp: sky_tex(atlas, "NPC_24", None),
        chaos_ball: sky_tex(atlas, "NPC_30", None),
        burning_sphere: sky_tex(atlas, "NPC_25", None),
        water_sphere: sky_tex(atlas, "NPC_33", None),''')

old = '''            NpcKind::EaterOfSouls | NpcKind::Hornet | NpcKind::MeteorHead => {
                let tex = match n.kind {'''
assert old in s, 'entities draw anchor'
s = s.replace(old, '''            // The casters: the humanoid sheets — the sorcerer's strip is 3
            // frames, the Dark Caster's 6 (`Main.npcFrameCount` 29/32/45,
            // `Main.cs:67082`), the display box the zombie's 38x48 shape.
            NpcKind::GoblinSorcerer | NpcKind::Tim | NpcKind::DarkCaster => {
                let (tex, frames) = match n.kind {
                    NpcKind::GoblinSorcerer => (assets.goblin_sorcerer.as_ref(), 3.0),
                    NpcKind::Tim => (assets.tim.as_ref(), 3.0),
                    _ => (assets.dark_caster.as_ref(), 6.0),
                };
                (
                    tex,
                    tex.map_or(48.0, |t| t.h / frames),
                    [38.0, 48.0],
                    [n.x - 10.0, n.y - 5.0],
                )
            }
            // The Fire Imp: the 10-frame strip (`Main.npcFrameCount[24]`).
            NpcKind::FireImp => (
                assets.fire_imp.as_ref(),
                assets.fire_imp.as_ref().map_or(48.0, |t| t.h / 10.0),
                [38.0, 48.0],
                [n.x - 10.0, n.y - 5.0],
            ),
            // The spheres: the single-frame 16x16 sheets.
            NpcKind::ChaosBall | NpcKind::BurningSphere | NpcKind::WaterSphere => {
                let tex = match n.kind {
                    NpcKind::ChaosBall => assets.chaos_ball.as_ref(),
                    NpcKind::BurningSphere => assets.burning_sphere.as_ref(),
                    _ => assets.water_sphere.as_ref(),
                };
                let (bw, bh) = n.size();
                (
                    tex,
                    tex.map_or(16.0, |t| t.h),
                    [bw + 4.0, bh],
                    [n.x - 2.0, n.y - 2.0],
                )
            }
            NpcKind::EaterOfSouls | NpcKind::Hornet | NpcKind::MeteorHead => {
                let tex = match n.kind {''')
io.open(p, 'w', encoding='utf-8', newline='').write(s)
print('entities.rs ok')

# ---------- 6) assets.rs ----------
p = 'crates/tr-app/src/assets.rs'
s = io.open(p, encoding='utf-8').read()

old = '        "NPC_23",'
assert old in s, 'atlas list anchor'
s = s.replace(old, '''        "NPC_23",
        "NPC_29",
        "NPC_45",
        "NPC_32",
        "NPC_24",
        "NPC_30",
        "NPC_25",
        "NPC_33",''')

old = '                atlas.sheet("NPC_23").copied(),'
assert old in s, 'sheets table anchor'
s = s.replace(old, '''                atlas.sheet("NPC_23").copied(),
                atlas.sheet("NPC_29").copied(),
                atlas.sheet("NPC_45").copied(),
                atlas.sheet("NPC_32").copied(),
                atlas.sheet("NPC_24").copied(),
                atlas.sheet("NPC_30").copied(),
                atlas.sheet("NPC_25").copied(),
                atlas.sheet("NPC_33").copied(),''')

old = 'pub bestiary_npc_sheets: [Option<tr_render::SheetRect>; 15],'
assert old in s, 'sheets len anchor'
s = s.replace(old, 'pub bestiary_npc_sheets: [Option<tr_render::SheetRect>; 22],')

old = '''    /// The fifteen NPC portrait sheets — `[slime, demon eye, wandering eye,
    /// zombie, king slime, guide, skeleton, undead miner, cave bat, jungle
    /// bat, hellbat, voodoo demon, eater of souls, hornet, meteor head]`,
    /// the `NpcAssets` set restated for the UI draws.'''
assert old in s, 'sheets doc anchor'
s = s.replace(old, '''    /// The twenty-two NPC portrait sheets — the `NpcAssets` set restated
    /// for the UI draws: `[slime, demon eye, wandering eye, zombie, king
    /// slime, guide, skeleton, undead miner, cave bat, jungle bat, hellbat,
    /// voodoo demon, eater of souls, hornet, meteor head, goblin sorcerer,
    /// tim, dark caster, fire imp, chaos ball, burning sphere, water
    /// sphere]`.''')
io.open(p, 'w', encoding='utf-8', newline='').write(s)
print('assets.rs ok')

# ---------- 7) ui/bestiary.rs ----------
p = 'crates/tr-app/src/ui/bestiary.rs'
s = io.open(p, encoding='utf-8').read()

old = '''        NpcKind::MeteorHead => 14,
    };'''
assert old in s, 'sheet_i anchor'
s = s.replace(old, '''        NpcKind::MeteorHead => 14,
        NpcKind::GoblinSorcerer => 15,
        NpcKind::Tim => 16,
        NpcKind::DarkCaster => 17,
        NpcKind::FireImp => 18,
        NpcKind::ChaosBall => 19,
        NpcKind::BurningSphere => 20,
        NpcKind::WaterSphere => 21,
    };''')

old = '        (NpcKind::MeteorHead, _) => (sheet_i, 22.0, 22.0, 22.0, white),'
assert old in s, 'portrait anchor'
s = s.replace(old, '''        (NpcKind::MeteorHead, _) => (sheet_i, 22.0, 22.0, 22.0, white),
        // The casters ride the 38x48 humanoid frame box; the spheres the
        // 16x16 sheet.
        (NpcKind::GoblinSorcerer, _) => (sheet_i, 48.0, 38.0, 48.0, white),
        (NpcKind::Tim, _) => (sheet_i, 48.0, 38.0, 48.0, white),
        (NpcKind::DarkCaster, _) => (sheet_i, 48.0, 38.0, 48.0, white),
        (NpcKind::FireImp, _) => (sheet_i, 48.0, 38.0, 48.0, white),
        (NpcKind::ChaosBall, _) => (sheet_i, 16.0, 16.0, 16.0, white),
        (NpcKind::BurningSphere, _) => (sheet_i, 16.0, 16.0, 16.0, white),
        (NpcKind::WaterSphere, _) => (sheet_i, 16.0, 16.0, 16.0, white),''')
io.open(p, 'w', encoding='utf-8', newline='').write(s)
print('ui/bestiary.rs ok')

# ---------- 8) bestiary.rs ENTRIES ----------
p = 'crates/tr-app/src/bestiary.rs'
s = io.open(p, encoding='utf-8').read()

old = '''    BestiaryEntryDef {
        credit_id: "MeteorHead",
        net_id: 23,
        kind: NpcKind::MeteorHead,
        variant: None,
        quick_unlock: false,
        name: "Meteor Head",
    },
];'''
assert old in s, 'entries tail anchor'
s = s.replace(old, '''    BestiaryEntryDef {
        credit_id: "MeteorHead",
        net_id: 23,
        kind: NpcKind::MeteorHead,
        variant: None,
        quick_unlock: false,
        name: "Meteor Head",
    },
    BestiaryEntryDef {
        credit_id: "GoblinSorcerer",
        net_id: 29,
        kind: NpcKind::GoblinSorcerer,
        variant: None,
        quick_unlock: false,
        name: "Goblin Sorcerer",
    },
    BestiaryEntryDef {
        credit_id: "Tim",
        net_id: 45,
        kind: NpcKind::Tim,
        variant: None,
        quick_unlock: false,
        name: "Tim",
    },
    BestiaryEntryDef {
        credit_id: "DarkCaster",
        net_id: 32,
        kind: NpcKind::DarkCaster,
        variant: None,
        quick_unlock: false,
        name: "Dark Caster",
    },
    BestiaryEntryDef {
        credit_id: "FireImp",
        net_id: 24,
        kind: NpcKind::FireImp,
        variant: None,
        quick_unlock: false,
        name: "Fire Imp",
    },
    // The spheres: the Chaos Ball entry also covers Tim's ChaosBallTim
    // clone 665 (the port folds it — identical stats and name).
    BestiaryEntryDef {
        credit_id: "ChaosBall",
        net_id: 30,
        kind: NpcKind::ChaosBall,
        variant: None,
        quick_unlock: false,
        name: "Chaos Ball",
    },
    BestiaryEntryDef {
        credit_id: "BurningSphere",
        net_id: 25,
        kind: NpcKind::BurningSphere,
        variant: None,
        quick_unlock: false,
        name: "Burning Sphere",
    },
    BestiaryEntryDef {
        credit_id: "WaterSphere",
        net_id: 33,
        kind: NpcKind::WaterSphere,
        variant: None,
        quick_unlock: false,
        name: "Water Sphere",
    },
];''')
io.open(p, 'w', encoding='utf-8', newline='').write(s)
print('bestiary.rs ok')
print('PART B DONE')
