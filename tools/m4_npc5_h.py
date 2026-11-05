"""M4-NPC-5 part H: the depth-band test's pocket depths."""
import io
import os

os.chdir('C:/Users/DreadLord/Desktop/Terraria-Rust')

p = 'crates/tr-app/src/spawner.rs'
s = io.open(p, encoding='utf-8').read()

old = '''        // The underworld pocket: the floor at 515 (past the 510 line).
        carve_pocket(&mut w, 515);
        let c_hell = ctx(&w, 515.0 * 16.0, false);
        let mut tims = 0;
        let mut imps = 0;
        for seed in 1..3000u64 {
            match outcome_at(&c_hell, seed, 515 * 16) {'''
assert old in s, 'hell pocket anchor'
s = s.replace(old, '''        // The underworld pocket: the floor at 720 (past the 710 line =
        // height 900 - 190).
        carve_pocket(&mut w, 720);
        let c_hell = ctx(&w, 720.0 * 16.0, false);
        let mut tims = 0;
        let mut imps = 0;
        for seed in 1..3000u64 {
            match outcome_at(&c_hell, seed, 720 * 16) {''')
io.open(p, 'w', encoding='utf-8', newline='').write(s)
print('pocket depths fixed')
