# W4-c10 merge bestiary.rs: drop the c10 dup entries (Seagull/Grebe ->
# d42's, GoldButterfly/GoldMouse -> d40's), drop the c10 Seagull2/Grebe2
# entries (they fold onto Seagull/Grebe - no own entry, the d42 design).
import sys, re
sys.stdout.reconfigure(encoding="utf8", errors="replace")
p = "crates/tr-app/src/bestiary.rs"
src = open(p, encoding="utf8", newline="").read()
nl = "\r\n" if "\r\n" in src else "\n"

def drop_entry(kind, occurrence=0):
    global src
    hits = list(re.finditer(r'    BestiaryEntryDef\s*\{[^}]*kind:\s*NpcKind::' + kind + r'\b[^}]*\},?' + re.escape(nl), src, re.S))
    m = hits[occurrence]
    src = src[:m.start()] + src[m.end():]

# the c10 Seagull/Grebe (the FIRST occurrences - the d42 ones are later)
drop_entry("Seagull", 0)
drop_entry("Grebe", 0)
# the c10 Seagull2/Grebe2 (the only occurrences - they should not exist)
drop_entry("Seagull2", 0)
drop_entry("Grebe2", 0)
# the c10 GoldButterfly/GoldMouse (the SECOND occurrences - d40's are first)
drop_entry("GoldButterfly", 1)
drop_entry("GoldMouse", 1)
open(p, "w", encoding="utf8", newline="").write(src)
print("bestiary dup entries dropped")
