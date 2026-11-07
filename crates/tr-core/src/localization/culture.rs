//! `GameCulture`: the twelve shipped languages and their legacy ids.

/// `GameCulture.CultureName` plus the culture names and legacy ids of `GameCulture`.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum GameCulture {
    English,
    German,
    Italian,
    French,
    Spanish,
    Russian,
    Chinese,
    Portuguese,
    Polish,
    Japanese,
    Korean,
    ChineseTraditional,
}

/// The twelve shipped cultures, in the order `GameCulture` registers them.
pub const CULTURES: [GameCulture; 12] = [
    GameCulture::English,
    GameCulture::German,
    GameCulture::Italian,
    GameCulture::French,
    GameCulture::Spanish,
    GameCulture::Russian,
    GameCulture::Chinese,
    GameCulture::Portuguese,
    GameCulture::Polish,
    GameCulture::Japanese,
    GameCulture::Korean,
    GameCulture::ChineseTraditional,
];

impl GameCulture {
    /// `GameCulture.DefaultCulture`: English.
    pub const DEFAULT: GameCulture = GameCulture::English;

    /// `CultureInfo.Name`.
    pub fn name(self) -> &'static str {
        match self {
            GameCulture::English => "en-US",
            GameCulture::German => "de-DE",
            GameCulture::Italian => "it-IT",
            GameCulture::French => "fr-FR",
            GameCulture::Spanish => "es-ES",
            GameCulture::Russian => "ru-RU",
            GameCulture::Chinese => "zh-Hans",
            GameCulture::Portuguese => "pt-BR",
            GameCulture::Polish => "pl-PL",
            GameCulture::Japanese => "ja-JP",
            GameCulture::Korean => "ko-KR",
            GameCulture::ChineseTraditional => "zh-Hant",
        }
    }

    /// `GameCulture.LegacyId`.
    pub fn legacy_id(self) -> i32 {
        match self {
            GameCulture::English => 1,
            GameCulture::German => 2,
            GameCulture::Italian => 3,
            GameCulture::French => 4,
            GameCulture::Spanish => 5,
            GameCulture::Russian => 6,
            GameCulture::Chinese => 7,
            GameCulture::Portuguese => 8,
            GameCulture::Polish => 9,
            GameCulture::Japanese => 10,
            GameCulture::Korean => 11,
            GameCulture::ChineseTraditional => 12,
        }
    }

    /// `GameCulture.FromLegacyId`: ids below 1 clamp to 1, unknown ids fall back to
    /// the default culture.
    pub fn from_legacy_id(id: i32) -> GameCulture {
        let id = id.max(1);
        CULTURES
            .iter()
            .copied()
            .find(|c| c.legacy_id() == id)
            .unwrap_or(GameCulture::DEFAULT)
    }

    /// The `Language.*` row key of the language screens (`Main.cs:48464-48476`
    /// names the twelve explicitly — `Language.English`, `Language.German`,
    /// …, `Language.ChineseTraditional`; the values live in the base
    /// `Terraria.Localization.Content.<culture>.json` and are localized in
    /// every culture).
    pub fn language_key(self) -> &'static str {
        match self {
            GameCulture::English => "Language.English",
            GameCulture::German => "Language.German",
            GameCulture::Italian => "Language.Italian",
            GameCulture::French => "Language.French",
            GameCulture::Spanish => "Language.Spanish",
            GameCulture::Russian => "Language.Russian",
            GameCulture::Chinese => "Language.Chinese",
            GameCulture::Portuguese => "Language.Portuguese",
            GameCulture::Polish => "Language.Polish",
            GameCulture::Japanese => "Language.Japanese",
            GameCulture::Korean => "Language.Korean",
            GameCulture::ChineseTraditional => "Language.ChineseTraditional",
        }
    }

    /// `GameCulture.FromName`.
    pub fn from_name(name: &str) -> GameCulture {
        CULTURES
            .iter()
            .copied()
            .find(|c| c.name() == name)
            .unwrap_or(GameCulture::DEFAULT)
    }

    /// `GameCulture.FromCultureName`.
    pub fn from_culture_name(index: usize) -> GameCulture {
        CULTURES.get(index).copied().unwrap_or(GameCulture::DEFAULT)
    }

    /// `LanguageManager.GetLanguageFilesForCulture` filter: the manifest resource names
    /// that belong to this culture.
    pub fn matches_resource(name: &str, culture: GameCulture) -> bool {
        let prefix = format!("Terraria.Localization.Content.{}", culture.name());
        name.starts_with(&prefix) && name.ends_with(".json")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cultures_match_the_original_ids_and_names() {
        assert_eq!(GameCulture::DEFAULT, GameCulture::English);
        assert_eq!(GameCulture::English.legacy_id(), 1);
        assert_eq!(GameCulture::ChineseTraditional.legacy_id(), 12);
        assert_eq!(GameCulture::from_name("ru-RU"), GameCulture::Russian);
        assert_eq!(GameCulture::from_legacy_id(7), GameCulture::Chinese);
        assert_eq!(GameCulture::from_legacy_id(0), GameCulture::English);
        assert_eq!(GameCulture::from_legacy_id(9999), GameCulture::English);
        assert_eq!(GameCulture::from_name("xx-XX"), GameCulture::English);
        assert!(GameCulture::matches_resource(
            "Terraria.Localization.Content.en-US.Items.json",
            GameCulture::English
        ));
        assert!(!GameCulture::matches_resource(
            "Terraria.Localization.Content.en-US.Items.json",
            GameCulture::German
        ));
    }
}
