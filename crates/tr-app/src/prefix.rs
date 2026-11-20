//! Item prefixes — `Item.Prefix` (`Item.cs:504-618`), the per-category roll
//! tables (`PrefixLegacy.cs`), the stat multipliers
//! (`Item.TryGetPrefixStatMultipliersForItem`, `Item.cs:709-1080`) and the
//! equipment pass (`Player.GrantPrefixBenefits`, `Player.cs:13953-14047`).
//!
//! The port recomputes the prefixed stats from the kind's defaults on demand
//! (`prefixed_stats`) instead of mutating an instance: a natural drop is a
//! fresh `SetDefaults` item in vanilla, so the recompute is exact for the
//! spawn/pickup flows we model (a reforge chain would compound — M3-REFORGE-1
//! stores a stat snapshot there).
//!
//! The RNG seam follows the vanilla draw order: `Prefix(-1)` checks
//! `CanHavePrefixes` first and returns before any roll, so non-prefixable
//! kinds consume no stream draws. The draws are the C# `Main.rand`
//! (`UnifiedRandom`) stream (`S1-A-5`); the worldgen chest seam still feeds the
//! splitmix stream until S1-A-4 migrates it ([`roll_prefix_splitmix`]).

use crate::itemdata::ItemDef;
use crate::player::Player;
use tr_core::UnifiedRandom;
use tr_core::damage::round_bankers;

// --- The roll tables (`PrefixLegacy.Prefixes`, static ctor :29-59) ----------

pub(crate) const PREFIXES_FOR_SWORDS: &[u16] = &[
    1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 36, 37, 38, 53, 54, 55, 39, 40, 56, 41, 57,
    42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 59, 60, 61, 81,
];
pub(crate) const PREFIXES_FOR_SPEARS: &[u16] = &[
    36, 37, 38, 53, 54, 55, 39, 40, 56, 41, 57, 59, 60, 61,
];
pub(crate) const PREFIXES_FOR_GUNS_BOWS: &[u16] = &[
    16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 58, 36, 37, 38, 53, 54, 55, 39, 40, 56, 41, 57, 42, 44,
    45, 46, 47, 48, 49, 50, 51, 59, 60, 61, 82,
];
pub(crate) const PREFIXES_FOR_MAGIC: &[u16] = &[
    26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 52, 36, 37, 38, 53, 54, 55, 39, 40, 56, 41, 57, 42, 43,
    44, 45, 46, 47, 48, 49, 50, 51, 59, 60, 61, 83,
];
pub(crate) const PREFIXES_FOR_SUMMONS: &[u16] = &[
    85, 86, 87, 88, 89, 90, 91, 92, 93, 94, 95, 96, 97, 55, 38, 54, 53, 57, 40, 56, 41, 39,
];
pub(crate) const PREFIXES_FOR_BOOMERANGS: &[u16] = &[
    36, 37, 38, 53, 54, 55, 39, 40, 56, 41, 57, 59, 60, 61,
];
/// `PrefixesForBoomeransAndChakrums_TerrarianYoyo` — the Terrarian (3389) adds
/// the yoyo-exclusive 84 to the boomerang list.
pub(crate) const PREFIXES_FOR_TERRARIAN: &[u16] = &[
    36, 37, 38, 53, 54, 55, 39, 40, 56, 41, 57, 59, 60, 61, 84,
];
pub(crate) const PREFIXES_FOR_ACCESSORIES: &[u16] = &[
    62, 63, 64, 65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79, 80,
];

// --- The category membership sets (`PrefixLegacy.ItemSets`, static ctor
// :62-71; membership decides the roll table in `Item.GetRollablePrefixes`
// order, `Item.cs:1101-1140`) ----------------------------------------------

const SWORDS_HAMMERS_AXES_PICKS: &[u16] = &[
    1, 4, 6, 7, 10, 24, 45, 46, 65, 103, 104, 121, 122, 155, 186, 190, 196, 198, 199, 200, 201,
    202, 203, 204, 213, 217, 273, 367, 368, 426, 482, 483, 484, 486, 653, 654, 656, 657, 659, 660,
    671, 672, 674, 675, 676, 723, 724, 757, 776, 777, 778, 787, 795, 797, 798, 799, 881, 882, 921,
    922, 946, 989, 990, 991, 992, 993, 1123, 1166, 1185, 1188, 1192, 1195, 1199, 1202, 1222, 1223,
    1224, 1226, 1227, 1230, 1233, 1234, 1294, 1304, 1305, 1306, 1320, 1327, 1506, 1507, 1786, 1826,
    1827, 1909, 1917, 1928, 2176, 2273, 2320, 2330, 2341, 2516, 2517, 2608, 2745, 2746, 2772, 2775,
    2776, 2777, 2780, 2781, 2782, 2785, 2786, 2880, 3013, 3018, 3063, 3065, 3106, 3211, 3258, 3349,
    3351, 3352, 3462, 3465, 3466, 3481, 3482, 3483, 3484, 3485, 3487, 3488, 3489, 3490, 3491, 3493,
    3494, 3495, 3496, 3497, 3499, 3500, 3501, 3502, 3503, 3505, 3506, 3507, 3508, 3509, 3511, 3512,
    3513, 3514, 3515, 3517, 3518, 3519, 3520, 3521, 3522, 3523, 3524, 3525, 3764, 3765, 3766, 3767,
    3768, 3769, 3772, 3823, 3827, 4059, 4258, 4259, 4317, 4672, 4678, 4679, 4680, 4707, 4711, 4911,
    4912, 4913, 4914, 4923, 4956, 5074, 5094, 5095, 5096, 5097, 5129, 5283, 5284, 5295, 5382, 5462,
    5473, 5474, 5475, 5476, 5477, 5478, 5479, 5480, 5535, 5536, 5669, 5670, 5671, 5688,
];
const SPEARS_MACES_CHAINSAWS_DRILLS_PUNCH_CANNON: &[u16] = &[
    160, 162, 163, 220, 274, 277, 280, 383, 384, 385, 386, 387, 388, 389, 390, 406, 537, 550, 579,
    756, 801, 802, 1186, 1189, 1190, 1193, 1196, 1197, 1200, 1203, 1204, 1228, 1231, 1232, 1259,
    1262, 1297, 1314, 1325, 1947, 2331, 2332, 2342, 2424, 2611, 2773, 2774, 2778, 2779, 2783, 2784,
    2798, 3012, 3098, 3368, 3463, 3464, 3473, 3835, 3836, 3858, 4061, 4144, 4272, 4463, 4788, 4789,
    4790, 5011, 5012, 5526, 5687,
];
pub(crate) const GUNS_BOWS: &[u16] = &[
    39, 44, 95, 96, 98, 99, 120, 164, 197, 219, 266, 281, 434, 435, 436, 481, 506, 533, 534, 578,
    655, 658, 661, 679, 682, 725, 758, 759, 760, 796, 800, 905, 923, 964, 986, 1156, 1187, 1194,
    1201, 1229, 1254, 1255, 1258, 1265, 1319, 1553, 1782, 1784, 1835, 1870, 1910, 1929, 1946, 2223,
    2269, 2270, 2515, 2624, 2747, 2796, 2797, 2888, 3007, 3008, 3019, 3029, 3052, 3107, 3210, 3350,
    3475, 3480, 3486, 3492, 3498, 3504, 3510, 3516, 3540, 3546, 3788, 3821, 3854, 3859, 3930, 4058,
    4060, 4381, 4703, 4953, 5117, 5282, 5460,
];
pub(crate) const MAGIC: &[u16] = &[
    64, 112, 113, 127, 157, 165, 218, 272, 494, 495, 496, 514, 517, 518, 519, 683, 726, 739, 740,
    741, 742, 743, 744, 788, 1121, 1155, 1178, 1244, 1256, 1260, 1264, 1266, 1295, 1296, 1308,
    1313, 1336, 1444, 1445, 1446, 1801, 1930, 1931, 2188, 2622, 2623, 2750, 2795, 2882, 3006, 3014,
    3051, 3053, 3069, 3105, 3209, 3269, 3377, 3476, 3541, 3542, 3570, 3779, 3787, 3852, 3870, 4062,
    4270, 4347, 4348, 4715, 4952, 5065, 5118, 5147, 5461, 6152, 6153, 6154, 6155, 6171, 6173,
];
pub(crate) const SUMMON: &[u16] = &[
    1157, 1309, 1572, 1802, 2364, 2365, 2366, 2535, 2551, 2584, 2621, 2749, 3249, 3474, 3531, 3569,
    3571, 3818, 3819, 3820, 3824, 3825, 3826, 3829, 3830, 3831, 3832, 3833, 3834, 4269, 4273, 4281,
    4607, 4758, 5005, 5069, 5114, 5119, 5456, 5463, 5663, 5664, 6148, 6149, 6161, 6164,
];
const BOOMERANGS_CHAKRAMS: &[u16] = &[
    55, 119, 191, 284, 561, 670, 1122, 1324, 1513, 1569, 1571, 1825, 1918, 3030, 3054, 3262, 3278,
    3279, 3280, 3281, 3282, 3283, 3284, 3285, 3286, 3287, 3288, 3289, 3290, 3291, 3292, 3315, 3316,
    3317, 3543, 4760, 4764, 4818, 5294, 5298, 5519,
];
const ITEMS_THAT_CAN_HAVE_LEGENDARY_2: &[u16] = &[3389];

/// `PrefixID.Sets.ReducedNaturalChance` (`PrefixID.cs:14`): the weak prefixes
/// only survive a natural roll 1/3 of the time.
const REDUCED_NATURAL_CHANCE: &[u16] = &[
    7, 8, 9, 10, 11, 22, 23, 24, 29, 30, 31, 39, 40, 41, 47, 48, 49, 56,
];

/// `ItemID.Sets.CanGetPrefixes` (`ItemID.cs:429`): `CreateBoolSet(true, ...)`
/// is a denial list — every id prefixes except these (the guide doll, dyes,
/// pet licenses and the other vanity accessories).
const CAN_GET_PREFIXES_EXCLUDED: &[u16] = &[
    267, 562, 563, 564, 565, 566, 567, 568, 569, 570, 571, 572, 573, 574, 576, 1307, 1596, 1597,
    1598, 1599, 1600, 1601, 1602, 1603, 1604, 1605, 1606, 1607, 1608, 1609, 1610, 1963, 1964, 1965,
    2742, 3044, 3235, 3236, 3237, 3370, 3371, 3796, 3869, 4077, 4078, 4079, 4080, 4081, 4082, 4237,
    4356, 4357, 4358, 4421, 4606, 4979, 4985, 4990, 4991, 4992, 5006, 5014, 5015, 5016, 5017, 5018,
    5019, 5020, 5021, 5022, 5023, 5024, 5025, 5026, 5027, 5028, 5029, 5030, 5031, 5032, 5033, 5034,
    5035, 5036, 5037, 5038, 5039, 5040, 5044, 5112, 5362, 5538, 5578, 5579, 5580, 5581, 5582, 5637,
    5638, 5639, 6144, 6145, 6146,
];

// --- Category resolution ----------------------------------------------------

pub(crate) fn in_set(set: &[u16], kind: u16) -> bool {
    set.binary_search(&kind).is_ok()
}

/// `Item.CanHavePrefixes` (`Item.cs:493-496`): the kind has a rollable
/// prefix table at all. The crafting cursor gate reads it
/// (`Main.CursorHasSpaceToCraftRecipe`, `Main.cs:42325`).
pub(crate) fn can_have_prefixes(kind: u16) -> bool {
    rollable_prefixes(kind).is_some()
}

/// `Item.GetRollablePrefixes` (`Item.cs:1101-1137`): the check order decides
/// the table for kinds that sit in several sets.
pub(crate) fn rollable_prefixes(kind: u16) -> Option<&'static [u16]> {
    if in_set(SWORDS_HAMMERS_AXES_PICKS, kind) {
        return Some(PREFIXES_FOR_SWORDS);
    }
    if in_set(SPEARS_MACES_CHAINSAWS_DRILLS_PUNCH_CANNON, kind) {
        return Some(PREFIXES_FOR_SPEARS);
    }
    if in_set(GUNS_BOWS, kind) {
        return Some(PREFIXES_FOR_GUNS_BOWS);
    }
    if in_set(MAGIC, kind) {
        return Some(PREFIXES_FOR_MAGIC);
    }
    if in_set(SUMMON, kind) {
        return Some(PREFIXES_FOR_SUMMONS);
    }
    if in_set(BOOMERANGS_CHAKRAMS, kind) {
        return Some(PREFIXES_FOR_BOOMERANGS);
    }
    if in_set(ITEMS_THAT_CAN_HAVE_LEGENDARY_2, kind) {
        return Some(PREFIXES_FOR_TERRARIAN);
    }
    // `IsAPrefixableAccessory` (`Item.cs:1150-1157`): `accessory && !vanity
    // && CanGetPrefixes[type]`. The vanity flag is the C#'s own `Item.vanity`
    // (the generated row) — the capes/rings/monoliths/tails sit in the
    // accessory family but take no prefix, and the denial list alone does
    // not cover them (only the music boxes/dyes/pet licenses are on it).
    // S1-P-i45/i49 seam: read the CORRECTED registry::item row (the 6171
    // Bejeweled Staff zero-row leak), which layers ITEM_CORRECTIONS over
    // the generated table.
    let def = crate::registry::item(kind)?;
    if def.accessory && !def.vanity && !in_set(CAN_GET_PREFIXES_EXCLUDED, kind) {
        return Some(PREFIXES_FOR_ACCESSORIES);
    }
    None
}

// --- The stat application ---------------------------------------------------

/// The multipliers `TryGetPrefixStatMultipliersForItem` produces for one
/// prefix (`Item.cs:709-1078`'s switch); identity for prefix 0 and the
/// accessory ids (62-80 grant player benefits, not item stats).
#[derive(Clone, Copy, Debug, PartialEq)]
struct Multipliers {
    dmg: f32,
    kb: f32,
    spd: f32,
    size: f32,
    shtspd: f32,
    mcst: f32,
    crt: i32,
    tagdmg: i32,
    arpen: i32,
}

impl Multipliers {
    const IDENTITY: Multipliers = Multipliers {
        dmg: 1.0,
        kb: 1.0,
        spd: 1.0,
        size: 1.0,
        shtspd: 1.0,
        mcst: 1.0,
        crt: 0,
        tagdmg: 0,
        arpen: 0,
    };

    fn stat_switch(prefix: u16) -> Multipliers {
        let (dmg, kb, spd, size, shtspd, mcst, crt, tagdmg, arpen) = match prefix {
            1 => (1.0, 1.0, 1.0, 1.12, 1.0, 1.0, 0, 0, 0),
            2 => (1.0, 1.0, 1.0, 1.18, 1.0, 1.0, 0, 0, 0),
            3 => (1.05, 1.0, 1.0, 1.05, 1.0, 1.0, 2, 0, 0),
            4 => (1.1, 1.1, 1.0, 1.1, 1.0, 1.0, 0, 0, 0),
            5 => (1.15, 1.0, 1.0, 1.0, 1.0, 1.0, 0, 0, 0),
            6 => (1.1, 1.0, 1.0, 1.0, 1.0, 1.0, 0, 0, 0),
            7 => (1.0, 1.0, 1.0, 0.82, 1.0, 1.0, 0, 0, 0),
            8 => (0.85, 0.85, 1.0, 0.87, 1.0, 1.0, 0, 0, 0),
            9 => (1.0, 1.0, 1.0, 0.9, 1.0, 1.0, 0, 0, 0),
            10 => (0.85, 1.0, 1.0, 1.0, 1.0, 1.0, 0, 0, 0),
            11 => (1.0, 0.9, 1.1, 0.9, 1.0, 1.0, 0, 0, 0),
            12 => (1.05, 1.1, 1.15, 1.1, 1.0, 1.0, 0, 0, 0),
            13 => (0.9, 0.8, 1.0, 1.1, 1.0, 1.0, 0, 0, 0),
            14 => (1.0, 1.15, 1.1, 1.0, 1.0, 1.0, 0, 0, 0),
            15 => (1.0, 0.9, 0.85, 1.0, 1.0, 1.0, 0, 0, 0),
            16 => (1.1, 1.0, 1.0, 1.0, 1.0, 1.0, 3, 0, 0),
            17 => (1.0, 1.0, 0.85, 1.0, 1.1, 1.0, 0, 0, 0),
            18 => (1.0, 1.0, 0.9, 1.0, 1.15, 1.0, 0, 0, 0),
            19 => (1.0, 1.15, 1.0, 1.0, 1.05, 1.0, 0, 0, 0),
            20 => (1.1, 1.05, 0.95, 1.0, 1.05, 1.0, 2, 0, 0),
            21 => (1.1, 1.15, 1.0, 1.0, 1.0, 1.0, 0, 0, 0),
            22 => (0.85, 0.9, 1.0, 1.0, 0.9, 1.0, 0, 0, 0),
            23 => (1.0, 1.0, 1.15, 1.0, 0.9, 1.0, 0, 0, 0),
            24 => (1.0, 0.8, 1.1, 1.0, 1.0, 1.0, 0, 0, 0),
            25 => (1.15, 1.0, 1.1, 1.0, 1.0, 1.0, 1, 0, 0),
            26 => (1.1, 1.0, 1.0, 1.0, 1.0, 0.85, 0, 0, 0),
            27 => (1.0, 1.0, 1.0, 1.0, 1.0, 0.85, 0, 0, 0),
            28 => (1.15, 1.05, 1.0, 1.0, 1.0, 0.85, 0, 0, 0),
            29 => (1.0, 1.0, 1.0, 1.0, 1.0, 1.1, 0, 0, 0),
            30 => (0.9, 1.0, 1.0, 1.0, 1.0, 1.2, 0, 0, 0),
            31 => (0.9, 0.9, 1.0, 1.0, 1.0, 1.0, 0, 0, 0),
            32 => (1.1, 1.0, 1.0, 1.0, 1.0, 1.15, 0, 0, 0),
            33 => (1.0, 1.1, 0.9, 1.0, 1.0, 1.1, 0, 0, 0),
            34 => (1.1, 1.1, 1.1, 1.0, 1.0, 0.9, 0, 0, 0),
            35 => (1.15, 1.15, 1.0, 1.0, 1.0, 1.2, 0, 0, 0),
            36 => (1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 3, 0, 0),
            37 => (1.1, 1.1, 1.0, 1.0, 1.0, 1.0, 3, 0, 0),
            38 => (1.0, 1.15, 1.0, 1.0, 1.0, 1.0, 0, 0, 0),
            39 => (0.7, 0.8, 1.0, 1.0, 1.0, 1.0, 0, 0, 0),
            40 => (0.85, 1.0, 1.0, 1.0, 1.0, 1.0, 0, 0, 0),
            41 => (0.9, 0.85, 1.0, 1.0, 1.0, 1.0, 0, 0, 0),
            42 => (1.0, 1.0, 0.9, 1.0, 1.0, 1.0, 0, 0, 0),
            43 => (1.1, 1.0, 0.9, 1.0, 1.0, 1.0, 0, 0, 0),
            44 => (1.0, 1.0, 0.9, 1.0, 1.0, 1.0, 3, 0, 0),
            45 => (1.0, 1.0, 0.95, 1.0, 1.0, 1.0, 0, 0, 0),
            46 => (1.07, 1.0, 0.94, 1.0, 1.0, 1.0, 3, 0, 0),
            47 => (1.0, 1.0, 1.15, 1.0, 1.0, 1.0, 0, 0, 0),
            48 => (1.0, 1.0, 1.2, 1.0, 1.0, 1.0, 0, 0, 0),
            49 => (1.0, 1.0, 1.08, 1.0, 1.0, 1.0, 0, 0, 0),
            50 => (0.8, 1.0, 1.15, 1.0, 1.0, 1.0, 0, 0, 0),
            51 => (1.05, 0.9, 0.9, 1.0, 1.0, 1.0, 2, 0, 0),
            52 => (0.9, 1.0, 0.9, 1.0, 1.0, 0.9, 0, 0, 0),
            53 => (1.1, 1.0, 1.0, 1.0, 1.0, 1.0, 0, 0, 0),
            54 => (1.0, 1.15, 1.0, 1.0, 1.0, 1.0, 0, 0, 0),
            55 => (1.05, 1.15, 1.0, 1.0, 1.0, 1.0, 0, 0, 0),
            56 => (1.0, 0.8, 1.0, 1.0, 1.0, 1.0, 0, 0, 0),
            57 => (1.18, 0.9, 1.0, 1.0, 1.0, 1.0, 0, 0, 0),
            58 => (0.85, 1.0, 0.85, 1.0, 1.0, 1.0, 0, 0, 0),
            59 => (1.15, 1.15, 1.0, 1.0, 1.0, 1.0, 5, 0, 0),
            60 => (1.15, 1.0, 1.0, 1.0, 1.0, 1.0, 5, 0, 0),
            61 => (1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 5, 0, 0),
            81 => (1.15, 1.15, 0.9, 1.1, 1.0, 1.0, 5, 0, 0),
            82 => (1.15, 1.15, 0.9, 1.0, 1.1, 1.0, 5, 0, 0),
            83 => (1.15, 1.15, 0.9, 1.0, 1.0, 0.9, 5, 0, 0),
            84 => (1.17, 1.17, 1.0, 1.0, 1.0, 1.0, 8, 0, 0),
            85 => (1.15, 1.15, 1.0, 1.0, 1.0, 1.0, 0, 3, 10),
            86 => (1.1, 1.05, 1.0, 1.0, 1.0, 1.0, 0, 3, 5),
            87 => (1.15, 1.0, 1.0, 1.0, 1.0, 1.0, 0, 0, 8),
            88 => (1.1, 1.0, 1.0, 1.0, 1.0, 1.0, 0, 3, 0),
            89 => (0.95, 1.0, 1.0, 1.0, 1.0, 1.0, 0, 3, 0),
            90 => (1.1, 0.9, 1.0, 1.0, 1.0, 1.0, 0, 0, 0),
            91 => (0.95, 1.0, 1.0, 1.0, 1.0, 1.0, 0, 0, 10),
            92 => (0.7, 1.0, 1.0, 1.0, 1.0, 1.0, 0, 0, 0),
            93 => (1.0, 0.75, 1.0, 1.0, 1.0, 1.0, 0, 0, 0),
            94 => (0.85, 0.9, 1.0, 1.0, 1.0, 1.0, 0, 0, 0),
            95 => (1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 0, 0, 25),
            96 => (1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 0, 5, 0),
            97 => (1.0, 1.25, 1.0, 1.0, 1.0, 1.0, 0, 0, 0),
            _ => (1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 0, 0, 0),
        };
        Multipliers {
            dmg,
            kb,
            spd,
            size,
            shtspd,
            mcst,
            crt,
            tagdmg,
            arpen,
        }
    }

    /// `Item.cs:1046`: the value multiplier the rarity ladder and the price
    /// both read. The accessory ids (62-80) carry no stat multipliers, only
    /// these bumps (`Item.cs:1047-1062`).
    fn value_multiplier(prefix: u16, m: &Multipliers) -> f32 {
        let mut v = 1.0_f32 * m.dmg * (2.0 - m.spd) * (2.0 - m.mcst) * m.size * m.kb * m.shtspd
            * (1.0 + m.crt as f32 * 0.02)
            * (1.0 + m.arpen as f32 * 0.015)
            * (1.0 + m.tagdmg as f32 * 0.03);
        if matches!(prefix, 62 | 69 | 73 | 77) {
            v *= 1.05;
        }
        if matches!(prefix, 63 | 70 | 74 | 78 | 67) {
            v *= 1.1;
        }
        if matches!(prefix, 64 | 71 | 75 | 79 | 66) {
            v *= 1.15;
        }
        if matches!(prefix, 65 | 72 | 76 | 80 | 68) {
            v *= 1.2;
        }
        v
    }

    /// `TryGetPrefixStatMultipliersForItem`'s tail (`Item.cs:1063-1078`): a
    /// prefix whose rounded change is invisible on this item is rejected (the
    /// caller retries with a fresh roll).
    fn rejected(&self, def: &ItemDef) -> bool {
        if self.dmg != 1.0 && round_bankers((def.damage as f32 * self.dmg) as f64) as i32
            == def.damage
        {
            return true;
        }
        if self.spd != 1.0
            && round_bankers((def.use_animation as f32 * self.spd) as f64) as i32
                == def.use_animation as i32
        {
            return true;
        }
        if self.mcst != 1.0
            && round_bankers((def.mana as f32 * self.mcst) as f64) as i32 == def.mana
        {
            return true;
        }
        if self.kb != 1.0 && def.knock_back == 0.0 {
            return true;
        }
        false
    }
}

/// The effective per-instance stats of a prefixed item — `Item.Prefix`'s
/// application block (`Item.cs:576-586`) computed from the kind's defaults.
/// `prefix == 0` returns the defaults unchanged.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PrefixedStats {
    pub damage: i32,
    pub use_animation: i32,
    pub use_time: i32,
    pub reuse_delay: i32,
    pub mana: i32,
    pub knock_back: f32,
    pub scale: f32,
    pub shoot_speed: f32,
    pub crit: i32,
    /// `Item.bonusTagDamage` — the whip tag damage; no consumer yet.
    pub tag_damage: i32,
    /// `Item.armorPenetration`; no consumer yet.
    pub armor_penetration: i32,
    pub rare: i32,
    /// `Item.value` after the ×value² price multiplier.
    pub value: i32,
}

/// The prefixed stats of `kind` (`Item.Prefix` on a fresh `SetDefaults`
/// item). Returns `None` for the unregistered kinds. The base row rides
/// `registry::item` — the M4-PROJ-6 corrections seam (the corrected
/// 3377/3779/3787 staff rows must reach the prefixed math).
pub fn prefixed_stats(kind: u16, prefix: u16) -> Option<PrefixedStats> {
    let def = crate::registry::item(kind)?;
    if prefix == 0 {
        return Some(PrefixedStats {
            damage: def.damage,
            use_animation: def.use_animation as i32,
            use_time: def.use_time as i32,
            reuse_delay: def.reuse_delay,
            mana: def.mana,
            knock_back: def.knock_back,
            scale: def.scale,
            shoot_speed: def.shoot_speed,
            crit: def.crit,
            tag_damage: 0,
            armor_penetration: 0,
            rare: def.rare,
            value: def.value,
        });
    }
    let m = Multipliers::stat_switch(prefix);
    Some(PrefixedStats {
        damage: round_bankers((def.damage as f32 * m.dmg) as f64) as i32,
        use_animation: round_bankers((def.use_animation as f32 * m.spd) as f64) as i32,
        use_time: round_bankers((def.use_time as f32 * m.spd) as f64) as i32,
        reuse_delay: round_bankers((def.reuse_delay as f32 * m.spd) as f64) as i32,
        mana: round_bankers((def.mana as f32 * m.mcst) as f64) as i32,
        knock_back: def.knock_back * m.kb,
        scale: def.scale * m.size,
        shoot_speed: def.shoot_speed * m.shtspd,
        crit: def.crit + m.crt,
        tag_damage: m.tagdmg,
        armor_penetration: m.arpen,
        rare: shift_rare(def.rare, prefix, &m),
        value: ((def.value as f32) * Multipliers::value_multiplier(prefix, &m).powi(2)) as i32,
    })
}

/// The rarity ladder + clamp (`Item.cs:587-613`): the value multiplier shifts
/// `rare` by ±1/±2 and the band clamps to [-1, 11] (a quest-fish −11 stays).
/// The C# compares `(double)num2` — the f32 multiplier promoted to f64 —
/// against DOUBLE literals, so the compares run in f64: `1.05f` is
/// 1.04999995 (< 1.05, no +1 — the Quick 45 and the ×1.05 accessory tier
/// 62/69/73/77 stay put) and `0.8f` is 0.80000001 (> 0.8, the Weak-knockback
/// 56 takes −1, not −2). An f32-vs-f32-literal compare flips all six.
fn shift_rare(rare: i32, prefix: u16, m: &Multipliers) -> i32 {
    let num2 = Multipliers::value_multiplier(prefix, m) as f64;
    let mut rare = if num2 >= 1.2 {
        rare + 2
    } else if num2 >= 1.05 {
        rare + 1
    } else if num2 <= 0.8 {
        rare - 2
    } else if num2 <= 0.95 {
        rare - 1
    } else {
        rare
    };
    if rare > -11 {
        if rare < -1 {
            rare = -1;
        }
        if rare > 11 {
            rare = 11;
        }
    }
    rare
}

// --- The roll engine --------------------------------------------------------

/// The one draw the engine takes: the C# `UnifiedRandom.Next(int)` (`[0, max)`).
/// Runtime prefix rolls join their site's stream ([`UnifiedRandom`] — the
/// stage stream in-world, the bare-`Main.rand` stand-in in menus, S1-A-11);
/// the worldgen chest seam adapts its splitmix state ([`SplitMixPrefix`]).
pub trait PrefixRandom {
    fn next_max(&mut self, max: i32) -> i32;
}

/// The blanket adapter: every [`WorldGenRand`](tr_core::worldgen::chest_stock::WorldGenRand)
/// stream rolls prefixes directly — the runtime `prefix_rng` ([`UnifiedRandom`])
/// and the gen-time chest stocking's pass stream alike (S1-A-4: the same
/// stream the row draws ride, the C#'s single-`genRand` interleave at the
/// pass granularity).
impl<T: tr_core::worldgen::chest_stock::WorldGenRand + ?Sized> PrefixRandom for T {
    fn next_max(&mut self, max: i32) -> i32 {
        T::next(self, max)
    }
}

/// The still-splitmix worldgen chest stream (`chest_gen.rs`), used only through
/// [`roll_prefix_splitmix`] until S1-A-4 restores the vanilla `genRand`
/// interleave. A `u64` state, one draw = one add-mix.
struct SplitMixPrefix<'a>(&'a mut u64);

/// The splitmix step the app's per-consumer streams share (`rain.rs`'s
/// `next_u64` shape).
fn next_u64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

impl PrefixRandom for SplitMixPrefix<'_> {
    fn next_max(&mut self, max: i32) -> i32 {
        (next_u64(self.0) % max as u64) as i32
    }
}

/// `Item.Prefix(prefixWeWant)` (`Item.cs:504-618`): returns the rolled prefix
/// (0 = none). The caller reads the applied stats with [`prefixed_stats`].
///
/// The modes: an exact id applies directly (a rejected multiplier set retries
/// from the category table); `-1` is the natural spawn roll (1/4 no prefix,
/// the reduced set 1/3 none); `-2` rolls a non-zero prefix (the best-value
/// marker feeds `rolledPrefixIsTopTier`, which [`reforge_roll`] returns); `-3`
/// keeps the current prefix (a no-op here); `-4` clears to none.
pub fn roll_prefix<T: PrefixRandom + ?Sized>(
    kind: u16,
    prefix_want: i32,
    rng: &mut T,
) -> u16 {
    let Some(rollable) = rollable_prefixes(kind) else {
        // `CanHavePrefixes` gate — no roll, no draws (`Item.cs:515-518`).
        return 0;
    };
    if prefix_want == 0 || prefix_want == -3 {
        return 0;
    }
    let def = match crate::registry::item(kind) {
        Some(d) => d,
        None => return 0,
    };
    let mut rolled: i32 = prefix_want;
    loop {
        if rolled == -1 && rng.next_max(4) == 0 {
            rolled = 0;
        }
        if prefix_want < -1 {
            rolled = -1;
        }
        if (-3..=-1).contains(&rolled) {
            // `RollAPrefix` (`Item.cs:1139-1148`).
            rolled = rollable[rng.next_max(rollable.len() as i32) as usize] as i32;
        }
        if prefix_want == -1
            && in_set(REDUCED_NATURAL_CHANCE, rolled as u16)
            && rng.next_max(3) != 0
        {
            rolled = 0;
        }
        if prefix_want == -4 {
            rolled = 0;
        }
        let m = Multipliers::stat_switch(rolled as u16);
        if rolled != 0 && m.rejected(def) {
            // `flag = true; rolledPrefix = -1` — the retry rolls again
            // (including the natural 1/4 gate, `Item.cs:564-568`).
            rolled = -1;
            continue;
        }
        if prefix_want == -2 && rolled == 0 {
            rolled = -1;
            continue;
        }
        return rolled as u16;
    }
}

/// `Item.BestPrefixValue` (`Item.cs:1082-1099`): the max value multiplier
/// over the rollable table, counting only the prefixes that survive the
/// no-visible-change rejection (the vanilla takes `TryGet...`'s `out val`
/// when it returns true). 0 when the kind is not prefixable.
pub(crate) fn best_prefix_value(kind: u16) -> f32 {
    let Some(rollable) = rollable_prefixes(kind) else {
        return 0.0;
    };
    let Some(def) = crate::registry::item(kind) else {
        return 0.0;
    };
    let mut best = 0.0_f32;
    for &p in rollable {
        let m = Multipliers::stat_switch(p);
        if m.rejected(def) {
            continue;
        }
        best = best.max(Multipliers::value_multiplier(p, &m));
    }
    best
}

/// The reforge entry (`Main.ReforgeItemInReforgeSlot`, `Main.cs:41902-41933`):
/// the caller resets the stats first (the port recomputes from the defaults,
/// so the reset is the prefix field write), then rolls `Prefix(-2)`; the
/// bool is `rolledPrefixIsTopTier` — the rolled value multiplier equals the
/// table's best (`Item.cs:575`), which drives the BestReforge sound and the
/// 60-tick cooldown (`Main.cs:41907-41910`).
///
/// The `best > 0.0` guard is the port's safety net for a non-prefixable kind
/// (best 0, rolled 0): the C# never reaches `ReforgeItemInReforgeSlot` with
/// one — the slot's `CanHavePrefixes` gate (`ItemSlot.cs:1515-1520`) keeps it
/// out — so for every reachable prefixable kind `best > 0` and the equality
/// is the verbatim `num2 == num`.
pub(crate) fn reforge_roll(kind: u16, rng: &mut impl PrefixRandom) -> (u16, bool) {
    let rolled = roll_prefix(kind, -2, rng);
    let m = Multipliers::stat_switch(rolled);
    let best = best_prefix_value(kind);
    (
        rolled,
        best > 0.0 && Multipliers::value_multiplier(rolled, &m) == best,
    )
}

// --- The reforge presentation tail (S1-P-i45) -------------------------------
//
// `Main.ReforgeItemInReforgeSlot` (`Main.cs:41902-41933`) plays a sound and
// spawns particles AFTER the `Prefix(-2)` roll, all drawing the shared
// `Main.rand`. The port consumes those draws on the bare-`Main.rand` stand-in
// (`NamedRngs::global`) in the C#'s order so the shared-stream interleave
// stays exact; the particle VISUALS are the documented skip (the
// `npc_ai_empress.rs::do_magic_effect_draws` convention — the port has no
// `ParticleOrchestrator` renderer). The sounds play through the existing
// `Audio::play_use_sound` façade (the audio-only `GetRandomPitch` draw rides
// `sound_style_random`, never the gameplay stream).

/// `Utils.NextVector2Circular` (`Utils.cs:1266-1269`): the unit draw
/// (`NextVector2Unit`, one `NextFloat`) THEN a second `NextFloat` radius
/// scale — two gameplay draws. The vector value is unused (the visuals are
/// skipped); only the stream advances.
fn reforge_circular_draws(rng: &mut UnifiedRandom) {
    rng.next_float(); // NextVector2Unit's angle, Utils.cs:1261-1264
    rng.next_float(); // the radius scale, Utils.cs:1268
}

/// `Dust.NewDust`'s RNG consumption (`Dust.cs:170-175`, `:190`) under the
/// documented pool assumption (no skip draws, no early-out): `Next(max(w,5)-4)`,
/// `Next(max(h,5)-4)`, `Next(-20,21)` ×2, `Next(3)`, `Next(-20,21)` — six
/// draws, in order. `NewDustPerfect` forwards w/h = 0 → clamped 5
/// (`Dust.cs:54-63`); `CloneDust` re-rolls a fresh `NewDust` (`Dust.cs:269`).
fn reforge_new_dust_draws(rng: &mut UnifiedRandom) {
    rng.next_max(1); // :170 — position.X, max(0,5)-4 = 1
    rng.next_max(1); // :171 — position.Y
    rng.next_range(-20, 21); // :172 — velocity.X
    rng.next_range(-20, 21); // :173 — velocity.Y
    rng.next_max(3); // :175 — frame.Y
    rng.next_range(-20, 21); // :190 — scale
}

/// `ParticleOrchestrator.Spawn_RainbowRodHit`'s RNG consumption
/// (`ParticleOrchestrator.cs:3444-3492`): 2 head floats (`:3446`/`:3448`),
/// then 6 sparkle iterations (`num2 = 6`, `:3447`/`:3449`) each drawing
/// `NextFloatDirection` (`:3451`) + `NextFloat` scale (`:3452`) + `NextFloat`
/// angle (`:3453`) + `NextVector2Circular(8,8)` (`:3457`, 2) + 2 colour
/// `NextFloat`s (`:3461`) = 7, then 12 dust iterations (`:3476`) each drawing
/// 2 colour `NextFloat`s (`:3478`) + `NewDust`'s 6 + `NextVector2Circular(1,1)`
/// (`:3479`, 2) + `NextFloatDirection` (`:3480`) + `NextFloat` scale (`:3482`)
/// + `NextFloat` fadeIn (`:3483`) + `CloneDust`'s `NewDust` 6 (`:3486`) = 19.
/// Total 2 + 6×7 + 12×19 = 272 draws (the settings' own circular is drawn by
/// the caller, `Main.cs:41914`/`:41919`, BEFORE this spawn).
fn reforge_rainbow_rod_hit_draws(rng: &mut UnifiedRandom) {
    rng.next_float(); // :3446 — num
    rng.next_float(); // :3448 — num3
    for _ in 0..6 {
        rng.next_float_direction(); // :3451
        rng.next_float(); // :3452 — vector2 scale
        rng.next_float(); // :3453 — f angle
        reforge_circular_draws(rng); // :3457 — NextVector2Circular(8,8)
        rng.next_float(); // :3461 — colour hue
        rng.next_float(); // :3461 — colour lightness
    }
    for _ in 0..12 {
        rng.next_float(); // :3478 — colour hue
        rng.next_float(); // :3478 — colour lightness
        reforge_new_dust_draws(rng); // :3478 — NewDust(..., 0, 0, 267)
        reforge_circular_draws(rng); // :3479 — NextVector2Circular(1,1)
        rng.next_float_direction(); // :3480
        rng.next_float(); // :3482 — scale
        rng.next_float(); // :3483 — fadeIn
        reforge_new_dust_draws(rng); // :3486 — CloneDust → NewDust
    }
}

/// `ParticleOrchestrator.Spawn_BestReforge`'s RNG consumption
/// (`ParticleOrchestrator.cs:3069-3087`): 8 iterations (`:3073`) each drawing
/// `NextVector2Circular(3,4)` (`:3075`, 2) = 16 draws. The settings' own
/// circular is drawn by the caller (`Main.cs:41925`) BEFORE this spawn.
fn reforge_best_reforge_particle_draws(rng: &mut UnifiedRandom) {
    for _ in 0..8 {
        reforge_circular_draws(rng); // :3075
    }
}

/// The reforge presentation tail (`Main.ReforgeItemInReforgeSlot`,
/// `Main.cs:41906-41932`): the draws + sounds that follow the `Prefix(-2)`
/// roll, in the C#'s order on the shared `Main.rand` (the port's `global()`).
/// `top_tier` selects the branch:
/// - top: `BestReforge` sound (`:41909`, a trackable — no gameplay draw), the
///   60-tick cooldown (`:41910`, set by the caller), then two `RainbowRodHit`
///   spawns (settings circular `:41914`/`:41919` + the spawn body) and three
///   `BestReforge` spawns (settings circular `:41925` + the spawn body);
/// - non-top: `Item37` sound (`:41931`, case-2's one `Next(-6,7)` pitch draw).
/// `PopupText.NewText` (`:41906`) draws no `Main.rand` at spawn for the reforge
/// contexts (`PopupText.cs:194-412` — the effects are `ItemCraft`-only), so it
/// adds no draw. The particle visuals are the documented skip.
pub(crate) fn reforge_presentation_tail(top_tier: bool, rng: &mut UnifiedRandom, audio: &mut crate::audio::Audio) {
    if top_tier {
        // `:41909` — SoundID.BestReforge (trackable id 42, no gameplay draw;
        // the GetRandomPitch NextFloat rides the audio-only stream).
        if let Some(idx) = crate::soundid::SOUND_STYLES
            .iter()
            .position(|s| s.name == "BestReforge")
        {
            audio.play_use_sound(idx as u16, -1.0, -1.0, 0.0, rng);
        }
        // `:41911-41915` — RainbowRodHit #1: the settings' MovementVector
        // circular (`:41914`) then the spawn body.
        reforge_circular_draws(rng); // :41914 — NextVector2Circular(16,16)
        reforge_rainbow_rod_hit_draws(rng);
        // `:41916-41920` — RainbowRodHit #2.
        reforge_circular_draws(rng); // :41919 — NextVector2Circular(5,5)
        reforge_rainbow_rod_hit_draws(rng);
        // `:41921-41927` — three BestReforge spawns.
        for _ in 0..3 {
            reforge_circular_draws(rng); // :41925 — NextVector2Circular(16,16)
            reforge_best_reforge_particle_draws(rng);
        }
    } else {
        // `:41931` — SoundID.Item37 (case 2: the GetRandomPitch audio draw,
        // then ONE Main.rand.Next(-6,7) pitch roll after the attenuation gate).
        if let Some(idx) = crate::soundid::SOUND_STYLES
            .iter()
            .position(|s| s.name == "Item37")
        {
            audio.play_use_sound(idx as u16, -1.0, -1.0, 0.0, rng);
        }
    }
}

/// The worldgen chest seam's entry (`chest_gen.rs`): the same engine over the
/// splitmix `chest_rng` stream, kept byte-identical until S1-A-4 puts the chest
/// loot back on the vanilla `WorldGen.genRand` interleave. Do not use it for
/// runtime prefix rolls — those take [`roll_prefix`] with their site's
/// stream (the stage stream in-world, `global()` in menus).
pub(crate) fn roll_prefix_splitmix(kind: u16, prefix_want: i32, rng: &mut u64) -> u16 {
    roll_prefix(kind, prefix_want, &mut SplitMixPrefix(rng))
}

// --- The equipment pass -----------------------------------------------------

/// `Player.GrantPrefixBenefits` (`Player.cs:13953-14044`): the accessory
/// prefixes 62-80 grant flat player stats on equip. The caller gates on the
/// slot holding an accessory (the grant pass loops the accessory slots).
pub(crate) fn grant_prefix_benefits(prefix: u16, p: &mut Player) {
    match prefix {
        62 => p.stat_defense += 1,
        63 => p.stat_defense += 2,
        64 => p.stat_defense += 3,
        65 => p.stat_defense += 4,
        66 => p.mana_max += 20,
        67 => {
            p.melee_crit += 2;
            p.ranged_crit += 2;
            p.magic_crit += 2;
        }
        68 => {
            p.melee_crit += 4;
            p.ranged_crit += 4;
            p.magic_crit += 4;
        }
        69 => {
            p.melee_damage += 0.01;
            p.ranged_damage += 0.01;
            p.magic_damage += 0.01;
            p.minion_damage += 0.01;
        }
        70 => {
            p.melee_damage += 0.02;
            p.ranged_damage += 0.02;
            p.magic_damage += 0.02;
            p.minion_damage += 0.02;
        }
        71 => {
            p.melee_damage += 0.03;
            p.ranged_damage += 0.03;
            p.magic_damage += 0.03;
            p.minion_damage += 0.03;
        }
        72 => {
            p.melee_damage += 0.04;
            p.ranged_damage += 0.04;
            p.magic_damage += 0.04;
            p.minion_damage += 0.04;
        }
        73 => p.move_speed += 0.01,
        74 => p.move_speed += 0.02,
        75 => p.move_speed += 0.03,
        76 => p.move_speed += 0.04,
        77 => p.melee_speed_bonus += 0.01,
        78 => p.melee_speed_bonus += 0.02,
        79 => p.melee_speed_bonus += 0.03,
        80 => p.melee_speed_bonus += 0.04,
        _ => {}
    }
}

// --- Tests ------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::itemdata;

    /// The C# `Main.rand` stand-in: `new UnifiedRandom(seed)` (`S1-A-5`).
    fn test_rng(seed: i32) -> UnifiedRandom {
        UnifiedRandom::new(seed)
    }

    /// The exact C# stream: 20 natural rolls of the Iron Pickaxe (kind 1) from
    /// `UnifiedRandom::new(seed)` land the ids the decompiled `Item.Prefix`
    /// draws (`Item.cs:529-574`). Generated by a `dotnet` oracle that runs the
    /// decompiled `UnifiedRandom` through the same `Next(4)` / `Next(len)` /
    /// `Next(3)` order with the vanilla Iron Pickaxe tables — this is the
    /// test that fails if the engine ever goes back to a stand-in stream.
    #[test]
    fn natural_roll_matches_the_csharp_stream() {
        for (seed, expected) in [
            (
                0,
                [
                    0, 0, 38, 11, 57, 81, 0, 81, 13, 0, 0, 0, 39, 0, 57, 2, 14, 5, 59, 14,
                ],
            ),
            (
                42,
                [
                    6, 0, 7, 0, 10, 13, 0, 0, 0, 0, 0, 0, 0, 6, 43, 8, 0, 0, 0, 6,
                ],
            ),
            (
                -7,
                [
                    50, 43, 0, 0, 60, 5, 37, 14, 2, 40, 36, 0, 1, 44, 0, 0, 0, 0, 0, 5,
                ],
            ),
            (
                123_456_789,
                [
                    0, 59, 4, 0, 81, 53, 0, 0, 0, 36, 42, 5, 0, 59, 42, 6, 7, 50, 15, 45,
                ],
            ),
        ] {
            let mut rng = test_rng(seed);
            let got: Vec<u16> = (0..20).map(|_| roll_prefix(1, -1, &mut rng)).collect();
            assert_eq!(got, expected, "seed {seed}");
        }
    }

    /// The landed distribution over many seeds, again vs the C#: seeds `0..2000`
    /// x 20 natural rolls of the Iron Pickaxe = 40 000 landings, tallied by
    /// prefix id. The C# oracle produced this exact histogram (same loop, same
    /// `UnifiedRandom`), so this is a deterministic equality check, not a
    /// tolerance band: the rejected invisible-damage ids (3/12/46/51/55 — table
    /// positions 2/11/30/35/20) never land; 0 is the gate + reduced-set funnel.
    #[test]
    fn natural_roll_distribution_matches_the_csharp_over_many_seeds() {
        let expected: [(u16, u32); 36] = [
            (0, 17695),
            (1, 823),
            (2, 848),
            (4, 854),
            (5, 804),
            (6, 840),
            (7, 282),
            (8, 280),
            (9, 261),
            (10, 261),
            (11, 283),
            (13, 819),
            (14, 814),
            (15, 850),
            (36, 800),
            (37, 804),
            (38, 858),
            (39, 269),
            (40, 273),
            (41, 269),
            (42, 828),
            (43, 820),
            (44, 804),
            (45, 884),
            (47, 274),
            (48, 265),
            (49, 262),
            (50, 852),
            (53, 847),
            (54, 816),
            (56, 280),
            (57, 810),
            (59, 804),
            (60, 817),
            (61, 855),
            (81, 795),
        ];
        let mut counts = std::collections::BTreeMap::new();
        for seed in 0..2000 {
            let mut rng = test_rng(seed);
            for _ in 0..20 {
                *counts.entry(roll_prefix(1, -1, &mut rng)).or_insert(0u32) += 1;
            }
        }
        let got: Vec<(u16, u32)> = counts.into_iter().collect();
        assert_eq!(got, expected.to_vec());
    }

    #[test]
    fn tables_are_sorted_and_prefix_lists_verbatim() {
        // The membership sets must stay sorted for the binary searches; the
        // roll tables keep the vanilla order (the pick is an index into them).
        for set in [
            SWORDS_HAMMERS_AXES_PICKS,
            SPEARS_MACES_CHAINSAWS_DRILLS_PUNCH_CANNON,
            GUNS_BOWS,
            MAGIC,
            SUMMON,
            BOOMERANGS_CHAKRAMS,
            ITEMS_THAT_CAN_HAVE_LEGENDARY_2,
            CAN_GET_PREFIXES_EXCLUDED,
            REDUCED_NATURAL_CHANCE,
        ] {
            assert!(
                set.windows(2).all(|w| w[0] < w[1]),
                "a membership set lost its sort order"
            );
        }
        assert_eq!(PREFIXES_FOR_SWORDS.len(), 40);
        assert_eq!(PREFIXES_FOR_SPEARS.len(), 14);
        assert_eq!(PREFIXES_FOR_GUNS_BOWS.len(), 35);
        assert_eq!(PREFIXES_FOR_MAGIC.len(), 36);
        assert_eq!(PREFIXES_FOR_SUMMONS.len(), 22);
        assert_eq!(PREFIXES_FOR_BOOMERANGS.len(), 14);
        assert_eq!(PREFIXES_FOR_TERRARIAN.len(), 15);
        assert_eq!(PREFIXES_FOR_ACCESSORIES.len(), 19);
    }

    #[test]
    fn category_resolution_order() {
        // 56 Wooden Boomerang sits in both Swords and Boomerangs? No — it is
        // only a boomerang; the Swords set's early check takes 1909 (Blood
        // Butcherer is a sword). 3389 Terrarian: Legendary2 beats accessory.
        assert_eq!(rollable_prefixes(1909), Some(PREFIXES_FOR_SWORDS));
        // 55/119 are boomerangs only (the vanilla Swords set omits them —
        // the Swords overlap I suspected does not exist).
        assert_eq!(rollable_prefixes(55), Some(PREFIXES_FOR_BOOMERANGS));
        assert_eq!(rollable_prefixes(119), Some(PREFIXES_FOR_BOOMERANGS));
        assert_eq!(rollable_prefixes(3389), Some(PREFIXES_FOR_TERRARIAN));
        assert_eq!(rollable_prefixes(986), Some(PREFIXES_FOR_GUNS_BOWS));
        assert_eq!(rollable_prefixes(1260), Some(PREFIXES_FOR_MAGIC));
        assert_eq!(rollable_prefixes(1157), Some(PREFIXES_FOR_SUMMONS));
        // A dirt block (2) and a torch (8): no table, no draws.
        assert_eq!(rollable_prefixes(2), None);
        assert_eq!(rollable_prefixes(8), None);
        // Accessories: 950 (Hermes Boots) prefixes; 267 (Guide Voodoo Doll —
        // an accessory) is on the denial list.
        assert_eq!(rollable_prefixes(950), Some(PREFIXES_FOR_ACCESSORIES));
        assert_eq!(rollable_prefixes(267), None);
        // The `!vanity` gate (`IsAPrefixableAccessory`, `Item.cs:1150-1157`):
        // the Jungle Rose (208), the Diamond Ring (2276), the Winter Cape
        // (2287) and the Solar Monolith (3539) are vanity accessories — the
        // denial list does NOT carry them, the flag refuses the table (and
        // with it every draw: `CanHavePrefixes` returns before any roll).
        for vanity in [208u16, 2276, 2287, 3539, 4822, 5075] {
            assert_eq!(rollable_prefixes(vanity), None, "vanity {vanity}");
        }
        // The functional pair still prefixes: the Aglet (285) and the
        // Band of Regeneration (49).
        assert_eq!(rollable_prefixes(285), Some(PREFIXES_FOR_ACCESSORIES));
        assert_eq!(rollable_prefixes(49), Some(PREFIXES_FOR_ACCESSORIES));
    }

    #[test]
    fn vanity_accessories_have_no_prefix_table() {
        // `IsAPrefixableAccessory` (`Item.cs:1150-1157`): `accessory &&
        // !vanity && CanGetPrefixes[type]`. The Jungle Rose (208) and the
        // voice-change items (5499 family, `DefaultToVoiceOverrideAccessory`
        // sets `vanity = true`, `Item.cs:47794-47795`) are vanity accessories
        // OUTSIDE the denial list — the old port let them roll; the C# never
        // does. The gate reads the generated `vanity` flag verbatim.
        let rose = crate::registry::item(208).unwrap();
        assert!(rose.accessory && rose.vanity);
        assert_eq!(rollable_prefixes(208), None);
        assert_eq!(rollable_prefixes(5499), None);
        assert_eq!(rollable_prefixes(5534), None);
        // The Whoopie Cushion (215) — vanity, not on the denial list either.
        assert_eq!(rollable_prefixes(215), None);
        // A NON-vanity accessory still rolls (950 Ice Skates).
        assert_eq!(rollable_prefixes(950), Some(PREFIXES_FOR_ACCESSORIES));
        // And the gate consumes no draws for the vanity kinds — the
        // `CanHavePrefixes` early return (`Item.cs:515-518`).
        let mut rng = test_rng(9);
        let before = rng.peek();
        for kind in [208, 215, 5499, 5534] {
            assert_eq!(roll_prefix(kind, -1, &mut rng), 0);
            assert_eq!(roll_prefix(kind, -2, &mut rng), 0);
        }
        assert_eq!(rng.peek(), before, "the vanity gate draws nothing");
    }

    #[test]
    fn the_roll_engine_reads_the_corrected_registry_row() {
        // The M4-PROJ-6 seam: `roll_prefix`/`best_prefix_value` read
        // `registry::item` (the corrections included), not the raw generated
        // row. The Bejeweled Staff (6171) is the corrections band's zero row
        // in `itemdata` (damage -1, mana 0) but the corrected row carries
        // damage 27 / mana 15 — the invisible-change rejection must read the
        // CORRECTED stats. Adept (28: dmg 1.15, kb 1.05, mcst 0.85) on 27
        // damage rounds 31.05 → 31 ≠ 27 (visible); on the raw -1 damage it
        // would round -1 → -1 (invisible → rejected). The exact-id mode
        // applies it only through the corrected row.
        assert_eq!(crate::registry::item(6171).unwrap().damage, 27);
        let mut rng = test_rng(5);
        assert_eq!(roll_prefix(6171, 28, &mut rng), 28);
        // The best value over the magic table also reads the corrected row:
        // Mythical (83) clears the rejection (27×1.15 → 31 visible, 15×0.9
        // → 14 visible after banker's rounding of 13.5) and its multiplier
        // 1.15×1.1×1.1×(2−0.9)×(2−0.9)×1.1 is the table's max.
        let best = best_prefix_value(6171);
        let m = Multipliers::stat_switch(83);
        assert_eq!(best, Multipliers::value_multiplier(83, &m));
    }

    #[test]
    fn accessory_reforge_rolls_the_accessory_pool_and_flags_the_top_tier() {
        // The accessory reforge (`Prefix(-2)` on a non-vanity accessory):
        // every landing sits in the 62-80 pool, never 0, and the top tier is
        // the ×1.2 value group (65/72/76/80/68, `Item.cs:1059-1062`) — the
        // accessory ids carry no stat multipliers, so none is ever rejected
        // and the roll takes exactly ONE `Next(19)` per attempt.
        let mut rng = test_rng(0xACCE55);
        let mut tops = 0;
        for _ in 0..500 {
            let (p, top) = reforge_roll(950, &mut rng);
            assert_ne!(p, 0, "Prefix(-2) never lands none");
            assert!(PREFIXES_FOR_ACCESSORIES.contains(&p));
            if top {
                tops += 1;
                assert!(matches!(p, 65 | 72 | 76 | 80 | 68), "the ×1.2 tier");
            }
        }
        assert!(tops > 0, "the seeded stream must land the top tier");
        // One draw per roll: the accessory pool has no rejection, so 100
        // rolls consume exactly 100 draws (the stream state matches a
        // 100-draw replay).
        let mut a = test_rng(77);
        let mut b = test_rng(77);
        for _ in 0..100 {
            reforge_roll(950, &mut a);
            b.next_max(19);
        }
        assert_eq!(a.peek(), b.peek(), "one Next(19) per accessory reforge");
    }

    #[test]
    fn natural_roll_is_quarter_no_prefix() {
        // Deterministic distribution: 20 000 natural rolls of the Iron
        // Pickaxe (a sword-table kind) land ~1/4 on no prefix and never on an
        // off-table id.
        let mut rng = test_rng(0xC0FFEE);
        let (mut none, mut total, mut reduced) = (0, 0, 0);
        for _ in 0..20_000 {
            let p = roll_prefix(1, -1, &mut rng);
            total += 1;
            if p == 0 {
                none += 1;
            }
            if in_set(REDUCED_NATURAL_CHANCE, p) {
                reduced += 1;
            }
            assert!(PREFIXES_FOR_SWORDS.contains(&p) || p == 0);
        }
        assert_eq!(total, 20_000);
        // The bare 1/4 gate is the entry share; every rejected pick
        // (dmg 1.05 on damage 5 rounds back to 5, mana prefixes on 0 mana,
        // ...) re-enters through the same gate, so the landing share runs
        // well above 25% for low-stat items (vanilla behaves the same —
        // `Prefix(-1)`'s retry loop re-rolls the natural gate).
        assert!(
            none > 6_000 && none < 12_000,
            "no-prefix share {none} must sit in the rejection-funnel band"
        );
        // The reduced set (18 of the 40 sword-table ids) survives the 1/3
        // gate AND absorbs the rejection retries, so the share runs far past
        // the naive 1/18×1/3 — pin it visible but below half the landings.
        assert!(reduced > 500 && reduced < 10_000, "reduced share {reduced}");
    }

    #[test]
    fn non_prefixable_kinds_consume_no_draws() {
        // The `CanHavePrefixes` early return keeps the stream intact — the
        // state after 100 dirt-drop rolls equals the state after none.
        let mut rng = test_rng(7);
        let before = rng.peek();
        for _ in 0..100 {
            assert_eq!(roll_prefix(2, -1, &mut rng), 0);
        }
        assert_eq!(rng.peek(), before, "the gate must consume no draws");
    }

    #[test]
    fn stat_switch_spot_checks() {
        // Heavy (38): kb ×1.15; Godly (59): kb+dmg ×1.15, crit +5.
        let iron = itemdata::ITEMS[1]; // Iron Pickaxe: damage 5, kb 2.0, anim 20
        let s = prefixed_stats(1, 38).unwrap();
        assert_eq!(s.knock_back, iron.knock_back * 1.15);
        assert_eq!(s.damage, 5);
        let s = prefixed_stats(1, 59).unwrap();
        assert_eq!(s.knock_back, iron.knock_back * 1.15);
        assert_eq!(s.damage, round_bankers(5.0 * 1.15) as i32);
        assert_eq!(s.crit, 5);
        // Nimble (47): swing ×1.15 → useAnimation 20 → 23, useTime 13 → 15.
        let s = prefixed_stats(1, 47).unwrap();
        assert_eq!(s.use_animation, round_bankers(20.0 * 1.15) as i32);
        assert_eq!(s.use_time, round_bankers(13.0 * 1.15) as i32);
        // Keen (44): animation ×0.9 → 18, crit +3.
        let s = prefixed_stats(1, 44).unwrap();
        assert_eq!(s.use_animation, 18);
        assert_eq!(s.crit, 3);
    }

    #[test]
    fn invisible_changes_are_rejected_and_rolled_again() {
        // The Iron Pickaxe (damage 5, kb 2.0): Shoddy (39, dmg 0.7) rounds
        // 5×0.7 = 3.5 → banker's 4 ≠ 5 → visible, so it applies.
        let s = prefixed_stats(1, 39).unwrap();
        assert_eq!(s.damage, round_bankers(5.0 * 0.7) as i32);
        assert_eq!(s.damage, 4);
    }

    #[test]
    fn rejection_retry_never_lands_the_invisible_prefix() {
        // The Iron Pickaxe's mana is 0 — a mana-cost prefix (29 Mystic,
        // mcst 1.1) rounds 0×1.1 = 0 == 0 → invisible → rejected. The exact
        // mode applies regardless of category but still honours the
        // rejection: the retry picks from the swords table, never landing 29.
        let mut rng = test_rng(3);
        for _ in 0..50 {
            assert_ne!(roll_prefix(1, 29, &mut rng), 29, "0 mana ×1.1 is invisible");
        }
    }

    #[test]
    fn value_and_rare_shift() {
        // Godly (59) on the Iron Pickaxe: value 2000 copper — the multiplier
        // squared then truncated.
        let m = Multipliers::stat_switch(59);
        let vm = Multipliers::value_multiplier(59, &m);
        let s = prefixed_stats(1, 59).unwrap();
        assert_eq!(s.value, (2000.0_f32 * vm * vm) as i32);
        assert!(vm >= 1.2, "Godly must clear the +2 rarity bar");
        assert_eq!(s.rare, 2);
        // Shoddy (39): dmg 0.7 × kb 0.8 = 0.56 ≤ 0.8 → rare −2, and the
        // clamp (`rare < -1 → -1`, `Item.cs:598-600`) lifts the pickaxe's
        // 0-rare to −1.
        let m = Multipliers::stat_switch(39);
        let vm = Multipliers::value_multiplier(39, &m);
        assert!((vm - 0.7 * 0.8).abs() < 1e-6, "Shoddy value {vm} = 0.7×0.8");
        let s = prefixed_stats(1, 39).unwrap();
        assert_eq!(s.rare, -1);
        // The clamp: a legendary bump on a rare-10 item lands on 11.
        assert_eq!(shift_rare(10, 59, &Multipliers::stat_switch(59)), 11);
    }

    #[test]
    fn accessory_prefixes_bump_value_only() {
        // 66 (the ×1.15 value tier — 64/71/75/79/66) grants no item stats:
        // the Ice Skates' (950) stats ride unchanged, the value ×1.3225,
        // and the ladder's ≥1.05 bar shifts the rare +1 (1.15f = 1.14999998
        // clears the double 1.05).
        let skates = itemdata::ITEMS[950];
        let s = prefixed_stats(950, 66).unwrap();
        assert_eq!(s.damage, skates.damage);
        assert_eq!(s.crit, skates.crit);
        assert_eq!(s.value, (skates.value as f32 * 1.15 * 1.15) as i32);
        assert_eq!(s.rare, skates.rare + 1);
    }

    #[test]
    fn rare_ladder_compares_the_promoted_double_not_the_f32_literal() {
        // `Item.cs:587-602` compares `(double)num2` against DOUBLE literals.
        // The ×1.05 tier's f32 product is 1.04999995 — BELOW the double 1.05
        // — so the Quick (45) and the Hard/Jagged/Brisk/Wild accessories
        // (62/69/73/77) shift the rare by 0, not +1. An f32-vs-1.05f compare
        // (1.05f rounds to the same 1.04999995) would say `>=` and bump.
        for p in [45u16, 62, 69, 73, 77] {
            let m = Multipliers::stat_switch(p);
            let vm = Multipliers::value_multiplier(p, &m);
            assert!(
                (vm as f64) < 1.05,
                "prefix {p}: the f32 product {vm} promotes below the double 1.05"
            );
            assert_eq!(shift_rare(3, p, &m), 3, "prefix {p}: no rare shift");
        }
        // The Weak-knockback (56): kb 0.8f = 0.80000001 — ABOVE the double
        // 0.8 — so the ≤0.8 −2 arm refuses and the ≤0.95 −1 arm takes it.
        let m = Multipliers::stat_switch(56);
        let vm = Multipliers::value_multiplier(56, &m);
        assert!((vm as f64) > 0.8);
        assert_eq!(shift_rare(3, 56, &m), 2, "56: −1, not −2");
        // The Sluggish (48): spd 1.2 → vm = 2 − 1.2f = 0.79999995, BELOW the
        // double 0.8 — the −2 arm takes it (both compare forms agree here;
        // the pin keeps the pair honest).
        let m = Multipliers::stat_switch(48);
        assert_eq!(shift_rare(3, 48, &m), 1, "48: −2");
        // The ×1.2 tier (65/68/72/76/80): 1.2f = 1.20000005 clears the
        // double 1.2 → +2, capped at 11.
        for p in [65u16, 68, 72, 76, 80] {
            let m = Multipliers::stat_switch(p);
            assert_eq!(shift_rare(3, p, &m), 5, "prefix {p}: +2");
        }
        // The ×1.15 tier (64/66/71/75/79): 1.15f = 1.14999998 clears the
        // double 1.05 but not 1.2 → +1.
        for p in [64u16, 66, 71, 75, 79] {
            let m = Multipliers::stat_switch(p);
            assert_eq!(shift_rare(3, p, &m), 4, "prefix {p}: +1");
        }
        // The clamp floor and the quest-fish exemption still hold.
        assert_eq!(shift_rare(0, 39, &Multipliers::stat_switch(39)), -1);
        assert_eq!(shift_rare(-11, 39, &Multipliers::stat_switch(39)), -11);
    }

    #[test]
    fn the_rejection_and_best_value_read_the_corrected_registry_row() {
        // `roll_prefix`/`best_prefix_value` resolve the base row through
        // `registry::item` — the M4-PROJ-6 corrections seam. The Bejeweled
        // Staff (6171) is the band's zero row in the generated table
        // (damage −1, kb 0); the correction carries damage 27 / kb 6
        // (`Item.cs:47550-47555`). The raw row would reject EVERY
        // knockback-only prefix (kb 0) and every damage row rounding back
        // to −1; the corrected row keeps them.
        let corrected = crate::registry::item(6171).unwrap();
        assert_eq!(corrected.damage, 27);
        // Heavy (38, kb ×1.15) survives on the corrected kb 6 — the raw
        // row's kb 0 would reject it (`Item.cs:1075-1078`).
        let mut rng = test_rng(5);
        assert_eq!(roll_prefix(6171, 38, &mut rng), 38);
        // The best value scans the corrected rows too: the staff's magic
        // table top (83, Mythical) beats the identity 0 the raw-row
        // rejection funnel would leave for the kb-only tiers.
        let best = best_prefix_value(6171);
        let m83 = Multipliers::stat_switch(83);
        assert!(best >= Multipliers::value_multiplier(83, &m83));
    }

    #[test]
    fn grant_prefix_benefits_sums() {
        let w = crate::test_world_helpers::ground();
        let mut p = crate::player::Player::spawn(&w);
        let base = (p.stat_defense, p.mana_max, p.melee_crit, p.move_speed);
        grant_prefix_benefits(65, &mut p); // Warding: defense +4
        grant_prefix_benefits(68, &mut p); // Arcane-crit tier: crit +4
        grant_prefix_benefits(76, &mut p); // Swift tier: moveSpeed +4%
        grant_prefix_benefits(80, &mut p); // Light tier: meleeSpeed +4%
        grant_prefix_benefits(66, &mut p); // Arcane: mana +20
        grant_prefix_benefits(72, &mut p); // Menacing tier: dmg +4%
        assert_eq!(p.stat_defense, base.0 + 4);
        assert_eq!(p.melee_crit, base.2 + 4);
        assert_eq!(p.ranged_crit, base.2 + 4);
        assert_eq!(p.magic_crit, base.2 + 4);
        assert!((p.melee_damage - 1.04).abs() < 1e-6);
        assert!((p.ranged_damage - 1.04).abs() < 1e-6);
        assert!((p.magic_damage - 1.04).abs() < 1e-6);
        assert!((p.minion_damage - 1.04).abs() < 1e-6);
        assert!((p.move_speed - (base.3 + 0.04)).abs() < 1e-6);
        assert!((p.melee_speed_bonus - 0.04).abs() < 1e-6);
        assert_eq!(p.mana_max, base.1 + 20);
    }

    #[test]
    fn best_prefix_value_matches_the_visible_table() {
        // The Iron Pickaxe: 81 (the swords' top universal tier — kb 1.15,
        // dmg 1.15, crt 5, spd 0.9, size 1.1) beats Godly (59, 1.4547):
        // 1.15·(2−0.9)·1.1·1.15·1.1 = 1.7602. The invisible ones (dmg 1.05
        // on damage 5, mana tiers on 0 mana) drop out of the max too.
        let best = best_prefix_value(1);
        let m = Multipliers::stat_switch(81);
        let legendary = Multipliers::value_multiplier(81, &m);
        assert_eq!(best, legendary);
        // A non-prefixable kind: 0 (vanilla returns 0 for a null table).
        assert_eq!(best_prefix_value(2), 0.0);
    }

    #[test]
    fn reforge_roll_never_lands_none_and_flags_the_top_tier() {
        let mut rng = test_rng(0xBEEF);
        let mut tops = 0;
        for _ in 0..500 {
            let (p, top) = reforge_roll(1, &mut rng);
            assert_ne!(p, 0, "Prefix(-2) never lands none");
            assert!(PREFIXES_FOR_SWORDS.contains(&p));
            if top {
                tops += 1;
                assert_eq!(p, 81, "the pickaxe's top tier is 81");
            }
        }
        assert!(tops > 0, "the seeded stream must land the top tier");
        // The reforge of a non-prefixable kind keeps nothing.
        let (p, top) = reforge_roll(2, &mut rng);
        assert_eq!((p, top), (0, false));
    }

    #[test]
    fn exact_and_special_modes() {
        let mut rng = test_rng(11);
        // An exact prefix applies regardless of category when visible.
        assert_eq!(roll_prefix(1, 5, &mut rng), 5);
        // -3 keeps (no-op), 0 refuses, -4 clears, -2 never rolls 0.
        assert_eq!(roll_prefix(1, -3, &mut rng), 0);
        assert_eq!(roll_prefix(1, 0, &mut rng), 0);
        assert_eq!(roll_prefix(1, -4, &mut rng), 0);
        for _ in 0..100 {
            assert_ne!(roll_prefix(1, -2, &mut rng), 0);
        }
    }

    #[test]
    fn the_roll_engine_reads_the_corrected_registry_seam() {
        // M4-PROJ-6: kind 6171 (Bejeweled Staff) is a MAGIC-table kind whose
        // generated row is a placeholder (damage −1, mana 0) while
        // `registry::item` carries the corrected stats (damage 27, mana 15).
        // The roll engine's rejection + the best-value marker must read the
        // SAME corrected seam `prefixed_stats` reads — the raw placeholder
        // would reject every dmg multiplier (round(−1×m) == −1) and every
        // mana multiplier (round(0×m) == 0), leaving only the crit-only ids.
        assert_eq!(rollable_prefixes(6171), Some(PREFIXES_FOR_MAGIC));
        let corrected = crate::registry::item(6171).unwrap();
        assert_eq!(corrected.damage, 27);
        assert_eq!(corrected.mana, 15);
        // The best value is computed over the corrected stats: the magic
        // top tier 83 (dmg 1.15, kb 1.15, crt 5, spd 0.9, mcst 0.9) survives
        // the rejection on damage 27 / mana 15 and beats the crit-only ids.
        let best = best_prefix_value(6171);
        let m83 = Multipliers::stat_switch(83);
        assert!(!m83.rejected(corrected), "83 is visible on the corrected staff");
        assert_eq!(best, Multipliers::value_multiplier(83, &m83));
        // A reforge never lands none; the top-tier flag fires exactly when
        // the rolled multiplier equals the best (the verbatim `num2 == num`,
        // `Item.cs:575`) — checked per roll, no fragile "the seed must land
        // it" assumption.
        let mut rng = test_rng(0x5EED);
        for _ in 0..400 {
            let (p, top) = reforge_roll(6171, &mut rng);
            assert_ne!(p, 0);
            assert!(PREFIXES_FOR_MAGIC.contains(&p));
            let mp = Multipliers::stat_switch(p);
            assert_eq!(top, Multipliers::value_multiplier(p, &mp) == best);
            if top {
                assert_eq!(p, 83, "83 is the staff's unique top tier");
            }
        }
    }

    #[test]
    fn vanity_accessories_are_not_prefixable() {
        // `IsAPrefixableAccessory` (`Item.cs:1150-1157`): `accessory &&
        // !vanity && CanGetPrefixes[type]`. The Jungle Rose (208) is an
        // accessory with `vanity = true` (`Item.cs:3987-3994`), so it has no
        // rollable table even though it is absent from the denial list.
        assert!(itemdata::ITEMS[208].accessory);
        assert!(itemdata::ITEMS[208].vanity);
        assert_eq!(rollable_prefixes(208), None);
        assert!(!can_have_prefixes(208));
        // A non-vanity accessory (950 Ice Skates) still prefixes.
        assert!(!itemdata::ITEMS[950].vanity);
        assert_eq!(rollable_prefixes(950), Some(PREFIXES_FOR_ACCESSORIES));
        // The denial-list guide doll (267) stays excluded regardless.
        assert_eq!(rollable_prefixes(267), None);
    }

    #[test]
    fn the_reforge_tail_consumes_the_cited_draws() {
        // The top-tier tail (`Main.cs:41909-41927`): BestReforge sound (no
        // gameplay draw — trackable), then 2×(settings circular 2 +
        // RainbowRodHit body 272) + 3×(settings circular 2 + BestReforge body
        // 16) = 2×274 + 3×18 = 602 gameplay draws. The non-top tail
        // (`Main.cs:41931`): Item37's case-2 pitch roll = 1 draw. Each helper
        // is replayed against a hand-built draw sequence on a twin stream —
        // state equality pins both the count AND the order.
        // RainbowRodHit body: 2 + 6×7 + 12×19 = 272 (ParticleOrchestrator.cs
        // :3446-3491).
        let mut a = test_rng(0x7A11);
        let mut b = test_rng(0x7A11);
        reforge_rainbow_rod_hit_draws(&mut a);
        b.next_float(); // :3446
        b.next_float(); // :3448
        for _ in 0..6 {
            b.next_float_direction(); // :3451
            b.next_float(); // :3452
            b.next_float(); // :3453
            b.next_float(); // :3457 circular unit
            b.next_float(); // :3457 circular radius
            b.next_float(); // :3461 hue
            b.next_float(); // :3461 lightness
        }
        for _ in 0..12 {
            b.next_float(); // :3478 hue
            b.next_float(); // :3478 lightness
            for _ in 0..2 {
                // NewDust + CloneDust's NewDust, 6 each.
                b.next_max(1);
                b.next_max(1);
                b.next_range(-20, 21);
                b.next_range(-20, 21);
                b.next_max(3);
                b.next_range(-20, 21);
            }
            b.next_float(); // :3479 circular unit
            b.next_float(); // :3479 circular radius
            b.next_float_direction(); // :3480
            b.next_float(); // :3482 scale
            b.next_float(); // :3483 fadeIn
        }
        assert_eq!(a, b, "Spawn_RainbowRodHit = 272 draws in the cited order");
        // Spawn_BestReforge: 8×NextVector2Circular(2) = 16.
        let mut a = test_rng(123);
        let mut b = test_rng(123);
        reforge_best_reforge_particle_draws(&mut a);
        for _ in 0..16 {
            b.next_float();
        }
        assert_eq!(a, b, "Spawn_BestReforge = 8×NextVector2Circular(2) = 16");
        // The settings circular is 2 draws.
        let mut a = test_rng(99);
        let mut b = test_rng(99);
        reforge_circular_draws(&mut a);
        b.next_float();
        b.next_float();
        assert_eq!(a, b, "NextVector2Circular = unit + radius = 2");
        // NewDust is 6 draws in the cited order.
        let mut a = test_rng(7);
        let mut b = test_rng(7);
        reforge_new_dust_draws(&mut a);
        b.next_max(1);
        b.next_max(1);
        b.next_range(-20, 21);
        b.next_range(-20, 21);
        b.next_max(3);
        b.next_range(-20, 21);
        assert_eq!(a, b, "NewDust = 6 draws (Dust.cs:170-175,:190)");
    }

    #[test]
    fn the_reforge_tail_branches_on_top_tier() {
        // The non-top branch plays Item37 (one case-2 pitch draw on the
        // gameplay stream); the top branch plays BestReforge (no gameplay
        // sound draw) + the particle draws. Both run on the caller's stream.
        let mut audio = crate::audio::Audio::new(None);
        let mut rng = test_rng(0xB0B);
        let before = rng.peek();
        reforge_presentation_tail(false, &mut rng, &mut audio);
        assert_ne!(rng.peek(), before, "Item37's case-2 pitch roll drew");
        // The top branch draws far more (the particle tail).
        let mut rng_top = test_rng(0xB0B);
        reforge_presentation_tail(true, &mut rng_top, &mut audio);
        // The two branches diverge (different draw counts).
        assert_ne!(rng, rng_top, "the top-tier tail draws the particles");
    }
}
