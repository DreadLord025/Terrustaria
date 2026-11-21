# W4-d37 E8-RECONCILE-SEAL: FINAL canonical plan at CURRENT origin tips
# (live-wave re-fetch). Owner-per-kind from census + depth-evidenced
# adjudications; same-vanilla-different-name collapse by net-id; final-id
# assignment; zero-collision / zero-base-clash / i16-safe verification.
import re, subprocess, json, collections

REPO = r"C:\Users\DreadLord\Desktop\TR-W4-d37"
def show(rev, path):
    r = subprocess.run(["git", "-C", REPO, "show", f"{rev}:{path}"], capture_output=True)
    return r.stdout.decode("utf8", "replace") if r.returncode == 0 else None
DATA = "crates/tr-app/src/npc/data.rs"
def fn_body(src, sig):
    i = src.index(sig); j = src.index("{", i); d = 0; k = j
    while True:
        ch = src[k]
        if ch == "{": d += 1
        elif ch == "}":
            d -= 1
            if d == 0: return src[j:k+1]
        k += 1
def arms_to(src):
    return {m[0]: int(m[1]) for m in re.findall(r"NpcKind::(\w+)\s*=>\s*(\d+)", fn_body(src, "pub fn to_u16"))}

census = json.load(open(REPO + r"\tools\_d37_e8s_census.json"))["lanes"]
base_to = arms_to(show("a725acb04", DATA))
base_names = set(base_to); base_ids = set(base_to.values())
LANES = ["c3","c4","c6","c7","c8","c9","c10","d9","d11","d23","d120","d125",
         "d33","d34","d35","d36","d37","d38","d39","d40","d41","d42","d43",
         "e26","f18","f42","f47","g5","g8","g10","c7b","d103","e21","g62"]
ports = collections.defaultdict(list)
for l in LANES:
    for k, v in census[l]["new"].items(): ports[k].append((l, v))

OWNER = {}
def setowner(k, lane): OWNER[k] = lane
# --- keep-receipt anchors (d34 266-280 / d38 281-337 / d37 354-399 / d40 430-488)
for k in ("CursedSkull","ManEater","Snatcher","CorruptGoldfish","Vulture","Demon","Harpy","BlueJellyfish"):
    setowner(k, "d34")
setowner("GiantCursedSkull", "d38"); setowner("PumpkingBlade", "d38")
for k in ("FlyingFish","FlyingSnake","Squid","BloodFeeder","FungiBulb","GiantFungiBulb"):
    setowner(k, "d37")
setowner("CrimsonGoldfish", "c4")                       # R2
# d35 band + its g8/d23/f42 dups:
for k in ("Pixie","Wraith","CursedHammer","EnchantedSword","Mimic","GreenJellyfish",
          "Gastropod","GiantBat","Slimer","IlluminantBat","Clinger","VileSpit",
          "Mummy","DarkMummy","LightMummy"):
    setowner(k, "d35")
# d42 executed 578-630 band minus 7 dragonflies (d33):
for k in ("BartenderUnconscious","WalkingAntlion","BloodMummy","Seagull","Seagull2",
          "Grebe","Grebe2","BloodSquid"):
    setowner(k, "d42")
for k in ("BlackDragonfly","BlueDragonfly","GreenDragonfly","OrangeDragonfly",
          "RedDragonfly","YellowDragonfly","GoldDragonfly"):
    setowner(k, "d33")
# d40 CultistDragon fam (d9 dups drop) + gold critters/crawlers (c9/c10/c8 drop):
for k in ("CultistDragonHead","CultistDragonBody1","CultistDragonBody2","CultistDragonBody3",
          "CultistDragonBody4","CultistDragonTail","GoldBird","GoldBunny","GoldButterfly",
          "GoldFrog","GoldGrasshopper","GoldMouse","GoldWorm","EnchantedNightcrawler",
          "Grubby","Sluggy","Buggy","ArmedZombie"):
    setowner(k, "d40")
setowner("AncientCultistSquidhead", "d9")               # NEW 521
# c4 fish-five (SEAL flip) + Arapaima + Pufferfish/Orca (c4 wired re-puff):
for k in ("FlyingAntlion","EyeballFlyingFish","GoldGoldfish","Pupfish","Dolphin",
          "Arapaima","Pufferfish","Orca"):
    setowner(k, "c4")
setowner("GiantFlyingAntlion", "c4")                    # LAGGARD-EXEC: keep c4@509 (style44 axis + overrides);
                                                        # d41 drops its dup row, dedicated fn composes at merge
# g8 EXECUTED machine-wiring band 745-762 (18 unique incl. Vampire):
for k in ("PinkJellyfish","Antlion","SpikeBall","WallCreeper","WallCreeperWall",
          "IceElemental","Herpling","Derpling","CrimsonAxe","FloatyGross","LostGirl",
          "Nymph","IceBat","Lavabat","GiantFlyingFox","VampireBat","RedDevil","Vampire"):
    setowner(k, "g8")
# small lanes:
for k in ("Firefly","LightningBug","Lavafly","Shimmerfly"): setowner(k, "c7")
for k in ("Worm","TruffleWorm","TruffleWormDigger","Maggot"): setowner(k, "c8")
for k in ("Snail","GlowingSnail","MagmaSnail"): setowner(k, "c9")
for k in ("Duck","Duck2","DuckWhite","DuckWhite2"): setowner(k, "c10")
for k in ("Frog","LadyBug","GoldLadyBug"): setowner(k, "e26")
setowner("AngryTrapper", "d23")
for k in ("SantaClaus","Truffle","Steampunker","Cyborg"): setowner(k, "d120")
# d125 EXECUTED empress 636 / lacewing 742 win over d43 unwired stubs (depth):
setowner("EmpressOfLight", "d125"); setowner("PrismaticLacewing", "d125")
setowner("LibrarianSkeleton", "g5")                     # g5 caster+1092 book wired (d43-retire concurs)
setowner("WaterBoltMimic", "c3")                        # d43-retire: 694->c3 style10 skull lane canonical
setowner("Pufferfish", "d43"); setowner("Orca", "d43")  # d43-retire conditional FIRED: d43 owns 688/692,
                                                        # absorbed the just_hit puff-fix + Orca wet-anchor
setowner("VileSpitEaterOfWorlds", "d43")                # style9 axis wired
setowner("QueenSlimeMinionPurple", "d23")               # d43-retire: ->d23 (style-14 axis update_cave_bat faithful)
setowner("SporeBat", "d23")                             # d43-retire: ->d23 (style-14 axis faithful)
for k in ("TownCat","TownDog","TownBunny","TownSlimeBlue","TownSlimeGreen","TownSlimeOld",
          "TownSlimePurple","TownSlimeRainbow","TownSlimeRed","TownSlimeYellow","TownSlimeCopper"):
    setowner(k, "g10")
setowner("RuneWizard", "g5")                            # pushed 827
setowner("Tumbleweed", "f42")                           # -> vanilla-free 546
setowner("ShimmerSlime", "f47")                         # shimmer-dust wired
# d39 seven 817-823; d41 thirty-eight 494-537 (incl. SolarFlare 516 entry-less C#-faithful);
# d36 eight 767-793:
setowner("SolarFlare", "d23")                           # LAGGARD-EXEC: d23 keeps (own visible ENTRIES row
                                                        # net 516); d41's entry-less dup drops. 408 free.
for k in census["d39"]["new"]: setowner(k, "d39")
for k in census["d41"]["new"]:
    if k not in OWNER: setowner(k, "d41")
for k in census["d36"]["new"]:
    if k not in OWNER: setowner(k, "d36")
setowner("GiantFlyingAntlion", "c4")                    # LAGGARD-EXEC final: c4 keeps 509
# d34 phase-4: Crab/DungeonGuardian NEW (re-pin off d38's 281-285 squat;
# LAGGARD-EXEC awards 791/792, both free — compose):
setowner("Crab", "d34"); setowner("DungeonGuardian", "d34")
# NEW claimants surfaced after LAGGARD-EXEC (E8-SEAL re-sweep):
setowner("PartyBunny", "e21")                           # NEW cs 540 -> vanilla-free 540 (237 is a
                                                        # d37-owned cs hole, re-pin off it)
setowner("Wolf", "d36")                                 # NEW 772 (d36 hover batch)
# d33 NOW wires the fairy/balloon machines (axis 112/113) — wired superset wins
# over d42's data-only {} stubs (the R3 dragonfly precedent extends):
for k in ("FairyCritterPink","FairyCritterGreen","FairyCritterBlue","WindyBalloon"):
    setowner(k, "d33")
setowner("Bunny", "d34")                                # d34@270 keep-receipt > e21@229 hole-squat
setowner("GoblinShark", "d42")                          # d42 executed 620 > g62 dup
# c7b (c7-style65-critters) ports the SAME four firefly kinds at the SAME ids as
# c7 (style64) — one canonical c7 owner, no new kind. d103 Wraith / g62 GoblinShark+
# Shimmerfly / e21 Bunny / d33 fairy-critters+WindyBalloon are all dups -> drop.
# auto-owner singletons
for k, pl in ports.items():
    if k not in OWNER:
        if len(pl) == 1: OWNER[k] = pl[0][0]
        else: print("UNASSIGNED contested:", k, pl)

# --- same-vanilla-different-name collapse (net-id equality)
net_of = {}
for l in LANES:
    for k, n in census[l]["new_entries"].items():
        if n and n > 0: net_of.setdefault(k, n)
by_net = collections.defaultdict(list)
for k, n in net_of.items():
    if k in OWNER: by_net[n].append(k)
collapses = {n: ks for n, ks in by_net.items() if len(ks) > 1}
print("same-net different-name groups:", collapses)
SURVIVOR = {636: "EmpressOfLight", 661: "PrismaticLacewing", 666: "VileSpitEaterOfWorlds"}
dropped_names = set()
for n, ks in collapses.items():
    surv = SURVIVOR.get(n)
    if surv is None: print("NO SURVIVOR for net", n, ks); continue
    for k in ks:
        if k != surv: dropped_names.add(k)
print("collapsed-away names:", sorted(dropped_names))

# --- final-id assignment
RENUM = {}
for k, pid in sorted(census["d35"]["new"].items(), key=lambda x: x[1]):
    RENUM[k] = pid + 269                                # d35 +269 order-preserving
for k, fid in (("Firefly",735),("LightningBug",736),("Worm",737),("TruffleWorm",738),
               ("TruffleWormDigger",739),("Snail",740),("GlowingSnail",741),
               ("AngryTrapper",743),("Duck",731),("Duck2",732),("DuckWhite",733),
               ("DuckWhite2",734),("Tumbleweed",546),("Crab",791),("DungeonGuardian",792),
               ("PartyBunny",540),
               ("FairyCritterPink",583),("FairyCritterGreen",584),
               ("FairyCritterBlue",585),("WindyBalloon",594),
               ("SporeBat",634),("QueenSlimeMinionPurple",660),
               ("SantaClaus",763),("Truffle",764),("Steampunker",765),("Cyborg",766)):
    RENUM[k] = fid
# executed ids accepted as-pushed: e26 Frog 744, d125 636/742, d36 767-793,
# g8 745-762, d39 817-823, d41 494-537, g5 RuneWizard 827, d42 578-630, c4 406.

owners = {}; final = {}
for k, lane in OWNER.items():
    if k in dropped_names: continue
    pid = census[lane]["new"][k]
    owners[k] = [lane, pid]; final[k] = RENUM.get(k, pid)

ids = {}; clash = []
for k, v in final.items():
    if v in ids: clash.append((v, ids[v], k))
    ids[v] = k
print("owned kinds:", len(owners), "distinct final ids:", len(ids))
print("id clashes:", clash)
print("base clash:", [(k, v) for k, v in final.items() if v in base_ids])
print("max id:", max(ids), "i16-safe:", max(ids) <= 32767)
print("ALL_KINDS / to / from / s1_f_3 pin:", 270 + len(owners))
pl = collections.Counter(v[0] for v in owners.values())
print("per-lane owned:", dict(sorted(pl.items())))
json.dump({"owners": owners, "final": final, "dropped_names": sorted(dropped_names)},
          open(REPO + r"\tools\_d37_e8s_plan.json", "w"), indent=1, sort_keys=True)
print("plan written")
