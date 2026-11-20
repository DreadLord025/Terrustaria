import re, os
ROOT = r"C:\Users\DreadLord\Desktop\TR-W4-d35"
def rd(p): return open(os.path.join(ROOT, p), encoding='utf-8').read()
data=rd(r'crates\tr-app\src\npc\data.rs')
b=rd(r'crates\tr-app\src\npc_buffs.rs')
ent=rd(r'crates\tr-app\src\render\entities.rs')
net=rd(r'crates\tr-app\src\app\netglue.rs')
uib=rd(r'crates\tr-app\src\ui\bestiary.rs')
aud=rd(r'crates\tr-app\src\audio.rs')
npc=rd(r'crates\tr-app\src\npc.rs')
E=['Clinger','AnglerFish','GreenJellyfish','Werewolf','Clown']
for fn in ['fn hit_sound(','fn death_sound(','fn behind_tiles(','fn no_gravity(']:
    i=data.index(fn);rest=data[i:]
    end=rest.index('_ => false') if fn=='fn no_gravity(' else rest.index('\n    }')
    body=rest[:end]
    print(fn,'E:',[k for k in E if ('NpcKind::'+k) in body])
i=b.index('fn debuff_immune_ids');j=b.index('\n}',i)
print('debuff E:',[k for k in E if ('NpcKind::'+k) in b[i:j]])
print('netglue E:',[k for k in E if ('NpcKind::'+k) in net])
print('ENT draw E:',[k for k in E if ('NpcKind::'+k+' =>') in ent])
print('ENT rotation Clinger:', '| NpcKind::Clinger' in ent)
print('UIB portrait E:',[k for k in E if ('(NpcKind::'+k+', _)') in uib])
print('AUD NPC_Hit_25:', '("NPC_Hit_25",' in aud, 'NPC_Killed_28:', '("NPC_Killed_28",' in aud)
# find_frame E
i=npc.index('pub(crate) fn find_frame');j=npc.index('fn update_slime',i)
print('find_frame E:',[k for k in E if ('NpcKind::'+k) in npc[i:j]])
# scale: none of E has scale!=1 (all 1.0) -> step13 scale skipped; alpha: GreenJellyfish 20
print('E alpha model: GreenJellyfish=20')
