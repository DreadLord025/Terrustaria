//! The bestiary unlock-ladder data tables (W4-c37, research/30 gaps #4/#5):
//! the per-entry UIInfoProvider overrides of
//! `BestiaryDatabaseNPCsPopulator.ModifyEntriesThatNeedIt`
//! (`BestiaryDatabaseNPCsPopulator.cs:649-730`), the boss/critter
//! classification the four `BestiaryEntry` factories ride
//! (`BestiaryEntry.cs:21-103`, `NPC.CountsAsACritter` `NPC.cs:6887-6897`,
//! `NPCID.Sets.ShouldBeCountedAsBossForBestiary` `NPCID.cs:10481`) and the
//! rarity-star tiers (`ContentSamples.FillNpcRarities` +
//! `GetNPCBestiaryRarityStarsCount`, `ContentSamples.cs:1094-1306`).
//!
//! The ladder maths itself is `tr_core::bestiary` (the ½/⅕ steps +
//! `kills_needed_for`); this module is the DATA half: which provider an
//! entry reads, which credit id it reads it from, and how many stars its
//! portrait carries.

use crate::bestiary::BestiaryEntryDef;

/// One entry's `IBestiaryUICollectionInfoProvider`
/// (`BestiaryEntryUnlockState` producers). `credit_net` is the net id whose
/// bestiary credit id the provider reads (`GetBestiaryCreditId`,
/// `NPC.cs:95139-95142` — the fold rows read the BOSS's credit, not their
/// own).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Gate {
    /// `CommonEnemyUICollectionInfoProvider` (`:14-48`): the kill ladder.
    /// `quick` is the provider's own `quickUnlock` argument; the entry's
    /// `quick_unlock` field (the `BestiaryEntry.Enemy` boss rule,
    /// `BestiaryEntry.cs:37-41`) ORs onto it at evaluation.
    Common { credit_net: i32, quick: bool },
    /// `TownNPCUICollectionInfoProvider` (`:14-20`): chatted-with → tier 4,
    /// else unknown. No kill ladder at all.
    Town { credit_net: i32 },
    /// `CritterUICollectionInfoProvider` (`:14-20`): sighted → tier 4,
    /// else unknown.
    Critter { credit_net: i32 },
    /// `GoldCritterUICollectionInfoProvider` (`:21-75`): the max of the
    /// gold credit's own sight and its normal-critter peers' sights
    /// (`:23-37`), held at tier 0 until ANY `GoldCrittersCollection`
    /// member has been seen (`TryFindingOneGoldCritterThatIsAlreadyUnlocked`,
    /// `:46-52`/`:56-66`). The W4-c8 census's provider, folded onto the
    /// c37 gate table by the W4-e23 final compose.
    GoldCritter {
        credit_net: i32,
        peers: &'static [i32],
    },
    /// `HighestOfMultipleUICollectionInfoProvider` (`:17-31`): the max of
    /// the sub-providers' states.
    Highest(&'static [Gate]),
}

/// The Skeletron head's override (`BestiaryDatabaseNPCsPopulator.cs:677`):
/// its own quick-unlock ladder OR the Clothier's chat. The Skeletron hand
/// 68 / Old Man 37 wider triples live at `:676`/`:678` (`gate` below).
const SKELETRON_GATES: &[Gate] = &[
    Gate::Common {
        credit_net: 35,
        quick: true,
    },
    Gate::Town { credit_net: 54 },
];

/// The Dungeon Guardian's override (`BestiaryDatabaseNPCsPopulator.cs:676`):
/// `HighestOfMultiple(own 68 quick, 35 quick, Clothier 54 town)`.
const DUNGEON_GUARDIAN_GATES: &[Gate] = &[
    Gate::Common {
        credit_net: 68,
        quick: true,
    },
    Gate::Common {
        credit_net: 35,
        quick: true,
    },
    Gate::Town { credit_net: 54 },
];

/// The Old Man's override (`BestiaryDatabaseNPCsPopulator.cs:678`):
/// `HighestOfMultiple(own 37 town, Clothier 54 town, 35 quick)`.
const OLD_MAN_GATES: &[Gate] = &[
    Gate::Town { credit_net: 37 },
    Gate::Town { credit_net: 54 },
    Gate::Common {
        credit_net: 35,
        quick: true,
    },
];

/// The town-NPC roster the populator registers by hand
/// (`AddTownNPCs_Manual`, `BestiaryDatabaseNPCsPopulator.cs:802-844`),
/// narrowed to the ids the port's `ENTRIES` carries. Every one reads the
/// chat provider — the Guide included (research/30 gap #5: it must NOT
/// ride the kill ladder).
const TOWN_NET_IDS: &[i32] = &[
    17, 18, 19, 20, 22, 38, 54, 107, 108, 124, 207, 208, 227, 228, 353, 368, 369, 441, 550, 588,
    633, 663,
];

/// `NPC.CountsAsACritter` (`NPC.cs:6887-6897`): `lifeMax <= 5 &&
/// damage == 0 && type != 594 && type != 686`. Of the port's roster only
/// the Goldfish 55 (`NPC.cs:9460-9470`: 5 HP, 0 damage, catchItem 261) and
/// the Etherian Portal 549 (`NPC.cs:16033-16040`: 5 HP, 0 damage,
/// dontTakeDamage) qualify — the automated pass registers both through
/// `BestiaryEntry.Critter` (`BestiaryDatabaseNPCsPopulator.cs:858-861`).
/// The W4-c8 wave's nine crawler kinds (357/374/375/448/484-486/606/661)
/// join this list when their ENTRIES rows land on the union - the c8
/// census's `is_critter_entry` set, recorded here for the compose. The
/// Prismatic Lacewing 661 joins now (`NPC.cs:17351-17366`: 5 HP, 0 damage).
const CRITTER_NET_IDS: &[i32] = &[55, 549, 661];

/// `NPCID.Sets.GoldCrittersCollection` (`NPCID.cs:10121-10125`), verbatim
/// order — the gold provider's any-seen gate
/// (`GoldCritterUICollectionInfoProvider.cs:56-66`).
pub const GOLD_CRITTERS_COLLECTION: &[i32] = &[
    443, 442, 592, 593, 444, 601, 445, 446, 605, 447, 627, 613, 448, 539,
];

/// The gold-critter override dictionary
/// (`BestiaryDatabaseNPCsPopulator.cs:703-720`), verbatim: the gold entry's
/// normal-critter peer list (the 539 row carries TWO peers). The provider
/// is `GoldCritterUICollectionInfoProvider(peers, ownCredit)` (`:719`).
const GOLD_CRITTER_PEERS: &[(i32, &[i32])] = &[
    (443, &[46]),
    (442, &[74]),
    (592, &[55]),
    (444, &[356]),
    (601, &[599]),
    (445, &[361]),
    (446, &[377]),
    (605, &[604]),
    (447, &[300]),
    (627, &[626]),
    (613, &[612]),
    (448, &[357]),
    (539, &[299, 538]),
];

/// The four critter-twin folds (`:721-730`) are the match arms in `gate`
/// below — the dictionary verbatim: {362:363, 364:365, 602:603, 608:609}.

/// The entry's provider — the populator's override walk
/// (`ModifyEntriesThatNeedIt`) over the factory defaults.
///
/// Unported override families (no `ENTRIES` row rides them, recorded for
/// the roster waves): the Salamander/Shelly/Dad 494-506 world-roster gate
/// (`:649-652`, `SalamanderShellyDadUICollectionInfoProvider.cs:18-66`),
/// the Tortured Soul 534 (`:653`); the Dungeon Guardian / Old Man triples
/// (`:676`/`:678`) are tabled below. The gold-critter collection gate
/// (`:703-720`)
/// and the four critter-twin folds (`:721-730`) are TABLED below (the
/// W4-c8 census compose) — their ids await the c8/d42 roster waves.
pub fn gate(net_id: i32) -> Gate {
    // The DD2 mini-boss quick rows (`BestiaryDatabaseNPCsPopulator.cs:679-682`):
    // quickUnlock on the provider even though the factory flag is false
    // (none of the four is a `SetDefaults` boss).
    match net_id {
        565 | 577 | 551 | 491 => {
            return Gate::Common {
                credit_net: net_id,
                quick: true,
            };
        }
        _ => {}
    }
    // The quick-unlock credit folds (`:683-702`, the dictionary verbatim):
    // the minion's entry reads the BOSS's kill count with quickUnlock: true.
    // The ids without a port row (116/139/658-660/454/521) stay listed so
    // the table is the C#'s, not a hand-picked subset.
    match net_id {
        5 => {
            return Gate::Common {
                credit_net: 4,
                quick: true,
            };
        } // Servant -> EyeofCthulhu
        267 => {
            return Gate::Common {
                credit_net: 266,
                quick: true,
            };
        } // Creeper -> BrainofCthulhu
        115 | 116 | 117 => {
            return Gate::Common {
                credit_net: 113,
                quick: true,
            };
        } // Hungry I/II/Leech -> WallofFlesh
        139 => {
            return Gate::Common {
                credit_net: 134,
                quick: true,
            };
        } // Probe -> TheDestroyer
        372 => {
            return Gate::Common {
                credit_net: 370,
                quick: true,
            };
        } // Sharkron -> DukeFishron
        658 | 659 | 660 => {
            return Gate::Common {
                credit_net: 657,
                quick: true,
            };
        } // SlimeQueen spawns -> QueenSlime
        454 | 521 => {
            return Gate::Common {
                credit_net: 439,
                quick: true,
            };
        } // Cultist minions -> AncientCultist
        // The Dungeon Guardian and Old Man triples (`:676`/`:678`).
        68 => return Gate::Highest(DUNGEON_GUARDIAN_GATES),
        37 => return Gate::Highest(OLD_MAN_GATES),
        35 => return Gate::Highest(SKELETRON_GATES),
        _ => {}
    }
    // The gold-critter dictionary (`:703-720`): the max(own, peers) sight
    // provider behind the any-gold-seen collection gate.
    if let Some(&(_, peers)) = GOLD_CRITTER_PEERS.iter().find(|&&(id, _)| id == net_id) {
        return Gate::GoldCritter {
            credit_net: net_id,
            peers,
        };
    }
    // The critter-twin folds (`:721-730`): HighestOfMultiple over the two
    // sight providers (the C# builds both from the dictionary's pair).
    match net_id {
        362 => {
            return Gate::Highest(&[
                Gate::Critter { credit_net: 362 },
                Gate::Critter { credit_net: 363 },
            ]);
        }
        364 => {
            return Gate::Highest(&[
                Gate::Critter { credit_net: 364 },
                Gate::Critter { credit_net: 365 },
            ]);
        }
        602 => {
            return Gate::Highest(&[
                Gate::Critter { credit_net: 602 },
                Gate::Critter { credit_net: 603 },
            ]);
        }
        608 => {
            return Gate::Highest(&[
                Gate::Critter { credit_net: 608 },
                Gate::Critter { credit_net: 609 },
            ]);
        }
        _ => {}
    }
    if TOWN_NET_IDS.contains(&net_id) {
        return Gate::Town { credit_net: net_id };
    }
    if CRITTER_NET_IDS.contains(&net_id) {
        return Gate::Critter { credit_net: net_id };
    }
    Gate::Common {
        credit_net: net_id,
        quick: false,
    }
}

/// `NPCID.Sets.ShouldBeCountedAsBossForBestiary` (`NPCID.cs:10481`):
/// the four pillars + the Eater of Worlds head + the Torch God count as
/// bosses for the bestiary even though `SetDefaults` never sets `boss`.
pub fn counted_as_boss_for_bestiary(net_id: i32) -> bool {
    matches!(net_id, 517 | 422 | 507 | 493 | 13 | 664)
}

/// The `SetDefaults` `boss = true` rows (`NPC.cs:8716/9180/9389/10358/
/// 10535/10552/10570/10676/11868/12176/12393/12452/13716/14061/14097/
/// 14116/14136/14227/17202/17323/17401/17460`), narrowed to the port's
/// roster. The Eater of Worlds' runtime `boss` latch (`NPC.cs:78425-78428`,
/// set in NPCLoot when no other segment lives) is NOT a SetDefaults row —
/// 13 rides `counted_as_boss_for_bestiary` instead. 657/664 still have no
/// port entry; 636 (the Empress of Light, `NPC.cs:17202`) landed with the
/// entity row.
pub fn set_defaults_boss(net_id: i32) -> bool {
    matches!(
        net_id,
        4 | 35
            | 50
            | 113
            | 125
            | 126
            | 127
            | 134
            | 222
            | 245
            | 262
            | 266
            | 370
            | 395
            | 396
            | 397
            | 398
            | 439
            | 636
            | 668
    )
}

/// `NPC.rarity` — the `SetDefaults` rarity rows (`NPC.cs`), narrowed to
/// the port's roster; every other entry keeps the class default 0. The
/// `RareSpawnBestiaryInfoElement` ("IsRare" search string,
/// `RareSpawnBestiaryInfoElement.cs:20-27`) and the star formula read it.
pub fn rarity(net_id: i32) -> i32 {
    match net_id {
        -4 => 2,  // Pinky (`NPC.cs:7697`)
        44 => 1,  // UndeadMiner (`:9320`)
        45 => 4,  // Tim (`:9334`)
        52 => 2,  // DoctorBones (`:9430`)
        53 => 2,  // TheGroom (`:9444`)
        66 => 1,  // VoodooDemon (`:9638`)
        71 => 1,  // DungeonSlime (`:9712`)
        216 => 1, // PirateCaptain (`:11786`)
        399 => 1, // MartianProbe (`:14080`)
        471 => 1, // GoblinSummoner (`:15740`)
        // The C# `this.rarity` rows the first pass missed (the 1.4.5.8
        // `SetDefaults` blocks; the negative slime variants ride
        // `SetDefaultsFromNetId` and are untouched above).
        73 | 109 | 217 | 218 | 219 | 221 | 244 | 287 | 292 | 293 | 477 | 480 | 586 | 587
        | 618 | 620 | 621 | 624 => 1,
        220 | 243 | 251 | 290 | 583 | 584 | 585 => 2,
        // The gold critters (`:14260-14345` 442-448, `:15896` 539,
        // `:16692`/`:16705` 592/593, `:16733` 601, `:16774` 605,
        // `:16871` 613, `:17062` 627) - rarity 3 -> 5 stars.
        442 | 443 | 444 | 445 | 446 | 447 | 448 | 592 | 601 | 605 | 613 | 627 => 3,
        85 | 375 | 629 => 4,
        473 | 474 | 475 | 476 => 5,
        _ => 0,
    }
}

/// `ContentSamples.GetNPCBestiaryRarityStarsCount`
/// (`ContentSamples.cs:1245-1306`): the star tier from rarity, the boss
/// flag and the `damage + defense + lifeMax/4` bulk ladder, capped at 5
/// and truncated to an int. The stats are the Classic-difficulty
/// `SetDefaults` sample (`FillNpcRarities:1096-1104`) — the port's
/// `entry_stats` reads the same tables.
pub fn stars_from_stats(rarity: i32, boss: bool, damage: i32, defense: i32, life_max: i32) -> i32 {
    let mut num = 1.0f32 + rarity as f32;
    match rarity {
        1 => num += 1.0,
        2 => num += 1.5,
        3 => num += 2.0,
        4 => num += 2.5,
        5 => num += 3.0,
        r if r > 0 => num += 3.5,
        _ => {}
    }
    if boss {
        num += 0.5;
    }
    let num2 = damage + defense + life_max / 4;
    if num2 > 10000 {
        num += 3.5;
    } else if num2 > 5000 {
        num += 3.0;
    } else if num2 > 1000 {
        num += 2.5;
    } else if num2 > 500 {
        num += 2.0;
    } else if num2 > 150 {
        num += 1.5;
    } else if num2 > 50 {
        num += 1.0;
    }
    if num > 5.0 {
        num = 5.0;
    }
    num as i32
}

/// The Classic sample's progression-gated stat rows — the two ids whose
/// `SetDefaults` stats ride world flags (`FillNpcRarities` builds the
/// sample on a fresh `Main`, so every flag reads false):
/// - 548 the Eternia Crystal keeps its pre-hardmode 1000 HP / 14 defense
///   (`NPC.cs:16003-16004`; the 3000/18 and 5000/20 rows at `:16012-16021`
///   need `downedMechBossAny`/`downedGolemBoss`);
/// - 59 the Lava Slime keeps 50 HP / 15 damage / 10 defense
///   (`NPC.cs:9518-9523`; the 25/7/2 row at `:9531-9535` needs
///   `Main.remixWorld`).
/// Every other roster id's sample stats equal the port's `entry_stats`
/// tables (the Classic `ScaleStats` multipliers are 1 and the <6-HP bump
/// skips the negative-net-id slimes, `NPC.cs:18160-18163`).
fn sample_stats(net_id: i32, stats: crate::npc::EntryStats) -> crate::npc::EntryStats {
    match net_id {
        548 => crate::npc::EntryStats {
            hp: 1000,
            defense: 14,
            ..stats
        },
        59 => crate::npc::EntryStats {
            hp: 50,
            damage: 15,
            defense: 10,
            ..stats
        },
        _ => stats,
    }
}

/// The `FillNpcRarities` override table (`ContentSamples.cs:1106-1242`),
/// verbatim — the rows win over the formula. Ids without a port entry
/// stay listed so the table is the C#'s, not a hand-picked subset.
pub fn stars_override(net_id: i32) -> Option<i32> {
    Some(match net_id {
        22 | 17 | 18 | 38 | 19 => 1,
        369 | 227 | 353 | 550 | 588 | 228 | 124 | 54 | 37 => 2,
        20 | 107 | 108 | 178 | 216 | 209 | 368 => 3,
        160 | 441 | 142 | 633 | 663 => 5,
        208 | 453 => 4,
        638 | 637 | 656 | 670 | 678 | 679 | 680 | 681 | 682 | 683 | 684 => 3,
        664 | 484 | 677 | 676 | 690 => 5,
        614 | 303 | 337 | 691 => 4,
        360 | 374 | 661 | 671 | 672 | 673 | 674 | 675 | 599 | 358 | 654 | 604 | 611 | 615 | 487
        | 669 | 185 | 206 | 302 | 316 | 317 | 318 | 319 | 320 | 321 | 322 | 323 | 324 | 331
        | 332 | 335 | 336 | 333 | 334 | 250 | -9 | 66 | 47 | 57 | 168 | 464 | 465 | 470 | 170
        | 180 | 171 => 3,
        362 | 364 | 616 | 298 | 355 | 653 | 540 | 612 | 608 | 607 | 626 | 486 | 149 | 366 | 223
        | 161 | 499 | 494 | 496 | 537 | 546 | 29 | 594 | 628 | 225 | 224 | 481 | 483 | 184 | 16
        | 4 | 50 | 655 | 301 => 2,
        541 | 205 | 471 | 491 => 4,
        35 | 13 => 3,
        134 | 262 => 4,
        668 => 3,
        _ => return None,
    })
}

/// `ContentSamples.NpcBestiaryRarityStars[netId]`
/// (`ContentSamples.cs:1100-1104` + the override table): the entry's
/// filled-star count (`NPCPortraitInfoElement`, `BestiaryEntry.cs:28`).
/// The portrait's star container (`CreateStarsContainer`,
/// `NPCPortraitInfoElement.cs:107-152`) draws `Icon_Rank_Light/Dim` —
/// neither has an atlas slot yet, so the UI wiring lands with the art;
/// the data half is pinned by the W4-c37 census witness.
#[allow(dead_code)]
pub fn rarity_stars(entry: &BestiaryEntryDef) -> i32 {
    if let Some(stars) = stars_override(entry.net_id) {
        return stars;
    }
    let stats = sample_stats(
        entry.net_id,
        crate::npc::entry_stats(
            entry.kind,
            entry.variant.unwrap_or(crate::npc::SlimeVariant::Blue),
        ),
    );
    stars_from_stats(
        rarity(entry.net_id),
        set_defaults_boss(entry.net_id),
        stats.damage,
        stats.defense,
        stats.hp,
    )
}
