import sys, re, subprocess
sys.stdout.reconfigure(encoding="utf8", errors="replace")
p = "crates/tr-app/src/assets.rs"
src = open(p, encoding="utf8", newline="").read()
nl = "\r\n" if "\r\n" in src else "\n"
head = subprocess.run(["git","show","HEAD:crates/tr-app/src/assets.rs"],capture_output=True).stdout.decode("utf8","replace")
# extract the d40 band from HEAD's STATIC_SHEET_NAMES (442..482, incl 455-459/478)
i = head.index('"NPC_442",')
j = head.index('"NPC_482",')
j = head.index("\n", j) + 1
band = head[i:j].replace("\r\n", "\n").replace("\n", nl)
# insert after NPC_438 in the current STATIC_SHEET_NAMES
anchor = '"NPC_438",'
k = src.index(anchor) + len(anchor)
if src[k:k+len(nl)] == nl: k += len(nl)
src = src[:k] + band + src[k:]
open(p, "w", encoding="utf8", newline="").write(src)
print("d40 STATIC_SHEET_NAMES band restored")
