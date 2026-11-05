"""M4-NPC-5 part C: the depth-gate correction, the sim RNG stream, the tests."""
import io
import os

os.chdir('C:/Users/DreadLord/Desktop/Terraria-Rust')

# ---------- 1) spawner.rs: the vanilla depth-branch structure ----------
p = 'crates/tr-app/src/spawner.rs'
s = io.open(p, encoding='utf-8').read()

old = '''    // The Tim: the deep-cavern roll (`NPC.cs:5014`): `(remixWorld ||
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
    }'''
assert old in s, 'part B arms anchor'
s = s.replace(old, '''    // The depth gates: `NPC.cs:4869`'s `spawnTileY > maxTilesY - 190`
    // underworld branch precedes the `:5005+` cavern branch — mutually
    // exclusive in vanilla; the port keeps the exclusivity inside one
    // section.
    let underworld_depth = (y as f64 / 16.0) > ctx.world.height as f64 - 190.0;
    // The Tim: the deep-cavern roll (`:5014`): `(remixWorld ||
    // spawnTileY > (rockLayer + maxTilesY) / 2.0) && (Next(200) == 0 ||
    // (offensiveToTim && Next(50) == 0)) → 45` — non-underworld depth
    // only. The port gates the depth alone: the offensiveToTim accessory
    // rows (`NPC.cs:418`, the M3-EQUIP-3 work) stay constants-off, so the
    // Next(50) never runs.
    if !underworld_depth
        && (y as f64 / 16.0) > (ctx.world.rock_layer + ctx.world.height as f64) / 2.0
        && rng.next(200) == 0
    {
        return Npc::spawn_caster(ctx.world, x, y / 16, NpcKind::Tim);
    }
    // The Fire Imp: the underworld arm (`:4891-4893`'s `Next(14) == 0 →
    // 24`); the lava-critter Next(8) and Demon Next(40) arms ahead of it
    // are unmodelled (constants-off, consume nothing), and a missed imp
    // roll falls through to the cavern arms (the vanilla's remaining
    // underworld arms map to the port's closest kinds there).
    if underworld_depth && rng.next(14) == 0 {
        return Npc::spawn_caster(ctx.world, x, y / 16, NpcKind::FireImp);
    }''')
io.open(p, 'w', encoding='utf-8', newline='').write(s)
print('spawner depth gates ok')

# ---------- 2) world_sim/npcs.rs: the real RNG stream ----------
p = 'crates/tr-app/src/world_sim/npcs.rs'
s = io.open(p, encoding='utf-8').read()

old = '''            for n in npcs.iter_mut() {
                n.update(world, player, step);
                black_slime_darkness_touch(n, player, &mut app.item_rng);
            }'''
assert old in s, 'sim loop anchor'
s = s.replace(old, '''            for n in npcs.iter_mut() {
                n.update_rng(world, player, step, &mut app.item_rng);
                black_slime_darkness_touch(n, player, &mut app.item_rng);
            }''')
io.open(p, 'w', encoding='utf-8', newline='').write(s)
print('sim rng stream ok')

# ---------- 3) npc.rs: the tests ----------
p = 'crates/tr-app/src/npc.rs'
s = io.open(p, encoding='utf-8').read()

tests = '''
    #[test]
    fn the_caster_teleports_and_shoots() {
        // `AI_008` (`NPC.cs:20817-21278`): the 500 counter init, the
        // teleport attempt at 650 (the spot ≥ 5 tiles from the player —
        // the telefrag gate), then the counter resets and the 100 trigger
        // winds the 30-tick shot that fires at `ai[1] == 25`.
        let mut w = ground_air();
        let mut player = crate::player::Player::spawn(&w);
        let pc = player.center();
        let mut tim = Npc::new(NpcKind::Tim, 40.0 * 16.0, 110.0 * 16.0, 1, 0.0);
        tim.hp = tim.max_hp;
        let start = (tim.x, tim.y);
        let mut rng = 7u64;
        let mut teleported = false;
        let mut shot = false;
        for _ in 0..400 {
            tim.update_rng(&w, &mut player, 1.0 / 60.0, &mut rng);
            if let Some((kind, sx, sy)) = tim.wants_sphere {
                assert_eq!(kind, NpcKind::ChaosBall);
                // The Chaos Ball spawn point: Tim's bottom-centre, 8 px
                // above the top (`:21242`'s NewNPC(x + w/2, y - 8) with the
                // `Bottom` semantics).
                let (tw, _) = tim.size();
                assert!((sx - (tim.x + tw * 0.5)).abs() < 0.01);
                assert!((sy - (tim.y - 8.0)).abs() < 0.01);
                shot = true;
                break;
            }
            if !teleported {
                let moved = (tim.x - start.0).abs() + (tim.y - start.1).abs();
                if moved > 0.0 {
                    teleported = true;
                    // The teleport respected the ±5-tile telefrag gate.
                    let dcx = ((tim.x + tim.size().0 * 0.5) - pc.0).abs();
                    let dcy = ((tim.y + tim.size().1 * 0.5) - pc.1).abs();
                    assert!(
                        dcx > 70.0 || dcy > 70.0,
                        "the telefrag gate held ({dcx}x{dcy})"
                    );
                    // The feet rest on a solid floor.
                    let floor = w
                        .tile(
                            (tim.x / 16.0) as i32,
                            ((tim.y + tim.size().1) / 16.0) as i32,
                        )
                        .is_some_and(|t| t.active() && tr_core::tiledata::TILE_SOLID.has(t.kind));
                    assert!(floor, "the teleport landed the feet on a solid floor");
                }
            }
        }
        assert!(teleported, "the caster never teleported");
        assert!(shot, "Tim never fired the Chaos Ball");
        // The post-teleport windup (20) never crosses the shoot tick (25)
        // — the shot came from the 100/200/300 trigger's 30-tick windup.
    }

    #[test]
    fn the_fire_imp_shoots_at_its_tick() {
        // The Fire Imp's shoot tick is `ai[1] == 10` (`:21166`) — 20 ticks
        // after the 30-tick windup, 15 after the standard casters'.
        let mut w = ground_air();
        let mut player = crate::player::Player::spawn(&w);
        let mut imp = Npc::new(NpcKind::FireImp, 60.0 * 16.0, 110.0 * 16.0, 1, 0.0);
        imp.hp = imp.max_hp;
        let mut rng = 11u64;
        let mut shot = false;
        for _ in 0..400 {
            imp.update_rng(&w, &mut player, 1.0 / 60.0, &mut rng);
            if let Some((kind, _, _)) = imp.wants_sphere {
                assert_eq!(kind, NpcKind::BurningSphere);
                shot = true;
                break;
            }
        }
        assert!(shot, "the Fire Imp never fired the Burning Sphere");
    }

    #[test]
    fn the_sphere_launches_once_and_flies_straight() {
        // `AI_009` (`NPC.cs:21404-21429`): the velocity points at the
        // player once (the speed 6 for the Chaos Ball, 5 for the Burning
        // Sphere), then the ball flies straight through the tiles.
        let mut w = ground_air();
        let mut player = crate::player::Player::spawn(&w);
        // The ball sits up-left of the player: the launch aims right/down.
        let mut ball = Npc::new(NpcKind::ChaosBall, 80.0 * 16.0, 100.0 * 16.0, 1, 0.0);
        let (pcx, _) = player.center();
        ball.update_rng(&w, &mut player, 1.0 / 60.0, &mut 0u64);
        let speed = (ball.vx * ball.vx + ball.vy * ball.vy).sqrt();
        assert!((speed - 6.0).abs() < 0.05, "the launch speed is 6 ({speed})");
        assert!(ball.vx > 0.0 && ball.vy > 0.0, "the ball dives at the player");
        let v = (ball.vx, ball.vy);
        // The straight fly: the velocity never re-aims and the dirt passes
        // through (the floor line sits at y = 120*16).
        for _ in 0..100 {
            ball.update_rng(&w, &mut player, 1.0 / 60.0, &mut 0u64);
        }
        assert_eq!(ball.vx, v.0);
        assert_eq!(ball.vy, v.1);
        assert!(ball.x > pcx, "the ball crossed the player's column");
        assert!(ball.y > 120.0 * 16.0, "the ball flew through the floor line");
        // The Burning Sphere's speed is 5 (`:21395-21399`).
        let mut sphere = Npc::new(NpcKind::BurningSphere, 80.0 * 16.0, 100.0 * 16.0, 1, 0.0);
        sphere.update_rng(&w, &mut player, 1.0 / 60.0, &mut 0u64);
        let speed = (sphere.vx * sphere.vx + sphere.vy * sphere.vy).sqrt();
        assert!((speed - 5.0).abs() < 0.05, "the burning sphere speed is 5");
    }

    #[test]
    fn the_casters_spawn_stand_and_map() {
        // `spawn_caster` plants the feet on the ground tile (`SpawnNPC`'s
        // `Bottom = (x*16+8, y*16)`), the stats ride the verified
        // `SetDefaults` rows, and the bestiary entries round-trip.
        let mut w = ground_air();
        let imp = Npc::spawn_caster(&w, 30, 119, NpcKind::FireImp)
            .expect("the imp spawns on the ground");
        assert_eq!(imp.size(), (18.0, 40.0));
        assert_eq!(imp.max_hp, 70);
        assert!((imp.y - (119.0 * 16.0 - 40.0)).abs() < 0.01);
        let sorcerer = Npc::spawn_caster(&w, 34, 119, NpcKind::GoblinSorcerer)
            .expect("the sorcerer spawns");
        assert_eq!(sorcerer.size(), (18.0, 38.0));
        assert_eq!(sorcerer.max_hp, 40);
        let tim = Npc::new(NpcKind::Tim, 0.0, 0.0, 1, 0.0);
        assert_eq!(tim.max_hp, 200);
        assert_eq!(tim.value(), 5000.0);
        for (kind, net) in [
            (NpcKind::GoblinSorcerer, 29),
            (NpcKind::Tim, 45),
            (NpcKind::DarkCaster, 32),
            (NpcKind::FireImp, 24),
            (NpcKind::ChaosBall, 30),
            (NpcKind::BurningSphere, 25),
            (NpcKind::WaterSphere, 33),
        ] {
            let e = crate::bestiary::entry_for_npc(kind, SlimeVariant::Blue);
            assert_eq!(e.net_id, net);
        }
    }
'''
idx = s.rstrip().rfind('\n}')
assert idx > 0, 'tests mod close'
s = s[:idx] + tests + s[idx:]
io.open(p, 'w', encoding='utf-8', newline='').write(s)
print('npc.rs tests ok')
print('PART C DONE')
