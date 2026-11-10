//! The secret-seed ("special world") framework — the port of
//! `WorldBuilding/WorldGenerationOptions` and the `WorldSeedOption_*` classes
//! (`decompiled/Terraria/WorldBuilding/`), plus the seed-text translation
//! (`IO/WorldFileData.cs:380-397` + `ReLogic.Utilities/Crc32.cs`).
//!
//! The C# has two seed registries: the nine classic *options* (this module —
//! drunk / anniversary / For The Worthy / don't starve / not the bees / no
//! traps / remix / everything / skyblock, `WorldGenerationOptions.cs:30-41`)
//! and the 38 modular 1.4.5 *micro* seeds (the BCrypt-hashed
//! `WorldGen.SecretSeed`, `WorldGen.cs:3022-3058` — M5-SEED-2's scope).
//!
//! Everything here is pure: a seed *text* resolves to the option (by numeric
//! value or by the alnum-lowered name, `WorldGenerationOptions.cs:66-83`),
//! the option resolves to the boolean world flags, and the flags carry the
//! serialised bitmask (`WorldFileData.cs:157-197`).

/// CRC-32 (reflected, poly `0xEDB88320`, init `0xFFFFFFFF`, final
/// complement) — the `Crc32.Calculate` of the decompiled
/// `ReLogic.Utilities.Crc32.cs:35-43`, which feeds each UTF-16 code unit's
/// **low byte** (`(num ^ c) & 0xFF`). The port's UI copy lives in
/// `tr-app/src/ui/world_creation.rs`; this one is the engine-side twin so
/// the worldgen and save layers need no UI dependency (the check value
/// `crc32("123456789") == 0xCBF4_3926` pins the table in both).
pub fn crc32(value: &str) -> i32 {
    const TABLE: [u32; 256] = {
        let mut table = [0u32; 256];
        let mut i = 0usize;
        while i < 256 {
            let mut entry = i as u32;
            let mut k = 0;
            while k < 8 {
                entry = if entry & 1 != 0 {
                    (entry >> 1) ^ 0xEDB8_8320
                } else {
                    entry >> 1
                };
                k += 1;
            }
            table[i] = entry;
            i += 1;
        }
        table
    };
    let mut num = 0xFFFF_FFFFu32;
    for c in value.chars() {
        num = (num >> 8) ^ TABLE[((num ^ (c as u32 & 0xFF)) & 0xFF) as usize];
    }
    !num as i32
}

/// `WorldFileData.TranslateSeed` (`WorldFileData.cs:386-397`): a numeric
/// text is `int.TryParse` + `Math.Abs` (`int.MinValue` → `int.MaxValue`),
/// everything else hashes through [`crc32`]. `NumberStyles.Integer`'s
/// surrounding whitespace is tolerated (`int.TryParse`'s default), which is
/// what `text.trim()` mimics for the non-numeric branch.
pub fn translate_seed(text: &str) -> i32 {
    if let Ok(v) = text.trim().parse::<i32>() {
        if v == i32::MIN {
            return i32::MAX;
        }
        return v.abs();
    }
    crc32(text)
}

/// `Regex.Replace(processedSeed.ToLower(), "[^a-z0-9]+", "")` — the
/// normaliser every name comparison runs through
/// (`WorldGenerationOptions.cs:68`, `WorldGen.SecretSeed.Check`'s
/// `WorldGen.cs:468-481`).
pub fn normalise_seed_text(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
        }
    }
    out
}

/// The nine classic options — `AWorldGenerationOption.SpecialSeedNames` /
/// `SpecialSeedValues` (per class, e.g.
/// `WorldSeedOption_Anniversary.cs:11-12`) in the static-ctor registration
/// order of `WorldGenerationOptions.cs:30-41` (the order the lookup walks,
/// so the first match wins).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SeedOptionId {
    Normal,
    NotTheBees,
    Drunk,
    Anniversary,
    DontStarve,
    ForTheWorthy,
    NoTraps,
    Remix,
    Everything,
    Skyblock,
}

/// One option's name table + numeric aliases.
#[derive(Clone, Copy, Debug)]
pub struct SeedOptionDecl {
    pub id: SeedOptionId,
    /// `ServerConfigName` (the `seed_<name>=1` server-config key,
    /// `WorldGenerationOptions.cs:18`).
    pub server_name: &'static str,
    /// `SpecialSeedNames` — already normalised.
    pub names: &'static [&'static str],
    /// `SpecialSeedValues` — the numeric seed aliases.
    pub values: &'static [i32],
}

pub const OPTION_TABLE: &[SeedOptionDecl] = &[
    SeedOptionDecl {
        id: SeedOptionId::Normal,
        server_name: "null",
        names: &[],
        values: &[],
    },
    // `WorldSeedOption_NotTheBees.cs:11-12`
    SeedOptionDecl {
        id: SeedOptionId::NotTheBees,
        server_name: "notthebees",
        names: &["notthebees"],
        values: &[],
    },
    // `WorldSeedOption_Drunk.cs:11-12` — 5162020 is the 05162020 seed
    // (the world-creation screen strips the leading zero, `int.Parse`).
    SeedOptionDecl {
        id: SeedOptionId::Drunk,
        server_name: "drunk",
        names: &[],
        values: &[5162020],
    },
    // `WorldSeedOption_Anniversary.cs:11-12`
    SeedOptionDecl {
        id: SeedOptionId::Anniversary,
        server_name: "celebration",
        names: &["celebrationmk10"],
        values: &[5162021, 5162011],
    },
    // `WorldSeedOption_DontStarve.cs:11-12`
    SeedOptionDecl {
        id: SeedOptionId::DontStarve,
        server_name: "theconstant",
        names: &["constant", "theconstant", "eye4aneye", "eyeforaneye"],
        values: &[],
    },
    // `WorldSeedOption_ForTheWorthy.cs:11-12`
    SeedOptionDecl {
        id: SeedOptionId::ForTheWorthy,
        server_name: "fortheworthy",
        names: &["fortheworthy"],
        values: &[],
    },
    // `WorldSeedOption_NoTraps.cs:11-12`
    SeedOptionDecl {
        id: SeedOptionId::NoTraps,
        server_name: "notraps",
        names: &["notraps"],
        values: &[],
    },
    // `WorldSeedOption_Remix.cs:11-12`
    SeedOptionDecl {
        id: SeedOptionId::Remix,
        server_name: "remix",
        names: &["dontdigup"],
        values: &[],
    },
    // `WorldSeedOption_Everything.cs:44-45`
    SeedOptionDecl {
        id: SeedOptionId::Everything,
        server_name: "zenith",
        names: &["getfixedboi"],
        values: &[],
    },
    // `WorldSeedOption_Skyblock.cs:11-12`
    SeedOptionDecl {
        id: SeedOptionId::Skyblock,
        server_name: "skyblock",
        names: &["skyblock"],
        values: &[],
    },
];

/// `WorldGenerationOptions.GetOptionFromSeedText`
/// (`WorldGenerationOptions.cs:66-83`): the translated numeric seed matches
/// `SpecialSeedValues`, the normalised text matches `SpecialSeedNames`; the
/// walk is the registration order. `None` = a classic world (the `Normal`
/// option).
pub fn option_for_seed_text(text: &str) -> Option<SeedOptionId> {
    let num = translate_seed(text);
    let norm = normalise_seed_text(text);
    for opt in OPTION_TABLE {
        if opt.values.iter().any(|&v| v == num) {
            return Some(opt.id);
        }
        if opt.names.iter().any(|&n| n == norm) {
            return Some(opt.id);
        }
    }
    None
}

/// The nine classic world flags as the `serializedSeedsSum` bitmask
/// (`WorldFileData.cs:157-197`): Drunk 1, NotTheBees 2, ForTheWorthy 4,
/// Anniversary 8, DontStarve 16, Remix 32, NoTraps 64, Zenith (Everything)
/// 128, Skyblock 256. Plain `u16` bits (the port adds no bitflags
/// dependency); the combinators cover the reads the ports need.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SeedFlags {
    bits: u16,
}

impl SeedFlags {
    pub const DRUNK: u16 = 1;
    pub const NOT_THE_BEES: u16 = 2;
    pub const FOR_THE_WORTHY: u16 = 4;
    pub const TENTH_ANNIVERSARY: u16 = 8;
    pub const DONT_STARVE: u16 = 16;
    pub const REMIX: u16 = 32;
    pub const NO_TRAPS: u16 = 64;
    pub const ZENITH: u16 = 128;
    pub const SKYBLOCK: u16 = 256;

    pub const fn none() -> Self {
        SeedFlags { bits: 0 }
    }

    pub const fn bits(self) -> u16 {
        self.bits
    }

    pub const fn contains(self, bit: u16) -> bool {
        self.bits & bit == bit
    }

    pub const fn intersects(self, bits: u16) -> bool {
        self.bits & bits != 0
    }

    pub fn insert(&mut self, bits: u16) {
        self.bits |= bits;
    }
}

impl std::ops::BitOr<u16> for SeedFlags {
    type Output = SeedFlags;
    fn bitor(self, rhs: u16) -> SeedFlags {
        SeedFlags {
            bits: self.bits | rhs,
        }
    }
}

impl std::ops::BitOrAssign<u16> for SeedFlags {
    fn bitor_assign(&mut self, rhs: u16) {
        self.bits |= rhs;
    }
}

impl From<u16> for SeedFlags {
    fn from(bits: u16) -> Self {
        SeedFlags { bits }
    }
}

impl SeedFlags {

    /// The resolved flags of a seed text — the `Reset()` read
    /// (`WorldGen.cs:11115-11123`) plus `WorldSeedOption_Everything`'s
    /// dependency cascade (`WorldSeedOption_Everything.cs:15-58`: selecting
    /// `getfixedboi` force-enables remix, drunk, not-the-bees, no-traps,
    /// don't starve, anniversary and For The Worthy; skyblock is NOT a
    /// dependency).
    pub fn from_seed_text(text: &str) -> Self {
        let Some(option) = option_for_seed_text(text) else {
            return Self::none();
        };
        let mut flags = match option {
            SeedOptionId::NotTheBees => SeedFlags::from(Self::NOT_THE_BEES),
            SeedOptionId::Drunk => SeedFlags::from(Self::DRUNK),
            SeedOptionId::Anniversary => SeedFlags::from(Self::TENTH_ANNIVERSARY),
            SeedOptionId::DontStarve => SeedFlags::from(Self::DONT_STARVE),
            SeedOptionId::ForTheWorthy => SeedFlags::from(Self::FOR_THE_WORTHY),
            SeedOptionId::NoTraps => SeedFlags::from(Self::NO_TRAPS),
            SeedOptionId::Remix => SeedFlags::from(Self::REMIX),
            SeedOptionId::Everything => SeedFlags::from(Self::ZENITH | Self::EVERYTHING_DEPS),
            SeedOptionId::Skyblock => SeedFlags::from(Self::SKYBLOCK),
            SeedOptionId::Normal => Self::none(),
        };
        // `WorldSeedOption_Everything.UpdateDependentState` keeps the
        // dependency set live for the other selections too: if zenith got
        // enabled by any other means it carries its seven.
        if flags.contains(Self::ZENITH) {
            flags |= Self::EVERYTHING_DEPS;
        }
        flags
    }

    /// The seven `Everything` dependencies (`WorldSeedOption_Everything.cs:23-37`).
    pub const EVERYTHING_DEPS: u16 = Self::REMIX
        | Self::DRUNK
        | Self::NOT_THE_BEES
        | Self::NO_TRAPS
        | Self::DONT_STARVE
        | Self::TENTH_ANNIVERSARY
        | Self::FOR_THE_WORTHY;

    /// The `serializedSeedsSum` bitmask round-trip (`SaveWorldFlags`,
    /// `WorldFile.cs:1238-1250` — the nine booleans written in the
    /// `Main.*` order).
    pub fn sum(self) -> u16 {
        self.bits()
    }

    pub fn from_sum(bits: u16) -> Self {
        SeedFlags { bits }
    }

    // --- the derived combo flags -------------------------------------------
    //
    // `WorldGen.cs:11125-11126` and `GenVars.cs:284-286` — the Reset
    // combinations the passes read.

    /// `GenVars.notTheBeesAndForTheWorthyNoCelebration`.
    pub fn not_the_bees_and_for_the_worthy_no_celebration(self) -> bool {
        self.contains(Self::NOT_THE_BEES)
            && self.contains(Self::FOR_THE_WORTHY)
            && !self.contains(Self::TENTH_ANNIVERSARY)
    }

    /// `GenVars.noTrapsAndForTheWorthyNoCelebration`.
    pub fn no_traps_and_for_the_worthy_no_celebration(self) -> bool {
        self.contains(Self::NO_TRAPS)
            && self.contains(Self::FOR_THE_WORTHY)
            && !self.contains(Self::TENTH_ANNIVERSARY)
    }

    /// `GenVars.flipInfections` — drunk && for-the-worthy && !remix
    /// (`WorldGen.cs:11127`): the corruption/crimson tile and wall ids swap
    /// (`:11298-11304`).
    pub fn flip_infections(self) -> bool {
        self.contains(Self::DRUNK)
            && self.contains(Self::FOR_THE_WORTHY)
            && !self.contains(Self::REMIX)
    }
}

/// `GetFullSeedText`'s serialised-seeds suffix (`WorldFileData.cs:123-155`)
/// — the `"secret|secret|...|seed"` paste form. Only the classic options are
/// emitted (the 37 micro seeds arrive with M5-SEED-2, which joins them
/// through `resolve_seed_text`).
pub fn serialize_seeds(flags: SeedFlags, seed_text: &str) -> String {
    let mut parts: Vec<&str> = Vec::new();
    if flags.contains(SeedFlags::NOT_THE_BEES) {
        parts.push("notthebees");
    }
    if flags.contains(SeedFlags::DRUNK) {
        parts.push("05162020");
    }
    if flags.contains(SeedFlags::FOR_THE_WORTHY) {
        parts.push("fortheworthy");
    }
    if flags.contains(SeedFlags::TENTH_ANNIVERSARY) {
        parts.push("celebrationmk10");
    }
    if flags.contains(SeedFlags::DONT_STARVE) {
        parts.push("theconstant");
    }
    if flags.contains(SeedFlags::REMIX) {
        parts.push("dontdigup");
    }
    if flags.contains(SeedFlags::NO_TRAPS) {
        parts.push("notraps");
    }
    if flags.contains(SeedFlags::ZENITH) {
        parts.push("getfixedboi");
    }
    if flags.contains(SeedFlags::SKYBLOCK) {
        parts.push("skyblock");
    }
    parts.push(seed_text);
    parts.join("|")
}

/// The flag of a single option id — the `Reset()` per-option read
/// (`WorldGen.cs:11115-11123`). `Everything` carries its seven dependencies
/// (`WorldSeedOption_Everything.OnEnabledStateChanged`), so it resolves to
/// the full mix rather than the zenith bit alone.
pub fn flags_for_option(id: SeedOptionId) -> SeedFlags {
    let bit = match id {
        SeedOptionId::Normal => 0,
        SeedOptionId::NotTheBees => SeedFlags::NOT_THE_BEES,
        SeedOptionId::Drunk => SeedFlags::DRUNK,
        SeedOptionId::Anniversary => SeedFlags::TENTH_ANNIVERSARY,
        SeedOptionId::DontStarve => SeedFlags::DONT_STARVE,
        SeedOptionId::ForTheWorthy => SeedFlags::FOR_THE_WORTHY,
        SeedOptionId::NoTraps => SeedFlags::NO_TRAPS,
        SeedOptionId::Remix => SeedFlags::REMIX,
        SeedOptionId::Everything => SeedFlags::ZENITH | SeedFlags::EVERYTHING_DEPS,
        SeedOptionId::Skyblock => SeedFlags::SKYBLOCK,
    };
    SeedFlags::from(bit)
}

/// `WorldGenerationOptions.TryEnablingFlagFrom` (`:92-110`) — the
/// `seed_<server-name>=<0|1>` server-config line; the `<name>` is the
/// `ServerConfigName`.
pub fn flag_for_server_config(name: &str, enabled: bool) -> SeedFlags {
    if !enabled {
        return SeedFlags::none();
    }
    let key = normalise_seed_text(name);
    if key == "null" {
        return SeedFlags::none();
    }
    let flags = OPTION_TABLE
        .iter()
        .find(|opt| opt.server_name == key)
        .map(|opt| flags_for_option(opt.id))
        .or_else(|| {
            // The value-only options (drunk / anniversary) resolve through
            // their numeric aliases: `seed_5162020=1` behaves like the seed.
            key.parse::<i32>()
                .ok()
                .and_then(option_for_seed_value)
                .map(flags_for_option)
        })
        .unwrap_or_else(SeedFlags::none);
    let mut out = flags;
    if out.contains(SeedFlags::ZENITH) {
        out |= SeedFlags::EVERYTHING_DEPS;
    }
    out
}

/// The option a numeric alias belongs to (the value table's reverse).
pub fn option_for_seed_value(value: i32) -> Option<SeedOptionId> {
    OPTION_TABLE
        .iter()
        .find(|opt| opt.values.iter().any(|&v| v == value))
        .map(|opt| opt.id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crc32_matches_the_reference_vector() {
        assert_eq!(crc32("123456789"), 0xCBF4_3926u32 as i32);
    }

    #[test]
    fn translate_seed_parses_numbers_and_hashes_names() {
        assert_eq!(translate_seed("42"), 42);
        assert_eq!(translate_seed("-5"), 5);
        assert_eq!(translate_seed("-2147483648"), 2147483647);
        assert_eq!(translate_seed(" 16 "), 16);
        // The name branch is the CRC-32 of the raw text.
        assert_eq!(translate_seed("fortheworthy"), 3784519424u32 as i32);
        assert_eq!(translate_seed("for the worthy"), 2517888731u32 as i32);
    }

    #[test]
    fn the_names_resolve_case_and_punctuation_insensitively() {
        assert_eq!(option_for_seed_text("for the worthy"), Some(SeedOptionId::ForTheWorthy));
        assert_eq!(option_for_seed_text("FOR THE WORTHY!"), Some(SeedOptionId::ForTheWorthy));
        assert_eq!(option_for_seed_text("Not.The.Bees"), Some(SeedOptionId::NotTheBees));
        assert_eq!(option_for_seed_text("dontdigup"), Some(SeedOptionId::Remix));
        assert_eq!(option_for_seed_text("constant"), Some(SeedOptionId::DontStarve));
        assert_eq!(option_for_seed_text("eye4aneye"), Some(SeedOptionId::DontStarve));
        assert_eq!(option_for_seed_text("getfixedboi"), Some(SeedOptionId::Everything));
        assert_eq!(option_for_seed_text("skyblock"), Some(SeedOptionId::Skyblock));
        assert_eq!(option_for_seed_text("celebrationmk10"), Some(SeedOptionId::Anniversary));
        assert_eq!(option_for_seed_text("seed12345"), None);
    }

    #[test]
    fn the_numeric_aliases_resolve() {
        assert_eq!(option_for_seed_text("05162020"), Some(SeedOptionId::Drunk));
        assert_eq!(option_for_seed_text("5162020"), Some(SeedOptionId::Drunk));
        assert_eq!(option_for_seed_text("5162021"), Some(SeedOptionId::Anniversary));
        assert_eq!(option_for_seed_text("5162011"), Some(SeedOptionId::Anniversary));
    }

    #[test]
    fn get_fixed_boi_cascades_all_seven_dependencies() {
        let f = SeedFlags::from_seed_text("getfixedboi");
        assert_eq!(f, SeedFlags::from(SeedFlags::EVERYTHING_DEPS | SeedFlags::ZENITH));
        assert!(!f.not_the_bees_and_for_the_worthy_no_celebration());
        // Skyblock is not a dependency.
        assert!(!f.contains(SeedFlags::SKYBLOCK));
    }

    #[test]
    fn the_combo_flags_follow_reset() {
        let bees_ftw = SeedFlags::from(SeedFlags::NOT_THE_BEES | SeedFlags::FOR_THE_WORTHY);
        assert!(bees_ftw.not_the_bees_and_for_the_worthy_no_celebration());
        assert!(!bees_ftw.no_traps_and_for_the_worthy_no_celebration());
        assert!(!bees_ftw.flip_infections());
        let drunk_ftw = SeedFlags::from(SeedFlags::DRUNK | SeedFlags::FOR_THE_WORTHY);
        assert!(drunk_ftw.flip_infections());
        assert!(!drunk_ftw.not_the_bees_and_for_the_worthy_no_celebration());
    }

    #[test]
    fn the_bitmask_round_trips() {
        let flags = SeedFlags::from_seed_text("for the worthy");
        assert_eq!(flags, SeedFlags::from(SeedFlags::from(SeedFlags::FOR_THE_WORTHY)));
        assert_eq!(SeedFlags::from_sum(flags.sum()), flags);
        assert_eq!(SeedFlags::from_seed_text("05162020").sum(), 1);
        // The serialised paste text carries the secrets then the seed.
        assert_eq!(serialize_seeds(flags, "999"), "fortheworthy|999");
    }

    #[test]
    fn the_server_config_line_resolves_by_name_and_value() {
        assert_eq!(
            flag_for_server_config("fortheworthy", true),
            SeedFlags::from(SeedFlags::from(SeedFlags::FOR_THE_WORTHY))
        );
        assert_eq!(flag_for_server_config("drunk", true), SeedFlags::from(SeedFlags::from(SeedFlags::DRUNK)));
        assert_eq!(flag_for_server_config("5162020", true), SeedFlags::from(SeedFlags::from(SeedFlags::DRUNK)));
        assert_eq!(
            flag_for_server_config("zenith", true),
            SeedFlags::from(SeedFlags::ZENITH | SeedFlags::EVERYTHING_DEPS)
        );
        assert_eq!(flag_for_server_config("drunk", false), SeedFlags::none());
        assert_eq!(flag_for_server_config("null", true), SeedFlags::none());
    }
}
// ---------------------------------------------------------------------------
// M5-SEED-2: the 37 modular micro-seeds
// ---------------------------------------------------------------------------

// The 1.4.5 modular micro-seeds - the second registry beside the classic
// options (M5-SEED-2). `WorldGen.SecretSeed` registers 37 seeds as
// `SecretSeed.Register(localization, SoundID.MenuAccept, "<base64 code>")`
// in its static ctor (`WorldGen.cs:3022-3058`); a seed text unlocks one by
// hashing it (`Secrets.ToSecret` - [`crate::bcrypt`] plus the deterministic
// shuffle) and comparing to the code. Unlike the classic options there is no
// plaintext table anywhere in the game: recognition IS the hash check, so
// the port needs the real hash, not a word list.
// Storage is a 64-bit bitset beside the classic `SeedFlags` (whose `u16` is
// the `serializedSeedsSum` mirror and must stay byte-identical, M5-SEED-1).
// `activeSecretSeedCount` (`WorldGen.cs:425-427`, maintained by `Enable`/
// `Disable`) is every `Variations` gate's input and counts the classic
// options too - [`active_secret_seed_count`] takes them.

/// The 37 micro-seed flags as a 64-bit bitset, beside the classic
/// `SeedFlags`. `Default` = a classic world, so every existing generated
/// world and pinned hash is unchanged until a caller sets the bits.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MicroSeedFlags {
    bits: u64,
}

impl MicroSeedFlags {
    /// `SecretSeed.paintEverythingGray` (registration order #1).
    pub const PAINT_EVERYTHING_GRAY: u64 = 1 << 0;
    /// `SecretSeed.paintEverythingNegative` (registration order #2).
    pub const PAINT_EVERYTHING_NEGATIVE: u64 = 1 << 1;
    /// `SecretSeed.coatEverythingEcho` (registration order #3).
    pub const COAT_EVERYTHING_ECHO: u64 = 1 << 2;
    /// `SecretSeed.coatEverythingIlluminant` (registration order #4).
    pub const COAT_EVERYTHING_ILLUMINANT: u64 = 1 << 3;
    /// `SecretSeed.noSurface` (registration order #5).
    pub const NO_SURFACE: u64 = 1 << 4;
    /// `SecretSeed.extraLivingTrees` (registration order #6).
    pub const EXTRA_LIVING_TREES: u64 = 1 << 5;
    /// `SecretSeed.extraFloatingIslands` (registration order #7).
    pub const EXTRA_FLOATING_ISLANDS: u64 = 1 << 6;
    /// `SecretSeed.errorWorld` (registration order #8).
    pub const ERROR_WORLD: u64 = 1 << 7;
    /// `SecretSeed.graveyardBloodmoonStart` (registration order #9).
    pub const GRAVEYARD_BLOODMOON_START: u64 = 1 << 8;
    /// `SecretSeed.surfaceIsInSpace` (registration order #10).
    pub const SURFACE_IS_IN_SPACE: u64 = 1 << 9;
    /// `SecretSeed.rainsForAYear` (registration order #11).
    pub const RAINS_FOR_A_YEAR: u64 = 1 << 10;
    /// `SecretSeed.biggerAbandonedHouses` (registration order #12).
    pub const BIGGER_ABANDONED_HOUSES: u64 = 1 << 11;
    /// `SecretSeed.randomSpawn` (registration order #13).
    pub const RANDOM_SPAWN: u64 = 1 << 12;
    /// `SecretSeed.addTeleporters` (registration order #14).
    pub const ADD_TELEPORTERS: u64 = 1 << 13;
    /// `SecretSeed.startInHardmode` (registration order #15).
    pub const START_IN_HARDMODE: u64 = 1 << 14;
    /// `SecretSeed.noInfection` (registration order #16).
    pub const NO_INFECTION: u64 = 1 << 15;
    /// `SecretSeed.hallowOnTheSurface` (registration order #17).
    pub const HALLOW_ON_THE_SURFACE: u64 = 1 << 16;
    /// `SecretSeed.worldIsInfected` (registration order #18).
    pub const WORLD_IS_INFECTED: u64 = 1 << 17;
    /// `SecretSeed.surfaceIsMushrooms` (registration order #19).
    pub const SURFACE_IS_MUSHROOMS: u64 = 1 << 18;
    /// `SecretSeed.surfaceIsDesert` (registration order #20).
    pub const SURFACE_IS_DESERT: u64 = 1 << 19;
    /// `SecretSeed.pooEverywhere` (registration order #21).
    pub const POO_EVERYWHERE: u64 = 1 << 20;
    /// `SecretSeed.noSpiderCaves` (registration order #22).
    pub const NO_SPIDER_CAVES: u64 = 1 << 21;
    /// `SecretSeed.actuallyNoTraps` (registration order #23).
    pub const ACTUALLY_NO_TRAPS: u64 = 1 << 22;
    /// `SecretSeed.rainbowStuff` (registration order #24).
    pub const RAINBOW_STUFF: u64 = 1 << 23;
    /// `SecretSeed.digExtraHoles` (registration order #25).
    pub const DIG_EXTRA_HOLES: u64 = 1 << 24;
    /// `SecretSeed.roundLandmasses` (registration order #26).
    pub const ROUND_LANDMASSES: u64 = 1 << 25;
    /// `SecretSeed.extraLiquid` (registration order #27).
    pub const EXTRA_LIQUID: u64 = 1 << 26;
    /// `SecretSeed.portalGunInChests` (registration order #28).
    pub const PORTAL_GUN_IN_CHESTS: u64 = 1 << 27;
    /// `SecretSeed.worldIsFrozen` (registration order #29).
    pub const WORLD_IS_FROZEN: u64 = 1 << 28;
    /// `SecretSeed.halloweenGen` (registration order #30).
    pub const HALLOWEEN_GEN: u64 = 1 << 29;
    /// `SecretSeed.endlessHalloween` (registration order #31).
    pub const ENDLESS_HALLOWEEN: u64 = 1 << 30;
    /// `SecretSeed.endlessChristmas` (registration order #32).
    pub const ENDLESS_CHRISTMAS: u64 = 1 << 31;
    /// `SecretSeed.vampirism` (registration order #33).
    pub const VAMPIRISM: u64 = 1 << 32;
    /// `SecretSeed.teamBasedSpawns` (registration order #34).
    pub const TEAM_BASED_SPAWNS: u64 = 1 << 33;
    /// `SecretSeed.dualDungeons` (registration order #35).
    pub const DUAL_DUNGEONS: u64 = 1 << 34;
    /// `SecretSeed.moreLightning` (registration order #36).
    pub const MORE_LIGHTNING: u64 = 1 << 35;
    /// `SecretSeed.noLightning` (registration order #37).
    pub const NO_LIGHTNING: u64 = 1 << 36;
}

impl MicroSeedFlags {
    pub const fn none() -> Self {
        MicroSeedFlags { bits: 0 }
    }

    pub const fn bits(self) -> u64 {
        self.bits
    }

    pub const fn contains(self, bit: u64) -> bool {
        self.bits & bit == bit
    }

    pub const fn intersects(self, bits: u64) -> bool {
        self.bits & bits != 0
    }

    pub fn insert(&mut self, bits: u64) {
        self.bits |= bits;
    }

    pub fn sum(self) -> u64 {
        self.bits()
    }

    pub fn from_sum(bits: u64) -> Self {
        MicroSeedFlags { bits }
    }

    /// `SecretSeed.activeSecretSeedCount`'s micro half (`WorldGen.cs:425-427`)
    /// - the enabled-seed population the `Variations` thresholds read.
    pub fn count(self) -> u32 {
        self.bits.count_ones()
    }
}

impl std::ops::BitOr<u64> for MicroSeedFlags {
    type Output = MicroSeedFlags;
    fn bitor(self, rhs: u64) -> MicroSeedFlags {
        MicroSeedFlags {
            bits: self.bits | rhs,
        }
    }
}

impl std::ops::BitOrAssign<u64> for MicroSeedFlags {
    fn bitor_assign(&mut self, rhs: u64) {
        self.bits |= rhs;
    }
}

impl From<u64> for MicroSeedFlags {
    fn from(bits: u64) -> Self {
        MicroSeedFlags { bits }
    }
}

/// `SecretSeed.Register` order (`WorldGen.cs:3022-3058`) == the field
/// declaration order (`:341-413`) == the `AllSecretSeeds` walk order
/// (`Register` appends, `:483-488`) — the port keeps that order so a
/// future first-match tie-break and the count semantics match. The
/// base64 `_code` strings are copied verbatim from the C#.
pub const MICRO_SEED_ORDER: &[(&str, &str)] = &[
    ("paintEverythingGray", "2htOIVagY/7JFx7acMpyUR6D3qJDr/u+"),
    ("paintEverythingNegative", "YJayFFSdWEl66+rlFoWJRNvBHJi8gHnx"),
    ("coatEverythingEcho", "5Czr2vSNyB9hJd1yob+TYo0qqH/5U2P9"),
    ("coatEverythingIlluminant", "5YXhKErRZovhjJkrP9fptrVHbNc1oSSn"),
    ("noSurface", "cptECrPRxYeNTULJULs4gVoKdRsf3c3n"),
    ("extraLivingTrees", "QQN1FbxlHeUCXPZc51GYvn8G5GXOJcny"),
    ("extraFloatingIslands", "0ebq4RCzI3PVaUPOT0f6/+vkXEaoLz2U"),
    ("errorWorld", "GkviuS3QN0pyESRJdjIs6oC8s8hOhUXw"),
    ("graveyardBloodmoonStart", "N8G20sWOkIa7ZP0rS/jopLpe9180N6Tx"),
    ("surfaceIsInSpace", "io2s6kMi4L7ZCDYZGP1Hc8nEWuYW4gp5"),
    ("rainsForAYear", "xYBNU5Soje9VhQHNQXETDKbwlc+7XZau"),
    ("biggerAbandonedHouses", "vWb/t7nNF+tnjgr5VgY2hi0HcT1j3kvC"),
    ("randomSpawn", "zSwnCH9E121+S6VQdB0k20E7IPdtobls"),
    ("addTeleporters", "+URq9gxzcyHxAXVqdwl1fz8wgPYYu0Wx"),
    ("startInHardmode", "6kX2PJe0FWt3i0fp0tVBh5jt84ozLXBo"),
    ("noInfection", "m1gQVuUnIRW083pnfFdnN3DPsg1qFYHZ"),
    ("hallowOnTheSurface", "KYvKIk2LK0oyNY86m+uPhKQ7QbzFmDsR"),
    ("worldIsInfected", "kbxnychxHNDcoyFHhxM9OJHRxis6mFF/"),
    ("surfaceIsMushrooms", "e48+tRi5DqzRkBPk3yq9udBG/kaYOQaB"),
    ("surfaceIsDesert", "eyGmBQhQ9QnE7UsIib1QmnNRVBNmQtMi"),
    ("pooEverywhere", "Iubz1XcBvsfPjSZucIJ3hCDFFEpjG57w"),
    ("noSpiderCaves", "SPlOdka0fv8wUovao6u3VB7ZS+IbcPDu"),
    ("actuallyNoTraps", "AoEz0g1XX0V/nJwcaN2RWwUf/6ghr9pT"),
    ("rainbowStuff", "6lK0Tn4t2UlklesGiJ94617yKvk01ICB"),
    ("digExtraHoles", "MucLvCERZix3rfcwUH68HDtuFYukiTv9"),
    ("roundLandmasses", "VSN8nV180t6PgabWDl4Uf55I1vu97JRD"),
    ("extraLiquid", "ZYO3rUjSeCaaBrCE8Bv0FBtkjigLMz90"),
    ("portalGunInChests", "ALdQZ+bxQA4VdfjVfdhO/sm9q3sZD9dJ"),
    ("worldIsFrozen", "eH2IYQwQyOud0hyoTPaeVsqYlAP7MvbS"),
    ("halloweenGen", "Z4Odmvd5lScy/KGXHUO2nvqA9l3KRvm8"),
    ("endlessHalloween", "KNSxbK83ZXH41aUhWLti9OFMxoMrCV1s"),
    ("endlessChristmas", "gkN386qfe3u1qqQDpGsUu3DsRkEBpD1R"),
    ("vampirism", "4eijvDtfcSl66CDifYSVP3WBZm9OLBoW"),
    ("teamBasedSpawns", "HnTdmrZ5OT1ldA3r0w3dCgrdLnJBtBSD"),
    ("dualDungeons", "ypBuvKpqKay//OvhG2COriSpGT7f4YY3"),
    ("moreLightning", "QvWX07AQaEJGhn5VUcbw4QQa0grflInc"),
    ("noLightning", "Kun6qdhFp8ia236+svHD27t2t3OMJ+0B"),
];

/// The BCrypt-derived seed code - `Utilities/Secrets.cs:16-26`: `CryptRaw`
/// over the UTF-8 bytes with the game's static salt (work factor 4), then the
/// deterministic 1000-round `Utils.Swap` shuffle (`Utils.cs:216-221`:
/// `num = i % len`, swap `bytes[num]` with `bytes[bytes[num] % len]`), then a
/// second `CryptRaw`, then standard Base64. The two golden vectors
/// (`pumpkinseason` -> halloweenGen, `arachnophobia` -> noSpiderCaves) are
/// pinned in `bcrypt.rs`'s tests - they prove the walk is the game's
/// non-NUL-terminated one, not the stock bcrypt crate's.
pub fn to_secret(text: &str) -> String {
    // `Secrets._salt` = `Convert.FromBase64String("fT2JQQzNMJl2NRoMbo9RjA==")`
    // (`Secrets.cs:13`) — the DECODED 16 bytes, not the base64 text.
    let salt: [u8; 16] = [
        0x7d, 0x3d, 0x89, 0x41, 0x0c, 0xcd, 0x30, 0x99, 0x76, 0x35, 0x1a, 0x0c, 0x6e, 0x8f, 0x51,
        0x8c,
    ];
    let mut bytes = crate::bcrypt::crypt_raw(text.as_bytes(), &salt, 4).to_vec();
    for i in 0..1000 {
        let num = i % bytes.len();
        let v = bytes[num];
        let j = (v as usize) % bytes.len();
        bytes.swap(num, j);
    }
    let out = crate::bcrypt::crypt_raw(&bytes, &salt, 4);
    // Standard Base64 (`Convert.ToBase64String`), plain alphabet - the 24-byte
    // input never pads, so no `=` is emitted.
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut s = String::with_capacity(32);
    for chunk in out.chunks(3) {
        let b = [chunk[0], *chunk.get(1).unwrap_or(&0), *chunk.get(2).unwrap_or(&0)];
        s.push(ALPHABET[(b[0] >> 2) as usize] as char);
        s.push(ALPHABET[(((b[0] & 0x03) << 4) | (b[1] >> 4)) as usize] as char);
        if chunk.len() > 1 {
            s.push(ALPHABET[(((b[1] & 0x0f) << 2) | (b[2] >> 6)) as usize] as char);
        }
        if chunk.len() > 2 {
            s.push(ALPHABET[(b[2] & 0x3f) as usize] as char);
        }
    }
    s
}

/// The registered micro-seed a normalised text unlocks - `SecretSeed.Check`
/// (`WorldGen.cs:468-481`), the hash comparison over the whole registry (the
/// registration-order walk, first match wins).
pub fn micro_seed_for_text(normalised: &str) -> Option<&'static str> {
    if normalised.is_empty() {
        return None;
    }
    let code = to_secret(normalised);
    MICRO_SEED_ORDER
        .iter()
        .find(|(_, c)| *c == code)
        .map(|(id, _)| *id)
}

/// `SecretSeed.CheckInputForSecretSeed` (`WorldGen.cs:498-525`): the
/// normalised text, then the hash match. The C# checks its `_plaintext`
/// session cache first - the port is stateless, so the hash walk plays both
/// roles (equivalent for the world-creation path, which has no session).
pub fn check_input_for_secret_seed(text: &str) -> Option<&'static str> {
    micro_seed_for_text(&normalise_seed_text(text))
}

/// The bit of a registered id (its `MICRO_SEED_ORDER` index).
pub fn bit_for_id(id: &str) -> u64 {
    let idx = MICRO_SEED_ORDER
        .iter()
        .position(|(name, _)| *name == id)
        .unwrap_or_else(|| panic!("unknown micro seed {id}"));
    1 << idx
}

/// The 37 `SecretSeed.Enable` calls of one world-creation entry - the
/// `|`-split plaintexts `FinishCreatingWorld` joins
/// (`UIWorldCreation.cs:993`) and `TryParseSecretSeed` re-splits
/// (`WorldFileData.cs:292-308`).
pub fn micro_flags_for_texts<'a>(texts: impl IntoIterator<Item = &'a str>) -> MicroSeedFlags {
    let mut flags = MicroSeedFlags::none();
    for text in texts {
        if text.is_empty() {
            continue;
        }
        if let Some(id) = check_input_for_secret_seed(text) {
            flags.insert(bit_for_id(id));
        }
    }
    flags
}

/// `SecretSeed.Variations`'s count gate - `activeSecretSeedCount` includes
/// the classic options (`Enable` is the counter for both registries), so a
/// caller passes the classic `SeedFlags` alongside.
pub fn active_secret_seed_count(micro: MicroSeedFlags, classic: SeedFlags) -> u32 {
    micro.count() + classic.sum().count_ones()
}

/// `TryApplyingCopiedSeed`'s seed-text grammar (`WorldFileData.cs:219-274`,
/// `:292-308`): the seed text is `[secret|secret|...|]<base seed text>` (the
/// classic `serializedSeedsSum` rides its own field, not the text). Returns
/// the micro flags, the classic flags of the *base* text and the base seed
/// text itself. The four leading `.`-separated paste values (size/difficulty/
/// evil/sum) stay the UI's job (a documented deviation since M5-SEED-1).
pub fn resolve_seed_text(seed_text: &str) -> (MicroSeedFlags, SeedFlags, String) {
    let mut parts = seed_text.split('|');
    let last = parts.next_back().unwrap_or("");
    let micro = micro_flags_for_texts(parts.filter(|s| !s.is_empty()));
    (micro, SeedFlags::from_seed_text(last), last.to_string())
}

#[cfg(test)]
mod micro_tests {
    use super::*;

    #[test]
    fn the_registrations_are_37_and_the_codes_are_base64() {
        assert_eq!(MICRO_SEED_ORDER.len(), 37);
        for (id, code) in MICRO_SEED_ORDER {
            assert_eq!(code.len(), 32, "{id}");
            assert!(
                code
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b == b'+' || b == b'/'),
                "{id}"
            );
        }
        // Registration order == the field declaration order (`WorldGen.cs:341-413`).
        assert_eq!(MICRO_SEED_ORDER[0].0, "paintEverythingGray");
        assert_eq!(MICRO_SEED_ORDER[4].0, "noSurface");
        assert_eq!(MICRO_SEED_ORDER[36].0, "noLightning");
    }

    #[test]
    fn the_two_known_plaintexts_resolve() {
        // The two words the game itself reveals: `SetSeedToRandomWithCurrentEvents`
        // writes "pumpkinseason|" (`WorldFileData.cs:409`) and the community-known
        // "arachnophobia" (`WorldGen.cs:3043`'s code).
        assert_eq!(check_input_for_secret_seed("pumpkin season"), Some("halloweenGen"));
        assert_eq!(check_input_for_secret_seed("ARACHNOPHOBIA"), Some("noSpiderCaves"));
    }

    #[test]
    fn a_random_word_resolves_to_nothing() {
        assert_eq!(check_input_for_secret_seed("probablynotaseed"), None);
        assert_eq!(check_input_for_secret_seed(""), None);
    }

    #[test]
    fn the_bit_theme_round_trips() {
        let f = micro_flags_for_texts(["pumpkinseason", "arachnophobia"]);
        assert!(f.contains(MicroSeedFlags::HALLOWEEN_GEN));
        assert!(f.contains(MicroSeedFlags::NO_SPIDER_CAVES));
        assert_eq!(f.count(), 2);
        assert_eq!(MicroSeedFlags::from_sum(f.sum()), f);
        assert_eq!(active_secret_seed_count(f, SeedFlags::from(SeedFlags::DRUNK)), 3);
    }

    #[test]
    fn the_seed_text_grammar_splits_secrets_from_the_base_seed() {
        let (micro, classic, base) = resolve_seed_text("pumpkinseason|arachnophobia|12345");
        assert!(micro.contains(MicroSeedFlags::HALLOWEEN_GEN));
        assert!(micro.contains(MicroSeedFlags::NO_SPIDER_CAVES));
        assert_eq!(classic, SeedFlags::none());
        assert_eq!(base, "12345");
        // A plain text resolves the classic options only.
        let (micro, classic, base) = resolve_seed_text("for the worthy");
        assert_eq!(micro, MicroSeedFlags::none());
        assert_eq!(classic, SeedFlags::from(SeedFlags::FOR_THE_WORTHY));
        assert_eq!(base, "for the worthy");
    }
}
