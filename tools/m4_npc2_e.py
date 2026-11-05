"""M4-NPC-2 part E: the sheets array → Vec (the Default derive's 32-array
limit)."""
import io
import os

os.chdir('C:/Users/DreadLord/Desktop/Terraria-Rust')

p = 'crates/tr-app/src/assets.rs'
s = io.open(p, encoding='utf-8').read()

old = '    pub bestiary_npc_sheets: [Option<tr_render::SheetRect>; 34],'
assert old in s, 'field anchor'
s = s.replace(old, '    pub bestiary_npc_sheets: Vec<Option<tr_render::SheetRect>>,')

old = '''            bestiary_npc_sheets: [
                atlas.sheet("NPC_1").copied(),'''
assert old in s, 'init anchor'
s = s.replace(old, '''            bestiary_npc_sheets: vec![
                atlas.sheet("NPC_1").copied(),''')
io.open(p, 'w', encoding='utf-8', newline='').write(s)
print('sheets Vec ok')
