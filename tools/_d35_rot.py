# W4-d35 sheets phase: rotation-write census per aiStyle region in NPC.cs AI().
# Regions (from the aiStyle dispatch): 2@19950, 5@20802, 9@21377, 13@22615,
# 14@22849, 16@23492, 18@24153, 22@24666, 23@25235, 24@25313, 25@25538,
# 26 = AI_026_Unicorns@62366. Prints every `rotation` write with line numbers.
import io, re
src = io.open(r"C:\Users\DreadLord\Desktop\TR-W4-d35\decompiled\Terraria\NPC.cs",
              encoding="utf-8", errors="replace").read().splitlines()
regions = [(2,19950,20802),(5,20802,21377),(9,21377,22615),(13,22615,22849),
           (14,22849,23492),(16,23492,24153),(18,24153,24666),(22,24666,25235),
           (23,25235,25313),(24,25313,25538),(25,25538,26100),(26,62366,62800)]
for style, a, b in regions:
    print(f"=== aiStyle {style} (lines {a}-{b})")
    for i in range(a-1, min(b, len(src))):
        l = src[i]
        if re.search(r"\brotation\b", l) and "=" in l and "==" not in l.split("rotation")[0][-3:]:
            print(f"  {i+1}: {l.strip()}")
