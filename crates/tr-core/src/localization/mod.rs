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

    /// `LocalizedText.ConditionsMet` (`LocalizedText.cs:35-45`): a plain
    /// value is always met; a `VariableText` value asks its conditions
    /// (`VariableText.ConditionsMet` → `CheckConditionsAndLoadArgs`,
    /// `VariableText.cs:73-99`).
    pub fn conditions_met(&self, lookup: &dyn Fn(&str) -> Option<Substitution>) -> bool {
        match &self.parsed {
            Some(parsed) => parsed.conditions_met(lookup),
            None => true,
        }
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

    /// `LocalizedText.GetValueIfConditionsMet(out string formatted)`
    /// (`LocalizedText.cs:70-78`): `Some(formatted)` when every condition and
    /// variable resolves; `None` leaves the caller's text untouched (the
    /// Guide's help-text gate, `Main.cs:39254`). A plain string is always
    /// `Some(Value)`.
    pub fn value_if_conditions_met(
        &self,
        lookup: &dyn Fn(&str) -> Option<Substitution>,
    ) -> Option<String> {
        match &self.parsed {
            Some(parsed) => parsed.try_format(lookup),
            None => Some(self.value.clone()),
        }
    }
}

/// `LanguageManager`.
#[derive(Clone, Debug)]
pub struct LanguageManager {
    localized_texts: HashMap<String, LocalizedText>,
    /// The `_localizedTexts` Dictionary's INSERTION order — .NET's
    /// `Dictionary<TKey,TValue>` enumerates in insertion order while no
    /// removals happen (the manager never removes), and `FindAll` /
    /// `SelectRandom` (`LanguageManager.cs:379-411`) walk that order. The
    /// chatter picks index into it, so the order is behaviour, not an
    /// implementation detail (W4-d120).
    text_order: Vec<String>,
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
            // The empty-key `LocalizedText.Empty` row is the Dictionary's
            // first entry (`LanguageManager.cs:25`'s initializer + the
            // ctor's `Add("", Empty)` shape).
            text_order: vec![String::new()],
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

    /// `LanguageManager.ReloadLanguage` — reload the texts from the
    /// registered sources (the public seam for the tests and for a runtime
    /// re-load).
    pub fn reload_language(&mut self, target_culture: GameCulture) {
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
            self.insert_text(key.to_string(), text);
        }
        &self.localized_texts[key]
    }

    /// One `_localizedTexts[key] = text` insertion that keeps the
    /// enumeration order ([`Self::text_order`]) in step — the Dictionary's
    /// indexer only appends to the order when the key is new.
    pub(super) fn insert_text(&mut self, key: String, text: LocalizedText) {
        if !self.localized_texts.contains_key(&key) {
            self.text_order.push(key.clone());
        }
        self.localized_texts.insert(key, text);
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

    /// `LanguageManager.FindAll(LanguageSearchFilter)`
    /// (`LanguageManager.cs:379-390`): the matching texts in the
    /// `_localizedTexts` Dictionary's insertion order.
    pub fn find_all(&self, filter: impl Fn(&str, &LocalizedText) -> bool) -> Vec<LocalizedText> {
        self.text_order
            .iter()
            .filter_map(|k| self.localized_texts.get(k))
            .filter(|v| filter(v.key(), v))
            .cloned()
            .collect()
    }

    /// `Language.SelectRandom(Lang.CreateDialogFilter(prefix))`
    /// (`LanguageManager.cs:392-411`) with the caller's drawn index: the
    /// count walk, then the SECOND walk picks by `--num == index` — the
    /// **reverse** position: index 0 answers the LAST matching row,
    /// index `count-1` the first. `LocalizedText.Empty` when nothing
    /// matches (`:410`). The C# draws `Main.rand.Next(count)` itself
    /// (`:402`); the port takes the draw as the parameter so the stream
    /// stays the caller's — INCLUDING the `Next(0)` draw of an empty
    /// category (the C# pays it, `UnifiedRandom.cs:140`).
    pub fn select_random_from_dialog(&mut self, prefix: &str, index: i32) -> LocalizedText {
        let matches: Vec<String> = self
            .text_order
            .iter()
            .filter(|k| {
                k.starts_with(prefix)
                    && self.localized_texts[*k]
                        .conditions_met(&|name| self.substitution(name))
            })
            .cloned()
            .collect();
        let mut num = matches.len() as i32;
        for key in &matches {
            num -= 1;
            if num == index {
                return self.get_text(key).clone();
            }
        }
        LocalizedText::empty()
    }

    /// The matching-row count of [`Self::select_random_from_dialog`] —
    /// the `num` the C# counts before its `Next(num)` (`:394-402`).
    pub fn dialog_category_count(&self, prefix: &str) -> usize {
        self.text_order
            .iter()
            .filter(|k| {
                k.starts_with(prefix)
                    && self.localized_texts[*k]
                        .conditions_met(&|name| self.substitution(name))
            })
            .count()
    }

    /// The i-th matching row of the dialog filter in insertion order —
    /// the `Language.FindAll(filter)[i]` arm (`NPC.GetChat`'s
    /// `array[num - k]` picks, `NPC.cs:93976-93984`).
    pub fn dialog_row(&mut self, prefix: &str, index: usize) -> LocalizedText {
        let key = self
            .text_order
            .iter()
            .filter(|k| {
                k.starts_with(prefix)
                    && self.localized_texts[*k]
                        .conditions_met(&|name| self.substitution(name))
            })
            .nth(index)
            .cloned();
        match key {
            Some(k) => self.get_text(&k).clone(),
            None => LocalizedText::empty(),
        }
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

    /// `LocalizedText.ConditionsMet` (`LocalizedText.cs:35-45`) and
    /// `GetValueIfConditionsMet` (`:70-78`, S1-L-12b): the gate the loading
    /// tips' reroll loop (`GameTipsProvider.cs:37-49`) and the Guide's
    /// help-text pick (`Main.cs:39254`) read.
    #[test]
    fn conditions_met_gates_the_conditional_rows() {
        let json = r#"{
            "Tips": {
                "Trash": "{?TrashEnabled}Sell with the trash can",
                "NoTrash": "{!TrashEnabled}Trash is disabled",
                "Mixed": "{?BloodMoon}The {PlayerName} era",
                "Plain": "No tokens here"
            }
        }"#;
        let mut m = manager_with(json);
        // Clone the rows out so the lookup closure can re-borrow the manager
        // (the `text_format_with` pattern).
        let trash = m.get_text("Tips.Trash").clone();
        let no_trash = m.get_text("Tips.NoTrash").clone();
        let mixed = m.get_text("Tips.Mixed").clone();
        let plain = m.get_text("Tips.Plain").clone();
        // Unset condition answers false (`CheckConditionsAndLoadArgs`'s
        // `as bool? ?? false`, VariableText.cs:79).
        assert!(!trash.conditions_met(&|n| m.substitution(n)));
        assert!(no_trash.conditions_met(&|n| m.substitution(n)));
        assert!(plain.conditions_met(&|n| m.substitution(n)));
        assert_eq!(trash.value_if_conditions_met(&|n| m.substitution(n)), None);
        assert_eq!(
            plain.value_if_conditions_met(&|n| m.substitution(n)),
            Some("No tokens here".to_string())
        );
        m.set_substitution("TrashEnabled", true);
        assert!(trash.conditions_met(&|n| m.substitution(n)));
        assert!(!no_trash.conditions_met(&|n| m.substitution(n)));
        assert_eq!(
            trash.value_if_conditions_met(&|n| m.substitution(n)),
            Some("Sell with the trash can".to_string())
        );
        // The gate checks the VARIABLES too: a passing condition with a
        // missing variable still fails (`CheckConditionsAndLoadArgs` loads
        // both, VariableText.cs:73-94).
        m.set_substitution("BloodMoon", true);
        assert!(!mixed.conditions_met(&|n| m.substitution(n)));
        assert_eq!(mixed.value_if_conditions_met(&|n| m.substitution(n)), None);
        m.set_substitution("PlayerName", "Jeff");
        assert!(mixed.conditions_met(&|n| m.substitution(n)));
        assert_eq!(
            mixed.value_if_conditions_met(&|n| m.substitution(n)),
            Some("The Jeff era".to_string())
        );
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

    /// W4-d120 — `SelectRandom` (`LanguageManager.cs:392-411`) picks by
    /// `--num == index`: index 0 is the LAST matching row, `count-1` the
    /// first, in the Dictionary's insertion order. The conditional rows
    /// (`{?Night}`…) only count while their substitution holds
    /// (`Lang.CreateDialogFilter`'s `ConditionsMet` gate, `Lang.cs:424`).
    ///
    /// RE-DERIVED (W4-d120 merge): the callers read `.Value`, i.e. the
    /// FORMATTED text — `VariableText.TryFormat` strips the `{?Night}`
    /// marker (`LocalizedText.cs:24-31`). `UnformattedValue` keeps it, so
    /// the condition row has to be compared through `value_with`.
    #[test]
    fn select_random_picks_reverse_insertion_order_with_conditions() {
        let json = r#"{"Chatter":{"A":"first","B":"second","C":"{?Night}third"}}"#;
        let mut m = manager_with(json);
        // Night unset → C fails its condition: two matches, A then B.
        assert_eq!(m.dialog_category_count("Chatter."), 2);
        assert_eq!(
            m.select_random_from_dialog("Chatter.", 0).unformatted_value(),
            "second"
        );
        assert_eq!(
            m.select_random_from_dialog("Chatter.", 1).unformatted_value(),
            "first"
        );
        // Night true → three matches; index 0 now answers C, formatted (the
        // `{?Night}` gate marker is consumed by the format pass).
        m.set_substitution("Night", true);
        assert_eq!(m.dialog_category_count("Chatter."), 3);
        let sel = m.select_random_from_dialog("Chatter.", 0);
        assert_eq!(sel.value_with(&|n| m.substitution(n)), "third");
        assert_eq!(
            m.select_random_from_dialog("Chatter.", 2).unformatted_value(),
            "first"
        );
        // The FindAll[i] arm walks FORWARD (`array[num - k]`'s array).
        assert_eq!(m.dialog_row("Chatter.", 0).unformatted_value(), "first");
        let row = m.dialog_row("Chatter.", 2);
        assert_eq!(row.value_with(&|n| m.substitution(n)), "third");
        // No match → `LocalizedText.Empty` (`:410`).
        assert_eq!(
            m.select_random_from_dialog("Nope.", 0).unformatted_value(),
            ""
        );
        assert_eq!(m.dialog_category_count("Nope."), 0);
    }

    /// W4-d120 — a plain `{Name}` variable with no substitution fails
    /// `ConditionsMet` too (`VariableText.cs:87-90`'s null check), so the
    /// row drops out of the dialog filter until the name resolves.
    ///
    /// RE-DERIVED (W4-d120 merge): index 0 is the LAST matching row, so
    /// once BOTH rows qualify index 0 is the plain "B" and index 1 the
    /// variable row (which formats to "hi Bob").
    #[test]
    fn dialog_filter_needs_every_variable_resolved() {
        let json = r#"{"Chatter":{"A":"hi {PlayerName}","B":"plain"}}"#;
        let mut m = manager_with(json);
        assert_eq!(m.dialog_category_count("Chatter."), 1);
        assert_eq!(
            m.select_random_from_dialog("Chatter.", 0).unformatted_value(),
            "plain"
        );
        m.set_substitution("PlayerName", "Bob");
        assert_eq!(m.dialog_category_count("Chatter."), 2);
        let last = m.select_random_from_dialog("Chatter.", 0);
        assert_eq!(last.value_with(&|n| m.substitution(n)), "plain");
        let first = m.select_random_from_dialog("Chatter.", 1);
        assert_eq!(first.value_with(&|n| m.substitution(n)), "hi Bob");
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
    /// and checks a few known keys. The language JSON lives in the gitignored
    /// `decompiled/` tree (there is none under `assets/`), so this test **fails**
    /// when that junction is absent: a silent skip let it pass green with 0 keys
    /// and no coverage (S1-L-12d).
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
        let mut missing = Vec::new();
        for name in resources {
            let path = dir.join(name);
            match std::fs::read_to_string(&path) {
                Ok(text) => manager.add_source(name, text),
                Err(_) => missing.push(name),
            }
        }
        assert!(
            missing.is_empty(),
            "the English localization resources are missing from {} (S1-L-12d): {:?}\n\
             the language JSON lives in the gitignored `decompiled/` tree — create the \
             junction (`cmd /c mklink /J decompiled ..\\Terraria-Rust\\decompiled`) before \
             running the suite",
            dir.display(),
            missing
        );
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
