import subprocess, re, json
REPO = r"C:\Users\DreadLord\Desktop\TR-W4-d37"
def show(rev, path):
    return subprocess.run(["git","-C",REPO,"show",f"{rev}:{path}"],
                          capture_output=True).stdout.decode("utf8","replace")
src = show("82c93cec", "crates/tr-app/src/assets.rs")
m = re.search(r"BESTIARY_NPC_SHEET_IDS:\s*\[u16;\s*(\d+)\]\s*=\s*\[(.*?)\];", src, re.S)
ids = [int(x) for x in re.findall(r"\b(\d+)\b", re.sub(r"//[^\n]*","",m.group(2)))]
base = show("a725acb0", "crates/tr-app/src/assets.rs")
mb = re.search(r"BESTIARY_NPC_SHEET_IDS:\s*\[u16;\s*(\d+)\]\s*=\s*\[(.*?)\];", base, re.S)
bids = [int(x) for x in re.findall(r"\b(\d+)\b", re.sub(r"//[^\n]*","",mb.group(2)))]
print("g8 slot adds beyond base order:", ids[len(bids):])
maps = json.load(open(REPO+r"\tools\_d37_receipts_maps.json"))["lanes"]
plan = json.load(open(REPO+r"\tools\_d37_receipts_plan_raw.json"))
own = {k: tuple(v) for k,v in plan["owners"].items()}
cs_by_id = {v[1]: (k, v[0]) for k, v in ((k, (maps[l]["cs"].get(k), l)) for k,(l,p) in own.items()) if v[0]}
for i in ids[len(bids):]:
    k = cs_by_id.get(i)
    print(f"  slot cs {i}: {k}", "OWNED" if k and own[k[0]][0]==k[1] else "DROPPED/OTHER")
