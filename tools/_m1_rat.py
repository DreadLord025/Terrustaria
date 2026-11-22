import sys, re, subprocess
sys.stdout.reconfigure(encoding="utf8", errors="replace")
head = subprocess.run(["git","show","HEAD:crates/tr-app/src/npc.rs"],capture_output=True).stdout.decode("utf8","replace")
bt = chr(96)
i = head.index("// W4-d42 - " + bt + "FindFrame" + bt + " case 610")
j = head.index("\n            }", i)
print(head[i:j+15])
