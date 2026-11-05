"""M4-NPC-5 part D: the tests module's ground_air helper."""
import io
import os

os.chdir('C:/Users/DreadLord/Desktop/Terraria-Rust')

p = 'crates/tr-app/src/npc.rs'
s = io.open(p, encoding='utf-8').read()

old = '    use tr_core::tile::{FLAG_ACTIVE, Tile};'
assert old in s, 'tests imports anchor'
s = s.replace(old, '''    use tr_core::tile::{FLAG_ACTIVE, Tile};

    /// The flat test ground: dirt from y = 120 in a 200x200 world — the
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
    }''')
io.open(p, 'w', encoding='utf-8', newline='').write(s)
print('ground_air helper ok')
