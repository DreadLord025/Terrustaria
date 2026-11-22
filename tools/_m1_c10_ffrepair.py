# Repair the c10 find_frame union breaks: the waterfowl swimmer arm's
# flight-band clamp + the Rat arm's walk branch (the c10 drops ate the
# closing braces). Restore from HEAD's d42 Rat arm + the c10 swimmer body.
import sys, re, subprocess
sys.stdout.reconfigure(encoding="utf8", errors="replace")
p = "crates/tr-app/src/npc.rs"
src = open(p, encoding="utf8", newline="").read()
nl = "\r\n" if "\r\n" in src else "\n"

# 1. The waterfowl swimmer flight-band clamp: the c10 arm has
#    `if self.frame >= 15 {` then `if self.frame >= self.rows() {` (the
#    d42 body interleaved). The c10 body is the keeper (15-row literal).
old = ("                    if self.frame >= 15 {" + nl +
       "                    if self.frame >= self.rows() {" + nl +
       "                        self.frame = 11;" + nl +
       "                    }")
new = ("                    if self.frame >= 15 {" + nl +
       "                        self.frame = 11;" + nl +
       "                    }")
assert old in src, "swimmer clamp not found"
src = src.replace(old, new)

# 2. The Rat arm: the c10 drop ate the walk branch's tail. Restore the
#    d42 Rat arm body (the walk accumulator + the air pin).
old = ("                        self.frame_counter += self.vx.abs() * 0.5;" + nl +
       "                        self.frame_counter += 1.0;" + nl +
       "            // E8 merge: the c10 crawler find_frame dups dropped - c8's")
new = ("                        self.frame_counter += self.vx.abs() * 0.5;" + nl +
       "                        self.frame_counter += 1.0;" + nl +
       "                        if self.frame_counter > 6.0 {" + nl +
       "                            self.frame += 1;" + nl +
       "                            self.frame_counter = 0.0;" + nl +
       "                        }" + nl +
       "                        if self.frame >= self.rows() {" + nl +
       "                            self.frame = 2;" + nl +
       "                        }" + nl +
       "                    }" + nl +
       "                } else {" + nl +
       "                    self.frame_counter = 0.0;" + nl +
       "                    self.frame = 0;" + nl +
       "                }" + nl +
       "            }" + nl +
       "            // E8 merge: the c10 crawler find_frame dups dropped - c8's")
assert old in src, "Rat walk branch not found"
src = src.replace(old, new)

open(p, "w", encoding="utf8", newline="").write(src)
print("find_frame union breaks repaired")
