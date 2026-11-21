# W4-d37 LAGGARD-EXEC FINAL: re-derive OWED moves/drops by comparing every
# claimant's CURRENT-tip to_u16 map against the adjudicated FINAL plan.
# Self-updating: a kind at its target id on its owner lane = DONE; at the
# old id = move owed; on a non-owner lane = drop owed. Read-only.
import re, subprocess, json

REPO = r"C:\Users\DreadLord\Desktop\TR-W4-d37"
DATA = "crates/tr-app/src/npc/data.rs"

def git(*a):
    r = subprocess.run(["git", "-C", REPO] + list(a), capture_output=True)
    return r.returncode, r.stdout.decode("utf8", "replace")

def show(rev, path):
    rc, out = git("show", f"{rev}:{path}")
    return out if rc == 0 else ""

def fn_body(src, sig):
    i = src.index(sig); j = src.index("{", i); d = 0; k = j
    while True:
        c = src[k]
        if c == "{": d += 1
        elif c == "}":
            d -= 1
            if d == 0: return src[j:k+1]
        k += 1

def arms(rev):
    src = show(rev, DATA)
    if not src: return {}
    return {m[0]: int(m[1]) for m in re.findall(
        r"NpcKind::(\w+) => (\d+),", fn_body(src, "pub fn to_u16"))}

# ---- the adjudicated FINAL plan (kind -> (id, owner-lane))
seal = json.load(open(REPO + r"\tools\_d37_seal_final.json"))
plan = {k: (v, seal["owners"][k][0]) for k, v in seal["final"].items()}
# post-SEAL accepts + re-pins (the lag_union additions, minus ones now executed)
union = json.load(open(REPO + r"\tools\_d37_lag_union.json"))
for k, v in union["additions"].items():
    plan[k] = (v["id"], v["lane"])
# live-wave corrections (probe6/7):
#  d36 executed its 5 re-pins AND added Crimera/MossHornet/Moth (free ids)
for k, i in [("ToxicSludge",767),("Penguin",768),("PenguinBlack",769),
             ("GiantTortoise",770),("IceTortoise",771),
             ("Crimera",781),("MossHornet",782),("Moth",793)]:
    plan[k] = (i, "d36")
#  d41 batch-6 ten new vanilla-free kinds ACCEPT
for k, i in [("AncientLight",522),("AncientDoom",523),("DesertGhoul",524),
             ("DesertGhoulCorruption",525),("DesertGhoulCrimson",526),
             ("DesertGhoulHallow",527),("DesertLamiaLight",528),
             ("DesertLamiaDark",529),("DesertScorpionWalk",530),
             ("DesertScorpionWall",531)]:
    plan[k] = (i, "d41")
#  d34's two UNIQUE new kinds re-pin out of d38's 281-285 corridor
plan["Crab"] = (791, "d34")
plan["DungeonGuardian"] = (792, "d34")
#  e21 PartyBunny unique, 237 free after d125's move + f18 drop -> ACCEPT
plan["PartyBunny"] = (237, "e21")
#  g8 Vampire renumber 388->762 (already in union additions)
#  d34's Antlion/PinkJellyfish/SpikeBall are dups of g8's WIRED machines ->
#    g8 owns (SEAL S1), d34 drops. (plan already has them under g8.)

# ---- every claimant lane -> current tip
LANES = {
 "c3":"origin/task/W4-c3-style10-cursed-skull","c4":"origin/task/W4-c4-style44-flying-fish",
 "c6":"origin/task/W4-c6-style59-pumpking-blade","c7":"origin/task/W4-c7-style64-fireflies",
 "c8":"origin/task/W4-c8-style66-crawling-critters","c9":"origin/task/W4-c9-style67-snails",
 "c10":"origin/task/W4-c10-style68-waterfowl","d23":"origin/task/W4-d23-npc-ai-styles-9-14",
 "d33":"origin/task/W4-d33-npc-ai-styles-95-127","d34":"origin/task/W4-d34-npc-kinds-1-70",
 "d35":"origin/task/W4-d35-npc-kinds-71-140","d36":"origin/task/W4-d36-npc-kinds-141-210",
 "d37":"750d8e446","d38":"origin/task/W4-d38-npc-kinds-281-350",
 "d39":"origin/task/W4-d39-npc-kinds-351-420","d40":"origin/task/W4-d40-npc-kinds-421-490",
 "d41":"origin/task/W4-d41-npc-kinds-491-560","d42":"origin/task/W4-d42-npc-kinds-561-630",
 "d43":"origin/task/W4-d43-npc-kinds-631-697","d120":"origin/task/W4-d120-town-chatter",
 "d125":"origin/task/W4-d125-empress-r7-drain","e26":"origin/task/W4-e26-luck-system",
 "g8":"origin/task/W4-g8-npc-kind-wire-1","g10":"origin/task/W4-g10-npc-kind-wire-3",
 "d9":"origin/task/W4-d9-proj-ai-81-90","d11":"origin/task/W4-d11-proj-ai-101-110",
 "g5":"origin/task/W4-g5-npc-style7-caster-doc","f42":"origin/task/W4-f42-sandstorm-2",
 "f18":"origin/task/W4-f18-npc-kinds-verify-2","f47":"origin/task/W4-f47-liquid-shimmer-2",
 "e21":"origin/task/W4-e21-wiring-device-arms-3","g62":"origin/task/W4-g62-npc-hit-dust",
}
base = set(arms("a725acb04"))
tips, cur = {}, {}
for lane, rev in LANES.items():
    rc, t = git("rev-parse", "--short", rev)
    tips[lane] = t.strip()
    cur[lane] = {k: v for k, v in arms(rev).items() if k not in base}

# alias kinds that fold onto a canonical (drop, never own)
ALIAS = {"HallowBoss":"EmpressOfLight","EmpressButterfly":"PrismaticLacewing",
         "VileSpitEaterOfWorlds":"VileSpitEoW"}

moves, drops, done = {}, {}, {}
for lane, kinds in cur.items():
    for k, id_ in kinds.items():
        if k in ALIAS:
            drops.setdefault(lane, []).append({"kind":k,"id":id_,"keeper":plan[ALIAS[k]][1],"why":"alias"})
            continue
        if k not in plan:
            # unclaimed kind a lane ported but the plan doesn't know -> flag
            drops.setdefault(lane, []).append({"kind":k,"id":id_,"keeper":"?","why":"UNPLANNED"})
            continue
        target, owner = plan[k]
        if lane == owner:
            if id_ == target:
                done.setdefault(lane, []).append(k)
            else:
                moves.setdefault(lane, []).append({"kind":k,"old":id_,"new":target})
        else:
            drops.setdefault(lane, []).append({"kind":k,"id":id_,"keeper":owner,"why":"dup"})

nm = sum(len(v) for v in moves.values()); nd = sum(len(v) for v in drops.values())
out = {"tips":tips,"plan_size":len(plan),"moves_total":nm,"drops_total":nd,
       "moves":moves,"drops":drops,"done":done}
json.dump(out, open(REPO+r"\tools\_d37_lag_final.json","w"), indent=1, sort_keys=True)
print(f"FINAL plan {len(plan)} kinds; OWED {nm} moves / {nd} drops")
for lane in sorted(set(moves)|set(drops)):
    print(f"  {lane:6s} tip={tips[lane]} moves={len(moves.get(lane,[])):2d} drops={len(drops.get(lane,[])):2d}")
unplanned=[(l,d) for l,ds in drops.items() for d in ds if d["why"]=="UNPLANNED"]
print("UNPLANNED:", unplanned)
