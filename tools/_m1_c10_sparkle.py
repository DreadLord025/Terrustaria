import sys
sys.stdout.reconfigure(encoding="utf8", errors="replace")
p = "crates/tr-app/src/world_sim/npcs.rs"
src = open(p, encoding="utf8", newline="").read()
nl = "\r\n" if "\r\n" in src else "\n"
bt = chr(96)
old = ("                            }" + nl +
       "                        // W4-c10 roster tail — " + bt + "UpdateNPC_CastLights" + bt + "'s")
new = ("                            }" + nl +
       "                        }" + nl +
       "                        // W4-c10 roster tail — " + bt + "UpdateNPC_CastLights" + bt + "'s")
assert old in src, "not found"
src = src.replace(old, new)
open(p, "w", encoding="utf8", newline="").write(src)
print("GoldWorm sparkle if closed")
