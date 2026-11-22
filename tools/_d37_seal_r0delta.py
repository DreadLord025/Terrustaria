# W4-d37 SEAL: per-lane static-sheet/slot/cue deltas R0-tip -> current-tip,
# to explain the 455 -> 463 sheet-union growth and re-pin the grand union.
import re, subprocess
REPO=r"C:\Users\DreadLord\Desktop\TR-W4-d37"
def show(rev,path):
    return subprocess.run(["git","-C",REPO,"show",f"{rev}:{path}"],
                          capture_output=True).stdout.decode("utf8","replace")
def sheets(rev):
    m=re.search(r"STATIC_SHEET_NAMES[^=]*=\s*&?\[(.*?)\n\];",
                show(rev,"crates/tr-app/src/assets.rs"),re.S)
    return set(re.findall(r'"(NPC_[^"]+)"',re.sub(r"//[^\n]*","",m.group(1))))
def slots(rev):
    m=re.search(r"BESTIARY_NPC_SHEET_IDS:\s*\[u16;\s*(\d+)\]\s*=\s*\[(.*?)\];",
                show(rev,"crates/tr-app/src/assets.rs"),re.S)
    return [int(x) for x in re.findall(r"\b(\d+)\b",re.sub(r"//[^\n]*","",m.group(2)))]
R0={"c3":"9299046f7","c4":"17a241f6a","c10":"9641e20db","d23":"19109424",
    "d120":"c5e68a80","d35":"6a2a155a","d38":"6f9dd02d","d42":"9b22c091",
    "d125":"cd2b1b1e","d37":"bdd59afb"}
CUR={"c3":"9ea0ee90d","c4":"0df6964be","c10":"5d8387955","d23":"43091b01c",
     "d120":"2408233da","d35":"e5558b0a9","d38":"1ba0395c7","d42":"2d7ac08c3",
     "d125":"86e3adfd3","d37":"a341daa12"}
for l in R0:
    a,b=sheets(R0[l]),sheets(CUR[l])
    sa,sb=slots(R0[l]),slots(CUR[l])
    if a!=b or sa!=sb:
        print(f"{l}: sheets +{sorted(b-a)} -{sorted(a-b)} | slots {len(sa)}->{len(sb)} adds {sb[len(sa):] if sb[:len(sa)]==sa else 'REORDERED'}")
    else:
        print(f"{l}: unchanged")
