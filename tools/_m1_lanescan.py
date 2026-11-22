# W4 merger wave-1 small-lanes: per-lane tip census vs main.
# For each lane: to_u16 map at tip; kinds added vs main; id changes for
# kinds main already has (would-be conflicts); ALL_KINDS count at tip.
import re, subprocess, json, sys
REPO = r"C:\Users\DreadLord\Desktop\TR-MERGE-1"
def show(rev, path):
    r = subprocess.run(["git","-C",REPO,"show",f"{rev}:{path}"],capture_output=True)
    return r.stdout.decode("utf8","replace") if r.returncode==0 else None
def to_map(src):
    i = src.index("pub fn to_u16"); j = src.index("{", i)
    depth=0; k=j
    while True:
        if src[k]=="{": depth+=1
        elif src[k]=="}":
            depth-=1
            if depth==0: break
        k+=1
    body = re.sub(r"//[^\n]*","",src[j:k])
    return dict((m[0],int(m[1])) for m in re.findall(r"NpcKind::(\w+)\s*=>\s*(\d+)", body))
def ak(src):
    m = re.search(r"const ALL_KINDS:\s*\[NpcKind;\s*(\d+)\]", src)
    return int(m.group(1)) if m else None

main_src = show("HEAD","crates/tr-app/src/npc/data.rs")
main_map = to_map(main_src)
main_npc = show("HEAD","crates/tr-app/src/npc.rs")

LANES = {
 "c3":"c3-style10-cursed-skull","c4":"c4-style44-flying-fish",
 "c7":"c7-style64-fireflies","c7b":"c7-style65-critters",
 "c8":"c8-style66-crawling-critters","c9":"c9-style67-snails",
 "c10":"c10-style68-waterfowl","d23":"d23-npc-ai-styles-9-14",
 "d120":"d120-town-chatter","d125":"d125-empress-r7-drain",
 "g5":"g5-npc-style7-caster-doc","g8":"g8-npc-kind-wire-1",
 "d33":"d33-npc-ai-styles-95-127","e21":"e21-wiring-device-arms-3",
 "f42":"f42-sandstorm-2","d9":"d9-proj-ai-81-90",
 "d39":"d39-npc-kinds-351-420","d41":"d41-npc-kinds-491-560",
 "e26":"e26-luck-system","g10":"g10-npc-kind-wire-3",
 "f47":"f47-liquid-shimmer-2","f18":"f18-npc-kinds-verify-2",
 "d103":"d103-worldgen-passes-3","g62":"g62-npc-hit-dust",
 "d11":"d11-proj-ai-101-110","d32":"d32-npc-ai-styles-85-93",
}
out={}
for lane,br in LANES.items():
    rev=f"origin/task/W4-{br}"
    src=show(rev,"crates/tr-app/src/npc/data.rs")
    npcsrc=show(rev,"crates/tr-app/src/npc.rs")
    if src is None or "pub fn to_u16" not in src:
        # older layout: to_u16 in npc.rs
        src=npcsrc
    m=to_map(src)
    added={k:v for k,v in m.items() if k not in main_map}
    changed={k:(main_map[k],v) for k,v in m.items() if k in main_map and main_map[k]!=v}
    removed=[k for k in main_map if k not in m]  # only meaningful vs merge-base
    out[lane]={"rev":subprocess.run(["git","-C",REPO,"rev-parse","--short",rev],capture_output=True).stdout.decode().strip(),
               "all_kinds":ak(npcsrc),"added":added,"changed":changed}
    print(f"== {lane} ({out[lane]['rev']}) ALL_KINDS={out[lane]['all_kinds']} adds={len(added)} changed={len(changed)}")
    print("   added:", json.dumps(added, sort_keys=True))
    if changed: print("   CHANGED-vs-main:", json.dumps(changed, sort_keys=True))
json.dump(out, open(REPO+r"\tools\_m1_lanescan.json","w"), indent=1, sort_keys=True)
