# W4-d37 SEAL: exact sheet-name delta R0-union(455) -> current-core(463),
# slot union accounting at current tips, and the c10/d23 duplicate-kind
# sheet/slot classification for the merger bible.
import re, subprocess, json
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
R0={"base":"a725acb0","c3":"9299046f7","c4":"17a241f6a","c6":"5de68021",
 "c7":"c2cc6ec6","c8":"3705670d","c9":"2042d77f","c10":"9641e20db",
 "d23":"19109424","d120":"c5e68a80","d125":"cd2b1b1e","d33":"b08acf463",
 "d34":"5f3435a8","d35":"6a2a155a","d37":"bdd59afb","d38":"6f9dd02d",
 "d40":"8fe1d6d3","d42":"9b22c091","e26":"048cca6c","g8":"82c93cec"}
CUR=dict(R0); CUR.update({"c3":"9ea0ee90d","c4":"0df6964be","c10":"5d8387955",
 "d23":"43091b01c","d120":"2408233da","d35":"e5558b0a9","d38":"1ba0395c7",
 "d42":"2d7ac08c3","d125":"86e3adfd3","d37":"a341daa12","base":"a725acb04"})
u0=set(); u1=set()
for l,r in R0.items(): u0|=sheets(r)
for l,r in CUR.items(): u1|=sheets(r)
print("R0 sheet union:",len(u0)," current-core:",len(u1))
print("new names:",sorted(u1-u0))
for n in sorted(u1-u0):
    print(" ",n,"on:",[l for l,r in CUR.items() if n in sheets(r)])
# grand union incl addendum lanes
for l,r in {"d39":"907234441","d41":"5d036721d","g10":"5dee28650","d36":"bcf4ac551","d43":"a725acb04"}.items():
    u1|=sheets(r)
print("grand sheet union (all current + local g10):",len(u1))
# slot union at current core tips (distinct), and naive append
bsl=slots("a725acb04")
naive=len(bsl); distinct=set(bsl)
for l,r in CUR.items():
    if l=="base": continue
    s=slots(r)
    assert s[:len(bsl)]==bsl,(l,"reordered base")
    naive+=len(s)-len(bsl); distinct|=set(s[len(bsl):])
print("current-core slots: naive append",naive,"distinct",len(distinct))
