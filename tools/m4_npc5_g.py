"""M4-NPC-5 part G (v2): the spawner-arm seed-sweep tests with air pockets."""
import io
import os

os.chdir('C:/Users/DreadLord/Desktop/Terraria-Rust')

p = 'crates/tr-app/src/spawner.rs'
s = io.open(p, encoding='utf-8').read()

# Replace the two failing tests (they sit at the tail I appended).
old_start = s.index('''
    #[test]
    fn the_deep_cavern_rolls_tim() {''')
old_end = s.rfind('\n}')
tests = '''
    /// Carves an air pocket (x 200..220) whose floor tile sits at
    /// `floor_y` — `spawn_caster` plants the feet on it, so the pocket
    /// must clear the 2.5-tile body above.
    fn carve_pocket(w: &mut World, floor_y: i32) {
        for x in 200..220 {
            for y in (floor_y - 6)..floor_y {
                let i = w.index(x, y);
                w.tiles[i] = Tile::default();
            }
        }
        w.compute_column_surface();
    }

    #[test]
    fn the_depth_bands_split_tim_and_fire_imp() {
        // The depth gates (`NPC.cs:4869`'s underworld branch vs `:5014`'s
        // cavern roll): the underworld depth never rolls Tim; the
        // deep-cavern depth (below the (rock+height)/2 gate, above the
        // underworld line) never rolls the imp; each rolls its own caster.
        // ground_tall's bands touch (both at 510), so a taller world
        // separates them: rock 320, height 900 → the Tim gate 610, the
        // underworld line 710.
        let mut w = World::new("t".into(), 400, 900);
        for x in 0..400 {
            for y in 300..900 {
                let i = w.index(x, y);
                w.tiles[i] = Tile {
                    kind: 1,
                    ..Default::default()
                };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        w.world_surface = 310.0;
        w.rock_layer = 320.0;
        w.spawn_x = 200;
        w.spawn_y = 296;
        // The underworld pocket: the floor at 515 (past the 510 line).
        carve_pocket(&mut w, 515);
        let c_hell = ctx(&w, 515.0 * 16.0, false);
        let mut tims = 0;
        let mut imps = 0;
        for seed in 1..3000u64 {
            match outcome_at(&c_hell, seed, 515 * 16) {
                Some((NpcKind::Tim, _)) => tims += 1,
                Some((NpcKind::FireImp, _)) => imps += 1,
                _ => {}
            }
        }
        assert_eq!(tims, 0, "the underworld depth never rolls Tim");
        assert!(imps > 0, "the underworld pocket never rolled the Fire Imp");
        // The deep-cavern pocket: the floor at 640 (past the 610 gate,
        // above the 710 underworld line).
        carve_pocket(&mut w, 640);
        let c_deep = ctx(&w, 640.0 * 16.0, false);
        let mut tims = 0;
        let mut imps = 0;
        for seed in 1..3000u64 {
            match outcome_at(&c_deep, seed, 640 * 16) {
                Some((NpcKind::Tim, _)) => tims += 1,
                Some((NpcKind::FireImp, _)) => imps += 1,
                _ => {}
            }
        }
        assert!(tims > 0, "the deep-cavern sweep never rolled Tim");
        assert_eq!(imps, 0, "the deep-cavern depth never rolls the Fire Imp");
    }
'''
s = s[:old_start] + tests + s[old_end:] + '\n'
# Drop the now-removed fire-imp test if it survived after old_end... it was
# inside the replaced range (old_start .. EOF close), so gone.
io.open(p, 'w', encoding='utf-8', newline='').write(s)
print('spawner tests v2 ok')
