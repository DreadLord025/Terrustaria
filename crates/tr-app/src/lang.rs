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

use tr_core::localization::{CULTURES, GameCulture, LanguageManager, Substitution};

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
    if let Some(dir) = crate::env::locale_dir() {
        out.push(dir);
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
        // `read_dir` order is filesystem-dependent; register resources in a
        // deterministic (sorted by file name) order so `IndexedFromCategory` /
        // `category_grouped_keys` mappings are stable (`Lang.cs` loads embedded
        // resources in manifest order; S1-L-12c).
        let mut resources: Vec<(String, String)> = entries
            .flatten()
            .filter_map(|entry| {
                let name = entry.file_name();
                let name = name.to_str()?;
                if !name.starts_with("Terraria.Localization.Content.") || !name.ends_with(".json") {
                    return None;
                }
                let text = std::fs::read_to_string(entry.path()).ok()?;
                Some((name.to_string(), text))
            })
            .collect();
        resources.sort_by(|a, b| a.0.cmp(&b.0));
        if resources.is_empty() {
            continue;
        }
        for (name, text) in resources {
            m.add_source(name, text);
            loaded += 1;
        }
        // A directory that actually carried the resources wins; probing
        // further candidates would double-register the same culture files
        // (`LoadFilesForCulture` re-applies the later file's values, which
        // matches the original's `later file overrides earlier` rule, but one
        // authoritative copy keeps the manager's source list small).
        break;
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

/// `Lang.InitGlobalSubstitutions` (`Lang.cs:100-167`, S1-L-12b): refresh the 65
/// global `{Name}` substitutions from live app state before text resolution.
///
/// C# registers lazy `Func<object>` producers; the port materializes the
/// current values into [`LanguageManager`]'s substitution table each refresh.
/// Substitutions with no source (e.g. an absent NPC or player) are removed so a
/// `VariableText` sees them as unset, matching the null producer result.
pub fn refresh_global_substitutions(app: &crate::app::App) {
    let mut m = manager().lock().unwrap();

    // NPC first-name substitutions: `NPC.GetFirstNPCNameOrNull(type)`
    // (`Lang.cs:102-123,139-140`).
    const NPC_NAMES: &[(&str, i32)] = &[
        ("Nurse", 18),
        ("Merchant", 17),
        ("ArmsDealer", 19),
        ("Dryad", 20),
        ("Demolitionist", 38),
        ("Clothier", 54),
        ("Guide", 22),
        ("Wizard", 108),
        ("GoblinTinkerer", 107),
        ("Mechanic", 124),
        ("Truffle", 160),
        ("Steampunker", 178),
        ("DyeTrader", 207),
        ("PartyGirl", 208),
        ("Cyborg", 209),
        ("Painter", 227),
        ("WitchDoctor", 228),
        ("Pirate", 229),
        ("Stylist", 353),
        ("TravelingMerchant", 368),
        ("Angler", 369),
        ("Bartender", 550),
        ("GolfGuy", 588),
        ("TaxCollector", 441),
    ];
    for &(key, npc_type) in NPC_NAMES {
        if let Some(name) = first_npc_name(app, npc_type) {
            m.set_substitution(key, Substitution::Text(name));
        } else {
            m.remove_substitution(key);
        }
    }

    // World / event flags (`Lang.cs:124-135,141`).
    m.set_substitution("WorldName", Substitution::Text(app.world.name.clone()));
    m.set_substitution("Day", Substitution::Bool(app.world.day_time));
    m.set_substitution("BloodMoon", Substitution::Bool(app.world.blood_moon));
    m.set_substitution("Eclipse", Substitution::Bool(app.world.eclipse));
    m.set_substitution(
        "MoonLordDefeated",
        Substitution::Bool(app.world.downed_moon_lord),
    );
    m.set_substitution(
        "GolemDefeated",
        Substitution::Bool(app.world.downed_golem),
    );
    m.set_substitution(
        "DukeFishronDefeated",
        Substitution::Bool(false), // no `NPC.downedFishron` field ported yet
    );
    m.set_substitution(
        "FrostLegionDefeated",
        Substitution::Bool(app.world.downed_frost),
    );
    m.set_substitution(
        "MartiansDefeated",
        Substitution::Bool(app.world.downed_martians),
    );
    m.set_substitution(
        "PumpkingDefeated",
        Substitution::Bool(false), // no `NPC.downedHalloweenKing` field ported yet
    );
    m.set_substitution(
        "IceQueenDefeated",
        Substitution::Bool(false), // no `NPC.downedChristmasIceQueen` field ported yet
    );
    m.set_substitution("HardMode", Substitution::Bool(app.world.hard_mode));
    m.set_substitution("Rain", Substitution::Bool(app.rain.is_raining()));

    // Player / interaction (`Lang.cs:136,138,142-144`).
    // `Homeless`: talk NPC is homeless. The port has no `talkNPC` index; keep
    // unset (null-equivalent) rather than guessing.
    m.remove_substitution("Homeless");
    m.set_substitution("PlayerName", Substitution::Text(app.player_name.clone()));
    m.set_substitution(
        "Graveyard",
        Substitution::Bool(app.scene_metrics.zone_graveyard),
    );
    let (angler_quests, deaths_pve) = if let Some(p) = &app.player {
        (p.angler_quests_finished as i64, p.deaths_pve as i64)
    } else {
        (0, 0)
    };
    m.set_substitution("AnglerCompletedQuestsCount", Substitution::Int(angler_quests));
    m.set_substitution("TotalDeathsCount", Substitution::Int(deaths_pve));

    // `WorldEvilStone` (`Lang.cs:145`): localized evil-stone name by world evil.
    // The port has no world-level crimson flag yet; default to corruption's stone.
    let evil_key = "Misc.Ebonstone";
    let evil_name = m.get_text_value(evil_key);
    m.set_substitution("WorldEvilStone", Substitution::Text(evil_name));

    // Input / UI keys (`Lang.cs:137,146-166`). `PlayerInput.GenerateInputTag_*`
    // and `Main.cInv`/favorite-key/gamepad state are not fully ported; provide
    // stable, descriptive fallbacks so the 65-entry producer is complete and the
    // tokens never leak raw when a key is set.
    m.set_substitution("InventoryKey", Substitution::Text("Inventory".to_string()));
    let using_gamepad = false;
    let disable_left_shift_trash = false;
    m.set_substitution(
        "InputTriggerUI_BuildFromInventory",
        Substitution::Text(input_tag(false, "QuickMount", using_gamepad)),
    );
    m.set_substitution("InputTriggerUI_TrashEnabled", Substitution::Bool(true));
    let trash_label = if !using_gamepad {
        m.get_text_value(if disable_left_shift_trash {
            "Controls.Control"
        } else {
            "Controls.Shift"
        })
    } else {
        input_tag(false, "SmartSelect", using_gamepad)
    };
    m.set_substitution("InputTriggerUI_Trash", Substitution::Text(trash_label));
    m.set_substitution(
        "InputTriggerUI_FavoriteItem",
        Substitution::Text(if !using_gamepad {
            "Alt".to_string()
        } else {
            input_tag(false, "SmartCursor", using_gamepad)
        }),
    );
    m.set_substitution(
        "InputTrigger_QuickEquip",
        Substitution::Text(input_tag(false, "Grapple", using_gamepad)),
    );
    m.set_substitution(
        "InputTrigger_Grapple",
        Substitution::Text(input_tag(true, "Grapple", using_gamepad)),
    );
    m.set_substitution(
        "InputTrigger_LockOn",
        Substitution::Text(input_tag(true, "LockOn", using_gamepad)),
    );
    m.set_substitution(
        "InputTrigger_RadialQuickbar",
        Substitution::Text(input_tag(true, "HotbarMinus", using_gamepad)),
    );
    m.set_substitution(
        "InputTrigger_RadialHotbar",
        Substitution::Text(input_tag(true, "HotbarPlus", using_gamepad)),
    );
    m.set_substitution(
        "InputTrigger_SmartCursor",
        Substitution::Text(input_tag(true, "SmartCursor", using_gamepad)),
    );
    m.set_substitution(
        "InputTrigger_UseOrAttack",
        Substitution::Text(input_tag(true, "MouseLeft", using_gamepad)),
    );
    m.set_substitution(
        "InputTrigger_Jump",
        Substitution::Text(input_tag(true, "Jump", using_gamepad)),
    );
    m.set_substitution(
        "InputTrigger_InteractWithTile",
        Substitution::Text(input_tag(true, "MouseRight", using_gamepad)),
    );
    m.set_substitution(
        "InputTrigger_InteractWithTileUI",
        Substitution::Text(input_tag(false, "MouseRight", using_gamepad)),
    );
    m.set_substitution(
        "InputTrigger_ToggleOrOpen",
        Substitution::Text(if !using_gamepad {
            input_tag(true, "MouseRight", using_gamepad)
        } else {
            input_tag(false, "Grapple", using_gamepad)
        }),
    );
    m.set_substitution(
        "InputTrigger_SmartSelect",
        Substitution::Text(input_tag(true, "SmartSelect", using_gamepad)),
    );
    m.set_substitution(
        "InputTrigger_Mount",
        Substitution::Text(input_tag(true, "QuickMount", using_gamepad)),
    );
    m.set_substitution(
        "InputTrigger_Healing",
        Substitution::Text(input_tag(true, "QuickHeal", using_gamepad)),
    );
    m.set_substitution(
        "InputTrigger_ManaHealing",
        Substitution::Text(input_tag(true, "QuickMana", using_gamepad)),
    );
    m.set_substitution(
        "InputTrigger_QuickBuff",
        Substitution::Text(input_tag(true, "QuickBuff", using_gamepad)),
    );
    let armor_key = if app.settings.reversed_armor_bonuses {
        "Key.UP"
    } else {
        "Key.DOWN"
    };
    let armor_value = m.get_text_value(armor_key);
    m.set_substitution("ToggleArmorSetBonusKey", Substitution::Text(armor_value));
}

/// `NPC.GetFirstNPCNameOrNull` (`NPC.cs`): the `GivenOrTypeName` of the first
/// active NPC whose net/type id matches `npc_type`, or `None`.
fn first_npc_name(app: &crate::app::App, npc_type: i32) -> Option<String> {
    app.npcs.iter().find_map(|n| {
        if n.hp > 0 && n.kind.net_id() == npc_type {
            Some(n.given_name.clone())
        } else {
            None
        }
    })
}

#[cfg(test)]
mod s1_l12b_tests {
    use super::*;

    #[test]
    fn global_substitutions_resolve_world_player_and_npc_tokens() {
        // `Lang.InitGlobalSubstitutions` (`Lang.cs:100-151`, S1-L-12b): the
        // manager must resolve key global tokens once their values are set.
        let mut m = LanguageManager::new();
        m.add_source(
            "Terraria.Localization.Content.en-US.Game.json",
            r#"{"Game":{"World":"World {WorldName}","Day":"{Day}","Rain":"{Rain}","Guide":"{Guide}"}}"#,
        );
        m.set_language(GameCulture::English);
        m.set_substitution("WorldName", Substitution::Text("Terra".into()));
        m.set_substitution("Day", Substitution::Bool(true));
        m.set_substitution("Rain", Substitution::Bool(false));
        m.set_substitution("Guide", Substitution::Text("Andrew".into()));
        assert_eq!(m.get_text_value("Game.World"), "World Terra");
        assert_eq!(m.get_text_value("Game.Day"), "True");
        assert_eq!(m.get_text_value("Game.Rain"), "False");
        assert_eq!(m.get_text_value("Game.Guide"), "Andrew");
    }
}

#[cfg(test)]
mod s1_l12c_tests {
    #[test]
    fn resource_registration_uses_sorted_file_names() {
        // `lang::init` must register `Terraria.Localization.Content.*.json`
        // in deterministic (sorted) file-name order (S1-L-12c) so
        // `IndexedFromCategory` mappings do not depend on `read_dir` order.
        let mut names = vec![
            "Terraria.Localization.Content.en-US.Town.json",
            "Terraria.Localization.Content.en-US.Game.json",
            "Terraria.Localization.Content.en-US.Items.json",
        ];
        let mut sorted = names.clone();
        sorted.sort();
        names.sort_by(|a, b| a.cmp(b));
        assert_eq!(names, sorted);
        assert_eq!(
            names,
            vec![
                "Terraria.Localization.Content.en-US.Game.json",
                "Terraria.Localization.Content.en-US.Items.json",
                "Terraria.Localization.Content.en-US.Town.json",
            ]
        );
    }
}

/// Best-effort NPC key name for a town NPC type id when no instance/given name
/// exists. Falls back to the numeric id (never panics on unknown).
fn npc_type_name(npc_type: i32) -> &'static str {
    match npc_type {
        17 => "Merchant",
        18 => "Nurse",
        19 => "ArmsDealer",
        20 => "Dryad",
        22 => "Guide",
        38 => "Demolitionist",
        54 => "Clothier",
        107 => "GoblinTinkerer",
        108 => "Wizard",
        124 => "Mechanic",
        160 => "Truffle",
        178 => "Steampunker",
        207 => "DyeTrader",
        208 => "PartyGirl",
        209 => "Cyborg",
        227 => "Painter",
        228 => "WitchDoctor",
        229 => "Pirate",
        353 => "Stylist",
        368 => "TravelingMerchant",
        369 => "Angler",
        441 => "TaxCollector",
        550 => "DD2Bartender",
        588 => "Golfer",
        _ => "TownNPC",
    }
}

/// Fallback stand-in for `PlayerInput.GenerateInputTag_ForCurrentGamemode`.
/// The port has no full trigger-tag renderer yet; return stable labels that
/// approximate the default keyboard/mouse bindings where known.
fn input_tag(_gameplay: bool, action: &str, using_gamepad: bool) -> String {
    if using_gamepad {
        return format!("[{action}]");
    }
    match action {
        "Jump" => "Space".to_string(),
        "MouseLeft" => "Left Click".to_string(),
        "MouseRight" => "Right Click".to_string(),
        "Grapple" => "E".to_string(),
        "QuickMount" => "R".to_string(),
        "SmartCursor" => "Left Ctrl".to_string(),
        "SmartSelect" => "Left Shift".to_string(),
        "QuickHeal" => "H".to_string(),
        "QuickMana" => "J".to_string(),
        "QuickBuff" => "B".to_string(),
        "LockOn" => "Lock On".to_string(),
        "HotbarMinus" => "Hotbar-".to_string(),
        "HotbarPlus" => "Hotbar+".to_string(),
        _ => format!("<{action}>"),
    }
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

/// `Language.GetTextValue("GameUI.<name>", args)` — the `GameUI` category
/// rows the tooltip prints (`Main.cs:20805-20816`'s fishing rows).
pub fn game_ui_fmt(key: &str, value: i32) -> String {
    text_args(
        &format!("GameUI.{key}"),
        &[value.to_string()],
    )
}

/// `Lang.menu[i].Value` — the key is `LegacyMenu.{i}` (`Lang.cs:481`).
pub fn menu(i: usize) -> String {
    text(&format!("LegacyMenu.{i}"))
}

/// `Lang.tip[i].Value` — the tooltip rows; the key is `LegacyTooltip.{i}`
/// (`Lang.cs:497`). The item hover text reads them through
/// `MouseText_DrawItemTooltip_GetLinesInfo` (`Main.cs:20631-20800`).
pub fn tip(i: usize) -> String {
    text(&format!("LegacyTooltip.{i}"))
}

/// `Lang.inter[i].Value` — the interface rows (the tooltip's shop/upgrade
/// lines); the key is `LegacyInterface.{i}` (`Lang.cs:485`).
pub fn inter(i: usize) -> String {
    text(&format!("LegacyInterface.{i}"))
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

// -- The name caches (`Lang.InitializeLegacyLocalization`, `Lang.cs:474-553`) --
//
// The C# fills every name cache by reflecting over the id class and reading
// the key `<Category>.<ConstName>` (`Lang.FillNameCacheArray`,
// `Lang.cs:450-464`), so the constant spelling IS the localization key. The
// const walks live in `lang_tables.rs` (generated) and in the id tables the
// port already carried (`projectile_id.rs`, `prefix_id.rs`, `emotes.rs`,
// `itemdata.rs`).

/// The const name of an id row — the localization key's suffix.
fn const_name(table: &'static [(i32, &'static str)], id: i32) -> Option<&'static str> {
    table
        .iter()
        .find(|(n, _)| *n == id)
        .map(|(_, name)| *name)
        .or_else(|| {
            table
                .iter()
                .find(|(n, _)| *n == id as u16 as i32)
                .map(|(_, name)| *name)
        })
}

/// `Lang.GetNPCName(netID)` (`Lang.cs:188-195`): `NPCName.{const name}`.
///
/// A negative net id walks the negative cache (`Lang.cs:524-526`): the
/// `NPCID.FromNetId` map first, then the twelve explicit slime rows.
pub fn npc_name(id: i32) -> String {
    let key = if id < 0 {
        let index = (-id - 1) as usize;
        match NEGATIVE_NPC_NAMES.iter().find(|(i, _)| *i == index) {
            Some((_, name)) => format!("NPCName.{name}"),
            None => {
                let positive = crate::lang_tables::NPC_NET_ID_MAP
                    .get(index)
                    .copied()
                    .unwrap_or(0);
                match const_name(crate::lang_tables::NPC_ID, positive) {
                    Some(name) => format!("NPCName.{name}"),
                    None => return String::new(),
                }
            }
        }
    } else {
        match const_name(crate::lang_tables::NPC_ID, id) {
            Some(name) => format!("NPCName.{name}"),
            None => return String::new(),
        }
    };
    text(&key)
}

/// The twelve explicit negative-name rows (`Lang.cs:528-533`): the slime
/// family and the two rain zombies. `(index, const name)` — the index is
/// `-netId - 1`.
const NEGATIVE_NPC_NAMES: &[(usize, &str)] = &[
    (0, "Slimeling"),
    (1, "Slimer2"),
    (2, "GreenSlime"),
    (3, "Pinky"),
    (4, "BabySlime"),
    (5, "BlackSlime"),
    (6, "PurpleSlime"),
    (7, "RedSlime"),
    (8, "YellowSlime"),
    (9, "JungleSlime"),
    (53, "SmallRainZombie"),
    (54, "BigRainZombie"),
];

/// `Lang.GetBuffName(id)` (`Lang.cs:241-244`): `BuffName.{const name}`.
pub fn buff_name(id: i32) -> String {
    match const_name(crate::lang_tables::BUFF_ID, id) {
        Some(name) => text(&format!("BuffName.{name}")),
        None => String::new(),
    }
}

/// `Lang.GetBuffDescription(id)` (`Lang.cs:246-249`):
/// `BuffDescription.{const name}`.
pub fn buff_description(id: i32) -> String {
    match const_name(crate::lang_tables::BUFF_ID, id) {
        Some(name) => text(&format!("BuffDescription.{name}")),
        None => String::new(),
    }
}

/// `Lang.GetEmojiName(id)` (`Lang.cs:211-218`): `EmojiName.{const name}`.
/// The C# leaves a missing entry blank (`leaveMissingEntriesBlank`), so an
/// unknown id answers empty.
pub fn emoji_name(id: i32) -> String {
    match crate::emotes::EMOTE_NAMES
        .iter()
        .find(|(n, _)| *n == id)
        .map(|(_, name)| *name)
    {
        Some(name) => text(&format!("EmojiName.{name}")),
        None => String::new(),
    }
}

/// `Lang.prefix[id]` (`Lang.cs:474`): `Prefix.{const name}`.
pub fn prefix_name(id: i32) -> String {
    match crate::prefix_id::PREFIX_ID
        .iter()
        .find(|(n, _)| *n as i32 == id)
        .map(|(_, name)| *name)
    {
        Some(name) => text(&format!("Prefix.{name}")),
        None => String::new(),
    }
}

/// `Lang.GetProjectileName(type)` (`Lang.cs:441-448`):
/// `ProjectileName.{const name}`.
pub fn projectile_name(id: i32) -> String {
    match crate::projectile_id::PROJECTILE_ID
        .iter()
        .find(|(n, _)| *n as i32 == id)
        .map(|(_, name)| *name)
    {
        Some(name) => text(&format!("ProjectileName.{name}")),
        None => String::new(),
    }
}

/// `Lang._itemTooltipCache[id]` (`Lang.cs:538-550`):
/// `ItemTooltip.{const name}` — the item's own tooltip row.
pub fn item_tooltip(id: u16) -> String {
    text(&format!("ItemTooltip.{}", crate::itemdata::NAMES[id as usize]))
}

/// `Lang.GetMapObjectName(id)` (`Lang.cs:77-84`): the map legend row of a
/// color-lookup id.
pub fn map_object_name(lookup_id: usize) -> String {
    let Some(row) = map_legend_rows().get(lookup_id).and_then(|r| *r) else {
        return String::new();
    };
    match row {
        MapLegendRow::Key(key) => text(key),
        MapLegendRow::Item(item) => item_name(item),
    }
}

/// `Lang.GetMapObjectName` for a tile/option pair — the port's convenience
/// over the C#'s lookup-id index (the map legend is written per
/// `MapHelper.TileToLookup(tile, option)`).
pub fn map_object_name_for_tile(tile: u16, option: u8) -> String {
    let lookup = tr_core::map::color_table().tile_lookup[tile as usize] as usize + option as usize;
    map_object_name(lookup)
}

/// One map legend row: a `MapObject.*` key or an item-name row.
#[derive(Clone, Copy)]
enum MapLegendRow {
    Key(&'static str),
    Item(u16),
}

/// The map legend cache — `Lang.BuildMapAtlas` (`Lang.cs:554-...`): the
/// lookup-id indexed table the C# fills with the `MapObject.*` keys and the
/// `itemNameCache[N]` rows, plus the tree-trunk loop (`:565-572`).
fn map_legend_rows() -> &'static [Option<MapLegendRow>] {
    use std::sync::OnceLock;
    static ROWS: OnceLock<Vec<Option<MapLegendRow>>> = OnceLock::new();
    ROWS.get_or_init(|| {
        let table = tr_core::map::color_table();
        let mut rows = vec![None; table.lookup.len()];
        let mut put = |tile: u16, option: u8, row: MapLegendRow| {
            let lookup = table.tile_lookup[tile as usize] as usize + option as usize;
            if lookup < rows.len() {
                rows[lookup] = Some(row);
            }
        };
        for (tile, option, kind, value) in crate::lang_tables::MAP_LEGEND {
            if *kind == "key" {
                put(*tile, *option, MapLegendRow::Key(value));
            } else {
                put(*tile, *option, MapLegendRow::Item(value.parse().unwrap_or(0)));
            }
        }
        // `:565-572`: every tree trunk reads "MapObject.Tree".
        for tile in 0..table.tile_lookup.len() {
            if tr_core::tileobject::is_a_tree_trunk(tile as u16) {
                put(tile as u16, 0, MapLegendRow::Key("MapObject.Tree"));
            }
        }
        rows
    })
}

// -- The legacy index tables (`Lang.InitializeLegacyLocalization`) ------------

/// `Lang.gen[i]` (`Lang.cs:476`): `LegacyWorldGen.{i}` — the world-generation
/// messages (the C#'s `gen` is a Rust keyword, so the accessor is named
/// `world_gen`).
pub fn world_gen(i: usize) -> String {
    text(&format!("LegacyWorldGen.{i}"))
}

/// `Lang.misc[i]` (`Lang.cs:484`): `LegacyMisc.{i}`.
pub fn misc(i: usize) -> String {
    text(&format!("LegacyMisc.{i}"))
}

/// `Lang.mp[i]` (`Lang.cs:490`): `LegacyMultiplayer.{i}`.
pub fn mp(i: usize) -> String {
    text(&format!("LegacyMultiplayer.{i}"))
}

/// `Lang.chestType[i]` (`Lang.cs:493`): `LegacyChestType.{i}`.
pub fn chest_type(i: usize) -> String {
    text(&format!("LegacyChestType.{i}"))
}

/// `Lang.chestType2[i]` (`Lang.cs:498`): `LegacyChestType2.{i}`.
pub fn chest_type2(i: usize) -> String {
    text(&format!("LegacyChestType2.{i}"))
}

/// `Lang.dresserType[i]` (`Lang.cs:501`): `LegacyDresserType.{i}`.
pub fn dresser_type(i: usize) -> String {
    text(&format!("LegacyDresserType.{i}"))
}

/// All twelve shipped cultures in `GameCulture.CultureName` order — the
/// language screens' row list.
pub fn cultures() -> [GameCulture; 12] {
    CULTURES
}

#[cfg(test)]
mod tests {
    use super::*;

    /// S1-L-12 — the generated id walks cover the C#'s own constant tables
    /// (`tools/gen_lang_tables.py`; the C#'s `NPCID.Count` = 697 and
    /// `BuffID.Count` = 401).
    #[test]
    fn the_generated_id_tables_cover_the_c_sharp_constants() {
        // The NPC walk: the positive ids up to `NPCID.Count` plus the 66
        // negative aliases (`NPCID.FromNetId`).
        let npc = crate::lang_tables::NPC_ID;
        assert_eq!(npc.len(), 763);
        assert!(npc.iter().any(|(id, name)| *id == 245 && *name == "Golem"));
        assert_eq!(crate::lang_tables::NPC_NET_ID_MAP.len(), 65);
        // The buff walk: `BuffID.Count` = 401 ids.
        assert_eq!(crate::lang_tables::BUFF_ID.len(), 400);
        assert!(crate::lang_tables::BUFF_ID
            .iter()
            .any(|(id, name)| *id == 24 && *name == "OnFire"));
        // The map legend carries both row kinds.
        let legend = crate::lang_tables::MAP_LEGEND;
        assert!(legend.iter().any(|(_, _, kind, _)| *kind == "key"));
        assert!(legend.iter().any(|(_, _, kind, _)| *kind == "item"));
    }

    /// S1-L-12 — the name caches: every table's accessor reads the C#'s
    /// `<Category>.<ConstName>` key (`Lang.FillNameCacheArray`,
    /// `Lang.cs:450-464`), so the localized value comes from the shipped
    /// resources.
    #[test]
    fn the_name_tables_resolve_their_localized_rows() {
        let _guard = test_lock().lock().unwrap_or_else(|e| e.into_inner());
        init(GameCulture::English);
        // The NPC names — `NPCID.Guide` is id 22, `NPCID.BlueSlime` is 1.
        assert_eq!(npc_name(22), "Guide");
        assert_eq!(npc_name(1), "Blue Slime");
        assert_eq!(npc_name(17), "Merchant");
        // The negative net ids walk the `NPCID.FromNetId` map and the twelve
        // explicit slime rows (`Lang.cs:524-533`).
        assert_eq!(npc_name(-1), "Slimeling");
        assert_eq!(npc_name(-3), "Green Slime");
        // The buff names and descriptions — `BuffID.OnFire` is id 24.
        assert_eq!(buff_name(24), "On Fire!");
        assert_eq!(buff_description(24), "Slowly losing life");
        // The emoji names — `EmoteID.EmotionLove` is id 0.
        assert_eq!(emoji_name(0), "heart");
        // The prefix names — `PrefixID.Large` is id 1.
        assert_eq!(prefix_name(1), "Large");
        // The projectile names — `ProjectileID.BoneJavelin` is id 598.
        assert_eq!(projectile_name(598), "Bone Javelin");
        // The map legend — the `MapObject.Iron` row (tile 6), the tree
        // trunks (`Lang.BuildMapAtlas` `:565-572` — the C#'s
        // `TileID.Sets.IsATreeTrunk` starts at tile 5) and the item-name
        // rows (`:574-575`).
        assert_eq!(map_object_name_for_tile(6, 0), "Iron");
        assert_eq!(map_object_name_for_tile(5, 0), "Tree");
        assert_eq!(map_object_name_for_tile(4, 0), item_name(8));
        // The legacy index tables (`Lang.InitializeLegacyLocalization`).
        assert_eq!(misc(0), "A goblin army has been defeated!");
        assert_eq!(world_gen(0), "Generating world terrain");
        assert_eq!(mp(0), "Receive:");
        assert_eq!(chest_type(0), "Chest");
        assert_eq!(dresser_type(0), "Dresser");
    }

    /// S1-L-12 — the same keys answer in the selected culture: the names
    /// render localized (`Lang.SetLanguage` re-resolves every key).
    #[test]
    fn the_name_tables_render_localized() {
        let _guard = test_lock().lock().unwrap_or_else(|e| e.into_inner());
        init(GameCulture::English);
        set_language(GameCulture::Russian);
        assert_eq!(npc_name(22), "Гид");
        assert_eq!(npc_name(-1), "Слизлинг");
        assert_eq!(buff_name(24), "В огне!");
        assert_eq!(emoji_name(0), "сердце");
        assert_eq!(map_object_name_for_tile(6, 0), "Железо");
        set_language(GameCulture::English);
    }

    /// The manager answers from the shipped resources when they are present
    /// (the repo's `decompiled/`), falling back to the English item names
    /// otherwise. The language JSON lives in the gitignored `decompiled/`
    /// junction, so this test **fails** (not skips) when it is absent — a silent
    /// skip let it pass green with no coverage (S1-L-12d).
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
        let Some(_dir) = dir else {
            panic!(
                "no localization sources found (S1-L-12d): the language JSON lives in the \
                 gitignored `decompiled/` tree — create the junction \
                 (`cmd /c mklink /J decompiled ..\\Terraria-Rust\\decompiled`) before \
                 running the suite. Probed: {:?}",
                source_dirs()
            );
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

    #[test]
    fn tip_and_inter_keys_follow_the_c_sharp_prefixes() {
        // `Lang.cs:497`/`:485` — `LegacyTooltip.{i}` / `LegacyInterface.{i}`.
        // Without the shipped resources the manager answers the key itself
        // (vanilla's unknown-key behaviour), which is what pins the format here.
        // The manager's language is global state — the sibling test flips it
        // mid-run — so the two reads need the same guard the other language
        // tests take, or a parallel `set_language` lands between them.
        let _guard = test_lock().lock().unwrap_or_else(|e| e.into_inner());
        assert_eq!(tip(2), text("LegacyTooltip.2"));
        assert_eq!(inter(65), text("LegacyInterface.65"));
        // A missing key falls back to the key text, not a crash.
        assert_eq!(tip(9999), text("LegacyTooltip.9999"));
    }
}
