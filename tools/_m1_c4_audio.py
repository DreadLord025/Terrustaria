# W4-c4 merge audio dedupe: drop the c4 dup bank rows (d42/d37 rows kept).
import sys, os
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from _m1_rep import make_rep
p = "crates/tr-app/src/audio.rs"
src = open(p, encoding="utf8", newline="").read()
box = [src]
rep = make_rep(lambda: box[0], lambda v: box.__setitem__(0, v))
rep("""    // W4-c4 (the style-44 swimmers): the antlion pair's NPCHit32
    // (`NPC.cs:14850`/`:14877`) - the `Sounds/NPC_Hit_32.xnb` asset
    // exists; the S1-E-21 wave C bank-gap close.
    ("NPC_Hit_32", 3),
    // W4-c4 (the style-16 swimmers): the BloodFeeder's NPCHit13
    // (`NPC.cs:12115`) - the `Sounds/NPC_Hit_13.xnb` asset exists; the
    // S1-E-21 wave C bank-gap close.
    ("NPC_Hit_13", 3),
""", """    // W4-c4 merge dedupe: the antlion pair's NPCHit32 is the d42 row
    // above; the BloodFeeder's NPCHit13 is the d37 row below (one bank
    // row per cue - the spine-end precedent).
""")
rep("""    // W4-c4 (the style-16 swimmers): the BloodFeeder's NPCDeath19
    // (`NPC.cs:12116`) - the `Sounds/NPC_Killed_19.xnb` asset exists; the
    // S1-E-21 wave C bank-gap close.
    ("NPC_Killed_19", 4),
""", """    // W4-c4 merge dedupe: the BloodFeeder's NPCDeath19 is the d37 row
    // below (one bank row per cue).
""")
rep("""    // W4-c4 (the style-44 swimmers): the antlion pair's NPCDeath35
    // (`NPC.cs:14851`/`:14878`) - the `Sounds/NPC_Killed_35.xnb` asset
    // exists; the bank-gap close.
    ("NPC_Killed_35", 4),
""", """    // W4-c4 merge dedupe: the antlion pair's NPCDeath35 is the d42 row
    // above (one bank row per cue).
""")
open(p, "w", encoding="utf8", newline="").write(box[0])
print("audio dedupe done")
