# W4-d37 SEAL: per-lane REMAINING E8 actions vs the AMENDED canonical plan,
# measured at each lane's CURRENT origin tip. A lane "consumed" its table
# when every owned kind sits at its final id and every non-owned kind is
# absent. Emits the receipt-status table + laggard flags.
import json
REPO=r"C:\Users\DreadLord\Desktop\TR-W4-d37"
census=json.load(open(REPO+r"\tools\_d37_seal_census.json"))["lanes"]
seal=json.load(open(REPO+r"\tools\_d37_seal_final.json"))
own={k:tuple(v) for k,v in seal["owners"].items()}
final=seal["final"]
maps=json.load(open(REPO+r"\tools\_d37_receipts_maps.json"))["lanes"]
# lanes addressed by E8 (the 20 core claimants)
LANES=["c3","c4","c6","c7","c8","c9","c10","d23","d120","d125","d33","d34",
       "d35","d37","d38","d40","d42","e26","g8"]
rows=[]
for l in LANES:
    cur=census[l]["new"]                 # kind -> current proto id
    # kinds this lane CURRENTLY carries that the seal says it OWNS
    owned_here={k for k,(ol,_) in own.items() if ol==l}
    move_ok=[]; move_owed=[]; drop_owed=[]; extra=[]
    for k in owned_here:
        want=final[k]; have=cur.get(k)
        if have==want: move_ok.append(k)
        elif have is None: move_owed.append((k,"ABSENT",want))
        else: move_owed.append((k,have,want))
    for k,have in cur.items():
        if k not in owned_here:
            # lane carries a kind it does not own -> must drop
            keeper=own.get(k,(None,))[0]
            drop_owed.append((k,have,keeper))
    rows.append((l,move_ok,move_owed,drop_owed))
    print(f"== {l}: owned-at-final {len(move_ok)}/{len(owned_here)} "
          f"| moves-owed {len(move_owed)} | drops-owed {len(drop_owed)}")
    for k,a,b in move_owed: print(f"     MOVE {k}: {a} -> {b}")
    for k,a,b in drop_owed: print(f"     DROP {k}@{a} (keeper {b})")
json.dump([{"lane":l,"move_ok":mo,"move_owed":mw,"drop_owed":dr}
           for l,mo,mw,dr in rows],
          open(REPO+r"\tools\_d37_seal_receipts.json","w"),indent=1)
