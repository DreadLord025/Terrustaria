import re, os
ROOT = r"C:\Users\DreadLord\Desktop\TR-W4-d35"
def rd(p): return open(os.path.join(ROOT, p), encoding='utf-8').read()
cs = rd(r'decompiled\Terraria\ID\ContentSamples.cs')
i = cs.index('private static void ModifyNPCIds') if 'private static void ModifyNPCIds' in cs else cs.index('ModifyNPCIds()')
j = cs.find('static void ModifyNPCIds', i)
# print the ModifyNPCIds body region around credit remaps
i2 = cs.index('void ModifyNPCIds')
print('--- ModifyNPCIds head ---')
print(cs[i2:i2+3000])
b = rd(r'crates\tr-app\src\bestiary.rs')
i = b.index('pub fn entry_for_npc'); j = b.index('\n}', i)
print('--- entry_for_npc full ---')
print(b[i:j+2])
