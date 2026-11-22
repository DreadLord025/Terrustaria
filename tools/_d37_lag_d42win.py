# W4-d37 LAGGARD-EXEC: read the ambiguous regions of the d42 dragonfly
# drop sites at bac408be0 (windows <=100 lines), to cut exact 7-only hunks.
import subprocess

REPO = r"C:\Users\DreadLord\Desktop\TR-W4-d37"
REV = "bac408be0"

def win(p, a, b):
    r = subprocess.run(["git", "-C", REPO, "show", f"{REV}:{p}"],
                       capture_output=True)
    ls = r.stdout.decode("utf8", "replace").split("\n")
    print(f"---- {p} {a}-{b}")
    for i in range(a - 1, min(b, len(ls))):
        print(f"{i+1}: {ls[i][:110]}")

N = "crates/tr-app/src/npc.rs"
D = "crates/tr-app/src/npc/data.rs"
B = "crates/tr-app/src/bestiary.rs"
E = "crates/tr-app/src/render/entities.rs"
win("crates/tr-app/src/npc_buffs.rs", 508, 526)
win(D, 2712, 2726)      # ai_style
win(D, 3313, 3327)      # size
win(D, 4091, 4105)      # max_hp
win(D, 4509, 4523)      # contact_damage
win(N, 4178, 4192)      # defense
win(N, 4604, 4618)      # value
win(N, 19957, 19971)    # render_key
win(E, 3416, 3424)      # draw comment boundary
win(B, 2655, 2662)      # fold comment boundary
