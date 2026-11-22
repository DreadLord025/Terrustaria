import sys, re
sys.stdout.reconfigure(encoding="utf8", errors="replace")
src = open("crates/tr-app/src/world_sim/npcs.rs", encoding="utf8").read()
i = src.index("// W4-c10 roster tail — the GoldButterfly's threat-scan")
k = src.rfind("{", 0, i)
j = src.index("// The single session player as slot 0", i)
seg = src[k:j]
print("brace delta c10 block:", seg.count("{") - seg.count("}"))
print(repr(src[k-60:k+60]))
# the c9 Shimmerfly pre-pass: was it a bare block? Check HEAD.
import subprocess
head = subprocess.run(["git","show","HEAD:crates/tr-app/src/world_sim/npcs.rs"],capture_output=True).stdout.decode("utf8","replace")
m = head.index("shimmerfly_threat_views = views.clone()")
print("HEAD context:", repr(head[m-700:m+120]))
