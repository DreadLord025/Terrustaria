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
/// emitted (the 38 micro seeds arrive with M5-SEED-2).
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
        assert_eq!(flags, SeedFlags::from(SeedFlags::FOR_THE_WORTHY));
        assert_eq!(SeedFlags::from_sum(flags.sum()), flags);
        assert_eq!(SeedFlags::from_seed_text("05162020").sum(), 1);
        // The serialised paste text carries the secrets then the seed.
        assert_eq!(serialize_seeds(flags, "999"), "fortheworthy|999");
    }

    #[test]
    fn the_server_config_line_resolves_by_name_and_value() {
        assert_eq!(
            flag_for_server_config("fortheworthy", true),
            SeedFlags::from(SeedFlags::FOR_THE_WORTHY)
        );
        assert_eq!(flag_for_server_config("drunk", true), SeedFlags::from(SeedFlags::DRUNK));
        assert_eq!(flag_for_server_config("5162020", true), SeedFlags::from(SeedFlags::DRUNK));
        assert_eq!(
            flag_for_server_config("zenith", true),
            SeedFlags::from(SeedFlags::ZENITH | SeedFlags::EVERYTHING_DEPS)
        );
        assert_eq!(flag_for_server_config("drunk", false), SeedFlags::none());
        assert_eq!(flag_for_server_config("null", true), SeedFlags::none());
    }
}
