import sys, re, subprocess
sys.stdout.reconfigure(encoding="utf8", errors="replace")
head = subprocess.run(["git","show","HEAD:crates/tr-app/src/world_sim/npcs.rs"],capture_output=True).stdout.decode("utf8","replace")
hl = head.split("\n")
i = [k for k, l in enumerate(hl) if "The small-kinds wiring wave — the ForceBubble" in l][0]
print("\n".join(f"{k+1}: {hl[k]}" for k in range(i-4, i+2)))
j = [k for k, l in enumerate(hl) if "if let Some(player) = player {" in l][0]
print("HEAD if-let at", j+1)
# the c9 shimmerfly pre-pass in HEAD
m = [k for k, l in enumerate(hl) if "shimmerfly_threat_views = views.clone()" in l][0]
print("\n".join(f"{k+1}: {hl[k]}" for k in range(m-30, m+6)))
