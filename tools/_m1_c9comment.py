import sys
sys.stdout.reconfigure(encoding="utf8", errors="replace")
p = "crates/tr-app/src/world_sim/npcs.rs"
src = open(p, encoding="utf8", newline="").read()
nl = "\r\n" if "\r\n" in src else "\n"
bt = chr(96)
old = "                // later slots. RNG-free by construction." + nl + "                {"
new = ("                // W4-c9 — the Shimmerfly's threat-scan pool" + nl +
       "                // (" + bt + "NPC.cs:34120-34128" + bt + ": " + bt + "Main.npc[0..maxNPCs]" + bt + " in slot" + nl +
       "                // order — active, damage > 0, !friendly, the int-truncated" + nl +
       "                // Hitbox). The per-NPC " + bt + "&mut self" + bt + " update cannot see its" + nl +
       "                // siblings, so the views are stamped here before the slot" + nl +
       "                // loop (the twin_time_lefts carrier precedent). ORDER" + nl +
       "                // DEVIATION (documented, the ForceBubble pre-pass's" + nl +
       "                // shape): the C# scans at the fly's own slot turn; the" + nl +
       "                // port reads the tick-start pool — one tick of lag for" + nl +
       "                // later slots. RNG-free by construction." + nl +
       "                {")
assert old in src
src = src.replace(old, new)
open(p, "w", encoding="utf8", newline="").write(src)
print("c9 comment restored")
