# W4-d37 LAGGARD-EXEC: FINAL master move/drop table. SEAL receipts (69/61)
# + post-SEAL mover adjudications. Emits per-lane old->new + drop-hunks with
# the arm-line sites gathered by the lag_* probes. Read-only derivation.
import json, subprocess, re

REPO = r"C:\Users\DreadLord\Desktop\TR-W4-d37"
seal_receipts = json.load(open(REPO + r"\tools\_d37_seal_receipts.json"))
union = json.load(open(REPO + r"\tools\_d37_lag_union.json"))
arms = json.load(open(REPO + r"\tools\_d37_lag_arms.json"))
g8new = json.load(open(REPO + r"\tools\_d37_lag_g8_newtip.json"))

# ---- start from the SEAL receipts (the 69 moves / 61 drops)
moves = {}   # lane -> list of {kind, old, new}
drops = {}   # lane -> list of {kind, id, keeper}
for r in seal_receipts:
    lane = r["lane"]
    for k, old, new in r["move_owed"]:
        moves.setdefault(lane, []).append({"kind": k, "old": old, "new": new})
    for k, old, keeper in r["drop_owed"]:
        drops.setdefault(lane, []).append({"kind": k, "id": old, "keeper": keeper})

# ---- post-SEAL MOVE additions
def mv(lane, k, old, new, why):
    moves.setdefault(lane, []).append({"kind": k, "old": old, "new": new, "why": why})
mv("g8", "Vampire", 388, 762, "WIP round-3 kind squats d37's RustyArmoredBonesAxe@388")
for k, old, new in [("ToxicSludge",762,767),("Penguin",763,768),
                    ("PenguinBlack",764,769),("GiantTortoise",765,770),
                    ("IceTortoise",766,771)]:
    mv("d36", k, old, new, "pushed 762-766 squats R6 d120 towns 763-766 + reserved 762")
mv("f42", "Tumbleweed", 354, 788, "unique; 354 squats d37 corridor")
mv("g5", "RuneWizard", 354, 789, "unique; 354 squats d37 corridor")
mv("d9", "AncientCultistSquidhead", 521, 790, "unique; 521 in d41 band")

# ---- post-SEAL DROP additions
def dp(lane, k, id_, keeper, why):
    drops.setdefault(lane, []).append({"kind": k, "id": id_, "keeper": keeper, "why": why})
# d43's 20 (11 g10 pets + 3 aliases + 6 canonical dups)
ALIAS_LANE = {"HallowBoss": "d125", "EmpressButterfly": "d125",
              "VileSpitEaterOfWorlds": "d23"}
for k, id_, why in union["d43_drops"]:
    if k in ALIAS_LANE:
        keeper = ALIAS_LANE[k]
    else:
        keeper = next((v["lane"] for kk, v in union["additions"].items() if kk == k), None)
        if keeper is None:
            seal = json.load(open(REPO + r"\tools\_d37_seal_final.json"))
            keeper = seal["owners"].get(k, ["?"])[0]
    dp("d43", k, id_, keeper, why)
# d41's 2 dups of adjudicated kinds
dp("d41", "GiantFlyingAntlion", 509, "c4", "same id 509, c4 pushed kind (SEAL keep)")
dp("d41", "SolarFlare", 516, "d23", "SEAL S0 accepted d23 SolarFlare@408; d41 dup retires")
# f18: 20 duplicate ports of canonical kinds (verify-lane)
f18 = {"ArmedZombie":"d40","Butterfly":"d39","ChatteringTeethBomb":"d39",
 "CultistArcherBlue":"d39","CultistArcherWhite":"d39","Duck":"c10","Duck2":"c10",
 "DuckWhite":"c10","DuckWhite2":"c10","Firefly":"c7","Frog":"e26","GlowingSnail":"c9",
 "Grasshopper":"d39","LightningBug":"c7","Scorpion":"d39","ScorpionBlack":"d39",
 "Snail":"c9","TruffleWorm":"c8","TruffleWormDigger":"c8","Worm":"c8"}
f18ids = {"ArmedZombie":258,"Butterfly":238,"ChatteringTeethBomb":255,
 "CultistArcherBlue":256,"CultistArcherWhite":257,"Duck":244,"Duck2":245,
 "DuckWhite":246,"DuckWhite2":247,"Firefly":237,"Frog":243,"GlowingSnail":242,
 "Grasshopper":254,"LightningBug":240,"Scorpion":249,"ScorpionBlack":248,
 "Snail":241,"TruffleWorm":252,"TruffleWormDigger":253,"Worm":239}
for k, keeper in f18.items():
    dp("f18", k, f18ids[k], keeper, "verify-lane dup port; canonical kinds-lane owns")
# d11 / f47 / d9 / g5 / f42 same-id or canonical dups
dp("d11", "BloodSquid", 619, "d42", "same id 619; d42 kinds-lane owns")
dp("f47", "ShimmerSlime", 676, "d43", "same id 676; d43 kinds-lane owns (f47 wiring folds)")
for k, id_ in [("CultistDragonHead",454),("CultistDragonBody1",455),
               ("CultistDragonBody2",456),("CultistDragonBody3",457),
               ("CultistDragonBody4",458),("CultistDragonTail",459)]:
    dp("d9", k, id_, "d40", "same id; d40 kinds-lane owns")
dp("g5", "LibrarianSkeleton", 693, "d43", "same id 693; d43 kinds-lane owns")
for k, id_, keeper in [("Antlion",356,"g8"),("BloodMummy",359,"d42"),
    ("DarkMummy",358,"d35"),("FlyingAntlion",362,"c4"),("LightMummy",360,"d35"),
    ("Mummy",357,"d35"),("Vulture",355,"d34"),("WalkingAntlion",361,"d42")]:
    dp("f42", k, id_, keeper, "dup port squats d37 corridor; canonical owns")

nm = sum(len(v) for v in moves.values())
nd = sum(len(v) for v in drops.values())
out = {"moves_total": nm, "drops_total": nd, "moves": moves, "drops": drops}
json.dump(out, open(REPO + r"\tools\_d37_lag_master.json", "w"), indent=1, sort_keys=True)
print(f"FINAL owed: {nm} moves / {nd} drops")
for lane in sorted(set(moves) | set(drops)):
    print(f"  {lane:6s} moves={len(moves.get(lane,[])):2d} drops={len(drops.get(lane,[])):2d}")
