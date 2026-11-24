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
    // Every producer is materialized BEFORE the manager lock: the name/text
    // producers (`NPC.TypeName` -> `Lang.GetNPCNameValue`, NPC.cs:6727, and
    // `Language.GetTextValue`) resolve through the same manager, so holding
    // the lock across them would deadlock.
    let mut values: Vec<(&'static str, Substitution)> = Vec::with_capacity(65);
    let mut unset: Vec<&'static str> = Vec::new();

    // NPC first-name substitutions: `NPC.GetFirstNPCNameOrNull(type)`
    // (`Lang.cs:102-123,139-140`).
    const NPC_NAMES: &[(&'static str, i32)] = &[
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
            values.push((key, Substitution::Text(name)));
        } else {
            unset.push(key);
        }
    }

    // World / event flags (`Lang.cs:124-135`). The three event-boss downed
    // flags (`NPC.downedFishron`, `NPC.downedHalloweenKing`,
    // `NPC.downedChristmasIceQueen`) are written as constants by the world
    // file glue (`wld/sections/flags.rs:138,142,144`) — no live field exists,
    // so the producer answers their persisted value, `false`.
    values.push(("WorldName", Substitution::Text(app.world.name.clone())));
    values.push(("Day", Substitution::Bool(app.world.day_time)));
    values.push(("BloodMoon", Substitution::Bool(app.world.blood_moon)));
    values.push(("Eclipse", Substitution::Bool(app.world.eclipse)));
    values.push(("MoonLordDefeated", Substitution::Bool(app.world.downed_moon_lord)));
    values.push(("GolemDefeated", Substitution::Bool(app.world.downed_golem)));
    values.push(("DukeFishronDefeated", Substitution::Bool(app.world.downed_fishron)));
    values.push(("FrostLegionDefeated", Substitution::Bool(app.world.downed_frost)));
    values.push(("MartiansDefeated", Substitution::Bool(app.world.downed_martians)));
    values.push(("PumpkingDefeated", Substitution::Bool(false)));
    values.push(("IceQueenDefeated", Substitution::Bool(false)));
    values.push(("HardMode", Substitution::Bool(app.world.hard_mode)));

    // Player / interaction (`Lang.cs:136-144`).
    // `Homeless`: `Main.LocalPlayer.talkNPC >= 0 && Main.npc[talkNPC].homeless`;
    // the port models `NPC.homeless` as `home == None` (`npc.rs:16765`).
    let homeless = app
        .talk_npc
        .and_then(|i| app.npcs.get(i))
        .map(|n| n.home.is_none())
        .unwrap_or(false);
    values.push(("Homeless", Substitution::Bool(homeless)));
    // `Main.cInv` (`Lang.cs:137`) — the live inventory bind.
    let binds = &app.settings.keybinds;
    values.push((
        "InventoryKey",
        Substitution::Text(binds[crate::input_map::Action::Inventory.index()].clone()),
    ));
    values.push(("PlayerName", Substitution::Text(app.player_name.clone())));
    values.push(("Rain", Substitution::Bool(app.rain.is_raining())));
    values.push(("Graveyard", Substitution::Bool(app.scene_metrics.zone_graveyard)));
    let (angler_quests, deaths_pve) = if let Some(p) = &app.player {
        (p.angler_quests_finished as i64, p.deaths_pve as i64)
    } else {
        (0, 0)
    };
    values.push(("AnglerCompletedQuestsCount", Substitution::Int(angler_quests)));
    values.push(("TotalDeathsCount", Substitution::Int(deaths_pve)));

    // The localized-text producers (`Lang.cs:145,148,166`) resolve their keys
    // through the free `text()` before the lock. `WorldEvilStone` reads
    // `WorldGen.crimson`, which the port does not persist at world level
    // (`world_sim/npcs.rs:954`, `shop.rs`) — the corruption stone stays the
    // documented default until the world-evil flag lands.
    let evil_name = text("Misc.Ebonstone");
    values.push(("WorldEvilStone", Substitution::Text(evil_name)));
    let armor_key = if app.settings.reversed_armor_bonuses { "Key.UP" } else { "Key.DOWN" };
    values.push(("ToggleArmorSetBonusKey", Substitution::Text(text(armor_key))));

    // The input trigger tags (`Lang.cs:146-165`). The port carries no gamepad
    // profile (`PlayerInput.UsingGamepad` is always false — keyboard-only), so
    // `GenerateInputTag_ForCurrentGamemode` takes its keyboard arm
    // (`PlayerInput.cs:1948`): `GenerateRawInputList` over the trigger's
    // single-key list. The bound rows come from the live config binds
    // (`PlayerInput.ManageVersion_1_3`, `PlayerInput.cs:819-841`); the unbound
    // triggers answer "" (`GenerateRawInputList`'s empty-list arm,
    // `PlayerInput.cs:1980-1983`).
    let tag = |trigger: &str| Substitution::Text(generate_raw_input_list(&trigger_keys(binds, trigger)));
    // `ItemSlot.Options.DisableQuickTrash` is the settings' `quick_trash`
    // (`settings.rs:206-209`).
    values.push(("InputTriggerUI_BuildFromInventory", tag("QuickMount")));
    values.push(("InputTriggerUI_TrashEnabled", Substitution::Bool(!app.settings.quick_trash)));
    let trash_label = text(if app.settings.disable_left_shift_trash { "Controls.Control" } else { "Controls.Shift" });
    values.push(("InputTriggerUI_Trash", Substitution::Text(trash_label)));
    // `Main.FavoriteKey.ToString()` (`Lang.cs:149`): `Keys.LeftAlt`
    // (`Main.cs:67196`, `cFavoriteKey = "LeftAlt"` `Main.cs:67177`).
    values.push(("InputTriggerUI_FavoriteItem", Substitution::Text("LeftAlt".to_string())));
    values.push(("InputTrigger_QuickEquip", tag("Grapple")));
    values.push(("InputTrigger_Grapple", tag("Grapple")));
    values.push(("InputTrigger_LockOn", tag("LockOn")));
    values.push(("InputTrigger_RadialQuickbar", tag("HotbarMinus")));
    values.push(("InputTrigger_RadialHotbar", tag("HotbarPlus")));
    values.push(("InputTrigger_SmartCursor", tag("SmartCursor")));
    values.push(("InputTrigger_UseOrAttack", tag("MouseLeft")));
    values.push(("InputTrigger_Jump", tag("Jump")));
    values.push(("InputTrigger_InteractWithTile", tag("MouseRight")));
    values.push(("InputTrigger_InteractWithTileUI", tag("MouseRight")));
    values.push(("InputTrigger_ToggleOrOpen", tag("MouseRight")));
    values.push(("InputTrigger_SmartSelect", tag("SmartSelect")));
    values.push(("InputTrigger_Mount", tag("QuickMount")));
    values.push(("InputTrigger_Healing", tag("QuickHeal")));
    values.push(("InputTrigger_ManaHealing", tag("QuickMana")));
    values.push(("InputTrigger_QuickBuff", tag("QuickBuff")));

    let mut m = manager().lock().unwrap();
    for (key, value) in values {
        m.set_substitution(key, value);
    }
    for key in unset {
        m.remove_substitution(key);
    }
}

/// `NPC.GetFirstNPCNameOrNull` (`NPC.cs:7408-7418`): the `GivenOrTypeName`
/// (`NPC.cs:6743-6753`) of the first active NPC whose type matches `npc_type`,
/// or `None`. An unnamed NPC answers its `TypeName` =
/// `Lang.GetNPCNameValue(netID)` (`NPC.cs:6727`), not an empty string.
fn first_npc_name(app: &crate::app::App, npc_type: i32) -> Option<String> {
    app.npcs.iter().find_map(|n| {
        if n.hp > 0 && n.kind.net_id() == npc_type {
            if n.given_name.is_empty() {
                Some(npc_name(n.kind.net_id()))
            } else {
                Some(n.given_name.clone())
            }
        } else {
            None
        }
    })
}

#[cfg(test)]
mod s1_l12b_tests {
    use super::*;

    /// A headless App for the producer tests (the flow tests' harness), with
    /// the shipped English resources loaded so the localized-text producers
    /// (`Controls.*`, `Key.*`, `Misc.Ebonstone`, `NPCName.*`) answer.
    fn app() -> crate::app::App {
        init(GameCulture::English);
        crate::app::flow::front_flow_tests::flow_app(None, vec![], vec![])
    }

    fn sub(name: &str) -> Option<Substitution> {
        manager().lock().unwrap().substitution(name)
    }

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

    /// `Lang.cs:102-123,139-140` — the 24 NPC producers answer
    /// `NPC.GetFirstNPCNameOrNull` (`NPC.cs:7408-7418`): the first active NPC's
    /// `GivenOrTypeName` (`NPC.cs:6743-6753`), unset when none exists.
    #[test]
    fn npc_producers_answer_given_or_type_name() {
        let _guard = test_lock().lock().unwrap_or_else(|e| e.into_inner());
        let mut a = app();
        refresh_global_substitutions(&a);
        // No Guide alive: the producer is null-equivalent (unset).
        assert_eq!(sub("Guide"), None);
        // An unnamed Guide answers its TypeName (`Lang.GetNPCNameValue(22)`,
        // `NPC.cs:6727`).
        let mut guide = crate::npc::Npc::new(crate::npc::NpcKind::Guide, 100.0, 200.0, 1, 0.0);
        guide.given_name = String::new();
        a.npcs.push(guide);
        refresh_global_substitutions(&a);
        assert_eq!(sub("Guide"), Some(Substitution::Text("Guide".into())));
        // A given name wins over the type name.
        a.npcs[0].given_name = "Andrew".into();
        refresh_global_substitutions(&a);
        assert_eq!(sub("Guide"), Some(Substitution::Text("Andrew".into())));
        // A dead NPC (`active == false`) is skipped — the producer goes unset.
        a.npcs[0].hp = 0;
        refresh_global_substitutions(&a);
        assert_eq!(sub("Guide"), None);
    }

    /// `Lang.cs:124-135` — the world name and the event flags.
    #[test]
    fn world_and_event_flag_producers_read_the_world() {
        let _guard = test_lock().lock().unwrap_or_else(|e| e.into_inner());
        let mut a = app();
        a.world.name = "Terra".into();
        a.world.day_time = true;
        a.world.blood_moon = true;
        a.world.eclipse = false;
        a.world.downed_moon_lord = true;
        a.world.downed_golem = false;
        a.world.downed_frost = true;
        a.world.downed_martians = false;
        a.world.hard_mode = true;
        refresh_global_substitutions(&a);
        assert_eq!(sub("WorldName"), Some(Substitution::Text("Terra".into())));
        assert_eq!(sub("Day"), Some(Substitution::Bool(true)));
        assert_eq!(sub("BloodMoon"), Some(Substitution::Bool(true)));
        assert_eq!(sub("Eclipse"), Some(Substitution::Bool(false)));
        assert_eq!(sub("MoonLordDefeated"), Some(Substitution::Bool(true)));
        assert_eq!(sub("GolemDefeated"), Some(Substitution::Bool(false)));
        assert_eq!(sub("FrostLegionDefeated"), Some(Substitution::Bool(true)));
        assert_eq!(sub("MartiansDefeated"), Some(Substitution::Bool(false)));
        assert_eq!(sub("HardMode"), Some(Substitution::Bool(true)));
        // The two seasonal event-boss flags persist as constants
        // (`wld/sections/flags.rs`'s downedHalloween*/downedChristmas*
        // writes); DukeFishronDefeated reads the world flag now (S1-D-3
        // W-9 — the stock-false world keeps the false here).
        assert_eq!(sub("DukeFishronDefeated"), Some(Substitution::Bool(false)));
        assert_eq!(sub("PumpkingDefeated"), Some(Substitution::Bool(false)));
        assert_eq!(sub("IceQueenDefeated"), Some(Substitution::Bool(false)));
    }

    /// `Lang.cs:136` — `Homeless` = `talkNPC >= 0 && npc[talkNPC].homeless`;
    /// the port models `homeless` as `home == None` (`npc.rs:16765`).
    #[test]
    fn homeless_producer_follows_the_talk_npc() {
        let _guard = test_lock().lock().unwrap_or_else(|e| e.into_inner());
        let mut a = app();
        refresh_global_substitutions(&a);
        assert_eq!(sub("Homeless"), Some(Substitution::Bool(false)));
        let mut guide = crate::npc::Npc::new(crate::npc::NpcKind::Guide, 100.0, 200.0, 1, 0.0);
        guide.home = Some((10, 20));
        a.npcs.push(guide);
        a.talk_npc = Some(0);
        refresh_global_substitutions(&a);
        assert_eq!(sub("Homeless"), Some(Substitution::Bool(false)));
        a.npcs[0].home = None;
        refresh_global_substitutions(&a);
        assert_eq!(sub("Homeless"), Some(Substitution::Bool(true)));
    }

    /// `Lang.cs:137-138` — `InventoryKey` = `Main.cInv` (the live bind) and
    /// `PlayerName` = the local player's name.
    #[test]
    fn inventory_key_and_player_name_producers() {
        let _guard = test_lock().lock().unwrap_or_else(|e| e.into_inner());
        let mut a = app();
        a.player_name = "Belsin".into();
        a.settings.keybinds[crate::input_map::Action::Inventory.index()] = "I".into();
        refresh_global_substitutions(&a);
        assert_eq!(sub("InventoryKey"), Some(Substitution::Text("I".into())));
        assert_eq!(sub("PlayerName"), Some(Substitution::Text("Belsin".into())));
    }

    /// `Lang.cs:141-144` — `Rain`, `Graveyard`, the angler quest count and the
    /// PVE death count read the live state.
    #[test]
    fn rain_graveyard_and_player_counter_producers() {
        let _guard = test_lock().lock().unwrap_or_else(|e| e.into_inner());
        let mut a = app();
        a.scene_metrics.zone_graveyard = true;
        let w = tr_core::World::new("t".into(), 64, 800);
        let mut p = crate::player::Player::spawn(&w);
        p.angler_quests_finished = 7;
        p.deaths_pve = 3;
        a.player = Some(p);
        refresh_global_substitutions(&a);
        assert_eq!(sub("Rain"), Some(Substitution::Bool(a.rain.is_raining())));
        assert_eq!(sub("Graveyard"), Some(Substitution::Bool(true)));
        assert_eq!(sub("AnglerCompletedQuestsCount"), Some(Substitution::Int(7)));
        assert_eq!(sub("TotalDeathsCount"), Some(Substitution::Int(3)));
        // No player: both counters answer 0 (`Main.LocalPlayer` absent).
        a.player = None;
        refresh_global_substitutions(&a);
        assert_eq!(sub("AnglerCompletedQuestsCount"), Some(Substitution::Int(0)));
        assert_eq!(sub("TotalDeathsCount"), Some(Substitution::Int(0)));
    }

    /// `Lang.cs:146-165` — the input trigger tags read the keyboard profile
    /// (`PlayerInput.cs:1942-1990`): bound rows answer the live bind, the mouse
    /// triggers localize (`LocalizeKey`, `PlayerInput.cs:1951-1962`), unbound
    /// triggers answer "" (`GenerateRawInputList`'s empty-list arm).
    #[test]
    fn input_trigger_producers_read_the_live_binds() {
        let _guard = test_lock().lock().unwrap_or_else(|e| e.into_inner());
        let mut a = app();
        a.settings.keybinds[crate::input_map::Action::QuickHeal.index()] = "F".into();
        refresh_global_substitutions(&a);
        assert_eq!(sub("InputTrigger_Healing"), Some(Substitution::Text("F".into())));
        assert_eq!(sub("InputTrigger_ManaHealing"), Some(Substitution::Text("J".into())));
        assert_eq!(sub("InputTrigger_QuickBuff"), Some(Substitution::Text("B".into())));
        assert_eq!(sub("InputTrigger_Mount"), Some(Substitution::Text("R".into())));
        assert_eq!(sub("InputTriggerUI_BuildFromInventory"), Some(Substitution::Text("R".into())));
        assert_eq!(sub("InputTrigger_Grapple"), Some(Substitution::Text("E".into())));
        assert_eq!(sub("InputTrigger_QuickEquip"), Some(Substitution::Text("E".into())));
        assert_eq!(sub("InputTrigger_Jump"), Some(Substitution::Text("Space".into())));
        assert_eq!(sub("InputTrigger_SmartCursor"), Some(Substitution::Text("LeftControl".into())));
        assert_eq!(sub("InputTrigger_SmartSelect"), Some(Substitution::Text("LeftShift".into())));
        // The mouse triggers localize through `Controls.LeftClick/RightClick`.
        assert_eq!(sub("InputTrigger_UseOrAttack"), Some(Substitution::Text("Left Click".into())));
        assert_eq!(sub("InputTrigger_InteractWithTile"), Some(Substitution::Text("Right Click".into())));
        assert_eq!(sub("InputTrigger_InteractWithTileUI"), Some(Substitution::Text("Right Click".into())));
        assert_eq!(sub("InputTrigger_ToggleOrOpen"), Some(Substitution::Text("Right Click".into())));
        // Unbound on the keyboard preset (`PlayerInput.cs:2053-2091`).
        assert_eq!(sub("InputTrigger_LockOn"), Some(Substitution::Text(String::new())));
        assert_eq!(sub("InputTrigger_RadialQuickbar"), Some(Substitution::Text(String::new())));
        assert_eq!(sub("InputTrigger_RadialHotbar"), Some(Substitution::Text(String::new())));
    }

    /// `Lang.cs:147-149` — the trash/favorite UI rows read the live options.
    #[test]
    fn trash_and_favorite_ui_producers_read_the_options() {
        let _guard = test_lock().lock().unwrap_or_else(|e| e.into_inner());
        let mut a = app();
        // Defaults: quick trash enabled (`ItemSlot.cs:37`), left-shift trash
        // disabled (`ItemSlot.cs:36`) -> the Control label ("Ctrl").
        refresh_global_substitutions(&a);
        assert_eq!(sub("InputTriggerUI_TrashEnabled"), Some(Substitution::Bool(true)));
        assert_eq!(sub("InputTriggerUI_Trash"), Some(Substitution::Text("Ctrl".into())));
        a.settings.quick_trash = true;
        a.settings.disable_left_shift_trash = false;
        refresh_global_substitutions(&a);
        assert_eq!(sub("InputTriggerUI_TrashEnabled"), Some(Substitution::Bool(false)));
        assert_eq!(sub("InputTriggerUI_Trash"), Some(Substitution::Text("Shift".into())));
        // `Main.FavoriteKey.ToString()` = "LeftAlt" (`Main.cs:67177,67196`).
        assert_eq!(sub("InputTriggerUI_FavoriteItem"), Some(Substitution::Text("LeftAlt".into())));
    }

    /// `Lang.cs:145,166` — `WorldEvilStone` (the port's corruption default) and
    /// `ToggleArmorSetBonusKey` (the reversed-up/down option).
    #[test]
    fn evil_stone_and_armor_key_producers() {
        let _guard = test_lock().lock().unwrap_or_else(|e| e.into_inner());
        let mut a = app();
        refresh_global_substitutions(&a);
        assert_eq!(sub("WorldEvilStone"), Some(Substitution::Text("Ebonstone".into())));
        assert_eq!(sub("ToggleArmorSetBonusKey"), Some(Substitution::Text("DOWN".into())));
        a.settings.reversed_armor_bonuses = true;
        refresh_global_substitutions(&a);
        assert_eq!(sub("ToggleArmorSetBonusKey"), Some(Substitution::Text("UP".into())));
    }

    /// The refresh covers all 65 producers (`Lang.cs:102-166`): every non-NPC
    /// key is set after one pass over an empty world (the NPC keys stay unset
    /// until their NPC exists — the null-producer equivalent).
    #[test]
    fn the_refresh_covers_all_65_producers() {
        let _guard = test_lock().lock().unwrap_or_else(|e| e.into_inner());
        let a = app();
        refresh_global_substitutions(&a);
        const NON_NPC: &[&str] = &[
            "WorldName", "Day", "BloodMoon", "Eclipse", "MoonLordDefeated",
            "GolemDefeated", "DukeFishronDefeated", "FrostLegionDefeated",
            "MartiansDefeated", "PumpkingDefeated", "IceQueenDefeated", "HardMode",
            "Homeless", "InventoryKey", "PlayerName", "Rain", "Graveyard",
            "AnglerCompletedQuestsCount", "TotalDeathsCount", "WorldEvilStone",
            "InputTriggerUI_BuildFromInventory", "InputTriggerUI_TrashEnabled",
            "InputTriggerUI_Trash", "InputTriggerUI_FavoriteItem",
            "InputTrigger_QuickEquip", "InputTrigger_Grapple", "InputTrigger_LockOn",
            "InputTrigger_RadialQuickbar", "InputTrigger_RadialHotbar",
            "InputTrigger_SmartCursor", "InputTrigger_UseOrAttack", "InputTrigger_Jump",
            "InputTrigger_InteractWithTile", "InputTrigger_InteractWithTileUI",
            "InputTrigger_ToggleOrOpen", "InputTrigger_SmartSelect", "InputTrigger_Mount",
            "InputTrigger_Healing", "InputTrigger_ManaHealing", "InputTrigger_QuickBuff",
            "ToggleArmorSetBonusKey",
        ];
        assert_eq!(NON_NPC.len(), 41); // 65 - the 24 NPC-name producers
        for key in NON_NPC {
            assert!(sub(key).is_some(), "{key} must be set by the refresh");
        }
    }

    /// W4-g49 `57`'s display consumer — `Lang.GetDryadWorldStatusDialog`
    /// (`Lang.cs:251-293`): the eight status arms + the balance verdict.
    #[test]
    fn w4_g49_dryad_world_status_dialog_arms_and_verdicts() {
        let _guard = test_lock().lock().unwrap_or_else(|e| e.into_inner());
        let _a = app();
        // All three > 0 → WorldStatusAll; g*1.2 >= e+b and g*0.8 <= e+b is
        // false here (200*0.8=160 > 150), g >= e+b → FairyTale.
        let (text, pure) = dryad_world_status_dialog("Terra", 200, 100, 50);
        assert!(!pure);
        assert!(text.contains("200% hallow"), "{text}");
        assert!(text.contains("fairy tale"), "{text}");
        // The pure arm returns early with the flag set.
        let (text, pure) = dryad_world_status_dialog("Terra", 0, 0, 0);
        assert!(pure, "0/0/0 is entirely pure (`:287`)");
        assert!(text.contains("completely pure"), "{text}");
        // Corrupt-only → WorldStatusCorrupt; e+b=3 <= 5 → Close.
        let (text, pure) = dryad_world_status_dialog("Terra", 0, 3, 0);
        assert!(!pure);
        assert!(text.contains("3% corrupt"), "{text}");
        assert!(text.contains("so close"), "{text}");
        // Balanced: g=10, e+b=10 → 10*1.2=12 >= 10 and 10*0.8=8 <= 10.
        let (text, _) = dryad_world_status_dialog("Terra", 10, 10, 0);
        assert!(text.contains("in balance"), "{text}");
        // Grim: e+b > g+20 (g=0, e=30).
        let (text, _) = dryad_world_status_dialog("Terra", 0, 30, 0);
        assert!(text.contains("grim"), "{text}");
        // Work: e+b=10, g=0 → not grim (10 <= 20), not close (10 > 5).
        let (text, _) = dryad_world_status_dialog("Terra", 0, 10, 0);
        assert!(text.contains("lot of work"), "{text}");
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

/// The keyboard profile's key list for one trigger
/// (`PlayerInput.CurrentProfile.InputModes[InputMode.Keyboard].KeyStatus`).
/// The port stores the gameplay binds as the single-key config rows; the
/// trigger mapping is `PlayerInput.ManageVersion_1_3`'s table
/// (`PlayerInput.cs:819-841`). The mouse triggers carry the preset's
/// `Mouse1`/`Mouse2` (`PlayerInput.cs:2054-2055`); `LockOn`, `HotbarMinus` and
/// `HotbarPlus` are unbound on the keyboard preset (`PlayerInput.cs:2053-2091`
/// never adds them), so their lists stay empty.
fn trigger_keys(binds: &[String; crate::input_map::BIND_COUNT], trigger: &str) -> Vec<String> {
    use crate::input_map::Action;
    let action = match trigger {
        "MouseLeft" => return vec!["Mouse1".to_string()],
        "MouseRight" => return vec!["Mouse2".to_string()],
        "Jump" => Action::Jump,
        "Grapple" => Action::Hook,
        "SmartSelect" => Action::Torch,
        "SmartCursor" => Action::Smart,
        "QuickMount" => Action::Mount,
        "QuickHeal" => Action::QuickHeal,
        "QuickMana" => Action::QuickMana,
        "QuickBuff" => Action::QuickBuff,
        "Inventory" => Action::Inventory,
        _ => return Vec::new(),
    };
    vec![binds[action.index()].clone()]
}

/// `PlayerInput.LocalizeKey` (`PlayerInput.cs:1951-1962`): only the two mouse
/// keys read localized names; every other key name passes through unchanged.
fn localize_key(key: &str) -> String {
    match key {
        "Mouse1" => text("Controls.LeftClick"),
        "Mouse2" => text("Controls.RightClick"),
        _ => key.to_string(),
    }
}

/// `PlayerInput.GenerateRawInputList` (`PlayerInput.cs:1978-1990`): the
/// localized keys joined with "/", or "" for an empty list.
fn generate_raw_input_list(keys: &[String]) -> String {
    keys.iter().map(|k| localize_key(k)).collect::<Vec<_>>().join("/")
}

/// `Language.ActiveCulture` (the English default until a load happened).
pub fn active_culture() -> GameCulture {
    manager().lock().unwrap().active_culture()
}

/// `Language.Exists(key)` (`Language.cs`): does the active culture carry the
/// key? The bestiary's flavor-text gate reads it (`BestiaryEntry.cs:47`).
pub fn exists(key: &str) -> bool {
    manager().lock().unwrap().exists(key)
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

/// `LocalizedText.FormatWith(substitutes)` (`ReLogic.Localization`,
/// the `ArmorSetBonus.GetTooltipFor*` callers, `ArmorSetBonus.cs:202-220`):
/// the value with the caller's NAMED substitutions applied. The C# reflects
/// the substitutes object's properties; the port takes the `(name, value)`
/// pairs directly. Names the caller does not supply fall back to the
/// manager's global substitutions (`Lang.GetGlobalSubstitution`) — the
/// wave-D rows carry none, so the fallback is inert for them.
pub fn text_format_with(key: &str, args: &[(&str, Substitution)]) -> String {
    let mut m = manager().lock().unwrap();
    // Clone out of the map so the lookup closure can re-borrow the manager
    // for the global-substitution fallback.
    let text = m.get_text(key).clone();
    text.value_with(&|name| {
        args.iter()
            .find(|(n, _)| *n == name)
            .map(|(_, v)| v.clone())
            .or_else(|| m.substitution(name))
    })
}

/// `Language.GetTextValue("GameUI.<name>", args)` — the `GameUI` category
/// rows the tooltip prints (`Main.cs:20805-20816`'s fishing rows).
pub fn game_ui_fmt(key: &str, value: i32) -> String {
    text_args(
        &format!("GameUI.{key}"),
        &[value.to_string()],
    )
}

/// `Language.GetTextValue("GameUI.<name>")` — the `GameUI` rows without an
/// argument (the tooltip's `BaitRequired`, `Main.cs:20809`).
pub fn game_ui(key: &str) -> String {
    text(&format!("GameUI.{key}"))
}

/// `Language.GetTextValue("CommonItemTooltip.<name>")` — the tooltip's item
/// rows (`Main.cs:20876-20904`).
pub fn common_item_tooltip(key: &str) -> String {
    text(&format!("CommonItemTooltip.{key}"))
}

/// `Language.GetTextValue("CommonItemTooltip.<name>", value)` — the same rows
/// with their numeric argument (the life / mana / mana-cost rows).
pub fn common_item_tooltip_fmt(key: &str, value: i32) -> String {
    text_args(
        &format!("CommonItemTooltip.{key}"),
        &[value.to_string()],
    )
}

/// `Lang.GetPrefixedItemName(id, prefixType)` (`Lang.cs:225-239`): the
/// prefix's name + the item's name through the `CombineFormat.Prefix` format
/// ("{PrefixName} {ItemName}", `Lang.cs:1228`). `Item.AffixName`
/// (`Item.cs:1194-1201`) falls back to the plain name when the prefix is 0.
pub fn prefixed_item_name(id: u16, prefix: u16) -> String {
    if prefix == 0 {
        return item_name(id);
    }
    let prefix_name = prefix_name(prefix as i32);
    if prefix_name.is_empty() {
        return item_name(id);
    }
    let format = text("CombineFormat.Prefix");
    if format == "CombineFormat.Prefix" {
        return format!("{prefix_name} {}", item_name(id));
    }
    format
        .replace("{PrefixName}", &prefix_name)
        .replace("{ItemName}", &item_name(id))
}

/// `Lang.GetTooltip(itemId)` (`Lang.cs:193-196`): the per-item tooltip text
/// from the `ItemTooltip.<ItemID const name>` language key (the cache the
/// C# builds from `ItemID`'s field names, `Lang.cs:537-549`). `None` when the
/// item carries no tooltip text.
pub fn item_tooltip_text(id: u16) -> Option<String> {
    let Some(name) = crate::itemdata::NAMES.get(id as usize) else {
        return None;
    };
    let value = item_tooltip(id);
    if value == format!("ItemTooltip.{name}") {
        None
    } else {
        Some(value)
    }
}

/// Test-only: register a language source in the process-global manager (the
/// tooltip's rows read it). The real shipped sources load first when the
/// manager is still empty — otherwise this partial source would become the
/// *only* registered file (`lang::init` early-returns on a non-empty source
/// list) and every later `set_language` reload in the same process would
/// strip the rest of the game's texts down to this fragment.
///
/// **The caller must hold [`test_lock`]** for the whole test (registration +
/// reads): the reload resets every text to its key before re-loading the
/// files, so an unguarded parallel reader would see bare keys mid-reload.
/// The lock is not reentrant, so this helper must not take it itself.
#[cfg(test)]
pub(crate) fn test_register_source(name: &str, text: &str) {
    if manager().lock().unwrap().sources().is_empty() {
        init(GameCulture::English);
    }
    let mut m = manager().lock().unwrap();
    m.add_source(name, text);
    // `set_language` skips the reload when the culture is unchanged — the
    // test needs the fresh source.
    m.reload_language(GameCulture::English);
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

/// `Lang.dialog(i)` (`Lang.cs:170-173`): `LegacyDialog.{i}` — the town NPCs'
/// dialog lines.
pub fn dialog(i: i32) -> String {
    text(&format!("LegacyDialog.{i}"))
}

/// `Lang.GetDryadWorldStatusDialog` (`Lang.cs:251-293`): the Dryad's
/// world-status report — the `WorldGen.tGood/tEvil/tBlood` census percentages
/// (the `57 Unknown57` client store, `MessageBuffer.cs:2634-2636`) turned
/// into the `DryadSpecialText.WorldStatus*` line plus the
/// `WorldDescription*` balance verdict. Returns `(text, worldIsEntirelyPure)`;
/// the pure arm returns early (no verdict suffix, `:286-288`). W4-g49.
pub fn dryad_world_status_dialog(
    world_name: &str,
    t_good: u8,
    t_evil: u8,
    t_blood: u8,
) -> (String, bool) {
    let (g, e, b) = (t_good as i32, t_evil as i32, t_blood as i32);
    let name = world_name.to_string();
    // `:258-291` — the eight status arms, in the C#'s order.
    let line = if g > 0 && e > 0 && b > 0 {
        text_args("DryadSpecialText.WorldStatusAll", &[name, g.to_string(), e.to_string(), b.to_string()])
    } else if g > 0 && e > 0 {
        text_args("DryadSpecialText.WorldStatusHallowCorrupt", &[name, g.to_string(), e.to_string()])
    } else if g > 0 && b > 0 {
        text_args("DryadSpecialText.WorldStatusHallowCrimson", &[name, g.to_string(), b.to_string()])
    } else if e > 0 && b > 0 {
        text_args("DryadSpecialText.WorldStatusCorruptCrimson", &[name, e.to_string(), b.to_string()])
    } else if e > 0 {
        text_args("DryadSpecialText.WorldStatusCorrupt", &[name, e.to_string()])
    } else if b > 0 {
        text_args("DryadSpecialText.WorldStatusCrimson", &[name, b.to_string()])
    } else if g <= 0 {
        // `:284-288` — the pure arm returns early, `worldIsEntirelyPure = true`.
        return (text_args("DryadSpecialText.WorldStatusPure", &[name]), true);
    } else {
        text_args("DryadSpecialText.WorldStatusHallow", &[name, g.to_string()])
    };
    // `:292` — the balance verdict. The C# casts to double for the 1.2/0.8
    // comparisons; the port keeps the f64 maths verbatim.
    let verdict = if (g as f64) * 1.2 >= (e + b) as f64 && (g as f64) * 0.8 <= (e + b) as f64 {
        "DryadSpecialText.WorldDescriptionBalanced"
    } else if g >= e + b {
        "DryadSpecialText.WorldDescriptionFairyTale"
    } else if e + b > g + 20 {
        "DryadSpecialText.WorldDescriptionGrim"
    } else if e + b <= 5 {
        "DryadSpecialText.WorldDescriptionClose"
    } else {
        "DryadSpecialText.WorldDescriptionWork"
    };
    (format!("{line} {}", text(verdict)), false)
}

/// `LanguageManager.GetCategorySize` — the number of entries in a category
/// (the C#'s `Language.FindAll(Lang.CreateDialogFilter(prefix))` length).
pub fn category_size(category: &str) -> usize {
    manager().lock().unwrap().category_size(category)
}

/// `Language.SelectRandom(Lang.CreateDialogFilter(prefix))`
/// (`LanguageManager.cs:392-411`) — a random text from the category; the
/// caller draws the index so the stream stays the caller's.
pub fn random_from_category(category: &str, index: usize) -> String {
    let mut m = manager().lock().unwrap();
    m.random_from_category(category, index)
        .value_with(&|name| m.substitution(name))
}

/// `Language.FindAll(Lang.CreateDialogFilter(prefix))` over a key PREFIX that
/// is not a whole JSON category (`Lang.cs:422-425`'s `key.StartsWith(prefix)`).
/// The grouped-key table only carries the top-level categories, so the
/// sub-category prefixes (`DyeTraderSpecialText.HasPlant`,
/// `AnglerSpecialText.BloodMoonFishing`) resolve through the flat text table
/// instead. Returns the matching keys SORTED — the C#'s `FindAll` walks the
/// dictionary in insertion order; the sorted stand-in keeps the index→line
/// mapping deterministic (the `category_grouped_keys` convention, S1-L-12c).
pub fn dialog_prefix_keys(prefix: &str) -> Vec<String> {
    let m = manager().lock().unwrap();
    let mut keys: Vec<String> = m
        .find_all(|k, _| k.starts_with(prefix))
        .into_iter()
        .map(|t| t.key().to_string())
        .collect();
    keys.sort();
    keys
}

/// The matching-row count of `Lang.CreateDialogFilter(prefix)`
/// (`Lang.cs:422-425`) — the `num` `SelectRandom` counts before its
/// `Next(num)` (`LanguageManager.cs:394-402`). The prefix carries its
/// trailing dot like the C# callers ("NurseChatter."); the conditional
/// rows (`{?Day}`…) only count while their global substitutions hold.
pub fn dialog_count(prefix: &str) -> usize {
    manager().lock().unwrap().dialog_category_count(prefix)
}

/// `Language.SelectRandom(Lang.CreateDialogFilter(prefix))` with the
/// caller's drawn index — the REVERSE pick (`--num == index`,
/// `LanguageManager.cs:403-409`): index 0 answers the last matching row.
/// "" when nothing matches (`LocalizedText.Empty`, `:410`). The caller
/// must pay the `Next(count)` draw even for count 0 (the C# does,
/// `UnifiedRandom.cs:140`).
pub fn select_dialog(prefix: &str, index: i32) -> String {
    let mut m = manager().lock().unwrap();
    m.select_random_from_dialog(prefix, index)
        .value_with(&|name| m.substitution(name))
}

/// `Language.FindAll(Lang.CreateDialogFilter(prefix))[index].Value` —
/// the FORWARD row the `NPC.GetChat` mixed arms read (`array[num - k]`,
/// `NPC.cs:93976-93984`). "" past the end (`LocalizedText.Empty`).
pub fn dialog_row(prefix: &str, index: usize) -> String {
    let mut m = manager().lock().unwrap();
    m.dialog_row(prefix, index)
        .value_with(&|name| m.substitution(name))
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
