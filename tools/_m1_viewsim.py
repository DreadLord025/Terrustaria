import sys, re
sys.stdout.reconfigure(encoding="utf8", errors="replace")
src = open("crates/tr-app/src/world_sim/npcs.rs", encoding="utf8").read()
lines = src.split("\n")
i = [k for k, l in enumerate(lines) if "W4-c10 roster tail — the GoldButterfly" in l][0]
print("\n".join(f"{k+1}: {lines[k]}" for k in range(i-42, i+28)))
