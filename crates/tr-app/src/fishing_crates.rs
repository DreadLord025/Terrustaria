//! W4-f35 — the fishing-crate OPEN trees: `Player.OpenFishingCrate`
//! (`Player.cs:8028-9250`), the crate grant switch the inventory
//! right-click reaches through `ItemSlot.TryOpenContainer_GrantItems`
//! (`UI/ItemSlot.cs:1812-1815`, the `ItemID.Sets.IsFishingCrate` gate).
//!
//! The FISHING-CATCH side (the bobber's crate roll,
//! `Projectile.cs:21052`'s `hardMode ? 3979 : 2334` family) is W4-f36's
//! lane — this module is the OTHER half: what opening a caught crate
//! grants. The three base crates (Wooden 2334/3979, Iron 2335/3980,
//! Golden 2336/3981) `return` early (`:8261/:8440/:8592`); every other
//! crate rides the shared while-loop (`:8595-8914`, the `maxValue = 1`
//! biome-weapon gate always passes on the first pass) + the shared
//! tails (`:8915-9249`).
//!
//! ## The RNG contract
//!
//! Same as the boss bags (`boss_bags.rs`): every draw rides `Main.rand`
//! (the open is a UI click outside every `SwapRandom` block — the
//! wiring feeds `NamedRngs::global`), and every `QuickSpawnItem`
//! (`Player.cs:7170-7180`) interleaves its `Prefix(-1)` natural-roll
//! draws between the crate's chance rolls. The shared
//! [`crate::boss_bags::quick_spawn`] keeps that order; the output
//! [`crate::boss_bags::BagGrant`] list is the `GetOrDropItem` fill order.
//!
//! ## The recorded quirks (kept verbatim)
//!
//! - The Wooden bar pick draws `Next(4)` but its switch maps SIX cases
//!   (`Player.cs:8111-8126`) — cases 2/3 (11/700) are dead, only
//!   12/699 ever land before the hardmode re-pick.
//! - The Wooden bar pick's hardmode re-pick draws `Next(2)` INSIDE the
//!   `(Next(2) == 0) & flag` gate — a miss consumes NO pick draw
//!   (`:8127-8139`), unlike the iron/golden gates.
//! - The Golden crate's second while-body half `continue`s on
//!   `Next(3) != 0` (`:8510-8512`) — SKIPPING the in-loop potion roll
//!   (`:8203`'s wooden shape has no such continue).
//! - The shared-loop ore arm re-types to 174/175 for the lava crates
//!   AFTER the pick draws (`:8823-8826`, `:8882-8885`) — the pick draws
//!   are still consumed.
//! - The Hallowed crate 3207/3986 has NO biome-weapon arm in the
//!   while-loop (the 3203/3204/3205/3206/3208 ids each have one) — it
//!   rides only the shared coins/ores/bars/potions + the 3986-only
//!   hardmode souls tail (`:9221-9249`).
//! - The dead `int num14/num16 = Main.rand.Next(5)` draws inside the
//!   jungle/lava 1/20 arms (`:8668`, `:8734`) are consumed and DISCARDED
//!   (the grant is the fixed 3017/906).
//! - The Ocean weapon arm draws `Next(4)` BEFORE the 1/10 Water Bolt
//!   gate (`:8601-8602`) — the pick draw happens even when 863 lands.
//! - `WorldGen.Skyblock.lowTiles` (`:9153`) is the gen-time active-tile
//!   census (`WorldGen.cs:3209-3217`) — the port carries no such scan;
//!   [`CrateEnv::skyblock_low_tiles`] is the seam (wiring passes false,
//!   the e25-style constants-off note).
//!
//! Census: `docs/research/65-expert-drop-trees-census.md`.

#![allow(dead_code)] // the clicks.rs dispatch lands with the wiring commit

use crate::boss_bags::{BagGrant, quick_spawn, range};
use crate::prefix::PrefixRandom;

/// The world scalars the crate trees read.
#[derive(Clone, Copy, Debug, Default)]
pub struct CrateEnv {
    /// `Main.remixWorld` (`Player.cs:8688`) — the Frozen crate's Ice
    /// Blade 1319 → Ice Bow 725 swap.
    pub remix: bool,
    /// `WorldGen.Skyblock.lowTiles` (`:9153`) — the Sky crate's
    /// Sky Mill 2197 tail. The gen-time tile-census seam (the module
    /// header); the wiring passes false.
    pub skyblock_low_tiles: bool,
}

/// `ItemID.Sets.IsFishingCrate` (`ItemID.cs:1309`): the 26 crate ids.
pub fn is_fishing_crate(kind: u16) -> bool {
    matches!(
        kind,
        2334 | 2335
            | 2336
            | 3203
            | 3204
            | 3205
            | 3206
            | 3207
            | 3208
            | 4405
            | 4407
            | 4877
            | 5002
            | 3979
            | 3980
            | 3981
            | 3982
            | 3983
            | 3984
            | 3985
            | 3986
            | 3987
            | 4406
            | 4408
            | 4878
            | 5003
    )
}

/// `ItemID.Sets.IsFishingCrateHardmode` (`ItemID.cs:1310`): the 13
/// hardmode variants — the `flag` of `OpenFishingCrate` (`:8031`).
pub fn is_hardmode_crate(kind: u16) -> bool {
    matches!(
        kind,
        3979 | 3980 | 3981 | 3982 | 3983 | 3984 | 3985 | 3986 | 3987 | 4406 | 4408 | 4878 | 5003
    )
}

/// `Player.OpenFishingCrate(int crateItemID)` (`Player.cs:8028-9250`).
/// The three base crates return from their own arms; the rest run the
/// shared while-loop + tails. Grants push in `QuickSpawnItem` order.
pub fn open_fishing_crate<R: PrefixRandom + ?Sized>(
    crate_kind: u16,
    env: &CrateEnv,
    rng: &mut R,
    out: &mut Vec<BagGrant>,
) {
    let flag = is_hardmode_crate(crate_kind); // `:8031`
    match crate_kind {
        2334 | 3979 => wooden_crate(flag, rng, out),
        2335 | 3980 => iron_crate(flag, rng, out),
        2336 | 3981 => golden_crate(flag, rng, out),
        _ => {
            shared_crate_loop(crate_kind, flag, env, rng, out);
            shared_crate_tails(crate_kind, flag, env, rng, out);
        }
    }
}

/// The Wooden crate (`Player.cs:8034-8262`): the retry-until-granted
/// while-loop (one accessory/coin/ore/potion chain per pass), then the
/// 1/3 healing-potion + 1/3 bait tails.
fn wooden_crate<R: PrefixRandom + ?Sized>(flag: bool, rng: &mut R, out: &mut Vec<BagGrant>) {
    let mut flag3 = true; // `:8037`
    while flag3 {
        // `:8040-8044` — hardmode-only 1/200 Enchanted Sundial.
        if flag && flag3 && rng.next_max(200) == 0 {
            quick_spawn(rng, out, 3064, 1);
            flag3 = false;
        }
        // `:8045-8054` — the two 1/40 accessories.
        if flag3 && rng.next_max(40) == 0 {
            quick_spawn(rng, out, 3200, 1); // Sailfish Boots
            flag3 = false;
        }
        if flag3 && rng.next_max(40) == 0 {
            quick_spawn(rng, out, 3201, 1); // Tsunami in a Bottle
            flag3 = false;
        }
        // `:8055-8059` — hardmode-only 1/25 Anchor.
        if flag && flag3 && rng.next_max(25) == 0 {
            quick_spawn(rng, out, 2424, 1);
            flag3 = false;
        }
        // `:8060-8086` — the 1/20 six-way accessory pick (no flag3 gate).
        if rng.next_max(20) == 0 {
            let item = [285, 953, 4341, 3068, 3084, 6165][rng.next_max(6) as usize];
            quick_spawn(rng, out, item, 1);
            flag3 = false;
        }
        // `:8087-8091` — pre-hardmode-only 1/50 Extractinator.
        if !flag && flag3 && rng.next_max(50) == 0 {
            quick_spawn(rng, out, 997, 1);
            flag3 = false;
        }
        // `:8092-8108` — the 1/7 coin arm: 1/3 gold 1-5, else silver 20-90.
        if rng.next_max(7) == 0 {
            let (item, stack) = if rng.next_max(3) == 0 {
                (73, range(rng, 1, 6))
            } else {
                (72, range(rng, 20, 91))
            };
            quick_spawn(rng, out, item, stack);
            flag3 = false;
        }
        // `:8109-8202` — the 1/7 bars, else the 1/8 ores (the else-if).
        if rng.next_max(7) == 0 {
            // `:8111-8126` — Next(4) but SIX switch cases: 11/700 are
            // dead (the module header's quirk).
            let mut num5 = [12, 699, 11, 700][rng.next_max(4) as usize];
            // `:8127-8139` — the gate draws ONE Next(2); the pick
            // Next(2) only inside a hit.
            if rng.next_max(2) == 0 && flag {
                num5 = [364, 1104][rng.next_max(2) as usize];
            }
            quick_spawn(rng, out, num5, range(rng, 4, 16));
            flag3 = false;
        } else if rng.next_max(8) == 0 {
            // `:8145-8172` — Next(4) over the eight-case switch: only
            // 20/703/22/704 land pre-re-pick.
            let mut num6 = [20, 703, 22, 704, 21, 705, 19, 706][rng.next_max(4) as usize];
            let mut stack2 = range(rng, 2, 6);
            if rng.next_max(2) == 0 && flag {
                // `:8176-8197` — Next(2) over SIX cases: only 381/1184 land.
                num6 = [381, 1184, 382, 1191, 391, 1198][rng.next_max(2) as usize];
                stack2 = range(rng, 2, 4);
            }
            quick_spawn(rng, out, num6, stack2);
            flag3 = false;
        }
        // `:8203-8241` — the 1/7 potion arm (no else — runs every pass).
        if rng.next_max(7) == 0 {
            let num7 = [288, 290, 292, 299, 298, 304, 291, 2322, 2323, 2329]
                [rng.next_max(10) as usize];
            quick_spawn(rng, out, num7, range(rng, 1, 4));
            flag3 = false;
        }
    }
    // `:8243-8256` — the 1/3 healing-potion tail: 28/110 pick, 5-15.
    if rng.next_max(3) == 0 {
        let num8 = [28, 110][rng.next_max(2) as usize];
        quick_spawn(rng, out, num8, range(rng, 5, 16));
    }
    // `:8257-8260` — the 1/3 bait tail: 2/3 Apprentice 2674, else
    // Journeyman 2675, 1-4.
    if rng.next_max(3) == 0 {
        let bait = if rng.next_max(3) != 0 { 2674 } else { 2675 };
        quick_spawn(rng, out, bait, range(rng, 1, 5));
    }
}

/// The Iron crate (`Player.cs:8263-8441`).
fn iron_crate<R: PrefixRandom + ?Sized>(flag: bool, rng: &mut R, out: &mut Vec<BagGrant>) {
    let mut flag4 = true; // `:8266`
    while flag4 {
        // `:8269-8273` — hardmode-only 1/60 Enchanted Sundial.
        if flag && flag4 && rng.next_max(60) == 0 {
            quick_spawn(rng, out, 3064, 1);
            flag4 = false;
        }
        // `:8274-8298` — the four accessory gates (1/25, 1/20, 1/15, 1/20, 1/20).
        if flag4 && rng.next_max(25) == 0 {
            quick_spawn(rng, out, 2501, 1); // Ginger Beard
            flag4 = false;
        }
        if flag4 && rng.next_max(20) == 0 {
            quick_spawn(rng, out, 2587, 1); // Tartar Sauce
            flag4 = false;
        }
        if flag4 && rng.next_max(15) == 0 {
            quick_spawn(rng, out, 2608, 1); // Falcon Blade
            flag4 = false;
        }
        if flag4 && rng.next_max(20) == 0 {
            quick_spawn(rng, out, 3200, 1); // Sailfish Boots
            flag4 = false;
        }
        if flag4 && rng.next_max(20) == 0 {
            quick_spawn(rng, out, 3201, 1); // Tsunami in a Bottle
            flag4 = false;
        }
        // `:8299-8303` — the 1/4 gold 5-10.
        if rng.next_max(4) == 0 {
            quick_spawn(rng, out, 73, range(rng, 5, 11));
            flag4 = false;
        }
        // `:8304-8397` — the 1/6 bars, else the 1/4 ores.
        if rng.next_max(6) == 0 {
            let mut num9 = [12, 699, 11, 700, 14, 701][rng.next_max(6) as usize];
            if rng.next_max(2) == 0 && flag {
                num9 = [364, 1104, 365, 1105][rng.next_max(4) as usize];
            }
            quick_spawn(rng, out, num9, range(rng, 12, 22));
            flag4 = false;
        } else if rng.next_max(4) == 0 {
            let mut num10 = [20, 703, 22, 704, 21, 705][rng.next_max(6) as usize];
            let mut num11 = range(rng, 4, 8);
            // `:8375` — the 2/3 hardmode re-pick gate.
            if rng.next_max(3) != 0 && flag {
                num10 = [381, 1184, 382, 1191][rng.next_max(4) as usize];
                num11 -= rng.next_max(2); // `:8393`
            }
            quick_spawn(rng, out, num10, num11);
            flag4 = false;
        }
        // `:8398-8430` — the 1/4 potion arm.
        if rng.next_max(4) == 0 {
            let num12 = [288, 296, 304, 305, 2322, 2323, 2324, 2327][rng.next_max(8) as usize];
            quick_spawn(rng, out, num12, range(rng, 2, 5));
            flag4 = false;
        }
    }
    // `:8432-8435` — the 1/2 Lesser Healing/Mana tail: Next(188, 190), 5-15.
    if rng.next_max(2) == 0 {
        quick_spawn(rng, out, range(rng, 188, 190), range(rng, 5, 16));
    }
    // `:8436-8439` — the 1/2 bait tail: 2/3 Journeyman 2675, else Master 2676, 2-4.
    if rng.next_max(2) == 0 {
        let bait = if rng.next_max(3) != 0 { 2675 } else { 2676 };
        quick_spawn(rng, out, bait, range(rng, 2, 5));
    }
}

/// The Golden crate (`Player.cs:8442-8593`).
fn golden_crate<R: PrefixRandom + ?Sized>(flag: bool, rng: &mut R, out: &mut Vec<BagGrant>) {
    let mut flag2 = true; // `:8445`
    while flag2 {
        // `:8448-8452` — hardmode-only 1/20 Enchanted Sundial.
        if flag && flag2 && rng.next_max(20) == 0 {
            quick_spawn(rng, out, 3064, 1);
            flag2 = false;
        }
        // `:8453-8462` — the 1/8 Life Crystal + 1/10 Hardy Saddle.
        if flag2 && rng.next_max(8) == 0 {
            quick_spawn(rng, out, 29, 1);
            flag2 = false;
        }
        if flag2 && rng.next_max(10) == 0 {
            quick_spawn(rng, out, 2491, 1);
            flag2 = false;
        }
        // `:8463-8467` — the 1/3 gold 8-20.
        if rng.next_max(3) == 0 {
            quick_spawn(rng, out, 73, range(rng, 8, 21));
            flag2 = false;
        }
        // `:8468-8551` — the 1/5 bars, ELSE the 1/3-gated ores with the
        // `continue` that skips nothing here (the loop body ends).
        if rng.next_max(5) == 0 {
            let mut num = [14, 701, 13, 702][rng.next_max(4) as usize];
            if rng.next_max(2) == 0 && flag {
                num = [365, 1105, 366, 1106][rng.next_max(4) as usize];
            }
            quick_spawn(rng, out, num, range(rng, 25, 35));
            flag2 = false;
        } else {
            // `:8510-8512` — the 2/3 miss CONTINUES (the next pass).
            if rng.next_max(3) != 0 {
                continue;
            }
            let mut num2 = [21, 19, 705, 706][rng.next_max(4) as usize];
            if rng.next_max(3) != 0 && flag {
                num2 = [382, 391, 1191, 1198][rng.next_max(4) as usize];
            }
            quick_spawn(rng, out, num2, range(rng, 8, 12));
            flag2 = false;
        }
    }
    // `:8553-8575` — the 1/3 potion tail: 5-way pick, 2-5.
    if rng.next_max(3) == 0 {
        let num3 = [288, 296, 305, 2322, 2323][rng.next_max(5) as usize];
        quick_spawn(rng, out, num3, range(rng, 2, 6));
    }
    // `:8576-8579` — the 1/2 Lesser Healing/Mana tail: 5-20.
    if rng.next_max(2) == 0 {
        quick_spawn(rng, out, range(rng, 188, 190), range(rng, 5, 21));
    }
    // `:8580-8583` — the 2/3 Master Bait tail: 3-7.
    if rng.next_max(3) != 0 {
        quick_spawn(rng, out, 2676, range(rng, 3, 8));
    }
    // `:8584-8591` — the two SEPARATE Enchanted Sword gates:
    // pre-hardmode 1/30, hardmode 1/15 (the draw always happens).
    if rng.next_max(30) == 0 && !flag {
        quick_spawn(rng, out, 989, 1);
    }
    if rng.next_max(15) == 0 && flag {
        quick_spawn(rng, out, 989, 1);
    }
}

/// The shared crate while-loop (`Player.cs:8595-8914`): the biome
/// weapon arms (`maxValue = 1` — the first-pass gate ALWAYS passes),
/// then the shared coins/bars/ores every pass. One pass always grants
/// (the coin/bar/ore chains each clear `flag5`), but the loop shape is
/// kept verbatim — the arms' gate order IS the draw order.
fn shared_crate_loop<R: PrefixRandom + ?Sized>(
    crate_kind: u16,
    flag: bool,
    env: &CrateEnv,
    rng: &mut R,
    out: &mut Vec<BagGrant>,
) {
    let max_value = 1; // `:8595`
    let lava = crate_kind == 4877 || crate_kind == 4878;
    let mut flag5 = true; // `:8596`
    while flag5 {
        // `:8599-8610` — the Ocean weapon: Next(4) BEFORE the 1/10
        // Water Bolt gate (the pick draw always lands).
        if (crate_kind == 5002 || crate_kind == 5003) && flag5 && rng.next_max(max_value) == 0 {
            let num13 = rng.next_max(4);
            let item = if rng.next_max(10) == 0 {
                863 // Water Bolt
            } else {
                [186, 4404, 277, 187][num13 as usize]
            };
            quick_spawn(rng, out, item, 1);
            flag5 = false;
        }
        // `:8611-8622` — the Corrupt weapon pick.
        if (crate_kind == 3203 || crate_kind == 3982) && flag5 && rng.next_max(max_value) == 0 {
            let item = [162, 111, 96, 115, 64][rng.next_max(5) as usize];
            quick_spawn(rng, out, item, 1);
            flag5 = false;
        }
        // `:8623-8634` — the Crimson weapon pick.
        if (crate_kind == 3204 || crate_kind == 3983) && flag5 && rng.next_max(max_value) == 0 {
            let item = [800, 802, 1256, 1290, 3062][rng.next_max(5) as usize];
            quick_spawn(rng, out, item, 1);
            flag5 = false;
        }
        // `:8635-8652` — the Dungeon crate: the Lock Box + the two
        // optional tails (Golden Key 5-15, bone-type bars 25-50).
        if (crate_kind == 3205 || crate_kind == 3984) && flag5 && rng.next_max(max_value) == 0 {
            quick_spawn(rng, out, 3085, 1);
            flag5 = false;
            if rng.next_max(2) == 0 {
                quick_spawn(rng, out, 149, range(rng, 5, 16));
            }
            if rng.next_max(2) == 0 {
                let item = [134, 137, 139][rng.next_max(3) as usize];
                quick_spawn(rng, out, item, range(rng, 25, 51));
            }
        }
        // `:8653-8663` — the Sky weapon pick.
        if (crate_kind == 3206 || crate_kind == 3985) && flag5 && rng.next_max(max_value) == 0 {
            let item = [158, 65, 159, 2219][rng.next_max(4) as usize];
            quick_spawn(rng, out, item, 1);
            flag5 = false;
        }
        // `:8664-8684` — the Jungle weapon: the 1/20 Leaf Blower arm
        // consumes + DISCARDS a Next(5) (`:8668`, the dead num14).
        if (crate_kind == 3208 || crate_kind == 3987) && flag5 && rng.next_max(max_value) == 0 {
            if rng.next_max(20) == 0 {
                let _num14 = rng.next_max(5); // `:8668` — drawn, discarded
                quick_spawn(rng, out, 3017, 1);
                flag5 = false;
            } else {
                let item = [212, 964, 211, 213, 2292][rng.next_max(5) as usize];
                quick_spawn(rng, out, item, 1);
                flag5 = false;
            }
        }
        // `:8685-8694` — the Frozen weapon: SelectRandom over 7, the
        // remix Ice Blade → Ice Bow swap AFTER the pick.
        if (crate_kind == 4405 || crate_kind == 4406) && flag5 && rng.next_max(max_value) == 0 {
            let mut num15 = [670, 724, 950, 1319, 987, 1579, 6153][rng.next_max(7) as usize];
            if env.remix && num15 == 1319 {
                num15 = 725; // `:8688-8691`
            }
            quick_spawn(rng, out, num15, 1);
            flag5 = false;
        }
        // `:8695-8727` — the Oasis arm block (NOT flag5-gated as a
        // whole — the three tails run every pass).
        if crate_kind == 4407 || crate_kind == 4408 {
            if flag5 && rng.next_max(max_value) == 0 {
                let item =
                    [4056, 4442, 4055, 4061, 4062, 4276, 4262, 4263][rng.next_max(8) as usize];
                quick_spawn(rng, out, item, 1);
                flag5 = false;
            }
            if rng.next_max(4) == 0 {
                quick_spawn(rng, out, 4423, range(rng, 4, 7)); // Magic Conch
                flag5 = false;
            }
            if rng.next_max(2) == 0 {
                quick_spawn(rng, out, 3380, range(rng, 10, 17)); // Sturdy Fossil
                flag5 = false;
            }
            if rng.next_max(35) == 0 {
                quick_spawn(rng, out, 857, 1); // Sandstorm in a Bottle
                flag5 = false;
            }
        }
        // `:8728-8787` — the Lava arm block: the weapon (the 1/20
        // Lava Charm arm discards a Next(5)), the ALWAYS-granted
        // Obsidian Lock Box, then the two tails.
        if lava {
            if flag5 && rng.next_max(max_value) == 0 {
                if rng.next_max(20) == 0 {
                    let _num16 = rng.next_max(5); // `:8734` — drawn, discarded
                    quick_spawn(rng, out, 906, 1); // Lava Charm
                    flag5 = false;
                } else {
                    let item = [4822, 4828, 4880, 4881, 4868][rng.next_max(5) as usize];
                    quick_spawn(rng, out, item, 1);
                    flag5 = false;
                }
                // `:8750-8754` — INSIDE the weapon gate: 1/4 ×2.
                if rng.next_max(4) == 0 {
                    quick_spawn(rng, out, 4858, 2);
                    flag5 = false;
                }
            }
            quick_spawn(rng, out, 4879, 1); // `:8756` — no gate
            flag5 = false;
            if rng.next_max(3) == 0 {
                quick_spawn(rng, out, 4824, range(rng, 7, 11)); // `:8760`
                flag5 = false;
            }
            if rng.next_max(2) == 0 {
                let num17 = [4902, 4903, 4904, 4905, 4906][rng.next_max(5) as usize];
                quick_spawn(rng, out, num17, 1);
                flag5 = false;
            }
        }
        // `:8788-8792` — the shared 1/4 gold 5-12.
        if rng.next_max(4) == 0 {
            quick_spawn(rng, out, 73, range(rng, 5, 13));
            flag5 = false;
        }
        // `:8793-8854` — the shared 1/7 bars (8-way pick; the lava
        // crates re-type to Hellstone Bar 174 AFTER the pick).
        if rng.next_max(7) == 0 {
            let mut num18 = [12, 699, 11, 700, 14, 701, 13, 702][rng.next_max(8) as usize];
            if lava {
                num18 = 174; // `:8823-8826`
            }
            if rng.next_max(2) == 0 && flag {
                num18 = [364, 1104, 365, 1105, 366, 1106][rng.next_max(6) as usize];
            }
            quick_spawn(rng, out, num18, range(rng, 20, 36));
            flag5 = false;
        }
        // `:8855-8858` — the 3/4 miss CONTINUES (skips the ore arm).
        if rng.next_max(4) != 0 {
            continue;
        }
        // `:8859-8913` — the shared ores: 6-way pick + 6-16 stack; the
        // lava re-type to Hellstone 175 lands AFTER the stack draw; the
        // 2/3 hardmode re-pick subtracts Next(2) from the stack.
        let mut num19 = [22, 21, 19, 704, 705, 706][rng.next_max(6) as usize];
        let mut num20 = range(rng, 6, 17);
        if lava {
            num19 = 175; // `:8882-8885`
        }
        if rng.next_max(3) != 0 && flag {
            num19 = [381, 382, 391, 1184, 1191, 1198][rng.next_max(6) as usize];
            num20 -= rng.next_max(2); // `:8910`
        }
        quick_spawn(rng, out, num19, num20);
        flag5 = false;
    }
}

/// The shared crate tails (`Player.cs:8915-9249`): the potion/bait
/// rolls, then the per-biome tails in the C#'s if order, then the
/// hardmode corrupt/crimson/hallowed souls tail.
fn shared_crate_tails<R: PrefixRandom + ?Sized>(
    crate_kind: u16,
    flag: bool,
    env: &CrateEnv,
    rng: &mut R,
    out: &mut Vec<BagGrant>,
) {
    // `:8915-8941` — the 1/4 potion tail: 6-way pick, 2-4.
    if rng.next_max(4) == 0 {
        let num21 = [288, 296, 304, 305, 2322, 2323][rng.next_max(6) as usize];
        quick_spawn(rng, out, num21, range(rng, 2, 5));
    }
    // `:8942-8945` — the 1/2 Lesser Healing/Mana tail: 5-17.
    if rng.next_max(2) == 0 {
        quick_spawn(rng, out, range(rng, 188, 190), range(rng, 5, 18));
    }
    // `:8946-8949` — the 1/2 bait tail: 1/2 Journeyman/Master, 2-6.
    if rng.next_max(2) == 0 {
        let bait = if rng.next_max(2) != 0 { 2675 } else { 2676 };
        quick_spawn(rng, out, bait, range(rng, 2, 7));
    }
    // `:8950-8968` — the Ocean tails.
    if crate_kind == 5002 || crate_kind == 5003 {
        if rng.next_max(5) == 0 {
            quick_spawn(rng, out, 5527, 1);
        }
        if rng.next_max(3) == 0 {
            quick_spawn(rng, out, 4090, range(rng, 20, 51));
        }
        if rng.next_max(10) == 0 {
            quick_spawn(rng, out, 4460, 1);
        }
        if rng.next_max(10) == 0 {
            quick_spawn(rng, out, 4425, 1);
        }
    }
    // `:8969-9051` — the Dungeon 1/2 painting tail: the 25-way pick.
    if (crate_kind == 3205 || crate_kind == 3984) && rng.next_max(2) == 0 {
        let num22 = [
            1438, 1375, 1573, 1420, 1435, 1434, 1421, 5234, 1374, 1425, 1372, 1441, 1373, 1433,
            1436, 1426, 1424, 1419, 2995, 1422, 1439, 1502, 1423, 1437, 1500,
        ][rng.next_max(25) as usize];
        quick_spawn(rng, out, num22, 1);
    }
    // `:9052-9062` — the Jungle tails.
    if crate_kind == 3208 || crate_kind == 3987 {
        if rng.next_max(3) == 0 {
            quick_spawn(rng, out, 4564, range(rng, 20, 51));
        }
        if rng.next_max(20) == 0 {
            quick_spawn(rng, out, 753, 1);
        }
    }
    // `:9063-9066` — the Frozen 1/20 Snowman pet tail.
    if (crate_kind == 4405 || crate_kind == 4406) && rng.next_max(20) == 0 {
        quick_spawn(rng, out, 669, 1);
    }
    // `:9067-9116` — the Oasis 1/2 desert-fossil tail: the 14-way pick.
    if (crate_kind == 4407 || crate_kind == 4408) && rng.next_max(2) == 0 {
        let num23 = [
            4639, 4627, 4628, 4632, 4630, 4638, 4629, 4633, 4634, 4635, 4636, 4637, 4631, 4626,
        ][rng.next_max(14) as usize];
        quick_spawn(rng, out, num23, 1);
    }
    // `:9117-9157` — the Sky tails (the lowTiles gate reads the seam).
    if crate_kind == 3206 || crate_kind == 3985 {
        if rng.next_max(2) == 0 {
            let num24 = [5226, 5254, 5238, 5258, 5255, 5388][rng.next_max(6) as usize];
            quick_spawn(rng, out, num24, 1);
        }
        if rng.next_max(2) == 0 {
            quick_spawn(rng, out, 751, range(rng, 50, 101)); // Cloud
        }
        if rng.next_max(40) == 0 {
            quick_spawn(rng, out, 4978, 1);
        }
        if env.skyblock_low_tiles && rng.next_max(3) == 0 {
            quick_spawn(rng, out, 2197, 1); // Sky Mill (`:9153-9156`)
        }
    }
    // `:9158-9220` — the Lava tails: the 1/2 12-way furniture pick +
    // the four 1/20 gates.
    if crate_kind == 4877 || crate_kind == 4878 {
        if rng.next_max(2) == 0 {
            let num25 = [
                1497, 1475, 1479, 1542, 1476, 1538, 1501, 1478, 1539, 1540, 1499, 1541,
            ][rng.next_max(12) as usize];
            quick_spawn(rng, out, num25, 1);
        }
        if rng.next_max(20) == 0 {
            quick_spawn(rng, out, 221, 1);
        }
        if rng.next_max(20) == 0 {
            quick_spawn(rng, out, 4443, 1);
        }
        if rng.next_max(20) == 0 {
            quick_spawn(rng, out, 4737, 1);
        }
        if rng.next_max(20) == 0 {
            quick_spawn(rng, out, 4551, 1);
        }
    }
    // `:9221-9224` — the hardmode corrupt/crimson/hallowed gate.
    if !flag || (crate_kind != 3982 && crate_kind != 3986 && crate_kind != 3983) {
        return;
    }
    // `:9225-9233` — the 1/2 souls of night/light: 521, hallowed 520.
    if rng.next_max(2) == 0 {
        let item2 = if crate_kind == 3986 { 520 } else { 521 };
        quick_spawn(rng, out, item2, range(rng, 2, 6));
    }
    // `:9234-9249` — the 1/2 second soul row: 522 / crimson 1332 /
    // hallowed 502 (the stack RE-DRAWN 4-10, the first draw discarded).
    if rng.next_max(2) == 0 {
        let mut item3 = 522;
        let mut stack3 = range(rng, 2, 6);
        match crate_kind {
            3983 => item3 = 1332,
            3986 => {
                item3 = 502;
                stack3 = range(rng, 4, 11); // `:9245` — overwrites the first draw
            }
            _ => {}
        }
        quick_spawn(rng, out, item3, stack3);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::VecDeque;
    use tr_core::UnifiedRandom;

    /// The `boss_bags.rs` ScriptRng shape: forced draws pop off the
    /// script (asserted inside range); an exhausted script returns the
    /// range minimum. The all-zero scripts hit every "no prefix" gate
    /// (one `Next(4)` per prefixable grant) and every chance-roll
    /// minimum; the non-zero picks are pinned by the same-seed replays.
    struct ScriptRng {
        draws: VecDeque<i32>,
        ranges: Vec<i32>,
    }
    impl ScriptRng {
        fn new(draws: &[i32]) -> Self {
            Self {
                draws: VecDeque::from(draws.to_vec()),
                ranges: Vec::new(),
            }
        }
    }
    impl PrefixRandom for ScriptRng {
        fn next_max(&mut self, max: i32) -> i32 {
            self.ranges.push(max);
            let v = self.draws.pop_front().unwrap_or(0);
            assert!((0..max).contains(&v), "draw {v} outside [0, {max})");
            v
        }
    }

    fn items(out: &[BagGrant]) -> Vec<u16> {
        out.iter().map(|g| g.item).collect()
    }

    /// The C# `QuickSpawnItem` replay helper (the boss_bags twin).
    fn spawn_replay(r: &mut UnifiedRandom, want: &mut Vec<BagGrant>, item: i32, stack: i32) {
        if stack <= 0 {
            return;
        }
        let prefix = crate::prefix::roll_prefix(item as u16, -1, r);
        want.push(BagGrant {
            item: item as u16,
            stack: stack as u32,
            prefix,
        });
    }

    #[test]
    fn w4_f35_the_crate_sets_are_the_26_and_13_id_tables() {
        // `ItemID.cs:1309-1310`.
        let all = [
            2334u16, 2335, 2336, 3203, 3204, 3205, 3206, 3207, 3208, 4405, 4407, 4877, 5002,
            3979, 3980, 3981, 3982, 3983, 3984, 3985, 3986, 3987, 4406, 4408, 4878, 5003,
        ];
        let hard = [
            3979u16, 3980, 3981, 3982, 3983, 3984, 3985, 3986, 3987, 4406, 4408, 4878, 5003,
        ];
        for k in all {
            assert!(is_fishing_crate(k), "{k}");
        }
        for k in hard {
            assert!(is_hardmode_crate(k), "{k}");
            assert!(is_fishing_crate(k), "{k}");
        }
        for k in [0u16, 2333, 2337, 3202, 3209, 4404, 4409, 4876, 4879, 5001, 5004, 3318] {
            assert!(!is_fishing_crate(k), "{k} not a crate");
        }
        for k in [2334u16, 2335, 2336, 3203, 3207, 4405, 4407, 4877, 5002] {
            assert!(!is_hardmode_crate(k), "{k} not hardmode");
        }
    }

    #[test]
    fn w4_f35_wooden_crate_zero_script_grants_the_whole_pass_and_tails() {
        // The all-zero pass: the 1/40 Sailfish Boots 3200 clears flag3
        // first; the 1/20 accessory pick + the 1/50 Extractinator gates
        // still DRAW (no flag3 guard); the coin arm's 1/3 gold lands
        // 73 ×1, the 1/7 bars pick 12 (Copper Bar) ×4, the 1/7 potion
        // pick 288 (Obsidian Skin) ×1 — one pass. Tails: the 1/3 healing
        // 28 ×5 + the 1/3 bait (the 2/3 gate != 0 is FALSE at zero →
        // 2675) ×1.
        let mut rng = ScriptRng::new(&[]);
        let mut out = Vec::new();
        open_fishing_crate(2334, &CrateEnv::default(), &mut rng, &mut out);
        assert_eq!(items(&out), vec![3200, 285, 73, 12, 288, 28, 2675]);
        let stacks: Vec<u32> = out.iter().map(|g| g.stack).collect();
        assert_eq!(stacks, vec![1, 1, 1, 4, 1, 5, 1]);
        // The draw census: the first 40 gate hits (3200 Sailfish Boots
        // — one Next(4) prefix gate), the second 40 + the 50
        // Extractinator SKIP (flag3 false); the UNGUARDED 20+6 pick
        // still fires (285 Aglet + its Next(4)); the 7 coin gate + the
        // 3 gold gate + the stack range(1,6)→5; the 7 bar gate + the 4
        // pick + the 2 hardmode gate (a hit, but `& flag` false draws
        // NO pick) + the range(4,16)→12; the 7 potion gate + the 10
        // pick + the range(1,4)→3; the tails 3 + 2 + range(5,16)→11,
        // 3 + 3 + range(1,5)→4.
        assert_eq!(
            rng.ranges,
            vec![40, 4, 20, 6, 4, 7, 3, 5, 7, 4, 2, 12, 7, 10, 3, 3, 2, 11, 3, 3, 4],
            "the wooden pre-hardmode draw census"
        );
    }

    #[test]
    fn w4_f35_wooden_hardmode_adds_the_sundial_and_wings_gates_and_repicks() {
        // The hardmode variant: the 1/200 sundial hits at zero (3064),
        // the two 1/40 accessory gates SKIP (flag3 false — no draws),
        // the 1/25 Anchor gate skips too; the 1/20 pick + the coin/bar/
        // potion arms still draw; the bar hardmode re-pick fires at
        // zero (the gate Next(2)==0 hits) → 364 (Cobalt Bar).
        let mut rng = ScriptRng::new(&[]);
        let mut out = Vec::new();
        open_fishing_crate(3979, &CrateEnv::default(), &mut rng, &mut out);
        assert_eq!(items(&out), vec![3064, 285, 73, 364, 288, 28, 2675]);
        // The census: 200 (sundial), 20+6+4 (the unguarded accessory
        // pick 285 + its prefix gate), 7+3+5 (coins), 7+4+2+2+12
        // (bars: gate, pick, hardmode gate HIT, re-pick, stack),
        // 7+10+3 (potions), then the tails.
        assert_eq!(
            rng.ranges,
            vec![200, 20, 6, 4, 7, 3, 5, 7, 4, 2, 2, 12, 7, 10, 3, 3, 2, 11, 3, 3, 4],
            "the wooden hardmode draw census"
        );
    }

    #[test]
    fn w4_f35_iron_crate_zero_script_grants_the_whole_pass_and_tails() {
        // Zero pass: the 1/25 Ginger Beard 2501 clears flag4 (the later
        // accessory gates skip); the 1/4 gold 73 ×5; the 1/6 bars pick
        // 12 ×12 (the hardmode gate draws + misses); the 1/4 potion
        // pick 288 ×2. Tails: the 1/2 lesser potion Next(188,190)=188
        // ×5, the 1/2 bait (Next(3)!=0 false → 2676) ×2.
        let mut rng = ScriptRng::new(&[]);
        let mut out = Vec::new();
        open_fishing_crate(2335, &CrateEnv::default(), &mut rng, &mut out);
        assert_eq!(items(&out), vec![2501, 73, 12, 288, 188, 2676]);
        let stacks: Vec<u32> = out.iter().map(|g| g.stack).collect();
        assert_eq!(stacks, vec![1, 5, 12, 2, 5, 2]);
        // 2501 Ginger Beard is a VANITY accessory — no prefix draws.
        assert_eq!(
            rng.ranges,
            vec![25, 4, 6, 6, 2, 10, 4, 8, 3, 2, 2, 11, 2, 3, 3],
            "the iron pre-hardmode draw census"
        );
    }

    #[test]
    fn w4_f35_iron_hardmode_ore_repick_subtracts_the_stack_draw() {
        // Force every gate to MISS until the ore arm: 60→1 (sundial),
        // 25→1, 20→1, 15→1, 20→1, 20→1 (accessories), 4→1 (gold),
        // 6→1 (bars), 4→0 (ores HIT), 6→0 (pick 20), range(4,8)→0
        // (stack 4), 3→1 (the 2/3 hardmode gate HITS), 4→0 (re-pick
        // 381 Cobalt Ore), 2→1 (stack 4-1=3), 4→1 (potion miss), then
        // the tails 2→1, 2→1. No retry: the ore grant cleared flag4
        // but the loop's gates all drew this pass; flag4 false ends it.
        let mut rng = ScriptRng::new(&[1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 1, 0, 1, 1, 1, 1]);
        let mut out = Vec::new();
        open_fishing_crate(3980, &CrateEnv::default(), &mut rng, &mut out);
        assert_eq!(items(&out), vec![381]);
        assert_eq!(out[0].stack, 3, "the 4-minimum stack minus the Next(2)=1");
    }

    #[test]
    fn w4_f35_golden_crate_zero_script_grants_the_pass_and_all_four_tails() {
        // Zero pass: the 1/8 Life Crystal 29 clears flag2 (the 1/10
        // saddle skips); the 1/3 gold 73 ×8; the 1/5 bars pick 14 ×25
        // (the hardmode gate draws + misses). Tails: the 1/3 potion
        // 288 ×2, the 1/2 lesser 188 ×5, the 2/3 master bait
        // (Next(3)!=0 FALSE at zero → NO bait), the pre-hardmode 1/30
        // Enchanted Sword 989 hits, the 1/15 hardmode gate draws +
        // `flag` refuses.
        let mut rng = ScriptRng::new(&[]);
        let mut out = Vec::new();
        open_fishing_crate(2336, &CrateEnv::default(), &mut rng, &mut out);
        assert_eq!(items(&out), vec![29, 73, 14, 288, 188, 989]);
        let stacks: Vec<u32> = out.iter().map(|g| g.stack).collect();
        assert_eq!(stacks, vec![1, 8, 25, 2, 5, 1]);
        // The 1/10 saddle gate SKIPS (flag2 false after the crystal).
        // The 989 Enchanted Sword IS a melee weapon: its Prefix(-1)
        // adds one Next(4) between the 30 and the 15.
        assert_eq!(
            rng.ranges,
            vec![8, 3, 13, 5, 4, 2, 10, 3, 5, 4, 2, 16, 3, 30, 4, 15],
            "the golden pre-hardmode draw census"
        );
    }

    #[test]
    fn w4_f35_golden_hardmode_band_gate_moves_to_one_in_fifteen() {
        // The hardmode variant: the 1/20 sundial hits at zero (3064),
        // the 1/8 + 1/10 gates skip, the bar re-pick lands 365, the
        // 1/30 pre-hardmode gate draws but its `!flag` refuses; the
        // 1/15 hardmode gate hits (the Enchanted Sword 989).
        let mut rng = ScriptRng::new(&[]);
        let mut out = Vec::new();
        open_fishing_crate(3981, &CrateEnv::default(), &mut rng, &mut out);
        assert_eq!(items(&out), vec![3064, 73, 365, 288, 188, 989]);
    }

    /// The Wooden crate's C# replay (`Player.cs:8034-8262`) on a twin
    /// `UnifiedRandom`: the same expression order, the same gates, the
    /// same `Prefix(-1)` interleave. Returns the grants + the final
    /// stream state for the byte-equality pin.
    fn wooden_replay(seed: i32, flag: bool) -> (Vec<BagGrant>, UnifiedRandom) {
        let mut r = UnifiedRandom::new(seed);
        let mut w = Vec::new();
        let mut f3_ = true;
        while f3_ {
            if flag && f3_ && r.next_max(200) == 0 {
                spawn_replay(&mut r, &mut w, 3064, 1);
                f3_ = false;
            }
            if f3_ && r.next_max(40) == 0 {
                spawn_replay(&mut r, &mut w, 3200, 1);
                f3_ = false;
            }
            if f3_ && r.next_max(40) == 0 {
                spawn_replay(&mut r, &mut w, 3201, 1);
                f3_ = false;
            }
            if flag && f3_ && r.next_max(25) == 0 {
                spawn_replay(&mut r, &mut w, 2424, 1);
                f3_ = false;
            }
            if r.next_max(20) == 0 {
                let n = r.next_max(6);
                let item = match n {
                    0 => 285,
                    1 => 953,
                    2 => 4341,
                    3 => 3068,
                    4 => 3084,
                    _ => 6165,
                };
                spawn_replay(&mut r, &mut w, item, 1);
                f3_ = false;
            }
            if !flag && f3_ && r.next_max(50) == 0 {
                spawn_replay(&mut r, &mut w, 997, 1);
                f3_ = false;
            }
            if r.next_max(7) == 0 {
                let (item, stack) = if r.next_max(3) == 0 {
                    (73, r.next_range(1, 6))
                } else {
                    (72, r.next_range(20, 91))
                };
                spawn_replay(&mut r, &mut w, item, stack);
                f3_ = false;
            }
            if r.next_max(7) == 0 {
                let mut n5 = match r.next_max(4) {
                    0 => 12,
                    1 => 699,
                    2 => 11,
                    _ => 700,
                };
                if r.next_max(2) == 0 && flag {
                    n5 = match r.next_max(2) {
                        0 => 364,
                        _ => 1104,
                    };
                }
                spawn_replay(&mut r, &mut w, n5, r.next_range(4, 16));
                f3_ = false;
            } else if r.next_max(8) == 0 {
                let mut n6 = match r.next_max(4) {
                    0 => 20,
                    1 => 703,
                    2 => 22,
                    3 => 704,
                    4 => 21,
                    5 => 705,
                    6 => 19,
                    _ => 706,
                };
                let mut s2 = r.next_range(2, 6);
                if r.next_max(2) == 0 && flag {
                    n6 = match r.next_max(2) {
                        0 => 381,
                        1 => 1184,
                        2 => 382,
                        3 => 1191,
                        4 => 391,
                        _ => 1198,
                    };
                    s2 = r.next_range(2, 4);
                }
                spawn_replay(&mut r, &mut w, n6, s2);
                f3_ = false;
            }
            if r.next_max(7) == 0 {
                let n7 = match r.next_max(10) {
                    0 => 288,
                    1 => 290,
                    2 => 292,
                    3 => 299,
                    4 => 298,
                    5 => 304,
                    6 => 291,
                    7 => 2322,
                    8 => 2323,
                    _ => 2329,
                };
                spawn_replay(&mut r, &mut w, n7, r.next_range(1, 4));
                f3_ = false;
            }
        }
        if r.next_max(3) == 0 {
            let n8 = match r.next_max(2) {
                0 => 28,
                _ => 110,
            };
            spawn_replay(&mut r, &mut w, n8, r.next_range(5, 16));
        }
        if r.next_max(3) == 0 {
            let bait = if r.next_max(3) != 0 { 2674 } else { 2675 };
            spawn_replay(&mut r, &mut w, bait, r.next_range(1, 5));
        }
        (w, r)
    }

    #[test]
    fn w4_f35_wooden_crate_replays_the_csharp_draw_order_both_modes() {
        // Same-seed twins: the port's grants (items + stacks + the
        // interleaved prefix rolls) AND the final stream state must
        // match the verbatim C# replay for both crate variants.
        for seed in [1i32, 7, 42, 1234, 99999] {
            for (kind, flag) in [(2334u16, false), (3979, true)] {
                let mut rng = UnifiedRandom::new(seed);
                let mut out = Vec::new();
                open_fishing_crate(kind, &CrateEnv::default(), &mut rng, &mut out);
                let (want, r2) = wooden_replay(seed, flag);
                assert_eq!(out, want, "seed {seed} kind {kind} grants");
                assert_eq!(rng, r2, "seed {seed} kind {kind} stream state");
            }
        }
    }

    /// The Iron crate's C# replay (`Player.cs:8263-8441`).
    fn iron_replay(seed: i32, flag: bool) -> (Vec<BagGrant>, UnifiedRandom) {
        let mut r = UnifiedRandom::new(seed);
        let mut w = Vec::new();
        let mut f4 = true;
        while f4 {
            if flag && f4 && r.next_max(60) == 0 {
                spawn_replay(&mut r, &mut w, 3064, 1);
                f4 = false;
            }
            if f4 && r.next_max(25) == 0 {
                spawn_replay(&mut r, &mut w, 2501, 1);
                f4 = false;
            }
            if f4 && r.next_max(20) == 0 {
                spawn_replay(&mut r, &mut w, 2587, 1);
                f4 = false;
            }
            if f4 && r.next_max(15) == 0 {
                spawn_replay(&mut r, &mut w, 2608, 1);
                f4 = false;
            }
            if f4 && r.next_max(20) == 0 {
                spawn_replay(&mut r, &mut w, 3200, 1);
                f4 = false;
            }
            if f4 && r.next_max(20) == 0 {
                spawn_replay(&mut r, &mut w, 3201, 1);
                f4 = false;
            }
            if r.next_max(4) == 0 {
                spawn_replay(&mut r, &mut w, 73, r.next_range(5, 11));
                f4 = false;
            }
            if r.next_max(6) == 0 {
                let mut n9 = match r.next_max(6) {
                    0 => 12,
                    1 => 699,
                    2 => 11,
                    3 => 700,
                    4 => 14,
                    _ => 701,
                };
                if r.next_max(2) == 0 && flag {
                    n9 = match r.next_max(4) {
                        0 => 364,
                        1 => 1104,
                        2 => 365,
                        _ => 1105,
                    };
                }
                spawn_replay(&mut r, &mut w, n9, r.next_range(12, 22));
                f4 = false;
            } else if r.next_max(4) == 0 {
                let mut n10 = match r.next_max(6) {
                    0 => 20,
                    1 => 703,
                    2 => 22,
                    3 => 704,
                    4 => 21,
                    _ => 705,
                };
                let mut n11 = r.next_range(4, 8);
                if r.next_max(3) != 0 && flag {
                    n10 = match r.next_max(4) {
                        0 => 381,
                        1 => 1184,
                        2 => 382,
                        _ => 1191,
                    };
                    n11 -= r.next_max(2);
                }
                spawn_replay(&mut r, &mut w, n10, n11);
                f4 = false;
            }
            if r.next_max(4) == 0 {
                let n12 = match r.next_max(8) {
                    0 => 288,
                    1 => 296,
                    2 => 304,
                    3 => 305,
                    4 => 2322,
                    5 => 2323,
                    6 => 2324,
                    _ => 2327,
                };
                spawn_replay(&mut r, &mut w, n12, r.next_range(2, 5));
                f4 = false;
            }
        }
        if r.next_max(2) == 0 {
            spawn_replay(&mut r, &mut w, r.next_range(188, 190), r.next_range(5, 16));
        }
        if r.next_max(2) == 0 {
            let bait = if r.next_max(3) != 0 { 2675 } else { 2676 };
            spawn_replay(&mut r, &mut w, bait, r.next_range(2, 5));
        }
        (w, r)
    }

    /// The Golden crate's C# replay (`Player.cs:8442-8593`).
    fn golden_replay(seed: i32, flag: bool) -> (Vec<BagGrant>, UnifiedRandom) {
        let mut r = UnifiedRandom::new(seed);
        let mut w = Vec::new();
        let mut f2 = true;
        while f2 {
            if flag && f2 && r.next_max(20) == 0 {
                spawn_replay(&mut r, &mut w, 3064, 1);
                f2 = false;
            }
            if f2 && r.next_max(8) == 0 {
                spawn_replay(&mut r, &mut w, 29, 1);
                f2 = false;
            }
            if f2 && r.next_max(10) == 0 {
                spawn_replay(&mut r, &mut w, 2491, 1);
                f2 = false;
            }
            if r.next_max(3) == 0 {
                spawn_replay(&mut r, &mut w, 73, r.next_range(8, 21));
                f2 = false;
            }
            if r.next_max(5) == 0 {
                let mut n = match r.next_max(4) {
                    0 => 14,
                    1 => 701,
                    2 => 13,
                    _ => 702,
                };
                if r.next_max(2) == 0 && flag {
                    n = match r.next_max(4) {
                        0 => 365,
                        1 => 1105,
                        2 => 366,
                        _ => 1106,
                    };
                }
                spawn_replay(&mut r, &mut w, n, r.next_range(25, 35));
                f2 = false;
            } else {
                if r.next_max(3) != 0 {
                    continue;
                }
                let mut n2 = match r.next_max(4) {
                    0 => 21,
                    1 => 19,
                    2 => 705,
                    _ => 706,
                };
                if r.next_max(3) != 0 && flag {
                    n2 = match r.next_max(4) {
                        0 => 382,
                        1 => 391,
                        2 => 1191,
                        _ => 1198,
                    };
                }
                spawn_replay(&mut r, &mut w, n2, r.next_range(8, 12));
                f2 = false;
            }
        }
        if r.next_max(3) == 0 {
            let n3 = match r.next_max(5) {
                0 => 288,
                1 => 296,
                2 => 305,
                3 => 2322,
                _ => 2323,
            };
            spawn_replay(&mut r, &mut w, n3, r.next_range(2, 6));
        }
        if r.next_max(2) == 0 {
            spawn_replay(&mut r, &mut w, r.next_range(188, 190), r.next_range(5, 21));
        }
        if r.next_max(3) != 0 {
            spawn_replay(&mut r, &mut w, 2676, r.next_range(3, 8));
        }
        if r.next_max(30) == 0 && !flag {
            spawn_replay(&mut r, &mut w, 989, 1);
        }
        if r.next_max(15) == 0 && flag {
            spawn_replay(&mut r, &mut w, 989, 1);
        }
        (w, r)
    }

    #[test]
    fn w4_f35_iron_and_golden_crates_replay_the_csharp_draw_order_both_modes() {
        for seed in [1i32, 7, 42, 1234, 99999] {
            for (kind, flag) in [(2335u16, false), (3980, true)] {
                let mut rng = UnifiedRandom::new(seed);
                let mut out = Vec::new();
                open_fishing_crate(kind, &CrateEnv::default(), &mut rng, &mut out);
                let (want, r2) = iron_replay(seed, flag);
                assert_eq!(out, want, "iron seed {seed} kind {kind} grants");
                assert_eq!(rng, r2, "iron seed {seed} kind {kind} stream state");
            }
            for (kind, flag) in [(2336u16, false), (3981, true)] {
                let mut rng = UnifiedRandom::new(seed);
                let mut out = Vec::new();
                open_fishing_crate(kind, &CrateEnv::default(), &mut rng, &mut out);
                let (want, r2) = golden_replay(seed, flag);
                assert_eq!(out, want, "golden seed {seed} kind {kind} grants");
                assert_eq!(rng, r2, "golden seed {seed} kind {kind} stream state");
            }
        }
    }

    #[test]
    fn w4_f35_every_shared_crate_zero_script_grants_the_traced_rows() {
        // The all-zero pass through the shared loop: the biome weapon
        // (maxValue=1 always passes), the 1/4 gold 73 ×5, the 1/7 bars
        // (pick 0 → 12; the hardmode gate HITS at zero: pre-hardmode
        // `& flag` refuses, hardmode re-picks 364), the 1/4 ores (pick
        // 0 → 22, stack 6; the hardmode Next(3)!=0 gate MISSES at zero
        // — no re-pick either mode). The shared tails: 288 ×2, 188 ×5,
        // 2676 ×2 (the bait Next(2)!=0 false → Master). Per-biome tails
        // all hit at zero. The lava 174/175 re-types are OVERWRITTEN by
        // the hardmode re-picks (364/381) — the C# order.
        let rows: &[(u16, &[u16], &[u32])] = &[
            (3203, &[162, 73, 12, 22, 288, 188, 2676], &[1, 5, 20, 6, 2, 5, 2]),
            (
                3982,
                &[162, 73, 364, 22, 288, 188, 2676, 521, 522],
                &[1, 5, 20, 6, 2, 5, 2, 2, 2],
            ),
            (3204, &[800, 73, 12, 22, 288, 188, 2676], &[1, 5, 20, 6, 2, 5, 2]),
            (
                3983,
                &[800, 73, 364, 22, 288, 188, 2676, 521, 1332],
                &[1, 5, 20, 6, 2, 5, 2, 2, 2],
            ),
            (
                3205,
                &[3085, 149, 134, 73, 12, 22, 288, 188, 2676, 1438],
                &[1, 5, 25, 5, 20, 6, 2, 5, 2, 1],
            ),
            (
                3984,
                &[3085, 149, 134, 73, 364, 22, 288, 188, 2676, 1438],
                &[1, 5, 25, 5, 20, 6, 2, 5, 2, 1],
            ),
            (
                3206,
                &[158, 73, 12, 22, 288, 188, 2676, 5226, 751, 4978],
                &[1, 5, 20, 6, 2, 5, 2, 1, 50, 1],
            ),
            (
                3985,
                &[158, 73, 364, 22, 288, 188, 2676, 5226, 751, 4978],
                &[1, 5, 20, 6, 2, 5, 2, 1, 50, 1],
            ),
            // The Hallowed crate has NO weapon arm (`:8595-8914`).
            (3207, &[73, 12, 22, 288, 188, 2676], &[5, 20, 6, 2, 5, 2]),
            (
                3986,
                &[73, 364, 22, 288, 188, 2676, 520, 502],
                &[5, 20, 6, 2, 5, 2, 2, 4],
            ),
            (
                3208,
                &[3017, 73, 12, 22, 288, 188, 2676, 4564, 753],
                &[1, 5, 20, 6, 2, 5, 2, 20, 1],
            ),
            (
                3987,
                &[3017, 73, 364, 22, 288, 188, 2676, 4564, 753],
                &[1, 5, 20, 6, 2, 5, 2, 20, 1],
            ),
            (4405, &[670, 73, 12, 22, 288, 188, 2676, 669], &[1, 5, 20, 6, 2, 5, 2, 1]),
            (4406, &[670, 73, 364, 22, 288, 188, 2676, 669], &[1, 5, 20, 6, 2, 5, 2, 1]),
            (
                4407,
                &[4056, 4423, 3380, 857, 73, 12, 22, 288, 188, 2676, 4639],
                &[1, 4, 10, 1, 5, 20, 6, 2, 5, 2, 1],
            ),
            (
                4408,
                &[4056, 4423, 3380, 857, 73, 364, 22, 288, 188, 2676, 4639],
                &[1, 4, 10, 1, 5, 20, 6, 2, 5, 2, 1],
            ),
            (
                4877,
                &[906, 4858, 4879, 4824, 4902, 73, 174, 175, 288, 188, 2676, 1497, 221, 4443, 4737, 4551],
                &[1, 2, 1, 7, 1, 5, 20, 6, 2, 5, 2, 1, 1, 1, 1, 1],
            ),
            (
                4878,
                &[906, 4858, 4879, 4824, 4902, 73, 364, 175, 288, 188, 2676, 1497, 221, 4443, 4737, 4551],
                &[1, 2, 1, 7, 1, 5, 20, 6, 2, 5, 2, 1, 1, 1, 1, 1],
            ),
            (
                5002,
                &[863, 73, 12, 22, 288, 188, 2676, 5527, 4090, 4460, 4425],
                &[1, 5, 20, 6, 2, 5, 2, 1, 20, 1, 1],
            ),
            (
                5003,
                &[863, 73, 364, 22, 288, 188, 2676, 5527, 4090, 4460, 4425],
                &[1, 5, 20, 6, 2, 5, 2, 1, 20, 1, 1],
            ),
        ];
        for &(kind, want_items, want_stacks) in rows {
            let mut rng = ScriptRng::new(&[]);
            let mut out = Vec::new();
            open_fishing_crate(kind, &CrateEnv::default(), &mut rng, &mut out);
            assert_eq!(items(&out), want_items, "kind {kind} grants");
            let stacks: Vec<u32> = out.iter().map(|g| g.stack).collect();
            assert_eq!(stacks, want_stacks, "kind {kind} stacks");
        }
    }

    #[test]
    fn w4_f35_corrupt_crate_zero_script_draw_census() {
        // The corrupt 3203 pass: weapon Next(1) + Next(5) pick 162
        // (a boomerang — one Next(4) prefix gate); gold Next(4) +
        // range(5,13)→8; bars Next(7) + Next(8) + Next(2) hardmode gate
        // (hit, `& flag` false) + range(20,36)→16; ore gate Next(4) +
        // Next(6) + range(6,17)→11 + Next(3) hardmode gate (miss);
        // tails Next(4) + Next(6) + range(2,5)→3, Next(2) +
        // range(188,190)→2 + range(5,18)→13, Next(2) + Next(2) bait
        // gate + range(2,7)→5. No corrupt tail before the hardmode
        // gate (`!flag` returns).
        let mut rng = ScriptRng::new(&[]);
        let mut out = Vec::new();
        open_fishing_crate(3203, &CrateEnv::default(), &mut rng, &mut out);
        assert_eq!(
            rng.ranges,
            vec![1, 5, 4, 4, 8, 7, 8, 2, 16, 4, 6, 11, 3, 4, 6, 3, 2, 2, 13, 2, 2, 5],
            "the corrupt pre-hardmode draw census"
        );
    }

    #[test]
    fn w4_f35_frozen_crate_remix_swaps_the_ice_blade_pick() {
        // Script the weapon pick to 3 (Ice Blade 1319): the remix seed
        // swaps it to the Ice Bow 725 AFTER the pick (`:8688-8691`).
        let mut rng = ScriptRng::new(&[0, 3]);
        let mut out = Vec::new();
        let env = CrateEnv { remix: true, ..Default::default() };
        open_fishing_crate(4405, &env, &mut rng, &mut out);
        assert_eq!(out[0].item, 725, "the remix swap");
        // Without the seed the 1319 stays.
        let mut rng2 = ScriptRng::new(&[0, 3]);
        let mut out2 = Vec::new();
        open_fishing_crate(4405, &CrateEnv::default(), &mut rng2, &mut out2);
        assert_eq!(out2[0].item, 1319, "no swap on a classic world");
    }

    #[test]
    fn w4_f35_sky_crate_low_tiles_seam_gates_the_sky_mill() {
        // `:9153` — the lowTiles gate: false draws NOTHING (the seam
        // off), true draws the Next(3) and hits at zero (2197).
        let mut off = ScriptRng::new(&[]);
        let mut out_off = Vec::new();
        open_fishing_crate(3206, &CrateEnv::default(), &mut off, &mut out_off);
        let mut on = ScriptRng::new(&[]);
        let mut out_on = Vec::new();
        let env = CrateEnv { skyblock_low_tiles: true, ..Default::default() };
        open_fishing_crate(3206, &env, &mut on, &mut out_on);
        assert_eq!(items(&out_on), {
            let mut v = items(&out_off);
            v.push(2197);
            v
        });
        assert_eq!(on.ranges.len(), off.ranges.len() + 1, "one extra Next(3)");
        assert_eq!(*on.ranges.last().unwrap(), 3);
    }

    #[test]
    fn w4_f35_hallowed_hardmode_soul_stack_is_redrawn() {
        // `:9237/:9245` — the second soul row draws range(2,6) FIRST,
        // then the 3986 arm OVERWRITES it with range(4,11). Script the
        // two draws apart: the shared tails all miss (4→1, 2→1, 2→1),
        // the first soul gate 2→0 (520, the range(2,6)→4 draw at 1 →
        // stack 3), the second 2→0 (the discarded range(2,6) at 3, then
        // the re-draw range(4,11)→7 at 6 → stack 10).
        // The pass itself: no weapon arm; gold 4→1 miss, bars 7→1
        // miss, ore gate 4→0 hit + 6→0 (22) + range 11→0 (6) + 3→0
        // (no re-pick) — flag5 false.
        let mut rng = ScriptRng::new(&[
            1, 1, 0, 0, 0, 0, // gold miss, bars miss, ore hit + pick + stack + gate
            1, 1, 1, // the three shared tails miss
            0, 1, // soul 1: 520 ×(2+1)=3 (the range(2,6)→4 draw)
            0, 3, 6, // soul 2: gate, discarded range(2,6)=2+3, re-draw range(4,11)=4+6=10
        ]);
        let mut out = Vec::new();
        open_fishing_crate(3986, &CrateEnv::default(), &mut rng, &mut out);
        assert_eq!(items(&out), vec![22, 520, 502]);
        assert_eq!(out[1].stack, 3);
        assert_eq!(out[2].stack, 10, "the RE-DRAWN stack, not the first");
    }

    /// The shared-loop + tails C# replay (`Player.cs:8595-9249`) on a
    /// twin `UnifiedRandom` — the verbatim expression order for the 20
    /// non-base crates.
    fn shared_replay(seed: i32, kind: u16, env: &CrateEnv) -> (Vec<BagGrant>, UnifiedRandom) {
        let mut r = UnifiedRandom::new(seed);
        let mut w = Vec::new();
        let flag = is_hardmode_crate(kind);
        let lava = kind == 4877 || kind == 4878;
        let max_value = 1;
        let mut f5 = true;
        while f5 {
            if (kind == 5002 || kind == 5003) && f5 && r.next_max(max_value) == 0 {
                let n13 = r.next_max(4);
                let item = if r.next_max(10) == 0 {
                    863
                } else {
                    match n13 {
                        0 => 186,
                        1 => 4404,
                        2 => 277,
                        _ => 187,
                    }
                };
                spawn_replay(&mut r, &mut w, item, 1);
                f5 = false;
            }
            if (kind == 3203 || kind == 3982) && f5 && r.next_max(max_value) == 0 {
                let item = match r.next_max(5) {
                    0 => 162,
                    1 => 111,
                    2 => 96,
                    3 => 115,
                    _ => 64,
                };
                spawn_replay(&mut r, &mut w, item, 1);
                f5 = false;
            }
            if (kind == 3204 || kind == 3983) && f5 && r.next_max(max_value) == 0 {
                let item = match r.next_max(5) {
                    0 => 800,
                    1 => 802,
                    2 => 1256,
                    3 => 1290,
                    _ => 3062,
                };
                spawn_replay(&mut r, &mut w, item, 1);
                f5 = false;
            }
            if (kind == 3205 || kind == 3984) && f5 && r.next_max(max_value) == 0 {
                spawn_replay(&mut r, &mut w, 3085, 1);
                f5 = false;
                if r.next_max(2) == 0 {
                    spawn_replay(&mut r, &mut w, 149, r.next_range(5, 16));
                }
                if r.next_max(2) == 0 {
                    let item = match r.next_max(3) {
                        0 => 134,
                        1 => 137,
                        _ => 139,
                    };
                    spawn_replay(&mut r, &mut w, item, r.next_range(25, 51));
                }
            }
            if (kind == 3206 || kind == 3985) && f5 && r.next_max(max_value) == 0 {
                let item = match r.next_max(4) {
                    0 => 158,
                    1 => 65,
                    2 => 159,
                    _ => 2219,
                };
                spawn_replay(&mut r, &mut w, item, 1);
                f5 = false;
            }
            if (kind == 3208 || kind == 3987) && f5 && r.next_max(max_value) == 0 {
                if r.next_max(20) == 0 {
                    let _n14 = r.next_max(5);
                    spawn_replay(&mut r, &mut w, 3017, 1);
                    f5 = false;
                } else {
                    let item = match r.next_max(5) {
                        0 => 212,
                        1 => 964,
                        2 => 211,
                        3 => 213,
                        _ => 2292,
                    };
                    spawn_replay(&mut r, &mut w, item, 1);
                    f5 = false;
                }
            }
            if (kind == 4405 || kind == 4406) && f5 && r.next_max(max_value) == 0 {
                let mut n15 = [670i32, 724, 950, 1319, 987, 1579, 6153][r.next_max(7) as usize];
                if env.remix && n15 == 1319 {
                    n15 = 725;
                }
                spawn_replay(&mut r, &mut w, n15, 1);
                f5 = false;
            }
            if kind == 4407 || kind == 4408 {
                if f5 && r.next_max(max_value) == 0 {
                    let item = match r.next_max(8) {
                        0 => 4056,
                        1 => 4442,
                        2 => 4055,
                        3 => 4061,
                        4 => 4062,
                        5 => 4276,
                        6 => 4262,
                        _ => 4263,
                    };
                    spawn_replay(&mut r, &mut w, item, 1);
                    f5 = false;
                }
                if r.next_max(4) == 0 {
                    spawn_replay(&mut r, &mut w, 4423, r.next_range(4, 7));
                    f5 = false;
                }
                if r.next_max(2) == 0 {
                    spawn_replay(&mut r, &mut w, 3380, r.next_range(10, 17));
                    f5 = false;
                }
                if r.next_max(35) == 0 {
                    spawn_replay(&mut r, &mut w, 857, 1);
                    f5 = false;
                }
            }
            if lava {
                if f5 && r.next_max(max_value) == 0 {
                    if r.next_max(20) == 0 {
                        let _n16 = r.next_max(5);
                        spawn_replay(&mut r, &mut w, 906, 1);
                        f5 = false;
                    } else {
                        let item = match r.next_max(5) {
                            0 => 4822,
                            1 => 4828,
                            2 => 4880,
                            3 => 4881,
                            _ => 4868,
                        };
                        spawn_replay(&mut r, &mut w, item, 1);
                        f5 = false;
                    }
                    if r.next_max(4) == 0 {
                        spawn_replay(&mut r, &mut w, 4858, 2);
                        f5 = false;
                    }
                }
                spawn_replay(&mut r, &mut w, 4879, 1);
                f5 = false;
                if r.next_max(3) == 0 {
                    spawn_replay(&mut r, &mut w, 4824, r.next_range(7, 11));
                    f5 = false;
                }
                if r.next_max(2) == 0 {
                    let n17 = match r.next_max(5) {
                        0 => 4902,
                        1 => 4903,
                        2 => 4904,
                        3 => 4905,
                        _ => 4906,
                    };
                    spawn_replay(&mut r, &mut w, n17, 1);
                    f5 = false;
                }
            }
            if r.next_max(4) == 0 {
                spawn_replay(&mut r, &mut w, 73, r.next_range(5, 13));
                f5 = false;
            }
            if r.next_max(7) == 0 {
                let mut n18 = match r.next_max(8) {
                    0 => 12,
                    1 => 699,
                    2 => 11,
                    3 => 700,
                    4 => 14,
                    5 => 701,
                    6 => 13,
                    _ => 702,
                };
                if lava {
                    n18 = 174;
                }
                if r.next_max(2) == 0 && flag {
                    n18 = match r.next_max(6) {
                        0 => 364,
                        1 => 1104,
                        2 => 365,
                        3 => 1105,
                        4 => 366,
                        _ => 1106,
                    };
                }
                spawn_replay(&mut r, &mut w, n18, r.next_range(20, 36));
                f5 = false;
            }
            if r.next_max(4) != 0 {
                continue;
            }
            let mut n19 = match r.next_max(6) {
                0 => 22,
                1 => 21,
                2 => 19,
                3 => 704,
                4 => 705,
                _ => 706,
            };
            let mut n20 = r.next_range(6, 17);
            if lava {
                n19 = 175;
            }
            if r.next_max(3) != 0 && flag {
                n19 = match r.next_max(6) {
                    0 => 381,
                    1 => 382,
                    2 => 391,
                    3 => 1184,
                    4 => 1191,
                    _ => 1198,
                };
                n20 -= r.next_max(2);
            }
            spawn_replay(&mut r, &mut w, n19, n20);
            f5 = false;
        }
        shared_tails_replay(&mut r, &mut w, kind, flag, env);
        (w, r)
    }

    /// The shared tails replay (`Player.cs:8915-9249`).
    fn shared_tails_replay(
        r: &mut UnifiedRandom,
        w: &mut Vec<BagGrant>,
        kind: u16,
        flag: bool,
        env: &CrateEnv,
    ) {
        if r.next_max(4) == 0 {
            let n21 = match r.next_max(6) {
                0 => 288,
                1 => 296,
                2 => 304,
                3 => 305,
                4 => 2322,
                _ => 2323,
            };
            spawn_replay(r, w, n21, r.next_range(2, 5));
        }
        if r.next_max(2) == 0 {
            spawn_replay(r, w, r.next_range(188, 190), r.next_range(5, 18));
        }
        if r.next_max(2) == 0 {
            let bait = if r.next_max(2) != 0 { 2675 } else { 2676 };
            spawn_replay(r, w, bait, r.next_range(2, 7));
        }
        if kind == 5002 || kind == 5003 {
            if r.next_max(5) == 0 {
                spawn_replay(r, w, 5527, 1);
            }
            if r.next_max(3) == 0 {
                spawn_replay(r, w, 4090, r.next_range(20, 51));
            }
            if r.next_max(10) == 0 {
                spawn_replay(r, w, 4460, 1);
            }
            if r.next_max(10) == 0 {
                spawn_replay(r, w, 4425, 1);
            }
        }
        if (kind == 3205 || kind == 3984) && r.next_max(2) == 0 {
            let n22 = match r.next_max(25) {
                0 => 1438,
                1 => 1375,
                2 => 1573,
                3 => 1420,
                4 => 1435,
                5 => 1434,
                6 => 1421,
                7 => 5234,
                8 => 1374,
                9 => 1425,
                10 => 1372,
                11 => 1441,
                12 => 1373,
                13 => 1433,
                14 => 1436,
                15 => 1426,
                16 => 1424,
                17 => 1419,
                18 => 2995,
                19 => 1422,
                20 => 1439,
                21 => 1502,
                22 => 1423,
                23 => 1437,
                _ => 1500,
            };
            spawn_replay(r, w, n22, 1);
        }
        if kind == 3208 || kind == 3987 {
            if r.next_max(3) == 0 {
                spawn_replay(r, w, 4564, r.next_range(20, 51));
            }
            if r.next_max(20) == 0 {
                spawn_replay(r, w, 753, 1);
            }
        }
        if (kind == 4405 || kind == 4406) && r.next_max(20) == 0 {
            spawn_replay(r, w, 669, 1);
        }
        if (kind == 4407 || kind == 4408) && r.next_max(2) == 0 {
            let n23 = match r.next_max(14) {
                0 => 4639,
                1 => 4627,
                2 => 4628,
                3 => 4632,
                4 => 4630,
                5 => 4638,
                6 => 4629,
                7 => 4633,
                8 => 4634,
                9 => 4635,
                10 => 4636,
                11 => 4637,
                12 => 4631,
                _ => 4626,
            };
            spawn_replay(r, w, n23, 1);
        }
        if kind == 3206 || kind == 3985 {
            if r.next_max(2) == 0 {
                let n24 = match r.next_max(6) {
                    0 => 5226,
                    1 => 5254,
                    2 => 5238,
                    3 => 5258,
                    4 => 5255,
                    _ => 5388,
                };
                spawn_replay(r, w, n24, 1);
            }
            if r.next_max(2) == 0 {
                spawn_replay(r, w, 751, r.next_range(50, 101));
            }
            if r.next_max(40) == 0 {
                spawn_replay(r, w, 4978, 1);
            }
            if env.skyblock_low_tiles && r.next_max(3) == 0 {
                spawn_replay(r, w, 2197, 1);
            }
        }
        if kind == 4877 || kind == 4878 {
            if r.next_max(2) == 0 {
                let n25 = match r.next_max(12) {
                    0 => 1497,
                    1 => 1475,
                    2 => 1479,
                    3 => 1542,
                    4 => 1476,
                    5 => 1538,
                    6 => 1501,
                    7 => 1478,
                    8 => 1539,
                    9 => 1540,
                    10 => 1499,
                    _ => 1541,
                };
                spawn_replay(r, w, n25, 1);
            }
            if r.next_max(20) == 0 {
                spawn_replay(r, w, 221, 1);
            }
            if r.next_max(20) == 0 {
                spawn_replay(r, w, 4443, 1);
            }
            if r.next_max(20) == 0 {
                spawn_replay(r, w, 4737, 1);
            }
            if r.next_max(20) == 0 {
                spawn_replay(r, w, 4551, 1);
            }
        }
        if !flag || (kind != 3982 && kind != 3986 && kind != 3983) {
            return;
        }
        if r.next_max(2) == 0 {
            let item2 = if kind == 3986 { 520 } else { 521 };
            spawn_replay(r, w, item2, r.next_range(2, 6));
        }
        if r.next_max(2) == 0 {
            let mut item3 = 522;
            let mut stack3 = r.next_range(2, 6);
            match kind {
                3983 => item3 = 1332,
                3986 => {
                    item3 = 502;
                    stack3 = r.next_range(4, 11);
                }
                _ => {}
            }
            spawn_replay(r, w, item3, stack3);
        }
    }

    #[test]
    fn w4_f35_every_shared_crate_replays_the_csharp_draw_order() {
        // The 20 non-base crates × 5 seeds × the classic env: the
        // grants AND the final stream state match the verbatim replay.
        let kinds = [
            3203u16, 3982, 3204, 3983, 3205, 3984, 3206, 3985, 3207, 3986, 3208, 3987, 4405,
            4406, 4407, 4408, 4877, 4878, 5002, 5003,
        ];
        for seed in [1i32, 7, 42, 1234, 99999] {
            for &kind in &kinds {
                let env = CrateEnv::default();
                let mut rng = UnifiedRandom::new(seed);
                let mut out = Vec::new();
                open_fishing_crate(kind, &env, &mut rng, &mut out);
                let (want, r2) = shared_replay(seed, kind, &env);
                assert_eq!(out, want, "seed {seed} kind {kind} grants");
                assert_eq!(rng, r2, "seed {seed} kind {kind} stream state");
            }
        }
        // The two env seams ride the same replay: the remix Frozen
        // swap + the skyblock Sky Mill.
        for seed in [3i32, 77] {
            for (kind, env) in [
                (4405u16, CrateEnv { remix: true, skyblock_low_tiles: false }),
                (4406, CrateEnv { remix: true, skyblock_low_tiles: false }),
                (3206, CrateEnv { remix: false, skyblock_low_tiles: true }),
                (3985, CrateEnv { remix: false, skyblock_low_tiles: true }),
            ] {
                let mut rng = UnifiedRandom::new(seed);
                let mut out = Vec::new();
                open_fishing_crate(kind, &env, &mut rng, &mut out);
                let (want, r2) = shared_replay(seed, kind, &env);
                assert_eq!(out, want, "env seed {seed} kind {kind} grants");
                assert_eq!(rng, r2, "env seed {seed} kind {kind} stream state");
            }
        }
    }
}

