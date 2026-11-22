import sys
sys.stdout.reconfigure(encoding="utf8", errors="replace")
p = "crates/tr-app/src/npc.rs"
src = open(p, encoding="utf8", newline="").read()
nl = "\r\n" if "\r\n" in src else "\n"
# the grasshopper fn's last else-if block is missing its close; the orphaned
# bird tail (8575-8596) follows. Close the grasshopper block + fn, drop the tail.
i = src.index("            self.collide_y = true;" + nl + "        let ty = self.kind.cs_id() as i32;" + nl + "        let out = crate::npc_ai_style24::ai_style_24(self, ty, world, &ctx, rng);")
# find the end of the orphaned bird tail (the integrate_flyer_collision + fn close)
j = src.index("        self.integrate_flyer_collision(world);" + nl + "    }", i)
j += len("        self.integrate_flyer_collision(world);" + nl + "    }")
repl = ("            self.collide_y = true;" + nl + "        }" + nl + "    }")
src = src[:i] + repl + src[j:]
open(p, "w", encoding="utf8", newline="").write(src)
print("grasshopper fn closed, bird tail dropped")
