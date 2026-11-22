# W4-d37 SEAL item 2: emit the EXACT c9 ALL_KINDS fix-hunk as a unified diff
# against origin/task/W4-c9-style67-snails tip 2042d77f1. The four snail-
# family kinds are in to_u16/from_u16 (274 arms, bijective) but ABSENT from
# the ALL_KINDS array (270 rows) -> the round-trip test's
# `decodable == ALL_KINDS.len()` assert FAILS (274 != 270). Fix: add the 4
# rows, bump the decl 270->274 and the s1_f_3 literal pin 270->274.
import subprocess
REPO=r"C:\Users\DreadLord\Desktop\TR-W4-d37"
src=subprocess.run(["git","-C",REPO,"show","2042d77f1:crates/tr-app/src/npc.rs"],
                   capture_output=True).stdout.decode("utf8","replace")
lines=src.split("\n")
# locate the three edit sites (1-indexed in the file)
decl_i=next(i for i,l in enumerate(lines) if "const ALL_KINDS: [NpcKind; 270]" in l)
close_i=next(i for i in range(decl_i,len(lines)) if lines[i].strip()=="];")
pin_i=next(i for i,l in enumerate(lines) if l.strip()=="270," and i>close_i)
print("decl line",decl_i+1,"close line",close_i+1,"pin line",pin_i+1)
print("decl:",lines[decl_i])
print("row before close:",lines[close_i-1])
print("pin ctx:",lines[pin_i-1].strip(),"|",lines[pin_i].strip())
# build the patched file
new=lines[:]
new[decl_i]=new[decl_i].replace("[NpcKind; 270]","[NpcKind; 274]")
ins=["        // The style-67 snail family (W4-c9): the four kinds were in",
     "        // to_u16/from_u16 but missing here, so the round-trip test's",
     "        // `decodable == ALL_KINDS.len()` assert failed (274 != 270).",
     "        // Shimmerfly drops to c7 at the 108 merge; the trio stays.",
     "        NpcKind::Snail,",
     "        NpcKind::GlowingSnail,",
     "        NpcKind::MagmaSnail,",
     "        NpcKind::Shimmerfly,"]
new[close_i:close_i]=ins
# pin index shifted by len(ins)
new[pin_i+len(ins)]=new[pin_i+len(ins)].replace("270,","274,")
import difflib
diff=difflib.unified_diff(lines,new,
    fromfile="a/crates/tr-app/src/npc.rs",tofile="b/crates/tr-app/src/npc.rs",lineterm="")
patch="\n".join(diff)+"\n"
open(REPO+r"\tools\_d37_c9_allkinds_fix.patch","w",newline="\n").write(patch)
print("---- patch ----")
print(patch)
