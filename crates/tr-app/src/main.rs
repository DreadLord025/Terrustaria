//! Terraria-Rust: Vulkan renderer prototype for a native reimplementation.


mod accessory_defense;
mod ammo;
mod bow_launch;
mod gun_launch;
mod angler;
mod armor_sets;
mod app;
mod assets;
mod achievements;
mod achievements_data;
mod accessory_combat;
mod audio;
mod bank;
mod banners;
mod bestiary;
mod biome;
mod buff_effects;
mod buff_effects_misc;
mod buffs;
mod buff_effects_accessory;
mod bug_net;
mod chat;
mod chest;
mod collision;
mod combat_text;
#[cfg(feature = "debug")]
mod debug_panel;
mod death;
mod drawdata_cases_a;
mod drawdata_cases_b;
mod dust_lights;
mod dye;
mod emotes;
mod env;
mod entity_lights;
mod item_lights;
mod npc_lights;
mod player_lights;
mod proj_lights;
mod eoc_fight;
mod input_map;
mod inv;
mod luck;
mod wiring_stub;
mod yoyo_string;
mod item;
mod itemdata;
mod item_sorting;
mod sortdata;
mod lang;
mod lang_tables;
mod loot_db;
mod loot_db_gen;
mod map;
mod map_layers;
mod melee_swing;
mod menu;
mod minecart;
mod mount;
mod net;
mod npc;
mod npc_chat;
mod npc_spatial;
mod revenge;
mod npc_ai;
mod npc_ai_prime_limbs;
mod npc_ai_style64;
mod npc_ai_style70;
mod npc_ai_frost_boss_b;
mod npc_ai_frost_boss;
mod npc_ai_frost_mob_b;
mod npc_ai_frost_mob;
mod npc_ai_caster;
mod npc_ai_town;
mod npc_ai3_speeds;
// S1-D-3z (zombie fighter audit): the AI_003 obstacle scan tail
// (NPC.cs:59831-60284) + the Drippler's aiStyle-22 machine.
mod npc_ai_fighter_scan;
mod npc_ai_drippler;
mod npc_ai_poltergeist;
mod npc_ai_pumpkin_boss;
mod npc_ai_unicorn;
mod npc_ai_style10;
mod npc_ai_style14;
mod npc_ai_style16;
mod npc_ai_style24;
mod npc_ai_style28;
mod npc_ai_style29;
mod npc_ai_style33;
mod npc_ai_style37;
mod npc_ai_style39;
mod npc_ai_style68;
mod npc_ai_unicorn_b;
mod npc_ai_style27;
mod npc_ai_fishron;
// S1-D-3 empress: UNWIRED Empress of Light machine (aiStyle 120, kind
// 636) - the NpcKind row + dispatch arm are the wiring follow-up (see
// the module doc + docs/ANALYSIS.md "Empress of Light's machine").
mod npc_ai_empress;
mod npc_ai_plantera;
// S1-D-3-plantera-minions (rescoped): the hook-centroid anchor seam
// (NPC.cs:31743-31762), UNWIRED - the merge union feeds it into
// npc_ai_plantera.rs's hook_anchor parameter (docs/ANALYSIS.md "The
// Plantera hook-centroid anchor"). The 263/264/265 machines themselves
// ride the sibling npc_ai_style52/53/50 branches.
mod npc_ai_plantera_anchor;
mod npc_ai_style41;
// S1-D-3 (aiStyle-42): UNWIRED Lost Girl machine - the kind-table rows +
// dispatch arm are the family's merge step (see the module doc).
mod npc_ai_style42;
mod npc_ai_style43;
// S1-D-3 (aiStyle-44): UNWIRED flying-fish / flying-antlion swimmer
// family - the four kind rows + dispatch claim are the wiring round
// (see the module doc).
mod npc_ai_style44;
// S1-D-3 (aiStyle-46): UNWIRED Golem-head machine - the kind-arm swap +
// style-axis claim are the golem family's merge step (see the module doc).
mod npc_ai_style46;
mod npc_ai_style49;
mod npc_ai_style50;
mod npc_ai_style52;
mod npc_ai_style53;
mod npc_ai_style55;
mod npc_ai_style56;
mod npc_ai_style59;
// S1-D-3 (aiStyle-67): UNWIRED snail wall-crawler machine (359/360/655) -
// no NpcKind yet; the kind-wave wiring is the S1-D-3 merge step.
mod npc_ai_style67;
// S1-D-3 (aiStyle-66): UNWIRED crawling-critter family (Worm 357 /
// TruffleWorm 374 / GoldWorm 448 / 484-487 / Maggot 606) - the kind rows
// + style-axis arm are the family's merge step (see the module doc).
mod npc_ai_style66;
mod npc_ai_style72;
mod npc_ai_style73;
mod npc_ai_style74;
mod npc_ai_style75;
mod npc_ai_style86;
// W4-d32 style 87: UNWIRED Big Mimic machine (NPC.cs:37471-37826) -
// the 473-476 kinds + the axis arm wait on the roster wave (the
// style86 precedent; see the module doc).
mod npc_ai_style87;
// W4-d32 style 88: UNWIRED Mothron machine (NPC.cs:37827-38351) -
// the 477 kind + the axis arm wait on the roster wave (the style86
// precedent; see the module doc).
mod npc_ai_style88;
// W4-d32 style 89: UNWIRED Mothron Egg machine (NPC.cs:38352-38393) -
// the 478 kind + the axis arm wait on the roster wave (the style86
// precedent; see the module doc).
mod npc_ai_style89;
// W4-d32 style 90: UNWIRED Mothron Spawn machine (NPC.cs:38394-38628) -
// the 479 kind + the axis arm wait on the roster wave (the style86
// precedent; see the module doc).
mod npc_ai_style90;
// W4-d32 style 91: UNWIRED Granite Flyer machine (NPC.cs:38629-38809) -
// the 483 kind + the axis arm wait on the roster wave (the style86
// precedent; see the module doc).
mod npc_ai_style91;
mod npc_ai_wyvern;
mod npc_ai_style9;
mod npc_ai_style30;
mod npc_ai_style31;
mod npc_ai_style47;
mod npc_ai_bat;
mod npc_ai_eye;
mod npc_ai_golem;
mod npc_ai_slime;
// S1-D-3 cultist: UNWIRED Lunatic Cultist machine (AI_084, 439/440) -
// the live npc.rs::update_lunatic_cultist stub stays byte-identical
// until the wiring round (the npc_ai_fishron precedent; states 4/7/8
// deferred, see the module doc).
mod npc_ai_cultist;
// S1-D-3 (B-6a): UNWIRED pillar machine (aiStyle 94) - the dispatch swap
// + the checkDead intercept are the wiring wave; R72 wires the 629
// shield-chip seams (see the module doc).
mod npc_ai_pillar;
mod npc_ai_style71;
mod npc_target;
mod npc_ai_style21;
mod npc_buffs;
mod npc_findframe_a;
mod npc_findframe_b;
mod persist;
mod grapple;
mod gore_seam;
mod creative;
mod creative_sacrifices;
mod happiness;
mod hotbar;
mod hurt_modifiers;
// S1-P-i10: the melee on-hit debuff infliction (Player.StatusToNPC,
// Player.cs:6502-6617) — wired at world_sim/editing.rs's pre-strike site.
mod melee_on_hit;
mod player;
mod player_file;
mod potions;
mod prefix;
pub(crate) mod prefix_id;
pub(crate) mod projectile_id;
pub(crate) mod id_tables;
pub(crate) mod world_events;
mod projectile;
mod projectile_ai001_b;
mod projectile_ai_frostmoon;
mod projectile_ai_moonlord;


mod projectile_ai001_a;
mod special_arrows;
mod projectile_frost_shots;
mod projectile_ai001_c;
mod projectile_ai067_b;
mod projectile_ai026_a;
mod projectile_ai026_b;
mod projectile_ai026_c;
mod projectile_ai_misc;
// S1-P-i42 — the torch/glowstick/flare light-item projectile family: the
// aiStyle-14 glowsticks (50/53/473/515/870/1089) + the aiStyle-33 flares
// (163/310/1008-1011) + the fairy's aiStyle-170 hover. The rows land in
// projectile/data.rs; the dispatch arms (14-gate + 33) are wired in
// projectile.rs::update. Code-only mode — witness tests written not run.
mod projectile_ai_light_items;
// S1-P-i55 golf: the two golf projectile AIs (721/739-752 aiStyle 149,
// 722 aiStyle 150) + the club strike / tee placement / cup scoring,
// reusing tr-core's golf.rs + golf_state.rs (code-only round — the
// witness tests are WRITTEN NOT RUN).
mod projectile_ai_golf;
// S1-P-i16: the dart guns' projectile AIs (267/477/478/479/442 aiStyle 1,
// 443 aiStyle 77, 167-170 aiStyle 34, the 480 flame drop's dust census).
mod projectile_ai_darts;
// S1-P-i18 special bullets: the 11 aiStyle-1 bullet specials' per-type AI_001
// arms + HandleMovement bounces + Kill() death effects (36/89/104/207/242/
// 279/283/284/285/286/287), pure per-kind fns behind the `covers` gate.
mod projectile_special_bullets;
// S1-P-i28: the thrown-consumable projectile family's missing flight arms
// (aiStyle 68 molotov / 113 bone javelin / 36 bees / the 507 javelin AI_001
// arm) + the death hooks (water splash, beenade bee burst, molotov fires) and
// the on-hit debuffs. The dispatch claim + the death-hook drain are wired in
// projectile.rs (module doc + docs/ANALYSIS.md "The thrown consumables").
mod projectile_ai_thrown;
// S1-D-4 empress proj: the Empress of Light's five projectile AIs
// (872/873/874/919/923 — AI_173/171/none/179/180), pure per-kind fns behind
// a kind gate, UNWIRED spawner (code-only mode). R7's npc_ai_empress.rs
// remainder; the dispatch claim + the R7 request drain are the wiring
// follow-up (module doc + docs/ANALYSIS.md "S1-D-4 empress proj").
mod projectile_ai_empress;
// S1-P-i35: the fishing bobber's AI_061 flight/bite machine + the FishingCheck
// orchestration (the pure roll engine is tr_core::fishing).
mod projectile_bobber;
// S1-P-i06 — the spear family (aiStyle 19, `AI_019_Spears`): the held-spear
// thrust arm + its extension/retract curve and reach-ahead hitbox, plus the
// small sub-shot arms (the spore/cloud/bolt/snowflake/flake the spears spawn).
mod projectile_ai_spear;
mod rain;
// S1-P-i38: the recall / magic-mirror / ice-mirror / potion-of-return use
// path — ItemCheck's spawn-recall arms (Player.cs:42976-42999 / :43107-43140 /
// :43141-43172) + Spawn(RecallFromItem) (:37534-37710) + the spawn-position
// helpers (:37718-37991). Wired into update_editing (the boss-spawner seam).
mod recall_teleport;
mod regen;
mod research;
mod sand;
mod ambience;
mod ambient_sound;
mod capture;
mod chest_gen;
mod craft;
mod craft_filter;
mod registry;
mod variants;
mod render;
mod spawner;
mod tag_effects;
mod settings;
mod shop;
mod shimmer;
mod signs;
mod sound;
mod soundid;
mod test_world_helpers;
mod town;
mod town_arrival;
mod tracked_sound;
mod touch_damage;
mod ui;
mod world_sim;
mod world_stepper;

use std::path::PathBuf;

use anyhow::{Context, Result};
use winit::event_loop::{ControlFlow, EventLoop};

// The `App` lives in `app` (the struct, the event loop, the frame and the
// startup assembly); `main` only orchestrates the bootstrap steps.
fn main() -> Result<()> {
    env_logger::Builder::from_env(
        // The Vulkan loader logs a benign registry warning on every start when no
        // Vulkan SDK layer manifests are installed; keep it out of the console.
        env_logger::Env::default().default_filter_or("info,wgpu_hal::vulkan::instance=error"),
    )
    .init();
    // The release build has no console; leave evidence for any panic instead of
    // losing the message when the window closes.
    std::panic::set_hook(Box::new(|info| {
        let backtrace = std::backtrace::Backtrace::force_capture();
        let msg = format!("panic: {info}
{backtrace}
");
        let _ = std::fs::write("crash.log", &msg);
        eprintln!("{msg}");
    }));

    let args: Vec<String> = std::env::args().collect();
    // `TR_WORLD=<path>` is the explicit debug/test path, and a positional
    // argument still loads a `.wld` from the original game or a saved `.trwld`.
    // The production path has neither: it boots the player's own newest world
    // (`UIWorldSelect`'s list is what `WorldPlay` opens, BUG-05) and falls back
    // to the pre-baked `Terrustaria` stand-in only when the saves dir holds no
    // `.trwld` at all.
    let world_path = if let Some(p) = crate::env::world_path() {
        p
    } else {
        match args.get(1) {
            Some(p) => PathBuf::from(p),
            None => app::latest_own_world().unwrap_or(app::terrstaria_world()?),
        }
    };

    log::info!("world: {}", world_path.display());
    // The world plus its side sections — the `.wld`'s section payloads
    // (M6-SAVE-1) or the native `.trwld`'s v7-v10 records (M6-SAVE-1b/c/d) —
    // the loaded stores the `App` merges ahead of the load-time loot roll.
    let (world, sections) = app::load_world(&world_path)?;

    let content_dir = app::find_content_dir().context(
        "assets/Content not found: copy the retail Content directory next to the executable",
    )?;
    assets::set_content_dir(content_dir.clone());
    log::info!("content: {}", content_dir.display());
    let assets_dir = content_dir.join("Images");

    // `LoadSettings`'s `LanguageManager.SetLanguage(text)` (`Main.cs:4345`):
    // register the shipped resources and apply the saved culture before the
    // first menu draws. `TR_LANG` mirrors LaunchInitializer's `-lang` flag
    // (`LaunchInitializer.cs:120` sets the culture ahead of the config read).
    let settings = settings::Settings::load();
    let culture = crate::env::lang()
        .map(|name| tr_core::localization::GameCulture::from_name(&name))
        .unwrap_or_else(|| tr_core::localization::GameCulture::from_name(&settings.language));
    lang::init(culture);

    app::log_world_diagnostics(&world);

    // The decoded static atlas half and the retail font tables are carried onto
    // the App so the first world swap (`App::swap_world`) repacks them instead of
    // re-reading ~7 000 sheets and re-decoding ~38 636 glyphs (BUG-21: the
    // select-world Play froze for seconds on that work).
    let mut boot_static: Option<Vec<tr_render::atlas::AtlasImage>> = None;
    let mut boot_fonts: Option<assets::FontTables> = None;
    let (atlas, font_atlas, lookups, atlas_stats) =
        app::build_atlas_reusing_static(&world, &assets_dir, &mut boot_static, &mut boot_fonts)?;

    app::log_atlas_diagnostics(&atlas, &lookups);

    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);
    // The one `Settings::load()` of the process: the value read for the culture
    // above is the master copy `App` keeps and the menu clones, so every
    // consumer sees the same config (the owner's read-once rule).
    let mut app = app::App::new(
        settings,
        world,
        sections,
        world_path,
        atlas,
        font_atlas,
        lookups,
        atlas_stats,
    );
    // `MapHelper.LoadMap`: restore the explored map from the `.map` sidecar.
    app.static_sheets = boot_static;
    app.fonts_cache = boot_fonts;
    // `AchievementManager`'s constructor binds its save file
    // (`AchievementManager.cs:41-55`): the production bootstrap is the port's
    // equivalent, so the achievements save has a destination.
    app.bind_achievements_save_path();
    app.load_map();
    // The bind table the `config.json` we just read carries (`Main.cs:4582`,
    // `PlayerInput.CurrentProfile`'s re-seed at `:4602`).
    app.apply_settings();
    event_loop.run_app(&mut app)?;
    Ok(())
}
