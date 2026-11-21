# W4-d35 sheets phase: frame-count witness.
# Parses Main.cs's Main.npcFrameCount initializer (Main.cs:67082-67154) and
# render/entities.rs's NPC_FRAME_COUNT, compares all 697 slots, and prints the
# 33 W4-d35 kinds' rows with both sources' values + the draw-arm literal.
import re, io, sys

ROOT = r"C:\Users\DreadLord\Desktop\TR-W4-d35"

def parse_cs():
    src = io.open(ROOT + r"\decompiled\Terraria\Main.cs", encoding="utf-8", errors="replace").read()
    i = src.index("Main.npcFrameCount = new int[697]")
    j = src.index("{", i)
    k = src.index("};", j)
    body = src[j + 1:k]
    return [int(x) for x in re.findall(r"\d+", body)]

def parse_rs():
    src = io.open(ROOT + r"\crates\tr-app\src\render\entities.rs", encoding="utf-8").read()
    i = src.index("pub(crate) const NPC_FRAME_COUNT: [u32; 697] = [")
    j = src.index("= [", i) + 2
    depth = 0
    for k in range(j, len(src)):
        if src[k] == "[": depth += 1
        elif src[k] == "]":
            depth -= 1
            if depth == 0: break
    body = src[j + 1:k]
    return [int(x) for x in re.findall(r"\d+", body)]

cs = parse_cs()
rs = parse_rs()
print("C# entries:", len(cs), "| Rust entries:", len(rs))
assert len(cs) == 697 and len(rs) == 697, "table length drift"

KINDS = {
    73: ("GoblinScout", 16), 74: ("Bird", 5), 75: ("Pixie", 4),
    77: ("ArmoredSkeleton", 15), 78: ("Mummy", 16), 79: ("DarkMummy", 16),
    80: ("LightMummy", 16), 82: ("Wraith", 4), 83: ("CursedHammer", 6),
    84: ("EnchantedSword", 6), 85: ("Mimic", 18), 86: ("Unicorn", 16),
    93: ("GiantBat", 4), 94: ("Corruptor", 3), 95: ("DiggerHead", 1),
    96: ("DiggerBody", 1), 97: ("DiggerTail", 1), 98: ("SeekerHead", 1),
    99: ("SeekerBody", 1), 100: ("SeekerTail", 1), 101: ("Clinger", 5),
    102: ("AnglerFish", 6), 103: ("GreenJellyfish", 7), 104: ("Werewolf", 16),
    109: ("Clown", 12), 110: ("SkeletonArcher", 20), 112: ("VileSpit", 1),
    116: ("TheHungryII", 6), 120: ("ChaosElemental", 15), 121: ("Slimer", 4),
    122: ("Gastropod", 11), 137: ("IlluminantBat", 4), 140: ("PossessedArmor", 15),
}

mism = [(i, cs[i], rs[i]) for i in range(697) if cs[i] != rs[i]]
print("global mismatches:", len(mism))
for i, a, b in mism[:40]:
    print(f"  id {i}: C#={a} Rust={b}")

print("\nW4-d35 kinds (id, name, C#, Rust, draw-arm literal):")
bad = 0
for i, (name, lit) in sorted(KINDS.items()):
    ok = cs[i] == rs[i] == lit
    if not ok: bad += 1
    print(f"  {i:3d} {name:16s} C#={cs[i]:2d} Rust={rs[i]:2d} arm={lit:2d} {'OK' if ok else 'MISMATCH'}")
print("\nRESULT:", "ALL 33 MATCH" if bad == 0 and not mism else f"{bad} kind mismatches, {len(mism)} global")
sys.exit(0 if bad == 0 else 1)
