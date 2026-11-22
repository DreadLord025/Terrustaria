# For the c4 drops: which surfaces do the canonical owners already cover?
import re, sys, subprocess, collections
sys.stdout.reconfigure(encoding="utf8", errors="replace")
FILES = ["crates/tr-app/src/npc/data.rs","crates/tr-app/src/npc.rs",
         "crates/tr-app/src/bestiary.rs","crates/tr-app/src/assets.rs",
         "crates/tr-app/src/npc_buffs.rs","crates/tr-app/src/render/entities.rs",
         "crates/tr-app/src/ui/bestiary.rs","crates/tr-app/src/app/netglue.rs",
         "crates/tr-app/src/world_sim/npcs.rs","crates/tr-app/src/audio.rs"]
KINDS = sys.argv[1:] or ["BloodFeeder","FlyingFish","CorruptGoldfish","Pufferfish","Orca"]
for k in KINDS:
    print(f"== {k}")
    for f in FILES:
        src = open(f, encoding="utf8").read()
        # find all fn-scoped occurrences
        fns = [(m.start(), m.group(1)) for m in re.finditer(r'^\s*(?:pub(?:\([^)]*\))?\s+)?(?:const\s+)?fn\s+(\w+)', src, re.M)]
        hits = collections.Counter()
        for m in re.finditer(r"NpcKind::" + k + r"\b", src):
            prev = [n for a, n in fns if a <= m.start()]
            hits[prev[-1] if prev else "<top>"] += 1
        if hits: print("  ", f.split("/")[-1], dict(hits))
