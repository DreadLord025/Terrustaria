import sys
sys.stdout.reconfigure(encoding="utf8", errors="replace")
p = "crates/tr-app/src/world_sim/npcs.rs"
src = open(p, encoding="utf8", newline="").read()
nl = "\r\n" if "\r\n" in src else "\n"
bt = chr(96)
old = ("            if n.kind.ai_style() == 66 {" + nl +
       "                n.ess_scale = ess;" + nl +
       "            }" + nl +
       "    // W4-c10 — " + bt + "Player.IsThisCenx()" + bt + " pre-stamp")
new = ("            if n.kind.ai_style() == 66 {" + nl +
       "                n.ess_scale = ess;" + nl +
       "            }" + nl +
       "        }" + nl +
       "    }" + nl +
       "    // W4-c10 — " + bt + "Player.IsThisCenx()" + bt + " pre-stamp")
assert old in src, "not found"
src = src.replace(old, new)
open(p, "w", encoding="utf8", newline="").write(src)
print("ess_scale block closed")
