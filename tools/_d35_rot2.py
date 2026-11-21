# W4-d35 sheets phase: rotation-write -> case-label map for NPC.FindFrame,
# plus the full AI_026_Unicorns function scan. Prints each `this.rotation`
# write with the nearest preceding `case N:` labels (the switch group).
import io, re
src = io.open(r"C:\Users\DreadLord\Desktop\TR-W4-d35\decompiled\Terraria\NPC.cs",
              encoding="utf-8", errors="replace").read().splitlines()

# Find the FindFrame function bounds.
start = next(i for i, l in enumerate(src) if "public void FindFrame(" in l)
# next method at ANY indent >= 1 tab after start
end = next((i for i in range(start + 1, len(src))
            if re.match(r"\t(public|private|internal|protected).*\(", src[i])), len(src))
print("FindFrame:", start + 1, "-", end + 1)
cases = []
for i in range(start, end):
    m = re.match(r"\t\t\tcase (\d+):", src[i])
    if m:
        cases.append(int(m.group(1)))
    if re.search(r"this\.rotation\s*[+\-]?=", src[i]):
        print(f"  {i+1}: cases={cases[-8:]} :: {src[i].strip()[:100]}")
        cases = []  # reset group after a write? no - keep collecting per write
# AI_026_Unicorns full extent
u = next(i for i, l in enumerate(src) if "private void AI_026_Unicorns(" in l)
uend = next((i for i in range(u + 1, len(src))
             if re.match(r"\t(public|private|internal|protected).*\(", src[i])), len(src))
print("AI_026_Unicorns:", u + 1, "-", uend + 1)
for i in range(u, uend):
    if re.search(r"this\.rotation\s*[+\-]?=", src[i]):
        print(f"  {i+1}: {src[i].strip()[:100]}")
