import sys, re, subprocess
sys.stdout.reconfigure(encoding="utf8", errors="replace")
p = "crates/tr-app/src/assets.rs"
src = open(p, encoding="utf8", newline="").read()
nl = "\r\n" if "\r\n" in src else "\n"
head = subprocess.run(["git","show","HEAD:crates/tr-app/src/assets.rs"],capture_output=True).stdout.decode("utf8","replace")
# extract the d40 band from HEAD's vec (442..487)
i = head.index('atlas.sheet("NPC_442").copied(),')
m = head.index('atlas.sheet("NPC_487").copied(),')
m = head.index("\n", m) + 1
band = head[i:m]
# normalize line endings
band = band.replace("\r\n", "\n").replace("\n", nl)
# insert after NPC_438 in the current vec
anchor = 'atlas.sheet("NPC_438").copied(),'
k = src.index(anchor) + len(anchor)
if src[k:k+len(nl)] == nl: k += len(nl)
src = src[:k] + band + src[k:]
open(p, "w", encoding="utf8", newline="").write(src)
print("d40 vec band restored")
