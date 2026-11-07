# -*- coding: utf-8 -*-
"""Final clean pass for chest_gen.rs."""
fp = r"C:/Users/DreadLord/Desktop/TR-chestgen/crates/tr-app/src/chest_gen.rs"
with open(fp, "rb") as f:
    s = f.read().decode("utf-8")
NL = "\r\n" if "\r\n" in s else "\n"

def sub(old, new, cnt=1):
    global s
    o = old.replace("\n", NL)
    n = new.replace("\n", NL)
    c = s.count(o)
    assert c == cnt, "expected %d, found %d of %r" % (cnt, c, o[:50])
    s = s.replace(o, n)

sub("""    let mut out: Vec<Row> = Vec::with_capacity(16);
    // Row 0: the special slot/weapon (`flag*` extras follow the main item).
    let main = match band {
        Band::Surface => None,
        Band::Mid => None,
        Band::Deep => None,
        Band::Deepest => None,
    };
    let _ = main;
    match band {""",
"""    let mut out: Vec<Row> = Vec::with_capacity(16);
    match band {""")

sub("""impl StockCtx {
    fn silver_bar() -> u16 {
        BAR_SILVER
    }

    fn iron_bar() -> u16 {
        BAR_IRON
    }

    fn gold_bar() -> u16 {
        BAR_GOLD
    }

    fn ore_pick(_pick: u64) -> u16 {
        ORE_COPPER
    }

    fn _ore_silver_alt() -> u16 {
        ORE_SILVER_ALT
    }
}

/// The unused alias keeping the vanilla GenVars names attached to the
/// helpers (`Self::` calls read through the alias).
type StockCtx = ();
""",
"""fn silver_bar() -> u16 {
    BAR_SILVER
}

fn iron_bar() -> u16 {
    BAR_IRON
}

fn gold_bar() -> u16 {
    BAR_GOLD
}

fn ore_pick() -> u16 {
    ORE_COPPER
}
""")

for old, new in [
    ("Self::silver_bar()", "silver_bar()"),
    ("Self::iron_bar()", "iron_bar()"),
    ("Self::gold_bar()", "gold_bar()"),
    ("Self::ore_pick(next(rng, 2))", "ore_pick()"),
    ("Self::ore_pick(next(rng, 1))", "ore_pick()"),
]:
    if old in s:
        s = s.replace(old, new)

sub("""    let _ = &mut 0u8;
    rows
}""", """    rows
}""")

sub("""            let hell = y as f64 > world.rock_layer && doubled_hell_depth(world, y);""",
"""            let hell = doubled_hell_depth(world, y);""")

sub("""                let ores = if pick == 0 { 265 } else { ore_pick(next(rng, 1)) };""",
"""                let ores = 265;""")

sub("""            if next(rng, 2) == 0 {
                if next(rng, 2) == 0 {
                    let ores = ore_pick();
                    out.push(Row::stacked(ores, ranged(rng, 16, 31) as u32));
                } else {
                    let ores = ore_pick();
                    out.push(Row::stacked(ores, ranged(rng, 16, 31) as u32));
                }
            }""",
"""            if next(rng, 2) == 0 {
                out.push(Row::stacked(ore_pick(), ranged(rng, 16, 31) as u32));
            }""")

sub("""const ORE_SILVER_ALT: u16 = 168;
""", "")

with open(fp, "wb") as f:
    f.write(s.encode("utf-8"))
print("cleaned")
