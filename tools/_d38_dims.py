# W4-d38 sheets phase: decode the 35 kinds' NPC sheet dims via atlas_sim's
# LZX/XNB reader (subprocess-free: require the module with a stub main), plus
# the C# Main.npcFrameCount rows and the port's NPC_FRAME_COUNT rows.
import json, os, re, subprocess, sys
ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
IDS = [281,282,283,284,285,286,287,289,290,291,292,293,294,295,296,297,298,299,
       300,301,302,303,304,316,317,318,322,323,324,328,333,334,335,336,337]
# node helper: load atlas_sim.js's sheetDims by cutting main() off
js = open(os.path.join(ROOT,'tools','atlas_sim.js'),encoding='utf-8').read()
# cut everything from the L-8b git-show block (not needed) — replace with stub
js = js[:js.index('// The L-8b branch')] + '''
const L8B_ADDITIONS=[],L8B_REMOVED=[],L8B_STATIC=[];
''' + js[js.index('// Projectile kinds:'):js.index('function main()')]
js += '''
const ids = JSON.parse(process.argv[2]);
const out = {};
for (const id of ids) out[id] = sheetDims('NPC_'+id);
console.log(JSON.stringify(out));
'''
p = os.path.join(ROOT,'tools','_d38_dims_helper.js')
open(p,'w',encoding='utf-8').write(js)
r = subprocess.run(['node',p,json.dumps(IDS)],capture_output=True,text=True,cwd=ROOT)
if r.returncode: print(r.stderr); sys.exit(1)
dims = json.loads(r.stdout)
# C# npcFrameCount (Main.cs:67082's array literal)
cs = open(os.path.join(ROOT,'decompiled','Terraria','Main.cs'),encoding='utf-8-sig',errors='replace').read()
i = cs.index('Main.npcFrameCount = new int[697]')
j = cs.index('{',i); k = cs.index('}',j)
cs_counts = [int(x) for x in re.findall(r'\d+', cs[j:k])]
assert len(cs_counts)==697, len(cs_counts)
# port table
ent = open(os.path.join(ROOT,'crates','tr-app','src','render','entities.rs'),encoding='utf-8').read()
i = ent.index('pub(crate) const NPC_FRAME_COUNT: [u32; 697] = [') + len('pub(crate) const NPC_FRAME_COUNT: [u32; 697] = [')
j = ent.index('];',i)
port_counts = [int(x) for x in re.findall(r'\d+', ent[i:j])]
assert len(port_counts)==697, len(port_counts)
rows=[]
for id in IDS:
    d = dims[str(id)]
    c, p_ = cs_counts[id], port_counts[id]
    ok = 'OK' if c==p_ else 'MISMATCH'
    div = (d['h']//c) if d else None
    rem = (d['h']%c) if d else None
    rows.append((id,d,c,p_,ok,div,rem))
    print(f"NPC_{id}: dims={d} cs_rows={c} port_rows={p_} {ok} row_h={div} rem={rem}")
json.dump({'dims':dims,'rows':[list(r) for r in rows]}, open(os.path.join(ROOT,'tools','_d38_dims.json'),'w'), indent=1)
print('wrote tools/_d38_dims.json')
