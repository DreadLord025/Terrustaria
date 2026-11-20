import re, os, json
ROOT = r"C:\Users\DreadLord\Desktop\TR-W4-d35"
raw = open(os.path.join(ROOT,r'decompiled\Terraria.Localization.Content.en-US.NPCs.json'),encoding='utf-8-sig').read()
raw = re.sub(r',\s*([}\]])', r'\1', raw)
loc = json.loads(raw)
# structure: {"NPCName": {...}} probably
key = 'NPCName' if 'NPCName' in loc else list(loc.keys())[0]
names = loc[key]
npcid = open(os.path.join(ROOT,r'decompiled\Terraria\ID\NPCID.cs'),encoding='utf-8').read().split('\n')
want = {73:'GoblinScout',74:'Bird',75:'Pixie',77:'ArmoredSkeleton',78:'Mummy',79:'DarkMummy',80:'LightMummy',82:'Wraith',83:'CursedHammer',84:'EnchantedSword',85:'Mimic',86:'Unicorn',93:'GiantBat',94:'Corruptor',95:'DiggerHead',96:'DiggerBody',97:'DiggerTail',98:'SeekerHead',99:'SeekerBody',100:'SeekerTail',101:'Clinger',102:'AnglerFish',103:'GreenJellyfish',104:'Werewolf',109:'Clown',110:'SkeletonArcher',112:'VileSpit',116:'TheHungryII',120:'ChaosElemental',121:'Slimer',122:'Gastropod',137:'IlluminantBat',140:'PossessedArmor'}
out={}
for i,n in want.items():
    ln = next((j+1 for j,l in enumerate(npcid) if re.search(r'public const short %s = %d;'%(n,i),l)), None)
    out[i]={'name':n,'npcid_line':ln,'display':names.get(n)}
print(json.dumps(out,indent=0))
json.dump(out,open(os.path.join(ROOT,'tools','_d35_names.json'),'w'),indent=0)
