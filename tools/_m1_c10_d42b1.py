# Restore the d42 b1 witness body (the c10 waterfowl witness insert
# truncated it). Insert the missing body between the for-loop header
# and the c10 waterfowl doc.
import sys, re, subprocess
sys.stdout.reconfigure(encoding="utf8", errors="replace")
p = "crates/tr-app/src/npc.rs"
src = open(p, encoding="utf8", newline="").read()
nl = "\r\n" if "\r\n" in src else "\n"
head = subprocess.run(["git","show","HEAD:crates/tr-app/src/npc.rs"],capture_output=True).stdout.decode("utf8","replace")
# the d42 b1 body from the for-loop header to the fn close
i = head.index("fn w4_d42_b1_the_defaults_match_the_csharp_rows")
j = head.index("\n    }", i) + len("\n    }")
full = head[i:j].replace("\r\n", "\n").replace("\n", nl)
# re-pin the ALL_KINDS.len to 512
full = full.replace("assert_eq!(ALL_KINDS.len(), 508);", "assert_eq!(ALL_KINDS.len(), 512);")
# the current file: the d42 b1 fn header + rows + for-loop header, then the
# c10 waterfowl doc. Replace the truncated d42 b1 with the full body.
i2 = src.index("fn w4_d42_b1_the_defaults_match_the_csharp_rows")
j2 = src.index("    /// W4-c10 — the waterfowl family's `SetDefaults` rows pin against the", i2)
src = src[:i2] + full + nl + nl + src[j2:]
open(p, "w", encoding="utf8", newline="").write(src)
print("d42 b1 witness restored")
