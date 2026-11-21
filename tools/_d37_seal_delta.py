# W4-d37 SEAL: addendum deltas vs the CORE union (R9 sets), so the bible
# can state grand-union pins without double counting. Git-show only.
import re, subprocess, json
REPO=r"C:\Users\DreadLord\Desktop\TR-W4-d37"
def show(rev,path):
    r=subprocess.run(["git","-C",REPO,"show",f"{rev}:{path}"],capture_output=True)
    return r.stdout.decode("utf-8","replace") if r.returncode==0 else None
def sheets(rev):
    m=re.search(r"STATIC_SHEET_NAMES[^=]*=\s*&?\[(.*?)\n\];",show(rev,"crates/tr-app/src/assets.rs"),re.S)
    return set(re.findall(r'"(NPC_[^"]+)"',re.sub(r"//[^\n]*","",m.group(1))))
def slots(rev):
    m=re.search(r"BESTIARY_NPC_SHEET_IDS:\s*\[u16;\s*(\d+)\]\s*=\s*\[(.*?)\];",show(rev,"crates/tr-app/src/assets.rs"),re.S)
    return [int(x) for x in re.findall(r"\b(\d+)\b",re.sub(r"//[^\n]*","",m.group(2)))]
def cues(rev):
    m=re.search(r"SFX_CUES[^=]*=\s*&?\[(.*?)\n\];",show(rev,"crates/tr-app/src/audio.rs"),re.S)
    return set(re.findall(r'"(NPC_(?:Hit|Killed)_\d+)"',m.group(1)))
CORE={"base":"a725acb04","c3":"9ea0ee90d","c4":"0df6964be","c6":"5de680219",
 "c7":"c2cc6ec61","c8":"3705670d3","c9":"2042d77f1","c10":"5d8387955",
 "d23":"43091b01c","d120":"2408233da","d125":"86e3adfd3","d33":"b08acf463",
 "d34":"5f3435a87","d35":"e5558b0a9","d37":"a341daa12","d38":"1ba0395c7",
 "d40":"8fe1d6d31","d42":"2d7ac08c3","e26":"048cca6cf","g8":"82c93cecf"}
# core union sets (as-pushed, before dedupe/gap-close)
cs=set(); csl=set(); cc=set()
for l,rev in CORE.items():
    cs|=sheets(rev); cc|=cues(rev)
    csl|=set(slots(rev))
print("core as-pushed: sheets",len(cs),"slots(distinct)",len(csl),"cues",len(cc))
ADD={"d39":"907234441","d41":"5d036721d","g10L":"5dee28650"}
for l,rev in ADD.items():
    sh=sheets(rev)-cs; sl=[x for x in slots(rev) if x not in csl]; cu=cues(rev)-cc
    print(f"{l:5s} sheets+{len(sh)} {sorted(sh)}")
    print(f"      slots+{len(sl)} {sl}  cues+{len(cu)} {sorted(cu)}")
# d23 new-kind sheets/slots vs core (its tip is IN core already, so delta=0;
# but its 9 slot adds include base backfills + drop-port slots - classify)
bsl=set(slots("a725acb04"))
d23sl=slots("43091b01c")
adds=[x for x in d23sl if x not in bsl]
print("d23 slot adds vs base:",adds)
print("  of those, already in core union:",[x for x in adds if x in csl])
print("  genuinely new to core:",[x for x in adds if x not in csl])
