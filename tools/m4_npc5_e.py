"""M4-NPC-5 part E: the body-space fix (rows i-4..i-1) + the test world fixes."""
import io
import os

os.chdir('C:/Users/DreadLord/Desktop/Terraria-Rust')

# ---------- 1) npc_ai.rs: the vanilla body-space rows ----------
p = 'crates/tr-app/src/npc_ai.rs'
s = io.open(p, encoding='utf-8').read()

old = '''            // The 3×3 body space above the floor (`:18971-18975`):
            // `SolidTiles(x - 1..x + 1, i - 4..i - 2)` must be false.
            let mut blocked = false;
            'outer: for bx in (x - 1)..=(x + 1) {
                for by in (y - 4)..=(y - 2) {'''
assert old in s, 'body rows anchor'
s = s.replace(old, '''            // The body space above the floor (`:18971-18975`):
            // `SolidTiles(x - fluff..x + fluff, i - 3 - fluff..i - 1)` must
            // be false — fluff 1: cols x-1..x+1, rows i-4..i-1 (four rows —
            // the 40 px caster body spans three of them plus the settle).
            let mut blocked = false;
            'outer: for bx in (x - 1)..=(x + 1) {
                for by in (y - 4)..=(y - 1) {''')
io.open(p, 'w', encoding='utf-8', newline='').write(s)
print('body rows fixed')

# ---------- 2) npc.rs tests: the pit + the player placement ----------
p = 'crates/tr-app/src/npc.rs'
s = io.open(p, encoding='utf-8').read()

old = '''    /// The flat test ground: dirt from y = 120 in a 200x200 world — the
    /// caster tests' arena (the teleport search finds floors with a free
    /// 3×3 body along the surface).
    fn ground_air() -> World {
        let mut w = World::new("t".into(), 200, 200);
        for x in 0..200 {
            for y in 120..200 {
                let i = w.index(x, y);
                w.tiles[i] = Tile {
                    kind: 1,
                    ..Default::default()
                };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        w.compute_column_surface();
        w
    }'''
assert old in s, 'ground_air anchor'
s = s.replace(old, '''    /// The test ground: dirt from y = 120 in a 200x200 world with a carved
    /// pit (x 60..140, y 125..134) — the caster tests' arena. The pit
    /// matters: on a flat surface every floor-level spot sits inside the
    /// player's ±5-tile telefrag rect (the vanilla casters legitimately
    /// fail to teleport there), so the search needs a floor outside the
    /// rect's y window.
    fn ground_air() -> World {
        let mut w = World::new("t".into(), 200, 200);
        for x in 0..200 {
            for y in 120..200 {
                let i = w.index(x, y);
                w.tiles[i] = Tile {
                    kind: 1,
                    ..Default::default()
                };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        for x in 60..140 {
            for y in 125..135 {
                let i = w.index(x, y);
                w.tiles[i] = Tile::default();
            }
        }
        w.compute_column_surface();
        w
    }

    /// The player parked on the surface at column 100 (the tests' fixed
    /// reference — `Player::spawn`'s world-spawn default rides the unset
    /// `spawn_x/spawn_y`).
    fn surface_player(w: &World) -> crate::player::Player {
        let mut p = crate::player::Player::spawn(w);
        p.x = 100.0 * TILE_PX;
        p.y = 116.0 * TILE_PX;
        p
    }''')

# The caster test: use surface_player.
old = '''        let mut w = ground_air();
        let mut player = crate::player::Player::spawn(&w);
        let pc = player.center();
        let mut tim = Npc::new(NpcKind::Tim, 40.0 * 16.0, 110.0 * 16.0, 1, 0.0);'''
assert old in s, 'caster test player anchor'
s = s.replace(old, '''        let mut w = ground_air();
        let mut player = surface_player(&w);
        let pc = player.center();
        let mut tim = Npc::new(NpcKind::Tim, 40.0 * 16.0, 110.0 * 16.0, 1, 0.0);''')

old = '''        let mut w = ground_air();
        let mut player = crate::player::Player::spawn(&w);
        let mut imp = Npc::new(NpcKind::FireImp, 60.0 * 16.0, 110.0 * 16.0, 1, 0.0);'''
assert old in s, 'imp test player anchor'
s = s.replace(old, '''        let mut w = ground_air();
        let mut player = surface_player(&w);
        let mut imp = Npc::new(NpcKind::FireImp, 60.0 * 16.0, 110.0 * 16.0, 1, 0.0);''')

old = '''        let mut w = ground_air();
        let mut player = crate::player::Player::spawn(&w);
        // The ball sits up-left of the player: the launch aims right/down.
        let mut ball = Npc::new(NpcKind::ChaosBall, 80.0 * 16.0, 100.0 * 16.0, 1, 0.0);
        let (pcx, _) = player.center();'''
assert old in s, 'sphere test player anchor'
s = s.replace(old, '''        let mut w = ground_air();
        let mut player = surface_player(&w);
        let (pcx, _) = player.center();
        // The ball sits up-left of the player: the launch aims right/down.
        let mut ball = Npc::new(NpcKind::ChaosBall, 80.0 * 16.0, 100.0 * 16.0, 1, 0.0);''')

old = '''        let mut sphere = Npc::new(NpcKind::BurningSphere, 80.0 * 16.0, 100.0 * 16.0, 1, 0.0);'''
assert old in s, 'burning sphere anchor'
s = s.replace(old, '''        let mut sphere = Npc::new(NpcKind::BurningSphere, 80.0 * 16.0, 100.0 * 16.0, 1, 0.0);''')

# The spawn test: ground_y 120 (the solid tile), not 119.
old = '''        let imp = Npc::spawn_caster(&w, 30, 119, NpcKind::FireImp)
            .expect("the imp spawns on the ground");
        assert_eq!(imp.size(), (18.0, 40.0));
        assert_eq!(imp.max_hp, 70);
        assert!((imp.y - (119.0 * 16.0 - 40.0)).abs() < 0.01);
        let sorcerer = Npc::spawn_caster(&w, 34, 119, NpcKind::GoblinSorcerer)
            .expect("the sorcerer spawns");'''
assert old in s, 'spawn test anchor'
s = s.replace(old, '''        let imp = Npc::spawn_caster(&w, 30, 120, NpcKind::FireImp)
            .expect("the imp spawns on the ground");
        assert_eq!(imp.size(), (18.0, 40.0));
        assert_eq!(imp.max_hp, 70);
        assert!((imp.y - (120.0 * 16.0 - 40.0)).abs() < 0.01);
        let sorcerer = Npc::spawn_caster(&w, 34, 120, NpcKind::GoblinSorcerer)
            .expect("the sorcerer spawns");''')
io.open(p, 'w', encoding='utf-8', newline='').write(s)
print('tests fixed')
print('PART E DONE')
