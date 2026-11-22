import sys
sys.stdout.reconfigure(encoding="utf8", errors="replace")
p = "crates/tr-app/src/npc.rs"
src = open(p, encoding="utf8", newline="").read()
nl = "\r\n" if "\r\n" in src else "\n"
i = src.index("    /// W4-c9 — the GoldBird (442) through the faithful")
j = src.index("    /// W4-c10 — the aiStyle-68 adapter:")
repl = ("    // W4-c10 merge (E8): the c9 update_gold_bird wrapper retired - the" + nl +
        "    // c10 axis-24 arm (ai_024_bird) owns the GoldBird (the superset;" + nl +
        "    // the truncated union body dropped)." + nl + nl)
src = src[:i] + repl + src[j:]
open(p, "w", encoding="utf8", newline="").write(src)
print("update_gold_bird retired")
