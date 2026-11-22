import sys, re
sys.stdout.reconfigure(encoding="utf8", errors="replace")
src = open("crates/tr-app/src/npc.rs", encoding="utf8").read()
lines = src.split("\n")
i = [k for k, l in enumerate(lines) if "fn w4_c8_the_crawler_defaults" in l][0]
j = [k for k, l in enumerate(lines) if "the axis claims aiStyle 64" in l and k > i][0]
print("\n".join(f"{k+1}: {lines[k]}" for k in range(j-34, j+4)))
