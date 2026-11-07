//! The localization runtime: the [`LanguageManager`] singleton behind the C#
//! `Language`/`Lang` statics.
//!
//! `Terraria.Localization`'s strings are embedded resources of the original
//! executable (`Terraria.Localization.Content.<culture>[.<Group>].json`, 12
//! cultures × 7 groups); the loader here reads the same files from the
//! extracted-resource directory — the same retail-derived, gitignored content
//! model as `assets/Content`. `init` registers every shipped file (the C#
//! assembly carries them all at once) and applies the saved culture; the
//! accessors mirror `Language.GetTextValue`, `Lang.menu[N]`
//! (= `LegacyMenu.N`, `Lang.cs:481`) and `Lang.GetItemNameValue`.
//!
//! Sources are found under `TR_LOCALE`, the content dir's `../decompiled`
//! sibling (the repo layout), or the working directory; without any of them
//! the manager keeps the English defaults baked into [`crate::itemdata`] and
//! the hardcoded menu rows degrade to their keys (the vanilla
//! `unknown_key_becomes_its_own_placeholder_text` behaviour).

use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

use tr_core::localization::{CULTURES, GameCulture, LanguageManager};

/// The resource groups `Main`'s assembly ships per culture
/// (`Assembly.GetManifestResourceNames()` order is irrelevant — the manager
/// filters by culture, `LoadFilesForCulture`).
const RESOURCE_GROUPS: [&str; 7] = ["", ".Items", ".Game", ".NPCs", ".Projectiles", ".Town", ".Legacy"];

static MANAGER: OnceLock<Mutex<LanguageManager>> = OnceLock::new();

fn manager() -> &'static Mutex<LanguageManager> {
    MANAGER.get_or_init(|| Mutex::new(LanguageManager::new()))
}

/// The directories that may hold the extracted `Terraria.Localization.Content`
/// json resources, in probe order.
fn source_dirs() -> Vec<PathBuf> {
    let mut out = Vec::new();
    if let Some(dir) = std::env::var_os("TR_LOCALE") {
        out.push(PathBuf::from(dir));
    }
    // The repo layout: `assets/Content` + `decompiled/` side by side.
    if let Some(content) = crate::assets::content_dir() {
        if let Some(parent) = content.parent() {
            out.push(parent.join("decompiled"));
        }
    }
    out.push(PathBuf::from("decompiled"));
    out.push(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../decompiled"));
    if let Some(exe) = std::env::current_exe().ok().and_then(|p| p.parent().map(|p| p.to_path_buf())) {
        out.push(exe.join("decompiled"));
    }
    out
}

/// Serializes the tests that touch the process-global `LanguageManager`
/// (the `app::tr_hooks` `ENV_LOCK` pattern): one test switching the culture
/// while another reads localized labels would race.
pub(crate) fn test_lock() -> &'static Mutex<()> {
    static LOCK: Mutex<()> = Mutex::new(());
    &LOCK
}

/// Registers every shipped resource file and applies `culture`. Called once
/// at startup (`main`), after the content dir is resolved; later calls reload
/// `SetLanguage`-style (idempotent for the same culture).
pub fn init(culture: GameCulture) {
    let mut m = manager().lock().unwrap();
    if !m.sources().is_empty() {
        m.set_language(culture);
        return;
    }
    let mut loaded = 0usize;
    for dir in source_dirs() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let name = entry.file_name();
            let Some(name) = name.to_str() else { continue };
            if !name.starts_with("Terraria.Localization.Content.") || !name.ends_with(".json") {
                continue;
            }
            let Ok(text) = std::fs::read_to_string(entry.path()) else {
                continue;
            };
            m.add_source(name.to_string(), text);
            loaded += 1;
        }
        // A directory that actually carried the resources wins; probing
        // further candidates would double-register the same culture files
        // (`LoadFilesForCulture` re-applies the later file's values, which
        // matches the original's `later file overrides earlier` rule, but one
        // authoritative copy keeps the manager's source list small).
        if loaded > 0 {
            break;
        }
    }
    if loaded == 0 {
        log::warn!(
            "no Terraria.Localization.Content.*.json found under {:?}; menus and item names stay English",
            source_dirs()
        );
    } else {
        log::info!("localization: {loaded} resource files, culture {}", culture.name());
    }
    m.set_language(culture);
}

/// `LanguageManager.SetLanguage` (the language-menu click, `Main.cs:48520`).
pub fn set_language(culture: GameCulture) {
    manager().lock().unwrap().set_language(culture);
}

/// `Language.ActiveCulture` (the English default until a load happened).
pub fn active_culture() -> GameCulture {
    manager().lock().unwrap().active_culture()
}

/// `Language.GetTextValue(key)`: the value with `{Name}` substitutions
/// applied, or the key itself when missing.
pub fn text(key: &str) -> String {
    manager().lock().unwrap().get_text_value(key)
}

/// `Language.GetTextValue(key, args)` (`LocalizedText.Format`).
pub fn text_args(key: &str, args: &[String]) -> String {
    manager().lock().unwrap().get_text_value_args(key, args)
}

/// `Lang.menu[i].Value` — the key is `LegacyMenu.{i}` (`Lang.cs:481`).
pub fn menu(i: usize) -> String {
    text(&format!("LegacyMenu.{i}"))
}

/// `Lang.GetItemNameValue(id)` (`Lang.cs:489-498`): `ItemName.{const name}`,
/// falling back to the bundled English name for ids the resources do not
/// cover (the manager answers a missing key with the key itself).
pub fn item_name(id: u16) -> String {
    let key = format!("ItemName.{}", crate::itemdata::NAMES[id as usize]);
    let value = text(&key);
    if value == key {
        crate::itemdata::ITEMS[id as usize].name.to_string()
    } else {
        value
    }
}

/// `GameCulture.FromLegacyId` — the language-menu rows are 1-based legacy ids
/// (`Main.cs:48520` passes `selectedMenu` straight through).
pub fn culture_from_legacy_id(id: i32) -> GameCulture {
    GameCulture::from_legacy_id(id)
}

/// All twelve shipped cultures in `GameCulture.CultureName` order — the
/// language screens' row list.
pub fn cultures() -> [GameCulture; 12] {
    CULTURES
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The manager answers from the shipped resources when they are present
    /// (the repo's `decompiled/`), falling back to the English item names
    /// otherwise.
    #[test]
    fn russian_texts_and_item_names_resolve() {
        let _guard = test_lock().lock().unwrap_or_else(|e| e.into_inner());
        let mut dir = None;
        for d in source_dirs() {
            if d.join("Terraria.Localization.Content.ru-RU.json").is_file() {
                dir = Some(d);
                break;
            }
        }
        let Some(dir) = dir else {
            eprintln!("skipping: no localization sources");
            return;
        };
        init(GameCulture::English);
        set_language(GameCulture::Russian);
        assert_eq!(menu(14), "Настройки");
        // The `Language.*` rows are localized in every culture — each name in
        // its own language (`decompiled/Terraria.Localization.Content.ru-RU.json`):
        // `Language.Russian` reads "Русский" there and the en-US parenthetical
        // ("Русский (Russian)") only under English.
        assert_eq!(text("Language.Russian"), "Русский");
        set_language(GameCulture::English);
        assert_eq!(text("Language.Russian"), "Русский (Russian)");
        set_language(GameCulture::Russian);
        // ItemName.DirtBlock — the ru-RU Items group.
        assert!(!item_name(2).is_empty());
        assert_ne!(item_name(2), "ItemName.DirtBlock");
        set_language(GameCulture::English);
        assert_eq!(menu(14), "Settings");
    }
}
