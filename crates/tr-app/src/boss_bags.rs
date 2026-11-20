//! W4-f35 — the expert/master boss bag OPEN trees: `Player.OpenBossBag`
//! (`Player.cs:7202-7883`), the per-bag grant switch the inventory
//! right-click reaches through `ItemSlot.TryOpenContainer`
//! (`UI/ItemSlot.cs:1786-1811`, the `ItemID.Sets.BossBag` gate).
//!
//! The DEATH side (the `ItemDropDatabase` boss registrations — the bag
//! itself behind `ItemDropRule.BossBag`'s expert gate + the money reset,
//! the NotExpert weapons, the master relics/pets) already runs through
//! `tr_core::loot` + `loot_db_gen` (S1-E-14a). This module is the OTHER
//! half: what opening the bag grants, verbatim per boss — the
//! expert-exclusive items (Worm Scarf 3224, Brain of Confusion 3223,
//! Hive Backpack 3333, Bone Glove 3245, Demon Heart 3335, the mech cart
//! pieces 3354-3356, Spore Sac 3336, Shiny Stone 3337, Shrimpy Truffle
//! 3367, …), the master-mode stack bumps (the EoW/BoC ore/ingot counts),
//! the shared `TryGettingDevArmor` roll (`Player.cs:7885-8026`) and the
//! bag money tail (`:7744-7882`).
//!
//! Census: `docs/research/65-expert-drop-trees-census.md`.
//!
//! ## The RNG contract
//!
//! Every draw rides `Main.rand` — the open is a UI click OUTSIDE every
//! `SwapRandom` block, so the wiring feeds `NamedRngs::global()` (the
//! bare-`Main.rand` stand-in, the reforge/chest-transfer precedent).
//! The grant order IS the draw order: each `QuickSpawnItem`
//! (`Player.cs:7170-7180`) runs `SetDefaults` → `Prefix(-1)` (the
//! natural roll, `Item.cs:504-618`, on the SAME stream) → the stack
//! write, so a prefixable grant's prefix draws interleave between the
//! bag's chance rolls. [`quick_spawn`] keeps that order; the output
//! [`BagGrant`] list is the `GetOrDropItem` fill order.
//!
//! ## The e25 boundary
//!
//! The money tail builds a FRESH `NPC.SetDefaults(num9)` (`:7833-7834`)
//! — whose `ScaleStats` tail (`NPC.cs:17856`) scales `value` by the
//! difficulty money curve in expert/master (×2.5, e25's lane, UNWIRED).
//! [`BagEnv::value_multiplier`] is that seam: the wiring passes 1.0
//! (the classic world the port models today); the `(int)` truncation
//! of `ScaleStats_ByDifficulty` (`NPC.cs:18176`) is kept here.

#![allow(dead_code)] // the clicks.rs open arm lands with the wiring commit

use crate::prefix::PrefixRandom;

/// One `QuickSpawnItem` grant (`Player.cs:7170-7191`): the item, the
/// stack and the natural-roll prefix — the `GetOrDropItem` input.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BagGrant {
    pub item: u16,
    pub stack: u32,
    pub prefix: u16,
}

/// The world/player scalars the bag trees read.
#[derive(Clone, Copy, Debug)]
pub struct BagEnv {
    /// `Main.masterMode` (`Player.cs:7204`) — the EoW/BoC stack bumps.
    pub master_mode: bool,
    /// `WorldGen.crimson` (`:7250-7251`) — the EoC bag's ore/bar kind.
    pub crimson: bool,
    /// `Main.remixWorld` (`:7560`) — the Fishron bag's option 6.
    pub remix: bool,
    /// `Main.tenthAnniversaryWorld` (`:7887`) — the dev-armor 1/8 gate.
    pub tenth_anniversary: bool,
    /// `Player.extraAccessory` (`:7380`) — the Demon Heart grant gate.
    pub extra_accessory: bool,
    /// `Player.HasItem(3384)` (`:7726`, `Player.cs:55894-55904`) — the
    /// Moon Lord bag's Portal Gun duplicate gate.
    pub has_portal_gun: bool,
    /// The fresh-`SetDefaults` money curve seam (the module header's
    /// e25 boundary): 1.0 classic, 2.5 expert/master once e25 wires.
    pub value_multiplier: f32,
}

impl Default for BagEnv {
    fn default() -> Self {
        Self {
            master_mode: false,
            crimson: false,
            remix: false,
            tenth_anniversary: false,
            extra_accessory: false,
            has_portal_gun: false,
            value_multiplier: 1.0,
        }
    }
}

/// `ItemID.Sets.BossBag` (`ItemID.cs:1383`): the 21 bag ids. 3861
/// (Ogre) / 3862 (Dark Mage) are in the set but have NO grant case in
/// `OpenBossBag` — opening them runs only the money tail.
pub fn is_boss_bag(kind: u16) -> bool {
    matches!(
        kind,
        3318..=3332 | 3860 | 3861 | 3862 | 4782 | 4957 | 5111
    )
}

/// `UnifiedRandom.Next(min, max)` (`UnifiedRandom.cs:120-132`) over the
/// [`PrefixRandom`] seam: for the small spans every bag roll uses,
/// `(int)(Sample() * (max - min)) + min` == `Next(max - min) + min`
/// (`:134-141`) — one draw, the same value.
fn range<R: PrefixRandom + ?Sized>(rng: &mut R, min: i32, max: i32) -> i32 {
    rng.next_max(max - min) + min
}

/// `Player.QuickSpawnItem(source, int item, int stack)`
/// (`Player.cs:7170-7180`): the `stack > 0` gate, then `SetDefaults` →
/// `Prefix(-1)` (the natural roll on the same `Main.rand` stream,
/// `Item.cs:504-618`) → the stack write. The prefix draws interleaved
/// here ARE the C#'s loot-slot RNG order.
fn quick_spawn<R: PrefixRandom + ?Sized>(
    rng: &mut R,
    out: &mut Vec<BagGrant>,
    item: i32,
    stack: i32,
) {
    if stack <= 0 {
        return; // `:7172`
    }
    let prefix = crate::prefix::roll_prefix(item as u16, -1, rng);
    out.push(BagGrant {
        item: item as u16,
        stack: stack as u32,
        prefix,
    });
}

/// `Player.TryGettingDevArmor` (`Player.cs:7885-8026`): the
/// `Next(tenthAnniversary ? 8 : 16) == 0` gate, then `Next(21)` picks
/// one of the 21 dev sets; each piece rides its own `QuickSpawnItem`.
/// Runs FIRST inside the hardmode bags' arms (3325-3332, 3860, 4782 —
/// NOT the Queen Slime 4957 / Deerclops 5111 bags).
fn try_getting_dev_armor<R: PrefixRandom + ?Sized>(
    rng: &mut R,
    out: &mut Vec<BagGrant>,
    tenth_anniversary: bool,
) {
    // `:7887`
    if rng.next_max(if tenth_anniversary { 8 } else { 16 }) != 0 {
        return;
    }
    // `:7889-8024` — the 21 sets, in case order. The two torch grants
    // (cases 10/11) carry stack 4.
    let set = rng.next_max(21);
    let pieces: &[i32] = match set {
        0 => &[666, 667, 668, 665, 3287],
        1 => &[1554, 1555, 1556, 1586],
        2 => &[1554, 1587, 1588, 1586],
        3 => &[1557, 1558, 1559, 1585],
        4 => &[1560, 1561, 1562, 1584],
        5 => &[1563, 1564, 1565, 3582],
        6 => &[1566, 1567, 1568],
        7 => &[1580, 1581, 1582, 1583],
        8 => &[3226, 3227, 3228, 3288],
        9 => &[3583, 3581, 3578, 3579, 3580],
        10 => &[3585, 3586, 3587, 3588],
        11 => &[3589, 3590, 3591, 3592],
        12 => &[3368, 3921, 3922, 3923, 3924],
        13 => &[3925, 3926, 3927, 3928, 3929],
        14 => &[4732, 4733, 4734, 4730],
        15 => &[4747, 4748, 4749, 4746],
        16 => &[4751, 4752, 4753, 4750],
        17 => &[4755, 4756, 4757, 4754],
        18 => &[5583, 5584, 5585, 5586, 5587],
        19 => &[5683, 5684, 5685, 5686],
        _ => &[6137, 6138, 6139, 6140, 6141],
    };
    for &p in pieces {
        quick_spawn(rng, out, p, 1);
    }
    // `:7957` / `:7964` — the two sets' torch tails.
    match set {
        10 => quick_spawn(rng, out, 3024, 4),
        11 => quick_spawn(rng, out, 3599, 4),
        _ => {}
    }
}

/// The bag money tail's boss map (`Player.cs:7744-7828`): bag id →
/// the fresh `NPC.SetDefaults(num9)` type. `None` = `num9 <= 0`, the
/// `:7829-7832` early return (no money draws at all).
fn bag_money_boss(bag: u16) -> Option<i32> {
    Some(match bag {
        3318 => 50,
        3319 => 4,
        3320 => 13,
        3321 => 266,
        3322 => 222,
        3323 => 35,
        3324 => 113,
        3325 => 134,
        3326 => 125,
        3327 => 127,
        3328 => 262,
        3329 => 245,
        3330 => 370,
        3331 => 439,
        3332 => 398,
        3860 => 551,
        3861 => 576,
        3862 => 564,
        4782 => 636,
        4957 => 657,
        5111 => 668,
        _ => return None,
    })
}

/// The CLASSIC `NPC.SetDefaults` base values the money tail reads
/// (the census table; `Item.buyPrice(0, g)` = `g * 10000`,
/// `Item.cs:49417-49420`). The expert/master money curve rides
/// [`BagEnv::value_multiplier`] (the e25 `ScaleStats` seam).
fn boss_base_value(boss: i32) -> f32 {
    match boss {
        50 => 10000.0,    // NPC.cs:9400
        4 => 30000.0,     // :8717
        13 => 800.0,      // :8919 (the EoW HEAD — the bag's num9)
        266 => 50000.0,   // :12453
        222 => 50000.0,   // :11869
        35 => 50000.0,    // :9178
        113 => 80000.0,   // :10359
        134 => 120000.0,  // :10674
        125 => 120000.0,  // :10536
        127 => 120000.0,  // :10568
        262 => 150000.0,  // :12391 buyPrice(0, 15)
        245 => 150000.0,  // :12174 buyPrice(0, 15)
        370 => 250000.0,  // :13715
        439 => 100000.0,  // :14226
        398 => 1000000.0, // :14130
        551 => 0.0,       // :16528
        576 => 0.0,       // :16286
        564 => 0.0,       // :16456
        636 => 250000.0,  // :17200
        657 => 60000.0,   // :17322
        668 => 50000.0,   // :17463 buyPrice(0, 5)
        _ => 0.0,
    }
}

/// The bag money tail (`Player.cs:7833-7882`): the fresh-`SetDefaults`
/// value × the difficulty seam (the `(int)` truncation of
/// `ScaleStats_ByDifficulty`, `NPC.cs:18176`), the ±20 % spread, the
/// 1/5, 1/10, 1/15, 1/20 bonus chains, then the platinum/gold/silver/
/// copper split — NO >50 shrink rolls here (unlike
/// `NPCLoot_DropMoney`), the `(int)` truncations and the copper
/// `< 1 → 1` floor verbatim. Every coin rides `QuickSpawnItem` (coins
/// take no prefix, so no extra draws).
fn bag_money<R: PrefixRandom + ?Sized>(
    rng: &mut R,
    out: &mut Vec<BagGrant>,
    boss: i32,
    value_multiplier: f32,
) {
    // `:7835` — the fresh NPC's value, difficulty-scaled + truncated.
    let base = (boss_base_value(boss) * value_multiplier) as i32 as f32;
    let mut num10 = base * (1.0 + range(rng, -20, 21) as f32 * 0.01);
    // `:7836-7851` — the four bonus chains.
    if rng.next_max(5) == 0 {
        num10 *= 1.0 + range(rng, 5, 11) as f32 * 0.01;
    }
    if rng.next_max(10) == 0 {
        num10 *= 1.0 + range(rng, 10, 21) as f32 * 0.01;
    }
    if rng.next_max(15) == 0 {
        num10 *= 1.0 + range(rng, 15, 31) as f32 * 0.01;
    }
    if rng.next_max(20) == 0 {
        num10 *= 1.0 + range(rng, 20, 41) as f32 * 0.01;
    }
    // `:7852-7882` — the coin split.
    while num10 as i32 > 0 {
        if num10 > 1000000.0 {
            let num11 = (num10 / 1000000.0) as i32;
            num10 -= (1000000 * num11) as f32;
            quick_spawn(rng, out, 74, num11);
            continue;
        }
        if num10 > 10000.0 {
            let num12 = (num10 / 10000.0) as i32;
            num10 -= (10000 * num12) as f32;
            quick_spawn(rng, out, 73, num12);
            continue;
        }
        if num10 > 100.0 {
            let num13 = (num10 / 100.0) as i32;
            num10 -= (100 * num13) as f32;
            quick_spawn(rng, out, 72, num13);
            continue;
        }
        let mut num14 = num10 as i32;
        if num14 < 1 {
            num14 = 1; // `:7876-7879`
        }
        num10 -= num14 as f32;
        quick_spawn(rng, out, 71, num14);
    }
}

/// `Player.OpenBossBag(int type)` (`Player.cs:7202-7883`): the bag's
/// grant switch (the C# keeps two sequential switches — `:7206-7674`
/// and `:7675-7743`; they are disjoint per type, so one match keeps
/// the same per-bag order), then the money tail. The grants push in
/// `QuickSpawnItem` order — the `GetOrDropItem` fill order.
pub fn open_boss_bag<R: PrefixRandom + ?Sized>(
    bag: u16,
    env: &BagEnv,
    rng: &mut R,
    out: &mut Vec<BagGrant>,
) {
    match bag {
        // King Slime (`Player.cs:7208-7240`).
        3318 => {
            if rng.next_max(2) == 0 {
                quick_spawn(rng, out, 2430, 1); // Slimy Saddle
            }
            if rng.next_max(7) == 0 {
                quick_spawn(rng, out, 2493, 1); // King Slime Mask
            }
            // `:7218-7224` — two DISTINCT slime-crown pieces: the
            // do-while re-picks the SECOND until it differs.
            let num3 = range(rng, 256, 259);
            let mut num4 = range(rng, 256, 259);
            while num4 == num3 {
                num4 = range(rng, 256, 259);
            }
            quick_spawn(rng, out, num3, 1);
            quick_spawn(rng, out, num4, 1);
            // `:7225-7232` — the hook/gun either-or.
            if rng.next_max(2) == 0 {
                quick_spawn(rng, out, 2610, 1); // Slime Gun
            } else {
                quick_spawn(rng, out, 2585, 1); // Slime Hook
            }
            quick_spawn(rng, out, 998, 1); // Solidifier
            if rng.next_max(30) == 0 {
                quick_spawn(rng, out, 1309, 1); // Slime Staff
            }
            quick_spawn(rng, out, 3090, 1); // Royal Gel
        }
        // Eye of Cthulhu (`Player.cs:7241-7254`).
        3319 => {
            if rng.next_max(7) == 0 {
                quick_spawn(rng, out, 2112, 1); // Eye Mask
            }
            if rng.next_max(30) == 0 {
                quick_spawn(rng, out, 1299, 1); // Binoculars
            }
            // `:7250` — the ore stack is THREE Next(21)+10 draws
            // summed (the argument-evaluation order: left to right).
            let ore = if env.crimson { 880 } else { 56 };
            let stack = range(rng, 0, 21) + 10
                + (range(rng, 0, 21) + 10)
                + (range(rng, 0, 21) + 10);
            quick_spawn(rng, out, ore, stack);
            // `:7251` — the bars: Next(3)+1.
            let bars = if env.crimson { 2171 } else { 59 };
            quick_spawn(rng, out, bars, range(rng, 0, 3) + 1);
            // `:7252` — the Unholy Arrows: Next(31)+20.
            quick_spawn(rng, out, 47, range(rng, 0, 31) + 20);
            quick_spawn(rng, out, 3097, 1); // Shield of Cthulhu
        }
        // Eater of Worlds (`Player.cs:7255-7284`).
        3320 => {
            // `:7257` — the DISCARDED Next(15, 30): the C# draws num5
            // and never reads it (the stream still advances).
            let _num5 = range(rng, 15, 30);
            // `:7258-7265` — the ore stack: master 110..135, else 80..110.
            if env.master_mode {
                quick_spawn(rng, out, 56, range(rng, 110, 136));
            } else {
                quick_spawn(rng, out, 56, range(rng, 80, 111));
            }
            // `:7266-7273` — the Shadow Scales: master 30..50, else 20..40.
            if env.master_mode {
                quick_spawn(rng, out, 86, range(rng, 30, 51));
            } else {
                quick_spawn(rng, out, 86, range(rng, 20, 41));
            }
            if rng.next_max(20) == 0 {
                quick_spawn(rng, out, 994, 1); // Eater's Bone
            }
            if rng.next_max(7) == 0 {
                quick_spawn(rng, out, 2111, 1); // Eater Mask
            }
            quick_spawn(rng, out, 3224, 1); // Worm Scarf (expert-exclusive)
        }
        // Brain of Cthulhu (`Player.cs:7285-7314`).
        3321 => {
            // `:7287` — the DISCARDED Next(20, 46) (num6, never read).
            let _num6 = range(rng, 20, 46);
            // `:7288-7295` — the Crimtane Ore: master 110..135, else 80..110.
            if env.master_mode {
                quick_spawn(rng, out, 880, range(rng, 110, 136));
            } else {
                quick_spawn(rng, out, 880, range(rng, 80, 111));
            }
            // `:7296-7303` — the Tissue Samples: master 30..50, else 20..40.
            if env.master_mode {
                quick_spawn(rng, out, 1329, range(rng, 30, 51));
            } else {
                quick_spawn(rng, out, 1329, range(rng, 20, 41));
            }
            if rng.next_max(7) == 0 {
                quick_spawn(rng, out, 2104, 1); // Brain Mask
            }
            if rng.next_max(20) == 0 {
                quick_spawn(rng, out, 3060, 1); // Bone Rattle
            }
            quick_spawn(rng, out, 3223, 1); // Brain of Confusion (expert-exclusive)
        }
        // Queen Bee (`Player.cs:7315-7357`).
        3322 => {
            if rng.next_max(7) == 0 {
                quick_spawn(rng, out, 2108, 1); // Bee Mask
            }
            // `:7321-7334` — the weapon pick: Next(3) → 1121/1123/2888
            // (the Bee Gun / Bee Keeper / Blowgun).
            let num7 = match rng.next_max(3) {
                0 => 1121,
                1 => 1123,
                _ => 2888,
            };
            quick_spawn(rng, out, num7, 1);
            quick_spawn(rng, out, 3333, 1); // Hive Backpack (expert-exclusive)
            if rng.next_max(3) == 0 {
                quick_spawn(rng, out, 1132, 1); // Honey Comb
            }
            if rng.next_max(9) == 0 {
                quick_spawn(rng, out, 1170, 1); // Nectar
            }
            if rng.next_max(9) == 0 {
                quick_spawn(rng, out, 2502, 1); // Honeyed Goggles
            }
            if rng.next_max(9) == 0 {
                quick_spawn(rng, out, 5483, 1); // Bee Hat… the 5483 row
            }
            quick_spawn(rng, out, 1129, 1); // Hive Wand
            // `:7353` — the bottle pick: Next(842, 845).
            quick_spawn(rng, out, range(rng, 842, 845), 1);
            // `:7354-7355` — the Beenades Next(10, 31), the Wax Next(17, 31).
            quick_spawn(rng, out, 1130, range(rng, 10, 31));
            quick_spawn(rng, out, 2431, range(rng, 17, 31));
        }
        // Skeletron (`Player.cs:7358-7372`).
        3323 => {
            quick_spawn(rng, out, 3245, 1); // Bone Glove (expert-exclusive)
            // `:7360-7371` — the weapon pick: Next(3) → 1281/1273/1313
            // (the Skeletron Mask / Hand / Book of Skulls).
            match rng.next_max(3) {
                0 => quick_spawn(rng, out, 1281, 1),
                1 => quick_spawn(rng, out, 1273, 1),
                _ => quick_spawn(rng, out, 1313, 1),
            }
        }
        // Wall of Flesh (`Player.cs:7373-7402`).
        3324 => {
            if rng.next_max(7) == 0 {
                quick_spawn(rng, out, 2105, 1); // Flesh Mask
            }
            quick_spawn(rng, out, 367, 1); // Pwnhammer
            // `:7380-7383` — the Demon Heart ONLY without the extra
            // accessory slot already granted (`!this.extraAccessory`).
            if !env.extra_accessory {
                quick_spawn(rng, out, 3335, 1); // Demon Heart (expert-exclusive)
            }
            // `:7384-7385` — the emblem: Next(4), 3 → the Summoner
            // Emblem 2998, else 489 + num8 (Sorcerer/Warrior/Ranger).
            let num8 = rng.next_max(4);
            quick_spawn(rng, out, if num8 != 3 { 489 + num8 } else { 2998 }, 1);
            // `:7386-7400` — the weapon: Next(4) → 514/426/434/4912
            // (Laser Rifle / Breaker Blade / Clockwork Assault Rifle /
            // Fire Whip… the 4912 row).
            match rng.next_max(4) {
                0 => quick_spawn(rng, out, 514, 1),
                1 => quick_spawn(rng, out, 426, 1),
                2 => quick_spawn(rng, out, 434, 1),
                _ => quick_spawn(rng, out, 4912, 1),
            }
        }
        // The Destroyer (`Player.cs:7403-7412`).
        3325 => {
            try_getting_dev_armor(rng, out, env.tenth_anniversary);
            if rng.next_max(7) == 0 {
                quick_spawn(rng, out, 2113, 1); // Destroyer Mask
            }
            // `:7409-7410` — Souls of Might Next(25, 41), Hallowed Bars
            // Next(20, 36).
            quick_spawn(rng, out, 548, range(rng, 25, 41));
            quick_spawn(rng, out, 1225, range(rng, 20, 36));
            quick_spawn(rng, out, 3355, 1); // Mechanical Wagon Piece
        }
        // The Twins (`Player.cs:7413-7422`).
        3326 => {
            try_getting_dev_armor(rng, out, env.tenth_anniversary);
            if rng.next_max(7) == 0 {
                quick_spawn(rng, out, 2106, 1); // Twin Mask
            }
            // `:7419-7420` — Souls of Sight Next(25, 41), Hallowed Bars
            // Next(20, 36).
            quick_spawn(rng, out, 549, range(rng, 25, 41));
            quick_spawn(rng, out, 1225, range(rng, 20, 36));
            quick_spawn(rng, out, 3354, 1); // Mechanical Wheel Piece
        }
        // Skeletron Prime (`Player.cs:7423-7432`).
        3327 => {
            try_getting_dev_armor(rng, out, env.tenth_anniversary);
            if rng.next_max(7) == 0 {
                quick_spawn(rng, out, 2107, 1); // Skeletron Prime Mask
            }
            // `:7429-7430` — Souls of Fright Next(25, 41), Hallowed Bars
            // Next(20, 36).
            quick_spawn(rng, out, 547, range(rng, 25, 41));
            quick_spawn(rng, out, 1225, range(rng, 20, 36));
            quick_spawn(rng, out, 3356, 1); // Mechanical Battery Piece
        }
        _ => {}
    }
    // The money tail (`:7744-7882`) runs for EVERY bag id in the map —
    // including the grant-less Ogre/Dark Mage bags (3861/3862).
    if let Some(boss) = bag_money_boss(bag) {
        bag_money(rng, out, boss, env.value_multiplier);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::VecDeque;
    use tr_core::UnifiedRandom;

    /// A scripted rng: forced draws pop off the script (each asserted
    /// inside its requested range); once the script runs out every
    /// draw returns the range's minimum (the neutral pass-through —
    /// every chance roll hits, every stack roll is the minimum). The
    /// `loot_db.rs` ScriptRng shape; records every requested range.
    struct ScriptRng {
        draws: VecDeque<i32>,
        ranges: Vec<(i32, i32)>,
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
            self.ranges.push((0, max));
            let v = self.draws.pop_front().unwrap_or(0);
            assert!((0..max).contains(&v), "draw {v} outside [0, {max})");
            v
        }
    }

    /// The grant items in order (the prefix is pinned separately).
    fn items(out: &[BagGrant]) -> Vec<u16> {
        out.iter().map(|g| g.item).collect()
    }

    /// The shared money tail's draw census for one bag: replay the
    /// C# expression order on a twin `UnifiedRandom` and compare the
    /// coin grants + the final stream state.
    fn money_replay(seed: i32, boss: i32, mult: f32) -> (Vec<BagGrant>, UnifiedRandom) {
        let mut r = UnifiedRandom::new(seed);
        let mut out = Vec::new();
        money_replay_on(&mut r, boss, mult, &mut out);
        (out, r)
    }

    /// The money-tail replay continuing on an ALREADY-ADVANCED stream
    /// (the per-boss full replays: the grant arm drew first).
    fn money_replay_on(r: &mut UnifiedRandom, boss: i32, mult: f32, out: &mut Vec<BagGrant>) {
        let base = (boss_base_value(boss) * mult) as i32 as f32;
        let mut num10 = base * (1.0 + r.next_range(-20, 21) as f32 * 0.01);
        if r.next_max(5) == 0 {
            num10 *= 1.0 + r.next_range(5, 11) as f32 * 0.01;
        }
        if r.next_max(10) == 0 {
            num10 *= 1.0 + r.next_range(10, 21) as f32 * 0.01;
        }
        if r.next_max(15) == 0 {
            num10 *= 1.0 + r.next_range(15, 31) as f32 * 0.01;
        }
        if r.next_max(20) == 0 {
            num10 *= 1.0 + r.next_range(20, 41) as f32 * 0.01;
        }
        while num10 as i32 > 0 {
            let (id, count) = if num10 > 1000000.0 {
                let n = (num10 / 1000000.0) as i32;
                num10 -= (1000000 * n) as f32;
                (74u16, n)
            } else if num10 > 10000.0 {
                let n = (num10 / 10000.0) as i32;
                num10 -= (10000 * n) as f32;
                (73, n)
            } else if num10 > 100.0 {
                let n = (num10 / 100.0) as i32;
                num10 -= (100 * n) as f32;
                (72, n)
            } else {
                let n = (num10 as i32).max(1);
                num10 -= n as f32;
                (71, n)
            };
            // Coins take no prefix (`CanHavePrefixes` false) — no draw.
            out.push(BagGrant {
                item: id,
                stack: count as u32,
                prefix: 0,
            });
        }
    }

    /// The C# `QuickSpawnItem` replay helper for the per-boss twins.
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

    /// The `TryGettingDevArmor` replay (`Player.cs:7885-8026`) for the
    /// mech-bag twins — the gate + the set pick + the pieces + the two
    /// torch tails, in the C# order.
    fn dev_armor_replay(r: &mut UnifiedRandom, want: &mut Vec<BagGrant>, tenth: bool) {
        if r.next_max(if tenth { 8 } else { 16 }) != 0 {
            return;
        }
        let set = r.next_max(21);
        let pieces: &[i32] = match set {
            0 => &[666, 667, 668, 665, 3287],
            1 => &[1554, 1555, 1556, 1586],
            2 => &[1554, 1587, 1588, 1586],
            3 => &[1557, 1558, 1559, 1585],
            4 => &[1560, 1561, 1562, 1584],
            5 => &[1563, 1564, 1565, 3582],
            6 => &[1566, 1567, 1568],
            7 => &[1580, 1581, 1582, 1583],
            8 => &[3226, 3227, 3228, 3288],
            9 => &[3583, 3581, 3578, 3579, 3580],
            10 => &[3585, 3586, 3587, 3588],
            11 => &[3589, 3590, 3591, 3592],
            12 => &[3368, 3921, 3922, 3923, 3924],
            13 => &[3925, 3926, 3927, 3928, 3929],
            14 => &[4732, 4733, 4734, 4730],
            15 => &[4747, 4748, 4749, 4746],
            16 => &[4751, 4752, 4753, 4750],
            17 => &[4755, 4756, 4757, 4754],
            18 => &[5583, 5584, 5585, 5586, 5587],
            19 => &[5683, 5684, 5685, 5686],
            _ => &[6137, 6138, 6139, 6140, 6141],
        };
        for &p in pieces {
            spawn_replay(r, want, p, 1);
        }
        match set {
            10 => spawn_replay(r, want, 3024, 4),
            11 => spawn_replay(r, want, 3599, 4),
            _ => {}
        }
    }

    #[test]
    fn w4_f35_the_boss_bag_set_is_the_21_id_table() {
        // `ItemID.cs:1383` — 3318-3332 (15), 3860/3861/3862, 4782,
        // 4957, 5111. The grant-less Ogre/Dark Mage bags ARE in it.
        for k in 3318..=3332u16 {
            assert!(is_boss_bag(k), "{k}");
        }
        for k in [3860u16, 3861, 3862, 4782, 4957, 5111] {
            assert!(is_boss_bag(k), "{k}");
        }
        for k in [0u16, 3317, 3333, 3859, 3863, 4781, 4956, 5110, 5112] {
            assert!(!is_boss_bag(k), "{k} not a bag");
        }
    }

    #[test]
    fn w4_f35_quick_spawn_gates_the_stack_and_rolls_the_prefix() {
        // `Player.cs:7172` — stack <= 0 grants nothing, draws nothing.
        let mut rng = UnifiedRandom::new(7);
        let mut out = Vec::new();
        quick_spawn(&mut rng, &mut out, 2430, 0);
        quick_spawn(&mut rng, &mut out, 2430, -3);
        assert!(out.is_empty());
        assert_eq!(rng, UnifiedRandom::new(7), "no draws on the gate");
        // A non-prefixable kind (the Solidifier 998) grants with no
        // prefix draws (`Item.cs:515-518`'s CanHavePrefixes gate).
        quick_spawn(&mut rng, &mut out, 998, 1);
        assert_eq!(out, vec![BagGrant { item: 998, stack: 1, prefix: 0 }]);
        assert_eq!(rng, UnifiedRandom::new(7), "998 is not prefixable");
        // A prefixable kind (the Slime Staff 1309, a summon weapon)
        // rolls the natural prefix on the same stream.
        let mut twin = UnifiedRandom::new(7);
        let want = crate::prefix::roll_prefix(1309, -1, &mut twin);
        quick_spawn(&mut rng, &mut out, 1309, 1);
        assert_eq!(out[1].prefix, want);
        assert_eq!(rng, twin, "the prefix draws ride the same stream");
    }

    #[test]
    fn w4_f35_king_slime_bag_zero_script_grants_the_whole_arm() {
        // The neutral script (every roll hits its minimum) with ONE
        // forced 1 to break the piece do-while: saddle, mask, TWO
        // DISTINCT pieces (256 then a re-pick to 257), the Slime Gun,
        // the Solidifier, the Slime Staff, the Royal Gel — then money.
        // The saddle/mask/ninja-piece grants are not prefixable (no
        // draws); the staff/gel prefix draws fall back to the script's
        // 0 and only affect prefix values, never the item order.
        let mut rng = ScriptRng::new(&[0, 0, 0, 0, 1, 0, 0, 0]);
        let mut out = Vec::new();
        open_boss_bag(3318, &BagEnv::default(), &mut rng, &mut out);
        let its = items(&out);
        assert_eq!(
            its[..8],
            [2430, 2493, 256, 257, 2610, 998, 1309, 3090],
            "the grant arm: {its:?}"
        );
        // The re-pick fired once: THREE Next(3) piece draws (num3=256,
        // num4=256, re-pick=257 breaks the loop).
        let piece_draws: Vec<_> = rng.ranges.iter().filter(|r| **r == (0, 3)).collect();
        assert_eq!(piece_draws.len(), 3, "the do-while re-picked once");
        // The money tail follows (the all-zero script hits every bonus
        // gate — the exact coin math is pinned by the replay tests);
        // here only that the remainder is coins.
        assert!(its[8..].iter().all(|i| (71..=74).contains(i)), "coins: {its:?}");
    }

    #[test]
    fn w4_f35_king_slime_bag_failed_rolls_skip_the_optionals() {
        // Forced misses: saddle 1/2 fails (1), mask 1/7 fails (1),
        // pieces 256/258 (distinct, no re-pick), hook/gun else arm
        // (1 → the Slime Hook 2585), staff 1/30 fails (1).
        let mut rng = ScriptRng::new(&[1, 1, 0, 2, 1, 1]);
        let mut out = Vec::new();
        open_boss_bag(3318, &BagEnv::default(), &mut rng, &mut out);
        let its = items(&out);
        assert_eq!(its[..5], [256, 258, 2585, 998, 3090], "{its:?}");
        // Exactly 6 bag draws before the money tail (no re-pick).
        assert_eq!(rng.ranges[..6], [(0, 2), (0, 7), (0, 3), (0, 3), (0, 2), (0, 30)]);
    }

    #[test]
    fn w4_f35_king_slime_bag_replays_the_csharp_draw_order() {
        // The same-seed twin: the bag arm's rolls replayed in the C#
        // expression order on a raw UnifiedRandom — the grants AND the
        // final stream state match (the prefix draws interleaved).
        for seed in [1i32, 42, 20261121] {
            let mut got_rng = UnifiedRandom::new(seed);
            let mut got = Vec::new();
            open_boss_bag(3318, &BagEnv::default(), &mut got_rng, &mut got);

            let mut r = UnifiedRandom::new(seed);
            let mut want: Vec<BagGrant> = Vec::new();
            if r.next_max(2) == 0 {
                spawn_replay(&mut r, &mut want, 2430, 1);
            }
            if r.next_max(7) == 0 {
                spawn_replay(&mut r, &mut want, 2493, 1);
            }
            let num3 = r.next_range(256, 259);
            let mut num4 = r.next_range(256, 259);
            while num4 == num3 {
                num4 = r.next_range(256, 259);
            }
            spawn_replay(&mut r, &mut want, num3, 1);
            spawn_replay(&mut r, &mut want, num4, 1);
            if r.next_max(2) == 0 {
                spawn_replay(&mut r, &mut want, 2610, 1);
            } else {
                spawn_replay(&mut r, &mut want, 2585, 1);
            }
            spawn_replay(&mut r, &mut want, 998, 1);
            if r.next_max(30) == 0 {
                spawn_replay(&mut r, &mut want, 1309, 1);
            }
            spawn_replay(&mut r, &mut want, 3090, 1);
            money_replay_on(&mut r, 50, 1.0, &mut want);
            assert_eq!(got, want, "seed {seed}: the grants + prefixes");
            assert_eq!(got_rng, r, "seed {seed}: the stream state");
        }
    }

    #[test]
    fn w4_f35_eoc_bag_zero_script_grants_the_whole_arm() {
        // Corruption (crimson=false): the ore 56, the bars 59. The
        // zero script: mask 1/7 hits, binoculars 1/30 hits, the three
        // ore draws all 10 (stack 30), bars 1, arrows 20, the shield.
        let mut rng = ScriptRng::new(&[]);
        let mut out = Vec::new();
        open_boss_bag(3319, &BagEnv::default(), &mut rng, &mut out);
        let its = items(&out);
        assert_eq!(its[..6], [2112, 1299, 56, 59, 47, 3097], "{its:?}");
        assert_eq!(out[2].stack, 30, "three Next(21)+10 draws summed");
        assert_eq!(out[3].stack, 1, "Next(3)+1 minimum");
        assert_eq!(out[4].stack, 20, "Next(31)+20 minimum");
        // The ore stack's THREE draws are all Next(21) — the census.
        let ore_draws: Vec<_> = rng.ranges.iter().filter(|r| **r == (0, 21)).collect();
        assert_eq!(ore_draws.len(), 3, "the triple-draw ore stack");
        // The money tail follows (the all-zero script hits every bonus
        // gate — the exact coin math is pinned by the replay tests).
        assert!(its[6..].iter().all(|i| (71..=74).contains(i)), "coins: {its:?}");
    }

    #[test]
    fn w4_f35_eoc_bag_crimson_swaps_the_ore_and_bars() {
        // `:7250-7251` — the crimson world grants Crimtane Ore 880 +
        // Crimtane Bars 2171 (the corruption pair is 56/59).
        let env = BagEnv { crimson: true, ..Default::default() };
        let mut rng = ScriptRng::new(&[6, 29]); // mask + binoculars miss
        let mut out = Vec::new();
        open_boss_bag(3319, &env, &mut rng, &mut out);
        let its = items(&out);
        assert_eq!(its[..4], [880, 2171, 47, 3097], "{its:?}");
    }

    #[test]
    fn w4_f35_eoc_bag_replays_the_csharp_draw_order() {
        for seed in [5i32, 99] {
            let mut got_rng = UnifiedRandom::new(seed);
            let mut got = Vec::new();
            open_boss_bag(3319, &BagEnv::default(), &mut got_rng, &mut got);
            let mut r = UnifiedRandom::new(seed);
            let mut want: Vec<BagGrant> = Vec::new();
            if r.next_max(7) == 0 {
                spawn_replay(&mut r, &mut want, 2112, 1);
            }
            if r.next_max(30) == 0 {
                spawn_replay(&mut r, &mut want, 1299, 1);
            }
            let stack = r.next_range(0, 21) + 10 + (r.next_range(0, 21) + 10) + (r.next_range(0, 21) + 10);
            spawn_replay(&mut r, &mut want, 56, stack);
            spawn_replay(&mut r, &mut want, 59, r.next_range(0, 3) + 1);
            spawn_replay(&mut r, &mut want, 47, r.next_range(0, 31) + 20);
            spawn_replay(&mut r, &mut want, 3097, 1);
            money_replay_on(&mut r, 4, 1.0, &mut want);
            assert_eq!(got, want, "seed {seed}: the grants + prefixes");
            assert_eq!(got_rng, r, "seed {seed}: the stream state");
        }
    }

    #[test]
    fn w4_f35_eow_bag_zero_script_grants_the_whole_arm() {
        // The zero script: the discarded num5 draws Next(15,30) first
        // (the stream advances though the value is never read), ore
        // 80, scales 20, the bone 1/20 hits, the mask 1/7 hits, the
        // Worm Scarf — then the money tail (the all-zero script hits
        // every bonus gate; the exact coin math is the replay's pin).
        let mut rng = ScriptRng::new(&[]);
        let mut out = Vec::new();
        open_boss_bag(3320, &BagEnv::default(), &mut rng, &mut out);
        let its = items(&out);
        assert_eq!(its[..5], [56, 86, 994, 2111, 3224], "{its:?}");
        assert_eq!(out[0].stack, 80, "the classic ore range minimum");
        assert_eq!(out[1].stack, 20, "the classic scales minimum");
        assert!(its[5..].iter().all(|i| (71..=74).contains(i)), "coins: {its:?}");
        // The FIRST draw is the discarded Next(15, 30) — the census.
        assert_eq!(rng.ranges[0], (0, 15), "num5 Next(15,30) span 15");
    }

    #[test]
    fn w4_f35_eow_bag_master_mode_bumps_the_ore_and_scales() {
        // `:7258-7273` — master: ore Next(110,136), scales Next(30,51).
        let env = BagEnv { master_mode: true, ..Default::default() };
        let mut rng = ScriptRng::new(&[0, 0, 0, 19, 6]); // bone + mask miss
        let mut out = Vec::new();
        open_boss_bag(3320, &env, &mut rng, &mut out);
        let its = items(&out);
        assert_eq!(its[..3], [56, 86, 3224], "{its:?}");
        assert_eq!(out[0].stack, 110, "the master ore minimum");
        assert_eq!(out[1].stack, 30, "the master scales minimum");
    }

    #[test]
    fn w4_f35_eow_bag_replays_the_csharp_draw_order_both_modes() {
        for master in [false, true] {
            for seed in [11i32, 2024] {
                let env = BagEnv { master_mode: master, ..Default::default() };
                let mut got_rng = UnifiedRandom::new(seed);
                let mut got = Vec::new();
                open_boss_bag(3320, &env, &mut got_rng, &mut got);
                let mut r = UnifiedRandom::new(seed);
                let mut want: Vec<BagGrant> = Vec::new();
                let _num5 = r.next_range(15, 30); // the discarded draw
                if master {
                    spawn_replay(&mut r, &mut want, 56, r.next_range(110, 136));
                    spawn_replay(&mut r, &mut want, 86, r.next_range(30, 51));
                } else {
                    spawn_replay(&mut r, &mut want, 56, r.next_range(80, 111));
                    spawn_replay(&mut r, &mut want, 86, r.next_range(20, 41));
                }
                if r.next_max(20) == 0 {
                    spawn_replay(&mut r, &mut want, 994, 1);
                }
                if r.next_max(7) == 0 {
                    spawn_replay(&mut r, &mut want, 2111, 1);
                }
                spawn_replay(&mut r, &mut want, 3224, 1);
                money_replay_on(&mut r, 13, 1.0, &mut want);
                assert_eq!(got, want, "master {master} seed {seed}");
                assert_eq!(got_rng, r, "master {master} seed {seed} stream");
            }
        }
    }

    #[test]
    fn w4_f35_boc_bag_zero_script_grants_the_whole_arm() {
        // The zero script: the discarded num6 Next(20,46) first, ore
        // 880×80, samples 1329×20, the mask 1/7 + rattle 1/20 hit, the
        // Brain of Confusion — then coins.
        let mut rng = ScriptRng::new(&[]);
        let mut out = Vec::new();
        open_boss_bag(3321, &BagEnv::default(), &mut rng, &mut out);
        let its = items(&out);
        assert_eq!(its[..5], [880, 1329, 2104, 3060, 3223], "{its:?}");
        assert_eq!(out[0].stack, 80);
        assert_eq!(out[1].stack, 20);
        assert!(its[5..].iter().all(|i| (71..=74).contains(i)), "coins: {its:?}");
        // The FIRST draw is the discarded Next(20, 46) — span 26.
        assert_eq!(rng.ranges[0], (0, 26), "num6 Next(20,46) span 26");
    }

    #[test]
    fn w4_f35_boc_bag_master_mode_bumps_the_ore_and_samples() {
        let env = BagEnv { master_mode: true, ..Default::default() };
        let mut rng = ScriptRng::new(&[0, 0, 0, 6, 19]); // mask + rattle miss
        let mut out = Vec::new();
        open_boss_bag(3321, &env, &mut rng, &mut out);
        let its = items(&out);
        assert_eq!(its[..3], [880, 1329, 3223], "{its:?}");
        assert_eq!(out[0].stack, 110);
        assert_eq!(out[1].stack, 30);
    }

    #[test]
    fn w4_f35_boc_bag_replays_the_csharp_draw_order_both_modes() {
        for master in [false, true] {
            for seed in [13i32, 555] {
                let env = BagEnv { master_mode: master, ..Default::default() };
                let mut got_rng = UnifiedRandom::new(seed);
                let mut got = Vec::new();
                open_boss_bag(3321, &env, &mut got_rng, &mut got);
                let mut r = UnifiedRandom::new(seed);
                let mut want: Vec<BagGrant> = Vec::new();
                let _num6 = r.next_range(20, 46);
                if master {
                    spawn_replay(&mut r, &mut want, 880, r.next_range(110, 136));
                    spawn_replay(&mut r, &mut want, 1329, r.next_range(30, 51));
                } else {
                    spawn_replay(&mut r, &mut want, 880, r.next_range(80, 111));
                    spawn_replay(&mut r, &mut want, 1329, r.next_range(20, 41));
                }
                if r.next_max(7) == 0 {
                    spawn_replay(&mut r, &mut want, 2104, 1);
                }
                if r.next_max(20) == 0 {
                    spawn_replay(&mut r, &mut want, 3060, 1);
                }
                spawn_replay(&mut r, &mut want, 3223, 1);
                money_replay_on(&mut r, 266, 1.0, &mut want);
                assert_eq!(got, want, "master {master} seed {seed}");
                assert_eq!(got_rng, r, "master {master} seed {seed} stream");
            }
        }
    }

    #[test]
    fn w4_f35_queen_bee_bag_zero_script_grants_the_whole_arm() {
        // The zero script: mask hits, weapon pick 0 → the Bee Gun 1121,
        // the Hive Backpack, all four 1/3+1/9×3 optionals hit, the Hive
        // Wand, the bottle pick 842, Beenades 10, Wax 17 — then coins.
        let mut rng = ScriptRng::new(&[]);
        let mut out = Vec::new();
        open_boss_bag(3322, &BagEnv::default(), &mut rng, &mut out);
        let its = items(&out);
        assert_eq!(
            its[..10],
            [2108, 1121, 3333, 1132, 1170, 2502, 5483, 1129, 842, 1130],
            "{its:?}"
        );
        assert_eq!(its[10], 2431);
        assert_eq!(out[9].stack, 10, "Beenades Next(10,31) minimum");
        assert_eq!(out[10].stack, 17, "Wax Next(17,31) minimum");
        assert!(its[11..].iter().all(|i| (71..=74).contains(i)), "coins: {its:?}");
    }

    #[test]
    fn w4_f35_queen_bee_bag_weapon_and_bottle_picks() {
        // The weapon Next(3): 1 → the Bee Keeper 1123, 2 → the Blowgun
        // 2888; the bottle Next(842,845): 2 → 844.
        for (pick, weapon) in [(1i32, 1123u16), (2, 2888)] {
            let mut rng = ScriptRng::new(&[6, pick, 2, 8, 8, 8, 2]);
            let mut out = Vec::new();
            open_boss_bag(3322, &BagEnv::default(), &mut rng, &mut out);
            let its = items(&out);
            assert_eq!(its[0], weapon, "pick {pick}");
            assert_eq!(its[1], 3333, "the backpack follows the weapon");
            assert_eq!(its[2], 1129, "the optionals all missed");
            assert_eq!(its[3], 844, "the bottle pick 2 → 844");
        }
    }

    #[test]
    fn w4_f35_queen_bee_bag_replays_the_csharp_draw_order() {
        for seed in [17i32, 808] {
            let mut got_rng = UnifiedRandom::new(seed);
            let mut got = Vec::new();
            open_boss_bag(3322, &BagEnv::default(), &mut got_rng, &mut got);
            let mut r = UnifiedRandom::new(seed);
            let mut want: Vec<BagGrant> = Vec::new();
            if r.next_max(7) == 0 {
                spawn_replay(&mut r, &mut want, 2108, 1);
            }
            let num7 = match r.next_max(3) {
                0 => 1121,
                1 => 1123,
                _ => 2888,
            };
            spawn_replay(&mut r, &mut want, num7, 1);
            spawn_replay(&mut r, &mut want, 3333, 1);
            if r.next_max(3) == 0 {
                spawn_replay(&mut r, &mut want, 1132, 1);
            }
            if r.next_max(9) == 0 {
                spawn_replay(&mut r, &mut want, 1170, 1);
            }
            if r.next_max(9) == 0 {
                spawn_replay(&mut r, &mut want, 2502, 1);
            }
            if r.next_max(9) == 0 {
                spawn_replay(&mut r, &mut want, 5483, 1);
            }
            spawn_replay(&mut r, &mut want, 1129, 1);
            spawn_replay(&mut r, &mut want, r.next_range(842, 845), 1);
            spawn_replay(&mut r, &mut want, 1130, r.next_range(10, 31));
            spawn_replay(&mut r, &mut want, 2431, r.next_range(17, 31));
            money_replay_on(&mut r, 222, 1.0, &mut want);
            assert_eq!(got, want, "seed {seed}");
            assert_eq!(got_rng, r, "seed {seed} stream");
        }
    }

    #[test]
    fn w4_f35_skeletron_bag_grants_the_glove_then_one_weapon() {
        // The Bone Glove ALWAYS, then exactly ONE of the three weapons
        // by Next(3) — then coins. The glove is an accessory (prefix
        // draws), the weapons vary; the item ORDER is the pin.
        for (pick, weapon) in [(0i32, 1281u16), (1, 1273), (2, 1313)] {
            let mut rng = ScriptRng::new(&[pick]);
            let mut out = Vec::new();
            open_boss_bag(3323, &BagEnv::default(), &mut rng, &mut out);
            let its = items(&out);
            assert_eq!(its[0], 3245, "the Bone Glove leads");
            assert_eq!(its[1], weapon, "pick {pick}");
            assert!(its[2..].iter().all(|i| (71..=74).contains(i)), "coins: {its:?}");
        }
    }

    #[test]
    fn w4_f35_skeletron_bag_replays_the_csharp_draw_order() {
        for seed in [19i32, 313] {
            let mut got_rng = UnifiedRandom::new(seed);
            let mut got = Vec::new();
            open_boss_bag(3323, &BagEnv::default(), &mut got_rng, &mut got);
            let mut r = UnifiedRandom::new(seed);
            let mut want: Vec<BagGrant> = Vec::new();
            spawn_replay(&mut r, &mut want, 3245, 1);
            match r.next_max(3) {
                0 => spawn_replay(&mut r, &mut want, 1281, 1),
                1 => spawn_replay(&mut r, &mut want, 1273, 1),
                _ => spawn_replay(&mut r, &mut want, 1313, 1),
            }
            money_replay_on(&mut r, 35, 1.0, &mut want);
            assert_eq!(got, want, "seed {seed}");
            assert_eq!(got_rng, r, "seed {seed} stream");
        }
    }

    #[test]
    fn w4_f35_wof_bag_zero_script_grants_the_whole_arm() {
        // The zero script: mask hits, the Pwnhammer, the Demon Heart
        // (no extra accessory), the emblem pick 0 → Sorcerer 489, the
        // weapon pick 0 → Laser Rifle 514 — then coins.
        let mut rng = ScriptRng::new(&[]);
        let mut out = Vec::new();
        open_boss_bag(3324, &BagEnv::default(), &mut rng, &mut out);
        let its = items(&out);
        assert_eq!(its[..5], [2105, 367, 3335, 489, 514], "{its:?}");
        assert!(its[5..].iter().all(|i| (71..=74).contains(i)), "coins: {its:?}");
    }

    #[test]
    fn w4_f35_wof_bag_demon heart_skips_with_the_extra_slot() {
        // `:7380` — extraAccessory=true (the Demon Heart already used)
        // skips the second heart; the emblem/weapon picks still run.
        let env = BagEnv { extra_accessory: true, ..Default::default() };
        let mut rng = ScriptRng::new(&[6, 3, 3]); // mask miss, emblem 3, weapon 3
        let mut out = Vec::new();
        open_boss_bag(3324, &env, &mut rng, &mut out);
        let its = items(&out);
        assert_eq!(its[..3], [367, 2998, 4912], "{its:?}");
        assert!(!its.contains(&3335), "no second Demon Heart");
    }

    #[test]
    fn w4_f35_wof_bag_emblem_and_weapon_picks() {
        // The emblem Next(4): 1 → Warrior 490, 2 → Ranger 491, 3 →
        // Summoner 2998; the weapon Next(4): 1 → Breaker Blade 426,
        // 2 → Clockwork Assault Rifle 434.
        for (e, w, emblem, weapon) in [
            (1i32, 1i32, 490u16, 426u16),
            (2, 2, 491, 434),
        ] {
            let mut rng = ScriptRng::new(&[6, e, w]);
            let mut out = Vec::new();
            open_boss_bag(3324, &BagEnv::default(), &mut rng, &mut out);
            let its = items(&out);
            assert_eq!(its[0], 367);
            assert_eq!(its[1], 3335);
            assert_eq!(its[2], emblem, "emblem {e}");
            assert_eq!(its[3], weapon, "weapon {w}");
        }
    }

    #[test]
    fn w4_f35_wof_bag_replays_the_csharp_draw_order_both_accessory_states() {
        for extra in [false, true] {
            for seed in [23i32, 113113] {
                let env = BagEnv { extra_accessory: extra, ..Default::default() };
                let mut got_rng = UnifiedRandom::new(seed);
                let mut got = Vec::new();
                open_boss_bag(3324, &env, &mut got_rng, &mut got);
                let mut r = UnifiedRandom::new(seed);
                let mut want: Vec<BagGrant> = Vec::new();
                if r.next_max(7) == 0 {
                    spawn_replay(&mut r, &mut want, 2105, 1);
                }
                spawn_replay(&mut r, &mut want, 367, 1);
                if !extra {
                    spawn_replay(&mut r, &mut want, 3335, 1);
                }
                let num8 = r.next_max(4);
                spawn_replay(&mut r, &mut want, if num8 != 3 { 489 + num8 } else { 2998 }, 1);
                match r.next_max(4) {
                    0 => spawn_replay(&mut r, &mut want, 514, 1),
                    1 => spawn_replay(&mut r, &mut want, 426, 1),
                    2 => spawn_replay(&mut r, &mut want, 434, 1),
                    _ => spawn_replay(&mut r, &mut want, 4912, 1),
                }
                money_replay_on(&mut r, 113, 1.0, &mut want);
                assert_eq!(got, want, "extra {extra} seed {seed}");
                assert_eq!(got_rng, r, "extra {extra} seed {seed} stream");
            }
        }
    }

    #[test]
    fn w4_f35_destroyer_bag_zero_script_grants_the_whole_arm() {
        // The zero script: the dev-armor gate HITS (0) and set 0 grants
        // the five Red pieces FIRST, then the mask 1/7, Souls of Might
        // 25, Hallowed Bars 20, the Mechanical Wagon Piece — then coins.
        let mut rng = ScriptRng::new(&[]);
        let mut out = Vec::new();
        open_boss_bag(3325, &BagEnv::default(), &mut rng, &mut out);
        let its = items(&out);
        assert_eq!(
            its[..9],
            [666, 667, 668, 665, 3287, 2113, 548, 1225, 3355],
            "the dev set leads: {its:?}"
        );
        assert_eq!(out[6].stack, 25, "Souls Next(25,41) minimum");
        assert_eq!(out[7].stack, 20, "Bars Next(20,36) minimum");
        assert!(its[9..].iter().all(|i| (71..=74).contains(i)), "coins: {its:?}");
    }

    #[test]
    fn w4_f35_destroyer_bag_dev_armor_miss_skips_the_set() {
        // The gate draw 1 misses: no dev set, the arm starts at the mask.
        let mut rng = ScriptRng::new(&[1, 6]); // dev miss, mask miss
        let mut out = Vec::new();
        open_boss_bag(3325, &BagEnv::default(), &mut rng, &mut out);
        let its = items(&out);
        assert_eq!(its[..3], [548, 1225, 3355], "{its:?}");
    }

    #[test]
    fn w4_f35_destroyer_bag_replays_the_csharp_draw_order() {
        for seed in [29i32, 134134] {
            let mut got_rng = UnifiedRandom::new(seed);
            let mut got = Vec::new();
            open_boss_bag(3325, &BagEnv::default(), &mut got_rng, &mut got);
            let mut r = UnifiedRandom::new(seed);
            let mut want: Vec<BagGrant> = Vec::new();
            dev_armor_replay(&mut r, &mut want, false);
            if r.next_max(7) == 0 {
                spawn_replay(&mut r, &mut want, 2113, 1);
            }
            spawn_replay(&mut r, &mut want, 548, r.next_range(25, 41));
            spawn_replay(&mut r, &mut want, 1225, r.next_range(20, 36));
            spawn_replay(&mut r, &mut want, 3355, 1);
            money_replay_on(&mut r, 134, 1.0, &mut want);
            assert_eq!(got, want, "seed {seed}");
            assert_eq!(got_rng, r, "seed {seed} stream");
        }
    }

    #[test]
    fn w4_f35_twins_bag_zero_script_grants_the_whole_arm() {
        // The dev set leads (gate 0, set 0), then the Twin Mask, Souls
        // of Sight 25, Hallowed Bars 20, the Mechanical Wheel Piece.
        let mut rng = ScriptRng::new(&[]);
        let mut out = Vec::new();
        open_boss_bag(3326, &BagEnv::default(), &mut rng, &mut out);
        let its = items(&out);
        assert_eq!(
            its[..9],
            [666, 667, 668, 665, 3287, 2106, 549, 1225, 3354],
            "{its:?}"
        );
        assert_eq!(out[6].stack, 25);
        assert_eq!(out[7].stack, 20);
        assert!(its[9..].iter().all(|i| (71..=74).contains(i)), "coins: {its:?}");
    }

    #[test]
    fn w4_f35_twins_bag_replays_the_csharp_draw_order() {
        for seed in [31i32, 125126] {
            let mut got_rng = UnifiedRandom::new(seed);
            let mut got = Vec::new();
            open_boss_bag(3326, &BagEnv::default(), &mut got_rng, &mut got);
            let mut r = UnifiedRandom::new(seed);
            let mut want: Vec<BagGrant> = Vec::new();
            dev_armor_replay(&mut r, &mut want, false);
            if r.next_max(7) == 0 {
                spawn_replay(&mut r, &mut want, 2106, 1);
            }
            spawn_replay(&mut r, &mut want, 549, r.next_range(25, 41));
            spawn_replay(&mut r, &mut want, 1225, r.next_range(20, 36));
            spawn_replay(&mut r, &mut want, 3354, 1);
            money_replay_on(&mut r, 125, 1.0, &mut want);
            assert_eq!(got, want, "seed {seed}");
            assert_eq!(got_rng, r, "seed {seed} stream");
        }
    }

    #[test]
    fn w4_f35_prime_bag_zero_script_grants_the_whole_arm() {
        // The dev set leads, then the Prime Mask, Souls of Fright 25,
        // Hallowed Bars 20, the Mechanical Battery Piece.
        let mut rng = ScriptRng::new(&[]);
        let mut out = Vec::new();
        open_boss_bag(3327, &BagEnv::default(), &mut rng, &mut out);
        let its = items(&out);
        assert_eq!(
            its[..9],
            [666, 667, 668, 665, 3287, 2107, 547, 1225, 3356],
            "{its:?}"
        );
        assert_eq!(out[6].stack, 25);
        assert_eq!(out[7].stack, 20);
        assert!(its[9..].iter().all(|i| (71..=74).contains(i)), "coins: {its:?}");
    }

    #[test]
    fn w4_f35_prime_bag_replays_the_csharp_draw_order() {
        for seed in [37i32, 127127] {
            let mut got_rng = UnifiedRandom::new(seed);
            let mut got = Vec::new();
            open_boss_bag(3327, &BagEnv::default(), &mut got_rng, &mut got);
            let mut r = UnifiedRandom::new(seed);
            let mut want: Vec<BagGrant> = Vec::new();
            dev_armor_replay(&mut r, &mut want, false);
            if r.next_max(7) == 0 {
                spawn_replay(&mut r, &mut want, 2107, 1);
            }
            spawn_replay(&mut r, &mut want, 547, r.next_range(25, 41));
            spawn_replay(&mut r, &mut want, 1225, r.next_range(20, 36));
            spawn_replay(&mut r, &mut want, 3356, 1);
            money_replay_on(&mut r, 127, 1.0, &mut want);
            assert_eq!(got, want, "seed {seed}");
            assert_eq!(got_rng, r, "seed {seed} stream");
        }
    }

    #[test]
    fn w4_f35_the_money_tail_matches_the_replay_for_every_boss() {
        // The tail alone (no bag arm): every boss's base value through
        // the same-seed replay — the coin ids/stacks and the stream.
        for bag in [3318u16, 3319, 3320, 3321, 3322, 3323, 3324, 3325, 3326, 3327,
                    3328, 3329, 3330, 3331, 3332, 3860, 3861, 3862, 4782, 4957, 5111] {
            let boss = bag_money_boss(bag).unwrap();
            for seed in [3i32, 77] {
                let mut rng = UnifiedRandom::new(seed);
                let mut out = Vec::new();
                bag_money(&mut rng, &mut out, boss, 1.0);
                let (want, want_rng) = money_replay(seed, boss, 1.0);
                assert_eq!(out, want, "bag {bag} seed {seed}");
                assert_eq!(rng, want_rng, "bag {bag} seed {seed} stream");
            }
        }
    }

    #[test]
    fn w4_f35_the_money_tail_expert_multiplier_truncates_like_scalestats() {
        // The e25 seam: value ×2.5 with the (int) truncation BEFORE
        // the spread (`ScaleStats_ByDifficulty`, NPC.cs:18176). The
        // EoW head's 800f → 2000 exactly; the Deerclops 50000 → 125000.
        let mut rng = ScriptRng::new(&[20, 4, 9, 14, 19]); // spread +0, all gates miss
        let mut out = Vec::new();
        bag_money(&mut rng, &mut out, 13, 2.5);
        // 2000 → 20 silver (the >100 tier), remainder 0.
        assert_eq!(out, vec![BagGrant { item: 72, stack: 20, prefix: 0 }]);
        // The zero-value bosses (Betsy/Ogre/Dark Mage) draw the 5
        // gates and mint NOTHING (`while ((int)num10 > 0)` false).
        for boss in [551, 576, 564] {
            let mut rng = ScriptRng::new(&[]);
            let mut out = Vec::new();
            bag_money(&mut rng, &mut out, boss, 1.0);
            assert!(out.is_empty(), "boss {boss} mints nothing");
            assert_eq!(rng.ranges.len(), 5, "the spread + 4 gates drew");
        }
    }

    #[test]
    fn w4_f35_the_grantless_ooa_bags_run_only_the_money_tail() {
        // 3861/3862: no grant case in OpenBossBag — the money tail's
        // 5 draws are the WHOLE consumption (value 0, no coins).
        for bag in [3861u16, 3862] {
            let mut rng = UnifiedRandom::new(9);
            let mut out = Vec::new();
            open_boss_bag(bag, &BagEnv::default(), &mut rng, &mut out);
            assert!(out.is_empty());
            let (want, want_rng) = money_replay(9, bag_money_boss(bag).unwrap(), 1.0);
            assert!(want.is_empty());
            assert_eq!(rng, want_rng, "bag {bag}: only the tail drew");
        }
        // An unknown id draws nothing at all (the `num9 <= 0` return).
        let mut rng = UnifiedRandom::new(9);
        let mut out = Vec::new();
        open_boss_bag(1234, &BagEnv::default(), &mut rng, &mut out);
        assert!(out.is_empty());
        assert_eq!(rng, UnifiedRandom::new(9));
    }

    #[test]
    fn w4_f35_dev_armor_gate_miss_draws_once_and_grants_nothing() {
        // `:7887` — a non-zero gate draw consumes exactly one draw.
        let mut rng = ScriptRng::new(&[1]);
        let mut out = Vec::new();
        try_getting_dev_armor(&mut rng, &mut out, false);
        assert!(out.is_empty());
        assert_eq!(rng.ranges, vec![(0, 16)]);
        // The anniversary world halves the denominator to 8.
        let mut rng = ScriptRng::new(&[7]);
        let mut out = Vec::new();
        try_getting_dev_armor(&mut rng, &mut out, true);
        assert!(out.is_empty());
        assert_eq!(rng.ranges, vec![(0, 8)]);
    }

    #[test]
    fn w4_f35_dev_armor_set_zero_grants_the_five_pieces_in_order() {
        // Gate 0, set 0: the 666/667/668/665/3287 set (`:7891-7897`).
        let mut rng = ScriptRng::new(&[0, 0]);
        let mut out = Vec::new();
        try_getting_dev_armor(&mut rng, &mut out, false);
        assert_eq!(items(&out), [666, 667, 668, 665, 3287]);
        assert!(out.iter().all(|g| g.stack == 1));
    }

    #[test]
    fn w4_f35_dev_armor_sets_ten_and_eleven_carry_the_torch_tails() {
        // `:7952-7965` — the 3024×4 / 3599×4 torch grants close sets
        // 10/11 (the only stack-4 grants in the whole switch).
        for (set, tail) in [(10i32, 3024u16), (11, 3599)] {
            let mut rng = ScriptRng::new(&[0, set]);
            let mut out = Vec::new();
            try_getting_dev_armor(&mut rng, &mut out, false);
            let last = out.last().unwrap();
            assert_eq!((last.item, last.stack), (tail, 4), "set {set}");
            assert_eq!(out.len(), 5, "4 pieces + the torch tail");
        }
        // Set 20 (the last case): the 6137-6141 set, no tail.
        let mut rng = ScriptRng::new(&[0, 20]);
        let mut out = Vec::new();
        try_getting_dev_armor(&mut rng, &mut out, false);
        assert_eq!(items(&out), [6137, 6138, 6139, 6140, 6141]);
    }
}
