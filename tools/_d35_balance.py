# W4-d35 sheets phase: crude brace/paren balance per touched file, with
# line comments and string literals stripped (a smoke check for the data
# wave's splice damage; NOT a compiler).
import io, re, sys
FILES = [
    "crates/tr-app/src/npc.rs",
    "crates/tr-app/src/npc/data.rs",
    "crates/tr-app/src/render/entities.rs",
    "crates/tr-app/src/ui/bestiary.rs",
    "crates/tr-app/src/app/netglue.rs",
    "crates/tr-app/src/npc_buffs.rs",
    "crates/tr-app/src/audio.rs",
    "crates/tr-app/src/bestiary.rs",
    "crates/tr-app/src/assets.rs",
]
ROOT = r"C:\Users\DreadLord\Desktop\TR-W4-d35"
for f in FILES:
    s = io.open(ROOT + "\\" + f.replace("/", "\\"), encoding="utf-8").read()
    s = re.sub(r'"(?:[^"\\]|\\.)*"', '""', s)
    s = re.sub(r"//[^\n]*", "", s)
    s = re.sub(r"/\*.*?\*/", "", s, flags=re.S)
    print(f, "braces", s.count("{") - s.count("}"),
          "parens", s.count("(") - s.count(")"),
          "brackets", s.count("[") - s.count("]"))
