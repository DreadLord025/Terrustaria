# W4-d37 E8-SEAL: enumerate every duplicate kind-name across claimants and
# every same-proto-different-kind collision at CURRENT tips. Pure analysis.
import json, collections

REPO = r"C:\Users\DreadLord\Desktop\TR-W4-d37"
c = json.load(open(REPO + r"\tools\_d37_e8s_census.json"))["lanes"]
# claimant lanes only (exclude base + local probes duplicating origin)
LANES = ["c3","c4","c6","c7","c8","c9","c10","d9","d11","d23","d120","d125",
         "d33","d34","d35","d36","d37","d38","d39","d40","d41","d42","d43",
         "e26","f18","f42","f47","g5","g8","g10","c7b","d103","e21","g62"]
# d9/d11 are proj lanes that ALSO port NPC kinds (CultistDragon fam / BloodSquid)
by_kind = collections.defaultdict(list)   # kind -> [(lane, proto)]
for l in LANES:
    for k, v in c[l]["new"].items():
        by_kind[k].append((l, v))

print("=== DUPLICATE kind-names (ported by 2+ lanes) ===")
dups = {k: v for k, v in by_kind.items() if len(v) > 1}
for k in sorted(dups):
    print(f"  {k:28s} " + "  ".join(f"{l}@{p}" for l, p in sorted(dups[k], key=lambda x: x[1])))
print(f"total duplicate kind-names: {len(dups)}, extra ports: {sum(len(v)-1 for v in dups.values())}")

print("\n=== SAME-PROTO DIFFERENT-KIND collisions (wire bugs) ===")
by_proto = collections.defaultdict(list)
for l in LANES:
    for k, v in c[l]["new"].items():
        by_proto[v].append((l, k))
hard = 0
for p in sorted(by_proto):
    entries = by_proto[p]
    kinds = set(k for _, k in entries)
    if len(kinds) > 1:
        hard += 1
        print(f"  proto {p}: " + "  ".join(f"{l}:{k}" for l, k in sorted(entries)))
print(f"total contested protos (different-kind): {hard}")

print("\n=== distinct new kind-names total ===")
print(len(by_kind))
