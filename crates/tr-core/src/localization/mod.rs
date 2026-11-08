//! `Terraria.Localization` port: `GameCulture`, `LocalizedText`, `VariableText` and
//! `LanguageManager`.
//!
//! The original ships its strings as embedded resources named
//! `Terraria.Localization.Content.<culture>.*.json`; the loader here takes the same
//! text and mirrors `LanguageManager.LoadLanguageFromFileTextJson`
//! (`Dictionary<string, Dictionary<string, string>>`), the `$` variant keys, the
//! `{$(Category.Key)}` copy commands and the `{Name}` variable substitution of
//! `VariableText`. Only the JSON path is ported — the shipped content is JSON and the
//! `.csv` path (`LoadLanguageFromFileTextCsv`) exists for mods only.
//!
//! `Lang.GetGlobalSubstitution` is modelled as a [`Substitution`] table on the
//! manager; `Lang`'s static methods are the corresponding `LanguageManager` methods.
//!
//! The layout follows those responsibilities: [`culture`] holds `GameCulture`,
//! [`format`] the `{Name}` substitution machinery and [`json`] the resource-file
//! loader.

mod culture;
mod format;
mod json;

use std::collections::HashMap;

use self::json::expand_copy_commands;

pub use culture::{CULTURES, GameCulture};
use format::VariableText;
pub use format::{Substitution, format_net};

/// `LanguageManager.VariationSeparatorSign`.
pub const VARIATION_SEPARATOR_SIGN: char = '$';

/// `LocalizedText`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct LocalizedText {
    key: String,
    /// `_value.ToString()`: the raw loaded string.
    value: String,
    /// `EnglishValue`: the value while the default culture is active.
    english_value: String,
    parsed: Option<VariableText>,
}

impl LocalizedText {
    /// `new LocalizedText(key, text)`; `english_value` is only remembered while the
    /// default culture is active (`LocalizedText.SetValue`).
    pub fn new(key: impl Into<String>, text: impl Into<String>, english: bool) -> Self {
        let key = key.into();
        let text = text.into();
        let parsed = VariableText::try_create(&text);
        LocalizedText {
            english_value: if english { text.clone() } else { String::new() },
            key,
            value: text,
            parsed,
        }
    }

    /// `LocalizedText.Empty`.
    pub fn empty() -> Self {
        LocalizedText::new("", "", true)
    }

    pub fn key(&self) -> &str {
        &self.key
    }

    /// `UnformattedValue`.
    pub fn unformatted_value(&self) -> &str {
        &self.value
    }

    /// `EnglishValue`.
    pub fn english_value(&self) -> &str {
        &self.english_value
    }

    /// `HasValue`.
    pub fn has_value(&self) -> bool {
        self.english_value != self.key
    }

    /// `SetValue`, called after a load: only the default culture records the English
    /// value.
    fn set_value(&mut self, value: impl Into<String>, english: bool) {
        let value = value.into();
        if english {
            self.english_value = value.clone();
        }
        self.parsed = VariableText::try_create(&value);
        self.value = value;
    }

    /// `SetValue(localizedText.Key)` from `SetAllTextValuesToKeys`.
    fn set_value_to_key(&mut self) {
        let key = self.key.clone();
        self.set_value(key, false);
    }

    /// `LocalizedText.Value`: the raw value with `{Name}` substitutions applied, or the
    /// original text when a condition/variable is missing.
    pub fn value_with(&self, lookup: &dyn Fn(&str) -> Option<Substitution>) -> String {
        if let Some(parsed) = &self.parsed {
            if let Some(text) = parsed.try_format(lookup) {
                return text;
            }
        }
        self.value.clone()
    }

    /// `LocalizedText.Format(params object[] args)`.
    pub fn format(&self, lookup: &dyn Fn(&str) -> Option<Substitution>, args: &[String]) -> String {
        format_net(&self.value_with(lookup), args)
    }
}

/// `LanguageManager`.
#[derive(Clone, Debug)]
pub struct LanguageManager {
    localized_texts: HashMap<String, LocalizedText>,
    category_grouped_keys: HashMap<String, Vec<String>>,
    text_variations: HashMap<String, HashMap<String, String>>,
    fallback_culture: GameCulture,
    /// `ActiveCulture` starts null in the original (the auto-property default), so the
    /// first `SetLanguage` always reloads even for the default culture.
    active_culture: Option<GameCulture>,
    substitutions: HashMap<String, Substitution>,
    /// `(manifest resource name, json text)` pairs, standing in for the embedded
    /// resources `Assembly.GetManifestResourceNames()` enumerates.
    sources: Vec<(String, String)>,
}

impl Default for LanguageManager {
    fn default() -> Self {
        Self::new()
    }
}

impl LanguageManager {
    pub fn new() -> Self {
        let mut localized_texts = HashMap::new();
        localized_texts.insert(String::new(), LocalizedText::empty());
        LanguageManager {
            localized_texts,
            category_grouped_keys: HashMap::new(),
            text_variations: HashMap::new(),
            fallback_culture: GameCulture::DEFAULT,
            active_culture: None,
            substitutions: HashMap::new(),
            sources: Vec::new(),
        }
    }

    /// `LanguageManager.ActiveCulture`; the default culture until a load has happened.
    pub fn active_culture(&self) -> GameCulture {
        self.active_culture.unwrap_or(GameCulture::DEFAULT)
    }

    pub fn fallback_culture(&self) -> GameCulture {
        self.fallback_culture
    }

    /// `LanguageManager.SetFallbackCulture`.
    pub fn set_fallback_culture(&mut self, culture: GameCulture) {
        self.fallback_culture = culture;
    }

    /// Registers one embedded-resource stand-in (`GetLanguageFilesForCulture`).
    pub fn add_source(&mut self, resource_name: impl Into<String>, text: impl Into<String>) {
        self.sources.push((resource_name.into(), text.into()));
    }

    /// The registered `(resource name, json text)` pairs — the runtime loader
    /// probes this to keep from double-registering the same files.
    pub fn sources(&self) -> &[(String, String)] {
        &self.sources
    }

    /// `Language`/`Lang.SetGlobalSubstitution`.
    pub fn set_substitution(&mut self, name: impl Into<String>, value: impl Into<Substitution>) {
        self.substitutions.insert(name.into(), value.into());
    }

    /// `Lang.RemoveGlobalSubstitution`.
    pub fn remove_substitution(&mut self, name: &str) {
        self.substitutions.remove(name);
    }

    /// `Lang.GetGlobalSubstitution` (`null` when unset).
    pub fn substitution(&self, name: &str) -> Option<Substitution> {
        self.substitutions.get(name).cloned()
    }

    /// `LanguageManager.SetLanguage`.
    pub fn set_language(&mut self, culture: GameCulture) {
        if self.active_culture != Some(culture) {
            self.reload_language(culture);
        }
    }

    /// `LanguageManager.ReloadLanguage`.
    fn reload_language(&mut self, target_culture: GameCulture) {
        if self.active_culture != Some(self.fallback_culture) {
            self.set_all_text_values_to_keys();
            if target_culture != self.fallback_culture {
                self.load_language(self.fallback_culture);
            }
        }
        self.load_language(target_culture);
        // `OnLanguageChanged` has no subscriber yet.
    }

    /// `LanguageManager.LoadLanguage`.
    fn load_language(&mut self, culture: GameCulture) {
        self.active_culture = Some(culture);
        self.text_variations.clear();
        self.load_files_for_culture(culture);
        self.process_copy_commands_in_texts();
    }

    /// `LanguageManager.LoadFilesForCulture`: the failed-file `catch`/`break` of the
    /// original is a silent skip here, so one bad file cannot abort the load.
    fn load_files_for_culture(&mut self, culture: GameCulture) {
        let files: Vec<(String, String)> = self
            .sources
            .iter()
            .filter(|(name, _)| GameCulture::matches_resource(name, culture))
            .cloned()
            .collect();
        for (name, text) in files {
            if text.len() < 2 {
                continue;
            }
            if let Err(e) = self.load_language_from_file_text_json(&text, true) {
                eprintln!("Failed to load language file: {name}: {e}");
            }
        }
    }

    /// `LanguageManager.SetAllTextValuesToKeys`.
    fn set_all_text_values_to_keys(&mut self) {
        for text in self.localized_texts.values_mut() {
            text.set_value_to_key();
        }
    }

    /// `LanguageManager.UpdateTextValue`.
    pub fn update_text_value(&mut self, key: &str, value: &str) -> bool {
        if key.contains(VARIATION_SEPARATOR_SIGN) {
            let parts: Vec<&str> = key.split(VARIATION_SEPARATOR_SIGN).collect();
            if parts.len() >= 2 {
                self.add_variant(parts[0], parts[1], value);
            }
            return true;
        }
        if let Some(text) = self.localized_texts.get_mut(key) {
            text.set_value(value, self.active_culture == Some(GameCulture::DEFAULT));
            return true;
        }
        false
    }

    /// `LanguageManager.AddVariant`.
    fn add_variant(&mut self, key: &str, variant: &str, value: &str) {
        self.text_variations
            .entry(key.to_string())
            .or_default()
            .insert(variant.to_string(), value.to_string());
    }

    /// `LanguageManager.TryGetVariation`.
    pub fn try_get_variation(&self, key: &str, variant: &str) -> Option<&str> {
        self.text_variations
            .get(key)
            .and_then(|v| v.get(variant))
            .map(String::as_str)
    }

    /// `LanguageManager.ProcessCopyCommandsInTexts`: expands `{$(Category.Key)}` until
    /// the text stops changing (the original caps this at 100 iterations).
    fn process_copy_commands_in_texts(&mut self) {
        let keys: Vec<String> = self.localized_texts.keys().cloned().collect();
        for key in keys {
            for _ in 0..100 {
                let current = self
                    .localized_texts
                    .get(&key)
                    .map(|t| t.unformatted_value().to_string())
                    .unwrap_or_default();
                let next = expand_copy_commands(&current, &self.localized_texts);
                if next == current {
                    break;
                }
                if let Some(text) = self.localized_texts.get_mut(&key) {
                    text.set_value(next, false);
                }
            }
        }
    }

    /// `LanguageManager.GetCategorySize`.
    pub fn category_size(&self, name: &str) -> usize {
        self.category_grouped_keys.get(name).map_or(0, Vec::len)
    }

    /// `LanguageManager.Exists`.
    pub fn exists(&self, key: &str) -> bool {
        self.localized_texts.contains_key(key)
    }

    /// `LanguageManager.GetText`, inserting the `key → key` placeholder of the
    /// original for unknown keys.
    pub fn get_text(&mut self, key: &str) -> &LocalizedText {
        if !self.localized_texts.contains_key(key) {
            let text = LocalizedText::new(key, key, false);
            self.localized_texts.insert(key.to_string(), text);
        }
        &self.localized_texts[key]
    }

    /// `LanguageManager.GetTextValue(key)`.
    pub fn get_text_value(&self, key: &str) -> String {
        match self.localized_texts.get(key) {
            Some(text) => text.value_with(&|name| self.substitution(name)),
            None => key.to_string(),
        }
    }

    /// `LanguageManager.GetTextValue(key, params object[] args)`.
    pub fn get_text_value_args(&self, key: &str, args: &[String]) -> String {
        match self.localized_texts.get(key) {
            Some(text) => text.format(&|name| self.substitution(name), args),
            None => key.to_string(),
        }
    }

    /// `LanguageManager.RandomFromCategory`, taking the RNG index explicitly so the
    /// caller controls determinism.
    pub fn random_from_category(&mut self, category: &str, index: usize) -> LocalizedText {
        let Some(keys) = self.category_grouped_keys.get(category) else {
            return LocalizedText::new(
                format!("{category}.RANDOM"),
                format!("{category}.RANDOM"),
                false,
            );
        };
        if keys.is_empty() {
            return LocalizedText::new(
                format!("{category}.RANDOM"),
                format!("{category}.RANDOM"),
                false,
            );
        }
        let name = keys[index % keys.len()].clone();
        self.get_text(&format!("{category}.{name}")).clone()
    }

    /// `LanguageManager.IndexedFromCategory`.
    pub fn indexed_from_category(&mut self, category: &str, index: usize) -> LocalizedText {
        let Some(keys) = self.category_grouped_keys.get(category) else {
            return LocalizedText::new(
                format!("{category}.INDEXED"),
                format!("{category}.INDEXED"),
                false,
            );
        };
        if keys.is_empty() {
            return LocalizedText::new(
                format!("{category}.INDEXED"),
                format!("{category}.INDEXED"),
                false,
            );
        }
        let name = keys[index % keys.len()].clone();
        self.get_text(&format!("{category}.{name}")).clone()
    }

    /// `LanguageManager.FindAll(LanguageSearchFilter)`.
    pub fn find_all(&self, filter: impl Fn(&str, &LocalizedText) -> bool) -> Vec<LocalizedText> {
        self.localized_texts
            .iter()
            .filter(|(k, v)| filter(k, v))
            .map(|(_, v)| v.clone())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn manager_with(json: &str) -> LanguageManager {
        let mut m = LanguageManager::new();
        m.add_source("Terraria.Localization.Content.en-US.json", json);
        m.set_language(GameCulture::English);
        m
    }

    #[test]
    fn json_creates_categories_and_english_values() {
        let json = r#"{
            "ItemName": { "DirtBlock": "Dirt Block", "StoneBlock": "Stone Block" },
            "ItemTooltip": { "Tooltip0": "-Item-" }
        }"#;
        let mut m = manager_with(json);
        assert_eq!(m.get_text_value("ItemName.DirtBlock"), "Dirt Block");
        assert_eq!(m.category_size("ItemName"), 2);
        assert_eq!(m.category_size("ItemTooltip"), 1);
        assert!(m.get_text("ItemName.DirtBlock").has_value());
        assert_eq!(
            m.get_text("ItemName.DirtBlock").english_value(),
            "Dirt Block"
        );
        // Unknown keys keep the key itself (`GetTextValue`).
        assert_eq!(m.get_text_value("Missing.Key"), "Missing.Key");
    }

    #[test]
    fn later_files_override_earlier_values() {
        let mut m = LanguageManager::new();
        m.add_source(
            "Terraria.Localization.Content.en-US.json",
            r#"{"A":{"K":"first"}}"#,
        );
        m.add_source(
            "Terraria.Localization.Content.en-US.Game.json",
            r#"{"A":{"K":"second","L":"new"}}"#,
        );
        m.set_language(GameCulture::English);
        assert_eq!(m.get_text_value("A.K"), "second");
        assert_eq!(m.get_text_value("A.L"), "new");
        assert_eq!(m.category_size("A"), 2);
    }

    #[test]
    fn dollar_keys_become_variations() {
        let json = r#"{
            "Town": {
                "Chatter$Day": "Nice day.",
                "Chatter$Night": "It is dark."
            }
        }"#;
        let m = manager_with(json);
        assert_eq!(
            m.try_get_variation("Town.Chatter", "Day"),
            Some("Nice day.")
        );
        assert_eq!(
            m.try_get_variation("Town.Chatter", "Night"),
            Some("It is dark.")
        );
        assert!(!m.exists("Town.Chatter$Day"));
    }

    #[test]
    fn copy_commands_expand_into_the_referenced_value() {
        let json = r#"{
            "Game": { "Name": "Terraria", "Title": "Welcome to {$Game.Name}!" },
            "Chain": { "B": "{$Game.Title}" }
        }"#;
        let m = manager_with(json);
        assert_eq!(m.get_text_value("Game.Title"), "Welcome to Terraria!");
        assert_eq!(m.get_text_value("Chain.B"), "Welcome to Terraria!");
    }

    #[test]
    fn format_substitutes_arguments_and_keeps_escapes() {
        let json = r#"{
            "Tooltip": {
                "UsesMana": "Uses {0} mana",
                "Range": "Restores from {0} to {1} life",
                "Braces": "{{literal}} {0}"
            }
        }"#;
        let m = manager_with(json);
        assert_eq!(
            m.get_text_value_args("Tooltip.UsesMana", &["20".into()]),
            "Uses 20 mana"
        );
        assert_eq!(
            m.get_text_value_args("Tooltip.Range", &["5".into(), "10".into()]),
            "Restores from 5 to 10 life"
        );
        assert_eq!(
            m.get_text_value_args("Tooltip.Braces", &["x".into()]),
            "{literal} x"
        );
        // A missing argument keeps the token instead of throwing.
        assert_eq!(
            m.get_text_value_args("Tooltip.UsesMana", &[]),
            "Uses {0} mana"
        );
    }

    #[test]
    fn variable_text_uses_substitutions_and_conditions() {
        let json = r#"{
            "Game": {
                "Hi": "Hello {PlayerName}",
                "NightOnly": "{?IsNight}The stars are out",
                "NotNight": "{!IsNight}The sun is out"
            }
        }"#;
        let mut m = manager_with(json);
        m.set_substitution("PlayerName", "Jeff");
        assert_eq!(m.get_text_value("Game.Hi"), "Hello Jeff");
        assert_eq!(
            m.get_text_value("Game.NightOnly"),
            "{?IsNight}The stars are out"
        );
        m.set_substitution("IsNight", true);
        assert_eq!(m.get_text_value("Game.NightOnly"), "The stars are out");
        assert_eq!(
            m.get_text_value("Game.NotNight"),
            "{!IsNight}The sun is out"
        );
        m.set_substitution("IsNight", false);
        assert_eq!(m.get_text_value("Game.NotNight"), "The sun is out");
    }

    #[test]
    fn unknown_key_becomes_its_own_placeholder_text() {
        let mut m = manager_with(r#"{"A":{"K":"v"}}"#);
        assert_eq!(m.get_text("Missing.Thing").key(), "Missing.Thing");
        assert_eq!(
            m.get_text("Missing.Thing").unformatted_value(),
            "Missing.Thing"
        );
    }

    #[test]
    fn switching_to_a_culture_reloads_fallback_then_target() {
        let mut m = LanguageManager::new();
        m.add_source(
            "Terraria.Localization.Content.en-US.Items.json",
            r#"{"ItemName":{"DirtBlock":"Dirt Block","StoneBlock":"Stone Block"}}"#,
        );
        m.add_source(
            "Terraria.Localization.Content.de-DE.Items.json",
            r#"{"ItemName":{"DirtBlock":"Erdblock"}}"#,
        );
        m.set_language(GameCulture::German);
        assert_eq!(m.get_text_value("ItemName.DirtBlock"), "Erdblock");
        // Missing in German, so the English fallback value survives.
        assert_eq!(m.get_text_value("ItemName.StoneBlock"), "Stone Block");
        // Switching back restores English even for the German override.
        m.set_language(GameCulture::English);
        assert_eq!(m.get_text_value("ItemName.DirtBlock"), "Dirt Block");
    }

    #[test]
    fn indexed_category_walks_file_order() {
        let json = r#"{"Dialogue":{"A":"first","B":"second","C":"third"}}"#;
        let mut m = manager_with(json);
        assert_eq!(
            m.indexed_from_category("Dialogue", 0).unformatted_value(),
            "first"
        );
        assert_eq!(
            m.indexed_from_category("Dialogue", 3).unformatted_value(),
            "first"
        );
        assert_eq!(
            m.indexed_from_category("Nope", 0).unformatted_value(),
            "Nope.INDEXED"
        );
    }

    /// Loads the strings the game ships in `Terraria.Localization.Content.en-US*.json`
    /// and checks a few known keys. Skipped when the decompiled resources are absent.
    #[test]
    fn shipped_english_resources_match_the_game() {
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../decompiled");
        let resources = [
            "Terraria.Localization.Content.en-US.json",
            "Terraria.Localization.Content.en-US.Items.json",
            "Terraria.Localization.Content.en-US.Game.json",
            "Terraria.Localization.Content.en-US.NPCs.json",
            "Terraria.Localization.Content.en-US.Projectiles.json",
            "Terraria.Localization.Content.en-US.Town.json",
            "Terraria.Localization.Content.en-US.Legacy.json",
        ];
        let mut manager = LanguageManager::new();
        let mut loaded = 0;
        for name in resources {
            let path = dir.join(name);
            if let Ok(text) = std::fs::read_to_string(&path) {
                manager.add_source(name, text);
                loaded += 1;
            }
        }
        if loaded == 0 {
            eprintln!("skipping: {} has no localization resources", dir.display());
            return;
        }
        manager.set_language(GameCulture::English);
        assert_eq!(manager.get_text_value("ItemName.DirtBlock"), "Dirt Block");
        assert_eq!(manager.get_text_value("ItemName.Shuriken"), "Shuriken");
        assert_eq!(
            manager.get_text_value_args("CommonItemTooltip.UsesMana", &["20".into()]),
            "Uses 20 mana"
        );
        assert!(
            manager.category_size("ItemName") > 1000,
            "ItemName category has {} keys",
            manager.category_size("ItemName")
        );
        // Every shipped file is a flat two-level map.
        assert!(manager.exists("Error.LoadFailed"));
    }
}
